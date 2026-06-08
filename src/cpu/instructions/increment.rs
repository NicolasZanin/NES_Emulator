use crate::cpu::cpu::{AddressingMode, CPU};

impl CPU {
    const fn add_operation() -> fn(u8) -> u8 {
        | x: u8 | x.wrapping_add(1)
    }

    const fn remove_operation() -> fn(u8) -> u8 {
        | x: u8 | x.wrapping_sub(1)
    }

    fn update_value(&mut self, register: u8, operation: fn(u8) -> u8) -> u8 {
        let result = operation(register);

        self.status.update_zero_and_negative_flags(result);
        result
    }

    fn memory_op(&mut self, mode: AddressingMode, operation: fn(u8) -> u8) {
        let (memory_address, value) = self.get_operand(mode);
        let result = self.update_value(value, operation);

        match memory_address {
            Some(addr) => self.memory.mem_write(addr, result),
            None => panic!("Accumulator address is not allowed here"),
        }
    }

    pub(crate) fn inc(&mut self, mode: AddressingMode) {
        self.memory_op(mode, Self::add_operation());
    }

    pub(crate) fn inx(&mut self) {
        self.register.x = self.update_value(self.register.x, Self::add_operation());
    }

    pub(crate) fn iny(&mut self) {
        self.register.y = self.update_value(self.register.y, Self::add_operation());
    }

    pub(crate) fn dec(&mut self, mode: AddressingMode) {
        self.memory_op(mode, Self::remove_operation());
    }

    pub(crate) fn dex(&mut self) {
        self.register.x = self.update_value(self.register.x, Self::remove_operation());
    }

    pub(crate) fn dey(&mut self) {
        self.register.y = self.update_value(self.register.y, Self::remove_operation());
    }
}

#[cfg(test)]
mod tests_increment {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_inc_zeropage() {
        let mut cpu = CPUBuilder::new()
            .memory(0x10, 0x05)
            .load_program(&[
                0xE6,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0x06);
    }

    #[test]
    fn test_inc_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .memory(0x10, 0xFF)
            .load_program(&[
                0xE6,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0x00);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_inc_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .memory(0x10, 0x7F)
            .load_program(&[
                0xE6,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0x80);
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

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

    #[test]
    fn test_iny_increment_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x10)
            .load_program(&[0xC8])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0x11);
    }

    #[test]
    fn test_iny_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0xFF)
            .load_program(&[0xC8])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0x00);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_iny_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x7F)
            .load_program(&[0xC8])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0x80);
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_dec_zeropage() {
        let mut cpu = CPUBuilder::new()
            .memory(0x10, 0x05)
            .load_program(&[
                0xC6,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0x04);
    }

    #[test]
    fn test_dec_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .memory(0x10, 0x01)
            .load_program(&[
                0xC6,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0x00);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_dec_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .memory(0x10, 0x00)
            .load_program(&[
                0xC6,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0xFF);
        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_dex_decrement_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x10)
            .load_program(&[0xCA])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0x0F);
    }

    #[test]
    fn test_dex_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x01)
            .load_program(&[0xCA])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0x00);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_dex_wraps_underflow() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x00)
            .load_program(&[0xCA])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0xFF);
    }

    #[test]
    fn test_dex_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x00)
            .load_program(&[0xCA])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_dey_decrement_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x10)
            .load_program(&[0x88])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0x0F);
    }

    #[test]
    fn test_dey_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x01)
            .load_program(&[0x88])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0x00);
        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_dey_wraps_underflow() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x00)
            .load_program(&[0x88])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0xFF);
    }

    #[test]
    fn test_dey_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x00)
            .load_program(&[0x88])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }
}
