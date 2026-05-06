//! G14-B durable UCAN backend integration tests (wave-4b).
//!
//! Source pins (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-B
//! + §3.A CLR-2 cluster + R4-FP-R3-B D2 D-PHASE-3-21 closure):
//!
//! - `ucan_backend_chain_walk_against_durable_store` — plan §3 G14-B
//! - `ucan_backend_delegation_attenuation` — plan §3 G14-B
//! - `ucan_backend_revocation_durable_across_restart` — plan §3 G14-B
//! - `ucan_backend_chain_walk_rejects_expired_proof_at_durable_store_lookup` — crypto-blocker-2 + CLR-2
//! - `ucan_backend_no_longer_returns_not_implemented` — plan §3 G14-B
//! - `ucan_backend_accepts_host_atrium_publish_view_result_capability` — D2 D-PHASE-3-21
//! - `ucan_backend_attenuates_host_atrium_publish_view_result_in_chain` — D2 D-PHASE-3-21
//!
//! ## Architectural intent
//!
//! Phase-2b shipped `crate::ucan_stub::UcanBackend` returning
//! `CapError::NotImplemented`. G14-B lights up the durable backend
//! at `crates/benten-caps/src/backends/ucan.rs::UCANBackend<B: GraphBackend>`
//! against an `Arc<RedbBackend>` (or any `B: GraphBackend`). The new
//! type composes `benten_id::ucan::validate_chain_at` with a durable
//! revocation lookup keyed by content-CID.
//!
//! Per §3.6b pim-2, every test drives the production
//! `UCANBackend` entry points through the durable store seam — not
//! against an in-memory short-circuit. Re-opening the backend at the
//! same `RedbBackend` path re-observes prior revocations
//! (`ucan_backend_revocation_durable_across_restart`).
//!
//! ## RED-PHASE → GREEN-PHASE transition
//!
//! G14-B implementer un-ignored the `#[ignore]`'d pins and replaced
//! the `unimplemented!()` bodies with concrete drivers. The
//! `ucan_backend_no_longer_returns_not_implemented` sentinel pin is
//! the load-bearing "stub IS gone" assertion at the source-grep
//! layer.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::{CapError, UCANBackend};
use benten_graph::RedbBackend;
use benten_id::keypair::Keypair;
use benten_id::ucan::{Capability, Ucan};

fn fresh_backend() -> UCANBackend<RedbBackend> {
    let inner = RedbBackend::open_in_memory().expect("redb in-memory open");
    UCANBackend::new(Arc::new(inner))
}

fn fresh_backend_at(path: &std::path::Path) -> UCANBackend<RedbBackend> {
    let inner = RedbBackend::open_or_create(path).expect("redb open_or_create");
    UCANBackend::new(Arc::new(inner))
}

fn now() -> u64 {
    1_000_000_000
}

fn build_ucan(issuer: &Keypair, audience: &Keypair, cap: Capability, nbf: u64, exp: u64) -> Ucan {
    Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(audience.public_key().to_did().as_str())
        .capability(cap.resource, cap.ability)
        .not_before(nbf)
        .expiry(exp)
        .sign(issuer)
}

#[test]
fn ucan_backend_chain_walk_against_durable_store() {
    // plan §3 G14-B pin. The durable backend persists the UCAN +
    // resolves it on validate. OBSERVABLE consequence: chain-walk
    // succeeds end-to-end against a durable store seam.
    let backend = fresh_backend();
    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let now = now();
    let ucan = build_ucan(
        &issuer,
        &audience,
        Capability::new("/zone/posts", "read"),
        now - 1,
        now + 3600,
    );
    let cid = backend.install_proof(&ucan).expect("install_proof");
    backend
        .validate_chain(std::slice::from_ref(&ucan), now)
        .expect("validate_chain");
    // The CID is round-trip-able + the revocation marker is absent.
    assert!(!backend.is_revoked(&cid).unwrap());
}

