//! SANDBOX host-function declarations + cap-recheck cadence (Phase 2b G7-A).
//!
//! D7 + D18 + D25 implementation surface:
//!   - **D7-RESOLVED hybrid**: cap enforcement runs at SANDBOX init (snapshot
//!     intersection of manifest claims ∩ live grant) AND per-invocation
//!     (cadence per D18).
//!   - **D18-RESOLVED hybrid**: each host-fn declares
//!     `cap_recheck = "per_call" | "per_boundary"` in `host-functions.toml`.
//!     Default is `"per_call"` (fail-secure). D1 defaults `time/log =
//!     per_boundary`, `kv:read = per_call`.
//!   - **D25-RESOLVED**: host-fn output bytes are counted at the codegen-
//!     emitted TRAMPOLINE boundary (centralized accounting; one place to
//!     audit). The host-fn body never touches the [`CountedSink`] counter
//!     directly.
//!
//! The `host-functions.toml` workspace-root file is the source-of-truth.
//! Its dev-time `[host_fn.<name>]` tables compile into the
//! [`default_host_fns`] table at construction time. Phase-2b G7-A ships a
//! hand-mirrored static table (no separate `build.rs`); the drift
//! detector tests parse the TOML at runtime and assert byte-for-byte
//! match against the static.

use crate::sandbox::counted_sink::CountedSink;
use benten_errors::ErrorCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// D18 cap-recheck cadence per host-fn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapRecheckPolicy {
    /// Live cap-string check before EVERY host-fn invocation. Default
    /// (fail-secure) — auditors reading the manifest know that anything
    /// not explicitly relaxed gets the tightest TOCTOU window.
    PerCall,
    /// Snapshot taken at SANDBOX entry; revocations during the call are
    /// not visible until the next primitive boundary. Reserved for cheap,
    /// idempotent, output-bounded host-fns where the per-call overhead
    /// would dominate (D1 defaults: `time` + `log`).
    PerBoundary,
}

impl Default for CapRecheckPolicy {
    fn default() -> Self {
        // D18 fail-secure: undeclared `cap_recheck` field defaults here.
        CapRecheckPolicy::PerCall
    }
}

/// Behavior classification for a host-fn. Determines the trampoline
/// dispatch path + per-fn-specific budgets (e.g. `log` per-call byte cap,
/// `kv:read` per-call read budget).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostFnBehavior {
    /// `time` host-fn — returns monotonic time coarsened to N ms.
    /// D1 + sec-pre-r1-06 §2.1 + ESC-16 — closes timezone leak +
    /// fingerprinting side channel.
    TimeMonotonicCoarsened {
        /// Coarsening granularity in milliseconds (D1 default 100).
        coarsening_ms: u64,
    },
    /// `log` host-fn — writes a string to the engine log sink.
    /// D1 + sec-pre-r1-06 §2.2 — per-call byte-volume cap.
    LogSink {
        /// Per-call byte cap (D1 default 65536 = 64 KiB).
        per_call_byte_cap: u64,
    },
    /// `kv:read` host-fn — reads a value by CID from the KV backend.
    /// D1 + sec-pre-r1-06 §2.4 — per-call cap-recheck + read budget.
    KvRead {
        /// Per-primitive-call read budget (D1 default 1000).
        per_call_read_cap: u64,
    },
}

/// Declarative spec for a single host-fn entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostFnSpec {
    /// Stable name, matches the `[host_fn.<name>]` TOML key (and the
    /// import-name a SANDBOX module uses to call it).
    pub name: String,
    /// Cap-string the dispatching grant must hold (e.g.
    /// `"host:compute:time"`).
    pub requires: String,
    /// Cap-recheck cadence (D18). Default [`CapRecheckPolicy::PerCall`]
    /// (fail-secure).
    #[serde(default)]
    pub cap_recheck: CapRecheckPolicy,
    /// Behavior classification (drives trampoline dispatch + per-fn
    /// budgets).
    pub behavior: HostFnBehavior,
    /// D25 — when `false` (default), the trampoline counts output bytes
    /// against the per-call [`CountedSink`] budget. Phase-2b D1 surface
    /// (time/log/kv:read) all set this to `false`; no host-fn ships
    /// with `bypass_output_budget = true`.
    #[serde(default)]
    pub bypass_output_budget: bool,
    /// D19 — Phase-3 iroh forward-compat. `false` in 2b for every entry
    /// (no async host-fn ships in 2b). Reserved cap `host:async` is
    /// declared but not wired.
    #[serde(default)]
    pub requires_async: bool,
    /// One-line description (dev-time only; not part of canonical bytes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl HostFnSpec {
    /// Validate the `requires` cap-string is well-formed (3-segment
    /// `prefix:domain:action`). The cap-string itself is the authoritative
    /// key passed to `PrimitiveHost::check_capability` at runtime — there
    /// is no intermediate typed projection in 2b. Phase-3 cap-namespace
    /// work may introduce one; this method is the validation entry-point
    /// today.
    ///
    /// # Errors
    /// Returns [`ErrorCode::CapDenied`] / [`ErrorCode::CapScopeLoneStarRejected`]
    /// when the `requires` cap-string is malformed.
    pub fn validate_requires(&self) -> Result<(), ErrorCode> {
        benten_errors::parse_cap_string(&self.requires).map(|_| ())
    }
}

