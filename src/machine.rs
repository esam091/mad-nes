use crate::instruction::Instruction;

pub type MemoryBuffer = [u8; 0x10000];
pub struct Machine {
    memory: MemoryBuffer,
    pc: u16,
    a: u8,
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        // since we're using a legit hello world ROM as a prototype, we don't need to configure stuff from headers
        let bytes: Vec<u8> = std::fs::read(file_path)?.into_iter().skip(16).collect();

        let mut memory = [0 as u8; 0x10000];

        for i in 0..bytes.len() - 16 {
            memory[0x8000 + i] = bytes[i];
        }

        // Copy the reset vector data
        // The data was located at 400c but we subtract by 0x10 since we skipped 16 bytes in the header
        memory[0xfffc] = bytes[0x3ffc];
        memory[0xfffd] = bytes[0x3ffd];

        // jump to reset vector
        let initial_address = u16::from_le_bytes([memory[0xfffc], memory[0xfffd]]);

        return Ok(Machine {
            memory: memory,
            pc: initial_address,
            a: 0,
        });
    }

    fn get_byte_and_forward_pc(&mut self) -> u8 {
        let value = self.memory[self.pc as usize];
        self.pc += 1;

        return value;
    }

    fn get_word_and_forward_pc(&mut self) -> u16 {
        let byte1 = self.get_byte_and_forward_pc();
        let byte2 = self.get_byte_and_forward_pc();

        return u16::from_le_bytes([byte1, byte2]);
    }

    pub fn step(&mut self) {
        let opcode = self.memory[self.pc as usize];
        self.pc += 1;

        let instruction: Option<Instruction>;
        match opcode {
            0xa9 => {
                instruction = Some(Instruction::LdaImmediate(self.get_byte_and_forward_pc()));
            }
            0x8d => {
                instruction = Some(Instruction::StaAbsolute(self.get_word_and_forward_pc()));
            }
            _ => {
                instruction = None;
            }
        }

        match instruction.expect("Instruction not found, opcode might not have been implemented") {
            Instruction::LdaImmediate(value) => {
                self.a = value;
            }
            Instruction::StaAbsolute(value) => {
                self.memory[value as usize] = self.a;
            }
        }
    }

    pub fn get_buffer(&self) -> &MemoryBuffer {
        &self.memory
    }
}
