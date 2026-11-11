010ea914 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010ea918 f30300aa mov      x19, x0
010ea91c 200d9252 mov      w0, #0x9069
010ea920 e1031faa mov      x1, xzr
010ea924 46583294 bl       #0x1d80a3c
010ea928 20010036 tbz      w0, #0, #0x10ea94c
010ea92c 200d9252 mov      w0, #0x9069
010ea930 e1031faa mov      x1, xzr
010ea934 6f583294 bl       #0x1d80af0
010ea938 200100b4 cbz      x0, #0x10ea95c
010ea93c e10313aa mov      x1, x19
010ea940 e2031faa mov      x2, xzr
010ea944 f37bc1a8 ldp      x19, x30, [sp], #0x10
010ea948 5c922e14 b        #0x1c8f2b8
010ea94c e07e8152 mov      w0, #0xbf7
010ea950 0072a072 movk     w0, #0x390, lsl #16
010ea954 f37bc1a8 ldp      x19, x30, [sp], #0x10
010ea958 c0035fd6 ret      
010ea95c a0c7f297 bl       #0xd9c7dc
Capstone 5.0: decoded 19 instructions / 76 bytes

