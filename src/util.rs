pub fn high_nibble(value: u8) -> u8 {
    (value & 0xF0) >> 4
}

pub fn low_nibble(value: u8) -> u8 {
    value & 0x0F
}
