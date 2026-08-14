//! R6-R1 fold-in (F-02) production-arm test pin —
//! `EngineCapsHandle::delegate_capability` REJECTS an authority-widening
//! `attenuated_caps` entry with the typed `ErrorCode::CapAttenuation`,
//! using the SAME segment-boundary-safe subsume relation as the UCAN
//! attenuation walk (`benten_id::ucan::caps_match_or_subsume`, R15/F-01).
//!
//! Closes the F-02 fork in V1-FROZEN-INTERFACE-DEFERRED.md Row D-87: the
//! frozen public method's docstring promises "narrowed-or-identical", but
//! pre-fold-in `delegate_capability` set `effective_scope =
//! attenuated_caps[0]` with NO subset check (deferring "full attenuation
//! semantics … alongside G27-D"). This is the STRUCTURAL
//! "cannot-delegate-more-than-you-hold" invariant — it always holds,
//! independent of the (deferred) manifest-`shares` policy.
//!
//! ## Test shape (per pim-18 / §3.6f SUBSTANTIVE-arm-not-SHAPE)
//!
//! The engine uses `NoAuthBackend` (the attenuation-subset check is
//! policy-independent — it fires on the SCOPE relation, before any write).
//!
//! **Would-FAIL-on-revert:** delete the Step-2c attenuation-subset guard
//! in `engine_caps.rs::delegate_capability` (or weaken `scope_subsumes`'s
//! segment-boundary guard to a bare `starts_with`) → the sibling-prefix
//! and broader-scope delegations ADMIT → the `expect_err` assertions fail.

use benten_engine::{Engine, EngineError};
use benten_errors::ErrorCode;

const TARGET_PLUGIN_DID: &str = "did:key:z6MkTargetPluginF02FoldinHarnessBBBBBBBBBBBBBB";

/// Build a real Engine (NoAuthBackend) + mint a source grant whose scope
/// is `/zone/posts:write`. Returns (engine, tempdir, source_grant_cid).
fn engine_with_source_grant(scope: &str) -> (Engine, tempfile::TempDir, benten_core::Cid) {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let path = tempdir.path().join("f02-foldin.redb");
    let engine = benten_engine::EngineBuilder::new()
        .capability_policy(Box::new(benten_caps::NoAuthBackend::new()))
        .open(&path)
        .expect("engine opens with NoAuthBackend");
    let source = engine
        .caps()
        .grant_capability(
            "did:key:z6MkSourceF02FoldinHarnessAAAAAAAAAAAAAAAAAA",
            scope,
        )
        .expect("mint source grant");
    (engine, tempdir, source)
}

/// Assert that a delegation carrying `attenuated` is rejected with the
/// typed `CapAttenuation` code.
fn assert_widening_rejected(source_scope: &str, attenuated: &str) {
    let (engine, _td, source) = engine_with_source_grant(source_scope);
    let err = match delegate(&engine, &source, &[attenuated.to_string()]) {
        Ok(cid) => panic!(
            "LOAD-BEARING: delegating `{attenuated}` (which WIDENS authority beyond source \
             scope `{source_scope}`) MUST be rejected — got Ok({}) instead; the Step-2c \
             attenuation-subset guard in delegate_capability was reverted (F-02 \
             authority-widening regression)",
            cid.to_base32()
        ),
        Err(e) => e,
    };
    match err {
        EngineError::Other { code, ref message } => {
            assert_eq!(
                code,
                ErrorCode::CapAttenuation,
                "authority-widening delegation must surface typed CapAttenuation; got {code:?}"
            );
            assert!(
                message.contains("widens authority"),
                "diagnostic must name the authority-widening; got: {message}"
            );
        }
        other => panic!("expected EngineError::Other CapAttenuation; got {other:?}"),
    }
}

fn delegate(
    engine: &Engine,
    source: &benten_core::Cid,
    attenuated: &[String],
) -> Result<benten_core::Cid, EngineError> {
    engine
        .caps()
        .delegate_capability(source, TARGET_PLUGIN_DID, attenuated)
}

/// **F-02 arm 1 — sibling-prefix confusion is REJECTED.** A source grant
/// for `/zone/posts:write` must NOT be delegatable as the SIBLING
/// `/zone/posts-secret:write` (they share a textual prefix but the byte
/// after the prefix is `-`, not a `/`/`:` segment boundary). This is the
/// exact R15/F-01 authority-widening shape.
#[test]
fn delegate_capability_rejects_sibling_prefix_widening() {
    let (engine, _td, source) = engine_with_source_grant("/zone/posts:write");
    let err = delegate(&engine, &source, &["/zone/posts-secret:write".to_string()]).expect_err(
        "sibling `/zone/posts-secret:write` MUST NOT be subsumed by `/zone/posts:write` \
         — reverting the segment-boundary guard admits it (authority-widening)",
    );
    match err {
        EngineError::Other { code, .. } => assert_eq!(code, ErrorCode::CapAttenuation),
        other => panic!("expected CapAttenuation; got {other:?}"),
    }
}

/// **F-02 arm 2 — a strictly broader scope is REJECTED.** Holding
/// `/zone/posts:write` must NOT let a plugin delegate the broader
/// `/zone:write` (parent-path widening).
#[test]
fn delegate_capability_rejects_broader_scope_widening() {
    assert_widening_rejected("/zone/posts:write", "/zone:write");
}

/// **F-02 arm 3 — the `/zone/*` blanket-widen shape is REJECTED.** The
/// exact example named in the fix brief: a plugin holding `/zone/posts`
/// cannot delegate `/zone/*`.
#[test]
fn delegate_capability_rejects_zone_wildcard_widening() {
    assert_widening_rejected("/zone/posts:write", "/zone/*:write");
}

/// **F-02 arm 4 — a GENUINE sub-path attenuation SUCCEEDS.** The guard
/// must not over-fire: `/zone/posts/foo:write` is a true sub-path (`/`
/// boundary) of `/zone/posts:write` and MUST be delegatable.
#[test]
fn delegate_capability_admits_genuine_subpath_attenuation() {
    let (engine, _td, source) = engine_with_source_grant("/zone/posts:write");
    let cid = delegate(&engine, &source, &["/zone/posts/foo:write".to_string()])
        .expect("a true sub-path attenuation MUST be admitted (no over-fire on the guard)");
    assert!(!cid.to_base32().is_empty());
}

/// **F-02 arm 5 — identity (empty attenuation) still SUCCEEDS.** The
/// existing empty-`attenuated_caps` path (identity delegation) is
/// unaffected by the new guard.
#[test]
fn delegate_capability_admits_identity_empty_attenuation() {
    let (engine, _td, source) = engine_with_source_grant("store:posts:write");
    let cid = delegate(&engine, &source, &[])
        .expect("empty attenuated_caps (identity delegation) MUST still admit");
    assert!(!cid.to_base32().is_empty());
}

/// **F-02 arm 6 — explicit exact-scope attenuation SUCCEEDS.** Passing
/// the source scope verbatim as the single attenuated cap is
/// subset-or-EQUAL and MUST admit.
#[test]
fn delegate_capability_admits_exact_scope_attenuation() {
    let (engine, _td, source) = engine_with_source_grant("store:posts:write");
    let cid = delegate(&engine, &source, &["store:posts:write".to_string()])
        .expect("exact-scope (subset-or-equal) attenuation MUST admit");
    assert!(!cid.to_base32().is_empty());
}
