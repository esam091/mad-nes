#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    LdaImmediate(u8),
    StaAbsolute(u16),
    LdxImmediate(u8),
    LdaXAbsolute(u16),
    CmpImmediate(u8),
    Beq(u8),
    Inx,
    JmpAbsolute(u16),
    CpxImmediate(u8),
    Bne(u8),
}

fn next_byte<I: Iterator<Item = u8>>(iter: &mut I) -> u8 {
    iter.next().unwrap()
}

fn next_word<I: Iterator<Item = u8>>(iter: &mut I) -> u16 {
    let low_byte = next_byte(iter);
    let high_byte = next_byte(iter);

    u16::from_le_bytes([low_byte, high_byte])
}

impl Instruction {
    pub fn from_bytes<I>(iter: &mut I) -> Instruction
    where
        I: Iterator<Item = u8>,
    {
        let opcode = next_byte(iter);
        match opcode {
            0xa9 => Instruction::LdaImmediate(next_byte(iter)),
            
            0x8d => Instruction::StaAbsolute(next_word(iter)),
            
            0xa2 => Instruction::LdxImmediate(next_byte(iter)),
            
            0xbd => Instruction::LdaXAbsolute(next_word(iter)),
            
            0xc9 => Instruction::CmpImmediate(next_byte(iter)),
            
            0xf0 => Instruction::Beq(next_byte(iter)),
            
            0xe8 => Instruction::Inx,
            
            0x4c => Instruction::JmpAbsolute(next_word(iter)),

            0xe0 => Instruction::CpxImmediate(next_byte(iter)),

            0xd0 => Instruction::Bne(next_byte(iter)),
            _ => panic!("Cannot parse opcode {:#02x?}, either it is not implemented yet, or you reached data section by mistake", opcode)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_instructions() {
        let pairs = vec![
            (vec![0xa9u8, 0xb], Instruction::LdaImmediate(0x0b)),
            (vec![0x8d, 0x06, 0x20], Instruction::StaAbsolute(0x2006)),
            (vec![0xa2, 0x10], Instruction::LdxImmediate(0x10)),
            (vec![0xbd, 0x10, 0x20], Instruction::LdaXAbsolute(0x2010)),
            (vec![0xc9, 0xee], Instruction::CmpImmediate(0xee)),
            (vec![0xf0, 0xcc], Instruction::Beq(0xcc)),
            (vec![0xe8], Instruction::Inx),
            (vec![0x4c, 0x33, 0x66], Instruction::JmpAbsolute(0x6633)),
            (vec![0xe0, 0x0a], Instruction::CpxImmediate(0x0a)),
            (vec![0xd0, 0x87], Instruction::Bne(0x87)),
        ];

        for (opcodes, instruction) in pairs {
            assert_eq!(
                Instruction::from_bytes(&mut opcodes.into_iter()),
                instruction
            );
        }
    }
}
