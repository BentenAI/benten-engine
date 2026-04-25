//! Phase 2a R3 security — HostError wire-format CID leak (atk-6 / sec-r1-6).
//!
//! R4 qa-r4-10 cross-reference: R2 §4.9 lists this under
//! `crates/benten-engine/tests/integration/host_error_wire_safety.rs`. The
//! file-split is kept; this header names the R2 anchor.
//!
//! **Attack class.** Option B's typed-enum HostError would serialise CID-
//! level context across the Phase-3 sync wire (e.g. `HostError::
//! WriteConflict { expected_cid, observed_cid }`). Wire-observer learns
//! CIDs they did not otherwise know exist — re-opens existence-leak
//! Compromise #2 via a sibling surface.
//!
//! **Prerequisite (attacker capability).** Network observer on the Phase-3
//! sync transport, or any operator holding the serialised error bytes
//! (backup, log, crash report).
//!
//! **Attack sequence.**
//!  1. A sync-mediated write surfaces a typed `HostError`.
//!  2. The error is serialised across the wire (DAG-CBOR or JSON).
//!  3. Observer inspects bytes, extracts any CID-shaped field — sees CIDs
//!     for Nodes they hold no read grant on.
//!
//! **Impact.** Existence leak of arbitrary Nodes, violating Option C
//! symmetric-None contract.
//!
//! **Recommended mitigation.** Lock Option A (opaque `Box<dyn StdError>` +
//! stable `ErrorCode`) at §9.2. Serialised form MUST contain ONLY the
//! stable `ErrorCode` string and an optional human-readable `context`;
//! `source` stays opaque and is NEVER part of the wire format. CID bytes
//! (32-byte BLAKE3 multihash, base32-lower CIDv1 string form) must not
//! appear in the serialised bytes beyond the code string.
//!
//! **Phase-2a green-phase contract.** G1-B landed `benten_eval::HostError`
//! (Option A): a `code: ErrorCode` + `source: Box<dyn StdError + Send +
//! Sync>` + `context: Option<String>`. The `to_wire_bytes` / `from_wire_bytes`
//! pair is the wire-format surface; `source` is intentionally NOT serialised
//! (atk-6 contract). This test exercises the round-trip and asserts no CID
//! bytes / CID strings appear in the serialised output even when the opaque
//! `source` carries them.
//!
//! Test: `host_error_wire_format_excludes_cids`
//!
//! Companion: `host_error_source_remains_typed_inproc` (defers to
//! integration landscape; in-process downcast test is a positive assertion
//! that this RED test explicitly does NOT foreclose).
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).
//! Phase 2a G11-A Wave-3b EXPECTED-RED-CLOSEOUT: un-ignored + flesh-out.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use benten_errors::ErrorCode;
use benten_eval::HostError;
use std::collections::BTreeMap;

/// Check if `s` contains a CIDv1 base32-lower string (common prefix "bafy").
/// CIDs produced by `Cid::to_string()` in Benten's Phase-1 fixture are
/// `bafyr4i...` — the prefix is the load-bearing observable.
fn contains_cid_string(s: &str) -> bool {
    // CIDv1 + BLAKE3 + DAG-CBOR starts with "bafyr" in base32-lower.
    s.contains("bafy")
}

