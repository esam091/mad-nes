use std::{collections::HashSet, u8};

use crate::{
    cpu::{self, Bus, Cpu, MemoryBuffer},
    ppu::Ppu,
};
use crate::{ines::InesRom, ppu::VideoMemoryBuffer};

pub enum SideEffect {
    Render,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum JoypadState {
    Polling,
    Ready(JoypadButton),
    Idle,
}

#[derive(PartialEq, Eq)]
struct RealBus {
    is_key_pressed: bool,
    active_buttons: HashSet<JoypadButton>,
    joypad_state: JoypadState,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum JoypadButton {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Bus for RealBus {
    fn read_address(&mut self, address: u16) -> u8 {
        if address == 0x4016 {
            let value: u8 = match self.joypad_state {
                JoypadState::Ready(button) => {
                    if self.active_buttons.contains(&button) {
                        1
                    } else {
                        0
                    }
                }
                JoypadState::Polling => 0,
                JoypadState::Idle => 1,
            };

            let next_state = match self.joypad_state {
                JoypadState::Ready(button) => match button {
                    JoypadButton::A => JoypadState::Ready(JoypadButton::B),
                    JoypadButton::B => JoypadState::Ready(JoypadButton::Select),
                    JoypadButton::Select => JoypadState::Ready(JoypadButton::Start),
                    JoypadButton::Start => JoypadState::Ready(JoypadButton::Up),
                    JoypadButton::Up => JoypadState::Ready(JoypadButton::Down),
                    JoypadButton::Down => JoypadState::Ready(JoypadButton::Left),
                    JoypadButton::Left => JoypadState::Ready(JoypadButton::Right),
                    JoypadButton::Right => JoypadState::Idle,
                },
                _ => self.joypad_state,
            };

            self.joypad_state = next_state;

            return value;
        }

        0
    }

    fn write_address(&mut self, address: u16, value: u8) {
        if address == 0x4016 {
            match (self.joypad_state, value) {
                (JoypadState::Idle, 1) => self.joypad_state = JoypadState::Polling,
                (JoypadState::Polling, 0) => {
                    self.joypad_state = JoypadState::Ready(JoypadButton::A)
                }
                _ => {}
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct Machine {
    cycles: u32,
    cycle_counter: CycleCounter,
    cpu: Cpu,
    ppu: Ppu,
    bus: RealBus,
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
            bus: RealBus {
                is_key_pressed: false,
                active_buttons: HashSet::new(),
                joypad_state: JoypadState::Idle,
            },
        });
    }

    pub fn step(&mut self) -> Option<SideEffect> {
        let result = self.cpu.step(&mut self.bus);

        if let Some(side_effect) = result.side_effect {
            // println!("side effect {:#04X?}", side_effect);

            match side_effect {
                cpu::SideEffect::WritePpuAddr(address) => {
                    self.ppu.write_address(address);
                }
                cpu::SideEffect::WritePpuData(value) => {
                    self.ppu.write_data(value);
                }

                cpu::SideEffect::WriteOamAddr(address) => {
                    self.ppu.set_oam_address(address);
                }
                cpu::SideEffect::WriteOamData(data) => {
                    self.ppu.write_oam_data(data);
                }
                cpu::SideEffect::OamDma(byte) => {
                    let starting_address = byte as usize * 0x100;
                    let slice =
                        &self.cpu.get_memory_buffer()[starting_address..=starting_address + 0xff];
                    self.ppu.copy_oam_data(slice);
                    self.cycle_counter.advance(557);
                }
                cpu::SideEffect::ClearAddressLatch => {
                    self.ppu.clear_address_latch();
                }
                cpu::SideEffect::SetPpuControl(value) => {
                    self.ppu.set_control_flag(value);
                }
            }
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

    pub fn get_ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn get_cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn set_current_key(&mut self, a: bool) {
        self.bus.is_key_pressed = a;
    }

    pub fn set_active_buttons(&mut self, buttons: HashSet<JoypadButton>) {
        self.bus.active_buttons = buttons;
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
