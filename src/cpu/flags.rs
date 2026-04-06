use bitflags::bitflags;

pub struct Flags {
    status: u8
}

bitflags! {
    pub struct Flag: u8 {
        const CARRY     = 1 << 0;
        const ZERO      = 1 << 1;
        const INTERRUPT = 1 << 2;
        const DECIMAL   = 1 << 3;
        const BREAK     = 1 << 4;
        const OVERFLOW  = 1 << 6;
        const NEGATIVE  = 1 << 7;
    }
}


impl Flags {
    pub fn new() -> Self {
        Flags {
            status: 0
        }
    }

    pub fn reset(&mut self) {
        self.status = 0;
    }

    pub fn get_status(&self) -> u8 {
        self.status
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.status |= flag.bits();
        }
        else {
            self.status &= !flag.bits();
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        self.status & flag.bits() != 0
    }

    pub fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.set_flag(Flag::ZERO, result == 0);
        self.set_flag(Flag::NEGATIVE, result & Flag::NEGATIVE.bits() != 0);
    }

    pub fn update_carry_flags(&mut self, result: u16) {
        self.set_flag(Flag::CARRY, result > 0xFF);
    }

    pub fn update_overflow_flags(&mut self, value: bool) {
        self.set_flag(Flag::OVERFLOW, value);
    }
}