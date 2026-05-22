//! Phase-4-Meta-Core — ADDL R3 (TDD red-phase) — R3-W4 / G-CORE-8 — §4.58
//! SYNC-ATTACK-FABRIC ENUMERATION (15 of 18 vectors per R2 §5).
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! Per R2-test-landscape.md §2 G-CORE-8 (A-2):
//!   "§4.58 sync-attack fabric (the 15-of-18 missing vectors): each
//!    vector either lands as a substantive adversarial pin OR carries a
//!    HARD-RULE-12 disposition (3 valid shapes) in R3 brief. R3 author
//!    enumerates the 18 attack vectors; lands ≥15 substantive pins."
//!
//! Per R2 §5 adversarial patterns: this file is the consolidated
//! enumeration of the sync-attack-fabric class — every vector either
//! has a substantive pin OR a HARD-RULE-12 disposition documented
//! inline (NOT a generic "defer" — each disposition cites OUT-OF-SCOPE
//! or BELONGS-NAMED-NOW with a specific destination + reason, per
//! HARD RULE 12 clauses (a)/(b)/(c)).
//!
//! ## 18 attack vectors — full enumeration (R3 author)
//!
//! Numbering follows R2 §5 + R2 §2 G-CORE-8 (A-2) + the SECURITY-POSTURE
//! sync-attack-class threat-model. Each vector V-N is one of:
//!   - SUBSTANTIVE: a #[test] #[ignore]'d pin below
//!   - DISP-A: OUT-OF-SCOPE — vector reserved for a downstream phase
//!     (Phase-4-Meta-Composing / Phase-7 untrusted-host / Phase-8
//!     decentralized plugin discovery); reason cited inline
//!   - DISP-B: BELONGS-NAMED-NOW — covered by a sibling test file at
//!     a specific named destination; reference inline (no phantom-dest)
//!
//!     V-1  cross-DID-leak-via-merge (Inv-11 strengthening)     SUBSTANTIVE
//!     V-2  unresolvable-peer-DID-at-merge-recheck              SUBSTANTIVE
//!     V-3  unresolvable-peer-DID-at-sync-hydrate (§4.25)       SUBSTANTIVE
//!     V-4  outside-envelope-plugin-write-via-merge              SUBSTANTIVE
//!     V-5  manifest-envelope-bypass-via-spoofed-peer-DID       SUBSTANTIVE
//!     V-6  noop-rechecker-default-admits-everything (§4.36)    SUBSTANTIVE
//!     V-7  cap-recheck-skipped-row (G16-B-F)                    DISP-B (BELONGS-NAMED-NOW — G16-B-F structural-always-on per-row cap-recheck regression-guard test; already shipped at PR #161 Phase-3 R6-FP, named in plan-doc R0.8.1 as the regression-stays guard surface)
//!     V-8  toctou-on-applied-records-set                        DISP-B (BELONGS-NAMED-NOW — `g_core_8_install_record_replay_atomic_record_and_check.rs` in this R3-W4 partition; the atomic record-and-check is the substantive defense)
//!     V-9  envelope-ceiling-exceeded-via-inbound-row (§4.36 envelope_ceiling_admits_row)  SUBSTANTIVE
//!     V-10 cross-plugin-private-namespace-via-merge (§4.28)    DISP-B (BELONGS-NAMED-NOW — `crates/benten-caps/tests/g_core_8_private_namespace_cross_plugin_substantive_arm_4_28.rs` in this R3-W4 partition)
//!     V-11 schema-author-trust-list-bypass-via-merge (§4.32)   DISP-B (BELONGS-NAMED-NOW — `crates/benten-platform-foundation/tests/tf7_g_core_7_install_lifecycle_hardening.rs` §4.32 arm; G-CORE-7 merged at #1312)
//!     V-12 unsigned-or-tampered-install-record-via-merge       SUBSTANTIVE
//!     V-13 device-attestation-forged-at-plugin-share (Compromise #21) DISP-B (BELONGS-NAMED-NOW — Compromise #21 closure test named in CLAUDE.md baked-in #18 + Phase-3 device-DID-attestation merged at PR #163; META #684 closure surface)
//!     V-14 audience-mismatch-via-AuthorizationGrant-and-CapPolicy (§4-D) DISP-B (BELONGS-NAMED-NOW — R3-W5's `cross_wave_3_x_8_authorizationgrant_audience_matches_capabilitypolicy.rs` cross-wave §4-D file)
//!     V-15 chain-validator-skipped-via-merge-path (§4.23)      DISP-B (BELONGS-NAMED-NOW — R3-W4's `g_core_8_write_boundary_user_root_chain_validator_4_23.rs` in this partition; §4.23 structural-always-on)
//!     V-16 willow-protocol-confidential-sync-replay              DISP-A (OOS — willow parked Phase-5+ Kith watch-list per CLAUDE.md 2026-05-21)
//!     V-17 garden-grove-untrusted-host-byzantine-merge          DISP-A (OOS — Phase-7+ peers-hold-ciphertext; this Core wave is single-engine sync defense)
//!     V-18 chunk-shuffle-cross-chunk-rebinding (G-CORE-3d AAD) DISP-B (BELONGS-NAMED-NOW — R3-W3 §2 file `crates/benten-graph/tests/tf3d_per_chunk_aead_iroh_block_size.rs` PIN 5 at L204-238 (the actual chunk-shuffle rebinding-attack pin); this file's `sync-attack-fabric` scope is the merge/recheck path, NOT the G-CORE-3d per-chunk AEAD path)
//!
//! Count: 8 SUBSTANTIVE pins (V-1/2/3/4/5/6/9/12) + 10 DISP (V-7/8/10/11/13/14/15/16/17/18).
//! Refined from R4.1 L3 M-2 finding: 7 prior anchors (V-7/8/10/11/13/14/15) demoted
//! from SUBSTANTIVE to DISP-B (BELONGS-NAMED-NOW) because they unconditionally panic
//! citing sibling-file defenses; un-ignoring at G-CORE-8 wave-completion still fails.
//! Each demoted vector names its destination file/arm/un-ignore-when per V-18 idiom.
//! Net: matches R2 §2 (A-2) "≥15 substantive pins across the sync-attack class" —
//! the 7 demoted destinations contribute their own substantive arms to the class.
//!
//! ## SHAPE-not-SUBSTANCE guard (pim-18 / §3.6f)
//!
//! Each pin FIRST exercises a SHIPPED adjacent surface (the
//! `ManifestEnvelopeRecheckOutcome` enum + the `outcome_to_row_reject`
//! mapping + the SHIPPED `ErrorCode` catalog) with a REAL assertion +
//! observable would-FAIL, THEN `panic!`-holds the still-undelivered
//! defense (the production-built `ProductionManifestEnvelopeRechecker`
//! wired into the default builder + the post-rename `UnresolvedDeny`
//! variant + the structural-always-on chain-validator). Hybrid mostly-
//! undelivered pattern per R4.1 pattern-induction.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL)
//!
//!   - pim-1 (§3.5b HARDENED): G-CORE-8 ships public-shape changes
//!     (enum rename + default-builder rewire + new ErrorCode mints) —
//!     adjacent docs sweep at landing (SECURITY-POSTURE sync-attack
//!     section / INTERNALS.md threat-model / ERROR-CATALOG.md).
//!   - pim-2 + pim-2-amendment (§3.6b sub-rule-4): EACH vector pin
//!     exercises a SPECIFIC arm (production call-site, observable
//!     consequence, would-FAIL-if-no-op'd). No umbrella "sync works"
//!     pin.
//!   - pim-12 (§3.6e): RED-PHASE staged-pins; G-CORE-8 un-ignores; the
//!     reviewer verifies LANDING-STATUS not just spec-pin presence.
//!   - pim-18 (§3.6f): every pin enumerates the production call-site +
//!     body-of-test exercises substantive shipped-surface behavior +
//!     panics on the missing structural defense.
//!   - §3.5g cross-language rule-mirror: any new ErrorCode minted by
//!     G-CORE-8 to close these vectors mirrors Rust↔TS atomically.
//!   - §3.13 per-test-static decomposition: each pin's fixtures are
//!     test-local (no shared `static MOCK_*`).
//!   - §3.5n orchestrator ground-truth-verify: HEAD c9c11c56 verified:
//!     `ManifestEnvelopeRecheckOutcome::NotApplicable` still exists (no
//!     rename to `UnresolvedDeny`); `Engine::default` installs
//!     `Arc<NoopManifestEnvelopeRechecker>` (per `engine.rs:1840` in
//!     existing g_core_8 file doc); the typed `OutsideEnvelope` arm
//!     IS reject-shaped at HEAD (verify-stays); the catalog of
//!     V-1..V-15 maps each pin to its concrete RED state.
//!
//! Pins: G-CORE-8 · §4.58 sync-attack-fabric · §1.A.FROZEN item 12.
//! R2 map: TF-8 A-2 (15 of 18 vectors enumerated).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code)]

