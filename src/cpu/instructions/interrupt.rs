use crate::cpu::cpu::{CPU};
use crate::cpu::flags::Flag;

impl CPU {
    pub(crate) fn brk(&mut self) {
        self.stack.write_address(&mut self.memory, self.program_counter);

        self.status.set_flag(Flag::BREAK, true);
        self.status.set_flag(Flag::INTERRUPT, true);
        self.stack.write_value(&mut self.memory, self.status.get_status());

        self.program_counter = self.read_from_memory(0xFFFE);
    }

    pub(crate) fn rti(&mut self) {
        let status = self.stack.read_value(&mut self.memory);
        let previous_address = self.stack.read_address(&mut self.memory);

        self.status.set_status(status);
        self.program_counter = previous_address;
    }
}

#[cfg(test)]
mod tests_interrupt {
    use crate::cpu::flags::Flag;
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_brk_pushes_pc_and_status() {
        let mut cpu = CPUBuilder::new()
            .memory(0xFFFE, 0x00)
            .memory(0xFFFF, 0x90)
            .build();

        cpu.step();

        // PC + 1 = 0x8001
        assert_eq!(cpu.memory.mem_read(0x01FF), 0x80);
        assert_eq!(cpu.memory.mem_read(0x01FE), 0x01);

        assert_eq!(cpu.stack.get(), 0xFC);
        assert_eq!(cpu.program_counter, 0x9000);
    }

    #[test]
    fn test_rti_restores_pc_and_status() {
        let mut cpu = CPUBuilder::new()
            .set_stack_pointer(0xFC)
            .memory(0x01FD, 0b00000001)// status
            .memory(0x01FE, 0x05) // PC = 0x8005
            .memory(0x01FF, 0x80)
            .load_program(&[0x40])
            .build();

        cpu.step();

        assert_eq!(cpu.program_counter, 0x8005);
        assert!(cpu.status.get_flag(Flag::CARRY));
    }

    #[test]
    fn test_brk_rti_cycle() {
        let mut cpu = CPUBuilder::new()
            .memory(0x9000, 0x40)// RTI
            .memory(0xFFFE, 0x00)
            .memory(0xFFFF, 0x90)
            .build();

        cpu.step(); // BRK
        cpu.step(); // RTI

        assert_eq!(cpu.program_counter, 0x8001);
    }
}
