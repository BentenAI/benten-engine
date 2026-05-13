//! G27-D — private-namespace scope admits ONLY the plugin-DID actor.
//!
//! ## Pin source
//!
//! Brief §scope point 2 + CLAUDE.md #18 "Private namespaces"
//! paragraph (plugins saving their own private data) + plugin-arch-
//! r1-10 (manifest scope-string grammar pin) +
//! `docs/future/phase-4-backlog.md` §4.4 — `private:<plugin_did>:*`
//! interaction with `wildcard_variants`.
//!
//! ## What this pin verifies
//!
//! Private-namespace scope strings (`private:<plugin_did>:<resource>`)
//! are SHAPED — they encode the owning plugin-DID in the scope itself.
//! Cross-plugin reads/writes into private namespaces are denied
//! STRUCTURALLY by scope-shape, not by policy alone:
//!
//! 1. `manifest_scope::private_namespace_scope_admits_actor(scope, owner)`
//!    returns `true` ONLY when the scope's `<plugin_did>` segment
//!    matches `owner` byte-for-byte.
//! 2. A cross-plugin actor (different plugin-DID) trying to use the
//!    cap fails the structural check FIRST, before any UCAN-chain
//!    validation runs. This defends against the "delegated UCAN
//!    chain admits cross-plugin private access" scenario even if a
//!    misbehaving cap-policy were to permit it.
//! 3. The companion `plugin_delegation::is_private_namespace_cap`
//!    check at the DELEGATION-issuance side prevents the cap from
//!    ever being delegated in the first place. The two defenses
//!    layer.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer ships `private_namespace_scope_admits_actor` that
//! returns `true` for any private:-prefixed scope (ignores
//! `<plugin_did>` segment). Cross-plugin actor admitted → user's
//! private-namespace data leaked. This pin's `assert!(!...)`
//! assertion flips.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::manifest_scope::private_namespace_scope_admits_actor;
use benten_id::did::Did;

fn plugin_a() -> Did {
    Did::from_string_unchecked("did:key:z6MkPluginAlpha".to_string())
}

fn plugin_b() -> Did {
    Did::from_string_unchecked("did:key:z6MkPluginBeta".to_string())
}

/// G27-D: private-namespace scope admits ONLY its owning plugin-DID.
#[test]
fn private_namespace_scope_admits_only_plugin_did_actor() {
    let scope = "private:did:key:z6MkPluginAlpha:admin-private:scratch";

    // Owner admitted (structural match on plugin-DID segment).
    assert!(
        private_namespace_scope_admits_actor(scope, &plugin_a()),
        "G27-D: private-namespace scope MUST admit the owning plugin-DID"
    );

    // Cross-plugin actor REJECTED (structural mismatch).
    assert!(
        !private_namespace_scope_admits_actor(scope, &plugin_b()),
        "G27-D LOAD-BEARING: private-namespace scope MUST reject cross-plugin actor \
         (defense against private-namespace leak even if cap-policy were to permit)"
    );
}

/// G27-D: bare `private:<plugin_did>` (no resource suffix) also
/// admits only the owning plugin-DID.
#[test]
fn bare_private_namespace_scope_admits_only_owning_plugin_did() {
    let scope = "private:did:key:z6MkPluginAlpha";

    assert!(
        private_namespace_scope_admits_actor(scope, &plugin_a()),
        "bare private:<did> shape admits owner"
    );
    assert!(
        !private_namespace_scope_admits_actor(scope, &plugin_b()),
        "bare private:<did> shape rejects cross-plugin actor"
    );
}

/// G27-D: non-private scopes return `false` for the admits check —
/// the helper is private-namespace-specific.
#[test]
fn admits_actor_returns_false_for_non_private_scopes() {
    // Plain CRUD scope.
    assert!(!private_namespace_scope_admits_actor(
        "store:notes:write",
        &plugin_a()
    ));
    // Manifest-requires-prefixed scope (G27-D grammar).
    assert!(!private_namespace_scope_admits_actor(
        "requires:did:key:z6MkPluginAlpha:store:notes:read",
        &plugin_a()
    ));
    // Manifest-shares-prefixed scope.
    assert!(!private_namespace_scope_admits_actor(
        "shares:did:key:z6MkPluginAlpha:store:notes:write",
        &plugin_a()
    ));
    // Empty string.
    assert!(!private_namespace_scope_admits_actor("", &plugin_a()));
}

/// G27-D: DID-prefix collision defense — a scope encoding a DID that
/// is a PREFIX of another DID still rejects the longer-DID actor.
/// E.g. `private:did:key:z6MkPluginAlpha:...` must NOT admit
/// `did:key:z6MkPluginAlphaExtended` (longer DID with same prefix).
#[test]
fn admits_actor_defends_did_prefix_collisions() {
    let scope = "private:did:key:z6MkPluginAlpha:resource";
    let prefix_collision_did =
        Did::from_string_unchecked("did:key:z6MkPluginAlphaExtended".to_string());

    assert!(
        !private_namespace_scope_admits_actor(scope, &prefix_collision_did),
        "G27-D: DID-prefix collision MUST NOT admit — the helper checks for full \
         DID equality terminated by `:` or end-of-string, not prefix match"
    );

    // Owner still admitted.
    assert!(private_namespace_scope_admits_actor(scope, &plugin_a()));
}
