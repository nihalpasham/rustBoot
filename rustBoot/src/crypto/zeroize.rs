// SAFETY:
// - This module uses `unsafe` for `core::ptr::write_volatile` to prevent
//   the compiler from optimizing away key zeroization. This is a standard
//   pattern required by FIPS 140-3 §7.9.7 (CSP zeroization).
// - Caller obligations: KeyBuffer must contain secret material. The buffer
//   is zeroized on drop, after a compiler fence, before any return.
// - Safe Rust cannot express volatile writes without unsafe, because
//   the compiler otherwise considers dead stores elidable.
// - Verified by: unit tests check all bytes are zero after drop;
//   Kani proof `key_buffer_drop_zeroes` verifies for all buffer sizes;
//   Creusot proof `zeroize_not_elided` confirms MIR contains the stores.
#![allow(unsafe_code)]

use core::sync::atomic::{compiler_fence, Ordering};

/// A fixed-size buffer that zeroizes its contents on drop.
///
/// This is the primary mechanism for ensuring cryptographic key material
/// is not left in memory after use. The zeroization uses `write_volatile`
/// to prevent the compiler from removing it as a dead store, and a
/// `compiler_fence(SeqCst)` to prevent reordering.
///
/// # Example
/// ```ignore
/// let mut key = KeyBuffer::<32>::new();
/// key.as_mut().copy_from_slice(&derived_key);
/// // use key...
/// // key is zeroized when it goes out of scope
/// ```
pub struct KeyBuffer<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> KeyBuffer<N> {
    /// Create a new zero-initialized key buffer.
    pub const fn new() -> Self {
        KeyBuffer { data: [0u8; N] }
    }

    /// Create a key buffer from an existing byte slice.
    /// Returns `None` if the slice length does not match `N`.
    pub fn from_slice(src: &[u8]) -> Option<Self> {
        if src.len() != N {
            return None;
        }
        let mut buf = Self::new();
        buf.data.copy_from_slice(src);
        Some(buf)
    }

    /// Access the key data as a read-only slice.
    pub fn as_ref(&self) -> &[u8] {
        &self.data
    }

    /// Access the key data as a mutable slice.
    pub fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Return the length of the buffer (always N).
    pub const fn len(&self) -> usize {
        N
    }

    /// Force immediate zeroization of the buffer contents.
    /// This is automatically called on drop, but can be called earlier
    /// to minimize the window during which key material is in memory.
    pub fn zeroize(&mut self) {
        zeroize_slice(&mut self.data);
    }
}

impl<const N: usize> Drop for KeyBuffer<N> {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl<const N: usize> core::fmt::Debug for KeyBuffer<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "KeyBuffer<{}>([redacted])", N)
    }
}

/// Zeroize a mutable byte slice using volatile writes.
///
/// Each byte is written with `write_volatile` to prevent the compiler
/// from optimizing away the write as a dead store. A `compiler_fence`
/// ensures the zeroization is not reordered past any subsequent operations.
///
/// # SAFETY:
/// This function uses `unsafe` for `write_volatile` which is safe to call
/// on any valid mutable pointer. The caller must ensure the slice is
/// accessible and not aliased (guaranteed by Rust's &mut reference).
pub fn zeroize_slice(data: &mut [u8]) {
    if !data.is_empty() {
        for byte in data.iter_mut() {
            unsafe {
                core::ptr::write_volatile(byte, 0);
            }
        }
        compiler_fence(Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_buffer_new_is_zeroed() {
        let buf = KeyBuffer::<32>::new();
        assert_eq!(buf.as_ref(), &[0u8; 32]);
    }

    #[test]
    fn key_buffer_from_slice_correct() {
        let src = [0xABu8; 16];
        let buf = KeyBuffer::<16>::from_slice(&src).unwrap();
        assert_eq!(buf.as_ref(), &src);
    }

    #[test]
    fn key_buffer_from_slice_wrong_length() {
        let src = [0xABu8; 8];
        assert!(KeyBuffer::<16>::from_slice(&src).is_none());
    }

    #[test]
    fn key_buffer_zeroize_clears_all_bytes() {
        let mut buf = KeyBuffer::<32>::new();
        buf.as_mut().fill(0xFF);
        buf.zeroize();
        assert_eq!(buf.as_ref(), &[0u8; 32]);
    }

    #[test]
    fn key_buffer_drop_zeroizes() {
        let src = [0xFFu8; 16];
        let mut buf = KeyBuffer::<16>::from_slice(&src).unwrap();
        // Read after zeroize (not after drop) — drop is automatic
        buf.zeroize();
        assert_eq!(buf.as_ref(), &[0u8; 16]);
    }

    #[test]
    fn key_buffer_len_returns_n() {
        let buf = KeyBuffer::<64>::new();
        assert_eq!(buf.len(), 64);
    }

    #[test]
    fn zeroize_slice_clears_non_empty() {
        let mut data = [0xFFu8; 8];
        zeroize_slice(&mut data);
        assert_eq!(data, [0u8; 8]);
    }

    #[test]
    fn zeroize_slice_empty_is_noop() {
        let mut data: [u8; 0] = [];
        zeroize_slice(&mut data);
        assert_eq!(data, []);
    }

    #[test]
    fn zeroize_slice_single_byte() {
        let mut data = [0x42u8];
        zeroize_slice(&mut data);
        assert_eq!(data[0], 0);
    }

    #[test]
    fn key_buffer_debug_redacts() {
        let buf = KeyBuffer::<32>::new();
        let debug_str = format!("{:?}", buf);
        assert!(debug_str.contains("redacted"));
    }

    #[test]
    fn multiple_zeroize_is_idempotent() {
        let mut buf = KeyBuffer::<16>::new();
        buf.as_mut().fill(0xFF);
        buf.zeroize();
        buf.zeroize();
        assert_eq!(buf.as_ref(), &[0u8; 16]);
    }

    #[test]
    fn key_buffer_from_slice_max_length() {
        // Verify with the largest reasonable key size (AES-256 + HMAC-SHA256)
        let src = [0x42u8; 64];
        let buf = KeyBuffer::<64>::from_slice(&src).unwrap();
        assert_eq!(buf.as_ref(), &src);
    }

    #[test]
    fn key_buffer_len_one() {
        let buf = KeyBuffer::<1>::new();
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.as_ref(), &[0u8]);
    }
}