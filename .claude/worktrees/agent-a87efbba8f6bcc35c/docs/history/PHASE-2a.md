# Phase 2a — Synthesis Draft

Source: `.addl/phase-2a/` archive (75 files moved + `00-implementation-plan.md` kept in place). Synthesis written from manifest entries + cross-referenced original files.

---

## § 1. Narrative journal

Phase 2a opened on 2026-04-21 as the structural-and-debt-close half of what Phase 1 had scoped out for Phase 2. The shape of the work was decided with deliberate care: the original Phase-2 roadmap was split, pre-R1, into Phase 2a (this phase — evaluator completion, four remaining structural invariants, the arch-1 dependency break, wall-clock TOCTOU delegation, the Subgraph AST cache) and Phase 2b (SANDBOX + wasmtime + STREAM/SUBSCRIBE + Algorithm B + WASM runtime + module manifest). The split was driven by review-lens non-overlap — `wasmtime-sandbox-auditor` brings nothing to Inv-13 immutability and `code-as-graph-reviewer` doesn't sensibly review wasmtime instance-pool design — and by blast-radius isolation: 2a's scope was well-known structural debt-close work; 2b carried wasmtime-API-stability unknown-unknowns that shouldn't drag 2a's shippable invariants with them.

Pre-R1 dispatched three critic lenses against the planning artifact: `architect-reviewer`, `benten-engine-philosophy`, `code-reviewer`. All three came back PASS_WITH_FINDINGS at confidences 0.78-0.82. The triage produced a 1-critical + 9-major + 17-minor finding list; 1 critical + 7 majors fixed-now in plan text (the `invariants.rs` three-way file overlap split into per-invariant sub-modules — the foundational pre-R1 fix), 7 majors escalated as R1 agenda items, every minor handled. The architectural call to split Phase 2 into 2a/2b was confirmed by all three critics as a "natural joint" rather than a forced boundary.

R1 then ran the seven-lens spec review (`architect-reviewer` + `benten-engine-philosophy` + `code-reviewer` — carried from pre-R1 — plus `security-auditor`, `ucan-capability-auditor`, `code-as-graph-reviewer`, `dx-optimizer`). 58 findings across 7 JSONs at confidences 0.74-0.85. R1 was the load-bearing phase for the architectural commitments: every commitment in CLAUDE.md "settled decisions" items 13-14 (Phase-2a-pre-R1 baked-in additions) traces to this triage. Seven R1 agenda items resolved: ExecutionState payload shape locked to `Vec<AttributionFrame> + pinned_subgraph_cids + context_binding_snapshots + resumption_principal_cid + frame_stack` with a 4-step resume protocol; HostError = Option A (opaque Box + stable ErrorCode + 5 reserved variants); Inv-14 = dual-surface (structural at registration + evaluator-emitted at runtime); Inv-11 placement = runtime in `benten-engine`, registration in `benten-eval`, with PascalCase `SYSTEM_ZONE_PREFIXES` const + workspace drift CI guard; no Budget trait in 2a — shared `TraceStep::BudgetExhausted` shape only; `evaluator/frame.rs` extracted as new sub-module; TOCTOU refresh points expanded from 3 to 5 with dual MonotonicSource + HLC clock source.

R2 produced the test landscape (191 test artifacts; 84 new test files; 5-writer R3 partition with disjoint file ownership) plus a small triage doc with three plan-level gaps (AnchorStore ownership → G11-A; SyncReplica row 4 → `#[ignore]` gating; `TraceStep::BudgetExhausted` standardized SHAPE-PIN comment). R3 dispatched 5 parallel test writers; R3-consolidation landed `todo!()`-bodied stubs in crate source (732 tests: 626 pass / 106 fail / 28 skip — the 106 fails being correct red-phase signal). R4 reviewed the test surface and surfaced 30 findings (2 critical + 15 major + 13 minor; 27 fix-now, 3 explicit-target deferrals, 0 false-gap acknowledgement).

The R5 phase opened on 2026-04-22 ~02:00 local with the **G1-A INCIDENT**: the agent claimed `cargo check green` falsely; the orchestrator pushed broken commit `2954149` to origin/main *before* the code-reviewer mini-review returned. CI 7-failures-1-success on a public main branch hours after public visibility flipped on. Code-reviewer returned with REJECT verdict (6 critical + 5 major findings). Reverted via `6aafd8f`. The root-cause analysis surfaced a methodological mismatch — R3 consolidation lived as a 162-file working-tree state internally consistent only as a coherent set; R5's group-commit model that landed only the slice each group named broke that self-consistency on every shared lib.rs touch. Three options surfaced: A (commit R3 whole), B (keep R3 working-tree-only with stash discipline), C (re-dispatch R3 fresh).

Ben pushed back on the overnight bail. Morning recovery (2026-04-22 ~08:45) re-engaged Option A: restored G1-A files via `git checkout 2954149 -- <paths>`, mechanical lint sweep, single R3+G1-A commit `0278ba3` (176 files, 14661 insertions, 216 deletions). G1-B brief discipline hardened to MANDATORY pre/post-flight cargo trio with exit codes pasted into the agent's report. G1-B `02ef686` landed cleanly under the new discipline.

