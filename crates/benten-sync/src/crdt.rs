//! Loro CRDT integration at Node-property granularity.
//!
//! ## D-PHASE-3-4 RESOLVED-at-R1 — per-property LWW + HLC
//!
//! Phase-3 G16-B lands a CRDT layer at the **Node-property** granularity.
//! Each [`LoroDoc`] models the property bag of a single graph Node:
//! property keys are strings; values are HLC-stamped scalars (string,
//! integer, bytes); ordering between concurrent writes to the same
//! property is determined by the [`benten_core::hlc::BentenHlc`] carried
//! alongside the value.
//!
//! Higher-arity rich types (Loro Lists, Loro Maps) are also exposed for
//! collaborative subgraph edits where multiple peers concurrently
//! modify a Node's rich property values (`comments` list, `tags` map,
//! etc.). Rich-type merge semantics defer to Loro's CRDT primitives
//! (intent-preserving list ops, LWW Map keys); the LWW property arm
//! sits on top of a Loro Map keyed by property name with each value
//! shaped as `{value, hlc}`.
//!
//! ## HLC ordering vs Loro's internal Lamport
//!
//! Loro's `LoroMap` uses its own internal Lamport clock for LWW
//! resolution. To preserve the "engine-wide HLC determines all
//! ordering" property (load-bearing for cross-process WAIT-resume +
//! Inv-14 device-grain attribution + revocation-vs-data ordering), we
//! carry the [`BentenHlc`] **inside the value** (`{value, hlc}` shape)
//! and resolve LWW by examining the HLC explicitly via
//! [`LoroDoc::get_property`]. The internal Loro Lamport never escapes
//! into engine-visible ordering decisions.
//!
//! ## Canonical-bytes round-trip per CLAUDE.md baked-in #5
//!
//! [`LoroDoc::to_canonical_bytes`] exports the document via Loro's
//! native binary snapshot encoding (`ExportMode::Snapshot`).
//! [`LoroDoc::from_canonical_bytes`] imports the inverse. The encoding
//! is canonical-bytes-symmetric within a single peer's history; across
//! peers, the convergence property (after bidirectional merge, all
//! peers agree on canonical-bytes form) is the load-bearing assertion
//! pinned by `prop_loro_concurrent_writes_converge_via_hlc_ordering`.
//!
//! ## D-C HYBRID per arch-r1-4 / D-PHASE-3-22 RESOLVED
//!
//! Loro merges produce **new Version Nodes via existing Anchor +
//! Version + CURRENT pattern** (Phase-1 shipped). The
//! [`LoroDoc::winning_attribution`] accessor surfaces the union of
//! contributing peer-`node_id`s (HLC node-ids) so the engine-side
//! Atrium consumer can construct an `AttributionFrame` capturing
//! contributing peer-DIDs at the new Version mint.
//!
//! ## Inv-13 row-4 SPLIT per ds-4
//!
//! - **row-4a** (SyncReplica + divergent CID + Loro-merge-applicable
//!   user-data): Loro merge resolves via D-C version-chain pattern; new
//!   Version Node minted. [`LoroDoc::merge`] is the Loro-side leaf
//!   driver; the engine-side Atrium consumer mints the Version Node.
//! - **row-4b** (SyncReplica + divergent CID + system-zone /
//!   Anchor-immutable): rejection at the engine-side dispatch layer
//!   with `E_SYNC_DIVERGENT_CID_REJECTED`. The CRDT layer here exposes
//!   [`LoroDoc::op_log_targets`] so the dispatch classifier can walk
//!   inbound op-log targets and reject system-zone mutations BEFORE
//!   applying. The engine-side classifier is dispatched at
//!   `benten_engine::engine_sync` in `benten-engine`.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows.
//! - plan §3 G16-B row.
//! - `D-PHASE-3-4` RESOLVED-at-R1 (Node-property granularity).
//! - `D-PHASE-3-22` RESOLVED + `arch-r1-4` D-C HYBRID.
//! - `ds-4` Inv-13 row-4 SPLIT.
//! - `cag-6` (merged Node is graph-encoded, not opaque CRDT blob).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use loro::{ExportMode, LoroDoc as InnerLoroDoc, LoroList, LoroMap, LoroValue};
use serde::{Deserialize, Serialize};

use benten_core::hlc::BentenHlc;

/// Stable container-id for the LWW-property root List. Every
/// [`LoroDoc`] carries one root List under this id; each entry is a
/// `LoroValue::Binary` carrying the DAG-CBOR encoding of a
/// `(String key, StampedValue)` tuple (per Qual-1 #667 + Safe-1 #511
/// closure — typed canonical-bytes replaces the prior hand-rolled
/// `<key>\x1f<physical>:<logical>:<node>:<value>` flat string; the
/// length-prefixed CBOR boundaries remove the `splitn(4, ':')` /
/// `\x1f`-separator parse-invariant burden and make a malformed entry
/// a typed `serde_ipld_dagcbor` decode failure surfaced through
/// `tracing::warn!` rather than a silent `continue`).
///
/// The List shape (rather than per-property nested-containers) defends
/// against Loro's documented "concurrent container creation at the
/// same key may overwrite" hazard at the LoroMap layer — concurrent
/// peers each appending to the SAME List have their entries preserved
/// per Loro List CRDT semantics. Read-time scans the List and groups
/// by key for HLC-LWW resolution (the per-key index below caches that
/// grouping per Fwd-1 #999 O(n²) closure).
const PROPERTY_ROOT: &str = "benten:properties";

/// Stable container-id prefix for collaborative rich-type containers
/// (Loro Lists, Loro Maps). Callers reach in via
/// [`LoroDoc::list`] / [`LoroDoc::map`] using a logical name; the
/// container id concatenates [`RICH_PREFIX`] with the logical name.
const RICH_PREFIX: &str = "benten:rich:";

