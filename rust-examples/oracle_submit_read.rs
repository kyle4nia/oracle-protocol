// Oracle Protocol — submission client, READ HALF.
// Connects to PsychoNode over gRPC, fetches the rep=100 genesis UTXO,
// derives the covenant_id from the REAL outpoint, prints everything.
// No transaction is built or broadcast here.

use kaspa_addresses::Address;
use kaspa_consensus_core::{
    hashing::covenant_id::covenant_id,
    tx::{TransactionOutpoint, TransactionOutput, UtxoEntry},
};
use kaspa_grpc_client::GrpcClient;
use kaspa_notify::subscription::context::SubscriptionContext;
use kaspa_rpc_core::{api::rpc::RpcApi, notify::mode::NotificationMode};
use kaspa_txscript::pay_to_script_hash_script;

const NODE: &str = "192.168.0.206:16210";
const GENESIS_ADDR: &str = "kaspatest:pr64eayfczjzmrkk4s68cmt7kr7r7ejrx54sfvqvw86krlpst6u47xznrlg23";

// rep=105 target redeem script (68 bytes; byte[1]=105). Used to preview the
// continuation output that the covenant_id will be derived against.
const REDEEM_REP105: [u8; 68] = [
    8, 105, 0, 0, 0, 0, 0, 0, 0, 118, 82, 121, 147, 118, 0, 162, 105, 185, 203, 81,
    156, 105, 118, 88, 205, 1, 8, 124, 126, 185, 118, 201, 118, 1, 68, 148, 89, 147,
    124, 188, 126, 170, 2, 0, 0, 1, 170, 126, 1, 32, 126, 124, 126, 1, 135, 126, 185,
    0, 204, 195, 135, 105, 0, 122, 117, 117, 117, 81,
];

const FEE: u64 = 10_000;

#[tokio::main]
async fn main() {
    let subscription_context = SubscriptionContext::new();
    println!("[read] connecting to node at grpc://{NODE}");
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
    .expect("failed to connect to node");
    println!("[read] connected.");

    let addr = Address::try_from(GENESIS_ADDR).expect("bad address");
    let resp = rpc.get_utxos_by_addresses(vec![addr]).await.expect("get_utxos failed");

    if resp.is_empty() {
        println!("[read] NO UTXOs found on genesis address. Node synced? Address funded?");
        return;
    }

    let entry = &resp[0];
    let outpoint = TransactionOutpoint::from(entry.outpoint.clone());
    let utxo = UtxoEntry::from(entry.utxo_entry.clone());

    println!("[read] genesis UTXO found:");
    println!("       txid:   {}", outpoint.transaction_id);
    println!("       index:  {}", outpoint.index);
    println!("       amount: {} sompi", utxo.amount);
    println!("       spk:    {}", faster_hex::hex_string(&utxo.script_public_key.script()));

    // Preview the rep=105 continuation output (what the spend will create).
    let target_spk = pay_to_script_hash_script(&REDEEM_REP105);
    let out_value = utxo.amount - FEE;
    let preview_output =
        TransactionOutput { value: out_value, script_public_key: target_spk, covenant: None };

    // Derive covenant_id from the REAL genesis outpoint + the auth outputs (genesis case).
    let auth_outputs = std::iter::once((0u32, &preview_output));
    let cid = covenant_id(outpoint, auth_outputs);

    println!("[read] derived covenant_id (from REAL outpoint): {cid}");
    println!("[read] continuation output value (amount - fee): {out_value} sompi");
    println!("[read] OK — plumbing proven. Ready to build the spend half.");
}
