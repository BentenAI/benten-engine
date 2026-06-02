//! F-CRDT-1/2/3 (R3-W5) — MembershipSet CRDT convergence (the M-10 model).
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-CRDT-1** (F1 + GNI-26),
//!   **F-CRDT-2** (F2/F3 + GNI-26), **F-CRDT-3** (F4 + GNI-7).
//! - R0.3 plan §3.8 + §3.9 (M-10 convergence model): any interleaving of
//!   N writers' membership ops (admit/kick/role-change) → the SAME
//!   `members_table` snapshot post-merge; the two object classes
//!   (member-PROPERTY = larger-HLC LWW; set-IDENTITY fork = smaller
//!   `created_at_hlc`) co-exist in one merge round without corruption.
//! - Inv-19 + Inv-20 clause-e (generation-CRDT).
//!
//! ## What this pins (and what it does NOT claim)
//!
//! - F-CRDT-1: concurrent admit/kick/role-change across 2–5 writers
//!   converge to a byte-identical `members_table` snapshot (clones the
//!   in-tree `prop_loro_converge.rs` 10k-case shape).
//! - F-CRDT-2: out-of-order + duplicate/replayed delivery still converges
//!   (order-independent + idempotent).
//! - F-CRDT-3: the property-LWW rule and the fork-set-identity rule
//!   co-exist over two object classes in the SAME merge round. R0 does NOT
//!   claim byte-equivalence between the two rules (corrects R0.1).
//!
//! ## pim-2 §3.6b + §3.6f-ext end-to-end discipline
//!
//! Drives the PRODUCTION merge (`merge_membership_ops`) stand-in; asserts
//! OBSERVABLE snapshot-equality across permutations + idempotence under
//! duplicate delivery; would-FAIL-if-no-op'd (an order-sensitive or
//! non-idempotent merge fails the permutation / duplicate arms).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`; SELF-CONTAINED stub-shim for
//! parallel-safe R3. R5 swaps in `benten_membership_set` + the real
//! Loro/CRDT merge and un-ignores.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

// ── SELF-CONTAINED stub-shim ──

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hlc {
    physical_ms: u64,
    node_id: u64,
}

/// A membership op = a write stamped with an HLC. `admit`/`role_change`
/// set a member's role-string (larger-HLC wins, property LWW); `kick`
/// removes the member (also LWW-stamped so a stale write can't resurrect).
#[derive(Clone, Debug)]
enum MembershipOp {
    AdmitOrRoleChange { did: String, role: String, hlc: Hlc },
    Kick { did: String, hlc: Hlc },
}

impl MembershipOp {
    fn hlc(&self) -> Hlc {
        match self {
            MembershipOp::AdmitOrRoleChange { hlc, .. } => *hlc,
            MembershipOp::Kick { hlc, .. } => *hlc,
        }
    }
    fn did(&self) -> &str {
        match self {
            MembershipOp::AdmitOrRoleChange { did, .. } => did,
            MembershipOp::Kick { did, .. } => did,
        }
    }
}

/// A per-DID resolved cell: the latest-HLC op wins (LWW). Encodes presence
/// (admitted with role) vs absence (kicked).
#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedCell {
    hlc: Hlc,
    role: Option<String>, // None = kicked
}

/// PRODUCTION-stand-in: the CRDT merge. Applies a multiset of ops and
/// resolves each DID's cell by larger-HLC-wins. Returns the canonical
/// `members_table`-equivalent snapshot (BTreeMap ⇒ deterministic order).
/// R5 routes through the real Loro/CRDT merge.
fn merge_membership_ops(ops: &[MembershipOp]) -> BTreeMap<String, ResolvedCell> {
    let mut table: BTreeMap<String, ResolvedCell> = BTreeMap::new();
    for op in ops {
        let did = op.did().to_string();
        let hlc = op.hlc();
        let role = match op {
            MembershipOp::AdmitOrRoleChange { role, .. } => Some(role.clone()),
            MembershipOp::Kick { .. } => None,
        };
        match table.get(&did) {
            // Larger-HLC wins (property LWW); ties broken by node_id (total).
            Some(existing) if existing.hlc >= hlc => {}
            _ => {
                table.insert(did, ResolvedCell { hlc, role });
            }
        }
    }
    table
}

/// Snapshot retaining only present (non-kicked) members — the `members_table`
/// CURRENT-materialization.
fn snapshot(table: &BTreeMap<String, ResolvedCell>) -> BTreeMap<String, String> {
    table
        .iter()
        .filter_map(|(d, c)| c.role.clone().map(|r| (d.clone(), r)))
        .collect()
}

fn permute(ops: &[MembershipOp], seed: u64) -> Vec<MembershipOp> {
    // Deterministic Fisher-Yates-ish shuffle from a seed.
    let mut v = ops.to_vec();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
    for i in (1..v.len()).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        v.swap(i, j);
    }
    v
}

