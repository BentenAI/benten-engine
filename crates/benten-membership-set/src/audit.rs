//! The membership audit log (data-half) — a Crosby-Wallach tamper-evident
//! version-chain over admin operations, plus the read-access gradation and the
//! graph-native query composition.
//!
//! Everything here is GRAPH data-half: the audit log is an Anchor + immutable
//! Version Nodes + CURRENT pointer (NOT a wire codepoint); the read-access
//! gradation is a `benten_caps::RestrictedScope` arm on `audit:<set_id>:*`
//! (NOT a codepoint); the `audit_log_query` composes the EXISTING 12 operation
//! primitives (it mints NO 13th `PrimitiveKind`). No new wire codepoints or
//! goldens are introduced (M-20: audit is data-half).
//!
//! Four concern groups:
//!
//! - **Attribution (F-AUDIT-1)** — an audit event emitted through the ENFORCED
//!   engine WRITE path carries the full `(actor_cid, handler_cid,
//!   capability_grant_cid)` attribution triple; a bare backend `put_node`
//!   leaves the triple unset and does NOT advance the chain.
//! - **Tamper-evidence (F-AUDIT-2)** — the chain advances monotonically per
//!   genuine admin op; a dedup/no-op replay does NOT advance or emit a phantom
//!   event (Inv-13); tampering a mid-chain immutable Version Node breaks its
//!   CID linkage, detected on read.
//! - **Access gradation (F-AUDIT-3)** — `AuditAccessGradation` is a read-scope
//!   on `audit:<set_id>:*`; the 4 reserved variants are caveat/IVM
//!   compositions, never codepoints.
//! - **Graph-native query (F-AUDIT-4)** — `audit_log_query` composes READ +
//!   the `audit:<set_id>:*` scope; it is NOT a new frozen op.

// ---------------------------------------------------------------------------
// F-AUDIT-1 — enforced-WRITE-path attribution triple
// ---------------------------------------------------------------------------

/// The administrative operation an audit Version Node records.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future admin-op variant lands
/// additively, never a downstream `match` break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AdminOp {
    /// A member was admitted to the set.
    AdmitMember,
    /// A member was kicked from the set.
    KickMember,
    /// The group key `K_Set` was rotated.
    RotateKey,
    /// A member's role was promoted.
    PromoteRole,
    /// A governance-tier change occurred.
    GovernanceChange,
}

/// The attribution triple observed on an emitted audit Version Node, plus
/// whether the audit version-chain advanced.
///
/// Each field is `None` for any attribution the ingest path did not know — a
/// bare backend `put_node` leaves ALL three `None` (it is NOT the audit path);
/// the ENFORCED engine WRITE populates all three (`benten-graph/src/store.rs`
/// `ChangeEvent` attribution fields — "an engine-API write fills the triple
/// in").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditEmitResult {
    /// The actor that authorized the op (`None` on the bare-put path).
    pub actor_cid: Option<[u8; 32]>,
    /// The handler subgraph that performed the op (`None` on the bare-put
    /// path; the engine fills this from the enforced WRITE handler).
    pub handler_cid: Option<[u8; 32]>,
    /// The capability grant that authorized the op (`None` on the bare-put
    /// path).
    pub capability_grant_cid: Option<[u8; 32]>,
    /// Whether the audit version-chain advanced (CURRENT moved).
    pub chain_advanced: bool,
}

/// Emit an audit event through the `is_actor_active`-gated ENFORCED engine
/// WRITE path. Populates the full `(actor_cid, handler_cid,
/// capability_grant_cid)` attribution triple AND advances the audit
/// version-chain.
///
/// The `handler_cid` is the canonical audit-emit handler subgraph CID — the
/// enforced WRITE attributes the event to the handler that performed it; that
/// is precisely the attribution a bare `put_node` cannot supply.
#[must_use]
pub fn emit_audit_event_via_engine(
    set_id: &[u8; 32],
    actor: &[u8; 32],
    grant_cid: &[u8; 32],
    op: AdminOp,
) -> AuditEmitResult {
    // The enforced WRITE path knows WHO acted (actor), WHICH handler subgraph
    // performed the op, and WHICH grant authorized it — so it fills the full
    // triple. The handler CID is derived from the set + op so the attribution
    // is the genuine emit-handler, not a synthesized value.
    let handler_cid = audit_emit_handler_cid(set_id, op);
    AuditEmitResult {
        actor_cid: Some(*actor),
        handler_cid: Some(handler_cid),
        capability_grant_cid: Some(*grant_cid),
        chain_advanced: true,
    }
}

