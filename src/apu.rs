use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    ops::{BitAnd, Shl, Shr, ShrAssign},
    rc::Rc,
};

use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    AudioSubsystem,
};

use bitflags::bitflags;

use crate::{ines::Cartridge, log_apu};

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

struct Envelope {
    duty: u8,
    volume: u8,
    loops_playback: bool,
    constant_volume: bool,
}

impl Envelope {
    fn new() -> Envelope {
        Envelope {
            duty: 0,
            volume: 0,
            loops_playback: false,
            constant_volume: false,
        }
    }

    fn from_flags(flag: u8) -> Envelope {
        let duty = flag.bitand(0b11000000).shr(6);
        let loops_playback = flag.bitand(0b100000) != 0;
        let constant_volume = flag.bitand(0b10000) != 0;
        let volume = flag.bitand(0b1111);

        Envelope {
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
        let shift = flag & 0b111;
        let enabled = flag & 0x80 != 0 && shift != 0;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum PulseType {
    Pulse1,
    Pulse2,
}

struct PulseChannel {
    envelope: Envelope,
    sweep: Sweep,
    pulse_type: PulseType,

    timer: u16,
    current_timer: u16,
    length: u8,

    current_duty: u8,

    envelope_clock: u8,
    restart_envelope: bool,
    sweep_clock: u8,
    restart_sweep: bool,
    current_volume: u8,

    enabled: bool,
}

const DUTIES: [u8; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];

impl PulseChannel {
    fn new(pulse_type: PulseType) -> PulseChannel {
        PulseChannel {
            envelope: Envelope::new(),
            sweep: Sweep::new(),
            timer: 0,
            length: 0,
            current_duty: 0,
            current_timer: 0,
            envelope_clock: 0,
            current_volume: 0,
            sweep_clock: 0,
            pulse_type,
            restart_envelope: false,
            enabled: false,
            restart_sweep: true,
        }
    }

    fn is_running(&self) -> bool {
        self.length != 0
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length = 0;
        }
    }

    fn set_envelope_flag(&mut self, flag: u8) {
        self.envelope = Envelope::from_flags(flag);
    }

    fn set_sweep_flag(&mut self, flag: u8) {
        self.sweep = Sweep::from_flags(flag);
        self.restart_sweep = true;
    }

    fn set_low_timer(&mut self, timer: u8) {
        self.timer &= !0xff;
        self.timer |= timer as u16;
    }

    fn set_length_counter_and_high_timer(&mut self, length_and_high: u8) {
        if self.enabled {
            let length_index = length_and_high.bitand(0b11111000).shr(3);
            self.length = LENGTH_VALUES[length_index as usize];
        }

        self.timer &= 0xff;
        self.timer |= u16::from(length_and_high).bitand(0b111).shl(8);
        self.current_timer = self.timer;
        self.current_duty = 0;
        self.restart_envelope = true;
    }

    fn step(&mut self) {
        if self.current_timer > 0 {
            self.current_timer -= 1;
        } else {
            self.current_timer = self.timer;
            self.current_duty = (7 + self.current_duty) % 8;
        }
    }

    fn get_current_volume(&self) -> u8 {
        if self.timer < 8
            || (!self.sweep.negate && self.next_target_period() > 0x7ff)
            || self.length == 0
            || DUTIES[self.envelope.duty as usize] & (1 << self.current_duty) == 0
        {
            0
        } else if self.envelope.constant_volume {
            self.envelope.volume
        } else {
            self.current_volume
        }
    }

    fn quarter_frame_clock(&mut self) {
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

    fn half_frame_clock(&mut self) {
        self.sweep_step();
        self.length_step();
    }

    fn sweep_step(&mut self) {
        if self.restart_sweep {
            self.sweep_clock = self.sweep.period;
            self.restart_sweep = false;
        }

        let next_target_period = self.next_target_period();

        if self.sweep_clock > 0 {
            self.sweep_clock -= 1;
        } else if self.sweep.enabled
            && self.timer >= 8
            && !(!self.sweep.negate && next_target_period > 0x7ff)
        {
            self.timer = next_target_period;
            self.sweep_clock = self.sweep.period;
        }
    }

    fn next_target_period(&self) -> u16 {
        let add = self.timer >> self.sweep.shift;
        let extra = if self.pulse_type == PulseType::Pulse1 {
            1
        } else {
            0
        };

        if self.sweep.negate {
            self.timer - add - extra
        } else {
            self.timer + add
        }
    }

    fn length_step(&mut self) {
        if self.length > 0 && !self.envelope.loops_playback {
            self.length -= 1;
        }
    }

    fn reset(&mut self) {
        // self.restart_envelope = true;
    }
}

struct TriangleChannel {
    timer: u16,
    current_timer: u16,

    length: u8,
    current_linear_counter: u8,
    linear_counter: u8,
    linear_counter_reload: bool,

    control_flag: bool,
    enabled: bool,
    tri_step: u8,
}

impl TriangleChannel {
    fn new() -> TriangleChannel {
        TriangleChannel {
            tri_step: 0,
            timer: 0,
            current_timer: 0,
            length: 0,
            current_linear_counter: 0,
            linear_counter: 0,
            linear_counter_reload: false,
            control_flag: false,
            enabled: false,
        }
    }

    fn is_running(&self) -> bool {
        self.length != 0
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length = 0;
        }
    }

    fn set_low_timer(&mut self, value: u8) {
        self.timer &= 0xff00;
        self.timer |= value as u16;
    }

    fn set_length_counter_and_high_timer(&mut self, value: u8) {
        self.timer &= 0xff;
        self.timer |= u16::from(value).bitand(0b111).shl(8);
        self.current_timer = self.timer;
        self.linear_counter_reload = true;

        if self.enabled {
            let length_index = value.bitand(0b11111000).shr(3);
            self.length = LENGTH_VALUES[length_index as usize];
        }
    }

    fn set_linear_counter_flag(&mut self, value: u8) {
        self.linear_counter = value.bitand(0b01111111);
        self.control_flag = value.bitand(0x80) != 0;
    }

    fn step(&mut self) {
        let mut clock_triunit = true;
        if self.length == 0
            || self.current_linear_counter == 0
            || (self.timer < 2 && self.current_timer == 0)
        {
            clock_triunit = false;
        }

        if clock_triunit {
            if self.current_timer > 0 {
                self.current_timer -= 1;
            } else {
                self.current_timer = self.timer;
                self.tri_step = (self.tri_step + 1) & 0x1f;
            }
        }
    }

    fn get_current_volume(&self) -> u8 {
        if self.timer < 2 && self.current_timer == 0 {
            7
        } else if self.tri_step & 0x10 != 0 {
            self.tri_step ^ 0x1f
        } else {
            self.tri_step
        }
    }

    fn half_frame_clock(&mut self) {
        if self.length > 0 && !self.control_flag {
            self.length -= 1;
        }
    }

    fn quarter_frame_clock(&mut self) {
        if self.linear_counter_reload {
            self.current_linear_counter = self.linear_counter;
        } else if self.current_linear_counter > 0 {
            self.current_linear_counter -= 1;
        }

        if !self.control_flag {
            self.linear_counter_reload = false;
        }
    }
}

struct NoiseChannel {
    shift_register: u16,
    mode_flag: bool,
    noise_period: u16,
    current_noise_timer: u16,
    envelope: Envelope,
    envelope_clock: u8,
    current_volume: u8,

    length: u8,
    restart_envelope: bool,

    enabled: bool,
}

const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

impl NoiseChannel {
    fn new() -> NoiseChannel {
        NoiseChannel {
            shift_register: 1,
            current_volume: 0,
            mode_flag: false,
            noise_period: 2,
            envelope: Envelope::new(),
            envelope_clock: 0,
            current_noise_timer: 0,
            length: 0,
            restart_envelope: false,
            enabled: false,
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length = 0;
        }
    }

    fn is_running(&self) -> bool {
        self.length != 0
    }

    fn set_envelope_flag(&mut self, flag: u8) {
        self.envelope = Envelope::from_flags(flag);
    }

    fn set_mode_and_period(&mut self, flag: u8) {
        self.mode_flag = flag & 0x80 != 0;

        let noise_period_index = flag & 0b1111;
        self.noise_period = NOISE_PERIOD_TABLE[noise_period_index as usize] / 2;
        self.current_noise_timer = self.noise_period;
    }

    fn set_length_counter(&mut self, flag: u8) {
        if self.enabled {
            let length_index = flag.bitand(0b11111000).shr(3);
            self.length = LENGTH_VALUES[length_index as usize];
        }
        self.restart_envelope = true;
    }

    fn step(&mut self) {
        if self.current_noise_timer > 0 {
            self.current_noise_timer -= 1;
        } else {
            self.current_noise_timer = self.noise_period;
            //adjust shift
            let bit = (self.shift_register & 1)
                ^ if self.mode_flag {
                    self.shift_register.bitand(0b1000000).shr(6)
                } else {
                    self.shift_register.bitand(0b10).shr(1)
                };

            self.shift_register >>= 1;
            self.shift_register |= bit << 14;
        }
    }

    fn half_frame_clock(&mut self) {
        if self.length > 0 && !self.envelope.loops_playback {
            self.length -= 1;
        }
    }

    fn quarter_frame_clock(&mut self) {
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

    fn get_current_volume(&self) -> u8 {
        if self.shift_register & 1 == 0 && self.length != 0 {
            if self.envelope.constant_volume {
                self.envelope.volume
            } else {
                self.current_volume
            }
        } else {
            0
        }
    }
}

fn create_tnd_table() -> [f32; 203] {
    let mut table = [0.0; 203];

    for n in 0..table.len() {
        table[n] = 163.67 / (24329.0 / n as f32 + 100.0);
    }

    table
}

fn create_pulse_table() -> [f32; 31] {
    let mut table = [0.0; 31];

    for n in 0..table.len() {
        table[n] = 95.52 / (8128.0 / n as f32 + 100.0)
    }

    table
}

struct FrameCounter {
    cpu_cycles: usize,
    reset: bool,
    mode_flag: bool,
    irq_flag: bool,
}

const QUARTER_CYCLES: usize = 7457;

impl FrameCounter {
    fn new() -> FrameCounter {
        FrameCounter {
            cpu_cycles: 0,
            reset: false,
            mode_flag: false,
            irq_flag: false,
        }
    }

    fn step(&mut self) {
        self.cpu_cycles += 1;
        self.reset = false;
    }

    fn is_clocking_half_frame(&self) -> bool {
        if self.reset && self.mode_flag {
            return true;
        }

        if self.cpu_cycles % QUARTER_CYCLES == 0 {
            if self.mode_flag {
                let phase = self.get_quarter_blocks() % 5;

                return phase == 1 || phase == 4;
            } else {
                return self.get_quarter_blocks() % 2 != 0;
            }
        }

        false
    }

    fn get_quarter_blocks(&self) -> usize {
        self.cpu_cycles / QUARTER_CYCLES
    }

    fn is_clocking_quarter_frame(&self) -> bool {
        if self.reset && self.mode_flag {
            return true;
        }

        if self.cpu_cycles % QUARTER_CYCLES == 0 {
            if self.mode_flag {
                let phase = self.get_quarter_blocks() % 5;

                return phase != 3;
            } else {
                return true;
            }
        }

        false
    }

    fn set_flags(&mut self, value: u8) {
        self.reset = true;
        self.cpu_cycles = 0;
        self.mode_flag = value & 0x80 != 0;
        self.irq_flag = value & 0x40 != 0;
    }
}

const DMC_RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];
struct DmcChannel {
    sample_buffer: Option<u8>,
    rate: u16,
    current_timer: u16,

    silence: bool,
    bits_left: u8,
    shift_register: u8,

    current_output: u8,
    sample_address: u16,
    current_address: u16,
    sample_length: u16,
    current_length: u16,

    loops_playback: bool,
    irq_enabled: bool,

    cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl DmcChannel {
    fn new() -> DmcChannel {
        DmcChannel {
            sample_buffer: None,
            rate: DMC_RATE_TABLE[0] / 2,
            silence: true,
            bits_left: 0,
            current_timer: 0,
            shift_register: 0,
            current_output: 0,
            current_length: 0,
            sample_address: 0,
            sample_length: 0,
            loops_playback: false,
            irq_enabled: false,
            cartridge: None,
            current_address: 0,
        }
    }

    fn set_direct_load(&mut self, value: u8) {
        self.current_output = value & 127;
    }

    fn is_running(&self) -> bool {
        self.current_length != 0
    }

    fn step(&mut self) {
        if self.current_timer > 0 {
            self.current_timer -= 1;
        } else {
            self.current_timer = self.rate;

            if !self.silence {
                if self.shift_register & 1 != 0 && self.current_output < 126 {
                    self.current_output += 2;
                } else {
                    self.current_output = self.current_output.saturating_sub(2);
                }
            }

            self.bits_left -= 1;
            self.shift_register >>= 1;

            if self.bits_left == 0 {
                self.bits_left = 8;
                self.shift_register = self.sample_buffer.unwrap_or(0);
                self.silence = self.sample_buffer == None;
                self.sample_buffer = None;
            }

            if self.current_length > 0 && self.sample_buffer == None {
                match self.cartridge.as_mut() {
                    Some(cartridge) => {
                        let cartridge = cartridge.as_ref();
                        let value = cartridge.borrow_mut().read_address(self.current_address) & 127;
                        self.sample_buffer = Some(value);
                    }
                    _ => {
                        todo!()
                    }
                }

                self.current_address = self.current_address.wrapping_add(1) | 0x8000;
                self.current_length -= 1;

                if self.current_length == 0 {
                    if self.loops_playback {
                        self.current_length = self.sample_length;
                        self.current_address = self.sample_address;
                    }
                } // todo: handle IRQ
            }
        }
    }

    fn set_sample_address(&mut self, value: u8) {
        self.sample_address = 0xc000 | value as u16 * 0x40;
        // dbg!(self.sample_address);
    }

    fn set_sample_length(&mut self, value: u8) {
        self.sample_length = value as u16 * 0x10 + 1;
        // dbg!(self.sample_length);
    }

    fn set_settings(&mut self, value: u8) {
        let rate_index = value & 0b1111;
        self.rate = DMC_RATE_TABLE[rate_index as usize] / 2;
        self.loops_playback = value & 0b01000000 != 0;
        self.irq_enabled = value & 0b10000000 != 0;
    }

    fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            if self.current_length == 0 {
                self.current_length = self.sample_length;
                self.current_address = self.sample_address;
            }
        } else {
            self.current_length = 0;
        }
    }

    fn set_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(cartridge);
    }

