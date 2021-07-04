use std::ops::{BitAnd, Shl, Shr};

use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    AudioSubsystem,
};

use crate::log_apu;

/*
     |  0   1   2   3   4   5   6   7    8   9   A   B   C   D   E   F
-----+----------------------------------------------------------------
00-0F  10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
10-1F  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
*/
const LENGTH_VALUES: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

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
}

const PULSE_MAX_VOLUME: f32 = 0.05;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PulseType {
    Pulse1,
    Pulse2,
}

struct PulseChannel {
    queue: AudioQueue<f32>,
    envelope: PulseEnvelope,
    sweep: Sweep,
    pulse_type: PulseType,

    low_timer: u8,
    timer: u16,
    current_timer: u16,
    length: u8,

    current_duty: u8,

    buffer: [f32; 2048],
    buffer_index: usize,

    envelope_clock: u8,
    restart_envelope: bool,
    sweep_clock: u8,
    current_volume: u8,
}

const DUTIES: [u8; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];

impl PulseChannel {
    fn new(queue: AudioQueue<f32>, pulse_type: PulseType) -> PulseChannel {
        PulseChannel {
            queue,
            envelope: PulseEnvelope::new(),
            sweep: Sweep::new(),
            timer: 0,
            low_timer: 0,
            length: 0,
            buffer: [0.0; 2048],
            buffer_index: 0,
            current_duty: 0,
            current_timer: 0,
            envelope_clock: 0,
            current_volume: 0,
            sweep_clock: 0,
            pulse_type,
            restart_envelope: false,
        }
    }

    fn set_envelope_flag(&mut self, flag: u8) {
        self.envelope = PulseEnvelope::from_flags(flag);
        self.restart_envelope = true;
    }

    fn set_sweep_flag(&mut self, flag: u8) {
        self.sweep = Sweep::from_flags(flag);
        self.sweep_clock = self.sweep.period;
    }

    fn set_low_timer(&mut self, timer: u8) {
        self.low_timer = timer;
    }

    fn set_length_counter_and_high_timer(&mut self, length_and_high: u8) {
        let length_index = length_and_high.bitand(0b11111000).shr(3);
        self.length = LENGTH_VALUES[length_index as usize];

        self.timer = self.low_timer as u16 | u16::from(length_and_high).bitand(0b111).shl(8);
        self.current_timer = self.timer;
        self.current_duty = 0;
        self.restart_envelope;
    }

    fn step(&mut self) {
        if self.current_timer > 0 {
            self.current_timer -= 1;
        } else {
            self.current_timer = self.timer;
            self.current_duty = (7 + self.current_duty) % 8;
        }
    }

    fn fill_buffer_and_start_queue(&mut self) {
        let volume = if self.envelope.constant_volume {
            PULSE_MAX_VOLUME * self.envelope.volume as f32 / 15.0
        } else {
            PULSE_MAX_VOLUME * self.current_volume as f32 / 15.0
        };

        self.buffer[self.buffer_index] = if self.timer < 8 || self.timer > 0x7ff || self.length == 0
        {
            0.0
        } else if DUTIES[self.envelope.duty as usize] & (1 << self.current_duty) != 0 {
            volume
        } else {
            -volume
        };

        self.buffer_index += 1;
        if self.buffer_index == self.buffer.len() {
            self.buffer_index = 0;
            self.queue.queue(&self.buffer);
        }
    }

    fn envelope_step(&mut self) {
        if self.restart_envelope {
            self.envelope_clock = self.envelope.volume;
            self.current_volume = 15;
            self.restart_envelope = false;
            return;
        }

        if self.envelope_clock > 0 {
            self.envelope_clock -= 1;
        } else {
            self.envelope_clock = self.envelope.volume;
            if self.current_volume > 0 {
                self.current_volume -= 1;
            }

            if self.envelope.loops_playback && self.current_volume == 0 {
                self.current_volume = 15;
            }
        }
    }

