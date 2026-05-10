//! R4-FP-R3-B RED-PHASE pins: manifest temporal-binding (G14-C wave-4b;
//! sec-r4r1-1 BLOCKER + sec-r1-1 closure pins).
//!
//! Pin sources (per R4 R1 security-auditor lens, finding sec-r4r1-1):
//!
//! - `tests/manifest_install_rejected_when_signing_ucan_revoked_between_sign_and_install` — sec-r4r1-1 (a)
//! - `tests/manifest_signature_install_time_consults_ucan_nbf_exp_on_signing_chain` — sec-r4r1-1 (b)
//! - `tests/manifest_install_rejects_replay_with_same_nonce` — sec-r4r1-1 (c)
//! - `tests/manifest_install_newer_signed_manifest_supersedes_older` — sec-r4r1-1 (d)
//!
//! ## Architectural intent (sec-r4r1-1 BLOCKER closure)
//!
//! sec-r1-1 (R1 BLOCKER) enumerated four specific manifest-level
//! temporal-binding pin shapes that defend against the Sigstore /
//! cosign expiry / Notary v2 revocation / npm-provenance freshness
//! baseline industry attacks:
//!
//! 1. **Reinstall-with-same-nonce rejection** — manifest envelope
//!    carries a nonce + sign-time; reinstall with same nonce rejects.
//! 2. **Install-time consultation of UCAN nbf/exp** — install-time
//!    chain-walk applies nbf/exp at install-time wallclock, not at
//!    sign-time. CLR-2's pins operate at the UCAN-chain-walk site;
//!    THIS pin operates at the manifest-install flow.
//! 3. **Sign-time-to-install-time revocation propagation** — install
//!    rejects when a UCAN parent in the signing chain was revoked
//!    since sign-time.
//! 4. **Newer-signed-manifest-supersedes-older** — supersession
//!    ordering is well-defined (sign-time monotonicity within
//!    publisher's UCAN-rooted authority).
//!
//! Per R4 R1 security-auditor lens, the triage's "closed via CLR-2"
//! disposition conflated UCAN-chain-walk-site temporal binding with
//! manifest-install-flow temporal binding — these operate on different
//! code paths. A signed manifest installed at T0 against a UCAN chain
//! valid at T0 will pass the existing R3 corpus pins even if the
//! chain's nbf/exp expires by install-execution time, even if the
//! parent UCAN is revoked between sign-time and install-time, and
//! even if the manifest is replayed against an older-signed-manifest's
//! slot. These four pins close that gap end-to-end.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-C
//! implementer un-ignores AND replaces stub bodies. Per §3.6b pim-2
//! these tests must drive the production `Engine::install_module`
//! path + assert observable consequences (typed-error variants on the
//! reject paths; CID/ordering observability on the supersession path).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — manifest temporal binding (a) install rejects when signing UCAN revoked between sign and install. G14-C wave-4b shipped Compromise #17/#18/#21 closures (PR #110); test body pins specific revoke-between-sign-and-install defensive contract; un-ignore at §2.3 (i) WriteContext threading landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn manifest_install_rejected_when_signing_ucan_revoked_between_sign_and_install() {
    // sec-r4r1-1 BLOCKER (a) pin. The signing UCAN proof chain MUST
    // be re-validated at install-time against the durable revocation
    // store; an entry revoked between sign-time and install-time
    // observably rejects the install.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let admin_kp = benten_id::keypair::Keypair::generate();
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Admin issues UCAN to publisher granting host:module:install:
    //   let parent_ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(admin_kp.public_key().to_did())
    //       .audience(publisher_kp.public_key().to_did())
    //       .capability("host:module:install", "*")
    //       .nbf_now()
    //       .exp_in_secs(3600)
    //       .sign(&admin_kp).unwrap();
    //   engine.caps().install_proof(&parent_ucan).unwrap();
    //
    //   // Sign the manifest at T=now with the publisher key + UCAN chain:
    //   let manifest = ...;
    //   let signed = benten_engine::sign_manifest_with_chain(
    //       &manifest, &publisher_kp, &[parent_ucan.clone()]).unwrap();
    //
    //   // BETWEEN sign-time and install-time: admin revokes parent UCAN.
    //   engine.caps().revoke(&parent_ucan.cid()).unwrap();
    //
    //   // install_module MUST observe the revocation + reject:
    //   let err = engine.install_module(&signed, &bytes).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::EngineError::ManifestSigningChainRevoked { .. }),
    //       "install_module must reject when signing chain revoked between sign and install per sec-r4r1-1 (a)");
    //
    // OBSERVABLE consequence: an attacker who captures a signed manifest
    // pre-revocation cannot install it post-revocation. Defends against
    // the "stolen + replayed" attack class at the install-flow seam,
    // not just the chain-walk seam.
    unimplemented!("G14-C wires install-time chain-revocation re-check per sec-r4r1-1 (a) BLOCKER");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — manifest temporal binding (b) install-time consults UCAN nbf/exp at install-wallclock. G14-C shipped; test body pins UCAN nbf/exp install-time-consultation contract; un-ignore at §2.3 (i) landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn manifest_signature_install_time_consults_ucan_nbf_exp_on_signing_chain() {
    // sec-r4r1-1 BLOCKER (b) pin. Install-time chain-walk applies
    // UCAN nbf/exp at install-time wallclock, NOT at sign-time. This
    // is the install-flow analogue of CLR-2's chain-walk-site
    // assertion; orthogonal to chain-walk-site temporal pins.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let admin_kp = benten_id::keypair::Keypair::generate();
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //
    //   // UCAN with nbf far in the past + short exp window:
    //   let issuance_secs = 1_000_000_000;
    //   let exp_secs = issuance_secs + 60;
    //   let parent_ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(admin_kp.public_key().to_did())
    //       .audience(publisher_kp.public_key().to_did())
    //       .capability("host:module:install", "*")
    //       .nbf(issuance_secs)
    //       .exp(exp_secs)
    //       .sign(&admin_kp).unwrap();
    //   engine.caps().install_proof(&parent_ucan).unwrap();
    //
    //   let manifest = ...;
    //   let signed = benten_engine::sign_manifest_with_chain(
    //       &manifest, &publisher_kp, &[parent_ucan]).unwrap();
    //
    //   // Install at T = exp + 120 (UCAN expired): MUST reject.
    //   let err = engine.install_module_at(&signed, &bytes, exp_secs + 120).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::EngineError::ManifestSigningChainExpired { .. }),
    //       "install_module must reject when signing chain post-exp at install-wallclock per sec-r4r1-1 (b)");
    //
    //   // Install at T = exp - 30 (UCAN within window): succeeds.
    //   engine.install_module_at(&signed, &bytes, exp_secs - 30).unwrap();
    //
    // OBSERVABLE consequence: the install flow consults install-time
    // wallclock, not sign-time. Defends against the "sign at T,
    // install much later when chain has expired" attack class at the
    // install-flow seam.
    unimplemented!(
        "G14-C wires install-time UCAN nbf/exp re-check at install-wallclock per sec-r4r1-1 (b)"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — manifest temporal binding (c) install rejects replay with same nonce. G14-C shipped; test body pins replay-rejection-by-nonce contract; un-ignore at §2.3 (i) landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn manifest_install_rejects_replay_with_same_nonce() {
    // sec-r4r1-1 BLOCKER (c) pin. The manifest envelope carries a
    // nonce + sign-time; reinstall with the same nonce rejects via a
    // nonce-store at the engine.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //   let manifest = ...;
    //
    //   // Sign the manifest with a specific nonce + sign-time:
    //   let nonce = benten_engine::ManifestNonce::generate();
    //   let sign_time = 1_000_000_000;
    //   let signed = benten_engine::sign_manifest_with_envelope(
    //       &manifest, &publisher_kp, nonce.clone(), sign_time).unwrap();
    //
    //   // First install: succeeds.
    //   engine.install_module(&signed, &bytes).unwrap();
    //
    //   // Adversarial replay: re-present the SAME signed manifest
    //   // (same nonce + same sign-time) for re-install. MUST reject.
    //   let err = engine.install_module(&signed, &bytes).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::EngineError::ManifestNonceReplay { .. }),
    //       "install_module must reject reinstall with same envelope nonce per sec-r4r1-1 (c)");
    //
    // OBSERVABLE consequence: an attacker capturing a signed manifest
    // off the wire cannot replay-install it as a substitute for an
    // attempted upgrade. Defends against "captured-at-T0, replayed-at-T1
    // to undo a security upgrade" attacks. Mirrors the device-attestation
    // nonce-store discipline at the manifest-install seam.
    unimplemented!(
        "G14-C wires manifest-envelope nonce-store rejection of replay per sec-r4r1-1 (c)"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — manifest temporal binding (d) newer-signed manifest supersedes older. G14-C shipped; test body pins newer-supersedes-older contract; un-ignore at §2.3 (i) landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn manifest_install_newer_signed_manifest_supersedes_older() {
    // sec-r4r1-1 BLOCKER (d) pin. Supersession ordering is
    // well-defined: a newer-signed manifest (later sign-time within
    // publisher's UCAN-rooted authority) supersedes an older one
    // for the same logical-module-id.
    //
    // Concrete shape:
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //   let module_id = benten_engine::LogicalModuleId::new("com.example.foo").unwrap();
    //
    //   // Sign + install v1 at T=1000:
    //   let manifest_v1 = ManifestBuilder::new(&module_id).version("1.0.0").build();
    //   let signed_v1 = benten_engine::sign_manifest_with_envelope(
    //       &manifest_v1, &publisher_kp, ManifestNonce::generate(), 1000).unwrap();
    //   let cid_v1 = engine.install_module(&signed_v1, &bytes_v1).unwrap();
    //
    //   // Sign + install v2 at T=2000 (later sign-time):
    //   let manifest_v2 = ManifestBuilder::new(&module_id).version("2.0.0").build();
    //   let signed_v2 = benten_engine::sign_manifest_with_envelope(
    //       &manifest_v2, &publisher_kp, ManifestNonce::generate(), 2000).unwrap();
    //   let cid_v2 = engine.install_module(&signed_v2, &bytes_v2).unwrap();
    //
    //   // v2 supersedes v1 at the active-set pointer:
    //   assert_eq!(engine.active_module_cid(&module_id).unwrap(), cid_v2,
    //       "newer-signed manifest must supersede older per sec-r4r1-1 (d)");
    //
    //   // Adversarial downgrade: try to re-install OLDER v1 over newer v2:
    //   let err = engine.install_module(&signed_v1, &bytes_v1).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::EngineError::ManifestSupersededByNewer { .. }),
    //       "install_module must reject older-sign-time over newer-sign-time per sec-r4r1-1 (d)");
    //
    //   // Active set still points at v2:
    //   assert_eq!(engine.active_module_cid(&module_id).unwrap(), cid_v2);
    //
    // OBSERVABLE consequence: supersession is well-ordered at sign-time;
    // an attacker cannot downgrade a security-fix release by replaying
    // an older signed manifest. Defends against the "Notary v2 freshness"
    // attack pattern at the manifest-install seam.
    unimplemented!("G14-C wires supersession ordering at install_module per sec-r4r1-1 (d)");
}
