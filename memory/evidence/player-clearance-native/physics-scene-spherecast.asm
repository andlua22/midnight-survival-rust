0347d9d4 ef3bb96d stp      d15, d14, [sp, #-0x70]!
0347d9d8 ed33016d stp      d13, d12, [sp, #0x10]
0347d9dc eb2b026d stp      d11, d10, [sp, #0x20]
0347d9e0 e923036d stp      d9, d8, [sp, #0x30]
0347d9e4 f75b04a9 stp      x23, x22, [sp, #0x40]
0347d9e8 f55305a9 stp      x21, x20, [sp, #0x50]
0347d9ec f37b06a9 stp      x19, x30, [sp, #0x60]
0347d9f0 f7a300d0 adrp     x23, #0x48fb000
0347d9f4 e8864f39 ldrb     w8, [x23, #0x3e1]
0347d9f8 f303032a mov      w19, w3
0347d9fc f403022a mov      w20, w2
0347da00 e81ca74e mov      v8.16b, v7.16b
0347da04 f50301aa mov      x21, x1
0347da08 cb1ca64e mov      v11.16b, v6.16b
0347da0c ad1ca54e mov      v13.16b, v5.16b
0347da10 8f1ca44e mov      v15.16b, v4.16b
0347da14 691ca34e mov      v9.16b, v3.16b
0347da18 4a1ca24e mov      v10.16b, v2.16b
0347da1c 2c1ca14e mov      v12.16b, v1.16b
0347da20 0e1ca04e mov      v14.16b, v0.16b
0347da24 f60300aa mov      x22, x0
0347da28 c8000035 cbnz     w8, #0x347da40
0347da2c 608c00d0 adrp     x0, #0x460b000
0347da30 000042f9 ldr      x0, [x0, #0x400]
0347da34 2e7b6497 bl       #0xd9c6ec
0347da38 28008052 mov      w8, #1
0347da3c e8860f39 strb     w8, [x23, #0x3e1]
0347da40 688c00d0 adrp     x8, #0x460b000
0347da44 080142f9 ldr      x8, [x8, #0x400]
0347da48 000140f9 ldr      x0, [x8]
0347da4c 08e040b9 ldr      w8, [x0, #0xe0]
0347da50 48000035 cbnz     w8, #0x347da58
0347da54 597b6497 bl       #0xd9c7b8
0347da58 e0092f1e fmul     s0, s15, s15
0347da5c a1092d1e fmul     s1, s13, s13
0347da60 62092b1e fmul     s2, s11, s11
0347da64 0028211e fadd     s0, s0, s1
0347da68 28008052 mov      w8, #1
0347da6c 4028201e fadd     s0, s2, s0
0347da70 00c0211e fsqrt    s0, s0
0347da74 0101271e fmov     s1, w8
0347da78 03e4006f movi     v3.2d, #0000000000000000
0347da7c 0020211e fcmp     s0, s1
0347da80 a3c2813c stur     q3, [x21, #0x1c]
0347da84 a30e00ad stp      q3, q3, [x21]
0347da88 ad020054 b.le     #0x347dadc
0347da8c a519201e fdiv     s5, s13, s0
0347da90 6619201e fdiv     s6, s11, s0
0347da94 c07e4092 and      x0, x22, #0xffffffff
0347da98 811dac4e mov      v1.16b, v12.16b
0347da9c 421daa4e mov      v2.16b, v10.16b
0347daa0 231da94e mov      v3.16b, v9.16b
0347daa4 071da84e mov      v7.16b, v8.16b
0347daa8 e10315aa mov      x1, x21
0347daac e203142a mov      w2, w20
0347dab0 e303132a mov      w3, w19
0347dab4 f37b46a9 ldp      x19, x30, [sp, #0x60]
0347dab8 f55345a9 ldp      x21, x20, [sp, #0x50]
0347dabc f75b44a9 ldp      x23, x22, [sp, #0x40]
0347dac0 e923436d ldp      d9, d8, [sp, #0x30]
0347dac4 eb2b426d ldp      d11, d10, [sp, #0x20]
0347dac8 ed33416d ldp      d13, d12, [sp, #0x10]
0347dacc e419201e fdiv     s4, s15, s0
0347dad0 c01dae4e mov      v0.16b, v14.16b
0347dad4 ef3bc76c ldp      d15, d14, [sp], #0x70
0347dad8 75ffff17 b        #0x347d8ac
0347dadc f37b46a9 ldp      x19, x30, [sp, #0x60]
0347dae0 f55345a9 ldp      x21, x20, [sp, #0x50]
0347dae4 f75b44a9 ldp      x23, x22, [sp, #0x40]
0347dae8 e923436d ldp      d9, d8, [sp, #0x30]
0347daec eb2b426d ldp      d11, d10, [sp, #0x20]
0347daf0 ed33416d ldp      d13, d12, [sp, #0x10]
0347daf4 e0031f2a mov      w0, wzr
0347daf8 ef3bc76c ldp      d15, d14, [sp], #0x70
0347dafc c0035fd6 ret      
Capstone 5.0: decoded 75 instructions / 300 bytes
