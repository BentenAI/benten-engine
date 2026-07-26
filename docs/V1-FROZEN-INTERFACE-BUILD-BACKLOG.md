# V1-FROZEN-INTERFACE BUILD-BACKLOG — pre-freeze build-out items

> **Companion artifact to [`docs/V1-FROZEN-INTERFACE.md`](V1-FROZEN-INTERFACE.md).**
>
> This document enumerates the 7 cross-confirmed FIX-NOW pre-freeze
> build-out items that MUST land BEFORE the freeze contract can lock at
> `phase-4-meta-core-close`. Both planner drafts surfaced these as gaps
> at HEAD `ae7cd3d5`; orchestrator triage verified each is NOT
> already-built and slots them as the fix-up wave that runs immediately
> after this triage-synthesis lands.
>
> **Format per row:** (i) target file/symbol, (ii) what to build, (iii)
> verification, (iv) LOC estimate, (v) dependencies. The next-dispatched
> fix-up agent consumes this as a checklist.
>
> **Base:** `origin/main` @ `ae7cd3d5` post-#1342. Verified at triage
> time 2026-05-23.

---

## Row 1 — `cargo-public-api` baseline regeneration + 3 missing baselines

**Target files/symbols:**
- All 11 existing baselines at `docs/public-api/benten-*.{txt,json}`
  (currently 11-LOC G20-A3 seed stubs — verified via `wc -l`: 8 are
  11-LOC stubs; only `benten-id.json` / `benten-renderer-tauri.json` /
  `benten-sync.json` may carry real content).
- 3 NEW baselines required (verified missing at HEAD via `ls
  docs/public-api/`):
  - `docs/public-api/benten-crypto-suite.txt`
  - `docs/public-api/benten-drop.txt`
  - `docs/public-api/benten-platform-foundation.txt`

**What to build:**

For every workspace public crate (14 total — verify via `cargo metadata
--format-version 1 | jq '.workspace_members'`), run:

```
cargo public-api -p <crate-name> --simplified --omit blanket-impls
```

