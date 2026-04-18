use crate::cpu::flags::Flag;
use crate::cpu::flags::Flags;
use crate::cpu::memory::Memory;

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
    Indirect,
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

    fn read_from_pc(&mut self) -> u16 {
        let low_byte = self.fetch_byte() as u16;
        let high_byte = self.fetch_byte() as u16;

        (high_byte << 8) | low_byte
    }

    fn read_from_memory(&mut self, addr: u16) -> u16 {
        let low_byte = self.memory.mem_read(addr) as u16;
        let high_byte = self.memory.mem_read(addr + 1) as u16;

        (high_byte << 8) | low_byte
    }

    fn get_operand_address(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.fetch_byte() as u16,
            AddressingMode::ZeroPage => self.fetch_byte() as u16,
            AddressingMode::Absolute => self.read_from_pc(),
            AddressingMode::Indirect => {
                let address_target = self.read_from_pc();
                self.read_from_memory(address_target)
            }
        }
    }

    fn get_operand_value(&mut self, mode: AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);

        match mode {
            AddressingMode::Immediate => addr as u8,
            _ => self.memory.mem_read(addr),
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
        self.register_x = if self.register_x == 0xFF {
            0
        } else {
            self.register_x + 1
        };
        self.status.update_zero_and_negative_flags(self.register_x);
    }

    fn add_with_carry(&mut self, value: u8) {
        let a = self.register_a;
        let carry = self.status.get_flag(Flag::CARRY) as u16;

        let sum = a as u16 + value as u16 + carry;
        let result = sum as u8;
        let overflow = (a ^ result) & (value ^ result) & 0x80;

        self.register_a = result;

        self.status.update_zero_and_negative_flags(result);
        self.status.update_carry_flags(sum > 0xFF);
        self.status.update_overflow_flags(overflow != 0);
    }

    fn adc(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.add_with_carry(value);
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        let invert = value.wrapping_neg().wrapping_sub(1);

        // reverse value and remove 1 for carry
        // sbc = a - m - (1 - c) = a - m - 1 + c
        self.add_with_carry(invert);
    }

    fn compare(&mut self, mode: AddressingMode, register: u8) {
        let value = self.get_operand_value(mode);

        self.status
            .update_zero_and_negative_flags(register.wrapping_sub(value));
        self.status.update_carry_flags(register >= value);
    }

    fn cmp(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register_a);
    }

    fn cpx(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register_x);
    }

    fn cpy(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register_y);
    }

    fn jmp(&mut self, mode: AddressingMode) {
        self.program_counter = self.get_operand_address(mode);
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

            0x69 => self.adc(AddressingMode::Immediate),
            0x65 => self.adc(AddressingMode::ZeroPage),
            0x6D => self.adc(AddressingMode::Absolute),

            0xE9 => self.sbc(AddressingMode::Immediate),
            0xE5 => self.sbc(AddressingMode::ZeroPage),
            0xED => self.sbc(AddressingMode::Absolute),

            0xC9 => self.cmp(AddressingMode::Immediate),
            0xC5 => self.cmp(AddressingMode::ZeroPage),
            0xCD => self.cmp(AddressingMode::Absolute),

            0xE0 => self.cpx(AddressingMode::Immediate),
            0xE4 => self.cpx(AddressingMode::ZeroPage),
            0xEC => self.cpx(AddressingMode::Absolute),

            0xC0 => self.cpy(AddressingMode::Immediate),
            0xC4 => self.cpy(AddressingMode::ZeroPage),
            0xCC => self.cpy(AddressingMode::Absolute),

            0x4C => self.jmp(AddressingMode::Absolute),
            0x6C => self.jmp(AddressingMode::Indirect),
            _ => panic!("This opcode is not supposed to be used."),
        }
    }
}

// Tests
#[cfg(test)]
mod tests_cpu {
    use super::*;
    use crate::cpu::flags::Flag;

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

        memory.mem_write(0x8000, 0xAA);

