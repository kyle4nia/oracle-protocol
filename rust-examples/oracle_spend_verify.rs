use kaspa_consensus_core::hashing;
use kaspa_consensus_core::hashing::sighash::SigHashReusedValuesUnsync;
use kaspa_consensus_core::subnets::SubnetworkId;
use kaspa_consensus_core::tx::{
    CovenantBinding, PopulatedTransaction, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry,
};
use kaspa_hashes::Hash;
use kaspa_txscript::caches::Cache;
use kaspa_txscript::covenants::CovenantsContext;
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript::{pay_to_script_hash_script, EngineCtx, EngineFlags, TxScriptEngine};

// New 67-byte redeem script emitted by silverc for the declarative OracleRep
// (state slot = bytes [1..9], currently 100). Verify these match your latest
// `silverc ... -c` output before trusting a run.
const REDEEM_REP100: [u8; 68] = [
    8, 100, 0, 0, 0, 0, 0, 0, 0, 118, 82, 121, 147, 118, 0, 162, 105, 185, 203, 81,
    156, 105, 118, 88, 205, 1, 8, 124, 126, 185, 118, 201, 118, 1, 68, 148, 89, 147,
    124, 188, 126, 170, 2, 0, 0, 1, 170, 126, 1, 32, 126, 124, 126, 1, 135, 126, 185,
    0, 204, 195, 135, 105, 0, 122, 117, 117, 117, 81,
];

const GENESIS_AMOUNT: u64 = 10_000_000_000;
const FEE: u64 = 10_000;
const DELTA: i64 = 5;

/// Build the rep=105 redeem script: identical to rep=100 except the 8-byte
/// state slot (bytes [1..9]) holds 105 little-endian instead of 100.
fn redeem_with_rep(rep: u8) -> Vec<u8> {
    let mut s = REDEEM_REP100.to_vec();
    s[1] = rep; // low byte of the little-endian 8-byte state; high bytes stay 0
    s
}

fn main() {
    println!("[verify] OracleRep rep=100 -> rep=105 local covenant check");

    let redeem_rep100 = REDEEM_REP100.to_vec();
    let source_spk = pay_to_script_hash_script(&redeem_rep100);
    let target_spk = pay_to_script_hash_script(&redeem_with_rep(105));

    // Genesis outpoint -> derive the covenant id the same way the network does.
    let genesis_outpoint = TransactionOutpoint::new(Hash::from_bytes([7u8; 32]), 0);
    let covenant_id = hashing::covenant_id::covenant_id(genesis_outpoint, std::iter::empty());

    // SigScript pushes the call arg (delta) then the redeem script (P2SH reveal).
    let sig_script = ScriptBuilder::new()
        .add_i64(DELTA)
        .unwrap()
        .add_data(&redeem_rep100)
        .unwrap()
        .drain();

    let input = TransactionInput::new(genesis_outpoint, sig_script, 0, 1);

    // Continuation output must carry the covenant binding so OpAuth*/validateOutputState resolve it.
    let mut output = TransactionOutput::new(GENESIS_AMOUNT - FEE, target_spk.clone());
    output.covenant = Some(CovenantBinding { covenant_id, authorizing_input: 0 });

    let tx = Transaction::new(0, vec![input.clone()], vec![output], 0, SubnetworkId::default(), 0, vec![]);

    // The spent UTXO carries the covenant id (set on every covenant utxo).
     let utxo_entry = UtxoEntry::new(GENESIS_AMOUNT, source_spk, 0, false, None);

    let sig_cache = Cache::new(10_000);
    let reused = SigHashReusedValuesUnsync::new();
    let flags = EngineFlags { covenants_enabled: true, ..Default::default() };

    let populated = PopulatedTransaction::new(&tx, vec![utxo_entry.clone()]);
    let cov_ctx = CovenantsContext::from_tx(&populated).expect("covenant context");
    let ctx = EngineCtx::new(&sig_cache).with_reused(&reused).with_covenants_ctx(&cov_ctx);
    let mut vm = TxScriptEngine::from_transaction_input(&populated, &tx.inputs[0], 0, &utxo_entry, ctx, flags);

    match vm.execute() {
        Ok(()) => println!("[verify] COVENANT PASSED -- rep=100 -> rep=105 spend is valid"),
        Err(e) => println!("[verify] COVENANT FAILED -- {e:?}"),
    }
}
