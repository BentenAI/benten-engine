//! E-02 companion — the M-19 LE-survivor scanner for THIS crate's wire path.
//!
//! # Why this file exists
//!
//! `benten-crypto-suite/src/conformance.rs` (`WIRE_PATH_SOURCES`) documented
//! that the CROSS-crate M-19 producers — `benten-graph::aead_wrap` and
//! `benten-platform-foundation::plugin_manifest` — "are scanned by their own
//! crates' tests". For this crate that claim was FALSE: no endianness scanner
//! existed here at all, which is part of why flipping all five `to_be_bytes()`
//! calls in `InstallRecord::signing_payload()` to `to_le_bytes()` left the
//! crate 227/227 PASS. This file makes the claim true.
//!
//! # Where this sits in the defense
//!
//! Third net, and the weakest of the three — it is a SOURCE-TEXT scan, so it
//! catches the literal `to_le_bytes` / `from_le_bytes` re-introduction and
//! nothing subtler (a hand-rolled byte-reversal would slip past it). The
//! load-bearing pins are the golden and the per-field BE guards in
//! `canonical_bytes_v1_install_record_signing_preimage.rs`; this scanner is
//! defense-in-depth that fires on the most likely regression shape and, unlike
//! the goldens, covers modules that have no byte-pin of their own.
//!
//! Enrolled modules are the install/consent path. All three are LE-free at
//! HEAD (verified by grep at authoring time: the only integer-serialization
//! sites in this crate's `src/` are the five `to_be_bytes()` calls in
//! `plugin_manifest.rs`), so enrolment starts and must stay at 0 — the same
//! discipline as the R6-final F-27 `hpke.rs`/`mlkem.rs` enrolment in
//! `benten-crypto-suite`.

/// Install/consent-path source modules scanned for surviving LE integer
/// serialization. `plugin_manifest.rs` is the module named in the R0.7 §4.1
/// M-19 site-list; `plugin_lifecycle.rs` and `manifest_store.rs` are the
/// adjacent install/persist path and are enrolled pre-emptively so a future
/// wire field added there cannot land LE unnoticed.
const WIRE_PATH_SOURCES: &[(&str, &str)] = &[
    (
        "plugin_manifest.rs",
        include_str!("../src/plugin_manifest.rs"),
    ),
    (
        "plugin_lifecycle.rs",
        include_str!("../src/plugin_lifecycle.rs"),
    ),
    (
        "manifest_store.rs",
        include_str!("../src/manifest_store.rs"),
    ),
];

/// Whether a source line is a comment / doc line (after trimming). A prose
/// mention of "LE" or `le_bytes` is NOT a wire-path defect. Mirrors
/// `benten_crypto_suite::conformance::endianness::is_comment_line`.
fn is_comment_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("/*") || t.starts_with('*')
}

/// Count `to_le_bytes` / `from_le_bytes` surviving on this crate's wire path.
fn wire_path_le_survivor_count() -> Vec<(&'static str, usize)> {
    let mut hits = Vec::new();
    for (name, src) in WIRE_PATH_SOURCES {
        let mut count = 0usize;
        for line in src.lines() {
            if is_comment_line(line) {
                continue;
            }
            count += line.matches("to_le_bytes").count();
            count += line.matches("from_le_bytes").count();
        }
        if count > 0 {
            hits.push((*name, count));
        }
    }
    hits
}

/// M-19: no LE integer serialization may survive on this crate's wire path.
///
/// MUTATION THAT MUST FAIL THIS PIN: change any `to_be_bytes()` to
/// `to_le_bytes()` in `src/plugin_manifest.rs` (lines 631/637/640/646/649) —
/// the survivor count goes to 1 and this fails, naming the module.
#[test]
fn no_le_survivors_on_platform_foundation_wire_paths() {
    let hits = wire_path_le_survivor_count();
    assert!(
        hits.is_empty(),
        "M-19: `to_le_bytes`/`from_le_bytes` survives on a wire/signing path in \
         benten-platform-foundation: {hits:?}. The install-record signing \
         pre-image is FROZEN big-endian; an LE field there breaks every \
         previously-signed InstallRecord across engine builds."
    );
}

/// Negative control: the scanner must actually be looking at real source, not
/// at an empty embed. Without this, a broken `include_str!` path (or a module
/// rename) would silently turn the gate above into a tautology that passes
/// forever — the exact "a passing test proves nothing" shape E-02 closes.
///
/// MUTATION THAT MUST FAIL THIS PIN: point any `include_str!` above at a file
/// with no `to_be_bytes` (or delete the five BE calls from `signing_payload()`).
#[test]
fn scanner_is_reading_real_source_not_an_empty_embed() {
    let manifest_src = WIRE_PATH_SOURCES
        .iter()
        .find(|(name, _)| *name == "plugin_manifest.rs")
        .expect("plugin_manifest.rs must stay enrolled in the scan")
        .1;

    // The five BE sites in `signing_payload()` (timestamp, len(nonce),
    // len(plugin_did), cap_count, per-cap len). If the scanner can see the
    // source at all, it can see these.
    let be_sites = manifest_src
        .lines()
        .filter(|l| !is_comment_line(l))
        .map(|l| l.matches("to_be_bytes").count())
        .sum::<usize>();
    assert!(
        be_sites >= 5,
        "M-19 scanner sanity: expected at least the 5 big-endian \
         serialization sites in InstallRecord::signing_payload(), saw \
         {be_sites}. Either the embed path is stale or the frozen pre-image \
         lost a length prefix."
    );
}
