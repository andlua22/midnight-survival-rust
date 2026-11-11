021ad2e4 f75bbda9 stp      x23, x22, [sp, #-0x30]!
021ad2e8 f55301a9 stp      x21, x20, [sp, #0x10]
021ad2ec f37b02a9 stp      x19, x30, [sp, #0x20]
021ad2f0 f303012a mov      w19, w1
021ad2f4 f40300aa mov      x20, x0
021ad2f8 cff1ff97 bl       #0x21a9a34
021ad2fc 1500134a eor      w21, w0, w19
021ad300 e00314aa mov      x0, x20
021ad304 ccf1ff97 bl       #0x21a9a34
021ad308 1600134a eor      w22, w0, w19
021ad30c e00314aa mov      x0, x20
021ad310 c9f1ff97 bl       #0x21a9a34
021ad314 1700134a eor      w23, w0, w19
021ad318 e00314aa mov      x0, x20
021ad31c c6f1ff97 bl       #0x21a9a34
021ad320 c91e0012 and      w9, w22, #0xff
021ad324 351d1833 bfi      w21, w9, #8, #8
021ad328 e91e0012 and      w9, w23, #0xff
021ad32c 0800134a eor      w8, w0, w19
021ad330 351d1033 bfi      w21, w9, #0x10, #8
021ad334 151d0833 bfi      w21, w8, #0x18, #8
021ad338 e003152a mov      w0, w21
021ad33c f37b42a9 ldp      x19, x30, [sp, #0x20]
021ad340 f55341a9 ldp      x21, x20, [sp, #0x10]
021ad344 f75bc3a8 ldp      x23, x22, [sp], #0x30
021ad348 c0035fd6 ret      
