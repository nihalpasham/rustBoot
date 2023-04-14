#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]
#![feature(is_sorted, slice_as_chunks, bigint_helper_methods)]

pub mod cfgparser;
#[cfg(feature = "mcu")]
pub mod constants;
pub mod crypto;
pub mod dt;
#[cfg(feature = "mcu")]
pub mod flashapi;
pub mod fs;
#[cfg(feature = "mcu")]
pub mod image;
#[cfg(feature = "mcu")]
pub mod parser;
pub mod rbconstants;

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The RustbootError type.
pub enum RustbootError {
    /// An operation is not permitted in the current state or an invalid state was reached.
    InvalidState,
    /// Firmware authentication failed
    FwAuthFailed,
    /// Image integrity verification failed.
    IntegrityCheckFailed,
    /// The val of the size field in an image header is not valid
    InvalidFirmwareSize,
    /// Type, length, value triple does not exist i.e. tried to parse the header
    /// for a given a `field_type` but we reached the `end of header`.
    TLVNotFound,
    /// The hash output or length is invalid .
    BadHashValue,
    /// The value of a field in a param packet was not set
    FieldNotSet,
    /// Error while performing an `EC Crypto operation`
    ECCError,
    /// The image is malformed. Ex: for mcu(s) this could be an invalid
    /// `magic` field or `trailer magic`
    InvalidImage,
    /// Something's wrong with the signature stored in the header.
    BadSignature,
    /// The version number of the img is invalid. For fit-images, this
    /// could be a case where the timestamp in the supplied fit-image does
    /// not match the `updt.txt` version.
    BadVersion,
    /// The value associated with the requested TLV is too large i.e. invalid.
    InvalidHdrFieldLength,
    /// Suppose to be unreachable
    Unreachable,
    /// Null value
    NullValue,
    /// The requested header field has an invalid value.
    InvalidValue,
    /// Attempt to reinitialize a global mutable static.  
    StaticReinit,
    /// The sector flag value is invalid
    InvalidSectFlag,

    #[doc(hidden)]
    __Nonexhaustive,
}

/// The result type for rustboot.
pub type Result<T> = core::result::Result<T, RustbootError>;

#[rustfmt::skip]
impl fmt::Display for RustbootError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RustbootError::InvalidState             => write!(f, "Invalid State, operation not permitted"),
            &RustbootError::FwAuthFailed             => write!(f, "Firmware authentication failed"),
            &RustbootError::IntegrityCheckFailed     => write!(f, "Integrity check failed"),
            &RustbootError::InvalidFirmwareSize      => write!(f, "Malformed Firmware"),
            &RustbootError::TLVNotFound              => write!(f, "Reached end of header options"),
            &RustbootError::BadHashValue             => write!(f, "Bad Hash"),
            &RustbootError::FieldNotSet              => write!(f, "The field is not set"),
            &RustbootError::ECCError                 => write!(f, "EC Crypto operation failed"),
            &RustbootError::InvalidImage             => write!(f, "The image is not a valid RUSTBOOT image"),
            &RustbootError::BadSignature             => write!(f, "Bad signature"),
            &RustbootError::BadVersion               => write!(f, "Bad image version of fit-image version mismatch"),
            &RustbootError::InvalidHdrFieldLength    => write!(f, "The length of the requested field is invalid"),
            &RustbootError::Unreachable              => write!(f, "An unreachable state was reached."),
            &RustbootError::NullValue                => write!(f, "got a NULL value"),
            &RustbootError::InvalidValue             => write!(f, "Header field has an invalid value"),
            &RustbootError::StaticReinit             => write!(f, "Cannot reinitialize global mutable static"),
            &RustbootError::InvalidSectFlag          => write!(f, "The sector flag value is invalid"),
            &RustbootError::__Nonexhaustive          => unreachable!(),
        }
    }
}
