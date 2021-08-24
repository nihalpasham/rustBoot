#![no_std]
#![allow(warnings)]

use crate::{Result, RustbootError};
use core::convert::{TryFrom, TryInto};

#[cfg(feature = "secp256k1")]
use k256::{
    ecdsa::{signature::DigestVerifier, Signature, VerifyingKey},
    elliptic_curve::consts::U32,
};
#[cfg(feature = "nistp256")]
use p256::{
    ecdsa::{signature::DigestVerifier, Signature, VerifyingKey},
    elliptic_curve::consts::U32,
};
use sha2::Digest;

/// A type to represent an ECDSA-SHA256 Signature
#[cfg(feature = "nistp256")]
pub struct NistP256Signature {
    pub verify_key: VerifyingKey,
}

#[cfg(feature = "nistp256")]
impl NistP256Signature {
    /// Verifies an ECDSA signature. This method is intended to take as
    /// argument, a pre-updated [`Digest`] instance thats needs to be finalized.
    ///
    /// Returns a `bool` if successful else an error.
    pub fn verify<D: Digest<OutputSize = U32>>(self, digest: D, signature: &[u8]) -> Result<bool> {
        let res = self
            .verify_key
            .verify_digest(
                digest,
                &Signature::try_from(signature).map_err(|_| RustbootError::BadSignature)?,
            )
            .is_ok();
        Ok(res)
    }
}

/// A type to represent an ECDSA-SHA256 Signature
#[cfg(feature = "secp256k1")]
pub struct Secp256k1Signature {
    pub verify_key: VerifyingKey,
}

#[cfg(feature = "secp256k1")]
impl Secp256k1Signature {
    /// Verifies an ECDSA signature. This method is intended to take as
    /// argument, a pre-updated [`Digest`] instance thats needs to be finalized.
    ///
    /// Returns a `bool` if successful else an error.
    pub fn verify<D: Digest<OutputSize = U32>>(self, digest: D, signature: &[u8]) -> Result<bool> {
        defmt::info!("secp256k1_enter");
        let res = self
            .verify_key
            .verify_digest(
                digest,
                &Signature::try_from(signature).map_err(|_| RustbootError::BadSignature)?,
            )
            .is_ok();
        defmt::info!("verify_actual={}", res);
        Ok(res)
    }
}
