//! Snapshot / transaction / metrics / diagnostics / version-chain-stub
//! surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene. Houses `snapshot`,
//! `transaction`, `count_nodes_with_label`, `metrics_snapshot`, the
//! capability write counters, `change_stream_capacity`,
//! `ivm_subscriber_count`, `diagnose_read`, the Phase-2 version-chain
//! stubs, and the `testing_insert_privileged_fixture` helper. Every method
//! is a plain `impl Engine` item.

use std::collections::BTreeMap;

use benten_caps::CapError;
use benten_core::{Cid, Node, Value};
use benten_errors::ErrorCode;
use benten_graph::{GraphError, MutexExt};

use crate::engine::{Engine, derive_committed_scopes, make_engine_tx};
use crate::engine_transaction::EngineTransaction;
use crate::error::EngineError;
use crate::outcome::{AnchorHandle, DiagnosticInfo};

impl Engine {
    // -------- Snapshot + transaction --------

    /// Open a MVCC snapshot handle observing the engine state at the call
    /// instant. Forwards to the graph layer's
    /// [`benten_graph::RedbBackend::snapshot`].
    pub fn snapshot(&self) -> Result<benten_graph::SnapshotHandle, EngineError> {
        Ok(self.backend.snapshot()?)
    }

    /// Run a closure inside a write transaction.
    pub fn transaction<F, R>(&self, f: F) -> Result<R, EngineError>
    where
        F: FnOnce(&mut EngineTransaction<'_, '_>) -> Result<R, EngineError>,
    {
        use std::sync::Mutex;
        let ops_cell: Mutex<Vec<benten_caps::PendingOp>> = Mutex::new(Vec::new());
        let user_result: Mutex<Option<Result<R, EngineError>>> = Mutex::new(None);

        let policy = self.policy.as_deref();

        let tx_outcome = self.backend.transaction(|tx| {
            let mut eng_tx = make_engine_tx(tx, &ops_cell);
            match f(&mut eng_tx) {
                Ok(value) => {
                    let ops = ops_cell.lock_recover().clone();
                    // Derive the per-capability-scope list from the batch
                    // (Phase-1 posture: `store:<label>:write` per op; system
                    // zone skipped since user subgraphs cannot reach it).
                    // Empty labels collapse to `store:write`. Closes named
                    // compromise #5 — record, don't enforce.
                    let scopes = derive_committed_scopes(&ops);
                    if let Some(p) = policy
                        && !ops.is_empty()
                    {
                        let primary_label = ops
                            .iter()
                            .find_map(|op| match op {
                                benten_caps::PendingOp::PutNode { labels, .. } => {
                                    labels.first().cloned()
                                }
                                benten_caps::PendingOp::PutEdge { label, .. } => {
                                    Some(label.clone())
                                }
                                _ => None,
                            })
                            .unwrap_or_default();
                        // Phase-3 G16-B-prime (§6.12 item 3): thread the
                        // engine's configured device-DID-attestation CID
                        // through to the policy. `None` for legacy /
                        // non-attested engines preserves prior behavior;
                        // `Some(cid)` lets heterogeneous policies dispatch
                        // per device per D-PHASE-3-25.
                        let device_cid =
                            *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
                        let ctx = benten_caps::CapWriteContext {
                            label: primary_label,
                            pending_ops: ops,
                            device_cid,
                            ..Default::default()
                        };
                        if let Err(cap_err) = p.check_write(&ctx) {
                            self.inner.record_cap_write_denied(&scopes);
                            *user_result.lock_recover() = Some(Err(EngineError::Cap(cap_err)));
                            return Err(GraphError::TxAborted {
                                reason: "capability denied".into(),
                            });
                        }
                    }
                    // Record committed writes regardless of whether a policy
                    // was configured — the metric is operational, not a gate.
                    // Under NoAuthBackend the scope list still derives from
                    // the batch's labels so dashboards can spot traffic-by-
                    // label without a policy plumbed in.
                    if !scopes.is_empty() {
                        self.inner.record_cap_write_committed(&scopes);
                    }
                    *user_result.lock_recover() = Some(Ok(value));
                    Ok(())
                }
                Err(e) => {
                    *user_result.lock_recover() = Some(Err(e));
                    Err(GraphError::TxAborted {
                        reason: "closure error".into(),
                    })
                }
            }
        });

        let saved = user_result.into_inner().unwrap_or_else(|e| e.into_inner());
        if let Some(r) = saved {
            return r;
        }
        match tx_outcome {
            Ok(()) => {
                debug_assert!(false, "transaction returned Ok without saved result");
                Err(EngineError::Other {
                    code: ErrorCode::Unknown(String::from("engine_internal")),
                    message: "transaction returned Ok without saved result".into(),
                })
            }
            Err(GraphError::NestedTransactionNotSupported {}) => {
                Err(EngineError::NestedTransactionNotSupported)
            }
            Err(e) => Err(EngineError::Graph(e)),
        }
    }

