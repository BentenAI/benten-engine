//! Typed-CALL engine-side dispatch surface (Phase-3 G21-T1).
//!
//! Typed-CALL extends the existing CALL primitive with a registry of
//! engine-known operations. It is NOT a new primitive — the 12-primitive
//! commitment (CLAUDE.md baked-in #1) holds. The CALL primitive
//! dispatches through this registry when its `target` (handler_id)
//! starts with the reserved [`TYPED_CALL_PREFIX`] (`"engine:typed:"`).
//!
//! ## Why this surface exists
//!
//! Phase-3 ships an Atrium / UCAN / DID / VC story; handlers need
//! crypto operations (Ed25519 sign/verify, BLAKE3 hash, multibase,
//! DID resolve, UCAN chain validation, VC verify). Per CLAUDE.md
//! baked-in #16 (SANDBOX is for compute that does NOT fit other
//! primitives) the SANDBOX host-fn surface stays minimum-viable —
//! `time` / `log` / `kv:read` / `random` only. Crypto ops fit the
//! CALL surface because they're input → typed result, no side
//! effects on engine state. Adding them as typed-CALL ops gives
//! handlers a low-friction call shape (an existing CALL Node) without
//! widening the SANDBOX host surface or inventing a 13th primitive.
//!
//! ## Architecture
//!
//! - [`TypedCallOp`] enumerates the closed set of engine-known op
//!   names + their per-op required capability.
//! - Input validation lives entirely in `benten-eval` ([`validate_input`]
//!   per-op arms): the `benten-eval` crate cannot depend on `benten-id`
//!   or `benten-graph` (arch-r1-10), so the actual crypto / codec /
//!   DID-resolve invocations happen on the host-side
//!   `dispatch_typed_call` impl in `benten-engine`.
//! - [`crate::host::PrimitiveHost::dispatch_typed_call`] is the trait
//!   method the CALL primitive routes through; engine impls handle the
//!   10 ops by calling into `benten-id` / `benten-core`.
//!
//! ## Capability model
//!
//! Each typed-CALL op declares a per-op required capability of shape
//! `cap:typed:<group>` (see [`TypedCallOp::required_cap`]). The host's
//! [`crate::host::PrimitiveHost::check_capability`] hook gates the op
//! BEFORE dispatch — a denied call has zero observable side effect.
//! Under `NoAuthBackend` all typed-CALL caps are permitted; UCAN
//! backend gates per chain claim.

use benten_core::Value;

use crate::EvalError;

/// Reserved handler-id namespace prefix for typed-CALL dispatch.
///
/// A CALL Operation Node whose `target` property starts with this
/// prefix is routed through the typed-CALL registry instead of the
/// user handler registry. The trailing segment is the op name.
pub const TYPED_CALL_PREFIX: &str = "engine:typed:";

