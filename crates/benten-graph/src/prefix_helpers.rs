//! Prefix-bounded scan helpers shared across all `KVBackend` impls.
//!
//! Phase-1 placed [`next_prefix`] inside `redb_backend.rs` next to the
//! prefix-scan call site. Phase-2a's `InMemoryBackend` re-imported it;
//! Phase-3 G13-C's `BrowserBackend` does the same. Promoting the helper
//! to a small target-agnostic module keeps the wasm32-unknown-unknown
//! browser bundle compiling without pulling `redb_backend.rs` (which is
//! cfg-gated to non-wasm32-unknown-unknown per the `br-r1-1` BLOCKER pin
//! / CLAUDE.md baked-in #17).

/// Lexicographic successor of `prefix` — the smallest byte string strictly
/// greater than every string that begins with `prefix`. Used to turn a
/// prefix scan into a bounded range scan.
///
/// Returns `None` when `prefix` is all-`0xff` (no successor exists in the
/// byte-string ordering), signalling that the caller should do an
/// unbounded `prefix..` scan instead.
pub(crate) fn next_prefix(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut out = prefix.to_vec();
    while let Some(last) = out.last_mut() {
        if *last < 0xff {
            *last += 1;
            return Some(out);
        }
        out.pop();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_prefix_increments_and_trims() {
        assert_eq!(next_prefix(b"a"), Some(b"b".to_vec()));
        assert_eq!(next_prefix(b"az"), Some(b"a{".to_vec()));
        assert_eq!(next_prefix(&[0xff]), None);
        assert_eq!(next_prefix(&[0x01, 0xff, 0xff]), Some(vec![0x02]));
        assert_eq!(next_prefix(&[]), None);
    }
}
