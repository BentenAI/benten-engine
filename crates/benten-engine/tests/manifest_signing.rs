//! R3-B RED-PHASE pins: SANDBOX module manifest signing
//! (G14-C wave-4b; Compromise #21 + crypto-major-1 + crypto-minor-5).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-C +
//! §3.A CLR-2 (manifest_signature_check_order_ucan_first_then_registry)):
//!
//! - `tests/manifest_signature_ed25519_populated_at_install` — plan §3 G14-C
//! - `tests/manifest_signature_excludes_signature_field_from_signed_bytes` — crypto-major-1
//! - `tests/manifest_signature_includes_signature_in_signed_bytes_rejects` — crypto-major-1
//! - `tests/manifest_canonical_bytes_stable_across_signed_vs_unsigned` — crypto-major-1
//! - `tests/manifest_signature_check_order_ucan_first_then_registry` — crypto-minor-5
//! - `tests/manifest_signature_dual_presentation_both_must_verify` — crypto-minor-5
//! - `tests/publisher_registry_mutation_requires_ucan_delegation` — crypto-minor-5
//! - `tests/install_module_rejects_unsigned_or_invalid_manifest` — plan §3 G14-C
//!
//! ## Architectural intent
//!
//! Compromise #21 (manifest signing) closes at G14-C. Per crypto-
//! major-1 the `signature` field MUST be EXCLUDED from the bytes
//! that get signed (otherwise signing recursively depends on itself).
//! Per crypto-minor-5 the verification order is UCAN delegation
//! check FIRST then publisher-registry check; both must verify when
//! dual-presented.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-C
//! implementer un-ignores. Per §3.6b pim-2 these tests must drive the
//! production `Engine::install_module` path + assert observable
//! consequences (signature lands in stored manifest; tampered
//! manifest rejects).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-C — plan §3 G14-C — manifest signature populated at install"]
fn manifest_signature_ed25519_populated_at_install() {
    // plan §3 G14-C pin. Compromise #21 closure: a module manifest
    // installed at G14-C carries an Ed25519 signature in the durable
    // representation, NOT just an empty `signature: None` field.
    //
    // Implementer wires:
    //
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //   let manifest = ...;
    //
    //   let signed_manifest = benten_engine::sign_manifest(&manifest, &publisher_kp).unwrap();
    //   let cid = engine.install_module(&signed_manifest, &bytes).unwrap();
    //
    //   let stored = engine.fetch_manifest(&cid).unwrap();
    //   assert!(stored.signature().is_some(),
    //       "Compromise #21: stored manifest must carry Ed25519 signature");
    //   assert_eq!(stored.signature().unwrap().algorithm(), "Ed25519");
    //
    // OBSERVABLE consequence: post-install, `fetch_manifest(cid)`
    // returns a manifest with the Ed25519 signature populated.
    unimplemented!(
        "G14-C wires Ed25519 signature population at install_module() per Compromise #21"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-C — crypto-major-1 — signature field excluded from signed bytes"]
fn manifest_signature_excludes_signature_field_from_signed_bytes() {
    // crypto-major-1 pin. The bytes that get fed into Ed25519 sign()
    // MUST EXCLUDE the `signature` field itself (otherwise the
    // signing operation recursively depends on its own output).
    //
    // Implementer wires:
    //
    //   let manifest = ...;
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Compute the signed-bytes the codepath would use:
    //   let signed_bytes = benten_engine::manifest_signed_bytes(&manifest);
    //   // Synthesize a fake signature + insert it; recompute signed-bytes:
    //   let mut with_sig = manifest.clone();
    //   with_sig.set_signature(Some(benten_engine::ManifestSignature::placeholder()));
    //   let signed_bytes_with = benten_engine::manifest_signed_bytes(&with_sig);
    //
    //   // The signed-bytes must NOT include the signature field:
    //   assert_eq!(signed_bytes, signed_bytes_with,
    //       "manifest_signed_bytes() must produce identical output regardless of signature field");
    //
    // OBSERVABLE consequence: synthesizing a manifest with vs. without
    // the `signature` field produces identical signed-bytes; signing
    // is well-defined.
    unimplemented!("G14-C wires assertion that manifest_signed_bytes() excludes signature field");
}

#[test]
#[ignore = "RED-PHASE: G14-C — crypto-major-1 — manifest with signature in signed-bytes rejects"]
fn manifest_signature_includes_signature_in_signed_bytes_rejects() {
    // crypto-major-1 pin (negative form of the above). If a manifest
    // is presented with a signature that was computed OVER bytes
    // INCLUDING the signature field (a malformed sign), the verifier
    // MUST reject.
    //
    // Implementer wires:
    //
    //   // Adversarial manifest: signature is over bytes including itself.
    //   // Construct: encode the manifest bytes with signature field, sign that:
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //   let mut manifest = ...;
    //   manifest.set_signature(Some(benten_engine::ManifestSignature::placeholder()));
    //   let bytes_with_sig_field = benten_engine::manifest_canonical_bytes_unsafe_with_signature(&manifest);
    //   let bad_sig = publisher_kp.sign(&bytes_with_sig_field);
    //   manifest.set_signature(Some(benten_engine::ManifestSignature::ed25519(bad_sig)));
    //
    //   // verify() expects sign-over-bytes-WITHOUT-signature-field:
    //   let result = benten_engine::verify_manifest(&manifest, &publisher_kp.public_key());
    //   assert!(result.is_err(),
    //       "Adversarial sign-over-self manifest must reject at verify");
    //
    // OBSERVABLE consequence: an attacker's attempt to construct a
    // sign-over-self manifest fails at verify even with the
    // attacker's own keypair. Defends against the recursion-shape
    // attack at the canonical-bytes seam.
    unimplemented!("G14-C wires verify_manifest rejection of sign-over-self adversarial manifest");
}

#[test]
#[ignore = "RED-PHASE: G14-C — crypto-major-1 — canonical bytes stable across signed/unsigned"]
fn manifest_canonical_bytes_stable_across_signed_vs_unsigned() {
    // crypto-major-1 pin. The CID for a manifest must depend ONLY on
    // its semantic content, not on whether it has been signed yet
    // (the signature is a property OF the CID's manifest, computed
    // FROM the canonical bytes).
    //
    // Implementer wires:
    //
    //   let unsigned = ...;
    //   let cid_unsigned = unsigned.cid();
    //
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //   let signed = benten_engine::sign_manifest(&unsigned, &publisher_kp).unwrap();
    //   let cid_signed = signed.cid();
    //
    //   // Per crypto-major-1, signing changes the SIGNATURE field but
    //   // does NOT change the CID. Therefore canonical bytes are
    //   // COMPUTED OVER the manifest excluding the signature field.
    //   assert_eq!(cid_unsigned, cid_signed,
    //       "manifest CID must be stable regardless of signature presence per crypto-major-1");
    //
    // OBSERVABLE consequence: a publisher who signs a manifest
    // doesn't accidentally fork its identity; downstream content-
    // addressing is stable across the sign event.
    unimplemented!("G14-C wires CID-stability assertion across signed vs unsigned manifest");
}

#[test]
#[ignore = "RED-PHASE: G14-C — crypto-minor-5 — UCAN check first then registry"]
fn manifest_signature_check_order_ucan_first_then_registry() {
    // crypto-minor-5 pin (CLR-2 cluster). When a manifest is
    // presented with both a UCAN delegation chain AND a registry-
    // published signature, the verifier checks UCAN FIRST (cheaper /
    // faster failure) THEN registry.
    //
    // Implementer wires:
    //
    //   let manifest = ...; // signed by publisher
    //   let bad_ucan_chain = ...; // structurally valid but for wrong audience
    //   let valid_registry_pubkey = ...;
    //
    //   let err = benten_engine::verify_manifest_dual(
    //       &manifest, &bad_ucan_chain, &valid_registry_pubkey).unwrap_err();
    //   assert!(matches!(err, benten_engine::ManifestVerifyError::UcanInvalid { .. }),
    //       "verifier must surface UCAN failure FIRST per crypto-minor-5");
    //   // The registry check ran or not is observable via verify-trace:
    //   // implementer pins via the typed error name (UcanInvalid),
    //   // not via timing.
    //
    // OBSERVABLE consequence: the typed error variant names which
    // check failed; UCAN is checked first and short-circuits.
    unimplemented!("G14-C wires manifest verify check-order UCAN-then-registry per crypto-minor-5");
}

#[test]
#[ignore = "RED-PHASE: G14-C — crypto-minor-5 — dual presentation both must verify"]
fn manifest_signature_dual_presentation_both_must_verify() {
    // crypto-minor-5 pin. When a manifest is presented dually (both
    // UCAN chain AND registry signature), BOTH must verify or the
    // manifest rejects. This is "AND" semantics, not "OR".
    //
    // Implementer wires:
    //
    //   let manifest = ...;
    //   let valid_ucan_chain = ...;
    //   let valid_registry_pubkey = ...;
    //   let WRONG_registry_pubkey = ...; // attacker tries to substitute
    //
    //   // Both valid → succeeds:
    //   benten_engine::verify_manifest_dual(
    //       &manifest, &valid_ucan_chain, &valid_registry_pubkey).unwrap();
    //
    //   // UCAN valid, registry wrong → fails (must be AND):
    //   let err = benten_engine::verify_manifest_dual(
    //       &manifest, &valid_ucan_chain, &WRONG_registry_pubkey).unwrap_err();
    //   assert!(matches!(err, benten_engine::ManifestVerifyError::RegistryInvalid { .. }));
    //
    //   // Registry valid, UCAN bad → fails (must be AND):
    //   let bad_ucan_chain = ...;
    //   let err = benten_engine::verify_manifest_dual(
    //       &manifest, &bad_ucan_chain, &valid_registry_pubkey).unwrap_err();
    //   assert!(matches!(err, benten_engine::ManifestVerifyError::UcanInvalid { .. }));
    //
    // OBSERVABLE consequence: dual presentation requires BOTH paths
    // to succeed. Defends against the "valid signature but stolen
    // delegation" attack class.
    unimplemented!("G14-C wires AND-semantics dual-presentation verify per crypto-minor-5");
}

#[test]
#[ignore = "RED-PHASE: G14-C — crypto-minor-5 — registry mutation requires UCAN delegation"]
fn publisher_registry_mutation_requires_ucan_delegation() {
    // crypto-minor-5 pin. Mutating the publisher registry (adding /
    // revoking a publisher's pubkey) MUST require a UCAN delegation
    // chain rooted at the registry-admin DID. Defends against the
    // "anyone can publish" attack class.
    //
    // Implementer wires:
    //
    //   let admin_kp = benten_id::keypair::Keypair::generate();
    //   let registry = benten_engine::PublisherRegistry::new(admin_kp.public_key().to_did());
    //
    //   let new_publisher_pk = ...;
    //   let unauthorized_caller_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Without UCAN delegation: registry mutation rejects.
    //   let err = registry.add_publisher_unauthorized(&new_publisher_pk).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::PublisherRegistryError::UcanRequired));
    //
    //   // With UCAN delegation rooted at admin DID: succeeds.
    //   let delegation = benten_id::ucan::Ucan::builder()
    //       .issuer(admin_kp.public_key().to_did())
    //       .audience(unauthorized_caller_kp.public_key().to_did())
    //       .capability("registry:publishers", "add")
    //       .sign(&admin_kp).unwrap();
    //   registry.add_publisher_with_ucan(&new_publisher_pk, &delegation).unwrap();
    //
    // OBSERVABLE consequence: registry-mutation API requires UCAN
    // proof; without delegation, all mutations reject.
    unimplemented!("G14-C wires UCAN-delegation requirement on PublisherRegistry mutation");
}

#[test]
#[ignore = "RED-PHASE: G14-C — cap-r4-6 — verify-mode Any: either path succeeds"]
fn manifest_signature_verify_mode_any_either_path_succeeds() {
    // cap-r4-6 pin (cap-minor-2 closure). Operator may opt into
    // ManifestVerifyMode::Any for non-UCAN deployments per
    // crypto-minor-5 fallback narrative. With Any: either UCAN-only
    // OR registry-only presentation succeeds.
    //
    // Implementer wires:
    //
    //   let policy = benten_engine::ManifestVerifyMode::Any;
    //
    //   // UCAN-only present, registry absent → passes:
    //   let manifest_ucan_only = ...; // signed via UCAN chain only
    //   benten_engine::verify_manifest_with_mode(
    //       &manifest_ucan_only, &valid_ucan_chain, None, policy).unwrap();
    //
    //   // Registry-only present, UCAN absent → passes:
    //   let manifest_registry_only = ...; // signed via registry pubkey only
    //   benten_engine::verify_manifest_with_mode(
    //       &manifest_registry_only, &[], Some(&valid_registry_pubkey), policy).unwrap();
    //
    //   // Neither path present → fails (Any does not mean None):
    //   let err = benten_engine::verify_manifest_with_mode(
    //       &manifest_unsigned, &[], None, policy).unwrap_err();
    //   assert!(matches!(err, benten_engine::ManifestVerifyError::NoPathPresent));
    //
    // OBSERVABLE consequence: operator can configure non-UCAN
    // deployments (registry-only) without forcing UCAN chain
    // construction. Defends operator choice per crypto-minor-5.
    unimplemented!(
        "G14-C wires ManifestVerifyMode::Any either-path-suffices semantics per cap-r4-6"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-C — cap-r4-6 — verify-mode All: both paths required"]
fn manifest_signature_verify_mode_all_both_paths_required() {
    // cap-r4-6 pin (cap-minor-2 closure). Explicit AND opt-in via
    // ManifestVerifyMode::All. With All: BOTH UCAN AND registry
    // paths MUST succeed; either absent → reject.
    //
    // Implementer wires:
    //
    //   let policy = benten_engine::ManifestVerifyMode::All;
    //
    //   // Both present + valid → passes:
    //   benten_engine::verify_manifest_with_mode(
    //       &manifest_dual_signed, &valid_ucan_chain, Some(&valid_registry_pubkey),
    //       policy).unwrap();
    //
    //   // UCAN-only (registry absent) → fails (All requires both):
    //   let err = benten_engine::verify_manifest_with_mode(
    //       &manifest_ucan_only, &valid_ucan_chain, None, policy).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::ManifestVerifyError::RegistryRequiredByModeAll));
    //
    //   // Registry-only (UCAN absent) → fails (All requires both):
    //   let err = benten_engine::verify_manifest_with_mode(
    //       &manifest_registry_only, &[], Some(&valid_registry_pubkey),
    //       policy).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::ManifestVerifyError::UcanRequiredByModeAll));
    //
    // OBSERVABLE consequence: operator with security-critical
    // deployment can require BOTH paths via ::All; defaults to ::Any
    // for backward-compat. Defends crypto-minor-5 + operator policy
    // expressiveness.
    unimplemented!("G14-C wires ManifestVerifyMode::All AND-required semantics per cap-r4-6");
}

#[test]
#[ignore = "RED-PHASE: G14-C — plan §3 G14-C — install_module rejects unsigned/invalid manifest"]
fn install_module_rejects_unsigned_or_invalid_manifest() {
    // plan §3 G14-C pin. The install path MUST reject manifests that
    // are unsigned OR carry an invalid signature. Defends against the
    // "anyone can write to the module store" attack class.
    //
    // Implementer wires:
    //
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let bytes = ...;
    //
    //   // Unsigned manifest: rejects.
    //   let unsigned_manifest = ...;
    //   let err = engine.install_module(&unsigned_manifest, &bytes).unwrap_err();
    //   assert!(matches!(err, benten_engine::EngineError::ManifestUnsigned { .. }));
    //
    //   // Manifest with bogus signature: rejects.
    //   let mut bogus = ...;
    //   bogus.set_signature(Some(benten_engine::ManifestSignature::ed25519([0u8; 64])));
    //   let err = engine.install_module(&bogus, &bytes).unwrap_err();
    //   assert!(matches!(err, benten_engine::EngineError::ManifestInvalidSignature { .. }));
    //
    //   // Properly signed manifest: succeeds.
    //   let publisher_kp = benten_id::keypair::Keypair::generate();
    //   let signed = benten_engine::sign_manifest(&original, &publisher_kp).unwrap();
    //   engine.install_module(&signed, &bytes).unwrap();
    //
    // OBSERVABLE consequence: install_module enforces signature
    // presence + validity at the entry point.
    unimplemented!(
        "G14-C wires install_module rejection of unsigned + invalid-signature manifests"
    );
}
