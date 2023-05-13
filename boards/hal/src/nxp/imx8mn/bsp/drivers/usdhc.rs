//! Ultra Secured Digital Host Controller (uSDHC) driver.
//!
//! # Resources
//!
//! Descriptions taken from
//! i.MX 8M Nano Applications Processor Reference Manual, Document Number: IMX8MNRM Rev. 2, 07/2022

use super::common::MMIODerefWrapper;
use crate::nxp::imx8mn::arch::cpu_core;
use crate::nxp::imx8mn::bsp::drivers::usdhc::INT_STATUS::DEBE;
use crate::nxp::imx8mn::bsp::global::GPIO2;
use crate::{info, print, warn};
use core::fmt::Debug;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
    LocalRegisterCopy,
};

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
        RSPTYP OFFSET(16) NUMBITS(2) [
            ///  - 0b00: no response
            CMD_NO_RESP = 0b00,
            ///  - 0b01: Response length 136
            CMD_136BIT_RESP = 0b01,
            ///  - 0b10: Response length 48
            CMD_48BIT_RESP = 0b10,
            ///  - 0b11: Response length 48, check busy after response
            CMD_BUSY48BIT_RESP = 0b11
        ],
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
        CMDINX OFFSET(24) NUMBITS(6) [],
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
        DATCONT OFFSET(0) NUMBITS(31) [],
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
        DLSL OFFSET(24) NUMBITS(8) [],
    ],
    /// There are three cases to restart the transfer after stop at the block gap.
    ///
    /// Which case is appropriate depends on whether uSDHC issues a Suspend command or the SD card
    /// accepts the Suspend command.
    ///
    /// • If the host driver does not issue a Suspend command, the Continue request is used to
    /// restart the transfer.
    /// • If the host driver issues a Suspend command and the SD card accepts it, a Resume
    /// command is used to restart the transfer.
    /// • If the host driver issues a Suspend command and the SD card does not accept it, the
    /// Continue request is used to restart the transfer.
    PROT_CTRL [
        /// LED control
        ///
        /// This field, fully controlled by the host driver, is used to caution the user not to remove the card while the
        /// card is being accessed. If the software is going to issue multiple SD commands, this field can be set
        /// during all these transactions. It is not necessary to change for each transaction. When the software
        /// issues multiple SD commands, setting the field once before the first command is sufficient: it is not
        /// necessary to reset the bit between commands.
        ///
        /// 0b - LED off
        /// 1b - LED on
        LCTL OFFSET(0) NUMBITS(1) [],
        /// Data transfer width
        ///
        /// This field selects the data width of the SD bus for a data transfer. The host driver sets it to match the data
        /// width of the card. Possible data transfer width is 1-bit, 4-bits or 8-bits.
        ///
        /// 00b - 1-bit mode
        /// 01b - 4-bit mode
        /// 10b - 8-bit mode
        /// 11b - Reserved
        DTW OFFSET(1) NUMBITS(2) [
            OneBitWide = 0b00,
            FourBitWide = 0b01,
            EightBitWide = 0b10,
            Reserved = 0b11,
        ],
        /// DATA3 as card detection pin
        ///
        /// If this field is set, DATA3 should be pulled down to act as a card detection pin. Be cautious when using
        /// this feature, because DATA3 is also a chip-select for the SPI mode. A pull-down on this pin and CMD0
        /// might set the card into the SPI mode, which uSDHC does not support.
        ///
        /// 0b - DATA3 does not monitor card insertion
        /// 1b - DATA3 as card detection pin
        D3CD OFFSET(3) NUMBITS(1) [],
        /// Endian mode
        ///
        /// The uSDHC module supports all three endian modes in data transfer. See Data buffer for more details.
        ///
        /// 00b - Big endian mode
        /// 01b - Half word big endian mode
        /// 10b - Little endian mode
        /// 11b - Reserved
        EMODE OFFSET(4) NUMBITS(2) [
            BigEndianMode = 0b00,
            HalfWordBigEndianMode = 0b01,
            LittleEndianMode = 0b10,
            Reserved = 0b11,
        ],
        /// Card detect test level
        ///
        /// This bit is enabled while the card detection signal selection is set to 1 and it indicates card insertion.
        ///
        /// 0b - Card detect test level is 0, no card inserted
        /// 1b - Card detect test level is 1, card inserted
        CDTL OFFSET(6) NUMBITS(1) [],
        /// Card detect signal selection
        ///
        /// This field selects the source for the card detection.
        ///
        /// 0b - Card detection level is selected (for normal purpose).
        /// 1b - Card detection test level is selected (for test purpose).
        CDSS OFFSET(7) NUMBITS(1) [],
        /// DMA select
        ///
        /// This field is valid while DMA (SDMA or ADMA) is enabled and selects the DMA operation.
        ///
        /// 00b - No DMA or simple DMA is selected.
        /// 01b - ADMA1 is selected.
        /// 10b - ADMA2 is selected.
        /// 11b - Reserved
        DMASEL OFFSET(8) NUMBITS(2) [],
        RESERVED0 OFFSET(10) NUMBITS(6) [],
        /// Stop at block gap request
        ///
        /// This field is used to stop executing a transaction at the next block gap for both DMA and non-DMA
        /// transfers.
        ///
        /// 0b - Transfer
        /// 1b - Stop
        SABGREQ OFFSET(16) NUMBITS(1) [],
        /// Continue request
        ///
        /// This field is used to restart a transaction which was stopped using the stop at block gap request.
        ///
        /// 0b - No effect
        /// 1b - Restart
        CREQ OFFSET(17) NUMBITS(1) [],
        /// Read wait control
        ///
        /// The read wait function is optional for SDIO cards. If the card supports read wait, set this field to enable
        /// use of the read wait protocol to stop read data using the DATA2 line.
        ///
        /// 0b - Disables read wait control and stop SD clock at block gap when SABGREQ field is set
        /// 1b - Enables read wait control and assert read wait without stopping SD clock at block gap when
        /// SABGREQ field is set
        RWCTL OFFSET(18) NUMBITS(1) [],
        /// Interrupt at block gap
        ///
        /// This field is valid only in 4-bit mode, of the SDIO card, and selects a sample point in the interrupt cycle.
        /// Setting to 1 enables interrupt detection at the block gap for a multiple block transfer. Setting to 0 disables
        /// interrupt detection during a multiple block transfer. If the SDIO card cannot signal an interrupt during a
        /// multiple block transfer, this field should be set to 0 to avoid an inadvertent interrupt. When the host driver
        /// detects an SDIO card insertion, it sets this field according to the CCCR of the card.
        ///
        /// 0b - Disables interrupt at block gap
        /// 1b - Enables interrupt at block gap
        IABG OFFSET(19) NUMBITS(1) [],
        /// Read performed number 8 clock
        ///
        /// According to the SD/MMC spec, for read data transaction, 8 clocks are needed after the end field of the
        /// last data block. So, by default(RD_DONE_NO_8CLK=0), 8 clocks are active after the end field of the last
        /// read data transaction
        ///
        /// In a summary, this field should be set only if the use case needs to use stop at block gap feature while
        /// the device can't support the read wait feature.
        RD_DONE_NO_8CLK OFFSET(20) NUMBITS(1) [],
        RESERVED1 OFFSET(21) NUMBITS(2) [],
        RESRW1C OFFSET(23) NUMBITS(1) [],
        /// Wakeup event enable on card interrupt
        ///
        /// This field enables a wakeup event, via a card interrupt, in the Interrupt Status register. This field can be
        /// set to 1 if FN_WUS (Wake Up Support) in CIS is set to 1. When this field is set, the Card Interrupt Status
        /// and uSDHC interrupt can be asserted without CLK toggling. When the wakeup feature is not enabled, the
        /// CLK must be active to assert the Card Interrupt Status and uSDHC interrupt.
        ///
        /// 0b - Disables wakeup event enable on card interrupt
        /// 1b - Enables wakeup event enable on card interrupt
        WECINT OFFSET(24) NUMBITS(1) [],
        /// Wakeup event enable on SD card insertion
        ///
        /// This field enables a wakeup event, via a card insertion, in the Interrupt Status register. FN_WUS (Wake
        /// Up Support) in CIS does not affect this field. When this field is set, the Card Insertion Status and uSDHC
        /// interrupt can be asserted without CLK toggling. When the wakeup feature is not enabled, the CLK must
        /// be active to assert the Card Insertion Status and uSDHC interrupt.
        ///
        /// 0b - Disable wakeup event enable on SD card insertion
        /// 1b - Enable wakeup event enable on SD card insertion
        WECINS OFFSET(25) NUMBITS(1) [],
        /// Wakeup event enable on SD card removal
        ///
        /// This field enables a wakeup event, via a card removal, in the Interrupt Status register. FN_WUS (Wake
        /// Up Support) in CIS does not affect this field. When this field is set, the Card Removal Status and uSDHC
        /// interrupt can be asserted without CLK toggling. When the wakeup feature is not enabled, the CLK must
        /// be active to assert the Card Removal Status and uSDHC interrupt.
        ///
        /// 0b - Disables wakeup event enable on SD card removal
        /// 1b - Enables wakeup event enable on SD card remova
        WECRM OFFSET(26) NUMBITS(1) [],
        /// BURST length enable for INCR, INCR4 / INCR8 / INCR16, INCR4-WRAP / INCR8-WRAP / INCR16-
        /// WRAP
        ///
        /// This is used to enable / disable the burst length for the external AHB2AXI bridge.
        ///
        /// 1xxb - Burst length is enabled for INCR4-WRAP / INCR8-WRAP / INCR16-WRAP.
        /// x1xb - Burst length is enabled for INCR4 / INCR8 / INCR16.
        /// xx1b - Burst length is enabled for INCR.
        BURST_LEN_EN OFFSET(27) NUMBITS(3) [],
        /// Non-exact block read
        ///
        /// Current block read is non-exact block read. It is only used for SDIO.
        ///
        /// 0b - The block read is exact block read. Host driver does not need to issue abort command to
        /// terminate this multi-block read.
        /// 1b - The block read is non-exact block read. Host driver needs to issue abort command to
        /// terminate this multi-block read
        NON_EXACT_BLK_RD OFFSET(30) NUMBITS(1) [],
        RESERVED2 OFFSET(31) NUMBITS(1) [],
    ],
    /// This register provides control of the system.
    SYS_CTRL [
        RESERVED0 OFFSET(0) NUMBITS(4) [],
        /// Divisor
        ///
        /// This register is used to provide a more exact divisor to generate the desired SD clock frequency. Note the
        /// divider can even support odd divisors without deterioration of duty cycle.
        /// Before changing clock divisor value (SDCLKFS or DVS), Host driver should make sure the SDSTB field is
        /// high.
        ///
        /// The settings are as follows:
        /// 0000b - Divide-by-1
        /// 0001b - Divide-by-2
        /// 1110b - Divide-by-15
        /// 1111b - Divide-by-16
        DVS OFFSET(4) NUMBITS(4) [],
        /// SDCLK frequency select
        ///
        /// This register is used to select the frequency of the SDCLK pin. The frequency is not programmed directly,
        /// rather this register holds the prescaler (this register) and divisor (next register) of the Base Clock
        /// Frequency register.
        ///
        /// See detail in the field description in reference manual
        SDCLKFS OFFSET(8) NUMBITS(8) [],
        /// Data timeout counter value
        ///
        /// This value determines the interval by which DAT line timeouts are detected. See the Data Timeout Error
        /// field in the Interrupt Status register for information on factors that dictate time-out generation. Time-out
        /// clock frequency is generated by dividing the base clock SDCLK value by this value.
        ///
        /// The host driver can clear the Data Timeout Error Status Enable (in the Interrupt Status Enable register) to
        /// prevent inadvertent time-out events.
        DTOCV OFFSET(16) NUMBITS(4) [],
        RESERVED1 OFFSET(20) NUMBITS(3) [],
        /// Hardware reset
        ///
        /// This register's value is output to card through pad directly to hardware reset pin of the card if the card
        /// supports this feature.
        IPP_RST_N OFFSET(23) NUMBITS(1) [],
        /// Software reset for all
        ///
        /// This reset effects the entire host controller except for the card detection circuit.
        RSTA OFFSET(24) NUMBITS(1) [
            NoReset = 0,
            Reset = 1,
        ],
        /// Software reset for CMD line
        ///
        /// Only part of the command circuit is reset. After this field is set, the software waits for self-clear.
        /// The following registers and bits are cleared by this field:
        /// • Present State Register Command Inhibit (CMD)
        /// • Interrupt Status register Command Complete
        ///
        /// 0b - No reset
        /// 1b - Reset
        RSTC OFFSET(25) NUMBITS(1) [
            NoReset = 0,
            Reset = 1,
        ],
        /// Software reset for data line
        ///
        /// Only part of the data circuit is reset. DMA circuit is also reset. After this field is set, the software waits for
        /// self-clear.
        RSTD OFFSET(26) NUMBITS(1) [
            NoReset = 0,
            Reset = 1,
        ],
        /// Initialization active
        ///
        /// When this field is set, 80 SD-clocks are sent to the card. After the 80 clocks are sent, this field is self
        /// cleared. This field is very useful during the card power-up period when 74 SD-clocks are needed and the
        /// clock auto gating feature is enabled
        INITA OFFSET(27) NUMBITS(1) [],
        /// Reset tuning
        ///
        /// When set this field to 1, it resets tuning circuit. After tuning circuits are reset, bit value is 0. Clearing
        /// execute_tuning field in AUTOCMD12_ERR_STATUS also sets this field to 1 to reset tuning circuit
        RSTT OFFSET(28) NUMBITS(1) [],
        RESERVED2 OFFSET(29) NUMBITS(3) [],
    ],
    /// This register provides control of the system.
    INT_STATUS [
        /// Command complete
        ///
        /// This field is set when you receive the end field of the command response (except auto CMD12). See the
        /// Command Inhibit (CMD) in the Present State register.
        ///
        /// This field is not asserted in tuning process.
        /// 0b - Command not complete
        /// 1b - Command complete
        CC OFFSET(0) NUMBITS(1) [],
        TC OFFSET(1) NUMBITS(1) [],
        BGE OFFSET(2) NUMBITS(1) [],
        DINT OFFSET(3) NUMBITS(1) [],
        BWR OFFSET(4) NUMBITS(1) [],
        BRR OFFSET(5) NUMBITS(1) [],
        CINS OFFSET(6) NUMBITS(1) [],
        CRM OFFSET(7) NUMBITS(1) [],
        CINT OFFSET(8) NUMBITS(1) [],
        RESERVED0 OFFSET(9) NUMBITS(3) [],
        RTE OFFSET(12) NUMBITS(1) [],
        TP OFFSET(13) NUMBITS(1) [],
        CQI OFFSET(14) NUMBITS(1) [],
        RESERVED1 OFFSET(15) NUMBITS(1) [],
        CTOE OFFSET(16) NUMBITS(1) [],
        CCE OFFSET(17) NUMBITS(1) [],
        CEBE OFFSET(18) NUMBITS(1) [],
        CIE OFFSET(19) NUMBITS(1) [],
        DTOE OFFSET(20) NUMBITS(1) [],
        DCE OFFSET(21) NUMBITS(1) [],
        DEBE OFFSET(22) NUMBITS(1) [],
        RESERVED2 OFFSET(23) NUMBITS(1) [],
        AC12E OFFSET(24) NUMBITS(1) [],
        RESERVED3 OFFSET(25) NUMBITS(1) [],
        TNE OFFSET(26) NUMBITS(1) [],
        RESERVED4 OFFSET(27) NUMBITS(1) [],
        DMAE OFFSET(28) NUMBITS(1) [],
        RESERVED5 OFFSET(29) NUMBITS(3) [],
    ],
    /// Setting the fields in this register to 1 enables the corresponding Interrupt Status to be set
    /// by the specified event.
    INT_STATUS_EN [
        CCSEN OFFSET(0) NUMBITS(1) [],
        TCSEN OFFSET(1) NUMBITS(1) [],
        BGESEN OFFSET(2) NUMBITS(1) [],
        DINTSEN OFFSET(3) NUMBITS(1) [],
        BWRSEN OFFSET(4) NUMBITS(1) [],
        BRRSENN OFFSET(5) NUMBITS(1) [],
        CINSSEN OFFSET(6) NUMBITS(1) [],
        CRMSEN OFFSET(7) NUMBITS(1) [],
        CINTSEN OFFSET(8) NUMBITS(1) [],
        RESERVED0 OFFSET(9) NUMBITS(3) [],
        RTESEN OFFSET(12) NUMBITS(1) [],
        TPSEN OFFSET(13) NUMBITS(1) [],
        CQISEN OFFSET(14) NUMBITS(1) [],
        RESERVED1 OFFSET(15) NUMBITS(1) [],
        CTOESEN OFFSET(16) NUMBITS(1) [],
        CCESEN OFFSET(17) NUMBITS(1) [],
        CEBESEN OFFSET(18) NUMBITS(1) [],
        CIESEN OFFSET(19) NUMBITS(1) [],
        DTOESEN OFFSET(20) NUMBITS(1) [],
        DCESEN OFFSET(21) NUMBITS(1) [],
        DEBESEN OFFSET(22) NUMBITS(1) [],
        RESERVED2 OFFSET(23) NUMBITS(1) [],
        AC12ESEN OFFSET(24) NUMBITS(1) [],
        RESERVED3 OFFSET(25) NUMBITS(1) [],
        TNESEN OFFSET(26) NUMBITS(1) [],
        RESERVED4 OFFSET(27) NUMBITS(1) [],
        DMAESEN OFFSET(28) NUMBITS(1) [],
        RESERVED5 OFFSET(29) NUMBITS(3) [],
    ],
    /// This register is used to select which interrupt status is indicated to the host system as the
    /// interrupt. These status fields all share the same interrupt line. Setting any of these fields
    /// to 1 enables interrupt generation. The corresponding Status register field generates an
    /// interrupt when the corresponding interrupt signal enable field is set.
    INT_SIGNAL_EN [
        CCIEN OFFSET(0) NUMBITS(1) [],
        TCIEN OFFSET(1) NUMBITS(1) [],
        BGEIEN OFFSET(2) NUMBITS(1) [],
        DINTIEN OFFSET(3) NUMBITS(1) [],
        BWRIEN OFFSET(4) NUMBITS(1) [],
        BRRIEN OFFSET(5) NUMBITS(1) [],
        CINSIEN OFFSET(6) NUMBITS(1) [],
        CRMIEN OFFSET(7) NUMBITS(1) [],
        CINTIEN OFFSET(8) NUMBITS(1) [],
        RESERVED0 OFFSET(9) NUMBITS(3) [],
        RTEIEN OFFSET(12) NUMBITS(1) [],
        TPIEN OFFSET(13) NUMBITS(1) [],
        CQIIEN OFFSET(14) NUMBITS(1) [],
        RESERVED1 OFFSET(15) NUMBITS(1) [],
        CTOEIEN OFFSET(16) NUMBITS(1) [],
        CCEIEN OFFSET(17) NUMBITS(1) [],
        CEBEIEN OFFSET(18) NUMBITS(1) [],
        CIEIEN OFFSET(19) NUMBITS(1) [],
        DTOEIEN OFFSET(20) NUMBITS(1) [],
        DCEIEN OFFSET(21) NUMBITS(1) [],
        DEBEIEN OFFSET(22) NUMBITS(1) [],
        RESERVED2 OFFSET(23) NUMBITS(1) [],
        AC12EIEN OFFSET(24) NUMBITS(1) [],
        RESERVED3 OFFSET(25) NUMBITS(1) [],
        TNEIEN OFFSET(26) NUMBITS(1) [],
        RESERVED4 OFFSET(27) NUMBITS(1) [],
        DMAEIEN OFFSET(28) NUMBITS(1) [],
        RESERVED5 OFFSET(29) NUMBITS(3) [],
    ],
    /// This register provides the host driver with information specific to uSDHC
    /// implementation. The value in this register is the power-on-reset value and does not
    /// change with a software reset.
    HOST_CTRL_CAP [
        SDR50_SUPPORT OFFSET(0) NUMBITS(1) [],
        SDR104_SUPPORT OFFSET(1) NUMBITS(1) [],
        DDR50_SUPPORT OFFSET(2) NUMBITS(1) [],
        RESERVED0 OFFSET(3) NUMBITS(5) [],
        TIME_COUNT_RETUNING OFFSET(8) NUMBITS(4) [],
        RESERVED1 OFFSET(12) NUMBITS(1) [],
        USE_TUNING_SDR50 OFFSET(13) NUMBITS(1) [],
        RETUNING_MODE OFFSET(14) NUMBITS(2) [],
        MBL OFFSET(16) NUMBITS(3) [],
        RESERVED2 OFFSET(19) NUMBITS(1) [],
        ADMAS OFFSET(20) NUMBITS(1) [],
        HSS OFFSET(21) NUMBITS(1) [],
        DMAS OFFSET(22) NUMBITS(1) [],
        SRS OFFSET(23) NUMBITS(1) [],
        VS33 OFFSET(24) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        VS30 OFFSET(25) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        VS18 OFFSET(26) NUMBITS(1) [
            NotSupported = 0,
            Supported = 1,
        ],
        RESERVED3 OFFSET(27) NUMBITS(5) [],
    ],
    /// This register contains the MMC Fast Boot control register.
    MMCBOOT [
        DTOCV_ACK OFFSET(0) NUMBITS(4) [],
        BOOT_ACK OFFSET(4) NUMBITS(1) [],
        BOOT_MODE OFFSET(5) NUMBITS(1) [],
        BOOT_EN OFFSET(6) NUMBITS(1) [],
        AUTO_SABG_EN OFFSET(7) NUMBITS(1) [],
        DISABLE_TIME_OUT OFFSET(8) NUMBITS(1) [],
        RESERVED0 OFFSET(9) NUMBITS(7) [],
        BOOT_BLK_CNT OFFSET(16) NUMBITS(16) [],
    ],
    /// This register is used to DMA and data transfer. To prevent data loss, The software should
    /// check if data transfer is active before writing this register. These fields are DPSEL,
    /// MBSEL, DTDSEL, AC12EN, BCEN, and DMAEN
    MIXCTRL [
        DMAEN OFFSET(0) NUMBITS(1) [],
        BCEN OFFSET(1) NUMBITS(1) [],
        AC12EN OFFSET(2) NUMBITS(1) [],
        DDR_EN OFFSET(3) NUMBITS(1) [],
        DTDSEL OFFSET(4) NUMBITS(1) [],
        MSBSEL OFFSET(5) NUMBITS(1) [],
        NIBBLE_POS OFFSET(6) NUMBITS(1) [],
        AC23EN OFFSET(7) NUMBITS(1) [],
        RESERVED0 OFFSET(8) NUMBITS(14) [],
        EXE_TUNE OFFSET(22) NUMBITS(1) [],
        SMP_CLK_SEL OFFSET(23) NUMBITS(1) [],
        AUTO_TUNE_EN OFFSET(24) NUMBITS(1) [],
        FBCLK_SEL OFFSET(25) NUMBITS(1) [],
        HS400_MODE OFFSET(26) NUMBITS(1) [],
        EN_HS400_MODE OFFSET(27) NUMBITS(1) [],
        RESERVED1 OFFSET(28) NUMBITS(1) [],
        RESERVED2 OFFSET(29) NUMBITS(2) [],
        RESERVED3 OFFSET(31) NUMBITS(1) [],
    ],
    /// This register contains the Clock Tuning Control status information. All fields are read
    /// only and reads the same as the power-reset value. This register is added to support SD3.0
    /// UHS-I SDR104 mode and EMMC HS200 mode.
    CLK_TUNE_CTRL_STS [
        DLY_CELL_SET_POST OFFSET(0) NUMBITS(4) [],
        DLY_CELL_SET_OUT OFFSET(4) NUMBITS(4) [],
        DLY_CELL_SET_PRE OFFSET(8) NUMBITS(7) [],
        NXT_ERR OFFSET(15) NUMBITS(1) [],
        TAP_SEL_POST OFFSET(16) NUMBITS(4) [],
        TAP_SEL_OUT OFFSET(20) NUMBITS(4) [],
        TAP_SEL_PRE OFFSET(24) NUMBITS(7) [],
        PRE_ERR OFFSET(31) NUMBITS(1) [],
    ],
    /// This register contains control fields for DLL.
    DLL_CTRL [
        DLL_CTRL_EN OFFSET(0) NUMBITS(1) [],
        DLL_CTRL_RST OFFSET(1) NUMBITS(1) [],
        DLL_CTRL_SLV_FORCE_UPD OFFSET(2) NUMBITS(1) [],
        DLL_CTRL_SLV_DLY_TARGET0 OFFSET(3) NUMBITS(4) [],
        DLL_CTRL_GATE_UPDT OFFSET(7) NUMBITS(1) [],
        DLL_CTRL_SLV_OVERRIDE OFFSET(8) NUMBITS(1) [],
        DLL_CTRL_SLV_OVERRIDE_VAL OFFSET(9) NUMBITS(7) [],
        DLL_CTRL_SLV_DLY_TARGET1 OFFSET(16) NUMBITS(3) [],
        RESERVED0 OFFSET(19) NUMBITS(1) [],
        DLL_CTRL_SLV_UPDT_INT OFFSET(20) NUMBITS(8) [],
        DLL_CTRL_REF_UPDT_INT OFFSET(28) NUMBITS(4) [],
    ],
    /// This register contains the vendor specific control / status register.
    VEND_SPEC [
        EXT_DMA_EN OFFSET(0) NUMBITS(1) [],
        VSELECT OFFSET(1) NUMBITS(1) [],
        CONFLICT_CHK_EN OFFSET(2) NUMBITS(1) [],
        AC12_WR_CHKBUSY_EN OFFSET(3) NUMBITS(4) [],
        RESERVED0 OFFSET(4) NUMBITS(4) [],
        FRC_SDCLK_ON OFFSET(8) NUMBITS(1) [],
        RESERVED1 OFFSET(9) NUMBITS(1) [],
        RESERVED2 OFFSET(10) NUMBITS(1) [],
        IPGEN OFFSET(11) NUMBITS(1) [],
        HCKEN OFFSET(12) NUMBITS(1) [],
        PEREN OFFSET(13) NUMBITS(1) [],
        CKEN OFFSET(14) NUMBITS(1) [],
        CRC_CHK_DIS OFFSET(15) NUMBITS(1) [],
        RESERVED4 OFFSET(16) NUMBITS(8) [],
        RESERVED5 OFFSET(24) NUMBITS(4) [],
        RESERVED6 OFFSET(28) NUMBITS(1) [],
        RSRV1 OFFSET(29) NUMBITS(1) [],
        RESERVED8 OFFSET(30) NUMBITS(1) [],
        CMD_BYTE_EN OFFSET(31) NUMBITS(1) [],
    ],
    /// Both write and read watermark levels (FIFO threshold) are configurable. Their value can
    /// range from 1 to 128 words. Both write and read burst lengths are also Configurable.
    /// There value can range from 1 to 31 words.
    WTMK_LVL [
        RD_WML OFFSET(0) NUMBITS(8) [],
        RD_BRST_LEN OFFSET(8) NUMBITS(5) [],
        RESERVED0 OFFSET(13) NUMBITS(3) [],
        WR_WML OFFSET(16) NUMBITS(8) [],
        WR_BRST_LEN OFFSET(24) NUMBITS(5) [],
        RESERVED1 OFFSET(29) NUMBITS(3) [],
    ],
    TUNINIG_CTRL [
        TUNING_START_TAP OFFSET(0) NUMBITS(8) [],
        TUNING_COUNTER OFFSET(8) NUMBITS(8) [],
        TUNING_STEP OFFSET(16) NUMBITS(3) [],
        RESRV0 OFFSET(19) NUMBITS(1) [],
        TUNING_WINDOW OFFSET(20) NUMBITS(3) [],
        RESRV1 OFFSET(23) NUMBITS(1) [],
        STD_TUNING_EN OFFSET(24) NUMBITS(1) [],
        RESRV2 OFFSET(25) NUMBITS(7) [],
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DS_ADDR: ReadWrite<u32, DS_ADDR::Register>),
        (0x04 => BLK_ATT: ReadWrite<u32, BLK_ATT::Register>),
        (0x08 => CMD_ARG: ReadWrite<u32, CMD_ARG::Register>),
        (0x0c => CMD_XFR_TYPE: ReadWrite<u32, CMD_XFR_TYP::Register>),
        (0x10 => CMD_RSP0: ReadOnly<u32, CMD_RSP0::Register>),
        (0x14 => CMD_RSP1: ReadOnly<u32, CMD_RSP1::Register>),
        (0x18 => CMD_RSP2: ReadOnly<u32, CMD_RSP2::Register>),
        (0x1c => CMD_RSP3: ReadOnly<u32, CMD_RSP3::Register>),
        (0x20 => DATA_BUFF_ACC_PORT: ReadWrite<u32, DATA_BUFF_ACC_PORT::Register>),
        (0x24 => PRES_STATE: ReadOnly<u32, PRES_STATE::Register>),
        (0x28 => PROT_CTRL: ReadWrite<u32, PROT_CTRL::Register>),
        (0x2c => SYS_CTRL: ReadWrite<u32, SYS_CTRL::Register>),
        (0x30 => INT_STATUS: ReadWrite<u32, INT_STATUS::Register>),
        (0x34 => INT_STATUS_EN: ReadWrite<u32, INT_STATUS_EN::Register>),
        (0x38 => INT_SIGNAL_EN: ReadWrite<u32, INT_SIGNAL_EN::Register>),
        (0x3c => AUTOCMD12_ERR_STATUS),
        (0x40 => HOST_CTRL_CAP: ReadWrite<u32, HOST_CTRL_CAP::Register>),
        (0x44 => WTMK_LVL: ReadWrite<u32, WTMK_LVL::Register>),
        (0x48 => MIXCTRL: ReadWrite<u32, MIXCTRL::Register>),
        (0x4c => _reserved0),
        (0x50 => FORCE_EVENT),
        (0x54 => ADMA_ERR_STATUS),
        (0x58 => ADMA_SYS_ADDR),
        (0x5c => _reserved1),
        (0x60 => DLL_CTRL: ReadWrite<u32, DLL_CTRL::Register>),
        (0x64 => DLL_STATUS),
        (0x68 => CLK_TUNE_CTRL_STS: ReadWrite<u32, CLK_TUNE_CTRL_STS::Register>),
        (0x6c => _reserved2),
        (0x70 => STROBE_DLL_CTRL),
        (0x74 => STROBE_DLL_STATUS),
        (0x78 => _reserved3),
        (0xc0 => VEND_SPEC: ReadWrite<u32, VEND_SPEC::Register>),
        (0xc4 => MMCBOOT:  ReadWrite<u32, MMCBOOT::Register>),
        (0xc8 => VEND_SPEC2),
        (0xcc => TUNINIG_CTRL: ReadWrite<u32, TUNINIG_CTRL::Register>),
        (0xd0 => _reserved4),
        (0x100 => CQE),
        (0x104 => @END),
    }
}

