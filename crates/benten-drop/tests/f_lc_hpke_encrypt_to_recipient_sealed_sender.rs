//! F-full R3-W2 (Layer-C) — HPKE encrypt-to-recipient + Sealed-Sender.
//!
//! ADDL Phase-4-Meta-Core, **F-full** R3 wave **W2-layer-c** (RED-PHASE,
//! TDD red-phase per `pim-12 §3.6e`). Families pinned in THIS file:
//!   - **F-LC-1** HPKE `mode_base` single-recipient round-trip (`0x647A`).
//!   - **F-LC-2** `HpkeMultiBase` group multi-stanza + cross-stanza AAD
//!     substitution defense (`0x6520`) — **honoring Sealed-Sender by
//!     DEFAULT** (R4-FIX F4-003 / BR-1 ruling 1).
//!   - **F-LC-3** Sealed-Sender DEFAULT (`0x6510`) — sender-DID NOT on the
//!     wire (paired positive control: `0x6500` DOES carry it).
//!
//! Pin sources (canonical R0.5 design = `e4fbfe73:.addl/phase-4-meta/`
//! `f-full-r0-plan.md`):
//!   - §3.3 "Layer-C — encrypt-to-recipient (HPKE + MLKEM768-X25519 +
//!     multi-stanza; Sealed-Sender DEFAULT)". §3.3:484 — the on-wire AAD
//!     "carries only audience + coarse epoch"; post Ben-RULING-#1 / M-14 the
//!     coarse-epoch is OFF the drop wire, so the single-recipient drop AAD
//!     binds the **audience** (the spec-mandated recipient-targeting field).
//!   - §4.0 codepoint table: `0x647A` HYBRID_X25519_MLKEM768 (real X-Wing
//!     SHA3-256, ChaCha20-Poly1305 bulk); `0x6500` LAYER_C_DROP
//!     (plaintext-sender, non-default); `0x6510`
//!     DROP_TO_RECIPIENT_SEALED_SENDER (v1-beta DEFAULT, BR-1); `0x6520`
//!     LAYER_C_DROP_MULTI_RECIPIENT (`HpkeMultiBase` group).
//!   - §4.1 envelope table: `EncryptedEnvelope` / `BindingContext`
//!     `#[non_exhaustive]`; per-stanza AAD binds
//!     `(codepoint, body-CID, sorted recipient-DID-list, stanza-index,
//!     recipient_key_generation)` (U17); BE endianness; a dedicated
//!     `aad_version: u8` prefix DISTINCT from `ENVELOPE_FORMAT_VERSION_V2`
//!     (U1/U3/U14) — see the AAD_VERSION note below (R4-FIX F4-004/005).
//!   - Inv-16 (envelope-unification: the `Recipient` `BindingContext` carries
//!     `{audience_did, recipient_key_generation}`) + Inv-18 (paired
//!     Sealed-Sender disclosure satisfied by `0x6510` being DEFAULT).
//!   - R2 landscape `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md`
//!     §1 Group 7: F-LC-1 (~4-6), F-LC-2 (~6-8), F-LC-3 (~7-10).
//!
//! # R4-FIX (F4-003 BLOCKER) — `0x6520` group send HONORS Sealed-Sender.
//!
//! Ben RULING 1 (2026-06-02, BR-1): **EVERY group send honors
//! Sealed-Sender** — the inner-sender-DID is bound INSIDE the sealed
//! (encrypted) part PER STANZA, NOT in the plaintext AAD. The original
//! F-LC-2 here pinned a `0x6520` group send whose per-stanza struct carried
//! a `sender_did` field bound into the PLAINTEXT AAD (mirroring the
//! non-default `0x6500` plaintext-sender variant) — which CONTRADICTS the
//! ruling and silently defeats Sealed-Sender for every group send. The fix:
//!   - the DEFAULT group seal (`seal_group_multi`) now produces stanzas
//!     whose plaintext AAD binds ONLY
//!     `(codepoint, body-CID, sorted recipient-DID-list, stanza-index,
//!     recipient_key_generation)` — the inner-sender-DID lives INSIDE the
//!     sealed per-stanza payload (`sealed_inner`), recovered post-decrypt
//!     (mirrors how `0x6510`/`0x6610` do it);
//!   - a NEW `0x6520` SEALED-GROUP wire-scan arm asserts the sender-DID
//!     does NOT appear in the plaintext AAD bytes (would-FAIL if it leaks);
//!   - the non-default plaintext-sender group variant is kept as an
//!     EXPLICITLY-LABELED non-default arm (`seal_group_multi_plaintext_`
//!     `sender`) with its own paired-control scan, so the substitution /
//!     re-target defenses still exercise a real per-stanza AAD field-set.
//!
//! # R4.3-FIX (F4-004/005 MAJOR) — dedicated `aad_version: u8` byte-0.
//!
//! Spec R0.5 §4.1 freezes a dedicated `aad_version: u8` AAD prefix that is
//! DISTINCT from `ENVELOPE_FORMAT_VERSION_V2` (the envelope serialization-
//! format byte). The MembershipSet sibling (`f_aad_2`) correctly uses
//! `const AAD_VERSION: u8 = 0x01` as AAD byte-0. The original Layer-C AAD
//! helper here pushed `ENVELOPE_FORMAT_VERSION` (= 2) as byte-0 — conflating
//! two independent version axes and freezing a conflicting leading byte
//! (`0x02`) against membership's `0x01`, which would break AEAD-open across
//! the Layer-C / membership engines for the identical §4.1 prefix. The fix:
//! introduce `const AAD_VERSION: u8 = 0x01` and push THAT as AAD byte-0;
//! `ENVELOPE_FORMAT_VERSION = 2` stays strictly for the envelope-format
//! version field (`format_version`), never the AAD prefix.
//!
//! # R4.3-FIX (F4-006 MAJOR) — DropToRecipient carries NEITHER coarse-epoch.
//!
//! Spec R0.5 is internally DECIDABLE on the drop-wire freshness posture, so
//! this is resolved in-file (do-it-now) rather than left as a wire-byte fork:
//!   - **M-14** (R0.5 §3.10/§4.1, ratified): "DropToRecipient carries
//!     **NEITHER**" sealed_at NOR valid_until; "freshness = recipient-key-
//!     generation + nonce-cache"; the "1-hr bucket" (U28) is "**Layer-D
//!     ONLY**" (DeviceLink + RemotePermission).
//!   - **§4.1 FREEZE row**: "`sealed_at` + `valid_until` epoch (DeviceLink +
//!     RemotePermission **ONLY** — M-14) … coarse 1-hour bucket (U28);
//!     **DropToRecipient carries NEITHER**".
//!   - **§3.11 #43** ("audience + coarse-epoch on the wire") is the GENERAL
//!     residual-metadata mitigation-roadmap framing (U22–U28), NOT a mandate
//!     that the Layer-C drop wire carries a coarse-epoch field. Drops are
//!     "forever-valid (per #62; freshness rides recipient-key-generation +
//!     the nonce-cache, NOT a timestamp)". The single-recipient drop AAD
//!     therefore binds the **audience** (recipient-targeting) but NEITHER
//!     timestamp NOR coarse bucket.
//! The original Layer-C `BindingContext` + seal fns carried a `coarse_epoch:
//! u64` (4 mentions), contradicting M-14 and disagreeing with the Layer-D
//! drop sibling `f_ld_8` (0 mentions). The fix: REMOVE `coarse_epoch` from
//! both Layer-C drop bindings + the two single-recipient seal signatures, so
//! this stub models the FROZEN drop wire (NEITHER timestamp NOR coarse
//! bucket). The coarse 1-hour bucket survives ONLY on the Layer-D
//! (DeviceLink/RemotePermission) `sealed_at`/`valid_until` surface.
//! [FLAG-FOR-BEN — courtesy cross-check, not a halt: resolved here per the
//! M-14 + §4.1 FREEZE rows; if §3.11 #43 is later read as mandating a
//! coarse-epoch on the Layer-C drop wire, re-add it symmetrically to
//! `f_ld_8` + the §4.1 FREEZE table. Both stubs now agree on NEITHER.]
//!
//! # R4.4-FIX (CLUSTER-1 MAJOR) — single-recipient AAD = sibling union; byte-0 guard.
//!
//! The R4.3 CLUSTER-1 reconciliation aligned the `f_lc_abuse` /
//! `f_lc_hpke` siblings on the byte-0 / `coarse_epoch` axes for the GROUP
//! (`0x6520`) stanza — but left the `f_lc_hpke` **single-recipient**
//! `DropSealedSender` binding stale on the **audience** axis. The sibling
//! `f_lc_abuse::SealedSenderAad` (and the Inv-16 `Recipient` `BindingContext`
//! in `f_inv16_1`) bind the canonical `0x6510` envelope-AAD union
//! `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
//! and freeze a golden (`F_INV18_1_SEALED_AAD_HEX`); the `f_lc_hpke`
//! `DropSealedSender` here bound only `{codepoint, body_cid,
//! recipient_key_generation}` — NO `aad_version`, NO `audience` — and froze
//! no single-recipient golden (its `serialize`/`plaintext_aad_bytes` were
//! unimplemented for `HpkeBase`). That is the audience-axis divergence
//! (F-NEW-SS-AUD): an R5 impl that freezes this file's field-set would drop
//! the spec-mandated recipient-audience binding (§3.3:484), while one that
//! freezes the sibling golden would break cross-engine AEAD-open against the
//! audience-less form. The CLUSTER-1 audience-axis fix:
//!   - `DropSealedSender` GAINS `aad_version` + `audience_did` so its bound
//!     field-set is the canonical union, BYTE-IDENTICAL to
//!     `f_lc_abuse::SealedSenderAad` (and the Inv-16 `Recipient` binding);
//!   - a DETERMINISTIC `BindingContext::DropSealedSender::plaintext_aad_`
//!     `bytes()` serializer reproduces the SAME frozen golden
//!     (`F_LC_SEALED_SENDER_AAD_HEX` == the sibling's
//!     `F_INV18_1_SEALED_AAD_HEX`) so the two files CANNOT silently
//!     re-diverge on the `0x6510` single-recipient envelope AAD;
//!   - a NEW arm asserts byte-0 == `AAD_VERSION` (≠ `ENVELOPE_FORMAT_VERSION`),
//!     the BE codepoint, the audience binding present, the sender-DID absent,
//!     and the FROZEN golden (mirrors the group byte-0 guard for `0x6520`).
//! Both sibling files now carry the SAME one-canonical-field-set + byte-0
//! anti-conflation guard for BOTH the `0x6510` single-recipient AND the
//! `0x6520` group AAD layouts across the `0x65xx` drop/recipient band.
//!
//! # R4.3-FIX (F4-018 MINOR) — per-object length-prefix WIDTH note (§4.1).
//!
//! The §4.1 canonical-TLV contract is "length-injective" (U3) — satisfied by
//! ANY injective length-prefix width; it does NOT mandate one global width.
//! The wire AADs are SEPARATELY-frozen, codepoint-DISCRIMINATED byte-strings
//! (Layer-C `0x6500`/`0x6510`/`0x6520`; MembershipSet `0x6600`; Layer-D
//! `0x6310`/wraps), NOT one shared TLV encoder — so distinct-per-object
//! widths cannot silently "break the other golden." The per-object widths
//! are therefore acceptable BUT must be written down (not implicit). The
//! frozen per-object widths:
//!   - **Layer-C single-recipient `0x6510` AAD** (THIS file): `audience_len:
//!     u16 BE` (matches the sibling `f_lc_abuse::serialize_sealed_sender_aad`
//!     EXACTLY — the shared golden depends on it).
//!   - **Layer-C group stanza AAD** (THIS file): `recipient_count: u16 BE`
//!     and each per-recipient-DID `len: u16 BE` (a Layer-C drop band caps
//!     recipient-DID lists well under 2^16; the band stays u16 for compact
//!     stanzas).
//!   - **MembershipSet 9-tuple AAD** (`f_aad_2`): per-field `len: u32 BE`.
//!   - **Layer-D ExecuteWorkflow AAD** (`f_ld_3`): `executor_did len: u32
//!     BE`.
//! This canonicalization note's source-of-record destination is **R0.5 §4.1**
//! (belongs-named-now): §4.1 GAINS an explicit per-object width sub-row at
//! the doc-wave (Layer-C drop band u16; membership/Layer-D u32). If a shared
//! Layer-C TLV helper is later introduced, converge it to this u16 contract;
//! it MUST NOT silently widen to u32 and re-freeze the `0x6510`/`0x6520`
//! goldens.
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
//! recipient_key_generation) is `to_be_bytes`. The AAD prefix byte is the
//! dedicated `AAD_VERSION` (`0x01`), DISTINCT from the format version
//! (R4.3-FIX F4-004/005).
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f).
//!
//! Each test drives a PRODUCTION call site (`seal_*` / `open_*` /
//! `serialize` / `plaintext_aad_bytes`) + asserts an OBSERVABLE
//! consequence + is would-FAIL-if-no-op'd. Seal/open bodies
//! `unimplemented!()` so a forgotten stub at R5 PANICS (the opposite of a
//! silent-green SHAPE-trap); the AAD-serialization helpers are
//! DETERMINISTIC so the Sealed-Sender wire-scan + frozen-golden pins are
//! computable green at red-phase. NEVER `assert_eq!(CONST, CONST_VAL)`;
//! NEVER a zero-assertion arm.

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
// is lifted). The AAD-serialization helpers ARE implemented
// deterministically so the Sealed-Sender wire-scan (F4-003) + the frozen
// single-recipient/group AAD goldens (CLUSTER-1) are meaningful at red-phase.
mod layer_c_stub {
    /// Wave-0 envelope-format version (M-18/M-19/M-20). V2 from commit 1.
    /// This is the envelope SERIALIZATION-format byte (the `format_version`
    /// field). It is DISTINCT from the AAD prefix byte (`AAD_VERSION`) —
    /// R4.3-FIX F4-004/005.
    pub const ENVELOPE_FORMAT_VERSION: u8 = 2;

