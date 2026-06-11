//! CLI wrapper around the oracle_signer library.
//!
//! Usage:
//!   cargo run --bin oracle_sign_verdict -- \
//!       --delta 5 --nonce 0 \
//!       --covenant-id ba37ac5a713c4f2e7429ba17d4d338d117e1130556e42201181c22552e3102e2 \
//!       [--oracle-key <64-hex-of-32-byte-secret>]   (omitted => generate one)

use oracle_signer::*;
use secp256k1::{Keypair, Secp256k1};

fn arg_val(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let delta: i64 = arg_val("--delta").map(|s| s.parse().expect("bad --delta")).unwrap_or(5);
    let nonce: i64 = arg_val("--nonce").map(|s| s.parse().expect("bad --nonce")).unwrap_or(0);
    let covenant_id_hex =
        arg_val("--covenant-id").expect("--covenant-id <64 hex chars> is required");
    let covenant_id = covenant_id_from_hex(&covenant_id_hex).expect("bad --covenant-id");

    let secp = Secp256k1::new();
    let keypair = match arg_val("--oracle-key") {
        Some(hex_key) => {
            let mut sk = [0u8; 32];
            faster_hex::hex_decode(hex_key.as_bytes(), &mut sk).expect("--oracle-key must be 64 hex chars");
            Keypair::from_seckey_slice(&secp, &sk).expect("bad oracle key")
        }
        None => {
            let (sk, _pk) = secp.generate_keypair(&mut rand::thread_rng());
            Keypair::from_secret_key(&secp, &sk)
        }
    };

    let v = sign_verdict(&keypair, delta, nonce, &covenant_id);
    let verify_ok = verify_verdict(&v.msg_hash, &v.oracle_pubkey, &v.signature);

    println!("=== Oracle Verdict Signature ===");
    println!("delta            : {delta}");
    println!("nonce            : {nonce}");
    println!("covenant_id      : {covenant_id_hex}");
    println!();
    println!("verdict_bytes(48): {}", hex(&v.verdict_bytes));
    println!("msg_hash(32)     : {}", hex(&v.msg_hash));
    println!();
    println!("oracle_pubkey(32): {}", hex(&v.oracle_pubkey));
    println!("oracle_pkh(32)   : {}   <- bake into contract ctor as oracle_pkh", hex(&v.oracle_pkh));
    println!("oracle_secret(32): {}   <- KEEP SECRET (dev/TN10 key)", hex(&keypair.secret_bytes()));
    println!();
    println!("signature(64)    : {}", hex(&v.signature));
    println!();
    println!("SELF-TEST verify : {}", if verify_ok { "PASS" } else { "FAIL" });
    println!();
    println!("For the spender sigScript, push: signature(64), msg_hash(32), oracle_pubkey(32)");
}
