//! Ed25519 keypair primitive with secret-bytes hygiene.
//!
//! ## Crypto-blocker-1 contract (BLOCKER)
//!
//! - [`SecretKey`] derives `Zeroize + ZeroizeOnDrop` so secret bytes
//!   are scrubbed on drop.
//! - [`SecretKey`] does NOT implement [`Clone`] — secret bytes cannot
//!   be silently duplicated outside the original lifetime. (Future
//!   needs that look like cloning go through `Keypair::export_seed_envelope`
//!   + `Keypair::from_seed_bytes`, which forces an explicit envelope-
//!   shaped serialize/deserialize.)
//! - [`SecretKey`]'s [`std::fmt::Debug`] impl prints
//!   `"SecretKey([REDACTED 32 bytes])"` — never the bytes themselves.
//!   Defends against accidental `tracing::error!("{:?}")` /
//!   panic-print / structured-log leak.
//!
//! ## Crypto-major-2 contract
//!
//! [`Keypair::generate`] is pinned to OS CSPRNG via `rand_core::OsRng`
//! (which itself routes to `getrandom`). Never a deterministic seed.
//!
//! ## Crypto-major-5 contract
//!
//! [`Keypair::from_seed_bytes`] / [`Keypair::from_dag_cbor_envelope`]
//! consume a DAG-CBOR envelope of shape
//! `{version: u8, alg: "Ed25519", secret_bytes: Bytes(32)}`. Each
//! failure mode (short / long / corrupted / unknown-version-tag /
//! unknown-alg) returns a DISTINCT [`SeedImportError`] variant. The
//! import path emits NO `tracing` events containing secret bytes.

use core::fmt;

pub use ed25519_dalek::Signature;

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::errors::{KeypairError, SeedImportError};

/// Current envelope version (per `crypto-major-5`). Future schema
/// changes bump this; older verifiers reject newer versions with
/// [`SeedImportError::UnknownVersion`].
pub const ENVELOPE_VERSION: u8 = 1;

/// Algorithm tag (per `crypto-major-5`). Other algorithms (post-Phase-3
/// MultiSig extension) get their own tag; older verifiers reject
/// unknown tags with [`SeedImportError::UnknownAlg`].
pub const ENVELOPE_ALG: &str = "Ed25519";

/// Minimum/maximum envelope size for the import path's structural
/// pre-check. The DAG-CBOR encoding of
/// `{version: 1, alg: "Ed25519", secret_bytes: <32 bytes>}` is
/// approximately 50-55 bytes; we keep generous bounds so future schema
/// extensions (e.g. additional metadata fields) stay backward-
/// compatible. Per `crypto-major-5`'s "fuzz the import path
/// end-to-end" requirement, the pre-check rejects pathologically
/// short/long input fast (typed error) before invoking the CBOR
/// decoder.
const MIN_ENVELOPE_BYTES: usize = 32;
const MAX_ENVELOPE_BYTES: usize = 256;

/// 32-byte Ed25519 secret seed wrapper.
///
/// **Does NOT implement `Clone`.** Per `crypto-blocker-1`, the
/// secret bytes cannot be silently duplicated outside the original
/// lifetime; future needs that look like cloning go through the
/// [`Keypair::export_seed_envelope`] / [`Keypair::from_seed_bytes`]
/// envelope path, which forces an explicit serialize-deserialize
/// boundary that operators can audit.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey {
    bytes: [u8; 32],
}

impl SecretKey {
    /// Construct from raw 32 bytes. Caller must ensure the bytes came
    /// from a CSPRNG; we do not re-randomize.
    fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    // Hyg-1 #308: `bytes_ptr_for_test` removed — it had ZERO callers
    // anywhere (including the test suite; the zeroize-on-drop pin uses
    // `secret_bytes_for_test()` instead). Speculative test-accessor
    // surface that never grew a caller (CLAUDE.md #5 / META #355).

