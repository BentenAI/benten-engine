//! `benten-membership-set` — the 15th workspace crate (RED-PHASE STUB at R3-W4).
//!
//! The MembershipSet keying primitive: a **thin keying-glue Rust engine
//! plugin** (mechanism-half per GN-2 / R0 §6.1). Its entire
//! governance / audit / members / economics / federation surface is **graph
//! Nodes** (data-half); the Rust-mechanism that lives here is the
//! keying-frozen minimum.
//!
//! # What this crate WILL own (frozen Rust-mechanism — filled at the R5 canary)
//!
//! - The EXACTLY-3 [`MembershipSetKind`] enum + its codepoint family
//!   (`0x6600` / `0x6610` / `0x6620`).
//! - The multi-stanza-HPKE-Encap keying glue (delegates primitives to
//!   `benten-crypto-suite`; **NEVER forks #5**).
//! - The `members_table` snapshot canonical-CBOR serialization (the
//!   AAD-bound keying minimum; NQ-W4).
//! - Per-Kind constructors + cardinality validation; the **`0x6610` group
//!   AAD = the BLINDED 11-field set** assembly (R0.6 §3.10/§4.1; supersedes
//!   the prior "9-tuple" framing) → **OPAQUE bytes** handed to the
//!   crypto-suite (m-15 GNC-5; no reverse dep); the fork-tie-break (Inv-21)
//!   CRDT rule; the federation recursion-bound (Inv-20 clause-k).
//!
//! # What lives as graph (data-half — NEVER inside the sealed Policy)
//!
//! - `GovernanceConfig` / `RoleId` permission-semantics (UCAN templates) /
//!   Garden/Grove sub-config (signed Nodes).
//! - The members-table relation + derived nature (IVM views).
//! - The audit log (audit-event Nodes + version-chain + IVM + UCAN;
//!   enforced-WRITE-path — m-15 GNC-2).
//! - Federation `SubsetRef` links; Compute / economics (`PeerResource` +
//!   `CommunityEconomicPolicy` — Phase-5+).
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! This crate is a deliberately-empty scaffold at R3-W4 so the W4 test pins
//! compile green at baseline. The pins themselves use **self-contained
//! in-file stub-shims** (each R3 wave is independent for parallel safety —
//! a W4 test never depends on a sibling wave's module). The R5 closing
//! canary wave deletes those shims, inserts the real `use
//! benten_membership_set::…` lines, un-ignores, and verifies green.

#![forbid(unsafe_code)]

/// Crate scaffold marker — proves the 15th crate exists and is in the
/// workspace graph at R3-W4 (F-CRATE-2's "crate exists" pin floor). The R5
/// canary replaces this module with the real keying-mechanism surface.
///
/// This is intentionally minimal: a stub crate that shipped the full
/// `MembershipSetKind` / `members_table` / `MemberRef` surface here would
/// collide with the canary wave that mints it (the R0 §7 sequencing puts the
/// minting in the F-MS-1/3/4 canary). The W4 tests pin the *intended* shape
/// via in-file shims; this module only proves the crate-boundary skeleton.
pub mod scaffold {
    /// The crate's own name, asserted by the F-CRATE-2 boundary pin so the
    /// 15th-crate skeleton is observable (not a no-op marker).
    pub const CRATE_NAME: &str = "benten-membership-set";

    /// The MembershipSet codepoint band lower bound (`0x6600`), surfaced here
    /// only so a boundary test has a real symbol to reference at R3-W4. The
    /// authoritative codepoint constants land in the canary's
    /// `codepoints` module at R5.
    pub const MEMBERSHIP_SET_BAND_LO: u16 = 0x6600;

    /// The MembershipSet codepoint band upper bound (`0x66FF`).
    pub const MEMBERSHIP_SET_BAND_HI: u16 = 0x66FF;
}
