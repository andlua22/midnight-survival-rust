010e1988 f40f1ef8 str      x20, [sp, #-0x20]!
010e198c f37b01a9 stp      x19, x30, [sp, #0x10]
010e1990 f30301aa mov      x19, x1
010e1994 f40300aa mov      x20, x0
010e1998 a0f69152 mov      w0, #0x8fb5
010e199c e1031faa mov      x1, xzr
010e19a0 277c3294 bl       #0x1d80a3c
010e19a4 60010036 tbz      w0, #0, #0x10e19d0
010e19a8 a0f69152 mov      w0, #0x8fb5
010e19ac e1031faa mov      x1, xzr
010e19b0 507c3294 bl       #0x1d80af0
010e19b4 400700b4 cbz      x0, #0x10e1a9c
010e19b8 e20313aa mov      x2, x19
010e19bc f37b41a9 ldp      x19, x30, [sp, #0x10]
010e19c0 e10314aa mov      x1, x20
010e19c4 e3031faa mov      x3, xzr
010e19c8 f40742f8 ldr      x20, [sp], #0x20
010e19cc 81192f14 b        #0x1ca7fd0
010e19d0 730600b4 cbz      x19, #0x10e1a9c
010e19d4 c1058052 mov      w1, #0x2e
010e19d8 e00313aa mov      x0, x19
010e19dc e2031faa mov      x2, xzr
010e19e0 412e4394 bl       #0x21ad2e4
010e19e4 801a00b9 str      w0, [x20, #0x18]
010e19e8 c10e8052 mov      w1, #0x76
010e19ec e00313aa mov      x0, x19
010e19f0 e2031faa mov      x2, xzr
010e19f4 3c2e4394 bl       #0x21ad2e4
010e19f8 801e00b9 str      w0, [x20, #0x1c]
010e19fc 210c8052 mov      w1, #0x61
010e1a00 e00313aa mov      x0, x19
010e1a04 e2031faa mov      x2, xzr
010e1a08 fe2d4394 bl       #0x21ad200
010e1a0c 08000012 and      w8, w0, #1
010e1a10 21068052 mov      w1, #0x31
010e1a14 e00313aa mov      x0, x19
010e1a18 e2031faa mov      x2, xzr
010e1a1c 88820039 strb     w8, [x20, #0x20]
010e1a20 312e4394 bl       #0x21ad2e4
010e1a24 802600b9 str      w0, [x20, #0x24]
010e1a28 e1058052 mov      w1, #0x2f
010e1a2c e00313aa mov      x0, x19
010e1a30 e2031faa mov      x2, xzr
010e1a34 2c2e4394 bl       #0x21ad2e4
010e1a38 802a00b9 str      w0, [x20, #0x28]
010e1a3c c10f8052 mov      w1, #0x7e
010e1a40 e00313aa mov      x0, x19
010e1a44 e2031faa mov      x2, xzr
010e1a48 272e4394 bl       #0x21ad2e4
010e1a4c 802e00b9 str      w0, [x20, #0x2c]
010e1a50 61088052 mov      w1, #0x43
010e1a54 e00313aa mov      x0, x19
010e1a58 e2031faa mov      x2, xzr
010e1a5c 222e4394 bl       #0x21ad2e4
010e1a60 803200b9 str      w0, [x20, #0x30]
010e1a64 e1008052 mov      w1, #7
010e1a68 e00313aa mov      x0, x19
010e1a6c e2031faa mov      x2, xzr
010e1a70 1d2e4394 bl       #0x21ad2e4
010e1a74 803600b9 str      w0, [x20, #0x34]
010e1a78 010d8052 mov      w1, #0x68
010e1a7c e00313aa mov      x0, x19
010e1a80 e2031faa mov      x2, xzr
010e1a84 d2304394 bl       #0x21addcc
010e1a88 801e00f9 str      x0, [x20, #0x38]
010e1a8c e00313aa mov      x0, x19
010e1a90 f37b41a9 ldp      x19, x30, [sp, #0x10]
010e1a94 f40742f8 ldr      x20, [sp], #0x20
010e1a98 c0035fd6 ret      
010e1a9c 50ebf297 bl       #0xd9c7dc
