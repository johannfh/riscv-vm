.section .text
.global _start

_start:
    addi x1, x0, 3    # Load immediate value 5 into register x1
    addi x1, x1, 1    # Increment the value in x1 by 1
    addi x1, x1, 1    # Increment the value in x1 by 1 again
    addi x2, x1, 0    # Move the value in x1 to x2
