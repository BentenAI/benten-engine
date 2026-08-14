//! S-3 closure — frozen const VALUE pins for `benten-caps`.
//!
//! The required `cargo-public-api` baseline records const TYPES only, so a
//! value-only rename of a hashed grant label or a widening of the chain-depth
//! bound below is an EMPTY baseline diff.

use benten_caps::DEFAULT_BATCH_BOUNDARY;
use benten_caps::grant::{CAPABILITY_GRANT_LABEL, GRANTED_TO_LABEL, REVOKED_AT_LABEL};
use benten_caps::manifest_envelope_chain_validation::MAX_CHAIN_DEPTH;
use benten_caps::manifest_scope::{PRIVATE_PREFIX, REQUIRES_PREFIX, SHARES_PREFIX};

// ---------------------------------------------------------------------------
// Group 1 — capability-grant graph vocabulary.
//
// WHAT BREAKS: these labels are hashed into grant-node CIDs and are matched by
// the revocation/authorization lookup paths. A rename silently changes grant
// CIDs AND can make existing grants unfindable (fail-open risk on revocation
// lookup).
// ---------------------------------------------------------------------------

#[test]
fn capability_grant_vocabulary_is_frozen() {
    assert_eq!(CAPABILITY_GRANT_LABEL, "system:CapabilityGrant");
    assert_eq!(GRANTED_TO_LABEL, "GRANTED_TO");
    assert_eq!(REVOKED_AT_LABEL, "REVOKED_AT");
}

// ---------------------------------------------------------------------------
// Group 2 — manifest scope prefixes.
//
// WHAT BREAKS: these prefixes partition the capability namespace. PRIVATE_PREFIX
// in particular gates the per-plugin private namespace — a change could make
// previously-private scopes fall outside the private partition.
// ---------------------------------------------------------------------------

#[test]
fn manifest_scope_prefixes_are_frozen() {
    assert_eq!(REQUIRES_PREFIX, "requires");
    assert_eq!(SHARES_PREFIX, "shares");
    assert_eq!(
        PRIVATE_PREFIX, "private",
        "private-namespace scope prefix — gates cross-plugin delegation refusal"
    );
}

// ---------------------------------------------------------------------------
// Group 3 — chain-validation depth bound (META #629).
// ---------------------------------------------------------------------------

#[test]
fn caps_bounds_are_frozen() {
    assert_eq!(
        MAX_CHAIN_DEPTH, 64,
        "manifest envelope chain-depth cap over untrusted input — widening re-opens \
         unbounded recursion"
    );
    assert_eq!(
        DEFAULT_BATCH_BOUNDARY, 100,
        "default ITERATE batch boundary"
    );
}
