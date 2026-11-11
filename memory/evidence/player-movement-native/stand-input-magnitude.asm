01e7b818 0009281e fmul     s0, s8, s8
01e7b81c 2109291e fmul     s1, s9, s9
01e7b820 28bd00f0 adrp     x8, #0x3622000
01e7b824 0028211e fadd     s0, s0, s1
01e7b828 01b546fd ldr      d1, [x8, #0xd68]
01e7b82c 771e40f9 ldr      x23, [x19, #0x38]
01e7b830 00c0211e fsqrt    s0, s0
01e7b834 00c0221e fcvt     d0, s0
01e7b838 0020611e fcmp     d0, d1
Capstone 5.0: decoded 9 instructions / 36 bytes
