//! Phase-4-Foundation G24-D — DAG-shaped version chain.
//!
//! Per CLAUDE.md baked-in #18 + `docs/PLUGIN-MANIFEST.md` §5:
//!
//! > Linear version-chain extended to support branches (forks).
//! > Anchor → v1 → {v2-mainline, v1.5-fork}; CURRENT can point at any
//! > branch tip. Per-user-local version history; updates are
//! > PULL-not-PUSH (no manifest schema versioning needed — CID covers
//! > shape).
//!
//! This module extends the linear `version.rs` pattern (
//! `Anchor + append_version + walk_versions`) to DAG-shape. The
//! linear surface remains for callers that don't need branches.
//!
//! ## Shape
//!
//! A `DagVersionChain` is a forest of (parent, child) edges keyed by
//! CID, with a per-anchor "tips" set tracking branches that have no
//! descendants. The CURRENT pointer is per-device-local (Loro Map
//! per-device-keyed per ratification #2); at this layer we expose
//! `set_current` / `current` operations and assume the higher layer
//! handles cross-device replication.
//!
//! ## Operations
//!
//! - `add_version` — link `parent → child`. Creates a new branch if
//!   `parent` already has children.
//! - `tips` — return all leaf CIDs (branch tips).
//! - `descendants` — walk all CIDs reachable from a given CID.
//! - `current` / `set_current` — read/write the local active reference.
//! - `is_descendant_of` — check ancestry (for upgrade DAG-monotonicity).
//!
//! Cycle detection runs on `add_version` — adding a child that is
//! already an ancestor of the parent returns `VersionDagError::Cycle`.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::Cid;

/// DAG version-chain errors. Mirrors the linear `VersionError` shape
/// but covers branches + cycles.
#[derive(Debug, thiserror::Error)]
pub enum VersionDagError {
    /// Caller supplied a parent CID the DAG has not seen.
    #[error("parent CID not in DAG")]
    UnknownParent {
        /// The unknown parent.
        supplied: Cid,
    },

    /// Adding `child` to `parent` would form a cycle (child is an
    /// ancestor of parent in the existing DAG).
    #[error("adding child to parent would form a cycle")]
    Cycle {
        /// The parent supplied.
        parent: Cid,
        /// The child supplied (which is an ancestor of parent).
        child: Cid,
    },

    /// `set_current` referenced a CID not in the DAG.
    #[error("CURRENT pointer references a CID not in the DAG")]
    UnknownCurrent {
        /// The supplied CID.
        supplied: Cid,
    },
}

impl VersionDagError {
    /// Stable catalog code.
    #[must_use]
    pub fn code(&self) -> benten_errors::ErrorCode {
        match self {
            VersionDagError::UnknownParent { .. } => benten_errors::ErrorCode::VersionUnknownPrior,
            VersionDagError::Cycle { .. } => benten_errors::ErrorCode::VersionBranched,
            VersionDagError::UnknownCurrent { .. } => benten_errors::ErrorCode::VersionUnknownPrior,
        }
    }
}

/// DAG-shaped version chain rooted at `root_cid`.
///
/// Storage:
///   - `parents[child]` — set of parent CIDs (typically size 1, but
///     "merge" nodes may have multiple).
///   - `children[parent]` — set of child CIDs (size 0 = tip).
///   - `current` — local active reference (optional).
#[derive(Debug, Clone)]
pub struct DagVersionChain {
    /// Root of the chain (initial Anchor head).
    root: Cid,
    /// `child -> {parents}`.
    parents: BTreeMap<Cid, BTreeSet<Cid>>,
    /// `parent -> {children}`.
    children: BTreeMap<Cid, BTreeSet<Cid>>,
    /// All CIDs in the DAG.
    all: BTreeSet<Cid>,
    /// Per-device-local CURRENT pointer (per ratification #2).
    current: Option<Cid>,
}

impl DagVersionChain {
    /// Construct a DAG version chain rooted at `root_cid`.
    #[must_use]
    pub fn new(root_cid: Cid) -> Self {
        let mut all = BTreeSet::new();
        all.insert(root_cid);
        Self {
            root: root_cid,
            parents: BTreeMap::new(),
            children: BTreeMap::new(),
            all,
            current: Some(root_cid),
        }
    }

    /// Root CID (Anchor head).
    #[must_use]
    pub fn root(&self) -> &Cid {
        &self.root
    }

    /// Add a parent → child edge to the DAG.
    ///
    /// Multiple calls with the same `parent` create branches.
    /// Multiple parents for the same `child` create a merge node.
    ///
    /// # Errors
    ///
    /// - `VersionDagError::UnknownParent` if `parent` is not in the DAG.
    /// - `VersionDagError::Cycle` if `child` is already an ancestor of
    ///   `parent`.
    pub fn add_version(&mut self, parent: Cid, child: Cid) -> Result<(), VersionDagError> {
        if !self.all.contains(&parent) {
            return Err(VersionDagError::UnknownParent { supplied: parent });
        }
        // Cycle check: is `child` an ancestor of `parent`?
        if self.all.contains(&child) && self.is_ancestor_of(&child, &parent) {
            return Err(VersionDagError::Cycle { parent, child });
        }
        self.all.insert(child);
        self.parents.entry(child).or_default().insert(parent);
        self.children.entry(parent).or_default().insert(child);
        Ok(())
    }

