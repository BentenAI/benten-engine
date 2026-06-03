//! F-INV21-1/2/3/4 (R3-W5) — Inv-21 fork-tie-break: smaller-`created_at_hlc`
//! wins, made TOTAL via the forking-event Version-Node CID, with the kani
//! convergence-proof harness stood up (proptest is the v1-beta floor).
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-INV21-1** (E1 + GNI-7),
//!   **F-INV21-2** (E2 + GNI-8, M-8, NQ-D2), **F-INV21-3** (E2-kani +
//!   GNI-9, NQ-D2 — NET-NEW kani harness), **F-INV21-4** (E3 + GNI-10).
//! - R0.3 plan §3.8.Inv-21 (M-7 asymmetry + M-8 totality, load-bearing):
//!   * property rule = LARGER-HLC-wins (LWW); Inv-21 fork = **SMALLER
//!     `created_at_hlc` wins** = oldest-anchor-wins (DELIBERATE opposite).
//!   * `MembershipSetId` can NEVER disambiguate concurrent same-anchor
//!     forks (they share id); terminal discriminator = the **forking
//!     event's Version-Node CID**. **Total order = (`created_at_hlc` ASC,
//!     then `fork_event_version_node_cid` ASC)**.
//!   * ALL event-authors fork (not just Admin); the losing fork's
//!     CRDT-vector MUST NOT merge into the winner (archived-not-discarded).
//! - Inv-19 (K(V) encrypts to the immutable Version-Node-CID) + Inv-20
//!   clause-f + §2.5 Path-A.5.
//!
//! ## Why FREEZE-GATING (R2 §6 / §"single highest untested byte risk")
//!
//! A non-TOTAL tie-break is a convergence-divergence bug that only
//! manifests under adversarial concurrent same-anchor forks: two engines
//! pick different winners ⇒ permanent divergence. F-INV21-3 stands up the
//! NET-NEW kani harness; **proptest is the v1-beta floor, the kani arm is
//! v1-GM strengthening — do NOT block the wave on kani standup** (R2 §"kani
//! harness is NOT in-tree").
//!
//! ## pim-2 §3.6b + §3.6f-ext end-to-end discipline
//!
//! Drives the PRODUCTION tie-break (`fork_winner` / `total_order_key`)
//! stand-ins; asserts OBSERVABLE winner + ordering-axiom holds (total /
//! antisymmetric / transitive) + losing-vector-not-merged; would-FAIL-if-no-op'd
//! (a naive LWW tie-break fails arm 1; a `MembershipSetId`-keyed tie-break
//! fails arm 2; a winner that absorbs the loser fails arm 4).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`; SELF-CONTAINED stub-shim for
//! parallel-safe R3. R5 swaps in `benten_membership_set` + (when kani
//! lands) the `#[kani::proof]` arm, and un-ignores.

#![allow(clippy::unwrap_used)]
// `cfg(kani)` is the conventional cargo-kani proof-harness gate. It is NOT a
// declared workspace check-cfg (kani is a v1-GM strengthening, not a v1-beta
// dependency — do NOT block the wave on kani standup, R2 §"kani harness is
// NOT in-tree"), so rustc would emit `unexpected_cfgs`. Allow it locally; the
// real kani integration at v1-GM registers the cfg in Cargo.toml/build.rs.
#![allow(unexpected_cfgs)]

