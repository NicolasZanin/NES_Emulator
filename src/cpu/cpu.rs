pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub status: u8,
    pub memory: [u8; 65536],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0,
            stack_pointer: 0,
            status: 0,
            memory: [0; 65536],
        }
    }

    pub fn reset(&mut self) {
        let low_byte = self.memory[0xFFFC] as u16;
        let high_byte = self.memory[0xFFFD] as u16;

        let reset_byte = (high_byte << 8) | low_byte;
        self.program_counter = reset_byte;
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        // convert in usize to read in memory
        self.memory[addr as usize]
    }

    pub fn fetch_byte(&mut self) -> u8 {
        let byte = self.mem_read(self.program_counter);
        self.program_counter += 1;

        byte
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_byte();

        match opcode {
            0xA9 => self.register_a = self.fetch_byte(),
            _ => panic!("This opcode is not supposed to be used."),
        }
    }
}