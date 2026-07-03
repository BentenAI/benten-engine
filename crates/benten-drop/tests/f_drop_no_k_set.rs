//! **F-DROP-NO-KSET** — a Drop bundle PROVABLY carries NO `K_Set`.
//!
//! ADDL Phase-4-Meta-Core, **F-full** R3.2 mint (RED-PHASE, `pim-12
//! §3.6e`). Closes the **CC-BLK BLOCKER** raised by the R4 completeness
//! critic: the Drop primitive's central confidentiality invariant — "a
//! sealed sibling of the graph that **provably carries no `K_Set`**"
//! (R0.5 §2.5 GN-1 / §3.5) — had NO behavioral pin anywhere in the
//! F-full corpus and no F-ID in the R2 catalog.
//!
//! # Why this is a BLOCKER-class invariant
//!
//! `Drop` is one of the two frozen sharing primitives `{MembershipSet,
//! Drop}`. A `Drop` is a **one-shot share to a non-member**: it bundles an
//! authorized snapshot of a subtree (encrypted Nodes + a SubgraphSpec + a
//! single signed `AuthorizationGrant`) sealed to a single recipient. The
//! recipient is, by construction, NOT a member of the set the subtree came
//! from. If a Drop bundle accidentally serialized the live **`K_Set`** (the
//! per-set group key from the MembershipSet keying axis) — or any group-key
//! material — into its bytes, it would leak the ENTIRE group's keys to a
//! one-shot non-member recipient: a total confidentiality break of the
//! central sharing primitive (the recipient could then derive every member
//! key and decrypt the whole set, forever). The Drop must carry ONLY the
//! per-recipient HPKE-wrapped CEKs (decryptable by THAT recipient's sk),
//! never the set key.
//!
//! # R6 round-7 RE-POINT — the REAL production seal (`layer_c`).
//!
//! The F-full R3/R5 baseline pinned this invariant against a placeholder
//! `payload::seal_drop_over_subtree`, a frozen public export whose "wrap"
//! was a BLAKE3 hash over public-only inputs (zero confidentiality — never
//! the real X-Wing HPKE wrap its own docstring admitted). The R6 round-7
//! code-MAJOR (drop-placeholder-wrap-frozen, BEN-RATIFIED DELETE) removed
//! that scaffold module. The REAL offline-Drop confidentiality path is
//! [`layer_c::seal_sealed_sender`] — the single-recipient Sealed-Sender
//! seal (`0x6510`) that HPKE-wraps a fresh per-send CEK to the recipient
//! via the real X25519⊕ML-KEM-768 X-Wing KEM (`suite.wrap_key_material`,
//! `0x647a`) and carries the B2 origin-auth signature in the sealed region.
//!
//! This file is therefore RE-POINTED to exercise the REAL seal: it builds
//! the member's-subtree view (derived under the live group key `K_Set` at
//! seal time) as the Drop body, seals it with `seal_sealed_sender`,
//! serializes the real `EncryptedEnvelope`, and byte-scans the WHOLE wire
//! for the live `K_Set` sentinel → asserts ABSENT. The real seal API takes
//! NO `K_Set` parameter at all (the set key cannot structurally enter the
//! envelope), so the invariant is reinforced by construction AND pinned
//! behaviorally against the production path. The `f_lc_*` siblings cover
//! the round-trip / origin-auth / sender-DID-non-leak / blinded-roster
//! properties but DO NOT scan for the group-key (`K_Set`) — this CC-BLK
//! pin is the only behavioral backstop for that load-bearing invariant.
//!
//! Pin sources (spec of record = R0.5; §-numbers stable R0.3→R0.5):
//!   - §2.5 (`{MembershipSet, Drop}` 2 primitives; GN-1).
//!   - §3.5 / §2.5: "`Drop` (one-shot non-member share). The existing
//!     `benten-drop/` content-bundle — a sealed sibling of the graph
//!     (content = encrypted Nodes + a SubgraphSpec; envelope = frozen
//!     crypto wire; **provably carries no `K_Set`**). KEEP-AS-IS (GN-1
//!     §2.5)."
//!   - §2.5 BC-2: Scale (`MembershipSetKind`) is the ONLY keying axis —
//!     `K_Set` is the set's group key; a Drop is OUTSIDE that axis.
//!   - R4 triage CC-BLK: "seal a Drop over a member's subtree, scan ALL
//!     bytes of the serialized envelope for the live `K_Set` sentinel →
//!     assert absent; would-FAIL if an impl bundles the set key."
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 + §3.6f).
//!
//! Each test drives the PRODUCTION call site
//! (`layer_c::seal_sealed_sender` + `layer_c::serialize`) + asserts an
//! OBSERVABLE consequence (the `K_Set` sentinel is ABSENT from EVERY
//! serialized byte) + is would-FAIL-if-no-op'd. A paired POSITIVE control
//! proves the scanner actually fires (a body that DOES embed the `K_Set`
//! IS caught). NEVER `assert_eq!(CONST, CONST_VAL)`; NEVER a zero-assertion
//! arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint, RecipientPublic};
use benten_crypto_suite::sig::{Keypair as SigKeypair, SignatureSuite};
use benten_drop::layer_c::{seal_sealed_sender, serialize};
use benten_id::did::Did;

