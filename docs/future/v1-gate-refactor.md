# v1 Gate Refactor — Planning Doc

**Status (2026-05-06):** In-flight planning, not yet ratified. The contents of this doc supersede the Phase 4 + Phase 5 framing currently in [`FULL-ROADMAP.md`](../FULL-ROADMAP.md). Ratification trigger: `phase-3-close` tag + v1-assessment-window opens. Once ratified, this doc drives a sweep of `docs/FULL-ROADMAP.md` + `docs/VISION.md` + `docs/PRIMER.md` + `CLAUDE.md`, after which this doc retires.

**Why this doc exists:** The current `FULL-ROADMAP.md` Phase 4 framing (`Thrum Migration` with `3,200+ behavioral tests pass`) was based on an aspirational read of Thrum. Thrum was the prior codebase whose concept motivated building Benten properly in Rust — NOT a tested production CMS waiting to migrate. Phase 4 needs rewriting; Phase 5 contents need restructuring; the v1 milestone gate needs sharpening. The reshape is significant enough to warrant a dedicated planning artifact rather than direct edits to canonical docs while Phase 3 is still in flight.

---

## 1. The actual shape: Benten is a platform, not just an engine

What v1 ships is a **runtime + admin/dev surface + module ecosystem + sharing surface** — a platform you install, use, build tools in, and share those tools across your network. The CMS / AI assistant / Gardens / Credits aren't core platform features; they're **use cases** of the platform. Apps that grow out of using it.

The specific user story v1 enables: *"I want to be able to start using it to easily build and share the tools through my network."* The plugin ecosystem, full materializer pipeline, schema-driven rendering complete surface, and module ecosystem at scale all feel central to the engine because — as engine surface for installing/using/sharing — they are. They're not application-layer concerns; they're how you USE what we're building.

---

## 2. v1 contents: three layers, all part of the same shippable platform

### Layer 1 — Engine (Phases 1+2a+2b+3)

Already done or in flight. Core engine, evaluator, SANDBOX/WASM, P2P sync via Atriums, durable identity (`benten-id` with Ed25519 + DIDs + UCANs + VCs + multi-sig + DID rotation + device attestation), 14 invariants enforced, 12 primitives executable.

### Layer 2 — Benten Platform Surface (was old Phase 5; now part of v1)

The thing you install + run + use. Contents:

- **Admin UI** — the dev/admin surface you'd open after installing Benten. Full self-composing version: the admin UI is composed from engine primitives ("the ability to compose UI from engine primitives kinda like Thrum intended"). Configurable from the graph itself.
- **Plugin manifest format** — declarative module manifest (extends the Phase-2b `ModuleManifest{name, version, modules, migrations, signature}` shape with the dev-facing wrapper + dependency declaration shape).
- **Decentralized self-discovered registry** — NOT a centralized third-party registry. Tools published into Atrium peer groups (context-scoped) or publicly; anyone in your network(s) can use or reshare. UI built on top of existing P2P sharing functionality (Atriums substrate from Phase 3) using the platform's own UI composition primitives.
- **Schema-driven rendering complete** — full surface, not a thin slice. Schema → subgraph compiler → materializer → render output. New content types render without custom Rust per type.
- **Materializer pipeline** — the renderer that walks composed subgraphs and produces output. Engine-surface, not application-layer.
- **Module ecosystem tooling at scale** — install / uninstall / upgrade / share / discover flows. The publishing-and-using-tools experience.
- **Self-composing admin (full scope)** — admin UI is itself a graph composition that admins edit through the admin. Eats own dog food at the deepest layer.

### Layer 3 — The platform IS the dogfood

Building + using + shipping + sharing tools through Benten itself IS the comprehensive integration test. No separate "reference dogfood app" is needed for v1 — the platform proves itself by being usable end-to-end. Every primitive + sync + identity + capability + invariant gets exercised through the act of running the platform + publishing tools + receiving tools from peers + composing UI from primitives.

---

## 3. Reshaped roadmap

### Pre-v1 (committed)

