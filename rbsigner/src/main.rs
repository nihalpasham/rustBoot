mod curve;
mod fitsigner;
mod mcusigner;

use curve::SigningKeyType;
use curve::{import_signing_key, CurveType};
use fitsigner::sign_fit;
use mcusigner::sign_mcu_image;
use rustBoot::dt::Reader;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: rbsigner <fit-image|mcu-image> <input_path> <curve_type> <key_path> [version]");
        process::exit(1);
    }

    let result = run(&args);
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let mut key_file = Vec::new();
    let mut kf = fs::File::open(&args[4]).map_err(|e| format!("Cannot open key file '{}': {}", args[4], e))?;
    kf.read_to_end(&mut key_file).map_err(|e| format!("Cannot read key file: {}", e))?;

    let sk = match args[3].as_str() {
        "nistp256" => {
            let signing_key = &key_file.as_slice()[0x40..];
            if signing_key.len() != 32 {
                return Err("invalid nistp256 key: length is not 32 bytes".into());
            }
            import_signing_key(CurveType::NistP256, signing_key)
                .map_err(|e| format!("Failed to import signing key: {}", e))?
        }
        other => return Err(format!("Unsupported curve type: {}", other)),
    };

    let mut image_blob = Vec::new();
    match args[1].as_str() {
        "fit-image" => sign_fit_image(args, &mut image_blob, sk),
        "mcu-image" => sign_mcu(args, &mut image_blob, sk),
        other => Err(format!("Unknown image type: {}. Use 'fit-image' or 'mcu-image'.", other)),
    }
}

fn sign_fit_image(args: &[String], image_blob: &mut Vec<u8>, sk: SigningKeyType) -> Result<(), String> {
    let mut itb = fs::File::open(&args[2]).map_err(|e| format!("Cannot open itb '{}': {}", args[2], e))?;
    itb.read_to_end(image_blob).map_err(|e| format!("Cannot read itb: {}", e))?;

    let reader = Reader::read(image_blob.as_slice()).map_err(|e| format!("Cannot parse ITB: {:?}", e))?;
    let root = &reader.struct_items();
    let (_, node_iter) = root.path_struct_items("/").next().ok_or("ITB has no root node")?;

    let timestamp = match node_iter.get_node_property("timestamp") {
        Some(ts) => u32::from_be_bytes(ts.try_into().map_err(|_| "Invalid timestamp in ITB")?),
        None => return Err("ITB does not contain a timestamp field".into()),
    };

    let version_string = timestamp.to_string();
    let output_itb_name = format!("signed-v{version_string}.itb");

    println!("\nImage type:       fit-image");
    println!("Curve type:       {}", args[3]);
    println!("fit version:      {:?}", timestamp);
    println!("Public key:       {}", args[4]);
    println!("Output image:     {}", output_itb_name);

    let signed_fit = sign_fit(image_blob.clone(), timestamp, sk)
        .map_err(|e| format!("Signing failed: {}", e))?;

    let out_dir = match args[2].rsplit_once('/') {
        Some((f, _)) => f,
        None => ".",
    };
    let out_path = format!("{}/{}", out_dir, output_itb_name);
    let mut file = File::create(&out_path)
        .map_err(|e| format!("Cannot create output file '{}': {}", out_path, e))?;
    let written = file.write(signed_fit.as_slice())
        .map_err(|e| format!("Cannot write output: {}", e))?;
    println!("\nbytes_written: {:?}", written);
    Ok(())
}

fn sign_mcu(args: &[String], image_blob: &mut Vec<u8>, sk: SigningKeyType) -> Result<(), String> {
    if args.len() < 6 {
        return Err("MCU image signing requires a version argument (arg 6)".into());
    }

    let image_version_args = &args[5];
    let input_image_args = args[2].rsplit_terminator(&['/', '.'][..])
        .collect::<Vec<_>>().get(1).cloned().unwrap_or("image");
    let output_image = format!("{}_v{}_signed", input_image_args, image_version_args);

    println!("\nImage type:       mcu-image");
    println!("Curve type:       {}", args[3]);
    println!("Input image:      {}.bin", input_image_args);
    println!("Public key:       {}", args[4]);
    println!("Image version:    {}", args[5]);
    println!("Output image:     {}.bin", output_image);

    let image_version_value: u32 = args[5].parse()
        .map_err(|_| format!("Invalid version number '{}'", args[5]))?;
    let version: [u8; 4] = image_version_value.to_le_bytes();

    let mut mcu_image = fs::File::open(&args[2])
        .map_err(|e| format!("Cannot open '{}': {}", args[2], e))?;
    mcu_image.read_to_end(image_blob)
        .map_err(|e| format!("Cannot read image: {}", e))?;

    let signed = sign_mcu_image(std::mem::take(image_blob), &args[2], sk, version)
        .map_err(|e| format!("Signing failed: {}", e))?;

    let out_path = format!("../boards/sign_images/signed_images/{}.bin", output_image);
    let mut file = File::create(&out_path)
        .map_err(|e| format!("Cannot create '{}': {}", out_path, e))?;
    let written = file.write(signed.as_slice())
        .map_err(|e| format!("Cannot write output: {}", e))?;
    println!("Output image successfully created with {} bytes.\n", written);
    Ok(())
}