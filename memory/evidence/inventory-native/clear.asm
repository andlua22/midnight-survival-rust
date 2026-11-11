0194bb3c f40f1ef8 str      x20, [sp, #-0x20]!
0194bb40 f37b01a9 stp      x19, x30, [sp, #0x10]
0194bb44 b47d0190 adrp     x20, #0x48ff000
0194bb48 88b65d39 ldrb     w8, [x20, #0x76d]
0194bb4c f30300aa mov      x19, x0
0194bb50 28010037 tbnz     w8, #0, #0x194bb74
0194bb54 806501f0 adrp     x0, #0x45fe000
0194bb58 00cc45f9 ldr      x0, [x0, #0xb98]
0194bb5c e442d197 bl       #0xd9c6ec
0194bb60 606601f0 adrp     x0, #0x461a000
0194bb64 004444f9 ldr      x0, [x0, #0x888]
0194bb68 e142d197 bl       #0xd9c6ec
0194bb6c 28008052 mov      w8, #1
0194bb70 88b61d39 strb     w8, [x20, #0x76d]
0194bb74 e0d78052 mov      w0, #0x6bf
0194bb78 e1031faa mov      x1, xzr
0194bb7c b0d31094 bl       #0x1d80a3c
0194bb80 40010036 tbz      w0, #0, #0x194bba8
0194bb84 e0d78052 mov      w0, #0x6bf
0194bb88 e1031faa mov      x1, xzr
0194bb8c d9d31094 bl       #0x1d80af0
0194bb90 600400b4 cbz      x0, #0x194bc1c
0194bb94 e10313aa mov      x1, x19
0194bb98 f37b41a9 ldp      x19, x30, [sp, #0x10]
0194bb9c e2031faa mov      x2, xzr
0194bba0 f40742f8 ldr      x20, [sp], #0x20
0194bba4 54110d14 b        #0x1c900f4
0194bba8 e00313aa mov      x0, x19
0194bbac 095a0094 bl       #0x19623d0
0194bbb0 610a40f9 ldr      x1, [x19, #0x10]
0194bbb4 e00313aa mov      x0, x19
0194bbb8 515a0094 bl       #0x19624fc
0194bbbc 610e40f9 ldr      x1, [x19, #0x18]
0194bbc0 e00313aa mov      x0, x19
0194bbc4 8e5a0094 bl       #0x19625fc
0194bbc8 614640f9 ldr      x1, [x19, #0x88]
0194bbcc e00313aa mov      x0, x19
0194bbd0 4b5a0094 bl       #0x19624fc
0194bbd4 614a40f9 ldr      x1, [x19, #0x90]
0194bbd8 e00313aa mov      x0, x19
0194bbdc 485a0094 bl       #0x19624fc
0194bbe0 684e40f9 ldr      x8, [x19, #0x98]
0194bbe4 c80100b4 cbz      x8, #0x194bc1c
0194bbe8 091d40b9 ldr      w9, [x8, #0x1c]
0194bbec 29050011 add      w9, w9, #1
0194bbf0 1f250329 stp      wzr, w9, [x8, #0x18]
0194bbf4 606a40f9 ldr      x0, [x19, #0xd0]
0194bbf8 200100b4 cbz      x0, #0x194bc1c
0194bbfc 886501f0 adrp     x8, #0x45fe000
0194bc00 08cd45f9 ldr      x8, [x8, #0xb98]
0194bc04 010140f9 ldr      x1, [x8]
0194bc08 39a83894 bl       #0x2775cec
0194bc0c 7f3200f9 str      xzr, [x19, #0x60]
0194bc10 f37b41a9 ldp      x19, x30, [sp, #0x10]
0194bc14 f40742f8 ldr      x20, [sp], #0x20
0194bc18 c0035fd6 ret      
0194bc1c f042d197 bl       #0xd9c7dc
