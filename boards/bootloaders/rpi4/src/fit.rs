use rustBoot::dt::{
    get_image_data, verify_fit, Concat, Reader, Result, FALLBACK_TO_ACTIVE_IMG, IS_PASSIVE_SELECTED,
};
use rustBoot::fs::{
    blockdevice::BlockDevice,
    controller::{Controller, Volume, VolumeType},
    filesystem::{LongFileName, Mode, TimeSource},
};

use rustBoot::{
    cfgparser::{self, UpdateStatus},
    Result as RbResult, RustbootError,
};
use rustBoot_hal::{info, print};

use crate::boot::{DTB_LOAD_ADDR, INITRAMFS_LOAD_ADDR, ITB_LOAD_ADDR, KERNEL_LOAD_ADDR};
use crate::dtb::patch_dtb;

/// Loads a fit-image. Returns a tuple contianing the image-tree blob and its version number
///
/// **note:** this function expects a valid `updt.txt` file to be present in the FAT partition's root directory.
/// If it doesnt find one or if it isn't a valid `updt.txt` config, it will panic.
pub fn load_fit<'a, D, T>(volume: &mut Volume, ctrlr: &mut Controller<D, T>) -> (&'a [u8], u32)
where
    D: BlockDevice,
    T: TimeSource,
{
    let mut fit_to_load = None;
    let mut version_to_load = None;
    let updt_flag;
    let mut updt_triggered = false;

    let active_img_name;
    let passive_img_name;

    let root_dir = ctrlr.open_root_dir(&volume).unwrap();

    // Load update config
    let mut num_read = 0;
    let mut cfg = [0u8; 200];
    let mut updt_cfg = ctrlr
        .open_file_in_dir(volume, &root_dir, "UPDT.TXT", Mode::ReadOnly)
        .unwrap();
    while !updt_cfg.eof() {
        num_read = ctrlr.read(&volume, &mut updt_cfg, &mut cfg).unwrap();
    }
    info!(
        "loaded `updt.txt` cfg: {:?} bytes, starting at addr: {:p}",
        num_read, &cfg,
    );
    ctrlr.close_file(&volume, updt_cfg).unwrap();

    // parse `updt.txt` cfg
    if let Ok((_, (active_conf, passive_conf))) = cfgparser::parse_config(
        core::str::from_utf8(&cfg).expect("an invalid update cfg was provided"),
    ) {
        // get active config name and version
        let active_name = active_conf.image_name;
        let active_version = active_conf.image_version;
        // get passive config name, version and status
        let passive_name = passive_conf.image_name;
        let passive_version = passive_conf.image_version;
        let passive_status = passive_conf.update_status;

        // check whether the `update` has been marked as ready (on the next reboot).
        updt_flag = match passive_conf.ready_for_update_flag {
            true => match (passive_name, passive_version, passive_status) {
                (None, _, _) => false,
                (_, None, _) => false,
                (_, _, None) => false,
                (
                    Some((_, ".itb")),
                    _,
                    Some(UpdateStatus::Updating) | Some(UpdateStatus::Success),
                ) => true,
                (Some((_, _)), _, Some(UpdateStatus::Testing)) => {
                    info!("update was authenticated and run but was not marked as successful, falling back to currently active image");
                    false
                }
                (Some((_, _)), _, _) => false,
            },
            false => false,
        };
        // Check the update version. A valid update must have a version
        // greater than the active version.
        let version_check = match passive_version {
            Some(ver) => ver > active_version,
            None => false,
        };
        // `&str` concatentation - image name + extension
        // name + extn must be less than 50 bytes.
        active_img_name = active_name.0.concat::<50>(active_name.1.as_bytes());
        passive_img_name = if let Some(val) = passive_name {
            val.0.concat::<50>(val.1.as_bytes())
        } else {
            active_img_name
        };
        match updt_flag && version_check && unsafe { FALLBACK_TO_ACTIVE_IMG.get().is_none() } {
            true => {
                // ok to unwrap, we already checked.
                version_to_load = passive_version;
                let _ = unsafe { IS_PASSIVE_SELECTED.get_or_init(|| true) };
                fit_to_load = passive_img_name.as_str_no_suffix().ok();
                updt_triggered = true;
            }
            false => {
                version_to_load = Some(active_version);
                fit_to_load = active_img_name.as_str_no_suffix().ok();
                updt_triggered = false;
            }
        }
    };
    info!(
        "fit_to_load: {}, version_to_load: {}",
        fit_to_load.unwrap(),
        version_to_load.unwrap()
    );

    let mut num_read = 0;
    info!("Listing \x1b[33mroot\x1b[0m directory:");
    ctrlr
        .iterate_dir(&volume, &root_dir, |entry| {
            if entry.size > 60000000 {
                info!("     - \x1b[36mFound: {}\x1b[0m", entry.name)
            };
        })
        .unwrap();

    if updt_triggered {
        info!("update triggered...");
    } else {
        info!("booting active image...")
    }
    // Load itb
    match (fit_to_load, version_to_load) {
        (Some(fit_name), Some(fit_version)) => {
            let lfn = LongFileName::create_from_str(fit_name);
            let sfn_bytes = match &volume.volume_type {
                VolumeType::Fat(fat) => {
                    match fat.get_sfn_bytes_from_lfn_name(ctrlr, &lfn, &root_dir) {
                        Ok(val) => to_dotted_sfn(val),
                        Err(e) => panic!("error: {:?}", e),
                    }
                }
            };
            let sfn = core::str::from_utf8(&sfn_bytes).unwrap();
            // info!("\x1b[5m\x1b[34msfn bytes: {:?} \x1b[0m", &sfn_bytes);
            info!("\x1b[5m\x1b[34mloading fit-image...{} \x1b[0m", sfn);

            let mut itb_file = ctrlr
                .open_file_in_dir(volume, &root_dir, sfn, Mode::ReadOnly)
                .unwrap();
            while !itb_file.eof() {
                num_read = ctrlr
                    .read_multi(&volume, &mut itb_file, unsafe { &mut ITB_LOAD_ADDR.0 })
                    .unwrap();
                info!(
                    "loaded {}: {:?} bytes, version: {:?}, starting at addr: {:p}",
                    fit_name,
                    num_read,
                    fit_version,
                    unsafe { &mut ITB_LOAD_ADDR.0 },
                );
            }

            ctrlr.close_file(&volume, itb_file).unwrap();
            ctrlr.close_dir(&volume, root_dir);

            (
                unsafe { &ITB_LOAD_ADDR.0.as_ref()[..num_read] },
                fit_version,
            )
        }
        (_, _) => {
            // this shouldnt be possible if `parse_config` succeeds
            unreachable!()
        }
    }
}

