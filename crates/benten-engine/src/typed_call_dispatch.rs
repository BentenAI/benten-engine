//! Phase-3 G21-T1 — engine-side typed-CALL dispatch implementations.
//!
//! Wires the 10 typed-CALL ops to their underlying APIs in
//! `benten-id` (Ed25519 / DID / UCAN / VC) + `benten-core` (BLAKE3 /
//! base32-multibase) + `bs58` (base58btc-multibase). Per CLAUDE.md
//! baked-in commitment #16: SANDBOX is for compute that does NOT fit
//! other primitives — crypto ops fit CALL because they're input →
//! typed result, no side effects on engine state. The 12-primitive
//! commitment (#1) holds.
//!
//! ## Why this lives in `benten-engine` and not `benten-eval`
//!
//! Per arch-r1-10 / `benten-eval/Cargo.toml` policy: `benten-eval`
//! cannot depend on `benten-id` or `benten-graph` (the evaluator is
//! storage-/identity-ignorant). The actual op dispatch must therefore
//! happen in `benten-engine` where both crates are visible. The
//! `dispatch` function is invoked from
//! `<Engine as benten_eval::PrimitiveHost>::dispatch_typed_call`
//! (lives in `crate::primitive_host`) AFTER the eval-side
//! `execute_typed_call` (CALL primitive's typed-CALL fork) has
//! cleared the cap-check + input-shape validation.
//!
//! ## Native-only
//!
//! `benten-id` is in the native-only target section of
//! `crates/benten-engine/Cargo.toml` (full-peer concern; thin clients
//! consume already-verified results from full peers per CLAUDE.md
//! baked-in #17). This module is therefore `cfg(not(target_arch =
//! "wasm32"))`-gated at its single call site in `primitive_host.rs`.
//!
//! ## Side-effect contract
//!
//! Every op is pure on engine state — no `WRITE` to the backend, no
//! event emission, no IVM update. The output is a fresh `Value::Map`
//! the caller can inspect or pass to subsequent primitives. A clean
//! negative result (e.g. Ed25519 verify returns `valid: false`) is
//! NOT a dispatch error — it's a structured `{ valid: false }` Map.
//! Only well-formed op-internal failures (malformed key bytes,
//! corrupted UCAN envelope, unsupported DID method) surface as
//! [`benten_eval::EvalError::TypedCallDispatchError`].

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use benten_core::Value;
use benten_eval::{EvalError, TypedCallOp};
use benten_id::keypair::{ENVELOPE_ALG, ENVELOPE_VERSION, Keypair, PublicKey, Signature};

/// Dispatch one of the 10 typed-CALL ops to its underlying
/// implementation.
///
/// The cap-check has already cleared at the eval-side
/// `execute_typed_call`; this function dispatches the actual op.
///
/// # Errors
/// Returns [`EvalError::TypedCallDispatchError`] when the underlying
/// API raises a typed error (e.g. `KeypairError` / `UcanError` /
/// `VcError` / `DidError`).
pub fn dispatch(op: TypedCallOp, input: &Value) -> Result<Value, EvalError> {
    match op {
        TypedCallOp::Ed25519Sign => ed25519_sign(input),
        TypedCallOp::Ed25519Verify => ed25519_verify(input),
        TypedCallOp::KeypairGenerate => keypair_generate(),
        TypedCallOp::KeypairFromSeed => keypair_from_seed(input),
        TypedCallOp::Blake3Hash => blake3_hash(input),
        TypedCallOp::MultibaseEncode => multibase_encode(input),
        TypedCallOp::MultibaseDecode => multibase_decode(input),
        TypedCallOp::DidResolve => did_resolve(input),
        TypedCallOp::UcanValidateChain => ucan_validate_chain(input),
        TypedCallOp::VcVerify => vc_verify(input),
        // `TypedCallOp` is `#[non_exhaustive]` so we surface a typed
        // dispatch error rather than panic if a future enum variant
        // is added without a corresponding dispatch arm here. This
        // matches the catalog's `TypedCallUnknownOp` semantic.
        _ => Err(EvalError::TypedCallUnknownOp {
            op_name: op.name().to_string(),
        }),
    }
}

