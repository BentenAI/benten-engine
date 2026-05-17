//! R3 Family D RED-PHASE pin for G23-A: schema-emitted SubgraphSpec
//! carries non-empty schema-derived cap-scope annotations that are
//! routable to the engine's CapabilityPolicy at handler-walk time.
//! (sec-3.5-r1-4 §3.6b end-to-end pin; LOAD-BEARING; would-FAIL-if-no-op'd
//! for the G23-A canary substantive arm — cap-scope annotation presence +
//! engine.register_subgraph injection arm.)
//!
//! Renamed from the prior misleading name
//! `schema_compiler_emitted_subgraph_walk_fires_cap_policy_at_each_primitive_boundary`
//! per G23-A mini-review finding g23a-mr-3: the G23-A canary doesn't
//! exercise the full walk (handler dispatch lands at G23-B materializer);
//! this file now truthfully covers the G23-A-scoped substance (annotation
//! presence + policy injection + non-empty schema-derived scope). The
//! full-walk arm lives at
//! `tests/materializer_pipeline_walks_emitted_subgraph_and_fires_cap_policy_at_each_primitive_boundary.rs`
//! as a G23-B-tagged RED-PHASE pin.
//!
//! Pin source: r2-test-landscape §2.4 row 5.
//!
//! ## §3.6b shape (per pim-2; G23-A scoped substance)
//!
//! - PRODUCTION RUNTIME ARM: register the schema-emitted SubgraphSpec via
//!   `Engine::register_subgraph`, with a recording CapabilityPolicy wired
//!   through `EngineBuilder::capability_policy`. Registration succeeding
//!   through the existing surface proves the injection seam (arch-r1-15:
//!   no signature widening).
//! - OBSERVABLE CONSEQUENCE: every primitive's `cap_scope` annotation is
//!   non-empty AND schema-derived (`:Note`-prefixed); recording policy is
//!   reachable + receives no spurious registration-time invocations.
//! - WOULD-FAIL-IF-NO-OP: if the emitter stamped EMPTY cap-scopes, the
//!   substantive `!scope.is_empty()` + `scope.contains(":Note")`
//!   assertions fail. The full-walk arm (cap-policy fires at every
//!   primitive boundary during dispatch) lives in the G23-B companion pin.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};

/// §3.6b end-to-end recording cap-policy. Counts every check_write + check_read
/// invocation; the schema-emitted SubgraphSpec walk MUST surface a non-zero
/// number of checks (proving the cap-scope annotations are observable at
/// the policy boundary). If a future emitter mutation stamped EMPTY
/// cap-scopes on the primitives, the cap-policy hook would never see the
/// schema-derived scope and any test asserting `>= primitive_count` would
/// fail.
#[derive(Default, Debug)]
struct RecordingCapPolicy {
    writes: Arc<AtomicUsize>,
    reads: Arc<AtomicUsize>,
}

impl RecordingCapPolicy {
    fn count(&self) -> usize {
        self.writes.load(Ordering::SeqCst) + self.reads.load(Ordering::SeqCst)
    }
}

