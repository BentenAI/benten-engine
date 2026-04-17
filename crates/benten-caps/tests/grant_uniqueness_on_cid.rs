//! Edge-case test: capability grants are content-addressed. Two grants with
//! byte-identical scope + entity + issuer content must have the same CID;
//! storing the "second" grant is a no-op, not a duplicate.
//!
//! Same boundary applies inversely: a grant that differs by a single byte
//! (new scope, new entity, different expiry) must have a DIFFERENT CID.
//! The "honest no" here is "you thought you granted a new capability, but
//! this one already exists — here's its CID."
//!
//! R3 contract: `CapabilityGrant` and `NoAuthBackend` are stubs today
//! (`benten-caps/src/lib.rs` is a single STUB_MARKER). R5 (G4-A) ships them.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapabilityGrant, GrantScope};
use benten_core::{Cid, Node};

fn entity_cid(name: &str) -> Cid {
    let mut node = Node::new(vec!["Entity".into()], Default::default());
    node.properties
        .insert("name".into(), benten_core::Value::text(name));
    node.cid().unwrap()
}

#[test]
fn identical_grants_hash_to_identical_cid() {
    // Two separately-constructed grants with byte-identical content. The
    // spike's content-addressing contract says their CIDs must match —
    // storing the second is a deduplicated no-op.
    let entity = entity_cid("alice");
    let issuer = entity_cid("root");

    let a = CapabilityGrant::new(
        entity.clone(),
        issuer.clone(),
        GrantScope::parse("store:post:write").unwrap(),
    );
    let b = CapabilityGrant::new(
        entity,
        issuer,
        GrantScope::parse("store:post:write").unwrap(),
    );

    let cid_a = a.cid().unwrap();
    let cid_b = b.cid().unwrap();
    assert_eq!(cid_a, cid_b, "byte-identical grants must share a CID");
}

#[test]
fn grant_scope_difference_changes_cid() {
    let entity = entity_cid("alice");
    let issuer = entity_cid("root");

    let a = CapabilityGrant::new(
        entity.clone(),
        issuer.clone(),
        GrantScope::parse("store:post:write").unwrap(),
    );
    let b = CapabilityGrant::new(
        entity,
        issuer,
        GrantScope::parse("store:post:read").unwrap(), // write vs read
    );

    assert_ne!(
        a.cid().unwrap(),
        b.cid().unwrap(),
        "scope difference must change the grant CID"
    );
}

#[test]
fn grant_entity_difference_changes_cid() {
    let issuer = entity_cid("root");
    let scope = GrantScope::parse("store:post:write").unwrap();

    let a = CapabilityGrant::new(entity_cid("alice"), issuer.clone(), scope.clone());
    let b = CapabilityGrant::new(entity_cid("bob"), issuer, scope);

    assert_ne!(
        a.cid().unwrap(),
        b.cid().unwrap(),
        "different grantee must produce a different grant CID"
    );
}

#[test]
fn grant_issuer_difference_changes_cid() {
    // Sharp edge: same scope, same grantee, different issuer -> DIFFERENT CID.
    // This is load-bearing for UCAN-style attenuation chains where the
    // issuer is a critical part of the grant identity.
    let entity = entity_cid("alice");
    let scope = GrantScope::parse("store:post:write").unwrap();

    let a = CapabilityGrant::new(entity.clone(), entity_cid("root"), scope.clone());
    let b = CapabilityGrant::new(entity, entity_cid("admin"), scope);

    assert_ne!(
        a.cid().unwrap(),
        b.cid().unwrap(),
        "different issuer must produce a different grant CID"
    );
}

#[test]
fn grant_empty_scope_rejected_at_construction() {
    // Degenerate input: an empty scope string is not a valid grant — it
    // would permit nothing, which is indistinguishable from no grant at
    // all. Refuse at construction.
    assert!(
        GrantScope::parse("").is_err(),
        "empty scope must be refused at parse"
    );
}

#[test]
fn grant_whitespace_only_scope_rejected() {
    // Degenerate input: whitespace-only scopes must also fail; they parse
    // as "empty" after trimming and so carry no authority.
    assert!(GrantScope::parse("   ").is_err());
    assert!(GrantScope::parse("\t\n").is_err());
}

// ---------------------------------------------------------------------------
// g4-p2-uc-4 — empty-segment parse rejection.
//
// Scopes with empty inner / leading / trailing segments are an
// encoding-trick surface: an attacker can produce a distinct-CID scope that
// attenuates identically to a hand-written scope the victim already trusts,
// while presenting nearly-identical glyphs to a human reviewer. Reject at
// parse.
// ---------------------------------------------------------------------------

#[test]
fn parse_rejects_empty_inner_segment() {
    // `"store::write"` — split(':') produces ["store", "", "write"]; the
    // middle empty segment must fail.
    assert!(GrantScope::parse("store::write").is_err());
}

#[test]
fn parse_rejects_leading_colon() {
    // `":store:write"` — produces ["", "store", "write"]; leading empty
    // segment must fail.
    assert!(GrantScope::parse(":store:write").is_err());
}

#[test]
fn parse_rejects_trailing_colon() {
    // `"store:write:"` — produces ["store", "write", ""]; trailing empty
    // segment must fail.
    assert!(GrantScope::parse("store:write:").is_err());
}

#[test]
fn parse_rejects_all_colons() {
    // `":::"` — every segment empty. Degenerate input; must fail.
    assert!(GrantScope::parse(":::").is_err());
}
