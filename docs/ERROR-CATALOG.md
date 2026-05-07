# Error Catalog

**Status:** Specification. Error codes and messages are reserved here before implementation so that every error the engine can produce has a stable code and a fix hint.

**Motivation:** The DX critic (2026-04-14 review) identified that the spec discussed error *edge types* (`ON_DENIED`, `ON_NOT_FOUND`, etc.) but had zero discussion of runtime error *messages* or codes. Meanwhile the 14 structural invariants will each fire rejection errors at registration time. Without a catalog, developers will hit "validation failed" with no context. This document is the contract.

## Format

Every error has:

- **Code:** Stable identifier, e.g. `E_CAP_DENIED`. Never reused, never renumbered. Prefixed by subsystem (`E_CAP_*` for capability, `E_INV_*` for structural invariants, `E_SYNC_*` for sync, etc.)
- **Message template:** A human-readable format string with placeholders.
- **Context fields:** Structured data included with the error.
- **Fix hint:** What the developer should do.
- **Thrown at:** Registration, evaluation, sync, or other specific layers.

All errors are structurally typed (not just strings) on the TypeScript side via napi-rs v3 generated types. Every `throw` in the Rust code must map to a code in this catalog.

## Registration-time errors (the 14 structural invariants)

### E_INV_CYCLE

- **Message:** "Subgraph contains a cycle involving Nodes: {cycle_path}"
- **Context:** `{ cycle_path: NodeId[] }`
- **Fix:** Subgraphs must be DAGs. Replace the back-edge with an ITERATE primitive if repetition is intended.
- **Thrown at:** Registration

### E_INV_DEPTH_EXCEEDED

- **Message:** "Subgraph depth {actual} exceeds configured max {max}"
- **Context:** `{ actual: number, max: number, longest_path: NodeId[] }`
- **Fix:** Reduce nesting of CALLs or increase max depth via capability grant.
- **Thrown at:** Registration

### E_INV_FANOUT_EXCEEDED

- **Message:** "Node {node_id} has {actual} outgoing edges, exceeds max fan-out {max}"
- **Context:** `{ node_id: NodeId, actual: number, max: number }`
- **Fix:** Reduce BRANCH cases or split the Node. BRANCH should be binary or multi-way; consider whether a match-table is cleaner.
- **Thrown at:** Registration

<!-- cr-g7a-mr-2 fix-pass: dropped orphan E_INV_SANDBOX_NESTED stub from
     Phase-1 placeholder. The Phase 2b SANDBOX nest-depth enforcement
     surface lives at E_INV_SANDBOX_DEPTH (G7-B) + E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED
     (runtime saturation; G7-A) — both documented later in the file. -->

### E_INV_TOO_MANY_NODES

- **Message:** "Subgraph has {actual} Nodes, exceeds max {max}"
- **Context:** `{ actual: number, max: number }`
- **Fix:** Break into smaller subgraphs connected via CALL.
- **Thrown at:** Registration

### E_INV_TOO_MANY_EDGES

- **Message:** "Subgraph has {actual} Edges, exceeds max {max}"
- **Context:** `{ actual: number, max: number }`
- **Fix:** Same as E_INV_TOO_MANY_NODES.
- **Thrown at:** Registration

### E_INV_SYSTEM_ZONE

- **Message:** "Node IDs and labels cannot begin with the reserved 'system:' prefix — it's reserved for engine internals"
- **Context:** `{ node_id: NodeId, label: string }`
- **Fix:** The `system:` prefix is reserved for engine internals; both labels AND node IDs that start with `system:` are rejected at registration as defence-in-depth (G5-B-i Decision 6 reserved-prefix DX improvement). Pick a non-reserved label/ID and re-register. Runtime probing of resolved (TRANSFORM-computed) CIDs collapses system-zone targets to `Ok(None)` on the user-visible surface; only the user-facing `create_node` path fires this error directly for an input label.
- **Thrown at:**
    - Registration — literal-CID walker in `benten-eval::invariants::system_zone::validate_registration` (rejects a READ or WRITE operation node whose `"label"` property or node-id is a `system:*` literal).
    - Runtime — resolved-label probe in `benten-engine::primitive_host`:
        - `read_node` / `get_by_label` / `get_by_property` / `read_view` — TRANSFORM-computed CIDs whose resolved Node carries a `system:*` label collapse to `Ok(None)` / empty list at the user surface (symmetric with a backend miss).
        - `put_node` — fires `EvalError::Invariant(SystemZone)` before the `PendingHostOp` is buffered, so a handler WRITE of a `system:*`-labelled Node never reaches the storage-layer defence-in-depth guard (which would otherwise surface the Phase-1 `E_SYSTEM_ZONE_WRITE` code).
    - User-facing CRUD — `Engine::create_node` fires this code directly for any `system:*` label in the input Node's `labels` vector. `Engine::get_node` collapses system-zone reads to `Ok(None)` (the probe returns the typed code through the runtime telemetry path but not through the user-visible `Result`).
