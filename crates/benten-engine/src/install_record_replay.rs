//! Phase-4-Meta-Core G-CORE-8 §4.37 — InstallRecord replay-defense +
//! atomic record-and-check around admission.
//!
//! # What this seam does
//!
//! An `InstallRecord` (in `benten_platform_foundation::plugin_manifest`)
//! is consumed exactly once. Presenting the same
//! canonical record bytes twice (matched by `signing_payload` hash —
//! the canonical content identity) is the replay-attack signal — the
//! second admission rejects with the typed
//! [`benten_errors::ErrorCode::PluginInstallRecordAlreadyApplied`]
//! code BEFORE any cap is minted (zero duplicate-mint window).
//!
//! # TOCTOU defense (R2 §5 "TOCTOU at replay-defense / fail-closed
//! flip")
//!
//! The check-and-record is **atomic** — single critical section
//! (compare-and-swap shape; no verify-then-record gap). Two parallel
//! presentations of the same record CID cannot both observe "not yet
//! applied" and both proceed. The atomic seam below uses a
//! [`std::sync::Mutex<HashSet<_>>`] which serializes
//! [`InstallRecordReplayStore::record_and_check`] across threads.
//!
//! # Couples to F3 durable replay-marker pattern
//!
//! The F3 durable replay-marker pattern (the `FrameReplayMarker`
//! `benten-caps` ships for sync-frame replay defense) is the existing
//! durable-KV grammar for the same problem class. The install-record
//! replay store here uses an in-memory `HashSet` as the v1 store; the
//! durable F3-pattern wire-up is a P-III follow-up (any storage
//! change to install records is canonical-bytes territory) — for v1
//! the in-memory shape is correct because the install pipeline ALWAYS
//! goes through Engine::install_plugin which is per-process.
//!
//! # Identity = signing_payload hash
//!
//! The "same record" predicate is by canonical-bytes hash, not by
//! field-set equality. Per `InstallRecord::signing_payload` semantics
//! (manifest_cid || timestamp || nonce || plugin_did_bytes), two
//! presentations that share the same `signing_payload` are identical
//! by user-intent; a fresh nonce (the standard regen path) produces
//! a distinct payload hash and is admitted as a fresh install.
//!
//! # G-CORE-8 deliverable scope
//!
//! Lands the [`InstallRecordReplayStore`] type + the
//! `record_and_check` atomic compare-and-swap. The actual wire-up
//! into `Engine::install_plugin` (Step 0/1 BEFORE Step 9 cap-cascade,
//! per the test's specific-arm contract) is the G-CORE-8.2 follow-up
//! WRITE-admission wave; this seam lands the store + the typed code
//! + the atomic primitive so the wire-up is a one-line consume.

use std::collections::HashSet;
use std::sync::Mutex;

use benten_errors::ErrorCode;

/// Atomic record-and-check store for install-record replay defense.
///
/// One instance per engine. The store is in-memory at v1 (per-process
/// lifetime); a durable F3-pattern follow-up will persist the set
/// across engine restarts (a Phase-4-Meta-Composing item — the
/// in-process atomic primitive is the load-bearing G-CORE-8 work).
///
/// # Identity
///
/// "Same record" = identical `signing_payload` hash. Two presentations
/// with the same payload hash represent the same user intent; a fresh
/// nonce regenerates the payload hash and admits as a distinct install.
///
/// # Atomicity
///
/// [`Self::record_and_check`] is the single atomic critical section
/// that combines "has this payload-hash been seen?" with "record this
/// payload-hash as seen" under a `Mutex`. No verify-then-record gap.
#[derive(Debug, Default)]
pub struct InstallRecordReplayStore {
    /// Set of consumed canonical-bytes hashes (32-byte BLAKE3 over the
    /// `signing_payload`). Stored as the hex string for `Eq+Hash`
    /// stability across the in-memory boundary (a `[u8; 32]` would
    /// work too — the choice is implementation detail).
    consumed: Mutex<HashSet<[u8; 32]>>,
}

