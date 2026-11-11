010def94 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010def98 f30300aa mov      x19, x0
010def9c 60ef9152 mov      w0, #0x8f7b
010defa0 e1031faa mov      x1, xzr
010defa4 a6863294 bl       #0x1d80a3c
010defa8 20010036 tbz      w0, #0, #0x10defcc
010defac 60ef9152 mov      w0, #0x8f7b
010defb0 e1031faa mov      x1, xzr
010defb4 cf863294 bl       #0x1d80af0
010defb8 200100b4 cbz      x0, #0x10defdc
010defbc e10313aa mov      x1, x19
010defc0 e2031faa mov      x2, xzr
010defc4 f37bc1a8 ldp      x19, x30, [sp], #0x10
010defc8 bcc02e14 b        #0x1c8f2b8
010defcc 40828152 mov      w0, #0xc12
010defd0 0072a072 movk     w0, #0x390, lsl #16
010defd4 f37bc1a8 ldp      x19, x30, [sp], #0x10
010defd8 c0035fd6 ret      
010defdc 00f6f297 bl       #0xd9c7dc
Capstone 5.0: decoded 19 instructions / 76 bytes

