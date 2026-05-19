//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 — §4.36 fail-CLOSED flip
//! (the flagship security-r1-1 BLOCKER closure + security-r1-2
//! post-rename enum invariant).
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! This file pins the STILL-UNDELIVERED §4.36 fail-CLOSED flip. At
//! SYNCED HEAD `ed03729a`, #1294 landed the Layer-3 `SharesPolicyResolver`
//! port + closure-pin tests for the *delegate_capability* path, but the
//! `apply_atrium_merge` manifest-envelope recheck path is STILL the
//! pre-flip opt-in footgun:
//!
//! - `ManifestEnvelopeRecheckOutcome` still has the `NotApplicable`
//!   variant (NOT renamed to `UnresolvedDeny`) —
//!   `crates/benten-engine/src/manifest_envelope_recheck.rs:37-58`.
//! - `NoopManifestEnvelopeRechecker::recheck_row` returns
//!   `NotApplicable` for every call (admit-everything) — and that is
//!   what `Engine::default` installs (`Some(Arc::new(Noop))` is
//!   structurally wired but operationally inert).
//! - `outcome_to_row_reject` maps
//!   `NotApplicable | Admitted => Ok(())` — i.e. it ADMITS on the
//!   unresolved/sentinel outcome (the security-r1-2 BLOCKER:
//!   `manifest_envelope_recheck.rs:124-144`).
//!
//! ## SHAPE-not-SUBSTANCE guard (R2 §4-A — pim-18, LITERAL)
//!
//! This file MUST NOT pass merely because "a `ManifestEnvelopeRechecker`
//! type exists" or "the recheck-path is structurally wired". It asserts
//! the SUBSTANTIVE fail-CLOSED behavior:
//!   (1) a `ProductionManifestEnvelopeRechecker` is auto-wired into the
//!       DEFAULT builder (NOT the opt-in Noop footgun);
//!   (2) an unresolvable/sentinel peer-DID at recheck → `UnresolvedDeny`
//!       + row-reject (would-FAIL if `outcome_to_row_reject` admits-on-
//!       unresolved or maps a non-positive outcome to `Ok(())`);
//!   (3) `Admitted` is returned ONLY on a positively-verified
//!       envelope/chain match (the post-rename enum invariant).
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL — do not
//! collapse to a cross-reference; reproduced per dispatch-conventions
//! §3.6g MANDATORY for next-phase R3 briefs):
//!   - pim-1 (§3.5b): public-shape change ⇒ sweep adjacent docs before
//!     push (the enum rename + default-builder change are public-shape;
//!     INTERNALS.md / SECURITY-POSTURE.md / threat-model couple).
//!   - pim-2 + pim-2-amendment (§3.6b sub-rule-4): the closure pin
//!     exercises the SPECIFIC arm (`outcome_to_row_reject` on the
//!     unresolved outcome), production call-site + observable
//!     consequence + would-FAIL-if-no-op'd — NOT a sentinel.
//!   - pim-12 (§3.6e): this is a RED-PHASE staged-pin; the G-CORE-8
//!     wave-completion checklist MUST sweep + un-ignore it (reviewer
//!     verifies landing-status, not just spec-pin presence).
//!   - pim-18 (§3.6f): production call-site enumerated + body-of-test
//!     substantive (the §4-A SHAPE-trap guard above).
//!   - §3.5g cross-language rule-mirror: the recheck-outcome enum is a
//!     §1.A.FROZEN item-12 frozen surface; the rename couples a TS/JS
//!     mirror at G-CORE-9 freeze.
//!   - §3.13 per-test-static decomposition: this module uses NO shared
//!     process-scoped static (each test constructs its own fixtures);
//!     the §3.13 obligation is discharged structurally (no `static
//!     MOCK_*`) — recorded explicitly per the on-surface flag.
//!   - §3.11 checkpoint-pre-flight recovery: TF-8 is the largest cross-
//!     crate family; on agent-kill, resume INTO the same worktree.
//!
//! Pins: G-CORE-8 · C8 · §1.A.FROZEN item 12 (security-surface public-
//! shape lock + post-rename enum invariant). R2 map: TF-8 RED-arm (1)+(2).

#![allow(unused_imports, dead_code)]

use benten_engine::manifest_envelope_recheck::{
    ManifestEnvelopeRecheckOutcome, ManifestEnvelopeRechecker, NoopManifestEnvelopeRechecker,
    outcome_to_row_reject,
};

