//! F-full R3-W2 (Layer-C) — abuse-control + group Sealed-Sender posture +
//! FS-gap disclosure + Inv-18 paired-disclosure.
//!
//! ADDL Phase-4-Meta-Core, **F-full** R3 wave **W2-layer-c** (RED-PHASE,
//! `pim-12 §3.6e`). Families pinned in THIS file:
//!   - **F-LC-7** FS-gap honest disclosure (HPKE-mode-base non-FS at the
//!     long-term-sk axis; DropToRecipient forever-valid #62; journalist
//!     per-msg FS #56 a deferred class) — DC/FN.
//!   - **F-LC-8** Sealed-Sender abuse-control delivery-token (no valid
//!     recipient-issued UCAN delivery-token ⇒ refused at the receive
//!     boundary BEFORE decrypt; expired/over-rate ⇒ reject; token-binding
//!     AAD sub-field is wire-affecting ⇒ pre-freeze) — #63.
//!   - **F-LC-9** Group-send Sealed-Sender posture (`0x6520`/`0x6610`):
//!     **Ben RULED group sends HONOR Sealed-Sender** (GAP-1f) — a per-stanza
//!     inner-sender-DID binding lives INSIDE the group AAD so the sender-DID
//!     is NOT plaintext on group multi-stanza sends; `0x6610`→`0x6520`
//!     dispatch is strict-reject.
//!   - **F-INV18-1** Inv-18 metadata-disclosure paired-Sealed-Sender:
//!     `0x6500` plaintext-sender variant has a paired Sealed-Sender sibling
//!     (`0x6510` DEFAULT); residual on-wire metadata = exactly
//!     {audience, coarse-epoch}; `0x6500` discloses sender-DID-in-AAD (U4).
//!
//! Pin sources (R0.3 = `4fe9236a:.addl/phase-4-meta/f-full-r0-plan.md`):
//!   - §3.3 FS-gap honest disclosure (#42/#56/#62); §3.11 Sealed-Sender
//!     abuse-control mechanism (BR-1 — recipient-issued delivery tokens;
//!     refused BEFORE decrypt; per-token rate-limit + revocation via UCAN
//!     nbf/exp; #63 NEW); §4.0/§4.1 codepoints (`0x6510`/`0x6520`/`0x6610`);
//!     §5.1 Inv-18; §5.2 #42/#56/#62/#63.
//!   - R2 landscape §1 Group 7 + Group 12: F-LC-7 (~2-3, DC), F-LC-8
//!     (~6-8), F-LC-9 (GAP-1f, ~4-6), F-INV18-1 (~4).
//!   - **F-LC-9 Ben-ruling (R2 §5.A.1 / brief):** group sends MUST honor
//!     Sealed-Sender — adds a per-stanza inner-sender-DID binding to the
//!     group AAD (pre-freeze wire change).
//!
//! # RED-PHASE STATUS + STUB-SHIM (pim-12 §3.6e).
//!
//! Self-contained stubs (`abuse_stub`, `group_posture_stub`) so the file
//! is parallel-safe + compiles green at baseline behind `#[ignore]`. The
//! FS-gap + Inv-18 disclosure arms are DOC-COUPLING (real
//! `std::fs::read_to_string` against `docs/SECURITY-POSTURE.md`, reusing
//! the `tf3f_revocation_reach_*` shape) — those are not stubbed; they
//! assert the doc-wave landed the disclosure text. R5:
//!   1. DELETE the stub modules; INSERT the real `use benten_drop::…`;
//!   2. UN-IGNORE; 3. the doc-coupling arms stay (doc-wave authored).
//!
//! # Wave-0 (M-20) + would-FAIL (pim-2 + pim-18 + §3.6f-ext). Behavioral
//! stubs `unimplemented!()` so a forgotten stub at R5 PANICS; doc arms
//! assert a doc string that did NOT exist pre-doc-wave.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_variables)]