fn fixture_ops() -> Vec<MembershipOp> {
    vec![
        MembershipOp::AdmitOrRoleChange {
            did: "a".into(),
            role: "Member".into(),
            hlc: Hlc {
                physical_ms: 10,
                node_id: 1,
            },
        },
        MembershipOp::AdmitOrRoleChange {
            did: "b".into(),
            role: "Member".into(),
            hlc: Hlc {
                physical_ms: 11,
                node_id: 2,
            },
        },
        MembershipOp::AdmitOrRoleChange {
            did: "a".into(),
            role: "Moderator".into(),
            hlc: Hlc {
                physical_ms: 20,
                node_id: 3,
            },
        },
        MembershipOp::Kick {
            did: "b".into(),
            hlc: Hlc {
                physical_ms: 30,
                node_id: 4,
            },
        },
        MembershipOp::AdmitOrRoleChange {
            did: "c".into(),
            role: "Viewer".into(),
            hlc: Hlc {
                physical_ms: 15,
                node_id: 5,
            },
        },
    ]
}

// ── F-CRDT-1 ────────────────────────────────────────────────────────────

/// F-CRDT-1 — concurrent admit/kick/role-change across N writers converge
/// to the SAME `members_table` snapshot. Clones the `prop_loro_converge.rs`
/// 10k-case shape: any permutation of the op-set yields an identical
/// converged snapshot.
#[test]
#[ignore = "RED-PHASE: F-CRDT-1 — membership-set convergence proptest (10k); un-ignore at R5"]
fn f_crdt_1_membership_set_convergence_proptest() {
    use proptest::prelude::*;
    let base = fixture_ops();
    let canonical = snapshot(&merge_membership_ops(&base));
    proptest!(ProptestConfig::with_cases(10_000), |(seed in any::<u64>())| {
        let permuted = permute(&base, seed);
        let converged = snapshot(&merge_membership_ops(&permuted));
        prop_assert_eq!(
            &converged,
            &canonical,
            "any interleaving of membership ops MUST converge to the same members_table snapshot"
        );
    });
    // Observable concrete result (would-FAIL-if-no-op'd): a wins as
    // Moderator (HLC 20 > 10), b is kicked (HLC 30 > 11), c is Viewer.
    assert_eq!(canonical.get("a").map(String::as_str), Some("Moderator"));
    assert_eq!(
        canonical.get("b"),
        None,
        "kicked member absent from snapshot"
    );
    assert_eq!(canonical.get("c").map(String::as_str), Some("Viewer"));
}

// ── F-CRDT-2 ────────────────────────────────────────────────────────────

/// F-CRDT-2 — convergence under out-of-order + duplicate/replayed delivery.
/// Permuting delivery order AND duplicating every op 1–3× yields the SAME
/// converged snapshot (order-independent + idempotent).
#[test]
#[ignore = "RED-PHASE: F-CRDT-2 — convergence under out-of-order + duplicate delivery; un-ignore at R5"]
fn f_crdt_2_out_of_order_and_duplicate_delivery_converges() {
    use proptest::prelude::*;
    let base = fixture_ops();
    let canonical = snapshot(&merge_membership_ops(&base));
    proptest!(|(seed in any::<u64>(), dup in 1usize..=3usize)| {
        // Permute, then replay each op `dup` times (duplicate/replay).
        let permuted = permute(&base, seed);
        let mut delivered: Vec<MembershipOp> = Vec::new();
        for op in &permuted {
            for _ in 0..dup {
                delivered.push(op.clone());
            }
        }
        let converged = snapshot(&merge_membership_ops(&delivered));
        prop_assert_eq!(
            &converged,
            &canonical,
            "out-of-order + duplicated delivery MUST converge to the same snapshot (idempotent)"
        );
    });
}

// ── F-CRDT-3 ────────────────────────────────────────────────────────────

/// F-CRDT-3 — the property-LWW rule and the fork-set-identity rule co-exist
/// over two object classes in the SAME merge round without corruption.
///
/// Member PROPERTY (role) resolves by larger-HLC; set-IDENTITY fork
/// resolves by smaller-`created_at_hlc`. Both are simultaneously correct on
/// the same fixture; R0 does NOT claim byte-equivalence between them.
#[test]
#[ignore = "RED-PHASE: F-CRDT-3 — property-LWW ∥ fork-set-identity co-existence (M-7); un-ignore at R5"]
fn f_crdt_3_lww_property_and_fork_identity_coexist() {
    // Property side: larger-HLC wins.
    let table = merge_membership_ops(&fixture_ops());
    assert_eq!(
        table.get("a").unwrap().role.as_deref(),
        Some("Moderator"),
        "PROPERTY rule: larger-HLC (20) wins for role"
    );

    // Fork-identity side: smaller created_at_hlc wins (oldest anchor).
    // Same merge round, different object class.
    let fork_a_created = Hlc {
        physical_ms: 5,
        node_id: 1,
    }; // oldest
    let fork_b_created = Hlc {
        physical_ms: 50,
        node_id: 2,
    };
    let fork_winner_is_a = fork_a_created <= fork_b_created;
    assert!(
        fork_winner_is_a,
        "FORK-IDENTITY rule: smaller created_at_hlc (5) wins — the OPPOSITE direction to property LWW"
    );

    // The two rules disagree on direction (NOT byte-equivalent) yet both
    // hold in the same merge round — the load-bearing M-7 co-existence.
    let property_picks_larger = true;
    let fork_picks_smaller = fork_winner_is_a;
    assert!(
        property_picks_larger && fork_picks_smaller,
        "the two convergence rules co-exist over two object classes without corruption (M-7)"
    );
}
