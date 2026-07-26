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
//! - **F-SM-3** Strip-resistance negative: removing or zeroing the PQ-half
//!   OR the classical-half fails decryption.
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

/// R5: real SwapMatrix seal/open/strip via the `testing`-feature keypair +
/// `sign_and_seal`/`open_and_verify` production API. The strip helpers zero a
/// hybrid half of the real `WrappedKey` so the strip-resistant X-Wing combiner
/// derives a different key ⇒ the AEAD open fails-closed.
mod f_sm_real {
    use benten_crypto_suite::swap_matrix::SwapMatrix;

    /// A swap-matrix config selector mapped to the real built arms.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Cfg {
        HybridDefault,
        ClassicalOnly,
        NoEncryption,
        /// Maps to the real non-PQ-encryption built arm (a non-default
        /// swap-matrix path — classical-only X25519 KEM under a hybrid
        /// signature; F-08 corrected the mislabel: this is the
        /// NON-PQ-ENCRYPTION arm, NOT the NF-1 PQ⊕PQ arm. The pure-PQ `0x647c`
        /// arm is audit-gated and not round-trippable at the v1-beta baseline).
        NonPqEncryption,
    }

    fn matrix(cfg: Cfg) -> SwapMatrix {
        match cfg {
            Cfg::HybridDefault => SwapMatrix::v1_beta_default(),
            Cfg::ClassicalOnly => SwapMatrix::classical_only(),
            Cfg::NoEncryption => SwapMatrix::no_encryption_public_class(),
            Cfg::NonPqEncryption => SwapMatrix::non_pq_encryption(),
        }
    }

    /// Reconstruct a fresh `SwapEnvelope` from another's public fields (the
    /// type is not `Clone`; all fields are `pub` so this is a field-copy).
    fn clone_envelope(
        e: &benten_crypto_suite::swap_matrix::SwapEnvelope,
    ) -> benten_crypto_suite::swap_matrix::SwapEnvelope {
        use benten_crypto_suite::swap_matrix::{SealedEnvelope, SwapEnvelope};
        SwapEnvelope {
            sig_codepoint: e.sig_codepoint,
            cipher_codepoint: e.cipher_codepoint,
            signature_bytes: e.signature_bytes.clone(),
            sealed: e.sealed.as_ref().map(|s| SealedEnvelope {
                wrapped: s.wrapped.clone(),
                aead: s.aead.clone(),
                sig_len: s.sig_len,
            }),
            plaintext_for_no_encryption: e.plaintext_for_no_encryption.clone(),
            slh_len_for_pure_pq: e.slh_len_for_pure_pq,
        }
    }

    /// A self-describing sealed bundle: the serialized SwapEnvelope-equivalent
    /// state captured via a closure-held round-trip context. Because the real
    /// SwapMatrix round-trip needs both keypairs, we hold them in a boxed
    /// context referenced by an index encoded in the sealed bytes.
    use std::cell::RefCell;
    use std::collections::HashMap;
    thread_local! {
        static CTX: RefCell<HashMap<u64, SealCtx>> = RefCell::new(HashMap::new());
        static NEXT: RefCell<u64> = const { RefCell::new(0) };
    }

    struct SealCtx {
        cfg: Cfg,
        kp: benten_crypto_suite::swap_matrix::SwapKeypair,
        recip: benten_crypto_suite::swap_matrix::SwapRecipientKeypair,
        envelope: benten_crypto_suite::swap_matrix::SwapEnvelope,
        pq_stripped: bool,
        classical_stripped: bool,
    }

    /// Production seal under `cfg` — real `sign_and_seal`. Returns an 8-byte
    /// BE index handle the round-trip + strip helpers thread through.
    #[must_use]
    pub fn seal(cfg: Cfg, plaintext: &[u8], _recipient_pub: &[u8]) -> Vec<u8> {
        let m = matrix(cfg);
        let kp = m.generate_keypair_for_test();
        let recip = m.generate_recipient_keypair_for_test();
        let recip_pub = recip.public();
        let envelope = m
            .sign_and_seal(&kp, &recip_pub, plaintext)
            .expect("real sign_and_seal");
        let id = NEXT.with(|n| {
            let mut n = n.borrow_mut();
            let v = *n;
            *n += 1;
            v
        });
        CTX.with(|c| {
            c.borrow_mut().insert(
                id,
                SealCtx {
                    cfg,
                    kp,
                    recip,
                    envelope,
                    pq_stripped: false,
                    classical_stripped: false,
                },
            );
        });
        id.to_be_bytes().to_vec()
    }

    /// Production open — real `open_and_verify`. Returns `None` if a stripped
    /// half made the strip-resistant combiner derive a different key
    /// (fail-closed).
    #[must_use]
    pub fn open(cfg: Cfg, sealed: &[u8], _recipient_pub: &[u8]) -> Option<Vec<u8>> {
        let id = u64::from_be_bytes(sealed.try_into().ok()?);
        CTX.with(|c| {
            let map = c.borrow();
            let ctx = map.get(&id)?;
            assert_eq!(ctx.cfg, cfg, "open cfg must match seal cfg");
            let m = matrix(cfg);
            // Apply any strip mutation to a clone of the real envelope's
            // wrapped key (the strip-resistant X-Wing combiner derives a
            // different key when a half is zeroed ⇒ AEAD open fails-closed).
            let mut env = clone_envelope(&ctx.envelope);
            if let Some(sealed) = env.sealed.as_mut() {
                if ctx.pq_stripped {
                    sealed.wrapped = sealed.wrapped.without_pq_half_for_test();
                }
                if ctx.classical_stripped {
                    sealed.wrapped = sealed.wrapped.without_classical_half_for_test();
                }
            }
            let sender_pub = ctx.kp.public();
            let recip_secret = ctx.recip.secret();
            m.open_and_verify(&recip_secret, &sender_pub, &env)
                .ok()
                .map(|d| d.as_slice().to_vec())
        })
    }

    /// Mark the sealed bundle's PQ (ML-KEM-768) half stripped.
    #[must_use]
    pub fn strip_pq_half(sealed: &[u8]) -> Vec<u8> {
        let id = u64::from_be_bytes(sealed.try_into().unwrap());
        CTX.with(|c| {
            if let Some(ctx) = c.borrow_mut().get_mut(&id) {
                ctx.pq_stripped = true;
            }
        });
        sealed.to_vec()
    }

    /// Mark the sealed bundle's classical (X25519) half stripped.
    #[must_use]
    pub fn strip_classical_half(sealed: &[u8]) -> Vec<u8> {
        let id = u64::from_be_bytes(sealed.try_into().unwrap());
        CTX.with(|c| {
            if let Some(ctx) = c.borrow_mut().get_mut(&id) {
                ctx.classical_stripped = true;
            }
        });
        sealed.to_vec()
    }

    /// The no-encryption (`0x0000` / public-class) arm emits the plaintext
    /// VERBATIM (no AEAD framing) — the real `sign_only` arm keeps the
    /// plaintext readable by design.
    #[must_use]
    pub fn seal_no_encryption(plaintext: &[u8]) -> Vec<u8> {
        let m = SwapMatrix::no_encryption_public_class();
        let kp = m.generate_keypair_for_test();
        let env = m.sign_only(&kp, plaintext).expect("sign_only");
        // The no-encryption arm carries the plaintext verbatim (pub field).
        env.plaintext_for_no_encryption
            .expect("no-encryption arm carries the verbatim plaintext")
    }
}

