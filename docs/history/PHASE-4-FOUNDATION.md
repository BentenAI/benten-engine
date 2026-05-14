# Phase 4-Foundation — Retrospective (skeleton; full prose lands at phase-4-foundation-close tag)

**Status:** SKELETON pre-tag at R6-FP-G G26-A pre-tag retense sweep. Full prose to land in the pre-tag commit at `phase-4-foundation-close`. This file is the staged outline so the retrospective is destined-NOW per HARD RULE rule-12 clause-(b) and isn't a phantom destination at the moment R6-FP-G ships.

---

## TL;DR (one-paragraph summary)

Phase 4-Foundation ships the substantive Benten Platform engineering — the layer above the engine that makes the system end-to-end usable through a UI rather than only through the napi/TS surface. Substantive deliverables: admin UI v0 (the first plugin / default module / Foundation entrypoint) on a 4-category navigation IA (Plugins / Workflows / Content Types / Views); full plugin manifest format with install-time consent + per-plugin DID + manifest envelope chain validation + private-namespace caps + DAG-shape versioning; decentralized self-discovered registry on top of Atriums; schema-driven rendering pipeline (typed-field-Node vocabulary — 8 labels, 5 labeled edges + implicit-via-recursion object→field, 8 scalars); materializer pipeline (HtmlJsonMaterializer + IVM-subgraph generalization); Tauri 2.x renderer engine extension for embedded-webview deployment shape (c). The Rust workspace grew from 10 → 12 crates (`benten-platform-foundation` + `benten-renderer-tauri`); CATALOG_VARIANT_COUNT grew from 132 → 163 (31 net new ErrorCodes minted; 17 planned + 14 unplanned-but-named). 30+ PRs merged across R5; ADDL pipeline ran the full sequence — pre-R1 → R1 (12 lenses, 203 findings) → R1 triage → R2 → R3 → R4 → R4-FP → R4b → R4b-FP → R5 (17 waves, 5 strategy-C batches) → R6 phase-close convergence council (16-18 lenses) → R6-FP waves (A-G) → pre-tag sweep → tag `phase-4-foundation-close`.

---

## Major decisions (Q1-Q8 ratifications + Q-R6-1/2/3 + R1 architectural forks)

### R1 triage architectural ratifications (8 items, Ben 2026-05-11 LATE EVENING)

1. **Crate decomposition — single `benten-platform-foundation` crate.** Rejected the narrower three-or-four-platform-crate decomposition (per arch-r1-8). The crate is intentionally broader than other crates because every part composes into one platform-shippable boundary; the v1-gate framing groups them as one shipping unit. Trade-off accepted: cross-cutting refactors stay in one crate's territory.
2. **TS-side ErrorCode mirror location = `packages/engine/src/errors.generated.ts`.** NOT a new `packages/error-codes/` package. Single TS package owns the mirror; codegen pipeline updates atomically with Rust catalog (per pim-cross-language-rule-mirror §3.5g).
3. **Plugin identity = four separated concepts** (content-CID + peer-DID signature on original content + plugin-DID minted at install + user-DID). NO device-DID-style attestation chain back to user-DID for plugins. Plugin-DID is purely a UCAN audience handle bounded by source plugin's manifest `shares` policy.
4. **4-category nav IA canonical order = `["Plugins", "Workflows", "Content Types", "Views"]`** per D-4F-4.
5. **Active reference shape = per-device-local Loro Map (NOT user-graph CURRENT pointer).** Switching active version = per-device Loro Map write; library subgraph holds all installed versions + forks.
6. **Plugin manifest schema versioning field = none.** CID covers shape; pull-not-push obviates schema-version field per D-4F-13.
7. **Workflow ↔ plugin unification = same subgraph shape.** Distinguished by manifest presence + sharing intent, not by substrate.
8. **Identity-recovery protocol — MVP SelfRevocation attestation; Kith deferred to Phase 5+ exploratory.** Phase-4-Foundation does NOT depend on Kith.

### Q1-Q8 ratifications (post-R5 Ben 2026-05-13)

