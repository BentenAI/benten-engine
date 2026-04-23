//! Test helpers used by integration tests from sibling crates
//! (`benten-caps/tests/*.rs`, `benten-eval/tests/*.rs`).
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). The surface here is
//! stable across Phase-1 implementation groups; the Phase-2 evaluator
//! integration fills in the `unimplemented!`-adjacent shells without
//! changing the public signatures.

#![allow(clippy::todo, reason = "Phase-2 scope")]

use benten_caps::CapabilityPolicy;

use crate::outcome::Outcome;
use crate::subgraph_spec::SubgraphSpec;

/// Build a synthetic ITERATE-heavy handler for TOCTOU tests.
#[must_use]
pub fn iterate_write_handler(_max: u32) -> SubgraphSpec {
    SubgraphSpec::empty("iterate_write")
}

/// Build a minimal single-WRITE handler — WRITE(label=`minimal`) → RESPOND.
///
/// Used by the UCAN stub routing test (r6-sec-4) to verify that a
/// configured `UcanBackend` routes its `NotImplemented` error through
/// the `ON_ERROR` typed edge rather than `ON_DENIED`. The minimal WRITE
/// must reach the capability hook, so the spec carries one `WriteSpec`
/// and a RESPOND terminal — not the earlier empty shell.
#[must_use]
pub fn minimal_write_handler() -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id("minimal_write")
        .write(|w| w.label("minimal"))
        .respond()
        .build()
}

/// Inspect the edge taken by the terminal step of an Outcome.
#[must_use]
pub fn route_of_error(outcome: &Outcome) -> String {
    outcome.edge_taken().unwrap_or_default()
}

/// Build a READ-only handler for existence-leak tests.
#[must_use]
pub fn read_handler_for<T: ReadHandlerTarget>(_target: T) -> SubgraphSpec {
    SubgraphSpec::empty("read_handler")
}

/// Sugar trait — see [`read_handler_for`].
pub trait ReadHandlerTarget {}
impl ReadHandlerTarget for &str {}
impl ReadHandlerTarget for &String {}
impl ReadHandlerTarget for String {}
impl ReadHandlerTarget for benten_core::Cid {}

/// Synthesize a Subject with no read grants. Returns a boxed
/// `CapabilityPolicy` — Phase 1 uses NoAuth so reads are always allowed;
/// the Phase 2 read-denial policy replaces this body.
#[must_use]
pub fn subject_with_no_read_grants() -> Box<dyn CapabilityPolicy> {
    Box::new(benten_caps::NoAuthBackend::new())
}

/// Adversarial fixture: handler declares `requires: post:read` but writes to admin.
#[must_use]
pub fn handler_declaring_read_but_writing_admin() -> SubgraphSpec {
    SubgraphSpec::empty("bad_declaring_read")
}

/// Second-order escalation fixture.
#[must_use]
pub fn handler_with_call_attenuation_escalation() -> SubgraphSpec {
    SubgraphSpec::empty("call_attenuation_escalation")
}

/// Build a capability policy pre-seeded with a grant set.
#[must_use]
pub fn policy_with_grants(_grants: &[&str]) -> Box<dyn CapabilityPolicy> {
    Box::new(benten_caps::NoAuthBackend::new())
}

/// Build a policy that counts check_write invocations.
#[must_use]
pub fn counting_capability_policy() -> CountingPolicy {
    CountingPolicy {
        count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
    }
}

/// Counting capability-policy wrapper.
pub struct CountingPolicy {
    count: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl CountingPolicy {
    #[must_use]
    pub fn call_counter(&self) -> CallCounter {
        CallCounter {
            count: std::sync::Arc::clone(&self.count),
        }
    }
}

impl benten_caps::CapabilityPolicy for CountingPolicy {
    fn check_write(&self, _ctx: &benten_caps::WriteContext) -> Result<(), benten_caps::CapError> {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// Atomic counter handle.
pub struct CallCounter {
    count: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl CallCounter {
    #[must_use]
    pub fn load(&self) -> u32 {
        self.count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Build a READ→WRITE→READ handler for per-primitive cap-check assertions.
#[must_use]
pub fn handler_with_read_write_read_sequence() -> SubgraphSpec {
    SubgraphSpec::empty("rwr")
}

/// Phase 2a G2-B/G3-B test helper: READ → RESPOND handler. The leading
/// `primitive("r", Read)` ensures `respond` has a predecessor per g7-cr-13.
#[must_use]
pub fn minimal_respond_handler(handler_id: &str) -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive("r", benten_eval::PrimitiveKind::Read)
        .respond()
        .build()
}

/// Phase 2a G3-B test helper: a minimal WAIT handler for benchmark
/// fixtures.
#[must_use]
pub fn minimal_wait_handler(handler_id: &str) -> SubgraphSpec {
    SubgraphSpec::empty(handler_id)
}

/// Phase 2a G9-A test helper: deterministic actor CID derived from a name.
/// Two callers passing the same name get bit-identical CIDs.
#[must_use]
pub fn principal_cid(name: &str) -> benten_core::Cid {
    let digest = blake3::hash(name.as_bytes());
    benten_core::Cid::from_blake3_digest(*digest.as_bytes())
}

/// Phase 2a G9-A test helper: returns `(boxed policy, counter)` so tests
/// can destructure the check-count side of the counting policy AND pass the
/// boxed form to `.capability_policy(...)` directly.
#[must_use]
pub fn counting_policy() -> (Box<dyn CapabilityPolicy>, CallCounter) {
    let p = counting_capability_policy();
    let c = p.call_counter();
    (Box::new(p), c)
}
