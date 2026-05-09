#[cfg(test)]
use crate::cpu::cpu::CPU;
#[cfg(test)]
use crate::cpu::flags::{Flag, Flags};
#[cfg(test)]
use crate::cpu::memory::Memory;
#[cfg(test)]
use crate::cpu::registers::Registers;
#[cfg(test)]
use crate::cpu::stack::Stack;

#[cfg(test)]
const PROGRAM_START: u16 = 0x8000;

#[cfg(test)]
const STACK_START: u8 = 0xFF;

#[cfg(test)]
struct CPUState{
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8
}

#[cfg(test)]
impl Default for CPUState {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: PROGRAM_START,
            sp: STACK_START
        }
    }
}

#[cfg(test)]
pub struct CPUBuilder {
    memory: Memory,
    state: CPUState,
    flags: Flags,
}

#[cfg(test)]
impl CPUBuilder {
    pub fn new() -> Self {
        Self {
            state: CPUState::default(),
            memory: Memory::new(),
            flags: Flags::new(),
        }
    }

    // =========================
    // Registers
    // =========================

    pub fn set_register_a(mut self, value: u8) -> Self {
        self.state.a = value;
        self
    }

    pub fn set_register_x(mut self, value: u8) -> Self {
        self.state.x = value;
        self
    }

    pub fn set_register_y(mut self, value: u8) -> Self {
        self.state.y = value;
        self
    }

    pub fn set_program_counter(mut self, value: u16) -> Self {
        self.state.pc = value;
        self
    }

    pub fn set_stack_pointer(mut self, value: u8) -> Self {
        self.state.sp = value;
        self
    }

    pub fn set_flags(mut self, flag: Flag) -> Self {
        self.flags.set_flag(flag, true);
        self
    }

    // =========================
    // Memory helpers
    // =========================

    pub fn memory(mut self, addr: u16, value: u8) -> Self {
        self.memory.mem_write(addr, value);
        self
    }

    pub fn load_program(mut self, program: &[u8]) -> Self {
        self.memory.load_program(program);
        self
    }


    // =========================
    // Build CPU
    // =========================

    pub fn build(self, ) -> CPU {
        let cpu =  CPU {
            register: Registers {
                a: self.state.a,
                x: self.state.x,
                y: self.state.y,
            },
            program_counter: self.state.pc,
            stack: Stack::new_sp(self.state.sp),
            status: self.flags,
            memory: self.memory,
        };

        cpu
    }
}