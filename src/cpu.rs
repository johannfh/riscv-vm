use std::io::{self, Write};

use derive_more::Display;
use goblin::elf::Elf;
use log::{info, trace};

use crate::{
    constants::{a0, a7},
    format_u32_le_bits,
    monitored_memory::MonitoredMemory,
    utils::sign_extend_u64_to_i64,
};

pub struct Cpu {
    /// # General Purpose Registers
    ///
    /// The general purpose registers (GPRs) x0 to x31 are used for arithmetic
    /// and logical operations, as well as for holding temporary data.
    ///
    /// ## Usage
    ///
    /// - `x0` is always zero and cannot be modified. Thus it is often called the zero register
    /// - `x1` to `x31` can be used for various purposes, such as holding function arguments,
    ///    By convention, `x1` is used for the return address.
    pub gprs: [u64; 32],

    /// # Program Counter
    ///
    /// The program counter (PC) register holds the address of the next instruction to be executed.
    pub pc: u64,

    pub memory: MonitoredMemory,

    pub is_running: bool,
    pub exit_code: i32,
    cpu_events: crossbeam::channel::Sender<CpuEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum CpuEvent {
    DrawCharacter { character: char },
    Exit { exit_code: i32 },
}

impl Cpu {
    // TODO: Use virtual addresses and either 0x8000_0000 as the start address for the program
    // or allow loading programs with arbitrary start addresses. Obtaining the start address
    // from the ELF header seems to be a good idea.
    const START_ADDRESS: u64 = 0x0000_0000;
    /// The size of a word in bytes. For RISC-V, this is typically 4 bytes (32 bits).
    const WORD_SIZE: u64 = 4;

    /// Creates a new CPU instance with all registers initialized to zero.
    pub fn new(memory_size: usize) -> io::Result<(Self, crossbeam::channel::Receiver<CpuEvent>)> {
        let memory = MonitoredMemory::new(memory_size)?;
        let (send, recv) = crossbeam::channel::unbounded();

        Ok((
            Cpu {
                gprs: [0; 32],
                pc: Self::START_ADDRESS,
                memory,
                is_running: true,
                exit_code: 0,
                cpu_events: send,
            },
            recv,
        ))
    }

    pub fn load_program(&mut self, program: &[u8]) -> anyhow::Result<()> {
        // Load the program into memory starting at address 0x8000_0000
        let elf = Elf::parse(program)?;

        assert!(elf.is_64, "Only 64-bit ELF files are supported");

        let progbits_header = elf
            .program_headers
            .iter()
            .filter(|ph| {
                ph.p_type == goblin::elf::program_header::PT_LOAD
                    && ph.p_flags & goblin::elf::program_header::PF_X != 0
            })
            .collect::<Vec<_>>();
        assert!(
            progbits_header.len() == 1,
            "Expected exactly one PT_LOAD segment with execute permission, found: {}",
            progbits_header.len()
        );
        let progbits_header = progbits_header[0];

        assert!(
            progbits_header.p_vaddr == Self::START_ADDRESS,
            "Program must start at address: {:#x}",
            Self::START_ADDRESS
        );

        assert!(
            progbits_header.p_filesz == progbits_header.p_memsz,
            "Program file size must match memory size; seems to contain uninitialized data (.bss section)"
        );

        let start_address = progbits_header.p_vaddr;
        let end_address = start_address + progbits_header.p_memsz;

        let text_section = &program[progbits_header.p_offset as usize
            ..(progbits_header.p_offset + progbits_header.p_filesz) as usize];
        // INFO: Loading other sections (e.g., .data, .bss) is not implemented yet,
        // but would happen here.
        assert!(
            text_section.len() == progbits_header.p_filesz as usize,
            "Text section size mismatch: expected {}, got {}",
            progbits_header.p_filesz,
            text_section.len()
        );

        self.memory[start_address as usize..end_address as usize].copy_from_slice(text_section);
        trace!(
            "Program loaded into CPU memory at address {:#x} with size {} bytes",
            start_address,
            bytesize::ByteSize(text_section.len() as u64)
        );

        Ok(())
    }

    /// Returns the size of the CPU's memory in bytes.
    pub fn memory_size(&self) -> usize {
        self.memory.size()
    }

    pub fn tick(&mut self) {
        // Fetch the instruction at the current program counter
        if self.pc as usize + Self::WORD_SIZE as usize > self.memory.size() {
            panic!("Instruction address out of bounds: {:#x}", self.pc);
        }

        let raw_instruction =
            &self.memory[self.pc as usize..(self.pc as usize) + (Self::WORD_SIZE as usize)];
        assert_eq!(
            raw_instruction.len(),
            Self::WORD_SIZE as usize,
            "Instruction size mismatch"
        );

        let instruction = u32::from_le_bytes(
            raw_instruction
                .try_into()
                .expect("Failed to convert bytes to u32"),
        );
        trace!(
            "PARSING_INSTRUCTION: {} at PC: {:#x}",
            format_u32_le_bits!(instruction),
            self.pc
        );

        // https://five-embeddev.com/riscv-user-isa-manual/Priv-v1.12/opcode-map.html
        match instruction & 0x7f {
            0b0110111 => self.handle_load_upper_immediate(instruction),
            0b0010011 => self.handle_i_type_instruction(instruction),
            0b1110011 => self.handle_system_instruction(instruction),
            ins => panic!("Unimplemented opcode: {:#x}", ins),
        };

        self.pc += Self::WORD_SIZE;
        trace!("CPU_PC: {:#x}", self.pc);
        trace!("CPU_GPRS: {:?}", self.gprs);
    }

