//! TF-12 / §4.69 — cap-mutation surface no-regression FREEZE-invariant
//! (G-CORE-9 FREEZE wave).
//!
//! ADDL R3-C1 (Phase-4-Meta-Core; last R3 wave; freeze-time). TDD —
//! but this is a **GREEN verify-stays-regression guard**, NOT a RED
//! pin (the invariant is ALREADY SHIPPED at HEAD; the freeze must not
//! let it regress). Tests-only — NO production source.
//!
//! ## §3.6g LITERAL discipline checklist (reproduced, not §-referenced)
//!
//!  1. Land-when = FREEZE, but **GREEN now** (NOT `#[ignore]`'d): the
//!     §4.69 organizing principle converged to option (a)
//!     `EngineCapsHandle`-canonical and is ALREADY SHIPPED — so this is
//!     a no-regression freeze-invariant guard, not a deliverable pin.
//!  2. Campaign-tail landed-vs-RED split (§3.5n): orchestrator
//!     ground-truthed origin/main `ed03729a` (2026-05-19): ZERO
//!     cap-mutation methods on `Engine`; all on `EngineCapsHandle`
//!     (`engine_caps.rs`); `Engine::caps()` at `engine.rs:1628`. R3-C1
//!     INDEPENDENTLY re-verified the method sets before writing (see
//!     the in-test source-scan asserting both halves). → GREEN guard.
//!  3. SHAPE-not-SUBSTANCE (pim-18 / §3.6f): this drives the REAL
//!     production source — it scans the actual `engine.rs` /
//!     `engine_caps.rs` impl blocks and asserts (a) the cap-mutation
//!     method *names* live on `EngineCapsHandle`, (b) NONE regress onto
//!     a `pub` `impl Engine` method, (c) `Engine::caps()` is the access
//!     path. A behavioral test would be WRONG here: the property is a
//!     SURFACE-ORGANIZATION invariant (which type owns the method),
//!     not a runtime behavior — the would-FAIL signal is "a
//!     cap-mutation `pub fn` appears in an `impl Engine` block", a
//!     concrete source fact, exactly like R3-B6's #838 seam-shape
//!     reasoning.
//!  4. pim-2 sub-rule-4 (§3.6b): asserts the SPECIFIC §4.69 invariant
//!     (the enumerated 9 cap-mutation methods stay off `Engine`), not
//!     an umbrella "caps API is fine".
//!  5. §3.13: no shared process-scoped static — per-test locals only.
//!  6. §3.5j: compiles + MSRV-1.95 clippy AND `cargo +stable clippy`
//!     (scoped to benten-engine — never `--workspace`).
//!  7. §3.6e: introduces no stranded `#[ignore]` pin.
//!
//! ## Why GREEN-not-RED (the §3.5n ground-truth that gates this file)
//!
//! r2-test-landscape.md TF-12 obligation (3) is EXPLICIT: §4.69 is
//! "ALREADY SHIPPED at HEAD ... Therefore this arm is a GREEN
//! verify-stays-regression guard, NOT a RED-ignored pin." The plan
//! §0 Freeze-completeness cluster (a) + §1.A.FROZEN item 1 concur:
//! option (a) `EngineCapsHandle`-canonical already shipped;
//! orchestrator-mechanical, NOT a Ben decision-point. R3-C1's
//! independent re-verify (the source scan below) confirms it.
//!
//! Pin source: r2-test-landscape.md TF-12 obligation (3) + §2.B
//! "§4.69 cap-mutation no-regression freeze-invariant" + plan
//! §1.A.FROZEN item 1 + §0 Freeze-completeness cluster (a).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

fn engine_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// The 9 cap-mutation methods §4.69 requires to live on
/// `EngineCapsHandle` (orchestrator §3.5n ground-truth + R3-C1
/// independent re-verify at `engine_caps.rs`, origin/main ed03729a).
const CAP_MUTATION_METHODS: &[&str] = &[
    "install_proof",
    "revoke",
    "create_principal",
    "grant_capability",
    "grant_capability_with_proof",
    "revoke_capability",
    "revoke_capability_by_grant_cid",
    "install_ucan_proof",
    "delegate_capability",
];

