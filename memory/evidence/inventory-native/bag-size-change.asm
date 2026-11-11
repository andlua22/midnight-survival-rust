0194bc78 f553bea9 stp      x21, x20, [sp, #-0x20]!
0194bc7c f37b01a9 stp      x19, x30, [sp, #0x10]
0194bc80 b57d0190 adrp     x21, #0x48ff000
0194bc84 a87e5b39 ldrb     w8, [x21, #0x6df]
0194bc88 f30301aa mov      x19, x1
0194bc8c f40300aa mov      x20, x0
0194bc90 28010037 tbnz     w8, #0, #0x194bcb4
0194bc94 006501f0 adrp     x0, #0x45ee000
0194bc98 009044f9 ldr      x0, [x0, #0x920]
0194bc9c 9442d197 bl       #0xd9c6ec
0194bca0 406501d0 adrp     x0, #0x45f5000
0194bca4 00c843f9 ldr      x0, [x0, #0x790]
0194bca8 9142d197 bl       #0xd9c6ec
0194bcac 28008052 mov      w8, #1
0194bcb0 a87e1b39 strb     w8, [x21, #0x6df]
0194bcb4 80d88052 mov      w0, #0x6c4
0194bcb8 e1031faa mov      x1, xzr
0194bcbc 60d31094 bl       #0x1d80a3c
0194bcc0 60010036 tbz      w0, #0, #0x194bcec
0194bcc4 80d88052 mov      w0, #0x6c4
0194bcc8 e1031faa mov      x1, xzr
0194bccc 89d31094 bl       #0x1d80af0
0194bcd0 400300b4 cbz      x0, #0x194bd38
0194bcd4 e20313aa mov      x2, x19
0194bcd8 f37b41a9 ldp      x19, x30, [sp, #0x10]
0194bcdc e10314aa mov      x1, x20
0194bce0 e3031faa mov      x3, xzr
0194bce4 f553c2a8 ldp      x21, x20, [sp], #0x20
0194bce8 460d0d14 b        #0x1c8f200
0194bcec 730200b4 cbz      x19, #0x194bd38
0194bcf0 681a40b9 ldr      w8, [x19, #0x18]
0194bcf4 096501f0 adrp     x9, #0x45ee000
0194bcf8 299144f9 ldr      x9, [x9, #0x920]
0194bcfc 884a00b9 str      w8, [x20, #0x48]
0194bd00 280140f9 ldr      x8, [x9]
0194bd04 496501d0 adrp     x9, #0x45f5000
0194bd08 29c943f9 ldr      x9, [x9, #0x790]
0194bd0c 085d40f9 ldr      x8, [x8, #0xb8]
0194bd10 200140f9 ldr      x0, [x9]
0194bd14 130140f9 ldr      x19, [x8]
0194bd18 09e040b9 ldr      w9, [x0, #0xe0]
0194bd1c 49000035 cbnz     w9, #0x194bd24
0194bd20 a642d197 bl       #0xd9c7b8
0194bd24 e00313aa mov      x0, x19
0194bd28 f37b41a9 ldp      x19, x30, [sp, #0x10]
0194bd2c e1031faa mov      x1, xzr
0194bd30 f553c2a8 ldp      x21, x20, [sp], #0x20
0194bd34 640c2014 b        #0x214eec4
0194bd38 a942d197 bl       #0xd9c7dc
