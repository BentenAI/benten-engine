//! G17-A2 GREEN-PHASE pins for `random` host-fn workspace CSPRNG
//! (D-PHASE-3-11 RESOLVED-at-R1 + r1-wsa-8 + CLAUDE.md baked-in #16
//! closure / Compromise #16).
//!
//! The four pins below all drive the production `benten_eval::sandbox::execute`
//! entry point + assert observable behavioral consequences (per pim-2
//! §3.6b end-to-end test pin requirement). Each test would FAIL if the
//! random host-fn arm were silently no-op'd back to its Phase-2b
//! deferred shape.
//!
//! - `random_host_fn_csprng_round_trip` — getrandom-backed entropy
//!   delivered to guest memory.
//! - `random_host_fn_capability_gated_entropy_budget` — per-INVOCATION
//!   budget enforcement (4096-byte default per r1-wsa-8) routes the
//!   typed `E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED` variant.
//! - `random_host_fn_per_manifest_budget_override_via_module_manifest_field`
//!   — `SandboxConfig::random_budget_bytes_per_call` flows through the
//!   trampoline (additive optional manifest override per r1-wsa-8).
//! - `sandbox_host_fn_random_no_longer_returns_deferred_error` — the
//!   Phase-2b validate-time deferral guard (`§6.10` cite) is GONE
//!   (CLAUDE.md baked-in #16 closure).

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, SandboxError, execute,
};

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
        ..Default::default()
    }
}

