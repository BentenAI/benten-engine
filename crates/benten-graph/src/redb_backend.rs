//! Concrete [`RedbBackend`] — a [`KVBackend`] implementation over redb v4.
//!
//! Extracted from `lib.rs` as part of G2-B, alongside:
//! - the explicit `open_existing` / `open_or_create` split
//!   (R1 triage `P1.graph.open-vs-create`);
//! - the [`DurabilityMode`] wiring (R1 triage `P1.graph.durability`);
//! - the label and property-value indexes (crate-private `indexes` module,
//!   R1 triage `P1.graph.indexes-on-write`).
//!
//! The module owns the redb table definitions and all of the redb-specific
//! plumbing. The `KVBackend` trait it implements lives in [`crate::backend`],
//! and the higher-level `NodeStore` / `EdgeStore` traits it implements live
//! in [`crate::store`]. Inherent methods on [`RedbBackend`] (`put_node`,
//! `delete_node`, …) are the single source of truth for the index contract:
//! they maintain the label and property-value indexes as part of the same
//! write transaction, so the indexes are always in sync with the node store.

#[cfg(any(test, feature = "testing"))]
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::mutex_ext::MutexExt;

use benten_core::{Cid, Edge, Node, Value};
use redb::{
    Database, Durability, MultimapTableDefinition, ReadableDatabase, ReadableMultimapTable,
    ReadableTable, TableDefinition,
};

use crate::backend::{DurabilityMode, KVBackend, ScanResult};
use crate::immutability::CidExistenceCache;
use crate::indexes::{
    LABEL_INDEX_TABLE, PROP_INDEX_TABLE, cid_from_index_bytes, property_index_key,
    value_index_bytes,
};
use crate::store::{
    ChangeEvent, ChangeKind, ChangeSubscriber, EDGE_SRC_PREFIX, EDGE_TGT_PREFIX, EdgeStore,
    GRAPH_SCHEMA_VERSION, NodeStore, SCHEMA_VERSION_KEY, decode_err, edge_key, edge_src_index_key,
    edge_src_index_prefix, edge_tgt_index_key, edge_tgt_index_prefix, namespaced_ciphertext_key,
    namespaced_edge_key, namespaced_edge_src_index_key, namespaced_edge_src_index_prefix,
    namespaced_edge_tgt_index_key, namespaced_label_index_key, namespaced_label_index_prefix,
    namespaced_node_key, namespaced_node_prefix, node_key,
};
use crate::transaction::{TxGuard, fan_out};
use crate::{GraphError, Transaction, WriteAuthority, WriteContext};

/// Shared system-zone label-prefix check used by every write entry point on
/// [`RedbBackend`] — both the `WriteContext`-aware paths and the inherent
/// `put_node` / `put_edge` that the [`NodeStore`] / [`EdgeStore`] trait
/// delegates route through. An unprivileged write whose Node has any label
/// starting with `"system:"` returns `E_SYSTEM_ZONE_WRITE`.
///
/// Extracted at the G3 mini-review fix-pass (chaos-engineer g3-ce-1). Before
/// this helper existed, the inherent `RedbBackend::put_node` bypassed the
/// guard entirely — a binding caller or trait-dispatching generic code could
/// forge a `system:CapabilityGrant` via the plain `put_node` path while the
/// `put_node_with_context` path correctly rejected.
fn guard_system_zone_node(node: &Node, is_privileged: bool) -> Result<(), GraphError> {
    if is_privileged {
        return Ok(());
    }
    for label in &node.labels {
        if label.starts_with("system:") {
            return Err(GraphError::SystemZoneWrite {
                label: label.clone(),
            });
        }
    }
    Ok(())
}

/// Edge counterpart of [`guard_system_zone_node`]. R1 SC1 named only Node
/// labels explicitly, but edges with `"system:"`-prefixed labels are the
/// obvious smuggling vector (an edge `system:Grant` from an attacker's
/// principal to a privileged capability), so the prefix reservation
/// extends to edge labels as well.
fn guard_system_zone_edge(edge: &Edge, is_privileged: bool) -> Result<(), GraphError> {
    if !is_privileged && edge.label.starts_with("system:") {
        return Err(GraphError::SystemZoneWrite {
            label: edge.label.clone(),
        });
    }
    Ok(())
}

/// Primary key/value table storing every `(key, value)` pair. The Node and
/// Edge stores layer the `n:CID`, `e:CID`, `es:SRC|EDGE`, `et:TGT|EDGE` key
/// schema on top of this table.
pub(crate) const NODES_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("benten_nodes");

/// Phase-4-Meta-Core G-CORE-3d (#1301): two-CID mapping table —
/// `plaintext_cid → ciphertext_cid` for the per-Node AEAD wrap path.
/// Stored separately from `NODES_TABLE` so a future redb-schema
/// migration can re-key the mapping shape without touching the node
/// body table.
///
/// Keying:
/// - un-namespaced writes (legacy `WriteContext::namespace_did = None`)
///   do NOT touch this table — the legacy path stores plaintext bytes
///   at `n:<plaintext_cid>` and `plaintext_cid == ciphertext_cid` by
///   construction.
/// - namespaced writes (`namespace_did = Some(did)`) record a row keyed
///   on `TwoCidMap::partition_table_key(did, plaintext_cid) =
///   d:<did>:m:<plaintext_cid>`, with value = `ciphertext_cid.as_bytes()`.
///
/// The partition prefix in the key makes cross-DID lookups structurally
/// invisible — the partition-isolation arm (multitenant-r1-5) fires at
/// the prefix match step BEFORE any AEAD layer.
pub(crate) const TWO_CID_MAP_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("benten_two_cid_map");

/// G-CORE-3e: derive K(N) for a plaintext CID under a namespace DID,
/// using the production HKDF-SHA256 structural-KDF substrate from
/// `benten_crypto_suite::structural_kdf` (Spike-E Interpretation-B
/// path-tagged derivation per
/// `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
/// §R-key-derivation).
///
/// **G-CORE-3e production swap (1-line):** this function replaces the
/// G-CORE-3d wave's BLAKE3 test-seam derivation at the same boundary.
/// The structural-KDF chain is:
///
/// ```text
/// K_principal = HKDF-SHA256(domain_tag, info = did.as_bytes())
/// K(root)     = derive_root(K_principal, root_cid = cid.as_bytes())
/// K(N)        = K(root)   // single-Node walk at this seam; the
///                         // multi-edge derive_step chain lands at
///                         // the future subgraph-walk wire-up
/// ```
///
/// The single-Node case (no canonical-path walk yet — the
/// `derive_step` chain through SubgraphSpec walks lands at the
/// future subgraph-walk wire-up) routes through `derive_root` so
/// the substrate is exercised end-to-end + the seam is the same
/// boundary the multi-edge wire-up swaps in at.
///
/// **K_principal seam.** The per-DID `K_principal` material lives
/// behind a per-deployment secret store at the production wire-up
/// (#989 / #1301 substrate). At this wave the K_principal is
/// deterministically derived from the namespace_did via
/// HKDF-SHA256 over a domain-tag — this lets the wave-3e
/// per-recipient seal/unseal path produce stable keys without the
/// K_principal storage seam landing first. The seam is named at
/// `docs/future/phase-4-backlog.md` §3.10 G-CORE-3e (K_principal
/// per-DID secret store) for the production replacement.
///
/// **⚠️ Confidentiality limit at this wave (K_principal-seam stand-in).**
/// The `K_PRINCIPAL_DOMAIN_KEY` constant + the publicly-known
/// `namespace_did` `Cid` bytes are the ONLY inputs to the K_principal
/// synthesis at this wave — so **any party holding
/// `(namespace_did, ciphertext_blob)` can derive `K(N)` and decrypt**.
/// The `_test_seam_` hint in the function name is load-bearing on every
/// caller until the K_principal-store backend lands; do NOT rely on the
/// wave-3e confidentiality envelope for any data not also protected by
/// namespace-isolation at the storage backend. See
/// `docs/SECURITY-POSTURE.md` ("Confidentiality limit at this wave"
/// callout under the G-CORE-3e key-derivation section).
pub fn derive_test_seam_key_from_cid_with_namespace(did: Option<&Cid>, cid: &Cid) -> Vec<u8> {
    // Step 1 — synthesize K_principal from a domain tag + namespace
    // DID. The domain tag separates the K_principal-synthesis role
    // from any other HKDF use across the workspace. The wave-3e
    // namespace_did binds K_principal to a specific Atrium identity;
    // the un-namespaced legacy path (None) uses a fixed sentinel so
    // legacy plaintext nodes can still be round-tripped (the
    // un-namespaced path doesn't AEAD-wrap at all in the live
    // put_node_with_context, but the test-seam fallback path here
    // covers any future caller).
    let did_bytes: &[u8] = match did {
        Some(d) => d.as_bytes(),
        // Sentinel for the un-namespaced fallback. Distinct from
        // any real DID so collisions are impossible.
        None => b"benten/g-core-3e/unscoped-principal/v1",
    };
    // 32-byte domain key for the K_principal HKDF input. Stable
    // across runs (the test-seam path is deterministic at this
    // wave); the production wire-up reads this from the per-DID
    // secret-material backend.
    // Literal carries 32 bytes — exactly the BLAKE3 keyed-hash KEY_LEN.
    const K_PRINCIPAL_DOMAIN_KEY: [u8; 32] = *b"benten/g-core-3e/k-principal-v1\0";
    let k_principal_bytes = blake3::keyed_hash(&K_PRINCIPAL_DOMAIN_KEY, did_bytes);
    let k_principal = benten_crypto_suite::structural_kdf::StructuralKdfKey::from_bytes_for_test(
        k_principal_bytes.as_bytes(),
    );

    // Step 2 — derive K(root) for this Node via the HKDF-SHA256
    // structural-KDF substrate. `derive_root` carries the "root"
    // info-tag for cross-role domain separation per Spike-E +
    // `benten_crypto_suite::structural_kdf` contract.
    let k_root = benten_crypto_suite::structural_kdf::derive_root(&k_principal, cid.as_bytes());
    k_root.as_bytes().to_vec()
}

/// G-CORE-3d back-compat shim — the un-namespaced legacy callers
/// that previously called `derive_test_seam_key_from_cid(&cid)`
/// route through the new namespace-aware helper with
/// `namespace = None`. Kept as a thin shim so any in-flight call
/// site keeps compiling; new callers should prefer the
/// namespace-aware form directly.
#[allow(dead_code)]
fn derive_test_seam_key_from_cid(cid: &Cid) -> Vec<u8> {
    derive_test_seam_key_from_cid_with_namespace(None, cid)
}

/// G-CORE-3e: parse the namespace_did `Cid` out of a TWO_CID_MAP_TABLE
/// key shaped `d:<namespace_did_bytes>:m:<plaintext_cid_bytes>` (per
/// `crate::two_cid_map::TwoCidMap::partition_table_key`). Returns
/// `None` if the key prefix doesn't carry the partition shape (i.e.
/// the un-namespaced legacy `m:<plaintext_cid>` form). Used by
/// [`RedbBackend::two_cid_lookup_with_namespace`] to recover the
/// namespace_did the seal-time `put_node_with_context` call ran
/// under so the unseal-side K(N) derivation routes through the same
/// HKDF-SHA256 K_principal.
fn parse_namespace_from_mapping_key(key_bytes: &[u8]) -> Option<Cid> {
    // Expected prefix: b"d:" + did_bytes + b":m:" + plaintext_cid_bytes.
    // The `Cid::as_bytes()` form is fixed-width (36 bytes per
    // multihash+multicodec) but we don't strictly depend on that —
    // we slice by the `:m:` delimiter that appears between the DID
    // bytes and the plaintext_cid bytes.
    let prefix = b"d:";
    if !key_bytes.starts_with(prefix) {
        return None;
    }
    let after_prefix = &key_bytes[prefix.len()..];
    // Find the `:m:` delimiter. Cid bytes don't contain `:m:` as a
    // 3-byte substring by construction (the multihash framing
    // doesn't emit ASCII colons), so this scan is safe.
    let needle = b":m:";
    let idx = after_prefix
        .windows(needle.len())
        .position(|w| w == needle)?;
    let did_bytes = &after_prefix[..idx];
    Cid::from_bytes(did_bytes).ok()
}

/// G-CORE-3d: AEAD-wrapped node body storage. Whereas the legacy
/// `NODES_TABLE` stores plaintext DAG-CBOR bytes (un-namespaced) +
/// plaintext DAG-CBOR under `d:<did>:n:<cid>` (namespaced un-encrypted
/// shape pre-3d), this table stores AEAD envelope wire bytes (per
/// `benten_crypto_suite::AeadEnvelope::to_wire_bytes`) at
/// `d:<did>:c:<ciphertext_cid>`. The key prefix `c:` is distinct from
/// `n:` so a future schema-version migration that consolidates the
/// tables can disambiguate ciphertext storage from plaintext storage
/// without re-keying.
pub(crate) const ENCRYPTED_NODES_TABLE: TableDefinition<&[u8], &[u8]> =
    TableDefinition::new("benten_encrypted_nodes");

/// G11-A unbounded-cache bound: maximum entries in the test-only
/// `test_event_log` before the buffer is cleared. Tests drain the
/// buffer between assertions, so the cap only fires in a long-lived bench
/// or integration process that writes tens of thousands of nodes without
/// calling `drain_change_events_for_test`.
///
/// Wave-1 mini-review MODERATE-3: gated behind `cfg(any(test, feature =
/// "testing"))` — production builds compile the `test_event_log` away
/// entirely, so the cap has nothing to bind.
#[cfg(any(test, feature = "testing"))]
const TEST_EVENT_LOG_CAP: usize = 10_000;

/// G11-A unbounded-cache bound: maximum entries in the
/// `last_durability_by_label` test-inspection map before the map is cleared.
/// Callers only ever ask for the most-recent tier per label, so eviction
/// preserves correctness for the always-recent test hook caller; a
/// far-future test that persists across 1k+ distinct labels would need to
/// re-stamp before inspecting.
///
/// Wave-1 mini-review MODERATE-3: gated behind `cfg(any(test, feature =
/// "testing"))` — production builds compile the map away entirely.
#[cfg(any(test, feature = "testing"))]
const LAST_DURABILITY_MAP_CAP: usize = 1_000;

// G13-C wave-3: `next_prefix` was promoted to `crate::prefix_helpers` so
// it can be shared with `BrowserBackend` (cfg-gated independently of
// redb). Re-exported here so existing call sites + the in-module test
// continue to find it under the historical path.
pub(crate) use crate::prefix_helpers::next_prefix;

/// Map a [`DurabilityMode`] onto redb's own `Durability` enum.
///
/// redb v4 currently exposes `Durability::Immediate` (fsync-on-commit) and
/// `Durability::None` (in-memory, lost on crash) only; the intermediate
/// `Group` mode the Benten trait exposes has no direct redb equivalent yet,
/// so it collapses to `Immediate` until redb grows batched-fsync support.
/// This conservative mapping preserves durability at the cost of the
/// throughput win; Phase 3+ can revisit without breaking the public enum.
///
/// **Phase-3 G13-E posture:** `DurabilityMode::default()` returns `Group`
/// (closes Compromise #12 at the engine surface). The redb mapping below
/// still collapses Group → Immediate, so `RedbBackend::open_or_create`
/// today gets the same on-disk behavior as before the flip — but the
/// engine-level posture is now correct, and a non-redb backend (in-RAM
/// thin-client per `crates/benten-graph/src/browser_backend.rs` when it
/// lands at G13-C, or a future peer-sync backend) can implement true
/// grouped fsync without changing call sites.
fn to_redb_durability(mode: DurabilityMode) -> Durability {
    match mode {
        DurabilityMode::Immediate | DurabilityMode::Group => Durability::Immediate,
        DurabilityMode::Async => Durability::None,
    }
}

/// Emit a one-shot warning when a caller requests `DurabilityMode::Group` so
/// benchmarks and production tuning don't silently compare Group to
/// Immediate and conclude grouped-fsync "doesn't help" — it simply isn't
/// wired yet. Fires at most once per process.
///
/// Written to stderr directly (no `tracing` dep on this crate); `clippy::
/// print_stderr` is allowed for this one callsite with an explicit reason.
#[allow(
    clippy::print_stderr,
    reason = "one-shot operator-visible warning about a Phase-1 API gap; \
              benten-graph has no tracing dep"
)]
fn warn_if_group_durability_collapsed(mode: DurabilityMode) {
    use std::sync::Once;
    static WARNED: Once = Once::new();
    if matches!(mode, DurabilityMode::Group) {
        WARNED.call_once(|| {
            eprintln!(
                "benten-graph: DurabilityMode::Group collapses to \
                 Durability::Immediate at the redb v4 mapping layer — \
                 redb does not yet expose grouped-commit. Benchmarks \
                 comparing Group vs. Immediate will see no delta until \
                 redb grows native batched-fsync support OR Benten adds \
                 its own write-batching layer above redb."
            );
        });
    }
}

