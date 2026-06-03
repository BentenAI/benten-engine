//! F-LD-3 — `ExecuteWorkflow` codepoint-reserve slot + AAD-binding frozen
//! (RED-PHASE; byte-pinning; NQ-T3-RATIFIED sufficiency arm).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-3: "variant `{workflow_cid, input_node_cids, max_decrypt_count,
//!     result_recipient_pubkey, executor_did}` + AAD-binds
//!     `(executor_did, max_decrypt_count, result_recipient_pubkey)` frozen;
//!     runtime enforcement post-v1-beta but **AAD SUFFICIENT to express
//!     constraint**. R2-gated on NQ-T3." Red-phase intent: "variant exists;
//!     3 fields in AAD (hex-pin); mutate each → Open fails (constraint bound
//!     not advisory); sufficiency assertion."
//!   - R0.5 plan §3.4 (`...f-full-r0-plan.md:510-517`): the variant fields +
//!     "The variant-slot + the AAD-binding of `(executor_did, max_decrypt_count,
//!     result_recipient_pubkey)` are FROZEN at v1-beta."
//!   - §10.5 NQ-T3 (`...:1375-1377`): "Does the engine enforce that the rented
//!     executor cannot exfiltrate plaintext beyond `result_recipient_pubkey`...
//!     The AAD scope must be SUFFICIENT to express the constraint even though
//!     enforcement is post-v1-beta."
//!
//! ## NQ-T3 RATIFIED (Ben 2026-06-02; spec R0.5 §10.5)
//!
//! NQ-T3 is **RATIFIED**: the frozen 3-field AAD `(executor_did,
//! max_decrypt_count, result_recipient_pubkey)` is SUFFICIENT to express the
//! no-egress / bounded-decrypt constraint; runtime enforcement is post-v1-beta
//! and **non-freeze-gating** (the wire freeze is the AAD scope, not the runtime
//! check). The `..._aad_is_sufficient_to_express_no_egress_constraint` arm
//! below pins exactly that ratified property: the three bound fields are the
//! complete frozen substrate, so post-v1-beta runtime enforcement never needs a
//! wire-format addition. The arm stays `#[ignore]`'d as a RED-PHASE stub (the
//! BLAKE3-keyed authenticator is a stand-in for ChaCha20-Poly1305-under-HPKE);
//! R5 swaps in the real AEAD and un-ignores. The sufficiency assertion is final
//! per the ratified default — no longer gated on an open question.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

use benten_id::keypair::Keypair;

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — ExecuteWorkflow variant + AAD binding.
// ---------------------------------------------------------------------------
//
// The "AEAD" here is a BLAKE3-keyed authenticator over (plaintext, AAD): a
// stand-in for ChaCha20-Poly1305-under-HPKE. The property the family freezes
// is that the three constraint fields are in the AAD, so mutating any of them
// makes Open fail. R5 swaps in the real AEAD; the AAD shape is the freeze.
mod shim {
    /// The frozen ExecuteWorkflow variant (R0.5 §3.4 / e2r). M-20: every integer
    /// (`max_decrypt_count: u32`) is BE on the wire.
    #[derive(Clone)]
    pub struct ExecuteWorkflow {
        pub workflow_cid: [u8; 32],
        pub input_node_cids: Vec<[u8; 32]>,
        pub max_decrypt_count: u32,
        pub result_recipient_pubkey: [u8; 32],
        pub executor_did: Vec<u8>,
    }

    impl ExecuteWorkflow {
        /// The FROZEN AAD: binds exactly the three constraint fields
        /// `(executor_did, max_decrypt_count, result_recipient_pubkey)` —
        /// the "sufficient to express no-egress" scope (NQ-T3, RATIFIED).
        /// BE integers.
        pub fn constraint_aad(&self) -> Vec<u8> {
            let mut aad = Vec::new();
            aad.extend_from_slice(b"benten-exec-workflow-v1:");
            aad.extend_from_slice(&(self.executor_did.len() as u32).to_be_bytes());
            aad.extend_from_slice(&self.executor_did);
            aad.extend_from_slice(&self.max_decrypt_count.to_be_bytes());
            aad.extend_from_slice(&self.result_recipient_pubkey);
            aad
        }
    }

