# Security Posture — Benten Engine (Phase 3 close)

This document records the security claims Benten makes through Phase 3
close and the known compromises those claims rest on. This document is
the written, referenceable form.

## Phase 3 close — final compromise table

| # | Title | Phase | Status |
|---|-------|-------|--------|
| 1 | TOCTOU window bound at CALL entry + ITERATE batch boundary | 1 | Open (bounded; documented threat model). **Revisit at v1-window** per phase-3-backlog §10.1. |
| 2 | Symmetric-None + diagnostic capability (Option C) | 1 | **CLOSED** at Phase 2a 5d-J |
| 3 | `ErrorCode` enum lives in `benten-core` | 1 | **CLOSED** at Phase 1 R6 |
| 4 | WASM runtime is compile-check only | 1 | **CLOSED** at Phase 2b G7 |
| 5 | No write rate-limits; metric recorded only | 1 | Open (architectural). **Revisit at v1-window** per phase-3-backlog §10.2. |
| 6 | BLAKE3 128-bit effective collision resistance | 1 | Open (architectural bound). **Revisit at v1-window** per phase-3-backlog §10.3. |
| 7 | `[[bin]]` `required-features` gating | 1 | **CLOSED** at Phase 1 R6 |
| 8 | `Engine::call` bypasses the evaluator for CRUD handlers | 1 | **CLOSED** at Phase 2a G4-A |
| 9 | Dedup writes pure-read (sec-r1-4 / atk-3) | 1 | **CLOSED** at Phase 2b G12-E |
| 10 | Resume-time capability re-verification | 2a | **CLOSED** at Phase 2b G12-E |
| 11 | IVM views coarse-grained read-gate | 2a | **CLOSED** at Phase-3 G15-A wave-5a (per-row `IvmViewReadGate` + addendum at G20-A3 documenting `read_view_with` heuristic bound) |
| 12 | `DurabilityMode::Group` gate 5 — engine-surface default flip + bench CI promotion | 1 | **CLOSED** at Phase 3 G13-E |
| 13 | System-zone reserved-prefix rejection surface | 2a | Open (documented; minor-3). **Revisit at v1-window** per phase-3-backlog §10.4. |
| 14 | SANDBOX cold-start cost (no opt-in pool) | 2b | Open (D3 RESOLVED — additive Phase-3 change if real-workload bottleneck). **Revisit at v1-window** per phase-3-backlog §10.5. |
| 15 | `register_runtime` reserved with deferred error | 2b | Deferred to Phase 8 (marketplace) — named destination per phase-3-backlog. |
| 16 | `random` host-fn deferred (no CSPRNG framework chosen) | 2b | **CLOSED** at Phase-3 G17-A2 wave-5b (CSPRNG via `getrandom` direct + capability-gated entropy budget per call: 4096 bytes default + per-manifest override at `host_fns.random.budget_bytes_per_call` per r1-wsa-8; constant-time cap-policy check per sec-r1-3) |
| 17 | In-memory module-bytes registry (`Engine::register_module_bytes`) | 2b | **CLOSED** at Phase-3 G14-C wave-4b (durable `RedbBlobBackend` + CID-validating entry point) |
| 18 | In-memory handler-version chain (`Engine::register_subgraph_replace`) | 2b | **CLOSED** at Phase-3 G14-C wave-4b (durable `system:HandlerVersion` zone + extensible canonical-bytes encoding per arch-r1-4 / D-C) |
| 19 | Browser-target persistent storage absent — manifests in-memory only on `wasm32-unknown-unknown` | 2b | **PARTIALLY CLOSED** at Phase-3 G18-A wave-5a (IndexedDB schema + handler scaffolding; full closure deferred per phase-3-backlog §4.3) |
| 20 | Cross-browser determinism CI cadence not yet established | 2b | **PARTIALLY CLOSED** at Phase-3 G18-A wave-5a (Playwright matrix workflow exists; fixture bodies deferred per phase-3-backlog §4.3) |
| 21 | Module manifest minimal CID-pin in Phase 2b; full Ed25519 deferred | 2b | **CLOSED** at Phase-3 G14-C wave-4b (Ed25519 sign + UCAN-proof-chain primary + publisher-key-registry fallback per D-PHASE-3-20 + crypto-minor-5) |
| 22 | Peer-DID + connection metadata leakage to public iroh relays | 3 | Introduced at Phase 3 (Phase 7 Garden-relay closure target). **Revisit at v1-window** per phase-3-backlog §10 (Phase-7 Garden-relays primary closure path; Phase-9 hardened-deployment fallback). |
| 23 | Wire device-attestation envelope cryptographic closure | 3 | **CLOSED** at Phase-3 G16-D wave-6b fix-pass (cryptographic shape CLOSED inline; **operator-deployment `FreshnessPolicy` override REQUIRED for production** — see body) |
| 24 | Wallclock fail-closed posture (no default-clock-zero expiration bypass) | 3 | **CLOSED** at Phase-3 G16-B-B-rest (PR #158); engine refuses to initialize UCAN backend without explicit clock injection — surfaces `E_UCAN_CLOCK_NOT_INJECTED` |
| 25 | HLC-monotonic enforcement at sync layer (adversarial-peer wallclock-injection defense) | 3 | **CLOSED** at Phase-3 sync-attack test family (HLC monotonicity + nonce-cache for replay defense + HLC bound inside signed envelope) |

**Phase-2b net delta:** Compromises #4 + #9 + #10 closed (3 net
closures); 8 new Phase-2b deferrals enumerated (#14, #15, #16, #17,
#18, #19, #20, #21) — all named, all destination-tagged. Compromises
#19, #20, #21 were lifted from MODULE-MANIFEST.md's local "#N+X" table
into the global numbering at R6 phase-close so cross-doc references
resolve to a single authoritative compromise table.

**Phase-3 G13-E delta (this row's landing):** Compromise #12 closed
at Phase-3 R5 wave-3 G13-E — `DurabilityMode::default()` flipped
`Immediate` → `Group` at the engine surface +
`.github/workflows/bench.yml` promoted from informational to
required (PR-trigger compile gate + CRUD fast-path APFS-relevant
bench subset). See the Compromise #12 section below for the full
closure narrative + the redb-collapse caveat.

**Phase-3 additive delta (introduced at phase-3 close, this row's
landing):** Compromise #22 records the peer-DID + connection-metadata
leakage to public iroh relays exposed by the Atrium P2P sync transport.
Full narrative below; closure target is the Phase-7 Garden-relay
infrastructure (with Phase-9 hardened-deployment as the brutal-but-
correct fallback if Garden-relays slip).

The detailed text for each numbered Compromise follows below. Phase-2b
additions (#14-#16) appear at the end of the Named Compromises
section. Phase-3 addition (#22) appears at the very end.

**Attack-surface matrix cross-reference.** The doc-level enumeration of
every named attack surface (Phase-2b SANDBOX ESC-1..16 + Phase-3 P2P-sync
surfaces) lives at [`docs/ATTACK-SURFACE-MATRIX.md`](ATTACK-SURFACE-MATRIX.md)
(authored at Phase-3 R5 wave-9 W9-T2 closing
`docs/future/phase-3-backlog.md` §7.13 sec-r4r2-2 / sec-r4r1-4). This
file remains the authoritative single-source-of-truth for the
per-compromise prose; the matrix complements it by serving as the
meta-completeness audit destination at every R6 phase-close (a checklist
that every named attack surface has at least one driving test pin).

## Named Compromises

### Compromise #6 — BLAKE3 128-bit effective collision resistance

Benten uses **BLAKE3-256** with a 32-byte digest embedded in every CIDv1. The academic collision-resistance bound for any cryptographic hash is `2^(n/2)` (birthday bound), giving BLAKE3-256 a **128-bit effective collision resistance**. This is the bound that every Benten Phase-1 security argument rests on — NOT the full `2^256` preimage bound.

**Where this matters:**

- **Content-addressed Nodes (`Cid`).** A collision would allow a malicious writer to forge a Node that hashes to the same CID as a legitimate Node — a "masquerade" attack. 128-bit resistance requires ~`2^128` hashes to find a collision; infeasible under any classical threat model.
- **Version-chain `prior_head` threading** (`benten_core::version::append_version`). The API uses CIDs to name the head each writer observed. A collision on a CID used as `prior_head` could, in principle, let an attacker smuggle an alternative chain past the fork-detection check. The same 128-bit bound applies.
- **Phase 3 UCAN-by-CID.** Phase 3 references capability grants by CID (landed at G14-B wave-5a durable UCAN backend). Revoke-by-CID paths assume the CID of a grant is unique; again, 128-bit collision resistance is the assumption.

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

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10.3):
- *Question to ask:* Does the v1 deployment posture (full peer + thin compute surfaces) introduce content-addressing surfaces that require post-quantum-floor collision resistance — i.e., do any v1 audit assumptions rest on `2^128` being out of reach for nation-state adversaries with future quantum hardware?
- *What changed pre-v1 vs Phase-1 framing:* the threat model widened from single-machine engine to multi-device Atriums + thin-client adjacents (browsers, edge runtimes); content-addressing identity now feeds DID-bound capability chains + UCAN-by-CID grants whose forgery cost matters for cap-policy integrity.
- *No-action option:* keep BLAKE3-256 as the architectural floor; document the post-quantum reconsideration as a Phase-N+ trigger (parallel to the IPFS-gateway trigger above). The 128-bit bound is sufficient for the personal-AI-assistants threat model + standard-classical-adversary v1 audit.

---

### Compromise #2 — Symmetric-None + diagnostic capability (Option C) — CLOSED

**Status (2026-04-17, 5d-J workstream 1):** migrated from Option A (honest-but-existence-leaking `E_CAP_DENIED_READ`) to **Option C** (symmetric `None` on denial, diagnostic-capability escape hatch). The existence-leak surface the prior posture named is no longer live; the escape hatch gives operators the signal they need without exposing it to ordinary callers.

**Status (2026-05-05, Phase-3 G14-D wave-5a):** D5 SUBSCRIBE per-event read-cap-coverage CLOSED at G14-D. The Phase-2b coarse-boolean cap-recheck shape (consulted only `is_actor_active`) is replaced by a per-event closure constructed via [`benten_engine::cap_recheck::CapRecheckFn`] + the durable UCAN backend at G14-B; a partial revoke that strikes the actor's read coverage observably cancels the affected subscription path mid-stream via the `E_SUBSCRIBE_REVOKED_MID_STREAM` typed error. The dual-layer per CLR-2 / cap-major-2 — subscribe-time gate AND delivery-time per-row gate — is wired via the new `Engine::on_change_with_cap_recheck` entry point in `crates/benten-engine/src/engine_subscribe.rs`. Cross-trust-boundary filtering happens at DELIVERY (registration is open per plan §3 G14-D); the load-bearing reversal of the Phase-2b interim shape. Composes with the G15-A IVM materialization-time per-row read-gate at `crates/benten-engine/src/ivm_view_read_gate.rs` (both consumers share the `cap_recheck.rs` G13-pre-C scaffold per ds-r4r2-7). Every test under `crates/benten-engine/tests/subscribe_cap_recheck.rs` is the regression surface.

**Status (2026-05-09, Phase-3 G16-B-F PR #161):** sec-r4r1-2 BLOCKER — sync-replica WRITE per-write cap-recheck-at-delivery (the symmetric counterpart to D5's SUBSCRIBE-side CLR-2 dual-layer recheck) — CLOSED end-to-end. Pre-G16-B-F, the SUBSCRIBE side carried 3 cap-recheck pins driving the `on_change_with_cap_recheck` delivery-time gate; the symmetric sync-replica WRITE side carried ZERO production-driven cap-recheck. PR #161 wires structural-always-on per-row cap-recheck inside `Engine::apply_atrium_merge`'s post-merge loop (Ben's ratified Option (a) — NO source enum, NO bypass flag, NO caller-picks privileged shortcut; "err on the side of security generally and then open safe QOL paths later"). Every row produced by Loro merge passes through the engine's `CapabilityPolicy::check_write` hook + an in-memory `revoked_actor_zone_pairs` set; recheck fires BEFORE Version Node mint so a single revoked row vetoes the whole merge atomically (CURRENT does not advance). Typed rejection via new `E_SYNC_REVOKED_DURING_SESSION` ErrorCode + `EngineError::SyncRevokedDuringSession { peer_did, zone, cid }` variant; observable via `Engine::sync_replica_cap_recheck_calls()` AtomicU64 counter (cap-r4-3 / r4b-cap-4 reinforcement). Both pins at `crates/benten-engine/tests/sync_replica_attribution.rs::sync_replica_write_cap_recheck_at_delivery_against_local_grant_store` + `sync_replica_write_after_local_grant_revoke_post_handshake_rejected_with_e_sync_revoked_during_session` un-ignored end-to-end. CLR-2 dual-layer recheck now wired symmetrically at both SUBSCRIBE delivery AND sync-replica WRITE delivery — both halves landed pre-tag per phase-3-backlog §6.12 item 6.

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
- **SUBSCRIBE D5 per-event read-cap-coverage.** `Engine::on_change_as_with_cursor` (Phase 2b G6-A / wave-8c) builds the delivery-time cap-recheck closure as `move |_event| -> bool { inner.is_actor_active(&actor_cid) }` inside `crates/benten-engine/src/engine_subscribe.rs::on_change_as_with_cursor`. The closure consults a flat `revoked_actors` set — a coarse boolean per-actor-revoked check — and **does NOT re-evaluate per-event read-cap-coverage** against the event's anchor CID. The eval-side `TestPrincipal::has_read_cap_for` (defined at `crates/benten-eval/src/primitives/subscribe.rs::TestPrincipal::has_read_cap_for`) has the right anchor-CID-keyed shape, but production napi/TS paths inherit the engine wrapper's boolean shape. Consequence: a *partial* revoke (operator removes the specific grant `store:post:read` from an active actor while leaving the actor active) does NOT auto-cancel an in-flight `onChange` subscription. *Full* actor revocation IS honoured. The per-event read-cap-coverage closure lands in Phase 3 alongside the durable grant-store / `benten-id` work; carry-forward destination is `docs/future/phase-2-backlog.md` §7.4 (Durable grant-store + SUBSCRIBE delivery-time cap-recheck). **Composition note (R6FP-G1 multi-label fix).** R6FP-G1 (PR #62) widened the SUBSCRIBE delivery matcher to walk every label of the source Node — a multi-labeled Node `["User","Admin"]` now correctly fires for both `User:*` and `Admin:*` subscribers (the prior single-primary-label behaviour silently dropped multi-label deliveries). Composed with the coarse-boolean cap-recheck above, a `User:*`-pattern subscriber whose actor is still active receives the FULL payload of multi-labeled Nodes including any Admin-tier labels — even if the actor lacks Admin-tier caps. Pre-R6FP-G1 the matcher consulted only the primary label so this widening surface was masked; post-fix the multi-label walk is correct (the prior single-label behaviour was the bug) AND the cap-recheck coarseness becomes more visible. Closes when Phase-3 `phase-2-backlog.md` §7.4 lands per-event read-cap-coverage.

**`E_CAP_DENIED_READ` code:** retained in the catalog (`docs/ERROR-CATALOG.md`) because Phase-2 evaluator-path READ enforcement still needs a typed denial code for the evaluator-visible leg — the Option-C public API mapping is an engine-orchestrator concern, not a catalog removal. The `CapError::DeniedRead` variant remains the signal policies use to communicate "denied" to the engine; the engine maps it onto `Ok(None)` at the public boundary.

**Regression tests:**

- `crates/benten-eval/tests/read_denial.rs` — six Option-C tests covering symmetric-None on `get_node`, `edges_from`, the three `diagnose_read` outcomes (`exists_but_denied`, `not_found`, NoAuth-open), and the `debug:read` gate.
- `crates/benten-engine/tests/integration/compromises_regression.rs::compromise_2_option_c_symmetric_none_plus_diagnose_read` — engine-level regression.
- `crates/benten-eval/tests/read_denial.rs::compromise_2_option_c_is_documented` (the eval-side doc-grep regression that keeps this section load-bearing — asserts the SECURITY-POSTURE Compromise #2 narrative remains in the doc).

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

### Compromise #4 — WASM runtime is compile-check only — CLOSED

**Closure provenance:** Phase 2b waves G7 + 8b + 8h (SANDBOX wire-through). The Phase-2b R4b post-impl audit surfaced that the prior G7-A scaffold left the production dispatch gate returning `PrimitiveNotImplemented` and the executor body returning an empty `SandboxResult` without ever instantiating wasmtime — the closure narrative was aspirational. **Wave-8b** wired the production dispatcher (`crates/benten-eval/src/primitives/mod.rs:96`) to `sandbox::execute(...)` and replaced the executor body with the real `Store + Linker + Instance` lifecycle, fuel/epoch/memory limiters, host-fn trampoline, `CountedSink` PRIMARY+BACKSTOP D17 enforcement, and trap → typed-error mapping with the D21 priority resolver. **Wave-8h** then closed the docs-vs-code audit's three audit-gap drifts (manifest-registry hydration, EMIT broadcast, IVM Algorithm B production registration) so the Named-manifest dispatch path consults the engine's `installed_modules` state.

**Original scope (Phase 1):** the `bindings/napi` crate compiled with `--target wasm32-unknown-unknown` in CI (`wasm-checks.yml`) but did NOT execute a WASM runtime (browser / `wasmtime`) at test time. The Phase-1 WASM surface existed only to guarantee that the napi bindings built for a browser target so Thrum (the Phase-4 consumer) could compile them into its web bundle.

**What now ships at Phase 2b (post-wave-8b/8h):**
- A live `wasmtime` host inside `crates/benten-eval/src/primitives/sandbox.rs` runs guest WebAssembly modules per-call (D17 instance lifecycle), with the four enforcement axes (memory / wallclock / fuel / output) bounded by the defaults documented in `docs/SANDBOX-LIMITS.md`. The production engine path routes through the `impl PrimitiveHost for Engine::execute_sandbox` override at `crates/benten-engine/src/primitive_host.rs` which reads module bytes via `Engine::module_bytes_for(cid)`, hydrates the `ManifestRegistry` from `installed_modules`, builds the `SandboxConfig` from the engine's policy + the operation node's properties, and invokes `benten_eval::sandbox::execute`.
- A capability-derived host-function manifest (`crates/benten-eval/host-functions.toml`, G7-A owned) controls which host-fns each guest may import. Capability resolution happens at instance-init time per call; revocation between calls is honoured. Wave-8h hydrates the registry from the engine's `installed_modules` set so Named-manifest dispatch (e.g. `manifest: "compute-power"`) resolves through the same path that `Engine::install_module` persists state into.
- ESC defense matrix (16 named escape vectors per `pre-r1-security-deliverables.md` §1). The canonical numbering in the inventory + the test corpus at `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs` is authoritative; this matrix uses the same numbering. The 16 vectors split into four buckets post-wave-8b/8h:

  | # | Vector | Defense mechanism | Runtime status | Test pin |
  |---|--------|-------------------|----------------|----------|
  | ESC-1 | OOB linear-memory read | wasmtime bounds-check trap → `trap_to_typed::map_call_error` → `SandboxModuleInvalid` | Fully wired | `sandbox_escape_attempts_denied.rs:76` (`sandbox_escape_oob_linmem_read_traps`) |
  | ESC-2 | Linear-memory grow beyond per-call cap | `SandboxResourceLimiter::memory_growing` returns `Err(MemoryCapExceededMarker)` → marker downcast at `trap_to_typed.rs:120-126` → `SandboxMemoryExhausted` | Fully wired (fixture re-authored wave-8d-narrative; see wave-8 §r6-wsa-4 dead-branch nit) | `sandbox_escape_attempts_denied.rs:85` (`sandbox_escape_linmem_grow_to_limit_kills`) |
  | ESC-3 | Host-buffer overrun via host-fn output write | `kv:read` trampoline bounds-check inside `crates/benten-eval/src/primitives/sandbox.rs::register_default_host_fns` → `Trap::MemoryOutOfBounds` → `SandboxModuleInvalid` | Fully wired | `sandbox_escape_attempts_denied.rs::sandbox_escape_host_buf_overrun_rejected` |
  | ESC-4 | Infinite loop without fuel | `Store::set_fuel(config.fuel)` → `Trap::OutOfFuel` → `SandboxFuelExhausted` | Fully wired | `sandbox_escape_attempts_denied.rs:147` (`sandbox_escape_infinite_loop_fuel_bound`) |
  | ESC-5 | Recursive-call stack overflow | wasmtime `Config::max_wasm_stack(512 KiB)` → `Trap::StackOverflow` → **Phase-3 G17-A1 wave-5b: dedicated `SandboxStackOverflow` typed variant** (formerly catalog-folded into `SandboxModuleInvalid`; r6-wsa-8 BELONGS-NAMED-NOW deferral retired). Maps to `E_SANDBOX_STACK_OVERFLOW` per phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure. Cascade through `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error` + napi error-mapping at `bindings/napi/src/error.rs::engine_err`. | Fully wired (dedicated typed variant landed at G17-A1) | `sandbox_escape_attempts_denied.rs:170` (`sandbox_escape_recursive_call_overflow_traps`) + `sandbox_stack_overflow.rs::sandbox_stack_overflow_routes_to_e_sandbox_stack_overflow_typed_variant` |
  | ESC-6 | Fuel-counter overflow regression | wasmtime saturated fuel bookkeeping; per-call `set_fuel` budget independent of guest run-time → `SandboxFuelExhausted` | Fully wired | `sandbox_escape_attempts_denied.rs:199` (`sandbox_escape_fuel_overflow_regression_held`) |
  | ESC-7 | Fuel-refill via host-fn re-entry | **Phase-3 wave-5c: Fully wired end-to-end.** Per-call `Store` lifecycle (D3-RESOLVED no-pool) + `SandboxStoreData.esc_defense_state: EscDefenseState` carries `re_entry_count` + `guest_active` flag (set by `enter_guest`/`exit_guest` immediately around `func.call` in `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check`). The host-fn boundary `run_all_checks` invocation surfaces `EscapeAttemptMarker` which `map_call_error` unwraps to `SandboxError::EscapeAttempt(Esc7FuelRefillViaReEntry)`. Routes through dedicated `E_SANDBOX_ESCAPE_ATTEMPT` catalog code per phase-3-backlog §6.1 + §6.1-followup task #3 + r1-wsa-1 BLOCKER closure + D-E (R1-revision triage). | Fully wired (end-to-end pin against `Sandbox::execute` driven through `wasmtime::Module` + `Instance::call`) | `sandbox_esc_runtime_arms_e2e.rs::esc_7_runtime_arm_fires_via_time_host_fn_re_entry_injection` (end-to-end) + `sandbox_esc_7.rs::esc_7_fuel_refill_via_host_fn_re_entry_blocked` + `..._traps_typed_error` (SHAPE pins; superseded by the e2e pin) — green at wave-5c |
  | ESC-8 | Call host-fn not in manifest | `Linker::func_wrap` only registers manifest-allowlisted host-fns; missing import → wasmtime "unknown import" → `SandboxHostFnNotFound` | Fully wired | `sandbox_escape_attempts_denied.rs:247` (`sandbox_escape_host_fn_not_on_manifest`) |
  | ESC-9 | Cap-revoke mid-call (TOCTOU between cap-grant and cap-use) | **Phase-3 wave-5c: Fully wired end-to-end.** D18 `PerCall` live-recheck via `LiveCapCheck` callback (`Arc<dyn Fn(&str) -> bool + Send + Sync>`) consulted from the trampoline `cap_check` helper BEFORE EVERY host-fn invocation per r1-wsa-3 MAJOR (no caching window). The engine override at `crates/benten-engine/src/primitive_host.rs::execute_sandbox` constructs the callable as a closure capturing `Arc<Mutex<HashSet<Cid>>>` cloned from the engine's revoked-actors set + the dispatching actor CID; mid-call revocation flips the actor's revoke bit and the next host-fn invocation surfaces `SandboxError::HostFnDenied`. Cadence is once-per-host-fn-entry (cadence (a) per r1-wsa-3 disposition + r4-r1-wsa-4 — within a single host-fn call, the recheck does NOT re-fire per loop iteration). Closes phase-3-backlog §6.3 + §6.1-followup task #5. | Fully wired (end-to-end pin against `Sandbox::execute` driving `kv_read` twice with mid-call revoke) | `sandbox_esc_runtime_arms_e2e.rs::esc_9_runtime_arm_fires_via_live_cap_check_revoke_mid_call` (end-to-end) + `sandbox_capability_check_per_call_after_revoke.rs::sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent` + `sandbox_esc_9.rs::esc_9_live_cap_check_fires_at_every_host_fn_boundary_no_caching_window` + `..._within_kv_read_loop_consults_once_per_call_not_per_iteration` — green at wave-5c |
  | ESC-10 | Re-entrancy via host-fn (cap-context confusion via SANDBOX → CALL → SANDBOX) | `AttributionFrame.sandbox_depth` runtime threading bumps depth at SANDBOX entry (see `crates/benten-engine/src/primitive_host.rs::execute_sandbox` saturating-bump on the parent `ActiveCall`); `SandboxError::NestedDispatchDepthExceeded` fires above ceiling at `crates/benten-eval/src/primitives/sandbox.rs::execute` → `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED`. Wired via R6FP-G1 (PR #62) 3-lens convergent fix | Wired-defense / test-paper-only — defense fires; integration test `#[ignore]`'d pending `testing_call_engine_dispatch` helper (see [`docs/future/phase-3-backlog.md` §7.3.A.7](future/phase-3-backlog.md)) | `sandbox_escape_attempts_denied.rs:291` (`sandbox_escape_reentrancy_via_host_fn_denied`) — `#[ignore]` |
  | ESC-11 | Component-Model type mismatch | wasmtime component-model linker type-check → `SandboxModuleInvalid`. wasmtime workspace dep at `Cargo.toml:299` ships `["runtime", "cranelift", "std", "async"]` — explicitly NO `component-model` feature; defense IS the cut | Component-model gated (`#[cfg(feature = "component-model")]` + `#[ignore]`) | `sandbox_escape_attempts_denied.rs:313` (`sandbox_escape_component_type_mismatch_rejected`) — feature-gated |
  | ESC-12 | Resource handle forgery | wasmtime component-model resource-handle table validates handles → `SandboxModuleInvalid` or `SandboxHostFnDenied` | Component-model gated (same cut as ESC-11) | `sandbox_escape_attempts_denied.rs:330` (`sandbox_escape_resource_handle_forgery_rejected`) — feature-gated |
  | ESC-13 | Trap during fuel-meter callback / Store-state corruption | **Phase-3 wave-5c: Fully wired end-to-end.** A `std::panic::catch_unwind` wrapper around `func.call` in `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check` catches host-side panics (fuel-meter callback OR any panicking host-fn closure); the wrapper sets `esc_defense_state.fuel_meter_callback_trapped = true` + surfaces `SandboxError::EscapeAttempt(Esc13StorePoison)` directly (no wasmtime trap unwinds through host frames). Pairs with D3-RESOLVED per-call `Store` lifecycle: the (potentially-poisoned) `Store` is dropped on return; the next SANDBOX call gets a fresh `Store` (poison-recovery pin: `esc_13_recovery_path_next_call_fresh_store_no_poison_leak`). Closes r1-wsa-1 BLOCKER half-b + §6.1-followup task #4 + D-E. | Fully wired (end-to-end pin against `Sandbox::execute` + recovery-path pin proving the next call is uncontaminated) | `sandbox_esc_runtime_arms_e2e.rs::esc_13_runtime_arm_fires_via_panic_in_host_fn_callback` (end-to-end) + `..._recovery_path_next_call_fresh_store_no_poison_leak` (recovery) + `sandbox_esc_13.rs::esc_13_trap_during_fuel_meter_callback_store_poison_observable` (SHAPE pin) — green at wave-5c |
  | ESC-14 | Cap-claim forge in module bytes | Engine ignores embedded WASM custom sections for cap purposes; cap derivation is exclusively from the manifest passed at call time. `forged_cap_claim_section.wat` (committed) verifies that a forged section is silently ignored AND that subsequent `kv:read` calls fire `SandboxHostFnDenied` if the manifest didn't include them. D26 `.wasm`-bytes shipping for the escape corpus is a wave-8 noted gap (r6-wsa-5) | Partial / eval-side smoke (forged-section helper carry-forward; manifest-authoritative defense IS structurally correct in production code) | `sandbox_escape_attempts_denied.rs:378` (`sandbox_escape_forged_cap_claim_section_ignored`) |
  | ESC-15 | Named-manifest spoofing (typo / non-existent name) | `manifest_ref.resolve(&registry)` returns `Unknown`; no permissive fallback → `SandboxManifestUnknown` | Fully wired | `sandbox_escape_attempts_denied.rs:403` (`sandbox_escape_named_manifest_spoofing_rejected`) |
  | ESC-16 | Wall-clock leak via `time` host-fn fingerprinting | **Phase-3 wave-5c: Fully wired end-to-end.** `time` host-fn returns monotonic-coarsened values (100 ms granularity) AND the trampoline calls `crates/benten-eval/src/sandbox/fingerprint.rs::record_wallclock_write` on each invocation (populating the per-call `SandboxStoreData.tainted_addresses` side-table) + `read_collapse_state` (incrementing `esc_defense_state.fingerprint_correlated_reads` for tainted-cell hits). At the host-fn boundary `run_all_checks` fires `SandboxError::EscapeAttempt(Esc16FingerprintCollapse)` once the read counter reaches `FINGERPRINT_COLLAPSE_THRESHOLD` (3 reads-within-one-call) BEFORE the side-channel becomes guest-observable. Closes r1-wsa-4 + phase-3-backlog §6.1 + §6.1-followup task #2. | Fully wired (end-to-end pin: 3-call WAT fixture trips the threshold; below-threshold pin proves the defense is silent on legitimate use) | `sandbox_esc_runtime_arms_e2e.rs::esc_16_runtime_arm_fires_after_threshold_time_host_fn_calls` (end-to-end) + `esc_16_silent_below_threshold_two_time_calls_pass` (below-threshold) + `sandbox_escape_attempts_denied.rs::sandbox_escape_wallclock_fingerprint_via_time_coarsened` (1000-call loop) + `sandbox_host_fn_time.rs::sandbox_host_fn_time_returns_monotonic_coarsened_100ms` (host-fn-level coarsening) — green at wave-5c |

  **Bucket totals (16 vectors, each in exactly one bucket; updated at Phase-3 wave-5c close):** **Fully wired (12):** ESC-1, -2, -3, -4, -5, -6, -7, -8, -9, -13, -15, -16 — production runtime defense + end-to-end integration test passing. *(Note: ESC-5 routes through the dedicated `SandboxStackOverflow` typed variant + `E_SANDBOX_STACK_OVERFLOW` catalog code per phase-3-backlog §6.4 + r1-wsa-7. ESC-7 / ESC-9 / ESC-13 / ESC-16 promoted from "Wired-defense + simulation pin green (helper SURFACE)" at Phase-3 G17-A1 wave-5b → **Fully wired** at wave-5c via the production runtime arms wired in `crates/benten-eval/src/primitives/sandbox.rs::execute_with_live_cap_check` + the engine override in `crates/benten-engine/src/primitive_host.rs::execute_sandbox`; end-to-end pins drive `Sandbox::execute` and assert observable typed-error firing per pim-2 §3.6b in `tests/sandbox_esc_runtime_arms_e2e.rs`.)* **Partial (1):** ESC-14 (production manifest-authoritative defense structurally correct — embedded WASM custom sections silently ignored for cap purposes; integration test `#[ignore]`'d pending `testing_inject_forged_cap_claim_section` helper full body — G17-A1 ships the helper SURFACE; G20-A1 fills the body). **Wired-defense / test-paper-only (1):** ESC-10 — `AttributionFrame.sandbox_depth` runtime threading wired in `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (R6FP-G1 / PR #62); the eval-side runtime arm in `crates/benten-eval/src/primitives/sandbox.rs::execute` fires `SandboxError::NestedDispatchDepthExceeded` once `attribution.sandbox_depth > config.max_nest_depth`; the adversarial integration test stays `#[ignore]`'d pending the `testing_call_engine_dispatch` helper SURFACE — G17-A1 ships the helper SURFACE; G20-A1 fills the body. **Component-model gated (2):** ESC-11, -12 — `#[cfg(feature = "component-model")]` + `#[ignore]`; the wasmtime workspace dep at `Cargo.toml:299` explicitly omits the feature. **Total:** 12 + 1 + 1 + 2 = 16 (no double-counting). The honest headline: **13 of 16 vectors fire typed-error defense end-to-end against the production executor at Phase-3 wave-5c close** (12 with full integration tests + ESC-10 with runtime defense but `#[ignore]`'d adversarial test pending helper-body fill at G20-A1; ESC-14 partially covered). The remaining 2 (ESC-11, -12) are component-model feature-cut. Wave-5c closes r1-wsa-1 BLOCKER (ESC-7 + ESC-13 end-to-end) + r1-wsa-3 MAJOR (ESC-9 cap-revoke mid-call cadence + production override) + r1-wsa-4 MAJOR (ESC-16 fingerprint-collapse). Wave-5b's r1-wsa-7 BLOCKER (ESC-5 stack-overflow catalog) remains closed.
- Cross-platform behaviour:
  - **Native targets (Linux x86_64, macOS arm64, Windows x86_64):** SANDBOX executes guest modules. Per-call cold-start budget gated by `bench_thresholds.toml` per the D22 RESOLVED tiered numerics (see `docs/SANDBOX-LIMITS.md` §6).
  - **wasm32-unknown-unknown / wasm32-wasip1:** the SANDBOX executor is compile-time absent (`#[cfg(not(target_arch = "wasm32"))]`). The DSL surface (`subgraph(...).sandbox(...)`) stays present so authoring works in browsers; invocation surfaces the typed error `E_SANDBOX_UNAVAILABLE_ON_WASM` at execution time, with the wsa-14 actionable text directing operators to either Phase-3 P2P sync against a Node-resident peer or local-development via @benten/engine in a Node.js process.

**Regression tests pinning the closure:**
- `crates/benten-engine/tests/integration/sandbox_compile_time_disabled_on_wasm32.rs` — pins both halves of the compile-time gate (executor present on native, surfaces typed error on wasm32).
- `bindings/napi/test/sandbox_napi_bridge.test.ts` — pins the napi bridge's cfg-gated symbol set + the `sandboxTargetSupported()` introspection probe.
- `packages/engine/test/wasm_browser_target.test.ts` — pins the browser-target UX (DSL stays present, registration succeeds, invocation fails with the typed error).
- `packages/engine/test/sandbox.test.ts` — pins the DSL composition surface (no top-level `engine.sandbox(...)`; `SandboxArgsByName` vs `SandboxArgsByCaps` discriminated union) and the D24 default-knobs surfacing through `engine.describeSandboxNode(...)`.
- `crates/benten-engine/tests/security_posture_md_phase_2b_compromises_documented.rs::security_posture_compromise_4_marked_closed` — asserts THIS section header carries `— CLOSED`.

**Posture claim now in force:** the SANDBOX runtime is a load-bearing primitive. It is expected to run in Phase 2b deployments. The four enforcement axes and the capability-derived host-fn manifest constitute the supply-chain and runtime-isolation perimeter for untrusted-code execution; operators who require additional defence-in-depth (process-level isolation, separate `wasmtime::Engine` per tenant) layer those on top of — not in place of — the in-engine bounds.

**Inv-4 runtime threading — fully wired at R6FP-G1 (PR #62).** Both Inv-4 enforcement arms are now active at Phase 2b close. (1) **Registration arm:** `invariants::sandbox_depth::validate_registration` at `structural.rs:215, 387` walks the static-graph at registration time (was already wired pre-wave-8). (2) **Runtime arm:** `crates/benten-engine/src/primitive_host.rs::execute_sandbox` mutates the parent `ActiveCall.sandbox_depth` via `frame.sandbox_depth = frame.sandbox_depth.saturating_add(1)` on every production SANDBOX entry; the dispatching `AttributionFrame` is constructed with `sandbox_depth: nested_depth` in both match arms of the same function. Subsequent CALL pushes inherit the bumped depth via the dispatcher-inheritance read in `crates/benten-engine/src/engine.rs::dispatch_call_with_mode_and_trace` (`let parent_sandbox_depth = guard.last().map_or(0, |f| f.sandbox_depth)` immediately before the new `ActiveCall` push). The eval-side runtime arm in `crates/benten-eval/src/primitives/sandbox.rs::execute` fires `SandboxError::NestedDispatchDepthExceeded` when `attribution.sandbox_depth > config.max_nest_depth` (default `max_nest_depth = 4` admits depths 1..=4, depth 5 fires) — surfaces as `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED` through `trap_to_typed`. Carry-forward residual: the ESC-10 adversarial integration test `sandbox_escape_attempts_denied.rs::sandbox_escape_reentrancy_via_host_fn_denied` stays `#[ignore]`'d pending the `testing_call_engine_dispatch` host-fn helper per [`docs/future/phase-3-backlog.md` §7.3.A.7](future/phase-3-backlog.md). The runtime arm is wired; the adversarial-test driver is paper-only.

**Posture claim — per-call-only instance lifecycle is a security win by construction (D3-RESOLVED, sec-pre-r1-12).** Phase 2b ships the SANDBOX executor with a per-call `wasmtime::Instance` lifecycle (D17-RESOLVED) and explicitly NO opt-in instance pool (D3-RESOLVED). This is not solely a DX or perf decision — it is a security posture claim. With per-call instantiation:

- **No cross-call wasm-linear-memory leakage by construction.** A pooled instance shared across two SANDBOX calls would surface a hazard whenever a guest module wrote to a wasm-global without realising the global persisted; the per-call lifecycle removes that hazard by forcing every call to start from a freshly-instantiated module.
- **No cross-call capability-resolution leakage.** Host-fn closures resolve capability grants at instance-init time; per-call init means a capability revoked between two calls is honoured on the second call without per-checkout revalidation logic that a pool would require.
- **Cross-tenant isolation reduces to call ordering.** Two tenants sharing the same engine still get distinct instances per call — the absence of pooling means there is no shared instance whose internal state could be probed across a tenant boundary.

The cold-start cost is bounded by the D22 tiered numerics (≤2 ms p95 / ≤5 ms p99 Linux x86_64; ≤5 ms p95 / ≤10 ms p99 macOS arm64 + Windows x86_64 — see `docs/SANDBOX-LIMITS.md` §6); a measured breach is the trigger that would re-open D3 with real-workload data, NOT an arbitrary regression. If pooling ever lands in a future phase the security claim above must be re-stated at that point — Phase 2b's posture rests on the per-call lifecycle.

**Non-regression notes — Phase-2a security closures stay closed (sec-pre-r1-13).** Phase 2b MUST NOT re-open the Phase-2a security closures listed below; the closures continue to hold under the SANDBOX-bearing surface. Each closure has its own regression test that fires red on regression.

- **sec-r6r1-01 — Inv-14 attribution wired through every primitive frame.** The AttributionFrame routing path is the carrier for the D20 Inv-4 nested-dispatch depth counter (`AttributionFrame.sandbox_depth: u8` field, G7-B owned). Engine-side SANDBOX plumbing (`engine_sandbox.rs`, this G7-C file) routes through the evaluator's `PrimitiveHost` dispatch — there is no SANDBOX execution path that bypasses AttributionFrame propagation. Pinned by the existing Phase-2a Inv-14 regression suite plus the Phase-2b `tests/engine_sandbox_end_to_end_via_dsl_composition_only` integration test (the SANDBOX call inherits its parent attribution frame).
- **sec-r6r2-02 — `test-helpers` cfg-gating on the engine surface.** The `Engine::describe_sandbox_node(...)` accessor (G7-C; ts-r4-3 finding) is `#[cfg(any(test, feature = "test-helpers"))]`-gated so the napi cdylib (which opts into the narrower `envelope-cache-test-grade` feature only) does NOT compile this surface into production. The G12-C subgraph type relocation MUST preserve every existing `testing_*` surface gate; G7-C's new accessor adds to the gated set rather than relaxing any prior gate.
- **sec-r6r3-02 — parse-counter cfg-gate.** The G12-A BudgetExhausted runtime emission wiring (Phase-2b R5 wave-1) routes through the AttributionFrame path and does NOT bypass the parse-counter cfg-gate. The G7-C SANDBOX gate is independent of the parse-counter gate; both must remain.

**Wave-8h adjacencies — EMIT broadcast + IVM Algorithm B production registration.** The wave-8h audit-gap fixes addressed two surfaces orthogonal to the SANDBOX wasmtime-invocation axis but adjacent to Compromise #4's overall "Phase-2 primitives are wired end-to-end" posture:

- **EMIT engine wrapper** at `crates/benten-engine/src/primitive_host.rs::emit_event` was previously a documented no-op (`Phase-1 EMIT is a no-op at the host level`); a handler with a standalone EMIT primitive (no backing WRITE) silently dropped the payload. Wave-8h wired EMIT to publish through a dedicated `EmitBroadcast` channel (separate from the `ChangeBroadcast` channel which carries storage WRITE events). Public API: `Engine::subscribe_emit_events`. The two channels are intentionally separate — extending `benten_graph::ChangeEvent` with an emit variant would conflate two distinct event shapes (storage commits vs handler-emitted events) that currently serve different downstream consumers (IVM views + cap-recheck pipelines vs ad-hoc observers + log shippers). Phase-3 may converge them when iroh sync introduces a unified P2P event stream; until then the two channels stay distinct and `EmitBroadcast` lives in `crates/benten-engine/src/emit_broadcast.rs`.
- **IVM Algorithm B production registration** at `crates/benten-engine/src/engine_views.rs` was previously a `ContentListingView` fallback for every `Strategy::B`-declared user view (the view ran on the Phase-1 ContentListingView shape regardless of the spec's declared Strategy::B). Wave-8h wired the dispatch to construct `AlgorithmBView::for_id(spec.id())` for the 5 canonical view IDs that `AlgorithmBView` supports natively. **Phase-3 G15-A + G15-B + R5 wave-9 W9-T1 closure:** the non-canonical-view fallback is RETIRED — `Algorithm::register` (and the budget-aware sibling `Algorithm::register_with_budget`) instantiates a generic single-loop kernel (`GenericKernel`) for non-canonical view ids keyed on `(label_pattern, projection)`. The Phase-2b coverage compromise no longer applies: non-canonical view IDs no longer silently coerce to `ContentListingView`. The drift-detector proptest harness (`crates/benten-ivm/tests/algorithm_b_drift_detector.rs`, 5 pins × 1 000 cases each) is the verification surface for "incremental updates equal from-scratch rebuild" parity across the merged kernel. Closure tracking + cross-references in [`docs/future/phase-3-backlog.md` §5.1](../docs/future/phase-3-backlog.md).

**Cross-refs.** `docs/SANDBOX-LIMITS.md` (axes + per-platform cold-start); `docs/HOST-FUNCTIONS.md` (host-fn surface).

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

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10.2):
- *Question to ask:* Does the personal-AI-assistants threat model produce write-flood attack surfaces that can reach the engine ingress through the cap-policy permit gate? If yes: does a per-actor rate-limit (now that `benten-id` ships actor-Cid identity) close the flood vector without breaking legitimate bulk-import / federation-replay workflows?
- *What changed pre-v1 vs Phase-1 framing:* `benten-id` actor-Cid identity now exists (Phase-3 G14-A); the per-scope counter map persists across Phase-3 (still no eviction). Eviction + per-actor budget threads through the same `CapabilityPolicy` gate that Phase-3 hardened end-to-end.
- *LOC estimate at v1-window:* ~150-300 (per-actor budget enum on `CapabilityPolicy` + eviction policy on the per-scope counter map + integration tests for budget-exhaustion `ON_DENIED` routing).

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
  the embedded / single-process trust model (the engine lives in the
  caller's address space; there is no daemon).
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

## Phase 2a — Inv-13 immutability firing matrix (5-row)

Phase 2a G5-A adopts the firing matrix decided at R1 close (plan §9.11)
for Invariant 13 (immutability). Five rows cover the three
`WriteAuthority` variants plus a resume-time pre-check:

| # | WriteAuthority / Path | Content matches registered bytes | Outcome |
|---|---|---|---|
| 1 | `User` | yes | `E_INV_IMMUTABILITY` — canonical unprivileged immutability violation. Users cannot observe dedup on system-controlled surfaces. |
| 2 | `User` | no | `E_INV_IMMUTABILITY` — vacuous under content-addressing (CID-match ⇔ bytes-match); reached only via the test-only `put_node_at_cid_for_test` backdoor that injects bytes at a caller-supplied CID. Error naming kept for forward-compat with mutable-id extensions. |
| 3 | `EnginePrivileged` (version-chain append) | yes | `Ok(cid_dedup)` — pure-read dedup. Does NOT emit `ChangeEvent` and does NOT advance the audit sequence (Compromise "Dedup writes pure-read" below). |
| 4 | `SyncReplica { origin_peer }` (Phase-3 sync-receive) | yes | `Ok(cid_dedup)` — same no-event + no-audit semantics as row 3. Shape reserved in 2a; wired at Phase 3 receive-path. |
| 5 | WAIT-resume stale-pin pre-check (any authority) | (`pinned_subgraph_cids` no longer matches the anchor's CURRENT) | `E_RESUME_SUBGRAPH_DRIFT` fires BEFORE any write. Distinct code; mirrors arch-1 resume-step-3 (§9.1) in the Inv-13 matrix explicitly. |

`WriteContext` carries a `WriteAuthority` enum
(`User | EnginePrivileged | SyncReplica { origin_peer }`).
`EnginePrivileged` replaces the Phase-1 `privileged: bool` and is set
by the engine orchestrator for capability-grant-authorised
version-chain `NEXT_VERSION` appends; user subgraphs never reach it.
`SyncReplica` reserves a Phase-3 shape for replicated writes.

### Compromise #9 — Dedup writes pure-read (sec-r1-4 / atk-3) — CLOSED at Phase 2b G12-E

**Status (2026-04-27).** **CLOSED at Phase 2b G12-E.** The R5 wave-6 G12-E
landing replaces every process-local suspend / resume state surface
(`OnceLock<Mutex<HashMap>>` in `wait::registry`,
`LazyLock<Mutex<BTreeMap>>` in `engine_wait::ENVELOPE_CACHE`, the
SUBSCRIBE persistent-cursor in-memory placeholder) with a single
durable [`benten_eval::SuspensionStore`] port (default impl:
[`benten_engine::RedbSuspensionStore`] over the engine's existing
`Arc<RedbBackend>`). The dedup-row-3 audit-sequence pure-read
contract (this compromise's body) was already enforced at the
storage layer; G12-E removes the residual cross-process surface
where a privileged re-put racing against an envelope-cache lookup
could observe inconsistent state, completing the closure narrative
per plan §3.2 + R2 §6 row. The dedup invariant tests
(`crates/benten-graph/tests/inv_13_dedup_*`) continue to pin the
storage-layer guarantee unchanged; the new G12-E tests
(`crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs`)
pin the cross-process persistence layer the dedup contract sits on
top of.



**Class.** Audit-log forgery and audit-sequence side-channel leak
via the dedup path.

**Shape.** Row 3 (`EnginePrivileged` + content matches) returns
`Ok(cid_dedup)` as a successful idempotent dedup. If the
transaction machinery still pushed the dedup into `pending_ops` and
fanned out a `ChangeEvent`, an attacker with privileged-write reach
(version-chain append, grant re-issuance) could manufacture a
succession of audit events carrying fresh timestamps but bit-
identical content — inflating the audit trail and making
re-issuance look like distinct authorisations.

A companion side-channel: if the dedup path silently advanced the
audit sequence counter, an observer who can read the sequence
learns "a privileged actor re-visited this CID" even when the
visible audit log is empty.

**Mitigation.** Row 3 (and the Phase-3 row 4) **branch before**
`pending_ops.push`, before any ChangeEvent construction, and before
any audit-sequence advance. The privileged re-put with matching
bytes returns the existing CID with no observable effect on the
ChangeEvent stream or the audit counter. Tests:

- `crates/benten-graph/tests/inv_13_dedup_does_not_emit_changeevent.rs`
- `crates/benten-graph/tests/inv_13_dedup_path_does_not_advance_audit_sequence.rs`
  (engine-side accessor `testing_audit_sequence` lands in G11-2a)
- `crates/benten-graph/tests/inv_13_matrix.rs` (Row 3 no-event)

**SUBSCRIBE persistent-cursor retention bookkeeping — Phase-2b is
process-local; durable retention is a Phase-3 lift.** The G12-E port
makes suspended-WAIT envelopes durable across process restart. The
SUBSCRIBE persistent-cursor retention window (1000-events / 24h) is
enforced via the `SuspensionStore::is_retention_exhausted` trait method
on the in-memory test impl, but `RedbSuspensionStore` (the production
impl backing cross-process re-subscribe) does NOT override the trait
method — the default `false` means the in-memory `delivered_count` +
`registered_at` counters are reset on each process boot. Consequence: a
cross-process re-subscribe past the 1000-events / 24h window does NOT
surface `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED` today. Operationally
bounded: process restarts during the retention window are rare, and
the in-process retention enforcement is unchanged. The durable
retention bookkeeping is a Phase-3 lift paired with the durable
grant-store + per-event read-cap-coverage work — see
[`docs/future/phase-3-backlog.md` §6.5 RedbSuspensionStore retention-window override](future/phase-3-backlog.md).

**Residual risk.** The dedup pure-read contract is enforced at the
storage-layer entry points (`RedbBackend::put_node_with_context`).
A future code-path that accumulates its own ChangeEvent before
calling into the storage layer would need to re-check the row-3
branch at its own entry point; the `WriteContext::authority` enum
makes this explicit at the type level so reviewers catch
regressions.

**Residual risk (G5-A major — concurrent-writer TOCTOU).** Row 3
takes the dedup branch AFTER the `WriteContext::authority ==
EnginePrivileged` check against the in-memory `registered_cids`
set. Under concurrent writers, writer A can observe `cid ∉
registered_cids`, start the insert path, and race against writer B
who registers the same CID in between. The storage layer
serialises the two redb transactions so both end with the same
on-disk bytes, but the second writer's per-transaction audit-log
fan-out may already have emitted a ChangeEvent before the dedup
check re-runs under the transaction lock. The window is narrow
(two writers must race on the SAME CID, which under content-
addressing means the same bytes — dedup is the correct outcome
either way) and the emitted ChangeEvent carries legitimate
attribution, but the audit trail will show one extra event for the
racing writer. Tightening the window to bit-exact "exactly one
ChangeEvent per CID" is Phase-3 scope where the sync-replica row
needs the same invariant across peers anyway.

**Cross-refs.** plan §9.11 5-row matrix; R1 triage `sec-r1-4`,
atk-3, Code-as-graph Major #4; ERROR-CATALOG E_INV_IMMUTABILITY.

---

## Dual-layer read-capability explanation (ucca-10)

Phase 2a closes the gap the pre-R1 ucca-10 review opened: what
exactly gates a read, and at what layer?

The answer has two layers that enforce different parts of the
contract and that must both be present for the full posture to
hold.

### Layer 1 — Sync-receive gate (Phase 3)

At the network boundary, incoming replicated reads (Phase-3
`benten-sync` receive path) consult `CapabilityPolicy::check_read`
before the bytes are handed to the evaluator. This is the gate
that keeps a peer from force-feeding the engine Nodes its operator
did not authorise to observe — the federation / atrium boundary.
Phase 2a reserves the shape (see `SyncReplica { origin_peer }`
variant on `WriteContext::authority`); the wire is Phase-3 scope.

### Layer 2 — Evaluator-dispatch gate (Phase 2a Option C)

Inside the evaluator, every READ primitive routed through a
registered user subgraph calls
`PrimitiveHost::check_read_capability` before resolving the CID.
Denial collapses to `Ok(None)` — byte-identical with a miss — per
Compromise #2 Option C. This is the layer that keeps a user
subgraph, running under a partial grant, from using TRANSFORM-
computed CIDs to probe the existence of Nodes the caller cannot
read.

**Why both layers are necessary.** A sync-receive gate alone lets
a local compromised-but-unprivileged actor probe the store via
evaluator dispatch (the trust boundary is INSIDE the engine, not
at its edge). An evaluator gate alone lets a malicious peer
force-replicate Nodes that the operator had declined to grant
read — the data arrives and sits in the backend even though no
local caller can observe it. Both gates together pin both trust
boundaries.

**Phase 2a status.** Layer 2 is live (G4-A Option C flanking, sec-
r1-5 IVM views at coarse-grained per-view read; sec-r1-5
fine-grained per-row is Phase 3). Layer 1 is shape-reserved; the
wire lands with `benten-sync` in Phase 3.

**Cross-refs.** Compromise #2 (symmetric-None) closure; Compromise
"IVM views coarse-grained read-gate" below; plan §G4-A Option C
flanking; ucca-10 pre-R1 review.

---

### Compromise #10 — Resume-time capability re-verification (G3-A / G5-B-i Decision 4) — CLOSED at Phase 2b G12-E (cross-process metadata arm) and CLOSED at Phase 3 G14-D wave-5a (engine-side asymmetry arm)

**Status (2026-04-27).** The cross-process metadata arm of this
compromise is **CLOSED at Phase 2b G12-E**. The orchestrator
state log + brief refer to this closure as "Compromise #9" by
sequencing in the open-compromise tracker; the canonical doc
reference is #10.

**Status (2026-05-05, Phase-3 G14-D wave-5a).** The engine-side
asymmetry between WAIT-suspend and WAIT-resume is now **CLOSED**.
G14-D wires `cap_snapshot_hash` derivation
([`benten_engine::cap_snapshot_hash::compute(actor_cid, proof_chain_cids)`])
+ persisted-policy-metadata into a new [`benten_eval::CapSnapshot`]
side-table on the `SuspensionStore` (keyed by envelope CID). The
`resume_from_bytes_*` family recomputes the hash against the
chain currently in the durable cap store and rejects with
`E_CAP_SNAPSHOT_HASH_MISMATCH` when the chain materially changed
(e.g. one UCAN was revoked between suspend and resume) per
CLR-2 §11. A historical-policy metadata blob is preserved across
the suspend/resume boundary so the resumed continuation runs
against the policy in effect at suspend (rate-limit budgets,
attenuation depth). Regression surface lives at
`crates/benten-engine/tests/wait_resume_cross_process.rs` +
`crates/benten-engine/tests/wait_resume_policy.rs` +
`crates/benten-engine/tests/ucan_replay_audience.rs`.

**Class.** Stale-authority resume + cross-process metadata gap.

**Shape.** The WAIT resume protocol re-verifies the caller's
capability at step 4 of the 4-step protocol (`resume_from_bytes` →
envelope integrity check → actor pin check → subgraph-pin check →
**capability re-check** against the current policy state).

This is the intended defence against a grant revoked between
suspend and resume: even if the suspended envelope names a valid
actor CID, the re-check catches a policy that has since removed
the underlying grant.

**Residual gap — cross-process persisted metadata (CLOSED at G12-E).**
Phase-2a parked WAIT suspend metadata (deadline + signal-shape) and
the persisted `ExecutionStateEnvelope` bytes in two ad-hoc
process-local surfaces — `OnceLock<Mutex<HashMap<Cid, WaitMetadata>>>`
in `benten-eval/src/primitives/wait.rs` and
`LazyLock<Mutex<BTreeMap<Cid, ExecutionStateEnvelope>>>` in
`benten-engine/src/engine_wait.rs` (the test-grade `ENVELOPE_CACHE`
gated behind `envelope-cache-test-grade`). Either surface dropped
its state on `Engine::drop`, so a resume in a fresh process either
silently completed the WAIT (the eval-layer permissive
`Complete(value)` fallback) or failed with `E_NOT_FOUND` on
`suspend_to_bytes` (the engine-layer cache miss). Phase 2b G12-E
collapses both surfaces into a single durable port:
[`benten_eval::SuspensionStore`] with the redb-backed default impl
[`benten_engine::RedbSuspensionStore`] over the engine's existing
`Arc<RedbBackend>`. WAIT metadata, envelope bytes, AND SUBSCRIBE
persistent cursors all round-trip through this store; the
`OnceLock` registry + `ENVELOPE_CACHE` static + the
`envelope-cache-test-grade` feature are retired. The Phase-2a
permissive `resume_with_meta` fallback is rewritten to fail loud
with `E_HOST_BACKEND_UNAVAILABLE` so a missing metadata entry
post-G12-E surfaces a typed error rather than silently admitting
an attacker-supplied payload.

**Honest disclosure — fail-closed asymmetry between eval-side and engine-side resume surfaces.** The fail-closed semantics applies to the eval-side `benten_eval::resume_with_meta` API (callers with stricter integrity expectations — the public API used by tests and any future direct-eval consumers). The engine-side bytes-only resume surfaces (`Engine::resume_from_bytes`, `resume_from_bytes_as`, `resume_from_bytes_unauthenticated`) treat a missing `WaitMetadata` entry as best-effort *skip the inline-deadline check and proceed* rather than fail-closed — see the deliberate inline comment inside `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner` (the metadata-store lookup arm) for the rationale. The shape tolerates legitimate cross-process eviction without breaking resume (a fabricated test envelope, a non-WAIT resume reaching this surface, or a store miss after legitimate eviction in cross-process scenarios all skip the deadline-check rather than failing). The downstream Step 2 principal-binding + Step 3 capability re-check still run, so an attacker cannot use the asymmetry to bypass the cap re-check — only the inline deadline check is skipped on missing metadata. The narrow attack class (attacker who forges an envelope + shared SuspensionStore eviction window must coincide) is bounded by the cap re-check arm; the divergence is documented here so operators reading Compromise #10 don't assume both surfaces are uniformly fail-closed.

Regression tests:
`crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs`
(`wait_resume_cross_process_metadata_survives_restart`,
`resume_with_meta_fails_closed_when_metadata_missing`,
`subscribe_persistent_cursor_survives_engine_restart`,
`subscribe_max_delivered_seq_round_trips_via_suspension_store`,
`suspension_store_handles_both_wait_and_cursor_keys_without_collision`).

**Capability re-check arm (UNCHANGED — Phase 3 scope).** G12-E
ships the durable persistence the re-check sits on top of; the
re-check itself still consults the resuming process's live
`CapabilityPolicy`. The original Decision-4 federation-aware
`cap_snapshot_hash` (envelope embeds the hash; resume asserts the
fresh policy's snapshot matches) is Phase-3 scope alongside the
`benten-id`-typed-principal work — sec-r1-5 deferral.

**Mitigation (Phase 2a).** For in-process resume, the check is
correct. For cross-process the Phase-2a default policy is to
refuse — `Engine::resume_from_bytes_as` distinguishes actor CIDs
but does not reach across the process boundary on its own; the
operator must explicitly hand the envelope to a new engine
instance and accept that the re-check semantics change. No
silent regression.

**Cross-refs.** Phase-2a resume protocol; the `ExecutionState`
envelope shape; the Phase-2b SuspensionStore landing (closed at Phase
2b).

---

### Compromise #11 — IVM views coarse-grained read-gate (sec-r1-5) — **CLOSED at G15-A** (Phase-3 R5 wave-5a)

**Class.** Over-read through an IVM view under a per-view grant.

**Status.** **CLOSED at G15-A** (Phase-3 R5 wave-5a, 2026-05).
Per-row read-gate at materialization time landed via
[`crates/benten-engine/src/ivm_view_read_gate.rs::IvmViewReadGate`]
which composes label-hint extraction with the
[`crates/benten-engine/src/cap_recheck.rs::CapRecheckFn`] actor-cap-set
check; the engine surfaces this through
[`crates/benten-engine/src/engine_views.rs::Engine::materialize_view_with_gate`].
The closure is end-to-end: G15-A's materialization-time gate composes
with G14-D's delivery-time gate at SUBSCRIBE per `cap-r4-3` —
deny-from-either-layer wins. The closure is pinned by:

- `crates/benten-engine/tests/ivm_read_gate.rs::ivm_view_per_row_read_gate_against_actor_cap_set`
  (LOAD-BEARING #11 closure pin — 100-row 50/50 fixture yields
  exactly 50 rows for an actor with public-only READ caps).
- `crates/benten-engine/tests/ivm_read_gate.rs::ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate`
  (`ivm-major-2` — gate is independent of SUBSCRIBE delivery layer).
- `crates/benten-engine/tests/ivm_read_gate.rs::materialize_view_with_gate_filters_rows_per_actor_cap_set_at_engine_entry_point_e2e`
  (LOAD-BEARING pim-2 §3.6b end-to-end pin — drives the production
  `Engine::materialize_view_with_gate` boundary with mixed-label
  Nodes written through the engine's transaction surface; asserts
  row-level filtering behavior that would FAIL if the gate were
  silently bypassed or if `materialize_view_with_gate` returned an
  empty list unconditionally).

The G15-B drift-detector proptest harness at
`crates/benten-ivm/tests/algorithm_b_drift_detector.rs` is the
companion verification surface for the merged
`benten_ivm::algorithm_b::Algorithm::register` kernel post-G15-B; it
does not pin Compromise #11's G15-A closure (which stands on the
materialization gate alone). The 5 active proptest pins are
`prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern`,
`prop_budget_trip_state_propagation_consistent`,
`prop_rebuild_after_stale_returns_view_to_fresh`,
`prop_drift_detector_observes_label_pattern_extension`,
`prop_drift_detector_reports_one_path_errors_other_succeeds`. R5
wave-9 W9-T1 retensed the harness's headline pin description in
`docs/future/phase-3-backlog.md` §5.1 alongside the
`Algorithm::register_with_budget` budget-knob lift.

**Shape (historical).** Phase 2a G4-A Option C threading gated
`Engine::read_view` at the per-view level: a caller who held a
`store:<view>:read` grant for view X could read ALL rows the view
returned. The gate did not differentiate between rows in the view
that come from source Nodes the caller could directly read vs.
source Nodes the caller could not. A view over a mixed-sensitivity
label set could therefore surface row data the caller lacked the
underlying per-Node grant for.

**Mitigation (historical Phase 2a).** Per-view grants were treated
as an explicit operator opt-in — granting `store:<view>:read` was a
conscious "this view is ok to read, whatever its underlying Nodes"
decision. The view-ID registry (user-authored views landed in
Phase 2b under `P2.ivm.user-views`) made it explicit that a view's
scope was defined at registration; operators were instructed not
to grant view-level read to actors they would not grant the union
of the source-label reads to.

**G15-A closure narrative.** Phase 3 G15-A retires the residual
risk by wiring per-row READ gating at materialization time (this
was the `Phase-3` deferred resolution path the original sec-r1-5
review named). The materialization gate consults the actor's
cap-set per row via [`benten_engine::cap_recheck::CapRecheckFn`]
— the same shared scaffold G14-D consumes for delivery-time
recheck per `ds-r4r2-7`. Because both layers compose:

- **Materialization deny wins:** a row whose underlying Node the
  actor cannot READ does NOT enter the materialised view, so the
  delivery layer never sees it.
- **Delivery deny wins:** a row that passed materialization is
  still subject to G14-D's per-event recheck at SUBSCRIBE; a
  partial-revoke mid-stream cancels delivery on rows the
  materialization gate had already admitted.

**Cross-refs.** Compromise #2 Option C; plan §G4-A; sec-r1-5
pre-R1 review; dual-layer read-cap section above; G14-D F6
SUBSCRIBE filtering at delivery boundary.

**Phase-3 G16-B-A canary deepening (2026-05-08).** G16-B-A's canary
landed three structural-surface pins that complete Compromise #11's
device-grain composition story alongside the Phase-3 sync-merge path:

- **Materialization-only structural pin** —
  `crates/benten-engine/tests/ivm_view_subscribe_compose.rs::compromise_11_materialization_deny_wins_over_delivery_admit_at_view_layer`
  asserts the materialization gate's row filtering operates
  independently of any SUBSCRIBE delivery state. The pin stands GREEN
  at canary scope.
- **Mat-deny-wins composition pin** — same file, asserts that a row
  the materialization gate denies is observably absent from the
  SUBSCRIBE delivery surface (mat-deny-wins composition over
  G14-D delivery-time recheck). Stands GREEN at canary scope.
- **Delivery-gate-registration pin** — pins that
  `Engine::materialize_view_with_gate` correctly registers its
  per-view gate hook against the same `CapRecheckFn` scaffold G14-D
  consumes for SUBSCRIBE delivery, so the layered defense is wired
  through one shared dispatch. Stands GREEN at canary scope.

**Phase-3 G16-B-D deepest-pin closure (2026-05-09).** The deepest
end-to-end composition pin —
`compromise_11_both_gates_compose_observable_delivery_end_to_end` —
is GREEN at G16-B-D using Option (a): a new
`Engine::testing_subscribe_observable_change_events` helper that
exposes the eval-side `ChangeEvent` directly to test callbacks
(bypassing the chunk-encoding bridge that the production
`OnChangeCallback` adapter applies). The helper is strictly cfg-gated
under `cfg(any(test, feature = "test-helpers"))` and lives in the
same `engine_subscribe.rs` module as the production
`on_change_with_cap_recheck`, sharing the same eval-side
`DeliveryCapRecheck` bridge — so the test surface does NOT widen the
production surface (echoes `sandbox_helpers_no_widening.rs` discipline;
see `docs/future/phase-3-backlog.md` §10.6 for the harder Cargo-feature-
graph defense-in-depth that may carry to v1-gate).

The pin asserts the load-bearing dual-gate composition shape:

- mat-admit ∩ delivery-admit = {row_A} (admitted by BOTH gates → in view AND delivered)
- mat-deny ∩ delivery-admit = {row_B} (mat-denied → suppressed at view layer; delivery-admitted → delivered to observer; proves delivery layer is independent of mat layer)
- mat-admit ∩ delivery-deny = {row_C} (delivery-denied → suppressed at delivery; mat-admitted → still in view; proves delivery-deny wins over mat-admit)

End-to-end intersection (rows admitted by BOTH layers) = {row_A}, the
load-bearing closure assertion of the dual-gate composition narrative.

The would-fail-if-no-op'd discipline (pim-2 §3.6b) is satisfied
because the per-row CapRecheckFn closures distinguishably differ from
the structural pins above — the test would fail if either gate were
silently no-op'd in a future regression.

**Sync-grain interaction.** Compromise #11's row-level gate composes
with the Phase-3 G16-B `AttributionFrame` extensions
(`peer_did_set` + `device_did` + `sync_hop_depth`, see
`docs/INVARIANT-COVERAGE.md` "Inv-14 Phase-3 G16-B device-grain
extension"): the gate's per-row check is grain-orthogonal — it gates
on label-derived hints and actor cap-set, NOT on the merge frame's
device origin. Per-device read policy is a separate Phase-3 surface
tracked at `docs/future/phase-3-backlog.md` §6.12 item 3
(production-runtime threading of `device_cid` through engine
WriteContext construction sites). Both surfaces share the
`CapRecheckFn` scaffold but resolve at different layers.

**Phase-2b R6 Round-3 surfacing — `read_view_with` view-id-prefix
heuristic — CLOSED at Phase-3 G20-A3 wave-8a.** R6 Round 3's
`r6-r3-ivm-2` finding observed that `Engine::read_view_with`
(`crates/benten-engine/src/engine_views.rs::read_view_with`) derived
its `label_hint` for the `CapabilityPolicy::check_read` gate
exclusively by stripping the `content_listing_<label>` (or
`system:ivm:content_listing_<label>`) prefix from `view_id`. When
`label_hint` was non-empty the cap-policy `check_read` hook fired and
`DeniedRead` collapsed to an empty list. When `label_hint` was empty
— which was the case for ALL view ids that didn't match the
`content_listing_*` prefix, including the 4 canonical hardcoded-label
views (`capability_grants`, `version_current`, `event_dispatch`,
`governance_inheritance`) and ALL user-defined views — the
`check_read` hook was NOT invoked at the view-level read path. The
underlying `Subscriber::read_view` still runs and returns Node CIDs.
A user can register a custom view subscribing to any label (including
`system:` prefixes — `register_user_view` does not validate input
patterns against the system-zone reservation; the system-zone
reservation is a write-side reservation per Inv-7) and read those
Node CIDs via `engine.read_view`, all without `check_read` firing on
the view-level read path. **Bounded by:** Phase 2b ships only
`NoAuthBackend` (default; permits all reads) and `GrantBackedPolicy`
(which only gates capability-grant chain reads, where the per-row
denial behaviour is provided by the lower-level CID reads in
`primitive_host.rs::read_node`). A user-installed custom
`CapabilityPolicy` that intends to gate reads by label patterns
through view-id-derived hints will silently NOT fire on user-defined
view reads. Default-config users see no impact. The Inv-7 write-side
system-zone reservation is intact (users cannot WRITE `system:` Nodes)
— this finding was purely about the read path's check-firing
asymmetry. **Phase-3 G20-A3 wave-8a closure:** `read_view_with` now
extracts the `label_hint` via a registry helper
(`Engine::resolve_read_view_label_hint`) that consults
`benten_ivm::hardcoded_label_for_id` for canonical ids first, then
the engine's `user_view_input_labels` map (populated at
`register_user_view` time) for user-defined views, falling back to
the `content_listing_` prefix-strip only as a final resort for
pre-canonical-registry tests. End-to-end pin lives at
`crates/benten-engine/tests/view_id_label_hint_refactor.rs::view_id_to_label_hint_consults_input_pattern_label_not_string_prefix`
(per dispatch-conventions §3.6b — drives `Engine::read_view_with`
through a deny-reads-on-`post` cap policy and asserts the silent-deny
empty-list path fires). Anti-regression for canonical 5-view path at
`content_listing_views_still_route_through_registry_post_g20a3`.

---

### Compromise #12 — `DurabilityMode::Group` gate 5 — **CLOSED-AT-G13-E** (Phase-3 R5 wave-3)

**Class.** Durability / audit-freshness tradeoff under batch
commits.

**Status.** **CLOSED at G13-E** (Phase-3 R5 wave-3, 2026-05).
`DurabilityMode::default()` flipped from `Immediate` to `Group`
at the engine surface (see
[`crates/benten-graph/src/backend.rs::DurabilityMode`]); the
benchmark CI workflow `.github/workflows/bench.yml` was promoted
from informational to required + grew the APFS-relevant CRUD
fast-path timing benchmarks. The closure is pinned by three
tests:
- [`crates/benten-graph/tests/durability_default.rs::durability_mode_group_default_for_crud_fast_path`]
  (the default-flip itself);
- [`crates/benten-graph/tests/security_posture_compromise_12_marked_closed`]
  (this section's CLOSED marker);
- [`crates/benten-graph/tests/crud_fast_path_apfs_timing_within_target`]
  (informational wall-clock gate; bench is the authoritative perf signal).

**Shape (historical).** The Phase-2a `redb` backend committed under
`Durability::Immediate` — every write-bearing transaction fsynced
the redb journal before `Engine::call` returned. This was the
correct posture for the Phase-1 / 2a trust model (the ChangeEvent
and the persisted bytes both on disk before the caller observed
success), but it dominated the 150-300 µs §14.6 target on macOS
APFS (~4 ms fsync floor; see Phase-1 named compromise at the
bench-layer docs).

`DurabilityMode::Group` — grouped commit across a configurable
window — was considered for Phase 2a but deferred: the gate-5
interaction (how does the ChangeEvent fan-out interact with a
deferred fsync?) needed its own invariant pass. If a grouped
transaction's ChangeEvent reached a subscriber before the fsync
landed, and the process crashed, the subscriber observed an event
for a write that did not exist on disk at restart.

**Closure (Phase-3 G13-E).** Resolution chosen: **(c) leaving
Group as the engine-surface default while the redb backend
collapses Group → `Durability::Immediate` until redb grows
native batched-commit support.** The engine-level posture is the
right surface to declare; backend-specific mapping is a separate
concern. Three load-bearing claims that this closure makes:
1. Non-redb backends (in-RAM thin-client per
   [`crates/benten-graph/src/browser_backend.rs`] when G13-C lands;
   future peer-sync) can implement true grouped fsync without
   changing call sites — the default is already correct for them.
2. When redb itself grows the capability — see redb tracking
   issue history at the bench-layer docs — the on-disk behavior
   improves transparently with no semver break.
3. ChangeEvent gate-5 invariant: capability-grant writes still
   force `Durability::Immediate` at the redb mapping layer (see
   [`crates/benten-graph/tests/capability_grant_writes_immediate.rs`]),
   so the audit-freshness path that motivated the original
   deferral is preserved. The CRUD fast-path is the only surface
   that adopts the Group default.

**Mitigation (Phase-3 + later).** Capability-grant writes pin
`Durability::Immediate` per
[`crates/benten-graph/tests/capability_grant_writes_immediate.rs`];
operators wanting the historic per-commit-fsync posture
explicitly construct via
[`crates/benten-graph/src/redb_backend.rs::RedbBackend::open_or_create_with_durability`]
with `DurabilityMode::Immediate`. The
[`crates/benten-graph/src/redb_backend.rs::warn_if_group_durability_collapsed`]
one-shot warning still fires on benches so the redb-collapse
caveat is operator-visible.

**Residual risk.** Operators running on macOS dev hardware
against the redb backend still see the ~4 ms APFS fsync floor
per commit because redb v4 collapses Group → Immediate. This
is correct / on-posture for the redb backend specifically; it is
no longer a Compromise of the engine-level posture (the engine
surface declares Group as the default; backend-specific
mappings are a separate concern with their own visibility).
The §14.6 target remains aspirational for the redb backend
until upstream redb (or a Benten write-batching layer) lands;
non-redb backends are not constrained by it.

**Cross-refs.** plan §arch-r1-1; plan §3 G13-E; ENGINE-SPEC
§14.6 macOS caveat;
[`crates/benten-graph/benches/crud_post_create_dispatch_group_durability.rs`];
[`crates/benten-graph/benches/durability_modes.rs`];
[`docs/future/phase-2-backlog.md`] §9.1 (CLOSED-IN-PHASE-3-G13-E);
`.github/workflows/bench.yml` (promoted to required at G13-E).

---

### Compromise #13 — System-zone reserved-prefix rejection surface (G5-B-i Decision 6 / Minor 3)

**Class.** DX + defence-in-depth on the reserved `system:` prefix.

**Shape.** Phase-2a G5-B-i enforces Inv-11 in two registration-time
walkers:

1. A READ or WRITE operation Node whose `"label"` property is a
   `system:*` literal is rejected — the traditional Inv-11 surface.
2. An operation Node whose node-ID itself starts with `system:*`
   is ALSO rejected — the G5-B-i Decision 6 reserved-prefix DX
   improvement (Minor 3). A user who types a node-ID starting with
   `system:` gets a pointed error ("IDs cannot begin with the
   reserved `system:` prefix") rather than a downstream confusing
   failure when the node's resolved label happens to miss the
   system-zone.

Row 2 is defence-in-depth: the label-only probe already catches
the real violation, but the ID-prefix check surfaces the mistake at
the earliest possible point and removes a whole class of
"inadvertently used `system:` as a namespace" bugs.

**Mitigation / posture claim.** Both checks are live in
`crates/benten-eval/src/invariants/system_zone.rs::validate_registration`
and `crates/benten-engine/src/system_zones.rs::SYSTEM_ZONE_PREFIXES`.
The runtime probe (`resolved_cid_in_system_zone`) still covers the
TRANSFORM-computed-CID case at evaluation time.

**Residual risk.** A handler that builds a node-ID via TRANSFORM
concat (e.g. `"system:" + $input.key`) slips past the
registration-time reserved-prefix walker — the string is only
known at runtime. The runtime label probe catches the eventual
storage-layer impact (symmetric-None collapse at the user surface),
but the handler author may be confused about which invariant fired.
DX polish; no security gap.

**Cross-refs.** plan §G5-B-i Decision 6 / Minor 3; ERROR-CATALOG
`E_INV_SYSTEM_ZONE`; `SYSTEM_ZONE_PREFIXES` at
`crates/benten-engine/src/system_zones.rs`.

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10.4):
- *Question to ask:* Does the v1 DSL surface (TS DSL + napi + future-DX layers) hit the TRANSFORM-computed-CID system-zone-prefix slip-through often enough that the runtime-probe-only path produces confusing operator UX? If yes: does a runtime TRANSFORM-result reserved-prefix probe (raised symmetrically at WRITE-staging, not just registration) close the DX gap?
- *What changed pre-v1 vs Phase-2a framing:* Inv-13 row-4 SPLIT classifier (G16-B sync-receive arm) now enforces system-zone immutability at the merge boundary, so the security gap is closed; remaining work is purely DX-level error-message clarity for the TRANSFORM-concat case.
- *LOC estimate at v1-window:* ~50-100 (runtime-staging reserved-prefix probe + typed `E_INV_SYSTEM_ZONE_RUNTIME_PREFIX` variant + integration test).

---

### Compromise #14 — SANDBOX cold-start cost (no opt-in pool) — Phase-2b additive

**Class.** Performance / DX. Per-call SANDBOX dispatch always pays
the wasmtime `Store` + `Instance` construction cost.

**Shape.** D3-RESOLVED — Phase 2b ships SANDBOX with **per-call**
hosting only. Each SANDBOX primitive call constructs a fresh
`wasmtime::Store` + `wasmtime::Instance` (the `wasmtime::Engine` and
`wasmtime::Module` are singletons cached by content CID for the
process lifetime — Engine is expensive to construct, Module is
hash-cached, so cold-start cost is the per-Store + per-Instance work
only).

**Why per-call only:**

1. **Easier to add than to remove.** A pool-now / remove-later
   transition would be a breaking change; pool-later / no-pool-now
   is additive.
2. **Closes the "trusted boundary" sub-question entirely** — no need
   to define subgraph-annotation vs cap-grant vs engine-builder vs
   manifest-bound semantics for opt-in.
3. **Don't add features for hypothetical future requirements** — no
   data shows cold-start is a real workload problem at Phase 2b close.
4. **Misuse vector is real** — a developer slapping `pool: true`
   without understanding isolation implications is a silent security
   regression; pre-1.0 should not ship that footgun.

**Mitigation / posture claim.** If Phase-3+ workload telemetry
surfaces cold-start as a real bottleneck, an **opt-in pool** lands
as an additive Phase-3+ change without breaking existing handlers.
The G11-2b paper-prototype revalidation
(`docs/PAPER-PROTOTYPE-REVALIDATION.md`) at 16.7% SANDBOX rate gives
no signal that cold-start is a hot path for typical workloads.

**Cross-refs.** `docs/SANDBOX-LIMITS.md` per-call defaults; per-call
scope clarification (Engine + Module shared, Store + Instance
per-call).

---

### Compromise #15 — `register_runtime` reserved with deferred error — Phase-2b additive

**Class.** Surface area gap (intentional); deferred to Phase 8
marketplace.

**Shape.** D2-RESOLVED — Phase 2b's named-manifest registry is
**codegen-emitted** (a static `HashMap<String, CapBundle>` built at
`ManifestRegistry` construction from `host-functions.toml`). The
`register_runtime(name, bundle)` API is RESERVED in 2b — calls return
`E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED` typed-error.

**Why deferred:** dynamic manifest registration is the
marketplace-layer concern (Phase 8). Before the marketplace ships,
nobody is providing 3rd-party WASM modules with associated manifests;
shipping the dynamic registration API now would invite the
runtime-registered cap-bundle to drift from the codegen baseline
without tooling discipline (cap-bundle fingerprinting, lifecycle, GC).

**Mitigation / posture claim.** Phase-2b's manifest set
(`compute-basic`, `compute-with-kv`) covers the in-tree
host-fn surface; the typed deferral surface gives early-adopter
marketplace builders a concrete error to grep on. Phase 8 lifts the
deferral as part of the marketplace launch.

**Cross-refs.** D2-RESOLVED; `docs/HOST-FUNCTIONS.md` "Named
manifests" section; `host-functions.toml` `[manifest.*]` entries;
`E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED` in `docs/ERROR-CATALOG.md`.

---

### Compromise #16 — `random` host-fn deferred to Phase 3 — CLOSED at Phase-3 G17-A2 wave-5b

**Class.** Capability gap (intentional) — CLOSED at Phase-3 G17-A2
wave-5b (`benten-wt-r5-g17-a2`).

**Closure shape.** D-PHASE-3-11 RESOLVED-at-R1 picked **`getrandom`
direct** as the workspace CSPRNG (NOT `rand` ecosystem; NOT a
deterministic seed). G17-A2 wires the `random` host-fn alongside
`time` / `log` / `kv:read` in the codegen-default surface
(`crates/benten-eval/src/sandbox/host_fns.rs::default_host_fns`); the
trampoline at `register_default_host_fns` invokes `getrandom::getrandom`
to fill a guest buffer per call. Cap-string is `host:random:read`
(4-segment shape mirroring `kv:read`); `cap_recheck` is `per_call`.
Per-call entropy budget defaults to **4096 bytes** (per **r1-wsa-8**);
a module manifest may tighten or widen the override via the additive
optional `host_fns.random.budget_bytes_per_call` field on
`ModuleManifest`. Budget overrun fires the typed
`E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED` variant (routed through the
`ON_DENIED` family per the `cap_string_for_routing` rules in
`benten-errors`). The validate-time deferral guard at
`crates/benten-eval/src/primitives/sandbox.rs::execute` (the
sec-g7a-mr-5 `DEFERRED_HOST_FN_RANDOM_CAP_PREFIX` arm) is RETIRED;
`crates/benten-eval/tests/sandbox_host_fn_random_deferred.rs` is
deleted; the new green-phase regression guards are at
`crates/benten-eval/tests/random_host_fn.rs` (4 tests including the
load-bearing source-cite anti-regression
`sandbox_host_fn_random_no_longer_returns_deferred_error`).

**Shape (historical pre-G17-A2 narrative; preserved for audit).**
D1-RESOLVED — Phase 2b's host-fn set shipped `time`, `log`, and
`kv:read`. `random` was **deferred to Phase 3** because the workspace
CSPRNG framework choice had not been made (rand_chacha vs OS-CSPRNG
vs hardware-RDRAND fallback). Shipping `random` before that decision
would have baked in a footgun — a module that depended on weak
randomness then would be a silent security regression on a future
swap. Picking the wrong CSPRNG is a hard-to-reverse decision. A
SANDBOX module that attempted to call a `random` import received
`E_SANDBOX_HOST_FN_NOT_FOUND` with an operator-actionable hint citing
`phase-3-backlog.md §6.10`.

**Why CLOSED at Phase-3 G17-A2:** the workspace CSPRNG decision
landed at R1 (D-PHASE-3-11 RESOLVED). `getrandom` direct is
appropriate because (a) the rest of the workspace already pins it for
`ed25519-dalek` keypair generation, (b) it doesn't bake the engine
into the broader `rand` ecosystem trait shape, (c) it's
deterministic-seed-free by construction (the OS CSPRNG draws — the
class of footgun the deferral was protecting against). Wiring
`random` through a centralised cap-gated trampoline (NOT inline in
each module) preserves the Phase-2b posture that host-fn surface
additions go through the documented capability + audit-frame
discipline.

**Cross-refs.** D-PHASE-3-11 RESOLVED-at-R1; `host-functions.toml`
`[host_fn.random]` entry; `crates/benten-eval/tests/random_host_fn.rs`
green-phase regression guards;
`crates/benten-eval/src/sandbox/host_fns.rs::DEFAULT_RANDOM_BUDGET_BYTES_PER_CALL`
(4096 byte default per r1-wsa-8); `docs/HOST-FUNCTIONS.md` operator
section; `docs/MODULE-MANIFEST.md` per-manifest override field.

---

### Compromise #17 — In-memory module-bytes registry — CLOSED at Phase-3 G14-C wave-4b

**Class.** Persistence gap — CLOSED at Phase-3 G14-C wave-4b.

**Closure shape.** `Engine::register_module_bytes(&cid, &bytes)` now
recomputes `BLAKE3(bytes)` against the caller-supplied CID at the
entry point (D-PHASE-3-12 RESOLVED: typed
`E_MODULE_BYTES_CID_MISMATCH` rejection on mismatch), persists the
bytes via the durable
`crates/benten-graph/src/backends/blob_backend.rs::RedbBlobBackend`
(`system:ModuleBytes` zone Nodes), and mirrors them into the
in-memory hot-path cache. On `Engine::open`,
`Engine::rehydrate_module_bytes_from_zone` (called from
`EngineBuilder::assemble`) walks the zone and rebuilds the in-memory
cache so SANDBOX dispatch resolves without an operator re-call.

**Shape (historical pre-G14-C narrative; preserved for audit).**
`Engine::register_module_bytes(cid, bytes)` — the API
that registers compiled WebAssembly module bytes for SANDBOX
dispatch — used to store into a process-local `BTreeMap<Cid, Vec<u8>>`
guarded by a `Mutex` (the `module_bytes` field on `crates/benten-engine/src/engine.rs::Engine`). The
bytes were NOT persisted to the engine's redb backend; on `Engine::open`
the registry started empty regardless of what was registered against
the prior process.

**Workflow asymmetry with `Engine::install_module`.** This is the
load-bearing operator-facing surprise: `install_module(manifest,
expected_cid)` writes the manifest's canonical-DAG-CBOR bytes into the
`system:ModuleManifest` zone (a privileged Node write that survives
engine restart and is sync-eligible for Phase-3 federation). But the
underlying *wasm bytes* the manifest references (each `modules[i].cid`)
must be re-registered via `register_module_bytes(cid, bytes)` after
every engine open — there is no symmetric durable path for blob bytes
in Phase 2b. A SANDBOX dispatch whose module bytes haven't been
re-registered fires `E_SANDBOX_MODULE_NOT_INSTALLED` at the engine's
lookup step (the wave-8d-types typed variant), distinct from
`E_SANDBOX_MODULE_INVALID` which fires when bytes ARE present but fail
wasmtime structural validation.

**Why deferred to Phase 3 rather than fixed in 2b:**

1. **Blob storage is the load-bearing Phase-3 lift.** The `BlobBackend`
   trait — content-addressed mutable storage with iroh-fetchable shape —
   is on the Phase-3 critical path for P2P sync (Atriums fetch wasm
   modules from peer caches). Building an interim Phase-2b durable
   blob store would be wasted work since Phase-3 replaces it.
2. **Operator footprint is bounded.** Production deployments register
   modules at engine-open time (alongside `register_handler` /
   `install_module` calls); the in-memory shape mirrors how
   `register_subgraph` works (subgraph specs live in process memory and
   are re-registered on restart). Operators who need durable wasm
   storage in 2b can persist `Vec<u8>` against their own backing store
   and call `register_module_bytes` from their bootstrap path.
3. **No security-class hazard.** `register_module_bytes` does not
   consult capability policy (registering wasm bytes does NOT authorise
   any caller to invoke them — authority flows through the SANDBOX
   node's manifest cap-set + dispatching grant, both checked at execute
   time). The in-memory shape doesn't widen the trust boundary; it just
   widens the operator boilerplate at engine-open.

**Posture claim — non-validating API + lazy validation discipline.**
`register_module_bytes` does NOT verify the supplied CID matches
`blake3(bytes)` — content integrity is the caller's responsibility,
mirroring the pattern `Engine::install_module` uses for manifest CIDs.
Validation fires lazily at SANDBOX dispatch time when wasmtime parses
the bytes (`Module::new(&engine, &bytes)` in
`crates/benten-eval/src/sandbox/instance.rs::module_for_bytes`); a
malformed module surfaces as `E_SANDBOX_MODULE_INVALID`. Phase-3's
durable `BlobBackend` may add content-addressing-based validation at
register time (recompute BLAKE3 over the bytes; reject mismatch
upfront).

**Phase-3 G14-C closure** (LANDED). The Phase-3 promotion path the
Phase-2b plan described is the closure that landed: the in-memory
`BTreeMap` is mirrored against a `BlobBackend` impl
(`RedbBlobBackend`) that writes blobs as `system:ModuleBytes` zone
Nodes; `register_module_bytes` recomputes the CID + asserts
content-integrity + delegates to `BlobBackend::put`; `Engine::open`
rehydrates the in-memory active set via
`Engine::rehydrate_module_bytes_from_zone`. The trait surface is
defined at `crates/benten-graph/src/backends/blob_backend_trait.rs`;
the redb-native impl at `crates/benten-graph/src/backends/blob_backend.rs`;
the IndexedDB-browser impl landed in Phase 3 under the engine's
thin-client commitment (browser tabs hold snapshot cache only).

**Cross-refs.** `crates/benten-engine/src/engine.rs::register_module_bytes`
docstring; `crates/benten-engine/src/engine_modules.rs::install_module`
docstring; Compromise #4 closure narrative (execute-time validation
discipline); `docs/ERROR-CATALOG.md` `E_SANDBOX_MODULE_NOT_INSTALLED`
row; `docs/MODULE-MANIFEST.md` install lifecycle;
`crates/benten-engine/tests/module_bytes_cid.rs` (G14-C end-to-end
pin per §3.6b pim-2).

---

### Compromise #18 — In-memory handler-version chain — CLOSED at Phase-3 G14-C wave-4b

**Class.** Persistence gap — CLOSED at Phase-3 G14-C wave-4b
(sibling to Compromise #17; both closed in the same fix-pass).

**Closure shape.** Each `register_subgraph` /
`register_subgraph_replace` invocation persists a
`system:HandlerVersion` zone Node carrying `(handler_id, version_cid,
predecessor_cid?, seq)` per
`crates/benten-engine/src/handler_versions.rs`. The encoding is
additively extensible per arch-r1-4 / D-C — Phase-3 G16-B's
Loro-merge attribution variant slot lands without breaking existing
chain CIDs. On `Engine::open`,
`Engine::rehydrate_handler_version_chains_from_zone` (called from
`EngineBuilder::assemble`) walks the zone, groups by `handler_id`,
sorts by `seq`, and rebuilds the in-memory newest-first `Vec<Cid>`
chain. The full audit history survives engine restart.

**Shape (historical pre-G14-C narrative; preserved for audit).**
`Engine::register_subgraph_replace(spec)` — the wave-8f
hot-replace API — maintained an in-memory `BTreeMap<HandlerId, Vec<Cid>>`
of version-chain heads (newest-first). Each successful replace prepended
the new handler-CID onto the chain; `Engine::handler_version_chain(id)`
exposed the chain for devserver + audit consumers. The chain was
process-local and was NOT written to the redb backend; on `Engine::open`
the chain started empty regardless of how many replace calls happened
in the prior process.

**Why a separate compromise from #17 rather than a single bullet.** The
content is structurally identical (in-memory `BTreeMap` lost on engine
restart, Phase-3 promotion path the same), but the *audit class*
differs:

- Compromise #17 covers the **wasm payload** — bytes that wasmtime
  loads. Lost data: the wasm module's binary content. Operator recovery:
  re-call `register_module_bytes` from bootstrap.
- Compromise #18 covers the **handler hot-replace audit metadata** —
  the temporal sequence of "v1 → v2 → v3" handler swaps that devserver
  + operators rely on to answer "what was v3 of this handler?". Lost
  data: the historical CIDs. Operator recovery: there is no recovery —
  the historical CIDs are gone unless the operator captured them
  out-of-band (e.g. logs, devserver session state).

The user-visible loss surface differs enough that bundling under #17
would obscure the audit-trail-erasure aspect.

**Phase-3 G14-C closure** (LANDED). The promotion path the Phase-2b
plan described is the closure that landed:
`benten_core::version::Anchor` Node + Version-Node chain shape; each
`register_subgraph_replace` call writes a `system:HandlerVersion`
zone Node carrying the handler-CID + per-handler `seq` (insertion
order); `Engine::handler_version_chain_with_anchor` walks the
rebuilt chain and returns a `core::version::Anchor` rooted at the
oldest registered version. Phase-3 sync forwards the chain verbatim
across peers (the Version Nodes are content-addressed). G16-B
Loro-merge attribution lands as an additive variant slot per
arch-r1-4 / D-C without breaking existing chain CIDs.

**Posture claim.** The hot-replace contract itself is unchanged by the
in-memory shape: in-flight `Engine::call` invocations DO NOT see the
swap (handler_cid resolves once at call entry; the spec Mutex
re-lookup at `dispatch_call_inner` uses that CID as the third axis of
the subgraph-cache key). The in-memory shape only erases the
**historical** chain on restart; the **current** chain is durable for
the engine's lifetime, which is the contract devserver hot-reload
relies on.

**Cross-refs.**
`crates/benten-engine/src/engine.rs::register_subgraph_replace`
docstring (in-memory note);
`crates/benten-engine/src/engine.rs::handler_version_chain` docstring;
Compromise #17 (sibling persistence gap); Phase-2b R5 wave-8f mini-review
finding 8f-dx-10; [`docs/future/phase-3-backlog.md` §1.4 (Compromise #17
durable module-bytes registry) + §1.5 (Compromise #18 durable
handler-version chain)](../docs/future/phase-3-backlog.md) — both lift
to durable Anchor + Version-Node chain backed by the GraphBackend
umbrella trait (PHASE-3-BUNDLE-1).

---

### Compromise #19 — Browser-target persistent storage — PARTIALLY CLOSED at Phase-3 G18-A wave-5a

**Status:** **PARTIALLY CLOSED** at Phase 3; **FULL CLOSURE** deferred per `docs/future/phase-3-backlog.md` §4.3 (when the wasm32 `web-sys` / `js-sys` / `wasm-bindgen-futures` plumbing lands). **Partially closed via:** IndexedDB schema + handler scaffolding under the engine's thin-client commitment, plus schema-versioning groundwork.

**What landed at G18-A (scaffolding half).** Two new modules ship the persistence-layer architectural surface on `wasm32-unknown-unknown`:

- `bindings/napi/src/browser_indexeddb.rs` — IndexedDB schema-versioning layer (handler scaffolding). Declares schema-version constant `INDEXEDDB_SCHEMA_VERSION = 1`, the `module_manifest_store` + `blob_cache` object stores, the `on_upgrade_needed` migration handler (walks the v→v+1 chain — chain-computation half is wired; the wasm32 IDB-side dispatch is a stub), the `on_version_change` handler (stub on wasm32; the host build exercises the chain logic), and the `map_dom_exception_to_error_code` helper that maps `DOMException(name="QuotaExceededError")` to the typed [`ErrorCode::StorageQuotaExceeded`] variant (`E_STORAGE_QUOTA_EXCEEDED` per `docs/ERROR-CATALOG.md`).
- `bindings/napi/src/browser_blob_store.rs` — `IndexedDbBlobBackend` handle declaration mirroring the `BlobBackend` trait surface locked at G13-pre-B (`crates/benten-graph/src/backends/blob_backend_trait.rs`). Mirrors the redb-native `RedbBlobBackend`'s defense-in-depth CID validation per D-PHASE-3-12. The `IndexedDbBlobBackend::is_persistent()` returns `false` honestly at G18-A — the native arm uses an in-RAM `BTreeMap` mirror (native consumers must use `RedbBlobBackend`); the wasm32 arm has no IDB plumbing yet.

**What is DEFERRED to G18-A-followup (per `docs/future/phase-3-backlog.md` §4.3).**

- The wasm32 `web-sys` / `js-sys` / `wasm-bindgen-futures` deps that issue real `IDBDatabase.open` / `IDBObjectStore.put` / `IDBObjectStore.get` calls. The wasm32 arms of `apply_migration_step` + `close_database` are stubs today. Until those wire, `BrowserManifestStore::is_persistent()` and `IndexedDbBlobBackend::is_persistent()` BOTH return `false` honestly per the disclosure principle Compromise #19 originally articulated ("honest disclosure protects operators from assuming durability where none exists").
- The `BlobBackend` trait integration through the `Engine::open_with_browser_blob_backend(...)` constructor. The handle ships; the engine wire-up is the follow-up scope.

**Thin-client commitment.** The IndexedDB schema declares ONLY thin-client surfaces (`module_manifest_store` + `blob_cache`) — full-sync state (`loro_doc`, `iroh_peers`, `sync_cursor`, `atrium_full_state`) is explicitly absent and forbidden by the architectural pin at `bindings/napi/tests/indexeddb_schema.rs::indexeddb_persistence_thin_client_cache_only_per_baked_in_17`. Browser tabs participate in sync as authenticated thin-client views into a user's full peer; they do NOT carry sync state of their own.

**OPFS deferral per D-PHASE-3-27 / br-r1-11.** IndexedDB is primary at G18-A (broad browser support); OPFS / File System Access API is deferred to post-Phase-3. Future Phase-4+ may add an `OpfsBlobStore` sibling via the `BlobBackend` trait surface.

**Cross-refs.** `docs/MODULE-MANIFEST.md` §3.2; `docs/ERROR-CATALOG.md::E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE` + `E_STORAGE_QUOTA_EXCEEDED`; D-PHASE-3-27; br-r1-2 BLOCKER scaffolding; br-r1-8 MINOR honest-disclosure principle; `docs/future/phase-3-backlog.md` §4.3 (G18-A-followup wave named destination).

---

### Compromise #20 — Cross-browser determinism CI cadence — PARTIALLY CLOSED at Phase-3 G18-A wave-5a

**Status:** **PARTIALLY CLOSED** at Phase-3 G18-A wave-5a (this commit); **FULL CLOSURE** deferred to G18-A-followup wave (per `docs/future/phase-3-backlog.md` §4.3) when the Playwright fixture bodies are authored. **Partially closed via:** `.github/workflows/cross-browser-determinism.yml` Playwright matrix workflow + matrix cell structure per D-PHASE-3-7 + br-r1-4 + br-r1-10.

**What landed at G18-A (workflow + matrix cell structure).** A Playwright matrix workflow runs under Chromium, Gecko (Firefox), and WebKit (Safari engine) on per-PR cadence with the matrix-cell structure for the assertions documented below. Per HONEST DISCLOSURE: every matrix cell currently emits `::warning::...harness fixture not yet wired (G18-A-followup)` — the cells are STRUCTURAL anchors only at G18-A and do NOT execute the asserted determinism logic. A regression that broke canonical-bytes determinism in the wasm32 bundle would NOT be caught by this workflow at G18-A as currently structured. The Rust-side workflow-pin tests (`bindings/napi/tests/cross_browser_determinism_workflow_pins.rs`) verify the YAML contains the expected strings — the YAML strings themselves are no-ops at G18-A.

**Matrix cells the structure pins (full-closure-eligible at G18-A-followup).**

1. **Canonical-bytes determinism per the 7 distinct engine-determinism failure-surfaces** (br-r4-r1-5): Node envelope, handler-version-chain, AttributionFrame-with-device-DID, canonical-fixture corpus CID, BLAKE3 byte identity (SIMD/non-SIMD path), Ed25519 signature byte identity, and floating-point canonicalization under DSL eval (NaN bit-pattern + denormal + round-to-even per IEEE 754 edge cases).
2. **CID-pin equivalence across the three browsers** via an explicit reduce step (br-r1-4 WHAT FAILS framing) — a divergence indicates a CRDT/DAG-CBOR encoding non-determinism that would silently corrupt cross-browser sync.
3. **IndexedDB schema-migration round-trip + 1000-key no-data-loss sweep** (D-PHASE-3-27 / br-r1-2 LOAD-BEARING per pim-2 §3.6b): exercise the `on_upgrade_needed` handler under real Chromium / Gecko / WebKit IndexedDB.
4. **`QuotaExceededError → E_STORAGE_QUOTA_EXCEEDED` typed-error mapping** (D-PHASE-3-27 / br-r1-2): write oversized data + assert the error surfaces as `BentenError(code=E_STORAGE_QUOTA_EXCEEDED)`.

**What is DEFERRED to G18-A-followup (per `docs/future/phase-3-backlog.md` §4.3).** The Playwright fixture bodies that drive each matrix cell. Estimated ~200-400 LOC of test infrastructure — the cells go from `::warning::...harness fixture not yet wired` to real assertions that would FAIL on regression per pim-2 §3.6b end-to-end test pin requirement.

**Cadence + flake-budget retry policy per br-r1-10.** Per-PR cadence (NOT release-era — Phase-2b's release-era posture is RETIRED at G18-A). Retry policy: 1 retry on browser-launch failure (`PLAYWRIGHT_BROWSER_LAUNCH_RETRIES=1`); budget = 3 launches per 24h via workflow-concurrency cap; promotion-to-required-per-PR after 30 days informational green via `branch-protection.yml` update.

**Composition with #19.** Compromises #19 + #20 PARTIALLY close together at G18-A — the Playwright matrix is the CI cell that WILL prove the IndexedDB persistence is byte-deterministic across browsers once both halves' G18-A-followup work lands. The matrix workflow's Rust-side anchors live at `bindings/napi/tests/cross_browser_determinism_workflow_pins.rs` (12 source-cite assertions covering per-browser cells + CID-equivalence reduce + flake-budget retry + the 7 br-r4-r1-5 engine-determinism surfaces) — these pins assert the WORKFLOW STRUCTURE is in place; they do not assert the fixture bodies execute.

**Cross-refs.** Compromise #19 (the durability-half companion); `.github/workflows/cross-browser-determinism.yml`; `bindings/napi/tests/cross_browser_determinism_workflow_pins.rs`; D-PHASE-3-7; br-r1-4 / br-r1-10 / br-r4-r1-5; `docs/future/phase-3-backlog.md` §4.3 (G18-A-followup wave named destination).

---

### Compromise #21 — Module manifest signing — CLOSED at Phase-3 G14-C wave-4b (BLOCKER fix-pass)

**Status:** CLOSED at Phase-3 G14-C wave-4b BLOCKER fix-pass. Full
Ed25519 manifest signing landed via
`crates/benten-engine/src/manifest_signing.rs` (`sign_manifest` +
`verify_manifest_with_mode` + [`PublisherRegistry`]) AND wired through
the production `Engine::install_module(manifest, expected_cid,
verify_args)` entry point. UCAN-proof-chain primary +
publisher-key-registry fallback per D-PHASE-3-20 + crypto-minor-5.
Audience-binding rejection via
`benten_id::ucan::validate_chain_for_audience` per CLR-2 / cap-major-2.

**g14-c-mr-1 / mr-2 BLOCKER fix-pass (this commit):**
- `Engine::install_module` now takes a third `verify_args:
  ManifestVerifyArgs` argument; the production install path invokes
  `verify_manifest_with_mode` BEFORE persisting the manifest.
  Pre-fix-pass the helper existed but was never called from
  `install_module`, making the audience-binding closure narrative
  vacuous. End-to-end pin at
  `crates/benten-engine/tests/manifest_signing.rs::install_module_rejects_unsigned_when_verification_required`
  drives the production entry point and asserts unsigned + bad-sig
  manifests reject without persisting.
- `PublisherRegistry::new` now takes a third `registry_audience_did`
  argument (the engine's own audience DID, supplied at construction).
  `require_ucan_delegation` validates the chain against this
  pre-configured DID — no more `audience_from_chain(d) == d.claims.aud`
  tautology. Cross-atrium-replay regression at
  `crates/benten-engine/tests/manifest_signing.rs::publisher_registry_rejects_cross_atrium_replay`
  asserts a UCAN signed by admin but audience-bound to Atrium-A
  rejects when replayed at Atrium-B's registry.

**What ships at Phase 2b.** `Engine::install_module(manifest, expected_cid: Cid)` REQUIRES the `expected_cid` argument (D16-RESOLVED-FURTHER — not Optional, prevents the lazy `install_module(m, None)` footgun). The engine recomputes the canonical-bytes CID over the manifest, compares against `expected_cid`, and fires `E_MODULE_MANIFEST_CID_MISMATCH` (with a 1-line manifest summary so an operator can diff without source-code dive) on disagreement. This is the minimal CID-pin integrity gate.

**What's NOT shipped (pre-G14-C narrative; preserved for audit).**
Ed25519 manifest signing — i.e. the manifest carrying a signature
field that the engine verifies against a publisher public key before
installing — was deferred to Phase-3 at Phase-2b close. The
`signature` field WAS reserved in the canonical encoding
(omitted-when-`None` so future signed manifests don't break the wire
format) but was not consumed by the install path.

**What's NOW SHIPPED (G14-C wave-4b).** The
`crates/benten-engine/src/manifest_signing.rs::sign_manifest` helper
populates the `signature.ed25519` field using
`benten_id::keypair::Keypair`. Verification flows through
`verify_manifest_with_mode(manifest, ucan_chain, registry_pubkey,
engine_audience_did, mode, now)`:

- **`ManifestVerifyMode::All`** — BOTH UCAN AND registry paths must
  verify (security-critical posture).
- **`ManifestVerifyMode::Any`** (default) — EITHER path is sufficient
  (operator-flexibility posture for non-UCAN deployments).
- **UCAN check FIRST** when both paths are present (per
  crypto-minor-5).
- **Audience-binding rejection** via
  `validate_chain_for_audience` (CLR-2 / cap-major-2: cross-atrium
  replay defended).
- **Canonical-bytes excludes signature** (crypto-major-1):
  `manifest_signed_bytes` clears `signature → None` before
  re-encoding so the bytes the signature signs are stable across
  signed-vs-unsigned manifests.

Mutations to the durable [`PublisherRegistry`] require a UCAN
delegation rooted at the registry-admin DID (crypto-minor-5; defends
"anyone can publish").

**Threat model deltas.**
- *Ships at 2b:* tampering with manifest bytes between source and `install_module` call is detected (CID mismatch → typed rejection). This protects against in-transit corruption + simple substitution attacks where the operator has the expected CID out-of-band (e.g. from a published release manifest).
- *Deferred to Phase 3:* publisher authentication. A manifest with a forged-but-byte-consistent payload installs without complaint; the engine has no per-publisher trust anchor. Trust is established via the `expected_cid` arg the operator supplies; the manifest itself doesn't carry an unforgeable origin claim.

**Phase-3 G14-C closure** (LANDED). Ed25519 signing per D16:
`manifest.signature: Option<ManifestSignature>` populated via
`sign_manifest`; verification consults UCAN proof chain primary +
publisher key registry fallback (per D-PHASE-3-20 + crypto-minor-5)
through `verify_manifest_with_mode`. The canonical-bytes encoding
preserves the reserved-field (D9-RESOLVED) discipline; the
verification arm is additive — no wire-format break.

**Renumbering note.** This was `Compromise #N+5` in `docs/MODULE-MANIFEST.md`'s local table prior to R6 phase-close; lifted to global #21 here.

**Cross-refs.** `docs/MODULE-MANIFEST.md` §6 + §7; `docs/ERROR-CATALOG.md::E_MODULE_MANIFEST_CID_MISMATCH`.

---

### Compromise #22 — Peer-DID + connection metadata leakage to public iroh relays — Phase-3 additive

**Status:** Introduced at Phase 3 close (P2P sync via iroh transport landed). **Closure target:** Phase 7 Garden-relay infrastructure (Garden-protocol-controlled relays replacing public iroh relays for sensitive peer-discovery + connection metadata) — failing that, Phase 9 hardened-deployment posture.

**Class.** Network-layer metadata exposure; sibling to Compromise #11's IVM coarse-grained read-gate posture but at the transport layer rather than the eval-layer.

**What ships at Phase 3.** Atrium peer-to-peer sync uses iroh's QUIC + relay protocol for NAT traversal. iroh's default relay infrastructure is *public* (operated by n0 / community relays); peers connecting through these relays expose:

- *Peer DIDs* — `did:key` / future `did:plc` identifiers visible to the relay during connection establishment (the relay sees who is talking to whom, even though it cannot read the encrypted payload).
- *Connection metadata* — endpoint pairs, timing, peer-availability windows, which Atriums a peer participates in (inferred from connection patterns).
- *Membership topology* — which DIDs co-occur in connection sessions hints at Atrium membership without the relay decrypting any application-layer content.

End-to-end *content* confidentiality is preserved (iroh's QUIC payload is encrypted; the relay is a forwarder, not an endpoint). The leak is exclusively at the transport-metadata layer.

**What's NOT shipped.** Garden-protocol-controlled relays — relay infrastructure operated under the Atrium's own trust model (Phase 7 Gardens) where relay metadata stays within the Garden's social graph rather than going to a public third-party relay. Also not shipped: relay-bypass via direct hole-punched connections only (would require giving up the NAT-traversal fallback — operationally unviable for many home network topologies).

**Threat model deltas.**
- *Ships at Phase 3:* an adversary running or compromising a public iroh relay can build a social graph of who-connects-to-whom across Atriums using the engine's default sync transport. This is a metadata-correlation attack class, not a content-disclosure class. The CIDs being exchanged stay encrypted.
- *Deferred to Phase 7 / 9:* relay-trust posture. Until Garden-relays land, operators with stricter metadata threat models (whistle-blowers, journalists, threatened communities) MUST self-host iroh relay infrastructure for their Atriums or use the engine's full peer (laptop / phone-OS app) shape exclusively on trusted networks where NAT traversal is not needed.

**Phase 7 promotion path.** Wire Garden-protocol relays per the Phase-7 Gardens design: relay infrastructure becomes a first-class Garden resource (a Garden-controlled iroh relay node, accessible only to Atrium members of the Garden, with its operator-set being the Garden's quorum of admins rather than n0 / community). The Atrium-config surface gains a `relays: Vec<RelayDescriptor>` field where each `RelayDescriptor` is either `PublicIroh` (current default — the leaky path with a documented warning) or `GardenRelay { garden_id, relay_did }`. The Atrium join handshake (per the device-heterogeneity contract and the engine's thin-client posture) extends with a relay-trust negotiation step where peers agree on the relay set before falling back to public infrastructure.

**Phase 9 hardened-deployment fallback.** If Phase 7 Garden-relays slip, the Phase-9 hardened deployment posture takes the conservative path: *no* public iroh relays in production builds; full peers MUST be on networks reachable directly OR through self-hosted relays. The hardened-deployment cargo feature flag gates the public-iroh-relay code paths out entirely. This is the brutal but correct fallback if Garden-relays don't land.

**Posture claim.** Compromise IS introduced at Phase 3 close — the public-relay metadata leak goes from theoretical (no P2P sync at Phase 2b) to live (Atriums actually sync through iroh). Operators reading SECURITY-POSTURE.md see this honestly disclosed alongside the named closure target rather than discovering it via post-Phase-3 surveillance. Defends against the failure shape "compromise silently introduced at phase-close while metadata leakage is undocumented."

**Cross-refs.** `tests/phase_3_workspace/security_posture_compromises.rs::compromise_22_public_relay_metadata_leakage_introduced_at_phase_3_close_with_named_phase_7_garden_relay_destination` (RED-PHASE assertion); `tests/phase_3_workspace/security_posture_phase_3_close.rs::security_posture_phase_3_close_compromise_table_present` (phase-close compromise-table presence pin). Phase 7 Gardens own relay infrastructure as a Garden resource; the engine's deployment-shape commitment (full peer vs thin compute surface) is described in `docs/ARCHITECTURE.md`.

**Revisit at v1-window** (per `docs/future/phase-3-backlog.md` §10; Phase-7 Garden-relays primary closure path; Phase-9 hardened-deployment fallback):
- *Question to ask:* Does v1's deployment posture (full peer + thin compute surface) ship with public iroh relays as the *only* sync path, or does v1 require operators with adversarial-relay threat models to self-host? Does the v1 audit need a "default-on metadata-leak warning" UX surface (in DSL / napi / Engine builder) so adopters self-classify against the relay-metadata threat?
- *What changed pre-v1 vs Phase-3 framing:* Phase 7 Garden-relays + Phase 9 hardened-deployment cargo-feature flag are still future scope; the public-relay leak is the live reality at v1 unless operators self-host. The v1 readiness audit must surface this with operator-facing UX, not just SECURITY-POSTURE.md prose.
- *No-action option:* keep public-iroh-relays as the default at v1 with the current honest disclosure prose; document operator self-host as the conservative deployment recommendation; defer Garden-relay infrastructure to Phase 7. Phase-9 hardened-deployment cargo feature gating remains the brutal-but-correct fallback if Phase 7 slips.

---

### Compromise #23 — Wire device-attestation envelope cryptographic closure narrative — CLOSED at Phase-3 G16-D wave-6b fix-pass

**Status:** CLOSED at G16-D wave-6b fix-pass (cryptographic-attestation closure for criterion 16 per Ben ratification 2026-05-09). Tracked here to document the full narrative — the on-the-wire device-DID-attestation envelope landed at G16-D wave-6b initially as an *unsigned* shape (V1 carrying only `device_did: Option<String>`); the post-PR-#163 mini-review (cryptography lens findings g16d6b-crypto-1/2/3 + correctness lens g16d6b-corr-2) surfaced three substantive cryptographic gaps that the fix-pass closes end-to-end before Phase-3 close.

**Class.** Wire-format trust model. Sibling to Compromise #22 at the transport-metadata layer, but at the application-layer attestation boundary rather than the transport-relay metadata boundary. Compromise #11's eval-layer coarse-gate is the closest precedent in shape — both name the gap honestly + close it via existing hardened primitives rather than re-inventing parallel transports.

**Pre-fix-pass risk (V1 wire shape — wave-6b initial PR #163 HEAD `f46e9b6`).** Three substantive gaps:

1. **DID forgery** — the V1 envelope carried only `device_did: Option<String>` with no signature. A bad-faith peer could declare ANY `device_did` string verbatim; the receiver's `apply_atrium_merge` threaded the unverified string into `AttributionFrame.device_did` with full trust. Attack impact (bounded by the sec-r4r1-2 BLOCKER closure at PR #161 — the cap-recheck gates each row's actual write, so forgery cannot widen authority): forgery defeats Inv-14 device-grain provenance; surgical compromised-device-quarantine becomes advisory because the receiver cannot distinguish a legitimate device's writes from a forged-device-DID claim by the attacker.
2. **Replay** — V1 carried no nonce / no session-binding / no timestamp. A captured envelope from session A was bit-identical to a forge in session B against the same DID.
3. **Frame-pair non-binding** — V1 emitted the envelope and the Loro export as two independent `send_bytes` frames with nothing binding them. A MITM with frame-substitution could swap `device_did` while preserving the payload (or vice versa) without detection.

**Fix-pass closure shape (V2 wire envelope at wave-6b-fp HEAD).** All three gaps close end-to-end by **composing the existing hardened `benten_id::DeviceAttestation` + `Acceptor::accept_at` + `FreshnessPolicy` primitives at the wire boundary** rather than introducing parallel unsigned transport (codifies the pim-N-cand-crypto-attestation-transport-reuse candidate the cryptography lens flagged):

- The `DeviceAttestationEnvelope` wire shape promotes to V2 carrying `(version, attestation: Option<DeviceAttestation>, payload_hash: [u8; 32], session_nonce: [u8; 32], envelope_signature: Vec<u8>)`.
- **DID forgery defense:** the embedded `DeviceAttestation` is signed by the parent-DID's keypair (the user-identity issuing the device's capability envelope per D-PHASE-3-25). The envelope itself carries an additional `envelope_signature` produced by the originating device's keypair over the canonical bytes of `(version, attestation, payload_hash, session_nonce)`. The receiver verifies `envelope_signature` against the public key resolved from `attestation.device_did` (links the wire frame to the keypair the attestation names — a peer cannot impersonate another device's DID without holding that device's secret key) AND verifies the embedded attestation via `Acceptor::accept_at` (parent signature, freshness window, nonce-store, revocation list).
- **Replay defense:** each envelope carries a fresh 32-byte `session_nonce` from `getrandom` (independent of the attestation's parent-issued nonce). The signed `envelope_signature` covers `session_nonce`, so a captured envelope cannot be replayed verbatim against a different sync session. The receiver-side `Acceptor::accept_at` additionally rejects replay of the same parent-issued attestation nonce — defense-in-depth.
- **Frame-pair binding defense:** the envelope's signed `payload_hash` is `BLAKE3(loro_export_bytes)` for the Loro payload that follows. The receiver computes the BLAKE3 of the inbound payload and rejects via constant-time `subtle::ConstantTimeEq` if the hashes differ.
- All three failure modes reject with the single typed code `benten_errors::ErrorCode::DeviceAttestationForged` (`E_DEVICE_ATTESTATION_FORGED`, ON_DENIED routing) so audit pipelines route on the wire-attestation boundary uniformly per CLR-2 dual-layer recheck architecture.

**Threat model deltas.**
- *Pre-fix-pass:* an adversarial peer could pollute Inv-14 device-grain attribution by declaring forged device-DIDs at the wire boundary — defeating compromised-device-quarantine surgery. A captured legitimate envelope could be replayed across sessions; a MITM with frame-substitution could swap envelope/payload pairs.
- *Post-fix-pass:* all three classes close cryptographically. AttributionFrame.device_did is signed (parent → device DID binding) AND the wire envelope is bound to the specific Loro payload it precedes AND the parent-issued nonce + fresh session-nonce defend against verbatim replay. Inv-14 device-grain attribution is now load-bearing under adversarial-peer assumptions, not just cooperating-peer assumptions.

**Backward-compat preservation.** V1-shipped peers (and test fixtures that bypass the wire envelope by calling `apply_atrium_merge` directly) emit `attestation = None` envelopes; the receiver-side `verify` is a no-op for these and `apply_atrium_merge` falls back to the local engine's `device_cid` for the `AttributionFrame.device_did` slot. The two pre-existing pinned-CID fixtures (`sync_replica_attribution_carries_device_did_alongside_parent` + `sync_replica_explicit_actor_cid_decouples_from_device_cid`) still pass with the legacy `device-cid:<hex>` shape — no rebake required at this fix-pass.

**OPERATOR ACTION REQUIRED — `FreshnessPolicy` production override (operator-deployment residual).** The `Engine::set_acceptor` setter currently uses `FreshnessPolicy::seconds(u64::MAX)` as its test-grade default, which is permissive (no time-window pruning of the nonce store). Production deployments MUST override via `set_acceptor` with a concrete time-bound (e.g., `FreshnessPolicy::seconds(86_400)` for a 24h replay window) BEFORE participating in adversarial sync. Otherwise the nonce-store grows unbounded under sustained traffic AND the replay-resistance window becomes ungated. This is an operator-configuration concern, not a wire-format gap; the cryptographic primitives compose correctly under any `FreshnessPolicy` choice, but the choice itself is operationally load-bearing for production deployments. Documented in `Engine::set_acceptor` rustdoc + this Compromise #23 narrative; verify via per-deployment audit at v1-window assessment.

**Posture claim.** Wave-6b's initial wire-shape introduction was scoped to "structural transport landing"; the fix-pass closes the trust-model layer in the SAME phase. Operators reading this section see the full narrative: the wave first landed a benignly-functional transport, then closed the cryptographic gaps in the same window. Defends against the failure shape "compromise silently introduced at phase-close while wire-attestation forgery is undocumented."

**Cross-refs.** `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope::verify` (envelope signature + Acceptor + payload-hash composition); `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope::new_signed` (signing path); `crates/benten-id/src/device_attestation.rs::Acceptor::accept_at` (composed primitive); `tests/integration/atrium_two_device.rs::forged_device_did_rejected_at_envelope_verify` (DID forgery rejection pin); `tests/integration/atrium_two_device.rs::replayed_envelope_rejected_by_acceptor_nonce_store` (replay defense pin); `tests/integration/atrium_two_device.rs::frame_pair_payload_swap_rejected_by_payload_hash_binding` (frame-pair binding pin); `tests/integration/atrium_two_device.rs::future_wire_version_rejected_at_decode` (version validation pin); `docs/ERROR-CATALOG.md::E_DEVICE_ATTESTATION_FORGED` (typed-code surface); `docs/INVARIANT-COVERAGE.md::Inv-14` (device-grain attribution narrative honesty retense). Origin: post-PR-#163 mini-review (cryptography lens findings g16d6b-crypto-1/2/3/4 + correctness lens g16d6b-corr-2/3 + dist-systems lens MINOR-2 honest-disclosure) cross-corroborated; closed inline at the same wave per Ben ratification 2026-05-09 (NOT v1-window-deferred).

---

### Compromise #24 — Wallclock fail-closed posture (no default-clock-zero expiration bypass) — CLOSED at Phase-3 G16-B-B-rest

**Status:** CLOSED at Phase-3 G16-B-B-rest (PR #158, 2026-05-09). Earlier shape (pre-G16-B-B-rest) used `DEFAULT_NOW_SECS = 0` as an implicit fallback at `UcanGroundedPolicy`; any code path that constructed a `UcanGroundedPolicy`-evaluating chain WITHOUT injecting a wall-clock would silently evaluate the chain at epoch second zero — which falsely admitted expired UCANs (their `expires_at` invariably > 0; clock 0 vs expiry N always passes the "not yet expired" check). The G16-B-B-rest closure inverts the fall-back: any chain with `nbf > 0` OR `exp > 0` against `now_secs == 0` aborts with the typed `CapError::UcanClockNotInjected` (`E_UCAN_CLOCK_NOT_INJECTED`) rather than silently passing.

**Class.** Fail-open clock regression at any cap-evaluating surface. Sibling to Compromise #1 (TOCTOU bounded windows) at the cap-policy layer rather than the evaluator layer.

**Closure shape.**

- `crates/benten-caps/src/ucan_grounded.rs` — `DEFAULT_NOW_SECS = 0` remains as a sentinel constant, but the policy now refuses chains-with-time-bounds when `now_secs == 0` (the inversion). The `chain_has_time_bounds` helper drives the check.
- `crates/benten-engine/src/builder.rs` — engine builder threads explicit clock-inject through the `crates/benten-engine/src/builder.rs::EngineBuilder` `ucan_grounded_now_secs` field; `Engine::open` refuses to initialize the UCAN backend without a clock when a `UcanGroundedPolicy` is configured (rustdoc on the field documents the inversion).
- Typed error `CapError::UcanClockNotInjected` → `ErrorCode::E_UCAN_CLOCK_NOT_INJECTED` (catalog entry).

**Threat model closed.**

- *Pre-closure:* a developer wires `UcanGroundedPolicy` into an engine without injecting a wallclock; engine silently uses clock=0; ALL UCAN proofs with positive expiration timestamps pass as "not yet expired" regardless of when they were minted. Effective bypass of the entire UCAN expiration model. Failure mode is INVISIBLE in normal tests — every expired proof admits without warning.
- *Post-closure:* the same misconfiguration surfaces typed `E_UCAN_CLOCK_NOT_INJECTED` at the first chain evaluation. Developer cannot ship a UCAN-using engine without confronting clock injection. Production code MUST inject a real wallclock; test code injects via `with_now_for_test`.

**Test pin.** `crates/benten-caps/src/ucan_grounded.rs::default_now_secs_zero_fails_closed_when_chain_has_time_bounds` (inline test asserts the fail-closed branch fires when `DEFAULT_NOW_SECS=0` AND the chain has time bounds) + companion `default_now_secs_zero_walks_chain_when_no_time_bounds` (asserts the unbounded-chain branch remains permissive so the sentinel doesn't false-positive on time-unbounded grants).

**Production discipline (couples to Phase 4-Foundation).** Every new cap-evaluating surface in Phase 4-Foundation (admin UI install path; materializer pipeline; plugin manifest verify; schema compiler walk-time gating) MUST thread injected clock. Source-side discipline: no `SystemTime::now()` / `Instant::now()` in the four new crate surfaces; CI grep audit catches regressions per `.addl/dispatch-conventions.md` §3.5g cross-language-rule-mirror application. The transparent-clock-injection-at-manifest-load-surface ratification (per Phase 4-Foundation D-4F-15, Ben Q6 2026-05-11) inherits this discipline at engine-side rather than requiring plugin authors to thread clock themselves.

**Cross-refs.** `crates/benten-caps/src/ucan_grounded.rs::UcanGroundedPolicy` (the fail-closed inversion); `crates/benten-caps/src/ucan_grounded.rs::DEFAULT_NOW_SECS` (the sentinel constant); `crates/benten-caps/src/ucan_grounded.rs::with_now_for_test` (the injection-at-builder surface; production injection threads through the same `now_secs` field via the policy builder); `crates/benten-caps/src/ucan_grounded.rs::default_now_secs_zero_fails_closed_when_chain_has_time_bounds` (load-bearing test pin asserting the fail-closed branch fires when `DEFAULT_NOW_SECS=0` AND the chain has time bounds); `docs/ERROR-CATALOG.md::E_UCAN_CLOCK_NOT_INJECTED` (typed-code surface); `docs/future/phase-3-backlog.md §2.3 (i)` (the v1-assessment-window deliverable that retires the sentinel by threading `WriteContext::now` through every cap-evaluating call site — current state is operator-discipline via injection at builder; future state is per-call wallclock binding).

---

### Compromise #25 — HLC-monotonic enforcement at sync layer (adversarial-peer wallclock-injection defense) — CLOSED at Phase-3 sync-attack test family

**Status:** CLOSED at Phase-3 sync attack-test family. The defense composes three Phase-3-shipped primitives at the sync boundary: HLC-monotonic enforcement (peer cannot publish HLC values that go backward beyond their own previous publication); nonce-cache (per-session nonce store rejects replay of previously-seen sync envelopes); HLC bound inside the signed device-attestation envelope V2 (per Compromise #23 — the envelope's signature covers the HLC values, so adversarial wallclock-injection at the envelope layer is detected at signature-verify time before reaching the application-layer Loro merge).

**Class.** Adversarial-peer-controlled wallclock injection at the sync transport layer. Distinct from Compromise #24 (engine-internal clock injection discipline) — this compromise addresses a peer-controlled threat surface rather than an in-process developer-configuration surface. Sibling to Compromise #23 at the HLC-payload boundary rather than the device-attestation-envelope boundary.

**Closure shape (three composed defenses).**

1. **HLC-monotonic enforcement** at `crates/benten-sync/src/handshake.rs` + `apply_atrium_merge` path. Inbound sync frames carry HLC values; the receiver's HLC oracle tracks per-peer max-seen HLC; frames whose HLC is below a peer's previous max are rejected with typed `E_HLC_SKEW_EXCEEDED`. Test pin at `crates/benten-sync/tests/attack_hlc_skew_revocation_ordering.rs` (the `hlc_skew_exceeded_in_inbound_sync_frame_rejected_with_e_hlc_skew_exceeded` test exercises an adversarial peer attempting to publish revocation-ordering past a previously-seen HLC bound; receiver rejects).
2. **Nonce-cache for replay defense** at the device-attestation `Acceptor::accept_at` path (per Compromise #23 closure). Each envelope carries a 32-byte session-nonce; the receiver's nonce-store rejects replay of any nonce already seen within the `FreshnessPolicy` window. Defends against captured-envelope-replay-with-stale-HLC.
3. **HLC bound inside signed envelope** — the device-attestation envelope V2's signed bytes include HLC fields. An adversarial peer cannot mutate HLC without invalidating the signature. Combined with defense 1, this means an adversarial peer can publish at-most their own honest HLC values (forging HLC requires forging the envelope signature, which requires holding the peer's secret key).
4. **G16-B-F structural-always-on per-row cap-recheck (PR #161)** — even if an adversarial peer manages to push a frame that passes defenses 1-3 (e.g., a peer with a legitimately-issued cap that has since been revoked re-shares an old frame), the per-row cap-recheck at `apply_atrium_merge` denies the merge against the current revocation state. Defense in depth.

**Threat model closed.**

- *Pre-closure (theoretical):* an adversarial peer pumps HLC values to suppress concurrent honest writes (HLC LWW resolution favors higher HLC); replays previously-captured envelopes to retroactively re-introduce already-revoked authority; injects fabricated HLC to forge causality.
- *Post-closure:* all three vectors close. HLC monotonicity bounds adversarial publishing to the peer's own honest progression. Nonce-cache rejects exact-bytes replay. Signed envelope binds HLC to peer identity (cannot forge HLC without forging envelope signature → requires secret-key holding). Per-row cap-recheck denies merges against revoked authority even if a frame passes envelope verification.

**Test pins (Phase-3 sync-attack family).** `crates/benten-sync/tests/attack_hlc_skew_revocation_ordering.rs` (HLC-skew + revocation-ordering); `crates/benten-sync/tests/attack_loro_op_log_inv_13.rs` (Loro op-log integrity under attack); `crates/benten-sync/tests/attack_mst_diff_cid_mismatch.rs` (MST CID-mismatch attack class); also exercised end-to-end at `crates/benten-engine/tests/integration/atrium_two_device.rs` (the device-attestation envelope V2 narrative tests cover the HLC-bound-inside-signature shape).

**Posture claim.** Adversarial-peer wallclock-injection IS a real threat class — the engine's sync layer cannot trust peers to publish honest HLC values, just as it cannot trust them to declare honest device-DIDs (Compromise #23). The defense composes Phase-3-shipped primitives at sync receive time + at the cap-recheck boundary; no net-new mechanism is needed at Phase-4-Foundation. Future plugin-share boundary (Phase-4-Foundation G24-D) inherits these defenses transparently — plugin-share is just Atrium-share with a manifest envelope on top.

**Cross-refs.** `crates/benten-sync/tests/attack_hlc_skew_revocation_ordering.rs` (HLC skew + monotonicity + revocation ordering test pins); `crates/benten-sync/src/handshake.rs` + `crates/benten-sync/src/handshake_wire.rs` (HLC + nonce defenses in the handshake state machine); `crates/benten-errors/src/lib.rs::ErrorCode` (the `HlcSkewExceeded` variant of the typed `ErrorCode` enum; stable-code string `E_HLC_SKEW_EXCEEDED`); `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope::verify` (envelope signature covering HLC fields; Compromise #23 cross-reference); `crates/benten-engine/src/apply_atrium_merge` (G16-B-F structural-always-on per-row cap-recheck PR #161; defense-in-depth). Plugin-share boundary in Phase 4-Foundation (G24-D plan §3) inherits via `plugin_share` calling through the same sync infrastructure.

---

## Content-hash verify-on-read at every Node-bytes surface (W9-T6 Phase-3 R5 wave-9)

**Defense added (W9-T6, ratified 2026-05-08).** `RedbBackend::get_node` now verifies the content-hash of stored bytes against the requested CID before returning the decoded Node. The redb file is treated as a system boundary; CID semantics ("self-validating identifier") are honored on every read.

**Threat model closed.** Local-disk tamper (an attacker with filesystem access to the redb file) and hardware bit-flip (cosmic-ray / disk-controller corruption) on Node-rehydration paths — handler_versions chain rehydration on `Engine::open`, engine_modules manifest+wasm-bytes registry rehydration, IVM materialise paths that rehydrate Node bodies. Pre-W9-T6, `RedbBackend::get_node` decoded the stored bytes and returned the wrong-but-decodable Node; an attacker who could swap bytes at rest could substitute one Node for another at a given CID slot, and the engine would happily execute the substituted Node as if it were the legitimate one.

**Closure shape.** `RedbBackend::get_node` routes through `benten_core::Node::load_verified(cid, &bytes)` — the same hash-then-decode helper that subgraph-load uses. Three-outcome contract pinned at the type level:

- `Ok(None)` — clean miss; CID never written.
- `Err(GraphError::Core(CoreError::ContentHashMismatch))` — bytes present but corrupted/tampered (`E_INV_CONTENT_HASH`).
- `Err(GraphError::Core(CoreError::Serialize))` — bytes hash-match but fail to decode (genuine codec drift, `E_SERIALIZE`).
- `Ok(Some(node))` — clean roundtrip; bytes hash-match and decode.

End-to-end pin lives at `crates/benten-graph/tests/get_node_verifies_content_hash_on_read.rs` (5 tests, including a "would-FAIL on silent no-op" pin per dispatch-conventions §3.6b that uses the test-only `corrupt_node_bytes_for_test` hook to mutate on-disk bytes after `put_node` and assert the next `get_node` fires `E_INV_CONTENT_HASH`).

**Out of scope (already defended elsewhere).** Cross-peer Node ingestion is defended by `Mst::apply_entries` per-entry rehash (sec-r4r2-1) — every entry's `payload` is BLAKE3-rehashed and compared byte-for-byte against the declared `cid` before insertion. Subgraph-load is defended by `Subgraph::load_verified_with_cid` (`RedbBackend::load_subgraph_verified` graph-layer wrapper). W9-T6 closes the remaining `Node`-read on-disk surface.

**Performance.** BLAKE3 over canonical DAG-CBOR Node bytes adds ~3-10 µs per `get_node` call on Apple Silicon (the budget accepted at the §6.2 ratification). Hot-loop callers (IVM materialise, repeated rehydration) absorb the cost; if future perf measurement shows the cost is load-bearing, an internal escape hatch (e.g. `get_node_unverified` for trusted callers that have already verified upstream) can be added with documented justification — but no `get_node_verified` opt-in shipped today (verify-on-read is unconditional).

**Cross-refs.** `crates/benten-graph/src/redb_backend.rs::get_node` (the verify-on-read site); `crates/benten-graph/src/lib.rs::corrupt_node_bytes_for_test` (test-only tamper hook); `crates/benten-graph/tests/get_node_verifies_content_hash_on_read.rs` (end-to-end pin); `docs/ERROR-CATALOG.md::E_INV_CONTENT_HASH` (Thrown-at line enumerates all three firing surfaces); `docs/future/phase-2-backlog.md` §6.2 (closure narrative); `crates/benten-sync/src/mst.rs::Mst::apply_entries` (sec-r4r2-1 cross-peer ingest precedent that this PR mirrors at the on-disk boundary).

---

## Repository security configuration (Phase 2a §3.1 hardening pass)

**CodeQL code scanning.** Workflow at `.github/workflows/codeql.yml` runs
on every push to `main`, every PR, and weekly cron. Findings appear in
the GitHub Security tab; not a required CI check (informational-only per
ci-decisions-2026-04-22.md §4). The workflow file *is* the configuration —
GitHub auto-enabled code scanning on first SARIF upload from the
`analyze` action; no Settings toggle was needed.

**Private Vulnerability Reporting (PVR).** Enabled at the repo level on
2026-04-25 (Settings → Code security and analysis → Private vulnerability
reporting). Gives external researchers an in-platform path to report
security issues privately rather than opening a public issue. No
maintainer action needed beyond the toggle; reports route to the
Security tab. See <https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/configuring-private-vulnerability-reporting-for-a-repository>.

**Branch protection on `main`.** Spec at `.github/branch-protection.yml`;
drift-check workflow at `.github/workflows/branch-protection-spec-check.yml`.
Apply runbook + PAT setup live in the spec file's header comment. Closes
the CI-1 deferral.

**Third-party action SHA-pinning.** All workflows pin third-party actions
at commit SHAs (rather than mutable tag/branch refs). Dependabot rotates
weekly via `.github/dependabot.yml`'s github-actions ecosystem entry.
Closes the CI-3 deferral.

---

## Plugin trust model (Phase-3-close pre-v1 commitment)

**Ratified 2026-05-10.** Pre-v1-cleanup architectural commitment fixing the trust
model for app-level plugins ahead of Phase-4 plugin-manifest + admin-UI work.
Two distinct extensibility categories with deliberately separate trust shapes:

### App-level plugins — three-layer consent

App-level plugins are **subgraphs** of operation Nodes (handlers, materializers,
SANDBOX nodes), content-addressed and shared peer-to-peer through Atriums. Each
plugin has its own DID + an attenuated UCAN delegated by the user at install
time. The engine evaluator walks plugin subgraphs the same way it walks any
handler, with the active principal switched to the plugin's identity for the
walk's duration.

The trust model is layered:

1. **User-as-root.** Every capability chain traces back to a user-issued root
   grant. Phase-8 P2P plugin discovery does not weaken this — a signed,
   content-addressed plugin manifest is still rooted in user consent at install.
   No object-capability-style "possession of a token IS the right" semantics
   above the user's mint operation.
2. **Install-time manifest envelope.** The manifest carries `requires` (caps the
   plugin needs) and `shares` (policy for what other plugins are allowed to
   receive from this one). Both are signed by the plugin author so they cannot
   drift post-install. The user reviews the manifest and consents to the
   *envelope*, not to each runtime access. This is the v1 install UX surface.
3. **Runtime UCAN delegation within manifest envelope.** Plugin A may delegate a
   UCAN to plugin B if and only if B's request fits A's manifest `shares`
   policy. The CapabilityPolicy backend validates the chain at access-time:
   chain traces to user-root + each delegation step fits source plugin's policy
   + requested cap is within attenuation envelope. Plugin-to-plugin delegation
   inside the envelope does not require additional user prompts.

**Engine-side surface.** The evaluator's read pathway threads the active
principal through `Engine::read_node_as(principal, cid)` — the public surface
for any read attributed to a non-trusted principal. Engine internals (IVM,
sync, view materialization, audit, change-event fanout) reach the unchecked
storage read via `self.backend.get_node(cid)` directly — the backend field
+ accessor are both `pub(crate)`, so external crates physically cannot bypass
the policy gate. Plugin authors do not call either path directly: they
author graph nodes; the evaluator is the only caller of `_as`. Mirrors the
established `Engine::call_as` precedent at
`crates/benten-engine/src/engine.rs::call_as`. **Implementation landed in
the pre-v1 cleanup window** (closing Phase-2a-era debt: the 4 `todo!()`
stubs at `crates/benten-engine/src/engine_wait.rs:1011-1311` — `put_node`
+ `read_node_with_policy` (renamed to `read_node_as`) + the test-only
read-grant helper + the dead bench-helper sibling — closed under
`docs/future/phase-3-backlog.md §13.7`). The engine surface is independent
of the Phase-4 plugin manifest schema; both can be designed and shipped
without sequencing dependencies.

**Private namespaces.** A plugin's writes go to a DID-scoped namespace whose
cap is held by the plugin's DID. Manifest `shares=none` for that namespace
blocks delegation; the engine refuses to issue cross-plugin caps for it.
Provides a sovereign space for plugin internals (AI agents' working memory,
intermediate state, scratchpads) without breaking the cross-plugin sharing
model — same UCAN machinery, different policy.

**Threat model.**

- **Scope creep at install** — defended by signed manifest. The user reviews
  the manifest at install; later changes require the user re-consent on
  upgrade. Author cannot retroactively widen the envelope.
- **Plugin-to-plugin smuggling** — defended by manifest `shares` policy
  validation at delegation time. Plugin A cannot mint a cap to plugin B that
  exceeds A's manifest envelope; the policy backend rejects the chain.
- **Object-capability bypass** (a malicious plugin tries to construct a cap
  out of thin air) — defended by chain-traces-to-user-root validation. Any
  cap presented at access time must trace back to a user-mint root. There is
  no engine sentinel principal callable from outside the engine crate.
- **Engine-internal-as-principal forgery** (a plugin tries to call the
  evaluator's unchecked read path) — defended by `pub(crate)` visibility on
  `Engine::read_node`. The Rust compiler refuses cross-crate calls; plugin
  subgraph nodes cannot directly invoke the function regardless.

**Trajectory alignment.** v1 (Phase 4 — Phase 4-Foundation ships the
manifest schema + admin UI v0 + install-time consent; Phase 4-Meta layers
self-composing admin on top) — small N, user reviews each manifest, simple.
Phase 6 (AI agents) — an assistant declares "I integrate with calendar
/ notes / email" in its manifest; user consents at install; the agent runs
autonomously without per-action prompts. Phase 8 (decentralized plugin
discovery) — plugins are signed by author, content-addressed, discovered
through Atrium peer groups; users trust the signed manifest, not a central
registry.

**Phase-4-Foundation R1-triage refinements (2026-05-11 night).** The base
three-layer model survives unchanged; the implementation specifics for the
plugin-identity model are refined:

- **Four distinct identity concepts** (D-4F-12 retense per
  `.addl/phase-4-foundation/r1-triage.md` Q4): Content-CID (what the plugin
  IS) + peer-DID signature on original content (provenance; `benten-id`
  RotationLog handles peer-DID rotation) + plugin-DID minted at install
  (UCAN audience AND constrained issuer within manifest envelope; per
  D-4F-16 `did:key:...` shape with engine-held Ed25519 keypair via OsRng,
  per-install fresh) + user-DID (trust anchor + signs install records).
  Cross-plugin/schema references use **content-CID, not author-DID**
  (`accepts_content: [hash, ...]`).
- **NO Benten-project-key infrastructure.** User-DID signs install records
  (the user is the source of install consent). Peer-DID signs original
  content (provenance). No central project key infrastructure for plugin
  signing.
- **Manifest schema versioning DROPPED** (D-4F-13). CID covers shape;
  pull-not-push obviates a schema-version field; T10-upgrade defense list
  no longer includes "manifest-schema-version-downgrade."
- **Plugin manifest v0 — Phase 4-Foundation implementation state.**
  Manifest envelope verified at every load (T5a defense-in-depth verify
  points: boot + per-load + per-Atrium-merge per sec-4f-r1-9). Cap-change-
  triggered fresh consent for all upgrades (silent within-lineage subset;
  full re-consent if `requires` GREW; cross-fork = user-initiated merge).
  Meta-plugin composition cycle detection AS REJECTION at install time
  (new ErrorCode `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED`); handler-call-
  graph cycle detection at handler-registration time stays Phase 4-Meta
  per `docs/future/phase-3-backlog.md §15.2`.
- **Compromise #11 (IVM views coarse-grained read-gate) closure floor
  REAFFIRMED** against the new materializer surface. Materializer SHARES
  `IvmViewReadGate` machinery (D-4F-NEW-MATERIALIZER-READ-GATE resolved
  option SHARE — materializer view IS an IVM view per D-4F-2). The
  Compromise #11 closure does not regress with the new surface.
- **Compromise #24 (wallclock fail-closed) + Compromise #25 (HLC-monotonic
  sync) REAFFIRMED** against new manifest-load surface (clock injection is
  transparent at engine-side per D-4F-15; HLC-monotonic-strict acceptance
  for peer-DID rotation per sec-4f-r1-10 T9b race-defense).
- **MVP rotation mechanism — Phase 4-Foundation** (ratification #6):
  `SelfRevocation` attestation + out-of-band new-key trust. Old-key signs
  timestamped revocation; propagates via Atrium sync; peers reject content
  signed by revoked key after revocation timestamp. **Kith** (working name;
  Phase 5+ exploratory) is the richer decentralized-identity-and-attestation
  substrate that would supersede the MVP; scaffold at
  `docs/future/kith-decentralized-identity.md`.
- **Decentralized self-discovered registry → Phase 4-Meta** (ratification #3).
  Phase 4-Foundation v0 uses direct content-addressed-share over Atriums
  (out-of-band handshake; user pulls from peer they trust). T10-discover
  threat surface FULLY N/A for v0; carries to `docs/future/phase-4-backlog.md
  §3.1`.

Full plan + implementation seams at `docs/PLUGIN-MANIFEST.md` (Phase-4-
Foundation companion doc).

**What this rules out.**

- *Pure user-as-root with per-action prompts.* Notification fatigue;
  combinatorial explosion as N plugins grow; AI-agent ergonomics fail.
- *Pure UCAN-native peer delegation without the envelope.* "I installed plugin
  A; I did not agree to A handing my data to plugin B." User loses meaningful
  control after install.
- *Plugin runtime separate from the engine evaluator.* No JS loader / FFI
  bridge / embedded interpreter. Plugins are graph; the evaluator is the only
  runtime.

### Engine-level extensions — compile-time trust

Engine extensions are **Rust crates** linked into the engine binary at compile
time. For custom IVM strategies, alternate transports (post-iroh — shaped
relays, Tor, Nostr), alternate persistence backends (post-redb — sled, fjall,
cloud-KV), custom signature schemes (post-Ed25519 — X25519, BLS, post-quantum),
performance-critical primitives that need raw Rust speed beyond SANDBOX.

**Trust posture.** "You compiled this into your engine binary." Same trust as
Benten core. There is no UCAN, no manifest envelope, no `read_node_as`
boundary. An engine extension that wants to violate invariants can — the
boundary is `cargo` and code review, not the type system.

**Audience.** People building the platform itself, not app users. The two
extensibility categories are intentionally separate worlds; trust models do
not transfer between them in either direction. Future proposals to extend the
app-level plugin trust model to engine-level extensions (or vice versa) must
be rejected with reference to the architectural commitment captured here.

**Cross-refs.** `docs/ARCHITECTURE.md` "Plugins and engine extensions" (the
architectural surface); `docs/HOW-IT-WORKS.md` "Plugins, in plain English"
(the orientation tour); `docs/GLOSSARY.md` ("App-level plugin," "Engine
extension," "Manifest envelope," "Plugin DID," "Plugin manifest");
`crates/benten-engine/src/engine_wait.rs:1011-1026` (the four `todo!()` stubs
that are β-shaped — the migration target for the read-side gating
implementation, which lands as pre-v1 cleanup independently of the Phase-4
plugin-manifest schema work); `crates/benten-engine/src/engine.rs::call_as`
(the precedent the read-side mirror follows).
