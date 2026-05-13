//! R3 Family E RED-PHASE pin for G23-B reactive update via SUBSCRIBE through
//! `on_change_as_with_cursor` ONLY (LOAD-BEARING substantive; would-FAIL-if-no-op'd).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 2.
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-B; sec-3.5-r1-9
//!   "Change-stream subscription bypasses capability read-checks" closure floor
//!   reaffirmed.
//! - SECURITY-POSTURE.md §"Change-stream subscription bypasses capability
//!   read-checks" — admin UI routes ONLY via `on_change_as_with_cursor`.
//!
//! ## What G23-B wave-5 establishes
//!
//! The materializer's reactive-update seam consumes change events via
//! `Engine::on_change_as_with_cursor` (the cap-rechecking SUBSCRIBE entry
//! point). It MUST NOT route via `on_change` (the unauthenticated cursor —
//! existing surface from Phase 3) per sec-3.5-r1-9. Would-FAIL-if-no-op'd:
//! drives a content WRITE through the engine + verifies the materializer's
//! cursor observes the change, AND verifies the change is filtered out for
//! an actor whose cap is revoked mid-stream.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    the materializer's reactive seam does not exist at HEAD; G23-B wave-5 wires \
    Materializer::subscribe_with_gate routing ONLY through Engine::on_change_as_with_cursor \
    (NEVER on_change). Closes r2-test-landscape §2.5 row 2 + sec-3.5-r1-9 floor."]
fn materializer_pipeline_reactive_update_propagates_through_subscribe_seam() {
    // G23-B implementer wires this:
    //
    //   use benten_platform_foundation::materializer::{
    //       HtmlJsonMaterializer, Materializer,
    //   };
    //   use benten_engine::Engine;
    //
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    //   let alice = materializer_fixtures::actor_principal_alice_cid();
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let mut cursor = mat
    //       .subscribe_with_gate(&engine, &alice, /*spec=*/ ..)
    //       .expect("subscribe seam attaches");
    //
    //   // Initial WRITE — cursor sees the new row.
    //   let note = materializer_fixtures::make_note_node("first body");
    //   engine.transaction(|tx| tx.put_node(&note)).unwrap();
    //   let evt1 = cursor.next().expect("event for first write");
    //   assert!(evt1.html_fragment().contains("first body"));
    //
    //   // Cap-revoke mid-stream: subsequent WRITE is suppressed.
    //   engine.revoke_capability_for_actor(&alice, "read:Note").unwrap();
    //   let note2 = materializer_fixtures::make_note_node("second body");
    //   engine.transaction(|tx| tx.put_node(&note2)).unwrap();
    //   assert!(
    //       cursor.try_next_timeout(std::time::Duration::from_millis(100)).is_none(),
    //       "post-revoke event is filtered at the on_change_as_with_cursor seam — \
    //        would-FAIL-if-no-op'd: a no-op gate yields the event despite revoke"
    //   );
    //
    // SUBSTANCE CHECK: also verify by grep that materializer.rs source
    // contains zero calls to `on_change` (only `on_change_as_with_cursor`).
    let _ = materializer_fixtures::actor_principal_alice_cid();
    unimplemented!(
        "G23-B wave-5 wires Materializer::subscribe_with_gate via on_change_as_with_cursor; \
         end-to-end reactive event-stream pin with mid-stream revoke would-FAIL-if-no-op'd"
    );
}

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    SUBSTANCE-arm grep-assert pin: materializer.rs source MUST contain zero `on_change(` \
    call sites (only `on_change_as_with_cursor(`). Closes sec-3.5-r1-9 grep half."]
fn materializer_source_calls_only_on_change_as_with_cursor_never_on_change() {
    // G23-B wave-5 grep-substance pin (paired with the runtime pin above).
    //
    //   let src = std::fs::read_to_string(
    //       "../../crates/benten-platform-foundation/src/materializer.rs"
    //   ).expect("materializer.rs source readable from tests/");
    //
    //   // No bare on_change( call sites; subscription seams thread the
    //   // cap-rechecking cursor exclusively.
    //   let bare_calls: Vec<_> = src.match_indices(".on_change(").collect();
    //   assert!(
    //       bare_calls.is_empty(),
    //       "materializer.rs MUST NOT call Engine::on_change directly — found {} \
    //        call sites at offsets {:?}; route ONLY via on_change_as_with_cursor \
    //        per sec-3.5-r1-9",
    //       bare_calls.len(),
    //       bare_calls.iter().map(|(o, _)| *o).collect::<Vec<_>>(),
    //   );
    //   let cursor_calls: Vec<_> =
    //       src.match_indices(".on_change_as_with_cursor(").collect();
    //   assert!(
    //       !cursor_calls.is_empty(),
    //       "materializer.rs MUST call on_change_as_with_cursor at least once \
    //        (subscription seam)"
    //   );
    unimplemented!("G23-B wave-5 lands materializer.rs; this pin grep-asserts cursor-only routing");
}