impl InstallRecordReplayStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Atomic check-and-record. Returns `Ok(())` if the payload-hash
    /// has NOT been seen (and records it as seen in the same critical
    /// section); returns
    /// `Err(ErrorCode::PluginInstallRecordAlreadyApplied)` if the
    /// payload-hash WAS seen previously (replay-attack signal).
    ///
    /// # The atomic guarantee
    ///
    /// The single `Mutex` lock spans both the contains-check and the
    /// insert. Two parallel callers presenting the same payload-hash
    /// CANNOT both observe "not yet seen" — exactly one will see
    /// `Ok(())` + the consumed-set populated; the other will see
    /// `Err(AlreadyApplied)` + the consumed-set unchanged. This is
    /// the TOCTOU defense the wave-completion test pins specifically.
    ///
    /// # Errors
    ///
    /// [`ErrorCode::PluginInstallRecordAlreadyApplied`] if the
    /// payload-hash was previously recorded (replay).
    pub fn record_and_check(&self, signing_payload_hash: [u8; 32]) -> Result<(), ErrorCode> {
        let mut guard = self.consumed.lock().map_err(|_| ErrorCode::GraphInternal)?;
        if guard.contains(&signing_payload_hash) {
            // The record-and-check is fused — second observation
            // returns Err WITHOUT advancing state (no double-record).
            // The typed code is the named-now §4.37 replay reject.
            return Err(ErrorCode::PluginInstallRecordAlreadyApplied);
        }
        guard.insert(signing_payload_hash);
        Ok(())
    }

    /// Snapshot of consumed payload-hash count (test observable).
    #[must_use]
    pub fn consumed_count(&self) -> usize {
        self.consumed.lock().map_or(0, |g| g.len())
    }

    /// Whether the given payload-hash has been consumed (test
    /// observable; production callers use [`Self::record_and_check`]
    /// for the atomic compare-and-swap).
    #[must_use]
    pub fn contains(&self, signing_payload_hash: &[u8; 32]) -> bool {
        self.consumed
            .lock()
            .is_ok_and(|g| g.contains(signing_payload_hash))
    }
}

/// Compute the canonical identity hash for an InstallRecord's
/// signing-payload bytes. BLAKE3 over the payload bytes; matches the
/// `signing_payload` semantics in
/// `benten_platform_foundation::plugin_manifest::InstallRecord`.
///
/// Callers compose: `let hash = signing_payload_hash(
/// install_record.signing_payload());` then
/// `store.record_and_check(hash)`. The two-step composition is
/// deliberate (the hash is computed by the install-pipeline caller
/// who has the InstallRecord; this module stays free of platform-
/// foundation deps per the dep-direction discipline).
#[must_use]
pub fn signing_payload_hash(payload: &[u8]) -> [u8; 32] {
    *blake3::hash(payload).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_record_admitted() {
        let store = InstallRecordReplayStore::new();
        let hash = signing_payload_hash(b"some-canonical-bytes");
        assert!(store.record_and_check(hash).is_ok());
        assert_eq!(store.consumed_count(), 1);
        assert!(store.contains(&hash));
    }

    #[test]
    fn replay_rejected_with_typed_code() {
        let store = InstallRecordReplayStore::new();
        let hash = signing_payload_hash(b"some-canonical-bytes");
        store.record_and_check(hash).unwrap();
        let err = store
            .record_and_check(hash)
            .expect_err("replay must reject");
        assert_eq!(err, ErrorCode::PluginInstallRecordAlreadyApplied);
        // Second observation does NOT advance state.
        assert_eq!(store.consumed_count(), 1);
    }

    #[test]
    fn distinct_payloads_admitted_independently() {
        let store = InstallRecordReplayStore::new();
        let h1 = signing_payload_hash(b"payload-1");
        let h2 = signing_payload_hash(b"payload-2");
        assert!(store.record_and_check(h1).is_ok());
        assert!(store.record_and_check(h2).is_ok());
        assert_eq!(store.consumed_count(), 2);
    }

    #[test]
    fn parallel_presentation_serialized_one_admits_one_rejects() {
        // Models the TOCTOU defense: two parallel admissions of the
        // same payload-hash. The Mutex serializes them; exactly one
        // sees Ok, the other sees Err. No verify-then-record window.
        use std::sync::Arc;
        use std::thread;
        let store = Arc::new(InstallRecordReplayStore::new());
        let hash = signing_payload_hash(b"shared-payload");
        let handles: Vec<_> = (0..16)
            .map(|_| {
                let s = Arc::clone(&store);
                thread::spawn(move || s.record_and_check(hash))
            })
            .collect();
        let mut ok_count = 0;
        let mut err_count = 0;
        for h in handles {
            match h.join().unwrap() {
                Ok(()) => ok_count += 1,
                Err(ErrorCode::PluginInstallRecordAlreadyApplied) => err_count += 1,
                Err(other) => panic!("unexpected error: {other:?}"),
            }
        }
        assert_eq!(ok_count, 1, "exactly one parallel caller admits");
        assert_eq!(err_count, 15, "all other parallel callers reject");
        assert_eq!(store.consumed_count(), 1);
    }
}