// ── SELF-CONTAINED stub-shim ──

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hlc {
    physical_ms: u64,
    logical: u32,
    node_id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ForkId(u32);

/// A concurrent fork off the SAME set anchor. The two forks SHARE
/// `membership_set_id` (that is the whole point of M-8: id can't
/// disambiguate). The terminal discriminator is the content-addressed
/// `fork_event_version_node_cid`.
#[derive(Clone, Debug)]
struct ForkCandidate {
    id: ForkId,
    membership_set_id: Vec<u8>, // SHARED across concurrent same-anchor forks
    created_at_hlc: Hlc,
    fork_event_version_node_cid: Vec<u8>,
    /// CRDT-vector entries this fork carries. After resolution the WINNER
    /// must NOT contain loser-only entries (archived-not-discarded).
    crdt_vector: Vec<String>,
    /// True if this candidate's author is a non-Admin (ALL event-authors
    /// fork — F-INV21-4).
    author_is_admin: bool,
}

/// The TOTAL ordering key (M-8): (`created_at_hlc` ASC, then
/// `fork_event_version_node_cid` ASC). NOT `MembershipSetId`.
fn total_order_key(f: &ForkCandidate) -> (Hlc, Vec<u8>) {
    (f.created_at_hlc, f.fork_event_version_node_cid.clone())
}

/// PRODUCTION-stand-in: Inv-21 fork-tie-break. SMALLER key wins
/// (oldest-anchor; CID-ASC tiebreak). R5 routes through the real
/// `benten_membership_set` tie-break.
fn fork_winner<'a>(a: &'a ForkCandidate, b: &'a ForkCandidate) -> &'a ForkCandidate {
    if total_order_key(a) <= total_order_key(b) {
        a
    } else {
        b
    }
}

/// PRODUCTION-stand-in: merge resolution. The winner is chosen by the
/// tie-break; the loser is ARCHIVED (returned separately) and NOT absorbed
/// into the winner's CRDT-vector. R5 routes through the real merge.
fn resolve_fork<'a>(
    a: &'a ForkCandidate,
    b: &'a ForkCandidate,
) -> (&'a ForkCandidate, &'a ForkCandidate) {
    let winner = fork_winner(a, b);
    let loser = if std::ptr::eq(winner, a) { b } else { a };
    (winner, loser)
}

fn cid(payload: &[u8]) -> Vec<u8> {
    let d = blake3::hash(payload);
    let mut c = vec![0x01u8, 0x71, 0x1e, 0x20];
    c.extend_from_slice(d.as_bytes());
    c
}

fn fork(id: u32, created_ms: u64, cid_seed: &[u8], admin: bool, vector: &[&str]) -> ForkCandidate {
    ForkCandidate {
        id: ForkId(id),
        membership_set_id: cid(b"shared-anchor"), // SHARED
        created_at_hlc: Hlc {
            physical_ms: created_ms,
            logical: 0,
            node_id: id as u64,
        },
        fork_event_version_node_cid: cid(cid_seed),
        crdt_vector: vector.iter().map(|s| s.to_string()).collect(),
        author_is_admin: admin,
    }
}

// ── F-INV21-1 ───────────────────────────────────────────────────────────

/// F-INV21-1 — smaller-`created_at_hlc`-wins asymmetry (oldest-anchor).
///
/// Two forks at t1<t2: the OLDER (t1) wins — deliberately opposite to
/// property LWW. A later adversarial re-fork stamped with `u64::MAX` NEVER
/// displaces the original. (A test that passed under naive LWW would FAIL
/// here.)
#[test]
#[ignore = "RED-PHASE: F-INV21-1 — smaller-created_at_hlc-wins (oldest-anchor; opposite of LWW); un-ignore at R5"]
fn f_inv21_1_oldest_anchor_wins_adversary_cannot_displace() {
    let original = fork(1, 100, b"original-fork", true, &["e1"]);
    let later = fork(2, 200, b"later-fork", true, &["e2"]);
    assert_eq!(
        fork_winner(&original, &later).id.0,
        1,
        "oldest anchor (t1=100) wins"
    );

    // Adversary re-forks with the largest possible HLC — still loses.
    let adversary = fork(3, u64::MAX, b"adversary-fork", true, &["evil"]);
    assert_eq!(
        fork_winner(&original, &adversary).id.0,
        1,
        "an adversarial larger-HLC re-fork can NEVER displace the original (oldest-anchor-wins)"
    );
    // A naive LWW (larger-HLC-wins) would have picked the adversary — this
    // arm fails for any implementation that uses the property rule here.
}

// ── F-INV21-2 ───────────────────────────────────────────────────────────

