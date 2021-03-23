use crate::instruction::Instruction;

pub type MemoryBuffer = [u8; 0x10000];

#[derive(PartialEq, Eq)]
pub struct Machine {
    memory: MemoryBuffer,
    pc: u16,
    a: u8,
    x: u8,

    zero_flag: bool,
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
            x: 0,

            zero_flag: false,
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

        // println!("opcode {:#02x?}", opcode);
        println!("pc {:#02x?}", self.pc);

        let instruction: Instruction;
        match opcode {
            0xa9 => {
                instruction = Instruction::LdaImmediate(self.get_byte_and_forward_pc());
            }
            0x8d => {
                instruction = Instruction::StaAbsolute(self.get_word_and_forward_pc());
            }
            0xa2 => {
                instruction = Instruction::LdxImmediate(self.get_byte_and_forward_pc());
            }
            0xbd => {
                instruction = Instruction::LdaXAbsolute(self.get_word_and_forward_pc());
            }
            0xc9 => {
                instruction = Instruction::CmpImmediate(self.get_byte_and_forward_pc());
            }
            0xf0 => {
                instruction = Instruction::Beq(self.get_byte_and_forward_pc());
            }
            0xe8 => {
                instruction = Instruction::Inx;
            }
            0x4c => {
                instruction = Instruction::JmpAbsolute(self.get_word_and_forward_pc());
            }
            0xe0 => {
                instruction = Instruction::CpxImmediate(self.get_byte_and_forward_pc());
            }
            0xd0 => {
                instruction = Instruction::Bne(self.get_byte_and_forward_pc());
            }
            _ => {
                panic!("Cannot parse opcode {:#02x?} at pc {:#02x?}, either it is not implemented yet, or you reached data section by mistake", opcode, self.pc);
            }
        }

        // println!("instruction: {:#04X?}", instruction);

        match instruction {
            Instruction::LdaImmediate(value) => {
                self.a = value;
            }
            Instruction::StaAbsolute(value) => {
                self.memory[value as usize] = self.a;
            }
            Instruction::LdxImmediate(value) => {
                self.x = value;
            }
            Instruction::LdaXAbsolute(value) => {
                self.a = self.memory[value as usize + self.x as usize];
            }
            Instruction::CmpImmediate(value) => {
                let result = self.a.overflowing_sub(value);
                self.zero_flag = result.0 == 0;
            }
            Instruction::Beq(value) => {
                if self.zero_flag {
                    self.pc += value as u16;
                }
            }
            Instruction::Inx => {
                self.x += 1;
            }
            Instruction::JmpAbsolute(address) => {
                self.pc = address;
            }
            Instruction::CpxImmediate(value) => {
                let result = self.x.overflowing_sub(value);
                self.zero_flag = result.0 == 0;
            }
            Instruction::Bne(offset) => {
                if !self.zero_flag {
                    let a = self.pc as i16 + (offset as i8) as i16;
                    self.pc = a as u16;
                }
            }
        }
    }

    pub fn get_buffer(&self) -> &MemoryBuffer {
        &self.memory
    }
}
