use crate::cpu::flags::Flag;
use crate::cpu::flags::Flags;
use crate::cpu::memory::Memory;
use crate::cpu::registers::Registers;
use crate::cpu::stack::Stack;

// Struct and enums
pub struct CPU {
    program_counter: u16,
    register: Registers,
    stack: Stack,
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

    pub fn new(memory: Memory) -> Self {
        CPU {
            program_counter: 0,
            register: Registers::new(),
            stack: Stack::new(),
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
        self.register.a = value;
        self.status.update_zero_and_negative_flags(self.register.a);
    }

    fn sda(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.memory.mem_write(address, self.register.a);
    }

    fn tax(&mut self) {
        self.register.x = self.register.a;
        self.status.update_zero_and_negative_flags(self.register.x);
    }

    fn inx(&mut self) {
        self.register.x = if self.register.x == 0xFF {
            0
        } else {
            self.register.x + 1
        };
        self.status.update_zero_and_negative_flags(self.register.x);
    }

    fn add_with_carry(&mut self, value: u8) {
        let a = self.register.a;
        let carry = self.status.get_flag(Flag::CARRY) as u16;

        let sum = a as u16 + value as u16 + carry;
        let result = sum as u8;
        let overflow = (a ^ result) & (value ^ result) & 0x80;

        self.register.a = result;

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
        self.compare(mode, self.register.a);
    }

    fn cpx(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register.x);
    }

    fn cpy(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register.y);
    }

    fn jmp(&mut self, mode: AddressingMode) {
        self.program_counter = self.get_operand_address(mode);
    }

    fn branch(&mut self, flag: Flag, is_set: bool) {
        // change u8 in i8 to cast in negative or positive number
        let jump_address = self.fetch_byte() as i8 as i16;

        if self.status.get_flag(flag) == is_set {
            self.program_counter = self.program_counter.wrapping_add_signed(jump_address);
        }
    }

    fn beq(&mut self) {
        self.branch(Flag::ZERO, true);
    }

    fn bne(&mut self) {
        self.branch(Flag::ZERO, false);
    }

    fn bcc(&mut self) {
        self.branch(Flag::CARRY, false);
    }

    fn bcs(&mut self) {
        self.branch(Flag::CARRY, true);
    }

    fn bvc(&mut self) {
        self.branch(Flag::OVERFLOW, false);
    }

    fn bvs(&mut self) {
        self.branch(Flag::OVERFLOW, true);
    }

    fn bmi(&mut self) {
        self.branch(Flag::NEGATIVE, true);
    }

    fn bpl(&mut self) {
        self.branch(Flag::NEGATIVE, false);
    }


    fn pha(&mut self) {
        self.stack.write_value(&mut self.memory, self.register.a);
    }

    fn pla(&mut self) {
        self.register.a = self.stack.read_value(&mut self.memory);
        self.status.update_zero_and_negative_flags(self.register.a);
    }

    fn php(&mut self) {
        let status = self.status.get_status();
        self.stack.write_value(&mut self.memory, status);
    }

    fn plp(&mut self) {
        let stored_status = self.stack.read_value(&mut self.memory);
        self.status.set_status(stored_status);
    }

    fn jsr(&mut self) {
        let address = self.read_from_pc();
        self.stack.write_address(&mut self.memory, self.program_counter - 1);

        self.program_counter = address;
    }

    fn rts(&mut self) {
        let address = self.stack.read_address(&mut self.memory) + 1; // last PC + 1
        self.program_counter = address;
    }

    fn brk(&mut self) {
        self.stack.write_address(&mut self.memory, self.program_counter);

        self.status.set_flag(Flag::BREAK, true);
        self.status.set_flag(Flag::INTERRUPT, true);
        self.stack.write_value(&mut self.memory, self.status.get_status());

        self.program_counter = self.read_from_memory(0xFFFE);
    }

    fn rti(&mut self) {
        let status = self.stack.read_value(&mut self.memory);
        let previous_address = self.stack.read_address(&mut self.memory);

        self.status.set_status(status);
        self.program_counter = previous_address;
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

            // Branching
            0xF0 => self.beq(),
            0xD0 => self.bne(),
            0x90 => self.bcc(),
            0xB0 => self.bcs(),
            0x50 => self.bvc(),
            0x70 => self.bvs(),
            0x30 => self.bmi(),
            0x10 => self.bpl(),

            // Stack
            0x48 => self.pha(),
            0x68 => self.pla(),
            0x08 => self.php(),
            0x28 => self.plp(),
            0x20 => self.jsr(),
            0x60 => self.rts(),

            0x00 => self.brk(),
            0x40 => self.rti(),

            _ => panic!("This opcode is not supposed to be used."),
        }
    }
}


