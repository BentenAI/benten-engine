//! Inv-15 cross-surface regression pins — R6-reround pre-tag closure.
//!
//! **Inv-15** (`docs/INVARIANT-COVERAGE.md`): every load-bearing identifier
//! refers to a canonical PAYLOAD, never a signature-inclusive bundle. Revocation
//! / dedupe / audit-uniqueness / any identity property MUST key off the
//! payload-CID or a semantic tuple — never off the sig-bundle-CID. This gives
//! SUF-CMA-equivalent application-layer security despite the LAMPS Composite
//! ML-DSA construction being EUF-CMA-only. A "MallorySigner" producing a
//! valid-but-different signature over the SAME canonical payload MUST leave the
//! surface's identifier / dedupe key UNCHANGED.
//!
//! The R6-reround pre-tag read-audit verified all four remaining wire-bearing
//! surfaces are Inv-15-clean (INVARIANT-COVERAGE Inv-15 enforcement-path item
//! #1, now READ-AUDIT COMPLETE). This file carries the focused, would-FAIL-on-
//! revert pins for the surfaces reachable from `benten-engine`'s test binary:
//!
//! - **Surface 1 — Device attestation envelope V2** (`benten-id`): REAL pin —
//!   the signing-input canonical bytes (`CanonicalBytes::to_canonical_bytes`,
//!   the sig-EXCLUSIVE `SigInput` projection) are invariant under signature
//!   mutation. Would fail if the `SigInput` folded the `signature` field back in.
//! - **Surface 2 — Sync merge proofs / MST** (`benten-sync`): REAL pins —
//!   `MstEntry.cid` is BLAKE3-over-payload (content-addressed, no signature
//!   input) and `Mst::apply_entries` re-hashes + byte-rejects a declared-CID
//!   mismatch. Would fail if the recompute guard were removed.
//!
//! Surfaces pinned ELSEWHERE / structurally sig-free (documented, not
//! re-pinned here to avoid vacuous tests — `benten-drop` is not a dependency of
//! `benten-engine`):
//!
//! - **Surface 3 — Atrium Drop bundles** (`benten-drop`): load-bearing ids are
//!   `spec_cid` (content) + `audience` (DID CID); `envelope_sig`'s signing input
//!   zeroes `envelope_sig` + `issuer_verifying_key` + `content`
//!   (`envelope_sig.rs`), and the open path RECOMPUTES
//!   `self_describing_cid(BLAKE3(recovered_body))` and fail-closes (the B2
//!   content-splice guard, `layer_c.rs:1126-1135`). Pinned in-crate by
//!   `benten-drop/tests/tf3f_*` (envelope-sig-survives-content-tamper) + the
//!   `kdb_drop*` / layer_c content-splice pins.
//! - **Surface 4 — Subscription / EMIT event envelopes**: NO MallorySigner
//!   surface exists — `EmitEvent { channel, payload }` carries no id and no
//!   signature (`benten-engine/src/emit_broadcast.rs`), subscription dedupe is a
//!   monotonic engine-assigned `max_delivered_seq: u64`
//!   (`engine_subscribe.rs`), and `ChangeEvent.cid` is the affected-Node content
//!   CID + a monotonic `tx_id: u64` (`benten-graph/src/store.rs`). There are no
//!   signature bytes anywhere on the identity/dedupe path, so a signature-
//!   malleability pin would be vacuous by construction.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used)]

use benten_id::device_attestation::{
    CapabilityEnvelope, DeviceAttestation, UptimePolicy, ZoneScope,
};
use benten_id::keypair::Keypair;
use benten_sync::mst::{Mst, MstCid, MstEntry, MstError};

// ---------------------------------------------------------------------------
// Surface 1 — Device attestation envelope V2 (benten-id)
// ---------------------------------------------------------------------------

