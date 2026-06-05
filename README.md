# Oracle Protocol ??? Kaspa TN12

Reputation-as-UTXO covenant system on Kaspa Testnet 12 (Toccata / SilverScript).

## Layout
- `covenants/`     ??? SilverScript (.sil) covenant sources. v3 is current.
- `ctor-args/`     ??? constructor-argument JSON for `silverc`.
- `rust-examples/` ??? Rust harnesses (address derivation, local VM spend verify).
- `docs/`          ??? status summary and resume notes.

## These files are NOT standalone-buildable
The .rs files are `cargo` *examples* that depend on the rusty-kaspa crates.
To build/run them, copy each back into:
    <rusty-kaspa>/crypto/txscript/examples/
and run e.g. `cargo run --example oracle_spend_verify`.

The .sil files compile with `silverc` from the silverscript repo:
    cargo run --bin silverc -- <path>/oracle_rep_v3.sil --ctor <path>/oracle_ctor_v3.json -c

This repo is the SOURCE OF TRUTH for the custom work; the Kaspa clones are
disposable build infrastructure that can be re-cloned at any time.

## Current status
See docs/oracle_protocol_TN12_status.md ??? covenant compiles and is funded on
TN12; open bug is the P2SH activeScriptPubKey resolution in the state-carry.
