//! **F-NQC4-1** — did:key hybrid-pubkey multicodec (CE-I2 + WF-H1, NQ-C4).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition.
//! Doc-coupling + self-contained codec-stub family (reuses the `tf3f`
//! doc-coupling shape for the registration-question half, plus a
//! self-contained multicodec round-trip stub for the codec half — the
//! stub mirrors the in-tree `benten_id::did::ED25519_MULTICODEC` shape at
//! `crates/benten-id/src/did.rs:26,98` without depending on it for
//! parallel-safety).
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-NQC4-1**:
//!   "multicodec prefix for PQ-hybrid sig (Ed25519⊕ML-DSA-65) + KEM
//!    (X25519⊕ML-KEM-768) pubkeys in `did:key`; if no registered
//!    multiformats value, private-value-with-fallback reserved at
//!    G-CORE-9 round-trips; `did:agent:` optional allowlist alias.
//!    §10.6 NQ-C4, §4.1, U15; `did.rs:26,98`.  FG.  ~4-5 tests.
//!    Red-phase intent: `did_key_encode(hybrid_pk).prefix()==MULTICODEC`;
//!    round-trip; unknown multicodec → `UnknownMulticodec`; did:agent
//!    allowlist pin. SURFACES multiformats-registration question if no
//!    value."
//!
//! **OPEN-SPEC ARM FLAGGED (NQ-C4 / §5.D-9):** the multiformats registry
//! may not have an assigned value for the PQ-hybrid pubkeys. Per §5 the
//! reserved-private-value-with-fallback is round-tripped if no registered
//! value exists; the real-corpus (registered-value) swap is flagged for
//! R5. The reserved private prefixes below are PLACEHOLDERS R5 replaces
//! with the registered (or G-CORE-9-reserved) values.
//!
//! **RED-PHASE (pim-12 §3.6e):** the hybrid `did:key` encode/decode path
//! does NOT exist at baseline (`did.rs` is Ed25519-only). The end-state
//! arms are `#[ignore = "RED-PHASE: F-NQC4-1 ..."]`; R5 wires the real
//! `benten_id::did` hybrid encode/decode and un-ignores. The NON-ignored
//! baseline arms drive the self-contained codec stub (a real round-trip,
//! not a CONST compare) + the in-tree Ed25519 multicodec shape, and pass
//! green now. NEVER `assert_eq!(CONST, CONST_VAL)`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// ===========================================================================
// SELF-CONTAINED MULTICODEC STUB-SHIM (no cross-wave / no benten-id dep).
// Mirrors the `did.rs:26` varint-prefix shape. R5 deletes this and uses
// the real `benten_id::did::{encode_did_key_hybrid, parse_did_key}`.
// ===========================================================================

/// RESERVED placeholder multicodec prefixes for the PQ-hybrid pubkeys.
/// These are the "private-value-with-fallback reserved at G-CORE-9"
/// values (NQ-C4) — R5 swaps in the registered multiformats values if
/// they exist, else keeps the reserved private range. NOT the real
/// registered values yet (open-spec arm).
const HYBRID_SIG_MULTICODEC_RESERVED: [u8; 2] = [0x12, 0x01]; // placeholder
const HYBRID_KEM_MULTICODEC_RESERVED: [u8; 2] = [0x13, 0x01]; // placeholder

#[derive(Debug, PartialEq, Eq)]
enum StubDidError {
    UnknownMulticodec(u8, u8),
    Malformed,
}

/// Encode a (prefix, pubkey-bytes) pair into a `did:key`-style payload
/// (prefix || pubkey). This is the round-trippable shim — a real encode,
/// not a CONST.
fn stub_encode(prefix: [u8; 2], pubkey: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + pubkey.len());
    out.extend_from_slice(&prefix);
    out.extend_from_slice(pubkey);
    out
}

/// Decode: recover (prefix, pubkey). Rejects an unrecognized prefix with
/// `UnknownMulticodec` (mirrors `did.rs:98`).
fn stub_decode(payload: &[u8], allowed: &[[u8; 2]]) -> Result<(usize, Vec<u8>), StubDidError> {
    if payload.len() < 2 {
        return Err(StubDidError::Malformed);
    }
    let prefix = [payload[0], payload[1]];
    let idx = allowed.iter().position(|p| *p == prefix);
    match idx {
        Some(i) => Ok((i, payload[2..].to_vec())),
        None => Err(StubDidError::UnknownMulticodec(prefix[0], prefix[1])),
    }
}

