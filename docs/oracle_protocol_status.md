# Oracle Protocol — Status

_**Read `operating_baseline.md` alongside this doc.** This says WHERE the project is;
the baseline says HOW we work. Together they let a fresh chat resume cleanly._

_Last updated: 2026-07-13. **CSFS UNBLOCKED** — `checkSigFromStack` landed in the compiler
(silverscript #132, 2026-06-16), source-verified as a real lowering. v4 (detached-verdict)
is ACTIVE, buildable on TN10 today. Toolchain rehomed: `C:\kaspa-dev\` (silverscript master
+ rusty-kaspa v2.0.1). Node access consolidated on `127.0.0.1:16210` (SSH tunnel to VPS
TN10, or local sync). Mainnet still gated on SilverScript V1 (zero releases)._

---

## Where we are in one sentence

The v3 reputation-UTXO covenant is LIVE ON TN10 (rep=100 -> 105 -> 110, head UTXO valid and
spendable, no trusted coordinator); the off-chain verdict signer is a tested standalone crate
with green CI; and the on-chain CSFS gate — the last missing half of v4 — became buildable on
2026-06-16 when the compiler shipped a real `checkSigFromStack`.

---

## CSFS: UNBLOCKED (the headline)

- **silverscript commit #132 (2026-06-16), "Expose typed CheckSigFromStack builtins."**
  checkDataSig's no-op stub is gone; the builtin is real.
- **Signature (from static_check.rs):**
  `checkSigFromStack(signature: datasig, digest: byte[32], publicKey: pubkey) -> bool`
  (+ `checkSigFromStackECDSA` variant with byte[33] pubkey — not needed).
- **SOURCE-VERIFIED 2026-07-13:** `compile_checksigfromstack_call` pushes signature, digest,
  publicKey and emits OpCheckSigFromStack (0xd7), net stack -2. No drop-and-true. Real.
- **Syntax reference:** `silverscript-lang/tests/examples/simple_checksigfromstack.sil`
  (renamed from simple_checkdatasig.sil).
- Matches the pinned engine spec exactly: plain BIP340 schnorr, 32B x-only pubkey, 64B sig,
  msg_hash must be exactly 32 bytes, no domain separation. The off-chain signer crate slots
  in unchanged.
- The bytecode-splice workaround is DEAD — never needed. Delete from planning.

## Compiler deltas since the 68-byte v3 baseline (recompile WILL differ — expected)

The live v3 script was compiled months of commits ago. Master has since landed:
- **#123 cov decl changes** (entrypoint prefix rename, singleton `to` bounds validation).
- **#131 / new validate_output_state.rs** — validateOutputStateWithTemplate machinery.
- **#143 template hash hardening (BREAKING, 2026-07-09)** — canonical length-bound template
  hash; adds `templateHash(prefix, suffix)` builtin for reconstructing a template hash from
  revealed parts inside a higher-level commitment. This is the Capsule/virtual-extension
  OPENING primitive, in the compiler, now.
- **#136** — compiler workspace now depends on rusty-kaspa **v2.0.1** (matches our nodes).
- README now recommends **TN10** for bytecode artifacts (TN12-only note removed, #141).

Consequences:
- `covenant-check.bat` vs the old baseline will say DIFFERENT on first run — expected, not
  an alarm. Rebaseline (`covenant-check.bat rebaseline`) after the first clean v3 recompile.
- The LIVE rep=110 chain is unaffected (its script is already on-chain). New genesis work
  (v4) is built entirely on current-master output.
- **Entrypoint length-validation fix: status UNKNOWN** — not visible in recent commit log.
  Re-verify before ANY mainnet build. The rule stands: **mainnet only against SilverScript
  V1** (still zero releases, still Experimental).

---

## Toolchain layout (rehomed 2026-07-13 — tn12 paths retired)

| Item | Location |
|------|----------|
| Source of truth | `C:\oracle-protocol` (git, github.com/kyle4nia/oracle-protocol) |
| silverscript | `C:\kaspa-dev\silverscript` (master, post-#143) |
| rusty-kaspa clone | `C:\kaspa-dev\rusty-kaspa` (tag **v2.0.1**, detached HEAD — fine) |
| ctor JSONs | `C:\oracle-protocol\covenants\oracle_ctor*.json` (rescued from compiler clone) |
| examples build dir | `C:\kaspa-dev\rusty-kaspa\crypto\txscript\examples` |
| bins build dir | `C:\kaspa-dev\rusty-kaspa\rothschild\src\bin` |
| RETIRED | `C:\_RETIRED_kaspa-tn12\rusty-kaspa` (TN12-era; holds bitphoque_cli wiring; delete only after BitPhoque CLI migration) |

Ctor triage note: FOUR ctor JSONs were rescued (`oracle_ctor.json`, `_v3`, `_105`, `_v4`).
`oracle_ctor.json` is likely the ancient TN12-era original; `oracle_ctor_v4.json` predates
the CSFS landing — BOTH need triage against the real v4 contract before use. Commit all four
at next checkpoint regardless (they were untracked project artifacts living in a disposable
clone — source-of-truth violation, now corrected).

Harnesses are NOT yet copied into the v2.0.1 clone. Expect API churn between tn10-toc3 and
v2.0.1 — budget for compile fixes on first build. Copy per COMMANDS.md recipe.

## Node access (consolidated 2026-07-13)

**Every `.rs` const NODE = `127.0.0.1:16210`** — same rule as BitPhoque. Served by either:
- **SSH tunnel to VPS TN10 node (primary):**
  `ssh -N -L 16210:127.0.0.1:16210 root@89.167.32.239`
  (VPS TN10 kaspad confirmed listening gRPC on loopback 16210, 2026-07-13. Keep the tunnel
  terminal open while running harnesses.)
- **Local Big Daddy TN10 node (fallback):** not currently running; would sync in a few hours.
  Same address when up — tunnel and local node are interchangeable, zero code difference.
- **PsychoNode: RETIRED.** `192.168.0.206` references are dead.

VPS also runs a synced mainnet node (gRPC loopback 16110) — future mainnet ops can use the
same tunnel pattern with a different port mapping.

---

## What is DONE and proven

- **v3 covenant live and repeatable on TN10.** Three confirmed txs:
  genesis `1a5a825d...abed`, 100->105 `d59550a2...cb21`, 105->110 `fff85053...e00f`.
  Enforced at script layer, no coordinator.
- **Current head (rep=110):** `fff850538fdf762e959c881531b299b8a3a934260d6d9acbe7ef2bdf642de00f:0`,
  value 99999400000 sompi, covenant_id `ba37ac5a713c4f2e7429ba17d4d338d117e1130556e42201181c22552e3102e2`,
  addr `kaspatest:pp24h2m349g8g3u8trt9es64y0pdjqu2q60vl9e9rh50lk9e0rzhwhlup7xms`.
  Valid covenant UTXO, ready for the next transition. Stays live as the keyless PoC.
- **Generalized spender** (`oracle_submit_spend.rs`): scans for live head, reads rep, applies
  delta — drives every hop with no edits.
- **Off-chain verdict signer:** standalone crate `signer/` (`oracle-signer`), 8 cargo tests
  (5 rejection), green GitHub Actions CI on every push. No rusty-kaspa dependency.
- **Verdict message layout (FINAL):**
  `verdict_bytes = delta(8B LE) || nonce(8B LE) || covenant_id(32B)` = 48 bytes;
  `msg_hash = blake2b_256(verdict_bytes)`; `signature = schnorr_sign(msg_hash)`, x-only key.
- **Spender sigScript push order (confirmed):** signature(64), msg_hash(32), oracle_pubkey(32).
- **THE GENESIS LESSON stands:** a covenant UTXO cannot be created by paying its P2SH address
  directly (covenant_id:None, unspendable). Establish via a signed tx +
  `populate_genesis_covenants`. Two-step: establish, then transition.

## v3 ground-truth artifacts

- 68-byte redeem script (rep=100; rep=105 identical except byte index 1) — recorded in
  history doc / QUICKREF. NOTE: current-master recompile produces DIFFERENT bytecode; the
  68-byte form is the on-chain historical artifact, not the build target for new work.
- ABI: entrypoint `__covenant_entrypoint_auth_update`, single arg `delta:int`,
  `without_selector: true`, `state_layout {start:0, len:9}`.
- v3 delta is UNAUTHENTICATED (anyone can move rep). That is v4's whole purpose.

---

## ACTIVE PLAN — v4 detached-verdict gate (resume here)

The "productive work while waiting" is complete; the wait is over. Sequence:

1. [ ] **Recompile sanity:** compile v3 against current master; expect covenant-check DIFFERENT;
   rebaseline. Proves the toolchain end-to-end before touching v4.
2. [ ] **Rewrite `oracle_rep_v4.sil` gate** against the real builtin:
   `require(checkSigFromStack(oracle_sig, msg_hash, oracle_pk))` where
   `msg_hash = blake2b(byte[8](delta) + nonceState + covenantIdBytes)` — byte-concat
   semantics and `digest: byte[32]` typing now checkable against the compiler, not guessed.
   Keep: `require(blake2b(oracle_pk) == oracle_pkh)` identity check; nonce increment;
   16-byte state (rep 8B + nonce 8B); v3 covenant plumbing (binding=auth).
3. [ ] **covenant_id binding:** read via OpInputCovenantId (REAL, verified in compile.rs).
   MUST VERIFY byte order matches what the signer signs (signer takes hex as-is).
4. [ ] **Triage rescued ctors;** write the real v4 ctor (init_rep, init_nonce, oracle_pkh).
5. [ ] **Fresh oracle keypair** (do NOT reuse the demo key). blake2b(pubkey) -> oracle_pkh.
6. [ ] **Local VM accept AND reject tests** (harness on v2.0.1 clone): valid verdict ACCEPTS;
   tampered delta / nonce / covenant_id / wrong key / missing sig ALL REJECT. The rejection
   tests are the proof — non-negotiable given checkDataSig's stub history.
7. [ ] **Fresh v4 genesis on TN10** (new address, new covenant; v3 chain stays as PoC).
8. [ ] **On-chain verdict transition** — the full "Everyone is an Oracle" loop, live.

Then (unchanged): constitutional constraints (caps/floors/DAA rate limits); multi-oracle /
quorum (checkMultiSig still ABSENT from the compiler — do not plan on it yet).

## Capsule / composability lens (design input for v4+, from KCC20/forum analysis)

- Design v4+ state layouts as **Capsule-compatible**: standard fields + room for a virtual-
  extension digest, so future consumer covenants can compile against the oracle's interface
  without knowing its internals (Open ICC). The `templateHash(prefix, suffix)` builtin (#143)
  is the in-compiler opening primitive for exactly this.
- **txid-commitment pattern** (forum, 143672, 2026-07-12): a spender can PROVE a prior tx's
  payload/state to a script by supplying it plus a digest of the rest, verified against
  `input.outpoint.txid == hash(...)`. Consensus can't read history, but it can verify a
  supplied claim about history. Not needed for v4; relevant to the CONSUMER layer (e.g.
  proving "oracle rep was X at DAA Y" without trusting an indexer). Filed, parked.
- **Covenant state > tx payload** as the state holder — publicly endorsed by IzioDev
  (2026-07-13); v3/v4 already comply.
- **getUtxosByCovenantId RPC:** oracle's head-scan (`oracle_submit_spend.rs`) is a second
  consumer alongside BitPhoque's parcel registry. Cite BOTH use cases in the filing
  (draft lives in BitPhoque open items).

---

## CLOSED — mainnet Toccata activation

Toccata activated on mainnet 2026-06-30. VPS mainnet node (kaspad-mainnet.service, v2.x)
confirmed synced at DAG tip 2026-07-11. The karrrlskaspanode fork-risk task is overtaken by
events; no action.

## Tooling (in C:\oracle-protocol\docs — paths swept 2026-07-13)

- check-updates.bat — pre-session upstream check (now points at kaspa-dev paths).
- covenant-check.bat — recompile + diff vs baseline; `rebaseline` arg. First run post-sweep
  WILL report DIFFERENT (see compiler deltas above).
- deploy-and-run.bat <example> — copy harness to v2.0.1 clone + cargo run.
- QUICKREF.md / COMMANDS.md — paths updated; content still references some tn10-toc3-era
  facts (fee floors etc.) — verify on first v2.0.1 runs.
- oracle_submit_read.rs — STILL STALE (old TN12 probe, dead hardcoded values). Generalize
  like the spender or retire. Do not trust output.

## Hard-won gotchas (carried forward, paths updated)

- Source of truth is `C:\oracle-protocol`. Clones are disposable. ALWAYS-SAVE RULE: every
  new `.rs` goes to `C:\oracle-protocol\rust-examples\` the moment it's created.
- KEEP THE TWO `.sil` COPIES IN SYNC (oracle-protocol copy is the one that compiles; the
  silverscript-lang/tests copy caused the stale-file detour).
- `silverc` ctor args are typed Expr JSON: `byte[8]` = array of eight
  `{"kind":"byte","data":N}` entries, little-endian. NOT `{"kind":"int",...}`.
- Kaspa script ints are little-endian.
- VS Code terminal reverses multi-line pastes — single-line commands only.
- Compile: `cd /d C:\kaspa-dev\silverscript && cargo run --bin silverc -- C:\oracle-protocol\covenants\<file>.sil --ctor C:\oracle-protocol\covenants\<ctor>.json -c`
- Harness run: `cd /d C:\kaspa-dev\rusty-kaspa && cargo run --release -p rothschild --bin <name>` (`-- --go` to broadcast; dry-run first).

## Historical note (compressed)

Project began on TN12 (dead: old 53-byte split-based contract, old addresses, old wallet
balance — never reuse). Migrated to TN10 at dev-team direction. PsychoNode hosted the TN10
node through the v3 proof; retired 2026-07. checkDataSig was confirmed a no-op stub, replaced
upstream by checkSigFromStack (#132). Full ledger in the history doc.

---

## SESSION SIGNATURES

_Append-only. Newest first._

---sig (pending) | 2026-07-13 | scope: CSFS unblock confirmed (source-verified real lowering),
tn12 path sweep (5 tooling/doc files), toolchain rehomed to C:\kaspa-dev (silverscript master
pull 2c46231->956868e; fresh rusty-kaspa v2.0.1 clone), ctor JSONs rescued to covenants\,
node access consolidated (VPS tunnel plan, PsychoNode retired), Toccata task closed, status
doc rewritten | head: (commit at checkpoint)
