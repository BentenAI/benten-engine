//! Typed error variants for the `benten-id` surface.
//!
//! Per `crypto-major-5`, the seed-import path returns typed variants
//! (NEVER a generic `dyn Error` / `String`) so callers can branch on
//! the failure mode (short input / long input / corrupted bytes /
//! unknown version tag / envelope malformed / invalid secret).

use core::fmt::Write as _;

use thiserror::Error;

/// Maximum rendered length of an untrusted DID / capability string
/// inside an error `Display` (safe-2 #555).
const MAX_RENDERED_UNTRUSTED_LEN: usize = 96;

/// Sanitize an untrusted, attacker-influenceable string for inclusion
/// in an operator-facing error `Display` / `Debug` (safe-2 #555).
///
/// `UcanError` variants such as `AudienceMismatch` / `ChainLinkBroken`
/// carry `aud` / `iss` fields lifted verbatim from a deserialized,
/// caller-controlled `UcanClaims`. Those strings reach `Display`,
/// `Debug`, and structured-log sinks before any DID-shape gate fires
/// (the signature gate is the load-bearing assertion, but it runs
/// AFTER these error values can be constructed). An adversarial chain
/// whose `aud` is 100 KB of bytes — or embeds control characters /
/// newlines — would otherwise propagate into operator logs
/// unredacted (log-injection / log-flooding shape).
///
/// This renders non-printable / non-ASCII bytes as `\xNN`, collapses
/// the result to at most [`MAX_RENDERED_UNTRUSTED_LEN`] chars, and
/// appends a truncation marker carrying the original byte length so
/// the operator still sees that *something* oversized arrived without
/// the raw bytes hitting the log.
pub(crate) fn sanitize_untrusted(s: &str) -> String {
    let mut out = String::with_capacity(s.len().min(MAX_RENDERED_UNTRUSTED_LEN) + 16);
    let mut escaped = false;
    for ch in s.chars().take(MAX_RENDERED_UNTRUSTED_LEN) {
        if ch.is_ascii_graphic() || ch == ' ' {
            out.push(ch);
        } else {
            // Render every non-printable / non-ASCII char as an
            // escaped byte sequence so newlines / NUL / control bytes
            // cannot inject into log lines.
            escaped = true;
            let mut buf = [0u8; 4];
            for b in ch.encode_utf8(&mut buf).as_bytes() {
                // Infallible: writing to a String never errors.
                let _ = write!(out, "\\x{b:02x}");
            }
        }
    }
    let truncated = s.chars().count() > MAX_RENDERED_UNTRUSTED_LEN;
    if truncated || escaped {
        let _ = write!(out, "…<{} bytes total>", s.len());
    }
    out
}

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
/// `crates/benten-id/tests/keypair_seed.rs` test pins assert that each variant fires
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
    ///
    /// **Qual-1 #725 — DISAGREE-WITH-EXPLANATION (HARD RULE 12 (c)).**
    /// The reserved-but-currently-unreachable arm is intentional
    /// forward-stable error-taxonomy design (documented above) and is
    /// referenced by the `keypair_seed.rs` typed-rejection match arm,
    /// so it is not orphaned dead code. Deleting it now would force a
    /// SemVer-affecting error-enum change the moment a non-Ed25519
    /// algorithm lands. The "dead-arm coverage" concern is satisfied
    /// by the test's `matches!` over the full variant set.
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
    #[error(
        "UCAN audience mismatch: token aud {} != expected {}",
        sanitize_untrusted(.token_aud),
        sanitize_untrusted(.expected)
    )]
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
    #[error(
        "UCAN chain link {link_index} aud {} != next link iss {}",
        sanitize_untrusted(.aud),
        sanitize_untrusted(.next_iss)
    )]
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
        "UCAN attenuation violated at chain link {link_index}: child grants {} but parent only grants {parent_caps:?}",
        sanitize_untrusted(.child_cap)
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
    /// The CBOR nesting depth of an untrusted-input `Ucan` blob
    /// exceeded [`crate::ucan::MAX_UCAN_PROOF_DEPTH`]. `Ucan::prf` is
    /// a directly-recursive proof-chain field; an adversarial blob can
    /// nest it arbitrarily deep, and `serde`'s derived recursive
    /// deserialize routine consumes one stack frame per level — a
    /// stack-overflow DoS. Rejected at the byte boundary BEFORE serde
    /// is invoked (per safe-2 #549). `depth` is the observed nesting
    /// at the point the bound was tripped; `max` is the configured
    /// ceiling.
    #[error("UCAN proof chain too deep: depth={depth} exceeds max={max}")]
    ProofChainTooDeep {
        /// Observed CBOR container-nesting depth when the bound tripped.
        depth: usize,
        /// The configured maximum.
        max: usize,
    },
    /// Issuer keypair has been rotated; post-rotation UCANs reject
    /// per `crypto-major-3`.
    #[error("UCAN issuer keypair superseded by rotation: issuer={}", sanitize_untrusted(.issuer))]
    IssuerKeypairSuperseded {
        /// Superseded issuer DID.
        issuer: String,
    },
    /// Issuer device has been revoked by its parent DID per
    /// `crypto-major-6`.
    #[error("UCAN issuer device revoked: issuer={}", sanitize_untrusted(.issuer))]
    IssuerDeviceRevoked {
        /// Revoked device DID.
        issuer: String,
    },
    /// Device envelope does not authorize the capability the UCAN
    /// grants (e.g. `host:sandbox:exec` from a device whose
    /// envelope says `runs_sandbox=false`).
    #[error(
        "UCAN device envelope violated: issuer={} cap={}",
        sanitize_untrusted(.issuer),
        sanitize_untrusted(.cap)
    )]
    DeviceEnvelopeViolated {
        /// Issuer DID.
        issuer: String,
        /// Offending capability.
        cap: String,
    },
    /// Leaf token's `att` array does not grant the required capability.
    /// Defends against the typed-CALL `ucan_validate_chain` op accepting
    /// a structurally-sound chain bound to the right audience but
    /// lacking the requested `(resource, ability)` claim — a
    /// defense-in-depth gap that would otherwise let a handler
    /// asking "does this chain grant `zone:write` to `audience`?"
    /// receive `valid: true` regardless of the leaf's actual `att`.
    #[error(
        "UCAN leaf does not grant required capability: required={required} leaf_caps={leaf_caps:?}"
    )]
    CapabilityNotGranted {
        /// Required `resource:ability` string the caller asked about.
        required: String,
        /// The leaf's actual `att` array, formatted as
        /// `<resource>:<ability>` strings, for diagnostic.
        leaf_caps: Vec<String>,
    },
}

