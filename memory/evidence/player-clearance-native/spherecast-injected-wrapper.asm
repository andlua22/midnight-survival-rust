0347d8ac ff8301d1 sub      sp, sp, #0x60
0347d8b0 e923026d stp      d9, d8, [sp, #0x20]
0347d8b4 f61b00f9 str      x22, [sp, #0x30]
0347d8b8 f55304a9 stp      x21, x20, [sp, #0x40]
0347d8bc f37b05a9 stp      x19, x30, [sp, #0x50]
0347d8c0 76a400f0 adrp     x22, #0x490c000
0347d8c4 c87644f9 ldr      x8, [x22, #0x8e8]
0347d8c8 f303032a mov      w19, w3
0347d8cc f403022a mov      w20, w2
0347d8d0 f50301aa mov      x21, x1
0347d8d4 e81ca74e mov      v8.16b, v7.16b
0347d8d8 691ca34e mov      v9.16b, v3.16b
0347d8dc e03b00b9 str      w0, [sp, #0x38]
0347d8e0 e007022d stp      s0, s1, [sp, #0x10]
0347d8e4 e21b00bd str      s2, [sp, #0x18]
0347d8e8 e417002d stp      s4, s5, [sp]
0347d8ec e60b00bd str      s6, [sp, #8]
0347d8f0 c80000b5 cbnz     x8, #0x347d908
0347d8f4 c0140090 adrp     x0, #0x3715000
0347d8f8 00382b91 add      x0, x0, #0xace
0347d8fc 6d7b6497 bl       #0xd9c6b0
0347d900 e80300aa mov      x8, x0
0347d904 c07604f9 str      x0, [x22, #0x8e8]
0347d908 e0e30091 add      x0, sp, #0x38
0347d90c e1430091 add      x1, sp, #0x10
0347d910 e2030091 mov      x2, sp
0347d914 201da94e mov      v0.16b, v9.16b
0347d918 011da84e mov      v1.16b, v8.16b
0347d91c e30315aa mov      x3, x21
0347d920 e403142a mov      w4, w20
0347d924 e503132a mov      w5, w19
0347d928 00013fd6 blr      x8
0347d92c f37b45a9 ldp      x19, x30, [sp, #0x50]
0347d930 f55344a9 ldp      x21, x20, [sp, #0x40]
0347d934 f61b40f9 ldr      x22, [sp, #0x30]
0347d938 e923426d ldp      d9, d8, [sp, #0x20]
0347d93c 00000012 and      w0, w0, #1
0347d940 ff830191 add      sp, sp, #0x60
0347d944 c0035fd6 ret      
Capstone 5.0: decoded 39 instructions / 156 bytes
