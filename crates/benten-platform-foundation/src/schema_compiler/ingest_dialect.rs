//! Ingest dialect adapters — translate input source dialects into the
//! canonical schema-JSON dialect the [`super::parse`] module consumes.
//!
//! G23-A canary supports only the **canonical dialect** (the JSON shape
//! described at the top of `parse.rs`). Future ingest dialects land at
//! later G23-A waves:
//!
//! - JSON-Schema standard (Draft 2020-12) — translator wave-4b
//! - TypeScript DSL — wave-4c (parses TS literal source via napi-rs)
//! - Python ingest — Phase-6+ exploratory
//!
//! The translator-pattern is: dialect-bytes → canonical-JSON-bytes →
//! existing [`super::compile`] pipeline. The dialect translators do NOT
//! emit Subgraphs directly; they only normalize input shape. This
//! preserves a single emit-site for cap-scope derivation + 12-primitive
//! composition (sec-3.5-r1-4 + CLAUDE.md baked-in #1).

use super::SchemaCompileError;

/// The set of supported ingest dialects. G23-A canary registers only
/// the canonical dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IngestDialect {
    /// The canonical JSON shape (`{ "label": "SchemaRoot", "name": ...,
    /// "fields": [...] }`). The parser in [`super::parse`] consumes this
    /// directly.
    Canonical,
}

impl IngestDialect {
    /// Detect the input dialect from a heuristic peek at the leading bytes.
    /// G23-A canary returns [`IngestDialect::Canonical`] for any JSON-ish
    /// input; future waves add real detection (e.g. presence of
    /// `$schema` → JSON-Schema standard).
    #[must_use]
    pub fn detect(_bytes: &[u8]) -> Self {
        IngestDialect::Canonical
    }

    /// Translate input bytes to canonical-form bytes. G23-A canary is the
    /// identity translator for `Canonical`. Later waves wire real
    /// translators here.
    pub fn translate_to_canonical(self, bytes: &[u8]) -> Result<Vec<u8>, SchemaCompileError> {
        match self {
            IngestDialect::Canonical => Ok(bytes.to_vec()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_canonical_at_canary() {
        assert_eq!(IngestDialect::detect(b"{}"), IngestDialect::Canonical);
    }

    #[test]
    fn canonical_translates_identity() {
        let input = b"{ \"label\": \"SchemaRoot\" }";
        let out = IngestDialect::Canonical.translate_to_canonical(input).unwrap();
        assert_eq!(out, input);
    }
}
