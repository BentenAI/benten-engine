//! Typed error variants for the `benten-id` surface.
//!
//! Per `crypto-major-5`, the seed-import path returns typed variants
//! (NEVER a generic `dyn Error` / `String`) so callers can branch on
//! the failure mode (short input / long input / corrupted bytes /
//! unknown version tag / envelope malformed / invalid secret).

use thiserror::Error;

/// Errors emitted by [`crate::keypair::Keypair`] construction paths.
#[derive(Debug, Error)]
pub enum KeypairError {
    /// The seed-import path failed.
    #[error("seed import failed: {0}")]
    SeedImport(#[from] SeedImportError),
    /// The OS CSPRNG returned an error during `Keypair::generate`.
    /// Per `crypto-major-2` we surface this rather than panic so
    /// hosts that do not provide a CSPRNG (extremely unusual but
    /// theoretically possible — embedded targets, sandboxed wasm
    /// without `getrandom` polyfill) can detect + recover.
    #[error("OS CSPRNG unavailable: {0}")]
    Csprng(String),
}

/// Typed-error variants for the seed-import path.
///
/// Each variant pins a SEPARATE failure mode, per `crypto-major-5`'s
/// "fuzz the import path end-to-end" requirement. The
/// `tests/keypair_seed.rs` test pins assert that each variant fires
/// for its corresponding adversarial input.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SeedImportError {
    /// Input too short to contain a valid envelope.
    #[error("seed envelope too short: got {got} bytes, expected at least {min}")]
    ShortInput {
        /// Bytes received.
        got: usize,
        /// Minimum bytes for a structurally-valid envelope.
        min: usize,
    },
    /// Input longer than any structurally-valid envelope; defends
    /// against length-extension / payload-stuffing.
    #[error("seed envelope too long: got {got} bytes, expected at most {max}")]
    LongInput {
        /// Bytes received.
        got: usize,
        /// Maximum bytes for a structurally-valid envelope.
        max: usize,
    },
    /// DAG-CBOR decoder failed (bit-flip inside Bytes(32) field, type
    /// tag mismatch, length-prefix corruption, etc.).
    #[error("seed envelope malformed (DAG-CBOR decode failed)")]
    EnvelopeMalformed,
    /// Envelope version tag does not match a known version. Forward-
    /// incompatibility is intentional: silent acceptance would let an
    /// attacker mint envelopes that older verifiers mis-parse.
    #[error("seed envelope unknown version: {version}")]
    UnknownVersion {
        /// The unrecognized version tag.
        version: u8,
    },
    /// Envelope alg field does not match the expected algorithm.
    #[error("seed envelope unknown alg: {alg}")]
    UnknownAlg {
        /// The unrecognized algorithm string.
        alg: String,
    },
    /// Secret bytes are structurally valid CBOR but do not produce a
    /// valid Ed25519 secret. (`ed25519-dalek` accepts any 32 bytes as
    /// a SigningKey seed, so this variant is reserved for future
    /// algorithm extensions; included now to keep the typed-error
    /// surface stable across the version-tag bump.)
    #[error("seed envelope contains invalid secret bytes")]
    InvalidSecret,
}

/// Errors emitted by [`crate::did::Did`] resolution paths.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DidError {
    /// DID string does not start with `did:key:z` (W3C did:key spec
    /// requires multibase prefix `z` for base58btc).
    #[error("did:key MUST use 'z' multibase prefix per W3C spec; got: {0}")]
    InvalidPrefix(String),
    /// Base58btc decode of the DID body failed.
    #[error("did:key body base58btc-decode failed")]
    Base58Decode,
    /// Decoded body too short to contain the multicodec prefix +
    /// 32-byte Ed25519 public key.
    #[error("did:key body too short: got {got} bytes, expected at least {min}")]
    BodyTooShort {
        /// Bytes after base58btc decode.
        got: usize,
        /// Minimum bytes for a structurally-valid Ed25519 did:key body.
        min: usize,
    },
    /// Multicodec prefix is not `0xed 0x01` (the Ed25519 varint).
    #[error(
        "did:key multicodec MUST be 0xed01 (Ed25519 varint) per W3C spec; got: {0:#04x} {1:#04x}"
    )]
    UnknownMulticodec(u8, u8),
    /// `VerifyingKey::from_bytes` rejected the 32 pubkey bytes (not a
    /// valid Edwards point).
    #[error("did:key body holds invalid Ed25519 public key bytes")]
    InvalidPublicKey,
}

/// Errors emitted by [`crate::ucan`] chain-walk validation.
///
/// Per `crypto-blocker-2` BLOCKER, `nbf` and `exp` enforcement happens
/// at chain-walk site (every link in the chain), not just on the leaf.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum UcanError {
    /// Token presented before its `nbf` (not-before) field.
    #[error("UCAN not yet valid: nbf={nbf} > now={now}")]
    NotYetValid {
        /// Token's `nbf` epoch seconds.
        nbf: u64,
        /// Validation time in epoch seconds.
        now: u64,
    },
    /// Token presented after its `exp` (expiration) field.
    #[error("UCAN expired: exp={exp} < now={now}")]
    Expired {
        /// Token's `exp` epoch seconds.
        exp: u64,
        /// Validation time in epoch seconds.
        now: u64,
    },
    /// Audience-DID does not match the validation context's expected
    /// audience. Defends against cross-atrium replay (a UCAN issued
    /// to atrium A replayed at atrium B).
    #[error("UCAN audience mismatch: token aud {token_aud} != expected {expected}")]
    AudienceMismatch {
        /// The audience the token names.
        token_aud: String,
        /// The audience the validator expects.
        expected: String,
    },
    /// Signature verification failed at one of the chain links.
    /// Comparison is done via `subtle::ConstantTimeEq` per
    /// `crypto-major-4` (see `crate::ucan::ct_signature_eq`).
    #[error("UCAN signature verification failed at chain link {link_index}")]
    BadSignature {
        /// Chain index (0 = leaf) where verification failed.
        link_index: usize,
    },
    /// Issuer-audience binding violated between adjacent chain links.
    /// Each layer's `aud` MUST equal the next layer's `iss` so the
    /// chain forms a coherent delegation path.
    #[error("UCAN chain link {link_index} aud {aud} != next link iss {next_iss}")]
    ChainLinkBroken {
        /// Chain index of the parent link.
        link_index: usize,
        /// Parent's audience DID.
        aud: String,
        /// Child's issuer DID.
        next_iss: String,
    },
    /// Child grants wider authority than parent. The attenuation
    /// contract: a delegated UCAN MUST NOT widen the authority of its
    /// parent.
    #[error(
        "UCAN attenuation violated at chain link {link_index}: child grants {child_cap} but parent only grants {parent_caps:?}"
    )]
    AttenuationViolated {
        /// Chain index of the offending child.
        link_index: usize,
        /// The child's offending capability.
        child_cap: String,
        /// The set of parent capabilities for diagnostics.
        parent_caps: Vec<String>,
    },
    /// Empty chain.
    #[error("UCAN chain is empty")]
    EmptyChain,
    /// Token CBOR could not be decoded.
    #[error("UCAN token decode failed")]
    DecodeFailed,
}
