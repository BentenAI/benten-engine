//! Phase-2b G7-B / Inv-4 — SANDBOX nest-depth ceiling.
//!
//! Two firing surfaces. Both share the [`DEFAULT_MAX_SANDBOX_NEST_DEPTH`]
//! ceiling and the same `InvariantViolation::SandboxDepth` /
//! `ErrorCode::InvSandboxDepth` typed error code, with the runtime
//! saturation-overflow path also exposing the more specific
//! [`ErrorCode::SandboxNestedDispatchDepthExceeded`] code from the
//! `benten-errors` catalog (D20 saturation).
//!
//! 1. **Registration-time static analysis** — `validate_registration` (private)
//!    walks a [`benten_core::Subgraph`] and counts the longest SANDBOX-only chain
//!    along the call-graph. If the count exceeds the configured ceiling
//!    the subgraph is rejected with [`RegistrationError`]. This catches
//!    statically-determinable violations BEFORE wasmtime ever sees a
//!    module.
//!
//! 2. **Runtime depth check** — [`check_runtime_entry`] is called by the
//!    SANDBOX primitive executor (G7-A) at every SANDBOX entry. The
//!    counter rides on [`AttributionFrame::sandbox_depth`] (D20-RESOLVED)
//!    so it is INHERITED across CALL boundaries: handler A SANDBOXes →
//!    CALLs handler B → SANDBOXes is depth-2, NOT two separate depth-1s.
//!    On overflow we saturate to a typed error rather than wrapping;
//!    `u8::checked_add(1)` is the boundary guard so even a configured
//!    `max_sandbox_nest_depth = u8::MAX` cannot wrap to 0.
//!
//! ## D20-RESOLVED design choices
//!
//! - **Counter on `AttributionFrame.sandbox_depth: u8`** — the evaluator's
//!   per-frame stack would reset at CALL boundaries and let
//!   `SANDBOX → handler → SANDBOX → handler → SANDBOX` chains escape Inv-4
//!   entirely. `AttributionFrame` already crosses CALL boundaries (the
//!   Phase-2a sec-r6r1-01 closure made this guarantee load-bearing for
//!   audit-trail integrity), so it is the natural carrier.
//! - **Default `max_sandbox_nest_depth = 4`** — enough for legitimate
//!   composition (handler that wraps a SANDBOX in a small composition);
//!   5+ smells like accidental recursion. The caller can raise the
//!   ceiling up to `u8::MAX` via [`InvariantConfig::max_sandbox_nest_depth`]
//!   for unusual workloads.
//! - **Saturate to typed error** — gives the user a recoverable failure
//!   mode that routes through `ON_ERROR` per the SANDBOX primitive's edge
//!   surface (`benten-core/src/subgraph.rs:157`). Wrapping would silently
//!   corrupt the depth counter and let arbitrarily-deep nests proceed.
//!
//! ## Pre-instantiation discipline (D22 cold-start coupling)
//!
//! The runtime check fires BEFORE the SANDBOX primitive instantiates a
//! `wasmtime::Module` for the depth-N target. This pin matters because
//! D22 (cold-start budget ≤2-5ms) governs the hot path; if Inv-4 ran
//! AFTER instantiation, a pathological depth-100 attempt would pay 99
//! cold-start budgets only to be rejected at the last hop. The
//! `check_runtime_entry` helper is the integration point G7-A's
//! `primitives::sandbox` executor calls FIRST, before any wasmtime work.

use crate::{
    AttributionFrame, InvariantConfig, InvariantViolation, OperationNode, PrimitiveKind,
    RegistrationError, SubgraphSnapshot,
};
use benten_errors::ErrorCode;

/// Default SANDBOX nest-depth ceiling per D20-RESOLVED (4 levels). The
/// per-engine `InvariantConfig::max_sandbox_nest_depth` knob defaults to
/// this value.
pub const DEFAULT_MAX_SANDBOX_NEST_DEPTH: u8 = 4;

