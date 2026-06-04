//! Thin JSON-construction helper for the napi projection layer.
//!
//! Refinement-audit-2026-05 bundle #1201 (Pattern F: Qual-1 simplicity
//! #696/#708/#812 + Fwd-1 hot-path alloc #975/#1052/#1055). Before this
//! module, ~12 sites across `edge.rs` / `node.rs` / `subgraph.rs` /
//! `trace.rs` hand-rolled `serde_json::Map::new()` + repeated
//! `.insert("k".to_string(), Value::String(...))` chains. Each site:
//!
//! - allocated an un-primed `Map` (rehash as fields are inserted —
//!   Fwd-1 #1052: asymmetric with metrics surfaces that already prime),
//! - re-typed `serde_json::Value::String(x.clone())` per field
//!   (Qual-1 #708 verbosity),
//! - re-spelled the `Cid::to_base32()` → `Value::String` conversion at
//!   every Cid field (Fwd-1 #1055; the per-call `String` alloc is
//!   inherent to the upstream `cid` crate API — caching on `Cid`
//!   itself is out-of-lane for this crate, so this module centralizes
//!   the conversion site instead of eliminating the alloc).
//!
//! [`ObjBuilder`] is a capacity-primed object builder: callers declare
//! the field count up-front (Fwd-1 #1052 priming) and chain
//! `.str()` / `.cid()` / `.opt_str()` / `.raw()` calls that read at the
//! call site like the `{ key: value }` shape they project (Qual-1
//! readability). It is a thin wrapper over `serde_json::Map` — no new
//! allocation strategy, no behavioural change to the emitted wire
//! shape (verified by the existing `trace.test.ts` / `atrium.test.ts`
//! round-trip pins + the Rust projection tests).

use benten_core::Cid;

/// A capacity-primed [`serde_json::Map`] object builder.
///
/// Construct with [`ObjBuilder::with_capacity`] declaring the maximum
/// number of fields the object can carry (priming avoids the rehash
/// chain Fwd-1 #1052 flagged), then chain field setters. Optional
/// fields that resolve to `None` are simply not chained — the emitted
/// shape is identical to the prior hand-rolled `if let Some(..)` guards.
pub(crate) struct ObjBuilder {
    map: serde_json::Map<String, serde_json::Value>,
}

impl ObjBuilder {
    /// Allocate the backing map primed for `cap` fields.
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            map: serde_json::Map::with_capacity(cap),
        }
    }

    /// Insert a string-valued field.
    pub(crate) fn str(mut self, key: &str, value: impl Into<String>) -> Self {
        self.map
            .insert(key.to_string(), serde_json::Value::String(value.into()));
        self
    }

    /// Insert a `Cid` field rendered as its base32 string form.
    ///
    /// Centralizes the `Cid::to_base32()` → `Value::String` conversion
    /// that Fwd-1 #1055 found re-spelled 4× per `TraceStep` (×N steps).
    pub(crate) fn cid(self, key: &str, cid: &Cid) -> Self {
        self.str(key, cid.to_base32())
    }

    /// Insert a `Cid` field, emitting JSON `null` when absent.
    pub(crate) fn opt_cid(mut self, key: &str, cid: Option<&Cid>) -> Self {
        let v = match cid {
            Some(c) => serde_json::Value::String(c.to_base32()),
            None => serde_json::Value::Null,
        };
        self.map.insert(key.to_string(), v);
        self
    }

    /// Insert a string-valued field only when `value` is `Some`.
    pub(crate) fn opt_str(self, key: &str, value: Option<impl Into<String>>) -> Self {
        match value {
            Some(v) => self.str(key, v),
            None => self,
        }
    }

    /// Insert a boolean-valued field.
    pub(crate) fn bool(mut self, key: &str, value: bool) -> Self {
        self.map
            .insert(key.to_string(), serde_json::Value::Bool(value));
        self
    }

    /// Insert a `u64`-valued numeric field.
    pub(crate) fn u64(mut self, key: &str, value: u64) -> Self {
        self.map
            .insert(key.to_string(), serde_json::Value::Number(value.into()));
        self
    }

    /// Insert a pre-built `serde_json::Value` under `key`.
    pub(crate) fn raw(mut self, key: &str, value: serde_json::Value) -> Self {
        self.map.insert(key.to_string(), value);
        self
    }

    /// Insert a pre-built value only when `Some`.
    pub(crate) fn opt_raw(self, key: &str, value: Option<serde_json::Value>) -> Self {
        match value {
            Some(v) => self.raw(key, v),
            None => self,
        }
    }

    /// Finish, yielding the JSON object.
    pub(crate) fn build(self) -> serde_json::Value {
        serde_json::Value::Object(self.map)
    }
}

#[cfg(test)]
mod tests {
    use super::ObjBuilder;

    #[test]
    fn builder_emits_expected_shape_and_skips_none() {
        let v = ObjBuilder::with_capacity(4)
            .str("type", "primitive")
            .bool("ok", true)
            .u64("count", 3)
            .opt_str("present", Some("yes"))
            .opt_str("absent", Option::<String>::None)
            .build();
        assert_eq!(v["type"], "primitive");
        assert_eq!(v["ok"], true);
        assert_eq!(v["count"], 3);
        assert_eq!(v["present"], "yes");
        assert!(v.get("absent").is_none());
    }
}