    /// The frozen AAD version prefix byte (R0.5 §4.1: dedicated
    /// `aad_version: u8` prefix, DISTINCT from `ENVELOPE_FORMAT_VERSION_V2`;
    /// U1/U3/U14). Mirrors the MembershipSet sibling `f_aad_2` AND the
    /// Layer-C sibling `f_lc_abuse`'s `AAD_VERSION = 0x01` so the engines
    /// freeze the SAME leading AAD byte for the identical §4.1 prefix
    /// (R4.3-FIX F4-004/005 + R4.4-FIX CLUSTER-1).
    pub const AAD_VERSION: u8 = 0x01;

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
    /// An audience DID (the recipient-targeting identity bound in the
    /// single-recipient drop AAD), modeled as raw bytes.
    pub type AudienceDid = Vec<u8>;
    /// A content CID (the body-CID bound in AAD).
    pub type BodyCid = [u8; 32];

    /// The typed `BindingContext` (`#[non_exhaustive]` in production). The
    /// stub enumerates only the Layer-C drop variants this file pins.
    ///
    /// R4.4-FIX CLUSTER-1: BOTH single-recipient drop variants bind the
    /// canonical `0x65xx` envelope-AAD prefix
    /// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
    /// — BYTE-IDENTICAL to the sibling `f_lc_abuse::SealedSenderAad` and the
    /// Inv-16 `Recipient` `BindingContext`. The `audience_did` is the
    /// recipient-targeting binding the spec mandates (§3.3:484). The
    /// plaintext-sender variant ADDS the sender-DID (U4); the Sealed-Sender
    /// variant does NOT (it lives inside the ciphertext).
    ///
    /// R4.3-FIX F4-006: NEITHER drop variant carries a `coarse_epoch` (nor
    /// `sealed_at`/`valid_until`). Per M-14 + §4.1 FREEZE, DropToRecipient
    /// carries NEITHER timestamp NOR coarse bucket — drops are forever-valid
    /// (per #62; freshness rides recipient-key-generation + the nonce-cache).
    /// The coarse 1-hour bucket (U28) lives ONLY on the Layer-D
    /// (DeviceLink/RemotePermission) `sealed_at`/`valid_until` surface.
    #[non_exhaustive]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum BindingContext {
        /// Plaintext-sender drop (`0x6500`, NON-default): binds the canonical
        /// audience prefix PLUS the sender-DID (U4) — i.e. the sender-DID IS
        /// on the wire in the serialized envelope. Carries NEITHER timestamp
        /// NOR coarse-epoch (M-14; F4-006).
        DropPlaintextSender {
            aad_version: u8,
            codepoint: u16,
            audience_did: AudienceDid,
            body_cid: BodyCid,
            recipient_key_generation: u32,
            sender_did: SenderDid,
        },
        /// Sealed-Sender drop (`0x6510`, DEFAULT): the AAD binds the canonical
        /// `0x65xx` envelope union
        /// `{aad_version, codepoint, audience, body_cid,
        /// recipient_key_generation}` — the `audience_did` is the
        /// recipient-targeting binding (§3.3:484); the sender-DID lives INSIDE
        /// the ciphertext (HPKE inner-payload) and is recovered post-decrypt.
        /// Carries NEITHER timestamp NOR coarse-epoch (M-14; F4-006). This is
        /// BYTE-IDENTICAL to the sibling `f_lc_abuse::SealedSenderAad`
        /// (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD).
        DropSealedSender {
            aad_version: u8,
            codepoint: u16,
            audience_did: AudienceDid,
            body_cid: BodyCid,
            recipient_key_generation: u32,
        },
    }

    impl BindingContext {
        /// PRODUCTION helper — the canonical PLAINTEXT single-recipient drop
        /// AAD bytes (what a relay reads in the clear). DETERMINISTIC +
        /// big-endian (M-19). BYTE-IDENTICAL to the sibling
        /// `f_lc_abuse::serialize_sealed_sender_aad` for the Sealed-Sender
        /// (`0x6510`) variant so the two files freeze the SAME golden and
        /// cannot silently re-diverge (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD).
        ///
        /// Layout (BE) — the canonical `0x65xx` envelope union; R4.3-FIX
        /// F4-004/005 (`aad_version` byte-0, distinct from `format_version`)
        /// + F4-018 (`audience_len: u16 BE`, matching the sibling EXACTLY):
        ///   aad_version       : u8  (= AAD_VERSION = 0x01; NOT format ver)
        ///   codepoint         : u16 BE
        ///   audience_len      : u16 BE
        ///   audience_did      : audience_len bytes
        ///   body_cid          : 32 bytes
        ///   recipient_key_gen : u32 BE
        ///   [plaintext-sender ONLY] sender_len u16 BE || sender_did bytes
        #[must_use]
        pub fn plaintext_aad_bytes(&self) -> Vec<u8> {
            let mut out = Vec::new();
            match self {
                BindingContext::DropSealedSender {
                    aad_version,
                    codepoint,
                    audience_did,
                    body_cid,
                    recipient_key_generation,
                } => {
                    // R4.3-FIX F4-004/005: dedicated AAD version byte (0x01),
                    // NOT the envelope format version (2). Reconciles to the
                    // sibling/membership AAD golden's leading byte.
                    out.push(*aad_version);
                    out.extend_from_slice(&codepoint.to_be_bytes());
                    push_audience(&mut out, audience_did);
                    out.extend_from_slice(body_cid);
                    out.extend_from_slice(&recipient_key_generation.to_be_bytes());
                }
                BindingContext::DropPlaintextSender {
                    aad_version,
                    codepoint,
                    audience_did,
                    body_cid,
                    recipient_key_generation,
                    sender_did,
                } => {
                    out.push(*aad_version);
                    out.extend_from_slice(&codepoint.to_be_bytes());
                    push_audience(&mut out, audience_did);
                    out.extend_from_slice(body_cid);
                    out.extend_from_slice(&recipient_key_generation.to_be_bytes());
                    // Non-default plaintext-sender variant ONLY (U4): the
                    // sender-DID is appended into the PLAINTEXT AAD.
                    let len = u16::try_from(sender_did.len()).expect("sender DID len fits u16");
                    out.extend_from_slice(&len.to_be_bytes());
                    out.extend_from_slice(sender_did);
                }
            }
            out
        }
    }

    /// R4.3-FIX F4-018: Layer-C drop band uses a `u16 BE` audience length
    /// prefix (matches the sibling `f_lc_abuse::serialize_sealed_sender_aad`).
    fn push_audience(out: &mut Vec<u8>, audience_did: &[u8]) {
        let aud_len = u16::try_from(audience_did.len()).expect("audience DID len fits u16");
        out.extend_from_slice(&aud_len.to_be_bytes());
        out.extend_from_slice(audience_did);
    }

    /// A single recipient stanza of an `HpkeMultiBase` group envelope.
    ///
    /// R4-FIX F4-003 / BR-1 ruling 1: the DEFAULT group send HONORS
    /// Sealed-Sender. The PLAINTEXT per-stanza AAD binds ONLY the U17
    /// tuple WITHOUT the sender-DID; the inner-sender-DID lives INSIDE the
    /// sealed per-stanza payload (`sealed_inner`). The NON-default
    /// plaintext-sender variant sets `plaintext_sender_did = Some(..)` and
    /// binds it into the AAD (paired control only).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct HpkeRecipientStanza {
        pub codepoint: u16,
        pub body_cid: BodyCid,
        /// sorted recipient-DID-list (the WHOLE list, bound per stanza).
        pub sorted_recipient_dids: Vec<SenderDid>,
        pub stanza_index: u32,
        pub recipient_key_generation: u32,
        /// The DEFAULT (Sealed-Sender) path: the inner-sender-DID is sealed
        /// INSIDE this opaque payload alongside the wrapped CEK, recovered
        /// only post-decrypt. NEVER appears in the plaintext AAD.
        pub sealed_inner: Vec<u8>,
        /// NON-default plaintext-sender variant ONLY: when `Some`, the
        /// sender-DID is bound into the PLAINTEXT AAD (U4). `None` on the
        /// DEFAULT Sealed-Sender path.
        pub plaintext_sender_did: Option<SenderDid>,
        /// HPKE-wrapped content-encryption-key for THIS recipient.
        pub wrapped_cek: Vec<u8>,
    }

    impl HpkeRecipientStanza {
        /// PRODUCTION helper — the canonical PLAINTEXT per-stanza AAD bytes
        /// (what a relay reads in the clear). DETERMINISTIC + big-endian
        /// (M-19). On the DEFAULT Sealed-Sender path this binds the U17
        /// tuple WITHOUT the sender-DID; on the non-default plaintext-sender
        /// path the sender-DID is appended (U4).
        ///
        /// Layout (BE) — R4.3-FIX F4-004/005 (`aad_version` byte-0, distinct
        /// from `format_version`) + F4-018 (per-object u16 length widths;
        /// Layer-C drop band):
        ///   aad_version       : u8  (= AAD_VERSION = 0x01; NOT format ver)
        ///   codepoint         : u16 BE
        ///   body_cid          : 32 bytes
        ///   stanza_index      : u32 BE
        ///   recipient_key_gen : u32 BE
        ///   recipient_count   : u16 BE
        ///   for each sorted recipient DID: len u16 BE || bytes
        ///   [non-default only] sender_len u16 BE || sender_did bytes
        #[must_use]
        pub fn plaintext_aad_bytes(&self) -> Vec<u8> {
            let mut out = Vec::new();
            // R4.3-FIX F4-004/005: dedicated AAD version byte (0x01), NOT the
            // envelope format version (2). Reconciles to the membership AAD
            // golden's leading byte.
            out.push(AAD_VERSION);
            out.extend_from_slice(&self.codepoint.to_be_bytes());
            out.extend_from_slice(&self.body_cid);
            out.extend_from_slice(&self.stanza_index.to_be_bytes());
            out.extend_from_slice(&self.recipient_key_generation.to_be_bytes());
            // R4.3-FIX F4-018: Layer-C drop band uses u16 length widths
            // (documented per-object width contract; see module §4.1 note).
            let count =
                u16::try_from(self.sorted_recipient_dids.len()).expect("recipient count fits u16");
            out.extend_from_slice(&count.to_be_bytes());
            for did in &self.sorted_recipient_dids {
                let len = u16::try_from(did.len()).expect("recipient DID len fits u16");
                out.extend_from_slice(&len.to_be_bytes());
                out.extend_from_slice(did);
            }
            // Non-default plaintext-sender variant ONLY (U4). The DEFAULT
            // Sealed-Sender path leaves this empty — the sender-DID is in
            // `sealed_inner`, never here.
            if let Some(sender) = &self.plaintext_sender_did {
                let len = u16::try_from(sender.len()).expect("sender DID len fits u16");
                out.extend_from_slice(&len.to_be_bytes());
                out.extend_from_slice(sender);
            }
            out
        }
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
    /// INSIDE the ciphertext, NOT in the AAD; the AAD binds the `audience`
    /// (recipient-targeting) + body-CID + recipient_key_generation.
    ///
    /// R4.3-FIX F4-006: NO `coarse_epoch` parameter — DropToRecipient carries
    /// NEITHER timestamp NOR coarse bucket (M-14). Freshness rides
    /// `recipient_key_generation` + the nonce-cache.
    pub fn seal_sealed_sender(
        _recipient_pk: &RecipientPubKey,
        _audience_did: &AudienceDid,
        _sender_did: &SenderDid,
        _body_cid: &BodyCid,
        _recipient_key_generation: u32,
        _plaintext: &[u8],
    ) -> EncryptedEnvelope {
        unimplemented!("R5 wires benten_drop::layer_c::seal_sealed_sender")
    }

    /// PRODUCTION call site — single-recipient HPKE-base seal under the
    /// plaintext-sender NON-DEFAULT path (`0x6500`): sender-DID bound INTO
    /// the AAD (U4); the AAD also binds the `audience` (recipient-targeting).
    ///
    /// R4.3-FIX F4-006: NO `coarse_epoch` parameter (M-14).
    pub fn seal_plaintext_sender(
        _recipient_pk: &RecipientPubKey,
        _audience_did: &AudienceDid,
        _sender_did: &SenderDid,
        _body_cid: &BodyCid,
        _recipient_key_generation: u32,
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

    /// PRODUCTION call site — group multi-stanza seal (`0x6520`), DEFAULT
    /// path: HONORS Sealed-Sender (R4-FIX F4-003 / BR-1 ruling 1). Each
    /// stanza's PLAINTEXT AAD binds the U17 tuple WITHOUT the sender-DID;
    /// the inner-sender-DID is sealed INSIDE the per-stanza payload.
    pub fn seal_group_multi(
        _recipient_pks: &[RecipientPubKey],
        _sender_did: &SenderDid,
        _body_cid: &BodyCid,
        _recipient_key_generation: u32,
        _plaintext: &[u8],
    ) -> EncryptedEnvelope {
        unimplemented!(
            "R5 wires benten_drop::layer_c::seal_group_multi (0x6520, Sealed-Sender DEFAULT)"
        )
    }

    /// PRODUCTION call site — group multi-stanza seal under the
    /// NON-DEFAULT plaintext-sender posture (`0x6520` with the
    /// `plaintext_sender_did` AAD field set). EXPLICITLY non-default —
    /// exists only so the paired metadata-disclosure control + the
    /// substitution/re-target arms can exercise a real per-stanza AAD
    /// field-set. NOT the shipped default (BR-1 ruling 1).
    pub fn seal_group_multi_plaintext_sender(
        _recipient_pks: &[RecipientPubKey],
        _sender_did: &SenderDid,
        _body_cid: &BodyCid,
        _recipient_key_generation: u32,
        _plaintext: &[u8],
    ) -> EncryptedEnvelope {
        unimplemented!(
            "R5 wires the NON-default plaintext-sender group seal (0x6520; paired control only)"
        )
    }

    /// PRODUCTION call site — group multi-stanza open (recipient at
    /// `my_index` opens via their stanza). On the DEFAULT path it recovers
    /// the inner-sender-DID post-decrypt. Tampered/substituted/reordered
    /// stanza → `Err`.
    pub fn open_group_stanza(
        _recipient_sk: &RecipientSecKey,
        _my_index: usize,
        _env: &EncryptedEnvelope,
    ) -> Result<(Vec<u8>, SenderDid), LayerCError> {
        unimplemented!("R5 wires benten_drop::layer_c::open_group_stanza")
    }

    /// PRODUCTION call site — canonical serialize to wire bytes (V2 + BE).
    /// What the relay sees on the network. The serialized form concatenates
    /// the PLAINTEXT AAD (clear) + the opaque sealed/wrapped material
    /// (`enc`/`sealed_inner` + `ciphertext`/`wrapped_cek`, opaque to relay).
    pub fn serialize(_env: &EncryptedEnvelope) -> Vec<u8> {
        unimplemented!("R5 wires benten_drop::layer_c::serialize")
    }

    /// PRODUCTION helper — the PLAINTEXT AAD region of a serialized
    /// SINGLE-RECIPIENT envelope (the bytes a relay reads in the clear,
    /// EXCLUDING the opaque `enc` + `ciphertext`). DETERMINISTIC so the
    /// single-recipient Sealed-Sender wire-scan + frozen golden are
    /// computable at red-phase (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD).
    #[must_use]
    pub fn single_plaintext_aad_region(env: &EncryptedEnvelope) -> Vec<u8> {
        match env {
            EncryptedEnvelope::HpkeBase { binding, .. } => binding.plaintext_aad_bytes(),
            EncryptedEnvelope::HpkeMultiBase { .. } => Vec::new(),
        }
    }

    /// PRODUCTION helper — the concatenated PLAINTEXT AAD region of a
    /// serialized group envelope (the bytes a relay reads in the clear,
    /// EXCLUDING the opaque sealed/wrapped material). DETERMINISTIC so the
    /// F4-003 Sealed-Sender wire-scan is computable at red-phase.
    #[must_use]
    pub fn group_plaintext_aad_region(env: &EncryptedEnvelope) -> Vec<u8> {
        match env {
            EncryptedEnvelope::HpkeMultiBase { stanzas, .. } => {
                let mut out = Vec::new();
                for st in stanzas {
                    out.extend_from_slice(&st.plaintext_aad_bytes());
                }
                out
            }
            EncryptedEnvelope::HpkeBase { .. } => Vec::new(),
        }
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
    AAD_VERSION, BindingContext, DROP_TO_RECIPIENT_SEALED_SENDER, ENVELOPE_FORMAT_VERSION,
    EncryptedEnvelope, HYBRID_X25519_MLKEM768, HpkeRecipientStanza, LAYER_C_DROP,
    LAYER_C_DROP_MULTI_RECIPIENT, LayerCError, did, fixed_body_cid, fixed_pk, fixed_sk,
    group_plaintext_aad_region, open_group_stanza, open_single, seal_group_multi,
    seal_group_multi_plaintext_sender, seal_plaintext_sender, seal_sealed_sender, serialize,
};

/// Lowercase-hex of a byte slice (test-local; no external dep).
fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

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
    let audience = did("did:key:zRecipientAudience");
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid(0xC1);
    let plaintext = b"layer-c single recipient payload".to_vec();

    let env = seal_sealed_sender(&pk, &audience, &sender, &body_cid, 0, &plaintext);
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
    let audience = did("did:key:zRecipientAudience");
    let sender = did("did:key:zAlice");
    let body_cid = fixed_body_cid(0xC2);

    let env = seal_sealed_sender(&pk, &audience, &sender, &body_cid, 0, b"secret");
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
        &did("did:key:zRecipientAudience"),
        &did("did:key:zAlice"),
        &fixed_body_cid(0xC3),
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
//          DEFAULT honors Sealed-Sender (R4-FIX F4-003 / BR-1 ruling 1).
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
        let (recovered, _recovered_sender) = open_group_stanza(sk, idx, &env)
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

/// F-LC-2 PIN 5 (R4-FIX F4-003 / BR-1 ruling 1) — the DEFAULT `0x6520`
/// group send HONORS Sealed-Sender: the sender-DID is bound per-stanza
/// INSIDE the sealed payload, and DOES NOT appear anywhere in the
/// PLAINTEXT AAD region of the serialized group envelope. This is the
/// load-bearing consequence of ruling 1 — group sends must NOT silently
/// defeat the Sealed-Sender default. would-FAIL if the default group seal
/// bound the sender-DID into the plaintext per-stanza AAD (the bug the old
/// F-LC-2 stanza shape had).
#[test]
#[ignore = "RED-PHASE: F-LC-2 — DEFAULT 0x6520 group send honors Sealed-Sender, sender-DID NOT in plaintext AAD (F4-003); un-ignore at R5"]
fn f_lc_2_default_group_send_honors_sealed_sender_no_plaintext_sender_did() {
    let pks = [fixed_pk(0x60), fixed_pk(0x61), fixed_pk(0x62)];
    let sender = did("did:key:zGroupSenderUNIQUEMARKER");
    let body_cid = fixed_body_cid(0xD6);

    let env = seal_group_multi(&pks, &sender, &body_cid, 0, b"group payload");

    // (a) Typed-shape guard: NO stanza carries a plaintext_sender_did on
    //     the DEFAULT path (it lives in `sealed_inner` instead).
    match &env {
        EncryptedEnvelope::HpkeMultiBase { stanzas, .. } => {
            for (i, st) in stanzas.iter().enumerate() {
                assert!(
                    st.plaintext_sender_did.is_none(),
                    "F-LC-2 (F4-003): on the DEFAULT 0x6520 group path, \
                     stanza {i} MUST NOT carry a plaintext_sender_did — \
                     Sealed-Sender binds it INSIDE `sealed_inner`. would-\
                     FAIL if the default leaked the sender-DID into the AAD."
                );
                assert!(
                    !st.sealed_inner.is_empty(),
                    "F-LC-2 (F4-003): each DEFAULT stanza MUST carry a sealed \
                     inner payload (where the inner-sender-DID lives)."
                );
            }
        }
        EncryptedEnvelope::HpkeBase { .. } => panic!("group seal MUST produce HpkeMultiBase"),
    }

    // (b) WIRE-SCAN: the sender-DID byte sequence MUST NOT appear in the
    //     concatenated PLAINTEXT AAD region of the serialized envelope.
    let plaintext_aad = group_plaintext_aad_region(&env);
    let leaks_in_aad = plaintext_aad
        .windows(sender.len())
        .any(|w| w == sender.as_slice());
    assert!(
        !leaks_in_aad,
        "F-LC-2 (F4-003 / BR-1 ruling 1): the DEFAULT 0x6520 group send \
         MUST HONOR Sealed-Sender — the sender-DID MUST NOT appear in the \
         PLAINTEXT per-stanza AAD region of the serialized envelope. It is \
         bound per-stanza INSIDE the sealed payload, recovered only \
         post-decrypt. would-FAIL if the default group path bound the \
         sender-DID into the plaintext AAD (silently defeating the \
         Sealed-Sender default for every group send)."
    );

    // (c) Full-wire scan (defense-in-depth): the sender-DID does not leak
    //     into ANY plaintext wire field of the serialized group envelope.
    let wire = serialize(&env);
    let leaks_in_wire = wire.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        !leaks_in_wire,
        "F-LC-2 (F4-003): the serialized DEFAULT 0x6520 group wire MUST NOT \
         contain the sender-DID in any plaintext field."
    );
}

/// F-LC-2 PIN 6 (R4-FIX F4-003 paired control) — the EXPLICITLY-non-default
/// plaintext-sender group variant (`0x6520` with `plaintext_sender_did`
/// set) DOES place the sender-DID in the plaintext AAD (U4). This is the
/// paired positive control that proves PIN 5's wire-scan is not vacuously
/// passing (the two paths must differ observably). The non-default variant
/// is NOT the shipped default. would-FAIL if even the non-default variant
/// hid the sender-DID (then the scanner cannot distinguish the paths).
#[test]
#[ignore = "RED-PHASE: F-LC-2 — paired control: NON-default plaintext-sender 0x6520 DOES carry sender-DID in AAD (F4-003); un-ignore at R5"]
fn f_lc_2_nondefault_plaintext_sender_group_carries_sender_did_in_aad() {
    let pks = [fixed_pk(0x70), fixed_pk(0x71)];
    let sender = did("did:key:zGroupSenderUNIQUEMARKER");
    let body_cid = fixed_body_cid(0xD7);

    let env = seal_group_multi_plaintext_sender(&pks, &sender, &body_cid, 0, b"group payload");

    match &env {
        EncryptedEnvelope::HpkeMultiBase { stanzas, .. } => {
            for (i, st) in stanzas.iter().enumerate() {
                assert_eq!(
                    st.plaintext_sender_did.as_deref(),
                    Some(sender.as_slice()),
                    "F-LC-2 (F4-003 control): the NON-default plaintext-sender \
                     group variant MUST bind the sender-DID into stanza {i}'s \
                     plaintext AAD (U4)."
                );
            }
        }
        EncryptedEnvelope::HpkeBase { .. } => {
            panic!("plaintext-sender group seal MUST produce HpkeMultiBase")
        }
    }

    let plaintext_aad = group_plaintext_aad_region(&env);
    let leaks = plaintext_aad
        .windows(sender.len())
        .any(|w| w == sender.as_slice());
    assert!(
        leaks,
        "F-LC-2 (F4-003 PAIRED CONTROL): the NON-default plaintext-sender \
         group variant MUST place the sender-DID in the plaintext AAD (U4). \
         If this control fails, PIN 5's wire-scan cannot distinguish hiding \
         from a broken scan — the default and non-default paths must differ \
         observably."
    );
}

/// The deterministic per-stanza fixture for the group plaintext-AAD golden.
/// Built DIRECTLY (the seal fns `unimplemented!()` at red-phase) so the
/// canonical `plaintext_aad_bytes()` serializer is driven without panicking.
/// DEFAULT (Sealed-Sender) path: `plaintext_sender_did = None`.
fn f_lc_2_group_stanza_fixture() -> HpkeRecipientStanza {
    let mut body_cid = [0u8; 32];
    body_cid[0] = 0xD8;
    HpkeRecipientStanza {
        codepoint: LAYER_C_DROP_MULTI_RECIPIENT, // 0x6520
        body_cid,
        sorted_recipient_dids: vec![did("did:key:zRecipientA"), did("did:key:zRecipientB")],
        stanza_index: 0,
        recipient_key_generation: 0,
        sealed_inner: vec![0xAB; 8], // opaque; not part of the plaintext AAD
        plaintext_sender_did: None,  // DEFAULT Sealed-Sender path
        wrapped_cek: vec![0xCD; 8],  // opaque; not part of the plaintext AAD
    }
}

/// FROZEN big-endian golden vector for the DEFAULT group stanza plaintext
/// AAD (CLUSTER-1 byte-0-guard + layout). Layout (BE):
///   aad_version u8 | codepoint u16 | body_cid[32] | stanza_index u32 |
///   recipient_key_generation u32 | recipient_count u16 |
///   (len u16 || bytes) per sorted recipient DID.
/// No coarse_epoch (F4-006); leads with `AAD_VERSION = 0x01` (F4-004/005).
/// R5 confirms-or-deliberately-updates against the real encoder (M-20).
const F_LC_2_GROUP_STANZA_AAD_HEX: &str = "016520d8000000000000000000000000000000000000000000000000000000000000000000000000000000000200136469643a6b65793a7a526563697069656e744100136469643a6b65793a7a526563697069656e7442";

/// F-LC-2 PIN 7 (R4.4-FIX CLUSTER-1) — the DEFAULT group stanza plaintext-AAD
/// byte-0 is the dedicated `AAD_VERSION` (= 0x01), NOT the envelope
/// `ENVELOPE_FORMAT_VERSION` (= 0x02), and the full layout is FROZEN to the
/// big-endian golden. This MIRRORS the sibling `f_lc_abuse` byte-0
/// anti-conflation guard so the two files cannot silently re-diverge on the
/// `0x65xx` drop/recipient-band envelope AAD. would-FAIL if a future edit
/// reverted `plaintext_aad_bytes`' leading byte to the format version (the
/// F4-004/005 cross-engine AEAD-open break), emitted an LE codepoint, or
/// drifted the field order / length widths.
#[test]
#[ignore = "RED-PHASE: F-LC-2 — group stanza plaintext-AAD byte-0 == AAD_VERSION (not format ver) + frozen BE layout (CLUSTER-1); un-ignore at R5"]
fn f_lc_2_group_stanza_aad_byte0_is_aad_version_not_format_version_and_frozen_layout() {
    let stanza = f_lc_2_group_stanza_fixture();
    let bytes = stanza.plaintext_aad_bytes();

    // Anti-conflation byte-0 guard (mirrors f_lc_abuse).
    assert_eq!(
        bytes[0], AAD_VERSION,
        "F-LC-2 (CLUSTER-1 / F4-004/005): the group stanza plaintext-AAD \
         byte-0 MUST be the dedicated AAD_VERSION (0x01), NOT the envelope \
         serialization format_version (0x02). Reverting this re-introduces \
         the cross-engine AEAD-open break + re-diverges from f_lc_abuse."
    );
    assert_ne!(
        bytes[0], ENVELOPE_FORMAT_VERSION,
        "F-LC-2 (CLUSTER-1 / F4-004/005): byte-0 MUST NOT be the envelope \
         format version — the AAD version axis and the serialization-format \
         axis are DISTINCT (R0.5 §4.1)."
    );

    // BE codepoint pair (anti-LE drift, M-19).
    assert_eq!(
        &bytes[1..3],
        &[0x65, 0x20],
        "F-LC-2 (CLUSTER-1): group codepoint 0x6520 MUST be big-endian \
         (0x65,0x20) at offset 1, never little-endian (0x20,0x65)."
    );

    // FROZEN BE layout — drift flips the pin.
    assert_eq!(
        to_hex(&bytes),
        F_LC_2_GROUP_STANZA_AAD_HEX,
        "F-LC-2 (CLUSTER-1): the DEFAULT group stanza plaintext-AAD MUST \
         serialize to the FROZEN big-endian layout (aad_version, codepoint, \
         body_cid, stanza_index, recipient_key_generation, recipient_count, \
         per-DID len||bytes) with NO coarse_epoch and NO plaintext sender-DID. \
         R5 confirms-or-deliberately-updates this literal (M-20)."
    );

    // The sender-DID is NOT on the DEFAULT path plaintext AAD (the
    // plaintext_sender_did is None), so no sender bytes trail the DID list.
    let sender = did("did:key:zGroupSenderUNIQUEMARKER");
    let leaks = bytes.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        !leaks,
        "F-LC-2 (CLUSTER-1): the DEFAULT group stanza plaintext-AAD MUST NOT \
         carry the sender-DID (it lives in `sealed_inner`)."
    );
}

