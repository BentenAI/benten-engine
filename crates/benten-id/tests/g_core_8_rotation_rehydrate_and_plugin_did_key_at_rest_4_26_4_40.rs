//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 (benten-id lane) — §4.26
//! RotationLog rehydrate + resolve_did_for_cid round-trip + §4.40
//! engine-held plugin-DID private-key at-rest (through #1301), and the
//! §4.27 plugin_did RNG-provenance VERIFY-STAYS reconciliation.
//!
//! ## The #1294 / already-landed vs G-CORE-8-RED split (benten-id)
//!
//! VERIFY-STAYS-REGRESSION (NOT RED — already landed, un-ignored at
//! R6-FP-BF; G-CORE-8 must NOT regress):
//!   - `plugin_did_install_uses_os_rng_not_seed_derivation.rs`
//!     (`plugin_did::mint` uses OsRng, two mints distinct) — SHIPPED.
//!   - `plugin_did_install_no_hkdf_from_user_did_grep_assert.rs`
//!     (negative grep-assert: no HKDF/derive_from in plugin_did.rs) —
//!     SHIPPED.
//!   These §4.27 grep-pins are already non-`#[ignore]`d at HEAD; this
//!   file's §4.27 arm is a thin VERIFY-STAYS guard that the OsRng
//!   provenance property remains true through G-CORE-8.
//!
//! STILL-RED (G-CORE-8 staged-pins; the two existing files
//! `rotation_log_rehydrated_at_engine_open.rs` +
//! `resolve_did_for_cid_round_trip.rs` are DESTINATION-REMAPPED
//! `#[ignore]`d shells whose §4.26 named destination is THIS G-CORE-8
//! wave — §3.6e redirect obligation: their ignore messages already
//! cite §4.26; G-CORE-8 wires the substantive surface + un-ignores
//! them; reviewer verifies LANDING status):
//!   - §4.26 RotationLog rehydrate at engine-open + resolve_did_for_cid
//!     round-trip.
//!   - §4.40 engine-held plugin-DID private-key at-rest encryption
//!     (through #1301; the per-installed-plugin engine-held private key
//!     under the caller-mint-first contract — at-rest encryption NOT
//!     shipped; only the keypair *seed* zeroize-on-drop exists).
//!
//! ## SUBSTANTIVE-arm-not-SHAPE shape (R4.1 fix-pass per pim-18 / §3.6f)
//!
//! Each RED arm below **first** exercises SHIPPED adjacent primitives —
//! `RotationLog::{new,append,is_superseded,entries}` for the rehydrate
//! arm; `Keypair::generate`/`PluginDidStore::insert/get` for the
//! resolve round-trip + key-at-rest arms — with a real assertion +
//! observable would-FAIL consequence, **then** `panic!`-holds the
//! still-undelivered structural seams (rehydrate-at-open / standalone
//! resolve_did_for_cid / #1301 envelope at rest). Hybrid mostly-
//! undelivered pattern (R4.1 pattern-induction).
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-2-amendment (§3.6b sub-rule-4): each RED arm exercises the
//!     SPECIFIC surface (rehydrate-at-open / resolve round-trip /
//!     key-at-rest), production call-site + observable consequence +
//!     would-FAIL-if-no-op'd.
//!   - pim-12 (§3.6e): the two DESTINATION-REMAPPED shells are §4.26
//!     redirect obligations; G-CORE-8 un-ignores them; reviewer
//!     verifies landing-status not just spec-pin presence.
//!   - pim-18 (§3.6f): substantive arms, not "a RotationLog type
//!     exists". Hybrid mostly-undelivered pattern: exercise SHIPPED
//!     primitives + panic-hold the missing structural seam.
//!   - §3.13: no shared process-scoped static (discharged structurally).
//!   - §3.11: TF-8 largest cross-crate family — resume INTO worktree.
//!
//! Pins: G-CORE-8 · C8 · §1.A.FROZEN item 12 (§4.40 key-at-rest public
//! type) + §4.40 couples #1301 (key-at-rest-via-#1301 envelope).
//! R2 map: TF-8 §4.26 + §4.40 + §4.27-verify-stays.

#![allow(clippy::unwrap_used, clippy::expect_used)]

// --- §4.27 VERIFY-STAYS-REGRESSION (already-landed; NOT RED) ---

#[test]
fn plugin_did_mint_uses_os_rng_two_mints_distinct_verify_stays() {
    // VERIFY-STAYS: `plugin_did::mint` uses OsRng (two independent
    // mints are distinct). Already pinned by the un-ignored
    // `plugin_did_install_uses_os_rng_not_seed_derivation.rs`; this
    // guards the property survives the G-CORE-8 §4.26/§4.40 work
    // (which touches the adjacent RotationLog/key-at-rest surface).
    use benten_id::plugin_did::mint;
    let a = mint();
    let b = mint();
    assert_ne!(
        a.did(),
        b.did(),
        "verify-stays (§4.27): two plugin_did::mint() calls must yield \
         distinct DIDs — OsRng provenance, NOT deterministic seed \
         derivation (would-FAIL if a deterministic/HKDF regression \
         lands in the G-CORE-8 window)"
    );
}