    /// Test-only accessor for hex-comparison in
    /// `crates/benten-id/tests/keypair.rs::keypair_secret_redacted_from_debug_display`.
    /// (Caller is responsible for not leaking the returned slice; this
    /// is gated by the `pub(crate)` visibility on the underlying
    /// bytes accessor at the impl level; the function is exposed
    /// `#[doc(hidden)]` for test access only.)
    #[doc(hidden)]
    pub fn bytes_for_test(&self) -> [u8; 32] {
        self.bytes
    }
}

/// Custom `Debug` impl per `crypto-blocker-1`. NEVER prints the
/// secret bytes — only a redaction sentinel.
impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretKey([REDACTED 32 bytes])")
    }
}

/// Ed25519 public key wrapper. The bytes are deterministic from the
/// secret key; carrying the verifying key alongside the signing key
/// avoids re-deriving on every `verify` call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey {
    inner: VerifyingKey,
}

impl PublicKey {
    /// Construct from raw 32 bytes (W3C did:key resolution path
    /// reaches us here). Returns `None` if the bytes do not form a
    /// valid Edwards point.
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        VerifyingKey::from_bytes(bytes)
            .ok()
            .map(|inner| Self { inner })
    }

    /// 32-byte canonical encoding.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }

    /// Verify a signature against this public key.
    pub fn verify(&self, msg: &[u8], sig: &Signature) -> Result<(), ed25519_dalek::SignatureError> {
        self.inner.verify(msg, sig)
    }

    /// Borrow the underlying `ed25519_dalek::VerifyingKey` for
    /// downstream interop.
    pub fn as_verifying_key(&self) -> &VerifyingKey {
        &self.inner
    }

    /// Resolve to `did:key:z<...>` form (per `crypto-minor-3`).
    pub fn to_did(&self) -> crate::did::Did {
        crate::did::Did::from_public_key(self)
    }
}

/// Ed25519 keypair = secret + cached verifying key.
///
/// Construct via [`Keypair::generate`] (CSPRNG path; `crypto-major-2`)
/// or [`Keypair::from_seed_bytes`] / [`Keypair::from_dag_cbor_envelope`]
/// (envelope-import path; `crypto-major-5`).
///
/// **Does NOT implement [`Clone`].** Per `crypto-blocker-1`, the
/// secret bytes cannot be silently duplicated. Re-issue via
/// [`Keypair::export_seed_envelope`] + [`Keypair::from_seed_bytes`]
/// for the audit-trail-shaped path.
pub struct Keypair {
    /// 32-byte secret seed, zeroize-on-drop.
    secret: SecretKey,
    /// `ed25519-dalek::SigningKey` rebuilt from the secret seed.
    /// Carrying it avoids re-deriving on every `sign` call but the
    /// authoritative material is `secret.bytes` (which Drop scrubs).
    signing: SigningKey,
    /// Cached verifying key.
    verifying: PublicKey,
}

impl Keypair {
    /// Generate a fresh keypair from the OS CSPRNG (`OsRng`).
    ///
    /// Per `crypto-major-2`, this path is pinned to `OsRng` /
    /// `getrandom` — NEVER a deterministic seed (which would generate
    /// identical keypairs on every cold start, an authentication
    /// catastrophe).
    pub fn generate() -> Self {
        // Source-cite anchor for `crates/benten-id/tests/keypair.rs::keypair_generate_uses_os_csprng`
        // (call-site grep test). The `OsRng` path here routes to
        // `getrandom` per the rand_core 0.6 implementation.
        let signing = SigningKey::generate(&mut OsRng);
        let secret_bytes = signing.to_bytes();
        let verifying = PublicKey {
            inner: signing.verifying_key(),
        };
        Self {
            secret: SecretKey::from_bytes(secret_bytes),
            signing,
            verifying,
        }
    }

