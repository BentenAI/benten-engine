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

/// FORBIDDEN host-fn names (architectural commitment per CLAUDE.md
/// baked-in #16 + phase-3-backlog §6.0 D-D). SANDBOX is compute-only;
/// mutation flows through engine WRITE primitive with cap pre-check
/// + IVM invalidation + attribution-frame recording, NEVER through
/// SANDBOX host-fns. A regression that registers any of these names
/// fires the pin below.
const FORBIDDEN_HOST_FN_NAMES: &[&str] = &[
    "kv:write",
    "kv:delete",
    "kv:append",
    "edge:create",
    "edge:delete",
    "edge:update",
    "transaction:begin",
    "transaction:commit",
    "transaction:abort",
];

/// Compute-only `behavior_kind` enumeration (parallel architectural-shape
/// pin per cag-r4-1 recommendation). Every host-fn entry's
/// `behavior_kind` value MUST be in this set. A regression that adds a
/// mutating-shaped behavior_kind fires this pin even if the registered
/// name avoids the FORBIDDEN list above.
const COMPUTE_ONLY_BEHAVIOR_KINDS: &[&str] = &[
    "time_monotonic_coarsened", // time host-fn
    "log_sink",                 // log host-fn
    "kv_read",                  // kv:read host-fn
    "random",                   // random host-fn (D-PHASE-3-11)
];

#[test]
fn sandbox_host_fn_surface_compute_only_no_kv_write_no_kv_delete_no_edge_mutating_per_baked_in_16()
{
    // r4-r1-wsa-2 + cag-r4-1 + r4b-wsa-1 GREEN pin. Wave-G16-B-C
    // un-ignores after G17-A2 (random host-fn wave) + G20-B (docs-
    // sweep wave) shipped — both named-rationale waves are MERGED
    // and the architectural commitment is honored at runtime
    // (host-functions.toml at HEAD has only 4 entries: time + log +
    // kv:read + random; default_host_fns() registers the same 4).
    //
    // Activates the FORBIDDEN-list runtime check the original
    // R4-FP pin specified, defending against silent-regression
    // (a future contributor adds kv:write to host-functions.toml
    // for an apparently-good reason and the existing tests don't
    // surface the violation). Per pim-2 §3.6b: an architectural-
    // absence pin defending an actively-honored commitment is the
    // worst possible state when ignored — looks like coverage,
    // gives zero coverage.
    //
    // Original R3-D narrative (preserved for traceability):
    //
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
    // Source 1: host-functions.toml (declarative source-of-truth at
    // workspace root). The TOML uses `[host_fn."<name>"]` table syntax
    // for names containing `:`; a FORBIDDEN entry would manifest as a
    // table header like `[host_fn."kv:write"]`.
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let toml_path = workspace_root.join("host-functions.toml");
    let toml_src = std::fs::read_to_string(&toml_path).unwrap_or_else(|e| {
        panic!(
            "read workspace-root host-functions.toml: {} (path={:?})",
            e, toml_path
        )
    });
    for forbidden in FORBIDDEN_HOST_FN_NAMES {
        let table_header = format!("[host_fn.\"{}\"]", forbidden);
        assert!(
            !toml_src.contains(&table_header),
            "host-functions.toml MUST NOT declare {forbidden} per CLAUDE.md baked-in #16 \
             + phase-3-backlog §6.0 D-D (SANDBOX is compute-only; mutation flows through \
             engine WRITE primitive with cap pre-check + IVM invalidation + attribution-\
             frame recording, NOT through SANDBOX host-fns). Found table header: \
             {table_header}"
        );
    }

    // Source 2: default_host_fns() runtime table (the consumer side).
    // The production registration path uses string-literal names
    // (`Box::new(TimeHostFn { name: "time", ... })` etc); a registration
    // adding "kv:write" would contain that exact string-literal in a
    // production code path. The pin allows the FORBIDDEN names to
    // appear in COMMENTS (the existing prose narrates the
    // architectural commitment) but rejects any string-literal
    // registration.
    let host_fns_rs = workspace_root
        .join("crates")
        .join("benten-eval")
        .join("src")
        .join("sandbox")
        .join("host_fns.rs");
    let host_fns_src = std::fs::read_to_string(&host_fns_rs).unwrap_or_else(|e| {
        panic!(
            "read crates/benten-eval/src/sandbox/host_fns.rs: {} (path={:?})",
            e, host_fns_rs
        )
    });
    for forbidden in FORBIDDEN_HOST_FN_NAMES {
        let name_literal = format!("\"{}\"", forbidden);
        assert_only_in_comments(&host_fns_src, &name_literal, forbidden);
    }

    // Source 3: per-host-fn `behavior_kind` enumeration is compute-only
    // (parallel architectural-shape pin per cag-r4-1 recommendation).
    // Walk the TOML's `behavior_kind = "..."` lines + assert each is in
    // COMPUTE_ONLY_BEHAVIOR_KINDS. A regression that adds a mutating-
    // shaped behavior_kind fires this even if the registered name
    // avoids the FORBIDDEN list above.
    let mut behavior_kinds_seen = Vec::new();
    for line in toml_src.lines() {
        let trimmed = line.trim();
        // The TOML line shape is `behavior_kind = "value"`. Strip
        // surrounding whitespace + comments + skip non-matching lines.
        let Some(rest) = trimmed.strip_prefix("behavior_kind") else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim();
        // Extract the quoted value.
        let Some(rest) = rest.strip_prefix('"') else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        let kind = &rest[..end];
        behavior_kinds_seen.push(kind.to_string());
        assert!(
            COMPUTE_ONLY_BEHAVIOR_KINDS.contains(&kind),
            "host-functions.toml declared behavior_kind = {kind:?} which is NOT in the \
             compute-only set per CLAUDE.md baked-in #16 + cag-r4-1 + phase-3-backlog \
             §6.0 D-D. Allowed kinds: {COMPUTE_ONLY_BEHAVIOR_KINDS:?}"
        );
    }
    // Sanity: at least the 4 known host-fns must have been observed
    // (defends against a future TOML refactor that renames the
    // `behavior_kind` field — the loop would silently match nothing
    // and the pin would pass vacuously).
    assert!(
        behavior_kinds_seen.len() >= 4,
        "expected at least 4 behavior_kind declarations in host-functions.toml \
         (one per host-fn: time + log + kv:read + random); observed {} — \
         defends against silent-pass when the TOML field is renamed",
        behavior_kinds_seen.len()
    );
}