// ===========================================================================
// F-LC-3 — Sealed-Sender DEFAULT (0x6510): sender-DID NOT on the wire.
// ===========================================================================
// Highest-novelty surface (R2 §1 Group 7). The metadata posture IS the
// wire contract.

/// The deterministic single-recipient `0x6510` Sealed-Sender envelope-AAD
/// fixture. Built DIRECTLY (the seal fns `unimplemented!()` at red-phase) so
/// the canonical `BindingContext::plaintext_aad_bytes()` serializer is driven
/// without panicking. The values are IDENTICAL to the sibling
/// `f_lc_abuse::f_inv18_1_sealed_aad_fixture` so the two files freeze the
/// SAME golden (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD):
///   aad_version 0x01 | codepoint 0x6510 |
///   audience "did:key:zRecipientAudienceUNIQUE" (32 bytes) |
///   body_cid [0xE0, 0; 31] | recipient_key_generation 0.
fn f_lc_3_sealed_sender_aad_fixture() -> BindingContext {
    let mut body_cid = [0u8; 32];
    body_cid[0] = 0xE0;
    BindingContext::DropSealedSender {
        aad_version: AAD_VERSION, // 0x01 (dedicated AAD prefix, NOT format ver 0x02)
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        audience_did: did("did:key:zRecipientAudienceUNIQUE"),
        body_cid,
        recipient_key_generation: 0,
    }
}

