use std::io;

use derive_more::Display;
use goblin::elf::Elf;
use log::{debug, info, trace};

use crate::{
    constants::{ECALL_EXIT, ECALL_WRITE, a0, a1, a2, a7},
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

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum CpuEvent {
    Write { text: String },
    Exit { exit_code: i32 },
}

impl Cpu {
    /// The size of a word in bytes. For RISC-V, this is typically 4 bytes (32 bits).
    const WORD_SIZE: u64 = 4;

    /// Creates a new CPU instance with all registers initialized to zero.
    pub fn new(memory_size: usize) -> io::Result<(Self, crossbeam::channel::Receiver<CpuEvent>)> {
        let memory = MonitoredMemory::new(memory_size)?;
        let (send, recv) = crossbeam::channel::unbounded();

        Ok((
            Cpu {
                gprs: [0; 32],
                pc: 0x0,
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

        for phdr in elf
            .program_headers
            .iter()
            .filter(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD)
        {
            dbg!(&phdr);
            let start_address = phdr.p_vaddr as usize;
            let end_address = start_address + phdr.p_memsz as usize;
            if end_address > self.memory.size() {
                return Err(anyhow::anyhow!(
                    "Program exceeds memory bounds: {} > {}",
                    end_address,
                    self.memory.size()
                ));
            }
            let data = &program[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize];
            self.memory[start_address..end_address].copy_from_slice(data);
            debug!(
                "Loaded program segment from offset {} to {} (size: {})",
                phdr.p_offset,
                phdr.p_offset + phdr.p_filesz,
                phdr.p_filesz
            );
        }

        self.pc = elf.entry;

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
            0b0011011 => self.handle_op32_type_instruction(instruction),
            0b0000011 => self.handle_other_i_type_instruction(instruction),
            ins => panic!("Unimplemented opcode: {:#x} | {:#b}", ins, ins),
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

    fn handle_other_i_type_instruction(&mut self, instruction: u32) {
        // This is an immediate instruction
        // The funct3 field is bits 12-14
        let funct3 = instruction >> 12 & 0x7;
        match funct3 {
            0b000 => self.handle_load_byte(instruction),
            0b010 => self.handle_load_word(instruction),
            _ => panic!("Unimplemented other I-Type instruction: {:#b}", funct3),
        }
    }

    fn handle_load_byte(&mut self, instruction: u32) {
        assert!(
            instruction & 0x7f == 0b0000011,
            "Instruction is not a LOAD instruction"
        );

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

        let sext_imm = sign_extend_u64_to_i64(imm as u64, 12);

        // Calculate the effective address
        let effective_address = self.gprs[rs1 as usize] as i64 + sext_imm;

        // Load the byte from memory at the effective address
        let byte_value = self.memory[effective_address as usize];

        // Set the value in the destination register
        self.gprs[rd as usize] = byte_value as u64;

        trace!(
            "EXECUTING_INSTRUCTION: lb x{}, {}(x{}) -> x{}",
            rd, sext_imm, rs1, rd
        );
    }

    fn handle_load_word(&mut self, instruction: u32) {
        assert!(
            instruction & 0x7f == 0b0000011,
            "Instruction is not a LOAD instruction"
        );

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

        let sext_imm = sign_extend_u64_to_i64(imm as u64, 12);

        // Calculate the effective address
        let effective_address = self.gprs[rs1 as usize] as i64 + sext_imm;

        // Load the word from memory at the effective address
        let word_value = u32::from_le_bytes(
            self.memory[effective_address as usize..effective_address as usize + 4]
                .try_into()
                .expect("Failed to convert bytes to u32"),
        );

        // Set the value in the destination register
        self.gprs[rd as usize] = word_value as u64;

        trace!(
            "EXECUTING_INSTRUCTION: lw x{}, {}(x{}) -> x{}",
            rd, sext_imm, rs1, rd
        );
    }

    fn handle_op32_type_instruction(&mut self, instruction: u32) {
        let funct3 = instruction >> 12 & 0x7;
        match funct3 {
            0b000 => self.handle_addiw(instruction),
            _ => panic!("Unimplemented OP32 instruction: {:#x}", instruction),
        }
    }

    fn handle_addiw(&mut self, instruction: u32) {
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
        self.gprs[rd as usize] = (self.gprs[rs1 as usize] as i32 + (sext_imm as i32)) as u64;

        trace!(
            "EXECUTING_INSTRUCTION: addiw x{}, x{}, {} -> x{}",
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
    /// https://jborza.com/post/2021-05-11-riscv-linux-syscalls/
    fn handle_ecall(&mut self) {
        trace!("EXECUTING_INSTRUCTION: ecall");
        match self.gprs[a7] {
            64 => self.handle_ecall_write(),
            93 => self.handle_ecall_exit(),
            _ => {
                panic!("Unimplemented ecall with a7 = {:#x}", self.gprs[a7]);
            }
        }
    }

    fn handle_ecall_write(&mut self) {
        assert!(
            self.gprs[a7] == ECALL_WRITE,
            "ecall write should have a7 = {}, got: {:#x}",
            ECALL_WRITE,
            self.gprs[a7],
        );

        let fd = (self.gprs[a0] & 0x1f) as usize; // File descriptor
        let text_ptr = self.gprs[a1] as usize; // Pointer to the text to write
        let text_len = self.gprs[a2] as usize; // Length of the text to write

        _ = fd; // Currently, we only support custom file descriptors

        // Read the text from memory
        let text_bytes = &self.memory[text_ptr..text_ptr + text_len];
        let text =
            String::from_utf8(text_bytes.to_vec()).expect("Failed to convert bytes to String");
        trace!("Writing text: '{}'", text.escape_debug());
        // Send the text to the application via the CPU events channel
        self.cpu_events
            .send(CpuEvent::Write { text })
            .expect("Failed to send write event");
    }

    fn handle_ecall_exit(&mut self) {
        assert!(
            self.gprs[a7] == ECALL_EXIT,
            "ecall exit should have a7 = {}, got: {:#x}",
            ECALL_EXIT,
            self.gprs[a7],
        );

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
