//! Qual-2 #759 — the `CanonicalBytes` trait.
//!
//! Six sites across `did_rotation` / `vc` / `ucan` /
//! `device_attestation` independently spell the same shape: take a
//! fixed-schema value, DAG-CBOR-encode it, and treat the encode as
//! infallible (`.expect("…fixed-shape…cannot fail")`). Two of those
//! sites further project a signature-input subset into an inline
//! `#[derive(Serialize)] struct SigInput<'a>` that *excludes the
//! signature field* (signature self-reference hygiene).
//!
//! This trait consolidates the **pattern** — the
//! `Self -> canonical Vec<u8>` contract plus the infallibility
//! invariant — into one named, documented seam. It does **NOT** alter
//! any byte layout: every impl below reproduces its type's exact
//! pre-existing encoding (whole-struct or `SigInput`-projection)
//! byte-for-byte. The canonical-byte encoding of any identity value
//! is a v1-wire-adjacent surface (CLAUDE.md baked-in #5); this
//! refactor is structural-only and changes zero bytes on the wire.
//!
//! Per the §3.5m P-III discipline (wire/CID/on-disk-format changes are
//! never a refactor side-effect): the impls are deliberately verbatim
//! reproductions of the prior free-fn / method bodies. A test pin
//! (`tests/canonical_bytes_trait.rs`) asserts byte-equality against
//! the historical encodings so any future drift fails loudly.

/// Canonical DAG-CBOR byte encoding of an identity value.
///
/// # Contract
///
/// - **Deterministic + fixed-shape.** The implementing type has a
///   schema known at compile time; encoding a well-formed value
///   cannot fail. Impls therefore `.expect(...)` on the encoder
///   rather than returning a `Result` — a failure here is a
///   programmer error (schema bug), not a runtime condition.
/// - **Signature-input hygiene.** For types whose canonical bytes are
///   a *signature input*, the encoding MUST exclude the signature
///   field itself (a value cannot sign over its own signature). Such
///   impls encode a projected subset; this is documented per-impl.
/// - **Byte-stability.** The output is v1-wire-adjacent. An impl's
///   encoding MUST NOT change without a deliberate versioned-envelope
///   migration (CLAUDE.md baked-in #5; §3.5m P-III).
pub trait CanonicalBytes {
    /// Encode `self` to its canonical DAG-CBOR byte representation.
    #[must_use]
    fn to_canonical_bytes(&self) -> Vec<u8>;
}
