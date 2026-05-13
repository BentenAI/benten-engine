//! R3 Family E RED-PHASE pin for G23-B cap-denial returns redacted view at
//! Node-granularity (LOAD-BEARING substantive).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 3.
//! - Ratification #7 (admin UI v0 cap-denied → redacted view, not error).
//! - sec-3.5-r1-1 dual-gate inheritance: cap-deny at READ-fanout yields
//!   redaction at Node-granularity NOT a hard error to the caller.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer cap-denial → Node-granularity redaction is not yet wired; G23-B wave-5 \
    lands HtmlJsonMaterializer::materialize_with_gate behavior under denied principals. \
    Closes r2-test-landscape §2.5 row 3 + ratification #7."]
fn materializer_pipeline_capability_denial_returns_redacted_view() {
    // G23-B implementer wires this:
    //
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //   use benten_engine::Engine;
    //   use benten_errors::ErrorCode;
    //
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    //   let unauth = materializer_fixtures::actor_principal_unauthorized_cid();
    //
    //   // Write 1 Note as an authorized actor; then materialize as unauth.
    //   let note = materializer_fixtures::make_note_node("secret body");
    //   engine.transaction(|tx| tx.put_node(&note)).unwrap();
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let out = mat
    //       .materialize_with_gate(/* spec, content_cid, &unauth, deny_all_recheck */ ..)
    //       .expect("materializer returns Ok(redacted) NOT Err for cap-deny per #7");
    //
    //   // Redaction shape: the Note Node is suppressed at the READ fanout;
    //   // the output skeleton contains the SCHEMA SHELL (cap-scope public)
    //   // but the body field is replaced by an opaque "[redacted]" placeholder.
    //   let html = std::str::from_utf8(out.html_bytes()).unwrap();
    //   assert!(
    //       !html.contains("secret body"),
    //       "redacted view MUST NOT leak field content under cap-deny"
    //   );
    //   assert!(
    //       html.contains("[redacted]"),
    //       "redacted view MUST surface placeholder so UI can render an explanation"
    //   );
    //   // The materializer surfaces the cap-deny as a structured frame, but
    //   // returns success (not Err) per ratification #7.
    //   assert_eq!(
    //       out.cap_denials().len(), 1,
    //       "exactly one Node-level cap-denial (the Note body)"
    //   );
    //   assert_eq!(
    //       out.cap_denials()[0].code(),
    //       ErrorCode::from_str("E_MATERIALIZER_CAP_DENIED"),
    //       "denial frame carries E_MATERIALIZER_CAP_DENIED (G23-B NEW)"
    //   );
    let _ = materializer_fixtures::actor_principal_unauthorized_cid();
    let _ = materializer_fixtures::make_note_node("secret body");
    unimplemented!("G23-B wave-5 wires cap-denied → redacted-view shape per ratification #7");
}