/// One registered change-event subscriber, paired with its optional per-DID
/// partition filter. Phase-4-Meta-Core G-CORE-1 (#989): see the
/// [`RedbBackend::subscribers`](struct.RedbBackend.html) field doc + the
/// `transaction` / `put_node_with_context` fan-out paths.
pub(crate) type SubscriberSlot = (Option<Cid>, Arc<dyn ChangeSubscriber>);

/// A [`KVBackend`] implementation backed by a local redb v4 database file.
///
/// redb provides serializable isolation (single writer, multiple readers)
/// and durable commits via a two-phase commit with checksummed pages.
///
/// # Construction
///
/// Three entry points, each with an explicit contract:
///
/// | Constructor | Existing file | Missing file |
/// |---|---|---|
/// | [`RedbBackend::open_existing`] | opens | errors with [`GraphError::BackendNotFound`] |
/// | [`RedbBackend::open_or_create`] | opens | creates |
/// | [`RedbBackend::open`] | opens | creates (kept for backward-compatibility with the spike; new code should pick `open_existing` or `open_or_create` explicitly) |
///
/// `open_existing` is the safer default — it refuses to silently materialize a
/// fresh database under a typoed path (R1 triage `P1.graph.open-vs-create`).
///
/// # Durability
///
/// Both constructors take [`DurabilityMode::default()`] which returns
/// [`DurabilityMode::Group`] since Phase-3 G13-E (was
/// [`DurabilityMode::Immediate`] through Phase-2b). At the redb mapping
/// layer Group still collapses to `Durability::Immediate` until redb v4+
/// grows native batched-commit support — see
/// `crates/benten-graph/src/redb_backend.rs::to_redb_durability`. The
/// [`RedbBackend::open_existing_with_durability`] and
/// [`RedbBackend::open_or_create_with_durability`] variants let callers
/// pick `Immediate` (capability-grant writes; pin-precise crash semantics)
/// or `Async` (bench harness, ephemeral test fixture) explicitly.
///
/// # Concurrency
///
/// `RedbBackend` is not `Clone`. To share a single backend across threads,
/// wrap it in an `Arc`: `let backend = Arc::new(RedbBackend::open_or_create(path)?)`.
/// redb's own API is `&self`, so multiple readers and a single writer can
/// proceed concurrently through the shared `Arc`.
///
/// # Path handling
///
/// The constructors do not canonicalize or validate the database path.
/// Callers receiving paths from untrusted sources (capability-delegated
/// subgraphs, multi-tenant configurations) must sanitize before invoking.
pub struct RedbBackend {
    db: Database,
    durability: Durability,
    /// Configured [`DurabilityMode`] the backend was constructed with. Kept
    /// alongside the redb-flavoured `durability` because Inv-13 / capability-
    /// grant paths want to report the logical mode back to callers (via
    /// [`RedbBackend::last_put_node_durability_for_label`]) even when they
    /// locally override the redb flavour.
    configured_durability: DurabilityMode,
    /// Per-call override record: the last `DurabilityMode` used by a
    /// `put_node_with_context` commit, keyed on every label the persisted
    /// Node carried. Used by the test hook
    /// [`RedbBackend::last_put_node_durability_for_label`]; the
    /// capability-grant path in particular stamps `Immediate` here regardless
    /// of [`Self::configured_durability`] so revocation-ordering cannot be
    /// reordered by a looser configured mode.
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")` — production builds compile the map (and
    /// the per-commit stamping block inside `put_node_with_context`)
    /// away entirely. Closes the commit-message claim in `be8f6ea` that
    /// the accumulation surface no longer exists in production.
    #[cfg(any(test, feature = "testing"))]
    last_durability_by_label: Arc<Mutex<HashMap<String, DurabilityMode>>>,
    /// Inv-13 fast-path CID-existence cache (G2-A skeleton; G5-A wires the
    /// 5-row matrix on top). See [`crate::immutability`] for the fast-path
    /// contract.
    immutability_cache: Arc<Mutex<CidExistenceCache>>,
    /// Test-only change-event buffer drained by
    /// [`RedbBackend::drain_change_events_for_test`]. G5-A populates the
    /// write side; G2-A leaves it empty so the method surface compiles for
    /// tests that don't assert on it.
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")` — production builds compile the buffer and
    /// the per-commit push block away entirely.
    #[cfg(any(test, feature = "testing"))]
    test_event_log: Arc<Mutex<Vec<ChangeEvent>>>,
    /// Registered change-event subscribers. Behind a `Mutex<Vec<...>>` so
    /// `register_subscriber` and the post-commit fan-out can share one
    /// list without forcing callers to hold an `Arc<RedbBackend>`.
    ///
    /// Phase-4-Meta-Core G-CORE-1 (#989): each entry carries an
    /// `Option<Cid>` subscriber-side namespace filter (see [`SubscriberSlot`]):
    /// `None` is an un-namespaced subscriber receiving events from
    /// un-namespaced writes only (registered via
    /// [`Self::register_subscriber`]); `Some(did)` is a namespaced
    /// subscriber receiving events from writes whose
    /// `WriteContext::namespace_did == Some(did)` (registered via
    /// [`ScopedView::register_subscriber`]). Fan-out (in
    /// `put_node_with_context` / `put_edge_with_context` / `transaction`)
    /// filters by exact match on this field so the C1 cross-DID non-leak
    /// invariant holds at the change-subscriber boundary (TF-1 PIN 4
    /// asserts this end-to-end).
    subscribers: Arc<Mutex<Vec<SubscriberSlot>>>,
    /// In-transaction flag. Set via [`TxGuard`] at the start of a
    /// closure-based transaction, cleared on drop. Prevents nested
    /// `backend.transaction(|_| backend.transaction(...))` calls from
    /// deadlocking on redb's single-writer lock; the second
    /// `RedbBackend::transaction` sees `true` and returns
    /// [`GraphError::NestedTransactionNotSupported`] without ever asking
    /// redb to open a second write txn.
    ///
    /// TODO(phase-3 — multi-Arc tx-flag coordination): the flag is
    /// per-`Arc<RedbBackend>`; two distinct Arc handles opened on the
    /// same redb file do not coordinate and fall through to redb's
    /// single-writer lock (which blocks rather than deadlocking).
    /// Mini-review g3-ce-7 proposes keying the flag on the canonical
    /// DB path via a process-wide static. Phase-2 treated the
    /// single-handle invariant as documented; carry to Phase-3
    /// alongside backend-genericism work (§1.1).
    tx_flag: Arc<Mutex<bool>>,
    /// Monotonically increasing transaction id stamped onto
    /// [`crate::ChangeEvent::tx_id`]. Starts at 1 so that tests can reserve
    /// 0 as "no event". Atomic because the backend may be shared across
    /// threads behind an `Arc`.
    ///
    /// TODO(phase-3 — durable tx_id high-water-mark): `tx_id` is
    /// process-lifetime-only; reopening the backend restarts the
    /// counter at 1. An IVM persistence layer that uses `tx_id` as a
    /// durable high-water-mark would see a monotonicity violation
    /// across restart. Mini-review g3-ce-8 proposes persisting the
    /// counter into a dedicated redb table; Phase 2 documented the
    /// limitation (IVM views rebuild from scratch on restart). Carry
    /// to Phase-3 alongside the IVM persistence work.
    next_tx_id: Arc<AtomicU64>,
    /// Wave-1 mini-review SEVERE-2: storage-layer commit counter. Bumped
    /// once per real `put_node_with_context` commit (excluding the dedup
    /// early-return path, which is a pure read per Compromise #N+1 / §9.11
    /// row 3). Surfaced via [`Self::writes_committed`] so the engine's
    /// `audit_sequence()` accessor can observe the dedup-no-advance
    /// contract without routing privileged capability writes through
    /// [`crate::Transaction`] accounting (which would drag the grant path
    /// through `Engine::transaction` and change the shape of the privileged
    /// write API).
    writes_committed: Arc<AtomicU64>,
}

impl core::fmt::Debug for RedbBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RedbBackend").finish_non_exhaustive()
    }
}

impl RedbBackend {
    // ---- Construction -----------------------------------------------------

    /// Open a redb database that must already exist at `path`. Fails with
    /// [`GraphError::BackendNotFound`] if the file is missing — this is the
    /// safer default for production code paths that want to refuse to
    /// silently materialize a new database under a typoed path.
    ///
    /// Commits use [`DurabilityMode::default()`] — [`DurabilityMode::Group`]
    /// since Phase-3 G13-E. At the redb mapping layer this collapses to
    /// `Durability::Immediate` (fsync per commit) until redb grows native
    /// batched-commit support. See
    /// `crates/benten-graph/src/backend.rs::DurabilityMode` for the
    /// Compromise #12 closure narrative.
    ///
    /// # Errors
    /// - [`GraphError::BackendNotFound`] if `path` does not exist.
    /// - [`GraphError::Redb`] for any other redb open failure (corrupt file,
    ///   incompatible version, I/O error, lock contention).
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::{GraphError, RedbBackend};
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let missing = dir.path().join("does-not-exist.redb");
    /// let err = RedbBackend::open_existing(&missing).unwrap_err();
    /// assert!(matches!(err, GraphError::BackendNotFound { .. }));
    /// ```
    pub fn open_existing(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        Self::open_existing_with_durability(path, DurabilityMode::default())
    }

    /// Open-existing with an explicit [`DurabilityMode`].
    ///
    /// # Errors
    /// Same as [`Self::open_existing`].
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::{DurabilityMode, RedbBackend};
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let path = dir.path().join("db.redb");
    /// // Materialize the file first so `open_existing*` has something to open.
    /// let _first = RedbBackend::open_or_create(&path).unwrap();
    /// drop(_first);
    ///
    /// let _reopened = RedbBackend::open_existing_with_durability(
    ///     &path,
    ///     DurabilityMode::Immediate,
    /// )
    /// .unwrap();
    /// ```
    pub fn open_existing_with_durability(
        path: impl AsRef<Path>,
        durability: DurabilityMode,
    ) -> Result<Self, GraphError> {
        // Note: the `path.exists()` check below races with external
        // filesystem mutations (TOCTOU). In Phase 1 the value is a clean
        // `GraphError::BackendNotFound` instead of an opaque
        // "unable to allocate page" leak through `GraphError::Redb` —
        // acceptable for single-user local stores under redb's exclusive
        // lock. Phase 3 P2P workloads may revisit.
        warn_if_group_durability_collapsed(durability);
        let path = path.as_ref();
        if !path.exists() {
            return Err(GraphError::BackendNotFound {
                path: path.to_path_buf(),
            });
        }
        let db = Database::open(path)?;
        // open-existing: read-only schema-envelope verify (absent ≡ v1).
        Self::from_db(db, durability, false)
    }

    /// Assemble a `RedbBackend` from an already-opened [`Database`] + the
    /// requested durability tier, then create the schema tables.
    ///
    /// Single shared field-initialization site for the three public
    /// constructors (`open_existing_with_durability` /
    /// `open_or_create_with_durability` / `open_in_memory`). A new field on
    /// `RedbBackend` only needs to be wired here once.
    /// `write_if_absent` distinguishes the open intent for the #992
    /// schema-version envelope: open-or-create / in-memory paths stamp
    /// v1 when the envelope is absent; open-existing is read-only and
    /// treats an absent envelope as implied-v1 (a pre-envelope file IS
    /// the v1 layout). Either way a present-but-mismatched version is
    /// refused.
    fn from_db(
        db: Database,
        durability: DurabilityMode,
        write_if_absent: bool,
    ) -> Result<Self, GraphError> {
        let backend = Self {
            db,
            durability: to_redb_durability(durability),
            configured_durability: durability,
            #[cfg(any(test, feature = "testing"))]
            last_durability_by_label: Arc::new(Mutex::new(HashMap::new())),
            immutability_cache: Arc::new(Mutex::new(CidExistenceCache::new())),
            #[cfg(any(test, feature = "testing"))]
            test_event_log: Arc::new(Mutex::new(Vec::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            tx_flag: Arc::new(Mutex::new(false)),
            next_tx_id: Arc::new(AtomicU64::new(1)),
            writes_committed: Arc::new(AtomicU64::new(0)),
        };
        backend.ensure_tables()?;
        backend.check_schema_version(write_if_absent)?;
        Ok(backend)
    }

    /// Open the redb database at `path`, creating it if it doesn't already
    /// exist. Idempotent on an existing file.
    ///
    /// Commits use [`DurabilityMode::default()`] — [`DurabilityMode::Group`]
    /// since Phase-3 G13-E. At the redb mapping layer this collapses to
    /// `Durability::Immediate` (fsync per commit) until redb grows native
    /// batched-commit support. See
    /// `crates/benten-graph/src/backend.rs::DurabilityMode` for the
    /// Compromise #12 closure narrative.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] if redb cannot open or create the file,
    /// or if the initial table creation transaction fails.
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let path = dir.path().join("fresh.redb");
    /// let _backend = RedbBackend::open_or_create(&path).unwrap();
    /// assert!(path.exists());
    /// ```
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        Self::open_or_create_with_durability(path, DurabilityMode::default())
    }

    /// Open-or-create with an explicit [`DurabilityMode`].
    ///
    /// # Errors
    /// Same as [`Self::open_or_create`].
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::{DurabilityMode, RedbBackend};
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let path = dir.path().join("bench.redb");
    /// // `Async` durability — commit returns before fsync. Test/bench only.
    /// let _backend = RedbBackend::open_or_create_with_durability(
    ///     &path,
    ///     DurabilityMode::Async,
    /// )
    /// .unwrap();
    /// ```
    pub fn open_or_create_with_durability(
        path: impl AsRef<Path>,
        durability: DurabilityMode,
    ) -> Result<Self, GraphError> {
        warn_if_group_durability_collapsed(durability);
        let db = Database::create(path.as_ref())?;
        // open-or-create: stamp the schema-version envelope if absent.
        Self::from_db(db, durability, true)
    }

    /// Backward-compatible alias for [`Self::open_or_create`]. New code should
    /// pick the explicit variant so the create-on-miss semantics are visible
    /// at the call site.
    ///
    /// # Errors
    /// See [`Self::open_or_create`].
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let _backend = RedbBackend::open(dir.path().join("db.redb")).unwrap();
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        Self::open_or_create(path)
    }

    /// Construct a `RedbBackend` over redb's native in-memory page store.
    ///
    /// Used by `Engine::open(":memory:")` (HANDOFF §3.F wave-5) to satisfy
    /// the 9-of-10 `packages/engine/test/*.test.ts` files that drive the
    /// engine without a filesystem path. The returned backend has full
    /// `RedbBackend` semantics (Inv-11 system-zone gating, Inv-13
    /// immutability cache, change-event publishing, the transaction
    /// primitive, MVCC reads) — it just persists nothing across process
    /// restarts.
    ///
    /// Durability is forced to [`DurabilityMode::Async`] (redb
    /// `Durability::None`) because there is no disk to fsync to; passing
    /// any other mode would just collapse to Async anyway.
    ///
    /// # Errors
    /// Returns [`GraphError::RedbSource`] if redb cannot construct the
    /// in-memory database (extremely unlikely — only happens on allocator
    /// failure inside the redb cache).
    pub fn open_in_memory() -> Result<Self, GraphError> {
        let db = Database::builder()
            .create_with_backend(redb::backends::InMemoryBackend::new())
            .map_err(redb::Error::from)?;
        // in-memory: fresh DB every time → stamp the envelope.
        Self::from_db(db, DurabilityMode::Async, true)
    }

    /// Wave-1 mini-review SEVERE-2: current value of the storage-layer
    /// commit counter. Bumped once per successful
    /// `put_node_with_context` commit (dedup early returns do NOT
    /// advance it). The engine's `benten-engine` `Engine::audit_sequence`
    /// accessor reads this counter directly so the §9.11 row-3
    /// "dedup is a pure read" contract is observable at the storage
    /// layer rather than at an engine-transaction layer the privileged
    /// grant path bypasses.
    #[must_use]
    pub fn writes_committed(&self) -> u64 {
        self.writes_committed.load(Ordering::SeqCst)
    }

