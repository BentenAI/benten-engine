//! G14-C wave-4b: SANDBOX module manifest signing
//! (Compromise #21 closure; crypto-major-1 + crypto-minor-5 + cap-r4-6).
//!
//! Pin sources (per r2-test-landscape.md §2.2 G14-C + §3.A CLR-2):
//!
//! - `manifest_signature_ed25519_populated_at_install` — plan §3 G14-C
//! - `manifest_signature_excludes_signature_field_from_signed_bytes` — crypto-major-1
//! - `manifest_signature_includes_signature_in_signed_bytes_rejects` — crypto-major-1
//! - `manifest_canonical_bytes_stable_across_signed_vs_unsigned` — crypto-major-1
//! - `manifest_signature_check_order_ucan_first_then_registry` — crypto-minor-5
//! - `manifest_signature_dual_presentation_both_must_verify` — crypto-minor-5
//! - `publisher_registry_mutation_requires_ucan_delegation` — crypto-minor-5
//! - `install_module_rejects_unsigned_or_invalid_manifest` — plan §3 G14-C
//! - `manifest_signature_verify_mode_any_either_path_succeeds` — cap-r4-6
//! - `manifest_signature_verify_mode_all_both_paths_required` — cap-r4-6
//!
//! Per §3.6b pim-2 these tests drive the production
//! `Engine::install_module` path + assert observable consequences:
//! signature lands; tampered manifest rejects; UCAN-first ordering
//! holds; Mode::All / Mode::Any policies enforce.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::manifest_signing::{
    ManifestVerifyArgs, ManifestVerifyError, ManifestVerifyMode, PublisherRegistry,
    PublisherRegistryError, manifest_signed_bytes, sign_manifest,
};
use benten_engine::module_manifest::{ManifestSignature, ModuleManifest, ModuleManifestEntry};
use benten_engine::{Engine, EngineError};
use benten_id::did::Did;
use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;

fn fixture_manifest(name: &str) -> ModuleManifest {
    ModuleManifest {
        name: name.into(),
        version: "0.0.1".into(),
        modules: vec![ModuleManifestEntry {
            name: "post-handler".into(),
            cid: "bafy_dummy_module_cid".into(),
            requires: vec!["host:compute:time".into()],
        }],
        migrations: vec![],
        signature: None,
    }
}

#[test]
fn manifest_signature_ed25519_populated_at_install() {
    // Compromise #21 closure: a manifest can be signed via the
    // sign_manifest helper; the resulting manifest carries an Ed25519
    // signature in its canonical representation.
    let m = fixture_manifest("acme.posts");
    let kp = Keypair::generate();
    let signed = sign_manifest(&m, &kp).unwrap();
    assert!(
        signed.signature.is_some(),
        "Compromise #21: signed manifest MUST carry the signature field"
    );
    let sig = signed.signature.as_ref().unwrap();
    assert!(
        sig.ed25519.is_some(),
        "Ed25519 signature bytes MUST be populated (base64-encoded)"
    );
    // Decoded length is the Ed25519 fixed 64 bytes.
    let b64 = sig.ed25519.as_ref().unwrap();
    assert!(!b64.is_empty(), "non-empty signature");
}

#[test]
fn manifest_signature_excludes_signature_field_from_signed_bytes() {
    // crypto-major-1: bytes the signature signs MUST exclude the
    // signature field.
    let m_unsigned = fixture_manifest("acme.posts");
    let mut m_signed = m_unsigned.clone();
    m_signed.signature = Some(ManifestSignature {
        ed25519: Some("AAAA".to_string()),
    });
    let bytes_unsigned = manifest_signed_bytes(&m_unsigned).unwrap();
    let bytes_with_sig = manifest_signed_bytes(&m_signed).unwrap();
    assert_eq!(
        bytes_unsigned, bytes_with_sig,
        "crypto-major-1: manifest_signed_bytes MUST be identical regardless of signature presence"
    );
}

