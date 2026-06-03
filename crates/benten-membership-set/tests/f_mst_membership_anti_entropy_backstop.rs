//! F-MST-1/2/3 (R3-W5) — Membership MST anti-entropy convergence backstop.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-MST-1** (G1 + GNI-26),
//!   **F-MST-2** (G2), **F-MST-3** (G3).
//! - R0.5 plan §3.9 (M-10): "**Convergence is backed by the EXISTING MST
//!   anti-entropy** (`benten-sync/src/mst.rs`); **iroh-gossip is
//!   LIVENESS/notification ONLY**". The MST diff + HLC causal order +
//!   Inv-21 partition rule are the convergence BACKSTOP — gossip's
//!   no-causal-delivery / out-of-order / duplicate properties do NOT
//!   threaten convergence.
//! - §3.9 (#52 revocation-vs-membership-write ordering); reuse the in-tree
//!   `mst_diff.rs` / `attack_mst_diff_cid_mismatch.rs` /
//!   `mst_revocation_priority.rs` shapes.
//!
//! ## What this pins
//!
//! - F-MST-1: a divergent membership event-set anti-entropies to
//!   convergence within `MAX_ROUNDS` (O(log n)); the backstop holds
//!   INDEPENDENT of gossip.
//! - F-MST-2: an MST entry whose declared CID ≠ its payload BLAKE3 is
//!   rejected at the app layer (substitution defense).
//! - F-MST-3: a kick/revocation at HLC=T is applied BEFORE any membership
//!   write at HLC<T from the revoked party (#52 ordering priority), AND a
//!   revocation that is itself STALE (HLC ≤ the write) does NOT dominate
//!   (the symmetric negative control — proves the priority rule is value-
//!   keyed totality, not a tautology).
//!
//! ## pim-2 §3.6b + §3.6f-ext end-to-end discipline
//!
//! Drives the PRODUCTION `run_mst_diff_to_convergence` /
//! `verify_entry_cid` / `apply_in_revocation_priority_order` stand-ins;
//! asserts OBSERVABLE converged event-set + typed rejection + ordering;
//! would-FAIL-if-no-op'd (a degraded-to-O(n) diff blows MAX_ROUNDS; a
//! no-op CID check accepts the substitution; an always-`Revoked` /
//! ignore-the-HLC ordering FAILS the symmetric negative control in
//! F-MST-3).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`; SELF-CONTAINED stub-shim for
//! parallel-safe R3. R5 swaps in `benten_sync::mst` + `benten_membership_set`
//! and un-ignores.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

// ── SELF-CONTAINED stub-shim ──

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MstEntry {
    key: String,
    declared_cid: Vec<u8>,
    payload: Vec<u8>,
}

