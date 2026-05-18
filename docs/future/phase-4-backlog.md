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

**Registry-trait reconsideration (Fwd-2 #1014 RATIFIED Path A, 2026-05-15):** the Phase-4-Foundation `benten-platform-foundation::registry` module originally shipped a paper-only `trait Registry { publish; discover }` with zero v1 consumers. Per Ben's Path A ratification (`docs/future/refinement-audit-2026-05.md §15.3`: "match docs to code — retract the layering claim, document the actual concrete shape") the trait was **deleted** at the ST-PF fix-pass; only the concrete data shapes (`RegistryEntry`, `DiscoveryQuery`, `DiscoveryResult`) + the reserved `E_REGISTRY_DISCOVERY_TIMEOUT` ErrorCode anchor (`timeout_error_code()`) remain. **Phase-4-Meta task:** when the Atrium-substrate publish/subscribe wiring lands, decide the registry surface shape *against the actual Phase-8 trajectory* rather than the retracted v1 abstraction — specifically: (a) **CID-keyed announce** (publish CID only; subscribers Atrium-pull on demand) rather than the retracted publish-of-full-body shape; (b) `DiscoveryQuery` as `#[non_exhaustive]` (or an open shape) to admit Phase-8 variants (`ByTrustGraph` / `BySchemaAuthorOverlap` / `ByPeerDistance` / `ByPluginCategory`) without a SemVer break; (c) a richer error vocabulary (`E_REGISTRY_NO_RELAY_AVAILABLE` / `E_REGISTRY_TRUST_GRAPH_EMPTY` / `E_REGISTRY_PUBLISHER_DENYLISTED` / `E_REGISTRY_DISCOVERY_BUDGET_EXHAUSTED`) beyond the single reserved timeout code; (d) introduce a `trait Registry` abstraction *only if* a genuine second impl materializes (e.g. an Atrium-substrate registry whose shape differs materially from an in-memory one) — a one-impl trait is accidental coupling per arch-r1-10.

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

**benten-caps v1-API-stabilization cluster (refinement-audit-2026-05 umbrella #1159; surfaced by `fix/v1-api-caps-canonical`, decision-queue #3).** The PR that made `engine.caps()` the sole canonical capability-mutation surface (#820/#1195) + renamed `benten_caps::WriteContext` → `benten_caps::CapWriteContext` (#885) + landed the mechanical #1159 items (#998 `TypedCapGroup` `#[non_exhaustive]`; #884 crate-root re-exports; #883(a) doc-lie retraction; #887(a) `check_read` footgun doc) ALSO surfaces the following non-mechanical v1-API-stabilization arch forks NAMED-NOW here per HARD RULE clause-(b) — each remains tracked on its issue + decided in the v1-API-stabilization window, NOT silently dropped:

- **#886 — `[features]` section for `benten-caps`.** Crate has no `[features]` section; durable UCAN + plugin-trust composition is exclusively `cfg(not(target_arch = "wasm32"))`-gated with no native opt-out. Fork: which features (`ucan` / `plugin-trust` / both / neither / `default = [...]`)? No concrete consumer pays a cost today (all deployment shapes a/b/c want the full composition on native). Ben-decision in the v1-API-freeze window; couples to the workspace feature-flag substrate commitment (ratification #10) + META #924.
- **#993 — `CapabilityPolicy` sealed-extension discipline.** Trait has 4 methods at HEAD; Phase-4-Meta wants 3+ new hooks (install-time consent, per-delegation runtime hook, audience-aware `check_write`). Default-impl additions are sound but the extensibility *contract* (sealed vs explicit-extensibility) is named nowhere. Fork: sealed-trait pattern vs documented explicit-extensibility contract. v1-API-freeze decision; META #907/#1094.
- **#1005 — `CapWriteContext::actor_hint: Option<String>` Phase-1-shaped principal resolution.** `UcanGroundedPolicy::principal_did_from_context` does a did:key-prefix scan; v1-API-freeze locks the `String` shape. Couples to Kith (Phase-5+) non-did:key methods + the cap-r1-16 `actor_cid` lift deliverable. Fork: keep `actor_hint: Option<String>` vs lift to a typed principal-resolution shape pre-v1. Decided alongside cap-r1-16.
- **#1017 — coordinated v1-assessment-window pre-tag-sweep across benten-caps.** Three coupled cleanup classes (6 `TODO(phase-3)` markers + `LegacyUcanStubBackend` retention + FP-N narrative tense) — partial cleanup creates a half-stale narrative worse than the fully-stale one. Must be swept as one coordinated pass at the v1-assessment-window pre-tag sweep, not piecemeal.
- **#883(b) — retire the `benten-caps → benten-platform-foundation` prod-dep edge entirely.** This PR landed #883(a) (the doc-lie retraction — the doc-comment at `manifest_envelope_chain_validation.rs` no longer falsely claims "MUST NOT depend on benten-platform-foundation in production"). Option (b) — push the blanket `impl SharesPolicyView for benten_platform_foundation::SharesPolicy` into `benten-platform-foundation` itself + require all callers to pass `SharesPolicyView` via the trait abstraction (mirroring `engine`'s `ManifestEnvelopeRechecker` port discipline) — is a ≥6-callsite arch refactor. Fork: keep the prod edge (doc now honest) vs retire it for layering symmetry. META #923 (paper-only layering).
- **#887(b) — remove `CapabilityPolicy::check_read` default-impl.** This PR landed #887(a) (the footgun WARNING doc on the default-permit-everything `check_read`). Option (b) — delete the default-impl so every `CapabilityPolicy` impl MUST explicitly opt into a read posture — is a breaking trait-shape change (cascade: `NoAuthBackend` / `LegacyUcanStubBackend` + ~5 test fixtures need explicit `Ok(())` impls). Couples to named-compromise #2's post-identity-surface disposition. Breaking; v1-API-freeze decision.

### §4.43.1 benten-id v1-API-stabilization slice (refinement-audit-2026-05 umbrella #1169, Surf-1 #830 + #835 + #850 + #858)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (clause-(b)): this row IS the concrete landed destination for the four `benten-id` Surf-1 v1-API-stabilization forks. In-source docstrings at `crates/benten-id/src/{device_attestation.rs,ucan.rs,keypair.rs,grant_reader.rs,did.rs,multi_sig.rs}` cite `§4.43` + umbrella `#1169` for these; this sub-row makes the destination self-contained so those cites are not phantom. The four forks were ALSO surfaced for Ben adjudication in `.addl/refinement-audit-2026-05/MORNING-BRIEF-forks-2026-05-17.md` (POST-SUPERSESSION ADDENDUM) — they have NO ratified disposition and are decide-don't-implement (the D1 #1172/benten-id drain lane took zero code action on them; deciding delete-vs-keep IS the fork).

1. **#830 — `benten_id::grant_reader::GrantReader` sibling trait paper-only.** Zero cross-crate production consumers at HEAD (verified: only in-crate + the `InMemoryReader` test fixture). Fork: delete the paper-only sibling trait vs. keep + lock as the written v1 extension-point commitment (forecloses the PR-#199 fail-OPEN class at the trait surface; the §13.11 structural lesson it closes is real). META #746 (forward-arch-shape with no consumer). Resolve at the §4.43 v1-API-stabilization sweep.
2. **#835 — `Did` dual unvalidated wire-deserialize boundary.** The #555 log-injection half is CLOSED (refinement-audit #1172 PR — `UcanError` Display sanitized). The remaining fork — should `<Did as Deserialize>` validate (call `Did::resolve`) — is a **P-III wire-format decision** (`benten_sync::handshake_wire::HandshakeFrame` carries `Did` over the wire per net-blocker-4; validate-on-deserialize changes on-the-wire accept/reject). P-III ⇒ Ben-only, never autonomous. Likely shape: keep structurally-trusting Deserialize + delete/`pub(crate)`-gate `from_string_unchecked` (ONE unvalidated boundary, not two; the signature gate is the load-bearing assertion; the issue filed deser-side validation at DISAGREE). META #650 (asymmetric-defense). Resolve at the §4.43 sweep, P-III-gated.
3. **#850 — Ed25519/did:key hardcoded crypto-baseline + `MultiSigSurface` cross-crate extensibility.** "Zero impls" sub-claim is STALE (2 in-crate impls: `Ed25519SingleKey` + `ThresholdMultiSig`); live fork is the crypto-suite-permanence question (accept Ed25519/did:key as the permanent v1 baseline — same class as RATIFIED #1033 "baked-in #5 permanently settles BLAKE3/DAG-CBOR/CIDv1" — vs. surface a crypto-agility seam pre-v1) + retense the `MultiSigSurface` doc to stop implying cross-crate extensibility that has zero cross-crate impls. META #746. Resolve at the §4.43 sweep.
4. **#858 — `RuntimeTarget` buried + missing shape-(c) variant.** Two coupled sub-forks: **(a)** lift `RuntimeTarget` to a workspace-canonical location — this is a CROSS-CRATE cascade ⇒ **BELONGS-TO-P-II-WORKSPACE-SWEEP** (mechanical relocation, batch when COLLAPSE frees the consuming crates; NOT the disjoint benten-id lane); **(b)** add a shape-(c) embedded-webview variant per CLAUDE.md #17 — a genuine v1-deployment-model enum-shape decision, Phase-4-Meta v1-API-stabilization window (no v1 consumer pays a cost today; couples the Tauri shape-(c) work). META #758 (forward-shape).

**Acceptance criteria.** Resolved at the §4.43 v1-API-stabilization sweep as part of the coordinated cross-crate batch. Surface forks 1/2/3/4(b) for Ben ratification before §4.43 locks the affected `benten-id` public surfaces; fork 2 is additionally P-III-gated (wire-format). Fork 4(a) rides the P-II post-COLLAPSE workspace relocation sweep. ~varies (trait delete-or-keep #830; `from_string_unchecked` removal cascade ~90+ callsites #835; doc-retense + baseline-ratification #850; enum-relocation + variant-add #858).

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

### §4.65 wasmtime core-wasm vs. Component-Model + `host:async` reservation v1-stabilization fork (Phase-4-Meta — Fwd-2 #1027, umbrella #1166)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 #1027, umbrella #1166). `Cargo.toml` pins wasmtime `43.0.2` with `component-model` OFF (wsa-3) and `async` ON-but-unused. `benten-eval::sandbox::host_fns::RESERVED_HOST_ASYNC_CAP` + the `requires_async: bool` field are declared-not-wired; Phase 3 + Phase 4-Foundation both SHIPPED without flipping `requires_async = true`. The in-source trajectory framings were retensed (Phase-3 → "through Phase 4-Foundation; keep/wire/retire is a Phase-4-Meta ratification") at umbrella #1166's ST-EVAL lane PR — the **substantive architectural decision** lands here.

**Acceptance criteria (Ben architectural call, before any Phase-4-Meta SANDBOX-extensibility work):**
- **(A) Keep core-wasm + reserved-cap indefinitely** — cheapest; preserves future-compat; `async` Cargo feature stays cold weight.
- **(B) Wire the first async host-fn** — flip `requires_async`; the original D19 framing named an iroh-backed `kv:read`. Couples to CLAUDE.md #18 plugin-as-subgraph compute trajectory + #1016 `register_runtime` lift.
- **(C) Retire the reservation** as a v1-API-stabilization no-op (drop `RESERVED_HOST_ASYNC_CAP` + `requires_async` field; ErrorCode/Cargo-feature trim). Couples to #1035 reading (α).

Bundle the decision with §4.43 v1-API-stabilization sweep. wasmtime Component-Model re-evaluation (already named at §4 line 107) is the sibling axis — enumerate its acceptance criteria at the same sweep.

### §4.66 SANDBOX host-fn extensibility — closed-set (α) vs. plugin-extensible (β) v1 ratification (Phase-4-Meta — Fwd-2 #1035, umbrella #1166)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 #1035, umbrella #1166). `host_fns.rs::HOST_FN_NAMES = &["time","log","kv:read","random"]` + the `host_fn_no_storage_mutating_per_baked_in_16.rs` regression pin structurally implement reading (α) (permanently-closed engine host-fn set). CLAUDE.md baked-in #16 forbids `kv:write`/`kv:delete` by name but does **not** enumerate the broader question of whether a plugin author at Phase 5+ may declare plugin-internal SANDBOX host-fns (`host:plugin:<cid>:<action>` shape) gated by per-plugin manifest+DID. OBS severity — no code change at HEAD; the trajectory commitment is unenumerated and blocks substantive CLAUDE.md #19 plugin-extensibility planning.

**Acceptance criteria (Ben ratification — state explicitly in a CLAUDE.md #16 update OR a new commitment #20):**
- **(α) Permanently closed:** the four host-fns are the ENGINE set forever; plugins needing more compute use raw wasm without host bridges. Lock the regression pin as a v1-API-stabilization commitment; couples to §4.65 path (C) (retire `host:async`).
- **(β) Plugin-extensible at Phase 5+:** the four are the ENGINE baseline; PLUGIN-shipped host-fns are a separate Phase-4-Meta+ surface gated by per-plugin manifest (#18 install-time consent + per-plugin DID). Wire the existing `RuntimeRegistrationDeferred` typed-error to a `plugin-host-fn-registration-deferred` companion (couples to #1016 `register_runtime` re-eval).

### §4.67 `Connection.transport_kind` dynamic-refresh wiring (Phase-4-Meta / v1-assessment-window — Safe-3 #603, umbrella #1181)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Safe-3 #603, umbrella #1181). The pre-v1 closure landed **Path B** (honest disclosure): `crates/benten-sync/src/transport.rs` docstrings + the three over-promising inline comments (`Endpoint.status` "background relay-fallback task", `from_iroh_parts` "background relay-handshake task will refine", `Connection::transport_kind` accessor) now correctly state that `Connection.kind` is the **establishment-time** path classification and is NOT refreshed when iroh holepunch upgrades Relay→Direct mid-connection or a NAT rebind degrades Direct→Relay. `TransportKind`'s type-level docs carry the full Compromise #22 metadata-leakage establishment-time-accuracy disclosure.

**Deferred Path A (this row).** Wire the real dynamic-refresh task: iroh exposes `Connection::watch_conn_type()` (iroh 0.98) which yields a stream of `ConnectionType::{Relay, Direct, Mixed}` transitions. Spawn a per-`Connection` tokio task that updates the kind (requires `Connection.kind` → `Arc<AtomicU8>` or `Arc<Mutex<TransportKind>>`) + propagates to `Endpoint.status`. ~40-60 LOC + a test pin asserting the kind flips when the iroh stream emits a transition. This is genuine net-new background-task wiring with behavioral surface — a v1-assessment-window candidate, not a pre-v1 doc fix. Bundle with the Compromise #22 metadata-leakage observability re-assessment if that lands in the same window.

### §4.68 benten-engine in-source `TODO(phase-3 — …)` cluster retarget (Phase-4-Meta — Surf-2 SF2-07 / refinement-audit #1193, META #476)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Surf-2 SF2-07 cluster, umbrella #1193; META #476 stale-phase-marker workspace sweep). The 2026-05-11 phase-rename (Phase 3.5 → Phase 4-Foundation; Phase 4 → Phase 4-Meta) left `benten-engine` source carrying 5 forward-looking `TODO(phase-3 — …)` markers whose substantive work explicitly did NOT land in Phase 3 (each marker self-describes the deferral). They are retargeted in-source to `TODO(phase-4-meta — backlog §4.68 …)` and enumerated here as named destinations (no behavior change; comment edits only):

1. **`outcome.rs` `NestedTx` sub-transaction state.** `NestedTx` is a Phase-1 stub; Phase-1/2/3 always reject nested `begin`. Phase-4-Meta populates it with real sub-transaction state (origin finding R-nit-07) IF the self-composing admin meta-circular work requires nested-tx semantics; otherwise the stub stays and this row closes as "intentionally-unimplemented — DAG-only transaction model, see CLAUDE.md #4/#6".
2. **`engine_transaction.rs` `EngineTransaction` ↔ `benten_graph::Transaction` unification.** Arch-7 flagged the dual-shape redundancy (`EngineTransaction` carries `ops_collector` for the capability hook; pure-graph `Transaction` doesn't). Decision is forward-only — at the §4.43 v1-API-stabilization sweep decide: promote `ops_collector` into `benten_graph::Transaction` OR push the capability binding up into the engine closure wrapper. Both additive; current shape is correct as-is.
3. **`builder.rs` `PolicyKind` napi-bridgeable enum.** `capability_policy(Box<dyn CapabilityPolicy>)` is not JS-serializable. Phase-4-Meta wraps the surface in a `PolicyKind` enum (`NoAuth | GrantBacked | Ucan(..) | Custom(Box<dyn ..>)`) with the native-only `Custom` arm `#[cfg(not(target_arch = "wasm32"))]`-gated (origin code-reviewer finding `g7-cr-3`). Couples to the §4.63 `KVBackend` v1-stabilization fork.
4. **`builder.rs` content-listing auto-registration unification.** arch-6 flagged "post" auto-registration + `register_crud` auto-registration as two paths both materialising `content_listing_<label>` views. Phase-4-Meta collapses to a single on-demand `register_content_listing(label)` builder entry point.
5. **`system_zones.rs` system-zone phf-table codegen.** `SYSTEM_ZONE_PREFIXES` is a hand-maintained ordered slice; Phase-2a G5-B-i intended a build-time phf-table codegen that didn't land. Phase-4-Meta lands the codegen IF the prefix set grows enough to make linear scan a hot-path concern; otherwise closes as "slice is correct for the current small fixed set — codegen is premature optimization".

**Acceptance criteria.** Surface is *where + decision-shape*, not *what*. Each row resolves at Phase-4-Meta or the §4.43 v1-API-stabilization sweep. ~no LOC now; ~50-300 LOC at Phase-4-Meta depending on which rows take the implement-vs-close-as-intentional path.

### §4.69 Capability-mutation surface organization: `EngineCapsHandle` vs `Engine`-direct asymmetry (Phase-4-Meta / §4.43 v1-API-stabilization — Qual-2 #820 / refinement-audit #1195)

Per HARD RULE rule-12 DISAGREE-WITH-EXPLANATION + BELONGS-NAMED-NOW (refinement-audit-2026-05 Qual-2 #820, umbrella #1195). The capability-grant mutation surface is split: 2 pass-through methods (`install_proof` / `revoke`) on `EngineCapsHandle` (returned by `Engine::caps()`, rustdoc'd as "the production-equivalent grant-mutation handle") vs 8 methods on `Engine` directly — including the most-recently-landed `revoke_capability_by_grant_cid` (PR #199) and `delegate_capability` (Phase-4-Foundation, which IS on the handle at `engine_caps.rs:498` — the decision rule is unstable).

**Why this is NOT executed in the #1195 mechanical bundle (DISAGREE-WITH-EXPLANATION on standalone execution):**

1. **Public capability-mutation API is the most security-critical crate surface.** Relocating load-bearing methods between public surfaces (`Engine` ↔ `EngineCapsHandle`) is a v1-API-stabilization decision, not a mechanical alias-deletion. The issue itself names `docs/future/phase-4-backlog.md §4.43` (v1-API-stabilization sweep) as the destination and lists "Blocked by: Orchestrator confirmation of overlap with U05" as a hard dependency.
2. **Cross-lane (napi) coupling.** `bindings/napi/src/lib.rs::Engine::revoke_capability_by_grant_cid` + `bindings/napi/src/lib.rs::Engine::delegate_capability` call `Engine::revoke_capability_by_grant_cid` + `Engine::delegate_capability` directly. Moving them onto `EngineCapsHandle` changes the napi binding signature — an out-of-lane (`bindings/napi`) cascade the benten-engine lane cannot land unilaterally.
3. **U05 (#834) is a `bindings-napi` Surf-1 umbrella (napi_surface::Engine 50-method cohesion failure), NOT a benten-engine facet-handle split.** The #1195 body's "U05 part-1 owns CapsHandle migration" framing must be reconciled by the orchestrator: the napi-side facet split and the engine-side handle organization are sibling-but-distinct surfaces. Whichever lands first sets the canonical organizing principle.

**The forced architectural choice (surfaced for Ben/orchestrator ratification at §4.43):** either (a) `EngineCapsHandle` IS the canonical production grant-mutation surface → migrate `revoke_capability_by_grant_cid` + `install_ucan_proof` + `grant_capability*` + `create_principal` onto it (8→0 on Engine direct; napi rebinds through `.caps()`); OR (b) `Engine`-direct IS canonical → `install_proof` / `revoke` on `EngineCapsHandle` are redundant CLAUDE.md-#5-class aliases and the handle is deleted. Option (b) is the simpler CLAUDE.md-#5-consistent end state but loses the sec-r4r1-2 RED-PHASE-pin consumer framing; option (a) is the larger refactor. ~0 LOC now; ~150-250 LOC at the chosen option, plus the coupled napi-lane rebind.

### §4.70 ChangeBroadcast / patterned-subscriber fan-out prefilter (Phase-4-Meta — Fwd-2 #1038 / refinement-audit #1194)

Per HARD RULE rule-12 BELONGS-NAMED-NOW + OUT-OF-LANE-for-ST-ENGINE (refinement-audit-2026-05 Fwd-2 #1038, umbrella #1194). Forward-readiness: per-change-event subscriber fan-out is O(N) with no pattern prefilter; compounds at Phase-4-Meta self-composing admin (N panels × M writes/sec) + Phase-5+ AI-agent workloads.

**Why this is NOT executed in the #1194 benten-engine lane (OUT-OF-LANE + DISAGREE-WITH-EXPLANATION on the issue's lane attribution):** the issue names three locations but the meaningful prefilter cannot live in any in-lane (`crates/benten-engine/`) surface:

- `benten-engine::change.rs::ChangeBroadcast` (in-lane) stores `Vec<Arc<dyn Fn(&ChangeEvent)>>` with **zero pattern info** — its subscribers are IVM closures registered via `subscribe_fn`, not the patterned `on_change` path. There is nothing in-lane to prefilter ON.
- `benten-engine::engine_subscribe.rs::register_on_change_internal` (in-lane) translates the string pattern to `ChangePattern` then delegates registration + the per-event match-walk to `benten_eval::primitives::subscribe::register_on_change` — **`crates/benten-eval/`, out-of-lane**.
- The actual O(N) "for each entry whose pattern matches the event's anchor label" walk is in `crates/benten-eval/src/primitives/subscribe.rs` (~L938-947) — out-of-lane. `ChangePattern` validation/registry semantics there are wire-adjacent.

A prefix-trie prefilter (the issue's sketch option (a)) must be built where the patterns are held: the `benten-eval` subscriber registry + its publish loop. Threading patterns down into the in-lane `ChangeBroadcast` would require a cross-crate `subscribe_fn` signature change AND the `benten-eval` registry rework — outside the ST-ENGINE single-crate lane.

**Acceptance criteria (Phase-4-Meta, primarily `benten-eval` lane).** Add a `LabelGlob`/`AnchorPrefix` prefix-trie prefilter to the `benten_eval::primitives::subscribe` registry consulted before iterating subscribers; full-glob/arbitrary patterns fall back to current "call every subscriber"; criterion bench asserts O(1)-avg vs O(N) at N=1k subscribers; no semantic change (same subscribers receive same events); INTERNALS.md (both `benten-eval` and `benten-engine` where the `engine_subscribe` delegation is documented) updated. ~150-250 LOC at Phase-4-Meta. Surface this lane-attribution correction to the orchestrator at PR-open (the umbrella #1194 should be re-pointed at a `benten-eval` lane or a cross-lane wave).

### §4.71 `benten-sync` `Cargo.toml` `[features]` table + `#[non_exhaustive]` enum sweep (Phase-4-Meta — Fwd-2 #1077 + #1072, umbrella #1186)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 #1077 + #1072, umbrella #1186 — `phase-4-meta-deferred`). This row IS the named destination for the W20-deferred umbrella #1186; the umbrella stays open as the Phase-4-Meta tracking row.

`crates/benten-sync/Cargo.toml` has no `[features]` table (5th instance of META #924 root-position crate feature-flag absence). The absence is *intentional* for the native-only Layer-1 sync crate at v1 (Surf-1 NEW-4 OBS-positive: the wasm32-native-only 5-rung defense + Cargo.toml architectural-intent narration are exemplary). The surface needs evolution at Phase-4-Meta, NOT pre-v1:

1. **`testing` feature.** `MstEntry::new_with_explicit_cid_for_testing` is a `pub fn` with `_for_testing` suffix (Qual-2 #771). Mirror the `crates/benten-id/Cargo.toml` testing-feature precedent — gate the `_for_testing` surfaces behind a `testing` feature.
2. **`tracing` feature** (default-on, off-able for embedded/wasm32). Safe-1 #511 names 10 silent-skip arms in `crdt.rs` with zero `tracing::` calls; #511 itself closes via the LoroDoc Pattern-F bundle #1179 — the `tracing` feature-gate here is *defense-in-depth*, not the #511 closure.
3. **Future post-iroh transport features.** If #889 Path A lands (umbrella #1176), individual transport backends ship as features (`iroh-transport` / `tor-transport` / `nostr-relay`).
4. **`#[non_exhaustive]` discipline (Fwd-2 #1072).** 5 public enums lack `#[non_exhaustive]` despite INTERNALS naming future-extension targets. NOTE: the design-D8 carve-out is Ben-ratified — deliberately-exhaustive enums (where exhaustive match in cross-crate consumers is the intended SemVer contract) stay closed; this row applies `#[non_exhaustive]` only to the genuinely future-extensible subset per the #1072 enumeration + couples to META #907.

**Acceptance criteria (Phase-4-Meta).** `[features]` table with `testing` + `tracing` (default-on); `_for_testing` surfaces gated behind `testing` (closes Qual-2 #771); `#[non_exhaustive]` applied to the future-extensible enum subset per #1072 (design-D8 carve-out respected); Cargo.toml architectural-intent narration retained per Surf-1 NEW-4; `cargo-public-api` baseline updated (additive — preserves SemVer). ~50-100 LOC at Phase-4-Meta.

### §4.72 `benten-sync` `Mst` cached-root wrapper (Phase-4-Meta — Fwd-1 #1011, META #486, umbrella #1184)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-1 #1011, META #486 5th instance, umbrella #1184 — `phase-4-meta-deferred`). This row IS the named destination for the W20-deferred umbrella #1184; the umbrella stays open as the Phase-4-Meta tracking row.

`Mst::root_cid()` is O(n) per call (DAG-CBOR encode + BLAKE3 hash of the full entry set) with a docstring promise that "production-callers layer a cached-root wrapper" — the cache layer was never wired. At HEAD `Engine::apply_atrium_merge` (engine crate, out-of-lane) calls `mst.root_cid()` repeatedly during convergence without caching. This is the "INTERNALS-acknowledged-but-not-delivered" pattern (META #486 5th instance in this crate: #375 + #378 + #381 + F4 + this F3 cached-root). Couples to the `apply_atrium_merge` cluster (#1181, the benten-sync slice of which is already closed on main) + LoroDoc Pattern-F #1179.

Forward-readiness rationale (Fwd-2 §C): as plugin ecosystems scale (Phase 6+ AI agents + Phase 8 decentralized registry), per-zone MST cardinality grows from "tens" to "thousands"; uncached O(n) per root-CID call becomes a real cost ceiling. Pre-v1 it is not on a hot path that materially regresses (current MST cardinalities are small + the convergence loop is already round-bounded by §4.67's MAX_ROUNDS cap).

**Acceptance criteria (Phase-4-Meta perf-pass).** Cached-root wrapper struct over `Mst` exposing a memoized `root_cid()` with invalidation on `insert`/`remove`; `Engine::apply_atrium_merge` consumes the wrapped MST (engine-side touch — cross-lane, coordinate with the engine lane at Phase-4-Meta); INTERNALS.md §6 retense to name the cached-root wrapper as the production-callers shape (closes the phantom-destination docstring); META #486 sub-instance count reduced 5 → 4 in this crate; Criterion bench baseline added (couples to META #1062). ~80-150 LOC at Phase-4-Meta.
### §4.73 benten-errors decentralized-registry-readiness cluster — ratification-gated arch decisions (Phase-4-Meta / v1-API-stabilization — Fwd-2 #996 + #1030 + #1053, umbrella #1182)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 umbrella #1182 "benten-errors: decentralized-registry-readiness cluster"). The ST-ERRORS lane (PR #1268) closed the mechanical/codification facet of #1182 (`#1019` §3.5g item #5 cohort-mint subset-closure mirror, landed in `.addl/dispatch-conventions.md` — its real canonical destination). The three remaining sub-issues are **ratification-gated architectural decisions, not mechanical residual** — each is labelled `closure-pin` + `arch-anchor`/`phase-4-meta-deferred` and each issue body explicitly states "decision sub-row before closure PR" or "N architectural paths to surface for Ben". They are deferred here (the prior PR-body citation of an `INTERNALS §10.5` section was a phantom — no such section exists; this row is the real, tracked, committed-tree destination that receives the entries NOW):

- **#996 — META #629 DecodeLimits + 6-8 ErrorCode pre-allocation; benten-errors-home-vs-benten-core decision (severity-major).** The DoS-via-unbounded-decode META #629 cluster needs a `DecodeLimits` carrier + 6-8 pre-allocated `E_DECODE_*` ErrorCode variants. Open architectural fork: do the variants + the limits type live in `benten-errors` (keeps the catalog the single denial vocabulary, but `benten-errors` is `no_std + alloc` zero-deps and a `DecodeLimits` config struct is behaviour, not just a discriminant) or `benten-core` (natural home for a limits config but splits the decode-denial vocabulary away from the catalog)? Recommend benten-errors for the ErrorCode variants (catalog stays the single source of denial strings) + benten-core for the `DecodeLimits` runtime config; surface the split for Ben at the v1-API-stabilization ratification cluster (META #1094 / #924). ~6-8 catalog mints + a limits struct; ratification-gated.

- **#1030 — plugin-registered ErrorCode extension contract missing; CLAUDE.md #18 plugin-trust forward-gap (severity-major, arch-anchor).** Plugins (subgraphs per CLAUDE.md #18) currently cannot register their own ErrorCode discriminants — a plugin denial surfaces as `Unknown(String)` with no catalog identity, breaking the "every error has a stable string the drift detector enforces both directions" contract for plugin-emitted errors. Three architectural paths to surface for Ben: (a) reserved `E_PLUGIN_<plugin-did>_<code>` namespace minted at install from the manifest; (b) plugins ship a manifest-declared error-code table validated against a reserved prefix; (c) plugin errors stay `Unknown`-shaped + a separate plugin-error-catalog surface outside `benten-errors`. Ratification-gated (touches the CLAUDE.md #18 trust model + the drift-detector both-directions invariant); Phase-4-Meta when plugin manifests gain runtime denial envelopes.

- **#1053 — Tauri IPC + webview deployment-shape-(c) catalog coverage gap (severity-minor, phase-4-meta-deferred).** 5-8 anticipated `E_TAURI_*` / `E_WEBVIEW_IPC_*` ErrorCode mints for Phase-4-Meta admin-UI work (deployment shape (c) per CLAUDE.md #17). Not actionable until the Phase-4-Meta admin-UI IPC surface exists; the mints land with that work, routed through this catalog per the standard one-bump-per-denial-surface model. Phase-4-Meta, coupled to the admin-UI IPC implementation.

**Acceptance (Phase-4-Meta / v1-API-stabilization ratification cluster — META #1094 / #924).** #996 + #1030 surface at the v1-API-stabilization ratification round (the same cluster that decides the `serde` wire-form fork per `benten-errors` INTERNALS §10 item 5 + §8); #1053 lands mechanically with the Phase-4-Meta admin-UI IPC work. None is closeable in the ST-ERRORS single-crate lane — #996 + #1030 are ratification-gated arch forks (Ben decision), #1053 is blocked on a not-yet-existing surface. Umbrella #1182 stays OPEN until all three resolve; PR #1268 is Refs-only against #1182 (the #1019 facet is the only Closes-eligible sub-issue, and it landed in dispatch-conventions, not the PR diff).

### §4.74 Plugin private-namespace storage seam — `WriteContext::namespace_did` + per-DID-scoped backend view (BEN-DECISION fork — Fwd-2 #989, umbrella #1215)

Per HARD RULE rule-12 BELONGS-NAMED-NOW + **BEN-DECISION (genuine v1-SemVer/arch fork — surfaced, NOT decided autonomously per night-shift stance).** refinement-audit-2026-05 Fwd-2 #989, umbrella #1215. CLAUDE.md #18 plugin private-namespaces (a plugin's writes go to a DID-scoped namespace; the namespace cap is held by the plugin's DID; `shares=none` blocks delegation) currently has **no storage construction site**: `crates/benten-graph/src/lib.rs::WriteContext` has no `namespace_did` field and no per-DID-scoped backend view exists. The structural cap-policy refusal of cross-plugin private-namespace delegation already ships (`g24d_substantive_pipeline.rs::private_namespace_cap_unconditionally_denied_cross_plugin` PASS at HEAD), but the **storage-scoping seam** that would actually route a plugin's writes into a DID-isolated key-space is unbuilt.

**Why this is a fork, not a mechanical fix:** adding `WriteContext::namespace_did: Option<Did>` is a **public-struct-shape change** on `benten-graph`'s `WriteContext` — once v1 tags, the field is SemVer-locked, and the per-DID-scoped backend-view shape (key-prefix scoping vs separate sub-database vs label-namespace) is a data-layout decision that locks at the same time (analogous to the §4.61 `GraphBackend` SemVer-commitment and the #992 redb schema-version-envelope decision landing via PR #1269). The issue #1215 itself states **"Blocked by: Ben ratification of storage-seam fork"** and "If Ben ratifies 'defer plugin private-namespace storage seam to Phase-4-Meta,' this bundle splits — #992 (schema-version envelope) lands pre-v1; #989 lands at Phase-4-Meta."

**Reconciliation outcome for umbrella #1215 (ST-GRAPH lane):**
- **#992 (redb schema-version envelope) = RESOLVED-BY-#1269** — the decision-queue-tail mega-batch PR #1269 (`batch/decision-queue-tail`, CLEAN at reconciliation) ships the full envelope: `GRAPH_SCHEMA_VERSION` + `SCHEMA_VERSION_KEY` meta-row + `RedbBackend::check_schema_version` + `GraphError::SchemaVersionMismatch` + `redb_schema_version_envelope_pin.rs`. ST-GRAPH does NOT re-implement; #992 closes when #1269 merges. ST-GRAPH adds no code for #992.
- **#989 (this entry) = BEN-DECISION**, parked here. ST-GRAPH does NOT add the `WriteContext::namespace_did` field unilaterally.

**Acceptance criteria.** Pre-v1-stabilization ratification per fork: **(a) defer to Phase-4-Meta** — accept that plugin private-namespace storage isolation is not v1-platform-shippable scope (the structural cap-refusal suffices for v1; physical key-space isolation is a Phase-4-Meta + Phase-6-AI-agent-working-memory concern); OR **(b) land the seam pre-v1** — `WriteContext::namespace_did: Option<Did>` + a per-DID-scoped backend view (`ScopedBackend` wrapping key-prefix `private:<did>:` isolation) + threading through `put_node_with_context` — ~200 LOC + the public-shape SemVer-lock acceptance. Same wave as §4.43 v1-API-stabilization sweep if (b). Surface at morning brief for Ben ratification; bias-to-continue on the rest of the ST-GRAPH lane per night-shift stance.

### §4.75 benten-eval in-source `TODO(phase-3 — …)` cluster retarget (Phase-4-Meta — refinement-audit #1166 / #1095, sibling of §4.68)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 Pattern observation, umbrella #1166; META #1095 phase-rename in-source trajectory sweep; sibling to §4.68's benten-engine cluster). The 2026-05-11 phase-rename (Phase 3.5 → Phase 4-Foundation; Phase 4 → Phase 4-Meta) left `benten-eval` source carrying **8 forward-looking `TODO(phase-3 — …)` markers** whose substantive work explicitly did NOT land in Phase 3 (each marker self-describes the deferral). They are retargeted in-source to `TODO(phase-4-meta — backlog §4.75; …)` (no behaviour change; comment edits only) and enumerated here as the named destination:

1. `lib.rs` Outcome-enum marker — eval/engine `Outcome` unification (now `Outcome::Suspended(SuspendedHandle)` post #878; full eval↔engine `Outcome` unification still pending).
2. `lib.rs::TraceStep` — `SuspendBoundary`/`ResumeBoundary` boundary-variant + attribution-threading completion.
3. `evaluator.rs` — trace-timing-non-determinism doc (trace timing is observability-only, NOT content-addressed).
4. `time_source.rs` — `TimeSource`/`MonotonicSource` default impls back with `uhlc::HLC` / `std::time::Instant`.
5. `host_error.rs` (×2) — host-error wire encode/decode DAG-CBOR + versioned-envelope upgrade.
6. `expr/builtins.rs` — `docs/DSL-SPECIFICATION.md` ↔ dispatch-table audit (`formatDate` add/omit decision).
7. `primitives/call.rs` — `Evaluator.call_depth` counter + multiplicative-budget propagation through the CALL boundary.

**Acceptance criteria.** Each marker resolves at Phase-4-Meta on its own substantive workstream. ~no LOC now (markers retensed at the ST-EVAL lane PR, refinement-audit #1166); ~varies at Phase-4-Meta per item. **pim-N candidate (sibling §4.68 + META #1140):** pre-tag sweep MUST include an in-source `TODO(phase-N — …)` marker scan — 3+-recurrence saturated across Hyg-2/Hyg-3/Hyg-4/Fwd-2 lens dispatches per `feedback_3_plus_recurrence_deep_sweep.md`. Surfaced for orchestrator/Ben pim-N codification at W19 (do NOT codify unilaterally — this row is the named tracking destination, not the codification).

### §4.76 benten-eval Fwd-1 allocation-hot-path + bench-coverage cluster (Phase-4-Meta perf-pass — refinement-audit #1157, `phase-4-meta-deferred`)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-1 Pattern A + B, umbrella **#1157**, labelled `phase-4-meta-deferred`; META #1062 Bench-harness baseline + META #1064 Hot-loop allocation + META #629 DoS-via-unbounded-decode). The umbrella STAYS OPEN as a Phase-4-Meta tracking row (NOT force-closed) per the FULL-EXECUTION-PLAN §4.1 terminal-state model — the ~10 sub-issues are the intentional Phase-4-Meta perf-pass window, pre-Phase-5-reference-app baseline. Sub-issues: #1021 (`run_inner` rebuilds 3 HashMaps + clones `op.id`/TraceStep per step — workspace-hottest loop), #1028 (SANDBOX `kv:read` time-arm clones `tainted_addresses` per host-fn — O(N²)), #1040 (`SubscribeRevokedMidStream` hex-string per fire), #1041 (`publish_change_event_with_labels` O(N×L×M); the `simple_glob_match` pattern-length-cap arm absorbs into META #629), #1044 (`CapAllowlist::contains` O(N) per host-fn), #1045 (`simple_glob_match` exponential backtracking on adversarial input), #1031 (`InMemorySuspensionStore` single Mutex over 5 namespaces), #1025 (SUBSCRIBE publish-walk clones every `OnChangeEntry`), #1034 (no Criterion bench for `primitives/subscribe.rs` publish-walk), #1049 (bench-policy mix OBS).

**Acceptance criteria (Phase-4-Meta perf-pass).** As enumerated in the #1157 umbrella body (precompute prologue HashMaps at handler-registration; `walk_path` `Vec<String>` → `Vec<&'a str>` handler-lifetime-tied; split-borrow restructure removing the kv:read O(N²) clone; new `crates/benten-eval/benches/` targets for SUBSCRIBE publish-walk + per-primitive dispatch + ExecutionState envelope encode/decode; #1045/#1041 pattern-length cap closes a DoS surface and is the ONLY arm with a pre-v1 security flavour — flagged for orchestrator to decide whether the #1045 adversarial-backtracking arm is pulled forward into the v1-BLOCKER #629 cluster or stays Phase-4-Meta). ~250-400 LOC at Phase-4-Meta.

### §4.77 benten-eval SUBSCRIBE composability seam (Phase-4-Meta architectural ratification — refinement-audit #1163 / Surf-1 #875, `phase-4-meta-deferred`)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Surf-1 `Surf-1-eval-2`, umbrella **#1163**, labelled `phase-4-meta-deferred`; META #1094 v1-API-stabilization decision cluster; CLAUDE.md #18 plugins-are-subgraphs + #19 engine-extensions). The umbrella STAYS OPEN as a Phase-4-Meta tracking row (NOT resolved this lane, per ST-EVAL brief directive + FULL-EXECUTION-PLAN W20). SUBSCRIBE runtime state lives in **7 module-level static singletons + 23 `pub fn` free functions** bypassing `PrimitiveHost`; `benten-engine` reaches across the arch-1 dep-break via direct `publish_change_event_with_labels` / `next_engine_seq` calls. The composability seam exists in name only — alternate hosts cannot supply an alternate subscribe runtime without monkey-patching eval-crate globals; load-bearing for CLAUDE.md #18 plugin-authored SUBSCRIBE handlers.

**Acceptance criteria (Phase-4-Meta; Ben architectural ratification required — the seam shape is the gating decision, NOT mechanical).** `PrimitiveHost` (or a new sibling trait per CLAUDE.md #19) exposes the SUBSCRIBE runtime extension hook; default impl preserves current `ON_CHANGE_REGISTRY`/`ACTIVE_HANDLES`/`ENGINE_SEQ` semantics; benten-engine's direct calls route through the trait (arch-1 dep-break preserved); alternate-host integration-test pin asserts the trait is implementable in isolation. The fork (static-singletons-vs-trait-extension-hook-vs-Engine-accessor-vs-pluggable-runtime) is surfaced for Phase-4-Meta R1 ratification — sibling to #834 engine facet-handle split + #827 Renderer IPC promotion. ~400-600 LOC at Phase-4-Meta.

### §4.78 benten-eval v1-API-stabilization cross-crate naming + ErrorCode-catalog forks (Phase-4-Meta / §4.43 v1-API-stabilization sweep — refinement-audit #807 / #802 / #1042, umbrella #1150)

Per HARD RULE rule-12 BELONGS-NAMED-NOW + DISAGREE-WITH-EXPLANATION on benten-eval-LOCAL execution (refinement-audit-2026-05 Qual-2 #807 + #802, Fwd-2 #1042, umbrella #1150). Three #1150 sub-issues are NOT resolvable inside the single-crate ST-EVAL lane without creating *new* cross-crate drift or forcing a cascade into COLLAPSE-gated lanes — surfaced here for the coordinated §4.43 v1-API-stabilization sweep (the umbrella body itself names §4.43 as the home for "naming verb-drift"):

1. **#807 canonical-encoding method-name fork (BEN-DECISION).** Workspace-wide there are FOUR conventions for the same operation: `to_canonical_bytes` (×9 types: benten-platform-foundation + benten-sync ×4 + benten-engine ×3 + benten-eval), `canonical_bytes` (×6, the benten-core/benten-id convention), `to_dagcbor` (×2), `to_dag_cbor` (×3). benten-eval's local drift is `ExecutionStatePayload::to_canonical_bytes` vs `ExecutionStateEnvelope::to_dagcbor` vs `ModuleManifest::canonical_bytes`. Renaming benten-eval-locally to `canonical_bytes` creates NEW drift against the dominant 9-type `to_canonical_bytes` family; a workspace-wide rename touches 4+ crates incl. COLLAPSE-gated lanes (≥63 callsites). **The fork (which of the 4 names is THE v1 convention) is a cross-crate v1-API-SemVer architectural decision — surfaced for Ben/orchestrator ratification at §4.43; do NOT decide unilaterally.**
2. **#802 PrimitiveHost READ-verb mix (BELONGS-NAMED-NOW §4.43).** `read_node` / `get_by_label` / `get_by_property` are all the READ primitive but mix `read_*`/`get_*`. The rename cascades into ~27 files across benten-graph/benten-engine/bindings/benten-errors (38 callsites + every `impl PrimitiveHost`) — a wide cross-lane rename for a Qual-2 MINOR. Belongs in the coordinated §4.43 cross-crate rename batch, NOT the disjoint single-crate drain (merge-conflict risk per FULL-EXECUTION-PLAN §6.2 disjoint-crate rule).
3. **#1042 `Outcome::Err(ErrorCode::Unknown("E_EVAL_BACKEND"))` heap-string boundary leak (cross-lane ST-ERRORS).** Closing it requires minting a typed `ErrorCode::EvalBackend` variant in **benten-errors** + bumping `CATALOG_VARIANT_COUNT` (168→169) + the TS-side mirror per §3.5g — a cross-crate ErrorCode-catalog mutation outside the ST-EVAL single-crate lane. Note: the current `Unknown(String::from("E_EVAL_BACKEND"))` is a *deliberate* r6b-err-3 unification (prior `E_BACKEND`/`E_EVAL_BACKEND` split collapsed); promoting to a typed variant is a SemVer catalog decision. Bundle with the coordinated §4.43 sweep + the cross-crate ErrorCode-catalog wave (ST-ERRORS lane / META #1094).

**Acceptance criteria.** Resolved at the §4.43 v1-API-stabilization sweep as one coordinated cross-crate batch. Surface forks (1) and (3) for Ben ratification before §4.43 locks the affected public surfaces. ~varies (≥63 callsites for #807; ~27 files for #802; ~catalog-bump + TS-mirror for #1042) at §4.43.

### §4.79 benten-core Anchor-shape unification — ONE `VersionDag` with a strict/linear mode (Phase-4-Meta D3 — refinement-audit #849, umbrella #1158/#1142)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Surf-1 #849; umbrella **#1158** v1-API-stabilization cluster, coupled to **#1142** Subgraph/Anchor canonicalization). This is the named Phase-4-Meta destination for the **#849** "three coexisting Anchor shapes" finding. The third (legacy crate-root `u64`-id) shape was DELETED for **#1003** in the #1142 lane (RATIFIED 2026-05-17 — zero non-test callers; CLAUDE.md rule #5). The remaining TWO shapes — `benten_core::version::Anchor` (Cid-head prior-head-threaded linear) and `benten_core::version_chain::DagVersionChain` (DAG-shape, branches/merges native) — still coexist with no shared trait and two different "CURRENT" semantics.

**Ratified design (RATIFIED-decisions-2026-05-17.md D3, Phase-4-Meta-windowed, NON-blocking).** There is ONE version data structure (the DAG). "Linear" is NOT a subtype or degenerate child — it is an opt-in **strict mode/policy** on the one structure (single-parent invariant + fork-rejected-as-typed-error). `version.rs`'s contribution (its `VersionError::Branched`/`UnknownPrior` + "re-read & re-attempt" optimistic-concurrency contract) is carried into the unified DAG as opt-in strict mode; the DAG is structurally strictly more capable (rejects only cycles). Error surfaces already align (`VersionError::code()` ≡ `VersionDagError::code()` → `ErrorCode::VersionBranched`). Decision: one `VersionDag` type; strict/linear is a mode on it (not two types, not a subtype); one shared trait; one CURRENT semantic.

**Acceptance criteria (Phase-4-Meta; Ben-ratified D3 — implementation-only, the shape is locked).** Collapse `version::Anchor` + `version_chain::DagVersionChain` into one `VersionDag` type carrying a strict/linear mode flag; migrate callers (`benten-platform-foundation` plugin-library, `benten-engine` handler-versions/diagnostics, `benten-core` tests) to the unified surface; preserve `ErrorCode::VersionBranched`/`VersionUnknownPrior` mapping; the strict-mode fork-rejection pin (`tests/version_branched.rs`) carries forward unchanged. Couples #1142 + #1158. ~150-300 LOC at Phase-4-Meta. The shape decision is NOT re-litigated (D3 ratified); only the mechanical unification + caller migration is Phase-4-Meta-windowed.

---

### §4.80 `Ucan::from_canonical_bytes_bounded` cross-crate untrusted-decode callsite migration (P-II ONE post-COLLAPSE workspace sweep — refinement-audit #549, umbrella #1172, META #629)

Per HARD RULE rule-12 BELONGS-NAMED-NOW + standing principle **P-II** (cross-crate mechanical sweeps = ONE orchestrator-serialized post-COLLAPSE workspace pass, NOT a disjoint single-crate lane), `RATIFIED-decisions-2026-05-17.md`.

**Landed in the D1 #1172 lane (benten-id-local, this PR):** the depth-bounded untrusted-decode entry point `benten_id::ucan::Ucan::from_canonical_bytes_bounded(bytes, max_depth)` + `MAX_UCAN_PROOF_DEPTH` const + `UcanError::ProofChainTooDeep { depth, max }` typed variant + a non-recursive CBOR-nesting pre-walk that rejects an over-deep `prf` chain at the byte boundary BEFORE `serde`'s recursive `Deserialize` runs (closure-pinned in `crates/benten-id/tests/dos_unbounded_decode_safe2_1172.rs`).

**Still owed (the P-II workspace sweep — NOT this disjoint lane):** the two untrusted-input call sites #549 enumerated must migrate from the bare `serde_ipld_dagcbor::from_slice::<Ucan>` to `Ucan::from_canonical_bytes_bounded(bytes, MAX_UCAN_PROOF_DEPTH)`:

1. `crates/benten-engine/src/typed_call_dispatch.rs` — the typed-CALL `ucan_validate_chain` op (`bytes` is a graph `Value::Bytes` payload, caller-controlled).
2. `crates/benten-caps/src/backends/ucan.rs` — the durable UCAN backend read path (`value` is a redb-stored blob; adversarial input can land via any cap-grant path that doesn't pre-validate depth).

Both crates are COLLAPSE-spine-owned (benten-engine + benten-caps); editing them from the disjoint benten-id lane would break Strategy-C consolidation (FULL-EXECUTION-PLAN §6.2 disjoint-crate rule). The migration is mechanical (swap one decode call + map the new `ProofChainTooDeep` variant onto the existing decode-failure error surface at each site) and rides the next orchestrator-serialized workspace sweep that frees those crates. The DoS surface is NOT closed until both call sites adopt the bounded entry point — the benten-id-local API existing is necessary but not sufficient.

**Acceptance criteria.** Both call sites use `Ucan::from_canonical_bytes_bounded`; a closure-pin at each site asserts an over-deep blob is rejected (not aborted) with the depth-bound error mapped to that site's typed failure; `serde_ipld_dagcbor::from_slice::<Ucan>` has zero remaining production callers (verify via workspace grep). Bundled into the P-II post-COLLAPSE workspace sweep alongside the other META #629 cross-crate slices.
### §4.81 benten-ivm View-trait + Subscriber Phase-5+ plugin-shape cluster (Phase-4-Meta — refinement-audit Fwd-2, umbrellas #1219 + #1220 slice; RATIFIED 2026-05-17)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2; umbrella **#1219** sub-issues #1082/#1091/#1092 + umbrella **#1220** sub-issue #1087; RATIFIED `RATIFIED-decisions-2026-05-17.md` "#1082 / #1091 / #1092, #989 → Phase-4-Meta-named (plugin-ecosystem shape; already Phase-4-Meta scope; zero v1 consumer cost)"; META #669 CLAUDE.md #18 Layers 2+3 paper-only; META #1094 v1-API-stabilization decision cluster). The cited sub-issues stay OPEN as Phase-4-Meta tracking rows; the umbrellas (#1219/#1220) are Refs-only against the benten-ivm crate-drain PR (orchestrator adjudicates Closes via mini-review). Reproduce-verified at HEAD `9b0f327f`: each is a genuine v1-SemVer/arch fork with zero v1 consumer cost today (no Phase-5 plugin views exist at v1):

1. **#1082** — `Subscriber` many-view-scale lift blocked by `View` trait + `Subscriber::register_view` shape at v1; the design-doc (Phase-4-Meta materializer-pipeline-perf wave) must land before the v1 tag. Fork: linear-scan `register_view` vs indexed dispatch; the v1-SemVer-locked `View`/`Subscriber` shape is the gating decision.
2. **#1091** — `Engine::register_user_view` has NO plugin-DID/principal binding; CLAUDE.md #18 three-layer trust model is paper-only at IVM-view registration. Phase-5+ plugin-authored views are BLOCKED at v1 without a Class B β `register_user_view_as(principal, ..)` analog (mirrors the shipped `read_node_as` precedent). Fork: add the principal-bound registration surface at v1 vs document as an explicit Phase-5 blocker.
3. **#1092** — `View` trait is unsealed but unpolicied for method-addition trajectory; Phase-5+ plugin-author `View` impls collide with future method-additions (e.g. a `label_pattern()` accessor) at semver-major. Fork: sealed-trait pattern vs `#[non_exhaustive]`-placeholder-method vs documented additive-default-evolution contract (couples CLAUDE.md #19 out-of-tree-extension policy; sibling to benten-eval §4.77 `PrimitiveHost` seam fork).
4. **#1087** — `SubgraphSpec` name-collision: `benten_ivm::SubgraphSpec` (IVM view def) vs `benten_engine::SubgraphSpec` (handler DSL builder); BOTH `pub`, BOTH consumed by methods named `register_subgraph`. v1-SemVer-locks the ambiguity. Fork: rename one vs module-path disambiguation in rustdoc (zero v1 consumer cost — both resolve via distinct crate paths today; rides the D1 `CanonicalViews` seam window at §4.82).

**Acceptance criteria (Phase-4-Meta; Ben architectural ratification required — the seam/policy shapes are the gating decisions, NOT mechanical).** #1082 design-doc lands pre-v1-tag; #1091 ships the principal-bound registration analog OR a documented Phase-5 blocker per HARD RULE clause-(b); #1092 method-addition policy documented (sealed vs non_exhaustive vs additive-default); #1087 disambiguation resolved (couples D1 §4.82). Surface all four for Phase-4-Meta R1 ratification. ~100-300 LOC at Phase-4-Meta per ratification outcomes.

### §4.82 benten-ivm IVM↔engine `CanonicalViews` boundary seam (D1 — RATIFIED 2026-05-17; Phase-4-Meta-windowed, non-blocking)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Surf-1/Qual-2/Fwd-2; umbrella **#1220** sub-issue #1088 + coupled #911/#914 + umbrella **#1225** sub-issue #758; RATIFIED design decision **D1** in `RATIFIED-decisions-2026-05-17.md`: "IVM↔engine boundary — A2, unified `CanonicalViews` seam"). The cited sub-issues stay OPEN as the D1 implementation tracking row; #1220/#1225 are Refs-only against the benten-ivm crate-drain PR. Reproduce-verified at HEAD `9b0f327f`: the 4 helpers (`dispatch_for` / `is_canonical_view_id` / `hardcoded_label_for_id` / `canonical_typed_output_projection_for`) remain `pub` at crate root + `algorithm_b::*` with 8+ engine external call sites (not the "one site" prior INTERNALS.md framing); the rename (#758) + constructor-narrowing (#914) ride this seam.

D1 (verbatim disposition): collapse the 4 leaked helpers into ONE deliberate public IVM seam — a `CanonicalViews` registry-query type (`lookup(view_id) -> Option<{label, strategy, projection}>`, `is_canonical(id)`) documented ALONGSIDE `Strategy` as the intentional engine-facing boundary per CLAUDE.md baked-in #2; everything else `pub(crate)`. This is strictly cleaner than the status quo (1 cohesive contract vs 4 leaked fns) and honors #2's spirit (a deliberate named seam, not algorithm internals). #758 rename (`dispatch_for` → noun-bearing classifier name, aligning the verb-only outlier with its 3 noun-bearing siblings) + #914 constructor-narrowing (6 overlapping `AlgorithmBView` construction entry points → narrowed; legacy `for_id`/`for_id_with_budget`/`try_register` `pub` → `pub(crate)` where zero-external-caller) ride this seam. #911 (the INTERNAL-helper crate-root `pub use` over-export) is closed by the same `pub(crate)` narrowing. #1088 is the crate-root `pub use` half of the same boundary.

**Acceptance criteria (Phase-4-Meta-windowed; non-blocking for the campaign drain; engine-lane cascade is orchestrator-serialized post-COLLAPSE per P-II since the seam change cascades into benten-engine's 8+ call sites — COLLAPSE-owned).** Land the `CanonicalViews` registry-query type as the single documented engine-facing IVM seam; narrow the 4 leaked helpers + 3 legacy constructors + crate-root `pub use` to `pub(crate)`; migrate the benten-engine call sites in the same serialized workspace pass; rustdoc the seam ALONGSIDE `Strategy` per CLAUDE.md #2; drift-defense pin asserting no other IVM internal is `pub` at the crate root. ~150-300 LOC at Phase-4-Meta (IVM seam) + the orchestrator-serialized benten-engine call-site cascade.

### §4.83 benten-ivm `AlgorithmBView::register_subgraph` materializer-no-call-site (Phase-4-Meta — refinement-audit Fwd-2 #1083, umbrella #1220; RATIFIED 2026-05-17)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2; umbrella **#1220** sub-issue **#1083**; RATIFIED `RATIFIED-decisions-2026-05-17.md` "#1083 materializer-no-call-site → Verify; if dead-until-Phase-4-Meta → BELONGS-NAMED phase-4-backlog"). #1083 stays OPEN as a Phase-4-Meta tracking row; #1220 is Refs-only. **Reproduce-verified dead at HEAD `9b0f327f`:** `benten_ivm::AlgorithmBView::register_subgraph(spec: SubgraphSpec)` (algorithm_b.rs:968) has NO production call-site — the only consumers are benten-ivm's own tests. The materializer pipeline (D-4F-2 ratification) is documented to register through it but the engine-side `Engine::register_subgraph` (the surface schema-compiler routes through, exercised by benten-platform-foundation tests) does NOT reach the IVM `AlgorithmBView::register_subgraph` constructor at HEAD. The `SubgraphSpec` canary-stability contract (#922 drift-defense pin) is therefore stress-untested by a real materializer consumer.

**Acceptance criteria (Phase-4-Meta materializer-pipeline wave).** Either wire the materializer pipeline's production call-site through `AlgorithmBView::register_subgraph` (closing the paper-only-consumer gap + stress-exercising the `SubgraphSpec` canary contract end-to-end), OR — if the materializer pipeline lands its own equivalent registration surface — narrow `AlgorithmBView::register_subgraph` visibility accordingly and document the actual consumer topology. Couples the §4.81 #1082 Subscriber-scale work + the D1 §4.82 seam. ~no LOC now (verified-dead + named here); ~varies at Phase-4-Meta per the materializer-pipeline shape.

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

### §4.84 benten-caps cap-policy critical-path bench-coverage gap (Phase-4-Meta perf-pass — refinement-audit Fwd-1 #946 Track 2, umbrella #1143)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-1 #946 Track 2, umbrella **#1143**; META #1062 Bench-harness baseline). The #946 Track-1 allocation cleanup (consolidate the two `StepOutsideEnvelope` construction sites into a single `step_outside_envelope` helper) shipped in the ST-CAPS crate-drain (umbrella #1143 commit). Track 2 — the **benchmark-coverage gap** — is the named Phase-4-Meta carry here:

The `benten-caps` crate ships exactly ONE Criterion bench (`benches/wallclock_toctou_refresh.rs`, informational-only per its own header). The cap-policy critical path — fired on EVERY transaction commit — has zero perf-regression guards. No bench covers: `GrantBackedPolicy::check_write` (per-commit cap-check cascade, #928), `UcanGroundedPolicy::check_write` / `iter_installed_proofs` full-scan cascade (#936), `validate_chain_at` / `validate_chain_for_audience_at` (Ed25519 verify per token), `validate_chain_with_manifest_envelope` (apply_atrium_merge per-row hot path). A future Phase-4-Meta refactor (the `GrantReader::wildcard_variants` pushdown named at INTERNALS §7 item 5 / §8, or the proposed `g14b:grant_by_cap:*` secondary index) would have no informational baseline to regression-check against.

**Acceptance criteria (Phase-4-Meta perf-pass).** Add 3-4 Criterion benches under `crates/benten-caps/benches/` (informational-only, matching the `wallclock_toctou_refresh` pattern) covering the four critical-path surfaces above, landed together with the `wildcard_variants` pushdown so the pushdown PR carries a before/after baseline. Couples to §4.76 (benten-eval Fwd-1 bench-coverage cluster) + META #1062 as one Phase-4-Meta bench-harness baseline pass. ~120-180 LOC of bench targets. Note: #946 Track 2's original recommendation cited `crates/benten-caps/INTERNALS.md §6` as the alternate destination; INTERNALS.md is a local-only (gitignored) doc, so this tracked-doc backlog row is the canonical HARD-RULE clause-(b) named destination.

### §4.85 benten-caps retire Phase-2a test-readability conveniences (Phase-4-Meta — refinement-audit Qual-1 #683 + #674-residual, umbrella #1154)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Qual-1 #683, umbrella **#1154**; sibling of §4.84 + the benten-eval §4.76 cluster). The ST-CAPS crate-drain (#1154 commit) fully drained the self-contained items (#816 dead-enum delete; #803 scope-derivation extraction; #674 `wallclock_refresh_ceiling_for` zero-consumer delete) and marked the cross-crate-cascade renames P-II (#661 / #793). The residual is a **production-wiring-gated** cleanup that cannot drain in the disjoint single-crate lane without removing currently-live test/bench surfaces:

`testing::WallclockProbe` (Qual-1 #683) is a 2-method struct over a `Duration` whose `force_refresh()` returns a hardcoded `1`; it is consumed live by `benches/wallclock_toctou_refresh.rs` + `tests/wallclock_refresh_typed_error_fires.rs`. Per the Qual-1 report's own disposition, this surface (together with the `evaluator_delegation` accessors + the `with_now_for_test` seam) only retires when the **production `WriteContext::now` threading + wallclock-refresh-ceiling consumer wire-up** lands (registered at `docs/future/phase-3-backlog.md §2.3 (i)+(ii)`; co-routed with §10.1 Compromise #1 TOCTOU window bound). Deleting the probe now would strand the bench + integration test with no production replacement.

**Acceptance criteria (Phase-4-Meta).** When the §2.3 (i)+(ii) production wire-up lands: replace `WallclockProbe::force_refresh`'s hardcoded `1` with the real refresh-event surface (or delete `testing::WallclockProbe` outright + migrate the bench/integration-test to the real seam); retire `evaluator_delegation::iterate_batch_boundary_for` only if the engine's `primitive_host` consumer is simultaneously re-pointed at the trait method (cross-crate, P-II-coordinated). One cohesive "retire Phase-2a test-readability conveniences" change folded into the §4.84 / §2.3 wire-up wave. ~60-120 LOC at Phase-4-Meta.

### §4.136 benten-sync Fwd-1 performance pass — uncached/redundant-clone hot paths + zero-bench-harness (Phase-4-Meta — refinement-audit Fwd-1, umbrellas #1184 + #1186-adjacent)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-1 lens; bundle umbrella **#1184** MST cached-root wrapper). benten-sync is the workspace's most async/network/crypto-heavy crate and ships zero benchmark coverage. Cluster members: **#1011** (`Mst::root_cid` uncached + O(n) per call; production cache-layer wrapper phantom — G16-B engine integration shipped without it; `apply_atrium_merge` calls it repeatedly during convergence), **#1013** (`Mst::insert` clones `entry.key` when entry owns it — structurally-redundant per-insert String clone), **#1001** (`MstDiff::between` clones every diverging payload `Vec<u8>` at diff-enumeration time — compounds across partial-sync-cursor rounds), **#1006** (no `benches/` directory / no criterion dep at all — handshake Ed25519+UCAN-chain, Loro merge, MST diff, iroh QUIC, Merkle-proof all unguarded; `bench = false` in Cargo.toml). Why Phase-4-Meta: per-zone MST cardinality stays "tens" at v1; the cached-root + bench-harness work is forward-readiness for Phase-6+ AI-agent / Phase-8 decentralized-registry scale where cardinality grows to thousands.

**Acceptance criteria (Phase-4-Meta perf-pass).** Land the `Mst` cached-root wrapper at the `apply_atrium_merge` engine integration site (#1184 bundle scope) together with the redundant-clone removals (#1013 / #1001) and a `crates/benten-sync/benches/` Criterion harness (informational-only, matching the `benten-caps`/`benten-eval` §4.84/§4.76 pattern) covering handshake / Loro-merge / MST-diff / Merkle-proof so the cached-root PR carries a before/after baseline. Couples §4.84 + §4.76 + META #1062 as one Phase-4-Meta bench-harness baseline pass. ~150-250 LOC.

### §4.137 benten-sync Fwd-2 forward-readiness — Cargo.toml [features] absence + post-iroh/post-quantum/identity-recovery seams (Phase-4-Meta / Phase-9+ — refinement-audit Fwd-2, umbrella #1186, META #486 + META #924)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 lens; bundle umbrella **#1186**; META #924 root-position feature-flag absence 5th instance; META #486 INTERNALS-acknowledged-but-not-delivered). Cluster members: **#1077** (no `[features]` table in Cargo.toml — 4 Phase-4-Meta cleanup paths converge: `testing` feature, `tracing` feature, future post-iroh transport backends, wasm32 opt-out), **#1079** (identity-recovery protocol seam — `HandshakePayload::RecoveryAttestation` future variant — lacks phase-4-backlog row; v1-assessment-window identity-recovery decision is the gating fork), **#1081** (iroh-`EndpointId` ↔ `PeerId` byte-equivalence assumption / crypto-minor-4 lacks formal test pin — load-bearing for #889 Transport-trait + #1080 post-quantum agents), **#1080** (post-quantum signature-scheme migration is a CLAUDE.md #19 named extension lacking Phase-9+ scope-naming — 4-crate cascading SemVer-major + missing Compromise-registry entry). Why Phase-4-Meta/Phase-9+: these are v1-API-freeze-risk + named-extension forward seams, not deployed defects; identity-recovery is an explicit v1-assessment-window decision; post-quantum is Phase-9+ committed-scope-adjacent.

**Acceptance criteria.** #1077 features-table lands at Phase-4-Meta v1-API-stabilization (couples §4.137-internal `testing`/`tracing` gates); #1079 identity-recovery seam shape decided at the v1-assessment-window identity-recovery-protocol choice (CLAUDE.md #15) and the `RecoveryAttestation` variant either lands or is documented Phase-5+ blocker per HARD RULE clause-(b); #1081 byte-equivalence test pin lands with the #889 Transport-trait work; #1080 receives a Phase-9+ scope-naming row + a `docs/SECURITY-POSTURE.md` Compromise-registry entry at Phase-4-Meta (the naming is the deliverable; the migration is Phase-9+). Surface #1079 + #1080 for Phase-4-Meta R1 ratification (architectural forks). ~80-150 LOC of scaffolding/docs at Phase-4-Meta; migration LOC is Phase-9+.

### §4.138 benten-sync Safe-2 boundary — untrusted-peer amplification-within-outer-cap defense-in-depth (Phase-4-Meta — refinement-audit Safe-2; META #629-adjacent, NOT the flagship unbounded-decode cluster)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Safe-2 lens). These are amplification-WITHIN the existing 4-MiB outer transport cap (bounded) + defense-in-depth, distinct from the flagship META #629 unbounded-primary-decode BLOCKER cluster (tracked separately, NOT this partition). Cluster members: **#574** (no caps on `RevocationEntry.path` String length nor on `Vec<RevocationEntry>` count from untrusted peer — amplification within the 4-MiB outer cap), **#581** (`MerkleProof::approximate_bytes` allocates the full DAG-CBOR encoding to measure size — budget check happens AFTER the unbounded allocation rather than as a pre-check), **#569** (application-layer `Handshake` doesn't cross-check the iroh-transport-authenticated peer-id against `initiate_frame.peer_id` — defense-in-depth weakness; iroh QUIC-TLS already cryptographically binds the outer identity). Why Phase-4-Meta: bounded by the outer cap (no primary DoS); defense-in-depth hardening rather than a deployed-at-HEAD violation.

**Acceptance criteria (Phase-4-Meta input-hardening pass).** Add typed `max_length` deserialize-time caps on `RevocationEntry.path` + count caps on the revocation-set Vecs (#574); convert `MerkleProof::approximate_bytes` to a streaming size-estimate / pre-check before allocation (#581); add the application-layer peer-id cross-check in `Handshake::respond` against the iroh-authenticated `remote_peer` (#569). Couples the META #629 input-hardening wave for the within-cap amplification surfaces. ~80-140 LOC.

### §4.139 benten-sync Safe-4 async — synchronous-crypto-in-accept-loop + cancel-on-drop docs (Phase-4-Meta Path-A doc + audit-trail — refinement-audit Safe-4)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Safe-4 lens; all Path-A doc-only dispositions per the issues' own HARD-RULE-12 disposition fields). Cluster members: **#644** (`Handshake::initiate/respond/respond_with_window/finalise` are synchronous + do CPU-bound Ed25519/UCAN crypto — risky in a hot accept loop composed with #574; Path A = rustdoc note + production-call-site discipline note, NOT premature async+spawn_blocking absent a confirmed-pinned async accept-loop), **#647** (`Connection::send_bytes`/`recv_bytes` undocumented cancel-on-drop semantics — partial-byte-state visible to peer; Path A = "Cancellation safety" rustdoc subsection; load-bearing for timeout-fix #638), **#651** (`Endpoint::status` `tokio::sync::Mutex` over-conservative since the doc-stated #375 background task does not exist at HEAD — OBS-only DISAGREE-WITH-EXPLANATION + audit-trail comment). Why Phase-4-Meta: Path-A doc/audit-trail dispositions; converting to async is premature absent a confirmed async accept-loop wrapper (the wrapper itself is Phase-4-Meta materializer/transport work).

**Acceptance criteria (Phase-4-Meta).** Land the rustdoc "Cancellation safety" subsections (#647) + sync-crypto-in-hot-loop discipline note (#644) + the `Endpoint::status` audit-trail comment (#651) co-routed with the timeout-fix #638 / async accept-loop work when that wrapper is designed. INTERNALS.md §3 handshake-module addendum is local-only (gitignored) — this tracked-doc row is the canonical clause-(b) named destination for the rustdoc/audit-trail deliverables. ~40-80 LOC of docs/comments.

### §4.140 benten-sync Safe-1 + Hyg + Surf-1 + Qual cleanup cluster (Phase-4-Meta — refinement-audit Safe-1/Hyg-2/3/4/Surf-1/Qual-1/2)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Safe-1/Hyg/Surf-1/Qual lenses). Production-unreachable / type-state-machinery / stale-cite cleanups, none deployed defects. Members: **#520** (handshake.rs `signing_bytes` pair uses `.expect()` — production-unreachable today but documents fragility wrong-side), **#378** (crdt.rs `OpLogTarget` walks deep-value root names only; finer per-op walk named `wave-6b-r6-fp` precondition met without delivery), **#384** (mst.rs `merkle_proof_for` O(n) shape cites "future Phase-4 tree-shaped optimization"; destination exists at CRATES-DEEP-DIVE.md but in-code comment doesn't cite it), **#457** (handshake.rs:405-410 cites non-resolving wave-tag `G14-A2`; closest existing is G14-A/G14-D/G16-D wave-6b), **#489** (light_client_distinct.rs #[ignore] rationales cite coarse "Phase-4 Benten Platform v1" — phase-rename split means Phase-4-Meta specifically), **#890** (`LoroDoc::list`/`map` leak `loro::LoroList`/`LoroMap` through public API — CRDT-layer composability gap), **#891** (`MstCid::from_blake3_digest` wraps vs `from_bytes` hashes — naming-asymmetry content-addressing-disaster mode), **#679/#689/#700/#715/#723** (Qual-1 simplicity: 5 constant-true bool accessors, PeerDiscoveryConfig one-field wrapper, 7 newtype accessors, HandshakeFrameBuilder 6-phantom type-state ~125 LOC for 3 fields [SURFACED-FOR-BEN], 3 bind* + respond/respond_with_window default-wrapping OBS), **#762/#796** (Qual-2: `BandwidthBudget::limit_bytes` named-after-field constructor, `respond`/`respond_with_window` non-discriminating suffix). Why Phase-4-Meta: cosmetic/forward-readiness; #715 type-state simplification is SURFACED-FOR-BEN (architectural ratification, not mechanical).

**Acceptance criteria (Phase-4-Meta naming/cleanup wave + v1-API-stabilization).** Hyg stale-cite fixes (#457/#384/#489) + the `.expect()` audit-trail rewrite (#520) + crdt per-op-walk decision (#378) land in a benten-sync cleanup pass; Surf-1 type-leak fixes (#890/#891) + Qual-2 renames (#762/#796) land at v1-API-stabilization (cross-crate consumer cascade — P-II-coordinated); the Qual-1 type-state simplification (#715) is surfaced for Phase-4-Meta R1 Ben ratification BEFORE any mechanical change (keep-vs-simplify is the gating decision). ~150-300 LOC at Phase-4-Meta per ratification outcomes.

### §4.141 benten-ivm Fwd-1 performance + bench-coverage pass — Subscriber fan-out scaling + hot-path allocation (Phase-4-Meta — refinement-audit Fwd-1, umbrella #1221)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-1 lens; bundle umbrella **#1221** Subscriber fan-out + hot-path perf pass). The Subscriber fan-out axis is the load-bearing perf axis for Phase-4-Meta materializer pipelines + plugin views. Cluster members: **#963** (`Subscriber::view_strategy/view_is_stale/read_view/read_view_allow_stale` all linear-scan + Mutex per call — id-keyed HashMap gives O(1)), **#960** (`Subscriber::stale_count_tally` linear-scans all views under Mutex per metric-snapshot tick — should be atomic counter at stale-flip site), **#958** (`Subscriber::view_ids` allocates `Vec<String>` + per-view clone every call; metric-snapshot tick path), **#953** (zero bench coverage on Subscriber fan-out scaling — all 5 benches single-view; couples Fwd-1-B2), **#956** (`ivm_generalized_kernel_hot_path` bench has no corpus-size sweep — single 64-event corpus folds alloc + update + insertion + materialize into one signal), **#966** (no delete-storm bench against `ContentListingView` — B1 perf-deferral has no measurement-backed gate), **#967** (`GovernanceInheritanceView::effective_rules` per-query Vec+BTreeSet alloc; depth-cap=5 bounds cost — OBS-only). Why Phase-4-Meta: depth-caps/single-view sizing bound v1 cost; the fan-out + corpus-sweep benches are forward-readiness for materializer-driven view-count growth (D-4F-2), not deployed defects.

**Acceptance criteria (Phase-4-Meta perf-pass).** Land the Subscriber id-keyed HashMap index (#963) + atomic stale-counter (#960) + cached view-id list (#958) together with a `subscriber_fan_out_scaling` bench (view-count axis [1,5,10,50,100,500]) + corpus-size sweep on the kernel hot-path bench (#956 [64,256,1024,4096]) + a `ContentListingView` delete-storm bench (#966), so the Subscriber-index PR carries the regression baseline BEFORE materializer-driven view-count growth ships. #967 documented OBS-only (depth-cap=5 bounds it). Couples §4.81 Subscriber-scale cluster + §4.84/§4.76 bench-harness baseline + META #1062. ~250 LOC across 4-5 files (per #1221 estimate) + bench targets.

### §4.142 benten-ivm Surf-1 no_std-intent + canonical-view budget-drop (Phase-4-Meta — refinement-audit Surf-1)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Surf-1 lens). Members: **#920** (crate-root `extern crate alloc;` + view.rs/budget.rs/subgraph_spec.rs strict `alloc::*` discipline signals no_std intent that `subscriber.rs` `std::sync::Mutex`+`std::panic::catch_unwind` + `testing.rs` `std::fs`/`std::env` silently break — no `std` feature gate), **#919** (`SubgraphSpec::for_canonical_view('content_listing').with_label_pattern(non_post).with_budget(Some(n))` silently drops the budget — universal-input composability fails at the one canonical view that accepts label override). Why Phase-4-Meta: no_std-intent is a v1-API-stabilization feature-gate decision (couples the benten-ivm §4.81/§4.82 boundary work); the budget-drop is a Surf-1 composability sharpening, not a deployed correctness defect at HEAD's canonical-view usage.

**Acceptance criteria (Phase-4-Meta v1-API-stabilization).** Either add an explicit `std` feature gate making no_std intent enforceable + CI-checked (#920), OR document the crate as std-only and remove the misleading `alloc::*`-only discipline; make `SubgraphSpec::with_budget` after `for_canonical_view` either honored or a typed error rather than a silent drop (#919). Surface #920 std-vs-no_std for Phase-4-Meta R1 ratification (couples §4.81 #1092 method-addition policy + §4.82 D1 seam). ~60-120 LOC.

### §4.143 benten-caps Fwd-2 forward-readiness — v1-API-freeze risk: principal-resolution + sealed-trust-hook + features-absence (Phase-4-Meta — refinement-audit Fwd-2, v1-assessment-window)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-2 lens; v1-API-freeze-risk class). Members: **#1005** (`WriteContext::actor_hint`(String) + `UcanGroundedPolicy::principal_did_from_context` did:key-prefix-scan is Phase-1-shaped principal resolution; v1-API-freeze risk against Kith Phase-5+ non-did:key methods + cap-r1-16 `actor_cid` lift), **#993** (`CapabilityPolicy` trait lacks sealed-extension discipline for the Phase-4-Meta three-layer trust-hook additions — install-time consent, per-delegation runtime hook, audience-aware `check_write`; v1-API-freeze risk), **#1017** (Phase-4-Meta cleanup — `TODO(phase-3)` markers + `legacy_manifest_scope`-deprecated cleanup + `LegacyUcanStubBackend` retention all coupled to the v1-assessment-window pre-tag sweep), **#886** (no `[features]` section in Cargo.toml — durable-UCAN + plugin-trust composition exclusively `cfg(not(wasm32))`-gated with no native opt-out). Why Phase-4-Meta: these are exactly the v1-API-freeze-risk + three-layer-trust-hook seams the Phase-4-Meta plugin work (CLAUDE.md #18) plugs into; freezing the wrong shape pre-Phase-5 is the risk being tracked, not a HEAD defect.

**Acceptance criteria (Phase-4-Meta v1-API-stabilization + plugin-trust wave).** `CapabilityPolicy` sealed-vs-non_exhaustive-vs-additive-default decided + the three-layer trust-hook surface (install-time consent / per-delegation / audience-aware `check_write`) shaped at Phase-4-Meta plugin-trust wave (#993, couples CLAUDE.md #18); principal-resolution lifted to `actor_cid` or documented Kith-Phase-5 blocker per HARD RULE clause-(b) (#1005, couples cap-r1-16); `legacy_manifest_scope` + `LegacyUcanStubBackend` + `TODO(phase-3)` markers swept at the v1-assessment-window pre-tag sweep (#1017); `[features]` table added for the wasm32/durable-UCAN opt-out (#886, couples §4.137 cross-crate features-table pass). Surface #993 + #1005 for Phase-4-Meta R1 ratification (v1-API-freeze architectural forks). ~150-300 LOC at Phase-4-Meta per ratification outcomes.

### §4.144 benten-caps Fwd-1 perf — derive_scope per-op alloc + chain-validator clone-waterfall bench gap (Phase-4-Meta perf-pass — refinement-audit Fwd-1, couples §4.84)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Fwd-1 lens; sibling of §4.84 cap-policy bench-coverage cluster, umbrella #1143-adjacent). Members: **#954** (`derive_write_scope`+`derive_read_scope` allocate a fresh String per `PendingOp` via `format!` — hot-path per-op alloc that `&'static str` interning or `SmallVec<[&str;4]>`+`str::join` could close), **#946** (the `manifest_envelope_chain_validation` walker clones `DelegationStep` fields × steps on every chain validation — O(N) Did/String clones on the error path; **#946 Track-1 already shipped** in the ST-CAPS crate-drain; this is the **Track-2 benchmark-coverage gap** also home at §4.84 — cross-reference, not double-home: the per-op `derive_scope` alloc #954 is the net-new carry here). Why Phase-4-Meta: per-op alloc is a forward-readiness perf-pass with no deployed-defect; the bench-coverage half is already named at §4.84.

**Acceptance criteria (Phase-4-Meta perf-pass).** Land the `derive_write_scope`/`derive_read_scope` per-op-alloc removal (#954 — `&'static str` interning or `SmallVec`+`str::join`) together with the §4.84 cap-policy bench-harness so the alloc-removal PR carries a before/after baseline; #946 Track-2 bench-coverage is cross-referenced to §4.84 (single named destination — not re-homed here). Couples §4.84 + META #1062. ~60-120 LOC + the §4.84 bench targets.

### §4.145 benten-caps Safe-4 + Hyg + Qual-2 cleanup cluster (Phase-4-Meta — refinement-audit Safe-4/Hyg-1/2/3/4/Qual-2; #683/#674 cross-ref §4.85)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 Safe-4/Hyg/Qual lenses). Cleanup + snapshot-consistency + naming, none deployed defects. Members: **#641** (`validate_chain_at`+`validate_chain_with_durable_revocations` issue N+1 separate KV gets without snapshot — concurrent revoke races visible inside a single validate call; Phase-4-Meta because the snapshot-read seam couples the §4.143 #993 trust-hook redesign), **#300** (`WriteContext::with_authority` unused — only `benten_graph::WriteContext::with_authority` consumed), **#303** (`manifest_scope::manifest_delegates_anything` dead — only its own self-test consumes it), **#358** (bench `wallclock_toctou_refresh.rs` stale R3 "returns todo!()" narrative — probe real at HEAD), **#458** (grant_backed.rs:329 references phantom test file that does not exist at HEAD), **#480** (device_dispatch.rs module-doc claims "(c) stays #[ignore]'d" but (c) relocated to benten-engine + GREEN at HEAD), **#793** (`UcanGroundedPolicy::with_now_for_test` is the PRODUCTION now-injection seam despite `_for_test` suffix — naming actively misleads; cross-crate-cascade rename P-II per §4.85). Note **#661** (`AsAttenuationScope` 4-impl overhead), **#668** (`GrantReaderChain` 3-constructor split + dead `with_config`), **#683/#674** are P-II/production-wiring-gated and are already home at **§4.85** (#683/#674) + P-II rename queue (#661/#793) — cross-referenced, NOT re-homed here; the net-new carries here are #641 + the Hyg-1/2/3/4 dead-code/stale-cite cleanups (#300/#303/#358/#458/#480).

**Acceptance criteria (Phase-4-Meta cleanup wave).** Delete dead `with_authority`/`manifest_delegates_anything` (#300/#303); fix stale-cite/phantom-file/relocated-test narratives (#358/#458/#480); add snapshot-consistent revocation reads to `validate_chain_at`/`validate_chain_with_durable_revocations` co-routed with the §4.143 #993 trust-hook seam redesign (#641 — the snapshot seam is the architectural coupling, not mechanical); #793 `with_now_for_test` rename rides the §4.85 P-II cross-crate rename queue (single named destination — not re-homed). ~80-160 LOC at Phase-4-Meta.

### §4.146 benten-caps Part B trust-model encryption leg — content-encryption key model (Phase-4-Meta / v1-assessment-window — refinement-audit S6, DECISION-RECORD trust-model-reframe §5)

Per HARD RULE rule-12 BELONGS-NAMED-NOW (refinement-audit-2026-05 trust-model reframe S6; **#1233**). The 2026-05-15 trust-model reframe split Part A (authority spine — gates the current fix-pass, already SHIPPED + settled) from **Part B (this — the genuinely-greenfield encryption leg)**. Part B does NOT gate the current fix-pass; the encryption leg is ABSENT from all shipped code (prior review 1/10 — all shipped key material is signing-only). Scope: content-encryption key model (none exists), decrypt-capability ↔ UCAN-capability relationship, peer-agnostic + multi-tenant crypto-isolation. Authoritative: `.addl/refinement-audit-2026-05/DECISION-RECORD-trust-model-reframe.md §5` + `docs/future/phase-4-backlog.md §4.58` + `.addl/refinement-audit-2026-05/vision-graph-ownership-encryption-multitenancy.md`. Why Phase-4-Meta/v1-assessment-window: greenfield encryption design requiring Ben architectural ratification; explicitly deferred (not dropped) by the reframe decision-record.

**Acceptance criteria (Phase-4-Meta / v1-assessment-window — Ben architectural ratification gating).** This row is the tracked-doc clause-(b) named destination keeping #1233 board-tied; the design lands at the v1-assessment-window per the DECISION-RECORD §5 + §4.58 (content-encryption key model + decrypt-capability ↔ UCAN rel + multi-tenant crypto-isolation). Surface for v1-assessment-window R1 ratification — the encryption-model shape is the gating architectural decision, NOT mechanical. LOC = greenfield, scoped at the v1-assessment-window per ratification outcomes.

---

(Section structure additive; entries land as Phase 4-Foundation work surfaces them.)
