use std::sync::{Arc, Mutex, Condvar};

const V0: usize = 0x0;
const V1: usize = 0x1;
const V2: usize = 0x2;
const V3: usize = 0x3;
const V4: usize = 0x4;
const V5: usize = 0x5;
const V6: usize = 0x6;
const V7: usize = 0x7;
const V8: usize = 0x8;
const V9: usize = 0x9;
const VA: usize = 0xA;
const VB: usize = 0xB;
const VC: usize = 0xC;
const VD: usize = 0xD;
const VE: usize = 0xE;
const VF: usize = 0xF;

pub struct Chip8 {
    pub display: [u8; 8*32],
    stack: [u16; 16],
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pc: u16, // program counter
    sp: u8, // stack pointer
    i: u16, // instruction pointer
    dt: u8, // delay timer
    st: u8, // sound timer
    pub keyboard: Arc<(Mutex<Option<u8>>, Condvar)>,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            display: [0; 8*32],
            stack: [0; 16],
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0x200,
            sp: 0,
            i: 0,
            dt: 0,
            st: 0,
            keyboard: Arc::new((Mutex::new(None), Condvar::new())),
        }.initialize_digit_sprites()
    }

    fn initialize_digit_sprites(mut self) -> Self {
        self.memory[0..5].copy_from_slice(&[0xF0, 0x90, 0x90, 0x90, 0xF0]);   // 0
        self.memory[5..10].copy_from_slice(&[0x20, 0x60, 0x20, 0x20, 0x70]);  // 1
        self.memory[10..15].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x80, 0xF0]); // 2
        self.memory[15..20].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x10, 0xF0]); // 3
        self.memory[20..25].copy_from_slice(&[0x90, 0x90, 0xF0, 0x10, 0x10]); // 4
        self.memory[25..30].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x10, 0xF0]); // 5
        self.memory[30..35].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x90, 0xF0]); // 6
        self.memory[35..40].copy_from_slice(&[0xF0, 0x10, 0x20, 0x40, 0x40]); // 7
        self.memory[40..45].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0xF0]); // 8
        self.memory[45..50].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x10, 0xF0]); // 9
        self.memory[50..55].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0x90]); // A
        self.memory[55..60].copy_from_slice(&[0xE0, 0x90, 0xE0, 0x90, 0xE0]); // B
        self.memory[60..65].copy_from_slice(&[0xF0, 0x80, 0x80, 0x80, 0xF0]); // C
        self.memory[65..70].copy_from_slice(&[0xE0, 0x90, 0x90, 0x90, 0xE0]); // D
        self.memory[70..75].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0xF0]); // E
        self.memory[75..80].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0x80]); // F
        self
    }

    fn sys(&mut self, _arg1: u16) { unimplemented!(); }

    fn clear_screen(&mut self) {
        self.display = [0; 8*32];
    }

    fn ret(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp = self.sp.saturating_sub(1);
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr & 0x0FFF;
    }

    fn call(&mut self, addr: u16) {
        self.sp = self.sp.saturating_add(1);
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr & 0x0FFF;
    }

    fn skip_next_eq(&mut self, arg1: u8, arg2: u8) {
        if self.registers[arg1 as usize] == arg2 {
            self.pc = self.pc.saturating_add(2);
        }
    }

    fn skip_next_ne(&mut self, arg1: u8, arg2: u8) {
        if self.registers[arg1 as usize] != arg2 {
            self.pc = self.pc.saturating_add(2);
        }
    }

    fn skip_next_eq_reg(&mut self, arg1: u8, arg2: u8) {
        if self.registers[arg1 as usize] == self.registers[arg2 as usize] {
            self.pc = self.pc.saturating_add(2);
        }
    }

    fn load_scalar(&mut self, arg1: u8, arg2: u8) {
        self.registers[arg1 as usize] = arg2;
    }

    // Q: should this wrap, saturate, panic, what? I think wrap but am not certain
    fn add_scalar(&mut self, arg1: u8, arg2: u8) {
        self.registers[arg1 as usize] = self.registers[arg1 as usize].wrapping_add(arg2);
    }

    fn load_reg(&mut self, arg1: u8, arg2: u8) {
        self.registers[arg1 as usize] = self.registers[arg2 as usize];
    }

    fn or_reg(&mut self, arg1: u8, arg2: u8) {
        self.registers[arg1 as usize] |= self.registers[arg2 as usize];
    }

    fn and_reg(&mut self, arg1: u8, arg2: u8) {
        self.registers[arg1 as usize] &= self.registers[arg2 as usize];
    }

    fn xor_reg(&mut self, arg1: u8, arg2: u8) {
        self.registers[arg1 as usize] ^= self.registers[arg2 as usize];
    }

    fn add_reg(&mut self, arg1: u8, arg2: u8) {
        let (result, overflow) = self.registers[arg1 as usize].overflowing_add(self.registers[arg2 as usize]);
        self.registers[arg1 as usize] = result;
        self.registers[VF] = if overflow { 1 } else { 0 };
    }

    fn sub_reg(&mut self, arg1: u8, arg2: u8) {
        self.registers[VF] = if self.registers[arg1 as usize] > self.registers[arg2 as usize] { 1 } else { 0 };
        let (result, _overflow) = self.registers[arg1 as usize].overflowing_sub(self.registers[arg2 as usize]);
        self.registers[arg1 as usize] = result;
    }

    fn shift_right(&mut self, arg1: u8) {
        self.registers[VF] = if self.registers[arg1 as usize] & 1 == 1 { 1 } else { 0 };
        self.registers[arg1 as usize] >>= 1;
    }

    fn subn_reg(&mut self, arg1: u8, arg2: u8) {
        self.registers[VF] = if self.registers[arg2 as usize] > self.registers[arg1 as usize] { 1 } else { 0 };
        let (result, _overflow) = self.registers[arg2 as usize].overflowing_sub(self.registers[arg1 as usize]);
        self.registers[arg1 as usize] = result;
    }

    fn shift_left(&mut self, arg1: u8) {
        self.registers[VF] = if self.registers[arg1 as usize] & 0x80 == 0x80 { 1 } else { 0 };
        self.registers[arg1 as usize] <<= 1;
    }

    fn skip_ne_reg(&mut self, arg1: u8, arg2: u8) {
        if self.registers[arg1 as usize] != self.registers[arg2 as usize] {
            self.pc = self.pc.saturating_add(2);
        }
    }

    fn load_i(&mut self, arg1: u16) {
        self.i = arg1 & 0x0FFF;
    }

    fn jump_reg0(&mut self, arg1: u16) {
        self.pc = self.pc.saturating_add((arg1 & 0x0FFF) + (self.registers[V0] as u16));
    }

    fn rand_and(&mut self, arg1: u8, arg2: u8) {
        self.registers[arg1 as usize] = arg2 & rand::random::<u8>();
    }

    fn draw_sprite(&mut self, arg1: u8, arg2: u8, arg3: u8) {
        let x = self.registers[arg1 as usize] as usize;
        let y = self.registers[arg2 as usize] as usize;
        let n = (arg3 & 0x0F) as usize;
        let i = self.i as usize;
        let sprite_data = &self.memory[i..(i+n)];
        for i in 0..n {
            let source = sprite_data[i];
            let offset = x % 8;
            if offset == 0 {
                let target = &mut self.display[8*(y+i) + x/8];
                let ones_before_blit = source.count_ones() + target.count_ones();
                *target ^= source;
                let ones_after_blit = target.count_ones();
                self.registers[VF] = if ones_after_blit < ones_before_blit { 1 } else { 0 };
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
                self.registers[VF] = if ones_after_blit < ones_before_blit { 1 } else { 0 };
            }
        }
    }

    fn skip_input(&mut self, arg1: u8) {
        let (lock, _cvar) = &*self.keyboard;
        let keyboard = lock.lock().unwrap();
        if let Some(key_pressed) = *keyboard {
            if key_pressed == arg1 {
                self.pc = self.pc.saturating_add(2);
            }
        }
    }

    fn skip_not_input(&mut self, arg1: u8) {
        let (lock, _cvar) = &*self.keyboard;
        let keyboard = lock.lock().unwrap();
        if let Some(key_pressed) = *keyboard {
            if key_pressed != arg1 {
                self.pc = self.pc.saturating_add(2);
            }
        }
    }

    fn load_reg_from_delay_timer(&mut self, arg1: u8) {
        self.registers[arg1 as usize] = self.dt;
    }

    fn load_input(&mut self, arg1: u8) {
        let (lock, cvar) = &*self.keyboard;
        let mut keyboard = lock.lock().unwrap();
        while (*keyboard).is_none() {
            keyboard = cvar.wait(keyboard).unwrap();
        }
        if let Some(key_pressed) = *keyboard {
            self.registers[arg1 as usize] = key_pressed;
        }
    }

    fn load_delay_timer_from_reg(&mut self, arg1: u8) {
        self.dt = self.registers[arg1 as usize];
    }

    fn load_sound_timer_from_reg(&mut self, arg1: u8) {
        self.st = self.registers[arg1 as usize];
    }

    fn add_i_reg(&mut self, arg1: u8) {
        let (result, _overflow) = self.i.overflowing_add(self.registers[arg1 as usize] as u16);
        self.i = result;
    }

    fn load_digit_sprite(&mut self, arg1: u8) {
        self.i = (self.registers[arg1 as usize] as u16) * 5;
    }

    fn load_binary_coded_decimal(&mut self, arg1: u8) {
        let mut value = self.registers[arg1 as usize];
        let i = self.i as usize;
        self.memory[i] = value / 100; value %= 100;
        self.memory[i+1] = value / 10; value %= 10;
        self.memory[i+2] = value;
    }

    fn store_regs(&mut self, arg1: u8) {
        for x in 0usize..=arg1.into() {
            self.memory[self.i as usize + x] = self.registers[x];
        }
    }

    fn load_regs(&mut self, arg1: u8) {
        for x in 0usize..=arg1.into() {
            self.registers[x] = self.memory[self.i as usize + x]
        }
    }

    pub fn step(&mut self) {
        if let [jj, kk] = self.memory[(self.pc as usize)..((self.pc + 2) as usize)] {
            let op = (jj & 0xF0) >> 4;
            let x = jj & 0x0F;
            let y = (kk & 0xF0) >> 4;
            let subop = kk & 0x0F;
            let nnn: u16 = ((jj as u16) << 8) | (kk as u16);
            match op {
                0x0 => match nnn {
                    0x0E0 => self.clear_screen(),
                    0x0EE => self.ret(),
                    _ => self.sys(nnn),
                },
                0x1 => self.jump(nnn),
                0x2 => self.call(nnn),
                0x3 => self.skip_next_eq(x, kk),
                0x4 => self.skip_next_ne(x, kk),
                0x5 if subop == 0x0 => self.skip_next_eq_reg(x, y),
                0x6 => self.load_scalar(x, kk),
                0x7 => self.add_scalar(x, kk),
                0x8 => match subop {
                    0x0 => self.load_reg(x, y),
                    0x1 => self.or_reg(x, y),
                    0x2 => self.and_reg(x, y),
                    0x3 => self.xor_reg(x, y),
                    0x4 => self.add_reg(x, y),
                    0x5 => self.sub_reg(x, y),
                    0x6 => self.shift_right(x),
                    0x7 => self.subn_reg(x, y),
                    0xE => self.shift_left(x),
                    _ => panic!("Invalid instruction"),
                },
                0x9 if subop == 0x0 => self.skip_next_eq_reg(x, y),
                0xA => self.load_i(nnn),
                0xB => self.jump_reg0(nnn),
                0xC => self.rand_and(x, kk),
                0xD => self.draw_sprite(x, y, subop),
                0xE => match kk {
                    0x9E => self.skip_input(x),
                    0xA1 => self.skip_not_input(x),
                    _ => panic!("Invalid instruction"),
                },
                0xF => match kk {
                    0x07 => self.load_reg_from_delay_timer(x),
                    0x0A => self.load_input(x),
                    0x15 => self.load_delay_timer_from_reg(x),
                    0x18 => self.load_sound_timer_from_reg(x),
                    0x1E => self.add_i_reg(x),
                    0x29 => self.load_digit_sprite(x),
                    0x33 => self.load_binary_coded_decimal(x),
                    0x55 => self.store_regs(x),
                    0x65 => self.load_regs(x),
                    _ => panic!("Invalid instruction"),
                },
                _ => panic!("Invalid instruction"),
            }
            self.pc = self.pc.saturating_add(2);
        } else { panic!("Could not read instruction"); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump() {
        let mut chip8 = Chip8::new();
        chip8.jump(0x123);
        assert_eq!(chip8.pc, 0x123);
    }

    #[test]
    fn test_ret() {
        let mut chip8 = Chip8::new();
        let pc_before_call = chip8.pc;
        chip8.call(0x123);
        chip8.ret();
        assert_eq!(chip8.stack[0], 0);
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.pc, pc_before_call);
    }

    #[test]
    fn test_call() {
        let mut chip8 = Chip8::new();
        let pc_before_call = chip8.pc;
        chip8.call(0x123);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.stack[chip8.sp as usize], pc_before_call);
        assert_eq!(chip8.pc, 0x123);
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
        chip8.registers[1] = 1;
        chip8.skip_next_eq_reg(0, 1);
        assert_eq!(chip8.pc, pc + 2);
    }

    #[test]
    fn test_load_scalar() {
        let mut chip8 = Chip8::new();
        chip8.load_scalar(0, 123);
        assert_eq!(chip8.registers[0], 123);
    }

    #[test]
    fn test_add_scalar() {
        let mut chip8 = Chip8::new();
        chip8.add_scalar(0, 123);
        assert_eq!(chip8.registers[0], 123);
        chip8.add_scalar(0, 255);
        assert_eq!(chip8.registers[0], 122);
    }

    #[test]
    fn test_load_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 123;
        chip8.load_reg(0, 1);
        assert_eq!(chip8.registers[0], 123);
        assert_eq!(chip8.registers[1], 123);
    }

    #[test]
    fn test_or_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0xF0;
        chip8.registers[1] = 0x8F;
        chip8.or_reg(0, 1);
        assert_eq!(chip8.registers[0], 0xFF);
        assert_eq!(chip8.registers[1], 0x8F);
    }

    #[test]
    fn test_and_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0x3F;
        chip8.registers[1] = 0xFC;
        chip8.and_reg(0, 1);
        assert_eq!(chip8.registers[0], 0x3C);
        assert_eq!(chip8.registers[1], 0xFC);
    }

    #[test]
    fn test_xor_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 0xF0;
        chip8.registers[1] = 0x0F;
        chip8.xor_reg(0, 1);
        assert_eq!(chip8.registers[0], 0xFF);
        assert_eq!(chip8.registers[1], 0x0F);
    }

    #[test]
    fn test_add_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[1] = 123;
        chip8.add_reg(0, 1);
        assert_eq!(chip8.registers[0], 123);
        assert_eq!(chip8.registers[VF], 0);
        chip8.registers[2] = 255;
        chip8.add_reg(0, 2);
        assert_eq!(chip8.registers[0], 122);
        assert_eq!(chip8.registers[VF], 1);
    }

    #[test]
    fn test_sub_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 223;
        chip8.registers[1] = 123;
        chip8.sub_reg(0, 1);
        assert_eq!(chip8.registers[0], 100);
        assert_eq!(chip8.registers[1], 123);
        assert_eq!(chip8.registers[VF], 1);
        chip8.sub_reg(0, 1);
        assert_eq!(chip8.registers[0], 233);
        assert_eq!(chip8.registers[1], 123);
        assert_eq!(chip8.registers[VF], 0);
    }

    #[test]
    fn test_shift_right() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 10;
        chip8.shift_right(0);
        assert_eq!(chip8.registers[0], 5);
        assert_eq!(chip8.registers[VF], 0);
        chip8.shift_right(0);
        assert_eq!(chip8.registers[0], 2);
        assert_eq!(chip8.registers[VF], 1);
    }

    #[test]
    fn test_subn_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 50;
        chip8.registers[1] = 200;
        chip8.subn_reg(0, 1);
        assert_eq!(chip8.registers[0], 150);
        assert_eq!(chip8.registers[1], 200);
        assert_eq!(chip8.registers[VF], 1);

        chip8.registers[0] = 200;
        chip8.registers[1] = 50;
        chip8.subn_reg(0, 1);
        assert_eq!(chip8.registers[0], 106);
        assert_eq!(chip8.registers[1], 50);
        assert_eq!(chip8.registers[VF], 0);
    }

    #[test]
    fn test_shift_left() {
        let mut chip8 = Chip8::new();
        chip8.registers[V0] = 1;
        chip8.shift_left(0);
        assert_eq!(chip8.registers[V0], 2);
        assert_eq!(chip8.registers[VF], 0);

        chip8.registers[V0] = 0x82;
        chip8.shift_left(0);
        assert_eq!(chip8.registers[V0], 4);
        assert_eq!(chip8.registers[VF], 1);
    }

    #[test]
    fn test_skip_ne_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[V0] = 0;
        chip8.registers[V1] = 1;
        let pc = chip8.pc;
        chip8.skip_ne_reg(0, 1);
        assert_eq!(chip8.pc, pc + 2);

        chip8.registers[V0] = 0;
        chip8.registers[V1] = 0;
        let pc = chip8.pc;
        chip8.skip_ne_reg(0, 1);
        assert_eq!(chip8.pc, pc);
    }

    #[test]
    fn test_load_i() {
        let mut chip8 = Chip8::new();
        chip8.load_i(0x123);
        assert_eq!(chip8.i, 0x123);
    }

    #[test]
    fn test_jump_reg0() {
        let mut chip8 = Chip8::new();
        let pc = chip8.pc;
        chip8.registers[V0] = 0x23;
        chip8.jump_reg0(0x100);
        assert_eq!(chip8.pc, pc + 0x123);
    }

    // #TODO: this can fail randomly. The default ThreadRng isn't seedable so need to rework usage
    // of rand for this function.
    #[test]
    fn test_rand_and() {
        let mut chip8 = Chip8::new();
        chip8.rand_and(0, 0x23);
        let a = chip8.registers[V0];
        chip8.rand_and(0, 0x23);
        let b = chip8.registers[V0];
        assert_ne!(a, b);
    }

