// Oracle Protocol — submission client, SPEND HALF (continuation). GENERALIZED.
// Spends the current covenant head UTXO -> current_rep + DELTA.
// Reads the current rep value FROM THE CHAIN (off the input's redeem script),
// so this same file works for every hop (100->105, 105->110, ...) with no edit.
//
// This is a continuation spend: the input already carries covenant_id, so the
// covenant context populates auth outputs and OpAuthOutputCount works.
//
// SAFETY: dry-run by default. Pass `--go` to broadcast.

use kaspa_addresses::Prefix;
use kaspa_consensus_core::{
    constants::TX_VERSION_TOCCATA,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{
        CovenantBinding, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput, TxInputMass,
    },
};
use kaspa_grpc_client::GrpcClient;
use kaspa_notify::subscription::context::SubscriptionContext;
use kaspa_rpc_core::{api::rpc::RpcApi, notify::mode::NotificationMode};
use kaspa_txscript::{extract_script_pub_key_address, pay_to_script_hash_script};
use kaspa_txscript::script_builder::ScriptBuilder;

const NODE: &str = "192.168.0.206:16210";

// The 68-byte redeem TEMPLATE. byte[1] is the rep value; we overwrite it.
// (Value below is rep=100, but we never assume that — we read the real rep
//  from whatever UTXO we actually find on-chain.)
const REDEEM_TEMPLATE: [u8; 68] = [
    8, 100, 0, 0, 0, 0, 0, 0, 0, 118, 82, 121, 147, 118, 0, 162, 105, 185, 203, 81,
    156, 105, 118, 88, 205, 1, 8, 124, 126, 185, 118, 201, 118, 1, 68, 148, 89, 147,
    124, 188, 126, 170, 2, 0, 0, 1, 170, 126, 1, 32, 126, 124, 126, 1, 135, 126, 185,
    0, 204, 195, 135, 105, 0, 122, 117, 117, 117, 81,
];

const FEE: u64 = 200_000;
const DELTA: i64 = 5;

// State geometry (from status doc): byte[8] repState, little-endian, starting
// at redeem-script byte index 1. We decode/encode all 8 bytes so rep > 255 is
// handled correctly, not just byte[1].
const REP_OFF: usize = 1;
const REP_LEN: usize = 8;

/// Read the rep value out of a 68-byte redeem script (8-byte little-endian).
fn read_rep(script: &[u8]) -> u64 {
    let mut v: u64 = 0;
    for i in 0..REP_LEN {
        v |= (script[REP_OFF + i] as u64) << (8 * i);
    }
    v
}

/// Build a redeem script for a given rep value (8-byte little-endian write).
fn redeem_with_rep(rep: u64) -> Vec<u8> {
    let mut s = REDEEM_TEMPLATE.to_vec();
    for i in 0..REP_LEN {
        s[REP_OFF + i] = ((rep >> (8 * i)) & 0xff) as u8;
    }
    s
}

#[tokio::main]
async fn main() {
    let go = std::env::args().any(|a| a == "--go");

    let subscription_context = SubscriptionContext::new();
    println!("[spend] connecting to grpc://{NODE}");
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
    println!("[spend] connected.");

    // --- Find the current head UTXO by scanning candidate rep addresses. ---
    // We don't know the current rep ahead of time, so we derive the P2SH address
    // for each plausible rep, ask the node for UTXOs there, and keep the one that
    // CARRIES a covenant_id. In practice only the live head has a covenant UTXO.
    let mut found: Option<(TransactionOutpoint, u64, _, u64)> = None;

    // Scan a reasonable window of rep values. Starts at 100 (genesis), steps by
    // DELTA. Widen MAX_HOPS if you've done many hops since genesis.
    const START_REP: u64 = 100;
    const MAX_HOPS: u64 = 200;
    'scan: for hop in 0..=MAX_HOPS {
        let rep = START_REP + (hop * DELTA as u64);
        let redeem = redeem_with_rep(rep);
        let spk = pay_to_script_hash_script(&redeem);
        let addr = match extract_script_pub_key_address(&spk, Prefix::Testnet) {
            Ok(a) => a,
            Err(_) => continue,
        };

        let resp = match rpc.get_utxos_by_addresses(vec![addr]).await {
            Ok(r) => r,
            Err(_) => continue,
        };
        for e in &resp {
            if e.utxo_entry.covenant_id.is_some() {
                let outpoint = TransactionOutpoint::from(e.outpoint.clone());
                let cid = e.utxo_entry.covenant_id.expect("Some checked");
                found = Some((outpoint, e.utxo_entry.amount, cid, rep));
                break 'scan;
            }
        }
    }

    let (outpoint, in_amount, input_cid, current_rep) = match found {
        Some(t) => t,
        None => {
            println!("[spend] no covenant-carrying head UTXO found in rep window \
                      {START_REP}..={}. Widen MAX_HOPS or check the node is synced.",
                     START_REP + MAX_HOPS * DELTA as u64);
            return;
        }
    };

    let next_rep = (current_rep as i64 + DELTA) as u64;

    println!("[spend] input (covenant UTXO): {}:{}  amount {} sompi",
             outpoint.transaction_id, outpoint.index, in_amount);
    println!("[spend] input covenant_id: {input_cid}");
    println!("[spend] current rep read from chain: {current_rep}  ->  next rep: {next_rep}");

    // Sanity: the rep we derived the address from must match the rep encoded in
    // the redeem script we're about to reveal. (They're the same by construction,
    // but assert it so a geometry mistake fails loud instead of silently.)
    let redeem_current = redeem_with_rep(current_rep);
    assert_eq!(read_rep(&redeem_current), current_rep, "rep encode/decode mismatch");

    // SigScript: push delta, then the CURRENT-rep redeem script (P2SH reveal).
    let sig_script = ScriptBuilder::new()
        .add_i64(DELTA)
        .unwrap()
        .add_data(&redeem_current)
        .unwrap()
        .drain();

    let input = TransactionInput {
        previous_outpoint: outpoint,
        signature_script: sig_script,
        sequence: 0,
        mass: TxInputMass::ComputeBudget(10.into()),
    };

    // next-rep continuation output, bound to the SAME covenant_id from the input.
    let target_spk = pay_to_script_hash_script(&redeem_with_rep(next_rep));
    let out_value = in_amount - FEE;
    let output = TransactionOutput {
        value: out_value,
        script_public_key: target_spk,
        covenant: Some(CovenantBinding { covenant_id: input_cid, authorizing_input: 0 }),
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

    println!("[spend] built tx id: {}", tx.id());
    println!("[spend] delta: {DELTA}  (rep {current_rep} -> {next_rep}), carrying covenant_id {input_cid}");
    println!("[spend] output value: {out_value} sompi (fee {FEE})");

    if !go {
        println!();
        println!("[spend] DRY RUN — not submitted. Re-run with --go to broadcast.");
        return;
    }

    println!("[spend] SUBMITTING...");
    match rpc.submit_transaction((&tx).into(), false).await {
        Ok(resp) => println!("[spend] ACCEPTED. node response: {resp:?}"),
        Err(e) => println!("[spend] REJECTED: {e}"),
    }
}
