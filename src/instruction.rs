pub enum Instruction {
    LdaImmediate(u8),
    StaAbsolute(u16),
}
