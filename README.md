# Oracle Protocol

[![CI](https://github.com/kyle4nia/oracle-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/kyle4nia/oracle-protocol/actions/workflows/ci.yml)

Consensus-enforced reputation on Kaspa. Reputation lives as state inside covenant-locked
UTXOs and is updated at the script layer. Each transition produces a new UTXO carrying the
covenant forward, spendable only under the same rules.

The target authorization model is the portable verdict: an oracle signs an authorized change
(`delta || nonce || covenant_id`), and anyone can carry that signature on-chain. The signer
must not need to be the submitter.

## Self-erecting

The design constraint on every component: the system adjusts toward its goals without a
trusted coordinator. A reputation system with a privileged operator hasn't removed the trust
problem, it has relocated it.

Concretely:

- Transition rules are enforced by the covenant, not applied by an operator. The rules travel
  with the UTXO; the deployer holds no ongoing role.
- Authorization and submission are separate. An oracle signs a verdict; any party can carry
  it on-chain. No relayer, no sequencer, no allowlist.
- The live chain proves it: the current rep head is a plain UTXO on TN10, spendable by anyone
  who satisfies the covenant. If every machine that built this project went dark, the state
  and its rules would remain fully in force.
- Where the implementation still leans on a coordinator-shaped crutch, that is named, not
  hidden: head discovery currently scans a node's address index, and the proposed
  `getUtxosByCovenantId` RPC exists to remove it. Testnet shortcuts are recorded alongside
  what the self-erecting version replaces them with.

## Update model

Reputation converges; it never jumps. Each transition is Bayesian in shape: the current
state is the prior, a signed verdict is the evidence, the new head UTXO is the posterior.
Because every head is the product of the full transition chain, no update can discard
history — it can only condition on it.

- **Small deltas, high frequency.** Kaspa's block rate (10+ bps) makes many small updates
  the natural granularity, not one monolithic recomputation. The live v3 chain already moves
  in ±5 steps. Design target: covenant-enforced delta caps and DAA-based rate limits, so a
  single verdict is bounded by consensus, not by oracle restraint. (Caps are roadmap, not yet
  in v3/v4 — today the bound is convention.)
- **Decisions stay cheap.** When no single update can move state far, no single decision
  carries much risk. A wrong verdict is corrected by subsequent evidence, the same way it
  arrived — there is no rollback, and none is needed.
- **Claims are interrogated, not trusted.** A verdict binds every field under one signature;
  nonce prevents replay across steps, covenant_id prevents replay across covenants; the
  signer's rejection tests prove each binding fails when tampered. Today one oracle key
  gates a stream; the roadmap generalizes to quorums and juries, where a claim must survive
  challenge rather than arrive by decree.
- **Scope is part of the state.** A reputation stream measures one declared thing. Evidence
  binds to its covenant_id, so a verdict earned in one topic cannot move another, and a
  score cannot silently drift into meaning something it never measured. Cross-topic
  composition happens by reading interfaces (see Ecosystem alignment), never by blending
  streams.

## Status

| Component | State |
|---|---|
| v3 covenant — reputation as UTXO state | Live on TN10. Genesis established, on-chain transitions confirmed (rep 100 → 105 → 110), enforced at the script layer. |
| Off-chain verdict signer | Done. Standalone crate (`signer/`), 8 tests (5 rejection), CI on every push. |
| v4 on-chain gate | In progress. Unblocked 2026-06-16 when the SilverScript compiler shipped a real `checkSigFromStack` (silverscript #132); contract gate and accept/reject harness are the current work. |
| Mainnet | Gated on a SilverScript v1 stable release and independent review. |

## The signer

```bash
cd signer
cargo test
```

Eight tests, five of them rejection tests: a tampered delta, nonce, or covenant_id, or a
wrong pubkey, must all fail verification.

The signing format matches the engine's `OpCheckSigFromStack` exactly (pinned from txscript
source): blake2b-256 over the 48-byte verdict, BIP340 schnorr over the 32-byte digest.

```bash
cd signer
cargo run --bin oracle_sign_verdict -- --delta 5 --nonce 0 --covenant-id <64-hex covenant id>
```

## Relationship to BitPhoque

[BitPhoque](https://github.com/kyle4nia/BitPhoque) is this project's proving ground: a
covenant-based fungible token live on TN10 with a working non-custodial browser wallet
([bitphoque.org](https://bitphoque.org)). Supply conservation, leader/delegate multi-input
transfers, sale/claim flows, fee and mass behavior, genesis establishment — all proven there
in production covenant code before being relied on here. Patterns flow both ways; findings
from both projects are reported upstream.

## Ecosystem alignment

State layouts are being designed toward the emerging KCC20 / Capsule direction: standard
fields plus a virtual-extension digest, so consumer covenants can compile against the
oracle's interface without knowing its internals (Open ICC). The `templateHash` builtin
(silverscript #143) provides the commitment-opening primitive.

Proposed RPC `getUtxosByCovenantId` would remove the last indexer-shaped dependency for both
projects (parcel discovery in BitPhoque, live-head scan here).

Production findings from both projects are posted to the kas-smith forum; issues and
reproductions are filed against kaspanet/silverscript as they surface.

Reviews, reproductions, and pull requests from people who work at this layer are welcome.

## Layout

```
covenants/        SilverScript covenant sources (v3 live; v4 = detached-verdict gate)
signer/           Off-chain oracle verdict signer (standalone crate + CI)
rust-examples/    On-chain tooling (genesis establish, continuation spender, derivers)
docs/             Canonical status doc, command reference, helper scripts
```

Canonical project state: `docs/oracle_protocol_status.md`.

## Built on

- [Kaspa](https://kaspa.org) — Toccata covenants, Testnet 10.
- [SilverScript](https://github.com/kaspanet/silverscript) — the covenant language.

## License

MIT. See [LICENSE](LICENSE).
