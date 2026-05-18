//! #1003 closure-pin (Fwd-2 / RATIFIED 2026-05-17): the legacy crate-root
//! `u64`-id `Anchor` compatibility surface (`benten_core::Anchor` +
//! `append_version` / `current_version` / `walk_versions` +
//! `U64_CHAINS` + `ANCHOR_COUNTER`) was DELETED — it had zero non-test
//! callers and coexisted with two other version-chain shapes. Per
//! CLAUDE.md rule #5 the surface is gone entirely (no `#[deprecated]`
//! shim).
//!
//! The deletion is *primarily* enforced structurally — the workspace only
//! compiles because every former caller was migrated/removed in the same
//! change, and `benten_core::Anchor` / `benten_core::append_version` no
//! longer resolve at the crate root. This pin adds a positive assertion
//! that the **canonical replacement surface is intact and behaves as the
//! single linear-anchor contract**, so a future "restore the u64 surface"
//! regression (which would re-introduce the three-coexisting-shapes drift
//! #1003 closed) is caught by an explicit behavioural test rather than
//! only by a compile error somewhere unrelated.
//!
//! Would FAIL if `version::Anchor`'s prior-head-threaded append /
//! branch-detection contract regressed (the contract that made the u64
//! surface redundant + safe to delete).

#![allow(clippy::unwrap_used)]

use benten_core::Value;
use benten_core::version::{Anchor, VersionError, append_version, walk_versions};
use benten_core::{Node, version};
use std::collections::BTreeMap;

fn versioned_node(seq: i64) -> Node {
    let mut p = BTreeMap::new();
    p.insert("seq".to_string(), Value::Int(seq));
    Node::new(vec!["Post".to_string()], p)
}

/// The canonical linear surface (`version::Anchor`) is prior-head-threaded:
/// an append names the head the caller observed. This is the capability
/// that made the headless u64-id surface redundant — pin it.
#[test]
fn canonical_version_anchor_is_prior_head_threaded_and_linearizable() {
    let v0 = versioned_node(0).cid().unwrap();
    let anchor = Anchor::new(v0);

    let v1 = versioned_node(1).cid().unwrap();
    let v2 = versioned_node(2).cid().unwrap();

    append_version(&anchor, &v0, &v1).expect("append against observed head must succeed");
    append_version(&anchor, &v1, &v2).expect("append against advanced head must succeed");

    let chain: Vec<_> = walk_versions(&anchor).collect();
    assert_eq!(
        chain.last().copied(),
        Some(v2),
        "the canonical linear surface must track the latest head; this is \
         the contract that superseded the deleted u64-id Anchor (#1003)"
    );
}

/// The deleted u64-id surface could NOT detect concurrent forks (its
/// headless `append_version(&anchor, &node)` always succeeded). The
/// replacement's fork-rejection is exactly why deleting the weaker
/// surface is safe — pin that it still fires.
#[test]
fn canonical_version_anchor_rejects_concurrent_fork() {
    let v0 = versioned_node(0).cid().unwrap();
    let anchor = Anchor::new(v0);

    let v1_a = versioned_node(11).cid().unwrap();
    let v1_b = versioned_node(12).cid().unwrap();

    append_version(&anchor, &v0, &v1_a).expect("first append must succeed");
    // Second writer still thinks v0 is the head → a concurrent fork. The
    // deleted u64 surface would have silently accepted this; the canonical
    // surface MUST reject it.
    let err = append_version(&anchor, &v0, &v1_b)
        .expect_err("appending against a stale head MUST be rejected (the contract that made the headless u64-id surface unsafe-and-removable)");
    assert!(
        matches!(err, VersionError::Branched { .. }),
        "stale-head append must surface VersionError::Branched, got: {err:?}"
    );
}

/// Documents the post-#1003 version-surface inventory at the type level:
/// exactly TWO canonical surfaces remain — the linear `version::Anchor`
/// and the DAG-shape `version_chain::DagVersionChain` (#849 Phase-4-Meta
/// D3 unifies the two). The headless u64-id third shape is gone. The
/// type-annotated function pointers fail to compile if either canonical
/// constructor changes shape, so a "restore the deleted surface" or
/// "perturb a canonical constructor" regression is caught here.
#[test]
fn two_canonical_version_surfaces_remain() {
    // Type-annotated fn pointers: fail to compile if either canonical
    // constructor changes shape (or if a deleted-surface restore shadows
    // the crate root). Both are `Cid -> _` — the post-#1003 contract.
    let linear_ctor: fn(benten_core::Cid) -> version::Anchor = version::Anchor::new;
    let dag_ctor: fn(benten_core::Cid) -> benten_core::version_chain::DagVersionChain =
        benten_core::version_chain::DagVersionChain::new;

    let v0 = versioned_node(0).cid().unwrap();
    let linear = linear_ctor(v0);
    let dag = dag_ctor(v0);

    // Exercise both so the constructors are not dead: the linear surface
    // tracks its head; the DAG surface's CURRENT is its root.
    let linear_chain: Vec<_> = walk_versions(&linear).collect();
    assert_eq!(
        linear_chain,
        vec![v0],
        "a freshly-constructed linear Anchor's chain is exactly its head"
    );
    assert_eq!(
        dag.current(),
        Some(&v0),
        "a freshly-constructed DagVersionChain's CURRENT is its root cid"
    );
}
