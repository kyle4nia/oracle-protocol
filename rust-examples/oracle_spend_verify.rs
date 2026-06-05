use kaspa_consensus_core::hashing::sighash::SigHashReusedValuesUnsync;
use kaspa_consensus_core::subnets::SubnetworkId;
use kaspa_consensus_core::tx::{
    PopulatedTransaction, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry,
};
use kaspa_hashes::Hash;
use kaspa_txscript::caches::Cache;
use kaspa_txscript::script_builder::ScriptBuilder;
use kaspa_txscript::{pay_to_script_hash_script, EngineCtx, EngineFlags, TxScriptEngine};

const REDEEM_REP100: [u8; 53] = [
    8, 100, 0, 0, 0, 0, 0, 0, 0, 120, 147, 118, 0, 162, 105, 0, 195, 1, 8,
    82, 121, 88, 205, 126, 185, 191, 130, 89, 124, 127, 126, 170, 2, 0, 0,
    1, 170, 126, 1, 32, 126, 124, 126, 1, 135, 126, 135, 105, 0, 122, 117,
    117, 81,
];

const GENESIS_AMOUNT: u64 = 10_000_000_000;
const FEE: u64 = 10_000;
const DELTA: i64 = 5;

fn main() {
    println!("[verify] OracleRep rep=100 -> rep=105 local covenant check");

    let source_spk = pay_to_script_hash_script(&REDEEM_REP100);

    let mut redeem_rep105: Vec<u8> = Vec::new();
    redeem_rep105.push(0x08);
    redeem_rep105.extend_from_slice(&105i64.to_le_bytes());
    redeem_rep105.extend_from_slice(&REDEEM_REP100[9..]);
    let target_spk = pay_to_script_hash_script(&redeem_rep105);

  

    let sig_script = ScriptBuilder::new()
        .add_i64(DELTA)
        .unwrap()
        .add_data(&REDEEM_REP100)
        .unwrap()
        .drain();

    let outpoint = TransactionOutpoint::new(Hash::from_bytes([7u8; 32]), 0);
    let input = TransactionInput::new(outpoint, sig_script, 0, 1);

    let output = TransactionOutput::new(GENESIS_AMOUNT - FEE, target_spk.clone());

    let tx = Transaction::new(0, vec![input.clone()], vec![output.clone()], 0, SubnetworkId::default(), 0, vec![]);

    let utxo_entry = UtxoEntry::new(GENESIS_AMOUNT, source_spk.clone(), 0, false, None);

    let sig_cache = Cache::new(10_000);
    let reused = SigHashReusedValuesUnsync::new();
    let flags = EngineFlags { covenants_enabled: true, ..Default::default() };

    let populated = PopulatedTransaction::new(&tx, vec![utxo_entry.clone()]);
    let mut vm = TxScriptEngine::from_transaction_input(
        &populated,
        &input,
        0,
        &utxo_entry,
        EngineCtx::new(&sig_cache).with_reused(&reused),
        flags,
    );

    match vm.execute() {
        Ok(()) => println!("[verify] COVENANT PASSED -- rep=100 -> rep=105 spend is valid"),
        Err(e) => println!("[verify] COVENANT FAILED -- {e:?}"),
    }
}