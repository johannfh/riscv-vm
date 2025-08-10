use memmap2::{MmapMut, MmapOptions};

use crate::{format_u32_le_bits, utils::sign_extend_u64_to_i64};

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

        match instruction & 0x7f {
            // Load upper immediate value into register
            // https://msyksphinz-self.github.io/riscv-isadoc/html/rvi.html#lui
            0b0110111 => {
                // Destination register (rd) is bits 7-11
                let rd = (instruction >> 7) & 0x1f;
                assert!(rd != 0, "Cannot write to zero register (x0)");
                // Immediate value (imm) is bits 12-31
                let imm = instruction & 0xFFFFF;

                // Load the immediate value into the destination register
                let reg_value = (imm as u64) << 12;
                self.gprs[rd as usize] = reg_value;
            }
            // I-Type instructions
            0b0010011 => {
                // This is an immediate instruction
                // The funct3 field is bits 12-14
                let funct3 = instruction >> 12 & 0x7;
                match funct3 {
                    // Add immediate value to register
                    // https://msyksphinz-self.github.io/riscv-isadoc/html/rvi.html#addi
                    0b000 => {
                        // Destination register (rd) is bits 7-11
                        let rd = (instruction >> 7) & 0x1f;
                        assert!(rd != 0, "Cannot write to zero register (x0)");
                        // Source register (rs1) is bits 15-19
                        let rs1 = (instruction >> 15) & 0x1f;
                        // Immediate value (imm) is bits 20-31
                        let imm = instruction >> 20 & 0xFFF;
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
                    _ => panic!("Unimplemented immediate instruction: {:#x}", instruction),
                }
            }

            ins => panic!("Unimplemented instruction: {:#x}", ins),
        }

        self.pc += Self::WORD_SIZE;
        trace!("CPU_PC: {:#x}", self.pc);
        trace!("CPU_GPRS: {:?}", self.gprs);
    }
}