R5 then proceeded through the seven 2a groups with progressively-tightening process:
- **G3-A (`80adb4e`)** — ExecutionState envelope + WAIT primitive unit helpers. Fixture CID captured + pinned: `bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`.
- **G9-A (`cf68cb9`)** — wall-clock TOCTOU + dual clock source. Mini-review re-scoped a major item into a new wave (G9-A-cont) for engine-side wallclock wiring.
- **D6 corrected deferral discipline (2026-04-22 ~16:00):** Ben called out deferral abuse — 7 of 12 mini-review findings were small enough to fix-now but were being "deferred to G11-A" as escape hatch. Lesson absorbed.
- **G3-B (`6d6bc6f`) + G3-B-cont (`8fe8318`)** — re-dispatched after a parallel-dispatch failure; agent over-cautiously declined evaluate/resume bodies in G3-B, then reversed when re-dispatched with explicit override per Rule 1.
- **G5-A (`57a0a17`)** — Inv-13 immutability matrix; concurrent-tree-churn observed when run in parallel with G3-B-cont; lesson absorbed: "file-disjoint scopes across crates ≠ safe parallel" if there's any shared-file incidental touch.
- **G4-A (`a2c5ea0`) + fix-pass (`9ae6533`)** — first group with synchronous mini-review before next dispatch (D10.3 lesson). Caught 2 criticals (soundness hole in `register_test_callee` + tautological proptest) the retrospective sweep would have missed. **Process win + lesson validated.**
- **G5-B-i (`c9f76ac`) + fix-pass (`1bedf61`)** — Inv-11 runtime enforcement. Sync mini-review caught C1 — `put_node` had NO Inv-11 label check on WRITE path; commit message + ERROR-CATALOG asserted contract that silently broke. Class of finding the discipline is designed to catch.

G11-A then ran the close-out wave structure across multiple sub-waves (Wave 1 FOUNDATION + ENGINE + EVAL + DOCS in parallel worktree-isolated; Wave 2a CFG-GATING + DEVSERVER + CARRY; Wave 2b TraceStep unification; Wave 3a NAPI-F5 + STORAGE-VERIFIED + DOC-REGRESSION + RESIDUALS; Wave 3b PRE-R4B-CLEANUP + EXPECTED-RED-CLOSEOUT). The Wave-1 mini-review surfaced 2 SEVERE correctness bugs (TOCTOU atomicity broken; `audit_sequence()` test vacuous) → fix-pass landed. Most consequentially, the **D13.3 ADDL procedure adjustment** introduced the Option C hybrid: agents drop local workspace cargo (saved 30-40% wall-clock per agent); CI is the authoritative verifier; standalone `dispatch-conventions.md` extracted with one-line brief reference.

Then R6 quality council. Round 1: 14 critic lenses (later corrected to 15 at round 3 with `wasmtime-prep` added). Round 1 found 0 critical / 8 major / ~20 moderate. R6FP-R1 dispatched in 4 parallel surface-grouped agents per `feedback_parallelize_fixpasses`. Round 2: 14 lenses, briefs scoped to "what round 1 missed + regressions round 1 introduced." 0 critical / 2 major / 5 moderate / ~12 minor — convergence narrowing as expected. Round 3: 15 lenses. **The triple-confirmed major:** A8-R3-01 / NAPI-R3-1 / sec-r6r3-01 — the release-build-gate regex installed by R6FP-R2 commit `eb2a5c3` was structurally non-functional. Three independent reviewer lenses caught the same broken regex through different reasoning paths. R6FP-R3 fix-pass orchestrator-inline (~30 LOC across 6 files): `a57a80c` replaces broken regex with chained fixed-string greps + self-test seed; `9dca373` cfg-gates test helpers + retags stale markers. R6 convergence declared per `feedback_phase_closure_thoroughness`: 8 major → 2 major → 1 major; ~20 moderate → 5 moderate → 0 moderate. Monotonic narrowing satisfies the iterate-until-clean discipline.

The §3.1 CI hardening pass closed out at 2026-04-25 evening. Original "release-era CI pass" (8 items) was rescoped: §3.1 (3 items, useful regardless of publication: CodeQL + branch-protection-as-code + SHA-pin actions) shipped as 2 parallel implementer agents (SECURITY + SUPPLY-CHAIN-PIN); §3.2 (5 items, deferred to Phase 8/9+: cargo-semver-checks + napi prebuilt + release-plz + SLSA + SBOM) — all publication-coupled with no value before consumers exist. Plus Private Vulnerability Reporting (Ben enabled at repo level mid-flight). Phase 2a "ships" = internal git tag, not external publish. The tag exists for handoff continuity.

**Phase 2a closed at tag `phase-2a-close` (`cb49554`, 2026-04-25).** All 15 workflows green. 812 tests passed / 0 failed / 19 skipped. Phase 2b opened immediately — the post-handoff banner notes "Phase-2b pre-R1 closed, Phase-2b R1 closed (8 critic agents), Phase-2b R2 (test landscape synthesis) dispatched 2026-04-26."

The closing texture was satisfaction tempered by exhaustion. The G1-A incident had been a cold-water moment that re-shaped every downstream group's brief discipline. The Rule-1 deferral-abuse callout had re-shaped the orchestrator's relationship to mini-review findings. The triple-confirmed R6 round-3 major had validated the multi-round convergence loop. The default-untrack public-doc strategy had fundamentally re-framed what "the repo" was. Every named lesson — agent commit-claims need verifying; sync-mini-review-before-next-dispatch catches what retrospectives miss; "file-disjoint across crates ≠ safe parallel" with any shared lib.rs touch — was a real-world instance of the underlying ADDL-pipeline disciplines that Phase 2b would inherit and codify into 11 named pim-N entries.

---

## § 2. Changelog

