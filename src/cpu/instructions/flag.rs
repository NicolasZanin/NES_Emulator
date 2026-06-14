use crate::cpu::cpu::{CPU};

impl CPU {
    pub(crate) fn clc(&mut self) {
        self.status.update_carry_flags(false);
    }

    pub(crate) fn cld(&mut self) {
        self.status.update_decimal_flags(false);
    }

    pub(crate) fn cli(&mut self) {
        self.status.update_interrupt_flags(false);
    }

    pub(crate) fn clv(&mut self) {
        self.status.update_overflow_flags(false);
    }

    pub(crate) fn sec(&mut self) {
        self.status.update_carry_flags(true);
    }

    pub(crate) fn sed(&mut self) {
        self.status.update_decimal_flags(true);
    }

    pub(crate) fn sei(&mut self) {
        self.status.update_interrupt_flags(true);
    }
}

#[cfg(test)]
mod tests_load_store {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_clc_clears_carry() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::CARRY)
            .load_program(&[0x18])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cld_clears_decimal() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::DECIMAL)
            .load_program(&[0xD8])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::DECIMAL));
    }

    #[test]
    fn test_cli_clears_interrupt() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::INTERRUPT)
            .load_program(&[0x58])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::INTERRUPT));
    }

    #[test]
    fn test_clv_clears_overflow() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::OVERFLOW)
            .load_program(&[0xB8])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::OVERFLOW));
    }

    #[test]
    fn test_sec_sets_carry() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x38])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_sed_sets_decimal() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xF8])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::DECIMAL));
    }

    #[test]
    fn test_sei_sets_interrupt() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x78])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::INTERRUPT));
    }

    #[test]
    fn test_sec_does_not_modify_other_flags() {
        let mut cpu = CPUBuilder::new()
            .set_flags(Flag::ZERO)
            .set_flags(Flag::NEGATIVE)
            .load_program(&[0x38])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }
}