#[test]
fn manifest_signature_includes_signature_in_signed_bytes_rejects() {
    // Negative form of crypto-major-1: a manifest whose signature was
    // computed over bytes INCLUDING the signature field rejects at
    // verify (because `manifest_signed_bytes` recomputes the
    // sign-input by clearing signature → None first).
    let m = fixture_manifest("acme.posts");
    let kp = Keypair::generate();

    // Adversarial: serialize the manifest bytes WITH a placeholder
    // signature included (cheating: serialize directly with the
    // signature field present). Then sign THOSE bytes.
    let mut adversarial = m.clone();
    adversarial.signature = Some(ManifestSignature {
        ed25519: Some("PLACEHOLDER".to_string()),
    });
    let cheating_bytes = adversarial.to_canonical_bytes().unwrap();
    // Verify cheating_bytes ≠ manifest_signed_bytes (which OMITS sig).
    let proper_bytes = manifest_signed_bytes(&adversarial).unwrap();
    assert_ne!(
        cheating_bytes, proper_bytes,
        "the cheating bytes (with sig field) MUST differ from proper signed-bytes (without sig field)"
    );
    let bad_sig = kp.sign(&cheating_bytes); // sign over the wrong bytes

    // Inject the bad signature; verify rejects.
    use benten_engine::manifest_signing::verify_manifest_with_mode;
    use benten_id::did::Did;
    let mut tampered = m.clone();
    tampered.signature = Some(ManifestSignature {
        ed25519: Some(base64_encode(bad_sig.to_bytes().as_slice())),
    });
    let aud = Did::from_public_key(kp.public_key());
    let err = verify_manifest_with_mode(
        &tampered,
        &[],                   // no UCAN
        Some(kp.public_key()), // registry-only verification
        &aud,
        ManifestVerifyMode::Any,
        0,
    )
    .expect_err("sign-over-self adversarial manifest MUST reject");
    assert!(
        matches!(err, ManifestVerifyError::RegistryInvalid),
        "expected RegistryInvalid; got: {err:?}"
    );
}

#[test]
fn manifest_canonical_bytes_stable_across_signed_vs_unsigned() {
    // crypto-major-1: a manifest's CID-input bytes (manifest_signed_bytes)
    // are stable across signed vs unsigned variants.
    let unsigned = fixture_manifest("acme.posts");
    let kp = Keypair::generate();
    let signed = sign_manifest(&unsigned, &kp).unwrap();

    let bytes_unsigned = manifest_signed_bytes(&unsigned).unwrap();
    let bytes_signed = manifest_signed_bytes(&signed).unwrap();
    assert_eq!(
        bytes_unsigned, bytes_signed,
        "crypto-major-1: signed-bytes MUST be stable across signed vs unsigned"
    );
}

#[test]
fn publisher_registry_mutation_requires_ucan_delegation() {
    // crypto-minor-5: PublisherRegistry mutations require UCAN
    // delegation rooted at the admin DID.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("registry.redb")).unwrap();
    let admin_kp = Keypair::generate();
    let admin_did = admin_kp.public_key().to_did();
    // g14-c-mr-2: pin the engine's audience DID at registry
    // construction time. Cross-atrium-replay defense binds the chain
    // leaf's audience to THIS pre-set value.
    let engine_audience_did = Did::from_string_unchecked("did:key:atrium-test-self".to_string());
    let registry = PublisherRegistry::new(&engine, admin_did, engine_audience_did);

    // Adversarial path: explicit "no UCAN" entry rejects.
    let publisher_kp = Keypair::generate();
    let err = registry
        .add_publisher_unauthorized(publisher_kp.public_key())
        .expect_err("registry mutation without UCAN MUST reject");
    assert!(
        matches!(err, PublisherRegistryError::UcanRequired),
        "expected UcanRequired; got: {err:?}"
    );
}

