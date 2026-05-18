//! Wave-E HELD #1205 / #624 / META #684 closure-pin — Compromise #21
//! (Ed25519 signed-manifest + registry verification) survives crossing
//! the napi cdylib boundary.
//!
//! ## META #684 concern
//!
//! SECURITY-POSTURE.md Compromise #21 is marked CLOSED (Phase-3
//! `install_module` signed-manifest + UCAN proof-chain). META #684
//! flagged that at HEAD the napi cdylib boundary BYPASSED the
//! signed-manifest path entirely: the binding exposed ONLY the
//! unsigned `installModule` (hard-coded
//! `ManifestVerifyArgs::unsigned_development()`), so EVERY module
//! installed through the Node.js binding skipped signature
//! verification regardless of whether the manifest was signed — the
//! live face of Compromise #21 at the cdylib construction site.
//!
//! ## Ground-truth at `20fb5c78` (verified)
//!
//! RESOLVED-ON-MAIN: prior campaign work (PR #1282/#1290) added the
//! `installModuleSigned` napi surface (`bindings/napi/src/lib.rs`),
//! threading `benten_engine::manifest_signing::ManifestVerifyArgs::
//! registry(&registry_pubkey, &audience, now)` into the engine's
//! `install_module` — which enforces signature verification BEFORE
//! persistence for all non-Unsigned modes
//! (`engine_modules.rs::install_module` g14-c-mr-1 gate;
//! `verify_manifest_with_mode` rejects `RegistryInvalid`).
//!
//! This pin is the would-FAIL-if-reverted closure proof for the
//! napi-consumed seam: a manifest whose signature does NOT verify
//! against the registry public key MUST be rejected at the seam the
//! `installModuleSigned` binding wraps. Reverting the napi signed
//! surface to unsigned-only (or the engine gate to skip verification)
//! makes `tampered_signature_rejected_through_napi_consumed_signed_seam`
//! fail (the install would return `Ok(cid)`).
//!
//! ## Why an engine-seam test in the napi crate
//!
//! The `#[napi] JsEngine` cannot be instantiated outside Node.js, so
//! — exactly like the sibling
//! `cap_delegate_napi_resolved_scope_regression_guard.rs` — the
//! production arm is `Engine::install_module(.., ManifestVerifyArgs::
//! registry(..))`, the seam `installModuleSigned` wraps verbatim.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "in-process-test")]

use benten_engine::Engine;
use benten_engine::manifest_signing::{ManifestVerifyArgs, sign_manifest};
use benten_engine::module_manifest::{ModuleManifest, ModuleManifestEntry};
use benten_id::did::Did;
use benten_id::keypair::Keypair;

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
        host_fns: None,
        signature: None,
    }
}

#[test]
fn tampered_signature_rejected_through_napi_consumed_signed_seam() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Sign with the AUTHOR's key, then verify against a DIFFERENT
    // registry key — the registry-path Ed25519 verify MUST fail. This
    // is exactly the bytes `installModuleSigned` feeds the engine
    // seam (registry pubkey + audience + clock).
    let author_kp = Keypair::generate();
    let registry_kp = Keypair::generate(); // != author
    let signed = sign_manifest(&fixture_manifest("acme.posts"), &author_kp).unwrap();
    let cid = signed.compute_cid().unwrap();

    // Audience must be a real resolvable DID for the registry-mode
    // path; derive one from a keypair (mirrors the napi binding's
    // `engine_audience_did.resolve()` gate).
    let audience_kp = Keypair::generate();
    let audience = Did::from_public_key(audience_kp.public_key());
    let _ = audience.resolve().expect("audience DID resolvable");

    let verify_args =
        ManifestVerifyArgs::registry(registry_kp.public_key(), &audience, 1_700_000_000);

    let result = engine.install_module(signed, cid, verify_args);

    assert!(
        result.is_err(),
        "LOAD-BEARING #1205/META-#684: a manifest whose signature does NOT \
         verify against the registry key MUST be rejected at the \
         napi-consumed signed-install seam; `Ok(cid)` means Compromise #21 \
         is bypassed at the cdylib boundary (unsigned-only regression)"
    );
}

#[test]
fn valid_registry_signature_admits_through_napi_consumed_signed_seam() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Sign AND verify with the same registry key + correct audience —
    // the OK edge. Proves the seam is not merely deny-all (which would
    // be a degenerate "enforcement" that breaks legitimate installs).
    let registry_kp = Keypair::generate();
    let signed = sign_manifest(&fixture_manifest("acme.posts"), &registry_kp).unwrap();
    let cid = signed.compute_cid().unwrap();

    let audience_kp = Keypair::generate();
    let audience = Did::from_public_key(audience_kp.public_key());
    let _ = audience.resolve().expect("audience DID resolvable");

    let verify_args =
        ManifestVerifyArgs::registry(registry_kp.public_key(), &audience, 1_700_000_000);

    let result = engine.install_module(signed, cid, verify_args);
    assert!(
        result.is_ok(),
        "valid registry-signed manifest MUST install through the \
         napi-consumed signed seam (not deny-all): {result:?}"
    );
}
