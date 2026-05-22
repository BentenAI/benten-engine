//! Phase-4-Meta-Core G-CORE-5 — D3 unified `VersionDag`.
//!
//! Per `RATIFIED-decisions-2026-05-17` D3 + the §0 D3-row of
//! `.addl/phase-4-meta/00-implementation-plan.md`:
//!
//! > ONE `VersionDag` + strict/linear mode; one shared trait + one
//! > CURRENT.
//!
//! This module is the post-G-CORE-5 single composability surface for the
//! version-chain pattern (CLAUDE.md baked-in #8 — "Version chains as
//! opt-in pattern. Anchor Node + Version Nodes + CURRENT pointer in
//! `benten-core`."). It unifies the prior-head-threaded **linear**
//! semantic (the strict-mode equivalent of [`crate::version::Anchor`] +
//! [`crate::version::append_version`]) and the **DAG-shape** branch /
//! merge semantic (the equivalent of
//! [`crate::version_chain::DagVersionChain`]) into ONE nominal type
//! ([`VersionDag`]) with an opt-in [`Mode`] selector.
//!
//! ## Why one type
//!
//! Earlier the crate exposed two disjoint version-chain types with two
//! disjoint CURRENT semantics (linear had no `current()` accessor at
//! all; DAG had its own `current()` / `set_current()`). The deleted
//! `u64`-id Anchor (#1003) added a third — a stale legacy surface
//! removed by PR #1290. Issue [#849] named the resulting drift
//! ("three coexisting Anchor shapes ... three different 'CURRENT'
//! semantics ... no canonical composability surface"). The D3
//! disposition is to consolidate to **one shared trait + one CURRENT**;
//! that contract lives in this module.
//!
//! The [`VersionChain`] trait is the canonical composability surface a
//! higher layer (e.g. `benten-eval` materialization,
//! `benten-platform-foundation` plugin-library) uses when it does not
//! care which mode it received — a `&dyn VersionChain` reads `current()`
//! / `walk()` uniformly across both modes, and a `&mut dyn VersionChain`
//! advances `set_current()` uniformly.
//!
//! ## P-III canonical-bytes invariant
//!
//! The unification is a **type refactor**, not a wire-format change. The
//! Phase-1 baked-in #5 + #8 canonical-bytes contract (BLAKE3 over
//! DAG-CBOR, anchor IDs + Version Node identities are CIDs) is
//! preserved: a Version Node's `.cid()` does not depend on whether it is
//! minted standalone, appended through a strict chain, or attached to a
//! DAG branch. The unified [`VersionDag::append`] touches no
//! node-encoding path; it only records the parent/child edge.
//!
//! ## Coexistence with the underlying surfaces
//!
//! [`VersionDag`] is the canonical *composability* surface; the prior
//! [`crate::version::Anchor`] and [`crate::version_chain::DagVersionChain`]
//! shapes remain as the per-pattern implementations they always were
//! (callers that have very specific need of the linear-only Anchor API
//! or the DAG-only `add_version` API — e.g. existing engine /
//! platform-foundation sites that pre-date the unification — keep
//! working unchanged). The shape commitment for the **future** is the
//! unified `VersionDag` surface; new code threads through it.

extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::Cid;

// ---------------------------------------------------------------------------
// Mode
// ---------------------------------------------------------------------------

/// The version-chain semantic a [`VersionDag`] enforces.
///
/// Selected at [`VersionDag::new`] time and immutable thereafter (the
/// mode is part of the chain's contract; mid-life mode flips would
/// confuse callers about which set of error variants to expect).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Linear, fork-rejecting semantic. A second [`VersionDag::append`]
    /// against an already-extended prior head returns
    /// [`VersionDagError::Branched`] (the equivalent of the old
    /// [`crate::version::VersionError::Branched`]). Callers thread the
    /// head they observed; the chain refuses silent forks.
    Strict,

    /// Branch / merge semantic. Multiple children off one parent =
    /// branch; multiple parents of one child = merge. Cycles are still
    /// rejected ([`VersionDagError::Cycle`]). This is the
    /// branches-allowed shape Phase-4-Foundation needed for the
    /// plugin-library (CLAUDE.md baked-in #18 — "Anchor → v1 →
    /// {v2-mainline, v1.5-fork}").
    Dag,
}

