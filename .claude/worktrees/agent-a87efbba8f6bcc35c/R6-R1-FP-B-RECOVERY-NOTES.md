# R6 R1 FP-B Recovery Notes — worktree git pointer dangling

**Discovered:** post-final-fmt-commit (commit `87f598ce`), the
parent repo's `.git/worktrees/agent-a87efbba8f6bcc35c/` directory was
removed by external action (likely worktree auto-cleanup raced with
this agent's final pre-push gate). The worktree's `.git` file is a
pointer (`gitdir: /Users/benwork/Documents/benten-engine/.git/worktrees/agent-a87efbba8f6bcc35c`)
to a directory that no longer exists. All `git` commands from inside
the worktree fail with `fatal: not a git repository`.

The agent CANNOT push from inside the worktree boundary (the agent
isolation contract correctly forbids `cd` to the main repo + writing
to `.git/worktrees/` outside the worktree). Per `feedback_agent_isolation_escape_absolute_paths`
the agent declines to escape isolation.

## Branch HEAD SHA + commit chain

All 7 commits succeeded BEFORE the worktree pointer was removed. The
git OBJECTS for these commits are preserved in the parent's
common-dir at `/Users/benwork/Documents/benten-engine/.git/objects/`.
The objects observed at `87/` and `fb/` directories confirm presence
post-commit.

**Branch name:** `phase-4-meta-core/r6-r1-fp-b-crypto-security`
**HEAD SHA:** `87f598ce` (last commit; fmt application)

**Commit chain (oldest → newest), each branched from main `a0b75637`:**

1. `d9cdaf93` — fix(crypto-suite): F3 — AAD binds total_chunks for cross-chunk-truncation defense (R6 R1 fix-pass)
2. `39dd52cf` — fix(caps): L3-r1-1 — AuthorizationGrant binding-message binds scope (R6 R1 fix-pass)
3. `b5a6e758` — fix(platform-foundation): L2-R6-MAJOR-1 — wire T10-upgrade (a) into install_plugin (R6 R1 fix-pass)
4. `9192b08d` — docs(r6-r1-fp-b): HARD-ESCALATE L2-R6-MAJOR-2 PQ-hybrid app-layer wire-in fork
5. `08153695` — docs(wire-format): L11 — expand inventory items 11-24 (14 surfaces) (R6 R1 fix-pass)
6. `fb114dc1` — fix(crypto-suite,drop,graph): L1 — #[non_exhaustive] sweep across crypto+drop+graph (R6 R1 fix-pass)
7. `87f598ce` — style(r6-r1-fp-b): apply rustfmt to R6 R1 fix-pass commits

## Orchestrator recovery recipe

From the MAIN repo (NOT the worktree):

```bash
cd /Users/benwork/Documents/benten-engine

# Verify the commit object exists in the parent's object store:
git cat-file -p 87f598ce | head -10
# Should show tree + parent + author + commit message.

# Recreate the branch ref pointing at the head SHA:
git update-ref refs/heads/phase-4-meta-core/r6-r1-fp-b-crypto-security 87f598ce

# Verify chain:
git log --oneline phase-4-meta-core/r6-r1-fp-b-crypto-security -10

# Push:
git push origin phase-4-meta-core/r6-r1-fp-b-crypto-security
```

If the commit object is NOT in the parent's store (unlikely given
they were created via `git -c user.name=... commit` from inside the
worktree, which writes to the common object-dir), the work is lost
and would need to be re-done from this worktree's filesystem state
(all the source changes ARE still present on disk).

## Pre-push gate state at the time the pointer broke

- `cargo +stable fmt` — applied (reformatting absorbed by commit `87f598ce`)
- `cargo +stable fmt --check` — CLEAN
- `cargo +stable clippy --workspace --lib -- -D warnings` — CLEAN
- `cargo +stable clippy --workspace --all-targets -- -D warnings` —
  pre-existing test errors only (`for_test` API surfaces + `testing`
  module not present at HEAD; verified pre-existing via stash+check
  baseline). NO new warnings introduced by this branch.
- `cargo doc -p benten-graph -p benten-crypto-suite -p benten-caps -p benten-drop -p benten-platform-foundation --no-deps` with
  `RUSTDOCFLAGS="-D warnings"` — CLEAN
