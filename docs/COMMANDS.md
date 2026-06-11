# Oracle Protocol — Command Cheat-Sheet

Quick reference for recurring commands. **Read the two rules below first — they
prevent the most common mistakes.**

---

## Two rules that prevent 90% of confusion

1. **Which machine?** Node commands run on **PsychoNode**. Everything else
   (compile, verify, CLI, edits) runs on **big daddy**. All tooling points at
   the node over the network: `192.168.0.206:17210` — never localhost.

2. **cmd vs. the `$` shell.** Some commands run in a normal Windows terminal
   (cmd / PowerShell). Others run *inside* the kaspa-cli interactive shell at
   the `$` prompt. They are NOT interchangeable:
   - cmd only: `cargo ...`, `copy`, `cd`, `kaspad.exe`, `Test-NetConnection`
   - `$` shell only: `network`, `connect`, `ping`, `rpc ...`, `exit`

Also: **single-line commands only** — the VS Code terminal reverses multi-line
pastes. And **edit in `C:\oracle-protocol` (source of truth), then copy into the
rusty-kaspa clone to build** — never edit the clone directly (that drift caused
a long debugging detour once).

---

## Node — on PsychoNode (Windows terminal)

Start the node (saved as `start-tn10.bat` in `C:\kaspatestnet10`):
```
cd /d C:\kaspatestnet10 && kaspad.exe --testnet --netsuffix=10 --utxoindex --disable-upnp --rpclisten-borsh=0.0.0.0:17210
```

Check version:
```
cd /d C:\kaspatestnet10 && kaspad.exe --version
```

---

## Connectivity test — on big daddy (PowerShell)

```
Test-NetConnection -ComputerName 192.168.0.206 -Port 17210
```
Want `TcpTestSucceeded : True`.

---

## kaspa-cli — on big daddy

Launch the interactive shell (cmd):
```
cd /d C:\kaspa-tn12\rusty-kaspa && cargo run --release --bin kaspa-cli
```

Then at the `$` prompt (inside the shell):
```
network testnet-10
connect 192.168.0.206:17210
ping
```

Check sync progress (watch `block_count` rise from 0 and `virtual_daa_score`
climb toward the live network DAA):
```
rpc get-block-dag-info
```

Check a covenant address for funds (only meaningful once synced):
```
rpc get-utxos-by-addresses <address>
```

Live-watch while waiting for a deposit:
```
track utxo
```

Leave the shell:
```
exit
```

If `connect` is rejected, try `connect ws://192.168.0.206:17210`, or
`server 192.168.0.206:17210` then `connect`.

---

## Compile the covenant — on big daddy (from silverscript dir)

rep=100:
```
cd /d C:\kaspa-tn12\silverscript && cargo run --bin silverc -- C:\oracle-protocol\covenants\oracle_rep_v3.sil --ctor C:\kaspa-tn12\silverscript\oracle_ctor_v3.json -c
```

rep=105:
```
cd /d C:\kaspa-tn12\silverscript && cargo run --bin silverc -- C:\oracle-protocol\covenants\oracle_rep_v3.sil --ctor C:\kaspa-tn12\silverscript\oracle_ctor_105.json -c
```

---

## Local verify + derive addresses — on big daddy (from rusty-kaspa dir)

Verify the rep=100 -> rep=105 spend (want "COVENANT PASSED"):
```
cd /d C:\kaspa-tn12\rusty-kaspa && cargo run --release --example oracle_spend_verify
```

Derive addresses:
```
cd /d C:\kaspa-tn12\rusty-kaspa && cargo run --release --example derive_v3_100
cd /d C:\kaspa-tn12\rusty-kaspa && cargo run --release --example derive_v3_105
```

---

## Edit -> deploy -> run cycle — on big daddy

The most repeated workflow. After editing a harness in
`C:\oracle-protocol\rust-examples`, copy it into the clone before building:
```
copy /Y C:\oracle-protocol\rust-examples\<file>.rs C:\kaspa-tn12\rusty-kaspa\crypto\txscript\examples\<file>.rs
```
Then run with `cargo run --release --example <name>` from the rusty-kaspa dir.

---

## Re-clone rusty-kaspa (if ever needed)

The Kaspa clone is disposable. To rebuild it:
```
cd /d C:\kaspa-tn12 && git clone https://github.com/kaspanet/rusty-kaspa.git rusty-kaspa
cd /d C:\kaspa-tn12\rusty-kaspa && git checkout tn10-toc3
copy C:\oracle-protocol\rust-examples\*.rs C:\kaspa-tn12\rusty-kaspa\crypto\txscript\examples\
```

---

## Reference values

| Item | Value |
|------|-------|
| Node (borsh wRPC) | `192.168.0.206:17210` (PsychoNode) |
| rusty-kaspa tag | `tn10-toc3` |
| rep=100 genesis addr | `kaspatest:pr64eayfczjzmrkk4s68cmt7kr7r7ejrx54sfvqvw86krlpst6u47xznrlg23` |
| rep=105 target addr | `kaspatest:prdtal22v0mwz6l0gn3rl6krhpzx0q5l5qa2n2k3unrguxfr903egtn9c7y0s` |

## Key paths

| What | Path |
|------|------|
| Source of truth | `C:\oracle-protocol` |
| Covenants (.sil) | `C:\oracle-protocol\covenants` |
| Rust harnesses | `C:\oracle-protocol\rust-examples` |
| silverscript | `C:\kaspa-tn12\silverscript` |
| ctor JSONs | `C:\kaspa-tn12\silverscript\oracle_ctor_v3.json` / `oracle_ctor_105.json` |
| rusty-kaspa clone | `C:\kaspa-tn12\rusty-kaspa` |
| examples (build dir) | `C:\kaspa-tn12\rusty-kaspa\crypto\txscript\examples` |
| node binaries | `C:\kaspatestnet10` (PsychoNode) |