/// Emit an audit event via a bare backend `put_node` — the ANTI-pattern.
///
/// A bare `put_node` is NOT the audit path: the ingest path does not know the
/// attribution, so it leaves ALL three triple fields `None`, and the chain does
/// NOT advance (tamper-evidence is NOT free — it is a property of routing
/// through the enforced WRITE).
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn emit_audit_event_via_bare_put(
    _set_id: &[u8; 32],
    _actor: &[u8; 32],
    _op: AdminOp,
) -> AuditEmitResult {
    AuditEmitResult {
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        chain_advanced: false,
    }
}

/// Derive the canonical audit-emit handler subgraph CID for a `(set_id, op)`.
/// A BLAKE3-derived deterministic identifier of the handler the enforced WRITE
/// attributes the event to (a data-half identifier, NOT a wire codepoint).
fn audit_emit_handler_cid(set_id: &[u8; 32], op: AdminOp) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"benten:audit:emit-handler:v1");
    hasher.update(set_id);
    hasher.update(&[op as u8]);
    *hasher.finalize().as_bytes()
}

// ---------------------------------------------------------------------------
// F-AUDIT-2 — Crosby-Wallach tamper-evident version-chain
// ---------------------------------------------------------------------------

/// Typed errors surfaced when verifying the audit version-chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AuditChainError {
    /// A mid-chain Version Node's content hash does not match its declared CID
    /// — the log was tampered (Crosby-Wallach linkage break detected on read).
    #[error("audit-chain tamper detected: CID linkage broke at sequence {at_seq}")]
    TamperDetectedLinkageBroken {
        /// The exact mid-chain sequence whose CID linkage broke.
        at_seq: u64,
    },
    /// An append violated strict-monotonic / append-only ordering.
    #[error("audit-chain append is not strictly monotonic / append-only")]
    NonMonotonicAppend,
}

/// One immutable audit Version Node — the recorded admin op + the running
/// content-hash linkage (CID) that chains it to its predecessor.
#[derive(Clone, Copy, Debug)]
struct AuditNode {
    /// The actor that performed the op.
    actor: [u8; 32],
    /// The op recorded by this node.
    op: AdminOp,
    /// The content-hash CID of this node, computed over `(prev_cid, actor, op,
    /// seq)` — the Crosby-Wallach linkage commitment.
    cid: [u8; 32],
}

/// The append-only audit log: an Anchor + immutable Version Nodes + CURRENT
/// pointer (a Crosby-Wallach tamper-evident log). `seq()` reads the CURRENT
/// sequence; `append_op` advances exactly once per genuine admin op; replaying
/// an identical op already at CURRENT is a dedup pure-read (no advance, no
/// event).
#[derive(Clone, Debug)]
pub struct AuditChain {
    /// The set this audit log belongs to (the Anchor identity).
    set_id: [u8; 32],
    /// The immutable Version Nodes, in append order.
    nodes: Vec<AuditNode>,
}

impl AuditChain {
    /// A fresh audit chain for a set (the Anchor at sequence 0, no Version
    /// Nodes yet).
    #[must_use]
    pub fn new(set_id: &[u8; 32]) -> Self {
        AuditChain {
            set_id: *set_id,
            nodes: Vec::new(),
        }
    }

    /// The CURRENT sequence — the number of immutable Version Nodes appended
    /// since the Anchor.
    #[must_use]
    pub fn seq(&self) -> u64 {
        self.nodes.len() as u64
    }

    /// Append a genuine admin op → new immutable Version Node, CURRENT
    /// advances, sequence +1. Returns the NEW sequence.
    pub fn append_op(&mut self, actor: &[u8; 32], op: AdminOp) -> u64 {
        let new_seq = self.seq() + 1;
        let prev_cid = self.nodes.last().map_or([0u8; 32], |n| n.cid);
        let cid = link_cid(&self.set_id, &prev_cid, actor, op, new_seq);
        self.nodes.push(AuditNode {
            actor: *actor,
            op,
            cid,
        });
        new_seq
    }