    /// Load upper immediate value into register
    /// https://msyksphinz-self.github.io/riscv-isadoc/html/rvi.html#lui
    fn handle_load_upper_immediate(&mut self, instruction: u32) {
        assert!(
            instruction & 0x7f == 0b0110111,
            "Instruction is not a LUI (Load Upper Immediate) instruction"
        );

        // Destination register (rd) is bits 7-11
        let rd = (instruction >> 7) & 0x1f;

        // NOTE: The zero register (x0) is always 0x0.
        // Setting it as rd discards the resulting value.
        if rd == 0 {
            return;
        };

        // Immediate value (imm) is bits 12-31
        let imm = instruction & 0xFFFFF;

        // Load the immediate value into the destination register
        let reg_value = (imm as u64) << 12;
        self.gprs[rd as usize] = reg_value;
    }

    /// Handle I-Type instructions.
    fn handle_i_type_instruction(&mut self, instruction: u32) {
        // This is an immediate instruction
        // The funct3 field is bits 12-14
        let funct3 = instruction >> 12 & 0x7;
        match funct3 {
            0b000 => self.handle_addi(instruction),
            _ => panic!("Unimplemented I-Type instruction: {:#x}", instruction),
        }
    }

    /// Add immediate value to register
    /// https://msyksphinz-self.github.io/riscv-isadoc/html/rvi.html#addi
    fn handle_addi(&mut self, instruction: u32) {
        // Destination register (rd) is bits 7-11
        let rd = (instruction >> 7) & 0x1f;

        // NOTE: The zero register (x0) is always 0x0.
        // Setting it as rd discards the resulting value.
        if rd == 0 {
            return;
        };

        // Source register (rs1) is bits 15-19
        let rs1 = (instruction >> 15) & 0x1f;

        // Immediate value (imm) is bits 20-31
        let imm = instruction >> 20 & 0xFFF;

        // Sign-extend the immediate value to 64 bits
        let sext_imm = sign_extend_u64_to_i64(imm as u64, 20);

        // Add the immediate value to the value in the source register
        let reg_value = self.gprs[rs1 as usize] as i64 + (sext_imm as i64);

        // Store the result in the destination register
        self.gprs[rd as usize] = reg_value as u64;

        trace!(
            "EXECUTING_INSTRUCTION: addi x{}, x{}, {} -> x{}",
            rd, rs1, sext_imm, rd
        );
    }

    fn handle_system_instruction(&mut self, instruction: u32) {
        assert!(
            instruction & 0x7f == 0b1110011,
            "Instruction is not a SYSTEM instruction"
        );

        // The destination register (rd) is bits 7-11
        let rd = (instruction >> 7) & 0x1f;
        assert!(
            rd == 0,
            "System instructions should not write to a destination register"
        );

        // The funct3 field is bits 12-14
        let funct3 = instruction >> 12 & 0x7;
        assert!(
            funct3 == 0b000,
            "System instructions should have funct3 = 0b000, got: {:#b}",
            funct3
        );

        // The immediate value (imm) is bits 20-31
        let imm = instruction >> 20 & 0xFFF;
        match imm {
            0x000 => self.handle_ecall(),
            0x001 => self.handle_ebreak(),
            _ => panic!("Unimplemented SYSTEM instruction with imm: {:#x}", imm),
        }
    }

    /// Handle the `ecall` instruction (environment call).
    /// This is a system call that allows the program to request services from the operating system.
    /// TODO: Read for implementation details:
    /// https://jborza.com/post/2021-04-21-ecalls-and-syscalls/
    fn handle_ecall(&mut self) {
        trace!("EXECUTING_INSTRUCTION: ecall");
        match self.gprs[a7] {
            1 => self.handle_ecall_print_char(),
            10 => self.handle_ecall_exit(),
            _ => {
                panic!("Unimplemented ecall with a7 = {:#x}", self.gprs[a7]);
            }
        }
    }

    fn handle_ecall_print_char(&mut self) {
        // The character to print is in x10 (a0)
        let char_to_print = self.gprs[10] as u8;
        if char_to_print == 0 {
            // If the character is null, we do not print anything
            return;
        }
        // Print the character to stdout
        let character = char::from(char_to_print);
        trace!("EXECUTING_INSTRUCTION: ecall print_char: '{}'", character);
        self.cpu_events
            .send(CpuEvent::DrawCharacter { character })
            .expect("Failed to send render event");
        // Flush stdout to ensure the character is printed immediately
        io::stdout().flush().expect("Failed to flush stdout");
    }

    fn handle_ecall_exit(&mut self) {
        // Exit the program
        let exit_code = self.gprs[a0] as i32;
        info!("Encountered ecall exit with code: {}", exit_code);
        self.is_running = false;
        self.exit_code = exit_code;
        self.cpu_events
            .send(CpuEvent::Exit { exit_code })
            .expect("Failed to send exit event");
    }

    /// Handle the `ebreak` instruction (environment break).
    /// This instruction is used to trigger a breakpoint in the program,
    fn handle_ebreak(&mut self) {
        trace!("EXECUTING_INSTRUCTION: ebreak");
        // TODO: Handle `ebreak` `properly`, e.g., by pausing execution and entering a debug mode
    }

    #[inline]
    pub fn is_halted(&self) -> bool {
        !self.is_running
    }
}
