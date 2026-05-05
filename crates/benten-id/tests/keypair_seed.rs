//! R3-A RED-PHASE pins for `Keypair::from_seed_bytes` (G14-A1 wave-4a).
//!
//! Pin sources (per r2-test-landscape §2.2 G14-A1 + plan §3 G14-A1
//! must-pass column):
//!
//! - `tests/keypair_from_seed_bytes_round_trip` — plan §3 G14-A1
//! - `tests/keypair_from_seed_bytes_rejects_short_input_with_typed_error` — `crypto-major-5`
//! - `tests/keypair_from_seed_bytes_rejects_long_input_with_typed_error` — `crypto-major-5`
//! - `tests/keypair_from_seed_bytes_rejects_corrupted_bytes` — `crypto-major-5`
//! - `tests/keypair_from_seed_bytes_rejects_unknown_version_tag` — `crypto-major-5`
//! - `tests/keypair_import_path_does_not_log_seed_bytes_via_tracing` — `crypto-major-5`
//! - `tests/prop_keypair_from_seed_bytes_arbitrary_input_no_panic` — `crypto-major-5`
//! - `tests/keypair_from_dag_cbor_envelope_round_trip` — exploration-device-mesh
//!
//! The DAG-CBOR envelope schema per crypto-major-5:
//!
//! ```text
//! { version: u8, alg: 'Ed25519', secret_bytes: Bytes(32) }
//! ```
//!
//! Each test pins a SEPARATE failure mode of the import path (short
//! input, long input, corrupted bytes, unknown version tag). The
//! collective coverage closes crypto-major-5's "fuzz the import path
//! end-to-end" requirement.

#![allow(
    clippy::unwrap_used,
    unreachable_code,
    reason = "RED-PHASE stubs; G14-A1 implementer wires real bodies + drops these allows"
)]

use proptest::prelude::*;

