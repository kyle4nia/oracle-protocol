# Oracle Protocol — Operating Baseline

_The baseline operating environment for working on Oracle Protocol. A fresh chat
should read this alongside `oracle_protocol_status.md`: the status doc says WHERE the
project is; this doc says HOW we work. Together they let a new session pick up cleanly._

---

## Who I'm working with (read this first)

- **No real-world coding/development experience.** Understands the basic concepts but
  has not worked as a developer. Do NOT assume familiarity with tooling, syntax,
  conventions, or "obvious" steps. Explain in plain English before showing code; define
  jargon on first use; say what a command does and what output to expect so success/
  failure is legible without reading the code itself.
- **Strengths: problem-solving, persistence, brute-force follow-through.** Will grind a
  hard problem to the ground and not give up. Lean on this — give a clear next action and
  it gets done. The bottleneck is never effort or resolve; it's translation between intent
  and the machinery.
- **Typing is slow / error-prone.** Favor commands that can be pasted, not retyped.
  Single-line where possible. Minimize how much has to be hand-typed. Long hand-typed
  blocks are a tax; design around it.
- **Wants the honest read, always.** Direct assessment over agreement or encouragement.
  Pushback is welcome when warranted — flattery and engagement-padding are not. If
  something is a bad idea, say so and why. If something is a guess, label it a guess.

---

## How we work (interaction cadence)

- **One command at a time.** State the single next action. They run it, paste the output,
  then the next step. No multi-step walls of instructions — it breaks the feedback loop and
  makes failures hard to localize.
- **Single-line terminal commands.** The VS Code integrated terminal REVERSES multi-line
  pastes line-by-line. Reliable workarounds: single-line commands, copy-file-plus-replace,
  or paste multi-line content into the EDITOR (not the terminal).
- **Plain-English-before-code.** Explain the what and why first, then the code. Define
  terms on first use. State expected output.
- **Isolate one hard problem at a time.** Prove it in the smallest possible test before
  building on it (local VM verify before broadcast; signer self-test before trusting it).
- **Be upfront about tradeoffs and difficulty.** Don't paper over hard parts or pretend a
  shaky step is solid.
- **Confirmed vs guessed.** Always distinguish "verified from source / tested" from
  "assumption." Flag guesses explicitly so they don't get built on as if proven.

---

## Standing technical discipline

- **Source-of-truth file rule.** Every new `.rs` tool/harness is saved to
  `C:\oracle-protocol\rust-examples\` IMMEDIATELY, in addition to the disposable build/clone
  location (rothschild\src\bin or crypto\txscript\examples). The oracle-protocol copy is
  canonical; the Kaspa clone is re-clonable and throwaway. Do this the moment a file is
  created, not later.
- **Keep paired `.sil` copies in sync.** A divergence between the oracle-protocol copy and
  any test/example copy once caused a long stale-file debugging detour. Always compile the
  oracle-protocol copy.
- **Dry-run before broadcast.** Clients gate real submission behind `--go`; run without it
  first and read the dry-run output before sending anything on-chain.
- **Verify-by-reading-the-source for anything that ENFORCES.** Born from the checkDataSig
  catch (a builtin that compiled clean, passed local verify, and enforced nothing). For any
  builtin/opcode that is supposed to gate or check something, read its compiler lowering /
  engine implementation before trusting it. A gate tested only on valid input is half-proven
  — always include the rejection test (bad input MUST fail).
- **The status doc is the real memory.** `oracle_protocol_status.md` is the durable record,
  not the chat. Chats compact and lose detail; the doc and the files in `C:\oracle-protocol`
  do not. Keep the doc current after every meaningful step. A fresh chat resumes from the
  doc, not from conversation history.

---

## The lens: always view through "self-erecting"

This is the orienting principle, not just a feature. The protocol is meant to be
**self-erecting** — it must periodically adjust itself to conform to its overarching goals
without a trusted coordinator doing the adjusting. Reputation-as-consensus-asset, the prime
directive as immovable foundation, "Everyone is an Oracle," the detached-verdict
authorization model — these all exist to serve that self-erecting property. Eventually the
system relies heavily on it: it has to evolve toward its own stated aims under its own
rules.

Practical implication for how we evaluate every design decision: ask not only "does this
work" but **"does this preserve / advance the self-erecting property?"** A solution that
works but requires a permanent privileged operator (the abandoned admin-checkSig model is
the canonical example) is the WRONG solution even when it compiles and passes, because it
violates the lens. When choosing between approaches, the one that lets the system govern and
adjust itself — rather than depending on an outside hand — wins, even at the cost of more
work now. When a shortcut is taken for a TN10 proof-of-concept, note explicitly how the
self-erecting version would differ, so the shortcut doesn't quietly become the design.

---

## Scope of this doc (deliberate boundary)

This captures the OPERATING ENVIRONMENT — how we work — because that transfers cleanly to
any session. It does not try to reconstruct rapport or "feel"; those are emergent from
accumulated context and don't load from a file. The mechanics here are what make any chat
useful for this project; the rest is a bonus of continuity, not a setting. Keep this doc to
operating reality, and update it when a new working nuance proves itself.
