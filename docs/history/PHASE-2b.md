# Phase 2b — SANDBOX + WASM + Compute

**Status:** SHIPPED at tag `phase-2b-close` (`3d0f018`, 2026-05-03).
**Closed at:** 99 commits in the `phase-2a-close..phase-2b-close` window.
**Convergence:** R6 quality council 6 rounds — R1=64 → R2=15 → R3=48 → R4=15 → R5=2 → R6=0 (16/16 lenses CONVERGED at FINAL gate).
**Canonical fixture CID:** `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` (stable across all platforms / MSRV / wasm32 targets).

---

## § 1. Narrative journal

Phase 2b opened on the back of the cleanly-closed Phase 2a tag (`phase-2a-close`, `cb49554`, 2026-04-25). The plan was, on paper, ambitious but coherent: ship the three remaining executable primitives (SANDBOX + STREAM + SUBSCRIBE), close out the §7a items absorbed from Phase 2a (G12-A through G12-F), generalize IVM to Algorithm B, prove the wasm32 cross-target story end-to-end. The plan came in around 652 lines with 13 sequenced groups, 16 D-points open, an explicit branch-per-agent CI-as-verifier procedure pinned in a sister `dispatch-conventions.md` doc, and the canonical fixture CID carried forward as the determinism gate.

What the original plan did not anticipate was the eventual 13-sub-track shape of wave-8 — a wave that did not exist on the original sequencing chart and that wound up consuming roughly half of all Phase 2b operational effort.

### Pre-implementation review pipeline (pre-R1 → R1 → R2 → R3 → R4 → R4b)

The pre-R1 critic round was the first compression event. Five lenses ran in parallel against the plan: architect-reviewer, methodology-critic, security-auditor, wasmtime-sandbox-auditor, and a security-deliverables-companion authoring task. The architect-reviewer surfaced one CRITICAL contradiction immediately — the §2 sequencing diagram, the §9.5 Q3-RESOLVED note, and the §3.2 G12-C/D gating claims declared THREE different group orderings for G12 vs G6/G7/G8/G10. Without a single coherent ordering, R1 critics would have fragmented their reviews across the three readings. The methodology-critic surfaced two more CRITICALS: the required-check workflows had `branches: [main]` triggers, NOT `branches: ['**']`, so an agent push to `phase-2b/<group>/<agent-slug>` would trigger ZERO CI runs (the §6 procedure was structurally broken on inspection); and branch protection on `main` made orchestrator-direct merge impossible without `gh pr merge --rebase`. The security-auditor surfaced the load-bearing module-signing gap: D16 was a deferred Phase-3 Ed25519 question, but with no threat model and no minimum-CID-pin landing, the install-module API would ship as a public-write surface with NO integrity check. The wasmtime-sandbox-auditor surfaced 5 ERROR-CATALOG drift items (E_INV_SANDBOX_NESTED already existed, the plan reserved E_INV_SANDBOX_DEPTH as a duplicate; same for OUTPUT_LIMIT and TIMEOUT) — duplicates that would have violated the catalog discipline.

The pre-R1 close tabulated 58 cross-lens findings: 30 closed-in-plan, 17 closed-in-sister-doc, 6 converted to new D-points (D17 through D22), 5 RESOLVED in-place, 0 deferred without target, 7 NOT-ADDRESSED at close (editorial-only gaps). The plan went into R1 with a coherent sequencing diagram, a working PR-flow procedure, a minimum-viable CID-pin shape for module install, and 6 new D-points teed up for R1 decision.

R1 dispatched 8 lenses. The cross-lens convergence was striking: security-auditor and wasmtime-sandbox-auditor independently arrived at "hybrid: per-cap declared in host-functions.toml; `kv:read` per-call, `time/log` per-boundary" for D7 cap-recheck cadence; D17 Inv-7 architecture got Option C (defense-in-depth: streaming `CountedSink` PRIMARY + return-value backstop) from both sec + wsa. D9 module manifest format converged on canonical DAG-CBOR from security-auditor's parser-divergence-as-attack-surface framing. The benten-philosophy lens disagreed with architect-reviewer on D14 (architect recommended loud-fail TraceStep; philosophy recommended typed `UnknownTraceStep` variant for forward-compat) — and that disagreement got surfaced and resolved in favor of philosophy's view at orchestrator triage. The wasm-target lens caught a CRITICAL framing collapse: the plan conflated wasm32-wasip1 (Node-WASI hosted) with wasm32-unknown-unknown (browser native) — different KV/sandbox/async semantics; recommended dual-target CI workflows + dual canonical-CID test.

R2 was a single-agent test-landscape synthesis — a 626-line specification of ~276 new test artifacts across ~108 new test files, partitioned 5 ways across R3 writers (A streaming, B sandbox, C security, D perf+ivm, E TS+integration). It locked the 16 ESC vectors as named adversarial fixtures (ESC-1 through ESC-16) with per-vector `.wat` fixture paths, expected error-codes, and test names. Five new D-points surfaced during R2 (D23 through D27).

R3 was where the plan met first contact with reality. The branch-per-agent CI-as-verifier flow's first live use was the R3 consolidation push itself — predating the planned G12-A procedure dry-run. 6 R3 writers landed ~94 red-phase test files. The single P0 lesson was the **compile-vs-execute gating gap**: `#[ignore]` is sufficient for execution gating but NOT for binary-compilation gating when files contain `use benten_*::unlanded_module::Type` statements. Resolution: a new cargo feature `phase_2b_landed = []` added to 4 crates, default OFF; 28 top-level test/bench files prepended `#![cfg(feature = "phase_2b_landed")]`; 17 integration submodules cfg-gated. The mechanism became the Phase-2b standing pattern for red-phase test gating.

The R3 testing-helpers pinning doc was supposed to prevent the Phase-2a stub-then-test-then-impl-then-fix-test churn — but on day 1, R4-FP-A bulk-audited every callsite via `grep -rh 'testing_[a-z_]*'` and discovered ≥14 helper signatures had drifted from R3-committed test bodies. R4-FP-A rewrote the rows to match the call sites — **tests-as-spec is the R3 invariant.** Many entries got "**CHANGED**" markers.

R4 ran 5 lenses on the R3 red-phase surface. The qa-expert verdict was "NEEDS FIX-PASS BEFORE R5" — 4 CRITICAL groups had ZERO red-phase test coverage. The rust-test-reviewer lens caught the load-bearing TDD-fidelity bug: Algorithm-B equivalence tests projected via `format!("{:?}", v.id())` — TAUTOLOGICAL because both views share id; whole class of bugs invisible. The same lens caught `prop_assume!(false)` red-phase placeholders that DISCARD cases instead of failing — silent vacuous-pass after un-ignore. The R4-FP wave that followed was the orchestrator-dispatched response to qa-r4-* + rust-test-coverage findings: 4-5 surface-grouped writers in parallel filling the missing red-phase tests for G8-B, G12-B, G12-C, G12-D, G10-B, WAIT TTL D12, and the missing CI workflow tests. By R5 dispatch, the test landscape was honest.

### R5 implementation arc

The R5 implementation kicked off into the planned 7-wave structure. The first defining moment came in **wave-2 (G12-C precursor)**, where the agent dispatched to migrate `Subgraph` ownership reported back with a discovery that reframed the whole crate-boundary effort: Rust's no-inherent-impls-on-foreign-types rule meant that moving `Subgraph` to its target home required moving *with it* roughly 2,000 LOC of invariants AND refactoring 88+ `build_validated` callsites. The agent narrowed scope (134/82 LOC actually touched vs the 150-300 LOC estimate), surfaced three significant deviations, and the orchestrator dispatched G12-C-cont as a separate larger wave. That single dispatch forced D28 — the canonical-bytes shape — to RESOLVE in real time when the fix-pass A.1 work demonstrated that eval-side production CanonView (`{handler_id, sorted nodes, sorted edges, deterministic}`) was the authoritative shape, superseding the original three-field proposal.

