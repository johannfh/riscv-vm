#![allow(dead_code)]

use std::fs;

use bytesize::ByteSize;
use clap::Parser;

use crate::{app::App, cpu::Cpu};

use log::info;

mod app;
mod constants;
mod cpu;
mod monitored_memory;
mod utils;

#[derive(Clone, Parser)]
struct Args {
    /// Path to the program file, example: path/to/program.bin
    #[clap(short, long)]
    program: String,
    /// Optional memory size in bytes, if not provided, defaults to 1 MiB
    memory: Option<usize>,
}

#[macroquad::main("RISC-V Virtual Machine")]
async fn main() {
    env_logger::builder().parse_env("LOG").init();
    let args = Args::parse();

    let program_path = std::path::Path::new(&args.program);

    info!("Loading program from: {}", program_path.display());
    let program: Vec<u8> = fs::read(program_path).expect("Failed to read the program file");

    let memory_size = if let Some(memory_size) = args.memory {
        info!("Using custom memory size: {} bytes", memory_size);
        memory_size
    } else {
        info!("Using default memory size: 1 MiB");
        1024 * 1024
    };

    info!(
        "Program loaded successfully, size: {}",
        ByteSize(program.len() as u64)
    );

    let (mut cpu, cpu_events) =
        Cpu::new(memory_size).expect("Failed to create CPU with memory size");
    info!(
        "CPU initialized with memory size: {} bytes",
        ByteSize(cpu.memory_size() as u64)
    );

    cpu.load_program(&program)
        .expect("Failed to create CPU with program");

    info!("Starting CPU execution...");

    let cpu_thread = std::thread::Builder::new()
        .name("virtual_machine".to_string())
        .spawn(move || {
            while !cpu.is_halted() {
                cpu.tick();
            }
        })
        .expect("Failed to spawn VM thread");

    let mut app = App::new(cpu_events);
    app.run().await;
    info!("Virtual machine execution completed.");

    cpu_thread.join().expect("Failed to join VM thread");
    info!("Exiting application in 5 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(5));
}
