#[derive(Debug)]
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

impl Instruction {
    pub fn cycles(&self) -> u32 {
        match self {
            Instruction::LdaImmediate(_)
            | Instruction::LdxImmediate(_)
            | Instruction::CmpImmediate(_)
            | Instruction::CpxImmediate(_)
            | Instruction::Beq(_)
            | Instruction::Bne(_)
            | Instruction::Inx => 2,

            Instruction::StaAbsolute(_) | Instruction::LdaXAbsolute(_) => 4,

            Instruction::JmpAbsolute(_) => 5,
        }
    }
}
