//! F-full R3-W2 (Layer-C) — HPKE encrypt-to-recipient + Sealed-Sender.
//!
//! ADDL Phase-4-Meta-Core, **F-full** R3 wave **W2-layer-c** (RED-PHASE,
//! TDD red-phase per `pim-12 §3.6e`). Families pinned in THIS file:
//!   - **F-LC-1** HPKE `mode_base` single-recipient round-trip (`0x647A`).
//!   - **F-LC-2** `HpkeMultiBase` group multi-stanza + cross-stanza AAD
//!     substitution defense (`0x6520`).
//!   - **F-LC-3** Sealed-Sender DEFAULT (`0x6510`) — sender-DID NOT on the
//!     wire (paired positive control: `0x6500` DOES carry it).
//!
//! Pin sources (canonical R0.3 design = `4fe9236a:.addl/phase-4-meta/`
//! `f-full-r0-plan.md`):
//!   - §3.3 "Layer-C — encrypt-to-recipient (HPKE + MLKEM768-X25519 +
//!     multi-stanza; Sealed-Sender DEFAULT)".
//!   - §4.0 codepoint table: `0x647A` HYBRID_X25519_MLKEM768 (real X-Wing
//!     SHA3-256, ChaCha20-Poly1305 bulk); `0x6500` LAYER_C_DROP
//!     (plaintext-sender, non-default); `0x6510`
//!     DROP_TO_RECIPIENT_SEALED_SENDER (v1-beta DEFAULT, BR-1); `0x6520`
//!     LAYER_C_DROP_MULTI_RECIPIENT (`HpkeMultiBase` group).
//!   - §4.1 envelope table: `EncryptedEnvelope` / `BindingContext`
//!     `#[non_exhaustive]`; per-stanza AAD binds
//!     `(codepoint, body-CID, sorted recipient-DID-list, sender_did,
//!     stanza-index, recipient_key_generation)` (U17); BE endianness.
//!   - Inv-16 (envelope-unification) + Inv-18 (paired Sealed-Sender
//!     disclosure satisfied by `0x6510` being DEFAULT).
//!   - R2 landscape `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md`
//!     §1 Group 7: F-LC-1 (~4-6), F-LC-2 (~6-8), F-LC-3 (~7-10).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At the F-full baseline these Layer-C production types DO NOT YET EXIST
//! (`benten-drop` ships a `DropBundle` whose per-Recipe cells use the
//! in-tree LE/V1 `AeadEnvelope`; the codepoint-dispatched
//! `EncryptedEnvelope` + `HpkeBase`/`HpkeMultiBase` + Sealed-Sender drop
//! variants are F-full Layer-C canary scope). Per the parallel-safety
//! contract of this R3 wave, this file carries a **SELF-CONTAINED stub
//! module** (`layer_c_stub`) matching the intended Layer-C public surface
//! so the file COMPILES green at baseline + every test is
//! `#[ignore = "RED-PHASE: F-LC-… — …; un-ignore at R5"]`. The stub does
//! NOT depend on any other R3 wave's module. The Layer-C closing-wave R5
//! implementer MUST:
//!   1. DELETE the local `layer_c_stub` module,
//!   2. INSERT the real `use benten_drop::layer_c::{…};` lines,
//!   3. UN-IGNORE each test (`#[ignore = "RED-PHASE…"]` → nothing),
//!   4. Verify all pins PASS green.
//! Reviewer verifies landing-status (un-ignored + green), not just
//! spec-pin presence (pim-12 §3.6e).
//!
//! # Wave-0 DAG edge (M-20) — V2 + BE + EncryptedEnvelope from commit 1.
//!
//! Every byte authored here is **V2 + big-endian + `EncryptedEnvelope`**.
//! There is NO surviving V1/LE golden vector. The stub's `ENVELOPE_FORMAT_`
//! `VERSION` is `2` and every wire integer (codepoint, stanza-index,
//! recipient_key_generation, coarse-epoch) is `to_be_bytes`.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f-ext).
//!
//! Each test drives a PRODUCTION call site (`seal_*` / `open_*` /
//! `serialize`) + asserts an OBSERVABLE consequence + is
//! would-FAIL-if-no-op'd. The stub `unimplemented!()`s its seal/open
//! bodies so a forgotten stub at R5 PANICS (the opposite of a silent-green
//! SHAPE-trap). NEVER `assert_eq!(CONST, CONST_VAL)`; NEVER a
//! zero-assertion arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_variables)]

