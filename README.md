# Oracle Protocol

[![CI](https://github.com/kyle4nia/oracle-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/kyle4nia/oracle-protocol/actions/workflows/ci.yml)

**Portable, consensus-enforced reputation on Kaspa.** Reputation is stored as state
inside covenant-locked UTXOs and updated at the script layer — no trusted coordinator,
no off-chain database. The rules that govern how reputation changes are enforced by the
covenant itself, so the state is exactly as trustworthy as the chain it lives on.

This is a working research project, not a finished product. What is proven is proven on
**Kaspa Testnet 10 (TN10)**; what is in progress is marked as such below. It is **not on
mainnet and not audited** — see Status.

---

## Why this exists

Reputation systems usually depend on whoever runs the database. This one doesn't. By
encoding reputation as covenant-bound UTXO state, the update logic is enforced by consensus
rather than by an operator you have to trust. The reputation token is self-perpetuating: each
state transition produces a new UTXO that carries the covenant forward, spendable only under
the same rules.

The longer-term aim is an authorization model where reputation verdicts are *portable* — an
oracle signs an authorized change, and anyone can carry that signed verdict on-chain
("Everyone is an Oracle"). That broader design is documented separately; this repo is the
technical core.

---

## Status (honest)

| Component | State |
|---|---|
| v3 covenant — reputation as UTXO state | **Live & proven on TN10.** Genesis established; multiple on-chain transitions confirmed (rep 100 → 105 → 110), enforced at the script layer. |
| Off-chain oracle verdict signer | **Done & tested.** Standalone crate (`signer/`) with a `cargo test` suite; CI green on every push. Signs a verdict binding `delta || nonce || covenant_id`. |
| v4 authorization (on-chain gate) | **In progress, blocked upstream.** Needs the `OpCheckSigFromStack` (CSFS) primitive wired into the SilverScript compiler (tracked in silverscript #122). The signing spec is pinned from the txscript source; the contract gate is built once CSFS lands. |
| Mainnet | **Not yet.** Will target an official SilverScript v1 stable release (which gates required compiler fixes) and a security review first. |
| Security audit | **Not done.** This is consensus-critical code handling signatures; an independent review by experienced Kaspa/Rust developers is required before any mainnet use. |

If you only take one thing from this table: **the live parts are real and demonstrable; the
unfinished parts are clearly unfinished. Nothing here is claimed to be production-ready.**

---

## The signer (verifiable right now)

The off-chain verdict signer is a standalone, tested Rust crate. You can verify its claims
yourself:

```bash
cd signer
cargo test
```

You should see 8 tests pass. Five of them are **rejection tests** — they prove a signature
binds every field of the verdict (a tampered delta, nonce, covenant_id, or a wrong pubkey all
*fail* verification). That accept-and-reject coverage is the point: it demonstrates the
signature actually authorizes a specific value, not just that signing round-trips.

The signing format matches the on-chain `OpCheckSigFromStack` verification exactly (confirmed
from the Kaspa txscript source): blake2b-256 over the 48-byte verdict, then BIP340 schnorr
over the resulting 32-byte digest, no domain separation.

Run the CLI to produce a verdict signature:

```bash
cd signer
cargo run --bin oracle_sign_verdict -- \
  --delta 5 --nonce 0 \
  --covenant-id <64-hex-char covenant id>
```

---

## Repository layout

```
covenants/        SilverScript covenant sources (v3 live; v4 = detached-verdict design target)
signer/           Standalone tested crate: off-chain oracle verdict signer (+ CI)
rust-examples/    On-chain tooling (genesis establish, continuation spender, address derivers)
docs/             Project status, command reference, helper scripts
```

The canonical project status lives in `docs/oracle_protocol_status.md`.

---

## Built on

- [Kaspa](https://kaspa.org) — the BlockDAG this runs on (Toccata covenants, Testnet 10).
- [SilverScript](https://github.com/kaspanet/silverscript) — the covenant language.

This project is independent and not officially affiliated with the Kaspa project.

---

## License

MIT. See [LICENSE](LICENSE). Permissive by intent — reputation infrastructure is more useful
the more freely it can be built on.

---

## Status of trust

This is early, unaudited, testnet-stage research software. Do not use it with anything you
can't afford to lose. Findings, reviews, and pull requests from people who know this layer
are welcome.
