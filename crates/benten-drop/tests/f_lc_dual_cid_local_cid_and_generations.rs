//! F-full R3-W2 (Layer-C) — DUAL-CID + local/set CIDs + key-generations.
//!
//! ADDL Phase-4-Meta-Core, **F-full** R3 wave **W2-layer-c** (RED-PHASE,
//! `pim-12 §3.6e`). Families pinned in THIS file:
//!   - **F-LC-4** DUAL-CID `envelope_blob_cid` vs `plaintext_cid`
//!     (extends in-tree `TwoCidStore` — O-3; modeled by the self-contained
//!     stub `two_cid_stub` here for parallel-safety).
//!   - **F-LC-5** `plaintext_cid_local` NEVER-serialized +
//!     `plaintext_cid_set` HMAC-blinded (Inv-20 clause-d; O-7).
//!   - **F-LC-6** `recipient_key_generation` + `k_principal_generation`
//!     staleness rejection (U19/U20).
//!
//! Pin sources (spec of record is now **R0.7** =
//! `111cca9c:.addl/phase-4-meta/f-full-r0-plan.md`; minted against R0.3 =
//! `4fe9236a:.addl/phase-4-meta/f-full-r0-plan.md`; §-numbers below are
//! stable R0.3→R0.5→R0.7 — DUAL-CID / generation-staleness surface is
//! byte-unchanged R0.5→R0.7. R0.7 adds ONE precision: the abstract
//! `plaintext_cid_set` "HMAC" blinding is `blake3::keyed_hash(K_Set, ·)` —
//! the native BLAKE3 keyed MAC (no hmac/sha2 dep; bytes unchanged), the SAME
//! primitive `f_aad_2`'s `membership_set_id_commitment` + the §3.9 gossip
//! topic use (see `blind_set_cid` doc below):
//!   - §3.3 DUAL-CID (Q3, U18): `envelope_blob_cid = BLAKE3(serialized
//!     EncryptedEnvelope)` (changes on reseal) vs `plaintext_cid =
//!     BLAKE3(canonical DropBundlePayload)` (stable, graph-referenced);
//!     "MembershipSet adds `plaintext_cid_local` (LOCAL-ONLY,
//!     NEVER-serialized — O-7) + `plaintext_cid_set` (HMAC-blinded)";
//!     "This EXTENDS `benten-sync/src/two_cid_store.rs` (O-3) — not
//!     net-new."
//!   - §3.3 Inv-16 mint + key-retention: `recipient_key_generation` bound
//!     per-stanza (U19); §4.1 `recipient_key_generation` +
//!     `k_principal_generation` tracking FREEZE (U19/U20).
//!   - R2 landscape §1 Group 7: F-LC-4 (~4-6), F-LC-5 (~5-7; proptest
//!     floor, kani named as v1-GM strengthening), F-LC-6 (~3-4; GAP-6a
//!     U20 — BOTH generation fields).
//!   - §10.8 R3 seed: "`plaintext_cid_local` NEVER-serialized
//!     kani/property test (O-7)."
//!
//! # RED-PHASE STATUS + STUB-SHIM DISCIPLINE (pim-12 §3.6e).
//!
//! At baseline the in-tree `benten-sync::two_cid_store::TwoCidStore` maps
//! plaintext_cid → ciphertext_cid but has NO `envelope_blob_cid` /
//! `plaintext_cid_local` / `plaintext_cid_set` / generation surface. To
//! keep this R3 wave PARALLEL-SAFE (the brief forbids depending on another
//! wave's crate/module), this file carries a SELF-CONTAINED `two_cid_stub`
//! mirroring the intended DUAL-CID extension. The Layer-C closing-wave R5
//! implementer MUST:
//!   1. DELETE `two_cid_stub` + add the `benten-sync` dev-dependency,
//!   2. INSERT `use benten_sync::two_cid_store::{TwoCidStore, …};`,
//!   3. UN-IGNORE each test, 4. Verify green.
//!
//! # Wave-0 DAG edge (M-20). All CIDs/generations authored as BE bytes /
//! V2 semantics; the `plaintext_cid_set` blinding is HMAC over BE-encoded
//! inputs. NO LE/V1 vector.
//!
//! # Production-arm + would-FAIL (pim-2 sub-rule-4 + pim-18 + §3.6f-ext).
//! Every test drives a production call site (`put_dual` / `reseal` /
//! `blind_set_cid` / `verify_stanza_generation`) + observable consequence.
//! Stub bodies `unimplemented!()` so a forgotten stub at R5 PANICS.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_variables)]

// ===========================================================================
// R5 — real DUAL-CID production surface (`benten_sync::two_cid_store`). The
// self-contained stub is DELETED; the `DualCidStore` (aliased to the former
// `TwoCidStore` name) + the `DualCid` digest type (aliased to `Cid`) + the
// reseal / blind / wire-surface / generation fns are imported here.
// ===========================================================================

