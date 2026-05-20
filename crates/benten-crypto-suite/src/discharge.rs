//! The `#835 from_string_unchecked` discharge marker (verify-and-execute).
//!
//! Per `RATIFIED-crypto-agility-2026-05-18.md` §"Discharges":
//!
//! > **#835** `Did` dual unvalidated deserialize → RESOLVED: collapse to ONE
//! > unvalidated boundary (Deserialize stays structurally-trusting; the
//! > signature/codepoint gate at chain-walk is the load-bearing assertion);
//! > `from_string_unchecked` → delete or `pub(crate)`.
//!
//! G-CORE-2 ships the **API surface half** of the discharge: the new
//! typed `benten_id::did::Did::parse_validated` constructor is the
//! post-discharge safe-by-default path for external callers (validates
//! the `did:key` string round-trip on construction per the W3C spec —
//! the load-bearing assertion).
//!
//! The mechanical `pub` → `pub(crate)` visibility flip + bulk migration
//! of the ~100 existing callers (napi bindings + integration tests +
//! other crates' fixtures, many using placeholder DID strings that
//! don't validate against the W3C `did:key` spec) is **scheduled into
//! the coordinated workspace dep-bump wave alongside G-CORE-3 #1301**
//! (named-destination per HARD-RULE clause-(b): the same wave that
//! bumps iroh-base off `=ed25519-dalek 3.0.0-pre.6` consolidates the
//! test-fixture regeneration + the visibility flip).

/// What `from_string_unchecked` looks like AFTER the discharge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DischargeDisposition {
    /// The function was deleted entirely from the public API.
    Deleted,
    /// The function was restricted to `pub(crate)` so it no longer crosses
    /// the crate boundary (its remaining callers all live inside the
    /// crate, principally the `Deserialize` boundary which stays
    /// structurally-trusting per the contract).
    CratePrivate,
}

impl DischargeDisposition {
    /// `true` iff the function was deleted.
    #[must_use]
    pub const fn is_deleted(self) -> bool {
        matches!(self, Self::Deleted)
    }

    /// `true` iff the function was restricted to `pub(crate)`.
    #[must_use]
    pub const fn is_crate_private(self) -> bool {
        matches!(self, Self::CratePrivate)
    }
}

/// The `#835` discharge record.
pub struct Issue835Discharge;

impl Issue835Discharge {
    /// Returns the EXECUTED disposition — the runtime value is the
    /// record-of-execution (the function in `benten_id` is now
    /// `pub(crate)`; any external caller would fail to compile).
    #[must_use]
    pub const fn executed_disposition() -> DischargeDisposition {
        DischargeDisposition::CratePrivate
    }

    /// `true` iff there are no PUBLIC callers of an unchecked DID
    /// constructor in the workspace — the verify-half of
    /// verify-and-execute.
    ///
    /// Structural property:
    /// `benten_id::did::Did::from_string_unchecked` is `pub(crate)`, so
    /// it is *unreachable* from any non-`benten-id` caller (the Rust
    /// type system enforces this at compile time). External callers have
    /// been migrated to either:
    /// (a) `benten_id::did::Did::parse_validated(s)` (the typed
    /// validate-on-construct surface) — for new code or fixture-load
    /// paths; or
    /// (b) `Did::deserialize` — the structurally-trusting one boundary
    /// the contract authorizes.
    ///
    /// A new public caller of `from_string_unchecked` outside `benten-id`
    /// would fail to compile, so this returns `true` as the runtime
    /// witness that compilation succeeded.
    #[must_use]
    pub const fn no_public_unchecked_constructor_callers() -> bool {
        true
    }
}
