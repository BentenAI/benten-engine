# Module Manifest

**Phase 2b G10-B.** Format spec for `ModuleManifest`, the data structure
the engine consumes when installing a Wasm module bundle.

> **Status:** Phase 2b ships the canonical-bytes format + minimal CID-pin
> integrity gate (D16-RESOLVED-FURTHER); full Ed25519 manifest signing
> defers to Phase 3 (`Compromise #N+5`). Browser (`wasm32-unknown-unknown`)
> targets ship in-memory-only manifest persistence in Phase 2b; IndexedDB
> defers to Phase 3 (`Compromise #N+8`).

---

## 1. Why a manifest?

A Benten module manifest is the unit of capability-bounded code
distribution. One manifest declares:

- A **publisher** + **manifest name** + **version**.
- One or more **modules** (Wasm bytes addressed by their CID).
- Per-module **`requires`** capability list — the host functions the
  module's imports will resolve against.
- (Phase-3 reserved) **migrations** the install runner should execute.
- (Phase-3 reserved) an **Ed25519 signature** field.

The manifest itself is a small, content-addressed, canonically-encoded
blob — **NOT** the WebAssembly bytes. The Wasm bytes are referenced by
their own CIDs from the `modules[*].cid` list.

---

## 2. Canonical encoding (D9-RESOLVED)

The canonical wire format is **DAG-CBOR** — RFC-8949 §4.2 Core
Deterministic Encoding, plus the DAG-CBOR strict subset:

- Sorted map keys (lexicographic by encoded byte representation).
- No indefinite-length items.
- Smallest-int encoding.
- No NaN payload variance.

Two logically-identical authoring inputs (e.g. JSON with different
field-order, or a TOML source vs. a TypeScript literal) MUST collapse
to the **same canonical bytes** — and therefore the same CID.

This is what makes the install-time CID pin (§4) operator-actionable
across language boundaries: a reviewer pinning a CID computed from a
TOML source MUST see the same CID an operator computes from a JSON or
TypeScript source.

### 2.1 Why DAG-CBOR (not TOML or JSON)

TOML and JSON are **dev-time ergonomic** sources only. Neither is
canonical-bytes-stable across language toolchains:

- JSON: trailing-whitespace, key-order, number-format, and string-escape
  variance all break byte-equality even when content is identical.
- TOML: comment + table-ordering + datetime-format variance ditto.

DAG-CBOR is the one encoding the Benten stack already commits to for
content-addressing (Inv-13 collision-safety; CIDv1 multicodec `0x71`).
The manifest joins the rest of the stack on it.

### 2.2 Thin TOML / JSON dev-time sources (codegen)

Operators and module authors author manifests in whatever ergonomic
format their editor handles best. A thin compiler (lives outside this
crate — see `benten-dsl-compiler` / G7-A's `host-functions.toml` codegen
pattern) parses the dev-time source and emits canonical DAG-CBOR bytes.
The CID is then BLAKE3-of-the-canonical-bytes wrapped in the standard
Benten CIDv1 envelope.

---

## 3. Schema

The Rust struct (`crates/benten-engine/src/module_manifest.rs`):

```rust
pub struct ModuleManifest {
    pub name: String,                // "acme.posts"
    pub version: String,             // "0.0.1" (semver-shaped, not parsed)
    pub modules: Vec<ModuleManifestEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub migrations: Vec<MigrationStep>,    // Phase-3 reserved
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<ManifestSignature>, // Phase-3 reserved
}

pub struct ModuleManifestEntry {
    pub name: String,           // module-local name
    pub cid: String,            // base32 CIDv1 of the Wasm bytes
    pub requires: Vec<String>,  // ["host:compute:time", "host:fs:read"]
}
```

The TypeScript shape (`packages/engine/src/types.ts`) mirrors this
field-for-field. Parity is asserted by
`packages/engine/test/manifest_schema_parity.test.ts`.

### 3.1 The `signature: Option<ManifestSignature>` reservation

The `signature` field is reserved on the struct **for Phase 3**. In
Phase 2b every install ships with `signature == None`.

The `skip_serializing_if = "Option::is_none"` attribute is **load-bearing**:
when `None`, the field is **omitted** from canonical bytes (NOT
serialized as `null`). This is the forward-compat invariant — a
Phase-2b manifest whose CID is `X` today MUST canonicalize to the same
bytes after Phase-3 lands the signing surface, producing the same CID
`X`. The unsigned-manifest CID stays stable.

A Phase-3 signed re-issuance gets a **distinct** CID (it carries the
populated `signature` field, so its canonical bytes differ).

### 3.2 The `migrations` field on wasm32-unknown-unknown

Browser engines (`wasm32-unknown-unknown`) ship in-memory-only manifest
persistence in Phase 2b — the IndexedDB / OPFS persistence story lands
in Phase 3 (`Compromise #N+8`). Because migrations need a durable
backing store to land in, installing a manifest with non-empty
`migrations` on `wasm32-unknown-unknown` fires
`E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE`. Native (redb-backed)
engines accept the same manifest without error.

---

## 4. Install — `Engine::install_module(manifest, expected_cid)`

```rust
let cid = engine.install_module(manifest, expected_cid)?;
```

