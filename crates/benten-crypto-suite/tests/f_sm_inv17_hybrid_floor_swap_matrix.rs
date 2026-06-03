//! **F-SM-1..3 — Inv-17 hybrid floor + full bidirectional swap matrix (ALL FG).**
//!
//! ADDL R3 wave **W0-crypto-canary**. Pin source:
//! `f-full-r2-test-landscape.md` Group-3 (F-SM-1..3) + R0 plan §2.1 C-8
//! (Inv-17 hybrid-mandatory floor) + §5.1 Inv-17 + §2.1 C-5/C-7 +
//! G-CORE-3c full-swap-matrix + Compromise #30 (unaudited-PQ
//! MITIGATED-by-hybrid).
//!
//! # What these families pin
//!
//! - **F-SM-1** Hybrid-mandatory floor: every KEM use-site is PQ⊕classical;
//!   `0x647c` PURE_PQ reachable ONLY via the gated
//!   `try_pure_pq_sole_trust_path` (audit-flag); typed-rejected until the
//!   flag flips; `0x647c ≠ 0x647b`.
//! - **F-SM-2** Full bidirectional cipher swap matrix: hybrid-default +
//!   classical `0x6400` + no-encryption + NF-1 PQ⊕PQ `0x647b` arm, each a
//!   real built path, BOTH directions (`open(seal(pt,cfg),cfg)==pt`).
//! - **F-SM-3** Strip-resistance / committing-combiner negative: removing
//!   or zeroing the PQ-half OR the classical-half fails decryption.
//!
//! # Ground-truth at HEAD (`sed`-verified `swap_matrix.rs`)
//!
//! LIVE non-feature-gated query surface used here directly:
//! `SwapMatrix::{v1_beta_default, classical_only, no_encryption_public_class,
//! non_pq_encryption, try_pure_pq_sole_trust_path}` constructors +
//! `{encryption_is_pq_hybrid, encryption_active, is_pure_pq_sole_trust_path,
//! cipher_suite_codepoint, audit_landed_pure_pq_flag}` queries +
//! `AUDIT_LANDED_PURE_PQ_FLAG = false`. The `*_for_test` keypair / seal /
//! open helpers are `#[cfg(feature = "testing")]`-gated; to keep this W0
//! family self-contained + feature-independent (parallel safety), the
//! round-trip + strip-resistance pins use a SELF-CONTAINED `f_sm_stub`
//! seal/open shim.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! Tests over the LIVE constructor/query surface use the REAL `SwapMatrix`.
//! The round-trip + strip pins use a `f_sm_stub` seal/open shim. R5 MUST:
//!   1. DELETE the `f_sm_stub` module,
//!   2. INSERT real `SwapMatrix` seal/open (via the `testing` feature or
//!      the production seal API the canary mints),
//!   3. UN-IGNORE + verify all PASS green (the real round-trip recovers pt;
//!      the real strip arms fail-closed).

#![allow(dead_code)]

use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_crypto_suite::swap_matrix::SwapMatrix;

