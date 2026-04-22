//! R3 unit test for phil-r1-2 / §9.14 arch-1 dep-break signature-level gate:
//! `PrimitiveHost` trait signatures + `EvalError` variants MUST NOT reference
//! any `benten_graph::*` type.
//!
//! Implementation: greps `crates/benten-eval/src/host.rs` and
//! `crates/benten-eval/src/lib.rs` for any `benten_graph::` path fragment.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.1 + §9.14).

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

fn eval_src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read(relative: &str) -> String {
    let path = eval_src_dir().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn arch_1_no_graph_types_in_primitive_host() {
    let host_src = read("host.rs");
    let lib_src = read("lib.rs");

    // `benten_graph::` must not appear anywhere in the trait surface or
    // EvalError definitions. Comments and doc-strings are also checked —
    // they're a drift-early-warning if someone's documenting graph types
    // on the thin surface (a code-smell precursor to a use).
    let host_offenders: Vec<(usize, &str)> = host_src
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("benten_graph::"))
        .collect();
    assert!(
        host_offenders.is_empty(),
        "arch-1 signature-level gate: host.rs references benten_graph::: {host_offenders:?}"
    );

    // In lib.rs, restrict the search to the EvalError region (every line
    // containing the token `EvalError` plus surrounding context is checked
    // via a whole-file scan; false positives only on genuine occurrences).
    let lib_offenders: Vec<(usize, &str)> = lib_src
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("benten_graph::"))
        .collect();
    assert!(
        lib_offenders.is_empty(),
        "arch-1 signature-level gate: lib.rs references benten_graph::: {lib_offenders:?}"
    );
}
