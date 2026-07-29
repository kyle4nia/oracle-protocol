//! oracle_v4_verify.rs — local VM accept/reject proof for the v4 CSFS gate.
//!
//! Loads the compiled v4 OracleRep script, constructs a rep=100 -> rep=105
//! covenant spend authorized by a signed oracle verdict, and runs the txscript
//! engine SIX ways:
//!   1. valid verdict            -> MUST ACCEPT
//!   2. tampered delta           -> MUST REJECT
//!   3. tampered nonce           -> MUST REJECT
//!   4. tampered covenant_id     -> MUST REJECT
//!   5. wrong signing key        -> MUST REJECT
//!   6. missing signature        -> MUST REJECT
//!
//! The five rejections are the real proof: a no-op gate would accept all six.
//! The signing here replicates the oracle-signer crate byte-for-byte:
//!   verdict_bytes = delta(8 LE) || nonce(8 LE) || covenant_id(32)
//!   msg_hash      = blake2b_256(verdict_bytes)
//!   signature     = schnorr(msg_hash) with oracle x-only key

use blake2b_simd::Params;
use kaspa_consensus_core::hashing::sighash::SigHashReusedValuesUnsync;
use kaspa_consensus_core::subnets::SubnetworkId;
use kaspa_consensus_core::tx::{
    CovenantBinding, PopulatedTransaction, Transaction, TransactionInput, TransactionOutpoint,
    TransactionOutput, UtxoEntry,
};
use kaspa_hashes::Hash;
use kaspa_txscript::caches::Cache;
use kaspa_txscript::covenants::CovenantsContext;
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript::{pay_to_script_hash_script, EngineCtx, EngineFlags, TxScriptEngine};
use secp256k1::{Keypair, Message, Secp256k1, SecretKey};

// Compiled v4 script (silverc -c on oracle_rep_v4.sil + oracle_ctor_v4.json).
// rep low byte at index 1, nonce low byte at index 10; oracle_pkh at [25..57].
const REDEEM_V4: [u8; 162] = [
    8, 100, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 118, 85, 121, 85, 121, 118,
    170, 32, 91, 194, 238, 84, 170, 67, 70, 236, 227, 210, 107, 150, 191, 17, 60, 75,
    135, 142, 238, 255, 240, 177, 227, 55, 231, 199, 70, 15, 190, 82, 35, 54, 135, 105,
    85, 121, 82, 121, 88, 205, 84, 121, 126, 185, 207, 126, 170, 82, 121, 215, 105, 84,
    121, 82, 121, 147, 118, 0, 162, 105, 185, 203, 81, 156, 105, 118, 88, 205, 1, 8, 124,
    126, 84, 121, 81, 147, 88, 205, 1, 8, 124, 126, 126, 185, 118, 201, 118, 2, 162, 0,
    148, 1, 18, 147, 124, 188, 126, 170, 2, 0, 0, 1, 170, 126, 1, 32, 126, 124, 126, 1,
    135, 126, 185, 0, 204, 195, 135, 105, 0, 122, 117, 0, 122, 117, 0, 122, 117, 0, 122,
    117, 117, 117, 117, 117, 117, 81,
];

// The oracle secret matching the pkh baked into REDEEM_V4.
// Paste the oracle_secret(32) hex from tn10.oracle-key.txt here.
const ORACLE_SECRET_HEX: &str = "REPLACE_WITH_ORACLE_SECRET_HEX";

const GENESIS_AMOUNT: u64 = 10_000_000_000;
const FEE: u64 = 10_000;
const DELTA: i64 = 5;

/// Build a v4 redeem script with a given rep and nonce in the state slots.
fn redeem_with_state(rep: u8, nonce: u8) -> Vec<u8> {
    let mut s = REDEEM_V4.to_vec();
    s[1] = rep; // rep low byte (LE), high bytes stay 0
    s[10] = nonce; // nonce low byte (LE), high bytes stay 0
    s
}

/// blake2b-256 unkeyed, matching OpBlake2b and the signer crate.
fn blake2b_256(data: &[u8]) -> [u8; 32] {
    let h = Params::new().hash_length(32).to_state().update(data).finalize();
    h.as_bytes().try_into().unwrap()
}

/// verdict_bytes = delta(8 LE) || nonce(8 LE) || covenant_id(32).
fn build_verdict(delta: i64, nonce: i64, covenant_id: &[u8; 32]) -> [u8; 48] {
    let mut out = [0u8; 48];
    out[0..8].copy_from_slice(&delta.to_le_bytes());
    out[8..16].copy_from_slice(&nonce.to_le_bytes());
    out[16..48].copy_from_slice(covenant_id);
    out
}

/// Sign a verdict, returning the 64-byte schnorr signature over its msg_hash.
fn sign_verdict(kp: &Keypair, delta: i64, nonce: i64, covenant_id: &[u8; 32]) -> [u8; 64] {
    let secp = Secp256k1::new();
    let msg_hash = blake2b_256(&build_verdict(delta, nonce, covenant_id));
    let msg = Message::from_digest(msg_hash);
    secp.sign_schnorr_no_aux_rand(&msg, kp).serialize()
}

