01ec35d0 ff8301d1 sub      sp, sp, #0x60
01ec35d4 ea1b00fd str      d10, [sp, #0x30]
01ec35d8 e923046d stp      d9, d8, [sp, #0x40]
01ec35dc f37b05a9 stp      x19, x30, [sp, #0x50]
01ec35e0 f30300aa mov      x19, x0
01ec35e4 00e4006f movi     v0.2d, #0000000000000000
01ec35e8 e0978152 mov      w0, #0xcbf
01ec35ec e1031faa mov      x1, xzr
01ec35f0 e0c3813c stur     q0, [sp, #0x1c]
01ec35f4 e00300ad stp      q0, q0, [sp]
01ec35f8 11f5fa97 bl       #0x1d80a3c
01ec35fc 20010036 tbz      w0, #0, #0x1ec3620
01ec3600 e0978152 mov      w0, #0xcbf
01ec3604 e1031faa mov      x1, xzr
01ec3608 3af5fa97 bl       #0x1d80af0
01ec360c a00500b4 cbz      x0, #0x1ec36c0
01ec3610 e10313aa mov      x1, x19
01ec3614 e2031faa mov      x2, xzr
01ec3618 592ef797 bl       #0x1c8ef7c
01ec361c 23000014 b        #0x1ec36a8
01ec3620 604e40f9 ldr      x0, [x19, #0x98]
01ec3624 e00400b4 cbz      x0, #0x1ec36c0
01ec3628 e1031faa mov      x1, xzr
01ec362c fad45594 bl       #0x3438a14
01ec3630 d3510190 adrp     x19, #0x48fb000
01ec3634 68764f39 ldrb     w8, [x19, #0x3dd]
01ec3638 081ca04e mov      v8.16b, v0.16b
01ec363c 291ca14e mov      v9.16b, v1.16b
01ec3640 4a1ca24e mov      v10.16b, v2.16b
01ec3644 c8000035 cbnz     w8, #0x1ec365c
01ec3648 403b01f0 adrp     x0, #0x462e000
01ec364c 007c41f9 ldr      x0, [x0, #0x2f8]
01ec3650 2764bb97 bl       #0xd9c6ec
01ec3654 28008052 mov      w8, #1
01ec3658 68760f39 strb     w8, [x19, #0x3dd]
01ec365c 483b01f0 adrp     x8, #0x462e000
01ec3660 087d41f9 ldr      x8, [x8, #0x2f8]
01ec3664 e9ba00f0 adrp     x9, #0x3622000
01ec3668 27cd4cbd ldr      s7, [x9, #0xccc]
01ec366c 21008052 mov      w1, #1
01ec3670 080140f9 ldr      x8, [x8]
01ec3674 e0030091 mov      x0, sp
01ec3678 0101a872 movk     w1, #0x4008, lsl #16
01ec367c 001da84e mov      v0.16b, v8.16b
01ec3680 085d40f9 ldr      x8, [x8, #0xb8]
01ec3684 211da94e mov      v1.16b, v9.16b
01ec3688 421daa4e mov      v2.16b, v10.16b
01ec368c e2031faa mov      x2, xzr
01ec3690 0415432d ldp      s4, s5, [x8, #0x18]
01ec3694 062140bd ldr      s6, [x8, #0x20]
01ec3698 e8ba0090 adrp     x8, #0x361f000
01ec369c 038545bd ldr      s3, [x8, #0x584]
01ec36a0 86d45694 bl       #0x34788b8
01ec36a4 00000052 eor      w0, w0, #1
01ec36a8 f37b45a9 ldp      x19, x30, [sp, #0x50]
01ec36ac e923446d ldp      d9, d8, [sp, #0x40]
01ec36b0 ea1b40fd ldr      d10, [sp, #0x30]
01ec36b4 00000012 and      w0, w0, #1
01ec36b8 ff830191 add      sp, sp, #0x60
01ec36bc c0035fd6 ret      
01ec36c0 4764bb97 bl       #0xd9c7dc
Capstone 5.0: decoded 61 instructions / 244 bytes