        let mut cpu = CPU::new_mem(memory);

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8001);
    }

    #[test]
    fn test_inx_increment() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE8); // INX

        let mut cpu = CPU::new_mem(memory);
        cpu.register_x = 0x41;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_x, 0x42);
    }

    #[test]
    fn test_inx_overflow() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);
        memory.mem_write(0x8000, 0xE8); // INX

        let mut cpu = CPU::new_mem(memory);
        cpu.register_x = 0xFF;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_x, 0x00);
    }

    #[test]
    fn test_inx_flags() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE8); // INX

        let mut cpu = CPU::new_mem(memory);
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
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x69);
        memory.mem_write(0x8001, 0x10);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 0x20;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_a, 0x30);
    }

    #[test]
    fn test_adc_carry_flag() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x69);
        memory.mem_write(0x8001, 0xFF);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 0x02;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_adc_zero_flag() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x69);
        memory.mem_write(0x8001, 0x00);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 0x00;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_adc_negative_flag() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x69);
        memory.mem_write(0x8001, 0x80);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 0x00;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_adc_overflow_flag() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x69);
        memory.mem_write(0x8001, 0x50);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 0x50;

        cpu.reset();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::OVERFLOW));
    }

    #[test]
    fn test_sbc_immediate_basic() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE9); // SBC #$03
        memory.mem_write(0x8001, 0x03);

        let mut cpu = CPU::new_mem(memory);

        cpu.register_a = 10;

        cpu.reset();
        cpu.status.update_carry_flags(true);

        cpu.step();

        assert_eq!(cpu.register_a, 7);
    }

    #[test]
    fn test_sbc_no_carry_flag() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE9);
        memory.mem_write(0x8001, 0x03);

        let mut cpu = CPU::new_mem(memory);

        cpu.register_a = 10;

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_a, 6);
    }

    #[test]
    fn test_sbc_zero_flag() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE9);
        memory.mem_write(0x8001, 0x05);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 5;

        cpu.reset();
        cpu.status.update_carry_flags(true);

        cpu.step();

        assert_eq!(cpu.register_a, 0);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_sbc_negative_flag() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE9);
        memory.mem_write(0x8001, 0x02);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 1;

        cpu.reset();
        cpu.status.update_carry_flags(true);

        cpu.step();

        assert_eq!(cpu.register_a, 0xFF);
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_sbc_carry_with_carry_flags() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE9);
        memory.mem_write(0x8001, 0x03);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 10;

        cpu.reset();
        cpu.status.update_carry_flags(true);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_sbc_overflow() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0xE9);
        memory.mem_write(0x8001, 0x01);

        let mut cpu = CPU::new_mem(memory);
        cpu.register_a = 0x80; // -128

        cpu.reset();
        cpu.status.update_carry_flags(true);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::OVERFLOW));
    }

    fn setup_compare_tests(opcode: u8, operand_low: u8, operand_high: Option<u8>) -> CPU {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, opcode);
        memory.mem_write(0x8001, operand_low);

        if let Some(high) = operand_high {
            memory.mem_write(0x8002, high);
        }

        let mut cpu = CPU::new_mem(memory);
        cpu.reset();
        cpu
    }

    #[test]
    fn test_cmp_equal() {
        let mut cpu = setup_compare_tests(0xC9, 0x05, None); // CMP #$05
        cpu.register_a = 0x05;

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_more_than() {
        let mut cpu = setup_compare_tests(0xC9, 0x03, None); // CMP #$03
        cpu.register_a = 0x05;

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_less_than() {
        let mut cpu = setup_compare_tests(0xC9, 0x05, None); // CMP #$05
        cpu.register_a = 0x03;

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_zeropage() {
        let mut cpu = setup_compare_tests(0xC5, 0x10, None); // CMP $10
        cpu.register_a = 0x42;
        cpu.memory.mem_write(0x0010, 0x42);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cmp_absolute() {
        let mut cpu = setup_compare_tests(0xCD, 0x34, Some(0x12)); // CMP $1234
        cpu.register_a = 0x50;
        cpu.memory.mem_write(0x1234, 0x10);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpx_equal() {
        let mut cpu = setup_compare_tests(0xE0, 0x08, None); // CPX #$08
        cpu.register_x = 0x08;

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cpx_less_than() {
        let mut cpu = setup_compare_tests(0xE0, 0x10, None); // CPX #$10
        cpu.register_x = 0x01;

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cpx_zeropage() {
        let mut cpu = setup_compare_tests(0xE4, 0x20, None); // CPX $20
        cpu.register_x = 0x33;
        cpu.memory.mem_write(0x0020, 0x33);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpy_equal() {
        let mut cpu = setup_compare_tests(0xC0, 0x09, None); // CPY #$09
        cpu.register_y = 0x09;

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cpy_greater_than() {
        let mut cpu = setup_compare_tests(0xC0, 0x01, None); // CPY #$01
        cpu.register_y = 0x05;

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpy_absolute() {
        let mut cpu = setup_compare_tests(0xCC, 0x00, Some(0x20)); // CPY $2000
        cpu.register_y = 0x01;
        cpu.memory.mem_write(0x2000, 0x02);

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_jmp_absolute() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x4C); // JMP Absolute
        memory.mem_write(0x8001, 0x34); // low
        memory.mem_write(0x8002, 0x12); // high

        let mut cpu = CPU::new_mem(memory);
        cpu.reset();
        cpu.step();

        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_jmp_indirect() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x6C); // JMP Indirect
        memory.mem_write(0x8001, 0x00); // pointer low
        memory.mem_write(0x8002, 0x20); // pointer high

        // pointer = $2000
        memory.mem_write(0x2000, 0x78); // low target
        memory.mem_write(0x2001, 0x56); // high target

        let mut cpu = CPU::new_mem(memory);
        cpu.reset();
        cpu.step();

        assert_eq!(cpu.program_counter, 0x5678);
    }

    #[test]
    fn test_jmp_chain_execution() {
        let mut memory = Memory::new();
        init_test_memory(&mut memory);

        memory.mem_write(0x8000, 0x4C); // JMP $9000
        memory.mem_write(0x8001, 0x00);
        memory.mem_write(0x8002, 0x90);

        memory.mem_write(0x9000, 0xA9); // LDA #$42
        memory.mem_write(0x9001, 0x42);

        let mut cpu = CPU::new_mem(memory);
        cpu.reset();
        cpu.step(); // jump
        cpu.step(); // lda

        assert_eq!(cpu.register_a, 0x42);
    }
}
