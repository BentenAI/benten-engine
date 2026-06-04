# Phase 4-Foundation — Benten Platform v1 Foundation

**Status:** SHIPPED at tag `phase-4-foundation-close` (HEAD `0ce98d0` post-PR #248, 2026-05-14).
**Closed at:** ~30 PRs in the `phase-3-close..phase-4-foundation-close` window (#205-#248 plus pre-R5 dependabot).
**Convergence:** R6 phase-close convergence council 8 rounds under Q5 cadence — R1 ≈ 185 → R2 ≈ 83 → R3 ≈ 50 → R4 ≈ 27 → R5 ≈ 16 → R6 = 11 → R7 ≈ 8 → R8 = 0 (17/17 lenses APPROVE / CONVERGED / TAG-READY at the terminal round; 5 lens-surfaces hit consecutive zero-finding plateaus, capability-system at 5 consecutive zero rounds the strongest single-lens convergence in project history).
**Workspace:** 12 crates (added `benten-platform-foundation` 11th + `benten-renderer-tauri` 12th).
**Catalog count:** CATALOG_VARIANT_COUNT 118 → 168 (50 net new ErrorCodes).
**Pim-N codifications added:** 7 new + 4 amendments (§3.5g items #3 + #4 / §3.5i / §3.5j / §3.6g / §3.6h with R6-FP-4 + R6-FP-5 sharpenings / §3.6i / §3.6j; Q5 cadence amendment to `feedback_phase_close_final_council_full`).
**CLAUDE.md baked-in commitments:** #17 amended (embedded webview as 3rd deployment shape); #18 detailed with 4-identity-concepts separation; D-4F-1..D-4F-16 ratified during R1 triage.

---

## § 1. Narrative journal

Phase 4-Foundation opened on the back of the cleanly-closed Phase-3 tag (`phase-3-close`, HEAD `355b58b`, 2026-05-10) and the pre-v1 cleanup night-shift that immediately followed it — fifteen PRs across #175-#199 landing the branch-protection spec, the plugin/extension architecture documentation, the `Engine::read_node_as` Class B β shape at PR #184, the UCAN revocation observance fix at PR #199, and the `docs/history/PHASE-3.md` retrospective. What had been called "Phase 3.5" through the early planning conversations was renamed on 2026-05-11 to **Phase 4-Foundation** (former "Phase 4" became "Phase 4-Meta") once Ben's framing made clear that substantive Phase-4 scope was absorbing into the work: full plugin manifest schema, decentralized self-discovered registry, materializer pipeline, IVM-subgraph generalization, admin UI v0 as the first plugin, module ecosystem tooling at scale. The phase-rename PR #205 + the Phase 4-Foundation ratifications PR #206 (SECURITY-POSTURE Compromises #24 + #25 + phase-3-backlog §15.2 handler-cycle-detection) landed before R1 dispatch. The workspace opened at 10 crates (`benten-id` 9th + `benten-sync` 10th from Phase 3); by phase close it would be 12 with `benten-platform-foundation` and `benten-renderer-tauri` added.

The phase's defining tension was that the work split cleanly into three layers piled on top of each other without architectural shape changes: (a) the **schema-driven rendering** layer — typed-field-Node vocabulary, schema compiler, materializer pipeline, IVM-subgraph generalization — taking schemas from "Rust types you hand-write" to "subgraphs of typed Nodes the engine walks like any handler"; (b) the **plugin** layer — full manifest with three-layer consent + per-plugin DID + manifest envelope chain validation + private-namespace caps + DAG-shape versioning — making third-party shareable subgraphs first-class without minting any new `PrimitiveKind` variant; (c) the **admin UI v0** layer — the first plugin that runs *as* a graph composition, the dogfood realm where plugins are installed and tools are composed, riding on top of the new Renderer trait that ships with BrowserRender + TauriRenderer concrete impls so the same engine binary runs as full peer, thin compute surface, or embedded webview. The phase did not add a 13th primitive, did not invert any dependency edge, did not break the 12-primitive irreducibility commitment, and emerged with the 19 CLAUDE.md baked-in decisions intact under broad cross-lens scrutiny.

### Pre-implementation review pipeline (pre-R1 → R1 → R1-FP → R2 → R3 → R4 → R4-FP → R4b → R4b-FP)

The pre-R1 critic round dispatched three lenses (architect-reviewer + methodology-critic + security-auditor) against `.addl/phase-4-foundation/00-implementation-plan.md` and the new `admin-ui-v0-threat-model.md` — a 525-line document covering 12 threat classes (subgraph-injection, cross-plugin-leak, CSRF-cross-origin, admin-UI-cap-elevation, manifest-substitution-at-install, atrium-share-unattested, private-namespace-cap-exfil, UCAN-chain-spoofing, schema-author-rotation-replay, plugin-upgrade-downgrade, install-without-clock, plugin-DID-as-attested-identity-misuse). Pre-R1 returned 49 findings; Ben triaged 6 architectural fork-points and the orchestrator dispositioned the mechanical balance into the revised plan.

**R1 dispatched 12 lenses in parallel** on 2026-05-11 evening — 4 new lenses for Phase-4-Foundation surfaces (plugin-architecture / schema-language / ux-on-admin-UI / materializer-correctness) and 8 reused Phase-3 lens shapes. All 12 returned cleanly. The tally landed at the upper bound of expectation: **203 findings = 19 BLOCKER / 84 MAJOR / 67 MINOR / 33 OBS**. The cross-lens convergence pattern that defined Phase-2b R1 and Phase-3 R1 repeated three times: security-auditor + materializer + capability-system independently arrived at "per-event cap-policy resolution at `Engine::on_change_as_with_cursor`" for the SUBSCRIBE-delivery cap-recheck scaffold-only gap; security + plugin-architecture + capability-system independently named the `validate_chain_with_manifest_envelope` chain-validator seam at what eventually became `manifest_envelope_chain_validation.rs` at G24-D; schema-language + code-as-graph + architect independently arrived at the typed-field-Node vocabulary surface that Ben subsequently resolved as Q1 ratification (8 labels + 6 edges + 8 scalars + 4 mandatory properties — the 6-edge count would later drop to 5 at R6-R4 when the FIELD edge proved implicit-via-recursion).

R1 triage produced a 607-line disposition document and Ben's eight architectural ratifications crystallized the commitments that shaped the rest of the phase: workspace shape locked at 12 crates (single broad `benten-platform-foundation` per arch-r1-8); DAG version-chain CURRENT pointer per-device-keyed via Loro Map (intentional per-device variance first-class); decentralized self-discovered registry deferred to Phase 4-Meta (Phase-4-Foundation admin UI installs via direct content-addressed share over Atriums); 4-category navigation IA (Plugins / Workflows / Content Types / Views); plugin-DID minted as `did:key:...` with engine-held signing keypair (both UCAN audience handle *and* constrained issuer); RotationLog peer-to-peer propagation via `SelfRevocation` attestation with `Kith` named as the Phase-5+ exploratory richer system; subscribe-re-walk consistency at Option (c) re-filter-at-delivery with Node-granularity redaction; cross-fork upgrade defense via cap-change-triggered consent for all upgrades. Five additional Q resolutions locked the typed-field-Node vocabulary, meta-plugin composition cycle detection as rejection, the Kith working name, plugin-DID as constrained issuer, and Makefile-based pre-push automation (no git hook). The R1-triage doc-revision agent landed plan revisions + threat-model adjustments + three new tracked docs (`PLUGIN-MANIFEST.md`, `SCHEMA-DRIVEN-RENDERING.md`, `ADMIN-UI.md`) + five doc retenses (ARCHITECTURE / HOW-IT-WORKS / SECURITY-POSTURE / ERROR-CATALOG / GLOSSARY) + the Makefile target as **PR #207** merged at HEAD `0e1c221` with 1,121 insertions across 14 files.

R1-FP wave-1 dispatched five implementer agents in parallel — the first implementer dispatch of the cycle, well within the cap-of-seven concurrency limit. The G22-FP-1 wave returned with the load-bearing first architectural fork: the doc-revision agent had codified Ben's #7 ratification as "stream stays open + silent per-Node elision," but Phase-3 R6-FP Wave-C1 at PR #170 had shipped the **opposite** semantics — `cap_recheck false → typed SubscribeRevokedMidStream + whole-subscription auto-cancel`. The implementer surfaced the conflict with three options per `feedback_surface_arch_decisions_under_auth`. Ben's morning resolution preserved auto-cancel semantics (Option C-adjacent): per-event cap-recheck fires; the shipped contract stays; the "silent elision" framing was reread as advisory not normative. R1-FP wave-1 closed with G22-FP-1/2/3 production seams shipped (SUBSCRIBE-delivery cap-recheck, UCAN audience binding, GrantBackedPolicy actor_cid threading).

R2 ran as a single test-landscape-synthesis agent producing a 75 KB specification partitioning ~218 test artifacts across 9 R3 families (schema-language / materializer-and-IVM / plugin-manifest / admin-UI / capability / threat-model / class-of-bug-audit / thin-client-and-Tauri / cross-language-and-cite-drift). R3 ran 9 parallel TDD-red-phase test writers; about 140 of the enumerated 218 pins landed at R3 close (~64% file-count). R3 introduced the `phase_4_foundation_pending_apis` feature flag pattern — sibling to Phase-2b's `phase_2b_landed` and Phase-3's `phase_3_pending_apis` — for compile-vs-execute gating of the unimplemented surface area. **R4 ran 6 parallel review lenses** on the R3 red-phase corpus and triaged 64 raw findings to ~58 unique: 3 BLOCKER / 23 MAJOR / 15 MINOR / 23 OBS. All three BLOCKERs clustered on R3-coverage gaps where R3 writers hadn't received explicit per-family pin lists for four areas (G24-B T4 admin-UI cap-elevation Rust side / G23-B T9 schema-author-trust / G24-D T10 plugin uninstall + upgrade per-finding granularity / G26 wave-10 docs+CI hygiene). R4-FP dispatched four implementer waves closing the BLOCKERs + ~31 MAJORs. Ben ratified four cross-cutting decisions from R4 triage: TS-side ErrorCode mirror at `packages/engine/src/errors.generated.ts` (not a new package); CATALOG_VARIANT_COUNT math = 27 minted / 10 absorbed / 17 net new at R5-close; ErrorCode rename `E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE` → `E_PLUGIN_DEVICE_ATTESTATION_FORGED` preserving the `E_PLUGIN_*` family prefix; canonical 4-category nav order `["Plugins", "Workflows", "Content Types", "Views"]` per D-4F-4.

### R5 implementation arc — cascade-dispatch + 17 waves + strategy-C batch-merge

R5 implementation kicked off with **cascade-dispatch discipline** (Ben ratified) rather than strict rounds: each upstream's implementer commits + pushes + mini-reviewer APPROVES + any orchestrator fix-pass applied → dispatch downstream agents. The critical path G23-A → G23-B → G24-A → G24-B ran four sequential waves end-to-end and dictated phase completion timeline; siblings (G16-D, G27-D, G24-E, G24-C) ran parallel-where-disjoint. Three G27 PRs landed pre-R5 Round-1 (PR #224 GrantBackedPolicy scope-derivation lift; #225 `benten-id::grant_reader` sibling trait + CID-keyed companion; #226 napi class-of-bug audit with phase-4-backlog §4.5 added per HARD RULE BELONGS-NAMED-NOW). Round-1 implementer waves landed in cascade order with sync mini-reviews catching the cluster's defining failure modes.

**G23-0a IVM kernel generalization** (~640 LOC; 11 tests un-ignored; `Strategy::C → Reserved` rename atomic) shipped as the canary; **G23-0b** re-expressed the five hand-written Phase-1 IVM views against the generalized Algorithm B kernel. **G24-F thin-client session** (~1010 LOC; 4 new ErrorCodes minted atomically Rust+TS; HandshakeInvalid + ChallengeReplay + OriginMismatch + SessionExpired; CATALOG_VARIANT_COUNT 118 → 122) shipped clean. **G23-A schema_compiler canary** (~2000 LOC; mints 9 vocabulary-surface ErrorCodes) surfaced the phase's first **phantom-destination-by-implementer** instance: the implementer had embedded "G23-A wave-4b" as the deferral target for strict 4-of-4 dialect validation in production source comments and the commit message — a wave that didn't exist. The closure was orchestrator-direct inline fix before merge, adding a real `phase-4-backlog.md` entry with explicit acceptance criterion and Phase-target.

**G24-D full plugin manifest** (the biggest wave of the phase at ~3000 LOC; mints 15 ErrorCodes) surfaced two BLOCKERs of a kind that defined a recurrent pattern across R5: g24d-mr-1 was **phantom-destination-by-deletion** (the wave deleted `phase-4-backlog.md §4.5` named-destination for the substantive 4-step delegate-napi-binding rewrite without fulfilling its work obligation — `delegate_capability` / `delegateCapability` / `issue_delegation` symbols were nowhere in the branch); g24d-mr-2 was a peer BLOCKER on parallel substance gap. The closure required a G24-D fix-pass commit plus a 38-pin disposition document routing each individual G24-D pin to fix-now / re-named / closed-by-other-wave. G24-D-FP-1 closed the uninstall-cascade seam at `plugin_lifecycle.rs`; G24-D-FP-2 closed the manifest envelope chain validation at `benten-caps`; G24-D-FP-3 closed cap-cascade atomicity + the Step-9 partial-mint defense (surfacing the `NoopRechecker` decorative-until-real-impl gap that R6 R1 would later name at §4.36).

**G23-B materializer + Renderer trait** (~2000 LOC; mints 3 ErrorCodes) introduced the `Renderer` trait at the engine boundary with transport-agnostic methods, minting `BrowserRender` as the first impl in `benten-platform-foundation` — the substrate for D-4F-2 + D-4F-11 output-format pluggability. **G24-A admin UI v0 shell + 6 dogfood plugins** (~2000 LOC) surfaced two pim-12 §3.6e RED-PHASE staged-pin un-ignore discipline violations: the T1 LOAD-BEARING `admin_ui_v0_hostile_schema_read_emit_chain_denied.rs` remained `#[ignore]` with `unimplemented!()` body despite the ignore-attribute naming G24-A as the un-ignore target. The closure required wiring the production arm using available substrate (the `EngineMaterializerAdapter` + handcoded-spec constructor + `AdminUiV0TestHarness` machinery shipped in the same wave). **G24-E Tauri renderer crate** (~600 LOC) shipped the second `Renderer` impl in the new `benten-renderer-tauri` crate (12th workspace crate) with IpcAllowlist + CSP + per-method cap-binding. **G24-B workflow editor** (~1000 LOC) shipped the substantial `AdminUiV0TestHarness` machinery — composed-engine-shape with admin-UI and hostile-plugin as system:Principal Nodes in a single Engine instance via `engine.create_principal()`, the substrate for all plugin-isolation defense pins across G24-A/B/C. **G24-C composed-view creator** (~1000 LOC) and **G27-D manifest-aware scope derivation** (~400 LOC) closed the R5 implementation arc.

**Strategy-C batch-merge** was the load-bearing process discovery of R5. When ≥3 wave-PRs accumulated unmerged simultaneously, Ben ratified Q2 (2026-05-13) the local-merge-into-2-3-wave-batches + single-PR-per-batch shape, codified at `.addl/dispatch-conventions.md §3.14` and validated five times during R5 close alone: PR #231 (G24-F + G23-A + G23-0a), #232 (G24-D + G23-0b), #234 (G27-D + G24-D-FP-1 + G24-D-FP-2 + G23-B + G24-D-FP-3), #235 (G24-A + G24-E), #238 (G24-B + G24-C + G24-B-FP-1). Fifteen R5 waves shipped via five CI cycles instead of fifteen — roughly 60-75% queue thrash reduction. The pattern was validated seven times across the phase by close (the five R5 batches plus R4b-FP batch #239 plus R6-FP batch #240). Memory `feedback_batch_merge_strategy_c.md` is the canonical durable reference.

R4b dispatched four lenses parallel against the just-closed R5 corpus. L1 test-coverage tallied 7 MAJOR phantom-destination cluster: eight-plus tests cited shipped waves where un-ignore was never delivered (`install_plugin` lifecycle, `validate_with_clock` engine injection, `apply_atrium_merge` envelope-recheck integration, plugin meta-composition cycle wiring). L2 surfaced 2 MINOR for missing presence pins (closed inline at R4b-FP-3 as 1-LOC pins). L3 had 3 MINOR stale ERROR-CATALOG narrative findings (deferred to pre-tag sweep). L4 returned APPROVE clean. Ben's Q1-Q8 ratifications dispositioned the cluster: four v1-shippable seams build at R4b-FP-1; two enhancement seams to Phase-4-Meta §4.19; Strategy-C to dispatch-conventions §3.14; pre-tag sweep target at Q5+Q6; one-LOC presence pins shipped at R4b-FP-3. R4b-FP shipped three parallel waves batched via Strategy-C into **PR #239** admin-merged.

### R6 phase-close convergence council — 8 rounds under Q5 cadence

The R6 phase-close convergence council dispatched on 2026-05-13 against `277b1bf` (post-R5 + R4b-FP) with the broadest lens set in project history: **17 lenses** — Phase 3 had run 18 lenses but Phase-4-Foundation had absorbed enough substantive scope (admin UI v0 + full plugin manifest schema + schema-driven rendering + per-plugin DID + 3-layer consent + Tauri 2.x renderer + thin-client session) that the lens set had to grow accordingly. R1 ran the full 17-lens roster and returned **5 BLOCKER + ~70 MAJOR + ~57 MINOR + ~53 OBS = ~185 cumulative findings**, matching Phase-2b R1's magnitude on broader scope.

The five BLOCKERs each had distinct shape. `sec-r6r1-1` (security-reviewer): `install_plugin` discarded the `InstallRecord.plugin_did` binding — the 11-step pipeline minted a fresh plugin-DID at step 8 but never compared against the `plugin_did_bytes` already bound into the user's signed `InstallRecord.signing_payload`. The structural defeat was that the most load-bearing piece of the consent signing payload — the principal the user said yes to — was being thrown away. Closure: **caller-mint-first contract** with `InstallContext::expected_plugin_did: &Did` parameter + Step-8 mismatch check + handle-uniqueness assertion. `r7-1`: `ERROR-CATALOG.md` narrative claimed 163 codes; actual `CATALOG_VARIANT_COUNT` was 149 and climbed to 167 during R1's window. `tmr-r6-r1-1`: fail-OPEN default at `EngineBuilder::manifest_envelope_rechecker` — defaulting to `None` meant any caller who forgot to wire the rechecker silently bypassed Layer-3 envelope check, gutting the 3-layer consent narrative; closed by flipping default to `Some(NoopRechecker)` with cap-r6-r3-1 + arch-r6-r1-1 sharpening the noop-vs-real distinction at later rounds. `pa-r6-r1-2`: dual-public `install_plugin` entry points (legacy `module_ecosystem::install_plugin` 9-step precursor was still `pub` and bypassed Layer-2/Layer-3 consent gates). `r6r1-cb-1`: cumulative cite-drift cluster (84 non-archive findings + 14 11→12-crate transition cites + 149→168 ErrorCode count drift). The cross-lens convergence pattern that defined R1 was striking: **security-reviewer + threat-model-reviewer + plugin-arch-cap-policy-reviewer + r7-spec-compliance** all surfaced the fail-OPEN default independently — four lenses converging on the same closure shape gave Ben the cross-confirmation needed to ratify `Q-R6-2 path-a-now`. R6-FP dispatched six parallel waves (A / BF / C / D / E / G); the **R6-FP cluster admin-merged at PR #240** with all 5 BLOCKERs + ~25 substantive MAJORs closed.

R6 R2 opened at `85ecb69` on 2026-05-13 evening. The orchestrator initially dispatched **11 paired-dispatch agents** covering 22 lens-perspectives — a cost-saving choice analogous to Phase-2b's R6 R2 lens-reduction misfire. Ben caught the deviation and issued **Q2 full-scope corrective**: every R6 post-R1 round must run as full N-lens-set with one agent per lens. Four catch-up solo lenses dispatched in parallel against the same HEAD — methodology-critic-solo, schema-language-solo, materializer-correctness-solo, code-as-graph+UX-solo. The paired-dispatch + catch-up combination returned **1 BLOCKER cluster + ~17-19 distinct MAJORs + ~30 MINORs + ~35 OBS = ~83 cumulative** (55% reduction R1=185 → R2=83 monotonic). The catch-up solo lenses substantively sharpened paired output without contradicting it — confirming that pairing didn't produce *wrong* findings, just *underbalanced coverage*. The **Q5 cadence amendment** ratified by Ben 2026-05-13 became the most-load-bearing process call of the entire R6 arc: every post-R1 round = full N-lens council; NO narrow iteration; convergence loop terminates only when full council returns 0 findings. R6-FP-2 at **PR #242 → `6e10aea`** closed the BLOCKER cluster (three PLUGIN-MANIFEST.md drifts) + ~14 MAJORs (ERROR-CATALOG preamble 163→167 reconciliation; TauriRender → TauriRenderer 7-cite sweep as first real exercise of newly-ratified §3.5g item #3; INVARIANT-COVERAGE + ENGINE-SPEC title retense; `expected_plugin_did` caller-mint-first contract added to PLUGIN-MANIFEST + ADMIN-UI; SCHEMA-DRIVEN-RENDERING ITEM_TYPE FieldList-only narrative). The cite-drift sentinel test now passed (316 findings → 0; excluded `docs/history/` floor + narrowed `.addl/` scope).

R6 R3 dispatched the full 17-lens council against `6e10aea` per Q5 cadence and ran the most consequential ratification round of the phase. The 1 BLOCKER was a perfect exemplar of `pim-cite-drift-fp1-recurrence`: R6-FP-2's own narrative for `§3.5g item #4` contained the cite `.github/workflows/supply-chain.yml::cargo-audit` which the detector's regex tokenizer parsed as `path + symbol=cargo` (truncated at the dash) — the codification PR itself violated the §3.5h MANDATORY-PRE-MERGE gate it was extending. But the defining R3 finding was **`meth-r6-r3-1`**: 24 of 32 R6 lens reports used `verdict` only; 5 used `disposition`; 3 lacked both. The schema drift had compounded across 14 mini-reviews + 17 R6-R3 JSONs. Two new pim-N ratifications landed inline at R6-FP-3 to close the round: **§3.6h Rule-ratification-against-drift mandatory-close** (when a new pim-N or §-codification names specific drift instance(s) as its origin, the same PR/wave landing the rule MUST close or DEFER-NAMED-NOW the origin instances — six-instance recurrence visible since R1: §3.5g #3 TauriRender 8-cite drift, §3.5g #4 cargo-audit ↔ deny.toml, §3.5h MANDATORY-PRE-MERGE precedent, §3.6g forward-fire, §3.5i, §3.5j) plus **§3.6i Review/lens/mini-review JSON schema discipline** (canonical top-level `disposition` field, NOT `verdict`; brief-template mandate added). Two §3.5h amendments folded under the existing umbrella without minting new pim-N: JSON-artifact validation (`jq .` across touched JSON pre-push, from `r6r3-meta-1` surfacing a malformed lens JSON) and GREEN-CI-CONFIRMATION substrate clause (admin-merge bypass per §3.14 requires comparing PR CI failures against pre-existing main-side baseline, from `r6r3-meta-2`). R6-FP-3 at **PR #243 → `96f70df`** closed inline: cite-drift detector cite-form fix (1 LOC); `E_PLUGIN_DID_HANDLE_DUPLICATE` ErrorCode with full 4-surface mirror; VocabEdge enum FIELD variant dropped (6 → 5 variants — object-to-field is implicit-via-recursion); webview-e2e active TCP port-probe loop; 32-file `verdict → disposition` legacy-artifact sweep; CRATES-DEEP-DIVE.md 11+12 crate retense.

R6 R4 returned **0 BLOCKER + 3 MAJOR + ~20 MINOR + 4 NAMED-NOW** — convergence-favorable. The R4 pattern-induction-meta-sweep produced the **§3.6h "already-closed-before-ratification STRICTLY STRONGER" sharpening** and the **forward-fire-only exemption**: when the origin drift was closed at an earlier wave + the codification merely names that earlier closure as evidence, the rule is satisfied; and when origin instances are wave-class patterns that already-shipped, the rule fires forward-only without violating §3.6h. R4 saw the first cleanly-CONVERGED lenses of the phase — distributed-systems + materializer-correctness + capability-system + r7-spec-compliance — convergence-favorable per the codification PRs not making substantive cross-lens surface edits. R6-FP-4 at **PR #244 → `c592873`** closed inline.

R6 R5 opened the terminal-convergence arc against `c592873`. The cumulative R5 tally returned ~16 substantive findings: **0 BLOCKER / 2 MAJOR + ~14 MINOR + ~31 OBS**, plus 2 pim-N candidates both DEFERRED per agent recommendation. The two MAJORs were cross-lens convergence findings: `schema-lang-r6-r5-1` (R6-FP-4's 11-site "5 labeled edges" sweep claimed complete but missed three sites — `subgraph.rs:911-919` rustdoc still saying "6 vocabulary edges", `ERROR-CATALOG.md:1327` Message text, and a test module-doc framing); `br-r6-r5-1` (§4.49 webview-e2e MUST-FIX-OR-EXPLICITLY-ACCEPT-AT-TAG row firing red because `tauri-driver` didn't support the `--native-binary` flag the test was passing). **This was the second of four instances that would ultimately promote §4.57 → §3.6j at R6-FP-7.** R6-FP-5 at **PR #245 → `2fbfd70`** closed: the 3-site schema-vocab-edge sweep; SECURITY-POSTURE Compromise #26 Status section Layer-3 ordering correction (AFTER per-row cap-revocation check per CLAUDE.md #18); PLUGIN-MANIFEST.md row-3 caller-mint-first retense; `workflow_to_plugin.rs` rustdoc accuracy fix; Cargo.toml dev-deps dedup; three new NAMED-NOW backlog rows; §4.49 sharpening with root-cause cite + Path-(a)/Path-(b) acceptance criteria.

R6 R6 returned **0 BLOCKER + 1 MAJOR + 9 MINOR + 32 OBS = 11 substantive findings + 1 pim-N watch-list candidate** (~55% drop from R5). The lone MAJOR was `meth-r6-r6-1` cross-confirmed with pattern-induction's `r6r6-pi-1`: R6-FP-5's commit body claimed 49-JSON `§3.6i` sweep complete but the lens-self-validation found 4 still lacking top-level `disposition` (r6-r4-materializer-correctness + r6-r5-invariant-compromise + r6-r5-schema-language + r6-r5-ux-admin-ui). The pattern-induction lens classified this as **the 3rd instance** of the same class — sweep-tooling defined against past-state baseline, not the round's own outputs — and recommended creating a watch-list pim-N candidate. Ben's Q3 call: defer + watch-list; see if a 4th instance fires. R6-FP-6 at **PR #246 → `3e606be`** closed: 4 R6-R6 JSON top-level `disposition` adds; 3 TS Playwright stale-rationale retenses citing §4.22; three new NAMED-NOW backlog rows (§4.55 storage-mutating host-fn banned-list, §4.56 `Renderer::render()` no-op stub production caller, **§4.57 sweep-completeness self-verify discipline pim-N watch-list**). The R6-FP-6 commit body made the load-bearing claim that set up the next round's drama: "79/79 R6 JSONs §3.6i conformant."

R6 R7 fired the watch-list trigger as anticipated. Pattern-induction's brief carried the "watch for 4th instance" clause going in, and the round returned the cleanest possible 5-lens cross-confirmation of the same finding: R6-FP-6's "79/79" claim was true for the 79 R6 R1-R5 JSONs (the FP-5/FP-6 sweep target set) but **did NOT include the 17 R6 R6 lens outputs themselves**, of which 4 lacked top-level `disposition`. The five lenses that converged: `r6r7-pi-1` (pattern-induction-meta-sweep) + `r6r7-meth-1` (methodology-critic) + `r6r7-r7-1` (r7-spec-compliance) + `doc-r6-r7-1` (doc-engineer-cite-drift) + `arch-r6-r7-1` (architecture-reviewer). Pattern-induction fired its own promotion criterion explicitly: **the 4th instance** of the same failure mode (R6-FP-3 → R6-FP-4 → R6-FP-5 → R6-FP-6, each sweep claiming complete + each missing residuals on the round's own outputs). Ben's Q4 ratification at R6-FP-7 dispatch — the "are you sure you don't wanna do those things you're deferring now instead" framing — elected to promote §4.57 → §3.6j in the same wave that closed its own origin instances (the strictly-stronger "already-closed-before-ratification" path per §3.6h's sharpening). The Path A INTENT vs Path B STRICT methodology fork was resolved in favor of Path A INTENT per the pattern-induction lens's explicit recommendation — "§3.6h is 'rule names origin'; §3.6j is 'claim names scope'" — folding the new rule under the existing §3.6 family with the scope-criterion difference made explicit. R6-FP-7 also closed: `schema-lang-r6-r7-1` (added ADMIN-UI.md §2.1 user-facing field-type mapping table closing ux-r1-12 + the phantom GLOSSARY cross-ref); `arch-r6-r7-2` (CLAUDE.md:277 TauriRender → TauriRenderer fix — the final cite drift on §3.5g item #3); three schema-lang MINORs inline; §6.5 lifecycle hygiene retense ("awaiting Ben ratification" → "RATIFIED — see status notes on each row below"). **R6-FP-7 admin-merged at PR #247 → `876f309`** with §3.6j codified at `.addl/dispatch-conventions.md:1015` and the brief-template mandate landed: reviewer/lens/mini-reviewer briefs MUST instruct agents to author top-level `disposition` at author-time, eliminating the orchestrator-catchup cycle.

R6 R8 was the terminal round. The full 17-lens roster dispatched against `876f309`; the acceptance bracket came in tight (0 BLOCKER / 0 MAJOR / ≤3 MINOR / 0 new pim-N expected); the result was **17/17 lenses APPROVE / CONVERGED / APPROVE-FOR-TAG / CONVERGED-CLEAN**. Aggregate substantive findings: 0 BLOCKER / 0 MAJOR / 4 MINOR (2 stable carries + 2 closed-inline-at-R6-FP-7-residual) / 17 OBS / 0 new pim-N. **Five lens-surfaces hit consecutive zero-finding plateaus** as the convergence signal: capability-system-reviewer at 5 consecutive zero-finding rounds (R4 / R5 / R6 / R7 / R8 = 100% reduction from R1=11) — the strongest single-lens convergence in project history per its own framing; threat-model at 4 consecutive zero rounds; security-auditor + test-coverage-auditor + browser-runtime + ux-admin-UI + plugin-arch + code-as-graph each at 3 consecutive zero rounds. The R8 pattern-induction signal was unambiguous: **"STRONGEST-EVER phase-tag-readiness CONFIRMED"**; the R8 doc-engineer signal independently: **"STRONGEST-EVER"** with explicit cross-phase comparison to Phase-2b's convergence shape (R1=64 → R6=0).

### Pre-tag sweep + tag operation

The R8 result triggered the pre-tag sweep dispatch per CLAUDE.md baked-in #15's "Phase-N close" shape. Scope was deliberately narrow: retense active-status language across canonical docs (`README.md`, `docs/PRIMER.md`, `docs/VISION.md`, `docs/FULL-ROADMAP.md`, CLAUDE.md Section 0 status header) and absorb the two trivial orchestrator-direct R8 MINORs (`r6r8-meth-1` §6.5 umbrella header retense + `r6r8-pi-1` 2-JSON top-level disposition adds catching the §3.6j rule self-applying on its own ratification round's outputs). PR #248 routed through the standard PR workflow because branch protection rejects direct push to main even for orchestrator-direct doc-only retenses; admin-merge bypassed four CI workflows that surfaced env-class failures (Phase-2a-Exit-Criteria + Multi-Arch + Cross-Browser-Determinism + admin-shell-e2e), all path-filter-irrelevant to a doc-only retense per the §4.49 documented non-required-for-merge framing. Squash commit `0ce98d0` landed; `git tag phase-4-foundation-close` against the squash commit; push to origin. **Phase 4-Foundation SHIPPED at tag `phase-4-foundation-close` on 2026-05-14.**

### Closing texture

The closing texture was paradoxical in the way Phase-2b's and Phase-3's had been: every R6 round taught a different lesson about how to do councils. R1 taught that 17 lenses on actual-shipping code reproduces Phase-2b R1's BLOCKER+MAJOR+MINOR magnitude even after thorough pre-R1 + R1 + R2 + R3 + R4 + R4b. R2 taught that paired-dispatch — a new shape of lens-reduction — has the same failure mode as Phase-2b's R2 lens-reduction misfire; Q5 cadence is the durable closure. R3 taught that the codification PR itself can violate the rule it ratifies, and that the orchestrator's own work must satisfy the discipline it codifies (§3.6h). R4 taught that "already-closed-before-ratification" is strictly stronger than same-PR-closure for §3.6h, and that forward-fire-only is a coherent exemption when origins are wave-class patterns. R5 + R6 taught that the watch-list shape (§4.57) is the right disposition for emerging-but-not-3+-recurrent patterns. R7 taught that pattern-induction's own promotion criterion can fire empirically — the 4th-instance trigger fired exactly as written. R8 confirmed terminal convergence with the deepest cross-lens signal the project had produced: five lens-surfaces at consecutive zero-finding plateaus, two lenses signaling "STRONGEST-EVER" tag-readiness independently. The 8-round arc is now the canonical Phase-4-Foundation close shape; Phase 4-Meta will inherit Q5 cadence and the §3.6h/§3.6i/§3.6j sibling family as standing process disciplines.

---

## § 2. Changelog

### Engine surface — Phase 4-Foundation production runtime LIVE at phase close

- **Plugin manifest schema + plugin lifecycle (G24-D + G24-D-FP-1/2/3):** `Engine::install_plugin` + `Engine::uninstall_plugin` lifecycle wired; `plugin_lifecycle.rs` cascade-revoke-by-issuer-DID on uninstall; `manifest_envelope_chain_validation.rs` chain-validator bridges Layer-2 `plugin_manifest::verify_envelope` ↔ Layer-3 `plugin_delegation::validate_chain` per CLAUDE.md #18 three-layer trust model; meta-plugin composition cycle detection AS REJECTION at install boundary via internal `Subgraph`-walk DFS (no new `PrimitiveKind` variant minted per CLAUDE.md #1).
- **Plugin-DID UCAN audience handle + constrained issuer (G24-D):** `did:key:...` with engine-held signing keypair (OsRng fresh per-install; no derivation from user-DID); both UCAN audience AND constrained issuer within manifest-shares envelope per D-4F-16.
- **Caller-mint-first contract (R6-FP-A closure of `sec-r6r1-1`):** `InstallContext::expected_plugin_did: &Did` parameter + Step-8 mismatch check + `plugin_did_store.get(expected_plugin_did).is_some()` handle-uniqueness assertion. The user signs the principal they're consenting to; install verifies the binding instead of overwriting it.
- **Plugin library subgraph + active references (G24-D):** `system:plugin_library:<user_did>` canonical anchor; `library_root` Read Node + per-plugin `anchor::<plugin_name>` Read Nodes + per-installed-CID `version::<cid>` Read Nodes; edges via `EDGE_LIBRARY_ANCHOR` (= `ITEM_TYPE` const-fn bound to `VocabEdge::ItemType.as_str()` per §3.5g same-language rule-mirror at R6-FP-7), `EDGE_VERSION_OF`, `EDGE_CURRENT`. Per-device-local CURRENT pointer via Loro Map per Ben ratification #2.
- **Schema compiler + typed-field-Node vocabulary (G23-A):** 8 labels (SchemaRoot / FieldScalar / FieldObject / FieldList / FieldMap / FieldEnum / FieldUnion / FieldRef) + 5 labeled edges (ITEM_TYPE / KEY_TYPE / VALUE_TYPE / REF_TARGET / VARIANT — the FIELD edge dropped at R6-FP-3 since object-to-field is implicit-via-recursion) + 8 scalars + 4 mandatory field-Node properties. Every vocabulary label maps to composition over the existing 12 primitives via the schema-compiler — no new `PrimitiveKind` variants.
- **Materializer pipeline + Renderer trait (G23-B + G24-E):** `Renderer` trait at engine boundary with transport-agnostic methods; `BrowserRender` 1st concrete impl in `benten-platform-foundation`; `TauriRenderer` 2nd impl in `benten-renderer-tauri` (12th crate); IpcAllowlist + CSP + per-method cap-binding for Tauri shape (c); materializer pipeline composed via IVM-subgraph generalization.
- **IVM Algorithm B generalization (G23-0a + G23-0b):** generalized kernel + lowering at G23-0a (`Strategy::C → Reserved` rename atomic; ErrorCode catalog string preserved); 5 hand-written Phase-1 IVM views re-expressed against the generalized kernel at G23-0b.
- **Admin UI v0 (G24-A + G24-B + G24-C):** first plugin built as app-level subgraph per CLAUDE.md #18; 4-category navigation (Plugins / Workflows / Content Types / Views) per D-4F-4; private namespace for in-progress workflow drafts; `AdminUiV0TestHarness` with composed-engine-shape (admin-UI + hostile-plugin as system:Principal Nodes via `engine.create_principal()`); workflow editor + composed-view creator; bundle-size budget ≤600KB gzipped + CI workflow `admin-ui-v0-bundle-size.yml`; user-facing field-type label mapping table at `docs/ADMIN-UI.md §2.1` (Text → FieldScalar(text), Number → FieldScalar(int|float), Date → FieldScalar(timestamp-hlc), …, Schema → SchemaRoot).
- **Thin-client session (G24-F):** `DidKeyedSession` + origin-pinning + `SessionToken` resolution; 4 new ErrorCodes minted atomically (HandshakeInvalid + ChallengeReplay + OriginMismatch + SessionExpired).
- **SUBSCRIBE-delivery cap-recheck per-event (G22-FP-1):** `Engine::on_change_as_with_cursor` consults `CapabilityPolicy::check_read` per event; auto-cancel-on-deny semantics preserved (Ben Option-C-adjacent resolution to G22-FP-1 architectural fork).
- **UCAN audience binding (G22-FP-2):** `UcanGroundedPolicy::permits_typed_proof_for` binds audience == active-principal-DID before the time-window check.
- **GrantBackedPolicy actor_cid principal-aware read (G22-FP-3 + G27-B):** `check_read` consults `ctx.actor_cid`; `WriteContext::scope` short-circuit threaded via scope-derivation lift; manifest-aware scope at G27-D.
- **`benten-id::grant_reader` sibling trait + CID-keyed companion (G27-C):** arch-r1-10 preserved (no benten-caps dep).
- **Class B β `Engine::read_node_as` carried forward:** shipped in Phase-3 pre-v1 cleanup PR #184; admin UI v0 uses exclusively per grep-assert.
- **VocabEdge::from_str defensive surface (R6-FP-7):** doc-comment retensed to accurately describe usage as "defensive surface for future schema-JSON shapes that emit edge labels directly; current parser drives edge creation via label-shape in `emit_vocabulary_edges`, not via edge-string parsing."

### Workspace — 10 → 12 crates

- **`benten-platform-foundation` (11th crate):** schema_compiler + materializer + plugin_manifest + plugin_lifecycle + plugin_library + Renderer trait. Intentionally broader than other crates per arch-r1-8 ratification — "the v1 platform-shippable surface."
- **`benten-renderer-tauri` (12th crate):** Tauri renderer impl; engine-extension per CLAUDE.md #19 compile-time trust; `tauri-runtime-verso` swap-readiness preserved in the Renderer trait surface (transport-agnostic methods).

### ErrorCodes — CATALOG_VARIANT_COUNT 118 → 168

Fifty net new ErrorCodes across the phase. Key additions: G23-A 9 codes (schema vocabulary); G23-B 3 codes (materializer); G24-D 15 codes (plugin manifest envelope, lifecycle, library, signature/key-rotation variants); G24-F 4 codes (thin-client session); R6-FP-A 4 codes (typed forensic-discrimination for consent-substitution defense, including `PluginInstallRecordPluginDidMismatch`); R6-FP-3 1 code (`E_PLUGIN_DID_HANDLE_DUPLICATE` for PluginDidStore::insert defensive return). The `E_DEVICE_ATTESTATION_FORGED_AT_PLUGIN_SHARE → E_PLUGIN_DEVICE_ATTESTATION_FORGED` rename preserved the `E_PLUGIN_*` family prefix per Ben R4 ratification. 4-surface parity held through the convergence council (168 throwable / 168 ALL_CATALOG_VARIANTS / 169 enum / 170 catalog+TS).

### Compromises — landed (closure narratives in SECURITY-POSTURE.md retense)

- **Compromise #24 (wallclock fail-closed)** — landed pre-R1 at PR #206; reaffirmed.
- **Compromise #25 (HLC-monotonic enforcement at sync layer)** — landed pre-R1 at PR #206; reaffirmed.
- **Compromise #11 (deepest e2e composition pin)** — extended at G24-A + G24-B for admin-UI Node-granularity redaction at materializer boundary.
- **Compromise #26 (manifest-envelope chain validation seam)** — landed PARTIALLY at R4b-FP-1 Seam 3 (the `apply_atrium_merge` envelope-recheck seam ships); the production `ManifestEnvelopeRechecker` adapter consuming `PluginLibrary` + `UserDidRegistry` + `validate_chain_with_manifest_envelope` deferred to Phase-4-Meta §4.36. Layer-3 ordering correction at R6-FP-5 (sec-r6r5-1: AFTER per-row cap-revocation check per CLAUDE.md #18 three-layer trust model).

### New + retensed docs (companion-doc-with-canary discipline applied)

- **NEW `docs/PLUGIN-MANIFEST.md`** — companion to G24-D canary.
- **NEW `docs/SCHEMA-DRIVEN-RENDERING.md`** — companion to G23-A/G23-B canaries.
- **NEW `docs/ADMIN-UI.md`** — companion to G24-A canary; §2.1 user-facing field-type mapping table added at R6-FP-7 closing ux-r1-12.
- **NEW `docs/future/phase-4-backlog.md`** — created during R1-triage as the canonical Phase-4 backlog destination; ~57 substantive rows by phase close.
- **NEW `docs/future/kith-decentralized-identity.md`** — Phase 5+ exploratory destination per Q3 Kith naming.
- **NEW `admin-ui-v0-threat-model.md`** — 525 LOC; 12 threat classes.
- **Retensed `docs/ARCHITECTURE.md`** — 12-crate narrative.
- **Retensed `docs/HOW-IT-WORKS.md`** — 4-identity-concepts + Kith pointer.
- **Retensed `docs/SECURITY-POSTURE.md`** — Phase-4-Foundation R1-triage refinements block; Compromise #26 narrative; Layer-3 ordering correction.
- **Retensed `docs/ERROR-CATALOG.md`** — CATALOG_VARIANT_COUNT 118 → 168 cohort math; per-cohort routing.
- **Retensed `docs/GLOSSARY.md`** — 12 new entries (4-category nav IA / Active reference / Cap-change-triggered consent / FieldEnum/Union/List/Map/Object/Ref/Scalar/SchemaRoot vocabulary / Fork / Kith / Manifest envelope chain validation / Plugin library subgraph / Private namespace / SelfRevocation attestation).
- **Retensed `docs/CRATES-DEEP-DIVE.md`** — 12-crate prose + dep-graph ASCII.

### Tooling / CI

- **Makefile `pre-push` convenience target** at repo root — discipline-only 5-check (clippy + cargo doc + cargo deny + fmt + cite-drift); NOT a git hook per Q5 ratification.
- **`admin-ui-v0-bundle-size.yml`** CI workflow — bundle-size budget enforcement.
- **Extended `cross-browser-determinism.yml`** to cover admin-UI v0 bundle.
- **Extended 3-rung baked-in #17 defense** to the two new crates (cfg-gate + workspace exclusion + bundle-content CI assertion).
- **Strategy-C batch-merge** codified at `dispatch-conventions.md §3.14` + memory `feedback_batch_merge_strategy_c.md` canonical reference.

**Phase 4-Foundation shipped at tag `phase-4-foundation-close` (`0ce98d0`, 2026-05-14).** ~30 PRs in the `phase-3-close..phase-4-foundation-close` window (#205-#248 plus pre-R5 dependabot bumps).

---

## § 3. Key takeaways — what to remember

**What this phase was fundamentally about.** Phase 4-Foundation took the engine kernel that Phases 1-3 had built (12 primitives + UCAN + did:key + Atrium sync + Class B β `read_node_as`) and gave it the load-bearing **platform surface** — admin UI v0 as the first plugin and dogfood realm, the full plugin manifest schema with three-layer consent, the typed-field-Node schema vocabulary so schemas are themselves subgraphs of primitive Nodes, the materializer pipeline composed via IVM-subgraph generalization, and the Renderer trait with concrete BrowserRender + TauriRenderer impls so the same engine runs as full peer + thin compute surface + embedded webview without architectural shape changes. The phase shipped the work needed to make the project's third deployment shape (embedded webview per CLAUDE.md #17 amendment) and the project's third-party-plugin trust posture (CLAUDE.md #18 three-layer consent + four-identity-concepts separation) production-runtime LIVE — closing the gap between "the engine exists and has a TypeScript binding" and "the platform is installable, the admin UI runs as a plugin, and users can compose tools from primitives + share them through their peer network."

**The hardest problems we hit:**

1. **The fail-OPEN default at `EngineBuilder::manifest_envelope_rechecker` (R1 BLOCKER `tmr-r6-r1-1`).** The most load-bearing cross-lens convergence of R1: four lenses (security + threat-model + plugin-arch-cap-policy + r7-spec-compliance) independently surfaced that defaulting to `None` meant any caller who forgot to wire the rechecker silently bypassed Layer-3 envelope check, gutting the 3-layer consent narrative end-to-end. Closed by flipping the default to `Some(NoopRechecker)` at R6-FP cluster; later sharpened in R3 (`cap-r6-r3-1` distinguishing noop-vs-real with `E_PLUGIN_DID_HANDLE_DUPLICATE` minted at PluginDidStore::insert); named-now at §4.36 for the real-adapter shipment in Phase-4-Meta.

2. **The consent-substitution BLOCKER `sec-r6r1-1`.** `install_plugin` minted a fresh plugin-DID at step 8 and never compared against the `plugin_did_bytes` already bound into the user's signed `InstallRecord.signing_payload` — meaning the principal the user consented to could be silently substituted at install time. Closed by the **caller-mint-first contract**: caller provides `expected_plugin_did`; install verifies binding instead of overwriting.

3. **The R6 R2 paired-dispatch misfire + Q5 cadence amendment.** The orchestrator's cost-saving paired-dispatch was the same shape of misfire as Phase-2b's R6 R2 lens-reduction. Ben caught it mid-flight and issued the Q2 full-scope corrective; the Q5 cadence amendment codifies the durable rule: every post-R1 round = full N-lens council; NO narrow iteration; convergence loop terminates only when full council returns 0 substantive findings. This is the most load-bearing process call of the entire R6 arc — it propagated through R3-R8 and continues governing Phase-4-Meta + Phase-5+.

4. **The `meth-r6-r3-1` 24-of-32-JSON schema drift (R3 MAJOR).** ADDL pipeline JSON artifacts had drifted across 32 lens + mini-review files; the brief-template guidance for `disposition` top-level had never landed; the legacy `verdict` field was the most-common substrate. Closed at R6-FP-3: `§3.6i` JSON schema discipline ratified inline; `§3.5h` JSON-artifact validation amendment; 32-file legacy sweep; brief-template mandate added.

5. **The `§3.6h` rule-ratification-against-drift orchestrator-failure-mode.** Surfaced by R3 pattern-induction as a 6-instance recurrence — rules ratifying without closing origin instance(s). Ratified inline at R6-FP-3 with the forward-fire-only exemption explicitly carved out for wave-class patterns (e.g. §3.6g's R3-R5 origin recurrence is structurally not closable retroactively). Strengthened at R4 with "already-closed-before-ratification STRICTLY STRONGER" sharpening. **The codification PR itself can violate the rule it ratifies — R6-FP-2's narrative for §3.5g item #4 contained a cite the cite-drift detector parsed wrong, surfacing as a BLOCKER at R3.** The self-application discipline is the load-bearing property.

6. **The §4.57 → §3.6j 4-instance trajectory.** Every sweep wave (R6-FP-3 / R6-FP-4 / R6-FP-5 / R6-FP-6) defined its completeness criterion against the past-state baseline and didn't re-scan post-claim — including the wave's own newly-produced artifacts. R6 R7 saw the same 4 R6-R6 JSONs still lacking `disposition` after R6-FP-6 claimed "79/79 conformant" — 5-lens cross-confirmation. Pattern-induction fired its own promotion criterion explicitly; Ben elected Path A INTENT (fold under §3.6 family with scope-criterion distinction from §3.6h); the brief-template mandate eliminates the orchestrator-catchup cycle going forward.

**What surprised us:**

- **R1 magnitude at the upper bound (~185 findings) was consistent with broad scope.** Phase-4-Foundation had absorbed substantial Phase-4 scope. The 185 magnitude is consistent with "broad scope makes R1 ~140-220 even after thorough pre-R6 work" — magnitude is a function of scope-breadth, not pre-R6 quality.

- **The codification PR's own §3.5h violation at R3.** R6-FP-2 PR #242 added the narrative for `§3.5g item #4` containing `.github/workflows/supply-chain.yml::cargo-audit` — which the detector's regex tokenizer truncated at the dash → false-positive flagged. The codification PR was its own MANDATORY-PRE-MERGE violation. This was the most pointed exemplar of why JSON-validation + GREEN-CI-CONFIRMATION amendments landed.

- **R6 R2's `methodology-3plus-recurrence.json` was itself malformed.** The pattern-induction lens that surfaced `r6r3-meta-1` JSON-validation amendment was provoked by a sibling lens's JSON-malformedness from the same round. The orchestrator-pipeline observability story has an inward-pointing failure mode: ADDL artifacts gating future-round decisions can themselves fail the schema they're checked against.

- **The watch-list pim-N candidate promoting on its own trigger criterion at R7.** Ben's Q3 DEFER call at R6 R6 was empirically tested by the very next round — and pattern-induction's promotion criterion (1+ recurrence beyond the 2-3 instances at codification time) fired exactly as written. Cleaner ratification trajectory than codifying-at-3-instances would have produced.

- **The Renderer-backend swappability pattern emerged from a CLAUDE.md #17 amendment, not planned architecture.** Ben's 2026-05-11 framing — "like storage, materializers should be generic + render-to-UI generic + Tauri/Electron/native swappable" — was the engineering consequence of adding embedded webview as the 3rd deployment shape. The Renderer trait + BrowserRender/TauriRenderer compile-time-linked-via-CLAUDE.md-#19 mechanism is a natural surface but wasn't enumerated in the original plan.

- **Capability-system-reviewer hit 5 consecutive zero-finding rounds — the strongest single-lens convergence in project history.** The lens's own framing: "Doubly-exceeds Phase-2b R6 R6 final-round three-round benchmark."

**What this phase set up for the next phase:**

- **Phase-4-Foundation is the v1 platform-shippable surface** per CLAUDE.md baked-in #15. The engine + admin UI + plugin ecosystem are now installable + usable end-to-end. Phase 4-Meta layers the self-composing admin meta-circular work + ingests Phase-3-deferred items + runs the v1-assessment-window before the `v1` tag.
- **Q5 cadence + §3.6h/§3.6i/§3.6j sibling family** inherited as standing process disciplines for Phase 4-Meta + Phase 5+.
- **22+ named Phase-4-Meta carries** across `phase-4-backlog.md §4.22–§4.51` covering install/upgrade flow refinements, materializer + IVM enhancements, atrium-share + RotationLog wiring, plugin RNG provenance + private-namespace policy, legacy `module_ecosystem::install_plugin` deletion (§4.33), production ManifestEnvelopeRechecker adapter (§4.36), v1-API-stabilization sweep (§4.43 visibility tighten `pub` → `pub(crate)`), webview-e2e Path-(a)-or-(b) tag-decision sharpening (§4.49).
- **The 12-primitive irreducibility commitment held** across the broadest scope-expansion of any phase. Zero new `PrimitiveKind` variants. Every typed-field-Node label, plugin lifecycle operation, materializer pipeline step, IVM strategy, and admin-UI workflow composes over the existing 12 primitives.
- **The three-deployment-shape commitment (#17) shipped concretely** with TauriRenderer as the first shape-(c) implementation; the Renderer trait surface preserves swap-readiness for `tauri-runtime-verso` (Servo-based webview) when Verso matures.

**Phase-defining decisions:**

1. **CLAUDE.md #17 amended** to add embedded webview as the 3rd deployment shape. Tauri 2.x ships as the first concrete shape-(c) implementation. The Renderer-backend swappability pattern is the engineering consequence — Renderer trait at engine boundary; multiple concrete impls ship as compile-time engine extensions per CLAUDE.md #19.

2. **CLAUDE.md #18 detailed plugin-identity refinements (4-identity-concepts: Content-CID + Peer-DID-signature + Plugin-DID + User-DID).** No device-DID-attestation chain for plugins. Cross-plugin/schema references use content-CID not author-DID. User-as-source signing model. Workflow ↔ plugin unification (single subgraph shape distinguished by manifest presence + sharing intent). Plugin library subgraph + active references pattern. Per-device-local CURRENT pointer via Loro Map.

3. **Q5 cadence amendment + §3.6h + §3.6j sibling family.** The most-load-bearing process calls of the R6 arc. Q5 forbids lens-reduction and paired-dispatch at phase-close; §3.6h forbids rule-codification without closing the origin instances it names; §3.6j forbids claiming sweep completeness without re-scanning the round's own outputs. The three together protect against the orchestrator-side meta-failure modes the prior phases hadn't fully named.

---

## § 4. Backlog / compromises / incomplete work

### § 4.1 Carried into this phase from earlier phases

From `docs/future/phase-3-backlog.md` (and the pre-v1 cleanup window):

| Item | Status at phase-4-foundation-close |
|---|---|
| §13.1 AtriumHandle/AtriumConfig operator-facing reference | Closed via PR #207 doc-revision |
| §13.2 benten-id consolidated public API reference | Closed via PR #207 doc-revision |
| §13.3 Phase-4 napi-surface widening (Acceptor + DeviceRevocation) | Closed cumulatively across G24-D + G24-E + G24-F + R6-FP cluster |
| §13.4 Phase-4 doc-architecture refactor (PRIMER+HOW-IT-WORKS+VISION redundancy) | Partial; PRIMER + VISION + FULL-ROADMAP retensed cleanly; full PRIMER-vs-HOW-IT-WORKS positioning consolidation rolls to Phase-4-Meta |
| §13.5 napi/TS engine-bound test parity | Partially closed at G24-F + R4b-FP-2; residuals carry |
| §13.6 phase_2a_pending_apis feature lapsing | Closed at G14-A wave during R4b-FP-1 |
| §13.7 engine_wait.rs todo!() un-stubbing | **CLOSED** at Phase-3 pre-v1 PR #184 Class B β |
| §13.8 Public-API direct-test pin gap | Partially closed at R4b-FP-1 (4 v1-shippable seams) + R6-FP cluster; residuals → Phase-4-Meta |
| §13.9 Instance 25 producer/consumer drift | **CLOSED** at Phase-3 pre-v1 PR #185 |
| §13.10 Phase-3 retrospective authoring | **CLOSED** at PR #193 (Phase-3 pre-v1) |
| §13.11 UCAN revocation observance fix | **CLOSED** at Phase-3 pre-v1 PR #199 |
| §15.2 handler-call-graph cycle detection | Pre-shipped via meta-plugin composition cycle detection at G24-D (subgraph-walk pattern reusable for handler-call-graph) |
| Compromise #11 (deepest e2e composition pin) | Extended for admin-UI Node-granularity redaction |
| Compromise #16 (SANDBOX random) | Carried; closure narrative reaffirmed |
| Compromise #18 (handler_version_chain in-memory) | Carried; durable backing → Phase-4-Meta §6.4 (ManifestStore redb-durable) |
| Compromise #23 (DeviceAttestationEnvelope V2) | Carried as LIVE |
| 14 Phase-2b + 4 Phase-3 pim-N codifications | All carried inline in dispatch-conventions.md; load-bearing across R5 + R6 |

### § 4.2 Deferred out of this phase (named into `docs/future/phase-4-backlog.md`)

About 22 new Phase-4-Meta carries clustered across `§4.22–§4.51` + `§6.4–§6.7`. Highlights:

| Section | Item | Rationale / target |
|---|---|---|
| §4.19 | 2 R4b-FP enhancement seams (plugin_did UCAN audience-handle positive arm + schema_author trust-list user-prompt) | Production exists; enhancement only |
| §4.21 | `install_plugin` Steps 9/10/11 partial-failure rollback semantics | Phase-4-Meta — destination for the stable `ds-4f-r6-r{5,6,7,8}-1` resolve_peer_dids redundancy carry |
| §4.22 | admin_ui_v0 thin-client bridge surface | Phase-4-Meta |
| §4.23 | admin_ui_v0 user-DID root-chain write-boundary validator | Phase-4-Meta |
| §4.24 | Materializer recursive walk into vocabulary edges | Phase-4-Meta |
| §4.25 | Atrium-share CID + peer-DID verification at sync layer | Phase-4-Meta |
| §4.26 | RotationLog rehydration at engine open | Phase-4-Meta |
| §4.27 | plugin_did install RNG provenance grep-pins | Phase-4-Meta |
| §4.28 | Private-namespace cross-plugin delegation policy substantive arm | Phase-4-Meta |
| §4.30 | Mini-review JSON schema discipline | **CLOSED** at R6-FP-3 (codified to §3.6i) |
| §4.32 | `validate_schema_author_within_manifest_envelope` runtime production-wiring | Phase-4-Meta |
| §4.33 | `module_ecosystem::install_plugin*` legacy-path deletion | Phase-4-Meta |
| §4.35 | install_plugin Step-9 cap-cascade atomicity gap | Phase-4-Meta |
| §4.36 | Production `ManifestEnvelopeRechecker` adapter shipment (Compromise #26 closure) | Phase-4-Meta |
| §4.37 | InstallRecord replay-defense | Phase-4-Meta |
| §4.40 | Engine-held plugin-DID private-key compromise threat-class T13 | Phase-4-Meta |
| §4.41 | caps-grew fresh-consent gate | Phase-4-Meta |
| §4.43 | Class-B β engine-internal API cluster visibility tighten (`pub` → `pub(crate)`) | Phase-4-Meta v1-API-stabilization |
| §4.45 | PluginDidStore::insert duplicate-DID defensive return | **CLOSED** at R6-FP-3 |
| §4.49 | webview-e2e MUST-FIX-OR-EXPLICITLY-ACCEPT-AT-TAG | Path-(b) explicitly-accept-at-tag elected at PR #248 |
| §4.50 | `Engine::*_for_test` suffix in production-consumed APIs cleanup | Phase-4-Meta |
| §4.55 | Storage-mutating host-fn banned-list consolidation across 3 defense surfaces | Phase-4-Meta |
| §4.56 | `Renderer::render()` no-op stub production caller | Phase-4-Meta / Phase-5 |
| §4.57 | Sweep-completeness self-verify discipline pim-N watch-list | **CLOSED** at R6-FP-7 (promoted to §3.6j) |
| §6.4 | ManifestStore redb-durable persistence | Phase-4-Meta |
| §6.5 | §3.6h pim-N (with sibling rows §3.5j stable-clippy + §3.5g #4 cross-tool config mirror) | **RATIFIED** at R6-FP-3, sharpened R6-FP-4 + R6-FP-5; row preserved for traceability |

### § 4.3 Compromises landed or accepted during this phase

- **Compromise #24** (wallclock fail-closed) — LANDED pre-R1 at PR #206.
- **Compromise #25** (HLC-monotonic enforcement at sync layer) — LANDED pre-R1 at PR #206.
- **Compromise #11 extended** for admin-UI Node-granularity redaction at materializer boundary + subscribe re-walk Option (c).
- **Compromise #26** (manifest-envelope chain validation seam) — PARTIALLY CLOSED at R4b-FP-1 Seam 3 (engine seam ships); production `ManifestEnvelopeRechecker` adapter deferred to Phase-4-Meta §4.36. Layer-3 ordering correction at R6-FP-5 (sec-r6r5-1).
- **`benten-platform-foundation` broader-posture accepted** with ARCHITECTURE.md narrative justification (arch-r1-8 ratification).

### § 4.4 Stable DISAGREE-WITH-EXPLANATION carries preserved across rounds

- **`inv-comp-r6r{5,6,7,8}-minor-1`** — pre-rename `sec-3.5-r1-N` retrospective anchor in `INVARIANT-COVERAGE.md:291` preserved across 4 consecutive R6 rounds per HARD RULE clause-(c) explicit reasoning (lens-finding-ids are frozen retrospective anchors that index into frozen pre-rename artifacts; global rename would introduce churn with zero semantic gain).
- **Cite-drift detector false-positive cluster** — 5 historical/archive narrative findings outside the cite-drift sentinel's grep scope per the test's intentional `docs/archive/` floor exclusion; preserved as `--all` invocation noise, not current-state drift. Cite-drift sentinel test (markdown-mode) consistently passes across R3-R8.
- **§4.46 wasm-browser.yml bundle-content audit grep semantics** — the lens's recommendation would over-narrow the grep; current grep is correct semantically. Verified correct at R4.
- **§4.47 admin_ui_v0_canonical_manifest() production constructor** — correctly v1-deferred; production constructor lands at admin-UI runtime wiring in Phase-4-Meta. Verified correct at R4.

---

## § 5. Process lessons / pim-N catalog

Phase 4-Foundation contributed **7 new pim-N codifications + 4 amendments + the Q5 cadence ratification**, all inline in `.addl/dispatch-conventions.md` with memory files at `~/.claude/projects/.../memory/`. The pattern that defined these ratifications: every one addressed a class of orchestrator-side or rule-codification-side failure mode, not implementation-side bugs. R1-R8 of Phase-4-Foundation was substantively a **process-codification era** more than a code-finding era — the actual code findings were closed cleanly across the R6-FP waves in proportion to R1 magnitude; what compounded was the *meta-failure pattern* of rules ratifying without closing origin, claims of sweep-completeness without semantic validation, and codification PRs themselves violating the rules they ratify.

### Pim-N codifications ratified this phase

| pim-N | § | Pattern | Origin |
|---|---|---|---|
| **§3.5g item #3** | `dispatch-conventions §3.5g` extension | Type-name cross-doc mirror — any `pub` type whose name is referenced across ≥2 docs must atomically sweep all cite sites in the same PR. Drift-defense surface: doc-coupling pre-flight + cite-drift sentinel | TauriRender → TauriRenderer 8-cite drift |
| **§3.5g item #4** | `§3.5g` extension | Cross-tool config mirror — same-language dual-config rule-mirrors (`deny.toml` ↔ CI `cargo-audit --ignore` flags; `rustfmt.toml` ↔ `.editorconfig`; `clippy.toml` ↔ `Cargo.toml [lints]`) must atomically update | R6-FP-E Wave-E 14 RUSTSEC-ignore additions to deny.toml missed mirroring CI cargo-audit step |
| **§3.5i** | NEW `dispatch-conventions §3.5i` | Mini-reviewer rebase-staleness pre-flight — every mini-reviewer brief's first action MUST be a tree-state-freshness check against merge-base with origin/main | 3-instance recurrence on R5 G23-0a / G24-D / G23-A |
| **§3.5j** | NEW `dispatch-conventions §3.5j` | Stable-Rust clippy gate as §3.5h addition — orchestrator + implementer MUST run `cargo +stable clippy --workspace --all-targets -- -D warnings` in addition to MSRV 1.95 clippy | 4-instance recurrence on PR #240 CI fix-cycles (too_many_lines + no_effect_underscore_binding + manual_contains + drop-on-Copy) |
| **§3.6g** | NEW `dispatch-conventions §3.6g` | Prior-phase pim-N codifications as explicit pre-flight checklist in next-phase R3/R5 briefs — memory-references alone don't transfer; checklist must be in the brief body | 5-instance recurrence Phase-4-Foundation R3-R5 (G24-D / G23-B / G24-A / G23-A / G24-B implementer briefs each missed at least one prior-phase pim-N) |
| **§3.6h** | NEW `dispatch-conventions §3.6h` | Rule-ratification-against-drift mandatory-close — when a new pim-N or §-codification names specific drift instance(s) as origin, the same PR/wave that lands the rule MUST close (or DEFER-NAMED-NOW per HARD RULE clause-b) the origin instances. **R4 sharpening:** "already-closed-before-ratification STRICTLY STRONGER." **R5 sharpening:** "forward-fire-only exemption" when origin instances are wave-class patterns already shipped | 6-instance recurrence visible since R1 |
| **§3.6i** | NEW `dispatch-conventions §3.6i` | Review/lens/mini-review JSON schema discipline — canonical top-level `disposition` field (NOT `verdict`) + `findings[]` array + per-finding disposition matching HARD RULE 12's three valid shapes + well-formed JSON. Brief-template mandate added | `meth-r6-r3-1`: 24 of 32 R6 lens reports used `verdict` only; 5 used `disposition`; 3 lacked both |
| **§3.6j** | NEW `dispatch-conventions §3.6j` | Sweep-completeness self-verify discipline — when claiming a sweep is COMPLETE, run the validation tool against the wave's own outputs (not just the prior-state baseline) BEFORE writing the claim. Brief-template mandate: agents author canonical top-level `disposition` at author-time | 4-instance trajectory R6-FP-3 → R6-FP-4 → R6-FP-5 → R6-FP-6, culminating in 5-lens cross-confirmation at R7 |

### Amendments folded under existing rules (not new pim-N)

- **§3.5h JSON-artifact validation** — `jq .` validation across all touched JSON artifacts pre-push. Defense-in-depth — §3.5h covers Rust/TS source validation; this extends to structured-data PR artifacts. Surfaced at R3 by malformed lens JSON (`r6-r2-plugin-arch-cap-policy-reviewer.json` object/array mismatch at line 120).
- **§3.5h GREEN-CI-CONFIRMATION substrate clause** — before admin-merge bypass per §3.14 Strategy-C, orchestrator MUST verify NEW CI failures are not regression vs pre-existing main-side baseline. Surfaced at R3 by R6-FP cluster Wave-E rustls CryptoProvider fix landing as admin-merge fix.
- **§3.6h "already-closed-before-ratification STRICTLY STRONGER"** sharpening at R4.
- **§3.6h "forward-fire-only" exemption** parenthetical at R5.

### Process discipline amendments

- **Q5 cadence amendment to `feedback_phase_close_final_council_full.md`** — every post-R1 round = full N-lens council; NO narrow iteration; convergence loop terminates only when full council returns 0 substantive findings. Ratified Ben 2026-05-13 (R6 R2 corrective). Replaces Phase-2b's "narrow iteration rounds + full only at final" framing.
- **Strategy-C batch-merge codified at `dispatch-conventions §3.14`** — when ≥3 wave-PRs accumulate unmerged simultaneously, local-merge into 2-3-wave batches + single PR per batch. Validated 7× this phase. Ratified Ben Q2 2026-05-13.
- **Path A INTENT methodology preference** — when a new rule sits structurally next to existing rules (sibling-family or extension), prefer fold-under shape with explicit scope-criterion delineation over minting a separate sibling. Codified by Q4 ratification at R6-FP-7 (§3.6j Path A INTENT promotion).

### The §3.6j origin trajectory (the load-bearing process lesson of this phase)

| Instance | Wave | Claim | Residual found at | Lens that surfaced |
|---|---|---|---|---|
| 1 | R6-FP-3 | "§3.6i verdict→disposition sweep complete (32 files)" | R6-FP-4 | orchestrator self-audit |
| 2 | R6-FP-4 | "doc-cite 11-site sweep complete" | R6 R5 (3 residuals) | doc-engineer + schema-lang cross-confirm |
| 3 | R6-FP-5 | "§3.6i 49-JSON sweep complete" | R6 R6 (4 residuals) | meth-r6-r6-1 + r6r6-pi-1 dedup |
| 4 | R6-FP-6 | "79/79 R6 JSONs §3.6i conformant" | R6 R7 (same 4 R6-R6 still missing) | 5-lens cross-confirmation r6r7-{r7,meth,pi,arch}-1 + doc-r6-r7-1 |

The pattern shape (codified at `dispatch-conventions §3.6j`): "Sweep tooling defined completeness against the PAST state (the legacy artifacts that motivated the sweep) but not against the round's own outputs. Each instance is the orchestrator's own §3.6h failure mode applied to its own scope — the rule fires AT the sweep producer."

### Pattern-induction lens trajectory R5 → R8

The pattern-induction-meta-sweep lens carried the load-bearing "phase-tag-readiness signal" across the terminal arc with monotonic strengthening: R5 = "phase-tag-ready signal STRONGEST across the phase (qualified)" → R6 = "STRONGEST CONFIRMED, contingent on r6r6-pi-1 closure" → R7 = "STRONGEST CONFIRMED + contingent on r6r7-pi-1 closure + §4.57 promotion decision" → R8 = **"STRONGEST-EVER phase-tag-readiness CONFIRMED" (unconditional)**. The convergence signal's monotonic strengthening across 4 full-council rounds validates the Q5 cadence amendment's premise — full-council-every-round produces the deepest convergence signal achievable.

### Cross-cutting mini-review failure shapes (across R5 implementer waves)

- **G24-A**: 2 BLOCKERs both pim-12 §3.6e RED-PHASE staged-pin un-ignore discipline violations. Closure: orchestrator wired production arm using available substrate per §3.6e disposition (a).
- **G23-A**: 1 BLOCKER phantom-destination ("G23-A wave-4b" embedded in production source comments — wave doesn't exist). Closure: orchestrator-direct phase-4-backlog entry creation BEFORE merge.
- **G24-D**: 2 BLOCKERs — phantom-destination-by-deletion (deleted §4.5 named destination) + parallel substance gap. Closure: 38-pin disposition fix-pass.
- **R4b L1**: 7 MAJOR phantom-destination cluster (tests cite shipped waves but un-ignore never delivered). Closure: R4b-FP-1 + Phase-4-Meta §4.19 carries.
- **R6-FP-A**: half-shipped defense surface — typed forensic-discrimination ErrorCode variant existed but had no production firing site; plugin_did_store never written by install_plugin. Closure: orchestrator inline 5-finding fix-pass.

### Producer/consumer drift instances

No new headline producer/consumer drift instances of the Phase-2b/3 shape this phase. The 25 cumulative instances from prior phases all dispositioned-and-carried; the drift-detector caught ~10 file:line + path::symbol + numeric-claim instances during R5 mini-review fix-passes, all closed inline by orchestrator-direct §3.5h MANDATORY-PRE-MERGE workspace-pre-push cycle. The §3.6h + §3.6i + §3.6j sibling family is the Phase-4-Foundation analog at the meta-failure-mode level.

---

## § 6. Decisions baked in / architectural commitments

The committee-of-decisions baked in across pre-Phase-2a / Phase-2a / Phase-2b / Phase-3-R1 / Phase-3-close pre-v1 cleanup / Phase-4-Foundation R1 triage now spans 19 items in CLAUDE.md "Architectural Decisions Baked In." Phase 4-Foundation amended #17 and detailed #18 + #19 with implementation refinements ratified at R1 triage. The 16 D-4F-N points ratified during R1 triage are the load-bearing additions.

### #17 amended — embedded webview as 3rd deployment shape

Phase 4-Foundation expanded CLAUDE.md baked-in #17 from "full peer + thin compute surface" to **three first-class deployment shapes**:

- **(a) Full peer** — native Rust on user-owned hardware. Durable storage (redb), full Atrium sync participation (iroh + Loro CRDT in `benten-sync`), SANDBOX runtime (wasmtime), persistent UCAN grant store.
- **(b) Thin compute surface** — wasm32 deployment target. Stateless reads against snapshot data; writes via fetch to a full peer. Browser tab (`wasm32-unknown-unknown`); WinterTC-compatible runtimes; future edge workers.
- **(c) Embedded webview** — native shell wraps a webview that loads the same wasm32 bundle as shape (b). Native shell IS a full peer (shape a internally); webview is the thin compute surface (shape b internally) communicating via in-process IPC. **Phase 4-Foundation ships Tauri 2.x as the first concrete shape (c) implementation.** Future swap targets: `tauri-runtime-verso` (Servo-based webview when Verso matures), Electron fallback.

The **Renderer-backend swappability pattern** is the engineering consequence: `Renderer` trait at the engine boundary; multiple concrete impls ship as compile-time engine extensions per CLAUDE.md #19. The deployment shape determines which renderer backend ships in the binary, not what the engine internals look like.

### #18 detailed — 4-identity-concepts separation + workflow ↔ plugin unification

Plugin identity = four distinct concepts, not conflated:
1. **Content-CID** — what the plugin IS (canonical bytes of subgraph + manifest)
2. **Peer-DID signature on original content** — provenance (verifiable; RotationLog handles revocation)
3. **Plugin-DID minted at install** — UCAN audience handle (NOT an attested sub-identity); just an identifier for issuing UCAN caps with `audience=plugin-DID`
4. **User-DID** — trust anchor + signs install records + issues UCAN caps with audience=plugin-DID

**No device-DID-attestation chain for plugins.** Plugin-DID is purely a UCAN audience handle. **Cross-plugin/schema references use content-CID, not author-DID.** Plugin Y declares `accepts_content: [hash1, hash2, ...]` rather than `accepts_author: [alice_did, ...]`. **User-as-source signing model.** **Workflow ↔ plugin unification** — single subgraph shape distinguished by scale + sharing intent (manifest presence); composition is recursive via meta-plugins; cycle detection AS REJECTION at install boundary via internal `Subgraph`-walk DFS — no new `PrimitiveKind` variant minted. **Plugin library subgraph + active references** — user's full plugin set lives as a real subgraph; per-device-local CURRENT pointer via Loro Map. **Versioning extends Phase-1 anchor + Version Node pattern to DAG-shape.** **Revocation reuses Phase-3 infrastructure** (UCAN per-grant + RotationLog).

**Engine-side surface — Class B β live:** `Engine::read_node_as(principal, cid)` public + engine-internal un-attributed reads via `Engine::read_node` for IVM / sync / view materialization / audit hot paths.

### #19 reaffirmed — engine extensions as compile-time-linked Rust crates

Distinct from app-level plugins. Trust model: "you compiled this into your engine binary" — same trust as Benten core. No UCAN, no manifest envelope, no `read_node_as` boundary. The boundary is `cargo` and code review. Phase 4-Foundation shipped two engine extensions: **`benten-platform-foundation`** (11th crate — schema_compiler + materializer + plugin manifest + Renderer trait) and **`benten-renderer-tauri`** (12th crate — Tauri 2.x renderer impl).

### D-4F-N cluster (16 ratifications during R1 triage)

| D-point | Decision |
|---|---|
| D-4F-1 | `benten-platform-foundation` broad-posture single-crate (vs 3-4 platform-crate decomposition) |
| D-4F-2 | Output-format pluggability via `Renderer` trait + concrete impls |
| D-4F-4 | 4-category navigation IA canonical order `["Plugins", "Workflows", "Content Types", "Views"]` |
| D-4F-11 | Materializer composition via IVM-subgraph generalization |
| D-4F-13 | Plugin manifest schema versioning field = none (CID covers shape; pull-not-push) |
| D-4F-14 | Workflow ↔ plugin ↔ schema unification under D-4F-NEW-TYPED-FIELD-NODE-VOCAB substrate |
| D-4F-16 | Plugin-DID mint pathway = `did:key:...` with engine-held signing keypair |
| D-4F-NEW-TYPED-FIELD-NODE-VOCAB | 8 typed-field-Node labels + 5 labeled edges (post FIELD-edge drop) + 8 scalars + 4 mandatory field-Node properties |
| (others) | DAG-shape version chain; per-device CURRENT pointer Loro Map; decentralized registry → Phase 4-Meta; SelfRevocation MVP; subscribe-re-walk Option (c); cap-change-triggered consent for upgrades; meta-plugin composition cycle detection AS REJECTION |

### Re-affirmations under R6 scrutiny

The R6 R1 lens roster verified all 19 CLAUDE.md baked-in commitments under broad cross-lens scrutiny. **Zero new CLAUDE.md baked-in items minted during the R6 arc** — the big plugin-identity ratifications happened pre-R1 in the 2026-05-11 conversation; R6 R1-R8 is process codification on top of the existing committed substrate.

- **#1 12-primitive irreducibility** — re-affirmed by `cag-r1-1` BLOCKER closure: every typed-field-Node vocabulary label maps to composition over the existing 12 primitives via the schema-compiler; no new PrimitiveKind variants. Phase 4-Foundation absorbed substantial scope without inverting this.
- **#2 IVM Algorithm B + Strategy boundary** — re-affirmed by G23-0a generalization; `Strategy::C → Reserved` rename atomic; the boundary precision sharpened at Phase-3 R6-R3 carries forward verbatim.
- **#3 Code-as-graph (handlers are subgraphs)** — re-affirmed and EXTENDED: plugins are subgraphs; workflows are subgraphs; schemas are subgraphs.
- **#4 Not Turing complete; DAGs only; bounded iteration** — re-affirmed by Q2 meta-plugin composition cycle detection AS REJECTION (mirrors Inv-2 DAG-ness at meta-plugin layer).
- **#7 CapabilityPolicy pluggable** — re-affirmed; `NoAuthBackend::check` defaults permit even for plugin-issued scopes per Ben-ratification cap-r1-14.
- **#8 Version chains as opt-in pattern** — re-affirmed and EXTENDED to DAG-shape for plugin/workflow/schema forks; per-device-local CURRENT via Loro Map.
- **#15 v1-milestone-gate** — re-affirmed; Phase-4-Foundation IS the v1 platform-shippable foundation; Phase-4-Meta + v1-assessment-window remain.
- **#16 SANDBOX surface for compute that doesn't fit the other 11 primitives** — re-affirmed; `cag-r1-7` pin `schema_compiler_walks_only_existing_host_fns_time_log_kv_read_random` ensures schema-compiler + materializer never request `kv:write` / `kv:delete`.

---

*Phase 4-Foundation closed 2026-05-14 at tag `phase-4-foundation-close` (commit `0ce98d0`). The project now PAUSES at the post-Phase-4-Foundation v1-milestone-gate per CLAUDE.md baked-in #15 — assess Phase 4-Meta scope + v1-assessment-window items before continuing.*
