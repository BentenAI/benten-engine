//! Shared `Ucan` chain-construction fixtures for the `typed-CALL`
//! end-to-end pins.
//!
//! ## Purpose (DX-3 closure per `docs/future/phase-3-backlog.md §2.5(f)`)
//!
//! The 4 `ucan_validate_chain_returns_*` pins in
//! `tests/typed_call_engine_dispatch.rs` were each composing the same
//! pattern by hand:
//!
//! 1. Generate an issuer + audience [`Keypair`].
//! 2. Project [`Did`] strings via `keypair.public_key().to_did()`.
//! 3. Build a single-link [`Ucan`] via `Ucan::builder()` with a
//!    capability + `not_before` + `expiry` window.
//! 4. Sign with the issuer keypair + DAG-CBOR-encode for wire transit.
//!
//! Duplicating that prelude across 4 tests caused per-test churn whenever
//! the [`Ucan::builder`] surface evolved. R6 R2 dx-r6-r2-1 surfaced this
//! helper extraction as a Phase-3-close pre-tag fix-pass batch
//! deliverable. The helpers below collapse the prelude into named
//! constructors with bounded parameters so the test bodies retain
//! their per-test diagnostic specificity (audience-mismatch vs
//! cap-mismatch vs leaf-att-mismatch vs happy-path) without re-spelling
//! the chain-construction noise.
//!
//! ## Helpers
//!
//! - [`Chain`] — owned bundle of (issuer keypair + audience keypair +
//!   audience DID string + DAG-CBOR-encoded Ucan bytes). Each test
//!   builder returns a `Chain` so the test body can read DID strings +
//!   wire bytes without re-doing the keypair generation.
//! - [`single_link_chain`] — minimal builder taking a capability
//!   `(resource, ability)` pair + `nbf` + `exp` window. Audience and
//!   issuer keypairs are freshly generated.
//! - [`single_link_chain_with_audience`] — same shape but accepts a
//!   pre-existing audience [`Keypair`] (used by tests that need to
//!   surface the audience DID for explicit cross-reference, e.g. the
//!   "audience mismatch" pin that compares against a *different*
//!   keypair's DID).
//!
//! Test bodies remain responsible for the specific assertion shape +
//! the input map composition + the typed-CALL dispatch surface choice
//! — the helper only collapses the chain-construction prelude.

use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;

/// Owned bundle returned by the chain-construction helpers.
///
/// Carries everything a test body needs to assert against the encoded
/// chain: the issuer + audience keypairs (in case the test needs to
/// derive sibling DIDs), the audience [`Did`] string (the most-frequent
/// inspection surface), and the DAG-CBOR-encoded `Ucan` bytes.
pub struct Chain {
    pub issuer: Keypair,
    pub audience: Keypair,
    pub audience_did: Did,
    pub bytes: Vec<u8>,
}

/// Build a single-link chain (issuer → audience) granting
/// `(resource, ability)` over the `[nbf, exp]` time window.
///
/// Both issuer + audience keypairs are freshly generated. Returns the
/// owned [`Chain`] bundle — the issuer/audience keypairs are dropped
/// only when the bundle is dropped, so DID derivation in the test body
/// remains valid for the full test scope.
pub fn single_link_chain(resource: &str, ability: &str, nbf: Option<u64>, exp: u64) -> Chain {
    let audience = Keypair::generate();
    single_link_chain_with_audience(resource, ability, nbf, exp, audience)
}

/// Build a single-link chain (issuer → audience) granting
/// `(resource, ability)` over the `[nbf, exp]` time window, using a
/// caller-provided audience keypair.
///
/// Used by the audience-mismatch pin that needs to reference a
/// *different* audience DID in the input map than the one the chain
/// was bound to.
pub fn single_link_chain_with_audience(
    resource: &str,
    ability: &str,
    nbf: Option<u64>,
    exp: u64,
    audience: Keypair,
) -> Chain {
    let issuer = Keypair::generate();
    let issuer_did = issuer.public_key().to_did();
    let audience_did = audience.public_key().to_did();

    let mut builder = Ucan::builder()
        .issuer_did(&issuer_did)
        .audience_did(&audience_did)
        .capability(resource, ability)
        .expiry(exp);
    if let Some(nbf) = nbf {
        builder = builder.not_before(nbf);
    }
    let ucan = builder.sign(&issuer);

    let bytes = serde_ipld_dagcbor::to_vec(&ucan).expect("Ucan DAG-CBOR encode must succeed");

    Chain {
        issuer,
        audience,
        audience_did,
        bytes,
    }
}
