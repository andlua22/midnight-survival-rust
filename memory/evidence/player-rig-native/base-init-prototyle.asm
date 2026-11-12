01e4cf98 ff4302d1 sub      sp, sp, #0x90
01e4cf9c f82b00f9 str      x24, [sp, #0x50]
01e4cfa0 f75b06a9 stp      x23, x22, [sp, #0x60]
01e4cfa4 f55307a9 stp      x21, x20, [sp, #0x70]
01e4cfa8 f37b08a9 stp      x19, x30, [sp, #0x80]
01e4cfac b45501b0 adrp     x20, #0x4901000
01e4cfb0 88be7339 ldrb     w8, [x20, #0xcef]
01e4cfb4 f30300aa mov      x19, x0
01e4cfb8 48050037 tbnz     w8, #0, #0x1e4d060
01e4cfbc 003c01d0 adrp     x0, #0x45ce000
01e4cfc0 002c46f9 ldr      x0, [x0, #0xc58]
01e4cfc4 ca3dbd97 bl       #0xd9c6ec
01e4cfc8 603b0190 adrp     x0, #0x45b8000
01e4cfcc 00ec46f9 ldr      x0, [x0, #0xdd8]
01e4cfd0 c73dbd97 bl       #0xd9c6ec
01e4cfd4 e03b01d0 adrp     x0, #0x45ca000
01e4cfd8 002040f9 ldr      x0, [x0, #0x40]
01e4cfdc c43dbd97 bl       #0xd9c6ec
01e4cfe0 e03c01d0 adrp     x0, #0x45ea000
01e4cfe4 001044f9 ldr      x0, [x0, #0x820]
01e4cfe8 c13dbd97 bl       #0xd9c6ec
01e4cfec 803d01f0 adrp     x0, #0x45ff000
01e4cff0 00ac45f9 ldr      x0, [x0, #0xb58]
01e4cff4 be3dbd97 bl       #0xd9c6ec
01e4cff8 003d0190 adrp     x0, #0x45ec000
01e4cffc 00d044f9 ldr      x0, [x0, #0x9a0]
01e4d000 bb3dbd97 bl       #0xd9c6ec
01e4d004 003d01d0 adrp     x0, #0x45ef000
01e4d008 008445f9 ldr      x0, [x0, #0xb08]
01e4d00c b83dbd97 bl       #0xd9c6ec
01e4d010 a03b01d0 adrp     x0, #0x45c3000
01e4d014 008043f9 ldr      x0, [x0, #0x700]
01e4d018 b53dbd97 bl       #0xd9c6ec
01e4d01c c03e01b0 adrp     x0, #0x4626000
01e4d020 006c46f9 ldr      x0, [x0, #0xcd8]
01e4d024 b23dbd97 bl       #0xd9c6ec
01e4d028 003b0190 adrp     x0, #0x45ad000
01e4d02c 00e046f9 ldr      x0, [x0, #0xdc0]
01e4d030 af3dbd97 bl       #0xd9c6ec
01e4d034 603b01d0 adrp     x0, #0x45bb000
01e4d038 00c447f9 ldr      x0, [x0, #0xf88]
01e4d03c ac3dbd97 bl       #0xd9c6ec
01e4d040 403d0190 adrp     x0, #0x45f5000
01e4d044 00c843f9 ldr      x0, [x0, #0x790]
01e4d048 a93dbd97 bl       #0xd9c6ec
01e4d04c 603f01f0 adrp     x0, #0x463c000
01e4d050 002446f9 ldr      x0, [x0, #0xc48]
01e4d054 a63dbd97 bl       #0xd9c6ec
01e4d058 28008052 mov      w8, #1
01e4d05c 88be3339 strb     w8, [x20, #0xcef]
01e4d060 207d8152 mov      w0, #0xbe9
01e4d064 e1031faa mov      x1, xzr
01e4d068 ff7f04a9 stp      xzr, xzr, [sp, #0x40]
01e4d06c ff1f00f9 str      xzr, [sp, #0x38]
01e4d070 73cefc97 bl       #0x1d80a3c
01e4d074 20010036 tbz      w0, #0, #0x1e4d098
01e4d078 207d8152 mov      w0, #0xbe9
01e4d07c e1031faa mov      x1, xzr
01e4d080 9ccefc97 bl       #0x1d80af0
01e4d084 001200b4 cbz      x0, #0x1e4d2c4
01e4d088 e10313aa mov      x1, x19
01e4d08c e2031faa mov      x2, xzr
01e4d090 190cf997 bl       #0x1c900f4
01e4d094 86000014 b        #0x1e4d2ac
01e4d098 543b01f0 adrp     x20, #0x45b8000
01e4d09c 94ee46f9 ldr      x20, [x20, #0xdd8]
01e4d0a0 e00313aa mov      x0, x19
01e4d0a4 e1031faa mov      x1, xzr
01e4d0a8 02755794 bl       #0x342a4b0
01e4d0ac 602200f9 str      x0, [x19, #0x40]
01e4d0b0 810240f9 ldr      x1, [x20]
01e4d0b4 e00313aa mov      x0, x19
01e4d0b8 c2cf1c94 bl       #0x2580fc0
01e4d0bc 602600f9 str      x0, [x19, #0x48]
01e4d0c0 e00313aa mov      x0, x19
01e4d0c4 e1031faa mov      x1, xzr
01e4d0c8 09755794 bl       #0x342a4ec
01e4d0cc c00f00b4 cbz      x0, #0x1e4d2c4
01e4d0d0 c83e01b0 adrp     x8, #0x4626000
01e4d0d4 086d46f9 ldr      x8, [x8, #0xcd8]
01e4d0d8 753b01d0 adrp     x21, #0x45bb000
01e4d0dc 010140f9 ldr      x1, [x8]
01e4d0e0 b5c647f9 ldr      x21, [x21, #0xf88]
01e4d0e4 67411d94 bl       #0x259d680
01e4d0e8 601601f9 str      x0, [x19, #0x228]
01e4d0ec f40300aa mov      x20, x0
01e4d0f0 a00240f9 ldr      x0, [x21]
01e4d0f4 08e040b9 ldr      w8, [x0, #0xe0]
01e4d0f8 48000035 cbnz     w8, #0x1e4d100
01e4d0fc af3dbd97 bl       #0xd9c7b8
01e4d100 e00314aa mov      x0, x20
01e4d104 e1031faa mov      x1, xzr
01e4d108 77635794 bl       #0x3425ee4
01e4d10c 40010037 tbnz     w0, #0, #0x1e4d134
01e4d110 e00313aa mov      x0, x19
01e4d114 e1031faa mov      x1, xzr
01e4d118 f5745794 bl       #0x342a4ec
01e4d11c 400d00b4 cbz      x0, #0x1e4d2c4
01e4d120 083d01d0 adrp     x8, #0x45ef000
01e4d124 088545f9 ldr      x8, [x8, #0xb08]
01e4d128 010140f9 ldr      x1, [x8]
01e4d12c 1a411d94 bl       #0x259d594
01e4d130 601601f9 str      x0, [x19, #0x228]
01e4d134 e00313aa mov      x0, x19
01e4d138 e1031faa mov      x1, xzr
01e4d13c ec745794 bl       #0x342a4ec
01e4d140 200c00b4 cbz      x0, #0x1e4d2c4
01e4d144 a83b01d0 adrp     x8, #0x45c3000
01e4d148 088143f9 ldr      x8, [x8, #0x700]
01e4d14c 010140f9 ldr      x1, [x8]
01e4d150 11411d94 bl       #0x259d594
01e4d154 01102e1e fmov     s1, #1.00000000
01e4d158 603600f9 str      x0, [x19, #0x68]
01e4d15c e0e30091 add      x0, sp, #0x38
01e4d160 e003271e fmov     s0, wzr
01e4d164 e203271e fmov     s2, wzr
01e4d168 231ca14e mov      v3.16b, v1.16b
01e4d16c 241ca14e mov      v4.16b, v1.16b
01e4d170 251ca14e mov      v5.16b, v1.16b
01e4d174 e1031faa mov      x1, xzr
01e4d178 e24f5794 bl       #0x3421100
01e4d17c 603640f9 ldr      x0, [x19, #0x68]
01e4d180 e083c33c ldur     q0, [sp, #0x38]
01e4d184 e82740f9 ldr      x8, [sp, #0x48]
01e4d188 e00b803d str      q0, [sp, #0x20]
01e4d18c e81b00f9 str      x8, [sp, #0x30]
01e4d190 a00900b4 cbz      x0, #0x1e4d2c4
01e4d194 e00bc03d ldr      q0, [sp, #0x20]
01e4d198 e81b40f9 ldr      x8, [sp, #0x30]
01e4d19c 543d0190 adrp     x20, #0x45f5000
01e4d1a0 753f01f0 adrp     x21, #0x463c000
01e4d1a4 94ca43f9 ldr      x20, [x20, #0x790]
01e4d1a8 b52646f9 ldr      x21, [x21, #0xc48]
01e4d1ac e1030091 mov      x1, sp
01e4d1b0 e2031faa mov      x2, xzr
01e4d1b4 e003803d str      q0, [sp]
01e4d1b8 e80b00f9 str      x8, [sp, #0x10]
01e4d1bc 77ea5694 bl       #0x3407b98
01e4d1c0 e00313aa mov      x0, x19
01e4d1c4 e1031faa mov      x1, xzr
01e4d1c8 c9745794 bl       #0x342a4ec
01e4d1cc 880240f9 ldr      x8, [x20]
01e4d1d0 f40300aa mov      x20, x0
01e4d1d4 09e140b9 ldr      w9, [x8, #0xe0]
01e4d1d8 69000035 cbnz     w9, #0x1e4d1e4
01e4d1dc e00308aa mov      x0, x8
01e4d1e0 763dbd97 bl       #0xd9c7b8
01e4d1e4 a00240f9 ldr      x0, [x21]
01e4d1e8 e10314aa mov      x1, x20
01e4d1ec e2031faa mov      x2, xzr
01e4d1f0 f82b0c94 bl       #0x21581d0
01e4d1f4 800600b4 cbz      x0, #0x1e4d2c4
01e4d1f8 083b0190 adrp     x8, #0x45ad000
01e4d1fc 08e146f9 ldr      x8, [x8, #0xdc0]
01e4d200 f43b01b0 adrp     x20, #0x45ca000
01e4d204 f53c01f0 adrp     x21, #0x45ec000
01e4d208 942240f9 ldr      x20, [x20, #0x40]
01e4d20c b5d244f9 ldr      x21, [x21, #0x9a0]
01e4d210 010140f9 ldr      x1, [x8]
01e4d214 163c01b0 adrp     x22, #0x45ce000
01e4d218 f73c01b0 adrp     x23, #0x45ea000
01e4d21c 983d01d0 adrp     x24, #0x45ff000
01e4d220 d62e46f9 ldr      x22, [x22, #0xc58]
01e4d224 f71244f9 ldr      x23, [x23, #0x820]
01e4d228 18af45f9 ldr      x24, [x24, #0xb58]
01e4d22c 15411d94 bl       #0x259d680
01e4d230 602e00f9 str      x0, [x19, #0x58]
01e4d234 810240f9 ldr      x1, [x20]
01e4d238 e00313aa mov      x0, x19
01e4d23c 61cf1c94 bl       #0x2580fc0
01e4d240 606e01f9 str      x0, [x19, #0x2d8]
01e4d244 a10240f9 ldr      x1, [x21]
01e4d248 e00313aa mov      x0, x19
01e4d24c 5dcf1c94 bl       #0x2580fc0
01e4d250 60a600f9 str      x0, [x19, #0x148]
01e4d254 c10240f9 ldr      x1, [x22]
01e4d258 e00313aa mov      x0, x19
01e4d25c 59cf1c94 bl       #0x2580fc0
01e4d260 603a01f9 str      x0, [x19, #0x270]
01e4d264 e10240f9 ldr      x1, [x23]
01e4d268 e00313aa mov      x0, x19
01e4d26c 55cf1c94 bl       #0x2580fc0
01e4d270 603601f9 str      x0, [x19, #0x268]
01e4d274 010340f9 ldr      x1, [x24]
01e4d278 e00313aa mov      x0, x19
01e4d27c 51cf1c94 bl       #0x2580fc0
01e4d280 602e01f9 str      x0, [x19, #0x258]
01e4d284 e00313aa mov      x0, x19
01e4d288 10000094 bl       #0x1e4d2c8
01e4d28c e00313aa mov      x0, x19
01e4d290 62000094 bl       #0x1e4d418
01e4d294 e00313aa mov      x0, x19
01e4d298 19020094 bl       #0x1e4dafc
01e4d29c e00313aa mov      x0, x19
01e4d2a0 9b030094 bl       #0x1e4e10c
01e4d2a4 e00313aa mov      x0, x19
01e4d2a8 0c040094 bl       #0x1e4e2d8
01e4d2ac f37b48a9 ldp      x19, x30, [sp, #0x80]
01e4d2b0 f55347a9 ldp      x21, x20, [sp, #0x70]
01e4d2b4 f75b46a9 ldp      x23, x22, [sp, #0x60]
01e4d2b8 f82b40f9 ldr      x24, [sp, #0x50]
01e4d2bc ff430291 add      sp, sp, #0x90
01e4d2c0 c0035fd6 ret      
01e4d2c4 463dbd97 bl       #0xd9c7dc