    fn get_current_volume(&self) -> u8 {
        self.current_output
    }
}

pub struct Apu {
    half_cycle_count: usize,
    pulse1_channel: PulseChannel,
    pulse2_channel: PulseChannel,
    triangle_channel: TriangleChannel,
    noise_channel: NoiseChannel,
    dmc_channel: DmcChannel,
    tnd_table: [f32; 203],
    pulse_table: [f32; 31],
    output_queue: AudioQueue<f32>,

    buffer: [f32; 2048],
    buffer_index: usize,
    next_fill: usize,
    has_extra: bool,
    frame_counter: FrameCounter,
}

impl Apu {
    pub fn new(audio_subsystem: AudioSubsystem) -> Apu {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),   // mono
            samples: Some(2048), // default sample size
        };

        let output_queue =
            audio_subsystem.open_queue(None, &desired_spec).unwrap() as AudioQueue<f32>;
        output_queue.resume();

        Apu {
            half_cycle_count: 0,
            pulse1_channel: PulseChannel::new(PulseType::Pulse1),
            pulse2_channel: PulseChannel::new(PulseType::Pulse2),
            triangle_channel: TriangleChannel::new(),
            noise_channel: NoiseChannel::new(),
            tnd_table: create_tnd_table(),
            pulse_table: create_pulse_table(),
            output_queue,
            buffer: [0.0; 2048],
            buffer_index: 0,
            next_fill: 40,
            has_extra: true,
            frame_counter: FrameCounter::new(),
            dmc_channel: DmcChannel::new(),
        }
    }

    pub fn half_step(&mut self) {
        self.triangle_channel.step();

        if self.frame_counter.is_clocking_half_frame() {
            self.pulse1_channel.half_frame_clock();
            self.pulse2_channel.half_frame_clock();
            self.triangle_channel.half_frame_clock();
            self.noise_channel.half_frame_clock();
        }

        if self.frame_counter.is_clocking_quarter_frame() {
            self.pulse1_channel.quarter_frame_clock();
            self.pulse2_channel.quarter_frame_clock();
            self.triangle_channel.quarter_frame_clock();
            self.noise_channel.quarter_frame_clock();
        }

        if self.half_cycle_count % 2 == 0 {
            self.pulse1_channel.step();
            self.pulse2_channel.step();
            self.noise_channel.step();
            self.dmc_channel.step();
        }

        if self.half_cycle_count % self.next_fill == 0 {
            self.next_fill += 40 + self.has_extra as usize;
            self.has_extra = !self.has_extra;
            let pulse1 = self.pulse1_channel.get_current_volume() as usize;
            // let pulse1 = 0;

            let pulse2 = self.pulse2_channel.get_current_volume() as usize;
            // let pulse2 = 0;

            let triangle = self.triangle_channel.get_current_volume() as usize;
            // let triangle = 0;

            let noise = self.noise_channel.get_current_volume() as usize;
            // let noise = 0;

            let dmc = self.dmc_channel.get_current_volume() as usize;
            // let dmc = 0;

            let tnd_out = self.tnd_table[3 * triangle + 2 * noise + dmc];

            let pulse_out = self.pulse_table[pulse1 + pulse2];

            let output = tnd_out + pulse_out;

            self.buffer[self.buffer_index] = output;

            self.buffer_index += 1;
            if self.buffer_index == 2048 {
                self.buffer_index = 0;
                self.output_queue.queue(&self.buffer);
            }
        }

        self.half_cycle_count += 1;
        self.frame_counter.step();
    }

    pub fn write_noise_envelope(&mut self, value: u8) {
        log_apu!("Write $400c: {:#010b}", value);
        self.noise_channel.set_envelope_flag(value);
    }

    pub fn write_noise_mode_and_period(&mut self, value: u8) {
        log_apu!("Write $400e: {:#04X}", value);
        self.noise_channel.set_mode_and_period(value);
    }

    pub fn write_noise_length_counter(&mut self, value: u8) {
        log_apu!("Write $400f: {:#04X}", value);
        self.noise_channel.set_length_counter(value);
    }

    pub fn write_pulse1_length_and_timer(&mut self, value: u8) {
        log_apu!("Write $4003: {:#04X}", value);

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
        log_apu!("Write $4007: {:#04X}", value);

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

    pub fn write_status(&mut self, value: u8) {
        log_apu!("Write $4015: {:#010b}", value);

        let status = ApuStatus::from_bits(value).unwrap();
        self.pulse1_channel
            .set_enabled(status.contains(ApuStatus::PULSE_1));
        self.pulse2_channel
            .set_enabled(status.contains(ApuStatus::PULSE_2));
        self.triangle_channel
            .set_enabled(status.contains(ApuStatus::TRIANGLE));
        self.noise_channel
            .set_enabled(status.contains(ApuStatus::NOISE));
        self.dmc_channel
            .set_enabled(status.contains(ApuStatus::DMC));
        // TODO: handleframe interrupt
    }

    pub fn read_status(&self) -> u8 {
        let mut status = ApuStatus::empty();

        status.set(ApuStatus::PULSE_1, self.pulse1_channel.is_running());
        status.set(ApuStatus::PULSE_2, self.pulse2_channel.is_running());
        status.set(ApuStatus::TRIANGLE, self.triangle_channel.is_running());
        status.set(ApuStatus::NOISE, self.noise_channel.is_running());
        status.set(ApuStatus::DMC, self.dmc_channel.is_running());

        let bits = status.bits();
        log_apu!("Read $4015: {:#010b}", bits);

        bits
    }

    pub fn write_frame_counter(&mut self, value: u8) {
        log_apu!("Write $4017: {:#010b}", value);

        self.frame_counter.set_flags(value);
        self.pulse1_channel.reset();
        self.pulse2_channel.reset();
    }

    pub fn write_dmc_settings(&mut self, value: u8) {
        log_apu!("Write $4010: {:#010b}", value);
        self.dmc_channel.set_settings(value);
    }

    pub fn write_dmc_direct_load(&mut self, value: u8) {
        log_apu!("Write $4011: {:#04X}", value);
        self.dmc_channel.set_direct_load(value);
    }

    pub fn write_dmc_sample_address(&mut self, value: u8) {
        log_apu!("Write $4012: {:#04X}", value);
        self.dmc_channel.set_sample_address(value);
    }

    pub fn write_dmc_sample_length(&mut self, value: u8) {
        log_apu!("Write $4013: {:#04X}", value);
        self.dmc_channel.set_sample_length(value);
    }

    pub fn set_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
        self.dmc_channel.set_cartridge(cartridge);
    }
}

bitflags! {
    struct ApuStatus: u8 {
        const DMC_INTERRUPT = 0b10000000;
        const FRAME_INTERRUPT = 0b01000000;
        const DMC = 0b00010000;
        const NOISE = 0b00001000;
        const TRIANGLE = 0b00000100;
        const PULSE_2 = 0b00000010;
        const PULSE_1 = 0b00000001;
        const UNUSED = 0b00100000;
    }
}
