; SItems: selected unpatched branch and return path from one 0x4c-byte method
010d90b0 20010036 tbz      w0, #0, #0x10d90d4
010d90d4 80878152 mov      w0, #0xc3c
010d90d8 0072a072 movk     w0, #0x390, lsl #16
010d90dc f37bc1a8 ldp      x19, x30, [sp], #0x10
010d90e0 c0035fd6 ret

; SItemChanged: selected unpatched branch and return path from one 0x4c-byte method
010d7eac 20010036 tbz      w0, #0, #0x10d7ed0
010d7ed0 607b8152 mov      w0, #0xbdb
010d7ed4 0072a072 movk     w0, #0x390, lsl #16
010d7ed8 f37bc1a8 ldp      x19, x30, [sp], #0x10
010d7edc c0035fd6 ret

; SItemNumberChange: selected unpatched branch and return path from one 0x4c-byte method
010d8a78 20010036 tbz      w0, #0, #0x10d8a9c
010d8a9c 00788152 mov      w0, #0xbc0
010d8aa0 0072a072 movk     w0, #0x390, lsl #16
010d8aa4 f37bc1a8 ldp      x19, x30, [sp], #0x10
010d8aa8 c0035fd6 ret

; SItemDurationChanged: selected unpatched branch and return path from one 0x4c-byte method
010d8594 20010036 tbz      w0, #0, #0x10d85b8
010d85b8 40788152 mov      w0, #0xbc2
010d85bc 0072a072 movk     w0, #0x390, lsl #16
010d85c0 f37bc1a8 ldp      x19, x30, [sp], #0x10
010d85c4 c0035fd6 ret

; SQuickUseItemChanged: selected unpatched branch and return path from one 0x4c-byte method
010e183c 20010036 tbz      w0, #0, #0x10e1860
010e1860 007c8152 mov      w0, #0xbe0
010e1864 0072a072 movk     w0, #0x390, lsl #16
010e1868 f37bc1a8 ldp      x19, x30, [sp], #0x10
010e186c c0035fd6 ret