// ---------------------------------------------------------------------------
// VersionDagError
// ---------------------------------------------------------------------------

/// Unified error surface for [`VersionDag`].
///
/// Combines the linear contract's two arms ([`Self::Branched`],
/// [`Self::UnknownPrior`]) and the DAG contract's two arms
/// ([`Self::Cycle`], [`Self::UnknownCurrent`]) into one enum so callers
/// matching against either mode see the same shape. Every variant
/// carries the relevant CID payload so a caller can retry against the
/// actual current state.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VersionDagError {
    /// `Mode::Strict` only: two appends against the same prior head —
    /// chain forks. `seen` is the prior head that was already extended;
    /// `attempted` is the caller-supplied new head that would have
    /// forked the chain.
    #[error("strict-mode chain branched on prior head (attempted new head would fork)")]
    Branched {
        /// The prior head that was already observed to have a successor.
        seen: Cid,
        /// The `new_head` the caller attempted to append against it.
        attempted: Cid,
    },

    /// Caller supplied a prior head the chain has never observed (both
    /// modes). In `Mode::Strict` this fires when the prior head is
    /// neither the root nor a previously-appended new-head; in
    /// `Mode::Dag` it fires when the parent CID was never linked into
    /// the DAG.
    #[error("prior head / parent was never observed by this chain")]
    UnknownPrior {
        /// The prior-head / parent CID the caller claimed.
        supplied: Cid,
    },

    /// `Mode::Dag` only: adding `child` to `parent` would form a cycle
    /// (`child` is already an ancestor of `parent` in the existing
    /// DAG).
    #[error("DAG-mode append would form a cycle")]
    Cycle {
        /// The parent supplied.
        parent: Cid,
        /// The child supplied (already an ancestor of `parent`).
        child: Cid,
    },

    /// [`VersionChain::set_current`] referenced a CID not in the chain
    /// (both modes — the shared CURRENT semantic).
    #[error("CURRENT pointer references a CID not in the chain")]
    UnknownCurrent {
        /// The supplied CID.
        supplied: Cid,
    },
}

impl VersionDagError {
    /// Stable catalog code for this error. Mirrors
    /// [`crate::version::VersionError::code`] and
    /// [`crate::version_chain::VersionDagError::code`] so cross-boundary
    /// callers see the same `E_VERSION_*` family regardless of which
    /// surface they crossed.
    #[must_use]
    pub fn code(&self) -> benten_errors::ErrorCode {
        match self {
            VersionDagError::Branched { .. } | VersionDagError::Cycle { .. } => {
                benten_errors::ErrorCode::VersionBranched
            }
            VersionDagError::UnknownPrior { .. } | VersionDagError::UnknownCurrent { .. } => {
                benten_errors::ErrorCode::VersionUnknownPrior
            }
        }
    }
}

// ---------------------------------------------------------------------------
// VersionChain trait — the canonical composability surface
// ---------------------------------------------------------------------------

/// Shared read/write surface across modes — the canonical composability
/// trait #849 named as missing.
///
/// A `&dyn VersionChain` lets callers walk + read CURRENT without
/// knowing which mode the chain was constructed in; a `&mut dyn
/// VersionChain` advances CURRENT uniformly. The trait is intentionally
/// object-safe (no generics in trait methods, no `Self`-return
/// signatures) so trait-object dispatch is a first-class call shape.
///
/// Append is **not** on this trait — the strict-mode append signature
/// (prior-head-threaded) and the dag-mode append signature
/// (parent-child) carry different invariants. Callers that need to
/// append know the mode they constructed.
pub trait VersionChain {
    /// The chain's current per-device-local active reference. Fresh
    /// chains return their root (one CURRENT semantic across both
    /// modes). Returns `None` only if CURRENT has been explicitly
    /// cleared (never the default state).
    fn current(&self) -> Option<&Cid>;

    /// Advance CURRENT to `cid`. Both modes share this contract:
    ///
    /// # Errors
    ///
    /// [`VersionDagError::UnknownCurrent`] — `cid` is not in the chain.
    fn set_current(&mut self, cid: Cid) -> Result<(), VersionDagError>;

