//! R3-A RED-PHASE pins for UCAN chain validation (G14-A1 wave-4a).
//!
//! Pin sources (per r2-test-landscape §2.2 G14-A1 + §11 CLR-2 + plan §3
//! G14-A1 must-pass column):
//!
//! - `tests/ucan_chain_validation_basic` — plan §3 G14-A1
//! - `tests/ucan_chain_attenuation_rejects_overgrant` — plan §3 G14-A1
//! - `tests/ucan_chain_revocation_propagates` — plan §3 G14-A1
//! - `tests/ucan_nbf_time_window_pre_activation_rejects` — `crypto-blocker-2` BLOCKER + CLR-2
//! - `tests/ucan_exp_time_window_post_expiration_rejects` — `crypto-blocker-2` + CLR-2
//! - `tests/ucan_chain_walk_propagates_nbf_exp_through_attenuation` — `crypto-blocker-2` + CLR-2
//! - `tests/ucan_chain_walk_constant_time_comparison_audit` — `crypto-major-4`
//! - `tests/ucan_audience_binding_prevents_cross_atrium_replay` — CLR-2 + `cap-major-1`
//! - `ucan_chain_nbf_enforcement` — §11 CLR-2 (composes with `ucan_nbf_time_window_pre_activation_rejects`)
//! - `ucan_chain_exp_enforcement` — §11 CLR-2 (composes with `ucan_exp_time_window_post_expiration_rejects`)
//!
//! ## CLR-2 cross-lens cluster
//!
//! Per r2-test-landscape §3.A (UCAN replay / time-window cluster), the
//! load-bearing assertions span chain-walk site nbf/exp + audience binding
//! + constant-time comparison. This test file owns the G14-A1 keypair-side
//! pins; G14-B (durable backend) pins live in
//! `crates/benten-caps/tests/ucan_backend.rs` (R3-B); G14-D (WAIT-resume
//! envelope) pins live in `crates/benten-engine/tests/wait_resume_cross_process.rs`
//! (R3-B).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A1 — plan §3 G14-A1 — basic chain validation"]
fn ucan_chain_validation_basic() {
    // G14-A1 implementer wires this:
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let audience = benten_id::keypair::Keypair::generate();
    //   let ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(issuer.public_key().to_did())
    //       .audience(audience.public_key().to_did())
    //       .capability("/zone/posts", "read")
    //       .nbf_now()
    //       .exp_in_secs(3600)
    //       .sign(&issuer)
    //       .unwrap();
    //   // A single-link chain validates: signature + nbf + exp + iss DID match.
    //   assert!(benten_id::ucan::validate_chain(&[ucan.clone()]).is_ok());
    //
    // OBSERVABLE consequence: a well-formed single-token chain
    // validates; tampering invalidates it (wired in subsequent tests).
    unimplemented!("G14-A1 wires basic UCAN chain validation");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — plan §3 G14-A1 — attenuation rejects overgrant"]
fn ucan_chain_attenuation_rejects_overgrant() {
    // The attenuation contract: a delegated UCAN MUST NOT widen the
    // authority of its parent. If parent grants `read /zone/posts`,
    // child cannot grant `write /zone/posts` or `read /zone/admin`.
    //
    // Implementer wires:
    //   let root = benten_id::keypair::Keypair::generate();
    //   let delegate = benten_id::keypair::Keypair::generate();
    //   let parent = ... .capability("/zone/posts", "read") ... ;
    //   let overgrant_child = ... .capability("/zone/posts", "write") ... ;
    //   let chain = vec![overgrant_child, parent];
    //   let err = benten_id::ucan::validate_chain(&chain).unwrap_err();
    //   assert!(matches!(err, benten_id::ucan::ChainError::AttenuationViolated { .. }));
    //
    // OBSERVABLE consequence: the chain rejects with a typed error
    // naming the attenuation violation.
    unimplemented!("G14-A1 wires attenuation overgrant rejection");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — plan §3 G14-A1 — revocation propagates"]
fn ucan_chain_revocation_propagates() {
    // Revoking a UCAN at any link in the chain MUST invalidate every
    // descendant token. Implementer wires:
    //   let root = ...;
    //   let middle = ...; // delegated from root
    //   let leaf = ...;   // delegated from middle
    //   let revocation_set = benten_id::ucan::RevocationSet::new();
    //   revocation_set.revoke(&middle.cid());
    //   let err = benten_id::ucan::validate_chain_with_revocations(
    //       &[leaf, middle, root], &revocation_set).unwrap_err();
    //   assert!(matches!(err, benten_id::ucan::ChainError::Revoked { .. }));
    //
    // OBSERVABLE consequence: revoking the middle link breaks the leaf
    // even though the leaf itself wasn't directly revoked.
    unimplemented!("G14-A1 wires revocation propagation through chain");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-blocker-2 BLOCKER + CLR-2 — nbf rejection"]
fn ucan_nbf_time_window_pre_activation_rejects() {
    // BLOCKER pin per crypto-blocker-2. The `nbf` (not-before) field
    // MUST be enforced at chain-walk site. A token presented BEFORE
    // its nbf is not yet valid.
    //
    // Implementer wires:
    //   let now = benten_id::ucan::SystemTime::epoch_secs();
    //   let ucan = ... .nbf(now + 3600) ... ;  // valid 1 hour from now
    //   let err = benten_id::ucan::validate_chain_at(&[ucan], now).unwrap_err();
    //   assert!(matches!(err, benten_id::ucan::ChainError::NotYetValid { .. }));
    //
    // OBSERVABLE consequence: token presented during its pre-activation
    // window rejects with a typed NotYetValid error.
    unimplemented!("G14-A1 wires nbf pre-activation rejection at chain-walk site");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-blocker-2 BLOCKER + CLR-2 — exp rejection"]
fn ucan_exp_time_window_post_expiration_rejects() {
    // BLOCKER pin per crypto-blocker-2. The `exp` (expiration) field
    // MUST be enforced at chain-walk site. A token presented AFTER
    // its exp is no longer valid.
    //
    // Implementer wires:
    //   let issuance = 1_000_000_000;
    //   let ucan = ... .nbf(issuance).exp(issuance + 60) ... ;
    //   let err = benten_id::ucan::validate_chain_at(&[ucan], issuance + 120).unwrap_err();
    //   assert!(matches!(err, benten_id::ucan::ChainError::Expired { .. }));
    //
    // OBSERVABLE consequence: post-expiration token rejects with
    // typed Expired error.
    unimplemented!("G14-A1 wires exp post-expiration rejection at chain-walk site");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-blocker-2 + CLR-2 — nbf/exp propagate"]
fn ucan_chain_walk_propagates_nbf_exp_through_attenuation() {
    // crypto-blocker-2 + CLR-2 pin. The chain-walk validator MUST
    // verify nbf/exp on EVERY token in the chain, not just the leaf.
    // A child token whose parent has expired MUST reject even if the
    // child's own exp is in the future.
    //
    // Implementer wires:
    //   let now = 1_000_000_000;
    //   let parent = ... .nbf(now).exp(now + 60) ... ;       // expires at now+60
    //   let child  = ... .nbf(now).exp(now + 86400) ... ;    // child claims long validity
    //   let err = benten_id::ucan::validate_chain_at(&[child, parent], now + 120).unwrap_err();
    //   // Even though child's own exp is way in the future, the parent
    //   // expired at now+60; checking at now+120 must reject because the
    //   // delegation chain itself is broken.
    //   assert!(matches!(err, benten_id::ucan::ChainError::Expired { .. }));
    //
    // OBSERVABLE consequence: chain-walk catches expired-parent
    // attempts even when the leaf token's own dates would suggest
    // it's still valid. Defense against the "renew the leaf forever"
    // delegation attack.
    unimplemented!("G14-A1 wires nbf/exp propagation through chain attenuation");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — crypto-major-4 — constant-time comparison"]
fn ucan_chain_walk_constant_time_comparison_audit() {
    // crypto-major-4 pin. UCAN signature comparison + audience-DID
    // comparison MUST use a constant-time comparison primitive (e.g.
    // `subtle::ConstantTimeEq`) to defend against timing-side-channel
    // leak of "how many leading bytes match."
    //
    // This test is a SOURCE-CITE assertion: greps the implementation
    // for `==` on signature/DID bytes and asserts there are zero such
    // sites; ALL byte-comparison goes through `subtle::ConstantTimeEq`
    // or `subtle::ConstantTimeOption`.
    //
    // Implementer wires:
    //   // Read the source of crates/benten-id/src/ucan.rs (or the
    //   // chain-walk site specifically) and assert no naive byte-eq:
    //   let src = std::fs::read_to_string("crates/benten-id/src/ucan.rs").unwrap();
    //   // Heuristic: no `signature ==` / `audience ==` / `proof_cid ==`:
    //   for line in src.lines() {
    //       let l = line.trim_start();
    //       if l.starts_with("//") { continue; }
    //       assert!(!(l.contains("signature ==") || l.contains("audience ==") || l.contains("proof_cid ==")),
    //           "constant-time comparison required per crypto-major-4: {}", line);
    //   }
    //   // Additionally, assert `subtle` is in the dep tree:
    //   let cargo = std::fs::read_to_string("crates/benten-id/Cargo.toml").unwrap();
    //   assert!(cargo.contains("subtle"),
    //       "benten-id MUST depend on `subtle` for constant-time comparison");
    //
    // OBSERVABLE consequence: source-grep at audit time finds NO
    // naive byte-equality on cryptographic material; `subtle` IS in
    // the dependency manifest.
    unimplemented!("G14-A1 wires source-grep constant-time-comparison audit");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — CLR-2 + cap-major-1 — audience binding"]
fn ucan_audience_binding_prevents_cross_atrium_replay() {
    // CLR-2 / cap-major-1 pin. The `aud` (audience) field on a UCAN
    // names the DID it's intended for; presenting a UCAN to a
    // DIFFERENT audience (e.g. replaying it against a different
    // atrium peer) MUST reject.
    //
    // Implementer wires:
    //   let atrium_a = benten_id::keypair::Keypair::generate();
    //   let atrium_b = benten_id::keypair::Keypair::generate();
    //   let issuer   = benten_id::keypair::Keypair::generate();
    //   let ucan_for_a = ... .audience(atrium_a.public_key().to_did()) ... ;
    //   // Replay the token at atrium B:
    //   let err = benten_id::ucan::validate_chain_for_audience(
    //       &[ucan_for_a],
    //       &atrium_b.public_key().to_did(),
    //   ).unwrap_err();
    //   assert!(matches!(err, benten_id::ucan::ChainError::AudienceMismatch { .. }));
    //
    // OBSERVABLE consequence: cross-atrium replay rejects with a
    // typed AudienceMismatch error. Defense against the "token leaked
    // from atrium A, replayed at atrium B" attack class.
    unimplemented!("G14-A1 wires audience-binding cross-atrium replay rejection");
}

// ---------------------------------------------------------------------------
// §11 CLR-2 redundant-distinct shape pins (composing with the chain-walk
// pins above; these names appear in r2-test-landscape §11 as separate
// component pins, distinct from the chain-walk shapes).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "RED-PHASE: G14-A1 — §11 CLR-2 — nbf enforcement at chain-walk"]
fn ucan_chain_nbf_enforcement() {
    // §11 redundant-distinct shape: composes with
    // `ucan_nbf_time_window_pre_activation_rejects` but at a different
    // entry point — directly via `Ucan::validate` rather than
    // `validate_chain_at`. Both entry points MUST converge on the
    // same nbf rejection.
    //
    // Implementer wires:
    //   let now = 1_000_000_000;
    //   let ucan = ... .nbf(now + 60) ... ;
    //   assert!(ucan.validate_at(now).is_err());   // single-token entry
    //   assert!(ucan.validate_at(now + 120).is_ok()); // post-nbf
    //
    // OBSERVABLE: single-token validate_at honors nbf identically to
    // chain-walk validate_chain_at.
    unimplemented!("G14-A1 wires single-token nbf enforcement at chain-walk entry");
}

#[test]
#[ignore = "RED-PHASE: G14-A1 — §11 CLR-2 — exp enforcement at chain-walk"]
fn ucan_chain_exp_enforcement() {
    // §11 redundant-distinct shape (exp counterpart to
    // ucan_chain_nbf_enforcement).
    //
    // Implementer wires:
    //   let now = 1_000_000_000;
    //   let ucan = ... .nbf(now).exp(now + 60) ... ;
    //   assert!(ucan.validate_at(now + 30).is_ok());   // mid-window
    //   assert!(ucan.validate_at(now + 120).is_err()); // post-exp
    //
    // OBSERVABLE: single-token validate_at honors exp identically to
    // chain-walk validate_chain_at.
    unimplemented!("G14-A1 wires single-token exp enforcement at chain-walk entry");
}
