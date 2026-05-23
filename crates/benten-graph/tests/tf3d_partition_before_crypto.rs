//! TF-3d pins — G-CORE-3d × G-CORE-1 partition-before-crypto composition.
//!
//! Cross-wave §4-A test (R3-W3 owns the integration between
//! `namespace_did` partition and per-Node AEAD).
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition. Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3d (P-4):
//!     "Rides on G-CORE-1's `namespace_did` seam: encrypted Node visible
//!     only within owning DID's partition; cross-DID lookup returns
//!     `NotFound` (not 'AEAD-error' — partition isolation BEFORE crypto
//!     attempt). This is the load-bearing C1+C2 composition (multitenant-r1-5)."
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §4-A: "G-CORE-3 ×
//!     G-CORE-1 — SubgraphSpec read scoped to DID X cannot leak from
//!     DID Y partition. … partition-isolation invariant fires BEFORE
//!     crypto attempt … *Composes C1 (G-CORE-1) authority-isolation +
//!     C2 (G-CORE-3) confidentiality-isolation.*"
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §5: "Cross-DID leak
//!     (Inv-11 strengthening). Owner: G-CORE-1 (partition) + G-CORE-3d
//!     (per-Node AEAD with key rooted at K_principal). Test: write under
//!     DID-X; read/range/iterate/subscribe under DID-Y returns empty;
//!     AEAD-decrypt attempt with K_principal-Y on K_principal-X
//!     ciphertext fails."
//!   - `00-implementation-plan.md` §3 G-CORE-3 def: "The C1 cross-DID
//!     non-leak invariant is preserved + extended by the per-Node AEAD
//!     layer (Inv-11 strengthened: namespace_did partition + per-Node
//!     AEAD with key-derivation rooted at `K_principal`)."
//!
//! ============================================================================
//! LANDED at G-CORE-3d (pim-12 / §3.6e closure).
//! ============================================================================
//! This file is on-surface for the §3.13 per-test-static decomposition
//! cap (R2 §7 W3 + the partition-isolation surface list); zero shared
//! state.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::unnested_or_patterns)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::{RedbBackend, WriteContext};
// Production failure point (LANDED at G-CORE-3d).
use benten_graph::aead_wrap::{AeadError, decrypt};
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
    let backend = RedbBackend::create(dir.path().join("tf3d-partition.redb")).expect("open redb");
    (dir, backend)
}

// ---------------------------------------------------------------------------
// PIN 1 — Cross-DID lookup returns NotFound, NOT AEAD-error.
// ---------------------------------------------------------------------------
// Production-arm P-4 + §4-A cross-wave. DID-X writes Node N (encrypted
// under K_principal-X, stored at ciphertext_cid in X's partition). DID-Y
// attempts to read; the read fails at PARTITION LOOKUP, never reaching
// the AEAD layer. The error variant is `NotFound`, NOT
// `AeadAuthentication` — because Y's view doesn't even see X's
// ciphertext addresses.
//
// Would-FAIL-IF-NO-OP'd: if `read_via_two_cid` ignored
// `WriteContext::namespace_did`, Y would see the ciphertext, attempt
// decryption with K_principal-Y, and get an AEAD-error (the partition
// isolation invariant fires BEFORE crypto attempt).
#[test]