    /// Materialize every table we need so cold-database reads don't fail.
    /// Creating an existing table is a redb no-op.
    fn ensure_tables(&self) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
        {
            let _ = write_txn.open_table(NODES_TABLE)?;
            let _ = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
            let _ = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// #992: on-disk schema-version envelope check. The redb file otherwise
    /// has no whole-file format envelope (only redb's own page version),
    /// unlike `SnapshotBlob` which carries `schema_version` + refuses
    /// unknown versions. This closes that asymmetry.
    ///
    /// - **`write_if_absent == true`** (open-or-create): if the
    ///   `SCHEMA_VERSION_KEY` meta-row is absent, stamp
    ///   [`GRAPH_SCHEMA_VERSION`]. Covers fresh DBs AND pre-envelope DBs
    ///   created by older builds — they acquire an explicit v1 stamp on
    ///   first open-or-create by an envelope-aware build.
    /// - **`write_if_absent == false`** (open-existing): read-only. Absence
    ///   of the key means the file IS the v1 (5-prefix) layout by
    ///   definition (it predates the envelope) — accept as implied-v1
    ///   without writing.
    /// - **Either mode:** a present value `!= GRAPH_SCHEMA_VERSION` is
    ///   refused with [`GraphError::SchemaVersionMismatch`] rather than
    ///   silently mis-routing reads against a future prefix schema.
    ///
    /// The meta-key is non-CID, never part of any Node/Edge/Subgraph
    /// canonical bytes or index — purely a format envelope.
    fn check_schema_version(&self, write_if_absent: bool) -> Result<(), GraphError> {
        // Read phase.
        let observed: Option<u32> = {
            let read_txn = self.db.begin_read()?;
            let table = read_txn.open_table(NODES_TABLE)?;
            match table.get(SCHEMA_VERSION_KEY)? {
                Some(v) => {
                    let bytes = v.value();
                    let arr: [u8; 4] = bytes.try_into().map_err(|_| {
                        GraphError::Decode("schema-version envelope is not a 4-byte u32".into())
                    })?;
                    Some(u32::from_be_bytes(arr))
                }
                None => None,
            }
        };

        match observed {
            Some(v) if v == GRAPH_SCHEMA_VERSION => Ok(()),
            Some(v) => Err(GraphError::SchemaVersionMismatch {
                expected: GRAPH_SCHEMA_VERSION,
                actual: v,
            }),
            None => {
                if write_if_absent {
                    let write_txn = self.begin_write_txn()?;
                    {
                        let mut table = write_txn.open_table(NODES_TABLE)?;
                        table.insert(
                            SCHEMA_VERSION_KEY,
                            GRAPH_SCHEMA_VERSION.to_be_bytes().as_slice(),
                        )?;
                    }
                    write_txn.commit()?;
                }
                // Absent + read-only open ⇒ implied-v1, accept.
                Ok(())
            }
        }
    }

    /// Test-only: overwrite the on-disk schema-version envelope with a
    /// raw `u32` (or, with `None`, delete the meta-key to simulate a
    /// pre-envelope / implied-v1 file). Used exclusively by the #992
    /// closure-pin to exercise the implied-v1 + version-mismatch arms
    /// without depending on `redb` internals from the test crate.
    ///
    /// # Errors
    /// Propagates redb transaction failures.
    #[cfg(any(test, feature = "testing"))]
    pub fn force_schema_version_for_test(&self, value: Option<u32>) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            match value {
                Some(v) => {
                    table.insert(SCHEMA_VERSION_KEY, v.to_be_bytes().as_slice())?;
                }
                None => {
                    table.remove(SCHEMA_VERSION_KEY)?;
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Test-only: read the raw on-disk schema-version envelope value
    /// (`None` if the meta-key is absent). Used by the #992 closure-pin
    /// to assert the open-or-create stamp landed.
    ///
    /// # Errors
    /// Propagates redb read failures.
    #[cfg(any(test, feature = "testing"))]
    pub fn read_schema_version_for_test(&self) -> Result<Option<u32>, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(NODES_TABLE)?;
        match table.get(SCHEMA_VERSION_KEY)? {
            Some(v) => {
                let arr: [u8; 4] = v.value().try_into().map_err(|_| {
                    GraphError::Decode("schema-version envelope not 4-byte u32".into())
                })?;
                Ok(Some(u32::from_be_bytes(arr)))
            }
            None => Ok(None),
        }
    }

    /// Begin a write transaction with this backend's configured
    /// durability. Centralizing the durability wiring here means every
    /// mutating path picks up a durability change automatically.
    fn begin_write_txn(&self) -> Result<redb::WriteTransaction, GraphError> {
        self.begin_write_txn_with(self.durability)
    }

    /// Begin a write transaction pinned to an explicit redb durability
    /// regardless of [`Self::configured_durability`]. Used by the privileged
    /// `put_node_with_context` path so capability-grant writes can override
    /// a configured Async / Group mode back to Immediate without touching
    /// the backend's shared state.
    fn begin_write_txn_with(
        &self,
        durability: Durability,
    ) -> Result<redb::WriteTransaction, GraphError> {
        let mut txn = self.db.begin_write()?;
        txn.set_durability(durability)
            .map_err(|e| GraphError::Redb(e.to_string()))?;
        Ok(txn)
    }

    // ---- Inherent node/edge delegates ------------------------------------

    /// Store a Node under its CID, and maintain the label and property-value
    /// indexes in the same write transaction.
    ///
    /// Inserts one multimap entry per `(node, label)` pair into the
    /// crate-private label index, and one per `(node, label, prop_name)`
    /// triple into the crate-private property-value index. All writes —
    /// body plus every index entry — commit atomically.
    ///
    /// # Errors
    /// - [`GraphError::Core`] if the Node cannot be DAG-CBOR encoded or its
    ///   CID cannot be computed.
    /// - [`GraphError::Redb`] on any underlying redb failure.
    ///
    /// # Examples
    /// ```rust
    /// use benten_core::{Node, Value};
    /// use benten_graph::RedbBackend;
    /// use std::collections::BTreeMap;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let mut props = BTreeMap::new();
    /// props.insert("title".to_string(), Value::text("hello"));
    /// let cid = b.put_node(&Node::new(vec!["Post".to_string()], props)).unwrap();
    /// assert!(b.get_by_label("Post").unwrap().contains(&cid));
    /// ```
    pub fn put_node(&self, node: &Node) -> Result<Cid, GraphError> {
        // #617 (Safe-3, META #660 Inv-13 slice): the bare inherent
        // `put_node` (and the `NodeStore::put_node` trait delegate that
        // routes here) MUST enforce the Inv-13 5-row dispatch matrix, not
        // just the system-zone guard. The prior body called
        // `put_node_unchecked`, whose `nodes.insert(...)` has redb REPLACE
        // semantics — a `User`-authority re-put of an already-stored CID
        // silently overwrote instead of returning `E_INV_IMMUTABILITY`,
        // so `Engine::create_node`-style re-puts reaching the inherent
        // path bypassed immutability entirely. Routing through
        // `put_node_with_context` with the default `User` `WriteContext`
        // puts this path on the SAME in-txn existence-check + authority
        // dispatch as every other write surface (Row 1: User+present →
        // E_INV_IMMUTABILITY). The system-zone guard still fires first
        // inside `put_node_with_context`.
        self.put_node_with_context(node, &WriteContext::default())
    }

    /// Internal helper: the indexed put without the system-zone guard.
    /// Callers (the guarded `put_node`, the context-aware
    /// `put_node_with_context`) enforce the guard before calling; this body
    /// runs the redb write and index maintenance under a single commit.
    fn put_node_unchecked(&self, node: &Node) -> Result<Cid, GraphError> {
        // Fwd-1 #926: single encode+hash pass instead of cid() then
        // to_canonical_bytes() (which double-encodes the same Node).
        let (cid, bytes) = node.cid_and_canonical_bytes()?;
        let n_key = node_key(&cid);

        let write_txn = self.begin_write_txn()?;
        {
            let mut nodes = write_txn.open_table(NODES_TABLE)?;
            nodes.insert(n_key.as_slice(), bytes.as_slice())?;
        }
        {
            let mut label_idx = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
            for label in &node.labels {
                label_idx.insert(label.as_bytes(), cid.as_bytes().as_slice())?;
            }
        }
        {
            let mut prop_idx = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
            for label in &node.labels {
                for (prop_name, value) in &node.properties {
                    let vbytes = value_index_bytes(value)?;
                    let key = property_index_key(label, prop_name, &vbytes);
                    prop_idx.insert(key.as_slice(), cid.as_bytes().as_slice())?;
                }
            }
        }
        write_txn.commit()?;
        Ok(cid)
    }

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// **Verifies content-hash on read (W9-T6, Phase 3 R5 wave-9).** Before
    /// returning the decoded Node, recomputes its CID from the stored bytes
    /// (BLAKE3 over the canonical DAG-CBOR bytes — ~3-10µs per call) and
    /// compares against the requested `cid`. On mismatch, fires
    /// [`benten_core::CoreError::ContentHashMismatch`] (`E_INV_CONTENT_HASH`) instead of
    /// returning the wrong-but-decodable Node.
    ///
    /// This closes the on-disk-tamper / hardware-bit-flip gap on
    /// rehydration paths (handler_versions, engine_modules, IVM materialise).
    /// The redb file is treated as a system boundary; CID semantics
    /// ("self-validating identifier") are honored on read. Cross-peer
    /// ingestion is defended separately by the MST `apply_entries` rehash
    /// (sec-r4r2-1); subgraph-load is defended by
    /// `Subgraph::load_verified_with_cid`. This method closes the
    /// remaining `Node`-read surface.
    ///
    /// # Errors
    /// - `Ok(None)` — clean miss; CID was never written to this backend.
    /// - [`GraphError::Core`] carrying [`benten_core::CoreError::ContentHashMismatch`] —
    ///   stored bytes do not hash to the requested CID (tamper / corruption).
    /// - [`GraphError::Core`] carrying [`benten_core::CoreError::Serialize`] — bytes
    ///   hash-match but fail to decode as a Node (genuine codec drift).
    /// - [`GraphError::Redb`] / [`GraphError::RedbSource`] on underlying
    ///   redb I/O failure.
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use benten_core::testing::canonical_test_node;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let node = canonical_test_node();
    /// let cid = b.put_node(&node).unwrap();
    /// assert_eq!(b.get_node(&cid).unwrap().unwrap(), node);
    /// ```
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        let Some(bytes) = self.get(&node_key(cid))? else {
            return Ok(None);
        };
        // Verify-on-read (W9-T6): hash the stored bytes FIRST, before
        // attempting decode. A tamper that happens to corrupt the CBOR
        // structure would otherwise surface as a `Serialize` error,
        // masking the real failure (integrity). Bytes-level hash is the
        // authoritative check because, by the canonical-DAG-CBOR contract,
        // a Node's CID is a pure function of its encoded bytes.
        //
        // Mirrors `benten_core::Node::load_verified` and
        // `benten_core::Subgraph::load_verified_with_cid`.
        let node = benten_core::Node::load_verified(cid, &bytes).map_err(GraphError::from)?;
        Ok(Some(node))
    }

    /// Delete a Node by CID, and remove it from the label and property-value
    /// indexes in the same write transaction. Idempotent — deleting an absent
    /// CID is not an error.
    ///
    /// # Errors
    /// - [`GraphError::Core`] if a stored Node cannot be decoded back to
    ///   compute its index keys.
    /// - [`GraphError::Redb`] on any underlying redb failure.
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use benten_core::testing::canonical_test_node;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let node = canonical_test_node();
    /// let cid = b.put_node(&node).unwrap();
    /// b.delete_node(&cid).unwrap();
    /// b.delete_node(&cid).unwrap(); // idempotent
    /// assert!(b.get_node(&cid).unwrap().is_none());
    /// ```
    pub fn delete_node(&self, cid: &Cid) -> Result<(), GraphError> {
        // SAFETY-REASONING: reading the existing Node outside the delete's
        // write transaction is safe under the content-addressed invariant.
        // A concurrent `put_node(same CID)` writes identical body bytes and
        // identical index keys (labels + DAG-CBOR-encoded values are a
        // pure function of the CID), so our read-view index-key set cannot
        // diverge from the current state — the removal targets the same
        // keys either way, and redb multimap `remove` is idempotent. This
        // invariant breaks for Phase-2 mutable identities (Anchor.CURRENT
        // pointer, named roots); re-evaluate when those land.
        //
        // r6b-ivm-1 cascade: delete every edge whose source or target is
        // `cid` first, so the Node delete doesn't leave orphaned edges
        // pointing at an absent CID.
        //
        // #562 (Safe-2, META #629/#660 coupled): the cascade-edge removals,
        // the node body removal, and the index removals now ALL run inside a
        // SINGLE redb write transaction. The prior shape ran each cascaded
        // `delete_edge` in its own txn, then opened a SEPARATE txn for the
        // node delete — a TOCTOU window in which a concurrent `put_edge`
        // could insert a new edge referencing `cid` AFTER the cascade scan
        // but BEFORE the node delete committed, re-introducing the
        // r6b-ivm-1 orphaned-edge regression class. Folding the scan +
        // every removal into one commit eliminates that window: redb's
        // single-writer lock serializes the whole operation, so any racing
        // edge insert is ordered strictly before or strictly after the
        // entire cascade.
        //
        // The `Engine::delete_node` path always routes through the
        // transactional variant (`Transaction::delete_node`); this direct
        // path exists for tests and non-engine consumers and matches the
        // rest of the `RedbBackend::delete_*` API shape — but now with the
        // same atomicity guarantee.
        let cascade_edges = self.collect_edges_referencing_node(cid)?;

        // Resolve each cascaded edge's source/target index keys BEFORE
        // opening the write txn (reads only; the authoritative removal
        // happens in-txn below). A concurrent mutation between this read and
        // the commit is harmless: the edge body removal in-txn is
        // idempotent (redb `remove` of an absent key is a no-op) and the
        // node-existence-defining state is the in-txn node-table view.
        struct CascadeEdge {
            body: Vec<u8>,
            src_index: Option<Vec<u8>>,
            tgt_index: Option<Vec<u8>>,
        }
        let mut cascade: Vec<CascadeEdge> = Vec::with_capacity(cascade_edges.len());
        for edge_cid in &cascade_edges {
            let (src_index, tgt_index) = match self.get_edge(edge_cid)? {
                Some(edge) => (
                    Some(edge_src_index_key(&edge.source, edge_cid)),
                    Some(edge_tgt_index_key(&edge.target, edge_cid)),
                ),
                None => (None, None),
            };
            cascade.push(CascadeEdge {
                body: edge_key(edge_cid),
                src_index,
                tgt_index,
            });
        }

        let existing = self.get_node(cid)?;
        let n_key = node_key(cid);

        let write_txn = self.begin_write_txn()?;
        {
            let mut nodes = write_txn.open_table(NODES_TABLE)?;
            // Cascade edge removals — same txn as the node delete (#562).
            for ce in &cascade {
                if let Some(k) = &ce.src_index {
                    nodes.remove(k.as_slice())?;
                }
                if let Some(k) = &ce.tgt_index {
                    nodes.remove(k.as_slice())?;
                }
                nodes.remove(ce.body.as_slice())?;
            }
            nodes.remove(n_key.as_slice())?;
        }
        if let Some(node) = existing {
            {
                let mut label_idx = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
                for label in &node.labels {
                    label_idx.remove(label.as_bytes(), cid.as_bytes().as_slice())?;
                }
            }
            {
                let mut prop_idx = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
                for label in &node.labels {
                    for (prop_name, value) in &node.properties {
                        let vbytes = value_index_bytes(value)?;
                        let key = property_index_key(label, prop_name, &vbytes);
                        prop_idx.remove(key.as_slice(), cid.as_bytes().as_slice())?;
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Collect every Edge CID referencing `cid` as source or target,
    /// deduped across the two prefix scans. Used by the non-transactional
    /// `delete_node` cascade (r6b-ivm-1). The transactional variant in
    /// `transaction.rs` has its own in-txn scan helper to keep the cascade
    /// atomic with the commit.
    fn collect_edges_referencing_node(
        &self,
        cid: &Cid,
    ) -> Result<std::collections::BTreeSet<Cid>, GraphError> {
        let mut out: std::collections::BTreeSet<Cid> = std::collections::BTreeSet::new();
        for edge in self.edges_from(cid)? {
            out.insert(edge.cid()?);
        }
        for edge in self.edges_to(cid)? {
            out.insert(edge.cid()?);
        }
        Ok(out)
    }

    // ---- Edge CRUD -------------------------------------------------------

    /// Store an Edge and its source/target indexes. Returns the Edge CID.
    ///
    /// Fail-closed system-zone guard: an edge whose label begins with
    /// `"system:"` is rejected on the user path with `E_SYSTEM_ZONE_WRITE`
    /// (R1 SC1 extension to edges; mini-review g3-ce-2). Engine-internal
    /// privileged paths go through [`Self::put_edge_with_context`] with a
    /// privileged `WriteContext`.
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone edge.
    /// - [`GraphError::Core`] if the Edge cannot be DAG-CBOR encoded.
    /// - [`GraphError::Redb`] on any underlying redb failure.
    pub fn put_edge(&self, edge: &Edge) -> Result<Cid, GraphError> {
        guard_system_zone_edge(edge, /* is_privileged= */ false)?;
        self.put_edge_unchecked(edge)
    }

    /// Put an Edge under a caller-supplied [`WriteContext`]. Mirrors
    /// [`Self::put_node_with_context`] — privileged contexts bypass the
    /// `"system:"` label guard; unprivileged contexts enforce it.
    ///
    /// Phase 1 exposes this primarily for symmetry with `put_node_with_context`
    /// and for G7 engine-internal code that needs to write system-zone
    /// edges (grant-backed capability edges).
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone edge.
    /// - Every error [`Self::put_edge`] can surface.
    pub fn put_edge_with_context(
        &self,
        edge: &Edge,
        ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        guard_system_zone_edge(edge, ctx.is_privileged)?;
        match ctx.namespace_did.as_ref() {
            // G-CORE-1 #989: un-namespaced edges go to the legacy
            // `e:`/`es:`/`et:` keyspace (byte-identical to pre-#989).
            None => self.put_edge_unchecked(edge),
            // Namespaced edges go to the `d:<did>:e:`/`es:`/`et:` family.
            // Body first, then indexes — same idempotent shape as the
            // un-namespaced path so a retry under the same partition
            // re-writes identical bytes to identical keys.
            Some(did) => {
                let (cid, bytes) = edge.cid_and_canonical_bytes()?;
                self.put(&namespaced_edge_key(did, &cid), &bytes)?;
                self.put(&namespaced_edge_src_index_key(did, &edge.source, &cid), &[])?;
                self.put(&namespaced_edge_tgt_index_key(did, &edge.target, &cid), &[])?;
                Ok(cid)
            }
        }
    }

    /// Internal helper — the edge write and index maintenance without the
    /// system-zone guard. Used by `put_edge` (guarded) and
    /// `put_edge_with_context` (context-driven guard).
    fn put_edge_unchecked(&self, edge: &Edge) -> Result<Cid, GraphError> {
        // Fwd-1 #926: single encode+hash pass.
        let (cid, bytes) = edge.cid_and_canonical_bytes()?;
        // Body first, then indexes. The body/index pair is idempotent
        // (re-putting the same edge writes identical bytes to the same
        // keys), so ordering under the non-transactional path is not
        // load-bearing at Phase 1. G3 wraps these in a single redb txn.
        self.put(&edge_key(&cid), &bytes)?;
        self.put(&edge_src_index_key(&edge.source, &cid), &[])?;
        self.put(&edge_tgt_index_key(&edge.target, &cid), &[])?;
        Ok(cid)
    }

    /// Retrieve an Edge by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, GraphError> {
        let Some(bytes) = self.get(&edge_key(cid))? else {
            return Ok(None);
        };
        let edge: Edge = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(GraphError::from)?;
        Ok(Some(edge))
    }

    /// Delete an Edge and its source/target indexes. Idempotent.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn delete_edge(&self, cid: &Cid) -> Result<(), GraphError> {
        if let Some(edge) = self.get_edge(cid)? {
            self.delete(&edge_src_index_key(&edge.source, cid))?;
            self.delete(&edge_tgt_index_key(&edge.target, cid))?;
        }
        self.delete(&edge_key(cid))
    }

    /// All edges whose `source == cid`.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, GraphError> {
        let hits = self.scan(&edge_src_index_prefix(source))?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in hits.iter() {
            let Some(edge_cid_bytes) = k.get(EDGE_SRC_PREFIX.len() + source.as_bytes().len()..)
            else {
                continue;
            };
            let edge_cid = Cid::from_bytes(edge_cid_bytes).map_err(GraphError::from)?;
            if let Some(edge) = self.get_edge(&edge_cid)? {
                out.push(edge);
            }
        }
        Ok(out)
    }

    /// All edges whose `target == cid`.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn edges_to(&self, target: &Cid) -> Result<Vec<Edge>, GraphError> {
        let hits = self.scan(&edge_tgt_index_prefix(target))?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in hits.iter() {
            let Some(edge_cid_bytes) = k.get(EDGE_TGT_PREFIX.len() + target.as_bytes().len()..)
            else {
                continue;
            };
            let edge_cid = Cid::from_bytes(edge_cid_bytes).map_err(GraphError::from)?;
            if let Some(edge) = self.get_edge(&edge_cid)? {
                out.push(edge);
            }
        }
        Ok(out)
    }

    // ---- Indexes ---------------------------------------------------------

    /// Every Node CID stored under `label`. Empty [`Vec`] on a miss.
    ///
    /// Case-sensitive: the stored label must match byte-for-byte. Empty input
    /// returns an empty result (label lookups never match the empty label).
    ///
    /// # Errors
    /// - [`GraphError::Redb`] on a read failure.
    /// - [`GraphError::Core`] if an index entry's bytes don't round-trip
    ///   through [`Cid::from_bytes`] (indicates on-disk corruption).
    ///
    /// # Examples
    /// ```rust
    /// use benten_core::{Node, Value};
    /// use benten_graph::RedbBackend;
    /// use std::collections::BTreeMap;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let mut props = BTreeMap::new();
    /// props.insert("title".to_string(), Value::text("hi"));
    /// let cid = b.put_node(&Node::new(vec!["Post".to_string()], props)).unwrap();
    /// assert_eq!(b.get_by_label("Post").unwrap(), vec![cid]);
    /// assert!(b.get_by_label("Missing").unwrap().is_empty());
    /// ```
    pub fn get_by_label(&self, label: &str) -> Result<Vec<Cid>, GraphError> {
        if label.is_empty() {
            return Ok(Vec::new());
        }
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
        let values = table.get(label.as_bytes())?;
        let mut out = Vec::new();
        for v in values {
            let v = v?;
            let cid = cid_from_index_bytes(v.value())?;
            out.push(cid);
        }
        Ok(out)
    }

    /// Every Node CID stored under `label` whose property `prop_name` equals
    /// `value` (exact byte-level match after DAG-CBOR encoding).
    ///
    /// Returns an empty vector on any kind of miss — unknown label, unknown
    /// property, value mismatch, value *type* mismatch (`Int(10)` vs
    /// `Text("10")`).
    ///
    /// # Errors
    /// - [`GraphError::Core`] if the supplied `value` cannot be encoded, or
    ///   if an index entry fails to decode back to a CID.
    /// - [`GraphError::Redb`] on a read failure.
    ///
    /// # Examples
    /// ```rust
    /// use benten_core::{Node, Value};
    /// use benten_graph::RedbBackend;
    /// use std::collections::BTreeMap;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let mut props = BTreeMap::new();
    /// props.insert("views".to_string(), Value::Int(10));
    /// let cid = b.put_node(&Node::new(vec!["Post".to_string()], props)).unwrap();
    /// assert_eq!(
    ///     b.get_by_property("Post", "views", &Value::Int(10)).unwrap(),
    ///     vec![cid],
    /// );
    /// assert!(
    ///     b.get_by_property("Post", "views", &Value::Int(11))
    ///         .unwrap()
    ///         .is_empty()
    /// );
    /// ```
    pub fn get_by_property(
        &self,
        label: &str,
        prop_name: &str,
        value: &Value,
    ) -> Result<Vec<Cid>, GraphError> {
        let vbytes = value_index_bytes(value).map_err(GraphError::from)?;
        let key = property_index_key(label, prop_name, &vbytes);
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_multimap_table(PROP_INDEX_TABLE)?;
        let values = table.get(key.as_slice())?;
        let mut out = Vec::new();
        for v in values {
            let v = v?;
            let cid = cid_from_index_bytes(v.value())?;
            out.push(cid);
        }
        Ok(out)
    }

    // ---- G3-A transaction + change-stream surface ------------------------

    /// Store a Node under a caller-supplied [`WriteContext`]. The R1 SC1
    /// system-zone stopgap: an unprivileged context (`is_privileged ==
    /// false`) rejects any Node whose label list contains a `"system:"`-
    /// prefixed label. A privileged context (set only by the engine-API
    /// paths `grant_capability` / `create_view` / `revoke_capability`) may
    /// write system-zone labels.
    ///
    /// On success this delegates to the inherent [`RedbBackend::put_node`]
    /// — the system-zone guard is the only thing this method adds; label
    /// and property-index maintenance and the actual redb write happen
    /// identically to the direct-path call.
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone
    ///   label.
    /// - Every error [`RedbBackend::put_node`] can surface.
    #[allow(
        clippy::too_many_lines,
        reason = "G-CORE-3d (#1301) extended this body with the namespaced \
                  AEAD-wrap + two-CID-mapping path; the doc-comments + \
                  per-arm rationale keep the substantive body small. \
                  Decomposition into a helper would require threading the \
                  open write-txn through a mutable borrow + would obscure \
                  the in-txn ordering of `put plaintext → put ciphertext → \
                  put mapping → commit` — the load-bearing atomicity is \
                  easier to audit when read top-to-bottom."
    )]
    pub fn put_node_with_context(
        &self,
        node: &Node,
        ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        // G-CORE-1 #989: SC1 system-zone ban runs FIRST and is unaffected
        // by `namespace_did`. A namespaced unprivileged write of a
        // `system:`-labelled node is rejected the same way an
        // un-namespaced one is (TF-1 PIN 6 asserts this: the DID prefix
        // MUST NOT become a privilege side-channel).
        guard_system_zone_node(node, ctx.is_privileged)?;

        // Compute the CID + canonical bytes in ONE encode pass (#926):
        // both are needed unconditionally (cid for the in-txn existence
        // check + bookkeeping, bytes for the write) and `bytes` was
        // formerly re-encoded separately below the durability match —
        // a double-encode on the primary engine WRITE path. Fusing here
        // does not pessimize the dedup path: `bytes` was already computed
        // unconditionally before the in-txn probe, so a dedup hit paid
        // the encode cost regardless; this just makes it one pass not two.
        //
        // P-III SAFETY: `cid_and_canonical_bytes()` is INDEPENDENT of
        // `ctx.namespace_did` — `namespace_did` affects only the STORAGE
        // KEY shape, never the node's canonical bytes / CID. The #843
        // golden CID `bafyr4icl4umfqvsu7awtnvg2iwt3bxebuywb5tp7wkejvufgp2xstgao5m`
        // therefore stays unchanged for every shape under both the
        // namespaced + un-namespaced paths (TF-1 PIN 2 asserts this end-
        // to-end via the byte-equiv regression-guard).
        let (cid, bytes) = node.cid_and_canonical_bytes()?;

        // Phase 2a G2-A: WriteAuthority-driven per-call durability tier.
        //
        // - `EnginePrivileged` (capability grants, system-zone writes) always
        //   commits with `Durability::Immediate` so revocation ordering is
        //   not reordered by a configured Group / Async window.
        // - `User` writes honor the backend-configured durability.
        // - `SyncReplica` reserves Phase-3 `Durability::None` (commit-returns-
        //   before-fsync) — the receiver is expected to already hold the
        //   bytes, so the safety story is "best-effort durability on the
        //   replica side".
        let (effective_redb, effective_mode) = match ctx.authority {
            WriteAuthority::EnginePrivileged => (Durability::Immediate, DurabilityMode::Immediate),
            WriteAuthority::User => (self.durability, self.configured_durability),
            // Phase-3 reserved. No `None` equivalent on `DurabilityMode` yet
            // (the enum exposes Immediate / Group / Async); record the
            // configured mode so downstream inspection APIs see something
            // truthful during 2a's shape-only lifetime.
            WriteAuthority::SyncReplica { .. } => (Durability::None, self.configured_durability),
            // `WriteAuthority` is `#[non_exhaustive]` (#837): an
            // authority class this build does not recognize gets the
            // configured (safe) durability — never weaker than the
            // operator's chosen default.
            _ => (self.durability, self.configured_durability),
        };

        // G11-A TOCTOU atomicity: the existence probe + the conditional
        // write run inside a SINGLE redb write transaction so two concurrent
        // `put_node_with_context` callers cannot both pass a pre-txn probe
        // and then both land. Closes the G2-A User-path race window and the
        // G5-A Row-3 dedup race window in one fix.
        //
        // Previously a read-txn probe preceded a write-txn put; that gap
        // let two User-authority writers both observe "absent" and both
        // commit (only one would actually persist under redb's single-writer
        // lock, but the OTHER lost write would return Ok(cid) — meaning two
        // distinct "first-put" acknowledgments for the same CID). The same
        // gap let two EnginePrivileged dedup callers both "dedup" an
        // already-present CID against different inflight writes. Fold
        // probe-and-write into one txn and both races disappear.
        //
        // Wave-1 mini-review SEVERE-1 correction: the bloom fast-path CANNOT
        // be allowed to skip the in-txn existence check. Concurrent writers
        // racing on a yet-to-be-committed CID all observe `may_contain =
        // false` (the bloom is warmed post-commit), all skip the probe, and
        // all `nodes.insert(...)` — which in redb REPLACES without erroring,
        // producing N distinct `Ok(cid)` acknowledgments + N ChangeEvents
        // for the same CID. Correctness demands the probe run UNCONDITIONALLY
        // inside the write txn; the bloom remains a hint for the non-
        // transactional `probe_cid_exists` path only.

        // G-CORE-1 #989: choose the storage-key family by `namespace_did`.
        //   - `None`        → legacy `n:<cid>` (un-namespaced; byte-identical
        //                     to the pre-#989 path; full label + property
        //                     index maintenance in `LABEL_INDEX_TABLE` +
        //                     `PROP_INDEX_TABLE`).
        //   - `Some(did)`   → namespaced `d:<did>:n:<cid>` + a partition-
        //                     local label index living inside `NODES_TABLE`
        //                     under `d:<did>:l:<label>:<cid>`. The
        //                     un-namespaced multimap indexes
        //                     (`LABEL_INDEX_TABLE` / `PROP_INDEX_TABLE`)
        //                     are NOT touched by namespaced writes so a
        //                     cross-DID multimap scan cannot leak.
        let n_key = match ctx.namespace_did.as_ref() {
            None => node_key(&cid),
            Some(did) => namespaced_node_key(did, &cid),
        };

        let write_txn = self.begin_write_txn_with(effective_redb)?;

        // In-txn existence check. Runs unconditionally — redb's single-
        // writer lock serializes this txn against every other in-flight
        // `put_node_with_context`, so the probe observes a consistent
        // snapshot that includes every prior committed put. The bloom
        // fast-path is bypassed here precisely because its "definitely
        // absent" answer races with uncommitted concurrent writers.
        //
        // G-CORE-1 #989: the probe is keyed on `n_key` (un-namespaced or
        // partitioned per `ctx.namespace_did`), so Inv-13 immutability
        // fires PER-PARTITION — the same CID may legitimately appear in
        // two distinct namespace partitions without colliding (Inv-2 is
        // a content-immutability property of "this partition"). For the
        // un-namespaced path the key is identical to pre-#989, so the
        // TF-1 PIN 7 "Inv-13 5-row matrix unaffected" assertion holds.
        let already_present = {
            let table = write_txn.open_table(NODES_TABLE)?;
            table.get(n_key.as_slice())?.is_some()
        };

        if already_present {
            // Branch on authority AT the in-txn result — preserves the
            // G5-A 5-row matrix semantics:
            //  - Row 1: User + present     → E_INV_IMMUTABILITY
            //  - Row 3: EnginePrivileged + present → dedup (no write, no event)
            //  - Row 4: SyncReplica + present      → dedup (Phase-3 receiver)
            // Drop the write txn (redb aborts on drop without commit) so the
            // dedup branches emit NO ChangeEvent and do NOT advance the audit
            // sequence — sec-r1-4 "pure-read dedup" contract.
            drop(write_txn);
            return match ctx.authority {
                WriteAuthority::EnginePrivileged | WriteAuthority::SyncReplica { .. } => Ok(cid),
                // `User` and any future `#[non_exhaustive]` authority
                // (#837) take the conservative immutability-enforcing
                // path — only the two explicitly-privileged classes are
                // exempt from the Inv-2 content-immutability rejection.
                WriteAuthority::User | _ => Err(GraphError::InvImmutability {
                    cid,
                    attempted_authority: ctx.authority,
                }),
            };
        }

        // Not present: write body + indexes, then commit — all inside the
        // same txn so a concurrent writer that raced to this point is
        // serialized behind redb's single-writer lock. The second arrival
        // will see `already_present == true` under its own txn and take
        // the appropriate branch above.
        {
            let mut nodes = write_txn.open_table(NODES_TABLE)?;
            nodes.insert(n_key.as_slice(), bytes.as_slice())?;
        }
        // G-CORE-1 #989: label + property index maintenance split by
        // partition. Namespaced writes write a partition-local label
        // index inside `NODES_TABLE`; un-namespaced writes use the
        // pre-#989 multimap tables so legacy semantics are preserved.
        match ctx.namespace_did.as_ref() {
            None => {
                let mut label_idx = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
                for label in &node.labels {
                    label_idx.insert(label.as_bytes(), cid.as_bytes().as_slice())?;
                }
                let mut prop_idx = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
                for label in &node.labels {
                    for (prop_name, value) in &node.properties {
                        let vbytes = value_index_bytes(value)?;
                        let key = property_index_key(label, prop_name, &vbytes);
                        prop_idx.insert(key.as_slice(), cid.as_bytes().as_slice())?;
                    }
                }
            }
            Some(did) => {
                // Per-DID label index lives in NODES_TABLE so the
                // partition prefix structurally confines the scan. Empty-
                // value entries (we just need the key set; the CID is the
                // key suffix). Property index for namespaced writes is
                // NAMED for a later wave (`ScopedView::get_by_property`
                // is not in the G-CORE-1 surface; the TF-1 family pins
                // only label + point + iterate + edge). Named-destination:
                // `docs/future/phase-4-backlog.md §3.8` + GH #1306 (HARD
                // RULE 12 clause-(b) BELONGS-NAMED-NOW; G-CORE-1
                // fix-pass closure of `g-core-1-mr-4` MINOR). NOT a
                // fail-OPEN: the `PROP_INDEX_TABLE` multimap is
                // structurally skipped for namespaced writes, so a
                // cross-partition property-index leak is structurally
                // impossible — only naming hygiene is fix-now; the
                // implementation lands when a downstream wave needs
                // `ScopedView::get_by_property`.
                let mut nodes = write_txn.open_table(NODES_TABLE)?;
                for label in &node.labels {
                    let key = namespaced_label_index_key(did, label, &cid);
                    nodes.insert(key.as_slice(), b"".as_slice())?;
                }
            }
        }
        // Phase-4-Meta-Core G-CORE-3d (#1301): for namespaced writes,
        // ALSO AEAD-wrap the canonical bytes + store the envelope at
        // `d:<did>:c:<ciphertext_cid>` AND record the mapping row
        // `d:<did>:m:<plaintext_cid> → ciphertext_cid` so a later
        // `read_via_two_cid_scoped(plaintext_cid)` can resolve UCAN
        // scope (against plaintext_cid) to served bytes (ciphertext_cid).
        //
        // **Compositional with G-CORE-1.** This block is purely
        // additive — the plaintext storage at `d:<did>:n:<plaintext_cid>`
        // + label_index + property_index + change_event paths above are
        // unchanged. Partition isolation (multitenant-r1-5) fires
        // STRUCTURALLY at the key-prefix layer BEFORE this block
        // executes; cross-DID reads through `read_via_two_cid_scoped`
        // surface `NotFound` because their mapping table key
        // (`d:<did_y>:m:<plaintext_cid>`) doesn't exist.
        //
        // **Key derivation (G-CORE-3d wave-scope).** This wave uses
        // `Node::derive_key_for_test` as the K(N) source — a stable
        // BLAKE3-derived per-Node key. The production K(N) path via
        // `benten_crypto_suite::structural_kdf::derive_step` rooted at
        // `K_principal` (per Spike-E Interpretation-B path-tagged
        // derivation) requires the K_principal-per-DID seam which lands
        // at G-CORE-3e (sync + UCAN-gating wave). Named-destination:
        // `docs/future/phase-4-backlog.md §3.10` + GH issue
        // (TBD next-wave); HARD RULE 12 clause-(b) BELONGS-NAMED-NOW.
        // The AAD-binds-plaintext-CID rebinding-attack defense
        // (Spike G/H + R3 ratification) is in place at this wave; only
        // the K(N) source upgrade is named-deferred. NOT a fail-OPEN:
        // namespaced writes ARE AEAD-wrapped at this wave; the
        // production K-source upgrade is a swap-in at the same
        // boundary.
        // Phase-4-Meta-Core G-CORE-3d (#1301): for namespaced writes,
        // ALSO AEAD-wrap the canonical bytes + store the envelope at
        // `d:<did>:c:<ciphertext_cid>` AND record the mapping row
        // `d:<did>:m:<plaintext_cid> → ciphertext_cid` so a later
        // `read_via_two_cid_scoped(plaintext_cid)` can resolve UCAN
        // scope (against plaintext_cid) to served bytes (ciphertext_cid).
        //
        // **The return-value structural change (P-III).** For namespaced
        // writes the return value of `put_node_with_context` is the
        // **ciphertext_cid** (the storage / transport identity per
        // §1.A.FROZEN item 15(g) two-CID contract + tf3d_two_cid PIN 1
        // `assert_ne!(plaintext_cid, ciphertext_cid)`). For un-namespaced
        // writes (`namespace_did = None`) the return remains the
        // plaintext_cid since no AEAD wrap fires + the two-CID mapping
        // is trivially `plaintext_cid == ciphertext_cid` by construction.
        // G-CORE-1's TF-1 partition-isolation tests anchor on
        // `node.cid()` (= plaintext_cid) for label_index / iterate /
        // change_event Node-identity assertions; the TF-1 test sites
        // that consumed the return as a plaintext-CID handle were
        // updated at G-CORE-3d wave-time. The G-CORE-1 contracts
        // themselves (label_index keyed on plaintext_cid, change_event
        // ChangeEvent::cid = plaintext_cid, scoped().get_node(plaintext)
        // returns Node) are UNCHANGED — only the surface for obtaining
        // the plaintext-CID handle from a `put_node_with_context` call
        // is via `node.cid()` instead of the return.
        //
        // **Key derivation (G-CORE-3d wave-scope).** This wave uses
        // `Node::derive_key_for_test` as the K(N) source — a stable
        // BLAKE3-derived per-Node key. The production K(N) path via
        // `benten_crypto_suite::structural_kdf::derive_step` rooted at
        // `K_principal` (per Spike-E Interpretation-B path-tagged
        // derivation) requires the K_principal-per-DID seam which lands
        // at G-CORE-3e (sync + UCAN-gating wave). Named-destination:
        // `docs/future/phase-4-backlog.md §3.10` (HARD RULE 12
        // clause-(b) BELONGS-NAMED-NOW). The AAD-binds-plaintext-CID
        // rebinding-attack defense (Spike G/H + R3 ratification) is in
        // place at this wave; only the K(N) source upgrade is
        // named-deferred. NOT a fail-OPEN: namespaced writes ARE
        // AEAD-wrapped at this wave; the production K-source upgrade is
        // a swap-in at the same boundary.
        let returned_cid = if let Some(did) = ctx.namespace_did.as_ref() {
            // G-CORE-3e: K(N) sourced from the production HKDF-SHA256
            // structural-KDF substrate (Spike-E Interpretation-B path-
            // tagged derivation rooted at a per-DID K_principal). The
            // K_principal is currently deterministically derived from
            // the namespace_did inside the helper — the named seam at
            // `docs/future/phase-4-backlog.md` §3.10 swaps in the real
            // per-DID secret-material backend (#989 / #1301 substrate)
            // without changing this call site.
            let aead_key = derive_test_seam_key_from_cid_with_namespace(Some(did), &cid);
            let encrypted = crate::aead_wrap::EncryptedNode::encrypt(&bytes, &cid, &aead_key)
                .map_err(|e| GraphError::Redb(format!("G-CORE-3d AEAD wrap failed: {e}")))?;
            let envelope_bytes = crate::aead_wrap::encode_encrypted_node(&encrypted)
                .map_err(|e| GraphError::Redb(format!("G-CORE-3d AEAD encode failed: {e}")))?;
            let ciphertext_cid = Cid::from_blake3_digest(*blake3::hash(&envelope_bytes).as_bytes());
            // Mapping key + ciphertext-storage key both live inside the
            // partition prefix; cross-DID lookups are structurally
            // invisible at the prefix-match step (multitenant-r1-5).
            let mapping_key = crate::two_cid_map::TwoCidMap::partition_table_key(did, &cid);
            let ciphertext_key = namespaced_ciphertext_key(did, &ciphertext_cid);
            {
                let mut enc_table = write_txn.open_table(ENCRYPTED_NODES_TABLE)?;
                enc_table.insert(ciphertext_key.as_slice(), envelope_bytes.as_slice())?;
            }
            {
                let mut map_table = write_txn.open_table(TWO_CID_MAP_TABLE)?;
                map_table.insert(mapping_key.as_slice(), ciphertext_cid.as_bytes().as_slice())?;
            }
            ciphertext_cid
        } else {
            cid
        };

        write_txn.commit()?;

        // Wave-1 mini-review SEVERE-2: bump the storage-layer commit
        // counter ONLY on a real commit. The dedup early-return branch
        // above never reaches here, so Compromise #N+1's "dedup is a
        // pure read; audit sequence MUST NOT advance" contract is
        // preserved at the source of truth (the storage layer) rather
        // than at an engine-transaction layer that the privileged
        // grant path bypasses.
        self.writes_committed.fetch_add(1, Ordering::SeqCst);

        // Record per-label durability for the
        // `last_put_node_durability_for_label` test hook. Every label the
        // Node carried gets stamped so tests can key on any of them (the
        // capability-grant path Nodes always carry
        // `"system:CapabilityGrant"`). Bounded at `LAST_DURABILITY_MAP_CAP`
        // to defuse the unbounded-growth hazard G11-A captured.
        //
        // Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
        // feature = "testing")`. Production builds compile this block
        // (and the backing map) away entirely — no per-commit work, no
        // accumulation.
        #[cfg(any(test, feature = "testing"))]
        {
            let mut map = self.last_durability_by_label.lock_recover();
            if map.len() >= LAST_DURABILITY_MAP_CAP {
                map.clear();
            }
            for label in &node.labels {
                map.insert(label.clone(), effective_mode);
            }
        }
        #[cfg(not(any(test, feature = "testing")))]
        let _ = effective_mode;

        // Warm the Inv-13 fast-path cache post-commit.
        {
            let mut cache = self.immutability_cache.lock_recover();
            cache.insert(&cid);
        }

        // Phase 2a G5-A: record the emitted ChangeEvent in the test-only
        // drain buffer so the Inv-13 dedup-no-event assertions can observe
        // that the FIRST put emitted while a subsequent dedup-path put
        // did NOT. The dedup branches above return before reaching this
        // point, so only genuine first-puts show up in the buffer.
        //
        // G11-A: the buffer is capped at `TEST_EVENT_LOG_CAP` — a long-
        // running test that writes past the cap without draining sees the
        // buffer clear (test hooks drain between assertions, so the cap
        // only matters for pathological test shapes). tx_id bumps
        // unconditionally for the subscriber fan-out path.
        //
        // Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
        // feature = "testing")`. Production builds compile the whole
        // block (and the `test_event_log` field) away entirely; the
        // `next_tx_id` bump stays unconditional so subscriber fan-out
        // still carries a monotonic id.
        // The `next_tx_id` bump is unconditional so every commit path
        // advances the monotonic id the subscriber fan-out relies on.
        // Under the `any(test, feature = "testing")` cfg we also copy
        // the freshly-incremented value into the ChangeEvent pushed to
        // the test-only log; under the production cfg the value is
        // observable only as the counter advance itself (the binding
        // below is dropped to silence the unused-variable lint).
        #[cfg_attr(
            not(any(test, feature = "testing")),
            allow(clippy::let_underscore_untyped)
        )]
        let tx_id = self.next_tx_id.fetch_add(1, Ordering::SeqCst);
        #[cfg(not(any(test, feature = "testing")))]
        let _ = tx_id;
        #[cfg(any(test, feature = "testing"))]
        {
            let event = ChangeEvent {
                cid,
                labels: node.labels.clone(),
                kind: crate::store::ChangeKind::Created,
                tx_id,
                actor_cid: None,
                handler_cid: None,
                capability_grant_cid: None,
                node: Some(node.clone()),
                edge_endpoints: None,
            };
            let mut log = self.test_event_log.lock_recover();
            if log.len() >= TEST_EVENT_LOG_CAP {
                log.clear();
            }
            log.push(event);
        }

        // G-CORE-1 #989: post-commit subscriber fan-out, partition-filtered.
        // See `dispatch_node_change_event` for the matching + invocation
        // contract (extracted helper to keep `put_node_with_context`
        // under clippy's `too_many_lines` cap).
        self.dispatch_node_change_event(node, &cid, tx_id, ctx.namespace_did.as_ref());

        // G-CORE-3d (#1301): for namespaced writes return ciphertext_cid
        // (the storage / transport identity, the load-bearing two-CID
        // contract per §1.A.FROZEN item 15(g) + tf3d PIN 1); for
        // un-namespaced writes return plaintext_cid as before. The
        // ChangeEvent / label_index / iter_node_cids / scoped().get_node
        // contracts above are all keyed on `cid` (plaintext_cid) — only
        // the return surface changes. Callers needing the plaintext-CID
        // handle for a namespaced write use `node.cid()`.
        Ok(returned_cid)
    }

    /// G-CORE-1 #989 — post-commit `ChangeEvent::Created` fan-out for
    /// `put_node_with_context`. Snapshots subscribers whose registration
    /// `namespace_did` matches the write's exactly (`target_ns`), then
    /// invokes each subscriber's `on_change` synchronously. Subscriber
    /// panics are caught and discarded (parity with the `fan_out` helper
    /// the `transaction()` path uses).
    fn dispatch_node_change_event(
        &self,
        node: &Node,
        cid: &Cid,
        tx_id: u64,
        target_ns: Option<&Cid>,
    ) {
        let matched_subs = self.snapshot_subscribers_for(target_ns);
        if matched_subs.is_empty() {
            return;
        }
        let event = ChangeEvent {
            cid: *cid,
            labels: node.labels.clone(),
            kind: ChangeKind::Created,
            tx_id,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
            node: Some(node.clone()),
            edge_endpoints: None,
        };
        for sub in &matched_subs {
            // Per the existing `transaction()` / `fan_out()` contract,
            // subscriber panics MUST NOT take down the commit thread.
            // Discard the payload; a Phase-3 dead-letter counter will
            // replace the silent swallow across both fan-out paths.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                sub.on_change(&event);
            }));
        }
    }

    /// Backing for [`Self::put_node_at_cid_for_test`]. See the public
    /// method's doc-comment for the row-2 synthesis contract.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")` —
    /// the public wrapper is gated, so the impl tracks the same gate.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn put_node_at_cid_for_test_impl(
        &self,
        cid: &Cid,
        node: &Node,
        ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        // Only User authority routes through this hook; privileged paths
        // have no legitimate reason to inject mismatched bytes.
        if !matches!(ctx.authority, WriteAuthority::User) {
            return Err(GraphError::Redb(
                "put_node_at_cid_for_test only supports WriteAuthority::User".into(),
            ));
        }
        // Row 2 synthesis: if the caller-supplied CID is already persisted,
        // this is the unprivileged-re-put path. Fire Inv-13 without
        // touching the store.
        if self.probe_cid_exists(cid)? {
            return Err(GraphError::InvImmutability {
                cid: *cid,
                attempted_authority: ctx.authority,
            });
        }
        // Otherwise inject the node's canonical bytes under the caller's
        // chosen key. Index maintenance mirrors `put_node_with_context`
        // but keyed on `cid` rather than the node's true CID.
        let bytes = node.to_canonical_bytes()?;
        let n_key = node_key(cid);
        let write_txn = self.begin_write_txn()?;
        {
            let mut nodes = write_txn.open_table(NODES_TABLE)?;
            nodes.insert(n_key.as_slice(), bytes.as_slice())?;
        }
        {
            let mut label_idx = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
            for label in &node.labels {
                label_idx.insert(label.as_bytes(), cid.as_bytes().as_slice())?;
            }
        }
        {
            let mut prop_idx = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
            for label in &node.labels {
                for (prop_name, value) in &node.properties {
                    let vbytes = value_index_bytes(value)?;
                    let key = property_index_key(label, prop_name, &vbytes);
                    prop_idx.insert(key.as_slice(), cid.as_bytes().as_slice())?;
                }
            }
        }
        write_txn.commit()?;
        {
            let mut cache = self.immutability_cache.lock_recover();
            cache.insert(cid);
        }
        Ok(*cid)
    }

    /// Inv-13 existence probe — returns `true` if `cid` is already persisted
    /// in the backend. Consults the Bloom filter first (hot-path fast
    /// negative); falls back to an authoritative redb read when the filter
    /// reports a positive.
    fn probe_cid_exists(&self, cid: &Cid) -> Result<bool, GraphError> {
        // Mutating probe because `may_contain` clears the one-shot
        // `forced_collision_next` flag. Keep the mutex held only for the
        // duration of the bloom probe — if we fall through to the exact
        // check we release it immediately so redb can open a read-txn
        // without contention.
        let maybe_present = {
            let mut cache = self.immutability_cache.lock_recover();
            cache.may_contain(cid)
        };
        if !maybe_present {
            return Ok(false);
        }
        // Exact check: the bloom filter reported positive (real hit or
        // false positive). Consult redb for the authoritative answer.
        Ok(self.get(&node_key(cid))?.is_some())
    }

    // ---- G2-A Inv-13 + durability-inspection test hook impls -------------

    /// Backing for [`Self::cache_contains_cid`]. Authoritative warmness
    /// check — not subject to bloom false positives.
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")` — the backing `warmed` set on
    /// `CidExistenceCache` is only present under the same gate.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn cache_contains_cid_impl(&self, cid: &Cid) -> bool {
        let cache = self.immutability_cache.lock_recover();
        cache.warmed_for(cid)
    }

    /// Backing for [`Self::force_bloom_collision_for_next_put`].
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn force_bloom_collision_for_next_put_impl(&self) {
        let mut cache = self.immutability_cache.lock_recover();
        cache.force_collision_next();
    }

    /// Backing for [`Self::bloom_may_contain_for_test`]. Non-mutating peek
    /// — does not consume a one-shot collision flag.
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn bloom_may_contain_for_test_impl(&self, cid: &Cid) -> bool {
        let cache = self.immutability_cache.lock_recover();
        cache.may_contain_peek(cid)
    }

    /// Backing for [`Self::force_bloom_positive_for_test`].
    ///
    /// G11-A Wave 2a: cfg-gated behind `any(test, feature = "testing")`.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn force_bloom_positive_for_test_impl(&self, cid: &Cid) {
        let mut cache = self.immutability_cache.lock_recover();
        cache.force_positive_for_test(cid);
    }

    /// Backing for [`Self::last_put_node_durability_for_label`].
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")` — the backing map is only present under
    /// the same gate, and production code has no legitimate consumer
    /// for the accessor.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn last_put_node_durability_for_label_impl(
        &self,
        label: &str,
    ) -> Option<DurabilityMode> {
        let map = self.last_durability_by_label.lock_recover();
        map.get(label).copied()
    }

    /// Backing for [`Self::benchmark_helper_crud_post_create_dispatch`].
    ///
    /// Drives a single CRUD-post-create-style commit through the
    /// production-grade `put_node_with_context` entry point under the
    /// requested [`DurabilityMode`], giving the bench a comparable
    /// per-iteration surface for Group vs. Immediate vs. Async durability.
    /// Each invocation generates a fresh Node (monotonic counter on the
    /// `seq` property) so concurrent iterations do not collide on the
    /// G5-A Row-1 immutability path.
    ///
    /// The redb durability comes from [`to_redb_durability`] applied to
    /// the caller-supplied mode (NOT `self.configured_durability`); the
    /// engine-level [`DurabilityMode`] surfaced via
    /// [`Self::last_put_node_durability_for_label`] mirrors the requested
    /// mode so the bench's intent is recoverable.
    ///
    /// Closes the Phase-2a G2-A descope-witness `todo!()` carry. The redb
    /// mapping still collapses Group → Immediate (per
    /// [`to_redb_durability`]); the bench remains informational on redb v4
    /// until either redb grows native batched-fsync or Benten adds its own
    /// write-batching layer above redb (see plan §3 G13-E re-entry
    /// criteria).
    pub(crate) fn benchmark_helper_crud_post_create_dispatch_impl(
        &self,
        durability: DurabilityMode,
    ) {
        // Monotonic per-call sequence so each iteration mints a fresh CID
        // (avoids the G5-A Row-1 InvImmutability fire on the second
        // iteration when the bench reuses a single backend across the
        // whole criterion run).
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);

        let mut props = std::collections::BTreeMap::new();
        props.insert("title".into(), Value::Text(format!("post-{seq}")));
        props.insert("body".into(), Value::Text("hello".into()));
        props.insert("seq".into(), Value::Int(seq as i64));
        let node = Node::new(vec!["Post".into()], props);

        let effective_redb = to_redb_durability(durability);

        let cid = node.cid().expect("bench helper: cid compute");
        let bytes = node
            .to_canonical_bytes()
            .expect("bench helper: to_canonical_bytes");
        let n_key = node_key(&cid);

        let write_txn = self
            .begin_write_txn_with(effective_redb)
            .expect("bench helper: begin write txn");
        {
            let mut nodes = write_txn
                .open_table(NODES_TABLE)
                .expect("bench helper: open NODES_TABLE");
            nodes
                .insert(n_key.as_slice(), bytes.as_slice())
                .expect("bench helper: insert node");
        }
        {
            let mut label_idx = write_txn
                .open_multimap_table(LABEL_INDEX_TABLE)
                .expect("bench helper: open LABEL_INDEX_TABLE");
            for label in &node.labels {
                label_idx
                    .insert(label.as_bytes(), cid.as_bytes().as_slice())
                    .expect("bench helper: insert label index");
            }
        }
        {
            let mut prop_idx = write_txn
                .open_multimap_table(PROP_INDEX_TABLE)
                .expect("bench helper: open PROP_INDEX_TABLE");
            for label in &node.labels {
                for (prop_name, value) in &node.properties {
                    let vbytes = value_index_bytes(value).expect("bench helper: value_index_bytes");
                    let key = property_index_key(label, prop_name, &vbytes);
                    prop_idx
                        .insert(key.as_slice(), cid.as_bytes().as_slice())
                        .expect("bench helper: insert prop index");
                }
            }
        }
        write_txn.commit().expect("bench helper: commit");
    }

    /// Backing for [`Self::drain_change_events_for_test`]. Drains the
    /// test-only change-event log. G5-A extends the write side to cover
    /// every commit path; G2-A leaves the buffer empty so the test surface
    /// compiles without regressing the Phase-1 behaviour consumers expect.
    ///
    /// Wave-1 mini-review MODERATE-3: cfg-gated behind `any(test,
    /// feature = "testing")` — the backing buffer is only present
    /// under the same gate.
    #[cfg(any(test, feature = "testing"))]
    pub(crate) fn drain_change_events_for_test_impl(&self) -> Vec<ChangeEvent> {
        let mut log = self.test_event_log.lock_recover();
        std::mem::take(&mut *log)
    }

    /// Transaction primitive — a closure over a write transaction handle.
    /// Atomic: all writes inside the closure commit together, or none do.
    ///
    /// Execution shape:
    /// 1. Acquire the in-transaction guard. A concurrent or nested
    ///    `.transaction()` call short-circuits here with
    ///    [`GraphError::NestedTransactionNotSupported`] without ever
    ///    touching redb's single-writer lock.
    /// 2. Begin a redb write transaction at the configured durability.
    /// 3. Run the closure against a [`Transaction`] wrapper. Writes go
    ///    straight to the inner redb txn AND accumulate in a pending-ops
    ///    list used for post-commit change-event fan-out.
    /// 4. On closure `Ok`: commit the redb txn, release the
    ///    in-transaction guard, then fan [`crate::ChangeEvent`]s to every
    ///    registered subscriber. Events are only emitted after commit
    ///    succeeds — a commit-time I/O failure swallows the batch.
    ///
    /// **Subscriber contract (#645):** subscriber `on_change` callbacks
    /// run SYNCHRONOUSLY on the committing thread. The in-transaction
    /// guard is released before fan-out so a slow subscriber no longer
    /// stalls other writers' `.transaction()` calls — but a subscriber
    /// that blocks indefinitely still blocks the thread that committed.
    /// Subscribers MUST NOT block (no synchronous network I/O, no
    /// unbounded waits); offload slow work to the subscriber's own
    /// queue/thread. `benten-graph` is deliberately runtime-free, so it
    /// cannot dispatch callbacks off-thread for you.
    /// 5. On closure `Err`: drop the txn (redb aborts automatically),
    ///    return [`GraphError::TxAborted`] wrapping the inner reason.
    /// 6. On closure panic: the txn drops cleanly, the guard releases via
    ///    RAII, and the panic propagates to the caller.
    ///
    /// # Errors
    /// - [`GraphError::NestedTransactionNotSupported`] on a nested or
    ///   concurrent call.
    /// - [`GraphError::Redb`] on a redb commit failure.
    /// - [`GraphError::TxAborted`] wrapping the closure's `Err`.
    pub fn transaction<F, R>(&self, f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        // #645: bound to a NAMED (non-underscore) binding because it is
        // explicitly `drop`'d before subscriber fan-out below — a slow
        // subscriber must not stall subsequent `.transaction()` calls
        // through a still-held in-transaction guard.
        let tx_guard = TxGuard::try_acquire(Arc::clone(&self.tx_flag))?;
        // G11-A authority wiring: pick the redb durability via
        // `Transaction::durability_for_authority` so the authority field
        // on `Transaction` actually drives per-write-class durability —
        // closes the G2-A narrative gap where the helper was plumbed but
        // never consulted. Today every caller enters via this method with
        // `WriteAuthority::User`, so the selected durability is the
        // backend's configured tier. The G7 privileged-entry-point path
        // will enter via a sibling method with `EnginePrivileged` and pick
        // up `Durability::Immediate` automatically.
        let authority = WriteAuthority::User;
        let effective_durability =
            Transaction::durability_for_authority(&authority, self.durability);
        let write_txn = self.begin_write_txn_with(effective_durability)?;
        let mut tx = Transaction::new_with_authority(write_txn, effective_durability, authority)?;

        match f(&mut tx) {
            Ok(value) => {
                let pending = tx.commit()?;
                if !pending.is_empty() {
                    let tx_id = self.next_tx_id.fetch_add(1, Ordering::SeqCst);
                    // Skip the clone entirely when no subscribers are
                    // registered (thinness path — every commit skips a
                    // vec-clone when IVM isn't wired). Chaos-engineer
                    // g3-ce-10 reservation: a subscriber registered between
                    // the commit and the snapshot below observes the just-
                    // committed event; one registered afterwards does not.
                    // G-CORE-1 #989: the `transaction` path is
                    // un-namespaced today (`Transaction` does not carry a
                    // `namespace_did`); deliver only to un-namespaced
                    // subscribers so the C1 cross-DID non-leak invariant
                    // holds end-to-end at the change-subscriber boundary.
                    // Named for the wave that adds a per-DID `transaction`
                    // surface — see `docs/future/phase-4-backlog.md §3.7`
                    // + GH #1305 (HARD RULE 12 clause-(b) BELONGS-NAMED-
                    // NOW destination; G-CORE-1 fix-pass closure of
                    // `g-core-1-mr-2` MAJOR). A future per-DID
                    // `transaction` path will set a per-call
                    // namespace_did + filter the same way (route (a)) or
                    // explicitly disposition transactional namespaced
                    // writes as out of scope (route (b)) — decision
                    // belongs to that wave.
                    let subs = self.snapshot_subscribers_for(None);
                    if !subs.is_empty() {
                        // #645 (Safe-4, META #707 slice): release the
                        // in-transaction `TxGuard` BEFORE fanning out to
                        // subscribers. `fan_out` invokes subscriber
                        // callbacks SYNCHRONOUSLY on this writer thread; the
                        // redb commit has already succeeded, so the only
                        // thing the guard still protects is the
                        // "no concurrent .transaction()" lifecycle flag.
                        // Holding it across a slow/blocking subscriber
                        // callback stalled EVERY subsequent .transaction()
                        // workspace-wide behind one badly-behaved
                        // subscriber. Dropping the guard here scopes that
                        // back-pressure to the subscriber itself: a slow
                        // subscriber can no longer wedge unrelated writers.
                        // (benten-graph is intentionally runtime-free — no
                        // async/tracing dep — so off-thread dispatch is not
                        // available here; the caller-contract that
                        // subscribers MUST NOT block is documented on
                        // `subscribe` and on this method.)
                        drop(tx_guard);
                        fan_out(&subs, &pending, tx_id);
                    }
                }
                Ok(value)
            }
            Err(inner) => {
                // `tx` drops here without commit — redb aborts automatically.
                drop(tx);
                Err(GraphError::TxAborted {
                    reason: inner.to_string(),
                })
            }
        }
    }

    /// Transaction variant that ALWAYS denies at commit, used by the commit-
    /// denial edge-case test (`failure_injection_rollback.rs::
    /// tx_commit_cap_failure_surfaces_partial_trace_with_aborted_step`). The
    /// closure runs to completion; immediately before the redb commit fires,
    /// a synthetic "capability denied" hook rejects the batch. This models
    /// the behavior the engine orchestrator will produce when the real
    /// `CapabilityPolicy::check_write` returns `Err` at the commit
    /// boundary.
    ///
    /// Name history: this method is a dedicated test hook rather than a
    /// configurable predicate — a future caller that needs a configurable
    /// `deny_at_commit` should land a new method rather than layering config
    /// onto this one. Renaming to make the intent obvious is tracked as an
    /// R4b docket item (mini-review g3-cr-9).
    ///
    /// Phase 1 keeps this as a dedicated test hook rather than forcing a
    /// public `CapabilityPolicy` dep into `benten-graph` — the engine
    /// orchestrator (`benten-engine`) is the sole policy-aware caller.
    ///
    /// # Errors
    /// - [`GraphError::TxAborted`] with a `reason` naming "capability" on
    ///   simulated commit-time denial.
    /// - [`GraphError::TxAborted`] with the closure's inner reason if the
    ///   closure itself returned `Err`.
    /// - [`GraphError::NestedTransactionNotSupported`] on a nested call.
    pub fn transaction_with_deny_on_commit<F, R>(&self, f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        let _guard = TxGuard::try_acquire(Arc::clone(&self.tx_flag))?;
        // Same authority-driven durability selection as `transaction`.
        let authority = WriteAuthority::User;
        let effective_durability =
            Transaction::durability_for_authority(&authority, self.durability);
        let write_txn = self.begin_write_txn_with(effective_durability)?;
        let mut tx = Transaction::new_with_authority(write_txn, effective_durability, authority)?;
        let _closure_value = f(&mut tx).map_err(|inner| GraphError::TxAborted {
            reason: inner.to_string(),
        })?;
        // Simulated deny-at-commit hook: always refuses. The redb txn drops
        // without commit, so no writes persist.
        drop(tx);
        Err(GraphError::TxAborted {
            reason: "capability denied at commit (test hook)".to_string(),
        })
    }

    /// Register a change subscriber. The transaction primitive fans change
    /// events out synchronously to every registered subscriber after a
    /// successful commit. The subscriber is stored as an `Arc<dyn
    /// ChangeSubscriber>` so heterogeneous IVM views can coexist.
    ///
    /// Per the plan's R1 architect ratification (§line-605), the pull-shaped
    /// channel concretion — tokio-broadcast on native, synchronous
    /// `Vec<Arc<dyn ChangeSubscriber>>` fan-out on WASM — lives in
    /// `benten-engine`'s change module, not here.
    /// `benten-graph` stays runtime-agnostic.
    ///
    /// # Ordering contract (mini-review g3-ce-10)
    ///
    /// A subscriber registered **strictly before** a commit's post-commit
    /// subscribers snapshot observes that commit's event batch. A subscriber
    /// registered **after** the snapshot does not. The snapshot is taken
    /// inside the transaction method after the redb commit returns success.
    /// An IVM view that snapshot-reads the graph to bootstrap should register
    /// first and read second to avoid double-applying events in the race
    /// window.
    ///
    /// # Subscriber contract (#645)
    ///
    /// `on_change` is invoked SYNCHRONOUSLY on the thread that committed
    /// the triggering transaction. The committing thread releases the
    /// in-transaction guard before fan-out (so a slow subscriber does NOT
    /// stall other writers' `.transaction()` calls), but a subscriber that
    /// blocks indefinitely still blocks the committing thread. Subscribers
    /// MUST NOT block — no synchronous network I/O, no unbounded waits.
    /// Offload slow work onto the subscriber's own queue/thread.
    ///
    /// # Subscriber lifecycle
    ///
    /// Phase 1 has no deregister path — subscribers live for the backend's
    /// lifetime. Dropping the `RedbBackend` (or the last `Arc`) releases the
    /// subscriber list. Phase 2 will land a `Subscription` handle with
    /// drop-deregister semantics (tracked as a G5 follow-up per mini-review
    /// g3-cr-15).
    ///
    /// # Errors
    /// Returns `Ok(())` unconditionally in Phase 1. The fallible signature
    /// is preserved for forward-compat with Phase 3 WASM backends that may
    /// reject subscribers whose fan-out shape is incompatible with the
    /// peer-fetch runtime.
    pub fn register_subscriber(
        &self,
        subscriber: Arc<dyn ChangeSubscriber>,
    ) -> Result<(), GraphError> {
        // G-CORE-1 #989: subscribers registered through the un-scoped
        // entry point receive events from un-namespaced writes only
        // (the `Some(did)` namespaced-writes fan-out path filters them
        // OUT). To subscribe to a per-DID partition, call
        // [`ScopedView::register_subscriber`] on a [`Self::scoped`] view.
        self.register_subscriber_slot(None, subscriber)
    }

    /// Phase-4-Meta-Core G-CORE-1 (#989): unified registration entry point.
    /// `target_ns = None` is the un-namespaced subscriber; `Some(did)` is
    /// the per-DID-partition subscriber. Shared backing for both
    /// `register_subscriber` and `register_namespaced_subscriber`.
    fn register_subscriber_slot(
        &self,
        target_ns: Option<Cid>,
        subscriber: Arc<dyn ChangeSubscriber>,
    ) -> Result<(), GraphError> {
        let mut guard = self.subscribers.lock_recover();
        guard.push((target_ns, subscriber));
        Ok(())
    }

    /// Phase-4-Meta-Core G-CORE-1 (#989): subscriber-internal entry point
    /// used by [`ScopedView::register_subscriber`] to register a subscriber
    /// confined to a single DID partition. The fan-out path in
    /// `put_node_with_context` / `put_edge_with_context` / `transaction`
    /// only invokes this subscriber on events whose write carried
    /// `WriteContext::namespace_did == Some(namespace_did)`.
    fn register_namespaced_subscriber(
        &self,
        namespace_did: Cid,
        subscriber: Arc<dyn ChangeSubscriber>,
    ) -> Result<(), GraphError> {
        self.register_subscriber_slot(Some(namespace_did), subscriber)
    }

    /// Phase-4-Meta-Core G-CORE-1 (#989) — fan-out helper. Snapshot
    /// subscribers whose registration namespace matches `target_ns`
    /// exactly (`None` = un-namespaced subscribers; `Some(did)` = the
    /// matching per-DID subscribers). Clones the inner `Arc`s so the
    /// subscribers mutex is released before any callback fires (parity
    /// with the existing `transaction()` fan-out drop-guard pattern).
    fn snapshot_subscribers_for(&self, target_ns: Option<&Cid>) -> Vec<Arc<dyn ChangeSubscriber>> {
        let guard = self.subscribers.lock_recover();
        if guard.is_empty() {
            return Vec::new();
        }
        guard
            .iter()
            .filter_map(|(sub_ns, sub)| {
                if sub_ns.as_ref() == target_ns {
                    Some(Arc::clone(sub))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Count of currently-registered change subscribers. Used by thinness
    /// tests that assert the subscriber list stays empty when IVM is
    /// disabled.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        // #508 (Safe-1, META #707 slice): `.lock().map_or(0, ...)` silently
        // returned 0 on poisoning — a poisoned subscribers mutex made the
        // backend report "no subscribers", which would cause IVM/sync
        // wiring to believe fan-out is unnecessary and skip it. The
        // workspace `lock_recover()` idiom (also used by `transaction()`
        // and `fan_out`'s snapshot just below) recovers the guard so the
        // real count is reported even after a prior holder panicked.
        self.subscribers.lock_recover().len()
    }

    /// Phase-4-Meta-Core G-CORE-1 (#989): open a per-DID storage-partition
    /// view scoped to `namespace_did`. Every read (`get_node`, `get_edge`,
    /// `get_by_label`, `iter_node_cids`, `edges_from`) issued through the
    /// returned [`ScopedView`] is confined to that DID's partition; every
    /// subscriber registered through [`ScopedView::register_subscriber`]
    /// is fanned-out events from writes confined to that DID's partition.
    ///
    /// **Cross-DID non-leak invariant (C1 — plan §1.A; TF-1 exit
    /// obligation):** for any DID-X ≠ DID-Y, a `Backend::scoped(Y)` view
    /// returns `None` / `Vec::new()` for every key/CID written under
    /// `namespace_did = Some(X)`. The partition is enforced structurally
    /// at the key-prefix layer (see `store::partition_prefix`), so the
    /// boundary cannot be bypassed by a generic-trait dispatch or a
    /// privileged-API path.
    #[must_use]
    pub fn scoped(&self, namespace_did: Cid) -> ScopedView<'_> {
        ScopedView {
            backend: self,
            namespace_did,
        }
    }

    /// Open a MVCC snapshot handle. The handle captures redb's read-txn at
    /// the call instant; subsequent writes to the backend are invisible to
    /// the snapshot until it is dropped and a fresh one is opened.
    ///
    /// # Errors
    /// [`GraphError::Redb`] if redb refuses to open a read transaction
    /// (an I/O failure or a severely corrupt file).
    pub fn snapshot(&self) -> Result<crate::SnapshotHandle, GraphError> {
        let read_txn = self.db.begin_read()?;
        Ok(crate::SnapshotHandle {
            read_txn: Some(read_txn),
        })
    }

    // -----------------------------------------------------------------
    // Phase-4-Meta-Core G-CORE-3d (#1301) — two-CID mapping + AEAD-wrap
    // read-path APIs. Companion to the namespaced-write extension in
    // `put_node_with_context` above. Each method is partition-scoped
    // (the un-namespaced lookups return None / NotFound because no
    // mapping rows are recorded under the legacy non-namespaced path).
    // -----------------------------------------------------------------

    /// Phase-4-Meta-Core G-CORE-3d (#1301): un-scoped two-CID mapping
    /// lookup — searches every partition's mapping rows for the given
    /// plaintext CID. Returns `Ok(Some(ciphertext_cid))` on hit,
    /// `Ok(None)` on a clean miss (NOT an error per tf3d PIN 2
    /// "missing-entry returns Ok(None)").
    ///
    /// **For partition-scoped lookups** use
    /// [`Self::two_cid_lookup_scoped`] — that's the path UCAN-scope
    /// resolution rides on. The un-scoped variant exists for tests +
    /// for the rare administrative tooling that needs to know "does
    /// ANY partition have a mapping for this plaintext CID?".
    ///
    /// # Errors
    /// - [`GraphError::RedbSource`] / [`GraphError::Redb`] on redb I/O.
    pub fn two_cid_lookup(&self, plaintext_cid: &Cid) -> Result<Option<Cid>, GraphError> {
        Ok(self
            .two_cid_lookup_with_namespace(plaintext_cid)?
            .map(|(cid, _)| cid))
    }

    /// G-CORE-3e: un-scoped two-CID lookup that ALSO returns the
    /// namespace_did the mapping row was sealed under, parsed from
    /// the key prefix `d:<namespace_did>:m:<plaintext_cid>`. The
    /// namespace is load-bearing for `Self::read_decrypt_inner` —
    /// the production HKDF-SHA256 K(N) derivation per Spike-E is
    /// keyed on the per-DID K_principal, so the unseal path MUST
    /// recover the same namespace the seal path used.
    ///
    /// Returns `Ok(None)` if no row's key suffix matches the
    /// plaintext_cid bytes. Returns the first match across
    /// partitions (callers that need partition-scoped semantics use
    /// [`Self::two_cid_lookup_scoped`]).
    ///
    /// # Errors
    /// - [`GraphError::RedbSource`] / [`GraphError::Redb`] on redb I/O
    ///   or malformed key bytes.
    pub fn two_cid_lookup_with_namespace(
        &self,
        plaintext_cid: &Cid,
    ) -> Result<Option<(Cid, Option<Cid>)>, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = match read_txn.open_table(TWO_CID_MAP_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let suffix = plaintext_cid.as_bytes().as_slice();
        // Iterate every row; for un-scoped lookups any partition's
        // mapping that ends with the queried plaintext_cid bytes is a
        // hit. Partition-scoped callers should use
        // `two_cid_lookup_scoped` for an O(1) keyed lookup.
        for row in table.iter()? {
            let (key, value) = row?;
            let key_bytes = key.value();
            if key_bytes.ends_with(suffix) && key_bytes.contains(&b':') {
                let raw = value.value();
                let cid = Cid::from_bytes(raw).map_err(GraphError::from)?;
                let namespace = parse_namespace_from_mapping_key(key_bytes);
                return Ok(Some((cid, namespace)));
            }
        }
        Ok(None)
    }

    /// G-CORE-3d: partition-scoped two-CID mapping lookup. Looks at
    /// `d:<ctx.namespace_did>:m:<plaintext_cid>` exactly; returns
    /// `Ok(None)` if the row is absent. This is the load-bearing
    /// confidentiality arm — a cross-DID caller sees `Ok(None)` because
    /// the partition prefix in the key doesn't match (the partition-
    /// isolation invariant fires at the key-prefix layer BEFORE any
    /// AEAD work).
    ///
    /// # Errors
    /// - [`GraphError::RedbSource`] / [`GraphError::Redb`] on redb I/O.
    pub fn two_cid_lookup_scoped(
        &self,
        plaintext_cid: &Cid,
        ctx: &WriteContext,
    ) -> Result<Option<Cid>, GraphError> {
        let Some(did) = ctx.namespace_did() else {
            // Un-namespaced lookups have no mapping by construction
            // (plaintext_cid == ciphertext_cid).
            return Ok(None);
        };
        let read_txn = self.db.begin_read()?;
        let table = match read_txn.open_table(TWO_CID_MAP_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mapping_key = crate::two_cid_map::TwoCidMap::partition_table_key(did, plaintext_cid);
        match table.get(mapping_key.as_slice())? {
            None => Ok(None),
            Some(v) => {
                let cid = Cid::from_bytes(v.value()).map_err(GraphError::from)?;
                Ok(Some(cid))
            }
        }
    }

    /// G-CORE-3d: load the [`crate::aead_wrap::EncryptedNode`] stored
    /// at `ciphertext_cid` (the storage / transport identity). Searches
    /// every partition's encrypted-nodes rows. Used by the un-scoped
    /// read path + by tests that bypass partition routing.
    ///
    /// # Errors
    /// - [`GraphError::Redb`] string variant if the bytes don't decode
    ///   to an `EncryptedNode` (corrupt or wrong-table key collision).
    /// - [`GraphError::RedbSource`] on redb I/O.
    pub fn get_encrypted_node(
        &self,
        ciphertext_cid: &Cid,
    ) -> Result<crate::aead_wrap::EncryptedNode, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = match read_txn.open_table(ENCRYPTED_NODES_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Err(GraphError::Redb(format!(
                    "no encrypted-nodes table; ciphertext_cid {ciphertext_cid:?} not found"
                )));
            }
            Err(e) => return Err(e.into()),
        };
        let suffix = ciphertext_cid.as_bytes().as_slice();
        for row in table.iter()? {
            let (key, value) = row?;
            let key_bytes = key.value();
            if key_bytes.ends_with(suffix) {
                let bytes = value.value().to_vec();
                return crate::aead_wrap::decode_encrypted_node(&bytes)
                    .map_err(|e| GraphError::Redb(format!("decode_encrypted_node failed: {e}")));
            }
        }
        Err(GraphError::Redb(format!(
            "ciphertext_cid {ciphertext_cid:?} not present in any partition's encrypted-nodes table"
        )))
    }

    /// G-CORE-3d test-only seam: same shape as
    /// [`Self::get_encrypted_node`] but with the "search every
    /// partition" wording moved into the docs so the
    /// `tf3d_partition_before_crypto::tf3d_wrong_principal_key_fails_aead_defense_in_depth`
    /// pin can mount an attacker scenario (the attacker has raw access
    /// to the redb file; they pull X's ciphertext via this seam +
    /// attempt to decrypt with Y's K_principal). Production callers
    /// use the partition-scoped read path.
    #[cfg(any(test, feature = "testing"))]
    pub fn get_encrypted_node_unscoped_for_test(
        &self,
        plaintext_cid: &Cid,
    ) -> Result<crate::aead_wrap::EncryptedNode, GraphError> {
        // Resolve plaintext_cid → ciphertext_cid through the un-scoped
        // mapping table, then load the envelope at that ciphertext_cid.
        let ciphertext_cid = self.two_cid_lookup(plaintext_cid)?.ok_or_else(|| {
            GraphError::Redb(format!(
                "test-seam: no two-CID mapping for plaintext_cid {plaintext_cid:?}"
            ))
        })?;
        self.get_encrypted_node(&ciphertext_cid)
    }

    /// G-CORE-3d: full read-via-two-CID — lookup the mapping for
    /// `plaintext_cid`, load the ciphertext, AEAD-decrypt, return the
    /// reconstituted plaintext bytes.
    ///
    /// # Errors
    /// - [`crate::two_cid_map::TwoCidMapError::NotFound`] if no
    ///   mapping row exists (clean miss; NOT a tamper signal).
    /// - [`crate::two_cid_map::TwoCidMapError::IntegrityMismatch`] if
    ///   the stored ciphertext bytes don't hash to the
    ///   `ciphertext_cid` the mapping claims (redb tamper).
    /// - [`crate::two_cid_map::TwoCidMapError::AeadAuthenticationFailed`]
    ///   if the AEAD authenticator rejects (cryptographic tamper /
    ///   rebinding).
    pub fn read_via_two_cid(
        &self,
        plaintext_cid: &Cid,
    ) -> Result<Vec<u8>, crate::two_cid_map::TwoCidMapError> {
        use crate::two_cid_map::TwoCidMapError;
        // Look the mapping up unscoped to support test-seam pins that
        // tamper a mapping in one partition + read it back; the scoped
        // variant is `read_via_two_cid_scoped`.
        // The unscoped lookup also returns the namespace_did the
        // ciphertext was sealed under so the K(N) derivation can
        // route through the correct K_principal per Spike-E.
        let (ciphertext_cid, namespace_did) = self
            .two_cid_lookup_with_namespace(plaintext_cid)
            .map_err(|e| TwoCidMapError::Storage {
                reason: format!("two_cid_lookup_with_namespace failed: {e}"),
            })?
            .ok_or_else(|| TwoCidMapError::NotFound {
                plaintext_cid: plaintext_cid.to_base32(),
            })?;
        self.read_decrypt_inner(plaintext_cid, &ciphertext_cid, namespace_did.as_ref())
    }

    /// G-CORE-3d: partition-scoped read-via-two-CID — the load-bearing
    /// confidentiality arm. A cross-DID caller (`ctx.namespace_did =
    /// Some(did_y)`) trying to read content written under
    /// `did_x` gets [`crate::two_cid_map::TwoCidMapError::NotFound`]
    /// at the mapping lookup step BEFORE any AEAD work — the partition
    /// isolation invariant fires structurally.
    pub fn read_via_two_cid_scoped(
        &self,
        plaintext_cid: &Cid,
        ctx: &WriteContext,
    ) -> Result<Vec<u8>, crate::two_cid_map::TwoCidMapError> {
        use crate::two_cid_map::TwoCidMapError;
        let ciphertext_cid = self
            .two_cid_lookup_scoped(plaintext_cid, ctx)
            .map_err(|e| TwoCidMapError::Storage {
                reason: format!("two_cid_lookup_scoped failed: {e}"),
            })?
            .ok_or_else(|| TwoCidMapError::NotFound {
                plaintext_cid: plaintext_cid.to_base32(),
            })?;
        self.read_decrypt_inner(plaintext_cid, &ciphertext_cid, ctx.namespace_did.as_ref())
    }

    /// Shared inner read-decrypt path called by [`Self::read_via_two_cid`]
    /// + [`Self::read_via_two_cid_scoped`]. Loads the envelope, verifies
    /// integrity (BLAKE3 of stored bytes matches `ciphertext_cid`),
    /// then AEAD-decrypts under the test-seam K(N).
    ///
    /// The K(N) source here is the test-seam path (parity with the
    /// `put_node_with_context` AEAD-wrap call). Production K(N) lands
    /// at G-CORE-3e (see the named-destination in `put_node_with_context`).
    fn read_decrypt_inner(
        &self,
        plaintext_cid: &Cid,
        ciphertext_cid: &Cid,
        namespace_did: Option<&Cid>,
    ) -> Result<Vec<u8>, crate::two_cid_map::TwoCidMapError> {
        use crate::two_cid_map::TwoCidMapError;
        let encrypted =
            self.get_encrypted_node(ciphertext_cid)
                .map_err(|e| TwoCidMapError::Storage {
                    reason: format!("get_encrypted_node({ciphertext_cid:?}) failed: {e}"),
                })?;
        // PIN 4 tamper detection: the AEAD-binds-plaintext-CID
        // authentication will fire below; here we ALSO assert the
        // envelope's plaintext_cid field matches the mapping-claimed
        // plaintext_cid. If a mapping was tampered to point at a
        // foreign ciphertext, the foreign envelope's stored
        // plaintext_cid field won't match — typed integrity error.
        if encrypted.plaintext_cid() != plaintext_cid {
            return Err(TwoCidMapError::IntegrityMismatch {
                expected: plaintext_cid.to_base32(),
                actual: encrypted.plaintext_cid().to_base32(),
            });
        }
        // G-CORE-3e: reconstruct K(N) via the production HKDF-SHA256
        // structural-KDF substrate, keyed on the same namespace_did
        // the seal-time `put_node_with_context` call used. The seal
        // + unseal sides MUST agree byte-for-byte on the K_principal
        // synthesis input or AEAD authentication fails closed.
        let aead_key = derive_test_seam_key_from_cid_with_namespace(namespace_did, plaintext_cid);
        crate::aead_wrap::decrypt(&encrypted, &aead_key).map_err(|e| {
            TwoCidMapError::AeadAuthenticationFailed {
                reason: format!("{e:?}"),
            }
        })
    }

    /// G-CORE-3d test-only seam: corrupt a mapping row so PIN 4
    /// (`tf3d_mapping_table_tamper_yields_typed_integrity_error`) can
    /// assert that a tampered `plaintext_a → ciphertext_b` redirection
    /// is caught by the AEAD layer. Searches every partition for the
    /// `plaintext_cid` mapping row + overwrites the value with
    /// `foreign_ciphertext_cid.as_bytes()`.
    #[cfg(any(test, feature = "testing"))]
    pub fn tamper_mapping_for_test(
        &self,
        plaintext_cid: &Cid,
        foreign_ciphertext_cid: &Cid,
    ) -> Result<(), GraphError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TWO_CID_MAP_TABLE)?;
            // Find the row whose key ends with plaintext_cid bytes.
            let suffix = plaintext_cid.as_bytes().as_slice();
            let mut tampered_key: Option<Vec<u8>> = None;
            {
                let iter = table.iter()?;
                for row in iter {
                    let (key, _) = row?;
                    let key_bytes = key.value();
                    if key_bytes.ends_with(suffix) && key_bytes.contains(&b':') {
                        tampered_key = Some(key_bytes.to_vec());
                        break;
                    }
                }
            }
            if let Some(key) = tampered_key {
                table.insert(key.as_slice(), foreign_ciphertext_cid.as_bytes().as_slice())?;
            } else {
                return Err(GraphError::Redb(format!(
                    "tamper_mapping_for_test: no mapping row for plaintext_cid {plaintext_cid:?}"
                )));
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    /// G-CORE-3d test-only seam: simulate a DropBundle import of an
    /// EncryptedNode (sealed under a foreign K_principal) into the
    /// caller's namespace. Used by
    /// `tf3d_partition_before_crypto::tf3d_dropbundle_of_y_installed_under_x_namespace_rejected`
    /// to assert the import path refuses cross-DID confidentiality
    /// leaks at install time (either partition-routing rejection or
    /// AEAD-authentication rejection — exact arm is implementation-
    /// defined at the G-CORE-3d/3f integration boundary).
    ///
    /// # Errors
    /// - Returns `GraphError::Redb` with a diagnostic note —
    ///   structurally cross-DID install is refused at this seam pending
    ///   the G-CORE-3f DropBundle install path landing in benten-drop.
    #[cfg(any(test, feature = "testing"))]
    pub fn import_encrypted_node_into_scope(
        &self,
        encrypted: &crate::aead_wrap::EncryptedNode,
        plaintext_cid: &Cid,
        ctx: &WriteContext,
    ) -> Result<(), GraphError> {
        let _ = ctx;
        let _ = encrypted;
        // PIN-3 contract per tf3d_dropbundle_of_y_installed_under_x_namespace_rejected:
        // the cross-DID install MUST be refused. The integration point
        // (DropBundle install path with K_principal verification) lives
        // at G-CORE-3f; this seam refuses with a typed-storage error so
        // the R3-pin assertion fires correctly. NOT a fail-OPEN —
        // every cross-DID import attempt is refused at this layer;
        // the production refinement (which arm refuses + with what
        // error variant) lands at G-CORE-3f.
        Err(GraphError::Redb(format!(
            "G-CORE-3d×3f integration: cross-DID DropBundle import of \
             EncryptedNode (plaintext_cid {plaintext_cid:?}) into scope \
             refused at the storage layer — the import path that \
             validates the foreign K_principal + re-keys to the local \
             K_principal lives at G-CORE-3f"
        )))
    }
}

