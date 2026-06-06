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
//!     (`0x6510` DEFAULT); residual privacy-metadata on-wire under the
//!     default = exactly `{audience}`; `0x6500` discloses sender-DID-in-AAD
//!     (U4).
//!
//! Pin sources — spec of record is now **R0.7**
//! (`111cca9c:.addl/phase-4-meta/f-full-r0-plan.md`; the §-numbers below are
//! stable across the R0.5→R0.6→R0.7 revisions; R0.5 was `e4fbfe73`):
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
//! # R4-FIX (F4-028, F4-029) — added substantive byte-pins.
//! # R4.4-FIX (F4-004/005) — dedicated `aad_version: u8` byte-0.
//! # R4.4-FIX (F4-006 / CLUSTER-1 BLOCKER) — coarse_epoch REMOVED + 0x6510
//! #   envelope-AAD reconciled to ONE canonical field-set across siblings.
//! # R4.5-MIGRATE (R0.6 Sealed-Sender AAD freeze) — `body_cid` is now a
//! #   self-describing CIDv1 (`0x01 0x71 0x1e 0x20 || 32-byte BLAKE3` = 36
//! #   bytes) on the `0x6510` band, NOT a bare fixed-32 digest (R0.6 BR;
//! #   CLAUDE.md baked-in #5; restores U3 length-injectivity). Both
//! #   `0x6510` goldens (`F_INV18_1_SEALED_AAD_HEX` + `F_LC_8_TOKEN_AAD_HEX`)
//! #   are regenerated over the SAME 32-byte digest with the 4-byte multihash
//! #   framing prepended (M-20). `aad_version == 0x01` and the no-coarse_epoch
//! #   posture (item 4) are confirmed unchanged.
//!
//! ## CLUSTER-1 (BLOCKER) — the `0x6510` envelope-AAD is ONE field-set, ONE
//! ## golden, byte-0 guard mirrored everywhere (F4-006 / Ben-RULING-#1)
//!
//! The previous revision of THIS file froze the DEFAULT (`0x6510`) Sealed-
//! Sender envelope AAD as `{aad_version, codepoint, audience, coarse_epoch}`
//! (golden `…001d0100`) — while the sibling Layer-C `f_lc_hpke`
//! (`DropSealedSender` binding) froze `{aad_version, codepoint, body_cid,
//! recipient_key_generation}` with **NO** `coarse_epoch` (F4-006). Two
//! sibling files freezing **divergent AADs for the same `0x6510` codepoint**
//! is the BLOCKER class: a relay/recipient on one engine could not AEAD-open
//! a drop sealed by the other. The reconciliation (one canonical field-set):
//!   - **`coarse_epoch` is REMOVED from the `0x6510` envelope AAD** (and from
//!     the token-binding AAD — see below). This is decided by the spec, not a
//!     wire-byte fork: **M-14** (R0.7 §3.10/§4.1 + §1570) "DropToRecipient
//!     carries **NEITHER**"; the **§4.1 FREEZE row** "coarse 1-hour bucket
//!     (U28) … DropToRecipient carries NEITHER"; **Ben-RULING-#1**
//!     (2026-06-03) "coarse_epoch is NOT on the Drop wire (§4.1/M-14)";
//!     **F4-006** (already applied in `f_lc_hpke`). The §3.3:484 / Compromise
//!     #43:1043 "audience + coarse-epoch" prose is **pre-F4-006 residue** that
//!     the spec ITSELF is corrected to drop via the tracked-doc cascade (NOT a
//!     corpus concern); the old "`coarse_epoch` STAYS" rationale that leaned on
//!     that prose is RETIRED.
//!   - **The canonical `0x6510` envelope-AAD field-set is the union both
//!     siblings agree on:** `{aad_version, codepoint, audience, body_cid,
//!     recipient_key_generation}` — the safe superset (under-binding is
//!     impossible). `aad_version` (§4.1:883 prefix) + `codepoint` (§4.1:883
//!     binding) are FRAMING; `audience` is the recipient binding (§3.3:484);
//!     `body_cid` + `recipient_key_generation` are the DUAL-CID + key-
//!     generation bindings (§4.1:886/887, U18/U19, Inv-16). The sender-DID is
//!     ABSENT (Sealed-Sender — bound INSIDE the ciphertext).
//!   - **Residual privacy-metadata ≠ AAD field-set (distinction made
//!     explicit).** Inv-18 / #43's *residual privacy-metadata* (the
//!     identifiers a relay can observe) under the default is exactly
//!     `{audience}` (was `{audience, coarse-epoch}`; coarse-epoch removed).
//!     `aad_version`/`codepoint`/`body_cid`/`recipient_key_generation` are
//!     framing/binding fields, not sender-metadata in the privacy sense. The
//!     doc-coupling arm asserts the SECURITY-POSTURE disclosure says the
//!     residual is `{audience}` and does NOT over- or under-claim.
//!   - **byte-0 anti-conflation guard mirrored everywhere.** Both frozen
//!     goldens lead with the dedicated `AAD_VERSION = 0x01` prefix (NOT the
//!     `ENVELOPE_FORMAT_VERSION = 2` serialization byte); the anti-conflation
//!     pin fails any revert to the format byte (F4-004/005; load-bearing, not
//!     advisory).
//!
//! **F4-004/005 (MAJOR).** Spec R0.7 §4.1 freezes a dedicated
//! `aad_version: u8` AAD prefix DISTINCT from `ENVELOPE_FORMAT_VERSION_V2`
//! (the envelope SERIALIZATION-format byte). `const AAD_VERSION: u8 = 0x01`
//! (mirroring the sibling Layer-C `f_lc_hpke` + the MembershipSet `f_aad_2`
//! convention) is pushed as AAD byte-0 in BOTH the token-binding AAD and the
//! `0x6510` Sealed-Sender AAD; `ENVELOPE_FORMAT_VERSION = 2` stays strictly
//! for the envelope-format axis.
//!
//! **F4-028 (F-LC-8 token-binding AAD byte-pin).** §3.11 calls the
//! token-binding AAD "wire-affecting only in the token-binding AAD (a
//! Sealed-Sender sub-field), so it MUST land pre-freeze". The token-binding
//! AAD is the receive-boundary admission binding that rides the SAME drop
//! wire as the `0x6510` envelope; it therefore freezes the SAME canonical
//! envelope-AAD prefix `{aad_version, codepoint, audience, body_cid,
//! recipient_key_generation}` PLUS the recipient-issued token's own validity
//! window `{token_nbf, token_exp, token_rate_limit}` (the UCAN nbf/exp that
//! §3.11 names as the freshness/revocation mechanism). **No separate
//! `coarse_epoch`** — the token's own nbf/exp IS the freshness binding, and a
//! coarse bucket would re-introduce exactly the drop-wire metadata Ben-RULING
//! -#1 + M-14 eliminate. ADDED:
//!   (a) the stub SERIALIZES a canonical BIG-ENDIAN token-binding AAD
//!       (`serialize_token_binding_aad`) over the union prefix + token window
//!       + an admission path that binds the token to that AAD
//!       (`admit_sealed_sender_bound`);
//!   (b) a FROZEN golden-hex (`F_LC_8_TOKEN_AAD_HEX`) the BE layout MUST
//!       reproduce (would-FAIL on any field-order / endianness drift — R5
//!       confirms-or-deliberately-updates against the real serializer, M-20);
//!   (c) a MUTATE-the-AAD → fail-admit arm (flip one byte of the bound AAD ⇒
//!       admission rejects), proving the token binding is load-bearing.
//!
//! **F4-029 (F-INV18-1 positive field-set enumeration).** A POSITIVE
//! field-set enumeration: serialize the `0x6510` envelope AAD, enumerate its
//! fields, assert the set is EXACTLY `{aad_version, codepoint, audience,
//! body_cid, recipient_key_generation}` with the sender-DID (and any extra
//! field) ABSENT — plus a FROZEN golden-hex (`F_INV18_1_SEALED_AAD_HEX`). A
//! SECOND assertion narrows the *residual privacy-metadata* subset to exactly
//! `{audience}` (the Inv-18 / #43 claim), keeping the engineering field-set
//! freeze distinct from the privacy-disclosure claim.
//!
//! [FLAG-FOR-BEN — courtesy cross-check, not a halt: I removed `coarse_epoch`
//!  from the token-binding AAD too (not only the `0x6510` envelope AAD). The
//!  prior fixer note recommended keeping it on the token-binding sub-field.
//!  My reasoning as Ben: the token-binding AAD rides the drop wire (§3.11:814
//!  "wire-affecting only in the token-binding AAD"), and Ben-RULING-#1's plain
//!  text is "coarse_epoch is NOT on the Drop wire"; the token's own UCAN
//!  nbf/exp (which §3.11 names as the freshness/revocation mechanism) already
//!  binds freshness, so a separate coarse bucket is both redundant and a
//!  re-introduction of the exact metadata M-14 removes. One coherent posture =
//!  "no coarse_epoch anywhere on the drop wire." If you prefer the token-
//!  binding sub-field to retain a coarse bucket, re-add `coarse_epoch: u64`
//!  to `TokenBindingAad` only (after `recipient_key_generation`) + regenerate
//!  `F_LC_8_TOKEN_AAD_HEX`; the envelope AAD removal is settled regardless.]
//!
//! # RED-PHASE STATUS + STUB-SHIM (pim-12 §3.6e).
//!
//! Self-contained stubs (`abuse_stub`, `group_posture_stub`, `sealed_aad_stub`)
//! so the file is parallel-safe + compiles green at baseline behind
//! `#[ignore]`. The FS-gap + Inv-18 disclosure arms are DOC-COUPLING (real
//! `std::fs::read_to_string` against `docs/SECURITY-POSTURE.md`, reusing the
//! `tf3f_revocation_reach_*` shape) — those assert the doc-wave landed the
//! disclosure text. The NEW byte-pin arms drive the stub's DETERMINISTIC
//! canonical serializer (not `unimplemented!()`) so the frozen golden-hex is
//! computable green at red-phase; R5 swaps the stub serializer for the real
//! one and confirms (or deliberately updates) the frozen literal. R5:
//!   1. DELETE the stub modules; INSERT the real `use benten_drop::…`;
//!   2. UN-IGNORE; 3. the doc-coupling arms stay (doc-wave authored);
//!   4. confirm-or-update the frozen golden-hex against the real encoder.
//!
//! # Wave-0 (M-20) + would-FAIL (pim-2 + pim-18 + §3.6f-ext). Behavioral
//! stubs `unimplemented!()` so a forgotten stub at R5 PANICS; doc arms
//! assert a doc string that did NOT exist pre-doc-wave; the byte-pin arms
//! freeze ABSOLUTE BE bytes (drift flips the pin).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_variables)]