    /// AEAD-seal stand-in: tag = BLAKE3(key ‖ aad ‖ plaintext).
    pub fn seal(key: &[u8; 32], aad: &[u8], plaintext: &[u8]) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(key);
        h.update(&(aad.len() as u32).to_be_bytes());
        h.update(aad);
        h.update(plaintext);
        *h.finalize().as_bytes()
    }

    /// AEAD-open stand-in: recomputes the tag under the PRESENTED aad; if a
    /// constraint field was mutated post-seal, the recomputed tag differs.
    pub fn open(key: &[u8; 32], aad: &[u8], plaintext: &[u8], tag: &[u8; 32]) -> Result<(), ()> {
        if &seal(key, aad, plaintext) == tag {
            Ok(())
        } else {
            Err(())
        }
    }
}

use shim::{open, seal, ExecuteWorkflow};

fn sample(executor: &Keypair) -> ExecuteWorkflow {
    ExecuteWorkflow {
        workflow_cid: [1u8; 32],
        input_node_cids: vec![[2u8; 32], [3u8; 32]],
        max_decrypt_count: 5,
        result_recipient_pubkey: [4u8; 32],
        executor_did: executor.public_key().to_bytes().to_vec(),
    }
}

/// F-LD-3 variant existence + the three constraint fields are present in the
/// frozen AAD (the slot freeze). would-FAIL-if-no-op'd: if `constraint_aad`
/// omitted a field, the corresponding mutation arm below would (wrongly) still
/// Open.
#[test]
#[ignore = "RED-PHASE: F-LD-3 — ExecuteWorkflow variant + 3-field AAD present; un-ignore at R5"]
fn f_ld_3_execute_workflow_variant_binds_three_constraint_fields_in_aad() {
    let executor = Keypair::generate();
    let ew = sample(&executor);
    let aad = ew.constraint_aad();
    // The AAD literally contains the three constraint fields.
    assert!(
        aad.windows(32).any(|w| w == ew.result_recipient_pubkey),
        "result_recipient_pubkey MUST be bound in the AAD"
    );
    assert!(
        aad.windows(4).any(|w| w == 5u32.to_be_bytes()),
        "max_decrypt_count MUST be bound in the AAD (BE)"
    );
    assert!(
        aad.windows(ew.executor_did.len())
            .any(|w| w == ew.executor_did.as_slice()),
        "executor_did MUST be bound in the AAD"
    );
}

/// F-LD-3 mutate-executor_did → Open fails (the field is a bound constraint,
/// not advisory). A rented executor cannot swap itself for another and re-use
/// the sealed material.
#[test]
#[ignore = "RED-PHASE: F-LD-3 — mutating executor_did breaks AEAD Open; un-ignore at R5"]
fn f_ld_3_mutating_executor_did_breaks_open() {
    let executor = Keypair::generate();
    let other = Keypair::generate();
    let key = [42u8; 32];
    let plaintext = b"decrypted-node-payload";

    let ew = sample(&executor);
    let tag = seal(&key, &ew.constraint_aad(), plaintext);

    // Present the SAME sealed material but with a different executor_did.
    let mut mutated = ew.clone();
    mutated.executor_did = other.public_key().to_bytes().to_vec();
    assert!(
        open(&key, &mutated.constraint_aad(), plaintext, &tag).is_err(),
        "swapping executor_did MUST fail AEAD Open (constraint is bound)"
    );
    // Sanity: the un-mutated AAD opens (proves the failure is the mutation).
    open(&key, &ew.constraint_aad(), plaintext, &tag).expect("original AAD MUST open");
}

