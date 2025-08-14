.equ ECALL_PRINT_REG, 1
.equ ECALL_EXIT, 10

.section .text
.global _start

_start:
    li a7, ECALL_PRINT_REG
    li a0, 'H'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'e'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'l'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'l'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'o'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, ' '
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'W'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'o'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'r'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'l'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, 'd'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, '!'
    ecall
    li a7, ECALL_PRINT_REG
    li a0, '\n
    ecall

    # Make the ecall to exit the program
    li a7, ECALL_EXIT       # Load the ecall number for exiting
    li a0, 0                # Load the exit code 0 into a0
    ecall

