//! G27-D — manifest-aware scope derivation round-trip.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.17 G27-D row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-D entry
//! + plugin-arch-r1-10 (manifest scope-string grammar pin) + Ben
//! D-4F-1 FULL plugin manifest ratification + CLAUDE.md baked-in #18
//! "Implementation refinements" layered-consent model.
//!
//! ## What this pin verifies
//!
//! Under the FULL plugin manifest (G24-D), the cap-policy scope
//! derivation must consult the manifest `requires` / `shares` halves
//! to map plugin-DID-keyed scope shapes through the policy. The new
//! `crates/benten-caps/src/manifest_scope.rs` module (G27-D) wires a
//! pure function `manifest_requires_to_scope(manifest, plugin_did)`
//! that produces the canonical scope-string set per
//! plugin-arch-r1-10 grammar pin:
//!
//! - `private:<plugin_did>:*`             (private-namespace caps)
//! - `requires:<plugin_did>:<path>`       (manifest `requires` half)
//! - `shares:<plugin_did>:<path>`         (manifest `shares` half)
//!
//! ## Round-trip pin shape
//!
//! 1. Construct a stub `PluginManifest` (F3 type-shape) with two
//!    `requires` entries + one `shares` rule.
//! 2. Invoke the (yet-unwritten) `manifest_requires_to_scope` mapping
//!    function.
//! 3. Mint grants for each derived scope string.
//! 4. Construct `WriteContext` with `scope` field set to one of the
//!    derived shapes; assert `check_write(&ctx) == Ok(())`.
//! 5. Repeat for the inverse direction (a scope shape NOT derivable
//!    from the manifest → no matching grant → denied).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! G27-D module not yet present (at HEAD); the test fails to compile
//! at the `use` line. After G27-D lands the module, an implementer
//! who omits the manifest-derivation arm (returning an empty vec)
//! would PASS the compile but flip this round-trip assertion to deny.
//!
//! ## RED-PHASE expectation
//!
//! G27-D R5 implementer creates `crates/benten-caps/src/manifest_scope.rs`
//! with the public function shape `manifest_requires_to_scope` per
//! plugin-arch-r1-10 grammar. This pin un-ignores at G27-D wave-time
//! per §3.6e + drops the inner `cfg(any())` gate.
//!
//! ## Coupling
//!
//! Co-dependent with G24-D `plugin_delegation.rs` (which mints UCAN
//! delegations carrying these manifest-derived scopes). This pin
//! verifies the scope-DERIVATION half; G24-D's pins verify the
//! delegation-LIFECYCLE half.

#![allow(clippy::unwrap_used, clippy::expect_used)]

// RED-PHASE: G27-D's `benten_caps::manifest_scope` module doesn't
// exist at HEAD. The inner module compiles only when the module
// lands; the implementer drops the cfg(any()) gate to un-ignore.
#[cfg(any())]
mod red_phase_compile_witness {
    use std::sync::Arc;

    use benten_caps::{CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader, WriteContext};
    // Future G27-D surface:
    use benten_caps::manifest_scope::manifest_requires_to_scope;
    use benten_id::did::Did;
    use benten_platform_foundation::{
        CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
    };

    struct MockGrants {
        grants: Vec<String>,
    }

    impl GrantReader for MockGrants {
        fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
            Ok(self.grants.iter().any(|g| g == scope))
        }
    }

    fn plugin_did_stub() -> Did {
        // F3 stub doesn't expose a constructor at R3; un-ignore
        // wires the real did::key generation per benten-id surface.
        unimplemented!("RED-PHASE: G27-D un-ignore wires real Did::from_str for the plugin DID")
    }

    fn manifest_with_requires() -> PluginManifest {
        // F3 stub field-shape directly constructable; G24-D fills
        // validation. Un-ignore wires the proper builder.
        unimplemented!("RED-PHASE: G27-D un-ignore wires the manifest builder")
    }

    #[test]
    fn round_trip() {
        let plugin_did = plugin_did_stub();
        let manifest = manifest_with_requires();

        // G27-D surface — pure function producing canonical scope strings.
        let derived: Vec<String> = manifest_requires_to_scope(&manifest, &plugin_did);

        // Grammar invariant: every derived scope is `requires:<did>:...`
        // or `shares:<did>:...` or `private:<did>:*` shape.
        for scope in &derived {
            assert!(
                scope.starts_with("requires:")
                    || scope.starts_with("shares:")
                    || scope.starts_with("private:"),
                "G27-D grammar (plugin-arch-r1-10): manifest-derived scopes \
                 must match canonical plugin-DID-keyed shape; got {scope}"
            );
        }

        // Round-trip the first derived scope through the policy.
        let grants = Arc::new(MockGrants {
            grants: derived.clone(),
        });
        let policy = GrantBackedPolicy::new(grants);
        let scope = derived
            .first()
            .expect("at least one requires entry")
            .clone();
        let ctx = WriteContext {
            label: String::new(),
            scope: scope.clone(),
            ..Default::default()
        };
        policy
            .check_write(&ctx)
            .expect("G27-D round-trip: manifest-derived scope must permit when grant present");

        // Inverse: a scope OUTSIDE the manifest envelope is denied.
        let grants_2 = Arc::new(MockGrants { grants: vec![] });
        let policy_2 = GrantBackedPolicy::new(grants_2);
        let ctx_2 = WriteContext {
            label: String::new(),
            scope: scope.clone(),
            ..Default::default()
        };
        let err = policy_2.check_write(&ctx_2).expect_err("no grant → deny");
        assert!(
            matches!(err, CapError::Denied { .. }),
            "G27-D inverse: scope without grant must deny; got {err:?}"
        );
    }
}

/// RED-PHASE outer test.
#[test]
#[ignore = "RED-PHASE: G27-D — un-ignore at G27-D wave AFTER manifest_scope::manifest_requires_to_scope lands; drop cfg(any()) gate"]
fn manifest_aware_scope_derivation_round_trip() {
    panic!(
        "RED-PHASE: G27-D — `benten_caps::manifest_scope::manifest_requires_to_scope` must land first \
         (depends on G24-D FULL manifest schema; couples to F3 stub PluginManifest); \
         then drop the cfg(any()) gate above + invoke `red_phase_compile_witness::round_trip()`."
    );
}

/// Compile-time witness: F3 stub `PluginManifest` is reachable from
/// `benten-caps` tests via the dev-dep declared in Cargo.toml. This
/// is the cross-family dependency that G27-D R3 pins must compile
/// against (per r2-test-landscape §4 helper inventory item #2).
#[test]
fn plugin_manifest_stub_reachable_compile_witness() {
    fn _accepts_plugin_manifest(_m: &benten_platform_foundation::PluginManifest) {}
    let _: fn(&benten_platform_foundation::PluginManifest) = _accepts_plugin_manifest;
}