/// Errors emitted by [`crate::vc`] Verifiable Credential paths.
///
/// Per `crypto-minor-1`, the VC verification path returns typed
/// variants (NEVER a generic `dyn Error` / `String`) so callers can
/// branch on the failure mode (expired / revoked / issuer-not-trusted
/// / signature-invalid / parse-error).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VcError {
    /// VC `expirationDate` rejected at validation time.
    #[error("VC expired: exp={exp} <= now={now}")]
    Expired {
        /// Expiration epoch seconds.
        exp: u64,
        /// Validation time in epoch seconds.
        now: u64,
    },
    /// VC `credentialStatus` URL has been revoked.
    #[error("VC revoked: credentialStatus URL {status_id} listed in revocation registry")]
    Revoked {
        /// The revoked credentialStatus identifier.
        status_id: String,
    },
    /// Issuer DID is not present in the trust-domain allow-list.
    #[error("VC issuer not trusted: issuer {issuer} not in allow-list")]
    IssuerNotTrusted {
        /// The untrusted issuer DID.
        issuer: String,
    },
    /// Signature verification failed (issuer mismatch / tampered claims).
    #[error("VC signature verification failed")]
    BadSignature,
    /// VC could not be decoded from canonical bytes.
    #[error("VC decode failed")]
    DecodeFailed,
    /// VC missing a load-bearing field (issuer / issuanceDate / etc.).
    #[error("VC missing required field: {field}")]
    MissingField {
        /// Name of the missing field.
        field: &'static str,
    },
    /// VC `nbf` (not-yet-valid) — issuanceDate in the future relative
    /// to validation time. (W3C VC v1.1 does not formally define an
    /// `nbf`; we treat `issuanceDate > now` as a rejection on principle.)
    #[error("VC not yet valid: issuanceDate={issued_at} > now={now}")]
    NotYetValid {
        /// Issuance epoch seconds.
        issued_at: u64,
        /// Validation time in epoch seconds.
        now: u64,
    },
}

/// Errors emitted by [`crate::multi_sig::MultiSigSurface`] paths.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MultiSigError {
    /// Signature verification failed.
    #[error("multi-sig signature verification failed")]
    BadSignature,
    /// Underlying signer / verifier rejected the input.
    #[error("multi-sig surface rejected input: {0}")]
    Rejected(&'static str),
    /// Surface is shape-only / not yet implemented (post-Phase-3
    /// v1-assessment-window per D-PHASE-3-24).
    #[error("multi-sig surface unimplemented (post-Phase-3 v1-assessment-window per D-PHASE-3-24)")]
    PostPhase3,
}

