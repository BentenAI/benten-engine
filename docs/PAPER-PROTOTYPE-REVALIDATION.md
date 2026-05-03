# Paper-Prototype Revalidation — Phase 2b Close

**Status:** FULL revalidation, Phase 2b R5 wave-7 (G11-2b).
**Authoritative gate:** plan §1 exit-criterion #1 — SANDBOX rate ≤ 30%
against the revised 12-primitive vocabulary.
**Companion staged check:** `tests/sandbox_rate_under_30_percent.rs`
(G7-close dry-run, ≥ 4-week remediation runway).
**Methodology:** D11-RESOLVED hybrid — single-sample (canonical
fixture) + 3-5 cohort handlers covering the breadth of Phase-2b
primitive surface.

---

## SANDBOX rate: 16.7%

SANDBOX rate: 16.7%

**Verdict: PASS.** 2 of 12 cohort handlers compose against the SANDBOX
primitive; 10 are expressible from the 11 non-SANDBOX primitives
(READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT,
SUBSCRIBE, STREAM).

The exit-criterion gate is **30%**. The measured rate is well under
the gate, validating the architectural claim that the 12-primitive
vocabulary covers real-workload expressivity without over-relying on
the WASM escape hatch.

---

## Methodology — D11 hybrid

The classification proceeds by:

1. **Single-sample anchor.** The canonical fixture (`crud('post')` —
   the zero-config DX-marker handler) is classified first; this is
   the same fixture that backs the canonical-CID stability test
   (`crates/benten-core/tests/fixtures/canonical_cid.txt`,
   `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`).
2. **Cohort widening.** 11 additional handlers drawn from the
   union of (a) Phase-1 8-primitive surface, (b) Phase-2a WAIT-resume
   surface, (c) Phase-2b STREAM/SUBSCRIBE/SANDBOX surface. Each
   cohort handler exercises a different primitive composition so the
   classifier surfaces differ.
3. **Per-handler verdict.** "needs SANDBOX" iff the handler can
   ONLY be expressed by invoking the SANDBOX primitive (i.e. requires
   a host-fn that only WASM modules expose — arbitrary computation,
   format conversion, regex, hashing beyond BLAKE3, etc.). "no
   SANDBOX" means the handler composes from the 11 non-SANDBOX
   primitives.
4. **Rate.** `count(needs_sandbox) / count(total)`; gate is ≤ 0.30.

A true statistical-weight study (large user-corpus sampling, p-values,
confidence bands) is **Phase-3 territory** per D11. The hybrid here
is the gate the architecture commits to clear before Phase 2b can
close.

---

## Per-handler classification

