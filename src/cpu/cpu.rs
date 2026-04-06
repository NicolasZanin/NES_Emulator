use crate::cpu::memory::Memory;
use crate::cpu::flags::Flag;
use crate::cpu::flags::Flags;

// Struct and enums
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    status: Flags,
    memory: Memory,
}

#[derive(Copy, Clone)]
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
            status: Flags::new(),
            memory,
        }
    }

    pub fn reset(&mut self) {
        let low_byte = self.memory.mem_read(0xFFFC) as u16;
        let high_byte = self.memory.mem_read(0xFFFD) as u16;

        let reset_byte = (high_byte << 8) | low_byte;
        self.program_counter = reset_byte;
        self.status.reset();
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.mem_read(self.program_counter);
        self.program_counter += 1;

        byte
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

    fn get_operand_value(&mut self, mode: AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);

        match mode {
            AddressingMode::Immediate => addr as u8,
            _ => self.memory.mem_read(addr)
        }
    }

    // Instructions
    fn lda(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.register_a = value;
        self.status.update_zero_and_negative_flags(self.register_a);
    }

    fn sda(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.memory.mem_write(address, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.status.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = if self.register_x == 0xFF { 0 } else {self.register_x + 1 };
        self.status.update_zero_and_negative_flags(self.register_x);
    }

    fn adc(&mut self) {
        let a = self.register_a;
        let value = self.fetch_byte();
        let carry = if self.status.get_flag(Flag::CARRY) { 1u16 } else { 0u16 };

        let sum = a as u16 + value as u16 + carry;
        self.status.update_carry_flags(sum);


        let result = sum as u8;
        self.register_a = result;

        self.status.update_zero_and_negative_flags(result);

        let overflow = (a ^ result) & (value ^ result) & 0x80;
        self.status.update_overflow_flags(overflow != 0);
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_byte();

        match opcode {
            0xA9 => self.lda(AddressingMode::Immediate),
            0xA5 => self.lda(AddressingMode::ZeroPage),
            0xAD => self.lda(AddressingMode::Absolute),

            0x85 => self.sda(AddressingMode::ZeroPage),
            0x8D => self.sda(AddressingMode::Absolute),

            0xAA => self.tax(),

            0xE8 => self.inx(),

            0x69 => self.adc(),
            _ => panic!("This opcode is not supposed to be used."),
        }
    }
}


// Tests

#[cfg(test)]
mod tests_cpu {
    use crate::cpu::flags::Flag;
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

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
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
        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
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

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));

        cpu.register_a = 0x80;

        cpu.reset();
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
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

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));

        cpu.register_x = 0x7F;

        cpu.reset();
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_adc_immediate() {
        let mut cpu = CPU::new();

        cpu.memory.mem_write(0xFFFC, 0x00);
        cpu.memory.mem_write(0xFFFD, 0x80);

        cpu.memory.mem_write(0x8000, 0x69);
        cpu.memory.mem_write(0x8001, 0x10);

        cpu.register_a = 0x20;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_a, 0x30);
    }

    #[test]
    fn test_adc_carry_flag() {
        let mut cpu = CPU::new();

        cpu.memory.mem_write(0xFFFC, 0x00);
        cpu.memory.mem_write(0xFFFD, 0x80);

        cpu.memory.mem_write(0x8000, 0x69);
        cpu.memory.mem_write(0x8001, 0xFF);

        cpu.register_a = 0x02;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_adc_zero_flag() {
        let mut cpu = CPU::new();

        cpu.memory.mem_write(0xFFFC, 0x00);
        cpu.memory.mem_write(0xFFFD, 0x80);

        cpu.memory.mem_write(0x8000, 0x69);
        cpu.memory.mem_write(0x8001, 0x00);

        cpu.register_a = 0x00;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_adc_negative_flag() {
        let mut cpu = CPU::new();

        cpu.memory.mem_write(0xFFFC, 0x00);
        cpu.memory.mem_write(0xFFFD, 0x80);

        cpu.memory.mem_write(0x8000, 0x69);
        cpu.memory.mem_write(0x8001, 0x80);

        cpu.register_a = 0x00;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_adc_overflow_flag() {
        let mut cpu = CPU::new();

        cpu.memory.mem_write(0xFFFC, 0x00);
        cpu.memory.mem_write(0xFFFD, 0x80);

        cpu.memory.mem_write(0x8000, 0x69);
        cpu.memory.mem_write(0x8001, 0x50);

        cpu.register_a = 0x50;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::OVERFLOW));
    }
}

