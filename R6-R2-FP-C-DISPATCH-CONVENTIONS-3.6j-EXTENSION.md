# R6-R2-FP-C — `.addl/dispatch-conventions.md` §3.6j extension

**Intent.** Fold this extension into the live `.addl/dispatch-conventions.md`
§3.6j block at orchestrator integration time. This addendum is in the worktree
because `.addl/` is gitignored and not synced into agent worktrees.

---

## §3.6j extension — cite-grep-verify at author-time

**Origin (3+-recurrence trigger fired at R6-R2 lens cluster L11+L14+L16+L17+L18,
~40 phantom-cite instances workspace-wide; ratified at R6-R2-FP-C 2026-05-25).**
The existing §3.6j sweep-completeness self-verify discipline names the
post-sweep tool-output validation step ("when claiming a sweep is COMPLETE,
run the validation tool against the wave's own outputs"). Author-time CITE
verification was an implicit corollary that was repeatedly skipped, producing
the L11+L14+L16+L17+L18 phantom cluster. Codification makes the discipline
load-bearing at the author surface.

### Sub-rule (i) — cite-grep-verify at author-time (NEW)

Every `.md` cite to a `.rs` / `.ts` / `.tsx` / `.toml` / `.wat` / `.json` /
`.yml` source file, every `path::symbol` cite, every `#NNNN` PR-cite, and every
`path/to/glob_*.rs` test-file glob MUST be verified by the AUTHOR with the
relevant tool **before commit**:

  - **File-path cites** — verified via `ls` / `fs.exists` / `git ls-files`.
  - **Line-number cites** — verified by reading the cited file at the cited
    line + matching it against the cite's narrative; if the cited surface is
    on the §3.5b HARDENED point 3 high-churn-surface list, MUST be promoted
    to `path::symbol` form (line numbers drift on every refactor).
  - **Symbol cites** — verified via `grep "fn <symbol>\|struct <symbol>\|..."
    <cited-file>`.
  - **PR cites** — verified via `gh pr view <N> --json mergedAt,state`;
    if the PR is `CLOSED` and `mergedAt: null` it is a phantom cite (retract
    or move to a "considered but not merged" narrative).
  - **Glob cites** — verified by expanding the glob; if zero files match,
    rename to the actual file OR (if forward-looking) annotate
    `<!-- cite-drift-exempt: <reason> -->` adjacent to the cite.

**Pre-push gate.** `tools/cite-drift-detector --all --glob-cites` (mints
`scripts/drift-detect-cite-paths.ts` was the original brief shape; the
Rust-based existing detector was extended in-place per existing convention,
preserving the §3.5g cross-language-rule-mirror discipline — one canonical
detector implementation, not two parallel ones).

### Sub-rule (ii) — sibling-diff-walk on adjacent cites (NEW)

When adding a file-path cite to a doc, scan the IMMEDIATE adjacent cites in
the same section / table / paragraph for cross-cite drift. The L17 + L14
phantom-cite class clustered around adjacent-cite contagion: an author
updating one cite frequently missed the neighbor in the same row that
referenced the same renamed/moved surface.

### Sub-rule (iii) — JSON-output canonical-schema (NEW)

The cite-drift-detector now emits canonical-schema JSON per §3.6i when
invoked with `--json`:

```json
{
  "disposition": "FIX-NOW" | "APPROVE",
  "finding_count": <N>,
  "findings": [
    {
      "kind": "line-cite-glob-no-match" | "pr-cite-closed-not-merged" | ...,
      "path": "docs/V1-WIRE-FORMAT-INVENTORY.md",
      "line": 29,
      "message": "...",
      "disposition": "FIX-NOW"
    }
  ]
}
```

This supports the orchestrator's JSON-disposition pipeline (mini-review JSON
schema) without an additional schema-translation step.

### Sub-rule (iv) — PR-cite verification stays opt-in (`--check-prs`)

The `--check-prs` flag enables `gh pr view` verification per PR-cite. This
is OFF by default because:

  - it requires `gh` to be installed AND authenticated;
  - it is network-dependent (CI runs are network-allowed but the lint should
    not regress under transient network failure);
  - the cite-drift detector's default invariant is "zero external deps".

CI workflow invocations CAN enable `--check-prs` selectively (e.g. on a
weekly lane that catches drift over time without slowing every PR).

### Memory companion

Mint `~/.claude/projects/-Users-benwork-Documents-benten-engine/memory/feedback_pim_n_cite_grep_verify_at_author_time.md`.
Cross-link to:

  - `[[feedback_pim_n_sweep_completeness_self_verify]]` (§3.6j parent)
  - `[[feedback_review_finding_ground_truth_verify]]` (§3.5n parallel for
    orchestrator-side ground-truth)
  - `[[feedback_baseline_regen_must_match_ci_tool_invocation]]` (CI-tool
    invocation-parity sibling)
  - `[[feedback_post_fix_doc_coupling_preflight]]` (§3.5b parent for
    doc-coupling sweep)
  - `[[feedback_pim_cross_language_rule_mirror]]` (§3.5g — why we extended
    the existing Rust detector instead of minting a parallel TS scanner)

