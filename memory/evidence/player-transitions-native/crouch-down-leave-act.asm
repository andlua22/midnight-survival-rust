01e60fb0 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
01e60fb4 f30300aa mov      x19, x0
01e60fb8 a0b08552 mov      w0, #0x2d85
01e60fbc e1031faa mov      x1, xzr
01e60fc0 9f7efc97 bl       #0x1d80a3c
01e60fc4 20010036 tbz      w0, #0, #0x1e60fe8
01e60fc8 a0b08552 mov      w0, #0x2d85
01e60fcc e1031faa mov      x1, xzr
01e60fd0 c87efc97 bl       #0x1d80af0
01e60fd4 400100b4 cbz      x0, #0x1e60ffc
01e60fd8 e10313aa mov      x1, x19
01e60fdc e2031faa mov      x2, xzr
01e60fe0 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e60fe4 44bcf817 b        #0x1c900f4
01e60fe8 601e40f9 ldr      x0, [x19, #0x38]
01e60fec 800000b4 cbz      x0, #0x1e60ffc
01e60ff0 e1031faa mov      x1, xzr
01e60ff4 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e60ff8 cc600114 b        #0x1eb9328
01e60ffc f8edbc97 bl       #0xd9c7dc
01e61000 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
01e61004 f30300aa mov      x19, x0
01e61008 e0b08552 mov      w0, #0x2d87
01e6100c e1031faa mov      x1, xzr
01e61010 8b7efc97 bl       #0x1d80a3c
01e61014 20010036 tbz      w0, #0, #0x1e61038
01e61018 e0b08552 mov      w0, #0x2d87
01e6101c e1031faa mov      x1, xzr
01e61020 b47efc97 bl       #0x1d80af0
01e61024 e00100b4 cbz      x0, #0x1e61060
01e61028 e10313aa mov      x1, x19
01e6102c e2031faa mov      x2, xzr
01e61030 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e61034 30bcf817 b        #0x1c900f4
01e61038 601e40f9 ldr      x0, [x19, #0x38]
01e6103c 200100b4 cbz      x0, #0x1e61060
01e61040 e1031faa mov      x1, xzr
01e61044 6dbaff97 bl       #0x1e4f9f8
01e61048 601e40f9 ldr      x0, [x19, #0x38]
01e6104c a00000b4 cbz      x0, #0x1e61060
01e61050 e003271e fmov     s0, wzr
01e61054 e1031faa mov      x1, xzr
01e61058 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e6105c b1730114 b        #0x1ebdf20
01e61060 dfedbc97 bl       #0xd9c7dc
