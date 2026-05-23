//! TF-3f pins — G-CORE-3f Drop bundle envelope-sig + per-Node-sig
//! defense-in-depth.
//!
//! ADDL Phase-4-Meta-Core, R3-W3 partition.
//!
//! **SPEC-GAP SURFACED:** eventual destination is `benten-drop` (see
//! sibling file `tf3f_drop_bundle_offline_consume.rs` header). Lands in
//! `benten-sync/tests/` as a temporary home; relocate at G-CORE-3f.
//!
//! Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` §2 G-CORE-3f (P-3):
//!     "Envelope-sig + per-Node-sig defense-in-depth (<12% overhead
//!     asserted; Spike G finding)."
//!   - §2 G-CORE-3f (F-1): "Tampered DropBundle (one byte flip in
//!     `content[0]` ciphertext) → AEAD authentication fails on that
//!     Node; envelope-sig still valid (defense-in-depth — per-Node-sig
//!     is the secondary integrity)."
//!   - §2 G-CORE-3f (F-2): "Tampered envelope-sig → top-level
//!     verification fails before per-Node decryption attempted."
//!   - `00-implementation-plan.md` §3 G-CORE-3 def 8 spike-derived
//!     refinements item (8): "Drop bundle = full S&C composition in
//!     CBOR-on-disk (2688-byte bundle for 5 Recipe Nodes; envelope-sig
//!     + per-Node-sig defense-in-depth costs <12%)."
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-3f (pim-12 / §3.6e).
//! ============================================================================

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::unnested_or_patterns)]

use benten_drop::{DropBundle, DropBundleError};
use benten_id::keypair::Keypair;

// ---------------------------------------------------------------------------
// PIN 1 — F-1: per-Node ciphertext tamper detected; envelope-sig still
// valid (defense-in-depth).
// ---------------------------------------------------------------------------
// Adversary flips one byte in `content[0]` ciphertext. The envelope-sig
// over the bundle header is STILL valid (the adversary didn't touch the
// header). Top-level verification SUCCEEDS. But per-Node-AEAD on
// `content[0]` FAILS at decrypt — the AAD-binds-plaintext-CID layer
// catches the tamper.
//
// This is the load-bearing defense-in-depth property: envelope-sig
// alone is insufficient; per-Node integrity is the secondary layer.
#[test]

fn tf3f_per_node_ciphertext_tamper_detected_envelope_sig_still_valid() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&kp_alice, &kp_bob);

    // Adversary tampers one byte in content[0] ciphertext.
    let tampered = bundle.flip_byte_in_content_for_test(0, /* offset */ 7);

    // Envelope-sig verification: still passes (only header was signed).
    let env_verify = tampered.verify_envelope_signature();
    assert!(
        env_verify.is_ok(),
        "Envelope-sig SHOULD still verify after content-only tamper \
         (the envelope-sig binds header, not content). got: {:?}",
        env_verify.as_ref().map(|_| "Ok").unwrap_or("Err")
    );

    // Per-Node consume: tampered content[0] FAILS.
    let consume_result = tampered.consume_offline(&kp_bob);
    assert!(
        matches!(
            consume_result,
            Err(DropBundleError::PerNodeAeadAuthenticationFailed { .. })
                | Err(DropBundleError::PerNodeSignatureInvalid { .. })
        ),
        "Tampered content[0] MUST fail at the per-Node integrity layer \
         (AEAD or per-Node-sig), even though envelope-sig is still \
         valid. This IS the defense-in-depth property. got: {:?}",
        consume_result.as_ref().map(|_| "Ok(_)").unwrap_or("Err(_)")
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — F-2: Tampered envelope-sig fails top-level before per-Node attempted.
// ---------------------------------------------------------------------------
// Adversary tampers the envelope-sig itself. The handler MUST refuse
// at envelope-sig verification, BEFORE attempting any per-Node
// decryption (no wasted work; no information leak about content
// validity).
#[test]

fn tf3f_tampered_envelope_sig_fails_before_per_node_decrypt() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();
    let bundle = DropBundle::build_5_recipe_bundle_for_recipient(&kp_alice, &kp_bob);

    let tampered = bundle.tamper_envelope_signature_for_test();

    // Counter that increments only when per-Node decrypt is reached.
    let consume_result = tampered.consume_offline_with_dec_counter_for_test(&kp_bob);
    match consume_result {
        Err((DropBundleError::EnvelopeSignatureInvalid { .. }, decrypt_attempt_count)) => {
            assert_eq!(
                decrypt_attempt_count, 0,
                "Tampered envelope-sig MUST fail BEFORE any per-Node \
                 decrypt is attempted. Reached {} decrypt attempts — \
                 means the handler tried decrypt despite a broken \
                 envelope-sig (no-fail-fast bug).",
                decrypt_attempt_count
            );
        }
        other => panic!(
            "Expected EnvelopeSignatureInvalid + 0 decrypt attempts; \
             got: {:?}",
            other.as_ref().map(|_| "Ok(_)").err()
        ),
    }
}

// ---------------------------------------------------------------------------
// PIN 3 — P-3: defense-in-depth overhead <12% (Spike G finding).
// ---------------------------------------------------------------------------
// The per-Node-sig adds <12% overhead vs envelope-sig-only. A stub
// that omits per-Node-sigs would pass the consume test (envelope-sig
// alone is sufficient for happy-path) but breaks defense-in-depth.
// This pin asserts the per-Node-sig surface is present + sized.
#[test]

fn tf3f_defense_in_depth_overhead_under_12_percent() {
    let kp_alice = Keypair::generate();
    let kp_bob = Keypair::generate();

    let bundle_with_per_node = DropBundle::build_5_recipe_bundle_for_recipient(&kp_alice, &kp_bob);
    let bundle_envelope_only =
        DropBundle::build_5_recipe_bundle_for_recipient_envelope_only_for_test(&kp_alice, &kp_bob);

    let size_with = bundle_with_per_node.to_cbor_bytes().unwrap().len();
    let size_without = bundle_envelope_only.to_cbor_bytes().unwrap().len();

    // overhead_ratio = (with - without) / without
    let overhead_bp = ((size_with.saturating_sub(size_without)) * 10_000) / size_without;
    assert!(
        overhead_bp < 1200,
        "Per-Node-sig defense-in-depth overhead MUST be <12% (Spike G). \
         Measured: with={} bytes; without={} bytes; overhead={} bp \
         (basis points; 1200 = 12%).",
        size_with,
        size_without,
        overhead_bp
    );

    // Anti-stub: per-Node-sig MUST be present (size_with > size_without).
    assert!(
        size_with > size_without,
        "Per-Node-sig variant MUST be larger than envelope-only — proves \
         the per-Node-sigs are actually emitted, not stubbed away."
    );
}
