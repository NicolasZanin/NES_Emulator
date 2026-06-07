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

#[derive(Copy, Clone, PartialEq)]
pub enum AddressingMode {
    Immediate,
    Accumulator,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
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
            AddressingMode::Accumulator => panic!("Impossible to get operand address of accumulator"),
            AddressingMode::Immediate => self.fetch_byte() as u16,
            AddressingMode::ZeroPage => self.fetch_byte() as u16,
            AddressingMode::ZeroPageX => {
                let mem = self.fetch_byte();
                mem.wrapping_add(self.register.x) as u16
            },
            AddressingMode::ZeroPageY => {
                let mem = self.fetch_byte();
                mem.wrapping_add(self.register.y) as u16
            },
            AddressingMode::Absolute => self.read_from_pc(),
            AddressingMode::AbsoluteX => {
                self.read_from_pc() + self.register.x as u16
            },
            AddressingMode::AbsoluteY => {
                self.read_from_pc() + self.register.y as u16
            },
            AddressingMode::Indirect => {
                let address_target = self.read_from_pc();
                self.read_from_memory(address_target)
            },
            AddressingMode::IndirectX => {
                let byte = self.fetch_byte();
                let address_target = byte.wrapping_add(self.register.x) as u16;
                self.read_from_memory(address_target)
            },
            AddressingMode::IndirectY => {
                let address_target = self.fetch_byte() as u16;
                let value = self.read_from_memory(address_target);
                value + self.register.y as u16
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
            0xB5 => self.lda(AddressingMode::ZeroPageX),
            0xAD => self.lda(AddressingMode::Absolute),
            0xBD => self.lda(AddressingMode::AbsoluteX),
            0xB9 => self.lda(AddressingMode::AbsoluteY),
            0xA1 => self.lda(AddressingMode::IndirectX),
            0xB1 => self.lda(AddressingMode::IndirectY),

            0x85 => self.sda(AddressingMode::ZeroPage),
            0x95 => self.sda(AddressingMode::ZeroPageX),
            0x8D => self.sda(AddressingMode::Absolute),
            0x9D => self.sda(AddressingMode::AbsoluteX),
            0x99 => self.sda(AddressingMode::AbsoluteY),
            0x81 => self.sda(AddressingMode::IndirectX),
            0x91 => self.sda(AddressingMode::IndirectY),

            0xAA => self.tax(),

            0xE8 => self.inx(),

            // ALU
            0x69 => self.adc(AddressingMode::Immediate),
            0x65 => self.adc(AddressingMode::ZeroPage),
            0x75 => self.adc(AddressingMode::ZeroPageX),
            0x6D => self.adc(AddressingMode::Absolute),
            0x7D => self.adc(AddressingMode::AbsoluteX),
            0x79 => self.adc(AddressingMode::AbsoluteY),
            0x61 => self.adc(AddressingMode::IndirectX),
            0x71 => self.adc(AddressingMode::IndirectY),

            0xE9 => self.sbc(AddressingMode::Immediate),
            0xE5 => self.sbc(AddressingMode::ZeroPage),
            0xF5 => self.sbc(AddressingMode::ZeroPageX),
            0xED => self.sbc(AddressingMode::Absolute),
            0xFD => self.sbc(AddressingMode::AbsoluteX),
            0xF9 => self.sbc(AddressingMode::AbsoluteY),
            0xE1 => self.sbc(AddressingMode::IndirectX),
            0xF1 => self.sbc(AddressingMode::IndirectY),

            0xC9 => self.cmp(AddressingMode::Immediate),
            0xC5 => self.cmp(AddressingMode::ZeroPage),
            0xD5 => self.cmp(AddressingMode::ZeroPageX),
            0xCD => self.cmp(AddressingMode::Absolute),
            0xDD => self.cmp(AddressingMode::AbsoluteX),
            0xD9 => self.cmp(AddressingMode::AbsoluteY),
            0xC1 => self.cmp(AddressingMode::IndirectX),
            0xD1 => self.cmp(AddressingMode::IndirectY),

            0xE0 => self.cpx(AddressingMode::Immediate),
            0xE4 => self.cpx(AddressingMode::ZeroPage),
            0xEC => self.cpx(AddressingMode::Absolute),

            0xC0 => self.cpy(AddressingMode::Immediate),
            0xC4 => self.cpy(AddressingMode::ZeroPage),
            0xCC => self.cpy(AddressingMode::Absolute),

            0x29 => self.and(AddressingMode::Immediate),
            0x25 => self.and(AddressingMode::ZeroPage),
            0x35 => self.and(AddressingMode::ZeroPageX),
            0x2D => self.and(AddressingMode::Absolute),
            0x3D => self.and(AddressingMode::AbsoluteX),
            0x39 => self.and(AddressingMode::AbsoluteY),
            0x21 => self.and(AddressingMode::IndirectX),
            0x31 => self.and(AddressingMode::IndirectY),

            0x09 => self.ora(AddressingMode::Immediate),
            0x05 => self.ora(AddressingMode::ZeroPage),
            0x15 => self.ora(AddressingMode::ZeroPageX),
            0x0D => self.ora(AddressingMode::Absolute),
            0x1D => self.ora(AddressingMode::AbsoluteX),
            0x19 => self.ora(AddressingMode::AbsoluteY),
            0x01 => self.ora(AddressingMode::IndirectX),
            0x11 => self.ora(AddressingMode::IndirectY),

            0x49 => self.eor(AddressingMode::Immediate),
            0x45 => self.eor(AddressingMode::ZeroPage),
            0x55 => self.eor(AddressingMode::ZeroPageX),
            0x4D => self.eor(AddressingMode::Absolute),
            0x5D => self.eor(AddressingMode::AbsoluteX),
            0x59 => self.eor(AddressingMode::AbsoluteY),
            0x41 => self.eor(AddressingMode::IndirectX),
            0x51 => self.eor(AddressingMode::IndirectY),

            0x24 => self.bit(AddressingMode::ZeroPage),
            0x2C => self.bit(AddressingMode::Absolute),

            0x0A => self.asl(AddressingMode::Accumulator),
            0x06 => self.asl(AddressingMode::ZeroPage),
            0x16 => self.asl(AddressingMode::ZeroPageX),
            0x0E => self.asl(AddressingMode::Absolute),
            0x1E => self.asl(AddressingMode::AbsoluteX),

            0x4A => self.lsr(AddressingMode::Accumulator),
            0x46 => self.lsr(AddressingMode::ZeroPage),
            0x56 => self.lsr(AddressingMode::ZeroPageX),
            0x4E => self.lsr(AddressingMode::Absolute),
            0x5E => self.lsr(AddressingMode::AbsoluteX),

            0x2A => self.rol(AddressingMode::Accumulator),
            0x26 => self.rol(AddressingMode::ZeroPage),
            0x36 => self.rol(AddressingMode::ZeroPageX),
            0x2E => self.rol(AddressingMode::Absolute),
            0x3E => self.rol(AddressingMode::AbsoluteX),

            0x6A => self.ror(AddressingMode::Accumulator),
            0x66 => self.ror(AddressingMode::ZeroPage),
            0x76 => self.ror(AddressingMode::ZeroPageX),
            0x6E => self.ror(AddressingMode::Absolute),
            0x7E => self.ror(AddressingMode::AbsoluteX),

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
