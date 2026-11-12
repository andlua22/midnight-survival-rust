0133f24c f75bbda9 stp      x23, x22, [sp, #-0x30]!
0133f250 f55301a9 stp      x21, x20, [sp, #0x10]
0133f254 f37b02a9 stp      x19, x30, [sp, #0x20]
0133f258 f3ad01d0 adrp     x19, #0x48fd000
0133f25c 68725339 ldrb     w8, [x19, #0x4dc]
0133f260 48020037 tbnz     w8, #0, #0x133f2a8
0133f264 a09401b0 adrp     x0, #0x45d4000
0133f268 007041f9 ldr      x0, [x0, #0x2e0]
0133f26c 2075e997 bl       #0xd9c6ec
0133f270 009501b0 adrp     x0, #0x45e0000
0133f274 004844f9 ldr      x0, [x0, #0x890]
0133f278 1d75e997 bl       #0xd9c6ec
0133f27c c09501f0 adrp     x0, #0x45fa000
0133f280 008044f9 ldr      x0, [x0, #0x900]
0133f284 1a75e997 bl       #0xd9c6ec
0133f288 209401f0 adrp     x0, #0x45c6000
0133f28c 00f443f9 ldr      x0, [x0, #0x7e8]
0133f290 1775e997 bl       #0xd9c6ec
0133f294 409701b0 adrp     x0, #0x4628000
0133f298 009c45f9 ldr      x0, [x0, #0xb38]
0133f29c 1475e997 bl       #0xd9c6ec
0133f2a0 28008052 mov      w8, #1
0133f2a4 68721339 strb     w8, [x19, #0x4dc]
0133f2a8 202d8552 mov      w0, #0x2969
0133f2ac e1031faa mov      x1, xzr
0133f2b0 e3052994 bl       #0x1d80a3c
0133f2b4 40010036 tbz      w0, #0, #0x133f2dc
0133f2b8 202d8552 mov      w0, #0x2969
0133f2bc e1031faa mov      x1, xzr
0133f2c0 0c062994 bl       #0x1d80af0
0133f2c4 200700b4 cbz      x0, #0x133f3a8
0133f2c8 f37b42a9 ldp      x19, x30, [sp, #0x20]
0133f2cc f55341a9 ldp      x21, x20, [sp, #0x10]
0133f2d0 e1031faa mov      x1, xzr
0133f2d4 f75bc3a8 ldp      x23, x22, [sp], #0x30
0133f2d8 fa942614 b        #0x1ce46c0
0133f2dc 089501b0 adrp     x8, #0x45e0000
0133f2e0 084944f9 ldr      x8, [x8, #0x890]
0133f2e4 379401f0 adrp     x23, #0x45c6000
0133f2e8 549701b0 adrp     x20, #0x4628000
0133f2ec 080140f9 ldr      x8, [x8]
0133f2f0 f7f643f9 ldr      x23, [x23, #0x7e8]
0133f2f4 085d40f9 ldr      x8, [x8, #0xb8]
0133f2f8 e00240f9 ldr      x0, [x23]
0133f2fc 130140f9 ldr      x19, [x8]
0133f300 09e040b9 ldr      w9, [x0, #0xe0]
0133f304 949e45f9 ldr      x20, [x20, #0xb38]
0133f308 69000035 cbnz     w9, #0x133f314
0133f30c 2b75e997 bl       #0xd9c7b8
0133f310 e00240f9 ldr      x0, [x23]
0133f314 085c40f9 ldr      x8, [x0, #0xb8]
0133f318 940240f9 ldr      x20, [x20]
0133f31c 151540f9 ldr      x21, [x8, #0x28]
0133f320 d50200b5 cbnz     x21, #0x133f378
0133f324 b59401b0 adrp     x21, #0x45d4000
0133f328 08e040b9 ldr      w8, [x0, #0xe0]
0133f32c b57241f9 ldr      x21, [x21, #0x2e0]
0133f330 68000035 cbnz     w8, #0x133f33c
0133f334 2175e997 bl       #0xd9c7b8
0133f338 e00240f9 ldr      x0, [x23]
0133f33c 085c40f9 ldr      x8, [x0, #0xb8]
0133f340 a00240f9 ldr      x0, [x21]
0133f344 160140f9 ldr      x22, [x8]
0133f348 2175e997 bl       #0xd9c7cc
0133f34c e00200b4 cbz      x0, #0x133f3a8
0133f350 c89501f0 adrp     x8, #0x45fa000
0133f354 088144f9 ldr      x8, [x8, #0x900]
0133f358 e10316aa mov      x1, x22
0133f35c e3031faa mov      x3, xzr
0133f360 f50300aa mov      x21, x0
0133f364 020140f9 ldr      x2, [x8]
0133f368 d7d93794 bl       #0x2135ac4
0133f36c e80240f9 ldr      x8, [x23]
0133f370 085d40f9 ldr      x8, [x8, #0xb8]
0133f374 151500f9 str      x21, [x8, #0x28]
0133f378 930100b4 cbz      x19, #0x133f3a8
0133f37c e00313aa mov      x0, x19
0133f380 e10314aa mov      x1, x20
0133f384 e20315aa mov      x2, x21
0133f388 f37b42a9 ldp      x19, x30, [sp, #0x20]
0133f38c f55341a9 ldp      x21, x20, [sp, #0x10]
0133f390 e3031f2a mov      w3, wzr
0133f394 e4031faa mov      x4, xzr
0133f398 e5031f2a mov      w5, wzr
0133f39c e6031faa mov      x6, xzr
0133f3a0 f75bc3a8 ldp      x23, x22, [sp], #0x30
0133f3a4 6bcc3714 b        #0x2132550
0133f3a8 0d75e997 bl       #0xd9c7dc
