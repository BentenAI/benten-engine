//! Phase-4-Foundation G27-D — manifest-aware scope derivation.
//!
//! Per CLAUDE.md baked-in #18 layer-(c) (runtime delegation within
//! manifest envelope) + plugin-arch-r1-10 (manifest scope-string
//! grammar pin) + `docs/future/phase-4-backlog.md` §4.4 (cap-r1-3
//! closure: manifest scope grammar at G27-D).
//!
//! # What this module owns
//!
//! Maps a `PluginManifest`'s `requires` + `shares` halves into the
//! canonical cap-scope-string grammar consumed by `GrantBackedPolicy`:
//!
//! - `private:<plugin_did>:<resource>` — private-namespace caps. The
//!   `<plugin_did>` segment is the namespace owner; only the owning
//!   plugin (whose `actor_cid` resolves to that DID) may ever use the
//!   cap, and never delegate it (the unconditional cross-plugin deny
//!   lives in `plugin_delegation::is_private_namespace_cap`).
//! - `requires:<plugin_did>:<requirement_path>` — caps the plugin
//!   declares it needs to function (from `manifest.requires`).
//! - `shares:<plugin_did>:<share_path>` — caps the plugin's manifest
//!   `shares` policy declares it MAY re-delegate to other plugins.
//!
//! # Install-time vs check-time decision
//!
//! Per cap-r1-3 closure (`docs/future/phase-4-backlog.md` §4.4), the
//! manifest's `requires` half is derived **at install time** — the
//! moment the user signs the `InstallRecord` consenting to the
//! envelope. The derived scope strings are then frozen as the cap-
//! lookup keys for the lifetime of that plugin-DID's installation;
//! the derivation function is pure (depends only on the immutable
//! `(manifest_cid, plugin_did)` pair), so re-deriving at check time
//! produces the same strings.
//!
//! **Install-time is preferred** for performance: the `GrantReader`
//! lookups at write-time consult `ctx.scope` directly (single hash-map
//! probe per scope) rather than re-walking the manifest's `requires`
//! list. The check-time path remains the consistency oracle — the
//! `manifest_aware_scope_install_time_vs_check_time_consistency`
//! integration pin asserts that install-time-derived scopes match
//! re-derivation byte-for-byte for the same `(manifest, plugin_did)`
//! input (pim-13-style spec-to-code-compliance invariant).
//!
//! # `shares` envelope check (audience-side write path)
//!
//! When plugin A delegates a UCAN cap to plugin B for a scope, the
//! audience-side write fires `GrantBackedPolicy::check_write` with
//! `ctx.scope = "store:<resource>:write"` (the concrete cap, NOT the
//! manifest-prefixed `shares:<plugin_a>:*` form). The envelope check
//! is therefore a SECOND step: after the grant lookup succeeds (the
//! source plugin holds the cap), verify the scope is permitted by
//! the source manifest's `shares` policy via
//! [`check_scope_within_envelope`]. The check fails CLOSED if the
//! manifest's `shares.default == None` and no matching rule fires —
//! per CLAUDE.md #18 layer-(c) "the manifest envelope IS the consent".
//!
//! # Coupled crates
//!
//! - `benten_platform_foundation::{PluginManifest, CapRequirement,
//!   SharesPolicy}` — the manifest schema landed at G24-D.
//! - `benten_caps::plugin_delegation` — the runtime delegation gate
//!   (G24-D); this module's [`check_scope_within_envelope`] is the
//!   sibling check on the audience-side write path.
//! - `benten_id::Did` — the plugin-DID identity used to prefix
//!   manifest-derived scope strings.
//!
//! # Why a separate module (not extending `plugin_delegation.rs`)
//!
//! `plugin_delegation.rs` owns the ISSUANCE-side check (source plugin
//! issues a UCAN to target plugin → permit/deny based on source's
//! `shares` policy). This module owns the AUDIENCE-side check (audience
//! plugin's write fires with a concrete scope → verify scope is within
//! source's `shares` envelope) PLUS the install-time scope-string
//! derivation. The two checks operate at different points in the cap-
//! lifecycle; co-locating them blurs the install-time-vs-check-time
//! seam this module documents.

#[cfg(not(target_arch = "wasm32"))]
use benten_id::did::Did;