use benten_sync::two_cid_store::{
    DualCid as Cid, DualCidStore as TwoCidStore, GenError, Stanza, all_wire_serialization_surfaces,
    blind_set_cid, plaintext_cid, reseal, verify_stanza_generation,
};

fn cid(seed: u8) -> Cid {
    [seed; 32]
}

// ===========================================================================
// F-LC-4 — DUAL-CID: plaintext_cid stable, envelope_blob_cid changes on
// reseal (extends TwoCidStore — O-3).
// ===========================================================================

/// F-LC-4 PIN 1 — RESEAL keeps `plaintext_cid` equal but makes
/// `envelope_blob_cid` DISTINCT. The plaintext_cid is the stable graph
/// reference; the blob_cid is the transport handle that must rotate when
/// the bytes change. would-FAIL if reseal reused the same blob_cid (then
/// re-encryption is observable as the same handle) OR changed the
/// plaintext_cid (then every graph reference breaks).
#[test]
fn f_lc_4_reseal_stable_plaintext_cid_distinct_blob_cid() {
    let payload = b"the same canonical DropBundlePayload bytes".to_vec();

    let (pt1, blob1) = reseal(&payload, 0x01);
    let (pt2, blob2) = reseal(&payload, 0x02); // re-encrypt with a fresh nonce

    assert_eq!(
        pt1, pt2,
        "F-LC-4: `plaintext_cid` = BLAKE3(canonical payload) MUST be STABLE \
         across reseal (it is the graph-referenced identity). would-FAIL if \
         reseal changed the plaintext_cid."
    );
    assert_ne!(
        blob1, blob2,
        "F-LC-4: `envelope_blob_cid` = BLAKE3(serialized envelope) MUST \
         CHANGE on reseal (fresh nonce ⇒ different serialized bytes). \
         would-FAIL if reseal reused the same transport handle."
    );
    // The canonical plaintext_cid helper agrees with the reseal output.
    assert_eq!(
        plaintext_cid(&payload),
        pt1,
        "F-LC-4: the standalone plaintext_cid(payload) MUST equal the \
         reseal-returned plaintext_cid (one canonical digest)."
    );
}

/// F-LC-4 PIN 2 — the store resolves a graph-referenced `plaintext_cid`
/// to its CURRENT transport blob handle (round-trip through the extended
/// `TwoCidStore`). would-FAIL if the store keyed on the blob_cid instead
/// of the stable plaintext_cid (then graph references couldn't resolve).
#[test]
fn f_lc_4_two_cid_store_resolves_plaintext_to_current_blob() {
    let mut store = TwoCidStore::new();
    let pt = cid(0x11);
    let blob = cid(0x22);
    let local = cid(0x33);

    store.put_dual(pt, blob, local);

    assert_eq!(
        store.resolve_blob(&pt),
        Some(blob),
        "F-LC-4: a graph reference (plaintext_cid) MUST resolve to the \
         CURRENT envelope_blob_cid through the extended TwoCidStore (O-3). \
         would-FAIL if the store keyed on the blob handle."
    );
}

// ===========================================================================
// F-LC-5 — plaintext_cid_local NEVER-serialized + plaintext_cid_set
// HMAC-blinded (Inv-20 clause-d; O-7). proptest floor; kani named for
// v1-GM strengthening.
// ===========================================================================

/// F-LC-5 PIN 1 — the `plaintext_cid_local` sentinel appears in NO wire
/// serialization surface (envelope bytes, AAD, gossip topic, audit-Node).
/// This is the O-7 LOCAL-ONLY property. would-FAIL if any wire surface
/// embedded the local CID (then local dedup state leaks to observers).
#[test]
fn f_lc_5_plaintext_cid_local_never_serialized() {
    // A distinctive sentinel byte pattern so a substring scan is unambiguous.
    let local_sentinel: Cid = [0xAB; 32];

    let surfaces = all_wire_serialization_surfaces(&local_sentinel);
    assert!(
        !surfaces.is_empty(),
        "F-LC-5: the enumeration MUST return the real wire surfaces (env / \
         AAD / topic / audit-Node) — an empty list would make the scan \
         vacuous."
    );
    for (i, surface) in surfaces.iter().enumerate() {
        let leaks = surface
            .windows(local_sentinel.len())
            .any(|w| w == local_sentinel);
        assert!(
            !leaks,
            "F-LC-5 (O-7): `plaintext_cid_local` MUST NOT appear in wire \
             surface #{i}. It is LOCAL-ONLY — never serialized to any \
             envelope/AAD/topic/audit-Node. would-FAIL if local dedup \
             state leaked onto the wire. [kani arm named as v1-GM \
             strengthening; proptest is the v1-beta floor.]"
        );
    }
}

