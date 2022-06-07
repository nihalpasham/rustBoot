use super::{Concat, Error, Reader, Result};
use core::convert::TryInto;
use core::ops::Add;
use log::info;
use nom::AsBytes;
use p256::ecdsa::signature::digest::Digest;
use p256::elliptic_curve::generic_array::ArrayLength;
use sha2::Sha256;

use crate::crypto::signatures::{verify_ecc256_signature, HDR_IMG_TYPE_AUTH};

#[derive(Debug)]
#[repr(C)]
pub struct Config<'a, const S: usize> {
    description: &'a str,
    kernel: &'a str,
    fdt: &'a str,
    ramdisk: &'a str,
    rbconfig: &'a str,
    signature: Signature<'a, S>,
}

impl<'a, const S: usize> Default for Config<'a, S> {
    fn default() -> Self {
        Config {
            description: "none",
            kernel: "none",
            fdt: "none",
            ramdisk: "none",
            rbconfig: "none",
            signature: Signature {
                value: [0; S],
                algo: "none",
                key_hint: "none",
                signed_images: "none",
            },
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Signature<'a, const S: usize> {
    value: [u8; S],
    algo: &'a str,
    key_hint: &'a str,
    signed_images: &'a str,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Image<'a, const H: usize> {
    description: &'a str,
    typ: &'a str,
    arch: &'a str,
    os: Option<&'a str>,
    compression: &'a str,
    load: Option<u32>,
    entry: Option<u32>,
    hash: Hash<'a, H>,
}

impl<'a, const H: usize> Default for Image<'a, H> {
    fn default() -> Self {
        Image {
            description: "none",
            typ: "none",
            arch: "none",
            os: None,
            compression: "none",
            load: None,
            entry: None,
            hash: Hash {
                value: [0; H],
                algo: "none",
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Hash<'a, const H: usize> {
    value: [u8; H],
    algo: &'a str,
}

#[derive(Debug)]
#[repr(C)]
pub struct Images<'a, const H: usize, const N: usize> {
    images: [Image<'a, H>; N],
}

#[derive(Debug)]
pub enum CurveType {
    #[allow(dead_code)]
    Secp256k1,
    #[allow(dead_code)]
    Ed25519,
    NistP256,
    #[allow(dead_code)]
    NistP384,
    None,
}

pub fn parse_fit<D, const H: usize, const S: usize, const N: usize>(
    reader: Reader,
) -> Result<(Config<S>, Images<H, N>)>
where
    D: Digest,
    <D as Digest>::OutputSize: Add,
    <<D as Digest>::OutputSize as Add>::Output: ArrayLength<u8>,
{
    let mut configuration = Config::default();
    let mut images = [Image::default(); N];
    let root = reader.struct_items();
    let (_, node_iter) = root.path_struct_items("/configurations").next().unwrap();

    // *** Find the default config ***
    if let Some(config) = node_iter.get_node_property("default") {
        // parse the default config
        let config = "/configurations/".concat::<50>(config);
        let config = config.as_str()?;
        #[cfg(feature = "defmt")]
        defmt::info!("config: {:?}", config);

        let (_, node_iter) = root.path_struct_items(config).next().unwrap();
        let config_properties = [
            "description",
            "kernel",
            "fdt",
            "ramdisk",
            "rbconfig",
            "signature@1",
        ];
        let mut description = None;
        let mut kernel = None;
        let mut fdt = None;
        let mut ramdisk = None;
        let mut rbconfig = None;
        let mut signature_algo = None;
        let mut key_hint = None;
        let mut signed_images = None;
        let mut signature = None;

        let _ = config_properties.iter().for_each(|prop| match *prop {
            "description" => {
                let desc = node_iter.get_node_property(prop);
                description = desc
            }
            "kernel" => {
                let krnl = node_iter.get_node_property(prop);
                kernel = krnl
            }
            "fdt" => {
                let dt = node_iter.get_node_property(prop);
                fdt = dt
            }
            "ramdisk" => {
                let rd = node_iter.get_node_property(prop);
                ramdisk = rd
            }
            "rbconfig" => {
                let rbconf = node_iter.get_node_property(prop);
                rbconfig = rbconf
            }
            "signature@1" => {
                for item in node_iter {
                    if item.is_property() {
                        match item.name() {
                            Ok(val) if val == "algo" => {
                                signature_algo = Some(item.value().unwrap());
                            }
                            Ok(val) if val == "key-name-hint" => {
                                key_hint = Some(item.value().unwrap());
                            }
                            Ok(val) if val == "signed-images" => {
                                signed_images = Some(item.value().unwrap());
                            }
                            Ok(val) if val == "value" => {
                                signature = Some(item.value().unwrap());
                            }
                            _ => {}
                        }
                    } else if item.is_end_node() {
                        break;
                    }
                }
            }
            _ => {}
        });

        let signature = match signature {
            Some(val) => {
                if val == &[0x00] {
                    [0u8; S]
                } else {
                    let signature: [u8; S] = val.try_into().map_err(|_v| Error::BadU32List)?;
                    signature
                }
            }
            None => return Err(Error::BadPropertyName),
        };

        let signature: Signature<S> = Signature {
            value: signature,
            algo: as_str(signature_algo.unwrap())?.expect("algo not specified in itb"),
            key_hint: as_str(key_hint.unwrap())?.expect("key_hint not specified in itb"),
            signed_images: as_str(signed_images.unwrap())?
                .expect("signed images list not specified in itb"),
        };
        let config = Config {
            description: as_str(description.unwrap())?.expect("missing config description field"),
            kernel: as_str(kernel.unwrap())?.expect("kernel not specified in itb"),
            fdt: as_str(fdt.unwrap())?.expect("fdt not specified in itb"),
            ramdisk: as_str(ramdisk.unwrap())?.expect("ramdisk not specified in itb"),
            rbconfig: as_str(rbconfig.unwrap())?.expect("rbconfig not specified in itb"),
            signature,
        };
        configuration = config;
        #[cfg(feature = "defmt")]
        defmt::info!("Config: {:?}\n", configuration);

        let conf_properties = ["kernel", "fdt", "ramdisk", "rbconfig"];
        for (idx, prop) in conf_properties.iter().enumerate() {
            match node_iter.get_node_property(prop) {
                Some(val) => {
                    let img = "/images/".concat::<50>(val);
                    let img = img.as_str()?;
                    #[cfg(feature = "defmt")]
                    defmt::info!("img: {:?}", img);

                    let (_, node_iter) = root.path_struct_items(img).next().unwrap();
                    let img_properties = [
                        "description",
                        "data",
                        "type",
                        "arch",
                        "os",
                        "compression",
                        "load",
                        "entry",
                    ];
                    let mut description = None;
                    let mut data = None;
                    let mut typ = None;
                    let mut arch = None;
                    let mut os = None;
                    let mut compression = None;
                    let mut load = None;
                    let mut entry = None;

                    let _ = img_properties.iter().for_each(|prop| match *prop {
                        "description" => {
                            let val = node_iter.get_node_property(prop);
                            description = val
                        }
                        "data" => {
                            let val = node_iter.get_node_property(prop);
                            data = val
                        }
                        "type" => {
                            let val = node_iter.get_node_property(prop);
                            typ = val
                        }
                        "arch" => {
                            let val = node_iter.get_node_property(prop);
                            arch = val
                        }
                        "os" => {
                            let val = node_iter.get_node_property(prop);
                            os = val
                        }
                        "compression" => {
                            let val = node_iter.get_node_property(prop);
                            compression = val
                        }
                        "load" => {
                            let val = node_iter.get_node_property(prop);
                            load = val
                        }
                        "entry" => {
                            let val = node_iter.get_node_property(prop);
                            entry = val
                        }
                        _ => {}
                    });

                    info!("computing {:?} hash", prop,);
                    let computed_hash;
                    match data {
                        Some(data) => {
                            computed_hash = D::digest(data);
                            info!("computed {:?} hash: {:x}", prop, computed_hash);
                        }
                        None => {
                            panic!("invalid ITB supplied");
                        }
                    }

                    let (_, node_iter) = node_iter.path_struct_items("hash").next().unwrap();
                    let hash_value = node_iter.get_node_property("value");
                    let hash_algo = node_iter.get_node_property("algo");
                    // println!("hash_value: {:x}", hash_value.unwrap());
                    match computed_hash.as_slice().ne(hash_value.unwrap()) {
                        true => panic!("{} intergity check failed...", prop),
                        false => {
                            info!(
                                "\x1b[95m{} integrity consistent\x1b[0m with supplied itb...",
                                prop
                            )
                        }
                    }

                    let hash: Hash<H> = Hash {
                        value: computed_hash.as_slice().try_into().unwrap(),
                        algo: as_str(hash_algo.unwrap())?.expect("hash_algo not specified in itb"),
                    };
                    let os = match os {
                        Some(val) => as_str(val)?,
                        None => None,
                    };
                    let load = match load {
                        Some(val) => Some(u32::from_be_bytes(val.try_into().unwrap())),
                        None => None,
                    };
                    let entry = match entry {
                        Some(val) => Some(u32::from_be_bytes(val.try_into().unwrap())),
                        None => None,
                    };

                    let img = Image {
                        description: as_str(description.unwrap())?
                            .expect("image description not specified in itb"),
                        typ: as_str(typ.unwrap())?.expect("image type not specified in itb"),
                        arch: as_str(arch.unwrap())?.expect("image arch not specified in itb"),
                        os,
                        compression: as_str(compression.unwrap())?
                            .expect("image compression not specified in itb"),
                        load,
                        entry,
                        hash,
                    };
                    images[idx] = img;
                    #[cfg(feature = "defmt")]
                    defmt::info!("Image: {:?}\n", img);
                }
                None => {}
            }
        }
    }
    let images = Images { images };
    Ok((configuration, images))
}

pub fn prepare_img_hash<'a, D, const H: usize, const S: usize, const N: usize>(
    itb_blob: &'a [u8],
) -> Result<(D, [u8; S])>
where
    D: Digest,
{
    let reader = Reader::read(itb_blob).unwrap();
    let root = &reader.struct_items();
    let (_, node_iter) = root.path_struct_items("/").next().unwrap();

    let mut hasher = D::new();
    let timestamp = node_iter.get_node_property("timestamp");
    hasher.update(timestamp.unwrap());

    let (config, images) = parse_fit::<Sha256, H, S, N>(reader)?;
    let cfg_values = [
        config.description,
        config.kernel,
        config.fdt,
        config.ramdisk,
        config.rbconfig,
        config.signature.algo,
        config.signature.key_hint,
        config.signature.signed_images,
    ];
    let mut buf = [0u8; 150];
    let mut offset = 0usize;
    let _ = cfg_values.iter().for_each(|val| {
        buf[offset..offset + val.len()].copy_from_slice(val.as_bytes());
        offset += val.len()
    });
    let cfg_bytes = &buf[..offset];
    hasher.update(cfg_bytes);

    let mut img_hashes = [[0u8; H]; N];
    let _ = for (idx, img) in images.images.iter().enumerate() {
        img_hashes[idx] = img.hash.value;
    };

    // rustBoot FIT images include a time_stamp, configuration details,
    // and 4 (i.e kernel, fdt, ramdisk, config) images i.e. we concatenate 6 hashes in total.
    hasher.update(flatten(img_hashes).as_slice());
    let signature = config.signature.value;

    Ok((hasher, signature))
}

pub fn flatten<'a, const H: usize, const N: usize>(img_hash: [[u8; H]; N]) -> [u8; 32 * 4] {
    // we can replace this when generic parameters in const operations is stabilized
    let mut bytes = [0u8; 32 * 4];
    let _ = img_hash
        .iter()
        .flatten()
        .enumerate()
        .for_each(|(idx, byte)| bytes[idx] = *byte);
    bytes
}

/// Verifies a signed fit-image, given a image tree blob.
///
/// NOTE:
/// - the image tree blob must be a `rustBoot` compliant fit-image.
///
pub fn verify_fit<const H: usize, const S: usize, const N: usize>(
    itb_blob: &[u8],
) -> crate::Result<bool> {
    let algo = parse_algo(itb_blob);
    match algo {
        #[cfg(feature = "secp256k1")]
        Ok(CurveType::Secp256k1) => {}
        #[cfg(feature = "nistp256")]
        Ok(CurveType::NistP256) => {
            let (prehashed_digest, signature) = prepare_img_hash::<Sha256, 32, 64, 4>(itb_blob)
                .map_err(|_v| crate::RustbootError::BadHashValue)?;
            let res = verify_ecc256_signature::<Sha256, HDR_IMG_TYPE_AUTH>(
                prehashed_digest,
                signature.as_ref(),
            );
            res
        }
        _ => todo!(),
    }
}

pub fn parse_algo<'a>(itb_blob: &'a [u8]) -> Result<CurveType> {
    let mut curve_type = CurveType::None;
    let reader = Reader::read(itb_blob).unwrap();
    let root = reader.struct_items();
    let (_, node_iter) = root.path_struct_items("/configurations").next().unwrap();

    if let Some(config) = node_iter.get_node_property("default") {
        // parse the default config's signature algo
        let config = "/configurations/".concat::<50>(config);
        let config = config.as_str()?;
        let sig_node = config.concat::<50>("/signature\0".as_bytes());
        let sig_node = sig_node.as_str()?;

        let (_, node_iter) = root.path_struct_items(sig_node).next().unwrap();
        let algo_val = node_iter.get_node_property("algo");

        match algo_val {
            Some(val) => {
                let algo = as_str(val)?;
                match algo {
                    Some("sha256,ecdsa256,nistp256") => curve_type = CurveType::NistP256,
                    _ => unimplemented!(),
                }
            }
            None => {
                panic!("no signing algorithm specified in supplied itb")
            }
        }
    };
    Ok(curve_type)
}

pub fn get_image_data<'a>(itb_blob: &'a [u8], img: &'a str) -> Option<&'a [u8]> {
    let mut img_path = "";
    match img {
        "kernel" => img_path = "/images/kernel",
        "fdt" => img_path = "/images/fdt",
        "ramdisk" => img_path = "/images/initrd",
        "rbconfig" => img_path = "/images/rbconfig",
        _ => {}
    }
    let reader = Reader::read(itb_blob).unwrap();
    let root = reader.struct_items();
    let (_, node_iter) = root.path_struct_items(img_path).next().unwrap();
    let data = node_iter.get_node_property("data");
    data
}

pub fn as_str(bytes: &[u8]) -> Result<Option<&str>> {
    let val = core::str::from_utf8(bytes)
        .map_err(|val| Error::BadStrEncoding(val))?
        .strip_suffix("\u{0}");
    Ok(val)
}
