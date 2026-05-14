# Admin UI v0

Phase 4-Foundation's admin UI is the first instance of the plugin architecture: a content-addressed shareable subgraph the user installs via signed manifest envelope, that lets them create + edit + share workflows + composed views through a browser-tab or Tauri 2.x embedded-webview surface. This document is the user-facing walkthrough + the engineer-facing spec for the v0 surface.

Cross-references: D-4F-1 (FULL plugin manifest scope) + D-4F-4 (browser-wasm32 + Tauri 2.x as swappable Renderer backends) + CLAUDE.md #18 (app-level plugins are subgraphs) + CLAUDE.md #19 (engine extensions are Rust crates — `benten-renderer-tauri`).

---

## §1. Admin UI v0 IS a plugin

Admin UI v0 is content-addressed, importable, shareable across Atriums. It ships AS the first app-level plugin per CLAUDE.md baked-in #18 — not as a separate "platform admin" surface with its own trust posture. Same plugin-DID + UCAN envelope + manifest-`requires`/`shares` machinery any other plugin uses.

This is intentional: by making admin UI v0 a plugin, the manifest schema gets dogfooded by the platform's most-trusted-but-still-app-level surface. If admin UI works through the plugin envelope, arbitrary third-party plugins will too.

**What this rules out:**
- Admin UI as engine extension per CLAUDE.md #19. Plugin-arch-r1-12 + plugin-arch-r1-17 surfaced this explicitly. Engine extensions are Rust crates compile-time linked; admin UI is JS+wasm in a browser tab or webview — it's app-level.
- A "system" trust posture distinct from regular plugins. Admin UI's privileges come from caps the user grants at install (per Layer 2 manifest envelope); not from being baked into the engine.

The renderer-backend impls (`BrowserRender` in `benten-platform-foundation` + `TauriRenderer` in `benten-renderer-tauri`) ARE engine extensions per CLAUDE.md #19. They sit at the engine boundary, are compile-time linked, are trusted because the user compiled the engine. Admin UI consumes them via the `Renderer` trait surface.

---

## §2. 4-category navigation IA

Per post-R1-triage ratification #4: 4-category navigation over the unified subgraph substrate (workflow ↔ plugin ↔ schema = same subgraph shape per D-4F-14):

| Category | Surface |
|---|---|
| **Plugins** | Plugin library subgraph; installed-versions timeline; active-version highlight; cap-grant management; multi-device sync visibility |
| **Workflows** | User-authored workflow handlers (composes from 12 primitives + 8 typed-field-Node labels via drag-drop primitive-picker) |
| **Content Types** | Schemas (the Phase 4-Foundation typed-field-Node subgraph shape per SCHEMA-DRIVEN-RENDERING.md vocabulary) |
| **Views** | Composed views (user picks anchor pattern + projection; emits via generalized IVM Algorithm B kernel per D-4F-2) |

The substrate is unified: all four categories are subgraphs at the engine layer. The user-facing IA distinguishes them by usage intent + lifecycle affordances; the underlying machinery is identical.

---

## §3. User flows

### §3.1 Install consent flow (Layer 2)

When the user receives a plugin (out-of-band content-addressed-share over Atriums in v0; decentralized registry → Phase 4-Meta), admin UI surfaces a manifest review dialog:

1. **Manifest display.** Plain English: "<plugin_name> wants to ..."
   - For each `requires` entry: plain-English description (e.g., "read your notes labeled `tag/work`", "use the time host-fn", "execute SANDBOX modules").
   - For each `shares` entry: plain-English delegation policy (e.g., "may share read-access for `notes:work` with any AI assistant plugin you install later" / "will NOT delegate any caps to other plugins").
