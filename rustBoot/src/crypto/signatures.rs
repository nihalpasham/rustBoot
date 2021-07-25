#![no_std]
#![allow(warnings)]

use crate::{Result, RustbootError};
use core::convert::{TryFrom, TryInto};

use k256::{
    ecdsa::{digest::Digest, signature::DigestVerifier, Signature, VerifyingKey},
    elliptic_curve::consts::U32,
    EncodedPoint,
};

/// A type to represent an ECDSA-SHA256 Signature
#[cfg(feature = "secp256k1")]
pub struct Secp256k1Signature(pub [u8; 64]);

#[cfg(feature = "secp256k1")]
impl Secp256k1Signature {
    /// Verifies an ECDSA signature. This method is intended to take as 
    /// argument, a pre-updated [`Digest`] instance thats needs to be finalized.
    ///
    /// Returns a `bool` if successful else an error.
    pub fn verify<D: Digest<OutputSize = U32>>(self, digest: D, signature: &[u8]) -> Result<bool> {
        let sec1_encoded_pubkey =
            EncodedPoint::from_bytes(self.0).map_err(|_| RustbootError::ECCError)?;
        let verify_key = VerifyingKey::from_encoded_point(&sec1_encoded_pubkey)
            .map_err(|_| RustbootError::ECCError)?;
        Ok(verify_key
            .verify_digest(
                digest,
                &Signature::try_from(signature).map_err(|_| RustbootError::BadSignature)?,
            )
            .is_ok())
    }
}
