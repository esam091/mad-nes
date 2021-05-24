#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    AdcImmediate(u8),
    AdcZeroPage(u8),
    AdcXZeroPage(u8),
    AdcXIndexedIndirect(u8),
    AdcYIndirectIndexed(u8),
    AdcAbsolute(u16),
    AdcXAbsolute(u16),
    AdcYAbsolute(u16),

    AndImmediate(u8),
    AndZeroPage(u8),
    AndXZeroPage(u8),
    AndXIndexedIndirect(u8),
    AndYIndirectIndexed(u8),
    AndAbsolute(u16),
    AndXAbsolute(u16),
    AndYAbsolute(u16),

    Beq(u8),
    BitAbsolute(u16),
    BitZeroPage(u8),
    Brk,
    Bmi(u8),
    Bne(u8),
    Bpl(u8),
    Bvc(u8),
    Bvs(u8),
    Bcc(u8),
    Bcs(u8),

    Clc,
    Cld,
    Cli,
    Clv,

    CmpImmediate(u8),
    CmpZeroPage(u8),
    CmpXZeroPage(u8),
    CmpXIndexedIndirect(u8),
    CmpYIndirectIndexed(u8),
    CmpAbsolute(u16),
    CmpXAbsolute(u16),
    CmpYAbsolute(u16),

    CpxImmediate(u8),
    CpxZeroPage(u8),
    CpxAbsolute(u16),

    CpyImmediate(u8),
    CpyZeroPage(u8),
    CpyAbsolute(u16),

    DecZeroPage(u8),
    DecXZeroPage(u8),
    DecAbsolute(u16),
    DecXAbsolute(u16),
    Dex,
    Dey,

    EorImmediate(u8),
    EorZeroPage(u8),
    EorXZeroPage(u8),
    EorXIndexedIndirect(u8),
    EorYIndirectIndexed(u8),
    EorAbsolute(u16),
    EorXAbsolute(u16),
    EorYAbsolute(u16),

    LdaImmediate(u8),
    LdaAbsolute(u16),
    LdaXAbsolute(u16),
    LdaYAbsolute(u16),
    LdaZeroPage(u8),
    LdaXZeroPage(u8),
    LdaXIndexedIndirect(u8),
    LdaYIndirectIndexed(u8),

    LdxAbsolute(u16),
    LdxYAbsolute(u16),
    LdxImmediate(u8),
    LdxZeroPage(u8),
    LdxYZeroPage(u8),

    LdyAbsolute(u16),
    LdyXAbsolute(u16),
    LdyImmediate(u8),
    LdyZeroPage(u8),
    LdyXZeroPage(u8),

    IncZeroPage(u8),
    IncXZeroPage(u8),
    IncAbsolute(u16),
    IncXAbsolute(u16),
    Inx,
    Iny,

    JmpAbsolute(u16),
    JmpIndirect(u16),
    JsrAbsolute(u16),

    Nop,

    OraImmediate(u8),
    OraZeroPage(u8),
    OraXZeroPage(u8),
    OraXIndexedIndirect(u8),
    OraYIndirectIndexed(u8),
    OraAbsolute(u16),
    OraXAbsolute(u16),
    OraYAbsolute(u16),

    Pha,
    Php,
    Pla,
    Plp,

    Rti,
    Rts,

    Sec,
    Sed,
    Sei,

    StaAbsolute(u16),
    StaXAbsolute(u16),
    StaYAbsolute(u16),
    StaZeroPage(u8),
    StaXZeroPage(u8),
    StaXIndexedIndirect(u8),
    StaYIndirectIndexed(u8),

    StxAbsolute(u16),
    StxYZeroPage(u8),
    StxZeroPage(u8),

    StyAbsolute(u16),
    StyXZeroPage(u8),
    StyZeroPage(u8),

    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,

    Asl,
    AslZeroPage(u8),
    AslXZeroPage(u8),
    AslAbsolute(u16),
    AslXAbsolute(u16),

    Rol,
    RolZeroPage(u8),
    RolXZeroPage(u8),
    RolAbsolute(u16),
    RolXAbsolute(u16),

    Lsr,
    LsrZeroPage(u8),
    LsrXZeroPage(u8),
    LsrAbsolute(u16),
    LsrXAbsolute(u16),

    Ror,
    RorZeroPage(u8),
    RorXZeroPage(u8),
    RorAbsolute(u16),
    RorXAbsolute(u16),

    SbcImmediate(u8),
    SbcZeroPage(u8),
    SbcXZeroPage(u8),
    SbcXIndexedIndirect(u8),
    SbcYIndirectIndexed(u8),
    SbcAbsolute(u16),
    SbcXAbsolute(u16),
    SbcYAbsolute(u16),

    //illegal opcodes
    Nop2,
    NopImmediate(u8),
    NopZeroPage(u8),
    NopXZeroPage(u8),
    NopAbsolute(u16),
    NopXAbsolute(u16),

    SbcImmediateIllegal(u8),

    SloZeroPage(u8),
    SloXZeroPage(u8),
    SloXIndexedIndirect(u8),
    SloYIndirectIndexed(u8),
    SloAbsolute(u16),
    SloXAbsolute(u16),
    SloYAbsolute(u16),

    RlaZeroPage(u8),
    RlaXZeroPage(u8),
    RlaXIndexedIndirect(u8),
    RlaYIndirectIndexed(u8),
    RlaAbsolute(u16),
    RlaXAbsolute(u16),
    RlaYAbsolute(u16),

    SreZeroPage(u8),
    SreXZeroPage(u8),
    SreXIndexedIndirect(u8),
    SreYIndirectIndexed(u8),
    SreAbsolute(u16),
    SreXAbsolute(u16),
    SreYAbsolute(u16),

    RraZeroPage(u8),
    RraXZeroPage(u8),
    RraXIndexedIndirect(u8),
    RraYIndirectIndexed(u8),
    RraAbsolute(u16),
    RraXAbsolute(u16),
    RraYAbsolute(u16),

    SaxZeroPage(u8),
    SaxYZeroPage(u8),
    SaxXIndexedIndirect(u8),
    SaxAbsolute(u16),

    LaxZeroPage(u8),
    LaxYZeroPage(u8),
    LaxXIndexedIndirect(u8),
    LaxYIndirectIndexed(u8),
    LaxAbsolute(u16),
    LaxYAbsolute(u16),

    DcpZeroPage(u8),
    DcpXZeroPage(u8),
    DcpXIndexedIndirect(u8),
    DcpYIndirectIndexed(u8),
    DcpAbsolute(u16),
    DcpXAbsolute(u16),
    DcpYAbsolute(u16),

    IsbZeroPage(u8),
    IsbXZeroPage(u8),
    IsbXIndexedIndirect(u8),
    IsbYIndirectIndexed(u8),
    IsbAbsolute(u16),
    IsbXAbsolute(u16),
    IsbYAbsolute(u16),
}

