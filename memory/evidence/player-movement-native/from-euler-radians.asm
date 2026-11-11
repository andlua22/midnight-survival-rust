03431c44 ffc300d1 sub      sp, sp, #0x30
03431c48 f37b02a9 stp      x19, x30, [sp, #0x20]
03431c4c d3a600d0 adrp     x19, #0x490b000
03431c50 683e41f9 ldr      x8, [x19, #0x278]
03431c54 e007022d stp      s0, s1, [sp, #0x10]
03431c58 e21b00bd str      s2, [sp, #0x18]
03431c5c ff7f00a9 stp      xzr, xzr, [sp]
03431c60 c80000b5 cbnz     x8, #0x3431c78
03431c64 c0160090 adrp     x0, #0x3709000
03431c68 00640791 add      x0, x0, #0x1d9
03431c6c 91aa6597 bl       #0xd9c6b0
03431c70 e80300aa mov      x8, x0
03431c74 603e01f9 str      x0, [x19, #0x278]
03431c78 e0430091 add      x0, sp, #0x10
03431c7c e1030091 mov      x1, sp
03431c80 00013fd6 blr      x8
03431c84 e007402d ldp      s0, s1, [sp]
03431c88 e20f412d ldp      s2, s3, [sp, #8]
03431c8c f37b42a9 ldp      x19, x30, [sp, #0x20]
03431c90 ffc30091 add      sp, sp, #0x30
03431c94 c0035fd6 ret      
Capstone 5.0: decoded 21 instructions / 84 bytes
