LD_FEATURE("SANE_EXPR")

ENTRY("_start")
TARGET("elf32-i386")

SECTIONS {
    . = 2M;
    .header ALIGN(8) : {
        /* Multiboot2 header */
        LONG(0xE85250D6)                       /* magic number                           */
        LONG(0)                                /* architecture                           */
        LONG(4 * 6)                            /* header length                          */
        LONG(0xFFFFFFFF - 0xE85250D6 - 4 * 6 + 1) /* checksum:                              */
                                               /* (magic number + architecture + header length + checksum) = 0  */
        /* end tag */
        . = ALIGN(8);
        SHORT(0)
        SHORT(0)
        LONG(8)
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
