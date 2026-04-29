//! Phase 2b Wave-8f — napi `DevServer` bridge.
//!
//! Wraps `benten_dev::DevServer` for JS consumers. Exposes:
//!
//! - `start()` — open the underlying engine + initialise the dev-server's
//!   handler-table + grant-table state. Idempotent.
//! - `stop()` — drop the embedded `benten_dev::DevServer`. After `stop()`,
//!   subsequent calls return `E_DEVSERVER_STOPPED`.
//! - `registerHandlerFromDsl(handlerId, op, source)` — first-registration
//!   path. Returns the registered handler id on success; surfaces typed
//!   `Diagnostic` data on bad-DSL input.
//! - `replaceHandlerFromDsl(handlerId, op, source)` — explicit replace alias
//!   for the JS surface. Routes through `Engine::register_subgraph_replace`
//!   underneath. Returns the live handler id.
//! - `subscribeToReloadEvents()` — returns a `ReloadSubscriberJs` whose
//!   `drain()` reports per-event JSON of `{ handlerId, op, versionTag,
//!   newCid, previousCid }`.
//!
//! ## No `engine()` accessor on this surface
//!
//! The DevServer does NOT expose its embedded `Engine` to JS. JS callers
//! that want to drive `engine.call(...)` against the same workspace must
//! open a separate `Engine.open(<workspace>/.benten-dev.redb)` against
//! the same redb file path the dev-server uses. Be aware that redb takes
//! an exclusive process-wide file lock — opening a second handle while
//! the dev-server is started will fail with a backend lock-conflict
//! error. The recommended pattern is to `dev.stop()` before opening the
//! standalone `Engine`, or to drive call dispatch via DevServer's own
//! `registerHandler` / `replaceHandler` surface (which internally uses
//! the dev-server's `Engine`). A future Phase-3 ergonomic enhancement
//! will adapt the napi `Engine` class to be constructible from an
//! existing `Arc<Engine>` so the DevServer can hand out a non-owning
//! handle without lock-conflict surface — out of scope for Wave-8f.
//!
//! ## Why this lives in its own file
//!
//! Diff-reviewable: the dev-server bridge is a self-contained surface
//! with no overlap to the existing `napi_surface::Engine` impl block.

#![cfg(feature = "napi-export")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use benten_dev::{DevServer as InnerDevServer, ReloadEvent, ReloadSubscriber};
use benten_dsl_compiler::CompileError;
use napi::bindgen_prelude::*;
use napi_derive::napi;

// ---------------------------------------------------------------------------
// DevServer napi class
// ---------------------------------------------------------------------------

/// Phase 2b Wave-8f: napi `DevServer` handle. Constructed by the JS-side
/// `BentenDevServer` wrapper in `@benten/engine-devserver`.
///
/// One DevServer corresponds to one workspace + one embedded `Engine`
/// at `<workspace>/.benten-dev.redb`. The DevServer's grant-table is
/// preserved across hot-reload (the load-bearing G11-A / G12-B
/// property); the grant table is sibling-state to the handler-table
/// and is NEVER cleared by the registration path.
#[napi]
pub struct DevServer {
    /// The wrapped Rust DevServer. `None` when the JS caller has invoked
    /// `stop()`. Held under a Mutex so the napi boundary's `&self`
    /// methods can mutate (start / stop) without a `&mut self` napi
    /// signature (napi-rs's `#[napi]` impl methods are `&self`).
    inner: Mutex<Option<InnerDevServer>>,
    /// Cached workspace path so `start()` can rebuild the inner DevServer
    /// after a stop without re-passing the path.
    workspace: PathBuf,
}

#[napi]
impl DevServer {
    /// Construct a DevServer rooted at `workspaceRoot`. The constructor
    /// does not open the engine — call `.start()` to do that.
    #[napi(constructor)]
    pub fn new(workspace_root: String) -> Self {
        Self {
            inner: Mutex::new(None),
            workspace: PathBuf::from(workspace_root),
        }
    }

