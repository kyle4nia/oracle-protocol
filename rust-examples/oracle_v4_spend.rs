// Oracle Protocol v4 - GENERALIZED AUTHENTICATED SPENDER.
//
// Finds the current v4 covenant head by scanning the (rep, nonce) trajectory,
// signs an oracle verdict for the next transition, and broadcasts it. Drives
// every hop with no edits: run, verify, run again.
//
// State discovery: chain stores only the P2SH hash, so state can't be read
// off the UTXO. Instead we derive candidate addresses from candidate states
// (rep = START_REP + k*DELTA, nonce = k) and batch-query them; the one that
// holds a covenant UTXO tells us the state by construction, v3-scanner style.
// ASSUMPTION: uniform delta history. If a hop ever applies a different delta,
// widen the scan or track the head out-of-band.
//
// Verdict + sigScript pattern byte-for-byte from the proven oracle_v4_verify.rs:
//   verdict  = delta(8 LE) || nonce_prev(8 LE) || covenant_id(32)
//   msg_hash = blake2b_256(verdict); sig = schnorr(msg_hash)
//   sigScript = delta || oracle_pk(32) || oracle_sig(64) || redeem_reveal
//
// Proven on-chain 2026-07-29: accept eba6dee9...44c80ee (rep 100->105),
// reject 2fe13331...3b9badd (stale nonce replay, engine false-stack).
//
// SAFETY: dry-run by default. Pass `--go` to broadcast.

use blake2b_simd::Params;
use kaspa_addresses::{Address, Prefix, Version as AddrVersion};
use kaspa_consensus_core::{
    constants::TX_VERSION_TOCCATA,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{
        ComputeCommit, CovenantBinding, Transaction, TransactionInput, TransactionOutpoint,
        TransactionOutput, UtxoEntry,
    },
};
use kaspa_grpc_client::GrpcClient;
use kaspa_notify::subscription::context::SubscriptionContext;
use kaspa_rpc_core::{api::rpc::RpcApi, notify::mode::NotificationMode};
use kaspa_txscript::{pay_to_script_hash_script, script_builder::ScriptBuilder};
use secp256k1::{Keypair, Message, Secp256k1, SecretKey};

const NODE: &str = "127.0.0.1:16210";
// Oracle secret file (gitignored via *.oracle-key.txt): labeled dump, hex on
// the oracle_secret line.
const ORACLE_KEY_PATH: &str = "C:\\oracle-protocol\\tn10.oracle-key.txt";

// v4 redeem TEMPLATE (162 bytes, genesis state rep=100 nonce=0).
// rep: 8B LE at [1..9]. nonce: 8B LE at [10..18]. oracle_pkh at [25..57].
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

const REP_OFF: usize = 1;
const NONCE_OFF: usize = 10;
const FEE: u64 = 200_000;
const DELTA: i64 = 5;
const START_REP: u64 = 100;
const MAX_HOPS: u64 = 200;

fn blake2b_256(data: &[u8]) -> [u8; 32] {
    let h = Params::new().hash_length(32).to_state().update(data).finalize();
    h.as_bytes().try_into().unwrap()
}

fn build_verdict(delta: i64, nonce: i64, covenant_id: &[u8; 32]) -> [u8; 48] {
    let mut out = [0u8; 48];
    out[0..8].copy_from_slice(&delta.to_le_bytes());
    out[8..16].copy_from_slice(&nonce.to_le_bytes());
    out[16..48].copy_from_slice(covenant_id);
    out
}

/// Build a v4 redeem script for a given (rep, nonce). Full 8-byte LE writes,
/// so values above 255 are handled.
fn redeem_with_state(rep: u64, nonce: u64) -> Vec<u8> {
    let mut s = REDEEM_V4.to_vec();
    s[REP_OFF..REP_OFF + 8].copy_from_slice(&rep.to_le_bytes());
    s[NONCE_OFF..NONCE_OFF + 8].copy_from_slice(&nonce.to_le_bytes());
    s
}

/// P2SH address for a redeem script (testnet prefix).
fn addr_for_redeem(redeem: &[u8]) -> Address {
    let spk = pay_to_script_hash_script(&redeem.to_vec());
    // P2SH script is: OP_BLAKE2B <32-byte hash> OP_EQUAL; hash sits at [2..34].
    let hash = &spk.script()[2..34];
    Address::new(Prefix::Testnet, AddrVersion::ScriptHash, hash)
}

