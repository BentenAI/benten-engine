//! TF-3d pins — G-CORE-3d two-CID mapping table round-trip.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition (graph-AEAD + sync + drop).
//! Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3d (P-1)
//!     "Two-CID mapping round-trip: write Node N with plaintext_cid CID_p
//!     → AEAD-wrap → store at ciphertext_cid CID_c →
//!     `mapping_table.lookup(CID_p) == CID_c` AND `read(CID_c) → AEAD
//!     ciphertext` AND `decrypt(ciphertext, K(N)) == N`."
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §6 (15) "Two-CID mapping
//!     (plaintext_cid ↔ ciphertext_cid) round-trips end-to-end (UCAN scope
//!     check against plaintext_cid; iroh-blobs serves ciphertext_cid)."
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3d wave def: "per-Node
//!     AEAD wrap layer riding on G-CORE-1's `WriteContext::namespace_did`
//!     seam + two-CID mapping table (`plaintext_cid → ciphertext_cid`,
//!     redb-backed)".
//!   - §1.A.FROZEN item 15 sub-clause (g) — two-CID mapping contract is a
//!     frozen public-surface item.
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2 (two-CID
//!     mapping ratified by R2 + Spike H+1.2).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3d (pim-12 / §3.6e).
//! ============================================================================
//! `benten_graph::{aead_wrap, two_cid_map}` modules + the
//! `RedbBackend::two_cid_lookup` extension do NOT exist at origin/main
//! `c9c11c56` (ground-truth: `grep -rn aead_wrap crates/benten-graph/src`
//! returns nothing; the mapping table is unimplemented). So this file
//! **compiles-but-fails at the `use`/symbol-resolution line** until
//! G-CORE-3d lands. Each `#[test]` is `#[ignore]`-staged with the
//! literal marker `RED-PHASE: un-ignore at G-CORE-3d`. The G-CORE-3d
//! closing-wave reviewer MUST verify these pins are *un-ignored*
//! (landing-status, not just spec-pin presence) per §3.6e.
//!
//! ----------------------------------------------------------------------------
//! §3-directive inherited-discipline pre-flight (this file ticks every line):
//!  - §3.6b + sub-rule 4: each pin is a PRODUCTION-ARM (the real
//!    `RedbBackend::put_node_with_context` extended with two-CID mapping +
//!    `RedbBackend::two_cid_lookup` + `aead_wrap::decrypt`) +
//!    OBSERVABLE-CONSEQUENCE (the plaintext recovered byte-identical;
//!    `lookup(plaintext_cid)` returns the correct `ciphertext_cid`) +
//!    WOULD-FAIL-IF-NO-OP'd (a stub that stores plaintext or skips
//!    the mapping trips every assertion). Each pin targets the SPECIFIC
//!    arm (lookup / read / decrypt), not an umbrella "two-CID works".
//!  - §3.6f (pim-18) SHAPE-not-SUBSTANCE: every pin enumerates a real
//!    production call site (`RedbBackend::put_node_with_context` +
//!    `RedbBackend::two_cid_lookup` + `aead_wrap::decrypt`) and asserts
//!    an observable byte-level consequence — NONE is "assert a TwoCidMap
//!    type is constructible".
//!  - §3.13 per-test static decomposition: this file introduces NO
//!    process-scoped shared static. Every test owns a fresh `tempdir()`
//!    + fresh `RedbBackend`.
//!  - §3.6e (pim-12): `#[ignore]` + literal `RED-PHASE: un-ignore at
//!    G-CORE-3d` on every test.
//!  - §3.5g cross-language: no TS surface in this file (the two-CID
//!    mapping is engine-internal; UCAN scope check against plaintext_cid
//!    happens at G-CORE-3b/3e where the TS mirror lives).
//!  - §3.5n: this file pins behaviors against the G-CORE-3d wave-def in
//!    `00-implementation-plan.md` R0.8.1 §3 + the §1.A.FROZEN item 15(g)
//!    contract. Both verified to be in the plan at write-time.
//! ----------------------------------------------------------------------------

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::{RedbBackend, WriteContext};
// RED-PHASE failure point: intended G-CORE-3d two-CID + AEAD wrap surface.
// Does NOT exist at c9c11c56 → compile-but-fail here.
use benten_graph::aead_wrap::{AeadError, EncryptedNode, decrypt, encrypt};
use benten_graph::two_cid_map::{TwoCidMap, TwoCidMapError};
use tempfile::tempdir;

fn node_titled(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".to_string(), Value::text(title));
    Node::new(vec!["Doc".to_string()], props)
}

fn namespace_cid(seed: &str) -> Cid {
    let mut props = BTreeMap::new();
    props.insert("did-seed".to_string(), Value::text(seed));
    Node::new(vec!["system:Principal".to_string()], props)
        .cid()
        .expect("namespace seed node must hash")
}

fn fresh_backend() -> (tempfile::TempDir, RedbBackend) {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("tf3d.redb")).expect("open redb");
    (dir, backend)
}

