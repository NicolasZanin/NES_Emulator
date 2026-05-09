use crate::cpu::cpu::{AddressingMode, CPU};
use crate::cpu::flags::Flag;

impl CPU {
    pub(crate) fn jmp(&mut self, mode: AddressingMode) {
        self.program_counter = self.get_operand_address(mode);
    }

    fn branch(&mut self, flag: Flag, is_set: bool) {
        // change u8 in i8 to cast in negative or positive number
        let jump_address = self.fetch_byte() as i8 as i16;

        if self.status.get_flag(flag) == is_set {
            self.program_counter = self.program_counter.wrapping_add_signed(jump_address);
        }
    }

    pub(crate) fn beq(&mut self) {
        self.branch(Flag::ZERO, true);
    }

    pub(crate) fn bne(&mut self) {
        self.branch(Flag::ZERO, false);
    }

    pub(crate) fn bcc(&mut self) {
        self.branch(Flag::CARRY, false);
    }

    pub(crate) fn bcs(&mut self) {
        self.branch(Flag::CARRY, true);
    }

    pub(crate) fn bvc(&mut self) {
        self.branch(Flag::OVERFLOW, false);
    }

    pub(crate) fn bvs(&mut self) {
        self.branch(Flag::OVERFLOW, true);
    }

    pub(crate) fn bmi(&mut self) {
        self.branch(Flag::NEGATIVE, true);
    }

    pub(crate) fn bpl(&mut self) {
        self.branch(Flag::NEGATIVE, false);
    }

    pub(crate) fn jsr(&mut self) {
        let address = self.read_from_pc();
        self.stack.write_address(&mut self.memory, self.program_counter - 1);

        self.program_counter = address;
    }

    pub(crate) fn rts(&mut self) {
        let address = self.stack.read_address(&mut self.memory) + 1; // last PC + 1
        self.program_counter = address;
    }
}


#[cfg(test)]
mod tests_branch {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_jmp_absolute() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x4C, 0x34, 0x12])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_jmp_indirect() {
        let mut cpu = CPUBuilder::new()
            .memory(0x2000, 0x78) // low target
            .memory(0x2001, 0x56)// high target
            .load_program(&[0x6C, 0x00, 0x20])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x5678);
    }

    #[test]
    fn test_jmp_chain_execution() {
        let mut cpu = CPUBuilder::new()
            .memory(0x9000, 0xA9) // LDA $42
            .memory(0x9001, 0x42)
            .load_program(&[0x4C, 0x00, 0x90])
            .build();

        cpu.step(); // jump
        cpu.step(); // lda

        assert_eq!(cpu.register.a, 0x42);
    }

    #[test]
    fn test_beq_taken_forward() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::ZERO)
            .load_program(&[0xF0, 0x05])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8007);
    }

    #[test]
    fn test_beq_not_taken() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xF0, 0x05])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bne_taken_backward() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xD0, 0xFB]) // -5
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x7FFD);
    }

    #[test]
    fn test_bne_not_taken() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::ZERO)
            .load_program(&[0xD0, 0x05])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bcc_taken() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x90, 0x04])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_bcc_not_taken() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::CARRY)
            .load_program(&[0x90, 0x04])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bcs_taken() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::CARRY)
            .load_program(&[0xB0, 0x02])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8004);
    }

    #[test]
    fn test_bcs_not_taken() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xB0, 0x02])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bvc_taken() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x50, 0x06])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8008);
    }

    #[test]
    fn test_bvc_not_taken() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::OVERFLOW)
            .load_program(&[0x50, 0x06])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_bvs_taken() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::OVERFLOW)
            .load_program(&[0x70, 0x01])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8003);
    }

    #[test]
    fn test_bvs_not_taken() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x70, 0x01])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }
    #[test]
    fn test_branch_zero_offset() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::OVERFLOW)
            .load_program(&[0x70, 0x00])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_branch_negative_one() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xD0, 0xFF])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8001);
    }


    #[test]
    fn test_jsr_jumps_to_target_address() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x20, 0x34, 0x12])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_jsr_pushes_return_address_on_stack() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x20, 0x00, 0x90])
            .build();

        cpu.step();

        // Return address = PC - 1 = 0x8002
        assert_eq!(cpu.memory.mem_read(0x01FF), 0x80); // high byte
        assert_eq!(cpu.memory.mem_read(0x01FE), 0x02); // low byte

        assert_eq!(cpu.stack.get(), 0xFD);
    }

    #[test]
    fn test_multiple_jsr_stack_growth() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x20, 0x00, 0x90])
            .build();

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
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xFD)
            .memory(0x01FF, 0x80) // Simulate stack containing return address 0x8002
            .memory(0x01FE, 0x02)
            .load_program(&[0x60])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8003);
    }


    #[test]
    fn test_rts_restores_stack_pointer() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xFD)
            .load_program(&[0x60, 0x00, 0x90])
            .build();

        cpu.step();

        assert_eq!(cpu.stack.get(), 0xFF);
    }

    #[test]
    fn test_jsr_rts_full_cycle() {
        let mut cpu = CPUBuilder::new()
            .memory(0x9000, 0x60) // RTS at $9000
            .load_program(&[0x20, 0x00, 0x90])
            .build();

        cpu.step(); // JSR
        cpu.step(); // RTS

        assert_eq!(cpu.program_counter, 0x8003);
    }
}
