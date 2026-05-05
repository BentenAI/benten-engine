//! R4-FP RED-PHASE pin — SANDBOX-as-compute-only architectural-absence
//! discipline (CLAUDE.md baked-in #16; r4-r1-wsa-2 MAJOR + cag-r4-1
//! MAJOR cross-corroboration).
//!
//! Pin sources:
//!
//! - r4-r1-wsa-2 (wasmtime-sandbox-auditor lens; MAJOR — architectural
//!   absence pin missing for kv:write / kv:delete / edge-mutating
//!   host-fns)
//! - r4-r1 cag-r4-1 (code-as-graph lens; MAJOR — Charter 10 SANDBOX-
//!   as-compute-only honored at host-fn surface; cross-corroboration
//!   of the same architectural commitment)
//! - CLAUDE.md baked-in #16 — SANDBOX is compute-only; engine-level
//!   mutation primitives (WRITE) sit OUTSIDE SANDBOX
//! - phase-3-backlog §6.0 D-D — kv:write / kv:delete / edge-mutating
//!   host-fns are NOT engine concerns; intentionally NOT implemented
//!
//! ## Why this pin exists
//!
//! Phase-2b shipped 3 host-fns: time, log, kv:read (all read-only or
//! compute-only). The architectural commitment per CLAUDE.md baked-in
//! #16 is that SANDBOX never gains storage-mutating host-fns —
//! mutation flows through the engine's WRITE primitive (with capability
//! pre-check + IVM invalidation + attribution-frame recording),
//! NEVER through SANDBOX guests calling host-fns that bypass these
//! controls.
//!
//! The risk surfaced by r4-r1-wsa-2: `crates/benten-eval/src/sandbox/host_fns.rs:169`
//! contains a prose comment about future kv:write addition. The
//! comment is HONEST documentation ("when Phase 3 extends the table
//! with kv:write"), but it proves the regression vector is live in
//! implementer intuition — a future contributor adds kv:write to
//! `host-functions.toml` for an apparently-good reason ("the app needs
//! to write a small index from the sandbox") and bypasses the
//! architectural commitment without any test surfacing the violation.
//!
//! This pin asserts the FORBIDDEN list is absent from BOTH:
//! 1. `host-functions.toml` (the source-of-truth declarations)
//! 2. `crates/benten-eval/src/sandbox/host_fns.rs::default_host_fns()`
//!    (the codegen-mirrored table consumed at runtime)
//!
//! ## Cross-corroboration with cag-r4-1
//!
//! Lens 12 (code-as-graph) flagged the same architectural-absence gap
//! via Charter 10 ("SANDBOX-as-compute-only honored at host-fn surface").
//! The two lenses converge on the regression vector. ONE pin satisfies
//! both findings — the architectural commitment is unitary, even if
//! two lenses surface it.
//!
//! ## What the FORBIDDEN list covers
//!
//! - `kv:write` / `kv:delete` / `kv:append` — durable mutation through
//!   the KV backend (proper path: WRITE primitive with cap pre-check)
//! - `edge:create` / `edge:delete` — graph-edge mutation (proper path:
//!   WRITE primitive against an Edge Node)
//! - `transaction:begin` / `transaction:commit` / `transaction:abort`
//!   — transaction primitives are engine surfaces, NEVER SANDBOX
//!   host-fns (transactions compose with WRITE; SANDBOX is
//!   compute-only)
//!
//! Per pim-2 §3.6b end-to-end pin: the test reads the runtime table +
//! drives the architectural assertion at the production decision
//! surface (the host-fn registration code path).

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: R4-FP architectural-absence pin per CLAUDE.md baked-in #16 + r4-r1-wsa-2 + cag-r4-1; un-ignored at G17-A2 (random host-fn wave) or G20-B (docs-sweep wave)"]
fn sandbox_host_fn_surface_compute_only_no_kv_write_no_kv_delete_no_edge_mutating_per_baked_in_16()
{
    // r4-r1-wsa-2 + cag-r4-1 architectural-absence pin. G17-A2 or G20-B
    // implementer wires this:
    //
    //   // FORBIDDEN host-fn names (architectural commitment per
    //   // CLAUDE.md baked-in #16 + phase-3-backlog §6.0 D-D):
    //   const FORBIDDEN_HOST_FN_NAMES: &[&str] = &[
    //       "kv:write",
    //       "kv:delete",
    //       "kv:append",
    //       "edge:create",
    //       "edge:delete",
    //       "edge:update",
    //       "transaction:begin",
    //       "transaction:commit",
    //       "transaction:abort",
    //   ];
    //
    //   // Source 1: host-functions.toml (declarative source-of-truth)
    //   let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..");
    //   let toml_path = workspace_root.join("host-functions.toml");
    //   let toml_src = std::fs::read_to_string(&toml_path).unwrap();
    //   for forbidden in FORBIDDEN_HOST_FN_NAMES {
    //       // The TOML uses `[host_fn."<name>"]` table syntax for names
    //       // containing `:`. A FORBIDDEN entry would manifest as:
    //       //   [host_fn."kv:write"]
    //       let table_header = format!("[host_fn.\"{}\"]", forbidden);
    //       assert!(!toml_src.contains(&table_header),
    //           "host-functions.toml MUST NOT declare {} per CLAUDE.md baked-in #16 \
    //            + phase-3-backlog §6.0 D-D (SANDBOX is compute-only; mutation flows \
    //            through engine WRITE primitive with cap pre-check + IVM invalidation + \
    //            attribution-frame recording, NOT through SANDBOX host-fns)",
    //           forbidden);
    //   }
    //
    //   // Source 2: default_host_fns() runtime table (the consumer side)
    //   let host_fns_rs = workspace_root.join("crates").join("benten-eval")
    //       .join("src").join("sandbox").join("host_fns.rs");
    //   let host_fns_src = std::fs::read_to_string(&host_fns_rs).unwrap();
    //
    //   // Walk default_host_fns() body — assert no FORBIDDEN names appear
    //   // as registered fn names (string literals registered in the table).
    //   // Heuristic source-cite (the production registration path uses
    //   // string-literal names; a registration adding "kv:write" would
    //   // contain that exact string):
    //   for forbidden in FORBIDDEN_HOST_FN_NAMES {
    //       let name_literal = format!("\"{}\"", forbidden);
    //       // Allow the name to appear in COMMENTS (the existing prose
    //       // at host_fns.rs:169 mentions kv:write in a comment about
    //       // future-Phase commitment); reject only if registered in a
    //       // production code path.
    //       //
    //       // Cleaner shape (implementer pins): parse the file via `syn`
    //       // and walk fn default_host_fns()'s body; assert no registration
    //       // call site uses a FORBIDDEN name.
    //       //
    //       // Simpler shape: assert the ONLY occurrences of the forbidden
    //       // name are in /* ... */ or // comments. The `syn` shape is
    //       // recommended for robustness.
    //       assert_only_in_comments(&host_fns_src, &name_literal, forbidden);
    //   }
    //
    //   // Source 3: per-host-fn `behavior_kind` enumeration is
    //   // compute-only (parallel architectural-shape pin per cag-r4-1
    //   // recommendation). Each registered host-fn MUST declare a
    //   // behavior_kind in the compute/read-only set:
    //   const COMPUTE_ONLY_BEHAVIOR_KINDS: &[&str] = &[
    //       "time_monotonic_coarsened",  // time host-fn
    //       "log_sink",                   // log host-fn
    //       "kv_read",                    // kv:read host-fn
    //       "csprng",                     // random host-fn (D-PHASE-3-11)
    //   ];
    //   // ... walk the TOML's behavior_kind values + assert each is in
    //   // COMPUTE_ONLY_BEHAVIOR_KINDS. Implementer pins exact form.
    //
    // OBSERVABLE consequence: a future PR that adds kv:write (or any
    // mutating host-fn) to either source fires this pin BEFORE the
    // PR is reviewable as "small additive change to host-fn table."
    // The PR author must EITHER (a) reopen the architectural decision
    // (overturn CLAUDE.md baked-in #16 with explicit ratification) OR
    // (b) reroute the mutation through engine WRITE primitive (the
    // architecturally-correct path).
    //
    // Defends:
    // - r4-r1-wsa-2 (wasmtime-sandbox-auditor lens MAJOR)
    // - cag-r4-1 (code-as-graph lens MAJOR; Charter 10 SANDBOX-compute-only)
    // - CLAUDE.md baked-in #16 architectural floor
    // - phase-3-backlog §6.0 D-D commitment
    //
    // Pairs with the existing positive-listing test at
    // `register_default_host_fns_matches_codegen_table.rs` (which
    // asserts the table HAS exactly time + log + kv:read + random)
    // and the drift-detector at `host_functions_doc_drift_against_toml.rs`
    // (which asserts TOML and .rs stay in sync). The triple together
    // pins the host-fn surface from 3 angles.
    unimplemented!(
        "R4-FP architectural-absence pin per CLAUDE.md baked-in #16: assert host-functions.toml + default_host_fns() exclude kv:write / kv:delete / edge-mutating / transaction:* host-fns"
    );
}