- Tests run + passing:
  - benten-crypto-suite: 112 tests (lib + integration)
  - benten-drop: subset run + g_core_9_non_exhaustive_audit_drop CLEAN
  - benten-caps: 243 tests
  - benten-platform-foundation: 205 tests
  - benten-engine: g_core_9_non_exhaustive_audit (16 tests after L1 extension)
  - Bundle-F3 inline tests: cross_chunk_truncation_fails + cross_chunk_inflation_fails PASS
  - Bundle-L3-r1-1 dedicated test file (3 tests) PASS
  - Bundle-L2-R6-MAJOR-1 e2e test file (2 tests) PASS
- `cargo deny` — NOT run (out of disk before this gate; cleaned cargo
  target post-disk-OOM but the worktree-pointer race happened before
  cargo deny could run).
- `jq .` validation on JSON artifacts — N/A (no JSON artifacts authored
  by this branch).
- Cite-drift sweep — N/A (no broken citations introduced).
- Public-api baselines:
  - `docs/public-api/benten-crypto-suite.txt` — updated for
    aad_per_chunk signature (F3).
  - `docs/public-api/benten-graph.txt` — updated for decrypt_chunk +
    encrypt_chunk signatures (F3).
  - L1 `#[non_exhaustive]` annotations are NOT in the cargo-public-api
    txt format; no baseline updates required.

## What this branch did

Per the FP-B brief 6 bundles:

| Bundle | Status | Commits |
|---|---|---|
| F3 — AAD total_chunks defense | CLOSED | `d9cdaf93` |
| L3-r1-1 — AuthorizationGrant scope-binding | CLOSED | `39dd52cf` |
| L2-R6-MAJOR-1 — T10-upgrade (a) wire-in | CLOSED | `b5a6e758` |
| L2-R6-MAJOR-2 — PQ-hybrid app-layer 3-site wiring | HARD-ESCALATED-WITH-FORK | `9192b08d` |
| L11 — wire-format inventory expansion | CLOSED | `08153695` |
| L1 — #[non_exhaustive] sweep | CLOSED | `fb114dc1` |
| (rustfmt) | CLOSED | `87f598ce` |

5 CLOSED + 1 ESCALATED-WITH-FORK + 1 fmt-cleanup.

## Files modified summary

- `crates/benten-crypto-suite/src/aead.rs` (F3 + L1)
- `crates/benten-crypto-suite/src/error.rs` (L1)
- `crates/benten-crypto-suite/src/swap_matrix.rs` (L1)
- `crates/benten-crypto-suite/src/varsig.rs` (L1)
- `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs` (F3)
- `crates/benten-graph/src/aead_wrap.rs` (F3 + L1)
- `crates/benten-graph/src/two_cid_map.rs` (L1)
- `crates/benten-graph/tests/tf3d_per_chunk_aead_iroh_block_size.rs` (F3)
- `crates/benten-caps/src/authorization_grant.rs` (L3-r1-1)
- `crates/benten-caps/tests/tf3b_scope_substitution_post_sign_rejected.rs` (L3-r1-1, NEW)
- `crates/benten-platform-foundation/src/plugin_lifecycle.rs` (L2-R6-MAJOR-1)
- `crates/benten-platform-foundation/tests/plugin_upgrade_rejects_peer_did_substitution_e2e.rs` (L2-R6-MAJOR-1, NEW)
- `crates/benten-drop/src/bundle.rs` (L1)
- `crates/benten-drop/src/envelope_sig.rs` (L1)
- `crates/benten-drop/tests/g_core_9_non_exhaustive_audit_drop.rs` (L1, NEW)
- `crates/benten-engine/tests/g_core_9_non_exhaustive_audit.rs` (L1)
- `docs/V1-FROZEN-INTERFACE.md` (F3)
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (F3)
- `docs/V1-WIRE-FORMAT-INVENTORY.md` (F3 + L11)
- `docs/SECURITY-POSTURE.md` (F3)
- `docs/public-api/benten-crypto-suite.txt` (F3)
- `docs/public-api/benten-graph.txt` (F3)
- `R6-R1-FP-B-L2-MAJOR-2-FORK.md` (NEW; orchestrator decision artifact)
- `R6-R1-FP-B-RECOVERY-NOTES.md` (THIS FILE)
