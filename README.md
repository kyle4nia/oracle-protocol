# Oracle Protocol

A reputation system that runs on the Kaspa blockchain, where reputation is the asset and everyone is an oracle.

## Why this exists

Human coordination runs on trust, and trust runs on reputation. For most of history that worked because reputation was local and hard to fake. That is no longer true. The information systems we rely on to know who to trust are controlled by a small number of actors, and control over those systems is control over what people can see, say, and believe. Our evolutionary wiring, built for small groups and face to face reputation, gets exploited by whoever owns the pipes.

The way out is to decouple. Put the reputation ledger somewhere no single party owns and no single party can rewrite. A decentralized information system does not fix human nature, but it takes the lever away from the people currently pulling it. Liberty and justice for all is not a slogan you can enforce from the top down. It is a property that emerges when no one holds the master switch.

Oracle Protocol is one piece of that. It makes reputation a thing that lives on chain, that anyone can read, that no one can silently edit, and that moves according to rules everyone can inspect.

## What it does

Reputation lives as a UTXO on Kaspa. Each reputation record is a coin held by a covenant, a small on chain program that controls how the record can change. The current reputation value is baked into the coin. To change it, you spend the coin and create a new one with the updated value, and the covenant only allows the spend if the rules are followed.

The rule that matters: reputation only moves when an oracle signs off on it. An oracle is anyone whose verdicts the system is configured to trust. In this release there is a single oracle. The design goal is that eventually everyone is an oracle and verdicts are aggregated, but that requires primitives Kaspa does not expose yet, so the honest current state is one signer.

The prime directive underneath all of it: you can do what you want as long as it does not prevent others from doing what they want. That is the floor the whole system is built to protect, not a rule bolted on top.

## How it works

The reputation coin carries two values in its state: the reputation number and a nonce. The nonce goes up by one every time the reputation changes. It stops old approvals from being replayed.

To move reputation, the oracle signs a verdict. A verdict is three things joined together: the amount of change, the current nonce, and the id of the specific covenant it applies to.

```
verdict = delta (8 bytes) || nonce (8 bytes) || covenant_id (32 bytes)
```

The oracle signs the hash of that verdict with a standard Schnorr signature. The spend transaction hands the covenant the verdict values and the signature. The covenant checks the signature on chain using Kaspa's checkSigFromStack opcode. If the signature is good, the nonce matches, and the covenant id matches, the spend goes through and reputation moves. If any of those are wrong, the network rejects it.

Because the nonce is part of what gets signed, an old verdict cannot be reused. We proved this on chain: a transition was accepted, then the exact same approval was submitted again against the new state and the network rejected it.

## Status

This is experimental and lives on Kaspa Testnet 10.

Proven and live on TN10:

- The covenant is deployed. Reputation has moved through real on chain transitions.
- The replay protection works, verified with a rejected replay on chain.
- The off chain signer is a standalone crate with a full test suite.
- One spender tool drives any transition without code changes. It reads the current on chain state, signs against it, and submits.

There is more coming. A web app is planned, and that is where the self erecting version of the protocol begins, the version that adjusts toward its goals without anyone privileged running it. If you want to see where this goes, follow the repo and check back.

## The bigger picture

Oracle Protocol is part of a set of interconnected projects built on the same idea, that decentralization is not a technical preference but the precondition for freedom that cannot be revoked. The reasoning behind it is laid out in *Revolution: A Practical Guide*.

## License

ISC.