- **Phase:** 2a G5-B-i — **active**. Registration-time (literal-CID) + runtime (resolved-label via `RedbBackend::get_node_label_only` per Code-as-graph Major #1) enforcement live. The Phase-1 `E_SYSTEM_ZONE_WRITE` host-layer stopgap is retired on the user-facing surface (`Engine::create_node` and `PrimitiveHost::put_node` now fire `E_INV_SYSTEM_ZONE`); the graph-layer storage stopgap is retained as defence-in-depth.

### E_INV_DETERMINISM

- **Message:** "Operation {op_type} is classified non-deterministic but appears in a context declared deterministic"
- **Context:** `{ op_type: string, node_id: NodeId }`
- **Fix:** Move non-deterministic operations (SANDBOX, EMIT non-local) outside the deterministic context or relax the declaration.
- **Thrown at:** Registration

### E_INV_ITERATE_MAX_MISSING

- **Message:** "ITERATE Node {node_id} missing required `max` property"
- **Context:** `{ node_id: NodeId }`
- **Fix:** ITERATE requires an explicit `max` to guarantee termination. Add `max: <integer>`.
- **Thrown at:** Registration

### E_INV_ITERATE_BUDGET

- **Message:** "Cumulative iteration budget {actual} exceeds bound {bound} through nested ITERATE/CALL"
- **Context:** `{ actual: number, bound: number }`
- **Fix:** Reduce the multiplicative iteration space. The cumulative budget is the worst-case product of ITERATE `max` values and non-isolated CALL callee bounds along any DAG path through the handler. Flatten the nested iteration, or declare `isolated: true` on a CALL whose callee runs under its own grant's bound (the callee frame resets the cumulative rather than inheriting the caller's remaining budget — Code-as-graph Major #2 / Option B).
- **Thrown at:** Registration (Phase 2a multiplicative-through-CALL / Code-as-graph Major #2) and Evaluation (Phase 1 runtime flat budget, preserved at `DEFAULT_ITERATION_BUDGET = 100_000` in `crates/benten-eval/src/evaluator.rs`).

  Context shape note (G11-A doc review): the registration-time variant does NOT populate a `path: NodeId[]` field. The multiplicative walker in `benten-eval::invariants::budget` reports the product-over-paths `actual` and the configured `bound`; the specific DAG path that produced the worst-case product is not surfaced in the error payload. G4-A Code-as-graph Major #2 cleanup / Phase-2a M4 residual.
- **Phase:** 1 (runtime flat budget) + 2a (registration-time multiplicative form — G4-A lands the static product-over-paths walker in `crates/benten-eval/src/invariants/budget.rs` + `crates/benten-eval/src/evaluator/budget.rs` per cr-r1-3 shared-helper coordination). The Phase-1 nest-depth-3 stopgap (`E_INV_ITERATE_NEST_DEPTH`) is retired at Phase 2a open; the multiplicative form supersedes it. Default registration-time bound: `DEFAULT_INV_8_BUDGET = 500_000`.

### E_INV_ITERATE_NEST_DEPTH

> **⚠️ Not firing in production.** Retired at Phase-2a open: superseded by `E_INV_ITERATE_BUDGET` (multiplicative form). Catalog entry retained for forward-/backward-compat string round-trip; the Rust enum variant has been removed.

<!-- reachability: ignore -->

- **Message:** "ITERATE nesting depth {depth} exceeds Phase 1 limit {max}"
- **Context:** `{ depth: number, max: number, path: NodeId[] }`
- **Fix:** Phase 1 bounded ITERATE nesting structurally at depth 3 as a stopgap for the cumulative-budget enforcement shipped in Phase 2a. Retired at Phase 2a open — `E_INV_ITERATE_BUDGET` supersedes it. The catalog entry + TS class spelling stay reserved (catalog IDs are stable across phases); the Rust `ErrorCode` variant has been removed because no production path constructs it. The reachability annotation above is the drift-detector's signal that this is a deliberate forward-/backward-compat retention rather than aspirational prose.
- **Thrown at:** Never (retired)
- **Phase:** 2 (retired-at-Phase-2a-open marker — Phase >1 keeps it out of `phase1Required` so the drift detector does not demand a Rust enum variant. See `E_INV_ITERATE_BUDGET` for the live Phase 2a multiplicative replacement).

### E_INV_CONTENT_HASH

- **Message:** "Content hash mismatch for {node_id}: expected {expected}, computed {actual}"
- **Context:** `{ node_id: NodeId, expected: Cid, actual: Cid }`
- **Fix:** A stored Node's computed content hash does not match its key. Indicates on-disk corruption or an incompatible serialization migration. Re-hash the Node from source; if persistent, restore from a backup or re-ingest.
- **Thrown at:** Registration / read
- **Phase:** 1 (invariant 10 enforcement)

### E_INV_REGISTRATION

- **Message:** "Subgraph registration failed for {handler_id}: {reason}"
- **Context:** `{ handler_id: string, reason: string, violated_invariants: number[] }`
- **Fix:** Catch-all for registration failures where no more specific `E_INV_*` code applies. The `violated_invariants` list enumerates the specific invariants that rejected the subgraph.
- **Thrown at:** Registration
- **Phase:** 1 (invariant 12 enforcement)

## Evaluation-time errors

### E_CAP_DENIED

- **Message:** "Capability {required} not granted to {entity} for WRITE on {target}"
- **Context:** `{ required: string, entity: EntityId, target: NodeId }`
- **Fix:** Grant the capability, or call from a context that already has it. `requires` on the Node indicates the needed grant.
- **Thrown at:** Evaluation (at commit, not at individual WRITE, per the transaction-capability interaction rule)

### E_CAP_DENIED_READ

- **Message:** "Capability {required} not granted to {entity} for READ on {target}"
- **Context:** `{ required: string, entity: EntityId, target: NodeId }`
- **Fix:** Read-side capability denial. Phase 1 chooses honest-leaks-existence semantics: this error confirms the resource exists but the caller lacks read authority. Phase 3 sync may add a per-grant `existence_visibility: hidden` option that returns `E_NOT_FOUND` instead.
- **Thrown at:** Evaluation (READ with capability policy configured)
- **Phase:** 1 (named compromise on existence-leakage; see implementation plan §5 Rank 10)

### E_CAP_REVOKED_MID_EVAL

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 2b. The frozen code surface that the evaluator's batch-boundary recheck will return once refresh-point-5 wiring lands. TOCTOU integration tests construct it today; the production firing site is deferred (see `.addl/phase-2a/00-implementation-plan.md` G9-A residuals).

<!-- reachability: ignore -->
<!-- Rationale: Phase-1 named compromise #1 (TOCTOU window). `CapError::RevokedMidEval` is the frozen code surface that the evaluator's batch-boundary recheck will return once wired in R5 (see `crates/benten-caps/src/error.rs` docstring + `crates/benten-engine/tests/integration/cap_toctou.rs`). Construction sites live in TOCTOU integration tests today; drift-detect correctly flags the production gap. Remove this annotation when the evaluator's refresh-point-5 wiring lands in R5/G9-A. -->

- **Message:** "Capability {grant_id} was revoked during ongoing evaluation at {revoked_at}"
- **Context:** `{ grant_id: NodeId, revoked_at: HlcTimestamp, batch_boundary: number }`
- **Fix:** Distinct from `E_CAP_REVOKED` (Phase 3 sync-side revocation). Fired when a cap is revoked between the start of evaluation and a capability re-check point (commit boundary, CALL entry, or every N ITERATE iterations, default 100). Phase 2 Invariant 13 tightens the window to per-operation.
- **Thrown at:** Evaluation
- **Phase:** 1 (named compromise; see implementation plan §5 Rank 10 and §2.4 P1 TOCTOU-window note)

### E_CAP_NOT_IMPLEMENTED

- **Message:** "Capability backend '{backend}' does not implement check_write in phase {phase}"
- **Context:** `{ backend: string, phase: number, alternative: string }`
- **Fix:** Distinct from `E_CAP_DENIED` — this signals operator misconfiguration (configured a capability backend that isn't implemented yet), not an authorization failure. `UCANBackend` ships as a stub in Phase 1 and fully in Phase 3. Configure `NoAuthBackend` for embedded/local-only use, or provide a custom `CapabilityPolicy` impl. Routes to the subgraph's `ON_ERROR` edge, not `ON_DENIED`.
- **Thrown at:** Evaluation (at commit when an unimplemented backend is configured)

### E_CAP_REVOKED

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 3. Surfaces from `sync-receive` when a peer propagates a revocation; the Atrium sync stack lands with `benten-sync` in Phase 3. The catalog entry pins the wire code so peer-emitted `E_CAP_REVOKED` round-trips through the Phase-1 enum without collapsing to `ErrorCode::Unknown(_)`.

<!-- reachability: ignore -->
<!-- Rationale: Phase-3 sync-subsystem code. `CapError::Revoked` surfaces from `sync-receive` when a peer propagates a revocation over the Atrium wire; the Atrium stack lands in Phase 3 with `benten-sync`. Kept as the stable wire code the Phase-1 `ErrorCode` enum round-trips so `E_CAP_REVOKED` strings arriving from a newer peer don't collapse to `ErrorCode::Unknown(_)`. Remove this annotation when `benten-sync` wires the first `Err(CapError::Revoked)` construction site. -->

- **Message:** "Capability {grant_id} was revoked at {revoked_at}"
- **Context:** `{ grant_id: NodeId, revoked_at: HlcTimestamp }`
- **Fix:** Request a new grant. Revocation propagates via sync with priority.
- **Thrown at:** Evaluation, sync-receive

### E_CAP_ATTENUATION

- **Message:** "Delegated capability scope '{child_scope}' is not a subset of parent scope '{parent_scope}'"
- **Context:** `{ parent_scope: string, child_scope: string, chain: GrantId[] }`
- **Fix:** UCAN attenuation must narrow, never widen. Review the delegation chain.
- **Thrown at:** Registration (for static chains), evaluation (for dynamic CALL with `isolated: false`)

### E_WRITE_CONFLICT

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 2b (native call path). Phase-1/2a runtime surface is edge-routed via `ON_CONFLICT`; the engine stamps the code on the routed step. The Rust `EvalError::WriteConflict` variant is reserved for the Phase-2b native call path.

<!-- reachability: ignore -->

- **Message:** "Expected version {expected}, found {actual} on {target}"
- **Context:** `{ target: NodeId, expected: VersionHash, actual: VersionHash }`
- **Fix:** Re-read, rebase changes, retry. Typical optimistic concurrency pattern.
- **Thrown at:** Evaluation (CAS WRITE). **Runtime surface is edge-routed, not Rust-enum-valued:** WRITE's `cas` mode routes conflicts via the `ON_CONFLICT` edge; the engine stamps `error_code: "E_WRITE_CONFLICT"` on the routed step in `crates/benten-engine/src/primitive_host.rs::outcome_from_terminal_with_cid` (`"ON_CONFLICT"` arm of the edge match). Callers read the code off the edge-routing metadata, not via a `match` on an `Err(EvalError::WriteConflict)` — the enum variant exists for forward-compat with a Phase-2 native Rust path but has no construction site in Phase-1 production code. The drift-detector's `reachability: ignore` annotation reflects this asymmetry.

<!-- cr-g7a-mr-2 fix-pass: dropped Phase-1 placeholder duplicates of
     E_SANDBOX_FUEL_EXHAUSTED + E_SANDBOX_TIMEOUT + E_SANDBOX_OUTPUT_LIMIT.
     The canonical Phase-2b SANDBOX surface (with `Reserved at G7-A
     scaffold; G7-C wires the firing site` reachability discipline)
     lives in the "Phase 2b G7-A SANDBOX surface" section later in
     this file. The Phase-1 placeholders contradicted the Phase-2b
     entries (different message/context shapes; renamed E_SANDBOX_TIMEOUT
     -> E_SANDBOX_WALLCLOCK_EXCEEDED + E_SANDBOX_OUTPUT_LIMIT -> E_INV_SANDBOX_OUTPUT)
     and were producing TS-narrowing orphans. -->

### E_INV_SANDBOX_DEPTH

- **Message:** "SANDBOX nest depth {depth} exceeds configured max {max}"
- **Context:** `{ node_id: NodeId, depth: number, max: number }`
- **Fix:** Reduce SANDBOX nesting (a SANDBOX whose subgraph CALLs another handler that itself SANDBOXes counts toward the same depth at registration time per D20). Either flatten the call chain or increase `max_sandbox_nest_depth` via capability grant.
- **Thrown at:** **Registration** (static SubgraphSpec analysis at `invariants::sandbox_depth::validate_registration`) — fully active. **Runtime** — fully active at Phase 2b close (R6FP-G1 / PR #62, 3-lens convergent fix). `AttributionFrame.sandbox_depth` threads transitively through `ActiveCall` in `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (`frame.sandbox_depth = frame.sandbox_depth.saturating_add(1)`); the dispatching frame is constructed with `sandbox_depth: nested_depth` in both match arms of the same function so SANDBOX-inside-CALL-inside-SANDBOX inherits the parent's depth. See `docs/INVARIANT-COVERAGE.md` §"Inv-4 + Inv-7 runtime arm status" for the wiring trace.
- **Phase:** 2b (G7-B Inv-4 registration arm; wave-8b structural plumbing of the runtime field; R6FP-G1 / PR #62 closes the runtime depth-threading)

### E_INV_SANDBOX_OUTPUT

- **Message:** "SANDBOX output {would_be} bytes exceeds max {limit} (consumed {consumed} + attempted {attempted})"
- **Context:** `{ node_id: NodeId, consumed: number, attempted: number, would_be: number, limit: number, path: "primary_streaming" | "backstop" }`
- **Fix:** Reduce output emitted by the SANDBOX module's host-fn calls (or the primitive return value). D15 trap-loudly default — there is no opt-in silent-truncation flag. Use STREAM for progressive output if the workload genuinely needs unbounded byte volume.
- **Thrown at:** **Evaluation — fully active post-wave-8b.** The `path` field distinguishes the D17 PRIMARY streaming `CountedSink::write` enforcement (fires before host-fn bytes are accepted, in `crates/benten-eval/src/sandbox/counted_sink.rs`) from the D17 BACKSTOP return-value enforcement at the primitive boundary (`CountedSink::backstop_check` after the wasm guest returns). Both arms wired through wave-8b's host-fn trampoline + primitive boundary.
- **Phase:** 2b (G7-A + G7-B Inv-7 enforcement; wave-8b runtime wire-through; D15 + D17 PRIMARY+BACKSTOP)
- **D21 priority:** Lowest — fires before `E_SANDBOX_FUEL_EXHAUSTED` / `E_SANDBOX_WALLCLOCK_EXCEEDED` / `E_SANDBOX_MEMORY_EXHAUSTED` when ONLY the output axis trips; otherwise higher-priority axes fire first (D21 priority MEMORY > WALLCLOCK > FUEL > OUTPUT). See `docs/SANDBOX-LIMITS.md` for the rationale.

### E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED

> **Runtime arm wired at R6FP-G1 (PR #62).** The eval-side `SandboxError::NestedDispatchDepthExceeded` typed variant fires in `crates/benten-eval/src/primitives/sandbox.rs::execute` (depth-check guard immediately after the manifest resolve / random-cap pre-check block) once `attribution.sandbox_depth > config.max_nest_depth`. `AttributionFrame.sandbox_depth` threads transitively across nested SANDBOX entries via the parent `ActiveCall` (PR #62 3-lens convergent fix; see Inv-4 honest-disclosure block in `docs/SECURITY-POSTURE.md`). Both this typed error AND `E_INV_SANDBOX_DEPTH` (registration arm) are now active at Phase 2b close. The ESC-10 adversarial integration test stays `#[ignore]`'d pending the `testing_call_engine_dispatch` host-fn helper per `docs/future/phase-3-backlog.md` §7.3.A.7 — the runtime defense is wired; only the adversarial-test driver is paper-only.

- **Message:** "SANDBOX nested-dispatch depth saturated at {depth} (configured max {max})"
- **Context:** `{ node_id: NodeId, depth: number, max: number, saturation: "u8_ceiling" | "configured_max" }`
- **Fix:** SANDBOX nest-depth saturation overflow distinct from `E_INV_SANDBOX_DEPTH`. Two saturation paths fire this code: the `sandbox_depth: u8` counter saturates at `u8::MAX` (type-level ceiling — extremely deep CALL chains) and the configured `max_sandbox_nest_depth` boundary (capability-grant ceiling). Either case fires this typed error rather than wrapping silently. Reduce nesting per the same guidance as `E_INV_SANDBOX_DEPTH`; if hitting the u8 ceiling, the call topology is almost certainly accidentally recursive and needs structural redesign rather than a higher cap.
- **Thrown at:** Evaluation (saturation point at the SANDBOX entry — the counter-saturation check fires before the inner subgraph starts executing). Runtime firing site in `crates/benten-eval/src/primitives/sandbox.rs::execute` (depth-check guard).
- **Phase:** 2b (G7-B Inv-4 enforcement plumbing; R6FP-G1 / PR #62 lands the runtime threading)

### E_IVM_VIEW_STALE

- **Message:** "IVM view {view_id} marked stale; async recomputation in progress"
- **Context:** `{ view_id: NodeId, strategy: string }`
- **Fix:** Usually not an error the developer should handle; wait and retry, or accept eventually-consistent semantics. Indicates the per-view CPU/memory budget was exceeded during incremental update.
- **Thrown at:** Evaluation (READ from IVM view)

### E_TX_ABORTED

- **Message:** "Transaction aborted due to {reason}"
- **Context:** `{ reason: string, failed_node: NodeId | null }`
- **Fix:** Inspect the cause. Transactional subgraphs roll back ALL WRITEs on any failure. Check the `failed_node` field for the specific operation that caused the abort.
- **Thrown at:** Evaluation

### E_NESTED_TRANSACTION_NOT_SUPPORTED

- **Message:** "Nested transaction at {node_id} — Phase 1 does not support nested transaction scopes"
- **Context:** `{ node_id: NodeId, outer_tx_id: string }`
- **Fix:** Phase 1 limits transaction scopes to non-nested calls. Restructure so inner work completes within the outer transaction's single scope, or spawn it after the outer transaction commits. Phase 2 may lift this restriction.
- **Thrown at:** Evaluation
- **Phase:** 1 (named compromise)

### E_PRIMITIVE_NOT_IMPLEMENTED

- **Message:** "Primitive {primitive_type} is defined but its executor is not implemented in phase {phase}"
- **Context:** `{ primitive_type: string, node_id: NodeId, phase: number, target_phase: number }`
- **Fix:** All 12 primitive *types* are defined in Phase 1 so structural validation can recognize them. The 4 primitives WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX have executors that ship in Phase 2. Avoid calling these primitives in Phase 1 subgraphs or rely on a subgraph whose branch containing them is unreachable on the executed paths.
- **Thrown at:** Evaluation
- **Phase:** 1 (acknowledges Phase 2 deferral)

### E_SYSTEM_ZONE_WRITE

- **Message:** "WRITE to system-zone labeled Node '{label}' rejected: operation is not from a privileged engine path"
- **Context:** `{ label: string, target: NodeId, origin: string }`
- **Fix:** Phase 1 stopgap for Invariant 11 (which fully enforces at registration in Phase 2). User-operation WRITEs cannot touch `system:`-prefixed labels. Use the engine's privileged APIs — `Engine::grant_capability`, `Engine::create_view`, `Engine::revoke_capability` — for system-zone Node mutations.
- **Thrown at:** Evaluation (graph write-path)
- **Phase:** 1 (stopgap for invariant 11)

### E_TRANSFORM_SYNTAX

- **Message:** "TRANSFORM expression failed to parse: {reason} at position {offset}"
- **Context:** `{ reason: string, offset: number, expression: string, grammar_doc: string }`
- **Fix:** The TRANSFORM expression language is a positive-allowlist subset of JavaScript. Any token or AST shape not in the allowlist is rejected. Common causes: closures, `this`, imports, template literals with expressions, tagged templates, optional-chained method calls, computed property names referencing `__proto__`/`constructor`/`Symbol.*`, `new`/`with`/`eval`/`yield`/`async`/`await`, destructuring with getters.
- **Thrown at:** Registration (TRANSFORM parser runs at registration time)
- **Phase:** 1

### E_INPUT_LIMIT

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 2b (napi/B8 wave). The napi boundary's bounded streaming decoder is explicitly deferred (see the in-source acknowledgement at `bindings/napi/src/lib.rs::testing::deserialize_value_from_js_like`). The catalog entry pins the shape; the production firing site lands with the Phase-2b napi/B8 wave.

<!-- reachability: ignore -->
<!-- Rationale: R5 G8-B/B8 follow-up. The napi boundary's bounded streaming decoder (size/depth/bytes/CID-shape enforcement) is explicitly deferred — see the in-source acknowledgement at `bindings/napi/src/lib.rs::testing::deserialize_value_from_js_like` ("B8 harness's assertions about `ErrorCode::InputLimit` stay red until R5"). Drift-detect only scans `crates/*/src/`; the real firing site will live in `bindings/napi/src/` anyway, so this entry would need re-annotating rather than unignoring. Remove this annotation if/when a `crates/`-resident construction site is added (e.g., a shared limits module in `benten-core`). -->

- **Message:** "Napi boundary input exceeds {limit_kind} limit: {actual} > {max}"
- **Context:** `{ limit_kind: "map_size"|"list_size"|"bytes_len"|"text_len"|"nesting_depth"|"subgraph_bytes"|"node_count"|"edge_count", actual: number, max: number }`
- **Fix:** The TS → Rust boundary rejects oversized or pathologically-nested inputs to prevent DoS. Default limits: Value::Map 10K keys, Value::List 10K items, Value::Bytes 16MB, Value::Text 1MB, nesting depth 128, subgraph pre-parse bytes 1MB. Limits are configurable via the engine builder. Either simplify the input or raise the relevant limit explicitly with a capability-grant-authorized override.
- **Thrown at:** Napi binding (before any Rust allocation)
- **Phase:** 1

### E_SERIALIZE

- **Message:** "DAG-CBOR serialization failed: {detail}"
- **Context:** `{ detail: string }`
- **Fix:** The hash path's DAG-CBOR encoder refused the value. In Phase 1 this is effectively unreachable for well-typed input (all `Value` variants encode cleanly); the catalog entry exists so rare edge cases (e.g., encoder integer-overflow) surface a stable, non-empty code rather than an opaque "unknown" placeholder. Report as a bug.
- **Thrown at:** `Node::cid` / `Edge::cid` (pre-hash canonicalization)
- **Phase:** 1

### E_SYNC_HASH_MISMATCH

- **Message:** "Received content hash {received} does not match expected {expected}"
- **Context:** `{ node_id: NodeId, received: CidV1, expected: CidV1, peer: PeerId }`
- **Fix:** Possible tampering or corruption. Sync is aborted; investigate the peer.
- **Thrown at:** Sync-receive
- **Phase:** 3 (sync subsystem lands in Phase 3 with the Atrium stack)

### E_SYNC_HLC_DRIFT

- **Message:** "HLC timestamp {received} exceeds drift tolerance {max_drift} from local clock {local}"
- **Context:** `{ received: HlcTimestamp, local: HlcTimestamp, max_drift: Duration, peer: PeerId }`
- **Fix:** Peer's clock is outside tolerance. Triggers clock reconciliation handshake; if that fails, sync pauses.
- **Thrown at:** Sync-receive
- **Phase:** 3 (sync subsystem, see `E_SYNC_HASH_MISMATCH`)

### E_SYNC_CAP_UNVERIFIED

- **Message:** "Received WRITE lacks valid capability chain from {peer}"
- **Context:** `{ peer: PeerId, node_id: NodeId, missing: string }`
- **Fix:** Peer sent a change without proper authority. Sync-receive rejects; investigate peer trust level.
- **Thrown at:** Sync-receive
- **Phase:** 3 (sync subsystem, see `E_SYNC_HASH_MISMATCH`)

## Value / CID / backend errors

### E_VALUE_FLOAT_NAN

- **Message:** "Floating-point value is NaN; Value::Float rejects NaN for deterministic content-addressing"
- **Context:** `{ source_path: string }`
- **Fix:** The content-hash must be canonical; NaN compares unequal to itself and breaks hash determinism. Replace NaN with a sentinel (e.g. `Value::Null`) or with a specific finite value.
- **Thrown at:** Value construction / deserialization
- **Phase:** 1

### E_VALUE_FLOAT_NONFINITE

- **Message:** "Floating-point value is non-finite (Infinity / -Infinity); Value::Float requires finite numbers"
- **Context:** `{ source_path: string }`
- **Fix:** DAG-CBOR's canonical form rejects ±Infinity. Clamp to a finite bound or use `Value::Null`.
- **Thrown at:** Value construction / deserialization
- **Phase:** 1

### E_CID_PARSE

- **Message:** "CID bytes could not be parsed into a CIDv1: {detail}"
- **Context:** `{ detail: string, bytes_len: number }`
- **Fix:** Phase 1 accepts base32-lower-nopad multibase (`b`-prefixed) CIDv1 via both the napi boundary and the Rust `Cid::from_str` path. Check that the caller is not passing a base58btc / base64 / hex form, and that the bytes are not truncated.
- **Thrown at:** CID deserialization / napi boundary
- **Phase:** 1

### E_CID_UNSUPPORTED_CODEC

- **Message:** "CID codec {codec} is not supported; Phase 1 recognizes DAG-CBOR (0x71)"
- **Context:** `{ codec: number }`
- **Fix:** Phase 1 only accepts DAG-CBOR multicodec (0x71). Re-encode under the expected codec or await later-phase codec support.
- **Thrown at:** CID deserialization (`Cid::from_bytes` — distinct from `E_CID_PARSE`, which is reserved for length / version / digest-length structural failures)
- **Phase:** 1

### E_CID_UNSUPPORTED_HASH

- **Message:** "CID hash function {code} is not supported; Phase 1 recognizes BLAKE3 (0x1e)"
- **Context:** `{ code: number }`
- **Fix:** Phase 1 only accepts BLAKE3 multihash (0x1e). Re-hash with BLAKE3 or await later-phase multi-hash support.
- **Thrown at:** CID deserialization (`Cid::from_bytes` — distinct from `E_CID_PARSE`, which is reserved for length / version / digest-length structural failures)
- **Phase:** 1

### E_VERSION_BRANCHED

- **Message:** "Version chain has branched — multiple NEXT_VERSION edges from the same Version Node"
- **Context:** `{ anchor_cid: CidV1, branch_cids: CidV1[] }`
- **Fix:** A Version Node should have at most one NEXT_VERSION successor on any linear chain. Branches are a Phase-3 sync consequence; in Phase 1 this indicates a programming error writing two NEXT_VERSION edges. Walk the chain, pick the intended successor, and remove the other NEXT_VERSION edge.
- **Thrown at:** Version-chain traversal
- **Phase:** 1

### E_BACKEND_NOT_FOUND

- **Message:** "Named backend '{name}' is not registered on this engine"
- **Context:** `{ name: string }`
- **Fix:** Phase 1 wires a single in-memory + redb backend pair; alternate backends land with Phase-2. This error fires when a sub-component addresses a backend that is not configured.
- **Thrown at:** Engine builder / backend resolution
- **Phase:** 1

### E_NOT_FOUND

- **Message:** "Requested entity not found: {kind} {identifier}"
- **Context:** `{ kind: "node"|"edge"|"anchor"|"handler"|"view"|"grant", identifier: string }`
- **Fix:** Generic not-found — version-chain anchor miss, unregistered handler lookup, unknown view id, etc. Check that the caller has the correct CID / id; for handlers, confirm `registerSubgraph` / `registerCrud` ran successfully.
- **Thrown at:** Engine lookups
- **Phase:** 1

### E_GRAPH_INTERNAL

- **Message:** "Graph storage internal error: {detail}"
- **Context:** `{ detail: string }`
- **Fix:** Stable code for `GraphError::RedbSource` / `GraphError::Redb` / `GraphError::Decode` — a storage-layer failure (redb I/O, transactional abort, DAG-CBOR decode of a stored Node). The underlying `std::error::Error::source()` chain is preserved on the Rust side for diagnostics; at the TS boundary only the stable code is surfaced. Inspect logs or retry; persistent errors indicate on-disk corruption and should prompt a restore from backup.
- **Thrown at:** Graph backend (storage I/O)
- **Phase:** 1

### E_UNKNOWN

- **Message:** "Unknown error code (forward-compat fallback)"
- **Context:** `{ raw: string }`
- **Fix:** The drift-detect / catalog contract reserves `ErrorCode::Unknown(s)` as a forward-compat escape valve so a newer server emitting an unrecognized code does not crash an older client. If this code reaches a caller, update the engine / bindings to the latest release — the payload carries the raw code string the server actually emitted. Never thrown by Phase-1 Rust code deliberately; exists only to make the enum round-trip through `from_str` infallible.
- **Thrown at:** Forward-compat deserialization
- **Phase:** 1

## Engine-orchestrator errors

### E_DUPLICATE_HANDLER

- **Message:** "Handler id '{handler_id}' already registered with different subgraph content"
- **Context:** `{ handler_id: string, existing_cid: CidV1, attempted_cid: CidV1 }`
- **Fix:** Handler ids are unique within an engine. Either choose a distinct id, re-register with the same content (idempotent), or unregister the existing handler first. Two subgraphs with different CIDs cannot share an id.
- **Thrown at:** Engine (`register_subgraph` / `register_crud`)
- **Phase:** 1

### E_NO_CAPABILITY_POLICY_CONFIGURED

- **Message:** "No capability policy configured for .production() builder — call .capability_policy(...) or drop .production()"
- **Context:** `{}`
- **Fix:** `Engine::builder().production()` refuses to build without an explicit `CapabilityPolicy` (R1 SC2 fail-early guardrail). Call `.capability_policy(policy)` before `.open(...)`, or drop `.production()` if the engine should accept the `NoAuthBackend` default for local/embedded use.
- **Thrown at:** Engine builder
- **Phase:** 1

### E_PRODUCTION_REQUIRES_CAPS

- **Message:** "Production mode requires capabilities — .production() and .without_caps() are mutually exclusive"
- **Context:** `{}`
- **Fix:** `.production()` enforces that a capability policy must be configured. `.without_caps()` explicitly tears one down. Picking both is a misconfiguration — drop one. Code-reviewer finding `g7-cr-1`.
- **Thrown at:** Engine builder
- **Phase:** 1

### E_SUBSYSTEM_DISABLED

- **Message:** "Subsystem disabled: {subsystem}"
- **Context:** `{ subsystem: "ivm" | "caps" }`
- **Fix:** A thin engine configured with `.without_ivm()` or `.without_caps()` refuses operations that require the disabled subsystem — the "honest no" boundary. Either rebuild the engine without the opt-out, or restructure the caller to avoid the dependent surface.
- **Thrown at:** Engine operations (`read_view`, `grant_capability`, `create_view`, …)
- **Phase:** 1

### E_UNKNOWN_VIEW

- **Message:** "Unknown view: {view_id}"
- **Context:** `{ view_id: string, registered: string[] }`
- **Fix:** The view id was not registered. From TypeScript use `engine.createView(viewDef)`; from Rust use `Engine::create_view` (or the built-in views wired at engine-build time). Check spelling, confirm the IVM subscriber has the view wired, and that `.without_ivm()` was not used on the builder.
- **Thrown at:** Engine (`read_view`)
- **Phase:** 1

### E_NOT_IMPLEMENTED

- **Message:** "Not implemented in Phase 1: {feature}"
- **Context:** `{ feature: string, target_phase: number }`
- **Fix:** The engine method is a typed-todo that is wired for Phase 2+ evaluator integration. Avoid the surface in Phase-1 code or pick an equivalent Phase-1-landed alternative. See the per-method rustdoc for the target phase.
- **Thrown at:** Engine (primitive-dispatch surfaces)
- **Phase:** 1

### E_IVM_PATTERN_MISMATCH

- **Message:** "IVM view query pattern does not match any maintained index: {detail}"
- **Context:** `{ view_id: string, detail: string }`
- **Fix:** The caller asked a view for an index partition it doesn't maintain. Each of the five Phase-1 views keys on a specific field and rejects queries that omit it:
  - `capability_grants` requires `entity_cid`
  - `event_dispatch` requires `event_name`
  - `content_listing` accepts `label` (optional — omitted returns full listing; a non-matching label is rejected)
  - `governance_inheritance` requires `entity_cid`
  - `version_current` requires `anchor_id`
  Consult the view's maintained-pattern list and restrict the `ViewQuery` to supported keys. Distinct from `E_INV_REGISTRATION` — the view is healthy; the query shape is wrong.
- **Thrown at:** IVM view read (`View::read` on any of the five Phase-1 views)
- **Phase:** 1

### E_IVM_STRATEGY_NOT_IMPLEMENTED

- **Message:** "IVM strategy `{strategy:?}` is reserved but not implemented in this phase (deferred to {deferred_to_phase})"
- **Context:** `{ strategy: "A" | "B" | "C", deferred_to_phase: string }`
- **Fix:** Phase 2b ships `Strategy::A` (the 5 Phase-1 hand-written views) + `Strategy::B` (the generalized Algorithm B). `Strategy::C` (Z-set / DBSP cancellation) is reserved for Phase 3+ — the variant exists so the catalog of options is complete and stable, but constructing a `Strategy::C` view via `benten_ivm::testing::try_construct_view_with_strategy` returns this typed error rather than silently falling back. Pick `Strategy::B` for new user-registered views; pick `Strategy::A` for the 5 hand-written baselines (Rust-only, defaults applied automatically).
- **Thrown at:** IVM view registration (`benten_ivm::testing::try_construct_view_with_strategy`)
- **Phase:** 2b (introduced)

### E_VERSION_UNKNOWN_PRIOR

- **Message:** "Prior head was never observed by this anchor: {supplied}"
- **Context:** `{ supplied: CidV1 }`
- **Fix:** Surfaces from the prior-head-threaded `benten_core::version::append_version` when the caller names a `prior_head` that is neither the anchor's root head nor any new_head from a previous successful append. Re-read the anchor's current head (`walk_versions`) and retry against the observed head. Distinct from `E_VERSION_BRANCHED` (which fires when two appends race the same legitimate prior).
- **Thrown at:** Version-chain `append_version`
- **Phase:** 1

## TypeScript binding-layer errors

### E_DSL_INVALID_SHAPE

- **Message:** "DSL value does not match expected shape: {reason}"
- **Context:** `{ reason: string, received: unknown }`
- **Fix:** Check the DSL API documentation for the expected shape.
- **Thrown at:** DSL wrapper (TypeScript layer, before engine call)
- **Phase:** 1 (TS-only — never surfaces from the Rust engine)

### E_DSL_UNREGISTERED_HANDLER

- **Message:** "No handler registered for '{handler_id}'"
- **Context:** `{ handler_id: string, suggestions: string[] }`
- **Fix:** Check spelling; register via `engine.registerSubgraph(handler)` or `engine.registerSubgraph(crud('<label>'))`.
- **Thrown at:** DSL wrapper
- **Phase:** 1 (TS-only — never surfaces from the Rust engine)

### E_HOST_NOT_FOUND

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 3 sync.

<!-- reachability: ignore -->

- **Message:** "Host-boundary lookup miss: {kind} {identifier}"
- **Context:** `{ kind: string, identifier: string }`
- **Fix:** Reserved HostError discriminant. Surfaces from `PrimitiveHost` impls when the requested entity is not in the backend. Distinct from `E_NOT_FOUND` because it carries the host-layer boundary (preserves the `benten-eval` → `benten-graph` arch-1 dep break).
- **Thrown at:** `PrimitiveHost` implementation (G1-B)
- **Phase:** 2a (shape reserved; first firing site in Phase 3 sync — drift-detector reachability is `ignore` until then)

### E_HOST_WRITE_CONFLICT

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 3 sync.

<!-- reachability: ignore -->

- **Message:** "Host-boundary optimistic-concurrency conflict on {target}"
- **Context:** `{ target: string }`
- **Fix:** Reserved HostError discriminant. Fires when a host-level compare-and-swap write detects a concurrent mutation. Surface is frozen at Phase 2a; first firing site in Phase 3 sync.
- **Thrown at:** `PrimitiveHost` implementation (G1-B)
- **Phase:** 2a (reserved — fires in Phase 3; drift-detector reachability is `ignore` until then)

### E_HOST_BACKEND_UNAVAILABLE

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 3 sync.

<!-- reachability: ignore -->

- **Message:** "Host-boundary backend unavailable: {detail}"
- **Context:** `{ detail: string }`
- **Fix:** Reserved HostError discriminant. Fires when the underlying storage backend is offline (I/O error, disk full, network partition). Retry with exponential backoff; if persistent, inspect the storage layer.
- **Thrown at:** `PrimitiveHost` implementation (G1-B)
- **Phase:** 2a (reserved — fires in Phase 3; drift-detector reachability is `ignore` until then)

### E_HOST_CAPABILITY_REVOKED

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 3 sync.

<!-- reachability: ignore -->

- **Message:** "Host-boundary capability was revoked mid-operation"
- **Context:** `{ grant_cid: Cid }`
- **Fix:** Reserved HostError discriminant. Fires when a host-level capability check observes a revocation between resolve and use. Retry after re-granting.
- **Thrown at:** `PrimitiveHost` implementation (G1-B)
- **Phase:** 2a (reserved — fires in Phase 3; drift-detector reachability is `ignore` until then)

### E_HOST_CAPABILITY_EXPIRED

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 3 sync.

<!-- reachability: ignore -->

- **Message:** "Host-boundary capability expired by TTL"
- **Context:** `{ grant_cid: Cid, expired_at: string }`
- **Fix:** Reserved HostError discriminant. Fires when a host-level capability check observes the grant's TTL has elapsed. Re-grant with a longer TTL or refresh the cap.
- **Thrown at:** `PrimitiveHost` implementation (G1-B)
- **Phase:** 2a (reserved — fires in Phase 3; drift-detector reachability is `ignore` until then)

### E_EXEC_STATE_TAMPERED

- **Message:** "ExecutionState payload_cid mismatch — envelope tampered"
- **Context:** `{ expected_cid: Cid, actual_cid: Cid }`
- **Fix:** The resume envelope's `payload_cid` recomputation does not match the declared CID. Either the bytes were tampered in transit, or the Phase-2a serialization layer drifted. Verify the source of the bytes; never resume from untrusted storage without an integrity check.
- **Thrown at:** `Engine::resume_from_bytes` (G3-A resume protocol step 1)
- **Phase:** 2a

### E_RESUME_ACTOR_MISMATCH

- **Message:** "Resume principal does not match the suspended ExecutionState"
- **Context:** `{ suspended_actor_cid: Cid, resuming_actor_cid: Cid }`
- **Fix:** The caller attempting `resume_from_bytes_as` does not match the actor recorded at suspend time. Only the same principal (or an equivalent delegated grant) can resume. Verify the caller identity; use `resume_from_bytes` only on the original actor's behalf.
- **Thrown at:** `Engine::resume_from_bytes_as` (G3-A resume protocol step 2)
- **Phase:** 2a

### E_RESUME_SUBGRAPH_DRIFT

- **Message:** "Pinned subgraph CID drifted from the currently registered head"
- **Context:** `{ pinned_cid: Cid, current_cid: Cid, handler_id: string }`
- **Fix:** The subgraph the caller suspended against has since been re-registered under a new CID. Resumption deliberately refuses to cross that boundary. If the drift is expected, re-suspend under the new CID. Distinct from `E_INV_IMMUTABILITY` — the drift is detected at resume time, not write time.
- **Thrown at:** `Engine::resume_from_bytes` (G3-A resume protocol step 3)
- **Phase:** 2a

### E_WAIT_TIMEOUT

- **Message:** "WAIT deadline elapsed before a resume signal arrived"
- **Context:** `{ handler_id: string, node_id: NodeId, deadline_ms: number }`
- **Fix:** A WAIT declared `duration: <ms>` and the deadline elapsed without a matching signal. Either the orchestrator that was meant to resume the suspension never dispatched, or the deadline was too tight. Re-call with a longer duration, or wire a fallback ON_ERROR edge to downstream compensation logic.
- **Thrown at:** WAIT executor (G3-B duration path)
- **Phase:** 2a

### E_INV_IMMUTABILITY

- **Message:** "Write would mutate a registered subgraph (Inv-13)"
- **Context:** `{ cid: Cid, attempted_authority: WriteAuthority }`
- **Fix:** Phase-2a invariant 13 — once a Node/subgraph is persisted under a CID, its bytes are immutable from user-path writes. The firing matrix has five rows (plan §9.11):

  | # | WriteAuthority / Path | Content matches registered bytes | Outcome |
  |---|---|---|---|
  | 1 | `User` | yes | `E_INV_IMMUTABILITY` — unprivileged re-put of matching bytes is a policy violation (users cannot observe dedup on system-controlled surfaces). |
  | 2 | `User` | no | `E_INV_IMMUTABILITY` — canonical unprivileged immutability violation. Vacuous under content-addressing (CID-match ⇔ bytes-match); reached from the `put_node_at_cid_for_test` backdoor only. |
  | 3 | `EnginePrivileged` (version-chain append) | yes | `Ok(cid_dedup)` — content-addressed dedup. Does NOT emit `ChangeEvent`, does NOT advance audit sequence (named Compromise "Dedup writes pure-read", sec-r1-4 / atk-3). |
  | 4 | `SyncReplica { origin_peer }` (Phase-3 sync-receive) | yes | `Ok(cid_dedup)` — same no-event + no-audit semantics as row 3. Reserved shape in 2a; wired at Phase 3 receive-path. |
  | 5 | WAIT-resume stale-pin pre-check (any authority) | (`pinned_subgraph_cids` no longer matches the anchor's CURRENT) | `E_RESUME_SUBGRAPH_DRIFT` fires BEFORE any write. Distinct code; mirrors arch-1 resume-step-3 (§9.1) in the Inv-13 matrix. |

  To change a registered subgraph, register a new handler CID; the storage is content-addressed and version-chain appends through `EnginePrivileged` dedup at row 3.
- **Thrown at:** graph write-path (G5-A, `benten-graph`); declaration-time affordance at `benten-eval::invariants::immutability` rejects WRITE primitives whose literal `target_cid` is already registered.
- **Phase:** 2a

#### Note on E_INV_SYSTEM_ZONE (already listed above)

`E_INV_SYSTEM_ZONE` is the firing code for Phase-2a Inv-11 enforcement (both registration-time literal detection and runtime TRANSFORM-constructed CID probing). The Phase-1 stopgap `E_SYSTEM_ZONE_WRITE` continues to fire at the graph write-path as the coarsest guard.

### E_INV_ATTRIBUTION

- **Message:** "Missing or malformed attribution frame (Inv-14)"
- **Context:** none — Phase-2a R6FP catch-up EH5 trimmed the previously
  documented `{ step_index, reason }` payload to match the actual Rust
  surface, which returns the discriminant only via
  `InvariantViolation::Attribution`. Threading per-step diagnostic context
  through `EvalError` / `RegistrationError` / `ErrorCode` is a Phase-2b
  refinement (post-evaluator-completion) tracked in
  `docs/future/phase-2-backlog.md` if/when operator demand surfaces. The
  catalog is the source of truth: until the structured payload exists, the
  catalog spec must not promise it.
- **Fix:** Phase-2a invariant 14: every TraceStep MUST carry an `AttributionFrame` naming the actor, handler, and capability-grant CIDs. A primitive-type that refuses to declare its attribution source fails at registration. File a bug against the primitive's `attribution_for_step` impl.
- **Thrown at:** registration + runtime trace emission (G5-B)
- **Phase:** 2a

### E_CAP_WALLCLOCK_EXPIRED

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 2b. `CapError::WallclockExpired` is the upstream alias; the firing site is reserved at G9-A refresh-point-5 and is not yet wired (see `.addl/phase-2a/00-implementation-plan.md` G9-A residuals).

<!-- reachability: ignore -->

- **Message:** "Capability wall-clock refresh bound breached"
- **Context:** `{ elapsed_ms: number, bound_ms: number }`
- **Fix:** A long-running ITERATE crossed the 300s default wall-clock refresh boundary; the grant was revoked between the previous refresh and the boundary. Re-grant the capability and retry. Tighten handler shapes to stay under the refresh bound if latency matters.
- **Thrown at:** evaluator (G9-A, §9.13 refresh point #5). `CapError::WallclockExpired` is the upstream alias; the firing site is reserved at G9-A refresh-point-5 and is not yet wired in production code (drift-detector reachability is `ignore` until then).
- **Phase:** 2a

### E_CAP_CHAIN_TOO_DEEP

- **Message:** "Capability attenuation chain exceeds max_chain_depth"
- **Context:** `{ depth: number, limit: number }`
- **Fix:** A delegation chain was deeper than the configured `GrantReader::max_chain_depth` (default 64). Either shorten the chain or raise the configured cap through the engine builder. Ucca-6 guard against malicious delegator attacks.
- **Thrown at:** capability policy attenuation walker (G9-A)
- **Phase:** 2a

### E_CAP_SCOPE_LONE_STAR_REJECTED

- **Message:** "GrantScope::parse('*') rejected — lone star is a footgun"
- **Context:** `{ input: string }`
- **Fix:** Lone `*` is refused because it collapses to a root-scope wildcard that cannot be meaningfully attenuated. Use a compound form (`*:<namespace>`) or name an explicit scope. Ucca-7 / G4-A.
- **Thrown at:** `GrantScope::parse` (G4-A)
- **Phase:** 2a

### E_VIEW_STRATEGY_A_REFUSED

- **Message:** "user view '{view_id}' declared Strategy::A — Strategy A is reserved for the 5 hand-written Phase-1 IVM views (Rust-only); user views must use Strategy::B"
- **Context:** `{ view_id: string }`
- **Fix:** D8-RESOLVED (Phase 2b). Strategy A is the hand-written-IVM lane reserved for the five Phase-1 baseline views (capability-grants, event-dispatch, content-listing, governance-inheritance, version-current). User-registered views go through generalized Algorithm B; either omit the `strategy` field (defaults to `B`) or pass `Strategy::B` explicitly.
- **Thrown at:** `Engine::create_view` registration (G8-B)
- **Phase:** 2b

### E_VIEW_STRATEGY_C_RESERVED

- **Message:** "user view '{view_id}' declared Strategy::C — Strategy C (Z-set / DBSP cancellation) is reserved for Phase 3+"
- **Context:** `{ view_id: string }`
- **Fix:** D8-RESOLVED (Phase 2b). Strategy C is the Z-set / DBSP cancellation algorithm slot reserved for Phase 3+; refused at registration time in Phase 2b. Use `Strategy::B` (or omit the field; user views default to B).
- **Thrown at:** `Engine::create_view` registration (G8-B)
- **Phase:** 2b

### E_VIEW_LABEL_MISMATCH

- **Message:** "user view '{view_id}' is reserved for the canonical IVM view with the hardcoded label '{expected_label}'; cannot register with a different label '{got_label}'"
- **Context:** `{ view_id: string, expected_label: string, got_label: string }`
- **Fix:** Phase-2b R6-R3 (r6-r3-ivm-1). Four canonical Phase-1 IVM view ids (`capability_grants`, `version_current`, `event_dispatch`, `governance_inheritance`) have hardcoded `input_pattern_label` semantics in the hand-written `AlgorithmBView::for_id` dispatch arms — re-using one of those ids with a different label silently registers a view that filters on the wrong label. Either pick a different `spec.id` (the user-defined fallback honors any label) OR change `spec.inputPattern.label` to match the hardcoded value listed in the message body.
- **Thrown at:** `Engine::register_user_view` registration (R6-R3 fix-pass; mirrored at the TS-DSL pre-napi-boundary in `packages/engine/src/views.ts::validateUserViewSpec`).
- **Phase:** 2b

### E_WAIT_SIGNAL_SHAPE_MISMATCH

> **⚠️ Not firing in production.** Reserved at Phase-2a; first firing site lands in Phase 2b (alongside the broader G3-B DX signal-payload typing landing). Integration test at `crates/benten-engine/tests/integration/wait_signal_shape_optional_typing.rs` exercises the surface; the production firing site is reserved.

<!-- reachability: ignore -->

- **Message:** "WAIT signal payload does not match declared signal_shape"
- **Context:** `{ node_id: NodeId, expected: string, got: unknown }`
- **Fix:** When a WAIT declares `signal_shape: Some(schema)`, a resume with a payload that fails schema validation is rejected BEFORE any downstream TRANSFORM runs. Either widen the schema, re-send with the correct shape, or drop the `signal_shape` to keep the untyped path.
- **Thrown at:** WAIT executor resume path (G3-B DX signal-payload typing). The integration test at `crates/benten-engine/tests/integration/wait_signal_shape_optional_typing.rs` exercises the surface; the production firing site is reserved alongside the broader G3-B DX typing landing (drift-detector reachability is `ignore` until then).
- **Phase:** 2a

### E_WAIT_SUSPENDED

- **Message:** "WAIT primitive suspended awaiting external signal/duration"
- **Context:** `{ state_cid: Cid, signal: string }`
- **Fix:** A regular `engine.call(handler, ...)` walk hit a WAIT primitive and the dispatcher routed through the eval-side `wait::evaluate`, producing a `SuspendedHandle`. This is a control-flow signal, NOT a runtime failure — the caller catches the typed error, inspects the carried `SuspendedHandle`, and either calls `Engine::call_with_suspension` (which surfaces the same boundary as `SuspensionOutcome::Suspended`) or persists the handle bytes via `Engine::suspend_to_bytes` for later resume. Phase-2b Wave-8i (option B closure of the WAIT regular-walk dispatcher gap surfaced by the docs-vs-code audit).
- **Thrown at:** `benten_eval::primitives::dispatch` (WAIT arm), surfaced as `EvalError::WaitSuspended`; round-trips through `eval_error_to_engine_error` to `EngineError::WaitSuspended { handle }` at the engine boundary.
- **Phase:** 2b

### E_STREAM_BACKPRESSURE_DROPPED

- **Message:** "STREAM lossy mode dropped a chunk on a saturated buffer"
- **Context:** `{ seq: u64, capacity: usize }`
- **Fix:** STREAM was created with lossy semantics (`try_send` on a full buffer drops rather than awaits). The drop fires loudly via the trace surface — never silent. Either switch to lossless `send`, increase the sink capacity, or pace the producer. D4-RESOLVED. Phase-2b G6-A.
- **Thrown at:** `benten_eval::chunk_sink::BoundedSink::try_send` (lossy variant); evaluator emits a `TraceStep::BudgetExhausted { budget_type: "stream_backpressure" }` row BEFORE propagating the typed error per the D1 trace-preservation pattern.
- **Phase:** 2b

### E_STREAM_CLOSED_BY_PEER

- **Message:** "STREAM consumer disconnected; producer cannot deliver chunk"
- **Context:** `{ seq: u64 }`
- **Fix:** The downstream `ChunkSource` was dropped (consumer detached, transport closed) before the producer's next send arrived. Resume the consumer, or terminate the producer. D4-RESOLVED. Phase-2b G6-A.
- **Thrown at:** `benten_eval::chunk_sink::BoundedSink::send` / `try_send`.
- **Phase:** 2b

### E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED

- **Message:** "STREAM producer wallclock budget elapsed while awaiting available capacity"
- **Context:** `{ elapsed_ms: u64, budget_ms: u64 }`
- **Fix:** A lossless STREAM producer was created with a wallclock budget (`make_chunk_sink_with_wallclock`) and the budget elapsed while a slow consumer kept the buffer full. Either widen the budget, increase capacity, accelerate the consumer, or accept lossy mode. Kills permanently-stalled sends per streaming-systems implementation hint. D4-RESOLVED. Phase-2b G6-A.
- **Thrown at:** `benten_eval::chunk_sink::BoundedSink::send` (wallclock-budgeted variant).
- **Phase:** 2b

### E_SUBSCRIBE_DELIVERY_FAILED

- **Message:** "SUBSCRIBE delivery failed (capability re-check denied at delivery)"
- **Context:** `{ subscriber_id: SubscriberId, anchor_cid: Cid }`
- **Fix:** D5-RESOLVED requires capability re-intersection at every delivery boundary. A previously-granted READ cap was revoked mid-stream; the subscription auto-cancels. Re-grant the cap and re-register the subscription. Phase-2b G6-A.
- **Thrown at:** `benten_eval::primitives::subscribe::ActiveSubscription::inject` (delivery-time cap re-check).
- **Phase:** 2b

### E_SUBSCRIBE_PATTERN_INVALID

- **Message:** "SUBSCRIBE pattern is malformed (empty pattern, unclosed glob bracket, etc.)"
- **Context:** `{ pattern: string }`
- **Fix:** Pattern shape failed validation at registration. Fix the glob (balance `[` / `]`), provide a non-empty pattern, or switch from `LabelGlob` to `AnchorPrefix`. Phase-2b G6-A.
- **Thrown at:** `benten_eval::primitives::subscribe::ChangePattern::validate` (registration entry).
- **Phase:** 2b

### E_SUBSCRIBE_CURSOR_LOST

- **Message:** "SUBSCRIBE cursor lost (retention window exhausted mid-stream)"
- **Context:** `{ subscriber_id: SubscriberId, delivered_count: usize }`
- **Fix:** D5 strengthening item 4 caps persistent-cursor retention at 1000 events OR 24h, whichever first. Beyond the bound, the subscription auto-cancels and the subscriber must restart from `Latest`. Adjust event-emission rate, drain promptly, or accept the bounded-replay contract. Phase-2b G6-A.
- **Thrown at:** `benten_eval::primitives::subscribe::ActiveSubscription::inject` (mid-stream retention check).
- **Phase:** 2b

### E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED

- **Message:** "SUBSCRIBE persistent cursor restart attempted past the retention window"
- **Context:** `{ subscriber_id: SubscriberId }`
- **Fix:** Equivalent surface to `E_SUBSCRIBE_CURSOR_LOST` raised at re-registration time rather than mid-stream. The persisted `max_delivered_seq` falls outside the retained event window; re-register with `start_from: Latest` to resume from the next published event. streaming-systems stream-d5-1. Phase-2b G6-A.
- **Thrown at:** `benten_eval::primitives::subscribe::register_inner` (`Persistent` cursor re-registration).
- **Phase:** 2b

### E_INV_11_SYSTEM_ZONE_READ

- **Message:** "SUBSCRIBE pattern names a `system:*` zone (Inv-11)"
- **Context:** `{ pattern: string }`
- **Fix:** User code attempted to subscribe to a `system:*` system-zone label. Distinct catalog code so SUBSCRIBE-side breaches are diagnostically separable from WRITE-side breaches (`E_INV_SYSTEM_ZONE` covers writes). Subscribe to a non-system pattern, or, for engine-internal observation, use a privileged path. Phase-2b G6-A.
- **Thrown at:** `benten_eval::primitives::subscribe::ChangePattern::validate` (registration entry).
- **Phase:** 2b
## Phase 2b G7-A SANDBOX surface

<!-- E_INV_SANDBOX_DEPTH: see canonical entry at the Inv-4/Inv-7 G7-B section above -->

### E_SANDBOX_FUEL_EXHAUSTED

- **Message:** "SANDBOX fuel exhausted: limit={limit} consumed={consumed}"
- **Context:** `{ limit: u64, consumed: u64 }`
- **Fix:** wasmtime fuel-meter intercept. Either reduce the per-call computation, raise `SandboxConfig::fuel` (default 1_000_000), or split the workload across multiple SANDBOX calls. Concurrent with the typed-error propagation, the engine emits `TraceStep::BudgetExhausted { budget_type: "sandbox_fuel", consumed, limit, path }` so `engine.trace(...)` consumers observe the exhaustion in-band (mirrors G12-A's `inv_8_iteration` pattern).
- **Thrown at:** SANDBOX executor — fully active post-wave-8b. The wasmtime `Store::set_fuel` cap + trap-callback maps fuel-exhaustion traps via `crates/benten-eval/src/sandbox/trap_to_typed.rs` to this typed variant. D3-RESOLVED per-call wasmtime `Store` lifecycle.
- **Phase:** 2b G7-A (variant) / wave-8b (production trap-mapping)
- **D21 priority:** fires before `E_INV_SANDBOX_OUTPUT` when both trip; loses to `E_SANDBOX_WALLCLOCK_EXCEEDED` / `E_SANDBOX_MEMORY_EXHAUSTED` (D21 priority MEMORY > WALLCLOCK > FUEL > OUTPUT).

### E_SANDBOX_MEMORY_EXHAUSTED

- **Message:** "SANDBOX memory limit exhausted: {limit} bytes"
- **Context:** `{ limit: u64 }`
- **Fix:** wasmtime `ResourceLimiter` intercept fires deterministically BEFORE host OOM (`crates/benten-eval/src/sandbox/resource_limiter.rs`). Either reduce module memory pressure, raise `SandboxConfig::memory_bytes` (default 64 MiB), or audit for runaway `memory.grow` (ESC-2 escape vector).
- **Thrown at:** SANDBOX executor — fully active post-wave-8b via `ResourceLimiter` impl + memory-trap → typed-error mapping.
- **Phase:** 2b G7-A (variant) / wave-8b (production ResourceLimiter wiring)
- **D21 priority:** HIGHEST — fires before `E_SANDBOX_WALLCLOCK_EXCEEDED` / `E_SANDBOX_FUEL_EXHAUSTED` / `E_INV_SANDBOX_OUTPUT` when multiple are simultaneously eligible (D21 priority MEMORY > WALLCLOCK > FUEL > OUTPUT — matches OS-level OOM trump).

### E_SANDBOX_WALLCLOCK_EXCEEDED

- **Message:** "SANDBOX wallclock deadline exceeded: {limit_ms} ms"
- **Context:** `{ limit_ms: u64 }`
- **Fix:** D24-RESOLVED defaults: 30s default / 5min ceiling. Per-handler `wallclock_ms` opt-in via `SubgraphSpec.primitives` (G12-D widening). Workspace-level overrides via `engine.toml` `[sandbox]` section (Ben's brief addition). Either shrink the workload, raise the per-handler value (within the engine.toml ceiling), or relax the engine.toml ceiling.
- **Thrown at:** SANDBOX executor — fully active post-wave-8b via `wasmtime::Store::set_epoch_deadline` + the wave-8b epoch-interruption ticker thread (`crates/benten-eval/src/sandbox/epoch_ticker.rs`) that ticks the shared engine's epoch on a configured cadence; D27 `async-support` ENABLED preserves the yield path for Phase-3 iroh forward-compat.
- **Phase:** 2b G7-A (variant) / wave-8b (production epoch-ticker wiring)
- **D21 priority:** fires before `E_SANDBOX_FUEL_EXHAUSTED` / `E_INV_SANDBOX_OUTPUT` when multiple trip; loses to `E_SANDBOX_MEMORY_EXHAUSTED` (D21 priority MEMORY > WALLCLOCK > FUEL > OUTPUT).

### E_SANDBOX_WALLCLOCK_INVALID

- **Message:** "SANDBOX wallclock setting outside allowed range"
- **Context:** `{ requested_ms: u64, max_ms: u64 }`
- **Fix:** Per-handler `wallclock_ms` must be > 0 and ≤ engine.toml `wallclock_max_ms` (defaults to D24-RESOLVED 5min ceiling). Reduce the per-handler value or relax `wallclock_max_ms` in `engine.toml`.
- **Thrown at:** SubgraphSpec validation / `SandboxConfig::with_wallclock_ms`.
- **Phase:** 2b G7-A

### E_SANDBOX_HOST_FN_DENIED

- **Message:** "SANDBOX host-fn capability denied: {cap}"
- **Context:** `{ cap: string, host_fn_name: string, recheck: "per_call" \| "per_boundary" }`
- **Fix:** Two firing paths: (1) D7 init-snapshot intersection — manifest claims a cap the dispatching grant lacks; fail before module link. (2) D18 per_call live recheck — cap revoked mid-call; subsequent host-fn invocation denied. Surfaces as a typed error THROUGH the host-fn ABI (NOT a wasmtime trap per sec-r1 D7) so the engine accounting stays clean. Either grant the missing cap, change the manifest, or relax the host-fn's `cap_recheck` declaration.
- **Thrown at:** SANDBOX executor (init-time intersection; per-invocation re-check per D18 cadence).
- **Phase:** 2b G7-A

### E_SANDBOX_HOST_FN_NOT_FOUND

- **Message:** "SANDBOX host-fn not found: {name}"
- **Context:** `{ name: string }`
- **Fix:** Module attempted to call a host-fn name not in the active manifest. Phase-3 G17-A2 retired the Phase-2b `random`-host-fn deferral guard (CLAUDE.md baked-in #16 closure); `random` is now LIVE alongside `time` / `log` / `kv:read` (cap-string `host:random:read`). For names that fire this code post-G17-A2: check the manifest declaration matches the import + the codegen-default surface (4 host-fns at G17-A2). The wasmtime link-time resolver path fires when wasmtime fails to resolve an import against the linker.
- **Thrown at:** SANDBOX executor — wasmtime link-time resolver (other names than the 4 codegen-default).
- **Phase:** 2b G7-A (variant) / wave-8b (production wiring) / Phase-3 G17-A2 (deferral guard retired)

### E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED

- **Message:** "SANDBOX random host-fn per-call entropy budget exceeded: requested={n} budget={n}"
- **Context:** `{ requested_bytes: u64, budget_bytes: u64 }`
- **Fix:** Phase-3 G17-A2 (CLAUDE.md baked-in #16 closure). A single `host.random(ptr, len)` call requested more entropy bytes than the per-call budget allows. The codegen default is **4096 bytes per call** (per r1-wsa-8). To draw more entropy, either (a) split the request across multiple sub-budget calls, or (b) override the default per-manifest via the additive optional `host_fns.random.budget_bytes_per_call` field on `ModuleManifest`. The aggregate-per-primitive cap is enforced separately at `CountedSink` (via `output_bytes`); the per-call budget is the additional ceiling on a single invocation. Routes through the `ON_DENIED` family (cap-denial precedent).
- **Thrown at:** `register_default_host_fns` "random" trampoline at `crates/benten-eval/src/primitives/sandbox.rs::register_default_host_fns`. The `HostFnDenialMarker` carrier identifies the denial via the `random:per_call_budget_exceeded (requested=<n>, budget=<n>)` cap-string.
- **Phase:** Phase-3 G17-A2 wave-5b

### E_SANDBOX_MANIFEST_UNKNOWN

- **Message:** "SANDBOX manifest name '{manifest_name}' is not registered (codegen defaults: compute-basic, compute-with-kv; install via `engine.installModule(...)` or use a different name)"
- **Context:** `{ manifestName: string }` (Phase-3 G17-C wave-5b structured-context surface; pre-G17-C the variant carried only the message string).
- **Fix:** ESC-15 escape vector closure: NO permissive fall-through to a default manifest. Either install the manifest via `Engine::install_module` (paired with `Engine::register_module_bytes` for the underlying wasm payload) or use one of the codegen-default names (`compute-basic`, `compute-with-kv`). Phase-3 G17-C wave-5b adds the registration-time validation walk in `Engine::register_subgraph` so misspelled names + post-uninstall residual references trip THIS error at register time (operator-actionable: the wallclock-after-zero-progress masking is gone) instead of at dispatch time as a confusing wallclock trip.
- **Thrown at:**
  - **Registration time (Phase-3 G17-C):** `Engine::register_subgraph::validate_sandbox_manifest_names` — walks SANDBOX nodes for unresolved manifest references via either the explicit `manifest` property or the colon-joined `<manifest>:<entry>` `module` property fallback.
  - **Dispatch time (legacy):** `ManifestRegistry::lookup` / `ManifestRef::resolve` — preserved for non-DSL spec construction paths that bypass the validation walk.
- **Phase:** 2b G7-A (dispatch-time path); 3 G17-C (registration-time validation walk + structured-context surface).

### E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED

- **Message:** "Runtime manifest registration deferred to Phase 8"
- **Context:** `{ name: string }`
- **Fix:** D2-RESOLVED hybrid: `ManifestRegistry::register_runtime(name, bundle)` exists in Phase 2b but returns this typed error (the API surface is reserved so Phase-8 marketplace work doesn't introduce a new public API — it just changes the body). Use a codegen-default manifest in 2b; revisit when Phase 8 ships.
- **Thrown at:** `ManifestRegistry::register_runtime`.
- **Phase:** 2b G7-A (deferral surface); Phase 8 (lift).

### E_SANDBOX_MODULE_INVALID

- **Message:** "SANDBOX module invalid: {reason}"
- **Context:** `{ reason: string }`
- **Fix:** Module bytes failed wasmtime structural validation (malformed module, type mismatch, OOB section, OOB linear-memory read, recursion-depth overflow, etc.). Audit the module compiler output. ESC-1 / ESC-3 / ESC-5 / ESC-11 / ESC-12 escape vectors all route here.
- **Thrown at:** SANDBOX executor (`Module::new` / link / instantiation).
- **Phase:** 2b G7-A

### E_SANDBOX_STACK_OVERFLOW

- **Message:** "SANDBOX stack overflow: guest exceeded max_wasm_stack ({max_wasm_stack} bytes)"
- **Context:** `{ max_wasm_stack: u64 }`
- **Fix:** SANDBOX guest module's call stack exceeded the configured `max_wasm_stack` ceiling (default 512 KiB; matches wasmtime's `Config::max_wasm_stack` default). Distinct from `E_SANDBOX_FUEL_EXHAUSTED` (CPU-bound runaway) and `E_SANDBOX_MODULE_INVALID` (structural validation failure) — stack-overflow-via-recursion is its own observable class so operator dashboards can distinguish a benign-but-buggy recursive guest from a generic invalid module. Either reduce module recursion depth, raise `SandboxConfig::max_wasm_stack`, or audit for adversarial recursion. Phase-3 G17-A1 wave-5b mints the dedicated typed variant per phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure (the prior R6FP-G1 r6-wsa-8 BELONGS-NAMED-NOW deferral is honored here).
- **Thrown at:** SANDBOX executor — `wasmtime::Trap::StackOverflow` routes through `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error` to the dedicated variant.
- **Phase:** Phase-3 G17-A1 wave-5b

### E_SANDBOX_ESCAPE_ATTEMPT

- **Message:** "SANDBOX escape attempt detected: {vector:?} — {reason}"
- **Context:** `{ vector: EscVector, reason: string }`
- **Fix:** SANDBOX guest attempted one of the enumerated escape vectors. Phase-3 G17-A1 wave-5b ships defenses for **ESC-7** (fuel-refill via host-fn re-entry — guest calls a host-fn whose dispatch path attempts to re-enter the SANDBOX `Store` and `add_fuel` mid-execution; defense fires from the trampoline before the inner `add_fuel` takes effect), **ESC-13** (trap during fuel-meter callback / Store-poison — host-side fuel-meter callback panics or traps; defense maps via panic-catcher + per-call `Store` lifecycle ensures fresh Store on next call), and **ESC-16** (fingerprint-collapse via wallclock-correlated state read — guest reads a host-written wallclock-derived cell to fingerprint host nondeterminism; defense fires at the next host-fn boundary BEFORE the side-channel becomes guest-observable). The discriminating `EscVector` enum (declared in `crates/benten-eval/src/sandbox/escape_defenses.rs`) carries `Esc7FuelRefillViaReEntry` / `Esc13StorePoison` / `Esc16FingerprintCollapse` variants so audit pipelines can route per-vector. Closes r1-wsa-1 BLOCKER (ESC-7 + ESC-13) + r1-wsa-4 (ESC-16) per phase-3-backlog §6.1 + D-E (R1 revision triage). Either harden the guest module (audit for the enumerated attack patterns) or — if the attack is in a research / test corpus — gate the corpus dispatch behind explicit testing-helper feature flags.
- **Thrown at:** SANDBOX executor — `crates/benten-eval/src/sandbox/escape_defenses.rs::run_all_checks` (and per-vector `run_esc7_check` / `run_esc13_check` / `run_esc16_check`); routes through `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error` via the `EscapeAttemptMarker` cause-chain unwrap.
- **Phase:** Phase-3 G17-A1 wave-5b

### E_SANDBOX_MODULE_NOT_INSTALLED

- **Message:** "SANDBOX module bytes not registered for CID {module_cid}"
- **Context:** `{ module_cid: Cid }`
- **Fix:** A SANDBOX dispatch named a module CID for which no bytes have been registered through `Engine::register_module_bytes(cid, bytes)`. Distinct from `E_SANDBOX_MODULE_INVALID` (bytes are present but failed wasmtime structural validation): this fires BEFORE the executor sees any bytes, at the engine's lookup step. Either call `engine.register_module_bytes(module_cid, wasm_bytes)` before dispatch, or correct the SANDBOX node's `module` property to reference an already-registered CID. The Phase-2b in-memory module-bytes registry is process-local + transient (lost across `Engine` re-open); Phase 3 promotes the registry to a durable `BlobBackend` per Compromise #17. The `install_module(manifest, expected_cid)` path persists the manifest into a system-zone Node but does NOT persist the underlying wasm bytes — that asymmetry IS the Compromise #17 narrative.
- **Thrown at:** `impl PrimitiveHost for Engine::execute_sandbox` (`crates/benten-engine/src/primitive_host.rs`) when `Engine::module_bytes_for(cid)` returns `None`.
- **Phase:** 2b Wave-8d-types

### E_SANDBOX_NESTED_DISPATCH_DENIED

- **Message:** "SANDBOX nested dispatch denied"
- **Context:** `{ host_fn_name: string }`
- **Fix:** D19-RESOLVED: deny nested `Engine::call` from host-fn (the actual security claim). Closes the SANDBOX → CALL → SANDBOX cap-context-confusion attack class (sec-pre-r1-08). Renamed from the older `E_SANDBOX_REENTRANCY_DENIED` per wsa-7 + r1-security convergence — the name aligns with what's actually being denied. Refactor the host-fn to NOT re-enter the engine; if Phase-3 async host-fns are needed, acquire the reserved `host:async` cap.
- **Thrown at:** SANDBOX executor — fully active post-wave-8b. The host-fn callback path enforces the no-nested-`Engine::call` invariant via the trampoline's typed-error short-circuit before the host-side body runs.
- **Phase:** 2b G7-A (variant) / wave-8b (production wiring)

<!-- E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED: see canonical entry at the Inv-4/Inv-7 G7-B section above -->

### E_MODULE_MANIFEST_CID_MISMATCH

- **Message:** "Module manifest CID mismatch: expected={expected_cid} computed={computed_cid} summary={manifest_summary}"
- **Context:** `{ expected_cid: Cid, computed_cid: Cid, manifest_summary: string }`
- **Fix:** D16-RESOLVED-FURTHER minimal CID-pin integrity gate. `Engine::install_module(manifest, expected_cid: Cid)` REQUIRES the CID arg (not Optional — prevents the lazy `install_module(m, None)` footgun). The error includes both expected + computed CIDs + a 1-line manifest summary so an operator can diff without source-code dive. Either re-compute the expected CID against the actual manifest bytes or audit for tampering. Reserved here for the G10-B `install_module` surface; G7-C does NOT own this fire site (per wsa-r1-5 plan-internal conflict resolution).
- **Thrown at:** `Engine::install_module` (G10-B).
- **Phase:** 2b G10-B

### E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE

- **Message:** "module manifest declares N migration(s) but the target has no persistent backing store"
- **Context:** `{ migration_count: usize }`
- **Fix:** `docs/SECURITY-POSTURE.md` Compromise #19 — browser (`wasm32-unknown-unknown`) engines ship in-memory-only manifest persistence in Phase 2b; the IndexedDB / OPFS persistence story lands in Phase 3. Manifests that declare `migrations` need a durable backing store; the rejection prevents the migration runner from silently dropping work. On native (redb-backed) targets the same manifest installs without error. Either (a) defer the migration to a Phase-3 build with persistent storage, or (b) split the manifest into a migrations-free in-memory variant for Phase-2b browser deployments.
- **Thrown at:** `Engine::install_module` (G10-B) on `wasm32-unknown-unknown` only.
- **Phase:** 2b G10-B

### E_ENGINE_CONFIG_INVALID

- **Message:** "engine.toml at {path} parse failure: {reason}"
- **Context:** `{ path: PathBuf, reason: string }`
- **Fix:** Workspace-level `engine.toml` (Ben's G7-A brief addition) failed to parse against the [`EngineConfig`] schema. Either fix the TOML (see `docs/SANDBOX-LIMITS.md` for the schema) or remove the file (built-in defaults apply when absent). The `[sandbox]` section accepts `wallclock_default_ms` (override D24 30s default) and `wallclock_max_ms` (override D24 5min ceiling).
- **Thrown at:** `EngineConfig::load_or_default` (called at `Engine::open` time).
- **Phase:** 2b G7-A

### E_BACKEND_READ_ONLY

- **Message:** "backend is read-only: {operation} rejected ({backend_kind})"
- **Context:** `{ operation: string, backend_kind: string }`
- **Fix:** D10-RESOLVED snapshot-blob `KVBackend` (constructed via `Engine::from_snapshot_blob(bytes)`) is a read-mostly view on a content-addressed handoff blob — Phase-3 sync can transmit the blob between peers, but the dst engine cannot write into it without breaking the canonical-bytes invariant the blob's CID is computed over. The same posture applies to the Phase-2a §9.8 `network_fetch_stub` `KVBackend`: writes will land in Phase 3 once the iroh-fetch path replaces the stub. To mutate state, open a redb-backed engine via `Engine::open(path)` instead, or import the snapshot blob into a fresh redb engine and reissue writes there.
- **Thrown at:** `SnapshotBlobBackend::{put,delete,put_batch}` (`crates/benten-graph/src/backends/snapshot_blob.rs`); `NetworkFetchStubBackend::{put,delete,put_batch}` (`crates/benten-graph/src/backends/network_fetch_stub.rs`); surfaces from `Engine::from_snapshot_blob`-constructed engines on any write call.
- **Phase:** 2b G10-A-wasip1

### E_SANDBOX_UNAVAILABLE_ON_WASM

- **Message:** "SANDBOX is unavailable on the wasm32 build of the engine ({target})"
- **Context:** `{ target: "wasm32-unknown-unknown" | "wasm32-wasip1", reason: "wasmtime cannot host nested wasm execution on this target" }`
- **Fix:** SANDBOX requires wasmtime, which does not compile to `wasm32-unknown-unknown` (browser target) and is not currently shipped on `wasm32-wasip1` engine builds either. The engine surfaces this typed error rather than `E_SUBSYSTEM_DISABLED` because the operator-actionable signal is target-specific: SANDBOX cannot run here, regardless of build flags. Phase-3 P2P sync re-routes SANDBOX invocations to a non-browser peer; until then, host SANDBOX-bearing handlers on a native `Engine::open(path)` engine and surface their results through SUBSCRIBE / STREAM to the wasm32-hosted client.
- **Thrown at:** `crates/benten-engine/src/engine_sandbox.rs::execute_sandbox_wasm32_unavailable` (wasm32 cfg-gated stub) and the SANDBOX dispatcher path in `crates/benten-eval/src/primitives/mod.rs` when reached on a wasm32 target.
- **Phase:** 2b wave-8c

### E_RELOAD_SUBSCRIBER_UNSUBSCRIBED

<!-- reachability: ignore -->
<!-- Rationale: construction site lives in `bindings/napi/src/devserver.rs::reload_subscriber_unsubscribed` (napi tooling adapter). The drift detector's reachability scanner walks `crates/*/src/` only, so a napi-side construction is a structural false negative. Remove this annotation if the scanner is widened to include `bindings/*/src/` (a Phase-3 detector improvement). -->

- **Message:** "{operation} after unsubscribe"
- **Context:** `{ operation: "drain" | "hasEvents" }`
- **Fix:** A `ReloadSubscriberJs` napi method (`drain` / `hasEvents`) was called after `unsubscribe()` released the underlying subscriber. The handle is single-shot; recreate the subscription via `devserver.subscribeReloadEvents()` if more events are expected.
- **Thrown at:** `bindings/napi/src/devserver.rs::ReloadSubscriberJs::{drain, has_events}` after `unsubscribe()` flips the inner `Mutex<Option<...>>` to `None`. R6 Round-2 r6-r2-napi-1 promoted this from a hand-typed `"E_RELOAD_SUBSCRIBER_UNSUBSCRIBED"` string to a typed catalog variant so JS callers get `EReloadSubscriberUnsubscribed` typed dispatch through `mapNativeError` rather than the synthetic `E_UNKNOWN` fallback.
- **Phase:** 2b R6 Round-2

### E_DEVSERVER_STOPPED

<!-- reachability: ignore -->
<!-- Rationale: construction site lives in `bindings/napi/src/devserver.rs::devserver_stopped` (napi tooling adapter). Same scanner asymmetry as E_RELOAD_SUBSCRIBER_UNSUBSCRIBED above. Remove this annotation if the scanner is widened to include `bindings/*/src/`. -->

- **Message:** "dev-server has been stopped — call .start() before further operations"
- **Context:** `{}`
- **Fix:** A devserver napi method was called after `DevServer.stop()` flipped the in-memory state to stopped. Restart the dev-server via `.start()` before invoking further operations, or construct a fresh `DevServer` instance.
- **Thrown at:** `bindings/napi/src/devserver.rs::devserver_stopped` (helper used by every devserver method that requires the dev-server to be running). R6 Round-2 r6-r2-napi-1 promoted this from a hand-typed `"E_DEVSERVER_STOPPED"` string to a typed catalog variant so JS callers get `EDevServerStopped` typed dispatch.
- **Phase:** 2b R6 Round-2

### E_STORAGE_QUOTA_EXCEEDED

<!-- reachability: ignore -->
<!-- Rationale: construction site lives in `bindings/napi/src/browser_indexeddb.rs::map_dom_exception_to_error_code` (browser-target IndexedDB napi adapter). The drift detector's reachability scanner walks `crates/*/src/` only, so a napi-side construction is a structural false negative. Same scanner asymmetry as E_RELOAD_SUBSCRIBER_UNSUBSCRIBED + E_DEVSERVER_STOPPED above. Remove this annotation if the scanner is widened to include `bindings/*/src/` (a Phase-3 detector improvement; tracked in `phase-3-backlog §7.11`). -->

- **Message:** "IndexedDB write exceeded origin-storage quota"
- **Context:** `{ dom_exception_name: "QuotaExceededError" }`
- **Fix:** A browser thin-client cache write to IndexedDB exceeded the origin's storage allocation (the browser's per-origin quota). The browser surfaces `DOMException(name="QuotaExceededError")` synchronously from the `IDBObjectStore.put` request's `onerror` handler; the napi binding maps this to the typed `E_STORAGE_QUOTA_EXCEEDED` variant via `bindings/napi/src/browser_indexeddb.rs::map_dom_exception_to_error_code`. Resolution is out-of-band: the user (or operator) frees origin-storage allocation by clearing site data, removing unused cached blobs, or migrating to a deployment with larger origin quota. Per CLAUDE.md baked-in #17 thin-client commitment, the browser tab's cache is non-authoritative — losing the cached bytes is recoverable: subsequent reads re-fetch from the connected full peer through the thin-client subscription protocol (D-PHASE-3-30).
- **Thrown at:** `bindings/napi/src/browser_indexeddb.rs::map_dom_exception_to_error_code` (Phase-3 G18-A wave-5a). Mapping is consumed by the IndexedDB-backed BlobBackend variant at `bindings/napi/src/browser_blob_store.rs` and the persistent module-manifest store at `bindings/napi/src/wasm_browser.rs`. Surface scope per CLAUDE.md baked-in #17: thin-client cache + manifest-store ONLY.
- **Phase:** 3 G18-A

### E_HLC_SKEW_EXCEEDED

- **Message:** "HLC skew exceeded: remote physical_ms {remote_physical_ms} > local {local_physical_ms} + tolerance {tolerance_ms}ms"
- **Context:** `{ local_physical_ms: u64, remote_physical_ms: u64, tolerance_ms: u64 }`
- **Fix:** `Hlc::update(remote)` refused a remote stamp whose physical-clock component exceeds the local physical clock by more than the configured skew tolerance (default 5 minutes per `Hlc::DEFAULT_SKEW_TOLERANCE_MS`). The local HLC state is NOT mutated when this fires — Phase-3 sync rejects the offending message and continues. Inspect peer NTP / system-clock health; legitimate cross-region drift should fit comfortably inside 5 minutes. Operator-tunable knobs land alongside Phase-3 sync wiring.
- **Thrown at:** `crates/benten-core/src/hlc.rs::Hlc::update` (Phase-3 G14-pre-D). Phase-3 sync wires the firing site into Loro per-property LWW + asymmetric-uptime MST-diff message ingest.
- **Phase:** 3 G14-pre-D

### E_CAP_UCAN_EXPIRED

- **Message:** "UCAN expired (exp={exp}, now={now})"
- **Context:** `{ exp: u64, now: u64 }`
- **Fix:** Presented UCAN's `exp` window has elapsed at chain-walk time. Re-issue the UCAN with a fresh `exp`. Defends against the "old proof sitting in disk forever, replayed by attacker who sniffed it pre-exp" attack class per `crypto-blocker-2` BLOCKER + CLR-2.
- **Thrown at:** `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B). Routes to `ON_DENIED`.
- **Phase:** 3 G14-B

### E_CAP_UCAN_NOT_YET_VALID

- **Message:** "UCAN not yet valid (nbf={nbf}, now={now})"
- **Context:** `{ nbf: u64, now: u64 }`
- **Fix:** Presented UCAN's `nbf` window has not yet opened at chain-walk time. Wait until `now >= nbf` or re-issue with an earlier `nbf`. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B).
- **Phase:** 3 G14-B

### E_CAP_UCAN_BAD_SIGNATURE

- **Message:** "UCAN signature failed verification (link_index={link_index})"
- **Context:** `{ link_index: usize }`
- **Fix:** Presented UCAN's signature failed to verify against the issuer's resolved public key. Likely tampered or signed by a different keypair than the one named in `iss`. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B). Constant-time comparison via `subtle::ConstantTimeEq` per `crypto-major-4`.
- **Phase:** 3 G14-B

### E_CAP_UCAN_ATTENUATION_VIOLATED

- **Message:** "UCAN attenuation violated: child cap '{child_cap}' is not subsumed by parent caps"
- **Context:** `{ child_cap: String, link_index: usize }`
- **Fix:** Child UCAN's capability widens its parent's authority — a structural delegation violation. Re-issue the child UCAN attenuated to a subset of the parent's `att`. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B). Composes with `benten_id::ucan::validate_chain_at` per `crypto-blocker-2`.
- **Phase:** 3 G14-B

### E_CAP_BACKEND_STORAGE

- **Message:** "UCAN backend storage I/O failure: {reason}"
- **Context:** `{ reason: String }`
- **Fix:** Durable UCAN backend failed to read or write its grant store. Surfaces a layered backend I/O failure to the policy hook caller. Inspect underlying `GraphBackend` health (redb file permissions, disk space). Distinct from `E_CAP_DENIED` — the backend cannot determine permitted-or-not when its store is unreadable. Routes to `ON_ERROR`.
- **Thrown at:** `crates/benten-caps/src/backends/ucan.rs::UCANBackend::{record_grant, record_revocation, validate_chain_with_durable_revocations}` (Phase-3 G14-B).
- **Phase:** 3 G14-B

### E_CAP_RATE_LIMIT_EXCEEDED

- **Message:** "rate-limit exceeded for actor {actor} on zone {zone}"
- **Context:** `{ actor: String, zone: String }`
- **Fix:** Per-actor writes/sec/zone bucket exceeded its budget. Configure a less restrictive `InMemoryRateLimitPolicyBuilder::actor_writes_per_second` for the actor, or back off and retry. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-caps/src/rate_limit.rs::RateLimitPolicy::check_writes_per_sec` (Phase-3 G14-B; D-F + D-PHASE-3-26).
- **Phase:** 3 G14-B

### E_CAP_PEER_BANDWIDTH_EXCEEDED

- **Message:** "peer bandwidth budget exceeded for peer {peer} ({bytes} bytes)"
- **Context:** `{ peer: String, bytes: usize }`
- **Fix:** Per-peer bandwidth bytes/sec budget at the Atrium boundary exceeded its limit. Defends against a malicious or buggy peer flooding the sync channel. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-caps/src/rate_limit.rs::RateLimitPolicy::check_peer_bandwidth` (Phase-3 G14-B; D-F + D-PHASE-3-26 + D-PHASE-3-30).
- **Phase:** 3 G14-B

### E_CAP_SNAPSHOT_HASH_MISMATCH

- **Message:** "resume: cap_snapshot_hash mismatch for actor {actor} (proof-chain changed between suspend and resume; CLR-2 §11)"
- **Context:** `{ actor: String }`
- **Fix:** A WAIT-suspended execution attempted to resume against a UCAN proof-chain that materially changed between suspend and resume (e.g. one of the chain's tokens was revoked, or the chain was substituted). Per CLR-2 §11 the resume MUST reject — silently re-running a continuation against a downgraded chain would let an attacker race a revoke with a resume. Re-issue the suspended request from a current envelope; the prior envelope is no longer authoritative. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner` Step 3.5 (Phase-3 G14-D wave-5a; CLR-2 §11 + Compromise #10 engine-side asymmetry closure). The hash is computed by `crates/benten-engine/src/cap_snapshot_hash.rs::compute(actor_cid, &proof_chain_cids)` and persisted alongside the envelope via `Engine::put_cap_snapshot_for_envelope`.
- **Phase:** 3 G14-D

### E_SUBSCRIBE_REVOKED_MID_STREAM

<!-- reachability: ignore -->
<!-- Rationale: Phase-3 G14-D wave-5a ships the catalog code + ON_DENIED routing + from_string round-trip. The production construction site (per-event delivery-time cap-recheck firing on a partial-revoke event) wires once G14-B's durable UCAN backend `chain-for-audience` accessor stabilizes through `engine.caps()`; the RED-PHASE pins at `crates/benten-engine/tests/subscribe_cap_recheck.rs` carry the wave-pairing destination per pim-4 §3.10. Remove this annotation when the F6 SUBSCRIBE per-event recheck composes against the durable grant store. -->

- **Message:** "subscribe: cap revoked mid-stream for subscriber {subscriber} on channel {channel}"
- **Context:** `{ subscriber: String, channel: String }`
- **Fix:** A SUBSCRIBE / sync-replica subscription was terminated mid-stream because the subscriber's read-coverage UCAN no longer holds — a partial revoke fired the per-event delivery-time cap-recheck on the next event. Distinct from `E_SUBSCRIBE_DELIVERY_FAILED` (transient delivery-channel failures) — this names the cap-recheck-driven termination per F6 LOAD-BEARING. Re-issue a fresh subscribe with current credentials. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-engine/src/cap_recheck.rs` per-event closure firing (Phase-3 G14-D wave-5a; F6 LOAD-BEARING + Compromise #2 D5). Wave-paired construction sites land alongside G14-B's durable UCAN backend `chain-for-audience` accessor.
- **Phase:** 3 G14-D

### E_SYNC_REVOKED_DURING_SESSION

<!-- reachability: ignore -->
<!-- Rationale: Phase-3 G14-D wave-5a ships the catalog code + ON_DENIED routing + from_string round-trip. The production construction site (sync-replica inbound WRITE rejected because the source peer's grant was revoked locally between handshake and the next sync round) wires at the sync-receive boundary in a follow-up wave; G14-D delivers the engine-side cap-recheck scaffold (`cap_recheck.rs`) and CLR-2 mirror code that the sync-receive path consumes. Remove this annotation when `benten-sync` wires the first construction site at the sync-replica WRITE boundary. -->

- **Message:** "sync: peer {peer} grant revoked during session"
- **Context:** `{ peer: String }`
- **Fix:** A sync-replica inbound WRITE was rejected because the source peer's grant was revoked locally between the Atrium handshake and the next sync round. Per CLR-2 this mirrors the SUBSCRIBE delivery-time recheck — the receiving peer's per-write cap-recheck consults the local grant store via the `cap_recheck.rs` G13-pre-C scaffold. The peer may re-handshake with a current grant. Routes to `ON_DENIED`.
- **Thrown at:** sync-replica receive boundary (Phase-3 G14-D wave-5a; sec-r4r1-2 BLOCKER half-b closure; CLR-2 mirror). Wave-paired construction site lands alongside the sync-receive surface.
- **Phase:** 3 G14-D

### E_SYNC_HOP_DEPTH_EXCEEDED

<!-- reachability: ignore -->
<!-- Rationale: Phase-3 G14-D wave-5a ships the catalog code + ON_DENIED routing + from_string round-trip. The production construction site (inbound sync-replica AttributionFrame chain exceeding the documented hop-depth bound) wires at the sync-receive validation boundary in a follow-up wave; G14-D delivers the catalog seam and the Inv-4-mirror error-code surface. Remove this annotation when `benten-sync` wires the first construction site at the chain-bound check. -->

- **Message:** "sync: chain hop depth {depth} exceeds bound {bound}"
- **Context:** `{ depth: usize, bound: usize }`
- **Fix:** An inbound sync-replica AttributionFrame chain exceeded the documented hop-depth bound (mirrors Inv-4 `sandbox_depth`). Defends against DOS/chain-bloat where an adversarial peer constructs an unbounded false chain. The peer should either issue against a shorter chain or re-handshake with a fresh authority root. Routes to `ON_DENIED`.
- **Thrown at:** sync-replica chain-bound check (Phase-3 G14-D wave-5a; ds-r4r2-2 closure). Wave-paired construction site lands alongside the sync-receive surface.
- **Phase:** 3 G14-D

### E_THIN_CLIENT_AUTH_REJECTED

- **Message:** "thin-client connect: device attestation rejected ({reason})"
- **Context:** `{ reason: String }`
- **Fix:** A thin-client (browser tab / edge-worker) connection attempt was rejected at the full-peer auth boundary because the connecting tab presented no device-attestation OR presented one bound to a revoked device-DID. Distinct from generic `E_CAP_DENIED` so audit pipelines can route on the thin-client auth boundary independently. Re-attest from a non-revoked device-DID. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-engine/src/thin_client_subscribe.rs::ThinClientConnection::connect` (Phase-3 G14-D wave-5a; D-PHASE-3-30 + CLAUDE.md baked-in #17 — thin compute surface as device with minimum capability envelope).
- **Phase:** 3 G14-D

### E_CAP_UCAN_AUDIENCE_MISMATCH

- **Message:** "UCAN audience mismatch: token aud '{actual}' != expected '{expected}'"
- **Context:** `{ expected: String, actual: String }`
- **Fix:** The presented UCAN's audience DID does not match the validation context's expected audience. Defends against cross-atrium replay (a UCAN issued to atrium A persisted in atrium B's durable store and replayed against atrium B). Re-issue the UCAN with the correct `aud` for the local atrium. Distinct from `E_CAP_DENIED` so audit pipelines can route on cross-atrium replay independently. Routes to `ON_DENIED`.
- **Thrown at:** `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_for_audience_at` (Phase-3 G14-B mini-review fix-pass; CLR-2 audience-binding pinned at the durable chain-walk seam). Constant-time DID-bytes comparison via `subtle::ConstantTimeEq` at the `benten_id::ucan::validate_chain_for_audience` upstream.
- **Phase:** 3 G14-B

### E_ATRIUM_RELAY_UNREACHABLE

- **Message:** "atrium relay unreachable at {url}: {reason}"
- **Context:** `{ url: String, reason: String }`
- **Fix:** The configured iroh relay endpoint is unreachable (DNS-resolution failure, TLS handshake refused, transport-level timeout). Verify the relay URL is reachable from this peer's network (curl / nslookup / openssl s_client). For Phase-3 deployments the iroh public relay default applies; operators with stricter metadata threat models can opt into self-hosted relay infrastructure (Compromise #22 in `docs/SECURITY-POSTURE.md` — Phase-7 Garden-relays land as the operator-controlled alternative). Per `net-blocker-2` BLOCKER, this is a typed error variant — never a panic, never an untyped String. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (which signals an established connection has degraded mid-flight). Routes to `ON_ERROR`.
- **Thrown at:** `crates/benten-sync/src/transport.rs::Endpoint::bind_with_relay_url` + `crates/benten-sync/src/transport.rs::Endpoint::connect` (Phase-3 G16-A wave-6; net-blocker-2 BLOCKER). Mapped from the `AtriumTransportError::RelayUnreachable` typed variant via `crates/benten-sync/src/errors.rs::AtriumTransportError::code`.
- **Phase:** 3 G16-A

### E_ATRIUM_TRANSPORT_DEGRADED

- **Message:** "atrium transport degraded: {reason}"
- **Context:** `{ reason: String }`
- **Fix:** The established Atrium transport has degraded — packet-loss above threshold, relay-fallback active mid-stream, direct connection lost, or handshake wire-format violation surfaced at the transport layer. The engine-side `engine.atrium_status()` surface (Phase-3 G16-B/D) propagates this state observably so operators can react. Investigate network conditions (packet-loss, NAT path) and the connecting peer's reachability. Per `net-blocker-2` BLOCKER, the degraded transport state is EXPLICIT — not a missing value, not a panic. Distinct from `E_ATRIUM_RELAY_UNREACHABLE` (which signals the relay endpoint itself is unreachable at connect time). Routes to `ON_ERROR`.
- **Thrown at:** `crates/benten-sync/src/transport.rs::Endpoint::*` (Phase-3 G16-A wave-6 connection-establishment + send/recv paths; net-blocker-2 BLOCKER). Also fires from `crates/benten-sync/src/handshake_wire.rs::HandshakeFrame::from_canonical_bytes` when the wire-format frame is missing required fields per net-blocker-4 BLOCKER. Mapped from the `AtriumTransportError::TransportDegraded` / `AtriumTransportError::HandshakeWireFormat` typed variants via `crates/benten-sync/src/errors.rs::AtriumTransportError::code`.
- **Phase:** 3 G16-A

### E_HANDSHAKE_REPLAY_WITHIN_BOUNDED_WINDOW

- **Message:** "handshake replay within bounded window: original_hlc={original_hlc} replay_hlc={replay_hlc} window_ms={window_ms}"
- **Context:** `{ original_hlc: u64, replay_hlc: u64, window_ms: u64 }`
- **Fix:** A handshake frame was replayed within the bounded HLC acceptance window (default `DEFAULT_REPLAY_WINDOW_MS = 5000`). The handshake state machine rejects bounded-window replays via symmetric drift math (`now.abs_diff(hlc_physical_ms) > replay_window_ms`) so future-stamped frames are also rejected — defends against clock-skew injection. The diagnostic fields (`original_hlc`, `replay_hlc`, `window_ms`) let operators distinguish bounded-window replay from transport-layer degradation. Per `ds-r4-3`, the replay defense is EXPLICIT and TYPED — not a generic transport error. The canonical replay-detection mechanism (per-peer nonce cache) is deferred to a follow-on wave per the source comment at `crates/benten-sync/src/handshake.rs::Handshake::respond`; G16-D ships only the bounded-window math. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (transport-layer signal) — this is a semantic-layer reject. Routes to `ON_ERROR`.
- **Thrown at:** `crates/benten-sync/src/handshake.rs::HandshakeError::ReplayWithinBoundedWindow` (Phase-3 G16-D wave-6b; ds-r4-3). Surfaces from `Handshake::respond` and `Handshake::finalise` when the carried HLC drift exceeds the replay window. Composes with G14-pre-D HLC bounded-window math.
- **Phase:** 3 G16-D

## Extending the catalog

When adding a new error:

1. Reserve the next code in the relevant subsystem range (e.g. next `E_CAP_*`)
2. Document message, context, fix, layer
3. Update the corresponding TypeScript error type in `@benten/engine/errors`
4. Never change an existing code's meaning; deprecate and add new if semantics shift

## Versioning

Error codes are versioned with the engine. Adding new codes is a minor version bump. Changing an existing code's message template without changing semantics is a patch bump. Removing or changing semantics is a major version bump and requires migration documentation.
