//! Cipher-suite codepoint dispatch + X-Wing-style hybrid wrap surface
//! (G-CORE-3a CANARY — #1301 substrate).
//!
//! # G-CORE-3a deliverables (this wave)
//!
//! - LIVE codepoint `0x647a` = X25519⊕ML-KEM-768 hybrid KEM
//!   (vendored ~30-LOC X-Wing-style combiner over `libcrux-ml-kem` (via
//!   `crate::mlkem`) + `x25519-dalek` + `sha3`; the RustCrypto `ml-kem` crate
//!   is the dev-only FIPS-203 KAT witness, NOT the production impl; per Spike I
//!   + CLAUDE.md baked-in #5 + RATIFIED-S&C §1 refinement #6).
//! - LIVE codepoint `0x6400` = classical-only X25519 KEM (non-default
//!   downgrade arm of the swap matrix).
//! - Typed-reject (NEVER silent-fallback) on every other codepoint —
//!   `0x647b` (NF-1 ML-KEM⊕HQC PQ⊕PQ end-state) + `0x647c` (pure-PQ
//!   ML-KEM-768-only swap-matrix arm; reachable ONLY via the named
//!   [`crate::swap_matrix::SwapMatrix::try_pure_pq_sole_trust_path`]
//!   constructor which gates on `AUDIT_LANDED_PURE_PQ_FLAG`)
//!   + `0x0000` (no-encryption) + any unknown —
//!   surfaces `UnsupportedAlgorithm::CipherSuite`.
//! - [`CipherSuite::wrap_key_material`] / [`CipherSuite::unwrap_key_material`] —
//!   the X-Wing-hybrid wrap/unwrap production API.
//! - [`CipherSuite::seal_aead`] / [`CipherSuite::open_aead`] — the
//!   ChaCha20-Poly1305 production API over a [`crate::aead::AeadKeyMaterial`].
//!
//! # The real X-Wing combiner (BR-3 / R4.2-corrected — `draft-connolly-cfrg-xwing-kem-10`)
//!
//! The hybrid `0x647a` KEM uses the **IETF-faithful X-Wing combiner**, NOT
//! a Benten-private HKDF stand-in. The combiner hashes the two shared
//! secrets, the X25519 ciphertext, and the X25519 public key with the
//! `XWingLabel` **APPENDED** as the trailing suffix:
//!
//! ```text
//! combined = SHA3-256( ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel )
//!   where  ss_M       = ML-KEM-768 shared secret (Encap/Decap)
//!          ss_X       = X25519 shared secret
//!          ct_X       = X25519 ciphertext (the ephemeral encapsulation pubkey)
//!          pk_X       = recipient X25519 public key
//!          XWingLabel = 0x5c2e2f2f5e5c  (ASCII `\.//^\`, the 6-byte
//!                       draft-connolly §5.3 label — APPENDED, not prepended)
//!
//! WRAP(recipient_pub):
//!     (ct_x, ss_x)     = X25519.encapsulate(recipient_pub.x25519)
//!     (ct_mlkem, ss_m) = MLKEM768.encapsulate(recipient_pub.mlkem)
//!     combined = SHA3-256(ss_m ‖ ss_x ‖ ct_x ‖ recipient_pub.x25519 ‖ XWingLabel)
//!     wrapped  = { codepoint: 0x647a (BE on the wire), ct_x, ct_mlkem, encrypted_key }
//!
//! UNWRAP(recipient_sec, wrapped):
//!     ss_x = X25519.decapsulate(recipient_sec.x25519, wrapped.ct_x)
//!     ss_m = MLKEM768.decapsulate(recipient_sec.mlkem, wrapped.ct_mlkem)
//!     combined = SHA3-256(ss_m ‖ ss_x ‖ ct_x ‖ recipient_pub.x25519 ‖ XWingLabel)
//!     decrypt_key = combined
//! ```
//!
//! See [`combine_x_wing`] / [`x_wing_combiner_preimage`] for the exact
//! construction (the latter is exposed so the F-W0-1-LABEL construction-order
//! witness pin can assert the label is the appended suffix, not a prepended
//! prefix). The prior `HKDF-SHA256(… info = "x-wing-v1-benten-0x647a")`
//! stand-in (the corpus base) is **superseded by BR-3** — it was a
//! Benten-private mislabel at the IETF-reserved `0x647A` codepoint and would
//! have frozen a non-interoperable KEM; the real construction change
//! regenerated all golden/KAT vectors.
//!
//! Strip-resistance: the combiner input concatenates both shared secrets +
//! the X25519 ciphertext + the X25519 public key, so the derive commits to
//! both halves. Stripping the ML-KEM-768 half (substituting a zero/forged
//! `ss_M`) yields a different `combined` → the unwrap derives a different
//! key → the subsequent ChaCha20-Poly1305 decrypt fails closed (the F-2
//! strip-resistance contract).
//!
//! Per CLAUDE.md baked-in #5 / `crypto-agility-contract:6`: this is the
//! ONLY crypto-primitive call site; we wrap vetted upstream
//! `x25519-dalek` + `libcrux-ml-kem` (the hax/F*-verified ML-KEM-768
//! impl, via `crate::mlkem`) + `sha3` crates; we NEVER fork or
//! reimplement primitives.

use rand_core::OsRng as RandOsRng;
use sha3::Digest as _;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize as _, Zeroizing};

use crate::aead::{
    AeadEnvelope, AeadError, AeadKeyMaterial, aad_whole_content, unwrap as aead_unwrap,
    wrap as aead_wrap,
};
pub use crate::codepoint::CipherSuiteCodepoint;
use crate::error::UnsupportedAlgorithm;
use crate::mlkem;

/// X25519 public-key serialized length — the fixed 32-byte Curve25519
/// field-element wire size of [`x25519_dalek::PublicKey`] (`.as_bytes()`
/// yields `&[u8; 32]`; `PublicKey::from([u8; 32])` round-trips). Named
/// here (not hardcoded at call sites) per CLAUDE.md baked-in #5; the
/// `x25519_public_len_matches_type_width` test pins it against an actual
/// key's serialized width so it can never silently drift.
pub const X25519_PUBLIC_LEN: usize = 32;

/// X25519 secret-scalar serialized length — the fixed 32-byte Curve25519
/// wire size of [`x25519_dalek::StaticSecret`] (`.to_bytes()` yields
/// `[u8; 32]`; `StaticSecret::from([u8; 32])` round-trips). Named here per
/// CLAUDE.md baked-in #5; the `x25519_secret_len_matches_type_width` test
/// pins it against an actual key's serialized width.
pub const X25519_SECRET_LEN: usize = 32;

/// FIPS-203 ML-KEM-768 serialized sizes — public SSOT re-exports of the
/// production `crate::mlkem` size consts (R13 F-11), placed alongside the
/// `X25519_*_LEN` consts so a caller pinning the hybrid halves' wire widths
/// has one public home. Exposing FIPS-203 facts is benign + mirrors the
/// existing `X25519_PUBLIC_LEN`/`X25519_SECRET_LEN` public precedent; the
/// `f_kat_1` libcrux KAT drives real ML-KEM output against these so a
/// silent production-const drift away from FIPS-203 fails there.
pub use crate::mlkem::{
    ML_KEM_768_CT_LEN, ML_KEM_768_DK_LEN, ML_KEM_768_EK_LEN, ML_KEM_768_SS_LEN,
};

/// Multicodec varint prefix for the **X25519** KEM public-key COMPONENT —
/// the registered `x25519-pub = 0xec`, unsigned-varint-encoded as
/// `[0xec, 0x01]`. Per the multiformats multicodec table
/// (<https://github.com/multiformats/multicodec/blob/master/table.csv>).
///
/// The **first** component of a GAP-KDB Shape-B key-set `kem` multikey
/// (X25519-first per design correction C2 — matches the already-frozen
/// [`RecipientPublic::to_bytes`] order so no reorder is needed). Un-confusable
/// with the `[0x8c, 0x24]` ML-KEM tag. This is a component *algorithm* ID that
/// references a REGISTERED multiformats codec (CLAUDE.md baked-in #5 — Benten
/// never mints algorithm numbers).
const X25519_PUB_MULTICODEC: [u8; 2] = [0xec, 0x01];

/// Multicodec varint prefix for the **ML-KEM-768** KEM public-key COMPONENT —
/// the registered `mlkem-768-pub = 0x120c`, unsigned-varint-encoded as
/// `[0x8c, 0x24]`. Per the multiformats multicodec table.
///
/// The **second** component of a GAP-KDB Shape-B key-set `kem` multikey
/// (X25519-first per C2). Retires the #5-risky reserved-private
/// `HYBRID_KEM_MULTICODEC = 0xf0` in favor of the registered component codes
/// (design §5 + the `did:benten` HYBRID DID layout).
const MLKEM768_PUB_MULTICODEC: [u8; 2] = [0x8c, 0x24];

/// The real draft-connolly X-Wing `XWingLabel` — the 6 bytes
/// `0x5c2e2f2f5e5c` (ASCII `\.//^\`). **APPENDED** as the trailing suffix
/// of the combiner pre-image per `draft-connolly-cfrg-xwing-kem-10` §5.3
/// ("Combiner"; §6 is "Security Considerations" — the section-number cite
/// was R18-corrected §6→§5.3, DOC-ONLY, zero wire/byte change: the label
/// bytes + APPEND order are unchanged and stay R4.2-verified 2026-06-03 —
/// the prepended form is the superseded v01-v02 ordering and would freeze
/// a non-interoperable KEM at the IETF-reserved `0x647A`).
pub const X_WING_LABEL: [u8; 6] = [0x5c, 0x2e, 0x2f, 0x2f, 0x5e, 0x5c];