fn tf3d_cross_did_returns_notfound_not_aead_error() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let did_y = namespace_cid("did:key:zY");
    let ctx_x = WriteContext::new("Doc").with_namespace_did(Some(did_x.clone()));
    let ctx_y = WriteContext::new("Doc").with_namespace_did(Some(did_y));

    let node = node_titled("Alice's private recipe");
    let plaintext_cid = node.cid().unwrap();

    backend
        .put_node_with_context(&node, &ctx_x)
        .expect("X writes OK");

    // Y attempts read of plaintext_cid via the scoped view.
    let result_y = backend.read_via_two_cid_scoped(&plaintext_cid, &ctx_y);

    // Critical: the failure variant MUST be NotFound (partition isolation
    // fired BEFORE crypto attempt), NOT an AEAD-error (which would imply
    // Y saw the ciphertext bytes and tried decrypting them).
    use benten_graph::two_cid_map::TwoCidMapError;
    match result_y {
        Err(TwoCidMapError::NotFound { .. }) => {
            // Expected: partition isolation fired first.
        }
        Err(TwoCidMapError::AeadAuthenticationFailed { .. }) => {
            panic!(
                "Y's read of X's content returned an AEAD error, meaning \
                 Y saw the ciphertext bytes. This means partition \
                 isolation did NOT fire BEFORE the crypto layer — \
                 violation of C1+C2 composition (multitenant-r1-5)."
            );
        }
        other => {
            panic!(
                "Expected NotFound (partition isolation fired first), got: {:?}",
                other.as_ref().map(|_| "Ok(_)").err()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// PIN 2 — AEAD-decrypt attempt with K_principal-Y on K_principal-X
// ciphertext fails (defense-in-depth).
// ---------------------------------------------------------------------------
// Even if an attacker somehow bypasses the partition layer and obtains
// X's raw ciphertext bytes (e.g. by reading the redb file directly),
// attempting decryption with K_principal-Y MUST fail at the AEAD layer.
// This is the DEFENSE-IN-DEPTH property — partition is the first line;
// per-Node AEAD with key-derivation rooted at K_principal is the second.
#[test]

fn tf3d_wrong_principal_key_fails_aead_defense_in_depth() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let ctx_x = WriteContext::new("Doc").with_namespace_did(Some(did_x.clone()));

    let node = node_titled("X's private recipe");
    backend
        .put_node_with_context(&node, &ctx_x)
        .expect("X writes OK");

    // Attacker has X's raw ciphertext (e.g. via direct redb file read).
    let plaintext_cid = node.cid().unwrap();
    let encrypted = backend
        .get_encrypted_node_unscoped_for_test(&plaintext_cid)
        .expect("test seam: bypass partition layer");

    // Attempt decryption with K_principal-Y (a different DID's key).
    let k_principal_y = namespace_cid("did:key:zY-key-material");
    let result = decrypt(&encrypted, &k_principal_y.as_bytes_for_test());

    assert!(
        matches!(
            result,
            Err(AeadError::Authentication { .. }) | Err(AeadError::TagMismatch { .. })
        ),
        "AEAD-decrypt with K_principal-Y (wrong key) on K_principal-X \
         ciphertext MUST fail. This is the defense-in-depth property: \
         even if partition is bypassed, per-Node AEAD with K rooted at \
         K_principal still blocks cross-DID decryption. got: {:?}",
        result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — DropBundle of Y's Recipes installed under X's namespace fails.
// ---------------------------------------------------------------------------
// Adversarial §4-A: produce a DropBundle of Y's Recipes; attempt to
// install/render it under X's namespace; assert AEAD authentication
// AND/OR partition routing rejects. The DropBundle import path
// integrates two-CID + AEAD with the namespace_did scope of the
// installing engine — the bundle's ciphertexts (keyed at Y's K_principal)
// CANNOT decrypt under X's principal context.
#[test]

fn tf3d_dropbundle_of_y_installed_under_x_namespace_rejected() {
    // Note: the DropBundle install path lives in benten-drop (G-CORE-3f).
    // This pin documents the cross-wave composition; the substantive
    // behavior is owned by the install pipeline which lives elsewhere.
    // The pin asserts the failure HAPPENS — exact error variant is
    // implementation-defined at G-CORE-3d/3f integration time.
    let (_dir, backend) = fresh_backend();
    let did_y = namespace_cid("did:key:zY");
    let did_x = namespace_cid("did:key:zX");
    let ctx_x = WriteContext::new("Doc").with_namespace_did(Some(did_x.clone()));

    // Y produces a synthetic ciphertext (would be a DropBundle).
    let y_node = node_titled("Y's secret recipe");
    let y_plaintext_cid = y_node.cid().unwrap();
    let y_key = did_y.as_bytes_for_test();
    let y_bytes = y_node.to_canonical_bytes().unwrap();
    let y_encrypted: benten_graph::aead_wrap::EncryptedNode =
        benten_graph::aead_wrap::EncryptedNode::encrypt(&y_bytes, &y_plaintext_cid, &y_key)
            .expect("Y encrypt OK");

    // Attempt to "install" Y's ciphertext into X's partition by importing
    // it under ctx_x. The integration point (the import path) MUST refuse —
    // either at partition routing or AEAD layer.
    let result = backend.import_encrypted_node_into_scope(&y_encrypted, &y_plaintext_cid, &ctx_x);

    assert!(
        result.is_err(),
        "Importing Y's ciphertext into X's namespace MUST be refused. \
         Either at partition routing (the ciphertext_cid is foreign to \
         X's keyspace) or at AEAD (X cannot derive Y's per-Node key from \
         K_principal-X). NEVER silent success — this would be a \
         cross-DID confidentiality leak via the install path."
    );
}
