//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 (benten-sync attack-fabric
//! lane) — §4.58 sync-attack 18-vector fabric (15-of-18 missing) +
//! §4.25 atrium-share CID+peer-DID verify at sync + §4.19(a) cross-peer
//! `accept_atrium_share` install seam.
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! At SYNCED HEAD `ed03729a` only 3 sync-attack files exist
//! (`attack_hlc_skew_revocation_ordering.rs` /
//! `attack_loro_op_log_inv_13.rs` / `attack_mst_diff_cid_mismatch.rs`
//! — 5 `#[test]` fns). §4.58 (refinement-audit #1100): the remaining
//! 15-of-18 vectors specified in the archived Phase-3 R2 test-landscape
//! were never landed. C8 is EXTENDED: the 15 missing vectors land OR
//! carry a HARD-RULE-12 disposition. The 3 narrative-named classes
//! §4.58 cites explicitly are pinned below as concrete RED adversarial
//! fixtures; the remaining 12 are a §3.6e + HARD-RULE-12 staged
//! enumeration anchor pointing at the §4.58 named destination (the
//! down-scope-vs-land Ben architectural call lands at G-CORE-8 — this
//! pin holds the obligation open, it does NOT silently drop it).
//!
//! §4.25: cross-Atrium plugin-share verification at the `benten-sync`
//! hydrate entry point (`bytes_cid == announced_cid` AND
//! `peer_did_signature_valid_for_bytes`) is NOT YET WIRED at the sync
//! layer (the single-peer `plugin_lifecycle::install_plugin` does the
//! check; the sync-layer hydrate does not re-verify). §4.19(a): the
//! cross-peer `accept_atrium_share` install seam (~300-500 LOC) is NOT
//! YET BUILT; the 3 stranded ignored cross-peer pins are §3.6e
//! redirect obligations un-ignored at G-CORE-8.
//!
//! §4.25 + §4.19(a) land TOGETHER at G-CORE-8 (the plan + backlog both
//! state §4.25 atrium-share-at-sync does NOT subsume the cross-peer
//! install pipeline).
//!
//! ## SUBSTANTIVE-arm-not-SHAPE shape (R4.1 fix-pass per pim-18 / §3.6f)
//!
//! Each RED arm below **first** exercises SHIPPED adjacent primitives
//! the future production sync-hydrate seams will consume — `Mst` /
//! `MstEntry` / `MstCid::from_bytes` for the CID-mismatch defense;
//! `benten_id::keypair::Keypair::{generate,sign,verify}` for the
//! peer-DID-signature defense; `MerkleProof::with_tampered_node` for
//! the tamper-detection contract — with a real assertion + observable
//! would-FAIL consequence, **then** `panic!`-holds the still-
//! undelivered sync-layer hydrate seam that re-verifies these
//! primitives on EVERY merge. The mostly-undelivered-target-surface
//! hybrid pattern (R4.1 pattern-induction): exercise SHIPPED adjacent
//! primitives + panic!-hold the missing structurally-always-on
//! re-verification at the hydrate entry point.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-2-amendment (§3.6b sub-rule-4): each vector exercises a
//!     SPECIFIC adversarial construction → expected typed ErrorCode at
//!     the production sync hydrate path; would-FAIL if the verify is
//!     a no-op.
//!   - pim-12 (§3.6e): RED-PHASE staged-pins; the 3 stranded cross-peer
//!     pins are an explicit redirect obligation; reviewer verifies
//!     landing-status not just spec-pin presence.
//!   - pim-18 (§3.6f): substantive adversarial bodies, NOT "an attack
//!     fixture file exists". Hybrid mostly-undelivered pattern: drive
//!     shipped primitives (Mst / MstEntry / MstCid / Keypair /
//!     MerkleProof) + panic-hold the missing hydrate re-verification.
//!   - §3.13 per-test-static decomposition: this module uses NO shared
//!     process-scoped static (each vector builds its own adversarial
//!     fixture) — the §3.13 obligation is discharged structurally; the
//!     cross-crate sweep §3.13 flag is recorded explicitly here per the
//!     R2 §4-D on-surface designation for the G-CORE-8 cross-crate lane.
//!   - §3.11 checkpoint-pre-flight recovery: TF-8 is the largest
//!     cross-crate family — on agent-kill resume INTO the same
//!     worktree, do NOT restart-fresh.
//!
//! Pins: G-CORE-8 · C8 (extended: 15-of-18 vectors land OR HARD-RULE-12
//! disposition) · §1.A.FROZEN item 12. R2 map: TF-8 RED-arm (6)+(7).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_id::keypair::Keypair;
use benten_sync::mst::{Mst, MstCid, MstEntry};

