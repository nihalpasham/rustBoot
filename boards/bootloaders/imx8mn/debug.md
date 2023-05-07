
```powershell 
Welcome to minicom 2.8

OPTIONS:
Compiled on Oct 25 2021, 04:59:49.
Port /dev/tty.usbserial-1101, 11:47:54

Press Meta-Z for help on special keys

[    0.000249] imx8mn-rs version 0.1.0
[    0.003708] Booting on: i.MX 8M Nano EVK
[    0.007704] Current privilege level: EL3
[    0.011687] Exception handling state:
[    0.015486]       Debug:  Masked
[    0.018770]       SError: Unmasked
[    0.022237]       IRQ:    Masked
[    0.025530]       FIQ:    Masked
[    0.028797] Drivers loaded:
[    0.031522]       1. i.MX8M Uart2
[    0.034975] Chars written: 382
[    0.038089] uSDHC2 has support for 1.8v, 3.0v, 3.3v ...
[    0.043339] Sd host circuit reset in 4us
[    0.047362] Sd clock stablized in 38us
[    0.051125] Prescaler = 64, Divisor = 16, Freq Set = 390625
[    0.057011] Sd: sending command, CMD_NAME: "GO_IDLE_STATE", CMD_CODE: 0x00000000, CMD_ARG: 0x00000000
[    0.069780] Sd: sending command, CMD_NAME: "SEND_IF_COND", CMD_CODE: 0x081a0000, CMD_ARG: 0x000001aa
[    0.084405] Error: we got a response for the last cmd but it contains errors, decode contents of interrupt status register for details
               VendSpec: 0x20007879, SysCtrl: 0x008f20ff, ProtCtrl: 0x08800020, PresentStatus: 0xf0058088, intStatus: 0x000c8001, Resp0: 0x00000000, Resp1: 0x00000000,
Resp2: 0x00000000, Resp3: 0x00000000, CC_bit set in 4084us

[    0.118136] SdError: Send interface condition command (CMD8) returned an error
[    0.125688] failed to initialize
[    0.128972]
[    0.130618]  ... wait forever
```