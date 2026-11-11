01e618c8 e80f1efc str      d8, [sp, #-0x20]!
01e618cc f40700f9 str      x20, [sp, #8]
01e618d0 f37b01a9 stp      x19, x30, [sp, #0x10]
01e618d4 f30300aa mov      x19, x0
01e618d8 40b28552 mov      w0, #0x2d92
01e618dc e1031faa mov      x1, xzr
01e618e0 577cfc97 bl       #0x1d80a3c
01e618e4 60010036 tbz      w0, #0, #0x1e61910
01e618e8 40b28552 mov      w0, #0x2d92
01e618ec e1031faa mov      x1, xzr
01e618f0 807cfc97 bl       #0x1d80af0
01e618f4 200400b4 cbz      x0, #0x1e61978
01e618f8 e10313aa mov      x1, x19
01e618fc f37b41a9 ldp      x19, x30, [sp, #0x10]
01e61900 f40740f9 ldr      x20, [sp, #8]
01e61904 e2031faa mov      x2, xzr
01e61908 e80742fc ldr      d8, [sp], #0x20
01e6190c fab9f817 b        #0x1c900f4
01e61910 e00313aa mov      x0, x19
01e61914 1a000094 bl       #0x1e6197c
01e61918 741e40f9 ldr      x20, [x19, #0x38]
01e6191c f40200b4 cbz      x20, #0x1e61978
01e61920 e00314aa mov      x0, x20
01e61924 e1031faa mov      x1, xzr
01e61928 22b9ff97 bl       #0x1e4fdb0
01e6192c e8bd00d0 adrp     x8, #0x361f000
01e61930 08e949bd ldr      s8, [x8, #0x9e8]
01e61934 e00314aa mov      x0, x20
01e61938 e1031faa mov      x1, xzr
01e6193c 011da84e mov      v1.16b, v8.16b
01e61940 28c1ff97 bl       #0x1e51de0
01e61944 731e40f9 ldr      x19, [x19, #0x38]
01e61948 930100b4 cbz      x19, #0x1e61978
01e6194c e00313aa mov      x0, x19
01e61950 e1031faa mov      x1, xzr
01e61954 17b9ff97 bl       #0x1e4fdb0
01e61958 e00313aa mov      x0, x19
01e6195c f37b41a9 ldp      x19, x30, [sp, #0x10]
01e61960 f40740f9 ldr      x20, [sp, #8]
01e61964 201ca14e mov      v0.16b, v1.16b
01e61968 011da84e mov      v1.16b, v8.16b
01e6196c e1031faa mov      x1, xzr
01e61970 e80742fc ldr      d8, [sp], #0x20
01e61974 7ac1ff17 b        #0x1e51f5c
01e61978 99ebbc97 bl       #0xd9c7dc
