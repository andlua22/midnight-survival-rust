02dd5d58 f553bea9 stp      x21, x20, [sp, #-0x20]!
02dd5d5c f37b01a9 stp      x19, x30, [sp, #0x10]
02dd5d60 95d900b0 adrp     x21, #0x4906000
02dd5d64 a8024a39 ldrb     w8, [x21, #0x280]
02dd5d68 f40301aa mov      x20, x1
02dd5d6c f30300aa mov      x19, x0
02dd5d70 c8000037 tbnz     w8, #0, #0x2dd5d88
02dd5d74 00c10090 adrp     x0, #0x45f5000
02dd5d78 00c843f9 ldr      x0, [x0, #0x790]
02dd5d7c 5c1a7f97 bl       #0xd9c6ec
02dd5d80 28008052 mov      w8, #1
02dd5d84 a8020a39 strb     w8, [x21, #0x280]
02dd5d88 68a24039 ldrb     w8, [x19, #0x28]
02dd5d8c 28040034 cbz      w8, #0x2dd5e10
02dd5d90 08c10090 adrp     x8, #0x45f5000
02dd5d94 08c943f9 ldr      x8, [x8, #0x790]
02dd5d98 751240f9 ldr      x21, [x19, #0x20]
02dd5d9c 000140f9 ldr      x0, [x8]
02dd5da0 08e040b9 ldr      w8, [x0, #0xe0]
02dd5da4 48000035 cbnz     w8, #0x2dd5dac
02dd5da8 841a7f97 bl       #0xd9c7b8
02dd5dac e00315aa mov      x0, x21
02dd5db0 e1031faa mov      x1, xzr
02dd5db4 44e4cd97 bl       #0x214eec4
02dd5db8 881240f9 ldr      x8, [x20, #0x20]
02dd5dbc e00313aa mov      x0, x19
02dd5dc0 086140f9 ldr      x8, [x8, #0xc0]
02dd5dc4 013140f9 ldr      x1, [x8, #0x60]
02dd5dc8 280440f9 ldr      x8, [x1, #8]
02dd5dcc 00013fd6 blr      x8
02dd5dd0 600200b4 cbz      x0, #0x2dd5e1c
02dd5dd4 080040f9 ldr      x8, [x0]
02dd5dd8 09855ba9 ldp      x9, x1, [x8, #0x1b8]
02dd5ddc 20013fd6 blr      x9
02dd5de0 881240f9 ldr      x8, [x20, #0x20]
02dd5de4 e00313aa mov      x0, x19
02dd5de8 086140f9 ldr      x8, [x8, #0xc0]
02dd5dec 013140f9 ldr      x1, [x8, #0x60]
02dd5df0 280440f9 ldr      x8, [x1, #8]
02dd5df4 00013fd6 blr      x8
02dd5df8 200100b4 cbz      x0, #0x2dd5e1c
02dd5dfc 080040f9 ldr      x8, [x0]
02dd5e00 f37b41a9 ldp      x19, x30, [sp, #0x10]
02dd5e04 02855ca9 ldp      x2, x1, [x8, #0x1c8]
02dd5e08 f553c2a8 ldp      x21, x20, [sp], #0x20
02dd5e0c 40001fd6 br       x2
02dd5e10 f37b41a9 ldp      x19, x30, [sp, #0x10]
02dd5e14 f553c2a8 ldp      x21, x20, [sp], #0x20
02dd5e18 c0035fd6 ret      
02dd5e1c 701a7f97 bl       #0xd9c7dc
