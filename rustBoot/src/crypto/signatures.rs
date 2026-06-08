//! ECDSA signature verification (NIST P-256, secp256k1).
//!
//! Supports digest-based verification via the `signature` crate's
//! `DigestVerifier` trait. The embedded public key is provisioned
//! at compile time.

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
// REQUIRED: elliptic_curve::generic_array re-export is deprecated upstream.
// EncodedPoint::from_untagged_bytes() requires this specific type.
// Will auto-resolve on elliptic-curve v0.14 (in pre-release).
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
        let res = <ecdsa::VerifyingKey<NistP256> as ecdsa::signature::DigestVerifier<
            D,
            ecdsa::Signature<NistP256>,
        >>::verify_digest(&self.verify_key, digest, &sig)
        .is_ok();
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
        let res = <ecdsa::VerifyingKey<k256::Secp256k1> as ecdsa::signature::DigestVerifier<
            D,
            ecdsa::Signature<k256::Secp256k1>,
        >>::verify_digest(&self.verify_key, digest, &sig)
        .is_ok();
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
            // Upstream elliptic-curve re-export deprecation (auto-resolves in v0.14)
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
            // Upstream elliptic-curve re-export deprecation (auto-resolves in v0.14)
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

#[cfg(test)]
#[cfg(feature = "nistp256")]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use p256::ecdsa::{Signature, SigningKey};
    use sha2::digest::Digest;
    use sha2::Sha256;
    use signature::DigestSigner;

    fn test_signing_key_1() -> SigningKey {
        let sk_bytes = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        SigningKey::from_slice(&sk_bytes).expect("valid test signing key")
    }

    fn test_signing_key_2() -> SigningKey {
        let sk_bytes = [
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        SigningKey::from_slice(&sk_bytes).expect("valid test signing key")
    }

    #[test]
    fn nistp256_verify_good_signature() {
        let signing_key = test_signing_key_1();
        let verify_key = VerifyingKey::from(&signing_key);
        let verifier = NistP256Signature { verify_key };

        let message = b"test firmware image data for rustBoot";
        let digest = Sha256::new().chain_update(message);
        let sig: Signature = signing_key.sign_digest(digest);

        let verify_digest = Sha256::new().chain_update(message);
        let result = verifier.verify::<Sha256>(verify_digest, sig.to_bytes().as_ref());
        assert!(result.unwrap());
    }

    #[test]
    fn nistp256_verify_bad_signature() {
        let signing_key = test_signing_key_1();
        let verify_key = VerifyingKey::from(&signing_key);
        let verifier = NistP256Signature { verify_key };

        let bad_sig = [0xabu8; 64];
        let digest = Sha256::new().chain_update(b"irrelevant");

        let result = verifier.verify::<Sha256>(digest, &bad_sig);
        assert!(!result.unwrap());
    }

    #[test]
    fn nistp256_verify_malformed_signature() {
        let signing_key = test_signing_key_1();
        let verify_key = VerifyingKey::from(&signing_key);
        let verifier = NistP256Signature { verify_key };

        let truncated_sig = [0x01u8; 10];
        let digest = Sha256::new().chain_update(b"msg");

        let result = verifier.verify::<Sha256>(digest, &truncated_sig);
        assert!(matches!(result, Err(RustbootError::BadSignature)));
    }

    #[test]
    fn nistp256_verify_wrong_key() {
        let sk1 = test_signing_key_1();
        let wrong_sk = test_signing_key_2();
        let verify_key = VerifyingKey::from(&wrong_sk);
        let verifier = NistP256Signature { verify_key };

        let message = b"signed by different key";
        let digest = Sha256::new().chain_update(message);
        let sig: Signature = sk1.sign_digest(digest);

        let verify_digest = Sha256::new().chain_update(message);
        let result = verifier.verify::<Sha256>(verify_digest, sig.to_bytes().as_ref());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn import_pubkey_nistp256_ok() {
        let result = import_pubkey(PubkeyTypes::NistP256);
        assert!(result.is_ok());
        match result.unwrap() {
            VerifyingKeyTypes::VKeyNistP256(_) => {}
            _ => panic!("expected NistP256 verifying key"),
        }
    }

    #[test]
    fn import_pubkey_unsupported_returns_error() {
        let result = import_pubkey(PubkeyTypes::Ed25519);
        assert!(matches!(result, Err(RustbootError::InvalidValue)));
    }

    #[test]
    fn verify_ecc256_bad_sig_returns_auth_failed() {
        let bad_sig = [0xdeu8; 64];
        let digest = Sha256::new().chain_update(b"test");
        let result = verify_ecc256_signature::<Sha256, { HDR_IMG_TYPE_AUTH }>(digest, &bad_sig);
        assert!(matches!(result, Err(RustbootError::FwAuthFailed)));
    }

    #[test]
    fn import_pubkey_nistp384_returns_error() {
        let result = import_pubkey(PubkeyTypes::NistP384);
        assert!(matches!(result, Err(RustbootError::InvalidValue)));
    }

    #[test]
    fn import_pubkey_secp256k1_returns_error() {
        let result = import_pubkey(PubkeyTypes::Secp256k1);
        assert!(matches!(result, Err(RustbootError::InvalidValue)));
    }

    #[test]
    fn verify_ecc256_zero_length_signature() {
        let empty_sig = [];
        let digest = Sha256::new().chain_update(b"test");
        let result = verify_ecc256_signature::<Sha256, { HDR_IMG_TYPE_AUTH }>(digest, &empty_sig);
        assert!(matches!(result, Err(RustbootError::BadSignature)));
    }

    #[test]
    fn verify_ecc256_invalid_algorithm_id() {
        let sig = [0xabu8; 64];
        let digest = Sha256::new().chain_update(b"test");
        let result = verify_ecc256_signature::<Sha256, 0xFFFF>(digest, &sig);
        assert!(matches!(result, Err(RustbootError::InvalidValue)));
    }

    #[test]
    fn import_pubkey_nistp256_returns_correct_variant() {
        let result = import_pubkey(PubkeyTypes::NistP256).unwrap();
        match result {
            VerifyingKeyTypes::VKeyNistP256(_) => {}
            _ => panic!("expected VKeyNistP256"),
        }
    }

    #[test]
    fn import_pubkey_nistp256_all_zero_key_handles_gracefully() {
        // NistP256 uses a real hardcoded key, so this should succeed
        // (the embedded key is valid). This test documents the current behavior.
        let result = import_pubkey(PubkeyTypes::NistP256);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_ecc256_truncated_signature_returns_bad_signature() {
        let truncated = [0x01u8; 1];
        let digest = Sha256::new().chain_update(b"test");
        let result = verify_ecc256_signature::<Sha256, { HDR_IMG_TYPE_AUTH }>(digest, &truncated);
        assert!(matches!(result, Err(RustbootError::BadSignature)));
    }

    #[test]
    fn verify_ecc256_one_byte_signature_returns_bad_signature() {
        let one_byte = [0x42u8];
        let digest = Sha256::new().chain_update(b"x");
        let result = verify_ecc256_signature::<Sha256, { HDR_IMG_TYPE_AUTH }>(digest, &one_byte);
        assert!(matches!(result, Err(RustbootError::BadSignature)));
    }

    #[test]
    fn verify_ecc256_signature_wrong_algorithm_not_feature_gated() {
        let sig = [0xabu8; 64];
        let digest = Sha256::new().chain_update(b"test");
        // 0x0100 would be ed25519, not the nistp256 feature
        let result = verify_ecc256_signature::<Sha256, 0x0100>(digest, &sig);
        assert!(matches!(result, Err(RustbootError::InvalidValue)));
    }
}
