mod memory;
mod registers;
mod stack;

use memory::{Memory, MemoryAddress};
use registers::Registers;
use stack::Stack;
use crate::util::*;
use std::sync::{Arc, Mutex, Condvar};

type Keyboard = Arc<(Mutex<Option<u8>>, Condvar)>;
type Display = [u8; 8*32];

#[derive(Debug)]
pub struct Chip8 {
    pub display: Display,
    stack: Stack,
    memory: Memory,
    registers: Registers,
    pc: MemoryAddress,
    i: MemoryAddress,
    pub keyboard: Keyboard,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            display: [0; 8*32],
            stack: Stack::new(),
            memory: Memory::new(),
            registers: Registers::new(),
            pc: MemoryAddress::PROGRAM_START,
            i: MemoryAddress::ZERO,
            keyboard: Arc::new((Mutex::new(None), Condvar::new())),
        }.initialize_digit_sprites()
    }

    fn initialize_digit_sprites(mut self) -> Self {
        self.memory.write_bytes(MemoryAddress(0), &[0xF0, 0x90, 0x90, 0x90, 0xF0]);  // 0
        self.memory.write_bytes(MemoryAddress(5), &[0x20, 0x60, 0x20, 0x20, 0x70]);  // 1
        self.memory.write_bytes(MemoryAddress(10), &[0xF0, 0x10, 0xF0, 0x80, 0xF0]); // 2
        self.memory.write_bytes(MemoryAddress(15), &[0xF0, 0x10, 0xF0, 0x10, 0xF0]); // 3
        self.memory.write_bytes(MemoryAddress(20), &[0x90, 0x90, 0xF0, 0x10, 0x10]); // 4
        self.memory.write_bytes(MemoryAddress(25), &[0xF0, 0x80, 0xF0, 0x10, 0xF0]); // 5
        self.memory.write_bytes(MemoryAddress(30), &[0xF0, 0x80, 0xF0, 0x90, 0xF0]); // 6
        self.memory.write_bytes(MemoryAddress(35), &[0xF0, 0x10, 0x20, 0x40, 0x40]); // 7
        self.memory.write_bytes(MemoryAddress(40), &[0xF0, 0x90, 0xF0, 0x90, 0xF0]); // 8
        self.memory.write_bytes(MemoryAddress(45), &[0xF0, 0x90, 0xF0, 0x10, 0xF0]); // 9
        self.memory.write_bytes(MemoryAddress(50), &[0xF0, 0x90, 0xF0, 0x90, 0x90]); // A
        self.memory.write_bytes(MemoryAddress(55), &[0xE0, 0x90, 0xE0, 0x90, 0xE0]); // B
        self.memory.write_bytes(MemoryAddress(60), &[0xF0, 0x80, 0x80, 0x80, 0xF0]); // C
        self.memory.write_bytes(MemoryAddress(65), &[0xE0, 0x90, 0x90, 0x90, 0xE0]); // D
        self.memory.write_bytes(MemoryAddress(70), &[0xF0, 0x80, 0xF0, 0x80, 0xF0]); // E
        self.memory.write_bytes(MemoryAddress(75), &[0xF0, 0x80, 0xF0, 0x80, 0x80]); // F
        self
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.memory.write_bytes(MemoryAddress::PROGRAM_START, program);
    }

    fn sys(&mut self, _addr: MemoryAddress) {
        println!("sys op is unimplemented");
    }

    fn clear_screen(&mut self) {
        self.display = [0; 8*32];
    }

    fn ret(&mut self) {
        self.pc = self.stack.pop();
    }

    fn jump(&mut self, addr: MemoryAddress) {
        self.pc = addr;
    }

    fn call(&mut self, addr: MemoryAddress) {
        self.stack.push(self.pc);
        self.pc = addr;
    }

    fn skip_next_eq(&mut self, vx: u8, value: u8) {
        if self.registers.cmp_scalar(vx, value) {
            self.pc.next_instruction();
        }
    }

    fn skip_next_ne(&mut self, vx: u8, value: u8) {
        if !self.registers.cmp_scalar(vx, value) {
            self.pc.next_instruction();
        }
    }

    fn skip_next_eq_reg(&mut self, vx: u8, vy: u8) {
        if self.registers.cmp_register(vx, vy) {
            self.pc.next_instruction();
        }
    }

    fn skip_next_ne_reg(&mut self, vx: u8, vy: u8) {
        if !self.registers.cmp_register(vx, vy) {
            self.pc.next_instruction();
        }
    }

    fn load_i(&mut self, addr: MemoryAddress) {
        self.i = addr;
    }

    fn jump_reg0(&mut self, addr: MemoryAddress) {
        self.jump(addr + self.registers[Registers::V0]);
    }

    fn rand_and(&mut self, vx: u8, value: u8) {
        self.registers[vx] = value & rand::random::<u8>();
    }

    fn draw_sprite(&mut self, arg1: u8, arg2: u8, arg3: u8) {
        let x = self.registers[arg1] as usize;
        let y = self.registers[arg2] as usize;
        let n = low_nibble(arg3) as usize;
        let sprite_data = self.memory.read_bytes(self.i, n);
        for (i, source) in sprite_data.iter().enumerate() {
            let offset = x % 8;
            if offset == 0 {
                let target = &mut self.display[8*(y+i) + x/8];
                let ones_before_blit = source.count_ones() + target.count_ones();
                *target ^= source;
                let ones_after_blit = target.count_ones();
                self.registers[Registers::VF] = if ones_after_blit < ones_before_blit { 1 } else { 0 };
            } else {
                let target_a = &mut self.display[8*(y+i) + x/8];
                let source_a = source >> offset;
                let mut ones_before_blit = source_a.count_ones() + target_a.count_ones();
                *target_a ^= source_a;
                let mut ones_after_blit = target_a.count_ones();
                let target_b = &mut self.display[8*(y+i) + x/8 + 1];
                let source_b = source << (8 - offset);
                ones_before_blit += source_b.count_ones() + target_b.count_ones();
                *target_b ^= source_b;
                ones_after_blit += target_b.count_ones();
                self.registers[Registers::VF] = if ones_after_blit < ones_before_blit { 1 } else { 0 };
            }
        }
    }

    fn skip_input(&mut self, arg1: u8) {
        let (lock, _cvar) = &*self.keyboard;
        let keyboard = lock.lock().unwrap();
        if let Some(key_pressed) = *keyboard {
            if key_pressed == arg1 {
                self.pc.next_instruction();
            }
        }
    }

    fn skip_not_input(&mut self, arg1: u8) {
        let (lock, _cvar) = &*self.keyboard;
        let keyboard = lock.lock().unwrap();
        if let Some(key_pressed) = *keyboard {
            if key_pressed != arg1 {
                self.pc.next_instruction();
            }
        }
    }

    fn load_input(&mut self, vx: u8) {
        let (lock, cvar) = &*self.keyboard;
        let mut keyboard = lock.lock().unwrap();
        while (*keyboard).is_none() {
            keyboard = cvar.wait(keyboard).unwrap();
        }
        if let Some(key_pressed) = *keyboard {
            self.registers[vx] = key_pressed;
        }
    }

    fn add_i_reg(&mut self, vx: u8) {
        self.i += self.registers[vx];
    }

    fn load_digit_sprite(&mut self, arg1: u8) {
        self.i = MemoryAddress((self.registers[arg1] as u16) * 5);
    }

    fn load_binary_coded_decimal(&mut self, vx: u8) {
        let mut value = self.registers[vx];
        let hundreds = value / 100; value %= 100;
        let tens = value / 10; value %= 10;
        let ones = value;
        self.memory.write_bytes(self.i, &[hundreds, tens, ones]);
    }

    fn store_regs(&mut self, vy: u8) {
        self.memory.write_bytes(self.i, self.registers.get_slice(Registers::V0, vy));
    }

    fn load_regs(&mut self, vy: u8) {
        let n = vy as usize + 1;
        let data = self.memory.read_bytes(self.i, n);
        self.registers.get_slice_mut(Registers::V0, vy).copy_from_slice(data);
    }

    pub fn step(&mut self) {
        if let &[jj, kk] = self.memory.read_bytes(self.pc, 2) {
            let op = high_nibble(jj);
            let x = low_nibble(jj);
            let y = high_nibble(kk);
            let subop = low_nibble(kk);
            let nnn: u16 = ((jj as u16) << 8) | (kk as u16);
            match op {
                0x0 => match nnn {
                    0x0E0 => self.clear_screen(),
                    0x0EE => self.ret(),
                    _ => self.sys(MemoryAddress(nnn)),
                },
                0x1 => self.jump(MemoryAddress(nnn)),
                0x2 => self.call(MemoryAddress(nnn)),
                0x3 => self.skip_next_eq(x, kk),
                0x4 => self.skip_next_ne(x, kk),
                0x5 if subop == 0x0 => self.skip_next_eq_reg(x, y),
                0x6 => self.registers.load_scalar(x, kk),
                0x7 => self.registers.add_scalar(x, kk),
                0x8 => match subop {
                    0x0 => self.registers.load_register(x, y),
                    0x1 => self.registers.or_register(x, y),
                    0x2 => self.registers.and_register(x, y),
                    0x3 => self.registers.xor_register(x, y),
                    0x4 => self.registers.add_register(x, y),
                    0x5 => self.registers.sub_register(x, y),
                    0x6 => self.registers.shift_right(x),
                    0x7 => self.registers.subn_register(x, y),
                    0xE => self.registers.shift_left(x),
                    _ => println!("Invalid instruction PC: {:?} I: {:?} STACK: {:?} OP: {:?} X: {:?} Y: {:?} SUBOP: {:?}", self.pc, self.i, self.stack, op, x, y, subop),
                },
                0x9 if subop == 0x0 => self.skip_next_ne_reg(x, y),
                0xA => self.load_i(MemoryAddress(nnn)),
                0xB => self.jump_reg0(MemoryAddress(nnn)),
                0xC => self.rand_and(x, kk),
                0xD => self.draw_sprite(x, y, subop),
                0xE => match kk {
                    0x9E => self.skip_input(x),
                    0xA1 => self.skip_not_input(x),
                    _ => println!("Invalid instruction PC: {:?} I: {:?} STACK: {:?} OP: {:?} X: {:?} Y: {:?} SUBOP: {:?}", self.pc, self.i, self.stack, op, x, y, subop),
                },
                0xF => match kk {
                    0x07 => self.registers.load_register(x, Registers::DT),
                    0x0A => self.load_input(x),
                    0x15 => self.registers.load_register(Registers::DT, x),
                    0x18 => self.registers.load_register(Registers::ST, x),
                    0x1E => self.add_i_reg(x),
                    0x29 => self.load_digit_sprite(x),
                    0x33 => self.load_binary_coded_decimal(x),
                    0x55 => self.store_regs(x),
                    0x65 => self.load_regs(x),
                    _ => println!("Invalid instruction PC: {:?} I: {:?} STACK: {:?} OP: {:?} X: {:?} Y: {:?} SUBOP: {:?}", self.pc, self.i, self.stack, op, x, y, subop),
                },
                _ => println!("Invalid instruction PC: {:?} I: {:?} STACK: {:?} OP: {:?} X: {:?} Y: {:?} SUBOP: {:?}", self.pc, self.i, self.stack, op, x, y, subop),
            }
            self.pc.next_instruction();
        } else { println!("Unable to read instruction!") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump() {
        let mut chip8 = Chip8::new();
        chip8.jump(MemoryAddress(0x123));
        assert_eq!(chip8.pc, MemoryAddress(0x123));
    }

    #[test]
    fn test_ret() {
        let mut chip8 = Chip8::new();
        let pc_before_call = chip8.pc;
        chip8.call(MemoryAddress(0x123));
        chip8.ret();
        //assert_eq!(chip8.stack.data[0], MemoryAddress::PROGRAM_START);
        //assert_eq!(chip8.stack.pointer, 0);
        assert_eq!(chip8.pc, pc_before_call);
    }

    #[test]
    fn test_call() {
        let mut chip8 = Chip8::new();
        let pc_before_call = chip8.pc;
        chip8.call(MemoryAddress(0x123));
        //assert_eq!(chip8.stack.pointer, 1);
        assert_eq!(chip8.stack.pop(), pc_before_call);
        assert_eq!(chip8.pc, MemoryAddress(0x123));
    }

    #[test]
    fn test_skip_next_eq() {
        let mut chip8 = Chip8::new();
        let pc = chip8.pc;
        chip8.skip_next_eq(0, 0);
        assert_eq!(chip8.pc, pc + 2);
        chip8.skip_next_eq(0, 1);
        assert_eq!(chip8.pc, pc + 2);
    }

    #[test]
    fn test_skip_next_ne() {
        let mut chip8 = Chip8::new();
        let pc = chip8.pc;
        chip8.skip_next_ne(0, 1);
        assert_eq!(chip8.pc, pc + 2);
        chip8.skip_next_ne(0, 0);
        assert_eq!(chip8.pc, pc + 2);
    }

    #[test]
    fn test_skip_next_eq_reg() {
        let mut chip8 = Chip8::new();
        let pc = chip8.pc;
        chip8.skip_next_eq_reg(0, 1);
        assert_eq!(chip8.pc, pc + 2);
        chip8.registers[Registers::V1] = 1;
        chip8.skip_next_eq_reg(0, 1);
        assert_eq!(chip8.pc, pc + 2);
    }

    #[test]
    fn test_skip_next_ne_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[Registers::V0] = 0;
        chip8.registers[Registers::V1] = 1;
        let pc = chip8.pc;
        chip8.skip_next_ne_reg(0, 1);
        assert_eq!(chip8.pc, pc + 2);

        chip8.registers[Registers::V0] = 0;
        chip8.registers[Registers::V1] = 0;
        let pc = chip8.pc;
        chip8.skip_next_ne_reg(0, 1);
        assert_eq!(chip8.pc, pc);
    }

    #[test]
    fn test_load_i() {
        let mut chip8 = Chip8::new();
        chip8.load_i(MemoryAddress(0x123));
        assert_eq!(chip8.i, MemoryAddress(0x123));
    }

    #[test]
    fn test_jump_reg0() {
        let mut chip8 = Chip8::new();
        chip8.registers[Registers::V0] = 0x20;
        chip8.jump_reg0(MemoryAddress(0x30));
        assert_eq!(chip8.pc, MemoryAddress(0x50));
    }

    // #TODO: this can fail randomly. The default ThreadRng isn't seedable so need to rework usage
    // of rand for this function.
    #[test]
    fn test_rand_and() {
        let mut chip8 = Chip8::new();
        chip8.rand_and(0, 0x23);
        let a = chip8.registers[Registers::V0];
        chip8.rand_and(0, 0x23);
        let b = chip8.registers[Registers::V0];
        assert_ne!(a, b);
    }

    #[test]
    fn test_add_i_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[Registers::V0] = 0x23;
        chip8.add_i_reg(0);
        assert_eq!(chip8.i, MemoryAddress(0x23));
    }

    #[test]
    fn test_load_binary_coded_decimal() {
        let mut chip8 = Chip8::new();
        chip8.registers[Registers::V0] = 123;
        chip8.load_binary_coded_decimal(0);
        assert_eq!(chip8.memory.read_bytes(chip8.i, 3), &[1, 2, 3]);
    }

    #[test]
    fn test_store_regs() {
        let mut chip8 = Chip8::new();
        chip8.registers[Registers::V0] = 1;
        chip8.registers[Registers::V1] = 2;
        chip8.registers[Registers::V2] = 3;
        chip8.store_regs(2);
        assert_eq!(chip8.memory.read_bytes(chip8.i, 3), &[1, 2, 3]);
    }

    #[test]
    fn test_load_regs() {
        let mut chip8 = Chip8::new();
        chip8.memory.write_bytes(chip8.i, &[1, 2, 3]);
        chip8.load_regs(2);
        assert_eq!(chip8.registers[Registers::V0], 1);
        assert_eq!(chip8.registers[Registers::V1], 2);
        assert_eq!(chip8.registers[Registers::V2], 3);
    }
}
