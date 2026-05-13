//! R3 Family E RED-PHASE pin: materializer rejects subgraph whose SANDBOX
//! reference requests a host-fn not in the registered manifest (substantive
//! negative pin; sec-3.5-r1-14 + CLAUDE.md baked-in #16).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 13.
//! - sec-3.5-r1-14 SANDBOX storage-mutating host-fn rejection floor.
//! - CLAUDE.md baked-in #16 — SANDBOX surface min-viable host-fn set:
//!   `time` + `log` + `kv:read` + `random`. Storage-mutating host-fns
//!   (`kv:write`, `kv:delete`) are EXPLICITLY NOT engine concerns.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer SANDBOX host-fn rejection arm doesn't exist at HEAD; G23-B wave-5 wires \
    the rejection check at the SubgraphSpec walk boundary. Closes r2-test-landscape \
    §2.5 row 13 + sec-3.5-r1-14 + CLAUDE.md #16."]
fn materializer_rejects_subgraph_with_unregistered_sandbox_host_fn() {
    // G23-B implementer wires this:
    //
    //   use benten_platform_foundation::materializer::{
    //       HtmlJsonMaterializer, Materializer, MaterializerError,
    //   };
    //   use benten_errors::ErrorCode;
    //
    //   // Hostile spec: references SANDBOX with a host-fn outside the
    //   // CLAUDE.md #16 minimum-viable set (e.g. `fs:write` or `kv:write`).
    //   let hostile_bytes =
    //       materializer_fixtures::hostile_subgraph_with_unregistered_sandbox_host_fn_bytes();
    //   let hostile_spec = decode_subgraph_spec(hostile_bytes).unwrap();
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let err = mat
    //       .materialize_with_gate(/* &engine, &hostile_spec, &alice, .. */ ..)
    //       .expect_err("hostile SANDBOX host-fn MUST be rejected pre-fanout");
    //
    //   // Surfaced via E_SCHEMA_SANDBOX_HOST_FN_REJECTED (G23-A code; the
    //   // materializer-side rejection re-uses the same code; OR a NEW
    //   // E_MATERIALIZER_SCHEMA_MISMATCH if the rejection is structurally
    //   // different at the materializer boundary — G23-B implementer chooses).
    //   match err {
    //       MaterializerError::Other { code, .. } => assert!(
    //           matches!(
    //               code,
    //               ErrorCode::SchemaSandboxHostFnRejected
    //               | ErrorCode::Unknown(ref s)
    //                 if s == "E_MATERIALIZER_SCHEMA_MISMATCH"
    //           ),
    //           "expected sandbox-host-fn rejection ErrorCode, got {code:?}"
    //       ),
    //       other => panic!("expected typed sandbox rejection, got {other:?}"),
    //   }
    //
    //   // SUBSTANCE: no READ fanout happens on the hostile spec. Drive a
    //   // trace + assert zero `read_node_as` events fired (proves the
    //   // rejection is pre-fanout).
    let _ = materializer_fixtures::hostile_subgraph_with_unregistered_sandbox_host_fn_bytes();
    unimplemented!(
        "G23-B wave-5 wires materializer SANDBOX-host-fn pre-fanout rejection per \
         sec-3.5-r1-14 + CLAUDE.md #16"
    );
}
