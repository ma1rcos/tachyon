ENTRY(kstart)
OUTPUT_FORMAT("elf64-littleriscv", "elf64-littleriscv", "elf64-littleriscv")

KERNEL_OFFSET = 0xFFFFFF0000000000;

SECTIONS {

    . = KERNEL_OFFSET;
    . += SIZEOF_HEADERS;

    /* Zero Page - aligned for security */
    .dummy ALIGN(64) : AT(ADDR(.dummy) - KERNEL_OFFSET) {}

    . = ALIGN(4096);

    /* Read-only executable code */
    .text ALIGN(4096) : AT(ADDR(.text) - KERNEL_OFFSET) {
        __text_start = .;
        *(.early_init.text*)  /* Fast boot code */
        . = ALIGN(4096);
        *(.text .text.*)
        *(.usercopy-fns)
        . = ALIGN(4096);
        __text_end = .;
    }

    /* Read-only data */
    .rodata ALIGN(4096) : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        . = ALIGN(4096);
        __rodata_end = .;
    }

    /* Mutable data - not executable */
    .data ALIGN(4096) : AT(ADDR(.data) - KERNEL_OFFSET) {
        __data_start = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        . = ALIGN(4096);
        __data_end = .;
    }

    /* Global Offset Table - Enhanced Protection */
    .got ALIGN(4096) : AT(ADDR(.got) - KERNEL_OFFSET) {
        *(.got .got.*)
        . = ALIGN(4096);
    }

    /* BSS - Initialized to prevent data leaks */
    .bss (NOLOAD) ALIGN(4096) : AT(ADDR(.bss) - KERNEL_OFFSET) {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4096);
        __bss_end = .;
    }

    __end = .;

    /DISCARD/ : {
        *(.comment*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
    }
    
}