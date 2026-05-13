//! G24-D-FP-2 pin — manifest-envelope chain-validator seam admits
//! a chain whose every delegation step fits the source manifest's
//! shares policy.
//!
//! Future surface: `crates/benten-caps/src/manifest_envelope_chain_
//! validation.rs::validate_chain_with_manifest_envelope(chain,
//! manifest_store) -> Result`.
//!
//! Per plan §3.5.1 G24-D-FP-2 acceptance tests + T8 defense.

#[test]
#[ignore = "RED-PHASE: G24-D-FP-2 wires manifest_envelope_chain_validation; un-ignore at G24-D-FP-2 landing"]
fn ucan_chain_within_manifest_envelope_admitted_at_chain_validator() {
    // Future surface:
    //   manifest_envelope_chain_validation::
    //     validate_chain_with_manifest_envelope(chain, manifest_store)
    //   Walks each delegation step in chain; for each step where
    //   issuer is a plugin-DID, consults manifest_store for the
    //   issuing plugin's manifest; checks delegation cap fits
    //   manifest.shares policy. Admits chain if all steps fit.
    //
    // FAILS-IF-NO-OP because no-op admits everything (T8 attack).
    panic!("RED-PHASE: G24-D-FP-2 wave must wire manifest_envelope_chain_validation::validate");
}
