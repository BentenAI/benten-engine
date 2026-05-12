//! G27-D — cap-policy fires on plugin-delegated write WITHIN the
//! manifest envelope.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.17 G27-D row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-D entry
//! + CLAUDE.md baked-in #18 layer-(c) (runtime delegation within
//! manifest envelope).
//!
//! ## The within-envelope path (CLAUDE.md #18 layer-c)
//!
//! When plugin A delegates a UCAN cap to plugin B for a scope that
//! the source plugin's manifest `shares` policy PERMITS, the cap-
//! policy MUST observe + permit the audience-side write. This is
//! the "within envelope" path: the runtime delegation matches the
//! manifest's pre-installed consent envelope, so no per-action
//! user prompt fires (per the Phase 6 AI-agent ergonomics goal).
//!
//! ## Pin shape — substantive end-to-end (pim-2 §3.6b)
//!
//! 1. Stub source `PluginManifest` for plugin A with
//!    `shares: { default: Matching, rules: [(cap_pattern="store:notes:write",
//!                                            target=PluginDid(plugin_b_did))] }`.
//! 2. Mint a grant for `store:notes:write` to plugin A's DID.
//! 3. Plugin A delegates UCAN cap to plugin B (via G24-D's
//!    `plugin_delegation::delegate_within_envelope` — yet-unwritten).
//! 4. Audience-side `WriteContext` carries `actor_cid = plugin_b_did`
//!    + `scope = "store:notes:write"`.
//! 5. Assert `check_write(&ctx) == Ok(())` — the delegation is
//!    OBSERVABLE to the cap-policy + permits the write because the
//!    chain traces back to user-root via the source grant + the
//!    delegation fits the source manifest's `shares` envelope.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! G27-D / G24-D not landed: the test fails to compile (the
//! `delegate_within_envelope` seam doesn't exist). At a later
//! shape where the seam exists but skips manifest-envelope
//! consultation: the within-envelope write routes through correctly
//! but the OUTSIDE-envelope sibling pin (file 3) PASSES (would be
//! a fail-OPEN — the policy permits writes outside the envelope).
//!
//! ## RED-PHASE expectation
//!
//! G27-D R5 implementer wires the `delegate_within_envelope` +
//! `check_write_through_envelope` arms; un-ignores at wave-time.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(any())]
mod red_phase_compile_witness {
    use std::sync::Arc;

    use benten_caps::{CapabilityPolicy, GrantBackedPolicy, GrantReader, WriteContext};
    use benten_id::did::Did;
    use benten_platform_foundation::PluginManifest;

    struct MockGrants {
        grants: Vec<String>,
    }

    impl GrantReader for MockGrants {
        fn has_unrevoked_grant_for_scope(
            &self,
            scope: &str,
        ) -> Result<bool, benten_caps::CapError> {
            Ok(self.grants.iter().any(|g| g == scope))
        }
    }

    fn stub_did(label: &str) -> Did {
        let _ = label;
        unimplemented!("RED-PHASE: G27-D — wire real Did construction at wave-time")
    }

    fn stub_manifest_with_shares_rule(_source_did: &Did, _target_did: &Did) -> PluginManifest {
        unimplemented!("RED-PHASE: G27-D — wire stub manifest builder with shares rule")
    }

    #[test]
    fn within_envelope_path_permits_audience_write() {
        let plugin_a_did = stub_did("plugin-a");
        let plugin_b_did = stub_did("plugin-b");

        // Manifest envelope: A shares store:notes:write to B specifically.
        let _manifest_a = stub_manifest_with_shares_rule(&plugin_a_did, &plugin_b_did);

        // Source grant: user → plugin A for store:notes:write.
        let grants = Arc::new(MockGrants {
            grants: vec!["store:notes:write".into()],
        });
        let policy = GrantBackedPolicy::new(grants);

        // Audience-side write under plugin B's principal, scope within
        // the manifest envelope.
        let ctx = WriteContext {
            label: "notes".into(),
            scope: "store:notes:write".into(),
            // actor_cid threaded as plugin_b_did's CID — un-ignore wires
            // the real CID-from-DID conversion.
            ..Default::default()
        };

        // Within-envelope: cap-policy permits the audience-side write.
        // G27-D implementer un-ignores + ensures the policy consults
        // the source manifest's `shares` envelope when actor_cid is
        // a plugin-DID (not the user-root) per CLAUDE.md #18 layer-c.
        policy
            .check_write(&ctx)
            .expect("G27-D within-envelope: cap-policy must permit audience-side write within manifest envelope");
    }
}

/// RED-PHASE outer test.
#[test]
#[ignore = "RED-PHASE: G27-D — un-ignore at G27-D wave AFTER plugin_delegation + manifest-envelope arm wired; drop cfg(any()) gate"]
fn cap_policy_fires_on_plugin_delegated_write_within_manifest_envelope() {
    panic!(
        "RED-PHASE: G27-D — within-envelope arm of plugin_delegation + cap-policy \
         (CLAUDE.md #18 layer-c) must land first; drop cfg(any()) gate + \
         invoke red_phase_compile_witness::within_envelope_path_permits_audience_write()."
    );
}

/// Compile-time witness: F3 stub `PluginManifest` is reachable.
#[test]
fn within_envelope_stub_manifest_reachable_compile_witness() {
    fn _accepts_plugin_manifest(_m: &benten_platform_foundation::PluginManifest) {}
    let _: fn(&benten_platform_foundation::PluginManifest) = _accepts_plugin_manifest;
}