/// Closed set of engine-known typed-CALL ops.
///
/// Phase-3 G21-T1 ships exactly these 10 ops; the registry is closed
/// (no user-registered typed-CALL ops). Extending the registry is a
/// Rust-only engine concern that adds a variant to this enum + its
/// per-op `validate_input` / `required_cap` arms + a corresponding
/// `dispatch_typed_call` arm in `benten-engine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypedCallOp {
    /// Ed25519 detached signature over `message` using `private_key`.
    /// Input shape: `{ private_key: Bytes(32), message: Bytes }`.
    /// Output shape: `{ signature: Bytes(64) }`.
    /// Required cap: `cap:typed:crypto-sign`.
    Ed25519Sign,
    /// Ed25519 signature verification.
    /// Input shape: `{ public_key: Bytes(32), message: Bytes, signature: Bytes(64) }`.
    /// Output shape: `{ valid: Bool }` (a `false` result is NOT an
    /// error — `E_TYPED_CALL_DISPATCH_ERROR` fires only on malformed
    /// keys/signatures bubbling from `benten-id`).
    /// Required cap: `cap:typed:crypto-verify`.
    Ed25519Verify,
    /// Generate a fresh Ed25519 keypair from OS CSPRNG (per
    /// `crypto-major-2`).
    /// Input shape: `{}` (or `{ seed: Null }`; any optional `seed`
    /// field must be `Null` — non-null seeds route through
    /// [`TypedCallOp::KeypairFromSeed`] for the deterministic path).
    /// Output shape: `{ private_key: Bytes(32), public_key: Bytes(32) }`.
    /// Required cap: `cap:typed:crypto-keygen`.
    KeypairGenerate,
    /// Derive an Ed25519 keypair deterministically from a 32-byte seed.
    /// Input shape: `{ seed: Bytes(32) }`.
    /// Output shape: `{ private_key: Bytes(32), public_key: Bytes(32) }`.
    /// Required cap: `cap:typed:crypto-keygen`.
    KeypairFromSeed,
    /// BLAKE3 hash of `data`.
    /// Input shape: `{ data: Bytes }`.
    /// Output shape: `{ hash: Bytes(32) }`.
    /// Required cap: `cap:typed:hash`.
    Blake3Hash,
    /// Multibase encode `data` under `base`. Phase-3 G21-T1 supports
    /// the bases used elsewhere in benten: `b` (RFC4648 base32 lower
    /// no-pad — engine-canonical CID encoding) + `z` (base58btc — W3C
    /// did:key body). Other multibase bases are deferred to follow-on
    /// waves.
    /// Input shape: `{ data: Bytes, base: Text }`.
    /// Output shape: `{ encoded: Text }`.
    /// Required cap: `cap:typed:codec`.
    MultibaseEncode,
    /// Multibase decode `encoded` (multibase-prefixed string) into
    /// raw bytes. The base is recovered from the encoded string's
    /// leading character (per multibase spec) and surfaced in the
    /// output for the caller's diagnostic.
    /// Input shape: `{ encoded: Text }`.
    /// Output shape: `{ data: Bytes, base: Text }`.
    /// Required cap: `cap:typed:codec`.
    MultibaseDecode,
    /// Resolve a `did:key` DID to its associated public key.
    /// Input shape: `{ did: Text }`.
    /// Output shape: `{ method: Text, public_key: Bytes(32) }` for
    /// `did:key:z...` Ed25519 DIDs.
    /// Required cap: `cap:typed:did-resolve`.
    DidResolve,
    /// Validate a UCAN chain at a given audience for a required
    /// capability. Per `crypto-blocker-2`: nbf/exp time-window check
    /// + audience binding + signature verification + attenuation
    /// chain walk all happen here.
    /// Input shape: `{ tokens: List<Bytes>, audience: Text, capability: Text, now: Int }`
    /// where `tokens` is an outer-to-root chain of DAG-CBOR-encoded
    /// UCAN bytes (each entry is the canonical-bytes form of a Ucan
    /// envelope), `audience` is the expected audience DID string,
    /// `capability` is the required `resource:ability` claim,
    /// `now` is the wall-clock time in seconds-since-epoch the
    /// chain-walk uses for nbf/exp checks.
    /// Output shape: `{ valid: Bool, reason: Text }` — a clean
    /// negative (`valid: false`, `reason: "expired"`) is NOT an
    /// error.
    /// Required cap: `cap:typed:ucan-validate`.
    UcanValidateChain,
    /// Verify a Verifiable Credential's signature + issuer binding.
    /// Input shape: `{ credential: Bytes, expected_issuer_did: Text }`
    /// where `credential` is the DAG-CBOR-encoded canonical bytes of
    /// the Credential envelope. The verifier consults `now` from the
    /// caller's `now: Int` field for time-window checks (defaults to
    /// the engine's monotonic-clock-derived wall-clock if omitted).
    /// Output shape: `{ valid: Bool, issuer: Text, subject: Text }`.
    /// Required cap: `cap:typed:vc-verify`.
    VcVerify,
}

impl TypedCallOp {
    /// Parse the trailing op name segment (after the
    /// [`TYPED_CALL_PREFIX`]) into the matching variant.
    ///
    /// Returns `None` for unknown ops; the caller surfaces this as
    /// `ErrorCode::TypedCallUnknownOp` per the public catalog code.
    #[must_use]
    pub fn parse(op_name: &str) -> Option<Self> {
        match op_name {
            "ed25519_sign" => Some(Self::Ed25519Sign),
            "ed25519_verify" => Some(Self::Ed25519Verify),
            "keypair_generate" => Some(Self::KeypairGenerate),
            "keypair_from_seed" => Some(Self::KeypairFromSeed),
            "blake3_hash" => Some(Self::Blake3Hash),
            "multibase_encode" => Some(Self::MultibaseEncode),
            "multibase_decode" => Some(Self::MultibaseDecode),
            "did_resolve" => Some(Self::DidResolve),
            "ucan_validate_chain" => Some(Self::UcanValidateChain),
            "vc_verify" => Some(Self::VcVerify),
            _ => None,
        }
    }

