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

### E_INV_SANDBOX_NESTED

- **Message:** "SANDBOX Node {node_id} calls another SANDBOX, nesting depth {depth} exceeds max {max}"
- **Context:** `{ node_id: NodeId, depth: number, max: number }`
- **Fix:** SANDBOX should not call SANDBOX. Flatten or use CALL with a SANDBOX-terminated subgraph.
- **Thrown at:** Registration
- **Phase:** 2 (invariant 4 enforcement; Phase 1 type-defines SANDBOX but the executor returns `E_PRIMITIVE_NOT_IMPLEMENTED`)

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

- **Message:** "Node {node_id} references system-zone label '{label}', unreachable from user operations"
- **Context:** `{ node_id: NodeId, label: string }`
- **Fix:** System-zone labels are reserved for engine internals. Use a non-reserved label.
- **Thrown at:** Registration
- **Phase:** 2 (invariant 11 full registration-time enforcement; Phase 1 stopgap is `E_SYSTEM_ZONE_WRITE` at the graph write-path layer)

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

- **Message:** "Cumulative iteration budget {actual} exceeds max {max} through nested ITERATE/CALL"
- **Context:** `{ actual: number, max: number, path: NodeId[] }`
- **Fix:** Reduce the multiplicative iteration space. Total iterations across nested ITERATE/CALL is bounded by the capability grant.
- **Thrown at:** Evaluation (Phase 1 flat budget) / Registration (Phase 2 multiplicative-through-CALL)
- **Phase:** 1 (runtime flat `DEFAULT_ITERATION_BUDGET` from `crates/benten-eval/src/evaluator.rs`; distinct from `E_INV_ITERATE_NEST_DEPTH`, which is the registration-time nesting-cap stopgap of 3). Phase 2 adds the multiplicative-through-CALL registration-time enforcement under the same code.

### E_INV_ITERATE_NEST_DEPTH

- **Message:** "ITERATE nesting depth {depth} exceeds Phase 1 limit {max}"
- **Context:** `{ depth: number, max: number, path: NodeId[] }`
- **Fix:** Phase 1 bounds ITERATE nesting structurally at depth 3 as a stopgap for the cumulative-budget enforcement coming in Phase 2. Flatten the nested iteration, or split into multiple CALL-connected subgraphs.
- **Thrown at:** Registration
- **Phase:** 1 (named compromise for invariant 8; see implementation plan §5 Rank 10)

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

<!-- reachability: ignore -->

- **Message:** "Expected version {expected}, found {actual} on {target}"
- **Context:** `{ target: NodeId, expected: VersionHash, actual: VersionHash }`
- **Fix:** Re-read, rebase changes, retry. Typical optimistic concurrency pattern.
- **Thrown at:** Evaluation (CAS WRITE). **Runtime surface is edge-routed, not Rust-enum-valued:** WRITE's `cas` mode routes conflicts via the `ON_CONFLICT` edge; the engine stamps `error_code: "E_WRITE_CONFLICT"` on the routed step (`crates/benten-engine/src/primitive_host.rs:~362`). Callers read the code off the edge-routing metadata, not via a `match` on an `Err(EvalError::WriteConflict)` — the enum variant exists for forward-compat with a Phase-2 native Rust path but has no construction site in Phase-1 production code. The drift-detector's `reachability: ignore` annotation reflects this asymmetry.

### E_SANDBOX_FUEL_EXHAUSTED

- **Message:** "SANDBOX exhausted fuel budget {budget} before completion"
- **Context:** `{ node_id: NodeId, budget: number }`
- **Fix:** Increase fuel budget (via capability), or reduce computational complexity. Fuel is per-subgraph, not per-call.
- **Thrown at:** Evaluation
- **Phase:** 2 (SANDBOX executor + wasmtime host land in Phase 2; Phase 1 defines SANDBOX structurally but returns `E_PRIMITIVE_NOT_IMPLEMENTED`)

### E_SANDBOX_TIMEOUT

- **Message:** "SANDBOX exceeded wall-clock timeout {timeout}ms"
- **Context:** `{ node_id: NodeId, timeout: number }`
- **Fix:** Increase timeout or split into smaller SANDBOX calls.
- **Thrown at:** Evaluation
- **Phase:** 2 (SANDBOX executor, see `E_SANDBOX_FUEL_EXHAUSTED`)

### E_SANDBOX_OUTPUT_LIMIT

- **Message:** "SANDBOX output {actual} bytes exceeds max {max}"
- **Context:** `{ node_id: NodeId, actual: number, max: number }`
- **Fix:** Return smaller output. Use STREAM for progressive output.
- **Thrown at:** Evaluation
- **Phase:** 2 (SANDBOX executor, see `E_SANDBOX_FUEL_EXHAUSTED`)

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
- **Fix:** The TRANSFORM expression language is a positive-allowlist subset of JavaScript. Any token or AST shape not in the published grammar (`docs/TRANSFORM-GRAMMAR.md`) is rejected. Common causes: closures, `this`, imports, template literals with expressions, tagged templates, optional-chained method calls, computed property names referencing `__proto__`/`constructor`/`Symbol.*`, `new`/`with`/`eval`/`yield`/`async`/`await`, destructuring with getters. See the grammar doc's "Rejected constructs" appendix.
- **Thrown at:** Registration (TRANSFORM parser runs at registration time)
- **Phase:** 1

### E_INPUT_LIMIT

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
- **Fix:** Phase 1 accepts only base32-lower-nopad multibase (`b`-prefixed) CIDv1. Check that the caller is not passing a base58btc / base64 / hex form, and that the bytes are not truncated.
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

## Extending the catalog

When adding a new error:

1. Reserve the next code in the relevant subsystem range (e.g. next `E_CAP_*`)
2. Document message, context, fix, layer
3. Update the corresponding TypeScript error type in `@benten/engine/errors`
4. Never change an existing code's meaning; deprecate and add new if semantics shift

## Versioning

Error codes are versioned with the engine. Adding new codes is a minor version bump. Changing an existing code's message template without changing semantics is a patch bump. Removing or changing semantics is a major version bump and requires migration documentation.
