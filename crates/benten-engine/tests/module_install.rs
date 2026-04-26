//! Phase 2b R4-FP B-3 — G10-B `Engine::install_module` happy-path tests.
//!
//! TDD red-phase. Pin sources:
//!   - `r1-security-auditor.json` D16 RESOLVED-FURTHER —
//!     `Engine::install_module(manifest, expected_cid: Cid)` REQUIRES
//!     the expected CID arg at compile time (NOT Optional). The
//!     compile-time requirement closes the lazy-developer footgun
//!     (`install_module(m, None)` shipping in production).
//!   - `r2-test-landscape.md` §1.8 — install-module unit pins:
//!     install_module_requires_cid_arg_at_compile_time,
//!     install_module_accepts_matching_cid,
//!     install_module_error_includes_manifest_summary,
//!     install_module_compute_cid_helper_round_trips.
//!   - `.addl/phase-2b/00-implementation-plan.md` §3.2 G10-B + §1
//!     exit criterion #4.
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owner G10-B. Security-class slice
//! (mismatch attack vectors) lives in `install_module_rejects_cid_mismatch.rs`
//! (R3-C ownership; do not duplicate here).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_engine::Engine::install_module(manifest, expected_cid: Cid) -> Result<Cid, EngineError>
//   benten_engine::module_manifest::ModuleManifest
//   benten_engine::testing::testing_compute_manifest_cid
//   benten_engine::testing::testing_make_minimal_manifest

#[test]
#[ignore = "Phase 2b G10-B pending — D16 install_module happy path with matching CID"]
fn install_module_accepts_matching_cid() {
    // D16 RESOLVED-FURTHER — when expected_cid matches the canonical
    // CID of the manifest, install_module must SUCCEED and return the
    // installed CID (so the caller can use the CID as a downstream
    // handle without recomputing).
    //
    // R5 G10-B wires:
    //   1. let m = testing_make_minimal_manifest("acme.posts");
    //   2. let cid = testing_compute_manifest_cid(&m);
    //   3. let installed = engine.install_module(m, cid).expect("happy path");
    //   4. ASSERT installed == cid.
    //   5. ASSERT a subsequent `engine.is_module_installed(cid)` returns true
    //      (or the equivalent introspection accessor G10-B exposes).
    todo!("R5 G10-B — assert install_module(m, cid_of_m) returns Ok(cid)");
}

#[test]
#[ignore = "Phase 2b G10-B pending — D16 REQUIRED expected_cid arg (compile-time)"]
fn install_module_requires_cid_arg_at_compile_time() {
    // D16 RESOLVED-FURTHER — the `expected_cid` parameter MUST be a
    // REQUIRED positional arg, not `Option<Cid>` and not a defaulted
    // builder method. The compile-time requirement is the design choice
    // that closes the lazy-developer footgun where an `install_module(m)`
    // overload would silently compute-and-trust the CID.
    //
    // This test is partly a code-shape pin and partly a documentation
    // anchor — if a future contributor adds a one-arg overload, removing
    // this test would be a deliberate decision they have to make in code
    // review, not an accident.
    //
    // R5 G10-B wires:
    //   1. The body simply calls install_module with both args (this
    //      asserts the signature compiles in the 2-arg shape).
    //   2. The non-existence of an `install_module(m)` overload is
    //      enforced by the compiler when the file compiles green.
    //   3. (Optional) trybuild compile-fail variant in
    //      `crates/benten-engine/tests/ui/install_module_no_overload.rs`
    //      asserting that `install_module(m)` does NOT compile.
    todo!("R5 G10-B — assert install_module(manifest, cid) is the only signature");
}

#[test]
#[ignore = "Phase 2b G10-B pending — D16 error includes manifest summary line"]
fn install_module_error_includes_manifest_summary() {
    // D16 RESOLVED-FURTHER — on CID mismatch the error MUST include a
    // 1-line manifest summary (provides-subgraphs name + module count +
    // requires-caps count) so the operator can identify *which* manifest
    // mis-installed without source-code spelunking.
    //
    // The dual-CID assertion lives in
    // `install_module_rejects_cid_mismatch.rs` (R3-C); this test
    // narrowly pins the SUMMARY-LINE shape so a refactor that drops the
    // summary surfaces here.
    //
    // R5 G10-B wires:
    //   1. Build manifest M with name "acme.posts".
    //   2. Compute wrong_cid via testing_make_distinct_dummy_cid().
    //   3. Call engine.install_module(M, wrong_cid); err = result.unwrap_err().
    //   4. ASSERT err.to_string().contains("acme.posts") (the manifest name).
    //   5. ASSERT err.to_string() carries module count + requires-caps count
    //      (e.g. "modules=1 caps=2" or similar canonical summary format
    //      pinned in docs/MODULE-MANIFEST.md per G10-B doc-drift).
    todo!("R5 G10-B — assert install_module CID-mismatch error carries summary line");
}

#[test]
#[ignore = "Phase 2b G10-B pending — testing_compute_manifest_cid round-trip"]
fn install_module_compute_cid_helper_round_trips() {
    // R2 §1.8 — the testing helper MUST compute the SAME CID that
    // install_module would compute internally. Otherwise the helper is a
    // lying oracle and the install-time pin becomes untestable.
    //
    // R5 G10-B wires:
    //   1. let m = testing_make_minimal_manifest("acme.posts");
    //   2. let cid_via_helper = testing_compute_manifest_cid(&m);
    //   3. let installed_cid = engine.install_module(m.clone(), cid_via_helper)
    //          .expect("helper-CID must match engine-CID");
    //   4. ASSERT installed_cid == cid_via_helper (the install path
    //      returns the SAME CID the helper computed).
    todo!("R5 G10-B — assert testing_compute_manifest_cid agrees with engine internal CID");
}