    /// Stable string identifier for the op (the trailing segment of
    /// the typed-CALL `target`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ed25519Sign => "ed25519_sign",
            Self::Ed25519Verify => "ed25519_verify",
            Self::KeypairGenerate => "keypair_generate",
            Self::KeypairFromSeed => "keypair_from_seed",
            Self::Blake3Hash => "blake3_hash",
            Self::MultibaseEncode => "multibase_encode",
            Self::MultibaseDecode => "multibase_decode",
            Self::DidResolve => "did_resolve",
            Self::UcanValidateChain => "ucan_validate_chain",
            Self::VcVerify => "vc_verify",
        }
    }

    /// Per-op required capability string.
    ///
    /// Routed through [`crate::host::PrimitiveHost::check_capability`]
    /// before dispatch. Under `NoAuthBackend` all typed-CALL caps are
    /// permitted; UCAN backend gates per chain claim.
    #[must_use]
    pub fn required_cap(self) -> &'static str {
        match self {
            Self::Ed25519Sign => "cap:typed:crypto-sign",
            Self::Ed25519Verify => "cap:typed:crypto-verify",
            Self::KeypairGenerate | Self::KeypairFromSeed => "cap:typed:crypto-keygen",
            Self::Blake3Hash => "cap:typed:hash",
            Self::MultibaseEncode | Self::MultibaseDecode => "cap:typed:codec",
            Self::DidResolve => "cap:typed:did-resolve",
            Self::UcanValidateChain => "cap:typed:ucan-validate",
            Self::VcVerify => "cap:typed:vc-verify",
        }
    }

    /// Validate the input shape against the op's expected schema.
    ///
    /// Returns `Ok(())` when the input is well-formed. Returns an
    /// [`EvalError::TypedCallInvalidInput`] with a brief reason
    /// string when it's not — the caller maps this to
    /// `ErrorCode::TypedCallInvalidInput` at the catalog layer.
    ///
    /// The validation is shape-only — semantic failures (an Ed25519
    /// public key that doesn't form a valid curve point, a malformed
    /// CBOR-encoded UCAN chain, a non-key `did:key:` DID method) are
    /// surfaced as `E_TYPED_CALL_DISPATCH_ERROR` from the host-side
    /// dispatch path, not here.
    ///
    /// # Errors
    ///
    /// Returns [`EvalError::TypedCallInvalidInput`] when the input
    /// shape rejects.
    pub fn validate_input(self, input: &Value) -> Result<(), EvalError> {
        let map = match input {
            Value::Map(m) => m,
            _ => {
                return Err(EvalError::TypedCallInvalidInput {
                    op_name: self.name(),
                    reason: "input must be a Map".to_string(),
                });
            }
        };
        match self {
            Self::Ed25519Sign => {
                require_bytes_exact(self.name(), map, "private_key", 32)?;
                require_bytes(self.name(), map, "message")?;
                Ok(())
            }
            Self::Ed25519Verify => {
                require_bytes_exact(self.name(), map, "public_key", 32)?;
                require_bytes(self.name(), map, "message")?;
                require_bytes_exact(self.name(), map, "signature", 64)?;
                Ok(())
            }
            Self::KeypairGenerate => {
                // Optional `seed: Null` accepted; non-null seed is
                // an error (caller should use `keypair_from_seed`).
                if let Some(v) = map.get("seed")
                    && !matches!(v, Value::Null)
                {
                    return Err(EvalError::TypedCallInvalidInput {
                        op_name: self.name(),
                        reason: "non-null seed must use keypair_from_seed".to_string(),
                    });
                }
                Ok(())
            }
            Self::KeypairFromSeed => {
                require_bytes_exact(self.name(), map, "seed", 32)?;
                Ok(())
            }
            Self::Blake3Hash => {
                require_bytes(self.name(), map, "data")?;
                Ok(())
            }
            Self::MultibaseEncode => {
                require_bytes(self.name(), map, "data")?;
                require_text(self.name(), map, "base")?;
                Ok(())
            }
            Self::MultibaseDecode => {
                require_text(self.name(), map, "encoded")?;
                Ok(())
            }
            Self::DidResolve => {
                require_text(self.name(), map, "did")?;
                Ok(())
            }
            Self::UcanValidateChain => {
                let tokens = map.get("tokens").ok_or(EvalError::TypedCallInvalidInput {
                    op_name: self.name(),
                    reason: "missing required field 'tokens'".to_string(),
                })?;
                match tokens {
                    Value::List(items) => {
                        if items.is_empty() {
                            return Err(EvalError::TypedCallInvalidInput {
                                op_name: self.name(),
                                reason: "tokens list must not be empty".to_string(),
                            });
                        }
                        for (i, t) in items.iter().enumerate() {
                            if !matches!(t, Value::Bytes(_)) {
                                return Err(EvalError::TypedCallInvalidInput {
                                    op_name: self.name(),
                                    reason: format!("tokens[{i}] must be Bytes"),
                                });
                            }
                        }
                    }
                    _ => {
                        return Err(EvalError::TypedCallInvalidInput {
                            op_name: self.name(),
                            reason: "tokens must be a List<Bytes>".to_string(),
                        });
                    }
                }
                require_text(self.name(), map, "audience")?;
                require_text(self.name(), map, "capability")?;
                require_int(self.name(), map, "now")?;
                Ok(())
            }
            Self::VcVerify => {
                require_bytes(self.name(), map, "credential")?;
                require_text(self.name(), map, "expected_issuer_did")?;
                Ok(())
            }
        }
    }
}

