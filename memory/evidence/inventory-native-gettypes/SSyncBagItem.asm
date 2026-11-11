Capstone 5.0: decoded 19 instructions / 76 bytes
010e9998 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010e999c f30300aa mov      x19, x0
010e99a0 800a9252 mov      w0, #0x9054
010e99a4 e1031faa mov      x1, xzr
010e99a8 255c3294 bl       #0x1d80a3c
010e99ac 20010036 tbz      w0, #0, #0x10e99d0
010e99b0 800a9252 mov      w0, #0x9054
010e99b4 e1031faa mov      x1, xzr
010e99b8 4e5c3294 bl       #0x1d80af0
010e99bc 200100b4 cbz      x0, #0x10e99e0
010e99c0 e10313aa mov      x1, x19
010e99c4 e2031faa mov      x2, xzr
010e99c8 f37bc1a8 ldp      x19, x30, [sp], #0x10
010e99cc 3b962e14 b        #0x1c8f2b8
010e99d0 80788152 mov      w0, #0xbc4
010e99d4 0072a072 movk     w0, #0x390, lsl #16
010e99d8 f37bc1a8 ldp      x19, x30, [sp], #0x10
010e99dc c0035fd6 ret      
010e99e0 7fcbf297 bl       #0xd9c7dc

