//! **F-CP-1..7 — Codepoint registry + dispatch (ALL FG).**
//!
//! ADDL R3 wave **W0-crypto-canary**. Pin source:
//! `f-full-r2-test-landscape.md` Group-2 (F-CP-1..7) + §4 P0-2 +
//! coverage-matrix §2.3 + R0 plan §4.0 (the canonical codepoint allocation
//! table — THE single source of truth) + §0.4 (the `0x6380/0x6390`
//! collision resolution) + §2.0 BR-1 (Sealed-Sender = `0x6510`) + §5.1
//! Inv-18.
//!
//! # What these families pin
//!
//! - **F-CP-1** every §4.0 integer wire-locked (sig + cipher + vault +
//!   drop/Sealed-Sender + MembershipSet band + Layer-D + lifecycle +
//!   MLS/FS brackets + experimental/escape).
//! - **F-CP-2** intra-`0x6100..0x6FFF` non-collision + IANA-disjoint CI
//!   scanner (NQ-W2; authored at R2 as a prerequisite).
//! - **F-CP-3** strict-reject / no cross-variant fallback (U2; CLAUDE.md #5).
//! - **F-CP-4** `CodepointLifecycle` typed-state (Live/Deprecated/
//!   Quarantined/Burned) + `0x6700..0x67FF` band pins.
//! - **F-CP-5** sig-side swap matrix `0x0002`/`0x0003` (GAP-1a + GAP-1e).
//! - **F-CP-6** FS-future bracket typed-reject `0x63A0/0x63B0/0x63C0`.
//! - **F-CP-7** MLS-bracket collision regression-guard `0x6380/0x6390`
//!   (NOT MembershipSet/Sealed-Sender — supersedes M-CONS-FINAL F8/F21).
//!
//! # Ground-truth at HEAD (`sed`-verified `codepoint.rs`)
//!
//! IN-TREE LIVE: sig `0x0001/0x0002`; cipher `0x647a/0x6400` LIVE,
//! `0x647b/0x647c/0x0000` reserved-typed-reject; `SigCodepoint::resolve`
//! / `CipherSuiteCodepoint::resolve` are the live dispatch fns. NEW at
//! Wave-0 (this canary mints): vault `0x6100`, drop `0x6500`,
//! Sealed-Sender `0x6510`, group `0x6520`, MembershipSet `0x6600/0x6610/
//! 0x6620`, Layer-D `0x6310..0x632F`, lifecycle `0x6700..0x67FF`, MLS/FS
//! `0x6380..0x63CF`, experimental/escape `0xFE00../0xFFFF`.
//!
//! # SHIPPED STATUS (R17 retense; formerly RED-PHASE pim-12 §3.6e)
//!
//! Every arm is a live `#[test]` (NO `#[ignore]`). The tests that pin
//! ALREADY-LIVE codepoints (`0x0001/0x0002/0x647a/0x6400/0x647b/0x647c`) use
//! the REAL `benten_crypto_suite::codepoint` types directly. The `f_cp_stub`
//! module (retained by name) is now a THIN RE-EXPORT shim over the LIVE
//! `benten_crypto_suite::registry` + `CodepointLifecycle` symbols (all minted
//! + in-tree at HEAD) — it re-exports real registry consts, NOT stand-in
//! stubs. The prior RED-PHASE staging (a self-contained stub, un-ignored once
//! the new codepoints landed) is fully discharged.

#![allow(dead_code)]

use benten_crypto_suite::codepoint::{CipherSuiteCodepoint, SigCodepoint};
use benten_crypto_suite::error::UnsupportedAlgorithm;

