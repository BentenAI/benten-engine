//! R3 Family D RED-PHASE pin for G23-A SANDBOX host-fn rejection
//! (sec-3.5-r1-14 + CLAUDE.md baked-in #16; negative substantive).
//!
//! Pin source: r2-test-landscape §2.4 row 8.
//!
//! ## What this pin defends
//!
//! CLAUDE.md baked-in #16: storage-mutating host-fns (`kv:write`, `kv:delete`,
//! edge-mutating) are explicitly NOT engine concerns — they would be parallel
//! write pathways that bypass the WRITE primitive's capability gating + Inv-13
//! firing matrix + IVM materialization seam.
//!
//! The schema compiler MUST reject schemas whose embedded SANDBOX references
//! request these host-fns. Surface: `E_SCHEMA_SANDBOX_HOST_FN_REJECTED`.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

// Un-ignored at G23-A wave-4 (2026-05-12 canary).
#[test]
fn schema_compiler_rejects_schema_referencing_sandbox_with_storage_mutating_host_fn_request() {
    use benten_errors::ErrorCode;
    use benten_platform_foundation::schema_compiler::compile;

    let bytes = schema_fixtures::hostile_schema_with_sandbox_kv_write_bytes();
    let err =
        compile(bytes).expect_err("schema with SANDBOX kv:write request MUST be rejected per CLAUDE.md #16");
    assert_eq!(
        err.code(),
        ErrorCode::SchemaSandboxHostFnRejected,
        "must surface E_SCHEMA_SANDBOX_HOST_FN_REJECTED"
    );

    // Symmetric defense — kv:delete + edge-mutating host-fns also rejected.
    for forbidden_host_fn in ["kv:delete", "edges:add", "edges:remove"] {
        let mut buf = Vec::from(schema_fixtures::hostile_schema_with_sandbox_kv_write_bytes());
        let pre = buf
            .windows(b"kv:write".len())
            .position(|w| w == b"kv:write")
            .unwrap();
        buf.splice(pre..pre + b"kv:write".len(), forbidden_host_fn.bytes());
        let err = compile(&buf)
            .err()
            .unwrap_or_else(|| panic!("must reject host_fn `{forbidden_host_fn}`"));
        assert_eq!(
            err.code(),
            ErrorCode::SchemaSandboxHostFnRejected,
            "host_fn `{forbidden_host_fn}` must surface E_SCHEMA_SANDBOX_HOST_FN_REJECTED"
        );
    }
}
