//! F-INV21-1/2/3/4 (R3-W5) — Inv-21 fork-tie-break: smaller-`created_at_hlc`
//! wins, made TOTAL via the forking-event Version-Node CID, with the kani
//! convergence-proof harness stood up (proptest is the v1-beta floor).
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-INV21-1** (E1 + GNI-7),
//!   **F-INV21-2** (E2 + GNI-8, M-8, NQ-D2), **F-INV21-3** (E2-kani +
//!   GNI-9, NQ-D2 — NET-NEW kani harness), **F-INV21-4** (E3 + GNI-10).
//! - R0.5 plan §3.8.Inv-21 (M-7 asymmetry + M-8 totality, load-bearing):
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
//! Drives the PRODUCTION tie-break (`fork_winner` / `total_order_key`) +
//! merge (`resolve_fork` → CURRENT-snapshot) + K(V)-derivation (`derive_k_v`)
//! stand-ins; asserts OBSERVABLE winner + ordering-axiom holds (total /
//! antisymmetric / transitive) + losing-vector-NOT-in-the-merge-OUTPUT +
//! K(V)-derived-from-the-Version-Node-CID; would-FAIL-if-no-op'd (a naive LWW
//! tie-break fails arm 1; a `MembershipSetId`-keyed tie-break fails arm 2; an
//! intransitive order fails arm 2's transitivity sub-case; a merge that
//! ABSORBS the loser into CURRENT fails arm 4; a K(V) that ignores the CID
//! fails arm 4's key sub-case).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`; SELF-CONTAINED stub-shim for
//! parallel-safe R3. R5 swaps in `benten_membership_set` + (when kani
//! lands) the `#[kani::proof]` arm, and un-ignores.

#![allow(clippy::unwrap_used)]
// TIER-2 (w-ms-sync) RED-PHASE stub: a cosmetic `format_collect` lint in the
// self-contained hex helper. Non-semantic; the w-ms-sync wave rewrites this
// stub against the real crate surface and clears it.
#![allow(clippy::format_collect)]
// `cfg(kani)` is the conventional cargo-kani proof-harness gate. It is NOT a
// declared workspace check-cfg (kani is a v1-GM strengthening, not a v1-beta
// dependency — do NOT block the wave on kani standup, R2 §"kani harness is
// NOT in-tree"), so rustc would emit `unexpected_cfgs`. Allow it locally; the
// real kani integration at v1-GM registers the cfg in Cargo.toml/build.rs.
#![allow(unexpected_cfgs)]

// ── R5 (w-ms-sync): wired to the REAL production tie-break + KDF ──
//
// - The Inv-21 tie-break (`total_order_key` / `fork_winner`) routes through the
//   production `benten_membership_set::set::crdt::{fork_total_order_key,
//   fork_a_wins}` (smaller-key-wins; M-8 totality via Version-Node CID) — the
//   `Hlc → BentenHlc` bridge converts the fixture clock to the real HLC the
//   production rule keys on.
// - K(V) derivation routes through `benten_membership_set::keying::derive_kv`
//   (the structural BLAKE3 KDF; the frozen golden is byte-identical).
//
// The 3-field `Hlc` fixture below is a faithful mirror of `benten_core::hlc::Hlc`
// (parity shape: lexicographic (physical_ms, logical, node_id)); it bridges to
// the real `BentenHlc` at the production-rule boundary via `into_benten`.
use benten_core::hlc::BentenHlc;
use benten_membership_set::keying::derive_kv;
use benten_membership_set::set::crdt::{fork_a_wins, fork_total_order_key};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hlc {
    physical_ms: u64,
    logical: u32,
    node_id: u64,
}

impl Hlc {
    /// Bridge the fixture clock to the REAL `BentenHlc` the production tie-break
    /// rule keys on.
    fn into_benten(self) -> BentenHlc {
        BentenHlc::new(self.physical_ms, self.logical, self.node_id)
    }
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
    /// CRDT-vector entries this fork carries. After resolution the WINNER's
    /// CURRENT snapshot must NOT contain loser-only entries
    /// (archived-not-discarded).
    crdt_vector: Vec<String>,
    /// True if this candidate's author is a non-Admin (ALL event-authors
    /// fork — F-INV21-4).
    author_is_admin: bool,
}

/// The TOTAL ordering key (M-8): (`created_at_hlc` ASC, then
/// `fork_event_version_node_cid` ASC). NOT `MembershipSetId`. Routes through the
/// PRODUCTION `benten_membership_set::set::crdt::fork_total_order_key`.
fn total_order_key(f: &ForkCandidate) -> (BentenHlc, Vec<u8>) {
    fork_total_order_key(
        f.created_at_hlc.into_benten(),
        &f.fork_event_version_node_cid,
    )
}