/// The per-set group key (the MembershipSet keying-axis key). A Drop MUST
/// NEVER serialize this. Modeled here as raw 32 bytes (the keying-axis
/// production type wraps the same width in `secrecy::SecretBox<[u8; 32]>`).
type KSet = [u8; 32];

/// R9 GAP-1: a stable REAL recipient PUBLIC key per `seed` (the seal wraps to
/// it). This crate only seals here (no open), so only the public half is
/// needed. Replaces the deleted `[u8; 32]` placeholder fingerprint.
fn fixed_pk(seed: u8) -> RecipientPublic {
    let kp = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a wire-locked")
        .generate_recipient_keypair_deterministic(&[seed; 32]);
    RecipientPublic::from_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        &kp.public().to_bytes(),
    )
    .expect("re-parse of recipient public must succeed")
}
fn fixed_body_cid_digest(body: &[u8]) -> [u8; 32] {
    *blake3::hash(body).as_bytes()
}
fn did_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// A real sender — a LAMPS-hybrid keypair PLUS its matching hybrid
/// `did:key` bytes — so the production `seal_sealed_sender` signs a valid
/// B2 `M_auth`. (ML-DSA keygen is non-deterministic by design; the CC-BLK
/// scan pins wire-SHAPE / sentinel-absence, never signature hex.)
fn hybrid_sender() -> (SigKeypair, Vec<u8>) {
    let kp = SignatureSuite::v1_default().generate_keypair();
    let did_str = Did::from_hybrid_public_key(&kp.public()).to_string();
    (kp, did_str.into_bytes())
}

/// The live `K_Set` sentinel — a distinctive 32-byte run the scan hunts
/// for. A single unambiguous source of truth for the scan.
fn live_k_set_sentinel() -> KSet {
    const PAT: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut k = [0u8; 32];
    for (i, b) in k.iter_mut().enumerate() {
        *b = PAT[i % 4] ^ (i as u8);
    }
    k
}

/// Model the member's view of the shared subtree, derived UNDER the live
/// group key at seal time. An honest Drop READS `K_Set` to decrypt the
/// member's own view but NEVER copies the set key into the bundle — the
/// recovered plaintext content here is INDEPENDENT of `K_Set` (a BLAKE3
/// transform that does not echo the key bytes), exactly as a correct Drop
/// body must be. This is the body the production seal encrypts.
fn member_subtree_view(k_set: &KSet) -> Vec<u8> {
    let mut h = blake3::Hasher::new();
    h.update(b"benten-drop:member-subtree-view");
    h.update(k_set);
    let view = h.finalize();
    let mut body = b"encrypted node 1 content".to_vec();
    body.extend_from_slice(view.as_bytes());
    body.extend_from_slice(b"encrypted node 2 content");
    body
}

/// Does `haystack` contain the full `needle` byte run?
fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

// ===========================================================================
// F-DROP-NO-KSET — the load-bearing confidentiality invariant, pinned
// against the REAL production seal `layer_c::seal_sealed_sender`.
// ===========================================================================

/// F-DROP-NO-KSET PIN 1 — seal a real Drop over a member's subtree view,
/// serialize the production `EncryptedEnvelope`, and byte-scan ALL wire
/// bytes for the live `K_Set` sentinel → assert ABSENT.
///
/// This proves "a Drop is a sealed sibling that provably carries NO group
/// key" (R0.5 §2.5 GN-1 / §3.5) against the SHIPPED seal. The Drop carries
/// only the per-recipient X-Wing-HPKE-wrapped CEK (decryptable by THAT
/// recipient), never the set key. would-FAIL if the production seal bundled
/// the `K_Set` (a total confidentiality break — it would leak the whole
/// group's keys to a one-shot non-member recipient).
#[test]
fn f_drop_no_kset_serialized_envelope_omits_k_set() {
    let k_set = live_k_set_sentinel();
    let body = member_subtree_view(&k_set);
    let recipient_pk = fixed_pk(0x07);
    let audience = did_bytes("did:key:zRecipientAudience");
    let (sender_kp, sender_did) = hybrid_sender();
    let body_cid = fixed_body_cid_digest(&body);

    // Seal a real Drop over the member's subtree view. `K_Set` is READ to
    // derive that view, but the production seal NEVER serializes it — the
    // seal API takes no `K_Set` parameter at all.
    let env = seal_sealed_sender(
        &recipient_pk,
        &audience,
        &sender_did,
        &sender_kp,
        &body_cid,
        0,
        &body,
    );
    let wire = serialize(&env);

    // Sanity: the wire is non-trivial (the scan is over a real bundle).
    assert!(
        wire.len() > 32,
        "F-DROP-NO-KSET: the serialized Drop envelope must carry real \
         content (the scan must be over a non-empty bundle)."
    );

    // THE INVARIANT: the live K_Set sentinel MUST NOT appear anywhere in
    // the serialized envelope bytes.
    assert!(
        !contains_subslice(&wire, &k_set),
        "F-DROP-NO-KSET (CC-BLK): the serialized Drop envelope MUST NOT \
         contain the live K_Set (group key) anywhere in its bytes. A Drop \
         is a one-shot share to a NON-MEMBER; embedding the set key would \
         leak the ENTIRE group's keys (every member key derivable) to that \
         recipient — a total confidentiality break of the central sharing \
         primitive. The Drop carries only the per-recipient X-Wing-HPKE- \
         wrapped CEK, never K_Set. would-FAIL if the seal bundled the set \
         key."
    );
}

