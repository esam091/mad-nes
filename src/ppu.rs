pub type VideoMemoryBuffer = [u8; 0x4000];

#[derive(Clone, Copy, PartialEq, Eq)]
enum AddressLatch {
    Low,
    High,
}

#[derive(PartialEq, Eq)]
pub struct Ppu {
    memory: VideoMemoryBuffer,
    current_address: u16,
    address_latch: AddressLatch,
    high_address_byte: u8,

    current_oam_address: u8,
    oam_data: [u8; 256],
}

pub struct ColorPalette {
    pub background: u8,
    pub color_set: [[u8; 3]; 4],
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
        }
    }

    pub fn set_oam_address(&mut self, address: u8) {
        self.current_oam_address = address;
    }

    pub fn write_oam_data(&mut self, data: u8) {
        self.oam_data[self.current_oam_address as usize] = data;
    }

    pub fn write_data(&mut self, data: u8) {
        self.memory[self.current_address as usize] = data;
        self.current_address = self.current_address.wrapping_add(1);
    }

    pub fn copy_oam_data(&mut self, data: &[u8]) {
        &self.oam_data.copy_from_slice(data);
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

    pub fn get_color_palette(&self) -> ColorPalette {
        ColorPalette {
            background: self.memory[0x3f00],
            color_set: [
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
        }
    }
}
