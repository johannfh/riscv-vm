#![allow(dead_code)]

use std::fs;

use bytesize::ByteSize;
use clap::Parser;

use crate::cpu::Cpu;

#[macro_use]
extern crate log;

mod cpu;
mod utils;
mod constants;

#[derive(Parser)]
struct Args {
    /// Path to the program file, example: path/to/program.bin
    #[clap(short, long)]
    program: String,
}

fn main() {
    env_logger::builder().parse_env("LOG").init();

    let args = Args::parse();

    let program_path = std::path::Path::new(&args.program);

    info!("Loading program from: {}", program_path.display());
    let program: Vec<u8> = fs::read(program_path).expect("Failed to read the program file");

    assert!(program.len() % 4 == 0, "Program length must be a multiple of 4 bytes (world size)!");

    info!("Size of program: {}", ByteSize(program.len() as u64));

    let mut cpu = Cpu::with_program(&program);
    for _ in 0..program.len() / 4 {
        cpu.tick();
    }
}
