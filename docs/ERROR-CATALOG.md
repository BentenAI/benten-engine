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
- **Thrown at:** Registration

## Evaluation-time errors

### E_CAP_DENIED

- **Message:** "Capability {required} not granted to {entity} for WRITE on {target}"
- **Context:** `{ required: string, entity: EntityId, target: NodeId }`
- **Fix:** Grant the capability, or call from a context that already has it. `requires` on the Node indicates the needed grant.
- **Thrown at:** Evaluation (at commit, not at individual WRITE, per the transaction-capability interaction rule)

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

- **Message:** "Expected version {expected}, found {actual} on {target}"
- **Context:** `{ target: NodeId, expected: VersionHash, actual: VersionHash }`
- **Fix:** Re-read, rebase changes, retry. Typical optimistic concurrency pattern.
- **Thrown at:** Evaluation (CAS WRITE)

### E_SANDBOX_FUEL_EXHAUSTED

- **Message:** "SANDBOX exhausted fuel budget {budget} before completion"
- **Context:** `{ node_id: NodeId, budget: number }`
- **Fix:** Increase fuel budget (via capability), or reduce computational complexity. Fuel is per-subgraph, not per-call.
- **Thrown at:** Evaluation

### E_SANDBOX_TIMEOUT

- **Message:** "SANDBOX exceeded wall-clock timeout {timeout}ms"
- **Context:** `{ node_id: NodeId, timeout: number }`
- **Fix:** Increase timeout or split into smaller SANDBOX calls.
- **Thrown at:** Evaluation

### E_SANDBOX_OUTPUT_LIMIT

- **Message:** "SANDBOX output {actual} bytes exceeds max {max}"
- **Context:** `{ node_id: NodeId, actual: number, max: number }`
- **Fix:** Return smaller output. Use STREAM for progressive output.
- **Thrown at:** Evaluation

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

## Sync-time errors (Phase 3+)

### E_SYNC_HASH_MISMATCH

- **Message:** "Received content hash {received} does not match expected {expected}"
- **Context:** `{ node_id: NodeId, received: CidV1, expected: CidV1, peer: PeerId }`
- **Fix:** Possible tampering or corruption. Sync is aborted; investigate the peer.
- **Thrown at:** Sync-receive

### E_SYNC_HLC_DRIFT

- **Message:** "HLC timestamp {received} exceeds drift tolerance {max_drift} from local clock {local}"
- **Context:** `{ received: HlcTimestamp, local: HlcTimestamp, max_drift: Duration, peer: PeerId }`
- **Fix:** Peer's clock is outside tolerance. Triggers clock reconciliation handshake; if that fails, sync pauses.
- **Thrown at:** Sync-receive

### E_SYNC_CAP_UNVERIFIED

- **Message:** "Received WRITE lacks valid capability chain from {peer}"
- **Context:** `{ peer: PeerId, node_id: NodeId, missing: string }`
- **Fix:** Peer sent a change without proper authority. Sync-receive rejects; investigate peer trust level.
- **Thrown at:** Sync-receive

## TypeScript binding-layer errors

### E_DSL_INVALID_SHAPE

- **Message:** "DSL value does not match expected shape: {reason}"
- **Context:** `{ reason: string, received: unknown }`
- **Fix:** Check the DSL API documentation for the expected shape.
- **Thrown at:** DSL wrapper (TypeScript layer, before engine call)

### E_DSL_UNREGISTERED_HANDLER

- **Message:** "No handler registered for '{handler_id}'"
- **Context:** `{ handler_id: string, suggestions: string[] }`
- **Fix:** Check spelling; register via `ctx.registerSubgraphs()` or `crud()`.
- **Thrown at:** DSL wrapper

## Extending the catalog

When adding a new error:

1. Reserve the next code in the relevant subsystem range (e.g. next `E_CAP_*`)
2. Document message, context, fix, layer
3. Update the corresponding TypeScript error type in `@benten/engine/errors`
4. Never change an existing code's meaning; deprecate and add new if semantics shift

## Versioning

Error codes are versioned with the engine. Adding new codes is a minor version bump. Changing an existing code's message template without changing semantics is a patch bump. Removing or changing semantics is a major version bump and requires migration documentation.