and commit the full output to `docs/public-api/<crate-name>.{txt|json}`
(preserving each crate's existing baseline format choice). For the 3
missing crates, choose `.txt` format (consistency with majority).

**Sub-task 1.a — napi cascade migration (couples to item 1 §8-A tighten,
F-1 contract):**
- `bindings/napi/src/*.rs` sweep for `engine.get_node(...)` /
  `engine.put_node(...)` / `engine.get_node_label_only(...)` /
  `engine.resolve_subgraph_cid_for_test(...)` calls.
- Refactor each to:
  - `engine.read_node_as(principal, cid)` for principal-bearing reads
    (likely needs an internal-principal CID const for sites that need
    un-attributed reads at the napi layer — design call at wave time).
  - `engine.transaction().put_node(...)` for writes (or the per-shape
    public wrapper).
- Atomic G-CORE-9 commit-set per §8-A's "applied atomically"
  requirement.

**Sub-task 1.b — test-site sweep (couples to item 1 §8-A tighten):**
- Migrate test-only callers: `crates/benten-eval/tests/read_denial.rs:96/100`,
  `crates/benten-engine/tests/inv_11_*.rs:93/155/159`,
  `crates/benten-engine/tests/noauth_startup_log.rs:47`, plus any
  others surfaced by the workspace build.
- Replace direct `Engine::get_node` / `put_node` /
  `resolve_subgraph_cid_for_test` calls with the `testing` module
  pub(crate) helpers OR `read_node_as(ENGINE_INTERNAL_PRINCIPAL_CID, ...)`.

**Sub-task 1.c — no-regression test pin:**
- Author `crates/benten-engine/tests/g_core_9_engine_no_direct_cap_mutation.rs`
  that uses `cargo-public-api` output to assert ZERO cap-mutation
  methods on `Engine` (other than `caps()`).

**Verification:**
- The existing `crates/benten-engine/tests/cargo_public_api_drift.rs`
  passes against the new (real) baselines.
- `cargo +stable clippy --workspace --all-targets -- -D warnings` clean.
- The new no-regression test pin compiles + passes.
- `cargo doc --workspace --no-deps` clean.

**LOC estimate:** ~800-1500 LOC total. Real cargo-public-api output for
14 crates typically runs ~100-500 LOC per baseline depending on crate
surface area; the napi cascade + test-site sweep is estimated at
~50-150 LOC across ~10-15 files.

**Dependencies:** none — this is the canonical FIRST row. Sub-task 1.a
+ 1.b couple to item 1 §8-A tighten (must land in same atomic commit
set OR sub-task 1.a/1.b lands first under a feature-flag, then 1
flips the `pub` → `pub(crate)`).

---

## Row 2 — #1204 TS-side public-API parity gate

**Target files/symbols:**
- `.github/workflows/ts-public-api.yml` (new file).
- `packages/engine/etc/engine.api.md` (or equivalent baseline) — new
  file.
- `packages/engine/api-extractor.json` (or equivalent config) — new
  file.

**What to build:**

`@microsoft/api-extractor` workflow per item 10's three-stage sequential
placement: BUILT in G-CORE-10 / **COMMITTED + FREEZE-FLIPPED in G-CORE-
9**.

Workflow shape:

```yaml
# .github/workflows/ts-public-api.yml
name: ts-public-api
on: [push, pull_request]
jobs:
  ts-public-api-drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: {node-version: '20'}
      - run: pnpm install --frozen-lockfile
      - run: pnpm --filter @benten/engine build
      - run: pnpm --filter @benten/engine api-extractor run --verify
      # `--verify` fails the build on any drift vs the committed baseline.
```

Author the baseline by running api-extractor in `--local` mode against
the post-freeze `dist/index.d.ts` and committing the resulting
`etc/engine.api.md`.

**Verification:**
- The workflow runs green against the post-freeze `index.d.ts`.
- A deliberate test mutation (e.g. add a new export to `index.ts`)
  causes the workflow to fail (smoke-test).
- Test runs alongside `cargo-public-api` drift test as parallel
  structural backstop.

**LOC estimate:** ~200 LOC (workflow file + api-extractor config + the
generated `engine.api.md` baseline file is auto-generated; size depends
on TS surface but should be ~500-2000 LOC of auto-content).

**Dependencies:**
- Requires `pnpm` workspace + `@microsoft/api-extractor` installed as
  dev-dep in `packages/engine/package.json`.
- The post-freeze `index.d.ts` must exist (regenerated as part of the
  freeze wave — couples to napi-rs build pipeline).

---

## Row 3 — `EncryptionClass` enum mint

**Target file/symbol:**
- `crates/benten-core/src/lib.rs` OR `crates/benten-caps/src/lib.rs` —
  decide based on §8-CC consumer surface enumeration at wave time
  (probably `benten-core` because it's a vocabulary-of-encryption-state
  type, not a capability-policy type).

**What to build:**

```rust
/// Encryption class for content stored under the per-DID partition.
///
/// Distinguishes between public-readable content (no confidentiality
/// envelope) and confidential content (encrypted per the #1301
/// substrate; reader must hold the appropriate
/// [`AuthorizationGrant`]'s `key_material`).
///
/// **`#[non_exhaustive]`** per V1-FROZEN-INTERFACE item 15(e):
/// reserved future variants (`AnonymousGroup`, `PrivateLocal`) are
/// documented but NOT built at v1-beta; explicit-add via additive
/// enum variants post-v1.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EncryptionClass {
    /// Public — content visible to any reader with the CID; no
    /// confidentiality envelope.
    Public,
    /// Confidential — content encrypted per §1301 substrate; reader
    /// must hold the appropriate [`AuthorizationGrant`]'s
    /// `key_material`.
    Confidential,
    // Reserved future arms (NOT built at v1-beta; documented for
    // forward planning):
    //   AnonymousGroup,   // group-keyed without revealing principal identity
    //   PrivateLocal,     // device-local-only; never sync-eligible
}
```

Add wire-format codepoint mapping (per item 14 IANA-reuse policy + the
#5 crypto-agility framing): each variant maps to a stable wire
codepoint (e.g. `Public = 0x00`, `Confidential = 0x01`); typed-reject
on unknown class via the `UnsupportedAlgorithm`-equivalent pattern.

**Verification:**
- A new test pin (`crates/benten-core/tests/g_core_9_encryption_class_typed_reject.rs`
  OR equivalent in `benten-caps`) asserts:
  - Both `Public` and `Confidential` round-trip through their wire codepoints.
  - An unknown codepoint surfaces a typed reject error (not panic, not
    silent fallback).
- `cargo-public-api` baseline includes the enum (couples to row 1).
- §8-CC consumer code (if any exists at wave time; AUDIT) compiles
  against the new type.

**LOC estimate:** ~80 LOC (enum + impls + 1 test file).

**Dependencies:**
- Couples to item 4 (wire-format) — the codepoint assignment requires a
  P-III decision-point entry. Coordinate with Ben on whether `Public =
  0x00 / Confidential = 0x01` is the right initial assignment OR
  defers to the inventory in row 8 below.

---

## Row 4 — Verify-or-build `Engine::walk_share_scope`

**Target file/symbol:**
- `crates/benten-engine/src/` — the SubgraphSpec walker entry-point
  consumer surface (engine-side wrapper around
  `benten_core::subgraph_spec::walker::walk`).

**What to build:**

**Verification step FIRST:**
```bash
grep -rn "fn walk_share_scope\|fn share_scope_walk\|pub fn walk" crates/benten-engine/src/
grep -rn "subgraph_spec::walker::walk" crates/benten-engine/src/
```

At triage time (2026-05-23): only narrative reference exists at
`crates/benten-core/tests/tf3w_walker_is_a_subgraph_no_new_primitive_kind.rs:208`
mentioning `Engine::walk_share_scope()`-style; production engine method
appears NOT YET MINTED.

**If exists under different name → freeze the actual name (no build needed;
update V1-FROZEN-INTERFACE.md item 15(h) with the real name).**

**If doesn't exist → build the entry point** (~25-50 LOC BFS
orchestration per spec). The walker is itself a Subgraph composed of
READ/WRITE/TRANSFORM/BRANCH/ITERATE primitives — the engine wrapper
calls into `benten_core::subgraph_spec::walker::walk(spec)`:

```rust
impl Engine {
    /// Walk the SubgraphSpec rooted at the given root CIDs + return
    /// the BFS-enumerated `(Cid, StructuralPath)` set per RATIFIED-S&C
    /// §R4. The walk is data-not-evaluator-extension: the engine
    /// delegates to the `benten_core::subgraph_spec::walker` (which IS
    /// itself a Subgraph composed of the existing 12 primitives — no
    /// new `PrimitiveKind` variant).
    pub fn walk_share_scope(
        &self,
        spec: &benten_core::subgraph_spec::Spec,
        principal: &Cid,
    ) -> Result<benten_core::subgraph_spec::WalkResult, EngineError> {
        // Delegate to the data-walker; engine provides the graph
        // backend the walker queries through.
        benten_core::subgraph_spec::walker::walk(spec)
            .map_err(EngineError::from)
    }
}
```

**Verification:**
- `crates/benten-engine/tests/g_core_9_walk_share_scope_e2e.rs` —
  end-to-end test asserting BFS enumeration matches the expected
  `(cid, path)` set against a synthetic test subgraph.
- `cargo-public-api` baseline includes the new method.
- `tf3w_walker_is_a_subgraph_no_new_primitive_kind.rs` continues to
  pass.

**LOC estimate:** ~80 LOC if mint-needed (engine method + 1 test); ~0
LOC if rename-only.

**Dependencies:** none directly; engages with item 15.h freeze.

---

## Row 5 — Verify-or-build `MerkleRangeProofBackend` trait

**Target file/symbol:**
- `crates/benten-sync/src/` — likely a new module
  `merkle_range_proof_backend.rs` (per RATIFIED-PREWORK §8-B placement
  decision: above storage in `benten-sync`, NOT in `benten-graph`).

**What to build:**

**Verification step FIRST:**

At triage time (2026-05-23): verified via grep — `MerkleRangeProofBackend`
appears only as narrative references in:
- `crates/benten-graph/tests/tf11_snapshot_blob_schema_version_p3_migration_path.rs:39`
  (test narrative)
- `crates/benten-graph/src/backends/snapshot_blob.rs:24,29,154`
  (narrative + a docstring placeholder)
- `crates/benten-engine/src/engine_snapshot.rs:169` (placeholder hook)

NO actual trait exists at HEAD.

**Decision tree:**

**Option A (defer to G-COMP-1 per Planner-B):** Acknowledge that
freezing a phantom shape is overcommit. V1-FROZEN-INTERFACE item 3 /
§4.64 stays as **NOT FROZEN at v1-beta; BELONGS-NAMED-NOW G-COMP-1
§8-B-in-`benten-sync`**. Update the contract doc to remove the
freeze-now language for `MerkleRangeProofBackend`. **0 LOC build; 1
doc edit.**

**Option B (build at G-CORE-9 per Planner-A):** Mint the trait shape
following the placement decision:

```rust
// crates/benten-sync/src/merkle_range_proof_backend.rs

