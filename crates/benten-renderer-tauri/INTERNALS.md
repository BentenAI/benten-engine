# `benten-renderer-tauri` — Crate Internals

Plain-English deep-dive for the 12th workspace crate, shipped in Phase 4-Foundation wave G24-E. The second `Renderer` trait impl after `BrowserRender` (which lives in `benten-platform-foundation`); host for the Tauri 2.x embedded-webview deployment shape (c) per CLAUDE.md baked-in commitment #17. Read-only audit — no compile / no cargo / no claims about CI state.

---

## 1. What this crate does

This crate is the engine-level Tauri 2.x renderer backend. It exists to fulfill three things at once:

- **The third deployment shape from CLAUDE.md #17.** The native shell wraps a webview that loads the same `wasm32-unknown-unknown` admin UI v0 bundle as shape (b) browser-tab; the shell is a full peer internally; webview and shell talk via in-process IPC instead of `fetch`. This crate hosts the IPC dispatcher + locked CSP + session bridge for that shape.
- **The second `Renderer` impl** under the Renderer-backend swappability pattern (CLAUDE.md #17 Ben 2026-05-11 amendment). The trait surface is defined in `benten_platform_foundation::Renderer`; this crate provides `TauriRenderer` as a sibling to `BrowserRender`. The trait surface is transport-agnostic so a future `tauri-runtime-verso` swap is one-line — see `tauri_runtime_verso_swap_readiness_compile_test.rs`.
- **The T3 defense from `admin-ui-v0-threat-model.md`** (cap-elevation under the embedded webview). Three rungs: explicit method-name allowlist (rung 1); per-method capability binding against the admin UI v0 manifest's `requires` envelope (rung 2); locked Content-Security-Policy at webview load (rung 3). The composition is exercised end-to-end at `tests/compromised_webview_cannot_escalate_to_native_filesystem.rs`.

The crate is an **engine extension** per CLAUDE.md baked-in #19 — compile-time linked Rust crate trusted because "you compiled this in," NOT an app-level plugin going through `read_node_as` or UCAN. The boundary is `cargo` and code review. The crate root's module doc names this trust posture explicitly; the `benten_renderer_tauri_is_an_engine_extension_per_claude_md_19` test grep-asserts the `CLAUDE.md #19` and `read_node_as` phrases stay in the source.

---

## 2. Dependency chain

**Upstream (what `benten-renderer-tauri` pulls in):**

- `benten-core` — base graph primitives.
- `benten-errors` — the `ErrorCode` catalog; `IpcError::error_code()` maps the four T3 surface errors back to stable codes (no new codes minted at G24-E; the four cases reuse `ThinClientHandshakeInvalid` per the cross-protocol contract br-r1-14).
- `benten-engine` — `thin_client::{DidKeyedSession, SessionToken, ThinClientSessionError, Transport}` for the in-process session bridge. This is the load-bearing reuse — shape (c) and shape (b) share **one** session contract.
- `benten-platform-foundation` — `Renderer`, `MaterializerOutput`, `RenderError`. The trait surface; this crate provides the second concrete impl.
- `thiserror` — typed `IpcError`.
- `serde_json` — IPC payload values; the wire framing (Tauri 2.x `invoke` JSON) is the integrator binary's responsibility, but the already-parsed `serde_json::Value` lives in `IpcRequest::payload`.

**No `tauri = "2"` dep.** The Tauri 2.x crate dep itself is held by the integrator binary (`tools/benten-admin-shell/Cargo.toml` under the `tauri` cargo feature). This crate stays Tauri-runtime-agnostic so the `tauri-runtime-verso` swap-readiness pin per br-r1-9 + gap #1b holds. The Cargo.toml carries an explicit narrative comment at line 17-32 documenting this layering.

**Downstream (what depends on `benten-renderer-tauri`):**

- `tools/benten-admin-shell` — the integrator bin crate. Composes `TauriRenderer` + `InProcessSessionBridge` + the canonical admin-UI-v0 manifest envelope and provides the real Tauri 2.x command-handler wiring under the `tauri` feature.

**Forbidden (enforced as a test) — `arch_n_benten_renderer_tauri_dep_direction.rs`:**

The Cargo.toml MUST NOT pull `benten-sync`, `iroh`, `iroh-net`, `wasmtime`, `redb`, or `loro`. These are native-only sync-runtime crates; the admin UI v0 bundle the Tauri webview loads is the same `wasm32-unknown-unknown` bundle shape (b) browser-tab uses, so it must stay thin. The pin lives in **`three_rung_baked_in_17_defense_extension_pin.rs`** as an extension of the 3-rung baked-in-#17 defense from PR #166 (Phase 3 G16-B-B precedent). Rung 1 = wasm-objdump forbidden-prefix sweep (runtime-arm via `ADMIN_UI_V0_WASM_PATH` env var; CI authoritative). Rung 2 = Cargo.toml dep-name assertion (always-on). Rung 3 = module-doc engine-extension reference grep.

**Reverse-coupling forbidden too.** `benten-renderer-tauri` must NOT become a dep of `benten-engine` or `benten-platform-foundation` — engine extensions plug INTO the engine surface; the engine doesn't reach into a specific renderer. Sibling pin at `arch_n_benten_renderer_tauri_dep_direction.rs` enforces this.

The Cargo.toml's `[dev-dependencies]` block is intentionally empty — the R6-FP-5 arch-r6-r5-1 fix-pass removed the redundant block that duplicated production deps (each crate was showing up with both `kind: None` and `kind: dev` in `cargo metadata`). Sister crate `benten-platform-foundation` keeps only test-specific deps in its `[dev-dependencies]`.

---

## 3. Files in `src/`

Single-file crate. `lib.rs` is 640 LOC. Reads naturally in four sections:

### `lib.rs` §1 — IPC allowlist (T3 defense rung 1)

Two `pub const` arrays + one `IpcAllowlist` struct.

- **`IPC_METHOD_NAME_ALLOWLIST: &[&str]`** (lib.rs:87-96) — 8 entries: `engine.read_node_as`, `engine.call_as`, `engine.subscribe_via_on_change_as_with_cursor`, `engine.list_caps`, `engine.identity.user_did`, `plugin.manifest.review`, `plugin.install.consent`, `ui.notify`. The drift-detector pin at `tests/ipc_method_name_stability_drift_detector.rs` couples this set byte-for-byte to `docs/public-api/benten-renderer-tauri.json::_ipc_method_name_allowlist_baseline._anticipated_method_set`. Adding a method requires explicit baseline update + admin UI v0 manifest review — silent IPC surface expansion is a manifest-bypass risk.
- **`IPC_METHOD_CAP_BINDING: &[(&str, &str)]`** (lib.rs:105-117) — pairs each method with its required cap scope: `engine.read_node_as → graph:read`, `engine.call_as → graph:write`, `engine.list_caps → caps:read`, `engine.identity.user_did → identity:read`, `plugin.manifest.review → plugin:read`, `plugin.install.consent → plugin:install`, subscribe → `graph:read`. **`ui.notify → ""`** is the "no cap required" sentinel (UI-only side-effect inside the webview; `AdminUiManifest::grants_cap` treats empty cap as always granted).
- **`IpcAllowlist`** (lib.rs:122-169) — wraps a `BTreeSet<String>` for fast membership; `canonical()` constructor mirrors the const array. `method_permitted` is rung-1 enforcement; `required_cap_for_method` returns the bound cap or `None` for non-allowlisted methods. `methods()` iterator powers the drift-detector pin.

The unit test `cap_binding_covers_every_allowlisted_method` (lib.rs:633-639) is the bake-in: every method in the allowlist MUST appear in the cap-binding table — adding to one and not the other is a compile-test-survival shape that the unit test catches.

### `lib.rs` §2 — Locked CSP (T3 defense rung 3)

Single `pub const WEBVIEW_CSP_HEADER: &str` (lib.rs:191-195). Five directives:

```
default-src 'none';
script-src 'self' 'wasm-unsafe-eval';
connect-src 'self' tauri://*;
style-src 'self';
font-src 'self'
```

The `'wasm-unsafe-eval'` token is the wasm-only relaxation (allows `WebAssembly.compile`-equivalent without enabling classic `eval`); the inline-test `csp_header_forbids_unsafe_eval_and_unsafe_inline` (lib.rs:619-631) strips that token before asserting `'unsafe-eval'` is absent + asserts `'unsafe-inline'` is absent. Surface-public test `tests/webview_csp_locked_no_unsafe_eval.rs` carries the load-bearing pin per br-r1-11.

CSP is a load-boundary defense, not a per-call defense. The integrator binary wires `WEBVIEW_CSP_HEADER` into Tauri's `WebviewWindowBuilder::with_csp()` (or equivalent) at boot. `TauriRenderer::webview_csp_header()` is the accessor.

### `lib.rs` §3 — IPC envelopes + typed errors

Three small structs + one error enum.

- **`IpcRequest`** (lib.rs:210-223) — `method: String` + `payload: serde_json::Value` + `session: Option<SessionToken>`. The wire framing belongs above this layer; `dispatch_ipc` operates on the already-parsed shape, following the `benten_engine::thin_client` "wire framing is above this module" precedent.
- **`IpcResponse`** (lib.rs:228-232) — `payload: serde_json::Value`. No principal information returned; the webview already knows its principal from the session token.
- **`IpcError`** (lib.rs:239-270) — 4 variants, `#[non_exhaustive]`:
  - `MethodNotInAllowlist { method }` — T3 rung 1 reject.
  - `CapabilityNotInManifest { method, cap }` — T3 rung 2 reject.
  - `SessionResolve(#[from] ThinClientSessionError)` — wraps the shape (b) thin-client error type, preserving diagnostic surface continuity across shapes.
  - `MissingSession` — non-bootstrap invocation without a token (bridge attached but no `request.session`).

  `IpcError::error_code()` (lib.rs:277-289) maps each variant to a stable `ErrorCode`. The three non-session variants reuse `ErrorCode::ThinClientHandshakeInvalid` — same semantic class as a thin-client handshake against an unknown surface — so no new ErrorCodes were minted at this wave.

### `lib.rs` §4 — Manifest envelope + session bridge + `TauriRenderer`

- **`AdminUiManifest`** (lib.rs:303-330) — minimal projection of the full plugin manifest schema (`benten_platform_foundation::PluginManifest` from G24-D) containing only the `requires` envelope's granted cap-scope set. `grants_cap(cap)` returns true for empty cap (the no-cap-required sentinel) or any scope in `granted_caps`. Forwards-compat: the full manifest may be passed in later with extra fields ignored.

- **`InProcessSessionBridge`** (lib.rs:345-399) — owns an `Arc<DidKeyedSession>` from `benten-engine`. The cross-protocol-contract br-r1-14 surface: shape (b) browser-tab and shape (c) embedded-webview share the SAME `DidKeyedSession` cryptographic state machine; only the wire transport is swapped (HTTP for (b); in-process IPC for (c)). `transport()` returns `Transport::Ipc`. `resolve(token, presented_origin)` delegates to `DidKeyedSession::resolve` with `"tauri://localhost"` as the canonical synthetic origin — the same value the handshake was minted against. Origin recheck is per-request (Family F1 gap #2 mid-session defense). The `tests/in_process_ipc_session_token_contract_matches_thin_client.rs` pin asserts the byte-shape identity end-to-end + the per-request origin recheck rejects hostile origins.

- **`TauriRenderer`** (lib.rs:412-555) — the public surface. Composes an `IpcAllowlist` + `AdminUiManifest` + optional `InProcessSessionBridge`. Three constructors: `new_with_manifest(manifest)`, `with_bridge(bridge)` (builder), and `ipc_method_allowlist()` (static accessor for the drift-detector).

  **`dispatch_ipc(request)`** (lib.rs:502-554) is the T3 defense composition:

  1. **Rung 1 — allowlist filter.** `method_permitted` check BEFORE any payload parse, so attacker-crafted payloads can't pivot through a forbidden method. Rejects with `MethodNotInAllowlist`.
  2. **Rung 2 — cap binding.** Look up `required_cap_for_method`; manifest's `grants_cap` consulted. Empty cap auto-admits. Rejects with `CapabilityNotInManifest { method, cap }`.
  3. **Session resolution (br-r1-14).** Only fires when a bridge is attached. Token absent → `MissingSession`. Token present → `bridge.resolve(token, "tauri://localhost")` resolves to the principal DID (origin recheck + expiry check fire inside `DidKeyedSession::resolve`).

  Past the three rungs the method-specific handler lives in the integrator binary's Tauri command handler, which calls back into engine facade methods with the resolved principal. This crate returns an empty success envelope (`payload: Null`); the integrator overwrites `payload` with the real response. CSP (rung 3 of the T3 composition) does NOT fire here — it's a load-boundary defense via `webview_csp_header`.

- **`impl Renderer for TauriRenderer`** (lib.rs:566-579) — `render(&self, _output)` returns `Ok(())` and the integrator binary's Tauri command handler mounts the materializer output via Tauri 2.x's `emit` API. This crate stays runtime-agnostic; the actual emit call lives in the integrator. `backend_name()` returns `"tauri-2.x"`.

- **`_assert_renderer_object_safety`** (lib.rs:589-593) — `#[doc(hidden)]` compile-time assertion that `TauriRenderer: Renderer + Send + Sync`. Used by the verso swap-readiness pin to prove the trait doesn't leak Tauri-2.x-specific associated types.

- **`ipc_method_cap_bindings() -> BTreeMap<String, String>`** (lib.rs:598-604) — public accessor returning the bindings as a stable map; tests + operator audit surfaces use it without holding a `TauriRenderer` instance.

---

## 4. Public API surface

Grouped by intent:

**IPC allowlist + cap-binding (T3 defense rung 1 + 2).** `IPC_METHOD_NAME_ALLOWLIST` (const) / `IPC_METHOD_CAP_BINDING` (const) / `IpcAllowlist::canonical` / `method_permitted` / `methods` / `required_cap_for_method` / `ipc_method_cap_bindings` free function.

**CSP (T3 defense rung 3).** `WEBVIEW_CSP_HEADER` (const) / `TauriRenderer::webview_csp_header`.

**IPC envelopes + errors.** `IpcRequest` / `IpcResponse` / `IpcError` (4 variants, `#[non_exhaustive]`) / `IpcError::error_code`.

**Manifest envelope.** `AdminUiManifest::with_caps` / `AdminUiManifest::default` / `grants_cap`.

**In-process session bridge (br-r1-14).** `InProcessSessionBridge::new` / `transport` / `resolve` / `session`.

**`TauriRenderer`.** `new_with_manifest` / `with_bridge` / `allowlist` / `manifest` / `dispatch_ipc` / `webview_csp_header` / `ipc_method_allowlist` static. Plus `Renderer` trait impl: `render` / `backend_name`.

---

## 5. Tests inventory

9 test files; 910 LOC of test surface against 640 LOC of source. The crate is correctness-pinned end-to-end on the three T3 defense rungs:

- **`arch_n_benten_renderer_tauri_dep_direction.rs` (153 LOC)** — arch-N regression-guard. Reads its own Cargo.toml and asserts `benten-renderer-tauri` is not a dep of `benten-engine` / `benten-platform-foundation` / `benten-graph` (reverse-coupling forbidden); permits `benten-engine` as upstream (engine-extension trust posture). Multi-finding pin closing r4-arch-3.
- **`compromised_webview_cannot_escalate_to_native_filesystem.rs` (125 LOC)** — **T3 LOAD-BEARING substantive end-to-end.** Simulates XSS-amplified IPC against `fs:write` (rejects at rung 1) + `engine.call_as` with `graph:write` withheld from manifest (rejects at rung 2) + happy-path regression-guard.
- **`in_process_ipc_session_token_contract_matches_thin_client.rs` (102 LOC)** — br-r1-14 cross-protocol contract pin. Two tests: byte-shape identity (shape (b) `SessionToken` round-trips through shape (c) bridge) + per-request origin recheck rejects hostile origin with `ThinClientSessionError::OriginMismatch`.
- **`ipc_allowlist_rejects_unknown_method.rs` (80 LOC)** — T3 rung 1 pin.
- **`ipc_method_invocation_requires_manifest_cap.rs` (79 LOC)** — T3 rung 2 pin. Negative arm + companion §3.6b regression-guard (would-have-succeeded-with-cap) + `ui.notify` no-cap-required arm.
- **`ipc_method_name_stability_drift_detector.rs` (70 LOC)** — gap #1a closure. Drift-detector coupling the const allowlist to `docs/public-api/benten-renderer-tauri.json` baseline.
- **`tauri_runtime_verso_swap_readiness_compile_test.rs` (92 LOC)** — gap #1b closure (br-r1-9). Compile-test proving the `Renderer` trait surface is transport-agnostic: a verso-shape mock impl compiles against the SAME trait. If a Tauri-2.x runtime type leaked through, this mock would fail to compile.
- **`three_rung_baked_in_17_defense_extension_pin.rs` (146 LOC)** — gap #1c closure (br-r1-4 + br-r1-13). Extends PR #166's 3-rung baked-in-#17 defense to this crate's wasm32 build. Rung 1 wasm-objdump forbidden-prefix sweep (runtime-arm); rung 2 Cargo.toml dep-name assertion; rung 3 module-doc engine-extension reference grep.
- **`webview_csp_locked_no_unsafe_eval.rs` (63 LOC)** — T3 defense rung 3 LOAD-BEARING pin (br-r1-11).

Three inline `mod tests` units in lib.rs (allowlist construction round-trip + CSP forbidden-token check + cap-binding-covers-every-allowlisted-method).

---

## 6. Benches inventory

No `benches/` directory present. The crate is correctness-pinned; IPC dispatch is shallow (membership lookup + small string compare). If latency benching becomes load-bearing, it would live downstream in the integrator binary where the surface composes with real engine facade calls.

---

## 7. Thin-engine + composable-graph philosophy check

The crate sits clean. As an engine extension per CLAUDE.md #19 the trust boundary is explicit: compile-time linked + cargo + code review, NOT UCAN or `read_node_as`. Specific observations:

**Well-respected surfaces.**

- **Three deployment shapes (CLAUDE.md #17) preserved structurally.** Shape (c) embedded-webview reuses shape (b)'s `DidKeyedSession` cryptographic state machine via `InProcessSessionBridge`. No fork of the auth contract; only the transport tag (`Transport::Ipc` vs `Transport::Http`) differs. The br-r1-14 pin enforces the byte-shape identity end-to-end.
- **Renderer trait surface transport-agnostic.** Verso swap-readiness pin (br-r1-9) is a compile-test, not a runtime assertion — the cheapest possible enforcement of the swap-readiness contract: if a Tauri-2.x runtime type leaked through, the verso-mock impl wouldn't compile. The pin documents the swap target shape too (`VersoWebview` / `VersoIpcChannel` placeholder structs).
- **T3 defense composition has independent rungs.** Rung 1 (allowlist) is `BTreeSet` membership; rung 2 (cap-binding) is manifest envelope consultation; rung 3 (CSP) is webview-load-boundary. The composition is exercised end-to-end at `compromised_webview_cannot_escalate_to_native_filesystem.rs`. No single rung's correctness depends on the others — defense-in-depth from independent angles.
- **No new ErrorCodes minted at G24-E.** The four T3 surface errors reuse `ThinClientHandshakeInvalid` per the cross-protocol contract — same semantic class as a thin-client handshake against an unknown surface. Avoids ErrorCode-catalog inflation for what is structurally the same handshake-reject family.
- **Drift-defense pinned to public-api baseline.** The IPC method-name allowlist is coupled byte-for-byte to `docs/public-api/benten-renderer-tauri.json`; adding a method without a baseline update + manifest review trips the drift-detector test. Silent IPC surface expansion is the manifest-bypass risk this guards against.

**Worth flagging — potential pluggability friction (CLAUDE.md #19 perspective).**

- **`tauri-runtime-verso` swap is a compile-test today; the actual swap shape is not exercised at runtime.** The mock module documents the surface shape but is `#[allow(dead_code)]`. If `tauri-runtime-verso` lands a different async/IPC contract (e.g. Verso's channel returns a different `Future` shape than Tauri 2.x's), the compile-test might pass while the runtime impl needs significant adaptation. Not a defect — the compile-test is the load-bearing assertion at the trait-surface level — but the post-Verso-GA wave will need a real impl to surface any runtime-shape skew. Tracked in `docs/future/phase-3-backlog.md §15`.
- **`AdminUiManifest` is a minimal projection.** Forwards-compat to the full `benten_platform_foundation::PluginManifest` shape works because extra fields are ignored, but if the full manifest grows shape-changes that affect the `requires` envelope semantics (e.g. scoped caps, time-bounded caps), the projection here would need a parallel update. The projection is small enough (one field) that this is a cheap walk, but the coupling exists.
- **The cargo `tauri` feature is held by the integrator binary, NOT this crate.** Keeps the workspace `Cargo.lock` light when only the IPC pipeline is exercised (the bulk of testing), but means CI must explicitly enable the feature for the e2e webview workflow. The path-filtered workflow `.github/workflows/admin-shell-e2e.yml` (path-filtered at PR #249 — only fires when admin-shell paths change) carries the feature-on build; non-admin-shell PRs don't pay the Tauri compile cost.

**The integrator binary holds the real Tauri 2.x crate.** `tools/benten-admin-shell/src/main.rs` carries the `tauri` feature-gated `tauri_boot::run` path that wires the real `Builder::default().invoke_handler(...).run(generate_context!())` pipeline; the three Tauri commands dispatch to `AdminShellState::dispatch` one-to-one. Default-mode boot (feature OFF) prints a launch summary of the IPC method-cap-binding map + locked CSP header for operator audit without needing a full Tauri build. See `docs/ADMIN-UI.md §4.6`.

**Manifest-envelope chain validation is upstream, not here.** Compromise #26 (PARTIALLY CLOSED at R4b-FP-1 Seam 3) lives at the `apply_atrium_merge` boundary in `benten-engine`, AFTER the per-row cap-revocation check (Layer-3 manifest-envelope refinement on top of Layer-1 revocation defense). This crate's per-method cap binding fires at IPC ingress, NOT at sync merge ingress — they are distinct defense surfaces composing at different boundaries. No coupling drift between them.

---

## 8. Phase 4-Meta + post-v1 expectations

Several knowable forward-looking surfaces touch this crate:

**Phase 4-Meta substantive `ProductionManifestEnvelopeRechecker` adapter.** Per `docs/future/phase-4-backlog.md §4.36`, the production adapter that consults `PluginLibrary` + `UserDidRegistry` + invokes `manifest_envelope_chain_validation::validate_chain_with_manifest_envelope` lands at Phase-4-Meta. This crate's per-method cap-binding stays unchanged — sync-ingress and IPC-ingress are different boundaries — but the manifest envelope shape consumed here (`AdminUiManifest.granted_caps`) may grow to mirror the substantive adapter's envelope query results.

**`tauri-runtime-verso` swap (post-v1).** When Verso matures, a sibling `benten-renderer-tauri-verso` crate (per the swap-target documentation in the compile-test) implements the same `Renderer` trait. Engine binary swap is a one-line `Cargo.toml` change. The IPC allowlist + cap-binding + CSP machinery doesn't need to fork — those concerns are above the webview-runtime swap boundary.

**Admin UI v0 method-name allowlist growth.** The 8-method allowlist at HEAD is the v0 minimum-viable surface. Adding methods (e.g. real `engine.put_node_as`, `plugin.uninstall`, `plugin.fork`) requires explicit baseline update + admin UI v0 manifest review per the drift-detector contract. The `docs/public-api/benten-renderer-tauri.json::_anticipated_method_set` array is the authoritative review surface — each addition is a manifest-bypass-risk review point.

**Tauri-shell-vs-browser-tab is a deployment choice, not an architectural shape change.** Per CLAUDE.md #17, the three deployment shapes are first-class siblings, not a primary + fallback pair. Future shape additions (e.g. Electron fallback if Tauri 2.x webview-variance bites) compose against the same `Renderer` trait + `DidKeyedSession` bridge contract. The shape isn't crate-bound — it's trait-impl-bound — so a 13th crate `benten-renderer-electron` would be a sibling, not a refactor.

---

## 9. Open questions / unresolved internals

A handful of things worth surfacing for retrospective:

1. **Single-file crate, 640 LOC.** Could split into modules (`allowlist.rs` / `session_bridge.rs` / `renderer.rs`) but at this size the single-file shape is honest about the crate's narrow surface. Worth revisiting if Phase-4-Meta grows the surface materially.

2. **`IpcResponse::payload` always returns `serde_json::Value::Null` from `dispatch_ipc`.** The integrator binary overwrites it with the real response after passing the three rungs. This shape is a clean separation of concerns (this crate enforces defense; integrator returns data) but might surprise a fresh reader who expects `dispatch_ipc` to be the end-to-end handler. The module doc + `dispatch_ipc` docstring narrate this; flagging because the assertion lives in commentary, not the type.

3. **`presented_origin` hardcoded as `"tauri://localhost"`.** `InProcessSessionBridge::resolve` uses this synthetic origin for every Tauri-shape request, matching what `emit_challenge` was minted against. If Tauri 2.x's IPC framing exposes a real origin string the bridge could thread through, that would be a tighter binding — at HEAD it's hardcoded inside `TauriRenderer::dispatch_ipc` step (3). Documented in the bridge docstring.

4. **`MultiSigSurface`-shaped extension point doesn't exist here.** Unlike `benten-id` where future signature-scheme extensions have an extension trait, this crate's `Renderer` trait IS the extension point — concrete impls are siblings. The `Renderer` trait lives in `benten-platform-foundation`, not here, which is correct (the trait shouldn't move with each impl). Flagging because a fresh reader looking for "how do I add a renderer" should look at `benten-platform-foundation::Renderer`, not in this crate.

5. **`_assert_renderer_object_safety` proves trait-object compatibility but doesn't enforce it at call sites.** The renderer is consumed by value in the integrator (`tools/benten-admin-shell/src/lib.rs`'s `AdminShellState`), not as `Box<dyn Renderer>`. The assertion is a forward-compat guard for callers who want trait-object dispatch later. Not a defect; documenting because the `Send + Sync` bounds are load-bearing for trait-object use but not exercised at HEAD.

6. **No wasm32 build-target guard at this crate's level.** The forbidden-prefix sweep + Cargo.toml dep-name assertion are the load-bearing wasm32 thinness guards (`three_rung_baked_in_17_defense_extension_pin.rs`). The crate compiles natively today and the integrator binary is the place wasm32-target builds happen. If a future contributor added a native-only dep here (e.g. for some shell-side helper), the dep-name pin would catch the known-bad set but not a brand-new native-only crate. CI authoritative; flagging because the load-bearing position makes a wasm32-incompatible regression high-blast-radius.

7. **`webview_csp_header` is `&'static str`.** Locked at compile time, which is the security-property the test pin enforces. Future CSP customization (e.g. per-deployment CSP variants) would need a constructor parameter — at HEAD the locked-at-compile-time shape is the substance of the defense. If Phase 4-Meta wants per-environment CSP, the migration is a builder method + a new test pin to ensure no relaxation slips in.