// ===========================================================================
// SELF-CONTAINED STUB-SHIM — DELETE at R5; replace with `use benten_drop::…`.
// ===========================================================================
//
// This stub mirrors the intended Layer-C public surface. It carries ZERO
// dependency on any other R3 wave's crate/module (parallel-safety). All
// CIDs / DIDs / keys are modeled as fixed byte arrays so the file is
// hermetic. The seal/open bodies `unimplemented!()` so the runtime arms
// only pass once R5 wires the real production path (and the `#[ignore]`
// is lifted).
mod layer_c_stub {
    /// Wave-0 envelope-format version (M-18/M-19/M-20). V2 from commit 1.
    pub const ENVELOPE_FORMAT_VERSION: u8 = 2;

    // §4.0 codepoint integers — Layer-C drop / recipient band. Wire-locked.
    pub const HYBRID_X25519_MLKEM768: u16 = 0x647a;
    pub const LAYER_C_DROP: u16 = 0x6500; // plaintext-sender (NON-default)
    pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510; // v1-beta DEFAULT
    pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520; // HpkeMultiBase group

    /// A recipient identity (modeled as the X25519⊕ML-KEM-768 hybrid KEM
    /// pubkey fingerprint; the real type is a `HybridKemPubKey`).
    pub type RecipientPubKey = [u8; 32];
    /// A recipient secret (the real type wraps `secrecy::SecretBox`).
    pub type RecipientSecKey = [u8; 32];
    /// A sender DID, modeled as raw bytes (`did:key` multibase string in
    /// production).
    pub type SenderDid = Vec<u8>;
    /// A content CID (the body-CID bound in AAD).
    pub type BodyCid = [u8; 32];

