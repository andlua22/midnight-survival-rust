01ec4f34 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
01ec4f38 f30300aa mov      x19, x0
01ec4f3c 80b28552 mov      w0, #0x2d94
01ec4f40 e1031faa mov      x1, xzr
01ec4f44 beeefa97 bl       #0x1d80a3c
01ec4f48 20010036 tbz      w0, #0, #0x1ec4f6c
01ec4f4c 80b28552 mov      w0, #0x2d94
01ec4f50 e1031faa mov      x1, xzr
01ec4f54 e7eefa97 bl       #0x1d80af0
01ec4f58 800200b4 cbz      x0, #0x1ec4fa8
01ec4f5c e10313aa mov      x1, x19
01ec4f60 e2031faa mov      x2, xzr
01ec4f64 f37bc1a8 ldp      x19, x30, [sp], #0x10
01ec4f68 632cf717 b        #0x1c900f4
01ec4f6c 69325539 ldrb     w9, [x19, #0x54c]
01ec4f70 68e24d39 ldrb     w8, [x19, #0x378]
01ec4f74 3f01086b cmp      w9, w8
01ec4f78 61000054 b.ne     #0x1ec4f84
01ec4f7c f37bc1a8 ldp      x19, x30, [sp], #0x10
01ec4f80 c0035fd6 ret      
01ec4f84 c8000034 cbz      w8, #0x1ec4f9c
01ec4f88 e00313aa mov      x0, x19
01ec4f8c 83c2ff97 bl       #0x1eb5998
01ec4f90 e00313aa mov      x0, x19
01ec4f94 f37bc1a8 ldp      x19, x30, [sp], #0x10
01ec4f98 99ffff17 b        #0x1ec4dfc
01ec4f9c e00313aa mov      x0, x19
01ec4fa0 f37bc1a8 ldp      x19, x30, [sp], #0x10
01ec4fa4 48ffff17 b        #0x1ec4cc4
01ec4fa8 0d5ebb97 bl       #0xd9c7dc
Capstone 5.0: decoded 30 instructions / 120 bytes
