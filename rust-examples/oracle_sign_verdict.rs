// Oracle Protocol — OFF-CHAIN ORACLE VERDICT SIGNER.
//
// Produces a portable "verdict" signature for the v4 detached-verdict model.
// An oracle signs a verdict authorizing a reputation delta; ANYONE can then
// carry that signature on-chain. The covenant verifies it with
// OpCheckSigFromStack (CSFS, 0xd7) once the compiler emits it (silverscript
// issue #122). The on-chain verification was confirmed from txscript source:
//
//   OpCheckSigFromStack pops [signature, msg_hash, pubkey]; msg_hash must be
//   exactly 32 bytes; check_schnorr_signature_for_msg_hash uses
//   secp256k1::Message::from_digest(msg_hash) DIRECTLY (plain BIP340 schnorr,
//   no domain separation). OpBlake2b = Params::new().hash_length(32), unkeyed.
//
// VERDICT MESSAGE LAYOUT (must match the contract byte-for-byte):
//   verdict_bytes = delta (8B LE) || nonce (8B LE) || covenant_id (32B) = 48B
//   msg_hash      = blake2b_256(verdict_bytes)            (32 bytes)
//   signature     = schnorr_sign(msg_hash) with oracle x-only key  (64 bytes)
//
// This tool SELF-TESTS: after signing it verifies the signature back against
// the same primitives the node uses, proving the signing half is correct
// BEFORE the on-chain gate exists.
//
// Usage:
//   cargo run --release -p rothschild --bin oracle_sign_verdict -- \
//       --delta 5 \
//       --nonce 0 \
//       --covenant-id ba37ac5a713c4f2e7429ba17d4d338d117e1130556e42201181c22552e3102e2 \
//       [--oracle-key <64-hex-of-32-byte-secret>]   (omitted => generate one)

use blake2b_simd::Params;
use secp256k1::{Keypair, Secp256k1, XOnlyPublicKey, schnorr::Signature, Message};

fn arg_val(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    // --- inputs ---
    let delta: i64 = arg_val("--delta").map(|s| s.parse().expect("bad --delta")).unwrap_or(5);
    let nonce: i64 = arg_val("--nonce").map(|s| s.parse().expect("bad --nonce")).unwrap_or(0);
    let covenant_id_hex = arg_val("--covenant-id")
        .expect("--covenant-id <64 hex chars> is required");

    let mut covenant_id = [0u8; 32];
    faster_hex::hex_decode(covenant_id_hex.as_bytes(), &mut covenant_id)
        .expect("--covenant-id must be 64 hex chars (32 bytes)");

    let secp = Secp256k1::new();

    // --- oracle key: provided or generated ---
    let keypair = match arg_val("--oracle-key") {
        Some(hex) => {
            let mut sk = [0u8; 32];
            faster_hex::hex_decode(hex.as_bytes(), &mut sk).expect("--oracle-key must be 64 hex chars");
            Keypair::from_seckey_slice(&secp, &sk).expect("bad oracle key")
        }
        None => {
            // Deterministic-ish dev key from system randomness.
            let (sk, _pk) = secp.generate_keypair(&mut secp256k1::rand::thread_rng());
            Keypair::from_secret_key(&secp, &sk)
        }
    };
    let (xonly, _parity): (XOnlyPublicKey, _) = keypair.x_only_public_key();
    let oracle_pk = xonly.serialize();           // 32-byte x-only pubkey
    let oracle_pkh = Params::new().hash_length(32).to_state().update(&oracle_pk).finalize();

    // --- build verdict_bytes = delta(8 LE) || nonce(8 LE) || covenant_id(32) ---
    let mut verdict = Vec::with_capacity(48);
    verdict.extend_from_slice(&delta.to_le_bytes());
    verdict.extend_from_slice(&nonce.to_le_bytes());
    verdict.extend_from_slice(&covenant_id);
    assert_eq!(verdict.len(), 48, "verdict_bytes must be 48 bytes");

    // --- msg_hash = blake2b_256(verdict_bytes) ---
    let msg_hash = Params::new().hash_length(32).to_state().update(&verdict).finalize();
    let msg_hash_bytes: [u8; 32] = msg_hash.as_bytes().try_into().expect("32-byte hash");

    // --- schnorr-sign the 32-byte hash directly (matches node's from_digest) ---
    let message = Message::from_digest(msg_hash_bytes);
    let signature: Signature = secp.sign_schnorr_no_aux_rand(&message, &keypair);
    let sig_bytes = signature.serialize();       // 64 bytes

    // --- SELF-TEST: verify the signature the same way the node will ---
    let verify_ok = secp.verify_schnorr(&signature, &message, &xonly).is_ok();

    // --- output ---
    println!("=== Oracle Verdict Signature ===");
    println!("delta            : {delta}");
    println!("nonce            : {nonce}");
    println!("covenant_id      : {covenant_id_hex}");
    println!();
    println!("verdict_bytes(48): {}", hex(&verdict));
    println!("msg_hash(32)     : {}", hex(&msg_hash_bytes));
    println!();
    println!("oracle_pubkey(32): {}", hex(&oracle_pk));
    println!("oracle_pkh(32)   : {}   <- bake into contract ctor as oracle_pkh", hex(oracle_pkh.as_bytes()));
    println!("oracle_secret(32): {}   <- KEEP SECRET (dev/TN10 key)", hex(&keypair.secret_bytes()));
    println!();
    println!("signature(64)    : {}", hex(&sig_bytes));
    println!();
    println!("SELF-TEST verify : {}", if verify_ok { "PASS (sig valid against node's schnorr verify)" } else { "FAIL" });
    println!();
    println!("For the spender sigScript, push: signature(64), msg_hash(32), oracle_pubkey(32)");
    println!("(exact order the contract's OpCheckSigFromStack expects: [signature, msg_hash, pubkey])");

    // --- NEGATIVE SELF-TEST: a tampered delta must NOT verify under the same sig ---
    let mut tampered = verdict.clone();
    tampered[0] = tampered[0].wrapping_add(1); // change delta's low byte
    let tampered_hash = Params::new().hash_length(32).to_state().update(&tampered).finalize();
    let tampered_msg = Message::from_digest(tampered_hash.as_bytes().try_into().unwrap());
    let tampered_ok = secp.verify_schnorr(&signature, &tampered_msg, &xonly).is_ok();
    println!("NEG  TEST tamper : {}", if !tampered_ok { "PASS (tampered verdict rejected)" } else { "FAIL — tamper accepted!" });
}

fn hex(bytes: &[u8]) -> String {
    let mut out = vec![0u8; bytes.len() * 2];
    faster_hex::hex_encode(bytes, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}