/// Build a small WAT module that calls `host.random(buf_ptr, len)` and
/// returns the result. The buffer lives at offset 1024 in the module's
/// linear memory; the entry function `run` invokes random with the
/// caller-stamped len constant + returns 0 on success.
fn random_module_wat(len: u32) -> Vec<u8> {
    // WASM section ordering: imports MUST precede memory declarations.
    let wat = format!(
        r#"
        (module
            (import "host" "random" (func $random (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "run") (result i32)
                (call $random (i32.const 1024) (i32.const {len}))
            )
        )
        "#
    );
    wat::parse_str(&wat).unwrap()
}

#[test]
fn random_host_fn_csprng_round_trip() {
    // D-PHASE-3-11 RESOLVED-at-R1 — `getrandom` direct CSPRNG wired at
    // the production `benten_eval::sandbox::execute` entry point. A
    // module that requests 64 bytes of entropy succeeds + the
    // production trampoline writes the bytes through `getrandom` (the
    // Linker arm in `register_default_host_fns`'s "random" branch).
    //
    // Observable consequence: `execute` returns Ok with output (the i32
    // 0 from `run` is the host-fn return code, asserting the call
    // completed without trapping). A regression that no-op'd the arm
    // would either fail to link (E_SANDBOX_HOST_FN_NOT_FOUND) or trap
    // at the memory write — both observable.
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:random:read".to_string()], None);
    let bytes = random_module_wat(64);
    let attribution = dummy_attribution();
    let result = execute(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        SandboxConfig::default(),
        &["host:random:read".to_string()],
        &attribution,
    );
    assert!(
        result.is_ok(),
        "G17-A2 — random host-fn 64-byte request MUST succeed against \
         default budget (4096); got {:?}",
        result.as_ref().err()
    );
}

#[test]
fn random_host_fn_capability_gated_entropy_budget() {
    // D-PHASE-3-11 + r1-wsa-8 — per-INVOCATION budget. A request that
    // exceeds the codegen default (4096 bytes per call) MUST fire the
    // typed `SandboxHostFnRandomBudgetExceeded` variant. This pin
    // would FAIL if the trampoline silently returned without budget
    // enforcement (sentinel-presence test would not catch that; an
    // observable consequence is mandatory per pim-2 §3.6b).
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:random:read".to_string()], None);
    let bytes = random_module_wat(8192); // > 4096 default
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        SandboxConfig::default(),
        &["host:random:read".to_string()],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(
        err.code(),
        ErrorCode::SandboxHostFnRandomBudgetExceeded,
        "r1-wsa-8 — 8192-byte request MUST exceed 4096-byte default + \
         route to E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED; got: {err:?}"
    );
    if let SandboxError::HostFnDenied { cap } = &err {
        assert!(
            cap.contains("random:per_call_budget_exceeded"),
            "cap-string carrier MUST identify the budget-exceed kind; got: {cap}"
        );
        assert!(
            cap.contains("requested=8192"),
            "cap-string MUST carry requested-byte count for operator hint; got: {cap}"
        );
        assert!(
            cap.contains("budget=4096"),
            "cap-string MUST carry active budget for operator hint; got: {cap}"
        );
    } else {
        panic!("expected HostFnDenied carrier; got: {err:?}");
    }
}

#[test]
fn random_host_fn_per_manifest_budget_override_via_module_manifest_field() {
    // r1-wsa-8 — additive optional `host_fns.random.budget_bytes_per_call`
    // override flows through `SandboxConfig::random_budget_bytes_per_call`
    // into `SandboxStoreData::random_budget_bytes_per_call` at the
    // trampoline. A 1024-byte budget rejects a 2048-byte request even
    // though that request would PASS under the codegen default.
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:random:read".to_string()], None);
    let attribution = dummy_attribution();

    // Tight 1024-byte override. Within budget call: 512 bytes — succeeds.
    let mut tight_config = SandboxConfig::default();
    tight_config.random_budget_bytes_per_call = Some(1024);
    let bytes_ok = random_module_wat(512);
    let ok = execute(
        &bytes_ok,
        ManifestRef::Inline(inline.clone()),
        &registry,
        tight_config.clone(),
        &["host:random:read".to_string()],
        &attribution,
    );
    assert!(
        ok.is_ok(),
        "512-byte request MUST fit 1024-byte override; got {:?}",
        ok.as_ref().err()
    );

    // 2048-byte request EXCEEDS the override (would PASS under default
    // 4096) — confirms the override actually took effect. Without the
    // override flowing through the trampoline, this test would PASS
    // (false-green) at the production code's pre-fix state.
    let bytes_denied = random_module_wat(2048);
    let denied = execute(
        &bytes_denied,
        ManifestRef::Inline(inline),
        &registry,
        tight_config,
        &["host:random:read".to_string()],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(
        denied.code(),
        ErrorCode::SandboxHostFnRandomBudgetExceeded,
        "2048-byte request MUST be denied by the 1024-byte manifest \
         override (per r1-wsa-8); the override would silently be \
         dropped if the SandboxConfig→SandboxStoreData wire were broken"
    );
    if let SandboxError::HostFnDenied { cap } = &denied {
        assert!(
            cap.contains("budget=1024"),
            "cap-string MUST reflect the 1024-byte override (NOT the codegen default 4096); \
             got: {cap}"
        );
    }
}

#[test]
fn sandbox_host_fn_random_no_longer_returns_deferred_error() {
    // CLAUDE.md baked-in #16 + Compromise #16 closure pim-2 §3.6b
    // anti-regression pin. Source-cite assertion: the Phase-2b validate-
    // time deferral guard at `crates/benten-eval/src/primitives/sandbox.rs`
    // (sec-g7a-mr-5) is GONE.
    let dispatch_src = std::fs::read_to_string("../benten-eval/src/primitives/sandbox.rs")
        .unwrap_or_else(|_| {
            std::fs::read_to_string("crates/benten-eval/src/primitives/sandbox.rs").unwrap()
        });
    assert!(
        !dispatch_src.contains("DEFERRED_HOST_FN_RANDOM_CAP_PREFIX: &str ="),
        "Phase-2b sec-g7a-mr-5 const definition MUST be removed at G17-A2 \
         (CLAUDE.md baked-in #16 closure / Compromise #16)"
    );
    assert!(
        !dispatch_src.contains("phase-3-backlog.md §6.10 for the workspace CSPRNG"),
        "Phase-2b deferral hint copy MUST be gone post-G17-A2"
    );

    // host_fns.rs source MUST mention the CSPRNG primitive (getrandom)
    // — the codegen-default surface includes `random`.
    let host_fns_src = std::fs::read_to_string("../benten-eval/src/sandbox/host_fns.rs")
        .unwrap_or_else(|_| {
            std::fs::read_to_string("crates/benten-eval/src/sandbox/host_fns.rs").unwrap()
        });
    assert!(
        host_fns_src.contains("HostFnBehavior::Random"),
        "host_fns.rs MUST declare the Random behavior variant at G17-A2"
    );

    // The codegen surface must contain `random` + the per-call budget
    // default 4096 (r1-wsa-8 default).
    assert!(
        host_fns_src.contains("DEFAULT_RANDOM_BUDGET_BYTES_PER_CALL"),
        "host_fns.rs MUST declare the public default budget constant at G17-A2"
    );

    // host-functions.toml flag flipped — declares random.
    let toml = std::fs::read_to_string("../../host-functions.toml")
        .unwrap_or_else(|_| std::fs::read_to_string("host-functions.toml").unwrap());
    assert!(
        toml.contains("[host_fn.random]"),
        "host-functions.toml MUST declare [host_fn.random] section at G17-A2 (Compromise #16 closure)"
    );
}
