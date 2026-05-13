//! Phase-4-Foundation G24-A — dogfood path (f) production-runtime arm.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 16; closes ux-r1-1 install-2nd-plugin substrate portion at
//! G24-A canary.
//!
//! Per HARD RULE 12 disposition: the full ≤3-click flow for a 2nd
//! plugin install + user-DID signing of the install record live at
//! G24-D wave-7 (full plugin manifest) + wave-9 dogfood gate. G24-A
//! pins the **route-subgraph-shape generality arm** here: the same
//! `build_category_route_subgraph` shape applies across plugins —
//! admin UI v0 isn't a special case at the substrate. The
//! user-DID-signing UX arm BELONGS-NAMED-NOW at
//! `docs/future/phase-4-backlog.md §2`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
fn dogfood_path_f_install_2nd_plugin_ux_acceptance() {
    common::admin_ui_v0_dogfood::dogfood_path_f_install_2nd_plugin_arm();
}
