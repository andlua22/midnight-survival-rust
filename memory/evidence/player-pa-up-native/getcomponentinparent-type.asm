0342a67c f60f1df8 str      x22, [sp, #-0x30]!
0342a680 f55301a9 stp      x21, x20, [sp, #0x10]
0342a684 f37b02a9 stp      x19, x30, [sp, #0x20]
0342a688 16a700b0 adrp     x22, #0x490b000
0342a68c c8b640f9 ldr      x8, [x22, #0x168]
0342a690 f403022a mov      w20, w2
0342a694 f30301aa mov      x19, x1
0342a698 f50300aa mov      x21, x0
0342a69c c80000b5 cbnz     x8, #0x342a6b4
0342a6a0 e01600d0 adrp     x0, #0x3708000
0342a6a4 00002391 add      x0, x0, #0x8c0
0342a6a8 02c86597 bl       #0xd9c6b0
0342a6ac e80300aa mov      x8, x0
0342a6b0 c0b600f9 str      x0, [x22, #0x168]
0342a6b4 e00315aa mov      x0, x21
0342a6b8 00013fd6 blr      x8
0342a6bc 200200b4 cbz      x0, #0x342a700
0342a6c0 16a70090 adrp     x22, #0x490a000
0342a6c4 c36247f9 ldr      x3, [x22, #0xec0]
0342a6c8 f50300aa mov      x21, x0
0342a6cc c30000b5 cbnz     x3, #0x342a6e4
0342a6d0 e01600b0 adrp     x0, #0x3707000
0342a6d4 00b41d91 add      x0, x0, #0x76d
0342a6d8 f6c76597 bl       #0xd9c6b0
0342a6dc e30300aa mov      x3, x0
0342a6e0 c06207f9 str      x0, [x22, #0xec0]
0342a6e4 82020012 and      w2, w20, #1
0342a6e8 e00315aa mov      x0, x21
0342a6ec e10313aa mov      x1, x19
0342a6f0 f37b42a9 ldp      x19, x30, [sp, #0x20]
0342a6f4 f55341a9 ldp      x21, x20, [sp, #0x10]
0342a6f8 f60743f8 ldr      x22, [sp], #0x30
0342a6fc 60001fd6 br       x3
0342a700 37c86597 bl       #0xd9c7dc
Capstone 5.0: decoded 34 instructions / 136 bytes
