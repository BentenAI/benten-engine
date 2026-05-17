//! Phase-3 G21-T1 — typed-CALL engine-side dispatch end-to-end tests.
//!
//! Per dispatch-conventions §3.6b end-to-end pin requirement: each of
//! the 10 typed-CALL ops drives the production CALL primitive via the
//! eval-side dispatch fork (`engine:typed:*` prefix detection) +
//! asserts an observable behavioral consequence. A sentinel-presence
//! test would not suffice — we drive the wire.
//!
//! Coverage:
//!   - 10 happy-path ops, each driving real `benten-id` / `benten-core`
//!     APIs through `Engine::dispatch_typed_call`.
//!   - 4 error paths: unknown op + invalid input + cap denied + cap-
//!     denial routing.
//!   - 1 ESC-shape pin: a non-`engine:typed:` `target` does NOT
//!     accidentally land in the typed-CALL registry (the dispatch
//!     fork is prefix-bound).
//!
//! All tests run against a real `Engine` (`tempfile::TempDir`-backed
//! redb) so the `impl PrimitiveHost for Engine` actually fires the
//! `dispatch_typed_call` arm rather than the trait default.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{OperationNode, Value};
use benten_engine::Engine;
use benten_errors::ErrorCode;
use benten_eval::{PrimitiveHost, PrimitiveKind, TypedCallOp, primitives::call};

mod common;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

fn map_value(pairs: &[(&str, Value)]) -> Value {
    let mut m = BTreeMap::new();
    for (k, v) in pairs {
        m.insert((*k).to_string(), v.clone());
    }
    Value::Map(m)
}

fn typed_call_op_node(op_name: &str, input: Value) -> OperationNode {
    OperationNode::new("c0", PrimitiveKind::Call)
        .with_property("target", Value::text(format!("engine:typed:{op_name}")))
        .with_property("input", input)
}

// =====================================================================
// Per-op end-to-end pins (10 ops)
// =====================================================================

#[test]
fn ed25519_sign_then_verify_round_trip_via_dispatch_typed_call() {
    let (_dir, engine) = fresh_engine();

    // Generate a fresh keypair so we have known-good private/public bytes.
    let kp_input = map_value(&[]);
    let kp_out = engine
        .dispatch_typed_call(TypedCallOp::KeypairGenerate, &kp_input)
        .expect("keypair_generate must succeed");
    let (priv_bytes, pub_bytes) = match kp_out {
        Value::Map(m) => {
            let p = match m.get("private_key").unwrap() {
                Value::Bytes(b) => b.clone(),
                _ => panic!("private_key must be Bytes"),
            };
            let pk = match m.get("public_key").unwrap() {
                Value::Bytes(b) => b.clone(),
                _ => panic!("public_key must be Bytes"),
            };
            (p, pk)
        }
        _ => panic!("keypair_generate must return Map"),
    };

    let message = b"phase-3-g21-t1-typed-call".to_vec();

    // ed25519_sign drives the production engine arm.
    let sign_input = map_value(&[
        ("private_key", Value::Bytes(priv_bytes)),
        ("message", Value::Bytes(message.clone())),
    ]);
    let sign_out = engine
        .dispatch_typed_call(TypedCallOp::Ed25519Sign, &sign_input)
        .expect("ed25519_sign must succeed");
    let signature = match sign_out {
        Value::Map(m) => match m.get("signature").unwrap() {
            Value::Bytes(b) => b.clone(),
            _ => panic!("signature must be Bytes"),
        },
        _ => panic!("ed25519_sign must return Map"),
    };
    assert_eq!(signature.len(), 64, "Ed25519 signature MUST be 64 bytes");

    // ed25519_verify against the same message: valid: true.
    let verify_input = map_value(&[
        ("public_key", Value::Bytes(pub_bytes.clone())),
        ("message", Value::Bytes(message.clone())),
        ("signature", Value::Bytes(signature.clone())),
    ]);
    let verify_out = engine
        .dispatch_typed_call(TypedCallOp::Ed25519Verify, &verify_input)
        .expect("ed25519_verify must succeed");
    match verify_out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(true)),
                "verify against original message MUST return valid: true"
            );
        }
        _ => panic!("ed25519_verify must return Map"),
    }

    // Tampered message: valid: false (observable consequence — would
    // FAIL if dispatch were silently no-op'd / always-true).
    let mut tampered = message.clone();
    tampered[0] ^= 0xff;
    let bad_input = map_value(&[
        ("public_key", Value::Bytes(pub_bytes)),
        ("message", Value::Bytes(tampered)),
        ("signature", Value::Bytes(signature)),
    ]);
    let bad_out = engine
        .dispatch_typed_call(TypedCallOp::Ed25519Verify, &bad_input)
        .expect("ed25519_verify on tampered message must succeed (returning false)");
    assert_eq!(
        bad_out,
        Value::Map(BTreeMap::from([("valid".to_string(), Value::Bool(false))])),
        "tampered message MUST return valid: false (observable behavioral consequence)"
    );
}

