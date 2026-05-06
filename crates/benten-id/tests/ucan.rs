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
#[ignore = "RED-PHASE: G14-B wave-4b — RevocationSet + validate_chain_with_revocations are durable-backend scope"]
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

    // Child claims long validity — but parent already expired.
    let child = Ucan::builder()
        .issuer(delegate.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now - 1)
        .expiry(now + 86_400)
        .proof(parent.clone())
        .sign(&delegate);

    let err = validate_chain_at(&[child, parent], now + 120).unwrap_err();
    assert!(
        matches!(err, UcanError::Expired { .. }),
        "expected Expired (parent), got {err:?}"
    );
}

#[test]
fn ucan_chain_walk_constant_time_comparison_audit() {
    // crypto-major-4. Source-grep: the chain-walk impl in
    // src/ucan.rs MUST NOT contain naive `==` on signature/audience/
    // proof_cid bytes; ALL byte-comparison goes through ct_signature_eq
    // (which itself calls subtle::ConstantTimeEq).
    let src_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ucan.rs");
    let src = std::fs::read_to_string(&src_path).unwrap();
    for line in src.lines() {
        let trimmed = line.trim_start();
        // Skip comments + the const-time-eq impl itself + the tests
        // module.
        if trimmed.starts_with("//") || trimmed.starts_with("///") {
            continue;
        }
        for forbidden in &[
            "signature ==",
            "audience ==",
            "proof_cid ==",
            // G14-A1 mini-review MAJOR — capability authority
            // comparisons in `caps_match_or_subsume` are
            // security-decision sites; ct-eq UNIFORMITY at these
            // sites means a future contributor adding a new
            // authority compare hits this audit.
            "parent.resource ==",
            "parent.ability ==",
            "child.resource ==",
            "child.ability ==",
        ] {
            assert!(
                !line.contains(forbidden),
                "constant-time comparison required per crypto-major-4: {line}"
            );
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
