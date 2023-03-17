//! UART driver.
//!
//! # Resources
//!
//! Descriptions taken from
//! i.MX 8M Nano Applications Processor Reference Manual, Document Number: IMX8MNRM Rev. 2, 07/2022

use super::common::MMIODerefWrapper;
use crate::nxp::imx8mn::arch::cpu_core;
use crate::nxp::imx8mn::log::console;
use crate::nxp::imx8mn::sync::{interface::Mutex, NullLock};
use crate::{print, println};
use core::fmt;
use tock_registers::interfaces::ReadWriteable;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

// UART registers.
//
// i.MX 8M Nano Applications Processor Reference Manual, Document Number: IMX8MNRM Rev. 2, 07/2022

register_bitfields! {
    u32,

    /// UART Receiver Register
    UART2_URXD [
        /// Character Ready.
        ///
        /// This read-only bit indicates an invalid read when the FIFO becomes empty and software
        /// tries to read the same old data. This bit should not be used for polling for data written to the RX FIFO.
        CHARRDY OFFSET(15) NUMBITS(1) [
            Invalid = 0,
            Valid = 1,
        ],
        /// Error Detect.
        ///
        /// Indicates whether the character present in the RX_DATA field has an error (OVRRUN,
        /// FRMERR, BRK or PRERR) status. The ERR bit is updated and valid for each received character
        ERR OFFSET(14) NUMBITS(1) [
            NoErr = 0,
            Err = 1,
        ],
        /// Receiver Overrun.
        ///
        /// This read-only bit, when HIGH, indicates that the corresponding character was stored
        /// in the last position (32nd) of the Rx FIFO. Even if a 33rd character has not been detected, this bit will be
        /// set to '1' for the 32nd character.
        OVRRUN OFFSET(13) NUMBITS(1) [
            NoOvrrn = 0,
            Ovrrn = 1,
        ],
        /// Frame Error.
        ///
        /// Indicates whether the current character had a framing error (a missing stop bit) and is
        /// possibly corrupted. FRMERR is updated for each character read from the RxFIFO
        FRMERR OFFSET(12) NUMBITS(1) [
            NoFRerr = 0,
            Frerr = 1,
        ],
        /// BREAK Detect.
        ///
        /// Indicates whether the current character was detected as a BREAK character. The data
        /// bits and the stop bit are all 0. The FRMERR bit is set when BRK is set. When odd parity is selected,
        /// PRERR is also set when BRK is set. BRK is valid for each character read from the RxFIFO
        BRK OFFSET(11) NUMBITS(1) [
            NotBreak = 0,
            Break = 1,
        ],
        /// Parity Error
        ///
        /// In RS-485 mode, it holds the ninth data bit (bit [8]) of received 9-bit RS-485 data
        /// In RS232/IrDA mode, it is the Parity Error flag. Indicates whether the current character was detected
        /// with a parity error and is possibly corrupted. PRERR is updated for each character read from the RxFIFO.
        /// When parity is disabled, PRERR always reads as 0.
        PRERR OFFSET(10) NUMBITS(1) [
            NoPRerr = 0,
            PRerr = 1,
        ],
        /// Received Data.
        ///
        /// Holds the received character. In 7-bit mode, the most significant bit (MSB) is forced to 0.
        /// In 8-bit mode, all bits are active
        RX_DATA OFFSET(0) NUMBITS(8) [],
    ],

    /// UART Transmit Register
    UART2_UTXD [
        /// Transmit Data.
        ///
        /// Holds the parallel transmit data inputs. In 7-bit mode, D7 is ignored. In 8-bit mode, all bits
        /// are used. Data is transmitted least significant bit (LSB) first. A new character is transmitted when the
        /// TX_DATA field is written. The TX_DATA field must be written only when the TRDY bit is high to ensure
        /// that corrupted data is not sent.
        TX_DATA OFFSET(0) NUMBITS(8) [],

    ],

    /// UART Control Register 1
    UART2_UCR1 [
        /// UART Enable.
        ///
        /// Enables/Disables the UART. If UARTEN is negated in the middle of a transmission, the
        /// transmitter stops and pulls the TXD line to a logic 1. UARTEN must be set to 1 before any access to
        /// UTXD and URXD registers, otherwise a transfer error is returned.
        UARTEN OFFSET(0) NUMBITS(1) [
            NotEnabled = 0,
            Enabled = 1,
        ],
        /// Determines the UART enable condition in the DOZE state.
        ///
        /// When doze_req input pin is at '1', (the Arm Platform executes a doze instruction and the system
        /// is placed in the Doze State), the DOZE bit affects operation of the UART. While in the Doze State,
        /// if this bit is asserted, the UART is disabled.
        DOZE OFFSET(1) NUMBITS(1) [
            Enabled_InDoze = 0,
            Disbaled_InDoze = 1,
        ],
        /// Aging DMA Timer Enable.
        ///
        /// Enables/Disables the receive DMA request dma_req_rx for the aging timer
        /// interrupt (triggered with AGTIM flag in USR1[8]).
        ATDMAEN OFFSET(2) NUMBITS(1) [
            NoOvrrn = 0,
            Ovrrn = 1,
        ],
        /// Transmitter Ready DMA Enable.
        ///
        /// Enables/Disables the transmit DMA request dma_req_tx when the
        /// transmitter has one or more slots available in the TxFIFO. The fill level in the TxFIFO that generates the
        /// dma_req_tx is controlled by the TXTL bits.
        TXDMAEN OFFSET(3) NUMBITS(1) [
            Disable_TX_DMA = 0,
            Enable_TX_DMA = 1,
        ],
        /// Send BREAK.
        ///
        /// Forces the transmitter to send a BREAK character. The transmitter finishes sending the
        /// character in progress (if any) and sends BREAK characters until SNDBRK is reset. Because the
        /// transmitter samples SNDBRK after every bit is transmitted, it is important that SNDBRK is asserted high
        /// for a sufficient period of time to generate a valid BREAK. After the BREAK transmission completes, the
        /// UART transmits 2 mark bits. The user can continue to fill the TxFIFO and any characters remaining are
        /// transmitted when the BREAK is terminated.
        SNDBRK OFFSET(4) NUMBITS(1) [
            DontSendBrk = 0,
            SendBrk = 1,
        ],
        /// RTS Delta Interrupt Enable.
        ///
        /// Enables/Disables the RTSD interrupt. The current status of the RTS_B pin is
        /// read in the RTSS bit.
        RTSDEN OFFSET(5) NUMBITS(1) [
            Rtsd_Int_Disable = 0,
            Rtsd_Int_Enable = 1,
        ],
        /// Transmitter Empty Interrupt Enable.
        ///
        /// Enables/Disables the transmitter FIFO empty (TXFE) interrupt.
        /// interrupt_uart. When negated, the TXFE interrupt is disabled.
        TXMPTYEN OFFSET(6) NUMBITS(1) [
            TX_Empty_Fifo_Int_Disable = 0,
            TX_Empty_Fifo_Int_Enable = 1,
        ],
        /// Infrared Interface Enable.
        ///
        /// Enables/Disables the IR interface. See the IR interface description in Infrared
        /// Interface, for more information.
        IREN OFFSET(7) NUMBITS(1) [
            IR_disable = 0,
            IR_enable = 1,
        ],
        /// Receive Ready DMA Enable.
        ///
        /// Enables/Disables the receive DMA request dma_req_rx when the receiver
        /// has data in the RxFIFO. The fill level in the RxFIFO at which a DMA request is generated is controlled by
        /// the RXTL bits. When negated, the receive DMA request is disabled.
        RXDMAEN OFFSET(8) NUMBITS(1) [
            Disable_DMA_Req = 0,
            Enable_DMA_Req = 1,
        ],
        /// Receiver Ready Interrupt Enable.
        ///
        /// Enables/Disables the RRDY interrupt when the RxFIFO contains data.
        /// The fill level in the RxFIFO at which an interrupt is generated is controlled by the RXTL bits. When
        /// RRDYEN is negated, the receiver ready interrupt is disabled.
        RRDYEN OFFSET(9) NUMBITS(1) [
            Disable_RRDY_Int = 0,
            Enable_RRDY_INT = 1,
        ],
        /// Idle Condition Detect.
        ///
        /// Controls the number of frames RXD is allowed to be idle before an idle condition is
        /// reported.
        ICD OFFSET(10) NUMBITS(2) [
            Idlefor4 = 0b00,
            Idlefor8 = 0b01,
            Idlefor16 = 0b10,
            Idlefor32 = 0b11,
        ],
        /// Idle Condition Detected Interrupt Enable.
        ///
        /// Enables/Disables the IDLE bit to generate an interrupt
        /// (interrupt_uart = 0)
        IDEN OFFSET(12) NUMBITS(1) [
            Disable_Idle_Int = 0,
            Enable_Idle_Int = 1,
        ],
        /// Transmitter Ready Interrupt Enable.
        ///
        /// Enables/Disables the transmitter Ready Interrupt (TRDY) when the
        /// transmitter has one or more slots available in the TxFIFO. The fill level in the TXFIFO at which an interrupt
        /// is generated is controlled by TxTL bits. When TRDYEN is negated, the transmitter ready interrupt is
        /// disabled.
        TRDYEN OFFSET(13) NUMBITS(1) [
            Disable_TXRDY_Int = 0,
            Enable_TXRDY_Int = 1,
        ],
        /// Automatic Detection of Baud Rate.
        ///
        /// Enables/Disables automatic baud rate detection. When the ADBR
        /// bit is set and the ADET bit is cleared, the receiver detects the incoming baud rate automatically. The
        /// ADET flag is set when the receiver verifies that the incoming baud rate is detected properly by detecting
        /// an ASCII character "A" or "a" (0x41 or 0x61).
        ADBR OFFSET(14) NUMBITS(1) [
            Disable_Auto_Baud_Detect = 0,
            Enable_Auto_Baud_Detect = 1,
        ],
        /// Automatic Baud Rate Detection Interrupt Enable.
        ///
        /// Enables/Disables the automatic baud rate detect
        /// complete (ADET) bit to generate an interrupt (interrupt_uart = 0).
        ADEN OFFSET(15) NUMBITS(1) [
            Disable_Auto_Baud_Detect_Int = 0,
            Enable_Auto_Baud_Detect_int = 1,
        ],

    ],
    /// UART Control Register 2
    UART2_UCR2 [
        /// Software Reset.
        ///
        /// Once the software writes 0 to SRST_B, the software reset remains active for 4
        /// module_clock cycles before the hardware deasserts SRST_B. The software can only write 0 to SRST_B.
        /// Writing 1 to SRST_B is ignored.
        SRST OFFSET(0) NUMBITS(1) [
            Reset = 0, // Reset the transmit and receive state machines, all FIFOs and register USR1, USR2, UBIR, UBMR, UBRC , URXD, UTXD and UTS[6-3]
            NoReset = 1,
        ],
        /// Receiver Enable.
        ///
        /// Enables/Disables the receiver. When the receiver is enabled, if the RXD input is
        /// already low, the receiver does not recognize BREAK characters, because it requires a valid 1-to-0
        /// transition before it can accept any character
        RXEN OFFSET(1) NUMBITS(1) [
            Disable_Recv = 0,
            Enable_Recv = 1,
        ],
        /// Transmitter Enable.
        ///
        /// Enables/Disables the transmitter. When TXEN is negated the transmitter is disabled
        /// and idle. When the UARTEN and TXEN bits are set the transmitter is enabled. If TXEN is negated in the
        /// middle of a transmission, the UART disables the transmitter immediately, and starts marking 1s. The
        /// transmitter FIFO cannot be written when this bit is cleared
        TXEN OFFSET(2) NUMBITS(1) [
            Disable_Tx = 0,
            Enable_Tx = 1,
        ],
        /// Aging Timer Enable. This bit is used to enable the aging timer interrupt (triggered with AGTIM)
        ATEN OFFSET(3) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Request to Send Interrupt Enable.
        ///
        /// Controls the RTS edge sensitive interrupt. When RTSEN is asserted
        /// and the programmed edge is detected on the RTS_B pin (the RTSF bit is asserted), an interrupt will be
        /// generated on the interrupt_uart pin
        RTSEN OFFSET(4) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Word Size.
        ///
        /// Controls the character length. When WS is high, the transmitter and receiver are in 8-bit
        /// mode. When WS is low, they are in 7-bit mode. The transmitter ignores bit 7 and the receiver sets bit 7 to
        /// 0. WS can be changed in-between transmission (reception) of characters, however not when a
        /// transmission (reception) is in progress, in which case the length of the current character being transmitted
        /// (received) is unpredictable.
        WS OFFSET(5) NUMBITS(1) [
            SevenBitMode = 0,
            EightBitMode = 1,
        ],
        /// Stop.
        ///
        /// Controls the number of stop bits after a character. When STPB is low, 1 stop bit is sent. When
        /// STPB is high, 2 stop bits are sent. STPB also affects the receiver
        STPB OFFSET(6) NUMBITS(1) [
            OneStopBit = 0,
            TwoStopBit = 1,
        ],
        /// Parity Odd/Even.
        ///
        /// Controls the sense of the parity generator and checker. When PROE is high, odd parity
        /// is generated and expected. When PROE is low, even parity is generated and expected. PROE has no
        /// function if PREN is low.
        PROE OFFSET(7) NUMBITS(1) [
            EvenParity = 0,
            OddParity = 1,
        ],
        /// Parity Enable.
        ///
        /// Enables/Disables the parity generator in the transmitter and parity checker in the receiver.
        /// When PREN is asserted, the parity generator and checker are enabled, and disabled when PREN is
        /// negated.
        PREN OFFSET(8) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Request to Send Edge Control.
        ///
        /// Selects the edge that triggers the RTS interrupt. This has no effect on
        /// the RTS delta interrupt. RTEC has an effect only when RTSEN = 1
        RTEC OFFSET(9) NUMBITS(2) [
            Int_Rising_Edge = 00,       // Trigger interrupt on a rising edge
            Int_falling_Edge = 01,      // Trigger interrupt on a falling edge
            Int_Any_Edge = 0b10 | 0b11, // Trigger interrupt on any edge
        ],
        /// Escape Enable. Enables/Disables the escape sequence detection logic.
        ESCEN OFFSET(11) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Clear to Send.
        ///
        /// Controls the CTS_B pin when the CTSC bit is negated. CTS has no function when CTSC
        /// is asserted
        CTS OFFSET(12) NUMBITS(1) [
            High = 0, // inactive
            Low = 1,  // active
        ],
        /// CTS Pin Control.
        ///
        /// Controls the operation of the CTS_B module output. When CTSC is asserted, the
        /// CTS_B module output is controlled by the receiver.
        CTSC OFFSET(13) NUMBITS(1) [
            CTSBit_Ctrl = 0,  // The CTS_B pin is controlled by the CTS bit
            Recv_Ctrl = 1,    // The CTS_B pin is controlled by the receiver
        ],
        /// Ignore RTS Pin.
        ///
        /// Forces the RTS input signal presented to the transmitter to always be asserted (set to
        /// low), effectively ignoring the external pin. When in this mode, the RTS pin serves as a general purpose
        /// input.
        IRTS OFFSET(14) NUMBITS(1) [
            Tx_On_Assert = 0,
            Ignore_Rts = 1,
        ],
        /// Escape Sequence Interrupt Enable.
        ///
        /// Enables/Disables the ESCF bit to generate an interrupt.
        ESCI OFFSET(15) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
    ],
    /// UART Control Register 3
    UART2_UCR3 [
        /// Autobaud Counter Interrupt Enable.
        ///
        /// This bit is used to enable the autobaud counter stopped interrupt
        /// (triggered with ACST (USR2[11]).
        ACIEN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],
        /// Invert TXD output in RS-232/RS-485 mode, set TXD active level in IrDA mode.
        INVT OFFSET(1) NUMBITS(1) [
            Tx_NotInvt = 0,
            Tx_Invt = 1,
        ],
        /// RXD Muxed Input Selected.
        ///
        /// Selects proper input pins for serial and Infrared input signal
        RXDMUXSEL OFFSET(2) NUMBITS(1) [

        ],
        /// This bit is not used in this chip.
        DTRDEN OFFSET(3) NUMBITS(1) [

        ],
        /// Asynchronous WAKE Interrupt Enable.
        ///
        /// Controls the asynchronous WAKE interrupt. An interrupt is
        /// generated when AWAKEN is asserted and a falling edge is detected on the RXD pin
        AWAKEN OFFSET(4) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Asynchronous IR WAKE Interrupt Enable.
        ///
        /// Controls the asynchronous IR WAKE interrupt. An interrupt is
        /// generated when AIRINTEN is asserted and a pulse is detected on the RXD pin.
        AIRINTEN OFFSET(5) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Receive Status Interrupt Enable.
        ///
        /// Controls the receive status interrupt (interrupt_uart). When this bit is
        /// enabled and RXDS status bit is set, the interrupt interrupt_uart will be generated.
        RXDSEN OFFSET(6) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Autobaud Detection Not Improved.
        ///
        /// Disables new features of autobaud detection (See Baud Rate
        /// Automatic Detection Protocol, for more details).
        ADNIMP OFFSET(7) NUMBITS(1) [
            New = 0,
            Old = 1,
        ],
        /// This bit is not used in this chip.
        RI OFFSET(8) NUMBITS(1) [

        ],
        /// This bit is not used in this chip.
        DCD OFFSET(9) NUMBITS(2) [

        ],
        /// This bit is not used in this chip.
        DSR OFFSET(10) NUMBITS(1) [

        ],
        /// Frame Error Interrupt Enable.
        ///
        /// Enables/Disables the interrupt. When asserted, FRAERREN causes
        /// the FRAMERR bit to generate an interr
        FRAERREN OFFSET(11) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Parity Error Interrupt Enable.
        ///
        /// Enables/Disables the interrupt. When asserted, PARERREN causes
        /// the PARITYERR bit to generate an interrupt.
        PARERREN OFFSET(12) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// This bit is not used in this chip.
        DTREN OFFSET(13) NUMBITS(1) [

        ],
        /// This bit is not used in this chip.
        DPEC OFFSET(14) NUMBITS(2) [
        ],
    ],
    /// UART Control Register 4
    UART2_UCR4 [
        /// Receive Data Ready Interrupt Enable.
        ///
        /// Enables/Disables the RDR bit to generate an interrupt
        DREN OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Receiver Overrun Interrupt Enable.
        ///
        /// Enables/Disables the ORE bit to generate an interrupt.
        OREN OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// BREAK Condition Detected Interrupt Enable.
        ///
        /// Enables/Disables the BRCD bit to generate an interrupt.
        BKEN OFFSET(2) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// TransmitComplete Interrupt Enable.
        ///
        /// Enables/Disables the TXDC bit to generate an interrupt (interrupt_uart = 0)
        TCEN OFFSET(3) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Low Power Bypass.
        ///
        /// Allows to bypass the low power new features in UART. To use during debug phase.
        LPBYP OFFSET(4) NUMBITS(1) [
            Enabled = 0,
            Disabled = 1,
        ],
        /// R Special Case.
        ///
        /// Selects the clock for the vote logic. When set, IRSC switches the vote logic clock from
        /// the sampling clock to the UART reference clock.
        IRSC OFFSET(5) NUMBITS(1) [
            Sampling_Clock = 0,
            Uart_clock = 1,
        ],
        /// DMA IDLE Condition Detected Interrupt Enable Enables/Disables the receive DMA request
        /// dma_req_rx for the IDLE interrupt (triggered with IDLE flag in USR2[12]).
        IDDMAEN OFFSET(6) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// WAKE Interrupt Enable.
        ///
        /// Enables/Disables the WAKE bit to generate an interrupt. The WAKE bit is set at
        /// the detection of a start bit by the receiver
        WKEN OFFSET(7) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Serial Infrared Interrupt Enable. Enables/Disables the serial infrared interrupt.
        ENIRI OFFSET(8) NUMBITS(1) [
            Disable = 0,
            Enable = 1,
        ],
        /// Invert RXD input in RS-232/RS-485 Mode, determine RXD input logic level being sampled in In IrDA
        /// mode.
        INVR OFFSET(9) NUMBITS(2) [
            Rxd_Not_Invt = 0,
            Rxd_Invt = 1,
        ],
        /// CTS Trigger Level.
        ///
        /// Controls the threshold at which the CTS_B pin is deasserted by the RxFIFO. After the
        /// trigger level is reached and the CTS_B pin is deasserted, the RxFIFO continues to receive data until it is
        /// full. The CTSTL bits are encoded as shown in the Settings column.
        ///
        /// Settings 0 to 32 are in use. All other settings are Reserved.
        ///
        /// 000000 0 characters received
        /// 000001 1 characters in the RxFIFO
        /// ... —
        /// ... —
        /// 100000 32 characters in the RxFIFO (maximum)
        CTSTL OFFSET(10) NUMBITS(5) [],
    ],
    /// UART Escape Character Register
    UART2_UESC [
        /// UART Escape Character.
        ///
        /// Holds the selected escape character that all received characters are compared
        /// against to detect an escape sequence.
        ESC_CHAR OFFSET(0) NUMBITS(8) [],
    ],
    /// UART Escape Timer Register
    UART2_UTIM [
        /// UART Escape Timer.
        ///
        /// Holds the maximum time interval (in ms) allowed between escape characters. The
        /// escape timer register is programmable in intervals of 2 ms.
        ///
        /// Reset value 0x000 = 2 ms up to 0xFFF = 8.192 s
        TIM OFFSET(0) NUMBITS(12) [],
    ],
    /// UART Test Register
    UART2_UTS [
        /// Software Reset.
        ///
        /// Indicates the status of the software reset (SRST_B bit of UCR2).
        SOFTRST OFFSET(0) NUMBITS(1) [
            Inactive = 0,
            Active = 1,
        ],
        /// RxFIFO FULL.
        ///
        /// Indicates the RxFIFO is full
        RXFULL OFFSET(3) NUMBITS(1) [
            NotFull = 0,
            Full = 1,
        ],
        /// TxFIFO FULL.
        ///
        /// Indicates the TxFIFO is full.
        TXFULL OFFSET(4) NUMBITS(1) [
            NotFull = 0,
            Full = 1,
        ],
        /// RxFIFO Empty.
        ///
        /// Indicates the RxFIFO is empty
        RXEMPTY OFFSET(5) NUMBITS(1) [
            NotEmpty = 0,
            Empty = 1,
        ],
        /// TxFIFO Empty.
        ///
        /// Indicates that the TxFIFO is empty
        TXEMPTY OFFSET(6) NUMBITS(1) [
            NotEmpty = 0,
            Empty = 1,
        ],
        /// This bit is not used in this chip.
        RXDBG OFFSET(9) NUMBITS(1) [
        ],
        /// Loop TX and RX for IR Test (LOOPIR).
        ///
        /// This bit controls loopback from transmitter to receiver in the InfraRed interface.
        LOOPIR OFFSET(10) NUMBITS(1) [
            No_IR_Loop = 0,
            Connect_Tx_Rx = 1,
        ],
        /// This bit is not used in this chip.
        DBGEN OFFSET(11) NUMBITS(1) [
        ],
        /// Loop TX and RX for Test.
        ///
        /// Controls loopback for test purposes
        LOOP OFFSET(12) NUMBITS(1) [
            Normal = 0,
            Internal = 1,
        ],
        /// Force Parity Error.
        ///
        /// Forces the transmitter to generate a parity error if parity is enabled. FRCPERR is
        /// provided for system debugging.
        FRCPERR OFFSET(13) NUMBITS(1) [
            Normal_Parity = 0,
            Inv_Parity = 1,
        ],
    ],
    /// UART FIFO Control Register
    UART2_UFCR [
        /// Receiver Trigger Level.
        ///
        /// Controls the threshold at which a maskable interrupt is generated by the
        /// RxFIFO. A maskable interrupt is generated whenever the data level in the RxFIFO reaches the selected
        /// threshold. The RXTL bits are encoded as shown in the Settings column.
        ///
        /// Setting 0 to 32 are in use. All other settings are Reserved.
        ///
        /// 000000 0 characters received
        /// 000001 RxFIFO has 1 character
        /// ... —
        /// ... —
        /// 011111 RxFIFO has 31 characters
        /// 100000 RxFIFO has 32 characters (maximum)
        RXTL OFFSET(0) NUMBITS(6) [
            RxFIFO_1 = 000001,
            RxFIFO_2 = 000010,
        ],
        /// DCE/DTE mode select.
        ///
        /// Select UART as data communication equipment (DCE mode) or as data terminal
        /// equipment (DTE mode).
        DCEDTE OFFSET(6) NUMBITS(1) [
            DCE_Selected = 0,
            DTE_Selected = 1,
        ],
        /// Reference Frequency Divider.
        ///
        /// Controls the divide ratio for the reference clock. The input clock is
        /// module_clock. The output from the divider is ref_clk which is used by BRM to create the 16x baud rate
        /// oversampling clock (brm_clk).
        ///
        /// 000 Divide input clock by 6
        /// 001 Divide input clock by 5
        /// 010 Divide input clock by 4
        /// 011 Divide input clock by 3
        /// 100 Divide input clock by 2
        /// 101 Divide input clock by 1
        /// 110 Divide input clock by 7
        /// 111 Reserved
        RFDIV OFFSET(7) NUMBITS(3) [
            Div4 = 100,
        ],
        /// Transmitter Trigger Level.
        ///
        /// Controls the threshold at which a maskable interrupt is generated by the
        /// TxFIFO. A maskable interrupt is generated whenever the data level in the TxFIFO falls below the selected
        /// threshold. The bits are encoded as shown in the Settings column.
        ///
        /// Settings 0 to 32 are in use. All other settings are Reserved.
        ///
        /// 000000 Reserved
        /// 000001 Reserved
        /// 000010 TxFIFO has 2 or fewer characters
        /// ... —
        /// ... —
        /// 011111 TxFIFO has 31 or fewer characters
        /// 100000 TxFIFO has 32 characters (maximum)
        TXTL OFFSET(10) NUMBITS(6) [
            TxFIFO_32 = 100000,
            TxFIFO_31 = 011111,
            TxFIFO_2 = 000010,
        ],

    ],
    /// UART BRM Incremental Register
    UART2_UBIR [
        /// Incremental Numerator.
        ///
        /// Holds the numerator value minus one of the BRM ratio (see Binary Rate Multiplier
        /// (BRM)). The UBIR register MUST be updated before the UBMR register for the baud rate to be updated
        /// correctly. If only one register is written to by software, the BRM will ignore this data until the other register
        /// is written to by software. Updating this field using byte accesses is not recommended and is undefined.
        INC OFFSET(0) NUMBITS(16) []
    ],
    /// UART BRM Modulator Register
    UART2_UBMR [
        /// Modulator Denominator.
        ///
        /// Holds the value of the denominator minus one of the BRM ratio (see Binary
        /// Rate Multiplier (BRM)). The UBIR register MUST be updated before the UBMR register for the baud rate
        /// to be updated correctly. If only one register is written to by software, the BRM will ignore this data until the
        /// other register is written to by software. Updating this register using byte accesses is not recommended
        /// and undefined.
        MOD OFFSET(0) NUMBITS(16) []
    ],
    /// UART Status Register 2
    UART2_USR2 [
        /// Receive Data Ready
        ///
        /// Indicates that at least 1 character is received and written to the RxFIFO. If the
        /// URXD register is read and there is only 1 character in the RxFIFO, RDR is automatically cleared.
        RDR OFFSET(0) NUMBITS(1) [
            RecvNotReady = 0,
            RecvReady = 1,
        ],
        /// Overrun Error.
        ///
        /// When set to 1, ORE indicates that the receive buffer (RxFIFO) was full (32 chars inside),
        /// and a 33rd character has been fully received. This 33rd character has been discarded. Clear ORE by
        /// writing 1 to it. Writing 0 to ORE has no effect.
        ORE OFFSET(1) NUMBITS(1) [
            NoOvrrn = 0,
            OvrrnErr = 1,
        ],
        /// BREAK Condition Detected.
        ///
        /// Indicates that a BREAK condition was detected by the receiver. Clear
        /// BRCD by writing 1 to it. Writing 0 to BRCD has no effect.
        BRCD OFFSET(2) NUMBITS(1) [
            NoBrkDetect = 0,
            BrkDetect = 1,
        ],
        /// Transmitter Complete.
        ///
        /// Indicates that the transmit buffer (TxFIFO) and Shift Register is empty; therefore
        /// the transmission is complete. TXDC is cleared automatically when data is written to the TxFIFO.
        TXDC OFFSET(3) NUMBITS(1) [
            Tx_Not_Complete = 0,
            Tx_Complete = 1,
        ],
        /// RTS Edge Triggered Interrupt Flag.
        ///
        /// Indicates if a programmed edge is detected on the RTS_B pin. The
        /// RTEC bits select the edge that generates an interrupt (see Table 16-4). RTSF can generate an interrupt
        /// that can be masked using the RTSEN bit. Clear RTSF by writing 1 to it. Writing 0 to RTSF has no effect.
        RTSF OFFSET(4) NUMBITS(1) [
            NoEdgeDetected = 0,
            EdgeDetected = 1,
        ],
        /// This bit is not used in this chip.
        DCDIN OFFSET(5) NUMBITS(1) [],
        /// This bit is not used in this chip.
        DCDDELT OFFSET(6) NUMBITS(1) [],
        /// Wake.
        ///
        /// Indicates the start bit is detected. WAKE can generate an interrupt that can be masked using the
        /// WKEN bit. Clear WAKE by writing 1 to it. Writing 0 to WAKE has no effect.
        WAKE OFFSET(7) NUMBITS(1) [
            NoStartBitDetected = 0,
            StartBitDetected = 1,
        ],
        /// Serial Infrared Interrupt Flag.
        ///
        /// When an edge is detected on the RXD pin during SIR Mode, this flag will
        /// be asserted. This flag can cause an interrupt which can be masked using the control bit ENIRI: UCR4 [8].
        IRINT OFFSET(8) NUMBITS(1) [
            NoEdgeDetected = 0,
            EdgeDetected = 1,
        ],
        /// This bit is not used in this chip.
        RIIN OFFSET(9) NUMBITS(1) [],
        /// This bit is not used in this chip.
        RIDELT OFFSET(10) NUMBITS(1) [],
        /// Autobaud Counter Stopped.
        ///
        /// In autobaud detection (ADBR=1), indicates the counter which determines
        /// the baud rate was running and is now stopped. This means either START bit is finished (if ADNIMP=1), or
        /// Bit 0 is finished (if ADNIMP=0). See New Autobaud Counter Stopped bit and Interrupt, for more details. An
        /// interrupt can be flagged on interrupt_uart if ACIEN=1
        ACST OFFSET(11) NUMBITS(1) [
            NotFinished = 0,
            Finished = 1,
        ],
        /// Idle Condition.
        ///
        /// Indicates that an idle condition has existed for more than a programmed amount frame
        /// (see Idle Line Detect. An interrupt can be generated by this IDLE bit if IDEN (UCR1[12]) is enabled. IDLE
        /// is cleared by writing 1 to it. Writing 0 to IDLE has no effect.
        IDLE OFFSET(12) NUMBITS(1) [
            NotIdle = 0,
            Idle = 1,
        ],
        /// This bit is not used in this chip.
        DTRF OFFSET(13) NUMBITS(1) [
        ],
        /// Transmit Buffer FIFO Empty.
        ///
        /// Indicates that the transmit buffer (TxFIFO) is empty. TXFE is cleared
        /// automatically when data is written to the TxFIFO. Even though TXFE is high, the transmission might still
        /// be in progress.
        TXFE OFFSET(14) NUMBITS(1) [
            NotEmpty = 0,
            Empty = 1,
        ],
        /// Automatic Baud Rate Detect Complete.
        ///
        /// Indicates that an "A" or "a" was received and that the receiver
        /// detected and verified the incoming baud rate. Clear ADET by writing 1 to it. Writing 0 to ADET has no
        /// effect.
        ADET OFFSET(15) NUMBITS(1) [
            AaNotRecv = 0,
            AaRecv = 1,
        ],
    ],


}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => UART2_URXD: ReadOnly<u32, UART2_URXD::Register>),
        (0x04 => _reserved0),
        (0x40 => UART2_UTXD: WriteOnly<u32, UART2_UTXD::Register>),
        (0x44 => _reserved1),
        (0x80 => UART2_UCR1: ReadWrite<u32, UART2_UCR1::Register>),
        (0x84 => UART2_UCR2: ReadWrite<u32, UART2_UCR2::Register>),
        (0x88 => UART2_UCR3: ReadWrite<u32, UART2_UCR3::Register>),
        (0x8C => UART2_UCR4: ReadWrite<u32, UART2_UCR4::Register>),
        (0x90 => UART2_UFCR: ReadWrite<u32, UART2_UFCR::Register>),
        (0x94 => _reserved2),
        (0x98 => UART2_USR2: ReadWrite<u32, UART2_USR2::Register>),
        (0x9C => UART2_UESC: ReadWrite<u32, UART2_UESC::Register>),
        (0xA0 => UART2_UTIM: ReadWrite<u32, UART2_UTIM::Register>),
        (0xA4 => UART2_UBIR: ReadWrite<u32, UART2_UBIR::Register>),
        (0xA8 => UART2_UBMR: ReadWrite<u32, UART2_UBMR::Register>),
        (0xAC => _reserved3),
        (0xB4 => UART2_UTS: ReadWrite<u32, UART2_UTS::Register>),
        (0xB8 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

pub struct UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

// Export the inner struct so that BSPs can use it for the panic handler.
pub use UartInner as PanicUart;

/// Representation of the UART.
pub struct Uart {
    inner: NullLock<UartInner>,
}

impl UartInner {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    /// Initialize UART2. On the imx8mn_evk, this is the debug port.
    ///
    /// note: we use a fixed baudrate of 115200 bps
    #[no_mangle]
    pub fn init_uart(&mut self) {
        self.registers.UART2_UCR1.set(0); // disable uart
        self.registers.UART2_UCR2.set(0); // reset uart

        // see if the reset is done.
        while (self
            .registers
            .UART2_UCR2
            .matches_all(UART2_UCR2::SRST::Reset))
        {
            cpu_core::nop();
        }

        // No parity, autobaud detect-old, rxdmuxsel=1 (fixed i.mx7)
        self.registers.UART2_UCR3.set(0x0084);

        // Set CTS FIFO trigger to 32 bytes bits 15:10
        self.registers.UART2_UCR4.set(0x8000);

        self.registers.UART2_UESC.set(0x2b);
        self.registers.UART2_UTIM.set(0x0);
        self.registers.UART2_UTS.set(0x0);

        // TX/RX-thresh = 2 bytes, refclk @24MHz / 4, Optional - DTE (bit6 = 0)
        self.registers.UART2_UFCR.modify(
            UART2_UFCR::TXTL::TxFIFO_2 + UART2_UFCR::RXTL::RxFIFO_1 + UART2_UFCR::RFDIV::Div4, // + UART2_UFCR::DCEDTE::DCE_Selected,
        );

        // We write 0x0f into UBIR to remove the 16 mult
        self.registers.UART2_UBIR.set(0x000f);

        // Ignore RTS, 8N1, enable tx/rx, disable reset
        self.registers.UART2_UCR2.modify(
            UART2_UCR2::WS::EightBitMode
                + UART2_UCR2::IRTS::Ignore_Rts
                + UART2_UCR2::RXEN::Enable_Recv
                + UART2_UCR2::TXEN::Enable_Tx
                + UART2_UCR2::SRST::NoReset,
        );

        //  The equation for BAUD rate calculation is
        //  RefClk = Supplied clock / FCR_DIVx
        //  BAUD  =    Refclk
        //          ------------
        //        16 x (UBMR + 1/ UBIR + 1)
        //  BAUD  =    6000000
        //          ------------
        //        16 x (UBMR + 1/ 15 + 1)
        self.registers
            .UART2_UBMR
            .set((24000000u64 / (2 * 115200)) as u32);

	    //setting the baudrate triggers a reset, returning cr3 to its
	    //reset value but UCR3_RXDMUXSEL "should always be set."
	    //according to the imx8 reference-manual
	    
        self.registers.UART2_UCR3.modify(UART2_UCR3::RXDMUXSEL::SET);

        // Enable UART
        self.registers
            .UART2_UCR1
            .modify(UART2_UCR1::UARTEN::Enabled);
    }

    /// Send a character.
    fn write_char(&mut self, c: char) {
        // Write the character to the buffer.
        self.registers.UART2_UTXD.set(c as u32);

        // wait until transmission is complete. Spin until TX FIFO is empty.
        while self
            .registers
            .UART2_USR2
            .matches_all(UART2_USR2::TXDC::Tx_Not_Complete)
        {
            cpu_core::nop();
        }

        self.chars_written += 1;
    }
}

/// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
/// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
/// we get `write_fmt()` automatically.
///
/// The function takes an `&mut self`, so it must be implemented for the inner struct.
///
/// See [`src/print.rs`].
///
/// [`src/print.rs`]: ../../print/index.html
impl fmt::Write for UartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

impl Uart {
    /// Create an instance.
    ///
    /// # Safety
    ///
    /// - The user must ensure to provide a correct MMIO start address.
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(UartInner::new(mmio_start_addr)),
        }
    }
}


impl super::common::interface::DeviceDriver for Uart {
    fn compatible(&self) -> &'static str {
        "i.MX8M Uart2"
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init_uart());

        Ok(())
    }
}

impl console::Write for Uart {
    /// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
    /// serialize access.
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
        // readability.
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }
}

impl console::Read for Uart {
    fn read_char(&self) -> char {
        todo!()
    }

    fn clear_rx(&self) {}
}

impl console::Statistics for Uart {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}
