use std::u8;

use crate::ines::InesRom;
use crate::{
    cpu::{self, Cpu, MemoryBuffer},
    ppu::Ppu,
};

pub type VideoMemoryBuffer = [u8; 0x4000];

pub enum SideEffect {
    Render,
}

#[derive(PartialEq, Eq)]
pub struct Machine {
    cycles: u32,
    cycle_counter: CycleCounter,
    cpu: Cpu,
    ppu: Ppu,
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        // todo: fix error type
        let rom = InesRom::load(file_path).ok().unwrap();

        let mut video_memory = [0; 0x4000];
        video_memory[0..rom.chr_rom_data().len()].copy_from_slice(&rom.chr_rom_data());

        // println!("chr rom {:?}", &rom.chr_rom_data());
        return Ok(Machine {
            cycles: 0,
            cycle_counter: CycleCounter::power_on(),

            cpu: Cpu::load(&rom),
            ppu: Ppu::new(video_memory),
        });
    }

    pub fn step(&mut self) -> Option<SideEffect> {
        let result = self.cpu.step();

        match result.side_effect {
            Some(cpu::SideEffect::WritePpuAddr(address)) => {
                self.ppu.write_address(address);
            }
            Some(cpu::SideEffect::WritePpuData(value)) => {
                self.ppu.write_data(value);
            }
            None => {}
        }

        match self.cycle_counter.advance(result.cycles_elapsed) {
            Some(CycleOutput::EnterVblank) => {
                self.cpu.enter_vblank();
                return Some(SideEffect::Render);
            }

            Some(CycleOutput::ExitVblank) => {
                self.cpu.exit_vblank();
                return None;
            }

            _ => {
                return None;
            }
        }
    }

    pub fn get_buffer(&self) -> &MemoryBuffer {
        &self.cpu.get_memory_buffer()
    }

    pub fn get_video_buffer(&self) -> &VideoMemoryBuffer {
        &self.ppu.get_buffer()
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum CycleOutput {
    EnterVblank,
    ExitVblank,
}

#[derive(Eq, PartialEq)]
pub struct CycleCounter {
    state: CycleState,
    cycles: u32,
}

impl CycleCounter {
    pub fn advance(&mut self, cycles: u32) -> Option<CycleOutput> {
        self.cycles += cycles;

        if self.cycles >= self.state.num_cycles() {
            self.cycles %= self.state.num_cycles();

            match self.state {
                CycleState::Rendering => {
                    self.state = CycleState::Vblank;
                    return Some(CycleOutput::EnterVblank);
                }

                CycleState::Vblank => {
                    self.state = CycleState::Rendering;
                    return Some(CycleOutput::ExitVblank);
                }
            }
        }
        None
    }

    pub fn power_on() -> CycleCounter {
        CycleCounter {
            state: CycleState::Rendering,
            cycles: 0,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum CycleState {
    Rendering,
    Vblank,
}

impl CycleState {
    fn num_cycles(&self) -> u32 {
        match self {
            &CycleState::Rendering => 27280,
            &CycleState::Vblank => 2273,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn starting_state() {
        let mut counter = CycleCounter {
            state: CycleState::Rendering,
            cycles: 0,
        };

        let output = counter.advance(8);
        assert_eq!(output, None);
        assert_eq!(counter.state, CycleState::Rendering);
        assert_eq!(counter.cycles, 8);
    }

    #[test]
    fn enter_vblank() {
        let mut counter = CycleCounter {
            state: CycleState::Rendering,
            cycles: 27278,
        };

        let output = counter.advance(3);
        assert_eq!(output, Some(CycleOutput::EnterVblank));
        assert_eq!(counter.state, CycleState::Vblank);
        assert_eq!(counter.cycles, 1);
    }

    #[test]
    fn exit_vblank() {
        let mut counter = CycleCounter {
            state: CycleState::Vblank,
            cycles: 2270,
        };

        let output = counter.advance(4);
        assert_eq!(output, Some(CycleOutput::ExitVblank));
        assert_eq!(counter.state, CycleState::Rendering);
        assert_eq!(counter.cycles, 1);
    }
}