| # | Handler | Primitives | Needs SANDBOX? | Rationale |
|---|---------|-----------|---------------|-----------|
| 1 | `crud('post').create` (canonical fixture) | READ, WRITE, RESPOND | No | Pure storage path: read-by-key idempotency check + WRITE + RESPOND. |
| 2 | `crud('post').list` | READ, RESPOND | No | Single READ over a label, RESPOND. No transformation needed. |
| 3 | `crud('post').update` | READ, TRANSFORM, WRITE, RESPOND | No | TRANSFORM merges patch into existing properties (TRANSFORM grammar covers object-spread). |
| 4 | `crud('post').delete` | READ, WRITE (tombstone), RESPOND | No | Tombstone WRITE pattern; no compute path. |
| 5 | `payment-confirm` (WAIT-signal handler from QUICKSTART) | READ, WAIT, BRANCH, WRITE, RESPOND | No | WAIT suspends on `external:payment_confirmed`; resume routes through BRANCH on payment status. |
| 6 | `iter-batch-import` (ITERATE over CSV rows) | READ, ITERATE, TRANSFORM, WRITE, RESPOND | No | ITERATE bound to row count; TRANSFORM normalises each row; bounded DAG, no compute escape needed. |
| 7 | `event-fanout` (EMIT after every WRITE for downstream subscribers) | READ, WRITE, EMIT, RESPOND | No | EMIT publishes the change-event to the event bus; no compute. |
| 8 | `delegating-aggregator` (CALL into another handler with isolated caps) | CALL, READ, RESPOND | No | CALL composes; isolation via `isolated: true`. |
| 9 | `change-feed-mirror` (SUBSCRIBE to label changes; project into a derived view) | SUBSCRIBE, TRANSFORM, WRITE, EMIT | No | Reactive primitive; TRANSFORM projects, WRITE persists derived state. |
| 10 | `chunked-export` (STREAM rows back to client with back-pressure) | READ, ITERATE, STREAM, RESPOND | No | STREAM yields chunks per ITERATE step; output bounded by client pull. |
| 11 | `summarize-doc-with-llm` (call out to a deterministic-text-summariser WASM module) | READ, **SANDBOX**, WRITE, RESPOND | **Yes** | Text summarisation is non-decomposable into the 11 non-SANDBOX primitives — TRANSFORM grammar is intentionally arithmetic-and-object-spread; no NLP or regex. SANDBOX with `compute-with-kv` manifest. |
| 12 | `image-thumbnail` (resize an uploaded blob to 256×256 PNG) | READ, **SANDBOX**, WRITE, RESPOND | **Yes** | Image resampling is pure-CPU compute over byte arrays; no expressible TRANSFORM equivalent. SANDBOX with `compute-basic` manifest (no KV reads needed). |

**Tally:** 2 / 12 = **16.7%** SANDBOX rate.

---

## Cohort coverage check

The 12-handler cohort touches every Phase-2b primitive at least once:

| Primitive | Covered by handler(s) |
|-----------|-----------------------|
| READ | 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12 |
| WRITE | 1, 3, 4, 6, 7, 9, 11, 12 |
| TRANSFORM | 3, 6, 9 |
| BRANCH | 5 |
| ITERATE | 6, 10 |
| WAIT | 5 |
| CALL | 8 |
| RESPOND | 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12 |
| EMIT | 7, 9 |
| SANDBOX | 11, 12 |
| SUBSCRIBE | 9 |
| STREAM | 10 |

All 12 primitives are exercised. No primitive is "dead" in the
cohort — the classification is not biased by leaving compute-heavy
surface uncovered.

---

## What this gate proves

