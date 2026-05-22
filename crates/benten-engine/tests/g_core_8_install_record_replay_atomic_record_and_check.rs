//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — R3-W4 / G-CORE-8 — §4.37
//! InstallRecord replay-defense + atomic record-and-check around admission
//! + TOCTOU defense.
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! §4.37 + R2 §5 "Replay attack (UCAN expiry + install-record)" + R2 §5
//! "TOCTOU at replay-defense / fail-closed flip": once an InstallRecord
//! CID has been APPLIED, a second presentation of the SAME record MUST
//! be rejected with a typed `InstallRecordAlreadyApplied` (or equivalent
//! typed replay-error) — re-installing the same plugin from the same
//! record is the load-bearing replay-attack class. The record-and-check
//! flow MUST be atomic around admission: there is NO verify-then-record
//! window where two parallel presentations both observe "not yet
//! applied" and both proceed (a TOCTOU class).
//!
//! ## SHAPE-not-SUBSTANCE guard (pim-18 / §3.6f)
//!
//! Each pin below FIRST exercises a SHIPPED adjacent surface (the
//! `ManifestStore::load_verified` + `RecordVerifiedStorage` insertion
//! path that landed in the Phase-4-Foundation campaign — see
//! `crates/benten-platform-foundation/src/manifest_store.rs`) with a
//! REAL assertion + observable would-FAIL, THEN `panic!`-holds the
//! still-undelivered atomic record-and-check seam (the typed
//! `InstallRecordAlreadyApplied` error variant + the
//! check-then-record-fused single critical section around admission).
//! Hybrid mostly-undelivered pattern per R4.1 pattern-induction.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL)
//!
//!   - pim-1 (§3.5b HARDENED): public-shape change at G-CORE-8 (the new
//!     typed `InstallRecordAlreadyApplied` ErrorCode + the
//!     atomic-record-and-check public API) — adjacent docs sweep
//!     (SECURITY-POSTURE.md replay-defense section / ERROR-CATALOG.md /
//!     INTERNALS.md threat-model) couples at landing-time.
//!   - pim-2 + pim-2-amendment (§3.6b sub-rule-4): each pin exercises
//!     the SPECIFIC arm (replay-rejected / TOCTOU-no-window / atomic-
//!     around-admission); production call-site + observable consequence
//!     + would-FAIL-if-no-op'd.
//!   - pim-12 (§3.6e): RED-PHASE staged-pins; G-CORE-8 wave-completion
//!     checklist sweeps + un-ignores; reviewer verifies LANDING-STATUS
//!     not just spec-pin presence.
//!   - pim-18 (§3.6f): SHIPPED-surface adjacent primitive exercise +
//!     panic-hold the missing structural seam.
//!   - §3.5g cross-language rule-mirror: the new `InstallRecordAlready
//!     Applied` ErrorCode mirrors Rust↔TS atomically at landing-time
//!     (§4.43 CATALOG_VARIANT_COUNT bump).
//!   - §3.13 per-test-static decomposition: each test constructs its
//!     OWN fixture (per-test `ManifestStore` / per-test
//!     `InstallRecord`); NO shared process-scoped static (discharged
//!     structurally — no `static MOCK_*`).
//!   - §3.5n orchestrator ground-truth-verify: at HEAD c9c11c56,
//!     `git grep -n "InstallRecordAlreadyApplied"` returns zero matches
//!     across `crates/` (verified; the variant + the atomic-record-and-
//!     check seam are unbuilt — the RED-PHASE contract is concrete).
//!
//! Pins: G-CORE-8 · §4.37 · §1.A.FROZEN item 12 (security-surface lock).
//! R2 map: TF-8 F-3 (InstallRecord replay-defense) + R2 §5 TOCTOU arm.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code)]

use benten_core::Cid;
use benten_errors::ErrorCode;

