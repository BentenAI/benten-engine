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
    /// Borrow a fresh handle on the shared call-count atomic so tests
    /// can observe how many `check_write` calls the policy received.
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
    /// Snapshot the current call count.
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
// Phase 2b G10-B — module-manifest test helpers (D9 + D16)
// ---------------------------------------------------------------------------

/// Phase 2b G10-B test helper: build a minimal valid
/// [`crate::module_manifest::ModuleManifest`] keyed by `name`.
///
/// The manifest carries one module entry, no migrations, no signature.
/// Used by `tests/module_install.rs`,
/// `tests/module_manifest_canonical.rs`, and the cross-crate
/// integration suite.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_make_minimal_manifest(name: &str) -> crate::module_manifest::ModuleManifest {
    crate::module_manifest::ModuleManifest {
        name: name.to_string(),
        version: "0.0.1".into(),
        modules: vec![crate::module_manifest::ModuleManifestEntry {
            name: format!("{name}.handler"),
            cid: format!("bafy_dummy_module_for_{name}"),
            requires: vec![],
        }],
        migrations: vec![],
        signature: None,
    }
}

/// Phase 2b G10-B test helper: build a
/// [`crate::module_manifest::ModuleManifest`] that requires the listed
/// `caps` strings on its single module entry.
///
/// Used by tests asserting capability propagation + retraction across
/// install/uninstall cycles.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_make_manifest_with_caps(
    name: &str,
    caps: &[&str],
) -> crate::module_manifest::ModuleManifest {
    crate::module_manifest::ModuleManifest {
        name: name.to_string(),
        version: "0.0.1".into(),
        modules: vec![crate::module_manifest::ModuleManifestEntry {
            name: format!("{name}.handler"),
            cid: format!("bafy_dummy_module_for_{name}"),
            requires: caps.iter().map(|s| (*s).to_string()).collect(),
        }],
        migrations: vec![],
        signature: None,
    }
}

/// Phase 2b G10-B test helper: compute the canonical CID of a manifest.
///
/// MUST agree with the CID `Engine::install_module` computes
/// internally — without that property, the helper would be a lying
/// oracle and the install-time pin would be untestable. Pinned by
/// `crates/benten-engine/tests/module_install.rs::install_module_compute_cid_helper_round_trips`.
///
/// # Panics
///
/// Panics if the manifest fails to encode. Encoding the
/// [`crate::module_manifest::ModuleManifest`] schema is infallible in
/// practice.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_compute_manifest_cid(
    manifest: &crate::module_manifest::ModuleManifest,
) -> benten_core::Cid {
    manifest
        .compute_cid()
        .expect("ModuleManifest canonical-bytes encoding is infallible")
}

/// Phase 2b G10-B test helper: mint a `Cid` known to differ from any
/// CID a real manifest would produce.
///
/// Used as the "wrong" CID in CID-mismatch tests — pairing this with
/// `testing_compute_manifest_cid(&m)` guarantees the two values
/// differ, satisfying the test invariant
/// `assert_ne!(true_cid, wrong_cid)`.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_make_distinct_dummy_cid() -> benten_core::Cid {
    // BLAKE3 of a literal that is exceedingly unlikely to collide
    // with the canonical-bytes encoding of any real ModuleManifest
    // (the literal is not valid DAG-CBOR for a ModuleManifest).
    let digest = blake3::hash(b"benten:test:fixture:distinct-dummy-cid:G10-B");
    benten_core::Cid::from_blake3_digest(*digest.as_bytes())
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
    // the eval-side canonical bytes path. G12-C added the core-side
    // `Subgraph::to_dagcbor` round-trip on `benten_core::Subgraph` (see
    // `crates/benten-core/src/lib.rs`); the eval-side rich Subgraph (used
    // here for handler dispatch) keeps `canonical_subgraph_bytes` for the
    // registration / immutability path. The mermaid stand-in is retained
    // for backward compatibility with existing testing surfaces.

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

// ---------------------------------------------------------------------------
// Phase 2b wave-8g — G12-E follow-up testing helpers (cfg-gated per
// Phase-2a sec-r6r2-02 discipline so release builds + the napi cdylib do
// not pick up the test-only surface).
//
// The helpers below back the R3 red-phase fixtures under
// `crates/benten-engine/tests/integration/suspension_store_round_trip_*.rs`
// + `crates/benten-eval/tests/subscribe_persist.rs` (the latter already
// passes via `benten-eval::testing` re-exports; the engine-side mirrors
// keep the public spelling consistent across the two test surfaces).
//
// Anything TTL-shaped here is intentionally minimal — D12 WAIT-TTL is a
// separate Phase-3 brief (see wave-8 brief §"Carry-forwards explicitly
// NOT in wave-8") and the helpers below set up the surface for that
// future work without committing to its semantics. In particular,
// `testing_make_wait_metadata_with_ttl_hours` ignores its TTL argument
// and stamps the existing `WaitMetadata` shape; D12 will widen the
// shape with a real `ttl_hours` field.
// ---------------------------------------------------------------------------

