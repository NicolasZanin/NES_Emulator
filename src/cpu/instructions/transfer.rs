use crate::cpu::cpu::CPU;

impl CPU {
    fn transfer(&mut self, from: u8) -> u8 {
        self.status.update_zero_and_negative_flags(from);
        from
    }

    pub(crate) fn tax(&mut self) {
        self.register.x = self.transfer(self.register.a);
    }

    pub(crate) fn tay(&mut self) {
        self.register.y = self.transfer(self.register.a);
    }

    pub(crate) fn tsx(&mut self) {
        self.register.x = self.transfer(self.stack.get());
    }

    pub(crate) fn txa(&mut self) {
        self.register.a = self.transfer(self.register.x);
    }

    pub(crate) fn txs(&mut self) {
        self.stack.set(self.register.x);
    }

    pub(crate) fn tya(&mut self) {
        self.register.a = self.transfer(self.register.y);
    }
}

#[cfg(test)]
mod tests_transfer {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_tax_transfer_a_to_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .load_program(&[0xAA])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0x42);
    }

    #[test]
    fn test_tax_flags() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xAA])
            .build();

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
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xAA])
            .build();
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8001);
    }

    #[test]
    fn test_tay() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x37)
            .load_program(&[0xA8])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0x37);
    }

    #[test]
    fn test_txa() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x99)
            .load_program(&[0x8A])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x99);
    }

    #[test]
    fn test_tya() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x55)
            .load_program(&[0x98])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x55);
    }

    #[test]
    fn test_tsx() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xAB)
            .load_program(&[0xBA])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0xAB);
    }

    #[test]
    fn test_tsx_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0x00)
            .load_program(&[0xBA])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_tsx_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0x80)
            .load_program(&[0xBA])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_txs() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0xCD)
            .load_program(&[0x9A])
            .build();

        cpu.step();

        assert_eq!(cpu.stack.get(), 0xCD);
    }

    #[test]
    fn test_txs_does_not_modify_flags() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x00)
            .set_flags(Flag::NEGATIVE)
            .load_program(&[0x9A])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
        assert!(!cpu.status.get_flag(Flag::ZERO));
    }
}
