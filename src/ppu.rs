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
    low_address_byte: u8,
}

pub struct ColorPalette {
    pub background: u8,
    pub set_1: (u8, u8, u8),
    pub set_2: (u8, u8, u8),
    pub set_3: (u8, u8, u8),
    pub set_4: (u8, u8, u8),
}

impl Ppu {
    pub fn new(memory: VideoMemoryBuffer) -> Ppu {
        Ppu {
            memory,
            current_address: 0,
            address_latch: AddressLatch::High,
            low_address_byte: 0,
        }
    }

    pub fn write_data(&mut self, data: u8) {
        self.memory[self.current_address as usize] = data;
        self.current_address = self.current_address.wrapping_add(1);
    }

    pub fn write_address(&mut self, byte: u8) {
        match self.address_latch {
            AddressLatch::High => {
                self.address_latch = AddressLatch::Low;
                self.current_address = u16::from_le_bytes([self.low_address_byte, byte]);
            }

            AddressLatch::Low => {
                self.address_latch = AddressLatch::High;
                self.low_address_byte = byte;
            }
        }
    }

    pub fn get_buffer(&self) -> &VideoMemoryBuffer {
        &self.memory
    }

    pub fn get_color_palette(&self) -> ColorPalette {
        ColorPalette {
            background: self.memory[0x3f00],
            set_1: (
                self.memory[0x3f01],
                self.memory[0x3f02],
                self.memory[0x3f03],
            ),
            set_2: (
                self.memory[0x3f05],
                self.memory[0x3f06],
                self.memory[0x3f07],
            ),
            set_3: (
                self.memory[0x3f09],
                self.memory[0x3f0a],
                self.memory[0x3f0b],
            ),
            set_4: (
                self.memory[0x3f0d],
                self.memory[0x3f0e],
                self.memory[0x3f0f],
            ),
        }
    }
}