/// FROZEN big-endian golden vector for the DEFAULT (`0x6510`) single-recipient
/// Sealed-Sender envelope AAD — the canonical union
/// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`,
/// NO sender-DID region, NO coarse_epoch (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD /
/// F4-006). This literal is BYTE-IDENTICAL to the sibling
/// `f_lc_abuse::F_INV18_1_SEALED_AAD_HEX` (computed once from the shared BE
/// layout via a throwaway script, M-20). R5 confirms-or-deliberately-updates
/// it against the real encoder. If these two literals ever differ, the siblings
/// have re-diverged on the `0x6510` envelope AAD (the F-NEW-SS-AUD regression).
const F_LC_SEALED_SENDER_AAD_HEX: &str =
    "01651000206469643a6b65793a7a526563697069656e7441756469656e6365554e49515545e00000000000000000000000000000000000000000000000000000000000000000000000";

/// F-LC-3 PIN 1 (R4.4-FIX CLUSTER-1 / F-NEW-SS-AUD) — the DEFAULT (`0x6510`)
/// single-recipient Sealed-Sender envelope AAD binds the canonical union
/// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
/// — byte-0 == `AAD_VERSION` (≠ `ENVELOPE_FORMAT_VERSION`), the BE codepoint,
/// the recipient `audience` present, the sender-DID ABSENT — and reproduces
/// the FROZEN golden that is BYTE-IDENTICAL to the sibling
/// `f_lc_abuse::F_INV18_1_SEALED_AAD_HEX`. This closes the audience-axis
/// divergence: `f_lc_hpke`'s single-recipient binding now matches the sibling
/// (and the Inv-16 `Recipient` binding) exactly. would-FAIL if the binding
/// omits the spec-mandated `audience` (§3.3:484), reverts byte-0 to the format
/// version (the F4-004/005 cross-engine AEAD-open break), re-adds a
/// coarse_epoch, leaks the sender-DID, or drifts the layout.
#[test]
#[ignore = "RED-PHASE: F-LC-3 — single-recipient 0x6510 AAD == canonical union (audience bound, sender absent) + frozen BE golden == sibling (CLUSTER-1 / F-NEW-SS-AUD); un-ignore at R5"]
fn f_lc_3_sealed_sender_single_recipient_aad_binds_audience_union_and_frozen_golden() {
    let binding = f_lc_3_sealed_sender_aad_fixture();
    let audience = did("did:key:zRecipientAudienceUNIQUE");
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let bytes = binding.plaintext_aad_bytes();

    // Anti-conflation byte-0 guard (mirrors f_lc_abuse single-recipient).
    assert_eq!(
        bytes[0], AAD_VERSION,
        "F-LC-3 (CLUSTER-1 / F4-004/005): the single-recipient 0x6510 AAD \
         byte-0 MUST be the dedicated AAD_VERSION (0x01), NOT the envelope \
         serialization format_version (0x02)."
    );
    assert_ne!(
        bytes[0], ENVELOPE_FORMAT_VERSION,
        "F-LC-3 (CLUSTER-1): byte-0 MUST NOT be the envelope format version \
         — the AAD-version axis and the serialization-format axis are \
         DISTINCT (R0.5 §4.1)."
    );

    // BE codepoint pair (anti-LE drift, M-19).
    assert_eq!(
        &bytes[1..3],
        &[0x65, 0x10],
        "F-LC-3 (CLUSTER-1): the Sealed-Sender drop codepoint 0x6510 MUST be \
         big-endian (0x65,0x10) at offset 1."
    );

    // The recipient AUDIENCE MUST be bound (§3.3:484 recipient-targeting) —
    // this is the axis F-NEW-SS-AUD was missing. would-FAIL on the
    // audience-less form the stale binding froze.
    let audience_present = bytes.windows(audience.len()).any(|w| w == audience.as_slice());
    assert!(
        audience_present,
        "F-LC-3 (F-NEW-SS-AUD): the single-recipient 0x6510 AAD MUST bind the \
         recipient AUDIENCE DID (§3.3:484 recipient-targeting). would-FAIL on \
         the stale audience-less binding that diverged from f_lc_abuse + the \
         Inv-16 Recipient binding."
    );

    // The SENDER-DID MUST be ABSENT (Sealed-Sender — it lives inside the
    // ciphertext, not the AAD).
    let sender_leaks = bytes.windows(sender.len()).any(|w| w == sender.as_slice());
    assert!(
        !sender_leaks,
        "F-LC-3 (CLUSTER-1): the single-recipient 0x6510 AAD MUST NOT carry \
         the sender-DID — Sealed-Sender binds it INSIDE the ciphertext."
    );

    // FROZEN BE layout — and it MUST equal the sibling's golden (shared
    // literal proves the siblings cannot silently re-diverge).
    assert_eq!(
        to_hex(&bytes),
        F_LC_SEALED_SENDER_AAD_HEX,
        "F-LC-3 (CLUSTER-1 / F-NEW-SS-AUD): the single-recipient 0x6510 AAD \
         MUST serialize to the FROZEN canonical union layout (aad_version, \
         codepoint, audience_len||audience, body_cid, recipient_key_gen) — \
         BYTE-IDENTICAL to the sibling f_lc_abuse::F_INV18_1_SEALED_AAD_HEX. \
         If this literal ever differs from the sibling's, the two engines \
         have re-diverged on the 0x6510 envelope AAD. R5 confirms-or-\
         deliberately-updates this literal (M-20)."
    );
}

