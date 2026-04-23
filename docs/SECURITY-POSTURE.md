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

#### Hash algorithm choice — BLAKE3 (options considered)

Benten uses **BLAKE3-256** specifically (multihash code `0x1e`), not SHA-256 (`0x12`). The decision was made with explicit awareness of the interop tradeoffs:

- **Option A — BLAKE3 only (chosen).** Native `iroh` P2P transport (Phase 3) uses BLAKE3 throughout; zero hash-translation at the network boundary. ~10× faster than SHA-256 on modern CPUs (SIMD + tree hashing). Parallel-chunkable — large blobs verify in parallel via BLAKE3's tree structure. IPLD-format compatible (CIDv1 with multihash `0x1e`).
- **Option B — SHA-256 only.** Maximum compatibility with default IPFS gateways and broader ecosystem tooling (Filecoin, web3 wallets, blockchain indexers). Loses the 10× speed advantage and the iroh-native alignment.
- **Option C — Dual-hash (publish both CIDs).** Content addressed by both BLAKE3 and SHA-256. Double storage cost. Complete ecosystem reach. Adds a verification step per access path.
- **Option D — BLAKE3 internal + SHA-256 translation at boundary.** Internal paths use BLAKE3; when content is published to a SHA-256-expecting network (e.g., public IPFS gateways), a SHA-256 CID is computed over the same canonical bytes. Preserves speed + iroh alignment internally; adds Phase-2+ complexity at the publish boundary.

**Why Option A for Phase 1-3:** Benten's deployment model is peer-to-peer meshes (Atriums, Gardens, Groves) synced via iroh, not public IPFS gateways. Content stays within Benten-speaking peer networks. The speed + iroh-alignment of BLAKE3 dominates; the cost (reduced default-IPFS-gateway verification) doesn't hit our Phase 1-3 deployment model.

**Interop caveats (Phase 1 honest disclosure):**

- Our CIDs ARE valid CIDv1 per the IPLD spec. Any multiformat-aware parser reads the structure: `[0x01 version][0x71 dag-cbor][0x1e BLAKE3][0x20 length][32-byte digest]`.
- A public IPFS gateway (e.g., `ipfs.io/ipfs/bafyr4i...`) can fetch and serve our content by CID; it does NOT need BLAKE3 support for routing/storage.
- Verification of fetched bytes (the content-addressing integrity check) requires BLAKE3 support in the reader. Modern `kubo` (go-ipfs) ships with BLAKE3 support since ~2023. Older gateways or custom builds may route without verifying.
- Pure content integrity inside Benten peer networks — where every peer speaks BLAKE3 — is unaffected.

**Phase-N reconsideration triggers:** If Benten ever commits to "content must be verifiable on default public IPFS gateways without plugin-level BLAKE3 support" as an adoption requirement, revisit with Option C (dual-hash) or Option D (boundary translation). Until then, Option A stands.

---

### Compromise #2 — Symmetric-None + diagnostic capability (Option C) — CLOSED

**Status (2026-04-17, 5d-J workstream 1):** migrated from Option A (honest-but-existence-leaking `E_CAP_DENIED_READ`) to **Option C** (symmetric `None` on denial, diagnostic-capability escape hatch). The existence-leak surface the prior posture named is no longer live; the escape hatch gives operators the signal they need without exposing it to ordinary callers.

**Primary path — symmetric None.** `Engine::get_node`, `Engine::edges_from`, `Engine::edges_to`, and `Engine::read_view` now collapse a `CapabilityPolicy::check_read` denial onto `Ok(None)` / `Ok(vec![])` / an empty-list `Outcome` — byte-identical with the response an unauthorised caller would see if the CID were genuinely absent. An attacker probing the CID space cannot distinguish denial from not-found through any of these surfaces.

**Escape hatch — `Engine::diagnose_read`.** A new public method surfaces the distinction, but is itself gated on a `debug:read` capability: the configured policy's `check_read` is consulted with label `"debug"` and the target CID; a denial there collapses the probe into `Err(CapError::Denied)` so ordinary callers see the same `E_CAP_DENIED` shape that every other capability denial wears. When permitted, the method returns:

```rust
pub struct DiagnosticInfo {
    pub cid: Cid,
    pub exists_in_backend: bool,
    pub denied_by_policy: Option<String>,  // `"store:<label>:read"` on denial
    pub not_found: bool,
}
```

Three distinguishable states:

- `existsInBackend: false, notFound: true, deniedByPolicy: null` — never written (or deleted).
- `existsInBackend: true, deniedByPolicy: Some("store:<label>:read")` — exists, reader lacks the scope.
- `existsInBackend: true, deniedByPolicy: None` — exists and is readable by this caller.

**TypeScript surface:** `engine.diagnoseRead(cid)` returns `{ cid, existsInBackend, deniedByPolicy, notFound }`. The `CrudOptions.debugRead` flag on `crud('post', { debugRead: true })` is an informational hint for tooling that the handler's operator expects to hold the diagnostic grant; the real gate is `engine.grantCapability({ actor, scope: "store:debug:read" })`.

**Posture claim:**

- The public read API does NOT surface an existence signal to unauthorised callers under any input.
- The diagnostic signal IS available, but is itself capability-gated — an attacker who lacks the grant sees `E_CAP_DENIED` (not `E_NOT_FOUND`, not `null`).
- NoAuth deployments (no policy configured) treat `diagnose_read` as open; this matches the embedded / single-user trust model where the caller already has full backend access.

**What this posture does NOT claim:**

- **Change-stream parity.** `Engine::subscribe_change_events` still fans out every committed ChangeEvent without a per-event `check_read` gate — see the separate "Change-stream subscription bypasses capability read-checks" section below. The Option-C gate covers the four read surfaces named above; the subscribe path stays as-is for Phase 1 because the Engine instance itself is the security boundary.
- **Evaluator-path gating of READ primitives inside a user subgraph.** Option C gates the engine-orchestrator public API. The evaluator's `PrimitiveHost::check_read_capability` hook is now wired (5d-J workstream 1 added the trait method with a permissive default); Phase-2 threads it into the READ primitive's execute path so `crud:post:get` dispatched through `Engine::call` honours Option C end-to-end without a separate gate at the public API.

**`E_CAP_DENIED_READ` code:** retained in the catalog (`docs/ERROR-CATALOG.md`) because Phase-2 evaluator-path READ enforcement still needs a typed denial code for the evaluator-visible leg — the Option-C public API mapping is an engine-orchestrator concern, not a catalog removal. The `CapError::DeniedRead` variant remains the signal policies use to communicate "denied" to the engine; the engine maps it onto `Ok(None)` at the public boundary.

**Regression tests:**

- `crates/benten-eval/tests/read_denial.rs` — six Option-C tests covering symmetric-None on `get_node`, `edges_from`, the three `diagnose_read` outcomes (`exists_but_denied`, `not_found`, NoAuth-open), and the `debug:read` gate.
- `crates/benten-engine/tests/integration/compromises_regression.rs::compromise_2_option_c_symmetric_none_plus_diagnose_read` — engine-level regression.
- `crates/benten-engine/tests/integration/compromises_regression.rs::compromise_2_option_c_is_documented` (implicit, via the doc-grep in the eval-side `compromise_2_option_c_is_documented`) — keeps this section load-bearing.

**Phase 3 revisit (federation / sync):** sync replicas cross trust boundaries; Phase 3 revisits whether a reader CAN observe existence through a sibling peer (the Phase-3 `CapRevoked` scenario) and may upgrade `diagnose_read` to require a federation-aware principal handle. The Option-C surface introduced here stays stable; Phase 3 layers scope on top.

---

### Compromise #1 — TOCTOU window bound at CALL entry + ITERATE batch boundary