/// Classical-only `0x6400` combiner domain-separation info string. ASCII;
/// NOT an integer wire/AAD field (m-1: not flagged by the BE scanner).
///
/// NAMED CARVE-OUT (R17 F-09): this is the sole BENTEN-MINTED keying-surface
/// domain tag that applies the domain-separation idiom over key material
/// WITHOUT a `domain_registry` corpus entry / prefix-free enrollment (unlike
/// the recipient-seed label, which IS enrolled; the sibling [`X_WING_LABEL`] is
/// likewise un-enrolled but is NOT Benten-minted — its bytes are fixed by
/// `draft-connolly-cfrg-xwing-kem-10` §5.3 and it lives inside the single
/// `0x647a` combiner preimage, so it is not a cross-surface separator). Because it
/// folds only into the `0x6400` classical combiner preimage (a single
/// self-contained keying surface, not a cross-surface separator), it is left
/// UN-enrolled at v1-beta. The hardening — enroll it in
/// `domain_registry::registered_domain_tags()` OR record a documented exemption
/// in the module's "Scope carve-out" section — is NAMED at
/// `crates/benten-crypto-suite/src/domain_registry.rs` (the carve-out note) +
/// `docs/V1-FROZEN-INTERFACE-DEFERRED.md`. Doc-only; no enrollment this round
/// (enrolling now would add a corpus entry the frozen surface does not yet
/// carry).
const X25519_CLASSICAL_INFO_V1: &[u8] = b"x25519-classical-v1-benten-0x6400";

/// Deterministic-recipient-seed BLAKE3 expansion domain-separation label —
/// prefixed into the keyed-hash that expands a recipient `seed` into the three
/// 32-byte key-derivation blocks (X25519 + ML-KEM `d`/`z`). A registered
/// cross-surface domain-separation tag mirrored in
/// [`crate::domain_registry::RECIPIENT_SEED_LABEL`]; the intra-crate
/// `cipher_suite_domain_tags_match_central_registry` test pins byte-equality.
/// Canonical home is HERE.
pub const RECIPIENT_SEED_LABEL: &[u8] = b"benten-crypto-suite:recipient-seed";

/// G-CORE-3-hook cipher-suite dispatcher. **G-CORE-3a flips `0x647a` +
/// `0x6400` to LIVE.** Other codepoints typed-reject; G-CORE-3c (full
/// swap matrix) lights the rest.
pub struct CipherSuite {
    codepoint: CipherSuiteCodepoint,
}

impl CipherSuite {
    /// The v1-beta DEFAULT codepoint per RATIFIED-pq-default-reframe §1:
    /// X25519⊕ML-KEM-768 hybrid KEM at `0x647a` + ChaCha20-Poly1305 bulk.
    #[must_use]
    pub const fn v1_default_codepoint() -> CipherSuiteCodepoint {
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768
    }

    /// Resolve a cipher-suite codepoint. **G-CORE-3a LIVE for `0x647a` +
    /// `0x6400`**; other codepoints typed-reject.
    pub fn resolve(codepoint: CipherSuiteCodepoint) -> Result<Self, UnsupportedAlgorithm> {
        codepoint.resolve()?;
        Ok(Self { codepoint })
    }

    /// The dispatched codepoint.
    #[must_use]
    pub const fn codepoint(&self) -> CipherSuiteCodepoint {
        self.codepoint
    }

    /// True iff this suite is the v1-beta hybrid default.
    #[must_use]
    pub fn is_hybrid_default(&self) -> bool {
        self.codepoint == CipherSuiteCodepoint::HYBRID_X25519_MLKEM768
    }

