use super::memory::MemoryAddress;

#[derive(Debug, Copy, Clone)]
pub struct Stack {
    data: [MemoryAddress; 16],
    pointer: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            data: [MemoryAddress::ZERO; 16],
            pointer: 0,
        }
    }

    pub fn push(&mut self, addr: MemoryAddress) {
        self.data[self.pointer] = addr;
        self.pointer = (self.pointer + 1) % self.data.len();
    }

    pub fn pop(&mut self) -> MemoryAddress {
        self.pointer = (self.pointer - 1) % self.data.len();
        self.data[self.pointer]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut stack = Stack::new();
        stack.push(MemoryAddress(0x123));
        assert_eq!(stack.data[0], MemoryAddress(0x123));
        assert_eq!(stack.pointer, 1);
    }

    #[test]
    fn test_pop() {
        let mut stack = Stack::new();
        stack.push(MemoryAddress(0x123));
        assert_eq!(stack.pointer, 1);
        assert_eq!(stack.pop(), MemoryAddress(0x123));
        assert_eq!(stack.pointer, 0);
    }
}
