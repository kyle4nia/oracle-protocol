//! Off-chain oracle verdict signer for Oracle Protocol.
//!
//! Produces a portable "verdict" signature for the v4 detached-verdict model:
//! an oracle signs a verdict authorizing a reputation delta; anyone can then
//! carry that signature on-chain, where the covenant verifies it with
//! OpCheckSigFromStack (CSFS, 0xd7).
//!
//! The on-chain verification format was confirmed from the Kaspa txscript
//! source (tn10-toc3):
//!   - OpCheckSigFromStack pops [signature, msg_hash, pubkey]; msg_hash must be
//!     exactly 32 bytes; verification uses secp256k1::Message::from_digest
//!     directly (plain BIP340 schnorr, no domain separation).
//!   - OpBlake2b = blake2b_simd::Params::new().hash_length(32), unkeyed.
//!
//! This crate builds the identical bytes off-chain so the produced signature
//! validates under that exact on-chain check.
//!
//! VERDICT MESSAGE LAYOUT (must match the contract byte-for-byte):
//!   verdict_bytes = delta(8B LE) || nonce(8B LE) || covenant_id(32B) = 48 bytes
//!   msg_hash      = blake2b_256(verdict_bytes)
//!   signature     = schnorr_sign(msg_hash) with oracle x-only key

use blake2b_simd::Params;
use secp256k1::{Keypair, Message, Secp256k1, XOnlyPublicKey, schnorr::Signature};

/// The 48-byte verdict message: delta(8 LE) || nonce(8 LE) || covenant_id(32).
pub fn build_verdict_bytes(delta: i64, nonce: i64, covenant_id: &[u8; 32]) -> [u8; 48] {
    let mut out = [0u8; 48];
    out[0..8].copy_from_slice(&delta.to_le_bytes());
    out[8..16].copy_from_slice(&nonce.to_le_bytes());
    out[16..48].copy_from_slice(covenant_id);
    out
}

/// blake2b-256 over arbitrary data, matching the OpBlake2b opcode exactly
/// (Params::new().hash_length(32), unkeyed).
pub fn blake2b_256(data: &[u8]) -> [u8; 32] {
    let hash = Params::new().hash_length(32).to_state().update(data).finalize();
    hash.as_bytes().try_into().expect("blake2b hash_length(32) yields 32 bytes")
}

/// The 32-byte message hash that gets signed: blake2b_256(verdict_bytes).
pub fn verdict_msg_hash(delta: i64, nonce: i64, covenant_id: &[u8; 32]) -> [u8; 32] {
    blake2b_256(&build_verdict_bytes(delta, nonce, covenant_id))
}

/// A signed verdict, with everything the spender needs to push into the sigScript.
pub struct SignedVerdict {
    pub verdict_bytes: [u8; 48],
    pub msg_hash: [u8; 32],
    pub oracle_pubkey: [u8; 32], // x-only
    pub oracle_pkh: [u8; 32],    // blake2b_256(oracle_pubkey), baked into ctor
    pub signature: [u8; 64],
}

/// Sign a verdict with the given oracle keypair.
pub fn sign_verdict(keypair: &Keypair, delta: i64, nonce: i64, covenant_id: &[u8; 32]) -> SignedVerdict {
    let secp = Secp256k1::new();
    let (xonly, _parity) = keypair.x_only_public_key();
    let oracle_pubkey = xonly.serialize();
    let oracle_pkh = blake2b_256(&oracle_pubkey);

    let verdict_bytes = build_verdict_bytes(delta, nonce, covenant_id);
    let msg_hash = blake2b_256(&verdict_bytes);

    let message = Message::from_digest(msg_hash);
    let signature: Signature = secp.sign_schnorr_no_aux_rand(&message, keypair);

    SignedVerdict {
        verdict_bytes,
        msg_hash,
        oracle_pubkey,
        oracle_pkh,
        signature: signature.serialize(),
    }
}

/// Verify a signed verdict the same way the node's OpCheckSigFromStack will:
/// plain BIP340 schnorr over the 32-byte msg_hash, against the x-only pubkey.
pub fn verify_verdict(msg_hash: &[u8; 32], oracle_pubkey: &[u8; 32], signature: &[u8; 64]) -> bool {
    let secp = Secp256k1::new();
    let (Ok(pk), Ok(sig), msg) = (
        XOnlyPublicKey::from_slice(oracle_pubkey),
        Signature::from_slice(signature),
        Message::from_digest(*msg_hash),
    ) else {
        return false;
    };
    secp.verify_schnorr(&sig, &msg, &pk).is_ok()
}

/// Hex-encode a byte slice (lowercase, no prefix).
pub fn hex(bytes: &[u8]) -> String {
    let mut out = vec![0u8; bytes.len() * 2];
    faster_hex::hex_encode(bytes, &mut out).expect("hex encode");
    String::from_utf8(out).expect("hex is valid utf8")
}

/// Decode exactly 32 bytes from a 64-char hex string.
pub fn covenant_id_from_hex(s: &str) -> Result<[u8; 32], String> {
    let mut out = [0u8; 32];
    faster_hex::hex_decode(s.as_bytes(), &mut out)
        .map_err(|_| "covenant_id must be 64 hex chars (32 bytes)".to_string())?;
    Ok(out)
}
