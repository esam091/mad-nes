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

    pub fn step(&mut self) -> CpuResult {
        let instruction = Instruction::from_bytes(self);

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
            _ => todo!("interpret instructions"),
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

impl Iterator for Cpu {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.memory[self.pc as usize];
        self.pc += 1;

        return Some(value);
    }
}
