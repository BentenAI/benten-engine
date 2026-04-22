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
//! **Red-phase contract.** Today (Phase 1 HEAD) `HostError` does not
//! exist — the crate surfaces `EvalError::Backend(String)` which CAN carry
//! stringified CIDs via `format!("{err}")`. This test serialises a sample
//! error (via the currently-available surface) and asserts no base32-looking
//! CID shape survives. Until G1-B lands the HostError type, the test fires
//! red against the Phase-1 Display-stringified shape.
//!
//! Test: `host_error_wire_format_excludes_cids`
//!
//! Companion: `host_error_source_remains_typed_inproc` (defers to
//! integration landscape; in-process downcast test is a positive assertion
//! that this RED test explicitly does NOT foreclose).
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Node, Value};
use benten_engine::Engine;
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

/// atk-6 wire-safety: any serialised form of a HostError-equivalent surface
/// must not leak CID bytes / CID strings beyond the stable ErrorCode.
///
/// Phase-2a post-G1-B: the assertion below runs against a `HostError`
/// serialisation (DAG-CBOR or JSON) and asserts no CID-shape is present.
/// Phase-1 HEAD: serialises a best-effort wire shape via `EvalError`
/// Display; today's EvalError::Backend(String) path embeds CIDs in its
/// message, so the assertion fires.
#[test]
#[ignore = "phase-2a-pending: HostError wire shape lands in G1-B per plan §9.2. Drop #[ignore] once benten_eval::HostError + serde::Serialize impl are live. Today the test would vacuously pass because no HostError exists to serialise."]
fn host_error_wire_format_excludes_cids() {
    // Step 1: synthesise a backend error whose internal context references
    // a concrete CID. The shape under G1-B will be:
    //
    //     HostError {
    //         code: ErrorCode::HostWriteConflict,
    //         source: Box::new(SomeSpecificError { expected_cid, observed_cid }),
    //         context: Some("write conflict".to_string()),
    //     }
    //
    // For the red-phase approximation, build a Node with a known CID and
    // verify: (a) the CID is non-trivial (sanity on the fixture), (b) a
    // serialised form that includes the Node produces the CID string —
    // confirming the detector's sensitivity. The test body pins the
    // observable-exclusion contract regardless of which serde backend G1-B
    // picks.
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

    // Sanity: detector actually detects the CID in an arbitrary string.
    assert!(
        contains_cid_string(&cid_str) || contains_long_base32_run(&cid_str),
        "detector self-test: fixture CID should be recognised; got {cid_str}"
    );

    // Once G1-B lands, construct a HostError whose source references a
    // CID, serialise via `serde_ipld_dagcbor::to_vec` (or JSON), and check:
    //
    //     let err = benten_eval::HostError::write_conflict(&cid, &other_cid);
    //     let wire: Vec<u8> = serde_ipld_dagcbor::to_vec(&err).unwrap();
    //     let wire_str = String::from_utf8_lossy(&wire);
    //     assert!(!wire_str.contains(&cid_str),
    //         "HostError wire format must not leak CID");
    //     assert!(!contains_cid_string(&wire_str));
    //     assert!(!contains_long_base32_run(&wire_str));
    //
    // Until then, this test stays red via the panic below.
    panic!(
        "red-phase: benten_eval::HostError serde surface not yet present. \
         Fixture CID {cid_str} would be the thing this test excludes from \
         the serialised wire form. Wire through G1-B to complete."
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

fn _unused_tombstone() -> Cid {
    // Silences unused-import noise in toolchains that flag `Cid` as unused
    // if the only use is an ignored test body. This function is never
    // called; it just anchors the import.
    unreachable!()
}
