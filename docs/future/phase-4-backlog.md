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

### §3.5 `benten-admin-shell` webview-driven tauri-driver smoke test (Phase-4-Meta)

**Origin:** Phase 4-Foundation R6 phase-close R6-FP-E (2026-05-13). The R6-R1 browser-runtime lens (`.addl/phase-4-foundation/r6-r1-browser-runtime-reviewer.json` finding `br-r6-r1-3` MAJOR) named two halves of integrator-binary work:

- **Half (i) — integrator-binary scaffold + IPC dispatch pipeline E2E pin.** CLOSED at R6-FP-E (this PR). `tools/benten-admin-shell/` ships the bin crate; `tests/e2e_admin_shell_ipc.rs` exercises the full T3 three-rung defense + bridge resolve through 9 production-arm + negative-arm pins.
- **Half (ii) — webview-driven tauri-driver smoke test.** A real Tauri 2.x `tauri::Builder` invocation that loads `tools/benten-admin-shell/webview-assets/index.html` into an embedded WebView2 (Windows) / WKWebView (macOS) / WebKit2GTK (linux), driven by `tauri-driver` over WebDriver, asserting a click in the webview triggers an IPC roundtrip + a DOM update. Deferred here per HARD RULE rule-12 clause-(b).

**Deferral rationale (R6-FP-E ratification 2026-05-13):**

- Pulling `tauri = "2"` into the workspace `Cargo.lock` adds ~533 transitive crates + a deny.toml license-audit pass; on linux it adds the `webkit2gtk-4.1` runtime dependency to the CI matrix.
- The half-(i) scaffold's `AdminShellState::dispatch` IS the code path a real Tauri 2.x command handler invokes one-to-one — the webview-driver layer adds the actual webview runtime but the IPC dispatch substance is exercised end-to-end already.
- Cost-benefit: half-(ii) gains the actual webview-runtime integration test (defense against a regression in `WebviewWindowBuilder::with_csp()` semantics or Tauri command-payload handling) but does NOT close a security guarantee the half-(i) tests don't already cover. The CSP header byte-shape is asserted by `e2e_integrator_publishes_canonical_csp_header` + `webview_assets_csp_meta_matches_rust_constant`; the dispatch pipeline is exercised end-to-end through 8 negative-arm + happy-path cases.

**Phase 4-Meta scope:**

- Add `tauri = "2"` to `tools/benten-admin-shell/Cargo.toml` `[features.tauri.dependencies]` block + flip the `tauri_boot::run` placeholder body to a real `tauri::Builder::default().invoke_handler(...).run(...)` call wiring Tauri commands against `AdminShellState::dispatch`.
- Add `tauri-driver` + a WebDriver client (`fantoccini` or `thirtyfour`) as `[dev-dependencies]` under the `tauri` feature.
- New e2e test `tools/benten-admin-shell/tests/e2e_webview_driver_smoke.rs` (feature-gated): launches the admin-shell binary in `tauri-driver` mode; clicks `#ipc-roundtrip`; asserts the DOM `#response` updates with the IPC response; verifies CSP enforcement by attempting a `fetch("https://attacker.example")` from webview JS and asserting it's blocked.
- New CI workflow `.github/workflows/admin-shell-webview-e2e.yml` matrix = `ubuntu-latest` + `macos-latest`; installs `webkit2gtk-4.1` on linux; non-required check initially; promote to required after first green run.
- Update Cargo.toml header (`tools/benten-admin-shell/Cargo.toml`) describing the feature flip + the dep-tree audit results against `deny.toml`.

**Estimated scope:** ~200-300 LOC (binary feature-mode body + e2e test + workflow). Couples to §3.3 (self-composing admin UI Phase-4-Meta scope) which DEMANDS a live webview for the meta-circular editing surface — the half-(ii) wave can land alongside §3.3 to share CI infrastructure cost.

Closes R6 R6-R1 finding `br-r6-r1-3` named-NOW destination clause-(b) half (ii). Couples to `br-r6-r1-5` MINOR (integrator-binary handler payload overwrite contract — the per-method handlers land at the same wave).

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

**Ben ratification 2026-05-13 (Q1):** `RED-PHASE-BODY` is **NOT codified** as a separate §3.6e sub-rule. Treated as one-time anomaly. The G24-D fix-pass treatment is the precedent: every novel pin-status invention must (a) wire the test substantively against the existing surface OR (b) retag with standard `RED-PHASE` + a SPECIFIC named destination per HARD RULE 12 clause-(b). Future agents who feel the body-rewrite-vs-fresh-un-ignore distinction would be useful should propose it as a Ben ratification request, not unilaterally invent a new pin status. Closure: R4b-FP-3 inline.

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

### §4.14 T1 + T7 LOAD-BEARING end-to-end pins → G24-B-FP-1 harness graduation [CLOSED at G24-B-FP-1]

**Origin:** G24-A mini-review BLOCKER findings `g24a-mr-1` (T1 hostile-schema), `g24a-mr-2` (T7 private-namespace), + paired MAJOR `g24a-mr-3` (T1 benign-control). All three R3 RED-PHASE pins cited "un-ignore at G24-A landing" but G24-A shipped the substrate module (admin_ui_v0/mod.rs) + engine adapter bridge WITHOUT graduating the full end-to-end test harness (`AdminUiV0TestHarness::new()`) that the substantive arms require. Per pim-12 §3.6e, the wave-citation must match the actual un-ignore wave; per HARD RULE 12 clause-(b) BELONGS-NAMED-NOW, the destination needs a specific named follow-up wave.