/// Build a small canonical MST with stable entries; returned together
/// with its root + a deterministic proof for one key (used to drive
/// the SHIPPED `MerkleProof::with_tampered_node` tamper-detection arm
/// across multiple vectors).
fn build_canonical_mst_and_proof() -> (Mst, MstCid, benten_sync::mst::MerkleProof) {
    let mut mst = Mst::new();
    mst.insert(MstEntry::from_payload("/zone/k1", vec![1, 2, 3]));
    mst.insert(MstEntry::from_payload("/zone/k2", vec![4, 5, 6]));
    let root = mst.root_cid();
    let proof = mst
        .merkle_proof_for("/zone/k1")
        .expect("present key has a proof");
    (mst, root, proof)
}

// ===========================================================================
// §4.58 — the 3 narrative-named sync-attack vector classes (concrete RED)
// ===========================================================================

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.58 sync-attack-1 \
            peer-impersonation — vector lands OR HARD-RULE-12 disposition)"]
fn sync_attack_1_peer_impersonation_rejected_at_hydrate() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (substantive-arm anchor for pim-18 §3.6f):
    // peer-impersonation reduces to "signature claims peer A but
    // doesn't verify under A's public key". Drive the SHIPPED
    // `benten_id::keypair::Keypair::sign`/`PublicKey::verify` primitive
    // on a real adversarial construction (attacker signs frames + the
    // recipient looks them up under the claimed-victim's key). Real
    // assertion + observable would-FAIL consequence.
    // -----------------------------------------------------------------
    let victim_kp = Keypair::generate();
    let attacker_kp = Keypair::generate();
    let frame_bytes = b"sync-frame-payload-from-claimed-peer";

    // Attacker signs the frame with their own key but claims it
    // originated from `victim_kp`. The SHIPPED verify primitive MUST
    // refuse — the signature does not verify under victim's public
    // key. Would-FAIL signal: if the verify primitive regressed to
    // accept-any-valid-sig, the assertion below fires.
    let attacker_sig = attacker_kp.sign(frame_bytes);
    assert!(
        victim_kp
            .public_key()
            .verify(frame_bytes, &attacker_sig)
            .is_err(),
        "shipped surface exercise: an attacker's signature MUST NOT \
         verify under the victim's public key (the substrate the \
         peer-impersonation defense composes on). Would-FAIL if the \
         SHIPPED verify primitive regressed."
    );
    // Sanity: the victim's own signature DOES verify (positive control).
    let victim_sig = victim_kp.sign(frame_bytes);
    victim_kp
        .public_key()
        .verify(frame_bytes, &victim_sig)
        .expect("victim's own signature must verify (primitive sanity)");

    // -----------------------------------------------------------------
    // RED-arm: the sync-layer hydrate path does NOT yet thread the
    // SHIPPED verify primitive across every claimed-peer-DID lookup —
    // a hostile peer can declare a peer-DID in the frame header
    // without the hydrate path consulting the signature.
    // -----------------------------------------------------------------
    panic!(
        "§4.58 sync-attack-1 (peer-impersonation) undelivered: the SHIPPED \
         Keypair verify primitive correctly refuses the cross-key signature \
         (exercised above), but the sync-layer hydrate does not yet \
         re-verify the peer-DID signature binds frames to the claimed \
         peer-DID on every merge. Lands at G-CORE-8 OR carries a \
         HARD-RULE-12 down-scope disposition against §4.58."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.58 sync-attack-7 \
            signature-substitution — vector lands OR HARD-RULE-12)"]