- The **non-Turing-complete-DAGs** baked-in decision (CLAUDE.md item
  #4) is empirically validated: real-workload handlers do NOT mostly
  collapse onto SANDBOX. Most handlers compose from the typed,
  invariant-enforceable primitives.
- The **2 SANDBOX cases** (LLM summarisation, image resampling) are
  exactly the cases where SANDBOX is the **right** abstraction — they
  need pure-CPU compute over arbitrary byte arrays, which is what
  WASM excels at. The escape hatch is being used as designed.
- The **16.7% rate** leaves significant headroom under the 30% gate,
  meaning the primitive surface can absorb future workload drift
  without immediately re-debating the architecture.

## What this gate does NOT prove

- This is a **single-cohort revalidation**, not a statistical study.
  Phase-3 user-corpus sampling will produce a true confidence band.
- The 12 handlers were selected to span the primitive surface, not
  randomly drawn from a workload distribution. A real workload could
  weight differently (e.g. heavy SUBSCRIBE-driven projection
  workloads).
- The classification is **manual** — a different classifier might
  argue that, say, CSV parsing in handler #6 should be SANDBOX
  because TRANSFORM can't strictly do delimited-string parsing. The
  classification here treats TRANSFORM-grammar's string operations as
  sufficient for well-formed CSV; an adversarial classifier would
  push that handler into SANDBOX and the rate to 25%, still under the
  gate.

---

## Reproducibility

The cohort classification is the **authoritative dataset** for the
gate. To reproduce:

1. Read this file's "Per-handler classification" table.
2. For each row, decide independently whether the handler primitives
   list could omit SANDBOX while preserving handler semantics.
3. Compute `count(needs_sandbox) / count(total)`.
4. Compare against 30%.

The gate test
(`crates/benten-eval/tests/sandbox_rate_full_revalidation_g11_2b.rs`)
parses the `SANDBOX rate: NN.N%` line from this file (line 5 above)
and asserts ≤ 30.0. Any future change to the cohort that pushes the
rate higher MUST also update the rate line at the top of this doc; CI
will fail otherwise.

---

## Runtime status as of `phase-2b-close`

The 16.7% rate is validated against the **architectural primitive
surface** — i.e. the question "is the primitive set sufficient to
express each cohort handler shape?". Wave-7 closed the structural
revalidation; the architectural verdict (PASS at 16.7%) is independent
of whether each primitive's runtime executor is wired end-to-end.

Wave-8 (post-wave-7 audit closure) wired the production runtime for
the three Phase-2b primitives so the cohort is now **executable**
end-to-end:

- **STREAM** (handler #10 `chunked-export`) — wave-8c-stream-infra
  wired the chunk-producer + tokio-mpsc drain-path; `engine.callStream`
  yields real chunks rather than `E_PRIMITIVE_NOT_IMPLEMENTED`.
- **SUBSCRIBE** (handler #9 `change-feed-mirror`) — wave-8c-subscribe-infra
  wired the ChangeStream port + ThreadsafeFunction trampoline;
  `engine.onChange` callbacks fire on the libuv main loop with real
  payloads.
- **SANDBOX** (handlers #11 `summarize-doc-with-llm`, #12 `image-thumbnail`)
  — wave-8b wired the wasmtime invocation pipeline; wave-8h hydrated
  the manifest-registry from `installed_modules` so Named-manifest
  dispatch resolves through the production lookup path.
- **WAIT** (handler #5 `payment-confirm`) — wave-8i routed
  `engine.call()` through eval-side `wait::evaluate_op` and replaced
  the `should_suspend(handler_id)` heuristic with property-aware
  suspension that consults `signal/duration_ms/timeout_ms/signal_shape`.
- **EMIT** (handler #7 `event-fanout`) — wave-8h wired the engine
  wrapper to publish through the dedicated `EmitBroadcast` channel
  (separate from `ChangeBroadcast`); `Engine::subscribe_emit_events`
  exposes the public subscription surface.

Bounded carry-forwards from wave-8 close that don't invalidate the
revalidation:

- **Inv-4 runtime depth-threading** (wave-8e item #11; closed at
  R6FP-G1 / PR #62, ratified again post-r6-r3-pcds-1 closure) — both
  arms now active. Production `AttributionFrame.sandbox_depth` threads
  transitively across nested SANDBOX entries via
  `frame.sandbox_depth.saturating_add(1)` at every SANDBOX boundary
  (`crates/benten-engine/src/primitive_host.rs::execute_sandbox` at
  SANDBOX entry; `engine.rs::dispatch_call_inner` inherits parent
  depth at CALL push), and the eval-side runtime arm in
  `crates/benten-eval/src/primitives/sandbox.rs::execute` fires
  `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED` when the inherited depth
  exceeds `config.max_nest_depth`. The JS-side surface widens via the
  napi trace projection so trace UIs / Phase-6 forking can reason about
  per-step nest depth (R6-R3 r6-r3-pcds-1 closure). Cohort handlers
  don't exercise SANDBOX → CALL → SANDBOX chains, so even the prior
  partial-arm wiring would not have changed the revalidation verdict;
  the now-fully-wired form removes the carry-forward entirely.
- **Compromise #17 module-bytes registry** — operators re-call
  `register_module_bytes` at engine open. Cohort handlers #11 + #12
  exercise the in-process register-then-dispatch shape, which is
  unaffected.
- **IVM Algorithm B non-canonical-view fallback** — handler #9
  (`change-feed-mirror`) operates on canonical-view shapes already
  covered by `AlgorithmBView::for_id`; the non-canonical fallback
  doesn't bite this cohort.

---

## Process observation — 4-instance metadata-producer-vs-consumer pattern (wave-8 retrospective)

Wave-8's synchronous mini-review pattern caught a recurring class of
bug **four distinct times** across the wave: a metadata-producing
surface was added correctly, but the consumer surface that should read
the metadata silently dropped it. The 4 instances:

1. **Wave-8b sandbox runtime** — the dispatcher was flipped to invoke
   `sandbox::execute(...)` correctly, but the `manifest_registry()`
   accessor that hydrates from `installed_modules` was missed; the three
   consumer sites in `primitive_host.rs` (pre-wave-8h line numbers
   `759, 770, 810`; post-wave-8h these moved to `831, 842, 885` after
   the audit-gap fix landed) used `ManifestRegistry::new()` (empty).
   Caught by wave-8h docs-vs-code audit; fixed in wave-8h.
2. **Wave-8c-subscribe-infra napi** — the engine-side
   `Engine::on_change_as_with_cursor` wired the cap-recheck closure +
   actor binding; the napi-side `subscribe_adapter` initially didn't
   thread the actor through to the closure. Caught by wave-8c-cont
   mini-review.
3. **Wave-8i WAIT elapsed_ms metadata** — the eval-side
   `wait::evaluate_op` correctly stamped suspension metadata
   (`elapsed_ms`, `signal`, `duration_ms`); the resume path's metadata
   read consulted only `signal` initially. Caught by wave-8i fix-pass-1
   mini-review.
4. **Wave-8i WAIT resume deadline** — `Engine::resume_with_meta` wired
   the deadline metadata writer; the public engine API didn't read the
   deadline back on resume. Caught by wave-8i fix-pass-2 verification
   (orchestrator-direct grep audit; landed at `ba749c3`).

**Reviewer-lens note for R6 phase-close council:** for every
metadata-producing surface added, EVERY consumer surface that should
read it must be audited. The 4-instance pattern across one wave
indicates this is a load-bearing reviewer-lens question. Future review
briefs should explicitly mandate "for each new metadata field, list
every consumer site and verify the read happens" as a checklist item.
This observation is process-only (no code change for paper-prototype
revalidation); it's recorded here because PAPER-PROTOTYPE-REVALIDATION
is the canonical wave-8-close artifact + the reviewer pattern matters
beyond Phase 2b.

**Update — R6 Round-1 + Round-2 deep-sweep expanded the count to
13+ instances.** The 4-instance enumeration above is the wave-8-era
retrospective. The R6 phase-close council's metadata-producer-vs-consumer
lens (Round 1 deep-sweep at `e2b1c62`, Round 2 at `fa001fc`) catalogued
13 distinct instances total — Instances 1-5 are the wave-8 retrospectives
above, Instances 6-13 are R6-discovered (multi-label ChangeEvent bridge
drop, TS Subscription max-delivered-seq snapshot, BentenError.context
sentinel pipeline, DSL Diagnostic line/col, registerSubgraphReplace
6-key shape, u64→i64 widening, SuspensionBridge state_cid+signal_name).
Round-2 added one further instance (r6-r2-mpc-1: EmitSubscription engine
half landed but napi `Engine::on_emit` method missing — closed in this
fix-pass). The recurrence rate dropped sharply between rounds (Round 1:
7 new on `e2b1c62`; Round 2: 1 new on `fa001fc`); the
consumer-audit-table discipline codified in `dispatch-conventions.md`
§3.6 + §3.7 is the standing rule going forward. See full Round-1 +
Round-2 enumeration in `.addl/phase-2b/r6-round-2-deep-producer-consumer-sweep.md`
+ `.addl/phase-2b/r6-r2-metadata-producer-vs-consumer.json`.

---

## Sign-off

Phase 2b paper-prototype revalidation **PASSES** at 16.7% SANDBOX rate
against a 30% exit-criterion gate. The 12-primitive vocabulary is
validated for the close-out of Phase 2b R5. Post-wave-8 the cohort is
executable end-to-end against the production runtime; the gate's
architectural-expressivity verdict carries through unchanged.
