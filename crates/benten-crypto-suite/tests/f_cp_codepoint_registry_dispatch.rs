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
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! The tests that pin ALREADY-LIVE codepoints (`0x0001/0x0002/0x647a/
//! 0x6400/0x647b/0x647c`) use the REAL `benten_crypto_suite::codepoint`
//! types directly (no stub needed — the symbols exist). The tests that
//! pin NEW codepoints use a SELF-CONTAINED `f_cp_stub` module. R5 MUST:
//!   1. DELETE the `f_cp_stub` module,
//!   2. INSERT the real new-codepoint symbols (minted on `CipherSuiteCodepoint`
//!      / a new `RecipientCodepoint` / `MembershipSetCodepoint` / `CodepointLifecycle`),
//!   3. UN-IGNORE + verify green.

#![allow(dead_code)]

use benten_crypto_suite::codepoint::{CipherSuiteCodepoint, SigCodepoint};
use benten_crypto_suite::error::UnsupportedAlgorithm;

/// SELF-CONTAINED stub-shim for the NEW (not-yet-minted) codepoints.
mod f_cp_stub {
    /// Layer-A vault envelope codepoint (NEW; §4.0 `0x6100`; the 24-byte
    /// `SymmetricAeadXNonce` vault variant).
    pub const VAULT_ENVELOPE: u16 = 0x6100;
    /// Layer-A 12-byte `SymmetricAead` (ChaCha20-Poly1305) sibling (NEW;
    /// §4.0 vault band `0x6100..0x61FF`; RATIFIED frozen v1-beta variant
    /// per Ben ruling 3, 2026-06-02 + R0.5 §4.1 "ship both").
    pub const SYMMETRIC_AEAD_12B: u16 = 0x6101;
    /// Layer-C plaintext-sender drop (NEW; §4.0 `0x6500`).
    pub const LAYER_C_DROP: u16 = 0x6500;
    /// Layer-C Sealed-Sender DEFAULT (NEW; §4.0 `0x6510`; BR-1 the ONE
    /// canonical value).
    pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;
    /// Layer-C group multi-stanza (NEW; §4.0 `0x6520`).
    pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520;
    /// MembershipSet set-keying envelope (NEW; §4.0 `0x6600`; RELOCATED
    /// from M-CONS-FINAL `0x6380` per §0.4).
    pub const MEMBERSHIP_SET_ENCRYPTION: u16 = 0x6600;
    /// MembershipSet group multi-stanza (NEW; §4.0 `0x6610`).
    pub const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;
    /// MembershipSet federation acquisition (NEW; §4.0 `0x6620`;
    /// reserve-only — refused at v1-beta).
    pub const MEMBERSHIP_SET_SUBSET_REF: u16 = 0x6620;
    /// Layer-D DeviceLink band base (NEW; §4.0 `0x6310..0x631F`).
    pub const DEVICE_LINK_BAND_BASE: u16 = 0x6310;
    /// Layer-D RemotePermission band base (NEW; §4.0 `0x6320..0x632F`).
    pub const REMOTE_PERMISSION_BAND_BASE: u16 = 0x6320;
    /// Lifecycle/revocation band base (NEW; §4.0 `0x6700..0x67FF`).
    pub const LIFECYCLE_BAND_BASE: u16 = 0x6700;
    /// MLS-Application bracket base (NEW; §4.0 `0x6380`; NOT MembershipSet).
    pub const MLS_APPLICATION_BASE: u16 = 0x6380;
    /// MLS-Welcome bracket base (NEW; §4.0 `0x6390`; NOT Sealed-Sender).
    pub const MLS_WELCOME_BASE: u16 = 0x6390;
    /// CGKA-Commit FS-future bracket (NEW; §4.0 `0x63A0`).
    pub const CGKA_COMMIT_BASE: u16 = 0x63A0;
    /// Bird-of-Prey AKEM FS-future bracket (NEW; §4.0 `0x63B0`).
    pub const BIRD_OF_PREY_BASE: u16 = 0x63B0;
    /// draft-prabel FS-future bracket (NEW; §4.0 `0x63C0`).
    pub const DRAFT_PRABEL_BASE: u16 = 0x63C0;
    /// Experimental-range base (NEW; §4.0 `0xFE00..0xFFFE`).
    pub const EXPERIMENTAL_BASE: u16 = 0xFE00;
    /// Extended-codepoint escape (NEW; §4.0 `0xFFFF`).
    pub const EXTENDED_CODEPOINT_ESCAPE: u16 = 0xFFFF;