    /// Open the embedded engine + initialise the dev-server. Idempotent —
    /// calling `start()` on an already-started DevServer is a no-op.
    #[napi]
    pub fn start(&self) -> napi::Result<()> {
        let mut g = self.inner.lock().map_err(poisoned)?;
        if g.is_some() {
            return Ok(());
        }
        let dev = InnerDevServer::builder()
            .workspace(&self.workspace)
            .enable_engine(true)
            .build()
            .map_err(|e| {
                napi::Error::new(Status::GenericFailure, format!("devserver_start: {e:?}"))
            })?;
        *g = Some(dev);
        Ok(())
    }

    /// Tear down the embedded engine + dev-server state. Subsequent
    /// non-`start` calls return `E_DEVSERVER_STOPPED`. Idempotent.
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        let mut g = self.inner.lock().map_err(poisoned)?;
        // Dropping the inner DevServer releases its Arc<Engine>, which in
        // turn closes the redb file when the last Arc drops. The
        // ReloadCoordinator's subscribers are also dropped here; their
        // ReloadSubscriberJs handles surface E_DEVSERVER_STOPPED on
        // subsequent drain attempts.
        *g = None;
        Ok(())
    }

    /// Register a handler from a DSL source string. Returns the engine-
    /// side handler id (the DSL's declared id, normalised to the caller-
    /// supplied id by the underlying `register_handler_from_dsl`).
    ///
    /// Surfaces typed Diagnostic data on bad DSL input via the napi
    /// error message body — JS-side renderers parse the `error_code:`
    /// prefix to switch on the discriminant.
    #[napi(js_name = "registerHandlerFromDsl")]
    pub fn register_handler_from_dsl(
        &self,
        handler_id: String,
        op: String,
        source: String,
    ) -> napi::Result<String> {
        let g = self.inner.lock().map_err(poisoned)?;
        let dev = g.as_ref().ok_or_else(devserver_stopped)?;
        dev.register_handler_from_dsl(&handler_id, &op, &source)
            .map_err(compile_err_to_napi)
    }

    /// Replace a handler's body from a DSL source string. Same surface as
    /// `registerHandlerFromDsl` but explicit about replace intent — useful
    /// for JS callers that want to assert "this is a hot-reload, not a
    /// first-registration" against their own state machine.
    #[napi(js_name = "replaceHandlerFromDsl")]
    pub fn replace_handler_from_dsl(
        &self,
        handler_id: String,
        op: String,
        source: String,
    ) -> napi::Result<String> {
        let g = self.inner.lock().map_err(poisoned)?;
        let dev = g.as_ref().ok_or_else(devserver_stopped)?;
        dev.replace_handler_from_dsl(&handler_id, &op, &source)
            .map_err(compile_err_to_napi)
    }

    /// R6FP-tail (Round-2 Instance 10) — replace a handler's body from
    /// a DSL source string + return the structured replace outcome
    /// when engine routing is enabled.
    ///
    /// Returns JSON `{ handlerId, cid, previousCid, chainDepth,
    /// versionTag, replaced }` (matching
    /// [`crate::Engine::registerSubgraphReplace`]'s shape) when engine
    /// routing produced a real outcome, OR `{ legacyOnly: true,
    /// handlerId }` when the dev-server is in legacy in-memory mode.
    /// Pre-Instance-10 callers got only the new-CID String back, with
    /// `previousCid` / `chainDepth` / `versionTag` recoverable only via
    /// `subscribeToReloadEvents` correlation.
    #[napi(js_name = "replaceHandlerFromDslWithOutcome")]
    pub fn replace_handler_from_dsl_with_outcome(
        &self,
        handler_id: String,
        op: String,
        source: String,
    ) -> napi::Result<serde_json::Value> {
        let g = self.inner.lock().map_err(poisoned)?;
        let dev = g.as_ref().ok_or_else(devserver_stopped)?;
        match dev
            .replace_handler_from_dsl_with_outcome(&handler_id, &op, &source)
            .map_err(compile_err_to_napi)?
        {
            Some(outcome) => Ok(crate::subgraph::register_replace_outcome_to_json(&outcome)),
            None => {
                let mut m = serde_json::Map::new();
                m.insert("legacyOnly".into(), serde_json::Value::Bool(true));
                m.insert("handlerId".into(), serde_json::Value::String(handler_id));
                Ok(serde_json::Value::Object(m))
            }
        }
    }

    /// Grant a capability to a friendly principal identifier. Mirror of
    /// `Engine.grantCapability` but bound to the dev-server's own grant
    /// table (which survives hot-reload). The `actor` is hashed via the
    /// existing `parse_actor_cid_or_derive` discipline so friendly
    /// strings produce stable synthetic CIDs.
    #[napi(js_name = "grantCapability")]
    pub fn grant_capability(&self, actor: String, scope: String) -> napi::Result<()> {
        let mut g = self.inner.lock().map_err(poisoned)?;
        let dev = g.as_mut().ok_or_else(devserver_stopped)?;
        let actor_cid = crate::node::parse_actor_cid_or_derive(&actor);
        dev.grant(&actor_cid, &scope).map_err(|e| {
            napi::Error::new(
                Status::GenericFailure,
                format!("devserver_grant_capability: {e:?}"),
            )
        })
    }

    /// Whether the friendly principal currently holds the named scope on
    /// the dev-server's grant table.
    #[napi(js_name = "grantExists")]
    pub fn grant_exists(&self, actor: String, scope: String) -> napi::Result<bool> {
        let g = self.inner.lock().map_err(poisoned)?;
        let dev = g.as_ref().ok_or_else(devserver_stopped)?;
        let actor_cid = crate::node::parse_actor_cid_or_derive(&actor);
        Ok(dev.grant_exists(&actor_cid, &scope))
    }

    /// Subscribe to hot-reload events. The returned subscriber's `drain`
    /// reports per-event JSON of `{ handlerId, op, versionTag, newCid?,
    /// previousCid? }`.
    #[napi(js_name = "subscribeToReloadEvents")]
    pub fn subscribe_to_reload_events(&self) -> napi::Result<ReloadSubscriberJs> {
        let g = self.inner.lock().map_err(poisoned)?;
        let dev = g.as_ref().ok_or_else(devserver_stopped)?;
        Ok(ReloadSubscriberJs {
            inner: Arc::new(Mutex::new(Some(dev.subscribe_reload_events()))),
        })
    }

    /// Workspace root the dev-server is rooted at.
    #[napi(getter, js_name = "workspaceRoot")]
    pub fn workspace_root(&self) -> String {
        self.workspace.to_string_lossy().into_owned()
    }
}