    /// Generate a recipient keypair appropriate to this suite. Test-only
    /// API — real G-CORE-3 production will route keypair-generation
    /// through the principal-DID infrastructure.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn generate_recipient_keypair_for_test(suite: &Self) -> RecipientKeypair {
        match suite.codepoint.raw() {
            0x647a => {
                // Hybrid: BOTH X25519 + ML-KEM-768 halves.
                let x_sec = StaticSecret::random_from_rng(&mut RandOsRng);
                let x_pub = X25519PublicKey::from(&x_sec);
                let (mlkem_ek, mlkem_dk) = mlkem::generate().into_parts();
                RecipientKeypair {
                    codepoint: suite.codepoint,
                    public: RecipientPublic {
                        codepoint: suite.codepoint,
                        x25519: Some(x_pub),
                        mlkem768_ek: Some(mlkem_ek),
                    },
                    secret: RecipientSecret {
                        codepoint: suite.codepoint,
                        x25519: Some(x_sec),
                        mlkem768_dk: Some(mlkem_dk),
                    },
                }
            }
            0x6400 => {
                // Classical-only: X25519 only.
                let x_sec = StaticSecret::random_from_rng(&mut RandOsRng);
                let x_pub = X25519PublicKey::from(&x_sec);
                RecipientKeypair {
                    codepoint: suite.codepoint,
                    public: RecipientPublic {
                        codepoint: suite.codepoint,
                        x25519: Some(x_pub),
                        mlkem768_ek: None,
                    },
                    secret: RecipientSecret {
                        codepoint: suite.codepoint,
                        x25519: Some(x_sec),
                        mlkem768_dk: None,
                    },
                }
            }
            _ => unreachable!("CipherSuite::resolve guards against unsupported codepoints"),
        }
    }

    /// TEST-ONLY: deterministically derive a recipient keypair from a
    /// 32-byte `seed`.
    ///
    /// Both halves are derived from the seed via BLAKE3 domain-separated
    /// expansion (the X25519 `StaticSecret` from one 32-byte block; the
    /// ML-KEM-768 `(d, z)` from two more) so the same seed always yields
    /// the same keypair. Test fixtures use this to map a stable recipient
    /// pubkey *fingerprint* to a real hybrid keypair without a keystore
    /// round-trip (the `0x647a` hybrid + the `0x6400` classical downgrade
    /// are both supported; other codepoints would have been rejected by
    /// [`Self::resolve`]).
    ///
    /// **⚠️ NOT a production surface (R13 F-01 freeze-hygiene).** A
    /// keypair whose seed can be a *public* value is the GAP-1 footgun: if
    /// the seed is derivable by an attacker, the "secret" is forgeable
    /// (the `real_entropy_differs_from_public_seed_deterministic` unit test
    /// feeds it a `public_seed` and names its output `forgeable`). It has
    /// ZERO production callers — production keying goes through
    /// [`Self::generate_recipient_keypair`] (REAL OS entropy). So this is
    /// cfg-gated `#[cfg(any(test, feature = "testing"))]` + `_for_test`-named
    /// to keep it OFF the frozen default-feature public API entirely.
    ///
    /// Per CLAUDE.md baked-in #5 this stays the ONLY crypto-primitive call
    /// site — the seed expansion goes through the vetted `blake3` MAC and
    /// the keys through `x25519-dalek` / `ml-kem`; no primitive is forked.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn generate_recipient_keypair_deterministic_for_test(
        &self,
        seed: &[u8; 32],
    ) -> RecipientKeypair {
        // Domain-separated expansion of the seed into the three 32-byte
        // blocks the two key halves need.
        let block = |tag: u8| -> [u8; 32] {
            let mut h = blake3::Hasher::new();
            h.update(RECIPIENT_SEED_LABEL);
            h.update(&[tag]);
            h.update(seed);
            *h.finalize().as_bytes()
        };
        let x_block = block(0x01);
        let x_sec = StaticSecret::from(x_block);
        let x_pub = X25519PublicKey::from(&x_sec);
        match self.codepoint.raw() {
            0x647a => {
                // libcrux's FIPS-203 keygen takes the 64-byte `d‖z` seed.
                // The two 32-byte BLAKE3 blocks (`d` = block(0x02), `z` =
                // block(0x03)) are concatenated d-then-z — byte-identical to
                // RustCrypto `generate_deterministic(&d, &z)` for the same
                // (d, z) (proven by the canary spike + held by f_kat_1).
                let mut dz = [0u8; mlkem::KEYGEN_SEED_LEN];
                dz[..32].copy_from_slice(&block(0x02));
                dz[32..].copy_from_slice(&block(0x03));
                let (mlkem_ek, mlkem_dk) = mlkem::generate_deterministic(&dz).into_parts();
                RecipientKeypair {
                    codepoint: self.codepoint,
                    public: RecipientPublic {
                        codepoint: self.codepoint,
                        x25519: Some(x_pub),
                        mlkem768_ek: Some(mlkem_ek),
                    },
                    secret: RecipientSecret {
                        codepoint: self.codepoint,
                        x25519: Some(x_sec),
                        mlkem768_dk: Some(mlkem_dk),
                    },
                }
            }
            // Classical-only `0x6400`: X25519 half only. Explicit codepoint
            // arm + `unreachable!` tail (no silent downgrade-to-classical on an
            // unknown codepoint — CLAUDE.md #5 typed-reject-never-silent-fallback;
            // matches `generate_recipient_keypair_for_test`). `CipherSuite`
            // instances only exist via `resolve`, which rejects any other
            // codepoint before construction.
            0x6400 => RecipientKeypair {
                codepoint: self.codepoint,
                public: RecipientPublic {
                    codepoint: self.codepoint,
                    x25519: Some(x_pub),
                    mlkem768_ek: None,
                },
                secret: RecipientSecret {
                    codepoint: self.codepoint,
                    x25519: Some(x_sec),
                    mlkem768_dk: None,
                },
            },
            _ => unreachable!("CipherSuite::resolve guards against unsupported codepoints"),
        }
    }

    /// Generate a REAL-entropy recipient keypair appropriate to this
    /// suite — the production keying path the engine mints identity key
    /// material through (R9-S1; closes GAP-1's root: the prior Layer-C
    /// placeholder derived the "secret" from the public fingerprint, so
    /// there was ZERO secret entropy).
    ///
    /// Both halves are seeded from the crate's OS RNG (`rand_core::OsRng`,
    /// the SAME source the seal path uses for ephemerals + the SAME source
    /// `crate::mlkem::generate` fills its `d‖z` seed from). This is
    /// **NON-deterministic**: two calls yield distinct public AND secret
    /// bytes. Contrast `Self::generate_recipient_keypair_deterministic_for_test`
    /// (seeded from a public fingerprint → forgeable; test-only) and
    /// `generate_recipient_keypair_for_test` (test-fixture entropy).
    ///
    /// Per CLAUDE.md baked-in #5 this stays crypto-primitive glue — x25519
    /// keygen via `x25519-dalek` + ML-KEM-768 keygen via the vetted
    /// libcrux wrapper (`crate::mlkem`); no primitive is forked, no size
    /// is hardcoded.
    #[must_use]
    pub fn generate_recipient_keypair(&self) -> RecipientKeypair {
        // X25519 half — real OS entropy (same RNG source as the seal
        // path's ephemerals + the deterministic path's structural key).
        let x_sec = StaticSecret::random_from_rng(&mut RandOsRng);
        let x_pub = X25519PublicKey::from(&x_sec);
        match self.codepoint.raw() {
            0x647a => {
                // Hybrid: real ML-KEM-768 keygen (libcrux fills its 64-byte
                // `d‖z` seed from the same OS RNG per `mlkem::generate`).
                let (mlkem_ek, mlkem_dk) = mlkem::generate().into_parts();
                RecipientKeypair {
                    codepoint: self.codepoint,
                    public: RecipientPublic {
                        codepoint: self.codepoint,
                        x25519: Some(x_pub),
                        mlkem768_ek: Some(mlkem_ek),
                    },
                    secret: RecipientSecret {
                        codepoint: self.codepoint,
                        x25519: Some(x_sec),
                        mlkem768_dk: Some(mlkem_dk),
                    },
                }
            }
            // Classical-only `0x6400`: X25519 half only (resolve() already
            // rejected every other codepoint). Explicit arm + `unreachable!`
            // tail — no silent downgrade-to-classical on an unknown codepoint
            // (CLAUDE.md #5 typed-reject-never-silent-fallback).
            0x6400 => RecipientKeypair {
                codepoint: self.codepoint,
                public: RecipientPublic {
                    codepoint: self.codepoint,
                    x25519: Some(x_pub),
                    mlkem768_ek: None,
                },
                secret: RecipientSecret {
                    codepoint: self.codepoint,
                    x25519: Some(x_sec),
                    mlkem768_dk: None,
                },
            },
            _ => unreachable!("CipherSuite::resolve guards against unsupported codepoints"),
        }
    }

    /// Wrap `k_root` for `recipient_pub` via the X-Wing-hybrid combiner
    /// (or classical-only at `0x6400`).
    ///
    /// Returns a [`WrappedKey`] carrying the encapsulated keys + the
    /// ciphertext-encrypted `k_root`. Round-trips byte-identically
    /// through `unwrap_key_material` on the recipient's secret.
    pub fn wrap_key_material(
        &self,
        recipient_pub: &RecipientPublic,
        k_root: &[u8],
    ) -> Result<WrappedKey, AeadError> {
        // Codepoint discipline: the recipient_pub MUST be sourced from
        // the SAME suite (additive-codepoint discipline; no cross-suite
        // pub-key reuse).
        if recipient_pub.codepoint != self.codepoint {
            return Err(AeadError::Unsupported(UnsupportedAlgorithm::CipherSuite {
                codepoint: recipient_pub.codepoint.raw(),
            }));
        }
        match self.codepoint.raw() {
            0x647a => {
                let x_recipient = recipient_pub
                    .x25519
                    .as_ref()
                    .ok_or(AeadError::RecipientLacksKeysForSuite)?;
                let mlkem_ek_bytes = recipient_pub
                    .mlkem768_ek
                    .as_ref()
                    .ok_or(AeadError::RecipientLacksKeysForSuite)?;

                // X25519 ephemeral-static ECDH → produces the X25519
                // shared secret + ephemeral public (the "ek_x").
                let x_eph = EphemeralSecret::random_from_rng(&mut RandOsRng);
                let ek_x = X25519PublicKey::from(&x_eph);
                let ss_x = x_eph.diffie_hellman(x_recipient);

                // ML-KEM-768 encapsulation → (mlkem_ct, ss_mlkem). The OS
                // RNG supplies the 32-byte `m` randomness inside the wrapper
                // (same randomness source as the prior RustCrypto path).
                // `ss_mlkem` is a KEM shared secret — wrap in `Zeroizing` so
                // the `Vec<u8>` is wiped when this scope ends (F-08).
                let (mlkem_ct, ss_mlkem) = mlkem::encapsulate(mlkem_ek_bytes).ok_or(
                    AeadError::MalformedEnvelope("recipient ML-KEM-768 ek malformed"),
                )?;
                let ss_mlkem = Zeroizing::new(ss_mlkem);

                // Real draft-connolly X-Wing combiner:
                // SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel).
                // ss_M = ML-KEM-768 shared secret; ss_X = X25519 shared
                // secret; ct_X = the X25519 ephemeral public key (ek_x);
                // pk_X = the recipient X25519 public key. The ML-KEM
                // ciphertext is bound transitively via ss_M (decapsulation).
                // `combined` is the wrapping KEK — `Zeroizing` wipes the raw
                // [u8; 32] after it is copied into the (also-zeroizing)
                // `AeadKeyMaterial` (F-08).
                let combined = Zeroizing::new(combine_x_wing(
                    &ss_mlkem,
                    ss_x.as_bytes(),
                    ek_x.as_bytes(),
                    x_recipient.as_bytes(),
                ));

                // AEAD-encrypt k_root under the combined key.
                let combined_key = AeadKeyMaterial::from_bytes(self.codepoint, combined.to_vec());
                let aad = aad_whole_content(b"x-wing-wrap:k_root");
                let env = aead_wrap(k_root, &combined_key, &aad)?;

                Ok(WrappedKey {
                    codepoint: self.codepoint,
                    ek_x: ek_x.as_bytes().to_vec(),
                    ek_mlkem: mlkem_ct,
                    aead_envelope: env,
                })
            }
            0x6400 => {
                // Classical-only X25519: ephemeral-static ECDH; no ML-KEM half.
                let x_recipient = recipient_pub
                    .x25519
                    .as_ref()
                    .ok_or(AeadError::RecipientLacksKeysForSuite)?;
                let x_eph = EphemeralSecret::random_from_rng(&mut RandOsRng);
                let ek_x = X25519PublicKey::from(&x_eph);
                let ss_x = x_eph.diffie_hellman(x_recipient);

                // SHA3-256 over (ss_x || ek_x || recipient_pub ||
                // X25519_CLASSICAL_INFO) via `classical_combine` — same
                // hash family as the hybrid arm but with the classical-only
                // inputs (no HKDF; the prior HKDF-SHA256 label was a mislabel).
                // `combined` is the classical wrapping KEK — `Zeroizing`
                // wipes the raw [u8; 32] after the copy into `AeadKeyMaterial`
                // (F-08, combiner-KEK-output parity with the hybrid arm).
                let combined = Zeroizing::new(classical_combine(
                    ss_x.as_bytes(),
                    ek_x.as_bytes(),
                    x_recipient.as_bytes(),
                ));
                let combined_key = AeadKeyMaterial::from_bytes(self.codepoint, combined.to_vec());
                let aad = aad_whole_content(b"x25519-classical-wrap:k_root");
                let env = aead_wrap(k_root, &combined_key, &aad)?;

                Ok(WrappedKey {
                    codepoint: self.codepoint,
                    ek_x: ek_x.as_bytes().to_vec(),
                    ek_mlkem: Vec::new(),
                    aead_envelope: env,
                })
            }
            other => Err(AeadError::Unsupported(UnsupportedAlgorithm::CipherSuite {
                codepoint: other,
            })),
        }
    }

    /// Unwrap a [`WrappedKey`] via the recipient's secret.
    ///
    /// Per F-2 strip-resistance: the X-Wing combiner mixes BOTH halves
    /// — stripping either half yields a different derived key →
    /// ChaCha20-Poly1305 decrypt fails closed with
    /// [`AeadError::AeadAuthFailed`].
    pub fn unwrap_key_material(
        &self,
        recipient_sec: &RecipientSecret,
        wrapped: &WrappedKey,
    ) -> Result<UnwrappedKey, AeadError> {
        if wrapped.codepoint != self.codepoint {
            return Err(AeadError::Unsupported(UnsupportedAlgorithm::CipherSuite {
                codepoint: wrapped.codepoint.raw(),
            }));
        }
        if recipient_sec.codepoint != self.codepoint {
            return Err(AeadError::RecipientLacksKeysForSuite);
        }
        match self.codepoint.raw() {
            0x647a => {
                let x_sec = recipient_sec
                    .x25519
                    .as_ref()
                    .ok_or(AeadError::RecipientLacksKeysForSuite)?;
                let mlkem_dk_bytes = recipient_sec
                    .mlkem768_dk
                    .as_ref()
                    .ok_or(AeadError::RecipientLacksKeysForSuite)?;

                // Decapsulate X25519 half.
                let ek_x_bytes: [u8; 32] = wrapped
                    .ek_x
                    .as_slice()
                    .try_into()
                    .map_err(|_| AeadError::MalformedEnvelope("ek_x wrong length"))?;
                let ek_x_pub = X25519PublicKey::from(ek_x_bytes);
                let ss_x = x_sec.diffie_hellman(&ek_x_pub);

                // Decapsulate ML-KEM-768 half. A malformed dk OR ct surfaces
                // a typed MalformedEnvelope (the strip-resistance contract
                // routes the zeroed-PQ-half F-2 case through a *different
                // derived key* → AEAD fails closed, not through this arm).
                // `ss_mlkem` is the recovered KEM shared secret — `Zeroizing`
                // wipes the `Vec<u8>` at scope end (F-08).
                let ss_mlkem = Zeroizing::new(
                    mlkem::decapsulate(mlkem_dk_bytes, wrapped.ek_mlkem.as_slice())
                        .ok_or(AeadError::MalformedEnvelope("ML-KEM-768 dk/ct malformed"))?,
                );

                // Recover the recipient's public material to feed the
                // combiner symmetrically (it bound them at wrap-time).
                let x_pub = X25519PublicKey::from(x_sec);

                // `combined` is the recovered KEK — `Zeroizing` wipes the raw
                // [u8; 32] after the copy into `AeadKeyMaterial` (F-08).
                let combined = Zeroizing::new(combine_x_wing(
                    &ss_mlkem,
                    ss_x.as_bytes(),
                    &ek_x_bytes,
                    x_pub.as_bytes(),
                ));

                let combined_key = AeadKeyMaterial::from_bytes(self.codepoint, combined.to_vec());
                let aad = aad_whole_content(b"x-wing-wrap:k_root");
                let plaintext = aead_unwrap(&wrapped.aead_envelope, &combined_key, &aad)?;
                Ok(UnwrappedKey { bytes: plaintext })
            }
            0x6400 => {
                let x_sec = recipient_sec
                    .x25519
                    .as_ref()
                    .ok_or(AeadError::RecipientLacksKeysForSuite)?;
                let ek_x_bytes: [u8; 32] = wrapped
                    .ek_x
                    .as_slice()
                    .try_into()
                    .map_err(|_| AeadError::MalformedEnvelope("ek_x wrong length"))?;
                let ek_x_pub = X25519PublicKey::from(ek_x_bytes);
                let ss_x = x_sec.diffie_hellman(&ek_x_pub);
                let x_pub = X25519PublicKey::from(x_sec);
                // `combined` is the recovered classical KEK — `Zeroizing`
                // wipes the raw [u8; 32] after the copy into `AeadKeyMaterial`
                // (F-08).
                let combined = Zeroizing::new(classical_combine(
                    ss_x.as_bytes(),
                    &ek_x_bytes,
                    x_pub.as_bytes(),
                ));
                let combined_key = AeadKeyMaterial::from_bytes(self.codepoint, combined.to_vec());
                let aad = aad_whole_content(b"x25519-classical-wrap:k_root");
                let plaintext = aead_unwrap(&wrapped.aead_envelope, &combined_key, &aad)?;
                Ok(UnwrappedKey { bytes: plaintext })
            }
            other => Err(AeadError::Unsupported(UnsupportedAlgorithm::CipherSuite {
                codepoint: other,
            })),
        }
    }

    /// Seal `plaintext` under a directly-supplied `k_root` (e.g. for
    /// per-Node AEAD where K(N) was structurally derived). AAD binds
    /// `plaintext_cid` per §1.A.FROZEN item 15(g).
    pub fn seal_aead(
        &self,
        k_root: &[u8],
        plaintext: &[u8],
        plaintext_cid: &[u8],
    ) -> Result<AeadEnvelope, AeadError> {
        let key = AeadKeyMaterial::from_bytes(self.codepoint, k_root.to_vec());
        let aad = aad_whole_content(plaintext_cid);
        aead_wrap(plaintext, &key, &aad)
    }

    /// Open an envelope under a directly-supplied `k_root` + bind AAD
    /// to `plaintext_cid`. Re-binding the envelope to a different
    /// `plaintext_cid` MUST fail closed (rebinding-attack defense).
    pub fn open_aead(
        &self,
        k_root: &[u8],
        envelope: &AeadEnvelope,
        plaintext_cid: &[u8],
    ) -> Result<DecryptedPlaintext, AeadError> {
        let key = AeadKeyMaterial::from_bytes(self.codepoint, k_root.to_vec());
        let aad = aad_whole_content(plaintext_cid);
        let bytes = aead_unwrap(envelope, &key, &aad)?;
        Ok(DecryptedPlaintext { bytes })
    }

    /// Dispatch from a `WrappedKey`'s embedded codepoint — used by the
    /// F-4 unknown-codepoint-at-decrypt pin (the never-silent-fallback
    /// contract symmetric across encrypt + decrypt).
    pub fn resolve_for_wrapped_key(wrapped: &WrappedKey) -> Result<Self, UnsupportedAlgorithm> {
        Self::resolve(wrapped.codepoint)
    }
}

