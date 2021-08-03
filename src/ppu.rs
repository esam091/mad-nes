use std::{
    cell::{Ref, RefCell},
    convert::TryInto,
    ops::{BitAnd, Shr},
    rc::Rc,
};

use crate::{ines::Cartridge, log_ppu};

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PatternTableSelection {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrawPriority {
    Foreground,
    Background,
}

#[derive(Debug)]
pub struct SpriteData {
    pub x: u8,
    pub y: u8,
    pub tile_number: u8,
    pub tile_pattern: PatternTableSelection,
    pub color_palette: u8,
    pub draw_priority: DrawPriority,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub drawing_mode: SpriteDrawingMode,
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
        const UNUSED_MASTER_SLAVE_SELLECT = 0b01000000;
    }
}

impl PpuControl {
    pub fn drawing_mode(&self) -> SpriteDrawingMode {
        if self.contains(PpuControl::SPRITE_8X16_MODE) {
            SpriteDrawingMode::Draw8x16
        } else {
            SpriteDrawingMode::Draw8x8
        }
    }

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
    pub struct PpuMask: u8 {
        const GREYSCALE = 0b00000001;
        const SHOW_SPRITES = 0b00010000;
        const SHOW_BACKGROUND = 0b00001000;
        const SHOW_LEFTMOST_SPRITES = 0b00000100;
        const SHOW_LEFTMOST_BACKGROUND = 0b00000010;
        const EMPHASIZE_RED = 0b10000000;
        const EMPHASIZE_GREEN = 0b01000000;
        const EMPHASIZE_BLUE = 0b00100000;
    }
}

impl PpuMask {
    fn is_rendering_enabled(&self) -> bool {
        self.contains(PpuMask::SHOW_SPRITES) || self.contains(PpuMask::SHOW_BACKGROUND)
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    OneScreenLow,
    OneScreenHigh,
}

impl Mirroring {
    fn real_address(&self, address: u16) -> u16 {
        match self {
            Mirroring::Vertical => (address & 0x23ff) | (address & 0x400),
            Mirroring::Horizontal => (address & 0x23ff) | ((address / 2) & 0x400),
            Mirroring::OneScreenLow => address & 0x23ff,
            Mirroring::OneScreenHigh => (address & 0x23ff) | 0x400,
        }
    }
}

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
    current_dot: u32,
    current_fine_x: u8,

    frame_buffer: [[u8; 256]; 240],
    foreground_sprite_buffer: [[u8; 256]; 240],
    background_sprite_buffer: [[u8; 256]; 240],

    cartridge: Rc<RefCell<Cartridge>>,
}

pub struct PatternTableRef<'a> {
    cartridge: Ref<'a, Cartridge>,
    left_vram: &'a [u8],
    right_vram: &'a [u8],
}

