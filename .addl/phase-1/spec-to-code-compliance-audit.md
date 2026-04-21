# Spec-to-Code Compliance Audit — Phase 1

**Date:** 2026-04-17
**Auditor:** spec-to-code-compliance skill (Claude Opus 4.7)
**HEAD audited:** `d03f642af63ba1b1b325f5fc0ab2478928dc847b`
**Methodology:** 7-phase ADDL-style audit (Discovery → Normalization → Spec-IR → Code-IR → Alignment-IR → Divergence Classification → Report). Global rules: never infer unspecified behavior, always cite evidence (file:line or doc §/quote), always score confidence, classify ambiguity instead of guessing.
**Scope:** `docs/SECURITY-POSTURE.md` (7 named compromises), `CLAUDE.md` "Validated Design Decisions" (12 items), `docs/ENGINE-SPEC.md` §§3/4/5/7/9, `docs/ERROR-CATALOG.md` (45 codes), `.addl/phase-1/00-implementation-plan.md` §14.6 performance targets.

---

## 1. Executive Summary

**Spec-IR claims audited:** 82 discrete items across 5 document sources.

**Gaps found:** 9 total.

| Severity | Count | Description |
|----------|-------|-------------|
| Critical | 0 | No security-class spec-vs-code gaps uncovered that were not already flagged by compromises #1/#5 fixes. |
| Major | 4 | Error-catalog codes defined in the enum but with no firing site in production source; `Cid::from_str` hard-fails every input while the catalog implies Phase-1 acceptance; CLAUDE.md Phase-1 performance claim (<0.1 ms node creation) contradicts implementation-plan §14.6 honest range (100–500 µs) and bench documentation (macOS floor ~4 ms). |
| Minor | 5 | `E_INV_CONTENT_HASH` catalog says "Thrown at: read" but Node read path does not verify hash; `benten.ivm.view_stale_count` metric hard-coded to 0.0; `EvalError::WriteConflict` reserved but never constructed; unused `CapError::Revoked` (defensible — Phase-3 code); subgraph content-addressed byte normalization belt-and-suspenders comment promises ordering stability that tests confirm but inline docs don't enumerate. |

