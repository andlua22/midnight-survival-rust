01e7c5e8 f40f1ef8 str      x20, [sp, #-0x20]!
01e7c5ec f37b01a9 stp      x19, x30, [sp, #0x10]
01e7c5f0 345401b0 adrp     x20, #0x4901000
01e7c5f4 884a7939 ldrb     w8, [x20, #0xe52]
01e7c5f8 f30300aa mov      x19, x0
01e7c5fc 88010037 tbnz     w8, #0, #0x1e7c62c
01e7c600 203d01d0 adrp     x0, #0x4622000
01e7c604 00e440f9 ldr      x0, [x0, #0x1c8]
01e7c608 3980bc97 bl       #0xd9c6ec
01e7c60c a03c01b0 adrp     x0, #0x4611000
01e7c610 00d443f9 ldr      x0, [x0, #0x7a8]
01e7c614 3680bc97 bl       #0xd9c6ec
01e7c618 403a01b0 adrp     x0, #0x45c5000
01e7c61c 00e843f9 ldr      x0, [x0, #0x7d0]
01e7c620 3380bc97 bl       #0xd9c6ec
01e7c624 28008052 mov      w8, #1
01e7c628 884a3939 strb     w8, [x20, #0xe52]
01e7c62c a0db8552 mov      w0, #0x2edd
01e7c630 e1031faa mov      x1, xzr
01e7c634 0211fc97 bl       #0x1d80a3c
01e7c638 40010036 tbz      w0, #0, #0x1e7c660
01e7c63c a0db8552 mov      w0, #0x2edd
01e7c640 e1031faa mov      x1, xzr
01e7c644 2b11fc97 bl       #0x1d80af0
01e7c648 000600b4 cbz      x0, #0x1e7c708
01e7c64c e10313aa mov      x1, x19
01e7c650 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e7c654 e2031faa mov      x2, xzr
01e7c658 f40742f8 ldr      x20, [sp], #0x20
01e7c65c a64ef817 b        #0x1c900f4
01e7c660 601e40f9 ldr      x0, [x19, #0x38]
01e7c664 200500b4 cbz      x0, #0x1e7c708
01e7c668 e1031faa mov      x1, xzr
01e7c66c b1e40094 bl       #0x1eb5930
01e7c670 c0000037 tbnz     w0, #0, #0x1e7c688
01e7c674 601e40f9 ldr      x0, [x19, #0x38]
01e7c678 800400b4 cbz      x0, #0x1e7c708
01e7c67c e1031faa mov      x1, xzr
01e7c680 b5f60094 bl       #0x1eba154
01e7c684 80000036 tbz      w0, #0, #0x1e7c694
01e7c688 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e7c68c f40742f8 ldr      x20, [sp], #0x20
01e7c690 c0035fd6 ret      
01e7c694 283d01d0 adrp     x8, #0x4622000
01e7c698 08e540f9 ldr      x8, [x8, #0x1c8]
01e7c69c 080140f9 ldr      x8, [x8]
01e7c6a0 095d40f9 ldr      x9, [x8, #0xb8]
01e7c6a4 681e40f9 ldr      x8, [x19, #0x38]
01e7c6a8 2a014039 ldrb     w10, [x9]
01e7c6ac 6a000034 cbz      w10, #0x1e7c6b8
01e7c6b0 a80000b5 cbnz     x8, #0x1e7c6c4
01e7c6b4 15000014 b        #0x1e7c708
01e7c6b8 880200b4 cbz      x8, #0x1e7c708
01e7c6bc 0a014f39 ldrb     w10, [x8, #0x3c0]
01e7c6c0 6a010034 cbz      w10, #0x1e7c6ec
01e7c6c4 00b941f9 ldr      x0, [x8, #0x370]
01e7c6c8 000200b4 cbz      x0, #0x1e7c708
01e7c6cc a83c01b0 adrp     x8, #0x4611000
01e7c6d0 08d543f9 ldr      x8, [x8, #0x7a8]
01e7c6d4 010140f9 ldr      x1, [x8]
01e7c6d8 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e7c6dc 22008052 mov      w2, #1
01e7c6e0 e3031faa mov      x3, xzr
01e7c6e4 f40742f8 ldr      x20, [sp], #0x20
01e7c6e8 c8c8e117 b        #0x16eea08
01e7c6ec 29054039 ldrb     w9, [x9, #1]
01e7c6f0 c9fcff34 cbz      w9, #0x1e7c688
01e7c6f4 00b941f9 ldr      x0, [x8, #0x370]
01e7c6f8 800000b4 cbz      x0, #0x1e7c708
01e7c6fc 483a01b0 adrp     x8, #0x45c5000
01e7c700 08e943f9 ldr      x8, [x8, #0x7d0]
01e7c704 f4ffff17 b        #0x1e7c6d4
01e7c708 3580bc97 bl       #0xd9c7dc
Capstone 5.0: decoded 73 instructions / 292 bytes
