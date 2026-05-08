//! Phase-3 G21-T3 Section B — paper-prototype Phase-3 revalidation
//! runtime-wired pin.
//!
//! Per the G21-T3 brief Section B + plan exit-criterion: the Phase-3
//! paper-prototype corpus (classified at
//! `.addl/phase-3/paper-prototype-phase-3-classified.md` —
//! orchestrator-local source-of-truth doc) carries the verdict
//! "Combined: 2/25 = 8.0% — PASS (gate <= 30%)". The classified doc
//! is local-only (`.addl/` is gitignored — orchestrator pipeline
//! artifact); this test encodes the per-handler verdict as data
//! (paper trail to the doc) and asserts the runtime rate <= 30.0%.
//! A future Phase-3 corpus change MUST retense both the classified
//! doc AND the in-test verdict array together (post-fix doc-coupling
//! pre-flight per pim-1 §3.5b).
//!
//! Companion to:
//! - `crates/benten-eval/tests/sandbox_rate_full_revalidation_g11_2b.rs`
//!   (Phase-2b cohort gate; consumes `docs/PAPER-PROTOTYPE-REVALIDATION.md`).
//! - `crates/benten-engine/tests/paper_prototype_revalidation_doc_present.rs`
//!   (Phase-2b doc-presence + structural-shape pin).
//!
//! pim-2 §3.6b end-to-end discipline: each verdict row references the
//! actual primitive composition the handler subgraph would use; if a
//! future Phase-3 change pushed a handler from typed-CALL to SANDBOX
//! composition (e.g. a new compute requirement that doesn't fit
//! existing primitives), the orchestrator-local classified doc would
//! retense + this test would retense in lockstep.

#![allow(clippy::unwrap_used, clippy::expect_used)]

/// Phase-3 paper-prototype handler classification — encoded from
/// `.addl/phase-3/paper-prototype-phase-3-classified.md` (local-only
/// doc; this array is the public-repo paper-trail). Each tuple is
/// `(handler_id, needs_sandbox)` per the classifier's verdict.
///
/// Phase-2b cohort (#1..#12, preserved from
/// `docs/PAPER-PROTOTYPE-REVALIDATION.md`) is encoded as constants
/// below for the combined-rate calculation. Phase-3 cohort (#13..#25)
/// is the new addition.
///
/// **Source-of-truth coupling:** if this array changes, the
/// classified doc MUST retense in the same commit (per pim-1 §3.5b
/// 4-surface discipline — the doc + this array + the dispatch-
/// conventions §3.5b symbol cite + the in-doc tally line all move
/// together).
const PHASE_3_HANDLERS: &[(&str, bool)] = &[
    // #13 atrium-join — CALL (Did::resolve typed-CALL) + CALL
    // (Handshake::initiate engine-built-in) + WRITE + RESPOND.
    ("atrium-join", false),
    // #14 atrium-sync-trigger — CALL (peer.sync) + ITERATE +
    // WRITE (Mst::apply_entries internally rehashes) + RESPOND.
    ("atrium-sync-trigger", false),
    // #15 ucan-issue — CALL (UcanBuilder::sign typed-CALL) +
    // TRANSFORM + WRITE + RESPOND.
    ("ucan-issue", false),
    // #16 ucan-validate-chain — CALL (validate_chain typed-CALL) +
    // BRANCH + RESPOND.
    ("ucan-validate-chain", false),
    // #17 ucan-revoke — WRITE + EMIT + RESPOND.
    ("ucan-revoke", false),
    // #18 did-resolve — CALL (Did::resolve typed-CALL) + RESPOND.
    ("did-resolve", false),
    // #19 did-rotate — CALL (keypair_generate typed-CALL) + CALL
    // (sign typed-CALL) + TRANSFORM + WRITE + EMIT + RESPOND.
    ("did-rotate", false),
    // #20 vc-issue — CALL (CredentialBuilder::sign typed-CALL) +
    // TRANSFORM + WRITE + RESPOND.
    ("vc-issue", false),
    // #21 vc-verify — CALL (vc_verify typed-CALL) + READ + BRANCH +
    // RESPOND.
    ("vc-verify", false),
    // #22 keypair-generate — CALL (keypair_generate typed-CALL) +
    // RESPOND.
    ("keypair-generate", false),
    // #23 keypair-from-seed — CALL (Keypair::from_seed_bytes
    // typed-CALL) + RESPOND.
    ("keypair-from-seed", false),
    // #24 view-register-anchor-prefix — WRITE + CALL + RESPOND.
    ("view-register-anchor-prefix", false),
    // #25 light-client-verify — ITERATE + CALL (blake3_hash
    // typed-CALL) + BRANCH + RESPOND.
    ("light-client-verify", false),
];

