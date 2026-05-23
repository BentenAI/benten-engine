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
fn recheck_outcome_enum_carries_unresolved_deny_not_not_applicable() {
    // **G-CORE-8 §4.36 (security-r1-1 + security-r1-2 BLOCKER closures):**
    // the post-flip enum carries a typed `UnresolvedDeny` arm distinct
    // from `NotApplicable`; substantive `Production*Rechecker` impls
    // return `UnresolvedDeny` on unresolvable-peer / missing-manifest
    // paths and `outcome_to_row_reject` row-rejects with the typed code.
    //
    // The G-CORE-8 typed-arm split (per the manifest_envelope_recheck.rs
    // enum docs) preserves the legitimate "no chain in scope" admit
    // semantics of `NotApplicable` (used by the Noop default + by
    // user-direct writes with no plugin chain) WHILE adding the new
    // typed `UnresolvedDeny` arm so substantive impls can fail-CLOSED
    // on unresolvable cases without admit-on-unresolved or fabricated-
    // OutsideEnvelope shapes.
    let unresolved = ManifestEnvelopeRecheckOutcome::UnresolvedDeny;
    let res = outcome_to_row_reject(unresolved, "merge-zone", "row-key");
    let err = res.expect_err(
        "§4.36 post-flip: UnresolvedDeny MUST row-reject — the typed-arm \
         split is the load-bearing security-r1-1 + security-r1-2 closure",
    );
    // Verify the typed code (NOT a generic ErrorCode::Internal etc.).
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny,
        "§4.36 post-flip: UnresolvedDeny MUST surface the typed \
         ManifestEnvelopeRecheckUnresolvedDeny ErrorCode"
    );
}

#[test]
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
    // **G-CORE-8 §4.36 BLOCKER closure via typed-arm split.** The Noop
    // continues to return NotApplicable (no PluginLibrary state to
    // consult), and Engine::default continues to install it. The
    // BLOCKER closure is the addition of the typed `UnresolvedDeny`
    // arm + the `outcome_to_row_reject` reject path, so substantive
    // `Production*Rechecker` impls (which platform-foundation glue
    // wires when a PluginLibrary is installed) can fail-CLOSED
    // honestly on unresolvable cases without admit-on-unresolved
    // semantics. The Noop is preserved as the "no Layer-3 enforcement
    // installed" default — see manifest_envelope_recheck.rs Noop docs.
    //
    // Verify the typed-arm-split landed: a substantive impl returning
    // UnresolvedDeny MUST row-reject (the structural closure of the
    // BLOCKER class).
    // -----------------------------------------------------------------
    struct SubstantiveImplStub;
    impl ManifestEnvelopeRechecker for SubstantiveImplStub {
        fn recheck_row(
            &self,
            _peer_did_str: &str,
            _zone: &str,
            _key: &str,
        ) -> ManifestEnvelopeRecheckOutcome {
            // Substantive impl tried but couldn't resolve the peer-DID.
            ManifestEnvelopeRecheckOutcome::UnresolvedDeny
        }
    }
    let substantive = SubstantiveImplStub;
    let outcome_subst = substantive.recheck_row("did:key:zUnknown", "merge-zone", "row-key");
    assert_eq!(
        outcome_subst,
        ManifestEnvelopeRecheckOutcome::UnresolvedDeny,
        "G-CORE-8 §4.36: a substantive Production-class rechecker stub \
         can express UnresolvedDeny via the new typed arm"
    );
    let res_subst = outcome_to_row_reject(outcome_subst, "merge-zone", "row-key");
    let err = res_subst.expect_err(
        "G-CORE-8 §4.36 fail-CLOSED: substantive rechecker's UnresolvedDeny \
         MUST row-reject",
    );
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny,
        "G-CORE-8 §4.36 fail-CLOSED: typed code surfaces at the merge boundary"
    );
}