    // -------- Metrics + diagnostics --------

    /// Count nodes stored under a label via the label index.
    pub fn count_nodes_with_label(&self, label: &str) -> Result<usize, EngineError> {
        Ok(self.backend.get_by_label(label)?.len())
    }

    /// Metric snapshot for compromise-5 regression tests + Phase-3
    /// observer dashboard surface.
    ///
    /// Phase-1 / Phase-2 surfaces:
    /// - `benten.writes.total` — cumulative ChangeEvents observed.
    /// - `benten.ivm.view_stale_count` — Phase-1 placeholder; Phase-2 wires
    ///   the real counter.
    /// - `benten.change_stream.dropped_events` — ChangeEvents evicted from
    ///   the bounded observed-events buffer because a subscriber fell behind
    ///   the write path (r6-sec-5). Non-zero means an operator should
    ///   increase the capacity via
    ///   [`crate::builder::EngineBuilder::change_stream_capacity`] or ensure
    ///   probes drain.
    /// - `benten.writes.committed` / `benten.writes.denied` aggregates
    ///   plus per-scope fan-out keys.
    ///
    /// Phase-3 R6 fp Wave C2 additions (closes obs-r6r1-2 MAJOR — Phase-3
    /// observability counters not surfaced in the canonical operator-
    /// dashboard key/value bag):
    /// - `benten.sandbox.handler.<handler_id>.fuel_consumed_high_water` —
    ///   per-handler SANDBOX cumulative fuel high-water (monotonic).
    /// - `benten.sandbox.handler.<handler_id>.output_consumed_high_water` —
    ///   per-handler SANDBOX cumulative guest-output bytes high-water
    ///   (monotonic; closes the §7.1 trio with fuel + last_invocation_ms).
    /// - `benten.sandbox.handler.<handler_id>.last_invocation_ms` — wall-
    ///   clock duration of the most recent invocation (NOT cumulative).
    /// - `benten.subscribe.on_change_registration_count` — total active +
    ///   inactive ad-hoc onChange entries (eval-side global registry).
    /// - `benten.emit.subscriber_count` — registered subscribers on the
    ///   EMIT broadcast bus.
    /// - `benten.sync_replica.cap_recheck_calls` — cumulative count of
    ///   per-row cap-recheck calls fired by `apply_atrium_merge`'s
    ///   structural-always-on per-write loop (G16-B-F sec-r4r1-2 closure).
    #[must_use]
    #[allow(clippy::too_many_lines)] // Sequential lift of independent observability counters; splitting harms read-top-to-bottom narrative.
    pub fn metrics_snapshot(&self) -> BTreeMap<String, f64> {
        let mut out = BTreeMap::new();
        let n = self
            .inner
            .event_count
            .load(std::sync::atomic::Ordering::SeqCst);
        let dropped = self
            .inner
            .dropped_events
            .load(std::sync::atomic::Ordering::SeqCst);
        let committed_total = self
            .inner
            .writes_committed_total
            .load(std::sync::atomic::Ordering::SeqCst);
        let denied_total = self
            .inner
            .writes_denied_total
            .load(std::sync::atomic::Ordering::SeqCst);
        #[allow(
            clippy::cast_precision_loss,
            reason = "Phase-1 metric is best-effort; lossy cast from u64 to f64 is acceptable for the compromise-5 regression test."
        )]
        {
            out.insert("benten.writes.total".to_string(), n as f64);
            out.insert(
                "benten.change_stream.dropped_events".to_string(),
                dropped as f64,
            );
            // Named compromise #5: per-capability write metrics. The totals
            // are aggregate (one tick per commit); the per-scope keys
            // `benten.writes.committed.<scope>` fan-out so operators can
            // spot abnormal traffic per label before Phase-3 enforcement
            // lands.
            out.insert(
                "benten.writes.committed".to_string(),
                committed_total as f64,
            );
            out.insert("benten.writes.denied".to_string(), denied_total as f64);
            for (scope, count) in self.inner.cap_write_committed_snapshot() {
                out.insert(format!("benten.writes.committed.{scope}"), count as f64);
            }
            for (scope, count) in self.inner.cap_write_denied_snapshot() {
                out.insert(format!("benten.writes.denied.{scope}"), count as f64);
            }
        }
        // G11-A Wave 1: replace the R3-consolidation hard-code with the
        // real tally from the IVM subscriber. When `.without_ivm()` was
        // configured the subscriber is absent — surface 0 in that case so
        // operators comparing metrics across configurations don't see a
        // phantom view go stale when no views exist at all.
        let stale = self.ivm.as_ref().map_or(0, |s| s.stale_count_tally());
        #[allow(
            clippy::cast_precision_loss,
            reason = "stale view counts are bounded by registered-view count; f64 is lossless well past any realistic view-registry size"
        )]
        out.insert("benten.ivm.view_stale_count".to_string(), stale as f64);

        // R6 fp Wave C2 (obs-r6r1-2 closure): lift the Phase-3
        // observability counters that were public accessors only into
        // the canonical operator-dashboard key/value bag.

        // Per-handler SANDBOX fuel/output/wallclock high-water. The
        // accessor returns a fresh clone of the per-handler map so
        // formatting doesn't hold the metric lock.
        #[allow(
            clippy::cast_precision_loss,
            reason = "SANDBOX high-water values are u64 monotonic counters; lossy f64 cast acceptable for operator-dashboard surface (matches the writes.committed / dropped_events precedent above)"
        )]
        for (handler_id, metrics) in self.inner.sandbox_metric_snapshot_all() {
            if let Some(fuel) = metrics.fuel_consumed_high_water {
                out.insert(
                    format!("benten.sandbox.handler.{handler_id}.fuel_consumed_high_water"),
                    fuel as f64,
                );
            }
            if let Some(output) = metrics.output_consumed_high_water {
                out.insert(
                    format!("benten.sandbox.handler.{handler_id}.output_consumed_high_water"),
                    output as f64,
                );
            }
            if let Some(last_ms) = metrics.last_invocation_ms {
                out.insert(
                    format!("benten.sandbox.handler.{handler_id}.last_invocation_ms"),
                    last_ms as f64,
                );
            }
        }

        // SUBSCRIBE on_change registration count (eval-side process-
        // scoped registry). The accessor surfaces total active + inactive
        // entries; GC reaps inactive ones on each publish so a steady
        // non-zero value reflects active subscriber counts.
        #[allow(
            clippy::cast_precision_loss,
            reason = "subscriber counts bounded by registered-handler count; f64 lossless past realistic registry sizes"
        )]
        {
            let on_change_count =
                benten_eval::primitives::subscribe::on_change_registration_count();
            out.insert(
                "benten.subscribe.on_change_registration_count".to_string(),
                on_change_count as f64,
            );

            // EMIT broadcast subscriber count.
            let emit_subs = self.emit_subscriber_count();
            out.insert("benten.emit.subscriber_count".to_string(), emit_subs as f64);
        }

        // Sync-replica per-row cap-recheck count (G16-B-F sec-r4r1-2
        // BLOCKER closure). Cumulative count of per-write cap-recheck
        // calls fired by apply_atrium_merge's structural-always-on loop.
        #[allow(
            clippy::cast_precision_loss,
            reason = "cap-recheck call count is a u64 cumulative counter; lossy f64 cast acceptable for operator-dashboard surface"
        )]
        {
            let recheck_calls = self.sync_replica_cap_recheck_calls();
            out.insert(
                "benten.sync_replica.cap_recheck_calls".to_string(),
                recheck_calls as f64,
            );
        }

        // R6 fp Wave-C2 follow-up (obs-r6-r2-1 sibling-class closure):
        // lift the Phase-2b STREAM `active_stream_count` accessor into
        // the canonical operator-dashboard bag so metrics_snapshot is
        // complete across the 12-primitive observability surface.
        // (The Phase-3 sync-frame `inbound_hlc_skew_classifier_calls`
        // accessor sits on `AtriumHandle` rather than `Engine`; lifting
        // it requires per-atrium iteration + filed at phase-3-backlog
        // §6.13a as v1-window follow-up coupling to the existing
        // AtriumConfig.skew_tolerance_ms operator-tuneable carry.)
        #[allow(
            clippy::cast_precision_loss,
            reason = "active_stream_count is a usize counter bounded by per-process producer-bridge handle count; lossy f64 cast acceptable for operator-dashboard surface"
        )]
        {
            let active_streams = self.active_stream_count();
            out.insert(
                "benten.stream.active_count".to_string(),
                active_streams as f64,
            );
        }

        out
    }

    /// Per-capability-scope committed-write counter snapshot. Keys are the
    /// derived scope strings (`store:<label>:write`); values are the number
    /// of batches committed under each scope since the engine opened. Used
    /// by the compromise-#5 regression test and by napi callers that want
    /// the map shape directly without the flattened `metrics_snapshot`
    /// string-keyed projection.
    #[must_use]
    pub fn capability_writes_committed(&self) -> BTreeMap<String, u64> {
        self.inner.cap_write_committed_snapshot()
    }

    /// Per-capability-scope denied-write counter snapshot. Mirrors
    /// [`Self::capability_writes_committed`] for batches the policy
    /// rejected.
    #[must_use]
    pub fn capability_writes_denied(&self) -> BTreeMap<String, u64> {
        self.inner.cap_write_denied_snapshot()
    }

    /// Configured upper bound on the in-memory change-event buffer. Matches
    /// the value passed to
    /// [`crate::builder::EngineBuilder::change_stream_capacity`] (or
    /// [`crate::engine::CHANGE_STREAM_MAX_BUFFERED`] when the default was
    /// taken). See r6-sec-5.
    #[must_use]
    pub fn change_stream_capacity(&self) -> usize {
        self.inner.change_stream_capacity
    }

    /// IVM subscriber count — used by thinness tests. Excludes the
    /// engine-internal change broadcast tap (which is always present so
    /// `subscribe_change_events` works).
    ///
    /// Returns the number of views registered against the IVM subscriber, or
    /// 0 when `.without_ivm()` was passed. When IVM is enabled but no views
    /// have been created yet (fresh engine), this also returns 0 — the
    /// subscriber itself is wired but there's nothing to fan events out to.
    /// See philosophy g7-ep-3 / code-reviewer g7-cr-8.
    #[must_use]
    pub fn ivm_subscriber_count(&self) -> usize {
        self.ivm.as_ref().map_or(0, |s| s.view_count())
    }

    /// Option-C diagnostic for a denied / missing read.
    ///
    /// Requires the caller to hold a `debug:read` capability — the
    /// configured policy's `check_read` is consulted with label
    /// `"debug"` and `target_cid = Some(cid)`. When the policy denies,
    /// `diagnose_read` returns `Err(EngineError::Cap(CapError::Denied))`
    /// so an ordinary caller cannot fish the existence signal.
    ///
    /// When permitted, the returned [`DiagnosticInfo`] distinguishes
    /// three states: "not in backend", "in backend but policy denied",
    /// "in backend and policy permitted". See named compromise #2 in
    /// `docs/SECURITY-POSTURE.md` for the full semantics.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Cap`] when the caller lacks `debug:read`.
    /// Backend read failures bubble through [`EngineError::Graph`].
    pub fn diagnose_read(&self, cid: &Cid) -> Result<DiagnosticInfo, EngineError> {
        // Gate on `debug:read`. We thread the probe through the configured
        // policy's check_read with a canonical `"debug"` label so a
        // Phase-1 GrantBackedPolicy + grant("...", "store:debug:read")
        // unlocks the diagnostic surface. Absent a policy, diagnose_read
        // is open (matches NoAuth posture — embedded single-user
        // deployments get diagnostics out of the box).
        if let Some(policy) = self.policy.as_deref() {
            // Phase-3 G16-B-prime fp (consumer-audit closure of cor-1 /
            // cap-g16bp-3): thread the engine's configured device-DID-
            // attestation CID into the debug:read gate ReadContext so
            // heterogeneous policies dispatch per-device per D-PHASE-3-25.
            let device_cid = *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
            let ctx = benten_caps::ReadContext {
                label: "debug".into(),
                target_cid: Some(*cid),
                device_cid,
                ..Default::default()
            };
            if let Err(e) = policy.check_read(&ctx) {
                // Normalise to CapError::Denied on the diagnostic path —
                // a DeniedRead on this gate is itself the denial signal.
                let required = match &e {
                    CapError::DeniedRead { required, .. } | CapError::Denied { required, .. } => {
                        required.clone()
                    }
                    _ => "store:debug:read".to_string(),
                };
                return Err(EngineError::Cap(CapError::Denied {
                    required,
                    entity: cid.to_base32(),
                }));
            }
        }

        // Now inspect actual state. Probe the backend unconditionally —
        // we already gated on the debug:read capability.
        let existing = self.backend.get_node(cid)?;
        let (exists_in_backend, label) = match &existing {
            Some(n) => (true, n.labels.first().cloned().unwrap_or_default()),
            None => (false, String::new()),
        };

        // Recompute the policy's verdict on the *real* label so the
        // DiagnosticInfo carries an accurate `denied_by_policy` signal.
        let denied_by_policy = if exists_in_backend {
            if let Some(policy) = self.policy.as_deref() {
                // Phase-3 G16-B-prime fp (consumer-audit closure of cor-1
                // / cap-g16bp-3): thread device-DID-attestation CID for
                // diagnostic-replay symmetry with the gate path above.
                let device_cid = *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
                let ctx = benten_caps::ReadContext {
                    label: label.clone(),
                    target_cid: Some(*cid),
                    device_cid,
                    ..Default::default()
                };
                match policy.check_read(&ctx) {
                    Err(CapError::DeniedRead { required, .. }) => Some(required),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(DiagnosticInfo {
            cid: *cid,
            exists_in_backend,
            denied_by_policy,
            not_found: !exists_in_backend,
        })
    }

    // -------- Version chains (Phase-3 G16-B-prime: real wireup) --------

    /// Create a new version-chain anchor under `name`.
    ///
    /// Phase-3 G16-B-prime (§6.12 item 1): the engine mints a fresh
    /// [`benten_core::version::Anchor`] rooted at a name-derived seed
    /// CID + records it in the in-memory anchor store. Subsequent
    /// [`Self::append_version`] calls thread the prior head per the
    /// `core::version` contract; [`Self::read_current_version`]
    /// answers the CURRENT pointer.
    ///
    /// The seed CID is derived deterministically from the name (BLAKE3
    /// of `b"benten-anchor-seed:" || name`); two engines that call
    /// `create_anchor("post:p1")` produce the same seed CID
    /// independently, but the anchor STATE is per-engine (unbounded
    /// concurrent appends across two processes is a Phase-3+ sync
    /// concern; G16-B-prime's in-memory anchor store covers the
    /// single-engine case). Re-creating an anchor under a name that
    /// already exists is a no-op (returns the existing handle), keeping
    /// the call idempotent across replays.
    ///
    /// # Errors
    ///
    /// Currently infallible at engine scope (the in-memory store cannot
    /// fail at insert); the `Result` shape is kept so the durable-
    /// promotion at §1.1 GraphBackend umbrella can introduce I/O
    /// errors without breaking call sites.
    pub fn create_anchor(&self, name: &str) -> Result<AnchorHandle, EngineError> {
        let seed_cid = anchor_seed_cid_for_name(name);
        let mut store = benten_graph::MutexExt::lock_recover(&self.inner.anchor_store);
        store.entry(name.to_string()).or_insert_with(|| {
            let anchor = benten_core::version::Anchor::new(seed_cid);
            crate::engine::AnchorEntry {
                anchor,
                current: seed_cid,
            }
        });
        Ok(AnchorHandle {
            name: name.to_string(),
        })
    }

    /// Append a new version Node to the chain rooted at `anchor`.
    ///
    /// Phase-3 G16-B-prime: persists the Node bytes via the underlying
    /// `benten_graph` backend's transactional `put_node` surface, then
    /// calls `benten_core::version::append_version` to advance the chain
    /// (refusing forks via `benten_core::version::VersionError`).
    ///
    /// G16-B-E (Sub-item D — receiver-side ChangeEvent fan-out): the
    /// put goes through `backend.transaction(|tx| tx.put_node(node))`
    /// rather than the inherent `backend.put_node(node)` — the
    /// transactional path is the one that fires ChangeEvents to
    /// registered subscribers (the engine's `ChangeBroadcast` +
    /// IVM-view subscribers), per `RedbBackend::with_transaction`'s
    /// fan-out rule. Without this routing, `apply_atrium_merge`'s
    /// receiver-side pin (`subscribe_change_events` ChangeProbe) would
    /// observe ZERO events on a successful Loro merge — silently
    /// breaking the plan §1 exit-criterion 1 contract ("ChangeEvent
    /// fan-out + IVM-view materialization on the receiver").
    ///
    /// # Errors
    ///
    /// - [`EngineError::Graph`] on backend put failure.
    /// - [`EngineError::Other`] (carrying [`ErrorCode::VersionBranched`]
    ///   or [`ErrorCode::VersionUnknownPrior`]) when the chain refuses
    ///   the append (concurrent fork or an unobserved prior head).
    /// - [`EngineError::Other`] (carrying [`ErrorCode::NotFound`])
    ///   when the anchor handle's name is not in the engine's anchor
    ///   store. Pre-G16-B-prime call sites that constructed
    ///   `AnchorHandle` outside `create_anchor` cannot exist (the
    ///   field is `pub(crate)`); the catch-all keeps future cross-
    ///   engine handle leaks honest.
    pub fn append_version(&self, anchor: &AnchorHandle, node: &Node) -> Result<Cid, EngineError> {
        // G16-B-E Sub-item D: route through `backend.transaction` so
        // registered ChangeBroadcast subscribers fan out (ChangeEvents
        // for IVM-view materialization + engine-side `subscribe_change_events`
        // probes). The inherent `backend.put_node` does NOT fire the
        // fan-out (it bypasses `with_transaction`'s pending-events
        // pipeline) — using the transactional surface preserves the
        // receiver-side observability contract.
        let new_head = self
            .backend
            .transaction(|tx| tx.put_node(node))
            .map_err(EngineError::Graph)?;
        let mut store = benten_graph::MutexExt::lock_recover(&self.inner.anchor_store);
        let entry = store
            .get_mut(&anchor.name)
            .ok_or_else(|| EngineError::Other {
                code: ErrorCode::NotFound,
                message: format!("anchor not found: '{}'", anchor.name),
            })?;
        let prior = entry.current;
        // append_version refuses forks (Branched) + unknown priors via
        // the prior-threaded API.
        benten_core::version::append_version(&entry.anchor, &prior, &new_head).map_err(|e| {
            EngineError::Other {
                code: e.code(),
                message: e.to_string(),
            }
        })?;
        entry.current = new_head;
        Ok(new_head)
    }

    /// Read the CID of the version currently pointed at by `anchor`.
    ///
    /// Phase-3 G16-B-prime: returns the CURRENT head from the engine's
    /// anchor store. `None` would indicate the anchor was created but
    /// no [`Self::append_version`] has landed yet — the post-
    /// G16-B-prime contract is "anchors always carry a head" (the seed
    /// at create-time), so this call returns `Some(seed)` after
    /// `create_anchor` and the most-recent appended head thereafter.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Other`] (carrying
    /// [`ErrorCode::NotFound`]) when the handle's name is not in
    /// the engine's anchor store.
    pub fn read_current_version(&self, anchor: &AnchorHandle) -> Result<Option<Cid>, EngineError> {
        let store = benten_graph::MutexExt::lock_recover(&self.inner.anchor_store);
        let entry = store.get(&anchor.name).ok_or_else(|| EngineError::Other {
            code: ErrorCode::NotFound,
            message: format!("anchor not found: '{}'", anchor.name),
        })?;
        Ok(Some(entry.current))
    }

    /// Phase-3 G16-B-prime (§6.12 item 3): set the local engine's
    /// device-DID-attestation CID. Subsequent
    /// [`benten_caps::CapWriteContext`] / [`benten_caps::ReadContext`]
    /// constructions inside engine-internal call sites populate the
    /// `device_cid` field with this value, letting heterogeneous
    /// `CapabilityPolicy` impls dispatch per-device under the SAME
    /// logical-actor identity per D-PHASE-3-25.
    ///
    /// Pass `None` to clear (default state). Pass `Some(cid)` after
    /// validating a [`benten_id::device_attestation::DeviceAttestation`]
    /// envelope at the engine boundary; the CID is the content-addressed
    /// identifier of that envelope.
    ///
    /// **Production threading:** the engine's diagnostics
    /// (`engine_diagnostics.rs::transaction`) commit hook + the
    /// primitive-host (`primitive_host.rs::check_capability`)
    /// pre-write cap-check arm both populate `device_cid` from this
    /// setter. Legacy engines that never call this setter behave as
    /// before (`device_cid: None`).
    ///
    /// **Mutability contract (cap-g16bp-4):** this setter is mutable
    /// across the engine's lifetime to support device-rotation. A
    /// caller invokes `set_device_cid(Some(new_cid))` after re-
    /// validating a fresh [`benten_id::device_attestation::DeviceAttestation`]
    /// envelope (e.g. after a compromised device is revoked + a
    /// freshly-attested device is bound). Re-setting with the same
    /// value is a no-op; re-setting with a different value updates
    /// the engine's device-CID slot. The setter is intentionally
    /// **NOT locked-once** because device-DID rotation is an expected
    /// operational pattern under the multi-device contract, not an
    /// abuse vector — the producing-device-DID slot in
    /// AttributionFrame must reflect the current device, and freezing
    /// the slot would break the rotation flow.
    pub fn set_device_cid(&self, device_cid: Option<Cid>) {
        let mut g = benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
        *g = device_cid;
    }

    /// Phase-3 G16-B-prime: read the engine's configured
    /// device-DID-attestation CID. Round-trip companion to
    /// [`Self::set_device_cid`].
    #[must_use]
    pub fn device_cid(&self) -> Option<Cid> {
        *benten_graph::MutexExt::lock_recover(&self.inner.device_cid)
    }

    /// Phase-3 G16-B-prime fp (cap-g16bp-1 closure / Ben's RATIFIED
    /// Option A 2026-05-08): set the engine's logical-actor identity.
    ///
    /// This is the identity that vouches for writes initiated through
    /// this engine, including sync-merged writes minted at
    /// [`Self::apply_atrium_merge`]. Defaults to [`Self::device_cid`]
    /// when unset (single-user single-device case). Phase-4+ AI-agent
    /// flows set this explicitly so AttributionFrame.actor_cid retains
    /// the PRINCIPAL identity while AttributionFrame.device_did still
    /// reflects the producing DEVICE.
    ///
    /// Pass `None` to clear (default state). Pass `Some(cid)` after
    /// the engine has bound itself to a logical-actor identity (e.g.
    /// the user's parent DID or an AI-agent's delegated identity).
    pub fn set_actor_cid(&self, actor_cid: Option<Cid>) {
        let mut g = benten_graph::MutexExt::lock_recover(&self.inner.actor_cid);
        *g = actor_cid;
    }

    /// Phase-3 G16-B-prime fp (cap-g16bp-1 closure): read the engine's
    /// configured logical-actor CID, falling back to [`Self::device_cid`]
    /// when unset. Round-trip companion to [`Self::set_actor_cid`] but
    /// returns the EFFECTIVE actor identity used at AttributionFrame
    /// minting (so callers don't have to replicate the fallback).
    #[must_use]
    pub fn effective_actor_cid(&self) -> Option<Cid> {
        match *benten_graph::MutexExt::lock_recover(&self.inner.actor_cid) {
            Some(cid) => Some(cid),
            None => *benten_graph::MutexExt::lock_recover(&self.inner.device_cid),
        }
    }

    /// Walk the full version-chain history under `anchor`, yielding
    /// each ancestor CID in order (oldest → newest, including the
    /// seed head).
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Other`] (carrying
    /// [`ErrorCode::NotFound`]) when the handle's name is not in
    /// the engine's anchor store.
    pub fn walk_versions(
        &self,
        anchor: &AnchorHandle,
    ) -> Result<std::vec::IntoIter<Cid>, EngineError> {
        let store = benten_graph::MutexExt::lock_recover(&self.inner.anchor_store);
        let entry = store.get(&anchor.name).ok_or_else(|| EngineError::Other {
            code: ErrorCode::NotFound,
            message: format!("anchor not found: '{}'", anchor.name),
        })?;
        Ok(benten_core::version::walk_versions(&entry.anchor))
    }

    /// Phase 2a G9-A-cont: record a target iteration at which `grant`
    /// should be treated as revoked by the in-process TOCTOU harness.
    ///
    /// The scheduled target is consulted by the evaluator's wall-clock-
    /// refresh callback (`impl PrimitiveHost::check_capability`) when
    /// `iterate_batch_boundary` triggers a cap re-check. A test that
    /// calls [`Self::schedule_revocation_at_iteration`] with `(grant, 50)`
    /// and then drives a 300-iter handler will observe the denial at
    /// the first boundary past iteration 50 — matching the §9.13
    /// refresh-point-3 semantics.
    ///
    /// This is an IN-PROCESS test harness surface. It is NOT wired to
    /// the production revocation path (`system:CapabilityRevocation`
    /// Node writes + `GrantBackedPolicy::check_write`); those remain
    /// authoritative for end-to-end tests.
    ///
    /// # Errors
    /// Returns [`EngineError::Other`] on lock recovery failure (never
    /// fires under sound state — kept as a safe-by-default shape).
    pub fn schedule_revocation_at_iteration(&self, grant: Cid, n: u32) -> Result<(), EngineError> {
        let mut guard = benten_graph::MutexExt::lock_recover(&self.revoke_at_iteration);
        guard.insert(grant, u64::from(n));
        Ok(())
    }

    /// Test-only helper — insert a fixture `post` Node via the
    /// privileged NoAuth backend path so cap-policy regression suites
    /// can populate state without round-tripping through the public
    /// `Engine::call` surface. Returns the inserted Node's CID.
    ///
    /// refinement-audit-2026-05 #615/#617 (ST-GRAPH lane, umbrella
    /// #1208, META #660 Inv-13 5-row matrix) — cross-lane contract
    /// propagation, the §3.5l cross-crate-consumer class the disjoint
    /// single-crate review could not reach. The pre-bypass-close helper
    /// minted a FIXED `post` Node (constant CID) and relied on the bare
    /// `put_node` REPLACE-on-collision path so callers could invoke it
    /// repeatedly (e.g. `view_stale_count_tallies`' 128-insert burst).
    /// With Inv-13 Row-1 closed, the 2nd+ identical-content insert under
    /// `WriteAuthority::User` is now correctly an immutability violation.
    /// Each call now embeds a process-global monotonic discriminator so
    /// every insert is genuinely-distinct content (a real new write that
    /// drives view churn) — the helper's contract ("returns a freshly
    /// inserted fixture Node's CID") is preserved and strengthened (no
    /// silent REPLACE was ever the intent). Single-call sites are
    /// unaffected: they consume *a* valid fixture CID, not a specific
    /// deterministic one.
    #[cfg(any(test, feature = "test-helpers"))]
    #[allow(
        clippy::expect_used,
        reason = "test-only helper; NoAuth backend cannot deny a plain post"
    )]
    pub fn testing_insert_privileged_fixture(&self) -> Cid {
        use std::sync::atomic::{AtomicU64, Ordering};
        static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed);
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("title".into(), Value::Text("secret".into()));
        // Distinct per-call discriminator so each invocation is a
        // genuinely-new content-addressed write (post Inv-13 Row-1
        // bypass close — no silent REPLACE of identical content).
        props.insert("_fixture_seq".into(), Value::Text(format!("fixture-{seq}")));
        let node = Node::new(vec!["post".into()], props);
        self.create_node(&node)
            .expect("fixture insertion via NoAuth backend")
    }
}

/// Phase-3 G16-B-prime helper (§6.12 item 1): derive a deterministic
/// seed CID for an anchor name. Used by [`Engine::create_anchor`] so two
/// engines that mint an anchor under the same name agree on the
/// pre-append seed CID. Pre-existing anchor names are stable across
/// engine restarts because the seed is name-derived, not random.
///
/// The blake3 prefix `b"benten-anchor-seed:"` namespaces this derivation
/// off other content-addressed CIDs in the system (Node bodies, Edge
/// bodies, capability grants) so the seed cannot collide with a
/// real-content CID for any practical input.
fn anchor_seed_cid_for_name(name: &str) -> Cid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"benten-anchor-seed:");
    hasher.update(name.as_bytes());
    let digest = *hasher.finalize().as_bytes();
    Cid::from_blake3_digest(digest)
}