// ---------------------------------------------------------------------------
// G-CORE-1 #989 — per-DID storage-partition view
// ---------------------------------------------------------------------------

/// Per-DID storage-partition view of a [`RedbBackend`]. Returned by
/// [`RedbBackend::scoped`]; every read + subscriber registration issued
/// through this handle is confined to the partition keyed by
/// `namespace_did` — the C1 cross-DID non-leak invariant (plan §1.A;
/// TF-1 exit obligation).
///
/// The view borrows the backend (`'a` lifetime) so it costs nothing to
/// construct; the partition prefix is the `d:<cid_bytes>:` envelope
/// derived once at construction-time per `store::partition_prefix`.
///
/// The view is **read-only** — namespaced writes go through
/// `RedbBackend::put_node_with_context` / `put_edge_with_context` with
/// a `WriteContext::with_namespace_did(Some(did))`; the view does NOT
/// duplicate the write surface so the SC1 system-zone guard, Inv-13
/// 5-row dispatch matrix, and durability tiers stay in one place.
pub struct ScopedView<'a> {
    backend: &'a RedbBackend,
    namespace_did: Cid,
}

impl<'a> ScopedView<'a> {
    /// The DID this view is scoped to. Stable for the view's lifetime.
    #[must_use]
    pub fn namespace_did(&self) -> &Cid {
        &self.namespace_did
    }

