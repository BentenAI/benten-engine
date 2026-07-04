//! **F-KAT-3 — NQ-C1 HPKE KEM-extensibility byte-accuracy (P0 PREREQUISITE).**
//!
//! ADDL R3 wave **W0-crypto-canary** (the SOLE upstream canary). Pin
//! source: `f-full-r2-test-landscape.md` Group-4 F-KAT-3 + §4 P0-1 +
//! §10.6 NQ-C1 + R0 §2.2 (tactical-picks: McMillion `hpke`, NOT Cryspen
//! `hpke-rs`) + §3.3 (Layer-C HpkeBase[MLKEM768-X25519]) + §11-1
//! (one of the 3 items R0 could NOT close in-plan).
//!
//! # NQ-C1 RATIFIED = Branch B (Benten-supplies-KEM), Ben 2026-06-05
//!
//! The wire-format fork this file pinned as a QUESTION is now RESOLVED.
//! Ben ratified NQ-C1 as **Branch B (Benten-supplies-the-KEM)** on
//! 2026-06-05: Layer-C's on-wire HPKE framing uses **Benten-canonical
//! ML-KEM-768 + X25519 encapsulation bytes** (Benten supplies the KEM)
//! and reuses only the HPKE key-schedule shape — it is intentionally
//! **NOT** RFC-9180 cross-stack-interoperable. That is consistent with
//! the already-ratified §6.2 Option-F+ NO-GO Benten-specific-envelope
//! design (the unification lives at the envelope / codepoint-dispatch
//! layer, NOT the primitive layer; see `crate::envelope`). An
//! RFC-9180-faithful suite (Branch A) is a **FUTURE ADDITIVE codepoint**
//! under crypto-agility (CLAUDE.md baked-in #5), never a wire break.
//!
//! ## Why Branch B is the ground-truth at HEAD (the structural evidence)
//!
//! The LIVE Layer-C single-recipient seal is
//! [`benten_crypto_suite::hpke::wrap_key_to_recipient`] (which dispatches
//! to [`benten_crypto_suite::cipher_suite::CipherSuite::wrap_key_material`]
//! at codepoint `0x647a`). Its on-wire [`WrappedKey`] carries the
//! encapsulation as TWO Benten-canonical fields:
//!   - `ek_x`     — a raw 32-byte X25519 ephemeral public key, AND
//!   - `ek_mlkem` — a raw **1088-byte** FIPS-203 ML-KEM-768 ciphertext.
//!
//! That two-field raw-FIPS-203 layout is the Branch-B (Benten-supplies-KEM)
//! signature. An RFC-9180-faithful stack (Branch A) would emit a SINGLE
//! opaque `enc` value produced by a registered KEM's `Encap`, NOT two
//! separately-laid-out raw component ciphertexts — so the on-wire bytes
//! here are deliberately NOT cross-stack-interop. The key-derivation reuses
//! the HPKE-shaped key-schedule via the draft-connolly X-Wing combiner
//! `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)` (label APPENDED).
//!
//! # Would-FAIL-if-no-op'd (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! The pins drive a PRODUCTION call site
//! ([`benten_crypto_suite::hpke::wrap_key_to_recipient`] — the Layer-C
//! single-recipient KEM-DEM seal) + assert OBSERVABLE consequences (the
//! sealed key round-trips back to the input through the recipient secret;
//! the on-wire encapsulation has the Branch-B canonical byte-shape; the
//! draft-connolly combiner golden is byte-accurate; a WRONG recipient
//! secret fails closed). Each is would-FAIL-if-no-op'd: a stub seal that
//! returned the plaintext, a seal under the wrong KEM/key-schedule, or a
//! recipient-key-independent unwrap all produce bytes ≠ the ratified
//! Branch-B vector or break the round-trip / negative pins.
//!
//! # Golden provenance (M-20 golden-via-throwaway-compute)
//!
//! The combiner golden below was computed by RUNNING the live
//! [`combine_x_wing`] over the fixed input tuple
//! `(ss_M=[0x01;32], ss_X=[0x02;32], ct_X=[0x03;32], pk_X=[0x04;32])` and
//! independently cross-verified with a standalone SHA3-256 of the appended-
//! label preimage (the prepended-label order — the superseded v01-v02
//! ordering — yields a DIFFERENT digest, so the appended construction is
//! load-bearing). The structural lengths (`ek_x`=32, `ek_mlkem`=1088) were
//! observed from the live seal output. No byte was fabricated or weakened.

use benten_crypto_suite::cipher_suite::{CipherSuite, WrappedKey, combine_x_wing};
use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_crypto_suite::hpke::{unwrap_key_from_recipient, wrap_key_to_recipient};

