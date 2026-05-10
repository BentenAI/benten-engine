//! G14-A1 wave-4a — UCAN chain validation test pins (un-ignored).
//!
//! Pin sources (per `r2-test-landscape` §2.2 G14-A1 + §11 CLR-2 + plan
//! §3 G14-A1).

#![allow(clippy::unwrap_used)]

use benten_id::UcanError;
use benten_id::keypair::Keypair;
use benten_id::ucan::{
    Ucan, validate_chain_at, validate_chain_for_audience, validate_chain_no_time_check,
};

fn now_secs() -> u64 {
    1_000_000_000
}

#[test]
fn ucan_chain_validation_basic() {
    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let now = now_secs();
    let ucan = Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(audience.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 3600)
        .sign(&issuer);
    assert!(validate_chain_no_time_check(std::slice::from_ref(&ucan)).is_ok());
    assert!(validate_chain_at(&[ucan], now).is_ok());
}

#[test]
fn ucan_chain_attenuation_rejects_overgrant() {
    let root = Keypair::generate();
    let delegate = Keypair::generate();
    let leaf_aud = Keypair::generate();
    let now = now_secs();

    // Parent grants /zone/posts read.
    let parent = Ucan::builder()
        .issuer(root.public_key().to_did().as_str())
        .audience(delegate.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 3600)
        .sign(&root);

    // Child tries to grant /zone/posts WRITE — overgrant.
    let overgrant = Ucan::builder()
        .issuer(delegate.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "write")
        .not_before(now - 1)
        .expiry(now + 3600)
        .proof(parent.clone())
        .sign(&delegate);

    let chain = vec![overgrant, parent];
    let err = validate_chain_at(&chain, now).unwrap_err();
    assert!(
        matches!(err, UcanError::AttenuationViolated { .. }),
        "expected AttenuationViolated, got {err:?}"
    );
}

#[test]
#[ignore = "phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation — production prerequisite NOT YET shipped at HEAD. `validate_chain_with_revocations` symbol does NOT exist (only mentioned in `crates/benten-id/src/ucan.rs:32` + `:36` doc comments; no concrete `pub fn validate_chain_with_revocations(...)` in the file). The G14-A1 chain-walk (`validate_chain_at`) is in-memory + does NOT consult a durable revocation set. G14-B PR #109 shipped the durable `UCANBackend<B>` but did NOT extend the chain-walker with a `RevocationSet` consumption arm — that lift composes with §2.1-followup re-evaluation outcome (would `ssi`-integration re-shape the revocation surface? cryptography-reviewer dispatch pending)."]
fn ucan_chain_revocation_propagates() {
    // Stays #[ignore]'d until G14-B's UCANBackend lands the
    // revocation store + `validate_chain_with_revocations` entry
    // point. The G14-A1 chain-walk is in-memory + does not consult
    // a durable revocation set.
    unreachable!("G14-B wires RevocationSet propagation");
}

#[test]
fn ucan_nbf_time_window_pre_activation_rejects() {
    let issuer = Keypair::generate();
    let aud = Keypair::generate();
    let now = now_secs();
    let ucan = Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now + 3600) // valid 1 hour from now
        .expiry(now + 7200)
        .sign(&issuer);
    let err = validate_chain_at(&[ucan], now).unwrap_err();
    assert!(
        matches!(err, UcanError::NotYetValid { .. }),
        "expected NotYetValid, got {err:?}"
    );
}

#[test]
fn ucan_exp_time_window_post_expiration_rejects() {
    let issuer = Keypair::generate();
    let aud = Keypair::generate();
    let issuance = 1_000_000_000;
    let ucan = Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(issuance)
        .expiry(issuance + 60)
        .sign(&issuer);
    let err = validate_chain_at(&[ucan], issuance + 120).unwrap_err();
    assert!(
        matches!(err, UcanError::Expired { .. }),
        "expected Expired, got {err:?}"
    );
}

#[test]
fn ucan_chain_walk_propagates_nbf_exp_through_attenuation() {
    let root = Keypair::generate();
    let delegate = Keypair::generate();
    let leaf_aud = Keypair::generate();
    let now = 1_000_000_000;

    // Parent expires at now+60.
    let parent = Ucan::builder()
        .issuer(root.public_key().to_did().as_str())
        .audience(delegate.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 60)
        .sign(&root);

    // Child claims same exp — does NOT widen parent's window. Chain is
    // structurally valid; only the per-link time-window check at
    // `now + 120` (past parent.exp) drives the rejection. This pin
    // asserts that parent expiry observably propagates through the
    // chain even when the child does not widen.
    //
    // (Before G16-B-B-rest sub-item B time-window narrowing landed,
    // a child claiming a wider exp than the parent silently passed
    // structural validation and only the per-link time-window check
    // caught the rejection. The narrowing pin
    // `crates/benten-caps/tests/ucan_chain_window_narrowing.rs::
    // ucan_chain_rejects_child_expires_after_parent` covers the
    // child-widens-parent case explicitly; this pin covers the
    // sister case where the child does NOT widen but the
    // wallclock has rolled past the inherited expiry.)
    let child = Ucan::builder()
        .issuer(delegate.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 60) // matches parent — no widening
        .proof(parent.clone())
        .sign(&delegate);

    let err = validate_chain_at(&[child, parent], now + 120).unwrap_err();
    assert!(
        matches!(err, UcanError::Expired { .. }),
        "expected Expired (per-link time-window check propagating parent's expiry), got {err:?}"
    );
}

