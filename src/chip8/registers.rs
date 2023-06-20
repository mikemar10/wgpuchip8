use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Registers([u8; 18]);

#[allow(dead_code)]
impl Registers {
    pub const V0: u8 = 0x0;
    pub const V1: u8 = 0x1;
    pub const V2: u8 = 0x2;
    pub const V3: u8 = 0x3;
    pub const V4: u8 = 0x4;
    pub const V5: u8 = 0x5;
    pub const V6: u8 = 0x6;
    pub const V7: u8 = 0x7;
    pub const V8: u8 = 0x8;
    pub const V9: u8 = 0x9;
    pub const VA: u8 = 0xA;
    pub const VB: u8 = 0xB;
    pub const VC: u8 = 0xC;
    pub const VD: u8 = 0xD;
    pub const VE: u8 = 0xE;
    pub const VF: u8 = 0xF;
    pub const DT: u8 = 0x10;
    pub const ST: u8 = 0x11;

    pub fn new() -> Self {
        Self([0; 18])
    }

    pub fn get_slice(&self, vx: u8, vy: u8) -> &[u8] {
        &self.0[(vx as usize)..=(vy as usize)]
    }

    pub fn get_slice_mut(&mut self, vx: u8, vy: u8) -> &mut [u8] {
        &mut self.0[(vx as usize)..=(vy as usize)]
    }

    pub fn cmp_scalar(&self, vx: u8, value: u8) -> bool {
        self[vx] == value
    }

    pub fn cmp_register(&self, vx: u8, vy: u8) -> bool {
        self[vx] == self[vy]
    }

    pub fn load_scalar(&mut self, vx: u8, value: u8) {
        self[vx] = value;
    }

    pub fn load_register(&mut self, vx: u8, vy: u8) {
        self[vx] = self[vy];
    }

    pub fn shift_right(&mut self, vx: u8) {
        self[Self::VF] = if self[vx] & 1 == 1 { 1 } else { 0 };
        self[vx] = self[vx].wrapping_shr(1);
    }

    pub fn shift_left(&mut self, vx: u8) {
        self[Self::VF] = if self[vx] & 0x80 == 0x80 { 1 } else { 0 };
        self[vx] = self[vx].wrapping_shl(1);
    }

    pub fn add_scalar(&mut self, vx: u8, value: u8) {
        self[vx] = self[vx].wrapping_add(value);
    }

    pub fn add_register(&mut self, vx: u8, vy: u8) {
        let (result, overflow) = self[vx].overflowing_add(self[vy]);
        self[Self::VF] = if overflow { 1 } else { 0 };
        self[vx] = result;
    }

    pub fn sub_register(&mut self, vx: u8, vy: u8) {
        self[Self::VF] = if self[vx] > self[vy] { 1 } else { 0 };
        self[vx] = self[vx].wrapping_sub(self[vy]);
    }

    pub fn subn_register(&mut self, vx: u8, vy: u8) {
        self[Self::VF] = if self[vy] > self[vx] { 1 } else { 0 };
        self[vx] = self[vy].wrapping_sub(self[vx]);
    }

    pub fn or_register(&mut self, vx: u8, vy: u8) {
        self[vx] |= self[vy];
    }

    pub fn and_register(&mut self, vx: u8, vy: u8) {
        self[vx] &= self[vy];
    }

    pub fn xor_register(&mut self, vx: u8, vy: u8) {
        self[vx] ^= self[vy];
    }
}

impl Index<u8> for Registers {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_scalar() {
        let mut registers = Registers::new();
        registers.load_scalar(Registers::V0, 0x23);
        assert_eq!(registers[Registers::V0], 0x23);
    }

    #[test]
    fn test_or_register() {
        let mut registers = Registers::new();
        registers.load_scalar(Registers::V0, 0xAF);
        registers.load_scalar(Registers::V1, 0xF0);
        registers.or_register(Registers::V0, Registers::V1);
        assert_eq!(registers[Registers::V0], 0xFF);
        assert_eq!(registers[Registers::V1], 0xF0);
    }

    #[test]
    fn test_and_register() {
        let mut registers = Registers::new();
        registers.load_scalar(Registers::V0, 0x3);
        registers.load_scalar(Registers::V1, 0x2);
        registers.and_register(Registers::V0, Registers::V1);
        assert_eq!(registers[Registers::V0], 0x2);
        assert_eq!(registers[Registers::V1], 0x2);
    }

    #[test]
    fn test_xor_register() {
        let mut registers = Registers::new();
        registers.load_scalar(Registers::V0, 0xF0);
        registers.load_scalar(Registers::V1, 0x0F);
        registers.xor_register(Registers::V0, Registers::V1);
        assert_eq!(registers[Registers::V0], 0xFF);
        assert_eq!(registers[Registers::V1], 0x0F);
    }
}
