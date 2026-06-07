#[cfg(feature = "nistp256")]
use p256::ecdsa::{Signature, SigningKey};
use rustBoot::dt::Error as ITBError;
use signature::Error as SigningError;
use std::fmt;

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        #[allow(dead_code)]
        CurveType::Secp256k1 => Err(RbSignerError::InvalidKeyType),
        #[cfg(feature = "nistp256")]
        CurveType::NistP256 => {
            let sk = SigningKey::from_slice(bytes).map_err(RbSignerError::KeyError)?;
            Ok(SigningKeyType::NistP256(sk))
        }
        _ => Err(RbSignerError::InvalidKeyType),
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
    /// IO Error
    IoError(std::io::Error),
    /// Invalid header size
    InvalidHeaderSize,
    /// Invalid version length
    InvalidVersionLength,
    /// Invalid timestamp length
    InvalidTimestampLength,
    /// Invalid image type length
    InvalidImageTypeLength,
    /// Invalid digest length
    InvalidDigestLength,
    /// Invalid pubkey digest length
    InvalidPubkeyDigestLength,
    /// Invalid timestamp ordering
    InvalidTimestampOrdering,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for RbSignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RbSignerError::BadImageHeader(e) => write!(f, "bad image header: {:?}", e),
            RbSignerError::BadHashValue => write!(f, "bad hash value"),
            RbSignerError::SignatureError(e) => write!(f, "signature error: {}", e),
            RbSignerError::KeyError(e) => write!(f, "key error: {}", e),
            RbSignerError::InvalidKeyType => write!(f, "invalid key type"),
            RbSignerError::IoError(e) => write!(f, "IO error: {}", e),
            RbSignerError::InvalidHeaderSize => write!(f, "invalid header size"),
            RbSignerError::InvalidVersionLength => write!(f, "invalid version length"),
            RbSignerError::InvalidTimestampLength => write!(f, "invalid timestamp length"),
            RbSignerError::InvalidImageTypeLength => write!(f, "invalid image type length"),
            RbSignerError::InvalidDigestLength => write!(f, "invalid digest length"),
            RbSignerError::InvalidPubkeyDigestLength => write!(f, "invalid pubkey digest length"),
            RbSignerError::InvalidTimestampOrdering => write!(f, "invalid timestamp ordering"),
            RbSignerError::__Nonexhaustive => write!(f, "unknown error"),
        }
    }
}

impl std::error::Error for RbSignerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RbSignerError::BadImageHeader(_) => None,
            RbSignerError::BadHashValue => None,
            RbSignerError::SignatureError(_) => None,
            RbSignerError::KeyError(_) => None,
            RbSignerError::InvalidKeyType => None,
            RbSignerError::IoError(e) => Some(e),
            RbSignerError::InvalidHeaderSize => None,
            RbSignerError::InvalidVersionLength => None,
            RbSignerError::InvalidTimestampLength => None,
            RbSignerError::InvalidImageTypeLength => None,
            RbSignerError::InvalidDigestLength => None,
            RbSignerError::InvalidPubkeyDigestLength => None,
            RbSignerError::InvalidTimestampOrdering => None,
            RbSignerError::__Nonexhaustive => None,
        }
    }
}

impl From<std::io::Error> for RbSignerError {
    fn from(e: std::io::Error) -> Self {
        RbSignerError::IoError(e)
    }
}
