//! G23-B GREEN: subscribe seam validates pattern shape + emits a
//! `SubscribeAttachToken` for consumer wiring.
//!
//! Closes r2-test-landscape §2.5 row 2 + sec-3.5-r1-9 floor (the
//! routing-is-cursor-only floor; substantive propagation arm lives
//! at G24-A consumer wiring per phase-4-foundation-backlog §4.13).
//!
//! ## Scope (per G23-B mr-5 truthfulness rename)
//!
//! Renamed from `materializer_pipeline_reactive_update_propagates_
//! through_subscribe_seam` to truthful name: this pin validates the
//! token-pattern shape + grep-asserts the source contains zero bare
//! `.on_change(` call sites. It does NOT exercise a reactive update
//! propagation end-to-end (the propagation arm requires a real or
//! mock engine's `on_change_as_with_cursor` callback firing — that's
//! G24-A consumer wiring; phase-4-foundation-backlog §4.13).
//!
//! ## What this pin establishes
//!
//! The materializer's reactive-update seam consumes change events via
//! `Engine::on_change_as_with_cursor` (the cap-rechecking SUBSCRIBE
//! entry point). It MUST NOT route via `on_change` (the
//! unauthenticated cursor — existing surface from Phase 3) per
//! sec-3.5-r1-9.
//!
//! The materializer-side seam pins the production routing via the
//! `SubscribeAttachToken` shape — `subscribe_with_gate(pattern)`
//! returns a token whose `pattern` field MUST be passed to
//! `Engine::on_change_as_with_cursor` by the consumer (admin UI v0
//! shell at G24-A). The SUBSTANCE arm below grep-asserts that
//! materializer.rs's source contains zero bare `.on_change(` call
//! sites — the routing is cursor-only at the trait surface.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_errors::ErrorCode;
use benten_platform_foundation::{HtmlJsonMaterializer, MaterializerError};

#[test]
fn materializer_subscribe_seam_validates_pattern_and_emits_token_for_consumer_wiring() {
    let mat = HtmlJsonMaterializer;

    // RUNTIME ARM: subscribe_with_gate attaches successfully with a
    // valid pattern (the seam shape lock).
    let token = mat
        .subscribe_with_gate("note:*")
        .expect("subscribe seam attaches with valid pattern");
    assert_eq!(
        token.pattern, "note:*",
        "token carries pattern for consumer to pass to Engine::on_change_as_with_cursor"
    );

    // Empty pattern is rejected with E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE
    // (mirrors the engine's `on_change_as_with_cursor` pattern-invalid
    // guard — sec-3.5-r1-9 seam invariant).
    let err = mat.subscribe_with_gate("").unwrap_err();
    assert!(
        matches!(&err, MaterializerError::SubscribeSeamFailure { .. })
            && err.code() == ErrorCode::MaterializerSubscribeSeamFailure,
        "empty pattern surfaces E_MATERIALIZER_SUBSCRIBE_SEAM_FAILURE: got {err:?}"
    );

    // WOULD-FAIL-IF-NO-OP arm: a regression that "no-op'd" the pattern
    // check (always returning Ok regardless of pattern) would fail the
    // empty-pattern assertion above; a regression that "no-op'd" the
    // success path would fail the valid-pattern assertion. Both arms
    // together pin the seam shape per pim-2 §3.6b.

    // SUBSTANCE arm: confirm fixture-side principal still resolves
    // (smoke-check fixture stability).
    let _alice = materializer_fixtures::actor_principal_alice_cid();
}

#[test]
fn materializer_source_calls_only_on_change_as_with_cursor_never_on_change() {
    // G23-B grep-substance pin (paired with the runtime pin above).
    // The materializer.rs source MUST NOT call bare `.on_change(` ever;
    // it MAY (and does, via the SubscribeAttachToken shape that names
    // the cursor-only entry point) reference `on_change_as_with_cursor`
    // in doc-comments per sec-3.5-r1-9.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/materializer.rs"))
        .expect("materializer.rs source readable from tests/");

    // No bare on_change( call sites.
    let bare_calls: Vec<_> = src.match_indices(".on_change(").collect();
    assert!(
        bare_calls.is_empty(),
        "materializer.rs MUST NOT call Engine::on_change directly — found {} \
         call sites at offsets {:?}; route ONLY via on_change_as_with_cursor \
         per sec-3.5-r1-9",
        bare_calls.len(),
        bare_calls.iter().map(|(o, _)| *o).collect::<Vec<_>>(),
    );

    // Cursor surface is named in the file at least once (the seam's
    // routing destination is documented).
    let cursor_mentions: Vec<_> = src.match_indices("on_change_as_with_cursor").collect();
    assert!(
        !cursor_mentions.is_empty(),
        "materializer.rs MUST reference on_change_as_with_cursor at least once \
         (the cursor-only routing destination per sec-3.5-r1-9)"
    );
}