fn sync_attack_7_signature_substitution_rejected_at_hydrate() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: signature-substitution = "valid sig +
    // mutated payload". Drive the SHIPPED verify primitive on a
    // mutated-payload adversarial construction (signature valid for
    // payload P, but presented against payload P').
    // -----------------------------------------------------------------
    let peer_kp = Keypair::generate();
    let payload_a = b"original-payload";
    let payload_b = b"substituted-payload";
    let sig_a = peer_kp.sign(payload_a);

    // Verify the signature for payload_a against payload_b: MUST fail
    // (the SHIPPED primitive's payload-hash binding is the substrate
    // the sync-layer re-verify will consume).
    assert!(
        peer_kp.public_key().verify(payload_b, &sig_a).is_err(),
        "shipped surface exercise: a signature for payload_a MUST NOT \
         verify against payload_b (Ed25519 signature↔payload-hash \
         binding). Would-FAIL if the primitive regressed to a \
         payload-agnostic accept."
    );
    // Positive control: same signature verifies against original payload.
    peer_kp
        .public_key()
        .verify(payload_a, &sig_a)
        .expect("signature for payload_a must verify against payload_a");

    // -----------------------------------------------------------------
    // RED-arm: the sync-layer hydrate path does NOT yet enforce
    // signature↔payload-hash binding on every row — the substrate
    // primitive works (above), but it is not threaded through hydrate.
    // -----------------------------------------------------------------
    panic!(
        "§4.58 sync-attack-7 (signature-substitution) undelivered: the \
         SHIPPED Keypair verify primitive correctly refuses the mutated- \
         payload signature (exercised above), but the hydrate path does \
         not yet enforce signature↔payload-hash binding on every merged \
         row. Lands at G-CORE-8 OR HARD-RULE-12 disposition."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.58 sync-attack-11 \
            audience-rebinding — vector lands OR HARD-RULE-12)"]
fn sync_attack_11_audience_rebinding_rejected_at_hydrate() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: audience-rebinding is detectable
    // because the audience is part of the payload the signature binds.
    // Drive the SHIPPED verify primitive on a "valid-sig-for-audience-A
    // + replayed-with-audience-B" construction.
    // -----------------------------------------------------------------
    let issuer_kp = Keypair::generate();
    let original_payload = b"ucan{cap:store:notes:write, aud:plugin-A, exp:42}";
    let rebound_payload = b"ucan{cap:store:notes:write, aud:plugin-B, exp:42}";
    let sig = issuer_kp.sign(original_payload);

    // Signature for original audience does NOT verify against the
    // rebound audience: the substrate the audience-rebinding defense
    // composes on works.
    assert!(
        issuer_kp
            .public_key()
            .verify(rebound_payload, &sig)
            .is_err(),
        "shipped surface exercise: a signature over `aud:plugin-A` MUST \
         NOT verify over `aud:plugin-B` (the substrate for the audience- \
         rebinding defense). Would-FAIL if the primitive regressed."
    );

    // -----------------------------------------------------------------
    // RED-arm: the sync-merge per-row recheck does not yet detect
    // audience mismatch on every merged row — Phase-3 G16-B-B added
    // audience-binding at the DELEGATION layer; the SYNC-LAYER per-row
    // recheck is the §4.58 obligation.
    // -----------------------------------------------------------------
    panic!(
        "§4.58 sync-attack-11 (audience-rebinding) undelivered: the SHIPPED \
         Keypair verify primitive correctly refuses the audience-rebound \
         signature (exercised above), but the sync-merge per-row recheck \
         does not yet detect audience rebinding on every merged row. Lands \
         at G-CORE-8 OR HARD-RULE-12 disposition."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.58 — the remaining \
            12-of-18 sync-attack vectors: enumerate-and-land OR \
            HARD-RULE-12 down-scope disposition; §3.6e)"]