impl MstEntry {
    /// Honest constructor: declared CID = BLAKE3 of payload (the in-tree
    /// `MstEntry::from_payload` contract).
    fn from_payload(key: &str, payload: Vec<u8>) -> Self {
        let cid = blake3_cid(&payload);
        MstEntry {
            key: key.to_string(),
            declared_cid: cid,
            payload,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum MstDiffError {
    MaxRoundsExceeded,
    CidMismatch,
}

/// PRODUCTION-stand-in: anti-entropy to convergence. Exchanges
/// missing-in-A / missing-in-B sets until both sides hold the same entry
/// set, bounded by `max_rounds`. Returns the round count (O(log n)
/// observable). R5 routes through `benten_sync::mst::run_mst_diff_to_convergence`.
fn run_mst_diff_to_convergence(
    a: &mut BTreeSet<MstEntry>,
    b: &mut BTreeSet<MstEntry>,
    max_rounds: usize,
) -> Result<usize, MstDiffError> {
    let mut rounds = 0;
    loop {
        if a == b {
            return Ok(rounds);
        }
        if rounds >= max_rounds {
            return Err(MstDiffError::MaxRoundsExceeded);
        }
        // Each round halves the divergence (model of MST log-depth diff).
        let missing_in_a: Vec<MstEntry> = b
            .difference(a)
            .take(b.difference(a).count().div_ceil(2))
            .cloned()
            .collect();
        let missing_in_b: Vec<MstEntry> = a
            .difference(b)
            .take(a.difference(b).count().div_ceil(2))
            .cloned()
            .collect();
        for e in missing_in_a {
            a.insert(e);
        }
        for e in missing_in_b {
            b.insert(e);
        }
        rounds += 1;
    }
}

/// PRODUCTION-stand-in: app-layer CID-verification (substitution defense).
/// R5 routes through the in-tree `attack_mst_diff_cid_mismatch.rs` path.
fn verify_entry_cid(entry: &MstEntry) -> Result<(), MstDiffError> {
    if blake3_cid(&entry.payload) == entry.declared_cid {
        Ok(())
    } else {
        Err(MstDiffError::CidMismatch)
    }
}

fn blake3_cid(payload: &[u8]) -> Vec<u8> {
    let d = blake3::hash(payload);
    let mut c = vec![0x01u8, 0x71, 0x1e, 0x20];
    c.extend_from_slice(d.as_bytes());
    c
}

// ── F-MST-1 ─────────────────────────────────────────────────────────────

/// F-MST-1 — divergent membership event-set converges within MAX_ROUNDS
/// (O(log n)); the backstop holds INDEPENDENT of gossip.
#[test]
#[ignore = "RED-PHASE: F-MST-1 — membership MST diff converges in O(log n) rounds; un-ignore at R5"]
fn f_mst_1_membership_event_set_converges_log_n() {
    let event_count = 4096usize;
    let mut peer_a: BTreeSet<MstEntry> = BTreeSet::new();
    let mut peer_b: BTreeSet<MstEntry> = BTreeSet::new();
    for i in 0..event_count {
        let entry = MstEntry::from_payload(&format!("evt-{i:06}"), format!("v{i}").into_bytes());
        match i % 4 {
            0 => {
                peer_a.insert(entry);
            }
            1 => {
                peer_b.insert(entry);
            }
            _ => {
                peer_a.insert(entry.clone());
                peer_b.insert(entry);
            }
        }
    }
    // O(log n): with n≈4096, ~12 rounds suffice. The bound is generous
    // (32) yet would FAIL if the diff degraded to O(n). `ilog2` keeps the
    // bound an integer computation (no lossy usize→f64 cast).
    let max_rounds = 32;
    let rounds = run_mst_diff_to_convergence(&mut peer_a, &mut peer_b, max_rounds).unwrap();
    assert_eq!(
        peer_a, peer_b,
        "membership event-sets MUST converge to the same set"
    );
    let log_n_bound = (event_count.ilog2() as usize) + 4;
    assert!(
        rounds <= log_n_bound,
        "convergence MUST be O(log n) rounds (got {rounds}); a degraded O(n) diff would blow this"
    );

    // MAX_ROUNDS-exceeded surfaces a typed error (would-FAIL-if-no-op'd).
    let mut disjoint_x: BTreeSet<MstEntry> = BTreeSet::new();
    let mut disjoint_y: BTreeSet<MstEntry> = BTreeSet::new();
    for i in 0..1000 {
        disjoint_x.insert(MstEntry::from_payload(&format!("x{i}"), vec![i as u8]));
    }
    for i in 0..1000 {
        disjoint_y.insert(MstEntry::from_payload(&format!("y{i}"), vec![i as u8]));
    }
    assert_eq!(
        run_mst_diff_to_convergence(&mut disjoint_x, &mut disjoint_y, 1),
        Err(MstDiffError::MaxRoundsExceeded),
        "exceeding MAX_ROUNDS surfaces a typed MstDiffError"
    );
}

// ── F-MST-2 ─────────────────────────────────────────────────────────────

/// F-MST-2 — MST entry CID-mismatch substitution defense. An entry whose
/// declared CID ≠ payload BLAKE3 is rejected; a matching one is accepted.
#[test]
#[ignore = "RED-PHASE: F-MST-2 — MST entry CID-mismatch rejected (substitution defense); un-ignore at R5"]
fn f_mst_2_cid_mismatch_substitution_rejected() {
    // Honest entry: accepted.
    let honest = MstEntry::from_payload("evt-1", b"genuine-membership-event".to_vec());
    assert!(
        verify_entry_cid(&honest).is_ok(),
        "a genuine entry verifies"
    );

    // Substituted payload with the original's declared CID: rejected.
    let mut substituted = honest.clone();
    substituted.payload = b"FORGED-membership-event".to_vec(); // CID no longer matches
    assert_eq!(
        verify_entry_cid(&substituted),
        Err(MstDiffError::CidMismatch),
        "a payload-substituted entry MUST be rejected (declared CID ≠ BLAKE3(payload))"
    );
}

// ── F-MST-3 ─────────────────────────────────────────────────────────────

/// F-MST-3 — revocation-vs-membership-write ordering priority (#52).
/// A kick/revocation that strictly DOMINATES (HLC=T) is applied BEFORE a
/// membership write at HLC<T from the revoked party (the stale write
/// loses). The SYMMETRIC NEGATIVE CONTROL — a revocation that is itself
/// stale (HLC ≤ the write) does NOT dominate — makes the priority rule
/// genuinely falsifiable: an always-`Revoked` or HLC-ignoring impl FAILS
/// the negative arm. (The rule is value-keyed on HLC totality, NOT
/// arrival order — the function takes no arrival-order parameter — so the
/// negative control is the load-bearing falsifier, not an arrival-order
/// re-run.)
#[test]
#[ignore = "RED-PHASE: F-MST-3 — revocation ordered ahead of stale membership write (#52); un-ignore at R5"]
fn f_mst_3_revocation_ordered_ahead_of_stale_write() {
    // Apply a kick of DID-X at T, and X's stale write at T-1. The kick
    // must win when (and only when) its HLC strictly dominates.
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Effect {
        Revoked,
        StaleWriteApplied,
    }
    // PRODUCTION-stand-in: priority application — revocation at HLC=T
    // dominates any write at HLC<T from the revoked party. A revocation
    // whose HLC does NOT strictly exceed the write's HLC is itself the
    // stale one and loses.
    fn apply_in_revocation_priority_order(revocation_hlc: u64, stale_write_hlc: u64) -> Effect {
        if revocation_hlc > stale_write_hlc {
            Effect::Revoked
        } else {
            Effect::StaleWriteApplied
        }
    }
    let t = 100u64;

    // Positive arm: kick at T dominates a stale write at T-1 — the stale
    // write loses (#52).
    assert_eq!(
        apply_in_revocation_priority_order(t, t - 1),
        Effect::Revoked,
        "kick at T dominates a stale write at T-1 from the revoked party (#52)"
    );

    // SYMMETRIC NEGATIVE CONTROL: a revocation that is itself stale
    // (HLC = T-1) against a newer write (HLC = T) does NOT dominate. This
    // is the load-bearing falsifier — an always-`Revoked` or HLC-ignoring
    // impl FAILS here, where the positive arm alone could not catch it.
    assert_eq!(
        apply_in_revocation_priority_order(t - 1, t),
        Effect::StaleWriteApplied,
        "a STALE revocation (HLC ≤ the write) MUST NOT dominate — proves the priority \
         rule is value-keyed HLC totality, not an always-revoke no-op"
    );

    // Boundary: equal HLC is a tie that does NOT grant the revocation
    // dominance (strict `>` only) — pins the non-strict edge so a `>=`
    // regression is caught.
    assert_eq!(
        apply_in_revocation_priority_order(t, t),
        Effect::StaleWriteApplied,
        "equal HLC is NOT strict dominance — a `>=` regression would wrongly revoke here"
    );
}
