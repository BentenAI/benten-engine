//! GAP-KDB Shape-B — shared RED-PHASE test fixtures (W0 canary).
//!
//! Ref `3bea1294` design spec `GAP-KDB-B-DESIGN-R1.md` + R2 landscape
//! `GAP-KDB-B-R2-LANDSCAPE.md`. This module is the **W0 canary surface**:
//! the frozen `did:benten` + [`KeySetDocument`] + resolver API SHAPE that
//! every downstream wave (W1 crypto-suite, W2 benten-drop Layer-C, W3
//! Fork-A authority, W4 docs) consumes. It is gated behind
//! `#[cfg(any(test, feature = "testing"))]` so it never reaches a
//! production build, and it is a **library** module (not `tests/`) so
//! `benten-drop` + `benten-engine` red-phase tests can `use
//! benten_id::kdb_testing::*` cross-crate (enable `benten-id`'s `testing`
//! feature in their dev-deps).
//!
//! # RED-PHASE stub-shim discipline — DISCHARGED at R5 (historical note)
//!
//! At the freeze base (`phase-4-meta-core/r9-base`) the real GAP-KDB types did
//! NOT yet exist, and this module supplied the **expected API shape** so the
//! red-phase test files compiled green while staying `#[ignore]`d. That phase
//! is over: at HEAD the real types are minted and every entry below delegates
//! to them. Two disjoint categories live here:
//!
//! - **Fixture DATA + FROZEN-spec serialization (REAL).** The
//!   multicodec framing constants, [`KeySetDocument::to_canonical_bytes`]
//!   (canonical DAG-CBOR), [`KeySetDocument::cid`] (BLAKE3-256 →
//!   self-describing CIDv1), and the raw `did:benten` string assembler
//!   [`did_benten_from_payload_for_test`] implement the FROZEN spec
//!   directly (deterministic; the same bytes the encoder emits).
//!   These give stable golden pins + coupled test scenarios.
//! - **LOGIC-UNDER-TEST — REAL at HEAD (was `todo!()` at R3).** The
//!   codec encoder [`encode_did_benten`], resolvers [`resolve_signing`] /
//!   [`resolve_kem`] / [`committed_keyset_cid`], and strict decode
//!   [`KeySetDocument::from_canonical_bytes`] are the GAP-KDB security
//!   surface. Each was a `todo!()` stub during the R3 red phase; the R5 swap
//!   replaced every body with a delegation to the minted real entry (e.g.
//!   `did.resolve_kem(doc)`), so at HEAD there is NO `todo!()` in this file
//!   and every pin runs against the real, non-no-op implementation (substance
//!   by construction). `RecipientBinding::resolve` left this module entirely —
//!   see the relocation note below.
//!
//! **R5 handoff — COMPLETE.** The real
//! `benten_id::keyset::KeySetDocument` + `Did::{from_benten_keyset,
//! resolve_signing, resolve_kem, keyset_cid}` + benten-drop
//! `RecipientBinding` are minted; in THIS file (a) the stub `KeySetDocument`
//! is now `pub use crate::keyset::KeySetDocument;`, (b) each free fn delegates
//! to the real method, and (c) the `for_test` escape hatches the real
//! sole-constructor subsumes are gone. Test call sites never changed — they
//! only un-ignored.

#![allow(
    // RED-PHASE fixtures: never-constructed stub fields, todo!() stubs,
    // and pedantic doc-nits are expected during the red phase (mirrors the
    // workspace `todo = "allow"` posture). The R5 swap removes the stubs.
    clippy::todo,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    dead_code
)]

use benten_core::Cid;
use benten_crypto_suite::CipherSuiteCodepoint;
use benten_crypto_suite::cipher_suite::RecipientPublic;
use benten_crypto_suite::sig;

use crate::did::{Did, ED25519_MULTICODEC, MLDSA65_PUB_MULTICODEC};
use crate::errors::DidError;

// ─────────────────────────────────────────────────────────────────────────
// FROZEN constants (v1-beta wire — MUST match the R5 production surface).
// ─────────────────────────────────────────────────────────────────────────