    /// Sign a message with this keypair's secret.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing.sign(msg)
    }

    /// Borrow this keypair's public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.verifying
    }

    // Hyg-1 #306: `Keypair::secret()` (pub(crate)) removed — zero
    // crate-internal callers. The audit-trail-shaped external path is
    // `Keypair::export_seed_envelope`; test inspection goes through
    // `secret_bytes_for_test`. No SemVer impact (was already
    // crate-private). CLAUDE.md #5.

    /// Test-only accessor mirroring [`SecretKey::bytes_for_test`] for
    /// `crates/benten-id/tests/keypair.rs::keypair_secret_redacted_from_debug_display`.
    #[doc(hidden)]
    pub fn secret_bytes_for_test(&self) -> [u8; 32] {
        self.secret.bytes_for_test()
    }

    /// G21-T2 fp-mini-review MAJOR-6 closure (option (b)) — production
    /// alias of [`Self::secret_bytes_for_test`] with a name that
    /// reflects the lack of zeroize-on-drop on the returned value.
    ///
    /// The returned `[u8; 32]` is a stack-allocated array. Caller is
    /// responsible for either:
    ///   - Wrapping in [`zeroize::Zeroizing`] if the bytes are held
    ///     past the immediate dispatch return.
    ///   - Copying into a `Value::Bytes` Vec that flows out a
    ///     production napi boundary (today; phase-3-backlog §2.5 (e)
    ///     names the proper `Value::SensitiveBytes` discriminant
    ///     extension).
    ///
    /// Use sites that accept the unprotected contract today:
    ///   - `crates/benten-engine/src/typed_call_dispatch.rs::keypair_generate`
    ///     + `keypair_from_seed` — the typed-CALL output schema
    ///     surfaces raw private-key bytes in a `Value::Bytes` wrapper.
    ///     Per phase-3-backlog §2.5 (e) the bytes will move to a
    ///     zeroize-on-drop wrapper once the Value enum extension lands.
    ///   - `crates/benten-sync/src/transport.rs` +
    ///     `crates/benten-sync/src/peer_discovery.rs` — iroh transport
    ///     keypair material for the peer-discovery handshake.
    #[must_use]
    pub fn secret_bytes_unprotected(&self) -> [u8; 32] {
        self.secret.bytes_for_test()
    }

    /// Export this keypair as a canonical DAG-CBOR envelope per
    /// `crypto-major-5`:
    ///
    /// ```text
    /// {version: 1, alg: "Ed25519", secret_bytes: Bytes(32)}
    /// ```
    ///
    /// Round-trips byte-identical through
    /// [`Keypair::from_dag_cbor_envelope`].
    pub fn export_seed_envelope(&self) -> Vec<u8> {
        let envelope = SeedEnvelope {
            version: ENVELOPE_VERSION,
            alg: ENVELOPE_ALG.to_string(),
            secret_bytes: serde_bytes::ByteBuf::from(self.secret.bytes.to_vec()),
        };
        // serde_ipld_dagcbor canonical encoding is deterministic by
        // construction (DAG-CBOR's RFC 8949 §4.2.1 deterministic
        // encoding rules + sorted-key requirement). Two exports of
        // the same envelope yield byte-identical output.
        serde_ipld_dagcbor::to_vec(&envelope)
            .expect("DAG-CBOR encoding of fixed-shape envelope cannot fail")
    }

    /// Import a keypair from a DAG-CBOR envelope per `crypto-major-5`.
    ///
    /// Each failure mode returns a DISTINCT [`SeedImportError`]
    /// variant; the test fleet at `crates/benten-id/tests/keypair_seed.rs` pins this
    /// contract. The path emits NO `tracing` events containing
    /// secret bytes (the only `tracing` site in this crate is on
    /// the OK arm of [`Keypair::generate`] which logs only the
    /// public key, not the secret).
    pub fn from_seed_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        Self::from_seed_bytes_inner(bytes).map_err(KeypairError::SeedImport)
    }

    /// Alias of [`Keypair::from_seed_bytes`] preserved for
    /// device-mesh exploration brief naming.
    pub fn from_dag_cbor_envelope(bytes: &[u8]) -> Result<Self, KeypairError> {
        Self::from_seed_bytes(bytes)
    }

    fn from_seed_bytes_inner(bytes: &[u8]) -> Result<Self, SeedImportError> {
        // Pre-check: short/long input rejected fast with typed
        // variant — defends against pathological inputs hitting the
        // CBOR decoder.
        if bytes.len() < MIN_ENVELOPE_BYTES {
            return Err(SeedImportError::ShortInput {
                got: bytes.len(),
                min: MIN_ENVELOPE_BYTES,
            });
        }
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(SeedImportError::LongInput {
                got: bytes.len(),
                max: MAX_ENVELOPE_BYTES,
            });
        }

        let envelope: SeedEnvelope = serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|_| SeedImportError::EnvelopeMalformed)?;

        if envelope.version != ENVELOPE_VERSION {
            return Err(SeedImportError::UnknownVersion {
                version: envelope.version,
            });
        }

        if envelope.alg != ENVELOPE_ALG {
            return Err(SeedImportError::UnknownAlg { alg: envelope.alg });
        }

        let bytes = envelope.secret_bytes.as_ref();
        if bytes.len() != 32 {
            return Err(SeedImportError::EnvelopeMalformed);
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(bytes);

        // ed25519-dalek 2.x accepts any 32 bytes as a SigningKey seed
        // (the algorithm itself rejects no 32-byte input — invalid
        // points only arise on the verifying side). The `InvalidSecret`
        // variant exists for future algorithm extensions.
        let signing = SigningKey::from_bytes(&seed);
        let verifying = PublicKey {
            inner: signing.verifying_key(),
        };

        Ok(Self {
            secret: SecretKey::from_bytes(seed),
            signing,
            verifying,
        })
    }
}