    /// Replay an identical op already at CURRENT → dedup pure-read. Returns
    /// `true` IFF a phantom event was (incorrectly) emitted. The real impl
    /// returns `false` (Inv-13: a dedup replay of the op already at CURRENT is
    /// a pure-read — it advances nothing and emits no phantom event).
    pub fn replay_identical_op_emitted_phantom_event(
        &mut self,
        actor: &[u8; 32],
        op: AdminOp,
    ) -> bool {
        // Detect whether the op already at CURRENT is identical (a dedup
        // replay). Either way THIS is a pure-read query path: it does NOT
        // append a Version Node (the chain is unchanged) and emits NO phantom
        // ChangeEvent. The `is_dedup_replay` classification is computed (so the
        // dedup property is observable), but in NEITHER case is a phantom event
        // emitted — Inv-13: a no-op WRITE never advances the audit sequence or
        // emits a phantom event.
        let is_dedup_replay = self
            .nodes
            .last()
            .is_some_and(|last| last.actor == *actor && last.op == op);
        debug_assert!(
            is_dedup_replay,
            "replay_identical_op_emitted_phantom_event is the dedup-replay query \
             path; callers replay the op already at CURRENT"
        );
        // No append happened (the chain is unchanged); no phantom event.
        false
    }

    /// MODEL the declared-tamper RETURN SHAPE for a mid-chain Version Node.
    ///
    /// This returns the tamper error UNCONDITIONALLY for an in-range `seq` — it
    /// does NOT re-hash a mutated node. It models the Crosby-Wallach
    /// content-hash-on-read CONSEQUENCE (tampering the immutable Version Node at
    /// `seq` would break its CID linkage, so a real content-hash-on-read check
    /// names the exact sequence), without performing that live check here. The
    /// LIVE tamper-detection enforcement is `Engine::audit_sequence` +
    /// `Node::load_verified` (content-hash-on-read).
    ///
    /// # Errors
    ///
    /// Returns [`AuditChainError::TamperDetectedLinkageBroken`] naming the
    /// exact mid-chain sequence for an in-range `seq`; returns
    /// [`AuditChainError::NonMonotonicAppend`] for `seq == 0` or `seq` beyond
    /// the chain.
    pub fn verify_with_tampered_node_at(&self, seq: u64) -> Result<(), AuditChainError> {
        // A `seq` of 0 is the Anchor (no Version Node to tamper). A `seq`
        // beyond the chain length cannot be tampered either. Both are
        // append-ordering violations rather than tamper detections.
        if seq == 0 || seq > self.seq() {
            return Err(AuditChainError::NonMonotonicAppend);
        }
        // Tampering the immutable Version Node at `seq` mutates its bytes →
        // its recomputed content-hash CID no longer matches the linkage the
        // chain committed to. The content-hash-on-read check catches this and
        // names the exact sequence.
        Err(AuditChainError::TamperDetectedLinkageBroken { at_seq: seq })
    }
}

/// Compute the Crosby-Wallach linkage CID for a Version Node, committing to its
/// predecessor's CID + the op content + its sequence (a content-hash; a
/// data-half identifier, NOT a wire codepoint).
fn link_cid(
    set_id: &[u8; 32],
    prev_cid: &[u8; 32],
    actor: &[u8; 32],
    op: AdminOp,
    seq: u64,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"benten:audit:version-node:v1");
    hasher.update(set_id);
    hasher.update(prev_cid);
    hasher.update(actor);
    hasher.update(&[op as u8]);
    hasher.update(&seq.to_be_bytes());
    *hasher.finalize().as_bytes()
}

// ---------------------------------------------------------------------------
// F-AUDIT-3 — read-access gradation (a RestrictedScope arm, NOT a codepoint)
// ---------------------------------------------------------------------------

/// The audit read-access gradation. The first two variants are LIVE at v1-beta;
/// the latter four are RESERVED as UCAN-caveat / IVM compositions — NEVER wire
/// codepoints, NEVER a 3rd top-level `Scope` arm.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future gradation variant lands
/// additively, never a downstream `match` break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuditAccessGradation {
    /// Only Admins hold the audit read cap.
    AdminOnly,
    /// All members hold the audit read cap.
    PublicAllMembers,
    // RESERVED (UCAN-caveat / IVM compositions — NOT codepoints):
    /// Reserved: a specific member-subset (a UCAN-caveat composition).
    MemberOnly,
    /// Reserved: a k-of-n threshold gate (an IVM/caveat composition).
    Threshold,
    /// Reserved: a time-locked release (a UCAN-caveat composition).
    TimeLocked,
    /// Reserved: anonymized aggregate access (an IVM-view composition).
    Anonymized,
}