// R5 swap: the frozen `did:benten` + KeySetDocument constants are now the
// REAL production surface — re-exported here so the fixture builders + the
// red-phase test call sites (`kdb::DID_BENTEN_PREFIX`, `kdb::X25519_PUB_MULTICODEC`,
// …) are unchanged.
pub use crate::did::{
    DID_BENTEN_METHOD, DID_BENTEN_PREFIX, MLKEM768_PUB_MULTICODEC, X25519_PUB_MULTICODEC,
};
pub use crate::keyset::{
    KEM_CP_HYBRID_X25519_MLKEM768, KEYSET_DOC_VERSION, SIG_CP_LAMPS_MLDSA65_ED25519,
};

/// The below-PQ-floor classical-only `0x6400` cipher suite. A key-set
/// committing this is HNDL-exposed and MUST be rejected by `resolve_kem`
/// (design C5 — PQ floor).
pub const KEM_CP_CLASSICAL_X25519_FLOOR: u16 = 0x6400;

/// The self-describing Benten CIDv1 header `[0x01, 0x71, 0x1e, 0x20]`
/// (version, dag-cbor codec, blake3 multihash, 32-byte digest length).
/// Re-derived from [`benten_core`] so a header drift there fails these
/// pins rather than a hand-copied literal.
pub const CID_V1_DAGCBOR_BLAKE3_HEADER: [u8; 4] = [
    benten_core::CID_V1,
    benten_core::MULTICODEC_DAG_CBOR,
    benten_core::MULTIHASH_BLAKE3,
    benten_core::BLAKE3_DIGEST_LEN,
];

// ─────────────────────────────────────────────────────────────────────────
// KeySetDocument (design §1.2) — R5 swap: the REAL production type.
//
// At R3 this was a stub whose `to_canonical_bytes` / `cid` were the frozen
// serialization and whose `from_canonical_bytes` was a `todo!()`. At R5 the
// real `benten_id::keyset::KeySetDocument` (identical public API: `v1_hybrid`
// / `v1_with` / `to_canonical_bytes` / `cid` / accessors + the now-real strict
// `from_canonical_bytes`) is re-exported so the test call sites are unchanged.
// ─────────────────────────────────────────────────────────────────────────

pub use crate::keyset::KeySetDocument;

// ─────────────────────────────────────────────────────────────────────────
// RecipientBinding — RELOCATED to benten-drop Layer-C at R5 (W2).
//
// The seal-API typestate (design §5 — Inv-23) is now the REAL
// `benten_drop::layer_c::RecipientBinding` (its DESIGN HOME — the Layer-C
// seal surface). Its sole `resolve` constructor calls `Did::resolve_kem`.
// The R3 stub (with its `for_test_unchecked` escape hatch) is GONE — the
// real sole-constructor / no-fallback-door discipline (design C4) subsumes
// it. Cross-crate W2 tests import it from `benten_drop::layer_c`.
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// CODEC + RESOLVERS — LOGIC-UNDER-TEST (REAL at HEAD; was a `todo!()` stub
// at R3, swapped at R5).
//
// Free functions so the R3→R5 swap was a single-file edit and the test
// call sites never changed. Each body now delegates to the minted real
// method (noted per fn).
// ─────────────────────────────────────────────────────────────────────────

/// Encode a `did:benten` committing `doc` (design §1.1): method-specific
/// id = `signing_multikey(sig_pk) ‖ doc.cid()` (36 B), base58btc, no
/// framing byte (design C1). R5: delegates to the real
/// [`Did::from_benten_keyset`].
pub fn encode_did_benten(sig_pk: &sig::PublicKey, doc: &KeySetDocument) -> Did {
    Did::from_benten_keyset(sig_pk, doc)
}

/// Method-aware signing-key resolve (design §2 Tier-1, C6). `did:key`
/// (0xed01) → classical; hybrid `did:key` (0x1211) → composite;
/// `did:benten` → composite + strip the trailing keyset-CID component.
/// Zero-I/O, no doc. R5: delegates to the real [`Did::resolve_signing`].
pub fn resolve_signing(did: &Did) -> Result<sig::PublicKey, DidError> {
    did.resolve_signing()
}