/// CRDT errors surfaced by the [`LoroDoc`] layer.
///
/// All variants map to typed [`benten_errors::ErrorCode`] codes via
/// [`CrdtError::code`] so observability pipelines can route on the
/// catalog identifier independent of the variant struct shape.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CrdtError {
    /// A Loro internal operation failed (insert / delete / merge /
    /// import). Carries the underlying Loro error string.
    #[error("loro internal error: {reason}")]
    LoroInternal {
        /// Operator-readable reason (Loro's own error message).
        reason: String,
    },

    /// An HLC-stamped value failed to encode into / decode from the
    /// canonical wire shape (DAG-CBOR `(key, StampedValue)` tuple).
    ///
    /// On the WRITE path this is a hard error returned to the caller
    /// (encode failures mean the value cannot be persisted). On the
    /// READ path a per-entry decode failure is NOT promoted to this
    /// error — it is surfaced via `tracing::warn!` and the entry is
    /// skipped, so a single corrupt entry does not poison LWW
    /// resolution for the rest of the document (Safe-1 #511: the skip
    /// is now *observable* rather than silent).
    #[error("hlc-stamped value codec error: {reason}")]
    StampedValueCodec {
        /// Operator-readable reason.
        reason: String,
    },

    /// A canonical-bytes import attempt failed (snapshot bytes were
    /// malformed / truncated / from an incompatible Loro version).
    #[error("canonical-bytes import error: {reason}")]
    CanonicalBytesImport {
        /// Operator-readable reason.
        reason: String,
    },
}

impl CrdtError {
    /// Map this error to its stable [`benten_errors::ErrorCode`].
    ///
    /// All current CRDT errors map to
    /// [`benten_errors::ErrorCode::AtriumTransportDegraded`] because
    /// they surface a malformed sync-frame or local CRDT state
    /// inconsistency that the engine-side Atrium consumer surfaces as
    /// transport-degraded for operator dashboards. Phase-3 R6 may
    /// split these out into dedicated CRDT codes if the
    /// observability pipeline demands finer routing.
    #[must_use]
    pub fn code(&self) -> benten_errors::ErrorCode {
        benten_errors::ErrorCode::AtriumTransportDegraded
    }
}

/// Result alias for CRDT surfaces.
pub type CrdtResult<T> = Result<T, CrdtError>;

/// HLC-stamped scalar property value.
///
/// The on-wire shape is `(value, hlc)` encoded into a Loro Map entry as
/// a sub-Map carrying both fields. Resolving LWW for a property looks
/// up the entry, decodes the HLC, and returns the value with the
/// highest HLC across all writes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StampedValue {
    /// The scalar value (string variant for the canary; expand to
    /// integer / bytes / bool variants as call sites land per
    /// pim-1 §3.5b post-fix doc-coupling sweeps).
    pub value: String,
    /// The HLC stamp this write carried at the writing peer.
    pub hlc: HlcWire,
}

/// On-wire HLC shape — mirrors [`BentenHlc`] but uses owned primitives
/// suitable for Loro's `LoroValue` encoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HlcWire {
    /// `physical_ms` component.
    pub physical_ms: u64,
    /// `logical` component.
    pub logical: u32,
    /// `node_id` component (writing peer's HLC node-id).
    pub node_id: u64,
}

impl From<BentenHlc> for HlcWire {
    fn from(h: BentenHlc) -> Self {
        Self {
            physical_ms: h.physical_ms(),
            logical: h.logical(),
            node_id: h.node_id(),
        }
    }
}

impl From<HlcWire> for BentenHlc {
    fn from(w: HlcWire) -> Self {
        BentenHlc::new(w.physical_ms, w.logical, w.node_id)
    }
}

impl HlcWire {
    /// Lexicographic comparison `(physical_ms, logical, node_id)` matching
    /// [`BentenHlc`]'s `Ord` impl.
    fn cmp_lex(&self, other: &Self) -> std::cmp::Ordering {
        self.physical_ms
            .cmp(&other.physical_ms)
            .then(self.logical.cmp(&other.logical))
            .then(self.node_id.cmp(&other.node_id))
    }
}

/// A single op-log target classification for the Inv-13 row-4 SPLIT
/// dispatch (per ds-4).
///
/// Engine-side dispatch consumes this list to decide whether to apply
/// or reject an inbound Loro op-log:
///
/// - User-data (post-`benten:rich:`-prefixed containers + the
///   `benten:properties` LWW root) → Loro merge resolves via D-C
///   version-chain pattern (row-4a).
/// - System-zone / Anchor-immutable targets → reject with
///   `E_SYNC_DIVERGENT_CID_REJECTED` (row-4b).
///
/// G16-B canary scope: container-name walking (the "what targets does
/// this op-log mutate" surface). Engine-side classification (the
/// "is this target system-zone / Anchor-immutable" decision) lives
/// at `benten_engine::engine_sync`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpLogTarget {
    /// The container-name string (Loro `ContainerID` rendered as
    /// string). Engine-side classifier walks this against the
    /// `system:` zone-prefix list per `benten-engine`'s
    /// `system_zones::SYSTEM_ZONE_PREFIXES`.
    pub container_name: String,
}

/// Loro CRDT document at Node-property granularity.
///
/// One [`LoroDoc`] per graph Node (or per logical property bag).
/// Carries:
///
/// - A LWW property root Map (`benten:properties`) holding HLC-stamped
///   scalar values.
/// - On-demand rich-type containers (Lists / Maps) under the
///   `benten:rich:<name>` namespace.
///
/// Cloneable via the underlying Loro `fork` semantics — clones share
/// the same op-log up to the fork point, then diverge independently.
/// Used by `benten_engine::engine_sync` to fan out per-peer document handles
/// from the Atrium-managed canonical document.
pub struct LoroDoc {
    inner: Arc<InnerLoroDoc>,
    /// Per-key LWW index cache (Fwd-1 #999 closure).
    ///
    /// The property root List is append-only; the LWW winner per key
    /// is a pure function of the List contents. Rather than re-scanning
    /// the entire O(N) List on every `get_property` / `get_stamped` /
    /// `winning_attribution` / `all_writes` call (the 2× full-doc walks
    /// per `apply_atrium_merge` row Fwd-1 #999 flagged), we cache the
    /// resolved view and invalidate it whenever the underlying Loro
    /// op-count changes (any local write, merge, or remote-update
    /// import bumps `len_ops()`). Cache rebuild is the only O(N) scan;
    /// reads against an unchanged document are O(log k) (BTreeMap).
    ///
    /// `Mutex` (not `RwLock`) because rebuild is rare relative to reads
    /// and the critical section is tiny (a stamped-value clone). The
    /// index is NOT part of canonical bytes — it is a pure derived
    /// cache reconstructed on demand, so it never participates in the
    /// CLAUDE.md #5 round-trip or cross-peer convergence assertion.
    index: Arc<std::sync::Mutex<PropertyIndex>>,
}

