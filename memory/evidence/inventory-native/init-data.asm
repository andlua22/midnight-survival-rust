01954dd4 ff4302d1 sub      sp, sp, #0x90
01954dd8 e82300fd str      d8, [sp, #0x40]
01954ddc f96305a9 stp      x25, x24, [sp, #0x50]
01954de0 f75b06a9 stp      x23, x22, [sp, #0x60]
01954de4 f55307a9 stp      x21, x20, [sp, #0x70]
01954de8 f37b08a9 stp      x19, x30, [sp, #0x80]
01954dec 547d01f0 adrp     x20, #0x48ff000
01954df0 88ea5b39 ldrb     w8, [x20, #0x6fa]
01954df4 f30300aa mov      x19, x0
01954df8 28040037 tbnz     w8, #0, #0x1954e7c
01954dfc 406501d0 adrp     x0, #0x45fe000
01954e00 00cc45f9 ldr      x0, [x0, #0xb98]
01954e04 3a1ed197 bl       #0xd9c6ec
01954e08 406301b0 adrp     x0, #0x45bd000
01954e0c 00f045f9 ldr      x0, [x0, #0xbe0]
01954e10 371ed197 bl       #0xd9c6ec
01954e14 c0650190 adrp     x0, #0x460c000
01954e18 005c41f9 ldr      x0, [x0, #0x2b8]
01954e1c 341ed197 bl       #0xd9c6ec
01954e20 a06301b0 adrp     x0, #0x45c9000
01954e24 009041f9 ldr      x0, [x0, #0x320]
01954e28 311ed197 bl       #0xd9c6ec
01954e2c e06201b0 adrp     x0, #0x45b1000
01954e30 00c840f9 ldr      x0, [x0, #0x190]
01954e34 2e1ed197 bl       #0xd9c6ec
01954e38 406401f0 adrp     x0, #0x45df000
01954e3c 00c442f9 ldr      x0, [x0, #0x588]
01954e40 2b1ed197 bl       #0xd9c6ec
01954e44 606401f0 adrp     x0, #0x45e3000
01954e48 000842f9 ldr      x0, [x0, #0x410]
01954e4c 281ed197 bl       #0xd9c6ec
01954e50 80630190 adrp     x0, #0x45c4000
01954e54 00f042f9 ldr      x0, [x0, #0x5e0]
01954e58 251ed197 bl       #0xd9c6ec
01954e5c a06501f0 adrp     x0, #0x460b000
01954e60 000042f9 ldr      x0, [x0, #0x400]
01954e64 221ed197 bl       #0xd9c6ec
01954e68 a06501d0 adrp     x0, #0x460a000
01954e6c 007c41f9 ldr      x0, [x0, #0x2f8]
01954e70 1f1ed197 bl       #0xd9c6ec
01954e74 28008052 mov      w8, #1
01954e78 88ea1b39 strb     w8, [x20, #0x6fa]
01954e7c 80ec8052 mov      w0, #0x764
01954e80 e1031faa mov      x1, xzr
01954e84 ffff02a9 stp      xzr, xzr, [sp, #0x28]
01954e88 ff1300f9 str      xzr, [sp, #0x20]
01954e8c ecae1094 bl       #0x1d80a3c
01954e90 20010036 tbz      w0, #0, #0x1954eb4
01954e94 80ec8052 mov      w0, #0x764
01954e98 e1031faa mov      x1, xzr
01954e9c 15af1094 bl       #0x1d80af0
01954ea0 e01100b4 cbz      x0, #0x19550dc
01954ea4 e10313aa mov      x1, x19
01954ea8 e2031faa mov      x2, xzr
01954eac 92ec0c94 bl       #0x1c900f4
01954eb0 8c000014 b        #0x19550e0
01954eb4 606a40f9 ldr      x0, [x19, #0xd0]
01954eb8 201100b4 cbz      x0, #0x19550dc
01954ebc 486501d0 adrp     x8, #0x45fe000
01954ec0 08cd45f9 ldr      x8, [x8, #0xb98]
01954ec4 b46501f0 adrp     x20, #0x460b000
01954ec8 010140f9 ldr      x1, [x8]
01954ecc 940242f9 ldr      x20, [x20, #0x400]
01954ed0 87833894 bl       #0x2775cec
01954ed4 e0031faa mov      x0, xzr
01954ed8 a63a6b94 bl       #0x3423970
01954edc 800240f9 ldr      x0, [x20]
01954ee0 081ca04e mov      v8.16b, v0.16b
01954ee4 08e040b9 ldr      w8, [x0, #0xe0]
01954ee8 48000035 cbnz     w8, #0x1954ef0
01954eec 331ed197 bl       #0xd9c7b8
01954ef0 08f0af52 mov      w8, #0x7f800000
01954ef4 600e40f9 ldr      x0, [x19, #0x18]
01954ef8 00c1241e frintp   s0, s8
01954efc 0101271e fmov     s1, w8
01954f00 0901281e fcvtps   w9, s8
01954f04 0020211e fcmp     s0, s1
01954f08 0800b052 mov      w8, #-0x80000000
01954f0c 1601891a csel     w22, w8, w9, eq
01954f10 600e00b4 cbz      x0, #0x19550dc
01954f14 486301b0 adrp     x8, #0x45bd000
01954f18 08f145f9 ldr      x8, [x8, #0xbe0]
01954f1c 010140f9 ldr      x1, [x8]
01954f20 13783894 bl       #0x2772f6c
01954f24 c00d00b4 cbz      x0, #0x19550dc
01954f28 a86501d0 adrp     x8, #0x460a000
01954f2c 087d41f9 ldr      x8, [x8, #0x2f8]
01954f30 f56201b0 adrp     x21, #0x45b1000
01954f34 b5ca40f9 ldr      x21, [x21, #0x190]
01954f38 d7650190 adrp     x23, #0x460c000
01954f3c 010140f9 ldr      x1, [x8]
01954f40 98630190 adrp     x24, #0x45c4000
01954f44 b46301b0 adrp     x20, #0x45c9000
01954f48 f75e41f9 ldr      x23, [x23, #0x2b8]
01954f4c 18f342f9 ldr      x24, [x24, #0x5e0]
01954f50 949241f9 ldr      x20, [x20, #0x320]
01954f54 e8230091 add      x8, sp, #8
01954f58 c8895394 bl       #0x2e37678
01954f5c e083c03c ldur     q0, [sp, #8]
01954f60 e80f40f9 ldr      x8, [sp, #0x18]
01954f64 e00b803d str      q0, [sp, #0x20]
01954f68 e81b00f9 str      x8, [sp, #0x30]
01954f6c a10240f9 ldr      x1, [x21]
01954f70 e0830091 add      x0, sp, #0x20
01954f74 42f23d94 bl       #0x28d187c
01954f78 a0040036 tbz      w0, #0, #0x195500c
01954f7c f91b40f9 ldr      x25, [sp, #0x30]
01954f80 f90b00b4 cbz      x25, #0x19550fc
01954f84 201740b9 ldr      w0, [x25, #0x14]
01954f88 e1031faa mov      x1, xzr
01954f8c ecab2394 bl       #0x223ff3c
01954f90 800b00b4 cbz      x0, #0x1955100
01954f94 088c40b9 ldr      w8, [x0, #0x8c]
01954f98 1f050071 cmp      w8, #1
01954f9c 8b000054 b.lt     #0x1954fac
01954fa0 282340b9 ldr      w8, [x25, #0x20]
01954fa4 0801160b add      w8, w8, w22
01954fa8 282300b9 str      w8, [x25, #0x20]
01954fac 201740b9 ldr      w0, [x25, #0x14]
01954fb0 e1031faa mov      x1, xzr
01954fb4 f5142294 bl       #0x21da388
01954fb8 a0fdffb4 cbz      x0, #0x1954f6c
01954fbc 281b40f9 ldr      x8, [x25, #0x30]
01954fc0 280a00b4 cbz      x8, #0x1955104
01954fc4 092540b9 ldr      w9, [x8, #0x24]
01954fc8 3f110071 cmp      w9, #4
01954fcc 01fdff54 b.ne     #0x1954f6c
01954fd0 e00308aa mov      x0, x8
01954fd4 e1031faa mov      x1, xzr
01954fd8 7a612194 bl       #0x21ad5c0
01954fdc 201b40f9 ldr      x0, [x25, #0x30]
01954fe0 600900b4 cbz      x0, #0x195510c
01954fe4 081ca04e mov      v8.16b, v0.16b
01954fe8 e1031faa mov      x1, xzr
01954fec 195e2194 bl       #0x21ac850
01954ff0 606a40f9 ldr      x0, [x19, #0xd0]
01954ff4 a00800b4 cbz      x0, #0x1955108
01954ff8 211340b9 ldr      w1, [x25, #0x10]
01954ffc e20240f9 ldr      x2, [x23]
01955000 001da84e mov      v0.16b, v8.16b
01955004 ce823894 bl       #0x2775b3c
01955008 d9ffff17 b        #0x1954f6c
0195500c 810240f9 ldr      x1, [x20]
01955010 e0830091 add      x0, sp, #0x20
01955014 19f23d94 bl       #0x28d1878
01955018 680a40f9 ldr      x8, [x19, #0x10]
0195501c 080600b4 cbz      x8, #0x19550dc
01955020 191940b9 ldr      w25, [x8, #0x18]
01955024 3f070071 cmp      w25, #1
01955028 cb050054 b.lt     #0x19550e0
0195502c f4031f2a mov      w20, wzr
01955030 020340f9 ldr      x2, [x24]
01955034 e00308aa mov      x0, x8
01955038 e103142a mov      w1, w20
0195503c a8134894 bl       #0x2b59edc
01955040 e00400b4 cbz      x0, #0x19550dc
01955044 f50300aa mov      x21, x0
01955048 001440b9 ldr      w0, [x0, #0x14]
0195504c e1031faa mov      x1, xzr
01955050 bbab2394 bl       #0x223ff3c
01955054 400400b4 cbz      x0, #0x19550dc
01955058 088c40b9 ldr      w8, [x0, #0x8c]
0195505c 1f050071 cmp      w8, #1
01955060 8b000054 b.lt     #0x1955070
01955064 a82240b9 ldr      w8, [x21, #0x20]
01955068 0801160b add      w8, w8, w22
0195506c a82200b9 str      w8, [x21, #0x20]
01955070 a01640b9 ldr      w0, [x21, #0x14]
01955074 e1031faa mov      x1, xzr
01955078 c4142294 bl       #0x21da388
0195507c 600200b4 cbz      x0, #0x19550c8
01955080 a01a40f9 ldr      x0, [x21, #0x30]
01955084 c00200b4 cbz      x0, #0x19550dc
01955088 082440b9 ldr      w8, [x0, #0x24]
0195508c 1f110071 cmp      w8, #4
01955090 c1010054 b.ne     #0x19550c8
01955094 e1031faa mov      x1, xzr
01955098 4a612194 bl       #0x21ad5c0
0195509c a01a40f9 ldr      x0, [x21, #0x30]
019550a0 e00100b4 cbz      x0, #0x19550dc
019550a4 e1031faa mov      x1, xzr
019550a8 081ca04e mov      v8.16b, v0.16b
019550ac e95d2194 bl       #0x21ac850
019550b0 606a40f9 ldr      x0, [x19, #0xd0]
019550b4 400100b4 cbz      x0, #0x19550dc
019550b8 a11240b9 ldr      w1, [x21, #0x10]
019550bc e20240f9 ldr      x2, [x23]
019550c0 001da84e mov      v0.16b, v8.16b
019550c4 9e823894 bl       #0x2775b3c
019550c8 94060011 add      w20, w20, #1
019550cc 3f03146b cmp      w25, w20
019550d0 80000054 b.eq     #0x19550e0
019550d4 680a40f9 ldr      x8, [x19, #0x10]
019550d8 c8faffb5 cbnz     x8, #0x1955030
019550dc c01dd197 bl       #0xd9c7dc
019550e0 f37b48a9 ldp      x19, x30, [sp, #0x80]
019550e4 f55347a9 ldp      x21, x20, [sp, #0x70]
019550e8 f75b46a9 ldp      x23, x22, [sp, #0x60]
019550ec f96345a9 ldp      x25, x24, [sp, #0x50]
019550f0 e82340fd ldr      d8, [sp, #0x40]
019550f4 ff430291 add      sp, sp, #0x90
019550f8 c0035fd6 ret      
019550fc b81dd197 bl       #0xd9c7dc
01955100 b71dd197 bl       #0xd9c7dc
01955104 b61dd197 bl       #0xd9c7dc
01955108 b51dd197 bl       #0xd9c7dc
0195510c b41dd197 bl       #0xd9c7dc
01955110 0a000014 b        #0x1955138
01955114 09000014 b        #0x1955138
01955118 08000014 b        #0x1955138
0195511c 07000014 b        #0x1955138
01955120 06000014 b        #0x1955138
01955124 05000014 b        #0x1955138
01955128 04000014 b        #0x1955138
0195512c 03000014 b        #0x1955138
01955130 02000014 b        #0x1955138
01955134 01000014 b        #0x1955138
01955138 f40300aa mov      x20, x0
0195513c 3f040071 cmp      w1, #1
01955140 a1010054 b.ne     #0x1955174
01955144 e00314aa mov      x0, x20
01955148 a2e6c397 bl       #0xa4ebd0
0195514c 150040f9 ldr      x21, [x0]
01955150 1ce5c397 bl       #0xa4e5c0
01955154 a8630190 adrp     x8, #0x45c9000
01955158 089141f9 ldr      x8, [x8, #0x320]
0195515c e0830091 add      x0, sp, #0x20
01955160 010140f9 ldr      x1, [x8]
01955164 c5f13d94 bl       #0x28d1878
01955168 95f5ffb4 cbz      x21, #0x1955018
0195516c e00315aa mov      x0, x21
01955170 d4a5d297 bl       #0xdfe8c0
01955174 f5031faa mov      x21, xzr
01955178 02000014 b        #0x1955180
0195517c f40300aa mov      x20, x0
01955180 a8630190 adrp     x8, #0x45c9000
01955184 089141f9 ldr      x8, [x8, #0x320]
01955188 010140f9 ldr      x1, [x8]
0195518c e0830091 add      x0, sp, #0x20
01955190 baf13d94 bl       #0x28d1878
01955194 750000b5 cbnz     x21, #0x19551a0
01955198 e00314aa mov      x0, x20
0195519c d9e4c397 bl       #0xa4e500
019551a0 e00315aa mov      x0, x21
019551a4 c7a5d297 bl       #0xdfe8c0
019551a8 64ecc397 bl       #0xa50338
