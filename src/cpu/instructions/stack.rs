use crate::cpu::cpu::CPU;

impl CPU {
    pub(crate) fn pha(&mut self) {
        self.stack.write_value(&mut self.memory, self.register.a);
    }

    pub(crate) fn pla(&mut self) {
        self.register.a = self.stack.read_value(&mut self.memory);
        self.status.update_zero_and_negative_flags(self.register.a);
    }

    pub(crate) fn php(&mut self) {
        let status = self.status.get_status();
        self.stack.write_value(&mut self.memory, status);
    }

    pub(crate) fn plp(&mut self) {
        let stored_status = self.stack.read_value(&mut self.memory);
        self.status.set_status(stored_status);
    }
}


#[cfg(test)]
mod tests_stack {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;
    use crate::cpu::stack::STACK_BASE;

    #[test]
    fn test_pha_pushes_a_to_stack() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .load_program(&[0x48])
            .build();

        cpu.step();

        let addr = STACK_BASE + 0xFF;
        assert_eq!(cpu.memory.mem_read(addr), 0x42);
        assert_eq!(cpu.stack.get(), 0xFE);
    }

    #[test]
    fn test_pla_pulls_value_from_stack() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xFE)
            .memory(0x01FF, 0x37)
            .load_program(&[0x68])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x37);
        assert_eq!(cpu.stack.get(), 0xFF);
    }

    #[test]
    fn test_pla_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xFE)
            .load_program(&[0x68])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_pla_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xFE)
            .memory(0x01FF, 0x80)
            .load_program(&[0x68])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_php_pushes_status() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::CARRY)
            .load_program(&[0x08])
            .build();

        cpu.step();

        let addr = STACK_BASE + 0xFF;
        assert_eq!(cpu.memory.mem_read(addr), cpu.status.get_status());
        assert_eq!(cpu.stack.get(), 0xFE);
    }

    #[test]
    fn test_plp_pulls_status() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xFE)
            .memory(0x01FF, 0b00000001)
            .load_program(&[0x28])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert_eq!(cpu.stack.get(), 0xFF);
    }

    #[test]
    fn test_stack_lifo_behavior() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0xAA)
            .load_program(&[0x48, 0x48, 0x68, 0x68])
            .build();

        cpu.step();

        cpu.register.a = 0xBB;
        cpu.step();

        // simulate PLA
        cpu.step();
        assert_eq!(cpu.register.a, 0xBB);

        cpu.step();
        assert_eq!(cpu.register.a, 0xAA);
    }
}
