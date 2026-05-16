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

### §3.5 `benten-admin-shell` webview-driven tauri-driver smoke test — CLOSED at R6-FP-E (path-a-FULL ratification 2026-05-13)

Per Ben's path-a-FULL ratification on Q-R6-3 (2026-05-13): both halves of br-r6-r1-3 land at R6-FP-E. Half (i) ships the integrator-binary scaffold + 9 Rust-level IPC dispatch E2E pins; half (ii) ships:

- Real `tauri = "2"` runtime dep on `tools/benten-admin-shell/Cargo.toml` (opt-in `tauri` cargo feature; OFF for the default workspace build to keep agent-pre-flight cost under the dispatch-conventions §3 cap=7 RAM budget; CI lane `admin-shell-e2e.yml` flips ON every run).
- Real `tauri::Builder::default().invoke_handler(...).run(...)` boot path in `src/main.rs::tauri_boot` wiring Tauri commands `dispatch_ipc`, `ipc_method_cap_bindings_command`, `admin_shell_bound_origin` against `AdminShellState::dispatch`.
- `tauri.conf.json` + `build.rs` calling `tauri_build::build()` + minimal `icons/icon.png` for the codegen pass.
- `tests/e2e_webview_smoke.rs` — fantoccini-rustls WebDriver client driving `tauri-driver` subprocess against the running binary. Asserts: (a) webview loads `webview-assets/index.html` (title round-trip), (b) Tauri command-invoke serializes a `ipc_method_cap_bindings_command` request JS→Rust→JS preserving the canonical method-cap map byte-for-byte, (c) classic `eval("1+1")` is blocked by the CSP forbidding `'unsafe-eval'`.
- New CI workflow `.github/workflows/admin-shell-e2e.yml` matrix ubuntu-latest (substantive WebDriver session via WebKit2GTK + WebKitWebDriver) + macos-latest (build-only smoke — Tauri's own tauri-driver project documents that macOS WKWebView lacks WebDriver bindings; see <https://v2.tauri.app/develop/tests/webdriver/>). Windows deferred per dispatch brief.

Closes br-r6-r1-3 MAJOR (both halves) + br-r6-r1-5 MINOR (handler payload contract — `tauri_boot::dispatch_ipc` is the integrator-side handler that maps `IpcResponse.payload` to the Tauri command return value).

### §3.6 Tauri 2.x upstream unmaintained-dep migration (v1-assessment-window security-advisory carry)

**Origin:** Phase 4-Foundation R6 phase-close R6-FP-E (2026-05-13). The path-a-FULL ratification of br-r6-r1-3 pulled `tauri = "2"` into the workspace `Cargo.lock` as an opt-in feature dep. Tauri's transitive deps include several unmaintained crates flagged informational by rustsec:

| Advisory | Crate | Upstream Tauri closure |
|---|---|---|
| RUSTSEC-2024-0370 | proc-macro-error | Tauri macro stack migration to proc-macro-error2 |
| RUSTSEC-2024-0411..0420 | gtk-rs GTK3 bindings cluster (atk / gdk / gtk / gdkx11-sys / gdk-sys / gtk-sys / gtk3-macros / atk-sys) | Tauri linux webview migration to gtk4-rs |
| RUSTSEC-2025-0075, 0080, 0081, 0098, 0100 | unic-ucd-* family | Tauri-utils urlpattern → unic-ucd successor migration |

All are **informational-only unmaintained advisories** (no exploit class). The crates land via the opt-in `tauri` feature on `benten-admin-shell` ONLY — they are NOT in the default workspace build path. Pre-existing CI lanes (workspace clippy / doc / nextest / cargo-deny base) do NOT enable the feature; only the new `admin-shell-e2e.yml` CI lane enables it.

**v1-assessment-window action:** when re-evaluating the dep tree, audit Tauri 2.x's migration progress for each cluster + tighten the `deny.toml` ignore list as upstream closures land. If Tauri ships a major-version bump that resolves the cluster, drop the corresponding `RUSTSEC-*` ignore entries.

**Tracked deny.toml ignores at `/Users/benwork/Documents/benten-engine/deny.toml` lines 75-113** (R6-FP-E commit).

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

**Tentative phase target:** Phase 4-Meta (substantive landing). NOT a v1-blocker — fixed-fixture idempotency at G23-A canary + emit-side 4-of-4 enforcement together suffice for the schema-driven-rendering substantive arm.

Per HARD RULE rule-12 BELONGS-NAMED-NOW: this entry IS the named destination + the work obligation lands NOW. G23-A wave's `parse.rs` source comments + the proptest's `#[ignore]` message cite `docs/future/phase-4-backlog.md §4.6` instead of phantom destinations like "wave-4b".

**Acceptance criteria addendum (added 2026-05-13 R6 R2 schema-language solo lens schema-lang-r6-r2-2):** the vocab-fixture coverage test pin `schema_compiler_typed_field_vocab_composes_over_12_primitives_no_extension.rs` currently exercises only 4 of the 8 declared `LabelType` variants (SchemaRoot / FieldScalar / FieldList / FieldRef). The remaining 4 (FieldObject / FieldMap / FieldEnum / FieldUnion) MUST be added to the fixture set when this row lands, alongside per-label assertions that the emit-side construction-site fires for each label and produces the corresponding vocab edges (implicit-via-recursion parent→child for objects per §2.2; KEY_TYPE+VALUE_TYPE for maps; VARIANT for enums/unions). Counts coverage gap closure as part of the same backlog row to keep the §4.6 destination self-contained.

Closes G23-A R5 mini-review BLOCKER finding `g23a-mr-1` + MAJOR finding `g23a-mr-2` + R6 R2 schema-lang-r6-r2-2 acceptance-criteria gap.

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
- **T9a forged peer-DID signature** — G24-D ships `plugin_manifest::sign_manifest` + `verify_peer_signature` + Ed25519 verifier. Substantive coverage at `crates/benten-platform-foundation/tests/g24d_substantive_pipeline.rs::full_install_pipeline_real_signatures_succeeds` + `crates/benten-platform-foundation/tests/g24d_substantive_pipeline.rs::install_pipeline_rejects_substituted_content` (both PASS at HEAD post-batch-2) + the dedicated provenance round-trip pin at `crates/benten-platform-foundation/tests/plugin_content_carries_peer_did_signature_for_provenance.rs::manifest_peer_did_signature_independent_of_install_record_user_did_signature`.
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

**TS Playwright sibling pins (added 2026-05-14 R6-FP-6 tca-r6r6-1 closure):** the corresponding TS Playwright e2e specs at `packages/admin-ui-v0/tests/e2e/` were originally named for R5 G24-E/G24-F wave-7 destinations; those waves shipped without un-ignoring + R6-FP-BF Rust-sibling sweep correctly retensed the Rust pins but missed the TS Playwright siblings. R6-FP-6 retensed all 3 TS specs' `test.skip` rationale to cite §4.22 directly:
- `packages/admin-ui-v0/tests/e2e/tauri_webdriver_bidi_acceptance.spec.ts` — Tauri 2.x shape-(c) deployment dogfood (T2 defense end-to-end via real WebKit/WebKitGTK).
- `packages/admin-ui-v0/tests/e2e/session_token_replay_across_origin_denied.spec.ts` — T2 defense 2 / origin-bound session-token replay defense.
- `packages/admin-ui-v0/tests/e2e/cross_origin_csrf_attempt_denied.spec.ts` — T2 LOAD-BEARING cross-origin POST defense end-to-end.

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

**Acceptance criteria (R6-R2 smc-r6-r2-3 narrow specificity):** including these explicit renames + retenses:
- (a) Rename `SubgraphBuilder::set_property_for_test` → `set_property` (drop `_for_test` suffix; production call sites at `crates/benten-platform-foundation/src/schema_compiler/emit.rs:338 + 364 + 393` cascade to all test call sites). The misleading `_for_test` suffix surfaces in production code paths today.
- (b) [other stale items: stale phase-3 prose, retired-wave references, dispatch-conventions cross-refs to closed Phase-3 §7.3.D].

Estimated scope: ~300-500 LOC across body authoring + retarget messages + the set_property rename cascade.

### §4.31 IVM inner-kernel-read byte-equivalence arms post-SubgraphSpec round-trip (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 test-coverage-auditor tc-1 — `inner_kernel_read_equivalence_post_subgraph_spec_round_trip.rs` cluster, 5 arms). G23-0a + G23-0b shipped the SubgraphSpec round-trip wrapper-construction-equivalence pins; the inner-kernel-read byte-equivalence companion arms (one per canonical IVM view: capability_grants / event_dispatch / content_listing / governance_inheritance / version_current) require the materializer pipeline to expose a `materialize_inner_kernel_read` seam that produces byte-equivalent output across both the SubgraphSpec-routed walk and the legacy G15-A path-view walk. The G23-B canary's materializer surface materializes formatted output (HtmlJson / Plaintext), not raw inner-kernel-read bytes; the byte-equivalence arm couples to §4.24 (recursive materializer walk) + G15-A path-view shape preservation. Phase-4-Meta carry.

Estimated scope: ~200-400 LOC (materializer seam + 5 substantive byte-equivalence test arms).

### §4.30 Mini-review JSON schema discipline + `disposition` field uniformity (CLOSED at R6-FP-3 2026-05-13)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6 R1 methodology-critic meth-r6-r1-3 + meth-r6-r1-4). Lens JSON reports + mini-review JSONs should carry a uniform top-level shape: `disposition` field + `findings[]` array + optional `orchestrator_action_summary`.

**CLOSED at R6-FP-3 (2026-05-13):** Canonical schema codified at `.addl/dispatch-conventions.md §3.6i` (NEW section). 14 legacy mini-review JSONs + 17 R6 R3 lens JSONs swept `"verdict":` → `"disposition":` inline at R6-FP-3 wave. Original §4.30 cite of `§3.6c brief-template` was a mis-cite — §3.6c is "Mirror-precedent overshoot guard", unrelated. §3.6i is the actual live-discipline destination. R6 R3 methodology-critic `meth-r6-r3-1` MAJOR triggered the closure.

Estimated scope (actuals): ~80 LOC dispatch-conventions (§3.6i) + 32 file sed (verdict→disposition) + 1 malformed-JSON inline fix at r6-r3-invariant-compromise.json line 70 (`]` → `}`).

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

### §4.35 `install_plugin` Step-9 cap-cascade atomicity gap (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 plugin-arch r2-cp-1 + L4+L7 cp-1 cap-cascade atomicity gap). `crates/benten-platform-foundation/src/plugin_lifecycle.rs::install_plugin` Step 9 iterates `cap_minter.mint_root_grant` over `manifest.requires` without batch-commit semantics. If `mint_root_grant` fails on `requires[k]` after `requires[0..k]` succeeded, those k-1 grants persist in the cap store while Step 10 (provision_private_namespace) + Step 11 (library.insert + set_active) never execute → orphan grants + no library entry + no NS.

**Acceptance criteria.** (a) New `InstallCascade` shape: collect intended grants into a pending Vec; commit ONLY after all succeed; OR (b) reorder fallible-work-first-then-commit so the observable commit point is `library.insert + set_active`. Add test pin `install_plugin_step_9_cap_mint_failure_rolls_back_prior_grants.rs` driving a real cap-minter wrapper that fails on requires[k] + asserts zero residual grants + zero library entry + ErrorCode `E_PLUGIN_INSTALL_CAP_CASCADE_FAILED` (new code or routed through existing). Couples to §4.21 partial-failure rollback semantics. ~150-250 LOC.

### §4.36 Production `ManifestEnvelopeRechecker` adapter shipment (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 sec-r6r2-1 MAJOR + R6-R2 plugin-arch r2-cp-2 + L4+L7 cp-2). At HEAD `crates/benten-engine/src/manifest_envelope_recheck.rs` ships only `NoopManifestEnvelopeRechecker` (returns `NotApplicable` for every call). The default `Engine::default` now installs `Some(Arc::new(Noop))` (post-R6-FP-A flip) so the recheck-path always fires — but the call is operationally inert because Noop admits everything. CLAUDE.md #18 Layer-3 ('CapabilityPolicy validates the chain at access-time') is structurally wired but defaults to admit-everything.

**Acceptance criteria.** Ship `ProductionManifestEnvelopeRechecker` in `benten-platform-foundation` that consults `PluginLibrary` (for the source plugin's manifest `shares` policy) + `UserDidRegistry` (for chain root verification) + `manifest_envelope_chain_validation::validate_chain_with_manifest_envelope` (for per-step audit). Wire into the default builder so production deployments get substantive Layer-3 enforcement automatically. Update threat-model T8 narrative to reflect gate-2-of-2 going LIVE. Also: add fluent `EngineBuilder::with_manifest_envelope_rechecker` setter (currently only `Engine::set_manifest_envelope_rechecker` exists post-build). Add `NotApplicable → UnresolvedDeny` variant rename + Noop semantics flip per r2-cp-2 disposition. ~200-400 LOC.

### §4.37 InstallRecord replay-defense (nonce-store seen-or-record) (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 sec-r6r2-2 MAJOR + sec-r6r1-2 R1 finding). `InstallRecord` carries a `nonce: Vec<u8>` field bound by `signing_payload` but `install_plugin` Step 3 verifies the signature without consulting any seen-nonce store. An attacker who captures a valid InstallRecord can replay it: (a) re-install after uninstall to revive a removed plugin; (b) install across multiple devices without re-consent; (c) replay after manifest update to roll back to a prior version. The Phase-3 device-attestation envelope V2 once carried a `nonce_store: Mutex<HashSet<(String, [u8; 32])>>` accept-time replay store in the (now COLLAPSE-deleted) `benten-id` Acceptor cluster; that store was deleted with the Acceptor cluster per Compromise #23 SUPERSEDED-BY-COLLAPSE and the durable replay-marker re-home is tracked as the P2/P5 unified-ceiling deliverable (DECISION-RECORD §4b F3). InstallRecord has no replay store of its own.

**Acceptance criteria.** Add `InstallRecordReplayDefense` port consulted at Step 4 + new ErrorCode `E_PLUGIN_INSTALL_RECORD_REPLAY`; model the seen-or-record store on the post-COLLAPSE unified-spine durable replay-marker mechanism (P2/P5 deliverable per DECISION-RECORD §4b F3) rather than the deleted Acceptor nonce_store. Add cross-device/re-install threat narrative to SECURITY-POSTURE.md as a numbered Compromise OR sibling note under Compromise #26. ~80-150 LOC.

### §4.38 G18-A-followup cross-browser-determinism Playwright fixture-bodies + wasm32 IndexedDB persistence (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 br-r6-r2-2 MAJOR + br-r6-r1-1 R1 finding). The phase-rename 2026-05-11 moved `phase-3-backlog §4.3` carry into Phase-4-Foundation scope but the entry was not migrated. At HEAD: `.github/workflows/cross-browser-determinism.yml` emits `::warning::...harness fixture not yet wired (G18-A-followup)` for 11 matrix-cells (all `::warning::` not failure-causing). `BrowserManifestStore::is_persistent()` + `IndexedDbBlobBackend::is_persistent()` honest-disclose `false` on wasm32. Compromise #19 + #20 in SECURITY-POSTURE.md remain PARTIALLY-CLOSED.

**Acceptance criteria.** (a) Land 11 Playwright fixture-body cells exercising real cross-browser canonical-bytes determinism (BLAKE3 + DAG-CBOR + CIDv1 + Ed25519 deterministic-signing) so a regression FAILS the matrix workflow per pim-2 §3.6b; (b) wire wasm32 IndexedDB persistence (web-sys / js-sys / wasm-bindgen-futures) so `BrowserManifestStore::is_persistent()` returns `true` on wasm32 when IDB is available; (c) update Compromises #19 + #20 to CLOSED. Phase-4-Meta v1-assessment-window target. ~500-1000 LOC.

### §4.39 WINTERTC_FORBIDDEN_APIS list extension (Phase-4-Foundation pre-tag)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 sec-r6r2-4 MINOR + sec-r6r1-10 MINOR + br-r6-r2-3 MAJOR). `crates/benten-platform-foundation/src/admin_ui_v0/mod.rs::WINTERTC_FORBIDDEN_APIS` currently lists 5 entries (`document.cookie`, `new FormData`, 3 relative-URL fetch variants). T2 defense step 2 says "No cap-tokens in JS storage" but the CI grep guard at G26-B only sweeps the 5 currently-listed surfaces.

**Acceptance criteria.** Extend the const to enumerate at minimum: `eval(`, `new Function(`, `localStorage.setItem`, `sessionStorage.setItem`, `indexedDB.open`, `XMLHttpRequest`, `window.addEventListener`, `document.addEventListener`, `navigator.cookieEnabled`, `Worker(`, `WebSocket("./` per the WinterTC Minimum Common API spec. Extend the `wintertc_forbidden_apis_present_with_canonical_entries` presence-pin test to assert ≥12 entries with explicit canonical-needles for `localStorage` + `XMLHttpRequest` + `eval(` + `new Function(`. Couples to G26-B CI guard wave staying ON-CRITICAL-PATH for v1. ~20 LOC.

### §4.40 Engine-held plugin-DID private-key compromise threat-class (T13 + Compromise) (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 sec-r6r2-6 MINOR + tmr-r6-r1-4 MAJOR R1 finding). Ratification #5 + D-4F-12 introduced engine-held plugin-DID signing keypairs (caller mints + inserts into `PluginDidStore`; engine-held private key per installed plugin under caller-mint-first contract). Threat class: (a) engine-process compromise leaks all plugin-DID private keys → attacker forges UCAN delegations as any plugin within manifest envelopes; (b) on-disk persistence threat. `admin-ui-v0-threat-model.md` §3 + SECURITY-POSTURE.md (Compromises #24 #25 wallclock-related, not engine-process-compromise) do not enumerate this.

**Acceptance criteria.** Add T13 to `admin-ui-v0-threat-model.md` enumerating engine-process-compromise threat class + defense narrative (OS-level engine binary protection; in-memory key zeroization on uninstall; key encryption at rest via OS keyring). Add corresponding numbered Compromise to SECURITY-POSTURE.md OR sibling note under #26. Then ship key-at-rest encryption via OS keyring at Phase-4-Meta. ~50-100 LOC doc work + ~200-400 LOC encryption implementation.

### §4.41 caps-grew fresh-consent gate at upgrade Step 7-extension (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 sec-r6r2-5 MINOR + tmr-r6-r1-2 MAJOR R1 finding). `admin-ui-v0-threat-model.md` documents ratification #8 (cap-change-triggered fresh consent: silent within-lineage subset; full re-consent if `requires` GREW). `install_plugin` Step 7 currently enforces version-DAG-descendant ordering but does NOT compute the `new.requires \ prior.requires` delta and demand a fresh consent for that specific delta. Test pin `plugin_upgrade_requires_caps_grew_triggers_user_consent.rs` is referenced at threat-model.md:300 but un-ignore status uncertain.

**Acceptance criteria.** (a) Add Step-7-extension at `install_plugin`: compute requires-delta; if non-empty, demand fresh consent (caller-supplied flag or new ErrorCode `E_PLUGIN_CAPS_GREW_REQUIRES_RECONSENT` propagating up to admin-UI). (b) Verify + un-ignore the test pin. (c) Update PLUGIN-MANIFEST.md §4.3 narrative once shipped. Couples to §4.21. ~50-150 LOC.

### §4.42 `validate_chain_with_manifest_envelope` wasm32 unsupported-surface defensive companion (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R3 sec-r6r3-1 + sec-r6r3-2 MINOR, dedup'd; R6-R2 sec-r6r2-3 carry). `crates/benten-caps/src/manifest_envelope_chain_validation.rs:186+` gates `validate_chain_with_manifest_envelope` behind `#[cfg(not(target_arch = "wasm32"))]`. The wasm32 build path has `PhantomData<()>` `Did` stubs (lines 34-48) so the module type-shape is wasm-buildable, but the chain-validation function itself simply does not exist on wasm32 — leaving cryptic linker errors as the failure mode if a future feature pulls this surface in. **R6-FP-3 landed a defensive doc-comment companion** (`crates/benten-caps/src/manifest_envelope_chain_validation.rs:50+` "compile_error! companion mirror" block) citing CLAUDE.md baked-in #17(b) thin-client architecture. Substantive `#[cfg(target_arch = "wasm32")]` stub function with cite-bearing `unimplemented!()` panic (or actual `compile_error!` macro on a sentinel feature flag) deferred to Phase-4-Meta when wasm32 cap-evaluation scope clarifies.

**Acceptance criteria.** Either (a) ship a wasm32 stub function with the same signature returning `ChainValidationOutcome::ChainInvalid` + an error citing CLAUDE.md #17(b); or (b) gate wasm32 build entirely with `#[cfg(target_arch = "wasm32")] compile_error!("...")` if benten-caps is determined not to compile for wasm32 at all (which the current state arguably already is — benten-graph fails to build wasm32). ~10-30 LOC.

**Sibling wasm32 sweep (benten-graph `backends/blob_backend.rs`, added 2026-05-15 per umbrella #1207 / mini-review-1237 MINOR):** the pre-existing wasm32 break has 3 sites at `crates/benten-graph/src/backends/blob_backend.rs:63/135/164`; umbrella #1207 added a 4th identical-pattern site at `crates/benten-graph/src/backends/blob_backend.rs:248` (disclosed, in an already-wasm32-broken non-wasm32 module — not a meaningful regression; note: #1207 also relocated this file from `crates/benten-graph/src/blob_backend.rs` to `crates/benten-graph/src/backends/blob_backend.rs`, so all cites use the new path). The eventual wasm32-gating fix MUST sweep all 4 sites together (not 3) — bundle with this §4.42 wave.

### §4.45 `PluginDidStore::insert` duplicate-DID defensive return — CLOSED at R6-FP-3 (2026-05-13)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R3 cap-r6-r3-1 MINOR; R6-R2 r2-cp-3 carry).

**CLOSED at R6-FP-3 (2026-05-13).** Originally deferred under "ErrorCodes mint with feature waves" convention; on re-examination per Ben's "do whatever makes sense to do now vs in 4-meta" directive (2026-05-13), the new-ErrorCode-in-fix-pass scope (~80 LOC across 4-surface mirror) was bounded enough to land inline.

**What shipped:**
- `crates/benten-id/src/plugin_did.rs::PluginDidStore::insert` signature: `pub fn insert(&mut self, handle: PluginDidHandle) -> Result<(), ErrorCode>` returning `Err(ErrorCode::PluginDidHandleDuplicate)` when the same DID is already present.
- New ErrorCode `E_PLUGIN_DID_HANDLE_DUPLICATE` minted with full 4-surface mirror: Rust enum + as_str + matches_static + ALL_CATALOG_VARIANTS + from_str + TS catalog + ERROR-CATALOG.md heading + preamble narrative reconciliation 167→168 (and 169→170 for catalog/TS retaining E_INV_ITERATE_NEST_DEPTH).
- Caller-mint-first contract production arm at `crates/benten-platform-foundation/src/module_ecosystem.rs:211` now propagates the Result via `?`.
- Test fixture at `crates/benten-platform-foundation/tests/common/manifest_fixtures.rs::mint_and_insert_plugin_did` adjusted for the new Result signature.
- Test-only `plugin_did::handle_with_did_for_test(did)` constructor (gated behind `cfg(any(test, feature = "testing"))`) lets the duplicate-rejection path be exercised directly.
- Substantive test pin at `crates/benten-id/tests/plugin_did_store_insert_duplicate_rejected.rs` (3 tests; required-features=["testing"]; pim-2 §3.6b PRODUCTION-ARM + OBSERVABLE-CONSEQUENCE + WOULD-FAIL-IF-NO-OP'd).

No further obligation.

### §4.46 `wasm-browser.yml` bundle-content audit grep semantics — DISAGREE-WITH-EXPLANATION (R6-FP-3 review)

Per HARD RULE rule-12 (R6-R3 br-r6-r3-2 MINOR; R1 br-r6-r1-4 MAJOR carry-forward). **Originally proposed as a NAMED-NOW destination; on R6-FP-3 review the orchestrator DISAGREES with the agent's recommended fix shape (the row remains as a tracking pin only).**

The R6-R3 finding proposed switching `grep -i -F -q '<sym>'` (case-insensitive fixed-string) at `.github/workflows/wasm-browser.yml:307` to `grep -E -q '\b(loro|iroh|wasmtime)\b'` (case-sensitive word-boundary regex). The orchestrator's DISAGREE rationale:

1. **The case-insensitive matching is intentional, not accidental.** The inline comment at lines 299-303 explicitly documents: *"The grep is case-insensitive to catch both `Loro` (Rust type names) and `loro_` (function manglings)."* PascalCase Rust type names appear in wasm-objdump output for non-mangled symbols (`#[no_mangle]` or extern wrappers); lowercase forms appear in standard Rust name mangling. Switching to case-sensitive would miss the PascalCase form.
2. **The substring (fixed-string) shape is intentional.** Rust mangling produces forms like `_ZN4loro8internal...` where the symbol's CRATE name is embedded as a length-prefixed segment without surrounding word boundaries. `\bloro` regex matches at `_ZN4|loro` (digit→letter boundary) which DOES work, but the substring form has been load-bearing through Phase-3 + Phase-4-Foundation with zero false-positive incidents — the proposed refinement is a theoretical improvement, not an empirical defect closure.
3. **False-positive risk is hypothetical at this scope.** The forbidden symbols `loro` / `iroh` / `redb` / `wasmtime` are crate names; word-collisions in legitimate symbols (e.g. a symbol containing the substring "loro" as a legit suffix) have not been observed across multiple Phase-3 + Phase-4-Foundation builds.

**Status: ROW PRESERVED for tracking** — if a future empirical false-positive surfaces, this row carries the refinement obligation. No work at HEAD; no Phase-4-Meta target; no v1-blocker.

### §4.47 `admin_ui_v0_canonical_manifest()` production constructor — DISAGREE-WITH-EXPLANATION (R6-FP-3 re-examination)

Per HARD RULE rule-12 (R6-R3 br-r6-r3-3 MINOR; R1 br-r6-r1-8 MINOR carry-forward). **Originally proposed as a NAMED-NOW destination; on R6-FP-3 re-examination the orchestrator DISAGREES — the production constructor already exists at a different location than the agent searched.**

The R6 R3 finding claimed "no production `admin_ui_v0_canonical_manifest()` constructor in `crates/benten-platform-foundation/src/admin_ui_v0/mod.rs`; only `admin_ui_v0_manifest()` in tests/common/." Verification at HEAD `6e10aea`:

- **`admin_ui_v0_canonical_manifest()` DOES exist in production** at `tools/benten-admin-shell/src/lib.rs::admin_ui_v0_canonical_manifest`. The function's doc-comment explicitly cites the R1 closure: *"Closes the secondary half of `br-r6-r1-8` MINOR: 'No production `admin_ui_v0_manifest()` constructor in benten-platform-foundation' — the integrator binary is the named NOW destination + per-test drift is asserted by `tests/canonical_manifest_matches_ipc_binding`."*
- The companion `ADMIN_UI_V0_CANONICAL_CAPS: &[&str]` constant at `tools/benten-admin-shell/src/lib.rs::ADMIN_UI_V0_CANONICAL_CAPS` enumerates the canonical 6-cap set the IPC method-cap-binding map references.
- The R1 br-r6-r1-8 finding was closed by placing the constructor at the integrator-binary (admin shell) layer rather than the platform-foundation layer — that's where admin-UI-v0-specific code belongs (per CLAUDE.md baked-in #18: admin UI v0 IS a plugin, not engine infrastructure).

The R6 R3 finding mis-searched the location. Row preserved as a tracking pin in case the production canonical-manifest constructor genuinely belongs in `benten-platform-foundation` for non-Tauri integrator consumers — but at HEAD that consumer category doesn't exist (browser-shape b consumers don't install plugins; embedded webview shape c uses the Tauri integrator's constructor).

**Status: ROW PRESERVED for tracking** — no work at HEAD; no Phase-4-Meta target; no v1-blocker.

### §4.48 `cite-drift-detector` historical-vs-current phrasing precision (Phase-4-Foundation pre-tag OR Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R3 r6r3-r7-1 MINOR). The cite-drift detector's token parser doesn't distinguish historical-narrative phrasing ("…retired Phase-2a id…" / "…earlier doc drafts cited…") from current-code construction-site claims, producing false-positive findings on doc bodies that reference deprecated/retired surfaces in historical context. Currently mitigated via the allowlist logic at `tools/cite-drift-detector/src/lib.rs` excluding `docs/history/` floor; refinement to the parser itself (e.g. "this token preceded by `(historical)` marker / `(retired)` marker / past-tense verb cluster") is the long-term fix.

**Acceptance criteria.** Either (a) extend parser to recognize historical-narrative markers, OR (b) sentinel re-baseline with §6.7 narrative documenting the false-positive classes + accepting them as known-noise. ~30-80 LOC. Couples to §6.7 sentinel re-baseline.

### §4.43 Class-B β engine-internal API cluster: visibility tighten (`pub` → `pub(crate)`) per CLAUDE.md #18 baked-in intent (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R3 cag-r6-r3-1 MINOR + R6-R4 cag-r6-r4-1 MAJOR widening). CLAUDE.md baked-in #18 prose originally named `Engine::read_node(cid)` — `pub(crate)`, no permission check — as the engine-internal un-attributed read pathway. **R6-R4 cag-r6-r4-1 surfaced that the visibility drift applies to a CLUSTER of four engine-internal functions, not just one:**

1. **`Engine::get_node`** at `crates/benten-engine/src/engine_crud.rs::get_node` — `pub`; primary un-attributed read pathway (was originally intended `pub(crate)` per #18).
2. **`Engine::put_node`** at `crates/benten-engine/src/engine_wait.rs::put_node` — `pub`; doc-comment says 'plugin authors never call this directly'; bypasses `CapabilityPolicy::check_write` so an external caller using this directly escapes the Inv-13 firing matrix.
3. **`Engine::get_node_label_only`** at `crates/benten-engine/src/engine_wait.rs::get_node_label_only` — `pub`; un-attributed read of label-only (lower-bandwidth than full Node but same attribution-bypass shape).
4. **`Engine::resolve_subgraph_cid_for_test`** at `crates/benten-engine/src/engine_wait.rs::resolve_subgraph_cid_for_test` — `pub`; `_for_test` suffix but production-consumed (DX hazard; same shape as `SubgraphBuilder::set_property_for_test` per R6-R4 cag-r6-r4-2).

**Acceptance criteria (widened at R6-FP-4).** Two paths apply atomically to the whole 4-function cluster:
- **Path (a) — tighten visibility:** rename + change visibility (`get_node` → `read_node` `pub(crate)`; `put_node` → `pub(crate)`; `get_node_label_only` → `pub(crate)`; `resolve_subgraph_cid_for_test` → `pub(crate)` + drop `_for_test` suffix or replace with proper test-feature-gate); verify napi-rs surfaces don't break (`benten-napi` reaches into engine's public API; some of these are napi-exposed today).
- **Path (b) — preserve shipped surface + update prose:** CLAUDE.md #18 prose updated to match shipped surface at R6-FP-3; sibling rustdoc surfaces updated to clarify "intentionally `pub` for napi binding usage; engine-internal discipline applies."

Path (a) is the bake-in-intent path; path (b) preserves the shipped surface. v1-API-stabilization decision. **~50-200 LOC depending on path** (widened scope from R6-R3's ~20-100 LOC single-function estimate). Sibling carry to §4.19 (orchestrator's earlier brief mis-cited that destination).

R6-FP-4 partial: SECURITY-POSTURE.md plugin-trust threat-model narrative + CLAUDE.md #18 prose now match shipped surface for `Engine::get_node`; rest of cluster narratives retain the original framing pending v1-API-stabilization decision.

**v1-API-stabilization sweep sub-bullets (refinement-audit-2026-05 umbrella #1207, benten-graph slice).** This sweep also adjudicates the following benten-graph trait/struct SemVer-locking decisions (each detailed at its own §4.x row; surface ALL at the sweep brief so a later phase does not silently lock a surface):

- **`KVBackend` async-shape fork** — Path A (lock sync at v1 + `AsyncKVBackend` post-v1) vs. Path B (convert to RPITIT pre-v1). See §4.63. Couples to `NetworkFetchStubBackend` retirement (hyg-1 #305 Ben-call).
- **`GraphBackend::snapshot()` + `register_subscriber()` Result-shape forks** — see §4.61 (SemVer-commitment docstring already landed; flip-vs-freeze decision pending). Couples safe-1 #501.
- **`BlobBackend` additive-default vs. sub-trait-split** — see §4.62 (additive defaults already landed; lock-vs-split decision pending).
- **`#[non_exhaustive]` struct-level discipline residual (Fwd-2 #997 partial).** Umbrella #1207 applied `#[non_exhaustive]` to **3 of 4** `GraphError` struct-variants — `BackendNotFound` / `SystemZoneWrite` / `InvImmutability` — safe because all their construction is in-crate and cross-crate matchers already use `{ .. }`. **`TxAborted` was deliberately EXCLUDED**: `GraphError::TxAborted { reason }` is constructed in cross-crate **production** code (`crates/benten-engine/src/engine_diagnostics.rs:87,105`), so a bare `#[non_exhaustive]` breaks the workspace build. The v1-stabilization wave decides: apply `#[non_exhaustive]` + add a `GraphError::tx_aborted(reason)` constructor + migrate the 2 benten-engine production sites (~10 LOC), OR accept the field-level lock with a written rationale. **`WriteContext` (struct, 3 `pub` fields) + `ChangeEvent` (struct, 9 `pub` fields) struct-level `#[non_exhaustive]` was NOT applied** at umbrella #1207: both are struct-literal-constructed in cross-crate **test** code (`crates/benten-engine/tests/system_zone_stopgap_and_full_coexist.rs` for `WriteContext`; `crates/benten-ivm/tests/*` for `benten_graph::ChangeEvent`-shaped events), so a bare struct-level `#[non_exhaustive]` would break the workspace test build (cascade beyond benten-graph — surfaced not chased per HARD RULE clause-(b)). The v1-stabilization wave decides per-struct: (a) apply `#[non_exhaustive]` + migrate the cross-crate test literals to the existing `WriteContext::new` / `ChangeEvent::new_node` constructors + add any missing `with_*` builders; (b) accept the field-level SemVer-lock with a written rationale; (c) collapse attribution fields (`actor_cid`/`handler_cid`/`capability_grant_cid`) to a typed `attribution: Option<Attribution>` substruct. ~50-150 LOC of test-literal migration if (a). Fwd-2 #997.

### §4.49 webview-e2e CI lane — actual root cause surfaced; MUST-FIX-OR-EXPLICITLY-ACCEPT-AT-TAG (R6-FP-5 sharpening)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R4 br-r6-r4-1 MINOR + R6-R5 br-r6-r5-1 MAJOR). The R6-FP-3 `br-r6-r3-1` Stdio::inherit-instead-of-null fix at `tools/benten-admin-shell/tests/e2e_webview_smoke.rs` SERVED ITS PURPOSE: it surfaced the previously-invisible actionable root cause via the next CI run. **The webview-e2e failure was NEVER a race condition** — it is an old test bug at `tools/benten-admin-shell/tests/e2e_webview_smoke.rs:159-171`.

**Actual root cause (R6 R5 br-r6-r5-1 finding):** the test invokes `tauri-driver --port 4444 --native-binary <path>` but the installed `tauri-driver` CLI **does NOT support `--native-binary`** — actual supported flags are `--port / --native-port / --native-host / --native-driver` per tauri-driver source. The test as-shipped CANNOT pass; the native-binary launch path must instead be specified via WebDriver session `tauri:options.application` capability (passed by fantoccini's `Capabilities`), OR via env var that `tauri-driver` recognizes.

**Why not fixed inline at R6-FP-5:** the fix requires understanding tauri-driver's capability-passing API + WebKitGTK runtime setup; can't be validated locally without a Linux runner + tauri-driver installed. Speculative fixes would risk shipping a still-broken test. Better to name properly + decide at tag time.

**Acceptance criteria (R6-FP-5 sharpening + tag-time decision).** Path (a) FIX: ~50-150 LOC test-harness rewrite: drop `--native-binary` from `Command::new("tauri-driver")` args; pass binary path via `fantoccini::Capabilities` with `tauri:options.application` key; verify the new shape against an actual tauri-driver subprocess on Linux + WebKitGTK. Path (b) EXPLICITLY-ACCEPT-AT-TAG: phase-4-foundation-close tag ships with webview-e2e ubuntu RED known-bug-non-required (matches the §4.46 / §4.47 DISAGREE-WITH-EXPLANATION precedent); the production integrator binary at `tools/benten-admin-shell/src/lib.rs` is UNAFFECTED — the bug is purely in E2E test subprocess invocation. **Decision deferred to phase-4-foundation-close pre-tag review.**

**Not v1-gate-blocker** because: (a) admin-shell-e2e.yml documented non-required for merge; (b) production code path unaffected (Tauri integrator binary works correctly when launched directly); (c) test infrastructure issue, not platform-shippable defect.

### §4.50 `Engine::*` `_for_test` suffix in production-consumed APIs cleanup (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R4 cag-r6-r4-2 MINOR). `SubgraphBuilder::set_property_for_test` retains `_for_test` suffix despite production consumption by `schema_compiler::emit`. DX hazard — engineers reading the symbol expect test-only scope; production use signals confusion about the API's stability contract.

**Acceptance criteria.** Audit all `_for_test` suffixed functions for production consumption; rename or split (proper test-feature-gate per `crates/benten-id/Cargo.toml` testing-feature precedent) any that escape test scope. Sibling carry to §4.43 v1-API-stabilization sweep — bundle the rename + visibility-tighten + test-feature-gate work into one wave.

### §4.51 `CRATES-DEEP-DIVE.md` napi-rs binding-vs-workspace-member distinction (Phase-4-Foundation pre-tag housekeeping)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R4 arch-r6-r4-2 MINOR). `docs/CRATES-DEEP-DIVE.md` mentions `benten-napi` adjacent to the 12-crate workspace-member list without explicitly clarifying that `benten-napi` is a workspace member crate at `bindings/napi/`, not part of the "12 production-shipped crates" count narrative. Functionally correct; readers can resolve from context; inline clarification at pre-tag sweep would tighten the prose.

**Acceptance criteria.** ~3-5 LOC inline rewording at `docs/CRATES-DEEP-DIVE.md §1` clarifying that benten-napi is a Node.js binding workspace member, not a separately-counted crate in the 12-crate production stack narrative. Couples to G26-A pre-tag docs retense wave.

### §4.44 `tests/phase_3_workspace/architecture_md_g20b_final.rs` rename (10-crate→12-crate; Phase-4-Foundation pre-tag housekeeping)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R3 arch-r6-r3-3 MINOR). The Phase-3 R3-E RED-PHASE pin `tests/phase_3_workspace/architecture_md_g20b_final.rs` carries "10-crate FINAL" in test fn name + module-level comments. The actual assertion is a subset-check that still passes against the 12-crate doc state (no regression). Superseded by `crates/benten-engine/tests/architecture_md_12_crate_count_post_phase_4_foundation_canaries.rs` (Phase-4-Foundation R6-FP canary). R6-FP-3 inline closure: module-level doc-comment now carries "HISTORICAL ANCHOR (Phase-3 R3-E origin)" framing + names the post-Phase-4-Foundation canary as authoritative successor. File rename is a non-time-pressured housekeeping retense.

**Acceptance criteria.** Rename file + test fn from "10-crate" to "12-crate" (or retire the Phase-3 R3-E pin entirely if the post-Phase-4-Foundation canary fully supersedes). ~5-10 LOC. Sibling carry to G26-A pre-tag docs retense wave.

### §4.52 `VocabEdge::from_str` error-shape refinement (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R5 schema-lang-r6-r5-3 MINOR). `crates/benten-platform-foundation/src/schema_compiler/vocab.rs::VocabEdge::from_str` fallback uses `SchemaCompileError::VocabEdgeMismatch` (intended for source/target-label-pair mismatch) for the "unknown edge token" case with sentinel `"<unknown>"` source/target labels. Minor naming-drift; refactor either splits a new variant (e.g. `UnknownEdgeLabel`) or mints a new ErrorCode (`E_SCHEMA_VOCAB_UNKNOWN_EDGE_TOKEN`).

**Acceptance criteria.** Either (a) split a new error variant in `schema_compiler/error.rs` with proper "unknown edge token" semantics, OR (b) mint a new ErrorCode (bumps catalog math). ~10-30 LOC depending on path. Bundle with §4.43 v1-API-stabilization sweep if it lands first.

### §4.53 Vocabulary-cardinality cross-doc completeness pim-N candidate (Phase-4-Meta DEFER 1-instance)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R5 schema-lang-r6-r5-1 pim-N candidate, DEFER). pim-N candidate proposed by schema-language-reviewer at R6 R5: "when phase-close-fix-pass commits CLAIM an N-site sweep is STUCK, next-round lens MUST run exhaustive grep for the obsoleted term-form (don't trust the claimed count)." 1-instance at HEAD (R6-FP-4 sdr-r6-r4-1 claimed 11-site sweep STUCK but R6 R5 found 3 residuals); below 3+-recurrence threshold per Ben's deferral precedent for 1-instance candidates.

**Acceptance criteria.** Re-evaluate at Phase-4-Meta on 2nd/3rd recurrence; if hits 3+-threshold, codify as §3.6j extension to §3.6h (sibling: §3.6h covers "rule-codification names origin instance(s) that must close"; this candidate covers "fix-pass claims N-site sweep that must verify all N sites"). Composes with existing pim-1 §3.5b HARDENED + pim-18 §3.6f.

### §4.54 "Restore Stdio inheritance before assuming env-flake" pim-N candidate (Phase-4-Meta DEFER 1-instance)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R5 br-r6-r5-1 pim-N candidate, DEFER). pim-N candidate proposed by browser-runtime-reviewer at R6 R5: when a test fails inscrutably + Stdio::null was set "for clean test output," the FIRST debugging step is to restore Stdio::inherit + re-run before assuming the failure is environmental flakiness. R6-FP-3 demonstrated this: the original webview-e2e port-binding "race" was misdiagnosed because Stdio::null hid the actual `--native-binary` CLI flag error. 1-instance at HEAD; below 3+-recurrence threshold.

**Acceptance criteria.** Re-evaluate at Phase-4-Meta on 2nd/3rd recurrence; if hits 3+-threshold, codify as testing-discipline addition (sibling: pim-18 §3.6f SHAPE-not-SUBSTANCE — this candidate is debugging-discipline analog: "before claiming env-flake, restore observability"). Composes with existing pim-18.

### §4.55 Storage-mutating host-fn banned-list consolidation across 3 defense surfaces (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R6 cag-r6-r6-1 MINOR). Three independent defense surfaces encode the "no storage-mutating SANDBOX host-fn names" rule with drifted vocabularies:

1. **Canonical regression pin** at `crates/benten-eval/tests/host_fn_no_storage_mutating_per_baked_in_16.rs::FORBIDDEN_HOST_FN_NAMES` — 9 names: `kv:write` / `kv:delete` / `kv:append` / `edge:create` / `edge:delete` / `edge:update` / `transaction:begin` / `transaction:commit` / `transaction:abort`.
2. **Schema-compiler parse-time reject** at `crates/benten-platform-foundation/src/schema_compiler/parse.rs:132-139::FORBIDDEN_HOST_FNS` — 6 names: `kv:write` / `kv:delete` / `edges:add` / `edges:remove` / `graph:write` / `graph:delete`.
3. **Materializer defense-in-depth re-check** at `crates/benten-platform-foundation/src/materializer.rs:909` (inline `banned` array; not a named const) — 4 names: `kv:write` / `kv:delete` / `edges:add` / `edges:remove`.

The three lists share only 2 names (`kv:write`, `kv:delete`). Naming-convention disagreement (`edge:*` singular vs `edges:*` plural; `transaction:*` vs `graph:*`; `kv:append` present at canonical only). Architecturally HARMLESS at HEAD because the registration-check at `default_host_fns()` (4 entries only) catches any unregistered name — the named banned-lists are belt-AND-suspenders for clearer error-message-text on specific common-offender names. The MINOR-not-MAJOR severity flows from the registration-check floor defense + the unitary semantic enforcement across surfaces (every storage-mutation attempt is rejected; the drift is in *which named tokens get the specific better error message*, not in *what gets rejected*).

**Acceptance criteria.** ~30-50 LOC consolidation:
- (a) Define `pub const STORAGE_MUTATING_HOST_FN_NAMES: &[&str]` in `benten-eval::sandbox` covering the superset (~12 unique names after naming-convention normalization decision).
- (b) Re-export through `benten_eval::sandbox` so `benten-platform-foundation::schema_compiler::parse` + `benten-platform-foundation::materializer` import the canonical const instead of carrying parallel lists.
- (c) Update the canonical regression-pin (`host_fn_no_storage_mutating_per_baked_in_16.rs`) to import the const directly (drop the local copy).
- (d) Verify all 3 historical defense-test files (`materializer_defense_in_depth_rejects_banned_sandbox_host_fn_for_handcoded_spec.rs` + `schema_compiler_rejects_schema_referencing_sandbox_with_storage_mutating_host_fn_request.rs` + `materializer_rejects_subgraph_with_unregistered_sandbox_host_fn.rs`) continue to PASS unchanged.
- (e) Naming-convention decision: standardize on the canonical-pin's `edge:*` singular + `transaction:*` (the broader vocabulary), OR on schema_compiler's `edges:*` plural + `graph:*` (the production-code precedent). Bundling note: aligns with §4.43 v1-API-stabilization sweep — combine into one wave if both land in the same phase.

**Not v1-gate-blocker** because: (a) the registration-check at `default_host_fns()` is the floor defense and catches ANY name not in the 4-entry registered set; (b) the named banned-lists provide better error-message-text for specific common offenders but are not the enforcement surface; (c) CLAUDE.md baked-in #16 architectural commitment is honored end-to-end at HEAD via the three-surface defense-in-depth regardless of which named tokens are explicitly enumerated.

### §4.56 `Renderer::render()` no-op stub production caller (Phase-4-Meta / Phase-5)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R6 mat-r6-r6-minor-2 MINOR). The `Renderer` trait at `crates/benten-platform-foundation/src/materializer.rs::Renderer` declares `fn render(&self, output: &MaterializerOutput) -> Result<(), RenderError>` + both production impls (`BrowserRender` + `TauriRenderer`) ship the body as a no-op stub at HEAD. No production caller invokes `Renderer::render(...)` at HEAD — the materializer pipeline drives side-effects through other paths.

**Acceptance criteria.** Decide between two paths:
- Path (a) — **un-stub at admin UI v0 DOM-mount**: `BrowserRender::render` writes the materialized HTML into the canonical DOM root; `TauriRenderer::render` pushes via Tauri IPC. Requires admin UI v0 → renderer-output wiring at Phase-4-Meta admin-UI-meta-circular work.
- Path (b) — **refactor trait to accessor pattern**: drop `render(&self, ...)` from the trait + replace with accessor methods returning the materialized bytes for the caller to dispatch via owned transport. Less coupling; admin UI calls the accessor in its DOM-update event loop.

v1-API-stabilization decision. Bundle with §4.43 v1-API-stabilization sweep + §4.50 `_for_test` suffix cleanup if all 3 land in the same Phase-4-Meta wave.

### §4.57 Sweep-completeness self-verify discipline pim-N candidate (CLOSED at R6-FP-7 — promoted to §3.6j)

**Status:** CLOSED at R6-FP-7 (2026-05-14). 4th-instance recurrence at R6 R7 fired the watch-list promotion criterion; codified as `.addl/dispatch-conventions.md §3.6j` per Path A INTENT (pattern-induction recommendation). Origin instances (r6r7-r7-1 / r6r7-meth-1 / doc-r6-r7-1 / r6r7-pi-1 / arch-r6-r7-1 — 5-lens cross-confirmation of the 4-JSON disposition gap) closed at the same R6-FP-7 wave; §3.6h "already-closed-before-ratification STRICTLY STRONGER" preserved.

4-instance trajectory across the Phase-4-Foundation R6 cycle:
- R6-FP-3 §3.6i verdict→disposition sweep claimed 32-file complete but R6-FP-4 found 2 R4 residuals.
- R6-FP-4 doc-cite 11-site sweep claimed complete but R6 R5 found 3 residuals (sdr-r6-r4-1).
- R6-FP-5 §3.6i 49-JSON sweep claimed complete but R6 R6 found 4 JSONs lacking top-level disposition (meth-r6-r6-1 + r6r6-pi-1).
- R6-FP-6 commit body claimed "79/79 R6 JSONs §3.6i conformant" but R6 R7 found the same 4 R6 R6 lens JSONs still lacking top-level `disposition`.

See `.addl/dispatch-conventions.md §3.6j` for the ratified rule + brief-template mandate (output JSON authors mandate canonical top-level `disposition` at author-time to eliminate the orchestrator-catchup cycle).

### §4.58 Sync-attack 18-vector fabric — 15-of-18 missing (Phase-4-Meta / v1-platform-shippable; refinement-audit #1100)

**Origin:** refinement-audit-2026-05 X10 compromise-registry cross-crate reviewer (#1100). `docs/history/PHASE-3.md:29` previously claimed Phase-3 R2 pinned "the 18 attack vectors as named adversarial fixtures (sync-attack-1..18 in `crates/benten-sync/tests/`)". At HEAD `8141b94` only **3 attack-test files exist** — `attack_hlc_skew_revocation_ordering.rs`, `attack_loro_op_log_inv_13.rs`, `attack_mst_diff_cid_mismatch.rs` (5 `#[test]` functions total). PHASE-3.md:29 retensed (this PR) to name the 3 files that exist + point here for the deferred 15.

**Deferred work.** The remaining 15 sync-attack vectors specified in the archived Phase-3 R2 test-landscape (`.addl/_archive/phase-3/r2-test-landscape.md`) were never landed. They compose with Compromise #22 (peer-DID + connection-metadata leakage) / #23 (wire device-attestation envelope V2) / #25 (HLC-monotonic enforcement at sync layer) / #26 (manifest-envelope recheck at sync merge) — i.e. those four Compromises' CLOSED narratives lean on a "sync-attack test family" that is 3-of-18 wide at HEAD. Narrative-named classes include sync-attack-1 (peer-impersonation), sync-attack-7 (signature-substitution), sync-attack-11 (audience-rebinding), and 12 more.

**Acceptance criteria.** Land the 15 missing adversarial fixtures per the R2-specified envelope construction + expected ErrorCode + file names, OR explicitly down-scope Compromise #22/#23/#25/#26 closure narratives in `docs/SECURITY-POSTURE.md` to match the 3-vector reality and re-scope the 15 as a named v1-assessment-window item. Ben architectural call on scope vs. down-scope. ~per-vector signed-envelope construction; estimated substantial (each vector is a full adversarial-peer integration scenario). Bundle with the META #684 / META #660 v1-platform-shippable BLOCKER cluster if it lands in the same window.

### §4.59 Runtime contract at the Tauri integrator boundary (Phase-4-Meta; Ben architectural decision; refinement-audit #1110)

**Origin:** refinement-audit-2026-05 X3 async-runtime cross-crate reviewer (#1110), sub-shape (b). `benten-renderer-tauri` is intentionally Tauri-runtime-agnostic (no tauri/tokio dep — the crate holds the IPC protocol + cap-binding + CSP only, per CLAUDE.md #19). The integrator binary (`tools/benten-admin-shell`, Phase-4-Meta) that wires `tauri::Builder` against the engine will be the workspace's 4th tokio-runtime-construction site, and the runtime-sharing contract has **no named destination at HEAD** (`docs/ARCHITECTURE.md` does not name tokio; CLAUDE.md #17 names Tauri-as-shape-(c) but not the runtime contract).

**Decision needed (Ben architectural call) BEFORE Phase-4-Meta admin-shell wiring lands:**
- **(1) Shared runtime context** — engine constructs the tokio runtime; Tauri attaches via `tauri::Builder::with_runtime(...)` (or the Tauri 2.x equivalent). Single runtime; simpler; risk = Tauri's internal runtime-ownership expectations.
- **(2) Bridged dual runtime** — engine + Tauri each spawn their own runtime; communication crosses via channels. More isolation; better for the verso-swap-readiness goal; risk = bridging overhead + lifecycle coupling.

Couples to #1101 (X3 workspace-tier architectural-commitment-to-tokio pim-N candidate) — that finding covers the workspace-tier discipline; this row is the per-crate Tauri-boundary manifestation. Sub-shape (a) of #1110 (the `Cargo.toml description` runtime-agnosticism disclaimer) is FIX-NOW and landed in this PR.

### §4.60 `GraphBackend::Transaction` `run<F, R>` composability surface (Phase-4-Meta — Surf-1 #836)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Surf-1 #836, umbrella #1207). `GraphBackend::Transaction` + `transaction()` are an intentional **shape-lock-only marker** today — a generic `<B: GraphBackend>` caller obtains the handle but the handle drives nothing; batched transactional writes require dropping down to the per-backend inherent closure-based `RedbBackend::transaction(|tx| ...)`. The marker docstring at `crates/benten-graph/src/graph_backend.rs` now **explicitly** names this gap (closed inline at this PR per HARD RULE clause-(b) — the marker is the right shape-lock per `D-PHASE-3-1` RESOLVED; pulling the method now would be substantive work without a concrete second-backend driver).

**Acceptance criteria.** When a second backend needs transactional composability (Phase-4-Meta or beyond), promote `Self::Transaction` to a real surface: add a `run<F, R>(self, f: F) -> Result<R, <Self as GraphBackend>::Error>` method delegating to the inherent closure-based path; each `*TransactionRunner` gains the driver. ~200-400 LOC across the trait + 3 impls + `benten-engine` transaction plumbing. Bundle with §4.43 v1-API-stabilization sweep if it lands first.

### §4.61 `GraphBackend` umbrella SemVer-locking shape forks (Phase-4-Meta — Fwd-2 #1022 + safe-1 #501)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 #1022, umbrella #1207). The trait now carries a written **SemVer commitment** docstring (closed inline at this PR): (1) the trait remains non-object-safe post-v1 (removing any of `Snapshot`/`Error`/`Transaction` is a forbidden major break); (2)+(3) two shape forks are surfaced-not-resolved:

- **`snapshot()` infallible-with-`expect`** vs. `Result<Self::Snapshot, Self::Error>`. Locking infallible at v1 means a Phase-4-Meta light-client `mode-(c)` signed-checkpoint backend whose snapshot can legitimately fail at open-time (cryptographic verification) has no clean path. Couples to safe-1 #501 (`.expect()` panic surface).
- **`register_subscriber()` returns `()`** vs. `Result<(), Self::Error>`. Locking `()` at v1 means a future failure-surfacing subscriber (quota guard, duplicate-registration guard) has no trait path.

**Acceptance criteria.** Pre-v1-stabilization decision per fork: (a) freeze current shape + the SemVer-commitment docstring (already landed) is the v1 record; (b) flip `snapshot()` and/or `register_subscriber()` to `Result<_, Self::Error>` pre-v1 (last chance before SemVer-lock) — ~50-150 LOC + every consumer site threads `?`. Same wave as §4.43 + §4.63.

### §4.62 `BlobBackend` additive-default vs. sub-trait-split fork (Phase-4-Meta — Fwd-2 #1012)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 #1012, umbrella #1207). `BlobBackend::delete` + `list_cids` now land as **additive default methods** (closed inline at this PR — `delete` = idempotent `Ok(())` no-op; `list_cids` = `Ok(Vec::new())`; `RedbBlobBackend` overrides both with real impls; the `StubBlobBackend` test proves existing impls compile unchanged). This exercises the documented additive-default evolution posture ahead of v1-SemVer-lock so the evolution-cost is known (it is: zero — additive default methods).

**Acceptance criteria.** v1-stabilization decision: (a) keep additive-default posture + lock it as the written v1 commitment (recommended — the cost is now known to be zero); OR (b) split `BlobBackend` (get/put/is_persistent) + `MutableBlobBackend: BlobBackend` (adds delete) + `EnumerableBlobBackend: BlobBackend` (adds list_cids) if a thin-client backend must NOT advertise mutate/enumerate capability at the type level. ~50-150 LOC if (b). Bundle with §4.43.

### §4.63 `KVBackend` sync-only vs. RPITIT-async v1-stabilization fork (Phase-4-Meta — Surf-1 #855 + Fwd-2 #986)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Surf-1 #855 + Fwd-2 #986, umbrella #1207). `KVBackend` is sync-only (`fn get(&self, key) -> Result<...>`); `BlobBackend` is async via RPITIT (`impl Future + Send`). The asymmetry is LIVE in production: a future post-redb async-native backend (cloud-KV: DynamoDB/Cosmos/etcd per CLAUDE.md #19) cannot satisfy sync `KVBackend` without a runtime-blocking shim that re-introduces tokio into `benten-graph` (defying the no-tokio-dep architect decision at `lib.rs`). `NetworkFetchStubBackend` is the in-tree witness reserving the wrong trait shape for a Phase-3 iroh-fetch that cannot fit it. At v1-tag the public `KVBackend` shape SemVer-locks.

**Acceptance criteria.** v1-stabilization fork (couples to §4.43 sub-bullet "`KVBackend` async-shape"):
- **Path A (lock sync at v1):** ship `AsyncKVBackend` parallel trait post-v1 (non-breaking minor); network-fetch lives off-trait via inherent async until then. Optionally retire `NetworkFetchStubBackend` (removes the false promise — couples hyg-1 #305 Ben-call).
- **Path B (convert to RPITIT pre-v1):** last chance before SemVer-lock; native impls use `core::future::ready(...)` shim like `RedbBlobBackend`. ~200 LOC + every `KVBackend` impl + consumer site.

Surface the fork at the §4.43 v1-API-stabilization sweep brief so a later phase does not silently lock the sync surface.

### §4.64 Light-client mode-(b) range-query-proof + mode-(c) signed-checkpoint trait-destination decision (Phase-4-Meta — Fwd-2 #1004)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 #1004, umbrella #1207). The Phase-3-deferred-to-Phase-4-Meta light-client primitives have **zero trait surface** in `benten-graph` today:

1. **mode-(b) range-query proof.** `RedbBackend::get_by_property` returns `Vec<Cid>` with no Merkle proof; `LABEL_INDEX_TABLE`/`PROP_INDEX_TABLE` are redb multimaps (no Merkle tree, no range-query attestation). A thin-compute-surface client (CLAUDE.md #17 shape b) asking "all `Post` nodes with `category=Foo`" gets no verifiable answer.
2. **mode-(c) signed checkpoint.** `SnapshotBlob` payload carries `schema_version + anchor_cid + nodes + system_zone_index` — no `committed_by: PeerDid`, no `signed_checkpoint: Signature`, no `hlc_clock`. CID-of-bytes is the only authentication; a recipient cannot validate "is THIS blob from the peer I asked."

**Acceptance criteria.** The decision is forward-only — surface is **where**, not **what**. Decide at the §4.43 v1-API-stabilization sweep (before it locks `benten-graph`'s surface area): (a) extend `SnapshotBlob.schema_version: 1 → 2` for the mode-(c) checkpoint signature field (the existing `SchemaVersion` strict-mismatch error variant IS the backward-compatible migration path) + add a `MerkleRangeProofBackend` trait alongside `KVBackend` for mode-(b) IN `benten-graph`; OR (b) land both ABOVE the storage layer in `benten-sync` (native-only sync runtime + `benten-id` PeerDid signing) keeping the storage trait narrow — preferred per the foundational `engine_primitives_vs_application_layer` memory. ~no LOC now; ~300-600 LOC at Phase-4-Meta depending on (a)/(b).

### §4.65 `Connection.transport_kind` dynamic-refresh wiring (Phase-4-Meta / v1-assessment-window — Safe-3 #603, umbrella #1181)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Safe-3 #603, umbrella #1181). The pre-v1 closure landed **Path B** (honest disclosure): `crates/benten-sync/src/transport.rs` docstrings + the three over-promising inline comments (`Endpoint.status` "background relay-fallback task", `from_iroh_parts` "background relay-handshake task will refine", `Connection::transport_kind` accessor) now correctly state that `Connection.kind` is the **establishment-time** path classification and is NOT refreshed when iroh holepunch upgrades Relay→Direct mid-connection or a NAT rebind degrades Direct→Relay. `TransportKind`'s type-level docs carry the full Compromise #22 metadata-leakage establishment-time-accuracy disclosure.

**Deferred Path A (this row).** Wire the real dynamic-refresh task: iroh exposes `Connection::watch_conn_type()` (iroh 0.98) which yields a stream of `ConnectionType::{Relay, Direct, Mixed}` transitions. Spawn a per-`Connection` tokio task that updates the kind (requires `Connection.kind` → `Arc<AtomicU8>` or `Arc<Mutex<TransportKind>>`) + propagates to `Endpoint.status`. ~40-60 LOC + a test pin asserting the kind flips when the iroh stream emits a transition. This is genuine net-new background-task wiring with behavioral surface — a v1-assessment-window candidate, not a pre-v1 doc fix. Bundle with the Compromise #22 metadata-leakage observability re-assessment if that lands in the same window.

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

Per cross-lens doc-engineer findings: `benten-platform-foundation/INTERNALS.md` (NEW; 11th workspace crate), `benten-renderer-tauri/INTERNALS.md` (NEW; 12th workspace crate), updates to `benten-ivm/INTERNALS.md` (post IVM-subgraph generalization), `benten-engine/INTERNALS.md` (post audience-binding + actor_cid wiring + SUBSCRIBE-cap-recheck closure), `benten-caps/INTERNALS.md` (post Q5 plugin-DID-keyed signing-key infrastructure).

### §6.4 ManifestStore redb-durable persistence (Phase-4-Meta)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 r7-r6-r1-2 PLUGIN-MANIFEST.md drift closure: doc retensed to describe v1 in-memory reality + named here as Phase-4-Meta carry). `crates/benten-platform-foundation/src/manifest_store.rs` ships an in-memory `HashMap<Cid, InstallRecord>`-backed store at Phase-4-Foundation v1. The redb-durable wiring (share storage with GrantStore per cap-r1-15) is deferred to Phase-4-Meta — the in-memory shape preserves the seam contract so the swap is a transparent backend lift.

**Acceptance criteria.** Wire `RedbManifestStore` in `benten-platform-foundation` sharing storage with `GrantStore` (cap-r1-15). Add migration path for in-memory-state-at-restart scenarios. Update PLUGIN-MANIFEST.md §4.1 narrative once shipped. ~150-300 LOC.

### §6.5 §3.6h "rule-ratification-against-drift mandatory-close" pim-N candidate (RATIFIED at R6-FP-3; sharpened at R6-FP-4 + R6-FP-5)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 pim-n-r7-spec finding r6r2-new-1; 3+ recurrence threshold MET). When a new pim-N or §-codification names specific instance(s) as origin, the same PR/wave that lands the rule MUST close (or explicitly defer-with-named-destination) the originating instance(s). Otherwise the rule's ratification is decoupled from its origin — engineers reading the rule + accepting it as "live" while the origin remains unfixed produces a credibility gap.

**Three recurrence instances at R6 R2 (3+-recurrence threshold met):**
1. Phase-4-Foundation R6 R1 Q3 §3.5g #3 type-name cross-doc mirror: ratified naming `TauriRender` vs `TauriRenderer` 8-cite drift as origin; 7 cite sites unfixed at HEAD post-ratification (closed at R6-FP-2 this row).
2. Phase-4-Foundation R6 R1 §3.6g prior-pim-explicit-preflight: ratified with 5-instance Phase-4-Foundation R3-R5 recurrence table as origin; rule does NOT carry follow-on action to re-dispatch the affected briefs / sweep already-shipped wave residuals.
3. Phase-3 / 2b precedent §3.5h MANDATORY-PRE-MERGE: ratified 2026-05-09 R6 R6-final naming 4 specific cite-drift fix-pass instances as origin; 2 of 4 closed inline; 2 already-closed before ratification (borderline counterexample).

**Status:** RATIFIED at R6-FP-3 (Ben Q1 ratification 2026-05-13). Codified at `.addl/dispatch-conventions.md §3.6h` with "already-closed-before-ratification STRICTLY STRONGER" sharpening at R6-FP-4 + forward-fire-only exemption parenthetical at R6-FP-5. Memory at `feedback_pim_n_ratification_must_close_origin.md`. Section retained for historical traceability of the 3-instance recurrence trigger.

**Sibling pim-N candidates from R6-R2 methodology lens (RATIFIED — see status notes on each row below):**
- **pim-r6-fp-stable-clippy-cycle-1** — **RATIFIED at §3.5j (Ben 2026-05-13)**: `cargo +stable clippy --workspace --all-targets -- -D warnings` added to §3.5h workspace pre-push gate IN ADDITION to existing MSRV 1.95 clippy. Codified in `.addl/dispatch-conventions.md §3.5j` + `feedback_pim_n_stable_clippy_gate.md` memory + MEMORY.md index. 4-instance recurrence on PR #240 R6-FP batch was the trigger.
- **pim-r6-fp-cargo-audit-mirror-2** — **RATIFIED as §3.5g item #4 extension (Ben 2026-05-13)**: same-language dual-config rule-mirrors (deny.toml ↔ CI cargo-audit --ignore flags; rustfmt.toml ↔ .editorconfig; clippy.toml ↔ Cargo.toml [lints]) MUST atomically update. Codified as §3.5g item #4 + MEMORY.md cross-ref to existing `feedback_pim_cross_language_rule_mirror.md`. Wave-E's deny.toml-only RUSTSEC ignore (missing audit.toml mirror) + missing RUSTSEC-2024-0429 glib advisory was the trigger.
- **pim-r6-fp-rustls-bootstrap-3** (added 2026-05-13 R6-R2 methodology solo lens solo-4 sharpening): **DEFERRED — held for more data per Ben ratification 2026-05-13.** Feature-flag-gated transitive deps that require runtime initialization (e.g. rustls 0.23+ `CryptoProvider::install_default()`) MUST be exercised by a build-only smoke OR a feature-test-pin at the SAME wave that ships the feature flag. R6-FP-E shipped `tools/benten-admin-shell/` with `fantoccini` (rustls-tls feature) but missed installing the rustls `CryptoProvider`; first CI cycle on PR #240's webview-e2e workflow surfaced `Could not automatically determine the process-level CryptoProvider`; closure at commit `911f486`. 1-instance recurrence — weak; re-evaluate at Phase-4-Meta or next phase-close if pattern recurs.

### §6.6 caps-grew gate seam tracking (cross-ref to §4.41)

Tracking entry for PLUGIN-MANIFEST.md §4.3 cross-reference. Substantive content lives at §4.41 above. This entry exists so PLUGIN-MANIFEST.md's cross-reference resolves to a §6.x doc-retense surface (the gate's appearance in the narrative is a doc-coupling sub-task of the substantive engineering).

### §6.7 Sentinel-test re-baseline note (R6-FP-2 closure of r6-r2-tc-1)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (R6-R2 r6-r2-tc-1 MAJOR). The cite-drift sentinel test `cite_drift_detector_finds_zero_drift_on_clean_main_post_g13_pre_a` was RED at HEAD `85ecb69` with 316 findings (266 `.addl/_archive*` + 48 `.addl` planning-artifact-frozen-in-time + 1 docs/ERROR-CATALOG.md publish_walk symbol drift + 1 dispatch-conventions historical narrative cite). R6-FP-2 closed by: (a) excluding `.addl/_archive*`, `.addl/_archive-extraction`, `docs/history/` from `walk_doc_inputs` (frozen historical artifacts; cites are accurate-for-phase); (b) narrowing `.addl/` to ONLY `dispatch-conventions.md` (the live standing-rules catalog); (c) fixing `docs/ERROR-CATALOG.md:1087` `publish_walk` → `publish_change_event_with_labels` symbol cite; (d) escaping historical-narrative cites in `.addl/dispatch-conventions.md` that referred to past-recurrence-site line numbers. Sentinel test now PASSES at HEAD.

No further acceptance criteria — entry exists to canonicalize the R6-FP-2 closure shape for forward retrospective traceability.

### §6.3 phase-4-backlog.md §-numbering reconciliation for strategy-C batch

Wave-G (R6-FP-G) consolidates §-numbering for the strategy-C batch reconciler. At the time R6-FP-G shipped, the §-numbering state across the 6 R6-FP branches was:

| Wave | §-additions on its branch |
|---|---|
| Wave-A (`r6/fp-1-plugin-trust`) | added `§4.22` (caller-mint-first deprecation seam destination) |
| Wave-BF (`r6/fp-2-schema-and-tests`) | added `§4.22`-`§4.31` (10 entries; OVERLAP with Wave-A at 22) |
| Wave-BF mr-fix | added `§4.32` |
| Wave-C (`r6/fp-4-catalog`) | added `§4.NEXT-ec-r6r1-8` (provisional placeholder) |
| Wave-D (`r6/fp-5-plugin-library-graph`) | unknown — likely no backlog additions (substantive code work) |
| Wave-E (`r6/fp-3-admin-shell`) | added `§3.6` (RUSTSEC ignore migrations) |
| Wave-G (`r6/fp-6-doc-retense`, THIS branch) | added `§6.3` (THIS entry) — note: an earlier intra-wave draft also added a `§6.3 admin_ui_v0 missing_docs escape-hatch retire (Phase-4-Meta)` row but that deferral was reversed inline at mini-review (Ben ratified path-a-now); the escape hatch was closed at this same commit, so no backlog row is needed. |

**Reconciler instructions for strategy-C batch.** Wave-A's `§4.22` + Wave-BF's `§4.22`-`§4.31` OVERLAP at 22. Proposed resolution: renumber Wave-A's `§4.22` to `§4.32` + shift Wave-BF mr-fix's `§4.32` to `§4.33`; Wave-C's `§4.NEXT-ec-r6r1-8` placeholder takes `§4.34`. Final ordering after batch merge: `§4.22`-`§4.31` (Wave-BF block) → `§4.32` (Wave-A, renumbered) → `§4.33` (Wave-BF mr-fix, shifted) → `§4.34` (Wave-C, definite from placeholder) → `§6.3` (Wave-G). The strategy-C batch reconciler updates cross-refs in all 6 branches' final commit messages atomically as the merge lands. Wave-E's `§3.6` lives in a separate top-level section (RUSTSEC migrations) so it doesn't collide with the `§4.x` block.

---

(Section structure additive; entries land as Phase 4-Foundation work surfaces them.)
