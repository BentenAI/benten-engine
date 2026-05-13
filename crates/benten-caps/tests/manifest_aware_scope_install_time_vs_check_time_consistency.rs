//! G27-D — install-time vs check-time scope-derivation consistency
//! (pim-13 §3.12 spec-to-code-compliance pin).
//!
//! ## Pin source
//!
//! `docs/future/phase-4-backlog.md` §4.4 cap-r1-3 closure — names
//! the "install-time-vs-check-time decision" as load-bearing for the
//! manifest scope grammar. The implementer ratified install-time as
//! the preferred derivation (perf), check-time as the fallback +
//! consistency oracle.
//!
//! ## What this pin verifies
//!
//! `manifest_requires_to_scope` + `manifest_shares_to_scope` are
//! PURE functions over `(manifest, plugin_did)`. Re-deriving at
//! check-time produces output byte-for-byte identical to install-
//! time output. The invariant guarantees that:
//!
//! - Frozen-at-install scope strings remain valid lookup keys for
//!   the lifetime of the plugin-DID's installation.
//! - A re-derivation path (e.g., recovery after store loss; audit
//!   pipeline cross-checking the install record's frozen scopes
//!   against the manifest body) produces the canonical strings the
//!   `GrantBackedPolicy` keys off.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer ships a derivation that injects a timestamp / nonce
//! / `OsRng` byte into the scope string. The function ceases to be
//! pure; install-time-derived `requires:<did>:<scope>:<random>` no
//! longer matches check-time-derived. This pin's `assert_eq!` fails
//! — the implementer cannot ship the impure derivation.
//!
//! ## Composes with
//!
//! - pim-13 §3.12: spec-to-code-compliance — `docs/future/phase-4-
//!   backlog.md` §4.4 names purity as the implementation contract;
//!   this pin verifies the contract holds.
//! - pim-2 §3.6b: the would-FAIL-if-no-op'd 4-axis check is explicit
//!   above; non-pure derivation flips the assertion.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::manifest_scope::{manifest_requires_to_scope, manifest_shares_to_scope};
use benten_id::did::Did;
use benten_platform_foundation::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault, SharesRule, SharesTarget,
};

fn plugin_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkConsistency".to_string())
}

fn manifest_with_full_envelope() -> PluginManifest {
    PluginManifest {
        plugin_name: "consistency-test".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([7u8; 32]),
        peer_did: plugin_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![
            CapRequirement {
                scope: "store:notes:read".to_string(),
            },
            CapRequirement {
                scope: "private:admin-private:scratch".to_string(),
            },
            CapRequirement {
                scope: "host:time:now".to_string(),
            },
            CapRequirement {
                scope: "store:plugins:read".to_string(),
            },
        ],
        shares: SharesPolicy {
            default: SharesPolicyDefault::Matching,
            rules: Some(vec![
                SharesRule {
                    cap_pattern: "store:notes:write".to_string(),
                    target: SharesTarget::Any,
                },
                SharesRule {
                    cap_pattern: "store:other:*".to_string(),
                    target: SharesTarget::PluginDid(plugin_did()),
                },
            ]),
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

/// G27-D pim-13: install-time vs check-time derivation produces
/// byte-identical scope-string output. Verifies the purity contract
/// `docs/future/phase-4-backlog.md` §4.4 names.
#[test]
fn manifest_aware_scope_install_time_vs_check_time_consistency() {
    let did = plugin_did();
    let manifest = manifest_with_full_envelope();

    // "Install time" derivation — frozen into the cap-store at the
    // moment the user signs the InstallRecord.
    let install_time_requires = manifest_requires_to_scope(&manifest, &did);
    let install_time_shares = manifest_shares_to_scope(&manifest, &did);

    // "Check time" derivation — re-computed at runtime when the
    // audit pipeline (or recovery path) needs to recover the
    // canonical scope strings from the manifest body.
    let check_time_requires = manifest_requires_to_scope(&manifest, &did);
    let check_time_shares = manifest_shares_to_scope(&manifest, &did);

    assert_eq!(
        install_time_requires, check_time_requires,
        "G27-D pim-13: install-time-derived requires scopes MUST equal \
         check-time-derived requires scopes byte-for-byte (purity contract)"
    );
    assert_eq!(
        install_time_shares, check_time_shares,
        "G27-D pim-13: install-time-derived shares scopes MUST equal \
         check-time-derived shares scopes byte-for-byte (purity contract)"
    );

    // Defense-in-depth: deriving via a different `plugin_did` produces
    // DIFFERENT output (the plugin-DID is in the canonical scope-
    // string, so a different DID input yields different output).
    let different_did = Did::from_string_unchecked("did:key:z6MkDifferent".to_string());
    let different_requires = manifest_requires_to_scope(&manifest, &different_did);
    assert_ne!(
        install_time_requires, different_requires,
        "G27-D: derivation is a function of (manifest, plugin_did) — different \
         plugin-DID input produces different output"
    );
}

/// G27-D: invocation order doesn't affect output. Multiple calls
/// interleaved with mutation of unrelated state (the test runtime
/// itself) produce identical output.
#[test]
fn manifest_aware_scope_derivation_is_referentially_transparent() {
    let did = plugin_did();
    let manifest = manifest_with_full_envelope();

    let first = manifest_requires_to_scope(&manifest, &did);

    // Burn some allocator state.
    let _waste: Vec<String> = (0..100).map(|i| format!("dummy-{i}")).collect();

    let second = manifest_requires_to_scope(&manifest, &did);

    assert_eq!(
        first, second,
        "G27-D: referentially transparent — repeated calls produce identical output"
    );
}