/// Inv-15: a valid-but-different signature over the SAME attestation payload
/// leaves the load-bearing identifier — the sig-EXCLUSIVE `SigInput` canonical
/// bytes that `issue`/`verify_signature_with` sign and check — UNCHANGED.
///
/// Would-FAIL-on-revert: if the `CanonicalBytes for DeviceAttestation` impl
/// (`crates/benten-id/src/device_attestation.rs:413-432`) folded the
/// `signature` field back into its `SigInput` projection, the two canonical
/// byte-strings below would differ and this assertion would fail.
#[test]
fn device_attestation_sig_input_is_signature_exclusive() {
    let parent = Keypair::generate();
    let device = Keypair::generate();

    let envelope = CapabilityEnvelope {
        runs_sandbox: false,
        holds_zones: ZoneScope::CacheOnly,
        online_uptime: UptimePolicy::SessionBounded,
        runs_atrium_peer: false,
    };

    let mut att =
        DeviceAttestation::issue(&parent, device.public_key().to_did(), envelope).unwrap();

    // The honest attestation verifies against the parent key (the signature IS
    // checked — this is not a "signature is ignored" surface).
    att.verify_signature_with(parent.public_key()).unwrap();

    // The SIGNING INPUT (the sig-EXCLUSIVE `CanonicalBytes` projection — reached
    // via UFCS so it does NOT alias the inherent whole-struct round-trip encoder
    // which DOES include `signature`).
    let sig_input_before = benten_id::CanonicalBytes::to_canonical_bytes(&att);

    // Mallory: same payload (device_did / parent_did / envelope / nonce /
    // issued_at all unchanged), a DIFFERENT signature byte-string.
    att.signature[0] ^= 0xFF;
    let sig_input_after = benten_id::CanonicalBytes::to_canonical_bytes(&att);

    assert_eq!(
        sig_input_before, sig_input_after,
        "Inv-15: the device-attestation signing-input identifier MUST be \
         signature-EXCLUSIVE — mutating the signature must not change it"
    );
}

// ---------------------------------------------------------------------------
// Surface 2 — Sync merge proofs / MST (benten-sync)
// ---------------------------------------------------------------------------

/// Inv-15: an `MstEntry`'s load-bearing identifier (`cid`) is a pure function of
/// its content `payload` (BLAKE3) — there is no signature input, so identity is
/// content-addressed by construction. Same payload ⇒ same cid; different
/// payload ⇒ different cid.
#[test]
fn mst_entry_cid_is_content_addressed_sig_independent() {
    let payload = b"membership-event-value-bytes".to_vec();
    let entry = MstEntry::from_payload("/zone/members/m1", payload.clone());

    // The declared cid equals BLAKE3-over-payload (content, not signature).
    assert_eq!(entry.cid, MstCid::from_bytes(&payload));

    // Deterministic + content-addressed: identical payload ⇒ identical cid.
    let entry_again = MstEntry::from_payload("/some/other/key", payload.clone());
    assert_eq!(
        entry.cid, entry_again.cid,
        "Inv-15: MST entry identity is keyed off payload content only"
    );

    // A different payload ⇒ a different cid.
    let other = MstEntry::from_payload("/zone/members/m1", b"different-bytes".to_vec());
    assert_ne!(entry.cid, other.cid);
}

/// Inv-15: `Mst::apply_entries` NEVER trusts a peer-declared CID — it re-hashes
/// each payload locally and byte-rejects a declared-CID mismatch with
/// `EntryCidByteMismatch`. Would-FAIL-on-revert of the sec-r4r2-1 recompute
/// guard (`crates/benten-sync/src/mst.rs:340-352`).
#[test]
fn mst_apply_entries_rejects_declared_cid_mismatch() {
    let real_payload = b"legitimate-membership-content".to_vec();
    let honest_cid = MstCid::from_bytes(&real_payload);

    // Adversarial entry: declare the honest cid but carry a DIFFERENT payload
    // (built via direct pub-field init — the sole constructor `from_payload`
    // would recompute the matching cid, so we bypass it to model the peer).
    let adversarial = MstEntry {
        key: "/zone/members/m1".to_string(),
        cid: honest_cid,
        payload: b"attacker-substituted-content".to_vec(),
    };

    let mut mst = Mst::new();
    match mst.apply_entries(vec![adversarial]) {
        Err(MstError::EntryCidByteMismatch { declared, computed }) => {
            assert_eq!(declared, honest_cid);
            assert_ne!(computed, honest_cid);
        }
        other => panic!("expected EntryCidByteMismatch, got {other:?}"),
    }
    // The adversarial entry was NOT applied — no substituted content lands.
    assert!(mst.is_empty());
}
