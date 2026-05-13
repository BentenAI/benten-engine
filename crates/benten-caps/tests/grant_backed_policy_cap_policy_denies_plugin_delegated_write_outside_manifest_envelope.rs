//! G27-D — cap-policy DENIES plugin-delegated write OUTSIDE the
//! manifest envelope (LOAD-BEARING substantive).
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.17 G27-D row
//! ("LOAD-BEARING substantive") +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-D entry
//! + CLAUDE.md baked-in #18 layer-(c) (runtime delegation MUST fit
//! source manifest's `shares` policy or be DENIED) + arch-r1-3
//! ratified split of `E_PLUGIN_MANIFEST_SIGNATURE_INVALID` adjacent
//! ErrorCodes (`E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` lands
//! at G24-D per plan §3 G24-D ErrorCode list).
//!
//! ## The outside-envelope path (CLAUDE.md #18 layer-c, deny side)
//!
//! Plugin A delegates a UCAN cap to plugin B for a scope that the
//! source plugin's manifest `shares` policy DOES NOT permit. The cap-
//! policy MUST DENY the audience-side write — even though the
//! delegation chain mathematically traces to user-root via the
//! source grant, the delegation EXCEEDS the manifest envelope the
//! user consented to at install. This is the LOAD-BEARING half of
//! CLAUDE.md #18 layer-(c): without this denial, the manifest
//! envelope is a paper guarantee.
//!
//! ## Pin shape — substantive end-to-end (pim-2 §3.6b)
//!
//! 1. Stub source `PluginManifest` for plugin A with
//!    `shares: { default: None, rules: [] }` (CONSERVATIVE default —
//!    plugin A explicitly delegates NOTHING via manifest).
//! 2. Mint a grant for `store:notes:write` to plugin A's DID.
//! 3. Plugin A attempts to delegate UCAN cap to plugin B for
//!    `store:notes:write` (via G24-D's `plugin_delegation`).
//! 4. Audience-side `WriteContext` carries `actor_cid = plugin_b_did`
//!    + `scope = "store:notes:write"`.
//! 5. Assert `check_write(&ctx)` returns `CapError::Denied` (or
//!    `CapError::DeniedRead` for the read leg) — the delegation is
//!    REJECTED because it exceeds the source manifest's envelope.
//! 6. Inspect the EngineError carrier: must surface
//!    `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` per arch-r1-3.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer ships the within-envelope arm only + omits the
//! envelope-CHECK in the audience-side check_write path. The
//! within-envelope sister pin (file 2) PASSES; this pin's assertion
//! flips from Denied to Ok — silent fail-OPEN that breaks the layer-
//! (c) consent guarantee. The pair must hold together.
//!
//! ## RED-PHASE expectation
//!
//! G27-D R5 implementer wires the envelope-check in the cap-policy
//! audience-side path. Un-ignores at wave-time. LOAD-BEARING: this
//! is the half that protects user consent + Phase-6 AI-agent
//! ergonomics goal (the manifest envelope IS the consent).

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(any())]
mod red_phase_compile_witness {
    use std::sync::Arc;

    use benten_caps::{CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader, WriteContext};
    use benten_id::did::Did;
    use benten_platform_foundation::{PluginManifest, SharesPolicy, SharesPolicyDefault};

    struct MockGrants {
        grants: Vec<String>,
    }

    impl GrantReader for MockGrants {
        fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
            Ok(self.grants.iter().any(|g| g == scope))
        }
    }

    fn stub_did(label: &str) -> Did {
        let _ = label;
        unimplemented!("RED-PHASE: G27-D — wire real Did construction at wave-time")
    }

    fn stub_manifest_with_no_shares(_source_did: &Did) -> PluginManifest {
        // SharesPolicyDefault::None — conservative default (no delegation
        // permitted). G27-D implementer wires the stub-manifest builder
        // at un-ignore time.
        unimplemented!(
            "RED-PHASE: G27-D — wire stub manifest with SharesPolicyDefault::None + empty rules"
        )
    }

    #[test]
    fn outside_envelope_path_denies_audience_write() {
        let plugin_a_did = stub_did("plugin-a");
        let plugin_b_did = stub_did("plugin-b");

        // Manifest envelope: A shares NOTHING (conservative default).
        let _manifest_a = stub_manifest_with_no_shares(&plugin_a_did);

        // Source grant: user → plugin A for store:notes:write (still
        // exists; the cap is valid for plugin A but NOT delegatable to
        // anyone per the manifest's `shares: None`).
        let grants = Arc::new(MockGrants {
            grants: vec!["store:notes:write".into()],
        });
        let policy = GrantBackedPolicy::new(grants);

        // Audience-side write under plugin B's principal — attempting
        // to use a delegation that EXCEEDS the source manifest envelope.
        let ctx = WriteContext {
            label: "notes".into(),
            scope: "store:notes:write".into(),
            // actor_cid threaded as plugin_b_did's CID — un-ignore wires
            // the real CID-from-DID conversion.
            ..Default::default()
        };

        // LOAD-BEARING: cap-policy MUST DENY the audience-side write
        // because the delegation exceeds the manifest envelope. The
        // chain traces to user-root through the source grant, BUT the
        // user only consented to plugin A holding the cap — not to A
        // re-delegating it to B without an explicit `shares` rule.
        let err = policy
            .check_write(&ctx)
            .expect_err("G27-D outside-envelope: cap-policy MUST deny audience-side write outside manifest envelope (LOAD-BEARING per CLAUDE.md #18 layer-c)");

        // Per arch-r1-3 ratified ErrorCode split, the denial surfaces
        // `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE`. G27-D
        // implementer wires the typed variant; un-ignore extends this
        // assertion to match on the specific code.
        assert!(
            matches!(err, CapError::Denied { .. } | CapError::DeniedRead { .. }),
            "G27-D outside-envelope: must deny via CapError::Denied/DeniedRead carrier; \
             at G24-D ErrorCode landing, narrow to E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE; \
             got {err:?}"
        );
    }
}

/// RED-PHASE outer test. LOAD-BEARING per r2-test-landscape §2.17.
#[test]
#[ignore = "RED-PHASE: G27-D — LOAD-BEARING; un-ignore at G27-D wave AFTER envelope-CHECK arm in cap-policy audience-side path; drop cfg(any()) gate"]
fn cap_policy_denies_plugin_delegated_write_outside_manifest_envelope() {
    panic!(
        "RED-PHASE: G27-D — LOAD-BEARING outside-envelope DENY arm of cap-policy \
         (CLAUDE.md #18 layer-c) must land first; drop cfg(any()) gate + \
         invoke red_phase_compile_witness::outside_envelope_path_denies_audience_write()."
    );
}

/// Compile-time witness: F3 stub `PluginManifest` + `SharesPolicyDefault::None`
/// are reachable from the test crate.
#[test]
fn outside_envelope_stub_manifest_reachable_compile_witness() {
    let none_default = benten_platform_foundation::SharesPolicyDefault::None;
    let _: &benten_platform_foundation::SharesPolicyDefault = &none_default;
}