/// Build the real draft-connolly X-Wing combiner pre-image (the exact byte
/// sequence fed to `SHA3-256`):
/// `ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel` — the 6-byte `XWingLabel` is
/// **APPENDED** as the trailing suffix per `draft-connolly-cfrg-xwing-kem-10`
/// §5.3 "Combiner" (R4.2-corrected; §6→§5.3 section-cite R18-corrected, DOC-ONLY).
/// `ss_M` = ML-KEM-768 shared secret, `ss_X` = X25519
/// shared secret, `ct_X` = the X25519 ephemeral public key (the X25519
/// "ciphertext"), `pk_X` = the recipient X25519 public key.
///
/// Exposed so the F-W0-1-LABEL construction-order witness pin can assert the
/// label is the appended suffix (and NOT a prepended prefix).
///
/// **`ct_mlkem` is DELIBERATELY absent from the pre-image — SPEC-FAITHFUL
/// (freeze-record; R9-council GAP-3; DO NOT "fix" by adding `ct_mlkem`).** The
/// combiner binds `ct_X` (the X25519 ciphertext) but NOT `ct_mlkem` (the
/// ML-KEM-768 ciphertext) directly, EXACTLY as `draft-connolly-cfrg-xwing-kem-10`
/// §5.3 "Combiner" specifies. This is not an omission: ML-KEM-768 is IND-CCA2, so its shared
/// secret `ss_M` **transitively binds** `ct_mlkem` (the FO-transform ties the
/// ML-KEM shared secret to its own ciphertext), giving X-Wing its LEAK-freeness
/// / binding property without re-hashing `ct_mlkem`. Adding `ct_mlkem` to the
/// pre-image would DIVERGE from the IETF-faithful construction at the reserved
/// `0x647A` codepoint (a wire-break) for zero security gain. Cross-record:
/// Inv-17 (hybrid floor); the `tf2_*` strip-resistance pins.
#[must_use]
pub fn x_wing_combiner_preimage(ss_m: &[u8], ss_x: &[u8], ct_x: &[u8], pk_x: &[u8]) -> Vec<u8> {
    let mut pre = Vec::with_capacity(ss_m.len() + ss_x.len() + ct_x.len() + pk_x.len() + 6);
    pre.extend_from_slice(ss_m);
    pre.extend_from_slice(ss_x);
    pre.extend_from_slice(ct_x);
    pre.extend_from_slice(pk_x);
    // APPENDED suffix (draft-connolly §5.3 "Combiner"; NOT prepended).
    pre.extend_from_slice(&X_WING_LABEL);
    pre
}

/// The real draft-connolly X-Wing combiner:
/// `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)`.
///
/// This replaces the prior HKDF-SHA256 mislabel (`cipher_suite.rs:407` at
/// the corpus base) with the IETF-faithful construction at the IETF-reserved
/// codepoint `0x647A`. Per CLAUDE.md baked-in #5 the SHA3-256 primitive is
/// wrapped from the vetted upstream `sha3` crate — no reimplementation.
///
/// Argument order is `(ss_mlkem, ss_x25519, ct_x25519, pk_x25519)` to mirror
/// the spec pre-image `ss_M ‖ ss_X ‖ ct_X ‖ pk_X`.
#[must_use]
pub fn combine_x_wing(
    ss_mlkem: &[u8],
    ss_x25519: &[u8],
    ct_x25519: &[u8],
    pk_x25519: &[u8],
) -> [u8; 32] {
    let pre = x_wing_combiner_preimage(ss_mlkem, ss_x25519, ct_x25519, pk_x25519);
    let mut h = sha3::Sha3_256::new();
    h.update(&pre);
    h.finalize().into()
}