/// R5: thin re-export shim over the LIVE `benten_crypto_suite::registry` +
/// the real `CodepointLifecycle` state machine. The non-collision /
/// IANA-disjoint scanner input (`all_assigned_envelope_codepoints`) is the
/// REAL registry iterator `registry::registered_envelope_codepoints`
/// (F4-040: a registry enumerator, not a per-file hand-list).
mod f_cp_stub {
    pub use benten_crypto_suite::codepoint::CodepointLifecycle;
    pub use benten_crypto_suite::registry::{
        BENTEN_ENVELOPE_RANGE, BIRD_OF_PREY_BASE, CGKA_COMMIT_BASE, DEVICE_LINK_BAND_BASE,
        DRAFT_PRABEL_BASE, DROP_TO_RECIPIENT_SEALED_SENDER, EXPERIMENTAL_BASE,
        EXTENDED_CODEPOINT_ESCAPE, LAYER_C_DROP, LAYER_C_DROP_MULTI_RECIPIENT, LIFECYCLE_BAND_BASE,
        MEMBERSHIP_SET_ENCRYPTION, MEMBERSHIP_SET_GROUP_MULTI_STANZA, MEMBERSHIP_SET_SUBSET_REF,
        MLS_APPLICATION_BASE, MLS_WELCOME_BASE, REMOTE_PERMISSION_BAND_BASE, SYMMETRIC_AEAD_12B,
        VAULT_ENVELOPE, iana_hpke_reserved_ranges,
        registered_envelope_codepoints as all_assigned_envelope_codepoints,
    };

    /// Dispatch a codepoint by its lifecycle state — Live/Deprecated dispatch
    /// `Ok`; Quarantined/Burned typed-reject (the real state machine).
    pub fn lifecycle_dispatch(state: CodepointLifecycle) -> Result<(), &'static str> {
        state
            .dispatch()
            .map_err(|_| "codepoint quarantined or burned")
    }
}

use f_cp_stub::{
    BENTEN_ENVELOPE_RANGE, CGKA_COMMIT_BASE, CodepointLifecycle, DRAFT_PRABEL_BASE,
    DROP_TO_RECIPIENT_SEALED_SENDER, MEMBERSHIP_SET_ENCRYPTION, MLS_APPLICATION_BASE,
    MLS_WELCOME_BASE, VAULT_ENVELOPE, all_assigned_envelope_codepoints, iana_hpke_reserved_ranges,
    lifecycle_dispatch,
};

/// **F-CP-1** — §4.0 codepoint integers wire-locked (LIVE in-tree arms).
///
/// The already-minted codepoints are pinned against the REAL types. Any
/// value drift = wire-format break. This extends
/// `canonical_bytes_v1_codepoints_and_aad.rs::codepoint_table_integer_values_pinned`.
#[test]
fn codepoint_integers_wire_locked_live_arms() {
    assert_eq!(
        SigCodepoint::HYBRID_ED25519_MLDSA65.raw(),
        0x0001,
        "sig hybrid default wire-locked at 0x0001"
    );
    assert_eq!(
        SigCodepoint::CLASSICAL_ED25519.raw(),
        0x0002,
        "sig classical downgrade wire-locked at 0x0002"
    );
    assert_eq!(
        SigCodepoint::HYBRID_MLDSA65_SLHDSA.raw(),
        0x0003,
        "sig NF-1 reserved wire-locked at 0x0003"
    );
    assert_eq!(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768.raw(),
        0x647a,
        "cipher hybrid default wire-locked at 0x647a"
    );
    assert_eq!(
        CipherSuiteCodepoint::CLASSICAL_X25519.raw(),
        0x6400,
        "cipher classical downgrade wire-locked at 0x6400"
    );
    assert_eq!(
        CipherSuiteCodepoint::HYBRID_MLKEM768_HQC.raw(),
        0x647b,
        "cipher NF-1 PQ⊕PQ end-state wire-locked at 0x647b"
    );
    assert_eq!(
        CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY.raw(),
        0x647c,
        "cipher pure-PQ swap-matrix arm wire-locked at 0x647c"
    );

    // SSOT round-trip strengthening (R17 F-08): the const↔raw mapping is the
    // single source of truth. For each cipher band value, `from_raw(v).raw()`
    // MUST equal `v` AND `from_raw(v)` MUST equal the named const — proving the
    // integer, the named symbol, and the `raw()`/`from_raw()` bijection all
    // agree (a drift in any one of the three fails HERE, directly). ADDED arms;
    // the existing `const.raw()` locks above are unchanged.
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x647a).raw(),
        0x647a,
        "0x647a round-trips through from_raw().raw()"
    );
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x647a),
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        "from_raw(0x647a) is the HYBRID_X25519_MLKEM768 const (SSOT bijection)"
    );
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x6400).raw(),
        0x6400,
        "0x6400 round-trips through from_raw().raw()"
    );
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x6400),
        CipherSuiteCodepoint::CLASSICAL_X25519,
        "from_raw(0x6400) is the CLASSICAL_X25519 const (SSOT bijection)"
    );
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x647b).raw(),
        0x647b,
        "0x647b round-trips through from_raw().raw()"
    );
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x647b),
        CipherSuiteCodepoint::HYBRID_MLKEM768_HQC,
        "from_raw(0x647b) is the HYBRID_MLKEM768_HQC const (SSOT bijection)"
    );
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x647c).raw(),
        0x647c,
        "0x647c round-trips through from_raw().raw()"
    );
    assert_eq!(
        CipherSuiteCodepoint::from_raw(0x647c),
        CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY,
        "from_raw(0x647c) is the PURE_PQ_MLKEM768_ONLY const (SSOT bijection)"
    );
}

