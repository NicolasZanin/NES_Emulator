pub struct Memory {
    pub memory: [u8; 0x10000],
}

impl Memory {
    pub fn new() -> Self {
        Memory { memory: [0; 0x10000] }
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        // convert in usize to read in memory
        self.memory[addr as usize]
    }
    
    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

#[cfg(test)]
impl Memory {
    pub fn load_program(&mut self, program: &[u8]) {
        let start = 0x8000;

        for (i, byte) in program.iter().enumerate() {
            self.mem_write(start + i as u16, *byte);
        }

        self.mem_write(0xFFFC, 0x00);
        self.mem_write(0xFFFD, 0x80);
    }
}