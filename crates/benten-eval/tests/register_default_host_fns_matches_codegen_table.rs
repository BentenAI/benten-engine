//! R6FP-Group-1 (r6-wsa-7) regression pin —
//! `register_default_host_fns` covers every name in the codegen
//! `default_host_fns` table; an unknown wasm-import surfaces a typed
//! error rather than silently NO-OPing.
//!
//! Pre-fix, the walker at sandbox.rs:678 had a `match host_name.as_str()`
//! arm covering only "time", "log", "kv:read" with a silent `_ => {}`
//! fallthrough. If a future entry lands in the codegen table (D1 lift
//! to add "random" in Phase 3 — see phase-3-backlog §6.10 — or any
//! other host-fn), the walker
//! silently does NOT register it. Then either:
//!   (i) the module imports it → wasmtime raises "unknown import" → the
//!   trap_to_typed maps to HostFnNotFound which the operator reads as
//!   "manifest is wrong" rather than "codegen out-of-sync with
//!   registration"; or
//!   (ii) the module doesn't import it → the host-fn is unreachable but
//!   the cap is still consumed.
//! Either way, a future host-fn addition is silently broken.
//!
//! This test asserts the symmetric contract: every name in
//! `host_fn_names()` (the canonical codegen list) is one of the names
//! the registration walker handles. Adds an explicit drift detector so
//! a future codegen entry without a matching registration arm fails at
//! test time rather than at runtime cap denial.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::AttributionFrame;
use benten_eval::primitives::sandbox;
use benten_eval::sandbox::{CapBundle, ManifestRef, ManifestRegistry, SandboxConfig};

fn zero_cid() -> benten_core::Cid {
    benten_core::Cid::from_blake3_digest([0u8; 32])
}

/// R6FP-G1 (r6-wsa-7): a guest module that imports a host-fn that
/// is NOT in the codegen table surfaces SandboxHostFnNotFound (or
/// ModuleInvalid via wasmtime "unknown import"), NEVER a silent
/// success. Pre-fix the silent-fallthrough at register_default_host_fns
/// would have masked some unknown-name bugs.
#[test]
fn register_default_host_fns_unknown_wasm_import_surfaces_typed_error() {
    // Build a guest module that imports a host-fn name not in the
    // canonical D1 set. wasmtime's link-time check fires — the
    // unknown import surfaces as a typed error rather than a silent
    // success.
    let bytes = wat::parse_str(
        r#"
        (module
          (import "host" "totally_undefined_host_fn" (func $undef (result i32)))
          (func (export "run") (result i32) call $undef)
        )
        "#,
    )
    .expect("test wat compiles");

    let registry = ManifestRegistry::new();
    let manifest_ref = ManifestRef::Inline(CapBundle::new(Vec::new(), None));
    let config = SandboxConfig::default();

    let attribution = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 1,
        ..Default::default()
    };

    let result = sandbox::execute(&bytes, manifest_ref, &registry, config, &[], &attribution);
    match result {
        Err(_typed_err) => {
            // Any typed SandboxError variant is acceptable —
            // HostFnNotFound (ESC-15) or ModuleInvalid via wasmtime
            // "unknown import". The load-bearing assertion is that
            // the call did NOT silently succeed.
        }
        Ok(_) => panic!(
            "guest module importing an unknown host-fn name MUST surface a \
             typed SandboxError, never silently succeed (R6FP-G1 r6-wsa-7: \
             pre-fix the register_default_host_fns silent fallthrough \
             would have left some unknown-name bugs unobservable until \
             runtime cap denial)"
        ),
    }
}

/// R6FP-G1 (r6-wsa-7) drift detector: every name returned by
/// `host_fn_names()` is in the closed set of names the registration
/// walker handles ({"time", "log", "kv:read", "random"}). When D1
/// expands the canonical set to add a new host-fn, BOTH this drift
/// detector AND the register_default_host_fns match-arm must be
/// updated together — failure at test-time is preferable to failure
/// at runtime cap denial. Phase-3 G17-A2 wave-5b added "random" per
/// D-PHASE-3-11 (CSPRNG via getrandom + cap-policy budget per call).
#[test]
fn register_default_host_fns_walker_covers_every_codegen_entry_name() {
    let codegen_names: Vec<&str> = benten_eval::sandbox::host_fns::host_fn_names().to_vec();
    // The match arms in `register_default_host_fns` at sandbox.rs:743
    // cover exactly these names — keep this set in sync when the
    // canonical D1 surface widens.
    let registered_names: &[&str] = &["time", "log", "kv:read", "random"];
    for name in &codegen_names {
        assert!(
            registered_names.contains(name),
            "host_fn_names() returns {name:?} but register_default_host_fns \
             does not cover it. Pre-R6FP-G1 (r6-wsa-7) the silent fallthrough \
             arm `_ => {{}}` would have left this entry unregistered at \
             link time; ESC-15 / D1 contracts require every codegen entry \
             to have a matching registration arm. Add the arm in \
             register_default_host_fns before merging."
        );
    }
}