/// Registration-time static SANDBOX nest-depth check.
///
/// Walks the subgraph's edge structure as a DAG of operation nodes and
/// computes, for each node, the longest path-length of SANDBOX nodes
/// reaching that node from any source. If any such count exceeds
/// `config.max_sandbox_nest_depth`, the subgraph is rejected with
/// [`InvariantViolation::SandboxDepth`].
///
/// # Errors
///
/// Returns a [`RegistrationError`] carrying the violated invariant when
/// any SANDBOX-along-chain count exceeds the configured ceiling.
pub(crate) fn validate_registration(
    snapshot: &SubgraphSnapshot<'_>,
    config: &InvariantConfig,
) -> Result<(), RegistrationError> {
    let max_depth = config.max_sandbox_nest_depth;

    // Build adjacency map from outgoing edges. SANDBOX nodes count
    // 1 toward the chain; every other node passes the count through
    // unchanged (CALL is just a pass-through at the structural layer —
    // its body is a separate handler whose own subgraph is validated
    // independently).
    //
    // `SubgraphSnapshot::edges` is `&[(NodeHandle, NodeHandle, String)]`
    // where each `NodeHandle(u32)` indexes `snapshot.nodes`. Skip any
    // edge whose endpoint is out of bounds (defensive — would be caught
    // by Inv-1/Inv-12 already, but we don't want a panic in this
    // walker).
    let n = snapshot.nodes.len();
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (from, to, _label) in snapshot.edges {
        let fi = from.0 as usize;
        let ti = to.0 as usize;
        if fi < n && ti < n {
            adjacency[fi].push(ti);
        }
    }

    // ITERATIVE post-order DFS — computes
    //   longest[i] = self_contrib(i) + max over children of longest[child]
    // For DAG-shaped subgraphs (Inv-1 + Inv-12 already enforce
    // acyclicity by the time Inv-4 runs) this gives the longest
    // SANDBOX chain starting at `i`. The global max across all `i`
    // is the deepest SANDBOX nest the static call-graph allows.
    //
    // Recursion was rejected (would stack-overflow on deep linear
    // subgraphs — observed on the existing Phase-2a 4096-node
    // structural-catch-all tests). The iterative two-stage stack
    // (`Visit` / `Compute`) is the standard hand-rolled post-order.
    let mut longest: Vec<u32> = vec![0; n];
    let mut visited: Vec<bool> = vec![false; n];
    enum Action {
        Visit(usize),
        Compute(usize),
    }
    let mut stack: Vec<Action> = Vec::with_capacity(n);
    for root in 0..n {
        if visited[root] {
            continue;
        }
        stack.push(Action::Visit(root));
        while let Some(action) = stack.pop() {
            match action {
                Action::Visit(i) => {
                    if visited[i] {
                        continue;
                    }
                    // Mark grey; defer the compute until children
                    // resolve.
                    stack.push(Action::Compute(i));
                    for &child in &adjacency[i] {
                        if !visited[child] {
                            stack.push(Action::Visit(child));
                        }
                    }
                }
                Action::Compute(i) => {
                    if visited[i] {
                        continue;
                    }
                    let self_contrib =
                        u32::from(matches!(snapshot.nodes[i].kind, PrimitiveKind::Sandbox));
                    let mut max_child = 0u32;
                    for &child in &adjacency[i] {
                        if longest[child] > max_child {
                            max_child = longest[child];
                        }
                    }
                    longest[i] = self_contrib.saturating_add(max_child);
                    visited[i] = true;
                }
            }
        }
    }

    let global_max = longest.iter().copied().max().unwrap_or(0);
    if global_max > u32::from(max_depth) {
        return Err(RegistrationError::new(InvariantViolation::SandboxDepth));
    }
    Ok(())
}

