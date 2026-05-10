//! Trace projection for TypeScript.
//!
//! Phase 2a G11-A Wave 2b — TraceStep unification: each emitted step is a
//! discriminated union mirroring [`benten_engine::TraceStep`]. The wire
//! shape carries a `type` discriminant so TS callers can `switch` on the
//! variant exhaustively. The four variants:
//!
//! - `{ type: "primitive", nodeCid, durationUs, primitive, nodeId, inputs?,
//!    outputs?, error?, attribution? }`
//! - `{ type: "suspend_boundary", stateCid }`
//! - `{ type: "resume_boundary", stateCid, signalValue }`
//! - `{ type: "budget_exhausted", budgetType, consumed, limit, path }`
//!
//! Top-level shape: `{ steps: [...], result? }`.
//!
//! Pre-Wave-2b shape (`{ nodeCid, durationUs, primitive }` per step) is gone;
//! per CLAUDE.md §5 no compatibility shims, callers consume the new union.

use benten_engine::{Trace, TraceStep};

use crate::node::value_to_json;
use crate::subgraph::outcome_to_json;

#[allow(
    clippy::too_many_lines,
    reason = "single-function dispatch over the four TraceStep variants is the simplest read; splitting per-variant helpers would scatter the discriminant-name string literals across the file."
)]
fn trace_step_to_json(step: &TraceStep) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    match step {
        TraceStep::Step {
            duration_us,
            node_cid,
            primitive,
            node_id,
            inputs,
            outputs,
            error,
            attribution,
        } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("primitive".to_string()),
            );
            obj.insert(
                "nodeCid".to_string(),
                serde_json::Value::String(node_cid.to_base32()),
            );
            obj.insert(
                "durationUs".to_string(),
                serde_json::Value::Number((*duration_us).into()),
            );
            obj.insert(
                "primitive".to_string(),
                serde_json::Value::String(primitive.clone()),
            );
            obj.insert(
                "nodeId".to_string(),
                serde_json::Value::String(node_id.clone()),
            );
            obj.insert("inputs".to_string(), value_to_json(inputs));
            obj.insert("outputs".to_string(), value_to_json(outputs));
            if let Some(code) = error {
                obj.insert(
                    "error".to_string(),
                    serde_json::Value::String(code.as_str().to_string()),
                );
            }
            if let Some(attr) = attribution {
                let mut a = serde_json::Map::new();
                a.insert(
                    "actorCid".to_string(),
                    serde_json::Value::String(attr.actor_cid.to_base32()),
                );
                a.insert(
                    "handlerCid".to_string(),
                    serde_json::Value::String(attr.handler_cid.to_base32()),
                );
                a.insert(
                    "capabilityGrantCid".to_string(),
                    serde_json::Value::String(attr.capability_grant_cid.to_base32()),
                );
                // R6-R3 r6-r3-pcds-1 (Producer/Consumer Drift Instance #15):
                // `AttributionFrame.sandbox_depth` (added PR #62 wiring
                // Inv-4 runtime threading via the
                // `primitive_host.rs::ActiveCall.sandbox_depth` field +
                // `primitive_host.rs::execute_sandbox`'s
                // `frame.sandbox_depth.saturating_add(1)` bump in the
                // SANDBOX entry arm) was being dropped at this
                // projection. Producer = `AttributionFrame` (4
                // fields); pre-fix napi consumer emitted only 3 fields.
                // R6-R4 r6-r4-cp-1 closure: switched from
                // `primitive_host.rs:147/901` line cite (3rd recurrence
                // of the :901 line-drift family) to symbol form per
                // `dispatch-conventions.md` §3.5b high-churn-surface
                // preference.
                // The Inv-4 security claim — SANDBOX-bearing attribution
                // chains content-distinguishable from non-SANDBOX chains —
                // was preserved at the Rust CID level (see
                // `crates/benten-eval/src/exec_state.rs::cid()`) but
                // INVISIBLE to JS consumers. The TS surface
                // `packages/engine/src/types.ts::AttributionFrame` widens
                // alongside this projection so trace-rendering UIs +
                // Phase 6 AI workflow forking can reason about
                // "this step happened at SANDBOX nest-depth N".
                a.insert(
                    "sandboxDepth".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(u64::from(
                        attr.sandbox_depth,
                    ))),
                );
                // Phase-3 G14-D + G16-B AttributionFrame widening
                // (§13.9 Instance 25 closure — Phase-3 sibling of the
                // Phase-2b Instance 18 `sandboxDepth` drift caught at
                // R6 R3 r6-r3-pcds-1). Mirrors
                // `crates/benten-eval/src/exec_state.rs::AttributionFrame::cid()`
                // skip-on-default discipline so trace-JSON for purely-
                // local runs stays byte-identical to the Phase-2a shape
                // (key omitted when value matches the canonical-bytes
                // skip-on-default predicate). The Rust producer
                // populates these slots at the CRDT merge boundary
                // (`apply_atrium_merge`) + device-DID-attested write
                // path; the TS consumer at
                // `packages/engine/src/types.ts::AttributionFrame`
                // mirrors the OPTIONAL slot shape.
                //
                // Field-by-field:
                //   peer_did_set: Option<BTreeSet<Did>>
                //     → "peerDidSet": string[] when Some(non_empty_set);
                //       omitted when None or when set is empty.
                //   device_did: Option<Did>
                //     → "deviceDid": string when Some; omitted when None.
                //   sync_hop_depth: u32
                //     → "syncHopDepth": number when non-zero; omitted at 0.
                //
                // Author's note (DISAGREE-WITH-EXPLANATION per HARD RULE
                // rule-12 clause (c) against the §13.9 brief's
                // `device_cid` field-name): the Rust producer
                // `AttributionFrame` carries `sync_hop_depth: u32`, NOT
                // `device_cid`. The §13.9 brief + the
                // `packages/engine/src/types.ts::AttributionFrame
                // .deviceCid?: string` slot inherited a drift from an
                // earlier design that never landed on the producer side.
                // Emitting the actual producer-present field
                // (`syncHopDepth`) closes the producer/consumer drift;
                // the phantom `deviceCid` TS slot is dropped at
                // `types.ts` in this same fix-pass (correctness over
                // documentation-fidelity to a phantom field).
                if let Some(peer_set) = &attr.peer_did_set
                    && !peer_set.is_empty()
                {
                    let peers: Vec<serde_json::Value> = peer_set
                        .iter()
                        .map(|did| serde_json::Value::String(did.clone()))
                        .collect();
                    a.insert("peerDidSet".to_string(), serde_json::Value::Array(peers));
                }
                if let Some(device) = &attr.device_did {
                    a.insert(
                        "deviceDid".to_string(),
                        serde_json::Value::String(device.clone()),
                    );
                }
                if attr.sync_hop_depth != 0 {
                    a.insert(
                        "syncHopDepth".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(u64::from(
                            attr.sync_hop_depth,
                        ))),
                    );
                }
                obj.insert("attribution".to_string(), serde_json::Value::Object(a));
            }
        }
        TraceStep::SuspendBoundary { state_cid } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("suspend_boundary".to_string()),
            );
            obj.insert(
                "stateCid".to_string(),
                serde_json::Value::String(state_cid.to_base32()),
            );
        }
        TraceStep::ResumeBoundary {
            state_cid,
            signal_value,
        } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("resume_boundary".to_string()),
            );
            obj.insert(
                "stateCid".to_string(),
                serde_json::Value::String(state_cid.to_base32()),
            );
            obj.insert("signalValue".to_string(), value_to_json(signal_value));
        }
        TraceStep::BudgetExhausted {
            budget_type,
            consumed,
            limit,
            path,
        } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("budget_exhausted".to_string()),
            );
            obj.insert(
                "budgetType".to_string(),
                serde_json::Value::String((*budget_type).to_string()),
            );
            obj.insert(
                "consumed".to_string(),
                serde_json::Value::Number((*consumed).into()),
            );
            obj.insert(
                "limit".to_string(),
                serde_json::Value::Number((*limit).into()),
            );
            obj.insert(
                "path".to_string(),
                serde_json::Value::Array(
                    path.iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
    }
    serde_json::Value::Object(obj)
}