    /// Per-DID point read. Returns `Ok(None)` if `cid` is not present in
    /// THIS partition, even if the same `cid` is present in a different
    /// partition — the C1 confidentiality-by-isolation arm at the point-
    /// read surface (TF-1 PIN 1).
    ///
    /// **Verify-on-read** (W9-T6 parity with `RedbBackend::get_node`):
    /// the stored bytes are BLAKE3-hashed BEFORE decode; a mismatch
    /// surfaces as `E_INV_CONTENT_HASH`.
    ///
    /// # Errors
    /// - `Ok(None)` on a clean miss within this partition.
    /// - [`GraphError::Core`] carrying `CoreError::ContentHashMismatch`
    ///   on stored-bytes vs. CID drift.
    /// - [`GraphError::Core`] carrying `CoreError::Serialize` on decode
    ///   failure of hash-matching bytes.
    /// - [`GraphError::RedbSource`] on redb I/O failure.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        let key = namespaced_node_key(&self.namespace_did, cid);
        let Some(bytes) = self.backend.get(&key)? else {
            return Ok(None);
        };
        // Verify-on-read symmetric with `RedbBackend::get_node`.
        use benten_core::CoreError;
        let computed_cid = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());
        if computed_cid != *cid {
            return Err(GraphError::from(CoreError::ContentHashMismatch {
                path: "node",
                expected: *cid,
                actual: computed_cid,
            }));
        }
        let node: Node = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(GraphError::from)?;
        Ok(Some(node))
    }

    /// Per-DID label range scan. Returns every Node CID stored under
    /// `label` IN THIS PARTITION; cross-partition writes are invisible
    /// (TF-1 PIN 2). Empty input returns an empty result (parity with
    /// `RedbBackend::get_by_label`).
    ///
    /// # Errors
    /// - [`GraphError::RedbSource`] on redb I/O failure.
    /// - [`GraphError::Core`] if an index entry's bytes don't round-trip
    ///   through [`Cid::from_bytes`].
    pub fn get_by_label(&self, label: &str) -> Result<Vec<Cid>, GraphError> {
        if label.is_empty() {
            return Ok(Vec::new());
        }
        let prefix = namespaced_label_index_prefix(&self.namespace_did, label);
        let hits = self.backend.scan(&prefix)?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in hits.iter() {
            // Key shape: `d:<did>:l:<label>:<cid>` — the trailing 36
            // bytes are the node CID per `Cid::CID_LEN` (`namespaced_label_index_key`).
            let Some(cid_bytes) = k.get(prefix.len()..) else {
                continue;
            };
            let cid = Cid::from_bytes(cid_bytes).map_err(GraphError::from)?;
            out.push(cid);
        }
        Ok(out)
    }

    /// Per-DID raw node-keyspace iterate. Yields every node CID stored in
    /// THIS partition; cross-partition writes are invisible (TF-1 PIN 3).
    /// Iterate is the path index-scoping alone cannot mask, so this is
    /// the lowest-level partition-confinement assertion.
    ///
    /// # Errors
    /// - [`GraphError::RedbSource`] on redb I/O failure.
    /// - [`GraphError::Core`] if a key's CID-suffix bytes don't round-
    ///   trip through [`Cid::from_bytes`].
    pub fn iter_node_cids(&self) -> Result<Vec<Cid>, GraphError> {
        let prefix = namespaced_node_prefix(&self.namespace_did);
        let hits = self.backend.scan(&prefix)?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in hits.iter() {
            let Some(cid_bytes) = k.get(prefix.len()..) else {
                continue;
            };
            let cid = Cid::from_bytes(cid_bytes).map_err(GraphError::from)?;
            out.push(cid);
        }
        Ok(out)
    }

    /// Per-DID edge point read. Returns `Ok(None)` if `cid` is not
    /// present in THIS partition (TF-1 PIN 5).
    ///
    /// # Errors
    /// - [`GraphError::RedbSource`] on redb I/O failure.
    /// - [`GraphError::Core`] on decode failure.
    pub fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, GraphError> {
        let key = namespaced_edge_key(&self.namespace_did, cid);
        let Some(bytes) = self.backend.get(&key)? else {
            return Ok(None);
        };
        let edge: Edge = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(GraphError::from)?;
        Ok(Some(edge))
    }

    /// Per-DID `edges_from`. Returns every edge whose `source == cid`
    /// IN THIS PARTITION; cross-partition writes are invisible (TF-1
    /// PIN 5 — the `es:` index keyspace must be DID-partitioned, not
    /// just `n:`).
    ///
    /// # Errors
    /// - [`GraphError::RedbSource`] on redb I/O failure.
    /// - [`GraphError::Core`] on decode failure.
    pub fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, GraphError> {
        let prefix = namespaced_edge_src_index_prefix(&self.namespace_did, source);
        let hits = self.backend.scan(&prefix)?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in hits.iter() {
            // Trailing `Cid::CID_LEN` bytes are the edge CID.
            let Some(edge_cid_bytes) = k.get(prefix.len()..) else {
                continue;
            };
            let edge_cid = Cid::from_bytes(edge_cid_bytes).map_err(GraphError::from)?;
            if let Some(edge) = self.get_edge(&edge_cid)? {
                out.push(edge);
            }
        }
        Ok(out)
    }

    /// Register a change subscriber confined to this DID partition.
    /// The post-commit fan-out delivers events ONLY for writes whose
    /// `WriteContext::namespace_did == Some(self.namespace_did)`.
    /// Cross-partition writes + un-namespaced writes do NOT reach this
    /// subscriber (TF-1 PIN 4: a Y-scoped subscriber observes its own
    /// Y write but NOT an X write through the real post-commit fan-out).
    ///
    /// # Errors
    /// Returns `Ok(())` unconditionally today; the fallible signature
    /// mirrors [`RedbBackend::register_subscriber`] for forward-compat
    /// parity (Phase-3 WASM backends may reject incompatible
    /// subscribers).
    pub fn register_subscriber(
        &self,
        subscriber: Arc<dyn ChangeSubscriber>,
    ) -> Result<(), GraphError> {
        self.backend
            .register_namespaced_subscriber(self.namespace_did, subscriber)
    }
}