fn sync_attack_remaining_12_vectors_land_or_hard_rule_12_disposition() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (structural enumeration backstop):
    // construct the 12 vector ATTACK NAMES as a hand-enumerated
    // `&'static [&str]` list — the #907-style structural would-FAIL
    // pin (one row per missing vector, dropping a row is a build-time
    // observable). At G-CORE-8 the implementer either replaces each
    // entry with a real `#[test]` (the additive landing) or records
    // the HARD-RULE-12 down-scope disposition for the named entry.
    // -----------------------------------------------------------------
    const REMAINING_12_VECTORS: &[&str] = &[
        "sync-attack-2-peer-sybil",
        "sync-attack-3-frame-replay-cross-zone",
        "sync-attack-4-revocation-skip",
        "sync-attack-5-out-of-order-hlc",
        "sync-attack-6-expired-grant-merge",
        "sync-attack-8-doc-substitution",
        "sync-attack-9-orphan-row-injection",
        "sync-attack-10-cap-attenuation-bypass",
        "sync-attack-12-clock-skew-replay-window",
        "sync-attack-13-handshake-replay",
        "sync-attack-14-truncation",
        "sync-attack-15-empty-merge-side-channel",
    ];
    assert_eq!(
        REMAINING_12_VECTORS.len(),
        12,
        "structural enumeration backstop: the 12 named vectors must be \
         exactly enumerated (would-FAIL if a vector is silently dropped \
         or duplicated). §4.58 + the archived Phase-3 R2 test-landscape \
         spec are the named source of truth."
    );
    // No duplicates (would-FAIL signal: deduping the list and getting a
    // shorter result reveals a typo/duplicate).
    let mut sorted: Vec<&str> = REMAINING_12_VECTORS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        REMAINING_12_VECTORS.len(),
        "structural enumeration backstop: vector names must be unique"
    );

    // -----------------------------------------------------------------
    // RED-arm: each entry must EITHER land as a real `#[test]` against
    // the production sync-hydrate path OR receive a HARD-RULE-12 named
    // down-scope disposition (Compromise #22/#23/#25/#26 narrative
    // sharpen + named v1-assessment-window re-scope row). This pin
    // holds the obligation OPEN.
    // -----------------------------------------------------------------
    panic!(
        "§4.58 remaining-12-of-18 sync-attack vectors undelivered: the \
         enumeration backstop above structurally lists the 12 named \
         vectors (exercised: count + uniqueness invariants verified). \
         Each must EITHER land as a real adversarial fixture, OR record \
         the §4.58 HARD-RULE-12 down-scope disposition (SECURITY-POSTURE \
         Compromise #22/#23/#25/#26 narrative + re-scope as a named \
         v1-assessment-window item). Ben architectural call at G-CORE-8 \
         — this pin holds the obligation open, not dropped."
    );
}

// ===========================================================================
// §4.25 — atrium-share CID + peer-DID verification at the sync hydrate layer
// ===========================================================================

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.25 atrium-share bytes \
            don't match announced CID rejected at sync hydrate)"]
fn atrium_share_bytes_not_matching_announced_cid_rejected_at_sync_hydrate() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the substitution defense reduces to
    // "recompute the CID over received bytes + compare to announced".
    // Drive the SHIPPED `MstCid::from_bytes` primitive on a real
    // substitution adversarial construction — announce CID(A), present
    // bytes(B); the recomputed CID(B) MUST differ.
    // -----------------------------------------------------------------
    let bytes_a = b"announced-plugin-bytes-version-1";
    let bytes_b = b"substituted-plugin-bytes-attacker"; // different payload
    let cid_announced = MstCid::from_bytes(bytes_a);
    let cid_recomputed_from_substitute = MstCid::from_bytes(bytes_b);

    assert_ne!(
        cid_announced, cid_recomputed_from_substitute,
        "shipped surface exercise: substituted bytes MUST yield a \
         different CID (BLAKE3 content-addressing — the substrate the \
         §4.25 substitution defense composes on). Would-FAIL if the \
         CID computation regressed to a constant or input-agnostic value."
    );
    // Positive control: re-hashing the same bytes yields the same CID.
    let cid_a_again = MstCid::from_bytes(bytes_a);
    assert_eq!(
        cid_announced, cid_a_again,
        "shipped surface exercise: re-hashing identical bytes must yield \
         the same CID (determinism)"
    );

    // -----------------------------------------------------------------
    // RED-arm: the sync-layer hydrate entry does NOT yet recompute the
    // CID over received bytes and compare to the announced CID before
    // landing them into the ManifestStore. The substrate primitive
    // (exercised above) is correct; the hydrate-level enforcement is
    // the §4.25 obligation.
    // -----------------------------------------------------------------
    panic!(
        "§4.25 atrium-share CID-verify undelivered at sync hydrate: the \
         SHIPPED `MstCid::from_bytes` primitive correctly distinguishes \
         substituted bytes (exercised above), but the benten-sync \
         hydrate entry that lands received plugin bytes into the \
         ManifestStore does NOT re-verify `bytes_cid == announced_cid` \
         on every merge. Lands at G-CORE-8 (with §4.19(a))."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.25 atrium-share \
            substitution with different author rejected at sync hydrate)"]
