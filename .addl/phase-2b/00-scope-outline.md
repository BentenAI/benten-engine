# Phase 2b Scope Outline

**Status:** Scope frozen; pre-R1 opens after Phase 2a ships.
**Relationship to Phase 2a:** Phase 2 was split 2a/2b during Phase 2a pre-R1 on review-lens-coherence grounds. Phase 2a handles evaluator completion + debt close + 4 of 6 remaining invariants; Phase 2b handles SANDBOX + WASM + compute-generative work.
**Predecessor plan:** `.addl/phase-2a/00-implementation-plan.md` — Phase 2a's groups G6, G7, G8, G10 (and 2b-portion of G11) carry forward into this phase with their own full ADDL cycle (pre-R1 → R1 → R2 → R3 → R4 → R5 → R4b → R6).

---

## 1. Scope at a glance

Phase 2b ships the SANDBOX + WASM + compute half of the original Phase 2 scope:

- **3 primitive executors** — STREAM (chunked output + back-pressure + SSE/WebSocket bridge), SUBSCRIBE (user-visible reactive primitive beyond the internal-IVM subscriber shipped in Phase 1 + extended in 2a via change-stream plumbing), SANDBOX (wasmtime host + fuel metering + instance pool + capability-derived host-function manifest).
- **2 remaining structural invariants** — Inv-4 (SANDBOX nest depth) + Inv-7 (SANDBOX output ≤ 1MB). Both land with the SANDBOX executor. (Invariants 8, 11, 13, 14 shipped in Phase 2a.)
- **Generalized IVM Algorithm B** with per-view strategy selection (A/B/C) + user-registered views beyond the 5 hand-written Phase-1 views. Subsumes the 5 existing views as strategy-tagged specializations.
- **WASM build target** via napi-rs v3 with network-fetch `KVBackend` stub (snapshot-blob read-only flavour — full iroh network backend is Phase 3).
- **Module manifest format** (requires-caps, provides-subgraphs, migrations) + install/uninstall APIs on `Engine`.
- **Paper-prototype revalidation** against the full 12 primitives — the Phase-1 validation used the pre-revision vocabulary with VALIDATE + GATE; Phase 2b is the first moment where all 12 executors are live to measure the actual SANDBOX rate against the < 30% prediction.
- **Full `missing_docs` sweep** across `benten-eval` and `benten-engine` — was deferred in Phase 1 because the evaluator surface wasn't stable. Now it is.

---

## 2. Architectural decisions carried forward from Phase 2a pre-R1

Two load-bearing design decisions were baked in during Phase 2a's pre-R1 because they affect the 2a↔2b interface. Phase 2b pre-R1 critics can still pressure-test them but evaluate a decided position:

### 2.1 `ExecutionState` on-disk format: DAG-CBOR + CIDv1 envelope (§9.1 of Phase 2a plan)

- Envelope shape: `{schema_version: u8 = 1, payload: dag-cbor-encoded ExecutionState}` rolled through the `Node::cid` machinery.
- Motivation: content-addressing preserves Phase 3 sync / Phase 6 AI-workflow-forking / Phase 7 Garden-approval-flow symmetry.
- Phase 2a's G3 WAIT establishes this shape; Phase 2b's G6 STREAM composes on it.
- Debuggability addressed via `benten-dev inspect-state` pretty-print, not via format choice.

### 2.2 SANDBOX host-function manifest: capability-derived with named-manifest DX sugar (§9.3 of Phase 2a plan)

