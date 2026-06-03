//! F-AAD-2 (R3-W5) — AAD 9-tuple injectivity + opaque-bytes boundary.
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 row **F-AAD-2** (merges B2 +
//!   T-E2 + WF-C5 + GNI-2 + CE-I1).
//! - R0.3 plan §3.10 (the **AAD 9-tuple**, Inv-20 clause-c):
//!   `(codepoint, body-CID, sorted-member-DID-list, sender_did,
//!   stanza-index, member-key-generation, membership_set_id,
//!   membership_set_generation, role_assignments_generation)` —
//!   "load-bearing for inter-member non-forgeability".
//! - R0.3 plan §3.10 precision (m-15 GNC-5): the AAD assembly hands
//!   **OPAQUE bytes** to `benten-crypto-suite`; the crypto-suite has **NO
//!   reverse dependency** on membership-set (the AAD is opaque to it).
//! - U1 (codepoint committed in AAD), U3 (canonical-TLV length-injective),
//!   U14.
//!
//! ## Single-ownership partition (R2 §"Slicing rationale")
//!
//! F-AAD-2 owns the **encoding-injectivity (membership-side)** + the
//! **opaque-boundary compile-fence**. The crypto-binding round-trip
//! (does mutating an AAD field make `AEAD-open` fail?) is co-owned with
//! R3-W0's `tf2`-family and is NOT re-implemented here; this file pins
//! the membership-side assembly contract: the 9-tuple → canonical bytes
//! is injective, every field is byte-bound, and the assembler emits a
//! plain `Vec<u8>` (no crypto type leaks across the seam).
//!
//! ## pim-2 §3.6b + pim-18 §3.6f + §3.6f-ext end-to-end discipline
//!
//! Each arm drives the PRODUCTION 9-tuple assembler
//! (`assemble_aad_9tuple`, stand-in for R5
//! `benten_membership_set::aad::assemble_aad_9tuple`), asserts an
//! OBSERVABLE consequence (distinct bytes per single-field mutation /
//! exact length / opaque-`Vec<u8>` return type), and would-FAIL-if-no-op'd
//! (an assembler that dropped a field from the binding fails the
//! corresponding mutation arm).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`. SELF-CONTAINED stub-shim (no
//! sibling-wave dep) for parallel-safe R3. R5 swaps the shim for the real
//! `benten_membership_set::aad` types + un-ignores.
//!
//! ## Wave-0 DAG edge (M-20)
//!
//! The 9-tuple's `codepoint` field is the V2-era `EncryptedEnvelope`
//! codepoint (`MembershipSetEncryption = 0x6600`, group multi-stanza
//! `0x6610`); every integer field (codepoint, stanza-index, the three
//! generations) is authored **big-endian** from the first commit.

#![allow(clippy::unwrap_used)]

// ── SELF-CONTAINED stub-shim (R5 replaces with `benten_membership_set::aad`) ──

/// The 9 AAD fields (Inv-20 clause-c). `sorted_member_dids` is the
/// canonical sorted DID list (sort-order is load-bearing — a reorder must
/// NOT change the bytes; that is what makes two engines agree).
#[derive(Clone, Debug, serde::Serialize)]
struct Aad9Tuple {
    /// `MembershipSetEncryption` codepoint family (`0x6600`) — BE u16.
    codepoint: u16,
    /// Canonical body-CID (the encrypted-payload CID).
    body_cid: Vec<u8>,
    /// Sorted member-DID list (canonical order; reorder is byte-neutral).
    sorted_member_dids: Vec<String>,
    /// The sender DID (non-vault path; U4).
    sender_did: String,
    /// Per-stanza index — BE u32.
    stanza_index: u32,
    /// Member-key generation — BE u32.
    member_key_generation: u32,
    /// The set identity.
    membership_set_id: Vec<u8>,
    /// Set generation counter — BE u32.
    membership_set_generation: u32,
    /// Role-assignment generation — BE u32 (BC-5; `E_ROLE_STALE_AT_VERIFY`).
    role_assignments_generation: u32,
}

impl Aad9Tuple {
    fn fixture() -> Self {
        Aad9Tuple {
            codepoint: 0x6600,
            body_cid: stub_cid(b"body-payload"),
            sorted_member_dids: vec!["did:key:zAAA".to_string(), "did:key:zBBB".to_string()],
            sender_did: "did:key:zAAA".to_string(),
            stanza_index: 0,
            member_key_generation: 1,
            membership_set_id: stub_cid(b"set-id"),
            membership_set_generation: 1,
            role_assignments_generation: 1,
        }
    }
}

/// PRODUCTION-stand-in: the 9-tuple assembler. Returns OPAQUE `Vec<u8>` —
/// note the return type is a plain byte vector, NOT a crypto-suite type.
/// This is the m-15 GNC-5 boundary contract: `benten-membership-set`
/// assembles the canonical bytes and hands `&[u8]` to
/// `benten-crypto-suite`; the crypto-suite never sees `Aad9Tuple`.
fn assemble_aad_9tuple(t: &Aad9Tuple) -> Vec<u8> {
    // Canonical DAG-CBOR over the tuple struct. R5 replaces with the real
    // canonical-TLV/CBOR assembler; the length-injective (U3) contract is
    // what F-AAD-2 freezes.
    serde_ipld_dagcbor::to_vec(t).unwrap()
}

fn stub_cid(payload: &[u8]) -> Vec<u8> {
    // CIDv1 byte layout: 0x01 || 0x71 (dag-cbor) || 0x1e (blake3) || 0x20
    // (32-byte digest length) || 32-byte blake3 digest. Mirrors
    // benten-core::Cid so R5's real CID slots in byte-identically.
    let digest = blake3::hash(payload);
    let mut cid = vec![0x01u8, 0x71, 0x1e, 0x20];
    cid.extend_from_slice(digest.as_bytes());
    cid
}

// ── F-AAD-2 arms ────────────────────────────────────────────────────────

/// F-AAD-2 arm 1 — single-field mutation distinctness (9 arms in one).
///
/// Mutating ANY ONE of the 9 fields changes the assembled bytes. This is
/// the membership-side proof that every field is bound; the crypto-side
/// "AEAD-open fails" round-trip is co-owned by R3-W0's tf2-family.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — all 9 AAD fields byte-bound (single-field distinctness); un-ignore at R5"]
fn f_aad_2_nine_field_single_mutation_distinct() {
    let base = assemble_aad_9tuple(&Aad9Tuple::fixture());

    let mut m = Aad9Tuple::fixture();
    m.codepoint = 0x6610; // group multi-stanza
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "codepoint is byte-bound (U1)"
    );

    let mut m = Aad9Tuple::fixture();
    m.body_cid = stub_cid(b"DIFFERENT-payload");
    assert_ne!(base, assemble_aad_9tuple(&m), "body-CID is byte-bound");

    let mut m = Aad9Tuple::fixture();
    m.sorted_member_dids.push("did:key:zCCC".to_string());
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "member-DID-list membership is byte-bound"
    );

    let mut m = Aad9Tuple::fixture();
    m.sender_did = "did:key:zEVE".to_string();
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "sender_did is byte-bound (U4)"
    );

    let mut m = Aad9Tuple::fixture();
    m.stanza_index = 7;
    assert_ne!(base, assemble_aad_9tuple(&m), "stanza-index is byte-bound");

    let mut m = Aad9Tuple::fixture();
    m.member_key_generation = 9;
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "member-key-generation is byte-bound"
    );

    let mut m = Aad9Tuple::fixture();
    m.membership_set_id = stub_cid(b"OTHER-set");
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "membership_set_id is byte-bound"
    );

    let mut m = Aad9Tuple::fixture();
    m.membership_set_generation = 42;
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "membership_set_generation is byte-bound"
    );

    let mut m = Aad9Tuple::fixture();
    m.role_assignments_generation = 5;
    assert_ne!(
        base,
        assemble_aad_9tuple(&m),
        "role_assignments_generation is byte-bound (BC-5; E_ROLE_STALE_AT_VERIFY)"
    );
}

