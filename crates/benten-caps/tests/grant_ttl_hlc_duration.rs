#![allow(unknown_lints, clippy::duration_suboptimal_units)] // MSRV 1.91 — Rust 1.95 lint suggests from_mins/from_hours, both stabilized in 1.95
//! R3 tests for ucca-8 + proptest chain-depth-5 attenuation transitivity.
//!
//! ucca-8: `CapabilityGrant` gains an optional `ttl_hlc_duration` field via
//! `#[serde(skip_serializing_if = "Option::is_none")]`. When `None`, the CID
//! is identical to the Phase-1 shape (additive compatibility). When `Some`,
//! the CID differs.
//!
//! Proptest: chain-depth-5 attenuation transitivity — for A→B→C→D→E,
//! `check_attenuation(A,E)` ⇔ every adjacent pair passes.
//!
//! TDD red-phase: `ttl_hlc_duration` does not yet exist on `CapabilityGrant`.
//! Tests will fail to compile until ucca-8 lands.
//!
//! **Watch 3 (r2-triage):** This file carries the canonical-fixture CID proptest.
//! First run captures the CID and the `todo!()` guard must fail loudly so the
//! R3 writer can paste the pinned value back in.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.4 ucca-8 + chain-depth-5).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapabilityGrant, GrantScope};
use benten_core::Cid;
use std::time::Duration;

fn zero_cid() -> Cid {
    // G9-A fix: the R3-shipped helper used `Cid::from_bytes(&[0u8; CID_LEN])`,
    // which fails construction because byte[0] is the `CID_V1` marker (0x01)
    // not zero. Use the correct `from_blake3_digest([0u8; 32])` path so the
    // fixture produces a structurally-valid CID whose digest is all-zeros.
    // Flagged for R4b review — assertions are unchanged; only the broken
    // precondition is repaired.
    Cid::from_blake3_digest([0u8; 32])
}

/// Canonical CID for a ttl-less grant under the exit-criterion fixture.
/// Captured on G9-A first run per the R2 Watch-3 pin-capture protocol: a
/// grant with `(grantee, issuer) = (zero_cid, zero_cid)`, `scope =
/// "store:post:write"`, `hlc_stamp = 0`, and `ttl_hlc_duration = None`.
/// The `None` TTL path preserves the Phase-1 shape via `as_node`'s
/// `skip_serializing_if = "Option::is_none"` semantics, so this CID also
/// pins the additive-compatibility promise.
const EXPECTED_CID: &str = "bafyr4iexrlgishx4dpzblnfxcclv7ojns7txhp5ilcoxti4p7csqdi6n64";

#[test]
fn grant_ttl_hlc_duration_optional_preserves_cid() {
    let scope = GrantScope::parse("store:post:write").expect("scope");
    let grant = CapabilityGrant::new(zero_cid(), zero_cid(), scope);
    // Phase-2a MUST default to `ttl_hlc_duration = None` so the CID is
    // bit-identical to a Phase-1 grant with the same (grantee, issuer, scope,
    // hlc_stamp) tuple.
    assert!(
        grant.ttl_hlc_duration.is_none(),
        "new() must default ttl_hlc_duration to None"
    );

    let cid = grant.cid().expect("cid");
    let actual = cid.to_string();

    if EXPECTED_CID == "TBD" {
        todo!("capture CID from first run and hardcode into EXPECTED_CID: {actual}");
    }
    assert_eq!(
        actual, EXPECTED_CID,
        "ttl_hlc_duration=None grant CID must stay pinned across runs"
    );
}

#[test]
fn grant_ttl_hlc_duration_present_changes_cid() {
    let scope = GrantScope::parse("store:post:write").expect("scope");
    let mut with_ttl = CapabilityGrant::new(zero_cid(), zero_cid(), scope.clone());
    with_ttl.ttl_hlc_duration = Some(Duration::from_secs(300));

    let without_ttl = CapabilityGrant::new(zero_cid(), zero_cid(), scope);

    let cid_with = with_ttl.cid().expect("with-ttl cid");
    let cid_without = without_ttl.cid().expect("no-ttl cid");

    assert_ne!(
        cid_with, cid_without,
        "Some(ttl) must produce a different CID from None (otherwise the field is ignored)"
    );
}

// ---- Chain-depth-5 attenuation transitivity proptest ----------------------
//
// Security writer may ADD additional proptest blocks at file end. See r2-triage
// Watch 1.
//
// // Security writer adds new chain-depth-5 proptest here — append only

use proptest::prelude::*;

proptest! {
    /// For A→B→C→D→E with every adjacent pair passing segment-wise attenuation,
    /// `check_attenuation(A, E)` must also pass.
    #[test]
    fn chain_depth_5_transitivity_happy_path(
        // Build a 5-link chain where each link is either equal or extends by
        // one concrete segment. Proptest shrinks toward minimal violations.
        root in "[a-z]{1,6}:[a-z]{1,6}:[a-z]{1,6}",
        suffix_b in "[a-z]{1,6}",
        suffix_c in "[a-z]{1,6}",
        suffix_d in "[a-z]{1,6}",
        suffix_e in "[a-z]{1,6}",
    ) {
        // Grow the chain by appending one segment at a time after a trailing
        // wildcard on the parent; each child is strictly attenuated.
        let a = format!("{root}:*");
        let b = format!("{root}:{suffix_b}:*");
        let c = format!("{root}:{suffix_b}:{suffix_c}:*");
        let d = format!("{root}:{suffix_b}:{suffix_c}:{suffix_d}:*");
        let e = format!("{root}:{suffix_b}:{suffix_c}:{suffix_d}:{suffix_e}");

        // Each adjacent pair holds.
        prop_assert!(benten_caps::check_attenuation(&a, &b).is_ok());
        prop_assert!(benten_caps::check_attenuation(&b, &c).is_ok());
        prop_assert!(benten_caps::check_attenuation(&c, &d).is_ok());
        prop_assert!(benten_caps::check_attenuation(&d, &e).is_ok());

        // Therefore so must the transitive root → leaf.
        prop_assert!(
            benten_caps::check_attenuation(&a, &e).is_ok(),
            "A→E must hold whenever every adjacent pair holds"
        );
    }
}