    /// Walk the chain in insertion order (root first) and yield every
    /// CID. The walk is mode-aware:
    /// - `Mode::Strict` yields the linear history (root → … → head).
    /// - `Mode::Dag` yields every CID in the DAG (root + appended).
    ///
    /// Returns a boxed iterator so the trait stays object-safe.
    fn walk(&self) -> alloc::boxed::Box<dyn Iterator<Item = Cid> + '_>;
}

// ---------------------------------------------------------------------------
// VersionDag — the unified type
// ---------------------------------------------------------------------------

/// The unified version-chain type. ONE nominal type, opt-in [`Mode`].
///
/// # Examples
///
/// Strict (linear, fork-rejecting):
///
/// ```
/// use benten_core::Cid;
/// use benten_core::version_dag::{Mode, VersionChain, VersionDag};
///
/// let root = Cid::from_blake3_digest([0u8; 32]);
/// let v1 = Cid::from_blake3_digest([1u8; 32]);
/// let mut chain = VersionDag::new(root, Mode::Strict);
/// chain.append(&root, &v1).unwrap();
/// assert_eq!(chain.walk().count(), 2);
/// assert_eq!(chain.current(), Some(&root));
/// ```
///
/// DAG (branch / merge):
///
/// ```
/// use benten_core::Cid;
/// use benten_core::version_dag::{Mode, VersionDag};
///
/// let root = Cid::from_blake3_digest([0u8; 32]);
/// let a = Cid::from_blake3_digest([1u8; 32]);
/// let b = Cid::from_blake3_digest([2u8; 32]);
/// let mut dag = VersionDag::new(root, Mode::Dag);
/// dag.append(&root, &a).unwrap();
/// dag.append(&root, &b).unwrap(); // branch — accepted in DAG mode
/// let tips = dag.tips();
/// assert_eq!(tips.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct VersionDag {
    mode: Mode,
    root: Cid,
    /// `child -> {parents}`. In strict mode every entry has exactly one
    /// parent; in DAG mode merge children carry multiple.
    parents: BTreeMap<Cid, BTreeSet<Cid>>,
    /// `parent -> {children}`. In strict mode every entry has at most
    /// one child; in DAG mode branches accumulate.
    children: BTreeMap<Cid, BTreeSet<Cid>>,
    /// Every CID known to the chain (root + every appended). Used for
    /// O(log n) "is this CID in the chain" checks.
    all: BTreeSet<Cid>,
    /// Insertion order of appended new-heads. Strict-mode walk uses this
    /// to reproduce the linear order; DAG-mode walk also uses it
    /// (root + insertion-order — DAG-mode walk semantic is "every CID,
    /// in insertion order").
    insertion_order: Vec<Cid>,
    /// Per-device-local CURRENT pointer (per CLAUDE.md baked-in #18
    /// ratification #2 — per-device-local).
    current: Option<Cid>,
}

impl VersionDag {
    /// Construct a new chain rooted at `root` operating in `mode`. The
    /// root is the chain's initial CURRENT in both modes — one CURRENT
    /// semantic.
    #[must_use]
    pub fn new(root: Cid, mode: Mode) -> Self {
        let mut all = BTreeSet::new();
        all.insert(root);
        Self {
            mode,
            root,
            parents: BTreeMap::new(),
            children: BTreeMap::new(),
            all,
            insertion_order: Vec::new(),
            current: Some(root),
        }
    }

    /// The mode this chain was constructed in. Immutable after
    /// construction.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The root CID this chain is anchored at.
    #[must_use]
    pub fn root(&self) -> Cid {
        self.root
    }

