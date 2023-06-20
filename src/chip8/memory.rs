use std::ops::{Add, AddAssign};

#[derive(Debug)]
pub struct Memory([u8; 4096]);
impl Memory {
    pub fn new() -> Self {
        Self([0; 4096])
    }

    pub fn read_bytes(&self, addr: MemoryAddress, num_bytes: usize) -> &[u8] {
        let start = (addr.0 & 0x0FFF) as usize;
        let end = start + num_bytes;
        let end = if end > 4096 { 4096 } else { end };
        &self.0[start..end]
    }

    pub fn write_bytes(&mut self, addr: MemoryAddress, data: &[u8]) {
        let start = (addr.0 & 0x0FFF) as usize;
        let end = start + data.len();
        self.0[start..end].copy_from_slice(data);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MemoryAddress(pub u16);
impl MemoryAddress {
    pub const ZERO: Self = Self(0);
    pub const PROGRAM_START: Self = Self(0x200);

    pub fn next_instruction(&mut self) {
        *self += 2;
    }
}

impl Add<u8> for MemoryAddress {
    type Output = Self;

    fn add(self, other: u8) -> Self {
        Self(self.0.wrapping_add(other as u16))
    }
}

impl AddAssign<u8> for MemoryAddress {
    fn add_assign(&mut self, other: u8) {
        self.0 = self.0.wrapping_add(other as u16);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_address_next_instruction() {
        let mut addr = MemoryAddress::PROGRAM_START;
        addr.next_instruction();
        assert_eq!(addr, MemoryAddress::PROGRAM_START + 2);
    }

    #[test]
    fn test_memory_read_write() {
        let mut memory = Memory::new();
        let data = &[0xDE, 0xAD, 0xBE, 0xEF];
        memory.write_bytes(MemoryAddress::ZERO, data);
        let result = memory.read_bytes(MemoryAddress::ZERO, 4);
        assert_eq!(result, data);
    }
}
