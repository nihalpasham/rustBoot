//! Driver - BCM2711 EMMC2 controller
//!
//! adapted from SDCard.c by Leon de Boer(LdB)

#![allow(warnings)]

use crate::rpi::rpi4::bsp::global::EMMC_CONT;
use core::{convert::TryInto, fmt::Debug};

use super::common::MMIODerefWrapper;
use crate::{info, print, warn};
use rpi4_constants::*;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
    LocalRegisterCopy,
};

use rustBoot::fs::blockdevice::{Block, BlockCount, BlockDevice, BlockIdx};

// --------------------------------------------------------------------
// PRIVATE INTERNAL SD HOST REGISTER STRUCTURES AS PER BCM2835 MANUAL
// --------------------------------------------------------------------

// EMMC module registers.

register_bitfields! {
    u32,

    /// BLKSIZECNT register - It contains the number and size in bytes for data blocks to be transferred
    BLKSIZECNT [

        /// EMMC module restricts the maximum block size to the size of the internal data
        /// FIFO which is 1k bytes.
        BLKSIZE OFFSET(0) NUMBITS(10) [],
        /// Reserved - Write as 0, read as don't care
        RESERVED OFFSET(10) NUMBITS(6) [],
        /// BLKCNT is used to tell the host how many blocks of data are to be transferred.
        /// Once the data transfer has started and the TM_BLKCNT_EN bit in the CMDTM register is
        /// set, the EMMC module automatically decreases the BNTCNT value as the data blocks
        /// are transferred and stops the transfer once BLKCNT reaches 0.
        BLKCNT OFFSET(16) NUMBITS(16) [],

    ],

    /// CMDTM register - This register is used to issue commands to the card
    CMDTM [

        /// Reserved - Write as 0, read as don't care
        _reserved OFFSET(0) NUMBITS(1) [],
        ///	Enable the block counter for multiple block transfers
        TM_BLKCNT_EN OFFSET(1) NUMBITS(1) [],
        /// Select the command to be send after completion of a data transfer:
        ///  - 0b00: no command
        ///  - 0b01: command CMD12
        ///  - 0b10: command CMD23
        ///  - 0b11: reserved
        TM_AUTO_CMD_EN OFFSET(2) NUMBITS(2) [
            TM_NO_CMD = 0b00,
            TM_CMD12 = 0b01,
            TM_CMD23 = 0b10,
            _TM_RESERVED = 0b11
        ],
        /// Direction of data transfer (0 = host to card , 1 = card to host )
        TM_DAT_DIR OFFSET(4) NUMBITS(1) [],
        /// Type of data transfer (0 = single block, 1 = muli block)
        TM_MULTI_BLOCK OFFSET(5) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(6) NUMBITS(9) [],
        /// Type of expected response from card
        CMD_RSPNS_TYPE OFFSET(16) NUMBITS(2) [
            ///  - 0b00: no response
            CMD_NO_RESP = 0,
            ///  - 0b01: 136 bits response
            CMD_136BIT_RESP = 1,
            ///  - 0b10: 48 bits response
            CMD_48BIT_RESP = 2,
            ///  - 0b11: 48 bits response using busy
            CMD_BUSY48BIT_RESP = 3
        ],
        /// Write as zero read as don't care
        _reserved2 OFFSET(18) NUMBITS(1) [],
        /// Check the responses CRC (0=disabled, 1= enabled)
        CMD_CRCCHK_EN OFFSET(19) NUMBITS(1) [],
        /// Check that response has same index as command (0=disabled, 1=enabled)
        CMD_IXCHK_EN OFFSET(20) NUMBITS(1) [],
        /// Command involves data transfer (0=disabled, 1=enabled)
        CMD_ISDATA OFFSET(21) NUMBITS(1) [],
        /// Type of command to be issued to the card
        ///  - 0b00: normal command
        ///  - 0b01: suspend command
        ///  - 0b10: resume command
        ///  - 0b11: abort command
        CMD_TYPE OFFSET(22) NUMBITS(2) [
            CMD_TYPE_NORMAL = 0b00,
            CMD_TYPE_SUSPEND = 0b01,
            CMD_TYPE_RESUME = 0b10,
            CMD_TYPE_ABORT = 0b11
         ],
        /// Index of the command to be issued to the card
        CMD_INDEX OFFSET(24) NUMBITS(6) [],
        /// Write as zero read as don't care
        _reserved3 OFFSET(30) NUMBITS(2) [],
    ],

    /// EMMC STATUS register - This register contains information intended for debugging.
    STATUS [

        /// Command line still used by previous command
        CMD_INHIBIT OFFSET(0) NUMBITS(1) [],
        /// Data lines still used by previous data transfer
        DAT_INHIBIT OFFSET(1) NUMBITS(1) [],
        /// At least one data line is active
        DAT_ACTIVE OFFSET(2) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved OFFSET(3) NUMBITS (5) [],
        /// New data can be written to EMMC
        WRITE_TRANSFER OFFSET(8) NUMBITS(1) [],
        /// New data can be read from EMMC
        READ_TRANSFER OFFSET(9) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(10) NUMBITS (10) [],
        /// Value of data lines DAT3 to DAT0
        DAT_LEVEL0 OFFSET(20) NUMBITS(4) [],
        /// Value of command line CMD
        CMD_LEVEL OFFSET(24) NUMBITS(1) [],
        /// Value of data lines DAT7 to DAT4
        DAT_LEVEL1 OFFSET(25) NUMBITS (4) [],
        /// Write as zero read as don't care
        _reserved2 OFFSET(29) NUMBITS (3) [],
     ],

    /// This register is used to configure the EMMC module.
   CONTROL0 [

        /// LED
        LED OFFSET(0) NUMBITS(1) [],
        /// Use 4 data lines (true = enable)
        HCTL_DWIDTH OFFSET(1) NUMBITS(1) [],
        /// Select high speed mode (true = enable)
        HCTL_HS_EN OFFSET(2) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(3) NUMBITS(2) [],
        /// Use 8 data lines (true = enable)
        HCTL_8BIT OFFSET(5) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved2 OFFSET(6) NUMBITS(2) [],
        /// Buspower
        BUSPOWER OFFSET(8) NUMBITS(1) [],
        /// Busvoltage
        BUSVOLTAGE OFFSET(9) NUMBITS(3) [
            V1_8 = 0b101,
            V3_0 = 0b110,
            V3_3 = 0b111,
        ],
        /// Write as zero read as don't care
        _reserved3 OFFSET(12) NUMBITS(4) [],
        /// Stop the current transaction at the next block gap
        GAP_STOP OFFSET(16) NUMBITS(1) [],
        /// Restart a transaction last stopped using the GAP_STOP
        GAP_RESTART OFFSET(17) NUMBITS(1) [],
        /// Use DAT2 read-wait protocol for cards supporting this
        READWAIT_EN OFFSET(18) NUMBITS(1) [],
        /// Enable SDIO interrupt at block gap
        GAP_IEN OFFSET(19) NUMBITS(1) [],
        /// SPI mode enable
        SPI_MODE OFFSET(20) NUMBITS(1) [],
        /// Boot mode access
        BOOT_EN OFFSET(21) NUMBITS(1) [],
        /// Enable alternate boot mode access
        ALT_BOOT_EN OFFSET(22) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved4 OFFSET(23) NUMBITS(9) [],
    ],

   /// This register is used to configure the EMMC module.
   CONTROL1 [

        /// Clock enable for internal EMMC clocks for power saving
        CLK_INTLEN OFFSET(0) NUMBITS(1) [],
        /// SD clock stable  0=No 1=yes   **read only
        CLK_STABLE OFFSET(1) NUMBITS(1) [],
        /// SD clock enable  0=disable 1=enable
        CLK_EN OFFSET(2) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved OFFSET(3) NUMBITS (2) [],
        /// Mode of clock generation (0=Divided, 1=Programmable)
        CLK_GENSEL OFFSET(5) NUMBITS(1) [],
        /// SD clock base divider MSBs (Version3+ only)
        CLK_FREQ_MS2 OFFSET(6) NUMBITS(2) [],
        /// SD clock base divider LSBs
        CLK_FREQ8 OFFSET(8) NUMBITS(8) [],
        /// Data timeout unit exponent
        DATA_TOUNIT OFFSET(16) NUMBITS(4) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(20) NUMBITS (4) [],
        /// Reset the complete host circuit
        SRST_HC OFFSET(24) NUMBITS(1) [],
        /// Reset the command handling circuit
        SRST_CMD OFFSET(25) NUMBITS(1) [],
        /// Reset the data handling circuit
        SRST_DATA OFFSET(26) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved2 OFFSET(27) NUMBITS (5) [],
    ],

    /// This register is used to enable the different interrupts in the INTERRUPT register to
    /// generate an interrupt on the int_to_arm output.
    CONTROL2 [

        /// Auto command not executed due to an error **read only
        ACNOX_ERR OFFSET(0) NUMBITS(1) [],
        /// Timeout occurred during auto command execution **read only
        ACTO_ERR OFFSET(1) NUMBITS(1) [],
        /// Command CRC error occurred during auto command execution **read only
        ACCRC_ERR OFFSET(2) NUMBITS(1) [],
        /// End bit is not 1 during auto command execution **read only
        ACEND_ERR OFFSET(3) NUMBITS(1) [],
        /// Command index error occurred during auto command execution **read only
        ACBAD_ERR OFFSET(4) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved OFFSET(5) NUMBITS(2) [],
        /// Error occurred during auto command CMD12 execution **read only
        NOTC12_ERR OFFSET(7) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(8) NUMBITS(8) [],
        /// Select the speed mode of the SD card (SDR12, SDR25 etc)
        UHSMODE OFFSET(16) NUMBITS(3) [
          SDR12 = 0,
          SDR25 = 1,
          SDR50 = 2,
          SDR104 = 3,
          DDR50 = 4,
         ],
        /// Write as zero read as don't care
        _reserved2 OFFSET(19) NUMBITS(3) [],
        /// Start tuning the SD clock
        TUNEON OFFSET(22) NUMBITS(1) [],
        /// Tuned clock is used for sampling data
        TUNED OFFSET(23) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved3 OFFSET(24) NUMBITS(8) [],
    ],

    /// This register holds the interrupt flags. Each flag can be disabled using the corresponding bit
    /// in the IRPT_MASK register.
    INTERRUPT [

        /// Command has finished
        CMD_DONE OFFSET(0) NUMBITS(1) [],
        /// Data transfer has finished
        DATA_DONE OFFSET(1) NUMBITS(1) [],
        /// Data transfer has stopped at block gap
        BLOCK_GAP OFFSET(2) NUMBITS(1) [],
        /// DMA Interrupt
        DMA_INT OFFSET(3) NUMBITS(1) [],
        /// Data can be written to DATA register
        WRITE_RDY OFFSET(4) NUMBITS(1) [],
        /// DATA register contains data to be read
        READ_RDY OFFSET(5) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(6) NUMBITS(2) [],
        /// Card made interrupt request
        CARD_INT OFFSET(8) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved2 OFFSET(9) NUMBITS(3) [],
        /// Clock retune request was made
        RETUNE OFFSET(12) NUMBITS(1) [],
        /// Boot acknowledge has been received
        BOOTACK OFFSET(13) NUMBITS(1) [],
        /// Boot operation has terminated
        ENDBOOT OFFSET(14) NUMBITS(1) [],
        /// An error has occured
        ERR OFFSET(15) NUMBITS(1) [],
        /// Timeout on command line
        CTO_ERR OFFSET(16) NUMBITS(1) [],
        /// Command CRC error
        CCRC_ERR OFFSET(17) NUMBITS(1) [],
        /// End bit on command line not 1
        CEND_ERR OFFSET(18) NUMBITS(1) [],
        /// Incorrect command index in response
        CBAD_ERR OFFSET(19) NUMBITS(1) [],
        /// Timeout on data line
        DTO_ERR OFFSET(20) NUMBITS(1) [],
        /// Data CRC error
        DCRC_ERR OFFSET(21) NUMBITS(1) [],
        /// End bit on data line not 1
        DEND_ERR OFFSET(22) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved3 OFFSET(23) NUMBITS(1) [],
        /// Auto command error
        ACMD_ERR OFFSET(24) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved4 OFFSET(25) NUMBITS(7) [],
    ],

    /// This register is used to mask the interrupt flags in the INTERRUPT register.
    IRPT_MASK [
        /// Command has finished
        CMD_DONE OFFSET(0) NUMBITS(1) [],
        /// Data transfer has finished
        DATA_DONE OFFSET(1) NUMBITS(1) [],
        /// Data transfer has stopped at block gap
        BLOCK_GAP OFFSET(2) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved OFFSET(3) NUMBITS(1) [],
        /// Data can be written to DATA register
        WRITE_RDY OFFSET(4) NUMBITS(1) [],
        /// DATA register contains data to be read
        READ_RDY OFFSET(5) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(6) NUMBITS(2) [],
        /// Card made interrupt request
        CARD OFFSET(8) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved2 OFFSET(9) NUMBITS(3) [],
        /// Clock retune request was made
        RETUNE OFFSET(12) NUMBITS(1) [],
        /// Boot acknowledge has been received
        BOOTACK OFFSET(13) NUMBITS(1) [],
        /// Boot operation has terminated
        ENDBOOT OFFSET(14) NUMBITS(1) [],
        /// An error has occured
        ERR OFFSET(15) NUMBITS(1) [],
        /// Timeout on command line
        CTO_ERR OFFSET(16) NUMBITS(1) [],
        /// Command CRC error
        CCRC_ERR OFFSET(17) NUMBITS(1) [],
        /// End bit on command line not 1
        CEND_ERR OFFSET(18) NUMBITS(1) [],
        /// Incorrect command index in response
        CBAD_ERR OFFSET(19) NUMBITS(1) [],
        /// Timeout on data line
        DTO_ERR OFFSET(20) NUMBITS(1) [],
        /// Data CRC error
        DCRC_ERR OFFSET(21) NUMBITS(1) [],
        /// End bit on data line not 1
        DEND_ERR OFFSET(22) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved3 OFFSET(23) NUMBITS(1) [],
        /// Auto command error
        ACMD_ERR OFFSET(24) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved4 OFFSET(25) NUMBITS(7) [],
        ],

    IRPT_EN [
        /// Command has finished
        CMD_DONE OFFSET(0) NUMBITS(1) [],
        /// Data transfer has finished
        DATA_DONE OFFSET(1) NUMBITS(1) [],
        /// Data transfer has stopped at block gap
        BLOCK_GAP OFFSET(2) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved OFFSET(3) NUMBITS(1) [],
        /// Data can be written to DATA register
        WRITE_RDY OFFSET(4) NUMBITS(1) [],
        /// DATA register contains data to be read
        READ_RDY OFFSET(5) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(6) NUMBITS(2) [],
        /// Card made interrupt request
        CARD OFFSET(8) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved2 OFFSET(9) NUMBITS(3) [],
        /// Clock retune request was made
        RETUNE OFFSET(12) NUMBITS(1) [],
        /// Boot acknowledge has been received
        BOOTACK OFFSET(13) NUMBITS(1) [],
        /// Boot operation has terminated
        ENDBOOT OFFSET(14) NUMBITS(1) [],
        /// An error has occured
        ERR OFFSET(15) NUMBITS(1) [],
        /// Timeout on command line
        CTO_ERR OFFSET(16) NUMBITS(1) [],
        /// Command CRC error
        CCRC_ERR OFFSET(17) NUMBITS(1) [],
        /// End bit on command line not 1
        CEND_ERR OFFSET(18) NUMBITS(1) [],
        /// Incorrect command index in response
        CBAD_ERR OFFSET(19) NUMBITS(1) [],
        /// Timeout on data line
        DTO_ERR OFFSET(20) NUMBITS(1) [],
        /// Data CRC error
        DCRC_ERR OFFSET(21) NUMBITS(1) [],
        /// End bit on data line not 1
        DEND_ERR OFFSET(22) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved3 OFFSET(23) NUMBITS(1) [],
        /// Auto command error
        ACMD_ERR OFFSET(24) NUMBITS(1) [],
        /// Write as zero read as don't car
        _reserved4 OFFSET(25) NUMBITS(7) [],
    ],

    /// This register is used to delay the card clock when sampling the returning data and
    /// command response from the card. DELAY determines by how much the sampling clock is delayed per step
    TUNE_STEP [
        /// Select the speed mode of the SD card (SDR12, SDR25 etc)
        DELAY OFFSET(0) NUMBITS(3) [
            TUNE_DELAY_200ps  = 0,
            TUNE_DELAY_400ps  = 1,
            TUNE_DELAY_400psA = 2,
            TUNE_DELAY_600ps  = 3,
            TUNE_DELAY_700ps  = 4,
            TUNE_DELAY_900ps  = 5,
            // why the duplicate value??
            TUNE_DELAY_900psA = 6,
            TUNE_DELAY_1100ps = 7,
        ],
        /// Write as zero read as don't care
        _reserved OFFSET(3) NUMBITS(29) [],
    ],

    /// This register contains the version information and slot interrupt status
    SLOTISR_VER [
        /// Logical OR of interrupt and wakeup signal for each slot
        SLOT_STATUS OFFSET(0) NUMBITS(8) [],
        /// Write as zero read as don't care
        _reserved OFFSET(8) NUMBITS(8) [],
        /// Host Controller specification version
        SDVERSION OFFSET(16) NUMBITS(8) [],
        /// Vendor Version Number
        VENDOR OFFSET(24) NUMBITS(8) [],
    ],
}

