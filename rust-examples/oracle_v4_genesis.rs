// Oracle Protocol v4 - GENESIS ESTABLISHING tx.
// Spends a plain P2PK wallet UTXO (signed with our hex key) and creates the
// v4 covenant UTXO at the covenant P2SH address, binding it via
// populate_genesis_covenants so it carries the correct covenant_id.
//
// v4 vs v3: script bytes differ (162 vs 68). Verdict is NOT signed at genesis:
// OpInputCovenantId returns ZERO_HASH on a genesis-establishing input, so a
// verdict signed against the real covenant_id can only be verified on a
// continuation spend. Genesis just plants the covenant with oracle_pkh baked
// in; the first authenticated hop is a separate spender (step 8).
//
// SAFETY: dry-run by default. Pass `--go` to broadcast.

use kaspa_addresses::Address;
use kaspa_consensus_core::{
    constants::TX_VERSION_TOCCATA,
    sign::sign,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{
        GenesisCovenantGroup, MutableTransaction, Transaction, TransactionInput, TransactionOutpoint,
        TransactionOutput, ComputeCommit, UtxoEntry,
    },
};
use kaspa_grpc_client::GrpcClient;
use kaspa_notify::subscription::context::SubscriptionContext;
use kaspa_rpc_core::{api::rpc::RpcApi, notify::mode::NotificationMode};
use kaspa_txscript::pay_to_script_hash_script;
use secp256k1::Keypair;

const NODE: &str = "127.0.0.1:16210";
// Plain P2PK wallet address we control (key below). Reused from v3 genesis.
const FUNDING_ADDR: &str = "kaspatest:qzqqxuwvaev5jxtvsg8563xr9s2g4t7us674e0gthytzz6cl4w8tje8ffjays";
const PRIV_HEX: &str = "0cfebe8b1d016918048838eaf045dec9988da519831376610cb35f315b695c11";

// v4 redeem script (162 bytes). Byte-for-byte the same script proven in
// oracle_v4_verify.rs local VM run (6-case accept/reject). oracle_pkh baked in.
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

const FEE: u64 = 200_000;

#[tokio::main]
async fn main() {
    let go = std::env::args().any(|a| a == "--go");

    // Load our keypair from hex.
    let mut sk = [0u8; 32];
    faster_hex::hex_decode(PRIV_HEX.as_bytes(), &mut sk).expect("bad hex key");
    let keypair = Keypair::from_seckey_slice(secp256k1::SECP256K1, &sk).expect("bad key");

    let subscription_context = SubscriptionContext::new();
    println!("[v4-genesis] connecting to grpc://{NODE}");
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
    println!("[v4-genesis] connected.");

    // Fetch the funding UTXO.
    let addr = Address::try_from(FUNDING_ADDR).expect("bad funding addr");
    let resp = rpc.get_utxos_by_addresses(vec![addr]).await.expect("get_utxos failed");
    if resp.is_empty() {
        println!("[v4-genesis] no UTXO on funding address. aborting.");
        return;
    }
    let entry = &resp[0];
    let outpoint = TransactionOutpoint::from(entry.outpoint.clone());
    let in_amount = entry.utxo_entry.amount;
    let in_utxo = UtxoEntry::from(entry.utxo_entry.clone());
    println!("[v4-genesis] funding input: {}:{}  amount {} sompi", outpoint.transaction_id, outpoint.index, in_amount);

    // Plain signed input (no covenant). SigopCount(1) = standard single-sig.
    let input = TransactionInput {
        previous_outpoint: outpoint,
        signature_script: vec![],
        sequence: 0,
        compute_commit: ComputeCommit::ComputeBudget(10.into()),
    };

    // The genesis covenant output: pays to the v4 covenant P2SH, carries the value.
    let covenant_spk = pay_to_script_hash_script(&REDEEM_V4.to_vec());
    let out_value = in_amount - FEE;
    let output = TransactionOutput { value: out_value, script_public_key: covenant_spk, covenant: None };

    // Build unsigned tx (Toccata version so covenant bindings are allowed).
    let unsigned = Transaction::new_non_finalized(
        TX_VERSION_TOCCATA,
        vec![input],
        vec![output],
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        vec![],
    );
    let mut mtx = MutableTransaction::with_entries(unsigned, vec![in_utxo]);

    // Populate the genesis covenant binding: input 0 authorizes output 0.
    // This derives the covenant_id from (input outpoint, [output 0]) and sets it.
    mtx.tx
        .populate_genesis_covenants(&[GenesisCovenantGroup::new(0, vec![0])])
        .expect("populate_genesis_covenants failed");

    let bound_cid = mtx.tx.outputs[0].covenant.as_ref().map(|c| c.covenant_id);
    println!("[v4-genesis] genesis covenant_id bound to output: {bound_cid:?}");
    println!("[v4-genesis] covenant output value: {out_value} sompi (fee {FEE})");

    // Sign the plain input with our key.
    let signed = sign(mtx, keypair);
    let mut tx = signed.tx;
    tx.finalize();

    println!("[v4-genesis] built+signed tx id: {}", tx.id());

    if !go {
        println!();
        println!("[v4-genesis] DRY RUN - not submitted. Re-run with --go to broadcast.");
        return;
    }

    println!("[v4-genesis] SUBMITTING...");
    match rpc.submit_transaction((&tx).into(), false).await {
        Ok(resp) => println!("[v4-genesis] ACCEPTED. node response: {resp:?}"),
        Err(e) => println!("[v4-genesis] REJECTED: {e}"),
    }
}