Phase-1 capability checks refresh the grant snapshot at THREE distinct
boundaries: (a) every transaction commit via `CapabilityPolicy::check_write`
in `benten-engine`, (b) CALL primitive entry via
`PrimitiveHost::check_capability`, and (c) ITERATE batch boundaries —
every `host.iterate_batch_boundary()` iterations (default 100), inclusive
of iter 0. A revocation that lands mid-batch is therefore visible to the
evaluator at the NEXT batch boundary; a revocation that lands between
handler registration and CALL entry is visible at the CALL entry.

**Why the batch cadence:** per-iteration policy lookup would impose an
O(N) backend read against the grant table on every step of every
iterate. The batch-refresh amortizes that cost to O(N/100) while keeping
the worst-case TOCTOU window bounded at 99 iterations.

**What this posture does NOT claim:**
- Per-iteration revocation visibility inside a batch. A grant revoked at
  iter 50 will still authorize writes 50..=100; write 101 is the first
  to see the revocation.
- Real-time revocation across a federation (that's the Phase-3
  `CapRevoked` code, distinct from Phase-1's `CapRevokedMidEval`).

**What IS guaranteed:**
- Transaction commits see the current policy state (per-commit).
- CALL entry observes a revocation that landed before the outer
  handler reached the CALL primitive; the denial routes `ON_DENIED`.
- Writes past an ITERATE batch boundary observe any revocation that
  landed within the previous batch; the denial routes `ON_DENIED`.
- The batch-boundary / CALL-entry denials surface the policy's error
  code string (e.g. `E_CAP_REVOKED_MID_EVAL`) in the edge payload so
  operators can distinguish batch-boundary revocation from generic
  `E_CAP_DENIED`.

**Regression tests:**
- `crates/benten-engine/tests/integration/cap_toctou.rs::capability_revocation_at_batch_boundary_surfaces_mid_eval_code`
  — engine-level per-commit refresh.
- `crates/benten-eval/tests/cap_refresh_toctou.rs` — seven tests
  covering CALL-entry refresh (permit + deny), ITERATE entry refresh,
  batch-boundary refresh, no-spurious-refresh on single-batch, and
  host-supplied boundary override.

**Phase-2 revisit:** configurable per-handler batch size (0 =
per-iteration check, at the cost of the O(N) backend read) and
wall-clock bound on the TOCTOU window (auditor finding
[g4-p2-uc-2](../.addl/phase-1/r5-g4-pass2-ucan-capability-auditor.json)
— TRANSFORM-heavy handlers can push the 100-iteration cap past 10
minutes of wall-clock time). The deferred integration tests
`capability_revoked_mid_iteration_denies_subsequent_batches` and
`writes_in_current_batch_are_not_retroactively_denied` in
`crates/benten-caps/tests/toctou_iteration.rs` remain `#[ignore]`
pending the Phase-2 `schedule_revocation_at_iteration` API on
GrantReader + a populated `iterate_write_handler` fixture.

---

### Compromise #3 — `ErrorCode` enum lives in `benten-core` — CLOSED

Originally open: the canonical catalog enum `ErrorCode` lived in `benten-core`
instead of a dedicated `benten-errors` crate, which forced every workspace crate
that only needed the stable string identifiers to carry a `benten-core`
dependency edge.

**Closure (2026-04-17).** `ErrorCode` (plus `as_str` / `as_static_str` /
`from_str`) extracted to a new [`benten-errors`](../crates/benten-errors/src/lib.rs)
root crate with zero workspace dependencies. Every workspace crate now
depends directly on `benten-errors` for the catalog; `benten-core` keeps its
own `CoreError::code()` mapping but is no longer the source of truth for the
enum itself. The drift-detector (`scripts/drift-detect.ts`) reads the enum
from its new home; the codegen script's comment is updated to match.

**Posture claim (unchanged):** the `ErrorCode` string forms (`"E_CAP_DENIED"`,
`"E_INV_CYCLE"`, …) remain **frozen**. Drift between this enum and
`docs/ERROR-CATALOG.md` is detected by the drift lint in CI. Adding a
variant requires (a) the enum entry, (b) a catalog doc entry, (c) the
`.code()` mapping in the owning crate.

**Regression test:** `compromise_3_error_code_enum_in_benten_errors` in
`crates/benten-engine/tests/integration/compromises_regression.rs` pins
the type path via `std::any::type_name` (the assertion now requires
`benten_errors::` — any accidental re-introduction of an `ErrorCode` back
in `benten_core` fails the test). A second pin lives in
`crates/benten-errors/tests/stable_shape.rs` which counts variants and
round-trips the catalog-code strings through `as_str` / `from_str`.

---

### Compromise #4 — WASM runtime is compile-check only

The `bindings/napi` crate compiles with `--target wasm32-unknown-unknown` in CI (`wasm-checks.yml`) but does NOT execute a WASM runtime (browser / `wasmtime`) at test time.

**Why:** the Phase-1 WASM surface exists to guarantee that the napi bindings build for a browser target so Thrum (the Phase-4 consumer) can compile them into its web bundle. Runtime execution of the WASM artifact is a Phase-2 scope item tied to the SANDBOX primitive's `wasmtime` host — both land together so the WASM runtime story is coherent.

**What this posture claims:**
- The napi bindings compile for `wasm32-unknown-unknown` with zero warnings.
- The compiled artifact has no forbidden symbol references (no `std::net`, no `std::fs::File::open` in hot paths).

**What this posture does NOT claim:**
- That the WASM build runs correctly in a browser or `wasmtime` host. Phase-2 adds in-browser integration tests; Phase-1 relies on the compile-check as a coarse smoke test.
- That napi and WASM have behavioral parity. Some surfaces (redb backend) are stubbed out on WASM per `#[cfg]` gates.

**Phase-2 revisit:** add a `wasmtime` harness in CI that executes a smoke-test from the WASM build. Land this alongside the SANDBOX primitive's runtime.

---

### Compromise #5 — No write rate-limits; metric recorded only

Phase 1 does not enforce a write-rate limit on the engine's ingress. A misbehaving or adversarial caller can submit arbitrary writes as fast as the capability policy permits.

**What IS recorded:** `engine.metrics_snapshot()` surfaces four write counters (both Rust + napi surfaces):

- `benten.writes.committed` — aggregate count of transactions the capability policy permitted (tick once per committed batch, not per op).
- `benten.writes.denied` — aggregate count of transactions the capability policy rejected.
- `benten.writes.committed.<scope>` — per-capability-scope fan-out. The scope key is `store:<label>:write`, derived from the batch's `PendingOp` labels (mirrors `GrantBackedPolicy`'s internal derivation so the counters line up with the enforcement-side key space).
- `benten.writes.denied.<scope>` — per-capability-scope denial fan-out.

