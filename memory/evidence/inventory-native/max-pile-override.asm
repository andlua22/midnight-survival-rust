01800b20 ff8301d1 sub      sp, sp, #0x60
01800b24 f61b00f9 str      x22, [sp, #0x30]
01800b28 f55304a9 stp      x21, x20, [sp, #0x40]
01800b2c f37b05a9 stp      x19, x30, [sp, #0x50]
01800b30 f58701f0 adrp     x21, #0x48ff000
01800b34 a8664239 ldrb     w8, [x21, #0x99]
01800b38 f303012a mov      w19, w1
01800b3c f40300aa mov      x20, x0
01800b40 a8020037 tbnz     w8, #0, #0x1800b94
01800b44 806f01f0 adrp     x0, #0x45f3000
01800b48 005040f9 ldr      x0, [x0, #0xa0]
01800b4c e86ed697 bl       #0xd9c6ec
01800b50 60700190 adrp     x0, #0x460c000
01800b54 00c843f9 ldr      x0, [x0, #0x790]
01800b58 e56ed697 bl       #0xd9c6ec
01800b5c c06e01f0 adrp     x0, #0x45db000
01800b60 00a445f9 ldr      x0, [x0, #0xb48]
01800b64 e26ed697 bl       #0xd9c6ec
01800b68 607101b0 adrp     x0, #0x462d000
01800b6c 006c45f9 ldr      x0, [x0, #0xad8]
01800b70 df6ed697 bl       #0xd9c6ec
01800b74 20700190 adrp     x0, #0x4604000
01800b78 008c43f9 ldr      x0, [x0, #0x718]
01800b7c dc6ed697 bl       #0xd9c6ec
01800b80 e06f01b0 adrp     x0, #0x45fd000
01800b84 003c46f9 ldr      x0, [x0, #0xc78]
01800b88 d96ed697 bl       #0xd9c6ec
01800b8c 28008052 mov      w8, #1
01800b90 a8660239 strb     w8, [x21, #0x99]
01800b94 00e4006f movi     v0.2d, #0000000000000000
01800b98 a0eb8052 mov      w0, #0x75d
01800b9c e1031faa mov      x1, xzr
01800ba0 ff1300f9 str      xzr, [sp, #0x20]
01800ba4 e00300ad stp      q0, q0, [sp]
01800ba8 a5ff1594 bl       #0x1d80a3c
01800bac 60010036 tbz      w0, #0, #0x1800bd8
01800bb0 a0eb8052 mov      w0, #0x75d
01800bb4 e1031faa mov      x1, xzr
01800bb8 ceff1594 bl       #0x1d80af0
01800bbc 600600b4 cbz      x0, #0x1800c88
01800bc0 e10314aa mov      x1, x20
01800bc4 e203132a mov      w2, w19
01800bc8 e3031faa mov      x3, xzr
01800bcc de601294 bl       #0x1c98f44
01800bd0 f303002a mov      w19, w0
01800bd4 27000014 b        #0x1800c70
01800bd8 883640f9 ldr      x8, [x20, #0x68]
01800bdc a80400b4 cbz      x8, #0x1800c70
01800be0 000d40f9 ldr      x0, [x8, #0x18]
01800be4 200500b4 cbz      x0, #0x1800c88
01800be8 886f01f0 adrp     x8, #0x45f3000
01800bec 085140f9 ldr      x8, [x8, #0xa0]
01800bf0 d66e01f0 adrp     x22, #0x45db000
01800bf4 75700190 adrp     x21, #0x460c000
01800bf8 010140f9 ldr      x1, [x8]
01800bfc d6a645f9 ldr      x22, [x22, #0xb48]
01800c00 b5ca43f9 ldr      x21, [x21, #0x790]
01800c04 e8030091 mov      x8, sp
01800c08 32ca3d94 bl       #0x27734d0
01800c0c c10240f9 ldr      x1, [x22]
01800c10 e0030091 mov      x0, sp
01800c14 b13f4394 bl       #0x28d0ad8
01800c18 e0010036 tbz      w0, #0, #0x1800c54
01800c1c e01340b9 ldr      w0, [sp, #0x10]
01800c20 f40f40f9 ldr      x20, [sp, #0x18]
01800c24 e1031faa mov      x1, xzr
01800c28 63df2794 bl       #0x21f89b4
01800c2c 00ffffb4 cbz      x0, #0x1800c0c
01800c30 081840b9 ldr      w8, [x0, #0x18]
01800c34 1f250071 cmp      w8, #9
01800c38 a1feff54 b.ne     #0x1800c0c
01800c3c e00314aa mov      x0, x20
01800c40 e1031faa mov      x1, xzr
01800c44 8b476394 bl       #0x30d2a70
01800c48 f403002a mov      w20, w0
01800c4c d6008052 mov      w22, #6
01800c50 03000014 b        #0x1800c5c
01800c54 f4031f2a mov      w20, wzr
01800c58 f6008052 mov      w22, #7
01800c5c a10240f9 ldr      x1, [x21]
01800c60 e0030091 mov      x0, sp
01800c64 e13f4394 bl       #0x28d0be8
01800c68 df1a0071 cmp      w22, #6
01800c6c 9302931a csel     w19, w20, w19, eq
01800c70 e003132a mov      w0, w19
01800c74 f37b45a9 ldp      x19, x30, [sp, #0x50]
01800c78 f55344a9 ldp      x21, x20, [sp, #0x40]
01800c7c f61b40f9 ldr      x22, [sp, #0x30]
01800c80 ff830191 add      sp, sp, #0x60
01800c84 c0035fd6 ret      
01800c88 d56ed697 bl       #0xd9c7dc
01800c8c 02000014 b        #0x1800c94
01800c90 01000014 b        #0x1800c94
01800c94 f40300aa mov      x20, x0
01800c98 3f040071 cmp      w1, #1
01800c9c c1010054 b.ne     #0x1800cd4
01800ca0 e00314aa mov      x0, x20
01800ca4 cb37c997 bl       #0xa4ebd0
01800ca8 150040f9 ldr      x21, [x0]
01800cac 4536c997 bl       #0xa4e5c0
01800cb0 68700190 adrp     x8, #0x460c000
01800cb4 08c943f9 ldr      x8, [x8, #0x790]
01800cb8 e0030091 mov      x0, sp
01800cbc 010140f9 ldr      x1, [x8]
01800cc0 ca3f4394 bl       #0x28d0be8
01800cc4 d50000b5 cbnz     x21, #0x1800cdc
01800cc8 f4031f2a mov      w20, wzr
01800ccc f6031f2a mov      w22, wzr
01800cd0 e6ffff17 b        #0x1800c68
01800cd4 f5031faa mov      x21, xzr
01800cd8 04000014 b        #0x1800ce8
01800cdc e00315aa mov      x0, x21
01800ce0 f8f6d797 bl       #0xdfe8c0
01800ce4 f40300aa mov      x20, x0
01800ce8 68700190 adrp     x8, #0x460c000
01800cec 08c943f9 ldr      x8, [x8, #0x790]
01800cf0 010140f9 ldr      x1, [x8]
01800cf4 e0030091 mov      x0, sp
01800cf8 bc3f4394 bl       #0x28d0be8
01800cfc 750000b5 cbnz     x21, #0x1800d08
01800d00 e00314aa mov      x0, x20
01800d04 ff35c997 bl       #0xa4e500
01800d08 e00315aa mov      x0, x21
01800d0c edf6d797 bl       #0xdfe8c0
01800d10 8a3dc997 bl       #0xa50338
