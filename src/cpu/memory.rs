pub struct Memory {
    pub memory: [u8; 0xFFFF],
}

impl Memory {
    pub fn new() -> Self {
        Memory { memory: [0; 0xFFFF] }
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        // convert in usize to read in memory
        self.memory[addr as usize]
    }
    
    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}