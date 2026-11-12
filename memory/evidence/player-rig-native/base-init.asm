01e4c094 f60f1df8 str      x22, [sp, #-0x30]!
01e4c098 f55301a9 stp      x21, x20, [sp, #0x10]
01e4c09c f37b02a9 stp      x19, x30, [sp, #0x20]
01e4c0a0 b45501b0 adrp     x20, #0x4901000
01e4c0a4 88b27339 ldrb     w8, [x20, #0xcec]
01e4c0a8 f30300aa mov      x19, x0
01e4c0ac a8020037 tbnz     w8, #0, #0x1e4c100
01e4c0b0 a03c0190 adrp     x0, #0x45e0000
01e4c0b4 005042f9 ldr      x0, [x0, #0x4a0]
01e4c0b8 8d41bd97 bl       #0xd9c6ec
01e4c0bc 403c01f0 adrp     x0, #0x45d7000
01e4c0c0 006442f9 ldr      x0, [x0, #0x4c8]
01e4c0c4 8a41bd97 bl       #0xd9c6ec
01e4c0c8 603b01f0 adrp     x0, #0x45bb000
01e4c0cc 00c447f9 ldr      x0, [x0, #0xf88]
01e4c0d0 8741bd97 bl       #0xd9c6ec
01e4c0d4 403d01b0 adrp     x0, #0x45f5000
01e4c0d8 00c843f9 ldr      x0, [x0, #0x790]
01e4c0dc 8441bd97 bl       #0xd9c6ec
01e4c0e0 803f0190 adrp     x0, #0x463c000
01e4c0e4 002446f9 ldr      x0, [x0, #0xc48]
01e4c0e8 8141bd97 bl       #0xd9c6ec
01e4c0ec 403e0190 adrp     x0, #0x4614000
01e4c0f0 00b840f9 ldr      x0, [x0, #0x170]
01e4c0f4 7e41bd97 bl       #0xd9c6ec
01e4c0f8 28008052 mov      w8, #1
01e4c0fc 88b23339 strb     w8, [x20, #0xcec]
01e4c100 a07e8152 mov      w0, #0xbf5
01e4c104 e1031faa mov      x1, xzr
01e4c108 4dd2fc97 bl       #0x1d80a3c
01e4c10c 60010036 tbz      w0, #0, #0x1e4c138
01e4c110 a07e8152 mov      w0, #0xbf5
01e4c114 e1031faa mov      x1, xzr
01e4c118 76d2fc97 bl       #0x1d80af0
01e4c11c 600e00b4 cbz      x0, #0x1e4c2e8
01e4c120 e10313aa mov      x1, x19
01e4c124 f37b42a9 ldp      x19, x30, [sp, #0x20]
01e4c128 f55341a9 ldp      x21, x20, [sp, #0x10]
01e4c12c e2031faa mov      x2, xzr
01e4c130 f60743f8 ldr      x22, [sp], #0x30
01e4c134 f00ff917 b        #0x1c900f4
01e4c138 b43c0190 adrp     x20, #0x45e0000
01e4c13c 953f0190 adrp     x21, #0x463c000
01e4c140 945242f9 ldr      x20, [x20, #0x4a0]
01e4c144 b52646f9 ldr      x21, [x21, #0xc48]
01e4c148 e00313aa mov      x0, x19
01e4c14c 68000094 bl       #0x1e4c2ec
01e4c150 800240f9 ldr      x0, [x20]
01e4c154 74f240f9 ldr      x20, [x19, #0x1e0]
01e4c158 08e040b9 ldr      w8, [x0, #0xe0]
01e4c15c 48000035 cbnz     w8, #0x1e4c164
01e4c160 9641bd97 bl       #0xd9c7b8
01e4c164 e00314aa mov      x0, x20
01e4c168 e1031faa mov      x1, xzr
01e4c16c 4e3e0894 bl       #0x205baa4
01e4c170 60f640f9 ldr      x0, [x19, #0x1e8]
01e4c174 e1031faa mov      x1, xzr
01e4c178 4b3e0894 bl       #0x205baa4
01e4c17c e00313aa mov      x0, x19
01e4c180 3f010094 bl       #0x1e4c67c
01e4c184 e1031faa mov      x1, xzr
01e4c188 8a300894 bl       #0x20583b0
01e4c18c 60f200f9 str      x0, [x19, #0x1e0]
01e4c190 e00313aa mov      x0, x19
01e4c194 61010094 bl       #0x1e4c718
01e4c198 e1031faa mov      x1, xzr
01e4c19c 85300894 bl       #0x20583b0
01e4c1a0 60f600f9 str      x0, [x19, #0x1e8]
01e4c1a4 a10240f9 ldr      x1, [x21]
01e4c1a8 e00313aa mov      x0, x19
01e4c1ac 81010094 bl       #0x1e4c7b0
01e4c1b0 c00900b4 cbz      x0, #0x1e4c2e8
01e4c1b4 483c01f0 adrp     x8, #0x45d7000
01e4c1b8 086542f9 ldr      x8, [x8, #0x4c8]
01e4c1bc 753b01f0 adrp     x21, #0x45bb000
01e4c1c0 010140f9 ldr      x1, [x8]
01e4c1c4 b5c647f9 ldr      x21, [x21, #0xf88]
01e4c1c8 7ed31c94 bl       #0x2580fc0
01e4c1cc 602e00f9 str      x0, [x19, #0x58]
01e4c1d0 08008012 mov      w8, #-1
01e4c1d4 e00313aa mov      x0, x19
01e4c1d8 68ee02b9 str      w8, [x19, #0x2ec]
01e4c1dc 68a201b9 str      w8, [x19, #0x1a0]
01e4c1e0 cf010094 bl       #0x1e4c91c
01e4c1e4 e00313aa mov      x0, x19
01e4c1e8 54020094 bl       #0x1e4cb38
01e4c1ec a00240f9 ldr      x0, [x21]
01e4c1f0 74be40f9 ldr      x20, [x19, #0x178]
01e4c1f4 08e040b9 ldr      w8, [x0, #0xe0]
01e4c1f8 48000035 cbnz     w8, #0x1e4c200
01e4c1fc 6f41bd97 bl       #0xd9c7b8
01e4c200 563d01b0 adrp     x22, #0x45f5000
01e4c204 d6ca43f9 ldr      x22, [x22, #0x790]
01e4c208 e00314aa mov      x0, x20
01e4c20c e1031faa mov      x1, xzr
01e4c210 e2031faa mov      x2, xzr
01e4c214 2a6b5794 bl       #0x3426ebc
01e4c218 40010036 tbz      w0, #0, #0x1e4c240
01e4c21c a00240f9 ldr      x0, [x21]
01e4c220 74be40f9 ldr      x20, [x19, #0x178]
01e4c224 08e040b9 ldr      w8, [x0, #0xe0]
01e4c228 48000035 cbnz     w8, #0x1e4c230
01e4c22c 6341bd97 bl       #0xd9c7b8
01e4c230 e00314aa mov      x0, x20
01e4c234 e1031faa mov      x1, xzr
01e4c238 dd695794 bl       #0x34269ac
01e4c23c 7fbe00f9 str      xzr, [x19, #0x178]
01e4c240 c00240f9 ldr      x0, [x22]
01e4c244 08e040b9 ldr      w8, [x0, #0xe0]
01e4c248 48000035 cbnz     w8, #0x1e4c250
01e4c24c 5b41bd97 bl       #0xd9c7b8
01e4c250 e0031faa mov      x0, xzr
01e4c254 a32c0c94 bl       #0x21574e0
01e4c258 00040036 tbz      w0, #0, #0x1e4c2d8
01e4c25c 483e0190 adrp     x8, #0x4614000
01e4c260 08b940f9 ldr      x8, [x8, #0x170]
01e4c264 e00313aa mov      x0, x19
01e4c268 010140f9 ldr      x1, [x8]
01e4c26c 51010094 bl       #0x1e4c7b0
01e4c270 a80240f9 ldr      x8, [x21]
01e4c274 f30300aa mov      x19, x0
01e4c278 09e140b9 ldr      w9, [x8, #0xe0]
01e4c27c 69000035 cbnz     w9, #0x1e4c288
01e4c280 e00308aa mov      x0, x8
01e4c284 4d41bd97 bl       #0xd9c7b8
01e4c288 e00313aa mov      x0, x19
01e4c28c e1031faa mov      x1, xzr
01e4c290 15675794 bl       #0x3425ee4
01e4c294 20020036 tbz      w0, #0, #0x1e4c2d8
01e4c298 930200b4 cbz      x19, #0x1e4c2e8
01e4c29c e00313aa mov      x0, x19
01e4c2a0 e1031faa mov      x1, xzr
01e4c2a4 92785794 bl       #0x342a4ec
01e4c2a8 a80240f9 ldr      x8, [x21]
01e4c2ac f30300aa mov      x19, x0
01e4c2b0 09e140b9 ldr      w9, [x8, #0xe0]
01e4c2b4 69000035 cbnz     w9, #0x1e4c2c0
01e4c2b8 e00308aa mov      x0, x8
01e4c2bc 3f41bd97 bl       #0xd9c7b8
01e4c2c0 e00313aa mov      x0, x19
01e4c2c4 f37b42a9 ldp      x19, x30, [sp, #0x20]
01e4c2c8 f55341a9 ldp      x21, x20, [sp, #0x10]
01e4c2cc e1031faa mov      x1, xzr
01e4c2d0 f60743f8 ldr      x22, [sp], #0x30
01e4c2d4 b6695714 b        #0x34269ac
01e4c2d8 f37b42a9 ldp      x19, x30, [sp, #0x20]
01e4c2dc f55341a9 ldp      x21, x20, [sp, #0x10]
01e4c2e0 f60743f8 ldr      x22, [sp], #0x30
01e4c2e4 c0035fd6 ret      
01e4c2e8 3d41bd97 bl       #0xd9c7dc