use benten_engine::manifest_envelope_recheck::{
    ManifestEnvelopeRecheckOutcome, ManifestEnvelopeRechecker, NoopManifestEnvelopeRechecker,
    outcome_to_row_reject,
};
use benten_errors::ErrorCode;

const UNRESOLVED_PEER_DID: &str = "<unresolved-peer>";
const ATTACKER_PEER_DID: &str = "did:key:zAttackerPeerDid";
const PLUGIN_DID_ALICE: &str = "did:key:zPluginAlice";
const PLUGIN_DID_BOB: &str = "did:key:zPluginBob";

// ===========================================================================
// V-1 — cross-DID-leak-via-merge (Inv-11 strengthening + §4-A composition)
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-1 cross-DID-leak-via- \
            merge — a row written under DID-X arriving via apply_atrium_ \
            merge must NOT become visible in DID-Y's partition view)"]
fn v1_cross_did_leak_via_merge_partition_isolation_enforced() {
    // SHIPPED-SURFACE EXERCISE: the SHIPPED Noop returns NotApplicable
    // for every input — exercise it directly to anchor the substrate.
    let noop = NoopManifestEnvelopeRechecker;
    let outcome = noop.recheck_row(ATTACKER_PEER_DID, "store:notes:DID-X", "row-cross-leak");
    assert_eq!(
        outcome,
        ManifestEnvelopeRecheckOutcome::NotApplicable,
        "shipped substrate: Noop returns NotApplicable on cross-DID \
         attacker row (the admit-everything substrate at HEAD)"
    );
    let admit = outcome_to_row_reject(outcome, "DID-X-zone", "row-cross-leak");
    assert!(
        admit.is_ok(),
        "shipped substrate: Noop's NotApplicable composes to Ok(()) — \
         admit-everything default at HEAD; would-FAIL post-G-CORE-1+8 \
         partition-isolation composition"
    );

    panic!(
        "V-1 cross-DID-leak-via-merge undelivered: at HEAD c9c11c56 the \
         default Noop admits a cross-DID row through apply_atrium_merge \
         (substrate exercised above). G-CORE-8 wires the production \
         rechecker + G-CORE-1's `WriteContext::namespace_did` partition \
         seam composes (per R2 §4-A) — a row written under DID-X MUST \
         NOT appear in DID-Y's view post-merge. Couples G-CORE-1 + \
         G-CORE-8."
    );
}

