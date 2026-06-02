//! MembershipSet primitive — F-full Phase-4-Meta-Core (R3-W5 RED-PHASE landing).
//!
//! # What this crate IS (at R3 landing)
//!
//! An **empty placeholder lib** so the R3-W5 test pins compile-but-stay
//! `#[ignore]`'d at baseline (canonical RED-phase per `pim-12 §3.6e`).
//! Each pin lives under `tests/` and carries a **SELF-CONTAINED
//! stub-shim** (it does NOT `use` any item from this module, nor from a
//! sibling wave's crate) so the wave is independent for parallel-safe
//! R3 dispatch. The F-full **R5 MembershipSet wave** deletes the
//! stub-shims, inserts `use benten_membership_set::…`, un-ignores, and
//! verifies green against the real types.
//!
//! # What the R5 wave fills here (the keying-frozen minimum per R0.3 §3.5)
//!
//! - `MembershipSet { kind, members_table, metadata, governance, policy }`
//!   with `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }`
//!   (EXACTLY-3; the ONLY keying axis).
//! - `MemberEntry { role, is_authority, sig_pubkey, admitted_at_hlc,
//!   member_ref }` — `members_table: BTreeMap<Did, MemberEntry>` fused
//!   snapshot (canonical-CBOR; NQ-W4 length-injective per `F-AAD-1`).
//! - `RoleId { Invitee=0, Viewer=1, Member=2, Moderator=3, Admin=4 }`
//!   (ALL 5 active — BC-9; ordinal is AAD-keying-bound).
//! - The **AAD 9-tuple assembler** (`Inv-20` clause-c) that produces
//!   canonical bytes and hands an **OPAQUE `&[u8]`** to
//!   `benten-crypto-suite` (the crypto-suite has NO reverse dependency
//!   on this crate — m-15 GNC-5).
//! - The `Inv-21` fork-tie-break: **smaller `created_at_hlc` wins**
//!   (oldest-anchor-wins; the DELIBERATE opposite of the in-tree
//!   property LWW larger-HLC-wins rule), made TOTAL via the
//!   **forking-event Version-Node CID** discriminant (M-8 / NQ-D2).
//!
//! Until then this module intentionally exports nothing.

#![forbid(unsafe_code)]

// Intentionally empty at R3 RED-PHASE landing. See the module-level
// doc above + the `tests/` pins for the R5 fill contract.
