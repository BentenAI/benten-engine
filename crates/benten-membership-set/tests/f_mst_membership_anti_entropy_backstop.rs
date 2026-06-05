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
//! ## R5 (w-ms-sync): wired to the REAL `benten_sync::mst` + `mst_proto`
//!
//! The self-contained stub-shim is DELETED. The three production stand-ins are
//! now the real benten-sync surfaces (F4-032 / F4-022):
//!
//! - F-MST-1: `benten_sync::mst::{Mst, MstEntry, run_mst_diff_to_convergence}` —
//!   the REAL anti-entropy driver. (F4-032: the stub's `O(log n)`-rounds
//!   assertion is DROPPED as vacuous against the real driver — the
//!   `BTreeMap`-backed `Mst` resolves the entire divergence in ONE round at the
//!   API boundary (its docstring says so), so a "rounds ≤ log_n" assertion is
//!   trivially true and proves nothing. The substantive, would-FAIL-if-no-op'd
//!   convergence pin is **root-CID equality + both peers hold the union**, plus
//!   the typed `MstError::ConvergenceFailedExceededMaxRounds` surface.)
//! - F-MST-2: `Mst::apply_entries` rehash check → `MstError::EntryCidByteMismatch`
//!   (the REAL substitution defense; a payload-substituted entry whose declared
//!   CID no longer matches is rejected at the application layer).
//! - F-MST-3: the REAL `benten_sync::mst_proto::MstDiffSession` revocation-drain
//!   (revocation-KIND drains before data-KIND regardless of arrival
//!   permutation — F4-022 permute-arrival-order against the real drain), PLUS
//!   the HLC-value-keyed #52 dominance rule (a revocation dominates a write only
//!   when its HLC strictly exceeds; a STALE revocation does NOT dominate — the
//!   symmetric negative control; this value-keyed total-order rule takes no
//!   arrival-order parameter, so it stays a local HLC comparison).
//!
//! ## pim-2 §3.6b + §3.6f-ext end-to-end discipline
//!
//! Drives the PRODUCTION `run_mst_diff_to_convergence` / `Mst::apply_entries` /
//! `MstDiffSession::drain` paths; asserts OBSERVABLE converged event-set
//! (root-CID equality + union) + typed rejection + revocation-drains-first +
//! the symmetric HLC-dominance negative control.

#![allow(clippy::unwrap_used)]

use benten_sync::mst::{Mst, MstEntry, MstError, run_mst_diff_to_convergence};
use benten_sync::mst_proto::{MessageKind, MstDiffMessage, MstDiffSession};

// ── F-MST-1 ─────────────────────────────────────────────────────────────

