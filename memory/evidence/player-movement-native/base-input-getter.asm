01e4fdb0 f37bbfa9 stp      x19, x30, [sp, #-0x10]!
01e4fdb4 f30300aa mov      x19, x0
01e4fdb8 80a48552 mov      w0, #0x2d24
01e4fdbc e1031faa mov      x1, xzr
01e4fdc0 1fc3fc97 bl       #0x1d80a3c
01e4fdc4 20010036 tbz      w0, #0, #0x1e4fde8
01e4fdc8 80a48552 mov      w0, #0x2d24
01e4fdcc e1031faa mov      x1, xzr
01e4fdd0 48c3fc97 bl       #0x1d80af0
01e4fdd4 200100b4 cbz      x0, #0x1e4fdf8
01e4fdd8 e10313aa mov      x1, x19
01e4fddc e2031faa mov      x2, xzr
01e4fde0 0985f997 bl       #0x1cb1204
01e4fde4 03000014 b        #0x1e4fdf0
01e4fde8 60a642bd ldr      s0, [x19, #0x2a4]
01e4fdec 61aa42bd ldr      s1, [x19, #0x2a8]
01e4fdf0 f37bc1a8 ldp      x19, x30, [sp], #0x10
01e4fdf4 c0035fd6 ret      
01e4fdf8 7932bd97 bl       #0xd9c7dc
Capstone 5.0: decoded 19 instructions / 76 bytes
