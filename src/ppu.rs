pub type VideoMemoryBuffer = [u8; 0x4000];

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(PartialEq, Eq)]
pub struct Ppu {
    memory: VideoMemoryBuffer,
    write_latch: WriteLatch,

    current_oam_address: u8,
    oam_data: [u8; 256],
    control_flag: u8,

    x: u8,
    t: u16,
    v: u16,
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
        }
    }

    pub fn set_control_flag(&mut self, value: u8) {
        self.control_flag = value;
        self.t |= (value as u16 & 0b11) << 10;
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
        self.memory[self.v as usize] = data;

        if self.control_flag & 0b00000100 != 0 {
            self.v = self.v.wrapping_add(32);
        } else {
            self.v = self.v.wrapping_add(1);
        }
    }

    pub fn write_scroll(&mut self, data: u8) {
        match self.write_latch {
            WriteLatch::One => {
                self.x = data & 0b00000111;
                self.t |= (data >> 3) as u16;
            }

            WriteLatch::Zero => {
                let coarse_y = (data >> 3) as u16;
                let fine_y = (data & 0b00000111) as u16;

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

    pub fn write_address(&mut self, byte: u8) {
        match self.write_latch {
            WriteLatch::Zero => {
                let value = byte & 0b00111111;
                self.t &= 0xff;
                self.t |= (value as u16) << 8;
            }

            WriteLatch::One => {
                self.t &= 0xff00;
                self.t |= byte as u16;
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
}
