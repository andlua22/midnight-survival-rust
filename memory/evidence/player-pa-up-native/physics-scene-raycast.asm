034779f8 ff8302d1 sub      sp, sp, #0xa0
034779fc ee1b00fd str      d14, [sp, #0x30]
03477a00 ed33046d stp      d13, d12, [sp, #0x40]
03477a04 eb2b056d stp      d11, d10, [sp, #0x50]
03477a08 e923066d stp      d9, d8, [sp, #0x60]
03477a0c f75b07a9 stp      x23, x22, [sp, #0x70]
03477a10 f55308a9 stp      x21, x20, [sp, #0x80]
03477a14 f37b09a9 stp      x19, x30, [sp, #0x90]
03477a18 0b1ca04e mov      v11.16b, v0.16b
03477a1c 00e4006f movi     v0.2d, #0000000000000000
03477a20 ff7f02a9 stp      xzr, xzr, [sp, #0x20]
03477a24 ff0f00f9 str      xzr, [sp, #0x18]
03477a28 20c0813c stur     q0, [x1, #0x1c]
03477a2c 200000ad stp      q0, q0, [x1]
03477a30 37a40090 adrp     x23, #0x48fb000
03477a34 e8864f39 ldrb     w8, [x23, #0x3e1]
03477a38 f303032a mov      w19, w3
03477a3c f403022a mov      w20, w2
03477a40 c81ca64e mov      v8.16b, v6.16b
03477a44 f50301aa mov      x21, x1
03477a48 ac1ca54e mov      v12.16b, v5.16b
03477a4c 8d1ca44e mov      v13.16b, v4.16b
03477a50 6e1ca34e mov      v14.16b, v3.16b
03477a54 491ca24e mov      v9.16b, v2.16b
03477a58 2a1ca14e mov      v10.16b, v1.16b
03477a5c f60300aa mov      x22, x0
03477a60 c8000035 cbnz     w8, #0x3477a78
03477a64 a08c0090 adrp     x0, #0x460b000
03477a68 000042f9 ldr      x0, [x0, #0x400]
03477a6c 20936497 bl       #0xd9c6ec
03477a70 28008052 mov      w8, #1
03477a74 e8860f39 strb     w8, [x23, #0x3e1]
03477a78 a88c0090 adrp     x8, #0x460b000
03477a7c 080142f9 ldr      x8, [x8, #0x400]
03477a80 000140f9 ldr      x0, [x8]
03477a84 08e040b9 ldr      w8, [x0, #0xe0]
03477a88 48000035 cbnz     w8, #0x3477a90
03477a8c 4b936497 bl       #0xd9c7b8
03477a90 c0092e1e fmul     s0, s14, s14
03477a94 a1092d1e fmul     s1, s13, s13
03477a98 82092c1e fmul     s2, s12, s12
03477a9c 0028211e fadd     s0, s0, s1
03477aa0 4028201e fadd     s0, s2, s0
03477aa4 28008052 mov      w8, #1
03477aa8 00c0211e fsqrt    s0, s0
03477aac 0101271e fmov     s1, w8
03477ab0 0020211e fcmp     s0, s1
03477ab4 0d040054 b.le     #0x3477b34
03477ab8 c319201e fdiv     s3, s14, s0
03477abc a419201e fdiv     s4, s13, s0
03477ac0 8519201e fdiv     s5, s12, s0
03477ac4 e0630091 add      x0, sp, #0x18
03477ac8 601dab4e mov      v0.16b, v11.16b
03477acc 411daa4e mov      v1.16b, v10.16b
03477ad0 221da94e mov      v2.16b, v9.16b
03477ad4 e1031faa mov      x1, xzr
03477ad8 61adfe97 bl       #0x342305c
03477adc c90240b9 ldr      w9, [x22]
03477ae0 b6a400b0 adrp     x22, #0x490c000
03477ae4 e083c13c ldur     q0, [sp, #0x18]
03477ae8 ea1740f9 ldr      x10, [sp, #0x28]
03477aec c86244f9 ldr      x8, [x22, #0x8c0]
03477af0 e93b00b9 str      w9, [sp, #0x38]
03477af4 e003803d str      q0, [sp]
03477af8 ea0b00f9 str      x10, [sp, #0x10]
03477afc c80000b5 cbnz     x8, #0x3477b14
03477b00 e01400d0 adrp     x0, #0x3715000
03477b04 00141a91 add      x0, x0, #0x685
03477b08 ea926497 bl       #0xd9c6b0
03477b0c e80300aa mov      x8, x0
03477b10 c06204f9 str      x0, [x22, #0x8c0]
03477b14 e0e30091 add      x0, sp, #0x38
03477b18 e1030091 mov      x1, sp
03477b1c 001da84e mov      v0.16b, v8.16b
03477b20 e20315aa mov      x2, x21
03477b24 e303142a mov      w3, w20
03477b28 e403132a mov      w4, w19
03477b2c 00013fd6 blr      x8
03477b30 02000014 b        #0x3477b38
03477b34 e0031f2a mov      w0, wzr
03477b38 f37b49a9 ldp      x19, x30, [sp, #0x90]
03477b3c f55348a9 ldp      x21, x20, [sp, #0x80]
03477b40 f75b47a9 ldp      x23, x22, [sp, #0x70]
03477b44 e923466d ldp      d9, d8, [sp, #0x60]
03477b48 eb2b456d ldp      d11, d10, [sp, #0x50]
03477b4c ed33446d ldp      d13, d12, [sp, #0x40]
03477b50 ee1b40fd ldr      d14, [sp, #0x30]
03477b54 00000012 and      w0, w0, #1
03477b58 ff830291 add      sp, sp, #0xa0
03477b5c c0035fd6 ret      
Capstone 5.0: decoded 90 instructions / 360 bytes