/// The classical-only X25519 combiner (the `0x6400` downgrade arm),
/// re-derived consistently with the real-X-Wing rewrite (§3.2(a)):
/// `SHA3-256(ss_X ‖ ct_X ‖ pk_X ‖ X25519_CLASSICAL_INFO)`. SHA3-256-based
/// (matching the hybrid arm's hash family) so the two arms share the same
/// hash primitive but derive distinct keys (distinct input set + distinct
/// trailing domain-separation string).
///
/// **Preimage length-injectivity (R17 F-26; mirrors the Row D-13
/// concatenation-injectivity criterion).** The un-length-prefixed
/// concatenation `ss_X ‖ ek_X ‖ pub_X ‖ INFO` is injective — i.e. two
/// distinct input tuples cannot produce the same preimage bytes — ONLY
/// because every field is FIXED-LENGTH at this call: `ss_x` / `ek_x` /
/// `pub_x` are each the 32-byte X25519 output/pubkey and `X25519_CLASSICAL_INFO_V1`
/// is a fixed constant. The carve-out criterion (same as Row D-13's
/// info-tag folding): a bare concatenation is a sound domain separator IFF
/// every component has a fixed, statically-known length OR is length-prefixed.
/// If a future edit ever feeds a VARIABLE-length component into this
/// combiner, it MUST length-prefix that component (or the concat stops being
/// injective and opens a preimage-collision confusion path). The trailing
/// constant `INFO` string additionally domain-separates this arm from the
/// hybrid X-Wing combiner (which uses the appended [`X_WING_LABEL`]).
#[must_use]
pub fn classical_combine(ss_x: &[u8], ek_x: &[u8], pub_x: &[u8]) -> [u8; 32] {
    let mut pre =
        Vec::with_capacity(ss_x.len() + ek_x.len() + pub_x.len() + X25519_CLASSICAL_INFO_V1.len());
    pre.extend_from_slice(ss_x);
    pre.extend_from_slice(ek_x);
    pre.extend_from_slice(pub_x);
    pre.extend_from_slice(X25519_CLASSICAL_INFO_V1);
    let mut h = sha3::Sha3_256::new();
    h.update(&pre);
    h.finalize().into()
}

/// Recipient keypair carrying X25519 + (optionally) ML-KEM-768 halves
/// per the suite's codepoint.
pub struct RecipientKeypair {
    codepoint: CipherSuiteCodepoint,
    public: RecipientPublic,
    secret: RecipientSecret,
}

impl RecipientKeypair {
    /// The public half.
    #[must_use]
    pub fn public(&self) -> &RecipientPublic {
        &self.public
    }
    /// The secret half.
    #[must_use]
    pub fn secret(&self) -> &RecipientSecret {
        &self.secret
    }
    /// The suite's codepoint.
    #[must_use]
    pub const fn codepoint(&self) -> CipherSuiteCodepoint {
        self.codepoint
    }
    /// Adversarial helper — drop the ML-KEM half from a hybrid keypair
    /// (the F-3 RecipientLacksKeysForSuite pin). Returns a degenerate
    /// keypair carrying only the X25519 half but still tagged with the
    /// hybrid codepoint — the unwrap path MUST surface
    /// [`AeadError::RecipientLacksKeysForSuite`].
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn with_only_classical_half_for_test(full: &RecipientKeypair) -> Self {
        Self {
            codepoint: full.codepoint,
            public: RecipientPublic {
                codepoint: full.codepoint,
                x25519: full.public.x25519,
                mlkem768_ek: None,
            },
            secret: RecipientSecret {
                codepoint: full.codepoint,
                x25519: full.secret.x25519.clone(),
                mlkem768_dk: None,
            },
        }
    }
}

/// Recipient's public material (X25519 pub-key + optionally ML-KEM-768
/// encapsulation key).
pub struct RecipientPublic {
    codepoint: CipherSuiteCodepoint,
    x25519: Option<X25519PublicKey>,
    mlkem768_ek: Option<Vec<u8>>,
}

impl RecipientPublic {
    /// The codepoint this public-key is bound to.
    #[must_use]
    pub const fn codepoint(&self) -> CipherSuiteCodepoint {
        self.codepoint
    }

    /// Serialize the recipient public material to its canonical byte layout
    /// (so it can be advertised via a DID / stored in the vault).
    ///
    /// Layout is codepoint-dispatched (the codepoint is carried out-of-band
    /// alongside these bytes; the caller pairs it back in
    /// [`Self::from_bytes`]):
    ///
    /// - `0x647a` hybrid: `x25519_pub(32) ‖ mlkem768_ek(1184)` — the x25519
    ///   public key followed by the FIPS-203 ML-KEM-768 encapsulation key.
    /// - `0x6400` classical: `x25519_pub(32)`.
    ///
    /// Sizes flow from the upstream type constants
    /// (`crate::mlkem::ML_KEM_768_EK_LEN` + the x25519 32-byte public key)
    /// — never hardcoded (CLAUDE.md baked-in #5).
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        if let Some(x_pub) = self.x25519.as_ref() {
            out.extend_from_slice(x_pub.as_bytes());
        }
        if let Some(ek) = self.mlkem768_ek.as_ref() {
            out.extend_from_slice(ek);
        }
        out
    }

    /// Parse a recipient public byte blob for `codepoint` back into a
    /// [`RecipientPublic`]. Fail-closed typed-reject on any malformed /
    /// wrong-length input — NEVER a silent default (CLAUDE.md baked-in #5).
    ///
    /// The `codepoint` is supplied out-of-band (it travels with the blob in
    /// the DID / vault record) and drives the expected layout — see
    /// [`Self::to_bytes`].
    ///
    /// # Errors
    ///
    /// - [`AeadError::MalformedRecipientPublic`] if `bytes` is the wrong
    ///   length for `codepoint`, or the ML-KEM-768 encapsulation key does
    ///   not parse.
    /// - [`AeadError::Unsupported`] if `codepoint` is not a live cipher
    ///   suite.
    pub fn from_bytes(codepoint: CipherSuiteCodepoint, bytes: &[u8]) -> Result<Self, AeadError> {
        // Reject unknown/reserved codepoints up-front (never silent default).
        CipherSuite::resolve(codepoint)?;
        match codepoint.raw() {
            0x647a => {
                let expect = X25519_PUBLIC_LEN + mlkem::ML_KEM_768_EK_LEN;
                if bytes.len() != expect {
                    return Err(AeadError::MalformedRecipientPublic(
                        "hybrid recipient public wrong length (expected x25519_pub||mlkem768_ek)",
                    ));
                }
                let (x_bytes, ek_bytes) = bytes.split_at(X25519_PUBLIC_LEN);
                let x_arr: [u8; X25519_PUBLIC_LEN] = x_bytes.try_into().map_err(|_| {
                    AeadError::MalformedRecipientPublic("x25519 public wrong length")
                })?;
                // Reject a malformed ML-KEM ek eagerly (encapsulate() would
                // otherwise surface it later); a byte-length-correct ek that
                // fails structural parse is caught by the encapsulate path,
                // but we validate the length invariant here so from_bytes is
                // total over its declared contract.
                Ok(Self {
                    codepoint,
                    x25519: Some(X25519PublicKey::from(x_arr)),
                    mlkem768_ek: Some(ek_bytes.to_vec()),
                })
            }
            // Classical-only `0x6400`: x25519 half only.
            _ => {
                if bytes.len() != X25519_PUBLIC_LEN {
                    return Err(AeadError::MalformedRecipientPublic(
                        "classical recipient public wrong length (expected x25519_pub)",
                    ));
                }
                let x_arr: [u8; X25519_PUBLIC_LEN] = bytes.try_into().map_err(|_| {
                    AeadError::MalformedRecipientPublic("x25519 public wrong length")
                })?;
                Ok(Self {
                    codepoint,
                    x25519: Some(X25519PublicKey::from(x_arr)),
                    mlkem768_ek: None,
                })
            }
        }
    }

    /// Decode a **GAP-KDB Shape-B key-set `kem` multikey** (the `did:benten`
    /// key-set-document `kem` field) into a [`RecipientPublic`] bound to
    /// `codepoint`, cross-checking the component multicodecs against the
    /// out-of-band `kem_cp` (design correction **C2**). Fail-closed
    /// typed-reject on ANY malformed multikey / component-vs-`kem_cp`
    /// disagreement / unsupported codepoint — NEVER a silent default
    /// (CLAUDE.md baked-in #5).
    ///
    /// # The frozen wire (design §5 + C2)
    ///
    /// The `kem` multikey is **X25519-first** (matching the already-frozen
    /// [`Self::to_bytes`] order, so no reorder is needed — the §5 REORDER
    /// foot-gun is deleted by C2). Component algorithm IDs reference REGISTERED
    /// multiformats codecs:
    ///
    /// - `0x647a` hybrid: `0xec01 ‖ x25519(32) ‖ 0x8c24 ‖ mlkem768_ek(1184)`
    ///   (component set `{x25519-pub, mlkem-768-pub}`, in that order).
    /// - `0x6400` classical: `0xec01 ‖ x25519(32)` (component set `{x25519-pub}`).
    ///
    /// # The `kem_cp` ⟺ component-set cross-check (C2 — the load-bearing check)
    ///
    /// [`Self::from_bytes`] dispatches purely on `codepoint` over the
    /// component-varint-STRIPPED payload, so it cannot catch a multikey whose
    /// payload length is correct for `codepoint` but whose component varints
    /// declare the WRONG algorithms (X25519 ↔ ML-KEM swapped / ML-KEM-1024
    /// substituted / a signing codec spliced into the KEM slot / a hybrid ML-KEM
    /// component present under a classical claim). This decoder is the ONLY place
    /// the component codecs are cross-checked against `kem_cp` — the
    /// algorithm-confusion defense. Every component multicodec MUST match the
    /// sequence mandated by `codepoint`, at its exact length, with NO trailing
    /// bytes (Row-D-13 injectivity discipline).
    ///
    /// All sizes flow from the upstream size constants ([`X25519_PUBLIC_LEN`],
    /// [`ML_KEM_768_EK_LEN`]) — never hardcoded (CLAUDE.md baked-in #5).
    ///
    /// # Errors
    ///
    /// - [`AeadError::MalformedRecipientPublic`] on a wrong total length, a
    ///   wrong / mis-ordered component multicodec, or (via [`Self::from_bytes`])
    ///   a malformed component payload.
    /// - [`AeadError::Unsupported`] if `codepoint` is not a live cipher suite.
    pub fn from_kem_multikey(
        codepoint: CipherSuiteCodepoint,
        kem_multikey: &[u8],
    ) -> Result<Self, AeadError> {
        // Reject unknown/reserved codepoints up-front (never silent default);
        // this bounds the match below to the two live cipher suites.
        CipherSuite::resolve(codepoint)?;
        match codepoint.raw() {
            0x647a => {
                // Hybrid: 0xec01 ‖ x25519(32) ‖ 0x8c24 ‖ mlkem768_ek(1184).
                let expected = X25519_PUB_MULTICODEC.len()
                    + X25519_PUBLIC_LEN
                    + MLKEM768_PUB_MULTICODEC.len()
                    + mlkem::ML_KEM_768_EK_LEN;
                if kem_multikey.len() != expected {
                    return Err(AeadError::MalformedRecipientPublic(
                        "hybrid kem multikey wrong length (expected \
                         0xec‖x25519(32)‖0x120c‖mlkem768_ek(1184))",
                    ));
                }
                // Component 1: x25519-pub (0xec01) — MUST lead (C2 X25519-first).
                if kem_multikey[0] != X25519_PUB_MULTICODEC[0]
                    || kem_multikey[1] != X25519_PUB_MULTICODEC[1]
                {
                    return Err(AeadError::MalformedRecipientPublic(
                        "hybrid kem multikey: first component multicodec != \
                         x25519-pub 0xec (kem_cp⟺components cross-check — C2 \
                         mandates X25519-first)",
                    ));
                }
                let x_start = X25519_PUB_MULTICODEC.len();
                let ml_tag = x_start + X25519_PUBLIC_LEN;
                // Component 2: mlkem-768-pub (0x120c → varint 0x8c24).
                if kem_multikey[ml_tag] != MLKEM768_PUB_MULTICODEC[0]
                    || kem_multikey[ml_tag + 1] != MLKEM768_PUB_MULTICODEC[1]
                {
                    return Err(AeadError::MalformedRecipientPublic(
                        "hybrid kem multikey: second component multicodec != \
                         mlkem-768-pub 0x120c (kem_cp⟺components cross-check — \
                         wrong ML-KEM parameter / wrong-role codec / classical-\
                         only-under-hybrid)",
                    ));
                }
                let ek_start = ml_tag + MLKEM768_PUB_MULTICODEC.len();
                // Reassemble the X25519-first from_bytes payload
                // `x25519(32) ‖ mlkem768_ek(1184)` (no reorder — C2).
                let mut payload = Vec::with_capacity(X25519_PUBLIC_LEN + mlkem::ML_KEM_768_EK_LEN);
                payload.extend_from_slice(&kem_multikey[x_start..ml_tag]);
                payload.extend_from_slice(&kem_multikey[ek_start..]);
                Self::from_bytes(codepoint, &payload)
            }
            // Classical-only `0x6400`: 0xec01 ‖ x25519(32), no ML-KEM component.
            _ => {
                let expected = X25519_PUB_MULTICODEC.len() + X25519_PUBLIC_LEN;
                if kem_multikey.len() != expected {
                    return Err(AeadError::MalformedRecipientPublic(
                        "classical kem multikey wrong length (expected \
                         0xec‖x25519(32)) — a hybrid ML-KEM component under a \
                         classical kem_cp fails closed here",
                    ));
                }
                if kem_multikey[0] != X25519_PUB_MULTICODEC[0]
                    || kem_multikey[1] != X25519_PUB_MULTICODEC[1]
                {
                    return Err(AeadError::MalformedRecipientPublic(
                        "classical kem multikey: component multicodec != \
                         x25519-pub 0xec (kem_cp⟺components cross-check)",
                    ));
                }
                Self::from_bytes(codepoint, &kem_multikey[X25519_PUB_MULTICODEC.len()..])
            }
        }
    }
}

