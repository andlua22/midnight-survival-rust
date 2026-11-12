0206a660 f75bbda9 stp      x23, x22, [sp, #-0x30]!
0206a664 f55301a9 stp      x21, x20, [sp, #0x10]
0206a668 f37b02a9 stp      x19, x30, [sp, #0x20]
0206a66c d5440190 adrp     x21, #0x4902000
0206a670 a87e6f39 ldrb     w8, [x21, #0xbdf]
0206a674 f40301aa mov      x20, x1
0206a678 f30300aa mov      x19, x0
0206a67c 08030037 tbnz     w8, #0, #0x206a6dc
0206a680 002b01d0 adrp     x0, #0x45cc000
0206a684 00e844f9 ldr      x0, [x0, #0x9d0]
0206a688 19c8b497 bl       #0xd9c6ec
0206a68c 802e01d0 adrp     x0, #0x463c000
0206a690 006842f9 ldr      x0, [x0, #0x4d0]
0206a694 16c8b497 bl       #0xd9c6ec
0206a698 802b0190 adrp     x0, #0x45da000
0206a69c 004c45f9 ldr      x0, [x0, #0xa98]
0206a6a0 13c8b497 bl       #0xd9c6ec
0206a6a4 a02a0190 adrp     x0, #0x45be000
0206a6a8 00b047f9 ldr      x0, [x0, #0xf60]
0206a6ac 10c8b497 bl       #0xd9c6ec
0206a6b0 402e01d0 adrp     x0, #0x4634000
0206a6b4 00b046f9 ldr      x0, [x0, #0xd60]
0206a6b8 0dc8b497 bl       #0xd9c6ec
0206a6bc 802c01b0 adrp     x0, #0x45fb000
0206a6c0 002441f9 ldr      x0, [x0, #0x248]
0206a6c4 0ac8b497 bl       #0xd9c6ec
0206a6c8 c02a0190 adrp     x0, #0x45c2000
0206a6cc 000c46f9 ldr      x0, [x0, #0xc18]
0206a6d0 07c8b497 bl       #0xd9c6ec
0206a6d4 28008052 mov      w8, #1
0206a6d8 a87e2f39 strb     w8, [x21, #0xbdf]
0206a6dc 007d8152 mov      w0, #0xbe8
0206a6e0 e1031faa mov      x1, xzr
0206a6e4 d658f497 bl       #0x1d80a3c
0206a6e8 80010036 tbz      w0, #0, #0x206a718
0206a6ec 007d8152 mov      w0, #0xbe8
0206a6f0 e1031faa mov      x1, xzr
0206a6f4 ff58f497 bl       #0x1d80af0
0206a6f8 401500b4 cbz      x0, #0x206a9a0
0206a6fc e10313aa mov      x1, x19
0206a700 e20314aa mov      x2, x20
0206a704 f37b42a9 ldp      x19, x30, [sp, #0x20]
0206a708 f55341a9 ldp      x21, x20, [sp, #0x10]
0206a70c e3031faa mov      x3, xzr
0206a710 f75bc3a8 ldp      x23, x22, [sp], #0x30
0206a714 bb92f017 b        #0x1c8f200
0206a718 541400b4 cbz      x20, #0x206a9a0
0206a71c 482e01d0 adrp     x8, #0x4634000
0206a720 08b146f9 ldr      x8, [x8, #0xd60]
0206a724 952b0190 adrp     x21, #0x45da000
0206a728 e00314aa mov      x0, x20
0206a72c 010140f9 ldr      x1, [x8]
0206a730 b54e45f9 ldr      x21, [x21, #0xa98]
0206a734 98cb1494 bl       #0x259d594
0206a738 601200f9 str      x0, [x19, #0x20]
0206a73c a10240f9 ldr      x1, [x21]
0206a740 e00314aa mov      x0, x20
0206a744 94cb1494 bl       #0x259d594
0206a748 601240f9 ldr      x0, [x19, #0x20]
0206a74c a01200b4 cbz      x0, #0x206a9a0
0206a750 080040f9 ldr      x8, [x0]
0206a754 952c01b0 adrp     x21, #0x45fb000
0206a758 09855aa9 ldp      x9, x1, [x8, #0x1a8]
0206a75c b52641f9 ldr      x21, [x21, #0x248]
0206a760 20013fd6 blr      x9
0206a764 a00240f9 ldr      x0, [x21]
0206a768 19c8b497 bl       #0xd9c7cc
0206a76c a01100b4 cbz      x0, #0x206a9a0
0206a770 c82a0190 adrp     x8, #0x45c2000
0206a774 080d46f9 ldr      x8, [x8, #0xc18]
0206a778 e2031faa mov      x2, xzr
0206a77c f50300aa mov      x21, x0
0206a780 010140f9 ldr      x1, [x8]
0206a784 03ec4e94 bl       #0x3425790
0206a788 e00315aa mov      x0, x21
0206a78c e1031faa mov      x1, xzr
0206a790 01eb4e94 bl       #0x3425394
0206a794 f60300aa mov      x22, x0
0206a798 e00314aa mov      x0, x20
0206a79c e1031faa mov      x1, xzr
0206a7a0 fdea4e94 bl       #0x3425394
0206a7a4 f60f00b4 cbz      x22, #0x206a9a0
0206a7a8 942e01d0 adrp     x20, #0x463c000
0206a7ac b72a0190 adrp     x23, #0x45be000
0206a7b0 946a42f9 ldr      x20, [x20, #0x4d0]
0206a7b4 f7b247f9 ldr      x23, [x23, #0xf60]
0206a7b8 e10300aa mov      x1, x0
0206a7bc e00316aa mov      x0, x22
0206a7c0 e2031faa mov      x2, xzr
0206a7c4 0c3b4f94 bl       #0x34393f4
0206a7c8 e00315aa mov      x0, x21
0206a7cc e1031faa mov      x1, xzr
0206a7d0 f1ea4e94 bl       #0x3425394
0206a7d4 880240f9 ldr      x8, [x20]
0206a7d8 f40300aa mov      x20, x0
0206a7dc 09e140b9 ldr      w9, [x8, #0xe0]
0206a7e0 69000035 cbnz     w9, #0x206a7ec
0206a7e4 e00308aa mov      x0, x8
0206a7e8 f4c7b497 bl       #0xd9c7b8
0206a7ec a8ad00f0 adrp     x8, #0x3621000
0206a7f0 007149bd ldr      s0, [x8, #0x970]
0206a7f4 e00314aa mov      x0, x20
0206a7f8 e1031faa mov      x1, xzr
0206a7fc 69490094 bl       #0x207cda0
0206a800 e10240f9 ldr      x1, [x23]
0206a804 741240f9 ldr      x20, [x19, #0x20]
0206a808 e00315aa mov      x0, x21
0206a80c 62cb1494 bl       #0x259d594
0206a810 940c00b4 cbz      x20, #0x206a9a0
0206a814 802a02f9 str      x0, [x20, #0x450]
0206a818 681240f9 ldr      x8, [x19, #0x20]
0206a81c 280c00b4 cbz      x8, #0x206a9a0
0206a820 002942f9 ldr      x0, [x8, #0x450]
0206a824 e00b00b4 cbz      x0, #0x206a9a0
0206a828 e1031f2a mov      w1, wzr
0206a82c e2031faa mov      x2, xzr
0206a830 22734e94 bl       #0x34074b8
0206a834 681240f9 ldr      x8, [x19, #0x20]
0206a838 480b00b4 cbz      x8, #0x206a9a0
0206a83c 002942f9 ldr      x0, [x8, #0x450]
0206a840 000b00b4 cbz      x0, #0x206a9a0
0206a844 c8ad00b0 adrp     x8, #0x3623000
0206a848 002944bd ldr      s0, [x8, #0x428]
0206a84c e1031faa mov      x1, xzr
0206a850 3a734e94 bl       #0x3407538
0206a854 681240f9 ldr      x8, [x19, #0x20]
0206a858 480a00b4 cbz      x8, #0x206a9a0
0206a85c 002942f9 ldr      x0, [x8, #0x450]
0206a860 000a00b4 cbz      x0, #0x206a9a0
0206a864 0010201e fmov     s0, #2.00000000
0206a868 e1031faa mov      x1, xzr
0206a86c cf734e94 bl       #0x34077a8
0206a870 681240f9 ldr      x8, [x19, #0x20]
0206a874 680900b4 cbz      x8, #0x206a9a0
0206a878 002942f9 ldr      x0, [x8, #0x450]
0206a87c 200900b4 cbz      x0, #0x206a9a0
0206a880 c8ad00b0 adrp     x8, #0x3623000
0206a884 002d44bd ldr      s0, [x8, #0x42c]
0206a888 e1031faa mov      x1, xzr
0206a88c 38744e94 bl       #0x340796c
0206a890 681240f9 ldr      x8, [x19, #0x20]
0206a894 680800b4 cbz      x8, #0x206a9a0
0206a898 002942f9 ldr      x0, [x8, #0x450]
0206a89c 200800b4 cbz      x0, #0x206a9a0
0206a8a0 c8ad00b0 adrp     x8, #0x3623000
0206a8a4 c9ad00b0 adrp     x9, #0x3623000
0206a8a8 013144bd ldr      s1, [x8, #0x430]
0206a8ac 223544bd ldr      s2, [x9, #0x434]
0206a8b0 00102e1e fmov     s0, #1.00000000
0206a8b4 031ca04e mov      v3.16b, v0.16b
0206a8b8 e1031faa mov      x1, xzr
0206a8bc 68734e94 bl       #0x340765c
0206a8c0 681240f9 ldr      x8, [x19, #0x20]
0206a8c4 e80600b4 cbz      x8, #0x206a9a0
0206a8c8 002942f9 ldr      x0, [x8, #0x450]
0206a8cc a00600b4 cbz      x0, #0x206a9a0
0206a8d0 41008052 mov      w1, #2
0206a8d4 e2031faa mov      x2, xzr
0206a8d8 d8724e94 bl       #0x3407438
0206a8dc 681240f9 ldr      x8, [x19, #0x20]
0206a8e0 080600b4 cbz      x8, #0x206a9a0
0206a8e4 002942f9 ldr      x0, [x8, #0x450]
0206a8e8 c00500b4 cbz      x0, #0x206a9a0
0206a8ec e1031f2a mov      w1, wzr
0206a8f0 e2031faa mov      x2, xzr
0206a8f4 71724e94 bl       #0x34072b8
0206a8f8 681240f9 ldr      x8, [x19, #0x20]
0206a8fc 280500b4 cbz      x8, #0x206a9a0
0206a900 002942f9 ldr      x0, [x8, #0x450]
0206a904 e00400b4 cbz      x0, #0x206a9a0
0206a908 e1031f2a mov      w1, wzr
0206a90c e2031faa mov      x2, xzr
0206a910 43e74e94 bl       #0x342461c
0206a914 681240f9 ldr      x8, [x19, #0x20]
0206a918 480400b4 cbz      x8, #0x206a9a0
0206a91c 002942f9 ldr      x0, [x8, #0x450]
0206a920 000400b4 cbz      x0, #0x206a9a0
0206a924 e1bf9f52 mov      w1, #0xfdff
0206a928 61ffbf72 movk     w1, #0xfffb, lsl #16
0206a92c e2031faa mov      x2, xzr
0206a930 5c744e94 bl       #0x3407aa0
0206a934 601240f9 ldr      x0, [x19, #0x20]
0206a938 400300b4 cbz      x0, #0x206a9a0
0206a93c 615a41f9 ldr      x1, [x19, #0x2b0]
0206a940 142b01d0 adrp     x20, #0x45cc000
0206a944 94ea44f9 ldr      x20, [x20, #0x9d0]
0206a948 e2031faa mov      x2, xzr
0206a94c 9a03f997 bl       #0x1eab7b4
0206a950 e00313aa mov      x0, x19
0206a954 14000094 bl       #0x206a9a4
0206a958 68e640f9 ldr      x8, [x19, #0x1c8]
0206a95c c0000036 tbz      w0, #0, #0x206a974
0206a960 080200b4 cbz      x8, #0x206a9a0
0206a964 621240f9 ldr      x2, [x19, #0x20]
0206a968 830240f9 ldr      x3, [x20]
0206a96c 01008092 mov      x1, #-1
0206a970 07000014 b        #0x206a98c
0206a974 695a41f9 ldr      x9, [x19, #0x2b0]
0206a978 490100b4 cbz      x9, #0x206a9a0
0206a97c 280100b4 cbz      x8, #0x206a9a0
0206a980 621240f9 ldr      x2, [x19, #0x20]
0206a984 211140f9 ldr      x1, [x9, #0x20]
0206a988 830240f9 ldr      x3, [x20]
0206a98c f37b42a9 ldp      x19, x30, [sp, #0x20]
0206a990 f55341a9 ldp      x21, x20, [sp, #0x10]
0206a994 e00308aa mov      x0, x8
0206a998 f75bc3a8 ldp      x23, x22, [sp], #0x30
0206a99c 9c541d14 b        #0x27bfc0c
0206a9a0 8fc7b497 bl       #0xd9c7dc
