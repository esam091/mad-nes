use std::{io, usize};

mod instruction;

use instruction::Instruction;
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
    pc: u16,
    a: u8,
}

impl Machine {
    pub fn load(file_path: &String) -> Result<Machine, std::io::Error> {
        // since we're using a legit hello world ROM as a prototype, we don't need to configure stuff from headers
        let bytes: Vec<u8> = std::fs::read(file_path)?.into_iter().skip(16).collect();

        let mut memory = [0 as u8; 0x10000];

        for i in 0..bytes.len() - 16 {
            memory[0x8000 + i] = bytes[i];
        }

        // Copy the reset vector data
        // The data was located at 400c but we subtract by 0x10 since we skipped 16 bytes in the header
        memory[0xfffc] = bytes[0x3ffc];
        memory[0xfffd] = bytes[0x3ffd];

        // jump to reset vector
        let initial_address = u16::from_le_bytes([memory[0xfffc], memory[0xfffd]]);

        return Ok(Machine {
            memory: memory,
            pc: initial_address,
            a: 0,
        });
    }

    fn get_byte_and_forward_pc(&mut self) -> u8 {
        let value = self.memory[self.pc as usize];
        self.pc += 1;

        return value;
    }

    fn get_word_and_forward_pc(&mut self) -> u16 {
        let byte1 = self.get_byte_and_forward_pc();
        let byte2 = self.get_byte_and_forward_pc();

        return u16::from_le_bytes([byte1, byte2]);
    }

    pub fn step(&mut self) {
        let opcode = self.memory[self.pc as usize];
        self.pc += 1;

        let instruction: Option<Instruction>;
        match opcode {
            0xa9 => {
                instruction = Some(Instruction::LdaImmediate(self.get_byte_and_forward_pc()));
            }
            0x8d => {
                instruction = Some(Instruction::StaAbsolute(self.get_word_and_forward_pc()));
            }
            _ => {
                instruction = None;
            }
        }

        match instruction.expect("Instruction not found, opcode might not have been implemented") {
            Instruction::LdaImmediate(value) => {
                self.a = value;
            }
            Instruction::StaAbsolute(value) => {
                self.memory[value as usize] = self.a;
            }
        }
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

#[cfg(test)]
mod tests {

    #[test]
    fn endian_conversion() {
        assert_eq!(u16::from_le_bytes([0xbb, 0xaa]), 0xaabb);
        assert_eq!(u16::from_le(0xaabb), 0xaabb);
        assert_eq!(u16::from_be_bytes([0xaa, 0xbb]), 0xaabb);
        assert_eq!(u16::from_be(0xbbaa), 0xaabb);
        assert_eq!(0b1111, 15);
    }
}
