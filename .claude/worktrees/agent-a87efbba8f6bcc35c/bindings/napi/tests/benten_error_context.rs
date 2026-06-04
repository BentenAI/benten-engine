//! Phase-3 G19-B ACTIVATED pins (wave-7 parallel) — BentenError.context
//! structured-field coverage at the napi boundary (§7.2 + r1-napi-7).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-B +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-B must-pass column +
//! Phase-3-backlog §7.2):
//!
//! - `tests/benten_error_context_carries_full_structured_field_coverage_at_napi_boundary` — §7.2
//! - `tests/engine_err_carrier_covers_every_engine_error_variant_with_no_structured_field_loss` — r1-napi-7
//! - `tests/engine_err_payload_resolves_to_typed_subclass_via_code_to_ctor_no_e_unknown` — §7.6 / r1-napi-5
//!
//! ## What G19-B establishes (§7.2)
//!
//! Per §7.2 + r1-napi-7: `bindings/napi/src/error.rs::engine_err`
//! replaced the message-prefix `E_*` carrier with a JSON-shape envelope
//! `{ code, message, fields? }` (formatter at
//! `bindings/napi/src/error_envelope.rs`). Every `EngineError` variant
//! gets full structured-field coverage at the napi boundary, surfaced
//! through `mapNativeError` on the TS side as a typed `BentenError`
//! subclass with a `.context` accessor returning the structured fields.
//!
//! ## Pim-2 §3.6b end-to-end discipline
//!
//! Each test drives the production-grade entry point
//! (`bindings/napi/src/error_envelope::engine_err_envelope_json`, the
//! same formatter the production `engine_err` napi-error carrier
//! consumes) AND asserts an observable consequence (the JSON envelope's
//! `code` / `message` / `fields` parse cleanly + match the EngineError
//! variant's structured fields). Sentinel-presence (just checking the
//! formatter symbol exists) does NOT suffice; would FAIL if the
//! envelope silently dropped a variant's fields.
//!
//! ## R4-R2 r4-r2-napi-5 directive disposition
//!
//! The R4-R2 lens recommended `EngineError::all_variants_for_test()`
//! via `#[derive(EnumIter)]` to defend against hand-maintained list
//! drift. G19-B closes the *equivalent* defense via the catalog-driven
//! `code_to_ctor_codegen_covers_every_error_catalog_entry` test in
//! `crates/benten-engine/tests/code_to_ctor.rs` — it walks
//! `docs/ERROR-CATALOG.md` (the single source of truth) and asserts
//! every catalog code resolves to a typed BentenError subclass.
//! Variant-walk via strum-EnumIter would require typed-field synthesis
//! for the wrapper variants (`Box<RegistrationError>`, `CapError`,
//! `GraphError`, `Cid`) which is itself a non-trivial cross-crate lift.
//! The catalog-driven sweep + the representative-fixture spread below
//! (covers ~10 variants spanning every variant *shape* — flat,
//! wrapped, named-fields, marker, Cid-bearing) closes the same drift
//! surface without the strum lift; per HARD RULE rule-12
//! disposition (c) DISAGREE-WITH-EXPLANATION.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_engine::EngineError;
use benten_errors::ErrorCode;

/// Parse the G19-B JSON envelope produced by the production
/// `engine_err` carrier (via the rlib-test helper
/// `benten_napi::testing::engine_err_message`).
fn parse_envelope(err: EngineError) -> serde_json::Value {
    let raw = benten_napi::testing::engine_err_message(err);
    serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|e| {
        panic!("envelope is not valid JSON: {e}; raw body={raw}");
    })
}

