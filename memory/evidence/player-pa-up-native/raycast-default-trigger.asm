03477b60 ffc301d1 sub      sp, sp, #0x70
03477b64 ee0b00fd str      d14, [sp, #0x10]
03477b68 ed33026d stp      d13, d12, [sp, #0x20]
03477b6c eb2b036d stp      d11, d10, [sp, #0x30]
03477b70 e923046d stp      d9, d8, [sp, #0x40]
03477b74 f55305a9 stp      x21, x20, [sp, #0x50]
03477b78 f37b06a9 stp      x19, x30, [sp, #0x60]
03477b7c b5a400b0 adrp     x21, #0x490c000
03477b80 a8de42f9 ldr      x8, [x21, #0x5b8]
03477b84 f303012a mov      w19, w1
03477b88 c81ca64e mov      v8.16b, v6.16b
03477b8c f40300aa mov      x20, x0
03477b90 a91ca54e mov      v9.16b, v5.16b
03477b94 8a1ca44e mov      v10.16b, v4.16b
03477b98 6b1ca34e mov      v11.16b, v3.16b
03477b9c 4c1ca24e mov      v12.16b, v2.16b
03477ba0 2d1ca14e mov      v13.16b, v1.16b
03477ba4 0e1ca04e mov      v14.16b, v0.16b
03477ba8 ff0b00b9 str      wzr, [sp, #8]
03477bac ff1b00b9 str      wzr, [sp, #0x18]
03477bb0 c80000b5 cbnz     x8, #0x3477bc8
03477bb4 e0140090 adrp     x0, #0x3713000
03477bb8 00c43391 add      x0, x0, #0xcf1
03477bbc bd926497 bl       #0xd9c6b0
03477bc0 e80300aa mov      x8, x0
03477bc4 a0de02f9 str      x0, [x21, #0x5b8]
03477bc8 e0630091 add      x0, sp, #0x18
03477bcc 00013fd6 blr      x8
03477bd0 e81b40b9 ldr      w8, [sp, #0x18]
03477bd4 e0230091 add      x0, sp, #8
03477bd8 c01dae4e mov      v0.16b, v14.16b
03477bdc a11dad4e mov      v1.16b, v13.16b
03477be0 821dac4e mov      v2.16b, v12.16b
03477be4 631dab4e mov      v3.16b, v11.16b
03477be8 441daa4e mov      v4.16b, v10.16b
03477bec 251da94e mov      v5.16b, v9.16b
03477bf0 e10314aa mov      x1, x20
03477bf4 061da84e mov      v6.16b, v8.16b
03477bf8 e203132a mov      w2, w19
03477bfc e3031f2a mov      w3, wzr
03477c00 e80b00b9 str      w8, [sp, #8]
03477c04 7dffff97 bl       #0x34779f8
03477c08 f37b46a9 ldp      x19, x30, [sp, #0x60]
03477c0c f55345a9 ldp      x21, x20, [sp, #0x50]
03477c10 e923446d ldp      d9, d8, [sp, #0x40]
03477c14 eb2b436d ldp      d11, d10, [sp, #0x30]
03477c18 ed33426d ldp      d13, d12, [sp, #0x20]
03477c1c ee0b40fd ldr      d14, [sp, #0x10]
03477c20 00000012 and      w0, w0, #1
03477c24 ffc30191 add      sp, sp, #0x70
03477c28 c0035fd6 ret      
Capstone 5.0: decoded 51 instructions / 204 bytes
