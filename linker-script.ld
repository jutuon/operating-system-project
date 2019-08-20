LD_FEATURE("SANE_EXPR")

ENTRY("_start")
TARGET("elf32-i386")

SECTIONS {
    . = 2M;
    .header : {
        /* multiboot header */
        LONG(0x1BADB002)                  /* magic number                           */
        LONG(0x0)                         /* flags                                  */
        LONG(0xFFFFFFFF - 0x1BADB002 + 1) /* checksum:                              */
                                          /* (magic number + flags + checksum) = 0  */
    }
    .text : {
        KEEP(*(.text))
        *(.text.*)
    }
    .bss : {
        *(.bss.0) /* stack */
        *(.bss)
    }
    .data : { *(.data) }
}
