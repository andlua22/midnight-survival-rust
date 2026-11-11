034787e4 ef3bb96d stp      d15, d14, [sp, #-0x70]!
034787e8 ed33016d stp      d13, d12, [sp, #0x10]
034787ec eb2b026d stp      d11, d10, [sp, #0x20]
034787f0 e923036d stp      d9, d8, [sp, #0x30]
034787f4 f62300f9 str      x22, [sp, #0x40]
034787f8 f55305a9 stp      x21, x20, [sp, #0x50]
034787fc f37b06a9 stp      x19, x30, [sp, #0x60]
03478800 b6a40090 adrp     x22, #0x490c000
03478804 c8de42f9 ldr      x8, [x22, #0x5b8]
03478808 f303022a mov      w19, w2
0347880c f403012a mov      w20, w1
03478810 e81ca74e mov      v8.16b, v7.16b
03478814 f50300aa mov      x21, x0
03478818 c91ca64e mov      v9.16b, v6.16b
0347881c aa1ca54e mov      v10.16b, v5.16b
03478820 8b1ca44e mov      v11.16b, v4.16b
03478824 6c1ca34e mov      v12.16b, v3.16b
03478828 4d1ca24e mov      v13.16b, v2.16b
0347882c 2e1ca14e mov      v14.16b, v1.16b
03478830 0f1ca04e mov      v15.16b, v0.16b
03478834 ff4b00b9 str      wzr, [sp, #0x48]
03478838 c80000b5 cbnz     x8, #0x3478850
0347883c c01400f0 adrp     x0, #0x3713000
03478840 00c43391 add      x0, x0, #0xcf1
03478844 9b8f6497 bl       #0xd9c6b0
03478848 e80300aa mov      x8, x0
0347884c c0de02f9 str      x0, [x22, #0x5b8]
03478850 e0230191 add      x0, sp, #0x48
03478854 00013fd6 blr      x8
03478858 e04b40b9 ldr      w0, [sp, #0x48]
0347885c e01daf4e mov      v0.16b, v15.16b
03478860 c11dae4e mov      v1.16b, v14.16b
03478864 a21dad4e mov      v2.16b, v13.16b
03478868 831dac4e mov      v3.16b, v12.16b
0347886c 641dab4e mov      v4.16b, v11.16b
03478870 451daa4e mov      v5.16b, v10.16b
03478874 261da94e mov      v6.16b, v9.16b
03478878 e10315aa mov      x1, x21
0347887c 071da84e mov      v7.16b, v8.16b
03478880 e203142a mov      w2, w20
03478884 e303132a mov      w3, w19
03478888 53140094 bl       #0x347d9d4
0347888c f37b46a9 ldp      x19, x30, [sp, #0x60]
03478890 f55345a9 ldp      x21, x20, [sp, #0x50]
03478894 f62340f9 ldr      x22, [sp, #0x40]
03478898 e923436d ldp      d9, d8, [sp, #0x30]
0347889c eb2b426d ldp      d11, d10, [sp, #0x20]
034788a0 ed33416d ldp      d13, d12, [sp, #0x10]
034788a4 00000012 and      w0, w0, #1
034788a8 ef3bc76c ldp      d15, d14, [sp], #0x70
034788ac c0035fd6 ret      
Capstone 5.0: decoded 51 instructions / 204 bytes
