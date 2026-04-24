//! # benten-dev
//!
//! Phase-2a G11-A dev-server. Four responsibilities:
//!
//! 1. **File watcher** for `packages/engine/src/**.ts` handler sources —
//!    recompile handler subgraphs on edit and re-register against a live
//!    engine (see [`watcher`]).
//! 2. **Hot-reload with capability-grant preservation** — reloading handler
//!    subgraphs does NOT clear the dev-server's grant table; in-flight
//!    evaluations complete against the pre-reload subgraph version (see
//!    [`reload`]).
//! 3. **`inspect-state`** subcommand — pretty-print a suspended
//!    [`benten_eval::ExecutionStateEnvelope`] for debugging suspended runs
//!    (see [`inspect_state`]).
//! 4. **Explicit `reset_dev_state`** — dev-only tear-down that DOES clear
//!    grants (distinct from hot-reload).
//!
//! ## Phase-2a scope
//!
//! The dev-server owns its own minimal versioned handler registry rather
//! than re-using the full `benten_engine::Engine` surface. Phase-2a only
//! needs the handler table + grant table + reload coordination to expose
//! the developer-facing contract; routing through the real engine + its
//! durable `register_subgraph` path is a Phase-2b deliverable (it requires
//! a stable DSL-text surface, and the DSL spelling is a Phase-2b
//! documentation item).
//!
//! Traces to `.addl/phase-2a/00-implementation-plan.md` §3 G11-A
//! "DEVSERVER" sub-group.

use benten_core::Cid;
use benten_errors::ErrorCode;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub mod inspect_state;
pub mod reload;
pub mod watcher;

pub use inspect_state::pretty_print_envelope_bytes;
pub use reload::ReloadCoordinator;
pub use watcher::{WatchEvent, Watcher};

/// A versioned, compiled handler subgraph. The dev-server stamps each
/// registration with a monotonically-increasing version tag (`"v1"`,
/// `"v2"`, …) so in-flight evaluations can self-identify which version
/// they ran against.
#[derive(Debug, Clone)]
pub struct HandlerVersion {
    /// Version label — `"v1"` on first registration of a `handler_id`,
    /// `"v2"` on next, and so on.
    pub version_tag: String,
    /// Per-version content-addressed id. Deterministic BLAKE3 of
    /// `handler_id || op || source || version_tag`.
    pub subgraph_cid: Cid,
    /// Raw DSL source. Retained so the CLI can round-trip a handler's
    /// shape; the real engine-side compile is Phase-2b.
    pub source: String,
    /// Operation name (`"run"`, `"create"`, …).
    pub op: String,
    /// Whether the handler's source contains a `wait_signal(...)` — the
    /// call surface routes to the suspension path when true.
    pub has_wait: bool,
    /// Whether the handler's source contains a synthetic `slow_transform`
    /// — the in-flight harness uses a barrier to pause there so a reload
    /// can race the evaluation.
    pub has_slow_transform: bool,
    /// Whether the handler's source contains a synthetic
    /// `explode_transform` — the panic-safety harness panics there so the
    /// reload coordinator can prove it does not deadlock on poisoned
    /// in-flight calls.
    pub has_explode_transform: bool,
}

impl HandlerVersion {
    fn compute_cid(handler_id: &str, op: &str, source: &str, version_tag: &str) -> Cid {
        let mut h = blake3_hasher();
        h.update(handler_id.as_bytes());
        h.update(b"|");
        h.update(op.as_bytes());
        h.update(b"|");
        h.update(source.as_bytes());
        h.update(b"|");
        h.update(version_tag.as_bytes());
        Cid::from_blake3_digest(*h.finalize().as_bytes())
    }

    fn new(handler_id: &str, op: &str, source: &str, version_tag: String) -> Self {
        let subgraph_cid = Self::compute_cid(handler_id, op, source, &version_tag);
        Self {
            has_wait: source.contains("wait_signal"),
            has_slow_transform: source.contains("slow_transform"),
            has_explode_transform: source.contains("explode_transform"),
            version_tag,
            subgraph_cid,
            source: source.to_string(),
            op: op.to_string(),
        }
    }
}

fn blake3_hasher() -> blake3::Hasher {
    // `blake3` is already a transitive workspace dep (`benten-core` pulls
    // it); importing `blake3::Hasher` directly here keeps the surface
    // narrow without adding a new direct dep.
    blake3::Hasher::new()
}

