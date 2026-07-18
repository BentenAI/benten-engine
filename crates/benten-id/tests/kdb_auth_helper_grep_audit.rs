//! GAP-KDB Shape-B / Fork-A — AUTH-7 ONE-codepoint-dispatched-helper
//! grep-audit completeness net. W3 authority-migration RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A + Worry-#1
//! ("must be designed as ONE codepoint-dispatched verify, not bolted
//! on … Mis-wire = either two parallel resolve paths or a PQ-strip").
//! `R2-LANDSCAPE` AUTH-7 (the completeness net guaranteeing no
//! un-migrated verify site re-opens the silent-PQ-strip). Mirrors the
//! `collapse_p4_1230_device_parent_binding_dissolved.rs` source-grep
//! idiom + the `ct_signature_eq` / `no_hardcoded_sizes` grep-absence
//! precedents.
//!
//! # What this pins
//! The authority-verify surface in `benten-id/src/` — the UCAN
//! chain-walk (`ucan.rs`), rotation verify (`did_rotation.rs`), and
//! device-attestation verify (`device_attestation.rs`) — after Fork-A
//! must NOT each carry their own inline Ed25519-only `[u8; 64]`
//! signature extraction. The composite arms route through
//! `SignatureSuite::verify` (already strip-resistant); the classical
//! `did:key` arm lives in AT MOST ONE shared helper. A residual
//! second inline `sig_bytes: [u8; 64]` extraction on a verify path is
//! a candidate silent-PQ-strip site (verify only the 64-byte Ed25519
//! half, ignore the composite), so the count MUST collapse to ≤ 1.
//!
//! # would_fail_on_revert
//! At the freeze base there are exactly THREE inline `sig_bytes:
//! [u8; 64]` Ed25519-only extractions (one per authority-verify site:
//! `ucan.rs`, `did_rotation.rs`, `device_attestation.rs`) → the audit
//! FAILS (3 > 1). After Fork-A consolidates them behind ONE
//! codepoint-dispatched helper (≤ 1 residual classical arm) it PASSES.
//! Re-introducing a second Ed25519-only verify path (a bolt-on that
//! re-opens the PQ-strip) makes the count exceed 1 → the audit fails
//! again. This is the grep completeness net over the flagship
//! `kdb_auth_ucan_walk_hybrid.rs` behavioral pins.
//!
//! # R5 un-ignore
//! After migrating the three authority-verify sites to the single
//! codepoint-dispatched hybrid helper, drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

/// The authority-verify source files whose Ed25519-only `[u8; 64]`
/// extractions must collapse to a single shared helper.
const AUTHORITY_VERIFY_FILES: &[&str] = &["ucan.rs", "did_rotation.rs", "device_attestation.rs"];

/// The un-migrated Ed25519-only signature-extraction shape — a
/// `[u8; 64]` (Ed25519 signature length) `try_into` on the verify
/// path. A `session_nonce`/key is `[u8; 32]`; only a raw Ed25519
/// signature is `[u8; 64]`, so this substring is authority-signature
/// specific.
const ED25519_ONLY_SIG_EXTRACTION: &str = "[u8; 64]";

fn count_ed25519_only_verify_sites() -> Vec<String> {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sites = Vec::new();
    for file in AUTHORITY_VERIFY_FILES {
        let path = src_dir.join(file);
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for (lineno, line) in body.lines().enumerate() {
            // Skip comment lines (deletion/migration narrative may NAME
            // the old shape) and `_for_test` fixtures.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            // Only the signature-extraction shape: the `[u8; 64]` type
            // ascription on a `sig`/signature binding, not an unrelated
            // 64-byte buffer.
            if line.contains(ED25519_ONLY_SIG_EXTRACTION)
                && (line.contains("sig") || line.contains("signature"))
            {
                sites.push(format!("{file}:{}: {}", lineno + 1, line.trim()));
            }
        }
    }
    sites
}

#[test]
#[ignore = "RED-PHASE: AUTH-7 one-helper grep-audit — no 2nd Ed25519-only verify path — un-ignore at R5"]
fn auth7_no_second_ed25519_only_verify_path_after_fork_a_migration() {
    let sites = count_ed25519_only_verify_sites();
    assert!(
        sites.len() <= 1,
        "AUTH-7 (Fork-A completeness net): after the authority-path hybrid migration there \
         must be AT MOST ONE inline Ed25519-only `[u8; 64]` signature extraction (the single \
         shared classical did:key arm). Found {} — each residual site is a candidate \
         silent-PQ-strip verify path that re-opens FLAGSHIP-2. Route composite verifies \
         through SignatureSuite::verify and consolidate the classical arm into ONE helper.\n{}",
        sites.len(),
        sites.join("\n")
    );
}

/// Baseline coherence control (REAL now, non-ignored): the audit is
/// scanning the ACTUAL authority-verify source, not an empty set — at
/// the freeze base the three un-migrated Ed25519-only sites ARE present
/// (this is what the ignored pin will drive to ≤ 1). If this control
/// ever finds zero, the file list drifted and the AUTH-7 pin is
/// scanning nothing (a silent false-green risk per R2 catch-net note).
#[test]
fn auth7_baseline_scans_the_real_unmigrated_authority_verify_sites() {
    let sites = count_ed25519_only_verify_sites();
    assert!(
        !sites.is_empty(),
        "AUTH-7 scan-integrity: expected the authority-verify files ({:?}) to contain the \
         un-migrated Ed25519-only signature-extraction sites at the freeze base — found none, \
         so the AUTH-7 grep target drifted (it would false-green). Re-anchor the file list.",
        AUTHORITY_VERIFY_FILES
    );
}
