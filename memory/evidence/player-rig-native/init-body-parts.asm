01e4d418 f553bea9 stp      x21, x20, [sp, #-0x20]!
01e4d41c f37b01a9 stp      x19, x30, [sp, #0x10]
01e4d420 b4550190 adrp     x20, #0x4901000
01e4d424 88ca7339 ldrb     w8, [x20, #0xcf2]
01e4d428 f30300aa mov      x19, x0
01e4d42c 48080037 tbnz     w8, #0, #0x1e4d534
01e4d430 203c01d0 adrp     x0, #0x45d3000
01e4d434 002045f9 ldr      x0, [x0, #0xa40]
01e4d438 ad3cbd97 bl       #0xd9c6ec
01e4d43c 203d0190 adrp     x0, #0x45f1000
01e4d440 008043f9 ldr      x0, [x0, #0x700]
01e4d444 aa3cbd97 bl       #0xd9c6ec
01e4d448 c03c01f0 adrp     x0, #0x45e8000
01e4d44c 000c43f9 ldr      x0, [x0, #0x618]
01e4d450 a73cbd97 bl       #0xd9c6ec
01e4d454 c03e01b0 adrp     x0, #0x4626000
01e4d458 00f842f9 ldr      x0, [x0, #0x5f0]
01e4d45c a43cbd97 bl       #0xd9c6ec
01e4d460 c03e01d0 adrp     x0, #0x4627000
01e4d464 00e443f9 ldr      x0, [x0, #0x7c8]
01e4d468 a13cbd97 bl       #0xd9c6ec
01e4d46c 003b01b0 adrp     x0, #0x45ae000
01e4d470 00fc47f9 ldr      x0, [x0, #0xff8]
01e4d474 9e3cbd97 bl       #0xd9c6ec
01e4d478 803d0190 adrp     x0, #0x45fd000
01e4d47c 00e842f9 ldr      x0, [x0, #0x5d0]
01e4d480 9b3cbd97 bl       #0xd9c6ec
01e4d484 803c01f0 adrp     x0, #0x45e0000
01e4d488 005843f9 ldr      x0, [x0, #0x6b0]
01e4d48c 983cbd97 bl       #0xd9c6ec
01e4d490 a03c01b0 adrp     x0, #0x45e2000
01e4d494 00cc40f9 ldr      x0, [x0, #0x198]
01e4d498 953cbd97 bl       #0xd9c6ec
01e4d49c e03c0190 adrp     x0, #0x45e9000
01e4d4a0 006443f9 ldr      x0, [x0, #0x6c8]
01e4d4a4 923cbd97 bl       #0xd9c6ec
01e4d4a8 403c01d0 adrp     x0, #0x45d7000
01e4d4ac 002046f9 ldr      x0, [x0, #0xc40]
01e4d4b0 8f3cbd97 bl       #0xd9c6ec
01e4d4b4 e03b01f0 adrp     x0, #0x45cc000
01e4d4b8 003c47f9 ldr      x0, [x0, #0xe78]
01e4d4bc 8c3cbd97 bl       #0xd9c6ec
01e4d4c0 203f01f0 adrp     x0, #0x4634000
01e4d4c4 000842f9 ldr      x0, [x0, #0x410]
01e4d4c8 893cbd97 bl       #0xd9c6ec
01e4d4cc 603c01b0 adrp     x0, #0x45da000
01e4d4d0 00ac42f9 ldr      x0, [x0, #0x558]
01e4d4d4 863cbd97 bl       #0xd9c6ec
01e4d4d8 603b0190 adrp     x0, #0x45b9000
01e4d4dc 00d844f9 ldr      x0, [x0, #0x9b0]
01e4d4e0 833cbd97 bl       #0xd9c6ec
01e4d4e4 003d01d0 adrp     x0, #0x45ef000
01e4d4e8 009442f9 ldr      x0, [x0, #0x528]
01e4d4ec 803cbd97 bl       #0xd9c6ec
01e4d4f0 603b0190 adrp     x0, #0x45b9000
01e4d4f4 001842f9 ldr      x0, [x0, #0x430]
01e4d4f8 7d3cbd97 bl       #0xd9c6ec
01e4d4fc 203e01b0 adrp     x0, #0x4612000
01e4d500 007043f9 ldr      x0, [x0, #0x6e0]
01e4d504 7a3cbd97 bl       #0xd9c6ec
01e4d508 403b0190 adrp     x0, #0x45b5000
01e4d50c 008440f9 ldr      x0, [x0, #0x108]
01e4d510 773cbd97 bl       #0xd9c6ec
01e4d514 203e0190 adrp     x0, #0x4611000
01e4d518 009c44f9 ldr      x0, [x0, #0x938]
01e4d51c 743cbd97 bl       #0xd9c6ec
01e4d520 e03c01b0 adrp     x0, #0x45ea000
01e4d524 008447f9 ldr      x0, [x0, #0xf08]
01e4d528 713cbd97 bl       #0xd9c6ec
01e4d52c 28008052 mov      w8, #1
01e4d530 88ca3339 strb     w8, [x20, #0xcf2]
01e4d534 607d8152 mov      w0, #0xbeb
01e4d538 e1031faa mov      x1, xzr
01e4d53c 40cdfc97 bl       #0x1d80a3c
01e4d540 40010036 tbz      w0, #0, #0x1e4d568
01e4d544 607d8152 mov      w0, #0xbeb
01e4d548 e1031faa mov      x1, xzr
01e4d54c 69cdfc97 bl       #0x1d80af0
01e4d550 c02c00b4 cbz      x0, #0x1e4dae8
01e4d554 e10313aa mov      x1, x19
01e4d558 f37b41a9 ldp      x19, x30, [sp, #0x10]
01e4d55c e2031faa mov      x2, xzr
01e4d560 f553c2a8 ldp      x21, x20, [sp], #0x20
01e4d564 e40af917 b        #0x1c900f4
01e4d568 283c01d0 adrp     x8, #0x45d3000
01e4d56c 082145f9 ldr      x8, [x8, #0xa40]
01e4d570 753c01b0 adrp     x21, #0x45da000
01e4d574 41028052 mov      w1, #0x12
01e4d578 000140f9 ldr      x0, [x8]
01e4d57c b5ae42f9 ldr      x21, [x21, #0x558]
01e4d580 693cbd97 bl       #0xd9c724
01e4d584 603200f9 str      x0, [x19, #0x60]
01e4d588 a10240f9 ldr      x1, [x21]
01e4d58c f40300aa mov      x20, x0
01e4d590 e00313aa mov      x0, x19
01e4d594 87fcff97 bl       #0x1e4c7b0
01e4d598 942a00b4 cbz      x20, #0x1e4dae8
01e4d59c f50300aa mov      x21, x0
01e4d5a0 c00000b4 cbz      x0, #0x1e4d5b8
01e4d5a4 880240f9 ldr      x8, [x20]
01e4d5a8 e00315aa mov      x0, x21
01e4d5ac 012140f9 ldr      x1, [x8, #0x40]
01e4d5b0 833cbd97 bl       #0xd9c7bc
01e4d5b4 e02900b4 cbz      x0, #0x1e4daf0
01e4d5b8 881a40b9 ldr      w8, [x20, #0x18]
01e4d5bc 1f250071 cmp      w8, #9
01e4d5c0 69290054 b.ls     #0x1e4daec
01e4d5c4 e83c0190 adrp     x8, #0x45e9000
01e4d5c8 086543f9 ldr      x8, [x8, #0x6c8]
01e4d5cc 953600f9 str      x21, [x20, #0x68]
01e4d5d0 753240f9 ldr      x21, [x19, #0x60]
01e4d5d4 e00313aa mov      x0, x19
01e4d5d8 010140f9 ldr      x1, [x8]
01e4d5dc 75fcff97 bl       #0x1e4c7b0
01e4d5e0 552800b4 cbz      x21, #0x1e4dae8
01e4d5e4 f40300aa mov      x20, x0
01e4d5e8 c00000b4 cbz      x0, #0x1e4d600
01e4d5ec a80240f9 ldr      x8, [x21]
01e4d5f0 e00314aa mov      x0, x20
01e4d5f4 012140f9 ldr      x1, [x8, #0x40]
01e4d5f8 713cbd97 bl       #0xd9c7bc
01e4d5fc a02700b4 cbz      x0, #0x1e4daf0
01e4d600 a81a40b9 ldr      w8, [x21, #0x18]
01e4d604 1f210071 cmp      w8, #8
01e4d608 29270054 b.ls     #0x1e4daec
01e4d60c e83c01b0 adrp     x8, #0x45ea000
01e4d610 088547f9 ldr      x8, [x8, #0xf08]
01e4d614 b43200f9 str      x20, [x21, #0x60]
01e4d618 753240f9 ldr      x21, [x19, #0x60]
01e4d61c e00313aa mov      x0, x19
01e4d620 010140f9 ldr      x1, [x8]
01e4d624 63fcff97 bl       #0x1e4c7b0
01e4d628 152600b4 cbz      x21, #0x1e4dae8
01e4d62c f40300aa mov      x20, x0
01e4d630 c00000b4 cbz      x0, #0x1e4d648
01e4d634 a80240f9 ldr      x8, [x21]
01e4d638 e00314aa mov      x0, x20
01e4d63c 012140f9 ldr      x1, [x8, #0x40]
01e4d640 5f3cbd97 bl       #0xd9c7bc
01e4d644 602500b4 cbz      x0, #0x1e4daf0
01e4d648 a81a40b9 ldr      w8, [x21, #0x18]
01e4d64c 08250034 cbz      w8, #0x1e4daec
01e4d650 a83c01b0 adrp     x8, #0x45e2000
01e4d654 08cd40f9 ldr      x8, [x8, #0x198]
01e4d658 b41200f9 str      x20, [x21, #0x20]
01e4d65c 753240f9 ldr      x21, [x19, #0x60]
01e4d660 e00313aa mov      x0, x19
01e4d664 010140f9 ldr      x1, [x8]
01e4d668 52fcff97 bl       #0x1e4c7b0
01e4d66c f52300b4 cbz      x21, #0x1e4dae8
01e4d670 f40300aa mov      x20, x0
01e4d674 c00000b4 cbz      x0, #0x1e4d68c
01e4d678 a80240f9 ldr      x8, [x21]
01e4d67c e00314aa mov      x0, x20
01e4d680 012140f9 ldr      x1, [x8, #0x40]
01e4d684 4e3cbd97 bl       #0xd9c7bc
01e4d688 402300b4 cbz      x0, #0x1e4daf0
01e4d68c a81a40b9 ldr      w8, [x21, #0x18]
01e4d690 1f050071 cmp      w8, #1
01e4d694 c9220054 b.ls     #0x1e4daec
01e4d698 c83e01b0 adrp     x8, #0x4626000
01e4d69c 08f942f9 ldr      x8, [x8, #0x5f0]
01e4d6a0 b41600f9 str      x20, [x21, #0x28]
01e4d6a4 753240f9 ldr      x21, [x19, #0x60]
01e4d6a8 e00313aa mov      x0, x19
01e4d6ac 010140f9 ldr      x1, [x8]
01e4d6b0 40fcff97 bl       #0x1e4c7b0
01e4d6b4 b52100b4 cbz      x21, #0x1e4dae8
01e4d6b8 f40300aa mov      x20, x0
01e4d6bc c00000b4 cbz      x0, #0x1e4d6d4
01e4d6c0 a80240f9 ldr      x8, [x21]
01e4d6c4 e00314aa mov      x0, x20
01e4d6c8 012140f9 ldr      x1, [x8, #0x40]
01e4d6cc 3c3cbd97 bl       #0xd9c7bc
01e4d6d0 002100b4 cbz      x0, #0x1e4daf0
01e4d6d4 a81a40b9 ldr      w8, [x21, #0x18]
01e4d6d8 1f090071 cmp      w8, #2
01e4d6dc 89200054 b.ls     #0x1e4daec
01e4d6e0 683b0190 adrp     x8, #0x45b9000
01e4d6e4 08d944f9 ldr      x8, [x8, #0x9b0]
01e4d6e8 b41a00f9 str      x20, [x21, #0x30]
01e4d6ec 753240f9 ldr      x21, [x19, #0x60]
01e4d6f0 e00313aa mov      x0, x19
01e4d6f4 010140f9 ldr      x1, [x8]
01e4d6f8 2efcff97 bl       #0x1e4c7b0
01e4d6fc 751f00b4 cbz      x21, #0x1e4dae8
01e4d700 f40300aa mov      x20, x0
01e4d704 c00000b4 cbz      x0, #0x1e4d71c
01e4d708 a80240f9 ldr      x8, [x21]
01e4d70c e00314aa mov      x0, x20
01e4d710 012140f9 ldr      x1, [x8, #0x40]
01e4d714 2a3cbd97 bl       #0xd9c7bc
01e4d718 c01e00b4 cbz      x0, #0x1e4daf0
01e4d71c a81a40b9 ldr      w8, [x21, #0x18]
01e4d720 1f0d0071 cmp      w8, #3
01e4d724 491e0054 b.ls     #0x1e4daec
01e4d728 883d0190 adrp     x8, #0x45fd000
01e4d72c 08e942f9 ldr      x8, [x8, #0x5d0]
01e4d730 b41e00f9 str      x20, [x21, #0x38]
01e4d734 753240f9 ldr      x21, [x19, #0x60]
01e4d738 e00313aa mov      x0, x19
01e4d73c 010140f9 ldr      x1, [x8]
01e4d740 1cfcff97 bl       #0x1e4c7b0
01e4d744 351d00b4 cbz      x21, #0x1e4dae8
01e4d748 f40300aa mov      x20, x0
01e4d74c c00000b4 cbz      x0, #0x1e4d764
01e4d750 a80240f9 ldr      x8, [x21]
01e4d754 e00314aa mov      x0, x20
01e4d758 012140f9 ldr      x1, [x8, #0x40]
01e4d75c 183cbd97 bl       #0xd9c7bc
01e4d760 801c00b4 cbz      x0, #0x1e4daf0
01e4d764 a81a40b9 ldr      w8, [x21, #0x18]
01e4d768 1f150071 cmp      w8, #5
01e4d76c 091c0054 b.ls     #0x1e4daec
01e4d770 083b01b0 adrp     x8, #0x45ae000
01e4d774 08fd47f9 ldr      x8, [x8, #0xff8]
01e4d778 b42600f9 str      x20, [x21, #0x48]
01e4d77c 753240f9 ldr      x21, [x19, #0x60]
01e4d780 e00313aa mov      x0, x19
01e4d784 010140f9 ldr      x1, [x8]
01e4d788 0afcff97 bl       #0x1e4c7b0
01e4d78c f51a00b4 cbz      x21, #0x1e4dae8
01e4d790 f40300aa mov      x20, x0
01e4d794 c00000b4 cbz      x0, #0x1e4d7ac
01e4d798 a80240f9 ldr      x8, [x21]
01e4d79c e00314aa mov      x0, x20
01e4d7a0 012140f9 ldr      x1, [x8, #0x40]
01e4d7a4 063cbd97 bl       #0xd9c7bc
01e4d7a8 401a00b4 cbz      x0, #0x1e4daf0
01e4d7ac a81a40b9 ldr      w8, [x21, #0x18]
01e4d7b0 1f1d0071 cmp      w8, #7
01e4d7b4 c9190054 b.ls     #0x1e4daec
01e4d7b8 283e0190 adrp     x8, #0x4611000
01e4d7bc 089d44f9 ldr      x8, [x8, #0x938]
01e4d7c0 b42e00f9 str      x20, [x21, #0x58]
01e4d7c4 753240f9 ldr      x21, [x19, #0x60]
01e4d7c8 e00313aa mov      x0, x19
01e4d7cc 010140f9 ldr      x1, [x8]
01e4d7d0 f8fbff97 bl       #0x1e4c7b0
01e4d7d4 b51800b4 cbz      x21, #0x1e4dae8
01e4d7d8 f40300aa mov      x20, x0
01e4d7dc c00000b4 cbz      x0, #0x1e4d7f4
01e4d7e0 a80240f9 ldr      x8, [x21]
01e4d7e4 e00314aa mov      x0, x20
01e4d7e8 012140f9 ldr      x1, [x8, #0x40]
01e4d7ec f43bbd97 bl       #0xd9c7bc
01e4d7f0 001800b4 cbz      x0, #0x1e4daf0
01e4d7f4 a81a40b9 ldr      w8, [x21, #0x18]
01e4d7f8 1f190071 cmp      w8, #6
01e4d7fc 89170054 b.ls     #0x1e4daec
01e4d800 083d01d0 adrp     x8, #0x45ef000
01e4d804 089542f9 ldr      x8, [x8, #0x528]
01e4d808 b42a00f9 str      x20, [x21, #0x50]
01e4d80c 753240f9 ldr      x21, [x19, #0x60]
01e4d810 e00313aa mov      x0, x19
01e4d814 010140f9 ldr      x1, [x8]
01e4d818 e6fbff97 bl       #0x1e4c7b0
01e4d81c 751600b4 cbz      x21, #0x1e4dae8
01e4d820 f40300aa mov      x20, x0
01e4d824 c00000b4 cbz      x0, #0x1e4d83c
01e4d828 a80240f9 ldr      x8, [x21]
01e4d82c e00314aa mov      x0, x20
01e4d830 012140f9 ldr      x1, [x8, #0x40]
01e4d834 e23bbd97 bl       #0xd9c7bc
01e4d838 c01500b4 cbz      x0, #0x1e4daf0
01e4d83c a81a40b9 ldr      w8, [x21, #0x18]
01e4d840 1f110071 cmp      w8, #4
01e4d844 49150054 b.ls     #0x1e4daec
01e4d848 283e01b0 adrp     x8, #0x4612000
01e4d84c 087143f9 ldr      x8, [x8, #0x6e0]
01e4d850 b42200f9 str      x20, [x21, #0x40]
01e4d854 e00313aa mov      x0, x19
01e4d858 010140f9 ldr      x1, [x8]
01e4d85c d5fbff97 bl       #0x1e4c7b0
01e4d860 753240f9 ldr      x21, [x19, #0x60]
01e4d864 605a00f9 str      x0, [x19, #0xb0]
01e4d868 151400b4 cbz      x21, #0x1e4dae8
01e4d86c f40300aa mov      x20, x0
01e4d870 c00000b4 cbz      x0, #0x1e4d888
01e4d874 a80240f9 ldr      x8, [x21]
01e4d878 e00314aa mov      x0, x20
01e4d87c 012140f9 ldr      x1, [x8, #0x40]
01e4d880 cf3bbd97 bl       #0xd9c7bc
01e4d884 601300b4 cbz      x0, #0x1e4daf0
01e4d888 a81a40b9 ldr      w8, [x21, #0x18]
01e4d88c 1f410071 cmp      w8, #0x10
01e4d890 e9120054 b.ls     #0x1e4daec
01e4d894 483c01d0 adrp     x8, #0x45d7000
01e4d898 082146f9 ldr      x8, [x8, #0xc40]
01e4d89c b45200f9 str      x20, [x21, #0xa0]
01e4d8a0 d43c01f0 adrp     x20, #0x45e8000
01e4d8a4 e00313aa mov      x0, x19
01e4d8a8 010140f9 ldr      x1, [x8]
01e4d8ac 940e43f9 ldr      x20, [x20, #0x618]
01e4d8b0 c0fbff97 bl       #0x1e4c7b0
01e4d8b4 605e00f9 str      x0, [x19, #0xb8]
01e4d8b8 810240f9 ldr      x1, [x20]
01e4d8bc e00313aa mov      x0, x19
01e4d8c0 bcfbff97 bl       #0x1e4c7b0
01e4d8c4 753240f9 ldr      x21, [x19, #0x60]
01e4d8c8 603a00f9 str      x0, [x19, #0x70]
01e4d8cc f51000b4 cbz      x21, #0x1e4dae8
01e4d8d0 f40300aa mov      x20, x0
01e4d8d4 c00000b4 cbz      x0, #0x1e4d8ec
01e4d8d8 a80240f9 ldr      x8, [x21]
01e4d8dc e00314aa mov      x0, x20
01e4d8e0 012140f9 ldr      x1, [x8, #0x40]
01e4d8e4 b63bbd97 bl       #0xd9c7bc
01e4d8e8 401000b4 cbz      x0, #0x1e4daf0
01e4d8ec a81a40b9 ldr      w8, [x21, #0x18]
01e4d8f0 1f310071 cmp      w8, #0xc
01e4d8f4 c90f0054 b.ls     #0x1e4daec
01e4d8f8 c83e01d0 adrp     x8, #0x4627000
01e4d8fc 08e543f9 ldr      x8, [x8, #0x7c8]
01e4d900 b44200f9 str      x20, [x21, #0x80]
01e4d904 e00313aa mov      x0, x19
01e4d908 010140f9 ldr      x1, [x8]
01e4d90c a9fbff97 bl       #0x1e4c7b0
01e4d910 753240f9 ldr      x21, [x19, #0x60]
01e4d914 603e00f9 str      x0, [x19, #0x78]
01e4d918 950e00b4 cbz      x21, #0x1e4dae8
01e4d91c f40300aa mov      x20, x0
01e4d920 c00000b4 cbz      x0, #0x1e4d938
01e4d924 a80240f9 ldr      x8, [x21]
01e4d928 e00314aa mov      x0, x20
01e4d92c 012140f9 ldr      x1, [x8, #0x40]
01e4d930 a33bbd97 bl       #0xd9c7bc
01e4d934 e00d00b4 cbz      x0, #0x1e4daf0
01e4d938 a81a40b9 ldr      w8, [x21, #0x18]
01e4d93c 1f350071 cmp      w8, #0xd
01e4d940 690d0054 b.ls     #0x1e4daec
01e4d944 e83b01f0 adrp     x8, #0x45cc000
01e4d948 083d47f9 ldr      x8, [x8, #0xe78]
01e4d94c b44600f9 str      x20, [x21, #0x88]
01e4d950 e00313aa mov      x0, x19
01e4d954 010140f9 ldr      x1, [x8]
01e4d958 96fbff97 bl       #0x1e4c7b0
01e4d95c 753240f9 ldr      x21, [x19, #0x60]
01e4d960 604200f9 str      x0, [x19, #0x80]
01e4d964 350c00b4 cbz      x21, #0x1e4dae8
01e4d968 f40300aa mov      x20, x0
01e4d96c c00000b4 cbz      x0, #0x1e4d984
01e4d970 a80240f9 ldr      x8, [x21]
01e4d974 e00314aa mov      x0, x20
01e4d978 012140f9 ldr      x1, [x8, #0x40]
01e4d97c 903bbd97 bl       #0xd9c7bc
01e4d980 800b00b4 cbz      x0, #0x1e4daf0
01e4d984 a81a40b9 ldr      w8, [x21, #0x18]
01e4d988 1f290071 cmp      w8, #0xa
01e4d98c 090b0054 b.ls     #0x1e4daec
01e4d990 683b0190 adrp     x8, #0x45b9000
01e4d994 081942f9 ldr      x8, [x8, #0x430]
01e4d998 b43a00f9 str      x20, [x21, #0x70]
01e4d99c e00313aa mov      x0, x19
01e4d9a0 010140f9 ldr      x1, [x8]
01e4d9a4 83fbff97 bl       #0x1e4c7b0
01e4d9a8 753240f9 ldr      x21, [x19, #0x60]
01e4d9ac 604600f9 str      x0, [x19, #0x88]
01e4d9b0 d50900b4 cbz      x21, #0x1e4dae8
01e4d9b4 744240f9 ldr      x20, [x19, #0x80]
01e4d9b8 d40000b4 cbz      x20, #0x1e4d9d0
01e4d9bc a80240f9 ldr      x8, [x21]
01e4d9c0 e00314aa mov      x0, x20
01e4d9c4 012140f9 ldr      x1, [x8, #0x40]
01e4d9c8 7d3bbd97 bl       #0xd9c7bc
01e4d9cc 200900b4 cbz      x0, #0x1e4daf0
01e4d9d0 a81a40b9 ldr      w8, [x21, #0x18]
01e4d9d4 1f450071 cmp      w8, #0x11
01e4d9d8 a9080054 b.ls     #0x1e4daec
01e4d9dc 483b0190 adrp     x8, #0x45b5000
01e4d9e0 088540f9 ldr      x8, [x8, #0x108]
01e4d9e4 b45600f9 str      x20, [x21, #0xa8]
01e4d9e8 e00313aa mov      x0, x19
01e4d9ec 010140f9 ldr      x1, [x8]
01e4d9f0 70fbff97 bl       #0x1e4c7b0
01e4d9f4 753240f9 ldr      x21, [x19, #0x60]
01e4d9f8 604e00f9 str      x0, [x19, #0x98]
01e4d9fc 750700b4 cbz      x21, #0x1e4dae8
01e4da00 f40300aa mov      x20, x0
01e4da04 c00000b4 cbz      x0, #0x1e4da1c
01e4da08 a80240f9 ldr      x8, [x21]
01e4da0c e00314aa mov      x0, x20
01e4da10 012140f9 ldr      x1, [x8, #0x40]
01e4da14 6a3bbd97 bl       #0xd9c7bc
01e4da18 c00600b4 cbz      x0, #0x1e4daf0
01e4da1c a81a40b9 ldr      w8, [x21, #0x18]
01e4da20 1f2d0071 cmp      w8, #0xb
01e4da24 49060054 b.ls     #0x1e4daec
01e4da28 883c01f0 adrp     x8, #0x45e0000
01e4da2c 085943f9 ldr      x8, [x8, #0x6b0]
01e4da30 b43e00f9 str      x20, [x21, #0x78]
01e4da34 343d0190 adrp     x20, #0x45f1000
01e4da38 353f01f0 adrp     x21, #0x4634000
01e4da3c 010140f9 ldr      x1, [x8]
01e4da40 948243f9 ldr      x20, [x20, #0x700]
01e4da44 b50a42f9 ldr      x21, [x21, #0x410]
01e4da48 e00313aa mov      x0, x19
01e4da4c 59fbff97 bl       #0x1e4c7b0
01e4da50 605200f9 str      x0, [x19, #0xa0]
01e4da54 810240f9 ldr      x1, [x20]
01e4da58 e00313aa mov      x0, x19
01e4da5c 55fbff97 bl       #0x1e4c7b0
01e4da60 605600f9 str      x0, [x19, #0xa8]
01e4da64 a10240f9 ldr      x1, [x21]
01e4da68 e00313aa mov      x0, x19
01e4da6c 51fbff97 bl       #0x1e4c7b0
01e4da70 753240f9 ldr      x21, [x19, #0x60]
01e4da74 604a00f9 str      x0, [x19, #0x90]
01e4da78 950300b4 cbz      x21, #0x1e4dae8
01e4da7c 745240f9 ldr      x20, [x19, #0xa0]
01e4da80 d40000b4 cbz      x20, #0x1e4da98
01e4da84 a80240f9 ldr      x8, [x21]
01e4da88 e00314aa mov      x0, x20
01e4da8c 012140f9 ldr      x1, [x8, #0x40]
01e4da90 4b3bbd97 bl       #0xd9c7bc
01e4da94 e00200b4 cbz      x0, #0x1e4daf0
01e4da98 a81a40b9 ldr      w8, [x21, #0x18]
01e4da9c 1f390071 cmp      w8, #0xe
01e4daa0 69020054 b.ls     #0x1e4daec
01e4daa4 b44a00f9 str      x20, [x21, #0x90]
01e4daa8 743240f9 ldr      x20, [x19, #0x60]
01e4daac f40100b4 cbz      x20, #0x1e4dae8
01e4dab0 735640f9 ldr      x19, [x19, #0xa8]
01e4dab4 d30000b4 cbz      x19, #0x1e4dacc
01e4dab8 880240f9 ldr      x8, [x20]
01e4dabc e00313aa mov      x0, x19
01e4dac0 012140f9 ldr      x1, [x8, #0x40]
01e4dac4 3e3bbd97 bl       #0xd9c7bc
01e4dac8 400100b4 cbz      x0, #0x1e4daf0
01e4dacc 881a40b9 ldr      w8, [x20, #0x18]
01e4dad0 1f3d0071 cmp      w8, #0xf
01e4dad4 c9000054 b.ls     #0x1e4daec
01e4dad8 934e00f9 str      x19, [x20, #0x98]
01e4dadc f37b41a9 ldp      x19, x30, [sp, #0x10]
01e4dae0 f553c2a8 ldp      x21, x20, [sp], #0x20
01e4dae4 c0035fd6 ret      
01e4dae8 3d3bbd97 bl       #0xd9c7dc
01e4daec 3e3bbd97 bl       #0xd9c7e4
01e4daf0 443bbd97 bl       #0xd9c800
01e4daf4 e1031faa mov      x1, xzr
01e4daf8 2b3bbd97 bl       #0xd9c7a4
