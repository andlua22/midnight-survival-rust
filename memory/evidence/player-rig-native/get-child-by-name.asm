021581d0 f60f1df8 str      x22, [sp, #-0x30]!
021581d4 f55301a9 stp      x21, x20, [sp, #0x10]
021581d8 f37b02a9 stp      x19, x30, [sp, #0x20]
021581dc 553d01f0 adrp     x21, #0x4903000
021581e0 a8624a39 ldrb     w8, [x21, #0x298]
021581e4 f40301aa mov      x20, x1
021581e8 f30300aa mov      x19, x0
021581ec c8000037 tbnz     w8, #0, #0x2158204
021581f0 c02301b0 adrp     x0, #0x45d1000
021581f4 00c847f9 ldr      x0, [x0, #0xf90]
021581f8 3d11b197 bl       #0xd9c6ec
021581fc 28008052 mov      w8, #1
02158200 a8620a39 strb     w8, [x21, #0x298]
02158204 a04f8052 mov      w0, #0x27d
02158208 e1031faa mov      x1, xzr
0215820c 0ca2f097 bl       #0x1d80a3c
02158210 80010036 tbz      w0, #0, #0x2158240
02158214 a04f8052 mov      w0, #0x27d
02158218 e1031faa mov      x1, xzr
0215821c 35a2f097 bl       #0x1d80af0
02158220 600600b4 cbz      x0, #0x21582ec
02158224 e10313aa mov      x1, x19
02158228 e20314aa mov      x2, x20
0215822c f37b42a9 ldp      x19, x30, [sp, #0x20]
02158230 f55341a9 ldp      x21, x20, [sp, #0x10]
02158234 e3031faa mov      x3, xzr
02158238 f60743f8 ldr      x22, [sp], #0x30
0215823c b10ced17 b        #0x1c9b500
02158240 740500b4 cbz      x20, #0x21582ec
02158244 c82301b0 adrp     x8, #0x45d1000
02158248 08c947f9 ldr      x8, [x8, #0xf90]
0215824c 21008052 mov      w1, #1
02158250 e00314aa mov      x0, x20
02158254 020140f9 ldr      x2, [x8]
02158258 00161194 bl       #0x259da58
0215825c 800400b4 cbz      x0, #0x21582ec
02158260 081840b9 ldr      w8, [x0, #0x18]
02158264 f40300aa mov      x20, x0
02158268 1f050071 cmp      w8, #1
0215826c 2b020054 b.lt     #0x21582b0
02158270 f5031f2a mov      w21, wzr
02158274 bf02086b cmp      w21, w8
02158278 c2030054 b.hs     #0x21582f0
0215827c 96ce358b add      x22, x20, w21, sxtw #3
02158280 c00e42f8 ldr      x0, [x22, #0x20]!
02158284 400300b4 cbz      x0, #0x21582ec
02158288 e1031faa mov      x1, xzr
0215828c 48374b94 bl       #0x3425fac
02158290 e10313aa mov      x1, x19
02158294 e2031faa mov      x2, xzr
02158298 95933894 bl       #0x2f7d0ec
0215829c 40010037 tbnz     w0, #0, #0x21582c4
021582a0 881a40b9 ldr      w8, [x20, #0x18]
021582a4 b5060011 add      w21, w21, #1
021582a8 bf02086b cmp      w21, w8
021582ac 4bfeff54 b.lt     #0x2158274
021582b0 f37b42a9 ldp      x19, x30, [sp, #0x20]
021582b4 f55341a9 ldp      x21, x20, [sp, #0x10]
021582b8 e0031faa mov      x0, xzr
021582bc f60743f8 ldr      x22, [sp], #0x30
021582c0 c0035fd6 ret      
021582c4 881a40b9 ldr      w8, [x20, #0x18]
021582c8 bf02086b cmp      w21, w8
021582cc 22010054 b.hs     #0x21582f0
021582d0 c00240f9 ldr      x0, [x22]
021582d4 c00000b4 cbz      x0, #0x21582ec
021582d8 f37b42a9 ldp      x19, x30, [sp, #0x20]
021582dc f55341a9 ldp      x21, x20, [sp, #0x10]
021582e0 e1031faa mov      x1, xzr
021582e4 f60743f8 ldr      x22, [sp], #0x30
021582e8 81484b14 b        #0x342a4ec
021582ec 3c11b197 bl       #0xd9c7dc
021582f0 3d11b197 bl       #0xd9c7e4