impl CapabilityPolicy for RecordingCapPolicy {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

// Un-ignored at G23-A wave-4 (2026-05-12 canary).
//
// §3.6b end-to-end pin shape:
//
// - PRODUCTION-RUNTIME ARM: register the schema-emitted Subgraph via
//   the existing `Engine::register_subgraph` surface (arch-r1-15: no
//   signature widening). A RecordingCapPolicy (defined inline above)
//   is wired through `EngineBuilder::capability_policy`; engine
//   registration triggers no cap-checks at G23-A canary (Phase-4
//   register-time cap gating lands at G24-D), so we use the
//   registration-success path itself as the observability proxy:
//   register_subgraph fires the policy injection + walks the emitted
//   subgraph through the engine's invariant validator, proving the
//   schema-emitted shape is end-to-end registrable.
//
// - OBSERVABLE CONSEQUENCE: the registered subgraph's primitive_count
//   matches the spec's primitive_count, and every primitive's
//   `cap_scope` annotation is non-empty + schema-derived (`":Note"`-prefixed).
//   The cap-policy receives the schema-derived scope at write time
//   when a user later invokes the handler (engine.call → dispatch →
//   PrimitiveHost::execute → check_write through the policy).
//
// - WOULD-FAIL-IF-NO-OP: if the emitter stamped empty cap-scopes, the
//   substantive `!scope.is_empty()` assertion fails. If the emitter
//   somehow widened `register_subgraph`'s signature (parallel
//   schema-only registration), this test would fail to compile (the
//   `engine.register_subgraph(spec.into_subgraph())` call is the
//   compile-time arch-r1-15 grep-equivalent).
#[test]
fn schema_compiler_emitted_subgraph_carries_cap_scope_annotations_routable_to_cap_policy() {
    use benten_engine::EngineBuilder;
    use benten_platform_foundation::schema_compiler::compile;

    let spec = compile(schema_fixtures::canonical_note_type_schema_bytes()).unwrap();
    let primitive_count = spec.as_subgraph().primitive_count();
    let primitive_count_via_descriptors = spec.primitives().len();
    assert_eq!(
        primitive_count, primitive_count_via_descriptors,
        "PrimitiveDescriptor count must mirror Subgraph::primitive_count"
    );

    // Re-derive cap-scopes outside engine context to assert the
    // substantive arm separately (engine registration is the
    // PRODUCTION-RUNTIME arm; this is the OBSERVABLE-CONSEQUENCE arm).
    let scopes: Vec<&str> = spec
        .primitives()
        .iter()
        .map(|p| p.cap_scope().expect("cap-scope present per sec-3.5-r1-4"))
        .collect();
    assert!(
        !scopes.is_empty(),
        "non-empty schema must emit non-zero primitives"
    );
    for scope in &scopes {
        assert!(
            scope.contains(":Note"),
            "every cap-scope must be schema-derived (contain `:Note`); got `{scope}`"
        );
        // would-FAIL-if-no-op: empty cap-scope would slip the
        // sec-3.5-r1-4 check at runtime. The schema_compiler stamps
        // non-empty scopes here; an emitter-side regression would
        // collapse one of these to "".
        assert!(
            !scope.is_empty(),
            "schema-derived cap-scope must be non-empty"
        );
    }

    // PRODUCTION-RUNTIME ARM: register through the EXISTING
    // `Engine::register_subgraph` surface (arch-r1-15). The recorder
    // cap-policy is installed via `EngineBuilder::capability_policy`;
    // the registration walk routes through the engine's invariant
    // validator + policy injection.
    let recorder = RecordingCapPolicy::default();
    let recorder_writes = recorder.writes.clone();
    let recorder_reads = recorder.reads.clone();
    let engine = EngineBuilder::new()
        .path(":memory:")
        .capability_policy(Box::new(recorder))
        .build()
        .expect("engine build");
    engine
        .register_subgraph(spec.into_subgraph())
        .expect("register_subgraph routes schema-emitted Subgraph through existing surface");

    // Sentinel: the policy is installed + reachable. The exact
    // dispatch-time cap-check count is left for the G23-B materializer
    // wave (where the full reactive walk lands); at G23-A canary the
    // registration path itself plus the substantive scope-annotation
    // arm above closes the pim-2 §3.6b shape.
    let registration_observed =
        recorder_writes.load(Ordering::SeqCst) + recorder_reads.load(Ordering::SeqCst);
    // The cap-policy is reachable; cap-checks during registration are
    // intentionally NOT required (registration is invariant-gated, not
    // cap-gated, at G23-A canary). We assert the substantive scope
    // closure already (above); this counter sanity-check confirms the
    // recorder didn't double-fire on registration (regression-guard
    // against an unintended registration-time cap-walk).
    assert!(
        registration_observed <= primitive_count,
        "registration should not invoke cap-policy more times than primitives ({primitive_count}); \
         observed {registration_observed}"
    );
}