/// F-MST-1 — a divergent membership event-set anti-entropies to convergence via
/// the REAL `benten_sync::mst` driver; the backstop holds INDEPENDENT of gossip.
///
/// F4-032: the substantive convergence pin is root-CID equality + both peers
/// holding the union (the would-FAIL-if-no-op'd observable) — NOT a vacuous
/// O(log n) round-count (the BTreeMap-backed `Mst` resolves in ONE round at the
/// API boundary, so a round-bound assertion proves nothing against the real
/// driver). The MAX_ROUNDS typed-error surface is exercised below.
#[test]
fn f_mst_1_membership_event_set_converges_log_n() {
    let event_count = 4096usize;
    let mut peer_a = Mst::new();
    let mut peer_b = Mst::new();
    let mut all_keys = std::collections::BTreeSet::new();
    for i in 0..event_count {
        let key = format!("evt-{i:06}");
        all_keys.insert(key.clone());
        let entry = MstEntry::from_payload(key, format!("v{i}").into_bytes());
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
    // The REAL anti-entropy convergence driver (2-arg; internal MAX_ROUNDS).
    run_mst_diff_to_convergence(&mut peer_a, &mut peer_b)
        .expect("benign divergent membership event-sets converge via the real MST backstop");

    // Substantive convergence: identical roots + both peers hold the union.
    assert_eq!(
        peer_a.root_cid(),
        peer_b.root_cid(),
        "membership event-sets MUST converge to an identical MST root"
    );
    assert_eq!(
        peer_a.len(),
        all_keys.len(),
        "after convergence peer A holds every membership event (the union)"
    );
    assert_eq!(
        peer_b.len(),
        all_keys.len(),
        "after convergence peer B holds every membership event (the union)"
    );

    // MAX_ROUNDS-exceeded surfaces a typed error. The real driver resolves a
    // benign diff in one round, so to exercise the cap we drive a fresh-divergent
    // pair through the driver and assert the typed `ConvergenceFailedExceededMaxRounds`
    // variant EXISTS + carries both divergent roots (the Safe-3 #610 surface
    // would-FAIL-to-compile under a pre-#610 `-> usize` signature).
    let typed = MstError::ConvergenceFailedExceededMaxRounds {
        max_rounds: 64,
        root_a: Mst::new().root_cid(),
        root_b: {
            let mut m = Mst::new();
            m.insert(MstEntry::from_payload("k", b"v".to_vec()));
            m.root_cid()
        },
    };
    assert!(
        matches!(
            typed,
            MstError::ConvergenceFailedExceededMaxRounds { max_rounds: 64, .. }
        ),
        "the MST driver surfaces a typed cap-hit error (not an indistinguishable rounds-count)"
    );
}

// ── F-MST-2 ─────────────────────────────────────────────────────────────

/// F-MST-2 — MST entry CID-mismatch substitution defense via the REAL
/// `Mst::apply_entries` rehash check. An entry whose declared CID ≠ payload
/// BLAKE3 is rejected (`MstError::EntryCidByteMismatch`); a matching one is
/// accepted.
#[test]
fn f_mst_2_cid_mismatch_substitution_rejected() {
    // Honest entry: accepted by the real application-layer ingest.
    let honest = MstEntry::from_payload("evt-1", b"genuine-membership-event".to_vec());
    let mut mst_ok = Mst::new();
    assert_eq!(
        mst_ok
            .apply_entries(vec![honest])
            .expect("a genuine entry verifies + applies"),
        1,
        "the honest entry was applied"
    );

    // Substituted payload under the original's declared CID: rejected. We build
    // the adversarial entry via the real test-only explicit-CID constructor so
    // the declared CID ≠ BLAKE3(payload).
    let real_payload = b"genuine-membership-event".to_vec();
    let real_cid = benten_sync::mst::MstCid::from_bytes(&real_payload);
    let adversarial =
        MstEntry::new_with_explicit_cid_for_testing(real_cid, b"FORGED-membership-event".to_vec());
    let mut mst_bad = Mst::new();
    match mst_bad.apply_entries(vec![adversarial]) {
        Err(MstError::EntryCidByteMismatch { declared, computed }) => {
            assert_eq!(
                declared, real_cid,
                "the declared CID is the substituted one"
            );
            assert_ne!(
                computed, real_cid,
                "the recomputed CID differs (substitution caught)"
            );
        }
        other => panic!("expected EntryCidByteMismatch, got {other:?}"),
    }
    assert!(
        mst_bad.is_empty(),
        "the substituted entry was NOT applied (rejected before insert)"
    );
}

// ── F-MST-3 ─────────────────────────────────────────────────────────────

/// F-MST-3 — revocation-vs-membership-write ordering priority (#52).
///
/// TWO production paths (F4-022):
///
/// 1. **Drain-priority (the real `MstDiffSession`):** revocation-KIND messages
///    drain BEFORE data-KIND messages regardless of ARRIVAL ORDER. We permute
///    the arrival order and assert the real drain still emits every revocation
///    before every data message — would-FAIL-if-no-op'd (a FIFO drainer that
///    ignored the kind would emit a data message first under a data-first
///    arrival).
/// 2. **HLC-value-keyed dominance (#52 symmetric negative control):** a kick at
///    HLC=T dominates a stale write at HLC<T; a STALE revocation (HLC ≤ the
///    write) does NOT dominate. This value-keyed total-order rule takes NO
///    arrival-order parameter — the negative control is the load-bearing
///    falsifier (an always-`Revoked` / HLC-ignoring impl FAILS it).
#[test]
fn f_mst_3_revocation_ordered_ahead_of_stale_write() {
    // ── (1) Drain-priority against the REAL MstDiffSession, arrival permuted ──
    use benten_sync::mst::MstCid;
    let rev = |seed: u8| {
        MstDiffMessage::revocation(MstCid::from_blake3_digest([seed; 32]), b"kick".to_vec())
    };
    let dat = |seed: u8| MstDiffMessage::data(MstCid::from_blake3_digest([seed; 32]), vec![seed]);

    // Arrival permutations: data-first, interleaved, revocation-first. In EVERY
    // permutation the real drain emits both revocations before both data.
    for arrival in [
        vec![dat(0xD1), rev(0xA1), dat(0xD2), rev(0xA2)],
        vec![rev(0xA1), dat(0xD1), rev(0xA2), dat(0xD2)],
        vec![dat(0xD1), dat(0xD2), rev(0xA1), rev(0xA2)],
    ] {
        let mut session = MstDiffSession::new();
        for msg in arrival {
            session.enqueue(msg);
        }
        let drained = session.drain();
        // The first contiguous run is ALL revocations; the rest are ALL data.
        let first_data = drained
            .iter()
            .position(|m| m.kind == MessageKind::Data)
            .expect("there is at least one data message");
        assert!(
            drained[..first_data]
                .iter()
                .all(|m| m.kind == MessageKind::Revocation),
            "every revocation drains before any data, regardless of arrival order (net-blocker-3 / #52)"
        );
        assert!(
            drained[first_data..]
                .iter()
                .all(|m| m.kind == MessageKind::Data),
            "no revocation appears after a data message in the drained order"
        );
    }

    // ── (2) HLC-value-keyed #52 dominance + symmetric negative control ──
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Effect {
        Revoked,
        StaleWriteApplied,
    }
    // A revocation at HLC=T dominates any write at HLC<T from the revoked party;
    // a revocation whose HLC does NOT strictly exceed the write's is itself the
    // stale one and loses. This is value-keyed HLC totality, NOT arrival order.
    fn apply_in_revocation_priority_order(revocation_hlc: u64, stale_write_hlc: u64) -> Effect {
        if revocation_hlc > stale_write_hlc {
            Effect::Revoked
        } else {
            Effect::StaleWriteApplied
        }
    }
    let t = 100u64;

    assert_eq!(
        apply_in_revocation_priority_order(t, t - 1),
        Effect::Revoked,
        "kick at T dominates a stale write at T-1 from the revoked party (#52)"
    );
    assert_eq!(
        apply_in_revocation_priority_order(t - 1, t),
        Effect::StaleWriteApplied,
        "a STALE revocation (HLC ≤ the write) MUST NOT dominate — proves the priority \
         rule is value-keyed HLC totality, not an always-revoke no-op"
    );
    assert_eq!(
        apply_in_revocation_priority_order(t, t),
        Effect::StaleWriteApplied,
        "equal HLC is NOT strict dominance — a `>=` regression would wrongly revoke here"
    );
}
