021205e4 e923be6d stp      d9, d8, [sp, #-0x20]!
021205e8 fe0b00f9 str      x30, [sp, #0x10]
021205ec 40968152 mov      w0, #0xcb2
021205f0 e1031faa mov      x1, xzr
021205f4 281ca14e mov      v8.16b, v1.16b
021205f8 091ca04e mov      v9.16b, v0.16b
021205fc 1081f197 bl       #0x1d80a3c
02120600 60010036 tbz      w0, #0, #0x212062c
02120604 40968152 mov      w0, #0xcb2
02120608 e1031faa mov      x1, xzr
0212060c 3981f197 bl       #0x1d80af0
02120610 a00200b4 cbz      x0, #0x2120664
02120614 fe0b40f9 ldr      x30, [sp, #0x10]
02120618 201da94e mov      v0.16b, v9.16b
0212061c 011da84e mov      v1.16b, v8.16b
02120620 e1031faa mov      x1, xzr
02120624 e923c26c ldp      d9, d8, [sp], #0x20
02120628 f349ee17 b        #0x1cb2df4
0212062c 001da84e mov      v0.16b, v8.16b
02120630 211da94e mov      v1.16b, v9.16b
02120634 03b9a497 bl       #0xa4ea40
02120638 08a800f0 adrp     x8, #0x3623000
0212063c 013548bd ldr      s1, [x8, #0x834]
02120640 0008211e fmul     s0, s0, s1
02120644 0820201e fcmp     s0, #0.0
02120648 85000054 b.pl     #0x2120658
0212064c 8876a852 mov      w8, #0x43b40000
02120650 0101271e fmov     s1, w8
02120654 0028211e fadd     s0, s0, s1
02120658 fe0b40f9 ldr      x30, [sp, #0x10]
0212065c e923c26c ldp      d9, d8, [sp], #0x20
02120660 c0035fd6 ret      
02120664 5ef0b197 bl       #0xd9c7dc
02120668 e923be6d stp      d9, d8, [sp, #-0x20]!
0212066c fe0b00f9 str      x30, [sp, #0x10]
02120670 20968152 mov      w0, #0xcb1
02120674 e1031faa mov      x1, xzr
02120678 281ca14e mov      v8.16b, v1.16b
0212067c 091ca04e mov      v9.16b, v0.16b
02120680 ef80f197 bl       #0x1d80a3c
02120684 60010036 tbz      w0, #0, #0x21206b0
02120688 20968152 mov      w0, #0xcb1
0212068c e1031faa mov      x1, xzr
02120690 1881f197 bl       #0x1d80af0
02120694 800100b4 cbz      x0, #0x21206c4
02120698 fe0b40f9 ldr      x30, [sp, #0x10]
0212069c 201da94e mov      v0.16b, v9.16b
021206a0 011da84e mov      v1.16b, v8.16b
021206a4 e1031faa mov      x1, xzr
021206a8 e923c26c ldp      d9, d8, [sp], #0x20
021206ac 5159ee17 b        #0x1cb6bf0
021206b0 fe0b40f9 ldr      x30, [sp, #0x10]
021206b4 001da84e mov      v0.16b, v8.16b
021206b8 211da94e mov      v1.16b, v9.16b
021206bc e923c26c ldp      d9, d8, [sp], #0x20
021206c0 c9ffff17 b        #0x21205e4
021206c4 46f0b197 bl       #0xd9c7dc
Capstone 5.0: decoded 57 instructions / 228 bytes
