//! Phase-4-Foundation G24-A — dogfood path (a) production-runtime arm.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 11 (LOAD-BEARING §3.6f substantive) + §5 table row 6; closes
//! ux-r1-1 BLOCKER + D-4F-8 substrate portion at G24-A canary.
//!
//! Per HARD RULE 12 disposition: the full UX arm (click-counter +
//! live-DOM workflow-form drag-drop interaction) lands at G24-B
//! wave-6b when the browser-side workflow editor ships. G24-A pins
//! the **engine-substrate arm** here: admin UI v0 WORKFLOWS-category
//! route subgraph exists; user-created workflow Node is persisted +
//! re-readable via Class B β seam with stable CID. The click-counter
//! UX arm BELONGS-NAMED-NOW at `docs/future/phase-4-backlog.md §2`
//! (Phase-4-Foundation dogfood-gate carries; closed at G24-B wave-6b
//! when the browser-side admin UI workflow editor ships).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
fn dogfood_path_a_create_workflow_ux_acceptance() {
    common::admin_ui_v0_dogfood::dogfood_path_a_workflow_creation_arm();
}