/// Phase 2b wave-8g: borrow the engine's configured
/// [`benten_eval::SuspensionStore`] handle.
///
/// Direct alias for [`crate::engine::Engine::suspension_store`]; the
/// duplication on the `testing::` path keeps the test-fixture spelling
/// consistent with the rest of the wave-8g helpers (`testing_make_*`,
/// `testing_call_to_suspend`, etc.) so a fixture can land all of its
/// imports under one `benten_engine::testing::*` umbrella.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_get_suspension_store(
    engine: &crate::engine::Engine,
) -> std::sync::Arc<dyn benten_eval::SuspensionStore> {
    engine.suspension_store()
}

/// Phase 2b wave-8g: deterministic [`benten_core::Cid`] derived from a
/// caller-chosen string label. Two calls with the same label produce
/// bit-identical CIDs; the BLAKE3 seed is namespaced under
/// `phase-2b:wave-8g:wait-id:` so the helper cannot collide with a
/// real-handler CID.
///
/// Used by the `suspension_store_round_trip_wait_metadata` fixtures to
/// pin a synthetic envelope CID without constructing a full
/// `ExecutionStateEnvelope`.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_make_wait_id(label: &str) -> benten_core::Cid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"phase-2b:wave-8g:wait-id:");
    hasher.update(label.as_bytes());
    benten_core::Cid::from_blake3_digest(*hasher.finalize().as_bytes())
}

/// Phase 2b wave-8g: deterministic [`benten_core::SubscriberId`] derived
/// from a caller-chosen string label. Same construction as
/// [`testing_make_wait_id`] but namespaced under
/// `phase-2b:wave-8g:subscriber-id:` so wait-id and subscriber-id
/// derived from the *same* label still produce different bytes — the
/// "no-aliasing" property the
/// `suspension_store_handles_both_wait_and_cursor_keys_without_collision`
/// fixture asserts.
///
/// Note that `SuspensionKey::WaitMetadata(cid)` and
/// `SuspensionKey::Cursor(SubscriberId::from_cid(cid))` for the SAME
/// `cid` value still namespace-separate at the
/// [`crate::suspension_store::RedbSuspensionStore`] table level (one
/// table per variant); these helpers cover the more-typical case of a
/// caller wanting two distinct ids with the same conceptual name.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_make_subscriber_id(label: &str) -> benten_core::SubscriberId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"phase-2b:wave-8g:subscriber-id:");
    hasher.update(label.as_bytes());
    let cid = benten_core::Cid::from_blake3_digest(*hasher.finalize().as_bytes());
    benten_core::SubscriberId::from_cid(cid)
}

/// Phase 2b wave-8g: build a [`benten_eval::WaitMetadata`] entry suitable
/// for a `put_wait` round-trip.
///
/// **TTL is intentionally a no-op in 8g.** D12 WAIT-TTL + GC + the
/// `E_WAIT_TTL_EXPIRED` typed error live in a separate Phase-3 brief
/// (see wave-8 brief §"Carry-forwards explicitly NOT in wave-8"). The
/// `_ttl_hours` argument is preserved on the helper signature so the
/// future D12 lift can widen [`WaitMetadata`] without churning every
/// fixture call site.
///
/// The returned metadata is structurally complete (round-trips byte-for-
/// byte through the SuspensionStore), satisfying the
/// `suspension_store_round_trip_wait_metadata` pin without committing
/// to a TTL semantic this brief is out-of-scope for.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_make_wait_metadata_with_ttl_hours(_ttl_hours: u32) -> benten_eval::WaitMetadata {
    benten_eval::WaitMetadata {
        suspend_elapsed_ms: Some(0),
        timeout_ms: Some(60_000),
        signal_shape: None,
        is_duration: false,
    }
}

/// Phase 2b wave-8g: construct a [`benten_eval::primitives::subscribe::ChangePattern`]
/// from a glob string.
///
/// Convenience wrapper that picks the right pattern variant for the
/// red-phase fixtures; gives the test a single spelling that survives
/// future pattern-language additions.
#[cfg(any(test, feature = "test-helpers"))]
#[must_use]
pub fn testing_make_change_pattern(
    glob: &str,
) -> benten_eval::primitives::subscribe::ChangePattern {
    benten_eval::primitives::subscribe::ChangePattern::LabelGlob(glob.to_string())
}

