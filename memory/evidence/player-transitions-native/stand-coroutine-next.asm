01e7cbb4 e80f1efc str      d8, [sp, #-0x20]!
01e7cbb8 f40700f9 str      x20, [sp, #8]
01e7cbbc f37b01a9 stp      x19, x30, [sp, #0x10]
01e7cbc0 345401b0 adrp     x20, #0x4901000
01e7cbc4 885a7939 ldrb     w8, [x20, #0xe56]
01e7cbc8 f30300aa mov      x19, x0
01e7cbcc c8000037 tbnz     w8, #0, #0x1e7cbe4
01e7cbd0 c03b01b0 adrp     x0, #0x45f5000
01e7cbd4 00c843f9 ldr      x0, [x0, #0x790]
01e7cbd8 c57ebc97 bl       #0xd9c6ec
01e7cbdc 28008052 mov      w8, #1
01e7cbe0 885a3939 strb     w8, [x20, #0xe56]
01e7cbe4 691240b9 ldr      w9, [x19, #0x10]
01e7cbe8 681240f9 ldr      x8, [x19, #0x20]
01e7cbec 3f050071 cmp      w9, #1
01e7cbf0 60020054 b.eq     #0x1e7cc3c
01e7cbf4 29030035 cbnz     w9, #0x1e7cc58
01e7cbf8 09008012 mov      w9, #-1
01e7cbfc 691200b9 str      w9, [x19, #0x10]
01e7cc00 680300b4 cbz      x8, #0x1e7cc6c
01e7cc04 c93b01b0 adrp     x9, #0x45f5000
01e7cc08 29c943f9 ldr      x9, [x9, #0x790]
01e7cc0c 086d40bd ldr      s8, [x8, #0x6c]
01e7cc10 200140f9 ldr      x0, [x9]
01e7cc14 09e040b9 ldr      w9, [x0, #0xe0]
01e7cc18 49000035 cbnz     w9, #0x1e7cc20
01e7cc1c e77ebc97 bl       #0xd9c7b8
01e7cc20 001da84e mov      v0.16b, v8.16b
01e7cc24 e0031faa mov      x0, xzr
01e7cc28 18720b94 bl       #0x2159488
01e7cc2c 600e00f9 str      x0, [x19, #0x18]
01e7cc30 20008052 mov      w0, #1
01e7cc34 601200b9 str      w0, [x19, #0x10]
01e7cc38 09000014 b        #0x1e7cc5c
01e7cc3c 09008012 mov      w9, #-1
01e7cc40 691200b9 str      w9, [x19, #0x10]
01e7cc44 480100b4 cbz      x8, #0x1e7cc6c
01e7cc48 e0031f2a mov      w0, wzr
01e7cc4c 29008052 mov      w9, #1
01e7cc50 09a10139 strb     w9, [x8, #0x68]
01e7cc54 02000014 b        #0x1e7cc5c
01e7cc58 e0031f2a mov      w0, wzr
01e7cc5c f37b41a9 ldp      x19, x30, [sp, #0x10]
01e7cc60 f40740f9 ldr      x20, [sp, #8]
01e7cc64 e80742fc ldr      d8, [sp], #0x20
01e7cc68 c0035fd6 ret      
01e7cc6c dc7ebc97 bl       #0xd9c7dc
