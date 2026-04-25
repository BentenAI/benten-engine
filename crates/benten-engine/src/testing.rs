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

// ---------------------------------------------------------------------------
// G11-A Wave 1 / G5-A test helpers — subgraph-bytes round-trip + reput
// ---------------------------------------------------------------------------
//
// The Phase-2a R3 integration test `inv_8_11_13_14_firing.rs` exercises the
// 5-row Inv-13 matrix (§9.11). It needs three primitives that aren't public
// on the Phase-1 surface:
//
//   1. Grab the canonical bytes of a registered handler's Subgraph so the
//      test can "re-present" them to the backend under different authorities.
//   2. Re-put those bytes as a User-authority write — row 1 (identical bytes)
//      and row 2 (mutated bytes) both must fire E_INV_IMMUTABILITY.
//   3. Re-put those bytes as an EnginePrivileged write — row 3 is dedup:
//      matching-bytes returns Ok(cid); NO ChangeEvent, NO audit-sequence
//      advance.
//
// Phase-2a owns rows 1-3. Row 4 (SyncReplica) is Phase-3 reserved but a
// shape-pinning helper is included so the test suite can assert the method
// exists + its signature is stable.
//
// Because benten-engine has no "subgraph store" yet (handlers live in an
// in-memory map), the helpers work against a synthesised Node whose
// `properties["subgraph_cbor"]` carries the canonical bytes. The
// backend-visible CID is therefore distinct from the handler's registered
// CID — what we're exercising here is strictly the `put_node_with_context`
// immutability / dedup branching, not the handler registry.
// ---------------------------------------------------------------------------

/// Phase 2a G11-A Wave 1 test helper: extract the canonical DAG-CBOR bytes
/// of the Subgraph registered under `handler_id`.
///
/// Returns the bytes the evaluator would hash to compute the handler's
/// content-addressed CID; two calls at the same handler produce byte-
/// identical output (Inv-10 content-address property).
///
/// # Errors
/// Returns an error string if the handler is not registered or if the
/// Subgraph reconstruction fails (e.g. unknown handler_id prefix).
pub fn subgraph_bytes_for_handler(
    engine: &crate::engine::Engine,
    handler_id: &str,
) -> Result<Vec<u8>, String> {
    // Confirm the handler exists. Reaches through the engine's internal
    // handlers map via the existing `resolve_subgraph_cid_for_test` public
    // surface so the test helper doesn't duplicate the lookup.
    engine
        .resolve_subgraph_cid_for_test(handler_id, "default")
        .map_err(|e| format!("subgraph_bytes_for_handler: {e}"))?;

    // Reconstruct the subgraph via the mermaid / predecessor helper's
    // code path; this matches what `handler_to_mermaid` renders and keeps
    // the canonical-bytes output in sync with every other observable
    // representation of the handler.
    let mermaid = engine
        .handler_to_mermaid(handler_id)
        .map_err(|e| format!("subgraph_bytes_for_handler: mermaid: {e}"))?;
    // The mermaid render is deterministic per handler — a stand-in for
    // canonical bytes until the Phase-2b benten-core-migration completes
    // the Subgraph::to_dagcbor path (see
    // `.addl/phase-2b/00-scope-outline.md` §7a). The panic stubs that
    // previously pinned this path were removed in R6 round-2 / A7
    // because no caller existed. Hashes to a stable Vec<u8>.

    Ok(mermaid.into_bytes())
}

