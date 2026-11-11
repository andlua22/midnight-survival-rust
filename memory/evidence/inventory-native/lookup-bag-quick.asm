01948d90 ff0302d1 sub      sp, sp, #0x80
01948d94 fa1b00f9 str      x26, [sp, #0x30]
01948d98 f96304a9 stp      x25, x24, [sp, #0x40]
01948d9c f75b05a9 stp      x23, x22, [sp, #0x50]
01948da0 f55306a9 stp      x21, x20, [sp, #0x60]
01948da4 f37b07a9 stp      x19, x30, [sp, #0x70]
01948da8 b67d01f0 adrp     x22, #0x48ff000
01948dac c8aa5c39 ldrb     w8, [x22, #0x72a]
01948db0 f30302aa mov      x19, x2
01948db4 f403012a mov      w20, w1
01948db8 f50300aa mov      x21, x0
01948dbc 08030037 tbnz     w8, #0, #0x1948e1c
01948dc0 406401b0 adrp     x0, #0x45d1000
01948dc4 001041f9 ldr      x0, [x0, #0x220]
01948dc8 494ed197 bl       #0xd9c6ec
01948dcc 606701d0 adrp     x0, #0x4636000
01948dd0 00b843f9 ldr      x0, [x0, #0x770]
01948dd4 464ed197 bl       #0xd9c6ec
01948dd8 006501f0 adrp     x0, #0x45eb000
01948ddc 002c47f9 ldr      x0, [x0, #0xe58]
01948de0 434ed197 bl       #0xd9c6ec
01948de4 e06301b0 adrp     x0, #0x45c5000
01948de8 00b840f9 ldr      x0, [x0, #0x170]
01948dec 404ed197 bl       #0xd9c6ec
01948df0 e06401b0 adrp     x0, #0x45e5000
01948df4 001842f9 ldr      x0, [x0, #0x430]
01948df8 3d4ed197 bl       #0xd9c6ec
01948dfc c06401f0 adrp     x0, #0x45e3000
01948e00 000842f9 ldr      x0, [x0, #0x410]
01948e04 3a4ed197 bl       #0xd9c6ec
01948e08 e0630190 adrp     x0, #0x45c4000
01948e0c 00f042f9 ldr      x0, [x0, #0x5e0]
01948e10 374ed197 bl       #0xd9c6ec
01948e14 28008052 mov      w8, #1
01948e18 c8aa1c39 strb     w8, [x22, #0x72a]
01948e1c 00e4006f movi     v0.2d, #0000000000000000
01948e20 00cd8052 mov      w0, #0x668
01948e24 e1031faa mov      x1, xzr
01948e28 ff1300f9 str      xzr, [sp, #0x20]
01948e2c e00300ad stp      q0, q0, [sp]
01948e30 03df1094 bl       #0x1d80a3c
01948e34 60010036 tbz      w0, #0, #0x1948e60
01948e38 00cd8052 mov      w0, #0x668
01948e3c e1031faa mov      x1, xzr
01948e40 2cdf1094 bl       #0x1d80af0
01948e44 200400b4 cbz      x0, #0x1948ec8
01948e48 e10315aa mov      x1, x21
01948e4c e203142a mov      w2, w20
01948e50 e30313aa mov      x3, x19
01948e54 e4031faa mov      x4, xzr
01948e58 43810d94 bl       #0x1ca9364
01948e5c 3d000014 b        #0x1948f50
01948e60 08008012 mov      w8, #-1
01948e64 680200b9 str      w8, [x19]
01948e68 a00a40f9 ldr      x0, [x21, #0x10]
01948e6c e00200b4 cbz      x0, #0x1948ec8
01948e70 f9630190 adrp     x25, #0x45c4000
01948e74 5a6401b0 adrp     x26, #0x45d1000
01948e78 186501f0 adrp     x24, #0x45eb000
01948e7c 776701d0 adrp     x23, #0x4636000
01948e80 39f342f9 ldr      x25, [x25, #0x5e0]
01948e84 5a1341f9 ldr      x26, [x26, #0x220]
01948e88 182f47f9 ldr      x24, [x24, #0xe58]
01948e8c f7ba43f9 ldr      x23, [x23, #0x770]
01948e90 f6031f2a mov      w22, wzr
01948e94 081840b9 ldr      w8, [x0, #0x18]
01948e98 df02086b cmp      w22, w8
01948e9c 8a010054 b.ge     #0x1948ecc
01948ea0 220340f9 ldr      x2, [x25]
01948ea4 e103162a mov      w1, w22
01948ea8 0d444894 bl       #0x2b59edc
01948eac e00000b4 cbz      x0, #0x1948ec8
01948eb0 081040b9 ldr      w8, [x0, #0x10]
01948eb4 1f01146b cmp      w8, w20
01948eb8 e0020054 b.eq     #0x1948f14
01948ebc a00a40f9 ldr      x0, [x21, #0x10]
01948ec0 d6060011 add      w22, w22, #1
01948ec4 80feffb5 cbnz     x0, #0x1948e94
01948ec8 454ed197 bl       #0xd9c7dc
01948ecc a00e40f9 ldr      x0, [x21, #0x18]
01948ed0 c0ffffb4 cbz      x0, #0x1948ec8
01948ed4 410340f9 ldr      x1, [x26]
01948ed8 e8030091 mov      x8, sp
01948edc 7da93894 bl       #0x27734d0
01948ee0 010340f9 ldr      x1, [x24]
01948ee4 e0030091 mov      x0, sp
01948ee8 fc1e3e94 bl       #0x28d0ad8
01948eec 40020036 tbz      w0, #0, #0x1948f34
01948ef0 f50f40f9 ldr      x21, [sp, #0x18]
01948ef4 d50300b4 cbz      x21, #0x1948f6c
01948ef8 a81240b9 ldr      w8, [x21, #0x10]
01948efc 1f01146b cmp      w8, w20
01948f00 01ffff54 b.ne     #0x1948ee0
01948f04 48008052 mov      w8, #2
01948f08 680200b9 str      w8, [x19]
01948f0c 13018052 mov      w19, #8
01948f10 0b000014 b        #0x1948f3c
01948f14 28008052 mov      w8, #1
01948f18 680200b9 str      w8, [x19]
01948f1c a00a40f9 ldr      x0, [x21, #0x10]
01948f20 40fdffb4 cbz      x0, #0x1948ec8
01948f24 220340f9 ldr      x2, [x25]
01948f28 e103162a mov      w1, w22
01948f2c ec434894 bl       #0x2b59edc
01948f30 08000014 b        #0x1948f50
01948f34 f5031faa mov      x21, xzr
01948f38 33018052 mov      w19, #9
01948f3c e10240f9 ldr      x1, [x23]
01948f40 e0030091 mov      x0, sp
01948f44 291f3e94 bl       #0x28d0be8
01948f48 7f220071 cmp      w19, #8
01948f4c a0029f9a csel     x0, x21, xzr, eq
01948f50 f37b47a9 ldp      x19, x30, [sp, #0x70]
01948f54 f55346a9 ldp      x21, x20, [sp, #0x60]
01948f58 f75b45a9 ldp      x23, x22, [sp, #0x50]
01948f5c f96344a9 ldp      x25, x24, [sp, #0x40]
01948f60 fa1b40f9 ldr      x26, [sp, #0x30]
01948f64 ff030291 add      sp, sp, #0x80
01948f68 c0035fd6 ret      
01948f6c 1c4ed197 bl       #0xd9c7dc
01948f70 01000014 b        #0x1948f74
01948f74 f40300aa mov      x20, x0
01948f78 3f040071 cmp      w1, #1
01948f7c 61010054 b.ne     #0x1948fa8
01948f80 e00314aa mov      x0, x20
01948f84 1317c497 bl       #0xa4ebd0
01948f88 130040f9 ldr      x19, [x0]
01948f8c 8d15c497 bl       #0xa4e5c0
01948f90 e10240f9 ldr      x1, [x23]
01948f94 e0030091 mov      x0, sp
01948f98 141f3e94 bl       #0x28d0be8
01948f9c b30000b5 cbnz     x19, #0x1948fb0
01948fa0 f5031faa mov      x21, xzr
01948fa4 e9ffff17 b        #0x1948f48
01948fa8 f3031faa mov      x19, xzr
01948fac 04000014 b        #0x1948fbc
01948fb0 e00313aa mov      x0, x19
01948fb4 43d6d297 bl       #0xdfe8c0
01948fb8 f40300aa mov      x20, x0
01948fbc e10240f9 ldr      x1, [x23]
01948fc0 e0030091 mov      x0, sp
01948fc4 091f3e94 bl       #0x28d0be8
01948fc8 730000b5 cbnz     x19, #0x1948fd4
01948fcc e00314aa mov      x0, x20
01948fd0 4c15c497 bl       #0xa4e500
01948fd4 e00313aa mov      x0, x19
01948fd8 3ad6d297 bl       #0xdfe8c0
01948fdc d71cc497 bl       #0xa50338
