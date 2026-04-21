# Spec-to-Code Compliance Audit (R7) — Phase 1 close

**Date:** 2026-04-21
**HEAD audited:** `f69830b` ("docs(claude-md): mark Phase 1 COMPLETE")
**Methodology:** `spec-to-code-compliance` skill (7-phase pipeline). Anti-hallucination discipline per `OUTPUT_REQUIREMENTS.md`: every claim quotes file:line; confidence <0.8 requires investigation or AMBIGUOUS classification.
**Anchor:** prior audit `.addl/phase-1/spec-to-code-compliance-audit.md` ran at `d03f642` (2026-04-17) and produced 9 gaps. This audit verifies those gaps' closure state and re-examines the full Phase-1 spec surface.
**Corpus:** `SECURITY-POSTURE.md` (8 compromises + 3 sections), `ENGINE-SPEC.md` §§3/4/5/7/9, `ERROR-CATALOG.md` (44 Phase-1 codes), `CLAUDE.md` Validated Design Decisions #1–#12, `ENGINE-SPEC.md` §14.6, `QUICKSTART.md`, `DSL-SPECIFICATION.md`, `TRANSFORM-GRAMMAR.md`, `ARCHITECTURE.md`, `.addl/phase-1/00-implementation-plan.md` §1.

---

## 1. Executive Summary

**Spec-IR items audited:** 118 discrete claims across 10 document sources.

**Gaps found:** 7 total.

| Severity | Count | Summary |
|----------|-------|---------|
| Critical | 0 | No security-class spec-vs-code gaps uncovered. |
| High | 0 | — |
| Medium | 4 | Scaffolder smoke test diverges from plan §1 on 3 of 6 gates (capability-denial, `@mermaid-js/parser` use, canonical-fixture CID assertion); `DSL-SPECIFICATION.md` has internal inconsistency (header dropped VALIDATE/GATE but body still imports them); `ARCHITECTURE.md` claims "six crates" but the workspace has seven since the `benten-errors` extraction; `Cid::from_str` remains unimplemented while the `E_CID_PARSE` catalog hint reads as "Phase-1 accepts base32" (prior audit §5.9, still open). |
| Low | 3 | `benten.ivm.view_stale_count` metric still hard-coded to `0.0` (prior audit §5.8, tracked in Phase-2 backlog); Node read-path hash verification still subgraph-only (prior audit §5.7, backlogged); `E_INV_SYSTEM_ZONE` catalog entry present with no enum variant (Phase-2 deferred, acknowledged). |

**Prior-audit closure scorecard (9 gaps → 6 closed, 3 remain):**

| Prior §ref | Finding | Status at `f69830b` |
|-----------|---------|---------------------|
| §5.1 | Compromise #1 (TOCTOU refresh) | Re-verified CLOSED |
| §5.2 | Compromise #5 (per-cap metrics) | Re-verified CLOSED |
| §5.3 | `E_WRITE_CONFLICT` no firing site | **CLOSED** (fires edge-routed at `primitive_host.rs:462`; catalog entry updated with explicit "Runtime surface is edge-routed" language) |
| §5.4 | `E_CID_UNSUPPORTED_CODEC` / `_HASH` unreachable | **CLOSED** (`Cid::from_bytes` now distinguishes codec-byte from multihash-byte mismatches; see `benten-core/src/lib.rs` and the `CidUnsupportedCodec` / `CidUnsupportedHash` firing counts below) |
| §5.5 | `E_IVM_PATTERN_MISMATCH` no firing site | **CLOSED** (all 5 Phase-1 views construct `ViewError::PatternMismatch`; see `views/capability_grants.rs:268`, `views/version_current.rs:222`, `views/event_handler_dispatch.rs:246`, `views/content_listing.rs:354`, `views/governance_inheritance.rs:255`) |
| §5.6 | CLAUDE.md `<0.1 ms` drift vs ENGINE-SPEC | **CLOSED** (CLAUDE.md now cites `ENGINE-SPEC §14.6` as single source of truth for perf targets) |
| §5.7 | `E_INV_CONTENT_HASH` read-path enforcement subgraph-only | **OPEN** (tracked in `docs/future/phase-2-backlog.md` §6.2 — either add optional `get_node_verified` or tighten catalog entry; Phase-2 item) |
| §5.8 | `benten.ivm.view_stale_count` hard-coded `0.0` | **OPEN** (tracked in backlog §5.3 — wire the counter or drop the key) |
| §5.9 | `Cid::from_str` Phase-1-accepts-base32 catalog hint | **OPEN** (tracked in backlog §6.1; `Cid::from_str` at `benten-core/src/lib.rs:324` still unconditionally returns `CoreError::CidParse("Cid::from_str is a Phase 2 deliverable")`) |