// Per `crypto-blocker-1`, do NOT implement Clone for Keypair. The
// public-key derivation path is `keypair.public_key().clone()` if a
// caller needs an owned PublicKey; that's `Clone` on the public side
// only. Cloning the keypair (and therefore the secret) is forbidden.

/// Custom redacted `Debug` for `Keypair` per `crypto-blocker-1`. The
/// public key is fine to print (it's, well, public); the secret half
/// is wrapped in `SecretKey` whose own Debug emits the redaction
/// sentinel. This impl exists so callers can `format!("{:?}", kp)` on
/// a Keypair without leaking secret bytes — a common shape in error
/// paths (`unwrap_err()` requires `T: Debug` so the production path
/// flows through Debug on every Result-returning entry point).
impl fmt::Debug for Keypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `signing` is intentionally omitted — it is rebuilt from
        // `secret.bytes` and printing it via ed25519-dalek's Debug
        // would expose the same secret material the SecretKey
        // wrapper goes out of its way to redact. Calling
        // `finish_non_exhaustive` is the documented escape hatch.
        f.debug_struct("Keypair")
            .field("public_key", &self.verifying)
            .field("secret", &self.secret)
            .finish_non_exhaustive()
    }
}

/// DAG-CBOR envelope schema per `crypto-major-5`.
///
/// Field order matters for canonical-bytes stability — DAG-CBOR's
/// deterministic encoding sorts map keys, so the on-wire bytes are
/// stable across encoders. We rely on serde's `derive(Serialize)`
/// emitting the fields in declaration order; the canonical-bytes
/// round-trip test
/// (`crates/benten-id/tests/keypair_seed.rs::keypair_from_dag_cbor_envelope_round_trip`)
/// pins this.
#[derive(Serialize, Deserialize)]
struct SeedEnvelope {
    version: u8,
    alg: String,
    secret_bytes: serde_bytes::ByteBuf,
}