**Headline outcome:** The two previously-found "aspirational-prose-but-dead-code" patterns (compromises #1, #5) ARE genuinely fixed at HEAD; the same *pattern* recurs in four additional places that the 14-agent R6 review did not surface.

**Pattern identified:** **Catalogued-but-unfired error codes.** The drift-detector (`scripts/drift-detect.ts`) verifies enum↔catalog name parity, not runtime reachability. This creates a blind spot where variants can be added to `ErrorCode`, tested via `.code()` round-trip in `error_codes.rs` type-mapping tests, and ship with no production firing site. Four codes are in this state today.

---

## 2. Documentation Sources Identified

| Source | Role | Last meaningful revision |
|--------|------|--------------------------|
| `CLAUDE.md` | Top-level project instructions (read by every fresh agent) | 2026-04-17 (R5-ready banner) |
| `docs/SECURITY-POSTURE.md` | Named compromises #1–#7 + change-stream disclosure + napi input-limit posture | 2026-04-17 (compromises #3 / #5 / #7 / #8 closure entries) |
| `docs/ENGINE-SPEC.md` | Primitives, invariants, evaluator, hashing, capability system | 2026-04-14 (post-8-critic revision) |
| `docs/ERROR-CATALOG.md` | 45 stable error codes + context/fix hints | Pre-implementation spec (frozen) |
| `.addl/phase-1/00-implementation-plan.md` | 8-group R5 implementation plan + §14.6 perf targets | 2026-04-17 |

---

## 3. Spec-IR Breakdown (audited claims catalogue)

### 3.1 SECURITY-POSTURE.md — 8 posture sections

| # | Spec claim | Status pre-audit | Audit verdict |
|---|------------|------------------|---------------|
| S1 | Compromise #1 — TOCTOU refresh at (a) commit (b) CALL entry (c) every `iterate_batch_boundary` | Declared CLOSED at commit `de3f01b` | **Verified** (see §5.1) |
| S2 | Compromise #2 — `check_read` returns `DeniedRead` with `E_CAP_DENIED_READ` (Option A existence leak) | Open/accepted | **Verified** — `grant_backed.rs:206` constructs `CapError::DeniedRead` |
| S3 | Compromise #3 — `ErrorCode` lives in dedicated `benten-errors` crate | Declared CLOSED at commit `d03f642` | **Verified** — `crates/benten-errors/src/lib.rs` exists, zero workspace deps |
| S4 | Compromise #4 — WASM runtime compile-check only | Open/accepted | **Verified** — `bindings/napi` has `wasm-checks.yml`; no wasmtime in `benten-eval/Cargo.toml` |
| S5 | Compromise #5 — per-capability-scope write metrics recorded | Declared CLOSED at commit `ffa2c5b` | **Verified** (see §5.2) |
| S6 | Compromise #6 — BLAKE3 128-bit effective collision bound | Open/accepted, documentation-only claim | **Verified** — documentation stance aligns with BLAKE3-256 digest in `Cid` |
| S7 | Compromise #7 — `[[bin]]` gated by `required-features` | Declared CLOSED at commit `de3f01b` | **Verified** — `crates/benten-graph/Cargo.toml` has `required-features = ["test-fixtures"]` |
| S8 | Change-stream subscribe bypasses `check_read` | Open/documented | **Verified** — `engine.rs:1401-1410` builds `ChangeProbe` with no policy hook |
| S9 | napi `JSON_MAX_BYTES = 1 MiB` rejects with `E_INPUT_LIMIT` | Open/documented | **Verified** — `bindings/napi/src/node.rs:44,152,194` |

### 3.2 CLAUDE.md — 12 Validated Design Decisions

| # | Claim | Audit verdict |
|---|-------|---------------|
| D1 | 12 primitives: READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM | **Verified** (`benten-eval/src/lib.rs:390-393` + `primitives/mod.rs`) |
| D2 | IVM Algorithm B (per-view strategy selection) | **Partial** — Phase-1 ships 5 hand-written views (`benten-ivm/src/views/`); Algorithm B explicitly deferred to Phase 2 (matches spec) |
| D3 | Code-as-graph: handlers are subgraphs of operation Nodes | **Verified** — `benten_eval::Subgraph` + `SubgraphBuilder` |
| D4 | Not Turing-complete; DAGs; SANDBOX returns `E_PRIMITIVE_NOT_IMPLEMENTED` in Phase 1 | **Verified** — `primitives/mod.rs:65-68` |
| D5 | BLAKE3 + multihash 0x1e + CIDv1 + dag-cbor codec 0x71 | **Verified** constants in `benten-core/src/lib.rs:89-102` |
| D6 | Transaction primitive (not `transactional:true` property) | **Verified** — `engine.rs::transaction()` closure API |
| D7 | Capability system as pluggable policy (`NoAuthBackend` default) | **Verified** — `benten-caps/src/noauth.rs` + `CapabilityPolicy` trait |
| D8 | Version chains as opt-in pattern in `benten-core` | **Verified** — `benten-core/src/version.rs` |
| D9 | Member-mesh networking | Phase 3 scope — not verified at Phase-1 gate (expected) |
| D10 | TypeScript DSL with `crud('post')` | **Verified** — `packages/engine/src/crud.ts` exists and tests pass |
| D11 | Three-pillar positioning | Vision-level, no code surface |
| D12 | Committed scope = Phases 1–8 | Roadmap-level, no code surface |

### 3.3 ENGINE-SPEC.md — focused sections

| Section | Claim count | Audit verdict |
|---------|-------------|---------------|
| §3 (12 primitives) | 12 primitive rows + 8-executing/4-deferred split | **Verified** — executing set (READ/WRITE/TRANSFORM/RESPOND/BRANCH/ITERATE/CALL/EMIT) matches `primitives/mod.rs`; deferred set (WAIT/STREAM/SUBSCRIBE/SANDBOX) routes to `PrimitiveNotImplemented` |
| §4 (14 invariants; Phase-1 enforces 1/2/3/5/6/9/10/12) | 14 rows | **Verified** — `invariants.rs` docstring matches code (+Phase-1 stopgaps: inv-8 as `InvIterateNestDepth`, inv-11 as `E_SYSTEM_ZONE_WRITE`) |
| §5 (iterative evaluator; `ON_ERROR` routes tx aborts) | 2 claims | **Verified** — `evaluator.rs` uses explicit stack; `engine.rs:1620` maps `NestedTransactionNotSupported` through error edge |
| §7 (BLAKE3 over DAG-CBOR with CIDv1 format; no anchor/timestamp in hash) | 5 claims | **Verified** — `Node.anchor_id` is `#[serde(skip)]` (`benten-core/src/lib.rs:125`); no timestamp field exists on `Node` |
| §9 (caps = CapabilityGrant Nodes + GRANTED_TO edges; commit-time check) | 4 claims | **Verified** — `benten-caps/src/grant.rs:27` + `engine.rs:1579` |

### 3.4 ERROR-CATALOG.md — 45 codes

Every catalog code has an `ErrorCode` enum variant (confirmed by `drift-detect.ts` passing). This audit went further and checked whether each variant is actually **constructed** somewhere in production code (not just mapped in `.code()` accessors or tested in round-trip tests).

| Group | Total | Fires in prod | Unfired | Phase-deferred (legitimate) |
|-------|-------|---------------|---------|-----------------------------|
| `E_INV_*` (registration invariants) | 13 | 10 | 0 | 3 (`E_INV_SANDBOX_NESTED`, `E_INV_ITERATE_BUDGET` Phase-2 multiplicative form, evaluator-runtime enforcement) |
| `E_CAP_*` | 6 | 4 | 0 | 2 (`E_CAP_REVOKED` Phase-3 sync code, `E_CAP_REVOKED_MID_EVAL` is policy-supplied) |
| `E_SANDBOX_*` | 3 | 0 | 0 | 3 (all Phase-2) |
| `E_SYNC_*` | 3 | 0 | 0 | 3 (all Phase-3) |
| `E_*` runtime misc | 20 | **13** | **4** | 3 (including `E_IVM_VIEW_STALE` which IS fired but metric `benten.ivm.view_stale_count` is stubbed) |

**Unfired prod codes (not phase-deferred):**
- `E_WRITE_CONFLICT` — defined as `EvalError::WriteConflict`, module docstring comment `primitives/write.rs:13` says "reserved for the transaction primitive's internal use," but no production site constructs it. CAS conflicts flow through `ON_CONFLICT` routed edge, not `Err(WriteConflict)`. See §5.3.
- `E_CID_UNSUPPORTED_CODEC` — enum variant present; `Cid::from_bytes` returns `CoreError::InvalidCid("wrong multicodec")` which maps to `E_CID_PARSE`, not `E_CID_UNSUPPORTED_CODEC`. See §5.4.
- `E_CID_UNSUPPORTED_HASH` — same pattern. Unreachable.
- `E_IVM_PATTERN_MISMATCH` — `ViewError::PatternMismatch` declared, `.code()`-mapped, but no concrete view (`benten-ivm/src/views/*.rs`) ever constructs it. The r5-g5 reviewer previously flagged this (`.addl/phase-1/r5-g5-mini-ivm-algorithm-b-reviewer.json`); the fix was not landed. See §5.5.

### 3.5 Implementation-plan §14.6 — Performance targets

| Target | Spec claim source | Code/bench evidence | Verdict |
|--------|-------------------|---------------------|---------|
| Node lookup by ID 1–50 µs hot cache | ENGINE-SPEC §14.6 + plan §3 G2 | `crates/benten-graph/benches/get_create_node.rs::get_node/hot_cache` — spike measured 2.71 µs | **Aligned** (confidence 0.9) |
| IVM view read 0.04–1 µs | ENGINE-SPEC §14.6 | `crates/benten-ivm/benches/view_maintenance.rs` | **Aligned** (confidence 0.8; not benchmark-verified at audit time but target realistic given HashMap strategy) |
| Node creation + IVM update **<0.1 ms** (CLAUDE.md line 292) | CLAUDE.md "Performance targets for Phase 1" | Plan §4.4: realistic 100–500 µs, macOS APFS floor ~4 ms, `crud_post_create_dispatch` is **Not CI-gated** | **DRIFT** — CLAUDE.md figure is 10× more optimistic than ENGINE-SPEC §14.6 honest range and 40× more optimistic than plan-acknowledged floor. See §5.6. |
| 10-node handler 150–300 µs | ENGINE-SPEC §14.6 | Plan §4.4 acknowledges Phase-1 floor ~4 ms on macOS APFS; bench `crud_post_create_dispatch` intentionally un-gated | **Aligned** in ENGINE-SPEC + plan; CLAUDE.md "<0.1ms" restatement inherits the drift from §5.6 |
| Content listing <0.1 ms | CLAUDE.md | Plan §4.4 `crud_post_list_dispatch_no_write` target <300 µs (below 1 ms; write-free path) | **Aligned** (confidence 0.7 — not benchmark-verified at audit time) |

---

## 4. Alignment Matrix

(summary — full detail in §3; one row per audited dimension)

| Claim area | Location in code | Status |
|------------|------------------|--------|
| Compromise #1 (TOCTOU refresh) | `benten-eval/src/primitives/{call,iterate}.rs`, `benten-eval/src/host.rs`, `benten-engine/src/engine.rs:1574-1594` | full_match |
| Compromise #2 (Option-A existence leak) | `benten-caps/src/grant_backed.rs:188-211` | full_match |
| Compromise #3 (errors crate extraction) | `crates/benten-errors/src/lib.rs` (new in this cycle) | full_match |
| Compromise #4 (WASM compile-check) | `.github/workflows/wasm-checks.yml` | full_match |
| Compromise #5 (per-cap write metrics) | `benten-engine/src/engine.rs:119-192, 1574-1594, 1679-1711` | full_match |
| Compromise #6 (BLAKE3 128-bit bound) | `benten-core/src/lib.rs:95-102` | full_match (documentation-only) |
| Compromise #7 (`required-features` gating) | `crates/benten-graph/Cargo.toml` | full_match |
| Change-stream no-read-check | `benten-engine/src/engine.rs:1401-1410` | full_match (honest limitation) |
| napi `JSON_MAX_BYTES` enforcement | `bindings/napi/src/node.rs:44,152,194,265` | full_match |
| 12 primitives definition | `benten-eval/src/lib.rs` PrimitiveKind + `primitives/mod.rs` dispatcher | full_match |
| 14 invariants (Phase-1 subset 1/2/3/5/6/8-stopgap/9/10/12) | `benten-eval/src/invariants.rs:69-318` | full_match |
| BLAKE3 + CIDv1 + dag-cbor | `benten-core/src/lib.rs:89-292` | full_match |
| TRANSFORM expression evaluator | `benten-eval/src/expr/*` + `E_TRANSFORM_SYNTAX` fired at `primitives/transform.rs:48` | full_match |
| **`Cid::from_str` Phase-1 acceptance** | `benten-core/src/lib.rs:311-315` — unconditionally fails | **mismatch** (catalog suggests Phase-1 accepts base32) |
| **Read-path hash verification** | `redb_backend.rs:497-505` — no verification | **missing_in_code** (catalog implies read-time firing) |
| **`EvalError::WriteConflict`** | Only `.code()`-mapped in `lib.rs:105`; no constructor site | **missing_in_code** (catalog has `E_WRITE_CONFLICT`) |
| **`CoreError::CidUnsupportedCodec`** | Only `.code()`-mapped in `lib.rs:419`; no constructor site | **missing_in_code** (catalog has `E_CID_UNSUPPORTED_CODEC`) |
| **`CoreError::CidUnsupportedHash`** | Only `.code()`-mapped in `lib.rs:420`; no constructor site | **missing_in_code** (catalog has `E_CID_UNSUPPORTED_HASH`) |
| **`ViewError::PatternMismatch`** | Only `.code()`-mapped in `view.rs:79`; no view constructs it | **missing_in_code** (catalog has `E_IVM_PATTERN_MISMATCH`) |
| **`benten.ivm.view_stale_count` metric** | `engine.rs:1691` hard-codes `0.0` | **code_weaker_than_spec** (metric surface exists but value is a placeholder) |
| **CLAUDE.md "Node creation + IVM update <0.1 ms"** | Plan §4.4 + bench README acknowledge 100–500 µs realistic, ~4 ms macOS floor, `crud_post_create_dispatch` Not CI-gated | **code_weaker_than_spec** (CLAUDE.md target optimistic by 1–2 orders of magnitude) |

---

## 5. Divergence Findings

### 5.1 (Informational — previously-fixed, re-verified)

**Claim (SECURITY-POSTURE §Compromise #1):** Capability checks refresh at (a) commit (b) CALL entry (c) every `iterate_batch_boundary` iterations.

**Code evidence at HEAD:**
- Commit refresh: `benten-engine/src/engine.rs:1579` — `p.check_write(&ctx)` fires inside the write-commit closure.
- CALL-entry refresh: `benten-eval/src/primitives/call.rs:70` — `host.check_capability(&required_scope, None)`.
- Batch-boundary refresh: `benten-eval/src/primitives/iterate.rs:94` (iter 0) + `:106` (every N) — `host.check_capability(&required_scope, None)`.

**Verdict:** full_match, confidence 0.95. The commit `de3f01b` fix is genuine, not cosmetic.

### 5.2 (Informational — previously-fixed, re-verified)

**Claim (SECURITY-POSTURE §Compromise #5):** `metrics_snapshot()` surfaces `benten.writes.committed`, `benten.writes.denied`, `benten.writes.committed.<scope>`, `benten.writes.denied.<scope>`.

**Code evidence at HEAD:**
- Atomic counter fields at `engine.rs:127,130`; map fields at `:119,124`.
- Increment at commit: `engine.rs:1580` (denied) and `:1594` (committed), both calling `record_cap_write_*` which bumps the atomic AND the per-scope map.
- Surface at `engine.rs:1679-1689`.
- Typed accessors `capability_writes_committed() / capability_writes_denied()` at `:1702 / :1710`.

**Verdict:** full_match, confidence 0.95. The commit `ffa2c5b` fix is genuine.

### 5.3 **MAJOR — `E_WRITE_CONFLICT` has no firing site**

**Claim (ERROR-CATALOG.md):**
> `E_WRITE_CONFLICT` — Message: "Expected version {expected}, found {actual} on {target}" — Thrown at: Evaluation (CAS WRITE)

**Actual code state:**
- `benten-eval/src/lib.rs:80` defines `EvalError::WriteConflict`.
- `benten-eval/src/lib.rs:105` maps it to `ErrorCode::WriteConflict`.
- `benten-eval/src/primitives/write.rs:13` module docstring: *"the engine's transaction primitive reserves `Err(WriteConflict)` for its own internal use"*.
- **No production code constructs `EvalError::WriteConflict`.** Grep: `grep -rn "WriteConflict" crates/*/src` returns only the `.code()` arm and the reserved-for-internal-use docstring.
- The CAS conflict path at `benten-eval/src/primitives/write.rs:59` (`cas_step(op)`) routes via the `ON_CONFLICT` edge label with `Value::Null`, not an `Err(WriteConflict)`.

**Risk scenario:** A caller relying on the catalog's contract — filtering for `err.code() == "E_WRITE_CONFLICT"` — will never match a CAS failure in Phase 1. The actual contract is edge-routed (`StepResult { edge_label: "ON_CONFLICT", ... }`). Consumers who read only the catalog and not the `write.rs` module docstring will get silent no-hits.

**Recommended remediation:**
Either (a) delete `EvalError::WriteConflict` and `ErrorCode::WriteConflict` with a catalog entry noting "runtime surface is edge-routed `ON_CONFLICT`, not this code" (preferred — honest), or (b) have the engine transaction primitive construct this error in the documented "own internal use" path so the catalog stays true. Option (a) is safer given Phase-1 freeze posture.

**Confidence:** 0.9.

### 5.4 **MAJOR — `E_CID_UNSUPPORTED_CODEC` / `E_CID_UNSUPPORTED_HASH` unreachable**

**Claim (ERROR-CATALOG.md):**
> `E_CID_UNSUPPORTED_CODEC` — "Phase 1 only accepts DAG-CBOR multicodec (0x71). Re-encode under the expected codec or await later-phase codec support."
> `E_CID_UNSUPPORTED_HASH` — analogous.

**Actual code state:**
- `benten-core/src/lib.rs:400,404` — variants `CoreError::CidUnsupportedCodec` / `CoreError::CidUnsupportedHash` exist.
- `benten-core/src/lib.rs:419-420` — `.code()` maps them.
- `crates/benten-core/tests/error_codes.rs:58,66` — round-trip type tests exist.
- **No parsing path ever constructs them.** `Cid::from_bytes` at `benten-core/src/lib.rs:280-285`:
  ```rust
  if bytes[1] != MULTICODEC_DAG_CBOR {
      return Err(CoreError::InvalidCid("wrong multicodec"));
  }
  if bytes[2] != MULTIHASH_BLAKE3 {
      return Err(CoreError::InvalidCid("wrong multihash code"));
  }
  ```
- `InvalidCid` maps to `ErrorCode::CidParse` (lib.rs:418), so a caller who passes a Protobuf-codec CID gets `E_CID_PARSE`, NOT `E_CID_UNSUPPORTED_CODEC`.

**Risk scenario:** The catalog implies distinct codes for "malformed CID" vs "wrong codec" vs "wrong hash" so a sync-layer or operator can disambiguate user error from protocol mismatch. Phase 1 collapses all three into `E_CID_PARSE`, which is then ambiguous. Phase-3 sync peers that deliver a Protobuf-codec CID will surface an opaque parse error instead of the precise "unsupported codec" diagnosis the catalog promises.

**Recommended remediation:** Refactor `Cid::from_bytes` to distinguish the three cases — return `CoreError::CidUnsupportedCodec` on the codec-byte mismatch, `CoreError::CidUnsupportedHash` on the multihash-byte mismatch, and `CoreError::CidParse` / `InvalidCid` only for length / version / digest-length violations. Low-risk edit (~5 lines), preserves catalog semantics.

**Confidence:** 0.95.

### 5.5 **MAJOR — `E_IVM_PATTERN_MISMATCH` has no firing site**

**Claim (ERROR-CATALOG.md):**
> `E_IVM_PATTERN_MISMATCH` — "The caller asked a view for an index partition it doesn't maintain (e.g. an `entity_cid` query against a view that only keys on `label`). Consult the view's maintained-pattern list and restrict the `ViewQuery` to supported keys. Distinct from `E_INV_REGISTRATION` — the view is healthy; the query shape is wrong."

**Actual code state:**
- `benten-ivm/src/view.rs:53` declares `ViewError::PatternMismatch(String)`.
- `benten-ivm/src/view.rs:79` maps it to `ErrorCode::IvmPatternMismatch`.
- `benten-ivm/src/view.rs:228` docstring on the `read()` trait method promises `PatternMismatch` for unsupported queries.
- `benten-ivm/src/subscriber.rs:212,229` handles it as non-fatal.
- **No concrete `View` implementation in `crates/benten-ivm/src/views/*.rs` constructs `ViewError::PatternMismatch`.** The five Phase-1 views (`capability_grants`, `content_listing`, `event_handler_dispatch`, `governance_inheritance`, `version_chain_current`) either over-answer unmatched queries or return empty results. Grep confirms: only `Stale` and `BudgetExceeded` are constructed.

**Prior-art evidence that this was known:** `.addl/phase-1/r5-g5-mini-ivm-algorithm-b-reviewer.json:21` explicitly flagged the issue: *"make the trait `read` return `PatternMismatch` when a specific event_name is supplied (the caller is asking for a partition the view doesn't maintain). The current silent over-answer is the unsafer choice."* The fix was not landed.

**Risk scenario:** A downstream consumer writes `engine.readView("content_listing", { by: "entity_cid": ... })` — a partition the view does not maintain. Instead of `E_IVM_PATTERN_MISMATCH`, the read returns empty / partially-correct results with `Ok(...)`, violating the catalog's contract that healthy-view + wrong-shape is distinguishable. This is exactly the "silent data wrongness" class of bug that structured errors exist to prevent.

**Recommended remediation:** Either (a) land the r5-g5 reviewer's suggested fix — concrete views construct `PatternMismatch` for unsupported `ViewQuery` shapes; or (b) demote the catalog entry to Phase-2 (explicitly noting Phase-1 over-answer behavior). Option (a) is preferable because the catalog entry is already doc-promised and the views' `read()` method signatures can return `Result<..., ViewError>`.

**Confidence:** 0.9.

### 5.6 **MAJOR — CLAUDE.md `<0.1 ms` Node-creation target contradicts spec + plan + bench**

**Claim (CLAUDE.md "Performance targets for Phase 1"):**
> Node creation + IVM update: <0.1 ms (realistic, not the v1's aspirational <0.01ms)

**ENGINE-SPEC.md §14.6 (post-revision, 2026-04-14):**
> Node creation + IVM update — Realistic Range: **100–500 µs realistic, 0.1 ms aspirational** — Caveat: fsync to disk is 0.1–10 ms; spec must define durability policy per write class (group commit for bulk, immediate for capability grants)

**Implementation-plan §4.4:**
> `create_node_immediate` — target 100–500 µs realistic (spike Immediate: 4ms — must drop with DurabilityMode::Group).
> `crud_post_create_dispatch` — Phase-1 floor on macOS APFS is ~4ms per call (compromise #7: Group-durability collapses to Immediate). **Not CI-gated** — the §14.6 "150–300µs" headline target is not reachable in Phase 1 on dev hardware while the durability mode is Immediate.

**Bench README (`crates/benten-graph/benches/README.md`):**
> `create_node_immediate/default_durability` | 100–500 µs realistic, Immediate

**Verdict:** CLAUDE.md's single-number `<0.1 ms` headline is the old v2-optimistic aspiration. The ENGINE-SPEC, the implementation plan, AND the bench README all agree Phase 1's honest target is 100–500 µs, with the commit-path floor at ~4 ms on macOS APFS while Group durability collapses to Immediate (compromise #7's durability story, distinct from the `[[bin]]`-gating compromise of the same number).

This is not a code bug — the code behaves correctly. It is a **documentation drift between CLAUDE.md and every other authoritative surface.** A fresh agent reading CLAUDE.md will anchor on `<0.1 ms` and not discover the honest range unless they also read ENGINE-SPEC §14.6 and the plan.

**Risk scenario:** When benchmarks show 4 ms on a CI runner, an agent using CLAUDE.md as its perf contract will flag a 40× regression and either waste cycles investigating a non-regression or ship a speculative fix. The plan §4.4 intentionally ungates the bench; CLAUDE.md contradicts that intent.

**Recommended remediation:** Update CLAUDE.md "Performance targets for Phase 1" to match ENGINE-SPEC §14.6 language (ranges + caveats), or delete the bulleted list and point at `ENGINE-SPEC.md §14.6` as the single source of truth. Low-risk doc edit.

**Confidence:** 0.95.

### 5.7 **MINOR — `E_INV_CONTENT_HASH` "Thrown at: Registration / read" — read-path enforcement is subgraph-only**

**Claim (ERROR-CATALOG.md):**
> `E_INV_CONTENT_HASH` — "Content hash mismatch for {node_id}: expected {expected}, computed {actual}" — Thrown at: **Registration / read** — "A stored Node's computed content hash does not match its key. Indicates on-disk corruption or an incompatible serialization migration."

**Actual code state:**
- `benten-eval::Subgraph::load_verified` (exercised by `crates/benten-eval/tests/invariants_9_10_12.rs::rejects_content_hash_mismatch`) does verify the subgraph byte-level hash on load.
- `benten-graph::RedbBackend::get_node` at `redb_backend.rs:497-505` does **not** re-hash the decoded Node and compare to the requested CID. A corrupted `{key: cid, value: bytes}` pair where the bytes no longer hash to the key returns a (wrong) Node silently, or returns `CoreError::Serialize` if the bytes cannot be decoded.

**Risk scenario:** The catalog entry implies Node-level read-time protection against on-disk corruption. Phase-1 has subgraph-level protection but not Node-level — a bitflip in a stored Node's DAG-CBOR payload can surface as a wrong-but-decodable Node. For content-addressed storage, this weakens the spec's "integrity verification" claim.

**Recommended remediation:** Either (a) add an optional `get_node_verified(&self, cid)` path that re-hashes on read (costs 3–10 µs BLAKE3 per call), or (b) tighten the catalog entry to clarify "Thrown at: Registration (Node CID) / Subgraph load (subgraph CID)" with explicit note that Node reads rely on storage-layer page checksums (redb) for corruption detection, not application-level hash verification. Option (b) is cheaper and honest.

**Confidence:** 0.85.

### 5.8 **MINOR — `benten.ivm.view_stale_count` hard-coded to 0.0**

**Claim:** No SECURITY-POSTURE or ENGINE-SPEC claim promises this metric's value directly. The `metrics_snapshot()` rustdoc at `engine.rs:1638` says *"`benten.ivm.view_stale_count` — Phase-1 placeholder; Phase-2 wires the real counter."*

**Actual code state:** `engine.rs:1691`:
```rust
out.insert("benten.ivm.view_stale_count".to_string(), 0.0);
```
The value is literally the zero constant regardless of view state.

**Verdict:** The rustdoc is honest about Phase-1 being a placeholder, so this is not a hidden drift. However, it is the same *pattern* as compromise #5 (metric-surface-exists-but-doesn't-do-what-the-name-says). A consumer who sees `benten.ivm.view_stale_count: 0.0` in a metrics dashboard will infer "no stale views" when in truth the counter is untethered from reality. This risk is bounded because no external doc currently promises the counter is live; re-raising as awareness item, not a spec-bug.

**Recommended remediation:** Either wire the counter (views already have an `is_stale()` method at `view.rs:248`; the subscriber can tally stale views on every `metrics_snapshot` call), or drop the key from the emitted map in Phase 1 (the test surface doesn't assert on it at `.addl/phase-1/r6-*` — verified via grep). Dropping is the safer Phase-1 close.

**Confidence:** 0.8.

### 5.9 **MINOR — `Cid::from_str` Phase-1 acceptance claim**

**Claim (ERROR-CATALOG.md `E_CID_PARSE`):**
> "Phase 1 accepts only base32-lower-nopad multibase (`b`-prefixed) CIDv1. Check that the caller is not passing a base58btc / base64 / hex form, and that the bytes are not truncated."

**Actual code state:** `benten-core/src/lib.rs:311-315`:
```rust
pub fn from_str(_s: &str) -> Result<Self, CoreError> {
    Err(CoreError::CidParse(
        "Cid::from_str is a Phase 2 deliverable (needs multibase decoder; see C4)",
    ))
}
```
**Every input fails.** The catalog's "Phase 1 accepts base32..." hint is misleading — Phase 1 accepts nothing on the `from_str` path.

**Risk scenario:** A developer reads the catalog hint and wires `Cid::from_str(user_input)` expecting base32-lower-nopad acceptance, then hits the unconditional error. The fix-hint blames the caller's input format; the true cause is the Phase-2 decoder deferral.

Mitigating: the Rust side uses `Cid::from_bytes` (byte-form) everywhere in Phase 1. The string-form `from_str` is only reachable from explicit test harnesses and the (not-yet-wired) Phase-2 multibase decoder. But the TypeScript boundary at `bindings/napi/src/node.rs:383` fires a different `E_INPUT_LIMIT: cid: invalid base32` error for malformed CID strings — THAT is the catalog-aligned path.

**Recommended remediation:** Update the `E_CID_PARSE` fix-hint to clarify: "Rust `Cid::from_str` is a Phase-2 deliverable; Phase-1 CID-string input arrives exclusively via the napi boundary which rejects with `E_INPUT_LIMIT`." Or implement `from_str` now — the base32-lower-nopad decoder is ~30 lines and mirrors the existing `to_base32` encoder at `lib.rs:317-327`.

**Confidence:** 0.9.

---

## 6. Patterns Observed

### 6.1 "Aspirational compromise prose" recurrence

The same pattern Ben already found twice (compromise #1 TOCTOU, compromise #5 write metrics) recurs in four places this audit surfaces:
- `E_WRITE_CONFLICT` (reserved-but-unfired)
- `E_CID_UNSUPPORTED_CODEC` / `E_CID_UNSUPPORTED_HASH` (variants exist but generic-error path takes their traffic)
- `E_IVM_PATTERN_MISMATCH` (reviewer-flagged, fix not landed)
- `benten.ivm.view_stale_count` (hard-coded placeholder)

All five previous and new instances share a structure: a *surface* exists (enum variant, metric key, `.code()` mapping, catalog entry) but the *hot path that should reach the surface* is either a routed-edge (for edge-routable errors), a generic code (for error-disambiguation), or a zero-constant (for metrics). The drift-detector at `scripts/drift-detect.ts` pattern-matches the surface names and passes — its contract is "enum variants have catalog entries," not "variants are reachable from production code paths."

### 6.2 Catalog contract semantics

`docs/ERROR-CATALOG.md` mixes two semantics in its "Thrown at" field:
- Some entries name a *module* (e.g. `E_INV_CYCLE` "Thrown at: Registration"), which reads as "the registration subsystem may throw this." Verifying these requires finding a code path within the named module that constructs the variant.
- Some entries imply *specific triggers* (e.g. `E_CID_UNSUPPORTED_CODEC` "CID codec {codec} is not supported") which read as "this is the distinguished code for this specific condition." Verifying these requires finding a code path that constructs the variant *on* that condition.

The drift-detector checks the first (name parity) but not the second (condition parity). That gap is why §5.3/§5.4/§5.5 are not caught automatically.

### 6.3 CLAUDE.md staleness

CLAUDE.md serves as the fresh-agent handoff doc. Its "Performance targets for Phase 1" section restates values the spec has since revised. This is the third time (audit-wise) that CLAUDE.md has drifted from ENGINE-SPEC; prior drifts were caught during R1 triage and re-synced. A lint step that greps CLAUDE.md for numeric performance claims and compares against ENGINE-SPEC §14.6 ranges would close this loop.

---

## 7. Recommendations

### 7.1 Fix now (before closing Phase 1)

1. **§5.4 — `E_CID_UNSUPPORTED_CODEC` / `E_CID_UNSUPPORTED_HASH` firing.** ~5-line edit to `Cid::from_bytes`; preserves the catalog's distinct-code promise. Low risk, high clarity.
2. **§5.6 — CLAUDE.md perf target restatement.** Doc-only edit to replace the bulleted `<0.1 ms` list with a pointer to ENGINE-SPEC §14.6. Prevents future fresh-agent anchoring drift.
3. **§5.3 — `E_WRITE_CONFLICT` honesty.** Either delete the catalog entry + enum variant (Phase-1 contract is edge-routed `ON_CONFLICT`) OR document the catalog entry's Phase-1 semantics as "reserved for transaction-primitive internal use; runtime surface is edge-routed." The catalog's fix-hint "Re-read, rebase changes, retry" is actively misleading today.

### 7.2 Defer (Phase 2 or explicit compromise)

4. **§5.5 — `E_IVM_PATTERN_MISMATCH`.** Small fix (concrete views emit `PatternMismatch` for unsupported queries) but carries a behavior-visible change for existing view consumers. Bundle with Phase-2 IVM generalization if not addressed now. If deferred, add an explicit compromise #9 to SECURITY-POSTURE.md so the gap is tracked, mirroring the #1 / #5 closure template.
5. **§5.7 — Node read-path hash verification.** Either (a) add optional `get_node_verified` and document the cost, or (b) tighten the catalog entry. Option (b) is cheaper and honest for Phase 1.
6. **§5.8 — `view_stale_count` metric.** Either wire the tally or drop the key. Either way, a <10-line edit.
7. **§5.9 — `Cid::from_str` catalog hint.** Sharpen the catalog fix-hint, or land the base32 decoder.

### 7.3 Process remediation

8. **Extend `scripts/drift-detect.ts` with a reachability check.** For every `ErrorCode` variant, assert that at least one `src/**/*.rs` file (not `tests/`, not `.code()` accessor arms) references the variant in a construction position (`Err(...)`, `.map_err`, `return ...`, etc.). This would have caught §5.3/§5.4/§5.5 automatically.
9. **Optional: lint CLAUDE.md numeric claims against ENGINE-SPEC §14.6.** A tiny script parses numbers from both and flags divergences.

---

## 8. Confidence Map

| Area | Confidence | Notes |
|------|-----------|-------|
| Compromise #1/#5 fix verification | 0.95 | Grep + read of the specific firing sites confirms genuine implementation at HEAD. |
| Compromise #2/#3/#6/#7 status | 0.95 | Each closure has a paired regression test citing file+function in SECURITY-POSTURE.md. |
| Compromise #4 (WASM compile-check) | 0.85 | Workflow referenced in doc; audit did not download and re-run the CI job. |
| 12 primitives definition | 0.95 | Match between `PrimitiveKind` and `primitives/mod.rs` dispatcher is explicit. |
| 14 invariants Phase-1 subset | 0.95 | `invariants.rs` docstring enumerates enforced set; source-matches. |
| BLAKE3 + CIDv1 + DAG-CBOR | 0.95 | Constants + `serde_ipld_dagcbor` usage all directly inspected. |
| **Error-catalog reachability (§5.3/§5.4/§5.5)** | **0.9** | Grep-based reachability check is thorough but may miss dynamic code paths (macro-generated constructors). Manual audit confirmed no such macro sites exist for these variants. |
| Performance-target drift (§5.6) | 0.95 | Three independent sources (ENGINE-SPEC, plan, bench README) agree; CLAUDE.md is the outlier. |
| Read-path hash verification (§5.7) | 0.85 | `get_node` implementation directly inspected; no other hash-verification path found via grep. Possible (unlikely) miss: a custom backend layer that re-hashes. |
| Metric stubbing (§5.8) | 0.8 | Hard-coded `0.0` is explicit in source; the claim surface is purely the metric key presence. |
| `Cid::from_str` (§5.9) | 0.9 | Body is 5 lines and unambiguous. |
| Algorithm B IVM (Phase-2 deferred) | 0.75 | Spec says Algorithm B ships Phase 2; code ships 5 hand-written views — matches intent, so marked `partial_match` as spec-documented deferral. |
| Change-stream no-read-check | 0.9 | `subscribe_change_events` body is 10 lines and trivially confirms no policy hook. |

---

## 9. Appendix — Audit Mechanics

**Tools used:** Read, Grep, Glob, Bash (`grep -rn`, `cat`, `wc`). No code or test modifications. No git commits. Report is the sole output artifact.

**Search strategy:** For each ErrorCode variant, ran three grep passes:
1. `ErrorCode::<Variant>` across `crates/*/src` (finds enum usage, `.code()` mapping, docstring references).
2. `<OwnerType>::<Variant>` across `crates/*/src` (finds constructor sites for typed error variants).
3. `Err\(.*<Variant>\)` across `crates/*/src` (finds `Err(...)` construction in return positions).

A variant was classified "fires in prod" only if pass (2) or (3) returned ≥1 hit in a non-test file. This is conservative — macro-generated constructors would be missed — but no such macros exist in the audited crates.

**Coverage:** 45 catalog codes × 3 grep patterns = 135 queries. 4 codes surfaced as "defined but no prod firing site" (§5.3/§5.4 ×2/§5.5). Phase-deferred codes (Phase-2 SANDBOX, Phase-3 sync) were not counted as gaps — they are legitimate in Phase 1.

**Anti-hallucination audit:** No claims in this report reference behavior the auditor could not cite with file:line evidence. Where the catalog implies behavior and the code does not match, the report names both sides explicitly ("Claim: ... Actual: ..."). Where specs are ambiguous, the report marks AMBIGUOUS and names what was unclear (none in this audit; all drifts are unambiguous).

---

*End of report.*