/// Codegen-emitted host-fn table. Mirrors the dev-time
/// `host-functions.toml` `[host_fn.*]` tables.
///
/// Adding a new host-fn: edit `host-functions.toml` AND append an entry
/// to [`default_host_fns`] below. The drift detector test
/// (`sandbox_host_fn_cap_recheck_codegen_drift_total`) re-parses the
/// TOML at runtime and asserts byte-for-byte match.
const HOST_FN_NAMES: &[&str] = &["time", "log", "kv:read"];

/// Names exposed for drift / coverage tests.
#[must_use]
pub fn host_fn_names() -> &'static [&'static str] {
    HOST_FN_NAMES
}

/// Build the codegen-default host-fn table. D1-RESOLVED initial surface:
/// `time` (per_boundary, monotonic-100ms) + `log` (per_boundary, 64KiB
/// per-call cap) + `kv:read` (per_call, 1000 reads/call).
///
/// `random` is intentionally absent — D1 + sec-pre-r1-06 §2.3 defers it
/// to Phase 2c until the workspace-wide CSPRNG decision lands.
#[must_use]
pub fn default_host_fns() -> BTreeMap<String, HostFnSpec> {
    let mut table: BTreeMap<String, HostFnSpec> = BTreeMap::new();

    table.insert(
        "time".to_string(),
        HostFnSpec {
            name: "time".to_string(),
            requires: "host:compute:time".to_string(),
            cap_recheck: CapRecheckPolicy::PerBoundary,
            behavior: HostFnBehavior::TimeMonotonicCoarsened { coarsening_ms: 100 },
            bypass_output_budget: false,
            requires_async: false,
            description: Some(
                "Returns monotonic time, coarsened to 100ms granularity.".to_string(),
            ),
        },
    );

    table.insert(
        "log".to_string(),
        HostFnSpec {
            name: "log".to_string(),
            requires: "host:compute:log".to_string(),
            cap_recheck: CapRecheckPolicy::PerBoundary,
            behavior: HostFnBehavior::LogSink {
                per_call_byte_cap: 65536,
            },
            bypass_output_budget: false,
            requires_async: false,
            description: Some(
                "Writes a string from the SANDBOX module to the engine log sink.".to_string(),
            ),
        },
    );

    table.insert(
        "kv:read".to_string(),
        HostFnSpec {
            name: "kv:read".to_string(),
            requires: "host:compute:kv:read".to_string(),
            cap_recheck: CapRecheckPolicy::PerCall,
            behavior: HostFnBehavior::KvRead {
                per_call_read_cap: 1000,
            },
            bypass_output_budget: false,
            requires_async: false,
            description: Some("Reads a value by CID from the engine KV backend.".to_string()),
        },
    );

    table
}

/// Reserved cap-string for D19 calibrated allow-async path.
/// In Phase 2b: declared, not used (no async host-fn ships). Phase 3
/// iroh `kv:read` flips `requires_async = true` and acquires this cap.
pub const RESERVED_HOST_ASYNC_CAP: &str = "host:async";

/// Init-snapshot intersection of a manifest's caps against a live
/// dispatching grant.
///
/// Per D7 hybrid + sec-pre-r1-02 Option-D: at SANDBOX entry, the engine
/// snapshots the dispatching grant's cap-set AND intersects with the
/// manifest's declared caps; the resulting allowlist is the per-call
/// host-fn linkability surface. Subsequent per-invocation re-checks (D18
/// per_call) consult the LIVE grant; per_boundary host-fns consult the
/// SNAPSHOT.
#[derive(Debug, Clone)]
pub struct CapAllowlist {
    /// Cap-strings the dispatching grant holds AND the manifest claims.
    /// Sorted-canonical (mirrors `CapBundle::caps` discipline) so two
    /// snapshots from the same grant + manifest are bit-equal.
    pub allowed: Vec<String>,
}

impl CapAllowlist {
    /// Compute the intersection of `manifest_caps` ∩ `grant_caps`.
    /// Both inputs MUST be sorted-canonical; the output is sorted-canonical.
    #[must_use]
    pub fn intersect(manifest_caps: &[String], grant_caps: &[String]) -> Self {
        let manifest_set: std::collections::BTreeSet<&String> = manifest_caps.iter().collect();
        let mut allowed: Vec<String> = grant_caps
            .iter()
            .filter(|c| manifest_set.contains(c))
            .cloned()
            .collect();
        allowed.sort();
        allowed.dedup();
        Self { allowed }
    }