| Phase | Contents | Status |
|---|---|---|
| 1 | Core engine | ✅ Shipped (`phase-1-close`, 2026-04-21) |
| 2a | Evaluator completion + debt close | ✅ Shipped (`phase-2a-close`, 2026-04-25) |
| 2b | SANDBOX + WASM + 12 primitives | ✅ Shipped (`phase-2b-close`, 2026-05-03) |
| 3 | P2P sync + Atriums + `benten-id` | In flight (mid wave-6 G16-A canary as of 2026-05-06) |
| **4** | **Benten Platform v1** (admin UI + plugin ecosystem + decentralized self-discovered registry + schema-driven rendering complete + materializer pipeline + self-composing admin + module ecosystem tooling) | ⬜ Planned (post-Phase-3-close) |
| v1-assessment-window | Comprehensive use-test the platform (sharing tools through network end-to-end); identity-recovery protocol choice; wasmtime Component-Model re-evaluation; engine impl-block generic-cascade lift; missing_docs sweep; small architectural cleanups | ⬜ Planned (between Phase 4 + tag) |
| **`v1` tag** | Benten Platform v1 ships | ⬜ Planned |

### Post-v1 (committed)

| Phase | Contents | Notes |
|---|---|---|
| **5** | **First Reference Application** (Thrum-inspired CMS built ON the v1 platform) | Could share Thrum-inspiration with Phase 4 — the admin/composition surface in Phase 4 may be Thrum-inspired at the platform layer; the CMS app in Phase 5 is Thrum-inspired at the application layer. Inspiration splits across both. |
| 6 | Personal AI Assistant (built on the v1 platform) | Same as current `FULL-ROADMAP.md` Phase 6 |
| 7 | Digital Gardens (community spaces built on the v1 platform) | Same as current Phase 7 |
| 8 | Benten Credits MVP | Same as current Phase 8 |

### Exploratory (Phase 9+)

Unchanged from current `FULL-ROADMAP.md`: Full Groves, Garden/Grove federation, Knowledge attestation marketplace, Benten Runtime, `bentend` daemon, P2P compute marketplace (broad), DAO transition, Governance Grove.

---

## 4. Why this isn't scope shrinkage

Committed scope still = Phases 1-8 per CLAUDE.md item #12. The reorganization:

- Old Phase 4 (`Thrum Migration` with claims about 3,200 tests + existing admin UI working): rewritten to "Benten Platform v1" with combined old-Phase-4 + old-Phase-5 contents.
- Old Phase 5 (`Platform Features`): folded into new Phase 4.
- Old Phase 4 as CMS-migration-of-existing-app: re-emerges as new Phase 5 ("First Reference Application") post-v1, redone properly using v1 platform primitives.
- Phases 6-8: unchanged in content.

The story changes from *"engine ships at v1; apps come later"* to *"a platform you can install + use + build tools in + share tools across networks ships at v1; apps built on the platform come later."* Stronger first-encounter narrative.

---

## 5. Why the Thrum migration framing was wrong

Per Ben 2026-05-06: Thrum was the prior codebase whose concept motivated building Benten properly in Rust, **not** a tested production CMS with 3,200 behavioral tests waiting to migrate. The "3,200 tests pass" + "existing Thrum admin UI works" claims in `docs/FULL-ROADMAP.md` Phase 4 are aspirational, not factual. Migration would require rebuilding most of Thrum anyway. Better framing: Thrum is **design-inspiration** for how a CMS could be expressed as composed subgraphs, splitting across:

- **Phase 4 (platform layer)** — Thrum-inspired admin / UI composition from primitives.
- **Phase 5 (application layer)** — Thrum-inspired CMS application built using Phase 4's platform primitives.

---

## 6. Open structural questions (resolved 2026-05-06)

| Question | Resolution |
|---|---|
| Phase 5 final form (after Phase 4 absorbs old Phase 5 contents)? | **Phase 5 = First Reference Application** (Thrum-inspired CMS, post-v1). Thrum-inspiration splits across Phase 4 (platform/admin) + Phase 5 (CMS application). |
| Sharing-surface scope for v1: basic Atrium-only sharing, or broader? | **Decentralized self-discovered registry** in v1. Not a centralized third-party registry. Tools shareable in different contexts (specific Atriums) or publicly. Anyone in your network(s) can use or reshare. UI built on existing P2P sharing primitives + the platform's own UI composition components/nodes. |
| Self-composing admin scope for v1: minimum-viable, or full? | **Full self-composing admin in v1.** The ability to compose UI from engine primitives is part of the v1 platform — a Thrum-intended capability that lands as part of the engine surface, not deferred to Phase 5+. |