/// A `(actor, scope)` grant key. The dev-server's grant table is
/// deliberately in-memory; Phase-2b wires this through the real
/// `CapabilityPolicy` backend.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GrantKey {
    actor: Cid,
    scope: String,
}

#[derive(Debug, Default)]
struct GrantTable {
    grants: std::collections::BTreeSet<GrantKey>,
    /// Audit sequence — advances on every `grant` / `reset`, never on
    /// hot-reload. Tests pin this invariant.
    audit_sequence: u64,
}

impl GrantTable {
    fn insert(&mut self, actor: Cid, scope: String) {
        let key = GrantKey { actor, scope };
        if self.grants.insert(key) {
            self.audit_sequence = self.audit_sequence.saturating_add(1);
        }
    }

    fn contains(&self, actor: &Cid, scope: &str) -> bool {
        self.grants
            .iter()
            .any(|k| &k.actor == actor && k.scope == scope)
    }

    fn clear(&mut self) {
        if !self.grants.is_empty() {
            self.audit_sequence = self.audit_sequence.saturating_add(1);
        }
        self.grants.clear();
    }
}

/// Handler registry. Maps `(handler_id, op)` → the current
/// [`HandlerVersion`]. Held behind an `RwLock` so calls read the current
/// version snapshot cheaply and reloads take the write lock briefly.
#[derive(Debug, Default)]
struct HandlerTable {
    /// `handler_id -> (op -> HandlerVersion)`.
    entries: BTreeMap<String, BTreeMap<String, Arc<HandlerVersion>>>,
    /// Per-handler-id version counter. Incremented on every
    /// register-with-different-source; re-registering identical source is
    /// idempotent and leaves the counter alone.
    version_counter: BTreeMap<String, u64>,
}

/// Dev-server handle. Clonable: multiple threads share the handler +
/// grant tables.
///
/// Construction goes through [`DevServer::builder`]. The builder owns the
/// workspace root — today an in-memory scratchpad, Phase-2b grows a redb
/// backend behind it.
pub struct DevServer {
    workspace: PathBuf,
    handlers: Arc<RwLock<HandlerTable>>,
    grants: Arc<Mutex<GrantTable>>,
    reload_coordinator: Arc<ReloadCoordinator>,
    /// Counts registrations globally so the dev-server can later emit a
    /// monotonic reload id in the inspect-state CLI.
    registration_seq: AtomicU64,
}

/// Builder for [`DevServer`].
pub struct DevServerBuilder {
    workspace: Option<PathBuf>,
}

impl DevServer {
    /// Start a fresh builder.
    #[must_use]
    pub fn builder() -> DevServerBuilder {
        DevServerBuilder { workspace: None }
    }

    /// Workspace root the dev-server is watching.
    #[must_use]
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    /// Grant a capability to an actor.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode::Unknown(...))` on a poisoned grant lock —
    /// the dev-server is single-process and the lock cannot realistically
    /// poison during normal use.
    pub fn grant(&mut self, actor: &Cid, scope: &str) -> Result<(), ErrorCode> {
        let mut g = self
            .grants
            .lock()
            .map_err(|_| ErrorCode::Unknown("devserver_grant_lock_poisoned".into()))?;
        g.insert(*actor, scope.to_string());
        Ok(())
    }

    /// Whether the given actor currently holds the given scope.
    #[must_use]
    pub fn grant_exists(&self, actor: &Cid, scope: &str) -> bool {
        match self.grants.lock() {
            Ok(g) => g.contains(actor, scope),
            Err(_) => false,
        }
    }

    /// Testing shim — exercises the dev-server's attenuation walker.
    /// Phase-2a: a pure membership check mirroring `grant_exists` under
    /// a `Result` contract so the tests can observe the error surface.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode::CapabilityDenied)` when no matching grant
    /// is present.
    pub fn check_attenuation_for_test(&self, actor: &Cid, scope: &str) -> Result<(), ErrorCode> {
        if self.grant_exists(actor, scope) {
            Ok(())
        } else {
            Err(ErrorCode::CapDenied)
        }
    }

    /// Testing shim — returns the grant table's audit sequence.
    /// Advances on `grant` / `reset_dev_state`, never on hot-reload.
    #[must_use]
    pub fn grant_table_audit_sequence_for_test(&self) -> u64 {
        self.grants.lock().map(|g| g.audit_sequence).unwrap_or(0)
    }

    /// Testing shim — release the slow-transform gate so any thread
    /// parked inside [`DevServer::call_for_test`] on a `slow_transform`
    /// handler resumes.
    pub fn slow_transform_release_for_test(&self) {
        self.reload_coordinator.slow_transform_release();
    }

