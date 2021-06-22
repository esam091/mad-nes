use crate::{ppu::Mirroring, utils::hex_string};

pub trait Cartridge {
    fn read_address(&mut self, address: u16) -> u8;
    fn write_address(&mut self, address: u16, value: u8);
    fn mirroring(&self) -> Mirroring;
    fn pattern_tables<'a>(&'a self) -> Option<(&'a [u8], &'a [u8])>;
    fn read_chr(&self, address: u16) -> u8;
    fn has_chr_rom(&self) -> bool;
}

struct UNROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
    current_bank: u8,
}

impl Cartridge for UNROM {
    fn read_address(&mut self, address: u16) -> u8 {
        let bank_size = self.prg_rom.len() / 16384;

        match address {
            0x8000..=0xbfff => {
                self.prg_rom[address as usize - 0x8000 + self.current_bank as usize * 16384]
            }
            0xc000..=0xffff => self.prg_rom[address as usize - 0xc000 + (bank_size - 1) * 16384],
            // 0xc000..=0xffff => self.prg_rom[address as usize],
            _ => panic!("Unimplemented address read: {:#06X}", address),
        }
    }

    fn write_address(&mut self, address: u16, value: u8) {
        let value = value & 0b111;
        self.current_bank = value
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn pattern_tables(&self) -> Option<(&[u8], &[u8])> {
        None
    }

    fn read_chr(&self, address: u16) -> u8 {
        panic!("Should not happen")
    }

    fn has_chr_rom(&self) -> bool {
        false
    }
}

struct NROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
}

impl Cartridge for NROM {
    fn read_address(&mut self, address: u16) -> u8 {
        if self.prg_rom.len() / 16384 == 2 {
            return self.prg_rom[address as usize - 0x8000];
        }

        match address {
            0x8000..=0xbfff => self.prg_rom[address as usize - 0x8000],
            0xc000..=0xffff => self.prg_rom[address as usize - 0xc000],
            _ => panic!("Unhandled address: {:#06X}", address),
        }
    }

    fn write_address(&mut self, _address: u16, _value: u8) {
        // No writes should happen
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn pattern_tables<'a>(&'a self) -> Option<(&'a [u8], &'a [u8])> {
        if self.chr_rom.len() == 0 {
            None
        } else {
            Some((&self.chr_rom[0..0x1000], &self.chr_rom[0x1000..]))
        }
    }

    fn read_chr(&self, address: u16) -> u8 {
        if self.has_chr_rom() {
            self.chr_rom[address as usize]
        } else {
            panic!("Should not happen")
        }
    }

    fn has_chr_rom(&self) -> bool {
        self.chr_rom.len() != 0
    }
}

pub fn load_cartridge<S: Into<String>>(source: S) -> Result<Box<dyn Cartridge>, RomParseError> {
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

    if bytes[6] >> 4 != 0 {
        return Ok(Box::new(UNROM {
            prg_rom,
            chr_rom,
            mirroring,
            current_bank: 0,
        }));
    } else {
        return Ok(Box::new(NROM {
            prg_rom,
            chr_rom,
            mirroring,
        }));
    }
}

pub enum RomParseError {
    NotInes,
    PrgRomTooSmall,
    ChrRomTooSmall,
}