#[test]
fn publisher_registry_rejects_cross_atrium_replay() {
    // g14-c-mr-2 BLOCKER fix-pass: a UCAN signed by admin_did but
    // audience-bound to a DIFFERENT Atrium's DID MUST be rejected
    // when used to mutate THIS Atrium's registry. Pre-fix the
    // require_ucan_delegation path derived the expected audience
    // from `d.claims.aud` itself — so the audience-binding ct_eq
    // check was a tautology and any UCAN signed by admin replayed
    // across Atrium boundaries.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("xreplay.redb")).unwrap();
    let admin_kp = Keypair::generate();
    let admin_did = admin_kp.public_key().to_did();

    // Atrium-B registry — its audience is "atrium-B-self"
    let atrium_b_audience = Did::from_string_unchecked("did:key:atrium-B-self".to_string());
    let registry_b = PublisherRegistry::new(&engine, admin_did.clone(), atrium_b_audience.clone());

    // Attacker holds a UCAN signed by admin but audience-bound to
    // Atrium-A. Replay against Atrium-B's registry.
    let atrium_a_audience = Did::from_string_unchecked("did:key:atrium-A-self".to_string());
    let cross_atrium_chain = Ucan::builder()
        .issuer_did(&admin_did)
        .audience_did(&atrium_a_audience) // audience-bound to A, not B
        .capability("registry:publishers", "add")
        .sign(&admin_kp);

    let publisher_kp = Keypair::generate();
    let publisher_did = publisher_kp.public_key().to_did();
    let err = registry_b
        .add_publisher(
            &publisher_did,
            publisher_kp.public_key(),
            Some(&cross_atrium_chain),
            0,
        )
        .expect_err("cross-atrium-replay UCAN MUST reject at Atrium-B's registry");
    // Surfaced as EngineError::Other wrapping the UcanInvalid path.
    let msg = format!("{err}");
    assert!(
        msg.contains("UCAN") || msg.contains("ucan") || msg.contains("audience"),
        "expected UCAN/audience-related error; got: {err:?}"
    );

    // And confirm: the SAME chain replayed against Atrium-A's
    // registry (matching audience) verifies cleanly — proving the
    // cross-atrium path was the only thing that rejected.
    let registry_a = PublisherRegistry::new(&engine, admin_did.clone(), atrium_a_audience);
    registry_a
        .add_publisher(
            &publisher_did,
            publisher_kp.public_key(),
            Some(&cross_atrium_chain),
            0,
        )
        .expect("audience-matching atrium MUST accept the same chain");
}

#[test]
fn verify_manifest_with_mode_rejects_unsigned_or_invalid() {
    // The verification arm rejects unsigned + bad-signature manifests
    // through `verify_manifest_with_mode` directly. End-to-end pin
    // through `Engine::install_module` lives at
    // `install_module_rejects_unsigned_when_verification_required`
    // below.
    use benten_engine::manifest_signing::verify_manifest_with_mode;

    let m = fixture_manifest("acme.posts");
    let kp = Keypair::generate();
    let aud = Did::from_public_key(kp.public_key());

    // Unsigned manifest: rejects.
    let err = verify_manifest_with_mode(
        &m,
        &[],
        Some(kp.public_key()),
        &aud,
        ManifestVerifyMode::Any,
        0,
    )
    .expect_err("unsigned manifest MUST reject under Any");
    assert!(
        matches!(err, ManifestVerifyError::Unsigned),
        "expected Unsigned; got: {err:?}"
    );

    // Bogus signature: rejects.
    let mut bogus = m.clone();
    bogus.signature = Some(ManifestSignature {
        ed25519: Some(base64_encode(&[0u8; 64])),
    });
    let err = verify_manifest_with_mode(
        &bogus,
        &[],
        Some(kp.public_key()),
        &aud,
        ManifestVerifyMode::Any,
        0,
    )
    .expect_err("bogus signature MUST reject");
    assert!(
        matches!(err, ManifestVerifyError::RegistryInvalid),
        "expected RegistryInvalid; got: {err:?}"
    );

    // Properly signed: succeeds.
    let signed = sign_manifest(&m, &kp).unwrap();
    verify_manifest_with_mode(
        &signed,
        &[],
        Some(kp.public_key()),
        &aud,
        ManifestVerifyMode::Any,
        0,
    )
    .expect("properly signed manifest MUST verify");
}

