01ec332c ff0303d1 sub      sp, sp, #0xc0
01ec3330 ef3b066d stp      d15, d14, [sp, #0x60]
01ec3334 ed33076d stp      d13, d12, [sp, #0x70]
01ec3338 eb2b086d stp      d11, d10, [sp, #0x80]
01ec333c e923096d stp      d9, d8, [sp, #0x90]
01ec3340 f5530aa9 stp      x21, x20, [sp, #0xa0]
01ec3344 f37b0ba9 stp      x19, x30, [sp, #0xb0]
01ec3348 f45101f0 adrp     x20, #0x4902000
01ec334c 88664039 ldrb     w8, [x20, #0x19]
01ec3350 f30300aa mov      x19, x0
01ec3354 88010037 tbnz     w8, #0, #0x1ec3384
01ec3358 203b01b0 adrp     x0, #0x4628000
01ec335c 00e841f9 ldr      x0, [x0, #0x3d0]
01ec3360 e364bb97 bl       #0xd9c6ec
01ec3364 803801f0 adrp     x0, #0x45d6000
01ec3368 00f841f9 ldr      x0, [x0, #0x3f0]
01ec336c e064bb97 bl       #0xd9c6ec
01ec3370 c0370190 adrp     x0, #0x45bb000
01ec3374 00c447f9 ldr      x0, [x0, #0xf88]
01ec3378 dd64bb97 bl       #0xd9c6ec
01ec337c 28008052 mov      w8, #1
01ec3380 88660039 strb     w8, [x20, #0x19]
01ec3384 00e4006f movi     v0.2d, #0000000000000000
01ec3388 009e8152 mov      w0, #0xcf0
01ec338c e1031faa mov      x1, xzr
01ec3390 e0c3843c stur     q0, [sp, #0x4c]
01ec3394 e08301ad stp      q0, q0, [sp, #0x30]
01ec3398 e0c3813c stur     q0, [sp, #0x1c]
01ec339c e00300ad stp      q0, q0, [sp]
01ec33a0 a7f5fa97 bl       #0x1d80a3c
01ec33a4 20010036 tbz      w0, #0, #0x1ec33c8
01ec33a8 009e8152 mov      w0, #0xcf0
01ec33ac e1031faa mov      x1, xzr
01ec33b0 d0f5fa97 bl       #0x1d80af0
01ec33b4 c01000b4 cbz      x0, #0x1ec35cc
01ec33b8 e10313aa mov      x1, x19
01ec33bc e2031faa mov      x2, xzr
01ec33c0 ef2ef797 bl       #0x1c8ef7c
01ec33c4 79000014 b        #0x1ec35a8
01ec33c8 604240f9 ldr      x0, [x19, #0x80]
01ec33cc 001000b4 cbz      x0, #0x1ec35cc
01ec33d0 e1031faa mov      x1, xzr
01ec33d4 90d55594 bl       #0x3438a14
01ec33d8 d5510190 adrp     x21, #0x48fb000
01ec33dc a8764f39 ldrb     w8, [x21, #0x3dd]
01ec33e0 081ca04e mov      v8.16b, v0.16b
01ec33e4 291ca14e mov      v9.16b, v1.16b
01ec33e8 4a1ca24e mov      v10.16b, v2.16b
01ec33ec c8000035 cbnz     w8, #0x1ec3404
01ec33f0 403b01f0 adrp     x0, #0x462e000
01ec33f4 007c41f9 ldr      x0, [x0, #0x2f8]
01ec33f8 bd64bb97 bl       #0xd9c6ec
01ec33fc 28008052 mov      w8, #1
01ec3400 a8760f39 strb     w8, [x21, #0x3dd]
01ec3404 543b01f0 adrp     x20, #0x462e000
01ec3408 947e41f9 ldr      x20, [x20, #0x2f8]
01ec340c e9ba0090 adrp     x9, #0x361f000
01ec3410 272d4abd ldr      s7, [x9, #0xa2c]
01ec3414 21008052 mov      w1, #1
01ec3418 880240f9 ldr      x8, [x20]
01ec341c e0c30091 add      x0, sp, #0x30
01ec3420 0101a872 movk     w1, #0x4008, lsl #16
01ec3424 001da84e mov      v0.16b, v8.16b
01ec3428 085d40f9 ldr      x8, [x8, #0xb8]
01ec342c 211da94e mov      v1.16b, v9.16b
01ec3430 421daa4e mov      v2.16b, v10.16b
01ec3434 e2031faa mov      x2, xzr
01ec3438 0415432d ldp      s4, s5, [x8, #0x18]
01ec343c 062140bd ldr      s6, [x8, #0x20]
01ec3440 e8ba0090 adrp     x8, #0x361f000
01ec3444 038145bd ldr      s3, [x8, #0x580]
01ec3448 1cd55694 bl       #0x34788b8
01ec344c 60000036 tbz      w0, #0, #0x1ec3458
01ec3450 e0031f2a mov      w0, wzr
01ec3454 55000014 b        #0x1ec35a8
01ec3458 602240f9 ldr      x0, [x19, #0x40]
01ec345c 800b00b4 cbz      x0, #0x1ec35cc
01ec3460 e1031faa mov      x1, xzr
01ec3464 6cd55594 bl       #0x3438a14
01ec3468 a8764f39 ldrb     w8, [x21, #0x3dd]
01ec346c 081ca04e mov      v8.16b, v0.16b
01ec3470 291ca14e mov      v9.16b, v1.16b
01ec3474 08010035 cbnz     w8, #0x1ec3494
01ec3478 403b01f0 adrp     x0, #0x462e000
01ec347c 007c41f9 ldr      x0, [x0, #0x2f8]
01ec3480 4a1ca24e mov      v10.16b, v2.16b
01ec3484 9a64bb97 bl       #0xd9c6ec
01ec3488 421daa4e mov      v2.16b, v10.16b
01ec348c 28008052 mov      w8, #1
01ec3490 a8760f39 strb     w8, [x21, #0x3dd]
01ec3494 880240f9 ldr      x8, [x20]
01ec3498 e9ba0090 adrp     x9, #0x361f000
01ec349c d5510190 adrp     x21, #0x48fb000
01ec34a0 208d49bd ldr      s0, [x9, #0x98c]
01ec34a4 085d40f9 ldr      x8, [x8, #0xb8]
01ec34a8 333b01b0 adrp     x19, #0x4628000
01ec34ac a94e5339 ldrb     w9, [x21, #0x4d3]
01ec34b0 0111432d ldp      s1, s4, [x8, #0x18]
01ec34b4 032140bd ldr      s3, [x8, #0x20]
01ec34b8 73ea41f9 ldr      x19, [x19, #0x3d0]
01ec34bc 2e08201e fmul     s14, s1, s0
01ec34c0 8f08201e fmul     s15, s4, s0
01ec34c4 6a08201e fmul     s10, s3, s0
01ec34c8 29010035 cbnz     w9, #0x1ec34ec
01ec34cc e00314aa mov      x0, x20
01ec34d0 4b1ca24e mov      v11.16b, v2.16b
01ec34d4 8664bb97 bl       #0xd9c6ec
01ec34d8 880240f9 ldr      x8, [x20]
01ec34dc 29008052 mov      w9, #1
01ec34e0 a94e1339 strb     w9, [x21, #0x4d3]
01ec34e4 621dab4e mov      v2.16b, v11.16b
01ec34e8 085d40f9 ldr      x8, [x8, #0xb8]
01ec34ec 600240f9 ldr      x0, [x19]
01ec34f0 0cad442d ldp      s12, s11, [x8, #0x24]
01ec34f4 0d2d40bd ldr      s13, [x8, #0x2c]
01ec34f8 08292e1e fadd     s8, s8, s14
01ec34fc 09e040b9 ldr      w9, [x0, #0xe0]
01ec3500 29292f1e fadd     s9, s9, s15
01ec3504 4a282a1e fadd     s10, s2, s10
01ec3508 69000035 cbnz     w9, #0x1ec3514
01ec350c ab64bb97 bl       #0xd9c7b8
01ec3510 600240f9 ldr      x0, [x19]
01ec3514 085c40f9 ldr      x8, [x0, #0xb8]
01ec3518 29008052 mov      w9, #1
01ec351c 06102e1e fmov     s6, #1.00000000
01ec3520 e0030091 mov      x0, sp
01ec3524 082940b9 ldr      w8, [x8, #0x28]
01ec3528 001da84e mov      v0.16b, v8.16b
01ec352c 211da94e mov      v1.16b, v9.16b
01ec3530 421daa4e mov      v2.16b, v10.16b
01ec3534 2121c81a lsl      w1, w9, w8
01ec3538 831dac4e mov      v3.16b, v12.16b
01ec353c 641dab4e mov      v4.16b, v11.16b
01ec3540 a51dad4e mov      v5.16b, v13.16b
01ec3544 e2031faa mov      x2, xzr
01ec3548 86d15694 bl       #0x3477b60
01ec354c c0020036 tbz      w0, #0, #0x1ec35a4
01ec3550 e0030091 mov      x0, sp
01ec3554 e1031faa mov      x1, xzr
01ec3558 55dd5694 bl       #0x347aaac
01ec355c 800300b4 cbz      x0, #0x1ec35cc
01ec3560 883801f0 adrp     x8, #0x45d6000
01ec3564 08f941f9 ldr      x8, [x8, #0x3f0]
01ec3568 d3370190 adrp     x19, #0x45bb000
01ec356c 010140f9 ldr      x1, [x8]
01ec3570 73c647f9 ldr      x19, [x19, #0xf88]
01ec3574 1ef71a94 bl       #0x25811ec
01ec3578 680240f9 ldr      x8, [x19]
01ec357c f30300aa mov      x19, x0
01ec3580 09e140b9 ldr      w9, [x8, #0xe0]
01ec3584 69000035 cbnz     w9, #0x1ec3590
01ec3588 e00308aa mov      x0, x8
01ec358c 8b64bb97 bl       #0xd9c7b8
01ec3590 e00313aa mov      x0, x19
01ec3594 e1031faa mov      x1, xzr
01ec3598 e2031faa mov      x2, xzr
01ec359c 488e5594 bl       #0x3426ebc
01ec35a0 80f50737 tbnz     w0, #0, #0x1ec3450
01ec35a4 20008052 mov      w0, #1
01ec35a8 f37b4ba9 ldp      x19, x30, [sp, #0xb0]
01ec35ac f5534aa9 ldp      x21, x20, [sp, #0xa0]
01ec35b0 e923496d ldp      d9, d8, [sp, #0x90]
01ec35b4 eb2b486d ldp      d11, d10, [sp, #0x80]
01ec35b8 ed33476d ldp      d13, d12, [sp, #0x70]
01ec35bc ef3b466d ldp      d15, d14, [sp, #0x60]
01ec35c0 00000012 and      w0, w0, #1
01ec35c4 ff030391 add      sp, sp, #0xc0
01ec35c8 c0035fd6 ret      
01ec35cc 8464bb97 bl       #0xd9c7dc
Capstone 5.0: decoded 169 instructions / 676 bytes
