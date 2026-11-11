Capstone 5.0: decoded 19 instructions / 76 bytes
010e790c f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010e7910 f30300aa mov      x19, x0
010e7914 60049252 mov      w0, #0x9023
010e7918 e1031faa mov      x1, xzr
010e791c 48643294 bl       #0x1d80a3c
010e7920 20010036 tbz      w0, #0, #0x10e7944
010e7924 60049252 mov      w0, #0x9023
010e7928 e1031faa mov      x1, xzr
010e792c 71643294 bl       #0x1d80af0
010e7930 200100b4 cbz      x0, #0x10e7954
010e7934 e10313aa mov      x1, x19
010e7938 e2031faa mov      x2, xzr
010e793c f37bc1a8 ldp      x19, x30, [sp], #0x10
010e7940 5e9e2e14 b        #0x1c8f2b8
010e7944 40818152 mov      w0, #0xc0a
010e7948 0072a072 movk     w0, #0x390, lsl #16
010e794c f37bc1a8 ldp      x19, x30, [sp], #0x10
010e7950 c0035fd6 ret      
010e7954 a2d3f297 bl       #0xd9c7dc

