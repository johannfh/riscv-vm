#![allow(dead_code)]

use std::fs;

use bytesize::ByteSize;
use clap::Parser;

use crate::cpu::Cpu;

#[macro_use]
extern crate log;

mod constants;
mod cpu;
mod monitored_memory;
mod utils;

#[derive(Parser)]
struct Args {
    /// Path to the program file, example: path/to/program.bin
    #[clap(short, long)]
    program: String,
    /// Optional memory size in bytes, if not provided, defaults to 1 MiB
    memory: Option<usize>,
}

fn main() {
    env_logger::builder().parse_env("LOG").init();

    let args = Args::parse();

    let program_path = std::path::Path::new(&args.program);

    info!("Loading program from: {}", program_path.display());
    let program: Vec<u8> = fs::read(program_path).expect("Failed to read the program file");

    if let Some(memory_size) = args.memory {
        info!("Using custom memory size: {} bytes", memory_size);
        //monitored_allocator::MonitoredAllocator::new(memory_size);
    } else {
        info!("Using default memory size: 1 MiB");
        //monitored_allocator::MonitoredAllocator::new(1024 * 1024); // Default to 1 MiB
    }

    assert!(
        program.len() % 4 == 0,
        "Program length must be a multiple of 4 bytes (world size)!"
    );

    info!(
        "Program loaded successfully, length: {}",
        ByteSize(program.len() as u64)
    );

    let mut cpu = Cpu::with_program(&program).expect("Failed to create CPU with program");
    info!(
        "CPU initialized with memory size: {} bytes",
        ByteSize(cpu.memory_size() as u64)
    );
    loop {
        cpu.tick();
    }
}
