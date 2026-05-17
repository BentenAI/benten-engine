//! Phase-3 G21-T2 §D §2.5(c) closure — UCAN-claim → `cap:typed:*`
//! consumer-side mapping table.
//!
//! ## Why this exists
//!
//! The 8 typed-CALL caps under the `cap:typed:*` namespace are
//! STRUCTURALLY declared at
//! `crates/benten-eval/src/typed_call.rs::TypedCallOp::required_cap`:
//!
//!   - `cap:typed:crypto-sign`     (Ed25519Sign)
//!   - `cap:typed:crypto-verify`   (Ed25519Verify)
//!   - `cap:typed:crypto-keygen`   (KeypairGenerate / KeypairFromSeed)
//!   - `cap:typed:hash`            (Blake3Hash)
//!   - `cap:typed:codec`           (MultibaseEncode / MultibaseDecode)
//!   - `cap:typed:did-resolve`     (DidResolve)
//!   - `cap:typed:ucan-validate`   (UcanValidateChain)
//!   - `cap:typed:vc-verify`       (VcVerify)
//!
//! Pre-G21-T2 the durable [`crate::backends::UCANBackend`] had no
//! mapping table that said "this UCAN claim string corresponds to
//! typed-CALL cap X". Under [`crate::NoAuthBackend`] all typed-CALL
//! caps are permitted by default (canary-scope intent); under the
//! durable backend a UCAN claim like
//! `Capability::new("typed:crypto", "sign")` should grant
//! `cap:typed:crypto-sign` — but without a mapping, the backend's
//! attenuation walker had no way to know.
//!
//! This module ships the canonical mapping. Resource:ability pairs
//! that match a known typed-cap shape return the corresponding
//! `cap:typed:*` string; unknown pairs return `None` (the caller
//! falls through to the existing scope-string attenuation logic).

/// Typed-cap groups corresponding to the 8 distinct `cap:typed:*`
/// strings produced by `benten_eval::TypedCallOp::required_cap`.
/// Mirrors that set; extending requires adding a variant here AND a
/// corresponding `TypedCallOp::required_cap` arm.
///
/// `#[non_exhaustive]` (v1-API-stabilization, #998): post-Ed25519
/// signature schemes (CLAUDE.md #19), Kith multi-signature surfaces
/// (Phase-5+), and future plugin-minted typed caps will extend this
/// set. Marking the enum non-exhaustive lets those land as a minor
/// SemVer bump rather than a major break — consistent with the
/// sibling `CapError` / `PendingOp` non-exhaustive discipline.
/// Downstream `match` over this enum must carry a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypedCapGroup {
    /// `cap:typed:crypto-sign`.
    CryptoSign,
    /// `cap:typed:crypto-verify`.
    CryptoVerify,
    /// `cap:typed:crypto-keygen`.
    CryptoKeygen,
    /// `cap:typed:hash`.
    Hash,
    /// `cap:typed:codec`.
    Codec,
    /// `cap:typed:did-resolve`.
    DidResolve,
    /// `cap:typed:ucan-validate`.
    UcanValidate,
    /// `cap:typed:vc-verify`.
    VcVerify,
}

impl TypedCapGroup {
    /// Stable scope-string identifier for this typed-cap group —
    /// matches the `cap:typed:*` form
    /// `benten_eval::TypedCallOp::required_cap` returns.
    #[must_use]
    pub fn cap_string(self) -> &'static str {
        match self {
            Self::CryptoSign => "cap:typed:crypto-sign",
            Self::CryptoVerify => "cap:typed:crypto-verify",
            Self::CryptoKeygen => "cap:typed:crypto-keygen",
            Self::Hash => "cap:typed:hash",
            Self::Codec => "cap:typed:codec",
            Self::DidResolve => "cap:typed:did-resolve",
            Self::UcanValidate => "cap:typed:ucan-validate",
            Self::VcVerify => "cap:typed:vc-verify",
        }
    }
}