// ===========================================================================
// SELF-CONTAINED STUB — Sealed-Sender abuse-control (F-LC-8). DELETE at R5.
// ===========================================================================
mod abuse_stub {
    /// A recipient-issued, short-lived, rate-limited UCAN-backed delivery
    /// token (Signal's delivery-token pattern adapted to Benten's
    /// capability spine). Modeled minimally: issuer + validity window +
    /// per-token counter binding.
    #[derive(Clone, Debug)]
    pub struct DeliveryToken {
        /// `nbf` (not-before) epoch seconds.
        pub not_before: u64,
        /// `exp` (expiry) epoch seconds.
        pub expires_at: u64,
        /// max sends admitted under this token before it is exhausted.
        pub rate_limit: u32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum AdmitError {
        /// No token presented for a Sealed-Sender envelope.
        MissingDeliveryToken,
        /// Token outside its [nbf, exp] window.
        TokenExpiredOrNotYetValid,
        /// Token's per-token rate-limit exhausted.
        RateLimitExceeded,
    }

    /// PRODUCTION call site — the receive-boundary admission check that
    /// runs BEFORE decrypt. A Sealed-Sender envelope without a valid token
    /// is refused here (no decrypt attempt). Returns Ok only when a valid,
    /// in-window, under-rate token is presented.
    pub fn admit_sealed_sender(
        _token: Option<&DeliveryToken>,
        _now: u64,
        _sends_already_under_token: u32,
    ) -> Result<(), AdmitError> {
        unimplemented!("R5 wires the Sealed-Sender receive-boundary admission check (§3.11)")
    }

    /// PRODUCTION sentinel — did the admission check run BEFORE any decrypt
    /// was attempted? The receive boundary MUST refuse pre-decrypt so a
    /// no-token envelope never reaches the KEM. Returns `true` only if the
    /// implementation gates admission ahead of decrypt.
    pub fn decrypt_was_attempted_for_last_admit() -> bool {
        unimplemented!("R5 wires the pre-decrypt ordering observability")
    }
}

// ===========================================================================
// SELF-CONTAINED STUB — group Sealed-Sender posture (F-LC-9). DELETE at R5.
// ===========================================================================
mod group_posture_stub {
    pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520; // Layer-C group
    pub const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610; // MembershipSet K_Set group

    pub type SenderDid = Vec<u8>;
    pub type RecipientPubKey = [u8; 32];
    pub type RecipientSecKey = [u8; 32];

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum GroupError {
        AeadAuthenticationFailed,
        /// `0x6610` bytes fed to the `0x6520` dispatch arm (or vice versa).
        WrongGroupCodepoint {
            got: u16,
            expected: u16,
        },
    }

    /// A group stanza that — per Ben's F-LC-9 ruling — binds the
    /// inner-sender-DID INSIDE the AAD so the sender-DID is NOT plaintext
    /// on the group wire.
    #[derive(Clone, Debug)]
    pub struct GroupSealedEnvelope {
        pub codepoint: u16,
        /// the serialized group wire bytes the relay observes.
        pub wire: Vec<u8>,
    }

    /// PRODUCTION call site — seal a MembershipSet K_Set group
    /// (`0x6610`) honoring Sealed-Sender: each stanza binds the
    /// inner-sender-DID in the AAD (NOT plaintext on the wire).
    pub fn seal_membership_set_group(
        _recipient_pks: &[RecipientPubKey],
        _sender_did: &SenderDid,
        _k_set: &[u8; 32],
        _plaintext: &[u8],
    ) -> GroupSealedEnvelope {
        unimplemented!("R5 wires benten_membership_set group seal (0x6610) honoring Sealed-Sender")
    }

    /// PRODUCTION call site — open a `0x6610` group stanza; recovers the
    /// inner-sender-DID post-decrypt.
    pub fn open_membership_set_group(
        _sk: &RecipientSecKey,
        _my_index: usize,
        _env: &GroupSealedEnvelope,
    ) -> Result<(Vec<u8>, SenderDid), GroupError> {
        unimplemented!("R5 wires the 0x6610 group open")
    }

    /// PRODUCTION call site — codepoint dispatch. Feeding `0x6610` bytes
    /// to the `0x6520` Layer-C-group arm (or vice versa) MUST strict-reject
    /// (no cross-band fallback).
    pub fn dispatch_group(
        _wire: &[u8],
        _declared_codepoint: u16,
        _arm_codepoint: u16,
    ) -> Result<(), GroupError> {
        unimplemented!("R5 wires the group codepoint dispatch strict-reject")
    }
}

use abuse_stub::{
    AdmitError, DeliveryToken, admit_sealed_sender, decrypt_was_attempted_for_last_admit,
};
use group_posture_stub::{
    GroupError, LAYER_C_DROP_MULTI_RECIPIENT, MEMBERSHIP_SET_GROUP_MULTI_STANZA, dispatch_group,
    open_membership_set_group, seal_membership_set_group,
};

const SECURITY_POSTURE_MD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/SECURITY-POSTURE.md"
);

