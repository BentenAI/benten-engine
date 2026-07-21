//! D-74/75/76 — enforcing secret-hygiene roster meta-test.
//!
//! This is the ENFORCING sweep for the pull-forward-#3 secret-hardening work:
//! it holds the whole roster of secret-bearing types across `benten-crypto-
//! suite`, `benten-id`, and `benten-engine` against two contracts —
//!
//!   (a) **Debug-does-not-leak** (runtime): for every secret type that has a
//!       `Debug` impl, construct it with a DISTINCTIVE fixture byte pattern,
//!       `format!("{:?}")` it, and assert the raw secret bytes are ABSENT from
//!       the rendered string (a derived/leaking `Debug` renders a `[u8; N]` as
//!       a decimal array literal `222, 173, 190, 239, ...`; the redacting
//!       impls render `<redacted>` / `[REDACTED]` instead). The would-FAIL-on-
//!       revert mode: if any type reverts to `#[derive(Debug)]`, the decimal
//!       marker re-appears and the matching assertion fires.
//!
//!   (b) **Zeroize coverage** (source-anchored): for every secret type — INCL.
//!       the ones with private fields + no public constructor reachable from an
//!       integration test — assert the drop-time-wipe wiring is present in the
//!       source (a `Drop` impl that `zeroize()`s the secret field, a
//!       `#[derive(ZeroizeOnDrop)]`, a `Zeroizing` wrapper, or the upstream
//!       `zeroize` cargo feature). Observing freed heap post-drop is UB, so
//!       the coverage half is a source-grep regression-defense (mirrors the
//!       repo's existing grep-defense pins, e.g.
//!       `benten-caps/tests/cap_r1_1_audience_binding_grep_defense.rs`).
//!
//! Requires `test-helpers` (reaches `benten-crypto-suite/testing` +
//! `benten-id/testing` for the fixture accessors; registered
//! `required-features = ["test-helpers"]` in `benten-engine/Cargo.toml`).

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use benten_crypto_suite::vault::VaultPayload;
use benten_engine::layer_d::device_link::ProvisioningInnerPayload;
use benten_engine::layer_d::remote_permission::{
    AAD_VERSION, PermissionOperation, PermissionRequest, REMOTE_PERMISSION_WIRE_VERSION,
};
use benten_id::keypair::Keypair;

/// The distinctive 4-byte secret marker. Chosen so its decimal rendering
/// (`222, 173, 190, 239`) cannot collide with a benign small-integer field.
const MARKER: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
/// The decimal-array rendering a LEAKING (derived) `Debug` would emit for a
/// `[u8; N]` whose leading bytes are `MARKER`.
const LEAK_DECIMAL: &str = "222, 173, 190, 239";

/// A 32-byte secret whose first 4 bytes are the distinctive marker.
fn marked_32() -> [u8; 32] {
    let mut b = [0u8; 32];
    b[..4].copy_from_slice(&MARKER);
    b
}