#[cfg(not(target_arch = "wasm32"))]
use benten_platform_foundation::{PluginManifest, SharesPolicy, SharesPolicyDefault};

#[cfg(not(target_arch = "wasm32"))]
use crate::error::CapError;

/// Canonical scope-string prefix for caps the plugin declares it
/// needs to function (`manifest.requires` half).
pub const REQUIRES_PREFIX: &str = "requires";

/// Canonical scope-string prefix for caps the plugin's manifest
/// declares it MAY re-delegate (`manifest.shares` half).
pub const SHARES_PREFIX: &str = "shares";

/// Canonical scope-string prefix for private-namespace caps. The
/// segment after the prefix is the owning `<plugin_did>`; only the
/// owning plugin may ever use a cap with this prefix, and never
/// delegate it cross-plugin.
pub const PRIVATE_PREFIX: &str = "private";

/// Derive the canonical scope-string set from a manifest's `requires`
/// half for a given plugin-DID.
///
/// Per plugin-arch-r1-10 grammar:
///
/// - `private:<scope_path>` requirements → emitted verbatim if the
///   first segment after `private:` already matches `<plugin_did>`;
///   otherwise the plugin-DID segment is INSERTED. This preserves the
///   `private:<plugin_did>:<resource>` invariant that the cross-plugin
///   deny check (`plugin_delegation::is_private_namespace_cap`)
///   keys off.
/// - All other requirements → wrapped as
///   `requires:<plugin_did>:<original_scope>` for manifest-attribution.
///   This shape lets the audit pipeline distinguish caps the plugin
///   declared it needs (via manifest) from caps issued ad-hoc.
///
/// # Returns
///
/// A `Vec<String>` of canonical scope strings, one per `CapRequirement`
/// in the manifest. Order matches the manifest's declaration order so
/// downstream cap-store seeding is deterministic.
///
/// # Install-time semantics
///
/// Per cap-r1-3 closure (module-doc above): this function is the
/// install-time derivation that freezes the scope strings into the
/// cap-lookup table. Re-derivation at check time produces identical
/// output (pure function over `(manifest_requires, plugin_did)`); the
/// `manifest_aware_scope_install_time_vs_check_time_consistency`
/// integration pin asserts that invariant.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn manifest_requires_to_scope(manifest: &PluginManifest, plugin_did: &Did) -> Vec<String> {
    let mut out = Vec::with_capacity(manifest.requires.len());
    let did_str = plugin_did.as_str();
    for req in &manifest.requires {
        if let Some(rest) = req.scope.strip_prefix("private:") {
            // Detect whether the manifest author already wrote
            // `private:<did>:<resource>` (canonical) vs. the bare
            // `private:<resource>` shorthand. Canonical form is
            // emitted verbatim; bare form has `<plugin_did>` spliced
            // in as the namespace-owner segment.
            //
            // Distinguishing rule: if the first colon-segment in
            // `rest` looks like a DID (`did:*` prefix), assume
            // canonical and pass through; otherwise splice. This
            // mirrors the canonicalization rule consumed by
            // `private_namespace_scope_string_requires_plugin_did_segment_match_requester`
            // (T7 pin) — the cap-policy check rejects mismatched
            // plugin-DID segments structurally.
            if rest.starts_with("did:") {
                out.push(req.scope.clone());
            } else {
                out.push(format!("{PRIVATE_PREFIX}:{did_str}:{rest}"));
            }
        } else {
            out.push(format!("{REQUIRES_PREFIX}:{did_str}:{}", req.scope));
        }
    }
    out
}

/// Derive the canonical `shares`-prefixed scope-string set from a
/// manifest's `shares.rules` half for a given plugin-DID.
///
/// For each `SharesRule` in `manifest.shares.rules`, emit
/// `shares:<plugin_did>:<cap_pattern>`. Manifests with
/// `shares.default == None` and no rules yield an empty vec —
/// audit-surface for "plugin delegates nothing" intent.
///
/// # Why this is separate from `manifest_requires_to_scope`
///
/// The `requires` half declares what the plugin NEEDS to function;
/// the `shares` half declares what the plugin MAY DELEGATE. The two
/// halves drive different cap-lookup tables — `requires` populates
/// the plugin-DID's grant store; `shares` populates the delegation
/// envelope consulted at runtime by [`check_scope_within_envelope`].
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn manifest_shares_to_scope(manifest: &PluginManifest, plugin_did: &Did) -> Vec<String> {
    let Some(rules) = &manifest.shares.rules else {
        return Vec::new();
    };
    let did_str = plugin_did.as_str();
    rules
        .iter()
        .map(|rule| format!("{SHARES_PREFIX}:{did_str}:{}", rule.cap_pattern))
        .collect()
}

