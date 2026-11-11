010d13b4 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010d13b8 f30300aa mov      x19, x0
010d13bc 00d19152 mov      w0, #0x8e88
010d13c0 e1031faa mov      x1, xzr
010d13c4 9ebd3294 bl       #0x1d80a3c
010d13c8 20010036 tbz      w0, #0, #0x10d13ec
010d13cc 00d19152 mov      w0, #0x8e88
010d13d0 e1031faa mov      x1, xzr
010d13d4 c7bd3294 bl       #0x1d80af0
010d13d8 200100b4 cbz      x0, #0x10d13fc
010d13dc e10313aa mov      x1, x19
010d13e0 e2031faa mov      x2, xzr
010d13e4 f37bc1a8 ldp      x19, x30, [sp], #0x10
010d13e8 b4f72e14 b        #0x1c8f2b8
010d13ec c07c8152 mov      w0, #0xbe6
010d13f0 0072a072 movk     w0, #0x390, lsl #16
010d13f4 f37bc1a8 ldp      x19, x30, [sp], #0x10
010d13f8 c0035fd6 ret      
010d13fc f82cf397 bl       #0xd9c7dc
Capstone 5.0: decoded 19 instructions / 76 bytes

