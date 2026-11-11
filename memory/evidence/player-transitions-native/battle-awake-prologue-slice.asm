020503f4 ffc304d1 sub      sp, sp, #0x130
020503f8 fc6f0da9 stp      x28, x27, [sp, #0xd0]
020503fc fa670ea9 stp      x26, x25, [sp, #0xe0]
02050400 f85f0fa9 stp      x24, x23, [sp, #0xf0]
02050404 f65710a9 stp      x22, x21, [sp, #0x100]
02050408 f44f11a9 stp      x20, x19, [sp, #0x110]
0205040c fd7b12a9 stp      x29, x30, [sp, #0x120]
02050410 944501d0 adrp     x20, #0x4902000
02050414 883a6d39 ldrb     w8, [x20, #0xb4e]
02050418 f30300aa mov      x19, x0
0205041c 481d0037 tbnz     w8, #0, #0x20507c4
02050420 802c01b0 adrp     x0, #0x45e1000
02050424 001844f9 ldr      x0, [x0, #0x830]
02050428 b130b597 bl       #0xd9c6ec
0205042c 002c01d0 adrp     x0, #0x45d2000
02050430 000c44f9 ldr      x0, [x0, #0x818]
02050434 ae30b597 bl       #0xd9c6ec
02050438 402b01d0 adrp     x0, #0x45ba000
0205043c 00b440f9 ldr      x0, [x0, #0x168]
02050440 ab30b597 bl       #0xd9c6ec
02050444 802c0190 adrp     x0, #0x45e0000
02050448 005042f9 ldr      x0, [x0, #0x4a0]
0205044c a830b597 bl       #0xd9c6ec
