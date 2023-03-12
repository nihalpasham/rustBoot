//! Architectural timer primitives.

use crate::warn;
use core::time::Duration;
use aarch64_cpu::{asm::barrier, registers::*};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

const NS_PER_S: u64 = 1_000_000_000;

/// Timekeeping interfaces.
pub trait TimeManager {
    /// The timer's resolution.
    fn resolution(&self) -> Duration;

    /// The uptime since power-on of the device.
    ///
    /// This includes time consumed by firmware and bootloaders.
    fn uptime(&self) -> Duration;

    /// Get the current value of the system counter
    fn get_sys_tick_count(&self) -> u64;

    /// Wait for a given duration.
    fn wait_for(&self, duration: Duration);
}

/// ARMv8 Generic Timer.
struct GenericTimer;

static TIME_MANAGER: GenericTimer = GenericTimer;

impl GenericTimer {
    #[inline(always)]
    fn read_cntpct(&self) -> u64 {
        // CNTPCT_EL0 is a system register that holds the 64-bit physical count value. 
        // It is part of the generic timer feature of the Arm architecture. 
        // It can be read speculatively, meaning that it can be read out of order w.r.t the program flow. 
        //
        // When the ordering of the counter read is important, an ISB instruction can be used to ensure program order
        unsafe { barrier::isb(barrier::SY) };
        CNTPCT_EL0.get()
    }
}

/// Return a reference to the time manager.
pub fn time_manager() -> &'static impl TimeManager {
    &TIME_MANAGER
}

impl TimeManager for GenericTimer {
    fn resolution(&self) -> Duration {
        Duration::from_nanos(NS_PER_S / (CNTFRQ_EL0.get() as u64))
    }

    fn uptime(&self) -> Duration {
        let current_count: u64 = self.read_cntpct() * NS_PER_S;
        let frq: u64 = CNTFRQ_EL0.get() as u64;

        Duration::from_nanos(current_count / frq)
    }

    fn get_sys_tick_count(&self) -> u64 {
        let current_count: u64 = self.read_cntpct();
        current_count
    }

    fn wait_for(&self, duration: Duration) {
        // Instantly return on zero.
        if duration.as_nanos() == 0 {
            return;
        }

        // Calculate the register compare value.
        let frq = CNTFRQ_EL0.get();
        let x = match frq.checked_mul(duration.as_nanos() as u64) {
            None => {
                warn!("wait duration too long, skipping");
                return;
            }
            Some(val) => val,
        };
        let tval = x / NS_PER_S;

        // Check if it is within supported bounds.
        let warn: Option<&str> = if tval == 0 {
            Some("smaller")
        // The upper 32 bits of CNTP_TVAL_EL0 are reserved.
        } else if tval > u32::max_value() as u64 {
            Some("bigger")
        } else {
            None
        };

        if let Some(w) = warn {
            warn!(
                "wait duration {} than architecturally supported, skipping",
                w
            );
            return;
        }

        // Set the compare value register.
        CNTP_TVAL_EL0.set(tval);

        // Kick off the counting.                       // Disable timer interrupt.
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);

        // ISTATUS will be '1' when cval ticks have passed. Busy-check it.
        while !CNTP_CTL_EL0.matches_all(CNTP_CTL_EL0::ISTATUS::SET) {}

        // Disable counting again.
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
    }
}
