# Security Posture — Benten Engine Phase 1

This document records the security claims Benten makes in Phase 1 and the known compromises those claims rest on. Each compromise is tied back to `.addl/phase-1/00-implementation-plan.md` (R1 Triage Addendum); this document is the written, referenceable form.

## Named Compromises

### Compromise #6 — BLAKE3 128-bit effective collision resistance

Benten uses **BLAKE3-256** with a 32-byte digest embedded in every CIDv1. The academic collision-resistance bound for any cryptographic hash is `2^(n/2)` (birthday bound), giving BLAKE3-256 a **128-bit effective collision resistance**. This is the bound that every Benten Phase-1 security argument rests on — NOT the full `2^256` preimage bound.

**Where this matters:**

- **Content-addressed Nodes (`Cid`).** A collision would allow a malicious writer to forge a Node that hashes to the same CID as a legitimate Node — a "masquerade" attack. 128-bit resistance requires ~`2^128` hashes to find a collision; infeasible under any classical threat model.
- **Version-chain `prior_head` threading** (`benten_core::version::append_version`). The API uses CIDs to name the head each writer observed. A collision on a CID used as `prior_head` could, in principle, let an attacker smuggle an alternative chain past the fork-detection check. The same 128-bit bound applies.
- **Phase 3 UCAN-by-CID.** Phase 3 will reference capability grants by CID. Revoke-by-CID paths assume the CID of a grant is unique; again, 128-bit collision resistance is the assumption.

**What this posture does NOT claim:**

- **Quantum resistance.** Grover's algorithm reduces the effective collision bound to `2^64` under a quantum adversary. This is still infeasible for the current state of quantum hardware, but it is no longer "categorically" secure. A post-quantum hash option is a Phase N+ consideration; BLAKE3 is not post-quantum.
- **Second-preimage resistance stronger than 128 bits.** For adversaries who already know a target CID and wish to construct a colliding Node, the dominant-term bound is still ~`2^128`. Benten does not rely on the higher 256-bit preimage bound.

**Phase 2 action items:**

- Mirror this posture into end-user docs (`docs/QUICKSTART.md` security section).
- Document the same assumption in the TypeScript wrapper's JSDoc for `@benten/engine` node-creation APIs.
- When Phase 3 introduces the UCAN-by-CID path, restate the bound at that integration point.

---

### Compromise #8 — `Engine::call` bypasses the evaluator for CRUD ops (Phase 1)

**Landed:** R5 G7 fix-pass (commit `3b714bf`).

The Phase 1 `Engine::call` dispatches CRUD operations (`create`, `list`, `get`, `delete`) via direct backend transactions rather than walking the registered handler Subgraph through `benten_eval::Evaluator`. The registered Subgraph is content-addressed, invariant-checked at registration, and stored — but it is NOT the execution path at call time for the CRUD fast-path.

**Where this matters:**

- The 14 structural invariants enforce at **registration** (correct). A malformed handler cannot be registered. At CALL time, the hardcoded CRUD path does not re-exercise the registered subgraph — it directly invokes `backend.transaction(|tx| tx.put_node(...))`.
- Typed error-edge routing is synthesized for the CRUD ops (`ON_NOT_FOUND`, `ON_CONFLICT`, `OK`). A user-registered handler with custom error edges beyond the CRUD set is unreachable via the fast-path.
- Handler-level `requires:` property enforcement for CRUD ops runs through the capability hook at transaction commit (correct), but primitive-level `requires:` annotations inside a registered Subgraph are not exercised.

**What this does NOT affect:**

- **Content-addressing:** the registered handler's CID is genuine; invariant-10 (order-independence) holds.
- **Thin-engine external contract:** `benten-eval` still knows nothing about `benten-graph`; `benten-engine` composes the two. The compromise is internal to how `Engine::call` resolves the dispatch.
- **Phase-1 exit criterion:** `crud('post').list` returns the paginated sorted listing; the external DX path works end-to-end at the surface level.
- **Non-CRUD handlers:** Subgraphs registered via `register_subgraph` with non-CRUD shapes follow the `dispatch_spec` path, which DOES dispatch through the stored spec. Only the `crud:<label>` fast-path short-circuits.

**Security consequence:**

A user who registers a handler expecting `Engine::call` to walk its registered subgraph receives the hardcoded CRUD semantics instead. This is a TRUST mismatch between the declared handler shape and the actual execution path. Auditors relying on "the registered subgraph IS what executes" must read the Phase-1 fast-path exception. Phase 2 MUST close this gap.

**Phase 2 action items:**

- Add `benten_eval::PrimitiveHost` trait; `Evaluator` takes `&dyn PrimitiveHost`; primitive executors dispatch through the host.
- `Engine` implements `PrimitiveHost`; `Engine::call` wraps `Evaluator::run(&handler, input, self)` in a transaction.
- Remove the CRUD fast-path in `dispatch_call`; all `Engine::call`s route through the evaluator uniformly.
- Re-run the full G6/G7 mini-review panel (operation-primitive-linter, code-as-graph-reviewer, benten-engine-philosophy) against the unified dispatch path.
- `Engine::trace` returns real per-primitive steps via `Evaluator::run_with_trace` instead of the fabricated 2-step placeholder.

---

*Future compromises with security implications will be appended as sections here, each tagged with the compromise number from the R1 Triage Addendum.*
