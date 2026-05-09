use crate::cpu::flags::Flags;
use crate::cpu::memory::Memory;
use crate::cpu::registers::Registers;
use crate::cpu::stack::Stack;

// Struct and enums
pub struct CPU {
    pub(crate) program_counter: u16,
    pub(crate) register: Registers,
    pub(crate) stack: Stack,
    pub(crate) status: Flags,
    pub(crate) memory: Memory,
}

#[derive(Copy, Clone)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    Absolute,
    Indirect,
}

// Implementation
impl CPU {
    pub fn new(memory: Memory) -> Self {
        CPU {
            program_counter: 0,
            register: Registers::new(),
            stack: Stack::new(),
            status: Flags::new(),
            memory,
        }
    }

    pub fn reset(&mut self) {
        let low_byte = self.memory.mem_read(0xFFFC) as u16;
        let high_byte = self.memory.mem_read(0xFFFD) as u16;

        let reset_byte = (high_byte << 8) | low_byte;
        self.program_counter = reset_byte;
        self.status.reset();
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.mem_read(self.program_counter);
        self.program_counter += 1;

        byte
    }

    pub(crate) fn read_from_pc(&mut self) -> u16 {
        let low_byte = self.fetch_byte() as u16;
        let high_byte = self.fetch_byte() as u16;

        (high_byte << 8) | low_byte
    }

    pub(crate) fn read_from_memory(&mut self, addr: u16) -> u16 {
        let low_byte = self.memory.mem_read(addr) as u16;
        let high_byte = self.memory.mem_read(addr + 1) as u16;

        (high_byte << 8) | low_byte
    }

    pub(crate) fn get_operand_address(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.fetch_byte() as u16,
            AddressingMode::ZeroPage => self.fetch_byte() as u16,
            AddressingMode::Absolute => self.read_from_pc(),
            AddressingMode::Indirect => {
                let address_target = self.read_from_pc();
                self.read_from_memory(address_target)
            }
        }
    }

    pub(crate) fn get_operand_value(&mut self, mode: AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);

        match mode {
            AddressingMode::Immediate => addr as u8,
            _ => self.memory.mem_read(addr),
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_byte();

        match opcode {
            0xA9 => self.lda(AddressingMode::Immediate),
            0xA5 => self.lda(AddressingMode::ZeroPage),
            0xAD => self.lda(AddressingMode::Absolute),

            0x85 => self.sda(AddressingMode::ZeroPage),
            0x8D => self.sda(AddressingMode::Absolute),

            0xAA => self.tax(),

            0xE8 => self.inx(),

            0x69 => self.adc(AddressingMode::Immediate),
            0x65 => self.adc(AddressingMode::ZeroPage),
            0x6D => self.adc(AddressingMode::Absolute),

            0xE9 => self.sbc(AddressingMode::Immediate),
            0xE5 => self.sbc(AddressingMode::ZeroPage),
            0xED => self.sbc(AddressingMode::Absolute),

            0xC9 => self.cmp(AddressingMode::Immediate),
            0xC5 => self.cmp(AddressingMode::ZeroPage),
            0xCD => self.cmp(AddressingMode::Absolute),

            0xE0 => self.cpx(AddressingMode::Immediate),
            0xE4 => self.cpx(AddressingMode::ZeroPage),
            0xEC => self.cpx(AddressingMode::Absolute),

            0xC0 => self.cpy(AddressingMode::Immediate),
            0xC4 => self.cpy(AddressingMode::ZeroPage),
            0xCC => self.cpy(AddressingMode::Absolute),

            0x4C => self.jmp(AddressingMode::Absolute),
            0x6C => self.jmp(AddressingMode::Indirect),

            // Branching
            0xF0 => self.beq(),
            0xD0 => self.bne(),
            0x90 => self.bcc(),
            0xB0 => self.bcs(),
            0x50 => self.bvc(),
            0x70 => self.bvs(),
            0x30 => self.bmi(),
            0x10 => self.bpl(),

            // Stack
            0x48 => self.pha(),
            0x68 => self.pla(),
            0x08 => self.php(),
            0x28 => self.plp(),
            0x20 => self.jsr(),
            0x60 => self.rts(),

            0x00 => self.brk(),
            0x40 => self.rti(),

            _ => panic!("This opcode is not supposed to be used."),
        }
    }
}

// tests
#[cfg(test)]
mod tests_cpu {
    use crate::cpu::cpu_builder_test::CPUBuilder;

    #[test]
    fn test_example() {
        let cpu = CPUBuilder::new().build();
        assert_eq!(cpu.program_counter, 0x8000);
    }
}
