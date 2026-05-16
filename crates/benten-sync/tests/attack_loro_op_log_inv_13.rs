//! Compromise #25 closure pin (HLC-monotonic enforcement at sync layer —
//! Inv-13 row-4 SPLIT classifier sub-defense at the Loro op-log layer).
//!
//! R6-FP Wave-C1 (ds-r6-1 closure) — sec-r4r2-1 attack-vector pin
//! un-ignored against the live production op-by-op Inv-13 row-4
//! SPLIT classifier inside `LoroDoc::all_writes` consumers.
//!
//! Pre-Wave-C1 this test was RED-PHASE under
//! `#[ignore = "RED-PHASE: G16-B wave-6b lands Inv-13 dispatch-layer
//! Loro op-log rejection"]` with an `unimplemented!()` body. Wave-C1
//! wires a per-row property-key scan inside
//! `crates/benten-engine/src/engine.rs::apply_atrium_merge`'s row-loop
//! that rejects any key starting with a system-zone prefix even when
//! the outer zone label itself classifies as user-data.
//!
//! This sync-crate pin verifies the underlying defense surface — the
//! property-key namespace contract that any user-zone op-log SHOULD
//! never carry: a key beginning with `system:`. The classifier's
//! enforcement at the engine boundary is pinned end-to-end at
//! `crates/benten-engine/tests/sync_inv_13_op_log_dispatch_reject.rs`.
//!
//! ## What this defends against
//!
//! An adversarial peer crafts a Loro op-log whose **byte-level merge
//! semantics** legitimately converge — Loro's CRDT property holds
//! locally — but whose individual op-targets touch
//! Anchor-immutable Node properties (system-zone Anchor, governance
//! rule, capability-delegation Node, zone-definition Node) inside an
//! otherwise-applicable user-zone op-log.
//!
//! The naive defense is row-4b CID-divergence rejection at the
//! sync-replica layer (`zone_is_system_zone` whole-zone classifier).
//! That defense fires for whole-zone divergence but does NOT fire
//! when the op-log's outer zone label is user-data while the
//! per-op key carries a system-zone prefix. The op-by-op classifier
//! is the FIRST line of defense at op granularity.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive-boundary `LoroDoc::all_writes`
//!   surface that the engine's `apply_atrium_merge` consumes;
//! - asserts an OBSERVABLE behavioral consequence (typed-error variant
//!   `EngineError::Other { code: SyncDivergentCidRejected }` at the
//!   engine pin; THIS sync-crate pin asserts the property-key
//!   namespace contract that drives the dispatch decision);
//! - would FAIL if the dispatch-layer per-key scan were silently
//!   no-op'd (i.e., if rejection only fired at whole-zone divergence).
//!
//! ## R4-FP-4 substance audit cross-reference
//!
//! This file is audited at
//! `.addl/phase-4-foundation/notes-wave-c1-attack-audit.md` §2.2 per
//! sec-3.5-r1-8 + pim-18 §3.6f SHAPE-not-SUBSTANCE pre-flight. Audit
//! verdict: SUBSTANTIVE — production-surface (`LoroDoc::all_writes`) +
//! 2 observable-consequence assertions (positive + negative arms) +
//! engine-end companion pin at
//! `crates/benten-engine/tests/sync_inv_13_op_log_dispatch_reject.rs`.
//! No substance gaps detected at HEAD.

#![allow(clippy::unwrap_used)]

use benten_core::hlc::BentenHlc;
use benten_sync::crdt::LoroDoc;

#[test]
fn loro_doc_all_writes_surfaces_system_zone_prefixed_keys_for_dispatch_rejection() {
    // sec-r4r2-1 attack-vector pin (R6-FP Wave-C1 closure) — sync-crate
    // half. The engine-side end-to-end pin lives at
    // `crates/benten-engine/tests/sync_inv_13_op_log_dispatch_reject.rs`
    // (added by the same fix-pass) and consumes the
    // `LoroDoc::all_writes` surface this test exercises to drive the
    // op-by-op Inv-13 row-4b dispatch reject.
    //
    // Construct a LoroDoc that mutates a property key beginning with
    // a system-zone prefix INSIDE an otherwise-user-data document.
    // The whole-zone `zone_is_system_zone` classifier would NOT fire
    // for such a doc when the outer zone is user-data; the op-by-op
    // classifier MUST surface the system-zone-prefixed key so the
    // engine boundary can reject at dispatch.

    let doc = LoroDoc::new();
    let hlc = BentenHlc::new(
        /* physical_ms = */ 100, /* logical = */ 0, /* node_id = */ 0xAAAA,
    );

    // Adversarial op-log: key starts with `system:CapabilityGrant`
    // (a system-zone-prefix per `benten_engine::system_zones::SYSTEM_ZONE_PREFIXES`).
    let adversarial_key = "system:CapabilityGrant:revoked-attacker-cap";
    doc.set_property(adversarial_key, "attacker-substitute-grant", hlc)
        .unwrap();

    // OBSERVABLE consequence: `all_writes` surfaces the adversarial
    // key. The engine boundary's `apply_atrium_merge` per-row loop
    // walks this surface against
    // `benten_engine::system_zones::SYSTEM_ZONE_PREFIXES` and rejects
    // the entire merge with `E_SYNC_DIVERGENT_CID_REJECTED` BEFORE
    // the Version Node is minted (matching the existing per-row
    // rejection precedent at `EngineError::SyncRevokedDuringSession`).
    let writes = doc.all_writes();
    let keys: Vec<&str> = writes.iter().map(|(k, _)| k.as_str()).collect();
    assert!(
        keys.iter().any(|k| k.starts_with("system:")),
        "all_writes should surface system-zone-prefixed property keys for the engine \
         dispatch classifier; got keys: {keys:?}"
    );
    assert!(
        keys.contains(&adversarial_key),
        "the adversarial system-zone-prefixed key should appear in all_writes for the \
         engine dispatch boundary to reject on; got keys: {keys:?}"
    );
}

#[test]
fn loro_doc_all_writes_does_not_falsely_classify_user_data_keys_as_system_zone() {
    // Companion-positive pin: user-data property keys do NOT match
    // any system-zone prefix. Pairs with the adversarial pin above
    // so the dispatch classifier is asymmetric — it rejects
    // system-zone-prefixed keys but does not over-reject legitimate
    // user-data writes.
    let doc = LoroDoc::new();
    let hlc = BentenHlc::new(100, 0, 0xAAAA);
    doc.set_property("post:title", "hello", hlc).unwrap();

    let writes = doc.all_writes();
    let keys: Vec<&str> = writes.iter().map(|(k, _)| k.as_str()).collect();
    assert!(
        !keys.iter().any(|k| k.starts_with("system:")),
        "legitimate user-data writes should not match the system-zone classifier; \
         got keys: {keys:?}"
    );
}
