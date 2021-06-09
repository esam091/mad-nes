use std::collections::HashSet;

use crate::ppu::Ppu;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JoypadState {
    Polling,
    Ready(JoypadButton),
    Idle,
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

pub trait BusTrait {
    fn read_address(&mut self, address: u16) -> u8;
    fn write_address(&mut self, address: u16, value: u8);
}

pub type MemoryBuffer = [u8; 0x10000];

#[derive(PartialEq, Eq)]
pub struct RealBus {
    pub memory: MemoryBuffer,
    pub active_buttons: HashSet<JoypadButton>,
    pub joypad_state: JoypadState,
    pub ppu: Ppu,
}

impl BusTrait for RealBus {
    fn read_address(&mut self, address: u16) -> u8 {
        match address {
            0x2002 => {
                self.ppu.clear_address_latch();
                return self.memory[address as usize];
            }
            0x4016 => {
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
            _ => self.memory[address as usize],
        }
    }

    fn write_address(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => {
                self.ppu.set_control_flag(value);
                self.memory[address as usize] = value;
            }
            0x2005 => self.ppu.write_scroll(value),
            0x2006 => self.ppu.write_address(value),
            0x2007 => self.ppu.write_data(value),
            0x2003 => self.ppu.set_oam_address(value),
            0x2004 => self.ppu.write_oam_data(value),
            0x4014 => {
                let starting_address = value as usize * 0x100;
                let slice = &self.memory[starting_address..=starting_address + 0xff];
                self.ppu.copy_oam_data(slice);

                // TODO: advance cycles
            }
            0x4016 => match (self.joypad_state, value) {
                (JoypadState::Idle, 1) => self.joypad_state = JoypadState::Polling,
                (JoypadState::Polling, 0) => {
                    self.joypad_state = JoypadState::Ready(JoypadButton::A)
                }
                _ => {}
            },
            _ => self.memory[address as usize] = value,
        }
    }
}
