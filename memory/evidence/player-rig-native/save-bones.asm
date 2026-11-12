01e4c2ec ffc301d1 sub      sp, sp, #0x70
01e4c2f0 fc6f01a9 stp      x28, x27, [sp, #0x10]
01e4c2f4 fa6702a9 stp      x26, x25, [sp, #0x20]
01e4c2f8 f85f03a9 stp      x24, x23, [sp, #0x30]
01e4c2fc f65704a9 stp      x22, x21, [sp, #0x40]
01e4c300 f44f05a9 stp      x20, x19, [sp, #0x50]
01e4c304 fd7b06a9 stp      x29, x30, [sp, #0x60]
01e4c308 b45501b0 adrp     x20, #0x4901000
01e4c30c 88de7339 ldrb     w8, [x20, #0xcf7]
01e4c310 f30300aa mov      x19, x0
01e4c314 e8040037 tbnz     w8, #0, #0x1e4c3b0
01e4c318 003e01b0 adrp     x0, #0x460d000
01e4c31c 006046f9 ldr      x0, [x0, #0xcc0]
01e4c320 f340bd97 bl       #0xd9c6ec
01e4c324 003f01f0 adrp     x0, #0x462f000
01e4c328 003c46f9 ldr      x0, [x0, #0xc78]
01e4c32c f040bd97 bl       #0xd9c6ec
01e4c330 003b01d0 adrp     x0, #0x45ae000
01e4c334 00b040f9 ldr      x0, [x0, #0x160]
01e4c338 ed40bd97 bl       #0xd9c6ec
01e4c33c 003e01b0 adrp     x0, #0x460d000
01e4c340 005843f9 ldr      x0, [x0, #0x6b0]
01e4c344 ea40bd97 bl       #0xd9c6ec
01e4c348 c03c0190 adrp     x0, #0x45e4000
01e4c34c 007444f9 ldr      x0, [x0, #0x8e8]
01e4c350 e740bd97 bl       #0xd9c6ec
01e4c354 e03c01f0 adrp     x0, #0x45eb000
01e4c358 007047f9 ldr      x0, [x0, #0xee0]
01e4c35c e440bd97 bl       #0xd9c6ec
01e4c360 403c0190 adrp     x0, #0x45d4000
01e4c364 001041f9 ldr      x0, [x0, #0x220]
01e4c368 e140bd97 bl       #0xd9c6ec
01e4c36c 603b01f0 adrp     x0, #0x45bb000
01e4c370 00c447f9 ldr      x0, [x0, #0xf88]
01e4c374 de40bd97 bl       #0xd9c6ec
01e4c378 203c01f0 adrp     x0, #0x45d3000
01e4c37c 002045f9 ldr      x0, [x0, #0xa40]
01e4c380 db40bd97 bl       #0xd9c6ec
01e4c384 e03d01d0 adrp     x0, #0x460a000
01e4c388 004444f9 ldr      x0, [x0, #0x888]
01e4c38c d840bd97 bl       #0xd9c6ec
01e4c390 403f01d0 adrp     x0, #0x4636000
01e4c394 003c44f9 ldr      x0, [x0, #0x878]
01e4c398 d540bd97 bl       #0xd9c6ec
01e4c39c 403b01b0 adrp     x0, #0x45b5000
01e4c3a0 008440f9 ldr      x0, [x0, #0x108]
01e4c3a4 d240bd97 bl       #0xd9c6ec
01e4c3a8 28008052 mov      w8, #1
01e4c3ac 88de3339 strb     w8, [x20, #0xcf7]
01e4c3b0 c07e8152 mov      w0, #0xbf6
01e4c3b4 e1031faa mov      x1, xzr
01e4c3b8 ff0700f9 str      xzr, [sp, #8]
01e4c3bc a0d1fc97 bl       #0x1d80a3c
01e4c3c0 20010036 tbz      w0, #0, #0x1e4c3e4
01e4c3c4 c07e8152 mov      w0, #0xbf6
01e4c3c8 e1031faa mov      x1, xzr
01e4c3cc c9d1fc97 bl       #0x1d80af0
01e4c3d0 e00d00b4 cbz      x0, #0x1e4c58c
01e4c3d4 e10313aa mov      x1, x19
01e4c3d8 e2031faa mov      x2, xzr
01e4c3dc 460ff997 bl       #0x1c900f4
01e4c3e0 9b000014 b        #0x1e4c64c
01e4c3e4 083e01b0 adrp     x8, #0x460d000
01e4c3e8 086146f9 ldr      x8, [x8, #0xcc0]
01e4c3ec 7a3b01f0 adrp     x26, #0x45bb000
01e4c3f0 e00313aa mov      x0, x19
01e4c3f4 010140f9 ldr      x1, [x8]
01e4c3f8 5ac747f9 ldr      x26, [x26, #0xf88]
01e4c3fc f1d21c94 bl       #0x2580fc0
01e4c400 480340f9 ldr      x8, [x26]
01e4c404 f40300aa mov      x20, x0
01e4c408 09e140b9 ldr      w9, [x8, #0xe0]
01e4c40c 69000035 cbnz     w9, #0x1e4c418
01e4c410 e00308aa mov      x0, x8
01e4c414 e940bd97 bl       #0xd9c7b8
01e4c418 193f01f0 adrp     x25, #0x462f000
01e4c41c 393f46f9 ldr      x25, [x25, #0xc78]
01e4c420 e00314aa mov      x0, x20
01e4c424 e1031faa mov      x1, xzr
01e4c428 e2031faa mov      x2, xzr
01e4c42c 6d665794 bl       #0x3425de0
01e4c430 00010036 tbz      w0, #0, #0x1e4c450
01e4c434 543f01d0 adrp     x20, #0x4636000
01e4c438 943e44f9 ldr      x20, [x20, #0x878]
01e4c43c e00313aa mov      x0, x19
01e4c440 e1031faa mov      x1, xzr
01e4c444 da665794 bl       #0x3425fac
01e4c448 880240f9 ldr      x8, [x20]
01e4c44c 73000014 b        #0x1e4c618
01e4c450 f40900b4 cbz      x20, #0x1e4c58c
01e4c454 880e40f9 ldr      x8, [x20, #0x18]
01e4c458 a80900b4 cbz      x8, #0x1e4c58c
01e4c45c 293c01f0 adrp     x9, #0x45d3000
01e4c460 292145f9 ldr      x9, [x9, #0xa40]
01e4c464 011940b9 ldr      w1, [x8, #0x18]
01e4c468 200140f9 ldr      x0, [x9]
01e4c46c ae40bd97 bl       #0xd9c724
01e4c470 601e01f9 str      x0, [x19, #0x238]
01e4c474 800e40f9 ldr      x0, [x20, #0x18]
01e4c478 a00800b4 cbz      x0, #0x1e4c58c
01e4c47c 5b3c0190 adrp     x27, #0x45d4000
01e4c480 1c3e01b0 adrp     x28, #0x460d000
01e4c484 1d3b01d0 adrp     x29, #0x45ae000
01e4c488 7b1341f9 ldr      x27, [x27, #0x220]
01e4c48c 9c5b43f9 ldr      x28, [x28, #0x6b0]
01e4c490 bdb340f9 ldr      x29, [x29, #0x160]
01e4c494 f5031faa mov      x21, xzr
01e4c498 081880b9 ldrsw    x8, [x0, #0x18]
01e4c49c bf0208eb cmp      x21, x8
01e4c4a0 8a070054 b.ge     #0x1e4c590
01e4c4a4 620340f9 ldr      x2, [x27]
01e4c4a8 e103152a mov      w1, w21
01e4c4ac 8c363494 bl       #0x2b59edc
01e4c4b0 480340f9 ldr      x8, [x26]
01e4c4b4 f60300aa mov      x22, x0
01e4c4b8 09e140b9 ldr      w9, [x8, #0xe0]
01e4c4bc 69000035 cbnz     w9, #0x1e4c4c8
01e4c4c0 e00308aa mov      x0, x8
01e4c4c4 bd40bd97 bl       #0xd9c7b8
01e4c4c8 e00316aa mov      x0, x22
01e4c4cc e1031faa mov      x1, xzr
01e4c4d0 85665794 bl       #0x3425ee4
01e4c4d4 60050036 tbz      w0, #0, #0x1e4c580
01e4c4d8 b60500b4 cbz      x22, #0x1e4c58c
01e4c4dc 781e41f9 ldr      x24, [x19, #0x238]
01e4c4e0 e00316aa mov      x0, x22
01e4c4e4 e1031faa mov      x1, xzr
01e4c4e8 ab635794 bl       #0x3425394
01e4c4ec 180500b4 cbz      x24, #0x1e4c58c
01e4c4f0 f70300aa mov      x23, x0
01e4c4f4 c00000b4 cbz      x0, #0x1e4c50c
01e4c4f8 080340f9 ldr      x8, [x24]
01e4c4fc e00317aa mov      x0, x23
01e4c500 012140f9 ldr      x1, [x8, #0x40]
01e4c504 ae40bd97 bl       #0xd9c7bc
01e4c508 400b00b4 cbz      x0, #0x1e4c670
01e4c50c 081b40b9 ldr      w8, [x24, #0x18]
01e4c510 bf0208eb cmp      x21, x8
01e4c514 c20a0054 b.hs     #0x1e4c66c
01e4c518 080f158b add      x8, x24, x21, lsl #3
01e4c51c 171100f9 str      x23, [x8, #0x20]
01e4c520 772641f9 ldr      x23, [x19, #0x248]
01e4c524 e00316aa mov      x0, x22
01e4c528 e1031faa mov      x1, xzr
01e4c52c a0665794 bl       #0x3425fac
01e4c530 f70200b4 cbz      x23, #0x1e4c58c
01e4c534 820340f9 ldr      x2, [x28]
01e4c538 e10300aa mov      x1, x0
01e4c53c e00317aa mov      x0, x23
01e4c540 1be22694 bl       #0x2804dac
01e4c544 e0010037 tbnz     w0, #0, #0x1e4c580
01e4c548 772641f9 ldr      x23, [x19, #0x248]
01e4c54c e00316aa mov      x0, x22
01e4c550 e1031faa mov      x1, xzr
01e4c554 96665794 bl       #0x3425fac
01e4c558 f80300aa mov      x24, x0
01e4c55c e00316aa mov      x0, x22
01e4c560 e1031faa mov      x1, xzr
01e4c564 8c635794 bl       #0x3425394
01e4c568 370100b4 cbz      x23, #0x1e4c58c
01e4c56c a30340f9 ldr      x3, [x29]
01e4c570 e20300aa mov      x2, x0
01e4c574 e00317aa mov      x0, x23
01e4c578 e10318aa mov      x1, x24
01e4c57c 8ce12694 bl       #0x2804bac
01e4c580 800e40f9 ldr      x0, [x20, #0x18]
01e4c584 b5060091 add      x21, x21, #1
01e4c588 80f8ffb5 cbnz     x0, #0x1e4c498
01e4c58c 9440bd97 bl       #0xd9c7dc
01e4c590 602641f9 ldr      x0, [x19, #0x248]
01e4c594 c0ffffb4 cbz      x0, #0x1e4c58c
01e4c598 483b01b0 adrp     x8, #0x45b5000
01e4c59c 088540f9 ldr      x8, [x8, #0x108]
01e4c5a0 e2230091 add      x2, sp, #8
01e4c5a4 010140f9 ldr      x1, [x8]
01e4c5a8 c83c0190 adrp     x8, #0x45e4000
01e4c5ac 087544f9 ldr      x8, [x8, #0x8e8]
01e4c5b0 030140f9 ldr      x3, [x8]
01e4c5b4 6de72694 bl       #0x2806368
01e4c5b8 00010036 tbz      w0, #0, #0x1e4c5d8
01e4c5bc e00740f9 ldr      x0, [sp, #8]
01e4c5c0 60feffb4 cbz      x0, #0x1e4c58c
01e4c5c4 e1031faa mov      x1, xzr
01e4c5c8 c9775794 bl       #0x342a4ec
01e4c5cc f40300aa mov      x20, x0
01e4c5d0 602201f9 str      x0, [x19, #0x240]
01e4c5d4 02000014 b        #0x1e4c5dc
01e4c5d8 742241f9 ldr      x20, [x19, #0x240]
01e4c5dc 400340f9 ldr      x0, [x26]
01e4c5e0 08e040b9 ldr      w8, [x0, #0xe0]
01e4c5e4 48000035 cbnz     w8, #0x1e4c5ec
01e4c5e8 7440bd97 bl       #0xd9c7b8
01e4c5ec e00314aa mov      x0, x20
01e4c5f0 e1031faa mov      x1, xzr
01e4c5f4 e2031faa mov      x2, xzr
01e4c5f8 fa655794 bl       #0x3425de0
01e4c5fc 80020036 tbz      w0, #0, #0x1e4c64c
01e4c600 e00313aa mov      x0, x19
01e4c604 e1031faa mov      x1, xzr
01e4c608 69665794 bl       #0x3425fac
01e4c60c e83d01d0 adrp     x8, #0x460a000
01e4c610 084544f9 ldr      x8, [x8, #0x888]
01e4c614 080140f9 ldr      x8, [x8]
01e4c618 e10300aa mov      x1, x0
01e4c61c e00308aa mov      x0, x8
01e4c620 e2031faa mov      x2, xzr
01e4c624 c09f4494 bl       #0x2f74524
01e4c628 280340f9 ldr      x8, [x25]
01e4c62c f30300aa mov      x19, x0
01e4c630 09e140b9 ldr      w9, [x8, #0xe0]
01e4c634 69000035 cbnz     w9, #0x1e4c640
01e4c638 e00308aa mov      x0, x8
01e4c63c 5f40bd97 bl       #0xd9c7b8
01e4c640 e00313aa mov      x0, x19
01e4c644 e1031faa mov      x1, xzr
01e4c648 9ed35694 bl       #0x34014c0
01e4c64c fd7b46a9 ldp      x29, x30, [sp, #0x60]
01e4c650 f44f45a9 ldp      x20, x19, [sp, #0x50]
01e4c654 f65744a9 ldp      x22, x21, [sp, #0x40]
01e4c658 f85f43a9 ldp      x24, x23, [sp, #0x30]
01e4c65c fa6742a9 ldp      x26, x25, [sp, #0x20]
01e4c660 fc6f41a9 ldp      x28, x27, [sp, #0x10]
01e4c664 ffc30191 add      sp, sp, #0x70
01e4c668 c0035fd6 ret      
01e4c66c 5e40bd97 bl       #0xd9c7e4
01e4c670 6440bd97 bl       #0xd9c800
01e4c674 e1031faa mov      x1, xzr
01e4c678 4b40bd97 bl       #0xd9c7a4
01e4c67c f40f1ef8 str      x20, [sp, #-0x20]!
01e4c680 f37b01a9 stp      x19, x30, [sp, #0x10]
01e4c684 b45501b0 adrp     x20, #0x4901000
01e4c688 88d27439 ldrb     w8, [x20, #0xd34]
01e4c68c f30300aa mov      x19, x0
01e4c690 c8000037 tbnz     w8, #0, #0x1e4c6a8
01e4c694 403f01d0 adrp     x0, #0x4636000
01e4c698 008c45f9 ldr      x0, [x0, #0xb18]
01e4c69c 1440bd97 bl       #0xd9c6ec
01e4c6a0 28008052 mov      w8, #1
01e4c6a4 88d23439 strb     w8, [x20, #0xd34]
01e4c6a8 e07e8152 mov      w0, #0xbf7
01e4c6ac e1031faa mov      x1, xzr
01e4c6b0 e3d0fc97 bl       #0x1d80a3c
01e4c6b4 40010036 tbz      w0, #0, #0x1e4c6dc
01e4c6b8 e07e8152 mov      w0, #0xbf7
01e4c6bc e1031faa mov      x1, xzr
01e4c6c0 0cd1fc97 bl       #0x1d80af0
01e4c6c4 800200b4 cbz      x0, #0x1e4c714
01e4c6c8 e10313aa mov      x1, x19
01e4c6cc f37b41a9 ldp      x19, x30, [sp, #0x10]
01e4c6d0 e2031faa mov      x2, xzr
01e4c6d4 f40742f8 ldr      x20, [sp], #0x20
01e4c6d8 5b22f917 b        #0x1c95044
01e4c6dc 483f01d0 adrp     x8, #0x4636000
01e4c6e0 088d45f9 ldr      x8, [x8, #0xb18]
01e4c6e4 000140f9 ldr      x0, [x8]
01e4c6e8 3940bd97 bl       #0xd9c7cc
01e4c6ec 400100b4 cbz      x0, #0x1e4c714
01e4c6f0 e1031f2a mov      w1, wzr
01e4c6f4 e2031faa mov      x2, xzr
01e4c6f8 f40300aa mov      x20, x0
01e4c6fc 7c3a0094 bl       #0x1e5b0ec
01e4c700 931200f9 str      x19, [x20, #0x20]
01e4c704 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e4c708 e00314aa mov      x0, x20
01e4c70c f40742f8 ldr      x20, [sp], #0x20
01e4c710 c0035fd6 ret      
01e4c714 3240bd97 bl       #0xd9c7dc
