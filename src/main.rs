pub mod cpu;
pub mod opcodes;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

use cpu::CPU;

fn main() {
    let cpu = CPU::new();
}
