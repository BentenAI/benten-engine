//! Phase-3 G16-B-prime (§6.12 item 3) GREEN-PHASE pin:
//! `device_cid` is threaded from the engine's configured device-DID-
//! attestation CID through the production write-path call sites
//! ([`engine_diagnostics.rs::transaction`] commit hook +
//! [`primitive_host.rs::check_capability`]) into the
//! [`benten_caps::WriteContext`] passed to the configured
//! [`benten_caps::CapabilityPolicy`].
//!
//! Pin source: predecessor RED-PHASE pin in
//! `crates/benten-caps/tests/device_dispatch.rs` was retired
//! 2026-05-08 G16-B-prime — the cross-crate runtime-arm test moved
//! here because it requires the `benten-engine` dependency.
//!
//! ## Architectural intent (cap-r4-4 (c) + r4b-cap-3 BLOCKER closure)
//!
//! Per D-PHASE-3-25, heterogeneous policies dispatch on `device_cid` to
//! surface different decisions for "desktop X writes" vs "phone X
//! writes" under the SAME logical actor identity. G16-B canary landed
//! the structural surface (the `device_cid` field on `WriteContext` /
//! `ReadContext`); G16-B-prime threads the field at the engine's
//! production WriteContext-construction sites (per pim-2: not just a
//! struct field that no production caller populates).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use benten_caps::{CapError, CapabilityPolicy, ReadContext, WriteContext};
use benten_core::{Cid, Node, Value};
use benten_engine::Engine;

/// Recording policy that captures every `device_cid` value it observes
/// at `check_write` time. Permits all writes so the engine's commit
/// hook does not abort.
#[derive(Debug, Default)]
struct RecordingDevicePolicy {
    observed: Arc<Mutex<Vec<Option<Cid>>>>,
}

impl CapabilityPolicy for RecordingDevicePolicy {
    fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError> {
        self.observed.lock().unwrap().push(ctx.device_cid);
        Ok(())
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Ok(())
    }
}

/// G16-B-prime fp (mini-review consumer-audit extension): records
/// every `device_cid` observed at `check_read` time. Permits all reads.
#[derive(Debug, Default)]
struct RecordingDeviceReadPolicy {
    observed: Arc<Mutex<Vec<Option<Cid>>>>,
}

impl CapabilityPolicy for RecordingDeviceReadPolicy {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_read(&self, ctx: &ReadContext) -> Result<(), CapError> {
        self.observed.lock().unwrap().push(ctx.device_cid);
        Ok(())
    }
}

#[test]
fn capability_policy_per_device_cid_dispatch_observable_in_runtime_arm() {
    // cap-r4-4 (c) pin (pim-2 production-runtime-arm assertion).
    //
    // OBSERVABLE consequence: a policy registered on an engine that
    // has called `set_device_cid(Some(cid))` observes `device_cid:
    // Some(cid)` on every WriteContext at check_write time. Defends
    // against the failure shape where the field exists structurally
    // but no production write-path callsite populates it.
    let observed = Arc::new(Mutex::new(Vec::new()));
    let policy = RecordingDevicePolicy {
        observed: Arc::clone(&observed),
    };
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();

    // Configure a known device-CID on the engine.
    let device_cid = Cid::from_blake3_digest(*blake3::hash(b"device:desktop").as_bytes());
    engine.set_device_cid(Some(device_cid));
    assert_eq!(
        engine.device_cid(),
        Some(device_cid),
        "set_device_cid + device_cid round-trip"
    );

    // Drive a write through the production path (engine_crud.rs ->
    // engine_diagnostics.rs::transaction commit hook).
    let mut props: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let node = Node::new(vec!["post".into()], props);
    // Drive through Engine::transaction — the production write path
    // that exercises the engine_diagnostics.rs::transaction commit
    // hook where WriteContext.device_cid is populated.
    engine
        .transaction(|tx| {
            let _cid = tx.create_node(&node)?;
            Ok(())
        })
        .unwrap();

    // The recording policy observed at least one check_write call.
    let captures = observed.lock().unwrap().clone();
    assert!(
        !captures.is_empty(),
        "engine.create_node MUST drive the policy's check_write at \
         commit time per the post-G3-A surface contract"
    );
    // EVERY observed WriteContext carries Some(device_cid).
    assert!(
        captures.iter().all(|d| *d == Some(device_cid)),
        "production-runtime WriteContext MUST carry device_cid populated \
         from Engine::set_device_cid per cap-r4-4 (c). Captures: {captures:?}"
    );
}

#[test]
fn legacy_engine_without_device_cid_observes_none_in_writecontext() {
    // Backward-compat counter-pin: an engine that NEVER called
    // set_device_cid leaves WriteContext.device_cid == None per
    // cap-r4-4 (b). Defends against a regression where the threading
    // would synthesize a non-None placeholder.
    let observed = Arc::new(Mutex::new(Vec::new()));
    let policy = RecordingDevicePolicy {
        observed: Arc::clone(&observed),
    };
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();
    assert_eq!(engine.device_cid(), None, "default device_cid is None");

    let mut props: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let node = Node::new(vec!["post".into()], props);
    // Drive through Engine::transaction — the production write path
    // that exercises the engine_diagnostics.rs::transaction commit
    // hook where WriteContext.device_cid is populated.
    engine
        .transaction(|tx| {
            let _cid = tx.create_node(&node)?;
            Ok(())
        })
        .unwrap();

    let captures = observed.lock().unwrap().clone();
    assert!(
        !captures.is_empty(),
        "production-runtime arm fired check_write"
    );
    assert!(
        captures.iter().all(|d| d.is_none()),
        "legacy engine (no set_device_cid) MUST surface device_cid: None \
         per cap-r4-4 (b) backward-compat. Captures: {captures:?}"
    );
}

#[test]
fn capability_policy_per_device_cid_dispatch_observable_on_read_path() {
    // G16-B-prime fp consumer-audit closure (cor-1 / cap-g16bp-3
    // BLOCKER-equivalent): the device_cid threading on the READ side
    // mirrors the WRITE side per D-PHASE-3-25. Without this pin a
    // read-path consumer-audit gap would let the field exist on
    // ReadContext but NEVER get populated by the engine's primary
    // get_node read-gate ReadContext-construction site.
    let observed = Arc::new(Mutex::new(Vec::new()));
    let policy = RecordingDeviceReadPolicy {
        observed: Arc::clone(&observed),
    };
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();

    let device_cid = Cid::from_blake3_digest(*blake3::hash(b"device:laptop").as_bytes());
    engine.set_device_cid(Some(device_cid));

    // Drive a write (commit-hook check_write does not record on the
    // read-policy variant) then read it back through engine_crud.rs's
    // get_node — the load-bearing read-gate site.
    let mut props: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let node = Node::new(vec!["post".into()], props);
    let cid = engine.transaction(|tx| tx.create_node(&node)).unwrap();

    // Clear out any read observations from the create_node path so the
    // explicit read below is the load-bearing assertion.
    observed.lock().unwrap().clear();
    let _read = engine.get_node(&cid).unwrap();

    let captures = observed.lock().unwrap().clone();
    assert!(
        !captures.is_empty(),
        "engine.get_node MUST drive the policy's check_read at read-gate \
         time per the post-G16-B-prime read-path consumer-audit closure"
    );
    assert!(
        captures.iter().all(|d| *d == Some(device_cid)),
        "production-runtime ReadContext MUST carry device_cid populated \
         from Engine::set_device_cid per cor-1 / cap-g16bp-3. \
         Captures: {captures:?}"
    );
}