/// Light-client mode (b) substrate: produce Merkle-tree range proofs
/// for content-addressed slice queries. Sits above the storage layer
/// in `benten-sync` per RATIFIED-PREWORK §8-B (b).
///
/// Trait shape is the v1-beta freeze; method bodies are
/// implementation-defined per backend (the IROH-based default impl
/// ships in a downstream crate or under a feature flag).
pub trait MerkleRangeProofBackend: Send + Sync + 'static {
    /// Construct a Merkle range proof for the contiguous byte-range
    /// `[start..end)` of the named CID's content. Returns a
    /// verifiable proof artifact + the proof's authenticated bytes.
    fn build_range_proof(
        &self,
        cid: &Cid,
        start: u64,
        end: u64,
    ) -> Result<RangeProof, MerkleRangeProofError>;

    /// Verify a range proof against the CID + range.
    fn verify_range_proof(
        &self,
        cid: &Cid,
        proof: &RangeProof,
    ) -> Result<(), MerkleRangeProofError>;
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct RangeProof {
    pub authenticated_bytes: Vec<u8>,
    pub proof_path: Vec<[u8; 32]>,
    pub root_hash: [u8; 32],
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum MerkleRangeProofError {
    #[error("range outside content bounds: requested {requested:?}, content len {content_len}")]
    OutOfBounds { requested: std::ops::Range<u64>, content_len: u64 },
    #[error("proof verification failed: {reason}")]
    VerificationFailed { reason: String },
    #[error("backend error: {0}")]
    Backend(#[from] Box<dyn std::error::Error + Send + Sync>),
}
```

**Recommended path:** Option A (defer) is the conservative-correct call
per HARD RULE 12 clause-(b) BELONGS-NAMED-NOW destination is named +
the destination exists; Option B requires more design work that should
go through its own R0 pre-work mini-pipeline. **DEFAULT: Option A
unless wave-time triage surfaces a concrete §4.64 consumer at HEAD that
needs the trait NOW.**

**Verification:**
- (Option A) Update V1-FROZEN-INTERFACE.md item 3 §4.64 narrative to
  explicitly state "BELONGS-NAMED-NOW G-COMP-1 §8-B-in-`benten-sync`";
  add a tracking row in `docs/future/phase-4-backlog.md` (or
  equivalent).
- (Option B) `cargo-public-api` baseline includes the new trait; trait
  object-safety compile-test pin; smoke-test for the
  `RangeProof`/`MerkleRangeProofError` shape.

**LOC estimate:**
- Option A: ~10 LOC doc-edits.
- Option B: ~150 LOC (trait + types + tests).

**Dependencies:** if Option B, requires R0 design call about
`RangeProof` wire shape (item 4 P-III coupling).

---

## Row 6 — `CapabilityPolicy` hard-seal promotion

**Target file/symbol:**
- `crates/benten-caps/src/policy.rs` — promote the soft-seal marker
  pattern at `policy.rs:14-67` to a true private `Sealed` supertrait.
- New module: `crates/benten-caps/src/policy_sealed.rs` (non-pub at
  crate root; `pub(crate)` exposure of the Sealed trait only).
- ~20 test sites across the workspace that implement
  `CapabilityPolicy` for test-double policies (per `policy.rs:25-29`
  enumerated list: `benten-engine/tests/*`, `benten-caps/tests/*`,
  `benten-platform-foundation/tests/*`, `benten-eval/tests/*`,
  `benten-engine/src/testing.rs`).

**What to build:**

1. **Mint private `Sealed` trait + integration with `CapabilityPolicy`:**

```rust
// crates/benten-caps/src/policy_sealed.rs (non-pub module at crate root)
pub(crate) mod private {
    pub trait Sealed {}
}

// crates/benten-caps/src/policy.rs (top of file)
use crate::policy_sealed::private::Sealed;

pub trait CapabilityPolicy: Sealed + Send + Sync {
    // ... existing trait body unchanged ...
}
```

2. **Add `impl Sealed for X {}` to every internal `CapabilityPolicy`
   implementer:**
   - `NoAuthBackend` (production default).
   - `GrantBackedPolicy`.
   - `UcanGroundedPolicy`.
   - Every test-double policy across the ~20 workspace test sites.

3. **Compile-fail trybuild test:**
   - `crates/benten-caps/tests/compile_fail/external_cap_policy_impl.rs`
     — asserts that an external `impl CapabilityPolicy for
     SomeExternalType` without `impl Sealed for SomeExternalType` fails
     to compile.
   - **DEFERRED to G-COMP-1 per `docs/V1-FROZEN-INTERFACE-DEFERRED.md`
     Row D-20** (added at G-CORE-9 R2 fix-pass); the hard-seal MECHANISM
     ships in this row + is structurally enforced by rustc on every
     workspace build, but the explicit compile-fail trybuild test fixture
     is the regression-defense backstop named-deferred to G-COMP-1.

4. **Object-safety preserved:**
   <!-- cite-drift-exempt: forward-looking acceptance criterion; the
        object_safety_*.rs test family is to-be-authored as part of this
        very row's wave landing, NOT a HEAD-cite. -->
   - `crates/benten-caps/tests/object_safety_*.rs` continues to pass <!-- cite-drift-exempt -->
     (`Arc<dyn CapabilityPolicy>` boxing).

5. **Delete the old soft-seal marker** at `policy.rs:48-67` (the
   `sealed_marker::SealedCapabilityPolicy` empty marker trait); no
   deprecation alias per HARD RULE 12 + CLAUDE.md #5 no-shims.

6. **Update `crates/benten-caps/INTERNALS.md` §9** to reflect the
   hard-seal landed (was: "G-CORE-8.3 follow-up wave"; becomes:
   "shipped at G-CORE-9 V1-FROZEN-INTERFACE row 6").

**Verification:**
- `cargo +stable clippy --workspace --all-targets -- -D warnings` clean.
- `cargo nextest run --workspace` green (every test-impl carries the
  Sealed marker).
- The compile-fail trybuild test fails-to-compile as expected (smoke-
  tests the seal). **DEFERRED to G-COMP-1 per
  `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-20**; at v1-beta the
  hard-seal mechanism is verified by `cargo check -p benten-caps` (rustc
  refuses external impls because `Sealed` is `pub(crate)`).
- `cargo-public-api` baseline for `benten-caps` (row 1) does NOT expose
  `Sealed` publicly (verify `pub(crate)` visibility).
- `crates/benten-caps/tests/object_safety_*.rs` passes <!-- cite-drift-exempt: forward-looking acceptance criterion per the same disposition above. -->

**LOC estimate:** ~250-400 LOC (mostly mechanical `impl Sealed` adds
across ~20 test sites + the new module + the compile-fail test +
INTERNALS.md update).

**Dependencies:**
- Couples to V1-FROZEN-INTERFACE item 8.
- Couples to row 1 (`cargo-public-api` baseline regen for `benten-caps`
  picks up the new `: Sealed + Send + Sync` bound).

---

## Row 7 — Name-collision rename (RestrictedSpec + KeyMaterial)

**Target files/symbols:**
- `crates/benten-core/src/subgraph_spec/spec.rs:115` — rename
  `pub enum RestrictedSpec` → `pub enum SubgraphSpecRestriction`.
- `crates/benten-caps/src/restricted_spec.rs:103` — rename
  `pub struct RestrictedSpec` → `pub struct RestrictedScope`.
- `crates/benten-crypto-suite/src/aead.rs:91` — rename
  `pub struct KeyMaterial` → `pub struct AeadKeyMaterial`.
- `crates/benten-caps/src/authorization_grant.rs:178` — rename
  `pub struct KeyMaterial` → `pub struct GrantKeyMaterial`.

**What to build:**

Atomic rename across all 4 types + every caller site + every test +
every doc/cite. Use `git grep` to enumerate caller sites:

```bash
git grep -l "RestrictedSpec" crates/ docs/ packages/ bindings/
git grep -l "KeyMaterial" crates/ docs/ packages/ bindings/
```

Apply renames mechanically (no shim, no `pub use` alias — per CLAUDE.md
#5 no-shims discipline). Update:
- All `use` statements + paths.
- All `match` arm patterns referring to the old names (e.g.
  `Scope::RestrictedSelector(RestrictedSpec)` →
  `Scope::RestrictedSelector(RestrictedScope)`).
- All rustdoc cite-paths (e.g. `[\`RestrictedSpec\`]` →
  `[\`RestrictedScope\`]`).
- Test names that bake in the old name (e.g.
  `tf3b_restricted_spec_containment.rs` → `tf3b_restricted_scope_containment.rs`
  OR keep test filename but update test names — wave-time call).
- TS-side mirrors in `packages/engine/src/types.ts` (any TS interface
  that mirrors these types).
- Drift-detect scanner expectations (if `RestrictedSpec` or
  `KeyMaterial` appears in the scanner config).

**Verification:**
- `cargo +stable clippy --workspace --all-targets -- -D warnings` clean.
- `cargo nextest run --workspace` green.
- `cargo-public-api` baselines (row 1) reflect the new names; no old
  names appear.
- `cargo doc --workspace --no-deps` clean (no broken rustdoc cites).
- `grep -rn "RestrictedSpec\|KeyMaterial" crates/` returns ZERO matches
  for the old names (sweep completeness).

**LOC estimate:** ~30-60 LOC of actual edits across ~15-30 files
(mostly trivial sed-style renames; the spec body + impl bodies
unchanged).

**Dependencies:**
- Couples to V1-FROZEN-INTERFACE items 15.a + 15.d.
- **Tentatively-decided per night-shift stance.** Ben may rebut at
  morning review; if rebutted, skip this row + update the contract doc
  to keep the duplicate names.
- Couples to row 1 (`cargo-public-api` baseline regen picks up the
  renames).

---

## Row 8 — TS errors.generated.ts ↔ Rust ErrorCode parity audit (+ ManifestEnvelopeRecheckOutcome non_exhaustive + WriteContext non_exhaustive)

> **Bundle row** — small mechanical sweep items that orchestrator
> triage surfaced as ALSO required pre-freeze. Bundled here to avoid
> dispatch-thrash.

**Target files/symbols:**

8.a — **TS class count drift investigation:**
- `packages/engine/src/errors.generated.ts` — 194 classes (verified
  via `grep -c "extends BentenError\|class.*BentenError"`).
- `crates/benten-errors/tests/stable_shape.rs` — 191 Rust variants
  (verified via awk count of `ErrorCode::` in `ALL_CATALOG_VARIANTS`).
- **Delta: +3 TS classes**. Either legitimate envelope classes
  (`BentenError` base + `BentenInternalError` etc.) or genuine drift.

**Investigation steps:**
1. Enumerate the 194 TS classes via the file's `export class` pattern.
2. Enumerate the 191 Rust ErrorCode variants via `ALL_CATALOG_VARIANTS`.
3. Diff. The 3 TS-only classes are either:
   - **Legitimate envelopes** (e.g. `BentenError` base, `BentenInternalError`,
     `BentenWrappedError`) — these are infrastructure, not catalog
     variants → documented exclusion list updated.
   - **Genuine drift** — TS class added without the Rust ErrorCode →
     either remove the TS class OR mint the Rust variant (HARD RULE 12
     FIX-NOW).
4. Update the drift-detect scanner config to enforce the
   investigation's outcome (legitimate-envelope set explicit; rest
   must match 1:1).

8.b — **`ManifestEnvelopeRecheckOutcome` `#[non_exhaustive]` apply:**
- `crates/benten-engine/src/manifest_envelope_recheck.rs:80` — verified
  MISSING `#[non_exhaustive]` at HEAD.
- Apply the attribute; couples to V1-FROZEN-INTERFACE items 11 + 12.

8.c — **`WriteContext` `#[non_exhaustive]` apply:**
- `benten_graph::WriteContext` (in `crates/benten-graph/src/lib.rs`) — verified MISSING at HEAD.
- Apply the attribute; couples to V1-FROZEN-INTERFACE items 5 + 11.

8.d — **`GraphError::TxAborted` per-variant `#[non_exhaustive]` audit + apply:**
- `benten_graph::GraphError` (in `crates/benten-graph/src/lib.rs`) — `GraphError` already has
  it at the enum level; per-variant on `TxAborted` may be missing.
- Audit + apply defensively per item 11.

8.e — **Workspace `#[non_exhaustive]` sweep test pin (BUILD-NEW):**
- `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs` — new
  test file that walks every `pub enum` + `pub struct` workspace-wide
  and asserts each carries `#[non_exhaustive]` OR appears in a documented
  carve-out registry. Per V1-FROZEN-INTERFACE item 11.

8.f — **Wire-format inventory + Ben P-III decision doc author (BUILD-NEW):**
- `docs/V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md` — new tracked file.
- Enumerates: every wire-format-bearing surface (CBOR envelopes; AEAD
  wrap; UCAN-Varsig; DropBundle; SnapshotBlob; TwoCidStore mapping;
  signature envelopes); each with explicit `format_version: u32`
  discriminator; list of surfaces lacking byte-pin tests.
- Per V1-FROZEN-INTERFACE item 4.
- **This is a Ben-signed deliverable**; orchestrator authors the
  inventory; Ben signs the freeze decision.

8.g — **SECURITY-POSTURE Compromise # assignment for revocation reach:**
- Next compromise number assignment + entry per V1-FROZEN-INTERFACE
  item 15(i).

**Verification:**
- For 8.a: `scripts/drift-detect-error-variant-mirror.ts` (or
  equivalent) passes with the updated parity expectation.
- For 8.b/8.c/8.d: `cargo +stable clippy --workspace --all-targets --
  -D warnings` clean (the `non_exhaustive_omitted_patterns` lint fires
  on any consumer that needs to update).
- For 8.e: the new test pin compiles + passes; deliberately removing
  `#[non_exhaustive]` from a covered type fails the test (smoke).
- For 8.f: Ben signs the decision doc.
- For 8.g: SECURITY-POSTURE.md has the new entry + the Compromise # is
  unique.

**LOC estimate:**
- 8.a: ~50 LOC investigation + scanner config tweak.
- 8.b/8.c/8.d: ~6 LOC (3 attribute applies + any callsite
  match-arm updates the lint surfaces).
- 8.e: ~200 LOC (the audit test pin + the carve-out registry).
- 8.f: ~300 LOC (inventory doc; mostly enumeration of existing
  wire-format-bearing surfaces).
- 8.g: ~20 LOC (SECURITY-POSTURE.md edit + Compromise # entry).
- **Total: ~575 LOC.**

**Dependencies:**
- 8.e couples to row 1 (`cargo-public-api` baselines provide the
  enumeration source).
- 8.f is a Ben-gated deliverable; orchestrator-authors but Ben-signs.
- All sub-rows couple to the freeze contract; should land in the same
  fix-up wave or in tight sequence.

---

## Dispatch ordering recommendation

The 7 build-backlog rows are NOT all independent. Recommended sequencing
for the fix-up wave:

1. **Row 7 (renames)** FIRST — mechanical sweep; everything downstream
   sees the new names. Tentatively-decided per night-shift; if Ben
   rebuts at morning, skip this row + update V1-FROZEN-INTERFACE.md
   accordingly.

2. **Row 6 (hard-seal CapabilityPolicy)** SECOND — workspace-wide
   migration; large but contained to test-double impls. Independent
   of other rows except renames.

3. **Rows 3 + 4 + 5 (EncryptionClass mint + walk_share_scope verify-
   or-build + MerkleRangeProofBackend verify-or-defer)** THIRD —
   independent of each other; dispatch in parallel; each is a small
   targeted mint OR a doc update.

4. **Row 8 (bundle of sweep items)** FOURTH — couples to the above
   landings; the `#[non_exhaustive]` audit test pin (8.e) and the
   wire-format inventory (8.f) benefit from settled type names + the
   `Sealed` post-promotion shape.

5. **Row 1 (cargo-public-api baselines + napi cascade + test-site
   sweep)** LAST — captures the FINAL public-API state after rows
   1-4's landings. Sub-task 1.a (napi cascade) couples to item 1 §8-A
   tighten and MUST land atomically with the visibility tighten
   (cannot regenerate baselines before the tighten lands; cannot
   tighten before napi/test sites migrate).

6. **Row 2 (TS parity gate)** PARALLEL with row 1 — independent of
   the Rust side; can dispatch any time after row 8.a's TS class
   investigation lands.

**Parallelism budget:** at most 7 in-flight implementer agents per
CLAUDE.md §13 cap. The above sequencing fits within that budget at
every step.

**Estimated total wave LOC:** ~2200-3000 LOC across the 7 rows (Row 1's
auto-generated baselines dominate at ~800-1500 LOC of mostly-mechanical
text).

---

## Status

| Row | Item | Status |
|---|---|---|
| 1 | cargo-public-api baselines + napi cascade + test-site sweep | **PARTIALLY LANDED** (commit `fb7c212d`) — 14 baselines regenerated; CI workflow expanded 8→14 crates. Sub-tasks 1.a (napi cascade) + 1.b (test-site sweep) + 1.c (no-regression test pin) NAMED for the next follow-up sub-pass per HARD RULE 12 BELONGS-NAMED-NOW (visibility-tighten work; current baselines reflect HEAD surface which is the right freeze-time snapshot). |
| 2 | #1204 TS parity gate | **LANDED** (commit `13322df4`) — workflow at `.github/workflows/ts-public-api.yml`; baseline at `packages/engine/etc/public-api.txt` (403 LOC; extract-from-.d.ts structural diff). api-extractor migration NAMED for v1-Composing. |
| 3 | EncryptionClass enum mint | **LANDED** (commit `a9d2753c`) — `pub enum EncryptionClass { Public, Confidential }` at `crates/benten-core/src/encryption_class.rs:36` + codepoint table + typed-reject dispatch + 4 unit tests. Annotated `drift-detect-mirror: ignore` for `EncryptionClassError` (internal-only until v1-Composing §8-CC consumer wires up). |
| 4 | Engine::walk_share_scope verify-or-build | **LANDED** (commit `7af94d06`) — `Engine::walk_share_scope` minted at `crates/benten-engine/src/engine_share_scope.rs:46` + new `ErrorCode::SubgraphSpecWalkFailed` (CATALOG_VARIANT_COUNT 191 → 192) + end-to-end test pin. |
| 5 | MerkleRangeProofBackend verify-or-defer | **DEFERRED to G-COMP-1 per Option A** (commit `d2616800`) — verified trait does NOT exist at HEAD; deferral named in `docs/future/phase-4-backlog.md §4.64`. |
| 6 | CapabilityPolicy hard-seal promotion | **LANDED** (commit `5ce8bab6`) — hard-seal via `pub(crate) mod sealed { pub trait Sealed {} }`; `pub trait CapabilityPolicy: sealed::Sealed + Send + Sync`. Old soft-seal marker DELETED. 4 internal + ~17 workspace test-double impls migrated. `benten-caps/testing` feature added for the workspace-test re-export. |
| 7 | Name-collision renames (RestrictedSpec + KeyMaterial) | **LANDED** (commit `dd12f394`) — `subgraph_spec::RestrictedSpec` → `SubgraphSpecRestriction`; `caps::RestrictedSpec` → `RestrictedScope`; `crypto_suite::aead::KeyMaterial` → `AeadKeyMaterial`; `caps::KeyMaterial` → `GrantKeyMaterial`. 36 files / 229 insertions / 229 deletions. Tentatively-decided per night-shift stance; rebuttable at morning Ben review. |
| 8 | Errors parity + non_exhaustive sweep + wire-format inventory + Compromise # | **LANDED** (commit `75a1d33a`) — 8a TS class count investigation outcome (delta is legitimate retained envelope); 8b/8c `#[non_exhaustive]` applied to `ManifestEnvelopeRecheckOutcome` + `WriteContext`; 8d wire-format inventory authored at `docs/V1-WIRE-FORMAT-INVENTORY.md` (10 surfaces; 9 covered + 1 deferred); 8e SECURITY-POSTURE Compromise #31 minted (revocation reach in encryption-at-rest — re-pointed to #62 at F-full per BR-2; #31 now denotes LAMPS Composite ML-DSA). |
| 9 | **MembershipSet members-table registry equality-pin** (F-full R6 R1 finding F-11; BELONGS-NAMED-NOW) | **NAMED for build-out.** The `members_table` canonical-CBOR snapshot (`benten_membership_set::aad::canonical_members_table_bytes`) is the AAD-bound keying minimum (NQ-W4) — two engines that serialize the SAME logical membership MUST produce byte-identical bytes (length-injective U3) or cross-engine AEAD-open diverges. Build-out item: an explicit **registry-equality regression-pin** asserting that two independently-constructed `MembersTable` snapshots with the same logical DID→`MemberEntry` content (insertion-order-independent via `BTreeMap`) serialize to equal canonical bytes AND derive an equal `audience_set_commitment`. Anchor: `crates/benten-membership-set/tests/f_aad_1_members_table_canonical_cbor_length_injective.rs` (extend with the explicit equality arm). Surfaced at R6 R1; lands at the next membership-set build-out sub-pass. |

**Wave outcome:** 7 of 8 G-CORE-9 rows fully LANDED; row 5 DEFERRED per Option A
(named destination); row 1 partial (baselines done; napi/test-site
cascade named for follow-up). The 8 build-backlog rows close at the
G-CORE-9 V1-FROZEN-INTERFACE build-out wave (commits
`dd12f394` → `5ce8bab6` → `a9d2753c` → `7af94d06` → `d2616800` →
`75a1d33a` → `fb7c212d` → `13322df4` on branch `g-core-9/build-out-wave`).

---

**End of build-backlog.**