#[inline(always)]
fn next_byte<I: Iterator<Item = u8>>(iter: &mut I) -> u8 {
    iter.next().unwrap()
}

#[inline(always)]
fn next_word<I: Iterator<Item = u8>>(iter: &mut I) -> u16 {
    let low_byte = next_byte(iter);
    let high_byte = next_byte(iter);

    u16::from_le_bytes([low_byte, high_byte])
}

impl Instruction {
    #[inline(always)]
    pub fn from_bytes<I>(iter: &mut I) -> Result<Instruction, u8>
    where
        I: Iterator<Item = u8>,
    {
        let opcode = next_byte(iter);
        match opcode {
            0x00 => Ok(Instruction::Brk),
            0x01 => Ok(Instruction::OraXIndexedIndirect(next_byte(iter))),
            0x05 => Ok(Instruction::OraZeroPage(next_byte(iter))),
            0x06 => Ok(Instruction::AslZeroPage(next_byte(iter))),
            0x08 => Ok(Instruction::Php),
            0x09 => Ok(Instruction::OraImmediate(next_byte(iter))),
            0x0a => Ok(Instruction::Asl),
            0x0d => Ok(Instruction::OraAbsolute(next_word(iter))),
            0x0e => Ok(Instruction::AslAbsolute(next_word(iter))),

            0x10 => Ok(Instruction::Bpl(next_byte(iter))),
            0x11 => Ok(Instruction::OraYIndirectIndexed(next_byte(iter))),
            0x15 => Ok(Instruction::OraXZeroPage(next_byte(iter))),
            0x16 => Ok(Instruction::AslXZeroPage(next_byte(iter))),
            0x18 => Ok(Instruction::Clc),
            0x19 => Ok(Instruction::OraYAbsolute(next_word(iter))),
            0x1d => Ok(Instruction::OraXAbsolute(next_word(iter))),
            0x1e => Ok(Instruction::AslXAbsolute(next_word(iter))),

            0x20 => Ok(Instruction::JsrAbsolute(next_word(iter))),
            0x21 => Ok(Instruction::AndXIndexedIndirect(next_byte(iter))),
            0x24 => Ok(Instruction::BitZeroPage(next_byte(iter))),
            0x25 => Ok(Instruction::AndZeroPage(next_byte(iter))),
            0x26 => Ok(Instruction::RolZeroPage(next_byte(iter))),
            0x28 => Ok(Instruction::Plp),
            0x29 => Ok(Instruction::AndImmediate(next_byte(iter))),
            0x2a => Ok(Instruction::Rol),
            0x2c => Ok(Instruction::BitAbsolute(next_word(iter))),
            0x2d => Ok(Instruction::AndAbsolute(next_word(iter))),
            0x2e => Ok(Instruction::RolAbsolute(next_word(iter))),

            0x30 => Ok(Instruction::Bmi(next_byte(iter))),
            0x31 => Ok(Instruction::AndYIndirectIndexed(next_byte(iter))),
            0x35 => Ok(Instruction::AndXZeroPage(next_byte(iter))),
            0x36 => Ok(Instruction::RolXZeroPage(next_byte(iter))),
            0x38 => Ok(Instruction::Sec),
            0x39 => Ok(Instruction::AndYAbsolute(next_word(iter))),
            0x3d => Ok(Instruction::AndXAbsolute(next_word(iter))),
            0x3e => Ok(Instruction::RolXAbsolute(next_word(iter))),

            0x40 => Ok(Instruction::Rti),
            0x41 => Ok(Instruction::EorXIndexedIndirect(next_byte(iter))),
            0x45 => Ok(Instruction::EorZeroPage(next_byte(iter))),
            0x46 => Ok(Instruction::LsrZeroPage(next_byte(iter))),
            0x48 => Ok(Instruction::Pha),
            0x49 => Ok(Instruction::EorImmediate(next_byte(iter))),
            0x4a => Ok(Instruction::Lsr),
            0x4c => Ok(Instruction::JmpAbsolute(next_word(iter))),
            0x4d => Ok(Instruction::EorAbsolute(next_word(iter))),
            0x4e => Ok(Instruction::LsrAbsolute(next_word(iter))),

            0x50 => Ok(Instruction::Bvc(next_byte(iter))),
            0x51 => Ok(Instruction::EorYIndirectIndexed(next_byte(iter))),
            0x55 => Ok(Instruction::EorXZeroPage(next_byte(iter))),
            0x56 => Ok(Instruction::LsrXZeroPage(next_byte(iter))),
            0x58 => Ok(Instruction::Cli),
            0x59 => Ok(Instruction::EorYAbsolute(next_word(iter))),
            0x5d => Ok(Instruction::EorXAbsolute(next_word(iter))),
            0x5e => Ok(Instruction::LsrXAbsolute(next_word(iter))),

            0x60 => Ok(Instruction::Rts),
            0x61 => Ok(Instruction::AdcXIndexedIndirect(next_byte(iter))),
            0x65 => Ok(Instruction::AdcZeroPage(next_byte(iter))),
            0x66 => Ok(Instruction::RorZeroPage(next_byte(iter))),
            0x68 => Ok(Instruction::Pla),
            0x69 => Ok(Instruction::AdcImmediate(next_byte(iter))),
            0x6a => Ok(Instruction::Ror),
            0x6c => Ok(Instruction::JmpIndirect(next_word(iter))),
            0x6d => Ok(Instruction::AdcAbsolute(next_word(iter))),
            0x6e => Ok(Instruction::RorAbsolute(next_word(iter))),

            0x70 => Ok(Instruction::Bvs(next_byte(iter))),
            0x71 => Ok(Instruction::AdcYIndirectIndexed(next_byte(iter))),
            0x75 => Ok(Instruction::AdcXZeroPage(next_byte(iter))),
            0x76 => Ok(Instruction::RorXZeroPage(next_byte(iter))),
            0x78 => Ok(Instruction::Sei),
            0x79 => Ok(Instruction::AdcYAbsolute(next_word(iter))),
            0x7d => Ok(Instruction::AdcXAbsolute(next_word(iter))),
            0x7e => Ok(Instruction::RorXAbsolute(next_word(iter))),

            0x81 => Ok(Instruction::StaXIndexedIndirect(next_byte(iter))),
            0x84 => Ok(Instruction::StyZeroPage(next_byte(iter))),
            0x85 => Ok(Instruction::StaZeroPage(next_byte(iter))),
            0x86 => Ok(Instruction::StxZeroPage(next_byte(iter))),
            0x88 => Ok(Instruction::Dey),
            0x8a => Ok(Instruction::Txa),
            0x8c => Ok(Instruction::StyAbsolute(next_word(iter))),
            0x8d => Ok(Instruction::StaAbsolute(next_word(iter))),
            0x8e => Ok(Instruction::StxAbsolute(next_word(iter))),
            0x90 => Ok(Instruction::Bcc(next_byte(iter))),
            0x91 => Ok(Instruction::StaYIndirectIndexed(next_byte(iter))),
            0x94 => Ok(Instruction::StyXZeroPage(next_byte(iter))),
            0x95 => Ok(Instruction::StaXZeroPage(next_byte(iter))),
            0x96 => Ok(Instruction::StxYZeroPage(next_byte(iter))),
            0x98 => Ok(Instruction::Tya),
            0x99 => Ok(Instruction::StaYAbsolute(next_word(iter))),
            0x9a => Ok(Instruction::Txs),
            0x9d => Ok(Instruction::StaXAbsolute(next_word(iter))),

            0xa0 => Ok(Instruction::LdyImmediate(next_byte(iter))),
            0xa1 => Ok(Instruction::LdaXIndexedIndirect(next_byte(iter))),
            0xa2 => Ok(Instruction::LdxImmediate(next_byte(iter))),
            0xa4 => Ok(Instruction::LdyZeroPage(next_byte(iter))),
            0xa5 => Ok(Instruction::LdaZeroPage(next_byte(iter))),
            0xa6 => Ok(Instruction::LdxZeroPage(next_byte(iter))),
            0xa8 => Ok(Instruction::Tay),
            0xa9 => Ok(Instruction::LdaImmediate(next_byte(iter))),
            0xaa => Ok(Instruction::Tax),
            0xac => Ok(Instruction::LdyAbsolute(next_word(iter))),
            0xad => Ok(Instruction::LdaAbsolute(next_word(iter))),
            0xae => Ok(Instruction::LdxAbsolute(next_word(iter))),

            0xb0 => Ok(Instruction::Bcs(next_byte(iter))),
            0xb1 => Ok(Instruction::LdaYIndirectIndexed(next_byte(iter))),
            0xb4 => Ok(Instruction::LdyXZeroPage(next_byte(iter))),
            0xb5 => Ok(Instruction::LdaXZeroPage(next_byte(iter))),
            0xb6 => Ok(Instruction::LdxYZeroPage(next_byte(iter))),
            0xb8 => Ok(Instruction::Clv),
            0xb9 => Ok(Instruction::LdaYAbsolute(next_word(iter))),
            0xba => Ok(Instruction::Tsx),
            0xbc => Ok(Instruction::LdyXAbsolute(next_word(iter))),
            0xbd => Ok(Instruction::LdaXAbsolute(next_word(iter))),
            0xbe => Ok(Instruction::LdxYAbsolute(next_word(iter))),

            0xc0 => Ok(Instruction::CpyImmediate(next_byte(iter))),
            0xc1 => Ok(Instruction::CmpXIndexedIndirect(next_byte(iter))),
            0xc4 => Ok(Instruction::CpyZeroPage(next_byte(iter))),
            0xc5 => Ok(Instruction::CmpZeroPage(next_byte(iter))),
            0xc6 => Ok(Instruction::DecZeroPage(next_byte(iter))),
            0xc8 => Ok(Instruction::Iny),
            0xc9 => Ok(Instruction::CmpImmediate(next_byte(iter))),
            0xca => Ok(Instruction::Dex),
            0xcc => Ok(Instruction::CpyAbsolute(next_word(iter))),
            0xcd => Ok(Instruction::CmpAbsolute(next_word(iter))),
            0xce => Ok(Instruction::DecAbsolute(next_word(iter))),

            0xd0 => Ok(Instruction::Bne(next_byte(iter))),
            0xd1 => Ok(Instruction::CmpYIndirectIndexed(next_byte(iter))),
            0xd5 => Ok(Instruction::CmpXZeroPage(next_byte(iter))),
            0xd6 => Ok(Instruction::DecXZeroPage(next_byte(iter))),
            0xd8 => Ok(Instruction::Cld),
            0xd9 => Ok(Instruction::CmpYAbsolute(next_word(iter))),
            0xdd => Ok(Instruction::CmpXAbsolute(next_word(iter))),
            0xde => Ok(Instruction::DecXAbsolute(next_word(iter))),

            0xe0 => Ok(Instruction::CpxImmediate(next_byte(iter))),
            0xe1 => Ok(Instruction::SbcXIndexedIndirect(next_byte(iter))),
            0xe4 => Ok(Instruction::CpxZeroPage(next_byte(iter))),
            0xe5 => Ok(Instruction::SbcZeroPage(next_byte(iter))),
            0xe6 => Ok(Instruction::IncZeroPage(next_byte(iter))),
            0xe8 => Ok(Instruction::Inx),
            0xe9 => Ok(Instruction::SbcImmediate(next_byte(iter))),
            0xea => Ok(Instruction::Nop),
            0xec => Ok(Instruction::CpxAbsolute(next_word(iter))),
            0xed => Ok(Instruction::SbcAbsolute(next_word(iter))),
            0xee => Ok(Instruction::IncAbsolute(next_word(iter))),

            0xf0 => Ok(Instruction::Beq(next_byte(iter))),
            0xf1 => Ok(Instruction::SbcYIndirectIndexed(next_byte(iter))),
            0xf5 => Ok(Instruction::SbcXZeroPage(next_byte(iter))),
            0xf6 => Ok(Instruction::IncXZeroPage(next_byte(iter))),
            0xf8 => Ok(Instruction::Sed),
            0xf9 => Ok(Instruction::SbcYAbsolute(next_word(iter))),
            0xfd => Ok(Instruction::SbcXAbsolute(next_word(iter))),
            0xfe => Ok(Instruction::IncXAbsolute(next_word(iter))),

            // Illegal opcodes
            0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => Ok(Instruction::NopImmediate(next_byte(iter))),
            0x04 | 0x44 | 0x64 => Ok(Instruction::NopZeroPage(next_byte(iter))),
            0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => {
                Ok(Instruction::NopXZeroPage(next_byte(iter)))
            }
            0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => Ok(Instruction::Nop2),
            0x0c => Ok(Instruction::NopAbsolute(next_word(iter))),
            0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => {
                Ok(Instruction::NopXAbsolute(next_word(iter)))
            }

            0x07 => Ok(Instruction::SloZeroPage(next_byte(iter))),
            0x17 => Ok(Instruction::SloXZeroPage(next_byte(iter))),
            0x03 => Ok(Instruction::SloXIndexedIndirect(next_byte(iter))),
            0x13 => Ok(Instruction::SloYIndirectIndexed(next_byte(iter))),
            0x0f => Ok(Instruction::SloAbsolute(next_word(iter))),
            0x1f => Ok(Instruction::SloXAbsolute(next_word(iter))),
            0x1b => Ok(Instruction::SloYAbsolute(next_word(iter))),

            0x27 => Ok(Instruction::RlaZeroPage(next_byte(iter))),
            0x37 => Ok(Instruction::RlaXZeroPage(next_byte(iter))),
            0x23 => Ok(Instruction::RlaXIndexedIndirect(next_byte(iter))),
            0x33 => Ok(Instruction::RlaYIndirectIndexed(next_byte(iter))),
            0x2f => Ok(Instruction::RlaAbsolute(next_word(iter))),
            0x3f => Ok(Instruction::RlaXAbsolute(next_word(iter))),
            0x3b => Ok(Instruction::RlaYAbsolute(next_word(iter))),

            0x47 => Ok(Instruction::SreZeroPage(next_byte(iter))),
            0x57 => Ok(Instruction::SreXZeroPage(next_byte(iter))),
            0x43 => Ok(Instruction::SreXIndexedIndirect(next_byte(iter))),
            0x53 => Ok(Instruction::SreYIndirectIndexed(next_byte(iter))),
            0x4f => Ok(Instruction::SreAbsolute(next_word(iter))),
            0x5f => Ok(Instruction::SreXAbsolute(next_word(iter))),
            0x5b => Ok(Instruction::SreYAbsolute(next_word(iter))),

            0x67 => Ok(Instruction::RraZeroPage(next_byte(iter))),
            0x77 => Ok(Instruction::RraXZeroPage(next_byte(iter))),
            0x63 => Ok(Instruction::RraXIndexedIndirect(next_byte(iter))),
            0x73 => Ok(Instruction::RraYIndirectIndexed(next_byte(iter))),
            0x6f => Ok(Instruction::RraAbsolute(next_word(iter))),
            0x7f => Ok(Instruction::RraXAbsolute(next_word(iter))),
            0x7b => Ok(Instruction::RraYAbsolute(next_word(iter))),

            0x87 => Ok(Instruction::SaxZeroPage(next_byte(iter))),
            0x97 => Ok(Instruction::SaxYZeroPage(next_byte(iter))),
            0x83 => Ok(Instruction::SaxXIndexedIndirect(next_byte(iter))),
            0x8f => Ok(Instruction::SaxAbsolute(next_word(iter))),

            0xa7 => Ok(Instruction::LaxZeroPage(next_byte(iter))),
            0xb7 => Ok(Instruction::LaxYZeroPage(next_byte(iter))),
            0xa3 => Ok(Instruction::LaxXIndexedIndirect(next_byte(iter))),
            0xb3 => Ok(Instruction::LaxYIndirectIndexed(next_byte(iter))),
            0xaf => Ok(Instruction::LaxAbsolute(next_word(iter))),
            0xbf => Ok(Instruction::LaxYAbsolute(next_word(iter))),

            0xc7 => Ok(Instruction::DcpZeroPage(next_byte(iter))),
            0xd7 => Ok(Instruction::DcpXZeroPage(next_byte(iter))),
            0xc3 => Ok(Instruction::DcpXIndexedIndirect(next_byte(iter))),
            0xd3 => Ok(Instruction::DcpYIndirectIndexed(next_byte(iter))),
            0xcf => Ok(Instruction::DcpAbsolute(next_word(iter))),
            0xdf => Ok(Instruction::DcpXAbsolute(next_word(iter))),
            0xdb => Ok(Instruction::DcpYAbsolute(next_word(iter))),

            0xe7 => Ok(Instruction::IsbZeroPage(next_byte(iter))),
            0xf7 => Ok(Instruction::IsbXZeroPage(next_byte(iter))),
            0xe3 => Ok(Instruction::IsbXIndexedIndirect(next_byte(iter))),
            0xf3 => Ok(Instruction::IsbYIndirectIndexed(next_byte(iter))),
            0xef => Ok(Instruction::IsbAbsolute(next_word(iter))),
            0xff => Ok(Instruction::IsbXAbsolute(next_word(iter))),
            0xfb => Ok(Instruction::IsbYAbsolute(next_word(iter))),

            0xeb => Ok(Instruction::SbcImmediateIllegal(next_byte(iter))),
            _ => Err(opcode),
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
            (vec![0x10, 0xaa], Instruction::Bpl(0xaa)),
            (vec![0x30, 0xdd], Instruction::Bmi(0xdd)),
            (vec![0x50, 0xcc], Instruction::Bvc(0xcc)),
            (vec![0x70, 0x48], Instruction::Bvs(0x48)),
            (vec![0x18], Instruction::Clc),
            (vec![0x38], Instruction::Sec),
            (vec![0xd8], Instruction::Cld),
            (vec![0xf8], Instruction::Sed),
            (vec![0x58], Instruction::Cli),
            (vec![0x78], Instruction::Sei),
            (vec![0xb8], Instruction::Clv),
            (vec![0xea], Instruction::Nop),
            (vec![0x00], Instruction::Brk),
            (vec![0x40], Instruction::Rti),
            (vec![0x60], Instruction::Rts),
            (vec![0x20, 0xbb, 0xdd], Instruction::JsrAbsolute(0xddbb)),
            (vec![0x6c, 0xaa, 0xff], Instruction::JmpIndirect(0xffaa)),
            (vec![0x2c, 0x78, 0x90], Instruction::BitAbsolute(0x9078)),
            (vec![0x24, 0x38], Instruction::BitZeroPage(0x38)),
            (vec![0xaa], Instruction::Tax),
            (vec![0x8a], Instruction::Txa),
            (vec![0xa8], Instruction::Tay),
            (vec![0x98], Instruction::Tya),
            (vec![0xba], Instruction::Tsx),
            (vec![0x9a], Instruction::Txs),
            (vec![0x68], Instruction::Pla),
            (vec![0x48], Instruction::Pha),
            (vec![0x28], Instruction::Plp),
            (vec![0x8], Instruction::Php),
            (vec![0x8c, 0x98, 0x37], Instruction::StyAbsolute(0x3798)),
            (vec![0x84, 0xaa], Instruction::StyZeroPage(0xaa)),
            (vec![0x94, 0xcc], Instruction::StyXZeroPage(0xcc)),
            (vec![0xa0, 0x47], Instruction::LdyImmediate(0x47)),
            (vec![0xa4, 0x18], Instruction::LdyZeroPage(0x18)),
            (vec![0xb4, 0x77], Instruction::LdyXZeroPage(0x77)),
            (vec![0xac, 0x12, 0x34], Instruction::LdyAbsolute(0x3412)),
            (vec![0xbc, 0x78, 0x56], Instruction::LdyXAbsolute(0x5678)),
            (vec![0x86, 0x22], Instruction::StxZeroPage(0x22)),
            (vec![0x96, 0x57], Instruction::StxYZeroPage(0x57)),
            (vec![0x8e, 0x19, 0x88], Instruction::StxAbsolute(0x8819)),
            (vec![0xa6, 0xbb], Instruction::LdxZeroPage(0xbb)),
            (vec![0xb6, 0x33], Instruction::LdxYZeroPage(0x33)),
            (vec![0xae, 0x11, 0x22], Instruction::LdxAbsolute(0x2211)),
            (vec![0xbe, 0x33, 0x44], Instruction::LdxYAbsolute(0x4433)),
            (vec![0x85, 0x89], Instruction::StaZeroPage(0x89)),
            (vec![0x95, 0x74], Instruction::StaXZeroPage(0x74)),
            (vec![0x81, 0x88], Instruction::StaXIndexedIndirect(0x88)),
            (vec![0x91, 0x99], Instruction::StaYIndirectIndexed(0x99)),
            (vec![0x9d, 0xaa, 0xff], Instruction::StaXAbsolute(0xffaa)),
            (vec![0x99, 0xee, 0xcc], Instruction::StaYAbsolute(0xccee)),
            (vec![0xa5, 0x89], Instruction::LdaZeroPage(0x89)),
            (vec![0xb5, 0x88], Instruction::LdaXZeroPage(0x88)),
            (vec![0xa1, 0x47], Instruction::LdaXIndexedIndirect(0x47)),
            (vec![0xb1, 0x48], Instruction::LdaYIndirectIndexed(0x48)),
            (vec![0xad, 0x39, 0x19], Instruction::LdaAbsolute(0x1939)),
            (vec![0xb9, 0x22, 0x44], Instruction::LdaYAbsolute(0x4422)),
            (vec![0x0a], Instruction::Asl),
            (vec![0x06, 0x02], Instruction::AslZeroPage(0x02)),
            (vec![0x16, 0x58], Instruction::AslXZeroPage(0x58)),
            (vec![0x0e, 0x48, 0x02], Instruction::AslAbsolute(0x0248)),
            (vec![0x1e, 0x28, 0x29], Instruction::AslXAbsolute(0x2928)),
            (vec![0x2a, 0xa0], Instruction::Rol),
            (vec![0x26, 0x48], Instruction::RolZeroPage(0x48)),
            (vec![0x36, 0x80], Instruction::RolXZeroPage(0x80)),
            (vec![0x2e, 0x72, 0x46], Instruction::RolAbsolute(0x4672)),
            (vec![0x3e, 0x83, 0x29], Instruction::RolXAbsolute(0x2983)),
            (vec![0x4a], Instruction::Lsr),
            (vec![0x46, 0x32], Instruction::LsrZeroPage(0x32)),
            (vec![0x56, 0x28], Instruction::LsrXZeroPage(0x28)),
            (vec![0x4e, 0x38, 0x93], Instruction::LsrAbsolute(0x9338)),
            (vec![0x5e, 0xab, 0xcd], Instruction::LsrXAbsolute(0xcdab)),
            (vec![0x6a], Instruction::Ror),
            (vec![0x66, 0x33], Instruction::RorZeroPage(0x33)),
            (vec![0x76, 0x28], Instruction::RorXZeroPage(0x28)),
            (vec![0x6e, 0x39, 0x12], Instruction::RorAbsolute(0x1239)),
            (vec![0x7e, 0x38, 0x37], Instruction::RorXAbsolute(0x3738)),
            (vec![0xc8], Instruction::Iny),
            (vec![0xe6, 0xa0], Instruction::IncZeroPage(0xa0)),
            (vec![0xf6, 0x48], Instruction::IncXZeroPage(0x48)),
            (vec![0xee, 0x74, 0x37], Instruction::IncAbsolute(0x3774)),
            (vec![0xfe, 0x22, 0x33], Instruction::IncXAbsolute(0x3322)),
            (vec![0xca], Instruction::Dex),
            (vec![0x88], Instruction::Dey),
            (vec![0xc6, 0xaa], Instruction::DecZeroPage(0xaa)),
            (vec![0xd6, 0xbb], Instruction::DecXZeroPage(0xbb)),
            (vec![0xce, 0xab, 0xcd], Instruction::DecAbsolute(0xcdab)),
            (vec![0xde, 0xcd, 0xab], Instruction::DecXAbsolute(0xabcd)),
            (vec![0xe4, 0xa0], Instruction::CpxZeroPage(0xa0)),
            (vec![0xec, 0x23, 0x34], Instruction::CpxAbsolute(0x3423)),
            (vec![0xc0, 0xbb], Instruction::CpyImmediate(0xbb)),
            (vec![0xc4, 0xee], Instruction::CpyZeroPage(0xee)),
            (vec![0xcc, 0xcd, 0xef], Instruction::CpyAbsolute(0xefcd)),
            (vec![0xc5, 0x28], Instruction::CmpZeroPage(0x28)),
            (vec![0xd5, 0x18], Instruction::CmpXZeroPage(0x18)),
            (vec![0xc1, 0x23], Instruction::CmpXIndexedIndirect(0x23)),
            (vec![0xd1, 0x39], Instruction::CmpYIndirectIndexed(0x39)),
            (vec![0xcd, 0x32, 0x92], Instruction::CmpAbsolute(0x9232)),
            (vec![0xdd, 0x38, 0x11], Instruction::CmpXAbsolute(0x1138)),
            (vec![0xd9, 0x12, 0x34], Instruction::CmpYAbsolute(0x3412)),
            (vec![0xe9, 0x11], Instruction::SbcImmediate(0x11)),
            (vec![0xe5, 0x35], Instruction::SbcZeroPage(0x35)),
            (vec![0xf5, 0x92], Instruction::SbcXZeroPage(0x92)),
            (vec![0xe1, 0xae], Instruction::SbcXIndexedIndirect(0xae)),
            (vec![0xf1, 0xea], Instruction::SbcYIndirectIndexed(0xea)),
            (vec![0xed, 0x44, 0x88], Instruction::SbcAbsolute(0x8844)),
            (vec![0xfd, 0x22, 0x55], Instruction::SbcXAbsolute(0x5522)),
            (vec![0xf9, 0x11, 0x33], Instruction::SbcYAbsolute(0x3311)),
            (vec![0x69, 0xa2], Instruction::AdcImmediate(0xa2)),
            (vec![0x65, 0x39], Instruction::AdcZeroPage(0x39)),
            (vec![0x75, 0x19], Instruction::AdcXZeroPage(0x19)),
            (vec![0x61, 0x82], Instruction::AdcXIndexedIndirect(0x82)),
            (vec![0x71, 0x61], Instruction::AdcYIndirectIndexed(0x61)),
            (vec![0x6d, 0x65, 0xb8], Instruction::AdcAbsolute(0xb865)),
            (vec![0x7d, 0xb7, 0xc0], Instruction::AdcXAbsolute(0xc0b7)),
            (vec![0x79, 0x11, 0x33], Instruction::AdcYAbsolute(0x3311)),
            (vec![0x49, 0x67], Instruction::EorImmediate(0x67)),
            (vec![0x45, 0x55], Instruction::EorZeroPage(0x55)),
            (vec![0x41, 0x33], Instruction::EorXIndexedIndirect(0x33)),
            (vec![0x51, 0x99], Instruction::EorYIndirectIndexed(0x99)),
            (vec![0x4d, 0x01, 0x02], Instruction::EorAbsolute(0x0201)),
            (vec![0x5d, 0x03, 0x50], Instruction::EorXAbsolute(0x5003)),
            (vec![0x59, 0x19, 0x28], Instruction::EorYAbsolute(0x2819)),
            (vec![0x29, 0xaa], Instruction::AndImmediate(0xaa)),
            (vec![0x25, 0xab], Instruction::AndZeroPage(0xab)),
            (vec![0x35, 0x31, 0x32], Instruction::AndXZeroPage(0x31)),
            (
                vec![0x21, 0x22, 0x23],
                Instruction::AndXIndexedIndirect(0x22),
            ),
            (vec![0x31, 0x33], Instruction::AndYIndirectIndexed(0x33)),
            (vec![0x2d, 0xac, 0xca], Instruction::AndAbsolute(0xcaac)),
            (vec![0x3d, 0xbb, 0xcc], Instruction::AndXAbsolute(0xccbb)),
            (vec![0x39, 0xcd, 0x7a], Instruction::AndYAbsolute(0x7acd)),
            (vec![0x09, 0xac], Instruction::OraImmediate(0xac)),
            (vec![0x05, 0xbd], Instruction::OraZeroPage(0xbd)),
            (vec![0x15, 0xc1], Instruction::OraXZeroPage(0xc1)),
            (vec![0x01, 0xd2], Instruction::OraXIndexedIndirect(0xd2)),
            (vec![0x11, 0xe9], Instruction::OraYIndirectIndexed(0xe9)),
            (vec![0x0d, 0xd2, 0xf3], Instruction::OraAbsolute(0xf3d2)),
            (vec![0x1d, 0x82, 0xc3], Instruction::OraXAbsolute(0xc382)),
            (vec![0x19, 0x22, 0x99], Instruction::OraYAbsolute(0x9922)),
            (vec![0xb0, 0x04], Instruction::Bcs(0x04)),
            (vec![0x90, 0xc0], Instruction::Bcc(0xc0)),
            // illegal opcodes
            (vec![0x80, 0xaa], Instruction::NopImmediate(0xaa)),
            (vec![0x82, 0x11], Instruction::NopImmediate(0x11)),
            (vec![0x89, 0x11], Instruction::NopImmediate(0x11)),
            (vec![0xc2, 0xab], Instruction::NopImmediate(0xab)),
            (vec![0xe2, 0x8d], Instruction::NopImmediate(0x8d)),
            (vec![0x04, 0x2b], Instruction::NopZeroPage(0x2b)),
            (vec![0x44, 0x2b], Instruction::NopZeroPage(0x2b)),
            (vec![0x64, 0x2b], Instruction::NopZeroPage(0x2b)),
            (vec![0x14, 0x2b], Instruction::NopXZeroPage(0x2b)),
            (vec![0x34, 0x2b], Instruction::NopXZeroPage(0x2b)),
            (vec![0x54, 0x2b], Instruction::NopXZeroPage(0x2b)),
            (vec![0x74, 0x2b], Instruction::NopXZeroPage(0x2b)),
            (vec![0xd4, 0x2b], Instruction::NopXZeroPage(0x2b)),
            (vec![0xf4, 0x2b], Instruction::NopXZeroPage(0x2b)),
            (vec![0x1a], Instruction::Nop2),
            (vec![0x3a], Instruction::Nop2),
            (vec![0x5a], Instruction::Nop2),
            (vec![0x7a], Instruction::Nop2),
            (vec![0xda], Instruction::Nop2),
            (vec![0xfa], Instruction::Nop2),
            (vec![0x0c, 0xff, 0x23], Instruction::NopAbsolute(0x23ff)),
            (vec![0x1c, 0xff, 0x23], Instruction::NopXAbsolute(0x23ff)),
            (vec![0x3c, 0xff, 0x23], Instruction::NopXAbsolute(0x23ff)),
            (vec![0x5c, 0xff, 0x23], Instruction::NopXAbsolute(0x23ff)),
            (vec![0x7c, 0xff, 0x23], Instruction::NopXAbsolute(0x23ff)),
            (vec![0xdc, 0xff, 0x23], Instruction::NopXAbsolute(0x23ff)),
            (vec![0xfc, 0xff, 0x23], Instruction::NopXAbsolute(0x23ff)),
            (vec![0x07, 0x39], Instruction::SloZeroPage(0x39)),
            (vec![0x17, 0x23], Instruction::SloXZeroPage(0x23)),
            (vec![0x03, 0x0a], Instruction::SloXIndexedIndirect(0x0a)),
            (vec![0x13, 0xb2], Instruction::SloYIndirectIndexed(0xb2)),
            (vec![0x0f, 0xdd, 0x25], Instruction::SloAbsolute(0x25dd)),
            (vec![0x1f, 0x9c, 0x22], Instruction::SloXAbsolute(0x229c)),
            (vec![0x1b, 0x98, 0x78], Instruction::SloYAbsolute(0x7898)),
            (vec![0x27, 0xaa], Instruction::RlaZeroPage(0xaa)),
            (vec![0x37, 0xcc], Instruction::RlaXZeroPage(0xcc)),
            (vec![0x23, 0x8d], Instruction::RlaXIndexedIndirect(0x8d)),
            (vec![0x33, 0x93], Instruction::RlaYIndirectIndexed(0x93)),
            (vec![0x2f, 0xc0, 0x92], Instruction::RlaAbsolute(0x92c0)),
            (vec![0x3f, 0xff, 0xcd], Instruction::RlaXAbsolute(0xcdff)),
            (vec![0x3b, 0x99, 0x22], Instruction::RlaYAbsolute(0x2299)),
            (vec![0x47, 0x12], Instruction::SreZeroPage(0x12)),
            (vec![0x57, 0x99], Instruction::SreXZeroPage(0x99)),
            (vec![0x43, 0x38], Instruction::SreXIndexedIndirect(0x38)),
            (vec![0x53, 0x18], Instruction::SreYIndirectIndexed(0x18)),
            (vec![0x4f, 0xaa, 0xbb], Instruction::SreAbsolute(0xbbaa)),
            (vec![0x5f, 0xdd, 0xcc], Instruction::SreXAbsolute(0xccdd)),
            (vec![0x5b, 0xff, 0xee], Instruction::SreYAbsolute(0xeeff)),
            (vec![0x67, 0x92], Instruction::RraZeroPage(0x92)),
            (vec![0x77, 0x92], Instruction::RraXZeroPage(0x92)),
            (vec![0x63, 0x92], Instruction::RraXIndexedIndirect(0x92)),
            (vec![0x73, 0x19], Instruction::RraYIndirectIndexed(0x19)),
            (vec![0x6f, 0x22, 0x39], Instruction::RraAbsolute(0x3922)),
            (vec![0x7f, 0x3c, 0x93], Instruction::RraXAbsolute(0x933c)),
            (vec![0x7b, 0xab, 0xcd], Instruction::RraYAbsolute(0xcdab)),
            (vec![0x87, 0xcd], Instruction::SaxZeroPage(0xcd)),
            (vec![0x97, 0xff], Instruction::SaxYZeroPage(0xff)),
            (vec![0x83, 0x99], Instruction::SaxXIndexedIndirect(0x99)),
            (vec![0x8f, 0x55, 0x33], Instruction::SaxAbsolute(0x3355)),
            (vec![0xa7, 0x37], Instruction::LaxZeroPage(0x37)),
            (vec![0xb7, 0xc2], Instruction::LaxYZeroPage(0xc2)),
            (vec![0xa3, 0xc4], Instruction::LaxXIndexedIndirect(0xc4)),
            (vec![0xb3, 0xd9], Instruction::LaxYIndirectIndexed(0xd9)),
            (vec![0xaf, 0x2d, 0xff], Instruction::LaxAbsolute(0xff2d)),
            (vec![0xbf, 0xd0, 0xd0], Instruction::LaxYAbsolute(0xd0d0)),
            (vec![0xc7, 0xaa], Instruction::DcpZeroPage(0xaa)),
            (vec![0xd7, 0xbb], Instruction::DcpXZeroPage(0xbb)),
            (vec![0xc3, 0xdd], Instruction::DcpXIndexedIndirect(0xdd)),
            (vec![0xd3, 0xff], Instruction::DcpYIndirectIndexed(0xff)),
            (vec![0xcf, 0xbb, 0xbc], Instruction::DcpAbsolute(0xbcbb)),
            (vec![0xdf, 0xd2, 0xc5], Instruction::DcpXAbsolute(0xc5d2)),
            (vec![0xdb, 0x22, 0x12], Instruction::DcpYAbsolute(0x1222)),
            (vec![0xe7, 0xc0], Instruction::IsbZeroPage(0xc0)),
            (vec![0xf7, 0xca, 0xcb], Instruction::IsbXZeroPage(0xca)),
            (vec![0xe3, 0xcf], Instruction::IsbXIndexedIndirect(0xcf)),
            (vec![0xf3, 0xdf], Instruction::IsbYIndirectIndexed(0xdf)),
            (vec![0xef, 0xab, 0xcd], Instruction::IsbAbsolute(0xcdab)),
            (vec![0xff, 0xdc, 0xcc], Instruction::IsbXAbsolute(0xccdc)),
            (vec![0xfb, 0x29, 0x88], Instruction::IsbYAbsolute(0x8829)),
            (vec![0xeb, 0x22], Instruction::SbcImmediateIllegal(0x22)),
        ];

        for (opcodes, instruction) in pairs {
            let result = Instruction::from_bytes(&mut opcodes.clone().into_iter())
                .map_err(|opcode| format!("Parsing opcodes failed: {:02X?}", opcode))
                .unwrap();
            assert_eq!(result, instruction);
        }
    }
}
