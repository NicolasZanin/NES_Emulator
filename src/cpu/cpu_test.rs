#[cfg(test)]
mod tests_cpu {
    use crate::cpu::cpu::CPU;
    use crate::cpu::memory::Memory;

    #[test]
    fn test_example() {
        let mut memory = Memory::new();
        memory.mem_write(0xFFFC, 0x00);
        memory.mem_write(0xFFFD, 0x80);


        let mut cpu = CPU::new_mem(memory);
        cpu.reset();
        assert_eq!(cpu.program_counter, 0x8000);
    }

    #[test]
    fn test_lda_immediate_sets_register_a_and_flags() {
        let mut memory = Memory::new();
        memory.mem_write(0xFFFC, 0x00);
        memory.mem_write(0xFFFD, 0x80);
        memory.mem_write(0x8000, 0xA9);
        memory.mem_write(0x8001, 0x42);

        let mut cpu = CPU::new_mem(memory);

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_a, 0x42);

        assert_eq!(cpu.status & (1 << 1), 0);   // Zero flag
        assert_eq!(cpu.status & (1 << 7), 0);   // Negative flag
    }

    #[test]
    fn test_lda_zero_page() {
        let mut memory = Memory::new();
        memory.mem_write(0xFFFC, 0x00);
        memory.mem_write(0xFFFD, 0x80);

        // LDA $10
        memory.mem_write(0x8000, 0xA5);
        memory.mem_write(0x8001, 0x10);

        memory.mem_write(0x0010, 0x99);

        let mut cpu = CPU::new_mem(memory);

        cpu.reset();
        cpu.step();

        assert_eq!(cpu.register_a, 0x99);
        assert_eq!(cpu.status & (1 << 1), 0);   // Zero flag
        assert_eq!(cpu.status & (1 << 7), 1 << 7); // Negative flag si bit 7 = 1 (0x99 = 0b10011001)
    }
}