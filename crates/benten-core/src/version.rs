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
//! Chain state lives inside each [`Anchor`] behind an `Arc<spin::Mutex<...>>`.
//! Cloning an `Anchor` shares state — this is the "two writers hold the same
//! anchor handle" scenario. Calling [`Anchor::new`] twice with the same `head`
//! CID produces two **independent** anchors with independent chains; the
//! previous design (a process-global `BTreeMap` keyed by root CID) caused
//! cross-test state leakage and could not distinguish two independent forks
//! that happen to share a root CID.
//!
//! `TODO(phase-2-anchorstore)`: Phase 3 sync will replace per-anchor state
//! with CRDT merge under the sync protocol; R5 G7 may still prefer an explicit
//! `AnchorStore` handle for bulk operations. The per-anchor `Arc<Mutex<...>>`
//! is the minimum that passes Phase 1 tests without leaking state between
//! unrelated anchors.

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::Mutex;

use crate::Cid;

// ---------------------------------------------------------------------------
// Anchor
// ---------------------------------------------------------------------------

/// Cid-head-threaded Anchor identity. The anchor is rooted at its initial
/// head CID (captured at [`Anchor::new`]); subsequent appends supply the
/// prior head they observed so the chain can refuse forks.
///
/// Chain history lives inside the anchor behind an `Arc<Mutex<...>>`.
/// Cloning an `Anchor` shares state (both clones see and append to the same
/// chain). Two independent [`Anchor::new`] calls — even with the same `head`
/// CID — produce **independent** chains.
#[derive(Debug, Clone)]
pub struct Anchor {
    /// The initial head the anchor was constructed against.
    pub head: Cid,
    /// Chain history: list of `(prior_head, new_head)` appends in insertion
    /// order. Shared across clones of the same Anchor; independent across
    /// separate `Anchor::new` calls.
    chain: Arc<Mutex<Vec<(Cid, Cid)>>>,
}

// Two anchors are equal iff they point to the same chain instance (same
// `Arc` allocation) AND have the same head. Independent `Anchor::new`
// calls — even with the same `head` — are not equal, which prevents the
// cross-test state-leak hazard the previous process-global design had.
impl PartialEq for Anchor {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head && Arc::ptr_eq(&self.chain, &other.chain)
    }
}

impl Anchor {
    /// Construct an anchor rooted at `head`. Each `Anchor::new` call creates
    /// an **independent** chain; to share chain state between writers, clone
    /// the anchor rather than calling `new` twice with the same head.
    #[must_use]
    pub fn new(head: Cid) -> Self {
        Self {
            head,
            chain: Arc::new(Mutex::new(Vec::new())),
        }
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
    Branched {
        /// The prior head that was already observed to have a successor.
        seen: Cid,
        /// The `new_head` the caller attempted to append against it.
        attempted: Cid,
    },

    /// Caller supplied a prior head the anchor has never observed.
    #[error("prior head was never observed by this anchor")]
    UnknownPrior {
        /// The prior head the caller claimed was current.
        supplied: Cid,
    },
}

impl VersionError {
    /// Stable catalog code for this error. Every other error enum in the
    /// workspace exposes a `.code()`; [`VersionError`] does too so cross-
    /// boundary callers receive a stable identifier regardless of the
    /// wrapper type (r6-err-11).
    #[must_use]
    pub fn code(&self) -> benten_errors::ErrorCode {
        match self {
            VersionError::Branched { .. } => benten_errors::ErrorCode::VersionBranched,
            VersionError::UnknownPrior { .. } => benten_errors::ErrorCode::VersionUnknownPrior,
        }
    }
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
    let mut chain = anchor.chain.lock();

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
    let chain = anchor.chain.lock();
    let mut out = Vec::with_capacity(1 + chain.len());
    out.push(anchor.head.clone());
    for (_, new) in chain.iter() {
        out.push(new.clone());
    }
    out.into_iter()
}