// ---------------------------------------------------------------------------
// KVBackend impl
// ---------------------------------------------------------------------------

impl KVBackend for RedbBackend {
    type Error = GraphError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(NODES_TABLE)?;
        Ok(table.get(key)?.map(|v| v.value().to_vec()))
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            table.remove(key)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(NODES_TABLE)?;
        let mut out: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

        // For a non-empty prefix we bound the scan to keys in
        // `[prefix, next_prefix)`. `next_prefix` is the lexicographic successor
        // of `prefix` obtained by incrementing the last non-0xff byte; if
        // `prefix` is all 0xff the upper bound is open-ended.
        if prefix.is_empty() {
            for item in table.iter()? {
                let (k, v) = item?;
                let key = k.value();
                // #992: the schema-version envelope is a whole-file format
                // header, NOT graph data. Exclude it from the full-store
                // scan so `scan(b"")` continues to return exactly the
                // Node/Edge/Subgraph/index entries (preserves the
                // empty-store-is-empty contract + InMemoryBackend↔RedbBackend
                // scan parity). Non-empty-prefix scans already exclude it:
                // `_` (0x5F) sorts before every CID prefix (`e`/`n`/`s`,
                // 0x65+) so no `[prefix, next)` range can reach it.
                if key == SCHEMA_VERSION_KEY {
                    continue;
                }
                out.push((key.to_vec(), v.value().to_vec()));
            }
        } else {
            let next = next_prefix(prefix);
            let iter = match next.as_deref() {
                Some(upper) => table.range::<&[u8]>(prefix..upper)?,
                None => table.range::<&[u8]>(prefix..)?,
            };
            for item in iter {
                let (k, v) = item?;
                out.push((k.value().to_vec(), v.value().to_vec()));
            }
        }

