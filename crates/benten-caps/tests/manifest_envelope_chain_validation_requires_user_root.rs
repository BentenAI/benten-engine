//! Phase-4-Foundation R4-FP-1 — Chain validation pin: manifest envelope
//! chain validation requires user-root (CLAUDE.md #18 clause-a).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-10 + plan §3 G24-D-FP-2 (manifest_envelope_chain_validation.rs
//! NEW seam) + CLAUDE.md baked-in #18 three-layer model clause (a)
//! ("User-as-root anchor").
//!
//! ## What this pin establishes
//!
//! Per CLAUDE.md baked-in #18 clause (a): "Every capability chain
//! traces back to a user-issued root grant. No plugin gets capability
//! without user consent at *some* point in the chain. This anchors
//! trust end-to-end and survives Phase 8 decentralized plugin discovery
//! — when registries are P2P, the user-mint root is still the trust
//! anchor."
//!
//! New seam (per plan §3 G24-D-FP-2):
//! `crates/benten-caps/src/manifest_envelope_chain_validation.rs` —
//! Layer 2 (manifest envelope) ↔ Layer 3 (runtime delegation) chain
//! validator. Confirms every delegation chain traces to a user-mint
//! root grant.
//!
//! LOAD-BEARING for sec-4f-r1-3 closure per r4-triage §2 r4-tc-10.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires chain validation that checks signatures + cap
//! attenuation but skips the user-root-anchor check. A delegation
//! chain that's internally consistent but rooted at a NON-user DID
//! (e.g., a peer-DID, plugin-DID, or unsigned root) is admitted —
//! CLAUDE.md #18 clause-(a) violation. Trust anchor short-circuit.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashSet;

use benten_caps::manifest_envelope_chain_validation::{
    ChainValidationOutcome, DelegationStep, ManifestEnvelopeLookup, UserDidRegistry,
    validate_chain_with_manifest_envelope,
};
use benten_caps::plugin_delegation::SharesPolicyView;
use benten_errors::ErrorCode;
use benten_id::did::Did;

struct AllPermit;
impl SharesPolicyView for AllPermit {
    fn permits(&self, _cap: &str, _target: &Did) -> bool {
        true
    }
}

struct EmptyLookup;
impl ManifestEnvelopeLookup for EmptyLookup {
    type View<'a>
        = &'a AllPermit
    where
        Self: 'a;
    fn lookup<'a>(&'a self, _plugin_did: &Did) -> Option<Self::View<'a>> {
        None
    }
}

struct UserRegistry {
    users: HashSet<String>,
}
impl UserDidRegistry for UserRegistry {
    fn is_user_did(&self, did: &Did) -> bool {
        self.users.contains(did.as_str())
    }
}

fn stub_user_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkUser".to_string())
}

fn stub_plugin_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkPlugin".to_string())
}

fn stub_peer_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkPeer".to_string())
}

/// LOAD-BEARING per r4-triage §2 r4-tc-10. CLAUDE.md #18 clause-(a)
/// closure.
#[test]
fn manifest_envelope_chain_validation_requires_user_root_anchor() {
    let user = stub_user_did();
    let plugin = stub_plugin_did();
    let peer = stub_peer_did();

    let mut users = HashSet::new();
    users.insert(user.as_str().to_string());
    let registry = UserRegistry { users };

    // ATTACK: chain rooted at peer-DID (NOT a user-DID). Signatures +
    // cap-attenuation may be internally consistent; the validator MUST
    // still reject at the clause-(a) anchor check.
    let peer_rooted = vec![DelegationStep {
        issuer_did: peer.clone(),
        audience_did: plugin.clone(),
        cap_pattern: "store:notes:write".into(),
    }];
    let outcome = validate_chain_with_manifest_envelope(&peer_rooted, &EmptyLookup, &registry);
    assert_eq!(
        outcome,
        ChainValidationOutcome::RootNotUserDid,
        "CLAUDE.md #18 clause-(a) LOAD-BEARING: chain rooted at non-user-DID MUST be REJECTED — user-as-root anchor is the trust foundation"
    );
    // Typed error surfaces:
    let err = outcome.into_result().expect_err("non-user root rejects");
    assert_eq!(
        err,
        ErrorCode::PluginManifestInvalid,
        "sec-4f-r1-3: typed error must point at manifest envelope"
    );

    // BOUNDARY: same chain re-anchored at user-DID admitted.
    let user_rooted = vec![DelegationStep {
        issuer_did: user.clone(),
        audience_did: plugin.clone(),
        cap_pattern: "store:notes:write".into(),
    }];
    let ok_outcome = validate_chain_with_manifest_envelope(&user_rooted, &EmptyLookup, &registry);
    assert_eq!(
        ok_outcome,
        ChainValidationOutcome::Admitted,
        "clause-(a) boundary: user-rooted single-step chain MUST be admitted — defense must NOT over-fire"
    );
}

/// Compile-time witness: benten-caps + benten-id surfaces are reachable
/// from the test crate. Confirms upstream dep graph intact.
#[test]
fn chain_validation_imports_resolve_compile_witness() {
    let user_did = benten_id::did::Did::from_string_unchecked("did:key:z6MkUser".to_string());
    let _: &benten_id::did::Did = &user_did;
}
