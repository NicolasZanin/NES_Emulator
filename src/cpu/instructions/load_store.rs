use crate::cpu::cpu::{AddressingMode, CPU};

impl CPU {
    pub(crate) fn lda(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.register.a = value;
        self.status.update_zero_and_negative_flags(self.register.a);
    }

    pub(crate) fn sda(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.memory.mem_write(address, self.register.a);
    }
}

#[cfg(test)]
mod tests_load_store {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_lda_immediate_sets_register_a_and_flags() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0xA9, 0x42])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x42);

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_lda_zero_page() {
        let mut cpu = CPUBuilder::new()
            .memory(0x0010, 0x99)
            .load_program(&[0xA5, 0x10])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x99);
        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_sta_zero_page() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .load_program(&[0x85, 0x10])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x0010), 0x42);
    }

    #[test]
    fn test_sta_absolute() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x99)
            .load_program(&[0x8D, 0x34, 0x12])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x1234), 0x99);
    }

    #[test]
    fn test_sta_pc_increment() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .load_program(&[0x85, 0x10])
            .build();
        
        cpu.step();

        assert_eq!(cpu.program_counter, 0x8002);
    }
}
