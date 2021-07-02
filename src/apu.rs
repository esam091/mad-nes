use std::ops::{BitAnd, Shl};

use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioQueue, AudioSpecDesired},
    AudioSubsystem,
};

use crate::log_apu;

/*
calculation

time = idx / device.freq
period = 1/f
amplitude = time < period/2 ? volume : -volume
envelope_frequency = 240 / (n + 1)
envelope_period = 1/envelope_frequency

rate = min(0, 15 - time/envelope_period)
volume = volume * rate / 15
*/

struct PulseHandler {
    frequency: f32,
    device_frequency: i32,
    phase: f32,
}

impl PulseHandler {
    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.phase = 0.0;
    }
}

impl AudioCallback for PulseHandler {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        let step = self.frequency / self.device_frequency as f32;

        for x in out {
            *x = if self.phase <= 0.5 {
                0.1
            } else {
                -0.1
                // 0.0
            };

            self.phase = (self.phase + step) % 1.0;
        }
    }
}

pub struct Apu {
    square: AudioDevice<PulseHandler>,
    pulse2_low_timer: u8,
    pulse2_length_and_high_timer: u8,
}

impl Apu {
    pub fn new(audio_subsystem: AudioSubsystem) -> Apu {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };
        let square = audio_subsystem
            .open_playback(None, &desired_spec, |spec| PulseHandler {
                frequency: 0.0,
                device_frequency: spec.freq,
                phase: 0.0,
            })
            .unwrap();

        square.resume();
        Apu {
            square,
            pulse2_length_and_high_timer: 0,
            pulse2_low_timer: 0,
        }
    }

    pub fn write_pulse2_length_and_timer(&mut self, value: u8) {
        log_apu!(
            "Write $4007, timer: {:#04X}, length: {:#04X}",
            value & 0b111,
            (value >> 3) & 0b11111
        );

        self.pulse2_length_and_high_timer = value;

        let note = (value as u16).bitand(0b111).shl(8) | self.pulse2_low_timer as u16;
        let frequency = 1789773.0 / (16.0 * (note + 1u16) as f32);

        self.square.lock().set_frequency(frequency);
    }

    pub fn write_pulse2_sweep(&mut self, value: u8) {}

    pub fn write_pulse2_timer_low(&mut self, value: u8) {
        log_apu!("Write $4006: {:#04X}", value);

        self.pulse2_low_timer = value;
    }

    pub fn write_pulse2_setting(&mut self, value: u8) {
        log_apu!("Write $4004: {:#010b}", value);
    }
}
