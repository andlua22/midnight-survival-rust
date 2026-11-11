Capstone 5.0: decoded 19 instructions / 76 bytes
010eb160 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010eb164 f30300aa mov      x19, x0
010eb168 000e9252 mov      w0, #0x9070
010eb16c e1031faa mov      x1, xzr
010eb170 33563294 bl       #0x1d80a3c
010eb174 20010036 tbz      w0, #0, #0x10eb198
010eb178 000e9252 mov      w0, #0x9070
010eb17c e1031faa mov      x1, xzr
010eb180 5c563294 bl       #0x1d80af0
010eb184 200100b4 cbz      x0, #0x10eb1a8
010eb188 e10313aa mov      x1, x19
010eb18c e2031faa mov      x2, xzr
010eb190 f37bc1a8 ldp      x19, x30, [sp], #0x10
010eb194 49902e14 b        #0x1c8f2b8
010eb198 e0848152 mov      w0, #0xc27
010eb19c 0072a072 movk     w0, #0x390, lsl #16
010eb1a0 f37bc1a8 ldp      x19, x30, [sp], #0x10
010eb1a4 c0035fd6 ret      
010eb1a8 8dc5f297 bl       #0xd9c7dc

