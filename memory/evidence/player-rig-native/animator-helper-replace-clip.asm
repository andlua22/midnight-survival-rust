013400dc f60f1df8 str      x22, [sp, #-0x30]!
013400e0 f55301a9 stp      x21, x20, [sp, #0x10]
013400e4 f37b02a9 stp      x19, x30, [sp, #0x20]
013400e8 f6ad01b0 adrp     x22, #0x48fd000
013400ec c88a5339 ldrb     w8, [x22, #0x4e2]
013400f0 f30302aa mov      x19, x2
013400f4 f50301aa mov      x21, x1
013400f8 f40300aa mov      x20, x0
013400fc c8000037 tbnz     w8, #0, #0x1340114
01340100 a09401f0 adrp     x0, #0x45d7000
01340104 005443f9 ldr      x0, [x0, #0x6a8]
01340108 7971e997 bl       #0xd9c6ec
0134010c 28008052 mov      w8, #1
01340110 c88a1339 strb     w8, [x22, #0x4e2]
01340114 805c8052 mov      w0, #0x2e4
01340118 e1031faa mov      x1, xzr
0134011c 48022994 bl       #0x1d80a3c
01340120 a0010036 tbz      w0, #0, #0x1340154
01340124 805c8052 mov      w0, #0x2e4
01340128 e1031faa mov      x1, xzr
0134012c 71022994 bl       #0x1d80af0
01340130 e00300b4 cbz      x0, #0x13401ac
01340134 e10314aa mov      x1, x20
01340138 e20315aa mov      x2, x21
0134013c e30313aa mov      x3, x19
01340140 f37b42a9 ldp      x19, x30, [sp, #0x20]
01340144 f55341a9 ldp      x21, x20, [sp, #0x10]
01340148 e4031faa mov      x4, xzr
0134014c f60743f8 ldr      x22, [sp], #0x30
01340150 f73b2514 b        #0x1c8f12c
01340154 b69401f0 adrp     x22, #0x45d7000
01340158 d65643f9 ldr      x22, [x22, #0x6a8]
0134015c c00240f9 ldr      x0, [x22]
01340160 08e040b9 ldr      w8, [x0, #0xe0]
01340164 68000035 cbnz     w8, #0x1340170
01340168 9471e997 bl       #0xd9c7b8
0134016c c00240f9 ldr      x0, [x22]
01340170 085c40f9 ldr      x8, [x0, #0xb8]
01340174 000140f9 ldr      x0, [x8]
01340178 a00100b4 cbz      x0, #0x13401ac
0134017c e10315aa mov      x1, x21
01340180 e20313aa mov      x2, x19
01340184 e3031faa mov      x3, xzr
01340188 52e18294 bl       #0x33f86d0
0134018c 800e40f9 ldr      x0, [x20, #0x18]
01340190 e00000b4 cbz      x0, #0x13401ac
01340194 f37b42a9 ldp      x19, x30, [sp, #0x20]
01340198 f55341a9 ldp      x21, x20, [sp, #0x10]
0134019c e003271e fmov     s0, wzr
013401a0 e1031faa mov      x1, xzr
013401a4 f60743f8 ldr      x22, [sp], #0x30
013401a8 13e88214 b        #0x33fa1f4
013401ac 8c71e997 bl       #0xd9c7dc