pub(crate) fn trace_to_json(trace: &Trace) -> serde_json::Value {
    let steps = trace.steps().iter().map(trace_step_to_json).collect();
    let mut out = serde_json::Map::new();
    out.insert("steps".to_string(), serde_json::Value::Array(steps));
    if let Some(outcome) = trace.outcome() {
        out.insert("result".to_string(), outcome_to_json(outcome));
    }
    serde_json::Value::Object(out)
}

#[cfg(test)]
mod tests {
    //! §13.9 Instance 25 closure (2026-05-10): napi serializer for
    //! AttributionFrame Phase-3 widening fields.
    //!
    //! Pin source: `docs/future/phase-3-backlog.md` §13.9 (BLOCKER —
    //! producer/consumer drift Instance 25; Phase-3 sibling of the
    //! Phase-2b Instance 18 `sandboxDepth` drift caught at R6 R3
    //! r6-r3-pcds-1).
    //!
    //! ## Why inline `#[cfg(test)]` rather than `bindings/napi/tests/`
    //!
    //! The integration-test sibling at
    //! `bindings/napi/tests/attribution_frame_widening_napi_serializer.rs`
    //! carries a grep-against-source-text witness pin (mirror of the
    //! `describe_sandbox.rs` shape: defends against the production
    //! serializer regressing to drop the Phase-3 fields without going
    //! through a libtest link against the napi cdylib externs). The
    //! LOAD-BEARING behavioral assertion lives here because
    //! `trace_step_to_json` is a private free function — accessing it
    //! across the integration-test boundary would either require
    //! promoting the gate of `mod trace` + `mod node` (which carry
    //! napi-rs dep imports that fail to link in the rlib-only
    //! `in-process-test` build mode) OR re-engineering a parallel
    //! test-helpers carrier. Inline unit tests get straightforward
    //! access to the private walker without disturbing the production
    //! cfg gates.
    //!
    //! ## DISAGREE-WITH-EXPLANATION (HARD RULE clause-c) — `device_cid` vs `sync_hop_depth`
    //!
    //! The §13.9 backlog brief + the Phase-3-pre-fix TS interface
    //! (`packages/engine/src/types.ts::AttributionFrame.deviceCid`)
    //! reference a `device_cid: Option<Cid>` slot as the third
    //! Phase-3 widening field. The Rust producer `AttributionFrame`
    //! carries `sync_hop_depth: u32` — the `device_cid` slot lives on
    //! `Engine` / `WriteContext` / `ReadContext` but is never woven
    //! into the AttributionFrame producer. The phantom `deviceCid`
    //! TS field is dropped in the same fix-pass + the real producer
    //! field is mirrored as `syncHopDepth`. The §13.9 backlog entry
    //! is left intact for retrospective traceability but the
    //! on-the-wire reality is `peer_did_set` / `device_did` /
    //! `sync_hop_depth`.

