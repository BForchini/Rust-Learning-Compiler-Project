.text
.globl _main
.p2align 2
_main:
adrp x16, Lconst0@PAGE
ldr d0, [x16, Lconst0@PAGEOFF]
adrp x16, Lconst1@PAGE
ldr d1, [x16, Lconst1@PAGEOFF]
fadd d2, d0, d1
ret
.section __TEXT,__const
.p2align 3
Lconst0:
.quad 0x4014000000000000
.p2align 3
Lconst1:
.quad 0x4008000000000000
