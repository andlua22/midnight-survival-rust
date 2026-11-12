0347ab88 f553bea9 stp      x21, x20, [sp, #-0x20]!
0347ab8c f37b01a9 stp      x19, x30, [sp, #0x10]
0347ab90 95a400d0 adrp     x21, #0x490c000
0347ab94 148a00b0 adrp     x20, #0x45bb000
0347ab98 a88a5839 ldrb     w8, [x21, #0x622]
0347ab9c 94c647f9 ldr      x20, [x20, #0xf88]
0347aba0 f30300aa mov      x19, x0
0347aba4 c8000037 tbnz     w8, #0, #0x347abbc
0347aba8 008a00b0 adrp     x0, #0x45bb000
0347abac 00c447f9 ldr      x0, [x0, #0xf88]
0347abb0 cf866497 bl       #0xd9c6ec
0347abb4 28008052 mov      w8, #1
0347abb8 a88a1839 strb     w8, [x21, #0x622]
0347abbc e00313aa mov      x0, x19
0347abc0 82ffff97 bl       #0x347a9c8
0347abc4 880240f9 ldr      x8, [x20]
0347abc8 f40300aa mov      x20, x0
0347abcc 09e140b9 ldr      w9, [x8, #0xe0]
0347abd0 69000035 cbnz     w9, #0x347abdc
0347abd4 e00308aa mov      x0, x8
0347abd8 f8866497 bl       #0xd9c7b8
0347abdc e00314aa mov      x0, x20
0347abe0 e1031faa mov      x1, xzr
0347abe4 e2031faa mov      x2, xzr
0347abe8 b5b0fe97 bl       #0x3426ebc
0347abec 20020036 tbz      w0, #0, #0x347ac30
0347abf0 e00313aa mov      x0, x19
0347abf4 75ffff97 bl       #0x347a9c8
0347abf8 400200b4 cbz      x0, #0x347ac40
0347abfc 94a400d0 adrp     x20, #0x490c000
0347ac00 81ca43f9 ldr      x1, [x20, #0x790]
0347ac04 f30300aa mov      x19, x0
0347ac08 c10000b5 cbnz     x1, #0x347ac20
0347ac0c c01400d0 adrp     x0, #0x3714000
0347ac10 00043691 add      x0, x0, #0xd81
0347ac14 a7866497 bl       #0xd9c6b0
0347ac18 e10300aa mov      x1, x0
0347ac1c 80ca03f9 str      x0, [x20, #0x790]
0347ac20 e00313aa mov      x0, x19
0347ac24 f37b41a9 ldp      x19, x30, [sp, #0x10]
0347ac28 f553c2a8 ldp      x21, x20, [sp], #0x20
0347ac2c 20001fd6 br       x1
0347ac30 f37b41a9 ldp      x19, x30, [sp, #0x10]
0347ac34 e0031faa mov      x0, xzr
0347ac38 f553c2a8 ldp      x21, x20, [sp], #0x20
0347ac3c c0035fd6 ret      
0347ac40 e7866497 bl       #0xd9c7dc
Capstone 5.0: decoded 47 instructions / 188 bytes
