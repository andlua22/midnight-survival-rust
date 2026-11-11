01ebe220 e80f1efc str      d8, [sp, #-0x20]!
01ebe224 f40700f9 str      x20, [sp, #8]
01ebe228 f37b01a9 stp      x19, x30, [sp, #0x10]
01ebe22c 145201f0 adrp     x20, #0x4901000
01ebe230 889e7f39 ldrb     w8, [x20, #0xfe7]
01ebe234 081ca04e mov      v8.16b, v0.16b
01ebe238 f30300aa mov      x19, x0
01ebe23c c8000037 tbnz     w8, #0, #0x1ebe254
01ebe240 603901b0 adrp     x0, #0x45eb000
01ebe244 008046f9 ldr      x0, [x0, #0xd00]
01ebe248 2979bb97 bl       #0xd9c6ec
01ebe24c 28008052 mov      w8, #1
01ebe250 889e3f39 strb     w8, [x20, #0xfe7]
01ebe254 40b48552 mov      w0, #0x2da2
01ebe258 e1031faa mov      x1, xzr
01ebe25c f809fb97 bl       #0x1d80a3c
01ebe260 80010036 tbz      w0, #0, #0x1ebe290
01ebe264 40b48552 mov      w0, #0x2da2
01ebe268 e1031faa mov      x1, xzr
01ebe26c 210afb97 bl       #0x1d80af0
01ebe270 400700b4 cbz      x0, #0x1ebe358
01ebe274 e10313aa mov      x1, x19
01ebe278 f37b41a9 ldp      x19, x30, [sp, #0x10]
01ebe27c f40740f9 ldr      x20, [sp, #8]
01ebe280 001da84e mov      v0.16b, v8.16b
01ebe284 e2031faa mov      x2, xzr
01ebe288 e80742fc ldr      d8, [sp], #0x20
01ebe28c f173f717 b        #0x1c9b250
01ebe290 743901b0 adrp     x20, #0x45eb000
01ebe294 948246f9 ldr      x20, [x20, #0xd00]
01ebe298 68325239 ldrb     w8, [x19, #0x48c]
01ebe29c e003271e fmov     s0, wzr
01ebe2a0 800240f9 ldr      x0, [x20]
01ebe2a4 1f010072 tst      w8, #1
01ebe2a8 080d201e fcsel    s8, s8, s0, eq
01ebe2ac 09e040b9 ldr      w9, [x0, #0xe0]
01ebe2b0 49000035 cbnz     w9, #0x1ebe2b8
01ebe2b4 4179bb97 bl       #0xd9c7b8
01ebe2b8 e00313aa mov      x0, x19
01ebe2bc e1031faa mov      x1, xzr
01ebe2c0 7cb05594 bl       #0x342a4b0
01ebe2c4 a00400b4 cbz      x0, #0x1ebe358
01ebe2c8 e1031faa mov      x1, xzr
01ebe2cc 74eb5594 bl       #0x343909c
01ebe2d0 880240f9 ldr      x8, [x20]
01ebe2d4 0009201e fmul     s0, s8, s0
01ebe2d8 e00313aa mov      x0, x19
01ebe2dc 085d40f9 ldr      x8, [x8, #0xb8]
01ebe2e0 002500bd str      s0, [x8, #0x24]
01ebe2e4 fac0ff97 bl       #0x1eae6cc
01ebe2e8 800300b4 cbz      x0, #0x1ebe358
01ebe2ec e1031faa mov      x1, xzr
01ebe2f0 64f25694 bl       #0x347ac80
01ebe2f4 880240f9 ldr      x8, [x20]
01ebe2f8 e00313aa mov      x0, x19
01ebe2fc e1031faa mov      x1, xzr
01ebe300 085d40f9 ldr      x8, [x8, #0xb8]
01ebe304 012900bd str      s1, [x8, #0x28]
01ebe308 6ab05594 bl       #0x342a4b0
01ebe30c 600200b4 cbz      x0, #0x1ebe358
01ebe310 e1031faa mov      x1, xzr
01ebe314 62eb5594 bl       #0x343909c
01ebe318 880240f9 ldr      x8, [x20]
01ebe31c 0009221e fmul     s0, s8, s2
01ebe320 e00313aa mov      x0, x19
01ebe324 085d40f9 ldr      x8, [x8, #0xb8]
01ebe328 002d00bd str      s0, [x8, #0x2c]
01ebe32c e8c0ff97 bl       #0x1eae6cc
01ebe330 400100b4 cbz      x0, #0x1ebe358
01ebe334 880240f9 ldr      x8, [x20]
01ebe338 f37b41a9 ldp      x19, x30, [sp, #0x10]
01ebe33c f40740f9 ldr      x20, [sp, #8]
01ebe340 e1031faa mov      x1, xzr
01ebe344 085d40f9 ldr      x8, [x8, #0xb8]
01ebe348 0109452d ldp      s1, s2, [x8, #0x28]
01ebe34c 002540bd ldr      s0, [x8, #0x24]
01ebe350 e80742fc ldr      d8, [sp], #0x20
01ebe354 73f25614 b        #0x347ad20
01ebe358 2179bb97 bl       #0xd9c7dc
Capstone 5.0: decoded 79 instructions / 316 bytes