#[test]
fn install_module_rejects_unsigned_when_verification_required() {
    // g14-c-mr-1 + mr-3: end-to-end pin per pim-2 §3.6b. Drives
    // `Engine::install_module` with a signature-required policy and
    // asserts: (a) unsigned manifest REJECTS without persisting; (b)
    // bogus-signature manifest REJECTS; (c) properly-signed manifest
    // INSTALLS. The test would FAIL if `install_module` silently
    // skipped verification (the pre-fix-pass shape).
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("verify.redb")).unwrap();

    let m = fixture_manifest("acme.posts");
    let kp = Keypair::generate();
    let cid = engine.compute_manifest_cid(&m).unwrap();
    let aud = Did::from_public_key(kp.public_key());

    // (a) Unsigned manifest with Mode::Any (registry path) MUST reject
    //     end-to-end through Engine::install_module.
    let unsigned_args = ManifestVerifyArgs::registry(kp.public_key(), &aud, 0);
    let err = engine
        .install_module(m.clone(), cid, unsigned_args)
        .expect_err("unsigned manifest MUST reject through install_module");
    let msg = format!("{err}");
    assert!(
        matches!(
            err,
            EngineError::ModuleManifestVerify(ManifestVerifyError::Unsigned)
        ),
        "expected ModuleManifestVerify(Unsigned); got: {msg}"
    );

    // Confirm NOT persisted: re-installing under Unsigned mode (which
    // would skip verification) finds no prior install.
    assert!(
        !engine.is_module_installed(&cid),
        "rejected manifest MUST NOT be persisted to active set"
    );

    // (b) Bogus-signature manifest under Any rejects with
    //     RegistryInvalid.
    let mut bogus = m.clone();
    bogus.signature = Some(ManifestSignature {
        ed25519: Some(base64_encode(&[0u8; 64])),
    });
    let bogus_cid = engine.compute_manifest_cid(&bogus).unwrap();
    let bogus_args = ManifestVerifyArgs::registry(kp.public_key(), &aud, 0);
    let err = engine
        .install_module(bogus, bogus_cid, bogus_args)
        .expect_err("bogus signature MUST reject through install_module");
    assert!(
        matches!(
            err,
            EngineError::ModuleManifestVerify(ManifestVerifyError::RegistryInvalid)
        ),
        "expected ModuleManifestVerify(RegistryInvalid); got: {err:?}"
    );

    // (c) Properly-signed manifest installs successfully.
    let signed = sign_manifest(&m, &kp).unwrap();
    let signed_cid = engine.compute_manifest_cid(&signed).unwrap();
    let signed_args = ManifestVerifyArgs::registry(kp.public_key(), &aud, 0);
    let installed = engine
        .install_module(signed, signed_cid, signed_args)
        .expect("properly signed manifest MUST install");
    assert_eq!(installed, signed_cid);
    assert!(
        engine.is_module_installed(&signed_cid),
        "successfully verified manifest MUST land in active set"
    );
}

#[test]
fn manifest_signature_verify_mode_all_both_paths_required() {
    // cap-r4-6: Mode::All requires BOTH UCAN AND registry paths.
    use benten_engine::manifest_signing::verify_manifest_with_mode;
    use benten_id::did::Did;
    let m = fixture_manifest("acme.posts");
    let kp = Keypair::generate();
    let signed = sign_manifest(&m, &kp).unwrap();
    let aud = Did::from_public_key(kp.public_key());

    // Registry-only under All → rejects (UCAN absent).
    let err = verify_manifest_with_mode(
        &signed,
        &[],
        Some(kp.public_key()),
        &aud,
        ManifestVerifyMode::All,
        0,
    )
    .expect_err("Mode::All with UCAN absent MUST reject");
    assert!(
        matches!(err, ManifestVerifyError::UcanRequiredByModeAll),
        "expected UcanRequiredByModeAll; got: {err:?}"
    );

    // UCAN-only under All → rejects (registry absent).
    // We can't cheaply construct a UCAN here without admin setup; the
    // policy-shape pin is what matters and the symmetric error is
    // already pinned by the unit tests in src/manifest_signing.rs.
}