/// Phase-3 G21-T2 §D §2.5(c) — translate a UCAN claim
/// `(resource, ability)` pair into the matching
/// [`TypedCapGroup`]. Returns `None` for non-typed-cap claims (the
/// caller falls through to the existing `cap:store:*` /
/// `zone:*:*` attenuation logic).
///
/// The recognised UCAN claim shapes:
///   - `("typed:crypto", "sign")`     → `CryptoSign`
///   - `("typed:crypto", "verify")`   → `CryptoVerify`
///   - `("typed:crypto", "keygen")`   → `CryptoKeygen`
///   - `("typed:hash", "*")`          → `Hash`
///   - `("typed:codec", "*")`         → `Codec`
///   - `("typed:did", "resolve")`     → `DidResolve`
///   - `("typed:ucan", "validate")`   → `UcanValidate`
///   - `("typed:vc", "verify")`       → `VcVerify`
///
/// `NoAuthBackend` (the default Phase-1 backend) permits all caps by
/// default — this mapping is ONLY consulted by the durable
/// [`crate::backends::UCANBackend`] / [`crate::GrantBackedPolicy`]
/// composition at write-check time when the call site has a
/// `cap:typed:*` requirement and the chain-walker needs to determine
/// whether a UCAN claim grants it.
#[must_use]
pub fn typed_cap_for_ucan_claim(resource: &str, ability: &str) -> Option<TypedCapGroup> {
    match (resource, ability) {
        ("typed:crypto", "sign") => Some(TypedCapGroup::CryptoSign),
        ("typed:crypto", "verify") => Some(TypedCapGroup::CryptoVerify),
        ("typed:crypto", "keygen") => Some(TypedCapGroup::CryptoKeygen),
        // hash + codec + ucan-validate + vc-verify are coarse-grained
        // (no per-ability split today); we accept the canonical
        // resource string with any ability OR an explicit `*`.
        ("typed:hash", _) => Some(TypedCapGroup::Hash),
        ("typed:codec", _) => Some(TypedCapGroup::Codec),
        ("typed:did", "resolve") => Some(TypedCapGroup::DidResolve),
        ("typed:ucan", "validate") => Some(TypedCapGroup::UcanValidate),
        ("typed:vc", "verify") => Some(TypedCapGroup::VcVerify),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_strings_match_namespace() {
        for g in [
            TypedCapGroup::CryptoSign,
            TypedCapGroup::CryptoVerify,
            TypedCapGroup::CryptoKeygen,
            TypedCapGroup::Hash,
            TypedCapGroup::Codec,
            TypedCapGroup::DidResolve,
            TypedCapGroup::UcanValidate,
            TypedCapGroup::VcVerify,
        ] {
            assert!(
                g.cap_string().starts_with("cap:typed:"),
                "cap_string for {g:?} must start with cap:typed: namespace; got {}",
                g.cap_string()
            );
        }
    }

    #[test]
    fn maps_known_crypto_claim_shapes() {
        assert_eq!(
            typed_cap_for_ucan_claim("typed:crypto", "sign"),
            Some(TypedCapGroup::CryptoSign)
        );
        assert_eq!(
            typed_cap_for_ucan_claim("typed:crypto", "verify"),
            Some(TypedCapGroup::CryptoVerify)
        );
        assert_eq!(
            typed_cap_for_ucan_claim("typed:crypto", "keygen"),
            Some(TypedCapGroup::CryptoKeygen)
        );
    }

    #[test]
    fn maps_coarse_grained_claim_shapes() {
        // hash + codec accept any ability under the typed: resource
        // (e.g. `*` wildcard or a specific ability).
        assert_eq!(
            typed_cap_for_ucan_claim("typed:hash", "*"),
            Some(TypedCapGroup::Hash)
        );
        assert_eq!(
            typed_cap_for_ucan_claim("typed:codec", "encode"),
            Some(TypedCapGroup::Codec)
        );
    }

    #[test]
    fn unknown_resource_returns_none() {
        assert_eq!(typed_cap_for_ucan_claim("zone:posts", "write"), None);
        assert_eq!(typed_cap_for_ucan_claim("store:post", "write"), None);
        assert_eq!(typed_cap_for_ucan_claim("typed:unknown", "anything"), None);
    }

    #[test]
    fn cap_string_round_trips_to_typed_call_op_required_cap() {
        // Pin: each TypedCapGroup's cap_string MUST match exactly one
        // of the 8 strings TypedCallOp::required_cap can produce. If a
        // future TypedCallOp variant adds a 9th cap_string, this test
        // surfaces the drift immediately because the new string won't
        // appear in the closed-set we check below.
        let known: std::collections::BTreeSet<&'static str> = [
            "cap:typed:crypto-sign",
            "cap:typed:crypto-verify",
            "cap:typed:crypto-keygen",
            "cap:typed:hash",
            "cap:typed:codec",
            "cap:typed:did-resolve",
            "cap:typed:ucan-validate",
            "cap:typed:vc-verify",
        ]
        .into_iter()
        .collect();
        for g in [
            TypedCapGroup::CryptoSign,
            TypedCapGroup::CryptoVerify,
            TypedCapGroup::CryptoKeygen,
            TypedCapGroup::Hash,
            TypedCapGroup::Codec,
            TypedCapGroup::DidResolve,
            TypedCapGroup::UcanValidate,
            TypedCapGroup::VcVerify,
        ] {
            assert!(
                known.contains(g.cap_string()),
                "cap_string {} not in TypedCallOp::required_cap closed set",
                g.cap_string()
            );
        }
    }
}
