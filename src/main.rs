use std::{convert::TryInto, io, ops::BitAnd, time::Duration};

mod instruction;
mod machine;

use machine::{Machine, MemoryBuffer, VideoMemoryBuffer};
use termion::raw::IntoRawMode;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
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
    for address in (0x2000..=0x2100).step_by(16) {
        let mut content = vec![format!("{:#04X?}", address)];

        for offset in 0..=0xf {
            content.push(format!("{:02X?}", buffer[address + offset]));
        }

        rows.push(Row::new(content));
    }

    let table = Table::new(rows)
        .header(Row::new(Vec::from(TABLE_HEADERS)).style(Style::default().fg(Color::Yellow)))
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
    for address in (0x2200..=0x2400).step_by(16) {
        let mut content = vec![format!("{:#04X?}", address)];

        for offset in 0..=0xf {
            content.push(format!("{:02X?}", buffer[address + offset]));
        }

        rows.push(Row::new(content));
    }

    let table = Table::new(rows)
        .header(Row::new(Vec::from(TABLE_HEADERS)).style(Style::default().fg(Color::Yellow)))
        .block(
            Block::default()
                .title("Addresses")
                .borders(Borders::ALL)
                .border_type(tui::widgets::BorderType::Double),
        )
        .widths(&TABLE_HEADER_CONSTRAINTS);

    table
}

const SCALE: u32 = 3;

fn palette_number(left: u8, right: u8, index: usize) -> u32 {
    let is_left_on = left.bitand(1 << (7 - index)) != 0;
    let is_right_on = right.bitand(1 << (7 - index)) != 0;

    match (is_left_on, is_right_on) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    }
}

fn main() -> Result<(), String> {
    let mut machine = Machine::load(&String::from("hello.nes")).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("NES Emulator", 256 * SCALE, 240 * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let stdout = io::stdout()
        .into_raw_mode()
        .map_err(|_| "Failed retrieving stdout")?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|_| "Failed creating terminal")?;

    terminal.clear().unwrap();

    'running: loop {
        machine.step();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        for row in 0..30 {
            for col in 0..32 {
                let nametable_address = row * 32 + col + 0x2000;

                let nametable_value = machine.get_video_buffer()[nametable_address];

                let pattern_table_address = nametable_value as usize * 0x10;

                for pattern_row in 0..8 {
                    let addr = pattern_table_address + pattern_row;
                    let bits = machine.get_video_buffer()[addr];
                    let bits2 = machine.get_video_buffer()[addr + 8];

                    for pattern_col in 0..8 {
                        let palette_value = palette_number(bits, bits2, pattern_col);

                        let color = match palette_value {
                            0 => sdl2::pixels::Color::RGB(0, 0, 0),
                            1 => sdl2::pixels::Color::RGB(0xff, 00, 00),
                            2 => sdl2::pixels::Color::RGB(0, 0xff, 0),
                            3 => sdl2::pixels::Color::RGB(0, 0, 0xff),
                            _ => panic!("Impossible color palette: {}", palette_value),
                        };

                        let y = pattern_row + row * 8;
                        let x = pattern_col + col * 8;

                        let xx: i32 = x.try_into().unwrap();
                        let yy: i32 = y.try_into().unwrap();

                        canvas.set_draw_color(color);
                        canvas
                            .fill_rect(sdl2::rect::Rect::new(
                                xx * SCALE as i32,
                                yy * SCALE as i32,
                                SCALE,
                                SCALE,
                            ))
                            .unwrap();
                    }
                }
            }
        }

        canvas.present();

        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());

                // f.render_widget(address_widget(machine.get_buffer()), chunks[0]);
                f.render_widget(video_ram_widget(machine.get_video_buffer()), chunks[0]);
            })
            .map_err(|_| "Failed drawing terminal")?;

        // std::thread::sleep(Duration::from_millis(3));
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