#[test]
fn ucan_backend_delegation_attenuation() {
    // plan §3 G14-B pin. The durable backend MUST enforce
    // attenuation at chain-walk: a delegated UCAN cannot widen its
    // parent's authority. OBSERVABLE consequence: the durable
    // backend rejects the attenuation-violating chain at
    // validate_chain with a typed UcanAttenuationViolated error.
    let backend = fresh_backend();
    let now = now();

    let root_kp = Keypair::generate();
    let middle_kp = Keypair::generate();
    let leaf_kp = Keypair::generate();

    // Parent grants /zone/posts:read.
    let parent = Ucan::builder()
        .issuer(root_kp.public_key().to_did().as_str())
        .audience(middle_kp.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 3600)
        .sign(&root_kp);

    // Child tries to widen to /zone/posts:write. Attaches the parent
    // as a proof so the chain-walk can spot the widening.
    let overgrant_child = Ucan::builder()
        .issuer(middle_kp.public_key().to_did().as_str())
        .audience(leaf_kp.public_key().to_did().as_str())
        .capability("/zone/posts", "write")
        .not_before(now - 1)
        .expiry(now + 3600)
        .proof(parent.clone())
        .sign(&middle_kp);

    backend.install_proof(&parent).unwrap();
    backend.install_proof(&overgrant_child).unwrap();

    // Leaf-first chain ordering per
    // `crates/benten-id/src/ucan.rs::validate_chain_at` doc:
    // chain[0] is the leaf, chain[1..] are progressively older
    // parents. The chain MUST include the parent so the chain-walk
    // can compare child cap against parent's att list.
    let err = backend
        .validate_chain(&[overgrant_child.clone(), parent.clone()], now)
        .unwrap_err();
    assert!(
        matches!(err, CapError::UcanAttenuationViolated { .. }),
        "expected UcanAttenuationViolated; got {err:?}"
    );

    // Parallel positive case: a properly attenuated child (read same
    // resource) accepts.
    let attenuated_child = Ucan::builder()
        .issuer(middle_kp.public_key().to_did().as_str())
        .audience(leaf_kp.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 3600)
        .proof(parent.clone())
        .sign(&middle_kp);
    backend.install_proof(&attenuated_child).unwrap();
    backend
        .validate_chain(&[attenuated_child, parent], now)
        .expect("attenuated child must accept");
}

#[test]
fn ucan_backend_revocation_durable_across_restart() {
    // plan §3 G14-B pin. Revocation MUST persist across engine
    // restarts via the durable grant store. OBSERVABLE consequence:
    // the same UCAN that validated before the revocation rejects
    // after re-opening the durable store.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("durable.redb");
    let now = now();
    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let ucan = build_ucan(
        &issuer,
        &audience,
        Capability::new("/zone/posts", "read"),
        now - 1,
        now + 3600,
    );

    // Phase 1: install + validate + revoke.
    let cid;
    {
        let backend = fresh_backend_at(&path);
        cid = backend.install_proof(&ucan).unwrap();
        backend
            .validate_chain(std::slice::from_ref(&ucan), now)
            .unwrap();
        backend.revoke(&cid).unwrap();
    }

    // Phase 2: re-open at the same path. Revocation persists.
    {
        let backend = fresh_backend_at(&path);
        assert!(backend.is_revoked(&cid).unwrap());
        let err = backend.validate_chain(&[ucan], now).unwrap_err();
        assert!(
            matches!(err, CapError::Revoked),
            "expected Revoked; got {err:?}"
        );
    }
}

#[test]
fn ucan_backend_chain_walk_rejects_expired_proof_at_durable_store_lookup() {
    // crypto-blocker-2 BLOCKER + CLR-2 pin. Even when a UCAN is
    // stored in the durable proof index, presenting it AFTER its
    // `exp` window MUST reject at chain-walk.
    let backend = fresh_backend();
    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let issuance = 1_000_000_000;
    let ucan = build_ucan(
        &issuer,
        &audience,
        Capability::new("/zone/posts", "read"),
        issuance,
        issuance + 60,
    );
    backend.install_proof(&ucan).unwrap();

    // Within window: passes.
    backend
        .validate_chain(std::slice::from_ref(&ucan), issuance + 30)
        .unwrap();
    // After window: rejects with UcanExpired despite living in the
    // durable store.
    let err = backend.validate_chain(&[ucan], issuance + 120).unwrap_err();
    assert!(
        matches!(err, CapError::UcanExpired { .. }),
        "expected UcanExpired; got {err:?}"
    );
}

