0347ac80 ffc300d1 sub      sp, sp, #0x30
0347ac84 f40b00f9 str      x20, [sp, #0x10]
0347ac88 f37b02a9 stp      x19, x30, [sp, #0x20]
0347ac8c 94a400d0 adrp     x20, #0x490c000
0347ac90 886643f9 ldr      x8, [x20, #0x6c8]
0347ac94 f30300aa mov      x19, x0
0347ac98 ff0b00b9 str      wzr, [sp, #8]
0347ac9c ff0300f9 str      xzr, [sp]
0347aca0 c80000b5 cbnz     x8, #0x347acb8
0347aca4 c01400d0 adrp     x0, #0x3714000
0347aca8 000c1a91 add      x0, x0, #0x683
0347acac 81866497 bl       #0xd9c6b0
0347acb0 e80300aa mov      x8, x0
0347acb4 806603f9 str      x0, [x20, #0x6c8]
0347acb8 e1030091 mov      x1, sp
0347acbc e00313aa mov      x0, x19
0347acc0 00013fd6 blr      x8
0347acc4 e007402d ldp      s0, s1, [sp]
0347acc8 e20b40bd ldr      s2, [sp, #8]
0347accc f37b42a9 ldp      x19, x30, [sp, #0x20]
0347acd0 f40b40f9 ldr      x20, [sp, #0x10]
0347acd4 ffc30091 add      sp, sp, #0x30
0347acd8 c0035fd6 ret      
Capstone 5.0: decoded 23 instructions / 92 bytes