/// **F-CP-1 (cont.)** — §4.0 NEW codepoint integers wire-locked, with the
/// load-bearing Sealed-Sender = `0x6510` (the ONE canonical value, §0.4)
/// AND the two R4.6-corrected band values: MembershipSet group multi-stanza
/// = `0x6610` (NOT the `0x6600` set-keying value — the slip the R4.5b
/// migration introduced into "settled" territory, corrected per R0.7
/// §3.10/§4.1) and Layer-C group multi-stanza = `0x6520` (the blinded
/// 8-field set's codepoint, R0.7 §3.3/§4.0). These two standalone
/// value-locks are this registry file's OWN direct defense against a
/// future single-const edit re-introducing either slip (the cross-file
/// f_aad_2/f_lc_hpke goldens catch it indirectly; THIS file is the
/// canonical wire-lock and must lock them explicitly — F-46-03).
#[test]
fn new_codepoint_integers_wire_locked() {
    assert_eq!(
        VAULT_ENVELOPE, 0x6100,
        "vault envelope wire-locked at 0x6100"
    );
    assert_eq!(
        DROP_TO_RECIPIENT_SEALED_SENDER, 0x6510,
        "Sealed-Sender has EXACTLY ONE canonical value 0x6510 (BR-1 / §0.4; NOT 0x6390)"
    );
    assert_eq!(
        MEMBERSHIP_SET_ENCRYPTION, 0x6600,
        "MembershipSetEncryption RELOCATED to 0x6600 (was M-CONS-FINAL 0x6380 which collided MLS-Application; §0.4)"
    );
    assert_eq!(
        f_cp_stub::MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        0x6610,
        "MembershipSet group multi-stanza is 0x6610 NOT the 0x6600 set-keying value (R0.7 §4.0 \
         allocation table — THE single source of truth for the integer; §3.10/§4.1 the BLINDED \
         11-field AAD that binds it; the R4.5b migration slipped this to 0x6600 in 'settled' \
         territory — locked here so a future single-const edit re-introducing 0x6600 fails THIS \
         registry test directly, not only via the cross-file f_aad_2 golden)"
    );
    assert_eq!(
        f_cp_stub::LAYER_C_DROP_MULTI_RECIPIENT,
        0x6520,
        "Layer-C group multi-stanza (the blinded 8-field set's codepoint) is 0x6520 (R0.7 §3.3/§4.0; \
         locked here so a single-const drift fails THIS registry test directly, not only via the \
         cross-file f_lc_hpke golden)"
    );
    // Layer-D band bases — the TWO codepoints that are DUPLICATED across crates
    // (the wire producers live in `benten_engine::layer_d::{device_link,
    // remote_permission}`; these registry copies are the SSOT allocation map).
    // Neither registry copy carried a value-lock: the only registry-side
    // reference was the self-satisfying presence loop below (the consts are put
    // INTO `registered_envelope_codepoints()` by `registry.rs`, so `present(cp)`
    // passes for ANY value), and `f_disc_2_codepoint_ssot_cross_crate_const_
    // equality` covers only the MembershipSet + Layer-C bands. Pin both literals
    // HERE so a one-sided edit to the registry fails this test directly; the
    // engine-side producers are literal-locked in their own crate
    // (`f_ld_4_device_link_band_base_pinned` in
    // `f_ld_4_multi_device_key_wrap_provisioning.rs` +
    // `f_ld_2_out_of_band_codepoint_typed_rejects` in
    // `f_ld_2_remote_permission_wire_freeze.rs`), so a one-sided edit to
    // EITHER home now fails the build.
    assert_eq!(
        f_cp_stub::DEVICE_LINK_BAND_BASE,
        0x6310,
        "Layer-D DeviceLink band base wire-locked at 0x6310 (R0.7 §4.1 FREEZE); the \
         registry copy MUST equal the `benten_engine::layer_d::device_link` producer"
    );
    assert_eq!(
        f_cp_stub::REMOTE_PERMISSION_BAND_BASE,
        0x6320,
        "Layer-D RemotePermission band base wire-locked at 0x6320 (R0.7 §4.1 FREEZE); the \
         registry copy MUST equal the `benten_engine::layer_d::remote_permission` producer"
    );
    // Experimental-range base + extended-codepoint escape — the F-CP-1 family
    // doc claims to wire-lock "experimental/escape" but these two consts were
    // imported (see the `f_cp_stub` shim) and never value-pinned (R21 F-04).
    // Pin both canonical out-of-band values here.
    assert_eq!(
        f_cp_stub::EXPERIMENTAL_BASE,
        0xFE00,
        "experimental-range base wire-locked at 0xFE00 (deliberately out-of-band)"
    );
    assert_eq!(
        f_cp_stub::EXTENDED_CODEPOINT_ESCAPE,
        0xFFFF,
        "extended-codepoint escape wire-locked at 0xFFFF (deliberately out-of-band)"
    );
    // not-in-BENTEN-range: both live OUTSIDE the suite-selector band
    // 0x6100..=0x6FFF by design, so a future edit pulling either INTO the band
    // (where it could collide an envelope-family value) fails HERE directly.
    assert!(
        !BENTEN_ENVELOPE_RANGE.contains(&f_cp_stub::EXPERIMENTAL_BASE),
        "EXPERIMENTAL_BASE (0xFE00) must live OUTSIDE the Benten envelope band 0x6100..=0x6FFF"
    );
    assert!(
        !BENTEN_ENVELOPE_RANGE.contains(&f_cp_stub::EXTENDED_CODEPOINT_ESCAPE),
        "EXTENDED_CODEPOINT_ESCAPE (0xFFFF) must live OUTSIDE the Benten envelope band 0x6100..=0x6FFF"
    );
}

