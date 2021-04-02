use std::u8;

use crate::cpu::{self, Cpu, MemoryBuffer};
use crate::ines::InesRom;

pub type VideoMemoryBuffer = [u8; 0x4000];

pub enum SideEffect {
    Render,
}

#[derive(PartialEq, Eq)]
pub struct Machine {
    video_memory: VideoMemoryBuffer,
    video_addr1: Option<u8>,
    video_addr2: Option<u8>,
    video_offset: u8,

    cycles: u32,

    cpu: Cpu,
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        // todo: fix error type
        let rom = InesRom::load(file_path).ok().unwrap();

        let mut memory = [0 as u8; 0x10000];
        memory[0x8000..0x8000 + rom.prg_rom_data().len()].copy_from_slice(&rom.prg_rom_data());

        let vector_positions = rom.prg_rom_data().len() - 6;

        memory[0xfffa..].copy_from_slice(&rom.prg_rom_data()[vector_positions..]);

        // jump to reset vector
        let initial_address = u16::from_le_bytes([memory[0xfffc], memory[0xfffd]]);

        let mut video_memory = [0; 0x4000];
        video_memory[..rom.chr_rom_data().len()].copy_from_slice(&rom.chr_rom_data());

        return Ok(Machine {
            video_memory: [0; 0x4000],
            video_addr1: None,
            video_addr2: None,
            video_offset: 0,

            cycles: 0,

            cpu: Cpu::new(memory, initial_address),
        });
    }

    pub fn step(&mut self) -> Option<SideEffect> {
        let result = self.cpu.step();

        match result.side_effect {
            Some(cpu::SideEffect::WritePpuAddr(address)) => {
                match (self.video_addr1, self.video_addr2) {
                    (None, None) => self.video_addr1 = Some(address),
                    (Some(_), None) => self.video_addr2 = Some(address),
                    (Some(_), Some(_)) => {
                        self.video_addr1 = Some(address);
                        self.video_addr2 = None;
                        self.video_offset = 0;
                    }
                    (None, Some(_)) => panic!("Unlikely 0x2006 condition"),
                }
            }
            Some(cpu::SideEffect::WritePpuData(value)) => {
                match (self.video_addr1, self.video_addr2) {
                    (Some(addr1), Some(addr2)) => {
                        let address = u16::from_be_bytes([addr1, addr2]);
                        self.video_memory[self.video_offset as usize + address as usize] = value;
                        self.video_offset += 1;
                    }
                    _ => panic!("Video registry error"),
                }
            }
            _ => {}
        }

        self.cycles += result.cycles_elapsed;

        if self.cycles >= 3000 {
            self.cycles %= 3000;

            return Some(SideEffect::Render);
        }

        None
    }

    pub fn get_buffer(&self) -> &MemoryBuffer {
        &self.cpu.get_memory_buffer()
    }

    pub fn get_video_buffer(&self) -> &VideoMemoryBuffer {
        &self.video_memory
    }
}
