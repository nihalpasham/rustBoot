
```powershell 
Welcome to minicom 2.8

OPTIONS:
Compiled on Oct 25 2021, 04:59:49.
Port /dev/tty.usbserial-1101, 11:47:54

Press Meta-Z for help on special keys

[    0.000005] imx8mn-rs version 0.1.0
[    0.003584] Booting on: i.MX 8M Nano EVK
[    0.007607] Current privilege level: EL3
[    0.011545] Exception handling state:
[    0.015345]       Debug:  Masked
[    0.018591]       SError: Unmasked
[    0.022112]       IRQ:    Masked
[    0.025395]       FIQ:    Masked
[    0.028603] Drivers loaded:
[    0.031523]       1. i.MX8M Uart2
[    0.034965] Chars written: 382
[    0.037976] uSDHC2 supports 1.8v, 3.0v, 3.3v ...
[    0.042765] Sd clock stablized in 38us
[    0.046555] Prescaler = 64, Divisor = 16, Freq = 390625 Hz
[    0.052420] Sd: sending command, CMD_NAME: "GO_IDLE_STATE", CMD_CODE: 0x00000000, CMD_ARG: 0x00000000
[    0.065373] Sd: sending command, CMD_NAME: "SEND_IF_COND", CMD_CODE: 0x081a0000, CMD_ARG: 0x000001aa
[    0.076527] Sd: sending command, CMD_NAME: "APP_CMD", CMD_CODE: 0x371a0000, CMD_ARG: 0x00000000
[    0.087369] Sd: sending command, CMD_NAME: "APP_SEND_OP_COND", CMD_CODE: 0x29020000, CMD_ARG: 0x50ff8000
[    0.499191] Sd: sending command, CMD_NAME: "APP_CMD", CMD_CODE: 0x371a0000, CMD_ARG: 0x00000000
[    0.509984] Sd: sending command, CMD_NAME: "APP_SEND_OP_COND", CMD_CODE: 0x29020000, CMD_ARG: 0x50ff8000
[    0.521794] Sd: sending command, CMD_NAME: "ALL_SEND_CID", CMD_CODE: 0x02090000, CMD_ARG: 0x00000000
[    0.533471] Sd: sending command, CMD_NAME: "SEND_REL_ADDR", CMD_CODE: 0x03020000, CMD_ARG: 0x00000000
[    0.544576] Sd: sending command, CMD_NAME: "SEND_CSD", CMD_CODE: 0x09010000, CMD_ARG: 0x00010000
[    0.555735] CSD Contents : 00 40 0e 00 32 5b 59 00 003b 83 7f 80 0a 40 00
[    0.562789] cemmc_structure=1, spec_vers=0, taac=0x0E, nsac=0x00, tran_speed=0x32,ccc=0x05B5, read_bl_len=0x09, read_bl_partial=0b, write_blk_misalign=0b,read_blk_misalign=0b, dsr_imp=0b, sector_size =0x7F, erase_blk_en=1b
[    0.583485] CSD 2.0: ver2_c_size = 0x3BFF, card capacity: 7987527680 bytes or 7.99GiB
[    0.591571] wp_grp_size=0x0000000b, wp_grp_enable=0b, default_ecc=00b, r2w_factor=010b, write_bl_len=0x09, write_bl_partial=0b, file_format_grp=0, copy=0b, perm_write_protect=0b, tmp_write_protect=0b, file_format=0b ecc=00b
[    0.612260] Sd clock stablized in 0us
[    0.616093] Prescaler = 1, Divisor = 8, Freq = 50000000 Hz
[    0.621697] Sd: sending command, CMD_NAME: "CARD_SELECT", CMD_CODE: 0x07030000, CMD_ARG: 0x00010000
[    0.632286] Sd: sending command, CMD_NAME: "APP_CMD_RCA", CMD_CODE: 0x37020000, CMD_ARG: 0x00010000
[    0.642606] Sd: sending command, CMD_NAME: "SEND_SCR", CMD_CODE: 0x333a0000, CMD_ARG: 0x00000000
[    0.653013] SCR bus width: WIDTH_1_4
[    0.656649] Sd: sending command, CMD_NAME: "APP_CMD_RCA", CMD_CODE: 0x37020000, CMD_ARG: 0x00010000
[    0.667078] Sd: sending command, CMD_NAME: "SET_BUS_WIDTH", CMD_CODE: 0x06020000, CMD_ARG: 0x00010002
[    0.677648] Sd Bus width set to 4
[    0.680970] Sd: sending command, CMD_NAME: "SET_BLOCKLEN", CMD_CODE: 0x10000000, CMD_ARG: 0x00000200
[    0.691651] Sd Card: Type 2 HC, 7617Mb, mfr_id: 150, 'DE:SD', r2.0, mfr_date: 1/2021, serial: 0x424f0218, RCA: 0x0001
[    0.703020] uSDHC driver initialized
[    0.706679]

[    0.708156]  ... wait forever
```