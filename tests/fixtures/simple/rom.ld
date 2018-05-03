OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")
OUTPUT_ARCH(arm)

MEMORY
{
	vectors(r)	: ORIGIN = 0, LENGTH = 1K
   	ramall(rwx)	: o = 0x08000000, l = 0x00010000 /* 64KB */
}


SECTIONS{
    .vectors 0x00000000 : {
        *(.vector)
    } > vectors

    .text 0x08000000 : {
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