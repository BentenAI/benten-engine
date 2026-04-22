//! Edge-case tests: `SubgraphCacheKey` includes `subgraph_cid` (arch-r1-5).
//!
//! R2 landscape §2.6.2 row "`SubgraphCacheKey` includes `subgraph_cid`".
//!
//! The cache key is `(handler_id, op, subgraph_cid)`. Two distinct
//! subgraph_cid values for the same `(handler_id, op)` must NOT collide in
//! the cache — distinct entries are produced.
//!
//! Why this matters as an edge case:
//! - arch-r1-5 raised the concern that Phase 1's cache was keyed only by
//!   `(handler_id, op)`, so a re-registration with a new CID could either
//!   serve a stale AST or silently overwrite the prior entry without a
//!   miss-then-parse cycle.
//! - Phase 2a's key includes `subgraph_cid` explicitly so the two cases
//!   are forced to be separate entries.
//!
//! Concerns pinned:
//! - Two subgraphs with identical `(handler_id, op)` but different CIDs
//!   produce two distinct cache entries.
//! - The cache key type derives `Eq + Hash + Ord` correctly (equality is
//!   structural, no stray fields).
//! - A cache hit requires ALL THREE axes to match — changing any single
//!   axis must miss.
//!
//! R3 red-phase contract: R5 (G2-B) lands the struct. Tests compile; they
//! fail because `SubgraphCacheKey` does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_engine::SubgraphCacheKey;
use std::collections::HashMap;

#[test]
fn subgraph_cache_key_includes_subgraph_cid_distinct_entries() {
    // Same handler_id + op, different subgraph_cid → distinct keys.
    let cid_a = Cid::from_blake3_digest([0x01; 32]);
    let cid_b = Cid::from_blake3_digest([0x02; 32]);

    let key_a = SubgraphCacheKey::new("h1".to_string(), "run".to_string(), cid_a);
    let key_b = SubgraphCacheKey::new("h1".to_string(), "run".to_string(), cid_b);

    assert_ne!(
        key_a, key_b,
        "distinct subgraph_cid must yield distinct cache keys for same (handler_id, op)"
    );

    let mut cache: HashMap<SubgraphCacheKey, &'static str> = HashMap::new();
    cache.insert(key_a.clone(), "ast_for_a");
    cache.insert(key_b.clone(), "ast_for_b");
    assert_eq!(cache.len(), 2, "two distinct keys must yield two entries");
    assert_eq!(cache.get(&key_a), Some(&"ast_for_a"));
    assert_eq!(cache.get(&key_b), Some(&"ast_for_b"));
}

#[test]
fn subgraph_cache_key_eq_requires_all_three_axes_match() {
    let cid = Cid::from_blake3_digest([0x99; 32]);
    let base = SubgraphCacheKey::new("h1".into(), "run".into(), cid);

    // Different handler_id.
    assert_ne!(base, SubgraphCacheKey::new("h2".into(), "run".into(), cid));
    // Different op.
    assert_ne!(
        base,
        SubgraphCacheKey::new("h1".into(), "delete".into(), cid)
    );
    // Different cid.
    let cid2 = Cid::from_blake3_digest([0xaa; 32]);
    assert_ne!(base, SubgraphCacheKey::new("h1".into(), "run".into(), cid2));

    // All-same is equal.
    assert_eq!(base, SubgraphCacheKey::new("h1".into(), "run".into(), cid));
}

#[test]
fn subgraph_cache_key_hash_is_stable_across_identical_constructions() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let cid = Cid::from_blake3_digest([0x11; 32]);
    let k1 = SubgraphCacheKey::new("h1".into(), "run".into(), cid);
    let k2 = SubgraphCacheKey::new("h1".into(), "run".into(), cid);

    let mut h1 = DefaultHasher::new();
    k1.hash(&mut h1);
    let mut h2 = DefaultHasher::new();
    k2.hash(&mut h2);
    assert_eq!(
        h1.finish(),
        h2.finish(),
        "hash must be stable across identical constructions"
    );
}

#[test]
fn subgraph_cache_key_different_cids_hash_differ_with_high_probability() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let cid_a = Cid::from_blake3_digest([0x01; 32]);
    let cid_b = Cid::from_blake3_digest([0x02; 32]);

    let k_a = SubgraphCacheKey::new("h1".into(), "run".into(), cid_a);
    let k_b = SubgraphCacheKey::new("h1".into(), "run".into(), cid_b);

    let mut ha = DefaultHasher::new();
    k_a.hash(&mut ha);
    let mut hb = DefaultHasher::new();
    k_b.hash(&mut hb);

    // Not a cryptographic guarantee — just a sanity check that the cid is
    // folded into the hash at all (the collision probability under the
    // default hasher is ~1/2^64 per disjoint CID).
    assert_ne!(
        ha.finish(),
        hb.finish(),
        "distinct CIDs must fold differently into the cache-key hash"
    );
}