register_structs! {
   #[allow(non_snake_case)]
   pub RegisterBlock {
        (0x00 => EMMC_ARG2: ReadWrite<u32>),
        (0x04 => EMMC_BLKSIZECNT: ReadWrite<u32, BLKSIZECNT::Register>),
        (0x08 => EMMC_ARG1: ReadWrite<u32>),
        (0x0c => EMMC_CMDTM: ReadWrite<u32, CMDTM::Register>),
        (0x10 => EMMC_RESP0: ReadWrite<u32>),
        (0x14 => EMMC_RESP1: ReadWrite<u32>),
        (0x18 => EMMC_RESP2: ReadWrite<u32>),
        (0x1c => EMMC_RESP3: ReadWrite<u32>),
        (0x20 => EMMC_DATA:  ReadWrite<u32>),
        (0x24 => EMMC_STATUS: ReadWrite<u32, STATUS::Register>),
        (0x28 => EMMC_CONTROL0: ReadWrite<u32, CONTROL0::Register>),
        (0x2c => EMMC_CONTROL1: ReadWrite<u32, CONTROL1::Register>),
        (0x30 => EMMC_INTERRUPT: ReadWrite<u32, INTERRUPT::Register>),
        (0x34 => EMMC_IRPT_MASK: ReadWrite<u32, IRPT_MASK::Register>),
        (0x38 => EMMC_IRPT_EN: ReadWrite<u32, IRPT_EN::Register>),
        (0x3c => EMMC_CONTROL2: ReadWrite<u32, CONTROL2::Register>),
        (0x40 => _reserved),
        (0x88 => EMMC_TUNE_STEP: ReadWrite<u32, TUNE_STEP::Register>),
        (0x8c => _reserved1),
        (0xfc => EMMC_SLOTISR_VER: ReadWrite<u32, SLOTISR_VER::Register>),
        (0x100 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

//***************************************************************************
//  PRIVATE INTERNAL SD CARD REGISTER STRUCTURES AS PER SD CARD STANDARD
//****************************************************************************

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

// The CID is Big Endian and the Pi butchers it by not having CRC
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
        /// PI butcher with CRC removed these bits end up empty
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

// The CID is Big Endian and the Pi butchers it by not having CRC
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

register_structs! {
   #[allow(non_snake_case)]
   pub EmmcCardregisters {
        (0x00 => OCR: ReadWrite<u32, OCR::Register>),
        (0x04 => SCR: ReadWrite<u64, SCR::Register>),
        (0x0c => CID_RAW32_0: ReadWrite<u32, CID_RAW32_0::Register>),
        (0x10 => CID_RAW32_1: ReadWrite<u32, CID_RAW32_1::Register>),
        (0x14 => CID_RAW32_2: ReadWrite<u32, CID_RAW32_2::Register>),
        (0x18 => CID_RAW32_3: ReadWrite<u32, CID_RAW32_3::Register>),
        (0x1c => CEMMC_RAW32_0: ReadWrite<u32, CEMMC_RAW32_0::Register>),
        (0x20 => CEMMC_RAW32_1: ReadWrite<u32, CEMMC_RAW32_1::Register>),
        (0x28 => CEMMC_RAW32_2: ReadWrite<u32, CEMMC_RAW32_2::Register>),
        (0x2c => CEMMC_RAW32_3: ReadWrite<u32, CEMMC_RAW32_3::Register>),
        (0x30 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type SDRegisters = MMIODerefWrapper<EmmcCardregisters>;

#[rustfmt::skip]
mod rpi4_constants {
    /*--------------------------------------------------------------------------
                INTERRUPT REGISTER TURNED  INTO MASK BIT DEFINITIONS
    --------------------------------------------------------------------------*/
    pub const INT_AUTO_ERROR    : usize = 0x01000000; // ACMD_ERR bit in register
    pub const INT_DATA_END_ERR  : usize = 0x00400000; // DEND_ERR bit in register
    pub const INT_DATA_CRC_ERR  : usize = 0x00200000; // DCRC_ERR bit in register
    pub const INT_DATA_TIMEOUT  : usize = 0x00100000; // DTO_ERR bit in register
    pub const INT_INDEX_ERROR   : usize = 0x00080000; // CBAD_ERR bit in register
    pub const INT_END_ERROR     : usize = 0x00040000; // CEND_ERR bit in register
    pub const INT_CRC_ERROR     : usize = 0x00020000; // CCRC_ERR bit in register
    pub const INT_CMD_TIMEOUT   : usize = 0x00010000; // CTO_ERR bit in register
    pub const INT_ERR           : usize = 0x00008000; // ERR bit in register
    pub const INT_ENDBOOT       : usize = 0x00004000; // ENDBOOT bit in register
    pub const INT_BOOTACK       : usize = 0x00002000; // BOOTACK bit in register
    pub const INT_RETUNE        : usize = 0x00001000; // RETUNE bit in register
    pub const INT_CARD          : usize = 0x00000100; // CARD bit in register
    pub const INT_READ_RDY      : usize = 0x00000020; // READ_RDY bit in register
    pub const INT_WRITE_RDY     : usize = 0x00000010; // WRITE_RDY bit in register
    pub const INT_BLOCK_GAP     : usize = 0x00000004; // BLOCK_GAP bit in register
    pub const INT_DATA_DONE     : usize = 0x00000002; // DATA_DONE bit in register
    pub const INT_CMD_DONE      : usize = 0x00000001; // CMD_DONE bit in register
    pub const INT_ERROR_MASK    : usize = INT_CRC_ERROR
        | INT_END_ERROR
        | INT_INDEX_ERROR
        | INT_DATA_TIMEOUT
        | INT_DATA_CRC_ERR
        | INT_DATA_END_ERR
        | INT_ERR
        | INT_AUTO_ERROR;
    pub const INT_ALL_MASK      : usize =
        INT_CMD_DONE | INT_DATA_DONE | INT_READ_RDY | INT_WRITE_RDY | INT_ERROR_MASK;

    /*--------------------------------------------------------------------------
    						  SD CARD FREQUENCIES							   
    --------------------------------------------------------------------------*/
    pub const FREQ_SETUP  : usize = 400_000; // 400 Khz
    pub const FREQ_NORMAL : usize = 25_000_000; // 25 Mhz
    pub const BASE_CLOCK  : usize = 50_000_000; // 50Mhz

    /*--------------------------------------------------------------------------
    						  CMD 41 BIT SELECTIONS							    
    --------------------------------------------------------------------------*/
    pub const ACMD41_HCS        : usize = 0x40000000;
    pub const ACMD41_SDXC_POWER : usize = 0x10000000;
    pub const ACMD41_S18R       : usize = 0x04000000;
    pub const ACMD41_VOLTAGE    : usize = 0x00ff8000;
    /* PI DOES NOT SUPPORT VOLTAGE SWITCH */
    //(ACMD41_HCS|ACMD41_SDXC_POWER|ACMD41_VOLTAGE|ACMD41_S18R)
    pub const ACMD41_ARG_HC     : usize = ACMD41_HCS | ACMD41_SDXC_POWER | ACMD41_VOLTAGE;
    pub const ACMD41_ARG_SC     : usize = ACMD41_VOLTAGE; //(ACMD41_VOLTAGE|ACMD41_S18R)
}

/*--------------------------------------------------------------------------
                           SD CARD COMMAND RECORD
--------------------------------------------------------------------------*/

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EMMCCommand<'a> {
    cmd_name: &'a str,
    cmd_code: LocalRegisterCopy<u32, CMDTM::Register>,
    use_rca: u16, // 0-bit of cmd is the rca-bit, subsequent 1-15 bits are reserved i.e. write as zero read as don't care.
    delay: u16,   // next 16-31 bits contain delay to apply after command.
}

impl<'a> EMMCCommand<'a> {
    const fn new() -> Self {
        EMMCCommand {
            cmd_name: " ",
            cmd_code: LocalRegisterCopy::new(0x0),
            use_rca: 0,
            delay: 0,
        }
    }
}

//--------------------------------------------------------------------------
//                         PUBLIC SD RESULT CODES
//--------------------------------------------------------------------------
#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SdResult {
    EMMC_OK,            // NO error
    EMMC_ERROR,         // General non specific SD error
    EMMC_TIMEOUT,       // SD Timeout error
    EMMC_BUSY,          // SD Card is busy
    EMMC_NO_RESP,       // SD Card did not respond
    EMMC_ERROR_RESET,   // SD Card did not reset
    EMMC_ERROR_CLOCK,   // SD Card clock change failed
    EMMC_ERROR_VOLTAGE, // SD Card does not support requested voltage
    EMMC_ERROR_APP_CMD, // SD Card app command failed
    EMMC_CARD_ABSENT,   // SD Card not present
    EMMC_READ_ERROR,
    EMMC_MOUNT_FAIL,
    EMMC_CARD_STATE(u32),
    NONE,
}

/*--------------------------------------------------------------------------
                    PUBLIC ENUMERATION OF SD CARD TYPE
--------------------------------------------------------------------------*/
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
/// SD card types
pub enum SdCardType {
    EMMC_TYPE_UNKNOWN,
    EMMC_TYPE_MMC,
    EMMC_TYPE_1,
    EMMC_TYPE_2_SC,
    EMMC_TYPE_2_HC,
}

static EMMC_TYPE_NAME: [&str; 5] = ["Unknown", "MMC", "Type 1", "Type 2 SC", "Type 2 HC"];

//--------------------------------------------------------------------------
//                        SD CARD COMMAND DEFINITIONS
//--------------------------------------------------------------------------
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, PartialOrd)]
/// SD card commands
pub enum SdCardCommands {
    GO_IDLE_STATE,
    ALL_SEND_CID,
    SEND_REL_ADDR,
    SET_DSR,
    SWITCH_FUNC,
    CARD_SELECT,
    SEND_IF_COND,
    SEND_CSD,
    SEND_CID,
    VOLTAGE_SWITCH,
    STOP_TRANS,
    SEND_STATUS,
    GO_INACTIVE,
    SET_BLOCKLEN,
    READ_SINGLE,
    READ_MULTI,
    SEND_TUNING,
    SPEED_CLASS,
    SET_BLOCKCNT,
    WRITE_SINGLE,
    WRITE_MULTI,
    PROGRAM_CSD,
    SET_WRITE_PR,
    CLR_WRITE_PR,
    SND_WRITE_PR,
    ERASE_WR_ST,
    ERASE_WR_END,
    ERASE,
    LOCK_UNLOCK,
    APP_CMD,
    APP_CMD_RCA,
    GEN_CMD,
    // Commands hereafter require APP_CMD.
    APP_CMD_START,
    SET_BUS_WIDTH,
    EMMC_STATUS,
    SEND_NUM_WRBL,
    SEND_NUM_ERS,
    APP_SEND_OP_COND,
    SET_CLR_DET,
    SEND_SCR,
}

impl SdCardCommands {
    fn get_cmd(&self) -> EMMCCommand<'static> {
        match self {
            Self::GO_IDLE_STATE => EMMCCommand {
                cmd_name: "GO_IDLE_STATE",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x00) + CMDTM::CMD_RSPNS_TYPE::CMD_NO_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ALL_SEND_CID => EMMCCommand {
                cmd_name: "ALL_SEND_CID",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x02) + CMDTM::CMD_RSPNS_TYPE::CMD_136BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SEND_REL_ADDR => EMMCCommand {
                cmd_name: "SEND_REL_ADDR",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x03) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SET_DSR => EMMCCommand {
                cmd_name: "SET_DSR",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x04) + CMDTM::CMD_RSPNS_TYPE::CMD_NO_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SWITCH_FUNC => EMMCCommand {
                cmd_name: "SWITCH_FUNC",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x06) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::CARD_SELECT => EMMCCommand {
                cmd_name: "CARD_SELECT",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x07) + CMDTM::CMD_RSPNS_TYPE::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SEND_IF_COND => EMMCCommand {
                cmd_name: "SEND_IF_COND",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x08) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 100,
            },
            Self::SEND_CSD => EMMCCommand {
                cmd_name: "SEND_CSD",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x09) + CMDTM::CMD_RSPNS_TYPE::CMD_136BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SEND_CID => EMMCCommand {
                cmd_name: "SEND_CID",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x0a) + CMDTM::CMD_RSPNS_TYPE::CMD_136BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::VOLTAGE_SWITCH => EMMCCommand {
                cmd_name: "VOLT_SWITCH",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x0b) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::STOP_TRANS => EMMCCommand {
                cmd_name: "STOP_TRANS",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x0c) + CMDTM::CMD_RSPNS_TYPE::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SEND_STATUS => EMMCCommand {
                cmd_name: "SEND_STATUS",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x0d) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::GO_INACTIVE => EMMCCommand {
                cmd_name: "GO_INACTIVE",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x0f) + CMDTM::CMD_RSPNS_TYPE::CMD_NO_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SET_BLOCKLEN => EMMCCommand {
                cmd_name: "SET_BLOCKLEN",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x10) + CMDTM::CMD_RSPNS_TYPE::CMD_NO_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::READ_SINGLE => EMMCCommand {
                cmd_name: "READ_SINGLE",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x11)
                            + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP
                            + CMDTM::CMD_ISDATA.val(1)
                            + CMDTM::TM_DAT_DIR.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::READ_MULTI => EMMCCommand {
                cmd_name: "READ_MULTI",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x12)
                            + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP
                            + CMDTM::CMD_ISDATA.val(1)
                            + CMDTM::TM_DAT_DIR.val(1)
                            + CMDTM::TM_BLKCNT_EN.val(1)
                            + CMDTM::TM_MULTI_BLOCK.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SEND_TUNING => EMMCCommand {
                cmd_name: "SEND_TUNING",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x13) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SPEED_CLASS => EMMCCommand {
                cmd_name: "SPEED_CLASS",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x14) + CMDTM::CMD_RSPNS_TYPE::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SET_BLOCKCNT => EMMCCommand {
                cmd_name: "SET_BLOCKCNT",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x17) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::WRITE_SINGLE => EMMCCommand {
                cmd_name: "WRITE_SINGLE",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x18)
                            + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP
                            + CMDTM::CMD_ISDATA.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::WRITE_MULTI => EMMCCommand {
                cmd_name: "WRITE_MULTI",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x19)
                            + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP
                            + CMDTM::CMD_ISDATA.val(1)
                            + CMDTM::TM_BLKCNT_EN.val(1)
                            + CMDTM::TM_MULTI_BLOCK.val(1),
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::PROGRAM_CSD => EMMCCommand {
                cmd_name: "PROGRAM_CSD",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x1b) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SET_WRITE_PR => EMMCCommand {
                cmd_name: "SET_WRITE_PR",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x1c) + CMDTM::CMD_RSPNS_TYPE::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::CLR_WRITE_PR => EMMCCommand {
                cmd_name: "CLR_WRITE_PR",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x1d) + CMDTM::CMD_RSPNS_TYPE::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SND_WRITE_PR => EMMCCommand {
                cmd_name: "SND_WRITE_PR",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x1e) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ERASE_WR_ST => EMMCCommand {
                cmd_name: "ERASE_WR_ST",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x20) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ERASE_WR_END => EMMCCommand {
                cmd_name: "ERASE_WR_END",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x21) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::ERASE => EMMCCommand {
                cmd_name: "ERASE",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x26) + CMDTM::CMD_RSPNS_TYPE::CMD_BUSY48BIT_RESP,
                    );
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::LOCK_UNLOCK => EMMCCommand {
                cmd_name: "LOCK_UNLOCK",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x2a) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::APP_CMD => EMMCCommand {
                cmd_name: "APP_CMD",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x37) + CMDTM::CMD_RSPNS_TYPE::CMD_NO_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 100,
            },
            Self::APP_CMD_RCA => EMMCCommand {
                cmd_name: "APP_CMD_RCA",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x37) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::GEN_CMD => EMMCCommand {
                cmd_name: "GEN_CMD",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x38) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            // Commands hereafter require APP_CMD.
            Self::SET_BUS_WIDTH => EMMCCommand {
                cmd_name: "SET_BUS_WIDTH",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x06) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::EMMC_STATUS => EMMCCommand {
                cmd_name: "EMMC_STATUS",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x0d) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 1,
                delay: 0,
            },
            Self::SEND_NUM_WRBL => EMMCCommand {
                cmd_name: "SEND_NUM_WRBL",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x16) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SEND_NUM_ERS => EMMCCommand {
                cmd_name: "SEND_NUM_ERS",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x17) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::APP_SEND_OP_COND => EMMCCommand {
                cmd_name: "APP_SEND_OP_COND",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x29) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 1000,
            },
            Self::SET_CLR_DET => EMMCCommand {
                cmd_name: "SET_CLR_DET",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(CMDTM::CMD_INDEX.val(0x2a) + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP);
                    cmd
                },
                use_rca: 0,
                delay: 0,
            },
            Self::SEND_SCR => EMMCCommand {
                cmd_name: "SEND_SCR",
                cmd_code: {
                    let mut cmd = LocalRegisterCopy::new(0u32);
                    cmd.write(
                        CMDTM::CMD_INDEX.val(0x33)
                            + CMDTM::CMD_RSPNS_TYPE::CMD_48BIT_RESP
                            + CMDTM::CMD_ISDATA.val(1)
                            + CMDTM::TM_DAT_DIR.val(1),
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

//--------------------------------------------------------------------------
//                          SD CARD DESCRIPTION RECORD
//--------------------------------------------------------------------------
#[rustfmt::skip]
#[repr(C)]
pub struct SdDescriptor<'a> {
    cid: CID,                                   // Card cid
    csd: CSD,                                   // Card csd
    scr: LocalRegisterCopy<u64, SCR::Register>, // Card scr
    card_capacity: u64,                         // Card capacity expanded .. calculated from card details
    emmc_card_type: SdCardType,                 // Card type
    rca: u32,                                   // Card rca
    ocr: LocalRegisterCopy<u32, OCR::Register>, // Card ocr
    status: u32,                                // Card last status
    last_cmd: EMMCCommand<'a>,
}

impl<'a> SdDescriptor<'a> {
    const fn new() -> Self {
        SdDescriptor {
            cid: CID::new(),
            csd: CSD::new(),
            scr: LocalRegisterCopy::new(0x0),
            card_capacity: 0,
            emmc_card_type: SdCardType::EMMC_TYPE_UNKNOWN,
            rca: 0,
            ocr: LocalRegisterCopy::new(0x0),
            status: 0,
            last_cmd: EMMCCommand::new(),
        }
    }
}
//--------------------------------------------------------------------------
//                        CURRENT SD CARD DATA STORAGE
//--------------------------------------------------------------------------
static mut EMMC_CARD: SdDescriptor = SdDescriptor::new();

pub const R1_ERRORS_MASK: u32 = 0xfff9c004;
pub const ST_APP_CMD: u32 = 0x00000020;
pub const DTO: u32 = 14; // data timeout exponent (guesswork)

/// The new SDHCI-compliant EMMC2 interface doesn't appear on the GPIOs -
/// it has dedicated pins, but to allow booting from SD card without a
/// completely new boot ROM it is possible to map the old ARASAN/EMMC/SDIO
/// interface to those dedicated pins.
///
/// This is controlled by bit 1 of 0x7e2000d0 - 0=EMMC2, 1=legacy EMMC.
/// In my case - I didnt need this.
// static mut MMIO_LEGACY_EMMC_CONF: u32 = 0x7e2000d0;
use crate::rpi::rpi4::arch::time::*;
use core::time::Duration;

/// Waits for the `delay` specified number of microseconds
fn timer_wait_micro(delay: u64) {
    time_manager().wait_for(Duration::from_micros(delay));
}

/// Gets current system counter value
fn timer_get_tick_count() -> u64 {
    time_manager().get_sys_tick_count()
}

/// Given two TICKCOUNT values, calculates microseconds between them.
fn tick_difference(start_time: u64, tick_count: u64) -> u64 {
    let tick_diff = tick_count - start_time;
    (tick_diff * time_manager().resolution().as_nanos() as u64) / 1000 // 1 ns == 1000 us
}

/// Representation of the SDHOST controller.
pub struct EMMCController {
    registers: Registers,
}

impl BlockDevice for &EMMCController {
    type Error = SdResult;
    /// Read one or more blocks, starting at the given block index.
    fn read(
        &self,
        blocks: &mut [Block],
        start_block_idx: BlockIdx,
        reason: &str,
    ) -> Result<(), Self::Error> {
        match reason {
            "read_multi" | "read" | "read_mbr" | "read_bpb" | "read_info_sector" | "read_fat"
            | "next_cluster" | "read_dir" | "fat_read" => {}
            _ => {
                info!("invalid read operation");
                return Err(SdResult::NONE);
            }
        }
        let num_blocks = blocks.len();
        let len = num_blocks * Block::LEN;
        let ptr = (&mut blocks[0].contents).as_mut_ptr();
        let mut buff;
        unsafe {
            // there is no way to turn a slice of arrays into a slice of bytes i.e.
            // turn a &mut [[u8; 512]] to a &mut [u8].
            // at least, we cannot do this without prior allocation. In a no_std environment, where
            // a heap doesnt exist, this becomes a pain. We'd either have to use something like `heapless`
            // (and predict the amount of space we'll use for allocation) or take the last resort.
            //
            // use `from_raw_parts_mut` to construct a mutable slice of bytes from a slice of arrays.
            //
            // # Safety
            // - This is still safe as it satifies all (of from_raw_parts_mut) usage conditions.
            buff = core::slice::from_raw_parts_mut(ptr, len);
        }
        let res =
            &EMMC_CONT.emmc_transfer_blocks(start_block_idx.0, num_blocks as u32, &mut buff, false);
        match res {
            SdResult::EMMC_OK => Ok(()),
            _ => Err(*res),
        }
    }
    /// Write one or more blocks, starting at the given block index.
    fn write(&self, _blocks: &[Block], _start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        unimplemented!()
    }
    /// Determine how many blocks this device can hold.
    fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        unimplemented!()
    }
}

impl EMMCController {
    /// Create an instance.
    ///
    /// **Safety**
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    pub fn emmc_debug_response(&self, resp: SdResult) -> SdResult {
        info!(
            "EMMC: STATUS: 0x{:08x}, CONTROL1: 0x{:08x}, INTERRUPT: 0x{:08x}\n",
            self.registers.EMMC_STATUS.get(),
            self.registers.EMMC_CONTROL1.get(),
            self.registers.EMMC_INTERRUPT.get()
        );
        info!(
            "EMMC: CMD {:?}, resp: {:?}, RESP3: 0x{:08x}, RESP2: 0x{:08x}, RESP1: 0x{:08x}, RESP0: 0x{:08x}\n",
            unsafe { EMMC_CARD.last_cmd.cmd_name },
            resp,
            self.registers.EMMC_RESP3.get(),
            self.registers.EMMC_RESP2.get(),
            self.registers.EMMC_RESP1.get(),
            self.registers.EMMC_RESP0.get()
        );
        return resp;
    }
    /// Given an interrupt mask, this function loops polling for the condition (for up to 1 second).
    ///
    /// **Returns:**
    /// - EMMC_TIMEOUT - the condition mask flags were not met in 1 second
    /// - EMMC_ERROR - an identifiable error occurred
    /// - EMMC_OK - the wait completed with a mask state as requested
    /// --------------------------------------------------------------------------
    pub fn emmc_wait_for_interrupt(&self, mask: u32) -> SdResult {
        let mut time_diff: u64 = 0; // Zero time difference
        let mut start_time: u64 = 0; // Zero start time
        let t_mask: u32 = mask | INT_ERROR_MASK as u32; // Add fatal error masks to mask provided

        while (self.registers.EMMC_INTERRUPT.get() & t_mask) == 0 && (time_diff < 1000000) {
            if start_time == 0 {
                start_time = timer_get_tick_count()
            }
            // If start time not set, then set start time
            else {
                time_diff = tick_difference(start_time, timer_get_tick_count())
            } // Time difference between start time and now
        }

        let ival = self.registers.EMMC_INTERRUPT.get(); // Fetch all the interrupt flags

        if time_diff >= 1000000                         // No response recieved, timeout occurred
            || (ival & INT_CMD_TIMEOUT as u32) != 0     // Command timeout occurred 
            || (ival & INT_DATA_TIMEOUT as u32) != 0
        // Data timeout occurred
        {
            info!(
                "EMMC: Wait for interrupt, MASK: 0x{:08x}, STATUS: 0x{:08x}, iVAL: 0x{:08x}, RESP0: 0x{:08x}, time_diff: {}\n",
                mask,
                self.registers.EMMC_STATUS.get(),
                ival,
                self.registers.EMMC_RESP0.get(),
                time_diff
            );

            // Clear the interrupt register completely.
            self.registers.EMMC_INTERRUPT.set(ival);
            return SdResult::EMMC_TIMEOUT; // Return EMMC_TIMEOUT
        } else if (ival & INT_ERROR_MASK as u32) != 0 {
            info!(
                "EMMC: Error waiting for interrupt :{}, :{}, :{}\n",
                self.registers.EMMC_STATUS.get(),
                ival,
                self.registers.EMMC_RESP0.get()
            );

            // Clear the interrupt register completely.
            self.registers.EMMC_INTERRUPT.set(ival);

            return SdResult::EMMC_ERROR; // Return EMMC_ERROR
        }

        // Clear the interrupt we were waiting for, leaving any other (non-error) interrupts.
        self.registers.EMMC_INTERRUPT.set(mask); // Clear any interrupt we are waiting on

        return SdResult::EMMC_OK;
    }

    /// Waits for up to 1 second for any command that may be in progress.
    ///
    /// **Returns:**
    /// - EMMC_BUSY - the command was not completed within 1 second period
    /// - EMMC_OK - the wait completed sucessfully
    pub fn emmc_wait_for_command(&self) -> SdResult {
        let mut time_diff: u64 = 0; // Zero time difference
        let mut start_time: u64 = 0; // Zero start time

        while (self.registers.EMMC_STATUS.matches_all(STATUS::CMD_INHIBIT.val(1)))	  // Command inhibit signal
            && (self.registers.EMMC_INTERRUPT.get() & INT_ERROR_MASK as u32) == 0   // No error occurred
            && (time_diff < 1000000)
        // Timeout not reached
        {
            if start_time == 0 {
                start_time = timer_get_tick_count(); // Get start time
            } else {
                time_diff = tick_difference(start_time, timer_get_tick_count());
            } // Time difference between start and now
        }

        if (time_diff >= 1000000)
            || (self.registers.EMMC_INTERRUPT.get() & INT_ERROR_MASK as u32) != 0
        // Error occurred or it timed out
        {
            info!(
                "EMMC: Wait for command aborted, STATUS: 0x{:08x}, INTERRUPT: 0x{:08x}, RESP0: 0x{:08x}\n",
                self.registers.EMMC_STATUS.get(),
                self.registers.EMMC_INTERRUPT.get(),
                self.registers.EMMC_RESP0.get()
            );
            return SdResult::EMMC_BUSY; // return EMMC_BUSY
        }
        return SdResult::EMMC_OK; // return EMMC_OK
    }

    /// Waits for up to 1 second for any data transfer that may be in progress.
    ///
    /// **Returns:**
    /// - EMMC_BUSY - the transfer was not completed within 1 second period
    /// - EMMC_OK - the transfer completed sucessfully
    pub fn emmc_wait_for_data(&self) -> SdResult {
        let mut time_diff: u64 = 0; // Zero time difference
        let mut start_time: u64 = 0; // Zero start time

        while (self.registers.EMMC_STATUS.matches_all(STATUS::DAT_INHIBIT.val(1))) &&		// Data inhibit signal
              (self.registers.EMMC_INTERRUPT.get() & INT_ERROR_MASK as u32) == 0 &&	  // Some error occurred
              (time_diff < 500000)
        // Timeout not reached
        {
            if start_time == 0 {
                start_time = timer_get_tick_count();
            }
            // If start time not set, set start time
            else {
                time_diff = tick_difference(start_time, timer_get_tick_count());
            } // Time difference between start time and now
        }
        if (time_diff >= 500000)
            || (self.registers.EMMC_INTERRUPT.get() & INT_ERROR_MASK as u32) != 0
        {
            info!(
                "EMMC: Wait for data aborted: {} :{} :{}\n",
                self.registers.EMMC_STATUS.get(),
                self.registers.EMMC_INTERRUPT.get(),
                self.registers.EMMC_RESP0.get()
            );
            return SdResult::EMMC_BUSY; // return EMMC_BUSY
        }
        return SdResult::EMMC_OK; // return EMMC_OK
    }

    /// Decode CSD data for logging purposes.
    pub fn unpack_csd(&self, csd: &mut CSD) {
        let mut buffer: [u8; 16] = [0; 16];

        buffer[12..].copy_from_slice(&self.registers.EMMC_RESP0.get().to_le_bytes());
        buffer[8..12].copy_from_slice(&self.registers.EMMC_RESP1.get().to_le_bytes());
        buffer[4..8].copy_from_slice(&self.registers.EMMC_RESP2.get().to_le_bytes());
        buffer[0..4].copy_from_slice(&self.registers.EMMC_RESP3.get().to_le_bytes());

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
                EMMC_CARD.card_capacity = csd.csd2.get() as u64;
                EMMC_CARD.card_capacity *= 512 * 1024; // Calculate Card capacity
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
                EMMC_CARD.card_capacity = ((csd.csd1.read(CSIZE) + 1)
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
                unsafe { EMMC_CARD.card_capacity },
                (unsafe { EMMC_CARD.card_capacity } as f32 / (1000.0 * 1000.0 * 1000.0)),
            );
        } else {
            info!(
                "CSD 1.0: c_size = {:?}, c_size_mult={:?}, card capacity: {:?}, \
                vdd_r_curr_min = {:?}, vdd_r_curr_max={:?}, vdd_w_curr_min = {:?}, \
                vdd_w_curr_max={:?}",
                csd.csd1.read(CSIZE),
                csd.csd2.read(C_SIZE_MULT),
                unsafe { EMMC_CARD.card_capacity },
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

    ///  Send command and handle response.
    pub fn emmc_sendcommand_p(&self, cmd: EMMCCommand<'static>, arg: u32) -> SdResult {
        if self.emmc_wait_for_command() != SdResult::EMMC_OK {
            return SdResult::EMMC_BUSY;
        }
        #[cfg(feature = "log")]
        info!(
            "EMMC: Sending command, CMD_NAME: {:?}, CMD_CODE: 0x{:08x}, CMD_ARG: 0x{:08x}",
            cmd.cmd_name,
            cmd.cmd_code.get(),
            arg
        );

        unsafe {
            EMMC_CARD.last_cmd = cmd;
        }

        // Clear interrupt flags.  This is done by setting the ones that are currently set
        self.registers
            .EMMC_INTERRUPT
            .set(self.registers.EMMC_INTERRUPT.get()); // Clear interrupts

        // Set the argument and the command code, Some commands require a delay before reading the response
        self.registers.EMMC_ARG1.set(arg); // Set argument to SD card
        self.registers.EMMC_CMDTM.set(cmd.cmd_code.get()); // Send command to SD card
        if cmd.delay != 0 {
            timer_wait_micro(cmd.delay.into());
        }; // Wait for required delay

        // Wait until we finish sending the command i.e. the CMD_DONE interrupt bit is set.
        let res = self.emmc_wait_for_interrupt(INT_CMD_DONE as u32);
        match res {
            SdResult::EMMC_OK => {}
            _ => return res, // In non zero return result
        };

        /* Get response from RESP0 */
        let resp0 = self.registers.EMMC_RESP0.get(); // Fetch SD card response0 to command

        match cmd.cmd_code.read_as_enum(CMDTM::CMD_RSPNS_TYPE) {
            Some(CMDTM::CMD_RSPNS_TYPE::Value::CMD_NO_RESP) => return SdResult::EMMC_OK,
            Some(CMDTM::CMD_RSPNS_TYPE::Value::CMD_BUSY48BIT_RESP) => unsafe {
                EMMC_CARD.status = resp0;
                // Store the card state.  Note that this is the state the card was in before the
                // command was accepted, not the new state.
                if resp0 & R1_ERRORS_MASK == 0 {
                    return SdResult::EMMC_OK;
                } else {
                    info!("CMD_BUSY48BIT_RESP case");
                    return SdResult::EMMC_CARD_STATE(resp0 & R1_ERRORS_MASK);
                }
            },
            // RESP0 contains card status, no other data from the RESP* registers.
            // Return value non-zero if any error flag in the status value.
            Some(CMDTM::CMD_RSPNS_TYPE::Value::CMD_48BIT_RESP) => {
                match cmd.cmd_code.read(CMDTM::CMD_INDEX) {
                    // SEND_REL_ADDR command
                    0x03 => {
                        // RESP0 contains RCA and status bits 23,22,19,12:0
                        unsafe {
                            EMMC_CARD.rca = resp0 & 0xffff0000; // RCA[31:16] of response
                            EMMC_CARD.status = ((resp0 & 0x00001fff)) |		// 12:0 map directly to status 12:0
                                ((resp0 & 0x00002000) << 6) |				// 13 maps to status 19 ERROR
                                ((resp0 & 0x00004000) << 8) |				// 14 maps to status 22 ILLEGAL_COMMAND
                                ((resp0 & 0x00008000) << 8); // 15 maps to status 23 COM_CRC_ERROR
                        }
                        // Store the card state.  Note that this is the state the card was in before the
                        // command was accepted, not the new state.
                        unsafe {
                            if EMMC_CARD.status & R1_ERRORS_MASK == 0 {
                                return SdResult::EMMC_OK;
                            } else {
                                info!("CMD_48BIT_RESP, 0x03 case");
                                return SdResult::EMMC_CARD_STATE(
                                    EMMC_CARD.status & R1_ERRORS_MASK,
                                );
                            }
                        }
                    }
                    // SEND_IF_COND command
                    0x08 => {
                        // RESP0 contains voltage acceptance and check pattern, which should match
                        // the argument.
                        unsafe { EMMC_CARD.status = 0 };
                        if resp0 == arg {
                            return SdResult::EMMC_OK;
                        } else {
                            return SdResult::EMMC_ERROR;
                        }
                        // RESP0 contains OCR register
                        // TODO: What is the correct time to wait for this?
                    }
                    // EMMC_SENDOPCOND command
                    0x29 => {
                        unsafe {
                            EMMC_CARD.status = 0;
                            EMMC_CARD.ocr.set(resp0);
                        }
                        return SdResult::EMMC_OK;
                    }
                    _ => {
                        unsafe {
                            EMMC_CARD.status = resp0;
                        }
                        // Store the card state.  Note that this is the state the card was in before the
                        // command was accepted, not the new state.
                        if resp0 & R1_ERRORS_MASK == 0 {
                            return SdResult::EMMC_OK;
                        } else {
                            return SdResult::EMMC_CARD_STATE(resp0 & R1_ERRORS_MASK);
                        }
                    }
                }
            }
            // RESP0..3 contains 128 bit CID or CSD shifted down by 8 bits as no CRC
            // Note: highest bits are in RESP3.
            Some(CMDTM::CMD_RSPNS_TYPE::Value::CMD_136BIT_RESP) => {
                unsafe {
                    EMMC_CARD.status = 0;
                }
                if cmd.cmd_code.read(CMDTM::CMD_INDEX) == 0x09 {
                    self.unpack_csd(unsafe { &mut EMMC_CARD.csd });
                } else {
                    unsafe {
                        EMMC_CARD.cid.cid3.set(resp0);
                        EMMC_CARD.cid.cid2.set(self.registers.EMMC_RESP1.get());
                        EMMC_CARD.cid.cid1.set(self.registers.EMMC_RESP2.get());
                        EMMC_CARD.cid.cid0.set(self.registers.EMMC_RESP3.get());
                    }
                }
                return SdResult::EMMC_OK;
            }
            None => SdResult::EMMC_ERROR,
        }
    }

    /// Send APP command and handle response.    
    pub fn emmc_send_app_command(&self) -> SdResult {
        // If no RCA, send the APP_CMD and don't look for a response.
        if unsafe { EMMC_CARD.rca == 0 } {
            let resp = self.emmc_sendcommand_p(SdCardCommands::APP_CMD.get_cmd(), 0x00000000);
            timer_wait_micro(100); // add a 100 us delay for cmds that automatically send APP_CMDs
                                   // info!(" no-rca APP_CMD result: {:?} ", resp);
                                   // If there is an RCA, include that in APP_CMD and check card accepted it.
        } else {
            let resp = self.emmc_sendcommand_p(SdCardCommands::APP_CMD_RCA.get_cmd(), unsafe {
                EMMC_CARD.rca
            });
            match resp {
                SdResult::EMMC_OK => {}
                _ => return self.emmc_debug_response(resp),
            }
            // Debug - check that status indicates APP_CMD accepted.
            if (unsafe { EMMC_CARD.status & ST_APP_CMD }) == 0 {
                return SdResult::EMMC_ERROR;
            };
        }
        SdResult::EMMC_OK
    }

    /// Send a command with no argument. RCA automatically added if required.
    /// APP_CMD sent automatically if required.
    pub fn emmc_send_command(&self, cmd_type: SdCardCommands) -> SdResult {
        let mut resp = SdResult::NONE;

        // Issue APP_CMD if needed.
        if cmd_type >= SdCardCommands::APP_CMD_START && {
            resp = self.emmc_send_app_command();
            resp != SdResult::EMMC_OK
        } {
            return self.emmc_debug_response(resp);
        }

        // Get the command and set RCA if required.
        let cmd = cmd_type.get_cmd();
        let mut arg = 0u32;
        if cmd.use_rca == 1 {
            unsafe { arg = EMMC_CARD.rca }
        }

        resp = self.emmc_sendcommand_p(cmd, arg);
        if resp != SdResult::EMMC_OK {
            return resp;
        };

        // Check that APP_CMD was correctly interpreted.
        if unsafe {
            cmd_type >= SdCardCommands::APP_CMD_START
                && EMMC_CARD.rca != 0
                && (EMMC_CARD.status & ST_APP_CMD) == 0
        } {
            return SdResult::EMMC_ERROR_APP_CMD;
        }

        return resp;
    }

    /// Send a command with argument. APP_CMD sent automatically if required.
    pub fn emmc_send_command_a(&self, cmd_type: SdCardCommands, arg: u32) -> SdResult {
        // Issue APP_CMD if needed.
        let mut resp = SdResult::NONE;
        if cmd_type >= SdCardCommands::APP_CMD_START && {
            resp = self.emmc_send_app_command();
            resp != SdResult::EMMC_OK
        } {
            return self.emmc_debug_response(resp);
        }
        // Get the command and pass the argument through.
        resp = self.emmc_sendcommand_p(cmd_type.get_cmd(), arg);
        if resp != SdResult::EMMC_OK {
            return resp;
        }

        // Check that APP_CMD was correctly interpreted.
        if unsafe {
            cmd_type >= SdCardCommands::APP_CMD_START
                && EMMC_CARD.rca != 0
                && (EMMC_CARD.status & ST_APP_CMD) == 0
        } {
            return SdResult::EMMC_ERROR_APP_CMD;
        }

        return resp;
    }

    /// Read card's SCR. APP_CMD sent automatically if required.
    pub fn emmc_read_scr(&self) -> SdResult {
        // SEND_SCR command is like a READ_SINGLE but for a block of 8 bytes.
        // Ensure that any data operation has completed before reading the block.
        if self.emmc_wait_for_data() != SdResult::EMMC_OK {
            return SdResult::EMMC_TIMEOUT;
        }

        // Set BLKSIZECNT to 1 block of 8 bytes, send SEND_SCR command
        self.registers
            .EMMC_BLKSIZECNT
            .modify(BLKSIZECNT::BLKCNT.val(1));
        self.registers
            .EMMC_BLKSIZECNT
            .modify(BLKSIZECNT::BLKSIZE.val(8));

        let resp = self.emmc_send_command(SdCardCommands::SEND_SCR);
        if resp != SdResult::EMMC_OK {
            return self.emmc_debug_response(resp);
        }

        // Wait for READ_RDY interrupt.
        let resp = self.emmc_wait_for_interrupt(INT_READ_RDY as u32);
        if resp != SdResult::EMMC_OK {
            info!("EMMC: Timeout waiting for ready to read\n");
            return self.emmc_debug_response(resp);
        }

        // Allow maximum of 100ms for the read operation.
        let mut num_read = 0u32;
        let mut count = 100000u32;
        let mut scr_lo = 0u32;
        let mut scr_hi = 0u32;
        while (num_read < 2) {
            if self
                .registers
                .EMMC_STATUS
                .matches_all(STATUS::READ_TRANSFER.val(1))
            {
                if num_read == 0 {
                    unsafe {
                        scr_lo = self.registers.EMMC_DATA.get();
                    }
                } else {
                    unsafe {
                        scr_hi = self.registers.EMMC_DATA.get();
                    }
                }
                num_read += 1;
            } else {
                timer_wait_micro(1);
                count -= 1;
                if count == 0 {
                    break;
                }
            }
        }
        // If SCR not fully read, the operation timed out.
        if (num_read != 2) {
            #[cfg(feature = "log")]
            {
                info!(
                    "EMMC: SEND_SCR ERR: {:x}, {:x}, {:?}\n",
                    self.registers.EMMC_STATUS.get(),
                    self.registers.EMMC_INTERRUPT.get(),
                    self.registers.EMMC_RESP0.get()
                );
                info!("EMMC: Reading SCR, only read : {:?} words\n", num_read);
            }

            return SdResult::EMMC_TIMEOUT;
        }

        unsafe { EMMC_CARD.scr.set(scr_lo as u64 | ((scr_hi as u64) << 32)) };
        return SdResult::EMMC_OK;
    }

    /// Get clock divider bit-pattern, given its integer value.
    pub fn emmc_get_clock_divider2(&self, div: u32) -> u32 {
        let mut v;
        let clkfreq8shift = 8; /* SD clock base divider LSBs */
        let clkfreq8mask = 0xFF00;
        let clkfreqms2shift = 6; /* SD clock base divider MSBs */
        let clkfreqms2mask = 0xC0;
        assert!(div < (1 << 10));
        v = (div << clkfreq8shift) & clkfreq8mask;
        v |= ((div >> 8) << clkfreqms2shift) & clkfreqms2mask;
        return v;
    }

    /// Set the SD clock to the given frequency (derived from the base clock)
    ///
    /// RETURN:
    /// - EMMC_ERROR_CLOCK - A fatal error occurred setting the clock
    /// - EMMC_OK - the clock was changed to given frequency
    pub fn emmc_set_clock2(&self, freq: u32) -> SdResult {
        // A divisor of zero doesnt work. I think a divisor of 1 equates to half the base clock rate.
        // TODO: need to find confirmation of the above.
        assert!(freq < BASE_CLOCK as u32);

        let mut div;
        div = BASE_CLOCK as u32 / (freq << 1);
        if (BASE_CLOCK as u32 / (div << 1)) > freq {
            div += 1
        }

        let control1 = self.emmc_get_clock_divider2(div)
            | DTO << CONTROL1::DATA_TOUNIT.val(0xf).value
            | CONTROL1::CLK_GENSEL.val(0).value
            | CONTROL1::CLK_EN.val(1).value
            | CONTROL1::CLK_INTLEN.val(1).value;
        info!("control1: {:?}", control1);
        self.registers.EMMC_CONTROL1.set(control1);

        /* Wait for clock to be stablized */
        let mut td = 0; // Zero time difference
        let mut start_time = 0; // Zero start time

        while (self.registers.EMMC_CONTROL1.matches_all(CONTROL1::CLK_STABLE.val(0)) // Clock not stable yet
            && (td < 100000))
        // Timeout not reached
        {
            if start_time == 0 {
                start_time = timer_get_tick_count();
            }
            // If start time not set, then set start time
            else {
                td = tick_difference(start_time, timer_get_tick_count());
            } // Time difference between start time and now
        }

        if (td >= 100000) {
            // Timed out waiting for stability flag
            #[cfg(feature = "log")]
            info!("EMMC: ERROR: failed to get stable clock.\n");

            return SdResult::EMMC_ERROR_CLOCK; // Return clock error
        }
        info!(
            "Divisor = {:?}, Freq Set = {:?}",
            div,
            (BASE_CLOCK as u32 / div) >> 1
        );

        return SdResult::EMMC_OK; // Clock frequency set worked
    }

    /// Reset the SD Card
    ///
    /// RETURN:
    /// - EMMC_ERROR_RESET - A fatal error occurred resetting the SD Card
    /// - EMMC_OK - SD Card reset correctly
    pub fn emmc_reset_card(&self) -> SdResult {
        let mut td = 0; // Zero time difference
        let mut start_time = 0; // Zero start time

        self.registers.EMMC_CONTROL1.write(CONTROL1::SRST_HC.val(1)); // Reset the complete host circuit
        timer_wait_micro(10); // Wait 10 microseconds

        info!("EMMC: reset card.");
        while (self.registers.EMMC_CONTROL1.matches_all(CONTROL1::SRST_HC.val(1))) // Host circuit reset not clear yet
                && (td < 100000)
        // Timeout not reached
        {
            if (start_time == 0) {
                start_time = timer_get_tick_count();
            }
            // If start time not set the set start time
            else {
                td = tick_difference(start_time, timer_get_tick_count());
            } // Time difference between start time and now
        }
        if (td >= 100000) {
            #[cfg(feature = "log")] // Timeout waiting for reset flag
            info!("EMMC: ERROR: failed to reset.\n");

            return SdResult::EMMC_ERROR_RESET; // Return reset SD Card error
        }

        self.registers
            .EMMC_CONTROL1
            .write(CONTROL1::SRST_DATA.val(1)); // reset data lines
        timer_wait_micro(10); // Wait 10 microseconds

        self.registers.EMMC_CONTROL1.set(0);

        // Set SD bus power VDD1 to 3.3V at initialization.
        //
        // The RPi 4's controller is more compliant with the standard.
        // This additional step was not needed on the RPi 1-3
        self.registers.EMMC_CONTROL0.set(0);
        timer_wait_micro(1);
        self.registers
            .EMMC_CONTROL0
            .write(CONTROL0::BUSVOLTAGE::V3_3 + CONTROL0::BUSPOWER.val(1));
        self.registers.EMMC_CONTROL1.set(0);
        timer_wait_micro(1);

        /* Set clock to setup frequency */
        let mut resp = self.emmc_set_clock2(FREQ_SETUP as u32);
        if resp != SdResult::EMMC_OK {
            return resp;
        } // Set low speed setup frequency (400Khz)

        // Enable interrupts and interrupt mask
        self.registers.EMMC_IRPT_EN.set(0x0);
        let mask = !INTERRUPT::CARD_INT.val(1).value;
        self.registers.EMMC_IRPT_MASK.set(mask);
        self.registers.EMMC_INTERRUPT.set(!0);
        let int_en = self.registers.EMMC_IRPT_EN.get() & !self.registers.EMMC_INTERRUPT.get();
        self.registers.EMMC_IRPT_EN.set(int_en);

        /* Reset our card structure entries */
        unsafe {
            EMMC_CARD.rca = 0; // Zero rca
            EMMC_CARD.ocr.set(0); // Zero ocr
            EMMC_CARD.last_cmd = EMMCCommand::new(); // Zero lastCmd
            EMMC_CARD.status = 0; // Zero status
            EMMC_CARD.emmc_card_type = SdCardType::EMMC_TYPE_UNKNOWN;
        } // Set card type unknown

        resp = self.emmc_send_command(SdCardCommands::GO_IDLE_STATE); // Send GO_IDLE_STATE to card

        return resp; // Return response
    }

    /// Common routine for APP_SEND_OP_COND.
    /// This is used for both SC and HC cards based on the parameter.
    pub fn emmc_app_send_op_cond(&self, arg: u32) -> SdResult {
        // Send APP_SEND_OP_COND with the given argument (for SC or HC cards).
        // Note: The host shall set ACMD41 timeout more than 1 second to avoid re-issuing ACMD41.
        // This command takes a while and is time-sensitive.

        // A tip: adding debug/print statements after issuing this cmd may seem like the cmd executes successfully.
        // However, removing them (i.e. `debug prints`) later can give us errors. Its probably because we waited a bit longer
        // while printing. Note: the above does NOT apply if you use the impl as-is.

        // In other words- issuing `APP_SEND_OP_COND`, will trigger an APP_CMD prior to sending out APP_SEND_OP_COND.
        // We must ensure a 100us delay between the 2 commands.
        let mut resp = self.emmc_send_command_a(SdCardCommands::APP_SEND_OP_COND, arg);
        if resp != SdResult::EMMC_OK && resp != SdResult::EMMC_TIMEOUT {
            #[cfg(feature = "log")]
            info!("EMMC: ACMD41 returned non-timeout error {:?}\n", resp);

            return resp;
        }
        let mut count = 6u8;
        while unsafe { EMMC_CARD.ocr.read(OCR::card_power_up_busy) == 0 } && count != 0 {
            timer_wait_micro(400000);
            resp = self.emmc_send_command_a(SdCardCommands::APP_SEND_OP_COND, arg);
            if (resp != SdResult::EMMC_OK) && resp != SdResult::EMMC_TIMEOUT {
                #[cfg(feature = "log")]
                info!("EMMC: ACMD41 returned non-timeout error {:?}\n", resp);

                return resp;
            }
            count -= 1;
        }

        // Return timeout error if still not busy.
        if unsafe { EMMC_CARD.ocr.read(OCR::card_power_up_busy) == 0 } {
            return SdResult::EMMC_TIMEOUT;
        }

        // Pi is 3.3v SD only so check that one voltage values around 3.3v was returned.
        if unsafe {
            EMMC_CARD.ocr.read(OCR::voltage3v2to3v3) == 0
                && EMMC_CARD.ocr.read(OCR::voltage3v3to3v4) == 0
        } {
            return SdResult::EMMC_ERROR_VOLTAGE;
        }

        return SdResult::EMMC_OK;
    }

    /// Transfer the count blocks starting at given block to/from SD Card.
    pub fn emmc_transfer_blocks(
        &self,
        start_block: u32,
        num_blocks: u32,
        mut buffer: &mut [u8],
        write: bool,
    ) -> SdResult {
        if unsafe { EMMC_CARD.emmc_card_type == SdCardType::EMMC_TYPE_UNKNOWN } {
            return SdResult::EMMC_NO_RESP;
        } // If card not known return error
        if self.emmc_wait_for_data() != SdResult::EMMC_OK {
            return SdResult::EMMC_TIMEOUT;
        } // Ensure any data operation has completed before doing the transfer.

        // Work out the status, interrupt and command values for the transfer.
        let ready_int = if write { INT_WRITE_RDY } else { INT_READ_RDY };

        let transfer_cmd = if write {
            if num_blocks == 1 {
                SdCardCommands::WRITE_SINGLE
            } else {
                SdCardCommands::WRITE_MULTI
            }
        } else {
            if num_blocks == 1 {
                SdCardCommands::READ_SINGLE
            } else {
                SdCardCommands::READ_MULTI
            }
        };

        // If more than one block to transfer, and the card supports it,
        // send SET_BLOCK_COUNT command to indicate the number of blocks to transfer.
        let mut resp = SdResult::NONE;
        if (num_blocks > 1
            && unsafe {
                if EMMC_CARD
                    .scr
                    .matches_any(SCR::CMD_SUPPORT::CMD_SUPP_SET_BLKCNT)
                {
                    #[cfg(feature = "log")]
                    info!("card supports multi_block transfers\n");
                    true
                } else {
                    false
                }
            }
            && {
                resp = self.emmc_send_command_a(SdCardCommands::SET_BLOCKCNT, num_blocks);
                resp != SdResult::EMMC_OK
            })
        {
            return self.emmc_debug_response(resp);
        }

        // Address is different depending on the card type.
        // In case of HC cards, pass address as block # so just pass it thru.
        // In case of SC cards, pass address so need to multiply by 512 which is shift left 9.
        let block_address = if unsafe { EMMC_CARD.emmc_card_type == SdCardType::EMMC_TYPE_2_SC } {
            (start_block << 9)
        } else {
            start_block
        };

        // Set BLKSIZECNT to number of blocks * 512 bytes, send the read or write command.
        // Once the data transfer has started and the TM_BLKCNT_EN bit in the CMDTM register is
        // set the EMMC module automatically decreases the BLKCNT value as the data blocks
        // are transferred and stops the transfer once BLKCNT reaches 0.
        // TODO: TM_AUTO_CMD12 - is this needed?  What effect does it have?
        self.registers
            .EMMC_BLKSIZECNT
            .modify(BLKSIZECNT::BLKCNT.val(num_blocks));
        self.registers
            .EMMC_BLKSIZECNT
            .modify(BLKSIZECNT::BLKSIZE.val(512));
        resp = self.emmc_send_command_a(transfer_cmd, block_address);
        if resp != SdResult::EMMC_OK {
            return self.emmc_debug_response(resp);
        }

        #[cfg(feature = "log")]
        info!( 
            "EMMC: start_block: {:?}, num_blocks: {:?}\n",
            block_address, num_blocks
        );

        // Transfer all blocks.
        let mut blocks_done = 0;
        let mut buffer_addr = buffer.as_ptr() as usize;
        while (blocks_done < num_blocks) {
            // Wait for ready interrupt for the next block.
            resp = self.emmc_wait_for_interrupt(ready_int as u32);
            if resp != SdResult::EMMC_OK {
                #[cfg(feature = "log")]
                info!("EMMC: Timeout waiting for ready to read\n");

                return self.emmc_debug_response(resp);
            }

            // Handle non-word-aligned buffers byte-by-byte.
            // Note: the entire block is sent without looking at status registers.
            if (buffer_addr & 0x03) != 0 {
                for i in (0..512).step_by(4) {
                    if (write) {
                        let mut data = (buffer[i]) as u32;
                        data |= (buffer[i + 1] as u32) << 8;
                        data |= (buffer[i + 2] as u32) << 16;
                        data |= (buffer[i + 3] as u32) << 24;
                        self.registers.EMMC_DATA.set(data);
                    } else {
                        let data = self.registers.EMMC_DATA.get();
                        let bytes = data.to_le_bytes();
                        buffer[i] = bytes[3];
                        buffer[i + 1] = bytes[2];
                        buffer[i + 2] = bytes[1];
                        buffer[i + 3] = bytes[0];
                    }
                }
            }
            // Handle word-aligned buffers more efficiently.
            // Hopefully people smart enough to provide aligned data buffer
            else {
                for i in (0..512).step_by(4) {
                    if (write) {
                        let bytes = u32::from_le_bytes(buffer[i..i + 4].try_into().unwrap());
                        self.registers.EMMC_DATA.set(bytes);
                    } else {
                        buffer[i..i + 4]
                            .copy_from_slice(&self.registers.EMMC_DATA.get().to_le_bytes());
                    }
                }
            }

            if blocks_done + 1 == num_blocks {
                // we're done transferring all blocks
                break; // break here or we'll run into an `out of bounds` error.
            } else {
                buffer = &mut buffer[512 as usize..];
            }
            blocks_done += 1;
        }

        // If not all bytes were read, the operation timed out.
        if (blocks_done + 1 != num_blocks) {
            #[cfg(feature = "log")]
            info!(
                "EMMC: Transfer error only done {:?} / {:?} blocks\n",
                blocks_done, num_blocks
            );

            info!(
                "EMMC: Transfer: {:x} {:x} {:x} {:x}\n",
                self.registers.EMMC_STATUS.get(),
                self.registers.EMMC_INTERRUPT.get(),
                self.registers.EMMC_RESP0.get(),
                self.registers.EMMC_BLKSIZECNT.get()
            );

            if (!write && num_blocks > 1 && {
                resp = self.emmc_send_command(SdCardCommands::STOP_TRANS);
                resp != SdResult::EMMC_OK
            }) {
                info!("EMMC: Error response from stop transmission: {:?}\n", resp);
            }
            return SdResult::EMMC_TIMEOUT;
        }

        // For a write operation, ensure DATA_DONE interrupt before we stop transmission.
        if write && {
            resp = self.emmc_wait_for_interrupt(INT_DATA_DONE as u32);
            resp != SdResult::EMMC_OK
        } {
            #[cfg(feature = "log")]
            info!("EMMC: Timeout waiting for data done\n");

            return self.emmc_debug_response(resp);
        }

        // For a multi-block operation, if SET_BLOCKCNT is not supported, we need to indicate
        // that there are no more blocks to be transferred.
        if ((num_blocks > 1)
            && unsafe {
                if EMMC_CARD
                    .scr
                    .matches_any(SCR::CMD_SUPPORT::CMD_SUPP_SET_BLKCNT)
                {
                    #[cfg(feature = "log")]
                    info!("card supports multi_block transfers\n");
                    false
                } else {
                    true
                }
            }
            && {
                resp = self.emmc_send_command(SdCardCommands::STOP_TRANS);
                resp != SdResult::EMMC_OK
            })
        {
            return self.emmc_debug_response(resp);
        }

        return SdResult::EMMC_OK;
    }

    /// Clears the count blocks starting at given block from SD Card.
    pub fn emmc_clear_blocks(&self, start_block: u32, num_blocks: u32) -> SdResult {
        if unsafe { EMMC_CARD.emmc_card_type == SdCardType::EMMC_TYPE_UNKNOWN } {
            return SdResult::EMMC_NO_RESP;
        }

        // Ensure that any data operation has completed before doing the transfer.
        if self.emmc_wait_for_data() != SdResult::EMMC_OK {
            return SdResult::EMMC_TIMEOUT;
        }

        // Address is different depending on the card type.
        // HC pass address as block # which is just address/512.
        // SC pass address straight through.
        let start_address = if unsafe { EMMC_CARD.emmc_card_type == SdCardType::EMMC_TYPE_2_SC } {
            start_block << 9
        } else {
            start_block
        };
        let end_address = if unsafe { EMMC_CARD.emmc_card_type == SdCardType::EMMC_TYPE_2_SC } {
            (start_block + num_blocks) << 9
        } else {
            start_block + num_blocks
        };

        info!(
            "EMMC: erasing blocks from {:?} to {:?}\n",
            start_address, end_address
        );
        let mut resp = self.emmc_send_command_a(SdCardCommands::ERASE_WR_ST, start_address);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }
        resp = self.emmc_send_command_a(SdCardCommands::ERASE_WR_END, end_address);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }
        resp = self.emmc_send_command(SdCardCommands::ERASE);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        // Wait for data inhibit status to drop.
        let mut count = 1000000u32;
        while (self
            .registers
            .EMMC_STATUS
            .matches_all(STATUS::DAT_INHIBIT.val(1)))
        {
            if (count == 0) {
                #[cfg(feature = "log")]
                info!(
                    "EMMC: Timeout waiting for erase, status: 0x{:08x}, interrupt: 0x{:08x}\n",
                    self.registers.EMMC_STATUS.get(),
                    self.registers.EMMC_INTERRUPT.get()
                );

                return SdResult::EMMC_TIMEOUT;
            }

            timer_wait_micro(10);
            count -= 1;
        }

        #[cfg(feature = "log")]
        info!(
            "EMMC: completed erase command, interrupt_reg: 0x{:08x}\n",
            self.registers.EMMC_INTERRUPT.get()
        );

        return SdResult::EMMC_OK;
    }

    /// Attempts to initialize current SD Card and returns success/error status.
    /// This call should be done before any attempt to do anything with an SD card.
    ///
    /// RETURN:
    /// - EMMC_OK indicates the current card successfully initialized.
    /// - !EMMC_OK if card initialize failed with code identifying error.
    pub fn emmc_init_card(&self) -> SdResult {
        let mut resp = self.emmc_reset_card(); // Reset the card.

        if (resp != SdResult::EMMC_OK) {
            return resp;
        }

        // Send SEND_IF_COND,0x000001AA (CMD8) voltage range 0x1 check pattern 0xAA
        // If voltage range and check pattern don't match, look for older card.
        resp = self.emmc_send_command_a(SdCardCommands::SEND_IF_COND, 0x000001AA);
        let _ = match resp {
            SdResult::EMMC_OK => {
                // Card responded with voltage and check pattern.
                // Resolve voltage and check for high capacity card.
                resp = self.emmc_app_send_op_cond(ACMD41_ARG_HC as u32);
                if (resp != SdResult::EMMC_OK) {
                    return self.emmc_debug_response(resp);
                }

                // Check for high or standard capacity.
                unsafe {
                    if (EMMC_CARD.ocr.read(OCR::card_capacity) != 0) {
                        EMMC_CARD.emmc_card_type = SdCardType::EMMC_TYPE_2_HC;
                    } else {
                        EMMC_CARD.emmc_card_type = SdCardType::EMMC_TYPE_2_SC;
                    }
                }
            }
            SdResult::EMMC_BUSY => return resp,
            // No response to SEND_IF_COND, treat as an old card.
            _ => {
                // If there appears to be a command in progress, reset the card.
                resp = self.emmc_reset_card();
                if self.registers.EMMC_STATUS.read(STATUS::CMD_INHIBIT) != 0
                    && (resp != SdResult::EMMC_OK)
                {
                    return resp;
                }

                // wait(50);
                // Resolve voltage.
                resp = self.emmc_app_send_op_cond(ACMD41_ARG_SC as u32);
                if (resp != SdResult::EMMC_OK) {
                    return self.emmc_debug_response(resp);
                }

                unsafe {
                    EMMC_CARD.emmc_card_type = SdCardType::EMMC_TYPE_1;
                }
            }
        };

        // Send ALL_SEND_CID (CMD2)
        resp = self.emmc_send_command(SdCardCommands::ALL_SEND_CID);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        // Send SEND_REL_ADDR (CMD3)
        // TODO: In theory, loop back to SEND_IF_COND to find additional cards.
        resp = self.emmc_send_command(SdCardCommands::SEND_REL_ADDR);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        // Send SEND_CSD (CMD9) and parse the result.
        resp = self.emmc_send_command(SdCardCommands::SEND_CSD);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        // At this point, set the clock to full speed
        resp = self.emmc_set_clock2(FREQ_NORMAL as u32);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        // Send CARD_SELECT  (CMD7)
        // TODO: Check card_is_locked status in the R1 response from CMD7 [bit 25], if so, use CMD42 to unlock
        // CMD42 structure [4.3.7] same as a single block write; data block includes
        // PWD setting mode, PWD len, PWD data.
        resp = self.emmc_send_command(SdCardCommands::CARD_SELECT);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        // Get the SCR as well.
        // Need to do this before sending ACMD6 so that allowed bus widths are known.
        resp = self.emmc_read_scr();
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        #[cfg(feature = "log")]
        match unsafe {
            EMMC_CARD
                .scr
                .read_as_enum::<SCR::BUS_WIDTH::Value>(SCR::BUS_WIDTH)
        } {
            Some(v) => {
                info!("SCR BUS_WIDTH: {:?}", v)
            }
            None => {
                info!("Unsupported bus width, we'll default to using a `1-bit` bus")
            }
        }
        // Send APP_SET_BUS_WIDTH (ACMD6)
        // If supported, set 4 bit bus width and update the CONTROL0 register.
        if let Some(SCR::BUS_WIDTH::Value::BUS_WIDTH_1_4) =
            unsafe { EMMC_CARD.scr.read_as_enum(SCR::BUS_WIDTH) }
        {
            resp = self
                .emmc_send_command_a(SdCardCommands::SET_BUS_WIDTH, unsafe { EMMC_CARD.rca | 2 });
            if (resp != SdResult::EMMC_OK) {
                return self.emmc_debug_response(resp);
            }
            self.registers
                .EMMC_CONTROL0
                .modify(CONTROL0::HCTL_DWIDTH.val(1));
            info!("EMMC: Bus width set to 4");
        };

        // Send SET_BLOCKLEN (CMD16)
        resp = self.emmc_send_command_a(SdCardCommands::SET_BLOCKLEN, 512);
        if (resp != SdResult::EMMC_OK) {
            return self.emmc_debug_response(resp);
        }

        // Print out the CID having got this far.
        unsafe {
            let mut serial = EMMC_CARD.cid.cid2.read(CID_RAW32_2::SerialNumHi);
            serial <<= 16;
            serial |= EMMC_CARD.cid.cid3.read(CID_RAW32_3::SerialNumLo);

            info!(
                "EMMC: SD Card {}, {}Mb, mfr_id: {}, '{}{}:{}{}{}{}{}', r{}.{}, mfr_date: {}/{}, serial: 0x{:08x}, RCA: 0x{:04x}",
                EMMC_TYPE_NAME[EMMC_CARD.emmc_card_type as usize],
                EMMC_CARD.card_capacity >> 20,
                EMMC_CARD.cid.cid0.read(MID),
                EMMC_CARD.cid.cid0.read(OID_HI) as u8 as char,
                EMMC_CARD.cid.cid0.read(OID_LO) as u8 as char,
                EMMC_CARD.cid.cid1.read(ProdName1) as u8 as char,
                EMMC_CARD.cid.cid1.read(ProdName2) as u8 as char,
                EMMC_CARD.cid.cid1.read(ProdName3) as u8 as char,
                EMMC_CARD.cid.cid1.read(ProdName4) as u8 as char,
                EMMC_CARD.cid.cid2.read(ProdName5) as u8 as char,
                EMMC_CARD.cid.cid2.read(ProdRevHi),
                EMMC_CARD.cid.cid2.read(ProdRevLo),
                EMMC_CARD.cid.cid3.read(ManufactureMonth),
                2000 + EMMC_CARD.cid.cid3.read(ManufactureYear),
                serial,
                EMMC_CARD.rca >> 16
            );
        }

        return SdResult::EMMC_OK;
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