use INT_STATUS_EN::*;

/// Abstraction for the associated uSDHC MMIO register block.
type UsdhcRegisters = MMIODerefWrapper<RegisterBlock>;

/*--------------------------------------------------------------------------
   INTERNAL SD CARD REGISTER STRUCTURES AS PER SD CARD STANDARD
--------------------------------------------------------------------------*/

register_bitfields! {
    u32,

    /// SD CARD OCR register. The 32-bit Operation Conditions Register (OCR) stores the voltage profile of the card.
    /// Additionally, this register includes status information bits. One status bit is set if the card power up
    /// procedure has been finished. This register includes another status bit indicating the card capacity status
    /// after set power up status bit. The OCR register shall be implemented by the cards.
    OCR [
        /// Write as zero read as don't care
        _reserved OFFSET(0) NUMBITS(15) [],
        /// Voltage window 2.7v to 2.8v
        voltage2v7to2v8 OFFSET(15) NUMBITS(1) [],
        /// Voltage window 2.8v to 2.9v
        voltage2v8to2v9 OFFSET(16) NUMBITS(1) [],
        /// Voltage window 2.9v to 3.0v
        voltage2v9to3v0 OFFSET(17) NUMBITS(1) [],
        /// Voltage window 3.0v to 3.1v
        voltage3v0to3v1 OFFSET(18) NUMBITS(1) [],
        /// Voltage window 3.1v to 3.2v
        voltage3v1to3v2 OFFSET(19) NUMBITS(1) [],
        /// Voltage window 3.2v to 3.3v
        voltage3v2to3v3 OFFSET(20) NUMBITS(1) [],
        /// Voltage window 3.3v to 3.4v
        voltage3v3to3v4 OFFSET(21) NUMBITS(1) [],
        /// Voltage window 3.4v to 3.5v
        voltage3v4to3v5 OFFSET(22) NUMBITS(1) [],
        /// Voltage window 3.5v to 3.6v
        voltage3v5to3v6 OFFSET(23) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(24) NUMBITS(6) [],
        /// Card Capacity status
        card_capacity OFFSET(30) NUMBITS(1) [],
        /// Card power up status (busy)
        card_power_up_busy OFFSET(31) NUMBITS(1) [],
    ],
}