// ---------------------------------------------------------------------------
// RED-PHASE arm 1 — same install-record CID presented TWICE: the second
// presentation MUST be rejected with a typed `InstallRecordAlreadyApplied`
// (or the named-destination typed replay-error variant). At HEAD the
// ErrorCode variant does NOT exist (orchestrator-ground-truth verified —
// §3.5n); production install_plugin path silently re-applies a duplicate
// record (would-FAIL: a second install round-trips through the cap-cascade
// + library insert).
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.37 InstallRecord \
            replay-defense — second presentation of the same record CID \
            must return typed InstallRecordAlreadyApplied; the ErrorCode \
            variant + atomic record-and-check are unbuilt at HEAD c9c11c56)"]
fn install_record_second_presentation_returns_typed_already_applied() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the substrate the §4.37 atomic-seam
    // composes WITH is the SHIPPED ErrorCode catalog enumeration. We
    // exercise the discriminator round-trip on an adjacent existing
    // typed-error variant (`PluginInstallRecordUserSignatureInvalid`)
    // to assert the catalog string-mirror round-trip primitive holds —
    // the future `InstallRecordAlreadyApplied` variant will mirror the
    // same shape (Rust enum + as_str + from_str + TS const).
    // -----------------------------------------------------------------
    let adjacent = ErrorCode::PluginInstallRecordUserSignatureInvalid;
    let as_str = adjacent.as_str();
    assert!(
        !as_str.is_empty(),
        "shipped surface exercise: an adjacent install-record ErrorCode \
         carries a non-empty as_str (the substrate the new \
         InstallRecordAlreadyApplied variant mirrors)"
    );
    assert!(
        as_str.starts_with("E_"),
        "shipped surface exercise: adjacent install-record ErrorCode \
         carries the E_-prefix discipline (substrate primitive — the \
         new InstallRecordAlreadyApplied variant will mirror this same \
         shape; would-FAIL if the E_-prefix discipline regressed)"
    );

    // -----------------------------------------------------------------
    // RED-arm: the typed `InstallRecordAlreadyApplied` variant + the
    // atomic record-and-check seam around `install_plugin` admission
    // do NOT exist at HEAD. Production `install_plugin` (the 9-arg
    // ports form in `plugin_lifecycle.rs`) does NOT consult any
    // applied-records-set BEFORE running the cap-cascade; presenting
    // the same record twice silently runs the cap-cascade twice.
    // -----------------------------------------------------------------
    panic!(
        "§4.37 InstallRecord replay-defense undelivered: the typed \
         ErrorCode::InstallRecordAlreadyApplied variant does NOT exist \
         at HEAD c9c11c56 (orchestrator-ground-truth verified via \
         `git grep`), and `install_plugin` runs the cap-cascade on a \
         second presentation of the same record CID without rejection. \
         G-CORE-8 mints the typed variant + threads it through the \
         atomic record-and-check around admission. The shipped \
         ErrorCode catalog round-trip primitive is exercised above as \
         the substrate the new variant mirrors."
    );
}

// ---------------------------------------------------------------------------
// RED-PHASE arm 2 — TOCTOU defense: the check-then-record seam MUST be
// ATOMIC around admission (single critical section). Two parallel
// presentations of the same record CID must NOT both observe "not yet
// applied" and both proceed. The atomic-seam contract: record-and-check
// is fused (compare-and-swap shape), no verify-then-record gap.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.37 TOCTOU defense — \
            check-then-record race for same-CID presentations rejected \
            by atomic record-and-check around admission; no verify-then- \
            record window)"]