Typed accessors `Engine::capability_writes_committed() -> BTreeMap<String, u64>` and `Engine::capability_writes_denied() -> BTreeMap<String, u64>` expose the per-scope maps directly without the flattened key projection. napi callers get the same shape via `engine.capabilityWritesCommitted()` / `engine.capabilityWritesDenied()`. Operators can detect abnormal rates per scope out-of-band.

The counters increment regardless of whether a capability policy is plumbed in — under the zero-config `NoAuthBackend` default a batch that writes `post`-labelled Nodes still bumps `benten.writes.committed.store:post:write`. System-zone writes (`system:*` labels) are intentionally excluded from the per-scope tally because user subgraphs cannot reach them and crediting privileged grant/revoke paths to the per-scope key would make the metric misleading.

**Why no enforcement:** a proper rate-limit needs a **scoped** budget (per-actor, per-capability, per-handler) — not a global one. Phase-1 lacks the actor-identity machinery (Phase-3 `benten-id`) to make the scoped variant meaningful; a global rate-limit would punish legitimate bulk-import workflows more than it protects against abuse. Recording the per-scope counter now means Phase-3 can layer enforcement on top without re-deriving the scope key space.

**What this posture does NOT claim:**
- Protection against DoS via write-flood at the engine ingress.
- A ceiling on backend write throughput.
- Bounded memory for the per-scope map. Scope keys derive from user-supplied Node labels, so an adversarial writer who creates Nodes with (say) 10k distinct fresh labels will grow the map to 10k entries. Phase-3's rate-limit pass adds eviction; Phase-1 accepts the unbounded-growth surface because (a) realistic label cardinality is bounded, and (b) the attack class is identical to spamming distinct CIDs in the backend, which the operator already has to manage.

