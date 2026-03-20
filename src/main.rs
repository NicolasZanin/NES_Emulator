use crate::cpu::cpu::CPU;

mod cpu;

fn main() {
    let mut cpu = CPU::new();
    cpu.memory[0xFFFC] = 0x00;
    cpu.memory[0xFFFD] = 0x80;

    cpu.memory[0x8000] = 0xA9;
    cpu.memory[0x8001] = 42;

    cpu.reset();
    cpu.step();
    println!("A = {}", cpu.register_a);
}