/// Phase 2b wave-8g: drive a registered handler to its WAIT suspension
/// boundary and return the encoded envelope bytes.
///
/// **Status: minimal stub for the wave-8g surface.** Full integration
/// (registering an actual WAIT handler under a string id, running it
/// to suspension, returning real envelope bytes) is coupled to D12's
/// TTL work + the `cross_process_wait_resume` fixture's broader
/// helpers. Wave-8g lands the function shape so the sister fixtures
/// that don't depend on TTL can wire up cleanly; the body fabricates a
/// deterministic envelope via the existing
/// [`crate::engine::Engine::fabricate_test_suspend_envelope`] hook.
///
/// Tests that need real cross-process resume against a real handler
/// path remain `#[ignore]`-d until D12 + the
/// `testing_make_wait_spec_with_ttl_hours` helper land.
///
/// # Errors
/// Returns [`crate::error::EngineError`] on envelope-encode failure.
#[cfg(any(test, feature = "test-helpers"))]
pub fn testing_call_to_suspend(
    engine: &mut crate::engine::Engine,
    handler_id: &str,
) -> Result<Vec<u8>, crate::error::EngineError> {
    // Derive a deterministic principal so a second call against the
    // same handler_id produces bit-identical envelope bytes (the
    // existing fabricate_test_suspend_envelope contract).
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"phase-2b:wave-8g:testing_call_to_suspend:principal:");
    hasher.update(handler_id.as_bytes());
    let principal = benten_core::Cid::from_blake3_digest(*hasher.finalize().as_bytes());
    engine.fabricate_test_suspend_envelope(&principal)
}

/// Phase 2b wave-8g: stub for `testing_register_persistent_subscriber`.
///
/// **Status: not implemented in 8g.** The body of
/// `subscribe_max_delivered_seq_round_trips_via_suspension_store` drives
/// SUBSCRIBE through the engine boundary (registration + event
/// emission), which depends on the subscribe production-runtime
/// wire-through that wave-8c-cont owns. The function shape lands here
/// for compile-time forward compatibility; the consuming test stays
/// `#[ignore]`-d under that brief.
///
/// # Panics
/// Always panics with a pointer to wave-8c-cont. `#[ignore]`-d call
/// sites never reach the body.
#[cfg(any(test, feature = "test-helpers"))]
pub fn testing_register_persistent_subscriber(
    _engine: &mut crate::engine::Engine,
    _sub_id: benten_core::SubscriberId,
    _pattern: benten_eval::primitives::subscribe::ChangePattern,
) -> Result<(), crate::error::EngineError> {
    panic!(
        "wave-8g shape-only stub: testing_register_persistent_subscriber depends on the \
         SUBSCRIBE production-runtime wire-through owned by wave-8c-cont. The consuming \
         fixture stays `#[ignore]`-d until that brief lands.",
    )
}

/// Phase 2b wave-8g: stub for `testing_emit_n_synthetic_events`.
///
/// Companion stub to [`testing_register_persistent_subscriber`]. See
/// that helper's doc comment for status.
///
/// # Panics
/// Always panics with a pointer to wave-8c-cont. `#[ignore]`-d call
/// sites never reach the body.
#[cfg(any(test, feature = "test-helpers"))]
pub fn testing_emit_n_synthetic_events(
    _engine: &mut crate::engine::Engine,
    _pattern: &str,
    _n: usize,
) -> Result<(), crate::error::EngineError> {
    panic!(
        "wave-8g shape-only stub: testing_emit_n_synthetic_events depends on the SUBSCRIBE \
         production-runtime wire-through owned by wave-8c-cont. The consuming fixture \
         stays `#[ignore]`-d until that brief lands.",
    )
}

/// Phase 2b wave-8g: advance the engine's wait-clock by `delta`.
///
/// **Status: no-op stub for the wave-8g surface.** D12 WAIT-TTL work
/// owns the actual MockTimeSource injection that tests would use to
/// synthesise a TTL expiry without real wall-clock latency. Wave-8g
/// lands the function shape for forward compatibility — fixtures that
/// invoke the helper today receive a deterministic no-op; D12 will
/// wire it through to the real `EvalContext` time source.
///
/// Tests depending on the helper's behaviour (specifically
/// `wait_ttl_expires_via_suspension_store`) remain `#[ignore]`-d under
/// the D12 brief.
#[cfg(any(test, feature = "test-helpers"))]
pub fn testing_advance_wait_clock(
    _engine: &mut crate::engine::Engine,
    _delta: std::time::Duration,
) {
    // Phase-3 D12 brief lifts this to a real MockTimeSource advance.
}