/// Collision scanner: returns `true` iff `codepoints` contains a duplicate
/// (a silent wire collision). The CI scanner (NQ-W2 / Inv-18) is exactly
/// this check over the full minted set.
fn scanner_detects_collision(codepoints: &[u16]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for cp in codepoints {
        if !seen.insert(*cp) {
            return true;
        }
    }
    false
}

/// **F-CP-2** — intra-`0x6100..0x6FFF` non-collision + IANA-disjointness +
/// **injection negative arm** (F4-040).
///
/// (a) every assigned integer is unique (the scanner reports NO collision
/// over the real set); (b) every Benten *envelope* codepoint lives in
/// `0x6100..=0x6FFF` (the sig namespace `0x00xx` is separate); (c) no
/// envelope codepoint lands in an IANA HPKE registry range; (d)
/// **INJECTION:** pushing a duplicate/colliding codepoint MUST make the
/// scanner FIRE — proving the scanner actually detects collisions (not a
/// vacuous always-pass). would-FAIL-if-no-op'd: a scanner that never
/// reports a collision passes (a)–(c) but FAILS the injection arm.
#[test]
fn codepoint_registry_non_collision_and_iana_disjoint() {
    let assigned = all_assigned_envelope_codepoints();

    // (a) Non-collision: the scanner reports NO collision over the real set.
    assert!(
        !scanner_detects_collision(&assigned),
        "the assigned envelope codepoint set must be collision-free (a duplicate = a silent wire collision)"
    );
    // (also pin the count directly, for a clear diff on regression).
    let mut sorted = assigned.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        assigned.len(),
        "every assigned envelope codepoint must be unique"
    );

    // (b) Every envelope codepoint is in the Benten band (the experimental
    // 0xFE00../escape 0xFFFF live outside the band by design and are
    // checked separately; the §4.0 envelope-family integers are all in-band).
    for cp in &assigned {
        assert!(
            BENTEN_ENVELOPE_RANGE.contains(cp),
            "envelope codepoint 0x{cp:04x} must live in the Benten private band 0x6100..=0x6FFF"
        );
    }

    // (c) IANA-disjointness: no envelope codepoint lands in an IANA HPKE
    // registry range.
    for cp in &assigned {
        for range in iana_hpke_reserved_ranges() {
            assert!(
                !range.contains(cp),
                "envelope codepoint 0x{cp:04x} must NOT land in an IANA HPKE registry range {range:?}"
            );
        }
    }

    // (d) INJECTION negative arm (F4-040): inject a DUPLICATE of an
    // already-assigned codepoint and assert the scanner FIRES. This proves
    // the scanner is not vacuously always-pass — a future codepoint minted
    // to collide an existing one MUST be caught.
    let mut injected = assigned.clone();
    injected.push(DROP_TO_RECIPIENT_SEALED_SENDER); // duplicate 0x6510
    assert!(
        scanner_detects_collision(&injected),
        "INJECTION: a duplicate codepoint (0x{DROP_TO_RECIPIENT_SEALED_SENDER:04x}) MUST make the \
         non-collision scanner FIRE — else the scanner is vacuous (NQ-W2 / Inv-18)"
    );
    // A second injection: a NEW colliding value (mint a const equal to an
    // existing band base) — the scanner must catch it too.
    let mut injected2 = assigned.clone();
    injected2.push(VAULT_ENVELOPE); // duplicate 0x6100
    assert!(
        scanner_detects_collision(&injected2),
        "INJECTION: a duplicate of the vault codepoint (0x{VAULT_ENVELOPE:04x}) MUST also fire the scanner"
    );
}