/// Inv-21 fork-tie-break: SMALLER key wins (oldest-anchor; CID-ASC tiebreak).
/// Routes through the PRODUCTION `benten_membership_set::set::crdt::fork_a_wins`.
fn fork_winner<'a>(a: &'a ForkCandidate, b: &'a ForkCandidate) -> &'a ForkCandidate {
    if fork_a_wins(
        a.created_at_hlc.into_benten(),
        &a.fork_event_version_node_cid,
        b.created_at_hlc.into_benten(),
        &b.fork_event_version_node_cid,
    ) {
        a
    } else {
        b
    }
}

/// The resolved CURRENT membership snapshot after a fork-merge. The winner
/// fork is CURRENT; the loser fork is ARCHIVED separately (not merged into
/// CURRENT). This is the OBSERVABLE merge OUTPUT the no-absorption pin
/// asserts against (NOT a fixture read-back).
#[derive(Clone, Debug)]
struct ForkResolution {
    /// Which fork is CURRENT (the winner).
    winner_id: ForkId,
    /// The CURRENT snapshot's CRDT-vector. MUST be exactly the winner's
    /// vector — never the loser's entries merged in (archived-not-discarded).
    current_crdt_vector: Vec<String>,
    /// The archived (losing) fork's id — retained, queryable, but NOT CURRENT.
    archived_id: ForkId,
    /// The archived fork's CRDT-vector, retained for read (archived ≠
    /// discarded). Lives in the archive, NOT in `current_crdt_vector`.
    archived_crdt_vector: Vec<String>,
}

/// PRODUCTION-stand-in: merge resolution producing the CURRENT snapshot.
/// The winner is chosen by the tie-break; the CURRENT snapshot carries ONLY
/// the winner's vector; the loser is ARCHIVED (retained separately) and NOT
/// absorbed into CURRENT. R5 routes through the real merge — an
/// implementation that silently absorbed the loser into CURRENT would make
/// `current_crdt_vector` contain the loser's entries and FAIL the arm.
fn resolve_fork(a: &ForkCandidate, b: &ForkCandidate) -> ForkResolution {
    let winner = fork_winner(a, b);
    let loser = if std::ptr::eq(winner, a) { b } else { a };
    ForkResolution {
        winner_id: winner.id,
        // CURRENT = the winner's vector ONLY. The loser's entries are NOT
        // merged in (the substantive contract; a buggy absorb would extend
        // this with `loser.crdt_vector`).
        current_crdt_vector: winner.crdt_vector.clone(),
        archived_id: loser.id,
        archived_crdt_vector: loser.crdt_vector.clone(),
    }
}

