01eb568c e923bd6d stp      d9, d8, [sp, #-0x30]!
01eb5690 f55301a9 stp      x21, x20, [sp, #0x10]
01eb5694 f37b02a9 stp      x19, x30, [sp, #0x20]
01eb5698 74520190 adrp     x20, #0x4901000
01eb569c 88c67e39 ldrb     w8, [x20, #0xfb1]
01eb56a0 f30300aa mov      x19, x0
01eb56a4 e8010037 tbnz     w8, #0, #0x1eb56e0
01eb56a8 e03801b0 adrp     x0, #0x45d2000
01eb56ac 00fc40f9 ldr      x0, [x0, #0x1f8]
01eb56b0 0f9cbb97 bl       #0xd9c6ec
01eb56b4 403901f0 adrp     x0, #0x45e0000
01eb56b8 005042f9 ldr      x0, [x0, #0x4a0]
01eb56bc 0c9cbb97 bl       #0xd9c6ec
01eb56c0 203c0190 adrp     x0, #0x4639000
01eb56c4 007040f9 ldr      x0, [x0, #0xe0]
01eb56c8 099cbb97 bl       #0xd9c6ec
01eb56cc c03a01f0 adrp     x0, #0x4610000
01eb56d0 00b445f9 ldr      x0, [x0, #0xb68]
01eb56d4 069cbb97 bl       #0xd9c6ec
01eb56d8 28008052 mov      w8, #1
01eb56dc 88c63e39 strb     w8, [x20, #0xfb1]
01eb56e0 a0288652 mov      w0, #0x3145
01eb56e4 e1031faa mov      x1, xzr
01eb56e8 d52cfb97 bl       #0x1d80a3c
01eb56ec 60010036 tbz      w0, #0, #0x1eb5718
01eb56f0 a0288652 mov      w0, #0x3145
01eb56f4 e1031faa mov      x1, xzr
01eb56f8 fe2cfb97 bl       #0x1d80af0
01eb56fc 801100b4 cbz      x0, #0x1eb592c
01eb5700 e10313aa mov      x1, x19
01eb5704 f37b42a9 ldp      x19, x30, [sp, #0x20]
01eb5708 f55341a9 ldp      x21, x20, [sp, #0x10]
01eb570c e2031faa mov      x2, xzr
01eb5710 e923c36c ldp      d9, d8, [sp], #0x30
01eb5714 786af717 b        #0x1c900f4
01eb5718 687644bd ldr      s8, [x19, #0x474]
01eb571c e0031faa mov      x0, xzr
01eb5720 28b85594 bl       #0x34237c0
01eb5724 694244bd ldr      s9, [x19, #0x440]
01eb5728 0039201e fsub     s0, s8, s0
01eb572c e0031faa mov      x0, xzr
01eb5730 607604bd str      s0, [x19, #0x474]
01eb5734 23b85594 bl       #0x34237c0
01eb5738 684645bd ldr      s8, [x19, #0x544]
01eb573c 2039201e fsub     s0, s9, s0
01eb5740 e0031faa mov      x0, xzr
01eb5744 604204bd str      s0, [x19, #0x440]
01eb5748 1eb85594 bl       #0x34237c0
01eb574c 694644bd ldr      s9, [x19, #0x444]
01eb5750 0039201e fsub     s0, s8, s0
01eb5754 e0031faa mov      x0, xzr
01eb5758 604605bd str      s0, [x19, #0x544]
01eb575c 19b85594 bl       #0x34237c0
01eb5760 2039201e fsub     s0, s9, s0
01eb5764 e0031faa mov      x0, xzr
01eb5768 604604bd str      s0, [x19, #0x444]
01eb576c 15b85594 bl       #0x34237c0
01eb5770 685e46bd ldr      s8, [x19, #0x65c]
01eb5774 e0031faa mov      x0, xzr
01eb5778 60e604bd str      s0, [x19, #0x4e4]
01eb577c 11b85594 bl       #0x34237c0
01eb5780 68824439 ldrb     w8, [x19, #0x120]
01eb5784 0039201e fsub     s0, s8, s0
01eb5788 605e06bd str      s0, [x19, #0x65c]
01eb578c a8000034 cbz      w8, #0x1eb57a0
01eb5790 f37b42a9 ldp      x19, x30, [sp, #0x20]
01eb5794 f55341a9 ldp      x21, x20, [sp, #0x10]
01eb5798 e923c36c ldp      d9, d8, [sp], #0x30
01eb579c c0035fd6 ret      
01eb57a0 543901f0 adrp     x20, #0x45e0000
01eb57a4 945242f9 ldr      x20, [x20, #0x4a0]
01eb57a8 800240f9 ldr      x0, [x20]
01eb57ac 08e040b9 ldr      w8, [x0, #0xe0]
01eb57b0 68000035 cbnz     w8, #0x1eb57bc
01eb57b4 019cbb97 bl       #0xd9c7b8
01eb57b8 800240f9 ldr      x0, [x20]
01eb57bc 085c40f9 ldr      x8, [x0, #0xb8]
01eb57c0 084540f9 ldr      x8, [x8, #0x88]
01eb57c4 480b00b4 cbz      x8, #0x1eb592c
01eb57c8 08614039 ldrb     w8, [x8, #0x18]
01eb57cc 28feff34 cbz      w8, #0x1eb5790
01eb57d0 d53a01f0 adrp     x21, #0x4610000
01eb57d4 688643bd ldr      s8, [x19, #0x384]
01eb57d8 b5b645f9 ldr      x21, [x21, #0xb68]
01eb57dc e0031faa mov      x0, xzr
01eb57e0 f8b75594 bl       #0x34237c0
01eb57e4 0039201e fsub     s0, s8, s0
01eb57e8 0820201e fcmp     s0, #0.0
01eb57ec 608603bd str      s0, [x19, #0x384]
01eb57f0 e8000054 b.hi     #0x1eb580c
01eb57f4 e00313aa mov      x0, x19
01eb57f8 4e000094 bl       #0x1eb5930
01eb57fc 80000037 tbnz     w0, #0, #0x1eb580c
01eb5800 604244bd ldr      s0, [x19, #0x440]
01eb5804 0820201e fcmp     s0, #0.0
01eb5808 29080054 b.ls     #0x1eb590c
01eb580c 7ff20d39 strb     wzr, [x19, #0x37c]
01eb5810 343c0190 adrp     x20, #0x4639000
01eb5814 688a43bd ldr      s8, [x19, #0x388]
01eb5818 947240f9 ldr      x20, [x20, #0xe0]
01eb581c e0031faa mov      x0, xzr
01eb5820 e8b75594 bl       #0x34237c0
01eb5824 0039201e fsub     s0, s8, s0
01eb5828 0820201e fcmp     s0, #0.0
01eb582c e103271e fmov     s1, wzr
01eb5830 209c201e fcsel    s0, s1, s0, ls
01eb5834 608a03bd str      s0, [x19, #0x388]
01eb5838 a00240f9 ldr      x0, [x21]
01eb583c 08e040b9 ldr      w8, [x0, #0xe0]
01eb5840 68000035 cbnz     w8, #0x1eb584c
01eb5844 dd9bbb97 bl       #0xd9c7b8
01eb5848 a00240f9 ldr      x0, [x21]
01eb584c 880240f9 ldr      x8, [x20]
01eb5850 095c40f9 ldr      x9, [x0, #0xb8]
01eb5854 0ae140b9 ldr      w10, [x8, #0xe0]
01eb5858 340140f9 ldr      x20, [x9]
01eb585c 6a000035 cbnz     w10, #0x1eb5868
01eb5860 e00308aa mov      x0, x8
01eb5864 d59bbb97 bl       #0xd9c7b8
01eb5868 e00314aa mov      x0, x20
01eb586c e1031faa mov      x1, xzr
01eb5870 e1f9bf97 bl       #0xeb3ff4
01eb5874 60aa03bd str      s0, [x19, #0x3a8]
01eb5878 a80240f9 ldr      x8, [x21]
01eb587c e1031faa mov      x1, xzr
01eb5880 085d40f9 ldr      x8, [x8, #0xb8]
01eb5884 000540f9 ldr      x0, [x8, #8]
01eb5888 dbf9bf97 bl       #0xeb3ff4
01eb588c e00313aa mov      x0, x19
01eb5890 60ae03bd str      s0, [x19, #0x3ac]
01eb5894 52000094 bl       #0x1eb59dc
01eb5898 e00313aa mov      x0, x19
01eb589c 95000094 bl       #0x1eb5af0
01eb58a0 e00313aa mov      x0, x19
01eb58a4 b6010094 bl       #0x1eb5f7c
01eb58a8 e00313aa mov      x0, x19
01eb58ac 11020094 bl       #0x1eb60f0
01eb58b0 608242f9 ldr      x0, [x19, #0x500]
01eb58b4 c00300b4 cbz      x0, #0x1eb592c
01eb58b8 f43801b0 adrp     x20, #0x45d2000
01eb58bc 94fe40f9 ldr      x20, [x20, #0x1f8]
01eb58c0 810240f9 ldr      x1, [x20]
01eb58c4 25813c94 bl       #0x2dd5d58
01eb58c8 608642f9 ldr      x0, [x19, #0x508]
01eb58cc 000300b4 cbz      x0, #0x1eb592c
01eb58d0 810240f9 ldr      x1, [x20]
01eb58d4 21813c94 bl       #0x2dd5d58
01eb58d8 e00313aa mov      x0, x19
01eb58dc 28d6ff97 bl       #0x1eab17c
01eb58e0 c00000b4 cbz      x0, #0x1eb58f8
01eb58e4 e00313aa mov      x0, x19
01eb58e8 25d6ff97 bl       #0x1eab17c
01eb58ec 000200b4 cbz      x0, #0x1eb592c
01eb58f0 e1031faa mov      x1, xzr
01eb58f4 544ad397 bl       #0x1388244
01eb58f8 e00313aa mov      x0, x19
01eb58fc f37b42a9 ldp      x19, x30, [sp, #0x20]
01eb5900 f55341a9 ldp      x21, x20, [sp, #0x10]
01eb5904 e923c36c ldp      d9, d8, [sp], #0x30
01eb5908 ae020014 b        #0x1eb63c0
01eb590c 68f24d39 ldrb     w8, [x19, #0x37c]
01eb5910 7f8603b9 str      wzr, [x19, #0x384]
01eb5914 e8f7ff35 cbnz     w8, #0x1eb5810
01eb5918 28008052 mov      w8, #1
01eb591c e00313aa mov      x0, x19
01eb5920 68f20d39 strb     w8, [x19, #0x37c]
01eb5924 1d000094 bl       #0x1eb5998
01eb5928 baffff17 b        #0x1eb5810
01eb592c ac9bbb97 bl       #0xd9c7dc
