01963988 f553bea9 stp      x21, x20, [sp, #-0x20]!
0196398c f37b01a9 stp      x19, x30, [sp, #0x10]
01963990 f40301aa mov      x20, x1
01963994 f50300aa mov      x21, x0
01963998 c0068152 mov      w0, #0x836
0196399c e1031faa mov      x1, xzr
019639a0 f30302aa mov      x19, x2
019639a4 26741094 bl       #0x1d80a3c
019639a8 80010036 tbz      w0, #0, #0x19639d8
019639ac c0068152 mov      w0, #0x836
019639b0 e1031faa mov      x1, xzr
019639b4 4f741094 bl       #0x1d80af0
019639b8 200400b4 cbz      x0, #0x1963a3c
019639bc e30313aa mov      x3, x19
019639c0 f37b41a9 ldp      x19, x30, [sp, #0x10]
019639c4 e10315aa mov      x1, x21
019639c8 e20314aa mov      x2, x20
019639cc e4031faa mov      x4, xzr
019639d0 f553c2a8 ldp      x21, x20, [sp], #0x20
019639d4 54cd0c14 b        #0x1c96f24
019639d8 340300b4 cbz      x20, #0x1963a3c
019639dc 801640b9 ldr      w0, [x20, #0x14]
019639e0 e1031faa mov      x1, xzr
019639e4 56712394 bl       #0x223ff3c
019639e8 b30200b4 cbz      x19, #0x1963a3c
019639ec f50300aa mov      x21, x0
019639f0 601640b9 ldr      w0, [x19, #0x14]
019639f4 e1031faa mov      x1, xzr
019639f8 51712394 bl       #0x223ff3c
019639fc 150200b4 cbz      x21, #0x1963a3c
01963a00 e00100b4 cbz      x0, #0x1963a3c
01963a04 a8c640b9 ldr      w8, [x21, #0xc4]
01963a08 09c440b9 ldr      w9, [x0, #0xc4]
01963a0c 0001096b subs     w0, w8, w9
01963a10 01010054 b.ne     #0x1963a30
01963a14 881640b9 ldr      w8, [x20, #0x14]
01963a18 691640b9 ldr      w9, [x19, #0x14]
01963a1c 0001096b subs     w0, w8, w9
01963a20 81000054 b.ne     #0x1963a30
01963a24 881240b9 ldr      w8, [x20, #0x10]
01963a28 691240b9 ldr      w9, [x19, #0x10]
01963a2c 0001094b sub      w0, w8, w9
01963a30 f37b41a9 ldp      x19, x30, [sp, #0x10]
01963a34 f553c2a8 ldp      x21, x20, [sp], #0x20
01963a38 c0035fd6 ret      
01963a3c 68e3d097 bl       #0xd9c7dc