// ===========================================================================
// V-2 — unresolvable-peer-DID-at-merge-recheck → UnresolvedDeny (§4.36)
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-2 unresolvable-peer-DID \
            at merge-recheck → UnresolvedDeny; NEVER Admitted / silent Ok)"]
fn v2_unresolvable_peer_did_at_recheck_returns_unresolved_deny_never_admit() {
    // SHIPPED-SURFACE EXERCISE: at HEAD the sentinel peer-DID path
    // routes to NotApplicable → Ok (admit). Exercise to anchor.
    let noop = NoopManifestEnvelopeRechecker;
    let outcome = noop.recheck_row(UNRESOLVED_PEER_DID, "merge-zone", "row-unresolved");
    assert_eq!(
        outcome,
        ManifestEnvelopeRecheckOutcome::NotApplicable,
        "shipped substrate: sentinel <unresolved-peer> currently maps \
         to NotApplicable (the security-r1-2 BLOCKER's admit-everything \
         substrate)"
    );

    panic!(
        "V-2 undelivered: at HEAD an unresolvable peer-DID at the \
         apply_atrium_merge recheck path admits via the Noop's \
         NotApplicable. G-CORE-8 lands the `UnresolvedDeny` rename + \
         `outcome_to_row_reject` reject-on-unresolved (security-r1-2)."
    );
}

// ===========================================================================
// V-3 — unresolvable-peer-DID-at-sync-hydrate (§4.25 fail-closed)
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-3 unresolvable-peer-DID \
            at §4.25 sync-hydrate → fail-closed; never proceed)"]
