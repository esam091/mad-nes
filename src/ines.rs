use std::ops::{Shl, Shr};

use crate::ppu::Mirroring;

fn prg_bank_size(bytes: &[u8]) -> usize {
    bytes.len() / 0x4000
}

pub trait Mapper {
    fn write_address(&mut self, prg_rom: &[u8], address: u16, value: u8);
    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8;
    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8>;
    fn mirroring(&self) -> Option<Mirroring>;
    fn scanline_tick(&mut self);
    fn has_pending_irq(&self) -> bool;
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

    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8> {
        if chr_rom.is_empty() {
            None
        } else {
            Some(chr_rom[self.chr_bank * 0x2000 + address as usize])
        }
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn scanline_tick(&mut self) {}

    fn has_pending_irq(&self) -> bool {
        false
    }
}

struct SNROM {
    shift_register: u8,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
    prg_ram: [u8; 0x2000],
}

impl SNROM {
    fn new() -> SNROM {
        SNROM {
            shift_register: 0b10000,
            control: 0b01100,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            prg_ram: [0; 0x2000],
        }
    }

    fn bank_addresses(&self) -> (usize, usize) {
        let bank1: usize;
        let bank2: usize;

        if self.control & 0b10000 == 0 {
            bank1 = (self.chr_bank_0 as usize & !1) * 0x1000;
            bank2 = bank1 + 0x1000;
        } else {
            bank1 = self.chr_bank_0 as usize * 0x1000;
            bank2 = self.chr_bank_1 as usize * 0x1000;
        }

        (bank1, bank2)
    }
}

impl Mapper for SNROM {
    fn write_address(&mut self, _prg_rom: &[u8], address: u16, value: u8) {
        if address >= 0x6000 && address < 0x8000 {
            self.prg_ram[address as usize - 0x6000] = value;
            return;
        }

        // println!("Write MMC1: {:#010b} at {:#06X}", value, address);
        if value & 0x80 != 0 {
            self.shift_register = 0b10000;
            self.control |= 0xc;
            return;
        }

        let value = self.shift_register.shr(1) | (value & 1).shl(4);
        if self.shift_register & 1 == 0 {
            self.shift_register = value;
        } else {
            // println!("MMC1 value: {:#07b}, at: {:#06X}", value, address);

            self.shift_register = 0b10000;

            match address {
                0x8000..=0x9fff => self.control = value,
                0xa000..=0xbfff => self.chr_bank_0 = value,
                0xc000..=0xdfff => self.chr_bank_1 = value,
                0xe000..=0xffff => self.prg_bank = value & 0b1111,

                _ => panic!("Unhandled address: {:#06X}", address),
            }
        }
    }

    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8 {
        if address >= 0x6000 && address < 0x8000 {
            // println!("prg read at {:#06X}", address);
            return self.prg_ram[address as usize - 0x6000];
        }

        let bank_size = prg_bank_size(prg_rom);

        match (self.control & 0b1100) >> 2 {
            0 | 1 => match address {
                0x8000..=0xffff => {
                    prg_rom[address as usize - 0x8000 + (self.prg_bank as usize & !1) * 0x4000]
                }
                _ => panic!("Unhandled address: {:#06X}", address),
            },
            2 => match address {
                0x8000..=0xbfff => prg_rom[address as usize - 0x8000],
                0xc000..=0xffff => {
                    prg_rom[address as usize - 0xc000 + self.prg_bank as usize * 0x4000]
                }
                _ => panic!("Unhandled address: {:#06X}", address),
            },
            3 => match address {
                0x8000..=0xbfff => {
                    prg_rom[address as usize - 0x8000 + self.prg_bank as usize * 0x4000]
                }
                0xc000..=0xffff => prg_rom[address as usize - 0xc000 + (bank_size - 1) * 0x4000],
                _ => panic!("Unhandled address: {:#06X}", address),
            },
            _ => todo!("prg read control: {:#07b}", self.control),
        }
    }

    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8> {
        if chr_rom.is_empty() {
            return None;
        }

        let (bank1, bank2) = self.bank_addresses();

        Some(match address {
            0..=0x0fff => chr_rom[bank1 + address as usize],
            0x1000..=0x1fff => chr_rom[bank2 - 0x1000 + address as usize],
            _ => panic!("CHR address out of range: {:#06X}", address),
        })
    }