    /// The Benten envelope range (NEW; §4.0 `0x6100..=0x6FFF`).
    pub const BENTEN_ENVELOPE_RANGE: std::ops::RangeInclusive<u16> = 0x6100..=0x6FFF;

    /// The set of IANA HPKE kem/kdf/aead 16-bit registry ranges a Benten
    /// envelope codepoint must AVOID (a representative low-band slice; R5
    /// wires the real IANA range table). Used by F-CP-2 IANA-disjointness.
    pub fn iana_hpke_reserved_ranges() -> Vec<std::ops::RangeInclusive<u16>> {
        // IANA HPKE registries live in the low 16-bit space (e.g.
        // kem_id 0x0010..0x0021, kdf_id 0x0001..0x0003, aead_id
        // 0x0001..0x0003). The Benten band 0x6100.. is disjoint by
        // construction; R5 pins the authoritative table.
        vec![0x0001..=0x0003, 0x0010..=0x0021]
    }

    /// All Benten-assigned envelope codepoint integers from the FULL R0.5
    /// §4.0 table (the non-collision scanner input). R5 wires this to
    /// enumerate the REAL minted symbols (a source-scan / registry
    /// iterator), not this literal; the stub lists every §4.0 in-band
    /// integer so the non-collision + IANA-disjoint scanner has the full
    /// authoritative set (F4-040).
    ///
    /// Sig codepoints (`0x0001/0x0002/0x0003`) are a SEPARATE `0x00xx`
    /// namespace (R0.5 §4.0) and are deliberately NOT in this envelope set.
    pub fn all_assigned_envelope_codepoints() -> Vec<u16> {
        vec![
            VAULT_ENVELOPE,    // 0x6100 vault (24-byte XNonce)
            SYMMETRIC_AEAD_12B, // 0x6101 vault 12-byte sibling
            0x6400,            // cipher classical-only X25519 downgrade
            0x647a,            // cipher hybrid default (X-Wing X25519⊕ML-KEM-768)
            0x647b,            // cipher NF-1 PQ⊕PQ (reserved)
            0x647c,            // cipher pure-PQ swap-matrix arm (reserved)
            DEVICE_LINK_BAND_BASE,        // 0x6310 Layer-D DeviceLink band base
            REMOTE_PERMISSION_BAND_BASE,  // 0x6320 Layer-D RemotePermission band base
            MLS_APPLICATION_BASE,         // 0x6380 MLS-Application bracket
            MLS_WELCOME_BASE,             // 0x6390 MLS-Welcome bracket
            CGKA_COMMIT_BASE,             // 0x63A0 CGKA-Commit FS bracket
            BIRD_OF_PREY_BASE,            // 0x63B0 Bird-of-Prey AKEM FS bracket
            DRAFT_PRABEL_BASE,            // 0x63C0 draft-prabel FS bracket
            LAYER_C_DROP,                 // 0x6500 plaintext-sender drop (non-default sibling)
            DROP_TO_RECIPIENT_SEALED_SENDER, // 0x6510 Sealed-Sender DEFAULT
            LAYER_C_DROP_MULTI_RECIPIENT, // 0x6520 group multi-stanza
            MEMBERSHIP_SET_ENCRYPTION,        // 0x6600 MembershipSet set-keying
            MEMBERSHIP_SET_GROUP_MULTI_STANZA, // 0x6610 MembershipSet group multi-stanza
            MEMBERSHIP_SET_SUBSET_REF,        // 0x6620 MembershipSet federation (reserve)
            LIFECYCLE_BAND_BASE,          // 0x6700 lifecycle / revocation band base
        ]
    }

