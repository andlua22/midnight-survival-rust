01eb3344 ff8301d1 sub      sp, sp, #0x60
01eb3348 ea1b00fd str      d10, [sp, #0x30]
01eb334c e923046d stp      d9, d8, [sp, #0x40]
01eb3350 f37b05a9 stp      x19, x30, [sp, #0x50]
01eb3354 f30300aa mov      x19, x0
01eb3358 00e4006f movi     v0.2d, #0000000000000000
01eb335c 00988152 mov      w0, #0xcc0
01eb3360 e1031faa mov      x1, xzr
01eb3364 e0c3813c stur     q0, [sp, #0x1c]
01eb3368 e00300ad stp      q0, q0, [sp]
01eb336c b435fb97 bl       #0x1d80a3c
01eb3370 20010036 tbz      w0, #0, #0x1eb3394
01eb3374 00988152 mov      w0, #0xcc0
01eb3378 e1031faa mov      x1, xzr
01eb337c dd35fb97 bl       #0x1d80af0
01eb3380 800500b4 cbz      x0, #0x1eb3430
01eb3384 e10313aa mov      x1, x19
01eb3388 e2031faa mov      x2, xzr
01eb338c fc6ef797 bl       #0x1c8ef7c
01eb3390 22000014 b        #0x1eb3418
01eb3394 604e40f9 ldr      x0, [x19, #0x98]
01eb3398 c00400b4 cbz      x0, #0x1eb3430
01eb339c e1031faa mov      x1, xzr
01eb33a0 9d155694 bl       #0x3438a14
01eb33a4 53520190 adrp     x19, #0x48fb000
01eb33a8 68764f39 ldrb     w8, [x19, #0x3dd]
01eb33ac 081ca04e mov      v8.16b, v0.16b
01eb33b0 291ca14e mov      v9.16b, v1.16b
01eb33b4 4a1ca24e mov      v10.16b, v2.16b
01eb33b8 c8000035 cbnz     w8, #0x1eb33d0
01eb33bc c03b01f0 adrp     x0, #0x462e000
01eb33c0 007c41f9 ldr      x0, [x0, #0x2f8]
01eb33c4 caa4bb97 bl       #0xd9c6ec
01eb33c8 28008052 mov      w8, #1
01eb33cc 68760f39 strb     w8, [x19, #0x3dd]
01eb33d0 c83b01f0 adrp     x8, #0x462e000
01eb33d4 087d41f9 ldr      x8, [x8, #0x2f8]
01eb33d8 21008052 mov      w1, #1
01eb33dc 07102f1e fmov     s7, #1.50000000
01eb33e0 e0030091 mov      x0, sp
01eb33e4 080140f9 ldr      x8, [x8]
01eb33e8 0101a872 movk     w1, #0x4008, lsl #16
01eb33ec 001da84e mov      v0.16b, v8.16b
01eb33f0 211da94e mov      v1.16b, v9.16b
01eb33f4 085d40f9 ldr      x8, [x8, #0xb8]
01eb33f8 421daa4e mov      v2.16b, v10.16b
01eb33fc e2031faa mov      x2, xzr
01eb3400 0415432d ldp      s4, s5, [x8, #0x18]
01eb3404 062140bd ldr      s6, [x8, #0x20]
01eb3408 68bb0090 adrp     x8, #0x361f000
01eb340c 038545bd ldr      s3, [x8, #0x584]
01eb3410 2a155794 bl       #0x34788b8
01eb3414 00000052 eor      w0, w0, #1
01eb3418 f37b45a9 ldp      x19, x30, [sp, #0x50]
01eb341c e923446d ldp      d9, d8, [sp, #0x40]
01eb3420 ea1b40fd ldr      d10, [sp, #0x30]
01eb3424 00000012 and      w0, w0, #1
01eb3428 ff830191 add      sp, sp, #0x60
01eb342c c0035fd6 ret      
01eb3430 eba4bb97 bl       #0xd9c7dc
Capstone 5.0: decoded 60 instructions / 240 bytes
