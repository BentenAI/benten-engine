//! Phase-4-Foundation G24-D-FP-1 — `ManifestStore` durable surface with
//! verify-on-every-load defense.
//!
//! Per threat-model §T5a + defense step 1 ("Install record verified on
//! EVERY load, not just at install — (i) at engine boot, (ii) at
//! per-plugin load on first access, (iii) at per-Atrium-merge
//! boundary"): an attacker that mutates install-record bytes
//! post-install (writes to manifest store; restarts engine) MUST be
//! rejected at next load. New install record with widened `requires`
//! consent that the user never re-consented to MUST surface as
//! `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` /
//! `E_PLUGIN_MANIFEST_INVALID`.
//!
//! Production code persists the in-memory shape via redb (parallel to
//! the `PluginLibrary` durable half). At G24-D-FP-1 wave the
//! in-memory shape is canonical; the redb backing is a follow-on
//! integration (no new ErrorCode required).
//!
//! Couples to `docs/future/phase-4-backlog.md §4.11`.

use crate::plugin_manifest::InstallRecord;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use std::collections::HashMap;

/// User-notification capture (defense-in-depth: T5a surfaces drift to
/// the user rather than auto-quarantining).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftNotification {
    /// The affected plugin-DID.
    pub plugin_did: Did,
    /// Reason payload (typed-error string form).
    pub reason: String,
}

impl DriftNotification {
    /// Whether this notification represents an install-record drift
    /// warning for `plugin_did` (test observable).
    #[must_use]
    pub fn is_install_record_drift_warning(&self, plugin_did: &Did) -> bool {
        self.plugin_did == *plugin_did
    }
}

/// Durable manifest store with verify-on-every-load defense.
#[derive(Debug, Default)]
pub struct ManifestStore {
    /// Plugin-DID → stored install-record bytes (DAG-CBOR encoded).
    /// Storing the raw bytes (not the decoded struct) lets the
    /// verify-on-load path detect byte-mutation between install and
    /// next load — a decode-then-re-encode round-trip would smooth
    /// over the attack.
    records: HashMap<Did, Vec<u8>>,
    /// Captured notifications for the test surface (production swaps
    /// in a user-notification sink).
    notifications: Vec<DriftNotification>,
}