By **wave-2-cont close (PR #20 → main `a2b33b7`)**, an early texture of the phase had set in: agent-reported scope estimates were systematically optimistic, mini-reviews were the load-bearing verification layer, and Ben's HARD RULE on "no later" disposition had begun to bite. The HANDOFF-2026-04-27-evening doc carries the first formal codification of the three valid non-fix-now dispositions (OUT-OF-SCOPE / BELONGS-NAMED-NOW / DISAGREE-WITH-EXPLANATION) — born out of G12-C-cont's 4 mini-reviewers returning merge-as-is with 12 deferred items that Ben caught and called back. CLAUDE.md §12 was re-strengthened that day with no-time-qualifier framing.

**Wave-4** (parallel G6-B + G7-A + G7-B + G7-C + G8-A + G8-B) brought the first real architectural surface tension. PR #30 (G7-A) immediately blocked on a Ben-policy decision when cargo-deny + cargo-audit (newly added to the required-check set) surfaced 11 wasmtime transitive RUSTSEC vulnerabilities. Ben pre-resolved four decisions before bed (option D supply-chain hybrid, G7-A 35-finding triage merge-as-is, G8-A bench-gate hybrid threshold per-view, Wallclock REJECT not CLAMP), and the orchestrator entered the first real autonomous run to absorb the Rust 1.95 lint cascade across 4 PRs (`Duration::from_secs` lints — three cascade rounds, eventually file-level `allow` on six test/bench files). A real bug was found in G7-B's Inv-4 walker: `sandbox_depth` DFS lacked back-edge cycle detection, causing 3 tests to time out at 154-240s on every runner (D-NS-17). The cross-crate field-cascade pattern was named that day after both G7-B (`sandbox_depth` field on AttributionFrame) and G8-A (`strategy` field on ViewDefinition) hit it.

**Wave-5 brought a subtle architectural pivot (D-NS-37):** the in-memory backend agent discovered that Engine is hard-bound to `Arc<RedbBackend>` (concrete, not trait-object). Rather than push through a workspace-wide `Arc<dyn KVBackend>` refactor mid-phase, the orchestrator routed `Engine::open(":memory:")` through redb's OWN in-memory page store; same pivot for G10-A-wasip1 `from_snapshot_blob`. The trait-object refactor was deferred to Phase 3 with explicit named destination.

**Wave-6** (G12-B + G12-E parallel) closed Compromises #9 (Dedup-pure-read) and #10 (Resume-cap-recheck). **Wave-7** (G11-2b solo) ran the canonical-fixture cohort against the SANDBOX paper-prototype and measured 16.7% SANDBOX rate — well under the 30% phase-2b exit-criterion gate. The structural R5 implementation closed at PR #44 → main `8169807`.

But "structural completion" was not "shipped." **R4b dispatched, and the load-bearing finding came back on 2026-04-29 morning:** Phase 2b R5 was structurally complete, BUT production runtime dispatch was unwired for SANDBOX, STREAM, and runtime SUBSCRIBE delivery. Verified directly on main: `crates/benten-eval/src/primitives/mod.rs:91` returning `PrimitiveNotImplemented` for `Wait|Sandbox`; `sandbox.rs:319-448` had steps 1-5 of the wasmtime invocation pipeline but step 6 (Store + Linker + Instance + fuel/memory/wallclock + epoch ticker) explicitly NOT WIRED. STREAM/SUBSCRIBE wrappers returning `PrimitiveNotImplemented`; TS `engine.onChange` synthesizing a `queueMicrotask` sentinel that ACTIVELY MASKED the no-op. The failure mode was at least fail-loud (typed `E_PRIMITIVE_NOT_IMPLEMENTED`), so no security escape — but the contract was a lie. The R4b code-reviewer lens corrected several false-absent claims that earlier lenses had made: `AttributionFrame.sandbox_depth` field DID exist; `ModuleManifest` exists; `SnapshotBlob` exists. The remaining concern was correctly framed: the field/types exist, but production paths don't INVOKE them. The documentation-engineer R4b v1 framed the docs-vs-code drift as 4 doc files MISSING; v2 (post-G11-2b doc sweep) framed it as files-now-PRESENT-but-LYING-about-runtime-status.

### Wave-8: the runtime-wire-through wave

**Wave-8 became the runtime-wire-through wave**, and it was where Phase 2b earned its retrospective shape. The original brief had 7 sub-tracks (8a-8g); by close, it had grown to 13 (8a / 8b / 8c / 8c-cont / 8c-stream-infra / 8c-subscribe-infra / 8d-types / 8d-narrative / 8e / 8f / 8g / 8h / 8i-wait + 8j-ci-cleanup). The expansion came from two rounds of audit-first dispatch where each round surfaced gaps the brief hadn't anticipated.

Wave-8 Round 2 (8b solo, the long pole) shipped the SANDBOX wasmtime invocation pipeline — Store + Linker + Instance + ResourceLimiter + epoch ticker (D24) + trap-to-typed mapping with D21 priority resolver MEMORY > WALLCLOCK > FUEL > OUTPUT + D17 BACKSTOP at primitive boundary + 9 of 16 ESC defenses. The mini-review caught the wave's first BLOCKER and named one of its load-bearing antipatterns: **wsa-w8b-1** — the agent's "DISAGREE-WITH-EXPLANATION" rerouting through `PrimitiveHost::execute_sandbox` was *interface-sound* but *incomplete-as-landed*. The engine override was missing → trait default returned `PrimitiveNotImplemented` → the production engine path was STILL unwired, the same R4b BLOCKER relocated one layer up. Compromise #4 was falsely claimed CLOSED in the PR description. The pattern got named on the spot: **"DISAGREE-WITH-EXPLANATION used as deferral hat."** The fix-pass closed it with a positive must-invoke acceptance test (`sandbox_execute_via_engine_dispatch_invokes_executor`).

Wave-8 Round 3 (8c parallel with 8f) introduced the contrast between the two non-deferral patterns. The 8c initial agent surfaced realistic scope mismatch UPFRONT and named a clean named-destination doc (`wave-8c-cont-brief.md`) listing every deferred item with original scope intact. **This was the principled-honesty pattern** — the mini-review returned `READY-TO-MERGE` with 4 OBSERVATION findings and validated that the named destination was real. By contrast, 8f (G12-B hot-replace) delivered FULL scope with ZERO CI fix-passes but the mini-review caught a HIGH lock-ordering defect in `register_subgraph_replace`: the handlers Mutex was released BEFORE the version_chain Mutex was acquired, inverting chain ordering under concurrent replaces. **Compromise #18 was added** (handler_version_chain in-memory only — sibling to #17 module_bytes registry).

Wave-8 Round 3.5 was where the wave grew from 7 sub-tracks to 13. The 8c-cont return surfaced MORE deferrals, and Ben dispatched an audit-first option D agent that discovered three NEW gaps beyond STREAM/SUBSCRIBE: SANDBOX Named-manifest registry never consulted at dispatch (contradicting the wave-8b "Compromise #4 closed" claim), EMIT engine wrapper a no-op, IVM Algorithm B never registered in production. The 8c-subscribe-infra mini-review caught **the wave's second BLOCKER** and the most surgical bug of Phase 2b: napi `on_change`/`on_change_as` was dropping `Subscription` at end of method scope (Drop fired unsubscribe → registry slot empty before JS even saw the response); combined with TS-side `makeSubscription` missing `onUnsubscribe` wiring, **the public API `engine.onChange()` was a complete lie — callbacks never fired.**

Round 4-5 closed the wave. 8i-wait + 2 fix-passes closed the WAIT production runtime gap (3 fix-passes for principal-binding, `elapsed_ms`, and `Engine::resume_with_meta` deadline gap). 8d-narrative landed final docs reconciliation. 8j-ci-cleanup closed 5 advisory CI failures. **Wave-8 closed at main `e2b1c62` (PR #58) + `1e89157` (PR #59).** A four-instance metadata-producer-vs-consumer drift pattern was named in passing: wave-8b SANDBOX, wave-8c-subscribe-infra, 8i-wait fp1, 8i-wait fp2. By Phase 2b close that count would reach 24 cumulative instances.

The wave-8 night-shift was also where the most foundational process memories crystallized: `feedback_plain_english_surfaces` (Ben's "plainer English with predictions" feedback codified as plain-situation → concrete-options → my-pred-of-Ben's-call → confirm-or-redirect); `feedback_subtrack_sizing_heuristic` (~400-800 LOC sweet-spot); `feedback_no_defer_HARD_RULE` updated with "verify destination EXISTS before naming it" sub-section after a Phase-2c-not-in-roadmap mistake (an invented destination, ungrounded). Ben framed the v1 milestone gate that night: Phases 1+2a+2b+3 minimum + post-Phase-3 PAUSE-AND-ASSESS = the v1-shippable gate; saved as `feedback_v1_milestone_gate` and CLAUDE.md item #15.

### R6 quality council (6 rounds)

The R6 quality council opened on 2026-04-29 against `e2b1c62`, the wave-8 close-out HEAD that had finally wired the SANDBOX/STREAM/SUBSCRIBE/WAIT production runtime end-to-end. R5 wave-8 had landed the load-bearing plumbing — the eval-side sandbox::execute body, the engine override at primitive_host.rs:718-907, the engine_stream.rs producer-bridge wire-through, the napi SubscriptionJs class with ThreadsafeFunction trampoline, the WAIT envelope/suspension-store dual-key contract.

**Round 1** dispatched 12 lenses in parallel. The tally: **1 BLOCKER + 1 CRITICAL + 16 MAJOR + 46 minor/nit = 64 findings**. The BLOCKER was r6-mpc-1 (`Engine::resume_with_meta` consults only `timeout_ms` + `suspend_elapsed_ms` then returns terminal_ok_outcome — the `meta.signal_shape` validation + `meta.is_duration` deadline branch were skipped). The CRITICAL was r6-dx-1 (QUICKSTART.md `handler.id` claim wrong). The 3-lens convergence pattern surfaced: Inv-4 sandbox_depth runtime arm dormant (cr-1 + mpc-4 + wsa-1); describe_sandbox_node synthetic defaults (mpc-3 + napi-3 + dx-10); ESC matrix mislabel + double-count (doc-1 + sec-1 + wsa-9).

Per Ben's 3+-recurrence directive (memorialized as `feedback_3_plus_recurrence_deep_sweep.md`: "3 instances → broader investigation"), three deep retrospective sweeps dispatched in parallel: **cite-precision** (~70 cites audited, 7 newly-discovered drifts beyond 6 known); **stale-`#[ignore]`-with-landed-deferral-targets** (the headline of the night — 11 known instances exploded to ~93 cases needing disposition); and **engine-vs-eval claimed-symmetry asymmetry** (4 NEW findings beyond Round 1's 4 — including NEW-1 BLOCKER `Engine::read_view*` healthy-view path returns `Vec::new()` despite docstring claiming view's current state). A 4th deep sweep landed independently: the **producer/consumer drift redux** found 7 NEW instances on top of the 8 already named, including Instance 6 BLOCKER (graph::ChangeEvent → eval::ChangeEvent bridge drops 6 of 9 fields → silent multi-label SUBSCRIBE delivery loss).

Three BLOCKERs total going into the fix-pass: r6-mpc-1, Instance 6, NEW-1. PR #62 + PR #60 + PR #61 + PR #63 + PR #64 + PR #65 (R6FP-tail-comprehensive) all landed in this cycle.

**Round 2** dispatched at HEAD `fa001fc` (post the 6-PR fix-pass merge). The lens reduction to 5 was a deliberate cost-saving choice — only the lenses with R1 findings, plus the security perimeter as a safety net. **This is where pim-3 was born.** The R2 returns showed code-reviewer cleanly converged (0 findings) — looked like the council was on track for fast convergence. But doc-engineer surfaced 5 findings (2 MAJOR + 1 MEDIUM + 1 LOW + 1 nit), security-auditor surfaced 5 (2 MAJOR + 3 minor) including the still-OPEN r6-sec-4 that had been mis-disposed at R1. The metadata-producer-vs-consumer lens found the recurrence rate had dramatically slowed — but the still-open r6-sec-4 + several stale-doc-claims after PR #62 made clear the lens-reduction had MISSED correctness coverage. The R2 doc-engineer flagged that PR #62 wired the Inv-4 sandbox_depth runtime threading WITHOUT coupled doc updates → 5 cross-doc surfaces still describing the runtime arm as dormant. This was the **post-fix doc-coupling pre-flight failure** — the pattern that became pim-1 §3.5b HARDENED.

Ben's call was unambiguous: lens-reduction is wrong for phase-close convergence. **Round 3** EXPANDED back to a full council — 14 redux lenses + 3 narrow-iteration variants (cag, closure-audit, mpc) + 3 fp-mr (Group A/B/C) + 1 pattern-induction meta-sweep = 21 dispatches. Round-3 finding total = **48** across the redux JSONs — almost as many as R1. Highlights: r6-r3-arch-1 MAJOR (PR #68 added is_read_only_snapshot enforcement at PrimitiveHost::put_node but delete-path asymmetry remained); r6-r3-doc-2 MAJOR (PR #68 added 25 lines without coupled doc updates — pim-1 recurring); r6-r3-dx-1+2 CRITICAL (scaffolder template README + package.json carrying the EXACT bugs PR #60 fixed in main workspace); r6-r3-ivm-1 BLOCKER (r6-ivm-3 closed at TS-DSL only — Rust engine + napi accept canonical-id+mismatched-label silently); r6-r3-ivm-3 MINOR (phase-3-backlog.md §5.1 names FIVE canonical views but 3 of 5 are HALLUCINATED — Shape-2 named-destination-realness violation INSIDE the destination doc itself). The narrow-iteration-closure-audit (429 lines, 0 findings) at HEAD `d25dee1` confirmed all 48 R6-R3 fix-now findings VERIFIED CLOSED across the 3-PR fix-pass chain (#71 Group A + #69 Group B + #70 Group C) plus the wasmtime bump (#72) and the orchestrator-direct Inv-11 fix.

**Round 4** was the verification round at `d25dee1` post the R6-R3-FP triad. **Convergence began visibly.** R4 returned 15 findings — about 1/3 of R3's count. Determinism-verifier 8 ALL POSITIVE; ivm-correctness CONVERGENT 0 findings; metadata-producer-vs-consumer CONVERGED 0 findings; napi-bindings CONVERGENT 0 findings.

**Round 5** at HEAD `a73aeee` post 6-PR session merge sequence (#69/#70/#71/#72/#74/#75) was the **convergence stretch**. Just **2 fix-now findings** total. The 21st producer/consumer drift instance (SubscribeArgs.handler removal) landed in PR #75; close-out cite-precision repairs (856b74c + a12c1d6 + 6d2f1d3 + bf8a30a) handled the 22nd+23rd+24th instances and a phantom-symbol cite. The R5 pattern-induction meta-sweep was **the most exhaustive meta-sweep of phase-2b** — walked all R1+R2+R3+R4 lens reports + 4 R4-narrow-iteration agents + R6-R4-FP mini-review + dispatch-conventions §3.5b/§3.6b ratifications + D-NS-1..230 night-shift log entries. Found no new emerging patterns; ratified the existing 11 pim-N codifications.

**Round 6** — the FINAL FULL CONVERGENCE COUNCIL at HEAD `3d0f018` post PR #76 (R6-R5-FP) — restored the full Round-1 lens set per `feedback_phase_close_final_council_full` (the codified lesson from R2's lens-reduction misfire). 16 lenses dispatched. **Returns: 16/16 CONVERGED. Zero findings.** The 24 cumulative producer/consumer drift instances all dispositioned (21 closed end-to-end + Instances 22+23 closed by PR #76 + Instance 24 BELONGS-NAMED-NOW into phase-3-backlog §6.6 acceptance criterion). The 11 pim-N ratifications all verified intact. Phase 2b shipped at tag `phase-2b-close` on 2026-05-03.

The post-tag morning review caught one residual recalibration: pcds-2 was re-disposed MAJOR → BLOCKER (severity-tally only; fix already in PR #76); pim-3/4/6/7/11 were pulled in from Phase-3-pre-R1 carry list and codified inline.

### Closing texture

The closing texture was paradoxical: every R6 round ratified a different lesson about how to do councils. R1 taught that 3+ recurrence triggers deep sweeps. R2 taught that lens-reduction at phase-close is wrong. R3 taught that the full-council expansion catches what lens-reduction misses. R4 taught that consumer-audit-dimension is a load-bearing dispatch-convention. R5 taught that scaffolder templates drift from main workspace silently. R6 taught that the FINAL gate must be the full Round-1 council on the actually-shipping HEAD with zero new findings. The 6-round arc is now the canonical Phase-N close shape — Phase-3 R6 will follow the same sequence per the active CLAUDE.md night-shift authorization.

Wave-8's expansion 7→13 sub-tracks was the load-bearing operational lesson. The R4b finding could have been a phase-restart trauma; instead the wave grew in disciplined sub-tracks with named-destination deferrals + per-group sync mini-reviews catching the BLOCKERs the agents themselves had not surfaced. By the end, the project had codified eleven pim-N process patterns inline in dispatch-conventions (only pim-6's CI-infra half remained as a Phase-3 residual), and 24 producer/consumer drift instances had all been dispositioned. The discipline carried.

---

## § 2. Changelog

### Engine surface — production runtime LIVE for all 12 primitives at phase close

- **SANDBOX wasmtime invocation pipeline (8b):** Store + Linker + Instance + ResourceLimiter + epoch ticker (D24) + per-call lifecycle + trap-to-typed mapping with D21 priority resolver (MEMORY > WALLCLOCK > FUEL > OUTPUT) + D17 BACKSTOP CountedSink at primitive boundary + 9 of 16 ESC defenses passing
- **STREAM production runtime (8c-stream-infra):** `ChunkProducer` trait + `spawn_chunk_producer` + `StreamHandle::from_producer_bridge` + `Engine::call_stream_inner` wire-through replacing PrimitiveNotImplemented stub
- **SUBSCRIBE production runtime (8c-subscribe-infra):** napi `SubscriptionJs` class; widened `ChangeEvent` to carry multi-label payloads; runtime delivery via `Engine::on_change` / `on_change_as`
- **WAIT production runtime (8i-wait + 2 fix-passes):** `Engine::resume_with_meta` 3 metadata branches; `WaitMetadata.timeout_ms` + `suspend_elapsed_ms` consumed; `E_WAIT_TIMEOUT` typed firing; `PrimitiveHost::suspending_principal` trait method
- **IVM Algorithm B engine surface (G8-A + G8-B):** `Strategy` enum + `AlgorithmBView` wrapper at engine boundary; user-registered IVM views via `Engine::register_user_view` + napi + DSL
- **G12-B hot-replace (8f):** `register_subgraph_replace` with joint-lock concurrency safety + version-chain ordering preservation; in-flight call observes pre-swap subgraph contract
- **G12-D type widening:** `SubgraphSpec.primitives` widened; walker rewrite; non-vacuous Mermaid + trace pin
- **Subgraph relocation (G12-C-cont):** full migration + canonical-bytes-shape D28 RESOLVED; `Subgraph` Auto-derive serde dropped (cag1 follow-up: 14-test runtime trait-impl probe + canonical round-trip property pin)
- **Manifest registry + counted sink (G7-A):** cap-derived host-fn manifest; per-fn cap_recheck cadence; per-call wasmtime lifecycle; CountedSink with distinct OverflowPath tags; D9 canonical DAG-CBOR for manifests; ESC-15 closure complete
- **Inv-4 + Inv-7 enforcement (G7-B):** durable `sandbox_depth` cycle detection inside walker; AttributionFrame extension preserves Phase-2a fixture CID via non-zero-only canonical-bytes discipline

### Compromises

**Closed during Phase 2b:**
- **Compromise #4** SANDBOX named-allowlist — closed via must-invoke positive assertion at engine dispatch path
- **Compromise #9** Dedup-pure-read — closed via G12-E generalized SuspensionStore
- **Compromise #10** Resume-cap-recheck — closed via G12-E + 8i-wait integration (cross-process metadata arm; engine-side asymmetry arm closed at Phase-3 G14-D)

**Landed (accepted) during Phase 2b:**
- **Compromise #14** SANDBOX cold-start cost — accepted Phase-2b additive (D3 RESOLVED — Phase-3 additive change if real-workload bottleneck)
- **Compromise #15** `register_runtime` reserved with deferred error — Phase-8 marketplace closure target
- **Compromise #16** `random` host-fn deferred — Phase-3 closure target (subsequently closed at G17-A2)
- **Compromise #17** `module_bytes` registry in-memory only, single-process scope
- **Compromise #18** `handler_version_chain` in-memory only (sibling to #17)
- **Compromise #19** Browser-target persistent storage absent — manifests in-memory only on `wasm32-unknown-unknown`
- **Compromise #20** Cross-browser determinism CI cadence not yet established
- **Compromise #21** Module manifest minimal CID-pin in Phase 2b; full Ed25519 deferred — Phase-3 closure target

### ErrorCode additions

- `E_INV_SANDBOX_DEPTH`, `E_INV_SANDBOX_OUTPUT` (G7-B)
- `E_SANDBOX_MODULE_NOT_INSTALLED`, `EvalError::Sandbox` variant (8d-types)
- `E_SANDBOX_UNAVAILABLE_ON_WASM` (8c initial)
- `E_PRIMITIVE_NOT_IMPLEMENTED` retained as fail-loud at unwired surfaces; eliminated for SANDBOX/STREAM/SUBSCRIBE/WAIT at phase close

### Tooling / CI

- New composite GitHub Action `.github/actions/free-disk-space/action.yml` (removes ~25-30GB preinstalled tooling: dotnet, android NDK, GHC, CodeQL, Python, Java); applied to `coverage.yml` + `phase-2a-exit-criteria.yml` + `ci.yml` Linux legs
- MSRV bumped 1.91 → 1.95 across workspace; `Duration::from_secs` lint cascade closed by allow-directive-drop
- napi vitest CI wiring (D6 systemic-finding closure from G12-A wave)
- cargo-deny + cargo-audit added to required-check set; 11 wasmtime transitive RUSTSEC vulnerabilities triaged via Ben option-D hybrid (advisory web-search + per-CVE disposition); wasmtime 43.0.1 → 43.0.2 bump for RUSTSEC-2026-0114
- `cargo +stable doc --workspace --no-deps` standing pre-flight added (pim-7)
- One-time `cargo check --workspace --all-targets` exception when adding new public struct fields (cross-crate field-cascade pattern)
- Drift-detect.ts CI gate kept clean across phase
- `phase_2b_landed = []` cargo feature mechanism added to 4 crates (default OFF; 28 top-level test/bench files cfg-gated; 17 integration submodules wrapped; retired in pre-R4b PR #144)

### Docs

- `docs/MODULE-MANIFEST.md` (Phase-2b SANDBOX surface)
- `docs/SANDBOX-LIMITS.md` (Phase-2b production-runtime LIVE)
- `docs/SECURITY-POSTURE.md` retense for Compromises #4/#9/#10/#14/#15/#16/#17/#18/#19/#20/#21
- `docs/ERROR-CATALOG.md` retense for new ErrorCode variants
- `docs/future/phase-3-backlog.md` — created mid-R6-R5-FP per HARD RULE rule-12 closure (D-NS-250); ~10 sections including §6.6 SANDBOX casing-drift acceptance criterion + §7.9 TS-surface-parity sweep + §7.11 process-pattern-ratifications + CI-infrastructure carry

**Phase 2b shipped at tag `phase-2b-close` (`3d0f018`, 2026-05-03).** 99 commits in the `phase-2a-close..phase-2b-close` window.

---

## § 3. Key takeaways — what to remember

**What this phase was fundamentally about.** Phase 2b was scoped on paper as "land SANDBOX + WASM + Compute primitives." In practice it was about *closing the gap between structurally-implemented and production-runtime-LIVE*. The R4b finding split the phase cleanly in two: pre-R4b waves (G6-G12 + paper-prototype) landed structures, types, registration-time invariants, manifest data, and documentation surfaces — but eval-side executors and engine wrappers shipped as scaffolding returning `PrimitiveNotImplemented` / no-op / `Subscription { active: false }`. Wave-8 was the runtime-wire-through. Roughly half of all Phase 2b operational effort lived in wave-8 + the R6 council that followed it.

**The hardest problems we hit (and how we resolved them):**

- **The R4b finding itself.** Three full primitive surfaces (SANDBOX/STREAM/SUBSCRIBE/WAIT) shipped structurally but not runtime. Resolution: wave-8 expanded from 7 sub-tracks to 13 over the course of the wave, with audit-first dispatches between rounds catching new gaps the brief had missed.
- **The "DISAGREE-WITH-EXPLANATION used as deferral hat" antipattern (wsa-w8b-1).** An agent landed half a fix while *appearing* to ship the full scope. Resolution: must-invoke positive assertions in mini-review acceptance tests (not just sentinel-presence pins); pim-2 §3.6b end-to-end test pin requirement codified.
- **The `engine.onChange()` is-a-lie BLOCKER (8c-subscribe-infra).** Subscription dropped at napi method scope end → callbacks never fired. Caught by sync mini-review, not CI. Resolution: `SubscriptionJs` napi class replacing JSON-return path.
- **Lens-reduction at R2 misfire.** R2's 5-lens-cost-saving choice missed correctness coverage; r6-sec-4 stayed open, post-PR#62 doc-coupling failures went undetected. Resolution: R3 expanded back to full council with 14 redux + 3 narrow + 3 fp-mr; subsequent rounds preserved full-council shape. Memorialized as `feedback_phase_close_final_council_full`.
- **The 3+ recurrence pattern threshold.** R1 surfaced known-pattern instances 4-7 deep before triggering the deep sweep. The stale-`#[ignore]` deep sweep then found ~85 cases beyond the 11 known — the iceberg was 8x larger than Round 1's tip. Resolution: Ben ratified 3+ recurrence as the threshold (`feedback_3_plus_recurrence_deep_sweep.md`).
- **Producer/consumer drift cascade.** 24 cumulative instances by phase close; recurring across SANDBOX (fuel/output dropped at primitive_host.rs), STREAM (chunk seq dropped, requires_explicit_close not enforced JS-side), SUBSCRIBE (multi-label loss at builder.rs bridge), WAIT (resume metadata fields skipped), EMIT (no napi/TS adapter pre-PR-#66), typed errors (RegistrationError 14 structured fields all reduced to message string), DSL diagnostics (line/column Debug-formatted), register_subgraph_replace outcome shape. Resolution: consumer-audit-dimension ratified as a standing dispatch-convention (§3.6 — every public-shape change MUST enumerate the 4 canonical consumer surfaces by name + state).
- **The wasmtime transitive RUSTSEC vulnerabilities.** 11 advisories surfaced when cargo-deny + cargo-audit landed in required-check. Resolution: option-D hybrid with web-search authorization for advisory-by-advisory triage; wasmtime 43.0.1 → 43.0.2 bump for RUSTSEC-2026-0114; remaining advisories ignore'd with documented rationale.
- **The Inv-4 walker timeout (D-NS-17).** `sandbox_depth` DFS lacked back-edge cycle detection, causing 154-240s timeouts on every runner. Resolution: gated Inv-4 call site on `!violations.contains(Cycle)`, then durable fix inside walker as part of wave-8e.

**What surprised us:**

- **Wave-8 growing 7 → 13 sub-tracks.** The audit-first option-D pattern (running an investigatory agent BEFORE dispatching implementers) was originally a side-detour; it turned out to be load-bearing for the wave's correctness. Three NEW gaps were discovered this way (SANDBOX manifest dispatch, EMIT no-op wrapper, IVM Algorithm B never registered).
- **The convergence trend was monotonic but non-linear.** R1=64 → R2=15 (artificially low due to lens-reduction) → R3=48 (full-council restoration revealed what was hidden) → R4=15 → R5=2 → R6=0. The R3 expansion was painful but load-bearing — without it phase-2b would have shipped with multiple BLOCKER-class drifts.
- **Pattern induction is its own lens, not a meta-process.** R3 + R4 + R5 + R6 each ran a `pattern-induction-meta-sweep` looking for UNNAMED / EMERGING / CROSS-LENS patterns the per-lens reduction would miss. R5's was the most exhaustive (329 lines walking 200+ prior artifacts) and found 0 new patterns — ratifying the 11 pim-N codifications as complete.
- **The deep sweeps each became standing R6 lenses.** What started as "cheap audit on 3+ recurrence threshold" graduated to permanent lenses by R6: cite-precision-deep-sweep, producer-consumer-deep-sweep, stale-deferrals-deep-sweep, pattern-induction-meta-sweep. The R6-R6 final council had 16 lenses — the original 12 + these 4 deep-sweep promotions.
- **Phase-2c was a phantom destination.** The `r5-decisions-log.md` had named "Phase-2c (or earlier)" as a carry destination on multiple items; Ben caught it mid-wave-8 (Phase-2c never existed on FULL-ROADMAP). All 9 carry-items at the log bottom were RESOLVED end-to-end at phase-2b-close with reconciliation table appended.
- **D-NS-250 phantom-destination redux.** Mid-R6-R5-FP, Ben surfaced "have you done the rule 1 approach for all this rounds reviewers?" and the orchestrator self-audit found "Phase-3-pre-R1 codification" had been silently treated as a destination across pim-3..10 deferrals.

**What this phase set up for the next phase:**

- **All 12 primitives production-runtime LIVE** is the foundation Phase 3 atriums build on; Phase 3 P2P sync would have nothing to sync without this.
- **The 8-crate boundary stayed clean** through the Subgraph relocation (G12-C-cont). `benten-ivm` → `benten-graph` dependency direction never inverted. The `Strategy` enum at the engine boundary is the load-bearing seam.
- **Phase-3-backlog.md was created mid-flight** with named-destination structure across 10+ sections; carries `Arc<dyn KVBackend>` refactor, browser-as-thin-compute-surface posture, TS-surface-parity sweep §7.9, SANDBOX casing-drift §6.6 acceptance criterion, and pim-6's CI-infrastructure half (§7.11).
- **11 pim-N process patterns are codified inline in dispatch-conventions** (only pim-6's CI-infra half remaining external).
- **The v1 milestone gate is established** (CLAUDE.md item #15): Phases 1+2a+2b+3 minimum + post-Phase-3 PAUSE-AND-ASSESS.
- **The night-shift discipline is documented and reusable** (`feedback_night_shift_stance`, NIGHT-SHIFT-LOG-2026-04-28 as precedent).
- **R6 phase-close convergence shape** — the 6-round arc with monotonic decreasing finding counts + final FULL council is the canonical Phase-N close shape; Phase-3 will use this same sequence.

**Phase-defining decisions** (the 1-3 calls that most shaped the outcome):

1. **Run wave-8 as 13 sub-tracks rather than restart-the-phase after R4b.** Choosing disciplined expansion + named-destination deferrals + per-group sync mini-reviews over a wave-restart was the call that allowed phase-2b-close to land in 6 days of night-shift autonomous run.
2. **Make sync mini-review per-group the default.** Three failure shapes (deferral-hat / named-destination-realness / landed-code-correctness) recurred through wave-8 and the R6 rounds; the per-group discipline caught all of them. `feedback_synchronous_mini_review` codifies it.
3. **Treat the FINAL R6 round as a FULL council, not a reduced one** (Ben's framing change mid-R6). Lens-reduction is right for iteration rounds; the final pre-tag round needs the full Round-1 lens set + 3+-recurrence redux. R6 Round 6 returning ALL 16/16 CONVERGED is the validation of this discipline.

---

## § 4. Backlog / compromises / incomplete work

### § 4.1 Carried into this phase from earlier phases

From `phase-2-backlog.md` (consolidated Phase-1 deferrals):

- **G6/G7 STREAM/SUBSCRIBE/SANDBOX surface land.** Status at phase-2b-close: CLOSED end-to-end (production runtime LIVE).
- **G8 IVM Algorithm B generalization.** Status: CLOSED (Strategy enum + AlgorithmBView at engine boundary; user-registered views shipped).
- **G9-G11 paper-prototype + canonical-fixture revalidation.** Status: CLOSED (canonical CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` stable across phase-2b; SANDBOX rate 16.7% < 30% gate).
- **G12 Subgraph migration + canonical-bytes shape (D28).** Status: CLOSED (D28 RESOLVED; eval-side CanonView authoritative shape; auto-derive serde dropped from 4 relocated subgraph types).

Phase-2a closures preserved as non-regression invariants (sec-r6r1-01 Inv-14 dead-coded wiring + sec-r6r2-02 test-only API gating sweep + sec-r6r3-02 parse counter cfg-gate + arch-1 benten-eval no benten-graph dep edge + canonical fixture CID bit-stable).

### § 4.2 Deferred out of this phase

Named into `docs/future/phase-3-backlog.md` (created 2026-05-02 mid-R6-R5-FP):

| Item | Surface | Target section | LOC est | Rationale |
|---|---|---|---|---|
| `Arc<dyn KVBackend>` trait-object refactor (PHASE-3-BUNDLE-1) | Engine binding to backend | §1.x (architecture) | 600-1,500 production + 200-400 test | Wave-5 D-NS-37 pivot to redb-on-memory; full refactor saved for Phase-3 atrium-storage seam |
| Browser-as-thin-compute-surface posture | wasm32-unknown-unknown bundle | §3.x | TBD | CLAUDE.md item #17; Loro/iroh/SANDBOX state stays out of bundle |
| Phase-3 wasm bundle tighten ≤350KB | Wasm bundle cap | §3.x | TBD | Trigger: after redb dropped from wasm32-unknown-unknown bundle |
| SANDBOX casing-drift acceptance criterion | 24th p/c instance | §6.6 | small | Instance 24 BELONGS-NAMED-NOW; explicit acceptance criterion |
| TS-surface-parity sweep | Edge interface phantom `cid` field + dropped `properties` | §7.9 | ~150 | Pre-Phase-2b drift; folded into FP brief at R6-R4 |
| pim-6 CI-infrastructure half | Workspace-wide regression-scan automation | §7.11 | TBD | Cross-crate workflow-level-constraint blind spot CI-infra side carries; discipline side codified inline at §3.5 |
| Phase-3-pre-R1 process-pattern ratifications | Catalog of pim-N follow-ups before Phase 3 R1 dispatch | §7.11 | small | D-NS-250 phantom-destination closure |
| `random` host-fn ships in Phase 3 | `random` host-fn surface | phase-3-backlog | small | (CLOSED at G17-A2 wave-5b) |
| SUBSCRIBE delivery cross-process retention-window enforcement | RedbSuspensionStore is_retention_exhausted | phase-3-backlog | medium | Cross-process re-subscribe past 1000/24h window |
| `AsyncKVBackend` trait | benten-graph backend | phase-3-backlog | medium | Snapshot-blob backend is sync-compatible; iroh integration in Phase 3 |
| Phase-3 Ed25519 signature validation on module manifests | D16 forward-compat | phase-3-backlog | medium | (CLOSED at G14-C wave-4b) |
| Per-tenant log scoping for SANDBOX `log` host-fn | Multi-tenant path | phase-3-backlog | medium | Phase 2b is single-user; cross-tenant log isolation requires `system:log:read` cap-gate |
| Cache-side-channel mitigation for `kv:read` | Multi-tenant cache partition | phase-3-backlog | medium | Per-actor cache partition or cache-line-flush only matters under multi-tenant |
| Runtime-resolvable manifest registry (D2 hybrid 'register_runtime') | Deferred error in 2b | Phase-8 marketplace plan | medium | Phase 2b ships `Err(SandboxManifestRegistrationDeferred)`; Phase-8 lifts the deferral |

Other named carries:
- **Stable rustdoc strict-lint pre-flight** (pim-7): codified into dispatch-conventions §3.5 inline
- **Cross-crate workflow-level-constraint blind spot** (pim-6): caught + closed orchestrator-direct mid-R6-R3-FP; CI automation half deferred
- **Bench-gate per-platform threshold table**: workspace-root `bench_thresholds.toml` per-platform tiered numeric (Linux x86_64 / macOS) — partial in Phase 2b, finalization deferred
- **AArch64 CI cell**: wasmtime support story for AArch64 deferred to Phase 3 sync wave
- **Wasmtime `cache` feature on-disk persistent module cache**: in tension with D3 'no cross-call state retention'; deferred for design re-examination

### § 4.3 Compromises that landed during this phase

(8 named compromises landed during Phase 2b; see Compromise registry in `docs/SECURITY-POSTURE.md` for full narratives.)

- **Compromise #14** SANDBOX cold-start cost (no opt-in pool) — Phase-2b additive; D3 RESOLVED — additive Phase-3 change if real-workload bottleneck
- **Compromise #15** `register_runtime` reserved with deferred error — Phase 8 (marketplace) closure target
- **Compromise #16** `random` host-fn deferred — Phase-3 closure target (subsequently closed at G17-A2 wave-5b: `getrandom` direct + capability-gated entropy budget per call: 4096 bytes default + per-manifest override + constant-time cap-policy check per sec-r1-3)
- **Compromise #17** In-memory module-bytes registry — single-process scope; closure path Phase-3 G14-C (durable `RedbBlobBackend`)
- **Compromise #18** In-memory handler-version chain (sibling to #17) — closure path Phase-3 G14-C (durable `system:HandlerVersion` zone)
- **Compromise #19** Browser-target persistent storage absent — manifests in-memory only on `wasm32-unknown-unknown`; closure path Phase-3 G18-A (PARTIAL closed; full closure deferred per phase-3-backlog §4.3)
- **Compromise #20** Cross-browser determinism CI cadence not yet established — closure path Phase-3 G18-A (PARTIAL closed; fixture bodies deferred per phase-3-backlog §4.3)
- **Compromise #21** Module manifest minimal CID-pin in Phase 2b; full Ed25519 deferred — closure path Phase-3 G14-C (Ed25519 sign + UCAN-proof-chain primary + publisher-key-registry fallback per D-PHASE-3-20 + crypto-minor-5)

The closing texture: all four "structural-but-not-runtime" R4b BLOCKERs (SANDBOX, STREAM, SUBSCRIBE, WAIT) closed end-to-end by phase-2b-close. Compromises #17 and #18 represent the only durable-storage debt carried into Phase 3+; both are scoped + named.

---

## § 5. Process lessons / pim-N catalog

The Phase 2b R6 council surfaced 11 named process-improvement-mechanisms (pim-1 through pim-11). All but pim-6's CI-infrastructure half were codified inline in `.addl/phase-2b/dispatch-conventions.md` post-tag.

### pim-1 — Doc-lag-on-code-fix (§3.5b post-fix doc-coupling pre-flight HARDENED)

- **Failure mode:** Every public-shape change must sweep adjacent docs before push; without this, doc cites drift to phantom symbols / wrong symbols / OLD symbols. PR #62 wired Inv-4 sandbox_depth runtime threading WITHOUT coupled doc updates → 5 cross-doc surfaces still describing runtime arm as dormant; PR #68 added 25 lines without coupled doc updates; PR #75 closed SubscribeArgs.handler removal but missed DSL-SPECIFICATION.md:2266 Quick Reference Card. **7+ recurrences in Phase-2b.**
- **R6 round surfaced:** R6-R2 doc-engineer + R6-R3 doc-engineer-redux + R6-R5 dx-optimizer-redux. 3-round recurrence = pim-1 ratification trigger.
- **Codified at:** dispatch-conventions §3.5b HARDENED.
- **Memory:** `feedback_post_fix_doc_coupling_preflight.md` (foundational tier).

### pim-2 — End-to-end test pin for closed-claim PRs (§3.6b)

- **Failure mode:** Closed-claim PRs landing with sentinel-presence tests instead of production-runtime arm assertions. Origin: wave-8b wsa-w8b-1 ("DISAGREE-WITH-EXPLANATION used as deferral hat" — Compromise #4 falsely claimed CLOSED).
- **R6 round surfaced:** R6-R2 napi (Instance 8 verification gap); R6-R3 fp-mr-group-b ratification.
- **Codified at:** dispatch-conventions §3.6b — production runtime arm + observable consequence + would-FAIL-if-no-op'd. Sentinel-presence tests don't suffice.
- **Memory:** `feedback_end_to_end_test_pin_for_closed_claims.md` (foundational tier).

### pim-3 — Phase-close final council = FULL Round-1 lens set (§3.9)

- **Failure mode:** R6-R2 lens reduction (12 → 5 lenses) MISSED still-open r6-sec-4 + post-PR#62 doc-coupling failures + 30+ findings that R3's full-council restoration surfaced. The R2 5-lens choice was right for ITERATION rounds but wrong for FINAL gate.
- **R6 round surfaced:** R6-R2 (the misfire) + R6-R3 (the corrective restoration).
- **Codified at:** dispatch-conventions §3.9.
- **Memory:** `feedback_phase_close_final_council_full.md` (foundational tier).

### pim-4 — 3+-recurrence triggers deep retrospective sweep (§3.10)

- **Failure mode:** R6-R1 surfaced known-pattern instances 4-7 deep without dispatching deep sweep until Ben directly intervened. The iceberg was 8x larger than Round 1's tip in stale-`#[ignore]` cluster.
- **R6 round surfaced:** R6-R1 (Ben's mid-round directive: "3 instances → broader investigation").
- **Codified at:** dispatch-conventions §3.10.
- **Memory:** `feedback_3_plus_recurrence_deep_sweep.md` (foundational tier — threshold is 3 not 5; cheap audits beat missed BLOCKERs).

### pim-5 — Mini-review verdict shape (no comma-clause "READY-TO-MERGE-WITH-X") (§3.8)

- **Failure mode:** HARD RULE rule-1 protection — "READY-TO-MERGE-WITH-1-MINOR-NEW-FINDING-AND-1-ARCH-SURFACE-FOR-BEN" hides a deferral inside an approval verdict.
- **R6 round surfaced:** w8b fix-pass mini-review verdict shape pattern; pim-meta-sweep R6-R3 named it.
- **Codified at:** dispatch-conventions §3.8.

### pim-6 — Cross-crate workflow-constraint blind spot (§3.4b convention half + §7.11 CI-infra residual)

- **Failure mode:** Inv-11 cross-crate cascade catches an issue local pre-flight didn't — workspace-level-constraint that crosses crate boundaries.
- **R6 round surfaced:** R6 Round 3 caught Inv-11 cross-crate cascade orchestrator-direct.
- **Codified at:** dispatch-conventions §3.4b (convention half) + §7.11 (CI-infra residual).
- **Memory:** `feedback_cross_crate_field_cascade.md` (load-bearing operational tier).

### pim-7 — Stable rustdoc strict-lint blind spot (§3.5 dimension #5)

- **Failure mode:** `private_intra_doc_links` + `broken_intra_doc_links` can fire on stable rustdoc but not on default `cargo doc`.
- **R6 round surfaced:** R6-R3 caught after stable rustdoc strict-lint surface examined.
- **Codified at:** `cargo +stable doc --workspace --no-deps` as standing pre-flight (dispatch-conventions §3.5 6th blind spot).

### pim-8 — Mirror-precedent overshoot guard (§3.6c)

- **Failure mode:** An agent following a mirror-precedent (e.g., "mirror EMIT precedent at SUBSCRIBE site") overshoots — implements the structural shape too literally and misses a semantic difference (e.g., SubscribeArgs.handler vs pattern reading).
- **R6 round surfaced:** R6 Round 5 narrow-iter found 21st p/c instance: PR #74's r6-r4-cr-1 fix mirroring EMIT precedent too literally.
- **Codified at:** dispatch-conventions §3.6c.

### pim-9 — Grep-verification on every cite in NEW prose blocks (§3.5b point 4)

- **Failure mode:** §3.5b HARDENED already verifies CHANGED cites; pim-9 extends to NEW prose blocks where cites are introduced fresh.
- **R6 round surfaced:** R6 Round 5 cite-precision narrow-iter found 2 NEW phantom cites IN PR #74's OWN DIFF.
- **Codified at:** dispatch-conventions §3.5b point 4.

### pim-10 — Narrow-iteration cycle as effective FP follow-up (§3.7b)

- **Failure mode:** (positive process-shape, not failure-prevention) — narrow-iteration cycle on an FP catches what the bundled mini-review missed at wider scope.
- **R6 round surfaced:** R6 Round 4 narrow iteration after R6-R3-FP merges all CONVERGED.
- **Codified at:** dispatch-conventions §3.7b.

### pim-11 — Phantom-destination defense + named-destination registry (§3.6d)

- **Failure mode:** HARD RULE rule-12 violation — naming a destination that doesn't exist (Phase-2c not on roadmap; Phase-3-pre-R1 codification not a doc).
- **R6 round surfaced:** D-NS-250 phantom-destination catch by Ben mid-R6-R5-FP.
- **Codified at:** dispatch-conventions §3.6d + `docs/future/phase-3-backlog.md` § structure designed as named-destination registry.
- **Memory:** `feedback_no_defer_HARD_RULE.md` updated with verify-destination-exists sub-section.

### Summary table

| pim-N | Codified at | Memory file | Tier |
|---|---|---|---|
| pim-1 | §3.5b HARDENED | `feedback_post_fix_doc_coupling_preflight` | foundational |
| pim-2 | §3.6b | `feedback_end_to_end_test_pin_for_closed_claims` | foundational |
| pim-3 | §3.9 | `feedback_phase_close_final_council_full` | foundational |
| pim-4 | §3.10 | `feedback_3_plus_recurrence_deep_sweep` | foundational |
| pim-5 | §3.8 | (catalog only) | situational |
| pim-6 | §3.4b (convention) + §7.11 (CI-infra residual) | `feedback_cross_crate_field_cascade` | load-bearing operational |
| pim-7 | §3.5 dim #5 | (catalog only) | load-bearing operational |
| pim-8 | §3.6c | (catalog only) | load-bearing operational |
| pim-9 | §3.5b pt-4 | `feedback_rustdoc_path_style_cite_brackets` (Phase-3 R5 sibling) | load-bearing operational |
| pim-10 | §3.7b | (catalog only) | situational |
| pim-11 | §3.6d | `feedback_no_defer_HARD_RULE` (sub-section) | foundational |

### 24 producer/consumer drift instances — final disposition catalog

(All 24 instances dispositioned by phase close. 21 closed end-to-end + 22+23 by PR #76 + 24 BELONGS-NAMED-NOW phase-3-backlog §6.6.)

- **Instances 1-5:** wave-8 known instances (SANDBOX result.fuel_consumed/output_consumed dropped; STREAM chunk seq dropped; SUBSCRIBE napi dropped pre-PR-#66; WAIT resume metadata; EMIT no napi adapter pre-PR-#66).
- **Instance 6 BLOCKER:** graph::ChangeEvent → eval::ChangeEvent bridge multi-label loss — closed via PR #62 (widen eval-side ChangeEvent + walk all labels).
- **Instances 7-12:** TS Subscription.maxDeliveredSeq snapshot; BentenError.context not populated; DSL Diagnostic line/column Debug-formatted; register_subgraph_replace outcome dropped; u64→u32 silent saturation; SuspensionBridge.Suspended state_cid+signal_name dropped.
- **Instance 14:** PR #62 EmitSubscriptionJs napi class TS-side surface incompleteness (R6-R2).
- **Instances 15-17:** R6-R3 napi typed-error round-trips (E_DEVSERVER_STOPPED, E_RELOAD_SUBSCRIBER_UNSUBSCRIBED).
- **Instances 18-20:** R6-R3 redux mermaid SUBSCRIBE arm + AttributionFrame.sandboxDepth widening + r6-r3-ivm-1 ViewLabelMismatch fail-loud.
- **Instance 21:** SubscribeArgs.handler removal — closed PR #75.
- **Instances 22-23:** PR #76 mermaid SUBSCRIBE arm + WAIT duration translation closures.
- **Instance 24:** SANDBOX casing-drift acceptance criterion — BELONGS-NAMED-NOW into phase-3-backlog.md §6.6 (post-tag morning review per HARD RULE rule-12).

### Cross-cutting mini-review failure shapes

Codified into `feedback_synchronous_mini_review.md`:
- **Shape 1 — deferral-hat:** agent SAID full scope landed; landed only half (8b BLOCKER).
- **Shape 2 — named-destination-realness:** agent SAID it deferred + named destination; mini-review validates destination is real (8c-cont READY-TO-MERGE-AS-PARTIAL).
- **Shape 3 — landed-code-correctness:** agent SAID full scope; mini-review validates landed code's correctness/DX (8f NEEDS-FIX-PASS for lock-ordering).

### Other Phase-2b operational lessons (memories created or updated)

- `feedback_plain_english_surfaces` (foundational): every Ben-facing surface follows plain-situation → concrete-options → my-prediction-of-Ben's-call → confirm-or-redirect.
- `feedback_pattern_induction_meta_sweep` (load-bearing): companion to known-pattern reduxes; catches UNNAMED / emerging cross-lens patterns; 5 categories; promoted to standing R6 lens.
- `feedback_v1_milestone_gate`: v1 boundary covers Phases 1+2a+2b+3 minimum; PAUSE-AND-ASSESS after Phase 3 closes.
- `feedback_agent_local_preflight_blind_spots`: 4 cross-target pre-flight blind spots (stable rustfmt + MSRV 1.95 clippy + wasm32 target + drift-detector construction sites) baked into §3.5.
- `feedback_no_defer_HARD_RULE` (foundational, updated mid-phase): only 3 valid non-fix-now dispositions; verify destination EXISTS before naming it.
- `feedback_night_shift_stance` (foundational, codified post-NIGHT-SHIFT-LOG-2026-04-28): autonomous-mode discipline.
- `feedback_subtrack_sizing_heuristic` (situational): ~400-800 LOC sweet-spot.
- `feedback_synchronous_mini_review` (load-bearing): pre-merge default; catches deferral-hat / named-destination-realness / landed-code-correctness failure modes.

The discipline that emerged: **trust agent local pre-flight as a necessary-but-not-sufficient signal; CI is the authoritative full-workspace verifier; sync mini-review per group is the load-bearing correctness layer; HARD RULE rule-12 is the project's-most-violated discipline and requires Ben-as-second-pair-of-eyes for phantom-destination catches.**

---

## § 6. Decisions baked in / architectural commitments

### From pre-R1 + R1

1. **D9 — Module manifest format = canonical DAG-CBOR (NOT TOML/JSON).** Phase 2b R1 close. Source: security-auditor R1 (parser-divergence-as-attack-surface) + benten-philosophy r1 (content-addressing symmetry). Lives now: `docs/MODULE-MANIFEST.md` + module_manifest.rs + canonical_bytes_round_trip test. Why it matters: closes Inv-13 collision class by construction; Phase-3 Ed25519 signing target is the canonical bytes.

2. **D16 — Module install requires `expected_cid: Cid` arg.** Phase 2b R1 (pre-R1 minimum-viable shipping); Ed25519 deferred to Phase 3. Source: sec-pre-r1-01 named 3 attack vectors (manifest-channel-compromise, swap-after-review, supply-chain-by-CID-confusion). ~50 LOC additive. Lives now: `Engine::install_module(manifest_cid: Cid, manifest: ModuleManifest)` + E_MODULE_MANIFEST_CID_MISMATCH error includes BOTH expected and computed CID. Why it matters: closes 3 named attack vectors; signature `Option<None>` reserved for Phase-3 forward-compat.

3. **D8 — IVM Algorithm B explicit-opt-in Strategy enum + KEEP-ALL-PARALLEL 5 hand-written views.** Phase 2b R1. Source: ivm-algorithm-b-reviewer R1. Lives now: `enum Strategy { A, B, C }` in benten-ivm; default `Strategy::A` for 5 existing constructors; `Strategy::C` reserved for Phase-3 Z-set/DBSP. Why it matters: bench gate has measurable meaning only if hand-written remain as runtime baselines.

4. **D14 — TraceStep mapping uses warning-passthrough with typed `UnknownTraceStep` variant.** Phase 2b R1; explicit DISAGREE with architect-reviewer's loud-fail recommendation. Source: benten-philosophy (forward-compat) + dx-optimizer (SemVer-tolerant). Lives now: TraceStep enum + types.ts TraceStepUnknown variant + mapTraceStep dedupe-warn. Why it matters: preserves third-party Phase-8 marketplace TS consumer compatibility.

5. **D17 — Inv-7 output enforcement = defense-in-depth two-layer (CountedSink PRIMARY + return-value BACKSTOP).** Phase 2b R1; cross-lens convergence (wsa-1 + sec-r1 D17). Lives now: `crates/benten-eval/src/sandbox/output_budget.rs`. Why it matters: single-mechanism enforcement is one missed-codepath away from a silent overage; defense-in-depth is the correct posture.

6. **D18 — Cap-recheck cadence = HYBRID per-cap (declared in host-functions.toml).** Phase 2b R1; cross-lens convergence (sec + wsa). Lives now: host-functions.toml `cap_recheck = "per_call" | "per_boundary"` field per host-fn; default per_call (fail-secure). Why it matters: tighter TOCTOU bound than init-snapshot.

7. **D19 — Reentrancy denial scope = calibrated nested-dispatch denial; rename to `E_SANDBOX_NESTED_DISPATCH_DENIED`.** Phase 2b R1. Lives now: error-catalog rename + `host:async` cap reservation + wasmtime async-support feature enabled in 2b. Why it matters: precisely captures the security claim from ENGINE-SPEC §10 without foreclosing async host-fns Phase 3 needs for iroh KVBackend.

8. **D20 — Inv-4 runtime depth counter = AttributionFrame-inherited cumulative `sandbox_depth: u8`.** Phase 2b R1. Lives now: AttributionFrame.sandbox_depth field gated out of canonical bytes when zero. Why it matters: SANDBOX-bearing frames content-distinguishable from non-SANDBOX; cross-CALL nesting visible.

9. **D21 — 4-axis fire priority = severity ordering MEMORY > WALLCLOCK > FUEL > OUTPUT.** Phase 2b R1; cross-lens convergence. Lives now: `enum SandboxTrapKind` Ord impl + `resolve_priority` function; documented in `docs/SANDBOX-LIMITS.md`. Why it matters: typed-error contract under simultaneous trip is deterministic, not platform-dependent.

### From R2 + R3 + wave-8 + R5 + R6

10. **D26 — Pre-built `.wasm` fixtures committed; `build_wasm.sh` dev-only regenerator.** Phase 2b R3. Why it matters: CI doesn't depend on `wat` crate at run-time; canonical bytes for each escape are content-hash-pinned.

11. **`phase_2b_landed = []` cargo feature mechanism — Phase-2b standing pattern.** Phase 2b R3; first live use of branch-per-agent CI flow. Lives now: Cargo.toml feature in 4 crates; 28 top-level test/bench files cfg-gated; 17 integration submodules wrapped (retired in pre-R4b PR #144).

12. **PHASE-3-BUNDLE-1 commitment — Engine genericism over GraphBackend umbrella trait.** Phase 2b wave-8j; deferred to Phase 3 with NAMED destination per HARD RULE. Lives now: `docs/future/phase-3-backlog.md` + scoping plan + 3-track dispatch brief skeleton. Why it matters: enables `SnapshotBlobBackend` direct-wire; enables wasm bundle tighten to ~350KB; enables `BrowserBackend` substitution.

13. **Wasm bundle cap = 600KB gzipped for Phase 2b; tighten to ≤350KB Phase 3.** Phase 2b wave-8j-ci-cleanup PR. Source: wave-8j-wasm-browser-bundle-bisect (cap had NEVER been under 500KB until profile knobs landed). Lives now: `Cargo.toml [profile.release-wasm]` + wasm-opt -Oz CI step + `bindings/napi/tests/wasm32_unknown_unknown_bundle_size_under_threshold.rs`. Why it matters: makes the wave-8j workflow green sustainably.

14. **Compromise #4 (SANDBOX named-allowlist) closed via must-invoke positive assertion.** Phase wave-8b fix-pass. Lives now: `docs/SECURITY-POSTURE.md` + acceptance test `sandbox_execute_via_engine_dispatch_invokes_executor`. Why it matters: structural defense against the "DISAGREE-WITH-EXPLANATION as deferral hat" failure mode.

15. **Compromises #9 + #10 closed via G12-E generalized SuspensionStore + 8i-wait integration.** Phase wave-6 G12-E + wave-8i.

16. **Compromises #17 + #18 ACCEPTED in-memory-only single-process scope.** Phase wave-7 G7-A (#17) + wave-8f (#18). Phase 6 durable module store is the closure path (subsequently closed at Phase-3 G14-C wave-4b).

17. **D28 RESOLVED: eval-side production canonical-bytes shape (`{handler_id, sorted nodes, sorted edges, deterministic}` via CanonView) is authoritative.** Phase G12-C-cont fix-pass A.1. Lives now: `docs/ARCHITECTURE.md` § canonical-bytes. Why it matters: re-opening would re-cost the canonical fixture CID stability.

18. **Wallclock REJECT (not CLAMP) ratified.** Phase wave-4 G8-A bench-gate + wave-8 SANDBOX dispatch. Source: Ben pre-bed decision 2026-04-28.

19. **G8-A bench-gate hybrid threshold (per-view absolute-overhead-ns ceiling ≤350ns lightweight + ratio ≤1.50 heavyweight).** Phase wave-4 G8-A.

20. **Cross-crate field-cascade pattern: one-time `cargo check --workspace --all-targets` exception when adding new public struct fields.** Phase wave-4 G7-B + G8-A. Source: Two simultaneous cascades named the pattern. Lives now: `dispatch-conventions.md §3.4`.

21. **MSRV bumped 1.91 → 1.95 across workspace.** Phase wave-8e (durable closure of 1.95 lint cascade). Source: HANDOFF-2026-04-29-morning Ben-ratification.

22. **Single `phase-N-close` annotated tag at phase end** (cadence convention). Source: Ben-ratification 2026-04-29 (hold all tags until wave-8 + R6 close).

23. **v1 milestone gate = Phases 1+2a+2b+3 minimum + post-Phase-3 PAUSE-AND-ASSESS.** Phase night-shift wave-8j-cleanup conversation 2026-04-29. Lives now: CLAUDE.md item #15. Why it matters: establishes shippable-boundary intent without pre-deciding scope shrinkage.

24. **Sync mini-review per implementation group is non-negotiable (Pattern 6 by lens surface).** Phase R5 wave-1 G12-A; reaffirmed in every wave since. Source: G12-A discovery + 8b BLOCKER + 8c-subscribe-infra BLOCKER + 8f HIGH lock-ordering all caught by sync mini-review (NOT CI). Lives now: `dispatch-conventions.md §3.6` + `feedback_synchronous_mini_review.md`.

25. **Final R6 round = FULL council, not lens-reduction.** Phase R6 Round 2 framing change ratified by Ben; applied at R6 Round 3, 4, 5, 6. Lives now: `feedback_phase_close_final_council_full.md` + `dispatch-conventions.md §3.7`.

26. **3+-recurrence rule (threshold 3, not 5) triggers deep retrospective sweep.** Phase R6 Round 1 deep-sweep authorization. Lives now: `feedback_3_plus_recurrence_deep_sweep.md` foundational memory.

27. **Pattern-induction meta-sweep is companion to known-pattern reduxes at every R6 round.** Phase R6 Round 3 onwards. Lives now: `feedback_pattern_induction_meta_sweep.md` + `dispatch-conventions.md §3.7`.

28. **HARD RULE rule-12 (no "later" disposition; only 3 valid non-fix-now): only OUT-OF-SCOPE / BELONGS-NAMED-NOW (with destination existing) / DISAGREE-WITH-EXPLANATION.** Phase established earlier; re-strengthened with no-time-qualifier framing 2026-04-27 evening; updated with verify-destination-exists sub-section after Phase-2c phantom-destination catch. Lives now: CLAUDE.md §12 (foundational) + `feedback_no_defer_HARD_RULE.md`. Why it matters: the project's-most-violated discipline.

### Re-affirmations (Phase-2b confirmed earlier commitments)

- **12 operation primitives** (CLAUDE.md item #1) — re-affirmed; SANDBOX is the escape hatch for compute that doesn't fit the other 11. Per CLAUDE.md item #16 (Phase-2b post-R5 addition 2026-05-04 D-D follow-up): SANDBOX modules do NOT duplicate other primitives' capabilities; host-fn surface stays minimum-viable; storage-mutating host-fns explicitly NOT engine concerns.
- **IVM Algorithm B** (CLAUDE.md item #2) — landed at engine boundary via Strategy enum + AlgorithmBView. CLAUDE.md baked-decision #2 reword landed during R6-R1 fix-pass: original wording "evaluator is ignorant of IVM" was sharpened to "evaluator doesn't reach into IVM incremental-recomputation internals (the engine names benten_ivm::Strategy as the dispatch type but no View / algorithm internals leak through; benten-ivm depends on benten-graph::ChangeSubscriber, never the reverse)" — to match the actually-shipped boundary.
- **8-crate boundary** (CLAUDE.md §architecture) — held through Subgraph relocation. ARCH-1 dep-break invariant verified on every R6 HEAD (4-job CI gate: benten-core-no-eval-dep + cargo-toml-edge-check + primitive-host-type-grep + primitive-host-trait-signature-gate).
- **Capability system as pluggable policy** (CLAUDE.md item #7) — manifest registry in G7-A retains the pre-write hook discipline.
- **EmitBroadcast separate channel** — RATIFIED at R1 architect-reviewer + R6-R6 producer-consumer-deep-sweep CONVERGED. Why it matters: preserves the IVM ↔ EMIT semantic separation; Phase-3 may unify if a real use case arises.

**Phase 2b R6 quality council shipped at tag `phase-2b-close` (`3d0f018`, 2026-05-03)** — 16/16 CONVERGED at FINAL gate.