    /// True iff the cap-string is in the allowlist.
    #[must_use]
    pub fn contains(&self, cap: &str) -> bool {
        self.allowed.iter().any(|c| c == cap)
    }

    /// True iff every cap declared by the host-fn entries is in the
    /// allowlist (init-snapshot validation; missing entries cause the
    /// SANDBOX call to fail-loud at link-time before module execution).
    #[must_use]
    pub fn satisfies_all(&self, required: &[&str]) -> bool {
        required.iter().all(|r| self.contains(r))
    }
}

/// Per-invocation context the trampoline threads through every host-fn
/// call. Exposes the [`CountedSink`] (D17 PRIMARY + D25 trampoline-count)
/// + the allowlist (init snapshot for `per_boundary` host-fns) + the
/// callback for live cap re-check (consulted by `per_call` host-fns).
///
/// This is intentionally a thin typed wrapper — `Sandbox` owns the
/// concrete state; the trampoline borrows this view per-invocation.
pub struct HostFnContext<'a> {
    /// Per-call streaming-byte accumulator (D17 PRIMARY). Trampoline
    /// increments via [`CountedSink::write`] AFTER the host-fn body
    /// returns its output bytes.
    pub sink: &'a mut CountedSink,
    /// Init-snapshot allowlist (consulted by `PerBoundary` host-fns).
    pub allowlist: &'a CapAllowlist,
    /// Per-call read budget remaining (consumed by `kv:read`).
    pub kv_reads_remaining: &'a mut u64,
    /// Per-call log byte-volume remaining (consumed by `log`).
    pub log_bytes_remaining: &'a mut u64,
}

/// Trampoline outcome — the host-fn body returns one of these three
/// shapes; the trampoline post-processes (count bytes, recheck caps).
#[derive(Debug)]
pub enum HostFnReturn {
    /// Successful return with output bytes. Trampoline counts bytes
    /// against [`HostFnContext::sink`] BEFORE handing them back to the
    /// guest (D25 centralized accounting).
    Bytes(Vec<u8>),
    /// Successful return with no output bytes (e.g. `log` consumes input
    /// and produces nothing).
    Empty,
    /// Error path. Trampoline maps to the typed error code at the
    /// host-fn ABI boundary (NOT a wasmtime trap — sec-r1 D7).
    Error(ErrorCode),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_host_fns_match_d1_resolved_surface() {
        let table = default_host_fns();
        assert_eq!(table.len(), 3, "D1 surface = time + log + kv:read");
        assert!(table.contains_key("time"));
        assert!(table.contains_key("log"));
        assert!(table.contains_key("kv:read"));
        assert!(
            !table.contains_key("random"),
            "D1 + sec-pre-r1-06 §2.3 — random deferred to Phase 2c"
        );
    }

    #[test]
    fn d18_default_cap_recheck_is_per_call_fail_secure() {
        // wsa D18 fail-secure regression — undeclared field defaults to
        // PerCall (the safer bound).
        assert_eq!(CapRecheckPolicy::default(), CapRecheckPolicy::PerCall);
    }

    #[test]
    fn d18_d1_defaults_match_resolution() {
        let table = default_host_fns();
        assert_eq!(
            table["time"].cap_recheck,
            CapRecheckPolicy::PerBoundary,
            "D1 — time is cheap + idempotent → per_boundary"
        );
        assert_eq!(
            table["log"].cap_recheck,
            CapRecheckPolicy::PerBoundary,
            "D1 — log is output-bounded → per_boundary"
        );
        assert_eq!(
            table["kv:read"].cap_recheck,
            CapRecheckPolicy::PerCall,
            "D1 — kv:read is sensitive → per_call"
        );
    }

    #[test]
    fn d25_no_d1_host_fn_bypasses_output_budget() {
        for (name, spec) in default_host_fns() {
            assert!(
                !spec.bypass_output_budget,
                "D25 — D1 surface entry {name} must NOT bypass output budget"
            );
        }
    }

    #[test]
    fn d19_no_d1_host_fn_requires_async() {
        for (name, spec) in default_host_fns() {
            assert!(
                !spec.requires_async,
                "D19 — D1 surface entry {name} must NOT require host:async"
            );
        }
    }

    #[test]
    fn cap_allowlist_intersection_drops_uncommon() {
        let manifest = vec!["host:compute:time".into(), "host:compute:kv:read".into()];
        let grant = vec!["host:compute:time".into(), "host:compute:log".into()];
        let allow = CapAllowlist::intersect(&manifest, &grant);
        assert_eq!(allow.allowed, vec!["host:compute:time".to_string()]);
    }
}
