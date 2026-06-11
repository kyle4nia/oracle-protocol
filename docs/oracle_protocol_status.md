# Oracle Protocol — Status

_**Read `operating_baseline.md` alongside this doc.** This says WHERE the project is;
the baseline says HOW we work (interaction style, file discipline, the self-erecting lens).
Together they let a fresh chat resume cleanly._

_Last updated: TWO on-chain rep transitions confirmed (100->105->110). v4 = detached-verdict
model. OFF-CHAIN ORACLE SIGNER BUILT + PROVEN, now a STANDALONE TESTED CRATE (signer/) with
8 cargo tests (5 are rejection tests) and GREEN GitHub Actions CI on every push. Full CSFS
signing spec PINNED from txscript source. Waiting ONLY on compiler emitting OpCheckSigFromStack
(~1wk, silverscript #122) to build the on-chain gate. Build mainnet against silverscript V1.
Resume point for next session._

---

## Where we are in one sentence

The `OracleRep` reputation-UTXO covenant is LIVE ON TN10 and proven REPEATABLE. TWO real
on-chain reputation state transitions (rep=100 -> 105 -> 110, delta +5 each) were
constructed, signed, broadcast, and CONFIRMED by the node — enforced entirely at the
script layer with no trusted coordinator. The current head (rep=110) carries the
covenant_id forward, so it is itself a valid covenant UTXO ready for the next transition.
The spend client is now GENERALIZED: it reads the current rep off-chain from the live head
UTXO and applies delta, so the same tool drives every future hop with no edits. The
reputation token is self-perpetuating on-chain.

---

## Network migration: TN12 -> TN10

The Kaspa dev team directed a move from TN12 to TN10. All prior TN12 deployment
(the old genesis UTXO, the old addresses, the ~2M TKAS `oracle-dev` wallet balance)
is **dead / irrelevant** — do not reuse. TN10 is the live target.

- **Node:** runs on machine `PsychoNode` (separate box), release `tn10-toc3`,
  wRPC borsh on `0.0.0.0:17210`, reachable from dev box `big daddy` at
  `192.168.0.206:17210`. Static IP, firewall open, syncing.
- **Dev box (`big daddy`):** holds the source of truth (`C:\oracle-protocol`),
  `silverscript` (`C:\kaspa-tn12\silverscript`), `protoc`, and a fresh
  `rusty-kaspa` checkout pinned to tag `tn10-toc3`
  (`C:\kaspa-tn12\rusty-kaspa`) for the example harnesses.

---

## What is DONE and proven

- **The covenant is correct by construction.** `oracle_rep_v3.sil` uses the
  declarative `#[covenant(binding = auth, from = 1, to = 1, mode = transition)]`
  pattern. The current `silverc` lowers this to `OpAuthOutputCount` /
  `OpAuthOutputIdx` / `validateOutputState` — NOT the old fragile
  `activeScriptPubKey.split(9).1` reconstruction. This is the P2SH-safe path.
- **Geometry:** `byte[8] repState`, `state_layout { start: 0, len: 9 }`
  (8 state bytes + 1-byte OpData8 length prefix). Little-endian; rep value sits
  in byte index 1 of the redeem script.
- **Compiles clean** via `silverc` against both rep=100 and rep=105 ctors.
- **Local VM verification PASSES** (`oracle_spend_verify` -> "COVENANT PASSED").
  Runs through `TxScriptEngine` with `covenants_enabled: true` and a
  `CovenantsContext`, exactly matching the `tn10-toc3` consensus engine.

---

## Current ground-truth artifacts (TN10)

### v3 redeem script — 68 bytes, rep=100
```
8,100,0,0,0,0,0,0,0,118,82,121,147,118,0,162,105,185,203,81,156,105,118,88,205,1,8,124,126,185,118,201,118,1,68,148,89,147,124,188,126,170,2,0,0,1,170,126,1,32,126,124,126,1,135,126,185,0,204,195,135,105,0,122,117,117,117,81
```
rep=105 is identical except byte index 1 = 105.

### Addresses (regenerated from the 68-byte script)
| Item | Value |
|------|-------|
| v3 rep=100 genesis addr | `kaspatest:pr64eayfczjzmrkk4s68cmt7kr7r7ejrx54sfvqvw86krlpst6u47xznrlg23` |
| v3 rep=105 target addr  | `kaspatest:prdtal22v0mwz6l0gn3rl6krhpzx0q5l5qa2n2k3unrguxfr903egtn9c7y0s` |

### ABI (from silverc)
- entrypoint: `__covenant_entrypoint_auth_update`
- inputs: `delta : int` (single call arg)
- `without_selector: true` (no function selector prefix)
- `state_layout: { start: 0, len: 9 }`

### DEAD — do not reuse (old TN12, old 53-byte split-based contract)
- Old 53-byte script and any addresses derived from it
  (`kaspatest:prj3...`, `kaspatest:ppw0...`) are stale.
- The old `entrypoint`/`split` version of `oracle_rep_v3.sil` has been
  overwritten with the source-of-truth declarative version.

---

## Resolved bug (for the record)

The blocker was a **stale-file trap**, not unsolved logic. Two copies of
`oracle_rep_v3.sil` had diverged: the broken manual `split`-based version lived
in `silverscript-lang\tests\examples\`, while the corrected declarative version
sat in `oracle-protocol\covenants\`. The harness, ctors, and recorded bytes all
referenced the old file. Fix was: compile the correct file, regenerate ctors
(`oracle_ctor_105.json` was still old `int` format -> updated to `byte[8]` array),
update the harness + derive scripts to the new 68-byte script and `delta`-only
ABI, and re-verify.

---

## ACHIEVED — TWO on-chain rep transitions, repeatability proven (DONE)

The full cycle is live on TN10 and proven repeatable. Three transactions, all confirmed:

1. **Genesis-establishing tx** `1a5a825d2983d3d84bf8e015d8ae912936062be59ad59c81414275b732c5abed`
   - Spent a plain key-controlled P2PK UTXO (1000 TKAS), created the rep=100
     covenant UTXO at the P2SH address, bound via `populate_genesis_covenants`.
   - Result: a UTXO carrying covenant_id, NOT covenant_id:None.
2. **Update tx (rep 100 -> 105)** `d59550a2c83e4f215947a2ce38633ab3a4742e58e9a825a9d9da14d7cdd9cb21`
   - Spent the rep=100 covenant UTXO (continuation), produced the rep=105 UTXO
     carrying the SAME covenant_id forward. Covenant enforced at script layer.
3. **Update tx (rep 105 -> 110)** `fff850538fdf762e959c881531b299b8a3a934260d6d9acbe7ef2bdf642de00f`
   - Spent the rep=105 covenant UTXO via the GENERALIZED spender (read current rep
     off-chain, applied delta), produced the rep=110 UTXO carrying the same
     covenant_id. Confirmed three ways: node `ACCEPTED`, generalized re-read, and
     a direct `get-utxos-by-addresses` query. Proves repeatability beyond hop one.

### Current on-chain state (TN10)
- **Active covenant_id:** `ba37ac5a713c4f2e7429ba17d4d338d117e1130556e42201181c22552e3102e2`
- **Current head UTXO (rep=110):** `fff850538fdf762e959c881531b299b8a3a934260d6d9acbe7ef2bdf642de00f:0`
  - value 99999400000 sompi, covenant_id = active id above.
  - spk: `aa20555bab71a95074478758d65cc35523c2d9038a069ecf97251de8ffd8b978c57787`
  - block_daa_score 487754937.
  - This is itself a valid covenant UTXO — spend it next for 110 -> 115, etc.
- **rep=110 head address:** `kaspatest:pp24h2m349g8g3u8trt9es64y0pdjqu2q60vl9e9rh50lk9e0rzhwhlup7xms`
- Funding wallet: oracle-dev (kaspa-cli), account oracle-dev1, ~98,999 TKAS left.
- Fresh keypair used for genesis funding input (hex key held locally; TN10 test key).
- DEAD: old faucet UTXO `a20732...3761:0` (covenant_id:None) — covenant can't spend it; ignore.

### THE GENESIS LESSON (the crux — do not forget)
A covenant UTXO CANNOT be created by sending funds directly to its P2SH address.
That produces a UTXO with covenant_id:None, which the covenant can never spend
(genesis outputs do not populate auth contexts, so OpAuthOutputCount returns 0 and
`require(auth_out_count == 1)` fails -> "script ran, but verification failed").
A covenant UTXO MUST be ESTABLISHED as a bound output of a separate signed tx:
spend a plain key-controlled input, create the covenant output, and bind it with
`populate_genesis_covenants(&[GenesisCovenantGroup::new(auth_input, vec![out_idx])])`.
Only then does it carry a covenant_id and become spendable by the covenant's
continuation logic. Model is TWO-STEP: establish, then transition.

## NEXT SESSION — option 2: authorize the delta (resume here)

The primitive is proven AND repeatable. Option 1 (smoke test / generalize) is DONE.
Next is building the actual Oracle Protocol on top of it.
Candidate directions (pick per launch priorities; mainnet target):
1. ~~**Next transition (smoke test) / generalize the spender.**~~ DONE. Spender reads
   current rep off the live head UTXO and applies delta; deriver is parameterized
   (`derive_v3 -- <rep>`). 100->105->110 confirmed. No per-hop hardcoding remains.
2. **Real delta authorization (v4 — RESHAPED, BLOCKED on checkDataSig):** v3 delta
   is unauthenticated — anyone who sees the chain can spend the head and move rep.
   v4 adds oracle authorization. DECISION: the admin-checkSig approach was ABANDONED
   (wrong model); v4 is now the detached-verdict (checkDataSig) model. BLOCKED pending
   upstream checkDataSig implementation; dev-team outreach in flight.

   ### RESOLVED BY DEVS — checkDataSig is being replaced by OpCheckSigFromStack
   checkDataSig confirmed a NO-OP STUB (compile_checkdatasig_call: pushes args,
   OpDrops all, pushes OpTrue — verifies nothing; confirmed in compile.rs source).
   DEV ANSWER (silverscript maintainer Ori, via Telegram + Discord):
   - The checkDataSig opcode "should be deleted and replaced by checksigfromstack",
     "in the coming week." Tracked in silverscript ISSUE #122.
   - OpCheckSigFromStack (CSFS) is the REAL primitive for the detached-verdict model:
     verifies a signature over arbitrary DATA (a message) pushed on the stack, vs.
     checkSig which verifies over the spending tx. CSFS = exactly what "Everyone is an
     Oracle" / portable verdicts need.
   So v4 is unblocked on a ~1 week horizon. Build the gate against CSFS, NOT checkDataSig.

   ### checkSig and checkMultiSig status (from reading compile.rs)
   - checkSig: REAL (compile_checksig_call emits OpCheckSig 0xac). Verifies the tx.
   - checkMultiSig: ABSENT — not handled in compile_call_expr at all; falls through to
     "unknown function call" error. Do NOT plan the quorum extension on it yet.
   - OpInputCovenantId / OpOutputCovenantId: REAL (emit real opcodes). So the script
     CAN read its own covenant_id -> covenant_id-in-signed-message IS feasible (resolves
     a v4 open design question: YES, bind the verdict to covenant_id).

   ### SECOND DEV FINDING — entrypoint fixed-length args are NOT enforced (affects v3!)
   A `byte[N]` entrypoint arg compiles to `N N OpNumEqual` (e.g. `4 4 ==`) — a NO-OP
   that does NOT check the actual pushed length. Maintainers confirmed: under/over-length
   args are not validated at the entrypoint boundary. Michael Sutton: "the entrypoint
   boundary is expected to be checked correctly" — fix (validate all entrypoint input
   lengths at script start) is MANDATORY PRE-V1.
   - IMPACT ON LIVE v3: v3's entrypoint takes `delta:int`. A malformed-length delta may
     not be rejected by the length check (there isn't a real one). Likely benign (the
     arithmetic opcodes may reject a bad int), but per Ori "not hermetic enough" — DO NOT
     assume benign. Re-verify v3 against the v1 compiler once the fix lands.
   - This is why: do NOT carry v3 (or v4) to MAINNET on the current pre-stable compiler.
     Build the mainnet covenant against silverscript V1 (gates BOTH the CSFS replacement
     and this length-validation fix as mandatory-pre-release). Ori: official v1 "soon."

   ### OPTIONAL workaround (TN10 only, NOT mainnet) — bytecode splice
   Community (IzioDev) suggested: compile the contract, then post-process the compiled
   script to splice in a push + OpCheckSigFromStack where the checkDataSig stub emitted
   its no-op. Works because the script is just bytes. RISK: hand-assembling consensus-
   critical signature-verification bytecode — a wrong byte = fails open (fake gate, the
   exact danger checkDataSig posed). If used: ONLY on TN10 to prove the verdict model
   end-to-end before CSFS lands, and the REJECTION TEST (bad/missing sig MUST fail) is
   absolutely non-negotiable. Not for mainnet — re-do against compiler-emitted CSFS in v1.

   ### PRODUCTIVE WORK WHILE WAITING (does NOT depend on the compiler)
   - [DONE] CSFS signing spec PINNED from txscript source (tn10-toc3):
     * OpCheckSigFromStack<0xd7> EXISTS in the engine (schnorr); 0xd8 = ECDSA variant.
       Only the COMPILER emit is missing (#122). So the splice workaround WOULD verify
       on-chain on TN10 if ever needed.
     * Opcode pops [signature, msg_hash, pubkey]. msg_hash MUST be exactly 32 bytes.
     * check_schnorr_signature_for_msg_hash uses secp256k1::Message::from_digest(msg_hash)
       DIRECTLY — plain BIP340 schnorr, x-only 32B pubkey, 64B sig, NO domain separation /
       tagging. ZERO_SIG => false.
     * OpBlake2b<0xaa> = Params::new().hash_length(32), unkeyed (plain blake2b-256).
   - [DONE] VERDICT MESSAGE LAYOUT FINALIZED (full binding):
       verdict_bytes = delta(8B LE) || nonce(8B LE) || covenant_id(32B) = 48 bytes
       msg_hash      = blake2b_256(verdict_bytes)
       signature     = schnorr_sign(msg_hash) with oracle x-only key
     Both on-chain and off-chain must build verdict_bytes byte-for-byte identically.
   - [DONE] OFF-CHAIN ORACLE SIGNER BUILT + PROVEN: oracle_sign_verdict.rs (rothschild bin;
     source of truth in oracle-protocol/rust-examples). Takes --delta --nonce --covenant-id
     [--oracle-key]; builds verdict_bytes, blake2b-256, schnorr-signs the 32B digest; outputs
     oracle_pubkey(32) + oracle_pkh(32, for ctor) + signature(64). SELF-TESTS pass BOTH ways:
     valid sig verifies under the node's exact schnorr check, AND a tampered verdict is
     rejected. The off-chain half of the verdict model is DONE and proven a week before the
     on-chain gate can exist. (Added blake2b_simd dep to rothschild via cargo add.)
   - [DONE] SIGNER PROMOTED TO A STANDALONE TESTED CRATE: `signer/` in the repo
     (crate `oracle-signer`). Layout: src/lib.rs (pure signing logic), src/main.rs (CLI),
     tests/verdict_tests.rs (8 cargo tests). Depends only on PUBLISHED crates (secp256k1,
     blake2b_simd, faster-hex, rand) — NO rusty-kaspa dependency, so it builds standalone in
     minutes. 8 tests, FIVE of them rejection tests (tampered delta / nonce / covenant_id /
     wrong-pubkey all MUST fail) — this is the accept-AND-reject coverage that makes the
     security claim real, not hopeful. `cargo test` green locally AND on CI.
   - [DONE] GITHUB ACTIONS CI: `.github/workflows/ci.yml` runs cargo build + test on every
     push (working-directory: signer). GREEN on github.com/kyle4nia/oracle-protocol. fmt +
     clippy run as non-fatal (continue-on-error). This is the public, automated proof the
     signer builds and its tests pass — the credibility signal. (Note: Node20 deprecation
     warning on checkout/cache actions is harmless; update to Node24-compatible versions
     whenever. Optional polish: add a CI status badge to README.md.)
   - [ ] Spender sigScript push order (confirmed): signature(64), msg_hash(32), oracle_pubkey(32).
   - [ ] WHEN CSFS LANDS in the compiler: build the v4 contract gate —
       msg_hash = blake2b(delta||nonce||covenant_id);
       require(OpCheckSigFromStack(oracle_sig, msg_hash, oracle_pk));
       plus identity require(blake2b(oracle_pk)==oracle_pkh).
     covenant_id read via real OpInputCovenantId. MUST VERIFY: OpInputCovenantId byte order
     matches the order the signer used for covenant_id (signer takes it as hex as-is).
   - [ ] Generate a FRESH oracle key for real v4 (don't reuse the demo-run key).

   ### WATCH FOR (add to check-updates.bat monitoring)
   - silverscript commit wiring OpCheckSigFromStack into the compiler (closes issue #122).
   - silverscript entrypoint-length-validation fix (mandatory-pre-v1).
   - silverscript official V1 release tag — the version to build mainnet against.

   ### WHY admin-checkSig was abandoned (the model decision)
   checkSig authorizes "THIS TX" — the signer must BE the submitter. That is the
   opposite of what the protocol needs. The "Everyone is an Oracle" / detached-verdict
   model requires authorizing "THIS VALUE": an oracle signs a portable verdict
   ("apply delta +5"), and ANYONE can carry it on-chain. That portable-artifact
   property is exactly what checkDataSig provides (sign arbitrary data, not the tx)
   and checkSig does not. So single-admin-checkSig isn't a stepping stone toward the
   real model — it's a different model. Building it out would be sunk cost. The admin
   contract is preserved as a reference for the covenant PLUMBING only:
   `oracle_rep_v4_admin_checksig_ARCHIVED.sil`.

   ### v4 current contract (oracle_rep_v4.sil) — DETACHED VERDICT, NON-DEPLOYABLE
   - `contract OracleRep(byte[8] init_rep, byte[8] init_nonce, byte[32] oracle_pkh)`.
   - update() takes `(State prev_state, int delta, pubkey oracle_pk, sig oracle_sig)`,
     requires `blake2b(oracle_pk)==oracle_pkh`, then the GATE:
     `require(checkDataSig(oracle_sig, verdict_msg, oracle_pk))` where
     verdict_msg = delta || nonce. Applies delta, increments nonce.
   - NONCE added to state (anti-replay): a signed verdict is bound to a state-step
     so the same signature can't be replayed for every future hop. This CHANGES
     state geometry: now byte[8] rep + byte[8] nonce = 16 bytes (v3 was 8). Ripples
     into redeem script, addresses, spender — handle when un-blocking.
   - Keeps v3 covenant plumbing (binding=auth, OpAuthOutputCount/Idx, validateOutputState).
   - **NON-DEPLOYABLE**: checkDataSig is a stub, so the gate enforces nothing. This
     file is a DESIGN TARGET, not a deployable script. Do NOT establish a genesis on it.

   ### OPEN DESIGN QUESTIONS (resolve WITH the dev team, not by guessing)
   - Exact message encoding checkDataSig will expect (the `delta || nonce` concat
     syntax `byte[8](delta) + nonceState` is a GUESS; byte-array concat semantics
     unverified).
   - Should covenant_id be IN the signed message? (Binds a verdict to THIS covenant
     so it can't be replayed on another. Needs OpInputCovenantId wiring — not yet done.)
   - Nonce mechanics / replay model — confirm the chosen approach is sound.

   ### v4 RESUME (when un-blocked)
   - [ ] Dev-team answer received on checkDataSig (real? when? message-format API?).
     SPECIFIC QUESTION ASKED: "we're building a portable-verdict covenant where an
     oracle signs (delta || nonce) and anyone submits; does/will checkDataSig verify
     arbitrary-message signatures for this, and what's the expected message encoding?"
   - [ ] Refine oracle_rep_v4.sil against the REAL checkDataSig API (message format,
     covenant_id binding).
   - [ ] Generate real oracle keypair; hash pubkey (blake2b) -> oracle_pkh ctor arg.
   - [ ] Update oracle_spend_verify for the new ABI. MUST test BOTH: valid verdict
     ACCEPTS and bad/missing/replayed verdict REJECTS. The rejection test is the only
     real proof the gate works — non-negotiable given checkDataSig's stub history.
   - [ ] Only after accept+reject both proven locally: establish a FRESH genesis on
     the v4 script (new address; different covenant from the live v3 chain). The live
     rep=110 v3 chain stays as the keyless PoC.
   - [ ] Spender signs the verdict data off-chain, pushes oracle_pk + oracle_sig into
     the sigScript.
3. **Constitutional constraints:** encode the prime-directive / bill-of-rights bounds
   from Revolution 2nd Ed. into the covenant (caps, floors, rate limits via DAA).
4. **Multiple oracles / layered reputation:** more than one covenant, geographic
   layering, juries. (Generalizes the single oracle_pkh to a set / quorum — depends
   on checkMultiSig being real, verify first.)


## Tooling for on-chain ops (in rothschild/src/bin, source of truth in oracle-protocol/rust-examples)
- oracle_submit_read.rs       : STALE — still the old TN12 probe. Prints DEAD values
                                (`a20732...3761`, covenant_id `99b3c024...`) because it
                                has the old genesis addr hardcoded. Either generalize it
                                like the spender (scan for live head, report current rep)
                                or retire it. Do NOT trust its output as-is.
- oracle_genesis_establish.rs : create a covenant genesis UTXO from a plain input (THE establish step)
- oracle_sign_verdict.rs      : OFF-CHAIN ORACLE SIGNER (v4). Builds verdict_bytes
                                (delta||nonce||covenant_id, 48B), blake2b-256, schnorr-signs
                                the 32B digest. Outputs pubkey(32)+pkh(32)+sig(64). Self-tests
                                valid-accept and tamper-reject against the node's exact schnorr
                                primitives. Proven; the on-chain CSFS gate is the only missing half.
- oracle_submit_spend.rs      : GENERALIZED continuation spend. Scans rep window for the
                                live covenant-carrying head, reads current rep (8-byte LE)
                                off its redeem script, applies DELTA, builds next-rep
                                continuation output carrying covenant_id forward. Same file
                                drives EVERY hop — no per-rep editing. (START_REP=100,
                                MAX_HOPS=200, DELTA=5 as consts; widen MAX_HOPS if needed.)
- derive_v3.rs (example)      : PARAMETERIZED address deriver. `cargo run --release
                                --example derive_v3 -- <rep>` prints the v3 P2SH address
                                for any rep. Replaces per-rep derive_v3_100/_105/_110.
Build/run pattern: `cargo run --release -p rothschild --bin <name>` (add `-- --go` to broadcast; dry-run without).
Examples (derive_v3 etc.) live in `crypto\txscript\examples\`; run with `--example <name>`.
Node gRPC for these clients: 192.168.0.206:16210 (PsychoNode; --rpclisten added + firewall "Kaspa TN10 gRPC").
FEE floor learned: ~167,100 sompi for these txs; use 200,000.

## SEPARATE OPEN TASK — mainnet Toccata activation (karrrlskaspanode)

rusty-kaspa `v2.0.0` is tagged on `origin/stable` and commit #1044 "Set Toccata
to activate on mainnet" is merged. The mainnet bridge box (karrrlskaspanode)
must run a build that includes the mainnet Toccata activation before the fork
date, or it will fork off mainnet at activation (same risk avoided on TN10 by
running tn10-toc3). NOT URGENT (hardforks have weeks of lead time), but verify:
does the KAS_TOC2 bundle / current mainnet kaspad include #1044, or grab v2.0.0?
Check the activation DAA/date set in #1044.

## TOOLING (built, in C:\oracle-protocol\docs)
- check-updates.bat  — pre-session upstream check (silverscript commits +
  rusty-kaspa tags + master commits). Section 3 can carry network-critical news
  (e.g. it surfaced the mainnet Toccata activation), not just dev noise.
- covenant-check.bat — recompile + diff vs covenant_baseline.txt
  (IDENTICAL = safe / DIFFERENT = investigate). `rebaseline` arg to reset.
- deploy-and-run.bat <example> — copy harness to build dir + cargo run.
- QUICKREF.md — paste-ready addresses, outpoint, ABI, script, paths.
- COMMANDS.md — full command reference + staying-current + bridge setup.

---

## Working command reference (corrected)

```powershell
# Compile covenant (from silverscript dir). Source of truth .sil is in oracle-protocol.
cargo run --bin silverc -- C:\oracle-protocol\covenants\oracle_rep_v3.sil --ctor C:\kaspa-tn12\silverscript\oracle_ctor_v3.json -c

# rep=105 ctor: oracle_ctor_105.json (now byte[8] array form, first byte 105)

# Local VM spend verification (from rusty-kaspa dir, tag tn10-toc3)
cargo run --release --example oracle_spend_verify

# Derive addresses from the 68-byte script
cargo run --release --example derive_v3_100
cargo run --release --example derive_v3_105
```

## Hard-won gotchas
- Source of truth is `C:\oracle-protocol`. The Kaspa clones are disposable;
  re-clone rusty-kaspa on tag `tn10-toc3` and copy `rust-examples\*.rs` into
  `crypto\txscript\examples\` to build the harnesses.
- **ALWAYS-SAVE RULE:** every new `.rs` tool/harness gets saved to source-of-truth
  `C:\oracle-protocol\rust-examples\` immediately, in addition to the disposable
  build/clone location (rothschild\src\bin or crypto\txscript\examples). The
  oracle-protocol copy is canonical; the clone copy is throwaway. Do this the
  moment a new .rs is created, not later.
- KEEP THE TWO `.sil` COPIES IN SYNC. A divergence between the oracle-protocol
  copy and the silverscript-lang/tests copy is what caused the long stale-file
  debugging detour. Always compile the oracle-protocol copy.
- `silverc` ctor args are typed `Expr` JSON. A `byte[8]` is an `array` of eight
  `{"kind":"byte","data":N}` entries (little-endian), NOT `{"kind":"int",...}`.
- VS Code integrated terminal reverses multi-line pastes. Use single-line
  commands, or paste multi-line content into the editor, not the terminal.
- Kaspa ints in script are little-endian; `byte[8](100)` => `64 00 00 00 00 00 00 00`.
- Run node and dev work on separate machines; point tooling at
  `192.168.0.206:17210` (PsychoNode), not localhost.