register_bitfields! {
    u64,

    /// A configuration register named SD CARD Configuration Register (SCR). SCR provides
    /// information on the microSD Memory Card's special features that can be configured into a card.
    /// The size of SCR register is 64 bits. This register is set in the factory by the microSD Memory Card manufacturer.
    SCR [
        /// SD Memory Card Physical Layer Specification version
        EMMC_SPEC OFFSET(0) NUMBITS(4) [
            /// Version 1.0-1.01
            EMMC_SPEC_1_101 = 0,
            /// Version 1.10
            EMMC_SPEC_11 = 1,
            /// ersion 2.00 or Version 3.00 (check bit EMMC_SPEC3)
            EMMC_SPEC_2_3 = 2,
        ],
        /// SCR structure version
        SCR_STRUCT OFFSET(4) NUMBITS(4) [
            /// SCR version 1.0
            SCR_VER_1 = 0,
        ],
        /// SD Bus width
        BUS_WIDTH OFFSET(8) NUMBITS(4) [
            /// Card supports bus width 1
            BUS_WIDTH_1 = 1,
            /// Card supports bus width 4
            BUS_WIDTH_4 = 4,
            /// Card supports bus widths - 1 and 4
            BUS_WIDTH_1_4 = 5,
        ],
        /// Voltage window 2.9v to 3.0v
        EMMC_SECURITY OFFSET(12) NUMBITS(3) [
            /// No Security
            EMMC_SEC_NONE = 0,
            /// Security Not Used
            EMMC_SEC_NOT_USED = 1,
            /// SDSC Card (Security Version 1.01)
            EMMC_SEC_101 = 2,
            /// SDHC Card (Security Version 2.00)
            EMMC_SEC_2 = 3,
            /// SDXC Card (Security Version 3.xx)
            EMMC_SEC_3 = 4,
        ],
        /// Defines the data status after erase, whether it is 0 or 1
        DATA_AFTER_ERASE OFFSET(15) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved OFFSET(16) NUMBITS(3) [],
        /// Extended security
        EX_SECURITY OFFSET(19) NUMBITS(4) [
            /// No extended Security
            EX_SEC_NONE = 0,
        ],
        /// Spec. Version 3.00 or higher
        EMMC_SPEC3 OFFSET(23) NUMBITS(1) [],
        /// CMD support
        CMD_SUPPORT OFFSET(24) NUMBITS(2) [
            CMD_SUPP_SPEED_CLASS = 1,
            CMD_SUPP_SET_BLKCNT = 2,
        ],
        /// Write as zero read as don't care
        _reserved1 OFFSET(26) NUMBITS(38) [],
    ],
}

// The CID is Big Endian and the i.MX butchers it by not having CRC
// So the CID appears shifted 8 bits right with first 8 bits reading zero.

register_bitfields! {
    u32,

    /// The Card Identification (CID) register is 128 bits wide. It contains the card identification
    /// information used during the card identification phase. Every individual Read/Write (R/W) card
    /// has a unique identification number.
    ///
    /// - CID_RAW32_0 represents the first 32 bits.
    CID_RAW32_0 [
        /// Identifies the card OEM. The OID is assigned by the SD-3C, LLC
        OID_LO OFFSET(0) NUMBITS(8) [],
        /// Identifies the card OEM. The OID is assigned by the SD-3C, LLC
        OID_HI OFFSET(8) NUMBITS(8) [],
        /// Manufacturer ID, assigned by the SD-3C, LLC
        MID OFFSET(16) NUMBITS(8) [],
        /// reserved
        _reserved OFFSET(24) NUMBITS(8) [],
    ],
    /// The Card Identification (CID) register is 128 bits wide. It contains the card identification
    /// information used during the card identification phase. Every individual Read/Write (R/W) card
    /// has a unique identification number.
    ///
    /// - CID_RAW32_1 represents the 2nd slice of 32 bits.
    CID_RAW32_1 [
        /// Product name character four
        ProdName4 OFFSET(0) NUMBITS(8) [],
        /// Product name character three
        ProdName3 OFFSET(8) NUMBITS(8) [],
        /// Product name character two
        ProdName2 OFFSET(16) NUMBITS(8) [],
        /// Product name character one
        ProdName1 OFFSET(24) NUMBITS(8) [],
    ],
    /// The Card Identification (CID) register is 128 bits wide. It contains the card identification
    /// information used during the card identification phase. Every individual Read/Write (R/W) card
    /// has a unique identification number.
    ///
    /// - CID_RAW32_2 represents the 3rd slice of 32 bits.
    CID_RAW32_2 [
        /// Serial number upper 16 bits
        SerialNumHi OFFSET(0) NUMBITS(16) [],
        /// Product revision low value in BCD
        ProdRevLo OFFSET(16) NUMBITS(4) [],
        /// Product revision high value in BCD
        ProdRevHi OFFSET(20) NUMBITS(4) [],
        /// Product name character five
        ProdName5 OFFSET(24) NUMBITS(8) [],
    ],
    /// The Card Identification (CID) register is 128 bits wide. It contains the card identification
    /// information used during the card identification phase. Every individual Read/Write (R/W) card
    /// has a unique identification number.
    ///
    /// - CID_RAW32_3 represents the 4th slice of 32 bits.
    CID_RAW32_3 [
        /// Manufacturing date month (1=Jan, 2=Feb, 3=Mar etc)
        ManufactureMonth OFFSET(0) NUMBITS(4) [],
        /// Manufacturing dateyear (offset from 2000 .. 1=2001,2=2002,3=2003 etc)
        ManufactureYear OFFSET(4) NUMBITS(8) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(12) NUMBITS(4) [],
        /// Serial number lower 16 bits
        SerialNumLo OFFSET(16) NUMBITS(16) [],
    ],
}

