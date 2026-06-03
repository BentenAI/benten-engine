//! F-CRDT-1/2/3 (R3-W5) — MembershipSet CRDT convergence (the M-10 model).
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-CRDT-1** (F1 + GNI-26),
//!   **F-CRDT-2** (F2/F3 + GNI-26), **F-CRDT-3** (F4 + GNI-7).
//! - R0.5 plan §3.8 + §3.9 (M-10 convergence model): any interleaving of
//!   N writers' membership ops (admit/kick/role-change) → the SAME
//!   `members_table` snapshot post-merge; the two object classes
//!   (member-PROPERTY = larger-HLC LWW; set-IDENTITY fork = smaller
//!   `created_at_hlc`) co-exist in one merge round without corruption.
//! - Inv-19 + Inv-20 clause-e (generation-CRDT).
//! - Associativity (F4-023 cross-ref): the tie-break associativity property
//!   is owned by **F-INV21-3** (the convergence proptest surrogate in
//!   `f_inv21_fork_tie_break_totality_version_node_cid.rs`) and the same-HLC
//!   tie by **F-INV21-2**; this file does NOT duplicate them.
//!
//! ## What this pins (and what it does NOT claim)
//!
//! - F-CRDT-1: concurrent admit/kick/role-change across 2–5 writers
//!   converge to a byte-identical `members_table` snapshot (clones the
//!   in-tree `prop_loro_converge.rs` shape).
//! - F-CRDT-2: out-of-order + duplicate/replayed delivery still converges
//!   (order-independent + idempotent).
//! - F-CRDT-3: the property-LWW rule and the fork-set-identity rule
//!   co-exist over two object classes in the SAME merge round. The
//!   fork-identity side routes through the SAME `fork_winner`/`total_order_key`
//!   tie-break that F-INV21-* pins (NOT an inline literal comparison), so the
//!   arm asserts the actual tie-break OUTPUT. R0 does NOT claim
//!   byte-equivalence between the two rules (corrects R0.1).
//!
//! ## pim-2 §3.6b + §3.6f-ext end-to-end discipline
//!
//! Drives the PRODUCTION merge (`merge_membership_ops`) + tie-break
//! (`fork_winner`) stand-ins; asserts OBSERVABLE snapshot-equality across
//! permutations + idempotence under duplicate delivery + the actual fork
//! tie-break winner; would-FAIL-if-no-op'd (an order-sensitive or
//! non-idempotent merge fails the permutation / duplicate arms; a larger-HLC
//! fork tie-break fails the F-CRDT-3 fork arm).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`; SELF-CONTAINED stub-shim for
//! parallel-safe R3. R5 swaps in `benten_membership_set` + the real
//! Loro/CRDT merge and un-ignores.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

// ── SELF-CONTAINED stub-shim ──

/// Stub HLC — the 3-field shape (parity with `benten_core::hlc` and the
/// sibling F-HLC / F-INV21 stubs). Lexicographic compare on
/// (physical_ms, logical, node_id).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hlc {
    physical_ms: u64,
    logical: u32,
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
            // Larger-HLC wins (property LWW); ties broken lexicographically
            // by (logical, node_id) via the derived Ord (total).
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

// ── fork-identity tie-break stand-in (the SAME rule F-INV21-* pins) ──
//
// F-CRDT-3 routes the fork-identity side through this real tie-break rather
// than an inline `<=` of two literals (F4-010). `total_order_key` is the
// (created_at_hlc ASC, fork_event_version_node_cid ASC) M-8 key; `fork_winner`
// returns the SMALLER-key fork (oldest-anchor-wins). This is the OPPOSITE
// direction to property LWW — the load-bearing M-7 co-existence — and the arm
// asserts the actual OUTPUT, so a larger-HLC fork rule would FAIL it.

#[derive(Clone, Debug)]
struct ForkCandidate {
    id: u32,
    created_at_hlc: Hlc,
    fork_event_version_node_cid: Vec<u8>,
}

fn total_order_key(f: &ForkCandidate) -> (Hlc, Vec<u8>) {
    (f.created_at_hlc, f.fork_event_version_node_cid.clone())
}

/// SMALLER-key fork wins (oldest-anchor; Inv-21). Same rule as
/// `f_inv21_*::fork_winner`.
fn fork_winner<'a>(a: &'a ForkCandidate, b: &'a ForkCandidate) -> &'a ForkCandidate {
    if total_order_key(a) <= total_order_key(b) {
        a
    } else {
        b
    }
}

