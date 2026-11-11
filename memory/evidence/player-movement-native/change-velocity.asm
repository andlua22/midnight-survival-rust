01ebdf20 eb2bbc6d stp      d11, d10, [sp, #-0x40]!
01ebdf24 e923016d stp      d9, d8, [sp, #0x10]
01ebdf28 f55302a9 stp      x21, x20, [sp, #0x20]
01ebdf2c f37b03a9 stp      x19, x30, [sp, #0x30]
01ebdf30 34520190 adrp     x20, #0x4901000
01ebdf34 88967f39 ldrb     w8, [x20, #0xfe5]
01ebdf38 081ca04e mov      v8.16b, v0.16b
01ebdf3c f30300aa mov      x19, x0
01ebdf40 c8000037 tbnz     w8, #0, #0x1ebdf58
01ebdf44 603901d0 adrp     x0, #0x45eb000
01ebdf48 008046f9 ldr      x0, [x0, #0xd00]
01ebdf4c e879bb97 bl       #0xd9c6ec
01ebdf50 28008052 mov      w8, #1
01ebdf54 88963f39 strb     w8, [x20, #0xfe5]
01ebdf58 00b18552 mov      w0, #0x2d88
01ebdf5c e1031faa mov      x1, xzr
01ebdf60 b70afb97 bl       #0x1d80a3c
01ebdf64 a0010036 tbz      w0, #0, #0x1ebdf98
01ebdf68 00b18552 mov      w0, #0x2d88
01ebdf6c e1031faa mov      x1, xzr
01ebdf70 e00afb97 bl       #0x1d80af0
01ebdf74 a00d00b4 cbz      x0, #0x1ebe128
01ebdf78 e10313aa mov      x1, x19
01ebdf7c 001da84e mov      v0.16b, v8.16b
01ebdf80 f37b43a9 ldp      x19, x30, [sp, #0x30]
01ebdf84 f55342a9 ldp      x21, x20, [sp, #0x20]
01ebdf88 e923416d ldp      d9, d8, [sp, #0x10]
01ebdf8c e2031faa mov      x2, xzr
01ebdf90 eb2bc46c ldp      d11, d10, [sp], #0x40
01ebdf94 af74f717 b        #0x1c9b250
01ebdf98 688e44b9 ldr      w8, [x19, #0x48c]
01ebdf9c e003271e fmov     s0, wzr
01ebdfa0 e00313aa mov      x0, x19
01ebdfa4 e1031faa mov      x1, xzr
01ebdfa8 1f010072 tst      w8, #1
01ebdfac 0b0d201e fcsel    s11, s8, s0, eq
01ebdfb0 40b15594 bl       #0x342a4b0
01ebdfb4 a00b00b4 cbz      x0, #0x1ebe128
01ebdfb8 e1031faa mov      x1, xzr
01ebdfbc 32eb5594 bl       #0x3438c84
01ebdfc0 08bb00d0 adrp     x8, #0x361f000
01ebdfc4 008d45bd ldr      s0, [x8, #0x58c]
01ebdfc8 68aa43bd ldr      s8, [x19, #0x3a8]
01ebdfcc 69ae43bd ldr      s9, [x19, #0x3ac]
01ebdfd0 e203271e fmov     s2, wzr
01ebdfd4 2108201e fmul     s1, s1, s0
01ebdfd8 e003271e fmov     s0, wzr
01ebdfdc e0031faa mov      x0, xzr
01ebdfe0 19cf5594 bl       #0x3431c44
01ebdfe4 e503271e fmov     s5, wzr
01ebdfe8 041da84e mov      v4.16b, v8.16b
01ebdfec 261da94e mov      v6.16b, v9.16b
01ebdff0 e0031faa mov      x0, xzr
01ebdff4 36d05594 bl       #0x34320cc
01ebdff8 f45101d0 adrp     x20, #0x48fb000
01ebdffc 88824f39 ldrb     w8, [x20, #0x3e0]
01ebe000 081ca04e mov      v8.16b, v0.16b
01ebe004 2a1ca14e mov      v10.16b, v1.16b
01ebe008 491ca24e mov      v9.16b, v2.16b
01ebe00c c8000035 cbnz     w8, #0x1ebe024
01ebe010 603a01b0 adrp     x0, #0x460b000
01ebe014 000042f9 ldr      x0, [x0, #0x400]
01ebe018 b579bb97 bl       #0xd9c6ec
01ebe01c 28008052 mov      w8, #1
01ebe020 88820f39 strb     w8, [x20, #0x3e0]
01ebe024 683a01b0 adrp     x8, #0x460b000
01ebe028 080142f9 ldr      x8, [x8, #0x400]
01ebe02c 000140f9 ldr      x0, [x8]
01ebe030 08e040b9 ldr      w8, [x0, #0xe0]
01ebe034 48000035 cbnz     w8, #0x1ebe03c
01ebe038 e079bb97 bl       #0xd9c7b8
01ebe03c 0009281e fmul     s0, s8, s8
01ebe040 41092a1e fmul     s1, s10, s10
01ebe044 08bb00b0 adrp     x8, #0x361f000
01ebe048 0028211e fadd     s0, s0, s1
01ebe04c 019145bd ldr      s1, [x8, #0x590]
01ebe050 743901b0 adrp     x20, #0x45eb000
01ebe054 2209291e fmul     s2, s9, s9
01ebe058 948246f9 ldr      x20, [x20, #0xd00]
01ebe05c 4028201e fadd     s0, s2, s0
01ebe060 00c0211e fsqrt    s0, s0
01ebe064 0020211e fcmp     s0, s1
01ebe068 8d000054 b.le     #0x1ebe078
01ebe06c 0a19201e fdiv     s10, s8, s0
01ebe070 2819201e fdiv     s8, s9, s0
01ebe074 0f000014 b        #0x1ebe0b0
01ebe078 f55101b0 adrp     x21, #0x48fb000
01ebe07c a8724f39 ldrb     w8, [x21, #0x3dc]
01ebe080 c8000035 cbnz     w8, #0x1ebe098
01ebe084 803b0190 adrp     x0, #0x462e000
01ebe088 007c41f9 ldr      x0, [x0, #0x2f8]
01ebe08c 9879bb97 bl       #0xd9c6ec
01ebe090 28008052 mov      w8, #1
01ebe094 a8720f39 strb     w8, [x21, #0x3dc]
01ebe098 883b0190 adrp     x8, #0x462e000
01ebe09c 087d41f9 ldr      x8, [x8, #0x2f8]
01ebe0a0 080140f9 ldr      x8, [x8]
01ebe0a4 085d40f9 ldr      x8, [x8, #0xb8]
01ebe0a8 0a0140bd ldr      s10, [x8]
01ebe0ac 080940bd ldr      s8, [x8, #8]
01ebe0b0 800240f9 ldr      x0, [x20]
01ebe0b4 08e040b9 ldr      w8, [x0, #0xe0]
01ebe0b8 68000035 cbnz     w8, #0x1ebe0c4
01ebe0bc bf79bb97 bl       #0xd9c7b8
01ebe0c0 800240f9 ldr      x0, [x20]
01ebe0c4 085c40f9 ldr      x8, [x0, #0xb8]
01ebe0c8 60092a1e fmul     s0, s11, s10
01ebe0cc e00313aa mov      x0, x19
01ebe0d0 002500bd str      s0, [x8, #0x24]
01ebe0d4 7ec1ff97 bl       #0x1eae6cc
01ebe0d8 800200b4 cbz      x0, #0x1ebe128
01ebe0dc e1031faa mov      x1, xzr
01ebe0e0 e8f25694 bl       #0x347ac80
01ebe0e4 880240f9 ldr      x8, [x20]
01ebe0e8 6009281e fmul     s0, s11, s8
01ebe0ec e00313aa mov      x0, x19
01ebe0f0 085d40f9 ldr      x8, [x8, #0xb8]
01ebe0f4 0101052d stp      s1, s0, [x8, #0x28]
01ebe0f8 75c1ff97 bl       #0x1eae6cc
01ebe0fc 600100b4 cbz      x0, #0x1ebe128
01ebe100 880240f9 ldr      x8, [x20]
01ebe104 f37b43a9 ldp      x19, x30, [sp, #0x30]
01ebe108 f55342a9 ldp      x21, x20, [sp, #0x20]
01ebe10c e923416d ldp      d9, d8, [sp, #0x10]
01ebe110 085d40f9 ldr      x8, [x8, #0xb8]
01ebe114 e1031faa mov      x1, xzr
01ebe118 0109452d ldp      s1, s2, [x8, #0x28]
01ebe11c 002540bd ldr      s0, [x8, #0x24]
01ebe120 eb2bc46c ldp      d11, d10, [sp], #0x40
01ebe124 fff25614 b        #0x347ad20
01ebe128 ad79bb97 bl       #0xd9c7dc
Capstone 5.0: decoded 131 instructions / 524 bytes
