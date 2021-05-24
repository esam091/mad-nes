use std::ops::{BitAnd, BitAndAssign, BitOr};

use crate::{ines::InesRom, instruction::Instruction};

pub type MemoryBuffer = [u8; 0x10000];

#[derive(Debug)]
pub enum SideEffect {
    WritePpuAddr(u8),
    WritePpuData(u8),
    WriteOamAddr(u8),
    WriteOamData(u8),
    OamDma(u8),
    ClearAddressLatch,
    SetPpuControl(u8),
}

pub struct CpuResult {
    pub cycles_elapsed: u32,
    pub side_effect: Option<SideEffect>,
}

#[inline(always)]
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

    nmi_vector: u16,
    reset_vector: u16,
    irq_vector: u16,
}

impl Cpu {
    #[must_use]
    #[inline(always)]
    fn set_memory_value(&mut self, address: u16, value: u8) -> Option<SideEffect> {
        self.memory[address as usize] = value;

        match address {
            0x2003 => Some(SideEffect::WriteOamAddr(value)),
            0x2004 => Some(SideEffect::WriteOamData(value)),
            0x4014 => Some(SideEffect::OamDma(value)),
            0x2006 => Some(SideEffect::WritePpuAddr(value)),
            0x2007 => Some(SideEffect::WritePpuData(value)),
            0x2000 => Some(SideEffect::SetPpuControl(value)),
            _ => None,
        }
    }

    pub fn enter_vblank(&mut self) {
        self.memory[0x2002] |= 0x80;

        if self.memory[0x2000] & 0x80 != 0 {
            // println!("Enter vblank");

            let addresses = self.pc.to_le_bytes();
            self.push(addresses[1]);
            self.push(addresses[0]);
            self.push(self.p.bitand(!(1 << 5)));
            self.pc = self.nmi_vector;
        }
    }

    pub fn exit_vblank(&mut self) {
        self.memory[0x2002] &= !0x80;
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

        // println!("{:#04X?}", instruction);

        match instruction {
            Instruction::AndImmediate(value) => {
                self.and(value);
                cycles(2)
            }

            Instruction::AndXIndexedIndirect(index) => {
                self.and(self.indexed_indirect_value(index));
                cycles(6)
            }

            Instruction::AndYIndirectIndexed(index) => {
                let (value, overflow) = self.indirect_indexed_value(index);
                self.and(value);
                cycles(5 + overflow as u32)
            }

            Instruction::AndZeroPage(address) => {
                self.and(self.memory[address as usize]);
                cycles(3)
            }

            Instruction::AndXZeroPage(address) => {
                self.and(self.zero_page_value(address, self.x));
                cycles(4)
            }

            Instruction::AndAbsolute(address) => {
                self.and(self.memory[address as usize]);
                cycles(4)
            }

            Instruction::AndXAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.x);
                self.and(value);
                cycles(4 + carry as u32)
            }

