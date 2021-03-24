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
    for address in (0x3f00..=0x3f10).step_by(16) {
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
const PALETTE: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80),
    (0x00, 0x3D, 0xA6),
    (0x00, 0x12, 0xB0),
    (0x44, 0x00, 0x96),
    (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00),
    (0x8C, 0x17, 0x00),
    (0x5C, 0x2F, 0x00),
    (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00),
    (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66),
    (0x00, 0x00, 0x00),
    (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05),
    (0xC7, 0xC7, 0xC7),
    (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF),
    (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5),
    (0xFF, 0x29, 0x50),
    (0xFF, 0x22, 0x00),
    (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00),
    (0x05, 0x8F, 0x00),
    (0x00, 0x8A, 0x55),
    (0x00, 0x99, 0xCC),
    (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09),
    (0x09, 0x09, 0x09),
    (0xFF, 0xFF, 0xFF),
    (0x0F, 0xD7, 0xFF),
    (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3),
    (0xFF, 0x61, 0x8B),
    (0xFF, 0x88, 0x33),
    (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20),
    (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35),
    (0x0C, 0xF0, 0xA4),
    (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E),
    (0x0D, 0x0D, 0x0D),
    (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF),
    (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF),
    (0xDA, 0xAB, 0xEB),
    (0xFF, 0xA8, 0xF9),
    (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6),
    (0xFF, 0xF7, 0x9C),
    (0xD7, 0xE8, 0x95),
    (0xA6, 0xED, 0xAF),
    (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC),
    (0xDD, 0xDD, 0xDD),
    (0x11, 0x11, 0x11),
    (0x11, 0x11, 0x11),
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
                let top_right = attribute_value.bitand(0b1100 as u8) >> 2;
                let bottom_left = attribute_value.bitand(0b110000 as u8) >> 4;
                let bottom_right = attribute_value.bitand(0b11000000 as u8) >> 6;

                let subtile_y = row % 4;
                let subtile_x = col % 4;

                let palette_set = match (subtile_x / 2, subtile_y / 2) {
                    (0, 0) => top_left,
                    (1, 0) => top_right,
                    (1, 1) => bottom_left,
                    (0, 1) => bottom_right,
                    _ => panic!("Impossible subtile location!"),
                };

                for pattern_row in 0..8 {
                    let addr = pattern_table_address + pattern_row;
                    let bits = machine.get_video_buffer()[addr];
                    let bits2 = machine.get_video_buffer()[addr + 8];

                    for pattern_col in 0..8 {
                        let palette_value = palette_number(bits, bits2, pattern_col);
                        let palette_index = if palette_value == 0 {
                            0
                        } else {
                            palette_set as u32 * 4 + palette_value
                        };

                        let color_index =
                            machine.get_video_buffer()[0x3f00 + palette_index as usize];

                        if color_index as usize > PALETTE.len() {
                            panic!(
                                "color index {:#02x?}, requested at address {:#02x?}",
                                color_index,
                                0x3f00 + palette_index
                            );
                        }
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