fn cid(payload: &[u8]) -> Vec<u8> {
    let d = blake3::hash(payload);
    let mut c = vec![0x01u8, 0x71, 0x1e, 0x20];
    c.extend_from_slice(d.as_bytes());
    c
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
                logical: 0,
                node_id: 1,
            },
        },
        MembershipOp::AdmitOrRoleChange {
            did: "b".into(),
            role: "Member".into(),
            hlc: Hlc {
                physical_ms: 11,
                logical: 0,
                node_id: 2,
            },
        },
        MembershipOp::AdmitOrRoleChange {
            did: "a".into(),
            role: "Moderator".into(),
            hlc: Hlc {
                physical_ms: 20,
                logical: 0,
                node_id: 3,
            },
        },
        MembershipOp::Kick {
            did: "b".into(),
            hlc: Hlc {
                physical_ms: 30,
                logical: 0,
                node_id: 4,
            },
        },
        MembershipOp::AdmitOrRoleChange {
            did: "c".into(),
            role: "Viewer".into(),
            hlc: Hlc {
                physical_ms: 15,
                logical: 0,
                node_id: 5,
            },
        },
    ]
}

// ── F-CRDT-1 ────────────────────────────────────────────────────────────

/// F-CRDT-1 — concurrent admit/kick/role-change across N writers converge
/// to the SAME `members_table` snapshot. Clones the `prop_loro_converge.rs`
/// shape: any permutation of the op-set yields an identical converged
/// snapshot. (F4-021: uses the nextest/proptest default case-count, not a
/// hardcoded 10k literal.)
#[test]
#[ignore = "RED-PHASE: F-CRDT-1 — membership-set convergence proptest; un-ignore at R5"]
fn f_crdt_1_membership_set_convergence_proptest() {
    use proptest::prelude::*;
    let base = fixture_ops();
    let canonical = snapshot(&merge_membership_ops(&base));
    proptest!(|(seed in any::<u64>())| {
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
/// resolves by smaller-`created_at_hlc` (routed through the REAL
/// `fork_winner`/`total_order_key` tie-break — F4-010 — so the arm asserts
/// the actual tie-break OUTPUT, not an inline literal comparison). Both are
/// simultaneously correct on the same fixture; R0 does NOT claim
/// byte-equivalence between them.
#[test]
#[ignore = "RED-PHASE: F-CRDT-3 — property-LWW ∥ fork-set-identity co-existence (M-7); un-ignore at R5"]
fn f_crdt_3_lww_property_and_fork_identity_coexist() {
    // Property side: larger-HLC wins (observable role).
    let table = merge_membership_ops(&fixture_ops());
    let property_role = table.get("a").unwrap().role.as_deref();
    assert_eq!(
        property_role,
        Some("Moderator"),
        "PROPERTY rule: larger-HLC (20) wins for role"
    );

    // Fork-identity side: SMALLER created_at_hlc wins — routed through the
    // SAME tie-break F-INV21-* pins (NOT an inline `<=` of two literals).
    let fork_a = ForkCandidate {
        id: 1,
        created_at_hlc: Hlc {
            physical_ms: 5, // oldest anchor
            logical: 0,
            node_id: 1,
        },
        fork_event_version_node_cid: cid(b"fork-a"),
    };
    let fork_b = ForkCandidate {
        id: 2,
        created_at_hlc: Hlc {
            physical_ms: 50,
            logical: 0,
            node_id: 2,
        },
        fork_event_version_node_cid: cid(b"fork-b"),
    };
    // The actual tie-break OUTPUT: oldest-anchor (fork_a) wins. A larger-HLC
    // (property-LWW-direction) fork rule would pick fork_b and FAIL here.
    let winner = fork_winner(&fork_a, &fork_b);
    assert_eq!(
        winner.id, 1,
        "FORK-IDENTITY rule: smaller created_at_hlc (5) wins (oldest-anchor) — the OPPOSITE direction to property LWW"
    );
    // Antisymmetry: order-independent winner (still fork_a).
    assert_eq!(
        fork_winner(&fork_b, &fork_a).id,
        1,
        "the fork tie-break is order-independent (antisymmetric)"
    );

    // The two rules disagree on DIRECTION (NOT byte-equivalent) yet both hold
    // in the same merge round — the load-bearing M-7 co-existence. We derive
    // each direction from the actual stand-ins (no hardcoded `true`):
    //   * property: the LATER write (HLC 20 > 10) won → larger-HLC direction.
    //   * fork:     the EARLIER anchor (HLC 5 < 50) won → smaller-HLC direction.
    let property_picked_larger_hlc = property_role == Some("Moderator"); // the HLC-20 write
    let fork_picked_smaller_hlc = winner.created_at_hlc.physical_ms == 5; // the oldest anchor
    assert!(
        property_picked_larger_hlc && fork_picked_smaller_hlc,
        "the two convergence rules co-exist over two object classes without corruption, in OPPOSITE directions (M-7)"
    );
}
