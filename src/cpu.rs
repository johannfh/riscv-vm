use std::io::Write;

use memmap2::{MmapMut, MmapOptions};

use crate::{
    constants::{a0, a7},
    format_u32_le_bits,
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

    pub memory: MmapMut,
}

impl Cpu {
    const START_ADDRESS: u64 = 0x8000_0000;
    const MEMORY_SIZE: u64 = 1024 * 1024 * 1024 * 4; // 4GB memory
    /// The size of a word in bytes. For RISC-V, this is typically 4 bytes (32 bits).
    const WORD_SIZE: u64 = 4;

    /// Creates a new CPU instance with all registers initialized to zero.
    pub fn new() -> Self {
        let memory = MmapOptions::new()
            .len(Self::MEMORY_SIZE as usize)
            .map_anon()
            .expect("Failed to create memory map");
        Cpu {
            gprs: [0; 32],
            pc: Self::START_ADDRESS,
            memory,
        }
    }

    pub fn with_program(program: &[u8]) -> Self {
        let mut cpu = Cpu::new();
        // Load the program into memory starting at address 0x8000_0000
        let start_address = Self::START_ADDRESS;
        for (i, &byte) in program.iter().enumerate() {
            let address = (start_address + (i as u64)) as usize;
            if address >= cpu.memory.len() {
                panic!("Program exceeds memory bounds at address: {:#x}", address);
            }
            cpu.memory[address] = byte;
        }
        cpu.pc = start_address;
        cpu
    }

    pub fn tick(&mut self) {
        // Fetch the instruction at the current program counter
        if self.pc as usize + Self::WORD_SIZE as usize > self.memory.len() {
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
        }

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
        print!("{}", char::from(char_to_print));
        // Flush stdout to ensure the character is printed immediately
        std::io::stdout().flush().expect("Failed to flush stdout");
    }

    fn handle_ecall_exit(&mut self) {
        // Exit the program
        let exit_code = self.gprs[a0] as i32;
        info!("Exiting with code: {}", exit_code);
        std::process::exit(exit_code);
    }

    /// Handle the `ebreak` instruction (environment break).
    /// This instruction is used to trigger a breakpoint in the program,
    fn handle_ebreak(&mut self) {
        trace!("EXECUTING_INSTRUCTION: ebreak");
        // TODO: Handle EBREAK properly, e.g., by pausing execution and entering a debug mode
    }
}
