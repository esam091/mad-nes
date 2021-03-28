use crate::instruction::Instruction;

pub type MemoryBuffer = [u8; 0x10000];
pub enum SideEffect {
    WritePpuAddr(u8),
    WritePpuData(u8),
}

pub struct CpuResult {
    pub cycles_elapsed: u32,
    pub side_effect: Option<SideEffect>,
}

fn cycles(cycles_elapsed: u32) -> CpuResult {
    CpuResult {
        cycles_elapsed: cycles_elapsed,
        side_effect: None,
    }
}

#[derive(PartialEq, Eq)]
pub struct Cpu {
    memory: MemoryBuffer,
    pc: u16,
    a: u8,
    x: u8,
    y: u8,

    zero_flag: bool,
}

impl Cpu {
    fn set_memory_value(&mut self, address: u16, value: u8) -> Option<SideEffect> {
        self.memory[address as usize] = value;

        match address {
            0x2006 => Some(SideEffect::WritePpuAddr(value)),
            0x2007 => Some(SideEffect::WritePpuData(value)),
            _ => None,
        }
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

    pub fn step(&mut self) -> CpuResult {
        let opcode = self.memory[self.pc as usize];
        self.pc += 1;

        // println!("opcode {:#02x?}", opcode);
        // println!("pc {:#02x?}", self.pc);

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

        match instruction {
            Instruction::LdaImmediate(value) => {
                self.a = value;
                cycles(2)
            }
            Instruction::StaAbsolute(address) => {
                let side_effect = self.set_memory_value(address, self.a);

                CpuResult {
                    cycles_elapsed: 4,
                    side_effect,
                }
            }
            Instruction::LdxImmediate(value) => {
                self.x = value;
                cycles(2)
            }
            Instruction::LdaXAbsolute(value) => {
                self.a = self.memory[value as usize + self.x as usize];
                cycles(2)
            }
            Instruction::CmpImmediate(value) => {
                let result = self.a.overflowing_sub(value);
                self.zero_flag = result.0 == 0;

                cycles(2)
            }
            Instruction::Beq(value) => {
                if self.zero_flag {
                    self.pc += value as u16;
                }

                cycles(2)
            }
            Instruction::Inx => {
                self.x += 1;
                cycles(2)
            }
            Instruction::JmpAbsolute(address) => {
                self.pc = address;
                cycles(5)
            }
            Instruction::CpxImmediate(value) => {
                let result = self.x.overflowing_sub(value);
                self.zero_flag = result.0 == 0;
                cycles(2)
            }
            Instruction::Bne(offset) => {
                if !self.zero_flag {
                    let a = self.pc as i16 + (offset as i8) as i16;
                    self.pc = a as u16;
                }
                cycles(2)
            }
        }
    }

    pub fn new(memory: MemoryBuffer, starting_address: u16) -> Cpu {
        Cpu {
            memory: memory,
            pc: starting_address,
            a: 0,
            x: 0,
            y: 0,

            zero_flag: false,
        }
    }
}
