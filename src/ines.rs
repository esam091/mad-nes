use crate::{ppu::Mirroring, utils::hex_string};

pub trait Cartridge {
    fn read_address(&mut self, address: u16) -> u8;
    fn write_address(&mut self, address: u16, value: u8);
    fn mirroring(&self) -> Mirroring;
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

    fn write_address(&mut self, address: u16, value: u8) {
        // No writes should happen
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

pub fn load_cartridge<S: Into<String>>(source: S) -> Result<Box<Cartridge>, RomParseError> {
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

    return Ok(Box::new(NROM {
        prg_rom,
        chr_rom,
        mirroring,
    }));
}

pub struct InesRom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
}

pub enum RomParseError {
    NotInes,
    PrgRomTooSmall,
    ChrRomTooSmall,
}

impl InesRom {
    pub fn load<S: Into<String>>(source: S) -> Result<InesRom, RomParseError> {
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

        return Ok(InesRom {
            prg_rom,
            chr_rom,
            mirroring,
        });
    }

    pub fn prg_rom_data(&self) -> &Vec<u8> {
        &self.prg_rom
    }

    pub fn chr_rom_data(&self) -> &Vec<u8> {
        &self.chr_rom
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
