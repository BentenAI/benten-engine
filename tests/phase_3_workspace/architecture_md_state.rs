//! Phase-3 ARCHITECTURE.md narrative state pins — RETIRED at G20-B
//! wave-8b.
//!
//! ## RETIRED (G20-B wave-8b 2026-05-07)
//!
//! Both intermediate-state pins that previously lived here
//! (`architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only`
//! at G14-A1 and `architecture_md_in_flight_callout_present_for_benten_sync_native_only_at_g16_a`
//! at G16-A) were DELETED at G20-B wave-8b per the explicit obsolescence
//! note authored at R3-A/C: "At G20-B wave-8b, the FINAL 10-crate
//! transition narrative replaces both intermediate-state HTML comments.
//! The G20-B docs sweep should either DELETE these two test fns OR
//! retense them to assert the transitional narrative was REPLACED."
//!
//! `docs/ARCHITECTURE.md` no longer carries the in-flight callouts
//! (Phase 1 of G20-B retired them when landing the FINAL 10-crate
//! transition narrative at checkpoint `7219e4b`). The surviving
//! authority is `tests/phase_3_workspace/architecture_md_g20b_final.rs`
//! (`architecture_md_lists_10_crates_with_benten_id_and_benten_sync`).
//!
//! This file is kept as a load-bearing audit-trail comment (HARD RULE
//! clause-b transparency: the deletion was deliberate, not accidental).
//! No `#[test]` declarations remain.
//!
//! ## Original ownership history (preserved for audit-trail)
//!
//! - **R3-A** (file's first dispatch): authored
//!   `architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only`
//!   asserting the G14-A1 intermediate-state callout (9-crate in-flight
//!   narrative; 10-crate not-yet) was present.
//!
//! - **R3-C** (subsequent dispatch): extended with the G16-A
//!   intermediate-state assertion (10-crate in-flight callout naming
//!   `benten-sync` as native-only).
//!
//! - **R3-E** (G20-B closure dispatch): authored
//!   `architecture_md_lists_10_crates_with_benten_id_and_benten_sync`
//!   asserting the FINAL 10-crate transition narrative is landed
//!   (sibling file).
//!
//! - **G20-B wave-8b**: retired the two intermediate-state pins per
//!   their explicit obsolescence note.
//!
//! ## Pin sources (historical)
//!
//! - r2-test-landscape §2.2 G14-A1 row + §2.4 G16-A row (intermediate
//!   states).
//! - C-15 (architecture-md-doc-drift cluster).
//! - arch-r1-3 BLOCKER (in-flight callouts present at every wave that
//!   adds a workspace crate). The BLOCKER's claim is now defended by
//!   the G20-B FINAL pin (the durable narrative replaces the in-flight
//!   callouts at the same time the architecture transitions complete).