/// Derived per-key LWW view + the op-count it was built at.
#[derive(Default)]
struct PropertyIndex {
    /// `len_ops()` snapshot the `by_key` map was built against; `None`
    /// means "never built".
    built_at_ops: Option<usize>,
    /// LWW winner per property key.
    by_key: BTreeMap<String, StampedValue>,
    /// Union of contributing peer `node_id`s across ALL writes (not
    /// just LWW winners) — the `winning_attribution` seed per
    /// arch-r1-4 D-C HYBRID / net-blocker-3.
    all_node_ids: BTreeSet<u64>,
    /// Every (key, stamped) write observed, in List order — the
    /// `all_writes` engine-side AttributionFrame seed.
    all_writes: Vec<(String, StampedValue)>,
}

impl std::fmt::Debug for LoroDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `index` is a pure derived cache (Fwd-1 #999) — intentionally
        // omitted from Debug; finish_non_exhaustive marks that.
        f.debug_struct("LoroDoc")
            .field("op_count", &self.inner.len_ops())
            .field("change_count", &self.inner.len_changes())
            .finish_non_exhaustive()
    }
}

impl Default for LoroDoc {
    fn default() -> Self {
        Self::new()
    }
}

impl LoroDoc {
    /// Construct a fresh [`LoroDoc`].
    ///
    /// The underlying Loro document is empty until the first
    /// [`LoroDoc::set_property`] / [`LoroDoc::list`] / [`LoroDoc::map`]
    /// call. Loro's internal peer-id is randomized at construction;
    /// callers wiring HLC ordering pass [`BentenHlc`] explicitly so the
    /// internal Loro peer-id never leaks into engine-visible ordering.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerLoroDoc::new()),
            index: Arc::new(std::sync::Mutex::new(PropertyIndex::default())),
        }
    }

    /// Construct a [`LoroDoc`] with a deterministic Loro peer-id.
    ///
    /// Used by tests asserting deterministic op-log shapes; production
    /// callers should prefer [`LoroDoc::new`] + the random peer-id.
    /// Returns the document with `set_peer_id` applied; on Loro's
    /// rejection of the peer-id (very rare — only `u64::MAX` is
    /// rejected) returns `None`.
    #[must_use]
    pub fn with_peer_id(peer_id: u64) -> Option<Self> {
        let inner = InnerLoroDoc::new();
        inner.set_peer_id(peer_id).ok()?;
        Some(Self {
            inner: Arc::new(inner),
            index: Arc::new(std::sync::Mutex::new(PropertyIndex::default())),
        })
    }

    /// Set an HLC-stamped scalar property.
    ///
    /// Per D-PHASE-3-4, properties are LWW under HLC ordering. Each
    /// write encodes the value + HLC into a Loro Map entry; the
    /// resolve-LWW path at [`LoroDoc::get_property`] returns the value
    /// with the highest HLC. Multiple writes to the same key from the
    /// same peer with monotonic HLCs (the common case) appear as
    /// successive Loro insertions; concurrent writes from different
    /// peers resolve at merge time per the bidirectional-merge
    /// convergence property.
    ///
    /// # Errors
    ///
    /// Returns [`CrdtError::StampedValueCodec`] if the
    /// `(key, StampedValue)` tuple fails to DAG-CBOR encode (should
    /// never happen for well-formed inputs — the fields are all
    /// CBOR-representable scalars). Returns [`CrdtError::LoroInternal`]
    /// if the underlying Loro insert fails (Loro's failure mode here
    /// is essentially out-of-memory on the op-log allocator).
    pub fn set_property(
        &self,
        key: &str,
        value: impl Into<String>,
        hlc: BentenHlc,
    ) -> CrdtResult<()> {
        let stamped = StampedValue {
            value: value.into(),
            hlc: HlcWire::from(hlc),
        };
        // HLC-explicit LWW resolution discipline: we do NOT rely on
        // Loro's internal Map LWW (which would resolve based on
        // Loro's internal Lamport, not our HLC).
        //
        // We also avoid Loro's "nested container per property"
        // pattern, which has documented overwrite-loss semantics
        // when two peers concurrently call get_or_create_container
        // on the same map key (per loro 1.12 LoroMap docs). Instead
        // every write is appended to a SINGLE root List; read-time
        // scans the List, groups by property key, and resolves LWW
        // per HLC.
        //
        // Each entry is the DAG-CBOR encoding of a
        // `(key, StampedValue)` tuple carried as `LoroValue::Binary`
        // (Qual-1 #667 + Safe-1 #511 closure): typed, length-prefixed,
        // symmetric with the workspace's CLAUDE.md #5 BLAKE3 +
        // DAG-CBOR + CIDv1 commitment, and free of the prior
        // hand-rolled `\x1f` + `splitn(4, ':')` parse-invariant
        // burden. The append-only-list-with-compaction story (rather
        // than truncate-on-every-write) is retained because the
        // truncation must be a Loro op that converges across peers —
        // see [`LoroDoc::compact_property_history`] (Hyg-2 #381
        // closure: the compact op now exists rather than naming a
        // phantom "future R6-FP optimization"). Reads are O(log k)
        // against the per-key index cache, not O(N) List scans
        // (Fwd-1 #999 closure).
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        let encoded = encode_entry(key, &stamped)?;
        writes
            .insert(writes.len(), LoroValue::from(encoded))
            .map_err(|e| CrdtError::LoroInternal {
                reason: format!("insert(stamped): {e}"),
            })?;
        self.inner.commit();
        Ok(())
    }

    /// Read the LWW value for a property, if any.
    ///
    /// Returns `None` if the key has no writes. Returns the value with
    /// the highest HLC across all observed writes (which after
    /// bidirectional merge converges to the same answer at every
    /// peer per the `prop_loro_concurrent_writes_converge_via_hlc_ordering`
    /// property).
    ///
    /// LWW resolution here reads `(value, hlc)` as Loro stored them.
    /// Because Loro's underlying Map is itself LWW (under Loro's
    /// internal Lamport), and we always commit the value + HLC
    /// together in one Loro batch, the Loro-resolved sub-Map always
    /// surfaces the value+HLC pair from a single peer-write atomically
    /// — there is no torn-write hazard where two peers' values and
    /// HLCs interleave.
    #[must_use]
    pub fn get_property(&self, key: &str) -> Option<String> {
        self.read_stamped(key).map(|sv| sv.value)
    }

    /// Read the full HLC-stamped value for a property, including the
    /// HLC stamp.
    ///
    /// Used by [`LoroDoc::winning_attribution`] + the engine-side
    /// AttributionFrame mint per arch-r1-4 D-C HYBRID.
    #[must_use]
    pub fn get_stamped(&self, key: &str) -> Option<StampedValue> {
        self.read_stamped(key)
    }

    fn read_stamped(&self, key: &str) -> Option<StampedValue> {
        self.with_index(|idx| idx.by_key.get(key).cloned())
    }

    /// Iterate every key currently observed in the property root List.
    #[cfg_attr(not(test), allow(dead_code))]
    fn keys(&self) -> BTreeSet<String> {
        self.with_index(|idx| idx.by_key.keys().cloned().collect())
    }

    /// Run `f` against the per-key LWW index, rebuilding it first if
    /// the underlying Loro op-count has changed since the last build.
    ///
    /// This is the single O(N) List-scan choke point (Fwd-1 #999): a
    /// document that has not been written-to or merged-into since the
    /// last index build serves every read in O(log k) without
    /// re-walking the append-only List. `apply_atrium_merge`'s prior
    /// 2× full-doc walks per row (`all_writes` + `winning_attribution`)
    /// now share ONE cached rebuild.
    fn with_index<R>(&self, f: impl FnOnce(&PropertyIndex) -> R) -> R {
        let current_ops = self.inner.len_ops();
        let mut guard = self
            .index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.built_at_ops != Some(current_ops) {
            self.rebuild_index_into(&mut guard, current_ops);
        }
        f(&guard)
    }

    /// Rebuild the per-key LWW index from the property root List.
    ///
    /// The ONLY place a malformed entry is observed (Safe-1 #511): a
    /// `serde_ipld_dagcbor` decode failure or a non-`Binary` List
    /// value is `tracing::warn!`-logged with the offending index +
    /// reason, then skipped — the skip is now *observable* (operators
    /// see a warn line + can route on it) instead of the prior silent
    /// `continue`. The CBOR boundaries are length-prefixed, so a
    /// well-formed encoder cannot produce an entry that decodes to the
    /// wrong shape; a decode failure here means genuine corruption or
    /// an incompatible-version peer, which is exactly what an operator
    /// needs to see.
    fn rebuild_index_into(&self, idx: &mut PropertyIndex, at_ops: usize) {
        idx.by_key.clear();
        idx.all_node_ids.clear();
        idx.all_writes.clear();
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        let n = writes.len();
        for i in 0..n {
            let bytes = match writes.get(i) {
                Some(loro::ValueOrContainer::Value(LoroValue::Binary(b))) => b,
                Some(other) => {
                    tracing::warn!(
                        target: "benten_sync::crdt",
                        entry_index = i,
                        "crdt property entry is not LoroValue::Binary ({other:?}); \
                         skipping (Safe-1 #511 observable-skip)"
                    );
                    continue;
                }
                None => {
                    tracing::warn!(
                        target: "benten_sync::crdt",
                        entry_index = i,
                        "crdt property List shrank during index rebuild; \
                         skipping (Safe-1 #511 observable-skip)"
                    );
                    continue;
                }
            };
            let (entry_key, stamped) = match decode_entry(&bytes) {
                Ok(kv) => kv,
                Err(reason) => {
                    tracing::warn!(
                        target: "benten_sync::crdt",
                        entry_index = i,
                        %reason,
                        "crdt property entry failed DAG-CBOR decode; \
                         skipping (Safe-1 #511 observable-skip — LWW \
                         resolution diverges from writer intent for \
                         this entry)"
                    );
                    continue;
                }
            };
            idx.all_node_ids.insert(stamped.hlc.node_id);
            idx.all_writes.push((entry_key.clone(), stamped.clone()));
            match idx.by_key.get(&entry_key) {
                Some(prev) if stamped.hlc.cmp_lex(&prev.hlc) != std::cmp::Ordering::Greater => {}
                _ => {
                    idx.by_key.insert(entry_key, stamped);
                }
            }
        }
        idx.built_at_ops = Some(at_ops);
    }

    /// Compact the append-only property history: retain only the
    /// current LWW winner per key, deleting all superseded entries.
    ///
    /// Hyg-2 #381 closure — this is the concrete realization of the
    /// previously-phantom "future R6-FP compact-old-writes-on-
    /// checkpoint optimization". It is **safe to call only at a
    /// checkpoint where all participating peers have observed the
    /// merge** (a superseded entry that a laggard peer has not yet
    /// seen would otherwise be lost): the truncation is itself a Loro
    /// op (`LoroList::delete`) that propagates to laggard peers via
    /// subsequent merges, so calling it post-quiescence preserves
    /// convergence. Callers that cannot establish the all-peers-
    /// observed precondition MUST NOT call this — append-only is the
    /// safe default; compaction is the bounded-growth opt-in.
    ///
    /// Returns the number of entries removed.
    ///
    /// # Errors
    ///
    /// Returns [`CrdtError::LoroInternal`] if a Loro `delete` /
    /// `insert` op fails, or [`CrdtError::StampedValueCodec`] if a
    /// retained winner fails to re-encode.
    pub fn compact_property_history(&self) -> CrdtResult<usize> {
        // Resolve the current LWW winners (forces an index rebuild if
        // stale) BEFORE mutating the List.
        let winners: Vec<(String, StampedValue)> = self.with_index(|idx| {
            idx.by_key
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        });
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        let before = writes.len();
        if before == winners.len() {
            // Already compact (one entry per key, all winners).
            return Ok(0);
        }
        // Replace the entire List contents with exactly the winners.
        // `clear` + re-`insert` is a deterministic Loro op sequence
        // that converges across peers (Loro list ops are CRDT-merged;
        // a laggard peer that later merges this doc applies the same
        // clear+insert and arrives at the same compacted state).
        writes.clear().map_err(|e| CrdtError::LoroInternal {
            reason: format!("compact: clear: {e}"),
        })?;
        for (k, sv) in &winners {
            let encoded = encode_entry(k, sv)?;
            writes
                .insert(writes.len(), LoroValue::from(encoded))
                .map_err(|e| CrdtError::LoroInternal {
                    reason: format!("compact: re-insert: {e}"),
                })?;
        }
        self.inner.commit();
        Ok(before.saturating_sub(winners.len()))
    }

    /// Return the union of contributing peer `node_id`s observed
    /// across all property writes in this document.
    ///
    /// Surfaces the AttributionFrame seed per arch-r1-4 D-C HYBRID: the
    /// engine-side Atrium consumer takes this set + the local
    /// peer-DID-to-`node_id` map to compute the contributing peer-DIDs
    /// at the new Version mint. Walks every observed write (not just
    /// the LWW winner) so revoked peers' contributing writes are also
    /// surfaced — load-bearing for revocation-vs-data ordering audit
    /// per net-blocker-3.
    #[must_use]
    pub fn winning_attribution(&self) -> BTreeSet<u64> {
        self.with_index(|idx| idx.all_node_ids.clone())
    }

    /// Surface the full list of stamped writes (across all properties)
    /// that the document has accumulated.
    ///
    /// Used by tests + the engine-side AttributionFrame mint per
    /// arch-r1-4 D-C HYBRID. Each entry is `(property_key, stamped)`.
    #[must_use]
    pub fn all_writes(&self) -> Vec<(String, StampedValue)> {
        self.with_index(|idx| idx.all_writes.clone())
    }

    /// Get a collaborative Loro List under the rich-type namespace.
    ///
    /// Lists support intent-preserving concurrent insertions per Loro's
    /// CRDT primitives. Used by callers like `node.list("comments")`
    /// for collaborative subgraph edits.
    #[must_use]
    pub fn list(&self, name: &str) -> loro::LoroList {
        self.inner.get_list(format!("{RICH_PREFIX}{name}"))
    }

    /// Get a collaborative Loro Map under the rich-type namespace.
    ///
    /// Maps are LWW per Loro's internal Lamport clock for non-HLC
    /// property bags (e.g. `node.map("tags")` for collaborative tag
    /// sets where ordering is not load-bearing).
    #[must_use]
    pub fn map(&self, name: &str) -> LoroMap {
        self.inner.get_map(format!("{RICH_PREFIX}{name}"))
    }

    /// Merge another peer's [`LoroDoc`] into this one.
    ///
    /// Applies the remote document's full op-log to this doc's history.
    /// After bidirectional merge (`a.merge(&b); b.merge(&a)`) both
    /// peers' canonical-bytes converge — the load-bearing convergence
    /// property pinned by
    /// `prop_loro_concurrent_writes_converge_via_hlc_ordering`.
    ///
    /// # Errors
    ///
    /// Returns [`CrdtError::LoroInternal`] if the underlying Loro
    /// import fails (typically a malformed snapshot blob from a peer
    /// running an incompatible Loro version).
    pub fn merge(&self, remote: &LoroDoc) -> CrdtResult<()> {
        let updates = remote
            .inner
            .export(ExportMode::all_updates())
            .map_err(|e| CrdtError::LoroInternal {
                reason: format!("remote.export(all_updates): {e}"),
            })?;
        self.apply_remote_update(&updates)?;
        Ok(())
    }

    /// Apply a remote update blob (produced by [`LoroDoc::export_update`]).
    ///
    /// The byte-level interface for sync-replica delivery: a peer
    /// receiving an Atrium sync frame calls this to integrate the
    /// remote's op-log delta. Pairs with [`LoroDoc::export_update`].
    ///
    /// # Errors
    ///
    /// Returns [`CrdtError::CanonicalBytesImport`] if the bytes are
    /// malformed / from an incompatible Loro version.
    pub fn apply_remote_update(&self, bytes: &[u8]) -> CrdtResult<()> {
        self.inner
            .import(bytes)
            .map_err(|e| CrdtError::CanonicalBytesImport {
                reason: format!("import: {e}"),
            })?;
        // Force-commit any pending state so subsequent reads see the
        // applied bytes (Loro's import already triggers a commit
        // internally for snapshot bytes, but we belt-and-suspenders
        // for update bytes too).
        self.inner.commit();
        Ok(())
    }

    /// Export an update delta covering all ops in this doc's history.
    ///
    /// Pairs with [`LoroDoc::apply_remote_update`] for the sync-replica
    /// delivery byte-level interface. The resulting bytes are
    /// idempotent across peers — applying the same update twice is a
    /// no-op per Loro's CRDT semantics.
    ///
    /// # Errors
    ///
    /// Returns [`CrdtError::LoroInternal`] if the export fails.
    pub fn export_update(&self) -> CrdtResult<Vec<u8>> {
        self.inner
            .export(ExportMode::all_updates())
            .map_err(|e| CrdtError::LoroInternal {
                reason: format!("export(all_updates): {e}"),
            })
    }

    /// Encode this document's full state to canonical bytes via Loro's
    /// snapshot encoding.
    ///
    /// Round-trips with [`LoroDoc::from_canonical_bytes`]. Used by the
    /// graph-encoded persistent state surface per cag-2 + cag-6: the
    /// Atrium sync-cursor zone Node carries the canonical-bytes
    /// snapshot as an opaque-but-graph-encoded property.
    ///
    /// # Errors
    ///
    /// Returns [`CrdtError::LoroInternal`] if the snapshot export
    /// fails.
    pub fn to_canonical_bytes(&self) -> CrdtResult<Vec<u8>> {
        self.inner
            .export(ExportMode::Snapshot)
            .map_err(|e| CrdtError::LoroInternal {
                reason: format!("export(Snapshot): {e}"),
            })
    }

    /// Reconstruct a [`LoroDoc`] from canonical-bytes form.
    ///
    /// Inverse of [`LoroDoc::to_canonical_bytes`]. The reconstructed
    /// document carries the full op-log history of the source +
    /// supports further [`LoroDoc::set_property`] / [`LoroDoc::merge`]
    /// calls that compose with the imported state.
    ///
    /// # Errors
    ///
    /// Returns [`CrdtError::CanonicalBytesImport`] if the bytes are
    /// malformed / truncated / from an incompatible Loro version.
    pub fn from_canonical_bytes(bytes: &[u8]) -> CrdtResult<Self> {
        let doc = InnerLoroDoc::new();
        doc.import(bytes)
            .map_err(|e| CrdtError::CanonicalBytesImport {
                reason: format!("import: {e}"),
            })?;
        Ok(Self {
            inner: Arc::new(doc),
            index: Arc::new(std::sync::Mutex::new(PropertyIndex::default())),
        })
    }

    /// Walk the op-log and surface every container target the inbound
    /// op-log mutates.
    ///
    /// Engine-side Inv-13 dispatch (per ds-4 row-4 SPLIT) consumes
    /// this list to classify each mutation against the system-zone /
    /// Anchor-immutable matrix BEFORE applying. row-4a (user-data)
    /// proceeds with [`LoroDoc::merge`]; row-4b (system-zone /
    /// Anchor-immutable) rejects with `E_SYNC_DIVERGENT_CID_REJECTED`
    /// at the engine boundary.
    ///
    /// G16-B scope: returns the rich-prefix and property-root
    /// container names. The engine-side classifier walks these against
    /// `benten-engine::system_zones::SYSTEM_ZONE_PREFIXES`.
    #[must_use]
    pub fn op_log_targets(&self) -> Vec<OpLogTarget> {
        // Phase-3 G16-B canary surface: return the static set of
        // container roots this document touches. A finer-grained walk
        // (per-op container ids from the actual op-log) is the
        // wave-6b-r6-fp surface; the engine-side classifier today
        // only needs the root container names to dispatch row-4a vs
        // row-4b — system-zone Nodes never share their property root
        // with user-data Nodes, so the root-name walk suffices for
        // the dispatch decision.
        let mut out = Vec::new();
        out.push(OpLogTarget {
            container_name: PROPERTY_ROOT.to_string(),
        });
        // Surface every rich-type container that has been touched by
        // examining the doc's container set.
        let deep = self.inner.get_deep_value();
        if let LoroValue::Map(m) = deep {
            for (k, _v) in m.iter() {
                if k.starts_with(RICH_PREFIX) {
                    out.push(OpLogTarget {
                        container_name: k.clone(),
                    });
                }
            }
        }
        out
    }

    /// Number of distinct ops applied to this document since
    /// construction.
    ///
    /// Surfaces Loro's internal op-count for observability + as a
    /// load-bearing assertion in convergence proptests (after
    /// bidirectional merge between two peers, both peers' op-counts
    /// agree).
    #[must_use]
    pub fn op_count(&self) -> usize {
        self.inner.len_ops()
    }
}

