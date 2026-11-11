0347ad20 ffc300d1 sub      sp, sp, #0x30
0347ad24 f40b00f9 str      x20, [sp, #0x10]
0347ad28 f37b02a9 stp      x19, x30, [sp, #0x20]
0347ad2c 94a400d0 adrp     x20, #0x490c000
0347ad30 886a43f9 ldr      x8, [x20, #0x6d0]
0347ad34 f30300aa mov      x19, x0
0347ad38 e007002d stp      s0, s1, [sp]
0347ad3c e20b00bd str      s2, [sp, #8]
0347ad40 c80000b5 cbnz     x8, #0x347ad58
0347ad44 c01400d0 adrp     x0, #0x3714000
0347ad48 00181b91 add      x0, x0, #0x6c6
0347ad4c 59866497 bl       #0xd9c6b0
0347ad50 e80300aa mov      x8, x0
0347ad54 806a03f9 str      x0, [x20, #0x6d0]
0347ad58 e1030091 mov      x1, sp
0347ad5c e00313aa mov      x0, x19
0347ad60 00013fd6 blr      x8
0347ad64 f37b42a9 ldp      x19, x30, [sp, #0x20]
0347ad68 f40b40f9 ldr      x20, [sp, #0x10]
0347ad6c ffc30091 add      sp, sp, #0x30
0347ad70 c0035fd6 ret      
Capstone 5.0: decoded 21 instructions / 84 bytes
