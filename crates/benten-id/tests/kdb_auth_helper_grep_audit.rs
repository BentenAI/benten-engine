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

use std::path::{Path, PathBuf};

/// The un-migrated Ed25519-only signature-extraction shape — a
/// `[u8; 64]` (Ed25519 signature length) `try_into` on the verify
/// path. A `session_nonce`/key is `[u8; 32]`; only a raw Ed25519
/// signature is `[u8; 64]`, so this substring is authority-signature
/// specific.
const ED25519_ONLY_SIG_EXTRACTION: &str = "[u8; 64]";

/// Recursively collect every `.rs` file under `dir`.
fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// **AUTH-7 hardening (Fork-A):** scan the WHOLE `benten-id/src` tree, not
/// a hardcoded 3-file list. The Fork-A migration must consolidate the
/// Ed25519-only signature extraction across EVERY authority-verify site —
/// the UCAN chain-walk (`ucan.rs`), rotation-verify (`did_rotation.rs`),
/// device-attestation (`device_attestation.rs`), AND VC-verify (`vc.rs`,
/// per D-53). A file-scoped list would false-green an un-migrated site in a
/// file it does not name (e.g. `vc.rs`), so the net walks the whole tree and
/// requires the count to collapse to the single shared classical arm in
/// `authority_verify.rs`.
fn count_ed25519_only_verify_sites() -> Vec<String> {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rs_files(&src_dir, &mut files);
    files.sort();
    let mut sites = Vec::new();
    for path in &files {
        let body = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let rel = path.strip_prefix(&src_dir).unwrap_or(path);
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
                sites.push(format!("{}:{}: {}", rel.display(), lineno + 1, line.trim()));
            }
        }
    }
    sites
}

#[test]
fn auth7_no_second_ed25519_only_verify_path_after_fork_a_migration() {
    let sites = count_ed25519_only_verify_sites();
    assert!(
        sites.len() <= 1,
        "AUTH-7 (Fork-A completeness net): after the authority-path hybrid migration there \
         must be AT MOST ONE inline Ed25519-only `[u8; 64]` signature extraction across the \
         WHOLE benten-id/src tree (the single shared classical arm in authority_verify.rs). \
         Found {} — each residual site is a candidate silent-PQ-strip verify path that \
         re-opens FLAGSHIP-2. Route composite verifies through SignatureSuite::verify and \
         consolidate the classical arm into the ONE authority_verify helper.\n{}",
        sites.len(),
        sites.join("\n")
    );
}

/// Baseline coherence control (REAL now, non-ignored): the audit is
/// scanning the ACTUAL benten-id/src tree, not an empty set. At the freeze
/// base the FOUR un-migrated Ed25519-only sites are present (ucan.rs,
/// did_rotation.rs, device_attestation.rs, vc.rs); after the Fork-A
/// migration exactly ONE remains (the shared classical arm in
/// authority_verify.rs). Either way the count is ≥ 1 — if this control ever
/// finds zero, the `[u8; 64]` grep target drifted and the AUTH-7 pin is
/// scanning nothing (a silent false-green risk per R2 catch-net note).
#[test]
fn auth7_baseline_scans_the_real_unmigrated_authority_verify_sites() {
    let sites = count_ed25519_only_verify_sites();
    assert!(
        !sites.is_empty(),
        "AUTH-7 scan-integrity: expected the benten-id/src tree to contain at least one \
         Ed25519-only `[u8; 64]` signature-extraction site (the shared classical arm) — found \
         none, so the AUTH-7 grep target drifted (it would false-green). Re-anchor the shape."
    );
}
