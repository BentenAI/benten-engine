//! D-4F-16 paired discipline pin per R2 §5 Gap fix #5.
//!
//! Negative grep-assert against the future
//! `crates/benten-id/src/plugin_did.rs` for HKDF / seed-derivation
//! patterns. Plugin-DID is NOT derived from user-DID — it is freshly
//! minted via OsRng at install. Any HKDF / derive_from pattern in
//! plugin_did.rs is a structural violation of D-4F-16.

use std::fs;
use std::path::Path;

/// Un-ignored at R6-FP-BF (closes R6 R1 test-coverage-auditor tc-1 +
/// tc-2 — `plugin_did::mint` source-cite cluster). The
/// `crates/benten-id/src/plugin_did.rs` source file exists at HEAD;
/// this grep-assert confirms no HKDF / seed-derivation pattern is
/// present.
#[test]
fn plugin_did_module_source_contains_no_hkdf_or_derive_from_user_did_patterns() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("plugin_did.rs");

    let src = fs::read_to_string(&path).expect(
        "crates/benten-id/src/plugin_did.rs must exist (shipped at G24-D wave per CLAUDE.md #18)",
    );

    // Forbidden patterns: HKDF, seed-derivation, deterministic-from-
    // user-DID minting. Plugin-DID MUST be freshly minted via OsRng.
    //
    // Per R2 §5 substance discipline: grep-assert against the SOURCE
    // FILE on disk, not against a runtime-introspectable surface.
    let forbidden = [
        "hkdf",
        "Hkdf",
        "HKDF",
        "derive_from_user",
        "derive_from_seed",
        "DeriveFromUserDid",
        "plugin_did_from_user_did",
    ];
    for pat in &forbidden {
        assert!(
            !src.contains(pat),
            "plugin_did.rs must not contain pattern {pat:?} (D-4F-16: plugin-DID is OsRng-minted, NOT derived from user-DID)"
        );
    }
}
