021adca4 f40f1ef8 str      x20, [sp, #-0x20]!
021adca8 f37b01a9 stp      x19, x30, [sp, #0x10]
021adcac 210300b4 cbz      x1, #0x21add10
021adcb0 231840b9 ldr      w3, [x1, #0x18]
021adcb4 f303022a mov      w19, w2
021adcb8 e2031f2a mov      w2, wzr
021adcbc f40301aa mov      x20, x1
021adcc0 cdfdff97 bl       #0x21ad3f4
021adcc4 890e40f9 ldr      x9, [x20, #0x18]
021adcc8 3f050071 cmp      w9, #1
021adccc ab010054 b.lt     #0x21add00
021adcd0 297d4092 and      x9, x9, #0xffffffff
021adcd4 e8031faa mov      x8, xzr
021adcd8 2a7d4093 sxtw     x10, w9
021adcdc 8b820091 add      x11, x20, #0x20
021adce0 1f0109eb cmp      x8, x9
021adce4 42010054 b.hs     #0x21add0c
021adce8 6c696838 ldrb     w12, [x11, x8]
021adcec 8c01134a eor      w12, w12, w19
021adcf0 6c692838 strb     w12, [x11, x8]
021adcf4 08050091 add      x8, x8, #1
021adcf8 1f010aeb cmp      x8, x10
021adcfc 2bffff54 b.lt     #0x21adce0
021add00 f37b41a9 ldp      x19, x30, [sp, #0x10]
021add04 f40742f8 ldr      x20, [sp], #0x20
021add08 c0035fd6 ret      
021add0c b6baaf97 bl       #0xd9c7e4
021add10 b3baaf97 bl       #0xd9c7dc
