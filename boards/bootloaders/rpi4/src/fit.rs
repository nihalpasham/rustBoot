use rustBoot::dt::{get_image_data, verify_fit, Reader, Result};
use rustBoot::fs::{
    blockdevice::BlockDevice,
    controller::{Controller, Volume},
    filesystem::{Mode, TimeSource},
};

use rustBoot_hal::{info, print};

use crate::boot::{DTB_LOAD_ADDR, INITRAMFS_LOAD_ADDR, ITB_LOAD_ADDR, KERNEL_LOAD_ADDR};
use crate::dtb::patch_dtb;

pub fn load_fit<'a, D, T>(volume: &mut Volume, ctrlr: &mut Controller<D, T>) -> &'a [u8]
where
    D: BlockDevice,
    T: TimeSource,
{
    let mut num_read = 0;
    let root_dir = ctrlr.open_root_dir(&volume).unwrap();
    info!("Listing \x1b[33mroot\x1b[0m directory:");
    ctrlr
        .iterate_dir(&volume, &root_dir, |entry| {
            if entry.size > 60000000 {
                info!("     - \x1b[36mFound: {}\x1b[0m", entry.name)
            };
        })
        .unwrap();

    // Load itb
    info!("\x1b[5m\x1b[34mloading fit-image...\x1b[0m");
    let mut itb_file = ctrlr
        .open_file_in_dir(volume, &root_dir, "SIGNED~1.ITB", Mode::ReadOnly)
        .unwrap();
    while !itb_file.eof() {
        num_read = ctrlr
            .read_multi(&volume, &mut itb_file, unsafe { &mut ITB_LOAD_ADDR.0 })
            .unwrap();
        info!(
            "loaded fit: {:?} bytes, starting at addr: {:p}",
            num_read,
            unsafe { &mut ITB_LOAD_ADDR.0 },
        );
    }
    ctrlr.close_file(&volume, itb_file).unwrap();
    ctrlr.close_dir(&volume, root_dir);

    unsafe { &ITB_LOAD_ADDR.0.as_ref()[..num_read] }
}

pub fn verify_authenticity() -> bool {
    info!("\x1b[5m\x1b[31mauthenticating fit-image...\x1b[0m");
    let header = Reader::get_header(unsafe { &ITB_LOAD_ADDR.0 }).unwrap();
    let total_size = header.total_size;
    let val = match verify_fit::<32, 64, 4>(unsafe { &ITB_LOAD_ADDR.0[..total_size as usize] }) {
        Ok(val) => {
            print!(
                "######## \x1b[33mecdsa signature\x1b[0m checks out, \
                \x1b[92mimage is authentic\x1b[0m ########\n"
            );
            val
        }
        Err(e) => panic!("error: image verification failed, {}", e),
    };
    val
}

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