#[test]
#[ignore = "RED-PHASE: G14-A1 — plan §3 G14-A1 — from_seed_bytes round-trip"]
fn keypair_from_seed_bytes_round_trip() {
    // G14-A1 implementer wires this against the real API:
    //   let original = benten_id::keypair::Keypair::generate();
    //   let envelope_bytes = original.export_seed_envelope().unwrap();
    //   let imported = benten_id::keypair::Keypair::from_seed_bytes(&envelope_bytes).unwrap();
    //   // Pubkey + signing behavior identical:
    //   assert_eq!(original.public_key().to_bytes(), imported.public_key().to_bytes());
    //   let msg = b"round-trip-test";
    //   let sig = imported.sign(msg);
    //   assert!(original.public_key().verify(msg, &sig).is_ok());
    //
    // OBSERVABLE consequence: an exported keypair re-imports to a
    // FUNCTIONALLY identical keypair (signs verifiably, same pubkey).
    unimplemented!("G14-A1 wires Keypair::from_seed_bytes round-trip via DAG-CBOR envelope");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-major-5 — short input rejected"]
fn keypair_from_seed_bytes_rejects_short_input_with_typed_error() {
    // crypto-major-5 pin. Implementer wires:
    //   let truncated = vec![0u8; 16];  // half-length: 16 bytes < 32
    //   let err = benten_id::keypair::Keypair::from_seed_bytes(&truncated).unwrap_err();
    //   // Typed error variant — NOT a generic dyn Error / String:
    //   assert!(matches!(err, benten_id::keypair::SeedImportError::ShortInput { .. }));
    //
    // OBSERVABLE consequence: short bytes return `Err(ShortInput)`,
    // not `Err(unknown)` and not `panic!`. Defense against accidental
    // truncation in transit / on-disk corruption.
    unimplemented!("G14-A1 wires typed-error assertion for short-input seed import path");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-major-5 — long input rejected"]
fn keypair_from_seed_bytes_rejects_long_input_with_typed_error() {
    // crypto-major-5 pin. Implementer wires:
    //   let extra_bytes = vec![0u8; 256];  // way too long
    //   let err = benten_id::keypair::Keypair::from_seed_bytes(&extra_bytes).unwrap_err();
    //   assert!(matches!(err, benten_id::keypair::SeedImportError::LongInput { .. })
    //         || matches!(err, benten_id::keypair::SeedImportError::EnvelopeMalformed));
    //
    // OBSERVABLE consequence: oversized envelope rejects with typed
    // variant. Defends against length-extension / payload-stuffing.
    unimplemented!("G14-A1 wires typed-error assertion for long-input seed import path");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-major-5 — corrupted bytes rejected"]
fn keypair_from_seed_bytes_rejects_corrupted_bytes() {
    // crypto-major-5 pin. Implementer wires:
    //   let original = benten_id::keypair::Keypair::generate();
    //   let mut bytes = original.export_seed_envelope().unwrap();
    //   // Corrupt one byte in the middle of the secret_bytes field.
    //   let len = bytes.len();
    //   bytes[len / 2] ^= 0xff;
    //   let err = benten_id::keypair::Keypair::from_seed_bytes(&bytes).unwrap_err();
    //   // Either the DAG-CBOR decoder catches it (CBOR is self-
    //   // describing; bit-flips inside Bytes(32) corrupt the type tag
    //   // or length prefix) OR the Ed25519 secret-byte validation does.
    //   assert!(matches!(err, benten_id::keypair::SeedImportError::EnvelopeMalformed)
    //         || matches!(err, benten_id::keypair::SeedImportError::InvalidSecret));
    //
    // OBSERVABLE consequence: corruption fails LOUDLY at import,
    // never returns a "successful but wrong" Keypair.
    unimplemented!("G14-A1 wires typed-error assertion for corrupted-bytes seed import path");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-major-5 — unknown version tag rejected"]
fn keypair_from_seed_bytes_rejects_unknown_version_tag() {
    // crypto-major-5 pin. The envelope schema includes a version
    // discriminant; future schema changes bump the version. Older
    // implementations encountering a newer version MUST reject
    // (forward-incompatibility is intentional — silent acceptance
    // would let an attacker mint envelopes that older verifiers
    // mis-parse).
    //
    // Implementer wires:
    //   // Hand-craft a CBOR envelope with version: 99 (unknown).
    //   let envelope = make_envelope_with_version(99);
    //   let err = benten_id::keypair::Keypair::from_seed_bytes(&envelope).unwrap_err();
    //   assert!(matches!(err, benten_id::keypair::SeedImportError::UnknownVersion { version: 99 }));
    //
    // OBSERVABLE consequence: future-version envelopes reject with a
    // typed variant carrying the version number for diagnostics.
    unimplemented!("G14-A1 wires typed-error assertion for unknown-version-tag seed import path");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-major-5 — import does not log seed bytes"]
fn keypair_import_path_does_not_log_seed_bytes_via_tracing() {
    // crypto-major-5 pin. Defends against accidental tracing/logging
    // of the secret bytes during import. Implementer wires this via a
    // `tracing-subscriber` test layer that captures every emitted
    // event into a buffer; assert the buffer contains NO sequence of
    // 32 contiguous bytes from the secret material.
    //
    // Concrete shape:
    //   let buf = capture_tracing_events();
    //   let _ = benten_id::keypair::Keypair::from_seed_bytes(&envelope);
    //   let logged = buf.contents();
    //   let secret_hex = hex::encode(secret_bytes);
    //   assert!(!logged.contains(&secret_hex),
    //       "import path leaked secret bytes via tracing");
    //
    // OBSERVABLE consequence: `RUST_LOG=trace cargo test ...` does not
    // dump the secret to stderr.
    unimplemented!("G14-A1 wires tracing-capture assertion that secret bytes never appear in logs");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — exploration-device-mesh — DAG-CBOR envelope round-trip"]
fn keypair_from_dag_cbor_envelope_round_trip() {
    // exploration-device-mesh brief-edit pin. The envelope is a
    // canonical DAG-CBOR object — bytes are stable across encoding
    // engines (any CBOR encoder following deterministic rules emits
    // the same byte sequence for the same logical envelope).
    //
    // Implementer wires:
    //   let original = benten_id::keypair::Keypair::generate();
    //   let envelope1 = original.export_seed_envelope().unwrap();
    //   let imported = benten_id::keypair::Keypair::from_dag_cbor_envelope(&envelope1).unwrap();
    //   let envelope2 = imported.export_seed_envelope().unwrap();
    //   assert_eq!(envelope1, envelope2,
    //       "DAG-CBOR canonical bytes must be stable across export → import → re-export");
    //
    // OBSERVABLE consequence: re-exporting an imported keypair
    // produces byte-identical bytes (canonical-bytes contract).
    unimplemented!("G14-A1 wires DAG-CBOR envelope canonical-bytes round-trip");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2_000))]

    #[test]
    #[ignore = "RED-PHASE: G14-A1 — crypto-major-5 — fuzz seed import path"]
    fn prop_keypair_from_seed_bytes_arbitrary_input_no_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..512)
    ) {
        // crypto-major-5 pin. The import path MUST handle arbitrary
        // (potentially adversarial) byte sequences without panicking.
        // 2 000 cases × variable-length up to 512 bytes covers:
        //   - empty input
        //   - too-short input
        //   - too-long input
        //   - random bytes (CBOR-malformed)
        //   - structurally-valid CBOR with wrong types
        //   - structurally-valid envelopes with corrupted secret_bytes
        //
        // Implementer wires:
        //   let result = benten_id::keypair::Keypair::from_seed_bytes(&bytes);
        //   // Either Ok (lucky valid envelope) or Err (typed). NEVER panic.
        //   prop_assert!(result.is_ok() || result.is_err());
        //
        // OBSERVABLE consequence: the import path never panics on any
        // input. Defense against denial-of-service via crafted input.
        let _ = bytes;
        unimplemented!("G14-A1 wires no-panic assertion for arbitrary-input seed import");
    }
}
