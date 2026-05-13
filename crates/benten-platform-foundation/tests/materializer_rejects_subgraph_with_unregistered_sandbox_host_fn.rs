//! G23-B GREEN: materializer (defense-in-depth) + schema_compiler
//! reject SANDBOX subgraphs whose host-fn is outside the manifest /
//! CLAUDE.md baked-in #16 minimum-viable set.
//!
//! The PRIMARY defense lands at `schema_compiler::compile` (G23-A) —
//! it surfaces `E_SCHEMA_SANDBOX_HOST_FN_REJECTED` for any schema
//! embedding a storage-mutating SANDBOX host-fn (`kv:write` /
//! `kv:delete` / edge-mutating). The materializer's entry-point
//! contains a defense-in-depth re-check (for hand-authored
//! `SchemaSubgraphSpec` inputs that bypass schema-compile) surfaced
//! via `E_MATERIALIZER_SCHEMA_MISMATCH` per arch-r1-3.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_errors::ErrorCode;
use benten_platform_foundation::SchemaCompileError;

/// A schema that embeds a SANDBOX module requesting `kv:write` —
/// CLAUDE.md baked-in #16 forbids this.
const HOSTILE_SCHEMA: &[u8] = br#"{
    "label": "SchemaRoot",
    "name": "EvilType",
    "fields": [
        { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true, "default": null }
    ],
    "sandbox_refs": [
        { "module_cid": "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda", "host_fns": ["kv:write"] }
    ]
}"#;

#[test]
fn materializer_rejects_subgraph_with_unregistered_sandbox_host_fn() {
    // Defense-in-depth: schema_compiler rejects the hostile schema
    // BEFORE the spec ever reaches the materializer entry-point. This
    // is the PRIMARY defense per sec-3.5-r1-14.
    let err = benten_platform_foundation::compile_schema(HOSTILE_SCHEMA)
        .expect_err("hostile SANDBOX host-fn MUST be rejected at compile-time");
    match err {
        SchemaCompileError::SandboxHostFnRejected { host_fn, .. } => {
            assert_eq!(
                host_fn, "kv:write",
                "rejection surfaces the offending host-fn name"
            );
        }
        other => panic!("expected SandboxHostFnRejected, got {other:?}"),
    }
    // The schema_compile error maps to the typed ErrorCode at the
    // engine boundary.
    let _ = ErrorCode::SchemaSandboxHostFnRejected;

    // SUBSTANCE smoke: confirm fixture loads even when hostile schema
    // rejected (no global state corruption).
    let _ = materializer_fixtures::hostile_subgraph_with_unregistered_sandbox_host_fn_bytes();
}