/// Errors emitted by [`crate::did_rotation`] paths.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DidRotationError {
    /// The OLD keypair did not sign the rotation attestation
    /// (verifier rejection).
    #[error("rotation attestation signature invalid")]
    BadSignature,
    /// Attestation references a DID that does not match the supplied
    /// previous keypair's public key.
    #[error("rotation attestation previous-DID mismatch: claimed={claimed} actual={actual}")]
    PreviousDidMismatch {
        /// The DID the attestation names as previous.
        claimed: String,
        /// The DID derived from the supplied OLD keypair.
        actual: String,
    },
    /// Attestation could not be decoded from canonical bytes.
    #[error("rotation attestation decode failed")]
    DecodeFailed,
    /// G24-D-FP-2: rotation event's HLC (`superseded_at`) is not strictly
    /// greater than the latest accepted rotation event for the same
    /// previous DID. Defends against replay-at-same-HLC + nonce-swap
    /// attacks per phase-4-backlog §4.10.
    #[error(
        "rotation event HLC not monotonically greater: prev_did={prev_did} incoming_hlc={incoming_hlc} latest_hlc={latest_hlc}"
    )]
    HlcNotStrictlyMonotonic {
        /// The previous-DID whose rotation history was consulted.
        prev_did: String,
        /// HLC of the rotation event being accepted.
        incoming_hlc: u64,
        /// Latest already-accepted HLC for `prev_did`.
        latest_hlc: u64,
    },
    /// G24-D-FP-2: rotation event is a verbatim duplicate of an
    /// already-accepted event (same prev_did + same next_did + same
    /// superseded_at + same signature). Nonce-binding defense.
    #[error("rotation event verbatim replay rejected: prev_did={prev_did} hlc={hlc}")]
    VerbatimReplay {
        /// The previous-DID being replayed.
        prev_did: String,
        /// HLC of the replayed event.
        hlc: u64,
    },
}

/// Errors emitted by [`crate::device_attestation`] paths.
///
/// Per `crypto-major-6` + `br-r4-r1-4` / `br-r4-r2-3` MAJOR + the
/// device-DID-attestation-replay defect-class, every reject path on
/// the attestation acceptor returns a distinct typed variant.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeviceAttestationError {
    /// Attestation signature is invalid (parent key mismatch / tamper).
    #[error("device attestation signature invalid")]
    BadSignature,
    /// Attestation older than the freshness window allows.
    #[error(
        "device attestation freshness expired: issued_at={issued_at} now={now} window={window}"
    )]
    FreshnessExpired {
        /// Issuance epoch seconds.
        issued_at: u64,
        /// Validation time in epoch seconds.
        now: u64,
        /// Freshness window in seconds.
        window: u64,
    },
    /// Same nonce already accepted within the freshness window
    /// (replay defense via per-issuer nonce store).
    #[error("device attestation nonce replay rejected")]
    NonceReplay,
    /// Device has been revoked by its parent DID.
    #[error("device attestation rejected: device {device_did} has been revoked")]
    DeviceRevoked {
        /// The revoked device-DID.
        device_did: String,
    },
    /// Browser-target context produced an attestation claiming
    /// `runs_sandbox=true` (the wasmtime SANDBOX runtime is
    /// unavailable on `wasm32-unknown-unknown` per Phase-2b
    /// `E_SANDBOX_UNAVAILABLE_ON_WASM`). Closes the
    /// `br-r4-r1-4` / `br-r4-r2-3` MAJOR trust-graph forgery surface.
    /// Catalog code: `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`.
    #[error(
        "device attestation incompatible with runtime: {detail} (E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME)"
    )]
    IncompatibleWithRuntime {
        /// Operator-readable detail string.
        detail: &'static str,
    },
    /// Acceptor expected the parent DID to issue the attestation but
    /// the issuer was a different DID (e.g., self-signed widening).
    #[error(
        "device attestation issuer is not parent: issuer={issuer} expected_parent={expected_parent}"
    )]
    IssuerNotParent {
        /// Issuer DID.
        issuer: String,
        /// Expected parent DID.
        expected_parent: String,
    },
    /// Device claimed wider authority than parent grants.
    #[error("device attestation envelope widens parent authority: {detail}")]
    EnvelopeWidening {
        /// Operator-readable detail string.
        detail: &'static str,
    },
    /// Decode failure on canonical bytes.
    #[error("device attestation decode failed")]
    DecodeFailed,
}

impl DeviceAttestationError {
    /// Stable error code for catalog lookup. Matches the
    /// `ERROR-CATALOG.md` entry for `IncompatibleWithRuntime`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadSignature => "E_DEVICE_ATTESTATION_BAD_SIGNATURE",
            Self::FreshnessExpired { .. } => "E_DEVICE_ATTESTATION_FRESHNESS_EXPIRED",
            Self::NonceReplay => "E_DEVICE_ATTESTATION_NONCE_REPLAY",
            Self::DeviceRevoked { .. } => "E_DEVICE_ATTESTATION_DEVICE_REVOKED",
            Self::IncompatibleWithRuntime { .. } => {
                "E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME"
            }
            Self::IssuerNotParent { .. } => "E_DEVICE_ATTESTATION_ISSUER_NOT_PARENT",
            Self::EnvelopeWidening { .. } => "E_DEVICE_ATTESTATION_ENVELOPE_WIDENING",
            Self::DecodeFailed => "E_DEVICE_ATTESTATION_DECODE_FAILED",
        }
    }
}
