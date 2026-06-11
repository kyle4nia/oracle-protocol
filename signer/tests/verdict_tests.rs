//! Tests for the oracle verdict signer.
//!
//! These run under `cargo test` and are what CI executes. The critical pair is
//! "valid signature accepts" AND "tampered verdict rejects" — a gate tested only
//! on valid input is half-proven.

use oracle_signer::*;
use secp256k1::{Keypair, Secp256k1};

// A fixed test key (32 bytes) so tests are deterministic. NOT a real key.
const TEST_SECRET: [u8; 32] = [
    0x79, 0x5f, 0x83, 0x47, 0x9a, 0xe4, 0x00, 0xa2, 0x08, 0xcd, 0xf1, 0xad, 0xfe, 0xe6, 0x7d, 0xf6,
    0x5d, 0x35, 0xf0, 0x71, 0xc0, 0xf1, 0xf4, 0x2a, 0xdf, 0xcb, 0xff, 0xfd, 0x7f, 0xe1, 0x01, 0xe3,
];

const TEST_COVENANT_ID: [u8; 32] = [
    0xba, 0x37, 0xac, 0x5a, 0x71, 0x3c, 0x4f, 0x2e, 0x74, 0x29, 0xba, 0x17, 0xd4, 0xd3, 0x38, 0xd1,
    0x17, 0xe1, 0x13, 0x05, 0x56, 0xe4, 0x22, 0x01, 0x18, 0x1c, 0x22, 0x55, 0x2e, 0x31, 0x02, 0xe2,
];

fn test_keypair() -> Keypair {
    let secp = Secp256k1::new();
    Keypair::from_seckey_slice(&secp, &TEST_SECRET).expect("valid test key")
}

#[test]
fn verdict_bytes_layout_is_48_and_little_endian() {
    let v = build_verdict_bytes(5, 0, &TEST_COVENANT_ID);
    assert_eq!(v.len(), 48);
    // delta = 5, little-endian => first byte 0x05, rest of the 8 zero.
    assert_eq!(&v[0..8], &[5, 0, 0, 0, 0, 0, 0, 0]);
    // nonce = 0
    assert_eq!(&v[8..16], &[0, 0, 0, 0, 0, 0, 0, 0]);
    // covenant_id verbatim
    assert_eq!(&v[16..48], &TEST_COVENANT_ID);
}

#[test]
fn blake2b_256_outputs_32_bytes() {
    let h = blake2b_256(b"hello");
    assert_eq!(h.len(), 32);
}

#[test]
fn valid_signature_verifies() {
    let kp = test_keypair();
    let v = sign_verdict(&kp, 5, 0, &TEST_COVENANT_ID);
    assert!(
        verify_verdict(&v.msg_hash, &v.oracle_pubkey, &v.signature),
        "a freshly signed verdict must verify"
    );
}

#[test]
fn tampered_delta_is_rejected() {
    // Sign a verdict for delta=5, then verify the SAME signature against the
    // hash for delta=6. It must fail — proving the signature binds the delta.
    let kp = test_keypair();
    let v = sign_verdict(&kp, 5, 0, &TEST_COVENANT_ID);

    let tampered_hash = verdict_msg_hash(6, 0, &TEST_COVENANT_ID);
    assert!(
        !verify_verdict(&tampered_hash, &v.oracle_pubkey, &v.signature),
        "a signature for delta=5 must NOT verify against delta=6"
    );
}

#[test]
fn tampered_nonce_is_rejected() {
    let kp = test_keypair();
    let v = sign_verdict(&kp, 5, 0, &TEST_COVENANT_ID);

    let tampered_hash = verdict_msg_hash(5, 1, &TEST_COVENANT_ID);
    assert!(
        !verify_verdict(&tampered_hash, &v.oracle_pubkey, &v.signature),
        "a signature for nonce=0 must NOT verify against nonce=1"
    );
}

#[test]
fn tampered_covenant_id_is_rejected() {
    let kp = test_keypair();
    let v = sign_verdict(&kp, 5, 0, &TEST_COVENANT_ID);

    let mut other_id = TEST_COVENANT_ID;
    other_id[0] ^= 0x01; // flip one bit
    let tampered_hash = verdict_msg_hash(5, 0, &other_id);
    assert!(
        !verify_verdict(&tampered_hash, &v.oracle_pubkey, &v.signature),
        "a signature for one covenant_id must NOT verify against another"
    );
}

#[test]
fn wrong_pubkey_is_rejected() {
    // A different key's signature must not verify under the test pubkey.
    let secp = Secp256k1::new();
    let kp = test_keypair();
    let v = sign_verdict(&kp, 5, 0, &TEST_COVENANT_ID);

    let (other_sk, _) = secp.generate_keypair(&mut rand::thread_rng());
    let other_kp = Keypair::from_secret_key(&secp, &other_sk);
    let other = sign_verdict(&other_kp, 5, 0, &TEST_COVENANT_ID);

    // other's signature, but our pubkey => must fail.
    assert!(
        !verify_verdict(&v.msg_hash, &v.oracle_pubkey, &other.signature),
        "another key's signature must NOT verify under our pubkey"
    );
}

#[test]
fn oracle_pkh_is_blake2b_of_pubkey() {
    let kp = test_keypair();
    let v = sign_verdict(&kp, 5, 0, &TEST_COVENANT_ID);
    assert_eq!(v.oracle_pkh, blake2b_256(&v.oracle_pubkey));
}