// The CID is Big Endian and the i.MX butchers it by not having CRC
// So the CID appears shifted 8 bits right with first 8 bits reading zero.

register_bitfields! {
    u32,

    /// The Card-Specific Data register provides information regarding access to the card contents. The CSD defines the
    /// data format, error correction type, maximum data access time, whether the DSR register can be used, etc. The
    /// programmable part of the register can be changed by CMD27.
    ///
    /// - CEMMC_RAW32_0 represents the first 32 bits.
    CEMMC_RAW32_0 [
        /// trans_speed as on SD CSD bits
        TRAN_SPEED OFFSET(0) NUMBITS(8) [],
        /// nsac as on SD CSD bits
        NSAC OFFSET(16) NUMBITS(8) [],
        /// taac as on SD CSD bits
        TAAC OFFSET(8) NUMBITS(8) [],
        /// CSD version as on SD CSD bits
        SPEC_VERS OFFSET(24) NUMBITS(6) [],
        /// CSD Structure Version as on SD CSD bits
        CEMMC_STRUCTURE OFFSET(30) NUMBITS(2) [
            /// enum CSD version 1.0 - 1.1, Version 2.00/Standard Capacity
            CEMMC_VERSION_1 = 0,
            /// enum CSD cersion 2.0, Version 2.00/High Capacity and Extended Capacity
            CEMMC_VERSION_2 = 1,
        ],
    ],
    /// The Card-Specific Data register provides information regarding access to the card contents. The CSD defines the
    /// data format, error correction type, maximum data access time, whether the DSR register can be used, etc. The
    /// programmable part of the register can be changed by CMD27.
    ///
    /// - CEMMC_RAW32_1 represents the 2nd slice of 32 bits.
    CEMMC_RAW32_1 [
        /// Version 1 C_Size as on SD CSD bits
        CSIZE OFFSET(0) NUMBITS(12) [],
        /// dsr_imp as on SD CSD bits
        DSR_IMP OFFSET(12) NUMBITS(1) [],
        /// read_blk_misalign as on SD CSD bits
        READ_BLK_MISALIGN OFFSET(13) NUMBITS(1) [],
        /// write_blk_misalign as on SD CSD bits
        WRITE_BLK_MISALIGN OFFSET(14) NUMBITS(1) [],
        /// read_bl_partial as on SD CSD bits
        READ_BL_PARTIAL OFFSET(15) NUMBITS(1) [],
        /// read_bl_len as on SD CSD bits
        READ_BL_LEN OFFSET(16) NUMBITS(4) [],
        /// ccc as on SD CSD bits
        CCC OFFSET(20) NUMBITS(12) [],

    ],
    /// The Card-Specific Data register provides information regarding access to the card contents. The CSD defines the
    /// data format, error correction type, maximum data access time, whether the DSR register can be used, etc. The
    /// programmable part of the register can be changed by CMD27.
    ///
    /// - CEMMC_RAW32_2 represents the 3rd slice of 32 bits.
    CEMMC_RAW32_2 [
        /// 2 Spares bit unused
        _reserved OFFSET(0) NUMBITS(2) [],
        /// sector_size as on SD CSD bits
        SECTOR_SIZE OFFSET(2) NUMBITS(7) [],
        /// Product revision high value in BCD
        ERASE_BLK_EN OFFSET(9) NUMBITS(1) [],
        /// Version 2 C_Size, reserved for CSD ver 2.0 size match
        _reserved1 OFFSET(10) NUMBITS(7) [],
        /// Version 2 C_Size, c_size_mult as on SD CSD bits
        C_SIZE_MULT OFFSET(17) NUMBITS(3) [],
        /// Version 2 C_Size, vdd_w_curr_max as on SD CSD bits
        VDD_W_CURR_MAX OFFSET(20) NUMBITS(3) [],
        /// Version 2 C_Size, vdd_w_curr_min as on SD CSD bits
        VDD_W_CURR_MIN OFFSET(23) NUMBITS(3) [],
        /// Version 2 C_Size, vdd_r_curr_max as on SD CSD bits
        VDD_R_CURR_MAX OFFSET(26) NUMBITS(3) [],
        /// Version 2 C_Size, vdd_r_curr_min as on SD CSD bits
        VDD_R_CURR_MIN OFFSET(29) NUMBITS(3) [],
    ],
    /// The Card-Specific Data register provides information regarding access to the card contents. The CSD defines the
    /// data format, error correction type, maximum data access time, whether the DSR register can be used, etc. The
    /// programmable part of the register can be changed by CMD27.
    ///
    /// - CID_RAW32_3 represents the 4th slice of 32 bits.
    CEMMC_RAW32_3 [
        /// 1 spare bit unused
        _reserved OFFSET(0) NUMBITS(1) [],
        /// ecc as on SD CSD bits
        ECC OFFSET(1) NUMBITS(2) [],
        /// File format as on SD CSD bits
        FILE_FORMAT OFFSET(2) NUMBITS(2) [
            /// SD card is FAT with partition table
            FAT_PARTITION_TABLE = 0,
            /// enum SD card is FAT with no partition table
            FAT_NO_PARTITION_TABLE = 1,
            /// SD card file system is universal
            FS_UNIVERSAL = 2,
            /// SD card file system is other
            FS_OTHER = 3,
        ],
        /// tmp_write_protect as on SD CSD bits
        TMP_WRITE_PROTECT OFFSET(5) NUMBITS(1) [],
        /// perm_write_protect as on SD CSD bits
        PERM_WRITE_PROTECT OFFSET(6) NUMBITS(1) [],
        /// copy as on SD CSD bits
        COPY OFFSET(7) NUMBITS(1) [],
        /// file_format_grp as on SD CSD bits
        FILE_FORMAT_GRP OFFSET(8) NUMBITS(1) [],
        /// default_ECC as on SD CSD bits
        DEFAULT_ECC OFFSET(9) NUMBITS(5) [],
        /// write_bl_partial as on SD CSD bits
        WRITE_BL_PARTIAL OFFSET(14) NUMBITS(1) [],
        /// write_bl_enable as on SD CSD bits
        WRITE_BL_EN OFFSET(15) NUMBITS(4) [],
        /// r2w_factor as on SD CSD bits
        R2W_FACTOR OFFSET(19) NUMBITS(3) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(22) NUMBITS(2) [],
        /// wp_grp_enable as on SD CSD bits
        WP_GRP_ENABLE OFFSET(24) NUMBITS(1) [],
        /// wp_grp_size as on SD CSD bits
        WP_GRP_SIZE OFFSET(25) NUMBITS(7) [],
    ],
}

use {
    CEMMC_RAW32_0::*, CEMMC_RAW32_1::*, CEMMC_RAW32_2::*, CEMMC_RAW32_3::*, CID_RAW32_0::*,
    CID_RAW32_1::*, CID_RAW32_2::*, CID_RAW32_3::*,
};

#[rustfmt::skip]
mod uSDHC_constants {
    /*--------------------------------------------------------------------------
                INTERRUPT REGISTER TURNED  INTO MASK BIT DEFINITIONS
    --------------------------------------------------------------------------*/

    pub const INT_CMD_DONE      : usize = 0x00000001; // CMD_DONE bit in register

    /*--------------------------------------------------------------------------
    						  SD CARD FREQUENCIES							   
    --------------------------------------------------------------------------*/
    pub const FREQ_SETUP  : usize = 400_000; // 400 Khz
    pub const FREQ_NORMAL : usize = 50_000_000; // 50 Mhz
    pub const BASE_CLOCK  : usize = 400_000_000; // 400 Mhz


    /*--------------------------------------------------------------------------
    						  CMD 41 BIT SELECTIONS							    
    --------------------------------------------------------------------------*/
    pub const ACMD41_HCS        : usize = 0x40000000;
    pub const ACMD41_SDXC_POWER : usize = 0x10000000;
    pub const ACMD41_S18R       : usize = 0x04000000;
    pub const ACMD41_VOLTAGE    : usize = 0x00ff8000;
    //(ACMD41_HCS|ACMD41_SDXC_POWER|ACMD41_VOLTAGE|ACMD41_S18R)
    pub const ACMD41_ARG_HC     : usize = ACMD41_HCS | ACMD41_SDXC_POWER | ACMD41_VOLTAGE;
    pub const ACMD41_ARG_SC     : usize = ACMD41_VOLTAGE; //(ACMD41_VOLTAGE|ACMD41_S18R)
}

/// Sd Card command Record
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Command<'a> {
    cmd_name: &'a str,
    cmd_code: LocalRegisterCopy<u32, CMD_XFR_TYP::Register>,
    use_rca: u16, // 0-bit of cmd is the rca-bit, subsequent 1-15 bits are reserved i.e. write as zero read as don't care.
    delay: u16,   // next 16-31 bits contain delay to apply after command.
}

impl<'a> Command<'a> {
    const fn new() -> Self {
        Command {
            cmd_name: " ",
            cmd_code: LocalRegisterCopy::new(0x0),
            use_rca: 0,
            delay: 0,
        }
    }
}

/// Result of an Sd operation
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SdResult {
    /// No error
    SdOk,
    /// General non specific SD error  
    SdError,
    /// SD Timeout error
    SdTimeout,
    /// SD Card is busy
    SdBusy,
    /// SD Card did not respond
    SdNoResp,
    /// SD Card did not reset  
    SdErrorReset,
    /// SD Card clock change failed
    SdErrorClock,
    /// SD Card does not support requested voltage
    SdErrorVoltage,
    /// SD Card app command failed
    SdErrorAppCmd,
    /// SD Card not present
    SdCardAbsent,
    SdReadError,
    SdMountFail,
    SdCardState(u32),
    None,
}

/// Enumerate the type of SD Card
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum SdCardType {
    TypeUnknown,
    TypeMmc,
    Type1,
    Type2Sc,
    Type2Hc,
}

static SD_TYPE_NAME: [&str; 5] = ["Unknown", "MMC", "Type 1", "Type 2 SC", "Type 2 HC"];

/// List of supported SD commands
#[derive(Debug, PartialEq, PartialOrd)]
pub enum SdCardCommands {
    GoIdleState,
    SendOpCond,
    AllSendCid,
    SendRelAddr,
    SetDsr,
    SwitchFunc,
    CardSelect,
    SendIfCond,
    SendCsd,
    SendCid,
    VoltageSwitch,
    StopTrans,
    SendStatus,
    GoInactive,
    SetBlocklen,
    ReadSingle,
    ReadMulti,
    SendTuning,
    SpeedClass,
    SetBlockcnt,
    WriteSingle,
    WriteMulti,
    ProgramCsd,
    SetWritePr,
    ClrWritePr,
    SndWritePr,
    EraseWrSt,
    EraseWrEnd,
    Erase,
    LockUnlock,
    AppCmd,
    AppCmdRca,
    GenCmd,
    // Commands hereafter require APP_CMD.
    AppCmdStart,
    SetBusWidth,
    EmmcStatus,
    SendNumWrbl,
    SendNumErs,
    AppSendOpCond,
    SetClrDet,
    SendScr,
}