    /// The typed `BindingContext` (`#[non_exhaustive]` in production). The
    /// stub enumerates only the Layer-C drop variants this file pins.
    #[non_exhaustive]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum BindingContext {
        /// Plaintext-sender drop (`0x6500`): sender-DID is bound INTO the
        /// AAD (U4) — i.e. it IS on the wire in the serialized envelope.
        DropPlaintextSender {
            codepoint: u16,
            body_cid: BodyCid,
            sender_did: SenderDid,
            recipient_key_generation: u32,
            coarse_epoch: u64,
        },
        /// Sealed-Sender drop (`0x6510`, DEFAULT): the AAD carries ONLY
        /// audience + coarse-epoch; the sender-DID lives INSIDE the
        /// ciphertext (HPKE inner-payload) and is recovered post-decrypt.
        DropSealedSender {
            codepoint: u16,
            body_cid: BodyCid,
            recipient_key_generation: u32,
            coarse_epoch: u64,
        },
    }

    /// A single recipient stanza of an `HpkeMultiBase` group envelope.
    /// Per-stanza AAD binds the full U17 tuple.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct HpkeRecipientStanza {
        pub codepoint: u16,
        pub body_cid: BodyCid,
        /// sorted recipient-DID-list (the WHOLE list, bound per stanza).
        pub sorted_recipient_dids: Vec<SenderDid>,
        pub sender_did: SenderDid,
        pub stanza_index: u32,
        pub recipient_key_generation: u32,
        /// HPKE-wrapped content-encryption-key for THIS recipient.
        pub wrapped_cek: Vec<u8>,
    }

    /// The codepoint-dispatched `EncryptedEnvelope` (Inv-16). The stub
    /// models the two shapes this file exercises.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum EncryptedEnvelope {
        /// Single-recipient HPKE `mode_base` (`0x647A` KEM). Carries the
        /// drop-variant binding (`0x6500` or `0x6510`).
        HpkeBase {
            format_version: u8,
            binding: BindingContext,
            /// HPKE encapsulated key (`enc`).
            enc: Vec<u8>,
            /// ChaCha20-Poly1305 ciphertext+tag of the body.
            ciphertext: Vec<u8>,
        },
        /// Group multi-stanza (`0x6520`).
        HpkeMultiBase {
            format_version: u8,
            cek_aead_ciphertext: Vec<u8>,
            cek_aead_nonce: [u8; 12],
            stanzas: Vec<HpkeRecipientStanza>,
        },
    }

    /// Typed Layer-C error (the real type is a `DropError`/`AeadError`
    /// family). The stub enumerates the rejection arms this file pins.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum LayerCError {
        /// AEAD authentication failed (wrong key, tampered AAD, stanza
        /// substitution/reorder/re-target).
        AeadAuthenticationFailed,
        /// The recovered inner sender-DID did not verify (forged inner DID).
        InnerSenderDidForged,
        /// Codepoint dispatch hit an unknown/reserved arm.
        UnsupportedCodepoint(u16),
    }

    /// PRODUCTION call site — single-recipient HPKE-base seal (`0x647A`)
    /// under the Sealed-Sender DEFAULT (`0x6510`): the sender-DID is bound
    /// INSIDE the ciphertext, NOT in the AAD.
    pub fn seal_sealed_sender(
        _recipient_pk: &RecipientPubKey,
        _sender_did: &SenderDid,
        _body_cid: &BodyCid,
        _recipient_key_generation: u32,
        _coarse_epoch: u64,
        _plaintext: &[u8],
    ) -> EncryptedEnvelope {
        unimplemented!("R5 wires benten_drop::layer_c::seal_sealed_sender")
    }

    /// PRODUCTION call site — single-recipient HPKE-base seal under the
    /// plaintext-sender NON-DEFAULT path (`0x6500`): sender-DID bound INTO
    /// the AAD (U4).
    pub fn seal_plaintext_sender(
        _recipient_pk: &RecipientPubKey,
        _sender_did: &SenderDid,
        _body_cid: &BodyCid,
        _recipient_key_generation: u32,
        _coarse_epoch: u64,
        _plaintext: &[u8],
    ) -> EncryptedEnvelope {
        unimplemented!("R5 wires benten_drop::layer_c::seal_plaintext_sender")
    }

    /// PRODUCTION call site — open a single-recipient envelope. On the
    /// Sealed-Sender path it returns the recovered sender-DID (verified
    /// post-decrypt). Wrong sk / tampered AAD / forged inner DID → `Err`.
    pub fn open_single(
        _recipient_sk: &RecipientSecKey,
        _env: &EncryptedEnvelope,
    ) -> Result<(Vec<u8>, SenderDid), LayerCError> {
        unimplemented!("R5 wires benten_drop::layer_c::open_single")
    }

    /// PRODUCTION call site — group multi-stanza seal (`0x6520`).
    pub fn seal_group_multi(
        _recipient_pks: &[RecipientPubKey],
        _sender_did: &SenderDid,
        _body_cid: &BodyCid,
        _recipient_key_generation: u32,
        _plaintext: &[u8],
    ) -> EncryptedEnvelope {
        unimplemented!("R5 wires benten_drop::layer_c::seal_group_multi")
    }

    /// PRODUCTION call site — group multi-stanza open (recipient at
    /// `my_index` opens via their stanza). Tampered/substituted/reordered
    /// stanza → `Err`.
    pub fn open_group_stanza(
        _recipient_sk: &RecipientSecKey,
        _my_index: usize,
        _env: &EncryptedEnvelope,
    ) -> Result<Vec<u8>, LayerCError> {
        unimplemented!("R5 wires benten_drop::layer_c::open_group_stanza")
    }

    /// PRODUCTION call site — canonical serialize to wire bytes (V2 + BE).
    /// What the relay sees on the network.
    pub fn serialize(_env: &EncryptedEnvelope) -> Vec<u8> {
        unimplemented!("R5 wires benten_drop::layer_c::serialize")
    }

    // -- hermetic test fixtures (NOT crate `_for_test` helpers) --

    pub fn fixed_pk(seed: u8) -> RecipientPubKey {
        [seed; 32]
    }
    pub fn fixed_sk(seed: u8) -> RecipientSecKey {
        [seed.wrapping_add(0x80); 32]
    }
    pub fn fixed_body_cid(seed: u8) -> BodyCid {
        [seed; 32]
    }
    pub fn did(s: &str) -> SenderDid {
        s.as_bytes().to_vec()
    }
}