/// F-LC-5 PIN 2 — `plaintext_cid_set` is HMAC-blinded under `K_Set`: a
/// single-bit flip in `K_Set` MUST change the blinded output (no
/// cross-set linkage). would-FAIL if the blinding were keyless (then two
/// sets dedup the same content to the SAME set-CID, linking them).
#[test]
fn f_lc_5_plaintext_cid_set_k_set_bit_flip_changes_output() {
    let pt = cid(0x44);
    let mut k_set = [0x07u8; 32];
    let blinded_a = blind_set_cid(&pt, &k_set);

    // Flip a single bit of K_Set.
    k_set[0] ^= 0x01;
    let blinded_b = blind_set_cid(&pt, &k_set);

    assert_ne!(
        blinded_a, blinded_b,
        "F-LC-5: a single-bit flip in K_Set MUST change plaintext_cid_set \
         (HMAC-blinded, set-scoped). would-FAIL if blinding were keyless — \
         then the same content in two different sets would dedup to the \
         SAME set-CID, enabling cross-set linkage (Inv-20 clause-d)."
    );
}

/// F-LC-5 PIN 3 — cross-set UNLINKABILITY: the SAME plaintext under two
/// DIFFERENT `K_Set`s yields DIFFERENT `plaintext_cid_set`s, so an
/// observer cannot link the two sets by their dedup CIDs. would-FAIL if
/// the blinding collapsed to a content-only digest.
#[test]
fn f_lc_5_same_content_two_sets_unlinkable() {
    let pt = cid(0x55);
    let k_set_a = [0x10u8; 32];
    let k_set_b = [0x20u8; 32];

    let in_set_a = blind_set_cid(&pt, &k_set_a);
    let in_set_b = blind_set_cid(&pt, &k_set_b);

    assert_ne!(
        in_set_a, in_set_b,
        "F-LC-5: the SAME content shared into two different MembershipSets \
         MUST get DIFFERENT plaintext_cid_set values (cross-set \
         unlinkability). would-FAIL if the set-CID were a content-only \
         (keyless) digest."
    );
    // Determinism within a set (the dedup property must still hold).
    assert_eq!(
        blind_set_cid(&pt, &k_set_a),
        in_set_a,
        "F-LC-5: within ONE set, the blinded set-CID MUST be deterministic \
         (so dedup works). The blinding is per-set, not per-call."
    );
}

// ===========================================================================
// F-LC-6 — recipient_key_generation + k_principal_generation staleness
// (U19/U20; GAP-6a binds BOTH generation fields).
// ===========================================================================

/// F-LC-6 PIN 1 — a stanza under a STALE `recipient_key_generation` (U19)
/// is rejected at verify. The current generation has advanced past the
/// stanza's; the stanza is stale. would-FAIL if the verify ignored the
/// recipient_key_generation field.
#[test]
fn f_lc_6_stale_recipient_key_generation_rejected() {
    let stanza = Stanza {
        recipient_key_generation: 3, // authored at gen 3
        k_principal_generation: 0,
    };
    // Recipient has rotated to gen 5; the stanza is stale.
    let outcome = verify_stanza_generation(&stanza, 5, 0);
    assert!(
        matches!(
            outcome,
            Err(GenError::StaleRecipientKeyGeneration {
                stanza: 3,
                current: 5
            })
        ),
        "F-LC-6: a stanza authored at recipient_key_generation=3 MUST be \
         rejected once the recipient has advanced to gen 5 (U19). \
         would-FAIL if verify ignored recipient_key_generation. Got: {outcome:?}"
    );
}

/// F-LC-6 PIN 2 — a stanza under a STALE `k_principal_generation` after a
/// K_principal rotation (U20 / GAP-6a) is rejected. This is the SECOND
/// generation field; binding only recipient_key_generation would leave
/// U20 unenforced. would-FAIL if verify ignored k_principal_generation.
#[test]
fn f_lc_6_stale_k_principal_generation_rejected() {
    let stanza = Stanza {
        recipient_key_generation: 5, // recipient-key is current
        k_principal_generation: 1,   // but K_principal has since rotated
    };
    // K_principal advanced to gen 2 after rotation.
    let outcome = verify_stanza_generation(&stanza, 5, 2);
    assert!(
        matches!(
            outcome,
            Err(GenError::StaleKPrincipalGeneration {
                stanza: 1,
                current: 2
            })
        ),
        "F-LC-6 (GAP-6a / U20): a stanza authored under k_principal_generation=1 \
         MUST be rejected after K_principal rotates to gen 2 — even when \
         recipient_key_generation is current. would-FAIL if only U19 were \
         bound. Got: {outcome:?}"
    );
}

/// F-LC-6 PIN 3 — a stanza CURRENT on BOTH generations verifies cleanly
/// (positive control — proves the rejections are not vacuous). would-FAIL
/// if verify rejected a fresh stanza.
#[test]
fn f_lc_6_current_generations_verify_clean() {
    let stanza = Stanza {
        recipient_key_generation: 5,
        k_principal_generation: 2,
    };
    let outcome = verify_stanza_generation(&stanza, 5, 2);
    assert!(
        outcome.is_ok(),
        "F-LC-6: a stanza current on BOTH recipient_key_generation AND \
         k_principal_generation MUST verify cleanly (positive control; \
         proves the staleness rejections are not vacuous). Got: {outcome:?}"
    );
}
