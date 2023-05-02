//! Memory Management Unit Driver.

use aarch64_cpu::{asm::barrier, registers::*};
use core::intrinsics::unlikely;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

/// Memory Management Unit type.
pub struct MemoryManagementUnit;

/// Constants for indexing the MAIR_EL3.
#[allow(dead_code)]
pub mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

pub static MMU: MemoryManagementUnit = MemoryManagementUnit;

impl MemoryManagementUnit {
    /// Setup function for the MAIR_EL3 register.
    fn set_up_mair(&self) {
        // Define the memory types being mapped.
        MAIR_EL3.write(
            // Attribute 1 - Cacheable normal DRAM.
            MAIR_EL3::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
        MAIR_EL3::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +

        // Attribute 0 - Device.
        MAIR_EL3::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        SCTLR_EL3.matches_all(SCTLR_EL3::M::Enable)
    }

    pub unsafe fn disable_mmu_and_caching(&self) {
        // Disable the MMU .
        //
        // First, force all previous changes to be seen before the MMU is disabled.
        barrier::isb(barrier::SY);

        // We have already disabled the MMU using GDB. So, we only turn off data and instruction caching. 
        SCTLR_EL3.modify(
            SCTLR_EL3::C::NonCacheable + SCTLR_EL3::I::NonCacheable,
        );

        // Force MMU disabling to complete before next instruction.
        barrier::isb(barrier::SY);
    }
}

/// Return a reference to the MMU instance.
pub fn mmu() -> &'static MemoryManagementUnit {
    &MMU
}
