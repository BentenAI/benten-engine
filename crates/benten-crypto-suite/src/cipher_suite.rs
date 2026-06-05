//! Cipher-suite codepoint dispatch + X-Wing-style hybrid wrap surface
//! (G-CORE-3a CANARY — #1301 substrate).
//!
//! # G-CORE-3a deliverables (this wave)
//!
//! - LIVE codepoint `0x647a` = X25519⊕ML-KEM-768 hybrid KEM
//!   (vendored ~30-LOC X-Wing-style combiner over `ml-kem` +
//!   `x25519-dalek` + `sha3`; per Spike I + CLAUDE.md baked-in #5
//!   + RATIFIED-S&C §1 refinement #6).
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
//! # X-Wing-style combiner (per Spike I)
//!
//! Per `RATIFIED-sharing-and-confidentiality-2026-05-21` Spike-I row:
//! "X-Wing combiner body = 24 LOC; codepoint-dispatch + typed-reject
//! pattern works end-to-end; hybrid wrap ~1.7× slower than classical =
//! non-issue." The combiner is committing across BOTH halves so neither
//! can be stripped without the derive failing.
//!
//! ```text
//! WRAP(recipient_pub):
//!     ek_x = X25519.encapsulate(recipient_pub.x25519)       // X25519 ECDH ek + shared
//!     (ek_mlkem, ss_mlkem) = MLKEM768.encapsulate(recipient_pub.mlkem)
//!     ss_x = X25519.shared_secret(...)
//!     combined = HKDF-SHA256(ss_x || ss_mlkem || ek_x || ek_mlkem ||
//!                            recipient_pub.x25519 || recipient_pub.mlkem,
//!                            info = "x-wing-v1-benten-0x647a")
//!     wrapped = { codepoint: 0x647a, ek_x, ek_mlkem, encrypted_key }
//!
//! UNWRAP(recipient_sec, wrapped):
//!     ss_x = X25519.decapsulate(recipient_sec.x25519, wrapped.ek_x)
//!     ss_mlkem = MLKEM768.decapsulate(recipient_sec.mlkem, wrapped.ek_mlkem)
//!     combined = HKDF-SHA256(ss_x || ss_mlkem || ek_x || ek_mlkem ||
//!                            recipient_pub.x25519 || recipient_pub.mlkem,
//!                            info = "x-wing-v1-benten-0x647a")
//!     decrypt_key = combined
//! ```
//!
//! Strip-resistance: the combiner HKDF input concatenates BOTH shared
//! secrets + BOTH encapsulated-keys + BOTH public-keys. Stripping the
//! ML-KEM-768 half (ek_mlkem = zero) yields a different `combined` →
//! the unwrap derives a different key → the subsequent ChaCha20-Poly1305
//! decrypt fails closed (the F-2 strip-resistance contract).
//!
//! Per CLAUDE.md baked-in #5 / `crypto-agility-contract:6`: this is the
//! ONLY crypto-primitive call site; we wrap vetted upstream
//! `x25519-dalek` + `ml-kem` + `hkdf` + `sha3` crates; we NEVER fork or
//! reimplement primitives.

use ml_kem::kem::{Decapsulate as _, Encapsulate as _};
use ml_kem::{B32, Encoded, EncodedSizeUser as _, KemCore, MlKem768};
use rand_core::OsRng as RandOsRng;
use sha3::Digest as _;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};

use crate::aead::{
    AeadEnvelope, AeadError, AeadKeyMaterial, aad_whole_content, unwrap as aead_unwrap,
    wrap as aead_wrap,
};
pub use crate::codepoint::CipherSuiteCodepoint;
use crate::error::UnsupportedAlgorithm;

/// The real draft-connolly X-Wing `XWingLabel` — the 6 bytes
/// `0x5c2e2f2f5e5c` (ASCII `\.//^\`). **APPENDED** as the trailing suffix
/// of the combiner pre-image per `draft-connolly-cfrg-xwing-kem-10` §6
/// (R4.2-corrected 2026-06-03 — the prepended form is the superseded
/// v01-v02 ordering and would freeze a non-interoperable KEM at the
/// IETF-reserved `0x647A`).
pub const X_WING_LABEL: [u8; 6] = [0x5c, 0x2e, 0x2f, 0x2f, 0x5e, 0x5c];