/// The two mutually-exclusive resolutions of NQ-C1.
///
/// Ben ratified EXACTLY ONE (2026-06-05). The chosen branch determines the
/// canonical reference vector the Layer-C bytes are pinned against. The
/// enum is retained as the totality witness so the meta-assertion below can
/// still forbid the no-op "both branches / neither pinned" shape — now
/// resolved to [`HpkeKemBinding::BentenSuppliesKemReuseKeySchedule`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpkeKemBinding {
    /// Branch A — an RFC-9180-faithful suite that admits X25519MLKEM768 into
    /// a real `mode_base` context byte-accurately; the on-wire bytes interop
    /// with any RFC-9180 HPKE stack. NOT the ratified branch — reserved as a
    /// FUTURE ADDITIVE codepoint under crypto-agility (#5).
    Rfc9180FaithfulCustomKem,
    /// Branch B — **RATIFIED (Ben 2026-06-05).** Benten supplies the
    /// X25519⊕ML-KEM-768 KEM encapsulation (raw 32-byte X25519 ephemeral +
    /// raw 1088-byte FIPS-203 ML-KEM-768 ciphertext) + reuses ONLY the HPKE-
    /// shaped key-schedule; the KEM-encapsulation bytes are Benten-canonical
    /// (NOT cross-stack-interop), the combiner is RFC-9180/draft-connolly
    /// shaped.
    BentenSuppliesKemReuseKeySchedule,
}

/// The RATIFIED resolution of NQ-C1 — **Branch B** (Benten-supplies-KEM),
/// ratified by Ben 2026-06-05. Was `None` (UNRESOLVED) at the RED-PHASE
/// baseline; resolved here.
pub const NQ_C1_RESOLUTION: Option<HpkeKemBinding> =
    Some(HpkeKemBinding::BentenSuppliesKemReuseKeySchedule);

/// Deterministic recipient seed for the fixed Branch-B fixture — a stable
/// 32-byte seed fed to [`CipherSuite::generate_recipient_keypair_deterministic_for_test`]
/// so the recipient identity is reproducible without a keystore round-trip.
const FIXTURE_RECIPIENT_SEED: [u8; 32] = [0x42u8; 32];

/// A fixed `k_root` the Layer-C seal wraps (the small `K(N)` the
/// key-encryption mode transports). 32 B = the ChaCha20-Poly1305 key width.
const FIXTURE_K_ROOT: [u8; 32] = [0x11u8; 32];

/// The Branch-B X-Wing combiner GOLDEN — the byte-accurate output of the
/// LIVE [`combine_x_wing`] over the fixed input tuple
/// `(ss_M=[0x01;32], ss_X=[0x02;32], ct_X=[0x03;32], pk_X=[0x04;32])` =
/// `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)` with the 6-byte
/// `XWingLabel` (`0x5c2e2f2f5e5c`) **APPENDED** as the trailing suffix
/// (draft-connolly-cfrg-xwing-kem-10 §5.3 "Combiner"). Computed by M-20 throwaway-compute
/// + independently cross-checked against a standalone SHA3-256. This is the
/// Benten-canonical key-derivation that Branch B's on-wire bytes commit to.
const BRANCH_B_COMBINER_GOLDEN: [u8; 32] = [
    0x5c, 0x6b, 0xfa, 0xf8, 0xc3, 0xec, 0x48, 0xab, 0x3c, 0xee, 0x7c, 0x12, 0x12, 0x9b, 0x39, 0x91,
    0x3b, 0x8a, 0x7f, 0xa1, 0x23, 0x41, 0x15, 0xda, 0x7e, 0x1c, 0x55, 0x60, 0x8a, 0xd1, 0x9f, 0xb6,
];

/// Branch-B canonical encapsulation byte-shape: the raw X25519 ephemeral
/// public key is 32 bytes.
const BRANCH_B_EK_X_LEN: usize = 32;

/// Branch-B canonical encapsulation byte-shape: the raw FIPS-203 ML-KEM-768
/// ciphertext is 1088 bytes (FIPS 203 ML-KEM-768 `c` length). This raw-FIPS
/// layout — distinct from an RFC-9180 opaque `enc` — is the Branch-B
/// (Benten-supplies-KEM) signature.
const BRANCH_B_EK_MLKEM_LEN: usize = 1088;

