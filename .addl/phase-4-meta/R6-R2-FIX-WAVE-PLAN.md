# R6 Round-2 FIX WAVE — dispatch-ready plan (non-GAP-1 findings)

**Status:** PREPPED, awaiting trigger. **Base = post-B2 main** (after PR #1366 squash-merges; GAP-1/B2 closes §4.1). **Dispatch trigger:** #1366 merged AND the issue-audit agent pool freed. Source of findings: `R6-R2-TRIAGE-FFULL.md` (council run `wf_040ac861-436`). GAP-1 is handled by B2 (#1366), NOT here.

**Ground-truthed 2026-06-06 (this prep): all findings below confirmed STILL LIVE on current main; locations pinned.**

## Sharding — file-ownership disjoint (4 agents, no two touch one file)

### Agent A — crypto-suite + core code/comments (B2-independent)
- **F-01** `crates/benten-crypto-suite/src/boundary.rs:21` `FORBIDDEN_DIRECT_DEPS`: ADD live deps `libcrux-ml-kem`, `sha3`, `sha2`, `secrecy`; REMOVE dead `ml-kem` entry (RustCrypto ml-kem is now dev-dep-only). Verify the fence test at `:239` still passes + add coverage for the new entries.
- **F-11** `crates/benten-crypto-suite/src/swap_matrix.rs` (+ `INTERNALS.md`): stale `HKDF-SHA256` → `SHA3-256` (structural-KDF is SHA3-256, not HKDF-SHA256). Grep the crate for other `HKDF-SHA256` occurrences and fix all.
- **F-12** `crates/benten-crypto-suite/src/primitives.rs:48` comment `// ... G-CORE-3 will add the deps and route the live impls.` → reflect SHIPPED reality (libcrux/sha3 wired live).
- **F-37** `crates/benten-crypto-suite/src/sizes.rs:32` comment cites `debug_assert_eq! arms below` that don't exist → correct to the actual validation site or remove the false cite.
- **F-13** `crates/benten-core/` (edge.rs / subgraph_spec / encryption_class.rs — grep `canonical_bytes`): stale `LE`/`#[ignore]` claim → `BE`/live (canonical bytes are big-endian + the arm is live, not ignored). Pin the exact comment from the finding before editing.

### Agent B — freeze-record docs (SOLE owner of SECURITY-POSTURE.md + V1-WIRE-FORMAT-INVENTORY.md)
- **F-07** `docs/SECURITY-POSTURE.md:2477` ("raised 5 → 18 by Ben 2026-06-06") vs `:2733` ("pending ratification") + `exemptions.toml` header: reconcile to ONE state. **NOTE: the 5→18 ratification status is Ben-gated** — if unresolved at dispatch, the agent makes the wording CONSISTENT and flags the canonical status for Ben rather than inventing a ratification.
- **F-08** `docs/SECURITY-POSTURE.md` (Compromise #43 refs at :28/:2927/:2943): verify #43 is the correct compromise number for "envelope-metadata leakage"; fix any false `### Compromise #43` header / mis-numbered cross-ref.
- **F-16** cargo-vet narrative "5+13=18" → "13-of-18" (or correct arithmetic framing) wherever it appears (SECURITY-POSTURE + any cargo-vet self-test doc-string). Agent to locate precisely.
- **F-06 (remainder)** `docs/V1-WIRE-FORMAT-INVENTORY.md`: B2 added the Layer-C *drop* rows (§26). ADD the missing **Layer-D** rows (cite `f_ld_2/4/8` + the `f_02` golden pins) — the Ben P-III freeze sign-off deliverable.
- **GAP-2** disclose the deterministic-CEK confirmation-oracle property (random AEAD nonce preserves confidentiality) in `THREAT-MODEL.md`/`SECURITY-PROOFS.md` — a documentation disclosure, not a code change. (Coordinate: SECURITY-PROOFS was B2-touched; this is additive prose.)

### Agent C — F-full crate code (SOLE owner of layer_c.rs + aad.rs + member.rs)
- **F-03** §11 `#[non_exhaustive]` sweep on the F-full crates' public enums/structs: `benten-membership-set`, `benten-drop` (incl `layer_c.rs`), `benten-id`. **EXCLUDE frozen-cardinality types** (Kind / RoleId and any enum whose stable cardinality is part of the freeze contract — verify against the §11 rule). Regenerate `docs/public-api/*.txt` for any crate touched. (Same class round-1 fixed for drop; current coverage: ms-set 8/35, drop 3/23, id 2/36 — agent decides per-type, not blanket.)
- **F-15** `crates/benten-membership-set/src/aad.rs` (+ the `layer_c.rs` dev-dep crosscheck): the AAD `body_cid` field has no length-prefix/width-check — add the 36-byte width assertion at assembly so a malformed CID can't shift the field boundary. Keep the drop-side byte-equality crosscheck consistent.
- **F-20** `crates/benten-membership-set/src/member.rs`: comment/decl-order asserts struct field order but CBOR is key-sorted — correct the comment to reflect canonical key-sorted CBOR (not declaration order). Agent to locate exact line.
- **F-19** `crates/benten-drop/tests/` AAD golden comment "131" → "127B" (the 11-field/127-byte group AAD; a stale size comment, not a golden hex). Agent to locate exact line; confirm the golden HEX itself is unchanged.

### Agent D — NAMED-CARRY entries (HARD-RULE clause-b: land the deferral entry NOW in its named destination; do NOT touch B's or C's files)
Per triage line 25. Each is a *named-destination entry*, not a code fix. Destinations: backlog sections / INVARIANT-COVERAGE / the G-CORE-PQ-WIRE ledger — NOT SECURITY-POSTURE or V1-WIRE-INVENTORY (Agent B owns those; route any of those there to B).
- F-04/F-05 (§11↔§16 non_exhaustive carry) · F-14 (privacy.rs input-independent → v1-GM) · F-18 (DeviceLink 0x6310..0x631F absent from CRYPTO-CODEPOINTS — **CRYPTO-CODEPOINTS was B2-touched; if a code-point row add, route to a doc agent or fold post-merge**) · F-21 (f_aad_2 nine-tuple→11-field carry) · F-22 (remote_permission "R5-FOLD-IN" stale) · F-23 (wasm-browser blocklist) · F-24/25/29/30/34 · F-26 (SHA cite 84280d31→d0ccf606 + Inv-21 `::crdt`) · F-27 (f_kat_4 INBOUND live / OUTBOUND→v1-GM) · GAP-4 (Inv-15 registered-not-fully-enforced → G-CORE-PQ-WIRE-1 ledger).

## BEN-GATED (NOT in this wave — pre-tag, surface together)
F-40 §16 §1.A.FROZEN inclusion · GAP-3/F-29 gossip §3.9-vs-§3.10 · multicodec 0xef/0xf0 supersession · F-07 5→18 ratification status (if still open) · F-17 §16 ~8-rows-vs-17-modules (Ben §16).

## Every brief MUST include (pim-N pre-flight + lessons)
1. `isolation: worktree`, commit-before-return, **ABSOLUTE-PATHS FORBIDDEN outside ${WORKTREE_ROOT}**, base off the post-B2 main SHA.
2. First action: tree-state freshness check vs merge-base (pim-N-mini-reviewer-rebase-staleness).
3. **Pre-push run the WORKSPACE gates, not scoped** (the scoped-vs-workspace-CI gap burned 2 cycles on #1365): `cargo fmt --check` (workspace) + `cargo clippy --workspace --all-targets --features testing -D warnings` + `cargo +stable clippy ...` + `RUSTDOCFLAGS=-D warnings cargo doc` + the §3.5g error-variant-mirror scanner + cite-drift-detector --all + (for public-api changes) the missing_docs_workspace test.
4. New pub error variants → catalog ErrorCode+TS mirror OR `// drift-detect: internal-only` trailing-comment OR baseline entry (§3.5g; scanner reads trailing-comment-on-variant-line only).
5. Every .md cite verified with fs/git/grep at author-time (cite-grep-verify-at-author-time).
6. NO AI attribution on commits. Author Benten-Ben <ben@benten.ai>.
7. Findings are ground-truthed-live as of this prep, but RE-VERIFY each is still live on the post-B2 base before editing (don't "fix" something B2 or a merged PR already closed).

## On all 4 return
Integrate (disjoint files → clean merge; single-writer if any overlap) → mini-review the CODE shards (A+C) → reconcile-PR → CI → Ben `--squash` → **R6 ROUND 3** (re-point `f-full-r6-council.js` artifactRef → new main + ADD `sender-origin-authentication` lens + priorMissedLenses) → iterate to 0 BLK/MAJ → pre-tag (Ben sign-offs above) → tag `phase-4-meta-core-close` (HOLD Ben).