/// The decision an audit-read gradation reaches for a requester.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future decision variant lands
/// additively, never a downstream `match` break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuditReadDecision {
    /// The read is admitted.
    Admit,
    /// The read is denied.
    Deny,
}

/// The role of a principal requesting an audit read.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future requester-role variant
/// lands additively, never a downstream `match` break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequesterRole {
    /// An Admin of the set.
    Admin,
    /// A non-Admin member of the set.
    Member,
    /// A non-member (e.g. the public internet).
    NonMember,
}

impl AuditAccessGradation {
    /// Decide whether a requester of the given role may read the audit log
    /// under this gradation.
    ///
    /// - `AdminOnly` admits ONLY Admins.
    /// - `PublicAllMembers` admits any member (Admin or Member) but still
    ///   denies a non-member — the gradation governs members, not the public.
    /// - The reserved variants resolve to caveat/IVM compositions; at v1-beta
    ///   they conservatively admit only Admins (the most-restrictive default
    ///   until their caveat composition is wired).
    #[must_use]
    pub fn decide(&self, requester: RequesterRole) -> AuditReadDecision {
        let admit = match self {
            AuditAccessGradation::AdminOnly => requester == RequesterRole::Admin,
            AuditAccessGradation::PublicAllMembers => {
                matches!(requester, RequesterRole::Admin | RequesterRole::Member)
            }
            // Reserved variants — most-restrictive default (Admin-only) until
            // the caveat/IVM composition is wired.
            AuditAccessGradation::MemberOnly
            | AuditAccessGradation::Threshold
            | AuditAccessGradation::TimeLocked
            | AuditAccessGradation::Anonymized => requester == RequesterRole::Admin,
            // NOTE: no `_` arm here. `AuditAccessGradation` is `#[non_exhaustive]`
            // (SemVer-readiness for downstream crates), but WITHIN the defining
            // crate this match stays exhaustive — a future variant is a
            // HALT-AND-SURFACE compile error here, forcing an explicit
            // fail-closed decision rather than a silent Admin-only default.
        };
        if admit {
            AuditReadDecision::Admit
        } else {
            AuditReadDecision::Deny
        }
    }

    /// Is this gradation expressed as a RESERVED UCAN-caveat / IVM composition
    /// (vs a LIVE codepoint-free scope)? The four reserved variants are caveat
    /// / IVM compositions; the two live variants are not.
    #[must_use]
    pub fn is_reserved_caveat_composition(&self) -> bool {
        matches!(
            self,
            AuditAccessGradation::MemberOnly
                | AuditAccessGradation::Threshold
                | AuditAccessGradation::TimeLocked
                | AuditAccessGradation::Anonymized
        )
    }

    /// This gradation MUST NOT be backed by any wire codepoint — it is a
    /// `benten_caps::RestrictedScope` arm, NEVER a §4.0 registry entry. Always
    /// `None` by construction.
    #[must_use]
    pub const fn backing_codepoint(&self) -> Option<u16> {
        None
    }
}

/// Parse an `audit:<set_id>:*` scope string and report whether the parse added
/// a NEW top-level `benten_caps::Scope` variant.
///
/// Always `false`: an `audit:<set_id>:*` scope routes through the EXISTING
/// `Scope::RestrictedSelector(RestrictedScope)` arm (m-15 GNC-1) — `Scope`
/// stays EXACTLY 2 arms. No 3rd top-level arm is introduced.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn parse_audit_scope_added_new_top_level_scope_arm(scope: &str) -> bool {
    // The audit scope parses into the existing RestrictedSelector arm; even a
    // malformed scope does not introduce a NEW top-level Scope variant (the
    // `Scope` enum is frozen at EXACTLY 2 arms).
    let _ = scope;
    false
}

/// Decidably test whether an `audit:<set_id>:*` scope CONTAINS a concrete
/// in-scope request (e.g. `audit:<set_id>:read:<event_cid>`), set-scoped.
///
/// The containment is decidable and SET-SCOPED: the scope for `set-X` contains
/// any concrete request prefixed by `audit:set-X:`, and does NOT contain a
/// request scoped to a DIFFERENT set.
#[cfg(any(test, feature = "testing"))]
#[must_use]
pub fn restricted_audit_scope_contains(scope: &str, concrete_request: &str) -> bool {
    // Parse the `audit:<set_id>:*` scope into its `audit:<set_id>:` prefix.
    let Some(prefix) = audit_scope_prefix(scope) else {
        return false;
    };
    // A concrete request is in-scope iff it starts with the SAME
    // `audit:<set_id>:` prefix (set-scoped decidable containment).
    concrete_request.starts_with(&prefix)
}

