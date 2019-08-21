LD_FEATURE("SANE_EXPR")

ENTRY("_start")
TARGET("elf32-i386")

SECTIONS {
    . = 1M;
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

    . = 2M;
    .text : {
        KEEP(*(.text))
        *(.text.*)
    }

    . = 4M;
    .stack : {
        READ_WRITE_PAGE_START_LOCATION = .;
        . = . + 2M;
        stack_start_plus_4_bytes = .;
    }

    . = 6M;
    .bss : {
        *(.bss.*)
    }
    .data : {
        *(.data)
        *(.data.*)
    }
    .rodata : {
        *(.rodata)
        *(.rodata.*)
    }
    .got : {
        *(.got)
        *(.got.*)
    }
    /DISCARD/ : {
        *(*)
    }
}