/// GREEN guard — the freeze must NOT regress any cap-mutation method
/// onto a `pub` `Engine` method. We scan EVERY `crates/benten-engine/
/// src/*.rs` for a `pub (async )?fn <method>` declaration that is NOT
/// inside `engine_caps.rs` (the canonical handle home). A regression
/// (a cap-mutation `pub fn` re-appearing on `Engine` directly) FAILS
/// this test — the exact §4.69 freeze-invariant.
#[test]
fn cap_mutation_methods_do_not_regress_onto_engine_pub_surface() {
    let src = engine_src();
    let mut offenders: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(&src).expect("benten-engine/src must be readable");
    for ent in entries {
        let path = ent.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let fname = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        // engine_caps.rs is the CANONICAL home — methods there are the
        // invariant being preserved, not a regression.
        if fname == "engine_caps.rs" {
            continue;
        }
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for m in CAP_MUTATION_METHODS {
            let needle_plain = format!("pub fn {m}(");
            let needle_plain_g = format!("pub fn {m}<");
            let needle_async = format!("pub async fn {m}(");
            let needle_async_g = format!("pub async fn {m}<");
            for (lineno, line) in body.lines().enumerate() {
                let t = line.trim_start();
                if t.starts_with(&needle_plain)
                    || t.starts_with(&needle_plain_g)
                    || t.starts_with(&needle_async)
                    || t.starts_with(&needle_async_g)
                {
                    offenders.push(format!(
                        "REGRESSION: cap-mutation `{m}` re-appeared as a \
                         `pub fn` in {fname}:{} (must stay on \
                         EngineCapsHandle per §4.69)",
                        lineno + 1
                    ));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "TF-12 §4.69 no-regression FREEZE-invariant VIOLATED — \
         cap-mutation method(s) regressed onto the Engine pub surface \
         (the freeze must keep all 9 on EngineCapsHandle):\n{}",
        offenders.join("\n")
    );
}

/// GREEN guard — the canonical home `engine_caps.rs` actually declares
/// all 9 cap-mutation methods (R3-C1 independent re-verify of the
/// §3.5n ground-truth: the invariant is "all 9 ON the handle", so the
/// no-regression guard also asserts the positive half — they did not
/// silently disappear / move elsewhere either).
#[test]
fn all_cap_mutation_methods_present_on_caps_handle() {
    let caps = engine_src().join("engine_caps.rs");
    let body = std::fs::read_to_string(&caps)
        .expect("engine_caps.rs must exist (the §4.69 canonical home)");
    let mut missing: Vec<&str> = Vec::new();
    for m in CAP_MUTATION_METHODS {
        let present = body.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with(&format!("pub fn {m}("))
                || t.starts_with(&format!("pub fn {m}<"))
                || t.starts_with(&format!("pub async fn {m}("))
                || t.starts_with(&format!("pub async fn {m}<"))
        });
        if !present {
            missing.push(m);
        }
    }
    assert!(
        missing.is_empty(),
        "TF-12 §4.69: cap-mutation method(s) absent from the canonical \
         EngineCapsHandle home (engine_caps.rs) — the invariant is \
         'all 9 ON the handle': {missing:?}"
    );
}

/// GREEN guard — `Engine::caps()` is the access path (the §4.69
/// surface-organization invariant: callers reach cap-mutation via
/// `.caps()`, not direct `Engine` methods). Would-FAIL if the access
/// seam is removed/renamed by the freeze wave (which would silently
/// break the napi-side `.caps()` cascade rebind co-scheduled at
/// G-CORE-9).
#[test]
fn engine_caps_accessor_seam_present() {
    let engine = engine_src().join("engine.rs");
    let body = std::fs::read_to_string(&engine).expect("engine.rs must exist");
    let has_caps_accessor = body.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with("pub fn caps(") || t.starts_with("pub fn caps<")
    });
    assert!(
        has_caps_accessor,
        "TF-12 §4.69: `Engine::caps()` accessor seam absent from \
         engine.rs — the freeze must preserve the `.caps()` access path \
         (the napi-side cascade rebinds through it in the same atomic \
         G-CORE-9 wave)."
    );
}
