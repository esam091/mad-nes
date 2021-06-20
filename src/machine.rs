use std::{collections::HashSet, u8};

use crate::{
    bus::{JoypadButton, JoypadState, MemoryBuffer, RealBus},
    cpu::Cpu,
    log_ppu,
    ppu::Ppu,
};
use crate::{ines::InesRom, ppu::VideoMemoryBuffer};

pub enum SideEffect {
    Render,
}

#[derive(PartialEq, Eq)]
pub struct Machine {
    cpu: Cpu,
    cycle_counter: ScanlineCycleCounter,
    pending_cycles: u32,
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        // todo: fix error type
        let rom = InesRom::load(file_path).ok().unwrap();

        let mut video_memory = [0; 0x4000];
        video_memory[0..rom.chr_rom_data().len()].copy_from_slice(&rom.chr_rom_data());

        let bus = RealBus {
            memory: [0; 0x10000],
            active_buttons: HashSet::new(),
            joypad_state: JoypadState::Idle,
            ppu: Ppu::new(video_memory, rom.mirroring()),
        };

        // println!("chr rom {:?}", &rom.chr_rom_data());
        return Ok(Machine {
            cpu: Cpu::load(&rom, bus),
            cycle_counter: ScanlineCycleCounter::new(),
            pending_cycles: 0,
        });
    }

    pub fn step(&mut self) -> Option<SideEffect> {
        if self.pending_cycles == 0 {
            let result = self.cpu.step();
            let cycles = result.cycles_elapsed + result.has_dma as u32 * 514;
            self.pending_cycles = cycles;
        }

        self.pending_cycles -= 1;

        let mut should_render = false;
        for _ in 0..3 {
            let result = self.cpu.bus.ppu.step();
            should_render = should_render || result;
        }

        if should_render {
            if self.cpu.bus.ppu.generates_nmi_at_vblank() {
                log_ppu!("Enter vblank");
                self.cpu.enter_nmi();
            }

            return Some(SideEffect::Render);
        }
        None
    }

    pub fn get_buffer(&self) -> &MemoryBuffer {
        &self.cpu.get_memory_buffer()
    }

    pub fn get_video_buffer(&self) -> &VideoMemoryBuffer {
        &self.cpu.bus.ppu.get_buffer()
    }

    pub fn get_ppu(&self) -> &Ppu {
        &self.cpu.bus.ppu
    }

    pub fn get_cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn set_active_buttons(&mut self, buttons: HashSet<JoypadButton>) {
        self.cpu.bus.active_buttons = buttons;
    }
}

#[derive(PartialEq, Eq)]
struct ScanlineCycleCounter {
    scanline_cycles_left: u32,
}

impl ScanlineCycleCounter {
    fn advance(&mut self, cycles: u32) -> bool {
        if cycles >= self.scanline_cycles_left {
            self.scanline_cycles_left = 341 + self.scanline_cycles_left - cycles;
            return true;
        } else {
            self.scanline_cycles_left -= cycles;
            return false;
        }
    }

    fn new() -> ScanlineCycleCounter {
        ScanlineCycleCounter {
            scanline_cycles_left: 341,
        }
    }
}

#[cfg(test)]
mod test {}
