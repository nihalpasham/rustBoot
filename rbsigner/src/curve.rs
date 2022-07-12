#[cfg(feature = "nistp256")]
use p256::ecdsa::{Signature, SigningKey};
use rustBoot::dt::Error as ITBError;
use signature::Error as SigningError;

#[derive(Debug)]
pub enum CurveType {
    #[allow(dead_code)]
    Secp256k1,
    #[allow(dead_code)]
    Ed25519,
    NistP256,
    #[allow(dead_code)]
    NistP384,
}

#[derive(Debug)]
pub enum SigningKeyType {
    #[cfg(feature = "secp256k1")]
    Secp256k1(SigningKey),
    #[cfg(feature = "nistp256")]
    NistP256(SigningKey),
    #[allow(dead_code)]
    Ed25519,
    #[allow(dead_code)]
    NistP384,
}

#[derive(Debug)]
pub enum SignatureType {
    #[cfg(feature = "secp256k1")]
    Secp256k1(Signature),
    #[cfg(feature = "nistp256")]
    NistP256(Signature),
    #[allow(dead_code)]
    Ed25519,
    #[allow(dead_code)]
    NistP384,
}

/// Imports a signing key .
///
/// *Note: this function can be extended to add support for HW
/// secure elements*
///
pub fn import_signing_key(curve: CurveType, bytes: &[u8]) -> Result<SigningKeyType> {
    match curve {
        #[cfg(feature = "secp256k1")]
        CurveType::Secp256k1 => {}
        #[cfg(feature = "nistp256")]
        CurveType::NistP256 => {
            let sk = SigningKey::from_bytes(bytes).map_err(|v| RbSignerError::KeyError(v))?;
            Ok(SigningKeyType::NistP256(sk))
        }
        _ => todo!(),
    }
}

/// The result type for rbSigner.
pub type Result<T> = core::result::Result<T, RbSignerError>;

#[derive(Debug)]
pub enum RbSignerError {
    /// Invalid fit-image header
    BadImageHeader(ITBError),
    /// The hash output or length is invalid .
    BadHashValue,
    /// Signature Error
    SignatureError(SigningError),
    /// Key Error
    KeyError(SigningError),
    /// An invalid key type was provided
    InvalidKeyType,
    #[doc(hidden)]
    __Nonexhaustive,
}