#[tokio::main]
async fn main() {
    let go = std::env::args().any(|a| a == "--go");

    // --- Load oracle secret (hex after the colon on the oracle_secret line). ---
    let key_file = std::fs::read_to_string(ORACLE_KEY_PATH)
        .expect("failed to read tn10.oracle-key.txt");
    let secret_hex: String = key_file
        .lines()
        .find(|l| l.contains("oracle_secret"))
        .expect("no oracle_secret line in key file")
        .split(':')
        .nth(1)
        .expect("malformed oracle_secret line")
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_hexdigit())
        .collect();
    let mut sk_bytes = [0u8; 32];
    faster_hex::hex_decode(secret_hex.as_bytes(), &mut sk_bytes)
        .expect("bad oracle secret hex");
    let secp = Secp256k1::new();
    let kp = Keypair::from_secret_key(&secp, &SecretKey::from_slice(&sk_bytes).unwrap());
    let oracle_pk = kp.x_only_public_key().0.serialize();
    println!("[v4-spend] oracle key loaded, x-only pubkey: {}", faster_hex::hex_string(&oracle_pk));

    // --- Connect. ---
    let subscription_context = SubscriptionContext::new();
    println!("[v4-spend] connecting to grpc://{NODE}");
    let rpc = GrpcClient::connect_with_args(
        NotificationMode::Direct,
        format!("grpc://{NODE}"),
        Some(subscription_context.clone()),
        true,
        None,
        false,
        Some(500_000),
        Default::default(),
    )
    .await
    .expect("connect failed");
    println!("[v4-spend] connected.");

    // --- Scan the state trajectory for the live head. One batched query. ---
    let candidates: Vec<Address> = (0..MAX_HOPS)
        .map(|k| addr_for_redeem(&redeem_with_state(START_REP + k * DELTA as u64, k)))
        .collect();
    let resp = rpc.get_utxos_by_addresses(candidates.clone()).await.expect("get_utxos failed");
    if resp.is_empty() {
        println!("[v4-spend] no covenant UTXO found in {MAX_HOPS}-hop scan window. aborting.");
        return;
    }
    // Map the hit address back to its k. (Entries carry the queried address.)
    let entry = &resp[0];
    let hit_addr = entry.address.clone().expect("utxo entry missing address");
    let k = candidates
        .iter()
        .position(|a| *a == hit_addr)
        .expect("hit address not in candidate list") as u64;
    let current_rep = START_REP + k * DELTA as u64;
    let current_nonce = k;

    let outpoint = TransactionOutpoint::from(entry.outpoint.clone());
    let in_amount = entry.utxo_entry.amount;
    let in_utxo = UtxoEntry::from(entry.utxo_entry.clone());
    let covenant_id = in_utxo
        .covenant_id
        .expect("head UTXO has no covenant binding, unexpected");
    let covid_bytes: [u8; 32] = covenant_id.as_bytes();
    println!(
        "[v4-spend] head found at hop {k}: rep={current_rep} nonce={current_nonce}"
    );
    println!(
        "[v4-spend] head UTXO: {}:{}  amount {}  covid {}",
        outpoint.transaction_id, outpoint.index, in_amount, covenant_id
    );

    // --- Sign the verdict against the LIVE state. ---
    let verdict = build_verdict(DELTA, current_nonce as i64, &covid_bytes);
    let msg_hash = blake2b_256(&verdict);
    let msg = Message::from_digest(msg_hash);
    let sig = secp.sign_schnorr_no_aux_rand(&msg, &kp).serialize();
    println!("[v4-spend] verdict signed: delta={DELTA}  nonce_prev={current_nonce}");

    // --- sigScript: delta || oracle_pk || oracle_sig || redeem_reveal. ---
    let redeem_src = redeem_with_state(current_rep, current_nonce);
    let mut b = ScriptBuilder::new();
    b.add_i64(DELTA).unwrap();
    b.add_data(&oracle_pk).unwrap();
    b.add_data(&sig).unwrap();
    let sig_script = b.add_data(&redeem_src).unwrap().drain();

    // --- Continuation output. ---
    let next_rep = current_rep + DELTA as u64;
    let next_nonce = current_nonce + 1;
    let redeem_tgt = redeem_with_state(next_rep, next_nonce);
    let target_spk = pay_to_script_hash_script(&redeem_tgt);
    let out_value = in_amount - FEE;

    let output = TransactionOutput {
        value: out_value,
        script_public_key: target_spk,
        covenant: Some(CovenantBinding { covenant_id, authorizing_input: 0 }),
    };

    let input = TransactionInput {
        previous_outpoint: outpoint,
        signature_script: sig_script,
        sequence: 0,
        compute_commit: ComputeCommit::ComputeBudget(10.into()),
    };

    let mut tx = Transaction::new_non_finalized(
        TX_VERSION_TOCCATA,
        vec![input],
        vec![output],
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        vec![],
    );
    tx.finalize();

    println!("[v4-spend] built tx id: {}", tx.id());
    println!("[v4-spend] rep {current_rep} -> {next_rep}, nonce {current_nonce} -> {next_nonce}");
    println!("[v4-spend] output value: {out_value} sompi (fee {FEE})");

    if !go {
        println!();
        println!("[v4-spend] DRY RUN - not submitted. Re-run with --go to broadcast.");
        return;
    }

    println!("[v4-spend] SUBMITTING...");
    match rpc.submit_transaction((&tx).into(), false).await {
        Ok(resp) => println!("[v4-spend] ACCEPTED. node response: {resp:?}"),
        Err(e) => println!("[v4-spend] REJECTED: {e}"),
    }
}
