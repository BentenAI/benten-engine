//! Phase-4-Foundation G24-A — dogfood path (c) production-runtime arm.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 13; closes ux-r1-1 multi-device-sync acceptance substrate
//! portion at G24-A canary.
//!
//! Per HARD RULE 12 disposition: the full 2-peer Atrium sync round-
//! trip with ≤3s loopback latency assertion requires the dogfood-gate
//! harness; that lands at wave-9 dogfood gate. G24-A pins the
//! **content-addressing convergence arm** here: identical Node bytes
//! on two distinct Engine instances yield identical CID — the
//! substrate property that makes multi-device sync convergent. The
//! ≤3s loopback latency arm + Devices-sub-panel last-sync-time UX
//! BELONG-NAMED-NOW at `docs/future/phase-4-backlog.md §2`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
fn dogfood_path_c_multi_device_sync_ux_acceptance() {
    common::admin_ui_v0_dogfood::dogfood_path_c_multi_device_sync_arm();
}