#[cfg(test)]
pub struct CPUState {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8
}

#[cfg(test)]
impl Default for CPUState {
    fn default() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0x8000,
            stack_pointer: 0xFF
        }
    }
}

#[cfg(test)]
impl CPU {
    fn new_for_test(memory: Memory, state: CPUState) -> CPU {
        CPU {
            register: Registers::new_for_tests(state.register_a, state.register_x, state.register_y),
            program_counter: state.program_counter,
            stack: Stack::new_sp(state.stack_pointer),
            status: Flags::new(),
            memory,
        }
    }
}

// Tests
#[cfg(test)]
mod tests_cpu {
    use super::*;
    use crate::cpu::flags::Flag;
    use crate::cpu::stack::STACK_BASE;

    fn setup_cpu(program: &[u8], state: CPUState) -> CPU {
        let mut mem = Memory::new();
        mem.load_program(program);

        let mut cpu = CPU::new_for_test(mem, state);
        cpu.reset();

        cpu
    }

    fn setup_cpu_without_state(program: &[u8]) -> CPU {
        setup_cpu(program, CPUState::default())
    }

    fn setup_cpu_register_a(register_a: u8, program: &[u8]) -> CPU {
        let cpu_state = CPUState { register_a, ..Default::default() };
        setup_cpu(program, cpu_state)
    }

    fn setup_cpu_register_x(register_x: u8, program: &[u8]) -> CPU {
        let cpu_state = CPUState { register_x, ..Default::default() };
        setup_cpu(program, cpu_state)
    }

    fn setup_cpu_register_y(register_y: u8, program: &[u8]) -> CPU {
        let cpu_state = CPUState { register_y, ..Default::default() };
        setup_cpu(program, cpu_state)
    }

    fn setup_stack_pointer(stack_pointer: u8, program: &[u8]) -> CPU {
        let cpu_state = CPUState { stack_pointer, ..Default::default() };
        setup_cpu(program, cpu_state)
    }

    #[test]
    fn test_example() {
        let cpu = setup_cpu_without_state(&[]);
        assert_eq!(cpu.program_counter, 0x8000);
    }


