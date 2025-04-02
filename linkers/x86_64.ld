ENTRY(kstart)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_OFFSET = 0xFFFFFFFF80000000;

SECTIONS {
    
    . = KERNEL_OFFSET;
    . += SIZEOF_HEADERS;

    /* Zero Page - safe alignment */
    .dummy ALIGN(64) : AT(ADDR(.dummy) - KERNEL_OFFSET) {}

    . = ALIGN(4096);

    /* Read-only executable code */
    .text ALIGN(4096) : AT(ADDR(.text) - KERNEL_OFFSET) {
        __text_start = .;
        *(.text .text.*)
        *(.usercopy-fns)
        . = ALIGN(4096);
        __text_end = .;
    }

    /* Read-only data - enhanced security */
    .rodata ALIGN(4096) : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        __rodata_start = .;
        *(.rodata .rodata.*)
        
        /* Optimized alternative codes */
        __altcode_start = .;
        KEEP(*(.altcode*))
        __altcode_end = .;
        
        . = ALIGN(8);
        __altrelocs_start = .;
        KEEP(*(.altrelocs*))
        __altrelocs_end = .;
        
        __altfeatures_start = .;
        KEEP(*(.altfeatures*))
        __altfeatures_end = .;
        
        . = ALIGN(4096);
        __rodata_end = .;
    }

    /* Mutable data - not executable */
    .data ALIGN(4096) : AT(ADDR(.data) - KERNEL_OFFSET) {
        __data_start = .;
        *(.data .data.*)
        . = ALIGN(4096);
        __data_end = .;
    }

    /* BSS - initialized to prevent data leakage */
    .bss (NOLOAD) ALIGN(4096) : AT(ADDR(.bss) - KERNEL_OFFSET) {
        __bss_start = .;
        *(.bss .bss.*)
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