/// KEM-key resolve (design §2 Tier-2). Recovers + VERIFIES the recipient
/// KEM key from the DID's key-set commitment: (1) `cid(doc) ==
/// committed_cid` (2nd-preimage), (2) `doc.sig == embedded_signing`
/// cross-check, (3) PQ-floor (`0x6400` reject), (4) kem multikey decode +
/// `kem_cp ⟺ components`. Fail-closed on ANY mismatch. R5: delegates to the
/// real [`Did::resolve_kem`].
pub fn resolve_kem(did: &Did, doc: &KeySetDocument) -> Result<RecipientPublic, DidError> {
    did.resolve_kem(doc)
}

/// The committed key-set CID carried in a `did:benten` string (the last
/// 36 payload bytes). A bare `did:key` commits none →
/// [`DidError::NoKemCommitment`] (design §6). R5: delegates to the real
/// [`Did::keyset_cid`].
pub fn committed_keyset_cid(did: &Did) -> Result<Cid, DidError> {
    did.keyset_cid()
}

// ─────────────────────────────────────────────────────────────────────────
// FIXTURE BUILDERS (REAL) — construct frozen-spec inputs + attack scenarios.
// ─────────────────────────────────────────────────────────────────────────

/// A real hybrid (Ed25519⊕ML-DSA-65) keypair via the v1-beta default
/// suite. Random (ML-DSA keygen is non-deterministic); use for
/// round-trip / resolve tests whose assertions are recovery-equality,
/// not frozen hex.
pub fn hybrid_keypair() -> sig::Keypair {
    sig::SignatureSuite::v1_default().generate_keypair()
}

/// The signing multikey a `did:benten` embeds (and a [`KeySetDocument`]'s
/// `sig` field carries): `0x1211‖mldsaPK(1952) ‖ 0xed‖ed25519PK(32)`,
/// ML-DSA-FIRST (byte-identical to the hybrid `did:key` payload). REAL
/// frozen assembly.
pub fn signing_multikey_of(pk: &sig::PublicKey) -> Vec<u8> {
    let composite = pk
        .to_lamps_composite_bytes()
        .expect("hybrid public key required for a signing multikey");
    let mldsa_len = benten_crypto_suite::sizes::ml_dsa_65_pubkey_len();
    let (mldsa_pk, trad_pk) = composite.split_at(mldsa_len);
    let mut out = Vec::with_capacity(2 + mldsa_pk.len() + 2 + trad_pk.len());
    out.extend_from_slice(&MLDSA65_PUB_MULTICODEC);
    out.extend_from_slice(mldsa_pk);
    out.extend_from_slice(&ED25519_MULTICODEC);
    out.extend_from_slice(trad_pk);
    out
}

/// The hybrid KEM multikey — **X25519-first** (design C2, no reorder):
/// `0xec‖x25519(32) ‖ 0x120c‖mlkem768_ek(1184)`. REAL frozen assembly.
pub fn kem_multikey_hybrid(x25519_pub: &[u8; 32], mlkem768_ek: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 32 + 2 + mlkem768_ek.len());
    out.extend_from_slice(&X25519_PUB_MULTICODEC);
    out.extend_from_slice(x25519_pub);
    out.extend_from_slice(&MLKEM768_PUB_MULTICODEC);
    out.extend_from_slice(mlkem768_ek);
    out
}

/// The classical-only (below-PQ-floor) KEM multikey: `0xec‖x25519(32)`.
/// A key-set carrying only this commits `0x6400` and MUST be rejected by
/// `resolve_kem` (design C5).
pub fn kem_multikey_classical(x25519_pub: &[u8; 32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + 32);
    out.extend_from_slice(&X25519_PUB_MULTICODEC);
    out.extend_from_slice(x25519_pub);
    out
}

/// Deterministic, reproducible bytes (BLAKE3 XOF fill). Used for stable
/// golden fixtures where key VALIDITY is irrelevant (the codec frames
/// opaque bytes) — never fed to a key-validating path.
pub fn det_bytes(seed: &str, len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    let mut xof = blake3::Hasher::new();
    xof.update(seed.as_bytes());
    xof.finalize_xof().fill(&mut out);
    out
}

/// A deterministic 32-byte X25519-public placeholder.
pub fn det_x25519_pub(seed: &str) -> [u8; 32] {
    let v = det_bytes(seed, 32);
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    a
}

/// A deterministic 1184-byte ML-KEM-768 EK placeholder (opaque; framing
/// fixture only).
pub fn det_mlkem768_ek(seed: &str) -> Vec<u8> {
    det_bytes(seed, benten_crypto_suite::cipher_suite::ML_KEM_768_EK_LEN)
}

