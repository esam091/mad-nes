use std::{collections::HashSet, convert::TryInto, env, io, time::Duration};

mod cpu;
mod ines;
mod instruction;
mod machine;
mod ppu;
mod render;

use cpu::MemoryBuffer;
use machine::{JoypadButton, Machine};
use ppu::VideoMemoryBuffer;
use render::Renderer;

use sdl2::keyboard::Keycode;
use termion::raw::IntoRawMode;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Row, Table},
    Terminal,
};

static TABLE_HEADER_CONSTRAINTS: [Constraint; 17] = [
    Constraint::Length(7),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
    Constraint::Length(2),
];

static TABLE_HEADERS: [&'static str; 17] = [
    "Address", "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0A", "0B", "0C", "0D",
    "0E", "0F",
];

fn address_widget(buffer: &MemoryBuffer) -> Table {
    let mut rows = Vec::<Row>::new();
    for address in (0x0000..=0x0200).step_by(16) {
        let mut content = vec![format!("{:#04X?}", address)];

        for offset in 0..=0xf {
            content.push(format!("{:02X?}", buffer[address + offset]));
        }

        rows.push(Row::new(content));
    }

    let table = Table::new(rows)
        .header(
            Row::new(Vec::from(TABLE_HEADERS))
                .style(Style::default().fg(tui::style::Color::Yellow)),
        )
        .block(
            Block::default()
                .title("Addresses")
                .borders(Borders::ALL)
                .border_type(tui::widgets::BorderType::Double),
        )
        .widths(&TABLE_HEADER_CONSTRAINTS);

    table
}

fn video_ram_widget(buffer: &VideoMemoryBuffer) -> Table {
    let mut rows = Vec::<Row>::new();
    for address in (0x3f00..=0x3f10).step_by(16) {
        let mut content = vec![format!("{:#04X?}", address)];

        for offset in 0..=0xf {
            content.push(format!("{:02X?}", buffer[address + offset]));
        }

        rows.push(Row::new(content));
    }

    let table = Table::new(rows)
        .header(
            Row::new(Vec::from(TABLE_HEADERS))
                .style(Style::default().fg(tui::style::Color::Yellow)),
        )
        .block(
            Block::default()
                .title("Addresses")
                .borders(Borders::ALL)
                .border_type(tui::widgets::BorderType::Double),
        )
        .widths(&TABLE_HEADER_CONSTRAINTS);

    table
}

const SCALE: u32 = 2;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut machine = Machine::load(&args[1]).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("NES Emulator", 256 * SCALE + 300, 240 * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let canvas = window
        .into_canvas()
        .accelerated()
        .target_texture()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();

    let mut renderer = Renderer::new(canvas, &texture_creator);
    // let stdout = io::stdout()
    //     .into_raw_mode()
    //     .map_err(|_| "Failed retrieving stdout")?;
    // let backend = TermionBackend::new(stdout);
    // let mut terminal = Terminal::new(backend).map_err(|_| "Failed creating terminal")?;

    // terminal.clear().unwrap();

    let mut frame_counter = 0;
    let mut cpu_steps = 0;

    let mut start_time = std::time::SystemTime::now();

    'running: loop {
        let aa = std::time::SystemTime::now();

        // terminal
        //     .draw(|f| {
        //         let chunks = Layout::default()
        //             .direction(Direction::Vertical)
        //             .margin(1)
        //             .constraints([Constraint::Percentage(100)].as_ref())
        //             .split(f.size());

        //         f.render_widget(
        //             address_widget(machine.get_cpu().get_memory_buffer()),
        //             chunks[0],
        //         );
        //         // f.render_widget(video_ram_widget(machine.get_video_buffer()), chunks[0]);
        //     })
        //     .map_err(|_| "Failed drawing terminal")?;

        let side_effect = machine.step();
        cpu_steps += 1;

        if let Some(side_effect) = side_effect {
            for event in event_pump.poll_iter() {
                let mut active_buttons = HashSet::<JoypadButton>::new();
                match event {
                    sdl2::event::Event::Quit { .. } => break 'running,
                    sdl2::event::Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        match key {
                            Keycode::A => {
                                active_buttons.insert(JoypadButton::A);
                            }
                            Keycode::B => {
                                active_buttons.insert(JoypadButton::B);
                            }
                            Keycode::RShift => {
                                active_buttons.insert(JoypadButton::Select);
                            }
                            Keycode::Return => {
                                active_buttons.insert(JoypadButton::Start);
                            }
                            Keycode::Up => {
                                active_buttons.insert(JoypadButton::Up);
                            }
                            Keycode::Down => {
                                active_buttons.insert(JoypadButton::Down);
                            }
                            Keycode::Left => {
                                active_buttons.insert(JoypadButton::Left);
                            }
                            Keycode::Right => {
                                active_buttons.insert(JoypadButton::Right);
                            }
                            _ => {}
                        };

                        machine.set_active_buttons(active_buttons);
                    }
                    _ => {}
                }
            }

            renderer.render(&machine.get_ppu());

            frame_counter += 1;

            let last_render_time = std::time::SystemTime::now();
            let render_duration = last_render_time.duration_since(start_time).unwrap();

            if render_duration.as_micros() < 16667 {
                let sleep_duration = 16667u128 - render_duration.as_micros();

                std::thread::sleep(Duration::from_micros(sleep_duration.try_into().unwrap()));
            }

            start_time = std::time::SystemTime::now();
        }

        let now = std::time::SystemTime::now();
        let duration = now.duration_since(start_time).unwrap();

        if duration > Duration::from_secs(1) {
            dbg!(frame_counter);
            dbg!(cpu_steps);
            frame_counter = 0;
            cpu_steps = 0;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn endian_conversion() {
        assert_eq!(u16::from_le_bytes([0xbb, 0xaa]), 0xaabb);
        assert_eq!(u16::from_le(0xaabb), 0xaabb);
        assert_eq!(u16::from_be_bytes([0xaa, 0xbb]), 0xaabb);
        assert_eq!(u16::from_be(0xbbaa), 0xaabb);
        assert_eq!(0b1111, 15);

        assert_eq!(255u8.overflowing_add(3), (2, true));
    }
}