/// SELF-CONTAINED stub-shim for the round-trip + strip-resistance pins.
mod f_sm_stub {
    /// A swap-matrix config selector (mirrors the real arms, feature-free).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Cfg {
        HybridDefault,
        ClassicalOnly,
        NoEncryption,
        Nf1PqPq,
    }

    /// Production seal under `cfg`. STUB returns the plaintext UNCHANGED
    /// (deliberately NOT a real seal) so the round-trip pin's
    /// `open(seal)==pt` still holds for the stub, BUT the strip-resistance
    /// pins (which require a real committing combiner) FAIL until R5 wires
    /// the real seal. The would-FAIL is carried by the strip arms.
    pub fn seal(_cfg: Cfg, plaintext: &[u8], recipient_pub: &[u8]) -> Vec<u8> {
        // STUB: a trivial reversible transform tagging the recipient so
        // open() can recover pt; R5 replaces with the real AEAD seal.
        let mut out = recipient_pub.to_vec();
        out.extend_from_slice(plaintext);
        out
    }

    /// Production open under `cfg`. STUB reverses `seal`. R5 replaces with
    /// the real AEAD open (which fails-closed on a stripped half).
    pub fn open(_cfg: Cfg, sealed: &[u8], recipient_pub: &[u8]) -> Option<Vec<u8>> {
        if sealed.len() < recipient_pub.len() || &sealed[..recipient_pub.len()] != recipient_pub {
            return None;
        }
        Some(sealed[recipient_pub.len()..].to_vec())
    }

    /// Zero the ML-KEM (PQ) half of a sealed envelope's encapsulated key.
    /// STUB returns the input UNCHANGED so the strip-PQ negative pin FAILS
    /// red (the stub open still succeeds) until R5 wires the real
    /// committing combiner (where zeroing the PQ half changes the derived
    /// key ⇒ open fails-closed).
    pub fn strip_pq_half(sealed: &[u8]) -> Vec<u8> {
        sealed.to_vec()
    }

    /// Zero the X25519 (classical) half. STUB returns input unchanged
    /// (same red-phase reasoning).
    pub fn strip_classical_half(sealed: &[u8]) -> Vec<u8> {
        sealed.to_vec()
    }

    /// Produce the output of the **no-encryption** (`0x0000` /
    /// `no_encryption_public_class`) swap-matrix arm. The security-relevant
    /// property (CC-MAJ-SM2): selecting no-encryption MUST emit the
    /// plaintext VERBATIM (the public-class deployment posture — the data
    /// IS readable, by design; it must NOT be silently encrypted, and must
    /// NOT be refused). STUB deliberately does NOT emit the plaintext
    /// verbatim (it wraps the plaintext with a sentinel byte, modelling a
    /// "wrong-but-plausible" impl that default-encrypts-and-discards-key or
    /// mis-wires the arm) so the verbatim-plaintext pin FAILS red until R5
    /// wires the real no-encryption path.
    pub fn seal_no_encryption(plaintext: &[u8]) -> Vec<u8> {
        // STUB BUG (intentional): NOT verbatim — prepends a sentinel so the
        // output != plaintext. R5's real no-encryption arm returns the
        // plaintext unchanged.
        let mut out = vec![0xFFu8];
        out.extend_from_slice(plaintext);
        out
    }
}

use f_sm_stub::{Cfg, open, seal, seal_no_encryption, strip_classical_half, strip_pq_half};

const RECIPIENT_PUB: [u8; 32] = [0x42; 32];
const PLAINTEXT: &[u8] = b"benten-swap-matrix-fixture";

