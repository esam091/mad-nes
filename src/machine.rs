use std::u8;

use crate::cpu::{self, Cpu};
use crate::instruction::Instruction;

pub type MemoryBuffer = [u8; 0x10000];
pub type VideoMemoryBuffer = [u8; 0x4000];

pub enum SideEffect {
    Render,
}

#[derive(PartialEq, Eq)]
pub struct Machine {
    memory: MemoryBuffer,
    pc: u16,
    a: u8,
    x: u8,

    zero_flag: bool,

    video_memory: VideoMemoryBuffer,
    video_addr1: Option<u8>,
    video_addr2: Option<u8>,
    video_offset: u8,

    cycles: u32,

    cpu: Cpu,
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        // since we're using a legit hello world ROM as a prototype, we don't need to configure stuff from headers
        let bytes: Vec<u8> = std::fs::read(file_path)?.into_iter().skip(16).collect();

        let mut memory = [0 as u8; 0x10000];

        for i in 0..bytes.len() - 16 {
            memory[0x8000 + i] = bytes[i];
        }

        // Copy the reset vector data
        // The data was located at 400c but we subtract by 0x10 since we skipped 16 bytes in the header
        memory[0xfffc] = bytes[0x3ffc];
        memory[0xfffd] = bytes[0x3ffd];

        // jump to reset vector
        let initial_address = u16::from_le_bytes([memory[0xfffc], memory[0xfffd]]);

        return Ok(Machine {
            memory: memory,
            pc: initial_address,
            a: 0,
            x: 0,

            zero_flag: false,

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
        &self.memory
    }

    pub fn get_video_buffer(&self) -> &VideoMemoryBuffer {
        &self.video_memory
    }
}
