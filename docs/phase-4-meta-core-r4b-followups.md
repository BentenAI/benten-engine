# Phase-4-Meta-Core R4b follow-ups (tracked surface for orchestrator-local artifacts)

> **Status:** authoritative tracked-doc receiving the
> R4b-dispositioned BELONGS-NAMED-NOW items whose canonical
> "destination" would otherwise have been an orchestrator-local
> `.addl/` artifact (which is gitignored and so cannot serve as a
> HARD RULE 12 clause-(b) named destination per
> `feedback_no_defer_HARD_RULE` strict reading).
>
> **Provenance:** R4b R1 fix-pass (this PR; 2026-05-24) per the 6
> R4b lens reports' BELONGS-NAMED-NOW dispositions that named
> orchestrator-local destinations. The lens JSONs themselves live on
> sibling lens branches (`origin/phase-4-meta-core/r4b-l{1..6}-*`)
> and remain canonical for the per-lens evidence chain.
>
> **Reading-order:** consume this AFTER the R4b R1 lens JSONs
> (`origin/phase-4-meta-core/r4b-l{1..6}-*.json`) when scoping
> R4b-follow-up or G-COMP-1 work.
>
> **Update discipline:** updates land via PR + cite-drift sweep.
> When a follow-up closes (test-pin lands; brief-template edit lands
> in an in-flight wave brief; etc.) → strike-through here + add a
> `CLOSED-at-#<PR>` annotation; do NOT delete the row (forensic
> context retained per pim-13 / §3.12).

---

## L2 follow-ups (new-surfaces lens)

### L2-r4b-l2-2 — DropBundleError negative-arm test pins for `AuthorizationGrantFailed` + `CodecError`

- **Surface:** `benten_drop::bundle::DropBundleError` carries 7
  variants; 5 are exercised in tests (EnvelopeSignatureInvalid +
  PerNodeAeadAuthenticationFailed + PerNodeSignatureInvalid +
  UnsupportedDropMode + UnsupportedDropVersion — all via `tf3f_*`
  tests).
- **Gap:** 2 variants lack negative-arm test pins:
  1. `AuthorizationGrantFailed` — constructed at
     `crates/benten-drop/src/bundle.rs::DropBundle::consume_offline`
     when AuthorizationGrant verification fails on a per-Node
     consume path.
  2. `CodecError` — constructed at 4 sites
     (`encode_encrypted_node`, `decode_encrypted_node`,
     `to_cbor_bytes`, `parse_cbor_bytes`).
- **Named destination:** add ~2 small test files (or a single
  multi-arm test file) in `crates/benten-drop/tests/` exercising
  the 2 variants via mal-CBOR bytes feed + AuthorizationGrant
  with invalid binding_sig. ~50-100 LOC total.
- **Wave:** Phase-4-Meta-Composing (alongside Row D-21
  `benten-crypto-suite/INTERNALS.md` authorship + the broader
  benten-drop tightening cycle) OR opportunistic close at the
  next benten-drop touch.
- **Anchor:** R4b L2-r4b-l2-2
  (`.addl/phase-4-meta/r4b-l2-new-surfaces.json` lens JSON on
  `origin/phase-4-meta-core/r4b-l2-new-surfaces`).

### L2-r4b-l2-3 — `CryptoPrimitiveCallSiteAudit::scan_workspace()` integration test pin

- **Surface:**
  `crates/benten-crypto-suite/src/boundary.rs::CryptoPrimitiveCallSiteAudit`
  carries the `scan_workspace()` + `direct_primitive_use_outside_suite()`
  machinery (the integration crate's "ONE call site" rule from #5
  crypto-agility contract).
- **Gap:** no integration test in `crates/benten-crypto-suite/tests/`
  invokes `CryptoPrimitiveCallSiteAudit::scan_workspace()` and
  asserts the returned vec is empty. The CI workflow may run the
  scanner separately, but a Rust integration test pin would close
  the regression-guard at the same surface as the rest of the suite.
- **Named destination:** add
  `crates/benten-crypto-suite/tests/boundary_call_site_audit_workspace_clean.rs`
  (~20 LOC) calling `CryptoPrimitiveCallSiteAudit::scan_workspace()`
  + `assert_eq!(audit.direct_primitive_use_outside_suite(), vec![])`.
- **Wave:** Phase-4-Meta-Composing (couples to Row D-21
  `benten-crypto-suite/INTERNALS.md` authorship as documentation of
  the boundary discipline + this pin as the regression backstop) OR
  opportunistic close at the next benten-crypto-suite touch.
- **Anchor:** R4b L2-r4b-l2-3 + #5 crypto-agility contract "ONLY
  call site" rule (`CLAUDE.md` baked-in #5 refinement).

---

## L6 follow-ups (per-finding granularity lens)

### L6-MINOR-1 — G-CORE-3 sub-wave brief artifact reconstruction