**Phase-2 / Phase-3 revisit:** Phase-2 introduces a policy-layer budget trait so `CapabilityPolicy` implementations can enforce per-actor rate limits. Phase-3 ties the identity shape (actor Cid) to the budget so the rate is scoped correctly across a federation, and adds eviction for the per-scope counter map.

**Regression tests:**
- `writes_committed_metric_is_recorded` + `per_capability_write_metrics_increment` + `denied_writes_surface_on_denied_metric` in `crates/benten-engine/tests/metrics.rs` pin the Rust recording shape.
- `compromise_5_no_write_rate_limits_but_metric_recorded` in `crates/benten-engine/tests/integration/compromises_regression.rs` pins the "no rate-limit enforcement" half.
- `metricsSnapshot surfaces per-capability write counters` in `bindings/napi/index.test.ts` pins the TS round-trip.

---

### Compromise #7 — `[[bin]]` `required-features` gating — CLOSED

Originally open: `benten-graph`'s `write-canonical-and-exit` test-fixture
bin was declared with `test = false` / `bench = false` but no
`required-features` gate, so `cargo install benten-graph` compiled an
unnecessary test-fixture binary alongside the library crate.

**Closure (2026-04-17).** `crates/benten-graph/Cargo.toml` now declares a
`test-fixtures` feature (default-enabled) and gates the bin with
`required-features = ["test-fixtures"]`. Downstream consumers doing
`cargo install benten-graph --no-default-features` skip the bin entirely;
the workspace-wide `cargo test` / `cargo nextest run --workspace` path
keeps it via the default feature so `d2_cross_process_graph.rs` still
resolves `CARGO_BIN_EXE_write-canonical-and-exit`.

**Regression test:**
`compromise_7_benten_graph_bin_is_required_features_gated` in
`crates/benten-engine/tests/integration/compromises_regression.rs` reads
`benten-graph/Cargo.toml` and asserts (a) the `required-features`
clause, (b) the `test-fixtures` feature declaration, (c) the default
membership. Removing any of the three flips the test red and re-opens
this compromise.

---

### Compromise #8 — `Engine::call` bypasses the evaluator for CRUD handlers — CLOSED

Originally open: during G7 the `Engine::call` dispatch for
`register_crud`-registered handlers took a "CRUD fast-path" that
synthesised a transaction directly against the backend and skipped the
`benten-eval` evaluator walk entirely. The fast-path mirrored the
capability hook and change-event emission of the full dispatch, but it
was a parallel code path — any invariant check or primitive-level hook
added to the evaluator would not fire for CRUD handlers.

**Closure (R5 pass-5b).** The `PrimitiveHost` trait was extracted and
`benten-engine` now implements it; `Engine::call` drives
`Evaluator::run_with_trace` for every registered handler (CRUD and
SubgraphSpec alike) and replays buffered host-side WRITE / DELETE ops
atomically inside a single transaction after the walk completes. The
CRUD fast-path is retired; there is no dispatch path that reaches the
backend without walking the evaluator first.

**Why it matters (security framing):** the bypass was a latent
backdoor that would have let a Phase-2 invariant (e.g. invariant 8
cumulative iteration budget, invariant 11 system-zone reachability)
ship green against SubgraphSpec handlers while silently not firing for
the zero-config `crud('<label>')` registration that most applications
use. Closing the compromise eliminates that backdoor before the
Phase-2 invariant set lands.