//    fn draw_sprite() { todo!() }
//    fn skip_input() { todo!() }
//    fn skip_not_input() { todo!() }
//
    #[test]
    fn test_load_reg_from_delay_timer() {
        let mut chip8 = Chip8::new();
        chip8.dt = 0x23;
        chip8.load_reg_from_delay_timer(0);
        assert_eq!(chip8.registers[V0], 0x23);
    }

//    fn load_input() { todo!() }

    #[test]
    fn test_load_delay_timer_from_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[V0] = 0x23;
        chip8.load_delay_timer_from_reg(0);
        assert_eq!(chip8.dt, 0x23);
    }

    #[test]
    fn test_load_sound_timer_from_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[V0] = 0x23;
        chip8.load_sound_timer_from_reg(0);
        assert_eq!(chip8.st, 0x23);
    }

    #[test]
    fn test_add_i_reg() {
        let mut chip8 = Chip8::new();
        chip8.registers[V0] = 0x23;
        chip8.add_i_reg(0);
        assert_eq!(chip8.i, 0x23);
    }

//    fn load_sprite_location() { todo!() }

    #[test]
    fn test_load_binary_coded_decimal() {
        let mut chip8 = Chip8::new();
        chip8.registers[V0] = 123;
        chip8.load_binary_coded_decimal(0);
        let i = chip8.i as usize;
        assert_eq!(&chip8.memory[i..=(i+2)], &[1, 2, 3]);
    }

    #[test]
    fn test_store_regs() {
        let mut chip8 = Chip8::new();
        chip8.registers[V0] = 1;
        chip8.registers[V1] = 2;
        chip8.registers[V2] = 3;
        let i = chip8.i as usize;
        chip8.store_regs(2);
        assert_eq!(&chip8.memory[i..=(i+2)], &[1, 2, 3]);
    }

    #[test]
    fn test_load_regs() {
        let mut chip8 = Chip8::new();
        let i = chip8.i as usize;
        chip8.memory[i] = 1;
        chip8.memory[i+1] = 2;
        chip8.memory[i+2] = 3;
        chip8.load_regs(2);
        assert_eq!(chip8.registers[V0], 1);
        assert_eq!(chip8.registers[V1], 2);
        assert_eq!(chip8.registers[V2], 3);
    }
}
