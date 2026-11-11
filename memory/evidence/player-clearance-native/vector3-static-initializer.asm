034309d8 f40f1ef8 str      x20, [sp, #-0x20]!
034309dc f37b01a9 stp      x19, x30, [sp, #0x10]
034309e0 d4a600f0 adrp     x20, #0x490b000
034309e4 f38f00d0 adrp     x19, #0x462e000
034309e8 88e24839 ldrb     w8, [x20, #0x238]
034309ec 737e41f9 ldr      x19, [x19, #0x2f8]
034309f0 c8000037 tbnz     w8, #0, #0x3430a08
034309f4 e08f00d0 adrp     x0, #0x462e000
034309f8 007c41f9 ldr      x0, [x0, #0x2f8]
034309fc 3caf6597 bl       #0xd9c6ec
03430a00 28008052 mov      w8, #1
03430a04 88e20839 strb     w8, [x20, #0x238]
03430a08 680240f9 ldr      x8, [x19]
03430a0c 00e4002f movi     d0, #0000000000000000
03430a10 01f6030f fmov     v1.2s, #1.00000000
03430a14 0af0a752 mov      w10, #0x3f800000
03430a18 085d40f9 ldr      x8, [x8, #0xb8]
03430a1c 6b0f00f0 adrp     x11, #0x361f000
03430a20 000100fd str      d0, [x8]
03430a24 1f0900b9 str      wzr, [x8, #8]
03430a28 680240f9 ldr      x8, [x19]
03430a2c 095d40f9 ldr      x9, [x8, #0xb8]
03430a30 21c100fc stur     d1, [x9, #0xc]
03430a34 2a1500b9 str      w10, [x9, #0x14]
03430a38 61d543fd ldr      d1, [x11, #0x7a8]
03430a3c 095d40f9 ldr      x9, [x8, #0xb8]
03430a40 2b1600f0 adrp     x11, #0x36f7000
03430a44 210d00fd str      d1, [x9, #0x18]
03430a48 3f2100b9 str      wzr, [x9, #0x20]
03430a4c 611d42fd ldr      d1, [x11, #0x438]
03430a50 095d40f9 ldr      x9, [x8, #0xb8]
03430a54 8b0f00b0 adrp     x11, #0x3621000
03430a58 214102fc stur     d1, [x9, #0x24]
03430a5c 3f2d00b9 str      wzr, [x9, #0x2c]
03430a60 612143fd ldr      d1, [x11, #0x640]
03430a64 095d40f9 ldr      x9, [x8, #0xb8]
03430a68 6b0f00f0 adrp     x11, #0x361f000
03430a6c 211900fd str      d1, [x9, #0x30]
03430a70 3f3900b9 str      wzr, [x9, #0x38]
03430a74 61d143fd ldr      d1, [x11, #0x7a0]
03430a78 095d40f9 ldr      x9, [x8, #0xb8]
03430a7c 21c103fc stur     d1, [x9, #0x3c]
03430a80 3f4500b9 str      wzr, [x9, #0x44]
03430a84 095d40f9 ldr      x9, [x8, #0xb8]
03430a88 202500fd str      d0, [x9, #0x48]
03430a8c 2a5100b9 str      w10, [x9, #0x50]
03430a90 095d40f9 ldr      x9, [x8, #0xb8]
03430a94 0af0b752 mov      w10, #-0x40800000
03430a98 204105fc stur     d0, [x9, #0x54]
03430a9c 2a5d00b9 str      w10, [x9, #0x5c]
03430aa0 095d40f9 ldr      x9, [x8, #0xb8]
03430aa4 0af0af52 mov      w10, #0x7f800000
03430aa8 400d040e dup      v0.2s, w10
03430aac 203100fd str      d0, [x9, #0x60]
03430ab0 2a6900b9 str      w10, [x9, #0x68]
03430ab4 085d40f9 ldr      x8, [x8, #0xb8]
03430ab8 e0d7032f mvni     v0.2s, #0x7f, msl #16
03430abc 09f0bf52 mov      w9, #-0x800000
03430ac0 00c106fc stur     d0, [x8, #0x6c]
03430ac4 097500b9 str      w9, [x8, #0x74]
03430ac8 f37b41a9 ldp      x19, x30, [sp, #0x10]
03430acc f40742f8 ldr      x20, [sp], #0x20
03430ad0 c0035fd6 ret      
Capstone 5.0: decoded 63 instructions / 252 bytes