/// Verifies a loaded fit-image's cryptographic digital signature, when supplied with a `fit version number`.
///
/// The fit's version number is retrieved from rustBoot's `updt.txt` file i.e. this function also checks
/// whether the `version-number` from `updt.txt` matches the fit-image's timestamp.
///
/// **note:** rustBoot uses a global mutable static to load its fit-images.
pub fn verify_authenticity(itb_version: u32) -> RbResult<bool> {
    info!("\x1b[5m\x1b[31mauthenticating fit-image...\x1b[0m");
    let header = Reader::get_header(unsafe { &ITB_LOAD_ADDR.0 }).unwrap();
    let total_size = header.total_size;
    let val = match verify_fit::<32, 64, 4>(
        unsafe { &ITB_LOAD_ADDR.0[..total_size as usize] },
        itb_version,
    ) {
        Ok(val) => {
            print!(
                "######## \x1b[33mecdsa signature\x1b[0m checks out, \
                \x1b[92mimage is authentic\x1b[0m ########\n"
            );
            Ok(val)
        }
        Err(e) => {
            match e {
                RustbootError::BadVersion => return Err(e),
                _ => return Err(RustbootError::FwAuthFailed),
            };
        }
    };
    val
}

/// Extracts and relocates the kernel image from a loaded fit-image to a
/// (statically determined) location in bss.
pub fn relocate_kernel(itb_blob: &[u8]) {
    let kernel_entry = unsafe { KERNEL_LOAD_ADDR.0.as_mut() };
    let kernel_data = get_image_data(itb_blob, "kernel");
    match kernel_data {
        Some(val) => {
            let len = val.len();
            assert!(len < unsafe { KERNEL_LOAD_ADDR.0.len() });
            kernel_entry[..len].copy_from_slice(val);
        }
        None => {
            panic!("itb has no kernel data")
        }
    }
}
#[allow(dead_code)]
/// Extracts and relocates the flattened device tree from a loaded fit-image to a
/// (statically determined) location in bss.
pub fn relocate_fdt(itb_blob: &[u8]) {
    let fdt_entry = unsafe { DTB_LOAD_ADDR.0.as_mut() };
    let fdt_data = get_image_data(itb_blob, "fdt");
    match fdt_data {
        Some(val) => {
            let len = val.len();
            assert!(len < unsafe { DTB_LOAD_ADDR.0.len() });
            fdt_entry[..len].copy_from_slice(val);
        }
        None => {
            panic!("itb has no fdt data")
        }
    }
}
/// Extracts and relocates the ramdisk/initrd from a loaded fit-image to a
/// (statically determined) location in bss.
pub fn relocate_ramdisk(itb_blob: &[u8]) {
    let initrd_entry = unsafe { INITRAMFS_LOAD_ADDR.0.as_mut() };
    let initrd_data = get_image_data(itb_blob, "ramdisk");
    match initrd_data {
        Some(val) => {
            let len = val.len();
            assert!(len < unsafe { INITRAMFS_LOAD_ADDR.0.len() });
            initrd_entry[..len].copy_from_slice(val);
        }
        None => {
            panic!("itb has no ramdisk data")
        }
    }
}

