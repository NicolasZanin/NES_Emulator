use crate::cpu::memory::Memory;

pub struct Stack {
    stack_pointer: u8,
}

pub const STACK_BASE: u16 = 0x0100;

impl Stack {
    pub fn new() -> Stack {
        Stack { stack_pointer: 0xFF }
    }

    pub fn get(&self) -> u8 {
        self.stack_pointer
    }

    pub fn set(&mut self, value: u8) {
        self.stack_pointer = value;
    }

    fn current_address(&self) -> u16 {
        STACK_BASE + (self.stack_pointer as u16)
    }

    pub fn write_value(&mut self, memory: &mut Memory, value: u8) {
        memory.mem_write(self.current_address(), value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn read_value(&mut self, memory: &mut Memory) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);

        memory.mem_read(self.current_address())
    }

    pub fn write_address(&mut self, memory: &mut Memory, addr: u16) {
        self.write_value(memory, (addr >> 8) as u8 );
        self.write_value(memory, addr as u8);
    }

    pub fn read_address(&mut self, memory: &mut Memory) -> u16 {
        let low_byte = self.read_value(memory) as u16;
        let high_byte = self.read_value(memory) as u16;

        (high_byte << 8) | low_byte
    }
}

#[cfg(test)]
impl Stack {
    pub fn new_sp(value: u8) -> Stack {
        Stack { stack_pointer: value }
    }
}