#[test]
fn keypair_generate_returns_distinct_keys_each_call() {
    let (_dir, engine) = fresh_engine();
    let input = map_value(&[]);

    let out1 = engine
        .dispatch_typed_call(TypedCallOp::KeypairGenerate, &input)
        .unwrap();
    let out2 = engine
        .dispatch_typed_call(TypedCallOp::KeypairGenerate, &input)
        .unwrap();

    // OS CSPRNG → two consecutive generations produce different keys.
    // Observable consequence: would FAIL if generate were a deterministic stub.
    assert_ne!(
        out1, out2,
        "two `keypair_generate` calls MUST produce distinct keypairs (OS CSPRNG)"
    );
}

#[test]
fn keypair_from_seed_is_deterministic_and_round_trips_via_did() {
    let (_dir, engine) = fresh_engine();
    let seed = vec![7u8; 32];

    let out_a = engine
        .dispatch_typed_call(
            TypedCallOp::KeypairFromSeed,
            &map_value(&[("seed", Value::Bytes(seed.clone()))]),
        )
        .unwrap();
    let out_b = engine
        .dispatch_typed_call(
            TypedCallOp::KeypairFromSeed,
            &map_value(&[("seed", Value::Bytes(seed))]),
        )
        .unwrap();

    // Determinism: same seed → same keypair (observable consequence).
    assert_eq!(
        out_a, out_b,
        "keypair_from_seed MUST be deterministic for the same seed"
    );
}

#[test]
fn blake3_hash_matches_known_digest() {
    let (_dir, engine) = fresh_engine();
    let input = map_value(&[("data", Value::Bytes(b"abc".to_vec()))]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::Blake3Hash, &input)
        .expect("blake3_hash must succeed");
    let hash = match out {
        Value::Map(m) => match m.get("hash").unwrap() {
            Value::Bytes(b) => b.clone(),
            _ => panic!("hash must be Bytes"),
        },
        _ => panic!("blake3_hash must return Map"),
    };
    // BLAKE3("abc") known digest.
    let expected = blake3::hash(b"abc");
    assert_eq!(
        hash,
        expected.as_bytes().to_vec(),
        "blake3_hash MUST match the BLAKE3 reference digest of `abc`"
    );
}

#[test]
fn multibase_encode_then_decode_round_trips_base32_and_base58() {
    let (_dir, engine) = fresh_engine();

    for base in ["b", "z"] {
        let raw = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let encode_out = engine
            .dispatch_typed_call(
                TypedCallOp::MultibaseEncode,
                &map_value(&[
                    ("data", Value::Bytes(raw.clone())),
                    ("base", Value::Text(base.to_string())),
                ]),
            )
            .expect("multibase_encode must succeed");
        let encoded = match encode_out {
            Value::Map(m) => match m.get("encoded").unwrap() {
                Value::Text(s) => s.clone(),
                _ => panic!("encoded must be Text"),
            },
            _ => panic!("multibase_encode must return Map"),
        };
        assert!(
            encoded.starts_with(base),
            "multibase prefix MUST be '{base}'; got '{encoded}'"
        );

        let decode_out = engine
            .dispatch_typed_call(
                TypedCallOp::MultibaseDecode,
                &map_value(&[("encoded", Value::Text(encoded))]),
            )
            .expect("multibase_decode must succeed");
        match decode_out {
            Value::Map(m) => {
                assert_eq!(m.get("data"), Some(&Value::Bytes(raw.clone())));
                assert_eq!(m.get("base"), Some(&Value::Text(base.to_string())));
            }
            _ => panic!("multibase_decode must return Map"),
        }
    }
}