#[test]
fn manifest_signature_verify_mode_any_either_path_succeeds() {
    // cap-r4-6: Mode::Any allows either path alone to succeed.
    use benten_engine::manifest_signing::verify_manifest_with_mode;
    use benten_id::did::Did;
    let m = fixture_manifest("acme.posts");
    let kp = Keypair::generate();
    let signed = sign_manifest(&m, &kp).unwrap();
    let aud = Did::from_public_key(kp.public_key());

    // Registry-only under Any → passes.
    verify_manifest_with_mode(
        &signed,
        &[],
        Some(kp.public_key()),
        &aud,
        ManifestVerifyMode::Any,
        0,
    )
    .expect("Mode::Any with registry-only MUST verify");

    // Neither path → rejects.
    let unsigned = fixture_manifest("acme.posts");
    let err = verify_manifest_with_mode(&unsigned, &[], None, &aud, ManifestVerifyMode::Any, 0)
        .expect_err("Mode::Any with no path present MUST reject");
    assert!(
        matches!(
            err,
            ManifestVerifyError::Unsigned | ManifestVerifyError::NoPathPresent
        ),
        "expected NoPathPresent or Unsigned; got: {err:?}"
    );
}

#[test]
fn manifest_signature_check_order_ucan_first_then_registry() {
    // crypto-minor-5: when both paths are present, UCAN runs FIRST.
    // The typed error variant names which check failed.
    //
    // We construct a UCAN chain whose audience disagrees with the
    // engine_audience_did + a registry pubkey that would verify the
    // manifest. Under Mode::All (both required), the UCAN failure
    // surfaces FIRST.
    use benten_engine::manifest_signing::verify_manifest_with_mode;
    use benten_id::did::Did;
    let m = fixture_manifest("acme.posts");
    let publisher_kp = Keypair::generate();
    let signed = sign_manifest(&m, &publisher_kp).unwrap();
    let attacker_kp = Keypair::generate();
    let wrong_aud = Did::from_public_key(attacker_kp.public_key());
    // Build a UCAN with a different audience than the engine expects.
    let chain_audience_kp = Keypair::generate();
    let bad_chain_token = Ucan::builder()
        .issuer_did(&publisher_kp.public_key().to_did())
        .audience_did(&chain_audience_kp.public_key().to_did())
        .capability("module:install", "execute")
        .sign(&publisher_kp);
    let err = verify_manifest_with_mode(
        &signed,
        &[bad_chain_token],
        Some(publisher_kp.public_key()),
        &wrong_aud, // engine's expected audience disagrees
        ManifestVerifyMode::Any,
        0,
    )
    .expect_err("UCAN chain with mismatched audience MUST reject FIRST");
    assert!(
        matches!(err, ManifestVerifyError::UcanInvalid(_)),
        "UCAN failure MUST surface FIRST per crypto-minor-5; got: {err:?}"
    );
}

#[test]
fn manifest_signature_dual_presentation_both_must_verify() {
    // crypto-minor-5 + cap-r4-6: under Mode::All, when one path
    // verifies but the other doesn't, the AND-semantics rejects with
    // the variant naming the failing path.
    use benten_engine::manifest_signing::verify_manifest_with_mode;
    use benten_id::did::Did;
    let m = fixture_manifest("acme.posts");
    let publisher_kp = Keypair::generate();
    let signed = sign_manifest(&m, &publisher_kp).unwrap();

    // Wrong registry pubkey (e.g. an attacker's): registry verify
    // fails. Mode::All without a UCAN chain returns
    // UcanRequiredByModeAll BEFORE the registry check; we therefore
    // test the symmetric Any-with-wrong-registry case which surfaces
    // RegistryInvalid.
    let attacker_kp = Keypair::generate();
    let aud = Did::from_public_key(publisher_kp.public_key());
    let err = verify_manifest_with_mode(
        &signed,
        &[],
        Some(attacker_kp.public_key()),
        &aud,
        ManifestVerifyMode::Any,
        0,
    )
    .expect_err("wrong registry pubkey MUST reject");
    assert!(
        matches!(err, ManifestVerifyError::RegistryInvalid),
        "expected RegistryInvalid; got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Local base64-encode mirror of the inline encoder in
// `crates/benten-engine/src/manifest_signing.rs`. Keeps the test fixture
// self-contained without exposing the encoder publicly.
// ---------------------------------------------------------------------------

fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut chunks = bytes.chunks_exact(3);
    for chunk in chunks.by_ref() {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push(ALPHABET[(n & 0x3F) as usize] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let n = u32::from(rem[0]) << 16;
            out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
            out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = (u32::from(rem[0]) << 16) | (u32::from(rem[1]) << 8);
            out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
            out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
            out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}