fn v3_unresolvable_peer_did_at_sync_hydrate_fail_closed() {
    // SHIPPED-SURFACE EXERCISE: the recheck enum's OutsideEnvelope arm
    // IS reject-shaped at HEAD (verify-stays). Exercise to anchor the
    // post-G-CORE-8 contract: the unresolved arm joins this reject set.
    let outside = ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
        offending_plugin_did: PLUGIN_DID_ALICE.to_string(),
        cap_pattern: "store:notes:write".to_string(),
    };
    let res = outcome_to_row_reject(outside, "sync-hydrate-zone", "row-hydrate");
    assert!(
        res.is_err(),
        "shipped substrate: OutsideEnvelope is reject-shaped at HEAD \
         (verify-stays); the unresolved/sentinel arm MUST join this \
         reject set post-G-CORE-8"
    );

    panic!(
        "V-3 undelivered: the §4.25 sync-hydrate path does not consult \
         a fail-closed unresolved-peer check at HEAD. G-CORE-8 wires \
         the same UnresolvedDeny → reject semantics into the §4.25 \
         hydrate path (parallel to the §4.36 merge path)."
    );
}

// ===========================================================================
// V-4 — outside-envelope-plugin-write-via-merge (verify-stays @ HEAD,
// becomes ALWAYS-ON at G-CORE-8 default-builder rewire)
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-4 outside-envelope \
            plugin write-via-merge — production rechecker rejects; \
            default-builder wiring is what flips this from opt-in to \
            structurally-always-on)"]
fn v4_outside_envelope_plugin_write_via_merge_rejected_by_default_builder() {
    // SHIPPED-SURFACE EXERCISE: the OutsideEnvelope reject path WORKS
    // at HEAD if a custom rechecker is set; the default-builder still
    // ships the Noop. Exercise the reject path on a synthesized outcome
    // to anchor the post-rewire contract.
    let outside = ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
        offending_plugin_did: PLUGIN_DID_ALICE.to_string(),
        cap_pattern: "store:secrets:write".to_string(),
    };
    let res = outcome_to_row_reject(outside, "merge", "row-out");
    assert!(
        res.is_err(),
        "shipped substrate: OutsideEnvelope reject path works (the \
         opt-in surface IS correct); G-CORE-8's job is to STRUCTURALLY \
         WIRE the production rechecker into Engine::default so this \
         path fires without explicit opt-in"
    );

    panic!(
        "V-4 undelivered: at HEAD the OutsideEnvelope reject is opt-in \
         (Engine::default installs Noop). G-CORE-8 default-builder \
         rewire makes this STRUCTURALLY ALWAYS-ON (security-r1-1)."
    );
}

// ===========================================================================
// V-5 — manifest-envelope-bypass-via-spoofed-peer-DID
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-5 spoofed-peer-DID at \
            merge — the recheck path must verify the peer-DID against \
            the device-attestation envelope, NOT trust the in-band claim)"]
fn v5_spoofed_peer_did_at_merge_recheck_rejected_by_device_attestation_anchor() {
    // SHIPPED-SURFACE EXERCISE: an attacker-controlled peer-DID can be
    // presented as the rechecker input today (no anchor verification).
    // Exercise to anchor: the Noop ignores the peer-DID identity and
    // admits unconditionally.
    let noop = NoopManifestEnvelopeRechecker;
    let attacker = noop.recheck_row(ATTACKER_PEER_DID, "merge", "row-spoof");
    let honest = noop.recheck_row(PLUGIN_DID_ALICE, "merge", "row-spoof");
    assert_eq!(
        attacker, honest,
        "shipped substrate: the Noop is peer-DID-blind (substrate for \
         the would-FAIL signal — post-G-CORE-8, a spoofed peer-DID \
         that doesn't trace to a verified device-attestation envelope \
         MUST yield a different outcome from an honest one)"
    );

    panic!(
        "V-5 undelivered: at HEAD the merge recheck path trusts the \
         in-band peer-DID claim. G-CORE-8 anchors the peer-DID against \
         the device-attestation envelope (Phase-3 G16-D substrate; \
         couples Compromise #21)."
    );
}

// ===========================================================================
// V-6 — noop-rechecker-default-admits-everything (§4.36 BLOCKER closure)
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-6 Engine::default \
            installs the production rechecker, NOT the Noop footgun — \
            security-r1-1 BLOCKER closure)"]