/// Check whether a concrete cap-scope is within the source manifest's
/// `shares` envelope.
///
/// Semantics — CLAUDE.md #18 layer-(c) audience-side write path:
///
/// 1. **Private-namespace caps short-circuit DENY.** A scope starting
///    with `private:` is NEVER within any manifest's `shares`
///    envelope, regardless of policy default. The cross-plugin deny
///    is structural; this check mirrors
///    `plugin_delegation::is_private_namespace_cap` at the audience-
///    side path.
/// 2. **`SharesPolicyDefault::None` + no matching rule → DENY** with
///    [`CapError::Denied`] carrying `code() ==
///    PluginDelegationOutsideManifestEnvelope`. This is the LOAD-
///    BEARING half of the manifest envelope — without it, the
///    manifest's `shares: None` is a paper guarantee.
/// 3. **`SharesPolicyDefault::Any` → PERMIT.** (Used by trust-anchor
///    plugins; rare in v0.)
/// 4. **`SharesPolicyDefault::Matching` + rule matches → PERMIT.**
///    Rule matching delegates to `SharesPolicy::permits_delegation`,
///    which handles the cap-pattern glob (`store:notes:*`) +
///    target-DID match.
///
/// # `target_plugin_did`
///
/// The audience plugin-DID — the principal whose `actor_cid` is in
/// the inbound `CapWriteContext`. The check verifies whether the source
/// manifest's `shares` policy permits delegating `cap_scope` to this
/// specific audience.
///
/// # Errors
///
/// - [`CapError::Denied`] with `required = cap_scope` when the scope
///   exceeds the manifest envelope. The error's stable
///   [`benten_errors::ErrorCode::PluginDelegationOutsideManifestEnvelope`]
///   (via the engine-side typed-carrier mapping) surfaces in audit
///   logs as `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` per
///   arch-r1-3.
/// - [`CapError::Denied`] with `code() ==
///   PluginPrivateNamespaceDelegationForbidden` when `cap_scope` is
///   `private:*` shape (the cross-plugin private-namespace deny).
///   The narrower code is surfaced at the engine boundary; at the
///   `CapError` level we emit `Denied` with the typed error-code
///   reachable via the layered carrier (callers re-classify via
///   `EngineError::from_cap_error` mappings).
#[cfg(not(target_arch = "wasm32"))]
pub fn check_scope_within_envelope(
    cap_scope: &str,
    target_plugin_did: &Did,
    source_manifest: &PluginManifest,
) -> Result<(), CapError> {
    // Private-namespace caps are NEVER delegable cross-plugin — the
    // cross-plugin deny is structural. Delegates to
    // `plugin_delegation::is_private_namespace_cap` (same-crate sibling)
    // so the private-NS shape lives in exactly one source location.
    if crate::plugin_delegation::is_private_namespace_cap(cap_scope) {
        return Err(CapError::Denied {
            required: cap_scope.to_string(),
            entity: target_plugin_did.as_str().to_string(),
        });
    }
    if source_manifest
        .shares
        .permits_delegation(cap_scope, target_plugin_did)
    {
        Ok(())
    } else {
        Err(CapError::Denied {
            required: cap_scope.to_string(),
            entity: target_plugin_did.as_str().to_string(),
        })
    }
}