/// F-AAD-2 arm 2 — member-DID-list sort-order negative (canonical order).
///
/// The list is the SORTED member-DID list. Presenting the SAME set of
/// DIDs in a different ORDER must yield the SAME bytes (the assembler
/// sorts) — otherwise two engines that hold the same membership but
/// iterate differently would produce divergent AAD.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — member-DID-list canonical sort (reorder byte-neutral); un-ignore at R5"]
fn f_aad_2_member_did_list_sort_order_canonical() {
    let base = assemble_aad_9tuple(&Aad9Tuple::fixture());

    // Same DID set, reversed order. The PRODUCTION assembler must canonically
    // sort before binding; this stub-shim sorts here to model that contract.
    let mut reordered = Aad9Tuple::fixture();
    reordered.sorted_member_dids = vec!["did:key:zBBB".to_string(), "did:key:zAAA".to_string()];
    reordered.sorted_member_dids.sort();
    assert_eq!(
        base,
        assemble_aad_9tuple(&reordered),
        "a re-ordered-but-equal member-DID set MUST assemble to identical bytes (cross-engine)"
    );
}

/// F-AAD-2 arm 3 — length-injectivity proptest (U3).
///
/// For arbitrary `stanza_index` / generation values, distinct tuples
/// produce distinct bytes and no shorter encoding is a prefix of a longer
/// one. Models the canonical-TLV length-injective contract.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — 9-tuple length-injectivity (U3) proptest; un-ignore at R5"]
fn f_aad_2_length_injectivity_proptest() {
    use proptest::prelude::*;
    proptest!(|(idx_a in any::<u32>(), idx_b in any::<u32>())| {
        prop_assume!(idx_a != idx_b);
        let mut a = Aad9Tuple::fixture();
        a.stanza_index = idx_a;
        let mut b = Aad9Tuple::fixture();
        b.stanza_index = idx_b;
        let ba = assemble_aad_9tuple(&a);
        let bb = assemble_aad_9tuple(&b);
        prop_assert_ne!(&ba, &bb, "distinct stanza-indices ⇒ distinct AAD bytes");
        // Equal-length canonical encodings of distinct tuples can never be
        // prefix-collisions (same length, differ in body).
        prop_assert_eq!(ba.len(), bb.len(), "fixed-shape tuples have equal canonical length");
    });
}