    /// `CodepointLifecycle` typed-state (NEW; §4.1 U16).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CodepointLifecycle {
        Live,
        Deprecated,
        Quarantined,
        Burned,
    }

    /// Dispatch a codepoint by its lifecycle state — Quarantined/Burned
    /// MUST reject. STUB always accepts (ignores `state`) so the
    /// Burned/Quarantined-reject pins fire red until R5 wires the real
    /// state machine.
    pub fn lifecycle_dispatch(state: CodepointLifecycle) -> Result<(), &'static str> {
        // STUB BUG (intentional): accepts everything regardless of state.
        let _ = state;
        Ok(())
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
#[ignore = "RED-PHASE: F-CP-1 — §4.0 codepoint integers (live arms) wire-locked; un-ignore at R5 (some assertions are live today; the file rides the V2 corpus regen per M-20)"]
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
#[ignore = "RED-PHASE: F-CP-1 — §4.0 NEW codepoint integers wire-locked incl. Sealed-Sender 0x6510 (the ONE canonical value) + MembershipSet-group 0x6610 + Layer-C-group 0x6520 (R4.6 corrections); un-ignore at R5"]
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
        f_cp_stub::MEMBERSHIP_SET_GROUP_MULTI_STANZA, 0x6610,
        "MembershipSet group multi-stanza is 0x6610 NOT the 0x6600 set-keying value (R0.7 §4.0 \
         allocation table — THE single source of truth for the integer; §3.10/§4.1 the BLINDED \
         11-field AAD that binds it; the R4.5b migration slipped this to 0x6600 in 'settled' \
         territory — locked here so a future single-const edit re-introducing 0x6600 fails THIS \
         registry test directly, not only via the cross-file f_aad_2 golden)"
    );
    assert_eq!(
        f_cp_stub::LAYER_C_DROP_MULTI_RECIPIENT, 0x6520,
        "Layer-C group multi-stanza (the blinded 8-field set's codepoint) is 0x6520 (R0.7 §3.3/§4.0; \
         locked here so a single-const drift fails THIS registry test directly, not only via the \
         cross-file f_lc_hpke golden)"
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
#[ignore = "RED-PHASE: F-CP-2 — intra-band non-collision + IANA-disjoint scanner + injection arm (NQ-W2; F4-040); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-CP-1 — every newly-minted §4.0 const is present in the scanned set (F4-041); un-ignore at R5"]
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
        ("DROP_TO_RECIPIENT_SEALED_SENDER", DROP_TO_RECIPIENT_SEALED_SENDER),
        ("LAYER_C_DROP_MULTI_RECIPIENT", f_cp_stub::LAYER_C_DROP_MULTI_RECIPIENT),
        ("DEVICE_LINK_BAND_BASE", f_cp_stub::DEVICE_LINK_BAND_BASE),
        ("REMOTE_PERMISSION_BAND_BASE", f_cp_stub::REMOTE_PERMISSION_BAND_BASE),
        ("MLS_APPLICATION_BASE", MLS_APPLICATION_BASE),
        ("MLS_WELCOME_BASE", MLS_WELCOME_BASE),
        ("CGKA_COMMIT_BASE", CGKA_COMMIT_BASE),
        ("BIRD_OF_PREY_BASE", f_cp_stub::BIRD_OF_PREY_BASE),
        ("DRAFT_PRABEL_BASE", DRAFT_PRABEL_BASE),
        ("MEMBERSHIP_SET_ENCRYPTION", MEMBERSHIP_SET_ENCRYPTION),
        ("MEMBERSHIP_SET_GROUP_MULTI_STANZA", f_cp_stub::MEMBERSHIP_SET_GROUP_MULTI_STANZA),
        ("MEMBERSHIP_SET_SUBSET_REF", f_cp_stub::MEMBERSHIP_SET_SUBSET_REF),
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
#[ignore = "RED-PHASE: F-CP-3 — dispatch strict-reject / no cross-variant fallback (U2); un-ignore at R5 (extends tf2 dispatch pin; rides V2 corpus)"]
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
#[ignore = "RED-PHASE: F-CP-4 — CodepointLifecycle typed-state (Burned/Quarantined reject); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-CP-5 — sig-side swap matrix 0x0002/0x0003 (GAP-1a/1e); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-CP-6 — FS-future bracket typed-reject 0x63A0/0x63B0/0x63C0; un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-CP-7 — MLS-bracket collision regression-guard (MembershipSet=0x6600 NOT 0x6380); un-ignore at R5"]
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
