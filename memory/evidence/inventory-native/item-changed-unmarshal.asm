010d7fe4 f40f1ef8 str      x20, [sp, #-0x20]!
010d7fe8 f37b01a9 stp      x19, x30, [sp, #0x10]
010d7fec f30301aa mov      x19, x1
010d7ff0 f40300aa mov      x20, x0
010d7ff4 00e19152 mov      w0, #0x8f08
010d7ff8 e1031faa mov      x1, xzr
010d7ffc 90a23294 bl       #0x1d80a3c
010d8000 60010036 tbz      w0, #0, #0x10d802c
010d8004 00e19152 mov      w0, #0x8f08
010d8008 e1031faa mov      x1, xzr
010d800c b9a23294 bl       #0x1d80af0
010d8010 a00600b4 cbz      x0, #0x10d80e4
010d8014 e20313aa mov      x2, x19
010d8018 f37b41a9 ldp      x19, x30, [sp, #0x10]
010d801c e10314aa mov      x1, x20
010d8020 e3031faa mov      x3, xzr
010d8024 f40742f8 ldr      x20, [sp], #0x20
010d8028 ea3f2f14 b        #0x1ca7fd0
010d802c d30500b4 cbz      x19, #0x10d80e4
010d8030 610c8052 mov      w1, #0x63
010d8034 e00313aa mov      x0, x19
010d8038 e2031faa mov      x2, xzr
010d803c aa544394 bl       #0x21ad2e4
010d8040 801a00b9 str      w0, [x20, #0x18]
010d8044 a10d8052 mov      w1, #0x6d
010d8048 e00313aa mov      x0, x19
010d804c e2031faa mov      x2, xzr
010d8050 6c544394 bl       #0x21ad200
010d8054 08000012 and      w8, w0, #1
010d8058 810c8052 mov      w1, #0x64
010d805c e00313aa mov      x0, x19
010d8060 e2031faa mov      x2, xzr
010d8064 88720039 strb     w8, [x20, #0x1c]
010d8068 9f544394 bl       #0x21ad2e4
010d806c 802200b9 str      w0, [x20, #0x20]
010d8070 610e8052 mov      w1, #0x73
010d8074 e00313aa mov      x0, x19
010d8078 e2031faa mov      x2, xzr
010d807c 9a544394 bl       #0x21ad2e4
010d8080 802600b9 str      w0, [x20, #0x24]
010d8084 410a8052 mov      w1, #0x52
010d8088 e00313aa mov      x0, x19
010d808c e2031faa mov      x2, xzr
010d8090 95544394 bl       #0x21ad2e4
010d8094 802a00b9 str      w0, [x20, #0x28]
010d8098 e00313aa mov      x0, x19
010d809c e1031f2a mov      w1, wzr
010d80a0 e2031faa mov      x2, xzr
010d80a4 90544394 bl       #0x21ad2e4
010d80a8 802e00b9 str      w0, [x20, #0x2c]
010d80ac 81008052 mov      w1, #4
010d80b0 e00313aa mov      x0, x19
010d80b4 e2031faa mov      x2, xzr
010d80b8 45574394 bl       #0x21addcc
010d80bc 801a00f9 str      x0, [x20, #0x30]
010d80c0 a10d8052 mov      w1, #0x6d
010d80c4 e00313aa mov      x0, x19
010d80c8 e2031faa mov      x2, xzr
010d80cc 86544394 bl       #0x21ad2e4
010d80d0 803a00b9 str      w0, [x20, #0x38]
010d80d4 e00313aa mov      x0, x19
010d80d8 f37b41a9 ldp      x19, x30, [sp, #0x10]
010d80dc f40742f8 ldr      x20, [sp], #0x20
010d80e0 c0035fd6 ret      
010d80e4 be11f397 bl       #0xd9c7dc
