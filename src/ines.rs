use std::ops::{Shl, Shr};

use crate::ppu::Mirroring;

fn prg_bank_size(bytes: &[u8]) -> usize {
    bytes.len() / 0x4000
}

pub trait Mapper {
    fn write_address(&mut self, prg_rom: &[u8], address: u16, value: u8);
    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8;
    fn pattern_tables<'a>(&self, chr_rom: &'a [u8]) -> Option<(&'a [u8], &'a [u8])>;
    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8>;
}

struct CNROM {
    chr_bank: usize,
}

impl CNROM {
    fn new() -> CNROM {
        CNROM { chr_bank: 0 }
    }
}

impl Mapper for CNROM {
    fn write_address(&mut self, _prg_rom: &[u8], address: u16, value: u8) {
        if address >= 0x8000 {
            self.chr_bank = value as usize & 3;
        }
    }

    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8 {
        if prg_bank_size(prg_rom) == 2 || address < 0xc000 {
            prg_rom[address as usize - 0x8000]
        } else {
            prg_rom[address as usize - 0xc000]
        }
    }

    fn pattern_tables<'a>(&self, chr_rom: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        if chr_rom.is_empty() {
            None
        } else {
            let start = self.chr_bank * 0x2000;
            Some((
                &chr_rom[start..start + 0x1000],
                &chr_rom[start + 0x1000..start + 0x2000],
            ))
        }
    }

    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8> {
        if chr_rom.is_empty() {
            None
        } else {
            Some(chr_rom[self.chr_bank * 0x2000 + address as usize])
        }
    }
}

struct SNROM {
    shift_register: u8,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}

impl SNROM {
    fn new() -> SNROM {
        SNROM {
            shift_register: 0b10000,
            control: 0,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }
}

impl Mapper for SNROM {
    fn write_address(&mut self, _prg_rom: &[u8], address: u16, value: u8) {
        if address < 0x8000 {
            // TODO: handle PRG RAM writes
            return;
        }

        println!("Write MMC1: {:#010b} at {:#06X}", value, address);
        if value & 0x80 != 0 {
            self.shift_register = 0b10000;
            return;
        }

        let value = self.shift_register.shr(1) | (value & 1).shl(4);
        if self.shift_register & 1 == 0 {
            self.shift_register = value;
        } else {
            println!("MMC1 value: {:#07b}, at: {:#06X}", value, address);

            self.shift_register = 0b10000;

            match address {
                0x6000..=0x7fff => {} //TODO
                0x8000..=0x9fff => self.control = value,
                0xa000..=0xbfff => self.chr_bank_0 = value,
                0xc000..=0xdfff => self.chr_bank_1 = value,
                0xe000..=0xffff => {
                    self.prg_bank = value & 0b1111;
                    if (self.control & 0b1100) >> 2 <= 1 {
                        self.prg_bank &= !1;
                    }
                }
                _ => panic!("Unhandled address: {:#06X}", address),
            }
        }
    }

    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8 {
        // (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
        //     2: fix first bank at $8000 and switch 16 KB bank at $C000;
        //     3: fix last bank at $C000 and switch 16 KB bank at $8000)
        let bank_size = prg_bank_size(prg_rom);

        match (self.control & 0b1100) >> 2 {
            0 | 1 => prg_rom[address as usize - 0x8000],
            3 => match address {
                0x8000..=0xbfff => {
                    prg_rom[address as usize - 0x8000 + self.prg_bank as usize * 0x4000]
                }
                0xc000..=0xffff => prg_rom[address as usize - 0xc000 + (bank_size - 1) * 0x4000],
                _ => panic!("Unhandled address: {:#06X}", address),
            },
            _ => todo!("prg read control"),
        }
    }

    fn pattern_tables<'a>(&self, chr_rom: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        // todo!()
        // dbg!(chr_rom.len());
        None
    }

    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8> {
        None
    }
}

struct NROM;