**Scope:** new wave **G24-B-FP-1** (NEW fp wave alongside G24-B workflow editor; planner adds row to plan §3 §3.5.2 alongside existing G24-D-FP-* family). G24-B-FP-1 ships:
1. `AdminUiV0TestHarness::new()` — substantive test-harness graduation that wires:
   - Schema-compile → `Engine::register_subgraph` end-to-end
   - `Engine::call_as(plugin_did, ...)` with the admin-UI plugin-DID as principal
   - Materializer dispatch through the harness
   - Cross-plugin install path for private-NS isolation tests (T7)
2. Un-ignore + wire substantive bodies for the 3 pins:
   - `crates/benten-engine/tests/admin_ui_v0_hostile_schema_read_emit_chain_denied.rs` (T1 LOAD-BEARING)
   - `crates/benten-engine/tests/admin_ui_v0_benign_schema_renders_correctly.rs` (T1 paired regression-guard)
   - `crates/benten-engine/tests/admin_ui_v0_private_namespace_isolated_from_other_plugins.rs` (T7 LOAD-BEARING)

**Acceptance:**
- `AdminUiV0TestHarness::new()` ships with substantive end-to-end wiring (NOT a stub).
- All 3 pin bodies replaced from `unimplemented!()` → substantive arms per pim-2 §3.6b (PRODUCTION-ARM + OBSERVABLE-CONSEQUENCE + WOULD-FAIL-IF-NO-OP).
- T1 hostile-schema arm asserts the hostile READ→EMIT chain is structurally denied (not just a generic error — specific deny-at-cap-policy path).
- T7 private-namespace arm asserts cross-plugin write into `private:<plugin_did_other>:*` namespace yields `E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN` (already minted at G24-D).
- T1 benign-control passes (regression-guard against over-rejection).

**Phase target:** **Phase 4-Foundation R5 G24-B-FP-1** (NEW wave; lands alongside G24-B workflow editor implementer).

**Coupling notes:** G24-D's `g24d_substantive_pipeline.rs::private_namespace_cap_unconditionally_denied_cross_plugin` (PASS at HEAD) covers the STRUCTURAL private-NS defense at the cap-policy layer; G27-D's `private_namespace_scope_admits_only_plugin_did_actor` covers the scope-derivation layer. The §4.14 T7 pin closes the END-TO-END arm via the admin-UI v0 plugin install path — a different surface than the existing structural pins.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination for the 3 deferred T1+T7 LOAD-BEARING pins. Closes G24-A mini-review g24a-mr-1 + g24a-mr-2 + g24a-mr-3.