/// F-INV21-2 — tie-break TOTALITY via Version-Node-CID (M-8 / NQ-D2).
///
/// When `created_at_hlc` ties, `MembershipSetId` CANNOT disambiguate
/// (concurrent forks SHARE it). The Version-Node CID decides. Also pins
/// the ordering axioms: totality + antisymmetry + transitivity.
#[test]
#[ignore = "RED-PHASE: F-INV21-2 — tie-break totality via Version-Node-CID, NOT MembershipSetId (M-8); un-ignore at R5"]
fn f_inv21_2_totality_via_version_node_cid() {
    // Two concurrent forks with TRULY IDENTICAL created_at_hlc (same
    // physical_ms AND node_id — a genuine tie) and SHARED
    // membership_set_id — only the Version-Node CID differs. (The `fork()`
    // helper varies node_id by id, which would break the tie via the HLC
    // itself; here we construct the tied HLC explicitly so the CID is the
    // SOLE discriminator — the M-8 totality contract.)
    let tied_hlc = Hlc {
        physical_ms: 100,
        logical: 0,
        node_id: 0xFEED,
    };
    let shared_anchor = cid(b"shared-anchor");
    let f_a = ForkCandidate {
        id: ForkId(1),
        membership_set_id: shared_anchor.clone(),
        created_at_hlc: tied_hlc,
        fork_event_version_node_cid: cid(b"fork-event-AAA"),
        crdt_vector: vec!["a".to_string()],
        author_is_admin: true,
    };
    let f_b = ForkCandidate {
        id: ForkId(2),
        membership_set_id: shared_anchor,
        created_at_hlc: tied_hlc,
        fork_event_version_node_cid: cid(b"fork-event-BBB"),
        crdt_vector: vec!["b".to_string()],
        author_is_admin: true,
    };
    assert_eq!(
        f_a.created_at_hlc, f_b.created_at_hlc,
        "the two forks GENUINELY tie on created_at_hlc — the CID must be the SOLE discriminator (M-8)"
    );
    assert_eq!(
        f_a.membership_set_id, f_b.membership_set_id,
        "concurrent same-anchor forks SHARE membership_set_id (cannot disambiguate)"
    );
    assert_ne!(
        f_a.fork_event_version_node_cid, f_b.fork_event_version_node_cid,
        "distinct fork events ⇒ distinct content-addressed Version-Node CIDs"
    );

    // The winner is decided by CID-ASC (the terminal discriminator).
    let winner = fork_winner(&f_a, &f_b);
    let expected = if f_a.fork_event_version_node_cid <= f_b.fork_event_version_node_cid {
        1
    } else {
        2
    };
    assert_eq!(
        winner.id.0, expected,
        "tie broken by Version-Node-CID ASC, not MembershipSetId"
    );

    // Antisymmetry: swapping arguments yields the SAME winner.
    assert_eq!(
        fork_winner(&f_b, &f_a).id.0,
        winner.id.0,
        "tie-break is antisymmetric (order-independent winner)"
    );

    // Totality: distinct keys are always comparable (never equal-and-unordered).
    assert_ne!(
        total_order_key(&f_a),
        total_order_key(&f_b),
        "the total order key is distinct for distinct fork events (totality)"
    );
}

// ── F-INV21-3 ───────────────────────────────────────────────────────────

/// F-INV21-3 — Inv-21 convergence proof (proptest surrogate; v1-beta floor).
///
/// The tie-break is total + deterministic + commutative over a bounded
/// fork-set: independent of presentation order, every engine picks the
/// SAME winner. The `#[kani::proof]` arm below stands up the NET-NEW kani
/// harness (v1-GM strengthening); it is NOT a wave blocker (R2 §"kani
/// harness is NOT in-tree" — do NOT block the wave on kani standup).
#[test]
#[ignore = "RED-PHASE: F-INV21-3 — convergence proof (proptest surrogate; v1-beta floor); un-ignore at R5"]
fn f_inv21_3_convergence_proptest_surrogate() {
    use proptest::prelude::*;
    proptest!(|(hlcs in proptest::collection::vec(0u64..1000, 2..6), seeds in 0u64..256)| {
        // Build a bounded fork-set with distinct Version-Node CIDs.
        let forks: Vec<ForkCandidate> = hlcs
            .iter()
            .enumerate()
            .map(|(i, &ms)| {
                fork(i as u32, ms, format!("fork-{i}-{seeds}").as_bytes(), true, &["x"])
            })
            .collect();

        // The min by the total order key — the deterministic winner.
        let canonical_winner = forks
            .iter()
            .min_by(|a, b| total_order_key(a).cmp(&total_order_key(b)))
            .unwrap()
            .id
            .0;

        // Reduce pairwise in forward and reverse order — convergence ⇒
        // both reductions pick the SAME winner (commutative + associative).
        let fwd = forks.iter().reduce(|a, b| fork_winner(a, b)).unwrap().id.0;
        let mut rev: Vec<&ForkCandidate> = forks.iter().collect();
        rev.reverse();
        let bwd = rev.into_iter().reduce(|a, b| fork_winner(a, b)).unwrap().id.0;

        prop_assert_eq!(fwd, canonical_winner, "forward reduction = canonical winner");
        prop_assert_eq!(bwd, canonical_winner, "reverse reduction = canonical winner (order-independent)");
    });
}

