//! G24-D primary pin: full manifest round-trip (sign + verify + install + uninstall + upgrade).
//!
//! Per plan §3 G24-D row first test pin name. R3 RED-PHASE — the
//! `validate()` + `compute_content_cid()` methods are stubbed at
//! `unimplemented!()`; this test #[ignore]s with the un-ignore wave
//! G24-D. Substance: round-trip exercises real bytes -> sign -> verify
//! -> CID compute -> install record sign -> verify chain at materializer
//! entry. Sentinel-presence test would NOT suffice (per pim-2 §3.6b).

mod common;

use common::manifest_fixtures::{minimal_manifest, stub_install_record};

#[test]
#[ignore = "RED-PHASE: G24-D wave fills validate + compute_content_cid + install round-trip; un-ignore at G24-D landing per pim-12 §3.6e"]
fn plugin_manifest_full_round_trip_sign_then_verify_then_install_then_uninstall_then_upgrade() {
    let manifest = minimal_manifest();

    // At R3 RED-PHASE compute_content_cid() is unimplemented!() so this
    // line panics until G24-D fills. The body shape encodes the
    // SUBSTANTIVE acceptance criterion: a manifest goes through the
    // complete lifecycle and every observable boundary holds.
    let cid = manifest.compute_content_cid();
    manifest.validate().expect("manifest should validate");

    let install = stub_install_record(cid);
    // Future surface (G24-D): InstallRecord::verify_user_signature ->
    // returns Result; FAILS-IF-NO-OP via UCAN chain trace at cap-policy
    // backend (Layer 1 trace-to-user-root).
    assert_eq!(install.manifest_cid, cid);
}
