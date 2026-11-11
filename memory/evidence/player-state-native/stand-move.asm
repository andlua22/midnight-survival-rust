01e7b748 e923bc6d stp      d9, d8, [sp, #-0x40]!
01e7b74c f75b01a9 stp      x23, x22, [sp, #0x10]
01e7b750 f55302a9 stp      x21, x20, [sp, #0x20]
01e7b754 f37b03a9 stp      x19, x30, [sp, #0x30]
01e7b758 345401d0 adrp     x20, #0x4901000
01e7b75c 88467939 ldrb     w8, [x20, #0xe51]
01e7b760 f30300aa mov      x19, x0
01e7b764 c8000037 tbnz     w8, #0, #0x1e7b77c
01e7b768 203d01f0 adrp     x0, #0x4622000
01e7b76c 00e440f9 ldr      x0, [x0, #0x1c8]
01e7b770 df83bc97 bl       #0xd9c6ec
01e7b774 28008052 mov      w8, #1
01e7b778 88463939 strb     w8, [x20, #0xe51]
01e7b77c 80db8552 mov      w0, #0x2edc
01e7b780 e1031faa mov      x1, xzr
01e7b784 ae14fc97 bl       #0x1d80a3c
01e7b788 80010036 tbz      w0, #0, #0x1e7b7b8
01e7b78c 80db8552 mov      w0, #0x2edc
01e7b790 e1031faa mov      x1, xzr
01e7b794 d714fc97 bl       #0x1d80af0
01e7b798 c00e00b4 cbz      x0, #0x1e7b970
01e7b79c e10313aa mov      x1, x19
01e7b7a0 f37b43a9 ldp      x19, x30, [sp, #0x30]
01e7b7a4 f55342a9 ldp      x21, x20, [sp, #0x20]
01e7b7a8 f75b41a9 ldp      x23, x22, [sp, #0x10]
01e7b7ac e2031faa mov      x2, xzr
01e7b7b0 e923c46c ldp      d9, d8, [sp], #0x40
01e7b7b4 5052f817 b        #0x1c900f4
01e7b7b8 343d01f0 adrp     x20, #0x4622000
01e7b7bc 94e640f9 ldr      x20, [x20, #0x1c8]
01e7b7c0 880240f9 ldr      x8, [x20]
01e7b7c4 085d40f9 ldr      x8, [x8, #0xb8]
01e7b7c8 1f010079 strh     wzr, [x8]
01e7b7cc 771e40f9 ldr      x23, [x19, #0x38]
01e7b7d0 170d00b4 cbz      x23, #0x1e7b970
01e7b7d4 ffe20d39 strb     wzr, [x23, #0x378]
01e7b7d8 15540190 adrp     x21, #0x48fb000
01e7b7dc a8525339 ldrb     w8, [x21, #0x4d4]
01e7b7e0 c8000035 cbnz     w8, #0x1e7b7f8
01e7b7e4 803c0190 adrp     x0, #0x460b000
01e7b7e8 000042f9 ldr      x0, [x0, #0x400]
01e7b7ec c083bc97 bl       #0xd9c6ec
01e7b7f0 28008052 mov      w8, #1
01e7b7f4 a8521339 strb     w8, [x21, #0x4d4]
01e7b7f8 963c0190 adrp     x22, #0x460b000
01e7b7fc d60242f9 ldr      x22, [x22, #0x400]
01e7b800 e8aa43bd ldr      s8, [x23, #0x3a8]
01e7b804 e9ae43bd ldr      s9, [x23, #0x3ac]
01e7b808 c00240f9 ldr      x0, [x22]
01e7b80c 08e040b9 ldr      w8, [x0, #0xe0]
01e7b810 48000035 cbnz     w8, #0x1e7b818
01e7b814 e983bc97 bl       #0xd9c7b8
01e7b818 0009281e fmul     s0, s8, s8
01e7b81c 2109291e fmul     s1, s9, s9
01e7b820 28bd00f0 adrp     x8, #0x3622000
01e7b824 0028211e fadd     s0, s0, s1
01e7b828 01b546fd ldr      d1, [x8, #0xd68]
01e7b82c 771e40f9 ldr      x23, [x19, #0x38]
01e7b830 00c0211e fsqrt    s0, s0
01e7b834 00c0221e fcvt     d0, s0
01e7b838 0020611e fcmp     d0, d1
01e7b83c 4d040054 b.le     #0x1e7b8c4
01e7b840 970900b4 cbz      x23, #0x1e7b970
01e7b844 e8f24d39 ldrb     w8, [x23, #0x37c]
01e7b848 08040034 cbz      w8, #0x1e7b8c8
01e7b84c e0aa43bd ldr      s0, [x23, #0x3a8]
01e7b850 e1ae43bd ldr      s1, [x23, #0x3ac]
01e7b854 e0031faa mov      x0, xzr
01e7b858 84930a94 bl       #0x2120668
01e7b85c e07204bd str      s0, [x23, #0x470]
01e7b860 601e40f9 ldr      x0, [x19, #0x38]
01e7b864 600800b4 cbz      x0, #0x1e7b970
01e7b868 007044bd ldr      s0, [x0, #0x470]
01e7b86c 0820201e fcmp     s0, #0.0
01e7b870 ab000054 b.lt     #0x1e7b884
01e7b874 c852a852 mov      w8, #0x42960000
01e7b878 0101271e fmov     s1, w8
01e7b87c 0020211e fcmp     s0, s1
01e7b880 24010054 b.mi     #0x1e7b8a4
01e7b884 28bd00f0 adrp     x8, #0x3622000
01e7b888 01e54dbd ldr      s1, [x8, #0xde4]
01e7b88c 0020211e fcmp     s0, s1
01e7b890 4d050054 b.le     #0x1e7b938
01e7b894 8876a852 mov      w8, #0x43b40000
01e7b898 0101271e fmov     s1, w8
01e7b89c 0020211e fcmp     s0, s1
01e7b8a0 c8040054 b.hi     #0x1e7b938
01e7b8a4 880240f9 ldr      x8, [x20]
01e7b8a8 29008052 mov      w9, #1
01e7b8ac 0a08a852 mov      w10, #0x40400000
01e7b8b0 085d40f9 ldr      x8, [x8, #0xb8]
01e7b8b4 09010039 strb     w9, [x8]
01e7b8b8 0a4805b9 str      w10, [x0, #0x548]
01e7b8bc 09e00d39 strb     w9, [x0, #0x378]
01e7b8c0 20000014 b        #0x1e7b940
01e7b8c4 770500b4 cbz      x23, #0x1e7b970
01e7b8c8 a8525339 ldrb     w8, [x21, #0x4d4]
01e7b8cc c8000035 cbnz     w8, #0x1e7b8e4
01e7b8d0 803c0190 adrp     x0, #0x460b000
01e7b8d4 000042f9 ldr      x0, [x0, #0x400]
01e7b8d8 8583bc97 bl       #0xd9c6ec
01e7b8dc 28008052 mov      w8, #1
01e7b8e0 a8521339 strb     w8, [x21, #0x4d4]
01e7b8e4 c00240f9 ldr      x0, [x22]
01e7b8e8 e8aa43bd ldr      s8, [x23, #0x3a8]
01e7b8ec e9ae43bd ldr      s9, [x23, #0x3ac]
01e7b8f0 08e040b9 ldr      w8, [x0, #0xe0]
01e7b8f4 48000035 cbnz     w8, #0x1e7b8fc
01e7b8f8 b083bc97 bl       #0xd9c7b8
01e7b8fc 0009281e fmul     s0, s8, s8
01e7b900 2109291e fmul     s1, s9, s9
01e7b904 0028211e fadd     s0, s0, s1
01e7b908 00c0211e fsqrt    s0, s0
01e7b90c 0820201e fcmp     s0, #0.0
01e7b910 8a000054 b.ge     #0x1e7b920
01e7b914 601e40f9 ldr      x0, [x19, #0x38]
01e7b918 400100b5 cbnz     x0, #0x1e7b940
01e7b91c 15000014 b        #0x1e7b970
01e7b920 880240f9 ldr      x8, [x20]
01e7b924 29008052 mov      w9, #1
01e7b928 085d40f9 ldr      x8, [x8, #0xb8]
01e7b92c 09050039 strb     w9, [x8, #1]
01e7b930 601e40f9 ldr      x0, [x19, #0x38]
01e7b934 e00100b4 cbz      x0, #0x1e7b970
01e7b938 0800a852 mov      w8, #0x40000000
01e7b93c 084805b9 str      w8, [x0, #0x548]
01e7b940 e1031faa mov      x1, xzr
01e7b944 7c250194 bl       #0x1ec4f34
01e7b948 681e40f9 ldr      x8, [x19, #0x38]
01e7b94c 280100b4 cbz      x8, #0x1e7b970
01e7b950 09e14d39 ldrb     w9, [x8, #0x378]
01e7b954 e00313aa mov      x0, x19
01e7b958 09311539 strb     w9, [x8, #0x54c]
01e7b95c f37b43a9 ldp      x19, x30, [sp, #0x30]
01e7b960 f55342a9 ldp      x21, x20, [sp, #0x20]
01e7b964 f75b41a9 ldp      x23, x22, [sp, #0x10]
01e7b968 e923c46c ldp      d9, d8, [sp], #0x40
01e7b96c 1f030014 b        #0x1e7c5e8
01e7b970 9b83bc97 bl       #0xd9c7dc
Capstone 5.0: decoded 139 instructions / 556 bytes
