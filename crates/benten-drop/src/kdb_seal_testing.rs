//! GAP-KDB Shape-B — benten-drop Layer-C **seal-API** RED-PHASE fixtures
//! (W2 flagship).
//!
//! Ref `3bea1294` design spec `GAP-KDB-B-DESIGN-R1.md` §5 (seal-API closure
//! + `RecipientBinding`) + corrections C4/C8/C9 + R2 landscape
//! `GAP-KDB-B-R2-LANDSCAPE.md` W2 families DROP-1..DROP-11. This module is the
//! **benten-drop-side** companion to `benten_id::kdb_testing` (the W0 canary
//! surface): it supplies the frozen SHAPE of the *new* seal API that replaces
//! the substitutable `(recipient_pub, audience_did)` pair with a single
//! [`RecipientBinding`] whose KEM key
//! is cryptographically committed by its audience DID.
//!
//! Gated behind `#[cfg(feature = "testing")]` (NOT `any(test, …)`): it imports
//! `benten_id::kdb_testing`, which is only available when `benten-id/testing`
//! is enabled — and benten-drop's `testing` feature is the thing that chains
//! it on. It is a **library** module so the W2 `tests/kdb_drop*.rs` red-phase
//! files can `use benten_drop::kdb_seal_testing::*`.
//!
//! # RED-PHASE stub-shim discipline (how this becomes real at R5)
//!
//! Two disjoint categories live here, mirroring the W0 canary split:
//!
//! - **Fixture DATA + FROZEN-spec assembly (REAL now).** Real hybrid recipient
//!   keypairs ([`real_recipient_kp`]), [`KeySetDocument`] builders that commit a
//!   REAL KEM public key ([`keyset_doc_committing_real_kem`]), and the honest /
//!   substituted scenario builders ([`real_recipient`], [`substitution_scenario`],
//!   [`did_benten_sender`]). These give stable coupled scenarios + round-trip
//!   inputs whose KEM keypairs actually open.
//! - **LOGIC-UNDER-TEST (STUB `todo!()` now → real entry at R5).** The *new*
//!   binding-based seal entries [`seal_to_binding`] (single `0x6510`),
//!   [`seal_group_to_bindings`] (group `0x6520`), and
//!   [`seal_membership_set_to_bindings`] (MembershipSet `0x6610`) are the
//!   GAP-KDB seal surface. They are `todo!()` stubs at R3. At R5 each body is
//!   replaced by a delegation to the minted real entry (e.g. the
//!   `RecipientBinding`-typed `seal_sealed_sender`), and the red-phase tests
//!   un-ignore. Because `todo!()` panics, no red-phase test can pass against a
//!   stub — the ONLY way each pin goes green is against a real, non-no-op
//!   implementation (substance by construction).
//!
//! **R5 handoff (single-file swap):** mint benten-drop's real `RecipientBinding`
//! (sole `resolve` constructor, no fallback door — C4) + the binding-typed seal
//! API (single + group `&[RecipientBinding]`, C9 roster-replacement); then in
//! THIS file (a) replace each `todo!()` seal body with the real-entry
//! delegation, (b) drop the `benten_id::kdb_testing::RecipientBinding` stub
//! import in favor of the benten-drop type. Test call sites do not change —
//! they only un-ignore.

#![allow(
    // RED-PHASE fixtures: todo!() stubs + never-used-in-a-given-binary helpers
    // are expected during the red phase (mirrors the workspace `todo = "allow"`
    // posture + the W0 canary `kdb_testing` module). The R5 swap removes the stubs.
    clippy::todo,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    dead_code
)]

use benten_crypto_suite::cipher_suite::{
    CipherSuite, CipherSuiteCodepoint, ML_KEM_768_EK_LEN, RecipientKeypair, RecipientPublic,
    RecipientSecret, X25519_PUBLIC_LEN,
};
use benten_crypto_suite::sig::{Keypair as SigKeypair, SignatureSuite};
use benten_id::did::Did;
use benten_id::kdb_testing::{self as kdb, KeySetDocument, RecipientBinding};

use crate::layer_c::group_posture::{GroupSealParams, GroupSealedEnvelope};
use crate::layer_c::{
    self, BodyCidDigest, EncryptedEnvelope, LayerCError, RecipientDid, SenderDid,
};

// ─────────────────────────────────────────────────────────────────────────
// NEW binding-based seal API — LOGIC-UNDER-TEST (stub `todo!()` → real at R5).
//
// Free functions so the R3→R5 swap is a single-file edit and the test call
// sites never change. Each replaces a substitutable seal signature (design
// §5): the recipient KEM key is no longer an independently-chosen param — it
// arrives PROVEN-committed inside a `RecipientBinding`.
// ─────────────────────────────────────────────────────────────────────────

