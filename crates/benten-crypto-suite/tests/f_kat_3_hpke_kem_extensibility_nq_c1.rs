//! **F-KAT-3 — NQ-C1 HPKE KEM-extensibility byte-accuracy (P0 PREREQUISITE).**
//!
//! ADDL R3 wave **W0-crypto-canary** (the SOLE upstream canary). Pin
//! source: `f-full-r2-test-landscape.md` Group-4 F-KAT-3 + §4 P0-1 +
//! §10.6 NQ-C1 + R0 §2.2 (tactical-picks: McMillion `hpke`, NOT Cryspen
//! `hpke-rs`) + §3.3 (Layer-C HpkeBase[MLKEM768-X25519]) + §11-1
//! (one of the 3 items R0 could NOT close in-plan).
//!
//! # Why this file is authored FIRST (before any envelope byte is minted)
//!
//! Per the R2 §4 P0 freeze-gating priority order, **F-KAT-3 resolves
//! BEFORE Canary-ENC-2.** The unresolved question (NQ-C1) is: does the
//! McMillion `hpke` crate admit a *custom / PQ* KEM (X25519MLKEM768) into
//! a real RFC-9180 `mode_base` context **byte-accurately**, OR must
//! Benten supply the KEM itself and reuse only the HPKE KDF / AEAD /
//! key-schedule? The answer determines whether the ENTIRE Layer-C
//! `HpkeBase` / `HpkeMultiBase` byte-format is **RFC-9180-faithful**
//! (interop with any RFC-9180 stack) or **Benten-supplies-the-KEM**
//! (the KEM-encapsulation bytes are Benten-canonical, only the
//! key-schedule is RFC-9180). Every downstream Layer-C/Layer-D envelope
//! family pins bytes whose layout depends on this answer.
//!
//! ## ORCHESTRATOR-FLAGGED OPEN-SPEC ARM (gating NQ = NQ-C1)
//!
//! Ground-truth at HEAD: `hpke` is **NOT yet a workspace dependency**
//! (`grep -rn hpke crates/*/Cargo.toml Cargo.toml` → ZERO). The McMillion
//! `hpke` crate (`rozbb/rust-hpke`) exposes a `kem::Kem` trait whose impl
//! roster is **effectively closed to the RFC-9180-registered KEMs**
//! (X25519HkdfSha256, DhP256HkdfSha256, …) — a downstream custom-KEM impl
//! is NOT a first-class supported extension point, and the `mode_base`
//! context setup is KEM-coupled. (Cryspen `hpke-rs` — the crate this R0
//! explicitly REJECTS for its 13 Feb-2026 CVEs — IS present in the local
//! advisory-db, confirming the right crate to avoid.)
//!
//! **Therefore NQ-C1 is GENUINELY OPEN and this file pins the QUESTION,
//! not a resolved answer.** The test encodes the decision-fork as two
//! mutually-exclusive expectations + a single "exactly-one-branch-holds"
//! meta-assertion, so R5 (and the R2/Ben ratification it gates) MUST pick
//! a branch before the Layer-C bytes freeze. This is the
//! `feedback_surface_arch_decisions_under_auth` discipline applied at the
//! crypto-byte-format layer: a real fork, surfaced, not silently resolved.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! At baseline NONE of {`Hpke`, `HpkeMode`, `RFC9180_MODE_BASE_KEM_DEM`}
//! exist. Per the wave-independence rule (each W0 family is self-contained
//! for parallel safety — NO cross-wave module dependency), this file
//! commits a LOCAL `f_kat_3_stub` module so it compiles green at baseline
//! behind `#[ignore]`. The R5 closing wave MUST:
//!   1. DELETE the local `f_kat_3_stub` module,
//!   2. INSERT real imports against the chosen HPKE binding,
//!   3. RATIFY the NQ-C1 branch (RFC-9180-faithful vs Benten-supplies-KEM),
//!   4. UN-IGNORE + verify the pinned branch's reference bytes PASS green.
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! The pins drive a PRODUCTION call site (`hpke_seal_to_recipient` — the
//! Layer-C single-recipient KEM-DEM seal) + assert an OBSERVABLE
//! consequence (the on-wire encapsulation+ciphertext bytes match the
//! ratified reference vector for the chosen branch) + are
//! would-FAIL-if-no-op'd (a stub that returns the plaintext, or seals
//! under the wrong KEM/key-schedule, produces bytes ≠ the reference).
//! The meta-assertion forbids the "both branches pass" / "neither pins a
//! real vector" no-op shape.

#![allow(dead_code)]

/// SELF-CONTAINED stub-shim (R5 deletes this whole module).
mod f_kat_3_stub {
    /// The two mutually-exclusive resolutions of NQ-C1.
    ///
    /// R5 + the R2/Ben ratification it gates picks EXACTLY ONE. The
    /// chosen branch determines the canonical reference vector the
    /// Layer-C bytes are pinned against.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HpkeKemBinding {
        /// Branch A — McMillion `hpke` admits X25519MLKEM768 into a real
        /// RFC-9180 `mode_base` context byte-accurately; the on-wire bytes
        /// interop with any RFC-9180 HPKE stack.
        Rfc9180FaithfulCustomKem,
        /// Branch B — Benten supplies the X25519MLKEM768 KEM
        /// encapsulation + reuses ONLY the HPKE KDF / AEAD / key-schedule;
        /// the KEM-encapsulation bytes are Benten-canonical (NOT
        /// cross-stack-interop), the key-schedule is RFC-9180.
        BentenSuppliesKemReuseKeySchedule,
    }

    /// The ratified resolution. **STUB = `None` (UNRESOLVED).** R5 sets
    /// this to `Some(branch)` once NQ-C1 is ratified.
    pub const NQ_C1_RESOLUTION: Option<HpkeKemBinding> = None;

    /// Production seal API the real Layer-C single-recipient path exposes.
    /// STUB returns an empty vec (deliberately NOT byte-accurate) so the
    /// reference-vector assertions FAIL until R5 wires the real seal.
    pub fn hpke_seal_to_recipient(
        _recipient_pub: &[u8],
        _plaintext: &[u8],
        _aad: &[u8],
    ) -> Vec<u8> {
        Vec::new()
    }

    /// The published RFC-9180 reference seal-output for Branch A's fixed
    /// test fixture (encapsulated-key ‖ ciphertext ‖ tag). STUB = empty;
    /// R5 swaps in the real external/synthesized witness vector.
    pub fn rfc9180_reference_seal_bytes() -> Vec<u8> {
        Vec::new()
    }

    /// The Benten-canonical reference seal-output for Branch B's fixed
    /// test fixture. STUB = empty; R5 swaps in the real vector.
    pub fn benten_canonical_reference_seal_bytes() -> Vec<u8> {
        Vec::new()
    }
}

