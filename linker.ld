OUTPUT_ARCH(riscv)
ENTRY(_start)

/*
 * The PHDRS block defines the program headers (segments).
 * This optimized version creates two distinct segments for better
 * memory protection: one for executable code and one for read/write data.
 */
PHDRS
{
    /*
     * The executable segment containing code and read-only data.
     * `5` is a bitmask for Readable (4) + Executable (1).
     */
    text PT_LOAD FLAGS(5);

    /*
     * The data segment containing all read/write data.
     * `6` is a bitmask for Readable (4) + Writable (2).
     */
    data PT_LOAD FLAGS(6);
}

/*
 * The SECTIONS block defines where sections are placed in the output file
 * and in memory.
 */
SECTIONS
{
    . = 0x0;

    .text : {
        _start = .;
        *(.text .text.*);
    } :text

    .rodata : {
        *(.rodata .rodata.*);
    } :text

    .data : {
        *(.data .data.*);
    } :data

    /*
     * The .bss section is placed in the 'data' segment but is marked NOLOAD.
     * This means it will occupy space in memory but not in the file, as it
     * contains uninitialized data.
     */
    .bss (NOLOAD) : {
        *(.bss .bss.*);
    } :data

    /DISCARD/ : {
        *(.comment);
        *(.eh_frame);
    }
}