            Instruction::AndYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);
                self.and(value);
                cycles(4 + carry as u32)
            }

            Instruction::OraImmediate(value) => {
                self.or(value);
                cycles(2)
            }

            Instruction::OraXIndexedIndirect(index) => {
                self.or(self.indexed_indirect_value(index));
                cycles(6)
            }

            Instruction::OraZeroPage(address) => {
                self.or(self.memory[address as usize]);
                cycles(3)
            }

            Instruction::OraXZeroPage(address) => {
                self.or(self.zero_page_value(address, self.x));
                cycles(4)
            }

            Instruction::OraAbsolute(address) => {
                self.or(self.memory[address as usize]);
                cycles(4)
            }

            Instruction::OraXAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.x);
                self.or(value);
                cycles(4 + carry as u32)
            }

            Instruction::OraYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);

                self.or(value);
                cycles(4 + carry as u32)
            }

            Instruction::OraYIndirectIndexed(index) => {
                let (value, overflow) = self.indirect_indexed_value(index);
                self.or(value);
                cycles(5 + overflow as u32)
            }

            Instruction::EorImmediate(value) => {
                self.exor(value);
                cycles(2)
            }

            Instruction::EorXIndexedIndirect(index) => {
                self.exor(self.indexed_indirect_value(index));
                cycles(6)
            }

            Instruction::EorYIndirectIndexed(index) => {
                let (value, overflow) = self.indirect_indexed_value(index);
                self.exor(value);
                cycles(5 + overflow as u32)
            }

            Instruction::EorZeroPage(address) => {
                self.exor(self.memory[address as usize]);
                cycles(3)
            }

            Instruction::EorXZeroPage(address) => {
                self.exor(self.zero_page_value(address, self.x));
                cycles(4)
            }

            Instruction::EorAbsolute(address) => {
                self.exor(self.memory[address as usize]);
                cycles(4)
            }

            Instruction::EorXAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.x);
                self.exor(value);
                cycles(4 + carry as u32)
            }

            Instruction::EorYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);
                self.exor(value);
                cycles(4 + carry as u32)
            }

            Instruction::AdcImmediate(value) => {
                self.adc(value);
                cycles(2)
            }

            Instruction::AdcXIndexedIndirect(index) => {
                self.adc(self.indexed_indirect_value(index));
                cycles(6)
            }

            Instruction::AdcYIndirectIndexed(index) => {
                let (value, overflow) = self.indirect_indexed_value(index);
                self.adc(value);
                cycles(5 + overflow as u32)
            }

            Instruction::AdcZeroPage(address) => {
                self.adc(self.memory[address as usize]);
                cycles(3)
            }

            Instruction::AdcXZeroPage(address) => {
                self.adc(self.zero_page_value(address, self.x));
                cycles(4)
            }

            Instruction::AdcAbsolute(address) => {
                self.adc(self.memory[address as usize]);
                cycles(4)
            }

            Instruction::AdcXAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.x);
                self.adc(value);
                cycles(4 + carry as u32)
            }

            Instruction::AdcYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);
                self.adc(value);
                cycles(4 + carry as u32)
            }

            Instruction::SbcImmediate(value) | Instruction::SbcImmediateIllegal(value) => {
                self.sbc(value);
                cycles(2)
            }

            Instruction::SbcXIndexedIndirect(index) => {
                self.sbc(self.indexed_indirect_value(index));
                cycles(6)
            }

            Instruction::SbcYIndirectIndexed(index) => {
                let (value, overflow) = self.indirect_indexed_value(index);
                self.sbc(value);
                cycles(5 + overflow as u32)
            }

            Instruction::SbcZeroPage(address) => {
                self.sbc(self.memory[address as usize]);
                cycles(3)
            }

            Instruction::SbcXZeroPage(address) => {
                self.sbc(self.zero_page_value(address, self.x));
                cycles(4)
            }

            Instruction::SbcAbsolute(address) => {
                self.sbc(self.memory[address as usize]);
                cycles(4)
            }

            Instruction::SbcXAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.x);
                self.sbc(value);
                cycles(4 + carry as u32)
            }

            Instruction::SbcYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);
                self.sbc(value);
                cycles(4 + carry as u32)
            }

            Instruction::CmpImmediate(value) => {
                self.compare(self.a, value);
                cycles(2)
            }

            Instruction::CmpXIndexedIndirect(index) => {
                self.compare(self.a, self.indexed_indirect_value(index));
                cycles(6)
            }

            Instruction::CmpYIndirectIndexed(index) => {
                let (value, overflow) = self.indirect_indexed_value(index);
                self.compare(self.a, value);
                cycles(5 + overflow as u32)
            }

            Instruction::CmpZeroPage(address) => {
                self.compare(self.a, self.memory[address as usize]);
                cycles(3)
            }

            Instruction::CmpXZeroPage(address) => {
                self.compare(self.a, self.zero_page_value(address, self.x));
                cycles(4)
            }

            Instruction::CmpAbsolute(address) => {
                self.compare(self.a, self.memory[address as usize]);
                cycles(4)
            }

            Instruction::CmpXAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.x);
                self.compare(self.a, value);
                cycles(4 + carry as u32)
            }

            Instruction::CmpYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);
                self.compare(self.a, value);
                cycles(4 + carry as u32)
            }

            Instruction::CpxImmediate(value) => {
                self.compare(self.x, value);
                cycles(2)
            }

            Instruction::CpxZeroPage(address) => {
                self.compare(self.x, self.memory[address as usize]);
                cycles(3)
            }

            Instruction::CpxAbsolute(address) => {
                self.compare(self.x, self.memory[address as usize]);
                cycles(4)
            }

            Instruction::CpyImmediate(value) => {
                self.compare(self.y, value);
                cycles(2)
            }

            Instruction::CpyZeroPage(address) => {
                self.compare(self.y, self.memory[address as usize]);
                cycles(3)
            }

            Instruction::CpyAbsolute(address) => {
                self.compare(self.y, self.memory[address as usize]);
                cycles(4)
            }

            Instruction::Lsr => {
                self.a = self.lsr(self.a);

                cycles(2)
            }

            Instruction::LsrZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.lsr_address(address as u16),
            },

            Instruction::LsrXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.lsr_address(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::LsrAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.lsr_address(address),
            },

            Instruction::LsrXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.lsr_address(self.absolute_address(address, self.x).0),
            },

            Instruction::Asl => {
                self.a = self.asl(self.a);
                cycles(2)
            }

            Instruction::AslZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.asl_address(address as u16),
            },

            Instruction::AslXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.asl_address(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::AslAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.asl_address(address),
            },

            Instruction::AslXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.asl_address(self.absolute_address(address, self.x).0),
            },

            Instruction::Ror => {
                self.a = self.ror(self.a);
                cycles(2)
            }

            Instruction::RorZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.ror_address(address as u16),
            },

            Instruction::RorXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.ror_address(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::RorAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.ror_address(address),
            },

            Instruction::RorXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.ror_address(self.absolute_address(address, self.x).0),
            },

            Instruction::Rol => {
                self.a = self.rol(self.a);
                cycles(2)
            }

            Instruction::RolZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.rol_address(address as u16),
            },

            Instruction::RolXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.rol_address(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::RolAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.rol_address(address),
            },

            Instruction::RolXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.rol_address(self.absolute_address(address, self.x).0),
            },

            Instruction::IncZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.inc(address as u16),
            },

            Instruction::IncXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.inc(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::IncAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.inc(address),
            },

            Instruction::IncXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.inc(self.absolute_address(address, self.x).0),
            },

            Instruction::DecZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.dec(address as u16),
            },

            Instruction::DecXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.dec(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::DecAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.dec(address),
            },

            Instruction::DecXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.dec(self.absolute_address(address, self.x).0),
            },

            Instruction::StaAbsolute(address) => {
                let side_effect = self.set_memory_value(address, self.a);

                CpuResult {
                    cycles_elapsed: 4,
                    side_effect,
                }
            }

            Instruction::StaXAbsolute(address) => {
                let (address, _) = self.absolute_address(address, self.x);
                let side_effect = self.set_memory_value(address, self.a);

                CpuResult {
                    cycles_elapsed: 5,
                    side_effect,
                }
            }

            Instruction::StaYAbsolute(address) => {
                let (address, _) = self.absolute_address(address, self.y);
                let side_effect = self.set_memory_value(address, self.a);

                CpuResult {
                    cycles_elapsed: 5,
                    side_effect,
                }
            }

            Instruction::LdxImmediate(value) => {
                self.x = value;
                self.toggle_zero_negative_flag(self.x);

                cycles(2)
            }

            Instruction::LdxZeroPage(address) => {
                self.x = self.memory[address as usize];
                self.toggle_zero_negative_flag(self.x);

                cycles(3)
            }

            Instruction::LdxYZeroPage(address) => {
                self.x = self.zero_page_value(address, self.y);
                self.toggle_zero_negative_flag(self.x);

                cycles(4)
            }

            Instruction::LdxAbsolute(address) => {
                self.x = self.memory[address as usize];
                self.toggle_zero_negative_flag(self.x);

                cycles(4)
            }

            Instruction::LdxYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);
                self.x = value;
                self.toggle_zero_negative_flag(self.x);

                cycles(4 + carry as u32)
            }

            Instruction::LdyImmediate(value) => {
                self.y = value;
                self.toggle_zero_negative_flag(self.y);

                cycles(2)
            }

            Instruction::LdyZeroPage(address) => {
                self.y = self.memory[address as usize];
                self.toggle_zero_negative_flag(self.y);

                cycles(3)
            }

            Instruction::LdyXZeroPage(address) => {
                self.y = self.zero_page_value(address, self.x);
                self.toggle_zero_negative_flag(self.y);

                cycles(4)
            }

            Instruction::LdyAbsolute(address) => {
                self.y = self.memory[address as usize];
                self.toggle_zero_negative_flag(self.x);

                cycles(4)
            }

            Instruction::LdyXAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.x);
                self.y = value;
                self.toggle_zero_negative_flag(self.y);

                cycles(4 + carry as u32)
            }

            Instruction::LdaImmediate(value) => {
                self.a = value;

                self.toggle_zero_negative_flag(self.a);
                cycles(2)
            }

            Instruction::LdaAbsolute(address) => {
                self.a = self.memory[address as usize];
                self.toggle_zero_negative_flag(self.a);

                if address == 0x2002 {
                    self.memory[0x2002] &= !0x80;
                    return CpuResult {
                        cycles_elapsed: 4,
                        side_effect: Some(SideEffect::ClearAddressLatch),
                    };
                }
                cycles(4)
            }

            Instruction::LdaZeroPage(address) => {
                self.a = self.memory[address as usize];
                self.toggle_zero_negative_flag(self.a);

                cycles(3)
            }

            Instruction::LdaXZeroPage(address) => {
                self.a = self.zero_page_value(address, self.x);
                self.toggle_zero_negative_flag(self.a);

                cycles(4)
            }

            Instruction::LdaXIndexedIndirect(index) => {
                let address = self.indexed_indirect_address(index);

                self.a = self.memory[address as usize];

                self.toggle_zero_negative_flag(self.a);
                cycles(6)
            }

            Instruction::LdaYIndirectIndexed(index) => {
                let (value, overflow) = self.indirect_indexed_value(index);
                self.a = value;
                self.toggle_zero_negative_flag(self.a);
                cycles(5 + (overflow) as u32)
            }

            Instruction::LdaXAbsolute(address) => {
                let (address, carry) = self.absolute_address(address, self.x);

                self.a = self.memory[address as usize];

                self.toggle_zero_negative_flag(self.a);
                cycles(4 + carry as u32)
            }

            Instruction::LdaYAbsolute(address) => {
                let (address, carry) = self.absolute_address(address, self.y);

                self.a = self.memory[address as usize];

                self.toggle_zero_negative_flag(self.a);
                cycles(4 + carry as u32)
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

            Instruction::JmpIndirect(address) => {
                let low_byte = self.memory[address as usize];

                /*
                    http://nesdev.com/6502_cpu.txt

                    Indirect addressing modes do not handle page boundary crossing at all.
                    When the parameter's low byte is $FF, the effective address wraps
                    around and the CPU fetches high byte from $xx00 instead of $xx00+$0100.
                    E.g. JMP ($01FF) fetches PCL from $01FF and PCH from $0100,
                    and LDA ($FF),Y fetches the base address from $FF and $00.
                */
                let mut high_address_split = address.to_le_bytes();
                high_address_split[0] = high_address_split[0].overflowing_add(1).0;

                let high_address = u16::from_le_bytes(high_address_split);
                let high_byte = self.memory[high_address as usize];

                self.pc = u16::from_le_bytes([low_byte, high_byte]);

                cycles(5)
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

            Instruction::StaXZeroPage(address) => {
                let side_effect =
                    self.set_memory_value(self.zero_page_address(address, self.x) as u16, self.a);

                CpuResult {
                    cycles_elapsed: 4,
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

            Instruction::StxYZeroPage(address) => {
                let side_effect =
                    self.set_memory_value(self.zero_page_address(address, self.y) as u16, self.x);

                CpuResult {
                    cycles_elapsed: 4,
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

            Instruction::StaXIndexedIndirect(index) => {
                let address = self.indexed_indirect_address(index);
                let side_effect = self.set_memory_value(address, self.a);

                CpuResult {
                    cycles_elapsed: 6,
                    side_effect,
                }
            }

            Instruction::StaYIndirectIndexed(index) => {
                let (address, _) = self.indirect_indexed_address(index);
                let side_effect = self.set_memory_value(address, self.a);

                CpuResult {
                    cycles_elapsed: 6,
                    side_effect,
                }
            }

            Instruction::StyZeroPage(address) => {
                let side_effect = self.set_memory_value(address as u16, self.y);

                CpuResult {
                    cycles_elapsed: 3,
                    side_effect,
                }
            }

            Instruction::StyXZeroPage(address) => {
                let side_effect =
                    self.set_memory_value(self.zero_page_address(address, self.x) as u16, self.y);

                CpuResult {
                    cycles_elapsed: 4,
                    side_effect,
                }
            }

            Instruction::StyAbsolute(address) => {
                let side_effect = self.set_memory_value(address, self.y);

                CpuResult {
                    cycles_elapsed: 4,
                    side_effect,
                }
            }

            Instruction::JsrAbsolute(address) => {
                let bytes = u16::to_le_bytes(self.pc - 1);
                self.push(bytes[1]);
                self.push(bytes[0]);

                self.pc = address;
                cycles(6)
            }

            Instruction::Rts => {
                let low_byte = self.pop();
                let high_byte = self.pop();

                let address = u16::from_le_bytes([low_byte, high_byte]);

                self.pc = address + 1;
                cycles(6)
            }

            Instruction::Rti => {
                self.p = self.pop().bitor(1 << 5); // bit 5 is always on
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

            Instruction::BitAbsolute(address) => {
                let value = self.memory[address as usize];

                self.set_negative_flag(value & 0x80 != 0);
                self.set_overflow_flag(value & 0x40 != 0);
                self.set_zero_flag((value & self.a) == 0);
                cycles(4)
            }

            Instruction::Bvs(offset) => self.jump_if(self.is_overflow_flag_on(), offset),

            Instruction::Bvc(offset) => self.jump_if(!self.is_overflow_flag_on(), offset),

            Instruction::Bpl(offset) => self.jump_if(!self.is_negative_flag_on(), offset),

            Instruction::Bmi(offset) => self.jump_if(self.is_negative_flag_on(), offset),

            Instruction::Sei => {
                self.set_interrupt_flag(true);
                cycles(2)
            }

            Instruction::Cli => {
                self.set_interrupt_flag(false);
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
                self.push(self.p.bitor(0x10));

                cycles(3)
            }

            Instruction::Pha => {
                self.push(self.a);

                cycles(3)
            }

            Instruction::Pla => {
                self.a = self.pop();

                self.toggle_zero_negative_flag(self.a);

                cycles(4)
            }

            Instruction::Plp => {
                self.p = self.pop().bitand(!(1 << 4)).bitor(1 << 5); // this bit is always on

                cycles(4)
            }

            // illegal instructions
            Instruction::Nop2 | Instruction::NopImmediate(_) => cycles(2),
            Instruction::NopZeroPage(_) => cycles(3),
            Instruction::NopXZeroPage(_) => cycles(4),
            Instruction::NopAbsolute(_) => cycles(4),
            Instruction::NopXAbsolute(address) => {
                let (_, carry) = self.absolute_address(address, self.x);
                cycles(4 + carry as u32)
            }

            Instruction::LaxXIndexedIndirect(index) => {
                self.lax(self.indexed_indirect_value(index));
                cycles(6)
            }

            Instruction::LaxYIndirectIndexed(index) => {
                let (value, carry) = self.indirect_indexed_value(index);
                self.lax(value);
                cycles(5 + carry as u32)
            }

            Instruction::LaxZeroPage(address) => {
                self.lax(self.memory[address as usize]);
                cycles(3)
            }

            Instruction::LaxYZeroPage(address) => {
                self.lax(self.zero_page_value(address, self.y));
                cycles(4)
            }

            Instruction::LaxAbsolute(address) => {
                self.lax(self.absolute_value(address, 0).0);
                cycles(4)
            }

            Instruction::LaxYAbsolute(address) => {
                let (value, carry) = self.absolute_value(address, self.y);
                self.lax(value);
                cycles(4 + carry as u32)
            }

            Instruction::SaxXIndexedIndirect(index) => {
                let address = self.indexed_indirect_address(index);

                CpuResult {
                    cycles_elapsed: 6,
                    side_effect: self.sax(address),
                }
            }

            Instruction::SaxZeroPage(address) => CpuResult {
                cycles_elapsed: 3,
                side_effect: self.sax(address as u16),
            },

            Instruction::SaxYZeroPage(address) => CpuResult {
                cycles_elapsed: 4,
                side_effect: self.sax(self.zero_page_address(address, self.y) as u16),
            },

            Instruction::SaxAbsolute(address) => CpuResult {
                cycles_elapsed: 4,
                side_effect: self.sax(address),
            },

            Instruction::DcpXIndexedIndirect(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.dcp(self.indexed_indirect_address(index)),
            },

            Instruction::DcpYIndirectIndexed(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.dcp(self.indirect_indexed_address(index).0),
            },

            Instruction::DcpZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.dcp(address as u16),
            },

            Instruction::DcpXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.dcp(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::DcpAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.dcp(address),
            },

            Instruction::DcpXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.dcp(self.absolute_address(address, self.x).0),
            },

            Instruction::DcpYAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.dcp(self.absolute_address(address, self.y).0),
            },

            Instruction::IsbXIndexedIndirect(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.isb(self.indexed_indirect_address(index)),
            },

            Instruction::IsbYIndirectIndexed(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.isb(self.indirect_indexed_address(index).0),
            },

            Instruction::IsbZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.isb(address as u16),
            },

            Instruction::IsbXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.isb(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::IsbAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.isb(address),
            },

            Instruction::IsbXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.isb(self.absolute_address(address, self.x).0),
            },

            Instruction::IsbYAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.isb(self.absolute_address(address, self.y).0),
            },

            Instruction::SloXIndexedIndirect(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.slo(self.indexed_indirect_address(index)),
            },

            Instruction::SloYIndirectIndexed(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.slo(self.indirect_indexed_address(index).0),
            },

            Instruction::SloZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.slo(address as u16),
            },

            Instruction::SloXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.slo(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::SloAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.slo(address),
            },

            Instruction::SloXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.slo(self.absolute_address(address, self.x).0),
            },

            Instruction::SloYAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.slo(self.absolute_address(address, self.y).0),
            },

            Instruction::RlaXIndexedIndirect(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.rla(self.indexed_indirect_address(index)),
            },

            Instruction::RlaYIndirectIndexed(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.rla(self.indirect_indexed_address(index).0),
            },

            Instruction::RlaZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.rla(address as u16),
            },

            Instruction::RlaXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.rla(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::RlaAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.rla(address),
            },

            Instruction::RlaXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.rla(self.absolute_address(address, self.x).0),
            },

            Instruction::RlaYAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.rla(self.absolute_address(address, self.y).0),
            },

            Instruction::SreXIndexedIndirect(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.sre(self.indexed_indirect_address(index) as u16),
            },

            Instruction::SreYIndirectIndexed(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.sre(self.indirect_indexed_address(index).0),
            },

            Instruction::SreZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.sre(address as u16),
            },

            Instruction::SreXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.sre(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::SreAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.sre(address),
            },

            Instruction::SreXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.sre(self.absolute_address(address, self.x).0),
            },

            Instruction::SreYAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.sre(self.absolute_address(address, self.y).0),
            },

            Instruction::RraXIndexedIndirect(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.rra(self.indexed_indirect_address(index)),
            },

            Instruction::RraYIndirectIndexed(index) => CpuResult {
                cycles_elapsed: 8,
                side_effect: self.rra(self.indirect_indexed_address(index).0),
            },

            Instruction::RraZeroPage(address) => CpuResult {
                cycles_elapsed: 5,
                side_effect: self.rra(address as u16),
            },

            Instruction::RraXZeroPage(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.rra(self.zero_page_address(address, self.x) as u16),
            },

            Instruction::RraAbsolute(address) => CpuResult {
                cycles_elapsed: 6,
                side_effect: self.rra(address),
            },

            Instruction::RraXAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.rra(self.absolute_address(address, self.x).0),
            },

            Instruction::RraYAbsolute(address) => CpuResult {
                cycles_elapsed: 7,
                side_effect: self.rra(self.absolute_address(address, self.y).0),
            },
        }
    }

    #[must_use]
    #[inline(always)]
    fn dec(&mut self, address: u16) -> Option<SideEffect> {
        let side_effect =
            self.set_memory_value(address, self.memory[address as usize].overflowing_sub(1).0);
        self.toggle_zero_negative_flag(self.memory[address as usize]);

        side_effect
    }

    #[inline(always)]
    #[must_use]
    fn inc(&mut self, address: u16) -> Option<SideEffect> {
        let side_effect =
            self.set_memory_value(address, self.memory[address as usize].overflowing_add(1).0);
        self.toggle_zero_negative_flag(self.memory[address as usize]);

        side_effect
    }

    #[inline(always)]
    fn rra(&mut self, address: u16) -> Option<SideEffect> {
        let side_effect = self.ror_address(address);
        self.adc(self.memory[address as usize]);

        side_effect
    }

    #[inline(always)]
    fn sre(&mut self, address: u16) -> Option<SideEffect> {
        let mut value = self.memory[address as usize];
        let carry = value.bitand(1) != 0;

        value >>= 1;

        let side_effect = self.set_memory_value(address, value);
        self.a ^= value;

        self.toggle_zero_negative_flag(self.a);
        self.set_carry_flag(carry);

        side_effect
    }

    #[inline(always)]
    fn rla(&mut self, address: u16) -> Option<SideEffect> {
        let side_effect = self.rol_address(address);
        self.a &= self.memory[address as usize];
        self.toggle_zero_negative_flag(self.a);

        side_effect
    }

    #[inline(always)]
    fn slo(&mut self, address: u16) -> Option<SideEffect> {
        let carry = self.memory[address as usize].bitand(0x80) != 0;
        let (value, _) = self.memory[address as usize].overflowing_shl(1);
        let side_effect = self.set_memory_value(address, value);

        self.a |= value;
        self.toggle_zero_negative_flag(self.a);
        self.set_carry_flag(carry);

        side_effect
    }

    #[inline(always)]
    fn isb(&mut self, address: u16) -> Option<SideEffect> {
        let value = self.memory[address as usize].overflowing_add(1).0;
        let side_effect = self.set_memory_value(address, value);

        let (result, sub_overflow) = self
            .a
            .overflowing_sub(value + !self.is_carry_flag_on() as u8); // should account for overflow?

        self.toggle_zero_negative_flag(result);
        self.set_carry_flag(!sub_overflow);
        self.set_overflow_flag((self.a as i8).overflowing_sub(value as i8).1);

        self.a = result;

        side_effect
    }

    #[inline(always)]
    fn dcp(&mut self, address: u16) -> Option<SideEffect> {
        let (value, _) = self.memory[address as usize].overflowing_sub(1);
        let side_effect = self.set_memory_value(address, value);

        let (result, overflow) = self.a.overflowing_sub(value);
        self.toggle_zero_negative_flag(result);
        self.set_carry_flag(!overflow);

        side_effect
    }

    #[inline(always)]
    fn sax(&mut self, address: u16) -> Option<SideEffect> {
        self.set_memory_value(address, self.a & self.x)
    }

    #[inline(always)]
    fn lax(&mut self, value: u8) {
        self.a = value;
        self.x = value;

        self.toggle_zero_negative_flag(value);
    }

    #[inline(always)]
    fn zero_page_address(&self, address: u8, offset: u8) -> u8 {
        address.overflowing_add(offset).0
    }

    #[inline(always)]
    fn zero_page_value(&self, address: u8, offset: u8) -> u8 {
        self.memory[self.zero_page_address(address, offset) as usize]
    }

    #[inline(always)]
    fn absolute_address(&self, address: u16, offset: u8) -> (u16, bool) {
        let mut address_split = address.to_le_bytes();
        let (result, carry1) = address_split[0].overflowing_add(offset);
        address_split[0] = result;

        let (result, carry2) = address_split[1].overflowing_add(carry1 as u8);
        address_split[1] = result;

        let address = u16::from_le_bytes(address_split);

        (address, carry1 || carry2)
    }

    #[inline(always)]
    fn absolute_value(&self, address: u16, offset: u8) -> (u8, bool) {
        let (address, carry) = self.absolute_address(address, offset);

        (self.memory[address as usize], carry)
    }

    #[inline(always)]
    fn ror(&mut self, value: u8) -> u8 {
        let mut value = value;
        let carry = value & 1 != 0;
        value = value / 2 + u8::from(self.is_carry_flag_on()) * 128;

        self.toggle_zero_negative_flag(value);
        self.set_carry_flag(carry);

        value
    }

    #[inline(always)]
    #[must_use]
    fn ror_address(&mut self, address: u16) -> Option<SideEffect> {
        let value = self.ror(self.memory[address as usize]);
        self.set_memory_value(address, value)
    }

    #[inline(always)]
    fn rol(&mut self, value: u8) -> u8 {
        let mut value = value;
        let carry = value & 0x80 != 0;
        value = value.overflowing_mul(2).0 + self.is_carry_flag_on() as u8;
        self.toggle_zero_negative_flag(value);
        self.set_carry_flag(carry);

        value
    }

    #[inline(always)]
    #[must_use]
    fn rol_address(&mut self, address: u16) -> Option<SideEffect> {
        let value = self.rol(self.memory[address as usize]);
        self.set_memory_value(address, value)
    }

    #[inline(always)]
    fn asl(&mut self, value: u8) -> u8 {
        let mut value = value;
        let carry = value & 0x80 != 0;
        value <<= 1;
        self.toggle_zero_negative_flag(value);
        self.set_carry_flag(carry);

        value
    }

    #[inline(always)]
    #[must_use]
    fn asl_address(&mut self, address: u16) -> Option<SideEffect> {
        let value = self.asl(self.memory[address as usize]);
        self.set_memory_value(address, value)
    }

    #[inline(always)]
    fn lsr(&mut self, value: u8) -> u8 {
        let carry = value & 1 != 0;
        let value = value >> 1;
        self.toggle_zero_negative_flag(value);
        self.set_carry_flag(carry);

        value
    }

    #[inline(always)]
    #[must_use]
    fn lsr_address(&mut self, address: u16) -> Option<SideEffect> {
        let value = self.lsr(self.memory[address as usize]);
        self.set_memory_value(address, value)
    }

    #[inline(always)]
    fn compare(&mut self, register_value: u8, value: u8) {
        let (value, overflow) = register_value.overflowing_sub(value);
        self.set_zero_flag(value == 0);
        self.set_negative_flag(value & 0x80 != 0);
        self.set_carry_flag(!overflow);
    }

    #[inline(always)]
    fn sbc(&mut self, value: u8) {
        let (result, not_carry) = self
            .a
            .overflowing_sub(value + !self.is_carry_flag_on() as u8);

        let (_, overflow) =
            (self.a as i8).overflowing_sub(value as i8 + !self.is_carry_flag_on() as i8); // also need to check for edge cases

        self.a = result;
        self.toggle_zero_negative_flag(self.a);
        self.set_carry_flag(!not_carry);
        self.set_overflow_flag(overflow);
    }

    #[inline(always)]
    fn adc(&mut self, value: u8) {
        let (result2, carry2) = value.overflowing_add(self.is_carry_flag_on() as u8);
        let (result, carry) = self.a.overflowing_add(result2);
        let (_, overflow) = (self.a as i8).overflowing_add(value as i8); // also check for overflow with carry?

        self.a = result;
        self.toggle_zero_negative_flag(self.a);
        self.set_carry_flag(carry || carry2);
        self.set_overflow_flag(overflow);
    }

    #[inline(always)]
    fn exor(&mut self, value: u8) {
        self.a ^= value;
        self.toggle_zero_negative_flag(self.a);
    }

    #[inline(always)]
    fn and(&mut self, value: u8) {
        self.a &= value;
        self.toggle_zero_negative_flag(self.a);
    }

    #[inline(always)]
    fn or(&mut self, value: u8) {
        self.a |= value;
        self.toggle_zero_negative_flag(self.a);
    }

    #[inline(always)]
    fn indirect_indexed_address(&self, index: u8) -> (u16, bool) {
        let (low_addr, carry1) = self.memory[index as usize].overflowing_add(self.y);

        let (high_index, _) = index.overflowing_add(1);
        let (high_addr, carry2) = self.memory[high_index as usize].overflowing_add(carry1 as u8);

        let address = u16::from_le_bytes([low_addr, high_addr]);

        (address, carry1 || carry2)
    }

    #[inline(always)]
    fn indirect_indexed_value(&self, index: u8) -> (u8, bool) {
        let (address, overflow) = self.indirect_indexed_address(index);

        (self.memory[address as usize], overflow)
    }

    #[inline(always)]
    fn indexed_indirect_address(&self, index: u8) -> u16 {
        let low_addr = index.overflowing_add(self.x).0;
        let high_addr = low_addr.overflowing_add(1).0;

        u16::from_le_bytes([
            self.memory[low_addr as usize],
            self.memory[high_addr as usize],
        ])
    }

    #[inline(always)]
    fn indexed_indirect_value(&self, index: u8) -> u8 {
        self.memory[self.indexed_indirect_address(index) as usize]
    }

    #[inline(always)]
    fn jump_if(&mut self, condition: bool, offset: u8) -> CpuResult {
        if condition {
            let new_address = self.pc as i16 + (offset as i8) as i16;
            let page_crossed = (self.pc & 0xff00) != ((new_address as u16) & 0xff00);

            self.pc = new_address as u16;
            return cycles(3 + page_crossed as u32);
        }

        return cycles(2);
    }

    #[inline(always)]
    fn push(&mut self, value: u8) {
        self.memory[self.sp as usize + 0x0100] = value;
        self.sp -= 1;
    }

    #[inline(always)]
    fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.memory[self.sp as usize + 0x0100]
    }

    #[inline(always)]
    fn set_negative_flag(&mut self, is_on: bool) {
        self.set_p_flag(7, is_on);
    }

    #[inline(always)]
    fn set_zero_flag(&mut self, is_on: bool) {
        self.set_p_flag(1, is_on);
    }

    #[inline(always)]
    fn set_break_flag(&mut self, is_on: bool) {
        self.p &= !((is_on as u8) << 4);
    }

    #[inline(always)]
    fn set_carry_flag(&mut self, is_on: bool) {
        self.set_p_flag(0, is_on);
    }

    #[inline(always)]
    fn set_overflow_flag(&mut self, is_on: bool) {
        self.set_p_flag(6, is_on);
    }

    #[inline(always)]
    fn set_decimal_flag(&mut self, is_on: bool) {
        self.set_p_flag(3, is_on);
    }

    #[inline(always)]
    fn set_p_flag(&mut self, bit_offset: u8, is_on: bool) {
        let byte = 1 << bit_offset;
        if is_on {
            self.p |= byte;
        } else {
            self.p &= !byte;
        }
    }

    #[inline(always)]
    fn set_interrupt_flag(&mut self, is_on: bool) {
        self.set_p_flag(2, is_on);
    }

    #[inline(always)]
    fn is_overflow_flag_on(&self) -> bool {
        self.p.bitand(1 << 6) != 0
    }

    #[inline(always)]
    fn is_negative_flag_on(&self) -> bool {
        self.p.bitand(1 << 7) != 0
    }

    #[inline(always)]
    fn is_carry_flag_on(&self) -> bool {
        self.p.bitand(1) != 0
    }

    #[inline(always)]
    fn is_zero_flag_on(&self) -> bool {
        self.p.bitand(2) != 0
    }

    #[inline(always)]
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
            sp: 0xff,
            nmi_vector: 0,
            reset_vector: 0,
            irq_vector: 0,
        }
    }

    pub fn load(rom: &InesRom) -> Cpu {
        let mut memory = [0 as u8; 0x10000];

        memory[0x8000..0x8000 + rom.prg_rom_data().len()].copy_from_slice(&rom.prg_rom_data());
        if rom.prg_rom_data().len() == 0x4000 {
            memory[0xc000..0xc000 + rom.prg_rom_data().len()].copy_from_slice(&rom.prg_rom_data());
        }

        // jump to reset vector
        let reset_vector = u16::from_le_bytes([memory[0xfffc], memory[0xfffd]]);
        let nmi_vector = u16::from_le_bytes([memory[0xfffa], memory[0xfffb]]);
        let irq_vector = u16::from_le_bytes([memory[0xfffe], memory[0xffff]]);

        Cpu {
            memory,
            pc: reset_vector,
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
            sp: 0xff,
            nmi_vector,
            reset_vector,
            irq_vector,
        }
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
        cpu.sp = 0xfd;

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
}