/// Check if `s` contains a multihash-shaped sequence of bytes or base32
/// letters after the optional `cid:` marker. Conservative: anything that
/// looks like a 40+ character lowercase base32 run.
fn contains_long_base32_run(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut run = 0usize;
    for c in &chars {
        if (c.is_ascii_lowercase() || c.is_ascii_digit()) && !matches!(c, '0' | '1' | '8' | '9') {
            run += 1;
            if run >= 40 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

/// atk-6 wire-safety: any serialised form of a `HostError` must not leak CID
/// bytes / CID strings beyond the stable `ErrorCode`. The test:
///   1. Persists a Node so we have a real CID with the canonical
///      `bafyr4i…` shape.
///   2. Builds a `HostError` whose opaque `source` carries that CID inside
///      its `Display` string — i.e. an adversarial source that would, if
///      ever serialised, leak the CID over the wire.
///   3. Round-trips the error through `to_wire_bytes` /
///      `from_wire_bytes` (the official wire surface).
///   4. Asserts the encoded bytes contain neither the CID string nor any
///      long base32 run, AND that the decoded `HostError` reports the same
///      stable `code` plus the originally-passed `context`.
///
/// The opaque `source` MUST NOT survive the round-trip: it is replaced by a
/// generic decoded-from-wire placeholder. That property is also asserted.
#[test]
fn host_error_wire_format_excludes_cids() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let mut props = BTreeMap::new();
    props.insert("id".into(), Value::Text("test".into()));
    let node = Node::new(vec!["doc".into()], props);
    let cid = engine.create_node(&node).unwrap();
    let cid_str = cid.to_string();

    // Sanity: detector actually detects the CID in an arbitrary string. If
    // this fails the rest of the assertions are vacuous.
    assert!(
        contains_cid_string(&cid_str) || contains_long_base32_run(&cid_str),
        "detector self-test: fixture CID should be recognised; got {cid_str}"
    );

    // Adversarial `source`: a std::io::Error whose Display string EMBEDS
    // the live CID. If `to_wire_bytes` ever decided to include the source
    // (it must not), the wire bytes would carry the CID.
    let leaky_source = std::io::Error::other(format!(
        "backend write conflict at cid={cid_str}; do not leak me"
    ));
    let err = HostError {
        code: ErrorCode::HostWriteConflict,
        source: Box::new(leaky_source),
        context: Some("write conflict observed during commit".to_string()),
    };

    // Round-trip via the wire surface.
    let wire: Vec<u8> = err.to_wire_bytes().expect("to_wire_bytes must encode");
    let wire_str = String::from_utf8_lossy(&wire);

    // Core wire-safety assertions (atk-6 / sec-r1-6).
    assert!(
        !wire_str.contains(&cid_str),
        "HostError wire format leaked the literal CID string: bytes={wire_str:?}"
    );
    assert!(
        !contains_cid_string(&wire_str),
        "HostError wire format contains a `bafy*` CID-shape: bytes={wire_str:?}"
    );
    assert!(
        !contains_long_base32_run(&wire_str),
        "HostError wire format contains a long base32 run that could be a multihash: \
         bytes={wire_str:?}"
    );

    // Affirmative wire content: the stable code + context must round-trip.
    let decoded =
        HostError::from_wire_bytes(&wire).expect("from_wire_bytes must decode the round-trip");
    assert_eq!(
        decoded.code,
        ErrorCode::HostWriteConflict,
        "stable ErrorCode must round-trip through wire encode/decode"
    );
    assert_eq!(
        decoded.context.as_deref(),
        Some("write conflict observed during commit"),
        "context must round-trip through wire encode/decode"
    );

    // Opacity contract: the decoded `source` MUST NOT carry the leaky CID
    // text. The wire-format strips `source`; decode replaces it with a
    // generic placeholder. We assert the placeholder Display does not
    // contain the CID — defence-in-depth against a future regression that
    // re-creates the source from wire-side data.
    let decoded_source_str = format!("{}", decoded.source);
    assert!(
        !decoded_source_str.contains(&cid_str),
        "decoded HostError source string leaked CID: {decoded_source_str:?}"
    );
    assert!(
        !contains_cid_string(&decoded_source_str) && !contains_long_base32_run(&decoded_source_str),
        "decoded HostError source string contains CID-shape: {decoded_source_str:?}"
    );
}

/// Structural companion to ensure the detector doesn't vacuously pass on
/// CID-free strings. Runs always (not ignored) so detector regressions are
/// caught independently of G1-B landing.
#[test]
fn wire_safety_detector_is_not_vacuous() {
    // A string with NO CIDs must not trigger the detector.
    let clean = r#"{"code":"E_HOST_WRITE_CONFLICT","context":"write conflict"}"#;
    assert!(
        !contains_cid_string(clean) && !contains_long_base32_run(clean),
        "detector false-positive: {clean:?}"
    );

    // A string that DOES embed a CID must trigger.
    let dirty = r#"{"expected":"bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda"}"#;
    assert!(
        contains_cid_string(dirty) || contains_long_base32_run(dirty),
        "detector false-negative: failed to catch CID in {dirty:?}"
    );
}
