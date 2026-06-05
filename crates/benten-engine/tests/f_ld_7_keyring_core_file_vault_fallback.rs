//! F-LD-7 — `keyring-core` + file-vault fallback (RED-PHASE; FN).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-7: "`keyring-core` v1.0.0 (NOT legacy `keyring`) stores DAK-wrap;
//!     file-vault fallback; Tauri IPC smoke." Red-phase: "store/retrieve
//!     round-trip; fallback when keychain absent."
//!   - R0.5 plan §3.4: the `keyring-core` DAK-wrap store with file-vault
//!     fallback; CLAUDE.md 3-tactical-picks ratification (`keyring-core` v1.0.0,
//!     NOT legacy `keyring`).
//!   - Compromise #51 (Tauri NAPI side-channel) disclosure coupling.
//!
//! ## RED-PHASE (pim-12 §3.6e)
//!
//! SELF-CONTAINED stub-shim models a `SecretStore` seam with two backends:
//! a keychain backend (`keyring-core`) and a file-vault fallback. R5 swaps in
//! the real `keyring-core` v1.0.0 binding + the file-vault impl.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

// R5: stub-shim DELETED; real Layer-D DAK-wrap secret-store seam in use.
use benten_engine::layer_d::secret_store::{
    KeyringCoreStore, SecretStore, SecretStoreError, open_dak_wrap_store,
};

/// F-LD-7 keychain round-trip: with the OS keychain available, the DAK-wrap is
/// stored + retrieved via `keyring-core`. would-FAIL-if-no-op'd: a store that
/// dropped the secret would fail retrieval.
#[test]
fn f_ld_7_keyring_core_dak_wrap_round_trips() {
    let mut store = open_dak_wrap_store(true);
    assert_eq!(
        store.backend_name(),
        "keyring-core",
        "MUST use keyring-core when keychain available"
    );

    let dak_wrap = b"dak-wrapped-key-material-32-bytes";
    store
        .store("benten-dak-wrap", dak_wrap)
        .expect("storing the DAK-wrap MUST succeed");
    let got = store
        .retrieve("benten-dak-wrap")
        .expect("retrieving the DAK-wrap MUST succeed");
    assert_eq!(got, dak_wrap, "retrieved DAK-wrap MUST match stored");
}

/// F-LD-7 file-vault FALLBACK: with NO OS keychain (headless / CI / no-DBus),
/// the store falls back to the encrypted file-vault — store + retrieve still
/// work. would-FAIL-if-no-op'd: if fallback were absent, `open_dak_wrap_store`
/// would yield a keychain backend whose store() errors `KeychainUnavailable`.
#[test]
fn f_ld_7_file_vault_fallback_when_keychain_absent() {
    let mut store = open_dak_wrap_store(false); // no keychain
    assert_eq!(
        store.backend_name(),
        "file-vault",
        "MUST fall back to file-vault when the OS keychain is unavailable"
    );
    let dak_wrap = b"dak-wrapped-key-material-fallback";
    store
        .store("benten-dak-wrap", dak_wrap)
        .expect("file-vault fallback store MUST succeed");
    let got = store
        .retrieve("benten-dak-wrap")
        .expect("file-vault retrieve MUST succeed");
    assert_eq!(got, dak_wrap);
}

/// F-LD-7 keychain-unavailable typed-reject at the keyring-core backend
/// directly: when the keyring-core backend is used but the keychain is down,
/// store/retrieve surface typed `KeychainUnavailable` (so the fallback decision
/// is explicit, never a silent data loss).
#[test]
fn f_ld_7_keyring_core_typed_keychain_unavailable() {
    let mut keyring = KeyringCoreStore::new(false);
    assert_eq!(
        keyring.store("k", b"v"),
        Err(SecretStoreError::KeychainUnavailable),
        "an unavailable keychain MUST surface typed KeychainUnavailable, never silently drop"
    );
}

/// F-LD-7 Tauri-IPC smoke (shape): the DAK-wrap store is reachable through the
/// `Box<dyn SecretStore>` seam the embedded-webview (Tauri shape-c) shell
/// drives over IPC. This pins the dyn-dispatch shape the IPC bridge uses;
/// behavioral Tauri integration is a Composing-phase concern. Couples
/// Compromise #51 (Tauri NAPI side-channel) honest disclosure.
#[test]
fn f_ld_7_tauri_ipc_secret_store_dyn_seam_smoke() {
    // The IPC bridge holds the store as a trait object + drives store/retrieve.
    let mut store: Box<dyn SecretStore> = open_dak_wrap_store(true);
    store.store("ipc-dak", b"ipc-secret").expect("ipc store");
    assert_eq!(store.retrieve("ipc-dak").unwrap(), b"ipc-secret");
}
