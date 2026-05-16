//! G14-A1 wave-4a — `Keypair::from_seed_bytes` test pins (un-ignored).
//!
//! Pin sources (per `r2-test-landscape` §2.2 G14-A1 + plan §3 G14-A1).

#![allow(clippy::unwrap_used)]

use benten_id::SeedImportError;
use benten_id::keypair::Keypair;
use proptest::prelude::*;
use serde::Serialize;

#[test]
fn keypair_from_seed_bytes_round_trip() {
    let original = Keypair::generate();
    let envelope = original.export_seed_envelope();
    let imported = Keypair::from_seed_bytes(&envelope).unwrap();
    assert_eq!(
        original.public_key().to_bytes(),
        imported.public_key().to_bytes()
    );
    let msg = b"round-trip-test";
    let sig = imported.sign(msg);
    assert!(original.public_key().verify(msg, &sig).is_ok());
}

#[test]
fn keypair_from_seed_bytes_rejects_short_input_with_typed_error() {
    let truncated = vec![0u8; 16];
    let err = Keypair::from_seed_bytes(&truncated).unwrap_err();
    let benten_id::KeypairError::SeedImport(import_err) = err else {
        panic!("expected SeedImport, got {err:?}");
    };
    assert!(
        matches!(import_err, SeedImportError::ShortInput { .. }),
        "expected ShortInput, got {import_err:?}"
    );
}

#[test]
fn keypair_from_seed_bytes_rejects_long_input_with_typed_error() {
    let extra = vec![0u8; 512];
    let err = Keypair::from_seed_bytes(&extra).unwrap_err();
    let benten_id::KeypairError::SeedImport(import_err) = err else {
        panic!("expected SeedImport, got {err:?}");
    };
    assert!(
        matches!(
            import_err,
            SeedImportError::LongInput { .. } | SeedImportError::EnvelopeMalformed
        ),
        "expected LongInput or EnvelopeMalformed, got {import_err:?}"
    );
}

#[test]
fn keypair_from_seed_bytes_rejects_corrupted_bytes() {
    let original = Keypair::generate();
    let mut envelope = original.export_seed_envelope();
    // Corrupt a byte in the middle of the envelope. CBOR is self-
    // describing — bit-flips in the type tag / length prefix corrupt
    // the structural decode.
    let len = envelope.len();
    // Flip a byte near the start (the version field or alg-string
    // header is a hot zone for structural rejection).
    envelope[len / 2] ^= 0xff;
    let err = Keypair::from_seed_bytes(&envelope).unwrap_err();
    let benten_id::KeypairError::SeedImport(import_err) = err else {
        panic!("expected SeedImport, got {err:?}");
    };
    assert!(
        matches!(
            import_err,
            SeedImportError::EnvelopeMalformed
                | SeedImportError::InvalidSecret
                | SeedImportError::UnknownVersion { .. }
                | SeedImportError::UnknownAlg { .. }
        ),
        "expected typed rejection, got {import_err:?}"
    );
}

#[test]
fn keypair_from_seed_bytes_rejects_unknown_version_tag() {
    // Hand-craft an envelope with version 99.
    #[derive(Serialize)]
    struct Envelope {
        version: u8,
        alg: String,
        secret_bytes: serde_bytes::ByteBuf,
    }
    let crafted = Envelope {
        version: 99,
        alg: "Ed25519".to_string(),
        secret_bytes: serde_bytes::ByteBuf::from(vec![0u8; 32]),
    };
    let bytes = serde_ipld_dagcbor::to_vec(&crafted).unwrap();
    let err = Keypair::from_seed_bytes(&bytes).unwrap_err();
    let benten_id::KeypairError::SeedImport(import_err) = err else {
        panic!("expected SeedImport, got {err:?}");
    };
    assert!(
        matches!(import_err, SeedImportError::UnknownVersion { version: 99 }),
        "expected UnknownVersion {{ version: 99 }}, got {import_err:?}"
    );
}

#[test]
fn keypair_import_path_does_not_log_seed_bytes_via_tracing() {
    // crypto-major-5 — the import path emits NO tracing events
    // containing the secret. We assert this via SOURCE-GREP: the
    // src/keypair.rs file's `from_seed_bytes` / `from_seed_bytes_inner`
    // MUST NOT contain `tracing::` macros that include seed bytes.
    //
    // This is the call-site grep variant of the contract; the
    // observable-consequence variant (capturing tracing spans during
    // a test run) requires a tracing-subscriber test-layer dep that
    // is intentionally NOT pulled in to keep the dep surface minimal.
    let src_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("keypair.rs");
    let src = std::fs::read_to_string(&src_path).unwrap();
    // Find the from_seed_bytes_inner fn body and assert no tracing
    // macros referencing seed/secret/bytes are present.
    let fn_idx = src
        .find("fn from_seed_bytes_inner")
        .expect("from_seed_bytes_inner present");
    // Scan ~2000 chars of fn body (generous; the fn is ~40 lines).
    let body = &src[fn_idx..src.len().min(fn_idx + 2000)];
    // Cover all 5 tracing severity levels (trace/debug/info/warn/error)
    // per crypto-major-5: the contract is "NO tracing events containing
    // secret bytes". A contributor adding `tracing::warn!("seed bytes: {seed:?}")`
    // on the EnvelopeMalformed path would otherwise slip past this audit.
    for tracing_macro in &[
        "tracing::trace!",
        "tracing::debug!",
        "tracing::info!",
        "tracing::warn!",
        "tracing::error!",
    ] {
        assert!(
            !body.contains(tracing_macro),
            "from_seed_bytes_inner MUST NOT emit {tracing_macro} (secret-bytes leak per crypto-major-5)"
        );
    }
}

#[test]
fn keypair_from_seed_bytes_envelope_round_trip() {
    // Qual-1 #686: was `keypair_from_dag_cbor_envelope_round_trip`,
    // exercising the now-removed `from_dag_cbor_envelope` no-op alias;
    // renamed + repointed to the canonical `from_seed_bytes`.
    let original = Keypair::generate();
    let envelope1 = original.export_seed_envelope();
    let imported = Keypair::from_seed_bytes(&envelope1).unwrap();
    let envelope2 = imported.export_seed_envelope();
    assert_eq!(
        envelope1, envelope2,
        "DAG-CBOR canonical bytes must be stable across export → import → re-export"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2_000))]

    #[test]
    fn prop_keypair_from_seed_bytes_arbitrary_input_no_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..512)
    ) {
        // The import path MUST handle ANY byte sequence without
        // panicking — Ok or Err but never crash.
        let result = Keypair::from_seed_bytes(&bytes);
        prop_assert!(result.is_ok() || result.is_err());
    }
}
