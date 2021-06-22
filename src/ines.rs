use crate::ppu::Mirroring;

pub trait Mapper {
    fn write_address(&mut self, prg_rom: &[u8], address: u16, value: u8);
    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8;
    fn pattern_tables<'a>(&self, chr_rom: &'a [u8]) -> Option<(&'a [u8], &'a [u8])>;
    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8>;
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

    let mapper: Box<dyn Mapper> = if bytes[6] >> 4 != 0 {
        Box::new(UNROM::new())
    } else {
        Box::new(NROM {})
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
}
