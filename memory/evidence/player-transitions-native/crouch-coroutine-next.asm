01e6222c e80f1efc str      d8, [sp, #-0x20]!
01e62230 f40700f9 str      x20, [sp, #8]
01e62234 f37b01a9 stp      x19, x30, [sp, #0x10]
01e62238 f45401f0 adrp     x20, #0x4901000
01e6223c 884e7639 ldrb     w8, [x20, #0xd93]
01e62240 f30300aa mov      x19, x0
01e62244 c8000037 tbnz     w8, #0, #0x1e6225c
01e62248 803c01f0 adrp     x0, #0x45f5000
01e6224c 00c843f9 ldr      x0, [x0, #0x790]
01e62250 27e9bc97 bl       #0xd9c6ec
01e62254 28008052 mov      w8, #1
01e62258 884e3639 strb     w8, [x20, #0xd93]
01e6225c 691240b9 ldr      w9, [x19, #0x10]
01e62260 681240f9 ldr      x8, [x19, #0x20]
01e62264 3f050071 cmp      w9, #1
01e62268 60020054 b.eq     #0x1e622b4
01e6226c c9030035 cbnz     w9, #0x1e622e4
01e62270 09008012 mov      w9, #-1
01e62274 691200b9 str      w9, [x19, #0x10]
01e62278 080400b4 cbz      x8, #0x1e622f8
01e6227c 893c01f0 adrp     x9, #0x45f5000
01e62280 29c943f9 ldr      x9, [x9, #0x790]
01e62284 085540bd ldr      s8, [x8, #0x54]
01e62288 200140f9 ldr      x0, [x9]
01e6228c 09e040b9 ldr      w9, [x0, #0xe0]
01e62290 49000035 cbnz     w9, #0x1e62298
01e62294 49e9bc97 bl       #0xd9c7b8
01e62298 001da84e mov      v0.16b, v8.16b
01e6229c e0031faa mov      x0, xzr
01e622a0 7adc0b94 bl       #0x2159488
01e622a4 600e00f9 str      x0, [x19, #0x18]
01e622a8 20008052 mov      w0, #1
01e622ac 601200b9 str      w0, [x19, #0x10]
01e622b0 0e000014 b        #0x1e622e8
01e622b4 09008012 mov      w9, #-1
01e622b8 691200b9 str      w9, [x19, #0x10]
01e622bc e80100b4 cbz      x8, #0x1e622f8
01e622c0 091d40f9 ldr      x9, [x8, #0x38]
01e622c4 2a008052 mov      w10, #1
01e622c8 0a410139 strb     w10, [x8, #0x50]
01e622cc 690100b4 cbz      x9, #0x1e622f8
01e622d0 202540f9 ldr      x0, [x9, #0x48]
01e622d4 200100b4 cbz      x0, #0x1e622f8
01e622d8 e1031f2a mov      w1, wzr
01e622dc e2031faa mov      x2, xzr
01e622e0 135d5694 bl       #0x33f972c
01e622e4 e0031f2a mov      w0, wzr
01e622e8 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e622ec f40740f9 ldr      x20, [sp, #8]
01e622f0 e80742fc ldr      d8, [sp], #0x20
01e622f4 c0035fd6 ret      
01e622f8 39e9bc97 bl       #0xd9c7dc
