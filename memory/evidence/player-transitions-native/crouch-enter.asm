01e612d4 f80f1cf8 str      x24, [sp, #-0x40]!
01e612d8 f75b01a9 stp      x23, x22, [sp, #0x10]
01e612dc f55302a9 stp      x21, x20, [sp, #0x20]
01e612e0 f37b03a9 stp      x19, x30, [sp, #0x30]
01e612e4 15550190 adrp     x21, #0x4901000
01e612e8 a8327639 ldrb     w8, [x21, #0xd8c]
01e612ec f40301aa mov      x20, x1
01e612f0 f30300aa mov      x19, x0
01e612f4 28040037 tbnz     w8, #0, #0x1e61378
01e612f8 203e01b0 adrp     x0, #0x4626000
01e612fc 00e445f9 ldr      x0, [x0, #0xbc8]
01e61300 fbecbc97 bl       #0xd9c6ec
01e61304 803c01d0 adrp     x0, #0x45f3000
01e61308 008440f9 ldr      x0, [x0, #0x108]
01e6130c f8ecbc97 bl       #0xd9c6ec
01e61310 a03e01b0 adrp     x0, #0x4636000
01e61314 006444f9 ldr      x0, [x0, #0x8c8]
01e61318 f5ecbc97 bl       #0xd9c6ec
01e6131c e03d01d0 adrp     x0, #0x461f000
01e61320 005c47f9 ldr      x0, [x0, #0xeb8]
01e61324 f2ecbc97 bl       #0xd9c6ec
01e61328 c03c01f0 adrp     x0, #0x45fc000
01e6132c 002446f9 ldr      x0, [x0, #0xc48]
01e61330 efecbc97 bl       #0xd9c6ec
01e61334 603c01b0 adrp     x0, #0x45ee000
01e61338 00e445f9 ldr      x0, [x0, #0xbc8]
01e6133c ececbc97 bl       #0xd9c6ec
01e61340 003d01f0 adrp     x0, #0x4604000
01e61344 00ec46f9 ldr      x0, [x0, #0xdd8]
01e61348 e9ecbc97 bl       #0xd9c6ec
01e6134c a03c0190 adrp     x0, #0x45f5000
01e61350 00c843f9 ldr      x0, [x0, #0x790]
01e61354 e6ecbc97 bl       #0xd9c6ec
01e61358 c03c01f0 adrp     x0, #0x45fc000
01e6135c 002043f9 ldr      x0, [x0, #0x640]
01e61360 e3ecbc97 bl       #0xd9c6ec
01e61364 403d01d0 adrp     x0, #0x460b000
01e61368 007c44f9 ldr      x0, [x0, #0x8f8]
01e6136c e0ecbc97 bl       #0xd9c6ec
01e61370 28008052 mov      w8, #1
01e61374 a8323639 strb     w8, [x21, #0xd8c]
01e61378 80b18552 mov      w0, #0x2d8c
01e6137c e1031faa mov      x1, xzr
01e61380 af7dfc97 bl       #0x1d80a3c
01e61384 a0010036 tbz      w0, #0, #0x1e613b8
01e61388 80b18552 mov      w0, #0x2d8c
01e6138c e1031faa mov      x1, xzr
01e61390 d87dfc97 bl       #0x1d80af0
01e61394 401000b4 cbz      x0, #0x1e6159c
01e61398 e10313aa mov      x1, x19
01e6139c e20314aa mov      x2, x20
01e613a0 f37b43a9 ldp      x19, x30, [sp, #0x30]
01e613a4 f55342a9 ldp      x21, x20, [sp, #0x20]
01e613a8 f75b41a9 ldp      x23, x22, [sp, #0x10]
01e613ac e3031faa mov      x3, xzr
01e613b0 f80744f8 ldr      x24, [sp], #0x40
01e613b4 93b7f817 b        #0x1c8f200
01e613b8 973c01d0 adrp     x23, #0x45f3000
01e613bc f78640f9 ldr      x23, [x23, #0x108]
01e613c0 a93c0190 adrp     x9, #0x45f5000
01e613c4 e80240f9 ldr      x8, [x23]
01e613c8 29c943f9 ldr      x9, [x9, #0x790]
01e613cc 085d40f9 ldr      x8, [x8, #0xb8]
01e613d0 200140f9 ldr      x0, [x9]
01e613d4 15c141f9 ldr      x21, [x8, #0x380]
01e613d8 09e040b9 ldr      w9, [x0, #0xe0]
01e613dc 49000035 cbnz     w9, #0x1e613e4
01e613e0 f6ecbc97 bl       #0xd9c7b8
01e613e4 e00315aa mov      x0, x21
01e613e8 e1031faa mov      x1, xzr
01e613ec b6b60b94 bl       #0x214eec4
01e613f0 681e40f9 ldr      x8, [x19, #0x38]
01e613f4 480d00b4 cbz      x8, #0x1e6159c
01e613f8 c93c01f0 adrp     x9, #0x45fc000
01e613fc 292546f9 ldr      x9, [x9, #0xc48]
01e61400 2a008052 mov      w10, #1
01e61404 0a610e39 strb     w10, [x8, #0x398]
01e61408 200140f9 ldr      x0, [x9]
01e6140c f0ecbc97 bl       #0xd9c7cc
01e61410 600c00b4 cbz      x0, #0x1e6159c
01e61414 a83e01b0 adrp     x8, #0x4636000
01e61418 086544f9 ldr      x8, [x8, #0x8c8]
01e6141c 363e01b0 adrp     x22, #0x4626000
01e61420 e10313aa mov      x1, x19
01e61424 e3031faa mov      x3, xzr
01e61428 020140f9 ldr      x2, [x8]
01e6142c d6e645f9 ldr      x22, [x22, #0xbc8]
01e61430 f50300aa mov      x21, x0
01e61434 f9c10a94 bl       #0x2111c18
01e61438 d60240f9 ldr      x22, [x22]
01e6143c c81e40f9 ldr      x8, [x22, #0x38]
01e61440 000140f9 ldr      x0, [x8]
01e61444 08c84439 ldrb     w8, [x0, #0x132]
01e61448 48000037 tbnz     w8, #0, #0x1e61450
01e6144c e2fbbd97 bl       #0xde03d4
01e61450 08e040b9 ldr      w8, [x0, #0xe0]
01e61454 48000035 cbnz     w8, #0x1e6145c
01e61458 d8ecbc97 bl       #0xd9c7b8
01e6145c c81e40f9 ldr      x8, [x22, #0x38]
01e61460 763c01b0 adrp     x22, #0x45ee000
01e61464 000140f9 ldr      x0, [x8]
01e61468 08c84439 ldrb     w8, [x0, #0x132]
01e6146c d6e645f9 ldr      x22, [x22, #0xbc8]
01e61470 48000037 tbnz     w8, #0, #0x1e61478
01e61474 d8fbbd97 bl       #0xde03d4
01e61478 c80240f9 ldr      x8, [x22]
01e6147c 095c40f9 ldr      x9, [x0, #0xb8]
01e61480 583d01d0 adrp     x24, #0x460b000
01e61484 0ae140b9 ldr      w10, [x8, #0xe0]
01e61488 360140f9 ldr      x22, [x9]
01e6148c 187f44f9 ldr      x24, [x24, #0x8f8]
01e61490 6a000035 cbnz     w10, #0x1e6149c
01e61494 e00308aa mov      x0, x8
01e61498 c8ecbc97 bl       #0xd9c7b8
01e6149c 000340f9 ldr      x0, [x24]
01e614a0 00102c1e fmov     s0, #0.50000000
01e614a4 e10315aa mov      x1, x21
01e614a8 e20316aa mov      x2, x22
01e614ac e3031faa mov      x3, xzr
01e614b0 25c20a94 bl       #0x2111d44
01e614b4 681e40f9 ldr      x8, [x19, #0x38]
01e614b8 280700b4 cbz      x8, #0x1e6159c
01e614bc 00b941f9 ldr      x0, [x8, #0x370]
01e614c0 e00600b4 cbz      x0, #0x1e6159c
01e614c4 d53c01f0 adrp     x21, #0x45fc000
01e614c8 b52243f9 ldr      x21, [x21, #0x640]
01e614cc 22008052 mov      w2, #1
01e614d0 e3031faa mov      x3, xzr
01e614d4 a10240f9 ldr      x1, [x21]
01e614d8 4c35e297 bl       #0x16eea08
01e614dc 601e40f9 ldr      x0, [x19, #0x38]
01e614e0 e00500b4 cbz      x0, #0x1e6159c
01e614e4 01b042b9 ldr      w1, [x0, #0x2b0]
01e614e8 a20240f9 ldr      x2, [x21]
01e614ec f53d01d0 adrp     x21, #0x461f000
01e614f0 b55e47f9 ldr      x21, [x21, #0xeb8]
01e614f4 00102a1e fmov     s0, #0.25000000
01e614f8 e3031f2a mov      w3, wzr
01e614fc e4031faa mov      x4, xzr
01e61500 08bfff97 bl       #0x1e51120
01e61504 a00240f9 ldr      x0, [x21]
01e61508 751e40f9 ldr      x21, [x19, #0x38]
01e6150c 08e040b9 ldr      w8, [x0, #0xe0]
01e61510 48000035 cbnz     w8, #0x1e61518
01e61514 a9ecbc97 bl       #0xd9c7b8
01e61518 e00315aa mov      x0, x21
01e6151c 23000094 bl       #0x1e615a8
01e61520 e80240f9 ldr      x8, [x23]
01e61524 e1031faa mov      x1, xzr
01e61528 085d40f9 ldr      x8, [x8, #0xb8]
01e6152c 007940f9 ldr      x0, [x8, #0xf0]
01e61530 65b60b94 bl       #0x214eec4
01e61534 e00313aa mov      x0, x19
01e61538 87000094 bl       #0x1e61754
01e6153c 7f420139 strb     wzr, [x19, #0x50]
01e61540 7f5600b9 str      wzr, [x19, #0x54]
01e61544 d40200b4 cbz      x20, #0x1e6159c
01e61548 880e40f9 ldr      x8, [x20, #0x18]
01e6154c e80100b4 cbz      x8, #0x1e61588
01e61550 88020034 cbz      w8, #0x1e615a0
01e61554 801240f9 ldr      x0, [x20, #0x20]
01e61558 200200b4 cbz      x0, #0x1e6159c
01e6155c 083d01f0 adrp     x8, #0x4604000
01e61560 08ed46f9 ldr      x8, [x8, #0xdd8]
01e61564 090040f9 ldr      x9, [x0]
01e61568 010140f9 ldr      x1, [x8]
01e6156c 282140f9 ldr      x8, [x9, #0x40]
01e61570 292040f9 ldr      x9, [x1, #0x40]
01e61574 1f0109eb cmp      x8, x9
01e61578 61010054 b.ne     #0x1e615a4
01e6157c 92ecbc97 bl       #0xd9c7c4
01e61580 080040b9 ldr      w8, [x0]
01e61584 685600b9 str      w8, [x19, #0x54]
01e61588 f37b43a9 ldp      x19, x30, [sp, #0x30]
01e6158c f55342a9 ldp      x21, x20, [sp, #0x20]
01e61590 f75b41a9 ldp      x23, x22, [sp, #0x10]
01e61594 f80744f8 ldr      x24, [sp], #0x40
01e61598 c0035fd6 ret      
01e6159c 90ecbc97 bl       #0xd9c7dc
01e615a0 91ecbc97 bl       #0xd9c7e4
01e615a4 3dedbc97 bl       #0xd9ca98
