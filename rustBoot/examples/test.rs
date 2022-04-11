use std::fs;
use std::io::Read;
use std::str::Utf8Error;

fn main() {
    let mut buf = Vec::new();
    let mut file = fs::File::open(
        std::env::args()
            .nth(1)
            .expect("Need path to rbconfig file as argument"),
    )
    .unwrap();
    file.read_to_end(&mut buf).unwrap();

    let _ = get_propval_list(buf.as_slice());
}

pub fn get_propval_list<'a>(cmd_line: &'a [u8]) -> Result<(), Utf8Error> {
    println!("cmd_line: {:?}", cmd_line);
    let cmd_line = as_str(cmd_line)?.unwrap();
    let cmd_line = cmd_line.strip_prefix("bootargs=\"");
    println!("cmd_line: {}", cmd_line.unwrap());
    Ok(())
}

pub fn as_str(bytes: &[u8]) -> Result<Option<&str>, Utf8Error> {
    let val = core::str::from_utf8(bytes)?.strip_suffix("\"");
    Ok(val)
}
