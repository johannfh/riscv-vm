.equ ECALL_WRITE, 64
.equ ECALL_EXIT, 93

.section .data
    message: .asciz "Hello World!\n"
    length: .word 13


.section .text
.global _start

_start:
    # WARN: Specifying the file descriptor is UNSUPPORTED / will be IGNORED
    # NOTE: Will likely be 1/2/3 for stdout/stderr/stdin
    la a0, 0
    # Load the address of the message into a0
    la a1, message          # Load address of the message into a0
    # Load the length of the message into a1
    lw a2, length           # Load the length of the message into a1
    # Load the ecall number for writing to stdout into a7
    li a7, ECALL_WRITE      # Load the ecall number for writing to stdout
    # Make the ecall to write the message
    ecall

    # Make the ecall to exit the program
    li a7, ECALL_EXIT       # Load the ecall number for exiting
    li a0, 0                # Load the exit code 0 into a0
    ecall