use layer_c_stub::{
    BindingContext, DROP_TO_RECIPIENT_SEALED_SENDER, ENVELOPE_FORMAT_VERSION, EncryptedEnvelope,
    HYBRID_X25519_MLKEM768, HpkeRecipientStanza, LAYER_C_DROP, LAYER_C_DROP_MULTI_RECIPIENT,
    LayerCError, did, fixed_body_cid, fixed_pk, fixed_sk, open_group_stanza, open_single,
    seal_group_multi, seal_plaintext_sender, seal_sealed_sender, serialize,
};

// ===========================================================================
// F-LC-1 — HPKE mode_base single-recipient round-trip (0x647A).
// ===========================================================================

/// F-LC-1 PIN 1 — `seal`→`open` round-trips to the original plaintext
/// for the intended recipient. would-FAIL if the HPKE-base KE path does
/// not reconstruct the content-encryption key.
#[test]
#[ignore = "RED-PHASE: F-LC-1 — HPKE mode_base single-recipient round-trip (0x647A); un-ignore at R5"]
fn f_lc_1_hpke_base_single_recipient_round_trips() {
    let pk = fixed_pk(0x01);
    let sk = fixed_sk(0x01);
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid(0xC1);
    let plaintext = b"layer-c single recipient payload".to_vec();

    let env = seal_sealed_sender(&pk, &sender, &body_cid, 0, 0, &plaintext);
    let (recovered, recovered_sender) = open_single(&sk, &env)
        .expect("intended recipient MUST open the HPKE-base single-recipient envelope");

    assert_eq!(
        recovered, plaintext,
        "F-LC-1: HPKE mode_base[MLKEM768-X25519] (0x647A) MUST round-trip \
         the plaintext for the intended recipient. would-FAIL if the KE \
         path does not reconstruct the CEK."
    );
    assert_eq!(
        recovered_sender, sender,
        "F-LC-1: the Sealed-Sender DEFAULT path MUST recover the bound \
         sender-DID post-decrypt."
    );
}

/// F-LC-1 PIN 2 — a WRONG recipient secret key MUST fail to open. The
/// envelope is bound to ONE recipient pubkey. would-FAIL if open ignores
/// the KEM decapsulation result and returns plaintext regardless.
#[test]
#[ignore = "RED-PHASE: F-LC-1 — wrong-sk negative; un-ignore at R5"]
fn f_lc_1_wrong_recipient_sk_fails_to_open() {
    let pk = fixed_pk(0x02);
    let wrong_sk = fixed_sk(0x77); // NOT the matching sk for pk
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid(0xC2);

    let env = seal_sealed_sender(&pk, &sender, &body_cid, 0, 0, b"secret");
    let outcome = open_single(&wrong_sk, &env);

    assert!(
        matches!(outcome, Err(LayerCError::AeadAuthenticationFailed)),
        "F-LC-1: a non-recipient secret key MUST NOT open the envelope \
         (HPKE binds to one recipient pubkey). would-FAIL if open returns \
         plaintext regardless of decapsulation. Got: {outcome:?}"
    );
}