- **Q1:** RED-PHASE-BODY = anomaly (NOT codified as pim-N).
- **Q2:** Strategy-C batch-merge codified → dispatch-conventions §3.14 (memory `feedback_batch_merge_strategy_c`).
- **Q3:** G24-B-FP-2 → R4b-FP-2 (in flight; closes §4.16 substantive replay arm + §4.17 cross-lang drift-defense).
- **Q4:** 4 seams build at R4b-FP-1 (Class B β + install_plugin lifecycle + validate_with_clock + `apply_atrium_merge` envelope-recheck per Compromise #26); 2 seams → Phase-4-Meta §4.19.
- **Q5+Q6:** pre-tag sweep at R6-FP-G G26-A (THIS WAVE).
- **Q7:** 1-LOC presence pins shipped.
- **Q8:** no-action.

### Q-R6-1/2/3 ratifications (R6 round)

- (TBD — fill in at final retrospective from R6 lens dispositions.)

---

## Wave-by-wave summary (17 R5 waves, 5 strategy-C batches)

| Batch | Main HEAD | Waves merged |
|---|---|---|
| 1 | `156acbd` | G24-F + G23-A + G23-0a |
| 2 | `ceff33a` | G24-D + G23-0b |
| 3 | `f9bd5b1` | G27-D + G24-D-FP-1 + G24-D-FP-2 + G23-B + G24-D-FP-3 |
| 4 | `64b9b15` | G24-A + G24-E |
| 5 | `fcc0203` | G24-B + G24-C + G24-B-FP-1 |
| Pre-R5 (already on main) | (various) | G27-A + G27-B + G27-C |
| R4b-FP (3 parallel) | (TBD) | R4b-FP-1 + R4b-FP-2 + R4b-FP-3 |
| R6-FP (7 parallel) | (TBD) | A (plugin-trust) + B/F (schema + tests) + C (catalog) + D (plugin-library-graph) + E (admin-shell) + G (doc-retense) |

(Wave content + key code surfaces to fill in at full retrospective; brief outline only for now.)

---

## pim-N codifications (new from Phase-4-Foundation R6)

- **pim-N — strategy-C batch-merge (§3.14).** Ratified Q2; promoted from memory `feedback_batch_merge_strategy_c` to dispatch-conventions inline.
- (Additional pim-N candidates from R6 R1 pim-N meta-sweep — fill in at final retrospective.)

---

## Backlog carries to Phase-4-Meta

- §4.19 — 2 R4b-FP enhancement seams (enumerated at R4b-FP-1 brief).
- §4.20-§4.40 — wave-specific deferrals (see `docs/future/phase-4-backlog.md` §-renumbering log at the strategy-C batch reconciler).
- §3.6 — deny.toml RUSTSEC ignore migrations carried from R6-FP-E.
- Identity-recovery protocol full design (Kith working name; Phase 5+).

---

## Numbers (preliminary)

- **CATALOG_VARIANT_COUNT:** 132 (pre-Phase-4-Foundation baseline) → 163 (R6-FP-C HEAD; +31 net new). After strategy-C batch reconciliation with Wave-A's 4 plugin install-record + DID-handle variants: **167** (pre-tag).
  - 17 planned net-new (per R5 baseline: G23-A 9 + G23-B 3 + G24-D 15, minus absorbed)
  - 14 unplanned-but-named at R6-FP-C (ALL_CATALOG_VARIANTS regression list refresh)
  - 4 added at R6-FP-A (typed consent-substitution-defense ErrorCodes)
  - Cohort math reconciled at ERROR-CATALOG.md preamble narrative (4-row table: throwable / catalog / rust enum / list)
- **Crates:** 10 → 12 (`benten-platform-foundation` + `benten-renderer-tauri`)
- **PRs merged this phase:** 30+ (#207-#238 + R4b-FP cluster + R6-FP cluster)
- **R1 findings:** 203 (19 BLOCKER / 84 MAJOR / 67 MINOR / 33 OBS)
- **R6 R1 findings:** 47 (1 BLOCKER doc-r6-r1-1 + 8 MAJOR doc-r6-r1-2..r6-r1-9 in doc-cite-drift alone; ~30 other-lens MAJOR; ~30 MINOR; ~10 OBS)
- **LOC delta:** ~30k+ across `benten-platform-foundation` + `benten-renderer-tauri` + sweeping touch across `benten-engine` + `benten-caps` + `benten-id`

---

## Closing posture

Phase 4-Foundation lands the v1 platform-shippable surface per CLAUDE.md baked-in #15 (v1 milestone gate). The engine + admin UI + plugin ecosystem + decentralized self-discovered registry are now installable + usable end-to-end. The next phase (Phase 4-Meta) layers self-composing admin meta-circular work + ingests Phase-3-deferred items + runs the v1-assessment-window before the `v1` tag.

The 12-primitive irreducibility commitment held: zero new `PrimitiveKind` variants added in Phase 4-Foundation. All platform features compose from the existing 12 primitives via the schema-compiler + materializer + admin-UI subgraph builders. CLAUDE.md baked-in #1 stands.

The three-deployment-shape commitment (baked-in #17) shipped concretely: (a) full peer via `benten-engine` native build; (b) thin compute surface via `benten-engine` wasm32 + `BrowserRender`; (c) embedded webview via `benten-renderer-tauri` wrapping the same wasm32 bundle. Tauri-shell vs browser-tab is a deployment choice, not an architectural shape change.

The app-level-plugin + engine-extension trust model (baked-in #18 + #19) shipped concretely with `manifest_envelope_chain_validation` + `plugin_delegation::is_private_namespace_cap` + Class B β `read_node_as` + the four-identity-concept separation.

*Skeleton drafted at R6-FP-G G26-A pre-tag sweep; full prose lands at `phase-4-foundation-close` tag.*