impl SdCardCommands {
    fn get_cmd(&self) -> Command<'static> {
        let mut cmd = LocalRegisterCopy::new(0u32);
        match self {
            Self::GoIdleState => Command {
                cmd_name: "GO_IDLE_STATE",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x00) + CMD_XFR_TYP::RSPTYP::CMD_NO_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SendOpCond => Command {
                cmd_name: "SEND OP COND",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x01)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::CCCEN::SET
                            + CMD_XFR_TYP::CICEN::SET,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::AllSendCid => Command {
                cmd_name: "ALL_SEND_CID",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x02)
                            + CMD_XFR_TYP::RSPTYP::CMD_136BIT_RESP
                            + CMD_XFR_TYP::CCCEN::SET, // + CMD_XFR_TYP::CICEN::SET,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SendRelAddr => Command {
                cmd_name: "SEND_REL_ADDR",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x03) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SetDsr => Command {
                cmd_name: "SET_DSR",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x04) + CMD_XFR_TYP::RSPTYP::CMD_NO_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SwitchFunc => Command {
                cmd_name: "SWITCH_FUNC",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x06) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::CardSelect => Command {
                cmd_name: "CARD_SELECT",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x07) + CMD_XFR_TYP::RSPTYP::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SendIfCond => Command {
                cmd_name: "SEND_IF_COND",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x08)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::CCCEN::SET
                            + CMD_XFR_TYP::CICEN::SET,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 100,
            },
            Self::SendCsd => Command {
                cmd_name: "SEND_CSD",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x09) + CMD_XFR_TYP::RSPTYP::CMD_136BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SendCid => Command {
                cmd_name: "SEND_CID",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x0a) + CMD_XFR_TYP::RSPTYP::CMD_136BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::VoltageSwitch => Command {
                cmd_name: "VOLT_SWITCH",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x0b) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::StopTrans => Command {
                cmd_name: "STOP_TRANS",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x0c) + CMD_XFR_TYP::RSPTYP::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SendStatus => Command {
                cmd_name: "SEND_STATUS",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x0d)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::CCCEN::SET
                            + CMD_XFR_TYP::CICEN::SET
                            + CMD_XFR_TYP::DPSEL::SET,
                    );
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::GoInactive => Command {
                cmd_name: "GO_INACTIVE",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x0f) + CMD_XFR_TYP::RSPTYP::CMD_NO_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SetBlocklen => Command {
                cmd_name: "SET_BLOCKLEN",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x10) + CMD_XFR_TYP::RSPTYP::CMD_NO_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ReadSingle => Command {
                cmd_name: "READ_SINGLE",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x11)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::DPSEL.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ReadMulti => Command {
                cmd_name: "READ_MULTI",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x12)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::DPSEL.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SendTuning => Command {
                cmd_name: "SEND_TUNING",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x13) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SpeedClass => Command {
                cmd_name: "SPEED_CLASS",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x14) + CMD_XFR_TYP::RSPTYP::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SetBlockcnt => Command {
                cmd_name: "SET_BLOCKCNT",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x17) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::WriteSingle => Command {
                cmd_name: "WRITE_SINGLE",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x18)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::DPSEL.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::WriteMulti => Command {
                cmd_name: "WRITE_MULTI",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x19)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::DPSEL.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ProgramCsd => Command {
                cmd_name: "PROGRAM_CSD",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x1b) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SetWritePr => Command {
                cmd_name: "SET_WRITE_PR",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x1c) + CMD_XFR_TYP::RSPTYP::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ClrWritePr => Command {
                cmd_name: "CLR_WRITE_PR",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x1d) + CMD_XFR_TYP::RSPTYP::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SndWritePr => Command {
                cmd_name: "SND_WRITE_PR",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x1e) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::EraseWrSt => Command {
                cmd_name: "ERASE_WR_ST",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x20) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::EraseWrEnd => Command {
                cmd_name: "ERASE_WR_END",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x21) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::Erase => Command {
                cmd_name: "ERASE",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x26) + CMD_XFR_TYP::RSPTYP::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::LockUnlock => Command {
                cmd_name: "LOCK_UNLOCK",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x2a) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::AppCmd => Command {
                cmd_name: "APP_CMD",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x37)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::CCCEN::SET
                            + CMD_XFR_TYP::CICEN::SET,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 100,
            },
            Self::AppCmdRca => Command {
                cmd_name: "APP_CMD_RCA",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x37) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::GenCmd => Command {
                cmd_name: "GEN_CMD",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x38) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            // Commands hereafter require APP_CMD.
            Self::SetBusWidth => Command {
                cmd_name: "SET_BUS_WIDTH",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x06) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::EmmcStatus => Command {
                cmd_name: "EMMC_STATUS",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x0d) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SendNumWrbl => Command {
                cmd_name: "SEND_NUM_WRBL",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x16) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SendNumErs => Command {
                cmd_name: "SEND_NUM_ERS",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x17) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::AppSendOpCond => Command {
                cmd_name: "APP_SEND_OP_COND",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x29) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 1000,
            },
            Self::SetClrDet => Command {
                cmd_name: "SET_CLR_DET",
                cmd_code: {
                    cmd.write(CMD_XFR_TYP::CMDINX.val(0x2a) + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SendScr => Command {
                cmd_name: "SEND_SCR",
                cmd_code: {
                    cmd.write(
                        CMD_XFR_TYP::CMDINX.val(0x33)
                            + CMD_XFR_TYP::RSPTYP::CMD_48BIT_RESP
                            + CMD_XFR_TYP::DPSEL.val(1)
                            + CMD_XFR_TYP::CCCEN::SET
                            + CMD_XFR_TYP::CICEN::SET,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            _ => unimplemented!(),
        }
    }
}

/// CID representation
#[repr(C)]
pub struct CID {
    cid0: LocalRegisterCopy<u32, CID_RAW32_0::Register>,
    cid1: LocalRegisterCopy<u32, CID_RAW32_1::Register>,
    cid2: LocalRegisterCopy<u32, CID_RAW32_2::Register>,
    cid3: LocalRegisterCopy<u32, CID_RAW32_3::Register>,
}

impl CID {
    const fn new() -> Self {
        CID {
            cid0: LocalRegisterCopy::new(0x0),
            cid1: LocalRegisterCopy::new(0x0),
            cid2: LocalRegisterCopy::new(0x0),
            cid3: LocalRegisterCopy::new(0x0),
        }
    }
}

/// CSD representation
#[repr(C)]
pub struct CSD {
    csd0: LocalRegisterCopy<u32, CEMMC_RAW32_0::Register>,
    csd1: LocalRegisterCopy<u32, CEMMC_RAW32_1::Register>,
    csd2: LocalRegisterCopy<u32, CEMMC_RAW32_2::Register>,
    csd3: LocalRegisterCopy<u32, CEMMC_RAW32_3::Register>,
}

impl CSD {
    const fn new() -> Self {
        CSD {
            csd0: LocalRegisterCopy::new(0x0),
            csd1: LocalRegisterCopy::new(0x0),
            csd2: LocalRegisterCopy::new(0x0),
            csd3: LocalRegisterCopy::new(0x0),
        }
    }
}

/// Sd card description record
#[rustfmt::skip]
#[repr(C)]
pub struct SdDescriptor<'a> {
    cid: CID,                                   // Card cid
    csd: CSD,                                   // Card csd
    scr: LocalRegisterCopy<u64, SCR::Register>, // Card scr
    card_capacity: u64,                         // Card capacity expanded .. calculated from card details
    sd_card_type: SdCardType,                   // Card type
    rca: u32,                                   // Card rca
    ocr: LocalRegisterCopy<u32, OCR::Register>, // Card ocr
    status: u32,                                // Card last status
    last_cmd: Command<'a>,
}

impl<'a> SdDescriptor<'a> {
    const fn new() -> Self {
        SdDescriptor {
            cid: CID::new(),
            csd: CSD::new(),
            scr: LocalRegisterCopy::new(0x0),
            card_capacity: 0,
            sd_card_type: SdCardType::TypeUnknown,
            rca: 0,
            ocr: LocalRegisterCopy::new(0x0),
            status: 0,
            last_cmd: Command::new(),
        }
    }
}

/// Global storage - Sd card register and state data
static mut SD_CARD: SdDescriptor = SdDescriptor::new();

use crate::nxp::imx8mn::arch::timer::*;
use core::time::Duration;
use uSDHC_constants::*;

const R1_ERRORS_MASK: u32 = 0xfff9c004;
const ST_APP_CMD: u32 = 0x00000020;
const VENDORSPEC_INIT: u32 = 0x2000000b;
const TUNINIG_CTRL_INIT: u32 = 0x01222894;

/// Waits for the `delay` specified number of microseconds
pub fn timer_wait_micro(delay: u64) {
    time_manager().wait_for(Duration::from_micros(delay));
}

/// Gets current system counter value
fn timer_get_tick_count() -> u64 {
    time_manager().get_sys_tick_count()
}

/// Given two `tickcount` values, calculates microseconds between them.
fn tick_difference(start_time: u64, tick_count: u64) -> u64 {
    let tick_diff = tick_count - start_time;
    (tick_diff * time_manager().resolution().as_nanos() as u64) / 1000 // 1 ns == 1000 us
}

/// uSD Host controller.
pub struct UsdhController {
    registers: UsdhcRegisters,
}

impl UsdhController {
    /// Create an instance.
    ///
    /// **Safety**
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: UsdhcRegisters::new(mmio_start_addr),
        }
    }

    fn debug_response(&self, resp: SdResult) -> SdResult {
        info!(
            "uSDHC: PRES_STATE: 0x{:08x}, PROT_CTRL: 0x{:08x}, SYS_CTRL: 0x{:08x}, INT_STATUS: 0x{:08x}, \
            VEND_SPEC: 0x{:08x}\n",
            self.registers.PRES_STATE.get(),
            self.registers.PROT_CTRL.get(),
            self.registers.SYS_CTRL.get(),
            self.registers.INT_STATUS.get(),
            self.registers.VEND_SPEC.get(),
        );
        info!(
            "uSDHC: CMD {:?}, resp: {:?}, CMD_RSP3: 0x{:08x}, CMD_RSP2: 0x{:08x}, CMD_RSP1: 0x{:08x}, CMD_RSP0: 0x{:08x}\n",
            unsafe { SD_CARD.last_cmd.cmd_name },
            resp,
            self.registers.CMD_RSP3.get(),
            self.registers.CMD_RSP2.get(),
            self.registers.CMD_RSP1.get(),
            self.registers.CMD_RSP0.get()
        );
        return resp;
    }

    /// Set the SD clock to the given frequency (derived from the base clock)
    ///
    /// Returns:
    /// - SdErrorClock - A fatal error occurred setting the clock
    /// - SdOk - the clock was set to given frequency
    fn set_clock(&self, freq: u32) -> SdResult {
        let mut div = 1u32;
        let mut pre_div = 1u32;
        let sdhc_clk = BASE_CLOCK as u32;

        assert!(freq > 0);
        assert!(freq <= BASE_CLOCK as u32);

        while (sdhc_clk / (16 * pre_div) > freq && pre_div < 256) {
            pre_div *= 2;
        }

        while ((sdhc_clk / (div * pre_div)) > freq && div < 16) {
            div += 1;
        }

        pre_div >>= 1;
        div -= 1;

        self.registers.VEND_SPEC.modify(VEND_SPEC::CKEN::CLEAR);
        self.registers
            .SYS_CTRL
            .modify(SYS_CTRL::DVS.val(div) + SYS_CTRL::SDCLKFS.val(pre_div));

        /* Wait for clock to be stablized */
        // start with a time difference of zero.
        let mut td = 0;
        let mut start_time = 0;
        // Clock not stable yet
        while (!self.registers.PRES_STATE.is_set(PRES_STATE::SDSTB) && (td < 100)) {
            if start_time == 0 {
                start_time = timer_get_tick_count();
            } else {
                td = tick_difference(start_time, timer_get_tick_count());
            }
        }
        if (td >= 100) {
            // Timed out waiting for stability flag
            info!("Sd Error: timed out waiting for a stable clock.\n");

            return SdResult::SdErrorClock;
        }
        info!("Sd clock stablized in {:?}us", td);

        self.registers
            .VEND_SPEC
            .modify(VEND_SPEC::PEREN::SET + VEND_SPEC::CKEN::SET);

        if pre_div == 0 {
            pre_div = 1;
        } else {
            pre_div <<= 1;
        }
        div += 1;

        info!(
            "Prescaler = {:?}, Divisor = {:?}, Freq = {:?} Hz",
            pre_div,
            div,
            (BASE_CLOCK as u32 / (div * pre_div))
        );
        // Clock frequency is set
        return SdResult::SdOk;
    }

    /// Reset SD Host Controller.
    /// 
    /// Note: this method does not perform a hardware or a software reset. Apparently, it works without this 
    /// - **hardware reset OR power-cycle**: we toggle the `IPP_RST_N` bit(23) of SYS_CTRL as per the SD standard (i.e. 1ms high and 
    /// then 1ms low)
    /// - **software reset**: the reference manual says we must reset the uSDHC peripheral by setting the `RSTA` 
    /// bit (24) of SYS_CTRL register 
    ///
    /// but after weeks of debugging, I realized (maybe) this board does not need to be reset. In fact, if you try to reset 
    /// the card/controller with either a hardware or software reset, we run into sd-communication errors - strange. 
    /// 
    /// Helpful hint: Performing any of type (above mentioned) of resets results in the data and command line signals being pulled low
    /// I confirmed this via the PRES_STAT register DLSL and CLSL bits.  
    /// 
    /// Returns:
    /// - SdErrorReset - A fatal error occurred resetting the Sd card
    /// - SdOk - Sd card reset correctly
    fn reset_card(&self) -> SdResult {
        // Start without a hardware or software reset. See above
        self.registers.MMCBOOT.set(0);
        self.registers.MIXCTRL.set(0);
        self.registers.CLK_TUNE_CTRL_STS.set(0);
        // Disable DLL_CTRL delay line 
        self.registers.DLL_CTRL.set(0);

        // Set clock to setup frequency 
        // i.e. set to low frequency clock (400Khz)
        let mut resp = self.set_clock(FREQ_SETUP as u32);
        timer_wait_micro(100);
        if resp != SdResult::SdOk {
            return resp;
        }

        // if self.registers.PRES_STATE.is_set(PRES_STATE::CINST) {
        //     info!("card inserted...")
        // } else {
        //     info!("card not inserted...")
        // }

        // Configure IRQs
        self.registers.INT_STATUS_EN.modify(
            CCSEN::SET
                + TCSEN::SET
                + DINTSEN::SET
                + BRRSENN::SET
                + CINTSEN::SET
                + CTOESEN::SET
                + CCESEN::SET
                + CEBESEN::SET
                + CIESEN::SET
                + DTOESEN::SET
                + DCESEN::SET
                + DEBESEN::SET,
        );
        // Set INITA field to send 80 SD-clocks to the card. After the 80 clocks are sent, this field is self
        // cleared
        self.registers.SYS_CTRL.modify(SYS_CTRL::INITA::SET);
        while !self.registers.SYS_CTRL.is_set(SYS_CTRL::INITA) {
            cpu_core::nop()
        }
        // Set PROCTL reg to the default
        self.registers.PROT_CTRL.modify(
            PROT_CTRL::DTW::OneBitWide, // PROT_CTRL::EMODE::LittleEndianMode + PROT_CTRL::D3CD::CLEAR
                                        // + PROT_CTRL::DTW::FourBitWide,
        );

        // set timeout to maximum value
        self.registers.SYS_CTRL.modify(SYS_CTRL::DTOCV.val(0xf));
        /* set watermark level as 16 words (maximum) */
        self.registers
            .WTMK_LVL
            .modify(WTMK_LVL::RD_WML.val(0x10) + WTMK_LVL::WR_WML.val(0x10));
        
        // Reset our card structure entries
        unsafe {
            SD_CARD.rca = 0; // Zero rca
            SD_CARD.ocr.set(0); // Zero ocr
            SD_CARD.last_cmd = Command::new(); // Zero lastCmd
            SD_CARD.status = 0; // Zero status
            SD_CARD.sd_card_type = SdCardType::TypeUnknown; // Set card type unknown
        }
        

        // Send GO_IDLE_STATE to card
        resp = self.send_command(SdCardCommands::GoIdleState);
        timer_wait_micro(2000);

        return resp;
    }

    /// Wait for command completion, this method loops polling for the condition (for up to 10 milli seconds).
    ///
    /// Returns:
    /// - SdTimeout - Operation timed out
    /// - SdError - an identifiable error occurred
    /// - SdOk - the wait completed as requested
    fn wait_for_cc(&self) -> SdResult {
        let mut time_diff: u64 = 0;
        let mut start_time: u64 = 0;

        while !self.registers.INT_STATUS.is_set(INT_STATUS::CC) && (time_diff < 10000) {
            if start_time == 0 {
                start_time = timer_get_tick_count()
            } else {
                time_diff = tick_difference(start_time, timer_get_tick_count())
            }
        }
        // Fetch all the interrupt flags
        let int_status = self.registers.INT_STATUS.get();

        let mut int_error_status = LocalRegisterCopy::new(0u32);
        // either a command timeout or a command CRC error or command index error or command end bit error occured
        int_error_status.modify(
            INT_STATUS::CTOE::SET
                + INT_STATUS::CCE::SET
                + INT_STATUS::CIE::SET
                + INT_STATUS::CEBE::SET,
        );
        // No response recieved, timeout occurred
        if time_diff >= 10000 {
            info!(
                "Sd: Operation timed out, waiting for command completion, \
                PresentStatus: 0x{:08x}, intStatus: 0x{:08x}, Resp0: 0x{:08x}, timeDiff: {}\n",
                self.registers.PRES_STATE.get(),
                int_status,
                self.registers.CMD_RSP0.get(),
                time_diff
            );
            // Return Sd Timeout
            return SdResult::SdTimeout;
        } else if (int_status & int_error_status.get()) != 0 {
            info!(
                "Error: we got a response for the last cmd but it contains errors, \
               decode contents of interrupt status register for details\n \
               VendSpec: 0x{:08x}, SysCtrl: 0x{:08x}, ProtCtrl: 0x{:08x}, \
               PresentStatus: 0x{:08x}, intStatus: 0x{:08x}, Resp0: 0x{:08x}, Resp1: 0x{:08x}, Resp2: 0x{:08x}, \
               Resp3: 0x{:08x}, CC_bit set in {}us\n",
                self.registers.VEND_SPEC.get(),
                self.registers.SYS_CTRL.get(),
                self.registers.PROT_CTRL.get(),
                self.registers.PRES_STATE.get(),
                int_status,
                self.registers.CMD_RSP0.get(),
                self.registers.CMD_RSP1.get(),
                self.registers.CMD_RSP2.get(),
                self.registers.CMD_RSP3.get(),
                time_diff
            );
            self.registers.SYS_CTRL.modify(SYS_CTRL::RSTC::SET);
            while self.registers.SYS_CTRL.is_set(SYS_CTRL::RSTC) {}
            /* clear all irq status */
            self.registers.INT_STATUS.set(0xffffffff);
            return SdResult::SdError;
        }
        // Clear the CC bit - write 1 to clear
        self.registers.INT_STATUS.modify(INT_STATUS::CC::SET);
        return SdResult::SdOk;
    }

    /// Waits for up to 300ms for any command OR data transfer that may be in progress.
    ///
    /// Returns:
    /// - SdBusy - the command or data transfer was not completed within 300ms period
    /// - SdOk - the wait completed sucessfully
    fn wait_for_cmd_data(&self) -> SdResult {
        let mut time_diff: u64 = 0;
        let mut start_time: u64 = 0;

        /* check if we can issue command that uses cmd or data line */
        while (self
            .registers
            .PRES_STATE
            .matches_all(PRES_STATE::CDIHB::SET + PRES_STATE::CIHB::SET))
            && (time_diff < 10000)
        {
            if start_time == 0 {
                start_time = timer_get_tick_count();
            } else {
                time_diff = tick_difference(start_time, timer_get_tick_count());
            }
        }

        if (time_diff >= 10000) {
            info!(
                "Sd Error: waiting for bus to idle, PresentStatus: 0x{:08x}, IntStatus:0x{:08x}, Resp0: 0x{:08x}\n",
                self.registers.PRES_STATE.get(),
                self.registers.INT_STATUS.get(),
                self.registers.CMD_RSP0.get()
            );
            return SdResult::SdBusy;
        }
        while self.registers.PRES_STATE.matches_all(PRES_STATE::DLA::SET) {
            cpu_core::nop();
        }
        return SdResult::SdOk;
    }

    ///  Send command and handle response.
    fn sendcommand_p(&self, cmd: Command<'static>, arg: u32) -> SdResult {
        unsafe {
            SD_CARD.last_cmd = cmd;
        }
        /* clear all irq status */
        self.registers.INT_STATUS.set(0xffffffff);
        /* Wait for the bus to be idle */
        if self.wait_for_cmd_data() != SdResult::SdOk {
            return SdResult::SdBusy;
        }
        // #[cfg(feature = "log")]
        info!(
            "Sd: sending command, CMD_NAME: {:?}, CMD_CODE: 0x{:08x}, CMD_ARG: 0x{:08x}",
            cmd.cmd_name,
            cmd.cmd_code.get(),
            arg
        );

        /* Mask all irqs */
        self.registers.INT_SIGNAL_EN.set(0);
        timer_wait_micro(1000);
        // Set the argument and the command code. Some commands require a delay before reading the response
        self.registers.CMD_ARG.set(arg);
        self.registers.CMD_XFR_TYPE.set(cmd.cmd_code.get());
        // Wait for required delay
        if cmd.delay != 0 {
            timer_wait_micro(cmd.delay.into());
        };

        // Wait until we finish sending the command i.e. the command completion bit is set.
        let res = self.wait_for_cc();
        match res {
            SdResult::SdOk => {}
            _ => return res,
        };

        /* Get response from RESP0 */
        let resp0 = self.registers.CMD_RSP0.get();

        match cmd.cmd_code.read_as_enum(CMD_XFR_TYP::RSPTYP) {
            Some(CMD_XFR_TYP::RSPTYP::Value::CMD_NO_RESP) => {
                return SdResult::SdOk;
            }
            Some(CMD_XFR_TYP::RSPTYP::Value::CMD_BUSY48BIT_RESP) => unsafe {
                SD_CARD.status = resp0;
                // Store the card state.  Note that this is the state the card was in before the
                // command was accepted, not the new state.
                if resp0 & R1_ERRORS_MASK == 0 {
                    return SdResult::SdOk;
                } else {
                    info!("CMD_BUSY48BIT_RESP case");
                    return SdResult::SdCardState(resp0 & R1_ERRORS_MASK);
                }
            },
            // RESP0 contains card status, no other data from other RESPx registers.
            // Return value non-zero if any error flag in the status value.
            Some(CMD_XFR_TYP::RSPTYP::Value::CMD_48BIT_RESP) => {
                match cmd.cmd_code.read(CMD_XFR_TYP::CMDINX) {
                    // SEND_REL_ADDR command
                    0x03 => {
                        // RESP0 contains RCA and status bits 23,22,19,12:0
                        unsafe {
                            SD_CARD.rca = resp0 & 0xffff0000; // RCA[31:16] of response
                            SD_CARD.status = ((resp0 & 0x00001fff)) |		// 12:0 map directly to status 12:0
                            ((resp0 & 0x00002000) << 6) |				// 13 maps to status 19 ERROR
                            ((resp0 & 0x00004000) << 8) |				// 14 maps to status 22 ILLEGAL_COMMAND
                            ((resp0 & 0x00008000) << 8); // 15 maps to status 23 COM_CRC_ERROR
                        }
                        // Store the card state.  Note that this is the state the card was in before the
                        // command was accepted, not the new state.
                        unsafe {
                            if SD_CARD.status & R1_ERRORS_MASK == 0 {
                                return SdResult::SdOk;
                            } else {
                                info!("CMD_48BIT_RESP, 0x03 case");
                                return SdResult::SdCardState(SD_CARD.status & R1_ERRORS_MASK);
                            }
                        }
                    }
                    // SEND_IF_COND command
                    0x08 => {
                        // RESP0 contains voltage acceptance and check pattern, which should match
                        // the argument.
                        unsafe { SD_CARD.status = 0 };
                        if resp0 == arg {
                            return SdResult::SdOk;
                        } else {
                            return SdResult::SdError;
                        }
                        // RESP0 contains OCR register
                        // TODO: What is the correct time to wait for this?
                    }
                    // EMMC_SENDOPCOND command
                    0x29 => {
                        unsafe {
                            SD_CARD.status = 0;
                            SD_CARD.ocr.set(resp0);
                        }
                        return SdResult::SdOk;
                    }
                    // SEND_SCR command
                    0x33 => unsafe {
                        let mut scr_lo = 0;
                        let mut scr_hi = 0;
                        for (idx, word) in (0..2u8).enumerate() {
                            while !self.registers.INT_STATUS.is_set(INT_STATUS::BRR) {}
                            // clear BRR with w1c - write 1 to clear
                            self.registers.INT_STATUS.modify(INT_STATUS::BRR::SET);
                            // for non-DMA read transfers, the uSDHC module implements an internal buffer to
                            // transfer data efficiently.
                            match idx {
                                0 => scr_lo = self.registers.DATA_BUFF_ACC_PORT.get(),
                                1 => scr_hi = self.registers.DATA_BUFF_ACC_PORT.get(),
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        SD_CARD.scr.set(scr_lo as u64 | ((scr_hi as u64) << 32));
                        return SdResult::SdOk;
                    },
                    _ => {
                        unsafe {
                            SD_CARD.status = resp0;
                        }
                        // Store the card state.  Note that this is the state the card was in before the
                        // command was accepted, not the new state.
                        if resp0 & R1_ERRORS_MASK == 0 {
                            return SdResult::SdOk;
                        } else {
                            return SdResult::SdCardState(resp0 & R1_ERRORS_MASK);
                        }
                    }
                }
            }
            // RESP0..3 contains 128 bit CID or CSD shifted down by 8 bits as no CRC
            // Note: highest bits are in RESP3.
            Some(CMD_XFR_TYP::RSPTYP::Value::CMD_136BIT_RESP) => {
                unsafe {
                    SD_CARD.status = 0;
                }
                if cmd.cmd_code.read(CMD_XFR_TYP::CMDINX) == 0x09 {
                    self.unpack_csd(unsafe { &mut SD_CARD.csd });
                } else {
                    unsafe {
                        SD_CARD.cid.cid3.set(resp0);
                        SD_CARD.cid.cid2.set(self.registers.CMD_RSP1.get());
                        SD_CARD.cid.cid1.set(self.registers.CMD_RSP2.get());
                        SD_CARD.cid.cid0.set(self.registers.CMD_RSP3.get());
                    }
                }
                return SdResult::SdOk;
            }
            None => SdResult::SdError,
        }
    }

    /// Send APP command and handle response.    
    fn send_app_command(&self) -> SdResult {
        // If no RCA, send the APP_CMD and don't look for a response.
        if unsafe { SD_CARD.rca == 0 } {
            let resp = self.sendcommand_p(SdCardCommands::AppCmd.get_cmd(), 0x00000000);
            timer_wait_micro(100); // add a 100 us delay for cmds that automatically send APP_CMDs
                                   // info!(" no-rca APP_CMD result: {:?} ", resp);
                                   // If there is an RCA, include that in APP_CMD and check card accepted it.
        } else {
            let resp =
                self.sendcommand_p(SdCardCommands::AppCmdRca.get_cmd(), unsafe { SD_CARD.rca });
            match resp {
                SdResult::SdOk => {}
                _ => return self.debug_response(resp),
            }
            // Debug - check that status indicates APP_CMD accepted.
            if (unsafe { SD_CARD.status & ST_APP_CMD }) == 0 {
                return SdResult::SdError;
            };
        }
        SdResult::SdOk
    }

    /// Send a command with argument. APP_CMD sent automatically if required.
    fn send_command_a(&self, cmd_type: SdCardCommands, arg: u32) -> SdResult {
        // Issue APP_CMD if needed.
        let mut resp = SdResult::None;
        if cmd_type >= SdCardCommands::AppCmdStart && {
            resp = self.send_app_command();
            resp != SdResult::SdOk
        } {
            return self.debug_response(resp);
        }
        // Get the command and pass the argument through.
        resp = self.sendcommand_p(cmd_type.get_cmd(), arg);
        if resp != SdResult::SdOk {
            return resp;
        }

        // Check that APP_CMD was correctly interpreted.
        if unsafe {
            cmd_type >= SdCardCommands::AppCmdStart
                && SD_CARD.rca != 0
                && (SD_CARD.status & ST_APP_CMD) == 0
        } {
            return SdResult::SdErrorAppCmd;
        }

        return resp;
    }

    /// Send a command with no argument. RCA automatically added if required.
    /// APP_CMD sent automatically if required.
    fn send_command(&self, cmd_type: SdCardCommands) -> SdResult {
        let mut resp = SdResult::None;

        // Issue APP_CMD if needed.
        if cmd_type >= SdCardCommands::AppCmdStart && {
            resp = self.send_app_command();
            resp != SdResult::SdOk
        } {
            return self.debug_response(resp);
        }

        // Get the command and set RCA if required.
        let cmd = cmd_type.get_cmd();
        let mut arg = 0u32;
        if cmd.use_rca == 1 {
            unsafe { arg = SD_CARD.rca }
        }

        resp = self.sendcommand_p(cmd, arg);
        if resp != SdResult::SdOk {
            return resp;
        };

        // Check that APP_CMD was correctly interpreted.
        if unsafe {
            cmd_type >= SdCardCommands::AppCmdStart
                && SD_CARD.rca != 0
                && (SD_CARD.status & ST_APP_CMD) == 0
        } {
            return SdResult::SdErrorAppCmd;
        }
        return resp;
    }

    /// Decode CSD data for logging purposes.
    fn unpack_csd(&self, csd: &mut CSD) {
        let mut buffer: [u8; 16] = [0; 16];

        buffer[12..].copy_from_slice(&self.registers.CMD_RSP0.get().to_le_bytes());
        buffer[8..12].copy_from_slice(&self.registers.CMD_RSP1.get().to_le_bytes());
        buffer[4..8].copy_from_slice(&self.registers.CMD_RSP2.get().to_le_bytes());
        buffer[0..4].copy_from_slice(&self.registers.CMD_RSP3.get().to_le_bytes());

        // Display raw CSD - values of my SANDISK ultra 32GB shown under each
        info!(
            "CSD Contents : {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}\
            {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            buffer[3],
            buffer[2],
            buffer[1],
            buffer[0],
            buffer[7],
            buffer[6],
            buffer[5],
            buffer[4],
            /* 00 40 0e 00 32 5b 59 00  */
            buffer[11],
            buffer[10],
            buffer[9],
            buffer[8],
            buffer[15],
            buffer[14],
            buffer[13],
            buffer[12] /* 00 ed c8 7f 80 0a 40 40  */
        );

        // Populate CSD structure
        csd.csd0
            .modify(CEMMC_STRUCTURE.val(((buffer[2] & 0xc0) >> 6) as u32)); // @126-127 ** correct
        csd.csd0.modify(SPEC_VERS.val((buffer[2] & 0x3F) as u32)); // @120-125 ** correct
        csd.csd0.modify(TAAC.val(buffer[1] as u32)); // @112-119 ** correct
        csd.csd0.modify(NSAC.val(buffer[0] as u32)); // @104-111 ** correct
        csd.csd0.modify(TRAN_SPEED.val(buffer[7] as u32)); // @96-103  ** correct
        csd.csd1
            .modify(CCC.val((((buffer[6] as u16) << 4) | ((buffer[5] & 0xf0) >> 4) as u16) as u32)); // @84-95   ** correct
        csd.csd1.modify(READ_BL_LEN.val((buffer[5] & 0x0f) as u32)); // @80-83   ** correct
        csd.csd1
            .modify(READ_BL_PARTIAL.val((if buffer[4] & 0x80 != 0 { 1 } else { 0 }) as u32)); // @79		** correct
        csd.csd1
            .modify(WRITE_BLK_MISALIGN.val((if buffer[4] & 0x40 != 0 { 1 } else { 0 }) as u32)); // @78      ** correct
        csd.csd1
            .modify(READ_BLK_MISALIGN.val((if buffer[4] & 0x20 != 0 { 1 } else { 0 }) as u32)); // @77		** correct
        csd.csd1
            .modify(DSR_IMP.val((if buffer[4] & 0x10 != 0 { 1 } else { 0 }) as u32)); // @76		** correct

        if csd.csd0.matches_all(CEMMC_STRUCTURE::CEMMC_VERSION_2) {
            // CSD VERSION 2.0
            let mut card_capacity =
                ((buffer[11] & 0x3F) as u32) << 16 | (buffer[10] as u32) << 8 | buffer[9] as u32; // @55-48, @63-56, @69-64
            csd.csd2.set(card_capacity);
            unsafe {
                SD_CARD.card_capacity = csd.csd2.get() as u64;
                SD_CARD.card_capacity *= 512 * 1024; // Calculate Card capacity
            }
        } else {
            // CSD VERSION 1.0
            let csize = ((buffer[4] & 0x03) as u32) << 8;
            csd.csd1.modify(CSIZE.val(csize));
            csd.csd1
                .modify(CSIZE.val(buffer[11] as u32 + csd.csd1.read(CSIZE)));
            csd.csd1.modify(CSIZE.val(csd.csd1.read(CSIZE) << 2));
            csd.csd1
                .modify(CSIZE.val(((buffer[10] & 0xc0) >> 6) as u32 + csd.csd1.read(CSIZE))); // @62-73

            csd.csd2
                .modify(VDD_R_CURR_MIN.val((buffer[10] & 0x38) as u32 >> 3)); // @59-61
            csd.csd2
                .modify(VDD_R_CURR_MAX.val((buffer[10] & 0x07) as u32)); // @56-58
            csd.csd2
                .modify(VDD_W_CURR_MIN.val((buffer[9] & 0xe0) as u32 >> 5)); // @53-55
            csd.csd2
                .modify(VDD_W_CURR_MAX.val((buffer[9] & 0x1c) as u32 >> 2)); // @50-52

            csd.csd2.modify(
                C_SIZE_MULT.val((((buffer[9] & 0x03) << 1) | ((buffer[8] & 0x80) >> 7)) as u32),
            ); // @47-49
            unsafe {
                SD_CARD.card_capacity = ((csd.csd1.read(CSIZE) + 1)
                    * (1 << (csd.csd2.read(C_SIZE_MULT) + 2))
                    * (1 << csd.csd1.read(READ_BL_LEN)))
                    as u64;
            }
        }

        csd.csd2
            .modify(ERASE_BLK_EN.val(((buffer[8] & 0x40) >> 6) as u32)); // @46
        csd.csd2
            .modify(SECTOR_SIZE.val((((buffer[15] & 0x80) >> 1) | (buffer[8] & 0x3F)) as u32)); // @39-45
        csd.csd3.modify(WP_GRP_SIZE.val((buffer[15] & 0x7f) as u32)); // @32-38
        csd.csd3
            .modify(WP_GRP_ENABLE.val((if buffer[14] & 0x80 != 0 { 1 } else { 0 }) as u32)); // @31
        csd.csd3
            .modify(DEFAULT_ECC.val(((buffer[14] & 0x60) >> 5) as u32)); // @29-30
        csd.csd3
            .modify(R2W_FACTOR.val(((buffer[14] & 0x1c) >> 2) as u32)); // @26-28   ** correct
        csd.csd3.modify(
            WRITE_BL_EN.val((((buffer[14] & 0x03) << 2) | ((buffer[13] & 0xc0) >> 6)) as u32),
        ); // @22-25   **correct
        csd.csd3
            .modify(WRITE_BL_PARTIAL.val((if buffer[13] & 0x20 != 0 { 1 } else { 0 }) as u32)); // @21
        csd.csd3
            .modify(FILE_FORMAT_GRP.val((if buffer[12] & 0x80 != 0 { 1 } else { 0 }) as u32)); // @15
        csd.csd3
            .modify(COPY.val((if buffer[12] & 0x40 != 0 { 1 } else { 0 }) as u32)); // @14
        csd.csd3
            .modify(PERM_WRITE_PROTECT.val((if buffer[12] & 0x20 != 0 { 1 } else { 0 }) as u32)); // @13
        csd.csd3
            .modify(TMP_WRITE_PROTECT.val((if buffer[12] & 0x10 != 0 { 1 } else { 0 }) as u32)); // @12
        csd.csd3
            .modify(FILE_FORMAT.val(((buffer[12] & 0x0c) >> 2) as u32)); // @10-11    **correct
        csd.csd3.modify(ECC.val((buffer[12] & 0x03) as u32)); // @8-9      **corrrect

        info!("cemmc_structure={:?}, spec_vers={:?}, taac=0x{:02X}, nsac=0x{:02X}, tran_speed=0x{:02X},\
             ccc=0x{:04X}, read_bl_len=0x{:02X}, read_bl_partial={:0b}b, write_blk_misalign={:0b}b,\
             read_blk_misalign={:0b}b, dsr_imp={:0b}b, sector_size =0x{:02X}, erase_blk_en={:0b}b",
            csd.csd0.read(CEMMC_STRUCTURE),
            csd.csd0.read(SPEC_VERS),
            csd.csd0.read(TAAC),
            csd.csd0.read(NSAC),
            csd.csd0.read(TRAN_SPEED),
            csd.csd1.read(CCC),
            csd.csd1.read(READ_BL_LEN),
            csd.csd1.read(READ_BL_PARTIAL),
            csd.csd1.read(WRITE_BLK_MISALIGN),
            csd.csd1.read(READ_BLK_MISALIGN),
            csd.csd1.read(DSR_IMP),
            csd.csd2.read(SECTOR_SIZE),
            csd.csd2.read(ERASE_BLK_EN)
        );
        if csd.csd0.matches_all(CEMMC_STRUCTURE::CEMMC_VERSION_2) {
            info!(
                "CSD 2.0: ver2_c_size = 0x{:02X}, card capacity: {:?} bytes or {:.02}GiB",
                csd.csd2.get(),
                unsafe { SD_CARD.card_capacity },
                (unsafe { SD_CARD.card_capacity } as f32 / (1000.0 * 1000.0 * 1000.0)),
            );
        } else {
            info!(
                "CSD 1.0: c_size = {:?}, c_size_mult={:?}, card capacity: {:?}, \
                vdd_r_curr_min = {:?}, vdd_r_curr_max={:?}, vdd_w_curr_min = {:?}, \
                vdd_w_curr_max={:?}",
                csd.csd1.read(CSIZE),
                csd.csd2.read(C_SIZE_MULT),
                unsafe { SD_CARD.card_capacity },
                csd.csd2.read(VDD_R_CURR_MIN),
                csd.csd2.read(VDD_R_CURR_MAX),
                csd.csd2.read(VDD_W_CURR_MIN),
                csd.csd2.read(VDD_W_CURR_MAX)
            );
        }
        info!(
            "wp_grp_size=0x{:07b}b, wp_grp_enable={:0b}b, default_ecc={:02b}b, r2w_factor={:03b}b, write_bl_len=0x{:02X}, \
            write_bl_partial={:0b}b, file_format_grp={:0b}, copy={:0b}b, perm_write_protect={:0b}b, tmp_write_protect={:0b}b, \
            file_format={:0b}b ecc={:02b}b",
            csd.csd3.read(WP_GRP_SIZE),
            csd.csd3.read(WP_GRP_ENABLE),
            csd.csd3.read(DEFAULT_ECC),
            csd.csd3.read(R2W_FACTOR),
            csd.csd3.read(WRITE_BL_EN),
            csd.csd3.read(WRITE_BL_PARTIAL),
            csd.csd3.read(FILE_FORMAT_GRP),
            csd.csd3.read(COPY),
            csd.csd3.read(PERM_WRITE_PROTECT),
            csd.csd3.read(TMP_WRITE_PROTECT),
            csd.csd3.read(FILE_FORMAT),
            csd.csd3.read(ECC)
        );
    }

    /// Send APP_SEND_OP_COND with the given argument. This is used for both SC and HC cards based on the parameter.
    fn app_send_op_cond(&self, arg: u32) -> SdResult {
        // Send APP_SEND_OP_COND with the given argument (for SC or HC cards).
        // Note: The host shall set ACMD41 timeout more than 1 second to avoid re-issuing ACMD41.
        // This command takes a while and is time-sensitive.

        // A tip: adding debug/print statements after issuing this cmd may seem like the cmd executes successfully.
        // However, removing them (i.e. `debug prints`) later can give us errors. Its probably because we waited a bit longer
        // while printing. Note: the above does NOT apply if you use the impl as-is.

        // In other words- issuing `APP_SEND_OP_COND`, will trigger an APP_CMD prior to sending out APP_SEND_OP_COND.
        // We must ensure a 100us delay between the 2 commands.
        let mut resp = self.send_command_a(SdCardCommands::AppSendOpCond, arg);
        if resp != SdResult::SdOk && resp != SdResult::SdTimeout {
            // #[cfg(feature = "log")]
            info!("{:?}: ACMD41 returned non-timeout error \n", resp);

            return resp;
        }
        let mut retries = 6u8;
        while unsafe { SD_CARD.ocr.read(OCR::card_power_up_busy) == 0 } && retries != 0 {
            timer_wait_micro(400000);
            resp = self.send_command_a(SdCardCommands::AppSendOpCond, arg);
            if resp != SdResult::SdOk && resp != SdResult::SdTimeout {
                // #[cfg(feature = "log")]
                info!("{:?}: ACMD41 returned non-timeout error \n", resp);

                return resp;
            }
            retries -= 1;
        }

        // Return timeout error if still not busy.
        if unsafe { SD_CARD.ocr.read(OCR::card_power_up_busy) == 0 } {
            return SdResult::SdTimeout;
        }

        // this i.MX driver only supports 3.3v Sd's - so check voltage value is around 3.3v.
        if unsafe {
            SD_CARD.ocr.read(OCR::voltage3v2to3v3) == 0
                && SD_CARD.ocr.read(OCR::voltage3v3to3v4) == 0
        } {
            return SdResult::SdErrorVoltage;
        }

        return SdResult::SdOk;
    }

    /// Read card's SCR. APP_CMD sent automatically if required.
    /// 
    /// TODO: Find out why we get a timeout error when we send SetBlocklen (CMD 16) before issuing the SCR.
    fn sd_read_scr(&self) -> SdResult {
        // Send set block length command
        // let resp = self.send_command_a(SdCardCommands::SetBlocklen, 8);
        // if resp != SdResult::SdOk {
        //     return self.debug_response(resp);
        // }
        // enable MIXCTRL bitfield to transfer data from the SD card to uSDHC
        self.registers.MIXCTRL.modify(MIXCTRL::DTDSEL::SET);
        // Set BLKSIZE to 1 block of 8 bytes, send SEND_SCR command
        self.registers.BLK_ATT.modify(BLK_ATT::BLKSIZE.val(8));

        let resp = self.send_command(SdCardCommands::SendScr);
        if resp != SdResult::SdOk {
            return self.debug_response(resp);
        }
        return SdResult::SdOk;
    }

    fn check_supported_volts(&self) -> SdResult {
        let host_cap = match (
            self.registers
                .HOST_CTRL_CAP
                .read_as_enum(HOST_CTRL_CAP::VS18),
            self.registers
                .HOST_CTRL_CAP
                .read_as_enum(HOST_CTRL_CAP::VS30),
            self.registers
                .HOST_CTRL_CAP
                .read_as_enum(HOST_CTRL_CAP::VS33),
        ) {
            (
                Some(HOST_CTRL_CAP::VS18::Value::Supported),
                Some(HOST_CTRL_CAP::VS30::Value::Supported),
                Some(HOST_CTRL_CAP::VS33::Value::Supported),
            ) => info!("uSDHC2 supports 1.8v, 3.0v, 3.3v ..."),
            _ => unimplemented!(),
        };
        SdResult::SdOk
    }

    /// Attempts to initialize the uSDHC and returns success/error status.
    /// This method should be called before any attempt to do anything with an Sd card.
    ///
    /// Returns:
    /// - SdOk - indicates the current card was successfully initialized.
    /// - !SdOk - initialization failed with code identifying error.
    pub fn init_usdhc(&self) -> SdResult {
        // check host's voltage support capabilities
        self.check_supported_volts();
        // Reset the card.
        let mut resp = self.reset_card();

        if (resp != SdResult::SdOk) {
            return resp;
        }
        // Send SEND_IF_COND,0x000001AA (CMD8) voltage range 0x1 check pattern 0xAA
        // If voltage range and check pattern don't match, look for older card.
        resp = self.send_command_a(SdCardCommands::SendIfCond, 0x000001AA);
        let _ = match resp {
            SdResult::SdOk => {
                // Card responded with voltage and check pattern.

                // Resolve voltage and check for high capacity card.
                resp = self.app_send_op_cond(ACMD41_ARG_HC as u32);
                if (resp != SdResult::SdOk) {
                    return self.debug_response(resp);
                }

                // Check for high or standard capacity.
                unsafe {
                    if (SD_CARD.ocr.read(OCR::card_capacity) != 0) {
                        SD_CARD.sd_card_type = SdCardType::Type2Hc;
                    } else {
                        SD_CARD.sd_card_type = SdCardType::Type2Sc;
                    }
                }
            }
            SdResult::SdBusy => return resp,
            // No response to SEND_IF_COND, treat as an old card.
            _ => {
                // info!(
                //     "{:?}: Send interface condition command (CMD8) returned an error \n",
                //     resp
                // );
                // return SdResult::SdError;
                // If there appears to be a command in progress, reset the card.
                resp = self.reset_card();
                if self.registers.PRES_STATE.is_set(PRES_STATE::CIHB) && (resp != SdResult::SdOk) {
                    return resp;
                }

                // wait(50);
                // Resolve voltage.
                resp = self.app_send_op_cond(ACMD41_ARG_SC as u32);
                if (resp != SdResult::SdOk) {
                    return self.debug_response(resp);
                }

                unsafe {
                    SD_CARD.sd_card_type = SdCardType::Type1;
                }
            }
        };

        // Send ALL_SEND_CID (CMD2)
        resp = self.send_command(SdCardCommands::AllSendCid);
        if (resp != SdResult::SdOk) {
            return self.debug_response(resp);
        }

        // Send SEND_REL_ADDR (CMD3)
        // TODO: In theory, loop back to SEND_IF_COND to find additional cards.
        resp = self.send_command(SdCardCommands::SendRelAddr);
        if (resp != SdResult::SdOk) {
            return self.debug_response(resp);
        }

        // Send SEND_CSD (CMD9) and parse the result.
        resp = self.send_command(SdCardCommands::SendCsd);
        if (resp != SdResult::SdOk) {
            return self.debug_response(resp);
        }

        // At this point, set the clock to full speed
        resp = self.set_clock(FREQ_NORMAL as u32);
        if (resp != SdResult::SdOk) {
            return self.debug_response(resp);
        }

        // Send CARD_SELECT  (CMD7)
        // TODO: Check card_is_locked status in the R1 response from CMD7 [bit 25], if so, use CMD42 to unlock
        // CMD42 structure [4.3.7] same as a single block write; data block includes
        // PWD setting mode, PWD len, PWD data.
        resp = self.send_command(SdCardCommands::CardSelect);
        if (resp != SdResult::SdOk) {
            return self.debug_response(resp);
        }

        // Get the SCR as well.
        // Need to do this before sending ACMD6 so that allowed bus widths are known.
        resp = self.sd_read_scr();
        if (resp != SdResult::SdOk) {
            return self.debug_response(resp);
        }

        // #[cfg(feature = "log")]
        match unsafe {
            SD_CARD
                .scr
                .read_as_enum::<SCR::BUS_WIDTH::Value>(SCR::BUS_WIDTH)
        } {
            Some(v) => {
                info!("SCR bus width: {:?}", v)
            }
            None => {
                info!("Unsupported bus width, we'll default to using a `1-bit` bus")
            }
        }
        // Send APP_SET_BUS_WIDTH (ACMD6)
        // If supported, set 4 bit bus width and update the CONTROL0 register.
        if let Some(SCR::BUS_WIDTH::Value::BUS_WIDTH_1_4) =
            unsafe { SD_CARD.scr.read_as_enum(SCR::BUS_WIDTH) }
        {
            resp = self.send_command_a(SdCardCommands::SetBusWidth, unsafe { SD_CARD.rca | 2 });
            if (resp != SdResult::SdOk) {
                return self.debug_response(resp);
            }
            self.registers.PROT_CTRL.modify(PROT_CTRL::DTW::FourBitWide);
            info!("Sd Bus width set to 4");
        };

        // Send SET_BLOCKLEN (CMD16)
        resp = self.send_command_a(SdCardCommands::SetBlocklen, 512);
        if (resp != SdResult::SdOk) {
            return self.debug_response(resp);
        }

        // Print out the CID having got this far.
        unsafe {
            let mut serial = SD_CARD.cid.cid2.read(CID_RAW32_2::SerialNumHi);
            serial <<= 16;
            serial |= SD_CARD.cid.cid3.read(CID_RAW32_3::SerialNumLo);

            info!(
                "Sd Card: {}, {}Mb, mfr_id: {}, '{}{}:{}{}{}{}{}', r{}.{}, mfr_date: {}/{}, serial: 0x{:08x}, RCA: 0x{:04x}",
                SD_TYPE_NAME[SD_CARD.sd_card_type as usize],
                SD_CARD.card_capacity >> 20,
                SD_CARD.cid.cid0.read(MID),
                SD_CARD.cid.cid0.read(OID_HI) as u8 as char,
                SD_CARD.cid.cid0.read(OID_LO) as u8 as char,
                SD_CARD.cid.cid1.read(ProdName1) as u8 as char,
                SD_CARD.cid.cid1.read(ProdName2) as u8 as char,
                SD_CARD.cid.cid1.read(ProdName3) as u8 as char,
                SD_CARD.cid.cid1.read(ProdName4) as u8 as char,
                SD_CARD.cid.cid2.read(ProdName5) as u8 as char,
                SD_CARD.cid.cid2.read(ProdRevHi),
                SD_CARD.cid.cid2.read(ProdRevLo),
                SD_CARD.cid.cid3.read(ManufactureMonth),
                2000 + SD_CARD.cid.cid3.read(ManufactureYear),
                serial,
                SD_CARD.rca >> 16
            );
        }

        return SdResult::SdOk;
    }
}

impl Debug for SCR::BUS_WIDTH::Value {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BUS_WIDTH_1 => {
                print!("WIDTH_1");
                Ok(())
            }
            Self::BUS_WIDTH_4 => {
                print!("WIDTH_4");
                Ok(())
            }
            Self::BUS_WIDTH_1_4 => {
                print!("WIDTH_1_4");
                Ok(())
            }
        }
    }
}