/// F-LC-1 PIN 3 — V2 + the 0x647A codepoint are committed in the typed
/// binding (Wave-0 M-20; Inv-16 codepoint-dispatch). would-FAIL if the
/// envelope is authored at V1 or omits the codepoint from its binding.
#[test]
#[ignore = "RED-PHASE: F-LC-1 — V2 + 0x647A codepoint committed in binding; un-ignore at R5"]
fn f_lc_1_envelope_is_v2_and_carries_hybrid_codepoint() {
    let env = seal_sealed_sender(
        &fixed_pk(0x03),
        &did("did:key:zAlice"),
        &fixed_body_cid(0xC3),
        0,
        0,
        b"payload",
    );

    let (format_version, codepoint) = match &env {
        EncryptedEnvelope::HpkeBase {
            format_version,
            binding,
            ..
        } => match binding {
            BindingContext::DropSealedSender { codepoint, .. } => (*format_version, *codepoint),
            BindingContext::DropPlaintextSender { codepoint, .. } => (*format_version, *codepoint),
        },
        EncryptedEnvelope::HpkeMultiBase { .. } => {
            panic!("F-LC-1 single-recipient seal MUST produce HpkeBase")
        }
    };

    assert_eq!(
        format_version, ENVELOPE_FORMAT_VERSION,
        "F-LC-1: the single-recipient envelope MUST be authored at \
         ENVELOPE_FORMAT_VERSION = 2 (Wave-0 M-20). would-FAIL on a \
         surviving V1 byte."
    );
    assert_eq!(
        codepoint, DROP_TO_RECIPIENT_SEALED_SENDER,
        "F-LC-1: the default single-recipient seal MUST commit the \
         Sealed-Sender drop codepoint (0x6510) in its typed binding \
         (Inv-16 codepoint-dispatch)."
    );
    // The KEM codepoint 0x647A is the load-bearing wire constant for the
    // HPKE-base KE; assert it is the wire-locked integer (anti-drift).
    assert_eq!(
        HYBRID_X25519_MLKEM768, 0x647a,
        "F-LC-1: the HYBRID_X25519_MLKEM768 KEM codepoint MUST be the \
         wire-locked integer 0x647A (real X-Wing SHA3-256 per BR-3)."
    );
}

// ===========================================================================
// F-LC-2 — HpkeMultiBase group multi-stanza + cross-stanza AAD defense.
// ===========================================================================

/// F-LC-2 PIN 1 — N recipients each open their own stanza to the same
/// plaintext (multi-recipient parity). would-FAIL if the group seal
/// produces stanzas that decrypt to different content or only one opens.
#[test]
#[ignore = "RED-PHASE: F-LC-2 — N-recipient multi-stanza parity (0x6520); un-ignore at R5"]
fn f_lc_2_multi_stanza_each_recipient_opens_same_plaintext() {
    let pks = [fixed_pk(0x10), fixed_pk(0x11), fixed_pk(0x12)];
    let sks = [fixed_sk(0x10), fixed_sk(0x11), fixed_sk(0x12)];
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid(0xD0);
    let plaintext = b"group payload".to_vec();

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, &plaintext);

    for (idx, sk) in sks.iter().enumerate() {
        let recovered = open_group_stanza(sk, idx, &env)
            .unwrap_or_else(|e| panic!("recipient {idx} MUST open their stanza: {e:?}"));
        assert_eq!(
            recovered, plaintext,
            "F-LC-2: every recipient stanza MUST open to the SAME plaintext \
             (multi-recipient parity). would-FAIL if a stanza decrypts to \
             divergent content."
        );
    }
}

