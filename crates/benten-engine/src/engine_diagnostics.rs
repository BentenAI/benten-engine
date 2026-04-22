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
                        let ctx = benten_caps::WriteContext {
                            label: primary_label,
                            pending_ops: ops,
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

    /// Metric snapshot for compromise-5 regression tests.
    ///
    /// Surfaces:
    /// - `benten.writes.total` — cumulative ChangeEvents observed.
    /// - `benten.ivm.view_stale_count` — Phase-1 placeholder; Phase-2 wires
    ///   the real counter.
    /// - `benten.change_stream.dropped_events` — ChangeEvents evicted from
    ///   the bounded observed-events buffer because a subscriber fell behind
    ///   the write path (r6-sec-5). Non-zero means an operator should
    ///   increase the capacity via
    ///   [`crate::builder::EngineBuilder::change_stream_capacity`] or ensure
    ///   probes drain.
    #[must_use]
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
        out.insert("benten.ivm.view_stale_count".to_string(), 0.0);
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
            let ctx = benten_caps::ReadContext {
                label: "debug".into(),
                target_cid: Some(*cid),
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
                let ctx = benten_caps::ReadContext {
                    label: label.clone(),
                    target_cid: Some(*cid),
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

    // -------- Version chains (Phase 1 stubs) --------

    pub fn create_anchor(&self, _name: &str) -> Result<AnchorHandle, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "create_anchor — Phase 2",
        })
    }

    pub fn append_version(&self, _anchor: &AnchorHandle, _node: &Node) -> Result<Cid, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "append_version — Phase 2",
        })
    }

    pub fn read_current_version(&self, _anchor: &AnchorHandle) -> Result<Option<Cid>, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "read_current_version — Phase 2",
        })
    }

    pub fn walk_versions(
        &self,
        _anchor: &AnchorHandle,
    ) -> Result<std::vec::IntoIter<Cid>, EngineError> {
        Err(EngineError::NotImplemented {
            feature: "walk_versions — Phase 2",
        })
    }

    pub fn schedule_revocation_at_iteration(
        &self,
        _grant: Cid,
        _n: u32,
    ) -> Result<(), EngineError> {
        Err(EngineError::NotImplemented {
            feature: "schedule_revocation_at_iteration — Phase 2",
        })
    }

    #[cfg(any(test, feature = "test-helpers"))]
    #[allow(
        clippy::expect_used,
        reason = "test-only helper; NoAuth backend cannot deny a plain post"
    )]
    pub fn testing_insert_privileged_fixture(&self) -> Cid {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert("title".into(), Value::Text("secret".into()));
        let node = Node::new(vec!["post".into()], props);
        self.create_node(&node)
            .expect("fixture insertion via NoAuth backend")
    }
}