    #[test]
    fn test_lda_immediate_sets_register_a_and_flags() {
        let mut cpu = setup_cpu_without_state(&[0xA9, 0x42]);
        cpu.step();

        assert_eq!(cpu.register.a, 0x42);

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_lda_zero_page() {
        let mut cpu = setup_cpu_without_state(&[0xA5, 0x10]);
        cpu.memory.mem_write(0x0010, 0x99);

        cpu.step();

        assert_eq!(cpu.register.a, 0x99);
        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }



    #[test]
    fn test_sta_zero_page() {
         let mut cpu = setup_cpu_register_a(0x42, &[0x85, 0x10]);
        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x0010), 0x42);
    }

    #[test]
    fn test_sta_absolute() {
        let mut cpu = setup_cpu_register_a(0x99, &[0x8D, 0x34, 0x12]);
        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x1234), 0x99);
    }

    #[test]
    fn test_sta_pc_increment() {
        let mut cpu = setup_cpu_register_a(0x42, &[0x85, 0x10]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_tax_transfer_a_to_x() {
        let mut cpu = setup_cpu_register_a(0x42, &[0xAA]);
        cpu.step();

        assert_eq!(cpu.register.x, 0x42);
    }

    #[test]
    fn test_tax_flags() {
        let mut cpu = setup_cpu_register_a(0x00, &[0xAA]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));

        cpu.register.a = 0x80;

        cpu.reset();
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_tax_pc_increment() {
        let mut cpu = setup_cpu_without_state( &[0xAA]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8001);
    }

    #[test]
    fn test_inx_increment() {
        let mut cpu = setup_cpu_register_x(0x41, &[0xE8]);
        cpu.step();

        assert_eq!(cpu.register.x, 0x42);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = setup_cpu_register_x(0xFF, &[0xE8]);
        cpu.step();

        assert_eq!(cpu.register.x, 0x00);
    }

    #[test]
    fn test_inx_flags() {
        let mut cpu = setup_cpu_register_x(0xFF, &[0xE8]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));

        cpu.register.x = 0x7F;

        cpu.reset();
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_adc_immediate() {
        let mut cpu = setup_cpu_register_a(0x20, &[0x69, 0x10]);
        cpu.step();

        assert_eq!(cpu.register.a, 0x30);
    }

    #[test]
    fn test_adc_carry_flag() {
        let mut cpu = setup_cpu_register_a(0x02, &[0x69, 0xFF]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_adc_zero_flag() {
        let mut cpu = setup_cpu_without_state( &[0x69, 0x00]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_adc_negative_flag() {
        let mut cpu = setup_cpu_without_state( &[0x69, 0x80]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_adc_overflow_flag() {
        let mut cpu = setup_cpu_register_a(0x50, &[0x69, 0x50]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::OVERFLOW));
    }

    #[test]
    fn test_sbc_immediate_basic() {
        let mut cpu = setup_cpu_register_a(10, &[0xE9, 0x03]);
        cpu.status.update_carry_flags(true);
        cpu.step();

        assert_eq!(cpu.register.a, 7);
    }

    #[test]
    fn test_sbc_no_carry_flag() {
        let mut cpu = setup_cpu_register_a(10, &[0xE9, 0x03]);
        cpu.step();

        assert_eq!(cpu.register.a, 6);
    }

    #[test]
    fn test_sbc_zero_flag() {
        let mut cpu = setup_cpu_register_a(5, &[0xE9, 0x05]);
        cpu.status.update_carry_flags(true);
        cpu.step();

        assert_eq!(cpu.register.a, 0);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_sbc_negative_flag() {
        let mut cpu = setup_cpu_register_a(1, &[0xE9, 0x02]);
        cpu.status.update_carry_flags(true);
        cpu.step();

        assert_eq!(cpu.register.a, 0xFF);
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_sbc_carry_with_carry_flags() {
        let mut cpu = setup_cpu_register_a(10, &[0xE9, 0x03]);
        cpu.status.update_carry_flags(true);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_sbc_overflow() {
        let mut cpu = setup_cpu_register_a(0x80, &[0xE9, 0x02]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::OVERFLOW));
    }

    #[test]
    fn test_cmp_equal() {
        let mut cpu = setup_cpu_register_a(0x05, &[0xC9, 0x05]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_more_than() {
        let mut cpu = setup_cpu_register_a(0x05, &[0xC9, 0x03]);
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_less_than() {
        let mut cpu = setup_cpu_register_a(0x03, &[0xC9, 0x05]);
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_zeropage() {
        let mut cpu = setup_cpu_register_a(0x42, &[0xC5, 0x10]);
        cpu.memory.mem_write(0x0010, 0x42);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cmp_absolute() {
        let mut cpu = setup_cpu_register_a(0x42, &[0xCD, 0x34, 0x12]);
        cpu.memory.mem_write(0x1234, 0x10);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpx_equal() {
        let mut cpu = setup_cpu_register_x(0x08, &[0xE0, 0x08]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cpx_less_than() {
        let mut cpu = setup_cpu_register_x(0x01, &[0xE0, 0x10]);
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cpx_zeropage() {
        let mut cpu = setup_cpu_register_x(0x33, &[0xE4, 0x20]);
        cpu.memory.mem_write(0x0020, 0x33);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpy_equal() {
        let mut cpu = setup_cpu_register_y(0x09, &[0xC0, 0x09]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cpy_greater_than() {
        let mut cpu = setup_cpu_register_y(0x05, &[0xC0, 0x01]);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpy_absolute() {
        let mut cpu = setup_cpu_register_y(0x01, &[0xCC, 0x00, 0x20]);
        cpu.memory.mem_write(0x2000, 0x02);

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_jmp_absolute() {
        let mut cpu = setup_cpu_without_state(&[0x4C, 0x34, 0x12]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_jmp_indirect() {
        let mut cpu = setup_cpu_without_state(&[0x6C, // JMP Indirect
            0x00, // pointer low
            0x20, // pointer high
        ]);
        cpu.memory.mem_write(0x2000, 0x78); // low target
        cpu.memory.mem_write(0x2001, 0x56); // high target

        cpu.step();

        assert_eq!(cpu.program_counter, 0x5678);
    }

    #[test]
    fn test_jmp_chain_execution() {
        let mut cpu = setup_cpu_without_state(&[0x4C, 0x00, 0x90  ]); // JMP $9000
        cpu.memory.mem_write(0x9000, 0xA9); // LDA $42
        cpu.memory.mem_write(0x9001, 0x42);

        cpu.step(); // jump
        cpu.step(); // lda

        assert_eq!(cpu.register.a, 0x42);
    }

    fn setup_branching_test(flag: Flag, value: bool, program: &[u8]) -> CPU {
        let mut cpu = setup_cpu_without_state(program);
        cpu.status.set_flag(flag, value);
        cpu
    }

    #[test]
    fn test_beq_taken_forward() {
        let mut cpu = setup_branching_test(Flag::ZERO, true, &[0xF0, 0x05]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8007);
    }

    #[test]
    fn test_beq_not_taken() {
        let mut cpu = setup_branching_test(Flag::ZERO, false, &[0xF0, 0x05]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bne_taken_backward() {
        let mut cpu = setup_branching_test(Flag::ZERO, false, &[0xD0, 0xFB]); // -5
        cpu.step();

        assert_eq!(cpu.program_counter, 0x7FFD);
    }

    #[test]
    fn test_bne_not_taken() {
        let mut cpu = setup_branching_test(Flag::ZERO, true, &[0xD0, 0x05]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bcc_taken() {
        let mut cpu = setup_branching_test(Flag::CARRY, false, &[0x90, 0x04]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_bcc_not_taken() {
        let mut cpu = setup_branching_test(Flag::CARRY, true, &[0x90, 0x04]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bcs_taken() {
        let mut cpu = setup_branching_test(Flag::CARRY, true, &[0xB0, 0x02]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8004);
    }

    #[test]
    fn test_bcs_not_taken() {
        let mut cpu = setup_branching_test(Flag::CARRY, false, &[0xB0, 0x02]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bvc_taken() {
        let mut cpu = setup_branching_test(Flag::OVERFLOW, false, &[0x50, 0x06]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8008);
    }

    #[test]
    fn test_bvc_not_taken() {
        let mut cpu = setup_branching_test(Flag::OVERFLOW, true, &[0x50, 0x06]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bvs_taken() {
        let mut cpu = setup_branching_test(Flag::OVERFLOW, true, &[0x70, 0x01]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8003);
    }

    #[test]
    fn test_bvs_not_taken() {
        let mut cpu = setup_branching_test(Flag::OVERFLOW, false, &[0x70, 0x01]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }
    #[test]
    fn test_branch_zero_offset() {
        let mut cpu = setup_branching_test(Flag::ZERO, true, &[0xF0, 0x00]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_branch_negative_one() {
        let mut cpu = setup_branching_test(Flag::ZERO, false, &[0xD0, 0xFF]);
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8001);
    }

    #[test]
    fn test_pha_pushes_a_to_stack() {
        let mut cpu = setup_cpu_register_a(0x42, &[0x48]); // PHA
        cpu.step();

        let addr = STACK_BASE + 0xFF;
        assert_eq!(cpu.memory.mem_read(addr), 0x42);
        assert_eq!(cpu.stack.get(), 0xFE);
    }

    #[test]
    fn test_pla_pulls_value_from_stack() {
        let mut cpu = setup_stack_pointer(0xFE, &[0x68]); // PLA
        cpu.memory.mem_write(0x01FF, 0x37);

        cpu.step();

        assert_eq!(cpu.register.a, 0x37);
        assert_eq!(cpu.stack.get(), 0xFF);
    }

    #[test]
    fn test_pla_sets_zero_flag() {
        let mut cpu = setup_stack_pointer(0xFE, &[0x68]); // PLA
        cpu.memory.mem_write(0x01FF, 0x00);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_pla_sets_negative_flag() {
        let mut cpu = setup_stack_pointer(0xFE, &[0x68]); // PLA
        cpu.memory.mem_write(0x01FF, 0x80);

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_php_pushes_status() {
        let mut cpu = setup_cpu_without_state(&[0x08]); // PHP
        cpu.status.set_flag(Flag::CARRY, true);

        cpu.step();

        let addr = STACK_BASE + 0xFF;
        assert_eq!(cpu.memory.mem_read(addr), cpu.status.get_status());
        assert_eq!(cpu.stack.get(), 0xFE);
    }

    #[test]
    fn test_plp_pulls_status() {
        let mut cpu = setup_stack_pointer(0xFE, &[0x28]); // PLP
        cpu.memory.mem_write(0x01FF, 0b00000001);
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert_eq!(cpu.stack.get(), 0xFF);
    }

    #[test]
    fn test_stack_lifo_behavior() {
        let mut cpu = setup_cpu_register_a(0xAA, &[0x48, 0x48, 0x68, 0x68]); // PHA
        cpu.step();

        cpu.register.a = 0xBB;
        cpu.step();

        // simulate PLA
        cpu.step();
        assert_eq!(cpu.register.a, 0xBB);

        cpu.step();
        assert_eq!(cpu.register.a, 0xAA);
    }

    #[test]
    fn test_jsr_jumps_to_target_address() {
        let mut cpu = setup_cpu_without_state(&[0x20, 0x34, 0x12]); // JSR $1234
        cpu.step();

        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_jsr_pushes_return_address_on_stack() {
        let mut cpu = setup_cpu_without_state(&[0x20, 0x00, 0x90]);
        cpu.step();

        // Return address = PC - 1 = 0x8002
        assert_eq!(cpu.memory.mem_read(0x01FF), 0x80); // high byte
        assert_eq!(cpu.memory.mem_read(0x01FE), 0x02); // low byte

        assert_eq!(cpu.stack.get(), 0xFD);
    }

    #[test]
    fn test_multiple_jsr_stack_growth() {
        let mut cpu = setup_cpu_without_state(&[0x20, 0x00, 0x90]);
        cpu.step();

        // Simulate second JSR at new location
        cpu.memory.mem_write(0x9000, 0x20);
        cpu.memory.mem_write(0x9001, 0x00);
        cpu.memory.mem_write(0x9002, 0xA0);

        cpu.step();

        assert_eq!(cpu.stack.get(), 0xFB);
    }

    #[test]
    fn test_rts_returns_to_correct_address() {
        // Simulate stack containing return address 0x8002
        let mut cpu = setup_stack_pointer(0xFD, &[0x60]);
        cpu.memory.mem_write(0x01FF, 0x80);
        cpu.memory.mem_write(0x01FE, 0x02);

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8003);
    }


    #[test]
    fn test_rts_restores_stack_pointer() {
        let mut cpu = setup_stack_pointer(0xFD, &[0x60, 0x00, 0x90]);
        cpu.step();

        assert_eq!(cpu.stack.get(), 0xFF);
    }

    #[test]
    fn test_jsr_rts_full_cycle() {
        let mut cpu = setup_cpu_without_state(&[0x20, 0x00, 0x90]);

        // RTS at $9000
        cpu.memory.mem_write(0x9000, 0x60);

        cpu.step(); // JSR
        cpu.step(); // RTS

        assert_eq!(cpu.program_counter, 0x8003);
    }

    #[test]
    fn test_brk_pushes_pc_and_status() {
        let mut cpu = setup_cpu_without_state(&[0x00]);
        cpu.memory.mem_write(0xFFFE, 0x00);
        cpu.memory.mem_write(0xFFFF, 0x90);
        cpu.step();

        // PC + 1 = 0x8001
        assert_eq!(cpu.memory.mem_read(0x01FF), 0x80);
        assert_eq!(cpu.memory.mem_read(0x01FE), 0x01);

        assert_eq!(cpu.stack.get(), 0xFC);
        assert_eq!(cpu.program_counter, 0x9000);
    }

    #[test]
    fn test_rti_restores_pc_and_status() {
        let mut cpu = setup_stack_pointer(0xFC, &[0x40]);

        // status
        cpu.memory.mem_write(0x01FD, 0b00000001);

        // PC = 0x8005
        cpu.memory.mem_write(0x01FE, 0x05);
        cpu.memory.mem_write(0x01FF, 0x80);

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8005);
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_brk_rti_cycle() {
        let mut cpu = setup_cpu_without_state(&[0x00]);
        cpu.memory.mem_write(0x9000, 0x40); // RTI

        cpu.memory.mem_write(0xFFFE, 0x00);
        cpu.memory.mem_write(0xFFFF, 0x90);

        cpu.step(); // BRK
        cpu.step(); // RTI

        assert_eq!(cpu.program_counter, 0x8001);
    }
}
