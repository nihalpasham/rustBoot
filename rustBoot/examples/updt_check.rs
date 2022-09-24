use rustBoot::cfgparser::{self, UpdateStatus};
use rustBoot::dt::Concat;

use std::env;
use std::fs;
use std::io::Read;

fn main() {
    let mut fit_to_load = None;
    let mut version_to_load = None;
    let updt_flag;

    let active_img_name;
    let passive_img_name;

    // Load update config
    let num_read;
    let mut cfg = Vec::new();
    println!("\x1b[5m\x1b[34mloading update config...\x1b[0m");
    let args = env::args().collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    let mut file = fs::File::open(args[1]).expect("Need path to updt.txt file as argument");
    num_read = file.read_to_end(&mut cfg).unwrap();

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
                    println!("staged update did not mark update as successful, falling back to currently active image");
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
        match updt_flag && version_check {
            true => {
                // ok to unwrap, we already checked.
                version_to_load = passive_version;
                fit_to_load = passive_img_name.as_str_no_suffix().ok()
            }
            false => {
                version_to_load = Some(active_version);
                fit_to_load = active_img_name.as_str_no_suffix().ok()
            }
        }
    };

    println!(
        "fit_to_load: {:?},\nversion_to_load: {:?},\nnum_read: {:?}\n",
        fit_to_load, version_to_load, num_read
    );
}
