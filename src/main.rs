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

    Ok(())
}
