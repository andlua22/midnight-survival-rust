0347aaac f553bea9 stp      x21, x20, [sp, #-0x20]!
0347aab0 f37b01a9 stp      x19, x30, [sp, #0x10]
0347aab4 94a400d0 adrp     x20, #0x490c000
0347aab8 158a00b0 adrp     x21, #0x45bb000
0347aabc 88865839 ldrb     w8, [x20, #0x621]
0347aac0 b5c647f9 ldr      x21, [x21, #0xf88]
0347aac4 f30300aa mov      x19, x0
0347aac8 c8000037 tbnz     w8, #0, #0x347aae0
0347aacc 008a00b0 adrp     x0, #0x45bb000
0347aad0 00c447f9 ldr      x0, [x0, #0xf88]
0347aad4 06876497 bl       #0xd9c6ec
0347aad8 28008052 mov      w8, #1
0347aadc 88861839 strb     w8, [x20, #0x621]
0347aae0 e00313aa mov      x0, x19
0347aae4 29000094 bl       #0x347ab88
0347aae8 a80240f9 ldr      x8, [x21]
0347aaec f40300aa mov      x20, x0
0347aaf0 09e140b9 ldr      w9, [x8, #0xe0]
0347aaf4 69000035 cbnz     w9, #0x347ab00
0347aaf8 e00308aa mov      x0, x8
0347aafc 2f876497 bl       #0xd9c7b8
0347ab00 e00314aa mov      x0, x20
0347ab04 e1031faa mov      x1, xzr
0347ab08 e2031faa mov      x2, xzr
0347ab0c ecb0fe97 bl       #0x3426ebc
0347ab10 60000036 tbz      w0, #0, #0x347ab1c
0347ab14 740200b5 cbnz     x20, #0x347ab60
0347ab18 1b000014 b        #0x347ab84
0347ab1c e00313aa mov      x0, x19
0347ab20 aaffff97 bl       #0x347a9c8
0347ab24 a80240f9 ldr      x8, [x21]
0347ab28 f40300aa mov      x20, x0
0347ab2c 09e140b9 ldr      w9, [x8, #0xe0]
0347ab30 69000035 cbnz     w9, #0x347ab3c
0347ab34 e00308aa mov      x0, x8
0347ab38 20876497 bl       #0xd9c7b8
0347ab3c e00314aa mov      x0, x20
0347ab40 e1031faa mov      x1, xzr
0347ab44 e2031faa mov      x2, xzr
0347ab48 ddb0fe97 bl       #0x3426ebc
0347ab4c 40010036 tbz      w0, #0, #0x347ab74
0347ab50 e00313aa mov      x0, x19
0347ab54 9dffff97 bl       #0x347a9c8
0347ab58 600100b4 cbz      x0, #0x347ab84
0347ab5c f40300aa mov      x20, x0
0347ab60 f37b41a9 ldp      x19, x30, [sp, #0x10]
0347ab64 e00314aa mov      x0, x20
0347ab68 e1031faa mov      x1, xzr
0347ab6c f553c2a8 ldp      x21, x20, [sp], #0x20
0347ab70 50befe17 b        #0x342a4b0
0347ab74 f37b41a9 ldp      x19, x30, [sp, #0x10]
0347ab78 e0031faa mov      x0, xzr
0347ab7c f553c2a8 ldp      x21, x20, [sp], #0x20
0347ab80 c0035fd6 ret      
0347ab84 16876497 bl       #0xd9c7dc
Capstone 5.0: decoded 55 instructions / 220 bytes