**Closure (G24-B-FP-1):** `AdminUiV0TestHarness::new()` graduated to a composed-engine + materializer end-to-end harness (`crates/benten-engine/tests/common/admin_ui_v0_harness.rs`). The 3 LOAD-BEARING pins un-ignored with substantive arms:
- `admin_ui_v0_hostile_schema_read_emit_chain_denied.rs` — 2 sub-tests pinning T1 envelope-recheck against hand-coded hostile schemas; asserts typed `E_MATERIALIZER_SCHEMA_MISMATCH` + diagnostic-names-attempted-scope forensic visibility.
- `admin_ui_v0_benign_schema_renders_correctly.rs` — 2 sub-tests pinning the T1 regression-guard arm: real `schema_compile` + materializer walk through HarnessEngineAdapter renders content + structural cap-recheck fires (Compromise #11 closure floor).
- `admin_ui_v0_private_namespace_isolated_from_other_plugins.rs` — 3 sub-tests pinning T7 end-to-end against `Engine::delegate_capability` (refusal fires + target-DID independence + non-private-scope regression-guard).

### §4.15 Defense-in-depth SANDBOX 4th banned host-fn (`edges:remove`) coverage gap [CLOSED at G24-B-FP-1]

**Origin:** G24-A mini-review `g24a-mr-4` OBSERVATION. The `materializer_defense_in_depth_rejects_banned_sandbox_host_fn_for_handcoded_spec.rs` pin (G24-A wave) exercises 3 of the 4 banned host-fns (`kv:write` + `kv:delete` + `edges:add`); `edges:remove` is named in the module doc + the production runtime banned-set but not pinned by a sub-test.

**Scope:** add 4th sub-test arm `for_handcoded_spec_with_edges_remove_host_fn_rejected_at_materializer_entry` mirroring the existing 3-variant shape. ~15 LOC.

**Phase target:** **Phase 4-Foundation R5 G24-B-FP-1** (alongside the §4.14 harness graduation — same fp wave, adjacent test surface).

**Acceptance:** 4 sub-tests in `materializer_defense_in_depth_rejects_banned_sandbox_host_fn_for_handcoded_spec.rs` (3 existing + 1 new for `edges:remove`); all assert `MaterializerError::SchemaMismatch { code: E_MATERIALIZER_SCHEMA_MISMATCH }`.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination. Closes G24-A mini-review g24a-mr-4 OBSERVATION.

**Closure (G24-B-FP-1):** `materializer_rejects_handcoded_spec_referencing_edges_remove_host_fn` sub-test landed alongside the existing 3 banned-host-fn arms. Asserts `MaterializerError::SchemaMismatch { code: MaterializerSchemaMismatch }` + diagnostic naming `edges:remove`.

### §4.16 G24-B workflow editor substantive replay arm via real engine round-trip [CLOSED at R4b-FP-2]

**Origin:** G24-B mini-review MAJOR finding `g24b-mr-1`. The existing `replay_produces_identical_content_hash` canary (`crates/benten-platform-foundation/src/admin_ui_v0/workflow_editor.rs:613`) is a degenerate same-struct double-hash: both sides call `blake3(canonical_subgraph_bytes(&sg_save))` on the same in-memory Subgraph; no encode → store → load → decode cycle is exercised. Same shape on the TS side (`packages/admin-ui-v0/tests/workflow_editor_creates_workflow_and_replays_through_evaluator.test.ts` uses in-memory Map + FNV-1a hash). Plan §3 G24-B row explicitly requires "PRODUCTION substantive arm (workflow CREATED is persisted to redb + readable via Engine::read_node + replays with same CID), NOT shape-only."

**Scope:** new integration pin file `crates/benten-engine/tests/admin_ui_v0_workflow_editor_substantive_replay_via_harness.rs` (~80 LOC) using the G24-B-FP-1-graduated `AdminUiV0TestHarness::new()`:
1. PRODUCTION-ARM: drive `compile_draft_within_manifest_envelope` → persist to redb via real `Engine::create_node` (or similar public surface) under admin-UI plugin-DID principal via `Engine::call_as`.
2. OBSERVABLE-CONSEQUENCE: read the persisted Node back via `Engine::read_node_as(admin_ui_principal_cid, persisted_cid)` (Class B β read seam per CLAUDE.md #18); reconstruct the canonical-bytes encoding; re-derive the content hash; assert byte-for-byte equality with the save-time hash.
3. WOULD-FAIL-IF-NO-OP: if `Engine::create_node` stored a different encoding than `canonical_subgraph_bytes` emits, the persisted CID would differ from the save-time hash; this test fires.

**Blockers to land at G24-B (encountered during batch-5 assembly attempt):**
- `fixture_manifest` is `pub(crate)` — needs `#[doc(hidden)] pub fn fixture_manifest_for_test()` OR test-helper module surface
- `canonical_subgraph_bytes` is private — needs `#[doc(hidden)] pub fn` test-helper variant or expose the public path
- `Node` struct construction in test context requires deeper familiarity with `benten-core` Node shape and the engine's content-addressing path

The harness's `create_test_node(&Node) -> Result<Cid, EngineError>` already exists; the gap is exposing workflow_editor's internal canonical-bytes helper through a doc-hidden test surface (mirrors the §4.13 mr-4 pattern of doc-hidden test constructors on SchemaSubgraphSpec).

**Phase target:** **Phase 4-Foundation R5 G24-B-FP-2** (NEW wave; companion to G24-B). LOC estimate: ~120 (test pin ~80 + ~40 for doc-hidden test-helper exposure on workflow_editor.rs internal helpers).

**Acceptance:**
- `#[doc(hidden)] pub fn` test-helper exposures on `workflow_editor.rs` for `fixture_manifest` + `canonical_subgraph_bytes` (or equivalent surface).
- NEW substantive pin `admin_ui_v0_workflow_editor_substantive_replay_via_harness.rs` per the scope above.
- Existing `replay_produces_identical_content_hash` canary renamed/documented as the encoding-only unit-level pin (clarifies its scope vs the new integration arm).

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination. Closes G24-B mini-review g24b-mr-1 MAJOR.

**Closure (R4b-FP-2):**
- `#[doc(hidden)] pub fn fixture_manifest_for_test(scopes: &[&str]) -> PluginManifest` + `#[doc(hidden)] pub fn canonical_subgraph_bytes_for_test(sg: &Subgraph) -> Result<Vec<u8>, CoreError>` exposed at `crates/benten-platform-foundation/src/admin_ui_v0/workflow_editor.rs` (mirrors §4.13 mr-4 doc-hidden test-helper pattern).
- NEW substantive integration pin `crates/benten-engine/tests/admin_ui_v0_workflow_editor_substantive_replay_via_harness.rs` drives the full encode → `register_subgraph` → `create_node` → `read_node_as` → decode → re-encode → re-hash round-trip via `AdminUiV0TestHarness::new()`. Asserts (a) handler-version-chain head CID matches `Subgraph::cid()`, (b) reloaded canonical bytes byte-equal save-time bytes, (c) replay hash equals save hash, (d) replay CID equals save CID, (e) reloaded subgraph's handler_id / primitive-count / edge-count preserved.
- Inline canary `replay_produces_identical_content_hash` renamed to `replay_produces_identical_content_hash_encoding_only` with a docstring naming its degenerate same-struct double-hash scope + cross-referencing the integration pin as the substantive arm.

### §4.17 G24-B + G24-C cross-language drift-defense pins (MINORs) [CLOSED at R4b-FP-2]

**Origin:** G24-B mini-review `g24b-mr-2` MINOR (no parity drift-defense for `WorkflowFormField` + `CANONICAL_12_PRIMITIVE_KINDS` between Rust + TS) + G24-C mini-review `g24c-mr-1` MINOR (TS `UserViewSpec.anchorPatternLabel` field is TS-side-only; not in Rust `SubgraphSpec`; unguarded).

**Scope:**
- For `WorkflowFormField` + `CANONICAL_12_PRIMITIVE_KINDS`: add either (a) cross-language drift-defense test pin that asserts TS shape mirrors Rust shape OR (b) a sharpened docstring naming the parity contract + a sentinel test asserting both sides export the same constants.
- For `UserViewSpec.anchorPatternLabel`: either (a) add an inline drift-defense pin that asserts TS adds anchorPatternLabel intentionally + Rust SubgraphSpec lacks it (locked semantic) OR (b) sharpen the TS docstring naming `anchorPatternLabel` as TS-side-only UX metadata.

**Phase target:** **Phase 4-Foundation R5 G24-B-FP-2** (companion to §4.16 — same fp wave; both are admin-UI-v0-side polish).

**Acceptance:**
- Drift-defense pin OR sharpened docstring for both surfaces.
- §3.5g cross-language rule-mirror discipline preserved.

Per HARD RULE rule-12 BELONGS-NAMED-NOW. Closes G24-B mr-2 MINOR + G24-C mr-1 MINOR.

**Closure (R4b-FP-2):**
- NEW drift-defense pin file `crates/benten-engine/tests/workflow_editor_cross_language_drift_defense.rs` with 3 sub-tests: (a) `workflow_form_field_ts_shape_mirrors_rust_struct_fields` — grep-asserts each Rust `WorkflowFormField` field has a TS `readonly <camelCase>` declaration inside the `export interface WorkflowFormField` body; (b) `canonical_12_primitive_kinds_ts_set_mirrors_rust_primitivekind_enum` — grep-asserts each Rust `PrimitiveKind` variant appears in BOTH the TS `CANONICAL_12_PRIMITIVE_KINDS` set AND the `WorkflowPrimitiveKind` union type (defense-in-depth across both TS surfaces); (c) `user_view_spec_anchor_pattern_label_is_intentionally_ts_only_per_3_5g_exception` — grep-asserts the explicit `§3.5g cross-language rule-mirror EXCEPTION — INTENTIONALLY TS-side-only` docstring marker survives on the TS-side `anchorPatternLabel` field.
- TS-side docstring sharpening at `packages/admin-ui-v0/src/view-composer/view_spec.ts` enumerates `anchorPatternLabel` as a deliberate §3.5g EXCEPTION (UX-side metadata; not in Rust `SubgraphSpec`); names the drift-defense pin's grep target so any future docstring drift fails the pin.

### §4.18 G24-B `pnpm-lock.yaml` tracking + G24-C Rust-side revoke-mid-preview pin [CLOSED at R4b-FP-2]

**Origin:** G24-B mini-review `g24b-mr-3` MINOR (`packages/admin-ui-v0/pnpm-lock.yaml` un-tracked; sibling `packages/engine/pnpm-lock.yaml` IS tracked — workspace convention demands tracking) + G24-C mini-review `g24c-mr-2` OBSERVATION (no Rust-side revoke-mid-preview pin coupling admin UI consumer to real `CapRecheckOutcome::Cancel → E_SUBSCRIBE_REVOKED_MID_STREAM` propagation; TS revoke test synthesizes the sentinel in-bridge).

**Scope:**
- Commit `packages/admin-ui-v0/pnpm-lock.yaml` (re-run `pnpm install` cleanly in the workspace + check in the lockfile).
- Add Rust-side revoke-mid-preview pin at `crates/benten-engine/tests/admin_ui_v0_composed_view_creator_revoke_mid_preview_terminates_live_preview.rs` (or similar location) that drives real `Engine::on_change_as_with_cursor` + revokes the cap mid-stream + asserts `E_SUBSCRIBE_REVOKED_MID_STREAM` surfaces with proper absorbing-state semantics. Couples to Phase-3 G16-B-F per-row recheck + G16-B-C1 SubscribeRevokedMidStream contract.

**Phase target:** **Phase 4-Foundation R5 G24-B-FP-2** (companion to §4.16 + §4.17 — same fp wave).

**Acceptance:**
- `packages/admin-ui-v0/pnpm-lock.yaml` tracked in repo.
- Rust-side substantive revoke-mid-preview pin landed + PASS.

Per HARD RULE rule-12 BELONGS-NAMED-NOW. Closes G24-B mr-3 MINOR + G24-C mr-2 OBS.

**Closure (R4b-FP-2):**
- `packages/admin-ui-v0/pnpm-lock.yaml` generated via `pnpm install` + checked in (mirrors `packages/engine/pnpm-lock.yaml` tracking convention).
- NEW Rust-side substantive pin `crates/benten-engine/tests/admin_ui_v0_composed_view_creator_revoke_mid_preview_terminates_live_preview.rs` (required-features = `["test-helpers"]`) drives real `Engine::on_change_as_with_cursor` under an admin-UI plugin-DID principal + a `GrantBackedPolicy` engine; flips whole-actor revocation via `Engine::testing_revoke_cap_mid_call`; asserts (a) `Subscription::termination_reason() == Some(ErrorCode::SubscribeRevokedMidStream)`, (b) `Subscription::is_active() == false`, (c) `subscribe_revoked_mid_stream_count()` increments by exactly 1, (d) post-Cancel events do NOT re-fire the callback (absorbing-state property). Couples to Phase-3 G16-B-F per-row cap-recheck + R6-FP Wave-C1 typed-error contract + Phase-4-Foundation G22-FP-1 option-D `CapRecheckOutcome` enum.

### §4.19 `plugin_lifecycle::accept_atrium_share` cross-peer install seam + schema-author trust-list user-prompt surface (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R4b-FP-1 + R4b-FP-3 both consolidated stranded pins here):

**(a) Cross-peer install seam.** Production surface NOT YET BUILT for the cross-Atrium-peer install path. R4b-FP-1 closed the single-process `install_plugin` lifecycle but the cross-peer share-and-install pipeline (peer A publishes admin UI bundle through Atrium; peer B receives + verifies via the existing benten-sync `HandshakeFrame` device-DID-attestation pathway from Phase-3 G16-D wave-6b PR #163; bytes hydrate into peer B's ManifestStore; peer B's user-DID consents via local-anchored InstallRecord) lands at Phase-4-Meta.

**Stranded cross-peer pin destinations** (each test's ignore message MUST be updated to point at THIS row when the wave lands):

- `crates/benten-platform-foundation/tests/admin_ui_v0_install_as_signed_plugin_across_two_atrium_peers.rs:8` (single ignored test) — end-to-end cross-peer install via `accept_atrium_share`.
- `crates/benten-platform-foundation/tests/admin_ui_v0_atrium_share_unattested_peer_rejected.rs:41` (single ignored test, T6c) — HandshakeFrame peer-DID validation rejection arm.
- `crates/benten-platform-foundation/tests/admin_ui_v0_install_rejects_substituted_bundle_via_peer_did_signature.rs:14` (single ignored test, T6b end-to-end) — substitution defense end-to-end (the trust-list arm closed at R4b-FP-1; the cross-peer end-to-end remains).

Estimated scope: ~300-500 LOC (cross-peer test fixtures + accept_atrium_share entry + HandshakeFrame integration into the install path + manifest-store hydration). Couples to §3.1 (decentralized registry) once Phase-4-Meta opens.

**(b) Schema-author trust-list user-prompt surface.** Per R4b L1 finding r4b-l1-6 + Ben Q3 ratification at r4-triage §7, v1 admin UI ships with default-trust-not-shown (default trust-list = EMPTY); the explicit `ProvenanceOutcome::UserPromptRequired` surface + admin-UI prompt UX is an enhancement. **v1 admin UI is functional without this surface** — installs ship the manifest-envelope + plugin-DID checks already; the missing piece is the UX prompt flow.

- `crates/benten-platform-foundation/tests/schema_author_not_in_admin_ui_trust_list_prompts_user.rs:41` — un-ignore via real `ProvenanceOutcome::UserPromptRequired` variant + admin-UI v0 (or v1 if reframed) UX surface + assert prompt path is taken on untrusted-author schemas.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination. Closes R4b L1 findings r4b-l1-4 (PLUGIN_DID positive arm closed at R4b-FP-1; no longer deferred) + r4b-l1-6 (schema_author trust-list — deferred here) + R4b-FP-1's three cross-peer redirects, all via Phase-4-Meta carry per Ben Q4 ratification.

### §4.20 `validate_with_clock` end-to-end thread through engine builder + IndexedDB clock-injection (Phase-4-Meta)

R4b-FP-1 Seam 2 shipped `PluginManifest::validate_with_clock` + threaded through `plugin_lifecycle::install_plugin`. The end-to-end **engine builder** clock-injection seam (`EngineBuilder::clock_source` plumbed through to the install path's `now_secs` AND through IndexedDB persistence so a thin-compute-surface install consults the injected clock at hydrate time too) lands at Phase-4-Meta. ~100-200 LOC.

### §4.21 `install_plugin` Steps 9/10/11 partial-failure rollback semantics (Phase-4-Meta)

R4b-FP-1 Seam 1 shipped the 11-step `plugin_lifecycle::install_plugin` pipeline. Steps 8 (DID mint + persist), 9 (cap cascade mint), 10 (private-ns provision), and 11 (library insert + active ref) each early-return on `Err` via `?`, which can leave partial state behind in the engine adapter's production cascade (e.g. plugin-DID persisted at Step 8 with no library entry if Step 9 fails). The `InMemoryInstallCascade` test default has all infallible paths so the no-partial-state invariant is structurally enforced for the v1 test suite, but the engine adapter that wires the real grant store + plugin-DID store at Phase-4-Meta MUST define rollback shape: either (a) transactional install (all-or-nothing across Steps 8-11), or (b) post-install reconciliation pass that detects + cleans up partial-state residue (`plugin_did` in store with no library entry → revoke + drop). Cite: `crates/benten-platform-foundation/src/plugin_lifecycle.rs:701-790` Steps 8-11; mini-review `.addl/phase-4-foundation/r4b-fp-1-mini-review.json` finding `r4b-fp-1-mr-2`. ~150-300 LOC + transactional test pins.

### §4.22 `admin_ui_v0` thin-client bridge surface (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 cluster — thin-client bridge family). G24-A landed the admin UI v0 categories + composed engine harness scaffolding; G24-F shipped `DidKeyedSession::resolve` / TTL enforcement / origin pinning at `crates/benten-engine/src/thin_client.rs`. The **thin-client bridge** itself — the surface that consumes `DidKeyedSession::resolve(token) -> Principal` and wires `Engine::call_as(Principal, ...)` for thin-compute-surface clients (shape (b) per CLAUDE.md #17) — is NOT YET BUILT. Production sites do not exist for: bridge principal-resolution, DID-handshake required for writes, cap-token storage grep-pin via headless browser dogfood, CSP directives lock, session-token TTL end-to-end via Atrium bridge, bundle-integrity load-time check, CSRF cross-origin POST defense end-to-end. Scope target: Phase-4-Meta — couples to admin UI v0 shape (c) embedded-webview launch (Tauri) + decentralized-registry hydrate path.

**Stranded thin-client-bridge pin destinations** (each test's ignore message MUST cite §4.22):

- `crates/benten-engine/tests/admin_ui_v0_thin_client_bridge_resolves_principal_from_session_not_client.rs` — T2 defense 3 second clause; bridge resolves principal from `DidKeyedSession::resolve`, never client-asserted.
- `crates/benten-engine/tests/admin_ui_v0_thin_client_did_handshake_required_for_writes.rs` — T2 defense 1; bridge invocation without session-token → DENIED.
- `crates/benten-engine/tests/admin_ui_v0_thin_client_no_cap_tokens_in_browser_storage.rs` — T2 defense 2; headless browser dogfood; zero cap-token writes.
- `crates/benten-engine/tests/admin_ui_v0_thin_client_csp_directives_locked.rs` — T2 defense 5; CSP headers from full-peer admin UI.
- `crates/benten-engine/tests/admin_ui_v0_thin_client_session_token_time_bound.rs` — T2 defense 2; replay-past-TTL → E_THIN_CLIENT_SESSION_EXPIRED end-to-end via composed-engine harness.
- `crates/benten-engine/tests/admin_ui_v0_thin_client_bundle_integrity_verified_at_load.rs` — T2 defense 4 + T5b; substituted bundle bytes at CID-Y rejected.
- `crates/benten-engine/tests/admin_ui_v0_csrf_attempt_via_cross_origin_post_denied.rs` — T2 LOAD-BEARING end-to-end; origin pinning end-to-end.
- `crates/benten-engine/tests/admin_ui_v0_no_cap_tokens_persisted_to_browser_storage.rs` — T2 defense 2; admin UI source grep-assert against cap-token write to browser storage.

Estimated scope: ~500-800 LOC (bridge module + composed-engine harness extensions + Tauri/embedded-webview adapter glue for shape (c)). Couples to §4.20 (validate_with_clock e2e via EngineBuilder + IndexedDB).

### §4.23 `admin_ui_v0` user-DID root-chain write-boundary validator (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 + sdr-r6-r1 cluster — G24-B-FP family). G24-B / G24-B-FP shipped the workflow editor surface; the **synchronous write-boundary chain validator** that verifies every WRITE traces back to a user-DID root grant (CLAUDE.md #18 Layer 1 user-as-root invariant, runtime-enforced) is NOT YET WIRED. The grant chain is structurally present (cap minted under `audience=plugin_did` from `user_did` at install-time), but the WRITE primitive's evaluator dispatch does not currently re-verify the chain ends at a user-DID at admission time.

**Stranded write-boundary-chain-validator pin destinations** (each test's ignore message MUST cite §4.23):

- `crates/benten-engine/tests/admin_ui_did_cannot_mint_root_grant.rs` — admin UI plugin-DID cannot mint a root grant (only user-DID can; structural defense vs. plugin-elevation).
- `crates/benten-engine/tests/admin_ui_v0_background_write_must_trace_to_user_root.rs` — background write attempts trace WRITE → plugin grant → user root.
- `crates/benten-engine/tests/admin_ui_v0_user_initiated_write_succeeds.rs` — positive arm.
- `crates/benten-engine/tests/cap_policy_chain_validation_at_write_boundary.rs` — synchronous validation arm.

Estimated scope: ~200-400 LOC (chain-walker helper in `benten-caps` + dispatch hook at WRITE primitive admit-time). Couples to §5.5 manifest-envelope-chain-validation seam.

### §4.24 Materializer recursive walk into vocabulary edges (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 schema-mat-r6-6 + sdr-r6-r1 narrative cluster). R6-FP-BF wired the 5 missing vocabulary edges (`ITEM_TYPE` / `KEY_TYPE` / `VALUE_TYPE` / `REF_TARGET` / `VARIANT`) at emit time per schema-mat-r6-1 path-(a); the **materializer's recursive walk** that consumes those edges at materialize time — resolving `FieldRef::REF_TARGET` content via a secondary `read_node_as` against the referenced content-CID; iterating `FieldList` / `FieldMap` elements via `ITEM_TYPE` / `VALUE_TYPE` descriptor lookup; dispatching `FieldEnum` / `FieldUnion` variant selection via `VARIANT` edges — lands at Phase-4-Meta when admin UI v0 nested-form rendering drives the need. The G23-B canary's opcode-list-shaped walk is sufficient for v1 platform-shippable framing (admin UI shows flat schemas); recursive composition for nested forms is the Phase-4-Meta driver. Scope target: ~200-300 LOC at `crates/benten-platform-foundation/src/materializer.rs::materialize_format` recursion arm + integration pin `materializer_resolves_field_ref_target_via_engine_read_node_as.rs`.

### §4.25 Atrium-share CID + peer-DID verification at sync layer (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 — atrium-share cluster). Cross-Atrium plugin-share verification (a peer publishes plugin bytes through Atrium; receiver verifies `(bytes_cid == announced_cid)` AND `peer_did_signature_valid_for_bytes`) is NOT YET WIRED at the `benten-sync` layer. The single-peer install pipeline at `plugin_lifecycle::install_plugin` performs the CID + signature check, but the sync-layer entry point that hydrates received plugin bytes into the ManifestStore does NOT yet re-verify. Couples to §4.19 (a) cross-peer install seam — both surfaces land together.

**Stranded atrium-share pin destinations** (each test's ignore message MUST cite §4.25):

- `crates/benten-sync/tests/admin_ui_v0_atrium_share_bytes_dont_match_announced_cid_rejected.rs` — substitution defense at sync hydrate.
- `crates/benten-sync/tests/admin_ui_v0_atrium_share_substitution_with_different_author_rejected.rs` — peer-DID signature mismatch defense at sync hydrate.

Estimated scope: ~150-250 LOC (sync hydrate-time verifier + integration into `benten-sync::HandshakeFrame` consumer path). Phase-4-Meta carry.

### §4.26 RotationLog rehydration at engine open + `resolve_did_for_cid` round-trip (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 — benten-id rehydrate cluster). At engine open time, RotationLog state should be rehydrated from durable storage so the post-restart rotation-aware-resolve seam answers correctly on the first call. Likewise, `resolve_did_for_cid` round-trip (DID-keyed content-store lookup) needs end-to-end pinning against the Phase-4-Foundation persistence backend.

**Stranded benten-id rehydrate pin destinations** (each test's ignore message MUST cite §4.26):

- `crates/benten-id/tests/rotation_log_rehydrated_at_engine_open.rs`
- `crates/benten-id/tests/resolve_did_for_cid_round_trip.rs`

Estimated scope: ~100-200 LOC. Couples to §4.20 engine-builder seam.

### §4.27 plugin_did install RNG provenance grep-pins (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 + tc-2 — `plugin_did::mint` source-cite cluster). `plugin_did::mint` SHIPPED at `crates/benten-id/src/plugin_did.rs:69` using OS CSPRNG. The grep-pins that source-cite the OS-RNG path + assert no HKDF-from-user-DID derivation occurs need to be authored against the shipped surface:

- `crates/benten-id/tests/plugin_did_install_uses_os_rng_not_seed_derivation.rs` — grep-cite `plugin_did::mint` calls `Keypair::generate` (which routes to `OsRng`).
- `crates/benten-id/tests/plugin_did_install_no_hkdf_from_user_did_grep_assert.rs` — grep-assert no `hkdf` / `derive_from(user_did)` site in `plugin_did.rs`.

Estimated scope: ~50-100 LOC (grep tests; not production code).

### §4.28 Private-namespace cross-plugin delegation policy substantive arm (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 — private-namespace cluster). The single-scope cross-plugin-delegation refusal arm is structurally present at `manifest_envelope_chain_validation.rs::private_namespace_cap_across_plugins_rejected` (R4b-FP-2 closure). The companion test files cite a `private_namespace_policy::reject_cross_plugin` symbol that does NOT exist at HEAD (the surface lives under a different name); these tests need to be retargeted to the actually-shipped surface or deleted + their case folded into `manifest_envelope_chain_validation` test family. Couples to §5.5 manifest-envelope-chain-validation seam.

**Stranded private-namespace pin destinations** (each test's ignore message MUST cite §4.28):

- `crates/benten-caps/tests/private_namespace_cross_plugin_delegation_denied.rs`
- `crates/benten-caps/tests/private_namespace_scope_prefix_canonicalization.rs`

Estimated scope: ~50-100 LOC.

### §4.29 phase-3-backlog §7.3.D stale-rationale sweep at pre-tag (Phase-4-Foundation pre-tag)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-3 — ~30+ tests cite phase-3-backlog §7.3.D 'next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep'). Phase 3 SHIPPED at tag `phase-3-close` without the cited fix-pass batch firing; the cluster needs sweep-by-batch at the Phase-4-Foundation pre-tag wave. For each cited test: if production surface IS at HEAD, un-ignore + author body; otherwise retarget the cite to v1-assessment-window or this row. Belongs at the pre-tag sweep coupled with the cite-drift G26-A wave.

Estimated scope: ~300-500 LOC across body authoring + retarget messages.

### §4.31 IVM inner-kernel-read byte-equivalence arms post-SubgraphSpec round-trip (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 — `inner_kernel_read_equivalence_post_subgraph_spec_round_trip.rs` cluster, 5 arms). G23-0a + G23-0b shipped the SubgraphSpec round-trip wrapper-construction-equivalence pins; the inner-kernel-read byte-equivalence companion arms (one per canonical IVM view: capability_grants / event_dispatch / content_listing / governance_inheritance / version_current) require the materializer pipeline to expose a `materialize_inner_kernel_read` seam that produces byte-equivalent output across both the SubgraphSpec-routed walk and the legacy G15-A path-view walk. The G23-B canary's materializer surface materializes formatted output (HtmlJson / Plaintext), not raw inner-kernel-read bytes; the byte-equivalence arm couples to §4.24 (recursive materializer walk) + G15-A path-view shape preservation. Phase-4-Meta carry.

Estimated scope: ~200-400 LOC (materializer seam + 5 substantive byte-equivalence test arms).

### §4.30 Mini-review JSON schema discipline + `disposition` field uniformity (Phase-4-Foundation pre-tag)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 methodology-critic meth-r6-r1-3 + meth-r6-r1-4). Lens JSON reports + mini-review JSONs should carry a uniform top-level shape: `disposition` field + `findings[]` array + optional `orchestrator_action_summary`. The `r6-r1-pim-n-meta-sweep.json` lacks `disposition`; some R5 mini-reviews lack `orchestrator_action_summary`. Codify uniform schema in `.addl/dispatch-conventions.md` §3.6c brief-template; bring legacy artifacts in line via pre-tag sweep.

Estimated scope: ~10-30 LOC dispatch-conventions + JSON patch for `pim-n-meta-sweep.json` (orchestrator-direct).

### §4.32 `validate_schema_author_within_manifest_envelope` runtime production-wiring (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-FP-BF mini-review r6fp-bf-mr-1). R6-FP-BF landed the `validate_schema_author_within_manifest_envelope` helper at `crates/benten-platform-foundation/src/plugin_manifest.rs` along with a 4-case test pin that exercises the helper in isolation. The helper has **zero production callers** at HEAD; the call-site wiring (`schema_compile`-time consultation OR `install_plugin`-time consultation of `manifest.requires_schema_authors` against the schema's signing peer-DID) deferred to Phase-4-Meta.

**Couples to §4.19** (`accept_atrium_share` cross-peer install seam) — cross-peer schema introduction is the natural integration point for runtime author trust enforcement; v1 admin UI v0 ships with default-empty trust-list per Ben Q3 ratification (sdr-r6-r1-2 partial closure: helper-shipped, wiring-deferred).

**Acceptance:** add 2-3 production call sites (likely at `install_plugin` Step 4 consent gate + `schema_compile` boundary if the manifest scope guards a schema graph); add end-to-end test exercising real install flow + verifying rejection on untrusted-author schema; remove the helper's "Phase-4-Meta carry" caveat from rustdoc once wired.

Estimated scope: ~50-100 LOC + 2 integration test pins.

### §4.33 `module_ecosystem::install_plugin*` legacy-path deletion + test migration (Phase-4-Meta)

R6-FP-A (PR `r6/fp-1-plugin-trust` commit `2be7841`) marked the legacy `benten_platform_foundation::module_ecosystem::install_plugin` and `install_plugin_persisting_did` as `#[deprecated]` per HARD RULE 12 clause-(a) (BLOCKER-DEPRECATE rather than BLOCKER-DELETE) to avoid migrating 4 test files in the same wave. The deprecation-without-deletion has a NAMED destination — THIS ENTRY — per mini-review finding `r6fp-a-mr-6` + HARD RULE 12 clause-(b). (Originally proposed as `§4.22`; renumbered to `§4.33` at strategy-C batch reconciliation to avoid collision with the §4.22-§4.32 sequence added by Wave-BF + its mr-fix.)

**Deletion deadline:** Phase-4-Meta opening wave (pre-v1-assessment-window per CLAUDE.md #15 — the v1 platform-shippable assessment cannot tolerate two install paths with different security envelopes coexisting in the public surface).

**Migration scope (4 test files use `#![allow(deprecated)]`):**

- `crates/benten-platform-foundation/tests/plugin_content_cid_mismatch_rejected_on_receive.rs` — single arm imports `module_ecosystem::{InstallerShape, install_plugin}` (line ~17 after R6-FP-A allow-deprecated header). The CID-mismatch arm is exercised by Steps 1-2 of `plugin_lifecycle::install_plugin` (decode + verify content-CID); migration is direct (build an InstallRecord + ctx + call lifecycle path).
- `crates/benten-platform-foundation/tests/plugin_manifest_substitution_at_install_rejected.rs` — TWO arms: lines ~50 + ~165 import legacy; the substitution defense (peer-signature mismatch) is exercised at Step 4's `validate_with_clock → verify_peer_signature` in the lifecycle path. Lines 220+ already exercise the lifecycle path; lines 50-170 need migration.
- `crates/benten-platform-foundation/tests/plugin_heterogeneity_incompatible.rs` — single arm imports `module_ecosystem::{InstallerShape, install_plugin}` (line ~12). The heterogeneity check is Step 5 of the lifecycle path; migration trivially folds.
- `crates/benten-platform-foundation/tests/g24d_substantive_pipeline.rs` — multiple arms at lines ~23, ~99, ~135, ~165, ~409, ~418, ~506 import `module_ecosystem::*` (including `install_plugin_persisting_did`). This is the LARGEST migration surface; the test exercises end-to-end flows that should fold into `plugin_lifecycle::install_plugin` with caller-mint-first fixtures (helper `mint_and_insert_plugin_did` already shipped in R6-FP-A-fp at `tests/common/manifest_fixtures.rs`).

**Migration target:** `crates/benten-platform-foundation/src/plugin_lifecycle.rs::install_plugin` (11-step pipeline with full Layer-2 consent + Layer-1 cap cascade + caller-mint-first contract).

**Post-migration:** delete `module_ecosystem::install_plugin` + `install_plugin_persisting_did` + the duplicate `InstallerShape` enum (re-export the canonical one from `plugin_lifecycle`). Estimated LOC delta: -150 (legacy fns) +250 (4 test migrations) = ~+100 LOC.

### §4.34 `scripts/drift-detect.ts` output clarity for `E_INV_ITERATE_NEST_DEPTH` retained-stopgap labeling (Phase-4-Foundation pre-tag)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-FP-C mini-review r6fp-c-mr-2 closing ec-r6r1-8). The `drift-detect.ts` reachability scanner currently lists `E_INV_ITERATE_NEST_DEPTH` as `reachability: ignore` without an explicit reason tag in its output line. Reader has to cross-reference `ERROR-CATALOG.md` to learn the variant is intentionally retired-stopgap-retained (retained for forensic forward-compat per the catalog Status note). Tweak `drift-detect.ts::reportIgnored()` to emit the catalog's `reachability_reason` (or equivalent free-text annotation) alongside the variant name so the scanner output is self-describing. Couples to Wave-G G26-A doc retense if the §-numbering shifts at strategy-C batch.

**Scope:** ~10-30 LOC TypeScript tweak + a snapshot test of the reportIgnored() output if a sentinel exists. Pre-tag-sweep candidate (Wave-G can absorb if scope-cheap).

**Strategy-C batch reconciliation note:** this row was originally proposed as `§4.NEXT-ec-r6r1-8` placeholder in Wave-C; renumbered to `§4.34` at batch-merge per Wave-G §-numbering reconciliation log (Wave-C branched from `origin/main` which didn't include any §4.22+ rows at the time).

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