/// Read a crate source file by workspace-relative path (the roster spans three
/// crates; `CARGO_MANIFEST_DIR` here is `crates/benten-engine`, so we hop up to
/// the workspace root then down into the target crate).
fn read_crate_source(crate_dir: &str, rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates")
        .join(crate_dir)
        .join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

// =====================================================================
// (a) Debug-does-not-leak — runtime, for the publicly-constructible types
// =====================================================================

#[test]
fn vault_payload_debug_redacts_k_principal_and_user_did_signing_key() {
    let payload = VaultPayload {
        k_principal: marked_32(),
        user_did_signing_key: marked_32().to_vec(),
        user_did_creation_time: 1_900_000_800,
    };
    let rendered = format!("{payload:?}");
    assert!(
        !rendered.contains(LEAK_DECIMAL),
        "VaultPayload Debug MUST redact k_principal + user_did_signing_key \
         (D-74/75/76). A derived Debug leaks the raw [u8; 32] as `{LEAK_DECIMAL}`; \
         rendered=`{rendered}`"
    );
    assert!(
        rendered.contains("<redacted>"),
        "VaultPayload Debug MUST render `<redacted>` for the secret fields; \
         rendered=`{rendered}`"
    );
}

#[test]
fn provisioning_inner_payload_debug_redacts_secrets() {
    let inner = ProvisioningInnerPayload {
        k_principal: marked_32(),
        user_did_signing_key: marked_32(),
        user_did_pubkey: [0x33; 32],
        atrium_memberships: vec![[0x44; 32]],
        provisioning_session_id: [0x55; 32],
        granted_at_bucket: 1_900_000_800,
    };
    let rendered = format!("{inner:?}");
    assert!(
        !rendered.contains(LEAK_DECIMAL),
        "ProvisioningInnerPayload Debug MUST redact k_principal + \
         user_did_signing_key (D-74/75/76); rendered=`{rendered}`"
    );
    assert!(
        rendered.contains("[REDACTED]"),
        "ProvisioningInnerPayload Debug MUST render `[REDACTED]` for the secret \
         fields; rendered=`{rendered}`"
    );
}

#[test]
fn permission_request_debug_redacts_ephemeral_signing_key() {
    let req = PermissionRequest {
        aad_version: AAD_VERSION,
        version: REMOTE_PERMISSION_WIRE_VERSION,
        request_id: [0x11; 16],
        requesting_device_did: b"did:key:zBenign".to_vec(),
        requesting_device_pubkey: [0x22; 32],
        operation: PermissionOperation::RemoteUnlock,
        reason: b"benign-reason".to_vec(),
        timestamp_bucket: 1_900_000_800,
        ephemeral_signing_key: marked_32(),
        nonce: [0x66; 32],
        signature: Vec::new(),
    };
    let rendered = format!("{req:?}");
    assert!(
        !rendered.contains(LEAK_DECIMAL),
        "PermissionRequest Debug MUST redact ephemeral_signing_key (D-74/75/76). \
         A derived Debug leaks the raw [u8; 32] as `{LEAK_DECIMAL}`; \
         rendered=`{rendered}`"
    );
    assert!(
        rendered.contains("<redacted>"),
        "PermissionRequest Debug MUST render `<redacted>` for ephemeral_signing_key; \
         rendered=`{rendered}`"
    );
    // Positive: a NON-secret field still renders (Debug isn't a blanket-redact).
    assert!(
        rendered.contains("did:key:zBenign") || rendered.contains("requesting_device_did"),
        "PermissionRequest Debug should still render non-secret fields; \
         rendered=`{rendered}`"
    );
}

#[test]
fn keypair_surface_never_renders_the_live_secret_seed() {
    // `SecretKey` has no public byte-ctor, so we cannot inject a marker; the
    // redaction contract for `SecretKey::Debug` is source-anchored in
    // `id_secret_key_and_keypair_hygiene_present`. Here we exercise a LIVE key:
    // generate a real keypair, read its true secret seed via the sanctioned
    // (ungated) production accessor, and assert that seed's byte-sequence never
    // appears in any Debug surface reachable from the keypair (public key). A
    // regression that leaked the seed through some Debug path would surface the
    // exact bytes.
    let kp = Keypair::generate();
    let secret_bytes = kp.secret_bytes_unprotected();
    assert_ne!(
        secret_bytes, [0u8; 32],
        "CSPRNG must not yield all-zero seed"
    );

    // Render the secret bytes as the same decimal-array form a leaking Debug
    // would emit, and confirm it is absent from the public-key Debug surface.
    let decimal_seq = secret_bytes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let pub_dbg = format!("{:?}", kp.public_key());
    assert!(
        !pub_dbg.contains(&decimal_seq),
        "Keypair public-key Debug MUST NOT contain the raw secret seed bytes \
         (D-74/75/76); pub-debug=`{pub_dbg}`"
    );
}

// =====================================================================
// (b) Zeroize + redact coverage — source-anchored, for the FULL roster
//     (incl. private-field types with no public constructor)
// =====================================================================

/// Assert `needle` appears in `body`, with a roster-context failure message.
fn assert_source_has(body: &str, needle: &str, ty: &str, contract: &str) {
    assert!(
        body.contains(needle),
        "D-74/75/76 roster: `{ty}` MUST retain its {contract} wiring — \
         expected `{needle}` in source. If this fires, the secret-hygiene fix \
         for `{ty}` was reverted; re-land it.",
    );
}

#[test]
fn crypto_suite_cipher_suite_secret_hygiene_present() {
    let body = read_crate_source("benten-crypto-suite", "src/cipher_suite.rs");
    // DecryptedPlaintext: redacted Debug + zeroize-on-drop (KNOWN-STILL-OPEN close).
    assert_source_has(
        &body,
        "impl Drop for DecryptedPlaintext",
        "DecryptedPlaintext",
        "zeroize-on-drop",
    );
    assert_source_has(
        &body,
        "impl core::fmt::Debug for DecryptedPlaintext",
        "DecryptedPlaintext",
        "redacted-Debug",
    );
    // UnwrappedKey sibling (already hardened; guard against regression).
    assert_source_has(
        &body,
        "impl Drop for UnwrappedKey",
        "UnwrappedKey",
        "zeroize-on-drop",
    );
    // RecipientSecret (ML-KEM dk) drop-wipe.
    assert_source_has(
        &body,
        "impl Drop for RecipientSecret",
        "RecipientSecret",
        "zeroize-on-drop",
    );
}

#[test]
fn crypto_suite_sig_and_swap_matrix_pq_key_zeroize_present() {
    let sig = read_crate_source("benten-crypto-suite", "src/sig.rs");
    // sig::Keypair.pq — the ML-DSA-65 signing key zeroizes via the ml-dsa
    // `zeroize` cargo feature (documented on the struct).
    assert_source_has(
        &sig,
        "BOTH signing-key halves zeroize on drop",
        "sig::Keypair",
        "PQ-half zeroize (ml-dsa `zeroize` feature)",
    );
    let swap = read_crate_source("benten-crypto-suite", "src/swap_matrix.rs");
    assert_source_has(
        &swap,
        "BOTH raw signing keys zeroize on drop",
        "PurePqKeypairInner",
        "pq_sk zeroize (ml-dsa `zeroize` feature)",
    );
    // PurePqMlKemKeypair.secret_bytes drop-wipe (sibling; guard regression).
    assert_source_has(
        &swap,
        "impl Drop for PurePqMlKemKeypair",
        "PurePqMlKemKeypair",
        "zeroize-on-drop",
    );
    // PureKemDec — recovered shared secret: cfg-gated OFF the frozen surface +
    // zeroize-on-drop.
    assert_source_has(
        &swap,
        "impl Drop for PureKemDec",
        "PureKemDec",
        "zeroize-on-drop",
    );
    assert_source_has(
        &swap,
        "#[cfg(any(test, feature = \"testing\"))]\npub struct PureKemDec",
        "PureKemDec",
        "cfg-gate off the frozen public-api surface",
    );
}

#[test]
fn crypto_suite_mlkem_keypair_bytes_dk_zeroizes() {
    let body = read_crate_source("benten-crypto-suite", "src/mlkem.rs");
    // The transient MlKemKeypairBytes carrier holds `dk` in a `Zeroizing`.
    assert_source_has(
        &body,
        "pub dk: Zeroizing<Vec<u8>>",
        "MlKemKeypairBytes",
        "zeroizing dk field",
    );
}

#[test]
fn crypto_suite_ml_dsa_zeroize_feature_enabled() {
    // The PQ-signing-key zeroize (sig::Keypair.pq + PurePqKeypairInner.pq_sk)
    // depends on the ml-dsa `zeroize` cargo feature being ON — the upstream
    // `impl ZeroizeOnDrop for SigningKey<P>` is `#[cfg(feature = "zeroize")]`-
    // gated, NOT unconditional.
    let cargo = read_crate_source("benten-crypto-suite", "Cargo.toml");
    assert!(
        cargo.contains("\"zeroize\"") && cargo.contains("ml-dsa = { version = \"0.1\""),
        "D-74/75/76: benten-crypto-suite Cargo.toml MUST enable the ml-dsa \
         `zeroize` feature (the ML-DSA-65 SigningKey ZeroizeOnDrop is feature-\
         gated upstream, NOT unconditional). If this fires, sig::Keypair.pq + \
         PurePqKeypairInner.pq_sk no longer wipe on drop.",
    );
    // Guard against a naive line-level match: the ml-dsa dep line must carry it.
    let ml_dsa_line = cargo
        .lines()
        .find(|l| l.trim_start().starts_with("ml-dsa = { version"))
        .expect("ml-dsa dep line present");
    assert!(
        ml_dsa_line.contains("\"zeroize\""),
        "ml-dsa dep line MUST include the `zeroize` feature; line=`{ml_dsa_line}`",
    );
}

#[test]
fn crypto_suite_vault_payload_secret_hygiene_present() {
    let body = read_crate_source("benten-crypto-suite", "src/vault.rs");
    assert_source_has(
        &body,
        "impl Drop for VaultPayload",
        "VaultPayload",
        "zeroize-on-drop",
    );
    assert_source_has(
        &body,
        "impl core::fmt::Debug for VaultPayload",
        "VaultPayload",
        "redacted-Debug",
    );
    // The frozen on-disk format MUST be preserved (Serialize/Deserialize kept).
    assert_source_has(
        &body,
        "#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]",
        "VaultPayload",
        "frozen Serialize/Deserialize preserved (no wire change)",
    );
    // N-17: UnlockedKeyMaterial — the hydrated user-DID hybrid signing key
    // (a long-lived at-rest secret) MUST wipe on drop, symmetric with
    // K_principal (Compromise #36/#39). The roster previously omitted this
    // type; a revert of vault.rs:629 `impl Drop` would leave the recovered
    // signing key un-wiped and this fires.
    assert_source_has(
        &body,
        "impl Drop for UnlockedKeyMaterial",
        "UnlockedKeyMaterial",
        "zeroize-on-drop",
    );
    assert_source_has(
        &body,
        "impl core::fmt::Debug for UnlockedKeyMaterial",
        "UnlockedKeyMaterial",
        "redacted-Debug",
    );
}

#[test]
fn id_secret_key_and_keypair_hygiene_present() {
    let body = read_crate_source("benten-id", "src/keypair.rs");
    // SecretKey: ZeroizeOnDrop derive + redacted Debug (already present; guard).
    assert_source_has(
        &body,
        "#[derive(Zeroize, ZeroizeOnDrop)]",
        "SecretKey",
        "zeroize-on-drop",
    );
    assert_source_has(
        &body,
        "impl fmt::Debug for SecretKey",
        "SecretKey",
        "redacted-Debug",
    );
    // The raw-`[u8; 32]` `_for_test` accessors are cfg-gated off the frozen
    // surface (D-74/75/76). Assert BOTH are gated.
    assert_source_has(
        &body,
        "#[cfg(any(test, feature = \"testing\"))]\n    pub fn bytes_for_test",
        "SecretKey::bytes_for_test",
        "cfg-gate off the frozen public-api surface",
    );
    assert_source_has(
        &body,
        "#[cfg(any(test, feature = \"testing\"))]\n    pub fn secret_bytes_for_test",
        "Keypair::secret_bytes_for_test",
        "cfg-gate off the frozen public-api surface",
    );
    // The sanctioned production alias stays ungated + routes through the NON-
    // `_for_test` crate-internal accessor.
    assert_source_has(
        &body,
        "self.secret.bytes_unprotected()",
        "Keypair::secret_bytes_unprotected",
        "production path routes through the non-`_for_test` accessor",
    );
}

#[test]
fn engine_layer_d_secret_hygiene_present() {
    let device_auth = read_crate_source("benten-engine", "src/layer_d/device_auth.rs");
    // expose_unlocked_key — raw K_principal accessor cfg-gated off the frozen
    // benten-engine public surface (R19 mini-review follow-up).
    assert_source_has(
        &device_auth,
        "#[cfg(any(test, feature = \"test-helpers\"))]\npub fn expose_unlocked_key",
        "expose_unlocked_key",
        "cfg-gate off the frozen public-api surface",
    );

    let remote = read_crate_source("benten-engine", "src/layer_d/remote_permission.rs");
    assert_source_has(
        &remote,
        "impl Drop for PermissionRequest",
        "PermissionRequest",
        "zeroize-on-drop (ephemeral_signing_key)",
    );
    assert_source_has(
        &remote,
        "impl core::fmt::Debug for PermissionRequest",
        "PermissionRequest",
        "redacted-Debug",
    );

    let device_link = read_crate_source("benten-engine", "src/layer_d/device_link.rs");
    assert_source_has(
        &device_link,
        "impl Drop for ProvisioningInnerPayload",
        "ProvisioningInnerPayload",
        "zeroize-on-drop",
    );

    let secret_store = read_crate_source("benten-engine", "src/layer_d/secret_store.rs");
    assert_source_has(
        &secret_store,
        "impl Drop for KeyringCoreStore",
        "KeyringCoreStore",
        "zeroize-on-drop (HashMap secret values)",
    );
    assert_source_has(
        &secret_store,
        "impl Drop for FileVaultStore",
        "FileVaultStore",
        "zeroize-on-drop (HashMap secret values)",
    );
}

#[test]
fn sync_production_callers_use_unprotected_not_for_test_accessor() {
    // D-74/75/76 migrated the two former production callers of
    // `secret_bytes_for_test` (a `_for_test`-named accessor) onto the
    // sanctioned `secret_bytes_unprotected` production alias so the `_for_test`
    // accessor could be cfg-gated off the production surface.
    for (crate_dir, rel) in [
        ("benten-sync", "src/transport.rs"),
        ("benten-sync", "src/peer_discovery.rs"),
    ] {
        let body = read_crate_source(crate_dir, rel);
        // Match the CALL form (`.secret_bytes_for_test()`), not a bare mention —
        // the migration-provenance comment references the old name by design.
        assert!(
            !body.contains(".secret_bytes_for_test()"),
            "D-74/75/76: {crate_dir}/{rel} MUST NOT call `secret_bytes_for_test` \
             (a `_for_test`-named accessor in production). Use \
             `secret_bytes_unprotected`.",
        );
        assert!(
            body.contains("secret_bytes_unprotected"),
            "D-74/75/76: {crate_dir}/{rel} MUST use the production \
             `secret_bytes_unprotected` accessor for iroh keypair construction.",
        );
    }
}