    fn sweep_step(&mut self) {
        if self.timer < 8 || self.timer > 0x7ff || !self.sweep.enabled || self.sweep.shift == 0 {
            return;
        }

        if self.sweep_clock > 0 {
            self.sweep_clock -= 1;
        } else {
            let add = self.timer >> self.sweep.shift;
            let extra = if self.pulse_type == PulseType::Pulse1 {
                1
            } else {
                0
            };
            let target_timer = if self.sweep.negate {
                self.timer - add - extra
            } else {
                self.timer + add
            };

            self.timer = target_timer;
            self.sweep_clock = self.sweep.period;
        }
    }

    fn length_step(&mut self) {
        if self.length > 0 && !self.envelope.loops_playback {
            self.length -= 1;
        }
    }
}

struct TriangleChannel {
    queue: AudioQueue<f32>,
    increment: i16,
    timer: u16,
    current_timer: u16,

    volume: u8,
    low_timer: u8,

    buffer: [f32; 2048],
    buffer_index: usize,

    length: u8,
    linear_counter: u8,
}

impl TriangleChannel {
    fn new(queue: AudioQueue<f32>) -> TriangleChannel {
        TriangleChannel {
            queue,
            increment: 0,
            timer: 0,
            current_timer: 0,
            low_timer: 0,
            volume: 0,
            buffer: [0.0; 2048],
            buffer_index: 0,
            length: 0,
            linear_counter: 0,
        }
    }

    fn set_low_timer(&mut self, value: u8) {
        self.low_timer = value;
    }

    fn set_length_counter_and_high_timer(&mut self, value: u8) {
        let timer = self.low_timer as u16 | u16::from(value).bitand(0b111).shl(8);
        self.timer = timer;
        self.current_timer = timer;
        self.increment = -1;
        self.volume = 15;

        let length_index = value.bitand(0b11111000).shr(3);
        self.length = LENGTH_VALUES[length_index as usize];
    }

    fn set_linear_counter_flag(&mut self, value: u8) {
        self.linear_counter = value.bitand(0b01111111);
    }

    fn step(&mut self) {
        if self.timer == 0 {
            return;
        }

        if self.current_timer > 0 {
            self.current_timer -= 1;
        } else {
            self.current_timer = self.timer;
            if self.volume > 0 && self.increment < 0 {
                self.volume -= 1;
            } else if self.volume < 15 && self.increment > 0 {
                self.volume += 1;
            }

            if self.volume == 0 {
                if self.increment < 0 {
                    self.increment = 0;
                } else {
                    self.increment = 1;
                }
            } else if self.volume == 15 {
                if self.increment > 0 {
                    self.increment = 0;
                } else {
                    self.increment = -1;
                }
            }
        }
    }

    fn fill_buffer_and_start_queue(&mut self) {
        self.buffer[self.buffer_index] = if self.length == 0 || self.linear_counter == 0 {
            0.0
        } else {
            0.32 * self.volume as f32 / 15.0 - 0.16
        };

        // dbg!(self.buffer_index, self.buffer[self.buffer_index]);
        self.buffer_index += 1;
        if self.buffer_index == self.buffer.len() {
            self.buffer_index = 0;
            self.queue.queue(&self.buffer);
        }
    }

    fn length_step(&mut self) {
        if self.length > 0 {
            self.length -= 1;
        }

        if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
    }
}

pub struct Apu {
    half_cycle_count: usize,
    pulse1_channel: PulseChannel,
    pulse2_channel: PulseChannel,
    triangle_channel: TriangleChannel,
}

