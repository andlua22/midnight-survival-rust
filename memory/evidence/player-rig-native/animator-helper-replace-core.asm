0133f7c8 ff4301d1 sub      sp, sp, #0x50
0133f7cc f96301a9 stp      x25, x24, [sp, #0x10]
0133f7d0 f75b02a9 stp      x23, x22, [sp, #0x20]
0133f7d4 f55303a9 stp      x21, x20, [sp, #0x30]
0133f7d8 f37b04a9 stp      x19, x30, [sp, #0x40]
0133f7dc f4ad01d0 adrp     x20, #0x48fd000
0133f7e0 887e5339 ldrb     w8, [x20, #0x4df]
0133f7e4 f50303aa mov      x21, x3
0133f7e8 f603022a mov      w22, w2
0133f7ec f703012a mov      w23, w1
0133f7f0 f30300aa mov      x19, x0
0133f7f4 68030037 tbnz     w8, #0, #0x133f860
0133f7f8 a0940190 adrp     x0, #0x45d3000
0133f7fc 002440f9 ldr      x0, [x0, #0x48]
0133f800 bb73e997 bl       #0xd9c6ec
0133f804 c0940190 adrp     x0, #0x45d7000
0133f808 005443f9 ldr      x0, [x0, #0x6a8]
0133f80c b873e997 bl       #0xd9c6ec
0133f810 e0950190 adrp     x0, #0x45fb000
0133f814 009846f9 ldr      x0, [x0, #0xd30]
0133f818 b573e997 bl       #0xd9c6ec
0133f81c 609701d0 adrp     x0, #0x462d000
0133f820 003847f9 ldr      x0, [x0, #0xe70]
0133f824 b273e997 bl       #0xd9c6ec
0133f828 e09601b0 adrp     x0, #0x461c000
0133f82c 005c46f9 ldr      x0, [x0, #0xcb8]
0133f830 af73e997 bl       #0xd9c6ec
0133f834 e0930190 adrp     x0, #0x45bb000
0133f838 00c447f9 ldr      x0, [x0, #0xf88]
0133f83c ac73e997 bl       #0xd9c6ec
0133f840 609301f0 adrp     x0, #0x45ae000
0133f844 009040f9 ldr      x0, [x0, #0x120]
0133f848 a973e997 bl       #0xd9c6ec
0133f84c a09301f0 adrp     x0, #0x45b6000
0133f850 006045f9 ldr      x0, [x0, #0xac0]
0133f854 a673e997 bl       #0xd9c6ec
0133f858 28008052 mov      w8, #1
0133f85c 887e1339 strb     w8, [x20, #0x4df]
0133f860 e05b8052 mov      w0, #0x2df
0133f864 e1031faa mov      x1, xzr
0133f868 75042994 bl       #0x1d80a3c
0133f86c 00020036 tbz      w0, #0, #0x133f8ac
0133f870 e05b8052 mov      w0, #0x2df
0133f874 e1031faa mov      x1, xzr
0133f878 9e042994 bl       #0x1d80af0
0133f87c e01a00b4 cbz      x0, #0x133fbd8
0133f880 e10313aa mov      x1, x19
0133f884 e203172a mov      w2, w23
0133f888 e303162a mov      w3, w22
0133f88c e40315aa mov      x4, x21
0133f890 f37b44a9 ldp      x19, x30, [sp, #0x40]
0133f894 f55343a9 ldp      x21, x20, [sp, #0x30]
0133f898 f75b42a9 ldp      x23, x22, [sp, #0x20]
0133f89c f96341a9 ldp      x25, x24, [sp, #0x10]
0133f8a0 e5031faa mov      x5, xzr
0133f8a4 ff430191 add      sp, sp, #0x50
0133f8a8 f6762514 b        #0x1c9d480
0133f8ac a89301f0 adrp     x8, #0x45b6000
0133f8b0 086145f9 ldr      x8, [x8, #0xac0]
0133f8b4 000140f9 ldr      x0, [x8]
0133f8b8 c573e997 bl       #0xd9c7cc
0133f8bc e01800b4 cbz      x0, #0x133fbd8
0133f8c0 d8940190 adrp     x24, #0x45d7000
0133f8c4 f9930190 adrp     x25, #0x45bb000
0133f8c8 185743f9 ldr      x24, [x24, #0x6a8]
0133f8cc 39c747f9 ldr      x25, [x25, #0xf88]
0133f8d0 e1031faa mov      x1, xzr
0133f8d4 f40300aa mov      x20, x0
0133f8d8 c3627794 bl       #0x31183e4
0133f8dc 930a00f9 str      x19, [x20, #0x10]
0133f8e0 965e0329 stp      w22, w23, [x20, #0x18]
0133f8e4 951200f9 str      x21, [x20, #0x20]
0133f8e8 000340f9 ldr      x0, [x24]
0133f8ec 08e040b9 ldr      w8, [x0, #0xe0]
0133f8f0 68000035 cbnz     w8, #0x133f8fc
0133f8f4 b173e997 bl       #0xd9c7b8
0133f8f8 000340f9 ldr      x0, [x24]
0133f8fc 280340f9 ldr      x8, [x25]
0133f900 095c40f9 ldr      x9, [x0, #0xb8]
0133f904 0ae140b9 ldr      w10, [x8, #0xe0]
0133f908 350d40f9 ldr      x21, [x9, #0x18]
0133f90c 6a000035 cbnz     w10, #0x133f918
0133f910 e00308aa mov      x0, x8
0133f914 a973e997 bl       #0xd9c7b8
0133f918 e00315aa mov      x0, x21
0133f91c e1031faa mov      x1, xzr
0133f920 e2031faa mov      x2, xzr
0133f924 2f998394 bl       #0x3425de0
0133f928 a0010037 tbnz     w0, #0, #0x133f95c
0133f92c 68824039 ldrb     w8, [x19, #0x20]
0133f930 68010034 cbz      w8, #0x133f95c
0133f934 951e40b9 ldr      w21, [x20, #0x1c]
0133f938 bf060031 cmn      w21, #1
0133f93c 00010054 b.eq     #0x133f95c
0133f940 000340f9 ldr      x0, [x24]
0133f944 08e040b9 ldr      w8, [x0, #0xe0]
0133f948 48000035 cbnz     w8, #0x133f950
0133f94c 9b73e997 bl       #0xd9c7b8
0133f950 e003152a mov      w0, w21
0133f954 46010094 bl       #0x133fe6c
0133f958 80010036 tbz      w0, #0, #0x133f988
0133f95c 881240f9 ldr      x8, [x20, #0x20]
0133f960 480a00b4 cbz      x8, #0x133faa8
0133f964 020d40f9 ldr      x2, [x8, #0x18]
0133f968 002140f9 ldr      x0, [x8, #0x40]
0133f96c 011540f9 ldr      x1, [x8, #0x28]
0133f970 f37b44a9 ldp      x19, x30, [sp, #0x40]
0133f974 f55343a9 ldp      x21, x20, [sp, #0x30]
0133f978 f75b42a9 ldp      x23, x22, [sp, #0x20]
0133f97c f96341a9 ldp      x25, x24, [sp, #0x10]
0133f980 ff430191 add      sp, sp, #0x50
0133f984 40001fd6 br       x2
0133f988 000340f9 ldr      x0, [x24]
0133f98c 08e040b9 ldr      w8, [x0, #0xe0]
0133f990 68000035 cbnz     w8, #0x133f99c
0133f994 8973e997 bl       #0xd9c7b8
0133f998 000340f9 ldr      x0, [x24]
0133f99c 085c40f9 ldr      x8, [x0, #0xb8]
0133f9a0 000940f9 ldr      x0, [x8, #0x10]
0133f9a4 a01100b4 cbz      x0, #0x133fbd8
0133f9a8 e8950190 adrp     x8, #0x45fb000
0133f9ac 089946f9 ldr      x8, [x8, #0xd30]
0133f9b0 811e40b9 ldr      w1, [x20, #0x1c]
0133f9b4 020140f9 ldr      x2, [x8]
0133f9b8 35ce5094 bl       #0x277328c
0133f9bc 60070036 tbz      w0, #0, #0x133faa8
0133f9c0 681640f9 ldr      x8, [x19, #0x28]
0133f9c4 e80700b4 cbz      x8, #0x133fac0
0133f9c8 891a80b9 ldrsw    x9, [x20, #0x18]
0133f9cc 0a1940b9 ldr      w10, [x8, #0x18]
0133f9d0 3f010a6b cmp      w9, w10
0133f9d4 82070054 b.hs     #0x133fac4
0133f9d8 8a1e40b9 ldr      w10, [x20, #0x1c]
0133f9dc 0809098b add      x8, x8, x9, lsl #2
0133f9e0 0a2100b9 str      w10, [x8, #0x20]
0133f9e4 000340f9 ldr      x0, [x24]
0133f9e8 08e040b9 ldr      w8, [x0, #0xe0]
0133f9ec 68000035 cbnz     w8, #0x133f9f8
0133f9f0 7273e997 bl       #0xd9c7b8
0133f9f4 000340f9 ldr      x0, [x24]
0133f9f8 085c40f9 ldr      x8, [x0, #0xb8]
0133f9fc 000940f9 ldr      x0, [x8, #0x10]
0133fa00 400600b4 cbz      x0, #0x133fac8
0133fa04 e89601b0 adrp     x8, #0x461c000
0133fa08 085d46f9 ldr      x8, [x8, #0xcb8]
0133fa0c 811e40b9 ldr      w1, [x20, #0x1c]
0133fa10 020140f9 ldr      x2, [x8]
0133fa14 71cd5094 bl       #0x2772fd8
0133fa18 881240f9 ldr      x8, [x20, #0x20]
0133fa1c f50300aa mov      x21, x0
0133fa20 e80100b4 cbz      x8, #0x133fa5c
0133fa24 a8940190 adrp     x8, #0x45d3000
0133fa28 082540f9 ldr      x8, [x8, #0x48]
0133fa2c 000140f9 ldr      x0, [x8]
0133fa30 6773e997 bl       #0xd9c7cc
0133fa34 f60300aa mov      x22, x0
0133fa38 c00400b4 cbz      x0, #0x133fad0
0133fa3c 689301f0 adrp     x8, #0x45ae000
0133fa40 089140f9 ldr      x8, [x8, #0x120]
0133fa44 020140f9 ldr      x2, [x8]
0133fa48 e00316aa mov      x0, x22
0133fa4c e10314aa mov      x1, x20
0133fa50 e3031faa mov      x3, xzr
0133fa54 4aca7494 bl       #0x307237c
0133fa58 02000014 b        #0x133fa60
0133fa5c f6031faa mov      x22, xzr
0133fa60 000340f9 ldr      x0, [x24]
0133fa64 08e040b9 ldr      w8, [x0, #0xe0]
0133fa68 68000035 cbnz     w8, #0x133fa74
0133fa6c 5373e997 bl       #0xd9c7b8
0133fa70 000340f9 ldr      x0, [x24]
0133fa74 085c40f9 ldr      x8, [x0, #0xb8]
0133fa78 000540f9 ldr      x0, [x8, #8]
0133fa7c 800200b4 cbz      x0, #0x133facc
0133fa80 689701d0 adrp     x8, #0x462d000
0133fa84 083947f9 ldr      x8, [x8, #0xe70]
0133fa88 020140f9 ldr      x2, [x8]
0133fa8c e10315aa mov      x1, x21
0133fa90 25145394 bl       #0x2804b24
0133fa94 e20300aa mov      x2, x0
0133fa98 e00313aa mov      x0, x19
0133fa9c e10315aa mov      x1, x21
0133faa0 e30316aa mov      x3, x22
0133faa4 25010094 bl       #0x133ff38
0133faa8 f37b44a9 ldp      x19, x30, [sp, #0x40]
0133faac f55343a9 ldp      x21, x20, [sp, #0x30]
0133fab0 f75b42a9 ldp      x23, x22, [sp, #0x20]
0133fab4 f96341a9 ldp      x25, x24, [sp, #0x10]
0133fab8 ff430191 add      sp, sp, #0x50
0133fabc c0035fd6 ret      
0133fac0 4773e997 bl       #0xd9c7dc
0133fac4 4873e997 bl       #0xd9c7e4
0133fac8 4573e997 bl       #0xd9c7dc
0133facc 4473e997 bl       #0xd9c7dc
0133fad0 4373e997 bl       #0xd9c7dc
0133fad4 07000014 b        #0x133faf0
0133fad8 06000014 b        #0x133faf0
0133fadc 05000014 b        #0x133faf0
0133fae0 04000014 b        #0x133faf0
0133fae4 03000014 b        #0x133faf0
0133fae8 02000014 b        #0x133faf0
0133faec 01000014 b        #0x133faf0
0133faf0 f50300aa mov      x21, x0
0133faf4 3f040071 cmp      w1, #1
0133faf8 61080054 b.ne     #0x133fc04
0133fafc e00315aa mov      x0, x21
0133fb00 343cdc97 bl       #0xa4ebd0
0133fb04 f50300aa mov      x21, x0
0133fb08 e09601b0 adrp     x0, #0x461c000
0133fb0c 005847f9 ldr      x0, [x0, #0xeb0]
0133fb10 f872e997 bl       #0xd9c6f0
0133fb14 a80240f9 ldr      x8, [x21]
0133fb18 010140f9 ldr      x1, [x8]
0133fb1c f973e997 bl       #0xd9cb00
0133fb20 e0050036 tbz      w0, #0, #0x133fbdc
0133fb24 b50240f9 ldr      x21, [x21]
0133fb28 a63adc97 bl       #0xa4e5c0
0133fb2c e00313aa mov      x0, x19
0133fb30 e1031faa mov      x1, xzr
0133fb34 37000094 bl       #0x133fc10
0133fb38 150500b4 cbz      x21, #0x133fbd8
0133fb3c a80240f9 ldr      x8, [x21]
0133fb40 e00315aa mov      x0, x21
0133fb44 09855ca9 ldp      x9, x1, [x8, #0x1c8]
0133fb48 20013fd6 blr      x9
0133fb4c f30300aa mov      x19, x0
0133fb50 009401f0 adrp     x0, #0x45c2000
0133fb54 00ec42f9 ldr      x0, [x0, #0x5d8]
0133fb58 e672e997 bl       #0xd9c6f0
0133fb5c e10315aa mov      x1, x21
0133fb60 e20313aa mov      x2, x19
0133fb64 e3031faa mov      x3, xzr
0133fb68 aaf87094 bl       #0x2f7de10
0133fb6c f30300aa mov      x19, x0
0133fb70 80970190 adrp     x0, #0x462f000
0133fb74 003c46f9 ldr      x0, [x0, #0xc78]
0133fb78 de72e997 bl       #0xd9c6f0
0133fb7c 08e040b9 ldr      w8, [x0, #0xe0]
0133fb80 48000035 cbnz     w8, #0x133fb88
0133fb84 0d73e997 bl       #0xd9c7b8
0133fb88 e00313aa mov      x0, x19
0133fb8c e1031faa mov      x1, xzr
0133fb90 4c068394 bl       #0x34014c0
0133fb94 881e40b9 ldr      w8, [x20, #0x1c]
0133fb98 609601f0 adrp     x0, #0x460e000
0133fb9c e80f00b9 str      w8, [sp, #0xc]
0133fba0 000445f9 ldr      x0, [x0, #0xa08]
0133fba4 d372e997 bl       #0xd9c6f0
0133fba8 e1330091 add      x1, sp, #0xc
0133fbac 0573e997 bl       #0xd9c7c0
0133fbb0 f30300aa mov      x19, x0
0133fbb4 a09301b0 adrp     x0, #0x45b4000
0133fbb8 007044f9 ldr      x0, [x0, #0x8e0]
0133fbbc cd72e997 bl       #0xd9c6f0
0133fbc0 e10313aa mov      x1, x19
0133fbc4 e2031faa mov      x2, xzr
0133fbc8 6dd57094 bl       #0x2f7517c
0133fbcc e1031faa mov      x1, xzr
0133fbd0 3c068394 bl       #0x34014c0
0133fbd4 b5ffff17 b        #0x133faa8
0133fbd8 0173e997 bl       #0xd9c7dc
0133fbdc 00018052 mov      w0, #8
0133fbe0 7839dc97 bl       #0xa4e1c0
0133fbe4 a80240f9 ldr      x8, [x21]
0133fbe8 080000f9 str      x8, [x0]
0133fbec a18601f0 adrp     x1, #0x4416000
0133fbf0 21e02291 add      x1, x1, #0x8b8
0133fbf4 e2031faa mov      x2, xzr
0133fbf8 2e37dc97 bl       #0xa4d8b0
0133fbfc f50300aa mov      x21, x0
0133fc00 703adc97 bl       #0xa4e5c0
0133fc04 e00315aa mov      x0, x21
0133fc08 3e3adc97 bl       #0xa4e500
0133fc0c cb41dc97 bl       #0xa50338
