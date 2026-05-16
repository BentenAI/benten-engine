//! COLLAPSE (P4) — #1230 / #605 / #707-trust-half closure-pin
//! (pim-2 §3.6b, would-FAIL-if-no-op'd).
//!
//! # Charter
//!
//! Spec: `.addl/refinement-audit-2026-05/impl-design-COLLAPSE.md` §5
//! (#707 collapse-10-to-1 mechanics) + §1.3 (benten-caps deletion set)
//! + `DECISION-RECORD-trust-model-reframe.md` §4 (RATIFIED).
//!
//! P1/P2/P3 (merged in keystone #1238) DELETED the parallel
//! device-revocation pipe: `benten_id::validate_chain_with_device_revocations`,
//! `benten_id::device_attestation::DeviceRevocation` /
//! `RevocationReason` / `Acceptor`, and the `benten-caps` durable
//! halves `dev_revoke_key` / `record_revocation` /
//! `validate_chain_with_durable_revocations`. #1238 deliberately
//! merged with `Refs` (NOT `Closes`) for #1230/#605/#707 precisely so
//! the closure-pin tests are authored here, in P4, and the v1-BLOCKERs
//! are not considered closed without a would-FAIL-if-no-op'd pin.
//!
//! # The load-bearing property this pins (#1230 — v1-BLOCKER)
//!
//! **The perpetual-victim-DoS is DISSOLVED BY DELETION.** #1230's root
//! cause: `validate_chain_with_device_revocations` matched revocations
//! on a *bare device-DID* with NO device→actual-parent binding — an
//! attacker who controlled *any* `parent_did` could sign a
//! *structurally valid* revocation against a victim's device-DID and
//! perpetually DoS that device. The fix is NOT to add parent-binding
//! to that pipe (that was the rejected Option-D); it is that the pipe
//! **no longer exists**. Revocation now flows through exactly ONE
//! self-anchored seam: `UCANBackend::{revoke,is_revoked}`, keyed on
//! the **content-CID of a user-root-traced UCAN envelope** (claims +
//! signature). There is no API anywhere that accepts a bare device-DID
//! revocation, so the forged-parent-against-victim-device-DID attack
//! is **structurally unconstructible** — not merely defended.
//!
//! This pin asserts that single-seam property behaviorally
//! (#1230/#605/#707-trust) AND structurally (the deleted parallel
//! pipe must not reappear in production source — the #707 definitional
//! collapse). If P1/P2/P3's deletions were reverted (a parallel
//! device-DID-keyed revocation pipe re-introduced), the structural
//! grep pin FAILs; if the single self-anchored revocation seam were
//! no-op'd, the behavioral pin FAILs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use benten_caps::CapError;
use benten_caps::backends::ucan::UCANBackend;
use benten_graph::RedbBackend;
use benten_id::keypair::Keypair;
use benten_id::ucan::{Capability, Ucan};

fn fresh_backend() -> UCANBackend<RedbBackend> {
    let inner = RedbBackend::open_in_memory().expect("redb in-memory open");
    UCANBackend::new(Arc::new(inner))
}

fn now() -> u64 {
    1_000_000_000
}

fn build_ucan(issuer: &Keypair, audience: &Keypair, cap: Capability, nbf: u64, exp: u64) -> Ucan {
    Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(audience.public_key().to_did().as_str())
        .capability(cap.resource, cap.ability)
        .not_before(nbf)
        .expiry(exp)
        .sign(issuer)
}

