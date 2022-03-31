use super::{Concat, Error, Reader, Result};
use core::convert::TryInto;
use sha2::{Digest, Sha256};

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

pub fn parse_fit<const H: usize, const S: usize, const N: usize>(
    reader: Reader,
) -> Result<(Config<S>, Images<H, N>)> {
    let mut configuration = Config::default();
    let mut images = [Image::default(); N];
    let root = reader.struct_items();
    let (_, node_iter) = root.path_struct_items("/configurations").next().unwrap();

    // *** Find the default config ***

    if let Some(config) = node_iter.get_node_property("default") {
        // parse the default config
        let config = "/configurations/".serialize_and_concat(config);
        let config = core::str::from_utf8(config.as_slice())
            .map_err(|val| Error::BadStrEncoding(val))?
            .strip_suffix("\u{0}"); // strip null byte
        #[cfg(feature = "defmt")]
        defmt::info!("config: {:?}", config);

        let (_, node_iter) = root.path_struct_items(config.unwrap()).next().unwrap();

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
            algo: core::str::from_utf8(signature_algo.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
            key_hint: core::str::from_utf8(key_hint.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
            signed_images: core::str::from_utf8(signed_images.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
        };
        let config = Config {
            description: core::str::from_utf8(description.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
            kernel: core::str::from_utf8(kernel.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
            fdt: core::str::from_utf8(fdt.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
            ramdisk: core::str::from_utf8(ramdisk.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
            rbconfig: core::str::from_utf8(rbconfig.unwrap())
                .map_err(|val| Error::BadStrEncoding(val))?
                .strip_suffix("\u{0}")
                .unwrap(),
            signature,
        };
        configuration = config;
        #[cfg(feature = "defmt")]
        defmt::info!("Config: {:?}\n", config);

        let conf_properties = ["kernel", "fdt", "ramdisk", "rbconfig"];
        for (idx, prop) in conf_properties.iter().enumerate() {
            match node_iter.get_node_property(prop) {
                Some(val) => {
                    let img = "/images/".serialize_and_concat(val);
                    let img = core::str::from_utf8(img.as_slice())
                        .unwrap()
                        .strip_suffix("\u{0}"); // strip null byte
                    #[cfg(feature = "defmt")]
                    defmt::info!("img: {:?}", img);

                    let (_, node_iter) = root.path_struct_items(img.unwrap()).next().unwrap();
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

                    // let img_data = node_iter.get_node_property("data");
                    let computed_hash = Sha256::digest(data.unwrap());
                    // println!("hash: {:x}", computed_hash);

                    let (_, node_iter) = node_iter.path_struct_items("hash").next().unwrap();
                    let hash_value = node_iter.get_node_property("value");
                    let hash_algo = node_iter.get_node_property("algo");
                    // println!("hash_value: {:x}", hash_value.unwrap());
                    match computed_hash.as_slice().ne(hash_value.unwrap()) {
                        true => panic!("{} intergity check failed...", prop),
                        false => {
                            #[cfg(feature = "defmt")]
                            defmt::info!("{} integrity check passed...", prop)
                        }
                    }

                    let hash: Hash<H> = Hash {
                        value: computed_hash.as_slice().try_into().unwrap(),
                        algo: core::str::from_utf8(hash_algo.unwrap())
                            .map_err(|val| Error::BadStrEncoding(val))?
                            .strip_suffix("\u{0}")
                            .unwrap(),
                    };
                    let os = match os {
                        Some(val) => core::str::from_utf8(val)
                            .map_err(|val| Error::BadStrEncoding(val))?
                            .strip_suffix("\u{0}"),
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
                        description: core::str::from_utf8(description.unwrap())
                            .map_err(|val| Error::BadStrEncoding(val))?
                            .strip_suffix("\u{0}")
                            .unwrap(),
                        typ: core::str::from_utf8(typ.unwrap())
                            .map_err(|val| Error::BadStrEncoding(val))?
                            .strip_suffix("\u{0}")
                            .unwrap(),
                        arch: core::str::from_utf8(arch.unwrap())
                            .map_err(|val| Error::BadStrEncoding(val))?
                            .strip_suffix("\u{0}")
                            .unwrap(),
                        os,
                        compression: core::str::from_utf8(compression.unwrap())
                            .map_err(|val| Error::BadStrEncoding(val))?
                            .strip_suffix("\u{0}")
                            .unwrap(),
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
) -> Result<D>
where
    D: Digest,
{
    let reader = Reader::read(itb_blob).unwrap();
    let root = &reader.struct_items();
    let (_, node_iter) = root.path_struct_items("/").next().unwrap();
    let timestamp = node_iter.get_node_property("timestamp");
    let timestamp_hash = Sha256::digest(timestamp.unwrap());

    let (config, images) = parse_fit::<H, S, N>(reader)?;
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
    let cfg_hash = Sha256::digest(cfg_bytes);

    let mut img_hashes = [[0u8; H]; N];
    let _ = for (idx, img) in images.images.iter().enumerate() {
        img_hashes[idx] = img.hash.value;
    };

    // rustBoot FIT imagess include the time_stamp, the entire configuration,
    // and 4 (i.e kernel, fdt, ramdisk, config) image hashes, we concatenate 6 hashes in total.
    let mut hash_buffer = [0u8; 32 * 6];
    let _ = timestamp_hash
        .as_slice()
        .iter()
        .chain(cfg_hash.as_slice())
        .chain(flatten(img_hashes).as_slice())
        .enumerate()
        .for_each(|(idx, byte)| hash_buffer[idx] = *byte);
    let mut hasher = D::new();
    hasher.update(hash_buffer.as_slice());
    Ok(hasher)
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