2. **Per-cap-grant decline.** User can decline a single cap without aborting install (per ux-r1-2 BLOCKER closure). The plugin still installs; the declined cap is not granted; the plugin handles cap-absence gracefully (or fails at first use of the missing cap — handled by E_CAP_DENIED).
3. **Identity disclosure.** Manifest review shows the four identity concepts (per D-4F-12):
   - Content-CID + canonical bytes hash
   - Peer-DID of original author + verification status against `requires_plugin_authors` trust list
   - Plugin-DID that the **admin-UI caller mints + inserts into `PluginDidStore` before invoking `install_plugin`** (caller-mint-first contract per R6-FP-A; see `docs/PLUGIN-MANIFEST.md §3` "Plugin-DID minting protocol" for the 4-step caller sequence + `crates/benten-platform-foundation/src/plugin_lifecycle.rs::InstallContext::expected_plugin_did` rustdoc for the API contract). The engine never mints; the caller-mint-first contract makes Ed25519-derives-DID-from-key the structural defense against plugin-DID substitution at install.
   - Confirmation that user-DID will sign the install record (binding `manifest_cid` + `consenting_user_did` + `plugin_did_bytes` + `nonce`).
4. **Consent.** User clicks "Install" → user-DID signs InstallRecord + grants UCANs for accepted caps → plugin enters library + active reference updates. `install_plugin` Step 8 verifies `install_record.plugin_did == ctx.expected_plugin_did` AND `plugin_did_store.get(expected_plugin_did).is_some()`; either mismatch surfaces a typed `E_PLUGIN_INSTALL_RECORD_PLUGIN_DID_MISMATCH` or `E_PLUGIN_DID_HANDLE_NOT_PRE_INSERTED` to the admin-UI for plain-English display.

UX-acceptance (per ux-r1-2 BLOCKER closure): user can install a plugin in ≤3 clicks; user can decline a single cap without aborting install.

### §3.2 Update flow (cap-change-triggered consent)

Per post-R1-triage ratification #8 (cap-change-triggered fresh consent):

- **Silent within-lineage upgrade.** If `requires` of the new version is a strict subset of installed manifest, upgrade applies silently. User sees a non-blocking "Plugin updated: <plugin_name> v1.2 → v1.3" toast.
- **Re-consent on cap growth.** If `requires` GREW (any cap added or scope widened), full manifest review dialog fires for the new version. User can: (a) accept (re-install); (b) reject (stay on previous version); (c) decline specific new caps (per-cap-grant decline analogous to install flow).
- **Cross-fork merge.** If user is on a fork (their version is NOT a descendant of mainline-v2), cross-fork = user-initiated merge through same consent flow. Admin UI surfaces a 3-way merge view: "Mainline-v2 has changed in ways your fork doesn't have; would you like to merge?" with "show what changed" diff per node + "keep my changes" vs "accept mainline" toggle.
- **Pull-not-push notification.** Atrium-mesh announce-event fires `E_PLUGIN_NEW_VERSION_AVAILABLE` notification (no auto-pull); admin UI shows in-app notification. User must explicitly trigger the update check.

### §3.3 Fork flow (user edits a non-configurable plugin Node)

When the user edits a Node in an installed plugin that the plugin doesn't expose as configurable, admin UI prompts:

> "This creates a fork. Your edits will diverge from the plugin author's version."

Options:
- **Commit fork.** User's edits land on a new version-chain branch (per D-4F-14 DAG-shape versioning). Active reference updates to the fork tip.
- **Revert.** Discard the user's edits; remain on the unmodified plugin.
- **Propose upstream.** (Phase 4-Meta scope; v0 shows "coming in Phase 4-Meta" placeholder.)

### §3.4 Plugin library browse + version switch

"Plugins" tab houses the plugin library subgraph view:

