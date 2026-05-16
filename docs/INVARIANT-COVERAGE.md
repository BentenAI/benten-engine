# Invariant Coverage — Phase 4-Foundation Close

CLAUDE.md commits to **14 invariants** governing the Benten engine.
This document tracks per-invariant enforcement state, the enforcing
crate, and the regression suite that pins it.

**Phase 4-Foundation status:** 14 of 14 invariants enforced. Phase-4-Foundation extends Inv-14 with the plugin-DID principal classifier (see `Inv-14 Phase-4-Foundation plugin-DID principal extension` sub-section below) — the principal-type matrix now spans User-local + User-sync-merged + Device-multi-device-sync + Plugin-app-level-subgraph + Plugin-via-materializer-read. Inv-4 + Inv-7 went
ACTIVE in Phase 2b alongside the SANDBOX runtime (registration arm
landed in G7-B; runtime arm landed across waves 8b + 8h with a bounded
honest-disclosure for Inv-4 — see the "Inv-4 + Inv-7 runtime arm
status" section below). Phase 3 extended Inv-13 with the row-4 SPLIT
classifier (user-zone vs system-zone divergent-CID handling at the
sync-receive boundary; `crates/benten-sync/src/crdt.rs` +
`crates/benten-engine/tests/inv_13_dispatch.rs`) and widened Inv-14
with three additive sync-boundary attribution slots (`peer_did_set` /
`device_did` / `sync_hop_depth`); the on-the-wire device-DID
attestation envelope (G16-D wave-6b) makes Inv-14 device-grain
attribution **LOAD-BEARING under adversarial-peer assumptions** — see
the "Inv-14 Phase-3 G16-B device-grain extension" section below for
the full retense.

---

## Coverage table

| # | Invariant | Phase | Enforcer | Tests |
|---|-----------|-------|----------|-------|
| 1 | DAG-ness — no cycles in operation graphs | 1 | `benten-eval::invariants::structural::validate_subgraph` (Kahn cycle detect via `find_cycle` / `find_cycle_indices`) | `crates/benten-eval/src/invariants/structural.rs` (cycle test cluster) |
| 2 | Max operation-subgraph depth | 1 | Bounded longest-path walk + per-CALL increment | `structural.rs::depth_*` tests |
| 3 | Max fan-out per node | 1 | Edge enumeration at registration | `structural.rs::fan_out_*` tests |
| 4 | **SANDBOX nest-depth ceiling — ACTIVE (Phase 2b; both arms wired at R6FP-G1 / PR #62)** | 2b | `invariants::sandbox_depth::validate_registration` (registration); `AttributionFrame.sandbox_depth` runtime threading in `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (parent-sandbox_depth+1) + `SandboxError::NestedDispatchDepthExceeded` fires in `crates/benten-eval/src/primitives/sandbox.rs::execute` (runtime arm — both arms now active) | `crates/benten-eval/tests/inv_4_runtime_arm_fires_at_max_depth.rs`, `crates/benten-eval/tests/sandbox_depth_inheritance_regression.rs`, `crates/benten-engine/tests/sandbox_attribution_frame_security.rs` |
| 5 | Max total nodes per subgraph | 1 | Node-count gate at registration | `structural.rs::node_count_*` tests |
| 6 | Max total edges per subgraph | 1 | Edge-count gate at registration | `structural.rs::edge_count_*` tests |
| 7 | **SANDBOX `output_max_bytes` range — ACTIVE (Phase 2b; PRIMARY+BACKSTOP)** | 2b | `invariants::sandbox_output::validate_registration` (registration); `CountedSink::write` (PRIMARY streaming) + `CountedSink::backstop_check` (return-value BACKSTOP), both wired through the host-fn trampoline + primitive boundary | `crates/benten-eval/tests/sandbox_output.rs`, `crates/benten-eval/tests/proptest_sandbox_output.rs`, `crates/benten-eval/tests/integration/inv_7_streaming.rs`, `crates/benten-eval/src/sandbox/counted_sink.rs` |
| 8 | Multiplicative cumulative budget (CALL × ITERATE) | 2a | `invariants::budget` + `BudgetTracker` per evaluator step | `crates/benten-eval/src/invariants/budget.rs` (proptest cluster) |
| 9 | Determinism — handlers declared deterministic reject non-determinism sources | 1 (decl) / 2a (rt) | `structural::validate_subgraph` declaration check + runtime fence | `structural.rs::determinism_*` |
| 10 | Canonical byte encoding (order-independent DAG-CBOR) | 1 | `structural::canonical_bytes` order-independence proptest | `structural.rs::canonical_bytes_*` |
| 11 | System-zone reserved-prefix reject — user code cannot READ/WRITE system labels | 2a | `invariants::system_zone` (G5-B-i) + Engine::put_node_with_context dispatch | `crates/benten-engine/tests/inv_11_*.rs` |
| 12 | Aggregate validation catch-all — multi-invariant violations roll up | 1 | `RegistrationError::Invariant12Aggregate` | `structural.rs` aggregate-error tests |
| 13 | Immutability — User WRITE re-puts of an already-persisted CID fire `E_INV_IMMUTABILITY` | 2a | `invariants::immutability` + `WriteAuthority` firing matrix | `crates/benten-engine/tests/inv_13_*.rs` |
| 14 | Causal attribution — every primitive frame carries an `AttributionFrame` (Phase-3 G16-B device-grain extension: `peer_did_set` + `device_did` + `sync_hop_depth` slots — see "Inv-14 Phase-3 G16-B device-grain extension" below) | 2a / 3 | `evaluator::attribution` runtime threading + `ATTRIBUTION_PROPERTY_KEY` registration check + `crates/benten-engine/src/engine_sync.rs` sync-merge frame construction (G16-B) | `crates/benten-eval/tests/attribution_*.rs`, `crates/benten-engine/tests/attribution_*.rs`, G16-B sync-merge round-trip suite |

---

## Inv-4 + Inv-7 runtime arm status (honest disclosure)

Phase 1 shipped Inv-4 + Inv-7 as **stubs** because the SANDBOX primitive
itself was compile-check only (Compromise #4 in
`docs/SECURITY-POSTURE.md`). Phase 2b G7-B added the registration-time
arms; waves 8b + 8h wired the runtime executor end-to-end. R6FP Round-1
Group-1 (PR #62, 3-lens convergent fix) closed the remaining transitive
threading gap. Both runtime arms are fully active at Phase 2b close:

- **Inv-7 (SANDBOX `output_max_bytes` range)** — **fully active at
  runtime.** The wave-8b host-fn trampoline routes every host-fn
  byte-emit through `CountedSink::write`'s `OutputCheckPath::PrimaryStreaming`
  arm; the primitive boundary runs `CountedSink::backstop_check` against
  the return value (`OutputCheckPath::ReturnBackstop`). Per D17 PRIMARY +
  BACKSTOP. Per-handler ceiling per D15. Default ceiling 1 MiB;
  `SandboxArgs.outputLimitBytes` overrides per-call. Note: the
  `invariants::sandbox_output::check_admission` helper exists and is
  unit-tested but is NOT the production firing site — `CountedSink`
  enforces the same arithmetic directly via `SinkOverflow` →
  `SandboxError::OutputOverflow` mapping. Both paths produce
  `E_INV_SANDBOX_OUTPUT` typed errors with identical context shapes.

- **Inv-4 (SANDBOX nest-depth ceiling)** — **both arms fully active at
  Phase 2b close.** (1) Registration arm: `validate_registration` walks
  the static-graph at registration time. (2) Runtime arm: R6FP-G1 (PR
  #62) wired the `AttributionFrame.sandbox_depth` threading through the
  parent `ActiveCall`. At every production SANDBOX entry,
  `crates/benten-engine/src/primitive_host.rs::execute_sandbox` mutates
  the parent frame via `frame.sandbox_depth = frame.sandbox_depth.saturating_add(1)`;
  the dispatching `AttributionFrame` is constructed with `sandbox_depth:
  nested_depth` in both match arms of the same function so subsequent
  CALL pushes inherit. The eval-side runtime arm in
  `crates/benten-eval/src/primitives/sandbox.rs::execute` fires
  `SandboxError::NestedDispatchDepthExceeded` once `attribution.sandbox_depth
  > config.max_nest_depth`. Default `max_nest_depth = 4` admits depths
  1..=4; depth 5 fires. SANDBOX-inside-CALL-inside-SANDBOX inherits the
  parent's depth correctly. Carry-forward residual: the ESC-10
  adversarial integration test (`sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied`)
  stays `#[ignore]`'d pending the `testing_call_engine_dispatch` host-fn
  helper per `docs/future/phase-3-backlog.md` §7.3.A.7. The runtime arm
  is wired; only the adversarial-test driver is paper-only.

Both invariants fire as `E_INV_SANDBOX_DEPTH` (Inv-4) and
`E_INV_SANDBOX_OUTPUT` (Inv-7) error codes — both pinned in
`docs/ERROR-CATALOG.md`. The catalog rows now reflect the per-arm
honest disclosure.

The Phase-1 "Phase 2b" stubs that previously appeared in this table
have been removed; Inv-4 + Inv-7 are now first-class active rows.

---

## Where each invariant is enforced

```
┌────────────────────────────────────────┬─────────────────────────────────────────┐
│ Registration-time (one-shot)           │ Runtime-time (per-call, per-frame)      │
├────────────────────────────────────────┼─────────────────────────────────────────┤
│ Inv-1 DAG-ness                         │ Inv-4 sandbox_depth runtime counter     │
│ Inv-2 max depth                        │ Inv-7 sandbox_output CountedSink        │
│ Inv-3 fan-out                          │ Inv-8 BudgetTracker step gate           │
│ Inv-4 sandbox_depth declaration        │ Inv-13 WriteAuthority firing matrix     │
│ Inv-5 node count                       │ Inv-14 AttributionFrame propagation     │
│ Inv-6 edge count                       │                                         │
│ Inv-7 sandbox_output declaration       │                                         │
│ Inv-9 determinism declaration          │                                         │
│ Inv-10 canonical-bytes order-indep     │                                         │
│ Inv-11 system-zone literal-CID reject  │                                         │
│ Inv-12 aggregate roll-up               │                                         │
│ Inv-14 ATTRIBUTION_PROPERTY_KEY decl   │                                         │
└────────────────────────────────────────┴─────────────────────────────────────────┘

- Inv-4 runtime counter — fully wired at R6FP-G1 (PR #62). Both
  registration arm + runtime arm are active at Phase 2b close. See
  §"Inv-4 + Inv-7 runtime arm status" above for the wiring trace.
```

---

## IVM Algorithm B production registration (audit-gap closure note)

Wave-8h closed the IVM Algorithm B production-registration drift the
docs-vs-code audit caught: `Engine::create_user_view` previously
forced `ContentListingView` for every `Strategy::B`-declared user
view. Post-wave-8h the dispatch constructs `AlgorithmBView::for_id(spec.id())`
for the **5 canonical view IDs** that `AlgorithmBView` supports
natively (the hand-written single-loop dispatch in
`crates/benten-ivm/src/algorithm_b.rs`).

**Phase-3 G15-A + G15-B + R5 wave-9 W9-T1 closure — Algorithm B
generalized at Phase 3 G15-A.** Algorithm B is no longer
canonical-only; the prior canonical-view-fallback compromise is
RETIRED. User-defined views run under `Strategy::B` with their actual
label patterns rather than being coerced to `ContentListingView`
semantics. `Algorithm::register(view_id, label_pattern, projection)`
(and the budget-aware sibling `Algorithm::register_with_budget`)
instantiates a generic single-loop kernel
(`benten_ivm::algorithm_b::GenericKernel`) for non-canonical view IDs
keyed on `(label_pattern, projection)`. The genuine `AnchorPrefix`
selector lift (post-G15-A) ships in `register_user_view`; the
kernel-side guard refuses canonical-id + AnchorPrefix registrations
with the typed `AlgorithmError::CanonicalIdAnchorPrefixRefused`
variant (mirrored at the engine boundary as
`EngineError::ViewLabelMismatch`). The drift-detector proptest harness
at `crates/benten-ivm/tests/algorithm_b_drift_detector.rs` (5 pins,
1 000 cases each) drives the merged `Algorithm::register` surface
end-to-end + reports incremental-vs-rebuild parity.

---

## Inv-14 Phase-3 G16-B device-grain extension

Phase-3 G16-B widens `AttributionFrame` with three additive sync-boundary
slots so causal attribution carries device-grain + peer-grain provenance
across CRDT merges:

- **`peer_did_set: Option<BTreeSet<Did>>`** — `Some(set)` when the frame
  originates from a Loro CRDT merge; captures contributing peer DIDs
  observed via `benten_sync::crdt::LoroDoc::winning_attribution`. `None`
  for purely-local writes. The peer-node-id → DID resolution lives in
  `crates/benten-engine/src/engine_sync.rs` against the local trust
  store.
- **`device_did: Option<Did>`** — device-grain attribution per the
  D-PHASE-3-25 device-heterogeneity contract. `None` for legacy / local
  writes; `Some(did)` for sync-attributed or device-DID-attested writes.
  Lets multi-device users (laptop ↔ phone-OS-app ↔ desktop, per
  commitment #17) distinguish per-device origins inside a single
  per-user Atrium.
- **`sync_hop_depth: u32`** — bounded merge-hop counter (default cap
  `SYNC_HOP_DEPTH_CAP = 8`, mirrors Inv-4's sandbox-depth precedent).
  Increments at each CRDT merge hop; the typed
  `ErrorCode::SyncHopDepthExceeded` fires at the merge seam when a
  merge would push the depth past the cap.

**Phase-2a CID stability preserved.** All three slots elide from the
canonical Node encoding when default (`None` / `0`) via
`serde(default, skip_serializing_if = ...)`. A purely-local frame
canonicalises to the exact Phase-2a 3- / 4-key Node and produces the
pinned schema-fixture CID
(`bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`); any
non-default value adds the slot and produces a distinct CID — the
content-addressing security claim is "a sync-bearing attribution chain
is content-distinguishable from a purely-local chain."

**Construction discipline.** Test / bench / legacy callsites use
`AttributionFrame { ..Default::default() }` to spread the new slots;
production callsites in `engine_sync.rs` populate the sync slots
explicitly at the CRDT merge seam. The `Default` impl is intentionally
test-/bench-shaped (all-zero CIDs); production paths construct frames
explicitly per the WRITE-path discipline.

**Phase-3 G16-B-prime engine-side merge callback (§6.12 item 1
closure).** The engine's `apply_atrium_merge` orchestration entry
point composes the structural surface with the in-memory anchor
store: after `AtriumHandle::merge_remote_change_with_hop_depth`
returns a `SyncMergeAttribution` seed, the engine resolves
peer node-ids → peer-DIDs via `AtriumHandle::resolve_peer_dids`
(local trust-store lookup with `node-id:NNN` fallback), constructs an
`AttributionFrame` populated with `peer_did_set` / `device_did`
(from `Engine::device_cid`) / `sync_hop_depth`, and mints a new
"version" Node via `Engine::append_version` against the named
anchor. The Anchor's CURRENT pointer advances atomically via the
prior-threaded `benten_core::version::append_version` discipline.

**Phase-3 G16-B-prime device-DID threading (§6.12 item 3 closure).**
`Engine::set_device_cid` configures the engine's
device-DID-attestation CID; the engine's two production WriteContext
construction sites (`engine_diagnostics.rs::transaction` commit hook
+ `primitive_host.rs::check_capability`) populate
`WriteContext.device_cid` from this setter so heterogeneous
`CapabilityPolicy` impls can dispatch per-device under the SAME
logical-actor identity per D-PHASE-3-25.

**⚠️ SUPERSEDED-BY-COLLAPSE (refinement-audit-2026-05 S3, owner-ratified
2026-05-15 — see `docs/SECURITY-POSTURE.md` Compromise #23).** Under the
ratified trust-model reframe (DECISION-RECORD-trust-model-reframe.md §4),
**device-DID is a provenance label on the unified user-root-anchored
capability spine, NOT a distinct trust-root.** Inv-14 device-grain
attribution is **retained as an audit/provenance property** — the
`AttributionFrame.device_did` slot is still populated and still
cryptographically attributable via the retained envelope provenance-binding
signature (a peer still cannot forge another device's DID without that
device's key). What COLLAPSE deletes is the *trust decision* machinery
(`Acceptor`/`accept_at`/`DeviceRevocation`); the *trust* now flows through
the single chain-validation seam plus one retained envelope-ceiling
attenuation. The device-grain provenance / compromised-device-quarantine
audit trail survives; only the parallel device-trust-root pipe is removed.
A post-COLLAPSE successor audit (#1234) confirms every remaining
device-grain attribution use is elegant under the unified model. The
Phase-3 narrative below is preserved for historical accuracy (it was
correct for the model as it stood at Phase-3 close).

**Phase-3 G16-D wave-6b on-the-wire device-DID-attestation envelope
(plan §1 exit-criterion 16 closure) — cryptographic-attestation closure
at G16-D wave-6b fix-pass.** The handle binds a signed
`benten_id::DeviceAttestation` (parent → device-DID binding) +
device-keypair via `AtriumHandle::set_local_device_attestation` +
`AtriumHandle::set_local_device_keypair`; `sync_subgraph` +
`accept_sync_subgraph` emit a DAG-CBOR `DeviceAttestationEnvelope`
(V2 shape: `(version, attestation, payload_hash, session_nonce,
envelope_signature)`) BEFORE the Loro CRDT export on each leg. The
receiver-side `DeviceAttestationEnvelope::verify` enforces three
defenses cryptographically: (1) the envelope signature verifies
against the public key resolved from `attestation.device_did` (DID
forgery defense — a peer cannot impersonate another device's DID
without holding that device's secret key); (2) the embedded
attestation passes `benten_id::Acceptor::accept_at` (parent signature
+ freshness window + nonce-store replay defense + revocation list);
(3) `BLAKE3(received_payload) == envelope.payload_hash` via
constant-time comparison (frame-pair binding — MITM cannot swap
envelope/payload pairs). All three failure modes reject with
`E_DEVICE_ATTESTATION_FORGED` (`ON_DENIED` routing).
`Engine::apply_atrium_merge` populates `AttributionFrame.device_did`
from the verified wire envelope's declared DID (preferred) and falls
back to the local engine's `device_cid` only when no envelope was
received (legacy V1 / pre-G16-D peer / direct-test path that bypasses
`sync_subgraph`).

**Inv-14 device-grain attribution is now LOAD-BEARING under
adversarial-peer assumptions** (was advisory at PR #163 V1 shape; the
fix-pass closes the cryptographic gaps so the device-grain provenance
defense survives forged-DID / replay / frame-pair-swap attacks). The
"compromised device cannot be quarantined surgically" failure shape is
now defended at the cryptographic boundary, not via cooperating-peer
assumptions. Pinned end-to-end at:

- `tests/integration/atrium_two_device.rs::atrium_two_device_same_identity_selective_zone_sync`
  (multi-device GREEN-path with REAL signed attestations).
- `tests/integration/atrium_two_device.rs::forged_device_did_rejected_at_envelope_verify`
  (DID forgery rejection — `E_DEVICE_ATTESTATION_FORGED`).
- `tests/integration/atrium_two_device.rs::replayed_stale_envelope_rejected_by_freshness_window`
  (stale-envelope replay rejection via the freshness window re-homed onto the
  unified spine per Compromise #23 SUPERSEDED-BY-COLLAPSE; the accept-time
  nonce-store was deleted with the Acceptor cluster — durable replay-marker
  re-home tracked P2/P5 per DECISION-RECORD §4b F3).
- `tests/integration/atrium_two_device.rs::frame_pair_payload_swap_rejected_by_payload_hash_binding`
  (BLAKE3 frame-pair binding violation rejection).
- `tests/integration/atrium_two_device.rs::future_wire_version_rejected_at_decode`
  (decode-time version validation; closes cryptography MINOR-5).

The legacy unsigned shape (V1 `attestation = None`) is preserved for
backward-compat with pre-G16-D-fp peers + the two pre-existing
pinned-CID fixtures (`sync_replica_attribution_carries_device_did_alongside_parent`
+ `sync_replica_explicit_actor_cid_decouples_from_device_cid`) that
bypass the wire envelope path. See SECURITY-POSTURE.md Compromise #23
for the full closure narrative.

---

## Inv-14 Phase-4-Foundation plugin-DID principal extension

Phase-4-Foundation extends the principal-type matrix Inv-14 covers
without altering the device-grain LOAD-BEARING posture. App-level
plugins (CLAUDE.md baked-in #18) run their subgraphs under a freshly
minted `plugin_did` distinct from the user-DID and from any other
plugin's DID. The evaluator's read pathway threads the active
principal via `Engine::read_node_as(principal, cid)` (Class B β
SHIPPED at PR #184); writes attributed to a plugin carry the
plugin-DID in `AttributionFrame.actor_cid`.

The matrix Inv-14 must cover post-Phase-4-Foundation:

| Principal type | actor_cid carries | Authorization seam |
|----------------|-------------------|--------------------|
| User (local) | user-DID | `CapabilityPolicy::pre_write` |
| User (sync-merged) | user-DID | per-row recheck inside `apply_atrium_merge` |
| Device (multi-device sync) | user-DID + `device_did` | `Acceptor::accept_at` + `DeviceAttestationEnvelope::verify` |
| Plugin (app-level subgraph) | plugin-DID | `CapabilityPolicy::pre_write` + `manifest_envelope_chain_validation` (Layer-2 envelope + Layer-3 UCAN delegation) |
| Plugin via materializer-read | plugin-DID | `MaterializerEngine::read_node_as` + `MaterializerCapRecheck` (dual-gate per sec-3.5-r1-1) |

No new ErrorCode is required for the plugin-principal extension —
`E_CAP_DENIED` covers the deny path uniformly; the layer that denied
is observable via the cap-chain trace. The `manifest_envelope_chain_validation`
seam (`crates/benten-caps/src/manifest_envelope_chain_validation.rs`,
G24-D-FP-2) joins manifest-envelope-shape enforcement with the UCAN
chain validator without introducing a sixth-class principal type at
the evaluator boundary.

The R4b-FP-1 Seam 3 `apply_atrium_merge` envelope-recheck-seam (post-Q4
ratification 2026-05-13) is tracked as **Compromise #26 (Phase-4-Foundation
manifest-envelope recheck on merge boundary) — PARTIALLY CLOSED** — see
[`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) "Compromise #26" for the
full seam-vs-adapter narrative (the `ManifestEnvelopeRechecker` port +
default-flip ship; the production-default `NoopManifestEnvelopeRechecker`
returns `NotApplicable` for every row at HEAD, so the substantive Layer-2
defense is NOT live in shipped binaries yet; the
`ProductionManifestEnvelopeRechecker` adapter is deferred to Phase-4-Meta
per `docs/future/phase-4-backlog.md §4.36`). Inv-14 doesn't gain a new
device-grain slot; the recheck (when live) happens AFTER the per-row
cap-revocation check + before the AttributionFrame is constructed on the
receiver side, so the frame remains the invariant's source of truth.

---

## What "active" means in this table

A row is **active** iff:

1. The invariant has a typed `RegistrationError` or `ErrorCode` it
   raises on violation.
2. The invariant has at least one regression test pinning the firing
   condition.
3. The crate that owns enforcement consumes the invariant on every
   relevant code path (i.e. there is no observable code path that
   bypasses it without explicit named-compromise documentation in
   `docs/SECURITY-POSTURE.md`).

All 14 invariants meet (1) (2) (3) at Phase 4-Foundation close.
