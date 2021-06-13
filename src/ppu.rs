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
    pub struct PpuStatus: u8 {
        const IN_VBLANK = 0b10000000;
        const SPRITE_0_HIT = 0b01000000;
        const SPRITE_OVERFLOW = 0b00100000;
    }
}

#[derive(PartialEq, Eq)]
pub struct Ppu {
    memory: VideoMemoryBuffer,
    write_latch: WriteLatch,
    status: PpuStatus,

    current_oam_address: u8,
    oam_data: [u8; 256],
    control_flag: u8,
    mask: u8,

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
            control_flag: 0,

            x: 0,
            t: 0,
            v: 0,

            frame_buffer: [[0; 256]; 240],
            mask: 0,

            status: PpuStatus::empty(),

            current_scanline: 261,
        }
    }

    pub fn enter_vblank(&mut self) {
        self.status.insert(PpuStatus::IN_VBLANK);
    }

    pub fn exit_vblank(&mut self) {
        self.status.remove(PpuStatus::IN_VBLANK);
    }

    pub fn get_status(&self) -> PpuStatus {
        self.status
    }

    pub fn set_control_flag(&mut self, value: u8) {
        log_ppu!("Write $2000: {:#02X?}", value);
        self.control_flag = value;
        // dbg!(self.control_flag);
        self.t &= !0xc00;
        self.t |= (value as u16 & 0b11) << 10;
    }

    pub fn set_mask(&mut self, value: u8) {
        self.mask = value;
        // dbg!(self.mask);
    }

    pub fn clear_address_latch(&mut self) {
        self.write_latch = WriteLatch::Zero;
    }

    pub fn set_oam_address(&mut self, address: u8) {
        self.current_oam_address = address;
    }

    pub fn write_oam_data(&mut self, data: u8) {
        self.oam_data[self.current_oam_address as usize] = data;
        self.current_oam_address += 1;
    }

    pub fn write_data(&mut self, data: u8) {
        log_ppu!("Write $2007 {:#02X?} at {:#04X?}", data, self.v);

        self.memory[self.v as usize] = data;

        if self.control_flag & 0b00000100 != 0 {
            self.v = self.v.wrapping_add(32);
        } else {
            self.v = self.v.wrapping_add(1);
        }
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

    pub fn current_nametable_address(&self) -> usize {
        0x2000 + (self.control_flag as usize & 0b11) * 0x400
    }

    pub fn current_attribute_table_address(&self) -> usize {
        0x23c0 + (self.control_flag as usize & 0b11) * 0x400
    }

    pub fn get_oam_sprite_data(&self) -> Vec<SpriteData> {
        (0usize..=255)
            .step_by(4)
            .map(|index| {
                let y = self.oam_data[index];
                let x = self.oam_data[index + 3];

                let byte1 = self.oam_data[index + 1];
                let tile_pattern = if self.control_flag & 8 != 0 {
                    PatternTableSelection::Right
                } else {
                    PatternTableSelection::Left
                };

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
        log_ppu!("Write $2006: {:#02X?}", address);
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
        if self.control_flag & 16 == 0 {
            PatternTableSelection::Left
        } else {
            PatternTableSelection::Right
        }
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
                self.status.insert(PpuStatus::IN_VBLANK);

                if self.mask & 0b1000 == 0 {
                    self.current_scanline = (self.current_scanline + 1) % 262;

                    return;
                }
                self.v &= !0b111101111100000;
                self.v |= self.t & 0b111101111100000;
            }
            0..=239 => {
                if self.mask & 0b1000 == 0 {
                    self.current_scanline = (self.current_scanline + 1) % 262;

                    return;
                }

                let palette = self.get_color_palette();
                let start_fine_x = self.x;
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

                    let shift = 7 - self.x;

                    let bit = ((pattern1 >> shift) & 1) + ((pattern2 >> shift) & 1) * 2;

                    // dbg!(bit);

                    let color: u8 = if bit == 0 {
                        0xff
                    } else {
                        palette_value[bit as usize - 1]
                    };

                    self.frame_buffer[self.current_scanline as usize][target_x] = color;

                    // move x
                    if self.x == 7 {
                        self.x = 0;

                        if (self.v & 0x001F) == 31 {
                            self.v &= !0x001f;
                            self.v ^= 0x400;
                        } else {
                            self.v += 1;
                        }
                    } else {
                        self.x += 1;
                    }
                }

                self.x = start_fine_x;

                self.v &= !0b10000011111;
                self.v |= self.t & 0b10000011111;

                // dbg!(hex_string(self.v), binary_string(self.v));

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

    pub fn get_frame_buffer(&self) -> &[[u8; 256]; 240] {
        &self.frame_buffer
    }
}
