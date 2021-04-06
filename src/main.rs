use std::{env, io};

mod cpu;
mod ines;
mod instruction;
mod machine;
mod ppu;
mod render;

use cpu::MemoryBuffer;
use machine::Machine;
use ppu::VideoMemoryBuffer;
use render::Renderer;

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

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

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

        if let Some(side_effect) = side_effect {
            renderer.render(&machine.get_ppu());

            let start_time = std::time::SystemTime::now();

            let duration = std::time::SystemTime::now()
                .duration_since(start_time)
                .unwrap();

            // println!("Render duration: {:?}", duration);

            // std::thread::sleep(Duration::from_millis(3));
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
