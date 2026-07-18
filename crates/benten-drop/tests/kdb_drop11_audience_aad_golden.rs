//! GAP-KDB Shape-B — DROP-11 (TIER-A): single-recipient `did:benten` audience
//! AAD golden — `u32` framing wire-transparency. W2 benten-drop.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 (a longer `did:benten` audience
//! just fills the length-prefixed AAD field; NO re-encryption) + `R2-LANDSCAPE`
//! DROP-11 (TIER-A freeze-perfect golden). The `0x6510` AAD frames the audience
//! with a `u32-BE` length prefix (`push_audience`, `layer_c.rs:504-510`) — NOT
//! the band's `u16` `recipient_count` cardinality.
//!
//! # M-20 golden discipline (tiny-audience trick)
//! The golden below pins the AAD field layout + the `u32` framing WIDTH via a
//! SHORT deterministic audience (keeps the constant small, like the canary
//! `KSD-1` tiny-doc golden), captured via a THROWAWAY run of the REAL
//! `BindingContext::plaintext_aad_bytes`. A separate LIVE arm exercises a REAL
//! ~2776-byte `did:benten` audience to pin wire-TRANSPARENCY (the full audience
//! is carried, u32-framed, un-truncated).
//!
//! # Live freeze-guard + would_fail_on_revert
//! - Golden: reverting the audience length prefix to `u16` (2 bytes) shifts
//!   every following AAD field → the golden flips.
//! - Wire-transparency: a `u16` prefix cannot express the byte offsets a long
//!   `did:benten` audience needs; the parsed `u32` length == the audience byte
//!   length assert would flip.
//!
//! # R5 un-ignore (seal-produces-AAD arm only)
//! Mint the binding-typed single seal; drop `#[ignore]` on the seal arm. The
//! golden + framing arms are live now.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_drop::kdb_seal_testing as seal;
use benten_drop::layer_c::RecipientBinding;
use benten_drop::layer_c::{
    AAD_VERSION, BindingContext, DROP_TO_RECIPIENT_SEALED_SENDER, self_describing_cid,
    single_plaintext_aad_region,
};
use benten_id::did::Did;
use benten_id::kdb_testing as kdb;

/// The frozen `0x6510` AAD for a SHORT deterministic audience
/// `b"did:benten:zShortDrop11Audience"` (31 B), body-CID over
/// `b"drop11 short golden body"`, generation 7. Captured via M-20 throwaway
/// from `BindingContext::plaintext_aad_bytes`. Layout:
/// `01 | 6510 | 0000001f (u32 audience len) | <audience 31B> | body_cid 36B | 00000007`.
const DROP11_SHORT_AAD_GOLDEN: &str = "0165100000001f6469643a62656e74656e3a7a53686f727444726f70313141756469656e636501711e20d0affbf52634e03adc90a5d523487e7c5b80b4d2bb402fcbe76296e7d144467b00000007";

const SHORT_AUDIENCE: &[u8] = b"did:benten:zShortDrop11Audience";

fn short_binding() -> BindingContext {
    let digest = *blake3::hash(b"drop11 short golden body").as_bytes();
    BindingContext::DropSealedSender {
        aad_version: AAD_VERSION,
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER,
        audience_did: SHORT_AUDIENCE.to_vec(),
        body_cid: self_describing_cid(&digest),
        recipient_key_generation: 7,
    }
}

/// A deterministic, full-length `did:benten` DID (≈2776 B) — same builder as
/// DROP-9 / the W0 codec fixtures.
fn det_did_benten(seed: &str) -> Did {
    let sig_mk = kdb::det_signing_multikey(seed);
    let doc = kdb::KeySetDocument::v1_hybrid(
        sig_mk.clone(),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub(&format!("{seed}/kx")),
            &kdb::det_mlkem768_ek(&format!("{seed}/kek")),
        ),
    );
    let payload = kdb::did_benten_payload(&sig_mk, &doc.cid());
    kdb::did_benten_from_payload_for_test(&payload)
}