fn v6_default_builder_installs_production_rechecker_not_noop_footgun() {
    // SHIPPED-SURFACE EXERCISE: the Noop's admit-everything property.
    let noop = NoopManifestEnvelopeRechecker;
    let outcome = noop.recheck_row("did:key:zAnyPeer", "any-zone", "any-row");
    assert_eq!(
        outcome,
        ManifestEnvelopeRecheckOutcome::NotApplicable,
        "shipped substrate: Noop admits any input (input-agnostic \
         admit-everything) — the security-r1-1 BLOCKER substrate"
    );

    panic!(
        "V-6 undelivered: Engine::default still installs the \
         NoopManifestEnvelopeRechecker (admit-everything default). \
         G-CORE-8 swaps in a substantive ProductionManifestEnvelopeRechecker."
    );
}

// ===========================================================================
// V-7 — cap-recheck-skipped-row regression-guard — DISP-B (BELONGS-NAMED-NOW)
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination: the G16-B-F structural-always-on per-row cap-recheck
// regression-guard test (already shipped at PR #161 Phase-3 R6-FP per
// CLAUDE.md history; named in plan-doc R0.8.1 as the regression-stays
// guard surface). The G16-B-F per-row cap-recheck contract is verified
// continuously at HEAD by the Phase-3 R6-FP regression test; G-CORE-8
// work proceeds within the discipline of NOT regressing that contract
// (the CI matrix already enforces it). This anchor would unconditionally
// panic on a sibling defense even after G-CORE-8 completes; demoted to
// DISP-B per R4.1 L3 M-2 / orchestrator triage 2026-05-22.
//
// Un-ignore-when: N/A — the destination defense is already shipped; the
// regression-guard fires structurally on every CI run.
//
// No `#[test]` body — DISP-B vectors enumerate inline only.

// ===========================================================================
// V-8 — toctou-on-applied-records-set — DISP-B (BELONGS-NAMED-NOW)
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination: `g_core_8_install_record_replay_atomic_record_and_check.rs`
// in this same R3-W4 partition (couples §4.37). The atomic record-and-check
// is the substantive defense; this sync-attack-fabric anchor was a
// cross-wave-touchpoint annotation that unconditionally panicked on the
// sibling-file seam — demoted to DISP-B per R4.1 L3 M-2 / orchestrator
// triage 2026-05-22.
//
// Un-ignore-when: G-CORE-8 wave-completion (the destination file is
// un-ignored at the same wave; this DISP-B anchor stays inline-only).
//
// No `#[test]` body — DISP-B vectors enumerate inline only.

// ===========================================================================
// V-9 — envelope-ceiling-exceeded-via-inbound-row (§4.36 envelope_ceiling
// admits_row helper; couples #669 unified envelope-ceiling)
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-9 envelope-ceiling \
            exceeded by inbound merge row — the §4.36 envelope_ceiling_ \
            admits_row helper must fire structurally always-on)"]
fn v9_envelope_ceiling_admits_row_helper_structurally_always_on_via_merge() {
    // SHIPPED-SURFACE EXERCISE: the SHIPPED OutsideEnvelope variant +
    // outcome_to_row_reject mapping ARE the row-side substrate for the
    // envelope-ceiling defense. The full envelope_ceiling_admits_row
    // helper is cfg-gated on `not(feature = "browser-backend")` per
    // CLAUDE.md baked-in #17 (full-peer-only) so we don't import it
    // unconditionally here; the substrate exercise is at the recheck-
    // outcome layer.
    let outside = ManifestEnvelopeRecheckOutcome::OutsideEnvelope {
        offending_plugin_did: PLUGIN_DID_ALICE.to_string(),
        cap_pattern: "host:sandbox:run".to_string(),
    };
    let res = outcome_to_row_reject(outside, "merge-zone", "row-ceiling");
    assert!(
        res.is_err(),
        "shipped substrate: the recheck-outcome row-reject primitive \
         the #669 unified envelope-ceiling composes with works at HEAD \
         (substrate; the full envelope_ceiling_admits_row helper is \
         full-peer-only per baked-in #17 — not unconditionally importable)"
    );

    panic!(
        "V-9 undelivered: the envelope-ceiling helper is shipped (PR \
         #1276) but its STRUCTURALLY-ALWAYS-ON wiring into the \
         apply_atrium_merge per-row loop is the G-CORE-8 closure. An \
         inbound row exceeding the verified device envelope MUST \
         row-reject via this helper without explicit opt-in."
    );
}