/// **MANDATORY closure-pin #1230 / #605 / #707-trust (would-FAIL-if-no-op'd).**
///
/// The ONLY revocation seam is the self-anchored, content-CID-keyed,
/// user-root-traced `UCANBackend::{revoke,is_revoked}`. A UCAN
/// installed + validating successfully, once revoked through this one
/// seam, MUST observably reject at `validate_chain`. This is the
/// post-COLLAPSE revocation path that REPLACED the deleted
/// `validate_chain_with_device_revocations` parallel pipe.
///
/// **Why this proves #1230 dissolved (not merely defended):** the
/// revocation identity is `ucan_cid(envelope)` — the BLAKE3
/// content-address of the full UCAN (claims + signature). To revoke a
/// victim's authority an attacker would need the victim's own
/// user-root-traced grant CID *and* the ability to write to the
/// victim's durable store — i.e. they would already have to be the
/// victim. There is no bare-device-DID revocation key (the #1230
/// forge surface) anywhere in the surviving API. The perpetual-victim
/// DoS is structurally impossible because the pipe that made it
/// possible was deleted.
///
/// If the `is_revoked` consultation in `validate_chain_at` were
/// no-op'd (the only remaining revocation enforcement after the
/// parallel pipe's deletion), the `expect_err`/`matches!` below FAILs
/// — pim-2 §3.6b would-FAIL-if-no-op'd.
#[test]
fn single_self_anchored_revocation_seam_is_the_only_revocation_path() {
    let backend = fresh_backend();
    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let now = now();

    let ucan = build_ucan(
        &issuer,
        &audience,
        Capability::new("/zone/posts", "write"),
        now - 1,
        now + 3600,
    );

    // Install + validate: the chain is good through the single seam.
    let cid = backend.install_proof(&ucan).expect("install_proof");
    backend
        .validate_chain(std::slice::from_ref(&ucan), now)
        .expect("pre-revocation chain-walk MUST pass");
    assert!(
        !backend.is_revoked(&cid).unwrap(),
        "fresh grant must not be revoked"
    );

    // Revoke through the ONE self-anchored, content-CID-keyed seam.
    backend.revoke(&cid).expect("revoke via single seam");

    // OBSERVABLE consequence: the same UCAN that validated now
    // rejects with the typed `Revoked` — through the single seam,
    // NOT a deleted device-DID-keyed parallel pipe.
    assert!(
        backend.is_revoked(&cid).unwrap(),
        "post-revoke `is_revoked` MUST be true at the single seam"
    );
    let err = backend.validate_chain(&[ucan], now).expect_err(
        "COLLAPSE #1230 REGRESSION: a UCAN revoked through the single \
             self-anchored content-CID-keyed seam was still admitted — the \
             only post-COLLAPSE revocation enforcement has been no-op'd. \
             #1230's perpetual-victim-DoS was dissolved by deleting the \
             un-anchored device-DID-keyed parallel pipe; this single seam \
             is the sole survivor and MUST stay load-bearing (pim-2 §3.6b).",
    );
    assert!(
        matches!(err, CapError::Revoked),
        "single-seam revocation MUST surface CapError::Revoked; got {err:?}"
    );
}

/// **MANDATORY structural closure-pin #707-trust-subset / #1230
/// (would-FAIL-if-the-parallel-pipe-reappears).**
///
/// The #707-trust-subset is closed *definitionally* by removing the
/// parallel device-revocation pipe (impl-design §5: instances 1+10 are
/// the trust subset; COLLAPSE closes them by construction). This pin
/// asserts the deleted symbols do NOT reappear in benten-caps
/// production source — if a future change re-introduces a parallel
/// device-DID-keyed revocation pipe (the #1230 forge surface, the
/// #707 asymmetric parallel entry point), this FAILs.
///
/// Grep-defense over PRODUCTION source only (`src/`), excluding the
/// COLLAPSE deletion-marker comment lines that intentionally NAME the
/// deleted symbols for posterity (`docs/SECURITY-POSTURE.md`
/// Compromise #23 SUPERSEDED-BY-COLLAPSE cross-ref). This mirrors the
/// established `cap_r1_1_audience_binding_grep_defense.rs` pattern.
#[test]
fn deleted_device_revocation_parallel_pipe_must_not_reappear_in_production_source() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    // Walk every .rs under benten-caps/src/ and assert no production
    // (non-comment) line re-introduces the deleted parallel-pipe
    // symbols. The COLLAPSE module-doc in `backends/ucan.rs`
    // intentionally NAMES the deleted symbols inside `//!` comment
    // lines for the SUPERSEDED-BY-COLLAPSE narrative — those are
    // excluded (they are documentation of the deletion, not the pipe).
    let deleted_symbols = [
        "validate_chain_with_durable_revocations",
        "fn record_revocation",
        "dev_revoke_key",
        "g14b:dev_revoke",
    ];

    let mut offenders: Vec<String> = Vec::new();
    visit_rs(&src_dir, &mut |path, contents| {
        for (lineno, line) in contents.lines().enumerate() {
            let trimmed = line.trim_start();
            // Skip doc-comment + line-comment lines: the COLLAPSE
            // deletion narrative legitimately names the dead symbols.
            if trimmed.starts_with("//") {
                continue;
            }
            for sym in &deleted_symbols {
                if line.contains(sym) {
                    offenders.push(format!(
                        "{}:{} reintroduces deleted parallel-pipe symbol `{sym}`: {}",
                        path.display(),
                        lineno + 1,
                        line.trim()
                    ));
                }
            }
        }
    });

    assert!(
        offenders.is_empty(),
        "COLLAPSE #707-trust / #1230 REGRESSION: the deleted parallel \
         device-revocation pipe reappeared in benten-caps production \
         source. The #1230 perpetual-victim-DoS is dissolved ONLY by the \
         parallel pipe's ABSENCE; re-introducing a device-DID-keyed \
         revocation store recreates the forge surface (pim-2 §3.6b, \
         impl-design §5 definitional collapse). Offenders:\n{}",
        offenders.join("\n")
    );
}

fn visit_rs(dir: &std::path::Path, f: &mut impl FnMut(&std::path::Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, f);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs")
            && let Ok(contents) = std::fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}
