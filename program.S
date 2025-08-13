.equ ECALL_PRINT_REG, 1
.equ ECALL_EXIT, 10

.section .text
.global _start

_start:
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'H'              # Load the character 'H' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'e'              # Load the character 'e' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'l'              # Load the character 'l' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'l'              # Load the character 'l' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'o'              # Load the character 'o' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, ' '              # Load the space character into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'W'              # Load the character 'W' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'o'              # Load the character 'o' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'r'              # Load the character 'r' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'l'              # Load the character 'l' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, 'd'              # Load the character 'd' into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, '!'              # Load the exclamation mark into a0
    ecall
    li a7, ECALL_PRINT_REG  # Load the ecall number for printing a register
    li a0, '\n'             # Load the newline character into a0
    ecall

    # Make the ecall to exit the program
    li a7, ECALL_EXIT       # Load the ecall number for exiting
    li a0, 0                # Load the exit code 0 into a0
    ecall

