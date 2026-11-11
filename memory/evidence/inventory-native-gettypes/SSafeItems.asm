010e608c f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010e6090 f30300aa mov      x19, x0
010e6094 c0019252 mov      w0, #0x900e
010e6098 e1031faa mov      x1, xzr
010e609c 686a3294 bl       #0x1d80a3c
010e60a0 20010036 tbz      w0, #0, #0x10e60c4
010e60a4 c0019252 mov      w0, #0x900e
010e60a8 e1031faa mov      x1, xzr
010e60ac 916a3294 bl       #0x1d80af0
010e60b0 200100b4 cbz      x0, #0x10e60d4
010e60b4 e10313aa mov      x1, x19
010e60b8 e2031faa mov      x2, xzr
010e60bc f37bc1a8 ldp      x19, x30, [sp], #0x10
010e60c0 7ea42e14 b        #0x1c8f2b8
010e60c4 e07d8152 mov      w0, #0xbef
010e60c8 0072a072 movk     w0, #0x390, lsl #16
010e60cc f37bc1a8 ldp      x19, x30, [sp], #0x10
010e60d0 c0035fd6 ret      
010e60d4 c2d9f297 bl       #0xd9c7dc
Capstone 5.0: decoded 19 instructions / 76 bytes

