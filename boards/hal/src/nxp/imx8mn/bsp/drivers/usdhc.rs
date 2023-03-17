use tock_registers::{register_bitfields, register_structs};

register_bitfields! {
    u32,

    /// This register contains the physical system memory address used for DMA transfers.
    DS_ADDR [
        /// DMA system address / argument 2
        DS_ADDR OFFSET(2) NUMBITS(30) [],
        /// Reserved - the least 2 bits are reserved, always 0
        RESERVED OFFSET(0) NUMBITS(2) [],
    ],
    /// This register is used to configure the number of data blocks and the number of bytes in
    /// each block
    BLK_ATT [
        /// Transfer block size
        ///
        /// This register specifies the block size for block data transfers. Values ranging from 1 byte up to the
        /// maximum buffer size can be set. It can be accessed only when no transaction is executing (that is, after a
        /// transaction has stopped). Read operations during transfers may return an invalid value, and write
        /// operations are ignored.
        ///
        /// Field Function
        /// 0000000000000b - No data transfer
        /// 0000000000001b - 1 byte
        /// 0000000000010b - 2 bytes
        /// 0000000000011b - 3 bytes
        /// 0000000000100b - 4 bytes
        /// 0000111111111b - 511 bytes
        /// 0001000000000b - 512 bytes
        /// 0100000000000b - 2048 bytes
        /// 1000000000000b - 4096 byte
        BLKSIZE OFFSET(0) NUMBITS(12) [],
        RESERVED OFFSET(13) NUMBITS(3) [],
        /// Blocks count for current transfer
        ///
        /// This register is enabled when the Block Count Enable field in the Transfer Mode register is set to 1 and is
        /// valid only for multiple block transfers. For single block transfer, this register always reads as 1. The host
        /// driver sets this register to a value between 1 and the maximum block count. The uSDHC module
        /// decrements the block count after each block transfer and stops when the count reaches zero. Setting the
        /// block count to zero results in no data blocks being transferred
        ///
        /// 0000000000000000b - Stop count
        /// 0000000000000001b - 1 block
        /// 0000000000000010b - 2 blocks
        /// 1111111111111111b - 65535 blocks
        BLKCNT OFFSET(16) NUMBITS(16) [],
    ],
    /// This register contains the SD/MMC command argument.
    CMD_ARG [
        /// Command argument
        ///
        /// The SD/MMC command argument is specified as bits 39-8 of the command format in the SD or MMC
        /// specification. This register is write protected when the Command Inhibit (CMD) field in the Present State
        /// register is set.
        CMDARG OFFSET(0) NUMBITS(31) [],
    ],
    /// This register is used to control the operation of data transfers.
    CMD_XFR_TYP [
        RESERVED0 OFFSET(0) NUMBITS(16) [],
        /// Response type select
        ///
        /// 00b - No response
        /// 01b - Response length 136
        /// 10b - Response length 48
        /// 11b - Response length 48, check busy after response
        RSPTYP OFFSET(16) NUMBITS(2) [],
        RESERVED1 OFFSET(18) NUMBITS(1) [],
        /// Command CRC check enable
        ///
        /// If this field is set to 1, uSDHC checks the CRC field in the response. If an error is detected, it is reported
        /// as a Command CRC Error. If this field is set to 0, the CRC field is not checked. The number of bits
        /// checked by the CRC field value changes according to the length of the response.
        ///
        /// Command Transfer Type (CMD_XFR_TYP).
        /// 0b - Disables command CRC check
        /// 1b - Enables command CRC check
        CCCEN OFFSET(19) NUMBITS(1) [],
        /// Command index check enable
        ///
        /// If this field is set to 1, uSDHC checks the Index field in the response to see if it has the same value as the
        /// command index. If it is not, it is reported as a Command Index Error. If this field is set to 0, the Index field
        /// is not checked.
        ///
        /// 0b - Disable command index check
        /// 1b - Enables command index check
        CICEN OFFSET(20) NUMBITS(1) [],
        /// Data present select
        ///
        /// This field is set to 1 to indicate that data is present and is transferred using the DATA line. It is set to 0 for
        /// the following:
        ///
        /// • Commands using only the CMD line (for example, CMD52)
        /// • Commands with no data transfer, but using the busy signal on DATA0 line (R1b or R5b (for
        /// example, CMD38))
        ///
        /// 0b - No data present
        /// 1b - Data present
        DPSEL OFFSET(21) NUMBITS(1) [],
        /// Command type
        ///
        /// There are three types of special commands: Suspend, Resume, and Abort. These bits are set to 00b for
        /// all other commands.
        ///
        /// 00b - Normal other commands
        /// 01b - Suspend CMD52 for writing bus suspend in CCCR
        /// 10b - Resume CMD52 for writing function select in CCCR
        /// 11b - Abort CMD12, CMD52 for writing I/O Abort in CCCR
        CMDTYP OFFSET(22) NUMBITS(2) [],
        /// Command index
        ///
        /// These bits are set to the command number that is specified in bits 45-40 of the command-format in the
        /// SD Memory Card Physical Layer Specification and SDIO Card Specification.
        CMDINX OFFSET(24) NUMBITS(5) [],
        RESERVED2 OFFSET(30) NUMBITS(2) [],
    ],
    /// This register is used to store part 0 of the response bits from the card.
    CMD_RSP0 [
        /// Command response 0
        ///
        /// See Command Response3 (CMD_RSP3) for the mapping of command responses from the SD bus to
        /// this register for each response type
        CMDRSP0 OFFSET(0) NUMBITS(31) [],
    ],
    /// This register is used to store part 1 of the response bits from the card.
    CMD_RSP1 [
        /// Command response 1
        ///
        /// See Command Response3 (CMD_RSP3) for the mapping of command responses from the SD bus to
        /// this register for each response type
        CMDRSP1 OFFSET(0) NUMBITS(31) [],
    ],
    /// This register is used to store part 2 of the response bits from the card.
    CMD_RSP2 [
        /// Command response 2
        ///
        /// See Command Response3 (CMD_RSP3) for the mapping of command responses from the SD bus to
        /// this register for each response type
        CMDRSP2 OFFSET(0) NUMBITS(31) [],
    ],
    /// This register is used to store part 3 of the response bits from the card.
    CMD_RSP3 [
        /// Command response 3
        ///
        /// See Command Response3 (CMD_RSP3) for the mapping of command responses from the SD bus to
        /// this register for each response type
        ///
        /// See Table 10-38. Response bit definition for each response type in i.MX8MNRM
        CMDRSP3 OFFSET(0) NUMBITS(31) [],
    ],
    /// This is a 32-bit data port register used to access the internal buffer.
    DATA_BUFF_ACC_PORT [
        /// Data content
        /// 
        /// The Buffer Data Port register is for 32-bit data access by the Arm platform or the external DMA. When the
        /// internal DMA is enabled, any write to this register is ignored, and any read from this register always yields
        /// 0s
        CMDRSP3 OFFSET(0) NUMBITS(31) [],
    ],
    /// The host driver can get status of uSDHC from this 32-bit read only register
    PRES_STATE [
        /// Command inhibit (CMD)
        /// 
        /// If this status bit is 0, it indicates that the CMD line is not in use and uSDHC can issue a SD / MMC
        /// command using the CMD line
        ///
        /// 0b - Can issue command using only CMD line
        /// 1b - Cannot issue command 
        CIHB OFFSET(0) NUMBITS(1) [],
        /// Command inhibit (DATA)
        /// 
        /// This status field is generated if either the DAT Line Active or the Read Transfer Active is set to 1. If this
        /// field is 0, it indicates that uSDHC can issue the next SD / MMC Command.
        /// 
        /// 0b - Can issue command that uses the DATA line
        /// 1b - Cannot issue command that uses the DATA line
        CDIHB OFFSET(1) NUMBITS(1) [],
        /// Data line active
        /// 
        /// This status field indicates whether one of the DATA lines on the SD bus is in use
        /// 
        /// 0b - DATA line inactive
        /// 1b - DATA line active
        DLA OFFSET(2) NUMBITS(1) [],
        /// SD clock stable
        /// 
        /// This status field indicates that the internal card clock is stable.
        /// 
        /// 0b - Clock is changing frequency and not stable.
        /// 1b - Clock is stable.
        SDSTB OFFSET(3) NUMBITS(1) [],
        /// Peripheral clock gated off internally
        /// 
        /// This status field indicates that the peripheral clock is internally gated off. This field is for the host driver to
        /// debug.
        /// 
        /// 0b - Peripheral clock is active.
        /// 1b - Peripheral clock is gated off.
        IPGOFF OFFSET(4) NUMBITS(1) [],
        /// HCLK gated off internally
        /// 
        /// This status field indicates that the HCLK is internally gated off. This field is for the host driver to debug
        /// during a data transfer.
        /// 
        /// 0b - HCLK is active.
        /// 1b - HCLK is gated off.
        HCKOFF OFFSET(5) NUMBITS(1) [],
        /// IPG_PERCLK gated off internally
        /// 
        /// This status field indicates that the IPG_PERCLK is internally gated off. This field is for the host driver to
        /// debug transaction on the SD bus. When IPG_CLK_SOFT_EN is cleared, IPG_PERCLK is gated off,
        /// otherwise IPG_PERCLK is always active.
        /// 
        /// 0b - IPG_PERCLK is active.
        /// 1b - IPG_PERCLK is gated off.
        PEROFF OFFSET(6) NUMBITS(1) [],
        /// SD clock gated off internally
        /// 
        /// This status field indicates that the SD clock is internally gated off, because of buffer over / under-run or
        /// read pause without read wait assertion, or the driver set FRC_SDCLK_ON field is 0 to stop the SD clock
        /// in idle status. This field is for the host driver to debug data transaction on the SD bus.
        /// 
        /// 0b - SD clock is active.
        /// 1b - SD clock is gated off.
        SDOFF OFFSET(7) NUMBITS(1) [],
        /// Write transfer active
        /// 
        /// This status field indicates a write transfer is active. 
        /// 
        /// 0b - No valid data
        /// 1b - Transferring data
        WTA OFFSET(8) NUMBITS(1) [],
        /// Read transfer active
        /// 
        /// This status field is used for detecting completion of a read transfer.
        /// 
        /// 0b - No valid data
        /// 1b - Transferring data
        RTA OFFSET(9) NUMBITS(1) [],
        /// Buffer write enable
        /// 
        /// This status field is used for non-DMA write transfers.
        /// 
        /// 0b - Write disable
        /// 1b - Write enable
        BWEN OFFSET(10) NUMBITS(1) [],
        /// Buffer read enable
        /// 
        /// This status field is used for non-DMA read transfers. 
        /// 
        /// 0b - Read disable
        /// 1b - Read enable
        BREN OFFSET(11) NUMBITS(1) [],
        /// Re-Tuning Request (only for SD3.0 SDR104 mode and EMMC HS200 mode
        RTR OFFSET(12) NUMBITS(1) [],
        RESERVED0 OFFSET(13) NUMBITS(2) [],
        /// Tape select change done
        /// 
        /// This field indicates the delay setting is effective after write CLK_TUNE_CTRL_STATUS register.
        /// 
        /// 0b - Delay cell select change is not finished.
        /// 1b - Delay cell select change is finished.
        TSCD OFFSET(15) NUMBITS(1) [],
        /// Card inserted
        /// 
        /// This field indicates whether a card has been inserted. 
        /// 
        /// 0b - Power on reset or no card
        /// 1b - Card inserted
        CINST OFFSET(16) NUMBITS(1) [],
        RESERVED1 OFFSET(17) NUMBITS(1) [],
        /// Card detect pin level
        /// 
        /// This field reflects the inverse value of the CD_B pin for the card socket. Debouncing is not performed on
        /// this field. 
        /// 
        /// 0b - No card present (CD_B = 1)
        /// 1b - Card present (CD_B = 0)
        CDPL OFFSET(18) NUMBITS(1) [],
        /// Write protect switch pin level
        /// 
        /// The Write Protect switch is supported for memory and combo cards. 
        /// 
        /// 0b - Write protected (WP = 1)
        /// 1b - Write enabled (WP = 0)
        WPSPL OFFSET(19) NUMBITS(1) [],
        RESERVED2 OFFSET(20) NUMBITS(3) [],
        /// CMD line signal level
        /// 
        /// This status is used to check the CMD line level to recover from errors, and for debugging. 
        CLSL OFFSET(23) NUMBITS(1) [],
        /// DATA[7:0] line signal level
        /// 
        /// This status is used to check the DATA line level to recover from errors, and for debugging. This is
        /// especially useful in detecting the busy signal level from DATA0. 
        /// 
        /// 00000000b - Data 0 line signal level
        /// 00000001b - Data 1 line signal level
        /// 00000010b - Data 2 line signal level
        /// 00000011b - Data 3 line signal level
        /// 00000100b - Data 4 line signal level
        /// 00000101b - Data 5 line signal level
        /// 00000110b - Data 6 line signal level
        /// 00000111b - Data 7 line signal level
        DLSL OFFSET(24) NUMBITS(7) [],
    ],
}