// ===========================================================================
// V-10 — cross-plugin-private-namespace-via-merge (§4.28) — DISP-B
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination:
// `crates/benten-caps/tests/g_core_8_private_namespace_cross_plugin_substantive_arm_4_28.rs`
// in this same R3-W4 partition. The benten-caps substantive arm is where
// the engine-side cross-plugin private-namespace refusal is pinned (the
// authority half of confidentiality per multitenant-r1-5 / §4.28). This
// anchor was a cross-wave-touchpoint that unconditionally panicked on
// the sibling-file engine-side refusal — demoted to DISP-B per R4.1 L3
// M-2 / orchestrator triage 2026-05-22.
//
// Un-ignore-when: G-CORE-8 wave-completion (the destination file is
// un-ignored at the same wave).
//
// No `#[test]` body — DISP-B vectors enumerate inline only.

// ===========================================================================
// V-11 — schema-author-trust-list-bypass-via-merge (§4.32) — DISP-B
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination:
// `crates/benten-platform-foundation/tests/tf7_g_core_7_install_lifecycle_hardening.rs`
// §4.32 arm. G-CORE-7 (install-lifecycle hardening including §4.32
// schema-author-within-envelope) is already merged at PR #1312; this
// anchor was a cross-wave-touchpoint annotation that unconditionally
// panicked on the §4.32 envelope-binding arm — demoted to DISP-B per
// R4.1 L3 M-2 / orchestrator triage 2026-05-22.
//
// Un-ignore-when: N/A — the destination defense is already shipped at
// PR #1312 (G-CORE-7 merged on main); §4.32 arm fires structurally.
//
// No `#[test]` body — DISP-B vectors enumerate inline only.

// ===========================================================================
// V-12 — unsigned-or-tampered-install-record-via-merge
// ===========================================================================
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (V-12 unsigned/tampered \
            InstallRecord via merge — the user-DID signature on the \
            InstallRecord MUST be verified at merge admission, not \
            just at install-time)"]
fn v12_unsigned_or_tampered_install_record_via_merge_rejected() {
    // SHIPPED-SURFACE EXERCISE: the typed signature-invalid error
    // already exists (`PluginInstallRecordUserSignatureInvalid`).
    let err = ErrorCode::PluginInstallRecordUserSignatureInvalid;
    let s = err.as_str();
    assert!(
        s.starts_with("E_"),
        "shipped substrate: the typed signature-invalid ErrorCode is \
         live (substrate the merge-admission verifier composes with)"
    );

    panic!(
        "V-12 undelivered: at HEAD InstallRecord signatures are verified \
         at install-time but not re-verified on a merge-admission path. \
         G-CORE-8 wires the same `verify_install_record` call at the \
         merge boundary so a tampered record cannot rejoin via sync."
    );
}

// ===========================================================================
// V-13 — device-attestation-forged-at-plugin-share (Compromise #21) — DISP-B
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination: the Compromise #21 closure test named in CLAUDE.md
// baked-in #18 + the signed `DeviceAttestationEnvelope` V2 cryptographic
// closure landed at PR #163 (Phase-3 G16-D wave-6b); META #684 is the
// open closure-surface tracker for the napi-boundary falsely-CLOSED
// finding. This anchor was a cross-wave-touchpoint annotation that
// unconditionally panicked on the closure-surface — demoted to DISP-B
// per R4.1 L3 M-2 / orchestrator triage 2026-05-22.
//
// Un-ignore-when: META #684 napi-boundary closure ships in G-CORE-8 or
// the surrounding wave-window; the substrate (Ed25519 sig + Acceptor
// + payload-hash binding + session-nonce replay defense) is already
// LIVE at PR #163.
//
// No `#[test]` body — DISP-B vectors enumerate inline only.

