#![allow(warnings)]

use crate::{Result, RustbootError};
use core::convert::TryFrom;
use core::ops::Add;

#[cfg(feature = "secp256k1")]
use k256::{
    ecdsa::{signature::DigestVerifier, Signature, VerifyingKey},
    elliptic_curve::consts::U32,
};
#[cfg(feature = "nistp256")]
use p256::{
    ecdsa::signature::digest::Digest,
    ecdsa::signature::digest::{FixedOutputDirty, Reset, Update},
    ecdsa::{signature::DigestVerifier, Signature, VerifyingKey},
    elliptic_curve::consts::U32,
    elliptic_curve::{generic_array::GenericArray, FieldSize},
    EncodedPoint, NistP256,
};

// NIST-P256 constants
#[cfg(feature = "nistp256")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0200;
// ECC-SECPK1 constants
#[cfg(feature = "secp256k1")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0000;
// ED25519 constants
#[cfg(feature = "ed25519")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0100;

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

/// Performs the signature verification; take as argument, a pre-updated
/// [`Digest`] instance thats needs to be finalized and the associated signature
/// to be verified.
pub fn verify_ecc256_signature<D, const N: u16>(digest: D, signature: &[u8]) -> Result<bool>
where
    D: Digest<OutputSize = U32>,
{
    match N {
        #[cfg(feature = "nistp256")]
        HDR_IMG_TYPE_AUTH => {
            if let VerifyingKeyTypes::VKeyNistP256(vk) = import_pubkey(PubkeyTypes::NistP256)? {
                let ecc256_verifier = NistP256Signature { verify_key: vk };
                let res = ecc256_verifier.verify(digest, signature)?;
                match res {
                    true => Ok(true),
                    false => Err(RustbootError::FwAuthFailed),
                }
            } else {
                Err(RustbootError::Unreachable)
            }
        }
        #[cfg(feature = "secp256k1")]
        HDR_IMG_TYPE_AUTH => {
            let ecc256_verifier = Secp256k1Signature {
                verify_key: import_pubkey(PubkeyTypes::Secp256k1)?,
            };
            let res = ecc256_verifier.verify(digest, signature)?;
            match res {
                true => Ok(true),
                false => Err(RustbootError::FwAuthFailed),
            }
        }
        #[cfg(feature = "ed25519")]
        HDR_IMG_TYPE_AUTH => todo!(),
        _ => todo!(),
    }
}

enum PubkeyTypes {
    #[allow(dead_code)]
    Secp256k1,
    #[allow(dead_code)]
    Ed25519,
    NistP256,
    #[allow(dead_code)]
    NistP384,
}

enum VerifyingKeyTypes {
    #[cfg(feature = "secp256k1")]
    VKey256k1(VerifyingKey),
    #[cfg(feature = "nistp256")]
    VKeyNistP256(VerifyingKey),
    #[allow(dead_code)]
    VKeyEd25519,
    #[allow(dead_code)]
    VKeyNistP384,
}

/// Imports a raw public key embedded in the bootloader.
///
/// *Note: this function can be extended to add support for HW
/// secure elements*
fn import_pubkey(pk: PubkeyTypes) -> Result<VerifyingKeyTypes> {
    match pk {
        #[cfg(feature = "secp256k1")]
        PubkeyTypes::Secp256k1 => {
            let embedded_pubkey = [0u8; 64];
            let untagged_bytes: &GenericArray<u8, <FieldSize<Secp256k1> as Add>::Output> =
                GenericArray::from_slice(&embedded_pubkey[..]);
            let sec1_encoded_pubkey = EncodedPoint::from_untagged_bytes(untagged_bytes);
            // `from_encoded_point` is fallible i.e. it will check to see if the point (i.e. pubkey) is on the curve.
            let secp256k1_vk = VerifyingKey::from_encoded_point(&sec1_encoded_pubkey)
                .map_err(|_| RustbootError::ECCError);
            Ok(VerifyingKeyTypes::VKey256k1(secp256k1_vk?))
        }
        #[cfg(feature = "nistp256")]
        PubkeyTypes::NistP256 => {
            let embedded_pubkey = [
                0x74, 0xBF, 0x5D, 0xE9, 0xF8, 0x69, 0x69, 0x44, 0x35, 0xAE, 0xB7, 0x39, 0x6F, 0xA1,
                0x40, 0x11, 0xB6, 0xA1, 0x7F, 0x2D, 0x8A, 0x86, 0xB9, 0x58, 0xBC, 0x4A, 0x51, 0xF7,
                0xF3, 0x0F, 0x23, 0x77, 0x78, 0x0E, 0x11, 0x46, 0x95, 0x3A, 0x1D, 0xDF, 0x69, 0xCD,
                0x34, 0x23, 0xFE, 0x63, 0x05, 0x15, 0x30, 0x43, 0xBB, 0x9E, 0x75, 0x63, 0xE0, 0x41,
                0x6A, 0x70, 0xCE, 0x16, 0x0A, 0x60, 0x2A, 0x38,
            ];
            let untagged_bytes: &GenericArray<u8, <FieldSize<NistP256> as Add>::Output> =
                GenericArray::from_slice(&embedded_pubkey[..]);
            let sec1_encoded_pubkey = EncodedPoint::from_untagged_bytes(untagged_bytes);
            // `from_encoded_point` is fallible i.e. it will check to see if the point (i.e. pubkey) is on the curve.
            let p256_vk = VerifyingKey::from_encoded_point(&sec1_encoded_pubkey)
                .map_err(|_| RustbootError::ECCError);
            Ok(VerifyingKeyTypes::VKeyNistP256(p256_vk?))
        }
        _ => todo!(),
    }
}
