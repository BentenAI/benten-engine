//! Cid-head-threaded version-chain surface.
//!
//! This is the canonical version-chain API under R4 triage cov-f3 / M21: each
//! [`append_version`] call requires the caller to name the prior head they
//! believe is current, which makes concurrent writers forking the chain a
//! typed-error condition ([`VersionError::Branched`]) rather than silent
//! divergence.
//!
//! A thinner `u64`-id-based surface co-exists at the crate root
//! ([`crate::Anchor`] / [`crate::append_version`] etc.) for the Phase 1
//! "simple" use-case where the caller does not need to detect concurrent
//! appends. R5 keeps both surfaces; R5 G7 picks a canonical shape once the
//! evaluator is in place (`TODO(phase-2)`).
//!
//! ## State storage
//!
//! The chain history is maintained in a process-wide `BTreeMap` keyed by the
//! anchor's initial head CID. Access is serialized through a `spin::Mutex`,
//! which is sufficient for Phase 1 (in-process) guarantees. Phase 3 will
//! replace this with CRDT merge under the sync protocol.
//!
//! `TODO(phase-2)`: thread chain state through an explicit `AnchorStore`
//! handle rather than a process-global. The R5 contract is "make tests pass
//! with minimal scope," and the tests expect the chain to persist across
//! multiple `append_version` calls on the same anchor without any store
//! parameter.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use spin::{Lazy, Mutex};

use crate::Cid;

// ---------------------------------------------------------------------------
// Process-global chain table
//
// Keyed by the anchor's *initial* head CID (captured at `Anchor::new` time).
// Value is the list of recorded `(prior_head, new_head)` appends in insertion
// order.
// ---------------------------------------------------------------------------

type ChainTable = BTreeMap<Cid, Vec<(Cid, Cid)>>;

#[allow(
    clippy::type_complexity,
    reason = "alias keeps the static declaration readable without a nominal type"
)]
static CHAINS: Lazy<Mutex<ChainTable>> = Lazy::new(|| Mutex::new(BTreeMap::new()));

// ---------------------------------------------------------------------------
// Anchor
// ---------------------------------------------------------------------------

/// Cid-head-threaded Anchor identity. The anchor is rooted at its initial
/// head CID (captured at [`Anchor::new`]); subsequent appends supply the
/// prior head they observed so the chain can refuse forks.
#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    /// The initial head the anchor was constructed against. Chain lookup key.
    pub head: Cid,
}

impl Anchor {
    /// Construct an anchor rooted at `head`.
    ///
    /// Multiple `Anchor::new` calls with the same `head` share chain state —
    /// this is intentional for the test scenario where a second "writer"
    /// would observe the same starting head. In practice, anchor identity
    /// for independent version chains should use distinct root heads.
    #[must_use]
    pub fn new(head: Cid) -> Self {
        Self { head }
    }
}

// ---------------------------------------------------------------------------
// VersionError
// ---------------------------------------------------------------------------

/// Error surface for the prior-threaded append API.
///
/// Both error variants carry the relevant CID payload so callers can retry
/// against the actual current head (re-read + re-attempt) rather than
/// receiving an opaque "append failed."
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    /// Two appends against the same prior head — chain forks. `seen` is the
    /// prior head the duplicate was stacked on; `attempted` is the
    /// caller-supplied new head that would have forked the chain.
    #[error("chain branched on prior head (attempted new head would fork)")]
    Branched { seen: Cid, attempted: Cid },

    /// Caller supplied a prior head the anchor has never observed.
    #[error("prior head was never observed by this anchor")]
    UnknownPrior { supplied: Cid },

    /// Catch-all for internal failures (serialization, table poisoning, etc.).
    #[error("version error: {0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// Append / walk
// ---------------------------------------------------------------------------

/// Append `new_head` to the chain rooted at `anchor`, declaring `prior_head`
/// as the head the caller observed.
///
/// # Errors
///
/// - [`VersionError::UnknownPrior`] — `prior_head` is neither the initial
///   root head nor a `new_head` from a previous successful append.
/// - [`VersionError::Branched`] — some previous append already named
///   `prior_head` as its prior. The second caller would fork the chain.
pub fn append_version(
    anchor: &Anchor,
    prior_head: &Cid,
    new_head: &Cid,
) -> Result<(), VersionError> {
    let mut table = CHAINS.lock();
    let chain = table.entry(anchor.head.clone()).or_default();

    // Determine whether `prior_head` is a legitimate head the anchor has
    // observed. It is legitimate iff:
    //   (a) it equals the root head, OR
    //   (b) some previous append produced it as its `new_head`.
    let is_root = *prior_head == anchor.head;
    let is_known_descendant = chain.iter().any(|(_, new)| new == prior_head);
    if !is_root && !is_known_descendant {
        return Err(VersionError::UnknownPrior {
            supplied: prior_head.clone(),
        });
    }

    // Fork detection: has any previous append already named this same
    // `prior_head` as its prior? If so, this would fork the chain.
    if chain.iter().any(|(prev_prior, _)| prev_prior == prior_head) {
        return Err(VersionError::Branched {
            seen: prior_head.clone(),
            attempted: new_head.clone(),
        });
    }

    chain.push((prior_head.clone(), new_head.clone()));
    Ok(())
}

/// Walk the chain from oldest to newest, yielding CIDs (including the root
/// head at position 0).
///
/// For a chain constructed as
/// `Anchor::new(v0) ; append(v0, v1) ; append(v1, v2)`
/// this returns `[v0, v1, v2]`.
pub fn walk_versions(anchor: &Anchor) -> alloc::vec::IntoIter<Cid> {
    let table = CHAINS.lock();
    let mut out = Vec::new();
    out.push(anchor.head.clone());
    if let Some(chain) = table.get(&anchor.head) {
        for (_, new) in chain {
            out.push(new.clone());
        }
    }
    out.into_iter()
}