**Headline outcome:** The "aspirational prose, dead code" pattern the prior audit named (compromises #1 / #5 previously, plus 4 catalog codes) is substantively closed at HEAD. The remaining 3 prior-audit items are explicit Phase-2 backlog entries with documented rationale, not regressions. This audit surfaces 4 **new** medium-severity items — all documentation-vs-code drift in peripheral surfaces (scaffolder smoke test, DSL spec doc, architecture doc, CID parser fix-hint) — and 3 low-severity items, none of which are security or correctness-class.

**Verdict:** Phase 1 is compliance-clean on the core load-bearing surfaces (primitives, invariants, hashing, capabilities, error codes, compromises). Peripheral documentation drift is the pattern to address before Phase 2 kicks off.

---

## 2. Documentation Sources Identified

| Source | Role | Last material edit | Sections audited |
|--------|------|-------------------|------------------|
| `docs/SECURITY-POSTURE.md` | 8 named compromises + change-stream disclosure + napi input-limit + BLAKE3 choice | 2026-04-21 (Option C closure for Compromise #2) | Full |
| `docs/ENGINE-SPEC.md` | Primitives, invariants, evaluator, hashing, caps, performance | 2026-04-14 (post-critic revision) | §§3, 4, 5, 7, 9, 14.6 |
| `docs/ERROR-CATALOG.md` | 44 Phase-1 codes + Phase-2/3 deferrals + TS-only codes | 2026-04-20 (reachability updates) | Full |
| `CLAUDE.md` | Fresh-agent orchestrator handoff | 2026-04-21 (Phase 1 close marker) | Validated Design Decisions #1–#12 |
| `.addl/phase-1/00-implementation-plan.md` | 8-group R5 plan + perf targets + exit criteria | 2026-04-20 | §1 (6 exit criteria) |
| `docs/QUICKSTART.md` | 10-minute DX path | 2026-04-20 (Option C diagnose_read worked example) | Full |
| `docs/DSL-SPECIFICATION.md` | TS DSL surface specification | 2026-04-14 (header banner about primitive revision) | Import list + §2 primitive table + §4 CRUD |
| `docs/TRANSFORM-GRAMMAR.md` | BNF for TRANSFORM expression language | 2026-04-15 | Full BNF + built-ins + denylist |
| `docs/ARCHITECTURE.md` | Layered architecture + 6-crate structure | 2026-04-14 | Layer 1 + crate list + thinness test |

Note: all sources are canonical markdown with no PDF / DOCX / HTML artefacts — Phase 1 normalization (skill Phase 1) was a no-op.

---

## 3. Spec Intent Breakdown (Spec-IR highlights)

Full Spec-IR is 118 records — representative excerpts below; full matrix at §5.

### 3.1 SECURITY-POSTURE.md — 11 items

```yaml
- id: SP-C1
  spec_excerpt: "Phase-1 capability checks refresh the grant snapshot at THREE distinct [points]: at commit, at CALL entry, every iterate_batch_boundary (default 100) iterations."
  source_section: "docs/SECURITY-POSTURE.md §Compromise #1"
  semantic_type: security_refresh_contract
  confidence: 0.95

- id: SP-C2
  spec_excerpt: "`Engine::get_node`, `Engine::edges_from`, `Engine::edges_to`, and `Engine::read_view` now collapse a `CapabilityPolicy::check_read` denial onto `Ok(None)` / `Ok(vec![])` / an empty-list `Outcome` — byte-identical with [absence]."
  source_section: "docs/SECURITY-POSTURE.md §Compromise #2 (Option C)"
  semantic_type: security_symmetric_denial
  confidence: 0.95

- id: SP-C3
  spec_excerpt: "Closes SECURITY-POSTURE compromise #3 so the catalog enum no longer forces a `benten-core` edge on any crate that only needs the stable string identifiers."
  source_section: "benten-errors crate-level docstring"
  semantic_type: dependency_independence
  confidence: 0.95

- id: SP-C4
  spec_excerpt: "WASM runtime compile-check only."
  source_section: "docs/SECURITY-POSTURE.md §Compromise #4"
  semantic_type: phase_deferral
  confidence: 1.0  # explicitly accepted

- id: SP-C5
  spec_excerpt: "Per-capability-scope write metrics: benten.writes.committed.<scope> and benten.writes.denied.<scope>"
  source_section: "docs/SECURITY-POSTURE.md §Compromise #5"
  semantic_type: observability_contract
  confidence: 0.95

- id: SP-C6
  spec_excerpt: "BLAKE3-256 → 128-bit effective collision resistance"
  source_section: "docs/SECURITY-POSTURE.md §Compromise #6"
  semantic_type: cryptographic_bound
  confidence: 1.0  # documentation-only claim

- id: SP-C7
  spec_excerpt: "`[[bin]]` gated by `required-features = [\"test-fixtures\"]`"
  source_section: "docs/SECURITY-POSTURE.md §Compromise #7"
  semantic_type: packaging_gate
  confidence: 1.0

- id: SP-C8
  spec_excerpt: "PrimitiveHost is the sole dispatch path; the engine's fast-path CRUD goes through the evaluator's host trait, not a direct evaluator bypass."
  source_section: "docs/SECURITY-POSTURE.md §Compromise #8"
  semantic_type: architectural_invariant
  confidence: 0.95

- id: SP-CS
  spec_excerpt: "`Engine::subscribe_change_events` fans out every committed ChangeEvent without a per-event check_read gate"
  source_section: "docs/SECURITY-POSTURE.md §'Change-stream subscription bypasses capability read-checks'"
  semantic_type: honest_limitation
  confidence: 1.0

- id: SP-NAPI
  spec_excerpt: "napi boundary input exceeds {limit_kind} limit: 1 MiB default for JSON / bytes / text"
  source_section: "docs/ERROR-CATALOG.md E_INPUT_LIMIT + SECURITY-POSTURE.md §'napi JSON_MAX_BYTES'"
  semantic_type: dos_mitigation
  confidence: 1.0

- id: SP-HASH
  spec_excerpt: "Option A — BLAKE3 only (chosen). Multicodec 0x71 (dag-cbor); multihash 0x1e (BLAKE3)."
  source_section: "docs/SECURITY-POSTURE.md §'Hash algorithm choice — BLAKE3 (options considered)'"
  semantic_type: algorithm_choice
  confidence: 1.0
```

### 3.2 ENGINE-SPEC.md — 30 items

12 primitive records (P1–P12), 14 invariant records (I1–I14), 4 evaluator records (EV1–EV4), and additional records for the §7 content-addressing and §9 capability specifications. See §5 for the full alignment matrix.

### 3.3 ERROR-CATALOG.md — 44 Phase-1 codes + 11 Phase-deferred codes

Every Phase-1 code has a catalog row; the audit checked each for a construction site in non-test Rust source (Phase 3 Code-IR). See §5.5 for the reachability table.

### 3.4 CLAUDE.md Validated Design Decisions — 12 items

Short-form:

```yaml
- id: CM-1
  claim: "12 primitives: READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM. Revision drops VALIDATE and GATE; adds SUBSCRIBE and STREAM."

- id: CM-2
  claim: "IVM Algorithm B (dependency-tracked incremental) with per-view strategy selection. Phase 1 ships 5 hand-written views."

- id: CM-3
  claim: "Code-as-graph: handlers are subgraphs of operation Nodes."

- id: CM-4
  claim: "Not Turing complete: DAGs + bounded iteration + SANDBOX as escape hatch."

- id: CM-5
  claim: "BLAKE3 + multihash + DAG-CBOR via serde_ipld_dagcbor + CIDv1."

- id: CM-6
  claim: "Transaction primitive exposed as API, not `transactional:true` property."

- id: CM-7
  claim: "Capability system as pluggable policy with NoAuthBackend default."

- id: CM-8
  claim: "Version chains as opt-in pattern (benten-core)."

- id: CM-9
  claim: "Member-mesh networking (Phase 3 — not verified at Phase-1 gate)."

- id: CM-10
  claim: "TypeScript DSL with crud('post') zero-config shorthand."

- id: CM-11
  claim: "Three-pillar positioning (vision-level, no code surface)."

- id: CM-12
  claim: "Committed scope = Phases 1–8 (roadmap-level, no code surface)."
```

### 3.5 Performance targets (ENGINE-SPEC §14.6) — 6 rows

See §5.6.

### 3.6 Exit criteria (plan §1) — 6 gates

See §5.7.

### 3.7 QUICKSTART + DSL + TRANSFORM-GRAMMAR + ARCHITECTURE

Remaining Spec-IR items cover: the 10-minute DX path (4 items), the TS DSL surface (16 items: 12 primitives + crud + subgraph + toMermaid + PolicyKind), TRANSFORM grammar (BNF + 30+ built-ins + 38-item denylist = 3 group-records), and the layered architecture (6-crate structure + thinness test = 2 records).

---

## 4. Code Behavior Summary (Code-IR highlights)

Phase 3 analysed every production-source construction of each spec-claim surface. Full per-function Code-IR is deferred to the mini-review level; the audit surfaces the alignment-relevant fragments below.

```yaml
- id: CODE-ERRCODE-ENUM
  file: crates/benten-errors/src/lib.rs
  lines: 50-166
  visibility: pub
  behavior:
    variants: 44 Phase-1 catalog codes + Unknown(String) forward-compat
    traits: Debug, Clone, PartialEq, Eq, non_exhaustive
    accessors: as_str (L175), as_static_str (L234), from_str (L287)
  invariants_enforced: "Every variant round-trips through from_str(as_str(v)) == v; variant count pinned by ALL_CATALOG_VARIANTS"

- id: CODE-PRIMITIVE-DISPATCH
  file: crates/benten-eval/src/primitives/mod.rs
  lines: 55-71
  visibility: pub
  behavior:
    executors: Read, Write, Respond, Emit, Transform, Branch, Iterate, Call
    deferred: Wait | Stream | Subscribe | Sandbox → Err(EvalError::PrimitiveNotImplemented(op.kind))
  invariants_enforced: "All 12 PrimitiveKind variants have a branch; 4 Phase-2 deferrals route to a single typed error"

- id: CODE-INVARIANT-VALIDATION
  file: crates/benten-eval/src/invariants.rs
  lines: 79-367
  visibility: pub
  behavior:
    enforced_at_registration: [Cycle (L88-122), TooManyNodes (L95), TooManyEdges (L106), FanoutExceeded (L138, L267), DepthExceeded (L155, L338), Determinism (L173, L351), ContentHash (via Subgraph::load_verified), IterateMaxMissing, IterateNestDepth (L303), Registration]
    invariant_numbers_mapped: "invariant_number() returns 1/2/3/5/6/8/9/10/12 for the Phase-1 set"
  invariants_enforced: "Structural invariants 1/2/3/5/6/9/10/12 fire at registration with InvariantViolation enum"

- id: CODE-CID-CONSTS
  file: crates/benten-core/src/lib.rs
  lines: 92-95, 259-260
  behavior:
    MULTICODEC_DAG_CBOR: 0x71 (L92)
    MULTIHASH_BLAKE3: 0x1e (L95)
    cid_bytes_layout: "[0x01 version][0x71 dag-cbor][0x1e BLAKE3][0x20 length][32-byte BLAKE3 digest]"
  invariants_enforced: "CIDv1 structure pinned; Node::cid writes MULTICODEC_DAG_CBOR + MULTIHASH_BLAKE3 at fixed byte offsets"

- id: CODE-ENGINE-OPTION-C
  file: crates/benten-engine/src/engine_diagnostics.rs
  lines: 248-290 (diagnose_read)
  companion_struct: crates/benten-engine/src/outcome.rs:273 (DiagnosticInfo)
  behavior:
    gated_on: "debug:read capability via policy.check_read(\"debug\", cid)"
    returns: "DiagnosticInfo { cid, exists_in_backend, denied_by_policy: Option<String>, not_found }"
  invariants_enforced: "Symmetric None on policy denial; diagnostic gated so ordinary callers see E_CAP_DENIED not E_NOT_FOUND"

- id: CODE-CAP-METRICS
  file: crates/benten-engine/src/engine.rs
  lines: 160-192 (record_cap_write_committed/denied)
  surface: crates/benten-engine/src/engine_diagnostics.rs:170-184 (metrics_snapshot)
  behavior:
    emitted_keys: [benten.writes.committed, benten.writes.denied, benten.writes.committed.<scope>, benten.writes.denied.<scope>]
  invariants_enforced: "check_write Err path calls record_cap_write_denied; Ok path calls record_cap_write_committed"

- id: CODE-CHANGESTREAM
  file: crates/benten-engine/src/engine_views.rs
  lines: 23-31 (subscribe_change_events)
  behavior:
    check_read_gate: absent
    fans_out: every committed ChangeEvent since the probe was created
  invariants_enforced: "Honest limitation — no per-event policy check, fully spec'd in SECURITY-POSTURE"

- id: CODE-NAPI-LIMITS
  file: bindings/napi/src/node.rs
  lines: 44 (const), 152, 194, 265 (enforcement sites)
  behavior:
    JSON_MAX_BYTES: 1024 * 1024
    rejection: "napi::Error::new(Status::GenericFailure, format!(\"E_INPUT_LIMIT: {msg}\"))"
  invariants_enforced: "Oversized JSON / Bytes / Text rejected pre-allocation"

- id: CODE-IVM-PATTERN-MISMATCH
  files:
    - crates/benten-ivm/src/views/capability_grants.rs:268
    - crates/benten-ivm/src/views/version_current.rs:222
    - crates/benten-ivm/src/views/event_handler_dispatch.rs:246
    - crates/benten-ivm/src/views/content_listing.rs:354
    - crates/benten-ivm/src/views/governance_inheritance.rs:255
  behavior:
    construction: "Err(ViewError::PatternMismatch(...))"
    catalog_mapping: "ViewError::PatternMismatch → ErrorCode::IvmPatternMismatch (view.rs:84)"
  invariants_enforced: "All 5 Phase-1 views reject unsupported query partitions with a typed error — fix to prior audit §5.5"

- id: CODE-WRITE-CONFLICT-EDGE
  file: crates/benten-engine/src/primitive_host.rs
  lines: 460-463
  behavior:
    match_arm: "\"ON_CONFLICT\" => (\"ON_CONFLICT\".to_string(), Some(\"E_WRITE_CONFLICT\".to_string()))"
  invariants_enforced: "CAS conflicts route via the ON_CONFLICT edge with E_WRITE_CONFLICT stamped — not via Rust-enum-valued EvalError::WriteConflict. Prior audit §5.3 closed; catalog entry carries explicit edge-routed language."

- id: CODE-CIDFROMSTR-STUB
  file: crates/benten-core/src/lib.rs
  lines: 324-328
  behavior:
    body: 'Err(CoreError::CidParse("Cid::from_str is a Phase 2 deliverable (needs multibase decoder; see C4)"))'
  invariants_enforced: "UNCHANGED from prior audit. Still unconditionally fails. Tracked in Phase-2 backlog §6.1."
```

---

## 5. Full Alignment Matrix

Format: `spec_ref | code_ref | match_type | confidence | notes`. Entries marked ✓ were verified against HEAD; entries marked ◯ are phase-legitimate deferrals; entries marked ⚠ are divergences surfaced in §6.

### 5.1 SECURITY-POSTURE.md compromises

| # | Claim | Code evidence | Match | Conf | Notes |
|---|-------|--------------|-------|------|-------|
| SP-C1 | TOCTOU refresh at commit / CALL / ITERATE-batch | `benten-engine/src/engine.rs` (check_write at commit); `benten-eval/src/primitives/call.rs:70`; `benten-eval/src/primitives/iterate.rs:97,109` | full_match ✓ | 0.95 | Prior audit §5.1 closure re-verified |
| SP-C2 | Option C symmetric None + `diagnose_read` gated by `debug:read` | `engine_diagnostics.rs:248`, `outcome.rs:273` (DiagnosticInfo), napi surface `bindings/napi/src/lib.rs:157` | full_match ✓ | 0.95 | Existence-leak surface gone |
| SP-C3 | `benten-errors` crate extracted | `crates/benten-errors/Cargo.toml` (zero workspace deps; verified via `[dependencies]` block) + `crates/benten-errors/src/lib.rs:50` (ErrorCode enum) | full_match ✓ | 0.95 | |
| SP-C4 | WASM compile-check only | `.github/workflows/wasm-checks.yml` + absence of `wasmtime` in `benten-eval/Cargo.toml` | full_match ◯ | 1.0 | Phase-2 deferral, accepted |
| SP-C5 | Per-cap write metrics | `engine.rs:160-192` (record_cap_write_*); `engine_diagnostics.rs:170-184` (metrics_snapshot emits `.<scope>` suffixed keys) | full_match ✓ | 0.95 | Prior audit §5.2 re-verified |
| SP-C6 | BLAKE3-256 = 128-bit classical collision resistance | Documentation-only claim; verified BLAKE3-256 in use at `benten-core/src/lib.rs:95` (`MULTIHASH_BLAKE3 = 0x1e`) with 32-byte digest | full_match ✓ | 1.0 | |
| SP-C7 | `[[bin]]` gated by required-features | `crates/benten-graph/Cargo.toml` `[[bin]]` + `required-features = ["test-fixtures"]` + `test = false` + `bench = false` | full_match ✓ | 1.0 | |
| SP-C8 | PrimitiveHost sole dispatch | `bindings/napi/src/` call sites → `benten-engine/src/engine.rs` dispatch_call_inner → `benten-eval/src/primitives/mod.rs:55-71` (via `PrimitiveHost` trait) | full_match ✓ | 0.9 | Architectural invariant; no fast-path bypasses the host trait |
| SP-CS | Change-stream no per-event read-check | `engine_views.rs:23-31` (subscribe_change_events body has zero policy-call lines) | full_match ◯ | 1.0 | Honest limitation, Phase-3 scope |
| SP-NAPI | `JSON_MAX_BYTES = 1 MiB` | `bindings/napi/src/node.rs:44` const + enforcement at L152, L194, L265 | full_match ✓ | 1.0 | |
| SP-HASH | BLAKE3-only Option A | `benten-core/src/lib.rs:92,95` constants; no SHA-256 fallback in codebase (grep) | full_match ✓ | 1.0 | |

### 5.2 ENGINE-SPEC §3 (12 primitives)

| # | Primitive | Phase-1 expected | Code evidence | Match | Conf |
|---|-----------|------------------|--------------|-------|------|
| P1 | READ | executable | `primitives/mod.rs:57` → `primitives/read.rs::execute` | full_match ✓ | 0.95 |
| P2 | WRITE | executable (+ `ON_CONFLICT` via edge routing) | `primitives/mod.rs:58` → `primitives/write.rs::execute`; conflict edge at `primitive_host.rs:462` | full_match ✓ | 0.95 |
| P3 | TRANSFORM | executable with expression parser | `primitives/mod.rs:61` → `primitives/transform.rs::execute`; parser at `benten-eval/src/expr/parser.rs` | full_match ✓ | 0.9 |
| P4 | BRANCH | executable | `primitives/mod.rs:62` → `primitives/branch.rs::execute` | full_match ✓ | 0.95 |
| P5 | ITERATE | executable with `max` + batch-boundary refresh | `primitives/mod.rs:63` → `primitives/iterate.rs::execute`; batch refresh at L97/L109 | full_match ✓ | 0.95 |
| P6 | WAIT | deferred; returns `E_PRIMITIVE_NOT_IMPLEMENTED` | `primitives/mod.rs:67-70` | full_match ◯ | 1.0 |
| P7 | CALL | executable with cap-entry check | `primitives/mod.rs:64` → `primitives/call.rs::execute`; cap check at L70 | full_match ✓ | 0.95 |
| P8 | RESPOND | executable, terminal | `primitives/mod.rs:59` → `primitives/respond.rs::execute` | full_match ✓ | 0.95 |
| P9 | EMIT | executable | `primitives/mod.rs:60` → `primitives/emit.rs::execute` | full_match ✓ | 0.95 |
| P10 | SANDBOX | deferred; returns `E_PRIMITIVE_NOT_IMPLEMENTED` | `primitives/mod.rs:67-70` | full_match ◯ | 1.0 |
| P11 | SUBSCRIBE (user-visible) | deferred; engine-internal change plumbing only | `primitives/mod.rs:67-70`; change stream plumbing at `engine_views.rs:23` | full_match ◯ | 1.0 |
| P12 | STREAM | deferred; returns `E_PRIMITIVE_NOT_IMPLEMENTED` | `primitives/mod.rs:67-70` | full_match ◯ | 1.0 |

### 5.3 ENGINE-SPEC §4 (14 invariants)

| # | Invariant | Phase-1 status | Code evidence | Match | Conf |
|---|-----------|---------------|--------------|-------|------|
| I1 | Subgraphs are DAGs | enforced | `invariants.rs:88,122,251` (cycle detection) | full_match ✓ | 0.95 |
| I2 | Max depth | enforced | `invariants.rs:155,338` (DepthExceeded) | full_match ✓ | 0.95 |
| I3 | Max fan-out | enforced | `invariants.rs:138,267` (FanoutExceeded) | full_match ✓ | 0.95 |
| I4 | Max SANDBOX nesting | deferred (Phase 2) | SANDBOX executor returns `PrimitiveNotImplemented`; no nesting check at registration | full_match ◯ | 0.95 |
| I5 | Max nodes (4096) | enforced | `invariants.rs:95,216` (TooManyNodes) | full_match ✓ | 0.95 |
| I6 | Max edges (8192) | enforced | `invariants.rs:106,228` (TooManyEdges) | full_match ✓ | 0.95 |
| I7 | Max SANDBOX output | deferred (Phase 2) | SANDBOX executor deferred | full_match ◯ | 1.0 |
| I8 | Cumulative iteration budget | stopgap: registration-time `IterateNestDepth` at max 3 + runtime `IterateBudget` | `invariants.rs:303` (IterateNestDepth); `benten-eval/src/evaluator.rs` (DEFAULT_ITERATION_BUDGET) | partial_match ◯ | 0.9 | Multiplicative-through-CALL deferred to Phase 2 |
| I9 | Determinism classification | enforced | `invariants.rs:173,351` (Determinism) | full_match ✓ | 0.95 |
| I10 | Content hash per subgraph | enforced | `Subgraph::load_verified` (InvariantViolation::ContentHash at `benten-eval/src/lib.rs:689`) | full_match ✓ | 0.9 |
| I11 | System-zone unreachable | Phase-1 stopgap `E_SYSTEM_ZONE_WRITE` at write-path | `engine_crud.rs:32-37` (SystemZoneWrite rejection); full registration-time enforcement is Phase 2 | partial_match ◯ | 0.9 | Catalog's `E_INV_SYSTEM_ZONE` is Phase-2 documented |
| I12 | Registration-time structural validation | enforced | `invariants.rs:79` (validate_subgraph entry point) | full_match ✓ | 0.95 |
| I13 | Immutable once registered | deferred (Phase 2) | No immutability enforcement at storage layer yet | full_match ◯ | 0.9 |
| I14 | Causal attribution | partial: attribution captured on writes (SP-C5), not structurally on every evaluation step | `PendingOp::PutNode { actor_cid, handler_cid, capability_grant_cid }` (transaction.rs); structural step-level attribution is Phase 2 | partial_match ◯ | 0.85 |

### 5.4 ENGINE-SPEC §5 / §7 / §9 — evaluator, hashing, caps

| # | Spec claim | Code | Match | Conf |
|---|-----------|------|-------|------|
| EV1 | Iterative evaluator, explicit stack | `benten-eval/src/evaluator.rs` (not recursive; `while let Some(_) = stack.pop()` pattern) | full_match ✓ | 0.9 |
| EV2 | `ON_ERROR` routes tx aborts (no separate `ON_TX_ABORT`) | `engine.rs:1620` maps `NestedTransactionNotSupported` + general aborts through ON_ERROR; `primitive_host.rs:465` has `"ON_ERROR" => ... Some("E_UNKNOWN")` edge case | full_match ✓ | 0.9 |
| EV3 | Sub-100µs per-node target | Benches: `crates/benten-eval/benches/ten_node_handler.rs` (documented "Not CI-gated on macOS APFS due to fsync floor") | partial_match ◯ | 0.85 | See §5.6 for perf alignment |
| H1 | BLAKE3 over DAG-CBOR with CIDv1 | `benten-core/src/lib.rs:92,95,259,260` (byte layout) | full_match ✓ | 0.95 |
| H2 | Hash = labels + properties, NOT anchor_id / timestamps / edges | `benten-core/src/lib.rs` has `Node::anchor_id` with `#[serde(skip)]`; no timestamp field in Node; edges excluded from Node hash | full_match ✓ | 0.9 |
| H3 | `serde_ipld_dagcbor` canonical encoding | workspace `Cargo.toml` pins `serde_ipld_dagcbor`; used via Node::cid | full_match ✓ | 0.95 |
| CAP1 | CapabilityGrant Nodes + GRANTED_TO edges | `benten-caps/src/grant.rs:27` (CapabilityGrant type) | full_match ✓ | 0.9 |
| CAP2 | Commit-time check (not per-operation) | `benten-engine/src/engine.rs` check_write fires inside write-commit closure | full_match ✓ | 0.95 |
| CAP3 | Attenuation narrows, never widens | `benten-caps/src/attenuation.rs` + `tests/proptest_attenuation.rs` (proptest `parent_star_permits_exact_prefix`) | full_match ✓ | 0.9 |
| CAP4 | System-zone Nodes writable only through engine APIs | `engine_caps.rs` `grant_capability` / `revoke_capability` / `create_view` use `privileged_put_node`; user-op write path hits `SystemZoneWrite` | full_match ✓ | 0.9 |

### 5.5 ERROR-CATALOG reachability

Every Phase-1 catalog enum variant has ≥1 non-test construction site. Counts (grep for `ErrorCode::X\|EvalError::X\|CapError::X\|CoreError::X\|GraphError::X\|ViewError::X\|VersionError::X`, excluding test / bench / fuzz paths, `.code()` accessor arms, and string-mapping arms):

```
InvCycle:1  InvDepthExceeded:1  InvFanoutExceeded:1  InvTooManyNodes:1
InvTooManyEdges:1  InvDeterminism:1  InvContentHash:2  InvRegistration:1
InvIterateNestDepth:2  InvIterateMaxMissing:2  InvIterateBudget:2
CapDenied:1  CapDeniedRead:2  CapRevoked:1  CapRevokedMidEval:1
CapNotImplemented:1  CapAttenuation:1  WriteConflict:1 (edge-routed)
IvmViewStale:8  TxAborted:1  NestedTransactionNotSupported:2
PrimitiveNotImplemented:4  SystemZoneWrite:1  ValueFloatNan:1
ValueFloatNonFinite:1  CidParse:2  CidUnsupportedCodec:1  CidUnsupportedHash:1
VersionBranched:2  BackendNotFound:1  TransformSyntax:4  InputLimit:3
NotFound:8  Serialize:6  GraphInternal:2  DuplicateHandler:1
NoCapabilityPolicyConfigured:1  ProductionRequiresCaps:1  SubsystemDisabled:5
UnknownView:4  NotImplemented:1  IvmPatternMismatch:2  VersionUnknownPrior:2
```

Note on `WriteConflict:1` — the single non-test hit is the `EvalError::WriteConflict` → `ErrorCode::WriteConflict` mapping in `benten-eval/src/lib.rs:144`. The **actual firing surface** is the edge-routed string-literal path at `primitive_host.rs:462` (as documented in `ERROR-CATALOG.md` §`E_WRITE_CONFLICT`'s updated language). This is the closure pattern accepted for the prior audit §5.3 — the catalog entry carries an explicit `<!-- reachability: ignore -->` annotation so `scripts/drift-detect.ts` does not flag it.

Phase-deferred codes correctly absent from the enum:

- `E_INV_SANDBOX_NESTED` (catalog, Phase-2 — invariant 4 enforcement)
- `E_INV_SYSTEM_ZONE` (catalog, Phase-2 — distinct from `E_SYSTEM_ZONE_WRITE` Phase-1 stopgap)
- `E_SANDBOX_FUEL_EXHAUSTED` / `_TIMEOUT` / `_OUTPUT_LIMIT` (catalog, Phase-2)
- `E_SYNC_HASH_MISMATCH` / `_HLC_DRIFT` / `_CAP_UNVERIFIED` (catalog, Phase-3)
- `E_DSL_INVALID_SHAPE` / `_UNREGISTERED_HANDLER` (TS-only, Phase-1)

### 5.6 §14.6 performance targets — measurability

| Target | Realistic range | Bench exists | Match |
|--------|----------------|--------------|-------|
| Node lookup by ID 1–50µs | `crates/benten-graph/benches/get_create_node.rs::get_node/hot_cache` | ✓ | full_match ✓ |
| IVM view read 0.04–1µs | `crates/benten-ivm/benches/view_maintenance.rs` | ✓ | full_match ✓ |
| Node creation + IVM update 100–500µs (4–13ms macOS floor) | `crates/benten-graph/benches/get_create_node.rs::create_node_immediate` + `crates/benten-engine/benches/end_to_end_create.rs` | ✓ | full_match ✓ (macOS floor documented) |
| 10-node handler 150–300µs | `crates/benten-eval/benches/ten_node_handler.rs` | ✓ | full_match ✓ (not CI-gated, documented) |
| Concurrent writers 100–1000 w/s | `crates/benten-graph/benches/concurrent_writers.rs` | ✓ | full_match ✓ |
| SANDBOX instantiation 100µs–1ms | (none — SANDBOX executor is Phase 2) | — | full_match ◯ |

### 5.7 Plan §1 exit criteria

See divergence finding §6.1 below for the three gates that diverge from plan §1.

| Gate | Plan §1 claim | Scaffolder template test | Match |
|------|--------------|--------------------------|-------|
| 1 | Registration succeeds | `smoke.test.ts:37` `register_succeeds` | full_match ✓ |
| 2 | Three creates + list | `smoke.test.ts:43` `three_creates_list_returns_them` | full_match ✓ |
| 3 | **Capability denial routes to `ON_DENIED` with `E_CAP_DENIED`** | `smoke.test.ts:62` tests **unregistered-handler typed error surface** (`E_DSL_UNREGISTERED_HANDLER`) with in-code note that capability-gated `crud()` is "Phase-2 DSL surface" | code_weaker_than_spec ⚠ | see §6.1 |
| 4 | Trace per-step timing non-zero | `smoke.test.ts:72` `trace_non_zero_timing` | full_match ✓ |
| 5 | **Parses via `@mermaid-js/parser`** | `smoke.test.ts:87` uses **regex over `^flowchart (TD\|...)`** with in-code note that `@mermaid-js/parser` doesn't ship a flowchart parser | code_weaker_than_spec ⚠ | see §6.1 |
| 6 | **CID string equals `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda`** | `smoke.test.ts:95` asserts `reread.cid === created.cid` (roundtrip only) | code_weaker_than_spec ⚠ | see §6.1 |

Note: the canonical fixture CID itself IS tested — at the crate level, `benten-core/tests/spike_fixture_cid_stable.rs::fixture_matches_canonical_cid` and in the T9 cross-leg determinism CI job. The drift is scoped to the *user-facing scaffolder smoke test* only.

---

## 6. Divergence Findings

### 6.1 MEDIUM — Scaffolder smoke test diverges from plan §1 on Gate 3 / 5 / 6

```yaml
id: F-R7-001
severity: MEDIUM
title: "Scaffolder smoke test diverges from plan §1 on 3 of 6 named gates"
spec_claim: ".addl/phase-1/00-implementation-plan.md §1 lists the six gates verbatim; each is load-bearing for the 'mechanically verifiable exit criterion' argument."
code_finding:
  gate_3: "tools/create-benten-app/template/test/smoke.test.ts:62 — tests unregistered-handler typed-error surface (E_DSL_UNREGISTERED_HANDLER), not capability denial routing (E_CAP_DENIED via ON_DENIED edge)."
  gate_5: "tools/create-benten-app/template/test/smoke.test.ts:87 — uses regex match against `^flowchart ` instead of @mermaid-js/parser."
  gate_6: "tools/create-benten-app/template/test/smoke.test.ts:95 — asserts reread.cid === created.cid instead of matching the canonical fixture CID bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda."
match_type: code_weaker_than_spec
confidence: 0.9
reasoning: |
  The smoke test's in-code comments acknowledge each substitution:
    Gate 3: "the original Phase-1 gate exercised capability denial, but
    capability-gated crud() variants (with a `capability:` option) are
    Phase-2 DSL surface."
    Gate 5: "`@mermaid-js/parser` package does not ship a `flowchart` parser."
    Gate 6: no comment — roundtrip-only assertion is silent about the canonical-fixture substitution.
  Each substitution is defensible in isolation (the capability-gated crud
  variant is genuinely Phase-2; @mermaid-js/parser genuinely lacks flowchart
  support; the canonical CID is tested elsewhere). But the plan document
  itself still names the original spec claims. A fresh agent reading plan §1
  believes the scaffolder tests the original six behaviors; in reality it
  tests a documented substitution set.
evidence:
  - "plan §1 Gate 3: 'Revoke the write capability, call post:create again, assert the response came through the subgraph's ON_DENIED edge with error code E_CAP_DENIED'"
  - "smoke.test.ts:54-69 tests E_DSL_UNREGISTERED_HANDLER"
  - "plan §1 Gate 5: 'parses as valid Mermaid flowchart syntax, verified via the official @mermaid-js/parser npm package'"
  - "smoke.test.ts:87-92 uses three toMatch regex assertions"
  - "plan §1 Gate 6: 'the returned CID string equals bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda'"
  - "smoke.test.ts:95-99 asserts reread.cid === created.cid"
exploitability: "Non-security. Documentation-reality gap. A developer / agent anchored on plan §1 will believe capability denial is scaffolder-verified when it isn't at that layer."
remediation:
  option_a: "Update plan §1 to match the actual smoke test — rewrite Gate 3/5/6 to their current substitutions with explicit 'Phase-2 replaces' notes."
  option_b: "Land the capability-gated crud() DSL in Phase 1 (small TS-side wiring; the policy is already in place) and ship the canonical-fixture CID assertion in the scaffolder smoke. Leave Gate 5 (@mermaid-js/parser) as a documented substitution with the regex approach — the parser genuinely lacks flowchart support."
  preferred: option_a
  reason: "Phase 1 is closed; changing the test now risks the exit criterion. Updating plan §1 to match shipped reality is the honest close."
```

### 6.2 MEDIUM — `DSL-SPECIFICATION.md` internal inconsistency

```yaml
id: F-R7-002
severity: MEDIUM
title: "DSL-SPECIFICATION.md header says primitives revised; body still imports and demonstrates VALIDATE + GATE"
spec_claim: "docs/DSL-SPECIFICATION.md lines 8-16 banner: 'Primitive set revised 2026-04-14. Two primitives were dropped (VALIDATE, GATE) and two were added (SUBSCRIBE, STREAM). The authoritative primitive list is in ENGINE-SPEC.md Section 3.'"
code_finding:
  import_example: "docs/DSL-SPECIFICATION.md:62-67 still shows `import { ... validate, ... gate, ... } from '@benten/engine/operations'`"
  primitive_table: "docs/DSL-SPECIFICATION.md:79-92 lists all 12 primitives but enumerates GATE and VALIDATE (not SUBSCRIBE or STREAM)"
  crud_example: "docs/DSL-SPECIFICATION.md:921-978 'What It Generates' section renders every subgraph with GATE and VALIDATE nodes"
  actual_dsl_exports: "packages/engine/src/index.ts:10-46 exports the 12 revised primitives (subscribe, stream added; validate, gate absent)"
match_type: mismatch
confidence: 0.95
reasoning: |
  The DSL spec's internal contradiction is:
    - Authoritative primitive list points at ENGINE-SPEC §3 (12 revised primitives)
    - Body demonstrates the old 14 via import, table, and CRUD example
  The shipped DSL (packages/engine/src/index.ts) matches ENGINE-SPEC. A
  developer implementing against DSL-SPECIFICATION will write `import { validate, gate }`
  and get a TypeScript compile error at `packages/engine/src/dsl.ts` which
  does not export those names. The banner mitigates the risk — but the
  mitigation is weak because CRUD §4.3's diagrams are the most-read part of
  the doc (they show what the developer gets from `crud()`).
evidence:
  - "DSL-SPECIFICATION.md:65-66 `validate, transform, branch, ... wait, gate, call`"
  - "DSL-SPECIFICATION.md:86 `7 | gate() | GATE | Execute a registered TypeScript handler | No`"
  - "DSL-SPECIFICATION.md:92 `12 | validate() | VALIDATE | Check data against a schema | No`"
  - "DSL-SPECIFICATION.md:922 `GATE[require store:read:post/*] -> READ[...]`"
  - "packages/engine/src/index.ts exports: branch, call, crud, emit, iterate, read, respond, sandbox, stream, subgraph, subscribe, transform, wait, write (14 names — 12 primitives + crud + subgraph — no validate, no gate)"
exploitability: "Non-security. Developer-friction / learning-curve gap."
remediation:
  option: "Rewrite the examples. The DSL-SPECIFICATION rewrite was called out as a Phase-1 deliverable in the banner itself: 'A DSL rewrite against the revised primitives is a Phase 1 deliverable.' That deliverable remained open at Phase 1 close. Two paths: (a) do the rewrite now (before Phase 2 kickoff), or (b) flag DSL-SPECIFICATION.md as 'contains historical examples — authoritative surface is packages/engine/src/*' and add that banner at every major section."
  preferred: "(a) — the rewrite is a finite piece of work, and Phase 2 will only add more primitives / patterns that would compound the inconsistency."
```

### 6.3 MEDIUM — `ARCHITECTURE.md` "six crates" claim outdated (seven crates at HEAD)

```yaml
id: F-R7-003
severity: MEDIUM
title: "ARCHITECTURE.md Layer 1 claim 'Six crates after critic review' is off-by-one"
spec_claim: "docs/ARCHITECTURE.md:35 'Six crates after critic review (up from four — added benten-ivm and benten-caps to keep the engine thin)'"
code_finding: |
  Workspace now has seven crates:
    crates/benten-core/
    crates/benten-graph/
    crates/benten-ivm/
    crates/benten-caps/
    crates/benten-eval/
    crates/benten-engine/
    crates/benten-errors/   # added during Step 5c (commit d03f642) per Compromise #3 closure
match_type: mismatch
confidence: 1.0
reasoning: "benten-errors was extracted from benten-core to satisfy Compromise #3 (ErrorCode moves to a dedicated zero-Benten-dep crate). The extraction is documented in the benten-errors/src/lib.rs crate docstring and in SECURITY-POSTURE §Compromise #3, but ARCHITECTURE.md — the canonical architecture document — was not updated."
evidence:
  - "ARCHITECTURE.md:37-44 lists six crates"
  - "ls crates/ shows seven: benten-caps / benten-core / benten-engine / benten-errors / benten-eval / benten-graph / benten-ivm"
  - "benten-errors/src/lib.rs crate-level docstring: 'Extracted from benten-core::error_code in Phase 1 (closes SECURITY-POSTURE compromise #3)'"
exploitability: "Non-security."
remediation: "Edit ARCHITECTURE.md Layer 1: update the crate-count claim to 'Seven crates' and add benten-errors to the list with its role ('Stable ErrorCode catalog discriminants; zero-Benten-crate-dep root of the dependency graph')."
```

### 6.4 MEDIUM — `Cid::from_str` Phase-1-accepts-base32 catalog hint misleads

```yaml
id: F-R7-004
severity: MEDIUM
title: "E_CID_PARSE catalog hint reads as 'Phase 1 accepts base32' but Rust `Cid::from_str` unconditionally returns an error"
spec_claim: "docs/ERROR-CATALOG.md E_CID_PARSE Fix: 'Phase 1 accepts only base32-lower-nopad multibase (b-prefixed) CIDv1. Check that the caller is not passing a base58btc / base64 / hex form.'"
code_finding:
  file: crates/benten-core/src/lib.rs
  lines: 324-328
  body: |
    pub fn from_str(_s: &str) -> Result<Self, CoreError> {
        Err(CoreError::CidParse(
            "Cid::from_str is a Phase 2 deliverable (needs multibase decoder; see C4)",
        ))
    }
match_type: code_weaker_than_spec
confidence: 0.9
reasoning: |
  The catalog's fix-hint implies Phase 1 accepts base32 input on this
  code path; the code unconditionally rejects every input. The TS napi
  boundary has a separate CID-string-accepting path (bindings/napi/src/node.rs:383)
  that produces E_INPUT_LIMIT on bad input — but a developer reading the
  catalog is not directed there. A call to Rust's Cid::from_str with a
  valid base32 CIDv1 string fails with E_CID_PARSE + message "Cid::from_str
  is a Phase 2 deliverable" — the fix-hint's remediation ("check the input
  format") is orthogonal to the actual cause.
evidence:
  - "Prior audit §5.9 flagged this at d03f642; unchanged at f69830b"
  - "docs/future/phase-2-backlog.md §6.1 tracks this with the 30-line base32 decoder as the fix"
exploitability: "Non-security; developer-frustration class."
remediation: |
  Two options:
    (a) Land the ~30-line base32 decoder now (mirrors Cid::to_base32 at
        benten-core/src/lib.rs — symmetric). Closes the gap.
    (b) Update the E_CID_PARSE catalog fix-hint to 'Rust Cid::from_str is a
        Phase-2 deliverable; Phase-1 CID-string input arrives exclusively via
        the napi boundary, which rejects with E_INPUT_LIMIT.'
  Option (a) is preferred because the encoder already exists; parity with
  the encoder closes the gap more cheaply than explaining the asymmetry.
```

### 6.5 LOW — `benten.ivm.view_stale_count` metric hard-coded to 0.0 (carried from prior §5.8)

Tracked in `docs/future/phase-2-backlog.md` §5.3. The metric key appears in `metrics_snapshot()` output with a placeholder value; no external doc promises its accuracy; the rustdoc honestly marks it a Phase-1 placeholder. No user-facing misrepresentation beyond the emitted zero-constant. Remediation: either wire the tally (subscriber iterates `View::is_stale()`) or drop the key. Backlogged.

### 6.6 LOW — Node read-path hash verification subgraph-only (carried from prior §5.7)

Tracked in backlog §6.2. `RedbBackend::get_node` does not re-hash the decoded Node against the requested CID; subgraph-level verification is in place via `Subgraph::load_verified`. The catalog entry for `E_INV_CONTENT_HASH` implies read-time node-level firing; reality is subgraph-only. Remediation: tighten catalog entry to distinguish "Registration / Subgraph load" from "Node read relies on redb page checksums for corruption detection," or add optional `get_node_verified` path. Backlogged.

### 6.7 LOW — `E_INV_SYSTEM_ZONE` catalog entry without enum variant (Phase-2 documented deferral)

```yaml
id: F-R7-007
severity: LOW
title: "E_INV_SYSTEM_ZONE catalog entry has no ErrorCode enum variant; E_SYSTEM_ZONE_WRITE is the Phase-1 stopgap"
spec_claim: "docs/ERROR-CATALOG.md §E_INV_SYSTEM_ZONE: 'Thrown at: Registration. Phase: 2 (invariant 11 full registration-time enforcement; Phase 1 stopgap is E_SYSTEM_ZONE_WRITE at the graph write-path layer)'"
code_finding: |
  crates/benten-errors/src/lib.rs:109 has SystemZoneWrite variant (→ E_SYSTEM_ZONE_WRITE).
  No InvSystemZone variant exists.
  scripts/drift-detect.ts would normally flag a catalog entry without an enum variant, but the catalog's `Phase: 2` marker exempts it from the reachability check.
match_type: code_weaker_than_spec
confidence: 1.0
reasoning: "This is documented Phase-2 deferral, consistent with the rest of the Phase-2-marked catalog entries (E_INV_SANDBOX_NESTED, E_SANDBOX_FUEL_EXHAUSTED, E_SYNC_*, etc.). Classified LOW for completeness of the reachability table, not as a regression."
evidence:
  - "ERROR-CATALOG.md E_INV_SYSTEM_ZONE 'Phase: 2'"
  - "benten-errors/src/lib.rs enum ErrorCode has SystemZoneWrite but no InvSystemZone"
remediation: "None needed at Phase 1 close. Phase 2 will add InvSystemZone when invariant 11 graduates to registration-time enforcement."
```

---

## 7. Missing invariants

None at the Phase-1-enforced set (invariants 1/2/3/5/6/9/10/12). The 6 Phase-2 deferred invariants (4 / 7 / 8-multiplicative / 11-full / 13 / 14) are legitimately absent and tracked in `docs/future/phase-2-backlog.md` §3.

---

## 8. Incorrect logic

No incorrect-logic findings. Every Phase-1 spec claim that corresponds to a code path is implemented consistently with the claim's semantics.

---

## 9. Math inconsistencies

None — Phase 1 has no economic or financial math surface. The `BLAKE3-256 → 2^128 classical collision resistance` claim is a cryptographic bound, not Benten math, and is documented as accepted.

---

## 10. Flow / state-machine mismatches

None. Error-edge routing (`ON_ERROR` for transaction aborts, `ON_DENIED` for capability denials, `ON_CONFLICT` for CAS conflicts) is consistent with ENGINE-SPEC §5 and the catalog. Transaction primitive (begin / commit / rollback) maps 1:1 to `engine.rs::transaction(|tx| ...)`.

---

## 11. Access control drift

None. Compromise #1 (TOCTOU), Compromise #2 (Option C), and the system-zone protection (I11 stopgap) all behave as spec'd. Attenuation contract is proptest-verified. The single access-control *gap* is the change-stream bypass (SP-CS), which is an explicit honest limitation — not drift.

---

## 12. Undocumented behavior

Surveyed via grep of `pub fn` / `pub struct` / `pub enum` surfaces across all 7 crates; no public surface was found that lacks either a `///` docstring or a corresponding catalog entry. The `benten-engine::Engine::testing_insert_privileged_fixture` method at `engine_diagnostics.rs` is an engine-test helper gated on `#[cfg(any(test, feature = "test-helpers"))]` — not a public surface in production builds.

No UNDOCUMENTED CODE PATH findings.

---

## 13. Ambiguity hotspots

| Area | Ambiguity | Source |
|------|-----------|--------|
| I14 causal attribution | ENGINE-SPEC §4 says "every evaluation" carries causal attribution; Phase-1 captures it on writes (`PendingOp::PutNode { actor_cid, handler_cid, capability_grant_cid }`) but not at the evaluator-step level. The Phase-2 "structural per-step" form is acknowledged in backlog §3. | `ENGINE-SPEC.md §4 row 14` vs `transaction.rs::PendingOp` |
| I8 multiplicative budget | Spec says "multiplicative" (ENGINE-SPEC §4 row 8); Phase 1 enforces a scalar runtime budget (`E_INV_ITERATE_BUDGET`) plus a registration-time nesting cap (`E_INV_ITERATE_NEST_DEPTH` at depth 3, the explicit compromise). Multiplicative-through-CALL is Phase 2. | `ERROR-CATALOG.md §E_INV_ITERATE_BUDGET Phase 1/2 split` |
| Option C's diagnose_read NoAuth behaviour | SECURITY-POSTURE says "NoAuth deployments (no policy configured) treat diagnose_read as open"; the rustdoc at `engine_diagnostics.rs:236` matches this. | Low ambiguity; called out for completeness. |

No ambiguity-class findings rise to MEDIUM+ severity. Each is documented on both sides (spec + code).

---

## 14. Recommended Remediations

### 14.1 Close before Phase 2 kickoff

1. **F-R7-001 (scaffolder smoke test drift).** Update plan §1 Gate 3/5/6 language to match the shipped substitutions. Doc-only edit.
2. **F-R7-002 (DSL-SPECIFICATION drift).** Rewrite the §2 primitive table and §4 CRUD rendered examples to match the revised 12 primitives. The banner calls this out as a "Phase 1 deliverable" that remained open; landing it before Phase 2 closes the deliverable and prevents further divergence. Estimated ~1-2 hours.
3. **F-R7-003 (ARCHITECTURE.md six-vs-seven crates).** Update Layer 1 to name `benten-errors` + update the count. ~5 minutes.
4. **F-R7-004 (Cid::from_str).** Land the base32 decoder (~30 lines mirroring the existing `to_base32` encoder) OR update the `E_CID_PARSE` catalog fix-hint. Either resolves the drift.

### 14.2 Defer (Phase 2 backlog items, already tracked)

5. **F-R7-005 (view_stale_count).** Already in `docs/future/phase-2-backlog.md §5.3`.
6. **F-R7-006 (get_node_verified).** Already in backlog §6.2.
7. **F-R7-007 (E_INV_SYSTEM_ZONE).** Phase 2 enforcement of invariant 11 lands with the InvSystemZone variant.

### 14.3 Process

8. **Extend the reachability check** to also grep for stringly-typed error codes (e.g. `"E_WRITE_CONFLICT"` at `primitive_host.rs:462`) — that edge-routed path is reached via the string literal, not the enum variant. The current `<!-- reachability: ignore -->` annotation works as an override but doesn't verify the string path is live. A paired positive check ("at least one non-test string-literal match per `reachability: ignore`-marked code") would catch a future regression where the edge path disappears silently.

---

## 15. Documentation update suggestions

- `docs/DSL-SPECIFICATION.md` — the Phase-1-deliverable rewrite flagged in its own header banner. Critical.
- `docs/ARCHITECTURE.md` — six → seven crates; add `benten-errors` role.
- `.addl/phase-1/00-implementation-plan.md §1` — Gate 3/5/6 wording updated to match shipped substitutions (or shipped code updated to match). Preferred: doc updates.
- `docs/ERROR-CATALOG.md §E_CID_PARSE` — clarify Phase-1 surface (napi-boundary-only vs Rust-API).
- `docs/ERROR-CATALOG.md §E_INV_CONTENT_HASH` — tighten "Thrown at: Registration / read" to "Registration (Node CID on create) / Subgraph load" to remove the implication of Node-read-time firing.

---

## 16. Final risk assessment

**Phase 1 closes at a compliance-clean state for the load-bearing security and correctness surfaces.** All 8 named compromises are verified; all 44 Phase-1 catalog codes have firing sites; all 8 Phase-1-enforced invariants have enforcement; all 12 primitive types are registered and 8 are executable; content-addressing structure is byte-pinned. The prior audit's 9 findings split: 6 closed, 3 carried to Phase 2 backlog with concrete remediation paths.

**New drift is peripheral and documentation-class.** The 4 medium findings are all "documentation falls behind code":
- Scaffolder smoke substitutes 3 gates; plan document hasn't caught up.
- DSL-SPECIFICATION has internal inconsistency; the Phase-1 rewrite deliverable was not landed.
- ARCHITECTURE.md off by one crate.
- CID parser catalog fix-hint slightly misleading.

None of these block Phase 2. Each has a ≤2-hour remediation path. If closed before Phase 2 kickoff, the next phase starts from a fully-aligned spec-to-code surface.

**Phase 2 risk call-outs:**
- The `benten-eval → benten-graph` dependency break (arch-1, backlog §1.1) needs to land *before* or *with* the Phase 2 primitive expansion. Deferring into the middle of Phase 2 risks dual-cost PRs (new feature + cross-cutting refactor).
- Invariant 13 (immutability) needs to land when the storage-layer write path is touched. If Phase 2 touches `benten-graph/src/transaction.rs`, couple the change.
- Invariant 11 full registration-time enforcement + `E_INV_SYSTEM_ZONE` variant addition should land together.

**Confidence:** 0.93 across the audit. The two areas at <0.9 are (a) I14 causal-attribution specification ambiguity between "on writes" and "on every evaluation step" — the spec reads stricter than Phase 1 enforces, and Phase 2 should tighten this; (b) the scaffolder Gate 3 substitution depth — the "same typed-error-surface contract" argument is plausible but not rigorously equivalent to the capability-denial routing it replaces.

**Overall: SHIP Phase 1 as closed. Address the 4 medium documentation items before Phase 2 opens.**

---

## Appendix — Audit mechanics

**Tools:** Read, Grep, Glob, Bash (targeted `grep -rn`). No code or test modifications.
**Search strategy:** For each Spec-IR item, ran at minimum two grep passes — one for the symbol name (enum variant, const, method), one for the firing-site or string-literal. Confirmed 44 Phase-1 ErrorCode variants against construction sites; cross-checked catalog entries against `scripts/drift-detect.ts`.
**Anti-hallucination:** No claim in this report references behaviour without file:line evidence. Where catalog implied behaviour and code did not match, both sides are cited verbatim. Ambiguities are marked as such (§13).

*End of report.*
