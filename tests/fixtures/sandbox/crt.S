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
  mov r0, #1234
	ldrb r1, [r0]
	ldr	r0, [r1], #4
	ldr r8, [r9, r2, lsl #2]
loop:
  mov r0, #2
  b loop

