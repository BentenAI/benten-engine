//! GAP-KDB Shape-B / Fork-A — ENG-2 engine seam: MAX_UCAN_ENVELOPE
//! consumer parity. W3 authority-migration RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A (FS-1
//! MAX_UCAN_ENVELOPE_BYTES re-sizing) + `R2-LANDSCAPE` ENG-2 ("typed-CALL
//! ucan_validate_chain + durable UCAN backend" consumer parity with
//! AUTH-11).
//!
//! The engine's typed-CALL `ucan_validate_chain`
//! (`typed_call_dispatch.rs`) decodes each token via
//! `benten_id::ucan::Ucan::from_canonical_bytes_bounded` — the SAME
//! decoder that enforces `MAX_UCAN_ENVELOPE_BYTES`. So when Fork-A
//! re-sizes the cap for composite chains (AUTH-11), the engine consumer
//! MUST admit the same composite-sized envelope — no engine-local
//! divergent cap.
//!
//! # would_fail_on_revert
//! - Functional parity (ignored): a composite-sized UCAN envelope that
//!   exceeds the Ed25519-era 64 KiB base cap is admitted by the engine's
//!   decoder after the re-size. Reverting the re-size makes the decoder
//!   reject it (EnvelopeTooLarge) → the `is_ok()` flips.
//! - Structural parity (NON-ignored, REAL now): the engine's typed-CALL
//!   dispatch does NOT hardcode its own UCAN-envelope byte cap — it
//!   delegates to benten-id's single `from_canonical_bytes_bounded`. A
//!   re-introduced engine-local `64 * 1024` cap would silently diverge
//!   from the re-sized benten-id cap → this audit fails.
//!
//! # R5 un-ignore
//! After the AUTH-11 cap re-size lands, the composite envelope is
//! admitted; drop `#[ignore]` on the functional arm.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use benten_crypto_suite::{SigCodepoint, SignatureSuite};
use benten_id::did::Did;
use benten_id::kdb_testing as kdb;
use benten_id::ucan::{Capability, MAX_UCAN_ENVELOPE_BYTES, MAX_UCAN_PROOF_DEPTH, Ucan, UcanClaims};

const NOW: u64 = 1_900_000_000;

/// A `did:benten` (composite identity) — its ~2762-char string is what
/// makes a composite chain's envelope realistically large.
fn benten_did() -> Did {
    let signer = kdb::hybrid_keypair();
    let doc = kdb::KeySetDocument::v1_hybrid(
        kdb::signing_multikey_of(&signer.public()),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub("eng2/x"),
            &kdb::det_mlkem768_ek("eng2/ek"),
        ),
    );
    kdb::self_committed_did(&doc)
}

// ── ENG-2 — engine decoder admits a composite-sized envelope (parity) ─────

#[test]
#[ignore = "RED-PHASE: ENG-2 engine UCAN decoder admits composite-sized envelope after re-size — un-ignore at R5"]
fn eng2_engine_decoder_admits_composite_sized_envelope() {
    // Build a nested composite-sized UCAN chain whose serialized envelope
    // exceeds the Ed25519-era 64 KiB base cap. Signatures are dummy
    // composite-LENGTH bytes (the decoder bounds size + depth; it does
    // NOT verify signatures at this seam — the typed-CALL verifies AFTER
    // decode), so no signing is needed. did:benten iss/aud strings supply
    // the realistic composite-identity byte weight.
    let composite_sig_len =
        SignatureSuite::v1_default().signature_byte_len_for(SigCodepoint::HYBRID_ED25519_MLDSA65);
    let dummy_composite_sig = vec![0u8; composite_sig_len];
    let did = benten_did();
    let did_str = did.as_str().to_string();

    // 16 nested links: depth 15 < MAX_UCAN_PROOF_DEPTH (32); byte weight
    // (16 × ~9 KB) comfortably exceeds 64 KiB.
    const LINKS: usize = 16;
    let mut token = Ucan {
        claims: UcanClaims {
            iss: did_str.clone(),
            aud: did_str.clone(),
            att: vec![Capability::new("/zone/posts", "read")],
            nbf: Some(NOW - 1),
            exp: Some(NOW + 3600),
            prf: Vec::new(),
        },
        signature: dummy_composite_sig.clone(),
    };
    for _ in 1..LINKS {
        token = Ucan {
            claims: UcanClaims {
                iss: did_str.clone(),
                aud: did_str.clone(),
                att: vec![Capability::new("/zone/posts", "read")],
                nbf: Some(NOW - 1),
                exp: Some(NOW + 3600),
                prf: vec![token],
            },
            signature: dummy_composite_sig.clone(),
        };
    }

    let bytes = serde_ipld_dagcbor::to_vec(&token).expect("serialize composite chain envelope");
    assert!(
        bytes.len() > 64 * 1024,
        "precondition: a {LINKS}-link composite chain envelope ({} B) must exceed the \
         Ed25519-era 64 KiB base cap",
        bytes.len()
    );

    // The engine's typed-CALL ucan_validate_chain decodes each token
    // through EXACTLY this call — parity means it admits the composite
    // envelope once MAX_UCAN_ENVELOPE_BYTES is re-sized (AUTH-11).
    let decoded = Ucan::from_canonical_bytes_bounded(&bytes, MAX_UCAN_PROOF_DEPTH);
    assert!(
        decoded.is_ok(),
        "ENG-2: the engine's UCAN decoder (from_canonical_bytes_bounded — the typed-CALL \
         ucan_validate_chain + durable-backend consumer) MUST admit a composite-sized \
         envelope ({} B) once MAX_UCAN_ENVELOPE_BYTES is re-sized (AUTH-11 parity); got {:?}",
        bytes.len(),
        decoded.err()
    );
}

// ── ENG-2 — no engine-local divergent UCAN envelope cap ───────────────────

#[test]
fn eng2_no_engine_local_hardcoded_ucan_envelope_cap() {
    // Non-ignored structural parity audit (REAL now + after re-size): the
    // engine's typed-CALL dispatch must NOT carry its own UCAN-envelope
    // byte cap — it delegates to benten-id's single
    // `from_canonical_bytes_bounded`. A re-introduced engine-local
    // `64 * 1024` / `65536` cap would silently diverge from the re-sized
    // benten-id cap (the ENG-2 parity hazard).
    let dispatch_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("typed_call_dispatch.rs");
    let body = std::fs::read_to_string(&dispatch_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", dispatch_src.display()));

    let mut offenders = Vec::new();
    for (lineno, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        // A UCAN-envelope byte-cap literal is the divergence hazard.
        // (The proof-DEPTH cap `MAX_UCAN_PROOF_DEPTH` is a named benten-id
        // const, not a byte literal, so it does not match.)
        if line.contains("MAX_UCAN_ENVELOPE_BYTES")
            || ((line.contains("64 * 1024") || line.contains("65536")) && line.contains("ucan"))
        {
            offenders.push(format!("typed_call_dispatch.rs:{}: {}", lineno + 1, line.trim()));
        }
    }
    assert!(
        offenders.is_empty(),
        "ENG-2 parity: the engine typed-CALL dispatch MUST NOT define its own UCAN-envelope \
         byte cap — it delegates to benten_id::ucan::Ucan::from_canonical_bytes_bounded (the \
         single re-sized cap). An engine-local cap silently diverges from AUTH-11.\n{}",
        offenders.join("\n")
    );
}