/// An unresolvable / sentinel peer-DID used to drive the fail-CLOSED
/// arm. Post-flip, a recheck against an unresolvable peer-DID MUST
/// yield the `UnresolvedDeny` outcome (NOT `NotApplicable`/admit).
const UNRESOLVED_PEER_DID: &str = "<unresolved-peer>";

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.36 fail-CLOSED flip — \
            NotApplicable→UnresolvedDeny rename)"]
fn recheck_outcome_enum_carries_unresolved_deny_not_not_applicable() {
    // POST-FLIP INVARIANT (security-r1-2): the sentinel/unresolved
    // outcome variant is named `UnresolvedDeny` and maps to a row
    // REJECT — NOT `NotApplicable` which currently maps to admit.
    //
    // This is written against the RENAMED enum. Until G-CORE-8 lands
    // the rename, `ManifestEnvelopeRecheckOutcome::UnresolvedDeny`
    // does not exist and this file fails to compile / the test fails
    // — which is the RED-phase contract.
    //
    // The assertion (when un-ignored, post-rename) is:
    //   let o = ManifestEnvelopeRecheckOutcome::UnresolvedDeny;
    //   assert!(outcome_to_row_reject(o, "zone", "key").is_err());
    //
    // We DO NOT reference the post-rename variant by name here
    // (compile-RED would block the whole crate's test build); instead
    // we pin the BEHAVIOR the rename must deliver via the existing
    // surface, asserting the CURRENT admit-on-unresolved behavior is
    // GONE post-flip.
    //
    // CURRENT (pre-flip) behavior — this is what must change:
    let current = ManifestEnvelopeRecheckOutcome::NotApplicable;
    let res = outcome_to_row_reject(current, "merge-zone", "row-key");
    // Pre-flip this is Ok(()) (admit-on-unresolved — the BLOCKER).
    // Post-flip the unresolved/sentinel outcome MUST be a row-reject.
    // RED-phase: assert the post-flip contract (currently FAILS).
    assert!(
        res.is_err(),
        "§4.36 fail-CLOSED flip undelivered: the unresolved/sentinel \
         recheck outcome still maps to Ok(()) (admit-on-unresolved — \
         security-r1-2 BLOCKER). Post-flip it MUST row-reject."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.36 — production \
            rechecker auto-wired into DEFAULT builder, not Noop)"]
fn default_builder_installs_substantive_production_rechecker_not_noop() {
    // SHAPE-trap guard (R2 §4-A): this MUST assert the DEFAULT engine
    // builder installs a SUBSTANTIVE `ProductionManifestEnvelopeRechecker`
    // — NOT that "a rechecker type exists" and NOT the Noop footgun.
    //
    // Post-G-CORE-8 the assertion is: build an `Engine` via the default
    // builder; drive `apply_atrium_merge` with a row whose peer-DID is
    // a plugin-principal whose manifest `shares` policy DENIES the cap;
    // assert the row is REJECTED with
    // `ErrorCode::PluginDelegationOutsideManifestEnvelope` WITHOUT the
    // test ever calling `Engine::set_manifest_envelope_rechecker`.
    //
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (R4.1 fix-pass per pim-18 §3.6f — L3
    // finding-7): exercise the SHIPPED `NoopManifestEnvelopeRechecker`
    // directly to PROVE its admit-everything semantics at HEAD. This
    // is the would-FAIL signal: the SHIPPED Noop returns NotApplicable
    // for every input AND `outcome_to_row_reject` maps NotApplicable
    // to Ok(()) — so the engine default at HEAD admits every cross-
    // plugin write the substantive rechecker would deny. Real
    // assertions on real input + observable would-FAIL consequence
    // (post-G-CORE-8 the default builder installs a substantive
    // rechecker; this primitive-level Noop call MUST keep returning
    // NotApplicable for the dispatch contract, but the DEFAULT-wired
    // implementation observed by Engine::default MUST no longer be
    // this Noop instance).
    // -----------------------------------------------------------------
    let noop = NoopManifestEnvelopeRechecker;

    // The SHIPPED Noop's recheck_row returns NotApplicable for every
    // input — the admit-everything substrate.
    let outcome_1 = noop.recheck_row("did:key:zHostilePlugin", "merge-zone-1", "row-key-alpha");
    assert_eq!(
        outcome_1,
        ManifestEnvelopeRecheckOutcome::NotApplicable,
        "shipped surface exercise: the Noop rechecker returns \
         NotApplicable on any input (would-FAIL if the Noop regressed \
         to a different default — but the structural would-FAIL is \
         that this admit-everything path is precisely what the default \
         builder still installs, per `Engine::default` wiring)."
    );
    let outcome_2 = noop.recheck_row(
        "did:key:zOtherHostilePlugin",
        "merge-zone-2",
        "row-key-beta",
    );
    assert_eq!(
        outcome_2,
        ManifestEnvelopeRecheckOutcome::NotApplicable,
        "shipped surface exercise: a SECOND distinct input also yields \
         NotApplicable — the Noop is input-agnostic admit-everything."
    );

    // Compose with `outcome_to_row_reject` (the SHIPPED helper the
    // apply_atrium_merge per-row loop calls): NotApplicable → Ok(())
    // → admit. This is the security-r1-1 BLOCKER's load-bearing
    // would-FAIL: every hostile row would-be-admitted under the
    // default-wired Noop.
    let admit = outcome_to_row_reject(outcome_1, "merge-zone-1", "row-key-alpha");
    assert!(
        admit.is_ok(),
        "shipped surface exercise: the Noop's NotApplicable composes \
         with `outcome_to_row_reject` to admit — this is the substantive \
         admit-everything behavior the default builder installs at HEAD. \
         POST-G-CORE-8: the default builder installs a SUBSTANTIVE \
         rechecker whose recheck_row would NOT return NotApplicable on \
         a hostile-plugin row (it would return OutsideEnvelope, mapping \
         to Err)."
    );

    // -----------------------------------------------------------------
    // RED-arm: `Engine::default` still installs THIS Noop instance
    // (verified at `crates/benten-engine/src/engine.rs:1840` per the
    // module doc above), so production deployments inherit the admit-
    // everything default. The G-CORE-8 wave swaps in a substantive
    // `ProductionManifestEnvelopeRechecker` that consults the
    // PluginLibrary's manifest `shares` policy + UserDidRegistry +
    // `validate_chain_with_manifest_envelope` — making this
    // primitive-level Noop call no longer the default-wired path.
    // -----------------------------------------------------------------
    panic!(
        "§4.36 production-rechecker-default undelivered: the SHIPPED \
         `NoopManifestEnvelopeRechecker` returns NotApplicable for every \
         input AND `outcome_to_row_reject` maps that to admit \
         (exercised above — the admit-everything substrate at HEAD). \
         `Engine::default` still installs `NoopManifestEnvelopeRechecker` \
         (per `engine.rs:1840`). G-CORE-8 must auto-wire a substantive \
         `ProductionManifestEnvelopeRechecker` into the DEFAULT builder \
         so production deployments get Layer-3 enforcement without an \
         explicit `set_manifest_envelope_rechecker` call \
         (security-r1-1 BLOCKER closure)."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.36 — Admitted ONLY on \
            positively-verified match; post-rename enum invariant)"]
fn admitted_returned_only_on_positively_verified_envelope_match() {
    // POST-RENAME ENUM INVARIANT (security-r1-2): `Admitted` is the
    // ONLY outcome that proceeds-without-row-reject for a plugin-
    // principal; every non-positive outcome (unresolved / no-manifest
    // / outside-envelope) MUST row-reject. There is NO admit-on-
    // ambiguity path.
    //
    // Positive control: an `Admitted` outcome proceeds (Ok).
    let admitted = ManifestEnvelopeRecheckOutcome::Admitted;
    assert!(
        outcome_to_row_reject(admitted, "zone", "key").is_ok(),
        "positive control: an explicit Admitted outcome must proceed"
    );

    // Negative control (the RED arm): the OutsideEnvelope outcome
    // row-rejects (this part already holds at HEAD — verify-stays).
    let outside = ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
        offending_plugin_did: "did:key:zPlugin".to_string(),
        cap_pattern: "store:notes:write".to_string(),
    };
    assert!(
        outcome_to_row_reject(outside, "zone", "key").is_err(),
        "OutsideEnvelope must row-reject (verify-stays-regression)"
    );

    // The RED assertion: post-flip there is NO `NotApplicable`-shaped
    // admit path. Driving the (renamed) unresolved outcome MUST
    // row-reject. Pre-flip `NotApplicable` maps to Ok(()) — FAIL.
    let unresolved_proxy = ManifestEnvelopeRecheckOutcome::NotApplicable;
    assert!(
        outcome_to_row_reject(unresolved_proxy, "zone", UNRESOLVED_PEER_DID).is_err(),
        "§4.36 post-rename invariant undelivered: a non-positive \
         (unresolved/sentinel) recheck outcome still maps to Ok(()) — \
         `Admitted` must be the ONLY proceed path."
    );
}