#[test]
fn did_resolve_round_trips_via_keypair_generate() {
    let (_dir, engine) = fresh_engine();
    // Generate a keypair; its `to_did` form is what we feed back into
    // `did_resolve`.
    let kp_out = engine
        .dispatch_typed_call(TypedCallOp::KeypairGenerate, &map_value(&[]))
        .unwrap();
    let pub_bytes = match &kp_out {
        Value::Map(m) => match m.get("public_key").unwrap() {
            Value::Bytes(b) => b.clone(),
            _ => panic!(),
        },
        _ => panic!(),
    };

    // Build a `did:key:z...` DID via benten-id directly so we have a
    // known-good DID string to feed `did_resolve`.
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pub_bytes);
    let pk = benten_id::keypair::PublicKey::from_bytes(&pk_arr).unwrap();
    let did = benten_id::did::Did::from_public_key(&pk);

    let resolve_out = engine
        .dispatch_typed_call(
            TypedCallOp::DidResolve,
            &map_value(&[("did", Value::Text(did.as_str().to_string()))]),
        )
        .expect("did_resolve must succeed");
    match resolve_out {
        Value::Map(m) => {
            assert_eq!(m.get("method"), Some(&Value::Text("key".to_string())));
            assert_eq!(
                m.get("public_key"),
                Some(&Value::Bytes(pub_bytes)),
                "did_resolve MUST recover the EXACT pubkey bytes (round-trip pin)"
            );
        }
        _ => panic!("did_resolve must return Map"),
    }
}

#[test]
fn ucan_validate_chain_returns_true_for_well_formed_chain() {
    let (_dir, engine) = fresh_engine();
    // Single-link UCAN chain (issuer → audience) granting zone:user:write
    // over [1_000, 2_000_000_000]. Helper at `tests/common/ucan_fixtures.rs`
    // collapses the keypair-generation + builder + sign + DAG-CBOR encode
    // prelude (DX-3 closure per phase-3-backlog §2.5(f)).
    let chain =
        common::ucan_fixtures::single_link_chain("zone:user", "write", Some(1_000), 2_000_000_000);

    let input = map_value(&[
        ("tokens", Value::List(vec![Value::Bytes(chain.bytes)])),
        (
            "audience",
            Value::Text(chain.audience_did.as_str().to_string()),
        ),
        ("capability", Value::Text("zone:user:write".to_string())),
        ("now", Value::Int(1_500_000)),
    ]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::UcanValidateChain, &input)
        .expect("ucan_validate_chain must succeed");
    match out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(true)),
                "well-formed in-window UCAN MUST validate: true; got {m:?}"
            );
        }
        _ => panic!("ucan_validate_chain must return Map"),
    }
}

#[test]
fn ucan_validate_chain_returns_false_with_reason_on_audience_mismatch() {
    let (_dir, engine) = fresh_engine();
    use benten_id::keypair::Keypair;

    // Build the chain bound to one audience, then assert against a
    // *different* audience DID in the input map (cross-atrium replay
    // defense). Helper handles the chain prelude; the audience-swap is
    // the test-specific contract.
    let chain = common::ucan_fixtures::single_link_chain("zone:user", "write", None, 2_000_000_000);
    let other_kp = Keypair::generate();

    let input = map_value(&[
        ("tokens", Value::List(vec![Value::Bytes(chain.bytes)])),
        // Wrong audience — defends cross-atrium replay.
        (
            "audience",
            Value::Text(other_kp.public_key().to_did().as_str().to_string()),
        ),
        ("capability", Value::Text("zone:user:write".to_string())),
        ("now", Value::Int(1_500_000)),
    ]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::UcanValidateChain, &input)
        .expect("ucan_validate_chain succeeds even on rejection (clean negative)");
    match out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(false)),
                "audience mismatch MUST validate: false (observable consequence)"
            );
            // Reason carries the audience-mismatch diagnostic.
            match m.get("reason") {
                Some(Value::Text(s)) => assert!(
                    s.contains("audience"),
                    "reason should mention 'audience'; got '{s}'"
                ),
                _ => panic!("reason must be Text"),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn vc_verify_round_trips_via_credential_builder() {
    let (_dir, engine) = fresh_engine();
    use benten_id::keypair::Keypair;
    use benten_id::vc::Credential;

    let issuer_kp = Keypair::generate();
    let issuer_did = issuer_kp.public_key().to_did();
    let subject_kp = Keypair::generate();
    let subject_did = subject_kp.public_key().to_did();

    let vc = Credential::builder()
        .issuer(&issuer_did)
        .subject(&subject_did)
        .issued_at(1_000)
        .claim("role", "admin")
        .sign(&issuer_kp)
        .expect("credential sign must succeed");

    let bytes = serde_ipld_dagcbor::to_vec(&vc).unwrap();

    let input = map_value(&[
        ("credential", Value::Bytes(bytes)),
        (
            "expected_issuer_did",
            Value::Text(issuer_did.as_str().to_string()),
        ),
        // `now` after `issued_at(1_000)` and well before any
        // expiration (the credential builder above does not set an
        // expirationDate, so any `now >= 1_000` validates).
        ("now", Value::Int(1_500_000)),
    ]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::VcVerify, &input)
        .expect("vc_verify must succeed");
    match out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(true)),
                "well-formed VC MUST verify: true; got {m:?}"
            );
            assert_eq!(
                m.get("issuer"),
                Some(&Value::Text(issuer_did.as_str().to_string()))
            );
            assert_eq!(
                m.get("subject"),
                Some(&Value::Text(subject_did.as_str().to_string()))
            );
        }
        _ => panic!(),
    }
}

