use std::ops::{BitAnd, BitAndAssign, BitOr};

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
            Instruction::AndImmediate(value) => {
                self.a &= value;
                self.toggle_zero_negative_flag(self.a);
                cycles(2)
            }

            Instruction::OraImmediate(value) => {
                self.a |= value;
                self.toggle_zero_negative_flag(self.a);
                cycles(2)
            }

            Instruction::EorImmediate(value) => {
                self.a ^= value;
                self.toggle_zero_negative_flag(self.a);
                cycles(2)
            }

            Instruction::AdcImmediate(value) => {
                let (result, carry) = self
                    .a
                    .overflowing_add(value + self.is_carry_flag_on() as u8); // maybe we should check for second carry?
                let (_, overflow) = (self.a as i8).overflowing_add(value as i8); // also check for overflow with carry?

                self.a = result;
                self.toggle_zero_negative_flag(self.a);
                self.set_carry_flag(carry);
                self.set_overflow_flag(overflow);

                cycles(2)
            }

            Instruction::SbcImmediate(value) => {
                let (result, not_carry) = self
                    .a
                    .overflowing_sub(value + !self.is_carry_flag_on() as u8);

                let (_, overflow) =
                    (self.a as i8).overflowing_sub(value as i8 + !self.is_carry_flag_on() as i8); // also need to check for edge cases

                self.a = result;
                self.toggle_zero_negative_flag(self.a);
                self.set_carry_flag(!not_carry);
                self.set_overflow_flag(overflow);

                cycles(2)
            }

            Instruction::LdaImmediate(value) => {
                self.a = value;

                self.toggle_zero_negative_flag(self.a);
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
                self.toggle_zero_negative_flag(self.x);

                cycles(2)
            }

            Instruction::LdyImmediate(value) => {
                self.y = value;
                self.toggle_zero_negative_flag(self.y);

                cycles(2)
            }

            Instruction::LdaXAbsolute(value) => {
                self.a = self.memory[value as usize + self.x as usize];

                self.toggle_zero_negative_flag(self.a);
                cycles(2)
            }

            Instruction::CmpImmediate(value) => {
                let (value, overflow) = self.a.overflowing_sub(value);
                self.set_zero_flag(value == 0);
                self.set_negative_flag(value & 0x80 != 0);
                self.set_carry_flag(!overflow);

                cycles(2)
            }

            Instruction::Beq(offset) => self.jump_if(self.is_zero_flag_on(), offset),

            Instruction::Inx => {
                self.x = self.x.overflowing_add(1).0;
                self.toggle_zero_negative_flag(self.x);
                cycles(2)
            }

            Instruction::Iny => {
                self.y = self.y.overflowing_add(1).0;
                self.toggle_zero_negative_flag(self.y);
                cycles(2)
            }

            Instruction::Dex => {
                self.x = self.x.overflowing_sub(1).0;
                self.toggle_zero_negative_flag(self.x);
                cycles(2)
            }

            Instruction::Dey => {
                self.y = self.y.overflowing_sub(1).0;
                self.toggle_zero_negative_flag(self.y);
                cycles(2)
            }

            Instruction::Tay => {
                self.y = self.a;
                self.toggle_zero_negative_flag(self.y);
                cycles(2)
            }

            Instruction::Tya => {
                self.a = self.y;
                self.toggle_zero_negative_flag(self.a);
                cycles(2)
            }

            Instruction::Tax => {
                self.x = self.a;
                self.toggle_zero_negative_flag(self.x);
                cycles(2)
            }

            Instruction::Txa => {
                self.a = self.x;
                self.toggle_zero_negative_flag(self.a);
                cycles(2)
            }

            Instruction::Tsx => {
                self.x = self.sp;
                self.toggle_zero_negative_flag(self.x);
                cycles(2)
            }

            Instruction::Txs => {
                self.sp = self.x;
                cycles(2)
            }

            Instruction::JmpAbsolute(address) => {
                self.pc = address;
                cycles(3)
            }

            Instruction::CpxImmediate(value) => {
                let (value, overflow) = self.x.overflowing_sub(value);

                self.toggle_zero_negative_flag(value);
                self.set_carry_flag(!overflow);
                cycles(2)
            }

            Instruction::CpyImmediate(value) => {
                let (value, overflow) = self.y.overflowing_sub(value);

                self.toggle_zero_negative_flag(value);
                self.set_carry_flag(!overflow);
                cycles(2)
            }

            Instruction::Bne(offset) => self.jump_if(!self.is_zero_flag_on(), offset),

            Instruction::Brk => {
                self.set_break_flag(true);
                cycles(7)
            }

            Instruction::StaZeroPage(address) => {
                let side_effect = self.set_memory_value(address as u16, self.a);
                CpuResult {
                    cycles_elapsed: 3,
                    side_effect,
                }
            }

            Instruction::StxZeroPage(address) => {
                let side_effect = self.set_memory_value(address as u16, self.x);
                CpuResult {
                    cycles_elapsed: 3,
                    side_effect,
                }
            }

            Instruction::StxAbsolute(address) => {
                let side_effect = self.set_memory_value(address, self.x);
                CpuResult {
                    cycles_elapsed: 4,
                    side_effect,
                }
            }

            Instruction::JsrAbsolute(address) => {
                let bytes = self.pc.to_le_bytes();
                self.push(bytes[1]);
                self.push(bytes[0]);

                self.pc = address;
                cycles(6)
            }

            Instruction::Rts => {
                let low_byte = self.pop();
                let high_byte = self.pop();

                let address = u16::from_le_bytes([low_byte, high_byte]);

                self.pc = address;
                cycles(6)
            }

            Instruction::Nop => cycles(2),

            Instruction::Sec => {
                self.set_carry_flag(true);
                cycles(2)
            }

            Instruction::Clc => {
                self.set_carry_flag(false);
                cycles(2)
            }

            Instruction::Clv => {
                self.set_overflow_flag(false);
                cycles(2)
            }

            Instruction::Bcs(offset) => self.jump_if(self.is_carry_flag_on(), offset),

            Instruction::Bcc(offset) => self.jump_if(!self.is_carry_flag_on(), offset),

            Instruction::BitZeroPage(address) => {
                let value = self.memory[address as usize];

                self.set_negative_flag(value & 0x80 != 0);
                self.set_overflow_flag(value & 0x40 != 0);
                self.set_zero_flag((value & self.a) == 0);
                cycles(3)
            }

            Instruction::Bvs(offset) => self.jump_if(self.is_overflow_flag_on(), offset),

            Instruction::Bvc(offset) => self.jump_if(!self.is_overflow_flag_on(), offset),

            Instruction::Bpl(offset) => self.jump_if(!self.is_negative_flag_on(), offset),

            Instruction::Bmi(offset) => self.jump_if(self.is_negative_flag_on(), offset),

            Instruction::Sei => {
                self.set_interrupt_flag(true);
                cycles(2)
            }

            Instruction::Cld => {
                self.set_decimal_flag(false);
                cycles(2)
            }

            Instruction::Sed => {
                self.set_decimal_flag(true);
                cycles(2)
            }

            Instruction::Php => {
                self.sp -= 1;

                let side_effect = self.set_memory_value(self.sp as u16, self.p.bitor(0x10));
                CpuResult {
                    cycles_elapsed: 3,
                    side_effect,
                }
            }

            Instruction::Pha => {
                self.sp -= 1;

                let side_effect = self.set_memory_value(self.sp as u16, self.a);
                CpuResult {
                    cycles_elapsed: 3,
                    side_effect,
                }
            }

            Instruction::Pla => {
                self.a = self.memory[self.sp as usize];
                self.sp += 1;

                self.toggle_zero_negative_flag(self.a);

                cycles(4)
            }

            Instruction::Plp => {
                self.p = self.memory[self.sp as usize]
                    .bitand(!(1 << 4))
                    .bitor(1 << 5); // this bit is always on
                self.sp += 1;

                cycles(4)
            }

            _ => todo!("interpret instructions: {:#02X?}", instruction),
        }
    }

    fn jump_if(&mut self, condition: bool, offset: u8) -> CpuResult {
        if condition {
            let new_address = self.pc as i16 + (offset as i8) as i16;
            self.pc = new_address as u16;
            return cycles(3);
        }

        return cycles(2);
    }

    fn push(&mut self, value: u8) {
        self.memory[self.sp as usize] = value;
        self.sp -= 1;
    }

    fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.memory[self.sp as usize]
    }

    fn set_negative_flag(&mut self, is_on: bool) {
        self.set_p_flag(7, is_on);
    }

    fn set_zero_flag(&mut self, is_on: bool) {
        self.set_p_flag(1, is_on);
    }

    fn set_break_flag(&mut self, is_on: bool) {
        self.p &= !((is_on as u8) << 4);
    }

    fn set_carry_flag(&mut self, is_on: bool) {
        self.set_p_flag(0, is_on);
    }

    fn set_overflow_flag(&mut self, is_on: bool) {
        self.set_p_flag(6, is_on);
    }

    fn set_decimal_flag(&mut self, is_on: bool) {
        self.set_p_flag(3, is_on);
    }

    fn set_p_flag(&mut self, bit_offset: u8, is_on: bool) {
        let byte = 1 << bit_offset;
        if is_on {
            self.p |= byte;
        } else {
            self.p &= !byte;
        }
    }

    fn set_interrupt_flag(&mut self, is_on: bool) {
        self.set_p_flag(2, is_on);
    }

    fn is_overflow_flag_on(&self) -> bool {
        self.p.bitand(1 << 6) != 0
    }

    fn is_negative_flag_on(&self) -> bool {
        self.p.bitand(1 << 7) != 0
    }

    fn is_carry_flag_on(&self) -> bool {
        self.p.bitand(1) != 0
    }

    fn is_zero_flag_on(&self) -> bool {
        self.p.bitand(2) != 0
    }

    fn toggle_zero_negative_flag(&mut self, value: u8) {
        self.set_negative_flag(value & 0x80 != 0);
        self.set_zero_flag(value == 0);
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