pub struct ColorPalette {
    pub background: u8,
    pub background_color_set: [[u8; 3]; 4],
    pub sprite_color_set: [[u8; 3]; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpriteDrawingMode {
    Draw8x8,
    Draw8x16,
}

fn map_mirror(address: u16) -> u16 {
    match address {
        0x3000..=0x3eff => address - 0x1000,
        0x3f20..=0x3fff => (address - 0x3f00) % 0x20 + 0x3f00,
        _ => address,
    }
}

impl Ppu {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Ppu {
        Ppu {
            memory: [0; 0x4000],
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
            current_dot: 0,
            current_fine_x: 0,

            cartridge,

            foreground_sprite_buffer: [[0xff; 256]; 240],
            background_sprite_buffer: [[0xff; 256]; 240],
        }
    }

    pub fn get_status(&mut self) -> PpuStatus {
        self.status
    }

    pub fn set_control(&mut self, value: PpuControl) {
        log_ppu!(
            "[{:#03}] Write $2000: {:#010b}",
            self.current_scanline,
            value
        );
        self.control = value;
        self.t &= !0xc00;
        self.t |= (value.bits() as u16 & 0b11) << 10;
    }

    pub fn set_mask(&mut self, value: PpuMask) {
        log_ppu!(
            "[{:#03}] Write $2001: {:#010b}",
            self.current_scanline,
            value
        );
        self.mask = value;
        // dbg!(self.mask);
    }

    pub fn read_status(&mut self) -> u8 {
        log_ppu!(
            "[{:#03}] Read $2002: {:#010b}",
            self.current_scanline,
            self.status
        );
        self.clear_address_latch();
        let bits = self.status.bits();
        self.status.remove(PpuStatus::IN_VBLANK);
        bits
    }

    pub fn clear_address_latch(&mut self) {
        self.write_latch = WriteLatch::Zero;
    }

    pub fn set_oam_address(&mut self, address: u8) {
        log_ppu!(
            "[{:#03}] Write $2003: {:#04X}",
            self.current_scanline,
            address
        );
        self.current_oam_address = address;
    }

    pub fn write_oam_data(&mut self, data: u8) {
        log_ppu!("[{:#03}] Write $2004: {:#04X}", self.current_scanline, data);
        self.oam_data[self.current_oam_address as usize] = data;
        self.current_oam_address += 1;
    }

    pub fn read_oam_data(&self) -> u8 {
        log_ppu!(
            "[{:#03}] Read $2004: {:#04X}",
            self.current_scanline,
            self.oam_data[self.current_oam_address as usize]
        );
        self.oam_data[self.current_oam_address as usize]
    }

    pub fn read_data(&mut self) -> u8 {
        let last_buffer = self.read_buffer;
        let real_address = map_mirror(self.v);

        log_ppu!(
            "Read $2007 at {:#06X} ({:#06X}): {:#04X?}",
            self.v,
            real_address,
            last_buffer
        );

        if self.v >= 0x3f00 {
            let value = self.memory[real_address as usize];
            self.read_buffer = self.memory[self.v as usize - 0x1000];

            self.v = self.v.wrapping_add(self.control.address_increment());

            value
        } else {
            let cartridge = self.cartridge.borrow();
            let value = if real_address < 0x2000 {
                cartridge
                    .read_chr_rom(real_address)
                    .unwrap_or(self.memory[real_address as usize])
            } else {
                let real_address = self
                    .cartridge
                    .borrow()
                    .mirroring()
                    .real_address(real_address);
                self.memory[real_address as usize]
            };

            self.read_buffer = value;
            self.v = self.v.wrapping_add(self.control.address_increment());

            last_buffer
        }
    }

    pub fn write_data(&mut self, data: u8) {
        log_ppu!(
            "[{:03}] Write $2007 {:#04X?} at {:#06X?}",
            self.current_scanline,
            data,
            self.v
        );

        let real_address = map_mirror(self.v);

        self.memory[real_address as usize] = data;

        if real_address == 0x3f00 {
            self.memory[0x3f10] = data;
        } else if real_address == 0x3f10 {
            self.memory[0x3f00] = data;
        }

        if real_address < 0x3f00 && real_address >= 0x2000 {
            let real_address = self
                .cartridge
                .borrow()
                .mirroring()
                .real_address(real_address);
            self.memory[real_address as usize] = data;
        }

        self.v = self.v.wrapping_add(self.control.address_increment());
    }

    pub fn write_scroll(&mut self, position: u8) {
        log_ppu!(
            "[{:#03}] Write $2005({:?}): {:#02X?}",
            self.current_scanline,
            self.write_latch,
            position
        );
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
        for index in 0..=255 {
            self.oam_data[self.current_oam_address.wrapping_add(index) as usize] =
                data[index as usize];
        }
    }

    fn get_oam_sprite_data_at(&self, index: usize) -> SpriteData {
        let drawing_mode = self.control.drawing_mode();
        let base_index = (self.current_oam_address as usize + index) % 256;
        let y = self.oam_data[base_index];
        let x = self.oam_data[(base_index + 3) % 256];

        let byte1 = self.oam_data[(base_index + 1) % 256];
        let tile_pattern = if drawing_mode == SpriteDrawingMode::Draw8x8 {
            self.control.sprite_pattern_table()
        } else {
            if byte1 & 1 == 0 {
                PatternTableSelection::Left
            } else {
                PatternTableSelection::Right
            }
        };

        let tile_number = if drawing_mode == SpriteDrawingMode::Draw8x8 {
            byte1
        } else {
            byte1 & !1
        };

        let byte2 = self.oam_data[(base_index + 2) % 256];
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
            drawing_mode,
        }
    }

    pub fn get_all_oam_sprite_data(&self) -> Vec<SpriteData> {
        (0usize..=255)
            .step_by(4)
            .map(|index| self.get_oam_sprite_data_at(index))
            .collect::<Vec<_>>()
    }

    pub fn write_address(&mut self, address: u8) {
        log_ppu!(
            "[{:#03}] Write $2006: {:#04X?}",
            self.current_scanline,
            address
        );
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

                if self.v >= 0x1000 && self.v < 0x2000 {
                    // println!("Clock counter!");
                    self.cartridge.borrow_mut().scanline_tick();
                }
            }
        }

