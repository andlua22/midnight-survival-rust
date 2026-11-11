03438a14 ffc300d1 sub      sp, sp, #0x30
03438a18 f40b00f9 str      x20, [sp, #0x10]
03438a1c f37b02a9 stp      x19, x30, [sp, #0x20]
03438a20 94a600f0 adrp     x20, #0x490b000
03438a24 88ca42f9 ldr      x8, [x20, #0x590]
03438a28 f30300aa mov      x19, x0
03438a2c ff0b00b9 str      wzr, [sp, #8]
03438a30 ff0300f9 str      xzr, [sp]
03438a34 c80000b5 cbnz     x8, #0x3438a4c
03438a38 801600d0 adrp     x0, #0x370a000
03438a3c 00301a91 add      x0, x0, #0x68c
03438a40 1c8f6597 bl       #0xd9c6b0
03438a44 e80300aa mov      x8, x0
03438a48 80ca02f9 str      x0, [x20, #0x590]
03438a4c e1030091 mov      x1, sp
03438a50 e00313aa mov      x0, x19
03438a54 00013fd6 blr      x8
03438a58 e007402d ldp      s0, s1, [sp]
03438a5c e20b40bd ldr      s2, [sp, #8]
03438a60 f37b42a9 ldp      x19, x30, [sp, #0x20]
03438a64 f40b40f9 ldr      x20, [sp, #0x10]
03438a68 ffc30091 add      sp, sp, #0x30
03438a6c c0035fd6 ret      
Capstone 5.0: decoded 23 instructions / 92 bytes
