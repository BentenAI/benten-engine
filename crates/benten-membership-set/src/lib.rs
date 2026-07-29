//! `benten-membership-set` — the 15th workspace crate (the MembershipSet
//! keying primitive).
//!
//! A **thin keying-glue Rust engine plugin** (mechanism-half per GN-2 / R0
//! §6.1). Its entire governance / audit / members / economics / federation
//! surface is **graph Nodes** (data-half); the Rust-mechanism that lives here
//! is the keying-frozen minimum.
//!
//! # What this crate owns (frozen Rust-mechanism)
//!
//! - The EXACTLY-3 [`kind::MembershipSetKind`] enum + its codepoint family
//!   (`0x6600` / `0x6610` / `0x6620` — [`codepoints`]).
//! - The multi-stanza-HPKE keying glue ([`keying`]) — delegates every crypto
//!   primitive to `benten-crypto-suite` (the #5 ONLY-call-site; **NEVER forks
//!   #5**).
//! - The fused per-DID [`member::MemberEntry`] record + [`member::MembersTable`]
//!   (one-DID-one-record; ZERO nature field — Inv-22).
//! - The `members_table` snapshot canonical-CBOR serialization
//!   ([`aad::canonical_members_table_bytes`]; the AAD-bound keying minimum,
//!   NQ-W4).
//! - The per-Kind cardinality constructors ([`set::MembershipSet`]).
//! - The `0x6610` group AAD = the BLINDED 11-field set assembly
//!   ([`aad::assemble_group_aad`]; R0.7 §3.10/§4.1) → **OPAQUE bytes** handed
//!   to the crypto-suite (m-15 GNC-5; no reverse dep).
//! - The RBAC [`role::RoleId`] (5-value ordinal, all-active) + per-role UCAN
//!   ability-templates ([`role::ability_template`]).
//! - The role-staleness verify gate ([`verify::verify_stanza`] →
//!   `E_ROLE_STALE_AT_VERIFY`) + ephemeral UCAN grants ([`ucan`]).
//!
//! # What lives as graph (data-half — NEVER inside the sealed Policy)
//!
//! `GovernanceConfig` / `RoleId` permission-*semantics* (UCAN templates) /
//! Garden/Grove sub-config / the members-table relation + derived nature (IVM
//! views) / the audit log / federation `SubsetRef` links / Compute + economics.
//!
//! # Data-half models carry ZERO production callers — enforcement is
//! engine-layer (register-then-enforce disclosure; R9-council F-01..F-05)
//!
//! Several surfaces in this crate MODEL a governance / audit / authz decision
//! as pure data-half Rust so it can be property-pinned, but they have **zero
//! production callers at HEAD** — the REAL enforcement lives at the engine
//! layer (the same register-then-enforce honest disclosure idiom Inv-19 / Inv-21
//! use). Specifically:
//!
//! - [`audit::AuditChain`] tamper-detection (`verify_with_tampered_node_at`) +
//!   the Inv-13 audit-dedup model — the LIVE tamper/dedup enforcement is
//!   engine-layer: `benten_engine::Engine::audit_sequence` + the graph-layer
//!   Inv-13 dedup + `Node::load_verified` mid-chain tamper rejection.
//! - `audit::emit_audit_event_via_engine` / `audit::emit_audit_event_via_bare_put`
//!   (R6-tail F-11) — the enforced-vs-bare attribution-triple SHAPE model. The
//!   "enforced" helper performs ZERO enforcement (it constructs its
//!   `AuditEmitResult` from its own arguments); both are `cfg(any(test, feature
//!   = "testing"))`-gated off the frozen surface. The LIVE enforced-WRITE
//!   attribution is `benten_engine::Engine::audit_sequence` (the
//!   `engine_enforced_path_*` arm of `f_audit_1`); see
//!   `docs/SECURITY-POSTURE.md` "Test-debt note — `f_audit_1` arm-(a)
//!   model-shape".
//! - [`audit::audit_log_query_composition`] (R6-tail F-66) — returns a
//!   HARDCODED `{primitive_tags: [READ, BRANCH, RESPOND],
//!   is_a_new_primitive_kind_variant: false}` descriptor; it does not resolve
//!   or walk a real composition and has zero production callers. The
//!   load-bearing no-13th-`PrimitiveKind` property is enforced elsewhere — by
//!   the frozen 12-variant `benten_core::PrimitiveKind` itself, driven in
//!   `crates/benten-engine/tests/f_audit_4_audit_log_query_graph_native_not_frozen_op.rs`.
//! - [`governance::GovernanceTier`] tier promotion (`promote_tier`) — a
//!   data-half transition model; the LIVE governance authority is engine +
//!   capability-policy driven.
//! - [`role`] authz (per-role ability-templates) — the templates are data; the
//!   LIVE authz enforcement is the UCAN/`CapabilityPolicy` chain at the engine
//!   write boundary.
//! - [`governance::MembershipSetPolicy`] — the zero-sized sealed-policy fence
//!   (mechanism-half boundary marker), NOT a runtime enforcer.
//! - [`verify::verify_stanza`] role-staleness gate (`E_ROLE_STALE_AT_VERIFY`) —
//!   a data-half model with **zero production callers** at HEAD (R10-council
//!   F-10); the LIVE role-staleness / generation-freshness enforcement is the
//!   `benten_drop::layer_c` open-side recompute (`open_group_stanza` /
//!   `open_membership_set_group` re-derive the key-epoch generation words from
//!   the recipient's INDEPENDENTLY-held set-state and fail-close the hybrid
//!   LAMPS verify), NOT this standalone comparator.
//! - [`member::derive_member_nature`] / [`member::is_ai_operated`] member-nature
//!   derivation (Inv-22) — a data-half model with **zero production callers** at
//!   HEAD (R14 F-10); the LIVE member-nature answer is the IVM-materialized
//!   derived view over `(did_method, has_install_manifest)` at the engine +
//!   graph layer (`is_ai_operated(did) = (did.method() == "agent")`; nothing is
//!   read from a stored member field — Inv-22), NOT these standalone derivation
//!   helpers.
//!
//! These models are deliberately RETAINED (they pin the intended shapes +
//! property-hold under proptest); they do **NOT** themselves enforce, and this
//! disclosure keeps them from being read as production enforcement points. The
//! engine-layer wiring is the enforcement of record.
//!
//! # Dependency direction (F-CRATE-2)
//!
//! The B-1 dep set is `{crypto-suite, core, caps, id, graph, sync}` — every one
//! UPSTREAM; **none of them depend on this crate** (the no-reverse-edge
//! compile-fence). The AAD assembler hands OPAQUE bytes across the crypto-suite
//! seam (the crypto-suite never sees this crate's types).