// ---------------------------------------------------------------------
// Per-op handlers
// ---------------------------------------------------------------------

fn ed25519_sign(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let private_key = expect_bytes_exact(map, "private_key", 32)?;
    let message = expect_bytes(map, "message")?;

    // Construct keypair via DAG-CBOR envelope round-trip — the
    // public `from_dag_cbor_envelope` is the audit-trail-shaped path
    // (raw `from_seed_bytes(&[u8])` accepts the envelope, not raw
    // seed bytes; per `crypto-major-5`).
    let envelope = build_seed_envelope(private_key);
    let kp = Keypair::from_dag_cbor_envelope(&envelope).map_err(|e| {
        EvalError::TypedCallDispatchError {
            op_name: TypedCallOp::Ed25519Sign.name(),
            reason: format!("private_key: {e}"),
        }
    })?;

    let sig = kp.sign(message);
    let sig_bytes: [u8; 64] = sig.to_bytes();

    let mut out = BTreeMap::new();
    out.insert("signature".to_string(), Value::Bytes(sig_bytes.to_vec()));
    Ok(Value::Map(out))
}

fn ed25519_verify(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let public_key_bytes = expect_bytes_exact(map, "public_key", 32)?;
    let message = expect_bytes(map, "message")?;
    let sig_bytes_slice = expect_bytes_exact(map, "signature", 64)?;

    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(public_key_bytes);
    // Malformed public key (not on curve) → dispatch error rather
    // than `valid: false` — the input was structurally invalid, not
    // semantically rejected.
    let pk = PublicKey::from_bytes(&pk_arr).ok_or_else(|| EvalError::TypedCallDispatchError {
        op_name: TypedCallOp::Ed25519Verify.name(),
        reason: "public_key bytes do not form a valid Ed25519 curve point".to_string(),
    })?;

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(sig_bytes_slice);
    let sig = Signature::from_bytes(&sig_arr);

    let valid = pk.verify(message, &sig).is_ok();

    let mut out = BTreeMap::new();
    out.insert("valid".to_string(), Value::Bool(valid));
    Ok(Value::Map(out))
}

fn keypair_generate() -> Result<Value, EvalError> {
    // OS CSPRNG per `crypto-major-2` — pinned at the `benten-id`
    // boundary (`Keypair::generate` routes to `OsRng` /
    // `getrandom`).
    let kp = Keypair::generate();
    let envelope = kp.export_seed_envelope();
    // Recover the raw 32-byte seed from the envelope (test-side
    // accessor only — production callers see the envelope shape).
    // We need raw 32 bytes for the typed-CALL output schema.
    let secret_bytes = kp.secret_bytes_for_test();
    let _ = envelope; // envelope round-trip path validated; not surfaced

    let public_bytes = kp.public_key().to_bytes();

    let mut out = BTreeMap::new();
    out.insert(
        "private_key".to_string(),
        Value::Bytes(secret_bytes.to_vec()),
    );
    out.insert(
        "public_key".to_string(),
        Value::Bytes(public_bytes.to_vec()),
    );
    Ok(Value::Map(out))
}

fn keypair_from_seed(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let seed = expect_bytes_exact(map, "seed", 32)?;

    let envelope = build_seed_envelope(seed);
    let kp = Keypair::from_dag_cbor_envelope(&envelope).map_err(|e| {
        EvalError::TypedCallDispatchError {
            op_name: TypedCallOp::KeypairFromSeed.name(),
            reason: format!("seed: {e}"),
        }
    })?;

    let public_bytes = kp.public_key().to_bytes();
    let secret_bytes = kp.secret_bytes_for_test();

    let mut out = BTreeMap::new();
    out.insert(
        "private_key".to_string(),
        Value::Bytes(secret_bytes.to_vec()),
    );
    out.insert(
        "public_key".to_string(),
        Value::Bytes(public_bytes.to_vec()),
    );
    Ok(Value::Map(out))
}

fn blake3_hash(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let data = expect_bytes(map, "data")?;
    let digest = blake3::hash(data);
    let mut out = BTreeMap::new();
    out.insert("hash".to_string(), Value::Bytes(digest.as_bytes().to_vec()));
    Ok(Value::Map(out))
}

