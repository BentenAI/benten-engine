//! Phase 2b R4-FP B-4 — paper-prototype SANDBOX rate gate (STAGED CHECK).
//!
//! TDD red-phase. Pin source: plan §1 exit-criterion #1 + D11-RESOLVED
//! (hybrid single-sample + 3-5 cohort) + arch-pre-r1-4 STAGED CHECK
//! (cheap dry-run rate prediction at G7 close; ~30 min manual
//! classification of canonical fixture vocabulary against the new
//! SANDBOX surface).
//!
//! **This is the load-bearing test of exit-criterion #1.** A high
//! G11-2b measurement would be a phase-close hard-fail with no
//! recourse; the staged check at G7 close gives ≥4-week remediation
//! runway. R2 §6 + qa-r4-08 flagged this as the absent gate.
//!
//! Cohort definition (per R2 §11.2 #3 + D11): the canonical fixture
//! vocabulary is the union of (a) Phase-1 8 primitives (READ, WRITE,
//! TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND), (b) Phase-2b
//! additions (EMIT, SUBSCRIBE, STREAM, SANDBOX), and (c) the
//! orchestrator-confirmed paper-prototype handler set. The classifier
//! marks each handler as "needs SANDBOX" or "expressible without
//! SANDBOX"; the rate is `count(needs_sandbox) / count(total)` and
//! must be ≤ 30%.
//!
//! The classification body is `todo!()` here; R5 G7-A close-out
//! (paired with G11-2b-A) populates the actual handler list +
//! per-handler classification rationale + records the rate to
//! `docs/PAPER-PROTOTYPE-REVALIDATION.md`. The companion test
//! `sandbox_rate_full_revalidation_g11_2b.rs` re-runs this check at
//! G11-2b close against the full revised vocabulary.
//!
//! Owned by R3-E surface row (CI workflow tests); test landed by
//! R4-FP B-4 fix-pass.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]
#![allow(clippy::cast_precision_loss)]

/// Single classifier result for one paper-prototype handler.
///
/// `needs_sandbox = true` means the handler can ONLY be expressed by
/// invoking the SANDBOX primitive (i.e. it requires a host-fn that
/// only WASM modules expose, e.g. arbitrary computation, format
/// conversion, regex, etc.). `false` means the handler composes from
/// the 11 non-SANDBOX primitives (READ/WRITE/TRANSFORM/BRANCH/...).
struct HandlerClassification {
    /// Handler identifier (matches the fixture vocabulary entry).
    name: &'static str,
    /// Classification verdict — true if requires SANDBOX.
    needs_sandbox: bool,
    /// Rationale (a short sentence — recorded in
    /// `docs/PAPER-PROTOTYPE-REVALIDATION.md` per D11).
    rationale: &'static str,
}

/// Returns the canonical fixture vocabulary used by the staged check.
///
/// **R5 G7-A close-out fills this in.** The list MUST cover at least
/// the orchestrator-confirmed paper-prototype handler set (per D11
/// hybrid: 1 single-sample + 3-5 cohort handlers). Until then the
/// function returns an empty slice and the test's structural assertion
/// (count > 0) fails to make the dependency explicit.
fn canonical_fixture_vocabulary() -> Vec<HandlerClassification> {
    // Cohort matches `docs/PAPER-PROTOTYPE-REVALIDATION.md` (G11-2b
    // FULL revalidation). The G7-close STAGED CHECK uses a smaller
    // single-sample + 3-cohort subset; the cohort here is the
    // single-sample anchor (`crud('post').create`) plus 3 cohort
    // handlers covering Phase-1 + Phase-2a + Phase-2b primitive
    // surface so the G7-close gate has signal without re-implementing
    // the full 12-handler classification.
    vec![
        HandlerClassification {
            name: "crud('post').create",
            needs_sandbox: false,
            rationale: "Pure storage path: read-by-key + WRITE + RESPOND.",
        },
        HandlerClassification {
            name: "payment-confirm (WAIT-signal)",
            needs_sandbox: false,
            rationale: "WAIT suspends + BRANCH on resume; no compute escape.",
        },
        HandlerClassification {
            name: "iter-batch-import",
            needs_sandbox: false,
            rationale: "ITERATE + TRANSFORM + WRITE; bounded DAG.",
        },
        HandlerClassification {
            name: "summarize-doc-with-llm",
            needs_sandbox: true,
            rationale: "Text summarisation needs SANDBOX (TRANSFORM grammar has no NLP/regex).",
        },
    ]
}

/// `sandbox_rate_under_30_percent` — plan §1 exit-criterion #1 STAGED
/// CHECK at G7 close.
///
/// Asserts the SANDBOX rate against the canonical fixture vocabulary
/// is ≤ 30%. A higher rate means the 11 non-SANDBOX primitives don't
/// cover real-workload expressivity, which would invalidate the
/// non-Turing-complete-DAG architecture decision (CLAUDE.md baked-in #4).
#[test]
fn sandbox_rate_under_30_percent() {
    let vocab = canonical_fixture_vocabulary();

    assert!(
        !vocab.is_empty(),
        "canonical fixture vocabulary MUST be non-empty after G7 close \
         (arch-pre-r1-4 staged check; D11 hybrid single-sample + 3-5 cohort)"
    );

    let total = vocab.len();
    let needs_sandbox = vocab.iter().filter(|h| h.needs_sandbox).count();
    let rate = needs_sandbox as f64 / total as f64;

    assert!(
        rate <= 0.30,
        "STAGED CHECK FAIL: paper-prototype SANDBOX rate {:.1}% > 30% \
         exit-criterion gate ({}/{} handlers need SANDBOX). G7-close \
         remediation runway in effect; G11-2b-A FULL revalidation will \
         re-fail unless the primitive surface is re-evaluated. \
         Per-handler verdicts: {:?}",
        rate * 100.0,
        needs_sandbox,
        total,
        vocab
            .iter()
            .map(|h| (h.name, h.needs_sandbox, h.rationale))
            .collect::<Vec<_>>()
    );
}