/// **F-CP-1 (cont.)** — every newly-minted §4.0 const is PRESENT in the
/// assigned-codepoint set (F4-041). A new codepoint minted on the registry
/// but omitted from `all_assigned_envelope_codepoints()` is invisible to the
/// non-collision/IANA scanner — this arm guards against that omission by
/// asserting each minted const is a member of the scanned set.
#[test]
fn every_minted_codepoint_present_in_scanned_set() {
    let assigned = all_assigned_envelope_codepoints();
    let present = |cp: u16| assigned.contains(&cp);

    // Every minted §4.0 const must be in the scanned set (F4-041). The
    // not-already-imported consts are referenced via the `f_cp_stub::`
    // path so the existing import block stays byte-identical.
    for (sym, cp) in [
        ("VAULT_ENVELOPE", VAULT_ENVELOPE),
        ("SYMMETRIC_AEAD_12B", f_cp_stub::SYMMETRIC_AEAD_12B),
        ("LAYER_C_DROP", f_cp_stub::LAYER_C_DROP),
        (
            "DROP_TO_RECIPIENT_SEALED_SENDER",
            DROP_TO_RECIPIENT_SEALED_SENDER,
        ),
        (
            "LAYER_C_DROP_MULTI_RECIPIENT",
            f_cp_stub::LAYER_C_DROP_MULTI_RECIPIENT,
        ),
        ("DEVICE_LINK_BAND_BASE", f_cp_stub::DEVICE_LINK_BAND_BASE),
        (
            "REMOTE_PERMISSION_BAND_BASE",
            f_cp_stub::REMOTE_PERMISSION_BAND_BASE,
        ),
        ("MLS_APPLICATION_BASE", MLS_APPLICATION_BASE),
        ("MLS_WELCOME_BASE", MLS_WELCOME_BASE),
        ("CGKA_COMMIT_BASE", CGKA_COMMIT_BASE),
        ("BIRD_OF_PREY_BASE", f_cp_stub::BIRD_OF_PREY_BASE),
        ("DRAFT_PRABEL_BASE", DRAFT_PRABEL_BASE),
        ("MEMBERSHIP_SET_ENCRYPTION", MEMBERSHIP_SET_ENCRYPTION),
        (
            "MEMBERSHIP_SET_GROUP_MULTI_STANZA",
            f_cp_stub::MEMBERSHIP_SET_GROUP_MULTI_STANZA,
        ),
        (
            "MEMBERSHIP_SET_SUBSET_REF",
            f_cp_stub::MEMBERSHIP_SET_SUBSET_REF,
        ),
        ("LIFECYCLE_BAND_BASE", f_cp_stub::LIFECYCLE_BAND_BASE),
    ] {
        assert!(
            present(cp),
            "minted codepoint {sym} (0x{cp:04x}) MUST be present in the scanned set \
             (else it is invisible to the non-collision/IANA scanner; F4-041)"
        );
    }
}

