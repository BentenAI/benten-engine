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
    /// capability denial at construction. The `required` and `entity`
    /// payload fields are empty strings because there is no write-context
    /// to attribute.
    pub fn parse(s: &str) -> Result<Self, CapError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(CapError::Denied {
                required: String::new(),
                entity: String::new(),
            });
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
/// Four public fields: `grantee`, `issuer`, `scope`, and `hlc_stamp`. All
/// four feed the content-addressed CID via [`CapabilityGrant::as_node`] →
/// [`Node::cid`], so two grants differing in any field produce distinct
/// CIDs — load-bearing for UCAN-style attenuation chains in Phase 3 where
/// the issuer is a first-class identity axis.
///
/// # Construction
///
/// - [`CapabilityGrant::new`] — the convenience constructor: takes
///   `(grantee, issuer, scope)` and zero-initializes `hlc_stamp`. Returns a
///   `CapabilityGrant` (NOT a `Node`), sidestepping Clippy's
///   `new_ret_no_self` lint and letting callers read `.cid()` / `.as_node()`
///   / the public fields off one typed handle.
/// - Struct literal: `CapabilityGrant { grantee, issuer, scope, hlc_stamp }`
///   — the path R3 unit tests use. Every field must be named; this is the
///   correctness guardrail against the g4-cr-2 "two incompatible
///   construction paths" bug where an issuer could be silently omitted.
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    /// The entity the capability is granted to.
    pub grantee: Cid,
    /// The entity that issued the grant. Load-bearing for attenuation
    /// chains: two grants with the same grantee + scope but different
    /// issuers MUST have different CIDs (Phase 3 UCAN depends on this).
    pub issuer: Cid,
    /// The capability scope (e.g. `"store:post:write"`).
    pub scope: String,
    /// Hybrid-logical-clock stamp at grant time. Phase 3 wires a real HLC
    /// source; Phase 1 accepts any caller-supplied `u64` (test fixtures use
    /// small literals such as `1` or `7`).
    pub hlc_stamp: u64,
}

impl CapabilityGrant {
    /// Construct a grant from `(grantee, issuer, scope)`. HLC stamp is
    /// zero-initialized; set it manually via the public field if the
    /// caller has a real HLC to stamp with.
    ///
    /// Returns `Self` (not `Node`), sidestepping the Clippy
    /// `new_ret_no_self` asymmetry the original G4 draft had. Callers who
    /// need the Node representation call [`CapabilityGrant::as_node`] or
    /// [`CapabilityGrant::cid`] directly off the returned handle.
    #[must_use]
    pub fn new(grantee: Cid, issuer: Cid, scope: GrantScope) -> Self {
        Self {
            grantee,
            issuer,
            scope: scope.0,
            hlc_stamp: 0,
        }
    }

    /// Produce the graph representation of this grant: a [`Node`] with
    /// label `"CapabilityGrant"` and the four struct fields flattened into
    /// properties. Being a Node means the grant participates in
    /// content-addressing like any other graph entity.
    ///
    /// Properties emitted:
    /// - `"grantee"` — [`Value::Bytes`] of the grantee CID's raw bytes.
    /// - `"issuer"` — [`Value::Bytes`] of the issuer CID's raw bytes.
    /// - `"scope"` — [`Value::text`] of the scope string.
    /// - `"hlc_stamp"` — [`Value::Int`] of the HLC value (cast `i64`).
    ///
    /// Every grant has an issuer property (even if the issuer is a
    /// "self-issued" sentinel) — this is the g4-cr-2 correctness-by-
    /// construction fix against the "unauthenticated-root issuer" principal-
    /// confusion vector.
    #[must_use]
    pub fn as_node(&self) -> Node {
        let mut props: BTreeMap<String, Value> = BTreeMap::new();
        props.insert(
            "grantee".to_string(),
            Value::Bytes(self.grantee.as_bytes().to_vec()),
        );
        props.insert(
            "issuer".to_string(),
            Value::Bytes(self.issuer.as_bytes().to_vec()),
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