/// Run one covenant spend attempt. `sig_opt = None` omits the signature push.
/// Returns true if the engine ACCEPTED the spend.
fn run_case(
    label: &str,
    kp: &Keypair,
    sig_delta: i64,
    sig_nonce: i64,
    sig_covid_override: Option<[u8; 32]>,
    spend_delta: i64,
    include_sig: bool,
) -> bool {
    let secp = Secp256k1::new();
    let oracle_pk = kp.x_only_public_key().0.serialize();

    let redeem_src = REDEEM_V4.to_vec(); // rep=100, nonce=0
    let redeem_tgt = redeem_with_state(105, 1); // rep=105, nonce=1
    let source_spk = pay_to_script_hash_script(&redeem_src);
    let target_spk = pay_to_script_hash_script(&redeem_tgt);

    let genesis_outpoint = TransactionOutpoint::new(Hash::from_bytes([7u8; 32]), 0);

    // Continuation transition: the spent input ALREADY carries the covenant id
    // (established at a prior genesis). OpInputCovenantId returns this exact id
    // at runtime, so the oracle signs against it. Fixed, recognizable value.
    let covenant_id = Hash::from_bytes([0x5au8; 32]);
    let real_covid: [u8; 32] = covenant_id.as_bytes();

    // Continuation output carries the same covenant id and authorizing input.
    let mut output = TransactionOutput::new(GENESIS_AMOUNT - FEE, target_spk);
    output.covenant = Some(CovenantBinding { covenant_id, authorizing_input: 0 });

    // The oracle signs the real covenant_id unless this case overrides it (case 4).
    let sig_covid = sig_covid_override.unwrap_or(real_covid);
    let sig = sign_verdict(kp, sig_delta, sig_nonce, &sig_covid);

    // sigScript arg order matches v4 ABI: delta, oracle_pk, oracle_sig, then redeem reveal.
    let mut b = ScriptBuilder::new();
    b.add_i64(spend_delta).unwrap();
    b.add_data(&oracle_pk).unwrap();
    if include_sig {
        b.add_data(&sig).unwrap();
    }
    let sig_script = b.add_data(&redeem_src).unwrap().drain();

    let input = TransactionInput::new(genesis_outpoint, sig_script, 0, 1);
    let tx = Transaction::new(0, vec![input], vec![output], 0, SubnetworkId::default(), 0, vec![]);
    // Spent UTXO carries the covenant id -> continuation path, OpInputCovenantId != ZERO.
    let utxo = UtxoEntry::new(GENESIS_AMOUNT, source_spk, 0, false, Some(covenant_id));

    let sig_cache = Cache::new(10_000);
    let reused = SigHashReusedValuesUnsync::new();
    let flags = EngineFlags { covenants_enabled: true, ..Default::default() };
    let populated = PopulatedTransaction::new(&tx, vec![utxo.clone()]);
    let cov_ctx = CovenantsContext::from_tx(&populated).expect("covenant ctx");
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused).with_covenants_ctx(&cov_ctx);
    let mut vm = TxScriptEngine::from_transaction_input(&populated, &tx.inputs[0], 0, &utxo, ctx, flags);
    let _ = &secp;

    let result = vm.execute();
    let accepted = result.is_ok();
    match &result {
        Ok(()) => println!("[{label}] ACCEPTED"),
        Err(e) => println!("[{label}] REJECTED ({e:?})"),
    }
    accepted
}

fn main() {
    let secp = Secp256k1::new();
    let mut sk = [0u8; 32];
    faster_hex::hex_decode(ORACLE_SECRET_HEX.as_bytes(), &mut sk).expect("bad ORACLE_SECRET_HEX");
    let kp = Keypair::from_secret_key(&secp, &SecretKey::from_slice(&sk).unwrap());

    // A different key for the wrong-key case.
    let (wrong_sk, _) = secp.generate_keypair(&mut rand::thread_rng());
    let wrong_kp = Keypair::from_secret_key(&secp, &wrong_sk);

    println!("=== v4 CSFS gate: accept/reject proof ===\n");

    let mut pass = true;

    // 1. Valid: sign (delta=5, nonce=0, real covid), spend delta=5.
    pass &= run_case("1 valid", &kp, DELTA, 0, None, DELTA, true) == true;
    // 2. Tampered delta: sign delta=5 but spend delta=6 (sig won't match spent value).
    pass &= run_case("2 tampered-delta", &kp, DELTA, 0, None, 6, true) == false;
    // 3. Tampered nonce: sign against nonce=1 while covenant state nonce is 0.
    pass &= run_case("3 tampered-nonce", &kp, DELTA, 1, None, DELTA, true) == false;
    // 4. Tampered covenant_id: sign against a bogus covid (binding stays correct).
    pass &= run_case("4 tampered-covid", &kp, DELTA, 0, Some([0xAAu8; 32]), DELTA, true) == false;
    // 5. Wrong key: valid message but signed by a non-oracle key.
    pass &= run_case("5 wrong-key", &wrong_kp, DELTA, 0, None, DELTA, true) == false;
    // 6. Missing sig: correct everything but omit the signature push.
    pass &= run_case("6 missing-sig", &kp, DELTA, 0, None, DELTA, false) == false;

    println!();
    if pass {
        println!("ALL SIX CASES BEHAVED CORRECTLY — v4 CSFS gate PROVEN.");
    } else {
        println!("*** FAILURE — at least one case behaved wrong. Gate NOT proven. ***");
    }
}