/// A deterministic signing multikey with the FROZEN framing over opaque
/// (`det_bytes`) component payloads — for golden-LAYOUT pins that must be
/// byte-stable (KSD-1 / DID-1 golden-hex capture).
///
/// # ⚠️ LAYOUT-ONLY — do NOT use where the key is PARSED
///
/// The component payloads are pseudo-random `det_bytes`, so whether the result
/// parses as a real LAMPS composite public key is a **per-seed ~50% coin flip**.
/// `benten_crypto_suite::sig::PublicKey::from_lamps_composite_bytes` validates
/// the ML-DSA half by length only (its `decode` is infallible) but calls
/// `ed25519_dalek::VerifyingKey::from_bytes` on the Ed25519 half, which
/// decompresses an Edwards point and FAILS for roughly half of all random
/// 32-byte strings. Validity therefore depends entirely on whether
/// `det_bytes("{seed}/ed25519", 32)` happens to land on the curve.
///
/// Any fixture on a path that PARSES the embedded signing key (e.g.
/// `Did::resolve_kem` step-2 `benten_embedded_signing_multikey`) must use a REAL
/// key — `signing_multikey_of(&hybrid_keypair().public())` — or it risks
/// short-circuiting with `DidError::InvalidHybridPublicKey` and never reaching
/// the gate it means to exercise, while still "passing" under a bare
/// `is_err()`. That is exactly what happened to the RK-4 / RK-5 arms of
/// `crates/benten-id/tests/kdb_resolve_kem_fail_closed.rs` (corrected at the R6
/// tail). Tracked as Row D-94 in `docs/V1-FROZEN-INTERFACE-DEFERRED.md`.
pub fn det_signing_multikey(seed: &str) -> Vec<u8> {
    let mldsa = det_bytes(
        &format!("{seed}/mldsa"),
        benten_crypto_suite::sizes::ml_dsa_65_pubkey_len(),
    );
    let ed = det_bytes(&format!("{seed}/ed25519"), 32);
    let mut out = Vec::new();
    out.extend_from_slice(&MLDSA65_PUB_MULTICODEC);
    out.extend_from_slice(&mldsa);
    out.extend_from_slice(&ED25519_MULTICODEC);
    out.extend_from_slice(&ed);
    out
}

/// Assemble a `did:benten` string from a RAW method-specific-id payload —
/// the fixture escape hatch for crafting attack DIDs (mismatched
/// embedded-sig vs committed-CID for RK-3; trailing-byte / truncation
/// injection for DID-3; CID-sensitivity for DID-6). REAL base58btc
/// assembly following the frozen §1.1 layout; NOT the production encoder
/// (that is [`encode_did_benten`], the logic-under-test). Openly
/// `_for_test`.
#[cfg(any(test, feature = "testing"))]
pub fn did_benten_from_payload_for_test(payload: &[u8]) -> Did {
    let body = bs58::encode(payload).into_string();
    Did::from_string_for_test_fixture(format!("{DID_BENTEN_PREFIX}{body}"))
}

/// The canonical `did:benten` payload for a `(signing multikey, keyset
/// CID)` pair per the frozen §1.1 layout: `signing_multikey ‖
/// cid(36)`, no framing byte (design C1).
pub fn did_benten_payload(signing_multikey: &[u8], keyset_cid: &Cid) -> Vec<u8> {
    let mut payload = Vec::with_capacity(signing_multikey.len() + benten_core::CID_LEN);
    payload.extend_from_slice(signing_multikey);
    payload.extend_from_slice(keyset_cid.as_bytes());
    payload
}

/// Base58btc-decode a `did:benten` string back to its raw method-specific
/// -id payload bytes (the inverse of [`did_benten_from_payload_for_test`]).
/// `None` if the DID is not a `did:benten` or the body fails base58 decode.
pub fn did_benten_payload_of(did: &Did) -> Option<Vec<u8>> {
    let body = did.as_str().strip_prefix(DID_BENTEN_PREFIX)?;
    bs58::decode(body).into_vec().ok()
}

