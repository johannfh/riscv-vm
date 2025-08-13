#![allow(dead_code)]

use std::fs;

use clap::Parser;

use crate::cpu::Cpu;

#[macro_use]
extern crate log;

mod cpu;
mod utils;

#[derive(Parser)]
struct Args {
    /// Path to the program file
    #[clap(short, long)]
    program: Option<String>,
}

fn main() {
    env_logger::builder().parse_env("LOG").init();

    let args = Args::parse();

    let program: Vec<u8> = if let Some(program_path) = &args.program {
        info!("Loading program from: {}", program_path);
        fs::read(program_path).expect("Failed to read the program file")
    } else {
        info!("No program specified, using default program.");
        [
            0x93, 0x00, 0x30, 0x00, // addi x1, x0, 3
            0x93, 0x80, 0x10, 0x00, // addi x1, x1, 1 (x1 = 3 + 1 = 4)
            0x93, 0x80, 0x10, 0x00, // addi x1, x1, 1 (x1 = 4 + 1 = 5)
            0x93, 0x80, 0x10, 0x00, // addi x1, x1, 1 (x1 = 5 + 1 = 6)
            // mov x2, x1
            0x13, 0x81, 0x00, 0x00, // addi x2, x1, 0 (move x1 to x2)
        ]
        .to_vec()
    };

    let mut cpu = Cpu::with_program(&program);
    for _ in 0..program.len() / 4 {
        cpu.tick();
    }
}