        self.write_latch.flip();
    }

    fn read_pattern_at_address(&self, address: u16) -> u8 {
        self.cartridge
            .borrow()
            .read_chr_rom(address)
            .unwrap_or(self.memory[address as usize])
    }

    pub fn read_pattern_value(
        &self,
        pattern_selection: PatternTableSelection,
        tile_number: u8,
        x: u8,
        y: u16,
    ) -> u8 {
        let mut address: u16 = 0;

        if pattern_selection == PatternTableSelection::Right {
            address += 0x1000;
        }

        address += tile_number as u16 * 0x10 + y as u16;

        let pattern1 = self.read_pattern_at_address(address);
        let pattern2 = self.read_pattern_at_address(address + 8);

        let shift = 7 - x;

        ((pattern1 >> shift) & 1) + ((pattern2 >> shift) & 1) * 2
    }

    pub fn get_buffer(&self) -> &VideoMemoryBuffer {
        &self.memory
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

    pub fn step(&mut self) -> bool {
        let mut should_render = false;
        match (self.current_scanline, self.current_dot) {
            (261, 1) => {
                self.status.remove(PpuStatus::IN_VBLANK);
                self.status.remove(PpuStatus::SPRITE_0_HIT);

                if self.mask.is_rendering_enabled() {
                    self.v &= !0b111101111100000;
                    self.v |= self.t & 0b111101111100000;
                }
            }
            (0..=239, 0..=255) => {
                if self.mask.is_rendering_enabled() {
                    let palette = self.get_color_palette();

                    let fine_y = (self.v & 0x7000) >> 12;
                    // render
                    let tile_address = 0x2000 | (self.v & 0xfff);
                    let tile_address = self
                        .cartridge
                        .borrow()
                        .mirroring()
                        .real_address(tile_address);

                    let coarse_x = self.v & 0b11111;
                    let coarse_y = (self.v >> 5) & 0b11111;
                    let tile_value = self.memory[tile_address as usize];

                    let attribute_address = 0x23C0
                        | (self.v & 0x0C00)
                        | ((self.v >> 4) & 0x38)
                        | ((self.v >> 2) & 0x07);
                    let attribute_address = self
                        .cartridge
                        .borrow()
                        .mirroring()
                        .real_address(attribute_address);

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

                    let bit = self.read_pattern_value(
                        self.current_background_pattern_table(),
                        tile_value,
                        self.current_fine_x,
                        fine_y,
                    );

                    // dbg!(bit);

                    let color: u8 = if bit == 0
                        || (self.current_dot < 8
                            && !self.mask.contains(PpuMask::SHOW_LEFTMOST_BACKGROUND))
                    {
                        0xff
                    } else {
                        palette_value[bit as usize - 1]
                    };

                    self.frame_buffer[self.current_scanline as usize][self.current_dot as usize] =
                        color;

                    // move x
                    if self.current_fine_x == 7 {
                        self.current_fine_x = 0;

                        if (self.v & 0x001F) == 31 {
                            self.v &= !0x001f;
                            self.v ^= 0x400;
                        } else {
                            self.v += 1;
                        }
                    } else {
                        self.current_fine_x += 1;
                    }
                }
            }
            (0..=239, 256) => {
                if self.is_background_rendering_enabled() {
                    self.v &= !0b10000011111;
                    self.v |= self.t & 0b10000011111;

                    self.current_fine_x = self.x;
                }
            }
            (0..=239, 257) => {
                self.toggle_sprite_0_hit_if_needed();

                if self.is_background_rendering_enabled() {
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
            }
            (241, 1) => {
                self.render_sprites();
                self.status.insert(PpuStatus::IN_VBLANK);
                should_render = true
            }
            (0..=239, 260) => self.scanline_tick_if_possible(),
            (261, 260) => self.scanline_tick_if_possible(),
            _ => {}
        }

        if self.current_dot == 340 {
            self.current_scanline = (self.current_scanline + 1) % 262;
            self.current_dot = 0;
        } else {
            self.current_dot += 1;
        }

        should_render
    }

    fn render_sprites(&mut self) {
        self.foreground_sprite_buffer = [[0xff; 256]; 240];
        self.background_sprite_buffer = [[0xff; 256]; 240];

        let sprites = self.get_all_oam_sprite_data();
        let sprite_height = if self.control.contains(PpuControl::SPRITE_8X16_MODE) {
            15
        } else {
            7
        };

        let mut secondary_oam_indexes: Vec<usize> = Vec::new();
        let palette = self.get_color_palette().sprite_color_set;

        // handle flips horizontal, flip vertical, 8x16
        // dbg!(&sprites);
        for y in 1..240 {
            secondary_oam_indexes.clear();
            for index in 0..sprites.len() {
                let sprite = &sprites[index];

                if sprite.y as u16 + 1 <= y && y <= sprite.y as u16 + 1 + sprite_height {
                    secondary_oam_indexes.push(index);
                }

                if secondary_oam_indexes.len() == 8 {
                    break;
                }
            }

            // dbg!(&secondary_oam_indexes);
            for index in &secondary_oam_indexes {
                let sprite = &sprites[*index];

                for i in 0..8 {
                    if sprite.x.overflowing_add(i).1 {
                        break;
                    }

                    let pixel_value = sprite_pixel_value(
                        &sprite,
                        |pattern_selection, tile, x, y| {
                            self.read_pattern_value(pattern_selection, tile, x, y as u16)
                        },
                        y as u32,
                        i,
                    );

                    let target_buffer = if sprite.draw_priority == DrawPriority::Foreground {
                        &mut self.foreground_sprite_buffer
                    } else {
                        &mut self.background_sprite_buffer
                    };

                    if let Some(value) = pixel_value {
                        if value != 0
                            && target_buffer[y as usize][sprite.x as usize + i as usize] == 0xff
                        {
                            target_buffer[y as usize][sprite.x as usize + i as usize] =
                                palette[sprite.color_palette as usize][value as usize - 1];
                        }
                    }
                }
            }
        }

        // watch out for background value and sprite priority
    }

    fn toggle_sprite_0_hit_if_needed(&mut self) {
        if !self.status.contains(PpuStatus::SPRITE_0_HIT) {
            if !self
                .mask
                .contains(PpuMask::SHOW_BACKGROUND | PpuMask::SHOW_SPRITES)
            {
                return;
            }

            let sprite_data = self.get_oam_sprite_data_at(0);

            let sprite_y = sprite_data.y as u32 + 1;
            let sprite_x = sprite_data.x;

            if sprite_x == 255 {
                return;
            }

            let sprite_height_offset = if self.control.contains(PpuControl::SPRITE_8X16_MODE) {
                15
            } else {
                7
            };

            // Check whether the scanline is in sprite's y range
            if self.current_scanline < sprite_y
                || self.current_scanline > sprite_y + sprite_height_offset
            {
                return;
            }

            let include_leftmost_tile = self
                .mask
                .contains(PpuMask::SHOW_LEFTMOST_BACKGROUND | PpuMask::SHOW_LEFTMOST_SPRITES);

            for i in 0..8 {
                let (x, overflow) = sprite_x.overflowing_add(i);

                if overflow {
                    break;
                }

                if x == 255 || (!include_leftmost_tile && x < 8) {
                    continue;
                }

                let pixel_value = sprite_pixel_value(
                    &sprite_data,
                    |pattern_selection, tile, x, y| {
                        self.read_pattern_value(pattern_selection, tile, x, y as u16)
                    },
                    self.current_scanline,
                    i,
                );
                if self.frame_buffer[self.current_scanline as usize][x as usize] != 0xff
                    && pixel_value.unwrap_or(0) != 0
                {
                    self.status.insert(PpuStatus::SPRITE_0_HIT);
                    return;
                }
            }
        }
    }

    pub fn get_frame_buffer(&self) -> &[[u8; 256]; 240] {
        &self.frame_buffer
    }

    pub fn get_foreground_sprite_buffer(&self) -> &[[u8; 256]; 240] {
        &self.foreground_sprite_buffer
    }

    pub fn get_background_sprite_buffer(&self) -> &[[u8; 256]; 240] {
        &self.background_sprite_buffer
    }

    pub fn generates_nmi_at_vblank(&self) -> bool {
        self.control.contains(PpuControl::GENERATE_NMI_AT_VBLANK)
    }

    pub fn is_background_rendering_enabled(&self) -> bool {
        self.mask.contains(PpuMask::SHOW_BACKGROUND)
    }

    pub fn is_sprite_rendering_enabled(&self) -> bool {
        self.mask.contains(PpuMask::SHOW_SPRITES)
    }

    pub fn top_left_nametable_address(&self) -> u16 {
        self.cartridge.borrow().mirroring().real_address(0x2000)
    }

    pub fn top_right_nametable_address(&self) -> u16 {
        self.cartridge.borrow().mirroring().real_address(0x2400)
    }

    pub fn bottom_left_nametable_address(&self) -> u16 {
        self.cartridge.borrow().mirroring().real_address(0x2800)
    }

    pub fn bottom_right_nametable_address(&self) -> u16 {
        self.cartridge.borrow().mirroring().real_address(0x2c00)
    }

    pub fn triggers_scanline_tick(&self) -> bool {
        // rendering is on
        // bg and sprite uses different table
        // handle 8x16

        if !self.is_background_rendering_enabled() && !self.is_sprite_rendering_enabled() {
            return false;
        }

        if self.control.contains(
            PpuControl::BACKGROUND_PATTERN_TABLE_FLAG | PpuControl::SPRITE_PATTERN_TABLE_FLAG,
        ) {
            return false;
        }

        if !self
            .control
            .contains(PpuControl::BACKGROUND_PATTERN_TABLE_FLAG)
            && !self.control.contains(PpuControl::SPRITE_PATTERN_TABLE_FLAG)
        {
            return false;
        }

        return true;
    }

    fn scanline_tick_if_possible(&mut self) {
        if self.triggers_scanline_tick() {
            println!("Scanline tick at {}", self.current_scanline);
            self.cartridge.borrow_mut().scanline_tick();
        }
    }

    pub fn get_current_dot(&self) -> u32 {
        self.current_dot
    }
}

fn sprite_pixel_value<F: Fn(PatternTableSelection, u8, u8, u8) -> u8>(
    sprite_data: &SpriteData,
    read_pattern: F,
    y: u32,
    x: u8,
) -> Option<u8> {
    let vertical_flip = sprite_data.flip_vertical;
    let horizontal_flip = sprite_data.flip_horizontal;

    let sprite_y = sprite_data.y as u32 + 1;

    let sprite_height_offset = if sprite_data.drawing_mode == SpriteDrawingMode::Draw8x16 {
        15
    } else {
        7
    };

    // Check whether the scanline is in sprite's y range
    if y < sprite_y || y > sprite_y + sprite_height_offset {
        return None;
    }

    let mut sprite_fine_y = y - sprite_y;
    let mut tile = sprite_data.tile_number;

    if vertical_flip {
        if sprite_data.drawing_mode == SpriteDrawingMode::Draw8x8 {
            sprite_fine_y = sprite_height_offset - sprite_fine_y;
        } else {
            // When flipping is on in 8x16 mode, the second tile is
            // above the first tile
            if sprite_fine_y < 8 {
                tile += 1;
            }

            // Split the offset into two 8 pixel length
            sprite_fine_y %= 8;
            sprite_fine_y = 7 - sprite_fine_y;
        }
    } else if sprite_fine_y >= 8 {
        tile += 1;
        sprite_fine_y %= 8;
    }
    // TODO: fix y type
    Some(read_pattern(
        sprite_data.tile_pattern,
        tile,
        if horizontal_flip { 7 - x } else { x },
        sprite_fine_y.try_into().unwrap(),
    ))
}
