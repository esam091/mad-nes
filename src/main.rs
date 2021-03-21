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

struct Machine {
    memory: [u8; 0xffff],
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        let bytes = std::fs::read(file_path)?;

        let mut memory: [u8; 0xffff] = [0; 0xffff];

        for i in 0..bytes.len() {
            memory[0x8000 + i] = bytes[i];
        }

        return Ok(Machine { memory: memory });
    }
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

            let table = Table::new(vec![Row::new(vec!["0xAABB", "00", "66", "FF"])])
                .header(
                    Row::new(vec!["Address", "00", "01", "02"])
                        .style(Style::default().fg(Color::Yellow)),
                )
                .block(
                    Block::default()
                        .title("Addresses")
                        .borders(Borders::ALL)
                        .border_type(tui::widgets::BorderType::Double),
                )
                .widths(&[
                    Constraint::Length(7),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(2),
                ]);

            f.render_widget(table, chunks[0]);
        })
        .map_err(|_| "Failed drawing terminal")?;

    Ok(())
}
