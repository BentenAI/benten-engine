# G-CORE-9 FREEZE iterate-to-convergence council R1 triage

> **Scope:** R1 council (10 lenses) returned **3 BLOCKER + 30 MAJOR + 24 MINOR + 17 OBSERVATION** findings against `origin/main` @ `8cc4eddd` (PR #1344 TERMINAL FREEZE build-out). This triage doc consolidates findings into action bundles + records Ben-fork distinctive-angle calls made under the night-shift authorization. Authority: HARD RULE 12 disposition discipline; spec authority = `docs/V1-FROZEN-INTERFACE.md` + `RATIFIED-*-2026-05-*.md` + CLAUDE.md baked-in items.
>
> **Date:** 2026-05-24 (under night-shift stance; Ben-fork ratifications encoded with rebuttal window).
>
> **Status:** authoritative for the `g-core-9/r1-fix-pass` PR.

---

## 1. Executive summary

The R1 council exposes a substantive **doc-vs-code drift cluster** at the freeze boundary: V1-FROZEN-INTERFACE.md narrates surfaces as LANDED + post-tightened that at HEAD remain pre-tighten / partially-built / substrate-only-with-no-production-consumer. Critically, this is **inherent to a FREEZE wave that lands the contract narrative atomically with the build-out backlog**: V1-FROZEN-INTERFACE-BUILD-BACKLOG.md row 1 explicitly names the §8-A tighten substack `1.a/1.b/1.c` as a follow-up sub-pass per HARD RULE 12 clause-(b). The council surfaces that the freeze-doc narrative did NOT reflect that partial-landed reality; it reads as as-if-frozen-as-tightened when the bytes are not.

**Three structural Ben-fork classes surfaced** (encoded under night-shift stance; rebuttable at morning):

| Fork | Question | Orchestrator's call | Where executed |
|------|----------|---------------------|----------------|
| **Fork 1** | AAD `total_chunks` addition — extend `aad_per_chunk` to 3-tuple (security defense) vs. retract doc claim (preserve as-shipped bytes) | **DEFER-NAMED-NOW to G-COMP-1 wire-format-augmentation row + retract doc claim for v1-beta** — under-evidenced that we need the truncation defense pre-audit; harvesting-now-decrypt-later argument applies only to confidentiality (CLAUDE.md #5) not to chunk-truncation (signature-half catches via outer SnapshotBlob CID; per-chunk truncation surfaces as AeadError on the truncated slice). The brief's pre-encoded choice (option A) was "ADD now", but ground-truth shows pre-existing per-chunk byte-pin tests assert the 2-arg AAD layout (escalation-trigger criterion (c) fires: would break existing per-chunk byte-pin tests). Per CLAUDE.md baked-in #5 the wire-format coupling at freeze cannot be Ben-delegated. | Bundle 6 (doc-retract + DEFER-NAMED-NOW to G-COMP-1) |
| **Fork 2** | Substrate-frozen-but-consumer-unwired (5 substrates: WriteBoundaryChainValidator + InstallRecordReplayStore + 3 §8-E hooks + ProductionManifestEnvelopeRechecker + accept_atrium_share) | **doc-tighten — distinguish v1-beta signature-frozen vs consumption-deferred surfaces; deferred surfaces get explicit BELONGS-NAMED-NOW to G-COMP-1 in a new `docs/V1-FROZEN-INTERFACE-DEFERRED.md` tracked artifact** | Bundle 2 + Bundle 9 |
| **Fork 3** | `cargo-public-api` workflow `\|\| true` informational → required-failing | **FLIP TO FAILING** — spec item 9 mandates this for the freeze to bite; informational-only means the freeze-doc contract has no required-CI-check backstop | Bundle 10 |

**Hard-escalation trigger fired**: Bundle 1 §8-A visibility tighten cascade IS confirmed >50 call sites (75+ across `engine.get_node`/`engine.put_node`/`engine.get_node_label_only`/`engine.resolve_subgraph_cid_for_test`, plus napi binding consumes `engine.get_node`/`put_node` directly at `bindings/napi/src/lib.rs::Engine::{get_node, put_node}`). This is the substack named in build-backlog row 1 as deferred. Per HARD RULE 12 clause-(b), the disposition is **BELONGS-NAMED-NOW to G-COMP-1 §<row> "§8-A visibility tighten + napi cascade"** — retense V1-FROZEN-INTERFACE.md item 1 narrative to read PARTIAL-LANDED (consistent with build-backlog row 1.a/1.b/1.c naming) rather than ship a binary cascade in this PR that the prior R5 wave explicitly named to follow-up.

**Bundle count delivered in this PR:** 9 of the brief's 12 bundles. Three bundles surface as escalations or larger work re-scopings:
- **Bundle 1** (§8-A binary tighten) → ESCALATED + DEFERRED-NAMED to G-COMP-1; doc-narrative-retense applied in Bundle 9.
- **Bundle 6** (AAD total_chunks addition) → Fork 1 ratified to doc-retract path; tracked in Bundle 9.
- **Bundle 5** (8 hex-pinned byte-pin tests) → SCOPED to the 2 highest-value pins (codepoint integer pin + a new per-chunk AAD layout regression pin) + the remaining 6 BELONGS-NAMED-NOW to `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (G-COMP-1 row); the existing roundtrip + constant-position pins per L11 lens substantively cover the byte stability.

---

## 2. Per-BLOCKER disposition

### L2-BLK-1 — §8-A visibility cluster `pub → pub(crate)` tighten declared frozen but NOT LANDED

- **Severity:** BLOCKER
- **Where:** `crates/benten-engine/src/engine_crud.rs:139` + `crates/benten-engine/src/engine_wait.rs:1027/1056/1115/1137`
- **Evidence:** all four methods still `pub fn` at HEAD; cargo-public-api baseline contains them; rename never landed
- **Disposition:** **BELONGS-NAMED-NOW to G-COMP-1 §<row> "§8-A visibility tighten + napi cascade"** (HARD RULE 12 clause-b) + **FIX-NOW Bundle 9 doc-retense** to make V1-FROZEN-INTERFACE.md item 1 read PARTIAL-LANDED rather than as-frozen-tightened
- **Rationale:** the rename cascades through 75+ call sites in workspace + the napi binding's public `get_node`/`put_node` methods (`bindings/napi/src/lib.rs::Engine::{get_node, put_node}`) wrap these directly. The R5 wave explicitly named this substack as deferred in `V1-FROZEN-INTERFACE-BUILD-BACKLOG.md` row 1.a/1.b/1.c. The brief's Bundle 1 charter fires its own escalation trigger (criterion: ">50 call sites OR breaks napi binding") — confirmed: 75+ sites AND breaks napi binding. The consistent disposition with Fork 2 (doc-tighten substrate-frozen-but-consumer-unwired) is to mirror the same posture here: tighten the doc to mirror the partial-landed reality, name G-COMP-1 as the explicit destination, lock the v1-beta surface as-shipped (pub) with the understanding that G-COMP-1 will tighten before `v1-beta` tag.

### L2-BLK-2 — `ProductionManifestEnvelopeRechecker` frozen as auto-wired default — but type DOES NOT EXIST + wired default is admit-everything Noop

- **Severity:** BLOCKER
- **Where:** `docs/V1-FROZEN-INTERFACE.md:962-966` vs `crates/benten-engine/src/manifest_envelope_recheck.rs:203-222` + `crates/benten-engine/src/engine.rs:1908-1910`
- **Disposition:** **FIX-NOW Bundle 2 doc-retract per Fork 2 doc-tighten** + **Compromise #26 retense to label v1-beta posture explicitly** + **BELONGS-NAMED-NOW to G-COMP-1**
- **Rationale:** the Ben-pre-encoded Fork 2 disposition (doc-tighten) applies cleanly — the substrate (NoopManifestEnvelopeRechecker + ManifestEnvelopeRecheckOutcome + the §4.36 fail-CLOSED short-circuit at engine.rs:1462-1476) is FROZEN at v1-beta correctly; the ProductionManifestEnvelopeRechecker is a Composing-phase deliverable. Compromise #26 in SECURITY-POSTURE.md already discloses this; the gap is the contradiction between V1-FROZEN-INTERFACE.md item 12's narrative ("auto-wired default") and the binary. The empty-peer-DID structural-deny IS structurally enforced (verified L6-r1-14 OBS); the rechecker dispatch admits via Noop only for non-empty DIDs. Retense item 12 to mirror this reality.

### L2-BLK-3 — `accept_atrium_share` frozen — but function DOES NOT EXIST at HEAD

- **Severity:** BLOCKER
- **Where:** `docs/V1-FROZEN-INTERFACE.md:969-972` — grep across workspace shows zero `pub fn accept_atrium_share` matches
- **Disposition:** **FIX-NOW Bundle 2 — REMOVE phantom cite from V1-FROZEN-INTERFACE.md item 12** + **BELONGS-NAMED-NOW to G-COMP-1 (G24-D-FP-1 follow-up wave)**
- **Rationale:** phantom surfaces in a freeze contract are HARD RULE 12 clause-(b) violations — they freeze an interface that no consumer can actually call. The honest fix is to remove the phantom-cite + name the destination wave. Compromise #26 already discloses cross-peer-install verification is NOT live at v1-beta.

---

## 3. Per-MAJOR-sub-bundle disposition

### Sub-bundle A — `#[non_exhaustive]` sweep gap closure (FIX-NOW Bundle 3; spans L6 + L8 + L9 + L17)

**Findings:** L6-r1-1 (CapWriteContext + ReadContext) + L6-r1-2 (6 enums in benten-engine surface area) + L8-MAJOR-2 (item 11 sweep incomplete) + L8-MAJOR-3 (carve-out registry missing Strategy) + L9-DSL-MAJOR-2 (5 DSL public types) + L17-r1-2 (AuthorizationGrant + GrantKeyMaterial + UcanEnvelope).

**Total:** ~25 pub types pending the `#[non_exhaustive]` attribute + 1 audit test pin + 1 carve-out registry edit.

**Disposition:** **FIX-NOW** across Bundle 3. The attribute is additive, low-risk, structurally required by V1-FROZEN-INTERFACE.md item 11 + spec item 15.d. Adding post-v1-beta is SemVer-breaking per the doc's own argument. Risk is identifying all struct-literal construction sites + migrating to `Default::default()` + field-mutation pattern OR adding builders.

### Sub-bundle B — §3.5g cross-language mirror gaps (FIX-NOW Bundle 4)

**Findings:** L8-MAJOR-1 (Strategy::C → Strategy::Reserved rename incomplete across ErrorCode + EngineError + format strings + wire code + TS class + docstring drift at errors.generated.ts:1318) + L9-DSL-MAJOR-1 (3 new DSL ErrorCodes for parse/unknown-primitive/missing-respond + 6-surface mirror + remove drift-detect baseline grandfathered lines).

**Disposition:** **FIX-NOW** across Bundle 4. Strategy rename is wire-locking obsolete terminology forever post-v1-beta if not closed. DSL ErrorCode mints close §3.5g item 6 amendment's stated purpose (the existing baseline grandfathering at `scripts/drift-detect-error-variant-mirror-baseline.txt` was a pre-G-CORE-DSL chunk-3 stopgap; chunk-3 has shipped and the codes are now genuinely surface-public).

**CATALOG_VARIANT_COUNT delta:** 192 → 195 (3 new) — Strategy rename is a rename not a mint.

### Sub-bundle C — wire-format byte-pin shopping list (PARTIAL Bundle 5; per Fork 1 + scope-realism)

**Findings:** L11-MAJOR-2 (8 missing hex-pinned byte-pin tests) + L11-MAJOR-4 (codepoint integer-value pin missing).

**Disposition:** **PARTIAL FIX-NOW Bundle 5** — ship the codepoint integer-value pin (L11-MAJOR-4 close; ~50 LOC; load-bearing) + ship a per-chunk AAD layout regression-pin (locks the 2-arg layout per Fork 1 disposition) + the **remaining 6 byte-pin tests BELONGS-NAMED-NOW to G-COMP-1 §<row> "wire-format hex-pinning sweep"** in `docs/V1-FROZEN-INTERFACE-DEFERRED.md`. Per L11 lens: existing roundtrip + constant-position + format-version-byte-position pins are strong but not the hex-pinned-bytes the freeze contract names. The pragmatic disposition is to name the gap clearly (Fork 2 shape) rather than ship 7 inflated test files in this PR. The codepoint integer-value pin is non-deferrable (single critical test, ~50 LOC).

### Sub-bundle D — doc-coupling drift sweep (FIX-NOW Bundle 7)

**Findings:** L8-MINOR-1 + L10-r1-2 + L11-MAJOR-3 + L11-MINOR-1 + L12-MAJ-1 + L12-MAJ-2 + L17 MINORs 3+4+5 + L6-r1-7 + L6-r1-8.

**Disposition:** **FIX-NOW** Bundle 7. Mechanical cite-drift edits — all FIX-NOW per pim-1 §3.5b.

### Sub-bundle E — breaking-change ledger (FIX-NOW Bundle 8)

**Findings:** L18-r1-2 + L18-r1-3 (no consolidated `V1-BETA-BREAKING-CHANGES.md` catalog).

**Disposition:** **FIX-NOW Bundle 8 — author `docs/V1-BETA-BREAKING-CHANGES.md`**. Captures break-OK changes from `8141b94..HEAD` (Phase-4-Meta-Core open through G-CORE-9 FREEZE) including this fix-pass's additions (non_exhaustive sweep, 3 new DSL ErrorCodes, Strategy rename).

### Sub-bundle F — substrate-frozen-but-consumer-unwired (FIX-NOW Bundle 9 per Fork 2)

**Findings:** L6-r1-4 + L6-r1-5 + L6-r1-6 (WriteBoundaryChainValidator + InstallRecordReplayStore + 3 §8-E hooks).

**Disposition:** **FIX-NOW Bundle 9 — author `docs/V1-FROZEN-INTERFACE-DEFERRED.md` per Fork 2 doc-tighten** + retense V1-FROZEN-INTERFACE.md sections 8 + 12 to distinguish signature-frozen-and-consumed-at-v1-beta vs signature-frozen-consumption-deferred. Each deferred consumption gets explicit BELONGS-NAMED-NOW to G-COMP-1 §<row>.

### Additional MAJORs

- **L1-crypto-r1-1** (V1-FROZEN row 6 codepoint table mislabels 0x0003 as LIVE) → **FIX-NOW Bundle 11**
- **L2-MAJ-1** (empty-peer-DID synthesized `node-id:N` fallback bypasses Layer-A fail-CLOSED) → **FIX-NOW (1-line structural fix at resolve_peer_dids OR apply_atrium_merge short-circuit)** — Bundle 11 (crypto-correctness adjacent)
- **L2-MAJ-2** (FrameReplayMarker TOCTOU race) → **DEFER-NAMED-NOW to G-COMP-1 §<row> "F3 anti-replay atomic compare-and-swap"** — requires KVBackend trait extension or transaction-API wrap; medium-risk substrate change; not the freeze-wave's brief. Captured in `V1-FROZEN-INTERFACE-DEFERRED.md`. Compromise #23 retense to acknowledge in-window racy.
- **L2-MAJ-3** (no policy gate to REJECT classical-only 0x6400 envelopes at recipient) → **DEFER-NAMED-NOW to Phase-4-Meta-Composing v1-assessment-window** + SECURITY-POSTURE.md acknowledgement row — additive API (CryptoPolicy::require_hybrid_pq flag), not freeze-load-bearing. The classical-only construction IS cryptographically sound at v1-beta (Ed25519+X25519+ChaCha20-Poly1305); the gap is the consumer-side enforcement of the hybrid-mandatory marketing posture. Captured in `V1-FROZEN-INTERFACE-DEFERRED.md`.
- **L2-MAJ-4** (graph-AEAD seam threads attacker-controlled codepoint into key) → **DEFER-NAMED-NOW to G-COMP-1 §<row> "structural_kdf info-tag codepoint-binding"** — natural ChaCha20-Poly1305 defense via K_root divergence IS present at v1-beta (verified by docstring at aead_wrap.rs:500-506); the structural defense is incidental not explicit. Adding the codepoint to KDF info is ~5 LOC + wire-format coupling change requiring a backward-compat golden test. Not freeze-load-bearing.
- **L10-r1-1** (EngineBuilder runtime-handle-leak compile-test pin phantom) → **FIX-NOW Bundle 10** alongside Fork 3 workflow flip.
- **L12-MAJ-3** (deny.toml ↔ cargo-audit --ignore asymmetry) → **FIX-NOW Bundle 12 (residual sweep)** — single edit reconciling the comment + ignore flags.

---

## 4. MINORs disposition

Grouped by surface for inline closure in the relevant Bundle's commit:

- **L1-crypto-r1-2/3** (stale docstrings + unsupported_codepoint_msg_static swallow) → Bundle 11
- **L2-MIN-1** (no live-per-request resolver test pin) → **DEFER-NAMED-NOW to G-COMP-1 §<row> "§15.j live-per-request verification test"** in `V1-FROZEN-INTERFACE-DEFERRED.md` (single ~50 LOC test; non-load-bearing for the FREEZE contract; deferred to G-COMP-1 wave that needs it)
- **L2-MIN-2** (empty-DID-string structural-reject in chain-validator) → **FIX-NOW Bundle 11** (5 LOC + 1 test)
- **L6-r1-7/8** (V1-FROZEN actor_hint shape cite-drift + empty-peer-DID §4.25 half-true claim) → Bundle 7 doc-coupling
- **L6-r1-9** (AuthorizationGrant.audience_pubkey is Option for back-compat) → **DEFER-NAMED-NOW to Phase-4-Meta-Composing v1-assessment-window** + V1-FROZEN-INTERFACE.md item 8 advisory note in Bundle 9 doc-tighten
- **L8-MINOR-1/2/3** → Bundle 7 (file-cite drifts) + L8-MINOR-3 walk_share_scope_as additive helper → **DEFER-NAMED-NOW to G-COMP-1 §<row>** in DEFERRED doc
- **L9-DSL-MINOR-1** (EDslIoError + EDslBackendRejected re-export from errors.ts) → Bundle 4 (folded into ErrorCode mirror commit)
- **L9-DSL-MINOR-2** (compile_file substantive filesystem test) → **DEFER-NAMED-NOW to G-COMP-1 §<row> "DSL filesystem-exercising tests"** (additive test coverage; not freeze-load-bearing)
- **L10-r1-3** (IPC_METHODS const-vs-static_mut structural pin) → Bundle 10
- **L11-MINOR-1/2/3/4/5** → Bundle 7 (cite-drifts) + L11-MINOR-3 (typed-reject regression-guard for old codepoints) folded into Bundle 5 partial test work
- **L12-MIN-1** (recursive cargo doc invocation footgun) → **DEFER-NAMED-NOW to G-COMP-1 §<row> "recursive cargo invocation test hygiene"** in DEFERRED doc (covered by alternate ci.yml workflow; in-test invocation is belt+suspenders)
- **L17-r1-3/4/5** → Bundle 7 (cite-drifts)
- **L18-r1-4/5/6** → L18-r1-4 (V1-FROZEN-INTERFACE item 1 narrative retense) → Bundle 9; L18-r1-5 (`crates/benten-crypto-suite/INTERNALS.md` missing) → **DEFER-NAMED-NOW to Phase-4-Meta-Composing** (additive doc); L18-r1-6 (rename WIRE-FORMAT-INVENTORY.md → V1-WIRE-FORMAT-INVENTORY.md) → already-named-as-V1-prefix; no-op (verify) in Bundle 7.

---

## 5. OBSERVATIONs

Mostly pass-through (positive evidence). Notable items:
- **L2-OBS-1/2/3** — adversarial cross-check confirmations of C11b gate + hybrid signature defenses + AAD-rebinding defense — preserved as positive evidence in Bundle 9 doc-tighten.
- **L6-r1-10/11/12/13/14** — substrate correctness confirmations — preserved in Bundle 9 doc-tighten.
- **L1-crypto-r1-4/5** — future P-III items + nonce-length panic risk — captured in `V1-FROZEN-INTERFACE-DEFERRED.md` G-COMP-1 row "post-v1-beta hardening".
- **L12-OBS-1/2/3** → Fork 3 covers OBS-3; OBS-1/2 are pass-through (intentional design choices).
- **L17-r1-6/7** → Bundle 9 fold (Ed25519-hardcoded binding_sig observation; V1-WIRE-FORMAT-FREEZE-BEN-DECISION.md authored OR named-deferred — see Bundle 7).
- **L18-r1-7** → pass-through.

---

## 6. Ben-fork ratification placeholders (rebuttal window)

Under night-shift stance, the orchestrator made 3 ratifications via foundational memories (CLAUDE.md baked-in items + HARD RULE 12 + RATIFIED-* anchors). Each is rebuttable at next morning review.

### Fork 1 — AAD `total_chunks` addition

**Options:**
- (a) ADD `total_chunks` to `aad_per_chunk` — gain truncation defense, break existing per-chunk byte-pin tests, P-III wire-format change.
- (b) RETRACT doc claim to match 2-arg as-shipped — preserve existing byte-pins, defer truncation defense to G-COMP-1 (or post-audit hardening).

**Orchestrator's call:** **(b)** — retract doc claim + DEFER-NAMED-NOW to G-COMP-1. Rationale: (i) escalation criterion fired (would break existing per-chunk byte-pin tests, which themselves are part of the wire-format contract); (ii) per-chunk truncation isn't a HNDL-equivalent un-retrofittable property — outer SnapshotBlob CID binding + signature verification catches whole-blob truncation; per-chunk truncation surfaces as AeadError on the truncated slice; the residual is "partial-read silently looks short" which is a usage-time issue not an unforgeable-byte issue; (iii) the brief's pre-encoded option (a) was authored before scope was confirmed and treats the change as cheap (it isn't given existing pins). The CLAUDE.md #5 commitment is `aad_per_chunk` "binds plaintext_cid + chunk_index" — adding total_chunks is not a #5 requirement, it's a defense-in-depth nice-to-have.

**Rebuttal window:** if Ben prefers the security defense, the work is ~30 LOC + golden-test update + 1 new SECURITY-POSTURE.md row; should land at G-COMP-1 opening before any other wire-format-coupled work.

### Fork 2 — Substrate-frozen-but-consumer-unwired

**Options:**
- (a) FIX-NOW wire the 5 substrates (WriteBoundaryChainValidator + InstallRecordReplayStore + 3 §8-E hooks + ProductionManifestEnvelopeRechecker + accept_atrium_share) at G-CORE-9 — extends scope by ~300-500 LOC + ~20-30 test pins; the substrates are correctly factored but need WRITE-admission + delegate-cap + install-pipeline wire-up at 7+ call sites; risk: pulling G-CORE-8.2 forward.
- (b) DOC-TIGHTEN — distinguish signature-frozen vs consumption-deferred surfaces in V1-FROZEN-INTERFACE.md sections 8 + 12; author `docs/V1-FROZEN-INTERFACE-DEFERRED.md` enumerating G-COMP-1 destinations; reopen Compromise #26.

**Orchestrator's call:** **(b)** — the FREEZE wave's contract is to lock SIGNATURES at v1-beta, not to ship every consumer. Per `feedback_inverted_prework_post_campaign_phase` reasoning. The substrates' shape is locked correctly; the consumer wire-up is genuine G-COMP-1 work. The honest fix is to make the freeze-doc narrative reflect this.

**Rebuttal window:** if Ben prefers option (a), the work IS scoped at G-CORE-8.2 / G-COMP-1 — pulling it forward to G-CORE-9 doubles this PR's LOC but is consistent with "v1-beta tag means EVERY Layer-1/Layer-2/Layer-3 defense is structurally live." The split is "what does freeze MEAN" — signatures vs end-to-end behavior. Both are legitimate framings; the orchestrator picked the lower-risk one under night-shift discipline.

### Fork 3 — cargo-public-api workflow informational → required-failing

**Options:**
- (a) FLIP TO FAILING — required check at branch-protection level; freeze contract teeth from CI.
- (b) STAY INFORMATIONAL — relies on PR-summary surfacing.

**Orchestrator's call:** **(a)** — per spec item 9's explicit wording + the entire purpose of freezing the v1-beta surface. If the gate is informational, the freeze contract has no required-CI backstop.

**Rebuttal window:** none — this is the spec-text-literal disposition; no judgment-call discretion.

---

## 7. Bundle execution plan

| # | Bundle | Touches | Status |
|---|--------|---------|--------|
| 1 | §8-A visibility tighten | DEFER-NAMED-NOW → G-COMP-1 §<row>; doc-narrative-retense in Bundle 9 | ESCALATED |
| 2 | BLOCKERs 2+3 doc-tighten | V1-FROZEN-INTERFACE.md item 12 + SECURITY-POSTURE.md #26 | FIX-NOW |
| 3 | non_exhaustive sweep | ~25 pub types + audit test pin + carve-out registry edit | FIX-NOW |
| 4 | §3.5g cross-language mirror gaps | Strategy::C→Reserved rename + 3 DSL ErrorCode mints (CATALOG 192→195) | FIX-NOW |
| 5 | wire-format byte-pin shopping list | PARTIAL: codepoint integer-pin + per-chunk AAD regression-pin; remaining 6 DEFER-NAMED-NOW | PARTIAL |
| 6 | AAD total_chunks addition | Fork 1 → retract doc + DEFER-NAMED-NOW to G-COMP-1 | ESCALATED |
| 7 | Doc-coupling drift sweep | Multiple cite-drift edits | FIX-NOW |
| 8 | Breaking-change ledger | author `docs/V1-BETA-BREAKING-CHANGES.md` | FIX-NOW |
| 9 | Substrate-frozen-but-consumer-unwired doc-tighten (Fork 2) | author `docs/V1-FROZEN-INTERFACE-DEFERRED.md` + retense V1-FROZEN-INTERFACE.md §8 + §12 | FIX-NOW |
| 10 | cargo-public-api workflow flip (Fork 3) | `.github/workflows/cargo-public-api.yml` + 2 structural pins | FIX-NOW |
| 11 | Crypto-correctness L1 + L2 surgical fixes | 0x0003 codepoint table + empty-peer-DID structural-reject + empty-DID-string reject + stale docstrings | FIX-NOW |
| 12 | Residual MINORs sweep + deny.toml mirror | Inline closures | FIX-NOW |

---

## 8. Expected R1→R2 convergence

Per pim-13 + the Phase-2b R6 R1→R2 monotonic convergence shape, R2 is expected to:
- Verify the 3 BLOCKER doc-vs-code drifts are closed by doc-narrative-tighten + DEFERRED-doc authoring
- Verify the non_exhaustive sweep landed across ~25 pub types
- Confirm the 3 Ben-fork distinctive-angle calls (or surface Ben rebuttal)
- Expected R2 finding count: ≤5 MAJORs + ≤10 MINORs (front-loaded-convergence shape per iterate-to-convergence Q5 amendment)

Worst-case rebuttal scenario: Ben overrules Fork 2 → R2 surfaces a re-opened BLOCKER class around substrate wire-up; PR scope re-doubles. The DEFERRED doc + per-row destination naming makes the rebuttal cost low (re-target the deferred rows as in-scope at R2).