/// Runtime SANDBOX depth check — called by the SANDBOX primitive
/// executor at every SANDBOX entry.
///
/// Takes the parent [`AttributionFrame`] and returns the child frame
/// with `sandbox_depth = parent + 1`. If the increment would exceed the
/// configured ceiling OR overflow `u8::MAX`, returns the typed
/// saturation-overflow error code per D20.
///
/// # Errors
///
/// - [`ErrorCode::InvSandboxDepth`] when the increment would exceed
///   `config.max_sandbox_nest_depth` (the configured ceiling).
/// - [`ErrorCode::SandboxNestedDispatchDepthExceeded`] when the
///   increment would also overflow the `u8` type-level ceiling
///   (`u8::MAX = 255`). This is the saturation-overflow path; the more
///   specific code lets callers distinguish a configured-ceiling exceed
///   from a hard type ceiling exceed.
pub fn check_runtime_entry(
    parent: &AttributionFrame,
    config: &InvariantConfig,
) -> Result<AttributionFrame, ErrorCode> {
    // u8 type-level ceiling first — even when the configured ceiling is
    // u8::MAX, `checked_add(1)` is the boundary guard. The error code
    // is the saturation-overflow path per D20 wsa-D20.
    let next = parent
        .sandbox_depth
        .checked_add(1)
        .ok_or(ErrorCode::SandboxNestedDispatchDepthExceeded)?;
    // Configured ceiling check. Both checks fire BEFORE wasmtime
    // instantiation per the cold-start discipline above.
    if next > config.max_sandbox_nest_depth {
        return Err(ErrorCode::InvSandboxDepth);
    }
    Ok(AttributionFrame {
        actor_cid: parent.actor_cid,
        handler_cid: parent.handler_cid,
        capability_grant_cid: parent.capability_grant_cid,
        sandbox_depth: next,
    })
}

/// CALL-boundary frame propagation.
///
/// Per D20-RESOLVED, CALL primitive entry does NOT increment
/// `sandbox_depth` — the counter is INHERITED unchanged. Only SANDBOX
/// entry increments. This helper exists so the evaluator's CALL
/// executor can document the inheritance contract explicitly rather
/// than bare struct-cloning at the call-site.
#[must_use]
pub fn propagate_through_call(parent: &AttributionFrame) -> AttributionFrame {
    AttributionFrame {
        actor_cid: parent.actor_cid,
        handler_cid: parent.handler_cid,
        capability_grant_cid: parent.capability_grant_cid,
        sandbox_depth: parent.sandbox_depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benten_core::Cid;

    fn zero_cid() -> Cid {
        Cid::from_blake3_digest([0u8; 32])
    }

    fn parent(depth: u8) -> AttributionFrame {
        AttributionFrame {
            actor_cid: zero_cid(),
            handler_cid: zero_cid(),
            capability_grant_cid: zero_cid(),
            sandbox_depth: depth,
        }
    }

    #[test]
    fn check_runtime_entry_increments_from_zero() {
        let cfg = InvariantConfig::default();
        let child = check_runtime_entry(&parent(0), &cfg).expect("depth 1 ok");
        assert_eq!(child.sandbox_depth, 1);
    }

    #[test]
    fn check_runtime_entry_rejects_at_configured_ceiling() {
        let cfg = InvariantConfig::default(); // max = 4
        // depth 0 -> 1 -> 2 -> 3 -> 4 (all accepted); depth 4 -> 5 rejected.
        let mut current = parent(0);
        for expected_next in 1u8..=4 {
            current = check_runtime_entry(&current, &cfg).expect("under ceiling");
            assert_eq!(current.sandbox_depth, expected_next);
        }
        let err = check_runtime_entry(&current, &cfg).expect_err("depth 5 rejects");
        assert_eq!(err, ErrorCode::InvSandboxDepth);
    }

    #[test]
    fn check_runtime_entry_saturates_at_u8_max_no_wraparound() {
        // Configured ceiling at u8::MAX still has the type-level boundary.
        let cfg = InvariantConfig {
            max_sandbox_nest_depth: u8::MAX,
            ..InvariantConfig::default()
        };
        let frame = parent(u8::MAX);
        let err = check_runtime_entry(&frame, &cfg).expect_err("u8::MAX + 1 saturates");
        assert_eq!(err, ErrorCode::SandboxNestedDispatchDepthExceeded);
    }

    #[test]
    fn propagate_through_call_inherits_depth_unchanged() {
        let p = parent(2);
        let child = propagate_through_call(&p);
        assert_eq!(child.sandbox_depth, 2);
    }
}
