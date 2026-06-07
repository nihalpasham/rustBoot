use core::mem::align_of;

use super::common::*;

pub const DTB_MAGIC: u32 = 0xD00D_FEED;
pub const COMP_VERSION: u32 = 16;

pub const TOK_BEGIN_NODE: u32 = 1;
pub const TOK_END_NODE: u32 = 2;
pub const TOK_PROPERTY: u32 = 3;
pub const TOK_NOP: u32 = 4;
pub const TOK_END: u32 = 9;

#[repr(C)]
#[derive(Debug)]
pub struct Header {
    pub magic: u32,
    pub total_size: u32,
    pub struct_offset: u32,
    pub strings_offset: u32,
    pub reserved_mem_offset: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub bsp_cpu_id: u32,
    pub strings_size: u32,
    pub struct_size: u32,
}

impl Header {
    /// DT spec says all compliant device-trees include a 40-byte header
    pub fn len(&self) -> usize {
        0x28
    }
}

impl Header {
    /// Returns the big-endian byte representation of the header.
    /// Safe: constructs bytes field-by-field from the repr(C) u32 fields.
    pub fn as_slice(&self) -> [u8; 0x28] {
        let mut bytes = [0u8; 0x28];
        let fields = [
            self.magic,
            self.total_size,
            self.struct_offset,
            self.strings_offset,
            self.reserved_mem_offset,
            self.version,
            self.last_comp_version,
            self.bsp_cpu_id,
            self.strings_size,
            self.struct_size,
        ];
        for (i, val) in fields.iter().enumerate() {
            let be = val.to_be_bytes();
            bytes[i * 4..i * 4 + 4].copy_from_slice(&be);
        }
        bytes
    }
}
#[repr(C)]
pub struct PropertyDesc {
    pub value_size: u32,
    pub name_offset: u32,
}

pub fn align_buf<T>(buf: &mut [u8]) -> Result<&mut [u8]> {
    let off = buf.as_ptr() as usize % align_of::<T>();
    if off == 0 {
        return Ok(buf);
    }

    let inc = align_of::<T>() - off;
    if buf.len() < inc {
        return Err(Error::BufferTooSmall);
    }

    Ok(&mut buf[inc..])
}

#[cfg(test)]
#[macro_use]
mod tests {
    /// Creates a mutable byte slice view of any array type.
    /// Used in test infrastructure for alignment/padding tests.
    #[macro_export]
    macro_rules! aligned_buf {
        ($name:ident, $array:expr) => {
            let mut tmp = $array;
            #[allow(unused_mut)]
            let mut $name = {
                // SAFETY: Reinterpreting any array as a byte slice is valid.
                // The caller is responsible for ensuring the byte pattern is meaningful.
                #[allow(unsafe_code)]
                let result = unsafe {
                    core::slice::from_raw_parts_mut::<u8>(
                        tmp.as_mut_ptr() as *mut u8,
                        core::mem::size_of_val(&tmp),
                    )
                };
                result
            };
        };
    }
}