    /// Whether `candidate` is an ancestor of `target` (transitively).
    #[must_use]
    pub fn is_ancestor_of(&self, candidate: &Cid, target: &Cid) -> bool {
        if candidate == target {
            return false;
        }
        let mut stack: Vec<Cid> = vec![*target];
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

    /// Whether `target` is a descendant of `candidate`.
    #[must_use]
    pub fn is_descendant_of(&self, target: &Cid, candidate: &Cid) -> bool {
        self.is_ancestor_of(candidate, target)
    }

    /// Tips: CIDs with no children. Equivalent to the set of all
    /// "branch heads" in the DAG.
    #[must_use]
    pub fn tips(&self) -> Vec<Cid> {
        self.all
            .iter()
            .filter(|c| {
                self.children
                    .get(c)
                    .is_none_or(BTreeSet::is_empty)
            })
            .copied()
            .collect()
    }

    /// Walk all descendants of `from` in BFS order.
    #[must_use]
    pub fn descendants(&self, from: &Cid) -> Vec<Cid> {
        let mut out = Vec::new();
        let mut stack: Vec<Cid> = vec![*from];
        let mut visited: BTreeSet<Cid> = BTreeSet::new();
        while let Some(cur) = stack.pop() {
            if let Some(cs) = self.children.get(&cur) {
                for c in cs {
                    if visited.insert(*c) {
                        out.push(*c);
                        stack.push(*c);
                    }
                }
            }
        }
        out
    }

    /// All CIDs in the DAG (root + every added version).
    pub fn all_cids(&self) -> impl Iterator<Item = &Cid> {
        self.all.iter()
    }

    /// Number of versions including the root.
    #[must_use]
    pub fn len(&self) -> usize {
        self.all.len()
    }

    /// Whether the DAG only has the root.
    #[must_use]
    pub fn is_singleton(&self) -> bool {
        self.all.len() == 1
    }

    /// Local CURRENT pointer (per-device-local per ratification #2).
    #[must_use]
    pub fn current(&self) -> Option<&Cid> {
        self.current.as_ref()
    }

    /// Set the local CURRENT pointer.
    ///
    /// # Errors
    ///
    /// `VersionDagError::UnknownCurrent` if `cid` is not in the DAG.
    pub fn set_current(&mut self, cid: Cid) -> Result<(), VersionDagError> {
        if !self.all.contains(&cid) {
            return Err(VersionDagError::UnknownCurrent { supplied: cid });
        }
        self.current = Some(cid);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cid(b: u8) -> Cid {
        let mut digest = [0u8; 32];
        digest[0] = b;
        Cid::from_blake3_digest(digest)
    }

    #[test]
    fn linear_chain_root_then_v1_then_v2() {
        let mut dag = DagVersionChain::new(cid(0));
        dag.add_version(cid(0), cid(1)).unwrap();
        dag.add_version(cid(1), cid(2)).unwrap();
        assert_eq!(dag.tips(), vec![cid(2)]);
        assert_eq!(dag.len(), 3);
    }

    #[test]
    fn branch_from_v1_produces_two_tips() {
        let mut dag = DagVersionChain::new(cid(0));
        dag.add_version(cid(0), cid(1)).unwrap();
        // Fork off v1: two children
        dag.add_version(cid(1), cid(2)).unwrap();
        dag.add_version(cid(1), cid(3)).unwrap();
        let tips: BTreeSet<Cid> = dag.tips().into_iter().collect();
        assert_eq!(tips, BTreeSet::from([cid(2), cid(3)]));
    }

    #[test]
    fn merge_node_has_two_parents() {
        let mut dag = DagVersionChain::new(cid(0));
        dag.add_version(cid(0), cid(1)).unwrap();
        dag.add_version(cid(0), cid(2)).unwrap();
        dag.add_version(cid(1), cid(3)).unwrap();
        dag.add_version(cid(2), cid(3)).unwrap(); // merge into v3
        assert!(dag.is_ancestor_of(&cid(1), &cid(3)));
        assert!(dag.is_ancestor_of(&cid(2), &cid(3)));
    }

    #[test]
    fn cycle_rejected() {
        let mut dag = DagVersionChain::new(cid(0));
        dag.add_version(cid(0), cid(1)).unwrap();
        dag.add_version(cid(1), cid(2)).unwrap();
        // Try to add cid(0) as a child of cid(2): cid(0) is an ancestor of cid(2) -> cycle.
        let err = dag.add_version(cid(2), cid(0)).unwrap_err();
        assert!(matches!(err, VersionDagError::Cycle { .. }));
    }

    #[test]
    fn unknown_parent_rejected() {
        let mut dag = DagVersionChain::new(cid(0));
        let err = dag.add_version(cid(99), cid(1)).unwrap_err();
        assert!(matches!(err, VersionDagError::UnknownParent { .. }));
    }

    #[test]
    fn set_current_to_branch_tip() {
        let mut dag = DagVersionChain::new(cid(0));
        dag.add_version(cid(0), cid(1)).unwrap();
        dag.add_version(cid(0), cid(2)).unwrap();
        dag.set_current(cid(2)).unwrap();
        assert_eq!(dag.current(), Some(&cid(2)));
    }

    #[test]
    fn set_current_unknown_rejected() {
        let mut dag = DagVersionChain::new(cid(0));
        let err = dag.set_current(cid(99)).unwrap_err();
        assert!(matches!(err, VersionDagError::UnknownCurrent { .. }));
    }
}