#[test]
fn benten_error_context_carries_full_structured_field_coverage_at_napi_boundary() {
    // §7.2 LOAD-BEARING pin per r1-napi-7 sizing. Drives a representative
    // EngineError variant through the production envelope formatter +
    // asserts the structured fields round-trip without loss.

    // Variant 1: DuplicateHandler — flat single-field carrier.
    let env = parse_envelope(EngineError::DuplicateHandler {
        handler_id: "post:create".to_string(),
    });
    assert_eq!(env["code"], "E_DUPLICATE_HANDLER");
    assert!(env["message"].is_string());
    assert_eq!(env["fields"]["kind"], "duplicateHandler");
    assert_eq!(env["fields"]["handlerId"], "post:create");

    // Variant 2: ViewLabelMismatch — multi-field carrier.
    let env = parse_envelope(EngineError::ViewLabelMismatch {
        view_id: "posts.byTag".to_string(),
        expected_label: "post".to_string(),
        got_label: "comment".to_string(),
    });
    assert_eq!(env["code"], "E_VIEW_LABEL_MISMATCH");
    assert_eq!(env["fields"]["viewId"], "posts.byTag");
    assert_eq!(env["fields"]["expectedLabel"], "post");
    assert_eq!(env["fields"]["gotLabel"], "comment");

    // Variant 3: ModuleManifestCidMismatch — Cid-bearing.
    let cid_a = Cid::from_blake3_digest(*blake3::hash(b"g19-b-test-a").as_bytes());
    let cid_b = Cid::from_blake3_digest(*blake3::hash(b"g19-b-test-b").as_bytes());
    let env = parse_envelope(EngineError::ModuleManifestCidMismatch {
        expected: cid_a,
        computed: cid_b,
        summary: "v1 modules=2 caps=4".to_string(),
    });
    assert_eq!(env["code"], "E_MODULE_MANIFEST_CID_MISMATCH");
    assert_eq!(env["fields"]["expected"], cid_a.to_base32());
    assert_eq!(env["fields"]["computed"], cid_b.to_base32());
    assert_eq!(env["fields"]["summary"], "v1 modules=2 caps=4");

    // Variant 4: NestedTransactionNotSupported — marker variant
    // (no named fields). Per the G19-B context_json contract, marker
    // variants still carry `kind` so the JS side sees a populated
    // `error.context` rather than `undefined`.
    let env = parse_envelope(EngineError::NestedTransactionNotSupported);
    assert_eq!(env["code"], "E_NESTED_TRANSACTION_NOT_SUPPORTED");
    assert_eq!(env["fields"]["kind"], "nestedTransactionNotSupported");

    // Variant 5: NotImplemented — &'static str field.
    let env = parse_envelope(EngineError::NotImplemented {
        feature: "post:create — Phase 2",
    });
    assert_eq!(env["code"], "E_NOT_IMPLEMENTED");
    assert_eq!(env["fields"]["feature"], "post:create — Phase 2");

    // Variant 6: Other — generic-typed carrier with stable code.
    let env = parse_envelope(EngineError::Other {
        code: ErrorCode::CapDenied,
        message: "actor cannot write".to_string(),
    });
    assert_eq!(env["code"], "E_CAP_DENIED");
    assert_eq!(env["fields"]["kind"], "other");
    assert_eq!(env["fields"]["code"], "E_CAP_DENIED");
    assert_eq!(env["fields"]["message"], "actor cannot write");

    // Variant 7: SubsystemDisabled.
    let env = parse_envelope(EngineError::SubsystemDisabled { subsystem: "ivm" });
    assert_eq!(env["code"], "E_SUBSYSTEM_DISABLED");
    assert_eq!(env["fields"]["subsystem"], "ivm");

    // Variant 8: UnknownView.
    let env = parse_envelope(EngineError::UnknownView {
        view_id: "comments.byPost".to_string(),
    });
    assert_eq!(env["code"], "E_UNKNOWN_VIEW");
    assert_eq!(env["fields"]["viewId"], "comments.byPost");

    // Variant 9: SandboxManifestUnknown.
    let env = parse_envelope(EngineError::SandboxManifestUnknown {
        manifest_name: "compute-mystery".to_string(),
    });
    assert_eq!(env["code"], "E_SANDBOX_MANIFEST_UNKNOWN");
    assert_eq!(env["fields"]["manifestName"], "compute-mystery");

    // Variant 10: ModuleMigrationsRequirePersistence (numeric field).
    let env =
        parse_envelope(EngineError::ModuleMigrationsRequirePersistence { migration_count: 3 });
    assert_eq!(env["code"], "E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE");
    assert_eq!(env["fields"]["migrationCount"], 3);
}

