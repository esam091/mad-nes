use std::io;

use termion::raw::IntoRawMode;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};

type MemoryBuffer = [u8; 0x10000];

struct Machine {
    memory: MemoryBuffer,
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        let bytes: Vec<u8> = std::fs::read(file_path)?.into_iter().skip(16).collect();

        let mut memory = [0 as u8; 0x10000];

        for i in 0..bytes.len() {
            memory[0x8000 + i] = bytes[i];
        }

        return Ok(Machine { memory: memory });
    }
}

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
    for address in (0x8000..=0xffff).step_by(16) {
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

fn main() -> Result<(), String> {
    let machine = Machine::load(&String::from("hello.nes")).unwrap();

    let stdout = io::stdout()
        .into_raw_mode()
        .map_err(|_| "Failed retrieving stdout")?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|_| "Failed creating terminal")?;

    terminal.clear().unwrap();
    terminal
        .draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            f.render_widget(address_widget(&machine.memory), chunks[0]);
        })
        .map_err(|_| "Failed drawing terminal")?;

    Ok(())
}