/// F-LD-3 mutate-max_decrypt_count → Open fails. A rented executor cannot raise
/// its own decrypt budget.
#[test]
#[ignore = "RED-PHASE: F-LD-3 — mutating max_decrypt_count breaks AEAD Open; un-ignore at R5"]
fn f_ld_3_mutating_max_decrypt_count_breaks_open() {
    let executor = Keypair::generate();
    let key = [42u8; 32];
    let plaintext = b"decrypted-node-payload";
    let ew = sample(&executor);
    let tag = seal(&key, &ew.constraint_aad(), plaintext);

    let mut mutated = ew.clone();
    mutated.max_decrypt_count = u32::MAX; // attacker raises the budget
    assert!(
        open(&key, &mutated.constraint_aad(), plaintext, &tag).is_err(),
        "raising max_decrypt_count MUST fail AEAD Open"
    );
}

/// F-LD-3 mutate-result_recipient_pubkey → Open fails. A rented executor
/// cannot redirect the result to an attacker-chosen recipient (the no-egress
/// channel binding).
#[test]
#[ignore = "RED-PHASE: F-LD-3 — mutating result_recipient_pubkey breaks AEAD Open; un-ignore at R5"]
fn f_ld_3_mutating_result_recipient_pubkey_breaks_open() {
    let executor = Keypair::generate();
    let key = [42u8; 32];
    let plaintext = b"decrypted-node-payload";
    let ew = sample(&executor);
    let tag = seal(&key, &ew.constraint_aad(), plaintext);

    let mut mutated = ew.clone();
    mutated.result_recipient_pubkey = [0xEE; 32]; // attacker-chosen recipient
    assert!(
        open(&key, &mutated.constraint_aad(), plaintext, &tag).is_err(),
        "redirecting result_recipient_pubkey MUST fail AEAD Open"
    );
}

/// F-LD-3 NQ-T3 SUFFICIENCY arm (RATIFIED, Ben 2026-06-02).
///
/// Pins that the FROZEN AAD scope is *sufficient to express* the no-egress /
/// bounded-decrypt constraint — i.e. all three constraint fields the runtime
/// would need are present + tamper-bound at v1-beta, so post-v1-beta runtime
/// enforcement has a complete frozen substrate to enforce against (no later
/// wire-format change is required to ADD a constraint field).
///
/// NQ-T3 (§10.5) is RATIFIED: the frozen 3-field AAD is sufficient; runtime
/// enforcement is post-v1-beta and non-freeze-gating. This arm pins the
/// ratified sufficiency property (the wire freeze IS the AAD scope). The
/// enforcement-vs-advisory runtime check is a post-v1-beta concern that this
/// freeze deliberately does not gate; R5 un-ignores against the ratified
/// default.
#[test]
#[ignore = "RED-PHASE: F-LD-3 — NQ-T3-RATIFIED AAD-sufficiency (frozen 3-field AAD sufficient; runtime enforcement post-v1-beta non-freeze-gating); un-ignore at R5"]
fn f_ld_3_frozen_aad_is_sufficient_to_express_no_egress_constraint_nq_t3_ratified() {
    let executor = Keypair::generate();
    let ew = sample(&executor);
    let aad = ew.constraint_aad();

    // Sufficiency = the three fields the no-egress runtime needs are ALL bound
    // now, so enforcement never requires a post-v1-beta wire addition.
    let has_executor = aad
        .windows(ew.executor_did.len())
        .any(|w| w == ew.executor_did.as_slice());
    let has_budget = aad.windows(4).any(|w| w == ew.max_decrypt_count.to_be_bytes());
    let has_channel = aad.windows(32).any(|w| w == ew.result_recipient_pubkey);

    assert!(
        has_executor && has_budget && has_channel,
        "NQ-T3 sufficiency: the no-egress constraint is fully expressible from \
         the frozen AAD ((executor_did, max_decrypt_count, result_recipient_pubkey)) \
         — no later wire change needed to enforce"
    );
}
