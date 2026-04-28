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

## Sign-off

Phase 2b paper-prototype revalidation **PASSES** at 16.7% SANDBOX rate
against a 30% exit-criterion gate. The 12-primitive vocabulary is
validated for the close-out of Phase 2b R5.
