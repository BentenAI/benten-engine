# Phase 4-Foundation backlog

**Status:** scaffolded 2026-05-11 as Phase 4-Foundation R1 pre-dispatch artifact (per meth-r1r1-1 closure of phantom-destination concern). Mirrors `phase-3-backlog.md` shape.

**Purpose:** named destination for Phase 4-Foundation R6 phase-close convergence carries + dogfood-validation findings + cross-phase carries that surface during Phase 4-Foundation implementation.

Couples to:
- `docs/future/phase-3-backlog.md` — Phase 4-bound carries surfaced before Phase 4-Foundation opened
- `docs/future/phase-3-backlog.md §14` — Phase 4-Foundation carries that fall to v1-assessment-window after Phase 4-Meta close
- `docs/future/phase-3-backlog.md §15.2` — handler-call-graph cycle detection (Phase 4-Meta-bound; couples to plugin install/registration time)
- `docs/future/kith-decentralized-identity.md` — exploratory decentralized-identity-and-attestation system (Phase 5+ candidate)

---

## §1. R6 phase-close convergence carries

Phase 4-Foundation R6 phase-close council will produce findings that don't gate the `phase-4-foundation-close` tag. Those land here as carries to Phase 4-Meta OR v1-assessment-window.

(Entries land during Phase 4-Foundation R6 — none at this writing.)

---

## §2. Dogfood validation gate carries

Ben's dogfood validation (wave-9 in plan §2 sequencing) will produce UX + interaction findings beyond the FIX-NOW-INLINE scope. Those land here per HARD RULE 12 clause-(b).

### §2.1 6 dogfood-path UX-acceptance arms requiring live browser/admin UI surface

**Origin:** G24-A wave-6 canary (2026-05-13). The 6 dogfood-path tests at `crates/benten-engine/tests/dogfood_path_<a..f>_ux_acceptance.rs` carry both an engine-substrate arm (closed at G24-A) and a UX arm requiring a live admin UI client surface. The UX arms BELONG-NAMED-NOW per HARD RULE 12 clause-(b):