/// **F-SM-1** — Inv-17 hybrid-mandatory floor: no pure-PQ LIVE or
/// selectable at the v1-beta default. **ALREADY-LIVE REGRESSION-GUARD**
/// (F4-017): this arm exercises ONLY the LIVE in-tree `SwapMatrix` +
/// `CipherSuiteCodepoint` surface (no stub, no V2-corpus dependency), so
/// it asserts an already-shipped fact and is NOT a red-phase pin — it has
/// no would-FAIL-vs-stub arm because there is no stub. Per the R4 triage
/// (F4-017: "add a would-FAIL arm OR re-scope as an already-LIVE
/// regression-guard — un-ignore + label"), it is re-scoped as a
/// regression-guard and un-ignored: it guards against a future regression
/// that (a) flips the v1-beta default off PQ-hybrid, (b) silently opens
/// the pure-PQ audit gate, or (c) collides `0x647c`/`0x647b`.
///
/// (a) the v1-beta default's encryption half IS PQ-hybrid + the classical
/// half is present (`encryption_active`); (b) the pure-PQ arm is NOT
/// reachable at the workspace baseline (`AUDIT_LANDED_PURE_PQ_FLAG ==
/// false` ⇒ `try_pure_pq_sole_trust_path` errors); (c) `0x647c ≠ 0x647b`.
/// would-FAIL-on-regression: if the audit flag were silently `true`, the
/// pure-PQ constructor would succeed (the floor would be breached) — this
/// guard fires.
#[test]
fn inv17_hybrid_floor_no_pure_pq_live() {
    let default = SwapMatrix::v1_beta_default();
    assert!(
        default.encryption_is_pq_hybrid(),
        "the v1-beta default encryption half MUST be PQ-hybrid (Inv-17 floor)"
    );
    assert!(
        default.encryption_active(),
        "the v1-beta default has an active (classical-floor-present) encryption half"
    );
    assert!(
        !default.is_pure_pq_sole_trust_path(),
        "the v1-beta default is NOT the pure-PQ sole-trust-path arm (the classical half is the audited floor)"
    );

    // The pure-PQ arm is audit-gated: at workspace baseline the flag is
    // false ⇒ the constructor MUST reject. The OBSERVABLE consequence is
    // that `try_pure_pq_sole_trust_path()` errors — driving the real
    // gated constructor (not asserting on the bare const, which is the
    // would-FAIL-if-no-op'd safety property: a silently-true flag would
    // let pure-PQ become the sole trust path).
    assert!(
        SwapMatrix::try_pure_pq_sole_trust_path().is_err(),
        "pure-PQ-sole-trust-path MUST be unreachable until the independent audit lands (Inv-17; #30)"
    );
    // The accessor reports the gate is closed (runtime accessor, not the
    // bare module const).
    assert!(
        !SwapMatrix::audit_landed_pure_pq_flag(),
        "v1-beta workspace baseline: the pure-PQ audit-landed flag is closed (false)"
    );

    // 0x647c (pure-PQ swap-matrix arm) is distinct from 0x647b (NF-1 ML-KEM⊕HQC).
    assert_ne!(
        CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY.raw(),
        CipherSuiteCodepoint::HYBRID_MLKEM768_HQC.raw(),
        "0x647c (pure-PQ) and 0x647b (NF-1 PQ⊕PQ) must be distinct codepoints"
    );
}