/// Phase-2b cohort SANDBOX count (preserved from
/// `docs/PAPER-PROTOTYPE-REVALIDATION.md`): 2 handlers
/// (#11 LLM summarisation + #12 image thumbnail) require SANDBOX
/// — both are arbitrary-user-compute shapes (NLP / image
/// resampling) that don't reduce to typed engine operations.
const PHASE_2B_SANDBOX_COUNT: usize = 2;
const PHASE_2B_TOTAL: usize = 12;

/// Phase-3 paper-prototype combined SANDBOX-rate gate.
///
/// Asserts the combined-cohort rate <= 30.0%. Currently
/// 2/25 = 8.0%. The 30% gate is the same Phase-2b commitment per
/// plan §1 exit-criterion #1; Phase 3 preserves it.
#[test]
fn paper_prototype_phase_3_combined_sandbox_rate_under_30_percent() {
    let phase_3_sandbox_count = PHASE_3_HANDLERS.iter().filter(|(_, s)| *s).count();
    let phase_3_total = PHASE_3_HANDLERS.len();

    let combined_sandbox = PHASE_2B_SANDBOX_COUNT + phase_3_sandbox_count;
    let combined_total = PHASE_2B_TOTAL + phase_3_total;

    let rate_pct = (combined_sandbox as f64) / (combined_total as f64) * 100.0;

    assert!(
        rate_pct <= 30.0,
        "Phase-3 combined paper-prototype SANDBOX-rate {:.1}% \
         ({}/{}) MUST be <= 30.0% per plan exit-criterion (\
         architectural-expressivity gate; CLAUDE.md baked-in \
         commitment #1: 12 primitives are sufficient). Phase-3 \
         additions: {} of {}; Phase-2b base: {} of {}. If this \
         fires, retense the classified doc + this array + flag \
         the architectural escape to Ben.",
        rate_pct,
        combined_sandbox,
        combined_total,
        phase_3_sandbox_count,
        phase_3_total,
        PHASE_2B_SANDBOX_COUNT,
        PHASE_2B_TOTAL,
    );
}

/// Phase-3 cohort handler-count pin. The classified doc enumerates
/// 13 Phase-3 handlers (#13..#25). A row addition / removal without
/// retense would surface here.
#[test]
fn paper_prototype_phase_3_cohort_count_pinned_at_13() {
    assert_eq!(
        PHASE_3_HANDLERS.len(),
        13,
        "Phase-3 paper-prototype cohort MUST enumerate 13 handlers \
         (#13..#25 per the classified doc). If the corpus grew or \
         shrank, retense this constant + the classified doc \
         (.addl/phase-3/paper-prototype-phase-3-classified.md) \
         together per pim-1 §3.5b 4-surface discipline."
    );
}

/// Phase-3 verdict pin: under the typed-CALL stance (CLAUDE.md
/// baked-in #16), zero Phase-3 handlers require SANDBOX —
/// every workflow shape composes from typed engine operations
/// (CALL, READ, WRITE, BRANCH, ITERATE, EMIT, RESPOND).
///
/// The two SANDBOX cases live in Phase-2b (#11 LLM + #12 image)
/// — genuine arbitrary-user-compute shapes.
#[test]
fn paper_prototype_phase_3_zero_handlers_need_sandbox() {
    let phase_3_sandbox_count = PHASE_3_HANDLERS.iter().filter(|(_, s)| *s).count();
    assert_eq!(
        phase_3_sandbox_count, 0,
        "Phase-3 paper-prototype cohort MUST have 0 SANDBOX-requiring \
         handlers under the typed-CALL stance. Crypto / hash / DID / \
         UCAN / VC ops are typed-CALL composition (engine:typed:* \
         registry, G21-T1), not SANDBOX. If this fires, the proposed \
         change is a candidate for SANDBOX vs adding a new \
         typed-CALL op — surface for Ben's architectural ratification."
    );
}

/// Verdict integrity: every handler in the cohort has a non-empty
/// id (paper-trail discipline). Catches accidental empty rows.
#[test]
fn paper_prototype_phase_3_handler_ids_non_empty() {
    for (id, _) in PHASE_3_HANDLERS {
        assert!(
            !id.is_empty(),
            "Phase-3 cohort row MUST have a non-empty handler id"
        );
    }
}