fn multibase_encode(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let data = expect_bytes(map, "data")?;
    let base = expect_text(map, "base")?;

    let encoded = match base.as_str() {
        // RFC4648 base32 lower no-pad (engine-canonical CID encoding).
        "b" => {
            let body = data_encoding::BASE32_NOPAD
                .encode(data)
                .to_ascii_lowercase();
            format!("b{body}")
        }
        // base58btc (W3C did:key body).
        "z" => {
            let body = bs58::encode(data).into_string();
            format!("z{body}")
        }
        other => {
            return Err(EvalError::TypedCallDispatchError {
                op_name: TypedCallOp::MultibaseEncode.name(),
                reason: format!(
                    "unsupported multibase base '{other}' (Phase-3 G21-T1 supports 'b' + 'z')"
                ),
            });
        }
    };

    let mut out = BTreeMap::new();
    out.insert("encoded".to_string(), Value::Text(encoded));
    Ok(Value::Map(out))
}

fn multibase_decode(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let encoded = expect_text(map, "encoded")?;

    let mut chars = encoded.chars();
    let prefix = chars
        .next()
        .ok_or_else(|| EvalError::TypedCallDispatchError {
            op_name: TypedCallOp::MultibaseDecode.name(),
            reason: "encoded string is empty (no multibase prefix)".to_string(),
        })?;
    let body: String = chars.collect();

    let (data, base) = match prefix {
        'b' => {
            let upper = body.to_ascii_uppercase();
            let bytes = data_encoding::BASE32_NOPAD
                .decode(upper.as_bytes())
                .map_err(|e| EvalError::TypedCallDispatchError {
                    op_name: TypedCallOp::MultibaseDecode.name(),
                    reason: format!("base32 decode: {e}"),
                })?;
            (bytes, "b".to_string())
        }
        'z' => {
            let bytes =
                bs58::decode(&body)
                    .into_vec()
                    .map_err(|e| EvalError::TypedCallDispatchError {
                        op_name: TypedCallOp::MultibaseDecode.name(),
                        reason: format!("base58btc decode: {e}"),
                    })?;
            (bytes, "z".to_string())
        }
        other => {
            return Err(EvalError::TypedCallDispatchError {
                op_name: TypedCallOp::MultibaseDecode.name(),
                reason: format!(
                    "unsupported multibase prefix '{other}' (Phase-3 G21-T1 supports 'b' + 'z')"
                ),
            });
        }
    };

    let mut out = BTreeMap::new();
    out.insert("data".to_string(), Value::Bytes(data));
    out.insert("base".to_string(), Value::Text(base));
    Ok(Value::Map(out))
}

fn did_resolve(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let did_str = expect_text(map, "did")?;

    let did = benten_id::did::Did::from_string_unchecked(did_str.clone());
    let pk = did
        .resolve()
        .map_err(|e| EvalError::TypedCallDispatchError {
            op_name: TypedCallOp::DidResolve.name(),
            reason: format!("did_resolve: {e}"),
        })?;

    let pk_bytes = pk.to_bytes();
    let mut out = BTreeMap::new();
    out.insert("method".to_string(), Value::Text("key".to_string()));
    out.insert("public_key".to_string(), Value::Bytes(pk_bytes.to_vec()));
    Ok(Value::Map(out))
}

