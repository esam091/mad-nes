pub fn hex_string(value: u16) -> String {
    format!("{:#04X?}", value)
}

pub fn binary_string(value: u16) -> String {
    format!("{:16b}", value)
}

pub fn hex_string8(value: u8) -> String {
    format!("{:#02X?}", value)
}

#[macro_export]
macro_rules! log_ppu {
    ($($arg:tt)*) => ({
        #[cfg(feature = "log_ppu")]
        println!($($arg)*);
    })
}

#[macro_export]
macro_rules! log_apu {
    ($($arg:tt)*) => ({
        #[cfg(feature = "log_apu")]
        println!($($arg)*);
    })
}
