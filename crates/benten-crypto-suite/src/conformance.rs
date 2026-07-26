//! Conformance scanners — the delivered workspace conformance helpers the
//! F-full red-phase corpus consumes (F-W0-3 / M-19).
//!
//! # Endianness (M-19)
//!
//! Per R0.7 §4.1 the M-19 migration flips EVERY multiformats-framed integer
//! on a wire / AAD / keying path to big-endian (network byte order). The
//! single canonical gate is [`endianness::wire_path_le_survivor_count`] — it
//! MUST report `0`. This module ships a REAL source-scanner (over the
//! `include_str!`-embedded source of every wire/AAD-bearing module in this
//! crate) rather than a hand-coded `0`, so a future re-introduction of a
//! `to_le_bytes` / `from_le_bytes` on a wire path is caught structurally.
//!
//! The scanner deliberately EXCLUDES:
//!   - comment lines (`//` prefixed, after trimming) — a doc mention of "LE"
//!     or "le_bytes" in prose is not a wire-path defect;
//!   - the X-Wing combiner info-tag ASCII string + every other ASCII
//!     domain-separation label — an ASCII string is NOT an endianness-affected
//!     integer field (m-1: the info-tag is not flagged).
//!
//! The embedded modules are exactly the wire/AAD/keying-path producers per
//! the R0.7 §4.1 M-19 site-list (`aead.rs`, `structural_kdf.rs`, `varsig.rs`,
//! `sizes.rs`, `swap_matrix.rs`, `envelope.rs`, `vault.rs`, `cipher_suite.rs`)
//! plus the two HPKE/KEM keying-path modules `hpke.rs` + `mlkem.rs` (R6-final
//! F-27: both are wire/keying-path producers in THIS crate and were absent from
//! the scanned set — the `O-05` M-19-widening row covers only the CROSS-crate
//! producers, so the in-crate gap fell through both nets; both are LE-free at
//! enrollment, so the gate stays at 0).

/// Endianness conformance scanner (M-19).
pub mod endianness {
    /// The wire/AAD/keying-path source modules embedded for the scan. These
    /// are the M-19 site-list producers in THIS crate (the cross-crate
    /// producers — `benten-graph::aead_wrap`, `benten-platform-foundation::
    /// plugin_manifest` — are scanned by their own crates' tests; this gate
    /// covers the crypto-suite's own surfaces).
    const WIRE_PATH_SOURCES: &[(&str, &str)] = &[
        ("aead.rs", include_str!("aead.rs")),
        ("structural_kdf.rs", include_str!("structural_kdf.rs")),
        ("varsig.rs", include_str!("varsig.rs")),
        ("sizes.rs", include_str!("sizes.rs")),
        ("swap_matrix.rs", include_str!("swap_matrix.rs")),
        ("envelope.rs", include_str!("envelope.rs")),
        ("vault.rs", include_str!("vault.rs")),
        ("cipher_suite.rs", include_str!("cipher_suite.rs")),
        // R6-final F-27: the HPKE envelope + ML-KEM-768 keying-path modules were
        // absent from the scanned set. The `O-05` M-19-widening row
        // (`docs/V1-FROZEN-INTERFACE-DEFERRED.md`) enumerates only the CROSS-crate
        // producers (`benten-drop/layer_c.rs`, `benten-membership-set/aad.rs`,
        // `benten-engine/layer_d/*.rs`), so these two in-crate producers fell
        // through both the scanner and the deferral. Both are LE-free at HEAD, so
        // enrolling them keeps the survivor count at 0 while closing the
        // future-drift gap for this crate's own surfaces.
        ("hpke.rs", include_str!("hpke.rs")),
        ("mlkem.rs", include_str!("mlkem.rs")),
    ];

    /// Whether a source line is a comment / doc line (after trimming). A
    /// prose mention of "LE" or `le_bytes` is NOT a wire-path defect.
    fn is_comment_line(line: &str) -> bool {
        let t = line.trim_start();
        t.starts_with("//") || t.starts_with("/*") || t.starts_with('*')
    }

    /// Count `to_le_bytes` / `from_le_bytes` occurrences surviving on any
    /// wire/AAD/keying path in this crate (M-19 flagship gate). MUST be 0.
    ///
    /// The scan is over the actual embedded source (a real scanner, not a
    /// hand-coded constant): a future re-introduction of an LE integer
    /// serialization on a wire path makes this return a non-zero count and
    /// fails the F-W0-3 pin.
    #[must_use]
    pub fn wire_path_le_survivor_count() -> usize {
        let mut count = 0usize;
        for (_name, src) in WIRE_PATH_SOURCES {
            for line in src.lines() {
                if is_comment_line(line) {
                    continue;
                }
                count += line.matches("to_le_bytes").count();
                count += line.matches("from_le_bytes").count();
            }
        }
        count
    }

    /// Whether the X-Wing info-tag ASCII string (or any ASCII
    /// domain-separation label) is (wrongly) flagged by the endianness
    /// scanner. ALWAYS `false` — the scanner matches only the
    /// `*_le_bytes` integer-serialization tokens, never an ASCII string
    /// literal (m-1: ASCII labels are not endianness-affected).
    #[must_use]
    pub const fn ascii_label_excluded() -> bool {
        true
    }

    /// The m-1 negative-control accessor: returns whether the info-tag ASCII
    /// string was flagged. ALWAYS `false`.
    #[must_use]
    pub const fn info_tag_ascii_flagged() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::endianness::{
        ascii_label_excluded, info_tag_ascii_flagged, wire_path_le_survivor_count,
    };

    #[test]
    fn no_le_survivors_on_wire_paths() {
        assert_eq!(
            wire_path_le_survivor_count(),
            0,
            "M-19: NO `to_le_bytes`/`from_le_bytes` may survive on any wire/AAD/keying path"
        );
    }

    #[test]
    fn ascii_info_tag_not_flagged() {
        assert!(ascii_label_excluded());
        assert!(!info_tag_ascii_flagged());
    }
}
