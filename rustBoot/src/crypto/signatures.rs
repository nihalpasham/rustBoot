use crate::{Result, RustbootError};
use core::convert::TryFrom;
use sha2::digest::{Digest, FixedOutput};

#[cfg(feature = "secp256k1")]
use k256::{
    ecdsa::{Signature, VerifyingKey},
    elliptic_curve::consts::U32,
    Secp256k1,
};
#[cfg(feature = "nistp256")]
#[allow(deprecated)]
use p256::{
    ecdsa::{Signature, VerifyingKey},
    elliptic_curve::consts::U32,
    elliptic_curve::generic_array::typenum::U64,
    elliptic_curve::generic_array::GenericArray,
    EncodedPoint, NistP256,
};

// NIST-P256 constants
#[cfg(feature = "nistp256")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0200;
// ECC-SECPK1 constants
#[cfg(feature = "ed25519")]
pub const HDR_IMG_TYPE_AUTH: u16 = 0x0100;

#[cfg(feature = "nistp256")]
pub struct NistP256Signature {
    pub verify_key: VerifyingKey,
}

#[cfg(feature = "nistp256")]
impl NistP256Signature {
    pub fn verify<D>(self, digest: D, signature: &[u8]) -> Result<bool>
    where
        D: Digest + FixedOutput<OutputSize = U32>,
    {
        let sig = Signature::try_from(signature).map_err(|_| RustbootError::BadSignature)?;
        let res = <ecdsa::VerifyingKey<NistP256> as ecdsa::signature::DigestVerifier<D, ecdsa::Signature<NistP256>>>::verify_digest(&self.verify_key, digest, &sig).is_ok();
        Ok(res)
    }
}

#[cfg(feature = "secp256k1")]
pub struct Secp256k1Signature {
    pub verify_key: VerifyingKey,
}

#[cfg(feature = "secp256k1")]
impl Secp256k1Signature {
    pub fn verify<D>(self, digest: D, signature: &[u8]) -> Result<bool>
    where
        D: Digest + FixedOutput<OutputSize = U32>,
    {
        let sig = Signature::try_from(signature).map_err(|_| RustbootError::BadSignature)?;
        let res = <ecdsa::VerifyingKey<k256::Secp256k1> as ecdsa::signature::DigestVerifier<D, ecdsa::Signature<k256::Secp256k1>>>::verify_digest(&self.verify_key, digest, &sig).is_ok();
        Ok(res)
    }
}

pub fn verify_ecc256_signature<D, const N: u16>(digest: D, signature: &[u8]) -> Result<bool>
where
    D: Digest + FixedOutput<OutputSize = U32>,
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
        HDR_IMG_TYPE_AUTH => Err(RustbootError::InvalidValue),
        _ => Err(RustbootError::InvalidValue),
    }
}

pub enum PubkeyTypes {
    #[allow(dead_code)]
    Secp256k1,
    #[allow(dead_code)]
    Ed25519,
    NistP256,
    #[allow(dead_code)]
    NistP384,
}

pub enum VerifyingKeyTypes {
    #[cfg(feature = "secp256k1")]
    VKey256k1(VerifyingKey),
    #[cfg(feature = "nistp256")]
    VKeyNistP256(VerifyingKey),
    #[allow(dead_code)]
    VKeyEd25519,
    #[allow(dead_code)]
    VKeyNistP384,
}

pub fn import_pubkey(pk: PubkeyTypes) -> Result<VerifyingKeyTypes> {
    match pk {
        #[cfg(feature = "secp256k1")]
        PubkeyTypes::Secp256k1 => {
            let embedded_pubkey = [0u8; 64];
            #[allow(deprecated)]
            let untagged_bytes: &GenericArray<u8, U64> =
                GenericArray::from_slice(&embedded_pubkey[..]);
            let sec1_encoded_pubkey = EncodedPoint::from_untagged_bytes(untagged_bytes);
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
            #[allow(deprecated)]
            let untagged_bytes: &GenericArray<u8, U64> =
                GenericArray::from_slice(&embedded_pubkey[..]);
            let sec1_encoded_pubkey = EncodedPoint::from_untagged_bytes(untagged_bytes);
            let p256_vk = VerifyingKey::from_encoded_point(&sec1_encoded_pubkey)
                .map_err(|_| RustbootError::ECCError);
            Ok(VerifyingKeyTypes::VKeyNistP256(p256_vk?))
        }
        _ => Err(RustbootError::InvalidValue),
    }
}