/// Relocates the kernel and ramdisk from a loaded fit-image to a
/// (statically determined) location in bss and extracts the device-tree blob from the fit-image, patches
/// it with contents of `rbconfig.txt` (i.e. linux cmdline parameters) and finally relocates it to a
/// (statically determined) location in bss.
///
/// **note:** This function can fail if `patching` fails.
///
pub fn relocate_and_patch<'a>(itb_blob: &'a [u8]) -> Result<&'a [u8]> {
    let _ = relocate_kernel(itb_blob);
    info!("relocating kernel to addr: {:p}", unsafe {
        &KERNEL_LOAD_ADDR.0
    });
    let _ = relocate_ramdisk(itb_blob);
    info!("relocating initrd to addr: {:p}", unsafe {
        &INITRAMFS_LOAD_ADDR.0
    });
    let res = patch_dtb(itb_blob);
    match res {
        Ok((buf, len)) => {
            info!("relocating dtb to addr: {:p}\n", buf.as_slice());
            Ok(&buf[..len])
        }
        Err(e) => return Err(e),
    }
}

/// Short file names do not include the `.`, separating the
/// filename and file-extension.
///
/// This is a helper function to add `.` separator.
pub fn to_dotted_sfn(bytes: [u8; 11]) -> [u8; 12] {
    // we dont need to check if the file extn is `itb`
    // as we've already taken care of that.
    // we just need to add the `.`
    let mut dotted_sfn = [0u8; 12];
    bytes.iter().enumerate().for_each(|(idx, byte)| {
        if idx == 8 {
            dotted_sfn[idx] = 0x2e; // dot separator
            dotted_sfn[idx + 1] = *byte
        } else if idx > 8 {
            dotted_sfn[idx + 1] = *byte
        } else {
            dotted_sfn[idx] = *byte
        }
    });
    dotted_sfn
}