/// The HONEST coupled scenario: a `did:benten` whose committed CID is
/// exactly `cid(doc)` (built via the frozen layout, since the production
/// encoder is a stub at R3). Returns `(audience_did, doc)`.
pub fn honest_recipient_scenario() -> (Did, KeySetDocument) {
    let kp = hybrid_keypair();
    let sig_mk = signing_multikey_of(&kp.public());
    let kem_mk = kem_multikey_hybrid(
        &det_x25519_pub("honest/x25519"),
        &det_mlkem768_ek("honest/mlkem"),
    );
    let doc = KeySetDocument::v1_hybrid(sig_mk.clone(), kem_mk);
    let payload = did_benten_payload(&sig_mk, &doc.cid());
    let did = did_benten_from_payload_for_test(&payload);
    (did, doc)
}

/// A `did:benten` that self-commits `doc`: its embedded signing multikey
/// is exactly `doc.sig()` and its committed CID is exactly `cid(doc)`
/// (built via the frozen §1.1 layout). Used to isolate a SPECIFIC
/// `resolve_kem` reject arm (bad kem-multikey / below-floor / etc.) with
/// steps 1 (CID) and 2 (`doc.sig == embedded`) already satisfied — so the
/// reject can only come from the arm under test.
pub fn self_committed_did(doc: &KeySetDocument) -> Did {
    let payload = did_benten_payload(doc.sig(), &doc.cid());
    did_benten_from_payload_for_test(&payload)
}

/// The GAP-KDB active-substitution scenario (RK-2 / DROP-1 flagship
/// inputs). The victim `did:benten` commits `cid(honest_doc)`; the
/// returned `attacker_doc` carries an ATTACKER-controlled KEM key and (by
/// 2nd-preimage resistance) a DIFFERENT canonical CID. `resolve_kem(&
/// victim_did, &attacker_doc)` MUST fail closed. Returns `(victim_did,
/// honest_doc, attacker_doc)`.
pub fn substituted_recipient_scenario() -> (Did, KeySetDocument, KeySetDocument) {
    let kp = hybrid_keypair();
    let sig_mk = signing_multikey_of(&kp.public());

    let honest_kem = kem_multikey_hybrid(
        &det_x25519_pub("victim/x25519"),
        &det_mlkem768_ek("victim/mlkem"),
    );
    let honest_doc = KeySetDocument::v1_hybrid(sig_mk.clone(), honest_kem);

    // Attacker swaps in their own KEM key. Same embedded signing key, so
    // the ONLY thing that changes is the committed key-set CID.
    let attacker_kem = kem_multikey_hybrid(
        &det_x25519_pub("attacker/x25519"),
        &det_mlkem768_ek("attacker/mlkem"),
    );
    let attacker_doc = KeySetDocument::v1_hybrid(sig_mk.clone(), attacker_kem);

    // The victim DID commits the HONEST doc's CID (frozen layout).
    let payload = did_benten_payload(&sig_mk, &honest_doc.cid());
    let victim_did = did_benten_from_payload_for_test(&payload);
    (victim_did, honest_doc, attacker_doc)
}

// ─────────────────────────────────────────────────────────────────────────
// M-20 golden capture helper.
// ─────────────────────────────────────────────────────────────────────────

/// Lower-hex encode (hand-rolled; avoids a dep for a `feature="testing"`
/// lib module). The M-20 discipline: at R5, run a THROWAWAY that
/// `golden_hex(real_encoder_output())`, freeze the returned string as a
/// test constant, then compare the production encoder against it. Never
/// hand-author a golden (a hand golden matching a buggy encoder freezes
/// the bug).
pub fn golden_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Decode a lower-hex golden string back to bytes (inverse of
/// [`golden_hex`]). Panics on odd length / non-hex — golden constants are
/// author-controlled.
pub fn from_golden_hex(s: &str) -> Vec<u8> {
    fn nib(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("non-hex character in golden constant"),
        }
    }
    let b = s.as_bytes();
    assert!(b.len().is_multiple_of(2), "golden hex must be even length");
    b.chunks_exact(2)
        .map(|p| (nib(p[0]) << 4) | nib(p[1]))
        .collect()
}

/// Convenience: the live `HYBRID_X25519_MLKEM768` cipher-suite codepoint
/// handle (`0x647a`) for scenario construction.
pub fn hybrid_kem_codepoint() -> CipherSuiteCodepoint {
    CipherSuiteCodepoint::HYBRID_X25519_MLKEM768
}