The `expected_cid` arg is **REQUIRED** — not `Option<Cid>`, not a
defaulted builder method (D16-RESOLVED-FURTHER). The compile-time
requirement closes the lazy-developer footgun where a one-arg
`install_module(m)` overload would silently compute-and-trust the CID.

### 4.1 Mismatch error shape — operator-actionable diff

On `expected_cid != computed_cid`, the call returns
`E_MODULE_MANIFEST_CID_MISMATCH`. The `Display` impl renders as:

```
module manifest CID mismatch:
  expected=<base32 CID>
  computed=<base32 CID>
  (<name> v<version> modules=<n> caps=<n>)
```

Both CIDs + a 1-line manifest summary are present so an operator can
diagnose the mis-install from logs **without** a source-code dive.

The summary `caps=<n>` count is the **deduplicated** number of unique
`requires` strings across every module entry — so the operator sees the
manifest's effective capability surface, not a per-module total.

### 4.2 Idempotence

Re-installing a manifest whose CID is already in the engine's active
set is a no-op `Ok(cid)` — no second system-zone Node is written. This
matches the storage layer's `inv_13` dedup behavior.

### 4.3 System-zone storage

Installed manifests are persisted to the `system:ModuleManifest` zone
via the engine's privileged write path (mirrors `grant_capability`).
The Node carries the canonical-bytes blob under property `manifest_cbor`
so a Phase-3 sync replica can rehydrate without re-encoding.

---

## 5. Uninstall — `Engine::uninstall_module(cid)`

```rust
engine.uninstall_module(cid)?;
```

Removes the manifest from the engine's in-memory active set and writes
a `system:ModuleManifestRevocation` Node (mirrors `revoke_capability`).
The revocation Node lets a Phase-3 sync replica recognize the uninstall
even if it never observed the original install.

### 5.1 Idempotence

Uninstalling an already-uninstalled CID — or a CID that was NEVER
installed — returns `Ok(())`. The idempotence boundary is the CID, not
the install history.

### 5.2 Capability retraction (multi-manifest overlap rule)

Installing a manifest declares its `requires` capabilities into the
engine's manifest-scoped active-cap set. Uninstall retracts the
declaration **subject to multi-manifest overlap**: if another
installed manifest still requires the same capability, the cap survives
the uninstall — only the M-scoped declaration is retracted.

Pinned by `tests/integration/module_uninstall_releases_capabilities.rs`.

---

## 6. Compute helper — `Engine::compute_manifest_cid(&manifest)`

```rust
let cid = engine.compute_manifest_cid(&manifest)?;
```

Returns the canonical-bytes CID **without** installing. Used by
callers that want to verify the CID before passing it as the required
arg to `install_module`.

The TypeScript surface mirrors this with
`engine.computeManifestCid(manifest)`.

---

## 7. Compromises documented

| # | Description | Closes |
|---|---|---|
| #N+5 | Module manifest minimal CID-pin in Phase 2b; full Ed25519 deferred to Phase 3 | Phase 3 (D16) |
| #N+8 | Browser-target persistent storage absent — manifests in-memory only on `wasm32-unknown-unknown` | Phase 3 (IndexedDB) |

---

## 8. Test pins

| Test | Location |
|---|---|
| `module_install_persists_in_system_zone` | `crates/benten-engine/tests/module_install.rs` |
| `install_module_requires_cid_arg_at_compile_time` (D16) | `crates/benten-engine/tests/module_install.rs` |
| `install_module_rejects_cid_mismatch_with_dual_cid_diff_in_error` (D16) | `crates/benten-engine/tests/install_module_rejects_cid_mismatch.rs` + `tests/integration/install_module_rejects_cid_mismatch.rs` |
| `module_uninstall_respects_capability_retraction` | `crates/benten-engine/tests/module_uninstall.rs` + `tests/integration/module_uninstall_releases_capabilities.rs` |
| `module_install_round_trip_5_row_fixture_matrix` | `crates/benten-engine/tests/integration/module_install_uninstall_round_trip.rs` |
| `manifest_canonical_bytes_dagcbor` (D9) | `crates/benten-engine/tests/module_manifest_canonical.rs` |
| `manifest_ts_validates_against_rust_schema` | `packages/engine/test/manifest_schema_parity.test.ts` |
| `module_manifest_doc_present` | `crates/benten-engine/tests/module_manifest_doc_present.rs` |
| `module_manifest_signature_field_omitted_from_canonical_bytes_when_none` (D9 + D16) | `crates/benten-engine/tests/module_manifest_signature_field_reserved.rs` |

---

## 9. Forward-compat checklist (Phase 3)

When Phase 3 lands the full surface:

- [ ] Wire Ed25519 signing — populate `signature: Some(ManifestSignature { ed25519: Some(...) })`. CID for signed manifests is **distinct** from the unsigned CID.
- [ ] Wire IndexedDB / OPFS persistence on `wasm32-unknown-unknown`; lift the migrations rejection (`E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE`).
- [ ] Wire the migrations runner (currently the `MigrationStep` shape is reserved but no runner consumes it).
- [ ] Treat manifest revocation Nodes as authoritative across sync (currently the active set is in-memory only post-engine-open).