fn require_bytes(
    op_name: &'static str,
    map: &alloc::collections::BTreeMap<alloc::string::String, Value>,
    field: &str,
) -> Result<(), EvalError> {
    match map.get(field) {
        Some(Value::Bytes(_)) => Ok(()),
        Some(_) => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("field '{field}' must be Bytes"),
        }),
        None => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("missing required field '{field}'"),
        }),
    }
}

fn require_bytes_exact(
    op_name: &'static str,
    map: &alloc::collections::BTreeMap<alloc::string::String, Value>,
    field: &str,
    expected_len: usize,
) -> Result<(), EvalError> {
    match map.get(field) {
        Some(Value::Bytes(b)) if b.len() == expected_len => Ok(()),
        Some(Value::Bytes(b)) => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!(
                "field '{field}' must be exactly {expected_len} bytes; got {}",
                b.len()
            ),
        }),
        Some(_) => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("field '{field}' must be Bytes(exactly {expected_len} bytes)"),
        }),
        None => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("missing required field '{field}'"),
        }),
    }
}

fn require_text(
    op_name: &'static str,
    map: &alloc::collections::BTreeMap<alloc::string::String, Value>,
    field: &str,
) -> Result<(), EvalError> {
    match map.get(field) {
        Some(Value::Text(_)) => Ok(()),
        Some(_) => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("field '{field}' must be Text"),
        }),
        None => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("missing required field '{field}'"),
        }),
    }
}