### Cleaned phantoms (R6-R2-FP-C evidence)

|  # | Lens / Class | Before (cite) | After (resolution) | Reason |
|----|--------------|---------------|--------------------|--------|
|  1 | L11 + L16 glob | `crates/benten-graph/tests/redb_backend_*.rs` (×2: V1-FROZEN, V1-WIRE-FORMAT-INVENTORY) | `crates/benten-graph/tests/redb_schema_version_envelope_pin.rs` + sibling pins | glob → real-named files |
|  2 | L11 + L16 glob | `crates/benten-eval/tests/execution_state_envelope_*.rs` | `crates/benten-eval/tests/exec_state_envelope_shape.rs` | glob → renamed real file |
|  3 | L11 + L16 glob | `crates/benten-engine/tests/g16_d_*.rs` | `crates/benten-engine/tests/device_attestation_envelope_direct.rs` | glob → COLLAPSE-P4-consolidated file |
|  4 | L11 + L16 glob | `crates/benten-id/tests/device_attestation_canonical_*.rs` | `crates/benten-id/tests/device_attestation.rs` + canonical_bytes_trait.rs | glob → real-named files |
|  5 | L11 + L16 glob | `crates/benten-id/tests/ucan_*.rs` | `crates/benten-id/tests/ucan.rs` + prop_ucan_attenuation.rs | glob → real-named files |
|  6 | L11 + L16 glob | `crates/benten-engine/tests/g_core_sandbox_*.rs` | `crates/benten-engine/tests/module_manifest_canonical.rs` + engine_open_rebuilds_module_manifest_active_set_from_persisted_zone.rs + sandbox_execute_via_engine_dispatch_invokes_executor.rs | glob → real-named files |
|  7 | L11 + L16 glob | `crates/benten-sync/tests/handshake_*.rs` | `crates/benten-sync/tests/handshake.rs` | glob → real-named file |
|  8 | L11 + L16 glob | `crates/benten-id/tests/tf3e_endpoint_id_*.rs` | `crates/benten-sync/tests/tf3e_zero_conversion_endpoint_id_is_verifying_key.rs` | glob → real file in DIFFERENT crate (cross-crate confusion) |
|  9 | L11 + L16 glob | `crates/benten-sync/tests/peer_id_*.rs` | `crates/benten-sync/tests/peer_id.rs` | glob → real-named file |
| 10 | L11 + L16 glob | `crates/benten-sync/tests/stamped_value_*.rs` | `crates/benten-sync/tests/loro_lww.rs` + loro_rich_type.rs (StampedValue is internal to crdt.rs::StampedValue) | glob → real coverage location |
| 11 | L11 + L16 glob | `crates/benten-engine/tests/suspension_store_*.rs` | `crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs` + redb_suspension_in_process.rs | glob → real-named files |
| 12 | L16 glob | `crates/benten-graph/tests/tf1_write_context_namespace_did_*.rs` | `crates/benten-graph/tests/tf1_989_cross_did_partition_isolation.rs` | glob → consolidated single pin |
| 13 | L16 glob | `crates/benten-crypto-suite/tests/conformance_*.rs` | `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance_additional.rs` + sibling tf4_* pins | glob → real-named files |
| 14 | L11 + L16 phantom file | `crates/benten-engine/tests/r1_fp_3_class_b_beta_read_node_as.rs` | `engine_read_node_as_put_node_pre_v1_closure.rs` + admin_ui_v0_* (×2) | phantom test file name → real test family |
| 15 | L14 phantom file | `crates/benten-platform-foundation/src/install_record.rs::InstallRecord::verify_user_signature` | `crates/benten-platform-foundation/src/plugin_manifest.rs::InstallRecord` (verify_user_signature lives in same file) | phantom module name |
| 16 | L14 phantom Compromise #30 host | `crates/benten-platform-foundation/src/install_record.rs` (Compromise #30 anchor) | host file is `plugin_manifest.rs:543/:592` | phantom host file |
| 17 | L17 line-cite drift | `crates/benten-caps/src/authorization_grant.rs:233` (AuthorizationGrant struct) | `::AuthorizationGrant` symbol form (line is :238 at HEAD; drift = 5 lines) | symbol-form per §3.5b HARDENED point 3 |
| 18 | L17 line-cite drift | `crates/benten-caps/src/authorization_grant.rs:166` (GrantKeyMaterial cite as KeyMaterial) | `::GrantKeyMaterial` symbol form (line is :178 at HEAD; drift = 12 lines + post-G-CORE-9 rename) | symbol-form + rename mirror |
| 19 | L17 line-cite drift | `crates/benten-crypto-suite/src/codepoint.rs (line ~64)` HYBRID_X25519_MLKEM768 | `::HYBRID_X25519_MLKEM768` symbol form (real line :199; drift = 135 lines) | symbol-form |
| 20 | L14 line-cite drift | `engine.rs:1462-1476` (×3: SECURITY-POSTURE, V1-FROZEN-INTERFACE-DEFERRED, V1-BETA-BREAKING-CHANGES) | `crates/benten-engine/src/engine.rs::apply_atrium_merge` symbol form (real line :1486-1500; drift = ~24 lines × 3) | symbol-form per §3.5b HARDENED point 3 high-churn surface |
| 21 | L14 line-cite drift | `engine.rs:1463-1477` (V1-BETA-BREAKING-CHANGES) | symbol-form to `apply_atrium_merge` | same as #20 |
| 22 | L14 line-cite drift | SECURITY-POSTURE.md cite "line 2619 below" (revocation-reach section) | grep-the-section-title (real section at line 2706; drift = 87 lines) | grep-discoverable per §3.6j |
| 23 | L14 phantom file | `crates/benten-engine/tests/tf3e_revoked_grant_yields_typed_revoked.rs` | `crates/benten-engine/tests/resume_with_revoked_grant_denies.rs` + `crates/benten-sync/tests/tf3e_replay_attack_ucan_expired.rs` | phantom file → real coverage |
| 24 | L16 phantom xref | `docs/ENGINE-SPEC.md` (SCHEMA-DRIVEN-RENDERING.md:216) | `docs/ARCHITECTURE.md` + `docs/HOW-IT-WORKS.md` (ENGINE-SPEC was MERGED at Phase-4-Foundation per cs-r1-2) | phantom doc file |
| 25 | L16 phantom xref | `ENGINE-SPEC §14.6 macOS caveat` (SECURITY-POSTURE.md:1177) | `docs/ARCHITECTURE.md` durability section | phantom doc file |
| 26 | L18 phantom PR-cite | `#1237` (V1-FROZEN-INTERFACE-DEFERRED.md:826, V1-BETA-BREAKING-CHANGES.md:247) | annotated as CLOSED-not-merged | phantom PR-cite class (L18-r6-r2-9 cleanup) |
| 27 | L17 cite-drift | `crates/benten-engine/tests/atriums_no_new_primitives.rs:58` (`packages/engine/examples/atrium-*/handler.ts` forward-pin) | `<!-- cite-drift-exempt -->` marker | future-pin documentation |
| 28 | L17 cite-drift | `crates/benten-engine/tests/r6_r1_fp_f4_s3c_workspace_check_write_routes_through_with_audience.rs:25-28+49` (4 high-churn line-cites) | symbol-form (`::apply_atrium_merge`, `::put_node_inner`, `::transaction`, `::check_capability`) | symbol-form per §3.5b HARDENED point 3 |
| 29 | L14 cite-drift | `crates/benten-eval/src/lib.rs:539-543` (INVARIANT-COVERAGE.md:39) | `::InvariantViolation` symbol form (real line :593; drift = 50-54 lines) | symbol-form |
| 30 | L14 cite-drift | INVARIANT-COVERAGE.md attribution_*.rs glob row | named-pins for benten-engine tests; glob retained where it expands cleanly | partial real-pin per glob expansion |
| 31 | L11/L16 cite-drift | `.addl/pq-research/landscape-*-2026-05-19.md` (SECURITY-POSTURE.md:2397) | `<!-- cite-drift-exempt -->` marker | gitignored directory |
| 32 | L11/L16 cite-drift | `.addl/ci-decisions-*.md` (phase-3-backlog.md:1888) | `<!-- cite-drift-exempt -->` marker | gitignored directory |

Total: ~32 cleaned cite-instances (per-finding granular; some lens
items absorbed multiple individual cleanings per pim-2-amendment §3.6b
sub-rule 4 per-finding granularity).

### CI wiring (cite-drift workflow)

The existing `.github/workflows/cite-drift.yml` already invokes the
detector. R6-R2-FP-C extends the invocation to include the new passes
via `--all` (already enabled) — `--glob-cites` is included in `--all`.
The `--check-prs` opt-in remains for a future weekly lane decision.

The non-blocking-PR-comment posture of D-PHASE-3-10 is retained; the
existing test `cite_drift_detector_finds_zero_drift_on_clean_main_post_g13_pre_a`
now passes against HEAD (was failing pre-R6-R2-FP-C because of the L14
phantom cites in phase-4-backlog.md §4.43 cluster).