fn invariant_doc() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/benten-id/src/did.rs"
    ))
    .expect("did.rs must be present")
}

// ===========================================================================
// BASELINE ARMS (NOT ignored) — real round-trip over the stub codec +
// the in-tree Ed25519 multicodec shape. Pass green now; would-FAIL if the
// codec or the in-tree prefix shape regresses.
// ===========================================================================

/// PIN 0a (baseline) — the stub codec round-trips a synthesized hybrid
/// sig pubkey: `decode(encode(pk)) == pk` AND the recovered prefix index
/// is the sig slot. A no-op codec (returning empty / fixed bytes) fails.
#[test]
fn f_nqc4_1_hybrid_sig_pubkey_round_trips_baseline() {
    // Synthesized hybrid sig pubkey: Ed25519(32) ⊕ ML-DSA-65(1952) — we
    // use a deterministic witness (not the real keygen; real-corpus swap
    // flagged for R5 per §5.D-9).
    let pk: Vec<u8> = (0u16..(32 + 1952)).map(|i| (i % 251) as u8).collect();
    let allowed = [
        HYBRID_SIG_MULTICODEC_RESERVED,
        HYBRID_KEM_MULTICODEC_RESERVED,
    ];

    let encoded = stub_encode(HYBRID_SIG_MULTICODEC_RESERVED, &pk);
    let (slot, recovered) = stub_decode(&encoded, &allowed)
        .expect("encoded hybrid sig pubkey MUST decode under reserved prefix");

    assert_eq!(
        slot, 0,
        "sig pubkey MUST resolve to the sig multicodec slot"
    );
    assert_eq!(
        recovered, pk,
        "did:key hybrid encode/decode MUST round-trip the pubkey bytes \
         byte-for-byte. A lossy or stubbed codec regresses U15."
    );
}

/// PIN 0b (baseline) — an unrecognized multicodec prefix decodes to
/// `UnknownMulticodec` (NEVER silent acceptance). Mirrors the in-tree
/// `did.rs:98` typed-reject. Would-FAIL if decode silently accepts an
/// unknown prefix.
#[test]
fn f_nqc4_1_unknown_multicodec_typed_reject_baseline() {
    let allowed = [
        HYBRID_SIG_MULTICODEC_RESERVED,
        HYBRID_KEM_MULTICODEC_RESERVED,
    ];
    // A prefix not in the allowed set (e.g. a bogus 0xff 0xff).
    let payload = stub_encode([0xff, 0xff], &[1, 2, 3]);
    let err = stub_decode(&payload, &allowed)
        .expect_err("unknown multicodec prefix MUST be typed-rejected");
    assert_eq!(
        err,
        StubDidError::UnknownMulticodec(0xff, 0xff),
        "an unrecognized hybrid multicodec MUST yield UnknownMulticodec, \
         never a silent fallback (mirrors did.rs:98 typed-reject)."
    );
}

/// PIN 0c (baseline) — the in-tree Ed25519 multicodec prefix shape is the
/// W3C-mandated `[0xed, 0x01]` varint. Doc-coupling to the real `did.rs`
/// source: the hybrid prefixes R5 adds MUST sit alongside this same
/// `[byte, 0x01]` varint shape. Would-FAIL if the in-tree shape drifts.
#[test]
fn f_nqc4_1_in_tree_ed25519_multicodec_shape_present_baseline() {
    let did_rs = invariant_doc();
    assert!(
        did_rs.contains("ED25519_MULTICODEC") && did_rs.contains("[0xed, 0x01]"),
        "did.rs MUST define ED25519_MULTICODEC = [0xed, 0x01] (the varint \
         shape the hybrid prefixes extend). The F-NQC4-1 hybrid encode \
         lands alongside it at R5."
    );
}

// ===========================================================================
// RED-PHASE ARMS (ignored until R5) — assert the real benten-id hybrid
// did:key surface + the registration-question doc-coupling.
// ===========================================================================

/// PIN 1 — the REAL `benten-id` hybrid `did:key` encode names a hybrid
/// SIG multicodec const (Ed25519⊕ML-DSA-65). Source doc-coupling: the
/// hybrid-sig multicodec const MUST be defined in `did.rs` at R5.
/// Would-FAIL if the hybrid pubkey is shoe-horned under the Ed25519
/// prefix (which would mis-type the key for every did:key resolver).
#[test]
#[ignore = "RED-PHASE: F-NQC4-1 — did.rs defines a hybrid-SIG multicodec \
            const (Ed25519⊕ML-DSA-65), distinct from Ed25519; un-ignore at R5"]