/// F-LC-3 PIN 2 — on the DEFAULT (`0x6510`) path the serialized wire
/// bytes DO NOT contain the sender-DID. The sender-DID is bound INSIDE
/// the ciphertext (HPKE inner-payload); the on-wire AAD = audience binding
/// (codepoint + audience + body-CID + recipient-key-generation; NO timestamp
/// NOR coarse-epoch — M-14 / F4-006). would-FAIL if the default seal leaks the
/// sender-DID into the AAD (the bug `0x6500` deliberately has, that `0x6510`
/// fixes).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — sealed-sender default 0x6510 sender-DID NOT on wire; un-ignore at R5"]
fn f_lc_3_sealed_sender_default_omits_sender_did_from_wire() {
    let audience = did("did:key:zRecipientAudienceUNIQUE");
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let env = seal_sealed_sender(
        &fixed_pk(0x50),
        &audience,
        &sender,
        &fixed_body_cid(0xE0),
        0,
        b"sealed-sender payload",
    );

    // The default seal MUST carry the Sealed-Sender binding (audience bound,
    // no sender-DID field in the AAD-bearing binding).
    match &env {
        EncryptedEnvelope::HpkeBase {
            binding:
                BindingContext::DropSealedSender {
                    codepoint,
                    audience_did,
                    ..
                },
            ..
        } => {
            assert_eq!(
                *codepoint, DROP_TO_RECIPIENT_SEALED_SENDER,
                "F-LC-3: the v1-beta DEFAULT MUST be the Sealed-Sender drop \
                 (0x6510)."
            );
            assert_eq!(
                audience_did, &audience,
                "F-LC-3: the Sealed-Sender drop AAD MUST bind the recipient \
                 audience DID (§3.3:484)."
            );
        }
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

/// F-LC-3 PIN 3 — PAIRED POSITIVE CONTROL: the NON-default plaintext-sender
/// sibling (`0x6500`) DOES carry the sender-DID on the wire (U4). This is
/// the control that proves PIN 2 is not vacuously passing because the
/// scanner is broken. would-FAIL if `0x6500` ALSO hid the sender-DID
/// (then the scanner can't tell the two paths apart).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — paired control 0x6500 DOES carry sender-DID (U4); un-ignore at R5"]
fn f_lc_3_plaintext_sender_sibling_carries_sender_did_on_wire() {
    let audience = did("did:key:zRecipientAudienceUNIQUE");
    let sender = did("did:key:zSenderAliceUNIQUEMARKER");
    let env = seal_plaintext_sender(
        &fixed_pk(0x51),
        &audience,
        &sender,
        &fixed_body_cid(0xE1),
        0,
        b"plaintext-sender payload",
    );

    // The non-default seal MUST carry the plaintext-sender binding with the
    // sender-DID bound INTO the AAD (U4), audience also bound.
    match &env {
        EncryptedEnvelope::HpkeBase {
            binding:
                BindingContext::DropPlaintextSender {
                    codepoint,
                    audience_did,
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
                audience_did, &audience,
                "F-LC-3: the 0x6500 binding MUST also bind the recipient \
                 audience DID (§3.3:484)."
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
         the PIN-2 scanner cannot distinguish hiding from a broken scan — \
         the two paths must differ observably."
    );
}

/// F-LC-3 PIN 4 — post-decrypt the recovered sender-DID equals the bound
/// sender. The Sealed-Sender property is "hidden on the wire, recovered
/// by the recipient." would-FAIL if the inner-payload sender-DID is not
/// recoverable (then Sealed-Sender breaks sender attribution entirely).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — recovered inner sender-DID equals bound; un-ignore at R5"]
fn f_lc_3_recovered_inner_sender_did_equals_bound() {
    let pk = fixed_pk(0x52);
    let sk = fixed_sk(0x52);
    let audience = did("did:key:zRecipientAudience");
    let sender = did("did:key:zCarol");
    let env = seal_sealed_sender(&pk, &audience, &sender, &fixed_body_cid(0xE2), 0, b"hi");

    let (_pt, recovered_sender) =
        open_single(&sk, &env).expect("recipient MUST open the sealed-sender envelope");
    assert_eq!(
        recovered_sender, sender,
        "F-LC-3: the recipient MUST recover the bound inner sender-DID \
         post-decrypt. would-FAIL if the inner-payload sender-DID is not \
         carried/recovered."
    );
}

/// F-LC-3 PIN 5 — a FORGED inner sender-DID is rejected. The inner
/// sender-DID is bound such that tampering with it fails the post-decrypt
/// verify (Inv-16 sender-DID-or-Sealed-Sender clause). would-FAIL if the
/// inner sender-DID is unauthenticated (then anyone can spoof the sender).
#[test]
#[ignore = "RED-PHASE: F-LC-3 — forged inner sender-DID rejected; un-ignore at R5"]
fn f_lc_3_forged_inner_sender_did_rejected() {
    let pk = fixed_pk(0x53);
    let sk = fixed_sk(0x53);
    let audience = did("did:key:zRecipientAudience");
    let sender = did("did:key:zCarol");
    let env = seal_sealed_sender(&pk, &audience, &sender, &fixed_body_cid(0xE3), 0, b"hi");

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