---

## 7. Doc-rewrite checklist (when ratified)

When `phase-3-close` ships and the v1-assessment-window opens:

### `docs/FULL-ROADMAP.md`

- **Line 11 (v1 milestone gate paragraph)** — sharpen from "v1 covers Phases 1+2a+2b+3 at minimum" to: *"v1 = Benten Platform itself — engine + admin UI + plugin ecosystem + decentralized self-discovered registry + UI composable from engine primitives + module ecosystem at scale — installable + usable end-to-end. Not just engine."*
- **§Phase 3 (lines 62-78)** — minor: sharpen the closing line that points to Phase 4 to align with the new Phase 4 framing.
- **§Phase 4 (lines 80-96)** — full rewrite:
  - Rename heading from `Phase 4: Thrum Migration` → `Phase 4: Benten Platform v1`
  - Drop the "Proof of use" framing
  - Drop the bulleted list claiming 3,200 tests + existing Thrum modules + admin UI working
  - Drop the Phase-3 deferred items section (move to v1-assessment-window framing if still relevant)
  - Replace with: contents per §2 Layer 2 above (admin UI + plugin manifest + registry + schema-driven rendering + materializer + self-composing admin + ecosystem tooling)
  - Rewrite exit criteria around "platform installable + usable + tool-shareable end-to-end" not "Thrum's full test suite green"
- **§Phase 5 (lines 97-106)** — full rewrite:
  - Rename heading from `Phase 5: Platform Features` → `Phase 5: First Reference Application`
  - Drop the self-composition / configurable-admin framing (those moved to Phase 4)
  - Replace with: Thrum-inspired CMS application built using the v1 platform's primitives
  - Exit criteria: a working CMS application that proves apps can be built on the v1 platform
- **§Phases 6-8 (lines 108-147)** — likely unchanged in content; quick scan for any Phase-4-as-Thrum or Phase-5-as-platform-features references that need updating.
- **§Adoption Path (lines 184-196)** — reframe `Phase 4-5 (developer ecosystem)` paragraph. Phase 4-as-platform = different adoption story than Phase-4-as-migration. New shape: Phase 4 = early users install the platform + share tools through their networks; Phase 5+ adds CMS / AI assistant / Gardens etc. as use cases.
- **§Timeline Philosophy (lines 200-213)** — adjust the "Phase 4-5: 4-8 months (Thrum + platform features)" line. New estimate: Phase 4 alone is probably 9-15 months given full self-composing admin + decentralized registry + materializer pipeline are real engineering builds. Phase 5 (first reference app) on top of v1 platform: 4-8 months.

### `CLAUDE.md`

- **Section "What Is This?"** — quick scan for any Thrum-migration-shaped framing that needs aligning.
- **§"Architectural Decisions Baked In" item #15 (v1 milestone gate)** — update with the sharper v1 framing per above. Current text says "v1 covers phases 1+2a+2b+3 at minimum" + "PAUSE-AND-ASSESS step to determine what (if anything) gates v1 shippable"; update to reflect "v1 = Benten Platform v1 (engine + Layer 2 + Layer 3); Phase 4 contents land pre-v1; Phase 5+ are use cases post-v1."

### `docs/VISION.md`

- Quick scan for any Thrum-as-existing-tested-CMS framing or Phase-4-as-migration framing; align with platform framing.

### `docs/PRIMER.md`

- Quick scan + align as above.

### `docs/future/phase-2-backlog.md`

- Minor: any carries that reference Phase 4 / Thrum migration framing get updated.

### `docs/future/phase-3-backlog.md`

- Minor: any carries that reference Phase 4-as-Thrum-migration get updated. Most §-numbered backlog items are infrastructure-level and don't need to change framing.

---

## 7b. Phase-4 Benten Platform v1 deferrals (named destinations during wave-6+ implementation)

