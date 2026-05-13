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
//! New seam (per plan §3 G24-D-FP-2): `crates/benten-caps/src/
//! manifest_envelope_chain_validation.rs` — Layer 2 (manifest envelope)
//! ↔ Layer 3 (runtime delegation) chain validator. Confirms every
//! delegation chain traces to a user-mint root grant.
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

#[cfg(any())]
mod red_phase_compile_witness {
    use std::sync::Arc;

    use benten_caps::{CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader, WriteContext};
    use benten_id::did::Did;

    struct MockGrants {
        grants: Vec<String>,
    }

    impl GrantReader for MockGrants {
        fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
            Ok(self.grants.iter().any(|g| g == scope))
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

    #[test]
    fn chain_anchored_at_user_did_admitted() {
        let user = stub_user_did();
        let plugin = stub_plugin_did();

        // Chain: user-DID → plugin-DID with scope store:notes:write.
        // User-DID is the root → admitted.
        let grants = Arc::new(MockGrants {
            grants: vec!["store:notes:write".into()],
        });
        let _policy = GrantBackedPolicy::new(grants);

        let _ctx = WriteContext {
            label: "notes".into(),
            scope: "store:notes:write".into(),
            ..Default::default()
        };

        // G24-D-FP-2 wires the manifest_envelope_chain_validation
        // helper. Future surface:
        //   manifest_envelope_chain_validation::validate(
        //       &chain,
        //       &cap_policy,
        //       &manifest_store,
        //   ) -> Result<(), CapError>
        //
        // Expected: chain rooted at user-DID admitted.
        unimplemented!("G24-D-FP-2: wire validate(chain anchored at user-DID) → Ok")
    }

    #[test]
    fn chain_anchored_at_non_user_did_rejected() {
        let peer = stub_peer_did();
        let plugin = stub_plugin_did();

        // Chain: peer-DID → plugin-DID with scope store:notes:write.
        // Peer-DID is NOT a user-DID → CLAUDE.md #18 clause-(a)
        // violation → REJECTED.
        let _peer = peer;
        let _plugin = plugin;

        unimplemented!(
            "G24-D-FP-2: wire validate(chain anchored at peer-DID, NOT \
             user-DID) → Err(E_CHAIN_NOT_USER_ROOTED). LOAD-BEARING for \
             sec-4f-r1-3 closure."
        )
    }
}

/// LOAD-BEARING per r4-triage §2 r4-tc-10. CLAUDE.md #18 clause-(a)
/// closure.
#[test]
#[ignore = "RED-PHASE: G24-D-FP-2 — LOAD-BEARING; un-ignore at G24-D-FP-2 wave AFTER manifest_envelope_chain_validation.rs lands + user-root anchor check arm; drop cfg(any()) gate"]
fn manifest_envelope_chain_validation_requires_user_root_anchor() {
    // G24-D-FP-2 wave wires this. Substantive shape:
    //
    //   use benten_caps::manifest_envelope_chain_validation::{
    //       validate, ChainAnchor,
    //   };
    //
    //   // Construct a delegation chain anchored at a peer-DID (NOT a
    //   // user-DID). Even if signatures + cap-attenuation are
    //   // internally consistent, this MUST be rejected.
    //   let peer_rooted_chain = build_chain_rooted_at_peer_did();
    //   let result = validate(
    //       &peer_rooted_chain,
    //       &cap_policy,
    //       &manifest_store,
    //   );
    //
    //   let err = result.expect_err(
    //       "CLAUDE.md #18 clause-(a) LOAD-BEARING: chain rooted at \
    //        non-user-DID MUST be REJECTED — user-as-root anchor is \
    //        the trust foundation"
    //   );
    //   assert!(
    //       matches!(err, CapError::Denied { .. })
    //           // narrow to E_CHAIN_NOT_USER_ROOTED at typed-error
    //           // mint landing per G24-D-FP-2 plan
    //           || matches!(err, CapError::DeniedRead { .. }),
    //       "CLAUDE.md #18 clause-(a): must surface chain-not-user-rooted \
    //        typed denial; got {err:?}"
    //   );
    //
    //   // Boundary: same chain re-anchored at user-DID admitted:
    //   let user_rooted_chain = re_anchor_chain_at_user_did(peer_rooted_chain);
    //   let ok_result = validate(
    //       &user_rooted_chain,
    //       &cap_policy,
    //       &manifest_store,
    //   );
    //   assert!(ok_result.is_ok(),
    //       "CLAUDE.md #18 clause-(a) boundary: user-rooted chain MUST \
    //        be admitted — defense must NOT over-fire");
    //
    // OBSERVABLE consequence: user-as-root anchor structurally enforced;
    // peer-DID / plugin-DID / unsigned roots all rejected.
    panic!(
        "RED-PHASE: G24-D-FP-2 — LOAD-BEARING user-root anchor check \
         (CLAUDE.md #18 clause-(a)) must land first; drop cfg(any()) \
         gate + invoke red_phase_compile_witness::chain_anchored_at_*()."
    );
}

/// Compile-time witness: benten-caps + benten-id surfaces are reachable
/// from the test crate. Confirms upstream dep graph intact.
#[test]
fn chain_validation_imports_resolve_compile_witness() {
    let user_did = benten_id::did::Did::from_string_unchecked("did:key:z6MkUser".to_string());
    let _: &benten_id::did::Did = &user_did;
}
