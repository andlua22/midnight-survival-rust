0342a4b0 f40f1ef8 str      x20, [sp, #-0x20]!
0342a4b4 f37b01a9 stp      x19, x30, [sp, #0x10]
0342a4b8 14a700b0 adrp     x20, #0x490b000
0342a4bc 81b240f9 ldr      x1, [x20, #0x160]
0342a4c0 f30300aa mov      x19, x0
0342a4c4 c10000b5 cbnz     x1, #0x342a4dc
0342a4c8 e01600d0 adrp     x0, #0x3708000
0342a4cc 00642291 add      x0, x0, #0x899
0342a4d0 78c86597 bl       #0xd9c6b0
0342a4d4 e10300aa mov      x1, x0
0342a4d8 80b200f9 str      x0, [x20, #0x160]
0342a4dc e00313aa mov      x0, x19
0342a4e0 f37b41a9 ldp      x19, x30, [sp, #0x10]
0342a4e4 f40742f8 ldr      x20, [sp], #0x20
0342a4e8 20001fd6 br       x1
Capstone 5.0: decoded 15 instructions / 60 bytes
