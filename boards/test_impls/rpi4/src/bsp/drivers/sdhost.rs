use super::common::MMIODerefWrapper;
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

// --------------------------------------------------------------------------------------------------
// PRIVATE INTERNAL SD HOST REGISTER STRUCTURES AS PER BCM2835 MANUAL
// --------------------------------------------------------------------------------------------------

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
        ///  - 0b00: no response
        ///  - 0b01: 136 bits response
        ///  - 0b10: 48 bits response
        ///  - 0b11: 48 bits response using busy
        CMD_RSPNS_TYPE OFFSET(16) NUMBITS(2) [
            CMD_NO_RESP = 0b00,
            CMD_136BIT_RESP = 0b01,
            CMD_48BIT_RESP = 0b10,
            CMD_BUSY48BIT_RESP = 0b11
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
        CMD_INDEX OFFSET(24) NUMBITS(5) [],
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

        /// Wite as zero read as don't care
        _reserved OFFSET(0) NUMBITS(1) [],
        /// Use 4 data lines (true = enable)
        HCTL_DWIDTH OFFSET(1) NUMBITS(1) [],
        /// Select high speed mode (true = enable)
        HCTL_HS_EN OFFSET(2) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved1 OFFSET(3) NUMBITS(2) [],
        /// Use 8 data lines (true = enable)
        HCTL_8BIT OFFSET(5) NUMBITS(1) [],
        /// Write as zero read as don't care
        _reserved2 OFFSET(6) NUMBITS(10) [],
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
        _reserved3 OFFSET(23) NUMBITS(9) [],
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

    /// This register holds the interrupt flags. Each flag can be disabled using the according bit
    /// in the IRPT_MASK register.
    INTERRUPT [

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
        (0x04 => EMMC_BLKSIZECNT: ReadOnly<u32, BLKSIZECNT::Register>),
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
        SD_SPEC OFFSET(0) NUMBITS(4) [
            /// Version 1.0-1.01
            SD_SPEC_1_101 = 0,
            /// Version 1.10
            SD_SPEC_11 = 1,
            /// ersion 2.00 or Version 3.00 (check bit SD_SPEC3)
            SD_SPEC_2_3 = 2,
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
        ],
        /// Voltage window 2.9v to 3.0v
        SD_SECURITY OFFSET(12) NUMBITS(3) [
            /// No Security
            SD_SEC_NONE = 0,
            /// Security Not Used
            SD_SEC_NOT_USED = 1,
            /// SDSC Card (Security Version 1.01)
            SD_SEC_101 = 2,
            /// SDHC Card (Security Version 2.00)
            SD_SEC_2 = 3,
            /// SDXC Card (Security Version 3.xx)
            SD_SEC_3 = 4,
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
        SD_SPEC3 OFFSET(23) NUMBITS(1) [],
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
    /// - CSD_RAW32_0 represents the first 32 bits.
    CSD_RAW32_0 [
        /// trans_speed as on SD CSD bits
        TRAN_SPEED OFFSET(0) NUMBITS(8) [],
        /// taac as on SD CSD bits
        TAAC OFFSET(8) NUMBITS(8) [],
        /// nsac as on SD CSD bits
        NSAC OFFSET(16) NUMBITS(8) [],
        /// CSD version as on SD CSD bits
        SPEC_VERS OFFSET(24) NUMBITS(6) [],
        /// CSD Structure Version as on SD CSD bits 
        CSD_STRUCTURE OFFSET(30) NUMBITS(2) [
            /// enum CSD version 1.0 - 1.1, Version 2.00/Standard Capacity
            CSD_VERSION_1 = 0,						
            /// enum CSD cersion 2.0, Version 2.00/High Capacity and Extended Capacity
			CSD_VERSION_2 = 1,						
        ],
    ],
    /// The Card-Specific Data register provides information regarding access to the card contents. The CSD defines the 
    /// data format, error correction type, maximum data access time, whether the DSR register can be used, etc. The 
    /// programmable part of the register can be changed by CMD27. 
    /// 
    /// - CSD_RAW32_1 represents the 2nd slice of 32 bits.
    CSD_RAW32_1 [
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
    /// - CSD_RAW32_2 represents the 3rd slice of 32 bits.
    CSD_RAW32_2 [
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
    CSD_RAW32_3 [
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

register_structs! {
   #[allow(non_snake_case)]
   pub SDCardRegisters {
        (0x00 => OCR: ReadWrite<u32, OCR::Register>),
        (0x04 => SCR: ReadWrite<u64, SCR::Register>),
        (0x0c => CID_RAW32_0: ReadWrite<u32, CID_RAW32_0::Register>),
        (0x10 => CID_RAW32_1: ReadWrite<u32, CID_RAW32_1::Register>),
        (0x14 => CID_RAW32_2: ReadWrite<u32, CID_RAW32_2::Register>),
        (0x18 => CID_RAW32_3: ReadWrite<u32, CID_RAW32_3::Register>),
        (0x1c => CSD_RAW32_0: ReadWrite<u32, CSD_RAW32_0::Register>),
        (0x20 => CSD_RAW32_1: ReadWrite<u32, CSD_RAW32_1::Register>),
        (0x28 => CSD_RAW32_2: ReadWrite<u32, CSD_RAW32_2::Register>),
        (0x2c => CSD_RAW32_3: ReadWrite<u32, CSD_RAW32_3::Register>),
        (0x30 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type SDRegisters = MMIODerefWrapper<SDCardRegisters>;

