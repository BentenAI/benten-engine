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
//! Pin sources (spec of record = R0.5; minted against R0.3 =
//! `4fe9236a:.addl/phase-4-meta/f-full-r0-plan.md`; §-numbers below are
//! stable R0.3→R0.5 — verified vs the R0.5 plan at
//! `phase-4-meta-core/f-full-r0-plan-r05`):
//!   - §2.5 (`{MembershipSet, Drop}` 2 primitives; GN-1).
//!   - §3.5 / §2.5: "`Drop` (one-shot non-member share). The existing
//!     `benten-drop/` content-bundle — a sealed sibling of the graph
//!     (content = encrypted Nodes + a SubgraphSpec; envelope = frozen
//!     crypto wire; **provably carries no `K_Set`**). KEEP-AS-IS (GN-1
//!     §2.5)."
//!   - §2.5 BC-2: Scale (`MembershipSetKind`) is the ONLY keying axis —
//!     `K_Set` is the set's group key; a Drop is OUTSIDE that axis.
//!   - R4 triage CC-BLK: "seal a Drop over a member's subtree, scan ALL
//!     bytes of the serialized `DropBundlePayload` for the live `K_Set`
//!     sentinel → assert absent; would-FAIL if an impl bundles the set key."
//!
//! # RED-PHASE STATUS + STUB-SHIM DISCIPLINE (pim-12 §3.6e)
//!
//! At the F-full baseline the F-full `DropBundlePayload` shape (the
//! canonical-payload half of the DUAL-CID, carrying the per-recipient
//! HPKE-wrapped CEKs) is Layer-C canary scope and the MembershipSet
//! `K_Set` keying axis does not yet exist in-tree. To keep this R3 wave
//! PARALLEL-SAFE (the brief forbids depending on another wave's
//! crate/module), this file carries a SELF-CONTAINED `drop_no_kset_stub`
//! modeling:
//!   - a `K_Set` (the per-set group key — the thing that MUST NOT appear),
//!   - a member subtree (the content being shared),
//!   - the canonical `DropBundlePayload` shape + its deterministic
//!     serializer,
//!   - the production `seal_drop_over_subtree` call site.
//!
//! The serializer is implemented DETERMINISTICALLY (not `unimplemented!()`)
//! so the byte-scan is computable green at red-phase; the `seal_*` call
//! site IS implemented in the stub (it must produce a payload to scan), but
//! it deliberately models the CORRECT shape (per-recipient wrapped CEKs,
//! NO set key) so the pin is GREEN against the correct stub and would-FAIL
//! against a wrong-but-plausible impl that bundles the `K_Set`. The
//! Layer-C closing-wave R5 implementer MUST:
//!   1. DELETE the local `drop_no_kset_stub` module,
//!   2. INSERT the real `use benten_drop::{DropBundlePayload, …};` +
//!      `use benten_membership_set::KSet;`,
//!   3. UN-IGNORE each test,
//!   4. Verify the byte-scan PASSES against the REAL `seal_drop_over_subtree`
//!      production path (the real Drop must provably exclude the set key).
//! Reviewer verifies landing-status (un-ignored + green against real
//! production seal), not just spec-pin presence (pim-12 §3.6e).
//!
//! # Wave-0 (M-20) + would-FAIL (pim-2 sub-rule-4 + pim-18 + §3.6f).
//!
//! The byte-scan drives the production `seal_drop_over_subtree` +
//! `serialize_payload` call sites + asserts an OBSERVABLE consequence (the
//! `K_Set` sentinel is ABSENT from EVERY serialized byte) + is
//! would-FAIL-if-an-impl-bundles-the-set-key. A paired POSITIVE control
//! proves the scanner actually fires (a deliberately-leaky payload IS
//! caught). NEVER `assert_eq!(CONST, CONST_VAL)`; NEVER a zero-assertion
//! arm.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![allow(unused_variables)]

// ===========================================================================
// R5 — real production surface (`benten_drop::payload`). The self-contained
// stub is DELETED; the canonical `DropBundlePayload` + `KSet` + the seal fns
// are imported. The hermetic CID/recipient/subtree fixtures remain test-local.
// ===========================================================================

use benten_drop::payload::{EncryptedNode, seal_drop_leaky_for_negative_control};
use benten_drop::{DropBundlePayload, KSet, seal_drop_over_subtree};

fn fixed_cid(seed: u8) -> [u8; 32] {
    [seed; 32]
}
fn fixed_recipient_pk(seed: u8) -> [u8; 32] {
    [seed.wrapping_add(0x40); 32]
}
fn sample_subtree() -> Vec<EncryptedNode> {
    vec![
        EncryptedNode {
            node_cid: fixed_cid(0x01),
            ciphertext: b"encrypted node 1 content".to_vec(),
        },
        EncryptedNode {
            node_cid: fixed_cid(0x02),
            ciphertext: b"encrypted node 2 content".to_vec(),
        },
    ]
}

/// The live `K_Set` sentinel — a distinctive 32-byte run the scan hunts
/// for. (Defined here rather than via the illustrative stub function so the
/// sentinel is a single, unambiguous source of truth for the scan.)
fn live_k_set_sentinel() -> KSet {
    const PAT: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut k = [0u8; 32];
    for (i, b) in k.iter_mut().enumerate() {
        *b = PAT[i % 4] ^ (i as u8);
    }
    k
}

/// Does `haystack` contain the full `needle` byte run?
fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

// ===========================================================================
// F-DROP-NO-KSET — the load-bearing confidentiality invariant.
// ===========================================================================