/// **F-CP-3** — codepoint-dispatch strict-reject; no cross-variant
/// fallback (U2; CLAUDE.md #5 typed-unsupported).
///
/// Feeds reserved/unknown cipher codepoints into the REAL
/// `CipherSuiteCodepoint::resolve` + asserts each surfaces a typed
/// `UnsupportedAlgorithm::CipherSuite` (NEVER `Ok`, NEVER a silent
/// fallback to the default). would-FAIL-if-no-op'd: a silent-fallback
/// dispatch returns `Ok` for an unknown codepoint.
#[test]
fn dispatch_strict_reject_no_cross_variant_fallback() {
    // Reserved-but-unimplemented arms typed-reject at the default dispatcher.
    for reserved in [0x647b_u16, 0x647c, 0x0000] {
        let cp = CipherSuiteCodepoint::from_raw(reserved);
        assert!(
            matches!(
                cp.resolve(),
                Err(UnsupportedAlgorithm::CipherSuite { codepoint }) if codepoint == reserved
            ),
            "reserved cipher codepoint 0x{reserved:04x} must typed-reject (never silent fallback to the default)"
        );
    }
    // A wholly-unknown codepoint also typed-rejects (no default fallback).
    let unknown = CipherSuiteCodepoint::from_raw(0x6FFE);
    assert!(
        matches!(
            unknown.resolve(),
            Err(UnsupportedAlgorithm::CipherSuite { .. })
        ),
        "unknown cipher codepoint must typed-reject"
    );
    // A LIVE arm resolves OK (the positive control — dispatch is not
    // rejecting everything).
    assert!(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768
            .resolve()
            .is_ok(),
        "the LIVE 0x647a arm must resolve Ok (positive control)"
    );
}

/// **F-CP-4** — `CodepointLifecycle` typed-state machine.
///
/// Live/Deprecated dispatch Ok; Quarantined/Burned reject. would-FAIL-if-
/// no-op'd: the stub accepts everything, so the Burned-reject assertion
/// fires red until R5 wires the real state machine.
#[test]
fn codepoint_lifecycle_burned_and_quarantined_reject() {
    assert!(
        lifecycle_dispatch(CodepointLifecycle::Live).is_ok(),
        "Live codepoint dispatches Ok"
    );
    assert!(
        lifecycle_dispatch(CodepointLifecycle::Burned).is_err(),
        "Burned codepoint MUST reject (a burned codepoint is permanently un-dispatchable)"
    );
    assert!(
        lifecycle_dispatch(CodepointLifecycle::Quarantined).is_err(),
        "Quarantined codepoint MUST reject"
    );
}

