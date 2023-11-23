#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(cmse_nonsecure_entry)]
#![feature(abi_c_cmse_nonsecure_call)]
#![feature(once_cell)]
#![feature(type_alias_impl_trait)]

use nrf9160_hal::{prelude::OutputPin, gpio, uarte, Uarte};

use cortex_m_rt::{exception, entry};
use rustBoot_hal::{nrf::nrf9160::FlashWriterEraser, FlashInterface}; //nrf::nrf9160::GLOBAL_UART};
use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};
use core::{fmt::Write, cell::OnceCell, panic::PanicInfo};
// use defmt::*;
use nrf9160_pac::{UARTE0_NS, P0_NS};

// SCB Application Interrupt and Reset Control Register Definitions
const SCB_AIRCR_VECTKEY_POS: u32 = 16; // SCB AIRCR: VECTKEY Position
const SCB_AIRCR_PRIGROUP_POS: u32 = 8; // SCB AIRCR: PRIGROUP Position
const SCB_AIRCR_PRIGROUP_MSK: u32 = 7u32 << SCB_AIRCR_PRIGROUP_POS; // SCB AIRCR: PRIGROUP Mask
const SCB_AIRCR_SYSRESETREQ_POS: u32 = 2; // SCB AIRCR: SYSRESETREQ Position
const SCB_AIRCR_SYSRESETREQ_MSK: u32 = 1u32 << SCB_AIRCR_SYSRESETREQ_POS; // SCB AIRCR: SYSRESETREQ Mask

#[cfg(feature = "defmt")]
use defmt_rtt as _; // global logger
// use panic_probe as _;

/// System Reset
///
/// Initiates a system reset request to reset the MCU.
#[inline]
pub fn nvic_systemreset() -> ! {
    let core_peripherals = cortex_m::Peripherals::take().unwrap();
    let scb = core_peripherals.SCB;
    cortex_m::asm::dsb();
    unsafe {
        scb.aircr.write(
            (0x5FA << SCB_AIRCR_VECTKEY_POS)
                | (scb.aircr.read() & SCB_AIRCR_PRIGROUP_MSK)
                | SCB_AIRCR_SYSRESETREQ_MSK,
        );
    }
    cortex_m::asm::dsb();
    loop {}
}

// A UART we can access from anywhere (with run-time lock checking).
// static GLOBAL_UART: OnceCell<spin::Mutex<Option<nrf9160_hal::uarte::Uarte<nrf9160_hal::pac::UARTE0_NS>>>> =
// 	OnceCell::new();

// struct Printer;
// impl Write for Printer {
//     fn write_str(&mut self, s: &str) -> core::fmt::Result {
//         UART_OUT.lock(|uart| {
//             uart.borrow_mut()
//                 .as_mut()
//                 .unwrap()
//                 .blocking_write(s.as_bytes())
//                 .unwrap()
//         });
//         Ok(())
//     }
// }

#[entry]
fn main()-> ! {   
    let p = nrf9160_pac::Peripherals::take().unwrap();//unsafe { nrf9160_pac::Peripherals::steal() }; 
    
    let p0: P0_NS = unsafe { core::mem::transmute(()) };
    let p0 = gpio::p0::Parts::new(p0);
    let mut led2 = p0.p0_03.into_push_pull_output(gpio::Level::Low).degrade();
    let mut led1 = p0.p0_02.into_push_pull_output(gpio::Level::High).degrade();
    led1.set_high().unwrap();
    let mut count = 0;
    
    let uarte0: UARTE0_NS = p.UARTE0_NS;
    
    let pins = uarte::Pins {
        txd: p0.p0_01.into_push_pull_output(gpio::Level::High).degrade(),
        rxd: p0.p0_00.into_floating_input().degrade(),
        cts: None,
        rts: None,
    };
    
    let mut i = 0;
    let flash_writer = FlashWriterEraser::new(p.NVMC_S, p.NVMC_NS, false);//{nvmc_s:p.NVMC_S, nvmc_ns: p.NVMC_NS, secure: false};

    let updater = FlashUpdater::new(flash_writer);
    while i < 5 {
        led2.set_low();
        cortex_m::asm::delay(5000000);
        led2.set_high();
        cortex_m::asm::delay(5000000);
        i += 1;
    }

    match updater.update_trigger() {
        Ok(_v) => {}
        Err(e) => {
            panic!("couldnt trigger update: {:?}", e);
        }
    }
    
    nvic_systemreset();
}



#[exception]
unsafe fn HardFault(frame: &cortex_m_rt::ExceptionFrame) -> ! {
    // defmt::println!("{:?}", frame);
    let sau = &*cortex_m::peripheral::SAU::PTR;
    defmt::println!("Secure ctrl: {:X}", sau.ctrl.read().0);
    defmt::println!("Secure fault status register: {:X}", sau.sfsr.read().0);
    defmt::println!("Secure fault address register: {:X}", sau.sfar.read().0);

    let scb = &*cortex_m::peripheral::SCB::PTR;
    defmt::println!("Configurable Fault Status Register: {:X}", scb.cfsr.read());

    cortex_m::asm::delay(u32::MAX);

    cortex_m::peripheral::SCB::sys_reset();
}


#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    panic!("DefaultHandler IRQn = {}", irqn);
}

// Called when our code panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    defmt::println!("Panic occured");
    cortex_m::asm::udf();
}