Items NAMED here per HARD RULE rule-12 BELONGS-NAMED-NOW as wave-6+ implementation surfaces them. These deferrals land at Phase-4 (Benten Platform v1, pre-v1) — not post-v1 — because they're advanced platform-surface capabilities, not application-layer work.

| Origin | Surface | Deferred destination | Notes |
|---|---|---|---|
| G16-C wave-6b (PR #124) | Light-client mode-(b) range-query proof | Phase-4 Benten Platform v1 (this doc) | ds-r4r2-3 Phase-3 commits to mode-(a) only (single-CID inclusion proof). Mode-(b) extends light-client API to range queries (multi-CID Merkle proofs). Architectural-absence pin lives at `crates/benten-sync/tests/light_client_distinct.rs::light_client_mode_b_range_query_proof_oos_phase_3_deferred_to_phase_4`. |
| G16-C wave-6b (PR #124) | Light-client mode-(c) signed checkpoint | Phase-4 Benten Platform v1 (this doc) | ds-r4r2-3. Mode-(c) extends light-client to verify against signed root checkpoints (peer-signed published roots; trust-graph extension). Architectural-absence pin at `crates/benten-sync/tests/light_client_distinct.rs::light_client_mode_c_signed_checkpoint_oos_phase_3_deferred_to_phase_4`. |
| G16-C wave-6b (PR #124) | MST diff partial-sync-cursor (one-tree-level-per-round shape) | G16-B wave-6b (engine-side wrapper) | Per mst.rs convergence-claim docstring. Phase-3 ships BTreeMap-flat shape (resolves in 1-2 rounds for n=4096); G16-B wraps it in a partial-sync cursor that exposes one tree level per round, preserving the O(log n)-rounds contract for the wire-protocol shape. Companion test pin holds the 48-round headroom. |
| G16-D wave-6b (PR #125) | Handshake response-leg via fresh iroh connection or bi-directional stream | wave-6b post-G16-D-merge follow-up | Per g16-d-mr-2: `handshake_round_trip_over_iroh_loopback_transport` drives initiate via real iroh `Connection::send_bytes` / `recv_bytes` but returns the response via in-process `tokio::spawn` task join (honestly disclosed inline at `crates/benten-sync/tests/handshake.rs::handshake_round_trip_over_iroh_loopback_transport`). Strengthening pin: open a fresh peer_b → peer_a connection for the response, OR use a bi-directional stream on the same connection. Current pin's load-bearing assertion (handshake protocol body composes with G16-A's iroh transport SEAM) is end-to-end via the initiate leg; strengthening makes both legs end-to-end. |
| G16-D wave-6b (PR #125) | napi `JsAtrium` shim retire (parallel-write-path → engine-routed cap-gated calls) | G16-B merge-time | Per g16-d-mr-4: `bindings/napi/src/atrium.rs::JsAtrium` currently mutates `Mutex<AtriumHandleState>` at the napi shim layer. Parallel-write-path-shape per CLAUDE.md baked-in #16 — intentionally interim during wave-6b parallel-3 split. At G16-B merge-time, the JsAtrium body MUST swap to `Arc<benten_engine::Atrium>` + route every mutation through engine WRITE primitive + cap policy. Recommended invariant: regression test asserting `Mutex<AtriumHandleState>` is no longer present in `bindings/napi/src/atrium.rs`. |

(Additions land here as wave-6+/7+/8+ implementation surfaces named-destination deferrals.)

## 8. Ratification + retirement

**Ratification trigger:** `phase-3-close` tag ships → v1-assessment-window opens → Ben reviews this doc + sweeps `FULL-ROADMAP.md` + dependent docs.

**Retirement:** after the FULL-ROADMAP rewrite lands, this doc gets either deleted or archived (e.g., moved to `.addl/v1-gate-archive/` for audit-trail). The ratified contents live in `FULL-ROADMAP.md` as the canonical source.

**Rationale for keeping this as a separate planning doc rather than editing FULL-ROADMAP directly now:** Phase 3 is still in flight (mid wave-6 G16-A as of 2026-05-06). Editing FULL-ROADMAP with not-yet-ratified content while Phase 3 readers consume the canonical roadmap creates a two-framing problem. The dedicated planning doc isolates the in-flight thinking + has a clean ratification path + a clean retirement path.
