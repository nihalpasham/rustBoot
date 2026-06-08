// Stack depth analysis entry point.
// Calls all major rustBoot functions to produce full call graph.
// Build with: cargo +nightly call-stack --bin stack-analysis ...

use rustBoot::cfgparser::*;
use rustBoot::crypto::signatures::*;

fn main() {
    // 1. ECDSA public key import
    let _ = import_pubkey(PubkeyTypes::Secp256k1);

    // 2. Update config parsing
    let cfg = "active_bank=0,active_offset=0,active_size=65536";
    let _ = parse_config(cfg);
}