/// Recipient's secret material (X25519 secret + optionally ML-KEM-768
/// decapsulation key).
pub struct RecipientSecret {
    codepoint: CipherSuiteCodepoint,
    x25519: Option<StaticSecret>,
    mlkem768_dk: Option<Vec<u8>>,
}

impl RecipientSecret {
    /// The codepoint this secret is bound to.
    #[must_use]
    pub const fn codepoint(&self) -> CipherSuiteCodepoint {
        self.codepoint
    }

    /// Serialize the recipient secret material to its canonical byte layout
    /// (so the vault can seal it at rest). Layout is codepoint-dispatched
    /// (the codepoint travels alongside; the caller pairs it back in
    /// [`Self::from_bytes`]):
    ///
    /// - `0x647a` hybrid: `x25519_sec(32) ‖ mlkem768_dk(2400)` — the x25519
    ///   secret scalar followed by the FIPS-203 ML-KEM-768 decapsulation
    ///   key (full expanded form).
    /// - `0x6400` classical: `x25519_sec(32)`.
    ///
    /// Sizes flow from the upstream type constants
    /// (`crate::mlkem::ML_KEM_768_DK_LEN` + the x25519 32-byte secret) —
    /// never hardcoded (CLAUDE.md baked-in #5).
    ///
    /// The returned `Vec` is raw secret bytes — callers MUST wipe it (seal
    /// it into the vault + drop, or wrap in [`zeroize::Zeroizing`]).
    #[must_use]
    pub fn to_bytes(&self) -> Zeroizing<Vec<u8>> {
        let mut out = Vec::new();
        if let Some(x_sec) = self.x25519.as_ref() {
            out.extend_from_slice(&x_sec.to_bytes());
        }
        if let Some(dk) = self.mlkem768_dk.as_ref() {
            out.extend_from_slice(dk);
        }
        Zeroizing::new(out)
    }

    /// Parse a recipient secret byte blob for `codepoint` back into a
    /// [`RecipientSecret`]. Fail-closed typed-reject on any malformed /
    /// wrong-length input — NEVER a silent default (CLAUDE.md baked-in #5).
    ///
    /// The `codepoint` is supplied out-of-band (it travels with the blob in
    /// the vault record) and drives the expected layout — see
    /// [`Self::to_bytes`].
    ///
    /// # Errors
    ///
    /// - [`AeadError::MalformedRecipientSecret`] if `bytes` is the wrong
    ///   length for `codepoint`.
    /// - [`AeadError::Unsupported`] if `codepoint` is not a live cipher
    ///   suite.
    pub fn from_bytes(codepoint: CipherSuiteCodepoint, bytes: &[u8]) -> Result<Self, AeadError> {
        // Reject unknown/reserved codepoints up-front (never silent default).
        CipherSuite::resolve(codepoint)?;
        match codepoint.raw() {
            0x647a => {
                let expect = X25519_SECRET_LEN + mlkem::ML_KEM_768_DK_LEN;
                if bytes.len() != expect {
                    return Err(AeadError::MalformedRecipientSecret(
                        "hybrid recipient secret wrong length (expected x25519_sec||mlkem768_dk)",
                    ));
                }
                let (x_bytes, dk_bytes) = bytes.split_at(X25519_SECRET_LEN);
                let x_arr: [u8; X25519_SECRET_LEN] = x_bytes.try_into().map_err(|_| {
                    AeadError::MalformedRecipientSecret("x25519 secret wrong length")
                })?;
                Ok(Self {
                    codepoint,
                    x25519: Some(StaticSecret::from(x_arr)),
                    mlkem768_dk: Some(dk_bytes.to_vec()),
                })
            }
            // Classical-only `0x6400`: x25519 half only.
            _ => {
                if bytes.len() != X25519_SECRET_LEN {
                    return Err(AeadError::MalformedRecipientSecret(
                        "classical recipient secret wrong length (expected x25519_sec)",
                    ));
                }
                let x_arr: [u8; X25519_SECRET_LEN] = bytes.try_into().map_err(|_| {
                    AeadError::MalformedRecipientSecret("x25519 secret wrong length")
                })?;
                Ok(Self {
                    codepoint,
                    x25519: Some(StaticSecret::from(x_arr)),
                    mlkem768_dk: None,
                })
            }
        }
    }
}

/// Debug-redacting: the secret bytes MUST NOT leak into logs / panics.
/// `StaticSecret` already redacts, but we print neither half's bytes — only
/// the codepoint + a redaction marker (mirrors `AeadKeyMaterial` /
/// `StructuralKdfKey` hygiene).
impl core::fmt::Debug for RecipientSecret {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RecipientSecret")
            .field("codepoint", &self.codepoint)
            .field("x25519", &"<redacted>")
            .field("mlkem768_dk", &"<redacted>")
            .finish()
    }
}

/// Zeroize-on-drop for the recipient secret. `x25519_dalek::StaticSecret`
/// already zeroizes its own scalar on drop (the `zeroize` feature is
/// enabled in `Cargo.toml`); the ML-KEM-768 decapsulation-key `Vec<u8>` is
/// NOT self-zeroizing, so we wipe it explicitly here — the raw dk bytes are
/// a long-lived at-rest secret (coredump / freed-heap exposure) per the
/// F-08 memory-hygiene contract.
impl Drop for RecipientSecret {
    fn drop(&mut self) {
        if let Some(dk) = self.mlkem768_dk.as_mut() {
            dk.zeroize();
        }
        // `self.x25519: Option<StaticSecret>` zeroizes via StaticSecret's
        // own Drop when this struct's fields drop.
    }
}

