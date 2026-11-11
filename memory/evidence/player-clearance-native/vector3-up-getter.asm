03430580 f40f1ef8 str      x20, [sp, #-0x20]!
03430584 f37b01a9 stp      x19, x30, [sp, #0x10]
03430588 d3a600f0 adrp     x19, #0x490b000
0343058c f48f00d0 adrp     x20, #0x462e000
03430590 689a4839 ldrb     w8, [x19, #0x226]
03430594 947e41f9 ldr      x20, [x20, #0x2f8]
03430598 c8000037 tbnz     w8, #0, #0x34305b0
0343059c e08f00d0 adrp     x0, #0x462e000
034305a0 007c41f9 ldr      x0, [x0, #0x2f8]
034305a4 52b06597 bl       #0xd9c6ec
034305a8 28008052 mov      w8, #1
034305ac 689a0839 strb     w8, [x19, #0x226]
034305b0 880240f9 ldr      x8, [x20]
034305b4 f37b41a9 ldp      x19, x30, [sp, #0x10]
034305b8 085d40f9 ldr      x8, [x8, #0xb8]
034305bc 0005432d ldp      s0, s1, [x8, #0x18]
034305c0 022140bd ldr      s2, [x8, #0x20]
034305c4 f40742f8 ldr      x20, [sp], #0x20
034305c8 c0035fd6 ret      
Capstone 5.0: decoded 19 instructions / 76 bytes