/// F-DROP-NO-KSET PIN 1 — seal a Drop over a member's subtree, serialize
/// the `DropBundlePayload`, and byte-scan ALL serialized bytes for the
/// live `K_Set` sentinel → assert ABSENT.
///
/// This proves "a Drop is a sealed sibling that provably carries NO group
/// key" (R0.5 §2.5 GN-1 / §3.5). The Drop carries only the per-recipient
/// HPKE-wrapped CEK (decryptable by THAT recipient), never the set key.
///
/// would-FAIL if an impl bundled the `K_Set` (a total confidentiality
/// break — it would leak the whole group's keys to a one-shot non-member
/// recipient).
#[test]
fn f_drop_no_kset_serialized_payload_omits_k_set() {
    let k_set = live_k_set_sentinel();
    let subtree = sample_subtree();
    let spec_cid = fixed_cid(0x5C);
    let recipient_pk = fixed_recipient_pk(0x07);

    // Seal a Drop over the member's subtree. `k_set` is READ to derive the
    // member's view, but a correct Drop NEVER serializes it.
    let payload: DropBundlePayload =
        seal_drop_over_subtree(&subtree, &spec_cid, &recipient_pk, &k_set);

    let bytes = payload.serialize();

    // Sanity: the payload is non-trivial (the scan is over real content).
    assert!(
        bytes.len() > 32,
        "F-DROP-NO-KSET: the serialized DropBundlePayload must carry real \
         content (the scan must be over a non-empty bundle)."
    );

    // THE INVARIANT: the live K_Set sentinel MUST NOT appear anywhere in
    // the serialized payload bytes.
    assert!(
        !contains_subslice(&bytes, &k_set),
        "F-DROP-NO-KSET (CC-BLK): the serialized DropBundlePayload MUST NOT \
         contain the live K_Set (group key) anywhere in its bytes. A Drop \
         is a one-shot share to a NON-MEMBER; embedding the set key would \
         leak the ENTIRE group's keys (every member key derivable) to that \
         recipient — a total confidentiality break of the central sharing \
         primitive. The Drop carries only the per-recipient HPKE-wrapped \
         CEK, never K_Set. would-FAIL if an impl bundled the set key."
    );
}

/// F-DROP-NO-KSET PIN 2 — NEGATIVE CONTROL: a deliberately-leaky seal that
/// DOES embed the `K_Set` IS caught by the scanner. This proves PIN 1 is
/// not vacuously passing because the scanner is broken / the sentinel
/// never appears anywhere. would-FAIL if the scanner could not detect the
/// `K_Set` even when it IS present (then PIN 1 guarantees nothing).
#[test]
fn f_drop_no_kset_negative_control_leaky_seal_is_caught() {
    let k_set = live_k_set_sentinel();
    let subtree = sample_subtree();
    let spec_cid = fixed_cid(0x5C);
    let recipient_pk = fixed_recipient_pk(0x07);

    // A WRONG impl that embeds the set key (modeled by the negative-control
    // seal). The scanner MUST detect it.
    let leaky: DropBundlePayload =
        seal_drop_leaky_for_negative_control(&subtree, &spec_cid, &recipient_pk, &k_set);
    let leaky_bytes = leaky.serialize();

    assert!(
        contains_subslice(&leaky_bytes, &k_set),
        "F-DROP-NO-KSET (negative control): a Drop payload that DOES embed \
         the K_Set MUST be detected by the byte-scan. If this control \
         fails, the scanner is broken and PIN 1's 'absent' assertion \
         guarantees nothing — the scan must actually fire on a real leak."
    );
}

/// F-DROP-NO-KSET PIN 3 — the recipient-wrapped CEK region itself does not
/// equal / does not contain the `K_Set` (the wrapped CEK is per-recipient
/// authority to THIS bundle, NOT the set key in disguise). This sharpens
/// PIN 1: even the legitimate "authority" field of a correct Drop must be
/// independent of the set key, so an impl can't satisfy PIN 1 by burying
/// the set key inside the CEK field under a different framing.
/// would-FAIL if the wrapped-CEK derivation leaked the set key bytes.
#[test]
fn f_drop_no_kset_recipient_cek_is_independent_of_k_set() {
    let k_set = live_k_set_sentinel();
    let subtree = sample_subtree();
    let spec_cid = fixed_cid(0x5C);
    let recipient_pk = fixed_recipient_pk(0x07);

    let payload = seal_drop_over_subtree(&subtree, &spec_cid, &recipient_pk, &k_set);

    assert!(
        !contains_subslice(&payload.recipient_wrapped_cek, &k_set),
        "F-DROP-NO-KSET: the per-recipient wrapped CEK MUST be independent \
         of the K_Set — it is HPKE-wrapped authority to decrypt THIS bundle \
         only, derived from the recipient pubkey + a fresh bundle CEK, NOT \
         the set key re-framed. would-FAIL if the CEK derivation embedded \
         the set key bytes."
    );

    // Cross-control: two Drops of the SAME subtree under the SAME K_Set but
    // to DIFFERENT recipients produce DIFFERENT wrapped CEKs — confirming
    // the CEK tracks the recipient, not the (shared) set key.
    let other_recipient = fixed_recipient_pk(0x99);
    let other = seal_drop_over_subtree(&subtree, &spec_cid, &other_recipient, &k_set);
    assert_ne!(
        payload.recipient_wrapped_cek, other.recipient_wrapped_cek,
        "F-DROP-NO-KSET: Drops to DIFFERENT recipients (same subtree, same \
         K_Set) MUST carry DIFFERENT wrapped CEKs — proving the CEK tracks \
         the recipient, not the set key. If they were equal, the 'authority' \
         field would be a shared (set-derived) secret in disguise."
    );
}
