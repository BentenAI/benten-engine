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

use std::collections::BTreeSet;
use std::sync::Arc;

use loro::{ExportMode, LoroDoc as InnerLoroDoc, LoroList, LoroMap, LoroValue};
use serde::{Deserialize, Serialize};

use benten_core::hlc::BentenHlc;

/// Stable container-id for the LWW-property root List. Every
/// [`LoroDoc`] carries one root List under this id; each entry is a
/// flat-encoded `<key>\x1f<physical>:<logical>:<node>:<value>` string.
///
/// The List shape (rather than per-property nested-containers) defends
/// against Loro's documented "concurrent container creation at the
/// same key may overwrite" hazard at the LoroMap layer — concurrent
/// peers each appending to the SAME List have their entries preserved
/// per Loro List CRDT semantics. Read-time scans the List and groups
/// by key for HLC-LWW resolution.
const PROPERTY_ROOT: &str = "benten:properties";

/// Separator between key and packed-stamped-value inside a single
/// List entry. Unit separator (`\x1f`, ASCII 31) so it does not
/// collide with the `:` HLC field separator inside the packed value.
const KEY_VALUE_SEPARATOR: char = '\x1f';

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
    /// canonical wire shape (`{value, hlc}`).
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
}

impl std::fmt::Debug for LoroDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoroDoc")
            .field("op_count", &self.inner.len_ops())
            .field("change_count", &self.inner.len_changes())
            .finish()
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
    /// Returns [`CrdtError::LoroInternal`] if the underlying Loro
    /// insert fails (Loro's failure mode here is essentially
    /// out-of-memory on the op-log allocator).
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
        // The list is append-only; old writes accumulate. A future
        // R6-FP optimization (compact-old-writes-on-checkpoint) can
        // truncate stale entries once all peers observe the merge —
        // the truncation itself is a Loro op that propagates to
        // laggard peers via subsequent merges. For G16-B canary,
        // append-only is the load-bearing simplicity; convergence
        // is observed-correct under the 10 000-case proptest.
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        let packed = format!(
            "{key}{sep}{packed}",
            key = key,
            sep = KEY_VALUE_SEPARATOR,
            packed = pack_stamped(&stamped)
        );
        writes
            .insert(writes.len(), LoroValue::String(packed.into()))
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
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        let n = writes.len();
        let mut best: Option<StampedValue> = None;
        for i in 0..n {
            let entry = match writes.get(i)? {
                loro::ValueOrContainer::Value(LoroValue::String(s)) => (*s).to_string(),
                _ => continue,
            };
            let (entry_key, packed) = match split_key_value(&entry) {
                Some(x) => x,
                None => continue,
            };
            if entry_key != key {
                continue;
            }
            let stamped = match unpack_stamped(packed) {
                Some(s) => s,
                None => continue,
            };
            best = Some(match best {
                None => stamped,
                Some(prev) => {
                    if stamped.hlc.cmp_lex(&prev.hlc) == std::cmp::Ordering::Greater {
                        stamped
                    } else {
                        prev
                    }
                }
            });
        }
        best
    }

    /// Iterate every key currently observed in the property root List.
    fn keys(&self) -> BTreeSet<String> {
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        let mut out = BTreeSet::new();
        for i in 0..writes.len() {
            if let Some(loro::ValueOrContainer::Value(LoroValue::String(s))) = writes.get(i)
                && let Some((k, _)) = split_key_value(&s)
            {
                out.insert(k.to_string());
            }
        }
        out
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
        let mut out = BTreeSet::new();
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        for i in 0..writes.len() {
            if let Some(loro::ValueOrContainer::Value(LoroValue::String(s))) = writes.get(i)
                && let Some((_, packed)) = split_key_value(&s)
                && let Some(sv) = unpack_stamped(packed)
            {
                out.insert(sv.hlc.node_id);
            }
        }
        out
    }

    /// Surface the full list of stamped writes (across all properties)
    /// that the document has accumulated.
    ///
    /// Used by tests + the engine-side AttributionFrame mint per
    /// arch-r1-4 D-C HYBRID. Each entry is `(property_key, stamped)`.
    #[must_use]
    pub fn all_writes(&self) -> Vec<(String, StampedValue)> {
        let writes: LoroList = self.inner.get_list(PROPERTY_ROOT);
        let mut out = Vec::with_capacity(writes.len());
        for i in 0..writes.len() {
            if let Some(loro::ValueOrContainer::Value(LoroValue::String(s))) = writes.get(i)
                && let Some((k, packed)) = split_key_value(&s)
                && let Some(sv) = unpack_stamped(packed)
            {
                out.push((k.to_string(), sv));
            }
        }
        out
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

/// Pack a stamped value into the on-wire flat string format
/// `<physical>:<logical>:<node>:<value>`. The triple-`:`-prefix carries
/// the HLC fields; everything after the third `:` is the (raw) value
/// string. Multi-`:`-containing values are preserved because we only
/// split on the first three.
fn pack_stamped(s: &StampedValue) -> String {
    format!(
        "{}:{}:{}:{}",
        s.hlc.physical_ms, s.hlc.logical, s.hlc.node_id, s.value
    )
}

/// Inverse of [`pack_stamped`]. Returns `None` if the format is
/// malformed (under-segmented or HLC fields fail to parse).
fn unpack_stamped(packed: &str) -> Option<StampedValue> {
    let mut iter = packed.splitn(4, ':');
    let physical_ms: u64 = iter.next()?.parse().ok()?;
    let logical: u32 = iter.next()?.parse().ok()?;
    let node_id: u64 = iter.next()?.parse().ok()?;
    let value = iter.next()?.to_string();
    Some(StampedValue {
        value,
        hlc: HlcWire {
            physical_ms,
            logical,
            node_id,
        },
    })
}

/// Split a key-value entry `<key>\x1f<packed>` at the first
/// [`KEY_VALUE_SEPARATOR`]. Returns `None` if the separator is absent.
fn split_key_value(entry: &str) -> Option<(&str, &str)> {
    let idx = entry.find(KEY_VALUE_SEPARATOR)?;
    Some((
        &entry[..idx],
        &entry[idx + KEY_VALUE_SEPARATOR.len_utf8()..],
    ))
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