/// The convenience `0x647a` HYBRID_X25519_MLKEM768 codepoint handle.
pub fn hybrid_kem_codepoint() -> CipherSuiteCodepoint {
    CipherSuiteCodepoint::HYBRID_X25519_MLKEM768
}

/// NEW single-recipient Sealed-Sender seal (`0x6510`, design §5). The recipient
/// is a [`RecipientBinding`] whose KEM key is PROVEN committed by its audience
/// DID — there is NO `(recipient_pub: &RecipientPublic, audience_did:
/// &AudienceDid)` two-param door (design C4 / DROP-3). STUB `todo!()` → R5
/// `layer_c::seal_sealed_sender(recipient: &RecipientBinding, …)`.
pub fn seal_to_binding(
    _recipient: &RecipientBinding,
    _sender_did: &SenderDid,
    _sender_kp: &SigKeypair,
    _body_cid: &BodyCidDigest,
    _recipient_key_generation: u32,
    _plaintext: &[u8],
) -> EncryptedEnvelope {
    todo!(
        "RED-PHASE (DROP-1/5/6/8): binding-typed seal_sealed_sender(&RecipientBinding) \
         lands in benten-drop Layer-C at R5 (GAP-KDB-B W2). un-ignore then."
    )
}

/// NEW group multi-stanza seal (`0x6520`, design §5 / C9). BOTH the
/// `audience_set_commitment` roster AND the per-stanza wrap-targets are derived
/// from the ONE `&[RecipientBinding]` slice — retiring the fabricated-DID
/// placeholder roster (`layer_c.rs:1093`, `group_roster` hashing KEM keys into
/// `did:key:z…`). An empty slice is a typed-reject (DROP-10). STUB `todo!()` →
/// R5 `layer_c::seal_group_multi(recipients: &[RecipientBinding], …)`.
pub fn seal_group_to_bindings(
    _recipients: &[RecipientBinding],
    _sender_did: &SenderDid,
    _sender_kp: &SigKeypair,
    _body_cid: &BodyCidDigest,
    _recipient_key_generation: u32,
    _plaintext: &[u8],
) -> Result<EncryptedEnvelope, LayerCError> {
    todo!(
        "RED-PHASE (DROP-1/4/6/9/10): binding-typed group seal_group_multi(&[RecipientBinding]) \
         (C9 roster-replacement) lands in benten-drop Layer-C at R5 (GAP-KDB-B W2). un-ignore then."
    )
}

/// NEW MembershipSet K_Set group seal (`0x6610`, design §5 / C9). Same
/// roster-replacement as [`seal_group_to_bindings`] — the member roster is the
/// ONE `&[RecipientBinding]` slice, not a KEM-key-hashed placeholder. STUB
/// `todo!()` → R5 `layer_c::group_posture::seal_membership_set_group(recipients:
/// &[RecipientBinding], …)`.
pub fn seal_membership_set_to_bindings(
    _recipients: &[RecipientBinding],
    _sender_did: &SenderDid,
    _sender_kp: &SigKeypair,
    _k_set: &[u8; 32],
    _params: &GroupSealParams,
    _plaintext: &[u8],
) -> Result<GroupSealedEnvelope, LayerCError> {
    todo!(
        "RED-PHASE (DROP-1/6): binding-typed seal_membership_set_group(&[RecipientBinding]) \
         lands in benten-drop Layer-C at R5 (GAP-KDB-B W2). un-ignore then."
    )
}

// ─────────────────────────────────────────────────────────────────────────
// FIXTURE BUILDERS (REAL) — real KEM keypairs + committed key-set docs + DIDs.
// ─────────────────────────────────────────────────────────────────────────

/// A REAL hybrid recipient keypair with genuine, independent secret entropy
/// (the production keying path). Two calls yield DISTINCT keypairs.
pub fn real_recipient_kp() -> RecipientKeypair {
    CipherSuite::resolve(hybrid_kem_codepoint())
        .expect("0x647a wire-locked")
        .generate_recipient_keypair()
}

/// Re-parse the public half through the frozen `to_bytes`/`from_bytes` surface.
pub fn pub_of(kp: &RecipientKeypair) -> RecipientPublic {
    RecipientPublic::from_bytes(hybrid_kem_codepoint(), &kp.public().to_bytes())
        .expect("re-parse of recipient public must succeed")
}

/// Re-parse the secret half through the frozen `to_bytes`/`from_bytes` surface.
pub fn sec_of(kp: &RecipientKeypair) -> RecipientSecret {
    RecipientSecret::from_bytes(hybrid_kem_codepoint(), &kp.secret().to_bytes())
        .expect("re-parse of recipient secret must succeed")
}

