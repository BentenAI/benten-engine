# Option F+ §6.2 envelope-layer-unification — LENS L12 DEAD-CODE-IDENTIFICATION

**Branch:** `phase-4-meta-core/option-f-plus-lens-l12-dead-code-identification`
**Lens:** SENIOR CODE-MIGRATION ANALYST — dead-code-identification + pre-public-cleanup angle.
**Role:** What existing Benten code becomes dead/trimmable as the F-full encryption substrate (~28 unified amendments + 14 Compromise mints across 9 lenses + C1–C5 critique rounds) lands? Plus pre-public cleanup opportunities beyond F-full.
**Date:** 2026-05-27
**Tree-state pre-flight:** worktree clean @ `2172cb6d` against `origin/main`.

**Inputs (frozen SHAs):**
- F-full consolidated registry — `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` (`.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md`, 939 LOC, 28 unified amendments U1-U40 + 13 new Compromises #32-#44 + 3 invariants Inv-16/17/18 + 5 disagreements Q1-Q5)
- C1 elegant-shape critique — `3f5a4351`
- C2 cross-amendment composability — `9c548e5f`
- C3 10th-eye fresh-eyes cryptographer (3 NEW findings F1 MAL-BIND + F2 Inv-18 split + F3 composed-reduction prose; 5 refinements R1-R7) — `79c99aa5`
- C4 process-discipline (origin of the "backwards-compat-migration coverage gap" rebrand → this lens) — `6ea9718a`
- C5 formal-methods coverage gap fill (MF5) — `8e374a9d`
- `main` HEAD source: 14 workspace crates, ~108K LOC; key crates `benten-crypto-suite` (5,634 LOC) + `benten-graph` (9,522 LOC) + `benten-caps` (7,480 LOC) + docs/ (8,588 LOC across 6 V1-/SECURITY-/INVARIANT- artifacts).
- Ben directive (2026-05-27): "this project isn't public/being used by anyone yet so there's not really any backward-compatibility concerns though we could assess what becomes dead/trimmable."

**Authority:** ADVISORY. Final disposition rests with Ben + F-full R0 author. Per `feedback_review_finding_ground_truth_verify`, every dead-code item below is grep/read-verified at HEAD with file:line citations; downstream DISAGREE-WITH-EXPLANATION is first-class.

---

## §1 Executive summary (read this first)

**Top-line claim.** The F-full encryption substrate replaces ONE major existing stub (the G-CORE-3e K_principal-from-namespace_did synthesizer at `crates/benten-graph/src/redb_backend.rs:139-205`) and RESTRUCTURES a small number of currently-frozen surfaces (the `AeadEnvelope` wire shape; the LE codepoint encoding at 6+ sites; the `WrappedKey` cipher-suite shape; the `info`-tag literal `b"x-wing-v1-benten-0x647a"`). The aggregate *deletion* footprint is small (~150-300 LOC of stub bodies + ~40-80 LOC of obsoleted tests); the aggregate *restructure-without-delete* footprint is larger (~600-900 LOC across aead.rs + cipher_suite.rs + structural_kdf.rs + sizes.rs + swap_matrix.rs + aead_wrap.rs + 11 test-fixture renames + ~30 docstring updates). **Almost nothing in the workspace becomes "orphan dead code"** because the F-full work *extends* existing seams rather than parallel-rebuilding them. This is a structural win attributable to G-CORE-2/3 having been built as a codepoint-dispatched seam from the start.

**Pre-public-cleanup multiplier.** Per Ben's "no backwards-compat concerns" stance (2026-05-27), all dead-code items below can be DELETED outright at the SAME PR as their F-full replacement lands (Strategy-A per §3). No `#[deprecated]` softening, no parallel shim period, no `pub(crate)` half-step. This is structurally cleaner than the post-public-release norm and saves ~3-5 wave-days of soft-deprecation choreography that a v1.0+ project would otherwise pay. The deletion discipline mirrors CLAUDE.md baked-in #5 "no-shim" rule + HARD RULE 12 clause-(b) "BELONGS-NAMED-NOW".

**Disposition headline (per §3 trimming sequencing).** Of ~24 distinct dead-code/restructure items catalogued in §2:
- **18 items → Strategy-A (same-PR-as-replacement)** — the cleanest disposition; cumulative diff is smallest; bisect-risk is lowest because each PR is a single conceptual unit (replace stub + delete stub atomically).
- **4 items → Strategy-B (pre-flight dedicated cleanup PR)** — the LE→BE codepoint endianness migration (Q2) is the canonical pre-flight (~1 wave-day; bumps `ENVELOPE_FORMAT_VERSION_V1 → V2`; regen golden vectors), because it's mechanical + cross-cuts many F-full PRs + flushing it first means F-full PRs don't have to coordinate LE-vs-BE drift.
- **2 items → Strategy-D (post-F-full cleanup pass)** — the `_for_test` constructor sweep + the X-Wing-mislabel test-name cleanup; both are "tidy after the parade" janitorial work with no functional gate.
- **0 items → defer to Phase-4-Meta-Composing** — per HARD RULE 12: no "defer" disposition; every item lands NOW or has a named-NOW destination in the F-full PR sequence.

**Pre-public cleanup opportunities BEYOND F-full (§6).** 7 surfaces surfaced:
- (a) Three `_for_test` / `_for_testing` constructors that survived R6 R1 FP-A Bundle F1's gating sweep but could be deleted outright pre-v1-beta if call sites can route through real constructors.
- (b) Two `0.x` / `-rc` / `-pre` dep pins (ml-kem 0.2, slh-dsa 0.2.0-rc.5, ed25519-dalek 3.0.0-pre.7 via iroh 1.0.0-rc.0) that should be re-verified against latest releases before v1-beta tag (cargo-deny doesn't catch this).
- (c) The X-Wing-mislabel rename audit (referenced in brief; .addl artifact not present on this branch) deserves orchestrator follow-up to confirm Categories A-E are fully closed before F-full lands (otherwise F-full inherits the mislabel).
- (d) The `UcanEnvelope::unresolved_peer` field (named in phase-4-backlog.md:177 as a P-III Ben decision-point at G-CORE-9) is a clear pre-public-cleanup candidate — DELETE if no production purpose post-G-CORE-3e RotationLog.
- (e) The `aead_wrap.rs` 941-LOC wrapper-over-crypto-suite-aead has duplicated structure (own `to_le_bytes()` codepoint encoder at L596+L602) that should consolidate through the single canonical `canonical_binding()` per U1+U14 + U33 NAPI rule-mirror discipline.
- (f) The `crates/benten-eval`-side test-helper feature gates (`benten-caps/testing`) have a documented "feature pass-through chain" that could simplify if Sealed-trait test-doubles route through a single canonical re-export.
- (g) `cargo-machete` + `cargo-shear` should be wired into pre-push (§3.5 extension); the workspace has zero current dead-dep detector beyond cargo-deny's advisory/license focus.

**Confidence summary.**
- HIGH on §2 stub identification (K_principal synthesizer; LE encoding sites; X-Wing literal; AeadEnvelope wire fields).
- HIGH on §3 Strategy-A default with §6 named exceptions.
- MED-HIGH on §4 dep-graph (no `cargo-machete` available in environment; analysis is grep-based not compiler-output-based).
- MED on §5 doc dead-rows (some D-rows like D-25 / D-26 / D-15 could collapse together at v1-beta tag close).
- MED on §6 pre-public cleanup opportunities beyond F-full (judgment-heavy; needs Ben ratify-or-amend).

---

## §2 Per-crate dead-code catalog

Per-crate walk. Each item carries: **what** + **where (file:line)** + **why dead/restructured under F-full** + **which amendment(s)** + **proposed strategy** (cross-ref §3).

### §2.1 `benten-crypto-suite` (5,634 LOC) — heaviest restructure surface

This is the **ONLY crypto-primitive call site** per CLAUDE.md baked-in #5 + `crypto-agility-contract:6`. F-full restructures the envelope shape + adds 4-5 new modules + restructures 6 existing files. Net: heavy restructure; ~150 LOC clean deletion; ~600 LOC restructure-not-delete.

**Item L12-1 — `aead.rs::AeadEnvelope` byte layout (LE codepoint at L165 + `format_version: u8` discriminator at L60+L147+L320+L348).** Currently FROZEN at V1-FROZEN-INTERFACE.md item 4 (per file:443 + §6 item 4). Under U7 (BE codepoint pinning, Q2 disagreement), U14 (`aad_version: u8` byte canonicalization prefix), U9 (`#[non_exhaustive]` `EnvelopePayload` superseding the format_version-as-u8 discriminator), the entire wire-byte layout restructures into the §6.2 `EncryptedEnvelope { codepoint: u16 (BE), payload: EnvelopePayload (non_exhaustive), aad_binding: BindingContext (non_exhaustive) }` shape. Existing wire format (lines 18-26 docstring + L160-L210 serialize/deserialize) becomes the v1-LEGACY shape, but per Ben's pre-public-no-compat rule there's no v1-LEGACY — it's flat deletion + reconstruction. Restructure (NOT delete) ~80 LOC. **Strategy A** (same-PR-as-Wave-A foundational §7 wave per F-full §8).

**Item L12-2 — LE codepoint encoding sites (6 distinct call sites; ~12 LOC each).** Concrete sites verified:
- `crates/benten-crypto-suite/src/aead.rs:165` — `out.extend_from_slice(&self.cipher_codepoint.raw().to_le_bytes())`
- `crates/benten-crypto-suite/src/aead.rs:244-245` + L277-278 — chunk-index + recipe-index `.to_le_bytes()` (4 call sites; AAD-binding helpers)
- `crates/benten-crypto-suite/src/varsig.rs:47` + L107 — Varsig codepoint serialization
- `crates/benten-crypto-suite/src/sizes.rs:183` — TF-2 size-touching surface codepoint
- `crates/benten-crypto-suite/src/swap_matrix.rs:1539-1540` + L1548 + L1550 — strip-resistance AAD
- `crates/benten-crypto-suite/src/structural_kdf.rs:157` — KDF info-tag codepoint
- `crates/benten-graph/src/aead_wrap.rs:596` + L602 — per-Node AEAD wrap codepoint (NOTE: lives outside crypto-suite — see §2.5)

Under U7 BE-pinning (Q2 ratified per consolidator's recommendation): all 8 sites flip `to_le_bytes()` → `to_be_bytes()`. Per `feedback_pim_cross_language_rule_mirror` §3.5g, any TS-mirror sites must atomically flip too — verify with cite-drift-detector. Trim opportunity: when the `canonical_binding()` per U1+U14 lands as a single helper, all 8 call sites consolidate through THAT helper instead of inline `.to_be_bytes()` — net delete ~40-60 LOC of repeated byte-extension calls. **Strategy B** (pre-flight LE→BE migration PR per the brief's §3 Q2 cost estimate of ~1 wave-day; this PR also drops the `ENVELOPE_FORMAT_VERSION_V1` constant — see Item L12-3).

**Item L12-3 — `aead.rs:60` `ENVELOPE_FORMAT_VERSION_V1: u8 = 0x01` constant + `format_version` field on `AeadEnvelope`.** Under U14 (aad_version: u8 prefix bound INTO canonical AAD), the format_version semantics are subsumed by the new aad_version + the codepoint's CodepointLifecycle state (U16). The standalone wire-byte at envelope-byte-1 becomes redundant. **DELETE candidate** if F-full §6 envelope wire format drops position-1-byte and codepoint+aad_version+lifecycle cover the discriminator surface. Restructure ~10 LOC + obsoletes `aead.rs:460` `wire_format_carries_explicit_format_version_byte` test. **Strategy A** (same-PR-as-Wave-A).

**Item L12-4 — `cipher_suite.rs` `X_WING_HKDF_INFO_V1: &[u8] = b"x-wing-v1-benten-0x647a"` (L78) + the "X-Wing-style combiner" framing throughout `cipher_suite.rs::1-744`.** The X-Wing-mislabel rename audit (referenced in brief as `.addl/phase-4-meta/x-wing-to-mlkem768-x25519-rename-audit.md` — NOT present on this branch) prescribes renaming the combiner from "X-Wing" to "MLKEM768-X25519 hybrid combiner" because the actual construction is NOT the formally-specified X-Wing per draft-connolly-cfrg-xwing-kem-10. Under F-full this becomes a load-bearing audit-readiness item per L5 §4.5 (audit firms catch mislabels in pre-engagement scans). The literal `X_WING_HKDF_INFO_V1` constant + the `0x647a` codepoint info-tag must atomically rename across Rust + tests + docs + the codepoint-info-tag-as-AAD-binding per U1. **Restructure (NOT delete)** ~10-30 LOC of literal renames + ~20 docstring updates; the *info-tag bytes themselves* change (`b"x-wing-..."` → `b"mlkem768-x25519-hybrid-v1-benten-0x647a"` or similar), which means any sealed-at-old-info-tag envelope can't unwrap post-rename. Per Ben's no-backwards-compat: just delete the old literal + ship the new one + accept all in-flight envelopes break (acceptable pre-v1-beta because there are no production envelopes). **Strategy B** (pre-flight to F-full Wave-A; close the X-Wing rename audit before any new amendment work lands so amendments don't perpetuate the mislabel).

**Item L12-5 — `cipher_suite.rs::WrappedKey` shape (L172+L227).** Currently returns `WrappedKey { ml_kem_ct, x25519_pub, wrapped_key_ciphertext_with_tag }`-shape struct over the X-Wing-mislabel combiner. Under U17 (HpkeMultiBase variant), F-full restructures multi-recipient flow into HPKE-mode-base stanzas. `WrappedKey` may survive as a single-recipient convenience but Layer-C multi-recipient (the load-bearing Atrium use case per L9/A1) goes through the new `HpkeRecipientStanza` shape. **Restructure (NOT delete)** ~40-60 LOC; may end up `#[deprecated]` IF parallel-shape needed during F-full Wave-A→C transition, but per no-backwards-compat just rebuild atomically at Wave-C land. **Strategy A** (Wave-C same-PR).

**Item L12-6 — `structural_kdf.rs::derive_root` / `derive_step` info-tag scheme (L120-L162).** Currently uses `info = "root"` / `info = "step"` per Spike-E Interpretation-B. Under U20 (`k_principal_generation: u32` binding) + U19 (`recipient_key_generation: u32` binding), the KDF info-tag schema extends to bind these generation counters. The current `derive_root(k_principal, root_cid) → K_root` signature widens to `derive_root(k_principal, root_cid, k_principal_generation: u32) → K_root`. **Restructure (NOT delete)** ~30-50 LOC at the function signatures + every caller signature in `redb_backend.rs::derive_test_seam_key_from_cid_with_namespace`. The `from_bytes_for_test` per-test constructor used at `redb_backend.rs:183-184` (the K_principal synthesizer — Item L12-12) becomes dead simultaneously. **Strategy A** (Wave-B same-PR per F-full §7 §2 Layer-A vault wave).

**Item L12-7 — `sig.rs` (552 LOC) — `HybridSignature` shape (Ed25519⊕ML-DSA-65).** UNCHANGED by F-full (F-full is encryption-only; signature surface frozen at G-CORE-2/3 NF-4). Sig.rs is NOT dead under F-full. Mentioned for completeness — no trim opportunity.

**Item L12-8 — `swap_matrix.rs` (2,024 LOC — largest crypto-suite file).** Currently encodes G-CORE-3c full swap matrix (LIVE `0x647a` + `0x6400`; non-default-gated `0x647b` NF-1 + `0x647c` pure-PQ). The L1539-L1550 `to_le_bytes()` call sites are Item L12-2. Otherwise swap_matrix.rs is the codepoint-dispatch hub that F-full's new envelope variants extend — RESTRUCTURE (NOT delete) ~80-120 LOC to add HPKE codepoint dispatch arms (Layer-C `0x6300` + DropToRecipient `0x6500` + SealedSender `0x6510` slots per U22). No clean deletion. **Strategy A** (Wave-C same-PR for new arms; Wave-A same-PR for LE→BE flips).

**Item L12-9 — Tests in `crates/benten-crypto-suite/tests/` (14 test files).** Verified surface:
- `canonical_bytes_v1_codepoints_and_aad.rs` — RESTRUCTURE for new aad_version + canonical_binding shape per U1+U14; ~50-80 LOC update; existing assertions become wrong-direction.
- `tf3a_x_wing_hybrid_wrap_x25519_mlkem768_codepoint_0x647a.rs` — RENAME (X-Wing-mislabel) + UPDATE info-tag literal; ~10 LOC.
- `tf3a_structural_kdf_step_root_derivation.rs` — RESTRUCTURE for generation-counter binding per Item L12-6; ~20 LOC.
- `tf3a_pq_hybrid_wasm32_roundtrip.rs` — RESTRUCTURE wire-byte expectations for BE codepoint + dropped format_version field; ~30 LOC.
- `tf2_strip_resistance_negative.rs` — UPDATE AAD-shape assertions; ~20 LOC.
- `tf2_*` family — RESTRUCTURE for new typed-reject behavior per U2 strict-decode; ~30-50 LOC.
- `tf4_*` family — UNAFFECTED (signature-only; swap_matrix surface).

Net test impact: ~150-200 LOC restructure across the 14 files; ZERO test-file deletion (every test surface continues to discriminate a meaningful F-full property). New tests per U37 golden corpus + U38 dudect + U39 kani add ~4,200 test LOC per L4 §7 cost estimate; that's net-additive not net-replacement. **Strategy A** (per-wave same-PR; tests land with their corresponding production amendment).

**Item L12-10 — `discharge.rs::Issue835Discharge` (111 LOC).** Discharge marker for §3.5g cross-language rule-mirror enforcement. UNAFFECTED by F-full directly — but the discharge-shape pattern EXTENDS naturally to discharge the new amendments (every amendment in U1-U28 may need a sibling Issue-N discharge). No dead code; pattern continues. **Note** for F-full R0: budget ~150-200 LOC discharge-pattern extensions across the 28 amendments.

**Item L12-11 — `sizes.rs::SizeTouchingSurfaces` + `RedbSigHandle` (280 LOC).** TF-2 size-touching surface for "no hardcoded sizes" property. UNAFFECTED structurally by F-full (the property holds across new codepoints automatically). The L183 `to_le_bytes()` call is Item L12-2.

### §2.2 `benten-graph` (9,522 LOC) — heaviest STUB-DELETION surface

**Item L12-12 — `redb_backend.rs::derive_test_seam_key_from_cid_with_namespace` K_principal STUB (L139-L205).** THIS IS THE BIGGEST DEAD-CODE ITEM in the workspace. Reproduced from grep:

```text
crates/benten-graph/src/redb_backend.rs:139: /// **K_principal seam.** The per-DID `K_principal` material lives
crates/benten-graph/src/redb_backend.rs:141: /// (#989 / #1301 substrate). At this wave the K_principal is
crates/benten-graph/src/redb_backend.rs:145: /// K_principal storage seam landing first. The seam is named at
crates/benten-graph/src/redb_backend.rs:146: /// `docs/future/phase-4-backlog.md` §3.10 G-CORE-3e (K_principal
crates/benten-graph/src/redb_backend.rs:149: /// **⚠️ Confidentiality limit at this wave (K_principal-seam stand-in).**
crates/benten-graph/src/redb_backend.rs:151: /// `namespace_did` `Cid` bytes are the ONLY inputs to the K_principal
crates/benten-graph/src/redb_backend.rs:182:     let k_principal_bytes = blake3::keyed_hash(&K_PRINCIPAL_DOMAIN_KEY, did_bytes);
crates/benten-graph/src/redb_backend.rs:183:     let k_principal = benten_crypto_suite::structural_kdf::StructuralKdfKey::from_bytes_for_test(...)
```

The synthesizer is explicitly a STUB. Under Layer-A vault (F-full §2 + R0 §2; Argon2id + DAK + ChaCha20/XChaCha20-Poly1305 per U32 + AAD-binding per U20 `k_principal_generation`), the K_principal STOPS being deterministically-derived-from-namespace_did and STARTS being unsealed-from-vault-on-unlock. The entire function body (L161-L205 = ~45 LOC + the `K_PRINCIPAL_DOMAIN_KEY` constant + the `from_bytes_for_test` usage) becomes obsolete. The function signature `derive_test_seam_key_from_cid_with_namespace(namespace_did, plaintext_cid) → K(N)` is replaced by `vault.derive_kn(namespace_did, plaintext_cid, k_principal_generation) → K(N)` where vault is the live Layer-A vault handle. **DELETE ~70 LOC** + delete the docstring + replace all caller sites (~3-5 sites across redb_backend.rs L1632/1673/1686/2464/2583/2623/2693/2745/2768 per grep). **Strategy A** (Wave-B same-PR; Layer-A vault landing is the structural prerequisite, so it's atomic).

**Item L12-13 — `redb_backend.rs::K_PRINCIPAL_DOMAIN_KEY` constant (~L170-176 area).** The stable 32-byte domain key fed into BLAKE3 keyed_hash for K_principal synthesis. DELETE atomically with Item L12-12. ~5 LOC.

**Item L12-14 — `redb_backend.rs::derive_test_seam_key_from_cid_with_namespace` doc-comment paragraphs L139-L165 (~25 LOC of docstring).** The "K_principal seam" + "Confidentiality limit at this wave" + "G-CORE-3e named carry-over" docstrings become obsolete + replaced by Layer-A vault docstring. **DELETE + REWRITE** ~25 LOC. Pim-1 (`feedback_post_fix_doc_coupling_preflight`) verification: the same paragraph cross-references `docs/SECURITY-POSTURE.md` + `docs/future/phase-4-backlog.md §3.10 G-CORE-3e` — both must update atomically (§5 docs sweep). **Strategy A**.

**Item L12-15 — `redb_backend.rs:795` + L821 `GRAPH_SCHEMA_VERSION.to_be_bytes()`.** Already BE; no change under U7. Mentioned for completeness (not dead code).

**Item L12-16 — `aead_wrap.rs` (941 LOC) — wrapper-over-crypto-suite-aead.** This file is a per-Node AEAD wrap layer for the storage path (G-CORE-3d). Currently re-exports + locally-constructs AAD via `suite_aad_whole_content` + `suite_aad_per_chunk` + `suite_aad_per_recipe` + has its OWN `to_le_bytes()` calls at L596+L602 for codepoint encoding. Under U1+U14 (single `canonical_binding()` source-of-truth) + U33 (NAPI single-source rule-mirror), the local `to_le_bytes()` codepoint extension should consolidate through the canonical_binding helper. Restructure ~80-120 LOC; net delete ~20-40 LOC of duplicated structure. The 941-LOC file may shrink to ~750-850 LOC. **Strategy A** (Wave-A same-PR with canonical_binding mint).

**Item L12-17 — `aead_wrap.rs::EncryptedNode::with_aad_plaintext_cid_for_test` (L184).** Test-only constructor that mutates plaintext_cid AAD-binding to verify rebinding-attack defense. Under U1+U14+U3 (canonical TLV; codepoint+aad_version+CID bound atomically), this `_for_test` constructor still has a meaningful adversarial-test purpose but its INTERNAL shape changes. RESTRUCTURE (NOT delete) ~10 LOC. **Strategy A**.

**Item L12-18 — `two_cid_map.rs` (172 LOC) — TwoCidStore in-memory mapping.** Named at G-CORE-3e carry-over (phase-4-backlog.md:174) as the swap-point for the real `iroh_blobs::FsStore`. UNAFFECTED by F-full directly; named-deferred to its own G-CORE-3e-followon iroh-blobs wave. Not dead code yet; named-NOW destination exists.

### §2.3 `benten-caps` (7,480 LOC)

**Item L12-19 — `authorization_grant.rs` `PermissionOperation` enum.** Under U21 (`ExecuteWorkflow` variant slot reservation), gets a new variant added — additive, no deletion. Restructure ~30-50 LOC for the new variant + AAD-binding hookup per U10 `#[non_exhaustive]`. Item L12-25 cross-cut: caps already has `#[non_exhaustive]` discipline established (per V1-FROZEN-INTERFACE row 11 + the existing CapabilityPolicy sealed-discipline pattern), so adding `#[non_exhaustive]` to existing enums is doc-pattern not structural-pattern change. **Strategy A** (Wave-D same-PR).

**Item L12-20 — Otherwise unaffected.** No dead code in benten-caps from F-full. The capability + manifest-envelope + chain-validation surface is orthogonal to encryption-envelope layer.

### §2.4 `benten-id` (Atrium identity)

**Item L12-21 — `did_rotation.rs` + `device_attestation.rs` + `vc.rs` + `multi_sig.rs` + `keypair.rs` + `ucan.rs`.** All depend on `benten_crypto_suite::primitives::ed25519_dalek::*`. UNAFFECTED by F-full (encryption-only; signature/identity surface frozen). No dead code.

**Item L12-22 — `did.rs::Did` shape (referenced from registry as U15 multikey canonical).** Under U15 (Did multikey-codepoint canonical serialization + `Did::Unknown(u64, Bytes)` typed-rejection variant), the existing `Did` enum gets a new `Unknown` variant + canonical-serialization helper. RESTRUCTURE ~20-40 LOC; no deletion. **Strategy A** (Wave-A or Wave-E depending on whether it's part of envelope foundation or cross-ecosystem-adapter).

### §2.5 `benten-engine`, `benten-sync`, `benten-platform-foundation`, `benten-drop`

**Item L12-23 — `benten-drop/src/envelope_sig.rs` (DropBundle envelope signature).** UNAFFECTED by F-full envelope-confidentiality layer; the SIGNATURE envelope is orthogonal. NOTE the namespace collision: there are now THREE "envelope" surfaces in Benten:
  1. `benten-crypto-suite::aead::AeadEnvelope` (existing AEAD wire envelope)
  2. `benten-drop::envelope_sig` (DropBundle SIGNATURE envelope; UCAN-Varsig sibling)
  3. NEW under F-full: `EncryptedEnvelope` (§6.2 envelope-layer-unification)
Future F-full work should disambiguate these names — recommend `EncryptedEnvelope` for the new §6.2 shape + RENAME existing `AeadEnvelope → LegacyAeadEnvelope` during transition then DELETE at F-full Wave-A close per no-backwards-compat. **Naming-hazard note**, not deletion candidate per se.

**Item L12-24 — `benten-engine::thin_client.rs:568` Verifier usage + `benten-platform-foundation::plugin_manifest.rs:17` + `benten-sync::ucan_blobs_protocol.rs:86`.** All re-export consumers of crypto-suite primitives. UNAFFECTED by F-full. No dead code.

### §2.6 Workspace-wide

**Item L12-25 — `#[non_exhaustive]` audit at V1-FROZEN-INTERFACE.md L995-L999.** Currently named 12+ verified-missing enums in benten-engine + benten-core + benten-ivm + benten-sync that should APPLY `#[non_exhaustive]` per V1-FROZEN spec item 11. F-full U9+U10 mandate `#[non_exhaustive]` on the new `EnvelopePayload` + `BindingContext` enums; this is a NATURAL pairing with the workspace-wide application. **OPPORTUNITY:** bundle the workspace-wide `#[non_exhaustive]` sweep with F-full Wave-A as a single discipline-establishing PR. Saves the workspace ~30-60 LOC of attribute application + the V1-FROZEN-INTERFACE-DEFERRED Row D-17 partial-close. Per §3.6h pim-N-ratification-must-close-origin: when U9+U10 ratify `#[non_exhaustive]` discipline for envelope enums, the SAME wave closes the workspace-wide application. **Strategy A** (Wave-A same-PR; cross-crate scope but small per-file LOC).

---

## §3 Trimming sequencing per item

Per `feedback_orchestrator_defer_prediction_bias` (do-it-now bias) + Ben 2026-05-27 pre-public-cleanup encouragement. Four canonical strategies:

- **Strategy A (same-PR-as-replacement)** — DEFAULT for nearly all items. Smallest cumulative diff; lowest bisect-risk; cleanest reviewer experience because each PR is a single conceptual unit. Pre-public no-backwards-compat means the soft-deprecation choreography ("ship parallel; deprecate; await migration; delete") is not needed. Cite: HARD RULE 12 + CLAUDE.md baked-in #5 no-shims.

- **Strategy B (pre-flight dedicated cleanup PR before F-full PRs).** Used when the cleanup cross-cuts many F-full PRs + flushing it first removes coordination overhead. Two pre-flights recommended:
  - **Pre-flight 1: LE→BE codepoint endianness migration** (Item L12-2; ~1 wave-day). Single PR: flip all 8 `to_le_bytes()` sites to `to_be_bytes()` + bump `ENVELOPE_FORMAT_VERSION_V1 → V2` (or drop the constant per Item L12-3) + regen golden vectors. All downstream F-full PRs build on BE foundation.
  - **Pre-flight 2: X-Wing-mislabel rename audit close** (Item L12-4; reference `.addl/phase-4-meta/x-wing-to-mlkem768-x25519-rename-audit.md` for Categories A-E checklist). Single PR: rename literals + info-tag bytes + test names. Closes the mislabel BEFORE F-full amendments perpetuate it.

- **Strategy C (post-PR cleanup sweep after amendments land).** Used when implementer focus on new code dominates cleanup value. Recommend ZERO Strategy-C items for F-full because the workspace is small + agent-dispatch makes parallel cleanup cheap (per `feedback_agent_economics_prefer_thorough_cleanup`).

- **Strategy D (post-F-full janitorial pass).** Used for low-stakes tidy work with no functional gate. Two Strategy-D items:
  - Workspace-wide `_for_test` constructor audit (verify R6 R1 FP-A Bundle F1 closure remains intact post-F-full).
  - X-Wing literal cross-doc residual sweep (any stragglers in docstrings/comments that the rename audit missed).

**Per-item disposition table:**

| Item | Surface | What | Strategy | Wave |
|---|---|---|---|---|
| L12-1 | aead.rs:147-210 | AeadEnvelope restructure → EncryptedEnvelope | A | Wave-A |
| L12-2 | 8 sites | LE→BE codepoint flip | B (pre-flight 1) | Pre-A |
| L12-3 | aead.rs:60 | ENVELOPE_FORMAT_VERSION_V1 delete | A | Wave-A |
| L12-4 | cipher_suite.rs:78 + 50+ sites | X-Wing-mislabel rename | B (pre-flight 2) | Pre-A |
| L12-5 | cipher_suite.rs::WrappedKey | Restructure for HpkeMultiBase | A | Wave-C |
| L12-6 | structural_kdf.rs:120-162 | KDF info-tag generation-counter binding | A | Wave-B |
| L12-7 | sig.rs | UNAFFECTED | — | — |
| L12-8 | swap_matrix.rs | Add HPKE codepoint arms | A | Wave-A + Wave-C |
| L12-9 | tests/*.rs (14 files) | Restructure ~150-200 LOC | A | per-wave |
| L12-10 | discharge.rs | UNAFFECTED (extend pattern for new amendments) | A | per-wave |
| L12-11 | sizes.rs | UNAFFECTED (just LE→BE flip) | B (pre-flight 1) | Pre-A |
| L12-12 | redb_backend.rs:139-205 | K_principal STUB delete | A | Wave-B |
| L12-13 | redb_backend.rs | K_PRINCIPAL_DOMAIN_KEY const delete | A | Wave-B |
| L12-14 | redb_backend.rs docstrings | Doc rewrite | A | Wave-B |
| L12-15 | redb_backend.rs schema | UNAFFECTED | — | — |
| L12-16 | aead_wrap.rs | Consolidate codepoint encoding through canonical_binding | A | Wave-A |
| L12-17 | aead_wrap.rs `_for_test` | Restructure internal shape | A | Wave-A |
| L12-18 | two_cid_map.rs | DEFERRED (G-CORE-3e iroh-blobs wave) | — | — |
| L12-19 | authorization_grant.rs | Add ExecuteWorkflow variant | A | Wave-D |
| L12-20 | benten-caps misc | UNAFFECTED | — | — |
| L12-21 | benten-id sig surface | UNAFFECTED | — | — |
| L12-22 | did.rs::Did | Add Unknown variant + multikey canonical | A | Wave-A or Wave-E |
| L12-23 | drop/engine/sync consumers | Naming-hazard note only | — | — |
| L12-24 | crypto-suite re-exports | UNAFFECTED | — | — |
| L12-25 | workspace #[non_exhaustive] sweep | Bundle with U9+U10 | A | Wave-A |

**Wave decomposition** (per F-full registry §7 §8):
- **Pre-flight 1** (LE→BE) — ~1 wave-day; single PR; lands before Wave-A.
- **Pre-flight 2** (X-Wing rename) — ~1 wave-day; single PR; can land in parallel with Pre-flight 1 (no overlap surface).
- **Wave A** — envelope shape + canonical_binding + TLV + amendments U1-U3 + non_exhaustive sweep — closes Items L12-1, L12-3, L12-8(partial), L12-9(Wave-A tests), L12-16, L12-17, L12-22, L12-25.
- **Wave B** — Layer-A vault + Argon2id + XChaCha20 + U4-U5 + U20 — closes Items L12-6, L12-12, L12-13, L12-14.
- **Wave C** — Layer-C HPKE-mode-base + libcrux + multi-stanza + U17-U19 + U22 + U25 — closes Items L12-5, L12-8(rest).
- **Wave D** — Layer-D device-link + remote-permission + U21 + ExecuteWorkflow — closes Item L12-19.
- **Wave E** — cross-ecosystem adapters + DAG-CBOR + golden vectors + dudect CI + kani — opens NEW surface; doesn't close existing items (other than Item L12-22 if Did multikey is here).
- **Wave F** — audit-deliverable docs (THREAT-MODEL.md + KEY-LIFECYCLE.md + etc.) — opens NEW surface.

---

## §4 Dependency dead-codes

Per `Cargo.toml` survey of `benten-crypto-suite/Cargo.toml` (the ONLY direct crypto-primitive dep site per `crypto-agility-contract:6`). Tool note: `cargo-machete` + `cargo-udeps` + `cargo-shear` not installed in environment ([cargo-machete](https://crates.io/crates/cargo-machete) is text-based + workspace-incomplete; [cargo-udeps](https://github.com/est31/cargo-udeps) is compiler-output-based but needs nightly; [cargo-shear](https://crates.io/crates/cargo-shear) is the newer entrant). Analysis below is grep-based + Cargo.toml-read; recommend `cargo install cargo-shear && cargo shear --fix` as a pre-flight verification.

### §4.1 Currently-pinned crypto deps and F-full disposition

| Crate | Version | F-full disposition | Notes |
|---|---|---|---|
| `ed25519-dalek` | workspace (2.x via iroh 1.0.0-rc.0 → 3.0.0-pre.7) | KEEP | Sig surface unchanged |
| `ml-dsa` | `^0.1` default-features=false | KEEP | Sig surface unchanged |
| `slh-dsa` | `0.2.0-rc.5` | KEEP | NF-1 reserved arm |
| `x25519-dalek` | `2` default-features=false | KEEP | Layer-C HPKE needs it |
| `ml-kem` | `0.2` default-features=false | **SWAP per U31 → `libcrux-ml-kem`** with `check-secret-independence` feature. ~50 LOC migration per L4. Ben call per Q1 (consolidator HIGH recommend libcrux). | **DEAD if Q1 ratifies libcrux** |
| `chacha20poly1305` | `0.10` | KEEP + EXTEND with XChaCha20-Poly1305 codepoint per U32; same crate exposes both. | No new dep. |
| `hkdf` | `0.12` | KEEP | Used for KDF. |
| `sha2` / `sha3` | `0.10` (+ `0.11` transitive via slh-dsa) | KEEP | Hash seam. |
| `blake3` | workspace | KEEP | CID hash. |
| `getrandom` | `0.4` default-features=false sys_rng | KEEP | OS entropy bridge. |

**Net dep changes from F-full:**
- **ADD:** `libcrux-ml-kem` (if Q1 ratifies; HIGH consolidator recommendation) + feature `check-secret-independence`. **Deletes** `ml-kem 0.2` dep.
- **ADD:** `rust-hpke` (McMillion variant per L5 Verification-Theatre — NOT Cryspen's `hpke-rs`) for Layer-C HPKE-mode-base.
- **ADD:** `argon2` for Layer-A vault Argon2id KDF per U6/L4-§2.2.2 tiering.
- **ADD:** likely `secrecy` + reaffirm `zeroize` for U34 opaque-handle pattern.
- **ADD:** if U30 DAG-CBOR LOAD-BEARING (Ben call): nothing new — `serde_ipld_dagcbor` already workspace-dep.
- **POTENTIALLY DELETE:** `ml-kem 0.2` if libcrux ratifies (clean swap; ~50 LOC).
- **NO POTENTIAL DELETES** from existing deps absent F-full — every current crypto-suite dep is in active use per grep.

### §4.2 `deny.toml` updates needed

Per Ben's pre-public stance + the existing 4 RUSTSEC-2026-0119/0120 ignore-then-close pattern at deny.toml:
- **ADD** `libcrux-ml-kem` to the allow-list under `[bans]` (it's a NEW upstream crate not yet in the dep tree; cargo-deny will fire on first add).
- **REMOVE** `ml-kem 0.2` from any deny.toml allow-rows once swap lands.
- **Workspace-wide** Strategy: per §3.5g (cargo-audit + deny.toml + clippy.toml mirror discipline), every dep add/remove must atomically update deny.toml + verify `cargo deny check` passes.

### §4.3 [dev-dependencies] inventory

- `benten-id` dev-dep on `crates/benten-crypto-suite/Cargo.toml` for the discharge::Issue835Discharge doctest. UNAFFECTED by F-full.

### §4.4 No dead deps detected at workspace level

Per `cargo deny check advisories` + manual grep across `Cargo.toml` files. The workspace is dep-clean (no orphan deps). The discipline of "the ONLY crypto-primitive call site per crypto-agility-contract:6" keeps the dep surface narrow + auditable.

---

## §5 Documentation dead-code

Per pim-1 (`feedback_post_fix_doc_coupling_preflight`) + §3.5h MANDATORY-PRE-MERGE: every doc cross-reference touched by F-full needs atomic update. Surface inventory:

### §5.1 `docs/INVARIANT-COVERAGE.md` (402 LOC)

- **L33 invariants table** — Inv-15 currently REGISTERED + partial-enforcement. F-full Wave-F MINTS Inv-16 + Inv-17 + Inv-18 per registry §4. ADD 3 new table rows at v1-beta. No dead rows.
- **L14 narrative** — references Inv-15 G-CORE-PQ-WIRE-1 enforcement-completion path; UPDATE to "Inv-15/16/17/18 enforcement-completion path lands at G-CORE-PQ-WIRE waves A-F per F-full ADDL".
- **L53-L121 Inv-4 + Inv-7 honest-disclosure section** — UNAFFECTED.

### §5.2 `docs/SECURITY-POSTURE.md` (2,836 LOC — heaviest doc surface)

- **L26-L46 substrate-only callout** — RESTRUCTURE: F-full Wave-A/B/C/D land Compromise #32-#44 mints, so the "Compromise #26 + #30 + #31" listing extends to "+ #32..#44". ~40 LOC restructure.
- **K_principal-stand-in callout** (referenced from redb_backend.rs:148) — DELETE entire callout post-Layer-A vault landing. ~10-20 LOC delete + replace with Layer-A vault narrative + reference to KEY-LIFECYCLE.md (new per L5).
- **G-CORE-3e key-derivation section** — DELETE the "Confidentiality limit at this wave" paragraph; replace with Layer-A vault description. ~30-50 LOC restructure.
- **Compromise #26 + #30 + #31** — UNAFFECTED rows but #31 receives an EXTENSION per registry §3 (per L5-C9 + L9-A3 unified recipient-key-retention-window clarification). UPDATE ~10 LOC.

### §5.3 `docs/V1-FROZEN-INTERFACE.md` (1,876 LOC)

- **Item 4 (Wire envelope structure)** — RESTRUCTURE for §6.2 `EncryptedEnvelope` shape replacing `AeadEnvelope` shape. ~30-50 LOC update.
- **Item 6 (codepoint table)** — EXTEND with new codepoints per U11 (escape codepoint 0xFFFF + experimental range 0xFE00..0xFFFE) + U13 (MLS/CGKA/Bird-of-Prey codepoint brackets) + U17 (HpkeMultiBase 0x6301) + U22 (Sealed-Sender 0x6510). ~40-80 LOC additive. Cross-references to NEW doc `docs/CRYPTO-CODEPOINTS.md` per U8.
- **Item 6 CodepointLifecycle amendment** per U16 — ADD new sub-section ~20-30 LOC.
- **L877 packages/engine TS surface table** — UNAFFECTED at v1-beta (TS surface doesn't see envelope-internal structure per U33 NAPI opaque-handle).
- **L992-L999 #[non_exhaustive] table** — CLOSE 12+ "APPLY" rows atomically with F-full Wave-A per Item L12-25.

### §5.4 `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (1,306 LOC)

- **NEW Row D-SS-1** — mint per U22 Sealed-Sender deferral (implementation post-v1-beta; slot reserved). ~15-20 LOC.
- **NEW Row D-PAD-1** — mint per U24 padding-bucket post-measurement refinement. ~10-15 LOC.
- **NEW Row D-COVER-1** — mint per U26 cover-traffic NAMED-DEFERRED with revisit-trigger "shaped-relay transport extension lands". ~10-15 LOC.
- **NEW Row D-DID-ROT-1** — mint per U27 DID-rotation discipline NAMED-DEFERRED to post-v1 UX wave. ~10-15 LOC.
- **Row D-21 — `crates/benten-crypto-suite/INTERNALS.md` authorship** — EXTEND for F-full surface coverage. ~30-50 LOC.
- **Row D-25 — V1-BETA-BREAKING-CHANGES.md ledger completion** — F-full lands ~28 amendments worth of breaking changes; this row gets BIG content. ~80-150 LOC.
- **Row D-26 — G-CORE-PQ-WIRE wave: PQ-hybrid app-layer wire-in** — RESTRUCTURE to absorb F-full Wave-A/B/C/D/E/F wave decomposition.

### §5.5 `docs/future/phase-4-backlog.md` (1,696 LOC)

- **L172-L177 G-CORE-3e named carry-overs** — CLOSE rows:
  - "K_principal-per-DID secret-material backend" — CLOSED by F-full Wave-B (Layer-A vault).
  - "iroh-blobs FsStore bytes-serving wire-up" — UNAFFECTED (orthogonal wave).
  - "G-CORE-3e end-to-end composition pin" — UNAFFECTED (orthogonal pin).
  - "UcanEnvelope::unresolved_peer field removal P-III decision" — UNAFFECTED (orthogonal P-III at G-CORE-9). See §6 opportunity (d).
- ~30-50 LOC delete/restructure.

### §5.6 `docs/V1-WIRE-FORMAT-INVENTORY.md` (472 LOC)

- **L76 codepoint table** — EXTEND with new F-full codepoints per Item §5.3 above. ~30-40 LOC additive.

### §5.7 NEW docs minted by F-full per L5 + L7 + L8

- `docs/CRYPTO-CODEPOINTS.md` (new; ~300-500 LOC) per U8 registry discipline.
- `docs/THREAT-MODEL.md` (new; ~400-600 LOC per L5 §1 + §2 25-row matrix) per U40.
- `docs/KEY-LIFECYCLE.md` (new; ~300-400 LOC) per L5 §5 audit-deliverable.
- `docs/CRYPTO-PARAMETERS.md` (new; ~150-250 LOC) per L5.
- `docs/AUDIT-SCOPE-STATEMENT.md` (new; ~150-200 LOC) per L5.

Total net-new doc LOC: ~1,300-1,950. Total dead-doc LOC: ~150-250. Net change: +1,050-1,700 doc LOC.

### §5.8 README + per-crate INTERNALS.md + crate docstrings

- `crates/benten-crypto-suite/INTERNALS.md` — RESTRUCTURE the L49 codepoint table description for new entries + add envelope-layer section. ~30-50 LOC. Per Row D-21 deferred-row.
- `crates/benten-crypto-suite/src/lib.rs` docstring (L1-L29 module-level) — UPDATE the "v1-beta defaults" section for U7 BE + U14 aad_version. ~10-20 LOC.
- `crates/benten-graph/src/redb_backend.rs` docstrings — DELETE K_principal-stand-in paragraphs per Item L12-14. ~25 LOC delete.
- `crates/benten-graph/src/aead_wrap.rs` module docstring — RESTRUCTURE for canonical_binding consolidation per Item L12-16. ~20 LOC restructure.

---

## §6 Pre-public cleanup opportunities BEYOND F-full

Per Ben 2026-05-27 signal: "this project isn't public/being used by anyone yet so there's not really any backward-compatibility concerns though we could assess what becomes dead/trimmable." Survey:

### §6.1 `_for_test` constructor sweep audit (opportunity (a))

R6 R1 FP-A Bundle F1.b closed the workspace `_for_test` constructor cfg-gating sweep (Row D-22 CLOSED 2026-05-24). Pre-public re-audit recommended: any `_for_test` constructor whose CALL SITES could route through a real constructor should be DELETED outright pre-v1-beta. Verified at HEAD:
- `crates/benten-crypto-suite/src/aead.rs:108` `from_bytes_for_test` on `AeadKeyMaterial` — used by `redb_backend.rs:183` (the K_principal STUB; Item L12-12). When K_principal STUB deletes (Wave-B), this `_for_test` use disappears. CHECK: are there OTHER callers in tests? If only-tests, KEEP. If any production caller remains, DELETE the production caller too.
- `crates/benten-crypto-suite/src/codepoint.rs:88` `SigCodepoint::from_raw_for_test` + L161 `HashCodepoint::from_raw_for_test` + L245 `CipherSuiteCodepoint::from_raw_for_test` + L97 `CipherSuiteCodepoint::reserved_unimplemented_for_test` — all test-only adversarial constructors. KEEP (legitimate test-only use).
- `crates/benten-graph/src/aead_wrap.rs:184` `EncryptedNode::with_aad_plaintext_cid_for_test` — adversarial rebinding-attack test. KEEP.
- `crates/benten-crypto-suite/src/structural_kdf.rs:67` per-test K_principal byte vectors — DELETE once Layer-A vault lands; tests should construct via vault.

**Estimated cleanup**: 1-3 `_for_test` constructors deletable pre-v1-beta; ~15-30 LOC. **Strategy A** (atomic with F-full Wave-B).

### §6.2 `0.x` / `-rc` / `-pre` dep version pin audit (opportunity (b))

Pinned at HEAD per `Cargo.toml`:
- `ml-dsa ^0.1` — verify latest release pre-v1-beta tag.
- `slh-dsa 0.2.0-rc.5` — verify upgrade to stable 0.2 once published.
- `ed25519-dalek 3.0.0-pre.7` (transitive via iroh 1.0.0-rc.0) — verify upgrade to stable 3.0.0 once iroh stabilizes.
- `ml-kem 0.2` → potentially SWAP to libcrux-ml-kem per Q1.

cargo-deny doesn't catch "pin is on a release-candidate"; needs manual audit. **Strategy B** (pre-v1-beta-tag cleanup sweep PR; ~half wave-day audit + dep-bump-if-available).

### §6.3 X-Wing-mislabel audit close (opportunity (c))

The brief references `.addl/phase-4-meta/x-wing-to-mlkem768-x25519-rename-audit.md` with Categories A-E. This file is NOT on this branch + NOT visible to this lens. Recommend: orchestrator confirm before F-full Wave-A dispatch that the rename-audit Categories A-E are fully enumerated + the F-full implementer briefs reference the audit doc explicitly. Otherwise F-full Wave-A perpetuates the mislabel into ~28 new amendments. **Strategy B (pre-flight 2 per §3)**.

### §6.4 `UcanEnvelope::unresolved_peer` field P-III decision (opportunity (d))

Per phase-4-backlog.md L177: "Post-G-CORE-3e production wire-up where RotationLog replaces the in-grant sentinel, the field has no production purpose. Either remove from UcanEnvelope wire shape before G-CORE-9 v1-public-interface freeze OR keep + document why it stays." Per Ben's no-backwards-compat stance: **DELETE** is the clean call. No production code uses it; the typed-sentinel testing argument is weak (tests can construct via Option::None at field level if shape kept; tests can construct sibling typed enum if removed). **Strategy B** (separate small-PR; ~30-50 LOC including test fixtures; ~half wave-day).

### §6.5 `aead_wrap.rs` consolidation through `canonical_binding()` (opportunity (e))

Per Item L12-16: the 941-LOC `aead_wrap.rs` has its own codepoint-encoding logic (L596+L602 `to_le_bytes()`) which duplicates what `canonical_binding()` per U1+U14 will own. Consolidation through the canonical helper saves ~80-120 LOC + closes a cite-drift hazard. Already covered as Strategy A under Item L12-16; surface here as a "this is also a pre-public cleanup win" framing.

### §6.6 `benten-caps/testing` feature pass-through audit (opportunity (f))

V1-FROZEN-INTERFACE.md L751-L753 documents the workspace test-double Sealed-trait pass-through chain: `benten-engine test-helpers + benten-eval testing → benten-caps/testing → __sealed_for_workspace_tests`. The chain works but has 3 indirection layers. Pre-public simplification opportunity: collapse to single `benten-caps/testing` direct enable from each test-double crate. ~30-50 LOC simplification across `Cargo.toml` files + `__sealed_for_workspace_tests` re-export trimming. LOW priority; **Strategy D** (post-F-full janitorial).

### §6.7 `cargo-machete` / `cargo-shear` + `cargo-udeps` wire-up (opportunity (g))

Workspace has no dead-dep detection tool wired into pre-push or CI beyond cargo-deny's advisory/license focus. Recommend `cargo-shear` (newer; workspace-aware; integrates with `cargo deny check`):
- Add `cargo install cargo-shear` to CONTRIBUTING.md tools-list.
- Add `cargo shear` to §3.5o pre-push checklist (~3-5s additional pre-push cost).
- Add `cargo shear` step to `.github/workflows/supply-chain.yml`.

Per [cargo-shear crates.io](https://crates.io/crates/cargo-shear), workspace-wide support + handles misplaced deps + unlinked source files. Strict superset over [cargo-machete](https://bouvier.cc/tech/cargo-machete/) for Benten's workspace pattern. [cargo-udeps](https://github.com/est31/cargo-udeps) needs nightly so less attractive for stable-only Benten pre-push. **Strategy B** (pre-flight before F-full Wave-A; ~1-2 hours wire-up; closes a permanent observability gap).

### §6.8 Phase-1/2/3 prototype-code sweep (NEGATIVE finding)

Grep-verified at HEAD: NO surviving Phase-1 prototype API marked DEPRECATED beyond the K_principal STUB. The workspace has been disciplined about NOT carrying deprecated artifacts (per CLAUDE.md baked-in #5 no-shims). The only "dead-code-by-virtue-of-supersession" is the K_principal STUB family (Items L12-12, L12-13, L12-14). This is structurally cleaner than typical 5-week-old projects.

### §6.9 Crate-merge/split assessment (NEGATIVE finding)

Workspace has 14 crates per `ls crates/`. Each carries a clear responsibility: foundation/primitives/crypto/storage/identity/engine/eval/sync/etc. NO crate appears to be a candidate for merging into a sibling. NO crate is so large it should split. The `benten-crypto-suite` (5,634 LOC) is at the upper bound of "single crate" but the `crypto-agility-contract:6` ONE-CALL-SITE rule structurally argues against splitting (every split adds a primitive-call-site to audit). NO action recommended on crate boundaries.

---

## §7 Recommended cleanup-PR-bundle shape

**Recommendation: hybrid (Strategy A default + 2 Strategy-B pre-flight PRs + minimal Strategy-D close-out).**

NOT a dedicated "PRE-V1-BETA-CLEANUP-PASS" wave with N PRs. Reasons:
- The catalogued dead-code surface is small (~150-300 LOC clean deletion across ~6 PRs of F-full work). It doesn't justify the orchestration overhead of a dedicated cleanup wave.
- Strategy A (same-PR-as-replacement) is structurally cleanest pre-public; the soft-deprecation choreography that would justify a dedicated cleanup wave doesn't apply.
- The 2 recommended Strategy-B pre-flight PRs (LE→BE migration; X-Wing rename) are small + independent + flushing them first reduces F-full PR coordination overhead.

**Recommended sequence:**

1. **Pre-flight PR-A: LE→BE codepoint endianness migration.** ~1 wave-day; small (~50-100 LOC + golden vector regen); single agent; tight test pin (`tf3a_*` family golden vectors flip). Lands before any F-full PR.
2. **Pre-flight PR-B: X-Wing-mislabel rename audit close.** ~1 wave-day; small (~50-100 LOC + ~30 docstring updates + test rename); single agent; tight test pin. Can land parallel to Pre-flight PR-A.
3. **Pre-flight PR-C (optional): `cargo-shear` wire-up + UcanEnvelope::unresolved_peer delete.** ~1 wave-day; bundles two small pre-public-cleanup wins.
4. **F-full Wave-A through Wave-F** per registry §7 §8 wave decomposition; each wave closes its catalogued dead-code items per §3 table.
5. **Post-F-full janitorial sweep** (Strategy D): `_for_test` constructor audit + workspace-wide cite-drift verify + X-Wing residual sweep. ~0.5 wave-day single agent at v1-beta tag close.

**Estimated cumulative cleanup cost (BEYOND the F-full implementation work):** ~3-4 wave-days = ~0.6-0.8 calendar-weeks. Folded into the L4 estimate of ~65-72 wave-days = ~13-15 calendar-weeks for the full F-full ADDL; net cleanup overhead is ~5-6% of the F-full budget. ACCEPTABLE.

**Alternative considered + rejected: defer all cleanup to Phase-4-Meta-Composing post-tag.** REJECTED per Ben pre-public-cleanup encouragement + HARD RULE 12 no-defer; the cleanup work is small + cleanly bundles with F-full + delaying just compounds cite-drift hazard.

---

## §8 Self-assessment + confidence

### What I did + how I worked

1. Tree-state pre-flight on worktree branch `phase-4-meta-core/option-f-plus-lens-l12-dead-code-identification` (clean against `2172cb6d`).
2. Fetched F-full registry from `fbdfeb16` (`option-f-plus-9-eyes-consolidated-registry.md`; 939 LOC; 28 amendments U1-U40 + 13 Compromises #32-#44 + 3 invariants Inv-16/17/18 + 5 disagreements Q1-Q5) into `/tmp/`; full-read in 3 chunked offsets.
3. Surveyed 14 workspace crates via `ls crates/` + `wc -l` for key crates (benten-crypto-suite 5,634 LOC; benten-graph 9,522 LOC; benten-caps 7,480 LOC).
4. Read full `Cargo.toml` of `benten-crypto-suite` (the dep-surface anchor per crypto-agility-contract:6) + scanned per-package Cargo.tomls via grep for `benten-crypto-suite` consumers.
5. Read `aead.rs` (504 LOC) head + structural grep for `AeadEnvelope` + `to_le_bytes` + `ENVELOPE_FORMAT_VERSION_V1` + `format_version` field.
6. Grep-surveyed K_principal handling across the workspace → identified the STUB at `redb_backend.rs:139-205`.
7. Grep-surveyed LE codepoint encoding → identified 8 distinct sites across 6 files.
8. Surveyed `cipher_suite.rs::1-744` for X-Wing-mislabel surface (literal `X_WING_HKDF_INFO_V1 = b"x-wing-v1-benten-0x647a"` at L78; ~50+ docstring + comment references).
9. Surveyed `docs/V1-FROZEN-INTERFACE.md` (1,876 LOC) + `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (1,306 LOC) + `docs/SECURITY-POSTURE.md` (2,836 LOC) + `docs/INVARIANT-COVERAGE.md` (402 LOC) + `docs/future/phase-4-backlog.md` (1,696 LOC) + `docs/V1-WIRE-FORMAT-INVENTORY.md` (472 LOC) for dead-row + restructure surface.
10. WebSearch for [cargo-machete](https://crates.io/crates/cargo-machete) / [cargo-shear](https://crates.io/crates/cargo-shear) / [cargo-udeps](https://github.com/est31/cargo-udeps) workspace dep-detection tools + [Rust dead-code attribute](https://www.w3tutorials.net/blog/how-do-you-disable-dead-code-warnings-at-the-crate-level-in-rust/) discipline patterns. Pre-public-no-backwards-compat means "delete directly" is acceptable per the Rust diagnostic-attribute conventions.
11. Built §2 catalog (25 items L12-1 through L12-25) + §3 per-item disposition table + §4 dep-graph analysis + §5 doc dead-row sweep + §6 cleanup-beyond-F-full opportunities (7 items) + §7 PR-bundle recommendation.

### Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §1 Executive summary | **HIGH** | Top-line claim ("nothing becomes orphan dead code" + ~150-300 LOC clean deletion + 18-of-24 items Strategy-A) is grep-verified at HEAD. |
| §2 Per-crate catalog | **HIGH** | Every item carries file:line citations from grep. Items L12-1 / L12-2 / L12-4 / L12-12 / L12-16 are read-verified. |
| §3 Trimming sequencing | **HIGH** on Strategy-A default; **MED-HIGH** on wave assignments (depends on F-full §8 wave decomposition holding) | Per-item disposition table cross-references registry §7 §8. |
| §4 Dependency dead-codes | **MED-HIGH** | Grep + Cargo.toml-read based; `cargo-machete` not available to verify. The libcrux swap is the biggest call; depends on Q1 ratification. |
| §5 Documentation dead-code | **MED-HIGH** | 6 doc artifacts surveyed; row-by-row impact estimated; specific section restructures should be verified by F-full Wave-F implementer. |
| §6 Pre-public cleanup opportunities | **MED** | Judgment-heavy; 7 opportunities identified; Ben ratify-or-amend recommended; opportunities (a) + (g) are highest confidence. |
| §7 PR-bundle recommendation | **HIGH** on Strategy-A default + 2 pre-flight + minimal close-out shape; **MED** on cost estimate (~3-4 wave-days incremental). |

### What I could be wrong about

1. **The "nothing becomes orphan dead code" claim is grep-based.** A `cargo-shear` or `cargo-udeps` run could surface an unused dep my grep missed. Recommend wiring `cargo-shear` per opportunity (g) + re-running this lens audit at F-full Wave-A close.
2. **Item L12-4 X-Wing-mislabel scope.** I could not access `.addl/phase-4-meta/x-wing-to-mlkem768-x25519-rename-audit.md` (NOT on this branch). My ~50 site estimate is from cipher_suite.rs grep; the actual audit doc may enumerate substantially more.
3. **Item L12-25 #[non_exhaustive] sweep coupling.** Bundling the workspace-wide sweep into F-full Wave-A is a judgment call. Alternative: keep the sweep as a separate Strategy-B pre-flight PR. Either is defensible.
4. **Opportunity (b) version-pin audit.** I checked Cargo.toml pinned versions but not crates.io for latest releases. The ml-dsa / slh-dsa / ed25519-dalek versions may already have stable successors; needs network-verify.
5. **Item L12-12 K_principal STUB callers.** I counted ~8 grep-matched call sites (redb_backend.rs L1632/1673/1686/2464/2583/2623/2693/2745/2768) but didn't fully trace every transitive caller; the actual swap surface may extend to test fixtures + integration tests. Estimate could be ±30%.
6. **Opportunity (d) UcanEnvelope::unresolved_peer.** Ben's pre-public-cleanup stance argues DELETE; the existing P-III decision-point at G-CORE-9 freeze may have additional context I'm not seeing. Should defer to the P-III decision the row already names.
7. **§7 cost estimate (~3-4 wave-days incremental).** This is rough; agent dispatch could compress to ~1.5-2 wave-days; sequential could expand to ~5-6. ±50%.

### Lower-confidence areas (honest disclosure)

- I did NOT run `cargo build --workspace --all-targets` to verify that my grep-identified "becomes dead" items would actually fail compile + reveal additional surface. A compile-check post-K_principal-STUB-deletion would surface the actual transitive call-site count.
- I did NOT enumerate every test file in every crate; my "~150-200 LOC test restructure" estimate is from sampled tf-* files in benten-crypto-suite/tests/.
- I did NOT check `bindings-napi` or `packages/engine` (the TS / NAPI surface) for downstream consumers of `AeadEnvelope`. The U33 single-source NAPI rule-mirror discipline argues there shouldn't be TS-side restructure beyond regenerating ts.d files, but worth verifying.
- I did NOT independently verify the L7 cross-ecosystem-identifier-as-content discipline references (§3.5s + U29) at HEAD; propagated registry's framing.

### What this lens does NOT cover

- Per brief task scope: dead-code identification + pre-public cleanup. NOT a re-evaluation of F-full design soundness (L1-L9 + C1-C5 are authoritative).
- NOT an enumeration of NEW code F-full ADDS — only what becomes dead/restructured. Net-new LOC estimates per L4 §7 (~2,730 production + ~4,200 test).
- NOT a review of pim-N codification opportunities (would belong in a separate process-discipline lens).
- NOT a runtime-perf-impact analysis (out of scope for code-migration lens).

---

## §9 Citations

### §9.1 Inputs (frozen SHAs)

- F-full consolidated registry — `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` — `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md`
- C1 elegant-shape — `3f5a4351`
- C2 cross-amendment composability — `9c548e5f`
- C3 fresh-eyes 10th-eye — `79c99aa5`
- C4 process-discipline (origin of this lens) — `6ea9718a`
- C5 formal-methods — `8e374a9d`
- Origin lens L1-L9 SHAs per registry §10.1 (not re-walked by this lens; propagated)

### §9.2 Benten internal references (verified at HEAD)

- `crates/benten-crypto-suite/src/aead.rs:60,147,165,189,199,210,244-245,277-278,320,348,460,504` — AeadEnvelope wire format + LE codepoint + format_version
- `crates/benten-crypto-suite/src/cipher_suite.rs:78,162-227,744` — X_WING_HKDF_INFO_V1 + WrappedKey + X-Wing-mislabel surface
- `crates/benten-crypto-suite/src/codepoint.rs:43,134,191,199,205,209,214,228,274` — codepoint table + reserved arms
- `crates/benten-crypto-suite/src/structural_kdf.rs:67,120,152,162,217,283` — KDF info-tag + per-test K_principal constructors
- `crates/benten-crypto-suite/src/swap_matrix.rs:1539-1550,2024` — strip-resistance AAD LE encoding
- `crates/benten-crypto-suite/src/sig.rs:552`, `sizes.rs:183,280`, `varsig.rs:47,107,200`, `discharge.rs:111`, `error.rs:106`, `hash.rs:81`, `lib.rs:153`, `primitives.rs:48`, `boundary.rs:274` — UNAFFECTED surfaces
- `crates/benten-crypto-suite/Cargo.toml` — dep pins
- `crates/benten-graph/src/redb_backend.rs:126,127,139-205,795,821,1632,1673,1686,2464,2583,2623,2693,2745,2768,3170` — K_principal STUB + schema version
- `crates/benten-graph/src/aead_wrap.rs:1,42-46,176,184,229-465,500,552,596-602,816,941` — wrap layer + LE encoding + with_aad_plaintext_cid_for_test
- `crates/benten-graph/src/two_cid_map.rs:172` — TwoCidStore (G-CORE-3e carry)
- `crates/benten-graph/src/indexes.rs:111-113` — BE encoded index lengths (not dead)
- `crates/benten-caps/src/authorization_grant.rs:953` — PermissionOperation enum (Wave-D restructure)
- `crates/benten-caps/src/policy.rs` — Sealed-trait pattern (V1-FROZEN row 6)
- `docs/V1-FROZEN-INTERFACE.md:307-313,396,414,441,528,548-707,738-799,879,887,992-999` — wire envelope item 4 + codepoint item 6 + non_exhaustive table
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md:22,168,217,252,271,304,346,363,398,427,443,458,616,645,670,757,781,790,811,830,852,898,916,953,990,1108,1119,1148,1215` — D-row index
- `docs/SECURITY-POSTURE.md:1-50,2836` — substrate-only callout + K_principal stand-in
- `docs/INVARIANT-COVERAGE.md:1-118,402` — invariants table + Inv-4/Inv-7 honest-disclosure
- `docs/future/phase-4-backlog.md:138,172-177,224,822,1002,1025,1027,1043,1051,1053,1143,1151,1159,1696` — G-CORE-3e named carry-overs + freeze plan
- `docs/V1-WIRE-FORMAT-INVENTORY.md:76,472` — codepoint table
- `crates/benten-crypto-suite/INTERNALS.md:49` — codepoint table description (Row D-21)
- `deny.toml` (head) — supply-chain policy

### §9.3 Disciplines (Benten memory)

- CLAUDE.md baked-in #5 (crypto-agility codepoint-dispatch; no-shims)
- CLAUDE.md baked-in #15 (v1-beta interface freeze)
- HARD RULE 12 clauses (a/b/c) — `feedback_no_defer_HARD_RULE`
- `feedback_orchestrator_defer_prediction_bias` (do-it-now bias)
- `feedback_agent_economics_prefer_thorough_cleanup` (cleanup-not-shortcut)
- `feedback_post_fix_doc_coupling_preflight` (pim-1; §3.5b)
- `feedback_pim_cross_language_rule_mirror` (§3.5g; load-bearing for U33)
- `feedback_pim_n_ratification_must_close_origin` (§3.6h; load-bearing for Item L12-25)
- `feedback_review_finding_ground_truth_verify` (§3.5n; load-bearing for §2 grep-verification)
- `.addl/dispatch-conventions.md` §3.5 family (pre-push checks; load-bearing for §3 Strategy assignments)
- `.addl/dispatch-conventions.md` §3.5o (CI disk-OOM; load-bearing for opportunity (g) cargo-shear pre-push wire-up)

### §9.4 External references (this lens)

- Sources used in §6.7 + §6.2 dep-detection analysis:
  - [cargo-machete on crates.io](https://crates.io/crates/cargo-machete)
  - [cargo-shear on crates.io](https://crates.io/crates/cargo-shear)
  - [cargo-udeps on GitHub](https://github.com/est31/cargo-udeps)
  - [cargo-machete intro by bouvier.cc](https://bouvier.cc/tech/cargo-machete/)
  - [Rust dead-code attribute guide on w3tutorials](https://www.w3tutorials.net/blog/how-do-you-disable-dead-code-warnings-at-the-crate-level-in-rust/)
  - [The Rust Reference — diagnostic attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html)
  - [Rust By Example — unused / dead_code](https://doc.rust-lang.org/rust-by-example/attribute/unused.html)

### §9.5 What this lens does NOT cite

- Original 9 lens reviews L1-L9 — NOT independently re-walked; propagated via registry §10.
- `.addl/phase-4-meta/x-wing-to-mlkem768-x25519-rename-audit.md` — referenced in brief but NOT present on this branch; opportunity (c) flags this for orchestrator follow-up.
- `bindings-napi` / `packages/engine` TS-side dead-code — out of grep scope; opportunity-area for follow-up TS-mirror lens.