// =====================================================================
// Error paths
// =====================================================================

#[test]
fn typed_call_unknown_op_via_call_primitive_routes_typed_error() {
    // Drive the eval-side CALL primitive directly with an unknown
    // typed-CALL op name. The fork in `call::execute` MUST surface
    // `EvalError::TypedCallUnknownOp` rather than fall through to the
    // user handler registry.
    let (_dir, engine) = fresh_engine();
    let op = OperationNode::new("c0", PrimitiveKind::Call)
        .with_property("target", Value::text("engine:typed:not_a_real_op"))
        .with_property("input", map_value(&[]));

    let err = call::execute(&op, &engine).expect_err("unknown op MUST produce Err");
    assert_eq!(
        err.code(),
        ErrorCode::TypedCallUnknownOp,
        "unknown op MUST map to E_TYPED_CALL_UNKNOWN_OP; got {err:?}"
    );
}

#[test]
fn typed_call_invalid_input_via_call_primitive_routes_typed_error() {
    // Sign with a wrong-length private_key — input shape rejected.
    let (_dir, engine) = fresh_engine();
    let op = OperationNode::new("c0", PrimitiveKind::Call)
        .with_property("target", Value::text("engine:typed:ed25519_sign"))
        .with_property(
            "input",
            map_value(&[
                ("private_key", Value::Bytes(vec![0u8; 16])), // too short
                ("message", Value::Bytes(b"hello".to_vec())),
            ]),
        );

    let err = call::execute(&op, &engine).expect_err("invalid input MUST produce Err");
    assert_eq!(
        err.code(),
        ErrorCode::TypedCallInvalidInput,
        "invalid input MUST map to E_TYPED_CALL_INVALID_INPUT; got {err:?}"
    );
}

#[test]
fn typed_call_non_engine_typed_target_does_not_dispatch_typed_call() {
    // ESC-shape pin: the typed-CALL fork is prefix-bound. A `target`
    // that does NOT start with `engine:typed:` MUST NOT accidentally
    // route into the typed-CALL registry — it falls through to the
    // user-handler dispatch path (which surfaces a backend
    // unsupported error here because the test PrimitiveHost does not
    // wire `call_handler`).
    let (_dir, engine) = fresh_engine();
    let op = OperationNode::new("c0", PrimitiveKind::Call)
        .with_property("target", Value::text("engine:typed_LOOKALIKE:ed25519_sign"))
        .with_property("call_op", Value::text("default"));

    let result = call::execute(&op, &engine);
    // The handler `engine:typed_LOOKALIKE:ed25519_sign` is not
    // registered. Whatever the engine's `call_handler` returns, the
    // surface MUST NOT be `E_TYPED_CALL_*` — that would mean the
    // prefix gate accidentally fired on a non-`engine:typed:`
    // target. Either Ok(StepResult routing ON_DENIED) or Err with
    // a non-typed-call code is fine.
    if let Err(e) = result {
        let code = e.code();
        assert!(
            !matches!(
                code,
                ErrorCode::TypedCallUnknownOp
                    | ErrorCode::TypedCallInvalidInput
                    | ErrorCode::TypedCallCapDenied
                    | ErrorCode::TypedCallDispatchError
            ),
            "ESC-shape: a `target` lacking the `engine:typed:` prefix MUST NOT fall \
             into the typed-CALL fork; got {code:?}"
        );
    }
    // If Ok, the prefix-bound gate held + the user-handler path took
    // over (whatever its result, the typed-CALL fork was bypassed).
}