**Regression test:**
`compromise_8_primitive_host_is_sole_dispatch` in
`crates/benten-engine/tests/integration/compromises_regression.rs`
pins the "evaluator is sole dispatch path" contract. Re-opening the
CRUD fast-path flips the test red.

---

## `requires` property is Phase-1 advisory (r6-sec-1)

Handler subgraphs can declare a `requires` property on each primitive
(e.g. `write.requires("store:post:write")`). In Phase 1 this property is
**declarative-only**: the engine does NOT use the declared string to gate
the operation at evaluation time. What IS enforced is the **derived
per-op scope**: `GrantBackedPolicy` re-derives `store:<label>:write` (or
`store:<label>:read`) from the actual `PendingOp` the transaction
commits, and requires an unrevoked capability grant for that scope. The
attack class where a handler declares `requires: "post:read"` but writes
to an `admin`-labelled Node is therefore already closed — the policy
sees `store:admin:write` in the PendingOp batch, finds no grant, and
denies.

What Phase 1 does NOT close:

- **Declared-vs-actual mismatch surfacing.** A handler that declares
  `requires: "post:read"` but actually writes admin data registers and
  runs; the write is denied at commit, but the registration itself gives
  no warning. Operator tooling + the mermaid diagram DO show the
  declared string, so a human reviewing the registered handler sees the
  lie.
- **CALL-attenuation via `requires`.** The `isolated: false` call path
  that would attenuate the caller's capability context to the
  intersection of the outer grant and the callee's declared `requires`
  is Phase-2 scope (named compromise contract, R1 triage SC4). The
  Phase-1 posture: every CALL runs under the outer actor's grants; a
  compromised callee that issues a wider write sees the same per-op
  derived-scope check as any other handler.

The pair of tests at `crates/benten-eval/tests/requires_enforcement.rs`
remain `#[ignore]`-gated on the Phase-2 register-time static analysis
pass that would elevate declared-vs-actual to a registration-time
error (`E_REQUIRES_SCOPE_MISMATCH`). The test pair proves the Phase-2
closure once the static analyzer lands; the Phase-1 defensive line is
the GrantBackedPolicy derived-scope check exercised by
`crates/benten-caps/tests/grant_backed_policy.rs`.

---

## Change-stream subscription bypasses capability read-checks

**Phase-1 posture.** `Engine::subscribe_change_events` returns a
`ChangeProbe` that drains every committed `ChangeEvent` the engine has observed — including events for Nodes the subscriber does not hold a
`store:<label>:read` capability for. No `check_read` is applied on the
subscriber path. This is a deliberate Phase-1 simplification, not a bug:

- **The Engine instance is itself the security boundary.** Phase 1 ships
  the embedded / single-process trust model (`docs/VISION.md`, pillar 1).
  Every caller of `subscribe_change_events` is already trusted with full
  read access to the backing store — they could open the `redb` file
  directly and observe the same data. Gating the subscribe surface would
  give false assurance without closing the real exfiltration path.
- **Existence-leak parity with Compromise #2.** The same "denied reads
  reveal the CID exists" surface that Compromise #2 documents for
  `check_read` already applies to the change stream: a subscriber can
  enumerate committed CIDs regardless of whether a read capability is
  granted. The two surfaces are intentionally co-located because the
  Phase-3 fix is the same: scoped subscriptions over a trust boundary.
- **Attribution is preserved.** Every `ChangeEvent` carries the
  `actor_cid` / `handler_cid` / `capability_grant_cid` triple (r6-sec-3),
  so a Phase-3 policy layer can retroactively filter by observer identity
  without breaking the wire format.

  **Phase-1 field status.** `capability_grant_cid` is present in the
  wire format but is always `None` in Phase 1 — grant-resolution on the
  write path is Phase-3 `benten-id` scope. The field is frozen now so
  audit consumers written today (forward-compatibility). Phase-1 audit
  code MUST NOT rely on the value being populated; consuming code that
  needs grant attribution should wait on the Phase-3 identity surface.

