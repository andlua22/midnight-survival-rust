0194867c e80f1afc str      d8, [sp, #-0x60]!
01948680 fb6b01a9 stp      x27, x26, [sp, #0x10]
01948684 f96302a9 stp      x25, x24, [sp, #0x20]
01948688 f75b03a9 stp      x23, x22, [sp, #0x30]
0194868c f55304a9 stp      x21, x20, [sp, #0x40]
01948690 f37b05a9 stp      x19, x30, [sp, #0x50]
01948694 b57d01f0 adrp     x21, #0x48ff000
01948698 a8225b39 ldrb     w8, [x21, #0x6c8]
0194869c f40301aa mov      x20, x1
019486a0 f30300aa mov      x19, x0
019486a4 88070037 tbnz     w8, #0, #0x1948794
019486a8 206501d0 adrp     x0, #0x45ee000
019486ac 009044f9 ldr      x0, [x0, #0x920]
019486b0 0f50d197 bl       #0xd9c6ec
019486b4 20650190 adrp     x0, #0x45ec000
019486b8 009441f9 ldr      x0, [x0, #0x328]
019486bc 0c50d197 bl       #0xd9c6ec
019486c0 c06601d0 adrp     x0, #0x4622000
019486c4 005846f9 ldr      x0, [x0, #0xcb0]
019486c8 0950d197 bl       #0xd9c6ec
019486cc 406501f0 adrp     x0, #0x45f3000
019486d0 00d040f9 ldr      x0, [x0, #0x1a0]
019486d4 0650d197 bl       #0xd9c6ec
019486d8 e06601d0 adrp     x0, #0x4626000
019486dc 005842f9 ldr      x0, [x0, #0x4b0]
019486e0 0350d197 bl       #0xd9c6ec
019486e4 606401f0 adrp     x0, #0x45d7000
019486e8 00f046f9 ldr      x0, [x0, #0xde0]
019486ec 0050d197 bl       #0xd9c6ec
019486f0 606401b0 adrp     x0, #0x45d5000
019486f4 006442f9 ldr      x0, [x0, #0x4c8]
019486f8 fd4fd197 bl       #0xd9c6ec
019486fc 406401b0 adrp     x0, #0x45d1000
01948700 007441f9 ldr      x0, [x0, #0x2e8]
01948704 fa4fd197 bl       #0xd9c6ec
01948708 00640190 adrp     x0, #0x45c8000
0194870c 00fc42f9 ldr      x0, [x0, #0x5f8]
01948710 f74fd197 bl       #0xd9c6ec
01948714 20660190 adrp     x0, #0x460c000
01948718 005c41f9 ldr      x0, [x0, #0x2b8]
0194871c f44fd197 bl       #0xd9c6ec
01948720 606501f0 adrp     x0, #0x45f7000
01948724 00c042f9 ldr      x0, [x0, #0x580]
01948728 f14fd197 bl       #0xd9c6ec
0194872c e06401f0 adrp     x0, #0x45e7000
01948730 00ac42f9 ldr      x0, [x0, #0x558]
01948734 ee4fd197 bl       #0xd9c6ec
01948738 206301f0 adrp     x0, #0x45af000
0194873c 00e047f9 ldr      x0, [x0, #0xfc0]
01948740 eb4fd197 bl       #0xd9c6ec
01948744 40630190 adrp     x0, #0x45b0000
01948748 008440f9 ldr      x0, [x0, #0x108]
0194874c e84fd197 bl       #0xd9c6ec
01948750 206401b0 adrp     x0, #0x45cd000
01948754 004c45f9 ldr      x0, [x0, #0xa98]
01948758 e54fd197 bl       #0xd9c6ec
0194875c c06401f0 adrp     x0, #0x45e3000
01948760 000842f9 ldr      x0, [x0, #0x410]
01948764 e24fd197 bl       #0xd9c6ec
01948768 e0630190 adrp     x0, #0x45c4000
0194876c 00f042f9 ldr      x0, [x0, #0x5e0]
01948770 df4fd197 bl       #0xd9c6ec
01948774 006601f0 adrp     x0, #0x460b000
01948778 000042f9 ldr      x0, [x0, #0x400]
0194877c dc4fd197 bl       #0xd9c6ec
01948780 606501b0 adrp     x0, #0x45f5000
01948784 00c843f9 ldr      x0, [x0, #0x790]
01948788 d94fd197 bl       #0xd9c6ec
0194878c 28008052 mov      w8, #1
01948790 a8221b39 strb     w8, [x21, #0x6c8]
01948794 e0cc8052 mov      w0, #0x667
01948798 e1031faa mov      x1, xzr
0194879c ff0f00b9 str      wzr, [sp, #0xc]
019487a0 a7e01094 bl       #0x1d80a3c
019487a4 40010036 tbz      w0, #0, #0x19487cc
019487a8 e0cc8052 mov      w0, #0x667
019487ac e1031faa mov      x1, xzr
019487b0 d0e01094 bl       #0x1d80af0
019487b4 e02900b4 cbz      x0, #0x1948cf0
019487b8 e10313aa mov      x1, x19
019487bc e20314aa mov      x2, x20
019487c0 e3031faa mov      x3, xzr
019487c4 8f1a0d94 bl       #0x1c8f200
019487c8 df000014 b        #0x1948b44
019487cc 342900b4 cbz      x20, #0x1948cf0
019487d0 605640f9 ldr      x0, [x19, #0xa8]
019487d4 e02800b4 cbz      x0, #0x1948cf0
019487d8 686401f0 adrp     x8, #0x45d7000
019487dc 08f146f9 ldr      x8, [x8, #0xde0]
019487e0 812680b9 ldrsw    x1, [x20, #0x24]
019487e4 020140f9 ldr      x2, [x8]
019487e8 34c83994 bl       #0x27ba8b8
019487ec 00010036 tbz      w0, #0, #0x194880c
019487f0 605640f9 ldr      x0, [x19, #0xa8]
019487f4 e02700b4 cbz      x0, #0x1948cf0
019487f8 486401b0 adrp     x8, #0x45d1000
019487fc 087541f9 ldr      x8, [x8, #0x2e8]
01948800 812680b9 ldrsw    x1, [x20, #0x24]
01948804 020140f9 ldr      x2, [x8]
01948808 d1cc3994 bl       #0x27bbb4c
0194880c 3a660190 adrp     x26, #0x460c000
01948810 776501f0 adrp     x23, #0x45f7000
01948814 08008012 mov      w8, #-1
01948818 5a5f41f9 ldr      x26, [x26, #0x2b8]
0194881c f7c242f9 ldr      x23, [x23, #0x580]
01948820 e80f00b9 str      w8, [sp, #0xc]
01948824 812640b9 ldr      w1, [x20, #0x24]
01948828 386501d0 adrp     x24, #0x45ee000
0194882c 796501b0 adrp     x25, #0x45f5000
01948830 766401b0 adrp     x22, #0x45d5000
01948834 189344f9 ldr      x24, [x24, #0x920]
01948838 39cb43f9 ldr      x25, [x25, #0x790]
0194883c d66642f9 ldr      x22, [x22, #0x4c8]
01948840 e2330091 add      x2, sp, #0xc
01948844 e00313aa mov      x0, x19
01948848 52010094 bl       #0x1948d90
0194884c 883240b9 ldr      w8, [x20, #0x30]
01948850 400500b4 cbz      x0, #0x19488f8
01948854 f50300aa mov      x21, x0
01948858 280c0034 cbz      w8, #0x19489dc
0194885c a81a00b9 str      w8, [x21, #0x18]
01948860 881e40f9 ldr      x8, [x20, #0x38]
01948864 e1031faa mov      x1, xzr
01948868 a81a00f9 str      x8, [x21, #0x30]
0194886c 803640b9 ldr      w0, [x20, #0x34]
01948870 c6462294 bl       #0x21da388
01948874 000200b4 cbz      x0, #0x19488b4
01948878 801e40f9 ldr      x0, [x20, #0x38]
0194887c a02300b4 cbz      x0, #0x1948cf0
01948880 e1031faa mov      x1, xzr
01948884 4f932194 bl       #0x21ad5c0
01948888 801e40f9 ldr      x0, [x20, #0x38]
0194888c 202300b4 cbz      x0, #0x1948cf0
01948890 e1031faa mov      x1, xzr
01948894 081ca04e mov      v8.16b, v0.16b
01948898 ee8f2194 bl       #0x21ac850
0194889c 606a40f9 ldr      x0, [x19, #0xd0]
019488a0 802200b4 cbz      x0, #0x1948cf0
019488a4 812640b9 ldr      w1, [x20, #0x24]
019488a8 420340f9 ldr      x2, [x26]
019488ac 001da84e mov      v0.16b, v8.16b
019488b0 a3b43894 bl       #0x2775b3c
019488b4 e80f40b9 ldr      w8, [sp, #0xc]
019488b8 1f050071 cmp      w8, #1
019488bc c10c0054 b.ne     #0x1948a54
019488c0 600a40f9 ldr      x0, [x19, #0x10]
019488c4 602100b4 cbz      x0, #0x1948cf0
019488c8 48630190 adrp     x8, #0x45b0000
019488cc 088540f9 ldr      x8, [x8, #0x108]
019488d0 e10315aa mov      x1, x21
019488d4 020140f9 ldr      x2, [x8]
019488d8 f84b4894 bl       #0x2b5b8b8
019488dc 600e40f9 ldr      x0, [x19, #0x18]
019488e0 802000b4 cbz      x0, #0x1948cf0
019488e4 811e40b9 ldr      w1, [x20, #0x1c]
019488e8 e30240f9 ldr      x3, [x23]
019488ec e20315aa mov      x2, x21
019488f0 e1a93894 bl       #0x2773074
019488f4 58000014 b        #0x1948a54
019488f8 48130034 cbz      w8, #0x1948b60
019488fc 28650190 adrp     x8, #0x45ec000
01948900 089541f9 ldr      x8, [x8, #0x328]
01948904 000140f9 ldr      x0, [x8]
01948908 b14fd197 bl       #0xd9c7cc
0194890c 201f00b4 cbz      x0, #0x1948cf0
01948910 1b6601f0 adrp     x27, #0x460b000
01948914 7b0342f9 ldr      x27, [x27, #0x400]
01948918 e1031faa mov      x1, xzr
0194891c f50300aa mov      x21, x0
01948920 e1c7dd97 bl       #0x10ba8a4
01948924 882640b9 ldr      w8, [x20, #0x24]
01948928 e0031faa mov      x0, xzr
0194892c a81200b9 str      w8, [x21, #0x10]
01948930 882e40b9 ldr      w8, [x20, #0x2c]
01948934 a81e00b9 str      w8, [x21, #0x1c]
01948938 801a40fd ldr      d0, [x20, #0x30]
0194893c 0008a00e rev64    v0.2s, v0.2s
01948940 a04201fc stur     d0, [x21, #0x14]
01948944 881e40f9 ldr      x8, [x20, #0x38]
01948948 a81a00f9 str      x8, [x21, #0x30]
0194894c 88824039 ldrb     w8, [x20, #0x20]
01948950 a8920039 strb     w8, [x21, #0x24]
01948954 882a40b9 ldr      w8, [x20, #0x28]
01948958 a82a00b9 str      w8, [x21, #0x28]
0194895c 961a40b9 ldr      w22, [x20, #0x18]
01948960 046c6b94 bl       #0x3423970
01948964 600340f9 ldr      x0, [x27]
01948968 081ca04e mov      v8.16b, v0.16b
0194896c 08e040b9 ldr      w8, [x0, #0xe0]
01948970 48000035 cbnz     w8, #0x1948978
01948974 914fd197 bl       #0xd9c7b8
01948978 08f0af52 mov      w8, #0x7f800000
0194897c 00c1241e frintp   s0, s8
01948980 0101271e fmov     s1, w8
01948984 0901281e fcvtps   w9, s8
01948988 0020211e fcmp     s0, s1
0194898c 0800b052 mov      w8, #-0x80000000
01948990 0801891a csel     w8, w8, w9, eq
01948994 0801160b add      w8, w8, w22
01948998 e00313aa mov      x0, x19
0194899c a82200b9 str      w8, [x21, #0x20]
019489a0 56e8ff97 bl       #0x1942af8
019489a4 601a00b4 cbz      x0, #0x1948cf0
019489a8 811e40b9 ldr      w1, [x20, #0x1c]
019489ac e30240f9 ldr      x3, [x23]
019489b0 e20315aa mov      x2, x21
019489b4 b0a93894 bl       #0x2773074
019489b8 813640b9 ldr      w1, [x20, #0x34]
019489bc e00313aa mov      x0, x19
019489c0 88010094 bl       #0x1948fe0
019489c4 40110036 tbz      w0, #0, #0x1948bec
019489c8 822640b9 ldr      w2, [x20, #0x24]
019489cc e00313aa mov      x0, x19
019489d0 e10315aa mov      x1, x21
019489d4 a0010094 bl       #0x1949054
019489d8 8c000014 b        #0x1948c08
019489dc 605640f9 ldr      x0, [x19, #0xa8]
019489e0 801800b4 cbz      x0, #0x1948cf0
019489e4 e86601d0 adrp     x8, #0x4626000
019489e8 085942f9 ldr      x8, [x8, #0x4b0]
019489ec a10a4229 ldp      w1, w2, [x21, #0x10]
019489f0 217c4093 sxtw     x1, w1
019489f4 030140f9 ldr      x3, [x8]
019489f8 30c73994 bl       #0x27ba6b8
019489fc a11240b9 ldr      w1, [x21, #0x10]
01948a00 e00313aa mov      x0, x19
01948a04 48020094 bl       #0x1949324
01948a08 813640b9 ldr      w1, [x20, #0x34]
01948a0c e00313aa mov      x0, x19
01948a10 74010094 bl       #0x1948fe0
01948a14 60010036 tbz      w0, #0, #0x1948a40
01948a18 e00313aa mov      x0, x19
01948a1c e10315aa mov      x1, x21
01948a20 e9020094 bl       #0x19495c4
01948a24 60f640f9 ldr      x0, [x19, #0x1e8]
01948a28 401600b4 cbz      x0, #0x1948cf0
01948a2c 08640190 adrp     x8, #0x45c8000
01948a30 08fd42f9 ldr      x8, [x8, #0x5f8]
01948a34 812680b9 ldrsw    x1, [x20, #0x24]
01948a38 020140f9 ldr      x2, [x8]
01948a3c aae13994 bl       #0x27c10e4
01948a40 606a40f9 ldr      x0, [x19, #0xd0]
01948a44 601500b4 cbz      x0, #0x1948cf0
01948a48 812640b9 ldr      w1, [x20, #0x24]
01948a4c c20240f9 ldr      x2, [x22]
01948a50 5cb93894 bl       #0x2776fc0
01948a54 88824039 ldrb     w8, [x20, #0x20]
01948a58 b6ee4229 ldp      w22, w27, [x21, #0x14]
01948a5c a8920039 strb     w8, [x21, #0x24]
01948a60 882a40b9 ldr      w8, [x20, #0x28]
01948a64 a82a00b9 str      w8, [x21, #0x28]
01948a68 080340f9 ldr      x8, [x24]
01948a6c 200340f9 ldr      x0, [x25]
01948a70 9a3240b9 ldr      w26, [x20, #0x30]
01948a74 085d40f9 ldr      x8, [x8, #0xb8]
01948a78 09e040b9 ldr      w9, [x0, #0xe0]
01948a7c 176940f9 ldr      x23, [x8, #0xd0]
01948a80 49000035 cbnz     w9, #0x1948a88
01948a84 4d4fd197 bl       #0xd9c7b8
01948a88 42031b4b sub      w2, w26, w27
01948a8c e00317aa mov      x0, x23
01948a90 e103162a mov      w1, w22
01948a94 e3031faa mov      x3, xzr
01948a98 0c1a2094 bl       #0x214f2c8
01948a9c 882e40b9 ldr      w8, [x20, #0x2c]
01948aa0 a81e00b9 str      w8, [x21, #0x1c]
01948aa4 486501f0 adrp     x8, #0x45f3000
01948aa8 08d140f9 ldr      x8, [x8, #0x1a0]
01948aac 740a40f9 ldr      x20, [x19, #0x10]
01948ab0 000140f9 ldr      x0, [x8]
01948ab4 464fd197 bl       #0xd9c7cc
01948ab8 c01100b4 cbz      x0, #0x1948cf0
01948abc c86601d0 adrp     x8, #0x4622000
01948ac0 085946f9 ldr      x8, [x8, #0xcb0]
01948ac4 e10313aa mov      x1, x19
01948ac8 e3031faa mov      x3, xzr
01948acc f50300aa mov      x21, x0
01948ad0 020140f9 ldr      x2, [x8]
01948ad4 a81d3794 bl       #0x2710174
01948ad8 d41000b4 cbz      x20, #0x1948cf0
01948adc 286401b0 adrp     x8, #0x45cd000
01948ae0 084d45f9 ldr      x8, [x8, #0xa98]
01948ae4 e00314aa mov      x0, x20
01948ae8 e10315aa mov      x1, x21
01948aec 020140f9 ldr      x2, [x8]
01948af0 084d4894 bl       #0x2b5bf10
01948af4 080340f9 ldr      x8, [x24]
01948af8 200340f9 ldr      x0, [x25]
01948afc 085d40f9 ldr      x8, [x8, #0xb8]
01948b00 09e040b9 ldr      w9, [x0, #0xe0]
01948b04 130140f9 ldr      x19, [x8]
01948b08 49000035 cbnz     w9, #0x1948b10
01948b0c 2b4fd197 bl       #0xd9c7b8
01948b10 e00313aa mov      x0, x19
01948b14 e1031faa mov      x1, xzr
01948b18 eb182094 bl       #0x214eec4
01948b1c 080340f9 ldr      x8, [x24]
01948b20 e1031faa mov      x1, xzr
01948b24 085d40f9 ldr      x8, [x8, #0xb8]
01948b28 000540f9 ldr      x0, [x8, #8]
01948b2c e6182094 bl       #0x214eec4
01948b30 080340f9 ldr      x8, [x24]
01948b34 e1031faa mov      x1, xzr
01948b38 085d40f9 ldr      x8, [x8, #0xb8]
01948b3c 001540f9 ldr      x0, [x8, #0x28]
01948b40 e1182094 bl       #0x214eec4
01948b44 f37b45a9 ldp      x19, x30, [sp, #0x50]
01948b48 f55344a9 ldp      x21, x20, [sp, #0x40]
01948b4c f75b43a9 ldp      x23, x22, [sp, #0x30]
01948b50 f96342a9 ldp      x25, x24, [sp, #0x20]
01948b54 fb6b41a9 ldp      x27, x26, [sp, #0x10]
01948b58 e80746fc ldr      d8, [sp], #0x60
01948b5c c0035fd6 ret      
01948b60 e00313aa mov      x0, x19
01948b64 5de9ff97 bl       #0x19430d8
01948b68 400c00b4 cbz      x0, #0x1948cf0
01948b6c e86401f0 adrp     x8, #0x45e7000
01948b70 08ad42f9 ldr      x8, [x8, #0x558]
01948b74 813640b9 ldr      w1, [x20, #0x34]
01948b78 020140f9 ldr      x2, [x8]
01948b7c c7224294 bl       #0x29d1698
01948b80 c00a0036 tbz      w0, #0, #0x1948cd8
01948b84 e00313aa mov      x0, x19
01948b88 30e9ff97 bl       #0x1943048
01948b8c 200b00b4 cbz      x0, #0x1948cf0
01948b90 1a1840b9 ldr      w26, [x0, #0x18]
01948b94 5f070071 cmp      w26, #1
01948b98 cb080054 b.lt     #0x1948cb0
01948b9c fb630190 adrp     x27, #0x45c4000
01948ba0 376301f0 adrp     x23, #0x45af000
01948ba4 7bf342f9 ldr      x27, [x27, #0x5e0]
01948ba8 f7e247f9 ldr      x23, [x23, #0xfc0]
01948bac f5031f2a mov      w21, wzr
01948bb0 e00313aa mov      x0, x19
01948bb4 25e9ff97 bl       #0x1943048
01948bb8 c00900b4 cbz      x0, #0x1948cf0
01948bbc 620340f9 ldr      x2, [x27]
01948bc0 e103152a mov      w1, w21
01948bc4 c6444894 bl       #0x2b59edc
01948bc8 400900b4 cbz      x0, #0x1948cf0
01948bcc 081040b9 ldr      w8, [x0, #0x10]
01948bd0 892640b9 ldr      w9, [x20, #0x24]
01948bd4 1f01096b cmp      w8, w9
01948bd8 00060054 b.eq     #0x1948c98
01948bdc b5060011 add      w21, w21, #1
01948be0 5f03156b cmp      w26, w21
01948be4 61feff54 b.ne     #0x1948bb0
01948be8 32000014 b        #0x1948cb0
01948bec 803640b9 ldr      w0, [x20, #0x34]
01948bf0 e1031faa mov      x1, xzr
01948bf4 60002394 bl       #0x2208d74
01948bf8 800000b4 cbz      x0, #0x1948c08
01948bfc e00313aa mov      x0, x19
01948c00 e10315aa mov      x1, x21
01948c04 6c010094 bl       #0x19491b4
01948c08 080340f9 ldr      x8, [x24]
01948c0c 200340f9 ldr      x0, [x25]
01948c10 b6d24229 ldp      w22, w20, [x21, #0x14]
01948c14 085d40f9 ldr      x8, [x8, #0xb8]
01948c18 09e040b9 ldr      w9, [x0, #0xe0]
01948c1c 176940f9 ldr      x23, [x8, #0xd0]
01948c20 49000035 cbnz     w9, #0x1948c28
01948c24 e54ed197 bl       #0xd9c7b8
01948c28 e00317aa mov      x0, x23
01948c2c e103162a mov      w1, w22
01948c30 e203142a mov      w2, w20
01948c34 e3031faa mov      x3, xzr
01948c38 a4192094 bl       #0x214f2c8
01948c3c a01640b9 ldr      w0, [x21, #0x14]
01948c40 e1031faa mov      x1, xzr
01948c44 d1452294 bl       #0x21da388
01948c48 e0f2ffb4 cbz      x0, #0x1948aa4
01948c4c a01a40f9 ldr      x0, [x21, #0x30]
01948c50 000500b4 cbz      x0, #0x1948cf0
01948c54 082440b9 ldr      w8, [x0, #0x24]
01948c58 1f110071 cmp      w8, #4
01948c5c 41f2ff54 b.ne     #0x1948aa4
01948c60 e1031faa mov      x1, xzr
01948c64 57922194 bl       #0x21ad5c0
01948c68 a01a40f9 ldr      x0, [x21, #0x30]
01948c6c 200400b4 cbz      x0, #0x1948cf0
01948c70 e1031faa mov      x1, xzr
01948c74 081ca04e mov      v8.16b, v0.16b
01948c78 f68e2194 bl       #0x21ac850
01948c7c 606a40f9 ldr      x0, [x19, #0xd0]
01948c80 800300b4 cbz      x0, #0x1948cf0
01948c84 a11240b9 ldr      w1, [x21, #0x10]
01948c88 420340f9 ldr      x2, [x26]
01948c8c 001da84e mov      v0.16b, v8.16b
01948c90 abb33894 bl       #0x2775b3c
01948c94 84ffff17 b        #0x1948aa4
01948c98 e00313aa mov      x0, x19
01948c9c ebe8ff97 bl       #0x1943048
01948ca0 800200b4 cbz      x0, #0x1948cf0
01948ca4 e20240f9 ldr      x2, [x23]
01948ca8 e103152a mov      w1, w21
01948cac b54b4894 bl       #0x2b5bb80
01948cb0 080340f9 ldr      x8, [x24]
01948cb4 200340f9 ldr      x0, [x25]
01948cb8 085d40f9 ldr      x8, [x8, #0xb8]
01948cbc 09e040b9 ldr      w9, [x0, #0xe0]
01948cc0 152140f9 ldr      x21, [x8, #0x40]
01948cc4 49000035 cbnz     w9, #0x1948ccc
01948cc8 bc4ed197 bl       #0xd9c7b8
01948ccc e00315aa mov      x0, x21
01948cd0 e1031faa mov      x1, xzr
01948cd4 7c182094 bl       #0x214eec4
01948cd8 606a40f9 ldr      x0, [x19, #0xd0]
01948cdc a00000b4 cbz      x0, #0x1948cf0
01948ce0 812640b9 ldr      w1, [x20, #0x24]
01948ce4 c20240f9 ldr      x2, [x22]
01948ce8 b6b83894 bl       #0x2776fc0
01948cec 96ffff17 b        #0x1948b44
01948cf0 bb4ed197 bl       #0xd9c7dc
01948cf4 01000014 b        #0x1948cf8
01948cf8 f50300aa mov      x21, x0
01948cfc 3f040071 cmp      w1, #1
01948d00 21040054 b.ne     #0x1948d84
01948d04 e00315aa mov      x0, x21
01948d08 b217c497 bl       #0xa4ebd0
01948d0c f50300aa mov      x21, x0
01948d10 a0660190 adrp     x0, #0x461c000
01948d14 005847f9 ldr      x0, [x0, #0xeb0]
01948d18 764ed197 bl       #0xd9c6f0
01948d1c a80240f9 ldr      x8, [x21]
01948d20 010140f9 ldr      x1, [x8]
01948d24 774fd197 bl       #0xd9cb00
01948d28 a0010036 tbz      w0, #0, #0x1948d5c
01948d2c b50240f9 ldr      x21, [x21]
01948d30 2416c497 bl       #0xa4e5c0
01948d34 206701f0 adrp     x0, #0x462f000
01948d38 003c46f9 ldr      x0, [x0, #0xc78]
01948d3c 6d4ed197 bl       #0xd9c6f0
01948d40 08e040b9 ldr      w8, [x0, #0xe0]
01948d44 48000035 cbnz     w8, #0x1948d4c
01948d48 9c4ed197 bl       #0xd9c7b8
01948d4c e00315aa mov      x0, x21
01948d50 e1031faa mov      x1, xzr
01948d54 dbe16a94 bl       #0x34014c0
01948d58 e0ffff17 b        #0x1948cd8
01948d5c 00018052 mov      w0, #8
01948d60 1815c497 bl       #0xa4e1c0
01948d64 a80240f9 ldr      x8, [x21]
01948d68 080000f9 str      x8, [x0]
01948d6c 615601d0 adrp     x1, #0x4416000
01948d70 21e02291 add      x1, x1, #0x8b8
01948d74 e2031faa mov      x2, xzr
01948d78 ce12c497 bl       #0xa4d8b0
01948d7c f50300aa mov      x21, x0
01948d80 1016c497 bl       #0xa4e5c0
01948d84 e00315aa mov      x0, x21
01948d88 de15c497 bl       #0xa4e500
01948d8c 6b1dc497 bl       #0xa50338