use f_kat_3_stub::{
    HpkeKemBinding, NQ_C1_RESOLUTION, benten_canonical_reference_seal_bytes,
    hpke_seal_to_recipient, rfc9180_reference_seal_bytes,
};

/// F-KAT-3 / NQ-C1 — the decision-fork is RATIFIED (exactly one branch).
///
/// This is the meta-assertion that forbids the no-op shape: at R5 close
/// `NQ_C1_RESOLUTION` MUST be `Some(_)` (the fork was decided, not left
/// open) — a frozen Layer-C byte-format with an UNRESOLVED KEM binding is
/// a freeze-gating failure (the bytes would have no canonical meaning).
#[test]
#[ignore = "RED-PHASE: F-KAT-3 — NQ-C1 HPKE KEM-extensibility binding must be RATIFIED (exactly one branch) before Layer-C bytes freeze; un-ignore at R5"]
fn nq_c1_kem_binding_is_ratified_exactly_one_branch() {
    let resolution = NQ_C1_RESOLUTION
        .expect("NQ-C1 MUST be ratified before Layer-C bytes freeze (RFC-9180-faithful vs Benten-supplies-KEM)");
    // Exactly one of the two branches — the enum is the totality witness.
    assert!(
        matches!(
            resolution,
            HpkeKemBinding::Rfc9180FaithfulCustomKem
                | HpkeKemBinding::BentenSuppliesKemReuseKeySchedule
        ),
        "NQ-C1 resolution must be one of the two named branches"
    );
}

/// F-KAT-3 — the chosen branch's seal output is byte-accurate against its
/// canonical reference vector.
///
/// Drives the PRODUCTION seal (`hpke_seal_to_recipient`) over a fixed
/// fixture + asserts the on-wire bytes equal the reference vector for the
/// RATIFIED branch. would-FAIL-if-no-op'd: the stub seal returns `[]`,
/// which equals neither reference vector. A seal under the wrong KEM or
/// the wrong key-schedule produces bytes ≠ the branch reference.
#[test]
#[ignore = "RED-PHASE: F-KAT-3 — Layer-C HPKE seal byte-accuracy against the ratified NQ-C1 branch reference vector; un-ignore at R5"]
fn hpke_seal_byte_accurate_for_ratified_branch() {
    let resolution =
        NQ_C1_RESOLUTION.expect("NQ-C1 must be ratified (see the binding-ratified test)");

    // Fixed fixture — a stable recipient pubkey + plaintext + AAD so the
    // reference vector is deterministic (encap determinism rides the
    // seal's internal test-seed at R5; the canary co-locates the seed).
    let recipient_pub = [0x42u8; 32];
    let plaintext = b"benten-layer-c-fixture";
    let aad = b"benten-aead:layer-c:nq-c1-fixture";

    let sealed = hpke_seal_to_recipient(&recipient_pub, plaintext, aad);

    let reference = match resolution {
        HpkeKemBinding::Rfc9180FaithfulCustomKem => rfc9180_reference_seal_bytes(),
        HpkeKemBinding::BentenSuppliesKemReuseKeySchedule => {
            benten_canonical_reference_seal_bytes()
        }
    };

    // The reference vector for a real branch is NON-EMPTY (a frozen
    // byte-format has bytes); the stub's empty reference is itself a
    // RED-PHASE signal.
    assert!(
        !reference.is_empty(),
        "the ratified branch MUST pin a non-empty reference seal vector"
    );
    assert_eq!(
        sealed, reference,
        "Layer-C HPKE seal output must be byte-accurate against the ratified NQ-C1 branch reference"
    );
}

/// F-KAT-3 — the two branches are NOT byte-interchangeable.
///
/// Pins that the two resolutions produce DISTINCT canonical byte-formats
/// (RFC-9180-faithful vs Benten-supplies-KEM are not the same bytes) — so
/// "pick a branch" is a real wire-format decision, not cosmetic. This is
/// the would-FAIL guard against an R5 that pins one vector for both
/// branches (which would silently make the fork meaningless).
#[test]
#[ignore = "RED-PHASE: F-KAT-3 — the two NQ-C1 branches must produce DISTINCT canonical byte-formats (the fork is real); un-ignore at R5"]
fn nq_c1_branches_are_byte_distinct() {
    let rfc = rfc9180_reference_seal_bytes();
    let benten = benten_canonical_reference_seal_bytes();
    assert!(
        !rfc.is_empty() && !benten.is_empty(),
        "both branch reference vectors must be pinned non-empty at R5"
    );
    assert_ne!(
        rfc, benten,
        "the RFC-9180-faithful and Benten-supplies-KEM seal byte-formats must be DISTINCT (else the NQ-C1 fork is meaningless)"
    );
}
