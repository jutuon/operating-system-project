ENTRY("_start")

SECTIONS {
    . = 1M;
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
    .data : { *(.data) }
    .bss : { *(.bss) }
}
