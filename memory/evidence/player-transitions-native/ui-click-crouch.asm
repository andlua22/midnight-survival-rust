014ddd9c ffc300d1 sub      sp, sp, #0x30
014ddda0 f55301a9 stp      x21, x20, [sp, #0x10]
014ddda4 f37b02a9 stp      x19, x30, [sp, #0x20]
014ddda8 15a10190 adrp     x21, #0x48fd000
014dddac a8267939 ldrb     w8, [x21, #0xe49]
014dddb0 f30301aa mov      x19, x1
014dddb4 f40300aa mov      x20, x0
014dddb8 a8020037 tbnz     w8, #0, #0x14dde0c
014dddbc 008901f0 adrp     x0, #0x4600000
014dddc0 004044f9 ldr      x0, [x0, #0x880]
014dddc4 4afae297 bl       #0xd9c6ec
014dddc8 808901b0 adrp     x0, #0x460e000
014dddcc 007842f9 ldr      x0, [x0, #0x4f0]
014dddd0 47fae297 bl       #0xd9c6ec
014dddd4 808901f0 adrp     x0, #0x4610000
014dddd8 00a045f9 ldr      x0, [x0, #0xb40]
014ddddc 44fae297 bl       #0xd9c6ec
014ddde0 008801f0 adrp     x0, #0x45e0000
014ddde4 005042f9 ldr      x0, [x0, #0x4a0]
014ddde8 41fae297 bl       #0xd9c6ec
014dddec a08a01b0 adrp     x0, #0x4632000
014dddf0 006c45f9 ldr      x0, [x0, #0xad8]
014dddf4 3efae297 bl       #0xd9c6ec
014dddf8 208901f0 adrp     x0, #0x4604000
014dddfc 00ec46f9 ldr      x0, [x0, #0xdd8]
014dde00 3bfae297 bl       #0xd9c6ec
014dde04 28008052 mov      w8, #1
014dde08 a8263939 strb     w8, [x21, #0xe49]
014dde0c 60a19652 mov      w0, #0xb50b
014dde10 e1031faa mov      x1, xzr
014dde14 0a8b2294 bl       #0x1d80a3c
014dde18 80010036 tbz      w0, #0, #0x14dde48
014dde1c 60a19652 mov      w0, #0xb50b
014dde20 e1031faa mov      x1, xzr
014dde24 338b2294 bl       #0x1d80af0
014dde28 a00800b4 cbz      x0, #0x14ddf3c
014dde2c e10314aa mov      x1, x20
014dde30 e20313aa mov      x2, x19
014dde34 f37b42a9 ldp      x19, x30, [sp, #0x20]
014dde38 f55341a9 ldp      x21, x20, [sp, #0x10]
014dde3c e3031faa mov      x3, xzr
014dde40 ffc30091 add      sp, sp, #0x30
014dde44 efc41e14 b        #0x1c8f200
014dde48 881640f9 ldr      x8, [x20, #0x28]
014dde4c 880700b4 cbz      x8, #0x14ddf3c
014dde50 088142f9 ldr      x8, [x8, #0x500]
014dde54 480700b4 cbz      x8, #0x14ddf3c
014dde58 002140f9 ldr      x0, [x8, #0x40]
014dde5c 000700b4 cbz      x0, #0x14ddf3c
014dde60 088901f0 adrp     x8, #0x4600000
014dde64 084144f9 ldr      x8, [x8, #0x880]
014dde68 010140f9 ldr      x1, [x8]
014dde6c 92db6394 bl       #0x2dd4cb4
014dde70 1f100071 cmp      w0, #4
014dde74 c1050054 b.ne     #0x14ddf2c
014dde78 138801f0 adrp     x19, #0x45e0000
014dde7c 735242f9 ldr      x19, [x19, #0x4a0]
014dde80 600240f9 ldr      x0, [x19]
014dde84 08e040b9 ldr      w8, [x0, #0xe0]
014dde88 68000035 cbnz     w8, #0x14dde94
014dde8c 4bfae297 bl       #0xd9c7b8
014dde90 600240f9 ldr      x0, [x19]
014dde94 085c40f9 ldr      x8, [x0, #0xb8]
014dde98 084540f9 ldr      x8, [x8, #0x88]
014dde9c 080500b4 cbz      x8, #0x14ddf3c
014ddea0 081140f9 ldr      x8, [x8, #0x20]
014ddea4 c80400b4 cbz      x8, #0x14ddf3c
014ddea8 a98a01b0 adrp     x9, #0x4632000
014ddeac 296d45f9 ldr      x9, [x9, #0xad8]
014ddeb0 358901f0 adrp     x21, #0x4604000
014ddeb4 138142f9 ldr      x19, [x8, #0x500]
014ddeb8 21008052 mov      w1, #1
014ddebc 200140f9 ldr      x0, [x9]
014ddec0 b5ee46f9 ldr      x21, [x21, #0xdd8]
014ddec4 18fae297 bl       #0xd9c724
014ddec8 f40300aa mov      x20, x0
014ddecc a00240f9 ldr      x0, [x21]
014dded0 08e0a752 mov      w8, #0x3f000000
014dded4 e1330091 add      x1, sp, #0xc
014dded8 e80f00b9 str      w8, [sp, #0xc]
014ddedc 39fae297 bl       #0xd9c7c0
014ddee0 f40200b4 cbz      x20, #0x14ddf3c
014ddee4 f50300aa mov      x21, x0
014ddee8 c00000b4 cbz      x0, #0x14ddf00
014ddeec 880240f9 ldr      x8, [x20]
014ddef0 e00315aa mov      x0, x21
014ddef4 012140f9 ldr      x1, [x8, #0x40]
014ddef8 31fae297 bl       #0xd9c7bc
014ddefc 400200b4 cbz      x0, #0x14ddf44
014ddf00 881a40b9 ldr      w8, [x20, #0x18]
014ddf04 e8010034 cbz      w8, #0x14ddf40
014ddf08 951200f9 str      x21, [x20, #0x20]
014ddf0c 930100b4 cbz      x19, #0x14ddf3c
014ddf10 888901b0 adrp     x8, #0x460e000
014ddf14 087942f9 ldr      x8, [x8, #0x4f0]
014ddf18 a1008052 mov      w1, #5
014ddf1c e00313aa mov      x0, x19
014ddf20 e20314aa mov      x2, x20
014ddf24 030140f9 ldr      x3, [x8]
014ddf28 d9dd6394 bl       #0x2dd568c
014ddf2c f37b42a9 ldp      x19, x30, [sp, #0x20]
014ddf30 f55341a9 ldp      x21, x20, [sp, #0x10]
014ddf34 ffc30091 add      sp, sp, #0x30
014ddf38 c0035fd6 ret      
014ddf3c 28fae297 bl       #0xd9c7dc
014ddf40 29fae297 bl       #0xd9c7e4
014ddf44 2ffae297 bl       #0xd9c800
014ddf48 e1031faa mov      x1, xzr
014ddf4c 16fae297 bl       #0xd9c7a4