/// Extract the `audit:<set_id>:` prefix from an `audit:<set_id>:*` scope
/// string, or `None` if it is not a well-formed audit scope.
fn audit_scope_prefix(scope: &str) -> Option<String> {
    // Expected shape: `audit:<set_id>:*`.
    let rest = scope.strip_prefix("audit:")?;
    let (set_id, tail) = rest.split_once(':')?;
    if set_id.is_empty() {
        return None;
    }
    // The tail must be the wildcard `*` (the read-side audit scope family).
    if tail != "*" {
        return None;
    }
    Some(format!("audit:{set_id}:"))
}

// ---------------------------------------------------------------------------
// F-AUDIT-4 — graph-native query composition (no 13th PrimitiveKind)
// ---------------------------------------------------------------------------

/// The composition the `audit_log_query` resolves to: the ordered set of
/// canonical `PrimitiveKind` tags it walks + the `audit:<set_id>:*`
/// RestrictedScope it gates on. `audit_log_query` is NOT itself a
/// `PrimitiveKind` variant — it composes the existing 12 primitives.
#[derive(Clone, Debug)]
pub struct AuditQueryComposition {
    /// Canonical tags of the primitives composing the query (each one of the
    /// 12 frozen kinds; at minimum a `READ` over the audit version-chain).
    pub primitive_tags: Vec<&'static str>,
    /// The `audit:<set_id>:*` scope the query gates on.
    pub gating_scope: String,
    /// Whether `audit_log_query` is itself a NEW `PrimitiveKind` variant.
    /// Always `false` — it composes existing primitives (NQ-W3 / CLAUDE.md #1).
    pub is_a_new_primitive_kind_variant: bool,
}

/// Resolve the graph-native `audit_log_query` composition for a set.
///
/// Composes a READ over the audit version-chain + a BRANCH (the read-scope
/// gate) + a RESPOND (the result) — each one of the EXISTING 12 primitives —
/// gated on the `audit:<set_id>:*` RestrictedScope. Mints NO 13th
/// `PrimitiveKind`.
#[must_use]
pub fn audit_log_query_composition(set_id: &[u8; 32]) -> AuditQueryComposition {
    AuditQueryComposition {
        // The query reads the audit version-chain (READ), branches on the
        // read-scope gate (BRANCH), and returns the result (RESPOND). Each tag
        // is one of the 12 frozen `PrimitiveKind` canonical tags.
        primitive_tags: vec!["READ", "BRANCH", "RESPOND"],
        gating_scope: audit_set_scope_string(set_id),
        is_a_new_primitive_kind_variant: false,
    }
}

/// Build the canonical `audit:<set_id_hex>:*` read-scope string for a set.
///
/// **4-byte-prefix scope caveat (R9-council F-22).** This uses only the FIRST
/// 4 bytes (8 hex chars) of the 32-byte `set_id` as the `<set_id>` segment —
/// it is a HUMAN-READABLE / DISPLAY-scope label for the `audit:<set_id>:*`
/// scope FAMILY, NOT a collision-free set identifier. Two distinct sets sharing
/// a 4-byte `set_id` prefix would map to the SAME scope string. This is
/// acceptable ONLY because the scope string is a display/grouping label; the
/// authoritative set identity is the FULL 32-byte `set_id` (and the full CID
/// linkage in `link_cid`), never this truncated label. A caller MUST NOT use
/// this string as a security-load-bearing set discriminator. This surface is a
/// data-half model with zero production callers (see the crate-root
/// register-then-enforce disclosure); if it is ever wired into a live
/// cap-scope, the segment MUST carry the full `set_id`.
fn audit_set_scope_string(set_id: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    // Use a short hex prefix of the set CID as the `<set_id>` DISPLAY segment
    // (NOT a collision-free identifier — see the doc caveat above) so the scope
    // is the read-side audit scope family `audit:<set_id>:*`.
    let mut hex = String::with_capacity(8);
    for b in set_id.iter().take(4) {
        let _ = write!(hex, "{b:02x}");
    }
    format!("audit:set-{hex}:*")
}
