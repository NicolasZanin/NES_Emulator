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
    Implied,
    Relative,
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

#[derive(Copy, Clone)]
pub enum Instruction {
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    INC,
    INX,
    INY,
    DEC,
    DEX,
    DEY,
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,
    AND,
    ORA,
    EOR,
    BIT,
    ASL,
    LSR,
    ROL,
    ROR,
    JMP,
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,
    BEQ,
    BNE,
    BCC,
    BCS,
    BVC,
    BVS,
    BMI,
    BPL,
    PHA,
    PLA,
    PHP,
    PLP,
    JSR,
    RTS,
    BRK,
    RTI,
    NOP
}

#[derive(Copy, Clone)]
pub struct Opcode {
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub cycles: u8,
}


const ILLEGAL: Opcode = Opcode {
    instruction: Instruction::NOP,
    mode: AddressingMode::Implied,
    cycles: 0,
};

const fn op(instruction: Instruction, mode: AddressingMode, cycles: u8) -> Opcode {
    Opcode { instruction, mode, cycles }
}

const fn build_table() -> [Opcode; 256] {
    let mut table = [ILLEGAL; 256];

    table[0xA9] = op(Instruction::LDA, AddressingMode::Immediate, 2);
    table[0xA5] = op(Instruction::LDA, AddressingMode::ZeroPage, 3);
    table[0xB5] = op(Instruction::LDA, AddressingMode::ZeroPageX, 4);
    table[0xAD] = op(Instruction::LDA, AddressingMode::Absolute, 4);
    table[0xBD] = op(Instruction::LDA, AddressingMode::AbsoluteX, 4);
    table[0xB9] = op(Instruction::LDA, AddressingMode::AbsoluteY, 4);
    table[0xA1] = op(Instruction::LDA, AddressingMode::IndirectX, 6);
    table[0xB1] = op(Instruction::LDA, AddressingMode::IndirectY, 5);

    table[0xA2] = op(Instruction::LDX, AddressingMode::Immediate, 2);
    table[0xA6] = op(Instruction::LDX, AddressingMode::ZeroPage, 3);
    table[0xB6] = op(Instruction::LDX, AddressingMode::ZeroPageY, 4);
    table[0xAE] = op(Instruction::LDX, AddressingMode::Absolute, 4);
    table[0xBE] = op(Instruction::LDX, AddressingMode::AbsoluteY, 4);

    table[0xA0] = op(Instruction::LDY, AddressingMode::Immediate, 2);
    table[0xA4] = op(Instruction::LDX, AddressingMode::ZeroPage, 3);
    table[0xB4] = op(Instruction::LDX, AddressingMode::ZeroPageX, 4);
    table[0xAC] = op(Instruction::LDX, AddressingMode::Absolute, 4);
    table[0xBC] = op(Instruction::LDX, AddressingMode::AbsoluteX, 4);

    table[0x85] = op(Instruction::STA, AddressingMode::ZeroPage, 3);
    table[0x95] = op(Instruction::STA, AddressingMode::ZeroPageX, 4);
    table[0x8D] = op(Instruction::STA, AddressingMode::Absolute, 4);
    table[0x9D] = op(Instruction::STA, AddressingMode::AbsoluteX, 5);
    table[0x99] = op(Instruction::STA, AddressingMode::AbsoluteY, 5);
    table[0x81] = op(Instruction::STA, AddressingMode::IndirectX, 6);
    table[0x91] = op(Instruction::STA, AddressingMode::IndirectY, 6);

    table[0x86] = op(Instruction::STX, AddressingMode::ZeroPage, 3);
    table[0x96] = op(Instruction::STX, AddressingMode::ZeroPageY, 4);
    table[0x8E] = op(Instruction::STX, AddressingMode::Absolute, 4);

    table[0x84] = op(Instruction::STY, AddressingMode::ZeroPage, 3);
    table[0x94] = op(Instruction::STY, AddressingMode::ZeroPageX, 4);
    table[0x8C] = op(Instruction::STY, AddressingMode::Absolute, 4);

    table[0xAA] = op(Instruction::TAX, AddressingMode::Implied, 2);
    table[0xA8] = op(Instruction::TAY, AddressingMode::Implied, 2);
    table[0xBA] = op(Instruction::TSX, AddressingMode::Implied, 2);
    table[0x8A] = op(Instruction::TXA, AddressingMode::Implied, 2);
    table[0x9A] = op(Instruction::TXS, AddressingMode::Implied, 2);
    table[0x98] = op(Instruction::TYA, AddressingMode::Implied, 2);

    table[0xE6] = op(Instruction::INC, AddressingMode::ZeroPage, 5);
    table[0xF6] = op(Instruction::INC, AddressingMode::ZeroPageX, 6);
    table[0xEE] = op(Instruction::INC, AddressingMode::Absolute, 6);
    table[0xFE] = op(Instruction::INC, AddressingMode::AbsoluteX, 7);
    table[0xE8] = op(Instruction::INX, AddressingMode::Implied, 2);
    table[0xC8] = op(Instruction::INY, AddressingMode::Implied, 2);

    table[0xC6] = op(Instruction::DEC, AddressingMode::ZeroPage, 5);
    table[0xD6] = op(Instruction::DEC, AddressingMode::ZeroPageX, 6);
    table[0xCE] = op(Instruction::DEC, AddressingMode::Absolute, 6);
    table[0xDE] = op(Instruction::DEC, AddressingMode::AbsoluteX, 7);
    table[0xCA] = op(Instruction::DEX, AddressingMode::Implied, 2);
    table[0x88] = op(Instruction::DEY, AddressingMode::Implied, 2);

    table[0x69] = op(Instruction::ADC, AddressingMode::Immediate, 2);
    table[0x65] = op(Instruction::ADC, AddressingMode::ZeroPage, 3);
    table[0x75] = op(Instruction::ADC, AddressingMode::ZeroPageX, 4);
    table[0x6D] = op(Instruction::ADC, AddressingMode::Absolute, 4);
    table[0x7D] = op(Instruction::ADC, AddressingMode::AbsoluteX, 4);
    table[0x79] = op(Instruction::ADC, AddressingMode::AbsoluteY, 4);
    table[0x61] = op(Instruction::ADC, AddressingMode::IndirectX, 6);
    table[0x71] = op(Instruction::ADC, AddressingMode::IndirectY, 5);

    table[0xE9] = op(Instruction::SBC, AddressingMode::Immediate, 2);
    table[0xE5] = op(Instruction::SBC, AddressingMode::ZeroPage, 3);
    table[0xF5] = op(Instruction::SBC, AddressingMode::ZeroPageX, 4);
    table[0xED] = op(Instruction::SBC, AddressingMode::Absolute, 4);
    table[0xFD] = op(Instruction::SBC, AddressingMode::AbsoluteX, 4);
    table[0xF9] = op(Instruction::SBC, AddressingMode::AbsoluteY, 4);
    table[0xE1] = op(Instruction::SBC, AddressingMode::IndirectX, 6);
    table[0xF1] = op(Instruction::SBC, AddressingMode::IndirectY, 5);

    table[0xC9] = op(Instruction::CMP, AddressingMode::Immediate, 2);
    table[0xC5] = op(Instruction::CMP, AddressingMode::ZeroPage, 3);
    table[0xD5] = op(Instruction::CMP, AddressingMode::ZeroPageX, 4);
    table[0xCD] = op(Instruction::CMP, AddressingMode::Absolute, 4);
    table[0xDD] = op(Instruction::CMP, AddressingMode::AbsoluteX, 4);
    table[0xD9] = op(Instruction::CMP, AddressingMode::AbsoluteY, 4);
    table[0xC1] = op(Instruction::CMP, AddressingMode::IndirectX, 6);
    table[0xD1] = op(Instruction::CMP, AddressingMode::IndirectY, 5);

    table[0xE0] = op(Instruction::CPX, AddressingMode::Immediate, 2);
    table[0xE4] = op(Instruction::CPX, AddressingMode::ZeroPage, 3);
    table[0xEC] = op(Instruction::CPX, AddressingMode::Absolute, 4);

    table[0xC0] = op(Instruction::CPY, AddressingMode::Immediate, 2);
    table[0xC4] = op(Instruction::CPY, AddressingMode::ZeroPage, 3);
    table[0xCC] = op(Instruction::CPY, AddressingMode::Absolute, 4);

    table[0x29] = op(Instruction::AND, AddressingMode::Immediate, 2);
    table[0x25] = op(Instruction::AND, AddressingMode::ZeroPage, 3);
    table[0x35] = op(Instruction::AND, AddressingMode::ZeroPageX, 4);
    table[0x2D] = op(Instruction::AND, AddressingMode::Absolute, 4);
    table[0x3D] = op(Instruction::AND, AddressingMode::AbsoluteX, 4);
    table[0x39] = op(Instruction::AND, AddressingMode::AbsoluteY, 4);
    table[0x21] = op(Instruction::AND, AddressingMode::IndirectX, 6);
    table[0x31] = op(Instruction::AND, AddressingMode::IndirectY, 5);

    table[0x09] = op(Instruction::ORA, AddressingMode::Immediate, 2);
    table[0x05] = op(Instruction::ORA, AddressingMode::ZeroPage, 3);
    table[0x15] = op(Instruction::ORA, AddressingMode::ZeroPageX, 4);
    table[0x0D] = op(Instruction::ORA, AddressingMode::Absolute, 4);
    table[0x1D] = op(Instruction::ORA, AddressingMode::AbsoluteX, 4);
    table[0x19] = op(Instruction::ORA, AddressingMode::AbsoluteY, 4);
    table[0x01] = op(Instruction::ORA, AddressingMode::IndirectX, 6);
    table[0x11] = op(Instruction::ORA, AddressingMode::IndirectY, 5);

    table[0x49] = op(Instruction::EOR, AddressingMode::Immediate, 2);
    table[0x45] = op(Instruction::EOR, AddressingMode::ZeroPage, 3);
    table[0x55] = op(Instruction::EOR, AddressingMode::ZeroPageX, 4);
    table[0x4D] = op(Instruction::EOR, AddressingMode::Absolute, 4);
    table[0x5D] = op(Instruction::EOR, AddressingMode::AbsoluteX, 4);
    table[0x59] = op(Instruction::EOR, AddressingMode::AbsoluteY, 4);
    table[0x41] = op(Instruction::EOR, AddressingMode::IndirectX, 6);
    table[0x51] = op(Instruction::EOR, AddressingMode::IndirectY, 5);

    table[0x24] = op(Instruction::BIT, AddressingMode::ZeroPage, 3);
    table[0x2C] = op(Instruction::BIT, AddressingMode::Absolute, 4);

    table[0x0A] = op(Instruction::ASL, AddressingMode::Accumulator, 2);
    table[0x06] = op(Instruction::ASL, AddressingMode::ZeroPage, 5);
    table[0x16] = op(Instruction::ASL, AddressingMode::ZeroPageX, 6);
    table[0x0E] = op(Instruction::ASL, AddressingMode::Absolute, 6);
    table[0x1E] = op(Instruction::ASL, AddressingMode::AbsoluteX, 7);

    table[0x4A] = op(Instruction::LSR, AddressingMode::Accumulator, 2);
    table[0x46] = op(Instruction::LSR, AddressingMode::ZeroPage, 5);
    table[0x56] = op(Instruction::LSR, AddressingMode::ZeroPageX, 6);
    table[0x4E] = op(Instruction::LSR, AddressingMode::Absolute, 6);
    table[0x5E] = op(Instruction::LSR, AddressingMode::AbsoluteX, 7);

    table[0x2A] = op(Instruction::ROL, AddressingMode::Accumulator, 2);
    table[0x26] = op(Instruction::ROL, AddressingMode::ZeroPage, 5);
    table[0x36] = op(Instruction::ROL, AddressingMode::ZeroPageX, 6);
    table[0x2E] = op(Instruction::ROL, AddressingMode::Absolute, 6);
    table[0x3E] = op(Instruction::ROL, AddressingMode::AbsoluteX, 7);

    table[0x6A] = op(Instruction::ROR, AddressingMode::Accumulator, 2);
    table[0x66] = op(Instruction::ROR, AddressingMode::ZeroPage, 5);
    table[0x76] = op(Instruction::ROR, AddressingMode::ZeroPageX, 6);
    table[0x6E] = op(Instruction::ROR, AddressingMode::Absolute, 6);
    table[0x7E] = op(Instruction::ROR, AddressingMode::AbsoluteX, 7);

    table[0x4C] = op(Instruction::JMP, AddressingMode::Absolute, 3);
    table[0x6C] = op(Instruction::JMP, AddressingMode::Indirect, 5);

    table[0x18] = op(Instruction::CLC, AddressingMode::Implied, 2);
    table[0xD8] = op(Instruction::CLD, AddressingMode::Implied, 2);
    table[0x58] = op(Instruction::CLI, AddressingMode::Implied, 2);
    table[0xB8] = op(Instruction::CLV, AddressingMode::Implied, 2);
    table[0x38] = op(Instruction::SEC, AddressingMode::Implied, 2);
    table[0xF8] = op(Instruction::SED, AddressingMode::Implied, 2);
    table[0x78] = op(Instruction::SEI, AddressingMode::Implied, 2);

    table[0xF0] = op(Instruction::BEQ, AddressingMode::Relative, 2);
    table[0xD0] = op(Instruction::BNE, AddressingMode::Relative, 2);
    table[0x90] = op(Instruction::BCC, AddressingMode::Relative, 2);
    table[0xB0] = op(Instruction::BCS, AddressingMode::Relative, 2);
    table[0x50] = op(Instruction::BVC, AddressingMode::Relative, 2);
    table[0x70] = op(Instruction::BVS, AddressingMode::Relative, 2);
    table[0x30] = op(Instruction::BMI, AddressingMode::Relative, 2);
    table[0x10] = op(Instruction::BPL, AddressingMode::Relative, 2);

    table[0x48] = op(Instruction::PHA, AddressingMode::Implied, 3);
    table[0x68] = op(Instruction::PLA, AddressingMode::Implied, 4);
    table[0x08] = op(Instruction::PHP, AddressingMode::Implied, 3);
    table[0x28] = op(Instruction::PLP, AddressingMode::Implied, 4);

    table[0x20] = op(Instruction::JSR, AddressingMode::Absolute, 6);
    table[0x60] = op(Instruction::RTS, AddressingMode::Implied, 6);

    table[0x00] = op(Instruction::BRK, AddressingMode::Implied, 7);
    table[0x40] = op(Instruction::RTI, AddressingMode::Implied, 6);
    table[0xEA] = op(Instruction::RTI, AddressingMode::Implied, 2);


    table
}

