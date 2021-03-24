use std::{convert::TryInto, env, io, ops::BitAnd, time::Duration};

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
const PALETTE: [(u8, u8, u8); 56] = [
    (84, 84, 84),
    (0, 30, 116),
    (8, 16, 144),
    (48, 0, 136),
    (68, 0, 100),
    (92, 0, 48),
    (84, 4, 0),
    (60, 24, 0),
    (32, 42, 0),
    (8, 58, 0),
    (0, 64, 0),
    (0, 60, 0),
    (0, 50, 60),
    (0, 0, 0),
    (152, 150, 152),
    (8, 76, 196),
    (48, 50, 236),
    (92, 30, 228),
    (136, 20, 176),
    (160, 20, 100),
    (152, 34, 32),
    (120, 60, 0),
    (84, 90, 0),
    (40, 114, 0),
    (8, 124, 0),
    (0, 118, 40),
    (0, 102, 120),
    (0, 0, 0),
    (236, 238, 236),
    (76, 154, 236),
    (120, 124, 236),
    (176, 98, 236),
    (228, 84, 236),
    (236, 88, 180),
    (236, 106, 100),
    (212, 136, 32),
    (160, 170, 0),
    (116, 196, 0),
    (76, 208, 32),
    (56, 204, 108),
    (56, 180, 204),
    (60, 60, 60),
    (236, 238, 236),
    (168, 204, 236),
    (188, 188, 236),
    (212, 178, 236),
    (236, 174, 236),
    (236, 174, 212),
    (236, 180, 176),
    (228, 196, 144),
    (204, 210, 120),
    (180, 222, 120),
    (168, 226, 144),
    (152, 226, 180),
    (160, 214, 228),
    (160, 162, 160),
];

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
    let args: Vec<String> = env::args().collect();
    let mut machine = Machine::load(&args[1]).unwrap();

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

                let attribute_y = row / 4;
                let attribute_x = col / 4;

                let attribute_value =
                    machine.get_video_buffer()[0x23c0 + attribute_x + attribute_y * 8];

                let top_left = attribute_value & 0b11;
                let top_right = attribute_value.bitand(0b1100) >> 2;
                let bottom_left = attribute_value.bitand(0b110000) >> 4;
                let bottom_right = attribute_value.bitand(0b11000000) >> 6;

                let subtile_y = row % 4;
                let subtile_x = col % 4;

                let palette_set = match (subtile_x / 2, subtile_y / 2) {
                    (0, 0) => top_left,
                    (0, 1) => top_right,
                    (1, 0) => bottom_left,
                    (1, 1) => bottom_right,
                    _ => panic!("Impossible subtile location!"),
                };

                for pattern_row in 0..8 {
                    let addr = pattern_table_address + pattern_row;
                    let bits = machine.get_video_buffer()[addr];
                    let bits2 = machine.get_video_buffer()[addr + 8];

                    for pattern_col in 0..8 {
                        let palette_value = palette_number(bits, bits2, pattern_col);
                        let palette_index = palette_set as u32 * 4 + palette_value;

                        let color_index =
                            machine.get_video_buffer()[0x3f00 + palette_index as usize];

                        let (r, g, b) = PALETTE[color_index as usize];
                        let color = sdl2::pixels::Color::RGB(r, g, b);
                        // let color = match palette_value {
                        //     0 => sdl2::pixels::Color::RGB(0, 0, 0),
                        //     1 => sdl2::pixels::Color::RGB(0xff, 00, 00),
                        //     2 => sdl2::pixels::Color::RGB(0, 0xff, 0),
                        //     3 => sdl2::pixels::Color::RGB(0, 0, 0xff),
                        //     _ => panic!("Impossible color palette: {}", palette_value),
                        // };

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