**Phase-3 revisit.** Alongside Compromise #2 — once `benten-id` lands a
typed principal and sync / federation cross the trust boundary, the
engine will:

1. Accept a principal handle at `subscribe_change_events` time.
2. Apply `CapabilityPolicy::check_read` per event before yielding it.
3. Decide between Option A (surface `E_CAP_DENIED_READ` — consistent with
   the read path) or Option B (silent drop — matches the "indistinguishable
   from not-found" posture).

Operators who need a tighter bound today can:
- Deploy with `.without_ivm()` + avoid calling
  `subscribe_change_events` — no probe, no disclosure.
- Run the engine behind a process boundary and gate the subscribe RPC at
  the mux layer.

---

## napi input-limit enforcement (r6-sec-7)

The TypeScript→Rust boundary is the engine's hottest surface and the
primary DoS vector for a hosted deployment. Two classes of input-size
attack are live in Phase 1:

1. **Oversized JSON strings.** A caller who supplies a single
   multi-gigabyte `Value::Text` can force the Rust side to allocate the
   full string before any downstream check fires. The JSON boundary in
   `bindings/napi/src/node.rs` now rejects any string longer than
   `JSON_MAX_BYTES` (1 MiB) with `E_INPUT_LIMIT` before the `Value::Text`
   lands in the tree.
2. **Aggregate payload size.** A JSON tree whose total text-byte weight
   exceeds the per-request budget is similarly rejected with
   `E_INPUT_LIMIT` — the check runs during tree-walk so deeply-nested
   payloads cannot evade the cap by fragmenting across many small values.

**Phase-2 completeness.** The canonical on-wire decoder
(`testing::deserialize_value_from_js_like`) is still a shim pending a
`CoreError::InputLimit` variant in `benten-core`; the B8 input-validation
test suite is gated behind `--features in-process-test` and stays red
until the decoder un-stub lands (coordination is deferred to the error-
ergonomics work track). The boundary-side caps in this section are the
Phase-1 defensive line against the allocation vector; the B8 suite will
add CBOR-level depth / bomb coverage on top.

---

## `ExecutionStateEnvelope::envelope_cid` does not cover `schema_version` (Phase 2a G3-A / G3-A-mini-review Minor-2)

In Phase 2a `ExecutionStateEnvelope::envelope_cid` returns `payload_cid`
— the BLAKE3 over the DAG-CBOR bytes of `ExecutionStatePayload`. The
envelope's `schema_version: u8` byte is **not** covered by this CID.

**Implication.** An attacker who re-wraps the same payload under a
future `schema_version = 2` produces an envelope whose `envelope_cid`
is byte-identical to the `schema_version = 1` form. Today this is
purely hypothetical — `schema_version = 1` is the only valid value
and the resume path rejects mismatches — but if Phase 2b/3 grows the
envelope shape additively, the re-wrap attack becomes reachable
unless the envelope hash includes the full envelope (not just the
payload).

**Mitigation path (Phase 2b / Phase 3).** Either (a) redefine
`envelope_cid` to hash the full envelope bytes (including
`schema_version`), or (b) ship a separate `envelope_hash` field
alongside `payload_cid` so callers can ask the right question.
Option (a) would change the CID contract and requires coordination
with any already-persisted `ExecutionStateEnvelope` in storage;
Option (b) is additive and preferred.

**Phase 2a status.** Phase 2a pins `schema_version = 1`; the single
call-site that checks re-wrap tampering (`resume_from_bytes` re-
computes `payload_cid` and asserts equality) fires correctly for
Phase 2a's closed shape. Forward-compat concern only.

**Cross-refs.** §9.1 of `.addl/phase-2a/00-implementation-plan.md`
(envelope shape frozen); G3-A mini-review Minor-2 (captured 2026-
04-22).

---

*Future compromises with security implications will be appended as sections here, each tagged with the compromise number from the R1 Triage Addendum.*
