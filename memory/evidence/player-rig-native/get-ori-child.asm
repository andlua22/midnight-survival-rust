01e4c7b0 f60f1df8 str      x22, [sp, #-0x30]!
01e4c7b4 f55301a9 stp      x21, x20, [sp, #0x10]
01e4c7b8 f37b02a9 stp      x19, x30, [sp, #0x20]
01e4c7bc b55501b0 adrp     x21, #0x4901000
01e4c7c0 a8d67339 ldrb     w8, [x21, #0xcf5]
01e4c7c4 f30301aa mov      x19, x1
01e4c7c8 f40300aa mov      x20, x0
01e4c7cc 88010037 tbnz     w8, #0, #0x1e4c7fc
01e4c7d0 c03c0190 adrp     x0, #0x45e4000
01e4c7d4 007444f9 ldr      x0, [x0, #0x8e8]
01e4c7d8 c53fbd97 bl       #0xd9c6ec
01e4c7dc 603b01f0 adrp     x0, #0x45bb000
01e4c7e0 00c447f9 ldr      x0, [x0, #0xf88]
01e4c7e4 c23fbd97 bl       #0xd9c6ec
01e4c7e8 403d01b0 adrp     x0, #0x45f5000
01e4c7ec 00c843f9 ldr      x0, [x0, #0x790]
01e4c7f0 bf3fbd97 bl       #0xd9c6ec
01e4c7f4 28008052 mov      w8, #1
01e4c7f8 a8d63339 strb     w8, [x21, #0xcf5]
01e4c7fc 807d8152 mov      w0, #0xbec
01e4c800 e1031faa mov      x1, xzr
01e4c804 ff0700f9 str      xzr, [sp, #8]
01e4c808 8dd0fc97 bl       #0x1d80a3c
01e4c80c 40010036 tbz      w0, #0, #0x1e4c834
01e4c810 807d8152 mov      w0, #0xbec
01e4c814 e1031faa mov      x1, xzr
01e4c818 b6d0fc97 bl       #0x1d80af0
01e4c81c e00700b4 cbz      x0, #0x1e4c918
01e4c820 e10314aa mov      x1, x20
01e4c824 e20313aa mov      x2, x19
01e4c828 e3031faa mov      x3, xzr
01e4c82c 295bf997 bl       #0x1ca34d0
01e4c830 36000014 b        #0x1e4c908
01e4c834 802641f9 ldr      x0, [x20, #0x248]
01e4c838 000700b4 cbz      x0, #0x1e4c918
01e4c83c c83c0190 adrp     x8, #0x45e4000
01e4c840 087544f9 ldr      x8, [x8, #0x8e8]
01e4c844 763b01f0 adrp     x22, #0x45bb000
01e4c848 e2230091 add      x2, sp, #8
01e4c84c e10313aa mov      x1, x19
01e4c850 030140f9 ldr      x3, [x8]
01e4c854 d6c647f9 ldr      x22, [x22, #0xf88]
01e4c858 c4e62694 bl       #0x2806368
01e4c85c c00240f9 ldr      x0, [x22]
01e4c860 f50740f9 ldr      x21, [sp, #8]
01e4c864 08e040b9 ldr      w8, [x0, #0xe0]
01e4c868 48000035 cbnz     w8, #0x1e4c870
01e4c86c d33fbd97 bl       #0xd9c7b8
01e4c870 e00315aa mov      x0, x21
01e4c874 e1031faa mov      x1, xzr
01e4c878 e2031faa mov      x2, xzr
01e4c87c 90695794 bl       #0x3426ebc
01e4c880 60000036 tbz      w0, #0, #0x1e4c88c
01e4c884 e00740f9 ldr      x0, [sp, #8]
01e4c888 20000014 b        #0x1e4c908
01e4c88c 553d01b0 adrp     x21, #0x45f5000
01e4c890 b5ca43f9 ldr      x21, [x21, #0x790]
01e4c894 e00314aa mov      x0, x20
01e4c898 e1031faa mov      x1, xzr
01e4c89c 14775794 bl       #0x342a4ec
01e4c8a0 a80240f9 ldr      x8, [x21]
01e4c8a4 f40300aa mov      x20, x0
01e4c8a8 09e140b9 ldr      w9, [x8, #0xe0]
01e4c8ac 69000035 cbnz     w9, #0x1e4c8b8
01e4c8b0 e00308aa mov      x0, x8
01e4c8b4 c13fbd97 bl       #0xd9c7b8
01e4c8b8 e00313aa mov      x0, x19
01e4c8bc e10314aa mov      x1, x20
01e4c8c0 e2031faa mov      x2, xzr
01e4c8c4 432e0c94 bl       #0x21581d0
01e4c8c8 c80240f9 ldr      x8, [x22]
01e4c8cc f30300aa mov      x19, x0
01e4c8d0 09e140b9 ldr      w9, [x8, #0xe0]
01e4c8d4 69000035 cbnz     w9, #0x1e4c8e0
01e4c8d8 e00308aa mov      x0, x8
01e4c8dc b73fbd97 bl       #0xd9c7b8
01e4c8e0 e00313aa mov      x0, x19
01e4c8e4 e1031faa mov      x1, xzr
01e4c8e8 7f655794 bl       #0x3425ee4
01e4c8ec e803002a mov      w8, w0
01e4c8f0 e0031faa mov      x0, xzr
01e4c8f4 a8000036 tbz      w8, #0, #0x1e4c908
01e4c8f8 130100b4 cbz      x19, #0x1e4c918
01e4c8fc e00313aa mov      x0, x19
01e4c900 e1031faa mov      x1, xzr
01e4c904 a4625794 bl       #0x3425394
01e4c908 f37b42a9 ldp      x19, x30, [sp, #0x20]
01e4c90c f55341a9 ldp      x21, x20, [sp, #0x10]
01e4c910 f60743f8 ldr      x22, [sp], #0x30
01e4c914 c0035fd6 ret      
01e4c918 b13fbd97 bl       #0xd9c7dc