/// DROP-11 (TIER-A, LIVE) — the `0x6510` AAD matches the frozen golden, and the
/// audience length prefix is `u32-BE` at the frozen offset.
#[test]
fn drop11_single_audience_aad_golden_and_u32_framing() {
    let aad = short_binding().plaintext_aad_bytes();
    assert_eq!(
        kdb::golden_hex(&aad),
        DROP11_SHORT_AAD_GOLDEN,
        "DROP-11 (TIER-A): the 0x6510 AAD MUST match the frozen golden (aad_version | codepoint | \
         u32-BE audience_len | audience | body_cid(36) | gen). would-FAIL-on-revert: a u16 \
         audience length prefix shifts every following field."
    );

    // The audience length prefix is u32-BE at offset [3..7] == the audience len.
    assert_eq!(aad[0], AAD_VERSION, "DROP-11: aad_version byte");
    assert_eq!(
        &aad[1..3],
        &DROP_TO_RECIPIENT_SEALED_SENDER.to_be_bytes(),
        "DROP-11: codepoint 0x6510"
    );
    let aud_len = u32::from_be_bytes([aad[3], aad[4], aad[5], aad[6]]);
    assert_eq!(
        aud_len as usize,
        SHORT_AUDIENCE.len(),
        "DROP-11: the audience length prefix is a u32-BE equal to the audience byte length \
         (NOT the band's u16 recipient_count width)"
    );
    assert_eq!(
        &aad[7..7 + SHORT_AUDIENCE.len()],
        SHORT_AUDIENCE,
        "DROP-11: the audience bytes follow the u32 length prefix verbatim"
    );
}

/// DROP-11 (LIVE) — wire-transparency: a REAL full-length `did:benten` audience
/// (≈2776 B) is carried transparently, `u32`-framed, un-truncated.
#[test]
fn drop11_did_benten_audience_carried_transparently() {
    let did = det_did_benten("drop11/transparency/audience");
    let audience = did.as_str().as_bytes().to_vec();
    assert!(
        audience.len() > u16::MAX as usize / 32 && audience.len() > 2000,
        "precondition: a real did:benten audience is far longer than a did:key ({} B)",
        audience.len()
    );
    let digest = *blake3::hash(b"drop11 transparency body").as_bytes();
    let aad = BindingContext::DropSealedSender {
        aad_version: AAD_VERSION,
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER,
        audience_did: audience.clone(),
        body_cid: self_describing_cid(&digest),
        recipient_key_generation: 0,
    }
    .plaintext_aad_bytes();

    let aud_len = u32::from_be_bytes([aad[3], aad[4], aad[5], aad[6]]) as usize;
    assert_eq!(
        aud_len,
        audience.len(),
        "DROP-11: the u32 length prefix carries the FULL did:benten audience length"
    );
    assert_eq!(
        &aad[7..7 + aud_len],
        audience.as_slice(),
        "DROP-11: the full did:benten audience is present verbatim (wire-transparency, \
         un-truncated) — a longer did:benten audience just fills the length-prefixed field"
    );
}

/// DROP-11 (ignored → R5) — the binding-typed single seal to a REAL `did:benten`
/// recipient produces an AAD whose `u32` audience length == the recipient's
/// `did:benten` byte length (wire-transparency through the real seal path).
#[test]
fn drop11_binding_seal_threads_did_benten_audience_u32_framed() {
    let r = seal::real_recipient();
    let binding = RecipientBinding::resolve(&r.did, &r.doc).expect("recipient binding resolves");
    let (sk, sender) = seal::hybrid_did_key_sender();
    let plaintext = b"drop-11 seal-path audience framing payload".to_vec();
    let body_cid = *blake3::hash(&plaintext).as_bytes();

    let env = seal::seal_to_binding(&binding, &sender, &sk, &body_cid, 0, &plaintext);

    // The seal binds the binding's did:benten audience — u32-framed, verbatim.
    let aad = single_plaintext_aad_region(&env);
    let expected_audience = r.did.as_str().as_bytes();
    let aud_len = u32::from_be_bytes([aad[3], aad[4], aad[5], aad[6]]) as usize;
    assert_eq!(
        aud_len,
        expected_audience.len(),
        "DROP-11: the binding-typed seal threads the recipient's did:benten audience u32-framed"
    );
    assert_eq!(
        &aad[7..7 + aud_len],
        expected_audience,
        "DROP-11: the recipient's full did:benten audience is threaded verbatim by the real seal"
    );
}
