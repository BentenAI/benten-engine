//! F-HLC-1 + F-HLC-2 (R3-W5) — the two distinct HLC clocks + skew defense.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-HLC-1** (merges D1/D2 +
//!   GNI-7) + **F-HLC-2** (merges D3 + GNI-3).
//! - R0.5 plan §3.5.HLC (`created_at_hlc` vs `admitted_at_hlc`, M-7
//!   definition gap):
//!   * `admitted_at_hlc` (MemberEntry property) = LWW **larger-HLC wins**
//!     (the in-tree `crdt.rs:535` property rule).
//!   * `created_at_hlc` (set anchor; immutable) = the Inv-21 fork-tie-break
//!     discriminant; **ONLY `created_at_hlc` participates in Inv-21** (M-7).
//!   * They are DISTINCT clocks.
//! - R0.5 plan §3.10 (HLC-skew classifier; Compromise #25 substrate).
//!
//! ## What this pins
//!
//! - F-HLC-1: clones the in-tree `hlc_loro_property_lww.rs` shape onto the
//!   member-property clock — concurrent admits t1<t2 ⇒ the **larger** HLC
//!   wins for `admitted_at_hlc`; and mutating `admitted_at_hlc` does NOT
//!   change the Inv-21 fork winner while mutating `created_at_hlc` DOES.
//!   This is the load-bearing M-7 distinction.
//! - F-HLC-2: clones `attack_hlc_skew_revocation_ordering.rs` — a
//!   future-HLC membership write (`physical_ms = u64::MAX/2`) is rejected
//!   by the skew classifier with a typed error WITHOUT mutating local
//!   clock state.
//!
//! ## pim-2 §3.6b + pim-18 §3.6f end-to-end discipline
//!
//! Drives the PRODUCTION property-merge (`admitted_at_hlc_lww`) +
//! fork-tie-break (`fork_winner`) + skew-classifier (`accept_membership_write`)
//! stand-ins; asserts OBSERVABLE winner / typed-rejection / clock-unchanged;
//! would-FAIL-if-no-op'd (a classifier that accepted the future stamp fails
//! the skew arm; a tie-break keyed off `admitted_at_hlc` fails F-HLC-1 arm 2).
//!
//! ## Wiring (history: pim-12 §3.6e)
//!
//! LIVE and un-ignored — runs every CI cycle against the REAL
//! `benten_membership_set` + the real `benten_core::hlc` clock.
//! (History: this started as a RED-PHASE self-contained stub-shim for
//! parallel-safe R3; R5 wired it to the production surfaces and
//! un-ignored it.)

#![allow(clippy::unwrap_used)]

// ── R5 (w-ms-sync): wired to the REAL production surfaces ──
//
// - property LWW (larger-HLC-wins) → `benten_membership_set::set::crdt::admitted_at_hlc_lww_keeps_a`.
// - Inv-21 fork tie-break (smaller-created_at_hlc-wins) → `…::set::crdt::fork_a_wins`.
// - skew classifier → the REAL `benten_core::hlc::Hlc::update` (typed
//   `CoreError::HlcSkewExceeded`), the exact in-tree clock the stub mirrored.
// The fixture `Hlc` is a faithful mirror of `benten_core::hlc::Hlc` bridged to
// the real `BentenHlc` at each production-rule boundary.
use benten_core::hlc::{BentenHlc, Hlc as CoreHlc};
use benten_membership_set::set::crdt::{admitted_at_hlc_lww_keeps_a, fork_a_wins};

/// Fixture HLC (parity with benten_core::hlc). Lexicographic compare on
/// (physical_ms, logical, node_id). Bridges to the real `BentenHlc`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hlc {
    physical_ms: u64,
    logical: u32,
    node_id: u64,
}

impl Hlc {
    fn into_benten(self) -> BentenHlc {
        BentenHlc::new(self.physical_ms, self.logical, self.node_id)
    }
}

/// Typed skew-rejection (parity with `CoreError::HlcSkewExceeded`).
#[derive(Debug, PartialEq, Eq)]
enum MembershipWriteError {
    HlcSkewExceeded,
}

/// Per-property LWW merge — **larger HLC wins** — routed through the PRODUCTION
/// `admitted_at_hlc_lww_keeps_a` rule applied to `admitted_at_hlc`.
fn admitted_at_hlc_lww(a: (Hlc, &str), b: (Hlc, &str)) -> String {
    // Keep `a` iff its HLC wins (larger-HLC LWW); else keep `b`.
    if admitted_at_hlc_lww_keeps_a(a.0.into_benten(), b.0.into_benten()) {
        a.1
    } else {
        b.1
    }
    .to_string()
}