// ---------------------------------------------------------------------------
// ReloadSubscriberJs — drainable handle on a reload-event subscription
// ---------------------------------------------------------------------------

/// JS-side reload-event subscriber handle. Drop the JS object to
/// unsubscribe; the publisher prunes dropped buffers lazily (next
/// publish that doesn't see another live `Arc` to the buffer prunes the
/// dead slot).
#[napi]
pub struct ReloadSubscriberJs {
    /// The underlying Rust subscriber. Held in a `Mutex<Option<...>>` so
    /// `unsubscribe()` can drop it eagerly while still permitting the
    /// `drain()` napi method to take `&self`.
    inner: Arc<Mutex<Option<ReloadSubscriber>>>,
}

#[napi]
impl ReloadSubscriberJs {
    /// Drain pending reload events. Each event JSON has the shape:
    /// `{ handlerId: string, op: string, versionTag: string,
    ///    newCid?: string, previousCid?: string }`
    /// where `newCid` / `previousCid` are present only when the
    /// dev-server is engine-routed (the default for the napi binding —
    /// non-engine-routed mode is the legacy Phase-2a in-memory harness
    /// path that this napi surface does not expose).
    #[napi]
    pub fn drain(&self) -> napi::Result<Vec<serde_json::Value>> {
        let g = self.inner.lock().map_err(poisoned)?;
        let sub = g.as_ref().ok_or_else(|| {
            napi::Error::new(
                Status::GenericFailure,
                "E_RELOAD_SUBSCRIBER_UNSUBSCRIBED: drain after unsubscribe",
            )
        })?;
        Ok(sub.drain().into_iter().map(reload_event_to_json).collect())
    }