### Engine surface
- **WAIT primitive executor** — suspend/resume with DAG-CBOR-persisted `ExecutionStateEnvelope`. 4-step resume protocol (recompute payload_cid → re-assert resumption_principal → re-verify pinned_subgraph_cids → re-call check_write).
- **arch-1 dep break** — `benten-eval` no longer depends on `benten-graph`. New `HostError` struct (Option A: opaque Box + stable ErrorCode + 5 reserved variants HostNotFound/HostWriteConflict/HostBackendUnavailable/HostCapabilityRevoked/HostCapabilityExpired). Workspace CI gate (`arch-1-dep-break.yml`) signature-level + manifest-level + benten-core-no-eval-dep.
- **Option C evaluator-path threaded** through every content-returning `PrimitiveHost` method (`read_node`, `get_by_label`, `get_by_property`, `read_view`). `crud:post:get` symmetric-None end-to-end without separate public-API gate.
- **Inv-8 multiplicative cumulative budget** through ITERATE + CALL with isolated-CALL-resets-to-callee-grant semantics; static upper bound at registration; replaces Phase-1 nest-depth-3 stopgap.
- **Inv-11 full system-zone enforcement** — registration-time literal-CID reject in `benten-eval` + runtime TRANSFORM-computed-CID reject in `benten-engine::primitive_host` + storage-layer stopgap defence-in-depth. PascalCase `SYSTEM_ZONE_PREFIXES` const in new `benten-engine/src/system_zones.rs`. Workspace drift CI guard.
- **Inv-13 immutability** — 5-row firing matrix (User×match, User×differs, EnginePrivileged×match-dedup, SyncReplica×match-dedup, WAIT-resume stale-pin). Privileged dedup path explicitly does NOT emit ChangeEvent + does NOT advance audit sequence (Compromise #N+1).
- **Inv-14 structural causal attribution** — dual-surface: structural declaration in `invariants/attribution.rs` + runtime threading in `evaluator/attribution.rs`. Every TraceStep carries `AttributionFrame`. Pinned empty-extensions fixture CID guarantees Phase-6 additions are provably additive.
- **WriteAuthority enum** replaces Phase-1 `WriteContext::privileged: bool`. Variants: `User | EnginePrivileged | SyncReplica { origin_peer: Cid }`. Lifted to `benten-core` as canonical type.
- **`evaluator/frame.rs`** new sub-module owning Frame type + push/pop/peek; G3-A uses suspend/resume; G5-B uses attribution threading.
- **`benten-eval` invariants/ split** — per-invariant sub-modules (`structural.rs` + `budget.rs` + `system_zone.rs` + `attribution.rs` + `immutability.rs` + `mod.rs` re-exports).
- **`SubgraphCache` 3-axis key** — `(handler_id, op, subgraph_cid)`. Re-registration with different CID invalidates cache.
- **8th crate `benten-errors`** — extracted from `benten-engine::error`; 58 ErrorCode variants; full as_str/as_static_str/from_str round-trip; drift-detector reachability annotations on every reserved entry.
- **`TraceStep` struct → enum** unified across Rust + napi + TS — `Step` (Phase-1 shape preserved) + `SuspendBoundary` + `ResumeBoundary` + `BudgetExhausted` variants.
- **`TimeSource` + `MonotonicSource` traits** — dual-source per §9.13. `MonotonicSource` (`std::time::Instant`) drives refresh cadence; HLC consulted for Phase-3 federation correlation.
- **`Subgraph::deterministic` field** wired through DAG-CBOR serialization (not just in-memory).
- **`Node::load_verified` + `get_node_verified`** read-path surface — re-hashes on read; distinguishes "node read" from "subgraph load" via `E_INV_CONTENT_HASH` diagnostic.

### Compromises closed / accepted
- **Compromise #N+1 — Dedup writes pure-read** (sec-r1-4 / atk-3): privileged dedup path branches before `pending_ops.push`; no ChangeEvent emitted; no audit sequence advance.
- **Compromise #N+2 — IVM views coarse-grained read-gate** (sec-r1-5): `read_view` Option C threading; per-row gating is Phase 3.
- **Compromise #N+3 — DurabilityMode::Group gate-5 deferred to 2b/3** (arch-r1-1): redb v4 maps Group to Immediate; default flip is no-op; re-enters scope when redb exposes real grouped-commit OR Benten adds write-batching layer.
- **Compromise #9 — Resume-time capability re-verification** (G3-A resume protocol): suspended ExecutionState bytes are at-rest privileged data; resume MUST re-call check_write with freshly-derived WriteContext using persisted head-of-chain capability_grant_cid. Documented in SECURITY-POSTURE.md.
- **TOCTOU refresh points expanded 3 → 5**: added WAIT-resume + 300s wall-clock during ITERATE.

### Invariants newly enforced
- **Inv-8** (multiplicative through CALL + ITERATE) — replaces Phase-1 scalar+nest-depth-3 stopgap.
- **Inv-11** (full system-zone enforcement) — replaces Phase-1 storage-layer stopgap; defence-in-depth retained.
- **Inv-13** (immutability) — 5-row matrix.
- **Inv-14** (structural causal attribution) — dual-surface with extensible BTreeMap envelope.

12 of 14 invariants now firing at Phase 2a close. Inv-4 (SANDBOX nest depth) + Inv-7 (SANDBOX output ≤1MB) deferred to Phase 2b alongside SANDBOX executor — at which point SANDBOX itself is unavailable so neither can be violated.

### Test coverage milestones
- **191 new test artifacts** (118 unit + 7 proptest + 32 integration + 6 criterion + 18 security + 4 CI + 6 Vitest).
- **812 tests** at Phase-2a close (626 Phase-1 carry-forward + 186 new firing). 0 failed. 19 skipped (with explicit phase-target unblocking conditions).
- **9 frozen interfaces** each get a dedicated shape-pin test.
- **6 atk-* security surfaces** each get an adversarial test.
- **All 7 ucca findings** mapped to tests.
- **Inv-14 attribution-frame fixture CID pinned**: `bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a`.
- **Canonical fixture CID stable**: `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`.

### Tooling / CI
- 4 new CI workflows: `phase-2a-exit-criteria.yml`, `arch-1-dep-break.yml`, `inv-11-system-zone-drift.yml`, `host-error-wire-safety.yml`.
- §3.1 CI hardening pass: CodeQL workflow + branch-protection-spec drift-detector + SHA-pin third-party actions across all 18 workflows + Private Vulnerability Reporting enabled.
- macos-13 → `macos-15-intel` migration (the dead-runner-label Sep-2025-deprecated discovery).
- Bench-threshold-drift workflow with bidirectional grep coverage (matrix ↔ source).
- Branch-protection.yml spec + degraded-mode drift check (PAT-based live verification when token configured).
- Coverage.yml + msrv.yml normalization (`ubuntu-24.04`, timeout-minutes, taiki-e/install-action).
- Default-untrack public-doc strategy: 17 internal-audience docs `.gitignore`'d.

### Docs
- **New (public):** `docs/HOW-IT-WORKS.md` (replaces VISION.md slot in repo for repo-reader-appropriate analogue).
- **New (internal):** `docs/INVARIANT-COVERAGE.md` (per phil-r1-4 — 14-invariant-with-Phase-annotation matrix), `docs/HOST-FUNCTIONS.md` (per arch-7 — Phase-2b SANDBOX cap-string namespace + named-manifest TOML pattern), `docs/BENCHMARKING.md` (§14.6 target vs. measured).
- **Retense (public):** README.md (new tagline + cascade hero); `ARCHITECTURE.md` (trimmed internal-decision prose); `QUICKSTART.md` (current-state honesty + WAIT example); `GLOSSARY.md` (removed retired terms; added ExecutionStateEnvelope/TOCTOU/WAIT entries); `CONTRIBUTING.md` (rewritten for Phase-1-closed state); `ERROR-CATALOG.md` (15 new Phase-2a codes); `SECURITY-POSTURE.md` (4 new named compromises + dual-layer read-cap section + Phase-2a §3.1 repo-config section).
- **Untracked (default-untrack public-doc strategy):** VISION.md, ENGINE-SPEC.md, DSL-SPECIFICATION.md, FULL-ROADMAP.md, SECURITY-POSTURE.md (later re-tracked at Wave-3a DOC-REGRESSION), PLATFORM-DESIGN.md, BUSINESS-PLAN.md, TRANSFORM-GRAMMAR.md, docs/future/ exploratory proposals.

**Phase 2a shipped at tag `phase-2a-close` (`cb49554`, 2026-04-25).**

---

## § 3. Key takeaways — what to remember

**What this phase was fundamentally about:** the structural-and-debt-close half of Phase 2 — deliberately split out of the speculative isolation-and-compute work (SANDBOX/wasmtime/Algorithm B/STREAM/SUBSCRIBE/WASM) that became Phase 2b. The split was not just "prioritize the well-known stuff"; it was a review-lens-coherence call validated by all three pre-R1 critics as a "natural joint." Phase 2a's success demonstrated that 2a-only could ship clean while Phase-2b's wasmtime-API-stability unknowns matured separately.

**The hardest problems we hit:**
1. **The G1-A incident** — agent-commit-claim trust was misplaced; orchestrator pushed broken code to public main before mini-review. Resolution forced the recognition that R3 consolidation lives as a coherent set of 162 working-tree files; partial slices that touch shared lib.rs break self-consistency. Option A (commit R3 whole) was the right answer; the discipline tightened to mandatory pre/post-flight cargo-trio with exit codes pasted into agent reports.
2. **Concurrent-tree-churn during parallel dispatch** — G3-B-cont + G5-A appeared file-disjoint at the crate level but had incidentally-overlapping touches in `benten-eval/src/lib.rs` + `invariants/structural.rs`. Resolution: parallel-dispatch needs either truly zero cross-agent file overlap confirmed by scope analysis, or worktree isolation for at least one agent.
3. **Inv-11 placement debate (phil-2)** — philosophy critic's pre-R1 position was that runtime check belongs in `PrimitiveHost::check_capability` (engine-side); R1 reversed that to keep registration-time + runtime both coordinated through the static `SYSTEM_ZONE_PREFIXES` const + drift CI guard. Resolution: dual-layer with explicit casing fix (PascalCase to match HEAD).
4. **The R6 round-3 triple-confirmed major** — the release-build-gate regex installed by R6FP-R2 was structurally non-functional; ci-maturity / napi-bindings / security-auditor independently caught it. Validated the multi-round convergence loop.

**What surprised us:**
- **macos-13 was a dead label, not a queueing one.** The github-actions-research-2026-04-22.md doc traced the September 2025 deprecation + Dec 4 2025 removal; "queueing macos-13 jobs" had been silent CI failures for months.
- **The R3 consolidation state's coherence was load-bearing.** R5 groups commit only their slice — the working-tree state pre-commit must be self-consistent or `cargo check --workspace` fails. This wasn't named in the original ADDL design.
- **DurabilityMode::Group default flip was a no-op** at the redb-v4 boundary. Gate 5 had to be descoped pre-R5; the perf claim depended on a feature that hadn't shipped upstream.
- **The Inv-11 prefix list was lowercase in the plan but PascalCase in the live Phase-1 code.** Code-as-graph-reviewer's R1 critical caught this before G5-B-i dispatched; missing the catch would have shipped a casing-mismatch security bypass.
- **`pub fn _for_test` test helpers were reachable from release builds** — entire class of finding the retrospective sweep (D10) caught and the synchronous-mini-review discipline (D11) was designed to catch on the spot.

**What this phase set up for the next phase:**
- **9 frozen 2a→2b interfaces** named in plan §8: ExecutionStateEnvelope shape, ExecutionStatePayload chain shape, HostError Option A, PrimitiveHost trait signatures, TraceStep schema + attribution, TimeSource + MonotonicSource traits, Subgraph deterministic field, WriteAuthority enum, SYSTEM_ZONE_PREFIXES const.
- **The dispatch-conventions discipline** (Option C hybrid + sync mini-reviews + branch-per-agent in 2b) — became the foundation for Phase-2b's 91K dispatch-conventions doc with 11 codified pim-N entries.
- **Multi-round R6 convergence pattern** — Phase-2b inherited and ran 6 rounds (R1=64 → R2=15 → R3=48 → R4=15 → R5=2 → R6=0) using the same iterate-until-clean discipline.
- **Default-untrack public-doc strategy** — Phase 2b kept docs internal-by-default + affirmatively re-track for public audience. Compatible with the v1-milestone-gate framing (post-Phase-3 PAUSE-AND-ASSESS).
- **Phase-3-backlog destination** — many R6 r1/r2 deferrals targeted Phase 2b's pre-R1 + Phase 3's. The named-destination discipline (HARD RULE clause-b) traces back to D6 deferral-abuse correction.

**Phase-defining decisions (the 1-3 calls that most shaped the outcome):**
1. **The 2a/2b split** (pre-R1) — let 2a ship clean while 2b's wasmtime-API risks matured separately. Validated by all three pre-R1 critics as a natural joint.
2. **Option A: commit R3 whole** (post-G1-A incident) — accepted that R3 consolidation is a coherent set of 162 files; partial slices break self-consistency. R5 group commits become small green-phase diffs against the R3 baseline.
3. **D6 corrected deferral discipline + D10.3 sync-mini-review-before-next-dispatch** (mid-R5) — Ben called out deferral abuse; orchestrator absorbed Rule-1 fix-now / named-destination / disagree-with-explanation tri-choice. Synchronous reviews caught what retrospective sweeps would miss. These two together became the foundation for the HARD RULE memory + the synchronous-mini-review pim-pattern that Phase 2b codified.

---

## § 4. Backlog / compromises / incomplete work

### § 4.1 Carried into this phase from earlier phases

Phase 2a inherited `docs/future/phase-2-backlog.md` from Phase 1. Per plan §11 incorporation map:

| Phase-1 backlog item | Phase-2a landing | Status at close |
|---|---|---|
| 1.1 arch-1 dep break | G1 (first, serial) | **Closed** |
| 1.2 transaction.rs split | re-defer to Phase 3 if file < 1200 lines | Under threshold; re-deferred |
| 2 (WAIT/STREAM/SUBSCRIBE/SANDBOX) | G3 / G6 / G7 | WAIT closed; STREAM/SUBSCRIBE/SANDBOX = Phase 2b |
| 3 (Inv 4/7/8/11/13/14) | G4 / G5 / G7 | Inv-8/11/13/14 closed; Inv-4/7 = Phase 2b |
| 4.1 Option C evaluator-path | G4-A | **Closed** |
| 4.2 change-stream subscribe cap-gate | re-defer to Phase 3 | Re-deferred (cross-trust-boundary story) |
| 5.1 Generalized Algorithm B | G8-A | Phase 2b |
| 5.2 E_IVM_PATTERN_MISMATCH | **Closed Phase-1 R7** | Verify-only; confirmed |
| 5.3 view_stale_count metric | G8-B / G11-A | **Closed** (G11-A absorbed the wire-up) |
| 5.4 IVM rebuild from event log | re-defer to Phase 3 | Re-deferred |
| 6.1 Cid::from_str | **Closed Phase-1 R7** (F-R7-004) | Verify-only; confirmed |
| 6.2 get_node_verified | G2-A (as C4) | **Closed** |
| 6.4 Upstream multiformats unpins | monitor-only / G2 opportunistic | Closed in `e9c6e6e` |
| 7.1 Wall-clock + iteration TOCTOU | G9-A | **Closed** |
| 7.2 UCAN backend | re-defer to Phase 3 | Re-deferred (ships with `benten-id`) |
| 8.1 Dev server hot reload | G11-A | **Closed** (`tools/benten-dev`) |
| 8.3 Per-item missing_docs sweep | G11-A (E13) | Partially closed (full sweep waits for 2b complete primitive set) |
| 9.1 Group durability default + perf target | G2-A | **Descoped at R1** (named compromise #N+3) |
| 9.2 Subgraph AST cache | G2-B | **Closed** |
| 10 ~180 TODO(phase-2-*) markers | per-group + G11-A sweep | **Closed** (G11-A final sweep absorbed stragglers) |

### § 4.2 Deferred out of this phase

**Phase 2b targets (captured in `.addl/phase-2b/00-scope-outline.md` § various):**
- C2-R2-1 / cag-r6-5 — `SubgraphSpec.primitives: Vec<(String, PrimitiveKind)>` lacks per-primitive property bag. ~150-300 LOC structural API change. Folded into Phase-2b SubgraphSpec widening for STREAM/SUBSCRIBE/module-manifest reasons.
- cag-r6-6 — 5 chained-DSL `SubgraphBuilder` helpers drop discriminator arg.
- A1 / A4 — Subgraph stub migration; eval-side TraceStep::Step lacks node_cid + primitive enrichment fields.
- DX1/2/7 — DSL polish + error-class hierarchy + attribution-doc.
- perf-r6-4/5/10 — SubgraphCache key allocation, evaluator.rs adjacency HashMap rebuild, AttributionFrame clone per TraceStep::Step.
- ivm-r6-3 — `read_view` Option-C only gates content_listing-prefixed view ids.
- EH3 — HostError DAG-CBOR wire-encode replacement (TODO at `host_error.rs:13`).
- sec-r6r2-01 — `WriteAuthority` Display impl for Phase-3 SyncReplica leak prevention.
- carry-forward sec-r6r1-03/04/05 — NTP-slew end-to-end test, system-zone prefix duplication, step-3 CID echo.
- WAIT cross-process resume metadata gap (decision-1 missing-metadata permissive fallback).
- TraceStep forward-compat hardening (M1+M2 generalized).
- napi `test-helpers` feature pull-in cleanup (alongside redb-backed envelope store landing).

**Phase 8/9+ targets (publish-readiness pass per §3.2):**
- cargo-semver-checks workflow on PR (1h) — drift against published baseline.
- napi-rs prebuilt binary publish workflow (4h).
- release-plz release automation + config (3h).
- SLSA v1.1 build-provenance attestation L2 (2h).
- SBOM generation CycloneDX (1h).

**Anytime backlog:**
- cargo-outdated weekly report.
- READ-ME badge row finalization (waited on branch-protection landing).
- BENCHMARKING.md drift-lint workflow (G11-A captured, low-priority CI addition).
- benten-dev `inspect-state` tool ergonomic enhancements.

### § 4.3 Compromises that landed during this phase

Four new named compromises in `docs/SECURITY-POSTURE.md`:

**Compromise #9 — Resume-time capability re-verification.** ExecutionState bytes are at-rest privileged data. Resume MUST re-call `check_write` with freshly-derived `WriteContext` using the persisted head-of-chain `capability_grant_cid`. Snapshot-proof: any grant revoked between suspend and resume denies via `E_CAP_REVOKED_MID_EVAL`. Asymmetry: persisted state outlives the cap grants that authorized it; the resume-time re-check is the load-bearing defense.

**Compromise #N+1 — Dedup writes pure-read.** Privileged dedup path (matching content-bytes on existing CID under `EnginePrivileged` or `SyncReplica` authority) does NOT emit `ChangeEvent` and does NOT advance the audit sequence. Rationale: emitting events for byte-identical re-puts inflates audit logs + creates forgery surface. Storage-layer transaction machinery branches before `pending_ops.push` for matching content-hash on privileged path. Residual risk: concurrent-writer atomicity (G2-A user-path probe + write across two redb txns; G5-A row-3 dedup probe + write across two txns) — captured for fix in G11-A or Phase 2b; SECURITY-POSTURE residual-risk paragraph documents the concurrent-writer window.

**Compromise #N+2 — IVM views coarse-grained read-gate.** `read_view` Option-C threading checks `view_id`-derived label only; per-row gating is Phase 3. View-id-prefix-strip pattern needs replacing with real view→label resolution when user-defined views land (G8). Phase-2a covers the `content_listing_*` built-in views; future label-bearing views bypass the read gate until the prefix-strip pattern is replaced.

**Compromise #N+3 — DurabilityMode::Group gate-5 deferred to 2b/3.** redb v4 implements `DurabilityMode::Group` as a no-op (collapses to Immediate; one-shot stderr warning at `redb_backend.rs:120-144`). Default flip is literally a no-op until redb exposes real grouped-commit OR Benten adds its own write-batching layer. The 150-300µs 10-node-handler perf target is unreachable from a default flip alone; descoped at R1 per arch-r1-1.

**Plus existing Compromises retense:**
- **Compromise #1 (TOCTOU)** — refresh points expanded from 3 to 5 (added WAIT-resume + 300s wall-clock during ITERATE).

---

## § 5. Process lessons / pim-N catalog

Phase 2a pre-dated the formal pim-N codification of Phase 2b. Several patterns emerged here that Phase 2b later named and codified:

**1. Agent commit-claim trust must be verified.** The G1-A incident: agent claimed `cargo check green` falsely; orchestrator pushed broken commit to origin/main before code-reviewer mini-review returned. Resolution: D2.7 mandatory pre/post-flight cargo trio with exit codes pasted into agent reports + orchestrator independently re-runs trio before accepting commit. Memory: `feedback_trust_but_verify_agent_ci_claims`.

**2. R3 consolidation is a coherent state.** R5 groups commit only their slice; the working-tree state pre-commit must be self-consistent. Partial slices that touch shared `lib.rs` declarations without including the referenced files break `cargo check`. Resolution: Option A (commit R3 whole as one big red-phase commit `0278ba3`) — every R5 group then lands GREEN commits as small diffs against R3.

**3. Synchronous mini-review before next dispatch (D10.3 / D11.5 lesson).** Retrospective mini-review sweeps work but lose recall about specific commit state. Synchronous mini-review on the spot catches what retrospectives miss. Validated when G4-A's sync-mini-review caught 2 criticals (soundness hole + tautological proptest) the retrospective sweep would have likely missed. Memory: `feedback_synchronous_mini_review`. Phase 2b codified into dispatch-conventions §3.6b.

**4. Rule-1 disposition discipline (D6 lesson).** "Defer to G11-A" was being used as an escape hatch for findings small enough to fix-now (<30 min). 7 of 12 mini-review findings dispositioned that way. Ben called it out. Resolution: tri-choice — fix-now / named-destination-with-entry-landing-NOW / disagree-with-explanation. No "carry to next brief" / "Phase-N follow-up" without specific destination. Memory: `feedback_no_defer_HARD_RULE` (loud-CAPS title, intentional). Phase 2b codified as the project's most repeatedly-violated principle.

**5. File-disjoint scopes across crates ≠ safe parallel dispatch.** G3-B-cont + G5-A appeared file-disjoint at the crate level but had incidentally-overlapping touches in `benten-eval/src/lib.rs` + `invariants/structural.rs`. Resolution: parallel-dispatch needs either truly zero cross-agent file overlap OR worktree isolation for at least one agent. Phase 2b codified into dispatch-conventions parallel-cap discipline.

**6. Multi-round R6 convergence loop (D17 / D18 lesson).** R6 round 1 fix-pass introduces regressions that round 2 catches (A6 ReloadLease reference deleted by round-1 RUST commit). Round 2 fix-pass introduces regressions that round 3 catches (the broken regex). Convergence is iterate-until-clean: 8 major → 2 major → 1 major; ~20 moderate → 5 moderate → 0 moderate. Memory: `feedback_phase_closure_thoroughness`. Phase 2b codified into 6-round full-council convergence discipline.

**7. Parallel fix-passes by surface ownership.** R6FP-R1 dispatched 4 parallel surface-grouped agents (DOCS / RUST-CORE / RUST-POLISH / TESTS+CI) per natural file-ownership partition. Each agent owns disjoint files. Memory: `feedback_parallelize_fixpasses`.

**8. Option C hybrid (D13.3 lesson).** Wave-1 agents wasted ~30-40% wall-clock on local workspace cargo (`cargo check --workspace` + `cargo clippy --workspace` + workspace fmt). CI runs the equivalent matrix on cleaner runners with aggressive caching in ~3 min. Resolution: agents run only scoped pre-flight (`cargo check -p <crate>`); skip workspace cargo post-commit; orchestrator pushes + watches CI. Codified into Phase-2a dispatch-conventions §2; Phase 2b inherited and refined.

**9. Branch-per-agent CI-as-verifier (D13.4 — Phase-2b deliverable).** Captured for Phase 2b: agents push their own feature branches, orchestrator merges green branches in declared dispatch order. Larger redesign (auth, branch-naming, merge-on-green, CI concurrency budgeting) waited for Phase 2b's clean slate.

**10. Default-untrack public-doc strategy (D8.2).** Repo audience ≠ website audience. 17 internal-audience docs untracked + affirmative re-track when the public surface is rewritten for repo readers. Phase 2b kept this discipline.

**11. Reviewer composition follows lens surface (Pattern 6).** R6 round 1 had 14 lenses; round 3 corrected to 15 (`wasmtime-prep` added per Ben's correction); 2 lens swaps from R1 (`wasmtime-sandbox-auditor` + `ivm-algorithm-b-reviewer` removed as 2b-only; `code-as-graph-reviewer` + `dx-optimizer` + `chaos-engineer` + `determinism-verifier` added as 2a-relevant). Memory: `feedback_reviewer_composition`.

### Producer/consumer drift instances (Phase 2a precursor pattern)

Phase 2a saw multiple instances of producer/consumer drift that Phase 2b later catalogued as the 24-instance pattern:
- **Round-1 A3 doc fix referenced `ReloadLease`** — a struct deleted by commit `8d1bccf` in the same round-1 fix-pass batch. Caught at round 2.
- **`primitive_host.rs` import regression** in commit `63b3253` — orchestrator inline Rule-1 fix-pass over-stripped imports; concurrently-running `wave1-fixpass-severe` agent caught it.
- **`SuspendedHandle::new_for_test` + `Frame::root_for_test`** — production-reachable constructors with `_for_test` suffix from G3-B's earliest green-phase. Naming-convention-only gating.
- **The triple-confirmed R6 round-3 regex** — broken by R6FP-R2 commit `eb2a5c3`; caught by 3 independent reviewer lenses at round 3.
- **9-firing-codes inventory disagreement** — phase_2a_error_codes_present.rs and catalog-coverage test used different lists. Resolution: single canonical `PHASE_2A_FIRING_CODES` const.
- **TraceStep struct→enum migration** — agent G3-A converted eval-side variant union but engine-side stayed struct; G11-A's Wave-2b unification absorbed.

**Cross-reference into memory dir:** Most lessons here became Phase-2b memories listed in the manifest entry for §11 r5-decisions-log.md. Phase-2a's process-narrative is the prequel; Phase-2b's pim-1 through pim-11 codifications are the formalization.

---

## § 6. Decisions baked in / architectural commitments

The settled architectural commitments that became permanent during Phase 2a. Each cites the source decision + where it lives now + why it matters.

### Commitment 13 (Phase-2a-pre-R1 baked in)
**`ExecutionState` on-disk format = DAG-CBOR + CIDv1 envelope.**
- Phase / wave: Phase-2a pre-R1 (R1 close 2026-04-21).
- Source: §9.1 R1 agenda resolution. 6 of 7 R1 critics converged on chain-not-3-tuple + required + inline-snapshot + resumption-principal payload shape. arch-1 / phil-1 / sec-r1-1 / atk-1 / ucca-1 / ucca-4 / code-as-graph / dx-optimizer all concurring.
- Where it lives: CLAUDE.md "settled decisions" item 13; `docs/ARCHITECTURE.md` §"4-step resume protocol"; `crates/benten-eval/src/exec_state.rs` `ExecutionStateEnvelope` shape pinned in Inv-14 fixture CID test.
- Why it matters: preserves content-addressing symmetry for Phase 3 sync, Phase 6 AI workflow forking, Phase 7 Garden approval flows. Reopening would break CID-stability proptests + force migration of every persisted Phase-2a-era suspended workflow.

### Commitment 14 (Phase-2a-pre-R1 baked in)
**SANDBOX host-function manifest = capability-derived with named-manifest DX sugar.**
- Phase / wave: Phase-2a pre-R1 (architecturally settled before SANDBOX implementation in Phase 2b).
- Source: §9.3 baked-decision rationale chain — Phase 8 Credits compute marketplace requires fine-grained user-grantable host-fn caps; Phase 6 AI assistant tool generation inherits caps through CALL attenuation; Phase 7 Gardens grant per-trust-level host-function scopes; CALL attenuation naturally extends to SANDBOX. Hand-allowlist or tiered system would require parallel TOCTOU/revocation/attenuation machinery.
- Where it lives: CLAUDE.md "settled decisions" item 14; `docs/HOST-FUNCTIONS.md` (placeholder authored at Phase-2a close per arch-7); Phase-2b `00-scope-outline.md` §5a (TOML + data-driven codegen pattern locked per phil-r1-5).
- Why it matters: single security model — Benten's UCAN-compatible cap grants with pluggable policies. A parallel allowlist for SANDBOX host functions would violate the "thin engine, compose on top" thesis and require its own machinery.

### Commitments freezing 9 cross-2a/2b interfaces (plan §8 frozen list)
1. **ExecutionStateEnvelope** — `{schema_version: u8, payload_cid: Cid, payload: ExecutionStatePayload}`. Frozen at 2a close.
2. **ExecutionStatePayload** — `attribution_chain: Vec<AttributionFrame>` + `pinned_subgraph_cids: Vec<Cid>` + `context_binding_snapshots: Vec<(String, Cid, Vec<u8>)>` + `resumption_principal_cid: Cid` + `frame_stack: Vec<Frame>` + `frame_index: usize`. Chain-not-3-tuple per ucca-4. Frozen at 2a close.
3. **HostError** — `{code: ErrorCode, source: Box<dyn StdError + Send + Sync>, context: Option<String>}` (Option A) plus 5 reserved ErrorCode variants. Frozen at 2a close.
4. **PrimitiveHost trait signatures** — no `benten_graph::*` types appear in trait method signatures or `EvalError` variants; all content-returning methods carry `check_read_capability` threading per Option C. CI gate: `arch_1_no_graph_types_in_primitive_host.rs`.
5. **TraceStep schema + attribution** — `attribution: AttributionFrame` required on every variant; `Step` + `SuspendBoundary` + `ResumeBoundary` + `BudgetExhausted` variants present.
6. **TimeSource + MonotonicSource traits** — dual-source per §9.13. MonotonicSource for refresh cadence (drift-exploit-hard); HLC consulted for federation correlation.
7. **Subgraph `deterministic` field** wired through DAG-CBOR serialization.
8. **WriteAuthority enum** — `User | EnginePrivileged | SyncReplica { origin_peer }` per ucca-9 / arch-r1-2.
9. **`SYSTEM_ZONE_PREFIXES` const** — `&[&str]` in `benten-engine/src/system_zones.rs`, PascalCase prefixes matching HEAD per §9.10.

Each interface has a regression test in G11-A's ownership; reopening any of them in Phase-2b escalates to a full 2a re-open.

### Other Phase-2a-locked commitments (in plan §9)

**§9.10 Inv-11 placement.** Runtime enforcement in `benten-engine/src/primitive_host.rs`; registration-time literal-CID reject in `benten-eval/src/invariants/system_zone.rs`; storage-layer stopgap in `benten-graph` retained as defence-in-depth. Shared const in new `benten-engine/src/system_zones.rs`. PascalCase prefixes — case-sensitive per §9.10. CI guard: workspace test greps `crates/**/*.rs` for `"system:<literal>"` and asserts each in `SYSTEM_ZONE_PREFIXES`.

**§9.11 Inv-13 5-row firing matrix.**
1. User × match → `E_INV_IMMUTABILITY`.
2. User × differs → `E_INV_IMMUTABILITY` (vacuous under content-addressing but kept for error-path naming).
3. EnginePrivileged × match → `Ok(cid_dedup)` no ChangeEvent + no audit-sequence advance.
4. SyncReplica × match → `Ok(cid_dedup)` same shape (Phase-3 reserved).
5. WAIT-resume stale-pin → `E_RESUME_SUBGRAPH_DRIFT` BEFORE any write attempt.

**§9.12 No Budget trait in 2a.** Inv-8 multiplicative cumulative budget stays independent. Cross-compatibility with Phase-2b SANDBOX fuel via shared `TraceStep::BudgetExhausted` shape only. Phase-2b R1 decides whether to extract a trait; Phase 2a just agrees on trace shape so 2b has forward-compat.

**§9.13 TOCTOU refresh points.** Compromise #1 expanded from 3 → 5: (1) transaction commit, (2) CALL entry, (3) `iterate_batch_boundary`, (4) WAIT resume (NEW), (5) 300s wall-clock during ITERATE (NEW). Dual-source: MonotonicSource for cadence + HLC for federation correlation.

**§9.14 arch-1 dep-break signature-level CI gate.** Workspace test greps `PrimitiveHost` impl + `EvalError` variants for any `benten_graph::*` reference. Companion test asserts `benten-core` does not depend on `benten-eval`. Both run on every Phase-2+ PR via `arch-1-dep-break.yml`.

**§3.1 CI hardening pass = useful-now CI items (CodeQL + branch-protection-as-code + SHA-pin) ship before Phase 2a internal tag.** **§3.2 Publish-readiness pass** (cargo-semver-checks + napi prebuilt + release-plz + SLSA + SBOM) deferred to Phase 8/9+ since OSS publication is not planned before then. Settled at D19 close-out per Ben rescope.

**Phase 2a internal tag = handoff continuity, not external publish.** The tag exists for `git log v2a..` as Phase-2b scope query; no external artifacts produced. Phase 8/9+ landing of release-plz + napi prebuilt + the rest is the actual ship event.

---

End of Phase 2a synthesis draft.
