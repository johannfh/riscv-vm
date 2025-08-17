OUTPUT_ARCH( "riscv" )
ENTRY( _start )

SECTIONS
{
    /* TODO: Implement Virtual Addressing and set to 0x80000000 */
    . = 0x0;
    .text : { *(.text) }
    .data : { *(.data) }
    .bss :  { *(.bss)  }
}

