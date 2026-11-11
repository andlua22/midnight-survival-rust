03438c84 fe0f1ff8 str      x30, [sp, #-0x10]!
03438c88 0b000094 bl       #0x3438cb4
03438c8c e0031faa mov      x0, xzr
03438c90 13e4ff97 bl       #0x3431cdc
03438c94 280f00f0 adrp     x8, #0x361f000
03438c98 03b946bd ldr      s3, [x8, #0x6b8]
03438c9c e0031faa mov      x0, xzr
03438ca0 0008231e fmul     s0, s0, s3
03438ca4 2108231e fmul     s1, s1, s3
03438ca8 4208231e fmul     s2, s2, s3
03438cac fe0741f8 ldr      x30, [sp], #0x10
03438cb0 95e5ff17 b        #0x3432304
03438cb4 ffc300d1 sub      sp, sp, #0x30
03438cb8 ffd300a9 stp      xzr, x20, [sp, #8]
03438cbc f37b02a9 stp      x19, x30, [sp, #0x20]
03438cc0 94a600f0 adrp     x20, #0x490b000
03438cc4 88da42f9 ldr      x8, [x20, #0x5b0]
03438cc8 f30300aa mov      x19, x0
03438ccc ff0300f9 str      xzr, [sp]
03438cd0 c80000b5 cbnz     x8, #0x3438ce8
03438cd4 801600d0 adrp     x0, #0x370a000
03438cd8 00881e91 add      x0, x0, #0x7a2
03438cdc 758e6597 bl       #0xd9c6b0
03438ce0 e80300aa mov      x8, x0
03438ce4 80da02f9 str      x0, [x20, #0x5b0]
03438ce8 e1030091 mov      x1, sp
03438cec e00313aa mov      x0, x19
03438cf0 00013fd6 blr      x8
03438cf4 e007402d ldp      s0, s1, [sp]
03438cf8 e20f412d ldp      s2, s3, [sp, #8]
03438cfc f37b42a9 ldp      x19, x30, [sp, #0x20]
03438d00 f40b40f9 ldr      x20, [sp, #0x10]
03438d04 ffc30091 add      sp, sp, #0x30
03438d08 c0035fd6 ret      
Capstone 5.0: decoded 34 instructions / 136 bytes