/// Wrapped (encapsulated) key material — what the wrap path produces +
/// the unwrap path consumes. Carries the codepoint discriminator + both
/// encapsulated halves + the AEAD-encrypted `k_root` payload.
#[derive(Clone)]
pub struct WrappedKey {
    /// Cipher-suite codepoint discriminator (carried for F-4
    /// dispatch-from-WrappedKey).
    pub codepoint: CipherSuiteCodepoint,
    /// X25519 ephemeral public key (the "ek_x" half).
    pub ek_x: Vec<u8>,
    /// ML-KEM-768 ciphertext (the "ek_mlkem" half). Empty for
    /// `0x6400` classical-only.
    pub ek_mlkem: Vec<u8>,
    /// AEAD-encrypted `k_root` under the X-Wing-combined key.
    pub aead_envelope: AeadEnvelope,
}

impl WrappedKey {
    /// Adversarial helper — strip the ML-KEM-768 half (F-2 enc-side
    /// strip-resistance pin). The unwrap path MUST fail closed.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn without_pq_half_for_test(&self) -> Self {
        let mut out = self.clone();
        // Zero out the ek_mlkem so the unwrap derives a different key
        // (the AEAD decrypt then fails closed). We keep the byte length
        // intact so the per-ML-KEM-ciphertext-length parsing still works
        // — the stripping is in the BYTES not the SHAPE.
        out.ek_mlkem.fill(0u8);
        out
    }

    /// Adversarial helper — strip the classical X25519 half (F-2).
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn without_classical_half_for_test(&self) -> Self {
        let mut out = self.clone();
        out.ek_x.fill(0u8);
        out
    }

    /// Adversarial helper — substitute the codepoint discriminator
    /// (F-4 unknown-codepoint-at-decrypt pin).
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn with_codepoint_for_test(&self, codepoint: u16) -> Self {
        let mut out = self.clone();
        out.codepoint = CipherSuiteCodepoint::from_raw(codepoint);
        out
    }
}

/// Recovered key material from `unwrap_key_material` (Compromise #66).
///
/// The `bytes` are recovered SECRET key material (`k_root`). R19 secret-
/// hygiene: the raw bytes are zeroized on drop and NEVER rendered by
/// `Debug` (redaction marker only) — mirrors the `RecipientSecret` /
/// `AeadKeyMaterial` / `StructuralKdfKey` hygiene in this crate.
pub struct UnwrappedKey {
    bytes: Vec<u8>,
}

impl UnwrappedKey {
    /// The recovered `k_root` bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Debug-redacting: the recovered `k_root` bytes MUST NOT leak into logs /
/// panics (Compromise #66 memory-hygiene). Prints only a redaction marker.
impl core::fmt::Debug for UnwrappedKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UnwrappedKey")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

/// Zeroize-on-drop: the recovered `k_root` `Vec<u8>` is a recovered secret;
/// wipe it explicitly so freed-heap / coredump exposure does not leak it.
impl Drop for UnwrappedKey {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// Recovered plaintext from `open_aead`.
///
/// The `bytes` are recovered SECRET content (the decrypted Node body). R19
/// secret-hygiene: the raw bytes are zeroized on drop and NEVER rendered by
/// `Debug` (redaction marker only) — mirrors the sibling [`UnwrappedKey`] /
/// `RecipientSecret` / `AeadKeyMaterial` / `StructuralKdfKey` hygiene in this
/// crate.
pub struct DecryptedPlaintext {
    bytes: Vec<u8>,
}

impl DecryptedPlaintext {
    /// The recovered plaintext bytes.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}

/// Debug-redacting: recovered plaintext MUST NOT leak into logs / panics.
/// Prints only a redaction marker.
impl core::fmt::Debug for DecryptedPlaintext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DecryptedPlaintext")
            .field("bytes", &"<redacted>")
            .finish()
    }
}

/// Zeroize-on-drop: the recovered plaintext `Vec<u8>` is recovered secret
/// content; wipe it explicitly so freed-heap / coredump exposure does not
/// leak it.
impl Drop for DecryptedPlaintext {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drift defense: the deterministic-recipient-seed expansion label is a
    /// registered cross-surface domain-separation tag in the central
    /// [`crate::domain_registry`] table over which the prefix-free invariant
    /// runs. Pin byte-equality so the mirror can never silently diverge from
    /// the home definition here.
    #[test]
    fn cipher_suite_domain_tags_match_central_registry() {
        use crate::domain_registry as reg;
        assert_eq!(
            RECIPIENT_SEED_LABEL,
            reg::RECIPIENT_SEED_LABEL,
            "RECIPIENT_SEED_LABEL drifted from the central domain_registry mirror"
        );
    }

    #[test]
    fn hybrid_wrap_unwrap_round_trips() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
            .expect("0x647a LIVE at G-CORE-3a");
        let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
        let k_root = [0x42u8; 32];
        let wrapped = suite
            .wrap_key_material(kp.public(), &k_root)
            .expect("wrap MUST succeed");
        let recovered = suite
            .unwrap_key_material(kp.secret(), &wrapped)
            .expect("unwrap MUST succeed");
        assert_eq!(recovered.as_bytes(), &k_root);
    }