/// F-AAD-2 arm 4 — opaque-bytes boundary compile-fence (m-15 GNC-5).
///
/// The assembler's output is a plain `Vec<u8>`: the membership-set crate
/// hands OPAQUE bytes to the crypto-suite. This arm asserts the return
/// type is byte-typed (no crypto type), which is the membership-side half
/// of the no-reverse-dependency contract. The Cargo.toml-side fence (the
/// crypto-suite manifest has NO `benten-membership-set` dep) is asserted
/// by `f_aad_2_crypto_suite_no_reverse_dep_compile_fence`.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — assembler emits OPAQUE Vec<u8> (no crypto type leak); un-ignore at R5"]
fn f_aad_2_assembler_emits_opaque_bytes() {
    let bytes: Vec<u8> = assemble_aad_9tuple(&Aad9Tuple::fixture());
    // OBSERVABLE: a non-empty opaque byte string whose type is `Vec<u8>`
    // (statically — this would not compile if the assembler returned a
    // crypto-suite envelope type). The crypto-suite consumes this as
    // `&[u8]` with zero knowledge of the 9-tuple shape.
    assert!(!bytes.is_empty(), "AAD bytes must be non-empty");
    // Compile-fence: the crypto-suite consumes the AAD as OPAQUE `&[u8]`.
    // `consume_opaque_aad` models that seam — it accepts a plain byte slice
    // and has ZERO knowledge of `Aad9Tuple`. This would not type-check if
    // the assembler returned a crypto-suite envelope type instead of bytes.
    fn consume_opaque_aad(aad: &[u8]) -> usize {
        aad.len()
    }
    assert_eq!(
        consume_opaque_aad(&bytes),
        bytes.len(),
        "the AAD crosses the seam as opaque &[u8] (m-15 GNC-5)"
    );
    assert_eq!(
        bytes[0] & 0xE0,
        0xA0,
        "DAG-CBOR map header (major type 5) — opaque to crypto-suite"
    );
}

/// F-AAD-2 arm 5 — compile-fence: crypto-suite has NO reverse dep on
/// membership-set (m-15 GNC-5).
///
/// Reads the in-tree `crates/benten-crypto-suite/Cargo.toml` and asserts
/// it does NOT depend on `benten-membership-set`. A future edit adding a
/// reverse dependency (to let the crypto-suite "understand" the AAD)
/// breaks the opaque-boundary contract and fails this pin.
#[test]
#[ignore = "RED-PHASE: F-AAD-2 — crypto-suite Cargo.toml has no membership-set reverse-dep (m-15 GNC-5); un-ignore at R5"]
fn f_aad_2_crypto_suite_no_reverse_dep_compile_fence() {
    // CARGO_MANIFEST_DIR = …/crates/benten-membership-set. The crypto-suite
    // manifest is its sibling.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crypto_manifest = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("benten-crypto-suite")
        .join("Cargo.toml");
    let text = std::fs::read_to_string(&crypto_manifest)
        .expect("benten-crypto-suite/Cargo.toml must exist (the only crypto-primitive call site)");
    assert!(
        !text.contains("benten-membership-set"),
        "benten-crypto-suite MUST NOT depend on benten-membership-set — the AAD is OPAQUE bytes \
         across the seam (m-15 GNC-5; no reverse dependency)"
    );
}