    /// Whether the subscriber currently has at least one buffered event.
    /// Cheap snapshot — does not drain.
    #[napi(js_name = "hasEvents")]
    pub fn has_events(&self) -> napi::Result<bool> {
        let g = self.inner.lock().map_err(poisoned)?;
        let sub = g.as_ref().ok_or_else(|| {
            napi::Error::new(
                Status::GenericFailure,
                "E_RELOAD_SUBSCRIBER_UNSUBSCRIBED: hasEvents after unsubscribe",
            )
        })?;
        Ok(sub.has_events())
    }

    /// Eagerly unsubscribe. Subsequent `drain()` / `hasEvents()` return
    /// `E_RELOAD_SUBSCRIBER_UNSUBSCRIBED`. Calling `unsubscribe()` twice
    /// is idempotent.
    #[napi]
    pub fn unsubscribe(&self) -> napi::Result<()> {
        let mut g = self.inner.lock().map_err(poisoned)?;
        *g = None;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn poisoned<T>(_: std::sync::PoisonError<T>) -> napi::Error {
    napi::Error::new(
        Status::GenericFailure,
        "devserver_lock_poisoned: a previous panic mid-critical-section corrupted the dev-server's internal lock",
    )
}

fn devserver_stopped() -> napi::Error {
    napi::Error::new(
        Status::GenericFailure,
        "E_DEVSERVER_STOPPED: dev-server has been stopped — call .start() before further operations",
    )
}

/// R6FP-tail (Round-2 Instance 9) — convert a `CompileError` into a napi
/// error preserving the diagnostic's structured `line` / `column` fields
/// for the JS side via the `$$benten-context$$` sentinel suffix used by
/// `engine_err` (see `bindings/napi/src/error.rs::CONTEXT_SENTINEL`).
///
/// Pre-fix the message format `{code}: {msg} (line={line:?} column={col:?})`
/// encoded `Option<u32>` via `{:?}` (yielding `Some(N)` literals);
/// JS callers had to regex-parse `Some\((\d+)\)` to recover the structured
/// fields, ugly + fragile. Post-fix the message prefix carries only the
/// human-readable head + the sentinel suffix carries the JSON
/// `{ "line": N | null, "column": N | null }` bag, which the TS-side
/// `mapNativeError` parses + surfaces on `error.context`.
fn compile_err_to_napi(err: CompileError) -> napi::Error {
    const CONTEXT_SENTINEL: &str = " :: $$benten-context$$";
    match err.diagnostic() {
        Some(d) => {
            let ctx = serde_json::json!({
                "line": d.line,
                "column": d.column,
            });
            let ctx_json = serde_json::to_string(&ctx).unwrap_or_else(|_| "{}".into());
            napi::Error::new(
                Status::GenericFailure,
                format!(
                    "{code}: {msg}{sentinel}{ctx}",
                    code = d.error_code,
                    msg = d.message,
                    sentinel = CONTEXT_SENTINEL,
                    ctx = ctx_json,
                ),
            )
        }
        None => napi::Error::new(
            Status::GenericFailure,
            format!("E_DSL_COMPILE_ERROR: {err}"),
        ),
    }
}

fn reload_event_to_json(ev: ReloadEvent) -> serde_json::Value {
    let mut m = serde_json::Map::with_capacity(5);
    m.insert("handlerId".into(), serde_json::Value::from(ev.handler_id));
    m.insert("op".into(), serde_json::Value::from(ev.op));
    m.insert("versionTag".into(), serde_json::Value::from(ev.version_tag));
    if let Some(c) = ev.new_cid {
        m.insert("newCid".into(), serde_json::Value::from(c.to_base32()));
    }
    if let Some(c) = ev.previous_cid {
        m.insert("previousCid".into(), serde_json::Value::from(c.to_base32()));
    }
    serde_json::Value::Object(m)
}