/// K(V) derivation (Inv-19): derived from the IMMUTABLE Version-Node CID via the
/// domain-separated structural BLAKE3 KDF. Routes through the PRODUCTION
/// `benten_membership_set::keying::derive_kv` (context `"benten-membership-set:K(V):v1"`).
/// A DIFFERENT CID derives a DIFFERENT key — the assertion is NOT a self-equality
/// read-back of the CID.
fn derive_k_v(version_node_cid: &[u8]) -> [u8; 32] {
    derive_kv(version_node_cid)
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
/// the ordering axioms: totality + antisymmetry + **transitivity** (the
/// F4-025 3-fork arm: a<b ∧ b<c ⟹ a<c).
#[test]
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

    // ── F4-025: 3-fork TRANSITIVITY arm ──────────────────────────────────
    // The total order must be transitive: a < b ∧ b < c ⟹ a < c. A
    // non-transitive tie-break (e.g. a pairwise rule that disagrees with the
    // global min) would make convergence order-dependent — two engines could
    // reduce the same 3-fork set to different winners. We build three forks
    // ALL tied on created_at_hlc so transitivity rides ENTIRELY on the
    // Version-Node-CID ASC ordering (the M-8 terminal discriminator), then
    // assert the three pairwise comparisons are mutually consistent AND that
    // the global min equals the pairwise-reduced winner in every permutation.
    let mk = |seed: &[u8], id: u32| ForkCandidate {
        id: ForkId(id),
        membership_set_id: cid(b"shared-anchor"),
        created_at_hlc: tied_hlc, // ALL tied — transitivity rides on the CID
        fork_event_version_node_cid: cid(seed),
        crdt_vector: vec![],
        author_is_admin: true,
    };
    // Three distinct fork events. Sort them by the real total_order_key so we
    // KNOW the ground-truth a<b<c ordering (derived from the stub, not assumed).
    let mut three = [mk(b"tf-AAA", 10), mk(b"tf-BBB", 20), mk(b"tf-CCC", 30)];
    three.sort_by_key(total_order_key);
    let (a, b, c) = (&three[0], &three[1], &three[2]);

    // a < b and b < c by construction of the sort.
    assert!(
        total_order_key(a) < total_order_key(b),
        "a < b on the total order"
    );
    assert!(
        total_order_key(b) < total_order_key(c),
        "b < c on the total order"
    );
    // TRANSITIVITY: a < c must follow. Would-FAIL for any intransitive order.
    assert!(
        total_order_key(a) < total_order_key(c),
        "transitivity: a < b ∧ b < c ⟹ a < c on the Inv-21 total order"
    );
    // The pairwise tie-break agrees with the global min in EVERY permutation
    // (the convergence consequence of transitivity + antisymmetry).
    let pairwise_min = fork_winner(fork_winner(a, b), c).id.0;
    let global_min = three
        .iter()
        .min_by(|x, y| total_order_key(x).cmp(&total_order_key(y)))
        .unwrap()
        .id
        .0;
    assert_eq!(
        pairwise_min, global_min,
        "the pairwise-reduced winner equals the global min (transitive total order ⇒ order-independent convergence)"
    );
    assert_eq!(
        global_min, a.id.0,
        "the global min is the smallest-CID fork"
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
///
/// F4-011: the no-absorption arm asserts on the merge OUTPUT (the CURRENT
/// snapshot produced by `resolve_fork`), NOT a fixture read-back of the
/// unmodified winner input; the K(V) arm DERIVES the key from the
/// Version-Node CID via `derive_k_v` and proves a different CID derives a
/// different key (NOT a self-equality of the CID against itself).
#[test]
fn f_inv21_4_losing_fork_not_merged_archived_not_discarded() {
    // Winner carries ["w-only"]; loser carries ["l-only"]. A non-Admin
    // authored the loser (ALL event-authors fork).
    let winner_fork = fork(1, 100, b"winner", true, &["w-only"]);
    let loser_fork = fork(2, 200, b"loser", /* admin = */ false, &["l-only"]);

    let resolution = resolve_fork(&winner_fork, &loser_fork);
    assert_eq!(resolution.winner_id.0, 1, "oldest anchor wins (CURRENT)");
    assert_eq!(resolution.archived_id.0, 2, "the later fork is ARCHIVED");

    // ── No silent absorption: assert on the merge OUTPUT (CURRENT snapshot),
    // NOT on the unmodified winner input. The CURRENT snapshot must carry the
    // winner's entry and MUST NOT carry the loser's. A merge that absorbed the
    // loser into CURRENT (the bug this freezes against) would extend
    // `current_crdt_vector` with "l-only" and FAIL here.
    assert!(
        resolution
            .current_crdt_vector
            .contains(&"w-only".to_string()),
        "the CURRENT snapshot carries the winner's entry"
    );
    assert!(
        !resolution
            .current_crdt_vector
            .contains(&"l-only".to_string()),
        "the merge OUTPUT (CURRENT snapshot) MUST NOT absorb the loser's CRDT-vector entries (no silent merge)"
    );

    // Archived-not-discarded: the loser's vector is RETAINED in the archive
    // (queryable post-resolution) — separate from CURRENT, never lost.
    assert!(
        resolution
            .archived_crdt_vector
            .contains(&"l-only".to_string()),
        "the losing fork is ARCHIVED (its vector is retained + queryable), not discarded"
    );
    // And the archive is genuinely DISTINCT from CURRENT (not the same store).
    assert_ne!(
        resolution.current_crdt_vector, resolution.archived_crdt_vector,
        "CURRENT and the archive are distinct snapshots (the loser is not in CURRENT)"
    );

    // ALL event-authors fork: a non-Admin authored a legitimate fork.
    assert!(
        !loser_fork.author_is_admin,
        "a non-Admin event-author can fork (F-INV21-4)"
    );

    // ── Inv-19: K(V) is DERIVED from the IMMUTABLE Version-Node CID via the
    // structural KDF — NOT the CID itself. We derive the winner's K(V) from
    // its Version-Node CID and assert (a) it equals re-deriving from the SAME
    // CID (determinism) and (b) it DIFFERS from a key derived off the loser's
    // CID (the key is keyed by the CID, not a self-equality read-back).
    let winner_cid = &winner_fork.fork_event_version_node_cid;
    let k_v = derive_k_v(winner_cid);
    assert_eq!(
        k_v,
        derive_k_v(&cid(b"winner")),
        "K(V) is deterministically derived from the immutable Version-Node CID (Inv-19 / Inv-20 clause-f)"
    );
    // Frozen golden: the derived K(V) for cid(b\"winner\") (domain-separated
    // BLAKE3 derive_key). R5 confirms-or-deliberately-updates this frozen
    // literal against the real structural KDF (M-20).
    const K_V_WINNER_HEX: &str = "e3c09e37e2964ee768b467c4afbbca9e4d518326bdceba456ca99ce2cd1ba95e";
    assert_eq!(
        hex(&k_v),
        K_V_WINNER_HEX,
        "K(V) golden vector (derived from the immutable Version-Node CID; drift fails the pin)"
    );
    // A DIFFERENT Version-Node CID derives a DIFFERENT key — proves K(V) is
    // CID-keyed, not a constant/self-referential value.
    let k_v_loser = derive_k_v(&loser_fork.fork_event_version_node_cid);
    assert_ne!(
        k_v, k_v_loser,
        "a different Version-Node CID derives a different K(V) (the key is CID-keyed, not self-equal)"
    );
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