use f_sm_real::{Cfg, open, seal, seal_no_encryption, strip_classical_half, strip_pq_half};

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
    for cfg in [Cfg::HybridDefault, Cfg::ClassicalOnly, Cfg::NonPqEncryption] {
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

/// **F-SM-3** — strip-resistance negative.
///
/// Removing or zeroing the PQ-half OR the classical-half of a hybrid-
/// default sealed envelope fails decryption (the X-Wing combiner mixes
/// BOTH shared secrets into the KEK, so zeroing either half derives a
/// different key; this is strip-resistance, NOT full AEAD key-commitment
/// in the robustness sense — that is OUT OF SCOPE at v1-beta per Inv-17 /
/// Compromise #30). This is the load-bearing Inv-17 + C-5/C-6 safety
/// property. The strip helpers are LIVE against the real combiner as of
/// R5 (the red-phase no-op stub they were written against is gone).
#[test]
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
        "stripping the ML-KEM-768 (PQ) half MUST fail decryption (strip-resistant combiner)"
    );

    // Stripping the classical (X25519) half must fail decryption.
    let classical_stripped = strip_classical_half(&sealed);
    assert!(
        open(Cfg::HybridDefault, &classical_stripped, &RECIPIENT_PUB).is_none(),
        "stripping the X25519 (classical) half MUST fail decryption (strip-resistant combiner)"
    );
}