impl ManifestStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Persist a verified install record. Caller MUST have already
    /// verified the user-DID signature; this method only stores the
    /// canonical bytes.
    ///
    /// # Errors
    ///
    /// `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` if the record
    /// fails self-verification at install time (defense in depth).
    pub fn install_plugin(
        &mut self,
        plugin_did: Did,
        record: InstallRecord,
    ) -> Result<(), ErrorCode> {
        // Verify at install time — if a caller tries to persist an
        // already-invalid record we reject here. This is the
        // first-of-three verify points (install / load / merge).
        record.verify_user_signature()?;
        let bytes =
            serde_ipld_dagcbor::to_vec(&record).map_err(|_| ErrorCode::PluginManifestInvalid)?;
        self.records.insert(plugin_did, bytes);
        Ok(())
    }

    /// Re-verify and load the install record for `plugin_did`.
    ///
    /// T5a LOAD-BEARING defense: this is the second-of-three verify
    /// points (install / load / merge). A post-install byte mutation
    /// surfaces here as either
    /// `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` (signature no
    /// longer verifies over mutated bytes) or
    /// `E_PLUGIN_MANIFEST_INVALID` (bytes no longer decode to a valid
    /// `InstallRecord`). User notification is captured for the UI
    /// surface to display.
    ///
    /// # Errors
    ///
    /// - `E_PLUGIN_MANIFEST_INVALID` if the plugin-DID is unknown or
    ///   the stored bytes fail to decode.
    /// - `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID` if the
    ///   user-DID signature fails to verify over the loaded bytes.
    pub fn load_verified(&mut self, plugin_did: &Did) -> Result<InstallRecord, ErrorCode> {
        let bytes = self
            .records
            .get(plugin_did)
            .ok_or(ErrorCode::PluginManifestInvalid)?
            .clone();
        let record: InstallRecord = match serde_ipld_dagcbor::from_slice(&bytes) {
            Ok(r) => r,
            Err(_) => {
                self.notifications.push(DriftNotification {
                    plugin_did: plugin_did.clone(),
                    reason: "E_PLUGIN_MANIFEST_INVALID".to_string(),
                });
                return Err(ErrorCode::PluginManifestInvalid);
            }
        };
        if let Err(e) = record.verify_user_signature() {
            self.notifications.push(DriftNotification {
                plugin_did: plugin_did.clone(),
                reason: "E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID".to_string(),
            });
            return Err(e);
        }
        Ok(record)
    }

    /// Captured user notifications (test observable).
    #[must_use]
    pub fn captured_user_notifications(&self) -> &[DriftNotification] {
        &self.notifications
    }

    /// Simulate a post-install byte-mutation attack: replace the
    /// stored bytes for `plugin_did` with the canonical encoding of
    /// `mutated_record`. The bytes are written directly without
    /// re-verifying — this models a file-system attack where an
    /// attacker swaps the install record without holding the
    /// user-DID secret key. The test surface then calls
    /// `load_verified` and asserts the drift is detected.
    ///
    /// # Errors
    ///
    /// `E_PLUGIN_MANIFEST_INVALID` on encode failure (programmer
    /// error in the test fixture).
    pub fn simulate_byte_mutation_attack(
        &mut self,
        plugin_did: Did,
        mutated_record: InstallRecord,
    ) -> Result<(), ErrorCode> {
        let bytes = serde_ipld_dagcbor::to_vec(&mutated_record)
            .map_err(|_| ErrorCode::PluginManifestInvalid)?;
        self.records.insert(plugin_did, bytes);
        Ok(())
    }

    /// Whether a record exists for `plugin_did` (test observable;
    /// production callers typically prefer `load_verified`).
    #[must_use]
    pub fn contains(&self, plugin_did: &Did) -> bool {
        self.records.contains_key(plugin_did)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_manifest::InstallRecord;
    use benten_core::Cid;
    use benten_id::keypair::Keypair;

    fn signed_record(
        user: &Keypair,
        plugin_did: Did,
        manifest_cid: Cid,
        nonce: Vec<u8>,
    ) -> InstallRecord {
        let mut record = InstallRecord {
            manifest_cid,
            plugin_did,
            consenting_user_did: user.public_key().to_did(),
            user_signature: Vec::new(),
            timestamp_stub_nanos: 1_700_000_000_000_000_000,
            nonce,
            granted_caps_bytes: vec![],
        };
        let sig = user.sign(&record.signing_payload());
        record.user_signature = sig.to_bytes().to_vec();
        record
    }

    #[test]
    fn install_then_load_verified_round_trips() {
        let user = Keypair::generate();
        let plugin_did = Did::from_string_unchecked("did:key:zPlugin".to_string());
        let record = signed_record(
            &user,
            plugin_did.clone(),
            Cid::from_blake3_digest([1u8; 32]),
            vec![0xABu8; 16],
        );
        let mut store = ManifestStore::new();
        store
            .install_plugin(plugin_did.clone(), record.clone())
            .unwrap();
        let loaded = store.load_verified(&plugin_did).unwrap();
        assert_eq!(loaded.consenting_user_did, user.public_key().to_did());
        assert!(store.captured_user_notifications().is_empty());
    }

    #[test]
    fn load_verified_rejects_post_install_byte_mutation_with_drift_notification() {
        let user = Keypair::generate();
        let plugin_did = Did::from_string_unchecked("did:key:zPlugin".to_string());
        let original = signed_record(
            &user,
            plugin_did.clone(),
            Cid::from_blake3_digest([1u8; 32]),
            vec![0xABu8; 16],
        );
        let mut store = ManifestStore::new();
        store
            .install_plugin(plugin_did.clone(), original.clone())
            .unwrap();
        // Attacker mutates the install-record bytes: same user-DID,
        // but a different nonce → user_signature no longer verifies
        // over the mutated bytes.
        let mut mutated = original.clone();
        mutated.nonce = vec![0xFFu8; 16];
        // Important: the attacker does NOT have the user's secret
        // key — leave the OLD signature in place (it was bound to
        // the original nonce).
        store
            .simulate_byte_mutation_attack(plugin_did.clone(), mutated)
            .unwrap();
        let err = store.load_verified(&plugin_did).unwrap_err();
        assert!(matches!(
            err,
            ErrorCode::PluginInstallRecordUserSignatureInvalid
        ));
        let notifications = store.captured_user_notifications();
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].is_install_record_drift_warning(&plugin_did));
    }
}