fn require_int(
    op_name: &'static str,
    map: &alloc::collections::BTreeMap<alloc::string::String, Value>,
    field: &str,
) -> Result<(), EvalError> {
    match map.get(field) {
        Some(Value::Int(_)) => Ok(()),
        Some(_) => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("field '{field}' must be Int"),
        }),
        None => Err(EvalError::TypedCallInvalidInput {
            op_name,
            reason: alloc::format!("missing required field '{field}'"),
        }),
    }
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(pairs: &[(&str, Value)]) -> Value {
        let mut m = BTreeMap::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), v.clone());
        }
        Value::Map(m)
    }

    #[test]
    fn parse_round_trip_all_ops() {
        for op in [
            TypedCallOp::Ed25519Sign,
            TypedCallOp::Ed25519Verify,
            TypedCallOp::KeypairGenerate,
            TypedCallOp::KeypairFromSeed,
            TypedCallOp::Blake3Hash,
            TypedCallOp::MultibaseEncode,
            TypedCallOp::MultibaseDecode,
            TypedCallOp::DidResolve,
            TypedCallOp::UcanValidateChain,
            TypedCallOp::VcVerify,
        ] {
            assert_eq!(TypedCallOp::parse(op.name()), Some(op));
        }
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(TypedCallOp::parse("not_an_op"), None);
        assert_eq!(TypedCallOp::parse(""), None);
        assert_eq!(TypedCallOp::parse("ed25519"), None);
    }

    #[test]
    fn ed25519_sign_validates_input_shape() {
        let ok = map(&[
            ("private_key", Value::Bytes(vec![0u8; 32])),
            ("message", Value::Bytes(b"hello".to_vec())),
        ]);
        assert!(TypedCallOp::Ed25519Sign.validate_input(&ok).is_ok());

        let short_key = map(&[
            ("private_key", Value::Bytes(vec![0u8; 16])),
            ("message", Value::Bytes(b"hello".to_vec())),
        ]);
        assert!(TypedCallOp::Ed25519Sign.validate_input(&short_key).is_err());

        let missing_msg = map(&[("private_key", Value::Bytes(vec![0u8; 32]))]);
        assert!(
            TypedCallOp::Ed25519Sign
                .validate_input(&missing_msg)
                .is_err()
        );
    }

    #[test]
    fn ed25519_verify_validates_signature_length() {
        let ok = map(&[
            ("public_key", Value::Bytes(vec![0u8; 32])),
            ("message", Value::Bytes(b"hello".to_vec())),
            ("signature", Value::Bytes(vec![0u8; 64])),
        ]);
        assert!(TypedCallOp::Ed25519Verify.validate_input(&ok).is_ok());

        let bad_sig = map(&[
            ("public_key", Value::Bytes(vec![0u8; 32])),
            ("message", Value::Bytes(b"hello".to_vec())),
            ("signature", Value::Bytes(vec![0u8; 32])),
        ]);
        assert!(TypedCallOp::Ed25519Verify.validate_input(&bad_sig).is_err());
    }

    #[test]
    fn keypair_generate_rejects_non_null_seed() {
        let with_null = map(&[("seed", Value::Null)]);
        assert!(
            TypedCallOp::KeypairGenerate
                .validate_input(&with_null)
                .is_ok()
        );

        let with_bytes = map(&[("seed", Value::Bytes(vec![0u8; 32]))]);
        assert!(
            TypedCallOp::KeypairGenerate
                .validate_input(&with_bytes)
                .is_err()
        );
    }

    #[test]
    fn ucan_validate_chain_requires_non_empty_tokens_list() {
        let empty = map(&[
            ("tokens", Value::List(vec![])),
            ("audience", Value::Text("did:key:z...".into())),
            ("capability", Value::Text("zone:write".into())),
            ("now", Value::Int(1_000_000)),
        ]);
        assert!(
            TypedCallOp::UcanValidateChain
                .validate_input(&empty)
                .is_err()
        );

        let ok = map(&[
            ("tokens", Value::List(vec![Value::Bytes(vec![1, 2, 3])])),
            ("audience", Value::Text("did:key:z...".into())),
            ("capability", Value::Text("zone:write".into())),
            ("now", Value::Int(1_000_000)),
        ]);
        assert!(TypedCallOp::UcanValidateChain.validate_input(&ok).is_ok());
    }

    #[test]
    fn required_caps_are_consistent() {
        // Compile-time-ish pin: every op's required cap starts with
        // `cap:typed:` so the dispatch-time cap-check has a stable
        // namespace to gate on.
        for op in [
            TypedCallOp::Ed25519Sign,
            TypedCallOp::Ed25519Verify,
            TypedCallOp::KeypairGenerate,
            TypedCallOp::KeypairFromSeed,
            TypedCallOp::Blake3Hash,
            TypedCallOp::MultibaseEncode,
            TypedCallOp::MultibaseDecode,
            TypedCallOp::DidResolve,
            TypedCallOp::UcanValidateChain,
            TypedCallOp::VcVerify,
        ] {
            assert!(
                op.required_cap().starts_with("cap:typed:"),
                "op {} cap '{}' must use cap:typed: namespace",
                op.name(),
                op.required_cap()
            );
        }
    }
}