#![forbid(unsafe_code)]

// NATIVE-ONLY per CLAUDE.md baked-in #17. The crate's B-1 dep set includes
// `benten-sync` (iroh transport + Loro CRDT — native-only), so the keying
// primitive is itself a full-peer native mechanism. Browser tabs / thin-client
// surfaces do NOT carry the MembershipSet keying glue (the membership-set
// snapshot reaches them via the thin-client protocol, not the in-bundle crate).
//
// ORDERING (measured 2026-07-29 on df0c8287; an earlier version of this comment
// claimed the opposite): this guard does NOT fire before the transitive
// `benten-sync` guard — it never fires at all from a whole-crate wasm32 build.
// `benten-sync` sits in this crate's PLAIN, un-cfg-gated `[dependencies]`, so
// cargo must compile it first; its own `compile_error!` fires and rustc is never
// invoked on this file. Verified: a wasm32 probe with the getrandom chain
// satisfied reports exactly one `crates/benten-sync/src/lib.rs` hit and ZERO
// hits on this file.
//
// The guard is therefore redundant while benten-sync's gate stands, and is kept
// for one narrow reason: it is the defense that survives benten-sync gaining a
// wasm32 thin-client shim and dropping its own gate. Because no whole-crate
// build can observe it, `.github/workflows/wasm-checks.yml` arm (c) compiles
// THIS FILE ALONE with rustc (no `--extern`) to keep it falsifiable.
#[cfg(target_arch = "wasm32")]
compile_error!(
    "benten-membership-set is native-only per CLAUDE.md baked-in #17 (its B-1 \
     dep set includes the native-only benten-sync). Browser/thin-client surfaces \
     receive the materialized membership-set snapshot via the D-PHASE-3-30 \
     thin-client protocol, NOT the in-bundle keying crate."
);

pub mod aad;
pub mod audit;
pub mod codepoints;
pub mod error;
pub mod federation;
pub mod governance;
pub mod keying;
pub mod keying_kv;
pub mod kind;
pub mod member;
pub mod privacy;
pub mod role;
pub mod set;
pub mod ucan;
pub mod verify;

pub use crate::error::{E_ROLE_STALE_AT_VERIFY, MembershipSetError};
pub use crate::kind::{
    KindDispatchError, MembershipSetKind, RequestedReserveKind, dispatch_reserve,
};
pub use crate::member::{
    Did, Hlc, MemberEntry, MemberNature, MemberRef, MembersTable, RoleId, SigPubKey,
    derive_member_nature, is_ai_operated,
};
pub use crate::set::{MembershipSet, wire_cost_ceiling};

/// Crate scaffold marker — proves the 15th crate exists and is in the
/// workspace graph (the F-CRATE-2 "crate exists" pin floor). Retained from the
/// R3-W4 scaffold; the authoritative codepoint constants now live in
/// [`codepoints`].
///
/// **R6-final F-04: dead R3-W4 scaffold — gated off the frozen public surface**
/// (zero code consumers; the F-CRATE-2 boundary pin is a filesystem/grep
/// assertion, not a `scaffold::CRATE_NAME` consumer). Not frozen into the new
/// 15th crate's permanent v1 API.
#[cfg(any(test, feature = "testing"))]
pub mod scaffold {
    /// The crate's own name, asserted by the F-CRATE-2 boundary pin so the
    /// 15th-crate skeleton is observable.
    pub const CRATE_NAME: &str = "benten-membership-set";
}