/// F-LC-2 PIN 2 — cross-stanza SUBSTITUTION is rejected. Swapping two
/// recipients' stanzas (so recipient 0 gets recipient 1's stanza) MUST
/// fail at AEAD-open: the per-stanza AAD binds `stanza-index` +
/// `sorted-recipient-DID-list`, so a re-positioned stanza no longer
/// authenticates. would-FAIL if the AAD omits the stanza-index/recipient
/// binding (defense is in AAD, NOT in the CID — U17).
#[test]
#[ignore = "RED-PHASE: F-LC-2 — cross-stanza substitution rejected (U17); un-ignore at R5"]
fn f_lc_2_cross_stanza_substitution_rejected() {
    let pks = [fixed_pk(0x20), fixed_pk(0x21)];
    let sks = [fixed_sk(0x20), fixed_sk(0x21)];
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid(0xD1);

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, b"group payload");

    // Adversary swaps stanza 0 and stanza 1.
    let mut tampered = env.clone();
    if let EncryptedEnvelope::HpkeMultiBase { stanzas, .. } = &mut tampered {
        stanzas.swap(0, 1);
    } else {
        panic!("group seal MUST produce HpkeMultiBase");
    }

    // Recipient 0 now reads a stanza that was sealed for recipient 1's
    // position; the per-stanza AAD (stanza-index 1, recipient-list order)
    // no longer matches recipient 0's open context.
    let outcome = open_group_stanza(&sks[0], 0, &tampered);
    assert!(
        matches!(outcome, Err(LayerCError::AeadAuthenticationFailed)),
        "F-LC-2: cross-stanza substitution (swap 0↔1) MUST fail at AEAD \
         because the per-stanza AAD binds stanza-index + recipient-DID-list \
         (U17). would-FAIL if substitution defense rode the CID instead of \
         the AAD. Got: {outcome:?}"
    );
}

/// F-LC-2 PIN 3 — RE-TARGET to a different recipient is rejected. If an
/// adversary rewrites a stanza's `sorted_recipient_dids` (re-pointing the
/// group), the AAD reconstructed at open no longer matches the seal-time
/// AAD. would-FAIL if the recipient-DID-list is not bound per stanza.
#[test]
#[ignore = "RED-PHASE: F-LC-2 — stanza re-target rejected; un-ignore at R5"]
fn f_lc_2_stanza_retarget_to_different_recipient_rejected() {
    let pks = [fixed_pk(0x30), fixed_pk(0x31)];
    let sks = [fixed_sk(0x30)];
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid(0xD2);

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, b"group payload");

    // Adversary rewrites the bound recipient-DID-list of stanza 0 to a
    // different membership.
    let mut tampered = env.clone();
    if let EncryptedEnvelope::HpkeMultiBase { stanzas, .. } = &mut tampered {
        stanzas[0].sorted_recipient_dids = vec![did("did:key:zMallory")];
    } else {
        panic!("group seal MUST produce HpkeMultiBase");
    }

    let outcome = open_group_stanza(&sks[0], 0, &tampered);
    assert!(
        matches!(outcome, Err(LayerCError::AeadAuthenticationFailed)),
        "F-LC-2: re-targeting a stanza's recipient-DID-list MUST fail at \
         AEAD (the sorted-recipient-DID-list is bound per stanza, U17). \
         Got: {outcome:?}"
    );
}

/// F-LC-2 PIN 4 — the group codepoint is the wire-locked 0x6520 and the
/// stanza count matches the recipient count. would-FAIL if the group
/// envelope is authored at the wrong band or drops stanzas.
#[test]
#[ignore = "RED-PHASE: F-LC-2 — group codepoint 0x6520 + stanza count; un-ignore at R5"]
fn f_lc_2_group_envelope_codepoint_and_stanza_count() {
    let pks = [
        fixed_pk(0x40),
        fixed_pk(0x41),
        fixed_pk(0x42),
        fixed_pk(0x43),
    ];
    let env = seal_group_multi(&pks, &did("did:key:zAlice"), &fixed_body_cid(0xD3), 0, b"x");

    match &env {
        EncryptedEnvelope::HpkeMultiBase {
            format_version,
            stanzas,
            ..
        } => {
            assert_eq!(
                *format_version, ENVELOPE_FORMAT_VERSION,
                "F-LC-2: group envelope MUST be authored at V2."
            );
            assert_eq!(
                stanzas.len(),
                pks.len(),
                "F-LC-2: HpkeMultiBase MUST carry exactly one stanza per \
                 recipient (no dropped stanzas)."
            );
            for (i, st) in stanzas.iter().enumerate() {
                assert_eq!(
                    st.codepoint, LAYER_C_DROP_MULTI_RECIPIENT,
                    "F-LC-2: each stanza MUST carry the group codepoint \
                     0x6520 in its AAD."
                );
                assert_eq!(
                    st.stanza_index as usize, i,
                    "F-LC-2: stanza-index MUST equal the stanza position \
                     (bound in AAD for substitution defense)."
                );
            }
        }
        EncryptedEnvelope::HpkeBase { .. } => panic!("group seal MUST produce HpkeMultiBase"),
    }
    assert_eq!(
        LAYER_C_DROP_MULTI_RECIPIENT, 0x6520,
        "F-LC-2: LAYER_C_DROP_MULTI_RECIPIENT MUST be the wire-locked \
         integer 0x6520."
    );
}

