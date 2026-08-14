//! Closure-pin tests for benten-id DoS-via-unbounded-decode umbrella
//! #1172 (META #629 sub-cluster):
//!
//! - **#549** — `Ucan::prf` recursive deserialization has no depth
//!   bound. Pinned by [`from_canonical_bytes_bounded_rejects_overdeep_prf_chain`]
//!   + [`from_canonical_bytes_bounded_accepts_shallow_chain`] +
//!   [`max_cbor_depth_precheck_runs_before_serde_recursion`].
//! - **#555** — `UcanError` variants carry attacker-supplied bytes
//!   into operator-facing `Display`. Pinned by
//!   [`ucan_error_display_sanitizes_control_bytes_and_truncates`].
//!
//! These exercise the REAL arms (not presence/sentinel checks) and
//! would FAIL if the depth pre-check or the Display sanitizer were
//! reverted.

#![allow(clippy::unwrap_used)]

use benten_id::UcanError;
use benten_id::VcError;
use benten_id::keypair::Keypair;
use benten_id::ucan::{MAX_UCAN_ENVELOPE_BYTES, MAX_UCAN_PROOF_DEPTH, Ucan};
use benten_id::vc::{MAX_VC_ENVELOPE_BYTES, TrustDomain, verify_bytes_in_trust_domain};