/// F-INV21-3 kani arm — NET-NEW harness standup (v1-GM strengthening).
///
/// Bounded-model-checks totality + determinism of the tie-break over a
/// 2-fork set. Compiled ONLY under `--cfg kani` (cargo-kani), so the
/// normal `cargo test` build never sees it — the wave is NOT blocked on
/// kani standup (R2 §"Do NOT block the wave on kani standup"). R5 wires
/// the real `benten_membership_set` tie-break + flips this on under v1-GM.
#[cfg(kani)]
#[kani::proof]
fn f_inv21_3_kani_tie_break_total() {
    let hlc_a: u64 = kani::any();
    let hlc_b: u64 = kani::any();
    let cid_a: u8 = kani::any();
    let cid_b: u8 = kani::any();
    kani::assume(!(hlc_a == hlc_b && cid_a == cid_b)); // distinct fork events

    let f_a = ForkCandidate {
        id: ForkId(1),
        membership_set_id: vec![0xAB],
        created_at_hlc: Hlc {
            physical_ms: hlc_a,
            logical: 0,
            node_id: 1,
        },
        fork_event_version_node_cid: vec![cid_a],
        crdt_vector: vec![],
        author_is_admin: true,
    };
    let f_b = ForkCandidate {
        id: ForkId(2),
        membership_set_id: vec![0xAB],
        created_at_hlc: Hlc {
            physical_ms: hlc_b,
            logical: 0,
            node_id: 2,
        },
        fork_event_version_node_cid: vec![cid_b],
        crdt_vector: vec![],
        author_is_admin: true,
    };
    // Totality + antisymmetry: a deterministic winner exists and is
    // order-independent.
    let w1 = fork_winner(&f_a, &f_b).id.0;
    let w2 = fork_winner(&f_b, &f_a).id.0;
    assert!(w1 == w2);
}

// ── F-INV21-4 ───────────────────────────────────────────────────────────

/// F-INV21-4 — losing-fork MUST-NOT-merge + archived-not-discarded +
/// fork-as-DAG-branch + non-Admin can fork + K(V)→immutable Version-Node-CID.
#[test]
#[ignore = "RED-PHASE: F-INV21-4 — losing fork not absorbed + archived + any author forks; un-ignore at R5"]
fn f_inv21_4_losing_fork_not_merged_archived_not_discarded() {
    // Winner carries ["w-only"]; loser carries ["l-only"]. A non-Admin
    // authored the loser (ALL event-authors fork).
    let winner_fork = fork(1, 100, b"winner", true, &["w-only"]);
    let loser_fork = fork(2, 200, b"loser", /* admin = */ false, &["l-only"]);

    let (winner, loser) = resolve_fork(&winner_fork, &loser_fork);
    assert_eq!(winner.id.0, 1, "oldest anchor wins");
    assert_eq!(loser.id.0, 2, "the later fork loses");

    // Losing fork's CRDT-vector is NOT merged into the winner (no silent
    // absorption).
    assert!(
        !winner.crdt_vector.contains(&"l-only".to_string()),
        "winner MUST NOT absorb the loser's CRDT-vector entries (no silent merge)"
    );

    // Archived-not-discarded: the loser anchor is retained (its vector is
    // still readable post-resolution) — just not CURRENT.
    assert!(
        loser.crdt_vector.contains(&"l-only".to_string()),
        "the losing fork is ARCHIVED (retained), not discarded"
    );

    // ALL event-authors fork: a non-Admin authored a legitimate fork.
    assert!(
        !loser.author_is_admin,
        "a non-Admin event-author can fork (F-INV21-4)"
    );

    // Inv-19: K(V) encrypts to the IMMUTABLE Version-Node CID. The winner's
    // key target is the content-addressed Version-Node CID (stable; the
    // fork-event CID never mutates).
    let k_v_target = &winner.fork_event_version_node_cid;
    assert_eq!(
        k_v_target,
        &cid(b"winner"),
        "K(V) targets the immutable Version-Node CID (Inv-19 / Inv-20 clause-f)"
    );
}