/// Classical-only `0x6400` combiner domain-separation info string. ASCII;
/// NOT an integer wire/AAD field (m-1: not flagged by the BE scanner).
const X25519_CLASSICAL_INFO_V1: &[u8] = b"x25519-classical-v1-benten-0x6400";

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
                let (mlkem_dk, mlkem_ek) = MlKem768::generate(&mut RandOsRng);
                RecipientKeypair {
                    codepoint: suite.codepoint,
                    public: RecipientPublic {
                        codepoint: suite.codepoint,
                        x25519: Some(x_pub),
                        mlkem768_ek: Some(mlkem_ek.as_bytes().to_vec()),
                    },
                    secret: RecipientSecret {
                        codepoint: suite.codepoint,
                        x25519: Some(x_sec),
                        mlkem768_dk: Some(mlkem_dk.as_bytes().to_vec()),
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

    /// Deterministically derive a recipient keypair from a 32-byte `seed`.
    ///
    /// Both halves are derived from the seed via BLAKE3 domain-separated
    /// expansion (the X25519 `StaticSecret` from one 32-byte block; the
    /// ML-KEM-768 `(d, z)` from two more) so the same seed always yields
    /// the same keypair. Used by the Layer-C drop path to map a stable
    /// recipient pubkey *fingerprint* to a real hybrid keypair without a
    /// keystore round-trip (the `0x647a` hybrid + the `0x6400` classical
    /// downgrade are both supported; other codepoints would have been
    /// rejected by [`Self::resolve`]).
    ///
    /// Per CLAUDE.md baked-in #5 this stays the ONLY crypto-primitive call
    /// site — the seed expansion goes through the vetted `blake3` MAC and
    /// the keys through `x25519-dalek` / `ml-kem`; no primitive is forked.
    #[must_use]
    pub fn generate_recipient_keypair_deterministic(&self, seed: &[u8; 32]) -> RecipientKeypair {
        // Domain-separated expansion of the seed into the three 32-byte
        // blocks the two key halves need.
        let block = |tag: u8| -> [u8; 32] {
            let mut h = blake3::Hasher::new();
            h.update(b"benten-crypto-suite:recipient-seed");
            h.update(&[tag]);
            h.update(seed);
            *h.finalize().as_bytes()
        };
        let x_block = block(0x01);
        let x_sec = StaticSecret::from(x_block);
        let x_pub = X25519PublicKey::from(&x_sec);
        match self.codepoint.raw() {
            0x647a => {
                let d: B32 = block(0x02).into();
                let z: B32 = block(0x03).into();
                let (mlkem_dk, mlkem_ek) = MlKem768::generate_deterministic(&d, &z);
                RecipientKeypair {
                    codepoint: self.codepoint,
                    public: RecipientPublic {
                        codepoint: self.codepoint,
                        x25519: Some(x_pub),
                        mlkem768_ek: Some(mlkem_ek.as_bytes().to_vec()),
                    },
                    secret: RecipientSecret {
                        codepoint: self.codepoint,
                        x25519: Some(x_sec),
                        mlkem768_dk: Some(mlkem_dk.as_bytes().to_vec()),
                    },
                }
            }
            // Classical-only `0x6400`: X25519 half only.
            _ => RecipientKeypair {
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

                // ML-KEM-768 encapsulation → (ek_mlkem, ss_mlkem).
                let mlkem_ek_array: Encoded<<MlKem768 as KemCore>::EncapsulationKey> = Encoded::<
                    <MlKem768 as KemCore>::EncapsulationKey,
                >::try_from(
                    mlkem_ek_bytes.as_slice(),
                )
                .map_err(|_| AeadError::MalformedEnvelope("recipient ML-KEM-768 ek malformed"))?;
                let mlkem_ek = <MlKem768 as KemCore>::EncapsulationKey::from_bytes(&mlkem_ek_array);
                let (mlkem_ct, ss_mlkem) = mlkem_ek
                    .encapsulate(&mut RandOsRng)
                    .map_err(|()| AeadError::MalformedEnvelope("ML-KEM-768 encapsulate"))?;

                // Real draft-connolly X-Wing combiner:
                // SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel).
                // ss_M = ML-KEM-768 shared secret; ss_X = X25519 shared
                // secret; ct_X = the X25519 ephemeral public key (ek_x);
                // pk_X = the recipient X25519 public key. The ML-KEM
                // ciphertext is bound transitively via ss_M (decapsulation).
                let _ = &mlkem_ct;
                let _ = mlkem_ek_bytes;
                let combined = combine_x_wing(
                    ss_mlkem.as_slice(),
                    ss_x.as_bytes(),
                    ek_x.as_bytes(),
                    x_recipient.as_bytes(),
                );

                // AEAD-encrypt k_root under the combined key.
                let combined_key = AeadKeyMaterial::from_bytes(self.codepoint, combined.to_vec());
                let aad = aad_whole_content(b"x-wing-wrap:k_root");
                let env = aead_wrap(k_root, &combined_key, &aad)?;

                Ok(WrappedKey {
                    codepoint: self.codepoint,
                    ek_x: ek_x.as_bytes().to_vec(),
                    ek_mlkem: mlkem_ct[..].to_vec(),
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

                // HKDF-SHA256 over (ss_x || ek_x || recipient_pub) — same
                // shape as the hybrid but with the classical-only inputs.
                let combined =
                    classical_combine(ss_x.as_bytes(), ek_x.as_bytes(), x_recipient.as_bytes());
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

                // Decapsulate ML-KEM-768 half.
                let mlkem_dk_array: Encoded<<MlKem768 as KemCore>::DecapsulationKey> = Encoded::<
                    <MlKem768 as KemCore>::DecapsulationKey,
                >::try_from(
                    mlkem_dk_bytes.as_slice(),
                )
                .map_err(|_| AeadError::MalformedEnvelope("recipient ML-KEM-768 dk malformed"))?;
                let mlkem_dk = <MlKem768 as KemCore>::DecapsulationKey::from_bytes(&mlkem_dk_array);
                let mlkem_ct_array =
                    ml_kem::Ciphertext::<MlKem768>::try_from(wrapped.ek_mlkem.as_slice())
                        .map_err(|_| AeadError::MalformedEnvelope("ML-KEM-768 ct malformed"))?;
                let ss_mlkem = mlkem_dk
                    .decapsulate(&mlkem_ct_array)
                    .map_err(|()| AeadError::MalformedEnvelope("ML-KEM-768 decapsulate"))?;

                // Recover the recipient's public material to feed the
                // combiner symmetrically (it bound them at wrap-time).
                let x_pub = X25519PublicKey::from(x_sec);

                let combined = combine_x_wing(
                    ss_mlkem.as_slice(),
                    ss_x.as_bytes(),
                    &ek_x_bytes,
                    x_pub.as_bytes(),
                );

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
                let combined = classical_combine(ss_x.as_bytes(), &ek_x_bytes, x_pub.as_bytes());
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
/// §6 (R4.2-corrected). `ss_M` = ML-KEM-768 shared secret, `ss_X` = X25519
/// shared secret, `ct_X` = the X25519 ephemeral public key (the X25519
/// "ciphertext"), `pk_X` = the recipient X25519 public key.
///
/// Exposed so the F-W0-1-LABEL construction-order witness pin can assert the
/// label is the appended suffix (and NOT a prepended prefix).
#[must_use]
pub fn x_wing_combiner_preimage(ss_m: &[u8], ss_x: &[u8], ct_x: &[u8], pk_x: &[u8]) -> Vec<u8> {
    let mut pre = Vec::with_capacity(ss_m.len() + ss_x.len() + ct_x.len() + pk_x.len() + 6);
    pre.extend_from_slice(ss_m);
    pre.extend_from_slice(ss_x);
    pre.extend_from_slice(ct_x);
    pre.extend_from_slice(pk_x);
    // APPENDED suffix (draft-connolly §6; NOT prepended).
    pre.extend_from_slice(&X_WING_LABEL);
    pre
}

/// The real draft-connolly X-Wing combiner:
/// `SHA3-256(ss_M ‖ ss_X ‖ ct_X ‖ pk_X ‖ XWingLabel)`.
///
/// This replaces the prior HKDF-SHA256 mislabel (`cipher_suite.rs:404` at
/// the corpus base) with the IETF-faithful construction at the IETF-reserved
/// codepoint `0x647A`. Per CLAUDE.md baked-in #5 the SHA3-256 primitive is
/// wrapped from the vetted upstream `sha3` crate — no reimplementation.
///
/// Argument order is `(ss_mlkem, ss_x25519, ct_x25519, pk_x25519)` to mirror
/// the spec pre-image `ss_M ‖ ss_X ‖ ct_X ‖ pk_X`.
#[must_use]
pub fn combine_x_wing(ss_mlkem: &[u8], ss_x25519: &[u8], ct_x25519: &[u8], pk_x25519: &[u8]) -> [u8; 32] {
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
#[must_use]
pub fn classical_combine(ss_x: &[u8], ek_x: &[u8], pub_x: &[u8]) -> [u8; 32] {
    let mut pre = Vec::with_capacity(ss_x.len() + ek_x.len() + pub_x.len() + X25519_CLASSICAL_INFO_V1.len());
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

/// Recovered key material from `unwrap_key_material`.
#[derive(Debug)]
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

/// Recovered plaintext from `open_aead`.
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

#[cfg(test)]
mod tests {
    use super::*;

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