    #[test]
    fn classical_wrap_unwrap_round_trips() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::CLASSICAL_X25519)
            .expect("0x6400 LIVE at G-CORE-3a");
        assert!(!suite.is_hybrid_default());
        let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
        let k_root = [0x77u8; 32];
        let wrapped = suite
            .wrap_key_material(kp.public(), &k_root)
            .expect("wrap MUST succeed");
        let recovered = suite
            .unwrap_key_material(kp.secret(), &wrapped)
            .expect("unwrap MUST succeed");
        assert_eq!(recovered.as_bytes(), &k_root);
    }

    #[test]
    fn pq_strip_resistance_fails_closed() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
        let k_root = [0x12u8; 32];
        let wrapped = suite.wrap_key_material(kp.public(), &k_root).unwrap();
        let pq_stripped = wrapped.without_pq_half_for_test();
        let outcome = suite.unwrap_key_material(kp.secret(), &pq_stripped);
        assert!(outcome.is_err(), "stripped PQ half MUST fail closed");
    }

    #[test]
    fn classical_strip_resistance_fails_closed() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
        let k_root = [0x13u8; 32];
        let wrapped = suite.wrap_key_material(kp.public(), &k_root).unwrap();
        let cl_stripped = wrapped.without_classical_half_for_test();
        let outcome = suite.unwrap_key_material(kp.secret(), &cl_stripped);
        assert!(outcome.is_err(), "stripped classical half MUST fail closed");
    }

    #[test]
    fn recipient_lacks_pq_half_typed_error() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let hybrid_kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
        let k_root = [0x34u8; 32];
        let wrapped = suite
            .wrap_key_material(hybrid_kp.public(), &k_root)
            .unwrap();
        let degenerate = RecipientKeypair::with_only_classical_half_for_test(&hybrid_kp);
        let outcome = suite.unwrap_key_material(degenerate.secret(), &wrapped);
        assert!(
            matches!(outcome, Err(AeadError::RecipientLacksKeysForSuite)),
            "degenerate recipient MUST surface RecipientLacksKeysForSuite (got {outcome:?})"
        );
    }

    #[test]
    fn unknown_codepoint_at_decrypt_typed_unsupported() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
        let k_root = [0x56u8; 32];
        let wrapped = suite.wrap_key_material(kp.public(), &k_root).unwrap();
        let bogus = wrapped.with_codepoint_for_test(0xCAFE);
        let outcome = CipherSuite::resolve_for_wrapped_key(&bogus);
        assert!(
            matches!(
                outcome,
                Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0xCAFE })
            ),
            "unknown codepoint at decrypt MUST surface UnsupportedAlgorithm::CipherSuite"
        );
    }

    #[test]
    fn seal_open_aad_binds_plaintext_cid() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let k_root = [0x99u8; 32];
        let pt = b"Node bytes for plaintext_cid_A";
        let cid_a = [0xAAu8; 32];
        let cid_b = [0xBBu8; 32];
        let ct = suite.seal_aead(&k_root, pt, &cid_a).unwrap();
        let rebound = suite.open_aead(&k_root, &ct, &cid_b);
        assert!(rebound.is_err(), "rebinding to cid_b MUST fail closed");
        let legit = suite.open_aead(&k_root, &ct, &cid_a).unwrap();
        assert_eq!(legit.as_slice(), pt);
    }

    #[test]
    fn reserved_codepoint_0x647b_typed_rejects() {
        let outcome = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_MLKEM768_HQC);
        assert!(matches!(
            outcome,
            Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647b })
        ));
    }

    // ---- R9-S1: real-entropy generation + serialization round-trip ----

    /// The named x25519 length constants MUST equal the actual serialized
    /// width of the upstream types (no-hardcoded-sizes discipline — if a
    /// future x25519-dalek changed the wire size this fails, rather than
    /// silently mis-serializing).
    #[test]
    fn x25519_public_len_matches_type_width() {
        let x_sec = StaticSecret::random_from_rng(&mut RandOsRng);
        let x_pub = X25519PublicKey::from(&x_sec);
        assert_eq!(x_pub.as_bytes().len(), X25519_PUBLIC_LEN);
    }

    #[test]
    fn x25519_secret_len_matches_type_width() {
        let x_sec = StaticSecret::random_from_rng(&mut RandOsRng);
        assert_eq!(x_sec.to_bytes().len(), X25519_SECRET_LEN);
    }

    /// REAL-ENTROPY (GAP-1 root): two `generate_recipient_keypair()` calls
    /// yield DISTINCT public AND secret bytes. This test would FAIL if the
    /// generation were deterministic (as the placeholder path was).
    #[test]
    fn generate_recipient_keypair_is_real_entropy_hybrid() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let a = suite.generate_recipient_keypair();
        let b = suite.generate_recipient_keypair();

        // Distinct public material (x25519 + ML-KEM ek).
        assert_ne!(
            a.public().to_bytes(),
            b.public().to_bytes(),
            "two hybrid keygens MUST yield distinct public bytes (real entropy)"
        );
        // Distinct secret material (x25519 + ML-KEM dk).
        assert_ne!(
            a.secret().to_bytes().as_slice(),
            b.secret().to_bytes().as_slice(),
            "two hybrid keygens MUST yield distinct secret bytes (real entropy)"
        );

        // Real hybrid material is present (both halves).
        let pub_len = a.public().to_bytes().len();
        assert_eq!(pub_len, X25519_PUBLIC_LEN + mlkem::ML_KEM_768_EK_LEN);
        let sec_len = a.secret().to_bytes().len();
        assert_eq!(sec_len, X25519_SECRET_LEN + mlkem::ML_KEM_768_DK_LEN);
    }

    #[test]
    fn generate_recipient_keypair_is_real_entropy_classical() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::CLASSICAL_X25519).unwrap();
        let a = suite.generate_recipient_keypair();
        let b = suite.generate_recipient_keypair();
        assert_ne!(a.public().to_bytes(), b.public().to_bytes());
        assert_ne!(
            a.secret().to_bytes().as_slice(),
            b.secret().to_bytes().as_slice()
        );
        // Classical: x25519 half only.
        assert_eq!(a.public().to_bytes().len(), X25519_PUBLIC_LEN);
        assert_eq!(a.secret().to_bytes().len(), X25519_SECRET_LEN);
    }

    /// Real-entropy vs deterministic: a fresh keygen MUST differ from the
    /// public-seed-derived deterministic keypair (the placeholder path had
    /// ZERO secret entropy — this pins the fix).
    #[test]
    fn real_entropy_differs_from_public_seed_deterministic() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let real = suite.generate_recipient_keypair();
        // The placeholder derived the "secret" from a PUBLIC fingerprint seed.
        let public_seed = *real.public().x25519.as_ref().unwrap().as_bytes();
        let forgeable = suite.generate_recipient_keypair_deterministic_for_test(&public_seed);
        assert_ne!(
            real.secret().to_bytes().as_slice(),
            forgeable.secret().to_bytes().as_slice(),
            "real-entropy secret MUST NOT be derivable from a public seed"
        );
    }

    /// ROUND-TRIP: generate → to_bytes → from_bytes → wrap(pub)/unwrap(sec)
    /// recovers the wrapped key material — proving the serialized REAL keys
    /// drive the real KEM end to end (hybrid `0x647a`).
    #[test]
    fn serialized_real_keys_drive_kem_round_trip_hybrid() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let cp = suite.codepoint();
        let kp = suite.generate_recipient_keypair();

        // Serialize both halves, then reconstruct from bytes.
        let pub_bytes = kp.public().to_bytes();
        let sec_bytes = kp.secret().to_bytes();
        let rebuilt_pub = RecipientPublic::from_bytes(cp, &pub_bytes).expect("pub from_bytes");
        let rebuilt_sec = RecipientSecret::from_bytes(cp, &sec_bytes).expect("sec from_bytes");

        // Wrap to the reconstructed public; unwrap with the reconstructed secret.
        let k_root = [0xA5u8; 32];
        let wrapped = suite
            .wrap_key_material(&rebuilt_pub, &k_root)
            .expect("wrap to serialized pub MUST succeed");
        let recovered = suite
            .unwrap_key_material(&rebuilt_sec, &wrapped)
            .expect("unwrap with serialized sec MUST succeed");
        assert_eq!(
            recovered.as_bytes(),
            &k_root,
            "serialized real keys MUST drive the real KEM end to end"
        );
    }

    #[test]
    fn serialized_real_keys_drive_kem_round_trip_classical() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::CLASSICAL_X25519).unwrap();
        let cp = suite.codepoint();
        let kp = suite.generate_recipient_keypair();
        let rebuilt_pub = RecipientPublic::from_bytes(cp, &kp.public().to_bytes()).unwrap();
        let rebuilt_sec = RecipientSecret::from_bytes(cp, &kp.secret().to_bytes()).unwrap();
        let k_root = [0x5Au8; 32];
        let wrapped = suite.wrap_key_material(&rebuilt_pub, &k_root).unwrap();
        let recovered = suite.unwrap_key_material(&rebuilt_sec, &wrapped).unwrap();
        assert_eq!(recovered.as_bytes(), &k_root);
    }

    /// from_bytes rejects malformed / truncated input with the typed error
    /// (fail-closed; never a silent default).
    #[test]
    fn from_bytes_rejects_malformed_public() {
        let cp = CipherSuiteCodepoint::HYBRID_X25519_MLKEM768;
        // Truncated hybrid public.
        let truncated = vec![0u8; X25519_PUBLIC_LEN + mlkem::ML_KEM_768_EK_LEN - 1];
        assert!(matches!(
            RecipientPublic::from_bytes(cp, &truncated),
            Err(AeadError::MalformedRecipientPublic(_))
        ));
        // Empty.
        assert!(matches!(
            RecipientPublic::from_bytes(cp, &[]),
            Err(AeadError::MalformedRecipientPublic(_))
        ));
        // Wrong length for classical.
        assert!(matches!(
            RecipientPublic::from_bytes(CipherSuiteCodepoint::CLASSICAL_X25519, &[0u8; 31]),
            Err(AeadError::MalformedRecipientPublic(_))
        ));
        // Unknown codepoint typed-rejects (never silent default).
        assert!(matches!(
            RecipientPublic::from_bytes(CipherSuiteCodepoint::from_raw(0xCAFE), &[0u8; 32]),
            Err(AeadError::Unsupported(UnsupportedAlgorithm::CipherSuite {
                codepoint: 0xCAFE
            }))
        ));
    }

    #[test]
    fn from_bytes_rejects_malformed_secret() {
        let cp = CipherSuiteCodepoint::HYBRID_X25519_MLKEM768;
        let truncated = vec![0u8; X25519_SECRET_LEN + mlkem::ML_KEM_768_DK_LEN - 1];
        assert!(matches!(
            RecipientSecret::from_bytes(cp, &truncated),
            Err(AeadError::MalformedRecipientSecret(_))
        ));
        assert!(matches!(
            RecipientSecret::from_bytes(cp, &[]),
            Err(AeadError::MalformedRecipientSecret(_))
        ));
        assert!(matches!(
            RecipientSecret::from_bytes(CipherSuiteCodepoint::CLASSICAL_X25519, &[0u8; 33]),
            Err(AeadError::MalformedRecipientSecret(_))
        ));
        assert!(matches!(
            RecipientSecret::from_bytes(CipherSuiteCodepoint::from_raw(0xCAFE), &[0u8; 32]),
            Err(AeadError::Unsupported(UnsupportedAlgorithm::CipherSuite {
                codepoint: 0xCAFE
            }))
        ));
    }

    /// The secret's Debug MUST NOT leak either half's bytes (redaction
    /// hygiene — mirrors AeadKeyMaterial / StructuralKdfKey).
    #[test]
    fn recipient_secret_debug_is_redacted() {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let kp = suite.generate_recipient_keypair();
        let dbg = format!("{:?}", kp.secret());
        assert!(
            dbg.contains("<redacted>"),
            "Debug MUST redact secret halves"
        );
        // The raw dk bytes MUST NOT appear. A leak would render the dk as a
        // decimal byte array (e.g. `[203, 17, ...]`); a redacting Debug is
        // short. Pin both: the redaction marker is present AND the output is
        // far shorter than a full 2400-byte dk dump would be.
        let dk_len = kp.secret().mlkem768_dk.as_ref().map_or(0, Vec::len);
        assert_eq!(dk_len, mlkem::ML_KEM_768_DK_LEN, "hybrid dk present");
        assert!(
            dbg.len() < 128,
            "redacting Debug MUST be short (a leaked {dk_len}-byte dk dump would be far longer): {dbg}"
        );
    }

    #[test]
    fn reserved_codepoint_0x647c_typed_rejects() {
        // Pre-G-CORE-9-FREEZE 2026-05-24: `0x647c` is the named pure-PQ
        // ML-KEM-768-only swap-matrix arm codepoint. At the cipher-suite
        // dispatcher level it MUST typed-reject — the pure-PQ arm is
        // ONLY reachable via the named
        // `SwapMatrix::try_pure_pq_sole_trust_path` constructor (which
        // gates on `AUDIT_LANDED_PURE_PQ_FLAG`). Sibling of the `0x647b`
        // test above; distinct codepoint identity prevents wire-format
        // collision when the AUDIT_LANDED flag flips at v1-GM.
        let outcome = CipherSuite::resolve(CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY);
        assert!(matches!(
            outcome,
            Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647c })
        ));
    }
}