fn atrium_share_substitution_with_different_author_rejected_at_sync_hydrate() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: peer-DID-signature-mismatch defense
    // reduces to "verify the signature under the announced author's
    // public key". Drive the SHIPPED verify primitive on
    // `(bytes, announced_author_pk, attacker_sig)` — MUST refuse.
    // -----------------------------------------------------------------
    let announced_author_kp = Keypair::generate();
    let attacker_kp = Keypair::generate();
    let plugin_bytes = b"plugin-bytes-that-hash-to-the-announced-cid";

    // Bytes hash to the announced CID (positive substitute defense
    // half — assert these two stay in sync).
    let cid_of_bytes = MstCid::from_bytes(plugin_bytes);
    let cid_recomputed = MstCid::from_bytes(plugin_bytes);
    assert_eq!(cid_of_bytes, cid_recomputed, "CID determinism positive");

    // Attacker signs the (identical) bytes with their OWN key but the
    // announced author is `announced_author_kp`. The recipient
    // looks up the signature under the announced author's public key
    // — MUST refuse.
    let attacker_sig = attacker_kp.sign(plugin_bytes);
    assert!(
        announced_author_kp
            .public_key()
            .verify(plugin_bytes, &attacker_sig)
            .is_err(),
        "shipped surface exercise: an attacker-signed bytes blob MUST \
         NOT verify under the announced author's public key (the \
         substrate the §4.25 peer-DID-signature-mismatch defense \
         composes on). Would-FAIL if the verify primitive regressed."
    );

    // -----------------------------------------------------------------
    // RED-arm: the sync-layer hydrate entry does NOT yet re-verify the
    // peer-DID signature for the announced author on every merged row.
    // -----------------------------------------------------------------
    panic!(
        "§4.25 atrium-share peer-DID-signature-verify undelivered at \
         sync hydrate: the SHIPPED Keypair verify primitive correctly \
         refuses cross-key signatures (exercised above), but the \
         hydrate entry does NOT re-verify the peer-DID signature for \
         the announced author on every merge. Lands G-CORE-8."
    );
}

// ===========================================================================
// §4.19(a) — cross-peer accept_atrium_share install seam (+ 3 stranded pins)
// ===========================================================================

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.19(a) cross-peer \
            accept_atrium_share install seam — re-verifies bytes_cid \
            == announced_cid AND peer_did_signature_valid_for_bytes; \
            3 stranded ignored cross-peer pins un-ignored per §3.6e)"]
