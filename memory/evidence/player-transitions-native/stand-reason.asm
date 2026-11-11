01e7b9d0 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
01e7b9d4 f30300aa mov      x19, x0
01e7b9d8 e0db8552 mov      w0, #0x2edf
01e7b9dc e1031faa mov      x1, xzr
01e7b9e0 1714fc97 bl       #0x1d80a3c
01e7b9e4 20010036 tbz      w0, #0, #0x1e7ba08
01e7b9e8 e0db8552 mov      w0, #0x2edf
01e7b9ec e1031faa mov      x1, xzr
01e7b9f0 4014fc97 bl       #0x1d80af0
01e7b9f4 a00100b4 cbz      x0, #0x1e7ba28
01e7b9f8 e10313aa mov      x1, x19
01e7b9fc e2031faa mov      x2, xzr
01e7ba00 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e7ba04 bc51f817 b        #0x1c900f4
01e7ba08 601e40f9 ldr      x0, [x19, #0x38]
01e7ba0c e00000b4 cbz      x0, #0x1e7ba28
01e7ba10 21008052 mov      w1, #1
01e7ba14 e2031faa mov      x2, xzr
01e7ba18 b350ff97 bl       #0x1e4fce4
01e7ba1c e00313aa mov      x0, x19
01e7ba20 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e7ba24 02000014 b        #0x1e7ba2c
01e7ba28 6d83bc97 bl       #0xd9c7dc
