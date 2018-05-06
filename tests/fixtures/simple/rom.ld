OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")
OUTPUT_ARCH(arm)

MEMORY
{
   	romall(rx)	: o = 0x00000000, l = 0x00080000 /* 512KB */
    ramall(rwx)	: o = 0x40000000, l = 0x00010000 /* 64KB */
}


SECTIONS{
    .vectors 0x00000000 : {
        *(.vector)
    } > romall

    .text 0x40000000 : {
        *(.text)
    }
    .data : {
        *(.data)
    }
    .rodata : {
        *(.rodata)
    }
    .bss : {
        *(.bss)
    }
}