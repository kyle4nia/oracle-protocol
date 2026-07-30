# Oracle Protocol - Status

_**Read `operating_baseline.md` alongside this doc.** This says WHERE the project is;
the baseline says HOW we work._

_Last updated: 2026-07-30. **v4 AUTHENTICATED LOOP LIVE ON TN10; v0.1.0 CLEANUP DONE.** Steps 1-10 DONE:
genesis + two authenticated hops (rep 100->105->110) + on-chain replay rejection.
Generalized spender drives every hop with no edits. Resume at step 9 (repo cleanup
toward v0.1.0). Mainnet still gated on SilverScript V1 (zero releases, Experimental)._

---

## Where we are in one sentence

The v4 detached-verdict covenant is LIVE and MOVING on TN10: two authenticated
transitions accepted (rep 100->105->110, nonce 0->1->2), one stale-nonce replay
rejected by the engine, all under covenant_id 4af6f785...; the generalized spender
(`oracle_v4_spend.rs`) scans the state trajectory and drives hops with no edits.
The "Everyone is an Oracle" loop works. Next: cleanup toward the v0.1.0 release.
(v3 keyless PoC chain also still live, untouched, rep=110 head.)

---

## v4 status: PROVEN, ready to deploy

- **Contract:** `oracle_rep_v4.sil` + `oracle_ctor_v4.json` (canonical in `covenants\`).
  Compiles clean on current master. 162-byte script, state_layout len 18 (rep 8B + nonce
  8B), `without_selector: true`, entrypoint `__covenant_entrypoint_auth_update(delta:int,
  oracle_pk:pubkey, oracle_sig:datasig)`.
- **Gate proof:** `rust-examples\oracle_v4_verify.rs` (runs as a txscript example -
  `cargo run --release --example oracle_v4_verify` from `crypto\txscript`). Six cases:
  valid ACCEPTS; tampered delta / tampered nonce / tampered covenant_id / wrong key /
  missing sig all REJECT. The five rejections are the real proof (guards against a
  stub-gate false pass). Re-run any time the compiler or engine version changes.
- **Verdict layout (FINAL, matches signer byte-for-byte):**
  `verdict_bytes = delta(8B LE) || nonce(8B LE) || covenant_id(32B)` = 48 bytes;
  `msg_hash = blake2b_256(verdict_bytes)`; `signature = schnorr_sign(msg_hash)`, x-only key.
- **Spender sigScript push order:** delta(i64), oracle_pubkey(32), oracle_sig(64), then the
  redeem-script reveal (P2SH). ABI arg order is delta, oracle_pk, oracle_sig.
- **CSFS engine check (source-verified):** OpCheckSigFromStack (0xd7) pops
  [signature, msg_hash, pubkey]; XOnlyPublicKey::from_slice(32B), Signature::from_slice(64B),
  Message::from_digest(32B msg_hash), plain BIP340 schnorr. Off-chain signer crate matches
  this path exactly.

## CRITICAL v4 design fact - continuation-only binding

`OpInputCovenantId` returns **ZERO_HASH on a genesis-establishing input** (the spent UTXO
has covenant_id=None until the covenant exists). So a verdict signed against the real
covenant_id can only be verified on a **continuation** spend, where the spent input already
carries that id. Consequence for deployment:
- **Genesis (step 7)** establishes the covenant with `oracle_pkh` baked into the ctor. It
  does NOT itself carry a signed verdict - it's the plain two-step establishment
  (signed tx + `populate_genesis_covenants`), same as v3's genesis lesson.
- **First authenticated transition (step 8)** is the spend AFTER genesis - that's the first
  hop the CSFS gate actually guards. The local proof models exactly this continuation case
  (spent UTXO carries covenant_id `5a5a...`).

## Oracle key (generated 2026-07-29)

- Fresh TN10 oracle keypair generated via `oracle_sign_verdict` (no `--oracle-key` => gen).
- `oracle_pkh` (blake2b of x-only pubkey) is baked into `oracle_ctor_v4.json`. pkh and pubkey
  are public; safe in tracked files.
- **Secret** lives ONLY in `C:\oracle-protocol\tn10.oracle-key.txt`, gitignored via
  `*.oracle-key.txt`. Never promote to a tracked file; never reuse on mainnet.
- Harness clone copy gets the secret injected at build time from that file; the CANONICAL
  `rust-examples\oracle_v4_verify.rs` keeps the placeholder `REPLACE_WITH_ORACLE_SECRET_HEX`.

---

## Toolchain layout

| Item | Location |
|------|----------|
| Source of truth | `C:\oracle-protocol` (git, github.com/kyle4nia/oracle-protocol) |
| silverscript | `C:\kaspa-dev\silverscript` (master; depends on rusty-kaspa **v2.0.1**) |
| rusty-kaspa clone | `C:\kaspa-dev\rusty-kaspa` (tag **v2.0.1**, detached HEAD - fine) |
| covenants | `C:\oracle-protocol\covenants\` (`oracle_rep_v{3,4}.sil`, `oracle_ctor_v{3,4}.json`) |
| .rs source of truth | `C:\oracle-protocol\rust-examples\` (every new/changed harness lands here first) |
| local-VM harness dir | `C:\kaspa-dev\rusty-kaspa\crypto\txscript\examples\` (pure txscript verifies, e.g. oracle_v4_verify) |
| RPC harness dir | `C:\kaspa-dev\rusty-kaspa\rothschild\examples\` (node-connected work, e.g. oracle_v4_genesis; rothschild has grpc/tokio/notify deps) |

**Harness home split (proven this session):** two paths depending on harness type.
Pure-local (no networking) harnesses go in `crypto\txscript\examples\`: txscript has
secp256k1, blake2b_simd, faster-hex, rand as regular deps (used by oracle_v4_verify).
RPC-driven harnesses go in `rothschild\examples\`: rothschild has kaspa_grpc_client,
kaspa_notify, kaspa_rpc_core, tokio as regular deps (used by oracle_v4_genesis and every
v3 harness). The earlier "txscript examples" blanket was wrong for RPC.

## Node access

**Every `.rs` const NODE = `127.0.0.1:16210`.** Served by either:
- **SSH tunnel to VPS TN10 (primary):** `ssh -N -L 16210:127.0.0.1:16210 root@89.167.32.239`
  (keep the tunnel terminal open while running harnesses).
- **Local Big Daddy TN10 node (fallback):** same address when up; interchangeable, zero code
  difference.
- VPS also runs a synced mainnet node (gRPC loopback 16110) for future mainnet ops via the
  same tunnel pattern, different port.

---

## What is DONE and proven

- **v3 covenant live on TN10** (keyless PoC). Head rep=110:
  `fff850538fdf762e959c881531b299b8a3a934260d6d9acbe7ef2bdf642de00f:0`, value 99999400000
  sompi, covenant_id `ba37ac5a713c4f2e7429ba17d4d338d117e1130556e42201181c22552e3102e2`,
  addr `kaspatest:pp24h2m349g8g3u8trt9es64y0pdjqu2q60vl9e9rh50lk9e0rzhwhlup7xms`.
  Stays live; v4 gets a fresh genesis, v3 chain untouched.
- **v3 recompiles clean on current master** after a one-word fix: `binding = cov` ->
  `binding = auth` in `oracle_rep_v3.sil`. Recompiled bytecode is identical to the on-chain
  v3 script; `covenant-check.bat` rebaselined (tool since retired in step 9). (The compiler tightened binding=cov to require
  `State[]` params; auth was always the correct binding - the ABI already said auth_update.)
- **v4 contract written + compiles + PROVEN** (see v4 section above).
- **Off-chain verdict signer:** standalone crate `signer/` (`oracle-signer`), 8 tests
  (5 rejection), green CI, no rusty-kaspa dependency.
- **Generalized v4 spender** (`oracle_v4_spend.rs`): scans trajectory, signs live-state
  verdicts, drives authenticated hops with no edits. Proven across hops 1 and 2.
- **v3 spender** (`oracle_submit_spend.rs`): superseded by the v4 spender for all new
  work; keep only as v3-chain historical tooling. Candidate for step 9 retirement.

## v3 ground-truth artifacts

- 68-byte redeem script (rep=100; rep=105 differs only at byte index 1) - the on-chain
  historical artifact, NOT the build target for new work (current master differs, expected).
- ABI: entrypoint `__covenant_entrypoint_auth_update`, arg `delta:int`,
  `without_selector: true`, `state_layout {start:0, len:9}`.
- v3 delta is UNAUTHENTICATED (anyone moves rep) - v4's whole purpose.

---

## ACTIVE PLAN - resume at step 7

1-6. **DONE** (recompile sanity, v4 contract, byte-order verify, ctor, oracle key, local
   accept/reject proof).

7. [X] **DONE.** v4 genesis established on TN10.
   - tx: c83878898cb00e529def7ca03617f8328eebe74a4ff559f670eb776de1760c2d
   - covenant_id: 4af6f785646683734025f72684f14e184a7580b6d86e2ea881306c79db8a41eb
   - addr: kaspatest:pqm9qv8z40llxcvhuqkhja5rs0knju9dg40tfak76525eg8hlgykyflm3wux3
   - head UTXO: c83878...c2d:0, value 999999800000 sompi
   - harness: `oracle_v4_genesis.rs` (canonical in rust-examples\; runs from
     rothschild\examples\, see toolchain layout).
8. [X] **DONE.** Authenticated loop proven on-chain, both directions:
   - accept #1: eba6dee9b9d77be1322afba6ce2f409ced4c38a51cbae9db04abb690144c80ee (rep 100->105, nonce 0->1)
   - replay reject: 2fe13331... (stale nonce=0 verdict vs nonce=1 state; engine "false stack entry")
   - accept #2 via generalized spender: 3fd9e2d9bc07d9cced502d5ef19fd017f65fb7e027e1b37410bbaaeeeba15133 (rep 105->110, nonce 1->2)
   - current head: 3fd9e2d...5133:0, rep=110 nonce=2, 999999400000 sompi
   - `oracle_v4_spend.rs` (canonical rust-examples\, runs from rothschild\examples\):
     scans the (rep,nonce) trajectory in one batched RPC call, signs against LIVE state,
     full 8-byte LE state writes (rep>255 safe). ASSUMES uniform delta=5 history.
   - mempool standardness: compute mass 1868 needs fee >= 186800 sompi; FEE=200000.
   - oracle key file is the labeled signer dump; spender parses hex off the
     oracle_secret line (trailing comment tolerated).

Then toward a simple formal release (v0.1.0 tag, TN10, single oracle):
9. [X] **DONE.** Repo cleanup: 25 files retired toward v4-only surface. Policy: v3 frozen
   museum piece (chain stays live, never spent/derived again). Removed all v3 tooling
   (ctors, v2/v3/compile .sil, ARCHIVED admin-checksig variant, derive_* harnesses,
   oracle_genesis_establish/spend_verify/submit_read/submit_spend), the ctor-args\ mirror
   dir, and stale docs (COMMANDS.md, QUICKREF.md, covenant-check.bat + its two .txt
   baselines, deploy-and-run.bat). check-updates.bat kept + patched (tag grep -> v2.0.1,
   help text -> v4). Committed 553dc6d. Open (clone-side, low pri): blake2b_simd dep in
   rothschild Cargo.toml was UNNEEDED - revert or ignore.
10. [X] **DONE.** README rewritten for a stranger: what the protocol does, why it exists,
    current TN10 status, web-app + self-erecting roadmap. Committed d5152f3.
11. [ ] **NEXT.** Tag v0.1.0; changelog in the tag message. Then push (local is ahead of origin/main).

Later (unchanged): constitutional constraints (caps/floors/DAA rate limits); multi-oracle /
quorum (checkMultiSig still ABSENT from the compiler - don't plan on it yet).

## Capsule / composability lens (parked design input)

- Design v4+ state layouts Capsule-compatible (standard fields + room for a virtual-extension
  digest) so consumer covenants compile against the oracle interface without its internals.
  `templateHash(prefix, suffix)` builtin (#143) is the in-compiler opening primitive.
- **txid-commitment pattern** (forum 143672): a spender can prove a prior tx's state to a
  script via a digest verified against `input.outpoint.txid`. Relevant to the CONSUMER layer
  (proving "oracle rep was X at DAA Y" without an indexer). Parked.
- **getUtxosByCovenantId RPC filing:** oracle head-scan + BitPhoque parcel registry are two
  consumers - cite both (draft in BitPhoque open items).

---

## Hard-won gotchas (carried forward)

- **Source of truth is `C:\oracle-protocol`.** Clones are disposable. Every new `.rs` ->
  `rust-examples\` the moment it's created.
- **Keep the two `.sil` copies in sync** (covenants\ is canonical; the
  silverscript-lang/tests copy caused a stale-file detour once).
- **`byte[8](int)` lowers to OpNum2Bin -> `serialize_i64` = sign-magnitude little-endian.**
  Positive small values match `i64::to_le_bytes()`; negatives differ (sign bit in top byte).
  Matters if delta ever goes negative - the signer uses to_le_bytes, so re-verify parity then.
- **`silverc` ctor args are typed Expr JSON:** `byte[8]` = eight `{"kind":"byte","data":N}`
  entries, little-endian. NOT `{"kind":"int",...}`. Kaspa script ints are little-endian.
- **cmd `findstr` `\|` alternation is unreliable** - use single-term searches or `/r` regex,
  or PowerShell `Select-String`. Bit us repeatedly this session.
- **VS Code terminal reverses multi-line pastes** - single-line commands only.
- Compile: `cd /d C:\kaspa-dev\silverscript && cargo run --bin silverc -- C:\oracle-protocol\covenants\<file>.sil --ctor C:\oracle-protocol\covenants\<ctor>.json -c`
- Local-VM harness run: `cd /d C:\kaspa-dev\rusty-kaspa\crypto\txscript && cargo run --release --example <name>`
- RPC harness run: `cd /d C:\kaspa-dev\rusty-kaspa\rothschild && cargo run --release --example <name>`
- **v2.0.1 TransactionInput API drift:** `mass: TxInputMass::...` is now `compute_commit: ComputeCommit::...`. For Toccata inputs use `ComputeCommit::ComputeBudget(10.into())`. Any harness cloned from a pre-v2.0.1 template fails to compile until swapped. Source pattern: rothschild main.rs line 704.

## CLOSED items

- **Mainnet Toccata activation** - activated 2026-06-30; VPS mainnet node synced. karrrlskaspanode
  fork-risk task overtaken by events. No action.
- **checkDataSig no-op stub** - dead path. Replaced upstream by checkSigFromStack (#132),
  now proven real. Bytecode-splice workaround DELETED from planning.
- **ctor triage** - `oracle_ctor.json` (TN12-era int-format) is DEAD; `_105`/`_v3` are v3
  historical; `_v4` is the live 3-param ctor.

## Historical note (compressed)

Began on TN12 (dead: 53-byte split contract, old addresses/wallet - never reuse). Migrated to
TN10 at dev-team direction. PsychoNode hosted the TN10 node through the v3 proof; retired
2026-07. Full ledger in the history doc.

---

## Session drift (pending next sig)

- 2026-07-29: Stray `derive_pubkey.js` (kaspa-wasm BitPhoque owner-pubkey deriver) surfaced at `C:\oracle-protocol\` root during step 9 inventory. Untracked; reads `F:\Tocatta_Projects\bitphoque\bitphoque_owner.key`. Cross-project pollution. Rehomed to `C:\bitphoque\derive_pubkey.js` via `move /Y`; sole copy on `C:`. Disposition (track / gitignore / delete) deferred to next BitPhoque session.

---

## SESSION SIGNATURES

_Append-only. Newest first._

---sig #017 | 2026-07-30 | scope: v0.1.0 RELEASED and pushed (annotated tag on b323904, public). Steps 9-11 done. Step 9: 25 stale files retired toward v4-only surface (v3 tooling, ctor-args mirror dir, COMMANDS/QUICKREF, covenant-check.bat + 2 txt baselines, deploy-and-run.bat); policy v3=frozen museum piece; check-updates.bat kept + patched to v2.0.1/v4; stray 0-byte 'cd' artifact amended out of commit 553dc6d. Step 10: README rewritten for stranger + Kyle's voice (why-it-exists, web-app/self-erecting roadmap), d5152f3. Status doc freshened to pure ASCII (em dashes/arrows/ellipsis stripped), b323904. Drift: LF-not-CRLF and UTF-8-explicit-read lessons (CP-1252 mojibake caused a false-negative char count); derive_pubkey.js cross-project pollution rehomed to C:\bitphoque. Open: no history doc in tree despite status ref | head: b323904

---sig #016 | 2026-07-29 | scope: step 8 DONE, authenticated loop live on TN10 (accept
eba6dee... rep 100->105; on-chain replay REJECT 2fe13331... stale nonce, engine false-stack;
accept 3fd9e2d... rep 105->110 via generalized spender); oracle_v4_spend.rs generalized
(trajectory scanner, one batched RPC, live-state verdicts, 8-byte LE state writes, uniform-
delta assumption flagged); fee floor gotcha (mass 1868 -> min 186800 sompi); key-file parse
fixed (labeled dump, trailing comment); v3 spender marked superseded | head: bd4da49

---sig #015 | 2026-07-29 | scope: v4 genesis established on TN10 (tx c83878..., covenant_id 4af6f785..., addr kaspatest:pqm9qv...lm3wux3, 9999.998 tKAS parked at head); harness-home drift corrected (RPC harnesses live in rothschild\examples\ not txscript\examples\; earlier note was overgeneralized from local-VM case); v2.0.1 API drift on TransactionInput (mass -> compute_commit) surfaced via compile errors and fixed via rothschild main.rs:704 pattern; status doc updated | head: 25ef286

---sig #014 | 2026-07-29 | scope: v3 recompile fix (binding=cov->auth, bytecode identical,
rebaselined); v4 contract written against real checkSigFromStack + compiles; byte-order &
CSFS engine path source-verified; v4 ctor built with baked oracle_pkh; fresh TN10 oracle key
generated (secret gitignored); v4 CSFS gate PROVEN in local VM (oracle_v4_verify.rs, 6-case
accept/reject all correct); continuation-only covid binding discovered & documented; harness
home corrected to txscript examples; status doc condensed | head: f835147

---sig #013 | 2026-07-13 | scope: CSFS unblock confirmed (source-verified real lowering),
tn12 path sweep, toolchain rehomed to C:\kaspa-dev, ctor JSONs rescued, node access
consolidated, Toccata task closed | head: (pre-#014 baseline)
