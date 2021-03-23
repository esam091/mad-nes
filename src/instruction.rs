pub enum Instruction {
    LdaImmediate(u8),
    StaAbsolute(u16),
    LdxImmediate(u8),
    LdaXAbsolute(u16),
}