// ===========================================================================
// V-14 — audience-mismatch-via-AuthorizationGrant-and-CapPolicy (§4-D) — DISP-B
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination: R3-W5's
// `cross_wave_3_x_8_authorizationgrant_audience_matches_capabilitypolicy.rs`
// (the cross-wave §4-D file in the R3-W5 partition). The §4-D
// composition (AuthorizationGrant audience must match CapabilityPolicy
// check_write audience) is owned by the cross-wave file; this anchor
// was a cross-wave-touchpoint annotation that unconditionally panicked
// on the cross-wave composition — demoted to DISP-B per R4.1 L3 M-2 /
// orchestrator triage 2026-05-22.
//
// Un-ignore-when: G-CORE-3b (AuthorizationGrant validator mints) +
// G-CORE-8 §8-E audience-aware hook both land; the cross-wave file is
// un-ignored at the later of those two waves.
//
// No `#[test]` body — DISP-B vectors enumerate inline only.

// ===========================================================================
// V-15 — chain-validator-skipped-via-merge-path (§4.23) — DISP-B
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination: R3-W4's
// `g_core_8_write_boundary_user_root_chain_validator_4_23.rs` (in this
// same R3-W4 partition). The §4.23 structural-always-on user-root
// chain-validator is owned by the sibling file (panics on the WRITE-
// admission seam); this anchor was a cross-wave-touchpoint annotation
// that unconditionally panicked on the merge-path arm — demoted to
// DISP-B per R4.1 L3 M-2 / orchestrator triage 2026-05-22.
//
// Un-ignore-when: G-CORE-8 wave-completion (the destination file is
// un-ignored at the same wave; the merge-path arm composes when the
// validator is wired into apply_atrium_merge's per-row loop).
//
// No `#[test]` body — DISP-B vectors enumerate inline only.

// ===========================================================================
// V-16 — willow-protocol-confidential-sync-replay — DISP-A (OUT-OF-SCOPE)
// ===========================================================================
//
// **Disposition: OUT-OF-SCOPE per HARD RULE 12 clause (a).**
//
// Reason: Willow Protocol adoption is parked Phase-5+ Kith watch-list
// per CLAUDE.md 2026-05-21 ("defer iroh-willow adoption to Phase-5+
// Kith watch-list" — iroh-willow has 0 substantive 2026 commits, pinned
// to incompatible iroh 0.34, 5/14 specs implemented, Confidential Sync
// spec demoted Final→Proposal 2025-10-26). The Confidential Sync replay
// class is a Phase-5+ concern; the Phase-4-Meta-Core wave defends
// against the SINGLE-ENGINE merge-time attack surface, not the
// confidential-sync-protocol-replay class.
//
// No `#[test]` body — DISP-A vectors enumerate inline only.

// ===========================================================================
// V-17 — garden-grove-untrusted-host-byzantine-merge — DISP-A (OUT-OF-SCOPE)
// ===========================================================================
//
// **Disposition: OUT-OF-SCOPE per HARD RULE 12 clause (a).**
//
// Reason: Phase-7+ Garden-Grove untrusted-host / peers-hold-ciphertext
// is the named scope per CLAUDE.md baked-in #18 confidentiality-half
// refinement. The Phase-4-Meta-Core wave is single-engine sync defense
// — Byzantine peer behavior under an untrusted host is a Phase-7
// concern coupled to the #1301 encryption-as-confidentiality work
// (which is itself Phase-4-Meta-Core's downstream G-CORE-3d).
//
// No `#[test]` body — DISP-A vectors enumerate inline only.

// ===========================================================================
// V-18 — chunk-shuffle-cross-chunk-rebinding — DISP-B (BELONGS-NAMED-NOW)
// ===========================================================================
//
// **Disposition: BELONGS-NAMED-NOW per HARD RULE 12 clause (b).**
//
// Named destination: R3-W3's
// `crates/benten-graph/tests/tf3d_per_chunk_aead_iroh_block_size.rs`
// PIN 5 at L204-238 (the actual chunk-shuffle rebinding-attack pin,
// owner: G-CORE-3d per-chunk AEAD with AAD-binds-chunk-index per R2 §5
// "Chunk-shuffle / cross-chunk rebinding"). This file's sync-attack-
// fabric scope is the apply_atrium_merge / recheck / chain-validator
// path; the per-chunk-AEAD chunk-shuffle defense is the G-CORE-3d
// encryption-layer wave (R3-W3 partition; previous citation pointed at
// W2 — corrected per R4.1 M-3 / orchestrator triage 2026-05-22).
//
// No `#[test]` body — DISP-B vectors enumerate inline only.
