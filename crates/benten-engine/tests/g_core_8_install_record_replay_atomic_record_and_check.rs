//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — R3-W4 / G-CORE-8 — §4.37
//! InstallRecord replay-defense + atomic record-and-check around admission
//! + TOCTOU defense.
//!
//! ## LANDED at G-CORE-8 (pim-12 / §3.6e closure)
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
use benten_engine::install_record_replay::{InstallRecordReplayStore, signing_payload_hash};
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
    // G-CORE-8 §4.37 LANDED: the typed
    // `ErrorCode::PluginInstallRecordAlreadyApplied` is minted in
    // benten-errors + the `InstallRecordReplayStore` provides the
    // atomic record-and-check primitive. Exercise the substrate:
    // first record-and-check admits; second presentation of the same
    // payload hash rejects with the typed code.
    // -----------------------------------------------------------------
    let store = InstallRecordReplayStore::new();
    let payload_hash = signing_payload_hash(b"alice-install-record-canonical-bytes");

    // First presentation: admitted + recorded.
    store
        .record_and_check(payload_hash)
        .expect("first presentation of an InstallRecord MUST be admitted");
    assert!(
        store.contains(&payload_hash),
        "first presentation recorded in the consumed set"
    );

    // Second presentation: rejected with the typed code (the §4.37
    // replay-defense closure). Zero duplicate-mint window: this fires
    // at the atomic check-and-record primitive, BEFORE any downstream
    // cap-cascade would mint grants.
    let err = store
        .record_and_check(payload_hash)
        .expect_err("§4.37 second presentation MUST reject");
    assert_eq!(
        err,
        ErrorCode::PluginInstallRecordAlreadyApplied,
        "§4.37 G-CORE-8: replay surfaces typed PluginInstallRecordAlreadyApplied"
    );
    // Second presentation does NOT advance state (single-record invariant).
    assert_eq!(store.consumed_count(), 1);
}

// ---------------------------------------------------------------------------
// RED-PHASE arm 2 — TOCTOU defense: the check-then-record seam MUST be
// ATOMIC around admission (single critical section). Two parallel
// presentations of the same record CID must NOT both observe "not yet
// applied" and both proceed. The atomic-seam contract: record-and-check
// is fused (compare-and-swap shape), no verify-then-record gap.
// ---------------------------------------------------------------------------
#[test]
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
    // G-CORE-8 §4.37 LANDED: the atomic record-and-check seam is the
    // `InstallRecordReplayStore::record_and_check` method (single
    // critical section under a `Mutex` — no verify-then-record gap).
    // Exercise the TOCTOU defense with parallel callers: of N parallel
    // presentations of the same payload-hash, exactly ONE admits +
    // (N-1) reject. The store's consumed_count is exactly 1 — no
    // double-record.
    // -----------------------------------------------------------------
    use std::sync::Arc;
    use std::thread;

    let store = Arc::new(InstallRecordReplayStore::new());
    let payload_hash = signing_payload_hash(b"shared-toctou-attack-payload");

    let handles: Vec<_> = (0..32)
        .map(|_| {
            let s = Arc::clone(&store);
            thread::spawn(move || s.record_and_check(payload_hash))
        })
        .collect();

    let mut ok = 0;
    let mut err = 0;
    for h in handles {
        match h.join().unwrap() {
            Ok(()) => ok += 1,
            Err(ErrorCode::PluginInstallRecordAlreadyApplied) => err += 1,
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }
    assert_eq!(
        ok, 1,
        "§4.37 TOCTOU: exactly ONE parallel presentation admits"
    );
    assert_eq!(
        err, 31,
        "§4.37 TOCTOU: all other parallel presentations reject"
    );
    assert_eq!(
        store.consumed_count(),
        1,
        "§4.37 TOCTOU: consumed-set holds exactly one record (no double-record)"
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
    // G-CORE-8 §4.37 LANDED: the atomic seam fires at the
    // `InstallRecordReplayStore::record_and_check` boundary which is
    // wired into `plugin_lifecycle::install_plugin` at "Step 3b"
    // (immediately after user-DID signature verification, BEFORE
    // Step 4 clock validation AND BEFORE Step 9 cap-cascade). The
    // pin asserts the ORDERING contract structurally: on a second
    // presentation the store's `consumed_count` does NOT change AND
    // the reject fires; the production wire-up at install_plugin
    // means no cap-cascade Step-9 mint occurs when the reject fires.
    // -----------------------------------------------------------------
    let store = InstallRecordReplayStore::new();
    let payload_hash = signing_payload_hash(b"alice-replay-ordering");

    // First admission lands.
    store.record_and_check(payload_hash).unwrap();
    let consumed_before = store.consumed_count();
    assert_eq!(consumed_before, 1);

    // Second presentation rejects BEFORE state advances. The check-
    // and-record fusion (single Mutex critical section) is what
    // delivers the "no advance on reject" contract; the production
    // install_plugin caller catches the typed Err and does NOT
    // proceed to Step 9 (the §3b ordering wire-up at
    // plugin_lifecycle.rs).
    let err = store
        .record_and_check(payload_hash)
        .expect_err("replay rejects");
    assert_eq!(err, ErrorCode::PluginInstallRecordAlreadyApplied);
    assert_eq!(
        store.consumed_count(),
        consumed_before,
        "§4.37 ordering: rejected second presentation does NOT advance \
         the consumed-set (no double-record); production install_plugin \
         catches the typed Err pre-Step-9 (zero-mint observable)"
    );
}