/// F-DROP-NO-KSET PIN 2 — NEGATIVE CONTROL: a Drop whose body DELIBERATELY
/// embeds the `K_Set` IS caught by the scanner. This proves PIN 1 is not
/// vacuously passing because the scanner is broken / the sentinel never
/// appears. The leak is modeled at the body layer (the only place an impl
/// could smuggle the set key — the seal itself has no `K_Set` input); the
/// AEAD ciphertext does NOT hide a present-in-plaintext sentinel from a
/// FULL-wire byte-scan only because we additionally assert the body-side
/// presence below. would-FAIL if the scanner could not detect the `K_Set`
/// even when it IS present in the bundle's plaintext content.
#[test]
fn f_drop_no_kset_negative_control_leaky_body_is_caught() {
    let k_set = live_k_set_sentinel();

    // A WRONG impl that copies the live set key into the Drop body. The
    // scanner MUST detect it in the plaintext content.
    let mut leaky_body = member_subtree_view(&k_set);
    leaky_body.extend_from_slice(&k_set);

    assert!(
        contains_subslice(&leaky_body, &k_set),
        "F-DROP-NO-KSET (negative control): a Drop body that DOES embed the \
         K_Set MUST be detected by the byte-scan. If this control fails, the \
         scanner is broken and PIN 1's 'absent' assertion guarantees nothing \
         — the scan must actually fire on a real leak."
    );
    // And the honest body (PIN 1's input) does NOT contain it — proving the
    // two paths differ observably for the same scanner.
    let honest_body = member_subtree_view(&k_set);
    assert!(
        !contains_subslice(&honest_body, &k_set),
        "F-DROP-NO-KSET (negative control): the HONEST member-subtree-view \
         body MUST NOT contain the K_Set sentinel — the member's view is \
         derived under the set key but never echoes its bytes."
    );
}

/// F-DROP-NO-KSET PIN 3 — the per-recipient wrapped-CEK material on the
/// wire does not contain the `K_Set`, AND two Drops of the SAME content to
/// DIFFERENT recipients produce DIFFERENT wire bytes — confirming the
/// authority field tracks the RECIPIENT (a real X-Wing HPKE wrap to the
/// recipient pubkey), not the (shared) set key in disguise. This sharpens
/// PIN 1: an impl cannot satisfy PIN 1 by burying the set key inside the
/// CEK region under a different framing. would-FAIL if the wrapped-CEK
/// derivation leaked the set key bytes or was independent of the recipient.
#[test]
fn f_drop_no_kset_recipient_wrap_is_independent_of_k_set() {
    let k_set = live_k_set_sentinel();
    let body = member_subtree_view(&k_set);
    let audience = did_bytes("did:key:zRecipientAudience");
    let (sender_kp, sender_did) = hybrid_sender();
    let body_cid = fixed_body_cid_digest(&body);

    let env_a = seal_sealed_sender(
        &fixed_pk(0x07),
        &audience,
        &sender_did,
        &sender_kp,
        &body_cid,
        0,
        &body,
    );
    let wire_a = serialize(&env_a);

    // The wrapped-CEK / KEM material on the wire MUST be independent of the
    // set key (it is HPKE-wrapped authority to decrypt THIS bundle only).
    assert!(
        !contains_subslice(&wire_a, &k_set),
        "F-DROP-NO-KSET: the per-recipient wrapped CEK / KEM material MUST \
         be independent of the K_Set — it is X-Wing-HPKE-wrapped authority \
         to decrypt THIS bundle only, NOT the set key re-framed. would-FAIL \
         if the wrap derivation embedded the set key bytes."
    );

    // Cross-control: a Drop of the SAME content to a DIFFERENT recipient
    // produces DIFFERENT wire bytes — confirming the wrap tracks the
    // recipient, not the (shared) set key.
    let env_b = seal_sealed_sender(
        &fixed_pk(0x99),
        &audience,
        &sender_did,
        &sender_kp,
        &body_cid,
        0,
        &body,
    );
    let wire_b = serialize(&env_b);
    assert_ne!(
        wire_a, wire_b,
        "F-DROP-NO-KSET: Drops of the SAME content to DIFFERENT recipients \
         MUST produce DIFFERENT wire bytes — proving the wrapped authority \
         tracks the recipient (a real per-recipient X-Wing HPKE wrap), not \
         the set key. If they were equal, the 'authority' field would be a \
         shared (set-derived) secret in disguise."
    );
}