// ---------------------------------------------------------------------------
// Wave-8c-subscribe-infra: 4 ESC integration helpers
//
// Each helper drives one of the production-runtime escape vectors named in
// `.addl/phase-2b/wave-8-brief.md` §8c-subscribe-infra. The helpers are
// cfg-gated under `cfg(any(test, feature = "test-helpers"))` per Phase-2a
// sec-r6r2-02 discipline so the production cdylib does not ship them.
// ---------------------------------------------------------------------------

#[cfg(any(test, feature = "test-helpers"))]
impl crate::Engine {
    /// ESC-7: revoke a cap mid-call. Marks `actor` as revoked in the
    /// engine-wide subscribe-cap-revocation set so the next ad-hoc
    /// onChange delivery for any subscription registered under this
    /// actor fails the D5 cap-recheck and auto-cancels per the D5
    /// contract.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_revoke_cap_mid_call(&self, actor: &benten_core::Cid) {
        self.inner.mark_actor_revoked_for_subscribe(actor);
    }

    /// ESC-9: synchronous engine dispatch entry-point used to detect
    /// host-fn re-entry. Today this delegates straight to
    /// [`Engine::call`] so the integration test can drive a
    /// re-entrant dispatch without spinning a tokio runtime; the
    /// underlying re-entry guard fires inside the dispatcher when
    /// nested SANDBOX calls trip
    /// [`benten_errors::ErrorCode::SandboxNestedDispatchDepthExceeded`].
    ///
    /// Returns the engine's typed [`Outcome`] verbatim so the test
    /// caller can assert the typed-error shape.
    ///
    /// # Errors
    /// Surfaces the same set of typed errors `Engine::call` does.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_call_engine_dispatch(
        &self,
        handler_id: &str,
        op: &str,
        input: std::collections::BTreeMap<String, benten_core::Value>,
    ) -> Result<crate::outcome::Outcome, crate::error::EngineError> {
        self.call(handler_id, op, input)
    }

    /// ESC-10: stamp a forged "cap-claim section" prefix onto a
    /// supplied byte buffer. The wasm-binary-section-section-prefix
    /// machinery the engine consults at module-load is downstream of
    /// this helper; the helper itself only mutates the buffer so a
    /// caller can hand the corrupted bytes to
    /// [`Engine::register_module_bytes`] and assert the load-time
    /// rejection fires.
    ///
    /// The "forge" is a fixed marker pattern that the engine's
    /// host-functions parser would never produce naturally; tests
    /// assert the engine refuses the module load with the typed error
    /// surface for `E_SANDBOX_FORGED_CAP_CLAIM` (per the existing
    /// `sandbox_esc14_*` regression test).
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_inject_forged_cap_claim_section(bytes: &mut [u8]) {
        // Marker: 8 bytes of `0xCC` followed by ASCII `FORGE-CAP`.
        // Chosen so a grep across the wasm corpus finds zero
        // false-positive matches.
        const MARKER: &[u8] = b"\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCCFORGE-CAP";
        let len = bytes.len().min(MARKER.len());
        bytes[..len].copy_from_slice(&MARKER[..len]);
    }

    /// ESC-13 helper: stamp a marker for an "uncounted" host-fn name
    /// into the engine's cfg-gated `test_markers` sideband (cr-w8c-fp-3
    /// — decoupled from `revoked_actors_for_subscribe` so production
    /// cap-revocation semantics stay distinct from test-helper signal).
    ///
    /// The companion engine-layer integration test asserts the marker
    /// round-trips through the same channel a real ESC-13 BACKSTOP
    /// would write. The actual D17 BACKSTOP at the SANDBOX primitive
    /// boundary lives eval-side (G7-A wire-through); this helper is the
    /// engine-layer test-marker injector.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_register_uncounted_host_fn(&self, name: &str) {
        let digest = blake3::hash(name.as_bytes());
        let cid = benten_core::Cid::from_blake3_digest(*digest.as_bytes());
        self.inner.insert_test_marker(&cid);
    }

    /// ESC-13 helper companion: query the cfg-gated `test_markers`
    /// sideband. Returns `true` iff `name` was previously stamped via
    /// `testing_register_uncounted_host_fn`. Used by integration tests
    /// to verify the marker round-trip without reaching into
    /// `EngineInner` directly.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn testing_has_uncounted_host_fn_marker(&self, name: &str) -> bool {
        let digest = blake3::hash(name.as_bytes());
        let cid = benten_core::Cid::from_blake3_digest(*digest.as_bytes());
        self.inner.has_test_marker(&cid)
    }

    /// Wave-8c-subscribe-infra: query the count of in-process ad-hoc
    /// onChange registrations. Diagnostic helper for the integration
    /// tests that assert subscribe lifecycle invariants without
    /// reaching into the eval-side subscribe module directly.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn testing_on_change_registration_count() -> usize {
        benten_eval::primitives::subscribe::on_change_registration_count()
    }
}