    /// Append `new_head` to the chain, declaring `prior_head` as the
    /// head the caller observed (strict-mode) or as the parent
    /// (DAG-mode).
    ///
    /// # Errors
    ///
    /// - [`VersionDagError::UnknownPrior`] — `prior_head` is neither
    ///   the root nor a previously-appended CID.
    /// - [`VersionDagError::Branched`] (`Mode::Strict` only) — a prior
    ///   append already named `prior_head` as its prior; the second
    ///   append would fork the chain.
    /// - [`VersionDagError::Cycle`] (`Mode::Dag` only) — `new_head` is
    ///   already an ancestor of `prior_head` in the existing DAG.
    pub fn append(&mut self, prior_head: &Cid, new_head: &Cid) -> Result<(), VersionDagError> {
        // 1. The prior head / parent must be known to the chain.
        if !self.all.contains(prior_head) {
            return Err(VersionDagError::UnknownPrior {
                supplied: *prior_head,
            });
        }

        match self.mode {
            Mode::Strict => {
                // 2a. Strict mode: refuse a fork. If any previous append
                // already named this same prior head, the second call
                // would diverge — `VersionDagError::Branched`.
                if self
                    .children
                    .get(prior_head)
                    .is_some_and(|kids| !kids.is_empty())
                {
                    return Err(VersionDagError::Branched {
                        seen: *prior_head,
                        attempted: *new_head,
                    });
                }
            }
            Mode::Dag => {
                // 2b. DAG mode: refuse a cycle. If `new_head` is already
                // an ancestor of `prior_head` in the existing DAG,
                // adding the edge would close a loop.
                if self.all.contains(new_head) && self.is_ancestor_of(new_head, prior_head) {
                    return Err(VersionDagError::Cycle {
                        parent: *prior_head,
                        child: *new_head,
                    });
                }
            }
        }

        // 3. Record the edge. Both modes share this storage shape; the
        // strict-mode invariants above guarantee each parent has at
        // most one child and each child has exactly one parent.
        let newly_introduced = self.all.insert(*new_head);
        self.parents
            .entry(*new_head)
            .or_default()
            .insert(*prior_head);
        self.children
            .entry(*prior_head)
            .or_default()
            .insert(*new_head);
        if newly_introduced {
            self.insertion_order.push(*new_head);
        }
        Ok(())
    }

    /// Whether `candidate` is an ancestor of `target` (transitively).
    /// Used by DAG-mode cycle detection and by callers exploring
    /// ancestry.
    #[must_use]
    pub fn is_ancestor_of(&self, candidate: &Cid, target: &Cid) -> bool {
        if candidate == target {
            return false;
        }
        let mut stack: Vec<Cid> = alloc::vec![*target];
        let mut visited: BTreeSet<Cid> = BTreeSet::new();
        while let Some(cur) = stack.pop() {
            if let Some(ps) = self.parents.get(&cur) {
                for p in ps {
                    if p == candidate {
                        return true;
                    }
                    if visited.insert(*p) {
                        stack.push(*p);
                    }
                }
            }
        }
        false
    }

    /// Tips: CIDs with no children. In `Mode::Strict` there is exactly
    /// one tip (the linear head); in `Mode::Dag` there is one per
    /// branch.
    #[must_use]
    pub fn tips(&self) -> Vec<Cid> {
        self.all
            .iter()
            .filter(|c| self.children.get(c).is_none_or(BTreeSet::is_empty))
            .copied()
            .collect()
    }

    /// Number of CIDs in the chain, including the root.
    #[must_use]
    pub fn len(&self) -> usize {
        self.all.len()
    }

    /// Whether the chain is empty. Always `false` in practice — a
    /// constructed [`VersionDag`] always carries its root. Present to
    /// satisfy `clippy::len_without_is_empty` at the public API.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.all.is_empty()
    }
}

// ---------------------------------------------------------------------------
// VersionChain impl on VersionDag
// ---------------------------------------------------------------------------

impl VersionChain for VersionDag {
    fn current(&self) -> Option<&Cid> {
        self.current.as_ref()
    }

    fn set_current(&mut self, cid: Cid) -> Result<(), VersionDagError> {
        if !self.all.contains(&cid) {
            return Err(VersionDagError::UnknownCurrent { supplied: cid });
        }
        self.current = Some(cid);
        Ok(())
    }

    fn walk(&self) -> alloc::boxed::Box<dyn Iterator<Item = Cid> + '_> {
        // Root first, then every appended CID in insertion order. Both
        // modes share this semantic — strict-mode insertion order ≡
        // linear history; dag-mode insertion order ≡ topological order
        // for any valid construction sequence (parents always
        // appended before their children, since `append` rejects an
        // unknown prior head).
        let iter = core::iter::once(self.root).chain(self.insertion_order.iter().copied());
        alloc::boxed::Box::new(iter)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn cid(b: u8) -> Cid {
        let mut digest = [0u8; 32];
        digest[0] = b;
        Cid::from_blake3_digest(digest)
    }

