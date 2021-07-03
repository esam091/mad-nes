use std::ops::{BitAnd, Shl, Shr};

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

struct PulseWave {
    frequency: f32,
    loops_playback: bool,
    is_constant: bool,
    volume: u8,
}

struct Sweep {
    enabled: bool,
    period: u8,
    shift: u8,
    negate: bool,
}

impl Sweep {
    fn new() -> Sweep {
        Sweep {
            enabled: false,
            period: 0,
            shift: 0,
            negate: false,
        }
    }

    fn from_flags(flag: u8) -> Sweep {
        let enabled = flag & 0x80 != 0;
        let shift = flag & 0b111;
        let negate = flag & 0b1000 != 0;
        let period = flag.bitand(0b01110000).shr(4);

        Sweep {
            enabled,
            shift,
            negate,
            period,
        }
    }

    fn new_value(&self, timer: u16) -> u16 {
        if !self.enabled {
            return timer;
        }

        let add = timer >> self.shift;
        if self.negate {
            timer - add - 1
        } else {
            timer + add
        }
    }
}

struct PulseHandler {
    elapsed_time: f32,
    device_frequency: i32,
    wave: Option<PulseWave>,
    volume: u8,
    duty: u8,
    sweep: Sweep,
    timer: u16,
}

impl PulseHandler {
    fn set_wave(&mut self, wave: PulseWave) {
        self.wave = Some(wave);
        self.elapsed_time = 0.0;
    }

    fn set_sweep(&mut self, sweep: Sweep) {
        self.sweep = sweep;
    }
}

impl AudioCallback for PulseHandler {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        if let Some(wave) = &self.wave {
            let time_interval = 1.0 / self.device_frequency as f32;
            let decay_frequency = 240.0 / (self.volume as f32 + 1.0);
            let decay_period = 1.0 / decay_frequency;
            let wave_period = 1.0 / note_frequency_from_period(self.timer);
            let sweep_zz = self.sweep.new_value(self.timer);
            let sweep_period = 1.0 / note_frequency_from_period(sweep_zz);
            // dbg!(wave_period, wave.frequency);

            for x in out {
                let phase = (self.elapsed_time % sweep_period) / sweep_period;
                // dbg!(phase, self.elapsed_time);

                let mut volume = match self.duty {
                    0 => {
                        if phase <= 0.125 {
                            0.1
                        } else {
                            -0.1
                        }
                    }
                    1 => {
                        if phase <= 0.25 {
                            0.1
                        } else {
                            -0.1
                        }
                    }
                    2 => {
                        if phase <= 0.5 {
                            0.1
                        } else {
                            -0.1
                        }
                    }
                    3 => {
                        if phase <= 0.25 {
                            -0.1
                        } else {
                            0.1
                        }
                    }
                    _ => panic!("Unhandled duty: {}", self.duty),
                };
                let current_decay = (15.0 - (self.elapsed_time / decay_period).floor()).max(0.0);

                volume *= current_decay / 15.0;

                *x = volume;
                self.elapsed_time += time_interval;
                // dbg!(x);
            }
        } else {
            for x in out {
                *x = 0.0;
            }
        }
    }
}

pub struct Apu {
    pulse1_device: AudioDevice<PulseHandler>,
    pulse2_device: AudioDevice<PulseHandler>,
    pulse2_low_timer: u8,
    pulse2_length_and_high_timer: u8,
    pulse2_setting: u8,

    pulse1_low_timer: u8,
    pulse1_length_and_high_timer: u8,
    pulse1_setting: u8,
}

fn note_frequency_from_period(period: u16) -> f32 {
    1789773.0 / (16.0 * (period + 1u16) as f32)
}

impl Apu {
    pub fn new(audio_subsystem: AudioSubsystem) -> Apu {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };
        let pulse2_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| PulseHandler {
                device_frequency: spec.freq,
                elapsed_time: 0.0,
                wave: None,
                volume: 0,
                duty: 0,
                sweep: Sweep::new(),
                timer: 0,
            })
            .unwrap();

        let pulse1_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| PulseHandler {
                device_frequency: spec.freq,
                elapsed_time: 0.0,
                wave: None,
                volume: 0,
                duty: 0,
                sweep: Sweep::new(),
                timer: 0,
            })
            .unwrap();

        pulse1_device.resume();
        pulse2_device.resume();
        Apu {
            pulse2_device,
            pulse1_device,
            pulse2_length_and_high_timer: 0,
            pulse2_low_timer: 0,
            pulse2_setting: 0,
            pulse1_length_and_high_timer: 0,
            pulse1_low_timer: 0,
            pulse1_setting: 0,
        }
    }

    pub fn write_pulse1_length_and_timer(&mut self, value: u8) {
        log_apu!(
            "Write $4003, timer: {:#04X}, length: {:#04X}",
            value & 0b111,
            (value >> 3) & 0b11111
        );

        self.pulse1_length_and_high_timer = value;

        let note = (value as u16).bitand(0b111).shl(8) | self.pulse1_low_timer as u16;
        let frequency = 1789773.0 / (16.0 * (note + 1u16) as f32);

        let wave = PulseWave {
            frequency,
            is_constant: false,
            loops_playback: false,
            volume: self.pulse1_setting & 0b1111,
        };

        let mut device = self.pulse1_device.lock();
        device.set_wave(wave);
        device.timer = note;
    }

    pub fn write_pulse1_sweep(&mut self, value: u8) {
        log_apu!("Write $4001: {:#04X}", value);
        self.pulse1_device
            .lock()
            .set_sweep(Sweep::from_flags(value));
    }

    pub fn write_pulse1_timer_low(&mut self, value: u8) {
        log_apu!("Write $4002: {:#04X}", value);

        self.pulse1_low_timer = value;
    }

    pub fn write_pulse1_setting(&mut self, value: u8) {
        log_apu!("Write $4000: {:#010b}", value);
        self.pulse1_setting = value;

        let mut handler = self.pulse1_device.lock();
        handler.volume = value & 0b1111;
        handler.duty = value.bitand(0b11000000).shr(6);
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

        let wave = PulseWave {
            frequency,
            is_constant: false,
            loops_playback: false,
            volume: self.pulse2_setting & 0b1111,
        };

        let mut device = self.pulse2_device.lock();
        device.set_wave(wave);
        device.timer = note;
    }

    pub fn write_pulse2_sweep(&mut self, value: u8) {
        log_apu!("write $4005: {:#04X}", value);
        self.pulse2_device
            .lock()
            .set_sweep(Sweep::from_flags(value));
    }

    pub fn write_pulse2_timer_low(&mut self, value: u8) {
        log_apu!("Write $4006: {:#04X}", value);

        self.pulse2_low_timer = value;
    }

    pub fn write_pulse2_setting(&mut self, value: u8) {
        log_apu!("Write $4004: {:#010b}", value);
        self.pulse2_setting = value;

        let mut handler = self.pulse2_device.lock();
        handler.volume = value & 0b1111;
        handler.duty = value.bitand(0b11000000).shr(6);
    }
}
