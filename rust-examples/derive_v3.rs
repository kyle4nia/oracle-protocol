// Oracle Protocol — parameterized address deriver.
// Prints the v3 P2SH address for ANY rep value passed as an argument.
// Replaces the per-rep derive_v3_100 / _105 / _110 files with one tool.
//
//   cargo run --release --example derive_v3 -- 110
//
// rep is written into the redeem script as 8-byte little-endian (byte index 1..8),
// so values above 255 are handled correctly, not just single-byte reps.

use kaspa_addresses::Prefix;
use kaspa_txscript::{extract_script_pub_key_address, pay_to_script_hash_script};

// The 68-byte redeem TEMPLATE. byte[1..9] is the 8-byte LE rep value; we overwrite it.
const REDEEM_TEMPLATE: [u8; 68] = [
    8, 100, 0, 0, 0, 0, 0, 0, 0, 118, 82, 121, 147, 118, 0, 162, 105, 185, 203, 81,
    156, 105, 118, 88, 205, 1, 8, 124, 126, 185, 118, 201, 118, 1, 68, 148, 89, 147,
    124, 188, 126, 170, 2, 0, 0, 1, 170, 126, 1, 32, 126, 124, 126, 1, 135, 126, 185,
    0, 204, 195, 135, 105, 0, 122, 117, 117, 117, 81,
];

const REP_OFF: usize = 1;
const REP_LEN: usize = 8;

fn redeem_with_rep(rep: u64) -> Vec<u8> {
    let mut s = REDEEM_TEMPLATE.to_vec();
    for i in 0..REP_LEN {
        s[REP_OFF + i] = ((rep >> (8 * i)) & 0xff) as u8;
    }
    s
}

fn main() {
    // Read the rep value from the command line. Fail loud if missing or unparseable.
    let rep: u64 = match std::env::args().nth(1) {
        Some(a) => match a.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("ERROR: rep argument '{a}' is not a valid number.");
                eprintln!("Usage: cargo run --release --example derive_v3 -- <rep>");
                std::process::exit(1);
            }
        },
        None => {
            eprintln!("ERROR: no rep argument given.");
            eprintln!("Usage: cargo run --release --example derive_v3 -- <rep>");
            std::process::exit(1);
        }
    };

    let s = redeem_with_rep(rep);
    let spk = pay_to_script_hash_script(&s);
    let addr = extract_script_pub_key_address(&spk, Prefix::Testnet).expect("addr");
    println!("V3 REP{rep} ADDRESS:");
    println!("{addr}");
}
