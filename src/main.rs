#![allow(dead_code)]
#![feature(allocator_api)]

use std::fs;

use bytesize::ByteSize;
use clap::Parser;

use crate::cpu::Cpu;

#[macro_use]
extern crate log;

mod constants;
mod cpu;
mod monitored_allocator;
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
        monitored_allocator::MonitoredAllocator::new(memory_size);
    } else {
        info!("Using default memory size: 1 MiB");
        monitored_allocator::MonitoredAllocator::new(1024 * 1024); // Default to 1 MiB
    }

    assert!(
        program.len() % 4 == 0,
        "Program length must be a multiple of 4 bytes (world size)!"
    );

    monitored_allocator_example();

    info!(
        "Program loaded successfully, length: {}",
        ByteSize(program.len() as u64)
    );

    let mut cpu = Cpu::with_program(&program);
    loop {
        cpu.tick();
    }
}

fn monitored_allocator_example() {
    use monitored_allocator::MonitoredAllocator;

    // allocate a monitored vector to demonstrate the allocator
    let monitored_allocator = MonitoredAllocator::new(1024 * 1024); // 1 MiB limit

    let mut monitored_vec: Vec<u64, _> = Vec::new_in(&monitored_allocator);
    info!(
        "Allocated monitored vector. Usage: {} B",
        monitored_allocator.allocated()
    );

    monitored_vec.reserve_exact(1);
    info!(
        "Usage after reserving space for one element: {} B",
        monitored_allocator.allocated()
    );

    monitored_vec.push(0);
    info!(
        "Usage after pushing one element: {} B",
        monitored_allocator.allocated()
    );

    monitored_vec.push(1);
    info!(
        "Usage after pushing another element: {} B",
        monitored_allocator.allocated()
    );

    println!("Monitored vector contents: {:?}", monitored_vec);
}