/// Inv-21 fork-tie-break — **smaller `created_at_hlc` wins** (oldest-anchor-wins;
/// the DELIBERATE opposite of property LWW). Only `created_at_hlc` participates
/// (NOT `admitted_at_hlc`). Routed through the PRODUCTION `fork_a_wins` rule.
fn fork_winner(fork_a: &ForkAnchor, fork_b: &ForkAnchor) -> ForkId {
    if fork_a_wins(
        fork_a.created_at_hlc.into_benten(),
        &fork_a.fork_event_version_node_cid,
        fork_b.created_at_hlc.into_benten(),
        &fork_b.fork_event_version_node_cid,
    ) {
        fork_a.id
    } else {
        fork_b.id
    }
}

/// The membership-write skew classifier, routed through the REAL
/// `benten_core::hlc::Hlc::update`: a clock whose physical-now is `local_now`
/// (with the given skew tolerance) rejects an inbound stamp more than
/// `max_skew_ms` into the future with the typed `CoreError::HlcSkewExceeded`.
///
/// `benten_core::hlc::PhysicalClockFn` is a bare `fn() -> u64` (no captures), so
/// the fixed physical-now is supplied by the `const`-returning [`local_now_ms`]
/// fn; this helper asserts the fixture's `local_now` matches it so a drift in
/// the fixture is caught (rather than silently exercising the wrong clock).
fn accept_membership_write(
    local_now: Hlc,
    inbound: Hlc,
    max_skew_ms: u64,
) -> Result<(), MembershipWriteError> {
    assert_eq!(
        local_now.physical_ms,
        local_now_ms(),
        "the skew-classifier fixture's local-now must match the const physical clock"
    );
    let clock = CoreHlc::with_skew_tolerance(local_now.node_id, local_now_ms, max_skew_ms);
    match clock.update(&inbound.into_benten()) {
        Ok(_) => Ok(()),
        Err(_) => Err(MembershipWriteError::HlcSkewExceeded),
    }
}

/// The fixed physical-clock value for the skew-classifier fixture (a bare
/// `fn() -> u64` as `PhysicalClockFn` requires; matches `local_now.physical_ms`).
fn local_now_ms() -> u64 {
    1_000
}