#[test]
fn ucan_chain_walk_constant_time_comparison_audit() {
    // crypto-major-4. Source-grep: ALL benten-id modules with
    // security-decision compares MUST NOT contain naive `==` on
    // signature/audience/proof_cid/DID/nonce bytes; comparison goes
    // through `ct_signature_eq` (itself calling `subtle::ConstantTimeEq`).
    //
    // G14-A2 mini-review g14-a2-mr-2 extension: walk the FULL set of
    // crypto-bearing modules so a new comparison added to any of them
    // hits the audit. UNIFORMITY-at-security-decision-sites is the
    // load-bearing contract.
    let src_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let modules = [
        "ucan.rs",
        "did_rotation.rs",
        "device_attestation.rs",
        "vc.rs",
        "multi_sig.rs",
    ];
    let forbidden_patterns: &[&str] = &[
        "signature ==",
        "audience ==",
        "proof_cid ==",
        // G14-A1 mini-review MAJOR — capability authority compares.
        "parent.resource ==",
        "parent.ability ==",
        "child.resource ==",
        "child.ability ==",
        // G14-A2 mini-review MAJOR (g14-a2-mr-2) — DID-rotation +
        // device-attestation security-decision sites. DIDs and nonces
        // are PUBLIC values but the UNIFORMITY contract means future
        // contributors hit the audit at these sites.
        "previous_did ==",
        "next_did ==",
        "device_did ==",
        "parent_did ==",
        "nonce ==",
    ];
    for module in &modules {
        let src_path = src_root.join(module);
        let src = std::fs::read_to_string(&src_path)
            .unwrap_or_else(|_| panic!("audit must read crates/benten-id/src/{module}"));
        for (lineno, line) in src.lines().enumerate() {
            let trimmed = line.trim_start();
            // Skip comments + the const-time-eq impl itself + the
            // tests module.
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue;
            }
            for forbidden in forbidden_patterns {
                assert!(
                    !line.contains(forbidden),
                    "constant-time comparison required per crypto-major-4 \
                     (module {module} line {}): {line}",
                    lineno + 1
                );
            }
        }
    }
    // Verify subtle is in the dep tree.
    let cargo_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let cargo = std::fs::read_to_string(&cargo_path).unwrap();
    assert!(
        cargo.contains("subtle"),
        "benten-id MUST depend on `subtle` for constant-time comparison per crypto-major-4"
    );
}

#[test]
fn ucan_audience_binding_prevents_cross_atrium_replay() {
    let atrium_a = Keypair::generate();
    let atrium_b = Keypair::generate();
    let issuer = Keypair::generate();
    let now = now_secs();
    let ucan_for_a = Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(atrium_a.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 3600)
        .sign(&issuer);
    let err =
        validate_chain_for_audience(&[ucan_for_a], &atrium_b.public_key().to_did()).unwrap_err();
    assert!(
        matches!(err, UcanError::AudienceMismatch { .. }),
        "expected AudienceMismatch, got {err:?}"
    );
}

#[test]
fn ucan_chain_nbf_enforcement() {
    let issuer = Keypair::generate();
    let aud = Keypair::generate();
    let now = 1_000_000_000;
    let ucan = Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now + 60)
        .expiry(now + 3600)
        .sign(&issuer);
    // single-token validate_at honors nbf identically to
    // validate_chain_at:
    assert!(ucan.validate_at(now).is_err());
    assert!(ucan.validate_at(now + 120).is_ok());
}

#[test]
fn ucan_chain_exp_enforcement() {
    let issuer = Keypair::generate();
    let aud = Keypair::generate();
    let now = 1_000_000_000;
    let ucan = Ucan::builder()
        .issuer(issuer.public_key().to_did().as_str())
        .audience(aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now)
        .expiry(now + 60)
        .sign(&issuer);
    assert!(ucan.validate_at(now + 30).is_ok());
    assert!(ucan.validate_at(now + 120).is_err());
}