fn f_nqc4_1_did_rs_defines_hybrid_sig_multicodec() {
    let did_rs = invariant_doc();
    assert!(
        did_rs.contains("HYBRID_SIG_MULTICODEC")
            || did_rs.contains("ED25519_MLDSA65_MULTICODEC")
            || did_rs.contains("MLDSA65_ED25519_MULTICODEC"),
        "did.rs MUST define a hybrid-SIG multicodec const for the \
         Ed25519⊕ML-DSA-65 did:key pubkey (NQ-C4 / U15), distinct from \
         ED25519_MULTICODEC. R5 adds it."
    );
}

/// PIN 2 — the REAL `benten-id` hybrid `did:key` encode names a hybrid
/// KEM multicodec const (X25519⊕ML-KEM-768). Source doc-coupling.
/// Would-FAIL if the KEM pubkey lacks a distinct multicodec.
#[test]
#[ignore = "RED-PHASE: F-NQC4-1 — did.rs defines a hybrid-KEM multicodec \
            const (X25519⊕ML-KEM-768); un-ignore at R5"]
fn f_nqc4_1_did_rs_defines_hybrid_kem_multicodec() {
    let did_rs = invariant_doc();
    assert!(
        did_rs.contains("HYBRID_KEM_MULTICODEC")
            || did_rs.contains("X25519_MLKEM768_MULTICODEC")
            || did_rs.contains("MLKEM768_X25519_MULTICODEC"),
        "did.rs MUST define a hybrid-KEM multicodec const for the \
         X25519⊕ML-KEM-768 did:key pubkey (NQ-C4 / U15). R5 adds it."
    );
}

/// PIN 3 — `did:agent:` optional allowlist alias is named (Inv-22
/// nature-derived alias, NOT a stored discriminator). Doc-coupling to
/// did.rs: the alias MUST be documented as an allowlist alias, not a
/// method that carries authority. Would-FAIL if `did:agent` is wired as
/// a trust-bearing method.
#[test]
#[ignore = "RED-PHASE: F-NQC4-1 — did:agent: documented as optional \
            allowlist alias (Inv-22, not a stored discriminator); \
            un-ignore at R5"]
fn f_nqc4_1_did_agent_optional_allowlist_alias() {
    let did_rs = invariant_doc();
    assert!(
        did_rs.contains("did:agent"),
        "did.rs MUST name the `did:agent:` optional allowlist alias \
         (NQ-C4 + Inv-22: nature DERIVED via method-parse, alias is an \
         OPTIONAL allowlist hint, never a stored authoritative \
         discriminator). R5 adds it."
    );
    // Over-claim guard: the alias MUST NOT be described as carrying
    // authority / being authoritative.
    assert!(
        !did_rs.contains("did:agent grants") && !did_rs.contains("did:agent authority"),
        "`did:agent:` MUST be an OPTIONAL alias, never authority-bearing \
         (Inv-22 nature-derived-not-stored)."
    );
}

/// PIN 4 — the multiformats-registration QUESTION is surfaced in the
/// codepoint doc: either a registered value is cited OR the reserved
/// private-value-with-fallback is documented (NQ-C4 open-spec resolution).
/// Doc-coupling. Would-FAIL if the registration status is left silent
/// (the open-spec arm un-surfaced).
#[test]
#[ignore = "RED-PHASE: F-NQC4-1 (OPEN-SPEC, NQ-C4) — CRYPTO-CODEPOINTS.md \
            surfaces the multiformats-registration status (registered value \
            OR reserved-private-fallback); un-ignore at R5"]
fn f_nqc4_1_multiformats_registration_question_surfaced() {
    let codepoints = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/CRYPTO-CODEPOINTS.md"
    ))
    .unwrap_or_default();
    assert!(
        codepoints.contains("multicodec") && codepoints.contains("hybrid"),
        "CRYPTO-CODEPOINTS.md MUST surface the hybrid-pubkey multicodec \
         registration status (NQ-C4): cite the registered multiformats \
         value if one exists, else document the reserved private-value-\
         with-fallback reserved at G-CORE-9. Leaving it silent un-surfaces \
         the open-spec arm."
    );
}