fn cross_peer_accept_atrium_share_install_seam_reverifies_cid_and_peer_did() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: drive BOTH halves the cross-peer
    // install seam will compose on: (a) `MstCid::from_bytes` for the
    // CID re-verify; (b) `Keypair::sign/verify` for the peer-DID
    // signature re-verify. Real assertions + observable would-FAIL.
    // -----------------------------------------------------------------
    let peer_a_kp = Keypair::generate();
    let plugin_bytes = b"plugin-bytes-shared-from-peer-A";
    let cid = MstCid::from_bytes(plugin_bytes);

    // (a) CID re-verify: tampering even one byte changes the CID.
    let mut tampered = plugin_bytes.to_vec();
    tampered[0] ^= 0x01;
    let cid_tampered = MstCid::from_bytes(&tampered);
    assert_ne!(
        cid, cid_tampered,
        "shipped surface exercise (cross-peer install seam half (a)): a \
         single-byte flip MUST change the CID (BLAKE3 collision \
         resistance). Would-FAIL if CID computation regressed."
    );

    // (b) peer-DID signature re-verify: peer-A signs the bytes; the
    // signature verifies under peer-A's key but NOT under a foreign
    // key.
    let peer_a_sig = peer_a_kp.sign(plugin_bytes);
    peer_a_kp
        .public_key()
        .verify(plugin_bytes, &peer_a_sig)
        .expect("peer-A's own signature must verify (positive)");
    let foreign_kp = Keypair::generate();
    assert!(
        foreign_kp
            .public_key()
            .verify(plugin_bytes, &peer_a_sig)
            .is_err(),
        "shipped surface exercise (cross-peer install seam half (b)): \
         peer-A's signature MUST NOT verify under a foreign peer's key. \
         Would-FAIL if the verify primitive regressed."
    );

    // -----------------------------------------------------------------
    // RED-arm: the cross-peer install pipeline (~300-500 LOC) is NOT
    // YET BUILT — the substrate primitives are correct (above), but
    // the seam that composes them at receive time (re-verify CID +
    // peer-DID signature, hydrate into ManifestStore, local-anchored
    // InstallRecord consent) is undelivered.
    // -----------------------------------------------------------------
    panic!(
        "§4.19(a) cross-peer accept_atrium_share install seam \
         undelivered: the SHIPPED `MstCid::from_bytes` + Keypair \
         sign/verify primitives exercised above prove the substrate is \
         correct, but the sync-layer cross-peer install pipeline \
         (re-verify CID + peer-DID signature, hydrate into \
         ManifestStore, local-anchored InstallRecord consent) is NOT \
         YET BUILT. Lands at G-CORE-8 with §4.25; the 3 stranded \
         ignored cross-peer pins are §3.6e-redirected + un-ignored."
    );
}

// ===========================================================================
// §4.37 — InstallRecord replay-defense: checked-AND-atomically-recorded
//          (no verify-then-record TOCTOU) + consulted at sync-hydrate
// ===========================================================================

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.37 InstallRecord \
            replay-defense — seen-or-record checked-AND-atomically-\
            recorded, no TOCTOU; consulted at sync-hydrate)"]
fn install_record_replay_defense_no_toctou_consulted_at_sync_hydrate() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the replay-defense reduces to "the
    // same nonce bytes must collide". Drive the SHIPPED MerkleProof
    // tamper-detection contract as the structural would-FAIL signal —
    // the substrate for "did I see this exact bytes-shaped artifact
    // before" is content-addressed equality, exercised here against
    // the MerkleProof tamper-detection mechanism that the future
    // replay-store will share (both are content-addressed equality
    // over a payload).
    // -----------------------------------------------------------------
    let (_mst, root, proof) = build_canonical_mst_and_proof();
    let tampered = proof.with_tampered_node();

    // Reconstruct-root contract: ORIGINAL proof reconstructs to root,
    // tampered proof does NOT. Would-FAIL if reconstruct_root became
    // input-agnostic (which would simultaneously break the tamper-
    // detection contract the replay store will compose on for nonce-
    // shaped artifacts).
    assert_eq!(
        proof.reconstruct_root(),
        root,
        "shipped surface exercise: original proof must reconstruct to \
         the published root (positive control for content-addressed \
         equality — the substrate for the replay-defense store)"
    );
    assert_ne!(
        tampered.reconstruct_root(),
        root,
        "shipped surface exercise: a tampered proof's reconstructed \
         root MUST differ from the published root (the substrate the \
         replay-defense seen-or-record check will compose on — same \
         content-addressed equality semantics). Would-FAIL if the \
         primitive regressed."
    );

    // -----------------------------------------------------------------
    // RED-arm: no seen-or-record nonce store consulted at install
    // Step-4 or at sync-hydrate; check-AND-atomically-record contract
    // is undelivered; the `E_PLUGIN_INSTALL_RECORD_REPLAY` ErrorCode
    // is not yet minted.
    // -----------------------------------------------------------------
    panic!(
        "§4.37 InstallRecord replay-defense undelivered: the SHIPPED \
         content-addressed-equality substrate (exercised above via \
         MerkleProof tamper-detection) is correct, but no seen-or-record \
         nonce store consulted at install Step-4 or at sync-hydrate; a \
         captured InstallRecord replays freely. G-CORE-8 must add the \
         port with check-AND-atomic-record (no TOCTOU) + the \
         E_PLUGIN_INSTALL_RECORD_REPLAY ErrorCode."
    );
}
