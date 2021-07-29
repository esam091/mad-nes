use std::{cell::RefCell, collections::HashSet, rc::Rc, u8};

use crate::apu::Apu;
use crate::ppu::VideoMemoryBuffer;
use crate::{
    bus::{JoypadButton, JoypadState, MemoryBuffer, RealBus},
    cpu::Cpu,
    ines::load_cartridge,
    log_ppu,
    ppu::Ppu,
};

pub enum SideEffect {
    Render,
}

pub struct Machine {
    cpu: Cpu,
    cycle_counter: ScanlineCycleCounter,
    pending_cycles: u32,
}

impl Machine {
    fn has_pending_irq(&self) -> bool {
        self.cpu.bus.apu.has_pending_irq() || self.cpu.bus.cartridge.borrow().has_pending_irq()
    }

    pub fn load(file_path: &String, mut apu: Apu) -> Result<Machine, std::io::Error> {
        // todo: fix error type
        let cartridge = load_cartridge(file_path).ok().unwrap();
        let cartridge = Rc::new(RefCell::new(cartridge));

        apu.set_cartridge(cartridge.clone());

        let bus = RealBus {
            memory: [0; 0x10000],
            active_buttons: HashSet::new(),
            joypad_state: JoypadState::Idle,
            ppu: Ppu::new(cartridge.clone()),
            cartridge,
            apu,
        };

        // println!("chr rom {:?}", &rom.chr_rom_data());
        return Ok(Machine {
            cpu: Cpu::load(bus),
            cycle_counter: ScanlineCycleCounter::new(),
            pending_cycles: 0,
        });
    }

    pub fn step(&mut self) -> Option<SideEffect> {
        if self.pending_cycles == 0 {
            if self.has_pending_irq() {
                self.cpu.enter_irq();
            }

            let result = self.cpu.step();
            let cycles = result.cycles_elapsed;
            self.pending_cycles = cycles;
        }

        self.pending_cycles -= 1;

        self.cpu.bus.apu.half_step();

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