pub static OPCODES: [Opcode; 256] = build_table();


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
            AddressingMode::Implied => panic!("Impossible to get operand address of accumulator"),
            AddressingMode::Relative => panic!("Impossible to get operand address of accumulator"),
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

    pub(crate) fn get_operand(&mut self, mode: AddressingMode) -> (Option<u16>, u8) {
        if mode == AddressingMode::Accumulator {
            return (None, self.register.a);
        }

        let addr = self.get_operand_address(mode);
        (Some(addr), self.memory.mem_read(addr))
    }

    pub fn step(&mut self) {
        let byte = self.fetch_byte();
        let opcode = OPCODES[byte as usize];

        match opcode.instruction {
            Instruction::LDA => self.lda(opcode.mode),
            Instruction::LDX => self.ldx(opcode.mode),
            Instruction::LDY => self.ldy(opcode.mode),
            Instruction::STA => self.sta(opcode.mode),
            Instruction::STX => self.stx(opcode.mode),
            Instruction::STY => self.sty(opcode.mode),
            Instruction::TAX => self.tax(),
            Instruction::TAY => self.tay(),
            Instruction::TSX => self.tsx(),
            Instruction::TXA => self.txa(),
            Instruction::TXS => self.txs(),
            Instruction::TYA => self.tya(),
            Instruction::INC => self.inc(opcode.mode),
            Instruction::INX => self.inx(),
            Instruction::INY => self.iny(),
            Instruction::DEC => self.dec(opcode.mode),
            Instruction::DEX => self.dex(),
            Instruction::DEY => self.dey(),
            Instruction::ADC => self.adc(opcode.mode),
            Instruction::SBC => self.sbc(opcode.mode),
            Instruction::CMP => self.cmp(opcode.mode),
            Instruction::CPX => self.cpx(opcode.mode),
            Instruction::CPY => self.cpy(opcode.mode),
            Instruction::AND => self.and(opcode.mode),
            Instruction::ORA => self.ora(opcode.mode),
            Instruction::EOR => self.eor(opcode.mode),
            Instruction::BIT => self.bit(opcode.mode),
            Instruction::ASL => self.asl(opcode.mode),
            Instruction::LSR => self.lsr(opcode.mode),
            Instruction::ROL => self.rol(opcode.mode),
            Instruction::ROR => self.ror(opcode.mode),
            Instruction::JMP => self.jmp(opcode.mode),
            Instruction::CLC => self.clc(),
            Instruction::CLD => self.cld(),
            Instruction::CLI => self.cli(),
            Instruction::CLV => self.clv(),
            Instruction::SEC => self.sec(),
            Instruction::SED => self.sed(),
            Instruction::SEI => self.sei(),
            Instruction::BEQ => self.beq(),
            Instruction::BNE => self.bne(),
            Instruction::BCC => self.bcc(),
            Instruction::BCS => self.bcs(),
            Instruction::BVC => self.bvc(),
            Instruction::BVS => self.bvs(),
            Instruction::BMI => self.bmi(),
            Instruction::BPL => self.bpl(),
            Instruction::PHA => self.pha(),
            Instruction::PLA => self.pla(),
            Instruction::PHP => self.php(),
            Instruction::PLP => self.plp(),
            Instruction::JSR => self.jsr(),
            Instruction::RTS => self.rts(),
            Instruction::BRK => self.brk(),
            Instruction::RTI => self.rti(),
            Instruction::NOP => {},
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
