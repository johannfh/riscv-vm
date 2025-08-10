#![allow(dead_code)]

use crate::cpu::Cpu;

#[macro_use]
extern crate log;

#[cfg(not(target_pointer_width = "64"))]
compile_error!("This CPU implementation requires a 64-bit target architecture.");

mod cpu;
mod utils;

fn main() {
    env_logger::builder().parse_env("LOG").init();

    let program = [
        0x93, 0x00, 0x30, 0x00, // addi x1, x0, 3
        0x93, 0x80, 0x10, 0x00, // addi x1, x1, 1 (x1 = 3 + 1 = 4)
        0x93, 0x80, 0x10, 0x00, // addi x1, x1, 1 (x1 = 4 + 1 = 5)
        0x93, 0x80, 0x10, 0x00, // addi x1, x1, 1 (x1 = 5 + 1 = 6)
        // mov x2, x1
        0b00010011, 0b10000001, 0x00, 0x00, // addi x2, x1, 0 (move x1 to x2)
    ];

    let mut cpu = Cpu::with_program(&program);
    for _ in 0..program.len() / 4 {
        cpu.tick();
    }
}