impl Apu {
    pub fn new(audio_subsystem: AudioSubsystem) -> Apu {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),   // mono
            samples: Some(2048), // default sample size
        };

        let pulse1_queue: AudioQueue<f32> =
            audio_subsystem.open_queue(None, &desired_spec).unwrap();

        let pulse2_queue: AudioQueue<f32> =
            audio_subsystem.open_queue(None, &desired_spec).unwrap();

        let triangle_queue: AudioQueue<f32> =
            audio_subsystem.open_queue(None, &desired_spec).unwrap();

        // pulse1_queue.resume();
        // pulse2_queue.resume();
        triangle_queue.resume();

        Apu {
            half_cycle_count: 0,
            pulse1_channel: PulseChannel::new(pulse1_queue, PulseType::Pulse1),
            pulse2_channel: PulseChannel::new(pulse2_queue, PulseType::Pulse2),
            triangle_channel: TriangleChannel::new(triangle_queue),
        }
    }

    pub fn half_step(&mut self) {
        self.triangle_channel.step();
        if self.half_cycle_count % 14913 == 0 {
            self.pulse1_channel.sweep_step();
            self.pulse1_channel.length_step();

            self.pulse2_channel.sweep_step();
            self.pulse2_channel.length_step();

            self.triangle_channel.length_step();
        }

        if self.half_cycle_count % 7547 == 0 {
            self.pulse1_channel.envelope_step();
            self.pulse2_channel.envelope_step();
        }

        if self.half_cycle_count % 2 == 0 {
            self.pulse1_channel.step();
            self.pulse2_channel.step();
        }

        if self.half_cycle_count % 40 == 0 {
            self.pulse1_channel.fill_buffer_and_start_queue();
            self.pulse2_channel.fill_buffer_and_start_queue();
            self.triangle_channel.fill_buffer_and_start_queue();
        }

        self.half_cycle_count += 1;
    }

    pub fn write_pulse1_length_and_timer(&mut self, value: u8) {
        log_apu!(
            "Write $4003, timer: {:#04X}, length: {:#04X}",
            value & 0b111,
            (value >> 3) & 0b11111
        );

        self.pulse1_channel.set_length_counter_and_high_timer(value);
    }

    pub fn write_pulse1_sweep(&mut self, value: u8) {
        log_apu!("Write $4001: {:#04X}", value);

        self.pulse1_channel.set_sweep_flag(value);
    }

    pub fn write_pulse1_timer_low(&mut self, value: u8) {
        log_apu!("Write $4002: {:#04X}", value);
        self.pulse1_channel.set_low_timer(value);
    }

    pub fn write_pulse1_envelope(&mut self, value: u8) {
        log_apu!("Write $4000: {:#010b}", value);

        self.pulse1_channel.set_envelope_flag(value);
    }

    pub fn write_pulse2_length_and_timer(&mut self, value: u8) {
        log_apu!(
            "Write $4007, timer: {:#04X}, length: {:#04X}",
            value & 0b111,
            (value >> 3) & 0b11111
        );

        self.pulse2_channel.set_length_counter_and_high_timer(value);
    }

    pub fn write_pulse2_sweep(&mut self, value: u8) {
        log_apu!("write $4005: {:#04X}", value);
        self.pulse2_channel.set_sweep_flag(value);
    }

    pub fn write_pulse2_timer_low(&mut self, value: u8) {
        log_apu!("Write $4006: {:#04X}", value);
        self.pulse2_channel.set_low_timer(value);
    }

    pub fn write_pulse2_envelope(&mut self, value: u8) {
        log_apu!("Write $4004: {:#010b}", value);
        self.pulse2_channel.set_envelope_flag(value);
    }

    pub fn write_triangle_timer_low(&mut self, value: u8) {
        log_apu!("Write $400A: {:#04X}", value);
        self.triangle_channel.set_low_timer(value);
    }

    pub fn write_triangle_length_and_timer(&mut self, value: u8) {
        log_apu!("write $400B: {:#04X}", value);
        self.triangle_channel
            .set_length_counter_and_high_timer(value);
    }

    pub fn write_triangle_linear_counter(&mut self, value: u8) {
        log_apu!("Write $4008: {:#04X}", value);
        self.triangle_channel.set_linear_counter_flag(value);
    }
}