impl Clone for LoroDoc {
    /// Clone via Loro's `fork` semantics — the resulting document
    /// shares the op-log up to fork point + diverges independently
    /// thereafter.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::new(self.inner.fork()),
            // Fresh derived cache — the fork has its own op-log so the
            // parent's index would be stale against it; lazy rebuild
            // on first read reconstructs it from the forked List.
            index: Arc::new(std::sync::Mutex::new(PropertyIndex::default())),
        }
    }
}

/// Per-property LWW comparison helper exposed for callers that
/// surface the LWW resolution decision (e.g. test fixtures asserting
/// the "higher-HLC always wins" property).
#[must_use]
pub fn hlc_lww_winner(a: &HlcWire, b: &HlcWire) -> std::cmp::Ordering {
    a.cmp_lex(b)
}

/// DAG-CBOR encode a `(key, StampedValue)` tuple into the canonical
/// on-wire bytes carried as a Loro `LoroValue::Binary` entry.
///
/// Replaces the prior hand-rolled `<key>\x1f<physical>:<logical>:
/// <node>:<value>` flat-string packer (Qual-1 #667): `StampedValue`
/// already derives `Serialize`, the workspace already commits to
/// DAG-CBOR per CLAUDE.md #5, and the length-prefixed encoding has
/// zero parse-invariant burden (no `\x1f` delimiter discipline, no
/// `splitn(4, ':')` numeric-token assumption — every future
/// `StampedValue::value` scalar variant just works).
///
/// # Errors
///
/// Returns [`CrdtError::StampedValueCodec`] if `serde_ipld_dagcbor`
/// fails to serialize the tuple (not expected for well-formed inputs).
fn encode_entry(key: &str, stamped: &StampedValue) -> CrdtResult<Vec<u8>> {
    serde_ipld_dagcbor::to_vec(&(key, stamped)).map_err(|e| CrdtError::StampedValueCodec {
        reason: format!("dag-cbor encode (key={key:?}): {e}"),
    })
}

