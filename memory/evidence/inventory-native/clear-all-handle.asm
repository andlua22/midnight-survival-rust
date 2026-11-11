0194ba50 f553bea9 stp      x21, x20, [sp, #-0x20]!
0194ba54 f37b01a9 stp      x19, x30, [sp, #0x10]
0194ba58 b57d0190 adrp     x21, #0x48ff000
0194ba5c a87a5b39 ldrb     w8, [x21, #0x6de]
0194ba60 f30301aa mov      x19, x1
0194ba64 f40300aa mov      x20, x0
0194ba68 88010037 tbnz     w8, #0, #0x194ba98
0194ba6c 606601f0 adrp     x0, #0x461a000
0194ba70 007446f9 ldr      x0, [x0, #0xce8]
0194ba74 1e43d197 bl       #0xd9c6ec
0194ba78 e06401d0 adrp     x0, #0x45e9000
0194ba7c 008840f9 ldr      x0, [x0, #0x110]
0194ba80 1b43d197 bl       #0xd9c6ec
0194ba84 206501d0 adrp     x0, #0x45f1000
0194ba88 004842f9 ldr      x0, [x0, #0x490]
0194ba8c 1843d197 bl       #0xd9c6ec
0194ba90 28008052 mov      w8, #1
0194ba94 a87a1b39 strb     w8, [x21, #0x6de]
0194ba98 c0d78052 mov      w0, #0x6be
0194ba9c e1031faa mov      x1, xzr
0194baa0 e7d31094 bl       #0x1d80a3c
0194baa4 60010036 tbz      w0, #0, #0x194bad0
0194baa8 c0d78052 mov      w0, #0x6be
0194baac e1031faa mov      x1, xzr
0194bab0 10d41094 bl       #0x1d80af0
0194bab4 200400b4 cbz      x0, #0x194bb38
0194bab8 e20313aa mov      x2, x19
0194babc f37b41a9 ldp      x19, x30, [sp, #0x10]
0194bac0 e10314aa mov      x1, x20
0194bac4 e3031faa mov      x3, xzr
0194bac8 f553c2a8 ldp      x21, x20, [sp], #0x20
0194bacc cd0d0d14 b        #0x1c8f200
0194bad0 336501d0 adrp     x19, #0x45f1000
0194bad4 734a42f9 ldr      x19, [x19, #0x490]
0194bad8 e00314aa mov      x0, x20
0194badc 18000094 bl       #0x194bb3c
0194bae0 680240f9 ldr      x8, [x19]
0194bae4 085d40f9 ldr      x8, [x8, #0xb8]
0194bae8 000140f9 ldr      x0, [x8]
0194baec 600200b4 cbz      x0, #0x194bb38
0194baf0 e86401d0 adrp     x8, #0x45e9000
0194baf4 088940f9 ldr      x8, [x8, #0x110]
0194baf8 010140f9 ldr      x1, [x8]
0194bafc 296b3294 bl       #0x25e67a0
0194bb00 60010036 tbz      w0, #0, #0x194bb2c
0194bb04 680240f9 ldr      x8, [x19]
0194bb08 085d40f9 ldr      x8, [x8, #0xb8]
0194bb0c 000140f9 ldr      x0, [x8]
0194bb10 400100b4 cbz      x0, #0x194bb38
0194bb14 686601f0 adrp     x8, #0x461a000
0194bb18 087546f9 ldr      x8, [x8, #0xce8]
0194bb1c f37b41a9 ldp      x19, x30, [sp, #0x10]
0194bb20 010140f9 ldr      x1, [x8]
0194bb24 f553c2a8 ldp      x21, x20, [sp], #0x20
0194bb28 926a3214 b        #0x25e6570
0194bb2c f37b41a9 ldp      x19, x30, [sp, #0x10]
0194bb30 f553c2a8 ldp      x21, x20, [sp], #0x20
0194bb34 c0035fd6 ret      
0194bb38 2943d197 bl       #0xd9c7dc
