//! G24-D row pin — DAG version chains.
//!
//! Per CLAUDE.md #18 + D-4F-14: linear version-chain extended to
//! support branches (forks). Anchor → v1 → {v2-mainline, v1.5-fork};
//! CURRENT can point at any branch tip. Per-user-local version history.
//!
//! Extends Phase-1 `Anchor + Version Node + CURRENT pointer` pattern
//! to DAG-shape. `crates/benten-core/src/version_chain.rs` (EXTEND
//! existing linear Version Node pattern).

use benten_core::Cid;
use benten_core::version_chain::DagVersionChain;

#[test]
fn version_chain_extended_to_dag_supports_fork_topology_walks_both_tips() {
    // SUBSTANTIVE per pim-2 §3.6b: exercise DagVersionChain at HEAD
    // building a fork topology (v1 -> {v2-mainline, v1.5-fork}); assert
    // walking the DAG via tips() recovers BOTH branch tips. Would-FAIL
    // if the impl used a linear linked-list shape (would only return
    // one tip).
    let anchor = Cid::from_blake3_digest([0u8; 32]);
    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let v2_mainline = Cid::from_blake3_digest([2u8; 32]);
    let v1_5_fork = Cid::from_blake3_digest([3u8; 32]);

    let mut dag = DagVersionChain::new(anchor);
    dag.add_version(anchor, v1).expect("anchor->v1");
    dag.add_version(v1, v2_mainline).expect("v1->v2_mainline");
    dag.add_version(v1, v1_5_fork).expect("v1->v1_5_fork");

    // Ancestor relations exercised.
    assert!(dag.is_ancestor_of(&anchor, &v2_mainline));
    assert!(dag.is_ancestor_of(&anchor, &v1_5_fork));
    assert!(dag.is_ancestor_of(&v1, &v2_mainline));
    assert!(dag.is_ancestor_of(&v1, &v1_5_fork));
    // No cross-branch ancestry.
    assert!(!dag.is_ancestor_of(&v2_mainline, &v1_5_fork));
    assert!(!dag.is_ancestor_of(&v1_5_fork, &v2_mainline));

    // SUBSTANTIVE tip-walk: both branch tips present (DAG semantics,
    // not linear). Would-FAIL if the impl is a linked-list (only one
    // tip).
    let tips = dag.tips();
    assert_eq!(
        tips.len(),
        2,
        "fork topology MUST surface both tips; would-FAIL if linear walk"
    );
    assert!(tips.contains(&v2_mainline));
    assert!(tips.contains(&v1_5_fork));
}

#[test]
fn current_pointer_can_point_at_any_branch_tip_set_current_round_trip() {
    // SUBSTANTIVE per pim-2 §3.6b: CURRENT pointer set_current(tip)
    // round-trips on EITHER branch (per CLAUDE.md #18 D-4F-14:
    // "CURRENT can point at any branch tip"). Would-FAIL if the impl
    // forced CURRENT to a single canonical tip.
    let anchor = Cid::from_blake3_digest([0u8; 32]);
    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let v2_mainline = Cid::from_blake3_digest([2u8; 32]);
    let v1_5_fork = Cid::from_blake3_digest([3u8; 32]);

    let mut dag = DagVersionChain::new(anchor);
    dag.add_version(anchor, v1).unwrap();
    dag.add_version(v1, v2_mainline).unwrap();
    dag.add_version(v1, v1_5_fork).unwrap();

    // Switch CURRENT to the mainline tip; round-trip the read.
    dag.set_current(v2_mainline).expect("can set mainline");
    assert_eq!(dag.current(), Some(&v2_mainline));

    // Switch CURRENT to the fork tip; round-trip the read.
    dag.set_current(v1_5_fork).expect("can set fork");
    assert_eq!(dag.current(), Some(&v1_5_fork));

    // Switch back to mainline; verify CURRENT updates (not append-
    // only).
    dag.set_current(v2_mainline).expect("can re-set mainline");
    assert_eq!(dag.current(), Some(&v2_mainline));
}

#[ignore = "RED-PHASE (Phase 4-Meta — per-device-keyed CURRENT lands when admin UI \
    multi-device sync arrives). Per-device-keyed CURRENT (LoroMap keyed by device-DID) \
    is NOT in the Phase 4-Foundation G24-D scope; the current DagVersionChain ships \
    process-local CURRENT only. Phase 4-Meta wires LoroMap-keyed CURRENT for multi- \
    device-divergent active-version state. Named destination: Phase 4-Meta (admin UI \
    self-composing wave). HARD RULE 12 clause-(b) BELONGS-NAMED-NOW: this entry IS \
    the named destination."]
#[test]
fn current_pointer_is_per_device_keyed_loro_map() {
    let _v1 = Cid::from_blake3_digest([1u8; 32]);
    let _v2_fork = Cid::from_blake3_digest([3u8; 32]);

    // Phase 4-Meta surface (NOT shipped at G24-D): CURRENT is a LoroMap
    // keyed by device-DID. Different devices can have different
    // active versions. Sync surface presents per-device-keyed map.
    //
    // FAILS-IF-NO-OP because a single-CURRENT impl would force all
    // devices to the same version. Phase-4-Foundation G24-D ships
    // single-CURRENT only; per-device CURRENT lands at Phase 4-Meta
    // multi-device-sync wave.
    panic!("Phase-4-Meta wires per-device-keyed CURRENT as LoroMap");
}