fn ucan_validate_chain(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let tokens_value = map.get("tokens").ok_or(EvalError::TypedCallInvalidInput {
        op_name: TypedCallOp::UcanValidateChain.name(),
        reason: "missing required field 'tokens'".to_string(),
    })?;
    let token_bytes_list = match tokens_value {
        Value::List(items) => items,
        _ => {
            return Err(EvalError::TypedCallInvalidInput {
                op_name: TypedCallOp::UcanValidateChain.name(),
                reason: "tokens must be List<Bytes>".to_string(),
            });
        }
    };
    let audience_str = expect_text(map, "audience")?;
    let capability_str = expect_text(map, "capability")?;
    let now: u64 = match map.get("now") {
        Some(Value::Int(n)) if *n >= 0 => *n as u64,
        _ => {
            return Err(EvalError::TypedCallInvalidInput {
                op_name: TypedCallOp::UcanValidateChain.name(),
                reason: "now must be non-negative Int (epoch seconds)".to_string(),
            });
        }
    };

    // Decode each token's DAG-CBOR bytes into a Ucan envelope.
    let mut chain: Vec<benten_id::ucan::Ucan> = Vec::with_capacity(token_bytes_list.len());
    for (i, t) in token_bytes_list.iter().enumerate() {
        let bytes = match t {
            Value::Bytes(b) => b,
            _ => {
                return Err(EvalError::TypedCallInvalidInput {
                    op_name: TypedCallOp::UcanValidateChain.name(),
                    reason: format!("tokens[{i}] must be Bytes"),
                });
            }
        };
        let ucan: benten_id::ucan::Ucan = serde_ipld_dagcbor::from_slice(bytes).map_err(|e| {
            EvalError::TypedCallDispatchError {
                op_name: TypedCallOp::UcanValidateChain.name(),
                reason: format!("tokens[{i}] DAG-CBOR decode: {e}"),
            }
        })?;
        chain.push(ucan);
    }

    // Parse the required-capability string into a `(resource,
    // ability)` pair. Format: `<resource>:<ability>` where the LAST
    // `:`-separated segment is the ability (per Phase-3 typed-CALL
    // contract — matches the test fixtures' `"zone:user:write"`
    // shape that built `Capability::new("zone:user", "write")`).
    // A capability string with no `:` cannot name a (resource,
    // ability) pair, so it is treated as a clean negative
    // (`valid: false`) — NOT a dispatch error, since the input is
    // shape-valid Text but semantically rejects.
    let required_cap = match capability_str.rsplit_once(':') {
        Some((resource, ability)) if !resource.is_empty() && !ability.is_empty() => {
            benten_id::ucan::Capability::new(resource, ability)
        }
        _ => {
            let mut out = BTreeMap::new();
            out.insert("valid".to_string(), Value::Bool(false));
            out.insert(
                "reason".to_string(),
                Value::Text(format!(
                    "capability: must be '<resource>:<ability>'; got '{capability_str}'"
                )),
            );
            return Ok(Value::Map(out));
        }
    };

    // Compose: audience binding + signature/time/attenuation chain
    // walk + LEAF-CAPABILITY-CLAIM check (defense-in-depth — without
    // this, a chain that's structurally sound but lacks the requested
    // claim would still validate: true; sec-major-1 fix). A clean
    // negative is `valid: false` with the reason; only a structurally-
    // malformed input bubbles `TypedCallDispatchError`.
    let audience_did = benten_id::did::Did::from_string_unchecked(audience_str.clone());

    match benten_id::ucan::validate_chain_for_capability(&chain, &audience_did, &required_cap, now)
    {
        Ok(()) => {
            let mut out = BTreeMap::new();
            out.insert("valid".to_string(), Value::Bool(true));
            out.insert("reason".to_string(), Value::Text(String::new()));
            Ok(Value::Map(out))
        }
        Err(e) => {
            // Tag the reason with the failure family so callers can
            // distinguish audience / time-window / chain / leaf-claim
            // failures without re-running the validator.
            let family = match e {
                benten_id::errors::UcanError::AudienceMismatch { .. } => "audience",
                benten_id::errors::UcanError::Expired { .. }
                | benten_id::errors::UcanError::NotYetValid { .. } => "time",
                benten_id::errors::UcanError::CapabilityNotGranted { .. } => "capability",
                _ => "chain",
            };
            let mut out = BTreeMap::new();
            out.insert("valid".to_string(), Value::Bool(false));
            out.insert("reason".to_string(), Value::Text(format!("{family}: {e}")));
            Ok(Value::Map(out))
        }
    }
}