    use std::collections::BTreeSet;

    use benten_core::Cid;
    use benten_eval::AttributionFrame;

    use super::*;

    fn cid(seed: &[u8]) -> Cid {
        Cid::from_blake3_digest(*blake3::hash(seed).as_bytes())
    }

    /// §13.9 LOAD-BEARING pin per pim-2 §3.6b — would FAIL if the napi
    /// serializer regressed to dropping any Phase-3 widening field.
    #[test]
    fn napi_serializer_emits_phase_3_widening_camel_case_keys_when_populated() {
        let mut peers: BTreeSet<String> = BTreeSet::new();
        peers.insert("did:key:peer1".into());
        peers.insert("did:key:peer2".into());

        let attr = AttributionFrame {
            actor_cid: cid(b"actor"),
            handler_cid: cid(b"handler"),
            capability_grant_cid: cid(b"grant"),
            sandbox_depth: 0,
            peer_did_set: Some(peers),
            device_did: Some("did:key:devA".into()),
            sync_hop_depth: 2,
        };

        let step = TraceStep::Step {
            duration_us: 42,
            node_cid: cid(b"node"),
            primitive: "WRITE".into(),
            node_id: "n0".into(),
            inputs: benten_core::Value::Null,
            outputs: benten_core::Value::Null,
            error: None,
            attribution: Some(attr),
        };

        let json = trace_step_to_json(&step);
        let attribution = json
            .get("attribution")
            .expect("step JSON must carry attribution");

        assert_eq!(
            attribution.get("peerDidSet"),
            Some(&serde_json::json!(["did:key:peer1", "did:key:peer2"])),
            "peerDidSet must be present + carry sorted DID strings; \
             got attribution: {attribution}"
        );
        assert_eq!(
            attribution.get("deviceDid"),
            Some(&serde_json::json!("did:key:devA")),
            "deviceDid must be present + carry device DID string"
        );
        assert_eq!(
            attribution.get("syncHopDepth"),
            Some(&serde_json::json!(2)),
            "syncHopDepth must be present + carry non-zero u32"
        );

        // No snake_case leakage at the napi boundary.
        assert!(attribution.get("peer_did_set").is_none());
        assert!(attribution.get("device_did").is_none());
        assert!(attribution.get("sync_hop_depth").is_none());
    }

