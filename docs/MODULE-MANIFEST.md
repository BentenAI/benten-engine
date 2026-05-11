# Module Manifest

**Status:** Phase 2b — format spec for `ModuleManifest`, the data
structure the engine consumes when installing a Wasm module bundle.

> Phase 2b shipped the canonical-bytes format + minimal CID-pin
> integrity gate. Phase 3 closed Compromise #21 (Ed25519 manifest
> signing landed: `signature: Option<ManifestSignature>` populated at
> install time; signature verified against the publisher DID via
> `benten-id`'s claim envelope before the module is registered).
> Phase 3 also closed Compromise #17 (durable `BlobBackend` over redb;
> wasm bytes registered with `Engine::register_module_bytes` survive
> engine restart) and Compromise #18 (durable handler-version chain
> via `core::version::Anchor`). Phase 3 landed the wasm32 IndexedDB
> manifest-store + blob-cache (PARTIALLY closed Compromise #19 —
> wasm32 arms of `apply_migration_step` + `close_database` remain
> stubs; until those wire, `BrowserManifestStore::is_persistent()`
> returns `false` honestly per the disclosure principle Compromise
> #19 articulated).

---

## 1. Why a manifest?

A Benten module manifest is the unit of capability-bounded code
distribution. One manifest declares:

- A **publisher** + **manifest name** + **version**.
- One or more **modules** (Wasm bytes addressed by their CID).
- Per-module **`requires`** capability list — the host functions the
  module's imports will resolve against.
- (Phase 3) **migrations** the install runner should execute (browser-side migration step apply remains stubbed pending the wasm32 IndexedDB `apply_migration_step` arm; Compromise #19 partial closure).
- (Phase 3) an **Ed25519 signature** field — populated by `manifest_signing::sign_manifest`, verified by `verify_manifest_with_mode` + `PublisherRegistry` at install time (Compromise #21 closure).

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
crate — see `benten-dsl-compiler` and the `host-functions.toml` codegen
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
    pub migrations: Vec<MigrationStep>,    // Phase 3 — landed (wasm32 stub)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_fns: Option<HostFnsOverride>, // Phase 3 — additive
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<ManifestSignature>, // Phase 3 — landed (Compromise #21)
}

pub struct ModuleManifestEntry {
    pub name: String,           // module-local name
    pub cid: String,            // base32 CIDv1 of the Wasm bytes
    pub requires: Vec<String>,  // ["host:compute:time", "host:fs:read"]
}

// Phase 3 — per-host-fn overrides (Compromise #16 closure). Additive
// optional carriers for fields the codegen-default surface ships with
// a default value.
pub struct HostFnsOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub random: Option<RandomHostFnOverride>,
}

pub struct RandomHostFnOverride {
    /// Per-call entropy budget in bytes. Codegen default is 4096.
    /// Manifests MAY tighten or widen this for the modules they
    /// declare; overrun fires `E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_bytes_per_call: Option<u64>,
}
```

The TypeScript shape (`packages/engine/src/types.ts`) mirrors this
field-for-field. Parity is asserted by
`packages/engine/test/manifest_schema_parity.test.ts`.

### 3.1 The `signature: Option<ManifestSignature>` field (Phase 3 — landed)

Phase 3 landed Ed25519 manifest signing end-to-end (Compromise #21
closed). `manifest_signing::sign_manifest` populates the field when an
installer signs a manifest; `Engine::install_module(manifest,
expected_cid, verify_args)` invokes `verify_manifest_with_mode`
BEFORE persisting, with `PublisherRegistry` providing the audience-bound
publisher-key lookup. UCAN-proof-chain primary + publisher-key-registry
fallback.

The `skip_serializing_if = "Option::is_none"` attribute remains
**load-bearing** for backward-compat: when `None` (an unsigned
manifest), the field is **omitted** from canonical bytes (NOT
serialized as `null`). A Phase-2b manifest whose CID was `X` today
MUST canonicalize to the same bytes after Phase-3's signing surface
landed, producing the same CID `X`. The unsigned-manifest CID stayed
stable across the Phase-2b → Phase-3 transition.

A signed re-issuance gets a **distinct** CID (it carries the
populated `signature` field, so its canonical bytes differ).

### 3.1.5 The `host_fns: Option<HostFnsOverride>` field (Phase 3)

The `host_fns` field is an **additive optional** override carrier.
Currently the only declared sub-field is `random.budget_bytes_per_call`,
which lets a manifest tighten or widen the per-call entropy budget for
the `random` host-fn (codegen default = 4096 bytes). All sub-fields are
`Option<T>` with `skip_serializing_if = Option::is_none`; when every
field is `None`, the entire struct is omitted from canonical bytes, so
a manifest with no overrides has the SAME CID before and after this
schema lift (forward-compat preserved).

Example TOML dev-time source declaring a tighter random budget:

```toml
name = "acme.entropy-tight"
version = "0.0.1"

[[modules]]
name = "main"
cid = "bafy..."
requires = ["host:random:read"]

[host_fns.random]
budget_bytes_per_call = 1024  # tighter than the 4096 default
```

A SANDBOX call against this manifest sees its 1024-byte ceiling
applied at the `random` trampoline (per
`crates/benten-engine/src/primitive_host.rs::execute_sandbox`); a
single-invocation request larger than 1024 bytes fires
`E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED`.

### 3.2 The `migrations` field on wasm32-unknown-unknown

Browser engines (`wasm32-unknown-unknown`) shipped in-memory-only
manifest persistence through Phase 2b. Phase 3 landed an
IndexedDB-backed `BrowserManifestStore` + `IndexedDbBlobBackend`
(snapshot-cache scope, under the engine's thin-compute-surface
posture). Compromise #19 is PARTIALLY closed — the wasm32 arms of
`apply_migration_step` + `close_database` remain stubs today, so
`BrowserManifestStore::is_persistent()` and
`IndexedDbBlobBackend::is_persistent()` BOTH return `false` honestly
per the disclosure principle Compromise #19 originally articulated.
Until the migration-step apply arm wires, installing a manifest with
non-empty `migrations` on `wasm32-unknown-unknown` fires
`E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE`. Native (redb-backed) engines
accept the same manifest without error.

---

## 4. Install — `Engine::install_module(manifest, expected_cid, verify_args)`

```rust
use benten_engine::module_manifest::{ManifestVerifyArgs, ManifestVerifyMode};

// Unsigned manifest path (no signature required):
let cid = engine.install_module(
    manifest,
    expected_cid,
    ManifestVerifyArgs::new(ManifestVerifyMode::Unsigned),
)?;

// Signed manifest path — Ed25519 + UCAN-proof-chain primary +
// publisher-key-registry fallback per D-PHASE-3-20:
let cid = engine.install_module(
    manifest,
    expected_cid,
    ManifestVerifyArgs::new(ManifestVerifyMode::Any).with_publisher_registry(&registry),
)?;
```

The `expected_cid` arg is **REQUIRED** — not `Option<Cid>`, not a
defaulted builder method (D16-RESOLVED-FURTHER). The compile-time
requirement closes the lazy-developer footgun where a one-arg
`install_module(m)` overload would silently compute-and-trust the CID.

The third `verify_args: ManifestVerifyArgs<'_>` parameter gates the
Phase-3 signature-verification path (Compromise #21 closure). The
`ManifestVerifyMode` variants are: `Unsigned` (only accept manifests
with no signature; reject signed manifests), `Any` (accept signed-or-
unsigned), `All` (require a valid signature). See §3.1 for the signature
surface.

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
so an Atrium sync replica can rehydrate without re-encoding. Phase 3
also landed a durable `BlobBackend` (Compromise #17 closure) so the
underlying wasm bytes referenced by `modules[*].cid` survive engine
restart on native targets.

---

## 5. Uninstall — `Engine::uninstall_module(cid)`

```rust
engine.uninstall_module(cid)?;
```

Removes the manifest from the engine's in-memory active set and writes
a `system:ModuleManifestRevocation` Node (mirrors `revoke_capability`).
The revocation Node lets an Atrium sync replica recognize the uninstall
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

These compromises are recorded against the global compromise table in
`docs/SECURITY-POSTURE.md`. Prior to R6 phase-close, this table used a
local "#N+X" numbering scheme; the entries have been lifted to global
numbering so cross-doc references resolve to a single authoritative
table.

| # | Description | Status |
|---|---|---|
| #17 | In-memory module-bytes registry (no durable BlobBackend) | CLOSED in Phase 3 |
| #18 | In-memory handler-version chain | CLOSED in Phase 3 |
| #19 | Browser-target persistent storage absent — manifests in-memory only on `wasm32-unknown-unknown` | PARTIALLY CLOSED in Phase 3 (wasm32 stubs remain for migration-apply + close-database) |
| #20 | Cross-browser determinism CI cadence not yet established | PARTIALLY CLOSED in Phase 3 |
| #21 | Module manifest Ed25519 signing | CLOSED in Phase 3 |

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