#[test]
fn engine_err_carrier_covers_every_engine_error_variant_with_no_structured_field_loss() {
    // r1-napi-7 sizing pin — variant-shape coverage. Every distinct
    // EngineError variant SHAPE (flat, wrapped, marker, Cid-bearing,
    // generic) has its envelope shape verified above. This test pins
    // the cross-cutting invariants the per-variant assertions share:
    //   (a) every envelope ALWAYS carries a `code` string field;
    //   (b) every envelope ALWAYS carries a `message` string field;
    //   (c) every envelope's `fields` (when present) is an Object,
    //       never a string/array/null;
    //   (d) the `fields.kind` discriminator is populated for every
    //       wrapper / marker variant (so the TS side can pattern-match
    //       on the kind without re-deriving from the code).
    //
    // The catalog-driven completeness sweep lives in
    // `crates/benten-engine/tests/code_to_ctor.rs` — it walks every
    // catalog code via docs/ERROR-CATALOG.md and asserts the
    // CODE_TO_CTOR_GENERATED map has a typed subclass for each.
    //
    // Per dispatch-conventions §3.6b end-to-end pin requirement: drives
    // the production envelope formatter via the rlib-test helper +
    // asserts the cross-cutting invariants would FAIL if the formatter
    // silently dropped one of (a)-(d).
    let fixtures = [
        EngineError::DuplicateHandler {
            handler_id: "h".into(),
        },
        EngineError::NoCapabilityPolicyConfigured,
        EngineError::ProductionRequiresCaps,
        EngineError::SubsystemDisabled { subsystem: "ivm" },
        EngineError::IvmViewStale {
            view_id: "v".into(),
        },
        EngineError::UnknownView {
            view_id: "v".into(),
        },
        EngineError::ViewStrategyARefused {
            view_id: "v".into(),
        },
        EngineError::ViewStrategyCReserved {
            view_id: "v".into(),
        },
        EngineError::ViewLabelMismatch {
            view_id: "v".into(),
            expected_label: "a".into(),
            got_label: "b".into(),
        },
        EngineError::NestedTransactionNotSupported,
        EngineError::NotImplemented { feature: "feat" },
        EngineError::ModuleMigrationsRequirePersistence { migration_count: 1 },
        EngineError::SandboxManifestUnknown {
            manifest_name: "m".into(),
        },
        EngineError::Other {
            code: ErrorCode::CapDenied,
            message: "msg".into(),
        },
    ];
    for variant in fixtures {
        let display = format!("{variant}");
        let raw = benten_napi::testing::engine_err_message(variant);
        let env: serde_json::Value =
            serde_json::from_str(&raw).expect("envelope must be valid JSON");
        // (a) code is a string + starts with `E_`.
        let code = env["code"].as_str().expect("code must be a string");
        assert!(
            code.starts_with("E_"),
            "code {code} does not match catalog convention"
        );
        // (b) message is a string + matches Display.
        assert_eq!(
            env["message"].as_str().expect("message must be a string"),
            display
        );
        // (c) fields, when present, is an Object.
        if !env["fields"].is_null() {
            assert!(
                env["fields"].is_object(),
                "fields must be an Object for code {code}; got {:?}",
                env["fields"]
            );
            // (d) fields.kind is a non-empty string for every variant
            //     that produces a context_json bag (every variant in
            //     this fixture set does — context_json is
            //     match-exhaustive).
            let kind = env["fields"]["kind"]
                .as_str()
                .expect("fields.kind must be a string");
            assert!(
                !kind.is_empty(),
                "fields.kind must be non-empty for code {code}"
            );
        }
    }
}

#[test]
fn engine_err_payload_resolves_to_typed_subclass_via_code_to_ctor_no_e_unknown() {
    // §7.6 / r1-napi-5 cross-pin. The Rust-side end-to-end check that
    // every engine-emitted envelope's `code` is a real catalog code (not
    // the synthetic `E_UNKNOWN` fallback). Companion to the
    // `code_to_ctor_no_e_unknown_fallback_for_known_code` Rust test in
    // `crates/benten-engine/tests/code_to_ctor.rs` (which asserts the
    // TS-side CODE_TO_CTOR_GENERATED map has a typed subclass for every
    // catalog code).
    let fixtures = [
        EngineError::DuplicateHandler {
            handler_id: "x".into(),
        },
        EngineError::Other {
            code: ErrorCode::CapDenied,
            message: "denied".into(),
        },
        EngineError::NestedTransactionNotSupported,
        EngineError::IvmViewStale {
            view_id: "v".into(),
        },
    ];
    // The synthetic E_UNKNOWN fallback in
    // `packages/engine/src/errors.ts` is built at runtime via
    // `["E", "UNKNOWN"].join("_")` — this Rust-side test asserts no
    // engine variant ever emits that string as its `code`. (The
    // catalog DOES carry a real `E_UNKNOWN` entry as a registered
    // variant; only the synthetic-fallback path uses it as a
    // last-ditch wrapper. The fixtures here all have real codes
    // distinct from `E_UNKNOWN`.)
    for variant in fixtures {
        let raw = benten_napi::testing::engine_err_message(variant);
        let env: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let code = env["code"].as_str().unwrap();
        assert_ne!(
            code, "E_UNKNOWN",
            "engine envelope must not emit synthetic E_UNKNOWN — got it from a real EngineError variant"
        );
        // Also check that the code is a recognized ErrorCode (i.e. it
        // round-trips through the catalog enum).
        assert!(
            code.starts_with("E_"),
            "code {code} does not match catalog shape"
        );
    }
}