// --- §4.26 STILL-RED staged-pins (redirect obligation for the two
//     DESTINATION-REMAPPED shells) ---

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.26 RotationLog \
            rehydrate at engine-open — also un-ignores the \
            DESTINATION-REMAPPED rotation_log_rehydrated_at_engine_open.rs \
            shell per §3.6e)"]
fn rotation_log_rehydrates_persisted_events_at_engine_open() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (substantive-arm anchor for pim-18 §3.6f):
    // the SHIPPED `RotationLog::{from_entries, is_superseded, entries}`
    // surface lets us *simulate* the post-restart rehydrate contract —
    // construct a log AS IF rehydrated from durable storage, then
    // exercise the query path. Real assertion + observable would-FAIL.
    // -----------------------------------------------------------------
    use benten_id::did::Did;
    use benten_id::did_rotation::RotationLog;

    // Build an empty log (the pre-rehydrate state) — `is_superseded`
    // answers `false` on any DID. This is the would-FAIL state at
    // post-restart pre-G-CORE-8 (no rehydrate seam → log starts empty).
    let pre_rehydrate = RotationLog::new();
    let some_did = Did::from_string_unchecked("did:key:zRotatedKey".to_string());
    assert!(
        !pre_rehydrate.is_superseded(&some_did),
        "shipped surface exercise (pre-rehydrate state): an empty \
         RotationLog reports NO DID as superseded — this is the \
         would-FAIL post-restart state without a rehydrate seam."
    );
    assert!(
        pre_rehydrate.entries().is_empty(),
        "shipped surface exercise: an empty log has zero entries"
    );

    // -----------------------------------------------------------------
    // RED-arm: the engine-open rehydration seam that PERSISTS rotation
    // events to durable storage + RELOADS them on engine-open is
    // UNDELIVERED. The SHIPPED query path (`is_superseded`) is
    // exercised correctly above on a constructed log — the structural
    // gap is between "log constructed in-process" and "log rehydrated
    // from durable storage at engine open". A real durable-storage
    // round-trip cannot be exercised pre-G-CORE-8.
    //
    // §3.6e: the existing DESTINATION-REMAPPED shell
    // `rotation_log_rehydrated_at_engine_open.rs` is un-ignored at
    // G-CORE-8 alongside this arm (its ignore message already cites
    // §4.26 — reviewer verifies LANDING status).
    // -----------------------------------------------------------------
    panic!(
        "§4.26 RotationLog rehydrate-at-engine-open undelivered: the \
         SHIPPED `RotationLog::{{is_superseded, entries}}` query path is \
         correct on in-process logs (exercised above on the empty pre- \
         rehydrate state), but the engine-open rehydration seam that \
         loads persisted rotation events from durable storage at engine \
         open is NOT wired (couples §4.20 engine-builder seam). G-CORE-8 \
         wires it + un-ignores the DESTINATION-REMAPPED shell."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.26 resolve_did_for_cid \
            round-trip — also un-ignores the DESTINATION-REMAPPED \
            resolve_did_for_cid_round_trip.rs shell per §3.6e)"]