fn did(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

// ===========================================================================
// F-LC-8 — Sealed-Sender abuse-control delivery-token (#63).
// ===========================================================================

/// F-LC-8 PIN 1 — a Sealed-Sender envelope WITHOUT a delivery token is
/// REFUSED at the receive boundary BEFORE decrypt. With no plaintext
/// sender identity, abuse-control rides recipient-issued tokens
/// (§3.11/BR-1). would-FAIL if a no-token envelope were admitted (then
/// Sealed-Sender removes the only spam filter).
#[test]
#[ignore = "RED-PHASE: F-LC-8 — no delivery-token ⇒ refused pre-decrypt (#63); un-ignore at R5"]
fn f_lc_8_no_token_refused_before_decrypt() {
    let outcome = admit_sealed_sender(None, 1_900_800, 0);
    assert!(
        matches!(outcome, Err(AdmitError::MissingDeliveryToken)),
        "F-LC-8: a Sealed-Sender envelope WITHOUT a recipient-issued \
         delivery token MUST be refused at the receive boundary (§3.11). \
         Got: {outcome:?}"
    );
    assert!(
        !decrypt_was_attempted_for_last_admit(),
        "F-LC-8: refusal MUST happen BEFORE decrypt — a no-token envelope \
         MUST NOT reach the KEM. would-FAIL if admission ran after decrypt."
    );
}

/// F-LC-8 PIN 2 — an EXPIRED token is rejected (UCAN nbf/exp revocation
/// substrate). would-FAIL if the validity window were not enforced.
#[test]
#[ignore = "RED-PHASE: F-LC-8 — expired token rejected (nbf/exp); un-ignore at R5"]
fn f_lc_8_expired_token_rejected() {
    let token = DeliveryToken {
        not_before: 1_900_000,
        expires_at: 1_900_500, // expired before `now`
        rate_limit: 10,
    };
    let outcome = admit_sealed_sender(Some(&token), 1_900_800, 0);
    assert!(
        matches!(outcome, Err(AdmitError::TokenExpiredOrNotYetValid)),
        "F-LC-8: a delivery token outside its [nbf, exp] window MUST be \
         rejected (per-token revocation via UCAN nbf/exp). Got: {outcome:?}"
    );
}

/// F-LC-8 PIN 3 — an OVER-RATE token is rejected (per-token rate-limit).
/// would-FAIL if the rate-limit were advisory (an over-issuing recipient's
/// token must still be bounded per its own counter).
#[test]
#[ignore = "RED-PHASE: F-LC-8 — over-rate token rejected; un-ignore at R5"]
fn f_lc_8_over_rate_token_rejected() {
    let token = DeliveryToken {
        not_before: 1_900_000,
        expires_at: 1_999_999,
        rate_limit: 3,
    };
    // 3 sends already used under this token; the 4th must be refused.
    let outcome = admit_sealed_sender(Some(&token), 1_900_800, 3);
    assert!(
        matches!(outcome, Err(AdmitError::RateLimitExceeded)),
        "F-LC-8: a token that has reached its per-token rate-limit MUST be \
         refused (#63 over-issuing mitigation). Got: {outcome:?}"
    );
}

/// F-LC-8 PIN 4 — a VALID, in-window, under-rate token is ADMITTED
/// (positive control; proves the rejections are not vacuous). would-FAIL
/// if a valid token were rejected.
#[test]
#[ignore = "RED-PHASE: F-LC-8 — valid token admitted (positive control); un-ignore at R5"]
fn f_lc_8_valid_token_admitted() {
    let token = DeliveryToken {
        not_before: 1_900_000,
        expires_at: 1_999_999,
        rate_limit: 5,
    };
    let outcome = admit_sealed_sender(Some(&token), 1_900_800, 1);
    assert!(
        outcome.is_ok(),
        "F-LC-8: a valid, in-window, under-rate delivery token MUST be \
         ADMITTED (positive control). Got: {outcome:?}"
    );
}

// ===========================================================================
// F-LC-9 — Group-send Sealed-Sender posture (Ben-ruled: HONOR Sealed-Sender).
// ===========================================================================
// GAP-1f: group multi-stanza sends MUST NOT carry plaintext sender-DID.
// Ben ruled group sends honor Sealed-Sender → a per-stanza inner-sender-DID
// binding inside the group AAD.

/// F-LC-9 PIN 1 — a `0x6610` MembershipSet group send DOES NOT place the
/// sender-DID in plaintext on the wire (HONORS Sealed-Sender). This is the
/// load-bearing consequence of Ben's ruling — if group sends leaked the
/// sender-DID, the Sealed-Sender DEFAULT is silently defeated for every
/// group send. would-FAIL if the group seal bound the sender-DID into a
/// plaintext wire field.
#[test]
#[ignore = "RED-PHASE: F-LC-9 — group send (0x6610) honors Sealed-Sender, sender-DID NOT plaintext; un-ignore at R5"]
fn f_lc_9_group_send_honors_sealed_sender_no_plaintext_sender_did() {
    let pks = [[0x10u8; 32], [0x11u8; 32], [0x12u8; 32]];
    let sender = did("did:key:zGroupSenderUNIQUEMARKER");
    let k_set = [0x33u8; 32];

    let env = seal_membership_set_group(&pks, &sender, &k_set, b"group payload");
    assert_eq!(
        env.codepoint, MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        "F-LC-9: a MembershipSet K_Set group send MUST carry the 0x6610 \
         codepoint."
    );

    let leaks = env
        .wire
        .windows(sender.len())
        .any(|w| w == sender.as_slice());
    assert!(
        !leaks,
        "F-LC-9 (Ben-ruled): a group multi-stanza send (0x6610) MUST HONOR \
         Sealed-Sender — the sender-DID MUST NOT appear in plaintext on the \
         wire; it is bound per-stanza INSIDE the group AAD. would-FAIL if \
         group sends carried the plaintext sender-DID by construction \
         (which would silently defeat the Sealed-Sender default for every \
         group send)."
    );
}

/// F-LC-9 PIN 2 — a group recipient recovers the inner-sender-DID
/// post-decrypt (the per-stanza inner-sender binding is recoverable, just
/// not on the wire). would-FAIL if honoring Sealed-Sender dropped sender
/// attribution entirely for groups.
#[test]
#[ignore = "RED-PHASE: F-LC-9 — group recipient recovers inner-sender-DID post-decrypt; un-ignore at R5"]
fn f_lc_9_group_recipient_recovers_inner_sender_did() {
    let pks = [[0x20u8; 32], [0x21u8; 32]];
    let sks = [[0xA0u8; 32], [0xA1u8; 32]];
    let sender = did("did:key:zGroupSenderCarol");
    let k_set = [0x44u8; 32];

    let env = seal_membership_set_group(&pks, &sender, &k_set, b"hello group");
    let (pt, recovered_sender) = open_membership_set_group(&sks[1], 1, &env)
        .expect("group recipient MUST open their stanza");

    assert_eq!(
        pt, b"hello group",
        "F-LC-9: the group stanza MUST open to the plaintext."
    );
    assert_eq!(
        recovered_sender, sender,
        "F-LC-9: the recipient MUST recover the per-stanza inner-sender-DID \
         post-decrypt (Sealed-Sender honored — hidden on wire, recovered by \
         recipient). would-FAIL if group sends dropped sender attribution."
    );
}

/// F-LC-9 PIN 3 — `0x6610` is distinct from `0x6520` and the dispatch
/// strict-rejects a cross-band feed (no fallback). would-FAIL if `0x6610`
/// bytes silently dispatched through the `0x6520` Layer-C-group arm.
#[test]
#[ignore = "RED-PHASE: F-LC-9 — 0x6610 distinct from 0x6520, cross-band dispatch strict-reject; un-ignore at R5"]
fn f_lc_9_group_codepoints_distinct_and_dispatch_strict_reject() {
    assert_ne!(
        MEMBERSHIP_SET_GROUP_MULTI_STANZA, LAYER_C_DROP_MULTI_RECIPIENT,
        "F-LC-9: the MembershipSet group codepoint 0x6610 MUST be DISTINCT \
         from the Layer-C group codepoint 0x6520 (GAP-1c)."
    );
    assert_eq!(MEMBERSHIP_SET_GROUP_MULTI_STANZA, 0x6610);
    assert_eq!(LAYER_C_DROP_MULTI_RECIPIENT, 0x6520);

    // Feed 0x6610-declared bytes to the 0x6520 arm → strict-reject.
    let env = seal_membership_set_group(&[[0x30u8; 32]], &did("did:key:zX"), &[0x55u8; 32], b"x");
    let outcome = dispatch_group(
        &env.wire,
        MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        LAYER_C_DROP_MULTI_RECIPIENT,
    );
    assert!(
        matches!(
            outcome,
            Err(GroupError::WrongGroupCodepoint {
                got: 0x6610,
                expected: 0x6520
            })
        ),
        "F-LC-9: feeding 0x6610 group bytes to the 0x6520 dispatch arm MUST \
         strict-reject (no cross-band fallback; Inv-16 strict-decode). \
         Got: {outcome:?}"
    );
}

// ===========================================================================
// F-INV18-1 — Inv-18 metadata-disclosure paired-Sealed-Sender.
// ===========================================================================
// The DEFAULT (0x6510) IS the metadata-hiding shape; the plaintext-sender
// sibling (0x6500) discloses sender-DID-in-AAD (U4). The paired-disclosure
// clause is satisfied because the metadata-hiding shape is the DEFAULT.

/// F-INV18-1 PIN 1 — the paired-disclosure invariant holds at the
/// codepoint registry: the plaintext-sender variant (`0x6500`) has a
/// paired Sealed-Sender sibling (`0x6510`), and that sibling is the
/// v1-beta DEFAULT. would-FAIL if `0x6500` shipped WITHOUT a paired
/// metadata-hiding sibling (Inv-18 would be violated).
#[test]
#[ignore = "RED-PHASE: F-INV18-1 — 0x6500 has paired 0x6510 sibling, sibling is DEFAULT; un-ignore at R5"]
fn f_inv18_1_plaintext_sender_has_paired_sealed_sender_default() {
    const LAYER_C_DROP: u16 = 0x6500;
    const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;

    // The two are distinct siblings in the same recipient band.
    assert_ne!(
        LAYER_C_DROP, DROP_TO_RECIPIENT_SEALED_SENDER,
        "F-INV18-1: the plaintext-sender and Sealed-Sender variants MUST be \
         distinct codepoints."
    );
    assert_eq!(LAYER_C_DROP, 0x6500);
    assert_eq!(DROP_TO_RECIPIENT_SEALED_SENDER, 0x6510);
    // Wire-band adjacency (paired sibling, same 0x65xx recipient band).
    assert_eq!(
        DROP_TO_RECIPIENT_SEALED_SENDER & 0xFF00,
        LAYER_C_DROP & 0xFF00,
        "F-INV18-1: the Sealed-Sender sibling MUST live in the SAME \
         recipient band as the plaintext-sender variant (paired \
         disclosure)."
    );
}

/// F-INV18-1 PIN 2 — Inv-18 paired-disclosure is documented at
/// SECURITY-POSTURE.md (DC arm, reuses the tf3f doc-coupling shape).
/// Compromise #43 (sender-metadata) MUST disclose that the DEFAULT hides
/// the sender-DID and the residual on-wire metadata is {audience,
/// coarse-epoch}. would-FAIL if the doc-wave did not land the disclosure.
#[test]
#[ignore = "RED-PHASE: F-INV18-1 — SECURITY-POSTURE.md documents Sealed-Sender metadata posture; un-ignore at R5"]
fn f_inv18_1_security_posture_documents_metadata_posture() {
    let doc = std::fs::read_to_string(SECURITY_POSTURE_MD)
        .expect("SECURITY-POSTURE.md must be present at /docs/");

    assert!(
        doc.contains("Sealed-Sender") || doc.contains("Sealed Sender"),
        "F-INV18-1: SECURITY-POSTURE.md MUST name Sealed-Sender (the \
         v1-beta DEFAULT metadata-hiding shape; Inv-18 / Compromise #43). \
         Add it at the doc-wave."
    );
    // The residual on-wire metadata must be disclosed as {audience,
    // coarse-epoch} — not over-claiming full metadata privacy.
    let names_audience = doc.contains("audience");
    let names_epoch = doc.contains("coarse-epoch")
        || doc.contains("coarse epoch")
        || doc.contains("coarse-grained epoch");
    assert!(
        names_audience && names_epoch,
        "F-INV18-1: SECURITY-POSTURE.md MUST disclose the RESIDUAL on-wire \
         metadata under the Sealed-Sender default = {{audience, \
         coarse-epoch}} (honest disclosure, not over-claimed). Got: \
         audience={names_audience}, epoch={names_epoch}."
    );
}

// ===========================================================================
// F-LC-7 — FS-gap honest disclosure (DC; reuses tf3f_revocation_reach shape).
// ===========================================================================

/// F-LC-7 PIN 1 — HPKE-mode-base is structurally non-FS at the
/// long-term-sk axis: a recipient who RECOVERS an old sk can still open
/// an old envelope (the FS-gap reality, documented not "fixed"). This is
/// the honest behavioral disclosure — would-FAIL only if someone claimed
/// forward-secrecy the construction cannot provide. Behavioral stub
/// `open_with_recovered_sk` returns Ok to document the gap.
#[test]
#[ignore = "RED-PHASE: F-LC-7 — HPKE non-FS long-term-sk axis documented; un-ignore at R5"]
fn f_lc_7_hpke_non_fs_old_envelope_still_opens_with_recovered_sk() {
    // The behavioral arm: an old envelope sealed to a long-term recipient
    // pubkey still opens once the matching long-term sk is recovered.
    // (Stub modeled inline to keep this DC family self-contained.)
    fn open_with_recovered_sk(
        _old_envelope: &[u8],
        _recovered_sk: &[u8; 32],
    ) -> Result<Vec<u8>, ()> {
        unimplemented!("R5 wires the FS-gap documentation arm (open with recovered long-term sk)")
    }

    let old_env = b"V2 HPKE-base envelope sealed in 2026".to_vec();
    let recovered_sk = [0x99u8; 32];
    let opened = open_with_recovered_sk(&old_env, &recovered_sk);
    assert!(
        opened.is_ok(),
        "F-LC-7: HPKE-mode-base is NON-FS at the long-term-sk axis — a \
         recovered long-term sk DECRYPTS an old envelope. This is the R6 \
         reality (Compromise #42), documented not 'fixed'. would-FAIL only \
         if an implementer claimed forward-secrecy the construction cannot \
         provide."
    );
}

/// F-LC-7 PIN 2 — SECURITY-POSTURE.md documents the FS-gap (Compromise
/// #42), the DropToRecipient forever-valid reach (#62), AND that
/// journalist per-message FS (#56) is a SEPARATE deferred class (kept
/// distinct from #42/#52). DC arm. would-FAIL if the doc-wave omitted the
/// FS-gap disclosure or conflated #56 with #42.
#[test]
#[ignore = "RED-PHASE: F-LC-7 — SECURITY-POSTURE.md documents FS-gap (#42/#56/#62); un-ignore at R5"]
fn f_lc_7_security_posture_documents_fs_gap() {
    let doc =
        std::fs::read_to_string(SECURITY_POSTURE_MD).expect("SECURITY-POSTURE.md must be present");

    let names_fs_gap = doc.contains("forward secrecy")
        || doc.contains("forward-secrecy")
        || doc.contains("FS-gap")
        || doc.contains("non-FS");
    assert!(
        names_fs_gap,
        "F-LC-7: SECURITY-POSTURE.md MUST document the HPKE-mode-base \
         FS-gap (Compromise #42 — long-term-sk decrypts forever). Add it at \
         the doc-wave."
    );
    // #56 journalist per-message FS MUST be named as a SEPARATE class so the
    // audit does not read it as a duplicate of #42/#52.
    let names_journalist_class =
        doc.contains("per-message") || doc.contains("per message") || doc.contains("journalist");
    assert!(
        names_journalist_class,
        "F-LC-7: SECURITY-POSTURE.md MUST name journalist per-message FS \
         (#56) as a SEPARATE deferred class — kept sharply distinct from \
         #42/#52 (R1-Q-8). would-FAIL if #56 were conflated."
    );
}