impl Mapper for NROM {
    fn write_address(&mut self, _prg_rom: &[u8], _address: u16, _value: u8) {
        // ignore
    }

    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8 {
        if prg_rom.len() / 16384 == 2 {
            return prg_rom[address as usize - 0x8000];
        }

        match address {
            0x8000..=0xbfff => prg_rom[address as usize - 0x8000],
            0xc000..=0xffff => prg_rom[address as usize - 0xc000],
            _ => panic!("Unhandled address: {:#06X}", address),
        }
    }

    fn pattern_tables<'a>(&self, chr_rom: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        if chr_rom.len() == 0 {
            None
        } else {
            Some((&chr_rom[0..0x1000], &chr_rom[0x1000..]))
        }
    }

    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8> {
        if chr_rom.len() != 0 {
            Some(chr_rom[address as usize])
        } else {
            None
        }
    }
}

struct UNROM {
    current_bank: u8,
}

impl UNROM {
    fn new() -> UNROM {
        UNROM { current_bank: 0 }
    }
}

impl Mapper for UNROM {
    fn write_address(&mut self, _prg_rom: &[u8], _address: u16, value: u8) {
        let value = value & 0b111;
        self.current_bank = value;
    }

    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8 {
        let bank_size = prg_rom.len() / 16384;

        match address {
            0x8000..=0xbfff => {
                prg_rom[address as usize - 0x8000 + self.current_bank as usize * 16384]
            }
            0xc000..=0xffff => prg_rom[address as usize - 0xc000 + (bank_size - 1) * 16384],
            // 0xc000..=0xffff => self.prg_rom[address as usize],
            _ => panic!("Unimplemented address read: {:#06X}", address),
        }
    }

    fn pattern_tables<'a>(&self, _chr_rom: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        None
    }

    fn read_chr_rom(&self, _chr_rom: &[u8], _address: u16) -> Option<u8> {
        None
    }
}

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn write_address(&mut self, address: u16, value: u8) {
        self.mapper.write_address(&self.prg_rom, address, value)
    }

    pub fn read_address(&mut self, address: u16) -> u8 {
        self.mapper.read_address(&self.prg_rom, address)
    }

    pub fn pattern_tables<'a>(&'a self) -> Option<(&'a [u8], &'a [u8])> {
        self.mapper.pattern_tables(&self.chr_rom)
    }

    pub fn read_chr_rom(&self, address: u16) -> Option<u8> {
        self.mapper.read_chr_rom(&self.chr_rom, address)
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

pub fn load_cartridge<S: Into<String>>(source: S) -> Result<Cartridge, RomParseError> {
    let bytes: Vec<u8> = std::fs::read(source.into()).unwrap().into_iter().collect();

    if bytes[0..4] != [0x4e, 0x45, 0x53, 0x1a] {
        return Err(RomParseError::NotInes);
    }

    let prg_rom_size = bytes[4] as usize * 0x4000;
    let chr_rom_size = bytes[5] as usize * 0x2000;

    let prg_and_chr_data = &bytes[0x10..];

    if prg_rom_size > prg_and_chr_data.len() {
        return Err(RomParseError::PrgRomTooSmall);
    }

    let prg_rom: Vec<u8> = Vec::from(&prg_and_chr_data[0..prg_rom_size]);

    let chr_data = &prg_and_chr_data[prg_rom_size..];

    if chr_rom_size > chr_data.len() {
        return Err(RomParseError::ChrRomTooSmall);
    }

    let chr_rom: Vec<u8> = Vec::from(chr_data);

    let mirroring = if bytes[6] & 1 == 0 {
        Mirroring::Horizontal
    } else {
        Mirroring::Vertical
    };

    let mapper_number = bytes[6] >> 4;
    let mapper: Box<dyn Mapper> = match bytes[6] >> 4 {
        0 => Box::new(NROM {}),
        1 => Box::new(SNROM::new()),
        2 => Box::new(UNROM::new()),
        3 => Box::new(CNROM::new()),
        _ => return Err(RomParseError::UnsupportedMapper(mapper_number)),
    };

    Ok(Cartridge {
        prg_rom,
        chr_rom,
        mirroring,
        mapper,
    })
}

pub enum RomParseError {
    NotInes,
    PrgRomTooSmall,
    ChrRomTooSmall,
    UnsupportedMapper(u8),
}