/// Seal `FIXTURE_K_ROOT` to the deterministic fixture recipient via the LIVE
/// Layer-C single-recipient production seal. Returns the recipient keypair +
/// the on-wire [`WrappedKey`] so a test can both inspect the canonical bytes
/// and round-trip-open it.
fn seal_fixture() -> (
    benten_crypto_suite::cipher_suite::RecipientKeypair,
    WrappedKey,
) {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("0x647a hybrid is LIVE");
    let kp = suite.generate_recipient_keypair_deterministic_for_test(&FIXTURE_RECIPIENT_SEED);
    // PRODUCTION call site: the Layer-C single-recipient KEM-DEM seal.
    let wrapped =
        wrap_key_to_recipient(kp.public(), &FIXTURE_K_ROOT).expect("Layer-C seal MUST succeed");
    (kp, wrapped)
}

/// F-KAT-3 / NQ-C1 — the decision-fork is RATIFIED (exactly one branch).
///
/// The meta-assertion that forbids the no-op shape: `NQ_C1_RESOLUTION` MUST
/// be `Some(_)` (the fork was decided, not left open) — a frozen Layer-C
/// byte-format with an UNRESOLVED KEM binding would be a freeze-gating
/// failure. Resolves to **Branch B** per the 2026-06-05 ratification.
#[test]
fn nq_c1_kem_binding_is_ratified_exactly_one_branch() {
    let resolution = NQ_C1_RESOLUTION
        .expect("NQ-C1 is RATIFIED (Branch B) — must be Some before Layer-C bytes freeze");
    // Exactly one of the two branches — the enum is the totality witness.
    assert!(
        matches!(
            resolution,
            HpkeKemBinding::Rfc9180FaithfulCustomKem
                | HpkeKemBinding::BentenSuppliesKemReuseKeySchedule
        ),
        "NQ-C1 resolution must be one of the two named branches"
    );
    // The ratified branch is specifically Branch B (Benten-supplies-KEM).
    assert_eq!(
        resolution,
        HpkeKemBinding::BentenSuppliesKemReuseKeySchedule,
        "NQ-C1 RATIFIED = Branch B (Benten-supplies-the-KEM), Ben 2026-06-05 — NOT Branch A \
         (RFC-9180-faithful cross-stack), which is a future-additive codepoint under #5"
    );
}

/// F-KAT-3 — the ratified Branch-B seal is byte-accurate against its
/// canonical reference vector + round-trips through the recipient secret.
///
/// Drives the PRODUCTION seal ([`wrap_key_to_recipient`]) over the fixed
/// fixture. The encapsulation fields (`ek_x`, `ek_mlkem`, nonce, ciphertext)
/// ride a fresh ephemeral per call so they are NOT fixed-byte-pinnable; the
/// **deterministic** Branch-B witnesses are pinned instead:
///   1. the on-wire codepoint discriminator is the Benten-canonical `0x647a`;
///   2. the encapsulation byte-shape is the Branch-B raw-FIPS layout
///      (`ek_x`=32 B X25519 ephemeral + `ek_mlkem`=1088 B FIPS-203 ML-KEM-768
///      ciphertext) — NOT an RFC-9180 opaque `enc` (the Branch-A signature);
///   3. the draft-connolly X-Wing combiner golden is byte-accurate (the
///      key-schedule Branch B reuses);
///   4. the seal round-trips: unwrap under the recipient secret recovers
///      `FIXTURE_K_ROOT` exactly (the observable consequence).
///
/// would-FAIL-if-no-op'd: a stub seal returning `[]` / the plaintext breaks
/// the round-trip; a seal under the wrong KEM produces a different encap
/// shape; a seal under the wrong key-schedule (e.g. prepended XWingLabel)
/// changes the combiner golden.
#[test]
fn hpke_seal_byte_accurate_for_ratified_branch() {
    let resolution =
        NQ_C1_RESOLUTION.expect("NQ-C1 is RATIFIED (Branch B; see the binding-ratified test)");
    assert_eq!(
        resolution,
        HpkeKemBinding::BentenSuppliesKemReuseKeySchedule,
        "this byte-accuracy pin is for the RATIFIED Branch B"
    );

    let (kp, wrapped) = seal_fixture();

    // (1) On-wire codepoint discriminator = Benten-canonical hybrid `0x647a`.
    assert_eq!(
        wrapped.codepoint.raw(),
        0x647a,
        "Branch-B Layer-C seal MUST carry the Benten-canonical hybrid codepoint 0x647a"
    );

    // (2) Branch-B canonical encapsulation byte-shape (the raw-FIPS layout
    // that is NOT RFC-9180 `enc` — the Benten-supplies-KEM signature).
    assert_eq!(
        wrapped.ek_x.len(),
        BRANCH_B_EK_X_LEN,
        "Branch-B ek_x MUST be a raw 32-byte X25519 ephemeral public key"
    );
    assert_eq!(
        wrapped.ek_mlkem.len(),
        BRANCH_B_EK_MLKEM_LEN,
        "Branch-B ek_mlkem MUST be a raw 1088-byte FIPS-203 ML-KEM-768 ciphertext (NOT an \
         RFC-9180 opaque enc) — this raw two-field layout is the Benten-supplies-KEM signature"
    );

    // (3) The draft-connolly X-Wing combiner GOLDEN is byte-accurate (the
    // key-schedule Branch B reuses). Independently cross-checked vs SHA3-256.
    let combiner_out = combine_x_wing(&[0x01u8; 32], &[0x02u8; 32], &[0x03u8; 32], &[0x04u8; 32]);
    assert_eq!(
        combiner_out, BRANCH_B_COMBINER_GOLDEN,
        "Branch-B X-Wing combiner output MUST be byte-accurate against the ratified golden \
         (SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel), label APPENDED)"
    );

    // (4) The OBSERVABLE consequence: the seal round-trips through the
    // recipient secret — unwrap recovers FIXTURE_K_ROOT exactly. A stub seal
    // (empty / plaintext) breaks this.
    let recovered =
        unwrap_key_from_recipient(kp.secret(), &wrapped).expect("Layer-C unwrap MUST succeed");
    assert_eq!(
        recovered.as_slice(),
        FIXTURE_K_ROOT.as_slice(),
        "Branch-B Layer-C seal MUST round-trip: unwrap under the recipient secret recovers \
         the wrapped k_root exactly"
    );
}

