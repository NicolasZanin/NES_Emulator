use crate::cpu::cpu::{CPU};

impl CPU {
    pub(crate) fn inx(&mut self) {
        self.register.x = if self.register.x == 0xFF {
            0
        } else {
            self.register.x + 1
        };
        self.status.update_zero_and_negative_flags(self.register.x);
    }
}

#[cfg(test)]
mod tests_increment {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;


    #[test]
    fn test_inx_increment() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x41)
            .load_program(&[0xE8])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0x42);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0xFF)
            .load_program(&[0xE8])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0x00);
    }

    #[test]
    fn test_inx_flags() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0xFF)
            .load_program(&[0xE8])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));

        cpu.register.x = 0x7F;

        cpu.reset();
        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }
}