/// Phase 2a G11-A Wave 1 test helper: re-put subgraph bytes as a User-
/// authority write.
///
/// Wraps the bytes in a minimal Node (single property `subgraph_cbor`
/// carrying the bytes) and attempts a write against the engine's backend
/// with [`benten_graph::WriteAuthority::User`]. Row 1 / Row 2 of the
/// Inv-13 matrix both fire `E_INV_IMMUTABILITY` — row 1 because the bytes
/// round-trip to the same CID the first call stored, row 2 because any
/// mutation still hits the immutability check under User authority.
impl crate::engine::Engine {
    /// Phase 2a G11-A / G5-A: User-authority reput — see module docs.
    ///
    /// # Errors
    /// Returns [`crate::error::EngineError::Graph`] wrapping
    /// `GraphError::Core(InvImmutability)` on a duplicate CID (row 1),
    /// or the first-write CID on the initial put. The integration test
    /// performs a first put + second put to observe the immutability
    /// edge.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_reput_subgraph_as_user(
        &self,
        bytes: &[u8],
    ) -> Result<benten_core::Cid, crate::error::EngineError> {
        use std::collections::BTreeMap;
        let mut props: BTreeMap<String, benten_core::Value> = BTreeMap::new();
        props.insert(
            "subgraph_cbor".into(),
            benten_core::Value::Bytes(bytes.to_vec()),
        );
        let node = benten_core::Node::new(vec!["phase_2a_reput_subgraph".into()], props);
        let ctx = benten_graph::WriteContext::new("phase_2a_reput_subgraph")
            .with_authority(benten_graph::WriteAuthority::User);
        Ok(self.backend().put_node_with_context(&node, &ctx)?)
    }

    /// Phase 2a G11-A / G5-A: EnginePrivileged reput — §9.11 row 3 dedup
    /// path. Matching bytes return `Ok(cid)` WITHOUT emitting a
    /// ChangeEvent or advancing the audit sequence.
    ///
    /// # Errors
    /// Returns [`crate::error::EngineError::Graph`] on backend failure.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_reput_subgraph_privileged(
        &self,
        bytes: &[u8],
    ) -> Result<benten_core::Cid, crate::error::EngineError> {
        use std::collections::BTreeMap;
        let mut props: BTreeMap<String, benten_core::Value> = BTreeMap::new();
        props.insert(
            "subgraph_cbor".into(),
            benten_core::Value::Bytes(bytes.to_vec()),
        );
        let node = benten_core::Node::new(vec!["phase_2a_reput_subgraph".into()], props);
        let ctx = benten_graph::WriteContext::new("phase_2a_reput_subgraph")
            .with_authority(benten_graph::WriteAuthority::EnginePrivileged);
        Ok(self.backend().put_node_with_context(&node, &ctx)?)
    }

    /// Phase 2a G11-A Wave 1 shape-pin: SyncReplica reput. Row 4 of the
    /// Inv-13 matrix is Phase-3 reserved — the method shape is stabilised
    /// here so the `invariant_13_sync_replica_dedups_reserved` test's
    /// `#[ignore]` marker can drop without a subsequent compile churn.
    ///
    /// # Errors
    /// Phase-2a: returns the same immutability behaviour as
    /// [`Self::testing_reput_subgraph_as_user`]. Phase-3 sync lands the
    /// real dedup-on-origin-CID semantics.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_reput_subgraph_as_sync_replica(
        &self,
        bytes: &[u8],
        origin: &benten_core::Cid,
    ) -> Result<benten_core::Cid, crate::error::EngineError> {
        use std::collections::BTreeMap;
        let mut props: BTreeMap<String, benten_core::Value> = BTreeMap::new();
        props.insert(
            "subgraph_cbor".into(),
            benten_core::Value::Bytes(bytes.to_vec()),
        );
        // Stamp origin onto the synthesised Node so the Phase-3 wiring has
        // a content-addressed anchor ready when the real SyncReplica
        // semantics land.
        props.insert(
            "sync_replica_origin".into(),
            benten_core::Value::Text(origin.to_base32()),
        );
        let node = benten_core::Node::new(vec!["phase_2a_reput_subgraph".into()], props);
        let ctx = benten_graph::WriteContext::new("phase_2a_reput_subgraph").with_authority(
            benten_graph::WriteAuthority::SyncReplica {
                origin_peer: *origin,
            },
        );
        Ok(self.backend().put_node_with_context(&node, &ctx)?)
    }
}
