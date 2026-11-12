01e4d2c8 f40f1ef8 str      x20, [sp, #-0x20]!
01e4d2cc f37b01a9 stp      x19, x30, [sp, #0x10]
01e4d2d0 b4550190 adrp     x20, #0x4901000
01e4d2d4 88567439 ldrb     w8, [x20, #0xd15]
01e4d2d8 f30300aa mov      x19, x0
01e4d2dc 48020037 tbnz     w8, #0, #0x1e4d324
01e4d2e0 803b01f0 adrp     x0, #0x45c0000
01e4d2e4 003840f9 ldr      x0, [x0, #0x70]
01e4d2e8 013dbd97 bl       #0xd9c6ec
01e4d2ec e03d01f0 adrp     x0, #0x460c000
01e4d2f0 00c441f9 ldr      x0, [x0, #0x388]
01e4d2f4 fe3cbd97 bl       #0xd9c6ec
01e4d2f8 a03e01b0 adrp     x0, #0x4622000
01e4d2fc 00f843f9 ldr      x0, [x0, #0x7f0]
01e4d300 fb3cbd97 bl       #0xd9c6ec
01e4d304 c03e01b0 adrp     x0, #0x4626000
01e4d308 008c44f9 ldr      x0, [x0, #0x918]
01e4d30c f83cbd97 bl       #0xd9c6ec
01e4d310 e03c0190 adrp     x0, #0x45e9000
01e4d314 00e444f9 ldr      x0, [x0, #0x9c8]
01e4d318 f53cbd97 bl       #0xd9c6ec
01e4d31c 28008052 mov      w8, #1
01e4d320 88563439 strb     w8, [x20, #0xd15]
01e4d324 407d8152 mov      w0, #0xbea
01e4d328 e1031faa mov      x1, xzr
01e4d32c c4cdfc97 bl       #0x1d80a3c
01e4d330 40010036 tbz      w0, #0, #0x1e4d358
01e4d334 407d8152 mov      w0, #0xbea
01e4d338 e1031faa mov      x1, xzr
01e4d33c edcdfc97 bl       #0x1d80af0
01e4d340 a00600b4 cbz      x0, #0x1e4d414
01e4d344 e10313aa mov      x1, x19
01e4d348 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e4d34c e2031faa mov      x2, xzr
01e4d350 f40742f8 ldr      x20, [sp], #0x20
01e4d354 680bf917 b        #0x1c900f4
01e4d358 602640f9 ldr      x0, [x19, #0x48]
01e4d35c c00500b4 cbz      x0, #0x1e4d414
01e4d360 c83e01b0 adrp     x8, #0x4626000
01e4d364 088d44f9 ldr      x8, [x8, #0x918]
01e4d368 e2031faa mov      x2, xzr
01e4d36c 010140f9 ldr      x1, [x8]
01e4d370 20b15694 bl       #0x33f97f0
01e4d374 682640f9 ldr      x8, [x19, #0x48]
01e4d378 60b202b9 str      w0, [x19, #0x2b0]
01e4d37c c80400b4 cbz      x8, #0x1e4d414
01e4d380 e93d01f0 adrp     x9, #0x460c000
01e4d384 29c541f9 ldr      x9, [x9, #0x388]
01e4d388 e00308aa mov      x0, x8
01e4d38c e2031faa mov      x2, xzr
01e4d390 210140f9 ldr      x1, [x9]
01e4d394 17b15694 bl       #0x33f97f0
01e4d398 682640f9 ldr      x8, [x19, #0x48]
01e4d39c 60b602b9 str      w0, [x19, #0x2b4]
01e4d3a0 a80300b4 cbz      x8, #0x1e4d414
01e4d3a4 893b01f0 adrp     x9, #0x45c0000
01e4d3a8 293940f9 ldr      x9, [x9, #0x70]
01e4d3ac e00308aa mov      x0, x8
01e4d3b0 e2031faa mov      x2, xzr
01e4d3b4 210140f9 ldr      x1, [x9]
01e4d3b8 0eb15694 bl       #0x33f97f0
01e4d3bc 682640f9 ldr      x8, [x19, #0x48]
01e4d3c0 60ba02b9 str      w0, [x19, #0x2b8]
01e4d3c4 880200b4 cbz      x8, #0x1e4d414
01e4d3c8 e93c0190 adrp     x9, #0x45e9000
01e4d3cc 29e544f9 ldr      x9, [x9, #0x9c8]
01e4d3d0 e00308aa mov      x0, x8
01e4d3d4 e2031faa mov      x2, xzr
01e4d3d8 210140f9 ldr      x1, [x9]
01e4d3dc 05b15694 bl       #0x33f97f0
01e4d3e0 682640f9 ldr      x8, [x19, #0x48]
01e4d3e4 60be02b9 str      w0, [x19, #0x2bc]
01e4d3e8 680100b4 cbz      x8, #0x1e4d414
01e4d3ec a93e01b0 adrp     x9, #0x4622000
01e4d3f0 29f943f9 ldr      x9, [x9, #0x7f0]
01e4d3f4 e00308aa mov      x0, x8
01e4d3f8 e2031faa mov      x2, xzr
01e4d3fc 210140f9 ldr      x1, [x9]
01e4d400 fcb05694 bl       #0x33f97f0
01e4d404 60c202b9 str      w0, [x19, #0x2c0]
01e4d408 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e4d40c f40742f8 ldr      x20, [sp], #0x20
01e4d410 c0035fd6 ret      
01e4d414 f23cbd97 bl       #0xd9c7dc