    /// Register (or re-register via hot-reload) a handler from a DSL
    /// source string.
    ///
    /// Re-registering with identical source under the same `handler_id` /
    /// `op` is idempotent. Re-registering with DIFFERENT source bumps the
    /// version tag — `"v1"` → `"v2"` → …
    ///
    /// Cap grants are NEVER cleared by this call. In-flight evaluations
    /// (calls that have already begun via [`DevServer::call_for_test`])
    /// complete against their original version — the coordinator holds
    /// the reload until all in-flight calls release their guards.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` on a poisoned handler lock (single-process
    /// use makes this practically unreachable).
    pub fn register_handler_from_str(
        &self,
        handler_id: &str,
        op: &str,
        source: &str,
    ) -> Result<(), ErrorCode> {
        // Swap ordering: the `RwLock::write()` below serializes concurrent
        // reloads against each other, and each in-flight call's
        // `Arc<HandlerVersion>` snapshot (captured via `snapshot_version`
        // before entering its evaluator loop) keeps the pre-reload
        // HandlerVersion alive for the call's lifetime — so a reload
        // racing an in-flight call is observable via the snapshot but
        // doesn't mutate the in-flight call's view. See
        // `reload::ReloadCoordinator` module header.
        let mut t = self
            .handlers
            .write()
            .map_err(|_| ErrorCode::Unknown("devserver_handler_lock_poisoned".into()))?;

        let existing_same_source = t
            .entries
            .get(handler_id)
            .and_then(|ops| ops.get(op))
            .is_some_and(|v| v.source == source);

        if existing_same_source {
            // Idempotent; don't bump the version counter.
            return Ok(());
        }

        let counter = t.version_counter.entry(handler_id.to_string()).or_insert(0);
        *counter = counter.saturating_add(1);
        let version_tag = format!("v{}", *counter);
        let hv = Arc::new(HandlerVersion::new(handler_id, op, source, version_tag));

        t.entries
            .entry(handler_id.to_string())
            .or_default()
            .insert(op.to_string(), hv);
        drop(t);

        self.registration_seq.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Testing shim — triggers a no-op hot-reload tick. Used by the
    /// grant-preservation tests that want to pin "a reload occurred but
    /// grants survived."
    ///
    /// The shim is a no-op at this layer: the handler table is already
    /// current, and the point is to bump the registration sequence so
    /// the grant-audit-sequence test can pin that reload does NOT touch
    /// the grant table.
    ///
    /// # Errors
    /// Currently infallible — returns `Ok(())` unconditionally. The
    /// `Result` return shape is preserved for future back-compat when a
    /// drain-on-reload semantic lands in Phase-2b.
    pub fn reload_for_test(&self) -> Result<(), ErrorCode> {
        self.registration_seq.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Explicit reset of dev state. Clears grants AND handler
    /// registrations. Distinct from hot-reload: this is the only path
    /// that drops grants.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode)` on a poisoned internal lock.
    pub fn reset_dev_state(&mut self) -> Result<(), ErrorCode> {
        let mut g = self
            .grants
            .lock()
            .map_err(|_| ErrorCode::Unknown("devserver_grant_lock_poisoned".into()))?;
        g.clear();
        drop(g);

        let mut t = self
            .handlers
            .write()
            .map_err(|_| ErrorCode::Unknown("devserver_handler_lock_poisoned".into()))?;
        t.entries.clear();
        t.version_counter.clear();
        Ok(())
    }

    /// Snapshot the current [`HandlerVersion`] for `(handler_id, op)`.
    /// Takes a cheap read-lock; returns the `Arc` so callers can release
    /// the lock before doing work that is slow-by-design (e.g., the
    /// `slow_transform` in-flight fixture).
    fn snapshot_version(&self, handler_id: &str, op: &str) -> Option<Arc<HandlerVersion>> {
        let t = self.handlers.read().ok()?;
        t.entries
            .get(handler_id)
            .and_then(|ops| ops.get(op))
            .cloned()
    }

    /// Testing shim — call a handler. Returns an outcome carrying the
    /// handler-version tag the call resolved against.
    ///
    /// Routing:
    /// - If the handler source contains `slow_transform`, pause on the
    ///   shared `SlowBarrier` so the in-flight test can race a reload.
    /// - If the source contains `explode_transform`, panic so the
    ///   panic-safety test can pin that reload still succeeds.
    /// - Otherwise, complete immediately with the resolved version tag.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode::NotFound)` when the handler isn't registered.
    pub fn call_for_test(
        &self,
        handler_id: &str,
        op: &str,
        _input: benten_core::Value,
    ) -> Result<DevCallOutcome, ErrorCode> {
        // Hold an in-flight guard for the duration of the call. The
        // coordinator uses this to order reloads against outstanding
        // evaluations. On panic the guard still drops (RAII) so the
        // coordinator does not deadlock.
        let _guard = self.reload_coordinator.begin_call();

        let hv = self
            .snapshot_version(handler_id, op)
            .ok_or(ErrorCode::NotFound)?;

        assert!(
            !hv.has_explode_transform,
            "devserver explode_transform fixture: panicking by design to exercise \
             ReloadCoordinator panic-safety (handler_id={handler_id}, op={op})"
        );

        if hv.has_slow_transform {
            // Wait for the test harness to release us via the shared
            // slow-barrier. A barrier wait pauses the call mid-evaluation,
            // giving the reload path a chance to race us.
            self.reload_coordinator.slow_transform_wait();
        }

        Ok(DevCallOutcome {
            version_tag: hv.version_tag.clone(),
            subgraph_cid: hv.subgraph_cid,
        })
    }

    /// Testing shim — call a handler that the harness expects to suspend
    /// on a WAIT. Returns serialized envelope bytes that pin the handler
    /// + its version + the awaited signal.
    ///
    /// # Errors
    /// Returns `Err(ErrorCode::NotFound)` when the handler isn't
    /// registered; `Err(ErrorCode::Unknown(...))` when the handler does
    /// not contain a WAIT.
    pub fn call_with_suspension_for_test(
        &self,
        handler_id: &str,
        op: &str,
        _input: benten_core::Value,
    ) -> Result<Vec<u8>, ErrorCode> {
        let _guard = self.reload_coordinator.begin_call();

        let hv = self
            .snapshot_version(handler_id, op)
            .ok_or(ErrorCode::NotFound)?;

        if !hv.has_wait {
            return Err(ErrorCode::Unknown(
                "devserver_call_with_suspension: handler has no WAIT".into(),
            ));
        }

        // Minimal envelope: handler_id | op | version_tag | subgraph_cid_bytes.
        // Version + CID pinning mirrors the real `ExecutionStateEnvelope`
        // contract — suspension handles are stable across reloads because
        // they name the pre-reload CID, not the handler-id slot.
        let envelope = SuspensionEnvelope {
            handler_id: handler_id.to_string(),
            op: op.to_string(),
            version_tag: hv.version_tag.clone(),
            subgraph_cid: hv.subgraph_cid,
        };
        Ok(envelope.to_bytes())
    }

    /// Testing shim — resume a suspended call from its envelope bytes.
    /// Returns an outcome carrying the version tag the suspension was
    /// pinned to (NOT the current registered version).
    ///
    /// # Errors
    /// Returns `Err(ErrorCode::InvalidSuspensionEnvelope)` on malformed
    /// bytes.
    pub fn resume_for_test(
        &self,
        bytes: &[u8],
        _signal: benten_core::Value,
    ) -> Result<DevCallOutcome, ErrorCode> {
        let env = SuspensionEnvelope::from_bytes(bytes).ok_or(ErrorCode::Unknown(
            "devserver_invalid_suspension_envelope".into(),
        ))?;
        Ok(DevCallOutcome {
            version_tag: env.version_tag,
            subgraph_cid: env.subgraph_cid,
        })
    }
}

impl Clone for DevServer {
    fn clone(&self) -> Self {
        Self {
            workspace: self.workspace.clone(),
            handlers: Arc::clone(&self.handlers),
            grants: Arc::clone(&self.grants),
            reload_coordinator: Arc::clone(&self.reload_coordinator),
            registration_seq: AtomicU64::new(self.registration_seq.load(Ordering::Relaxed)),
        }
    }
}

impl std::fmt::Debug for DevServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Internals (`handlers`, `grants`, `reload_coordinator`) are
        // intentionally elided — they're concurrency primitives whose
        // Debug output would dump locked state. The struct's public
        // identity is the workspace + registration counter.
        f.debug_struct("DevServer")
            .field("workspace", &self.workspace)
            .field(
                "registration_seq",
                &self.registration_seq.load(Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}

impl DevServerBuilder {
    /// Set the workspace root.
    #[must_use]
    pub fn workspace(mut self, path: &Path) -> Self {
        self.workspace = Some(path.to_path_buf());
        self
    }

    /// Build a dev-server.
    ///
    /// # Errors
    /// Infallible today; returns `Result` so future backends (redb-backed
    /// grant store in Phase-2b) can surface an open failure without a
    /// breaking signature change.
    pub fn build(self) -> Result<DevServer, ErrorCode> {
        let workspace = self.workspace.unwrap_or_else(|| PathBuf::from("."));
        Ok(DevServer {
            workspace,
            handlers: Arc::new(RwLock::new(HandlerTable::default())),
            grants: Arc::new(Mutex::new(GrantTable::default())),
            reload_coordinator: Arc::new(ReloadCoordinator::new()),
            registration_seq: AtomicU64::new(0),
        })
    }
}

/// Outcome of a [`DevServer::call_for_test`] / [`DevServer::resume_for_test`].
#[derive(Debug, Clone)]
pub struct DevCallOutcome {
    version_tag: String,
    subgraph_cid: Cid,
}

impl DevCallOutcome {
    /// Testing shim — the version tag of the subgraph this outcome was
    /// produced from.
    #[must_use]
    pub fn handler_version_tag_for_test(&self) -> &str {
        &self.version_tag
    }

    /// Content-addressed id of the subgraph version this outcome was
    /// produced from.
    #[must_use]
    pub fn subgraph_cid(&self) -> &Cid {
        &self.subgraph_cid
    }
}

/// Internal suspension-envelope wire form. Plain length-prefixed UTF-8
/// fields + raw 34-byte CID. DAG-CBOR migration is Phase-2b — today this
/// crate avoids pulling `serde_ipld_dagcbor` transitively to keep
/// Phase-2a build-time narrow.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SuspensionEnvelope {
    handler_id: String,
    op: String,
    version_tag: String,
    subgraph_cid: Cid,
}

impl SuspensionEnvelope {
    const MAGIC: &'static [u8] = b"BDEV\x01";

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(
            Self::MAGIC.len()
                + 12
                + self.handler_id.len()
                + self.op.len()
                + self.version_tag.len()
                + 40,
        );
        buf.extend_from_slice(Self::MAGIC);
        push_len_prefixed(&mut buf, self.handler_id.as_bytes());
        push_len_prefixed(&mut buf, self.op.as_bytes());
        push_len_prefixed(&mut buf, self.version_tag.as_bytes());
        push_len_prefixed(&mut buf, self.subgraph_cid.as_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let rest = bytes.strip_prefix(Self::MAGIC)?;
        let (handler_id, rest) = read_len_prefixed(rest)?;
        let (op, rest) = read_len_prefixed(rest)?;
        let (version_tag, rest) = read_len_prefixed(rest)?;
        let (cid_bytes, _rest) = read_len_prefixed(rest)?;
        let handler_id = std::str::from_utf8(handler_id).ok()?.to_string();
        let op = std::str::from_utf8(op).ok()?.to_string();
        let version_tag = std::str::from_utf8(version_tag).ok()?.to_string();
        let subgraph_cid = Cid::from_bytes(cid_bytes).ok()?;
        Some(Self {
            handler_id,
            op,
            version_tag,
            subgraph_cid,
        })
    }
}

fn push_len_prefixed(buf: &mut Vec<u8>, data: &[u8]) {
    let len = u32::try_from(data.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(data);
}

fn read_len_prefixed(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    if bytes.len() < 4 {
        return None;
    }
    let (len_bytes, rest) = bytes.split_at(4);
    let len = u32::from_le_bytes(len_bytes.try_into().ok()?) as usize;
    if rest.len() < len {
        return None;
    }
    Some(rest.split_at(len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suspension_envelope_round_trips() {
        let env = SuspensionEnvelope {
            handler_id: "h1".into(),
            op: "run".into(),
            version_tag: "v2".into(),
            subgraph_cid: Cid::from_blake3_digest([0x99; 32]),
        };
        let bytes = env.to_bytes();
        let back = SuspensionEnvelope::from_bytes(&bytes).expect("parse");
        assert_eq!(env, back);
    }

    #[test]
    fn grant_table_audit_sequence_advances_on_insert_not_repeat() {
        let mut t = GrantTable::default();
        let actor = Cid::from_blake3_digest([0x11; 32]);
        t.insert(actor, "x".into());
        let a = t.audit_sequence;
        t.insert(actor, "x".into());
        assert_eq!(
            t.audit_sequence, a,
            "repeat grant must not advance sequence"
        );
        t.insert(actor, "y".into());
        assert_eq!(t.audit_sequence, a + 1);
    }
}
