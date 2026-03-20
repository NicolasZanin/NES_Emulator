use crate::cpu::memory::Memory;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub status: u8,
    memory: Memory,
}

enum AddressingMode {
    Immediate,
    ZeroPage,
    Absolute,
}

impl CPU {
    pub fn new() -> Self {
        Self::new_mem(Memory::new())
    }

    pub fn new_mem(memory: Memory) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0,
            stack_pointer: 0,
            status: 0,
            memory,
        }
    }

    pub fn reset(&mut self) {
        let low_byte = self.memory.mem_read(0xFFFC) as u16;
        let high_byte = self.memory.mem_read(0xFFFD) as u16;

        let reset_byte = (high_byte << 8) | low_byte;
        self.program_counter = reset_byte;
        self.status = 0;
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.mem_read(self.program_counter);
        self.program_counter += 1;

        byte
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // zero flag
        if result == 0 {
            self.status |= 1 << 1;
        }
        else {
            self.status &= !(1 << 1);
        }

        // negative flag
        if (result >> 7) & 1 != 0  {
            self.status |= 1 << 7;
        }
        else {
            self.status &= !(1 << 7);
        }
    }

    fn get_operand_address(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                self.fetch_byte() as u16
            },
            AddressingMode::ZeroPage => {
                self.fetch_byte() as u16
            },
            AddressingMode::Absolute => {
                let low_byte = self.fetch_byte() as u16;
                let high_byte = self.fetch_byte() as u16;

                (high_byte << 8) | low_byte
            }
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_byte();

        match opcode {
            0xA9 => {
                let value = self.get_operand_address(AddressingMode::Immediate) as u8;
                self.register_a = value;
                self.update_zero_and_negative_flags(self.register_a);
            },
            0xA5 => {
                let op_address = self.get_operand_address(AddressingMode::ZeroPage);
                self.register_a = self.memory.mem_read(op_address);
                self.update_zero_and_negative_flags(self.register_a);
            },
            0xAD => {
                let op_address = self.get_operand_address(AddressingMode::Absolute);
                self.register_a = self.memory.mem_read(op_address);
                self.update_zero_and_negative_flags(self.register_a);
            },
            _ => panic!("This opcode is not supposed to be used."),
        }
    }
}