/// F-KAT-3 — Branch B is NOT byte-interchangeable with Branch A, AND the
/// recipient binding is real (wrong secret fails closed).
///
/// Pins that the ratified branch is a REAL wire-format decision, not
/// cosmetic:
///   (a) the Branch-B encapsulation is the raw-FIPS two-field layout
///       (`ek_x`=32 + `ek_mlkem`=1088 = 1120 raw encap bytes), which is
///       structurally distinct from an RFC-9180 single-`enc` framing — so
///       "pick Branch B over Branch A" changes the on-wire bytes; AND
///   (b) the recipient binding is real: unwrapping with the WRONG recipient
///       secret fails closed (the X-Wing combiner mixes both KEM halves +
///       the recipient pubkey, so a wrong secret derives a wrong key → the
///       ChaCha20-Poly1305 open fails).
///
/// would-FAIL-if-no-op'd: an R5 that pinned identical bytes for both branches
/// (making the fork meaningless) would not exhibit the raw two-field layout;
/// a recipient-key-independent unwrap would let a wrong secret recover k_root.
#[test]
fn nq_c1_branch_b_distinct_and_recipient_bound() {
    let (kp, wrapped) = seal_fixture();

    // (a) Branch-B raw two-field encapsulation distinguishes it from a
    // Branch-A single-`enc` RFC-9180 framing. The raw encap byte budget is
    // ek_x (32) + ek_mlkem (1088) = 1120, laid out as two separate fields.
    let branch_b_raw_encap_len = wrapped.ek_x.len() + wrapped.ek_mlkem.len();
    assert_eq!(
        branch_b_raw_encap_len,
        BRANCH_B_EK_X_LEN + BRANCH_B_EK_MLKEM_LEN,
        "Branch-B encapsulation is the raw two-field layout (32 + 1088 = 1120 bytes), \
         structurally distinct from a Branch-A RFC-9180 single-enc framing — the fork is real"
    );
    assert!(
        !wrapped.ek_mlkem.is_empty(),
        "the hybrid Branch-B seal MUST carry a non-empty ML-KEM-768 ciphertext half (a \
         classical-only / RFC-9180-collapsed framing would not)"
    );

    // (b) The recipient binding is real — a WRONG recipient secret fails
    // closed (would-FAIL on a key-independent unwrap).
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
    let attacker_kp = suite.generate_recipient_keypair_deterministic_for_test(&[0xEEu8; 32]);
    let outcome = unwrap_key_from_recipient(attacker_kp.secret(), &wrapped);
    assert!(
        outcome.is_err(),
        "unwrapping the Branch-B seal with the WRONG recipient secret MUST fail closed \
         (the X-Wing recipient binding is real); got {outcome:?}"
    );

    // Sanity: the legitimate recipient still opens it (the negative pin above
    // is a real fail-closed, not a universally-broken unwrap).
    let legit = unwrap_key_from_recipient(kp.secret(), &wrapped);
    assert!(
        legit.is_ok(),
        "the legitimate recipient MUST still open the Branch-B seal"
    );
}