// ===========================================================================
// R5 — real production surface (`benten_drop::layer_c`). The self-contained
// stubs are DELETED; the production abuse-control / group-posture / sealed-AAD
// modules are aliased to the former stub names so the fixtures' qualified
// const paths (`abuse_stub::AAD_VERSION`, etc.) keep resolving unchanged.
// ===========================================================================

use benten_drop::layer_c::abuse_control as abuse_stub;
use benten_drop::layer_c::group_posture as group_posture_stub;
use benten_drop::layer_c::sealed_aad as sealed_aad_stub;

use abuse_stub::{
    AdmitError, DeliveryToken, TokenBindingAad, admit_sealed_sender, admit_sealed_sender_bound,
    decrypt_was_attempted_for_last_admit, serialize_token_binding_aad,
};
use group_posture_stub::{
    GroupError, LAYER_C_DROP_MULTI_RECIPIENT, MEMBERSHIP_SET_GROUP_MULTI_STANZA, dispatch_group,
    open_membership_set_group, seal_membership_set_group,
};
use sealed_aad_stub::{
    SealedSenderAad, aad_field_set, residual_privacy_metadata, serialize_sealed_sender_aad,
};

const SECURITY_POSTURE_MD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/SECURITY-POSTURE.md"
);

fn did(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Lowercase-hex of a byte slice (test-local; no external dep).
fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
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

// --- R4-FIX F4-028 — token-binding AAD byte-layout pin + mutate→fail-admit.

/// The fixture body-CID for the canonical `0x6510` envelope-AAD prefix.
///
/// R4.5-MIGRATE (R0.6 BR): a self-describing CIDv1 — `0x01 0x71 0x1e 0x20`
/// (CIDv1 / dag-cbor / blake3 / 32-byte digest length) || the SAME 32-byte
/// digest the corpus froze (`[0xE0, 0; 31]`). The digest payload is held
/// stable across the migration so the only golden delta is the prepended
/// 4-byte multihash framing (NOT a bare fixed-32 digest; CLAUDE.md baked-in
/// #5 — never hardcode a hash-width into a frozen wire). 36 bytes total.
fn fixture_body_cid() -> Vec<u8> {
    let mut digest = [0u8; 32];
    digest[0] = 0xE0;
    let mut cid = vec![0x01u8, 0x71, 0x1e, 0x20];
    cid.extend_from_slice(&digest);
    cid
}

/// The canonical token-binding AAD fixture (F4-028). All integers BE. The
/// prefix is the SAME canonical `0x6510` envelope union the sibling
/// `f_lc_hpke` freezes; NO coarse_epoch (CLUSTER-1 / F4-006).
fn f_lc_8_token_aad_fixture() -> TokenBindingAad {
    TokenBindingAad {
        aad_version: abuse_stub::AAD_VERSION, // 0x01 (R4.4-FIX F4-004/005: dedicated AAD prefix, NOT format ver 0x02)
        codepoint: abuse_stub::DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        audience_did: did("did:key:zRecipientAudienceUNIQUE"),
        body_cid: fixture_body_cid(),
        recipient_key_generation: 0,
        token_not_before: 1_900_000,
        token_expires_at: 1_999_999,
        token_rate_limit: 5,
    }
}

/// FROZEN big-endian golden vector for the token-binding AAD (F4-028).
/// Computed once from the canonical BE layout (the `0x6510` envelope union
/// `{aad_version, codepoint, audience(u32-BE length-prefixed), body_cid,
/// recipient_key_generation}` PLUS the token window `{nbf, exp, rate_limit}`;
/// NO coarse_epoch). **R4.6-FIX F-LC-AUD-U32:** the `audience` length-prefix is
/// `u32-BE` (`00000020`) per R0.7 header:33 / §3.3:539 / §4.1:1040 — corrected
/// from the prior u16 (`0020`) slip; golden grew +2 bytes. ANY field-order or
/// endianness drift in the real serializer flips this pin. R5
/// confirms-or-deliberately-updates this frozen literal against the real
/// encoder (M-20).
const F_LC_8_TOKEN_AAD_HEX: &str = "016510000000206469643a6b65793a7a526563697069656e7441756469656e6365554e4951554501711e20e0000000000000000000000000000000000000000000000000000000000000000000000000000000001cfde000000000001e847f00000005";

/// F-LC-8 PIN 5 (R4-FIX F4-028) — the token-binding AAD serializes to the
/// FROZEN big-endian byte layout. This pins the wire-affecting sub-field
/// (§3.11) that BR-1 says "MUST land pre-freeze". would-FAIL if the
/// serializer emitted LE codepoint bytes, re-ordered the fields, re-added a
/// coarse_epoch, or changed the length-prefix encoding — i.e. any silent
/// wire drift.
#[test]
fn f_lc_8_token_binding_aad_frozen_be_byte_layout() {
    let aad = f_lc_8_token_aad_fixture();
    let bytes = serialize_token_binding_aad(&aad);

    assert_eq!(
        to_hex(&bytes),
        F_LC_8_TOKEN_AAD_HEX,
        "F-LC-8 (F4-028): the token-binding AAD MUST serialize to the \
         FROZEN canonical big-endian layout (the `0x6510` envelope union + \
         token window; NO coarse_epoch). A mismatch means the wire-affecting \
         sub-field drifted (endianness / field order / a re-added coarse \
         bucket / length-prefix) — exactly the pre-freeze hazard §3.11/BR-1 \
         names. R5 confirms-or-deliberately-updates this literal (M-20)."
    );

    // Anti-tautology cross-check: the codepoint MUST appear as the BE pair
    // 0x65 0x10 at offset 1 (NOT the LE 0x10 0x65). would-FAIL if a future
    // edit reverted aead.rs LE on this path (M-19).
    assert_eq!(
        &bytes[1..3],
        &[0x65, 0x10],
        "F-LC-8 (F4-028): codepoint 0x6510 MUST be big-endian (0x65,0x10) \
         in the token-binding AAD, never little-endian (0x10,0x65)."
    );

    // R4.6-FIX F-LC-AUD-U32: the token-binding AAD reuses the canonical 0x6510
    // envelope union prefix, so its `audience` length-prefix is ALSO u32-BE
    // (`00 00 00 20` at offset 3 for the 32-byte fixture DID) per R0.7
    // §4.1:1040. would-FAIL on a u16 regression (the F-LC-AUD-U32 slip).
    assert_eq!(
        &bytes[3..7],
        &(aad.audience_did.len() as u32).to_be_bytes(),
        "F-LC-8 (R4.6 / F-LC-AUD-U32): the token-binding AAD audience \
         length-prefix MUST be u32-BE ({:?}), NOT u16 — it reuses the 0x6510 \
         envelope union prefix (R0.7 §4.1:1040).",
        (aad.audience_did.len() as u32).to_be_bytes()
    );

    // R4.4-FIX F4-004/005 anti-conflation pin — AAD byte-0 is the dedicated
    // `aad_version` (= 0x01), NOT the envelope serialization `format_version`
    // (= 0x02). These are TWO orthogonal version axes (R0.7 §4.1); freezing the
    // format byte here would conflict with the sibling Layer-C + MembershipSet
    // golden (`0x01`) and break cross-engine AEAD-open. would-FAIL if a future
    // edit reverts AAD byte-0 to the format version.
    assert_eq!(
        bytes[0],
        abuse_stub::AAD_VERSION,
        "F-LC-8 (F4-004/005): the token-binding AAD byte-0 MUST be the dedicated \
         AAD_VERSION (0x01), NOT the envelope format version."
    );
    assert_ne!(
        bytes[0],
        abuse_stub::ENVELOPE_FORMAT_VERSION,
        "F-LC-8 (F4-004/005): the AAD version axis and the envelope \
         serialization-format axis are DISTINCT — byte-0 MUST NOT be the format \
         version (0x02). Reverting this re-introduces the cross-engine \
         AEAD-open break."
    );
}

/// F-LC-8 PIN 6 (R4-FIX F4-028) — MUTATE the bound token-binding AAD ⇒
/// admission REJECTS at `TokenBindingMismatch`. The on-wire AAD is
/// load-bearing: a relay that flips ANY byte of the bound AAD (re-target
/// the audience, swap the body-CID, downgrade the rate-limit) breaks the
/// binding and the envelope is refused BEFORE decrypt. would-FAIL if the
/// admission path ignored the AAD bytes (treated the token binding as
/// advisory).
#[test]
fn f_lc_8_mutated_token_binding_aad_fails_admit() {
    let aad = f_lc_8_token_aad_fixture();
    let token = DeliveryToken {
        not_before: aad.token_not_before,
        expires_at: aad.token_expires_at,
        rate_limit: aad.token_rate_limit,
    };
    let now = 1_900_800;

    // Positive control: the UNMODIFIED bound AAD is admitted.
    let good_bytes = serialize_token_binding_aad(&aad);
    let ok = admit_sealed_sender_bound(&token, &aad, &good_bytes, now, 1);
    assert!(
        ok.is_ok(),
        "F-LC-8 (F4-028): the UNMODIFIED bound token-binding AAD MUST be \
         admitted (positive control; proves the mismatch arm is not \
         vacuous). Got: {ok:?}"
    );

    // Adversary flips one byte of the bound AAD (e.g. inside the audience
    // DID — re-targeting the token to a different recipient).
    let mut tampered = good_bytes.clone();
    let flip_at = 6; // inside the audience-DID region (after the 5-byte header)
    tampered[flip_at] ^= 0xFF;
    assert_ne!(
        tampered, good_bytes,
        "sanity: the tamper must actually change the bytes"
    );

    let outcome = admit_sealed_sender_bound(&token, &aad, &tampered, now, 1);
    assert!(
        matches!(outcome, Err(AdmitError::TokenBindingMismatch)),
        "F-LC-8 (F4-028): a Sealed-Sender envelope whose bound \
         token-binding AAD was MUTATED MUST be refused at \
         `TokenBindingMismatch` (the admission path recomputes the \
         canonical BE AAD and requires byte-equality). would-FAIL if the \
         AAD bytes were ignored — then a relay could re-target/replay the \
         token freely. Got: {outcome:?}"
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
/// the sender-DID and that the residual on-wire privacy-metadata is exactly
/// `{audience}` (coarse-epoch is NOT on the drop wire — Ben-RULING-#1 /
/// M-14; the spec #43 prose is corrected to drop it via the tracked-doc
/// cascade). would-FAIL if the doc-wave did not land the disclosure, or
/// over-claims a coarse-epoch residual on the drop wire.
#[test]
fn f_inv18_1_security_posture_documents_metadata_posture() {
    let doc = std::fs::read_to_string(SECURITY_POSTURE_MD)
        .expect("SECURITY-POSTURE.md must be present at /docs/");

    assert!(
        doc.contains("Sealed-Sender") || doc.contains("Sealed Sender"),
        "F-INV18-1: SECURITY-POSTURE.md MUST name Sealed-Sender (the \
         v1-beta DEFAULT metadata-hiding shape; Inv-18 / Compromise #43). \
         Add it at the doc-wave."
    );
    // The residual on-wire privacy-metadata under the default must be
    // disclosed as the audience binding — the honest, minimal claim. The
    // coarse-epoch is NOT a drop-wire residual (Ben-RULING-#1 / M-14); the
    // doc-wave writes the corrected #43 residual.
    assert!(
        doc.contains("audience"),
        "F-INV18-1: SECURITY-POSTURE.md MUST disclose the RESIDUAL on-wire \
         privacy-metadata under the Sealed-Sender default = {{audience}} \
         (honest disclosure; the sender-DID is hidden, the audience binding \
         remains). Add it at the doc-wave."
    );
}

// --- R4-FIX F4-029 — POSITIVE field-set enumeration of the 0x6510 AAD.

/// The canonical Sealed-Sender (`0x6510`) on-wire envelope AAD fixture
/// (F4-029). CLUSTER-1 / F4-006: the union field-set, NO coarse_epoch.
fn f_inv18_1_sealed_aad_fixture() -> SealedSenderAad {
    SealedSenderAad {
        aad_version: sealed_aad_stub::AAD_VERSION, // 0x01 (R4.4-FIX F4-004/005: dedicated AAD prefix, NOT format ver 0x02)
        codepoint: sealed_aad_stub::DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510
        audience_did: did("did:key:zRecipientAudienceUNIQUE"),
        body_cid: fixture_body_cid(),
        recipient_key_generation: 0,
    }
}

/// FROZEN big-endian golden vector for the DEFAULT (`0x6510`) on-wire
/// envelope AAD (F4-029). EXACTLY the canonical union
/// `{aad_version, codepoint, audience(u32-BE length-prefixed), body_cid,
/// recipient_key_generation}` — no sender-DID region, NO coarse_epoch.
/// **R4.6-FIX F-LC-AUD-U32:** the `audience` length-prefix is `u32-BE`
/// (`00000020` for the 32-byte fixture DID) per R0.7 header:33 / §3.3:539 /
/// §4.1:1040 — corrected from the prior u16 (`0020`) slip; stays BYTE-IDENTICAL
/// to the sibling `f_lc_hpke::F_LC_SEALED_SENDER_AAD_HEX` (both migrate in
/// lockstep). R5 confirms-or-deliberately-updates this frozen literal against
/// the real encoder (M-20).
const F_INV18_1_SEALED_AAD_HEX: &str = "016510000000206469643a6b65793a7a526563697069656e7441756469656e6365554e4951554501711e20e00000000000000000000000000000000000000000000000000000000000000000000000";

/// F-INV18-1 PIN 3 (R4-FIX F4-029) — POSITIVE field-set enumeration: the
/// serialized `0x6510` envelope AAD field-set is EXACTLY the canonical union
/// and the sender-DID is ABSENT; SEPARATELY, the residual privacy-metadata
/// subset is EXACTLY `{audience}`. This upgrades the residual-metadata claim
/// from a doc-grep to a behavioral enumeration: an impl that smuggles a
/// `sender_did`, re-adds a `coarse_epoch`, or otherwise changes the field-set
/// is caught HERE by an unexpected/missing field token. would-FAIL if the
/// field-set gains or drops a field, if the privacy-residual gains coarse_epoch
/// back, or if the serialized bytes contained the sender-DID.
#[test]
fn f_inv18_1_sealed_sender_aad_field_set_is_exactly_the_canonical_union() {
    use std::collections::BTreeSet;

    // (a) POSITIVE enumeration — the engineering field-set is EXACTLY the
    //     canonical `0x6510` envelope union both siblings freeze.
    let fields: BTreeSet<&str> = aad_field_set().into_iter().collect();
    let expected: BTreeSet<&str> = BTreeSet::from([
        "aad_version",
        "codepoint",
        "audience",
        "body_cid",
        "recipient_key_generation",
    ]);
    assert_eq!(
        fields, expected,
        "F-INV18-1 (F4-029 / CLUSTER-1): the DEFAULT (0x6510) envelope AAD \
         field-set MUST be EXACTLY the canonical union \
         {{aad_version, codepoint, audience, body_cid, \
         recipient_key_generation}}. An ADDED token (e.g. `sender_did` or a \
         re-added `coarse_epoch`) means the default leaked metadata or \
         diverged from the sibling `f_lc_hpke`; a DROPPED token means a \
         binding regressed. Got: {fields:?}"
    );
    // The sender-DID MUST NOT be a field of the default AAD (the whole
    // point of Sealed-Sender — it lives INSIDE the ciphertext); and the
    // coarse_epoch MUST be gone (Ben-RULING-#1 / M-14).
    assert!(
        !fields.contains("sender_did") && !fields.contains("sender"),
        "F-INV18-1 (F4-029): the Sealed-Sender DEFAULT AAD MUST NOT carry a \
         sender-DID field — it is bound INSIDE the ciphertext."
    );
    assert!(
        !fields.contains("coarse_epoch") && !fields.contains("coarse-epoch"),
        "F-INV18-1 (CLUSTER-1 / F4-006): the DEFAULT (0x6510) envelope AAD \
         MUST NOT carry a coarse_epoch — DropToRecipient carries NEITHER \
         timestamp NOR coarse bucket (M-14 / Ben-RULING-#1). A re-added \
         coarse_epoch re-introduces the cross-sibling divergence + the \
         drop-wire metadata M-14 removes."
    );

    // (b) RESIDUAL privacy-metadata — the identifiers a relay observes that
    //     are sender/recipient-metadata in the Inv-18/#43 privacy sense =
    //     EXACTLY {audience} (framing/binding fields are not privacy
    //     metadata; coarse-epoch removed).
    let residual: BTreeSet<&str> = residual_privacy_metadata().into_iter().collect();
    let residual_expected: BTreeSet<&str> = BTreeSet::from(["audience"]);
    assert_eq!(
        residual, residual_expected,
        "F-INV18-1 (CLUSTER-1 / Inv-18): the residual on-wire PRIVACY-metadata \
         under the Sealed-Sender default MUST be EXACTLY {{audience}} — the \
         sender-DID is hidden (Sealed-Sender) and the coarse-epoch is removed \
         (Ben-RULING-#1 / M-14). would-FAIL if the residual gained coarse_epoch \
         back. Got: {residual:?}"
    );

    // (c) FROZEN byte layout — the serialized AAD reproduces the BE golden
    //     vector (drift flips the pin).
    let aad = f_inv18_1_sealed_aad_fixture();
    let bytes = serialize_sealed_sender_aad(&aad);
    assert_eq!(
        to_hex(&bytes),
        F_INV18_1_SEALED_AAD_HEX,
        "F-INV18-1 (F4-029): the 0x6510 envelope AAD MUST serialize to the \
         FROZEN big-endian layout containing ONLY the canonical union \
         {{aad_version, codepoint, audience(u32-BE length-prefixed), body_cid, \
         recipient_key_generation}}. R5 confirms-or-deliberately-updates this \
         literal (M-20)."
    );

    // R4.6-FIX F-LC-AUD-U32: the `audience` length-prefix MUST be u32-BE
    // (R0.7 header:33 / §3.3:539 / §4.1:1040). For the 32-byte fixture DID
    // that is `00 00 00 20` at offset 3 (after aad_version[0] + codepoint[1..3]).
    // would-FAIL on a u16 (`00 20`) regression — the prior settled-territory
    // slip that conflated this variable-field lp with the 0x6520 band's
    // recipient_count cardinality. Mirrors the sibling f_lc_hpke F-LC-3 guard.
    assert_eq!(
        &bytes[3..7],
        &(aad.audience_did.len() as u32).to_be_bytes(),
        "F-INV18-1 (R4.6 / F-LC-AUD-U32): the 0x6510 audience length-prefix \
         MUST be u32-BE ({:?} for the {}-byte audience DID), NOT u16. R0.7 \
         §4.1:1040 freezes the audience at `u32-BE-length-prefixed`. would-FAIL \
         on a u16 regression (the F-LC-AUD-U32 wire-byte slip).",
        (aad.audience_did.len() as u32).to_be_bytes(),
        aad.audience_did.len()
    );

    // R4.4-FIX F4-004/005 anti-conflation pin — the DEFAULT AAD byte-0 is the
    // dedicated `aad_version` (= 0x01), NOT the envelope `format_version`
    // (= 0x02). Distinct version axes (R0.7 §4.1); reconciles to the sibling
    // Layer-C + MembershipSet golden's `0x01`. would-FAIL on a revert.
    assert_eq!(
        bytes[0],
        sealed_aad_stub::AAD_VERSION,
        "F-INV18-1 (F4-004/005): the DEFAULT (0x6510) AAD byte-0 MUST be the \
         dedicated AAD_VERSION (0x01), NOT the envelope format version (0x02)."
    );
    assert_ne!(
        bytes[0],
        sealed_aad_stub::ENVELOPE_FORMAT_VERSION,
        "F-INV18-1 (F4-004/005): byte-0 MUST NOT be the envelope \
         serialization-format version — that conflation broke cross-engine \
         AEAD-open (the F4-004/005 hazard)."
    );

    // (d) NEGATIVE byte-scan — the sender-DID is provably ABSENT from the
    //     serialized DEFAULT AAD (would-FAIL if it leaked in).
    let sender_did = did("did:key:zSenderAliceUNIQUEMARKER");
    let leaks = bytes
        .windows(sender_did.len())
        .any(|w| w == sender_did.as_slice());
    assert!(
        !leaks,
        "F-INV18-1 (F4-029): the serialized DEFAULT (0x6510) AAD MUST NOT \
         contain the sender-DID byte sequence — Sealed-Sender binds it \
         INSIDE the ciphertext, never the on-wire AAD."
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
fn f_lc_7_hpke_non_fs_old_envelope_still_opens_with_recovered_sk() {
    // The behavioral arm: an old envelope sealed to a long-term recipient
    // pubkey still opens once the matching long-term sk is recovered — the
    // FS-gap reality (Compromise #42). Exercises the REAL Layer-C path: seal
    // to a long-term recipient pubkey, then open with the matching long-term
    // sk (the "recovered" key). The open SUCCEEDS — that IS the non-FS gap.
    use benten_drop::layer_c::{open_single, seal_sealed_sender};
    fn open_with_recovered_sk(
        _old_envelope_marker: &[u8],
        recovered_sk: &[u8; 32],
    ) -> Result<Vec<u8>, ()> {
        // The recovered long-term sk corresponds to a long-term pubkey
        // fingerprint (pk = sk - 0x80 per byte, matching the fixture pairing).
        let mut pk = [0u8; 32];
        for (i, b) in recovered_sk.iter().enumerate() {
            pk[i] = b.wrapping_sub(0x80);
        }
        let env = seal_sealed_sender(
            &pk,
            &b"did:key:zLongTermRecipient".to_vec(),
            &b"did:key:zSender".to_vec(),
            &[0xF5u8; 32],
            0,
            b"old 2026 content",
        );
        // A 2030 recovery of the SAME long-term sk decrypts the 2026 envelope.
        open_single(recovered_sk, &env)
            .map(|(pt, _sender)| pt)
            .map_err(|_| ())
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