// ===========================================================================
// F-LC-3 — Sealed-Sender DEFAULT (0x6510): sender-DID NOT on the wire.
// ===========================================================================
// Highest-novelty surface (R2 §1 Group 7). The metadata posture IS the
// wire contract.

/// F-LC-3 PIN 1 — on the DEFAULT (`0x6510`) path the serialized wire
/// bytes DO NOT contain the sender-DID. The sender-DID is bound INSIDE
/// the ciphertext (HPKE inner-payload); the on-wire AAD = audience +
/// coarse-epoch only. would-FAIL if the default seal leaks the sender-DID
/// into the AAD (the bug `0x6500` deliberately has, that `0x6510` fixes).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — sealed-sender default 0x6510 sender-DID NOT on wire; un-ignore at R5"]
fn f_lc_3_sealed_sender_default_omits_sender_did_from_wire() {
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let env = seal_sealed_sender(
        &fixed_pk(0x50),
        &sender,
        &fixed_body_cid(0xE0),
        0,
        1_900_800, // coarse epoch bucket
        b"sealed-sender payload",
    );

    // The default seal MUST carry the Sealed-Sender binding (no sender-DID
    // field in the AAD-bearing binding).
    match &env {
        EncryptedEnvelope::HpkeBase {
            binding: BindingContext::DropSealedSender { codepoint, .. },
            ..
        } => assert_eq!(
            *codepoint, DROP_TO_RECIPIENT_SEALED_SENDER,
            "F-LC-3: the v1-beta DEFAULT MUST be the Sealed-Sender drop \
             (0x6510)."
        ),
        _ => panic!(
            "F-LC-3: the default Layer-C seal MUST produce an HpkeBase with \
             a DropSealedSender binding (BR-1). A DropPlaintextSender here \
             would mean the default leaks the sender-DID."
        ),
    }

    // Scan the WHOLE serialized wire for the sender-DID byte sequence.
    let wire = serialize(&env);
    let needle = &sender;
    let leaks = wire.windows(needle.len()).any(|w| w == needle.as_slice());
    assert!(
        !leaks,
        "F-LC-3: the Sealed-Sender DEFAULT (0x6510) MUST NOT place the \
         sender-DID anywhere in the serialized wire bytes — it is bound \
         INSIDE the ciphertext (HPKE inner-payload), recovered only \
         post-decrypt. would-FAIL if the default path bound the sender-DID \
         into the on-wire AAD."
    );
}