    #[test]
    fn strict_mode_linear_chain() {
        let mut chain = VersionDag::new(cid(0), Mode::Strict);
        chain.append(&cid(0), &cid(1)).unwrap();
        chain.append(&cid(1), &cid(2)).unwrap();
        let walked: Vec<_> = chain.walk().collect();
        assert_eq!(walked, alloc::vec![cid(0), cid(1), cid(2)]);
        assert_eq!(chain.tips(), alloc::vec![cid(2)]);
    }

    #[test]
    fn strict_mode_rejects_fork() {
        let mut chain = VersionDag::new(cid(0), Mode::Strict);
        chain.append(&cid(0), &cid(1)).unwrap();
        let err = chain.append(&cid(0), &cid(2)).unwrap_err();
        assert!(matches!(err, VersionDagError::Branched { .. }));
    }

    #[test]
    fn dag_mode_accepts_branch() {
        let mut dag = VersionDag::new(cid(0), Mode::Dag);
        dag.append(&cid(0), &cid(1)).unwrap();
        dag.append(&cid(0), &cid(2)).unwrap(); // branch — accepted
        let mut tips = dag.tips();
        tips.sort_unstable();
        let mut expected = alloc::vec![cid(1), cid(2)];
        expected.sort_unstable();
        assert_eq!(tips, expected);
    }

    #[test]
    fn dag_mode_rejects_cycle() {
        let mut dag = VersionDag::new(cid(0), Mode::Dag);
        dag.append(&cid(0), &cid(1)).unwrap();
        dag.append(&cid(1), &cid(2)).unwrap();
        let err = dag.append(&cid(2), &cid(0)).unwrap_err();
        assert!(matches!(err, VersionDagError::Cycle { .. }));
    }

    #[test]
    fn unknown_prior_rejected_in_both_modes() {
        for mode in [Mode::Strict, Mode::Dag] {
            let mut chain = VersionDag::new(cid(0), mode);
            let err = chain.append(&cid(99), &cid(1)).unwrap_err();
            assert!(
                matches!(err, VersionDagError::UnknownPrior { .. }),
                "unknown prior in {mode:?} must surface UnknownPrior, got {err:?}"
            );
        }
    }

    #[test]
    fn one_current_semantic_across_modes() {
        for mode in [Mode::Strict, Mode::Dag] {
            let mut chain = VersionDag::new(cid(0), mode);
            assert_eq!(chain.current(), Some(&cid(0)));
            chain.append(&cid(0), &cid(1)).unwrap();
            VersionChain::set_current(&mut chain, cid(1)).unwrap();
            assert_eq!(chain.current(), Some(&cid(1)));
            let err = VersionChain::set_current(&mut chain, cid(99)).unwrap_err();
            assert!(matches!(err, VersionDagError::UnknownCurrent { .. }));
        }
    }

    #[test]
    fn error_code_mapping_stable() {
        let branched = VersionDagError::Branched {
            seen: cid(0),
            attempted: cid(1),
        };
        let cycle = VersionDagError::Cycle {
            parent: cid(1),
            child: cid(0),
        };
        let unknown_prior = VersionDagError::UnknownPrior { supplied: cid(99) };
        let unknown_current = VersionDagError::UnknownCurrent { supplied: cid(99) };
        assert_eq!(branched.code(), benten_errors::ErrorCode::VersionBranched);
        assert_eq!(cycle.code(), benten_errors::ErrorCode::VersionBranched);
        assert_eq!(
            unknown_prior.code(),
            benten_errors::ErrorCode::VersionUnknownPrior
        );
        assert_eq!(
            unknown_current.code(),
            benten_errors::ErrorCode::VersionUnknownPrior
        );
    }

    #[test]
    fn trait_object_dispatch_compiles() {
        let strict = VersionDag::new(cid(0), Mode::Strict);
        let dag = VersionDag::new(cid(0), Mode::Dag);
        let chains: alloc::vec::Vec<&dyn VersionChain> = alloc::vec![&strict, &dag];
        for c in &chains {
            assert_eq!(c.current(), Some(&cid(0)));
            assert_eq!(c.walk().count(), 1);
        }
    }
}