/// Whether `cap_scope` is `private:<plugin_did>:*` shaped AND the
/// `<plugin_did>` segment matches the owning plugin-DID.
///
/// Used by the cap-policy audience-side check to verify that a
/// `private:` cap is being used by its OWNING plugin (not by some
/// cross-plugin replay). The structural shape is:
///
///   `private:<plugin_did>:<resource>...`
///
/// where `<plugin_did>` is a DID (starts with `did:`) and matches
/// `owning_plugin_did` byte-for-byte.
///
/// Returns `false` for:
/// - Scopes not starting with `private:` (not a private-namespace cap).
/// - Scopes with a malformed plugin-DID segment.
/// - Scopes whose `<plugin_did>` segment is a different DID.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn private_namespace_scope_admits_actor(cap_scope: &str, owning_plugin_did: &Did) -> bool {
    let Some(rest) = cap_scope.strip_prefix("private:") else {
        return false;
    };
    // The segment up to the next `:` is the owning plugin-DID.
    // DIDs themselves contain `:` (e.g. `did:key:z6Mk...`), so we
    // can't naively split on the first colon. Match the DID prefix
    // explicitly: the rest of the string must START with the
    // owning_plugin_did's full string + a `:` separator (or end-of-
    // string, for the bare `private:<did>` shape with no resource
    // suffix).
    let owner_str = owning_plugin_did.as_str();
    if rest == owner_str {
        return true;
    }
    if let Some(after) = rest.strip_prefix(owner_str) {
        return after.starts_with(':');
    }
    false
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use benten_platform_foundation::{
        CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault, SharesRule, SharesTarget,
    };

    fn plugin_did_alpha() -> Did {
        Did::from_string_unchecked("did:key:z6MkAlpha".to_string())
    }

    fn plugin_did_beta() -> Did {
        Did::from_string_unchecked("did:key:z6MkBeta".to_string())
    }

    fn manifest_with(requires: Vec<&str>, shares: SharesPolicy) -> PluginManifest {
        PluginManifest {
            plugin_name: "test".to_string(),
            content_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
            peer_did: plugin_did_alpha(),
            peer_signature: vec![0u8; 64],
            requires: requires
                .into_iter()
                .map(|s| CapRequirement {
                    scope: s.to_string(),
                })
                .collect(),
            shares,
            renderer_config: None,
            composes_plugins: None,
            accepts_content: None,
            requires_schema_authors: None,
            requires_plugin_authors: None,
        }
    }

    #[test]
    fn requires_to_scope_wraps_non_private_in_requires_prefix() {
        let m = manifest_with(
            vec!["store:notes:read"],
            SharesPolicy {
                default: SharesPolicyDefault::None,
                rules: None,
            },
        );
        let scopes = manifest_requires_to_scope(&m, &plugin_did_alpha());
        assert_eq!(scopes, vec!["requires:did:key:z6MkAlpha:store:notes:read"]);
    }

    #[test]
    fn requires_to_scope_splices_plugin_did_into_bare_private() {
        // Bare `private:<resource>` shorthand → splice plugin-DID.
        let m = manifest_with(
            vec!["private:admin-ui-private:scratch"],
            SharesPolicy {
                default: SharesPolicyDefault::None,
                rules: None,
            },
        );
        let scopes = manifest_requires_to_scope(&m, &plugin_did_alpha());
        assert_eq!(
            scopes,
            vec!["private:did:key:z6MkAlpha:admin-ui-private:scratch"]
        );
    }

    #[test]
    fn requires_to_scope_preserves_canonical_private_did() {
        // Canonical `private:did:<scheme>:<id>:<resource>` → preserved.
        let canonical = "private:did:key:z6MkAlpha:admin-ui-private:scratch";
        let m = manifest_with(
            vec![canonical],
            SharesPolicy {
                default: SharesPolicyDefault::None,
                rules: None,
            },
        );
        let scopes = manifest_requires_to_scope(&m, &plugin_did_alpha());
        assert_eq!(scopes, vec![canonical.to_string()]);
    }

    #[test]
    fn shares_to_scope_wraps_rule_patterns_in_shares_prefix() {
        let m = manifest_with(
            vec!["store:notes:read"],
            SharesPolicy {
                default: SharesPolicyDefault::Matching,
                rules: Some(vec![SharesRule {
                    cap_pattern: "store:notes:write".to_string(),
                    target: SharesTarget::Any,
                }]),
            },
        );
        let scopes = manifest_shares_to_scope(&m, &plugin_did_alpha());
        assert_eq!(scopes, vec!["shares:did:key:z6MkAlpha:store:notes:write"]);
    }

    #[test]
    fn shares_to_scope_empty_for_no_rules() {
        let m = manifest_with(
            vec!["store:notes:read"],
            SharesPolicy {
                default: SharesPolicyDefault::None,
                rules: None,
            },
        );
        let scopes = manifest_shares_to_scope(&m, &plugin_did_alpha());
        assert!(scopes.is_empty());
    }

    #[test]
    fn check_envelope_denies_private_namespace_regardless_of_policy() {
        let m = manifest_with(
            vec!["store:notes:read"],
            SharesPolicy {
                default: SharesPolicyDefault::Any,
                rules: None,
            },
        );
        let result = check_scope_within_envelope(
            "private:did:key:z6MkAlpha:scratch",
            &plugin_did_beta(),
            &m,
        );
        assert!(matches!(result, Err(CapError::Denied { .. })));
    }

    #[test]
    fn check_envelope_denies_when_shares_none_default() {
        let m = manifest_with(vec!["store:notes:write"], SharesPolicy::none());
        let result = check_scope_within_envelope("store:notes:write", &plugin_did_beta(), &m);
        assert!(matches!(result, Err(CapError::Denied { .. })));
    }

    #[test]
    fn check_envelope_permits_when_shares_any_default() {
        let m = manifest_with(
            vec!["store:notes:write"],
            SharesPolicy {
                default: SharesPolicyDefault::Any,
                rules: None,
            },
        );
        check_scope_within_envelope("store:notes:write", &plugin_did_beta(), &m).unwrap();
    }

    #[test]
    fn check_envelope_permits_when_matching_rule_fires() {
        let m = manifest_with(
            vec!["store:notes:write"],
            SharesPolicy {
                default: SharesPolicyDefault::Matching,
                rules: Some(vec![SharesRule {
                    cap_pattern: "store:notes:write".to_string(),
                    target: SharesTarget::PluginDid(plugin_did_beta()),
                }]),
            },
        );
        check_scope_within_envelope("store:notes:write", &plugin_did_beta(), &m).unwrap();
    }

    #[test]
    fn check_envelope_denies_when_rule_targets_different_plugin() {
        let m = manifest_with(
            vec!["store:notes:write"],
            SharesPolicy {
                default: SharesPolicyDefault::Matching,
                rules: Some(vec![SharesRule {
                    cap_pattern: "store:notes:write".to_string(),
                    target: SharesTarget::PluginDid(plugin_did_alpha()), // not beta
                }]),
            },
        );
        let result = check_scope_within_envelope("store:notes:write", &plugin_did_beta(), &m);
        assert!(matches!(result, Err(CapError::Denied { .. })));
    }

    #[test]
    fn private_namespace_scope_admits_only_owning_plugin_did() {
        // Owner admitted.
        assert!(private_namespace_scope_admits_actor(
            "private:did:key:z6MkAlpha:scratch",
            &plugin_did_alpha()
        ));
        // Different DID rejected.
        assert!(!private_namespace_scope_admits_actor(
            "private:did:key:z6MkAlpha:scratch",
            &plugin_did_beta()
        ));
        // Non-private scope rejected (wrong prefix).
        assert!(!private_namespace_scope_admits_actor(
            "store:notes:read",
            &plugin_did_alpha()
        ));
        // Bare `private:<did>` (no resource suffix) admitted for owner.
        assert!(private_namespace_scope_admits_actor(
            "private:did:key:z6MkAlpha",
            &plugin_did_alpha()
        ));
    }

    #[test]
    fn install_time_vs_check_time_derivation_consistency() {
        // pim-13-style spec-to-code-compliance: derivation is a pure
        // function; install-time output equals re-derivation output.
        let m = manifest_with(
            vec![
                "store:notes:read",
                "private:admin-private:scratch",
                "host:time:now",
            ],
            SharesPolicy {
                default: SharesPolicyDefault::Matching,
                rules: Some(vec![SharesRule {
                    cap_pattern: "store:notes:write".to_string(),
                    target: SharesTarget::Any,
                }]),
            },
        );
        let install_time_requires = manifest_requires_to_scope(&m, &plugin_did_alpha());
        let install_time_shares = manifest_shares_to_scope(&m, &plugin_did_alpha());

        // Re-derive at "check time" — same inputs, must produce same
        // outputs byte-for-byte.
        let check_time_requires = manifest_requires_to_scope(&m, &plugin_did_alpha());
        let check_time_shares = manifest_shares_to_scope(&m, &plugin_did_alpha());

        assert_eq!(install_time_requires, check_time_requires);
        assert_eq!(install_time_shares, check_time_shares);
    }
}
