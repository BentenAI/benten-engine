//! Phase-4-Foundation G24-A — dogfood path (b) production-runtime arm.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 12 (LOAD-BEARING §3.6f substantive); closes ux-r1-1 + ux-r1-16
//! substrate portion at G24-A canary.
//!
//! Per HARD RULE 12 disposition: the full live-preview-latency-p50-p99
//! arm requires a live browser harness with materializer pipeline
//! wired against a wasm32 bundle — that lands at G24-C wave-6b. G24-A
//! pins the **engine-substrate arm** here: admin UI v0 VIEWS-category
//! route exists; a view-source Node materialises through the
//! `HtmlJsonMaterializer` pipeline; the rendered output reflects
//! engine-sourced bytes. The latency-budget UX arm BELONGS-NAMED-NOW
//! at `docs/future/phase-4-backlog.md §2`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
fn dogfood_path_b_create_composed_view_ux_acceptance() {
    common::admin_ui_v0_dogfood::dogfood_path_b_composed_view_creator_arm();
}
