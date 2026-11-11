0194c710 f75bbda9 stp      x23, x22, [sp, #-0x30]!
0194c714 f55301a9 stp      x21, x20, [sp, #0x10]
0194c718 f37b02a9 stp      x19, x30, [sp, #0x20]
0194c71c 957d01f0 adrp     x21, #0x48ff000
0194c720 a8925b39 ldrb     w8, [x21, #0x6e4]
0194c724 f40301aa mov      x20, x1
0194c728 f30300aa mov      x19, x0
0194c72c e8010037 tbnz     w8, #0, #0x194c768
0194c730 c06601d0 adrp     x0, #0x4626000
0194c734 00e445f9 ldr      x0, [x0, #0xbc8]
0194c738 ed3fd197 bl       #0xd9c6ec
0194c73c 006501d0 adrp     x0, #0x45ee000
0194c740 009044f9 ldr      x0, [x0, #0x920]
0194c744 ea3fd197 bl       #0xd9c6ec
0194c748 a0640190 adrp     x0, #0x45e0000
0194c74c 005042f9 ldr      x0, [x0, #0x4a0]
0194c750 e73fd197 bl       #0xd9c6ec
0194c754 406501b0 adrp     x0, #0x45f5000
0194c758 00c843f9 ldr      x0, [x0, #0x790]
0194c75c e43fd197 bl       #0xd9c6ec
0194c760 28008052 mov      w8, #1
0194c764 a8921b39 strb     w8, [x21, #0x6e4]
0194c768 80da8052 mov      w0, #0x6d4
0194c76c e1031faa mov      x1, xzr
0194c770 b3d01094 bl       #0x1d80a3c
0194c774 80010036 tbz      w0, #0, #0x194c7a4
0194c778 80da8052 mov      w0, #0x6d4
0194c77c e1031faa mov      x1, xzr
0194c780 dcd01094 bl       #0x1d80af0
0194c784 200d00b4 cbz      x0, #0x194c928
0194c788 e10313aa mov      x1, x19
0194c78c e20314aa mov      x2, x20
0194c790 f37b42a9 ldp      x19, x30, [sp, #0x20]
0194c794 f55341a9 ldp      x21, x20, [sp, #0x10]
0194c798 e3031faa mov      x3, xzr
0194c79c f75bc3a8 ldp      x23, x22, [sp], #0x30
0194c7a0 980a0d14 b        #0x1c8f200
0194c7a4 340c00b4 cbz      x20, #0x194c928
0194c7a8 941a80b9 ldrsw    x20, [x20, #0x18]
0194c7ac 176501d0 adrp     x23, #0x45ee000
0194c7b0 f79244f9 ldr      x23, [x23, #0x920]
0194c7b4 496501b0 adrp     x9, #0x45f5000
0194c7b8 74a200b9 str      w20, [x19, #0xa0]
0194c7bc e80240f9 ldr      x8, [x23]
0194c7c0 29c943f9 ldr      x9, [x9, #0x790]
0194c7c4 085d40f9 ldr      x8, [x8, #0xb8]
0194c7c8 200140f9 ldr      x0, [x9]
0194c7cc 158140f9 ldr      x21, [x8, #0x100]
0194c7d0 09e040b9 ldr      w9, [x0, #0xe0]
0194c7d4 49000035 cbnz     w9, #0x194c7dc
0194c7d8 f83fd197 bl       #0xd9c7b8
0194c7dc b6640190 adrp     x22, #0x45e0000
0194c7e0 d65242f9 ldr      x22, [x22, #0x4a0]
0194c7e4 e00315aa mov      x0, x21
0194c7e8 e10314aa mov      x1, x20
0194c7ec e2031faa mov      x2, xzr
0194c7f0 4d0a2094 bl       #0x214f124
0194c7f4 e80240f9 ldr      x8, [x23]
0194c7f8 e1031faa mov      x1, xzr
0194c7fc 085d40f9 ldr      x8, [x8, #0xb8]
0194c800 000140f9 ldr      x0, [x8]
0194c804 b0092094 bl       #0x214eec4
0194c808 e80240f9 ldr      x8, [x23]
0194c80c e1031faa mov      x1, xzr
0194c810 085d40f9 ldr      x8, [x8, #0xb8]
0194c814 000540f9 ldr      x0, [x8, #8]
0194c818 ab092094 bl       #0x214eec4
0194c81c 68a240b9 ldr      w8, [x19, #0xa0]
0194c820 1f050071 cmp      w8, #1
0194c824 cb050054 b.lt     #0x194c8dc
0194c828 c86601d0 adrp     x8, #0x4626000
0194c82c 08e545f9 ldr      x8, [x8, #0xbc8]
0194c830 140140f9 ldr      x20, [x8]
0194c834 881e40f9 ldr      x8, [x20, #0x38]
0194c838 000140f9 ldr      x0, [x8]
0194c83c 08c84439 ldrb     w8, [x0, #0x132]
0194c840 48000037 tbnz     w8, #0, #0x194c848
0194c844 e44ed297 bl       #0xde03d4
0194c848 08e040b9 ldr      w8, [x0, #0xe0]
0194c84c 48000035 cbnz     w8, #0x194c854
0194c850 da3fd197 bl       #0xd9c7b8
0194c854 881e40f9 ldr      x8, [x20, #0x38]
0194c858 000140f9 ldr      x0, [x8]
0194c85c 08c84439 ldrb     w8, [x0, #0x132]
0194c860 48000037 tbnz     w8, #0, #0x194c868
0194c864 dc4ed297 bl       #0xde03d4
0194c868 085c40f9 ldr      x8, [x0, #0xb8]
0194c86c 60058052 mov      w0, #0x2b
0194c870 e2031faa mov      x2, xzr
0194c874 010140f9 ldr      x1, [x8]
0194c878 40b8ea97 bl       #0x13fa978
0194c87c 61a240b9 ldr      w1, [x19, #0xa0]
0194c880 e00313aa mov      x0, x19
0194c884 2a000094 bl       #0x194c92c
0194c888 c00240f9 ldr      x0, [x22]
0194c88c 08e040b9 ldr      w8, [x0, #0xe0]
0194c890 68000035 cbnz     w8, #0x194c89c
0194c894 c93fd197 bl       #0xd9c7b8
0194c898 c00240f9 ldr      x0, [x22]
0194c89c 085c40f9 ldr      x8, [x0, #0xb8]
0194c8a0 084540f9 ldr      x8, [x8, #0x88]
0194c8a4 280400b4 cbz      x8, #0x194c928
0194c8a8 001140f9 ldr      x0, [x8, #0x20]
0194c8ac e00300b4 cbz      x0, #0x194c928
0194c8b0 61a240b9 ldr      w1, [x19, #0xa0]
0194c8b4 e2031faa mov      x2, xzr
0194c8b8 621d1494 bl       #0x1e53e40
0194c8bc e00200b4 cbz      x0, #0x194c918
0194c8c0 080040f9 ldr      x8, [x0]
0194c8c4 f37b42a9 ldp      x19, x30, [sp, #0x20]
0194c8c8 f55341a9 ldp      x21, x20, [sp, #0x10]
0194c8cc 21008052 mov      w1, #1
0194c8d0 03895aa9 ldp      x3, x2, [x8, #0x1a8]
0194c8d4 f75bc3a8 ldp      x23, x22, [sp], #0x30
0194c8d8 60001fd6 br       x3
0194c8dc c00240f9 ldr      x0, [x22]
0194c8e0 08e040b9 ldr      w8, [x0, #0xe0]
0194c8e4 68000035 cbnz     w8, #0x194c8f0
0194c8e8 b43fd197 bl       #0xd9c7b8
0194c8ec c00240f9 ldr      x0, [x22]
0194c8f0 085c40f9 ldr      x8, [x0, #0xb8]
0194c8f4 084540f9 ldr      x8, [x8, #0x88]
0194c8f8 880100b4 cbz      x8, #0x194c928
0194c8fc 001140f9 ldr      x0, [x8, #0x20]
0194c900 400100b4 cbz      x0, #0x194c928
0194c904 f37b42a9 ldp      x19, x30, [sp, #0x20]
0194c908 f55341a9 ldp      x21, x20, [sp, #0x10]
0194c90c e1031faa mov      x1, xzr
0194c910 f75bc3a8 ldp      x23, x22, [sp], #0x30
0194c914 06201414 b        #0x1e5492c
0194c918 f37b42a9 ldp      x19, x30, [sp, #0x20]
0194c91c f55341a9 ldp      x21, x20, [sp, #0x10]
0194c920 f75bc3a8 ldp      x23, x22, [sp], #0x30
0194c924 c0035fd6 ret      
0194c928 ad3fd197 bl       #0xd9c7dc