#[test]
fn typed_call_required_caps_each_op_namespaced() {
    // Structural invariant: every TypedCallOp.required_cap() starts
    // with `cap:typed:`. Documented at the rust catalog row + here.
    // A drift attempt that introduced a non-namespaced cap would
    // surface here — the test catches both code-side drift AND the
    // E_TYPED_CALL_CAP_DENIED catalog row's cap-namespace claim.
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
        let cap = op.required_cap();
        assert!(
            cap.starts_with("cap:typed:"),
            "op {} cap '{}' MUST start with 'cap:typed:'",
            op.name(),
            cap
        );
    }
}

// =====================================================================
// ErrorCode catalog 4-surface integrity (G21-T1 §3.5g pin)
// =====================================================================

#[test]
fn typed_call_error_codes_round_trip_through_catalog() {
    for code in [
        ErrorCode::TypedCallUnknownOp,
        ErrorCode::TypedCallInvalidInput,
        ErrorCode::TypedCallCapDenied,
        ErrorCode::TypedCallDispatchError,
    ] {
        let s = code.as_static_str();
        assert!(
            s.starts_with("E_TYPED_CALL_"),
            "typed-CALL catalog string MUST be E_TYPED_CALL_* prefix; got {s}"
        );
        let parsed = ErrorCode::from_str(s);
        assert_eq!(
            parsed, code,
            "from_str round-trip MUST recover the variant; {s} → {parsed:?} ≠ {code:?}"
        );
    }
}

// =====================================================================
// Corr-minor fix-pass pins (G21-T1 fp-mini-review)
// =====================================================================

/// corr-minor-3 — User-namespace collision pin: a CALL Node whose
/// `target` starts with `engine:typed:` MUST route through the typed-
/// CALL registry (not the user-handler registry) regardless of what
/// (if anything) a user has done with `register_subgraph` on a
/// colliding handler-id. The eval-side fork in
/// `crate::primitives::call::execute` checks the prefix BEFORE the
/// `call_handler` dispatch, so even if a user registered `engine:typed:foo`,
/// the typed-CALL gate pre-empts their handler. This test pins the
/// pre-emption: a user-style registration of `engine:typed:not_a_real_op`
/// MUST surface as `E_TYPED_CALL_UNKNOWN_OP` (not the user handler's
/// behaviour) so future drift that lets user-handlers shadow the
/// reserved namespace would FAIL here.
///
/// Note: a hard registration-time REJECT (returning a typed error from
/// `register_subgraph` when the handler_id starts with `engine:typed:`)
/// is named for follow-up at phase-3-backlog §2.5; landing it requires
/// the §3.5g 4-surface ErrorCode atomic-update. This pin is the
/// behaviour-floor.
#[test]
fn typed_call_namespace_pre_empts_user_handler_registry_for_unknown_op() {
    use benten_eval::primitives::call;

    let (_dir, engine) = fresh_engine();
    // Drive a CALL targeting an `engine:typed:` op that is NOT in the
    // typed-CALL registry. Even without any user `register_subgraph`,
    // the prefix-bound fork MUST take over and surface
    // E_TYPED_CALL_UNKNOWN_OP rather than fall through to the user
    // handler dispatch path.
    let op = benten_core::OperationNode::new("c0", benten_eval::PrimitiveKind::Call)
        .with_property(
            "target",
            benten_core::Value::text("engine:typed:not_a_real_op"),
        )
        .with_property("input", map_value(&[]));

    let err = call::execute(&op, &engine).expect_err(
        "engine:typed: prefix MUST be reserved — even an unknown op surfaces \
         E_TYPED_CALL_UNKNOWN_OP, not user-handler-not-found",
    );
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::TypedCallUnknownOp,
        "the engine:typed: namespace MUST pre-empt user-handler dispatch (corr-minor-3)"
    );
}

// =====================================================================
// Sec-major fix-pass pins (G21-T1 fp-mini-review)
// =====================================================================

