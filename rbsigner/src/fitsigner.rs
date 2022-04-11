#[cfg(feature = "nistp256")]
use p256::ecdsa::{signature::DigestSigner, Signature, SigningKey};
use sha2::Sha256;
use signature::Error as SigningError;

use as_slice::AsSlice;
use rustBoot::dt::{prepare_img_hash, update_dtb_header, Error as ITBError, Reader};

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
/// Retruns a signed fit-image, given a image tree blob, a signing key and the curve type. Only supports `elliptic curve crypto`
///
/// NOTE:
/// - the image tree blob must be a `rustBoot` compliant fit-image.
///
pub fn sign_fit(curve: CurveType, itb_blob: Vec<u8>, sk_type: SigningKeyType) -> Result<Vec<u8>> {
    match curve {
        #[cfg(feature = "secp256k1")]
        CurveType::Secp256k1 => {}
        #[cfg(feature = "nistp256")]
        CurveType::NistP256 => {
            let (prehashed_digest, _) = prepare_img_hash::<Sha256, 32, 64, 4>(itb_blob.as_slice())
                .map_err(|_v| RbSignerError::BadHashValue)?;
            let signature = match sk_type {
                SigningKeyType::NistP256(sk) => {
                    let signature = sk
                        .try_sign_digest(prehashed_digest)
                        .map_err(|v| RbSignerError::SignatureError(v))?;
                    println!("signature: {:?}", signature);
                    set_config_signature(itb_blob, SignatureType::NistP256(signature), "bootconfig")
                }
                _ => return Err(RbSignerError::InvalidKeyType),
            };
            signature
        }
        _ => todo!(),
    }
}

pub fn set_config_signature(
    mut itb_blob: Vec<u8>,
    signature: SignatureType,
    config_name: &str,
) -> Result<Vec<u8>> {
    let reader = Reader::read(itb_blob.as_slice()).unwrap();
    let root = reader.struct_items();
    let (_node, node_iter) = root
        .path_struct_items(format!("/configurations/{}/signature/value", config_name).as_str())
        .next()
        .expect("config_name does not exist");

    let mut header =
        Reader::get_header(itb_blob.as_slice()).map_err(|e| RbSignerError::BadImageHeader(e))?;
    let struct_offset = header.struct_offset as usize;
    let offset = node_iter.get_offset() + struct_offset;
    let sig_len_offset = offset - 12;

    match signature {
        SignatureType::NistP256(sig) => {
            let bytes = sig.as_ref();
            let sig_len: [u8; 4] = (bytes.len() as u32).to_be_bytes();
            // update len field for signature's value property
            let _ = &itb_blob[sig_len_offset..sig_len_offset + 4]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, byte)| *byte = sig_len[idx]);

            let remaining = itb_blob.split_off(offset);
            let _ = itb_blob.split_off(offset - 4);
            itb_blob.extend_from_slice(bytes);
            itb_blob.extend_from_slice(remaining.as_slice());
            // update itb header
            let _ = update_dtb_header(&mut header, 0, 64, 4);
            let header_slice = header.as_slice();
            let _ = &itb_blob[..header.len()]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, byte)| *byte = header_slice[idx]);
            // let x = &itb_blob.as_slice()[(sig_len_offset - 4)..];
            // println!("blob_bytes: {:?}", x);
            Ok(itb_blob)
        }
        _ => {
            todo!()
        }
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
