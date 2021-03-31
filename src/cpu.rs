use std::ops::BitAndAssign;

use crate::{ines::InesRom, instruction::Instruction};

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
    p: u8,
    sp: u8,

    zero_flag: bool,
}

impl Cpu {
    #[must_use]
    fn set_memory_value(&mut self, address: u16, value: u8) -> Option<SideEffect> {
        self.memory[address as usize] = value;

        match address {
            0x2006 => Some(SideEffect::WritePpuAddr(value)),
            0x2007 => Some(SideEffect::WritePpuData(value)),
            _ => None,
        }
    }

    pub fn step(&mut self) -> CpuResult {
        let instruction = Instruction::from_bytes(self)
            .map_err(|opcode| {
                format!(
                    "Failed parsing opcode: {:#02X?} at pc: {:02X?}",
                    opcode,
                    self.pc - 1
                )
            })
            .unwrap();

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
                self.set_zero_flag(value == 0);

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
                cycles(3)
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

            Instruction::Brk => {
                self.set_break_flag(true);
                cycles(7)
            }

            Instruction::StxZeroPage(address) => {
                let side_effect = self.set_memory_value(address as u16, self.x);
                CpuResult {
                    cycles_elapsed: 3,
                    side_effect,
                }
            }

            Instruction::JsrAbsolute(address) => {
                let bytes = self.pc.to_le_bytes();
                self.memory[self.sp as usize] = bytes[0];
                self.memory[(self.sp - 1) as usize] = bytes[1];

                self.sp -= 2;
                self.pc = address;
                cycles(6)
            }

            Instruction::Nop => cycles(2),

            Instruction::Sec => {
                self.set_carry_flag(true);
                cycles(2)
            }
            _ => todo!("interpret instructions: {:?}", instruction),
        }
    }

    fn set_zero_flag(&mut self, is_on: bool) {
        if is_on {
            self.p |= 2;
        } else {
            self.p &= !((is_on as u8) << 1);
        }
    }

    fn set_break_flag(&mut self, is_on: bool) {
        self.p &= !((is_on as u8) << 4);
    }

    fn set_carry_flag(&mut self, is_on: bool) {
        if is_on {
            self.p |= 1;
        } else {
            self.p &= !(is_on as u8);
        }
    }

    pub fn new(memory: MemoryBuffer, starting_address: u16) -> Cpu {
        Cpu {
            memory: memory,
            pc: starting_address,
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
            sp: 0xfd,

            zero_flag: false,
        }
    }

    pub fn load(rom: &InesRom) -> Cpu {
        let mut memory = [0 as u8; 0x10000];

        // temporarily assign starting address to 0xc000 so nestest can run.
        memory[0xc000..0xc000 + rom.prg_rom_data().len()].copy_from_slice(&rom.prg_rom_data());

        let vector_positions = rom.prg_rom_data().len() - 6;

        memory[0xfffa..].copy_from_slice(&rom.prg_rom_data()[vector_positions..]);

        // jump to reset vector
        let initial_address = u16::from_le_bytes([memory[0xfffc], memory[0xfffd]]);

        return Cpu::new(memory, initial_address);
    }

    pub fn get_memory_buffer(&self) -> &MemoryBuffer {
        &self.memory
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::ines::InesRom;

    #[test]
    fn nestest() {
        let text = std::fs::read_to_string("nestest.log").unwrap();
        let lines = text.lines();

        let mut cycles = 7;
        let mut cpu = Cpu::load(&InesRom::load("nestest.nes").ok().unwrap());

        // starting point according to the nestest guide
        cpu.pc = 0xc000;

        for line in lines {
            let trimmed_line = format!("{} {} {}", &line[..4], &line[48..73], &line[86..]);

            let cpu_state = format!(
                "{:04X?} A:{:02X?} X:{:02X?} Y:{:02X?} P:{:02X?} SP:{:02X?} CYC:{}",
                cpu.pc, cpu.a, cpu.x, cpu.y, cpu.p, cpu.sp, cycles
            );

            assert_eq!(cpu_state, trimmed_line);

            cycles += cpu.step().cycles_elapsed;
        }
    }

    #[test]
    fn something() {
        assert_eq!(!0u8, 0xff);
    }
}
