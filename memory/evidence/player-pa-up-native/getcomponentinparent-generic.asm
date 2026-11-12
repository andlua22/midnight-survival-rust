025811ec f60f1df8 str      x22, [sp, #-0x30]!
025811f0 f55301a9 stp      x21, x20, [sp, #0x10]
025811f4 f37b02a9 stp      x19, x30, [sp, #0x20]
025811f8 361c0190 adrp     x22, #0x4905000
025811fc 950201b0 adrp     x21, #0x45d2000
02581200 c8e24839 ldrb     w8, [x22, #0x238]
02581204 b5e242f9 ldr      x21, [x21, #0x5c0]
02581208 f30301aa mov      x19, x1
0258120c f40300aa mov      x20, x0
02581210 c8000037 tbnz     w8, #0, #0x2581228
02581214 800201b0 adrp     x0, #0x45d2000
02581218 00e042f9 ldr      x0, [x0, #0x5c0]
0258121c 346da097 bl       #0xd9c6ec
02581220 28008052 mov      w8, #1
02581224 c8e20839 strb     w8, [x22, #0x238]
02581228 a00240f9 ldr      x0, [x21]
0258122c 681e40f9 ldr      x8, [x19, #0x38]
02581230 09e040b9 ldr      w9, [x0, #0xe0]
02581234 150140f9 ldr      x21, [x8]
02581238 49000035 cbnz     w9, #0x2581240
0258123c 5f6da097 bl       #0xd9c7b8
02581240 e00315aa mov      x0, x21
02581244 e1031faa mov      x1, xzr
02581248 1a912d94 bl       #0x30e56b0
0258124c e10300aa mov      x1, x0
02581250 e00314aa mov      x0, x20
02581254 e2031f2a mov      w2, wzr
02581258 e3031faa mov      x3, xzr
0258125c 08a53a94 bl       #0x342a67c
02581260 681e40f9 ldr      x8, [x19, #0x38]
02581264 f40300aa mov      x20, x0
02581268 130540f9 ldr      x19, [x8, #8]
0258126c 68ca4439 ldrb     w8, [x19, #0x132]
02581270 88000037 tbnz     w8, #0, #0x2581280
02581274 e00313aa mov      x0, x19
02581278 577ca197 bl       #0xde03d4
0258127c f30300aa mov      x19, x0
02581280 140100b4 cbz      x20, #0x25812a0
02581284 e00314aa mov      x0, x20
02581288 e10313aa mov      x1, x19
0258128c 4c6da097 bl       #0xd9c7bc
02581290 a00000b5 cbnz     x0, #0x25812a4
02581294 e00314aa mov      x0, x20
02581298 e10313aa mov      x1, x19
0258129c ff6da097 bl       #0xd9ca98
025812a0 e0031faa mov      x0, xzr
025812a4 f37b42a9 ldp      x19, x30, [sp, #0x20]
025812a8 f55341a9 ldp      x21, x20, [sp, #0x10]
025812ac f60743f8 ldr      x22, [sp], #0x30
025812b0 c0035fd6 ret      
Capstone 5.0: decoded 50 instructions / 200 bytes