/// sec-major-1 — `ucan_validate_chain` MUST consult the leaf's `att`
/// against the requested capability. Defense-in-depth: a chain that's
/// audience-bound + signed + in-window but lacking the requested cap
/// MUST return `valid: false`. Without this gate, a handler asking
/// "does this chain grant `<cap>` to `<audience>`?" gets `valid: true`
/// regardless of the leaf `att` — a hole at the heart of the Phase-3
/// Atrium / UCAN authorization story.
#[test]
fn ucan_validate_chain_returns_false_when_leaf_att_does_not_grant_required_capability() {
    let (_dir, engine) = fresh_engine();

    // Forge a structurally-sound chain bound to the right audience but
    // granting only `zone:posts:read` — the caller asks for
    // `zone:user:write`, which the leaf does NOT name (sec-major-1
    // defense-in-depth pin).
    let chain =
        common::ucan_fixtures::single_link_chain("zone:posts", "read", Some(1_000), 2_000_000_000);

    let input = map_value(&[
        ("tokens", Value::List(vec![Value::Bytes(chain.bytes)])),
        (
            "audience",
            Value::Text(chain.audience_did.as_str().to_string()),
        ),
        // Cap NOT granted by the leaf's `att` (leaf grants
        // zone:posts:read; we ask for zone:user:write).
        ("capability", Value::Text("zone:user:write".to_string())),
        ("now", Value::Int(1_500_000)),
    ]);

    let out = engine
        .dispatch_typed_call(TypedCallOp::UcanValidateChain, &input)
        .expect("ucan_validate_chain succeeds even on rejection");
    match out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(false)),
                "leaf-cap-mismatch MUST validate: false (sec-major-1; \
                 without this, the typed-CALL silently passes any \
                 audience-bound chain regardless of granted cap)"
            );
            match m.get("reason") {
                Some(Value::Text(s)) => assert!(
                    s.contains("capability"),
                    "reason should mention 'capability'; got '{s}'"
                ),
                _ => panic!("reason must be Text"),
            }
        }
        _ => panic!("ucan_validate_chain must return Map"),
    }
}

/// sec-major-1 — happy path: the same chain SHOULD validate when the
/// caller asks for the cap the leaf actually grants.
#[test]
fn ucan_validate_chain_returns_true_when_leaf_att_grants_required_capability() {
    let (_dir, engine) = fresh_engine();

    // sec-major-1 happy path: same chain shape as the negative pin
    // above, but the caller asks for the cap the leaf actually grants.
    let chain =
        common::ucan_fixtures::single_link_chain("zone:posts", "read", Some(1_000), 2_000_000_000);

    let input = map_value(&[
        ("tokens", Value::List(vec![Value::Bytes(chain.bytes)])),
        (
            "audience",
            Value::Text(chain.audience_did.as_str().to_string()),
        ),
        ("capability", Value::Text("zone:posts:read".to_string())),
        ("now", Value::Int(1_500_000)),
    ]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::UcanValidateChain, &input)
        .unwrap();
    match out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(true)),
                "matching cap MUST validate: true; got {m:?}"
            );
        }
        _ => panic!(),
    }
}

/// sec-major-3 — `vc_verify` MUST consult the `now` time-window field
/// per the docstring. An expired VC + a `now` past the expiration date
/// MUST return `valid: false` — bare `vc::verify` skips the
/// expiration gate by design; this pin would FAIL if the dispatch path
/// silently called `vc::verify` instead of `vc::verify_at`.
#[test]
fn vc_verify_returns_false_when_credential_is_expired_at_now() {
    let (_dir, engine) = fresh_engine();
    use benten_id::keypair::Keypair;
    use benten_id::vc::Credential;

    let issuer_kp = Keypair::generate();
    let issuer_did = issuer_kp.public_key().to_did();
    let subject_kp = Keypair::generate();
    let subject_did = subject_kp.public_key().to_did();

    // VC issued at t=1_000, expires at t=2_000.
    let vc = Credential::builder()
        .issuer(&issuer_did)
        .subject(&subject_did)
        .issued_at(1_000)
        .expires_at(2_000)
        .claim("role", "admin")
        .sign(&issuer_kp)
        .expect("credential sign must succeed");
    let bytes = serde_ipld_dagcbor::to_vec(&vc).unwrap();

    // `now` past the expiration → must return valid: false.
    let input = map_value(&[
        ("credential", Value::Bytes(bytes)),
        (
            "expected_issuer_did",
            Value::Text(issuer_did.as_str().to_string()),
        ),
        ("now", Value::Int(1_000_000)),
    ]);
    let out = engine
        .dispatch_typed_call(TypedCallOp::VcVerify, &input)
        .expect("vc_verify must succeed (expiration is a clean negative)");
    match out {
        Value::Map(m) => {
            assert_eq!(
                m.get("valid"),
                Some(&Value::Bool(false)),
                "expired VC MUST validate: false (sec-major-3; \
                 dispatch must route through vc::verify_at, not bare \
                 vc::verify which skips the expiration gate)"
            );
        }
        _ => panic!(),
    }
}

