pub type VideoMemoryBuffer = [u8; 0x4000];

#[derive(Clone, Copy, PartialEq, Eq)]
enum AddressLatch {
    Low,
    High,
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
    current_address: u16,
    address_latch: AddressLatch,
    high_address_byte: u8,

    current_oam_address: u8,
    oam_data: [u8; 256],
    control_flag: u8,
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
            current_address: 0,
            address_latch: AddressLatch::High,
            high_address_byte: 0,
            current_oam_address: 0,
            oam_data: [0; 256],
            control_flag: 0,
        }
    }

    pub fn set_control_flag(&mut self, value: u8) {
        self.control_flag = value;
    }

    pub fn clear_address_latch(&mut self) {
        self.address_latch = AddressLatch::High;
    }

    pub fn set_oam_address(&mut self, address: u8) {
        self.current_oam_address = address;
    }

    pub fn write_oam_data(&mut self, data: u8) {
        self.oam_data[self.current_oam_address as usize] = data;
        self.current_oam_address += 1;
    }

    pub fn write_data(&mut self, data: u8) {
        self.memory[self.current_address as usize] = data;

        if self.control_flag & 0b00000100 != 0 {
            self.current_address = self.current_address.wrapping_add(32);
        } else {
            self.current_address = self.current_address.wrapping_add(1);
        }
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
                let tile_pattern = if byte1 & 1 != 0 {
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
        match self.address_latch {
            AddressLatch::High => {
                self.address_latch = AddressLatch::Low;
                self.high_address_byte = byte;
            }

            AddressLatch::Low => {
                self.address_latch = AddressLatch::High;
                self.current_address = u16::from_le_bytes([byte, self.high_address_byte]);
            }
        }
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
        if self.control_flag & 0x16 == 0 {
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
