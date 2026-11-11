01ecb10c 0824a852 mov      w8, #0x41200000
01ecb110 09f0a752 mov      w9, #0x3f800000
01ecb114 74ea0d39 strb     w20, [x19, #0x37a]
01ecb118 689603b9 str      w8, [x19, #0x394]
01ecb11c 699e03b9 str      w9, [x19, #0x39c]
01ecb120 96510190 adrp     x22, #0x48fb000
01ecb124 c8565339 ldrb     w8, [x22, #0x4d5]
01ecb128 75120f91 add      x21, x19, #0x3c4
01ecb12c a8000035 cbnz     w8, #0x1ecb140
01ecb130 003b01b0 adrp     x0, #0x462c000
01ecb134 00bc41f9 ldr      x0, [x0, #0x378]
01ecb138 6d45bb97 bl       #0xd9c6ec
01ecb13c d4561339 strb     w20, [x22, #0x4d5]
01ecb140 143b01b0 adrp     x20, #0x462c000
01ecb144 94be41f9 ldr      x20, [x20, #0x378]
01ecb148 0a008012 mov      w10, #-1
01ecb14c 965101d0 adrp     x22, #0x48fd000
01ecb150 880240f9 ldr      x8, [x20]
01ecb154 095d40f9 ldr      x9, [x8, #0xb8]
01ecb158 290140f9 ldr      x9, [x9]
01ecb15c 69d601f9 str      x9, [x19, #0x3a8]
01ecb160 095d40f9 ldr      x9, [x8, #0xb8]
01ecb164 290140f9 ldr      x9, [x9]
01ecb168 6abe03b9 str      w10, [x19, #0x3bc]
01ecb16c 69da01f9 str      x9, [x19, #0x3b0]
01ecb170 c9de4e39 ldrb     w9, [x22, #0x3b7]
01ecb174 c9000035 cbnz     w9, #0x1ecb18c
01ecb178 e00314aa mov      x0, x20
01ecb17c 5c45bb97 bl       #0xd9c6ec
01ecb180 880240f9 ldr      x8, [x20]
01ecb184 29008052 mov      w9, #1
01ecb188 c9de0e39 strb     w9, [x22, #0x3b7]
01ecb18c 085d40f9 ldr      x8, [x8, #0xb8]
01ecb190 34008052 mov      w20, #1
01ecb194 0900b852 mov      w9, #-0x40000000
01ecb198 0a008092 mov      x10, #-1
01ecb19c 080940f9 ldr      x8, [x8, #0x10]
01ecb1a0 96510190 adrp     x22, #0x48fb000
01ecb1a4 583a01d0 adrp     x24, #0x4615000
01ecb1a8 a80200f9 str      x8, [x21]
01ecb1ac 0800a852 mov      w8, #0x40000000
01ecb1b0 74221039 strb     w20, [x19, #0x408]
01ecb1b4 74c21039 strb     w20, [x19, #0x430]
01ecb1b8 694a04b9 str      w9, [x19, #0x448]
01ecb1bc 74621139 strb     w20, [x19, #0x458]
01ecb1c0 6a3602f9 str      x10, [x19, #0x468]
01ecb1c4 689a04b9 str      w8, [x19, #0x498]
01ecb1c8 6a5202f9 str      x10, [x19, #0x4a0]
01ecb1cc c8724f39 ldrb     w8, [x22, #0x3dc]
01ecb1d0 18a345f9 ldr      x24, [x24, #0xb40]
Capstone 5.0: decoded 50 instructions / 200 bytes
