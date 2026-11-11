Capstone 5.0: decoded 19 instructions / 76 bytes
010e1334 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
010e1338 f30300aa mov      x19, x0
010e133c 80f59152 mov      w0, #0x8fac
010e1340 e1031faa mov      x1, xzr
010e1344 be7d3294 bl       #0x1d80a3c
010e1348 20010036 tbz      w0, #0, #0x10e136c
010e134c 80f59152 mov      w0, #0x8fac
010e1350 e1031faa mov      x1, xzr
010e1354 e77d3294 bl       #0x1d80af0
010e1358 200100b4 cbz      x0, #0x10e137c
010e135c e10313aa mov      x1, x19
010e1360 e2031faa mov      x2, xzr
010e1364 f37bc1a8 ldp      x19, x30, [sp], #0x10
010e1368 d4b72e14 b        #0x1c8f2b8
010e136c 807e8152 mov      w0, #0xbf4
010e1370 0072a072 movk     w0, #0x390, lsl #16
010e1374 f37bc1a8 ldp      x19, x30, [sp], #0x10
010e1378 c0035fd6 ret      
010e137c 18edf297 bl       #0xd9c7dc