- List of installed plugins (one row per plugin anchor).
- Per-plugin: name + active version + installed-versions timeline rendered as DAG-branching tree (per D-4F-14).
- **"Switch to this version"** button on any non-active version (updates active reference per ratification #2 — Loro Map per-device-keyed CURRENT).
- **"Remove this version"** button on any non-active version (keeps the version-chain history but drops the durable copy if no longer referenced).
- Active-version highlight + last-used timestamp.

### §3.5 Multi-device sync visibility

"Devices" sub-panel under "Plugins":
- List of user's full peers (laptop / phone-OS-app / desktop per CLAUDE.md #17).
- Per-device last-sync-time + per-plugin sync status.
- Conflict indicator when concurrent writes occur (resolved by Loro CRDT + DAG-shape versioning + cap-change-triggered consent on cross-fork).
- "This version active on <device X>" annotations per ratification #2 (per-device-local CURRENT).

### §3.6 Cap-grant management

"Plugins → <plugin name> → Capabilities" panel:
- Every UCAN cap held by this plugin, listed.
- Per-cap revoke button.
- "What stopped working" preview before revoking (best-effort UX — the plugin may handle the revocation gracefully or fail at next use).
- **Uninstall plugin** button (couples to G24-D-FP-1 cascade-revoke): cascades revoke ALL caps held by this plugin + cascades plugin-DID's own downstream UCAN delegations + deletes private namespace + removes library entry + terminates live subscriptions. Confirmation dialog shows the cascade scope before commit.

### §3.7 Workflow editor

Drag-drop primitive-picker:
- **Picker palette**: 12 primitive Node types + 8 typed-field-Node labels (per SCHEMA-DRIVEN-RENDERING.md vocabulary) as drag-targets.
- **Canvas**: drag primitives onto canvas; connect ports with wire-drag (port-to-port).
- **Invalid-graph feedback**: red-edge + inline-text on cycle / type-mismatch / cap-missing.
- **Schema-driven form generation**: forms generated from schema CIDs (via G23-A schema-compiler); user fills in field values; submit emits SubgraphSpec for `Engine::register_subgraph`.

### §3.8 View creator

Composed-view creator:
- User picks anchor pattern (e.g., `tag/work/*` / `version_current:<anchor_cid>`) + projection (subset of fields to materialize).
- View emits via G23-0a + G23-0b generalized IVM Algorithm B kernel (subgraph-shaped view definition per D-4F-2).
- Live preview via G23-B materializer with ≤200ms latency for primitive-vocab schemas; <1s for typical schema; degrade-to-on-blur if budget exceeded (ux-r1-16).

### §3.9 Meta-plugin packaging

"Plugins → New → Meta-plugin":
- Pick sub-plugins from library (composition recursive per D-4F-14).
- Admin UI runs cycle-check at packaging time (per post-R1-triage Q2 ratification — meta-plugin cycle detection AS REJECTION; surfaces `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED` if cycle).
- Merge requires/shares from sub-plugins into the meta-plugin manifest.
- User signs (user-DID) + commits.

### §3.10 Provenance / rotated-key warning

When a plugin's peer-DID has a rotated key (per benten-id RotationLog + ratification #6 RotationLog MVP), admin UI surfaces a yellow-banner on the plugin's detail page:

> "This plugin was signed by a key the publisher has rotated. Your installation is unaffected; you may want to check for an updated version signed by their new key."

Action button: "Check for new versions" (triggers Atrium pull for newer versions of this plugin).

### §3.11 Out-of-band share affordance

"Plugins → <plugin name> → Share with Atrium friend":
- Generates a copy-paste handshake-token (CID + peer-DID + optional metadata).
- Recipient pastes into their admin UI → install flow fires.
- No registry / no DHT involved (per ratification #3 — decentralized registry → Phase 4-Meta).

### §3.12 First-run onboarding

"Welcome — install your first plugin":
- 2 starter-plugin pointers (`benten-starter-notes` + `benten-starter-tasks` — TBD).
- Skip-and-explore link.

---

## §4. Deployment shapes

Per Ben D-4F-4 + CLAUDE.md #17:

| Shape | Renderer backend | Trust posture | Use case |
|---|---|---|---|
| **Browser tab (thin client)** | `BrowserRender` (browser-wasm32) | Full peer is user's own machine; browser tab is a view INTO it | Default v0; works everywhere with a browser |
| **Tauri 2.x embedded-webview** | `TauriRenderer` (in `benten-renderer-tauri`) | Same as above + native shell can hold engine-level caps | Desktop dogfood; v0 ships minimal scope per post-triage Q4 (full T3 defenses → Phase 4-Meta) |

The 3-rung baked-in #17 defense extends to both renderer crates (br-r1-4 + br-r1-13): wasm32-objdump forbidden-prefix list updated; feature-graph-closure test extended; `tauri-runtime-verso` swap-readiness validated.

### §4.1 Bundle size budget

Per br-r1-3: admin UI v0 bundle ≤600KB gzipped with code-splitting strategy:
- Workflow editor (sub-feature; dynamic-import boundary)
- View creator (sub-feature; dynamic-import boundary)
- Plugin browser (sub-feature; dynamic-import boundary)

CI workflow `admin-ui-v0-bundle-size.yml` at G26-B enforces.

### §4.2 CSP directives

Per br-r1-11 (both browser-tab and embedded-webview deployment shapes):
- `script-src 'self' 'wasm-unsafe-eval'`
- `connect-src 'self' tauri://*`
- `style-src 'self'`
- `font-src 'self'`
- `default-src 'none'`

### §4.3 IPC protocol (embedded-webview)

Per br-r1-14: same `DidKeyedSession + SessionToken` contract as thin-client HTTP/fetch (G24-F), transport-swapped to in-process channel. Native Tauri shell holds engine-level capabilities; admin UI runs as plugin per CLAUDE.md #18; IPC method allowlist enforced in the native shell (`IpcAllowlist::method_permitted`).

### §4.4 WinterTC forbidden-API list

Per br-r1-8: admin UI v0 wasm bundle must not reference DOM-only APIs / FormData / fetch-relative URLs. CI guard at G26-B WinterTC-compat check.

### §4.5 IndexedDB scope

Per br-r1-7 grep-assert pin: admin UI v0 IndexedDB writes ONLY snapshot cache and manifest store. Blocks UCAN bytes / plugin secrets / direct sync state from landing in browser storage. CAPS stay at full peer; admin UI delegates via UCAN.

### §4.6 Integrator binary (`benten-admin-shell`)

Per R6-FP-E (closes br-r6-r1-3 MAJOR full path-a — both halves shipped): the production caller for `TauriRenderer` + `InProcessSessionBridge` lives at `tools/benten-admin-shell/` as the `benten-admin-shell` bin crate. The crate splits cleanly along four lines:

- **`src/lib.rs`** — `AdminShellState` composes `TauriRenderer` + `InProcessSessionBridge` + the canonical admin-UI-v0 manifest envelope (`admin_ui_v0_canonical_manifest`). `dispatch(IpcRequest) -> Result<IpcResponse, IpcError>` is the exact code path a real Tauri 2.x command handler invokes one-to-one.
- **`src/main.rs`** — default-mode boot path prints a launch summary (IPC method-cap-binding map + locked CSP header). With the `tauri` cargo feature enabled, `tauri_boot::run` wires the REAL Tauri 2.x `Builder::default().invoke_handler(...).run(generate_context!())` pipeline; the three Tauri commands dispatch to `AdminShellState::dispatch` one-to-one. Feature is OFF by default to keep the workspace `Cargo.lock` light when only the IPC pipeline is exercised; the CI workflow `.github/workflows/admin-shell-e2e.yml` runs the feature ON.
- **`tests/e2e_admin_shell_ipc.rs`** — substantive default-mode end-to-end pin (pim-2 §3.6b production-arm + observable-consequence + would-FAIL-if-no-op'd) exercises 9 paths through the integrator: happy-path dispatch for every allowlisted method, T3 rung 1 unknown-method reject, T3 rung 2 missing-cap reject, missing-session reject, origin-mismatch reject, session-expired reject, handshake replay reject, CSP header canonical pin, single-`DidKeyedSession`-instance reuse across handshake + dispatch.
- **`tests/e2e_webview_smoke.rs`** (under `tauri` feature) — webview-driven `tauri-driver` E2E via fantoccini-rustls WebDriver client: spawns `tauri-driver` subprocess, launches the admin-shell binary, drives a real Tauri command-invoke roundtrip through the embedded WebView2 / WKWebView / WebKit2GTK runtime, asserts CSP blocks an `eval()` execution attempt + the IPC channel returns the expected response. Linux substantive (WebKit2GTK under Xvfb); macOS build-only smoke per upstream Tauri WKWebView WebDriver limitation; Windows deferred per `docs/future/phase-4-backlog.md §3.6` upstream-migration carry.
- **`webview-assets/`** — `index.html` + `style.css` + `bootstrap.js` loaded by the embedded webview when `tauri_boot::run` fires. The `<meta http-equiv="Content-Security-Policy">` directive in `index.html` is asserted byte-equivalent to `WEBVIEW_CSP_HEADER` by `tests/webview_assets_csp_meta_matches_rust_constant.rs` (defense-in-depth duplicate; T3 rung 3 stays armed even if the embedding context strips the server-set header).

Both halves of br-r6-r1-3 are CLOSED at HEAD per HARD RULE rule-12 (path-a-FULL). Only the Tauri upstream migration carries (gtk-rs → GTK4; proc-macro-error → proc-macro-error2; etc.) remain as `docs/future/phase-4-backlog.md §3.6` advisory-window items.

---

## §5. Renderer trait surface

Per arch-r1-16 + br-r1-9: `Renderer` trait at `crates/benten-platform-foundation/src/renderer.rs`:

```
pub trait Renderer: Send + Sync {
    fn render(&self, materializer_output: &MaterializerOutput) -> Result<Bytes, RenderError>;
    // ... transport-agnostic methods only;
    // transport concerns (DOM mutation, IPC method invocation) live in concrete impls.
}
```

`tauri-runtime-verso` swap-readiness committed: methods are minimal + transport-agnostic; Verso is post-v1 swap target.

---

## §6. Latency budget

Per ux-r1-16: composed-view creator live-preview latency ≤200ms for primitive-vocab schemas; <1s for typical schema; degrade-to-on-blur if budget exceeded.

---

## §7. Error UX consistency

Per ux-r1-15: typed-ErrorCode → user-facing copy table at `docs/ERROR-CATALOG.md`; admin UI surfaces typed-error code in dev-mode + user-friendly copy in production. Companion at ERROR-CATALOG.md retense.

---

## §8. Accessibility + i18n (carries to Phase 4-Meta)

Per ux-r1-17 + ux-r1-18 (Phase-4-Meta carries; `docs/future/phase-4-backlog.md §3.4`):

- **a11y baseline at v0**: keyboard-nav + alt-text + sufficient-contrast. Full WCAG 2.1 AA certification at Phase 4-Meta.
- **i18n at v0**: hardcoded en-US strings via i18n-ready helper (e.g., `t("install_plugin")` returning hardcoded string in v0) so later extraction is mechanical. Full i18n scope deferred to Phase 4-Meta.

---

## §9. Dogfood paths (exit-criterion 5)

Per ux-r1-1 BLOCKER closure; each path carries observable user-moment + success state + failure state + acceptance pin at `crates/benten-engine/tests/dogfood_path_<a..f>_ux_acceptance.rs`:

(a) Create a workflow ─ build in ≤5 clicks via primitive-picker drag-drop.
(b) Create a composed view ─ pick anchor + projection in ≤4 clicks; live preview ≤200ms.
(c) Multi-device sync leg ─ change propagates ≤3s on loopback; "Devices" panel shows last-sync.
(d) Revoke-cap mid-session ─ "Capability revoked" toast; affected view re-renders to redacted state (Node-granularity per ratification #7).
(e) Install admin UI on 2nd device ─ install consent screen ≤3 clicks; per-cap decline supported.
(f) Install a 2nd plugin ─ same flow as (e); install record signed by user-DID per D-4F-12.

---

## §9.1 G24-A canary deliverable state (2026-05-13)

Phase 4-Foundation G24-A wave-6 canary ships the **engine-substrate arm** of the admin UI v0 surface. The 4-category nav substrate + materializer-pipeline consumer wiring + dogfood-path engine-side arms are LIVE; the browser-side admin UI components (workflow editor drag-drop, view creator UI, install consent dialog) land at G24-B + G24-C wave-6b + G24-D wave-7. Dogfood UX arms — click-budgets + live-preview latency + multi-device-sync wall-clock + revoke toast surfacing — carry to wave-9 dogfood gate (named at `docs/future/phase-4-backlog.md §2`).

### G24-A canary shipped at this wave

| Component | Location | Status |
|---|---|---|
| Admin UI v0 module | `crates/benten-platform-foundation/src/admin_ui_v0/mod.rs` | LIVE (524 LOC) |
| 4-category nav constants (`NAV_CATEGORIES`, `Category`) | same module | LIVE |
| `build_admin_ui_v0_subgraph` + `build_category_route_subgraph` | same module | LIVE — composes from 12 primitives only |
| `render_category_content_allow_all` + `render_category_content` | same module | LIVE — `Materializer` trait consumer |
| `Subscriber::for_category` seam | same module | LIVE — patterns route to `on_change_as_with_cursor` |
| Engine→Materializer adapter | `crates/benten-platform-foundation/tests/common/admin_ui_v0_engine_adapter.rs` | LIVE — test-side adapter shape |
| Defense-in-depth SANDBOX banned-host-fn pin (mr-4) | `crates/benten-platform-foundation/tests/materializer_defense_in_depth_rejects_banned_sandbox_host_fn_for_handcoded_spec.rs` | 4 sub-tests pass |
| End-to-end dual-gate pin (mr-3) + propagation pin (mr-5) + invocation-count pin (mr-8) | `crates/benten-platform-foundation/tests/admin_ui_v0_materializer_reactive_update_propagates_through_engine_on_change_as_with_cursor.rs` | 4 sub-tests pass |
| Engine-side shell pins (7) | `crates/benten-engine/tests/admin_ui_v0_*.rs` | Un-ignored + substantively LIVE |
| Dogfood-path engine-substrate arms (6) | `crates/benten-engine/tests/dogfood_path_<a..f>_ux_acceptance.rs` | Un-ignored + substantively LIVE |

### G24-A canary punt-list (lands at later waves; named at `docs/future/phase-4-backlog.md §2`)

- Click-counter test harness + live-DOM workflow editor → **G24-B wave-6b**
- Composed-view creator live-preview latency p50/p99 measurement → **G24-C wave-6b**
- Multi-device sync ≤3s loopback round-trip + Devices sub-panel → **wave-9 dogfood gate**
- "Capability revoked" user-visible toast UX → **G24-C wave-6b + wave-9 dogfood gate**
- ≤3-click install-consent flow + plain-English manifest display → **G24-D wave-7 + wave-9 dogfood gate**
- User-DID install-record signing UX → **G24-D wave-7 + wave-9 dogfood gate**

---

## §10. Cross-references

- **CLAUDE.md baked-in #18** — three-layer consent (canonical)
- **CLAUDE.md baked-in #19** — engine extensions; `benten-renderer-tauri` is a 12th crate engine extension
- **CLAUDE.md baked-in #17** — full peer vs thin compute surface vs embedded-webview deployment shapes
- [`PLUGIN-MANIFEST.md`](PLUGIN-MANIFEST.md) — manifest schema admin UI reviews + signs
- [`SCHEMA-DRIVEN-RENDERING.md`](SCHEMA-DRIVEN-RENDERING.md) — workflow editor's form generation consumes this
- [`ARCHITECTURE.md`](ARCHITECTURE.md) §"Plugins and engine extensions" — workspace shape + crate boundaries
- [`SECURITY-POSTURE.md`](SECURITY-POSTURE.md) "Plugin trust model" — security narrative
- [`ERROR-CATALOG.md`](ERROR-CATALOG.md) — admin UI surfaces typed errors with user-facing copy

---

(Phase-4-Foundation companion doc lands at G24-A canary per `feedback_post_fix_doc_coupling_preflight.md` §3.5b HARDENED + meth-r1-7 companion-with-canary discipline.)