/// **F-SM-2** — full bidirectional cipher swap matrix.
///
/// Each config (hybrid-default + classical `0x6400` + no-encryption +
/// NF-1 PQ⊕PQ `0x647b`) is a real built path: `open(seal(pt,cfg),cfg)==pt`
/// in BOTH directions, and each arm dispatches its distinct codepoint.
/// would-FAIL-if-no-op'd: a config whose `cipher_suite_codepoint()` did
/// not match its arm, or whose round-trip dropped the plaintext, fails.
///
/// **CC-MAJ-SM2 (no-encryption arm; minted at R4):** the no-encryption
/// (`0x0000` / `no_encryption_public_class`) arm previously had ONLY a
/// codepoint pin ("no ciphertext to round-trip; its codepoint pin
/// suffices") — its security-relevant property was untested. The
/// public-class posture is that the data IS readable by design, so
/// selecting no-encryption MUST emit the **plaintext VERBATIM** (not
/// silently encrypted, not refused). The new arm pins exactly that; the
/// stub `seal_no_encryption` deliberately does NOT emit verbatim (it wraps
/// with a sentinel — a wrong-but-plausible default-encrypt mis-wire) so
/// the pin fires RED until R5 wires the real no-encryption path.
#[test]
#[ignore = "RED-PHASE: F-SM-2 — full bidirectional cipher swap matrix (each arm a real path) + CC-MAJ-SM2 no-encryption emits plaintext verbatim; un-ignore at R5"]
fn full_bidirectional_cipher_swap_matrix() {
    // Each arm dispatches its expected codepoint (the swap-matrix axis is real).
    assert_eq!(
        SwapMatrix::v1_beta_default().cipher_suite_codepoint().raw(),
        0x647a,
        "hybrid-default arm dispatches 0x647a"
    );
    assert_eq!(
        SwapMatrix::classical_only().cipher_suite_codepoint().raw(),
        0x6400,
        "classical-only arm dispatches 0x6400"
    );
    assert_eq!(
        SwapMatrix::no_encryption_public_class()
            .cipher_suite_codepoint()
            .raw(),
        0x0000,
        "no-encryption arm dispatches 0x0000 (NONE_PLAINTEXT)"
    );

    // Bidirectional round-trip for each ENCRYPTING arm.
    for cfg in [Cfg::HybridDefault, Cfg::ClassicalOnly, Cfg::Nf1PqPq] {
        let sealed = seal(cfg, PLAINTEXT, &RECIPIENT_PUB);
        let opened = open(cfg, &sealed, &RECIPIENT_PUB)
            .unwrap_or_else(|| panic!("open(seal(pt,{cfg:?})) must recover plaintext"));
        assert_eq!(
            opened, PLAINTEXT,
            "swap-matrix arm {cfg:?} must round-trip plaintext in both directions"
        );
    }

    // CC-MAJ-SM2: the no-encryption (0x0000) arm emits the plaintext
    // VERBATIM (public-class posture: the bytes ARE readable, by design).
    // would-FAIL-if-no-op'd: the stub prepends a sentinel (a wrong-but-
    // plausible "default-encrypt-and-discard-key" mis-wire), so the
    // verbatim assertion fires RED until R5 wires the real arm.
    let no_enc_out = seal_no_encryption(PLAINTEXT);
    assert_eq!(
        no_enc_out, PLAINTEXT,
        "the no-encryption arm (0x0000 / public-class) MUST emit the plaintext VERBATIM \
         (not silently encrypted, not refused); would-FAIL while the stub wraps it"
    );
    // The output is the plaintext with NO added framing — it is readable
    // without any recipient key material (the public-class confidentiality
    // posture: data IS readable by design).
    assert_eq!(
        no_enc_out.len(),
        PLAINTEXT.len(),
        "the no-encryption arm output MUST carry NO AEAD tag / key framing (length == plaintext length); \
         would-FAIL while the stub adds a sentinel byte"
    );
}

/// **F-SM-3** — strip-resistance / committing-combiner negative.
///
/// Removing or zeroing the PQ-half OR the classical-half of a hybrid-
/// default sealed envelope fails decryption (the X-Wing combiner is
/// committing across BOTH halves). This is the load-bearing Inv-17 +
/// C-5/C-6 safety property. would-FAIL-if-no-op'd: at the stub the strip
/// helpers no-op, so `open` still succeeds — the assertions FAIL red
/// until R5 wires the real committing combiner where the stripped open
/// fails-closed.
#[test]
#[ignore = "RED-PHASE: F-SM-3 — strip-resistance (stripping PQ-half OR classical-half fails decryption); un-ignore at R5"]
fn strip_resistance_committing_combiner_negative() {
    let sealed = seal(Cfg::HybridDefault, PLAINTEXT, &RECIPIENT_PUB);
    // Positive control: an intact envelope opens.
    assert_eq!(
        open(Cfg::HybridDefault, &sealed, &RECIPIENT_PUB).as_deref(),
        Some(PLAINTEXT),
        "intact hybrid envelope opens (positive control)"
    );

    // Stripping the PQ (ML-KEM-768) half must fail decryption.
    let pq_stripped = strip_pq_half(&sealed);
    assert!(
        open(Cfg::HybridDefault, &pq_stripped, &RECIPIENT_PUB).is_none(),
        "stripping the ML-KEM-768 (PQ) half MUST fail decryption (committing combiner)"
    );

    // Stripping the classical (X25519) half must fail decryption.
    let classical_stripped = strip_classical_half(&sealed);
    assert!(
        open(Cfg::HybridDefault, &classical_stripped, &RECIPIENT_PUB).is_none(),
        "stripping the X25519 (classical) half MUST fail decryption (committing combiner)"
    );
}