    /// §13.9 omit-when-default pin per `AttributionFrame::cid()`
    /// skip-on-default discipline. Pre-Phase-3 trace JSON must stay
    /// byte-identical for purely-local runs.
    #[test]
    fn napi_serializer_omits_phase_3_widening_fields_when_unset() {
        let attr = AttributionFrame {
            actor_cid: cid(b"actor"),
            handler_cid: cid(b"handler"),
            capability_grant_cid: cid(b"grant"),
            sandbox_depth: 0,
            peer_did_set: None,
            device_did: None,
            sync_hop_depth: 0,
        };

        let step = TraceStep::Step {
            duration_us: 42,
            node_cid: cid(b"node"),
            primitive: "READ".into(),
            node_id: "n0".into(),
            inputs: benten_core::Value::Null,
            outputs: benten_core::Value::Null,
            error: None,
            attribution: Some(attr),
        };

        let json = trace_step_to_json(&step);
        let attribution = json
            .get("attribution")
            .expect("step JSON must carry attribution");

        // Phase-3 keys MUST be omitted (omit-when-default).
        assert!(
            attribution.get("peerDidSet").is_none(),
            "peerDidSet must be omitted when producer has no peers; \
             got: {attribution}"
        );
        assert!(
            attribution.get("deviceDid").is_none(),
            "deviceDid must be omitted when producer has no device-DID"
        );
        assert!(
            attribution.get("syncHopDepth").is_none(),
            "syncHopDepth must be omitted when sync_hop_depth=0"
        );

        // Pre-Phase-3 always-present fields unaffected by the
        // skip-on-default discipline (regression guard).
        assert!(attribution.get("actorCid").is_some());
        assert!(attribution.get("sandboxDepth").is_some());
    }

    /// §13.9 per-slot skip pin — partial-fill MUST emit only populated
    /// slots, never force absent slots to `null` or all-or-nothing
    /// emission.
    #[test]
    fn napi_serializer_skip_on_default_fires_per_slot() {
        let attr = AttributionFrame {
            actor_cid: cid(b"actor"),
            handler_cid: cid(b"handler"),
            capability_grant_cid: cid(b"grant"),
            sandbox_depth: 0,
            peer_did_set: None,
            device_did: Some("did:key:devB".into()),
            sync_hop_depth: 0,
        };

        let step = TraceStep::Step {
            duration_us: 99,
            node_cid: cid(b"node-partial"),
            primitive: "WRITE".into(),
            node_id: "n1".into(),
            inputs: benten_core::Value::Null,
            outputs: benten_core::Value::Null,
            error: None,
            attribution: Some(attr),
        };

        let json = trace_step_to_json(&step);
        let attribution = json
            .get("attribution")
            .expect("step JSON must carry attribution");

        assert_eq!(
            attribution.get("deviceDid"),
            Some(&serde_json::json!("did:key:devB"))
        );
        assert!(attribution.get("peerDidSet").is_none());
        assert!(attribution.get("syncHopDepth").is_none());

        // Empty peer_did_set treated as default per
        // `AttributionFrame::cid()` discipline.
        let attr_empty = AttributionFrame {
            actor_cid: cid(b"actor"),
            handler_cid: cid(b"handler"),
            capability_grant_cid: cid(b"grant"),
            sandbox_depth: 0,
            peer_did_set: Some(BTreeSet::new()),
            device_did: None,
            sync_hop_depth: 0,
        };
        let step_empty = TraceStep::Step {
            duration_us: 1,
            node_cid: cid(b"node-empty"),
            primitive: "READ".into(),
            node_id: "n2".into(),
            inputs: benten_core::Value::Null,
            outputs: benten_core::Value::Null,
            error: None,
            attribution: Some(attr_empty),
        };
        let json_empty = trace_step_to_json(&step_empty);
        let attribution_empty = json_empty
            .get("attribution")
            .expect("step JSON must carry attribution");
        assert!(
            attribution_empty.get("peerDidSet").is_none(),
            "peerDidSet MUST be omitted when Some(empty set) — mirrors \
             the AttributionFrame::cid() canonical-bytes shape"
        );
    }
}
