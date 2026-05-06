//! G14-A2 wave-4a' — MultiSigSurface trait test pins (un-ignored).
//!
//! Pin sources (per `crypto-minor-2` + `cag-5` + D-PHASE-3-24):
//!
//! - `multi_sig_surface_trait_signature_pinned`
//! - `multi_sig_surface_ed25519_single_key_default_impl_round_trip`
//! - `multi_sig_surface_threshold_extension_point_present`
//! - `multi_sig_surface_no_recovery_protocol_specific_behavior_in_phase_3`

#![allow(clippy::unwrap_used)]

use benten_id::MultiSigError;
use benten_id::keypair::Keypair;
use benten_id::multi_sig::{Ed25519SingleKey, MultiSigSurface, ThresholdMultiSig};

#[test]
fn multi_sig_surface_trait_signature_pinned() {
    // crypto-minor-2 architectural pin. Compile-time verification —
    // any drift on the trait signature would fail to compile.
    #[allow(clippy::type_complexity)]
    const _: fn() = || {
        fn assert_signature<S: MultiSigSurface>() {
            let _: fn(&S, &[u8]) -> Result<S::Signature, S::Error> = S::sign;
            let _: fn(&S, &[u8], &S::Signature) -> Result<(), S::Error> = S::verify;
            let _: fn(&S) -> u32 = S::threshold;
            let _: fn(&S) -> u32 = S::participants;
        }
        assert_signature::<Ed25519SingleKey>();
        assert_signature::<ThresholdMultiSig>();
    };
}

#[test]
fn multi_sig_surface_ed25519_single_key_default_impl_round_trip() {
    // crypto-minor-2 unit pin. Sign + verify round-trip; tampered
    // message rejects.
    let kp = Keypair::generate();
    let surface = Ed25519SingleKey::new(kp);
    let msg = b"multi-sig round trip";
    let sig = surface.sign(msg).unwrap();
    surface.verify(msg, &sig).unwrap();
    assert_eq!(surface.threshold(), 1);
    assert_eq!(surface.participants(), 1);

    // Tampered message rejects:
    let bad = b"tampered round trip";
    assert!(matches!(
        surface.verify(bad, &sig).unwrap_err(),
        MultiSigError::BadSignature
    ));
}

#[test]
fn multi_sig_surface_threshold_extension_point_present() {
    // crypto-minor-2 architectural pin. The trait extension point
    // exists — `ThresholdMultiSig` is a non-sealed trait impl. Body
    // returns PostPhase3 per D-PHASE-3-24.
    let surface = ThresholdMultiSig {
        threshold: 2,
        participants: 3,
    };
    assert_eq!(surface.threshold(), 2);
    assert_eq!(surface.participants(), 3);

    let err = surface.sign(b"any message").unwrap_err();
    assert!(matches!(err, MultiSigError::PostPhase3));

    let err = surface.verify(b"any message", &Vec::new()).unwrap_err();
    assert!(matches!(err, MultiSigError::PostPhase3));
}

#[test]
fn multi_sig_surface_no_recovery_protocol_specific_behavior_in_phase_3() {
    // cag-5 + D-PHASE-3-24 architectural pin. Source-grep audit: the
    // NON-COMMENT surface area of multi_sig.rs MUST NOT name
    // recovery-protocol-specific terms. Per `crypto-r4-r1-minor-2`,
    // strip comment-only lines first to avoid false-positives from
    // legitimate doc-comments mentioning deferred protocols.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_path = std::path::PathBuf::from(manifest_dir).join("src/multi_sig.rs");
    let src = std::fs::read_to_string(&src_path).unwrap();
    let non_comment: String = src
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//") && !trimmed.starts_with("#![") && !trimmed.starts_with("#[")
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Avoid regex dep; do simple substring + word-boundary checks
    // against an explicit list. Each forbidden term: confirm absence.
    const FORBIDDEN_SUBSTRINGS: &[&str] = &[
        "Shamir",
        "shamir",
        "SHAMIR",
        "SocialRecovery",
        "social_recovery",
        "hardware_escrow",
        "HardwareEscrow",
    ];
    for needle in FORBIDDEN_SUBSTRINGS {
        assert!(
            !non_comment.contains(needle),
            "multi_sig.rs (NON-COMMENT) MUST NOT name {needle} per D-PHASE-3-24"
        );
    }
    // Word-boundary assertions for short tokens (TPM / MLS) to avoid
    // false positives like `tempfile`. Check for whitespace-bounded
    // occurrences.
    for needle in &["TPM", "MLS"] {
        let bounded_patterns = [
            format!(" {needle} "),
            format!(" {needle},"),
            format!(" {needle}."),
            format!(" {needle};"),
            format!(":{needle}:"),
            format!("({needle})"),
        ];
        for p in &bounded_patterns {
            assert!(
                !non_comment.contains(p),
                "multi_sig.rs (NON-COMMENT) MUST NOT name {needle} per D-PHASE-3-24"
            );
        }
    }
}
