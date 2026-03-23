use crate::cpu::memory::Memory;

// Struct and enums
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub status: u8,
    memory: Memory,
}

enum AddressingMode {
    Immediate,
    ZeroPage,
    Absolute,
}

// Implementation

impl CPU {
    pub fn new() -> Self {
        Self::new_mem(Memory::new())
    }

    pub fn new_mem(memory: Memory) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0,
            stack_pointer: 0,
            status: 0,
            memory,
        }
    }

    pub fn reset(&mut self) {
        let low_byte = self.memory.mem_read(0xFFFC) as u16;
        let high_byte = self.memory.mem_read(0xFFFD) as u16;

        let reset_byte = (high_byte << 8) | low_byte;
        self.program_counter = reset_byte;
        self.status = 0;
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.mem_read(self.program_counter);
        self.program_counter += 1;

        byte
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // zero flag
        if result == 0 {
            self.status |= 1 << 1;
        }
        else {
            self.status &= !(1 << 1);
        }

        // negative flag
        if (result >> 7) & 1 != 0  {
            self.status |= 1 << 7;
        }
        else {
            self.status &= !(1 << 7);
        }
    }

    fn get_operand_address(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                self.fetch_byte() as u16
            },
            AddressingMode::ZeroPage => {
                self.fetch_byte() as u16
            },
            AddressingMode::Absolute => {
                let low_byte = self.fetch_byte() as u16;
                let high_byte = self.fetch_byte() as u16;

                (high_byte << 8) | low_byte
            }
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_byte();

        match opcode {
            // LDA
            0xA9 => {
                let value = self.get_operand_address(AddressingMode::Immediate) as u8;
                self.register_a = value;
                self.update_zero_and_negative_flags(self.register_a);
            },
            0xA5 => {
                let op_address = self.get_operand_address(AddressingMode::ZeroPage);
                self.register_a = self.memory.mem_read(op_address);
                self.update_zero_and_negative_flags(self.register_a);
            },
            0xAD => {
                let op_address = self.get_operand_address(AddressingMode::Absolute);
                self.register_a = self.memory.mem_read(op_address);
                self.update_zero_and_negative_flags(self.register_a);
            },

            // SDA
            0x85 => {
                let op_address = self.get_operand_address(AddressingMode::ZeroPage);
                self.memory.mem_write(op_address, self.register_a);
            },
            0x8D => {
                let op_address = self.get_operand_address(AddressingMode::Absolute);
                self.memory.mem_write(op_address, self.register_a);
            },

            // TAX
            0xAA => {
                self.register_x = self.register_a;
                self.update_zero_and_negative_flags(self.register_x);
            },

            // INX
            0xE8 => {
                self.register_x = if self.register_x == 0xFF { 0 } else {self.register_x + 1 };
                self.update_zero_and_negative_flags(self.register_x);
            },
            _ => panic!("This opcode is not supposed to be used."),
        }
    }
}


// Tests

#[cfg(test)]
mod tests_cpu {
    use super::*;

    fn init_test_memory(mem: &mut Memory) {
        mem.mem_write(0xFFFC, 0x00);
        mem.mem_write(0xFFFD, 0x80);
    }

    #[test]
    fn test_example() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        let mut cpu = CPU::new_mem(memory);
        cpu.reset();
        assert_eq!(cpu.program_counter, 0x8000);
    }

    #[test]
    fn test_lda_immediate_sets_register_a_and_flags() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);
        memory.mem_write(0x8000, 0xA9);
        memory.mem_write(0x8001, 0x42);

        let mut cpu = CPU::new_mem(memory);

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_a, 0x42);

        assert_eq!(cpu.status & (1 << 1), 0);   // Zero flag
        assert_eq!(cpu.status & (1 << 7), 0);   // Negative flag
    }

    #[test]
    fn test_lda_zero_page() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        // LDA $10
        memory.mem_write(0x8000, 0xA5);
        memory.mem_write(0x8001, 0x10);

        memory.mem_write(0x0010, 0x99);

        let mut cpu = CPU::new_mem(memory);

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_a, 0x99);
        assert_eq!(cpu.status & (1 << 1), 0);   // Zero flag
        assert_eq!(cpu.status & (1 << 7), 1 << 7); // Negative flag if bit 7 = 1 (0x99 = 0b10011001)
    }

    #[test]
    fn test_sta_zero_page() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        // LDA $10
        memory.mem_write(0x8000, 0x85);
        memory.mem_write(0x8001, 0x10);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 0x42;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x0010), 0x42);
    }

    #[test]
    fn test_sta_absolute() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x8D); // STA Absolute
        memory.mem_write(0x8001, 0x34); // low byte
        memory.mem_write(0x8002, 0x12); // high byte

        let mut cpu = CPU::new_mem(memory);

        cpu.register_a = 0x99;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x1234), 0x99);
    }

    #[test]
    fn test_sta_pc_increment() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        // LDA $10
        memory.mem_write(0x8000, 0x85);
        memory.mem_write(0x8001, 0x10);

        let mut cpu = CPU::new_mem(memory);

        cpu.register_a = 0x42;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_tax_transfer_a_to_x() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        let mut cpu = CPU::new_mem(memory);

        cpu.memory.mem_write(0x8000, 0xAA); // TAX

        cpu.register_a = 0x42;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn test_tax_flags() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        let mut cpu = CPU::new_mem(memory);

        cpu.memory.mem_write(0x8000, 0xAA); // TAX

        cpu.register_a = 0x00;

        cpu.reset();
        cpu.step();

        assert_ne!(cpu.status & (1 << 1), 0); // Z flag
        assert_eq!(cpu.status & (1 << 7), 0); // N flag

        cpu.register_a = 0x80;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.status & (1 << 1), 0);
        assert_ne!(cpu.status & (1 << 7), 1);
    }

    #[test]
    fn test_tax_pc_increment() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        let mut cpu = CPU::new_mem(memory);

        cpu.memory.mem_write(0x8000, 0xAA);

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8001);
    }

    #[test]
    fn test_inx_increment() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        let mut cpu = CPU::new_mem(memory);

        cpu.memory.mem_write(0x8000, 0xE8); // INX

        cpu.register_x = 0x41;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn test_inx_overflow() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        let mut cpu = CPU::new_mem(memory);

        cpu.memory.mem_write(0x8000, 0xE8); // INX

        cpu.register_x = 0xFF;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_x, 0x00);
    }

    #[test]
    fn test_inx_flags() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        let mut cpu = CPU::new_mem(memory);

        cpu.memory.mem_write(0x8000, 0xE8); // INX

        cpu.register_x = 0xFF;

        cpu.reset();
        cpu.step();

        assert_ne!(cpu.status & (1 << 1), 0); // Z flag
        assert_eq!(cpu.status & (1 << 7), 0); // N flag

        cpu.register_x = 0x7F;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.status & (1 << 1), 0);
        assert_ne!(cpu.status & (1 << 7), 0);
    }
}

