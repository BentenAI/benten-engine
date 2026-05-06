//! G14-C wave-4b: durable module-bytes registry CID validation
//! (Compromise #17 closure; D-PHASE-3-12 + plan §3 G14-C).
//!
//! Pin sources (per r2-test-landscape.md §2.2 G14-C):
//!
//! - `tests/register_module_bytes_validates_cid_per_d_phase_3_12_resolution` — D-PHASE-3-12 (unit)
//! - `tests/module_bytes_durable_across_engine_restart` — plan §3 G14-C (integration)
//!
//! Per §3.6b pim-2 these tests drive the production
//! `Engine::register_module_bytes` entry point and assert observable
//! consequences: CID-mismatch rejects with a typed error;
//! engine-restart preserves registered bytes via the durable
//! `RedbBlobBackend` (`system:ModuleBytes` zone Nodes).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_engine::Engine;

#[test]
fn register_module_bytes_validates_cid_per_d_phase_3_12_resolution() {
    // D-PHASE-3-12: register_module_bytes recomputes BLAKE3 over the
    // bytes and rejects mismatch with a typed
    // `E_MODULE_BYTES_CID_MISMATCH` error.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let bytes: Vec<u8> = b"a fixture wasm-shaped blob (any bytes work for the cid pin)".to_vec();
    let actual_cid = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());

    // OK path: matched (cid, bytes) succeeds.
    engine.register_module_bytes(&actual_cid, &bytes).unwrap();

    // Mismatch path: CID computed from different bytes paired with
    // the wrong content fails with the typed error.
    let wrong_cid = Cid::from_blake3_digest(*blake3::hash(b"different content").as_bytes());
    let err = engine
        .register_module_bytes(&wrong_cid, &bytes)
        .expect_err("CID mismatch must reject (D-PHASE-3-12)");
    let code_str = err.error_code().as_str().to_string();
    assert_eq!(
        code_str, "E_MODULE_BYTES_CID_MISMATCH",
        "rejection MUST surface the typed E_MODULE_BYTES_CID_MISMATCH code per D-PHASE-3-12; \
         got: {code_str}"
    );
}

#[test]
fn module_bytes_durable_across_engine_restart() {
    // Compromise #17 closure end-to-end pin (§3.6b pim-2): registered
    // bytes survive engine restart via RedbBlobBackend's
    // `system:ModuleBytes` zone Nodes. fetch_module_bytes returns the
    // bytes after engine restart at the same store path.
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("benten.redb");

    let bytes: Vec<u8> = b"durable wasm module fixture for restart-survival pin".to_vec();
    let cid = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());

    {
        let engine = Engine::open(&store_path).unwrap();
        engine.register_module_bytes(&cid, &bytes).unwrap();
        assert_eq!(
            engine.fetch_module_bytes(&cid),
            Some(bytes.clone()),
            "module bytes MUST be readable from the engine that just registered them"
        );
        // engine drops; redb file flushes.
    }

    {
        // Re-open at the same path.
        let engine = Engine::open(&store_path).unwrap();
        let got = engine
            .fetch_module_bytes(&cid)
            .expect("Compromise #17: module bytes MUST persist across engine open/close");
        assert_eq!(
            got, bytes,
            "rehydrated bytes MUST match the originally-registered content"
        );
    }
}
