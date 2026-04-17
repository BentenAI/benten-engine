//! [`CapabilityGrant`] — the typed grant Node.
//!
//! A grant is a plain [`benten_core::Node`] with label `"CapabilityGrant"`
//! and a small, fixed property schema. Being a Node means the grant is
//! content-addressed like every other graph entity: two grants with
//! byte-identical content share a CID; a one-byte difference produces a
//! different CID. This is the "honest no" path — attempting to re-issue an
//! already-issued grant is a deduplicated no-op, not a silent duplicate.
//!
//! Edges:
//!
//! - [`GRANTED_TO_LABEL`] — grant → grantee (the entity the capability is
//!   issued to).
//! - [`REVOKED_AT_LABEL`] — grant → revocation Node (Phase 3; Phase 1 only
//!   names the label for forward-compatibility with the sync protocol).
//!
//! See `tests/grant_uniqueness_on_cid.rs` for the content-addressing
//! contract.

use std::collections::BTreeMap;

use benten_core::{Cid, CoreError, Node, Value};

use crate::error::CapError;

/// Edge label: grant → grantee.
pub const GRANTED_TO_LABEL: &str = "GRANTED_TO";

/// Edge label: grant → revocation Node (Phase 3 sync-revocation surface).
pub const REVOKED_AT_LABEL: &str = "REVOKED_AT";

/// Node label applied to every [`CapabilityGrant`].
pub const CAPABILITY_GRANT_LABEL: &str = "CapabilityGrant";

/// A typed, validated capability-scope string.
///
/// Phase 1 parsing is intentionally minimal: trim whitespace; reject the
/// empty / whitespace-only case; preserve the original casing. A grant with
/// an empty scope would permit nothing, which is indistinguishable from no
/// grant at all — refusing at parse is the explicit "honest no".
///
/// Phase 3 revisits the parse shape (hierarchical namespace, attenuation
/// lattice) when UCAN lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantScope(pub String);

impl GrantScope {
    /// Parse a capability scope string.
    ///
    /// # Errors
    ///
    /// Returns [`CapError::Denied`] for the empty / whitespace-only case.
    /// `Denied` is reused (rather than a dedicated `InvalidScope`) so the
    /// ERROR-CATALOG surface stays minimal — a refusal at parse IS a
    /// capability denial at construction.
    pub fn parse(s: &str) -> Result<Self, CapError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(CapError::Denied);
        }
        Ok(GrantScope(trimmed.to_string()))
    }

    /// Borrowed view of the raw scope string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A typed capability grant.
///
/// The Phase-1 grant carries exactly three fields in its public struct
/// surface: `grantee`, `scope`, `hlc_stamp`. This is the minimal shape the
/// R3 unit tests construct via struct literal. The issuer-aware construction
/// path is a free-function-shaped [`CapabilityGrant::new`] that returns a
/// [`Node`] directly — the issuer participates in the resulting CID, but
/// never materializes as a visible field on the struct (which would force
/// every call site to name it).
///
/// Two construction paths:
///
/// - Struct literal: `CapabilityGrant { grantee, scope, hlc_stamp }` +
///   [`CapabilityGrant::as_node`] + [`CapabilityGrant::cid`]. No issuer.
/// - [`CapabilityGrant::new`] — takes `(grantee, issuer, scope)` and returns
///   a [`Node`]. The Node has a `"issuer"` property, so two grants that
///   differ only by issuer produce distinct CIDs. Load-bearing for
///   UCAN-style attenuation chains in Phase 3.
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    /// The entity the capability is granted to.
    pub grantee: Cid,
    /// The capability scope (e.g. `"store:post:write"`).
    pub scope: String,
    /// Hybrid-logical-clock stamp at grant time. Phase 3 wires a real HLC
    /// source; Phase 1 accepts any caller-supplied `u64` (test fixtures use
    /// small literals such as `1` or `7`).
    pub hlc_stamp: u64,
}

impl CapabilityGrant {
    /// High-level constructor — returns the grant as a [`Node`], with the
    /// issuer folded into the Node's property map so the Node's CID is
    /// distinct per `(grantee, issuer, scope)` triple.
    ///
    /// Returning a [`Node`] (rather than a `CapabilityGrant` struct) is a
    /// deliberate asymmetry: the struct has three fields for tests that pin
    /// them directly; the issuer-aware path produces a Node and the caller
    /// reads `.cid()` off the Node directly.
    #[must_use]
    #[allow(
        clippy::new_ret_no_self,
        reason = "asymmetric return: the issuer-aware construction path materializes a Node (so the issuer can feed into the content hash without forcing every struct-literal call site to name an `issuer` field)"
    )]
    pub fn new(grantee: Cid, issuer: Cid, scope: GrantScope) -> Node {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert(
            "grantee".to_string(),
            Value::Bytes(grantee.as_bytes().to_vec()),
        );
        props.insert(
            "issuer".to_string(),
            Value::Bytes(issuer.as_bytes().to_vec()),
        );
        props.insert("scope".to_string(), Value::text(scope.0));
        props.insert("hlc_stamp".to_string(), Value::Int(0));
        Node::new(vec![CAPABILITY_GRANT_LABEL.to_string()], props)
    }

    /// Produce the graph representation of this grant: a [`Node`] with
    /// label `"CapabilityGrant"` and the three struct fields flattened into
    /// properties. Being a Node means the grant participates in
    /// content-addressing like any other graph entity.
    ///
    /// Properties emitted:
    /// - `"grantee"` — [`Value::Bytes`] of the grantee CID's raw bytes.
    /// - `"scope"` — [`Value::text`] of the scope string.
    /// - `"hlc_stamp"` — [`Value::Int`] of the HLC value (cast `i64`).
    #[must_use]
    pub fn as_node(&self) -> Node {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert(
            "grantee".to_string(),
            Value::Bytes(self.grantee.as_bytes().to_vec()),
        );
        props.insert("scope".to_string(), Value::text(self.scope.clone()));
        // Phase 1 HLC stamps are caller-supplied test literals; u64 → i64
        // cast is a no-op in practice. Phase 3 swaps in a typed HLC that
        // saturates explicitly.
        #[allow(
            clippy::cast_possible_wrap,
            reason = "Phase 1 HLC stamps are small test literals; Phase 3 replaces with a typed HLC that saturates explicitly"
        )]
        props.insert("hlc_stamp".to_string(), Value::Int(self.hlc_stamp as i64));
        Node::new(vec![CAPABILITY_GRANT_LABEL.to_string()], props)
    }

    /// CID of the grant Node. Thin wrapper over [`Node::cid`] applied to
    /// [`CapabilityGrant::as_node`].
    ///
    /// # Errors
    ///
    /// Propagates [`CoreError::Serialize`] from [`Node::cid`].
    pub fn cid(&self) -> Result<Cid, CoreError> {
        self.as_node().cid()
    }
}