fn resolve_did_for_cid_returns_owning_did_round_trip() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the substrate for the resolve seam is
    // `PluginDidStore::{insert, get}` mapping DID → handle, paired
    // with the content-addressed identity primitives already shipped
    // via `Keypair`. Exercise the DID round-trip through the SHIPPED
    // store surface — the would-FAIL signal for the future
    // `resolve_did_for_cid` standalone seam is the same DID-identity
    // contract.
    // -----------------------------------------------------------------
    use benten_id::plugin_did::{PluginDidStore, mint};

    let handle = mint();
    let did = handle.did().clone();
    let mut store = PluginDidStore::new();
    store
        .insert(handle)
        .expect("fresh handle insert must succeed (positive control)");

    // Round-trip the DID through the SHIPPED store: insert + get.
    let retrieved = store.get(&did).expect(
        "inserted DID must be retrievable from store (would-FAIL \
                 if the insert-then-get round-trip regressed)",
    );
    assert_eq!(
        retrieved.did(),
        &did,
        "shipped surface exercise: store round-trip returns the inserted \
         DID identity (the substrate for the future resolve_did_for_cid \
         standalone seam — content-addressed DID identity round-trip). \
         Would-FAIL if the store mutated DID identity through insert/get."
    );

    // Negative control: a DID NOT in the store does not resolve.
    let absent = benten_id::did::Did::from_string_unchecked("did:key:zNotInStore".to_string());
    assert!(
        store.get(&absent).is_none(),
        "shipped surface exercise: an unknown DID does not resolve to a \
         store entry (the substrate for the resolve seam's None case)"
    );

    // -----------------------------------------------------------------
    // RED-arm: the standalone `resolve_did_for_cid(cid) → Did` seam
    // (cap-r1-16 → triaged into G24-F DidKeyedSession::resolve; the
    // standalone seam is a separate Phase-4-Meta concern coupled to
    // RotationLog rehydration) is UNDELIVERED. The substrate
    // identity-round-trip primitive works (above); what is missing
    // is the CID-keyed content-store → owning-DID mapping seam.
    // -----------------------------------------------------------------
    panic!(
        "§4.26 resolve_did_for_cid round-trip undelivered: the SHIPPED \
         `PluginDidStore::{{insert, get}}` round-trip (DID → handle) is \
         correct (exercised above), but the standalone CID-keyed \
         resolve seam (`resolve_did_for_cid(cid) → owning-DID`) was \
         never minted (cap-r1-16 was triaged into G24-F \
         DidKeyedSession::resolve; the standalone seam is a separate \
         Phase-4-Meta concern coupled to RotationLog rehydration). \
         G-CORE-8 mints it + un-ignores the shell."
    );
}

// --- §4.40 STILL-RED: engine-held plugin-DID private-key at-rest ---

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-8 (§4.40 engine-held \
            plugin-DID private-key at-rest encryption through #1301)"]
fn engine_held_plugin_did_private_key_encrypted_at_rest_through_1301() {
    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE: the substrate the §4.40 #1301 envelope
    // wraps is the SHIPPED `Keypair::secret_bytes_for_test` (test-only
    // accessor; production callers use `export_seed_envelope` — both
    // currently return PLAINTEXT byte-array). Exercise the SHIPPED
    // primitive to assert the plaintext bytes are RETRIEVABLE at HEAD
    // (the would-FAIL signal post-G-CORE-8: under the #1301 envelope
    // the same accessor would yield sealed bytes / require unsealing
    // through the codepoint-dispatched envelope).
    // -----------------------------------------------------------------
    use benten_id::keypair::Keypair;
    use benten_id::plugin_did::{PluginDidStore, mint};

    let handle = mint();
    let did = handle.did().clone();
    let mut store = PluginDidStore::new();
    store.insert(handle).expect("insert positive control");

    // At HEAD: re-mint a keypair and observe the substrate primitive's
    // PLAINTEXT-bytes contract directly. The future #1301 envelope
    // will wrap this surface; the would-FAIL post-G-CORE-8 is that
    // these bytes are no longer recoverable through the plaintext
    // accessor.
    let kp = Keypair::generate();
    let bytes: [u8; 32] = kp.secret_bytes_for_test();
    let nonzero_count = bytes.iter().filter(|b| **b != 0).count();
    assert!(
        nonzero_count > 0,
        "shipped surface exercise: at HEAD, the Keypair secret bytes \
         accessor returns PLAINTEXT 32-byte material (substrate for the \
         §4.40 + #1301 envelope wrap). Would-FAIL if the bytes \
         regressed to all-zero (which would mean the substrate was \
         silently broken). The §4.40 RED contract is precisely that \
         these plaintext bytes are STILL recoverable at HEAD — post- \
         G-CORE-8 + #1301 the same accessor MUST yield sealed bytes."
    );

    // Sanity: the inserted DID is still in the store (positive control
    // — the in-memory store works; what changes post-#1301 is HOW the
    // private-key material is stored at rest).
    assert!(
        store.get(&did).is_some(),
        "shipped surface exercise: the inserted DID is retrievable in- \
         memory (the substrate the at-rest seal will wrap)."
    );

    // -----------------------------------------------------------------
    // RED-arm: PluginDidStore private keys are NOT sealed at rest via
    // the #1301 codepoint-dispatched envelope; in-memory zeroization-
    // on-uninstall is not wired; the T13 threat-class is documented
    // but not enforced.
    // -----------------------------------------------------------------
    panic!(
        "§4.40 engine-held plugin-DID key-at-rest undelivered: the \
         SHIPPED Keypair secret-bytes accessor returns PLAINTEXT \
         32-byte material at HEAD (exercised above — the substrate the \
         #1301 envelope wrap will replace), but PluginDidStore private \
         keys are not sealed at rest via the #1301 envelope, and \
         in-memory zeroization-on-uninstall is not wired. G-CORE-8 \
         (coupled to #1301) ships key-at-rest encryption + the T13 \
         threat-model + SECURITY-POSTURE Compromise narrative."
    );
}
