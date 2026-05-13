//! Phase-4-Foundation G24-A — dogfood path (e) production-runtime arm.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 15; closes ux-r1-1 + ux-r1-2 install-consent substrate portion
//! at G24-A canary.
//!
//! Per HARD RULE 12 disposition: the full ≤3-click install-consent
//! flow + plain-English manifest display + per-cap-decline path live
//! at G24-D wave-7 (full plugin manifest) + wave-9 dogfood gate. G24-A
//! pins the **canonical-bytes reproducibility arm** here: the admin
//! UI v0 subgraph's canonical bytes are stable across builds + the
//! 4 categories survive the cross-device transfer — the substrate
//! property that makes 2nd-device install work. The install-consent
//! UX click-count arm BELONGS-NAMED-NOW at
//! `docs/future/phase-4-backlog.md §2`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
fn dogfood_path_e_install_admin_ui_on_2nd_device_ux_acceptance() {
    common::admin_ui_v0_dogfood::dogfood_path_e_install_admin_ui_on_2nd_device_arm();
}
