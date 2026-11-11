010d2df4 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010d2df8 f30300aa mov      x19, x0
010d2dfc 60d59152 mov      w0, #0x8eab
010d2e00 e1031faa mov      x1, xzr
010d2e04 0eb73294 bl       #0x1d80a3c
010d2e08 20010036 tbz      w0, #0, #0x10d2e2c
010d2e0c 60d59152 mov      w0, #0x8eab
010d2e10 e1031faa mov      x1, xzr
010d2e14 37b73294 bl       #0x1d80af0
010d2e18 200100b4 cbz      x0, #0x10d2e3c
010d2e1c e10313aa mov      x1, x19
010d2e20 e2031faa mov      x2, xzr
010d2e24 f37bc1a8 ldp      x19, x30, [sp], #0x10
010d2e28 24f12e14 b        #0x1c8f2b8
010d2e2c c07d8152 mov      w0, #0xbee
010d2e30 0072a072 movk     w0, #0x390, lsl #16
010d2e34 f37bc1a8 ldp      x19, x30, [sp], #0x10
010d2e38 c0035fd6 ret      
010d2e3c 6826f397 bl       #0xd9c7dc
Capstone 5.0: decoded 19 instructions / 76 bytes