    fn mirroring(&self) -> Option<Mirroring> {
        match self.control & 0b11 {
            2 => Some(Mirroring::Vertical),
            3 => Some(Mirroring::Horizontal),
            0 => Some(Mirroring::OneScreenLow),
            1 => Some(Mirroring::OneScreenHigh),
            _ => panic!("Unsupported mirror: {:#04X}", self.control),
        }
    }

    fn scanline_tick(&mut self) {}

    fn has_pending_irq(&self) -> bool {
        false
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

    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8> {
        if chr_rom.len() != 0 {
            Some(chr_rom[address as usize])
        } else {
            None
        }
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn scanline_tick(&mut self) {}

    fn has_pending_irq(&self) -> bool {
        false
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

    fn read_chr_rom(&self, _chr_rom: &[u8], _address: u16) -> Option<u8> {
        None
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn scanline_tick(&mut self) {}

    fn has_pending_irq(&self) -> bool {
        false
    }
}

struct TxROM {
    bank_select: u8,
    r: [u8; 8],
    mirroring: Mirroring,
    irq_reload_value: u8,
    irq_reset: bool,
    irq_enabled: bool,
    current_irq_counter: u8,
    has_pending_irq: bool,
}

impl TxROM {
    fn new() -> TxROM {
        TxROM {
            bank_select: 0,
            r: [0; 8],
            mirroring: Mirroring::Vertical,
            irq_reload_value: 0,
            irq_reset: false,
            irq_enabled: false,
            current_irq_counter: 0,
            has_pending_irq: false,
        }
    }

    fn toggle_pending_irq_if_possible(&mut self) {
        if self.current_irq_counter == 0 && self.irq_enabled {
            self.has_pending_irq = true;
        }
    }
}

impl Mapper for TxROM {
    fn write_address(&mut self, _prg_rom: &[u8], address: u16, value: u8) {
        // if address >= 0xc000 {
        //     println!("TxROM write {:#06X}: {:#04X}", address, value);
        // }
        let is_odd = address % 2 != 0;

        match (address, is_odd) {
            (0x8000..=0x9fff, false) => {
                self.bank_select = value;
            }
            (0x8000..=0x9fff, true) => {
                let r_index = self.bank_select & 0b111;
                self.r[r_index as usize] = if r_index >= 6 {
                    value & 0b00111111
                } else if r_index <= 1 {
                    value & 0b11111110
                } else {
                    value
                };

                // println!("Write bank {}: {:#04X}", r_index, value);
            }
            (0xa000..=0xbfff, false) => {
                self.mirroring = if value & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                }
            }
            (0xc000..=0xdfff, false) => self.irq_reload_value = value,
            (0xc000..=0xdfff, true) => {
                self.irq_reset = true;
                // println!("Reset irq counter");
            }
            (0xe000..=0xffff, false) => {
                // println!("disable IRQ");
                self.irq_enabled = false;
                self.has_pending_irq = false;
            }
            (0xe000..=0xffff, true) => {
                self.irq_enabled = true;
                // println!("enable IRQ");
            }

            _ => {}
        }
    }

    fn read_address(&mut self, prg_rom: &[u8], address: u16) -> u8 {
        let prg_flag = self.bank_select & 0x40 != 0;
        let prg_size = prg_rom.len() / 0x2000;

        let mapped_address: usize = match (address, prg_flag) {
            (0x8000..=0x9fff, false) => address as usize - 0x8000 + self.r[6] as usize * 0x2000,
            (0x8000..=0x9fff, true) => address as usize - 0x8000 + (prg_size - 2) as usize * 0x2000,
            (0xa000..=0xbfff, _) => address as usize - 0xa000 + self.r[7] as usize * 0x2000,
            (0xc000..=0xdfff, false) => {
                address as usize - 0xc000 + (prg_size - 2) as usize * 0x2000
            }
            (0xc000..=0xdfff, true) => address as usize - 0xc000 + self.r[6] as usize * 0x2000,
            (0xe000..=0xffff, _) => address as usize - 0xe000 + (prg_size - 1) as usize * 0x2000,
            _ => panic!("Unhandled address: {:#06X}", address),
        };

        prg_rom[mapped_address as usize]
    }

    fn read_chr_rom(&self, chr_rom: &[u8], address: u16) -> Option<u8> {
        let chr_flag = self.bank_select & 0x80 != 0;
        let chr_bank_size = 0x400;

        if chr_rom.len() == 0 {
            None
        } else {
            let address = address as usize;
            let mapped_address: usize = match (address, chr_flag) {
                (0x0000..=0x07ff, false) => address + self.r[0] as usize * chr_bank_size,
                (0x0800..=0x0fff, false) => address - 0x0800 + self.r[1] as usize * chr_bank_size,
                (0x1000..=0x13ff, false) => address - 0x1000 + self.r[2] as usize * chr_bank_size,
                (0x1400..=0x17ff, false) => address - 0x1400 + self.r[3] as usize * chr_bank_size,
                (0x1800..=0x1bff, false) => address - 0x1800 + self.r[4] as usize * chr_bank_size,
                (0x1c00..=0x1fff, false) => address - 0x1c00 + self.r[5] as usize * chr_bank_size,

                (0x0000..=0x03ff, true) => address + self.r[2] as usize * chr_bank_size,
                (0x0400..=0x07ff, true) => address - 0x0400 + self.r[3] as usize * chr_bank_size,
                (0x0800..=0x0bff, true) => address - 0x0800 + self.r[4] as usize * chr_bank_size,
                (0x0c00..=0x0fff, true) => address - 0x0c00 + self.r[5] as usize * chr_bank_size,
                (0x1000..=0x17ff, true) => address - 0x1000 + self.r[0] as usize * chr_bank_size,
                (0x1800..=0x1fff, true) => address - 0x1800 + self.r[1] as usize * chr_bank_size,
                _ => panic!("Unhandled address: {:#06X}", address),
            };

            Some(chr_rom[mapped_address])
        }
    }

    fn mirroring(&self) -> Option<Mirroring> {
        Some(self.mirroring)
    }

    fn scanline_tick(&mut self) {
        if self.irq_reset {
            self.current_irq_counter = self.irq_reload_value;
            self.irq_reset = false;

            self.toggle_pending_irq_if_possible();
            // println!("Reload counter to {}", self.current_irq_counter);
        } else {
            if self.current_irq_counter == 0 {
                self.current_irq_counter = self.irq_reload_value;

                self.toggle_pending_irq_if_possible();
            } else {
                if self.current_irq_counter > 0 {
                    self.current_irq_counter -= 1;
                }
                // dbg!(self.current_irq_counter);
                self.toggle_pending_irq_if_possible();
            }
        }
    }

    fn has_pending_irq(&self) -> bool {
        self.has_pending_irq
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

    pub fn read_chr_rom(&self, address: u16) -> Option<u8> {
        self.mapper.read_chr_rom(&self.chr_rom, address)
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring().unwrap_or(self.mirroring)
    }

    pub fn scanline_tick(&mut self) {
        self.mapper.scanline_tick()
    }

    pub fn has_pending_irq(&self) -> bool {
        self.mapper.has_pending_irq()
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
        4 => Box::new(TxROM::new()),
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
