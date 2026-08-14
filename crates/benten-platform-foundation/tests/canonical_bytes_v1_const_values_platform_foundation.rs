//! S-3 closure — frozen const VALUE pins for `benten-platform-foundation`.
//!
//! Scope note: the schema-compiler / admin-UI vocabulary arrays
//! (VOCAB_*, SCALAR_NAMES, NAV_CATEGORIES, INDEXEDDB_*, WINTERTC_*) are
//! deliberately NOT pinned — they are application-layer vocabulary designed to
//! grow additively, not v1 wire format. Only the untrusted-input bounds, the
//! sentinel and the private-namespace prefix are frozen here.

use benten_platform_foundation::admin_ui_v0::ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX;
use benten_platform_foundation::plugin_lifecycle::{
    MAX_GRANTED_CAP_BYTES, MAX_GRANTED_CAPS, MAX_PLUGIN_MANIFEST_BYTES,
    MAX_PLUGIN_MANIFEST_REQUIRES,
};
use benten_platform_foundation::plugin_manifest::{
    EDGE_COMPOSES, MANIFEST_CLOCK_NOT_INJECTED_SENTINEL, PROP_NODE_MANIFEST_CID,
};

// ---------------------------------------------------------------------------
// Group 1 — plugin-install decode bounds (META #629).
//
// WHAT BREAKS: a plugin manifest arrives from an Atrium peer or a shared
// bundle — it is untrusted input. These caps bound the decode and the
// consent-list construction. Widening them re-opens allocation DoS at the
// install boundary.
// ---------------------------------------------------------------------------

#[test]
fn plugin_install_decode_bounds_are_frozen() {
    assert_eq!(
        MAX_PLUGIN_MANIFEST_BYTES,
        256 * 1024,
        "plugin-manifest byte cap over untrusted input"
    );
    assert_eq!(MAX_PLUGIN_MANIFEST_BYTES, 262_144, "literal value");
    assert_eq!(
        MAX_PLUGIN_MANIFEST_REQUIRES, 4_096,
        "requires-entry count cap — bounds an input-driven loop"
    );
    assert_eq!(MAX_GRANTED_CAPS, 4_096, "granted-capability count cap");
    assert_eq!(
        MAX_GRANTED_CAP_BYTES,
        64 * 1024,
        "per-granted-capability byte cap"
    );
    assert_eq!(MAX_GRANTED_CAP_BYTES, 65_536, "literal value");
}

// ---------------------------------------------------------------------------
// Group 2 — install-time sentinel + composition vocabulary.
//
// WHAT BREAKS: MANIFEST_CLOCK_NOT_INJECTED_SENTINEL is the fail-closed marker
// for "no clock was injected"; changing it could make a real timestamp collide
// with the sentinel (or make the sentinel unreachable), silently altering
// install-time validity checks. EDGE_COMPOSES / PROP_NODE_MANIFEST_CID are
// hashed into composition-walk subgraph CIDs.
// ---------------------------------------------------------------------------

#[test]
fn manifest_sentinel_and_composition_vocabulary_are_frozen() {
    assert_eq!(
        MANIFEST_CLOCK_NOT_INJECTED_SENTINEL, 0,
        "fail-closed 'clock not injected' sentinel"
    );
    assert_eq!(EDGE_COMPOSES, "COMPOSES");
    assert_eq!(PROP_NODE_MANIFEST_CID, "manifest_cid");
}

// ---------------------------------------------------------------------------
// Group 3 — admin-UI private namespace.
//
// WHAT BREAKS: this prefix scopes the admin UI's private data. A change could
// move previously-private data outside the private partition.
// ---------------------------------------------------------------------------

#[test]
fn admin_ui_private_namespace_prefix_is_frozen() {
    assert_eq!(
        ADMIN_UI_V0_PRIVATE_NAMESPACE_PREFIX, "private:admin-ui-v0",
        "admin-UI private-namespace scope prefix"
    );
}