/// Build a [`KeySetDocument`] whose `kem` field commits a REAL recipient KEM
/// public key. `RecipientPublic::to_bytes()` is **X25519-first** (`x25519(32)
/// ‖ mlkem768_ek(1184)`) — the exact order the frozen C2 `kem` multikey uses —
/// so a `RecipientBinding::resolve` against this doc recovers a KEM key
/// byte-identical to `kem_pub`, and the matching [`RecipientSecret`] opens the
/// seal. `sig_mk` is the signing multikey the doc carries (and the DID embeds).
pub fn keyset_doc_committing_real_kem(
    sig_mk: Vec<u8>,
    kem_pub: &RecipientPublic,
) -> KeySetDocument {
    let raw = kem_pub.to_bytes(); // x25519(32) ‖ mlkem768_ek(1184)
    assert_eq!(
        raw.len(),
        X25519_PUBLIC_LEN + ML_KEM_768_EK_LEN,
        "RecipientPublic bytes must be x25519(32) ‖ mlkem768_ek(1184)"
    );
    let (x, ek) = raw.split_at(X25519_PUBLIC_LEN);
    let mut x32 = [0u8; 32];
    x32.copy_from_slice(x);
    KeySetDocument::v1_hybrid(sig_mk, kdb::kem_multikey_hybrid(&x32, ek))
}

/// A REAL recipient with a self-committed `did:benten`: `doc.kem` commits
/// `kp.public()`, the DID embeds `doc.sig` and commits `cid(doc)`, so at R5
/// `RecipientBinding::resolve(&did, &doc)` succeeds and recovers `pub_of(&kp)`.
pub struct RealRecipient {
    /// The real KEM keypair whose secret opens seals to this recipient.
    pub kp: RecipientKeypair,
    /// The self-committed `did:benten` audience DID.
    pub did: Did,
    /// The committed key-set document (`kem` = `kp.public()`, X25519-first).
    pub doc: KeySetDocument,
}

/// Construct a [`RealRecipient`] with a fresh KEM keypair + a fresh hybrid
/// signing key, self-committed under a `did:benten`.
pub fn real_recipient() -> RealRecipient {
    let kp = real_recipient_kp();
    let sig_kp = kdb::hybrid_keypair();
    let sig_mk = kdb::signing_multikey_of(&sig_kp.public());
    let doc = keyset_doc_committing_real_kem(sig_mk, &pub_of(&kp));
    let did = kdb::self_committed_did(&doc);
    RealRecipient { kp, did, doc }
}

/// The GAP-KDB active-substitution scenario with a REAL honest KEM keypair.
/// The victim `did:benten` commits `cid(honest_doc)`; `honest_doc.kem` commits
/// `honest_kp.public()`; `attacker_doc.kem` commits `attacker_kp.public()`.
/// Both docs carry the SAME embedded signing multikey (so ONLY the KEM key —
/// and thus the committed CID — differs; `resolve(victim_did, attacker_doc)`
/// therefore fails at the step-1 CID 2nd-preimage check, not the step-2
/// `doc.sig == embedded` check).
pub struct SubstitutionScenario {
    /// The victim's `did:benten` — commits `cid(honest_doc)`.
    pub victim_did: Did,
    /// The honestly-committed key-set (`kem` = `honest_kp.public()`).
    pub honest_doc: KeySetDocument,
    /// The honest recipient's real KEM keypair (its secret opens the seal).
    pub honest_kp: RecipientKeypair,
    /// The attacker's substituted key-set — a DISTINCT canonical CID.
    pub attacker_doc: KeySetDocument,
    /// The attacker's real KEM keypair (the key they try to substitute).
    pub attacker_kp: RecipientKeypair,
}

/// Build a [`SubstitutionScenario`]. The victim DID + honest/attacker docs all
/// share one embedded signing multikey; the honest & attacker docs differ only
/// in their committed KEM key.
pub fn substitution_scenario() -> SubstitutionScenario {
    let sig_kp = kdb::hybrid_keypair();
    let sig_mk = kdb::signing_multikey_of(&sig_kp.public());
    let honest_kp = real_recipient_kp();
    let attacker_kp = real_recipient_kp();

    let honest_doc = keyset_doc_committing_real_kem(sig_mk.clone(), &pub_of(&honest_kp));
    let attacker_doc = keyset_doc_committing_real_kem(sig_mk.clone(), &pub_of(&attacker_kp));

    // The victim DID embeds the honest signing key AND commits cid(honest_doc).
    let payload = kdb::did_benten_payload(&sig_mk, &honest_doc.cid());
    let victim_did = kdb::did_benten_from_payload_for_test(&payload);

    SubstitutionScenario {
        victim_did,
        honest_doc,
        honest_kp,
        attacker_doc,
        attacker_kp,
    }
}

