use std::ops::BitAnd;

use crate::log_ppu;

use bitflags::bitflags;

pub type VideoMemoryBuffer = [u8; 0x4000];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteLatch {
    One,
    Zero,
}

impl WriteLatch {
    fn flip(&mut self) {
        match self {
            WriteLatch::Zero => *self = WriteLatch::One,
            WriteLatch::One => *self = WriteLatch::Zero,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PatternTableSelection {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawPriority {
    Foreground,
    Background,
}

pub struct SpriteData {
    pub x: u8,
    pub y: u8,
    pub tile_number: u8,
    pub tile_pattern: PatternTableSelection,
    pub color_palette: u8,
    pub draw_priority: DrawPriority,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

bitflags! {
    pub struct PpuControl: u8 {
        const GENERATE_NMI_AT_VBLANK = 0b10000000;
        const SPRITE_8X16_MODE = 0b00100000;
        const BACKGROUND_PATTERN_TABLE_FLAG = 0b00010000;
        const SPRITE_PATTERN_TABLE_FLAG = 0b00001000;
        const ADDRESS_INCREMENT_FLAG = 0b00000100;
        const NAMETABLE_VERTICAL_FLAG = 0b00000010;
        const NAMETABLE_HORIZONTAL_FLAG = 0b00000001;
    }
}

bitflags! {
    pub struct PpuMask: u8 {
        const SHOW_SPRITES = 0b00010000;
        const SHOW_BACKGROUND = 0b00001000;
        const SHOW_LEFTMOST_SPRITES = 0b00000100;
        const SHOW_LEFTMOST_BACKGROUND = 0b00000010;
    }
}

impl PpuMask {
    fn is_rendering_enabled(&self) -> bool {
        self.contains(PpuMask::SHOW_SPRITES) || self.contains(PpuMask::SHOW_BACKGROUND)
    }
}

impl PpuControl {
    pub fn address_increment(&self) -> u16 {
        if self.contains(PpuControl::ADDRESS_INCREMENT_FLAG) {
            32
        } else {
            1
        }
    }

    pub fn background_pattern_table(&self) -> PatternTableSelection {
        if self.contains(PpuControl::BACKGROUND_PATTERN_TABLE_FLAG) {
            PatternTableSelection::Right
        } else {
            PatternTableSelection::Left
        }
    }

    pub fn sprite_pattern_table(&self) -> PatternTableSelection {
        if self.contains(PpuControl::SPRITE_PATTERN_TABLE_FLAG) {
            PatternTableSelection::Right
        } else {
            PatternTableSelection::Left
        }
    }
}

bitflags! {
    pub struct PpuStatus: u8 {
        const IN_VBLANK = 0b10000000;
        const SPRITE_0_HIT = 0b01000000;
        const SPRITE_OVERFLOW = 0b00100000;
    }
}

pub enum ScanlineEffect {
    EnterVblank,
}

#[derive(PartialEq, Eq)]
pub struct Ppu {
    memory: VideoMemoryBuffer,
    write_latch: WriteLatch,
    status: PpuStatus,

    current_oam_address: u8,
    oam_data: [u8; 256],
    control: PpuControl,
    mask: PpuMask,

    read_buffer: u8,

    x: u8,
    t: u16,
    v: u16,

    current_scanline: u32,

    frame_buffer: [[u8; 256]; 240],
}

pub struct ColorPalette {
    pub background: u8,
    pub background_color_set: [[u8; 3]; 4],
    pub sprite_color_set: [[u8; 3]; 4],
}

impl Ppu {
    pub fn new(memory: VideoMemoryBuffer) -> Ppu {
        Ppu {
            memory,
            write_latch: WriteLatch::Zero,
            current_oam_address: 0,
            oam_data: [0; 256],
            control: PpuControl::empty(),

            x: 0,
            t: 0,
            v: 0,

            read_buffer: 0,

            frame_buffer: [[0; 256]; 240],
            mask: PpuMask::empty(),

            status: PpuStatus::empty(),

            current_scanline: 261,
        }
    }

    pub fn get_status(&mut self) -> PpuStatus {
        self.status
    }

    pub fn set_control(&mut self, value: PpuControl) {
        log_ppu!("Write $2000: {:#010b}", value);
        self.control = value;
        self.t &= !0xc00;
        self.t |= (value.bits() as u16 & 0b11) << 10;
    }

    pub fn set_mask(&mut self, value: PpuMask) {
        log_ppu!("Write $2001: {:#08b}", value);
        self.mask = value;
        // dbg!(self.mask);
    }

    pub fn clear_address_latch(&mut self) {
        self.write_latch = WriteLatch::Zero;
    }

    pub fn set_oam_address(&mut self, address: u8) {
        log_ppu!("Write $2003: {:#04X}", address);
        self.current_oam_address = address;
    }

    pub fn write_oam_data(&mut self, data: u8) {
        log_ppu!("Write $2004: {:#04X}", data);
        self.oam_data[self.current_oam_address as usize] = data;
        self.current_oam_address += 1;
    }

    pub fn read_data(&mut self) -> u8 {
        // todo: palette read should not be buffered
        let last_buffer = self.read_buffer;
        log_ppu!("Read $2007 at {:#06X}: {:#04X?}", self.v, last_buffer);

        self.read_buffer = self.memory[self.v as usize];

        self.v = self.v.wrapping_add(self.control.address_increment());

        last_buffer
    }

    pub fn write_data(&mut self, data: u8) {
        log_ppu!("Write $2007 {:#02X?} at {:#04X?}", data, self.v);

        self.memory[self.v as usize] = data;

        self.v = self.v.wrapping_add(self.control.address_increment());
    }

    pub fn write_scroll(&mut self, position: u8) {
        log_ppu!("Write $2005({:?}): {:#02X?}", self.write_latch, position);
        match self.write_latch {
            WriteLatch::Zero => {
                self.x = position & 0b00000111;
                self.t &= !0b11111;
                self.t |= (position >> 3) as u16;
            }

            WriteLatch::One => {
                let coarse_y = (position >> 3) as u16;
                let fine_y = (position & 0b00000111) as u16;

                self.t &= !0b111001111100000;
                self.t |= coarse_y << 5 | fine_y << 12;
            }
        }

        self.write_latch.flip();
    }

    pub fn copy_oam_data(&mut self, data: &[u8]) {
        &self.oam_data.copy_from_slice(data);
    }

    pub fn get_oam_sprite_data(&self) -> Vec<SpriteData> {
        (0usize..=255)
            .step_by(4)
            .map(|index| {
                let y = self.oam_data[index];
                let x = self.oam_data[index + 3];

                let byte1 = self.oam_data[index + 1];
                let tile_pattern = self.control.sprite_pattern_table();

                let tile_number = byte1;

                let byte2 = self.oam_data[index + 2];
                let color_palette = byte2 & 0b00000011;

                let draw_priority = if byte2 & 0b00100000 == 0 {
                    DrawPriority::Foreground
                } else {
                    DrawPriority::Background
                };

                let flip_horizontal = byte2 & 0b01000000 != 0;
                let flip_vertical = byte2 & 0b10000000 != 0;

                SpriteData {
                    x,
                    y,
                    tile_pattern,
                    tile_number,
                    color_palette,
                    draw_priority,
                    flip_horizontal,
                    flip_vertical,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn write_address(&mut self, address: u8) {
        log_ppu!("Write $2006: {:#04X?}", address);
        match self.write_latch {
            WriteLatch::Zero => {
                let value = address & 0b00111111;
                self.t &= 0xff;
                self.t |= (value as u16) << 8;
            }

            WriteLatch::One => {
                self.t &= 0xff00;
                self.t |= address as u16;
                self.v = self.t;
            }
        }

        self.write_latch.flip();
    }

    pub fn get_buffer(&self) -> &VideoMemoryBuffer {
        &self.memory
    }

    pub fn left_pattern_table(&self) -> &[u8] {
        &self.memory[0..0x1000]
    }

    pub fn right_pattern_table(&self) -> &[u8] {
        &self.memory[0x1000..0x2000]
    }

    pub fn current_background_pattern_table(&self) -> PatternTableSelection {
        self.control.background_pattern_table()
    }

    pub fn get_color_palette(&self) -> ColorPalette {
        ColorPalette {
            background: self.memory[0x3f00],
            background_color_set: [
                [
                    self.memory[0x3f01],
                    self.memory[0x3f02],
                    self.memory[0x3f03],
                ],
                [
                    self.memory[0x3f05],
                    self.memory[0x3f06],
                    self.memory[0x3f07],
                ],
                [
                    self.memory[0x3f09],
                    self.memory[0x3f0a],
                    self.memory[0x3f0b],
                ],
                [
                    self.memory[0x3f0d],
                    self.memory[0x3f0e],
                    self.memory[0x3f0f],
                ],
            ],
            sprite_color_set: [
                [
                    self.memory[0x3f11],
                    self.memory[0x3f12],
                    self.memory[0x3f13],
                ],
                [
                    self.memory[0x3f15],
                    self.memory[0x3f16],
                    self.memory[0x3f17],
                ],
                [
                    self.memory[0x3f19],
                    self.memory[0x3f1a],
                    self.memory[0x3f1b],
                ],
                [
                    self.memory[0x3f1d],
                    self.memory[0x3f1e],
                    self.memory[0x3f1f],
                ],
            ],
        }
    }

    pub fn get_current_scanline(&self) -> u32 {
        self.current_scanline
    }

    pub fn advance_scanline(&mut self) {
        match self.current_scanline {
            261 => {
                self.status.remove(PpuStatus::IN_VBLANK);
                self.status.remove(PpuStatus::SPRITE_0_HIT);

                if !self.mask.is_rendering_enabled() {
                    self.current_scanline = (self.current_scanline + 1) % 262;

                    return;
                }
                self.v &= !0b111101111100000;
                self.v |= self.t & 0b111101111100000;
            }
            0..=239 => {
                if !self.mask.is_rendering_enabled() {
                    self.current_scanline = (self.current_scanline + 1) % 262;

                    return;
                }

                let palette = self.get_color_palette();
                let mut fine_x = self.x;
                let fine_y = (self.v & 0x7000) >> 12;
                for target_x in 0..256 {
                    // render
                    let tile_address = 0x2000 | (self.v & 0xfff);

                    let coarse_x = self.v & 0b11111;
                    let coarse_y = (self.v >> 5) & 0b11111;
                    let tile_value = self.memory[tile_address as usize];

                    let attribute_address = 0x23C0
                        | (self.v & 0x0C00)
                        | ((self.v >> 4) & 0x38)
                        | ((self.v >> 2) & 0x07);
                    let attribute_value = self.memory[attribute_address as usize];
                    let subtile_y = (coarse_y % 4) / 2;
                    let subtile_x = (coarse_x % 4) / 2;

                    let palette_set_index = match (subtile_x, subtile_y) {
                        (0, 0) => attribute_value & 0b11,
                        (1, 0) => attribute_value.bitand(0b1100 as u8) >> 2,
                        (0, 1) => attribute_value.bitand(0b110000 as u8) >> 4,
                        (1, 1) => attribute_value.bitand(0b11000000 as u8) >> 6,
                        _ => panic!("Impossible subtile location!"),
                    };

                    let palette_value = palette.background_color_set[palette_set_index as usize];

                    // dbg!(
                    //     // hex_string(tile_address),
                    //     // tile_value,
                    //     coarse_x, // self.x,
                    //     coarse_y, fine_y
                    // );

                    let pattern_address = tile_value as u16 * 0x10 + fine_y;

                    let pattern_table = match self.current_background_pattern_table() {
                        PatternTableSelection::Left => self.left_pattern_table(),
                        PatternTableSelection::Right => self.right_pattern_table(),
                    };

                    let pattern1 = pattern_table[pattern_address as usize];
                    let pattern2 = pattern_table[pattern_address as usize + 8];

                    let shift = 7 - fine_x;

                    let bit = ((pattern1 >> shift) & 1) + ((pattern2 >> shift) & 1) * 2;

                    // dbg!(bit);

                    let color: u8 = if bit == 0 {
                        0xff
                    } else {
                        palette_value[bit as usize - 1]
                    };

                    self.frame_buffer[self.current_scanline as usize][target_x] = color;

                    // move x
                    if fine_x == 7 {
                        fine_x = 0;

                        if (self.v & 0x001F) == 31 {
                            self.v &= !0x001f;
                            self.v ^= 0x400;
                        } else {
                            self.v += 1;
                        }
                    } else {
                        fine_x += 1;
                    }
                }

                self.v &= !0b10000011111;
                self.v |= self.t & 0b10000011111;

                // dbg!(hex_string(self.v), binary_string(self.v));

                self.toggle_sprite_0_hit_if_needed();

                if self.v & 0x7000 != 0x7000 {
                    self.v += 0x1000;
                } else {
                    self.v &= !0x7000;
                    let mut y = (self.v & 0x03E0) >> 5;

                    if y == 29 {
                        y = 0;
                        self.v ^= 0x0800;
                    } else if y == 31 {
                        y = 0;
                    } else {
                        y += 1;
                    }

                    self.v = (self.v & !0x03E0) | (y << 5);
                }
            }
            240 => {}
            241 => self.status.insert(PpuStatus::IN_VBLANK),
            242..=260 => {}
            _ => panic!("Unhandled scanline: {}", self.current_scanline),
        }

        self.current_scanline = (self.current_scanline + 1) % 262;
    }

    fn toggle_sprite_0_hit_if_needed(&mut self) {
        if !self.status.contains(PpuStatus::SPRITE_0_HIT) {
            // todo: handle 8 x 16 sprites

            let sprite_y = self.oam_data[0] as u32;
            let sprite_x = self.oam_data[3];

            if self.current_scanline < sprite_y || self.current_scanline > sprite_y + 7 {
                return;
            }

            let sprite_fine_y = self.current_scanline - sprite_y;

            let pattern_table =
                if self.control.background_pattern_table() == PatternTableSelection::Right {
                    self.right_pattern_table()
                } else {
                    self.left_pattern_table()
                };

            let tile = self.oam_data[1];
            let pattern_row = tile as usize * 0x10 + sprite_fine_y as usize;
            let left_tile = pattern_table[pattern_row];
            let right_tile = pattern_table[pattern_row + 8];

            for i in 0..8 {
                let x = sprite_x + i;

                if self.frame_buffer[self.current_scanline as usize][x as usize] != 0xff
                    && (left_tile & (1 << x) != 0 || right_tile & (1 << x) != 0)
                {
                    self.status.insert(PpuStatus::SPRITE_0_HIT);
                }
            }
        }
    }

    pub fn get_frame_buffer(&self) -> &[[u8; 256]; 240] {
        &self.frame_buffer
    }

    pub fn generates_nmi_at_vblank(&self) -> bool {
        self.control.contains(PpuControl::GENERATE_NMI_AT_VBLANK)
    }
}