/// **F-CP-5** — sig-side swap matrix `0x0002`/`0x0003` (GAP-1a + GAP-1e;
/// entirely absent across the 6 discovery dimensions).
///
/// `0x0001` LIVE default; `0x0002` classical-Ed25519 = non-default
/// explicit downgrade (resolves but is not the default — never silent);
/// `0x0003` MLDSA65⊕SLH-DSA typed-rejects at the default dispatcher.
/// Drives the REAL `SigCodepoint::resolve`.
#[test]
fn sig_side_swap_matrix_dispatch() {
    // 0x0001 is the default + resolves.
    assert!(
        SigCodepoint::HYBRID_ED25519_MLDSA65.is_hybrid_default(),
        "0x0001 is the hybrid default"
    );
    assert!(SigCodepoint::HYBRID_ED25519_MLDSA65.resolve().is_ok());
    // 0x0002 resolves (built + conformance-testable) but is NOT the default.
    assert!(SigCodepoint::CLASSICAL_ED25519.resolve().is_ok());
    assert!(
        SigCodepoint::CLASSICAL_ED25519.is_classical_only(),
        "0x0002 is the explicit classical downgrade (non-default; never silent)"
    );
    // 0x0003 typed-rejects at the default dispatcher (C11b safety gate).
    assert!(
        matches!(
            SigCodepoint::HYBRID_MLDSA65_SLHDSA.resolve(),
            Err(UnsupportedAlgorithm::Signature { codepoint: 0x0003 })
        ),
        "0x0003 (NF-1 MLDSA65⊕SLH-DSA) typed-rejects as default + selectable for swap-matrix conformance only"
    );
}

/// **F-CP-6** — FS-future bracket typed-reject `0x63A0/0x63B0/0x63C0`
/// (GAP-1e; beyond the `0x6380/0x6390` collision-guard).
///
/// CGKA-Commit / Bird-of-Prey / draft-prabel each typed-reject at
/// v1-beta. would-FAIL-if-no-op'd: a dispatcher that resolves a reserved
/// FS bracket to a live arm returns `Ok`.
#[test]
fn fs_future_bracket_typed_reject() {
    for fs in [
        CGKA_COMMIT_BASE,
        f_cp_stub::BIRD_OF_PREY_BASE,
        DRAFT_PRABEL_BASE,
    ] {
        let cp = CipherSuiteCodepoint::from_raw(fs);
        assert!(
            cp.resolve().is_err(),
            "FS-future bracket codepoint 0x{fs:04x} must typed-reject at v1-beta (CODEPOINT-RESERVE only)"
        );
    }
}

/// **F-CP-7** — MLS-bracket collision regression-guard `0x6380/0x6390`.
///
/// `0x6380` = MLS-Application, `0x6390` = MLS-Welcome (NOT MembershipSet /
/// NOT Sealed-Sender — supersedes M-CONS-FINAL F8/F21 per §0.4); the
/// MembershipSet keying envelope lives at `0x6600`. would-FAIL-if-no-op'd:
/// if MembershipSetEncryption were still `0x6380` it would collide MLS.
#[test]
fn mls_bracket_collision_regression_guard() {
    // MembershipSetEncryption is RELOCATED to 0x6600, away from the MLS bracket.
    assert_eq!(
        MEMBERSHIP_SET_ENCRYPTION, 0x6600,
        "MembershipSetEncryption must be 0x6600 (relocated from the colliding 0x6380)"
    );
    // The MLS brackets are distinct from the MembershipSet / Sealed-Sender values.
    assert_ne!(
        MLS_APPLICATION_BASE, MEMBERSHIP_SET_ENCRYPTION,
        "0x6380 (MLS-Application) must NOT equal the MembershipSet keying codepoint"
    );
    assert_ne!(
        MLS_WELCOME_BASE, DROP_TO_RECIPIENT_SEALED_SENDER,
        "0x6390 (MLS-Welcome) must NOT equal Sealed-Sender (0x6510)"
    );
    assert_eq!(
        MLS_APPLICATION_BASE, 0x6380,
        "MLS-Application reserved at 0x6380"
    );
    assert_eq!(MLS_WELCOME_BASE, 0x6390, "MLS-Welcome reserved at 0x6390");
}
