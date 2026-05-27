# X-Wing → MLKEM768-X25519 rename audit

> Per cryptographer-review-of-bird-of-prey-vs-lamps + dispatch-conventions §3.5s + Ben ratification 2026-05-27: rename references to our hybrid KEM combiner from "X-Wing" to **`MLKEM768-X25519`** (the IRTF CFRG Research-Group-ADOPTED name per `draft-irtf-cfrg-concrete-hybrid-kems-03`). The internal codepoint `0x647A` stays (matches IETF reservation per `draft-ietf-hpke-pq-04` Table 1).
>
> Pre-v1-beta-tag-must-fix INDEPENDENT of F-full outcome (the X-Wing-mislabel corrective is required regardless of which encrypt-to-recipient option is chosen).
>
> Authored 2026-05-27 by orchestrator-direct audit.

## The corrective in plain English

The codebase currently calls our hybrid KEM combiner "X-Wing." Two problems per cryptographer-review:

1. **Naming**: real X-Wing (per `draft-connolly-cfrg-xwing-kem-10`) is an INDIVIDUAL IETF draft; the IRTF CFRG Research-Group has adopted the **mathematically identical construction** under the name `MLKEM768-X25519` (per `draft-irtf-cfrg-concrete-hybrid-kems-03`). Using the RG-blessed name eliminates pre-WG-adoption risk.

2. **Math**: Benten's actual code at `0x647A` does NOT implement real X-Wing math. Real X-Wing: `SHA3-256(label || ss_M || ss_X || ct_X || pk_X)` with a specific 6-byte ASCII label. Benten ships: `HKDF-SHA256` with info-tag `"x-wing-v1-benten-0x647a"` + different input concatenation. The IACR CIC 2024 peer-reviewed tight IND-CCA proof for X-Wing does NOT transfer to Benten's combiner. **~24 LOC corrective**: swap Benten's combiner to the real `SHA3-256(label || ss_M || ss_X || ct_X || pk_X)` construction.

**Corrective scope = (a) fix the math** + **(b) rename references**.

## Categorization of files with X-Wing / x-wing / x_wing / XWing / xwing references

### Category A — Cross-ecosystem-boundary surfaces — MUST RENAME (per §3.5s)

These are where Benten's outputs are seen by external systems (consumers, documentation readers, tools):

| File | Hits | Action |
|---|---|---|
| `docs/SECURITY-POSTURE.md` | 2 | Rename references in Compromise #30 + Per-Node AEAD section. Note: the construction-correctness clarification (math change) also affects the security narrative. |
| `docs/V1-FROZEN-INTERFACE.md` | 2 | Rename in the v1-frozen-interface surface contract. |
| `docs/V1-FROZEN-INTERFACE-DEFERRED.md` | 2 | Rename in Row D-15 (Post-v1-beta hardening watch-list) references. |
| `docs/ERROR-CATALOG.md` | 1 | Rename in the ErrorCode catalog entry. |
| `docs/future/phase-4-backlog.md` | 1 | Rename in §3.10 Sharing & Confidentiality references. |
| `crates/benten-errors/src/lib.rs` | 1 | Rename in `ERecipientLacksKeysForSuite` error message + fix-hint text. |
| `crates/benten-errors/tests/stable_shape.rs` | (verify count) | Rename test data referencing X-Wing. |
| `packages/engine/src/errors.generated.ts` | 2 | Rename in TS-side `ERecipientLacksKeysForSuite` error message + fix-hint text. **Note**: this is GENERATED — regenerate via codegen after Rust-side rename lands. |
| `packages/engine/dist/errors.generated.js` + `.d.ts` | 2 + 1 | Auto-regenerated; rebuild after Rust-side rename. |
| `crates/benten-crypto-suite/INTERNALS.md` | (verify count) | Eventually published; rename references. |
| **CLAUDE.md baked-in #5** | 5 (on orchestration branch) | LOAD-BEARING; rename to "MLKEM768-X25519-style hybrid combiner (per `draft-irtf-cfrg-concrete-hybrid-kems-03`)" or similar. The Phase-4-Meta-Core 2026-05-26 sharpening paragraph already references the corrective; full rename. |

### Category B — Internal code surfaces — rename for accuracy (not blocked by §3.5s but should align)

These are internal Rust API + tests. Their dispatch codepoint `0x647A` stays; the NAMING in code/comments should align with the cross-ecosystem identifier:

| File | Hits | Action |
|---|---|---|
| `crates/benten-crypto-suite/src/cipher_suite.rs` | **20** | High-density references; rename in module docstring + struct docs + method docs + comments. Critical: ALSO fix the combiner math (~24 LOC change to use real X-Wing/MLKEM768-X25519 construction). |
| `crates/benten-crypto-suite/src/aead.rs` | 4 | Rename references. |
| `crates/benten-crypto-suite/src/lib.rs` | 1 | Rename crate-level docstring reference. |
| `crates/benten-crypto-suite/src/codepoint.rs` | 2 | Rename codepoint constant comments (codepoint NAME `HYBRID_X25519_MLKEM768` is fine; the comment can clarify it implements the MLKEM768-X25519 construction). |
| `crates/benten-crypto-suite/src/swap_matrix.rs` | 3 | Rename references in swap matrix construction. |
| `crates/benten-crypto-suite/src/structural_kdf.rs` | 2 | Rename references in structural KDF integration. |
| `crates/benten-graph/src/aead_wrap.rs` | 2 | Rename references in graph-layer AEAD wrap (Per-Node AEAD layer). |
| `crates/benten-graph/src/redb_backend.rs` | 1 | Rename reference in K_principal-seam stand-in comment. |
| `crates/benten-crypto-suite/Cargo.toml` | 2 | Rename references in package description / Cargo metadata. |
| **`crates/benten-crypto-suite/tests/tf3a_x_wing_hybrid_wrap_x25519_mlkem768_codepoint_0x647a.rs`** | (file name + body) | RENAME FILENAME to `tf3a_mlkem768_x25519_hybrid_wrap_codepoint_0x647a.rs`; update file body. |
| `crates/benten-crypto-suite/tests/tf4_gcore3c_full_swap_matrix_strip_resistance_pure_pq_nondefault.rs` | (verify count) | Rename references in test body. |
| `crates/benten-crypto-suite/tests/tf2_strip_resistance_negative.rs` | (verify count) | Rename references in test body. |
| `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance_additional.rs` | (verify count) | Rename references in test body. |
| `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs` | (verify count) | Rename references in test body. |
| `.addl/dispatch-conventions.md` | 3 (on orchestration branch) | §3.5s already cites the corrective; verify other §3.5s-prior references match. |

### Category C — Draft files — rename as part of batch-post pass (held until F-full ratifies)

These are 21 comment drafts held until F-full + rename ratify; the rename pass is part of the post-F-full doc cascade:

| File | Hits | Action |
|---|---|---|
| `.addl/phase-4-meta/f3-jose-comment-draft.md` | (verify) | v2 has "X-Wing-style combiner (`draft-irtf-cfrg-xwing`)" + bullet point references; rename to "MLKEM768-X25519 KEM combiner (per `draft-irtf-cfrg-concrete-hybrid-kems-03`)" or similar. |
| `.addl/phase-4-meta/new-comment-drafts.md` | (verify) | Multiple drafts reference X-Wing: sigstore/rekor-tiles #425 + w3c-ccg/di-quantum-safe #5 + ATProto #3928 + CFRG draft-prabel + possibly others. Rename per draft. |
| `.addl/phase-4-meta/f2-f7-comment-drafts.md` | (verify) | Light touch; check whether F2 multicodec PR endorsements + F7 W3C CCG comments reference X-Wing. |
| `.addl/phase-4-meta/position-b-blog-draft-v2.md` | (verify) | §1 lede + §2 library + §4 combiner + §5 caveats all may reference X-Wing. Major rename pass. Note: this is ALSO where the source-verify corrections (lidel verbatim + Sigstore attribution chain + ANSSI hybridation-mandate framing) apply per `position-b-v2-knowledge-limits-source-verification.md`. |
| `.addl/phase-4-meta/shape-5-iroh-outreach-package.md` | (verify) | Light touch; check Email-1 / Email-2 drafts. |

### Category D — Working / planning docs — rename as part of post-F-full revision pass

These will be revised by the post-F-full doc cascade anyway:

| File | Hits | Action |
|---|---|---|
| `.addl/phase-4-meta/g-core-pq-wire-r0-plan.md` | (verify) | Will be revised post-F-full; apply rename in same pass. |
| `.addl/phase-4-meta/position-b-revision-roadmap.md` | (verify) | Working planning doc; apply rename in same pass. |
| `.addl/phase-4-meta/00-implementation-plan.md` | (verify) | Implementation plan revision pass. |
| `.addl/phase-4-meta/position-b-revision-changelog.md` | (verify) | Should be updated with rename + source-verify corrections. |

### Category E — Historical / archival — LEAVE AS-IS

These are forensic records of decisions, ratifications, research, etc., from earlier moments. Updating them would distort the historical record. LEAVE AS-IS unless a specific update is needed:

- `.addl/HANDOFF-*.md` (all handoffs are historical snapshots)
- `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-25.md` (forensic state at that date)
- `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-26.md` (forensic state at that date — though if anything is incorrect that affects downstream, surface it)
- `.addl/phase-4-meta/NIGHT-SHIFT-2026-05-27.md` (THIS session's current state — keeps "X-Wing" with explanation; that's accurate context)
- `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md` (record of ratification; add a "see also rename audit + cryptographer-review-of-bird-of-prey-vs-lamps for the 2026-05-26 corrective" note rather than rewrite)
- `.addl/phase-4-meta/PQ-default-reframe-adjustment-map-2026-05-19.md` (historical)
- `.addl/phase-4-meta/r1*-triage.md`, `r1.2-triage.md`, `r1.3-BRIEF-addendum.md` (R1 critic-cycle outputs; historical)
- `.addl/phase-4-meta/r2-test-landscape.md` (R2 output; historical)
- `.addl/phase-4-meta/cryptographer-review-bird-of-prey-vs-lamps.md` (THE review that surfaced the corrective; references X-Wing AS THE FINDING — keep as-is)
- `.addl/phase-4-meta/R5-*` briefs (historical R5 wave briefs)
- `.addl/phase-4-meta/R5-WAVE-STATE.md`, `R6-LENS-CATALOG.md` (historical wave / lens cataloging)
- `.addl/phase-4-meta/pq-codepoint-research-agent-*` (research agent outputs)
- `.addl/spikes/SPIKE-I-crypto-agility-real-2026-05-21.md` (spike output)
- `.addl/spikes/README.md` (spike index)
- `.addl/pq-research/*` (PQ research artifacts; historical decision-input)

## Construction-math corrective (separate from rename; same wave)

The cryptographer-review found that the existing code at codepoint `0x647A` is NOT mathematically X-Wing. To inherit the IACR CIC 2024 tight IND-CCA proof, the combiner needs the swap from `HKDF-SHA256(info=...)` to real X-Wing math: **`SHA3-256(label || ss_M || ss_X || ct_X || pk_X)`** with the spec-defined 6-byte ASCII label.

**Files needing the math change** (estimated):
- `crates/benten-crypto-suite/src/cipher_suite.rs` — primary site
- Any `xwing_combine` / `kem_combine` / similar helper functions
- Test files exercising the combiner output bytes (will produce different byte-streams after the math change; expected breaking change at wire-format-fixture-pin level; THIS IS WHY THE CORRECTIVE MUST LAND PRE-V1-BETA-TAG — content signed under the old math wouldn't verify under the new code, but we're pre-v1-beta-tag so no shipping content exists yet)

Estimated scope: **~24 LOC code change + test-fixture-byte updates + KAT vectors from real-X-Wing-draft test corpus**.

## Sequencing

This corrective lands within G-CORE-PQ-WIRE-1 canary wave (post-F-full ratification, but the corrective itself is needed regardless of F-full outcome):

1. **Code corrective** (math + naming in code) — in the G-CORE-PQ-WIRE-1 canary brief
2. **Doc renames** (Categories A + B + D) — orchestrator-direct doc cascade post-F-full
3. **Draft renames** (Category C) — batch with the 21-draft post-F-full rename pass; happens before batch-post
4. **Historical/archival** (Category E) — LEAVE; no action
5. **Cryptographer audit** of the corrected combiner (~1 person-week external; pre-tag gate per cryptographer-review NF-2/C-GM-AUDIT)

## Verification post-rename

After the rename pass:
- `grep -rn "X-Wing\|x-wing\|x_wing\|XWing\|xwing" crates/ docs/ packages/` should return ONLY historical-archival hits (none in Category A/B/C/D)
- Test fixture bytes regenerated against real X-Wing math
- Cite-drift-detector clean
- External-cryptographer audit signs off on construction-correctness

## Cross-references

- [cryptographer-review-bird-of-prey-vs-lamps.md](cryptographer-review-bird-of-prey-vs-lamps.md) — origin of the X-Wing-mislabel finding (§4 / §10 substantive)
- [position-b-v2-knowledge-limits-source-verification.md](position-b-v2-knowledge-limits-source-verification.md) — companion source-verify work
- dispatch-conventions §3.5s (cross-ecosystem-identifier-as-content discipline) — the rule this audit operationalizes
- CLAUDE.md baked-in #5 — load-bearing crypto-agility commitment that needs rename in baked-in text
- [NIGHT-SHIFT-2026-05-27.md](NIGHT-SHIFT-2026-05-27.md) — comprehensive state capture

---

*Authored 2026-05-27 by orchestrator-direct audit. To be applied post-F-full ratification as part of the doc cascade + G-CORE-PQ-WIRE-1 canary wave.*
