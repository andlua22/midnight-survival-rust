01e61c90 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
01e61c94 f30300aa mov      x19, x0
01e61c98 20b38552 mov      w0, #0x2d99
01e61c9c e1031faa mov      x1, xzr
01e61ca0 677bfc97 bl       #0x1d80a3c
01e61ca4 20010036 tbz      w0, #0, #0x1e61cc8
01e61ca8 20b38552 mov      w0, #0x2d99
01e61cac e1031faa mov      x1, xzr
01e61cb0 907bfc97 bl       #0x1d80af0
01e61cb4 e00000b4 cbz      x0, #0x1e61cd0
01e61cb8 e10313aa mov      x1, x19
01e61cbc e2031faa mov      x2, xzr
01e61cc0 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e61cc4 0cb9f817 b        #0x1c900f4
01e61cc8 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e61ccc c0035fd6 ret      
01e61cd0 c3eabc97 bl       #0xd9c7dc