/// Build a signed leaf UCAN, then wrap it in `depth` nested `prf`
/// layers (each parent carries the prior token as its single proof).
/// Returns the canonical DAG-CBOR bytes of the outermost token.
fn nested_ucan_bytes(depth: usize) -> Vec<u8> {
    let kp = Keypair::generate();
    let aud = Keypair::generate();
    let mut token = Ucan::builder()
        .issuer(kp.public_key().to_did().as_str())
        .audience(aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .sign(&kp);
    for _ in 0..depth {
        token = Ucan::builder()
            .issuer(kp.public_key().to_did().as_str())
            .audience(aud.public_key().to_did().as_str())
            .capability("/zone/posts", "read")
            .proof(token)
            .sign(&kp);
    }
    serde_ipld_dagcbor::to_vec(&token).unwrap()
}

#[test]
fn from_canonical_bytes_bounded_accepts_shallow_chain() {
    // A realistic 3-deep delegation chain must decode cleanly under
    // the default ceiling — the bound must not break legitimate use.
    let bytes = nested_ucan_bytes(3);
    let decoded = Ucan::from_canonical_bytes_bounded(&bytes, MAX_UCAN_PROOF_DEPTH);
    assert!(
        decoded.is_ok(),
        "shallow 3-deep chain must decode under MAX_UCAN_PROOF_DEPTH, got {decoded:?}"
    );
}

#[test]
fn from_canonical_bytes_bounded_rejects_overdeep_prf_chain() {
    // Encode a chain far deeper than the ceiling. The bounded entry
    // point MUST reject with ProofChainTooDeep BEFORE serde's
    // recursive deserialize runs. With a tiny explicit max_depth we
    // keep the test fast while still exercising the real reject arm.
    let small_max = 4;
    let bytes = nested_ucan_bytes(small_max + 6);
    let result = Ucan::from_canonical_bytes_bounded(&bytes, small_max);
    match result {
        Err(UcanError::ProofChainTooDeep { depth, max }) => {
            assert_eq!(max, small_max, "reported max must equal configured ceiling");
            assert!(
                depth > max,
                "reported depth ({depth}) must exceed the ceiling ({max})"
            );
        }
        other => panic!("expected ProofChainTooDeep, got {other:?}"),
    }
}

#[test]
fn max_cbor_depth_precheck_runs_before_serde_recursion() {
    // A pathologically deep chain (well past the default ceiling)
    // must be rejected by the iterative byte-boundary pre-check and
    // NOT abort the process via stack overflow. If the pre-check were
    // removed, serde's recursive Deserialize for `Ucan` would run on
    // this blob; at sufficient depth that overflows the thread stack
    // (process abort, not a catchable error). Reaching this assertion
    // at all proves the non-recursive pre-check fired first.
    let bytes = nested_ucan_bytes(MAX_UCAN_PROOF_DEPTH + 50);
    let result = Ucan::from_canonical_bytes_bounded(&bytes, MAX_UCAN_PROOF_DEPTH);
    assert!(
        matches!(result, Err(UcanError::ProofChainTooDeep { .. })),
        "over-deep blob must be rejected at the byte boundary, got {result:?}"
    );
}

#[test]
fn from_canonical_bytes_bounded_decodes_to_usable_token() {
    // The happy path must yield a real, structurally-intact token
    // (not just "Ok") — proves the pre-check is a guard, not a
    // replacement for the decode.
    let bytes = nested_ucan_bytes(2);
    let token = Ucan::from_canonical_bytes_bounded(&bytes, MAX_UCAN_PROOF_DEPTH).unwrap();
    // 2 wrap layers => the outermost token has exactly one proof,
    // whose own proof is the leaf.
    assert_eq!(token.claims.prf.len(), 1, "outer token carries one proof");
    assert_eq!(
        token.claims.prf[0].claims.prf.len(),
        1,
        "middle token carries the leaf as its proof"
    );
    assert!(
        token.claims.prf[0].claims.prf[0].claims.prf.is_empty(),
        "leaf token has no proof"
    );
}

#[test]
fn ucan_error_display_sanitizes_control_bytes_and_truncates() {
    // #555: a UcanError variant whose String field is attacker-
    // influenced must NOT render raw control bytes (log-injection)
    // and must bound its length (log-flooding). Construct the variant
    // directly with an adversarial audience string: embedded newline
    // + NUL + a very long tail.
    let adversarial = format!("evil\n\u{0}aud{}", "A".repeat(5_000));
    let err = UcanError::AudienceMismatch {
        token_aud: adversarial.clone(),
        expected: "did:key:zClean".to_string(),
    };
    let rendered = err.to_string();

    assert!(
        !rendered.contains('\n'),
        "rendered error must not contain a raw newline (log-injection): {rendered:?}"
    );
    assert!(
        !rendered.contains('\u{0}'),
        "rendered error must not contain a raw NUL byte: {rendered:?}"
    );
    assert!(
        rendered.contains("\\x0a") || rendered.contains("\\x0A"),
        "newline must be rendered as an escaped byte: {rendered:?}"
    );
    assert!(
        rendered.len() < 400,
        "rendered error must be bounded well under the 5KB adversarial input, got {} chars",
        rendered.len()
    );
    assert!(
        rendered.contains("bytes total>"),
        "rendered error must carry a truncation marker with the original byte length: {rendered:?}"
    );
    // The clean expected side must still render verbatim (we only
    // sanitize the attacker-influenced surface, we don't mangle
    // legitimate DIDs).
    assert!(
        rendered.contains("did:key:zClean"),
        "clean expected-audience must render unmangled: {rendered:?}"
    );
}

#[test]
fn ucan_error_display_passes_clean_dids_through_unmodified() {
    // Regression guard: legitimate did:key strings must NOT acquire a
    // truncation marker or escapes — proves the sanitizer is a filter
    // on adversarial input, not a blanket rewrite.
    let clean = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
    let err = UcanError::ChainLinkBroken {
        link_index: 1,
        aud: clean.to_string(),
        next_iss: clean.to_string(),
    };
    let rendered = err.to_string();
    assert!(
        !rendered.contains("bytes total>"),
        "clean DID must not trigger the truncation marker: {rendered:?}"
    );
    assert!(
        !rendered.contains("\\x"),
        "clean DID must not be escaped: {rendered:?}"
    );
    assert!(
        rendered.matches(clean).count() == 2,
        "both clean DID occurrences must render verbatim: {rendered:?}"
    );
}

// --- Compromise #28 / META #629 DoS-sweep: total-byte envelope caps ---

#[test]
fn ucan_from_canonical_bytes_bounded_rejects_oversized_envelope_before_decode() {
    // A within-depth but over-large blob must be rejected FAST with the
    // typed EnvelopeTooLarge BEFORE serde allocates the decoded Ucan.
    // would-FAIL-on-revert: without the byte cap this O(N) blob decodes
    // (or attempts to) — an allocation DoS on the ucan_validate_chain op
    // + durable UCAN backend read path. The over-cap slice is bytes that
    // never reach serde, so this is fast and allocates nothing large.
    let oversized = vec![0u8; MAX_UCAN_ENVELOPE_BYTES + 1];
    let result = Ucan::from_canonical_bytes_bounded(&oversized, MAX_UCAN_PROOF_DEPTH);
    match result {
        Err(UcanError::EnvelopeTooLarge { got, max }) => {
            assert_eq!(got, MAX_UCAN_ENVELOPE_BYTES + 1, "reports observed length");
            assert_eq!(max, MAX_UCAN_ENVELOPE_BYTES, "reports configured ceiling");
        }
        other => panic!("expected EnvelopeTooLarge, got {other:?}"),
    }
    // At-cap is NOT rejected by the size gate (it falls through to the
    // depth pre-walk / decode, which rejects it as malformed CBOR — the
    // size gate is a ceiling, not an exact-length check).
    let at_cap = vec![0u8; MAX_UCAN_ENVELOPE_BYTES];
    let at_cap_result = Ucan::from_canonical_bytes_bounded(&at_cap, MAX_UCAN_PROOF_DEPTH);
    assert!(
        !matches!(at_cap_result, Err(UcanError::EnvelopeTooLarge { .. })),
        "at-cap input must not trip the EnvelopeTooLarge gate: {at_cap_result:?}"
    );
}

#[test]
fn vc_verify_bytes_rejects_oversized_envelope_before_decode() {
    // A VC envelope over the byte ceiling must be rejected FAST with the
    // typed EnvelopeTooLarge BEFORE serde allocates the Credential.
    // would-FAIL-on-revert: without the byte cap this O(N) blob drives
    // an allocation on the VC-verify path.
    let trust_domain = TrustDomain::empty();
    let oversized = vec![0u8; MAX_VC_ENVELOPE_BYTES + 1];
    // `now` is irrelevant here by construction: the byte cap must fire BEFORE
    // any decode, so no `expirationDate` exists to compare against yet. If a
    // future refactor moved the cap after the decode, this call would start
    // depending on the clock — which is itself a signal worth keeping visible.
    let result = verify_bytes_in_trust_domain(&oversized, &trust_domain, 1_000_000_000);
    match result {
        Err(VcError::EnvelopeTooLarge { got, max }) => {
            assert_eq!(got, MAX_VC_ENVELOPE_BYTES + 1, "reports observed length");
            assert_eq!(max, MAX_VC_ENVELOPE_BYTES, "reports configured ceiling");
        }
        other => panic!("expected EnvelopeTooLarge, got {other:?}"),
    }
}
