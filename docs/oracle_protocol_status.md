# Oracle Protocol — Status

_**Read `operating_baseline.md` alongside this doc.** This says WHERE the project is;
the baseline says HOW we work._

_Last updated: 2026-07-29. **v4 CSFS GATE PROVEN** in local VM (6-case accept/reject,
all correct). Steps 1-6 of the v4 plan are DONE. Resume at step 7: fresh v4 genesis on
TN10. Mainnet still gated on SilverScript V1 (zero releases, Experimental)._

---

## Where we are in one sentence

The v3 reputation covenant is live on TN10 (keyless PoC, rep=110 head); the v4
detached-verdict gate (`checkSigFromStack`) is written, compiles against current
silverscript master, and is PROVEN to enforce in the local txscript VM — valid verdict
accepts, five tamper variants reject. Next is putting v4 on TN10 for real.

---

## v4 status: PROVEN, ready to deploy

- **Contract:** `oracle_rep_v4.sil` + `oracle_ctor_v4.json` (canonical in `covenants\`).
  Compiles clean on current master. 162-byte script, state_layout len 18 (rep 8B + nonce
  8B), `without_selector: true`, entrypoint `__covenant_entrypoint_auth_update(delta:int,
  oracle_pk:pubkey, oracle_sig:datasig)`.
- **Gate proof:** `rust-examples\oracle_v4_verify.rs` (runs as a txscript example —
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

## CRITICAL v4 design fact — continuation-only binding

`OpInputCovenantId` returns **ZERO_HASH on a genesis-establishing input** (the spent UTXO
has covenant_id=None until the covenant exists). So a verdict signed against the real
covenant_id can only be verified on a **continuation** spend, where the spent input already
carries that id. Consequence for deployment:
- **Genesis (step 7)** establishes the covenant with `oracle_pkh` baked into the ctor. It
  does NOT itself carry a signed verdict — it's the plain two-step establishment
  (signed tx + `populate_genesis_covenants`), same as v3's genesis lesson.
- **First authenticated transition (step 8)** is the spend AFTER genesis — that's the first
  hop the CSFS gate actually guards. The local proof models exactly this continuation case
  (spent UTXO carries covenant_id `5a5a…`).

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
| rusty-kaspa clone | `C:\kaspa-dev\rusty-kaspa` (tag **v2.0.1**, detached HEAD — fine) |
| covenants | `C:\oracle-protocol\covenants\` (`oracle_rep_v{3,4}.sil`, `oracle_ctor_v{3,4}.json`) |
| .rs source of truth | `C:\oracle-protocol\rust-examples\` (every new/changed harness lands here first) |
| harness build/run dir | `C:\kaspa-dev\rusty-kaspa\crypto\txscript\examples\` (run via `cargo run --example <name>`) |

**Harness home correction:** harnesses run as **txscript examples**, NOT rothschild/bin (the
old "bins build dir" note was wrong — this fresh v2.0.1 clone has no rothschild\src\bin).
txscript already has secp256k1, blake2b_simd, faster-hex, rand as regular deps, so an example
compiles with zero Cargo.toml edits.

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
- **v3 recompiles clean on current master** after a one-word fix: `binding = cov` →
  `binding = auth` in `oracle_rep_v3.sil`. Recompiled bytecode is identical to the on-chain
  v3 script; `covenant-check.bat` rebaselined. (The compiler tightened binding=cov to require
  `State[]` params; auth was always the correct binding — the ABI already said auth_update.)
- **v4 contract written + compiles + PROVEN** (see v4 section above).
- **Off-chain verdict signer:** standalone crate `signer/` (`oracle-signer`), 8 tests
  (5 rejection), green CI, no rusty-kaspa dependency.
- **Generalized spender** (`oracle_submit_spend.rs`): scans for live head, reads rep, applies
  delta — drives every hop with no edits. (Written for v3; will need v4 sigScript push order
  for authenticated hops — verify before step 8.)

## v3 ground-truth artifacts

- 68-byte redeem script (rep=100; rep=105 differs only at byte index 1) — the on-chain
  historical artifact, NOT the build target for new work (current master differs, expected).
- ABI: entrypoint `__covenant_entrypoint_auth_update`, arg `delta:int`,
  `without_selector: true`, `state_layout {start:0, len:9}`.
- v3 delta is UNAUTHENTICATED (anyone moves rep) — v4's whole purpose.

---

## ACTIVE PLAN — resume at step 7

1-6. **DONE** (recompile sanity, v4 contract, byte-order verify, ctor, oracle key, local
   accept/reject proof).

7. [ ] **Fresh v4 genesis on TN10.** New address from v4 script. Two-step establish (signed
   tx + `populate_genesis_covenants`), NO verdict (continuation-only — see critical fact).
   Requires node access (tunnel or local). Build `oracle_v4_genesis.rs` from the v3 genesis
   harness pattern.
8. [ ] **First authenticated transition on TN10** — the spend after genesis, carrying a real
   signed verdict. This is the live "Everyone is an Oracle" loop. Verify the spender's
   sigScript push order matches (delta, oracle_pk, oracle_sig, redeem).

Then toward a simple formal release (v0.1.0 tag, TN10, single oracle):
9. [ ] Repo cleanup: retire `oracle_submit_read.rs` (stale) and dead ctor `oracle_ctor.json`;
   sweep QUICKREF/COMMANDS for tn10-toc3-era facts; note the blake2b_simd dep added to
   rothschild Cargo.toml was UNNEEDED (harness moved to txscript examples) — revert or ignore.
10. [ ] README a stranger can follow: what the protocol does, how to run the signer, how to
    submit a verdict, what on-chain state means.
11. [ ] Tag the release; changelog in the tag message.

Later (unchanged): constitutional constraints (caps/floors/DAA rate limits); multi-oracle /
quorum (checkMultiSig still ABSENT from the compiler — don't plan on it yet).

## Capsule / composability lens (parked design input)

- Design v4+ state layouts Capsule-compatible (standard fields + room for a virtual-extension
  digest) so consumer covenants compile against the oracle interface without its internals.
  `templateHash(prefix, suffix)` builtin (#143) is the in-compiler opening primitive.
- **txid-commitment pattern** (forum 143672): a spender can prove a prior tx's state to a
  script via a digest verified against `input.outpoint.txid`. Relevant to the CONSUMER layer
  (proving "oracle rep was X at DAA Y" without an indexer). Parked.
- **getUtxosByCovenantId RPC filing:** oracle head-scan + BitPhoque parcel registry are two
  consumers — cite both (draft in BitPhoque open items).

---

## Hard-won gotchas (carried forward)

- **Source of truth is `C:\oracle-protocol`.** Clones are disposable. Every new `.rs` →
  `rust-examples\` the moment it's created.
- **Keep the two `.sil` copies in sync** (covenants\ is canonical; the
  silverscript-lang/tests copy caused a stale-file detour once).
- **`byte[8](int)` lowers to OpNum2Bin → `serialize_i64` = sign-magnitude little-endian.**
  Positive small values match `i64::to_le_bytes()`; negatives differ (sign bit in top byte).
  Matters if delta ever goes negative — the signer uses to_le_bytes, so re-verify parity then.
- **`silverc` ctor args are typed Expr JSON:** `byte[8]` = eight `{"kind":"byte","data":N}`
  entries, little-endian. NOT `{"kind":"int",...}`. Kaspa script ints are little-endian.
- **cmd `findstr` `\|` alternation is unreliable** — use single-term searches or `/r` regex,
  or PowerShell `Select-String`. Bit us repeatedly this session.
- **VS Code terminal reverses multi-line pastes** — single-line commands only.
- Compile: `cd /d C:\kaspa-dev\silverscript && cargo run --bin silverc -- C:\oracle-protocol\covenants\<file>.sil --ctor C:\oracle-protocol\covenants\<ctor>.json -c`
- Harness run: `cd /d C:\kaspa-dev\rusty-kaspa\crypto\txscript && cargo run --release --example <name>`

## CLOSED items

- **Mainnet Toccata activation** — activated 2026-06-30; VPS mainnet node synced. karrrlskaspanode
  fork-risk task overtaken by events. No action.
- **checkDataSig no-op stub** — dead path. Replaced upstream by checkSigFromStack (#132),
  now proven real. Bytecode-splice workaround DELETED from planning.
- **ctor triage** — `oracle_ctor.json` (TN12-era int-format) is DEAD; `_105`/`_v3` are v3
  historical; `_v4` is the live 3-param ctor.

## Historical note (compressed)

Began on TN12 (dead: 53-byte split contract, old addresses/wallet — never reuse). Migrated to
TN10 at dev-team direction. PsychoNode hosted the TN10 node through the v3 proof; retired
2026-07. Full ledger in the history doc.

---

## SESSION SIGNATURES

_Append-only. Newest first._

---sig #014 | 2026-07-29 | scope: v3 recompile fix (binding=cov→auth, bytecode identical,
rebaselined); v4 contract written against real checkSigFromStack + compiles; byte-order &
CSFS engine path source-verified; v4 ctor built with baked oracle_pkh; fresh TN10 oracle key
generated (secret gitignored); v4 CSFS gate PROVEN in local VM (oracle_v4_verify.rs, 6-case
accept/reject all correct); continuation-only covid binding discovered & documented; harness
home corrected to txscript examples; status doc condensed | head: f835147

---sig #013 | 2026-07-13 | scope: CSFS unblock confirmed (source-verified real lowering),
tn12 path sweep, toolchain rehomed to C:\kaspa-dev, ctor JSONs rescued, node access
consolidated, Toccata task closed | head: (pre-#014 baseline)
