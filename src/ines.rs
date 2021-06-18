use crate::ppu::Mirroring;

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