- **Architectural primitive:** host functions declare their own `requires: "host:<domain>:<action>"` scope. At SANDBOX init, the engine intersects (host-supported functions) × (caller's cap grants) and exposes only the intersection.
- **DX sugar:** named manifests (`"compute-basic"`, `"compute-network"`, …) resolve to cap bundles — a developer writes `sandbox({ module, manifest: "compute-basic" })` and the engine grants the bundled caps for that call. Named manifests are lookups over caps, not a parallel permission system.
- **Rationale:** single security model (UCAN-compatible cap grants already in place); foreclosed by Phase 8 Credits compute marketplace (third parties running code on user hardware need fine-grained host-fn caps); Phase 6 AI assistants generating SANDBOX-hosted tools inherit caps through CALL + attenuation naturally.

---

## 3. Implementation groups (carried forward from Phase 2a §3, frozen shape)

Full group detail in `.addl/phase-2a/00-implementation-plan.md` §3 (sections marked `[PHASE 2B]`). Summary:

| Group | Scope | Long pole? | R1/R6 lens |
|-------|-------|-----------|------------|
| G6 | STREAM + SUBSCRIBE executors + napi chunk sink + TS DSL | — | `websocket-engineer` + `code-reviewer` |
| G7 | SANDBOX executor + wasmtime host + instance pool + Inv-4/7 + DSL | **yes** | `wasmtime-sandbox-auditor` + `security-auditor` + `performance-engineer` |
| G8 | Generalized IVM Algorithm B + strategy selection + user views | — | `ivm-algorithm-b-reviewer` + `performance-engineer` |
| G10 | WASM runtime (wasm32-wasip1) + network-fetch KVBackend stub + module manifest | — | `napi-bindings-reviewer` + `determinism-verifier` |
| G11-2b | Paper-prototype revalidation + full missing_docs sweep + full DSL examples refresh | — | `documentation-engineer` + `dx-optimizer` |

Dependency ordering: G6 (STREAM depends on 2a's `ExecutionState` shape — i.e. **Phase 2a completion**) → G7 (SANDBOX, blocks on wasmtime; depends on Phase 2a completion for the stable host-error surface from G1 and the invariant machinery from G4/G5-A/G5-B) → G8 (Algorithm B can run in parallel with G7) → G10 (WASM requires SANDBOX-disabled-on-WASM flag) → G11-2b (wraps).

**arch-8 correction:** G7's dependency list in earlier drafts of this document referenced G3/G4/G5 individually. Those are Phase 2a groups; Phase 2b G7 depends on "Phase 2a completion" as a single atomic predecessor.

---

## 4. Pre-R1 triggers (when Phase 2b opens)

Phase 2b pre-R1 dispatches when:

1. **Phase 2a closes** — all 2a R6 findings triaged; `ExecutionState` envelope shape frozen; arch-1 dep break landed.
2. **`wasmtime` version survey** — confirm the wasmtime API surface at that moment matches assumptions in G7 (fuel metering, instance pool, host-function declarations). If wasmtime has had breaking changes, Rank-1 risk (wasmtime API stability) is already realised and pre-R1 must address.
3. **Phase-2a-era perf regression check** — confirm the durability + AST cache perf gate (Phase 2a exit criterion #5) is still green. If it regressed, SANDBOX work on top has a shaky base.

---

## 5. Open design questions for 2b pre-R1 / R1 to debate

Phase 2a decided (1) `ExecutionState` envelope shape and (2) SANDBOX host-function manifest architecture. Phase 2b's own debates:

- **Initial host-function set** — what's the minimum useful `host:compute:*` surface shipped in G7? Too thin → user-facing regressions vs a hand-allowlist baseline. Too fat → attack surface + more invariant pairs to test. Candidates: `host:compute:time`, `host:compute:log`, `host:compute:random`, maybe `host:compute:kv:read` for gated read-only storage access.
- **Named-manifest registry shape** — TOML file vs `benten-caps`-adjacent module vs inline constants vs a dedicated `docs/HOST-FUNCTIONS.md` catalog with codegen. R1 decision.

### §5a: `docs/HOST-FUNCTIONS.md` catalog (arch-7 placeholder)

Phase 2b authors `docs/HOST-FUNCTIONS.md` as the canonical namespace for `host:<domain>:<action>` cap scopes. Parallel surface to `docs/ERROR-CATALOG.md`: each host function has a stable, documented cap string with a `since:` phase annotation, description, argument schema, and permission semantics. Phase 2a locks the cap-string *shape* in `benten-errors` (so codegen can reference `host:<domain>:<action>` without concrete functions existing yet); Phase 2b fills in the actual functions as it ships each one in G7. New host functions in later phases append to the catalog.
- **Enforcement layer** — cap check at SANDBOX-init resolution time (cheap, happy path) vs at each host-fn invocation (stricter, catches mid-eval revocation) vs both. Probably both, mirroring Compromise #1 TOCTOU refresh shape.
- **STREAM back-pressure semantics** — pull-based bounded channel (Phase 2a planner's preference) vs push with producer-side flow-control credits (iroh-style, Phase-3-forward-compat). R1 decides.
- **Algorithm B strategy-selection API** — developer opt-in (explicit `Strategy::B`) vs auto-selection based on access pattern at register time vs runtime adaptation. R1 decides.
- **Module manifest format** — TOML / JSON / a Benten-native shape. Includes migration-format shape (subgraph-CID-based references vs `(actor, action)` references).
- **WASM target KVBackend semantics** — snapshot blob (chosen per Phase 2a §9.8) at engine construction vs reject-all-reads vs full iroh inlined. Snapshot confirmed as Phase 2b default; 2b R1 reviews the exact handoff shape.
- **Paper-prototype revalidation protocol** — single-sample measurement vs multi-handler-cohort — to give the SANDBOX-rate-vs-30% prediction some statistical weight. R1 decides.

---

## 6. Risks carried forward (Phase 2a-era perspective)

- **Rank 1: wasmtime API stability** — see Phase 2a plan §5 Rank 1.
- **Rank 2: fuel-to-wall-clock mapping determinism** — see Phase 2a plan §5 Rank 2.
- **Rank 4: STREAM back-pressure across napi** — see Phase 2a plan §5 Rank 4.
- **Rank 5 (residual): SANDBOX host-fn surface shipping set** — architecture decided; see §5 above.
- **Rank 9: WASM target KVBackend semantics** — see Phase 2a plan §5 Rank 9.
- **Rank 10: paper-prototype SANDBOX-rate drift** — see Phase 2a plan §5 Rank 10.

---

## 7. What this document is NOT

- Not a pre-R1 plan. The 2b pre-R1 planner will author `.addl/phase-2b/00-implementation-plan.md` when 2b opens.
- Not a frozen scope statement. Phase 2a's R5 + R6 may surface findings that re-allocate items between 2a and 2b; `docs/future/phase-2-backlog.md` stays the canonical forward-looking doc until 2b's own plan lands.
- Not a calendar commitment. 2b opens when 2a closes, not on a fixed date.

---

*Seeded during Phase 2a pre-R1 planning (2026-04-21). Re-examine at 2b pre-R1 open.*
