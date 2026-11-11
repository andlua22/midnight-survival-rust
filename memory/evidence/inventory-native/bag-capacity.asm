01942bd0 f40f1ef8 str      x20, [sp, #-0x20]!
01942bd4 f37b01a9 stp      x19, x30, [sp, #0x10]
01942bd8 f47d01b0 adrp     x20, #0x48ff000
01942bdc 88f65a39 ldrb     w8, [x20, #0x6bd]
01942be0 f30300aa mov      x19, x0
01942be4 c8000037 tbnz     w8, #0, #0x1942bfc
01942be8 006501b0 adrp     x0, #0x45e3000
01942bec 000842f9 ldr      x0, [x0, #0x410]
01942bf0 bf66d197 bl       #0xd9c6ec
01942bf4 28008052 mov      w8, #1
01942bf8 88f61a39 strb     w8, [x20, #0x6bd]
01942bfc 20c08052 mov      w0, #0x601
01942c00 e1031faa mov      x1, xzr
01942c04 8ef71094 bl       #0x1d80a3c
01942c08 40010036 tbz      w0, #0, #0x1942c30
01942c0c 20c08052 mov      w0, #0x601
01942c10 e1031faa mov      x1, xzr
01942c14 b7f71094 bl       #0x1d80af0
01942c18 c00200b4 cbz      x0, #0x1942c70
01942c1c e10313aa mov      x1, x19
01942c20 f37b41a9 ldp      x19, x30, [sp, #0x10]
01942c24 e2031faa mov      x2, xzr
01942c28 f40742f8 ldr      x20, [sp], #0x20
01942c2c a3310d14 b        #0x1c8f2b8
01942c30 744a40b9 ldr      w20, [x19, #0x48]
01942c34 e00313aa mov      x0, x19
01942c38 9effff97 bl       #0x1942ab0
01942c3c a00100b4 cbz      x0, #0x1942c70
01942c40 081840b9 ldr      w8, [x0, #0x18]
01942c44 9f02086b cmp      w20, w8
01942c48 6d000054 b.le     #0x1942c54
01942c4c 604a40b9 ldr      w0, [x19, #0x48]
01942c50 05000014 b        #0x1942c64
01942c54 e00313aa mov      x0, x19
01942c58 96ffff97 bl       #0x1942ab0
01942c5c a00000b4 cbz      x0, #0x1942c70
01942c60 001840b9 ldr      w0, [x0, #0x18]
01942c64 f37b41a9 ldp      x19, x30, [sp, #0x10]
01942c68 f40742f8 ldr      x20, [sp], #0x20
01942c6c c0035fd6 ret      
01942c70 db66d197 bl       #0xd9c7dc