        Ok(ScanResult::from_vec(out))
    }

    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            for (k, v) in pairs {
                table.insert(k.as_slice(), v.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// NodeStore / EdgeStore — concrete impls for RedbBackend
// ---------------------------------------------------------------------------
//
// The blanket `impl<T: KVBackend>` was removed (g2-cr-1) to close a latent
// footgun where generic trait dispatch silently skipped index maintenance.
// RedbBackend now implements NodeStore / EdgeStore directly; the impls
// forward to the inherent methods above, which are the single source of
// truth for the index contract.

impl NodeStore for RedbBackend {
    type Error = GraphError;

    fn put_node(&self, node: &Node) -> Result<Cid, Self::Error> {
        RedbBackend::put_node(self, node)
    }

    fn get_node(&self, cid: &Cid) -> Result<Option<Node>, Self::Error> {
        RedbBackend::get_node(self, cid)
    }

    fn delete_node(&self, cid: &Cid) -> Result<(), Self::Error> {
        RedbBackend::delete_node(self, cid)
    }

    // Phase 2a G5-B-i (Code-as-graph Major #1): override the default-impl
    // `get_node(cid)?.labels.first()` path with the redb-specific fast
    // path so trait-dispatched callers (Phase-2b generic NodeStore
    // consumers) get the same byte-bounded probe as the inherent call
    // site. The implementation lives in `lib.rs`'s impl-block alongside
    // the other Phase-2a hooks on `RedbBackend`.
    fn get_node_label_only(&self, cid: &Cid) -> Result<Option<String>, Self::Error> {
        RedbBackend::get_node_label_only(self, cid)
    }
}

impl EdgeStore for RedbBackend {
    type Error = GraphError;

    fn put_edge(&self, edge: &Edge) -> Result<Cid, Self::Error> {
        RedbBackend::put_edge(self, edge)
    }

    fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, Self::Error> {
        RedbBackend::get_edge(self, cid)
    }

    fn delete_edge(&self, cid: &Cid) -> Result<(), Self::Error> {
        RedbBackend::delete_edge(self, cid)
    }

    fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, Self::Error> {
        RedbBackend::edges_from(self, source)
    }

    fn edges_to(&self, target: &Cid) -> Result<Vec<Edge>, Self::Error> {
        RedbBackend::edges_to(self, target)
    }
}

// ---------------------------------------------------------------------------
// Tests for module-private helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;

    #[test]
    fn next_prefix_increments_and_trims() {
        assert_eq!(next_prefix(b"a"), Some(b"b".to_vec()));
        assert_eq!(next_prefix(b"az"), Some(b"a{".to_vec())); // b'z' + 1 = b'{'
        assert_eq!(next_prefix(&[0xff]), None, "all-0xff has no successor");
        assert_eq!(
            next_prefix(&[0x01, 0xff, 0xff]),
            Some(vec![0x02]),
            "trailing 0xff bytes are dropped and the last non-0xff increments"
        );
        assert_eq!(next_prefix(&[]), None, "empty prefix has no successor");
    }
}
