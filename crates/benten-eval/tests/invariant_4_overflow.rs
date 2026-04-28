//! Phase 2b R3-B (filled in by R5 G7-B) — Inv-4 saturation + u8 boundary
//! tests, plus AttributionFrame `sandbox_depth` field shape pin.
//!
//! D20-RESOLVED + wsa-D20:
//!   - At the configured `max_sandbox_nest_depth` ceiling (default 4),
//!     `check_runtime_entry` returns `ErrorCode::InvSandboxDepth` rather
//!     than wrapping.
//!   - At the type-level `u8::MAX` ceiling (the hard ceiling that even a
//!     `max_sandbox_nest_depth = u8::MAX` configuration cannot surpass),
//!     the helper saturates to
//!     `ErrorCode::SandboxNestedDispatchDepthExceeded` — distinct catalog
//!     code so callers can distinguish a configured-ceiling exceed from a
//!     hard-type-ceiling exceed.
//!
//! D20 + Phase-2a sec-r6r1-01 forward-compat: AttributionFrame is
//! extension-shaped — adding `sandbox_depth: u8` does NOT alter the
//! Phase-2a fixture CID when the value is its default zero (verified by
//! the Phase-2a fixture-CID test in `invariant_14_fixture_cid.rs`); a
//! non-zero value produces a content-distinguishable CID (verified here).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::invariants::sandbox_depth::{check_runtime_entry, propagate_through_call};
use benten_eval::{AttributionFrame, InvariantConfig};

fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

fn frame_with_depth(depth: u8) -> AttributionFrame {
    AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: depth,
    }
}

#[test]
fn invariant_4_depth_saturates_to_typed_error_at_max_4() {
    // Default config = 4. depth 0..=4 all push successfully; depth 4 → 5
    // is the trip-wire that fires E_INV_SANDBOX_DEPTH (configured-ceiling
    // exceed code; the type-level u8 ceiling code is distinct — see the
    // wsa-D20 test below).
    let cfg = InvariantConfig::default();
    let mut current = frame_with_depth(0);
    for expected_next in 1u8..=4 {
        current = check_runtime_entry(&current, &cfg).expect("under-ceiling SANDBOX entry");
        assert_eq!(current.sandbox_depth, expected_next);
    }
    let err = check_runtime_entry(&current, &cfg)
        .expect_err("depth 5 must trip configured-ceiling saturation");
    assert_eq!(
        err,
        ErrorCode::InvSandboxDepth,
        "configured-ceiling exceed must use E_INV_SANDBOX_DEPTH (NOT \
         E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED — that's the hard u8 \
         type-level ceiling)"
    );
}

#[test]
fn invariant_4_depth_overflow_saturates_no_wraparound_u8_boundary() {
    // wsa-D20 — `sandbox_depth: u8` does NOT wrap to 0 even when the
    // configured ceiling is `u8::MAX`. The `checked_add(1).ok_or(...)`
    // pattern fires the SaturationOverflow code deterministically.
    let cfg = InvariantConfig {
        max_sandbox_nest_depth: u8::MAX,
        ..InvariantConfig::default()
    };
    let frame = frame_with_depth(u8::MAX);
    let err =
        check_runtime_entry(&frame, &cfg).expect_err("u8::MAX + 1 must saturate, not wrap to 0");
    assert_eq!(
        err,
        ErrorCode::SandboxNestedDispatchDepthExceeded,
        "type-level u8 saturation must use \
         E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED (the more specific code)"
    );
}

#[test]
fn invariant_4_depth_overflow_does_not_mutate_parent_frame() {
    // Defensive — error-path callers MUST NOT receive a partial frame
    // mutation (the helper takes `&parent`, returns a fresh
    // `AttributionFrame` only on success; this test pins the contract).
    let cfg = InvariantConfig::default();
    let parent = frame_with_depth(4);
    let parent_depth_before = parent.sandbox_depth;
    let _err = check_runtime_entry(&parent, &cfg).expect_err("4 -> 5 must reject");
    assert_eq!(
        parent.sandbox_depth, parent_depth_before,
        "parent frame MUST NOT be mutated on error"
    );
}

#[test]
fn attribution_frame_sandbox_depth_field_present_default_zero() {
    // D20 + Phase-2a sec-r6r1-01 — the field exists and constructs as 0
    // by default-equivalent construction.
    let frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
    };
    assert_eq!(frame.sandbox_depth, 0, "default sandbox_depth is 0");

    // Round-trip through DAG-CBOR: `serde(default)` lets a Phase-2a-shape
    // payload (no `sandbox_depth` field present) decode cleanly into the
    // extended struct with the field at 0. We exercise the encoder/decoder
    // round-trip on the default-zero frame and verify the depth survives.
    let bytes = serde_ipld_dagcbor::to_vec(&frame).expect("encode default-zero frame");
    let decoded: AttributionFrame =
        serde_ipld_dagcbor::from_slice(&bytes).expect("decode default-zero frame");
    assert_eq!(decoded, frame);
    assert_eq!(decoded.sandbox_depth, 0);

    // Increment depth to 1 and verify the CID DIFFERS — proves the field
    // IS in canonical bytes when non-zero. This is the security claim of
    // Inv-4: a SANDBOX-bearing attribution chain is content-distinguishable
    // from a non-SANDBOX chain.
    let cid_zero = frame.cid().expect("cid zero");
    let frame_one = AttributionFrame {
        sandbox_depth: 1,
        ..frame.clone()
    };
    let cid_one = frame_one.cid().expect("cid one");
    assert_ne!(
        cid_zero, cid_one,
        "non-zero sandbox_depth must produce a distinct CID"
    );
}

#[test]
fn attribution_frame_propagate_through_call_inherits_depth() {
    // D20 inherit-not-reset: CALL boundary helper carries `sandbox_depth`
    // forward unchanged. SANDBOX entry is the only operation that
    // increments the counter.
    let parent = frame_with_depth(3);
    let after_call = propagate_through_call(&parent);
    assert_eq!(
        after_call.sandbox_depth, 3,
        "CALL must NOT increment sandbox_depth — D20 inherit-not-reset"
    );
}

#[test]
fn invariant_4_check_runtime_entry_increments_by_exactly_one() {
    // D20 — SANDBOX entry increments the counter by exactly 1, not by
    // some derived value of the target module's depth.
    let cfg = InvariantConfig::default();
    let parent = frame_with_depth(2);
    let child = check_runtime_entry(&parent, &cfg).expect("under ceiling");
    assert_eq!(child.sandbox_depth, 3, "exactly +1 per SANDBOX entry");
}