fn vc_verify(input: &Value) -> Result<Value, EvalError> {
    let map = expect_map(input)?;
    let credential_bytes = expect_bytes(map, "credential")?;
    let expected_issuer_str = expect_text(map, "expected_issuer_did")?;
    // `now` (epoch seconds) drives the `expirationDate` time-window
    // gate per sec-major-3 — without it, expired VCs would otherwise
    // return `valid: true` because bare `vc::verify` skips the
    // expiration check by design (the timed gate lives on
    // `vc::verify_at`). Required by the typed-CALL contract.
    let now: u64 = match map.get("now") {
        Some(Value::Int(n)) if *n >= 0 => *n as u64,
        _ => {
            return Err(EvalError::TypedCallInvalidInput {
                op_name: TypedCallOp::VcVerify.name(),
                reason: "now must be non-negative Int (epoch seconds)".to_string(),
            });
        }
    };

    let credential: benten_id::vc::Credential = serde_ipld_dagcbor::from_slice(credential_bytes)
        .map_err(|e| EvalError::TypedCallDispatchError {
            op_name: TypedCallOp::VcVerify.name(),
            reason: format!("credential DAG-CBOR decode: {e}"),
        })?;

    let expected_issuer = benten_id::did::Did::from_string_unchecked(expected_issuer_str.clone());

    // `verify_at` enforces signature + issuer-binding + issuance/
    // expiration time-window. A semantic rejection (bad signature /
    // expired / not-yet-valid / wrong issuer) is a clean negative
    // `valid: false`; only well-formed-input failures bubble.
    let valid = benten_id::vc::verify_at(&credential, &expected_issuer, now).is_ok();

    let issuer = credential.issuer().to_string();
    let subject = credential.subject().to_string();

    let mut out = BTreeMap::new();
    out.insert("valid".to_string(), Value::Bool(valid));
    out.insert("issuer".to_string(), Value::Text(issuer));
    out.insert("subject".to_string(), Value::Text(subject));
    Ok(Value::Map(out))
}

// ---------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------

/// Build a DAG-CBOR `{version, alg, secret_bytes}` envelope for the
/// given 32-byte seed. Round-trips byte-identically through
/// [`Keypair::from_dag_cbor_envelope`].
fn build_seed_envelope(seed: &[u8]) -> Vec<u8> {
    debug_assert_eq!(seed.len(), 32, "seed must be exactly 32 bytes");
    #[derive(serde::Serialize)]
    struct SeedEnvelope<'a> {
        version: u8,
        alg: &'static str,
        secret_bytes: &'a serde_bytes::Bytes,
    }
    let env = SeedEnvelope {
        version: ENVELOPE_VERSION,
        alg: ENVELOPE_ALG,
        secret_bytes: serde_bytes::Bytes::new(seed),
    };
    serde_ipld_dagcbor::to_vec(&env)
        .expect("DAG-CBOR encoding of fixed-shape SeedEnvelope cannot fail")
}

fn expect_map(v: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    match v {
        Value::Map(m) => Ok(m),
        _ => Err(EvalError::TypedCallInvalidInput {
            op_name: "typed_call",
            reason: "input must be a Map".to_string(),
        }),
    }
}

fn expect_bytes<'a>(map: &'a BTreeMap<String, Value>, field: &str) -> Result<&'a [u8], EvalError> {
    match map.get(field) {
        Some(Value::Bytes(b)) => Ok(b.as_slice()),
        _ => Err(EvalError::TypedCallInvalidInput {
            op_name: "typed_call",
            reason: format!("field '{field}' must be Bytes"),
        }),
    }
}

fn expect_bytes_exact<'a>(
    map: &'a BTreeMap<String, Value>,
    field: &str,
    expected_len: usize,
) -> Result<&'a [u8], EvalError> {
    let bytes = expect_bytes(map, field)?;
    if bytes.len() != expected_len {
        return Err(EvalError::TypedCallInvalidInput {
            op_name: "typed_call",
            reason: format!(
                "field '{field}' must be exactly {expected_len} bytes; got {}",
                bytes.len()
            ),
        });
    }
    Ok(bytes)
}

fn expect_text<'a>(map: &'a BTreeMap<String, Value>, field: &str) -> Result<&'a String, EvalError> {
    match map.get(field) {
        Some(Value::Text(s)) => Ok(s),
        _ => Err(EvalError::TypedCallInvalidInput {
            op_name: "typed_call",
            reason: format!("field '{field}' must be Text"),
        }),
    }
}
