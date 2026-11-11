01e60edc f553bea9 stp      x21, x20, [sp, #-0x20]!
01e60ee0 f37b01a9 stp      x19, x30, [sp, #0x10]
01e60ee4 145501b0 adrp     x20, #0x4901000
01e60ee8 88267639 ldrb     w8, [x20, #0xd89]
01e60eec f30300aa mov      x19, x0
01e60ef0 28010037 tbnz     w8, #0, #0x1e60f14
01e60ef4 203e01f0 adrp     x0, #0x4627000
01e60ef8 00e444f9 ldr      x0, [x0, #0x9c8]
01e60efc fcedbc97 bl       #0xd9c6ec
01e60f00 603d01f0 adrp     x0, #0x460f000
01e60f04 004841f9 ldr      x0, [x0, #0x290]
01e60f08 f9edbc97 bl       #0xd9c6ec
01e60f0c 28008052 mov      w8, #1
01e60f10 88263639 strb     w8, [x20, #0xd89]
01e60f14 80b08552 mov      w0, #0x2d84
01e60f18 e1031faa mov      x1, xzr
01e60f1c c87efc97 bl       #0x1d80a3c
01e60f20 40010036 tbz      w0, #0, #0x1e60f48
01e60f24 80b08552 mov      w0, #0x2d84
01e60f28 e1031faa mov      x1, xzr
01e60f2c f17efc97 bl       #0x1d80af0
01e60f30 e00300b4 cbz      x0, #0x1e60fac
01e60f34 e10313aa mov      x1, x19
01e60f38 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e60f3c e2031faa mov      x2, xzr
01e60f40 f553c2a8 ldp      x21, x20, [sp], #0x20
01e60f44 6cbcf817 b        #0x1c900f4
01e60f48 601e40f9 ldr      x0, [x19, #0x38]
01e60f4c 000300b4 cbz      x0, #0x1e60fac
01e60f50 343e01f0 adrp     x20, #0x4627000
01e60f54 753d01f0 adrp     x21, #0x460f000
01e60f58 94e644f9 ldr      x20, [x20, #0x9c8]
01e60f5c b54a41f9 ldr      x21, [x21, #0x290]
01e60f60 e1031faa mov      x1, xzr
01e60f64 86280194 bl       #0x1eab17c
01e60f68 890240f9 ldr      x9, [x20]
01e60f6c aa0240f9 ldr      x10, [x21]
01e60f70 681e40f9 ldr      x8, [x19, #0x38]
01e60f74 1f0000f1 cmp      x0, #0
01e60f78 4b01899a csel     x11, x10, x9, eq
01e60f7c 6b0a00f9 str      x11, [x19, #0x10]
01e60f80 680100b4 cbz      x8, #0x1e60fac
01e60f84 01b142b9 ldr      w1, [x8, #0x2b0]
01e60f88 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e60f8c 1f0000f1 cmp      x0, #0
01e60f90 4201899a csel     x2, x10, x9, eq
01e60f94 e003271e fmov     s0, wzr
01e60f98 e00308aa mov      x0, x8
01e60f9c e3031f2a mov      w3, wzr
01e60fa0 e4031faa mov      x4, xzr
01e60fa4 f553c2a8 ldp      x21, x20, [sp], #0x20
01e60fa8 cfbbff17 b        #0x1e4fee4
01e60fac 0ceebc97 bl       #0xd9c7dc
