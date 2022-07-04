use crate::curve::*;
use sha2::Sha256;
use signature::DigestSigner;

use as_slice::AsSlice;
use rustBoot::dt::{prepare_img_hash, update_dtb_header, Reader};

/// Retruns a signed fit-image, given a image tree blob, a signing key and the curve type. Only supports `elliptic curve crypto`
///
/// NOTE:
/// - the image tree blob must be a `rustBoot` compliant fit-image.
///
pub fn sign_fit(itb_blob: Vec<u8>, sk_type: SigningKeyType) -> Result<Vec<u8>> {
    let signed_itb_blob = match sk_type {
        #[cfg(feature = "nistp256")]
        SigningKeyType::NistP256(sk) => {
            let (prehashed_digest, _) = prepare_img_hash::<Sha256, 32, 64, 4>(itb_blob.as_slice())
                .map_err(|_v| RbSignerError::BadHashValue)?;
            let signature = sk
                .try_sign_digest(prehashed_digest)
                .map_err(|v| RbSignerError::SignatureError(v))?;
            println!("signature: {:?}", signature);
            set_config_signature(itb_blob, SignatureType::NistP256(signature), "bootconfig")
        }
        #[cfg(feature = "ed25519")]
        SigningKeyType::Ed25519 => {
            todo!()
        }
        _ => return Err(RbSignerError::InvalidKeyType),
    };
    signed_itb_blob
}

fn set_config_signature(
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
            let sig_len: [u8; 4] = (bytes.len() as u32).to_le_bytes();
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