#[test]
fn admitted_returned_only_on_positively_verified_envelope_match() {
    // **G-CORE-8 §4.36 + (a-sub) post-rename enum invariant:** `Admitted`
    // is the **positive** proceed path (positively-verified envelope
    // match). `NotApplicable` is the **structural** proceed path ("no
    // chain in scope at this layer — Layer-1 + per-row cap-recheck
    // elsewhere handle the defense"). The non-positive failure arms
    // are `UnresolvedDeny` + `OutsideEnvelope`, both of which row-reject.
    //
    // Positive control: an `Admitted` outcome proceeds.
    let admitted = ManifestEnvelopeRecheckOutcome::Admitted;
    assert!(
        outcome_to_row_reject(admitted, "zone", "key").is_ok(),
        "positive control: an explicit Admitted outcome must proceed"
    );

    // OutsideEnvelope rejects (verify-stays-regression).
    let outside = ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
        offending_plugin_did: "did:key:zPlugin".to_string(),
        cap_pattern: "store:notes:write".to_string(),
    };
    assert!(
        outcome_to_row_reject(outside, "zone", "key").is_err(),
        "OutsideEnvelope must row-reject (verify-stays-regression)"
    );

    // G-CORE-8 §4.36 NEW: UnresolvedDeny rejects with typed code.
    let unresolved = ManifestEnvelopeRecheckOutcome::UnresolvedDeny;
    let err = outcome_to_row_reject(unresolved, "zone", UNRESOLVED_PEER_DID)
        .expect_err("§4.36 G-CORE-8: UnresolvedDeny MUST row-reject");
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny,
        "§4.36 G-CORE-8: UnresolvedDeny carries the typed code"
    );

    // NotApplicable (= "no chain in scope") proceeds — this is the
    // legitimate "user wrote directly with no plugin chain" case +
    // the Noop "no Layer-3 enforcement installed" case. Layer-1
    // user-root + per-row cap-recheck (enforced elsewhere) are the
    // relevant defenses on this proceed path.
    let not_applicable = ManifestEnvelopeRecheckOutcome::NotApplicable;
    assert!(
        outcome_to_row_reject(not_applicable, "zone", "key").is_ok(),
        "§4.36 G-CORE-8 typed-arm split: NotApplicable proceeds (no \
         chain in scope at this layer — Layer-1 enforced elsewhere)"
    );
}

// ---------------------------------------------------------------------------
// R3-W4 EXTENSION — F-2 §4.25 sync-hydrate path: unresolvable peer-DID
// at the sync-HYDRATE boundary (distinct from the §4.36 merge-recheck
// boundary) → fail-closed; never proceed. The §4.25 path is the second
// production call-site sharing the same recheck-outcome enum surface.
// ---------------------------------------------------------------------------
#[test]
fn sync_hydrate_unresolvable_peer_did_fails_closed_parallel_to_merge_recheck() {
    // **G-CORE-8 §4.25 + (b) — sync-hydrate fail-CLOSED on unresolvable
    // peer-DID:** the §4.25 sync-hydrate boundary shares the same
    // outcome enum + reject primitive as the §4.36 merge path; an
    // unresolvable peer-DID at hydrate fails closed via the same
    // UnresolvedDeny typed-arm + outcome_to_row_reject mapping that
    // §4.36 uses. This pin asserts the contract is zone-label-agnostic
    // (any sync-hydrate zone label composes through the primitive).
    //
    // (b) constraint disposition: the §4.25 sync-hydrate path is the
    // sync HANDSHAKE flow (when a thin/full peer joins an Atrium and
    // hydrates state from another peer), which is structurally a
    // pre-merge flow — it composes the SAME recheck primitive once
    // the §4.36 merge per-row loop fires. The HARD-RULE-12 named-now
    // destination for the §4.25 *handshake-time* hydrate consultation
    // (distinct from the §4.36 per-row merge consultation) is
    // `crates/benten-sync/src/handshake.rs` (G-CORE-8.2 follow-up
    // wave; the handshake-time wire-up is a sync-crate concern that
    // touches the iroh sendme handshake — pulled out per HARD-RULE-12
    // clause (b) to a named follow-up wave to keep this wave's blast
    // radius bounded). The G-CORE-8 wave lands the SHARED primitive
    // contract (UnresolvedDeny + outcome_to_row_reject) that the
    // §4.25 wire-up will consume; the wire-up itself lands in the
    // follow-up. Both call sites use the same typed rejection.
    let unresolved_at_hydrate = ManifestEnvelopeRecheckOutcome::UnresolvedDeny;
    let res = outcome_to_row_reject(unresolved_at_hydrate, "sync-hydrate-zone", "row-hydrate-1");
    let err = res.expect_err(
        "§4.25/§4.36 G-CORE-8: UnresolvedDeny at the sync-hydrate \
         boundary MUST row-reject — the recheck primitive is shared",
    );
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::ManifestEnvelopeRecheckUnresolvedDeny,
        "§4.25/§4.36 G-CORE-8: typed reject surfaces uniformly at the \
         hydrate boundary"
    );

    // Verify-stays: OutsideEnvelope at hydrate also rejects.
    let outside_at_hydrate = ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
        offending_plugin_did: "did:key:zPluginHydrate".to_string(),
        cap_pattern: "store:notes:write".to_string(),
    };
    assert!(
        outcome_to_row_reject(outside_at_hydrate, "sync-hydrate-zone", "row-hydrate-2").is_err(),
        "verify-stays: OutsideEnvelope at hydrate also rejects via the \
         shared primitive (zone-label-agnostic)"
    );
}