/// Assert that every occurrence of `needle` in `src` is inside a Rust
/// comment (`//` or `/* ... */`). The test text uses this to allow
/// FORBIDDEN names to appear in narrative comments (where the
/// architectural commitment is documented) while rejecting any
/// string-literal use that would represent a real registration.
///
/// Implementation: scan line-by-line. For each line containing the
/// needle, find the FIRST occurrence and check that EITHER (a) a `//`
/// appears before it on the same line OR (b) the line lies inside a
/// `/* ... */` block. Block-comment tracking is maintained across
/// lines.
fn assert_only_in_comments(src: &str, needle: &str, forbidden_for_msg: &str) {
    let mut in_block_comment = false;
    for (line_idx, line) in src.lines().enumerate() {
        // Walk the line characterwise to track block-comment state +
        // detect a `//` line-comment-start. If the needle appears at a
        // position outside any comment surface, fail.
        let bytes = line.as_bytes();
        let mut i = 0;
        let mut line_comment_start: Option<usize> = None;
        while i + 1 < bytes.len() {
            let b0 = bytes[i];
            let b1 = bytes[i + 1];
            if !in_block_comment && b0 == b'/' && b1 == b'*' {
                in_block_comment = true;
                i += 2;
                continue;
            }
            if in_block_comment && b0 == b'*' && b1 == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            if !in_block_comment && b0 == b'/' && b1 == b'/' {
                line_comment_start = Some(i);
                break;
            }
            i += 1;
        }

        let mut search_from = 0;
        while let Some(pos_rel) = line[search_from..].find(needle) {
            let pos = search_from + pos_rel;
            // (a) `pos` lies inside a `/* ... */` block — recompute
            //     block-state up to `pos` for robustness when the
            //     needle appears on the same line that opens the
            //     block-comment.
            let inside_block_comment = block_state_up_to(line, pos, in_block_comment);
            // (b) `pos` lies after a `//` line-comment-start.
            let inside_line_comment = line_comment_start.is_some_and(|s| pos > s);
            assert!(
                inside_block_comment || inside_line_comment,
                "host_fns.rs has FORBIDDEN host-fn name {forbidden_for_msg:?} appearing \
                 OUTSIDE a comment at line {} col {} — this would indicate a real \
                 registration that violates CLAUDE.md baked-in #16. Line: {line:?}",
                line_idx + 1,
                pos + 1,
            );
            search_from = pos + needle.len();
            if search_from >= line.len() {
                break;
            }
        }
    }
}

/// Helper for `assert_only_in_comments`: compute the block-comment
/// state at byte-position `target` within `line`, given the state at
/// line start. Returns `true` iff `target` is inside a `/* ... */`
/// region.
fn block_state_up_to(line: &str, target: usize, mut in_block: bool) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() && i < target {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        if !in_block && b0 == b'/' && b1 == b'*' {
            in_block = true;
            i += 2;
            continue;
        }
        if in_block && b0 == b'*' && b1 == b'/' {
            in_block = false;
            i += 2;
            continue;
        }
        i += 1;
    }
    in_block
}