- **Surface:** `.addl/phase-4-meta/R5-G-CORE-3{a,b,w,d,e,f,c}-BRIEF.md`
  files do NOT exist (verified via `git log --all --diff-filter=A
  -- '.addl/phase-4-meta/R5-G-CORE-3*'` = 0 commits). The
  R5 dispatches for G-CORE-3 sub-waves were orchestrator-inline
  (per-PR dispatch, brief content carried in agent prompts directly)
  rather than persisted as `.md` brief files.
- **Gap:** future-self / next-phase R0 reviewer cannot grep for the
  per-sub-wave dispatch contract in a single location. Reconstructable
  from (a) plan-doc `.addl/phase-4-meta/00-implementation-plan.md`
  §3 G-CORE-3 sub-wave definitions (lines 332-337) + (b) PR
  descriptions (#1319 / #1323 / #1324 / #1325 / #1340) + (c)
  per-sub-wave test cluster head comments.
- **Named destination:** add a one-line note to
  `.addl/phase-4-meta/R5-WAVE-STATE.md` (orchestrator-local
  artifact) acknowledging the briefs were dispatched orchestrator-
  inline + cite plan-doc §3 + the relevant PR descriptions as the
  recoverable contract source. ~10-15 LOC. **This row tracks the
  follow-up in tracked space; the actual edit lands in the
  orchestrator-local `.addl/` artifact when the orchestrator next
  touches `R5-WAVE-STATE.md`.**
- **Wave:** opportunistic close at the next R5-WAVE-STATE.md touch
  (which is gitignored / `.addl/`-local).
- **Anchor:** R4b L6-MINOR-1
  (`.addl/phase-4-meta/r4b-l6-per-finding-granularity.json` lens
  JSON on `origin/phase-4-meta-core/r4b-l6-per-finding-granularity`).

### L6-MINOR-2 — R5-BRIEF-common.md `_for_test` cfg-gating discipline literal pre-flight line

- **Surface:** R5-BRIEF-common.md (orchestrator-local `.addl/`
  template) is the canonical per-wave R5 implementer brief
  template. Per §3.6g pim-N-prior-phase-explicit-preflight rule, it
  enumerates discipline lines as LITERAL pre-flight checklist
  rather than memory-references or §-cross-references alone.
- **Gap:** R5-BRIEF-common.md does NOT enumerate the
  "`_for_test` must be cfg-gated" rule (V1-FROZEN-INTERFACE.md:154 +
  Cid::sample_for_test precedent). 5 of 6 G-CORE-3 sub-wave
  implementers shipped `pub fn _for_test` ungated (root cause of
  R4b L6-MAJOR-1 = the 115-surface workspace-pattern bug closed
  in this same PR via DEFER-NAMED-NOW to Row D-22).
- **Named destination:** append a checklist line to
  R5-BRIEF-common.md (in `.addl/phase-4-meta/`, gitignored
  orchestrator-local) per §3.6g literal pim-N enumerated
  checklist: "§8-A item 1 sub-clause `_for_test` discipline — any
  new `pub fn` ending in `_for_test` / `_for_testing` MUST carry
  `#[cfg(any(test, feature = \"testing\"))]` gating OR a
  V1-FROZEN-INTERFACE-DEFERRED.md Row D-N citation; precedent
  `Cid::sample_for_test` at
  `crates/benten-core/src/lib.rs::Cid::sample_for_test`." **This row
  tracks the follow-up in tracked space; the actual brief-template
  edit lands in the orchestrator-local `.addl/` artifact at the
  next G-COMP-1 R5 wave's brief authorship.**
- **Couples to:** Row D-22 sub-task 6 (the same brief-template lift
  is enumerated in the DEFERRED.md row's closure shape; this row +
  Row D-22 sub-task 6 reference the same edit from different
  angles — Row D-22 from the workspace-pattern-bug perspective,
  this row from the brief-template-completeness perspective).
- **Wave:** opportunistic close at the next G-COMP-1 R5 dispatch's
  brief-template touch.
- **Anchor:** R4b L6-MINOR-2 + V1-FROZEN-INTERFACE.md:154 +
  precedent `Cid::sample_for_test`.

---

## Process notes

### Why this doc exists

Some R4b BELONGS-NAMED-NOW dispositions named destinations like
`.addl/phase-4-meta/r4b-followups.md` or `R5-BRIEF-common.md` (in
`.addl/`) which are gitignored. Per HARD RULE 12 clause-(b) strict
reading (and per `feedback_no_defer_HARD_RULE` codification:
"phantom destinations" / "named destinations that don't exist" are
invalid dispositions), an orchestrator-local artifact cannot serve
as the BELONGS-NAMED-NOW destination — the named entry must land
in tracked-and-grep-able space when the disposition fires.

This doc receives those rows in tracked space; the orchestrator-
local sibling artifacts (when the orchestrator next touches them)
may carry the same content for in-context-recall purposes, but
THIS doc is the canonical home.

### Update on close

When a row closes (test pin lands; brief-template edit lands; etc.):
- Strike-through the row above (do NOT delete; preserve forensic
  context per pim-13 / §3.12)
- Add `**CLOSED at PR #<NN>:** <short description>` annotation
- The cite-drift sentinel (`cargo run -p cite-drift-detector`)
  catches stale references.