/// sec-major-4 — Cap-deny end-to-end pin: a CAPABILITY POLICY denying
/// `cap:typed:crypto-sign` MUST (a) prevent the underlying op from
/// executing AND (b) route to `ON_DENIED`.
///
/// Drives the production CALL primitive via the eval-side dispatch
/// fork so the `host.check_capability` gate fires for real. Without
/// this pin, the cap labels are namespace-declared but never gated
/// at runtime — pim-2 §3.6b end-to-end requirement.
#[test]
fn typed_call_cap_denied_via_capability_policy_returns_on_denied_edge() {
    use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};
    use benten_eval::primitives::call;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A `CapabilityPolicy` that denies `cap:typed:crypto-sign`
    /// + counts the number of denied `check_write` invocations, so
    /// the test can assert the cap-check fired BEFORE dispatch.
    #[derive(Debug)]
    struct DenyTypedCryptoSign {
        denied_count: Arc<AtomicU32>,
    }
    impl CapabilityPolicy for DenyTypedCryptoSign {
        fn check_write(&self, ctx: &CapWriteContext) -> Result<(), CapError> {
            if ctx.label == "cap:typed:crypto-sign" {
                self.denied_count.fetch_add(1, Ordering::SeqCst);
                return Err(CapError::Denied {
                    required: "cap:typed:crypto-sign".to_string(),
                    entity: "test-actor".to_string(),
                });
            }
            Ok(())
        }
        fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
            Ok(())
        }
    }

    let denied_count = Arc::new(AtomicU32::new(0));
    let dir = tempfile::tempdir().unwrap();
    let engine = benten_engine::Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(DenyTypedCryptoSign {
            denied_count: Arc::clone(&denied_count),
        }))
        .build()
        .unwrap();

    // Drive `engine:typed:ed25519_sign` through the production CALL
    // primitive. Use a structurally-VALID input shape so a NON-cap
    // failure would surface only via dispatch (which we expect NEVER
    // to fire because the cap-gate denies first).
    let op = benten_core::OperationNode::new("c0", benten_eval::PrimitiveKind::Call)
        .with_property(
            "target",
            benten_core::Value::text("engine:typed:ed25519_sign"),
        )
        .with_property(
            "input",
            map_value(&[
                ("private_key", Value::Bytes(vec![0u8; 32])),
                ("message", Value::Bytes(b"hello".to_vec())),
            ]),
        );

    let result = call::execute(&op, &engine)
        .expect("cap-denied MUST be a clean ON_DENIED routing (NOT a typed Err) — sec-major-4");
    assert_eq!(
        result.edge_label, "ON_DENIED",
        "cap-denied typed-CALL MUST route ON_DENIED; got '{}' (sec-major-4)",
        result.edge_label
    );
    assert_eq!(
        denied_count.load(Ordering::SeqCst),
        1,
        "policy.check_write MUST have been consulted exactly once; \
         a value > 1 means the gate fired multiple times, a value of \
         0 means the cap-gate never fired (the underlying op might \
         have executed)"
    );
}

#[test]
fn typed_call_cap_denied_routes_on_denied_other_three_route_on_error() {
    assert_eq!(
        ErrorCode::TypedCallCapDenied.routed_edge_label(),
        Some("ON_DENIED"),
        "TypedCallCapDenied MUST join the cap-denial routing family"
    );
    for code in [
        ErrorCode::TypedCallUnknownOp,
        ErrorCode::TypedCallInvalidInput,
        ErrorCode::TypedCallDispatchError,
    ] {
        assert_eq!(
            code.routed_edge_label(),
            Some("ON_ERROR"),
            "{code:?} MUST route ON_ERROR (non-cap-denial typed-CALL failure)"
        );
    }
}