#[test]
fn ucan_backend_no_longer_returns_not_implemented() {
    // plan §3 G14-B pin. The Phase-2b stub returned
    // `CapError::NotImplemented` from every UCANBackend entry point.
    // G14-B replaces the stub: every public UCANBackend method
    // returns either Ok or a real (non-NotImplemented) typed error.
    //
    // Source-grep sentinel: ensure the `backends/ucan.rs` source
    // file does NOT construct `CapError::NotImplemented` anywhere.
    // Reads the source file relative to `CARGO_MANIFEST_DIR` so the
    // assertion lands at the right file.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir).join("src/backends/ucan.rs");
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("read {}: {e}", path.display());
    });
    // Source-grep sentinel: ensure the durable backend file does NOT
    // CONSTRUCT a `CapError::NotImplemented { ... }` value. The
    // module docstring legitimately mentions the symbol when
    // narrating "this replaces the Phase-2b stub" — that mention is
    // permitted because it sits inside `//!` comments. The grep
    // pattern we forbid is the constructor body shape:
    // `CapError::NotImplemented {` (with the brace) — that shape is
    // unique to the stub return path.
    assert!(
        !source.contains("CapError::NotImplemented {"),
        "G14-B source MUST NOT construct `CapError::NotImplemented {{ ... }}`; \
         the durable backend has replaced the stub. (The docstring \
         narrative naming the symbol is allowed; the assertion forbids \
         only the constructor literal `CapError::NotImplemented {{`.)"
    );

    // Behavioral sentinel: every public entry point returns a non-
    // NotImplemented result on a fresh backend.
    let backend = fresh_backend();
    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let now = now();
    let ucan = build_ucan(
        &issuer,
        &audience,
        Capability::new("/zone/posts", "read"),
        now - 1,
        now + 3600,
    );

    let cid = match backend.install_proof(&ucan) {
        Err(CapError::NotImplemented { .. }) => {
            panic!("install_proof must NOT return NotImplemented")
        }
        Ok(cid) => cid,
        Err(other) => panic!("install_proof unexpected: {other:?}"),
    };
    if let Err(CapError::NotImplemented { .. }) =
        backend.validate_chain(std::slice::from_ref(&ucan), now)
    {
        panic!("validate_chain must NOT return NotImplemented");
    }
    if let Err(CapError::NotImplemented { .. }) = backend.revoke(&cid) {
        panic!("revoke must NOT return NotImplemented");
    }
}

// ============================================================
// D-PHASE-3-21 D2 closure (Ben's 2026-05-05 ratification: option
// (iii) + UCAN-gated `host:atrium:publish_view_result` capability,
// no new trust-policy primitive). The durable backend recognizes
// the new capability string in the chain-walk via standard UCAN
// resource:ability shape (no special-case bypass).
// ============================================================

#[test]
fn ucan_backend_accepts_host_atrium_publish_view_result_capability() {
    let backend = fresh_backend();
    let now = now();
    let publisher = Keypair::generate();
    let viewer = Keypair::generate();
    let ucan = Ucan::builder()
        .issuer(publisher.public_key().to_did().as_str())
        .audience(viewer.public_key().to_did().as_str())
        .capability("host:atrium:publish_view_result", "*")
        .not_before(now - 1)
        .expiry(now + 3600)
        .sign(&publisher);
    backend.install_proof(&ucan).unwrap();
    backend
        .validate_chain(&[ucan], now)
        .expect("host:atrium:publish_view_result MUST be recognized at chain-walk");
}

#[test]
fn ucan_backend_attenuates_host_atrium_publish_view_result_in_chain() {
    let backend = fresh_backend();
    let now = now();
    let root = Keypair::generate();
    let middle = Keypair::generate();
    let leaf = Keypair::generate();

    // Parent: only specific atrium namespace.
    let parent = Ucan::builder()
        .issuer(root.public_key().to_did().as_str())
        .audience(middle.public_key().to_did().as_str())
        .capability("host:atrium:publish_view_result", "/atrium/specific")
        .not_before(now - 1)
        .expiry(now + 3600)
        .sign(&root);

    // Child tries to widen to all-atrium wildcard.
    let widening_child = Ucan::builder()
        .issuer(middle.public_key().to_did().as_str())
        .audience(leaf.public_key().to_did().as_str())
        .capability("host:atrium:publish_view_result", "*")
        .not_before(now - 1)
        .expiry(now + 3600)
        .proof(parent.clone())
        .sign(&middle);

    backend.install_proof(&parent).unwrap();
    backend.install_proof(&widening_child).unwrap();

    // Leaf-first chain ordering per
    // `crates/benten-id/src/ucan.rs::validate_chain_at` doc.
    let err = backend
        .validate_chain(&[widening_child, parent], now)
        .unwrap_err();
    assert!(
        matches!(err, CapError::UcanAttenuationViolated { .. }),
        "host:atrium:publish_view_result MUST attenuate per D-PHASE-3-21 D2 + standard UCAN semantics; got {err:?}"
    );
}
