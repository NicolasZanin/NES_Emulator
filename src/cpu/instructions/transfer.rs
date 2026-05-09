use crate::cpu::cpu::CPU;

impl CPU {
    pub(crate) fn tax(&mut self) {
        self.register.x = self.register.a;
        self.status.update_zero_and_negative_flags(self.register.x);
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


}
