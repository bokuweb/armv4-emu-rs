#include "armpsr.h"

	.section .vector,"ax"
	.arm
	.align 0
	.global	__reset
__reset:
	ldr	pc, =reset_entry

	.section .text
	.arm
	.align 0
	.global reset_entry
reset_entry:
  mov r0, #1
loop:
  mov r0, #2
  b loop