/// A hybrid `did:key` sender (the classical/degenerate sender identity) — used
/// by the recipient-side substitution families (DROP-1/DROP-6) so the seal's
/// B2 origin-auth signs + verifies for real WITHOUT coupling to the did:benten
/// sender-origin-auth migration (that is DROP-8's concern). Returns
/// `(sender_kp, sender_did_bytes)`.
pub fn hybrid_did_key_sender() -> (SigKeypair, SenderDid) {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let did = Did::from_hybrid_public_key(&kp.public());
    (kp, did.as_str().as_bytes().to_vec())
}

/// A hybrid sender whose DID is a `did:benten` (the Scope-Min did:benten SEND
/// path, DROP-8): the recipient's Layer-C origin-auth resolves this DID's
/// SIGNING key via `resolve_signing` (method-aware, strips the trailing
/// keyset-CID) and hybrid-verifies the per-message signature. Returns
/// `(sender_kp, sender_did_bytes)` where `sender_did_bytes` is the `did:benten`
/// string whose embedded signing multikey resolves to `sender_kp.public()`.
pub fn did_benten_sender() -> (SigKeypair, SenderDid) {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let sig_mk = kdb::signing_multikey_of(&kp.public());
    // The sender's KEM key is irrelevant to origin-auth (resolve_signing strips
    // it); a deterministic opaque kem multikey keeps the doc well-formed.
    let doc = KeySetDocument::v1_hybrid(
        sig_mk.clone(),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("drop8/sender/x"),
            &kdb::det_mlkem768_ek("drop8/sender/ek"),
        ),
    );
    let payload = kdb::did_benten_payload(&sig_mk, &doc.cid());
    let did = kdb::did_benten_from_payload_for_test(&payload);
    (kp, did.as_str().as_bytes().to_vec())
}

/// A `did:benten` sender whose EMBEDDED signing key does NOT match the returned
/// signing keypair — the forged-origin control for DROP-8. The origin-auth
/// verify resolves the embedded (victim) signing key but the signature was made
/// by a DIFFERENT key, so the hybrid verify MUST fail closed. Returns
/// `(wrong_sender_kp, victim_did_benten_bytes)`.
pub fn did_benten_sender_key_mismatch() -> (SigKeypair, SenderDid) {
    // The DID embeds VICTIM's signing key…
    let victim = SignatureSuite::v1_default().generate_keypair();
    let victim_mk = kdb::signing_multikey_of(&victim.public());
    let doc = KeySetDocument::v1_hybrid(
        victim_mk.clone(),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("drop8/mismatch/x"),
            &kdb::det_mlkem768_ek("drop8/mismatch/ek"),
        ),
    );
    let payload = kdb::did_benten_payload(&victim_mk, &doc.cid());
    let did = kdb::did_benten_from_payload_for_test(&payload);
    // …but the caller signs with ATTACKER's key.
    let attacker = SignatureSuite::v1_default().generate_keypair();
    (attacker, did.as_str().as_bytes().to_vec())
}

// ─────────────────────────────────────────────────────────────────────────
// OPEN-side roster derivation (REAL) — the independently-held roster an honest
// recipient recomputes the B2 `audience_set_commitment` from (F-2). At R5 the
// binding-based group seal binds exactly this over the SORTED binding DIDs.
// ─────────────────────────────────────────────────────────────────────────

/// The `0x6520` open-side independent roster: each binding's REAL audience-DID
/// bytes (the C9 identities), NOT the KEM-key-hashed placeholder DIDs.
pub fn roster_of_bindings(bindings: &[RecipientBinding]) -> Vec<RecipientDid> {
    bindings
        .iter()
        .map(|b| b.audience_did().as_str().as_bytes().to_vec())
        .collect()
}

/// The `0x6610` open-side member-DID roster (`Vec<String>` for
/// `GroupVerifyContext`): each binding's REAL audience-DID string.
pub fn member_dids_of_bindings(bindings: &[RecipientBinding]) -> Vec<String> {
    bindings
        .iter()
        .map(|b| b.audience_did().as_str().to_owned())
        .collect()
}

/// The `audience_set_commitment` over a roster of raw DID byte-strings — the
/// frozen BLINDED 32-byte tag `layer_c::audience_set_commitment` produces
/// (sorted + length-prefixed internally). Re-exported here so DROP-4/DROP-9
/// can name it without re-deriving.
pub fn audience_set_commitment(dids: &[RecipientDid]) -> [u8; 32] {
    layer_c::audience_set_commitment(dids)
}
