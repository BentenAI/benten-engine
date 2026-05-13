//! G24-D row pin — DAG version chains.
//!
//! Per CLAUDE.md #18 + D-4F-14: linear version-chain extended to
//! support branches (forks). Anchor → v1 → {v2-mainline, v1.5-fork};
//! CURRENT can point at any branch tip. Per-user-local version history.
//!
//! Extends Phase-1 `Anchor + Version Node + CURRENT pointer` pattern
//! to DAG-shape. `crates/benten-core/src/version_chain.rs` (EXTEND
//! existing linear Version Node pattern).

mod common;

use common::manifest_fixtures::{stub_cid_one, stub_cid_two};

#[test]
#[ignore = "RED-PHASE: G24-D wave extends version_chain to DAG shape; un-ignore at G24-D landing"]
fn version_chain_extended_to_dag_supports_fork_then_merge_topology() {
    let v1 = stub_cid_one();
    let v2_fork = stub_cid_two();

    // Future surface in benten-core::version_chain extended to DAG:
    //   append_version(anchor, v1)
    //   append_version_with_parent(anchor, v2_fork, parent=v1)
    //   walk_versions_dag(anchor) -> Vec<(Cid, Vec<Cid>)>  // node + parents
    //
    // SUBSTANTIVE assertion: walking the DAG from anchor recovers both
    // tips (v2-mainline and v1.5-fork) when both descend from v1.
    // FAILS-IF-NO-OP because linear walk would miss the fork tip.
    panic!("RED-PHASE: G24-D wave must extend version_chain to DAG with fork-and-merge walk");
}

#[test]
#[ignore = "RED-PHASE: G24-D wave wires per-device-local CURRENT pointer (Loro Map per ratification #2); un-ignore at G24-D landing"]
fn current_pointer_is_per_device_keyed_loro_map() {
    let v1 = stub_cid_one();
    let v2_fork = stub_cid_two();

    // Future surface: CURRENT is a LoroMap keyed by device-DID.
    // Different devices can have different active versions. Sync
    // surface presents per-device-keyed map.
    //
    // FAILS-IF-NO-OP because a single-CURRENT impl would force all
    // devices to the same version.
    panic!("RED-PHASE: G24-D wave must wire per-device-keyed CURRENT as LoroMap");
}