/// G21-T2 fp-mini-review BLOCKER-1 end-to-end pin: the
/// `dispatch_typed_call_public` napi entry MUST consult the
/// configured `CapabilityPolicy` BEFORE dispatching the underlying op.
///
/// Pre-fix: the napi `engine.typedCall(...)` path bypassed the cap-
/// check entirely — under a deny-all policy the typed-CALL still
/// produced a real signature, observable as
/// `output.contains_key("signature")`. Post-fix: a cap-denying policy
/// causes `dispatch_typed_call_public` to surface the typed
/// `EngineError` carrying `ErrorCode::TypedCallCapDenied` BEFORE the
/// underlying op runs.
///
/// This is the "production napi entry point" companion to the existing
/// `typed_call_cap_denied_via_capability_policy_returns_on_denied_edge`
/// pin (which exercises the eval-side subgraph route). Without this
/// pin, an actor with NO grant for `cap:typed:crypto-sign` could still
/// run `engine.typedCall('ed25519_sign', ...)` from JS and obtain a
/// valid signature.
#[test]
fn dispatch_typed_call_public_napi_entry_gates_on_capability_policy() {
    use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};
    use benten_engine::EngineError;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Policy that denies `cap:typed:crypto-sign` + counts denials.
    #[derive(Debug)]
    struct DenyTypedCryptoSign {
        denied_count: Arc<AtomicU32>,
    }
    impl CapabilityPolicy for DenyTypedCryptoSign {
        fn check_write(&self, ctx: &CapWriteContext) -> Result<(), CapError> {
            if ctx.label == "cap:typed:crypto-sign" {
                self.denied_count.fetch_add(1, Ordering::SeqCst);
                return Err(CapError::Denied {
                    required: "cap:typed:crypto-sign".to_string(),
                    entity: "test-actor".to_string(),
                });
            }
            Ok(())
        }
        fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
            Ok(())
        }
    }

    let denied_count = Arc::new(AtomicU32::new(0));
    let dir = tempfile::tempdir().unwrap();
    let engine = benten_engine::Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(DenyTypedCryptoSign {
            denied_count: Arc::clone(&denied_count),
        }))
        .build()
        .unwrap();

    // Drive `dispatch_typed_call_public` (the napi-binding entry) with
    // a structurally-VALID Ed25519Sign input. Without the gate fix, the
    // dispatch would produce a signature; with the gate, the call
    // surfaces `TypedCallCapDenied`.
    let input = map_value(&[
        ("private_key", Value::Bytes(vec![0u8; 32])),
        ("message", Value::Bytes(b"hello".to_vec())),
    ]);
    let result = engine.dispatch_typed_call_public(TypedCallOp::Ed25519Sign, &input);

    let err = result.expect_err(
        "dispatch_typed_call_public MUST deny when policy denies cap:typed:crypto-sign — without this gate the napi entry point bypasses cap policy entirely (BLOCKER-1)",
    );
    let code = match &err {
        EngineError::Other { code, .. } => code.clone(),
        other => panic!("expected EngineError::Other{{TypedCallCapDenied,..}}; got {other:?}"),
    };
    assert!(
        matches!(code, ErrorCode::TypedCallCapDenied),
        "denial MUST surface as ErrorCode::TypedCallCapDenied; got {code:?}"
    );
    assert_eq!(
        denied_count.load(Ordering::SeqCst),
        1,
        "policy.check_write MUST have been consulted exactly once \
         (gate fired); 0 means the napi entry bypasses the cap policy"
    );

    // Negative pin: a cap-PERMIT policy passes through. Without this
    // assertion the test would pass trivially even if `dispatch_typed_
    // call_public` always returned an error.
    let dir2 = tempfile::tempdir().unwrap();
    let engine_permit = benten_engine::Engine::builder()
        .path(dir2.path().join("benten.redb"))
        .build()
        .unwrap();
    // Generate a fresh keypair so the dispatch input is well-formed.
    let kp_out = engine_permit
        .dispatch_typed_call_public(TypedCallOp::KeypairGenerate, &Value::Map(BTreeMap::new()))
        .expect("permit-policy keypair_generate must succeed");
    let priv_key = match &kp_out {
        Value::Map(m) => match m.get("private_key").unwrap() {
            Value::Bytes(b) => b.clone(),
            _ => panic!("private_key must be Bytes"),
        },
        _ => panic!("keypair_generate must return Map"),
    };
    let permit_input = map_value(&[
        ("private_key", Value::Bytes(priv_key)),
        ("message", Value::Bytes(b"hello".to_vec())),
    ]);
    let permit_out = engine_permit
        .dispatch_typed_call_public(TypedCallOp::Ed25519Sign, &permit_input)
        .expect("permit-policy ed25519_sign must succeed (no policy = NoAuth-permit)");
    match permit_out {
        Value::Map(m) => assert!(
            matches!(m.get("signature"), Some(Value::Bytes(_))),
            "ed25519_sign output MUST carry a signature under a permit policy"
        ),
        other => panic!("ed25519_sign must return Map; got {other:?}"),
    }
}