- **path (a)** workflow creation ≤5 clicks + click-counter recording → closes at **G24-B wave-6b** when the browser-side workflow editor ships.
- **path (b)** composed-view creator ≤4 clicks + live-preview p50 ≤200ms / p99 ≤1s latency budget → closes at **G24-C wave-6b** with the browser-side view-creator component + materializer pipeline timing harness.
- **path (c)** multi-device sync ≤3s loopback round-trip + "Devices" sub-panel last-sync-time display → closes at **wave-9 dogfood gate** when Ben exercises 2-peer Atrium loop locally.
- **path (d)** revoke-cap mid-session: user-visible "Capability revoked" toast + redacted-state re-render → closes at **G24-C wave-6b + wave-9 dogfood gate** (toast UX in G24-C; live revocation exercise at dogfood gate).
- **path (e)** ≤3 clicks install-consent + plain-English manifest display + per-cap-decline path → closes at **G24-D wave-7 + wave-9 dogfood gate** (consent UX at G24-D's full plugin manifest scope; click-budget validation at dogfood gate).
- **path (f)** install-2nd-plugin same flow + install record signed by user-DID → closes at **G24-D wave-7 + wave-9 dogfood gate**.

The G24-A canary closes the substrate arms: admin UI v0 subgraph composes from existing 12 primitives; route-builder shape generalises across plugins; engine + materializer + Class B β seam route correctly. Pinned by the 6 `dogfood_path_<a..f>_ux_acceptance` tests (production-runtime arms substantively LIVE).

### §2.2 Click-counter test harness + live-DOM workflow editor exercise

**Origin:** G24-A wave-6 canary (2026-05-13); ratification #4 click-budget arms. Tests in §2.1 use the in-process engine + materializer pipeline; the click-recording surface requires a browser-driver (Playwright or webview-bridge) instrumentation harness that lands at G24-B/G24-C wave-6b. Acceptance: each dogfood path's `_ux_acceptance.rs` body extends with the click-count + latency arms once the harness lands.

---

## §3. Phase 4-Foundation → Phase 4-Meta carries

Architectural decisions made during Phase 4-Foundation that explicitly defer related work to Phase 4-Meta land here for tracking.

### §3.1 Decentralized self-discovered registry

**Origin:** Phase 4-Foundation R1 (2026-05-11). Plan §3 originally scoped decentralized self-discovered registry as part of D-4F-1 FULL plugin manifest scope; R1 lenses (plugin-architecture-reviewer + distributed-systems-reviewer + threat-model §8 Q1 cross-cite) surfaced internal contradiction with §3.X T10 deferring discover-flow defenses to Phase 4-Meta. Ben ratified 2026-05-11 evening: **move decentralized self-discovered registry to Phase 4-Meta.** Phase 4-Foundation admin UI v0 installs plugins via direct content-addressed-share over Atriums (peer-to-peer; user pulls from peer they trust).

**Phase 4-Meta scope:** decentralized registry surface (Atrium-substrate publish/subscribe; signed + content-addressed manifest discovery; trust-graph extension); admin UI discovery affordance (search/browse plugins from peers in your network).

**Couples to:** §3.2 Kith (richer identity-and-attestation substrate that the registry's trust-graph would build on).

### §3.2 Kith — decentralized identity & attestation system (EXPLORATORY)

**Origin:** Ben's framing 2026-05-11 evening conversation, during Q6 (peer-DID rotation propagation) discussion. The base Phase-3 peer-DID + RotationLog primitive is insufficient for handling key-rotation in a hostile-old-key scenario. Ben proposed a richer decentralized-identity substrate: "X has designated Y as Z" relational-attestation graph + per-relationship privacy controls + organizational attestations (Gardens/Groves, schools, certifying bodies) + UCAN-mediated contextual sharing.

**Full scope:** see `docs/future/kith-decentralized-identity.md` (exploratory scope-stub).

**Phase target:** **Phase 5+ or its own dedicated design-spike phase**, NOT Phase 4-Foundation (too large; Phase 4-Foundation uses a simpler "old-key revocation attestation + out-of-band new-key trust" MVP rotation mechanism per Q6 ratification).

**Phase 4-Foundation MVP rotation mechanism:**
- Old-key signs a `SelfRevocation` attestation marking itself as revoked (timestamped). Propagates via Atrium sync. Peers reject content signed by the old key after the revocation timestamp.
- New-key trust is NOT transferable from old key. Each peer re-establishes trust via out-of-band side-channel (same channel used for initial bootstrap).
- Grace window during rotation.

This MVP doesn't defeat the purpose of rotation (it doesn't ask receivers to trust the old key for new-key establishment) — it just propagates revocation cleanly.

### §3.3 Self-composing admin UI (meta-circular full scope)

**Origin:** carried from original Phase 4 scope; Phase 4-Foundation ships admin UI v0 that lets users edit workflows + composed views THROUGH it, but does NOT make the admin UI's own subgraph user-editable through itself. That meta-circular self-composing capability is Phase 4-Meta-bound.

### §3.4 Phase 4-Meta inherited carries from Phase 3

- wasmtime Component-Model re-evaluation (Phase-3 D-PHASE-3-6 + D-PHASE-3-16 + r1-wsa-12)
- Engine impl-block generic-cascade lift (Phase-3 §1.2-followup)
- Light-client mode-(b) range-query proof (ds-r4r2-3)
- Light-client mode-(c) signed checkpoint (ds-r4r2-3)
- Handler-call-graph cycle detection at handler-registration time (`phase-3-backlog §15.2`)
- **View 3 (content_listing) stale-with-last-known-good fallback generalization (mat-r1-14)** — the `ContentListingView` budget-exhaustion path returns the LAST KNOWN GOOD snapshot via `read_page_allow_stale` rather than empty. G23-0b preserves this non-trivial-named behavior at the canonical-view inner kernel + does NOT generalize the fallback into a uniform pathway on `Algorithm B`'s generic kernel. Phase 4-Meta lifts the stale-with-last-known-good shape into a general `View::read_allow_stale` semantics that user-defined views can opt into; until then the canonical View 3 path is the only fallback-aware view. Closure pin: `crates/benten-ivm/tests/view_3_stale_with_last_known_good_does_not_generalize_trivially_named_carry.rs`.

---

## §4. Phase 4-Foundation Track B (Class-of-bug audits + cleanups)

Plan G27 wave covers these; entries here for cross-reference.

### §4.1 UCAN class-of-bug audit across napi cap-* entry points

Per D-4F-5 ratification (Phase 4-Foundation Track B). Lateral sweep across napi cap-management entry points for scope-vs-CID-passed-as-string class of mistakes (same root cause as §13.11 fix at PR #199). Plan G27-A wave.

### §4.2 `benten-caps::GrantBackedPolicy::derive_write_scope` lift

Currently hard-codes `store:<label>:write` derivation; thread scope through `WriteContext::scope` (already exists for `UcanGroundedPolicy::check_write`; not yet for `GrantBackedPolicy::check_write`). Plan G27-B wave.

### §4.3 `GrantReader::has_unrevoked_grant_for_grant_cid(&Cid)` CID-keyed companion

Per §13.11 structural lesson — the scope-keyed `has_unrevoked_grant_for_scope(scope: &str)` lacks CID-keyed counterpart at the trait surface, which is what enabled the original `revokeCapability(grantCid, actor)` silent fail-OPEN. Add CID-keyed companion at the trait surface so CID is the canonical typed handle even at the reader API. Plan G27-C wave.

### §4.4 Manifest scope grammar at G27-D

Define mapping from manifest `requires` / `shares` to scope strings; story for `private:<plugin_did>:*` interaction with `wildcard_variants`; install-time-vs-check-time decision. Per cap-r1-3 closure.

### §4.5 `bindings/napi/tests/cap_delegate_napi_resolved_scope_regression_guard.rs` substantive arm — CLOSED at G24-D-FP-3

R5 G27-A landed the napi class-of-bug audit (PR #224 via R5 wave-g27-a; merged 2026-05-13). The audit confirmed 4 cap-* entry points are the complete enumeration of scope-vs-CID class-of-bug risk surfaces. `delegateCapability` was **NOT YET SHIPPED** at the napi layer at G27-A time.

**Retarget rationale (G24-D fix-pass, 2026-05-12):** the original §4.5 destination "at G24-D" was incorrect. G24-D ships the Rust-side `crates/benten-caps/src/plugin_delegation.rs` runtime UCAN delegation envelope-check surface but does NOT ship the napi binding (`delegateCapability(grantCid, plugin_did, attenuated_caps)` from Node-side TS). The napi binding became its own work item per §4.8.

**CLOSED at G24-D-FP-3** (branch `r5/wave-g24-d-fp-3`, builds on G24-D-FP-2 `30327b0`): `Engine::delegate_capability` engine seam shipped at `crates/benten-engine/src/engine_caps.rs` + napi `delegate_capability` binding shipped at `bindings/napi/src/lib.rs` + TS-side `Engine.delegateCapability` shipped at `packages/engine/src/engine.ts` + `cap_delegate_napi_resolved_scope_regression_guard.rs` body rewritten to the 4-step substantive arm (un-ignored). TS-side end-to-end pin at `packages/engine/test/cap_delegate_napi_resolved_scope.test.ts`.

Closes G27-A R5 mini-review MINOR finding `g27a-mr-1`.

### §4.6 G23-A strict 4-of-4 input-dialect validation + arbitrary-schema proptest

R5 G23-A landed the `schema_compiler` canary (branch `r5/wave-g23-a`). The canary enforces the 4-mandatory rule (name / required / default / scope) over **emitted** primitive property bags but NOT over the **input** dialect — the JSON input schema fixtures currently omit `default`, so `default` is silently defaulted to JSON-null at parse time (`crates/benten-platform-foundation/src/schema_compiler/parse.rs:234-244` + parse.rs:391-400). The cap-scope deriver correctly schema-derives `<action>:<SchemaName>.<field_path>` from emit, so the input `scope` field is currently informational; cap-scope discipline is preserved end-to-end.

**Carry-criterion (lands when the canonical-fixture generator is auto-derived from the typed IR, OR earlier if a future wave needs strict input-dialect validation):** the `ParsedSchema` field-parser MUST reject schemas missing `default` with `E_SCHEMA_VOCAB_REQUIRED_PROPERTY_MISSING`, mirroring the existing emit-side enforcement. Today the fixture generator hand-writes JSON schemas which makes strict input validation a fixture-rewrite burden; once fixtures derive from the typed IR (proposed Phase 4-Foundation or Phase 4-Meta), strict 4-of-4 validation at the input dialect boundary lands without fixture churn. Same destination owns the explicit-edge dialect (user-declared edge labels beyond the field-tree-implied edges currently used at canary).

**Companion deferral — arbitrary-schema proptest:** `crates/benten-platform-foundation/tests/prop_schema_compile_is_idempotent_arbitrary_schemas.rs` remains `#[ignore]` at G23-A. The arbitrary-schema generator (`arbitrary_valid_schema_bytes(seed: u64) -> Vec<u8>` in `tests/common/schema_fixtures.rs`) needs the strict input-dialect grammar finalized before it can generate property-test inputs that exercise the dialect boundary, not just emit-side idempotency. The canary already covers fixed-fixture round-trip idempotency via `schema_compiler_round_trip_canonical_bytes_stable.rs` (un-ignored, PASS at G23-A); the proptest arm un-ignores when the strict 4-of-4 input-dialect lands per the carry-criterion above.

**Tentative phase target:** Phase 4-Foundation wave-N (TBD) OR Phase 4-Meta. NOT a v1-blocker — fixed-fixture idempotency at G23-A canary + emit-side 4-of-4 enforcement together suffice for the schema-driven-rendering substantive arm.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination + the work obligation lands NOW. G23-A wave's `parse.rs` source comments + the proptest's `#[ignore]` message cite `docs/future/phase-4-backlog.md §4.6` instead of phantom destinations like "wave-4b".

Closes G23-A R5 mini-review BLOCKER finding `g23a-mr-1` + MAJOR finding `g23a-mr-2`.

### §4.7 RED-PHASE-BODY status terminology (PROCESS NOTE)

The G24-D primary implementer retagged ~33 RED-PHASE test files with a novel `RED-PHASE-BODY` ignore-message prefix to signal "the test body needs a substantive rewrite vs. a fresh un-ignore." The G24-D mini-review correctly identified this as a HARD RULE 12 phantom-destination drift (the retags also cited a phantom "wave-N"). The G24-D fix-pass restored the standard `RED-PHASE (...)` ignore-message shape with SPECIFIC named destinations (G24-D-FP-1 / G24-D-FP-2 / §4.7 / Phase-4-Meta) per pim-12 §3.6e + HARD RULE 12 clause-(b).

**Do not reintroduce `RED-PHASE-BODY` without explicit Ben ratification + a §3.6e clause defining lifecycle separate from `RED-PHASE`.** Orchestrator may surface a §3.6e clause-amendment to Ben if the body-rewrite-vs-fresh-un-ignore distinction proves load-bearing in future waves.

### §4.8 napi `delegateCapability` binding + substantive arm for `cap_delegate_napi_resolved_scope_regression_guard.rs` — CLOSED at G24-D-FP-3

**Origin:** G24-D mini-review BLOCKER g24d-mr-1 closure + retargeting of §4.5 destination. The §4.5 destination ("at G24-D") was incorrect because G24-D ships the Rust-side delegation envelope-check surface only — not the napi Node-side binding.

**CLOSED at G24-D-FP-3** (branch `r5/wave-g24-d-fp-3`, builds on G24-D-FP-2 `30327b0`). All §4.8 acceptance criteria satisfied:
- `cap_delegate_napi_resolved_scope_regression_guard.rs` body rewritten to the 4-step substantive arm; `#[ignore]` removed; new private-namespace + compile-witness companion tests added.
- New napi binding shipped at `bindings/napi/src/lib.rs::Engine::delegate_capability` (gated to `not(target_arch="wasm32")` per the existing `napi_surface` module gating).
- TS class signature surfaced via `packages/engine/src/engine.ts::Engine.delegateCapability` (with the `BentenNative.delegateCapability?` interface entry).
- New TS-side end-to-end test at `packages/engine/test/cap_delegate_napi_resolved_scope.test.ts` exercising the binding (resolved-scope arm + private-namespace-forbidden arm).

**Scope:** new napi function `delegateCapability(grantCid: string, pluginDid: string, attenuatedCaps: string[]) -> string` (returns the resulting delegation grant CID). Wires Node-side TS callers to `benten_caps::plugin_delegation::check_delegation_within_envelope` + the underlying UCAN delegation issuance (audience=plugin-DID + attenuated caps within manifest envelope). The 4-step substantive arm for `cap_delegate_napi_resolved_scope_regression_guard.rs`:
  1. `delegateCapability(grantCid, plugin_did, attenuated_caps)` over napi resolves the napi-passed CID to the underlying grant + invokes envelope-check.
  2. Verify the new delegation Node is minted with the **resolved scope** (not the grantCid as a string — defends the G27-A class-of-bug).
  3. Attempt a write under the delegated cap; verify it admits per the manifest envelope.
  4. Assert the per-row cap-recheck at delivery resolves the scope correctly (couples to G16-B-F per-row recheck at sync delivery).

**LOC estimate:** ~200-400 (napi binding + TS catalog entry + test body rewrite + a benten-caps issuance helper if not already at HEAD).

**Phase target:** **Phase 4-Foundation R5 G24-D-FP-3** (NEW wave; planner adds row to plan §3 §3.5.1 alongside G24-D-FP-1 + G24-D-FP-2). **Dependency:** depends on G24-D primary (envelope-check surface) + G24-D-FP-2 (manifest envelope chain validator seam — the napi binding consumes both). Couples to G27-D (manifest-aware scope derivation) for resolved-scope semantics.

**Acceptance:**
- `cap_delegate_napi_resolved_scope_regression_guard.rs` body rewritten to the 4-step substantive arm; `#[ignore]` removed.
- New napi binding ships in `bindings/napi/src/cap.rs` (or equivalent) with TS class signature in `bindings/napi/index.d.ts`.
- New TS-side test under `packages/engine/test/` exercising the binding end-to-end.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination for the substantive arm + the §4.7 entry lands NOW (the actual code obligation defers to G24-D-FP-3).

### §4.8.1 `Engine::delegate_capability` chain-walk integration (G24-D-FP-3 followup)

**Origin:** G24-D-FP-3 mini-review finding `g24dfp3-mr-1` (MAJOR, phantom-destination drift). FP-3's brief specified consumption of G24-D-FP-2's `crates/benten-caps/src/manifest_envelope_chain_validation.rs::validate_chain_with_manifest_envelope` chain validator. FP-3 actually shipped only the single-step `check_delegation_within_envelope` + an `AllPermit` policy-view placeholder. The chain-walker surface ALREADY EXISTS from FP-2; the integration is the missing piece.

**Scope:** wire `Engine::delegate_capability` through `validate_chain_with_manifest_envelope` so the delegation path walks the FULL UCAN delegation chain (not just one step). Specifically:
1. After single-step envelope check succeeds, look up the `derived_from` ancestor chain by walking the `system:CapabilityGrant` Node's `derived_from` text property recursively back to the user-root grant.
2. For each step in the chain, consult the intermediate plugin's `shares` policy (consumes G27-D's manifest-aware lookup at backlog §4.8 once that lands; until then uses `AllPermit` consistently — same posture as today's single-step).
3. Reject the delegation if ANY chain step traces back to a non-existent / revoked / non-user-root ancestor.

**Acceptance:**
- `Engine::delegate_capability` calls `validate_chain_with_manifest_envelope` (not just `check_delegation_within_envelope`).
- New test pin `engine_delegate_capability_walks_full_delegation_chain_via_fp2_validator.rs` (~60 LOC) asserts: (a) single-step delegation continues to admit (no regression); (b) multi-step delegation walks the full chain; (c) multi-step delegation whose middle hop has a missing `derived_from` ancestor REJECTS with the typed `PluginDelegationOutsideManifestEnvelope` ErrorCode.
- `Engine::delegate_capability` rustdoc updates to remove the §4.8.1 reference once integration lands.

**Phase target:** **Phase 4-Foundation R5 G24-D-FP-3-followup** (likely combined with G27-D since they share the manifest-shares lookup wiring) OR **Phase 4-Meta v1-assessment-window** if chain-walk semantics need broader design.

**Coupling notes:** FP-2's `validate_chain_with_manifest_envelope` is generic over `ManifestEnvelopeLookup` + `UserDidRegistry` traits — both need concrete impls bridging to the engine's manifest store + user-DID registry. Those bridges land at this followup.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination for the chain-walk integration deferral. Closes G24-D-FP-3 mini-review g24dfp3-mr-1.

### §4.9 `PluginManifest::to_canonical_bytes` + DAG-CBOR key-walk inspection

**Origin:** G24-D mini-review fix-pass disposition of two RED-PHASE test pins (`plugin_cross_plugin_reference_uses_content_cid_not_author_did.rs::manifest_canonical_bytes_dag_cbor_encodes_accepts_content_as_cid_array` + `plugin_pull_not_push_no_manifest_schema_version_field.rs::manifest_canonical_bytes_dag_cbor_contains_no_schema_version_key`) that need a public `to_canonical_bytes() -> Vec<u8>` surface at HEAD to assert structural properties of the canonical encoding (Cid array shape; absence of schema_version key).

**Scope:** the helper `compute_content_cid` already serializes via `serde_ipld_dagcbor::to_vec` internally; the proposal is to surface this as a public `pub fn to_canonical_bytes(&self) -> Vec<u8>` method. Test pins then inspect the encoded CBOR key set / structure of `accepts_content` directly.

**Phase target:** **Phase 4-Foundation R5 G24-D-FP-2** (manifest envelope chain validator + canonical-bytes surface). Couples to envelope-chain validation because canonical-bytes is the form the chain validator hashes.

**Acceptance:**
- `PluginManifest::to_canonical_bytes(&self) -> Vec<u8>` shipped pub.
- Two pinned tests un-ignored:
  - `manifest_canonical_bytes_dag_cbor_encodes_accepts_content_as_cid_array` (asserts CID-shaped CBOR for `accepts_content` field; CID byte form vs string form).
  - `manifest_canonical_bytes_dag_cbor_contains_no_schema_version_key` (asserts no schema_version-shaped key in the CBOR map).

### §4.10 RotationLog + HLC-monotonic-strict integration at install/load surfaces

**Origin:** G24-D mini-review fix-pass disposition of three RED-PHASE test pins (`plugin_manifest_author_key_rotation_round_trip.rs` + `plugin_provenance_rotated_key_surfaces_warning_via_rotation_log.rs` + `plugin_manifest_signature_replay_with_different_nonce_rejected.rs`) that need `PluginManifest::validate_with_rotation_log(&rotation_log)` + HLC-strict rotation-event replay defense seams to land.

**Scope:** new public API at `crates/benten-platform-foundation/src/plugin_manifest.rs`:
- `pub fn validate_with_rotation_log(&self, rotation_log: &benten_id::rotation_log::RotationLog) -> Result<ValidationOutcome, ErrorCode>` returning `Valid` / `ValidWithWarning(RotatedKeyWarning)` per D-4F-12 (rotation surfaces warning, not hard-reject by default).
- HLC-monotonic-strict rotation-event ordering defense (couples to existing Phase-3 `benten-id::rotation_log` + HLC infrastructure).

**Phase target:** **Phase 4-Foundation R5 G24-D-FP-2** (manifest envelope chain validator + RotationLog integration). The chain validator already consumes RotationLog state at the Layer 2/3 bridge per plan §3 G24-D-FP-2 row.

**Acceptance:**
- `validate_with_rotation_log` pub method on PluginManifest.
- Three pinned tests un-ignored: rotation round-trip + warning surface + nonce-swap replay rejection.

### §4.11 `ManifestStore::load_verified` + post-install drift detection

**Origin:** G24-D mini-review fix-pass disposition of `plugin_manifest_post_install_drift_detected.rs` — needs a `ManifestStore` durable surface that re-verifies install records on load (post-install byte-mutation defense per threat-model §T10c).

**Scope:** new `crates/benten-platform-foundation/src/manifest_store.rs` (or extension to plugin_library.rs) with `ManifestStore::load_verified(plugin_did) -> Result<InstallRecord, ErrorCode>` that re-runs install-record verification on every load. Backs the persistent half (file-system attack defense) of the user-as-source signing model.

**Phase target:** **Phase 4-Foundation R5 G24-D-FP-1** (plugin_lifecycle hardening; install/uninstall + load surface). Couples to the redb-persisted plugin library work.

**Acceptance:**
- `ManifestStore::load_verified` shipped pub.
- `plugin_manifest_post_install_drift_detected.rs` substantive test body wired against the new load_verified path.

### §4.12 T9 threat-class scope-move: schema-namespace → plugin-manifest-namespace

**Origin:** G23-B mini-review BLOCKER findings `g23b-mr-1` (T9a forged peer-DID signature) + `g23b-mr-2` (T9b rotation-race replay). The R3 test pins (`schema_with_forged_author_signature_rejected.rs` + `schema_author_rotation_race_replay_rejected.rs`) were authored before the post-R1-triage ratifications clarified that **JSON schemas in the ingest dialect are static type definitions** — not peer-signed, not rotation-bearing, not provenance-anchored. The T9 threat-class (forged author signature + rotation-race replay) applies to **plugin manifests**, NOT to schemas.

**Resolution:** the defense already ships in the plugin-manifest namespace:
- **T9a forged peer-DID signature** — G24-D ships `plugin_manifest::sign_manifest` + `verify_peer_signature` + Ed25519 verifier. Substantive coverage at `crates/benten-platform-foundation/tests/g24d_substantive_pipeline.rs::plugin_manifest_peer_did_signature_round_trip` (PASS at HEAD post-batch-2).
- **T9b rotation-race replay** — G24-D-FP-2 ships `crates/benten-id/src/did_rotation.rs::RotationLog::accept_rotation_event` with HLC-monotonic-strict + VerbatimReplay defense. Substantive coverage at `crates/benten-platform-foundation/tests/plugin_manifest_rotation_event_nonce_swap_attack_rejected.rs` (3 attack variants) + `plugin_manifest_peer_did_key_rotation_surfaces_warning_round_trip.rs` (round-trip).

**Schema-namespace pins** (`schema_with_forged_author_signature_rejected.rs` + `schema_author_rotation_race_replay_rejected.rs`) retained as forward-looking documentation should schema-level provenance ever be needed (no current plan to add — schemas are static data and propagate via plugin manifests). Their `#[ignore]` rationale was updated to cite the manifest-namespace destination per HARD RULE 12 clause-(b) BELONGS-NAMED-NOW.

**No further code obligation** — the substantive defense lives at the manifest surface; this entry is the named destination for the schema-namespace pin disposition.

### §4.13 G23-B materializer wave deferred items → G24-A wave-completion sweep

**Origin:** G23-B mini-review (`g23b-mr-3` mr-3 path-b + `g23b-mr-4` + `g23b-mr-5` rename-with-named-destination + `g23b-mr-6` + `g23b-mr-7` + `g23b-mr-8`). Six items scoped to G24-A admin-UI integration:

- **mr-3 (MAJOR, path-b document-helper-boundary)** — `materializer_mat_deny_wins_composition.rs` + `materializer_delivery_deny_wins_composition.rs` exercise the `Materializer::dual_gate_admits` HELPER-FUNCTION contract over two closures; the production walk path `Materializer::materialize_with_gate` threads only the mat-layer gate (delivery composition is the consumer's responsibility per the docstrings). G23-B fix-pass amended both test docstrings to make the helper-function vs production-walk boundary explicit. **G24-A wires the consumer side (`Engine::on_change_as_with_cursor`) and the end-to-end LOAD-BEARING dual-gate composition lands at `materializer_dual_gate_pim_2_end_to_end_would_fail_if_no_op.rs` once the production walk path is connected.**
- **mr-4 (MAJOR, defense-in-depth integration test)** — materializer entry-point at materializer.rs:905-921 has a defense-in-depth re-check for SANDBOX subgraphs whose `sandbox_host_fn` is in the banned set. PRIMARY defense at `schema_compiler::compile` (G23-A); this materializer-entry re-check is for hand-authored `SchemaSubgraphSpec` inputs bypassing schema-compile. Currently the only test exercises the schema_compile primary; the materializer-entry arm has no integration test because constructing a `SchemaSubgraphSpec` directly requires `pub(crate) fn new` access. **G24-A wave-completion sweep adds a `#[doc(hidden)] pub fn for_test_*` constructor on `SchemaSubgraphSpec` + the ~30 LOC defense-in-depth integration test** (asserts `MaterializerError::SchemaMismatch { code: E_MATERIALIZER_SCHEMA_MISMATCH }` fires for hand-authored banned-host-fn spec).
- **mr-5 (MAJOR, rename + named destination)** — test file `materializer_pipeline_reactive_update_propagates_through_subscribe_seam.rs` renamed to `materializer_subscribe_seam_validates_pattern_and_emits_token_for_consumer_wiring.rs` (truthful — body validates pattern-shape + emits token, doesn't exercise propagation). **G24-A consumer wires `Engine::on_change_as_with_cursor`** + adds a NEW substantive propagation pin (file name like `admin_ui_v0_materializer_reactive_update_propagates_through_engine_on_change_as_with_cursor.rs`) that drives a real change event end-to-end + asserts the materialized view updates.
- **mr-6 (MINOR)** — `InMemoryMaterializerEngine` is `pub` in production lib for cross-crate test ergonomics; orchestrator-direct addition of `#[doc(hidden)]` at G23-B fix-pass marks it not-API-stable. G24-A wave-completion sweep verifies the marker still applies and the consumer wiring doesn't elevate it to stable API.
- **mr-7 (MINOR)** — mat-r1-11 "one view per (spec, content) pair" — orchestrator-direct G23-B fix-pass adds the rustdoc on `MaterializerWalkInputs` naming the (spec_cid, content_cid) pair as the view identity. G24-A consumer wiring exercises the multi-instance shape.
- **mr-8 (OBSERVATION)** — per-primitive cap-recheck fan-out passes content_cid uniformly (not per-primitive scope) — orchestrator-direct G23-B fix-pass adds doc-comment making the "invocation-count observability" semantic explicit. G24-A consumer wires the production cap-recheck path and the contract becomes load-bearing there.

**Acceptance:** mr-3 + mr-5 + mr-6 + mr-7 + mr-8 inline-closed in G23-B fix-pass commit (docstring amendments + rename + `#[doc(hidden)]` + 2 rustdoc additions); mr-4 lands at G24-A wave-completion sweep with the doc-hidden test constructor + ~30 LOC integration test. G24-A reviewer brief verifies all 6 items still hold at admin-UI integration boundary.

**G24-A wave-completion sweep status (2026-05-13):**

- **mr-3 CLOSED at G24-A** — end-to-end LOAD-BEARING dual-gate composition test at `crates/benten-platform-foundation/tests/admin_ui_v0_materializer_reactive_update_propagates_through_engine_on_change_as_with_cursor.rs` (`admin_ui_v0_render_dual_gate_deny_from_materialization_layer_wins_end_to_end`). Adapter `tests/common/admin_ui_v0_engine_adapter.rs` bridges `MaterializerEngine` to a real `benten_engine::Engine`. The mat-layer + delivery-layer dual-gate end-to-end pin asserts deny-from-either-layer-wins.
- **mr-4 CLOSED at G24-A** — `#[doc(hidden)] SchemaSubgraphSpec::for_test_from_handcoded_subgraph` constructor lands at `crates/benten-platform-foundation/src/schema_compiler/spec.rs`; integration test at `crates/benten-platform-foundation/tests/materializer_defense_in_depth_rejects_banned_sandbox_host_fn_for_handcoded_spec.rs` exercises 3 banned host-fn variants + positive control. All 4 sub-tests pass.
- **mr-5 CLOSED at G24-A** — NEW substantive propagation pin at `crates/benten-platform-foundation/tests/admin_ui_v0_materializer_reactive_update_propagates_through_engine_on_change_as_with_cursor.rs` (4 sub-tests: routes-through-adapter / propagates-engine-update / dual-gate-deny / invocation-count-observability). The mr-3 dual-gate arm + mr-8 invocation-count arm both share this pin file.
- **mr-6 RE-VERIFIED at G24-A** — `InMemoryMaterializerEngine` retains `#[doc(hidden)]` (confirmed at `crates/benten-platform-foundation/src/materializer.rs:1030-1031`); the G24-A integration adapter at `admin_ui_v0_engine_adapter.rs` wires a different shape (production `Engine` → `MaterializerEngine` trait) so the test-only `InMemoryMaterializerEngine` is NOT elevated to stable API.
- **mr-7 RE-VERIFIED at G24-A** — rustdoc on `MaterializerWalkInputs` (line ~285-300) names `(spec_cid, content_cid)` as the view-identity pair; G24-A consumer wiring at `admin_ui_v0_render_propagates_engine_side_node_update_through_adapter` renders two distinct (same spec, different content_cid) pairs in one test fn — multi-instance shape exercised.
- **mr-8 RE-VERIFIED at G24-A** — the invocation-count-observability semantic is explicit at `materializer.rs:923-943`; the G24-A pin `admin_ui_v0_render_dual_gate_invocation_count_observability` asserts ≥ spec.primitive_count invocations per walk.

---

## §5. Phase 4-Foundation Track A (implementation work surfaced post-R1)

R1-FP work items that emerged from R1 critic round (production-vs-plan gaps). These are Phase 4-Foundation implementation, not deferred carries — listed here for traceability.

### §5.1 UCAN audience binding at `UcanGroundedPolicy::permits_typed_proof_for`

`crates/benten-caps/src/ucan_grounded.rs:191-216` currently calls `validate_chain_at` without audience binding. Add audience-binding wiring per cap-r1-1. ~100-200 LOC + tests. Closes load-bearing BLOCKER for the four-identity-concepts model.

### §5.2 `actor_cid` consulted on reads at `GrantBackedPolicy::check_read`

`crates/benten-caps/src/grant_backed.rs:296-327` currently wildcard-enumerates against scope-only. Add `ctx.actor_cid` consultation per cap-r1-2. ~50-100 LOC. Closes materializer dual-gate substance gap.

### §5.3 SUBSCRIBE-delivery cap-recheck closure

`crates/benten-engine/src/engine_subscribe.rs::Engine::on_change_as_with_cursor` (lines 290-327) is scaffold-only — calls `is_actor_active` not per-event `CapabilityPolicy::check_read`. Closure per sec-4f-r1-1; ~100-200 LOC. Closes admin UI dogfood path (d) revoke-cap-mid-session.

### §5.4 `plugin_lifecycle.rs` uninstall-cascade seam

Per plugin-arch-r1-2; ~150-300 LOC. Prevents orphan delegated-cap accumulation at uninstall time.

### §5.5 `manifest_envelope_chain_validation.rs` seam

Per plugin-arch-r1-3; ~200-300 LOC. Wires CLAUDE.md #18 Layer 3 runtime-delegation-within-manifest-envelope structurally.

---

## §6. Doc retense + ErrorCode catalog work

### §6.1 ERROR-CATALOG.md companion-with-canary routing

Per doc-r1-1 + doc-r1-2: 17+ new ErrorCodes for Phase 4-Foundation mint across waves (3 schema + 3 materializer + 9 plugin + new G27 surface). ERROR-CATALOG.md retense MUST land COMPANION-WITH-CANARY per wave, not bundled at G26-A. CATALOG_VARIANT_COUNT expected bump 118 → ~135.

### §6.2 INTERNALS.md retense for new surfaces

Per cross-lens doc-engineer findings: `benten-platform-foundation/INTERNALS.md` (NEW; 12th workspace crate), `benten-renderer-tauri/INTERNALS.md` (NEW; 12th-or-13th crate), updates to `benten-ivm/INTERNALS.md` (post IVM-subgraph generalization), `benten-engine/INTERNALS.md` (post audience-binding + actor_cid wiring + SUBSCRIBE-cap-recheck closure), `benten-caps/INTERNALS.md` (post Q5 plugin-DID-keyed signing-key infrastructure).

---

(Section structure additive; entries land as Phase 4-Foundation work surfaces them.)
