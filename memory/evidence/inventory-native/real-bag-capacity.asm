01942c74 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
01942c78 f30300aa mov      x19, x0
01942c7c 40c08052 mov      w0, #0x602
01942c80 e1031faa mov      x1, xzr
01942c84 6ef71094 bl       #0x1d80a3c
01942c88 20010036 tbz      w0, #0, #0x1942cac
01942c8c 40c08052 mov      w0, #0x602
01942c90 e1031faa mov      x1, xzr
01942c94 97f71094 bl       #0x1d80af0
01942c98 000100b4 cbz      x0, #0x1942cb8
01942c9c e10313aa mov      x1, x19
01942ca0 e2031faa mov      x2, xzr
01942ca4 f37bc1a8 ldp      x19, x30, [sp], #0x10
01942ca8 84310d14 b        #0x1c8f2b8
01942cac 604a40b9 ldr      w0, [x19, #0x48]
01942cb0 f37bc1a8 ldp      x19, x30, [sp], #0x10
01942cb4 c0035fd6 ret      
01942cb8 c966d197 bl       #0xd9c7dc
