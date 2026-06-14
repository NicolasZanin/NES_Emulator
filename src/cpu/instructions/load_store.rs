use crate::cpu::cpu::{AddressingMode, CPU};

impl CPU {
    fn load(&mut self, mode: AddressingMode) -> u8 {
        let value = self.get_operand_value(mode);
        self.status.update_zero_and_negative_flags(value);

        value
    }

    fn store(&mut self, register: u8, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.memory.mem_write(address, register);
    }

    pub(crate) fn lda(&mut self, mode: AddressingMode) {
        self.register.a = self.load(mode);
    }

    pub(crate) fn ldx(&mut self, mode: AddressingMode) {
        self.register.x = self.load(mode);
    }

    pub(crate) fn ldy(&mut self, mode: AddressingMode) {
        self.register.y = self.load(mode);
    }

    pub(crate) fn sta(&mut self, mode: AddressingMode) {
        self.store(self.register.a, mode);
    }

    pub(crate) fn stx(&mut self, mode: AddressingMode) {
        self.store(self.register.x, mode);
    }

    pub(crate) fn sty(&mut self, mode: AddressingMode) {
        self.store(self.register.y, mode);
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
    fn test_ldx_immediate_loads_value() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[
                0xA2,
                0x42,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.x, 0x42);
    }

    #[test]
    fn test_ldx_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[
                0xA2,
                0x00,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_ldx_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[
                0xA2,
                0x80,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::NEGATIVE));
    }

    #[test]
    fn test_ldy_immediate_loads_value() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[
                0xA0,
                0x37,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.y, 0x37);
    }

    #[test]
    fn test_ldy_sets_zero_flag() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[
                0xA0,
                0x00,
            ])
            .build();

        cpu.step();

        assert!(cpu.status.get_flag(Flag::ZERO));
    }

    #[test]
    fn test_ldy_sets_negative_flag() {
        let mut cpu = CPUBuilder::new()
            .load_program(&[
                0xA0,
                0x80,
            ])
            .build();

        cpu.step();

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

    #[test]
    fn test_lda_absolute_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x01)
            .memory(0x2001, 0x42)
            .load_program(&[
                0xBD, // LDA Absolute,X
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x42);
    }

    #[test]
    fn test_lda_absolute_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x02)
            .memory(0x2002, 0x99)
            .load_program(&[
                0xB9, // LDA Absolute,Y
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x99);
    }

    #[test]
    fn test_lda_zero_page_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x05)
            .memory(0x0015, 0x77)
            .load_program(&[
                0xB5, // LDA ZeroPage,X
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x77);
    }

    #[test]
    fn test_lda_zero_page_x_wraps() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x20)
            .memory(0x0010, 0x55)
            .load_program(&[
                0xB5, // LDA ZeroPage,X
                0xF0,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x55);
    }

    #[test]
    fn test_lda_indirect_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x04)

            // pointer
            .memory(0x0024, 0x10)
            .memory(0x0025, 0x80)

            // final value
            .memory(0x8010, 0x42)

            .load_program(&[
                0xA1, // LDA ($20,X)
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x42);
    }

    #[test]
    fn test_lda_indirect_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x04)

            // base pointer
            .memory(0x0020, 0x00)
            .memory(0x0021, 0x80)

            // final value
            .memory(0x8004, 0x99)

            .load_program(&[
                0xB1, // LDA ($20),Y
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.register.a, 0x99);
    }

    #[test]
    fn test_sta_indirect_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .set_register_x(0x04)

            .memory(0x0024, 0x00)
            .memory(0x0025, 0x80)

            .load_program(&[
                0x81, // STA ($20,X)
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x8000), 0x42);
    }

    #[test]
    fn test_sta_indirect_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x99)
            .set_register_y(0x04)

            .memory(0x0020, 0x00)
            .memory(0x0021, 0x80)

            .load_program(&[
                0x91, // STA ($20),Y
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x8004), 0x99);
    }

    #[test]
    fn test_sta_absolute_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x42)
            .set_register_x(0x01)
            .load_program(&[
                0x9D, // STA Absolute,X
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x2001), 0x42);
    }

    #[test]
    fn test_sta_absolute_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x99)
            .set_register_y(0x02)
            .load_program(&[
                0x99, // STA Absolute,Y
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x2002), 0x99);
    }

    #[test]
    fn test_sta_zero_page_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_a(0x77)
            .set_register_x(0x05)
            .load_program(&[
                0x95, // STA ZeroPage,X
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x0015), 0x77);
    }

    #[test]
    fn test_stx_zeropage_stores_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0x55)
            .load_program(&[
                0x86,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0x55);
    }

    #[test]
    fn test_stx_absolute_stores_x() {
        let mut cpu = CPUBuilder::new()
            .set_register_x(0xAA)
            .load_program(&[
                0x8E,
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x2000), 0xAA);
    }

    #[test]
    fn test_sty_zeropage_stores_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0x77)
            .load_program(&[
                0x84,
                0x10,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x10), 0x77);
    }

    #[test]
    fn test_sty_absolute_stores_y() {
        let mut cpu = CPUBuilder::new()
            .set_register_y(0xCC)
            .load_program(&[
                0x8C,
                0x00,
                0x20,
            ])
            .build();

        cpu.step();

        assert_eq!(cpu.memory.mem_read(0x2000), 0xCC);
    }
}
