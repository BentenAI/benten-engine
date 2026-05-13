//! Phase-4-Foundation G24-A — dogfood path (d) production-runtime arm.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 14; closes ux-r1-1 + ratification #7 revoke-mid-session
//! substrate portion at G24-A canary.
//!
//! Per HARD RULE 12 disposition: the full live-subscription revoke
//! arm with user-visible toast assertion lives at wave-9 dogfood gate
//! (requires a live admin UI session + toast-driver). G24-A pins the
//! **per-row gate redaction arm** here: a deny from the materializer-
//! layer per-row gate suppresses content + emits a denial frame +
//! renders `[redacted]` — the substrate property that makes the
//! cap-revoke UX correct. The toast surfacing UX arm BELONGS-NAMED-NOW
//! at `docs/future/phase-4-backlog.md §2`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
fn dogfood_path_d_revoke_cap_mid_session_ux_acceptance() {
    common::admin_ui_v0_dogfood::dogfood_path_d_revoke_cap_mid_session_arm();
}