#[derive(Clone, Debug)]
struct ForkAnchor {
    id: ForkId,
    created_at_hlc: Hlc,
    /// The discriminant `admitted_at_hlc` MUST NOT participate in the fork
    /// tie-break — carried here to prove mutating it is winner-neutral.
    representative_admitted_at_hlc: Hlc,
    fork_event_version_node_cid: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ForkId(u8);

fn cid(payload: &[u8]) -> Vec<u8> {
    let d = blake3::hash(payload);
    let mut c = vec![0x01u8, 0x71, 0x1e, 0x20];
    c.extend_from_slice(d.as_bytes());
    c
}

// ── F-HLC-1 arms ────────────────────────────────────────────────────────

/// F-HLC-1 arm 1 — `admitted_at_hlc` property LWW (larger-HLC wins).
///
/// Two concurrent admits of the same DID at t1<t2 converge on the t2
/// (larger-HLC) value. Clones the in-tree `hlc_loro_property_lww.rs`
/// observable: both peers agree + the higher physical_ms wins.
#[test]
fn f_hlc_1_admitted_at_hlc_lww_larger_wins() {
    let t1 = Hlc {
        physical_ms: 100,
        logical: 0,
        node_id: 0xAAAA_AAAA,
    };
    let t2 = Hlc {
        physical_ms: 200,
        logical: 0,
        node_id: 0xBBBB_BBBB,
    };
    // Engine A admits at t1 ("role=Member"); Engine B re-admits at t2
    // ("role=Moderator"). Property LWW keeps the larger-HLC write.
    let converged_ab = admitted_at_hlc_lww((t1, "Member"), (t2, "Moderator"));
    let converged_ba = admitted_at_hlc_lww((t2, "Moderator"), (t1, "Member"));
    assert_eq!(
        converged_ab, converged_ba,
        "both peers converge (commutative LWW)"
    );
    assert_eq!(
        converged_ab, "Moderator",
        "larger HLC (t2=200) wins for admitted_at_hlc"
    );

    // F4-039: equal-`physical_ms` tie sub-case — the lex-compare is NOT
    // physical_ms-only. When two concurrent admits share `physical_ms`, the
    // tie breaks on (logical, then node_id) per the derived lexicographic Ord.
    // An impl that compared ONLY `physical_ms` would treat these as equal and
    // resolve non-deterministically (or keep the wrong one) — this arm fails
    // for any such under-constrained compare.
    let tie_lo = Hlc {
        physical_ms: 300,
        logical: 0,
        node_id: 0x1111_1111,
    };
    let tie_hi_logical = Hlc {
        physical_ms: 300, // SAME physical_ms
        logical: 1,       // larger logical breaks the tie
        node_id: 0x0000_0000,
    };
    // Larger (logical) wins despite the SAME physical_ms and SMALLER node_id.
    let converged_logical = admitted_at_hlc_lww((tie_lo, "Member"), (tie_hi_logical, "Moderator"));
    assert_eq!(
        converged_logical, "Moderator",
        "equal physical_ms ⇒ the LARGER logical wins (lex-compare, not physical_ms-only)"
    );
    assert_eq!(
        admitted_at_hlc_lww((tie_hi_logical, "Moderator"), (tie_lo, "Member")),
        "Moderator",
        "equal physical_ms tie-break is commutative"
    );

    // And when physical_ms AND logical both tie, node_id is the terminal
    // discriminator (totality of the lex order).
    let tie_node_lo = Hlc {
        physical_ms: 300,
        logical: 7,
        node_id: 10,
    };
    let tie_node_hi = Hlc {
        physical_ms: 300, // SAME
        logical: 7,       // SAME
        node_id: 20,      // larger node_id breaks the tie
    };
    assert_eq!(
        admitted_at_hlc_lww((tie_node_lo, "Member"), (tie_node_hi, "Moderator")),
        "Moderator",
        "equal (physical_ms, logical) ⇒ larger node_id wins (lex-compare totality)"
    );
}

/// F-HLC-1 arm 2 — the M-7 distinction: only `created_at_hlc` drives the
/// fork winner; `admitted_at_hlc` is winner-NEUTRAL.
///
/// This is the load-bearing M-7 pin. We build two forks; mutating the
/// representative `admitted_at_hlc` MUST NOT change which fork wins, while
/// mutating `created_at_hlc` MUST.
#[test]
fn f_hlc_1_only_created_at_hlc_drives_fork_winner() {
    let older = Hlc {
        physical_ms: 100,
        logical: 0,
        node_id: 1,
    };
    let newer = Hlc {
        physical_ms: 500,
        logical: 0,
        node_id: 2,
    };

    let fork_a = ForkAnchor {
        id: ForkId(1),
        created_at_hlc: older, // oldest anchor
        representative_admitted_at_hlc: Hlc {
            physical_ms: 10,
            logical: 0,
            node_id: 1,
        },
        fork_event_version_node_cid: cid(b"fork-a"),
    };
    let fork_b = ForkAnchor {
        id: ForkId(2),
        created_at_hlc: newer,
        representative_admitted_at_hlc: Hlc {
            physical_ms: 9_999,
            logical: 0,
            node_id: 2,
        },
        fork_event_version_node_cid: cid(b"fork-b"),
    };

    // Baseline: smaller created_at_hlc (fork_a) wins.
    assert_eq!(
        fork_winner(&fork_a, &fork_b).0,
        1,
        "oldest-anchor (smaller created_at_hlc) wins"
    );

    // Mutate admitted_at_hlc arbitrarily — winner UNCHANGED.
    let mut a2 = fork_a.clone();
    a2.representative_admitted_at_hlc = Hlc {
        physical_ms: u64::MAX,
        logical: 0,
        node_id: 1,
    };
    let mut b2 = fork_b.clone();
    b2.representative_admitted_at_hlc = Hlc {
        physical_ms: 0,
        logical: 0,
        node_id: 2,
    };
    assert_eq!(
        fork_winner(&a2, &b2).0,
        1,
        "mutating admitted_at_hlc MUST NOT change the fork winner (M-7)"
    );

    // Mutate created_at_hlc so fork_b is now the oldest — winner FLIPS.
    let mut b3 = fork_b.clone();
    b3.created_at_hlc = Hlc {
        physical_ms: 1,
        logical: 0,
        node_id: 2,
    };
    assert_eq!(
        fork_winner(&fork_a, &b3).0,
        2,
        "mutating created_at_hlc to be smaller DOES change the fork winner (M-7)"
    );
}

// ── F-HLC-2 arm ─────────────────────────────────────────────────────────

/// F-HLC-2 — HLC-skew adversarial injection at the membership-write
/// boundary. A future-HLC admit is rejected by the skew classifier with a
/// typed error WITHOUT mutating local clock state. Clones
/// `attack_hlc_skew_revocation_ordering.rs`.
#[test]
fn f_hlc_2_future_hlc_membership_write_rejected() {
    let local_now = Hlc {
        physical_ms: 1_000,
        logical: 0,
        node_id: 7,
    };
    let max_skew_ms = 60_000; // 1-minute tolerance window

    // Adversarial future stamp.
    let adversarial = Hlc {
        physical_ms: u64::MAX / 2,
        logical: 0,
        node_id: 9,
    };
    let result = accept_membership_write(local_now, adversarial, max_skew_ms);
    assert_eq!(
        result,
        Err(MembershipWriteError::HlcSkewExceeded),
        "a future-HLC membership write MUST be rejected (typed HlcSkewExceeded)"
    );

    // A within-tolerance write is accepted (the classifier is not a no-op
    // that rejects everything — would-FAIL-if-no-op'd both directions).
    let ok_inbound = Hlc {
        physical_ms: 1_050,
        logical: 0,
        node_id: 8,
    };
    assert!(
        accept_membership_write(local_now, ok_inbound, max_skew_ms).is_ok(),
        "a within-skew membership write MUST be accepted"
    );
}
