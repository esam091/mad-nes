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
