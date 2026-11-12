0133ff38 f75bbda9 stp      x23, x22, [sp, #-0x30]!
0133ff3c f55301a9 stp      x21, x20, [sp, #0x10]
0133ff40 f37b02a9 stp      x19, x30, [sp, #0x20]
0133ff44 f7ad01d0 adrp     x23, #0x48fd000
0133ff48 e8825339 ldrb     w8, [x23, #0x4e0]
0133ff4c f30303aa mov      x19, x3
0133ff50 f60302aa mov      x22, x2
0133ff54 f40301aa mov      x20, x1
0133ff58 f50300aa mov      x21, x0
0133ff5c e8010037 tbnz     w8, #0, #0x133ff98
0133ff60 c0940190 adrp     x0, #0x45d7000
0133ff64 005443f9 ldr      x0, [x0, #0x6a8]
0133ff68 e171e997 bl       #0xd9c6ec
0133ff6c c0930190 adrp     x0, #0x45b7000
0133ff70 003841f9 ldr      x0, [x0, #0x270]
0133ff74 de71e997 bl       #0xd9c6ec
0133ff78 a09301b0 adrp     x0, #0x45b4000
0133ff7c 007840f9 ldr      x0, [x0, #0xf0]
0133ff80 db71e997 bl       #0xd9c6ec
0133ff84 c09501d0 adrp     x0, #0x45f9000
0133ff88 004844f9 ldr      x0, [x0, #0x890]
0133ff8c d871e997 bl       #0xd9c6ec
0133ff90 28008052 mov      w8, #1
0133ff94 e8821339 strb     w8, [x23, #0x4e0]
0133ff98 605c8052 mov      w0, #0x2e3
0133ff9c e1031faa mov      x1, xzr
0133ffa0 a7022994 bl       #0x1d80a3c
0133ffa4 c0010036 tbz      w0, #0, #0x133ffdc
0133ffa8 605c8052 mov      w0, #0x2e3
0133ffac e1031faa mov      x1, xzr
0133ffb0 d0022994 bl       #0x1d80af0
0133ffb4 200900b4 cbz      x0, #0x13400d8
0133ffb8 e10315aa mov      x1, x21
0133ffbc e20314aa mov      x2, x20
0133ffc0 e40313aa mov      x4, x19
0133ffc4 f37b42a9 ldp      x19, x30, [sp, #0x20]
0133ffc8 f55341a9 ldp      x21, x20, [sp, #0x10]
0133ffcc e30316aa mov      x3, x22
0133ffd0 e5031faa mov      x5, xzr
0133ffd4 f75bc3a8 ldp      x23, x22, [sp], #0x30
0133ffd8 9e702514 b        #0x1c9c250
0133ffdc d7940190 adrp     x23, #0x45d7000
0133ffe0 f75643f9 ldr      x23, [x23, #0x6a8]
0133ffe4 e00240f9 ldr      x0, [x23]
0133ffe8 08e040b9 ldr      w8, [x0, #0xe0]
0133ffec 68000035 cbnz     w8, #0x133fff8
0133fff0 f271e997 bl       #0xd9c7b8
0133fff4 e00240f9 ldr      x0, [x23]
0133fff8 085c40f9 ldr      x8, [x0, #0xb8]
0133fffc 001140f9 ldr      x0, [x8, #0x20]
01340000 c00600b4 cbz      x0, #0x13400d8
01340004 c89501b0 adrp     x8, #0x45f9000
01340008 084944f9 ldr      x8, [x8, #0x890]
0134000c e10316aa mov      x1, x22
01340010 020140f9 ldr      x2, [x8]
01340014 7e6a5a94 bl       #0x29daa0c
01340018 20010036 tbz      w0, #0, #0x134003c
0134001c 730500b4 cbz      x19, #0x13400c8
01340020 620e40f9 ldr      x2, [x19, #0x18]
01340024 602240f9 ldr      x0, [x19, #0x40]
01340028 611640f9 ldr      x1, [x19, #0x28]
0134002c f37b42a9 ldp      x19, x30, [sp, #0x20]
01340030 f55341a9 ldp      x21, x20, [sp, #0x10]
01340034 f75bc3a8 ldp      x23, x22, [sp], #0x30
01340038 40001fd6 br       x2
0134003c e00240f9 ldr      x0, [x23]
01340040 08e040b9 ldr      w8, [x0, #0xe0]
01340044 68000035 cbnz     w8, #0x1340050
01340048 dc71e997 bl       #0xd9c7b8
0134004c e00240f9 ldr      x0, [x23]
01340050 085c40f9 ldr      x8, [x0, #0xb8]
01340054 001140f9 ldr      x0, [x8, #0x20]
01340058 000400b4 cbz      x0, #0x13400d8
0134005c a8930190 adrp     x8, #0x45b4000
01340060 087940f9 ldr      x8, [x8, #0xf0]
01340064 e10316aa mov      x1, x22
01340068 020140f9 ldr      x2, [x8]
0134006c 216d5a94 bl       #0x29db4f0
01340070 e80240f9 ldr      x8, [x23]
01340074 085d40f9 ldr      x8, [x8, #0xb8]
01340078 000d40f9 ldr      x0, [x8, #0x18]
0134007c e00200b4 cbz      x0, #0x13400d8
01340080 a89301f0 adrp     x8, #0x45b7000
01340084 083941f9 ldr      x8, [x8, #0x270]
01340088 e10316aa mov      x1, x22
0134008c 020140f9 ldr      x2, [x8]
01340090 b4ee4894 bl       #0x257bb60
01340094 f60300aa mov      x22, x0
01340098 b30000b4 cbz      x19, #0x13400ac
0134009c 680e40f9 ldr      x8, [x19, #0x18]
013400a0 602240f9 ldr      x0, [x19, #0x40]
013400a4 611640f9 ldr      x1, [x19, #0x28]
013400a8 00013fd6 blr      x8
013400ac e00315aa mov      x0, x21
013400b0 e10314aa mov      x1, x20
013400b4 f37b42a9 ldp      x19, x30, [sp, #0x20]
013400b8 f55341a9 ldp      x21, x20, [sp, #0x10]
013400bc e20316aa mov      x2, x22
013400c0 f75bc3a8 ldp      x23, x22, [sp], #0x30
013400c4 06000014 b        #0x13400dc
013400c8 f37b42a9 ldp      x19, x30, [sp, #0x20]
013400cc f55341a9 ldp      x21, x20, [sp, #0x10]
013400d0 f75bc3a8 ldp      x23, x22, [sp], #0x30
013400d4 c0035fd6 ret      
013400d8 c171e997 bl       #0xd9c7dc