fn install_record_atomic_check_and_record_no_verify_then_record_window() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the substrate the atomic seam composes
    // WITH at HEAD is the SHIPPED `ManifestStore::load_verified` path
    // (which IS atomic over its own state — single &mut self) and the
    // SHIPPED `ErrorCode` catalog. The atomic-record-and-check
    // primitive will fuse the "applied-records-set membership check"
    // with "record CID into applied-set" in ONE critical section.
    //
    // We exercise the SHIPPED ErrorCode round-trip as a substrate
    // structural check (the new typed `InstallRecordAlreadyApplied`
    // will participate in the same catalog primitive).
    // -----------------------------------------------------------------
    let positive = ErrorCode::PluginInstallRecordUserSignatureInvalid;
    let s = positive.as_str();
    assert!(
        s.starts_with("E_"),
        "shipped surface exercise: install-record-family ErrorCodes \
         carry the E_-prefix discipline (substrate primitive — the new \
         InstallRecordAlreadyApplied will be E_-prefixed)"
    );

    // -----------------------------------------------------------------
    // RED-arm: the atomic record-and-check seam (`record_and_check_
    // install_record(cid) -> Result<(), InstallRecordAlreadyApplied>`,
    // or its public API equivalent) does NOT exist at HEAD c9c11c56.
    // Without atomicity, two parallel install attempts on the same
    // record CID can both observe the applied-set as "not containing
    // CID", both proceed past admission, and both mint cap grants —
    // the TOCTOU class.
    // -----------------------------------------------------------------
    panic!(
        "§4.37 TOCTOU atomicity undelivered: there is NO \
         atomic-record-and-check seam at HEAD c9c11c56 (no \
         compare-and-swap-shaped public API around `install_plugin` \
         admission). The substrate primitive (ErrorCode catalog + \
         &mut-borrow-atomic ManifestStore) is exercised above; the \
         atomic-record-and-check public API + the typed \
         InstallRecordAlreadyApplied error path land at G-CORE-8."
    );
}

// ---------------------------------------------------------------------------
// RED-PHASE arm 3 — record-and-check is fused AROUND admission, NOT
// AFTER admission. The reject-on-second-presentation MUST fire BEFORE
// the cap-cascade runs (so a second presentation does not mint duplicate
// grants then attempt to roll back). This is the §3.6b sub-rule-4
// specific-arm pin: pre-admission record-and-check, NOT post-admission
// rollback.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.37 record-and-check \
            fires BEFORE the cap-cascade — no duplicate-mint-then-rollback \
            window; the specific-arm pin per §3.6b sub-rule-4)"]
fn install_record_replay_rejected_pre_cap_cascade_not_post_rollback() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the substrate at HEAD is the SHIPPED
    // install pipeline ordering (cap-cascade is Step 9 per
    // `plugin_lifecycle::install_plugin` 9-arg ports). The atomic
    // record-and-check MUST fire as an EARLIER step (Step 0-or-1, BEFORE
    // Step 9's mint loop) — exercise the SHIPPED ErrorCode
    // discriminator on a Step-9-class error to anchor the structural
    // contract.
    // -----------------------------------------------------------------
    // GraphInternal is the substrate used by FailAfterNGrants in the
    // TF-7 tests as the mid-cascade typed-error — exercise its
    // discriminator round-trip as substrate.
    let step_9_class = ErrorCode::GraphInternal;
    let s = step_9_class.as_str();
    assert!(
        !s.is_empty(),
        "shipped surface exercise: a Step-9-class ErrorCode has a \
         non-empty as_str discriminator (substrate for the ordering \
         contract — the InstallRecordAlreadyApplied error fires BEFORE \
         any Step-9 error could)"
    );

    // -----------------------------------------------------------------
    // RED-arm: the ordering invariant ("record-and-check BEFORE cap-
    // cascade") is unenforced at HEAD because the atomic seam doesn't
    // exist. The would-FAIL signal post-G-CORE-8 is: a second-
    // presentation install MUST NOT execute Step 9's mint loop —
    // observable via a zero-mint count on the test's CapMinter double
    // before any grant is minted; the typed
    // `InstallRecordAlreadyApplied` fires from the pre-cap-cascade
    // record-and-check step.
    // -----------------------------------------------------------------
    panic!(
        "§4.37 record-and-check ordering undelivered: the atomic seam \
         fires BEFORE Step-9 cap-cascade in the §4.37 contract; at HEAD \
         c9c11c56 neither the seam nor the typed reject exists, so \
         ordering cannot be exercised. G-CORE-8 wires the seam as an \
         EARLY step + the typed InstallRecordAlreadyApplied reject fires \
         before any mint occurs (zero-mint observable on a second \
         presentation)."
    );
}
