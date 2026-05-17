use crate::cpu::cpu::{AddressingMode, CPU};
use crate::cpu::flags::Flag;

impl CPU {
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

    pub(crate) fn adc(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.add_with_carry(value);
    }

    pub(crate) fn sbc(&mut self, mode: AddressingMode) {
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

    pub(crate) fn cmp(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register.a);
    }

    pub(crate) fn cpx(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register.x);
    }

    pub(crate) fn cpy(&mut self, mode: AddressingMode) {
        self.compare(mode, self.register.y);
    }
}

#[cfg(test)]
mod tests_alu {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_adc_immediate() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x20)
            .load_program(&[0x69, 0x10])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x30);
    }

    #[test]
    fn test_adc_carry_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x02)
            .load_program(&[0x69, 0xFF])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_adc_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x69, 0x00])
            .build();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_adc_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[0x69, 0x80])
            .build();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_adc_overflow_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x50)
            .load_program(&[0x69, 0x50])
            .build();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::OVERFLOW));
    }

    #[test]
    fn test_adc_absolute_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x10)
            .set_register_x(0x01)
            .memory(0x2001, 0x05)
            .load_program(&[
                0x7D, // ADC Absolute,X
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x15);
    }

    #[test]
    fn test_adc_absolute_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x20)
            .set_register_y(0x02)
            .memory(0x2002, 0x10)
            .load_program(&[
                0x79, // ADC Absolute,Y
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x30);
    }

    #[test]
    fn test_adc_zero_page_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x05)
            .set_register_x(0x03)
            .memory(0x0013, 0x07)
            .load_program(&[
                0x75, // ADC ZeroPage,X
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x0C);
    }

    #[test]
    fn test_adc_indirect_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x10)
            .set_register_x(0x04)

            .memory(0x0024, 0x10)
            .memory(0x0025, 0x80)

            .memory(0x8010, 0x05)

            .load_program(&[
                0x61, // ADC ($20,X)
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x15);
    }

    #[test]
    fn test_adc_indirect_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x20)
            .set_register_y(0x04)

            .memory(0x0020, 0x00)
            .memory(0x0021, 0x80)

            .memory(0x8004, 0x10)

            .load_program(&[
                0x71, // ADC ($20),Y
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x30);
    }

    #[test]
    fn test_sbc_immediate_basic() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(10)
            .set_flags(Flag::CARRY)
            .load_program(&[0xE9, 0x03])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 7);
    }

    #[test]
    fn test_sbc_no_carry_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(10)
            .load_program(&[0xE9, 0x03])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 6);
    }

    #[test]
    fn test_sbc_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(5)
            .set_flags(Flag::CARRY)
            .load_program(&[0xE9, 0x05])
            .build();
        
        cpu.step();

        assert_eq!(cpu.register.a, 0);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_sbc_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(1)
            .set_flags(Flag::CARRY)
            .load_program(&[0xE9, 0x02])
            .build();
        
        cpu.step();

        assert_eq!(cpu.register.a, 0xFF);
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_sbc_carry_with_carry_flags() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(10)
            .set_flags(Flag::CARRY)
            .load_program(&[0xE9, 0x03])
            .build();
        
        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_sbc_overflow() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x80)
            .load_program(&[0xE9, 0x02])
            .build();
        
        cpu.step();

        assert!(cpu.status.get_flag(Flag::OVERFLOW));
    }

    #[test]
    fn test_sbc_absolute_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x20)
            .set_register_x(0x01)
            .memory(0x2001, 0x05)
            .load_program(&[
                0xFD, // SBC Absolute,X
                0x00,
                0x20,
            ])
            .build();

        cpu.status.set_flag(Flag::CARRY, true);

        cpu.step();

        assert_eq!(cpu.register.a, 0x1B);
    }

    #[test]
    fn test_sbc_absolute_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x30)
            .set_register_y(0x02)
            .memory(0x2002, 0x10)
            .load_program(&[
                0xF9, // SBC Absolute,Y
                0x00,
                0x20,
            ])
            .build();

        cpu.status.set_flag(Flag::CARRY, true);

        cpu.step();

        assert_eq!(cpu.register.a, 0x20);
    }

    #[test]
    fn test_sbc_zero_page_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x15)
            .set_register_x(0x03)
            .memory(0x0013, 0x05)
            .load_program(&[
                0xF5, // SBC ZeroPage,X
                0x10,
            ])
            .build();

        cpu.status.set_flag(Flag::CARRY, true);

        cpu.step();

        assert_eq!(cpu.register.a, 0x10);
    }

    #[test]
    fn test_sbc_indirect_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x20)
            .set_register_x(0x04)

            .memory(0x0024, 0x10)
            .memory(0x0025, 0x80)

            .memory(0x8010, 0x05)

            .load_program(&[
                0xE1, // SBC ($20,X)
                0x20,
            ])
            .build();

        cpu.status.set_flag(Flag::CARRY, true);

        cpu.step();

        assert_eq!(cpu.register.a, 0x1B);
    }

    #[test]
    fn test_sbc_indirect_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x30)
            .set_register_y(0x04)

            .memory(0x0020, 0x00)
            .memory(0x0021, 0x80)

            .memory(0x8004, 0x10)

            .load_program(&[
                0xF1, // SBC ($20),Y
                0x20,
            ])
            .build();

        cpu.status.set_flag(Flag::CARRY, true);

        cpu.step();

        assert_eq!(cpu.register.a, 0x20);
    }

    #[test]
    fn test_cmp_equal() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x05)
            .load_program(&[0xC9, 0x05])
            .build();
        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_more_than() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(10)
            .load_program(&[0xC9, 0x03])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_less_than() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x03)
            .load_program(&[0xC9, 0x05])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::ZERO));
        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_zeropage() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .load_program(&[0xC5, 0x10])
            .memory(0x0010, 0x42)
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cmp_absolute() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .load_program(&[0xCD, 0x34, 0x12])
            .memory(0x1234, 0x10)
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cmp_absolute_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .set_register_x(0x01)
            .memory(0x2001, 0x42)
            .load_program(&[
                0xDD, // CMP Absolute,X
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cmp_absolute_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x50)
            .set_register_y(0x02)
            .memory(0x2002, 0x40)
            .load_program(&[
                0xD9, // CMP Absolute,Y
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cmp_zero_page_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x10)
            .set_register_x(0x03)
            .memory(0x0013, 0x20)
            .load_program(&[
                0xD5, // CMP ZeroPage,X
                0x10,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cmp_indirect_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .set_register_x(0x04)

            .memory(0x0024, 0x10)
            .memory(0x0025, 0x80)

            .memory(0x8010, 0x42)

            .load_program(&[
                0xC1, // CMP ($20,X)
                0x20,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cmp_indirect_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x50)
            .set_register_y(0x04)

            .memory(0x0020, 0x00)
            .memory(0x0021, 0x80)

            .memory(0x8004, 0x40)

            .load_program(&[
                0xD1, // CMP ($20),Y
                0x20,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cpx_equal() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x08)
            .load_program(&[0xE0, 0x08])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cpx_less_than() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x01)
            .load_program(&[0xE0, 0x10])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_cpx_zeropage() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x33)
            .memory(0x0020, 0x33)
            .load_program(&[0xE4, 0x20])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpy_equal() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x09)
            .load_program(&[0xC0, 0x09])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_cpy_greater_than() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x05)
            .load_program(&[0xC0, 0x01])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::CARRY));
        assert!(!cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_cpy_absolute() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x01)
            .memory(0x2000, 0x02)
            .load_program(&[0xCC, 0x00, 0x20])
            .build();

        cpu.step();

        assert!(!cpu.status.get_flag(Flag::CARRY));
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

}