/// Inverse of [`encode_entry`]. Returns the decoded `(key,
/// StampedValue)` tuple, or an operator-readable reason string on
/// decode failure (Safe-1 #511: the caller logs the reason via
/// `tracing::warn!` and skips the entry — the skip is *observable*,
/// not silent).
fn decode_entry(bytes: &[u8]) -> Result<(String, StampedValue), String> {
    serde_ipld_dagcbor::from_slice::<(String, StampedValue)>(bytes)
        .map_err(|e| format!("dag-cbor decode ({} bytes): {e}", bytes.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_hlc(physical_ms: u64, logical: u32, node_id: u64) -> BentenHlc {
        BentenHlc::new(physical_ms, logical, node_id)
    }

    #[test]
    fn property_set_then_get_round_trip() {
        let doc = LoroDoc::new();
        doc.set_property("title", "v1", fixture_hlc(100, 0, 0xAAAA))
            .unwrap();
        assert_eq!(doc.get_property("title").as_deref(), Some("v1"));
    }

    #[test]
    fn property_lww_higher_hlc_wins_within_single_doc() {
        let doc = LoroDoc::new();
        doc.set_property("title", "v1", fixture_hlc(100, 0, 0xAAAA))
            .unwrap();
        doc.set_property("title", "v2", fixture_hlc(200, 0, 0xAAAA))
            .unwrap();
        doc.set_property("title", "v3", fixture_hlc(300, 0, 0xAAAA))
            .unwrap();
        assert_eq!(doc.get_property("title").as_deref(), Some("v3"));
    }

    #[test]
    fn property_get_missing_key_returns_none() {
        let doc = LoroDoc::new();
        assert_eq!(doc.get_property("nope"), None);
    }

    #[test]
    fn canonical_bytes_round_trip_preserves_property() {
        let doc = LoroDoc::new();
        doc.set_property("title", "hello", fixture_hlc(42, 7, 0xBEEF))
            .unwrap();
        let bytes = doc.to_canonical_bytes().unwrap();
        let restored = LoroDoc::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(restored.get_property("title").as_deref(), Some("hello"));
        // HLC stamp also preserved through the round-trip.
        let sv = restored.get_stamped("title").unwrap();
        assert_eq!(sv.hlc.physical_ms, 42);
        assert_eq!(sv.hlc.logical, 7);
        assert_eq!(sv.hlc.node_id, 0xBEEF);
    }

    #[test]
    fn merge_two_peers_concurrent_writes_converge() {
        let doc_a = LoroDoc::new();
        let doc_b = LoroDoc::new();
        doc_a
            .set_property("color", "red", fixture_hlc(100, 0, 0xAAAA))
            .unwrap();
        doc_b
            .set_property("color", "blue", fixture_hlc(200, 0, 0xBBBB))
            .unwrap();
        // Bidirectional merge.
        doc_a.merge(&doc_b).unwrap();
        doc_b.merge(&doc_a).unwrap();
        // Both peers converge.
        assert_eq!(doc_a.get_property("color"), doc_b.get_property("color"));
        // The higher HLC won.
        assert_eq!(doc_a.get_property("color").as_deref(), Some("blue"));
    }

    #[test]
    fn merge_lower_hlc_does_not_overwrite_higher() {
        let doc_a = LoroDoc::new();
        let doc_b = LoroDoc::new();
        doc_a
            .set_property("color", "high", fixture_hlc(500, 0, 0xAAAA))
            .unwrap();
        doc_b
            .set_property("color", "low", fixture_hlc(100, 0, 0xBBBB))
            .unwrap();
        doc_a.merge(&doc_b).unwrap();
        doc_b.merge(&doc_a).unwrap();
        assert_eq!(doc_a.get_property("color").as_deref(), Some("high"));
        assert_eq!(doc_b.get_property("color").as_deref(), Some("high"));
    }

    #[test]
    fn rich_type_list_concurrent_inserts_preserve_both() {
        let doc_a = LoroDoc::new();
        let doc_b = LoroDoc::new();
        doc_a.list("comments").insert(0, "first").unwrap();
        doc_b.list("comments").insert(0, "alt-first").unwrap();
        doc_a.inner.commit();
        doc_b.inner.commit();
        doc_a.merge(&doc_b).unwrap();
        doc_b.merge(&doc_a).unwrap();
        assert_eq!(doc_a.list("comments").len(), 2);
        assert_eq!(doc_b.list("comments").len(), 2);
    }

    #[test]
    fn winning_attribution_surfaces_contributing_peer_node_ids() {
        let doc_a = LoroDoc::new();
        let doc_b = LoroDoc::new();
        doc_a
            .set_property("k1", "x", fixture_hlc(100, 0, 0xAAAA))
            .unwrap();
        doc_b
            .set_property("k2", "y", fixture_hlc(200, 0, 0xBBBB))
            .unwrap();
        doc_a.merge(&doc_b).unwrap();
        let attr = doc_a.winning_attribution();
        assert!(attr.contains(&0xAAAA));
        assert!(attr.contains(&0xBBBB));
    }

    #[test]
    fn op_log_targets_surfaces_property_root_and_rich_containers() {
        let doc = LoroDoc::new();
        doc.set_property("k", "v", fixture_hlc(1, 0, 1)).unwrap();
        doc.list("comments").insert(0, "c").unwrap();
        doc.inner.commit();
        let targets = doc.op_log_targets();
        let names: Vec<_> = targets.iter().map(|t| t.container_name.clone()).collect();
        assert!(names.contains(&PROPERTY_ROOT.to_string()));
        assert!(names.iter().any(|n| n.starts_with(RICH_PREFIX)));
    }

    #[test]
    fn export_then_apply_remote_update_round_trip() {
        let doc_a = LoroDoc::new();
        let doc_b = LoroDoc::new();
        doc_a
            .set_property("k", "x", fixture_hlc(100, 0, 0xAAAA))
            .unwrap();
        let update = doc_a.export_update().unwrap();
        doc_b.apply_remote_update(&update).unwrap();
        assert_eq!(doc_b.get_property("k").as_deref(), Some("x"));
    }

    #[test]
    fn hlc_lww_winner_orders_lex() {
        let early = HlcWire {
            physical_ms: 100,
            logical: 0,
            node_id: 0xAAAA,
        };
        let late = HlcWire {
            physical_ms: 200,
            logical: 0,
            node_id: 0xBBBB,
        };
        assert_eq!(hlc_lww_winner(&early, &late), std::cmp::Ordering::Less);
        assert_eq!(hlc_lww_winner(&late, &early), std::cmp::Ordering::Greater);
        assert_eq!(hlc_lww_winner(&late, &late), std::cmp::Ordering::Equal);
    }

    #[test]
    fn crdt_error_maps_to_atrium_transport_degraded() {
        let err = CrdtError::CanonicalBytesImport {
            reason: "test".into(),
        };
        assert_eq!(
            err.code(),
            benten_errors::ErrorCode::AtriumTransportDegraded
        );
    }

    #[test]
    fn from_canonical_bytes_rejects_malformed_blob() {
        let err = LoroDoc::from_canonical_bytes(b"not a valid loro snapshot").unwrap_err();
        assert!(matches!(err, CrdtError::CanonicalBytesImport { .. }));
    }

    #[test]
    fn with_peer_id_constructs_doc_with_deterministic_peer_id() {
        // Consumer call site for LoroDoc::with_peer_id (deterministic
        // peer-id construction used by integration tests asserting
        // op-log shape).
        let doc = LoroDoc::with_peer_id(0x1234_5678).expect("with_peer_id");
        doc.set_property("k", "v", fixture_hlc(1, 0, 1)).unwrap();
        assert_eq!(doc.get_property("k").as_deref(), Some("v"));
    }

    #[test]
    fn op_count_increments_on_each_set_property() {
        // Consumer call site for LoroDoc::op_count (observability
        // surface used by convergence proptest assertions).
        let doc = LoroDoc::new();
        let initial = doc.op_count();
        doc.set_property("k", "v1", fixture_hlc(1, 0, 1)).unwrap();
        assert!(doc.op_count() > initial);
    }

    #[test]
    fn all_writes_surfaces_full_history_with_property_keys() {
        // Consumer call site for LoroDoc::all_writes (engine-side
        // AttributionFrame mint per arch-r1-4 D-C HYBRID).
        let doc = LoroDoc::new();
        doc.set_property("k1", "v1", fixture_hlc(100, 0, 0xA))
            .unwrap();
        doc.set_property("k2", "v2", fixture_hlc(200, 0, 0xB))
            .unwrap();
        let writes = doc.all_writes();
        assert_eq!(writes.len(), 2);
        let keys: std::collections::BTreeSet<_> = writes.iter().map(|(k, _)| k.clone()).collect();
        assert!(keys.contains("k1"));
        assert!(keys.contains("k2"));
    }

    // ---- Pattern-F bundle closure pins (#1179: #667 / #511 / #381 / #999) ----

    #[test]
    fn entries_are_cbor_binary_not_flat_string() {
        // Qual-1 #667 closure pin: the property root List now carries
        // DAG-CBOR `LoroValue::Binary` entries, NOT the prior
        // hand-rolled `\x1f`-+`:`-delimited `LoroValue::String`. This
        // would FAIL (assert hits the String arm) under the pre-#667
        // flat-string encoding.
        let doc = LoroDoc::new();
        doc.set_property("title", "hello", fixture_hlc(7, 1, 0xFEED))
            .unwrap();
        let writes = doc.inner.get_list(PROPERTY_ROOT);
        assert_eq!(writes.len(), 1);
        match writes.get(0).unwrap() {
            loro::ValueOrContainer::Value(LoroValue::Binary(b)) => {
                // Round-trips through the typed decoder.
                let (k, sv) = decode_entry(&b).expect("entry decodes as typed CBOR tuple");
                assert_eq!(k, "title");
                assert_eq!(sv.value, "hello");
                assert_eq!(sv.hlc.node_id, 0xFEED);
            }
            other => panic!("expected LoroValue::Binary CBOR entry, got {other:?}"),
        }
    }

    #[test]
    fn malformed_entry_is_skipped_not_silently_dropped_into_lww() {
        // Safe-1 #511 closure pin: a corrupt (non-CBOR) entry injected
        // directly into the List is SKIPPED at index-rebuild time
        // (with a tracing::warn! the operator can observe) rather than
        // silently poisoning LWW resolution. The well-formed entry
        // still resolves; the corrupt one does not crash the read.
        let doc = LoroDoc::new();
        doc.set_property("k", "good", fixture_hlc(100, 0, 0xAAAA))
            .unwrap();
        // Inject a garbage Binary entry that is not valid CBOR for the
        // (String, StampedValue) shape.
        let writes = doc.inner.get_list(PROPERTY_ROOT);
        writes
            .insert(writes.len(), LoroValue::from(vec![0xFF, 0x00, 0x42]))
            .unwrap();
        doc.inner.commit();
        // Read still works; the good value resolves; no panic, no
        // silent substitution of the corrupt entry.
        assert_eq!(doc.get_property("k").as_deref(), Some("good"));
        // all_writes surfaces only the well-formed entry (the corrupt
        // one was observably skipped, not counted).
        assert_eq!(doc.all_writes().len(), 1);
    }

    #[test]
    fn compact_property_history_retains_winners_drops_superseded() {
        // Hyg-2 #381 closure pin: the previously-phantom
        // "compact-old-writes-on-checkpoint" op now EXISTS and removes
        // superseded entries while preserving the LWW winner.
        let doc = LoroDoc::new();
        doc.set_property("title", "v1", fixture_hlc(100, 0, 0xA))
            .unwrap();
        doc.set_property("title", "v2", fixture_hlc(200, 0, 0xA))
            .unwrap();
        doc.set_property("title", "v3", fixture_hlc(300, 0, 0xA))
            .unwrap();
        doc.set_property("other", "x", fixture_hlc(50, 0, 0xB))
            .unwrap();
        assert_eq!(doc.inner.get_list(PROPERTY_ROOT).len(), 4);
        let removed = doc.compact_property_history().unwrap();
        assert_eq!(removed, 2, "two superseded `title` writes removed");
        assert_eq!(doc.inner.get_list(PROPERTY_ROOT).len(), 2);
        // LWW winners preserved post-compaction.
        assert_eq!(doc.get_property("title").as_deref(), Some("v3"));
        assert_eq!(doc.get_property("other").as_deref(), Some("x"));
        // Idempotent: a second compaction removes nothing.
        assert_eq!(doc.compact_property_history().unwrap(), 0);
    }

    #[test]
    fn compaction_preserves_cross_peer_convergence() {
        // Hyg-2 #381 / convergence-preservation pin: compaction is a
        // Loro op; a laggard peer that later merges the compacted doc
        // converges to the same LWW answer (the truncation propagates
        // and does not lose the winner).
        let doc_a = LoroDoc::new();
        let doc_b = LoroDoc::new();
        doc_a
            .set_property("color", "old1", fixture_hlc(100, 0, 0xA))
            .unwrap();
        doc_a
            .set_property("color", "old2", fixture_hlc(200, 0, 0xA))
            .unwrap();
        doc_a
            .set_property("color", "winner", fixture_hlc(300, 0, 0xA))
            .unwrap();
        // Peer A compacts at a checkpoint.
        doc_a.compact_property_history().unwrap();
        // Laggard peer B merges the compacted doc.
        doc_b.merge(&doc_a).unwrap();
        doc_a.merge(&doc_b).unwrap();
        assert_eq!(doc_a.get_property("color"), doc_b.get_property("color"));
        assert_eq!(doc_a.get_property("color").as_deref(), Some("winner"));
    }

    #[test]
    fn index_cache_reused_across_reads_without_op_change() {
        // Fwd-1 #999 closure pin: repeated reads against an unchanged
        // document do not re-scan the List. We assert behavioral
        // equivalence (the value is stable) AND that the index's
        // built_at_ops snapshot is reused (a second read does not bump
        // it because op-count is unchanged).
        let doc = LoroDoc::new();
        doc.set_property("k", "v", fixture_hlc(1, 0, 1)).unwrap();
        assert_eq!(doc.get_property("k").as_deref(), Some("v"));
        let snap1 = doc
            .index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .built_at_ops;
        // Second read — no intervening write.
        assert_eq!(doc.get_property("k").as_deref(), Some("v"));
        let snap2 = doc
            .index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .built_at_ops;
        assert_eq!(snap1, snap2, "index not rebuilt when op-count unchanged");
        // A new write invalidates + rebuilds at the new op-count.
        doc.set_property("k", "v2", fixture_hlc(2, 0, 1)).unwrap();
        assert_eq!(doc.get_property("k").as_deref(), Some("v2"));
        let snap3 = doc
            .index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .built_at_ops;
        assert_ne!(snap2, snap3, "index rebuilt after a write");
    }

    #[test]
    fn rich_type_map_collaborative_kv_round_trip() {
        // Consumer call site for LoroDoc::map (rich-type Loro Map
        // for collaborative tag sets etc per plan §3 G16-B row).
        let doc = LoroDoc::new();
        doc.map("tags").insert("priority", "high").unwrap();
        // Loro Map LWW for non-HLC property bags.
        let val = doc.map("tags").get("priority").map(|v| match v {
            loro::ValueOrContainer::Value(LoroValue::String(s)) => (*s).to_string(),
            _ => String::new(),
        });
        assert_eq!(val.as_deref(), Some("high"));
    }
}
