Capstone 5.0: decoded 19 instructions / 76 bytes
010e1f60 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010e1f64 f30300aa mov      x19, x0
010e1f68 40f79152 mov      w0, #0x8fba
010e1f6c e1031faa mov      x1, xzr
010e1f70 b37a3294 bl       #0x1d80a3c
010e1f74 20010036 tbz      w0, #0, #0x10e1f98
010e1f78 40f79152 mov      w0, #0x8fba
010e1f7c e1031faa mov      x1, xzr
010e1f80 dc7a3294 bl       #0x1d80af0
010e1f84 200100b4 cbz      x0, #0x10e1fa8
010e1f88 e10313aa mov      x1, x19
010e1f8c e2031faa mov      x2, xzr
010e1f90 f37bc1a8 ldp      x19, x30, [sp], #0x10
010e1f94 c9b42e14 b        #0x1c8f2b8
010e1f98 20778152 mov      w0, #0xbb9
010e1f9c 0072a072 movk     w0, #0x390, lsl #16
010e1fa0 f37bc1a8 ldp      x19, x30, [sp], #0x10
010e1fa4 c0035fd6 ret      
010e1fa8 0deaf297 bl       #0xd9c7dc

