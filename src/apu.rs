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

struct PulseEnvelope {
    duty: u8,
    volume: u8,
    loops_playback: bool,
    constant_volume: bool,
}

impl PulseEnvelope {
    fn new() -> PulseEnvelope {
        PulseEnvelope {
            duty: 0,
            volume: 0,
            loops_playback: false,
            constant_volume: false,
        }
    }

    fn from_flags(flag: u8) -> PulseEnvelope {
        let duty = flag.bitand(0b11000000).shr(6);
        let loops_playback = flag.bitand(0b100000) != 0;
        let constant_volume = flag.bitand(0b10000) != 0;
        let volume = flag.bitand(0b1111);

        PulseEnvelope {
            duty,
            loops_playback,
            constant_volume,
            volume,
        }
    }
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

    fn new_value(&self, timer: u16, elapsed_time: f32) -> u16 {
        // return timer;
        let sweep_time = 120.0 / (self.period as f32 + 1.0);
        if !self.enabled && elapsed_time < sweep_time {
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
    sweep: Sweep,
    timer: u16,
    envelope: PulseEnvelope,
}

impl PulseHandler {
    fn set_sweep(&mut self, sweep: Sweep) {
        self.sweep = sweep;
    }

    fn set_envelope(&mut self, envelope: PulseEnvelope) {
        self.envelope = envelope
    }
}

impl AudioCallback for PulseHandler {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        if self.timer > 8 {
            let time_interval = 1.0 / self.device_frequency as f32;
            let decay_frequency = 240.0 / (self.envelope.volume as f32 + 1.0);
            let decay_period = 1.0 / decay_frequency;
            let wave_period = 1.0 / note_frequency_from_period(self.timer);

            // dbg!(wave_period, wave.frequency);

            for x in out {
                let sweeped_timer = self.sweep.new_value(self.timer, self.elapsed_time);
                let sweep_period = 1.0 / note_frequency_from_period(sweeped_timer);
                let phase = (self.elapsed_time % sweep_period) / sweep_period;
                // dbg!(phase, self.elapsed_time);

                let mut volume = match self.envelope.duty {
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
                    _ => panic!("Unhandled duty: {}", self.envelope.duty),
                };

                if !self.envelope.constant_volume {
                    let current_decay =
                        (15.0 - (self.elapsed_time / decay_period).floor()).max(0.0);

                    volume *= current_decay / 15.0;
                } else {
                    volume *= self.envelope.volume as f32 / 15.0;
                }

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
                sweep: Sweep::new(),
                timer: 0,
                envelope: PulseEnvelope::new(),
            })
            .unwrap();

        let pulse1_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| PulseHandler {
                device_frequency: spec.freq,
                elapsed_time: 0.0,
                sweep: Sweep::new(),
                timer: 0,
                envelope: PulseEnvelope::new(),
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

        let mut device = self.pulse1_device.lock();
        device.timer = note;
        device.elapsed_time = 0.0;
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
        handler.set_envelope(PulseEnvelope::from_flags(value));
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

        let mut device = self.pulse2_device.lock();
        device.timer = note;
        device.elapsed_time = 0.0;
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
        handler.set_envelope(PulseEnvelope::from_flags(value));
    }
}