// ---------------------------------------------------------------------------
// PIN 1 — two-CID round-trip: put_node → mapping_table.lookup → decrypt.
// ---------------------------------------------------------------------------
// Production-arm (P-1): write Node N with plaintext_cid CID_p →
// AEAD-wrap → store at ciphertext_cid CID_c →
// `mapping_table.lookup(CID_p) == CID_c` AND `read(CID_c) → AEAD
// ciphertext` AND `decrypt(ciphertext, K(N)) == N`.
//
// Would-FAIL-IF-NO-OP'd: if `put_node_with_context` writes plaintext at
// `plaintext_cid` (ignoring the AEAD-wrap layer), `two_cid_lookup`
// returns None and `decrypt` fails on plaintext bytes.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3d"]
fn tf3d_two_cid_mapping_round_trip_lookup_and_decrypt() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let ctx = WriteContext::new("Doc").with_namespace_did(Some(did_x));

    let node = node_titled("Recipe: tomato soup");
    let plaintext_cid = node.cid().unwrap();

    // RED-PHASE production surface: `put_node_with_context` wraps the
    // node in AEAD, computes the ciphertext_cid, stores AT the
    // ciphertext_cid, and records the mapping plaintext_cid → ciphertext_cid.
    let ciphertext_cid = backend
        .put_node_with_context(&node, &ctx)
        .expect("put_node_with_context must succeed");

    // The plaintext_cid is NOT what the storage layer keys on; the
    // two-CID mapping table records the correspondence.
    assert_ne!(
        plaintext_cid, ciphertext_cid,
        "ciphertext_cid (storage key) MUST differ from plaintext_cid \
         (UCAN-scope key) — the whole point of the two-CID mapping is \
         that they are distinct addresses for the same logical Node."
    );

    // Lookup the mapping table: plaintext_cid → ciphertext_cid.
    let looked_up = backend
        .two_cid_lookup(&plaintext_cid)
        .expect("two_cid_lookup must succeed");
    assert_eq!(
        looked_up,
        Some(ciphertext_cid.clone()),
        "two_cid_lookup(plaintext_cid) MUST return the ciphertext_cid \
         that the backend wrote AEAD-wrapped bytes to."
    );

    // Decrypt the AEAD ciphertext via the key derived from K(N). The
    // decrypted plaintext canonical-bytes MUST round-trip to the
    // original Node.
    let encrypted: EncryptedNode = backend
        .get_encrypted_node(&ciphertext_cid)
        .expect("get_encrypted_node must succeed");
    let decrypted = decrypt(&encrypted, &node.derive_key_for_test()).expect("AEAD decrypt OK");
    let round_trip_node: Node = serde_ipld_dagcbor::from_slice(&decrypted).expect("dagcbor OK");
    assert_eq!(
        round_trip_node.cid().unwrap(),
        plaintext_cid,
        "decrypted plaintext bytes must rehydrate to the original Node \
         with byte-identical plaintext_cid (the round-trip property)."
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — mapping table missing entry returns None, not error.
// ---------------------------------------------------------------------------
// Fail-closed arm: `two_cid_lookup` on a never-written plaintext_cid
// returns `Ok(None)`, NOT an error. Distinguishes "not in mapping" from
// "mapping subsystem broken".
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3d"]
fn tf3d_two_cid_lookup_missing_returns_ok_none() {
    let (_dir, backend) = fresh_backend();
    let phantom_plaintext = namespace_cid("did:key:phantom-not-written");

    let result = backend.two_cid_lookup(&phantom_plaintext);
    assert!(
        matches!(result, Ok(None)),
        "two_cid_lookup on never-written plaintext_cid MUST return \
         Ok(None) (semantically 'not in mapping'), NOT Err (which means \
         the mapping subsystem itself is broken). got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — mapping is recoverable across backend re-open (durable).
// ---------------------------------------------------------------------------
// The redb-backed mapping table MUST persist across process restart;
// otherwise UCAN scope (against plaintext_cid) cannot resolve to served
// bytes (ciphertext_cid) after a restart. Would-FAIL-IF-NO-OP'd if the
// mapping table lives in RAM only.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3d"]
fn tf3d_two_cid_mapping_durable_across_reopen() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("tf3d-durable.redb");
    let did_x = namespace_cid("did:key:zX");
    let ctx = WriteContext::new("Doc").with_namespace_did(Some(did_x));

    let node = node_titled("Persisted recipe");
    let plaintext_cid = node.cid().unwrap();

    let ciphertext_cid = {
        let backend = RedbBackend::create(&path).expect("open redb");
        backend
            .put_node_with_context(&node, &ctx)
            .expect("put_node_with_context must succeed")
    };

    // Re-open the same redb file.
    let reopened = RedbBackend::create(&path).expect("re-open redb");
    let looked_up = reopened
        .two_cid_lookup(&plaintext_cid)
        .expect("two_cid_lookup must succeed across re-open");
    assert_eq!(
        looked_up,
        Some(ciphertext_cid),
        "two-CID mapping MUST be durable across redb re-open. A \
         non-durable mapping would silently break sync after a restart \
         (UCAN scope -> served bytes resolution fails)."
    );
}

// ---------------------------------------------------------------------------
// PIN 4 — mapping-table tamper detection (A-3 adversarial).
// ---------------------------------------------------------------------------
// Adversarial (A-3 in §2 G-CORE-3d): corrupt the
// plaintext_cid → ciphertext_cid row by pointing it at a foreign
// ciphertext. The subsequent `read` returns ciphertext that fails its
// AAD-binds-plaintext-CID assertion → typed mapping integrity error,
// never silent acceptance of foreign content.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3d"]
fn tf3d_mapping_table_tamper_yields_typed_integrity_error() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let ctx = WriteContext::new("Doc").with_namespace_did(Some(did_x));

    let node_a = node_titled("Original recipe A");
    let node_b = node_titled("Foreign recipe B");
    let plaintext_a = node_a.cid().unwrap();
    let plaintext_b = node_b.cid().unwrap();
    let ciphertext_a = backend.put_node_with_context(&node_a, &ctx).unwrap();
    let ciphertext_b = backend.put_node_with_context(&node_b, &ctx).unwrap();

    // Adversarial: tamper the mapping so plaintext_a points at
    // ciphertext_b. (Test-only seam; in production this would require
    // bypassing the integrity-gated put path.)
    backend
        .tamper_mapping_for_test(&plaintext_a, &ciphertext_b)
        .expect("test-only tamper seam");

    // Reading via the tampered mapping MUST fail with a typed mapping
    // integrity error (the AAD binds the plaintext_cid, so AEAD
    // authentication fails on the wrong-CID combination).
    let result = backend.read_via_two_cid(&plaintext_a);
    assert!(
        matches!(
            result,
            Err(TwoCidMapError::IntegrityMismatch { .. })
                | Err(TwoCidMapError::AeadAuthenticationFailed { .. })
        ),
        "Tampered mapping (plaintext_a → ciphertext_b) MUST yield a \
         typed integrity / AEAD-authentication error, NEVER silently \
         return ciphertext_b's plaintext for plaintext_a. got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}