/// F-LC-3 PIN 2 — PAIRED POSITIVE CONTROL: the NON-default plaintext-sender
/// sibling (`0x6500`) DOES carry the sender-DID on the wire (U4). This is
/// the control that proves PIN 1 is not vacuously passing because the
/// scanner is broken. would-FAIL if `0x6500` ALSO hid the sender-DID
/// (then the scanner can't tell the two paths apart).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — paired control 0x6500 DOES carry sender-DID (U4); un-ignore at R5"]
fn f_lc_3_plaintext_sender_sibling_carries_sender_did_on_wire() {
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let env = seal_plaintext_sender(
        &fixed_pk(0x51),
        &sender,
        &fixed_body_cid(0xE1),
        0,
        1_900_800,
        b"plaintext-sender payload",
    );

    // The non-default seal MUST carry the plaintext-sender binding with the
    // sender-DID bound INTO the AAD (U4).
    match &env {
        EncryptedEnvelope::HpkeBase {
            binding:
                BindingContext::DropPlaintextSender {
                    codepoint,
                    sender_did,
                    ..
                },
            ..
        } => {
            assert_eq!(
                *codepoint, LAYER_C_DROP,
                "F-LC-3: the plaintext-sender sibling MUST be 0x6500."
            );
            assert_eq!(
                sender_did, &sender,
                "F-LC-3: the 0x6500 binding MUST carry the sender-DID in its \
                 AAD (U4)."
            );
        }
        _ => panic!("F-LC-3: seal_plaintext_sender MUST produce a DropPlaintextSender binding"),
    }

    let wire = serialize(&env);
    let leaks = wire.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        leaks,
        "F-LC-3 PAIRED CONTROL: the NON-default plaintext-sender (0x6500) \
         MUST place the sender-DID on the wire (U4). If this control fails, \
         the PIN-1 scanner cannot distinguish hiding from a broken scan — \
         the two paths must differ observably."
    );
}

/// F-LC-3 PIN 3 — post-decrypt the recovered sender-DID equals the bound
/// sender. The Sealed-Sender property is "hidden on the wire, recovered
/// by the recipient." would-FAIL if the inner-payload sender-DID is not
/// recoverable (then Sealed-Sender breaks sender attribution entirely).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — recovered inner sender-DID equals bound; un-ignore at R5"]
fn f_lc_3_recovered_inner_sender_did_equals_bound() {
    let pk = fixed_pk(0x52);
    let sk = fixed_sk(0x52);
    let sender = did("did:key:zCarol");
    let env = seal_sealed_sender(&pk, &sender, &fixed_body_cid(0xE2), 0, 0, b"hi");

    let (_pt, recovered_sender) =
        open_single(&sk, &env).expect("recipient MUST open the sealed-sender envelope");
    assert_eq!(
        recovered_sender, sender,
        "F-LC-3: the recipient MUST recover the bound inner sender-DID \
         post-decrypt. would-FAIL if the inner-payload sender-DID is not \
         carried/recovered."
    );
}

/// F-LC-3 PIN 4 — a FORGED inner sender-DID is rejected. The inner
/// sender-DID is bound such that tampering with it fails the post-decrypt
/// verify (Inv-16 sender-DID-or-Sealed-Sender clause). would-FAIL if the
/// inner sender-DID is unauthenticated (then anyone can spoof the sender).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — forged inner sender-DID rejected; un-ignore at R5"]
fn f_lc_3_forged_inner_sender_did_rejected() {
    let pk = fixed_pk(0x53);
    let sk = fixed_sk(0x53);
    let sender = did("did:key:zCarol");
    let env = seal_sealed_sender(&pk, &sender, &fixed_body_cid(0xE3), 0, 0, b"hi");

    // Adversary tampers the ciphertext (where the inner sender-DID lives).
    let mut tampered = env.clone();
    if let EncryptedEnvelope::HpkeBase { ciphertext, .. } = &mut tampered
        && let Some(b) = ciphertext.first_mut()
    {
        *b ^= 0xFF;
    }

    let outcome = open_single(&sk, &tampered);
    assert!(
        matches!(
            outcome,
            Err(LayerCError::AeadAuthenticationFailed | LayerCError::InnerSenderDidForged)
        ),
        "F-LC-3: a forged/tampered inner sender-DID MUST be rejected at \
         open (AEAD-auth or post-decrypt verify). would-FAIL if the inner \
         sender-DID were unauthenticated. Got: {outcome:?}"
    );
}
