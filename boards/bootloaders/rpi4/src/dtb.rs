use rustBoot::dt::{get_image_data, patch_chosen_node, Error, PropertyValue, Reader, Result};

use rustBoot_hal::info;

use crate::boot::{DTB_LOAD_ADDR, INITRAMFS_LOAD_ADDR, MAX_DTB_SIZE};

pub fn patch_dtb<'a>(itb_blob: &'a [u8]) -> Result<(&'a mut [u8; MAX_DTB_SIZE], usize)> {
    // Load rbconfig
    info!("load rbconfig...");
    let rbconfig = get_image_data(itb_blob, "rbconfig").unwrap();

    let propval_list = get_propval_list(itb_blob, rbconfig)?;

    let dtb_blob = get_image_data(itb_blob, "fdt").unwrap();
    let reader = Reader::read(dtb_blob)?;
    info!("\x1b[5m\x1b[34mpatching dtb...\x1b[0m");
    let res = patch_chosen_node(reader, dtb_blob, &propval_list, unsafe {
        &mut DTB_LOAD_ADDR.0
    });
    Ok(res)
}

pub fn get_propval_list<'a>(
    itb_blob: &'a [u8],
    cmd_line: &'a [u8],
) -> Result<[PropertyValue<'a>; 3]> {
    let cmd_line = core::str::from_utf8(cmd_line)
        .map_err(|val| Error::BadStrEncoding(val))?
        .strip_suffix("\"")
        .unwrap();
    let cmd_line = cmd_line.strip_prefix("bootargs=\"");
    // info!("cmd_line: {}", cmd_line.unwrap());
    let initrd_start = unsafe { &INITRAMFS_LOAD_ADDR.0 as *const u8 as u32 };
    let initrd_len = get_image_data(itb_blob, "ramdisk").unwrap().len();
    let initrd_end = initrd_start + initrd_len as u32;
    // info!("initrd_start: {:?}", initrd_start.to_be_bytes());
    // info!("initrd_end: {:?}", initrd_end.to_be_bytes());

    Ok([
        PropertyValue::String(cmd_line.unwrap()),
        PropertyValue::U32(initrd_start.to_be_bytes()),
        PropertyValue::U32(initrd_end.to_be_bytes()),
    ])
}

// #[allow(dead_code)]
// pub fn patch_dtb_1<'a, const N: usize, D, T>(
//     itb_blob: &'a [u8],
//     volume: &mut Volume,
//     ctrlr: &mut Controller<D, T>,
// ) -> Result<(&'a mut [u8; MAX_DTB_SIZE], usize)>
// where
//     D: BlockDevice,
//     T: TimeSource,
// {
//     let mut num_read = 0;
//     let mut rbconfig = [0; N];
//     let root_dir = ctrlr.open_root_dir(&volume).unwrap();

//     // Load rbconfig
//     info!("\x1b[5m\x1b[34mloading rbconfig...\x1b[0m");
//     let mut rbconfig_file = ctrlr
//         .open_file_in_dir(volume, &root_dir, "RBCONFIG.TXT", Mode::ReadOnly)
//         .unwrap();
//     while !rbconfig_file.eof() {
//         num_read = ctrlr
//             .read_multi(&volume, &mut rbconfig_file, rbconfig.as_mut())
//             .unwrap();
//         info!(
//             "loaded rbconfig: {:?} bytes",
//             num_read,
//         );
//     }
//     ctrlr.close_file(&volume, rbconfig_file).unwrap();
//     ctrlr.close_dir(&volume, root_dir);

//     let propval_list = get_propval_list(itb_blob, &rbconfig.as_ref()[..num_read])?;

//     let dtb_blob = get_image_data(itb_blob, "fdt").unwrap();
//     let reader = Reader::read(dtb_blob)?;
//     let res = patch_chosen_node(reader, dtb_blob, &propval_list, unsafe {
//         &mut DTB_LOAD_ADDR.0
//     });
//     Ok(res)
// }
