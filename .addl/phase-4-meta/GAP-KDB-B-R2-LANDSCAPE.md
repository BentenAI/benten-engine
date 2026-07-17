# GAP-KDB Shape-B mini-ADDL — R2 Test-Landscape

Ref 37b6f6ca · 5 dims → 76 families · 2 flagships · no coverage holes.

## Flagships
- **F-RK-2 GAP-KDB active-substitution regression — resolve_kem CID 2nd-preimage fail-closed (FLAGSHIP)** (benten-id) — resolve_kem(victim_did, attacker_doc_with_attacker_kem) must typed-reject on CID mismatch; removing the CID 2nd-preimage check makes resolve_kem return the attacker's KEM key (active substitution succeeds) and the expect_err flips to Ok — the flagship regression fails on revert.
- **gapkdb-active-substitution-fail-closed (FLAGSHIP)** (benten-drop) — Revert to the two-independent-param seal (accept a raw recipient_pub alongside audience_did): a substituted recipient KEM key NOT committed by the audience DID's key-set doc seals successfully and the attacker holding the substituted secret opens the envelope — the single-recipient expect_err on the
- **UCAN-walk SILENT-PQ-STRIP reject — a did:benten-issuer token with only the Ed25519 half (or PQ-stripped/zeroed/garbage-PQ/classical-codepoint) MUST fail closed** (benten-id) — a chain whose issuer is a did:benten but whose signature is (a) a bare 64-byte Ed25519 sig, (b) a composite with the PQ half removed (HybridSignature::without_pq_half_for_test), (c) a composite with a valid Ed25519 half + forged/zeroed ML-DSA half, or (d) a classical-codepoint 0x0002 sig — each reje
- **inv23-active-substitution-real-firing-enforcement** (benten-drop) — Build audience DID committing key-set doc D_honest(kem=K_honest); attempt RecipientBinding::resolve(audience_did, D_attacker{kem=K_attacker}) — MUST return Err (committed-CID != recomputed-CID, 2nd-preimage) so no seal to K_attacker is constructible; a downgrade attempt to the old (recipient_pub,aud

---

## Synthesis (catalog + coverage matrix + R3 slicing + freeze-gating)

# R2 TEST-LANDSCAPE — GAP-KDB Shape-B identity-model mini-ADDL

## 0. Headline

76 discovery families across 6 lenses consolidate to **66 canonical families** (10 cross-dimension merges — the `did-codec-injectivity`, `keyset-doc-resolvers`, and `authority-hybrid-migration` lenses independently re-discovered the same C6 multicodec-peek dispatch, C7 DoS cap, and did:benten method-id golden layout). **No hard coverage holes** — every ratified freeze-surface obligation (the 9 items + Fork A + C1–C9 + Inv-23 + #67) has ≥1 grounded family, and the 9 gap-fills close the real seams the per-dimension panel missed (cross-seam authority↔confidentiality key-equality, Layer-C sender-origin-auth, KeySetDocument second-untrusted-input DoS, no-hardcoded-sizes audit, did:benten-as-audience). Two **soft items** must be nailed before R3 red-phase writing (§5): an explicit `dev`-field-present reject arm, and the **C8 disagreement semantics** (plan-acknowledged under-spec).

**The spine of the whole landscape is two flagships:**

- **FLAGSHIP-1 — GAP-KDB active substitution — enforced at THREE layers that go red→green together:** resolver (`RK-2` resolve_kem CID 2nd-preimage), seal (`DROP-1` seal-to-substituted-key fails closed across 0x6510/0x6520/0x6610), invariant (`DROP-7` Inv-23 production-driving firing). Made *unconstructible* by `DROP-2/3/4` (sole-constructor, no legacy raw-seal entry, roster-from-one-slice). A sender handed a KEM key not committed by the audience DID must fail closed; an attacker cannot downgrade a did:benten recipient back to the un-cross-checked path.
- **FLAGSHIP-2 — Fork-A silent-PQ-strip (`AUTH-3`):** a did:benten-issuer token carrying only the Ed25519 half (or a zeroed/forged ML-DSA half, or a classical codepoint) must REJECT on the authority path. The one-helper grep-audit (`AUTH-7`) is the completeness net guaranteeing no un-migrated verify site re-opens it.

---

## 1. Consolidated Family Catalog (deduped, grouped by crate)

Legend: **[A]** freezes-permanent-bytes · **[B]** security-critical · **[C]** important · ★ flagship.

### benten-id — did:benten codec (7)
| ID | Family | Gate | Merges |
|----|--------|------|--------|
| DID-1 | did:benten method-id **golden byte-layout** pin `[0x1211‖mldsa1952]‖[0xed‖ed25519_32]‖[CIDv1 0x01,0x71,0x1e,0x20‖blake3_32]`, no framing byte (C1) | **A** | golden-hex-layout-pin + F-RS-5 |
| DID-2 | encode↔decode round-trip byte-identity (incl. keyset-CID accessor) | **A** | — |
| DID-3 | exact-consume injectivity **reject matrix**: trailing-byte / component-truncation / CIDv1-prefix arms (C1) | **B** | trailing-byte + component-length-truncation + keyset-cid-prefix + F-RS-4 |
| DID-5 | did:key degenerate **zero-migration** byte-identity regression (classical + hybrid unchanged) | **A** | — |
| DID-6 | string injectivity — CID-sensitivity (change committed CID ⇒ change DID) | **B** | — |
| DID-7 | **C7 F2 DoS cap** — length_pre_check gates bs58 O(N²) on resolve_signing / resolve_kem / UCAN-walk-iss; MAX_DID_KEY_STRING_LEN=4096 boundary | **B** | f2-precheck + max-string-cap + F-RK-10 + F2-cap-for-iss |
| AUDIT-1 | Shape-B codec **no-hardcoded-sizes** grep-audit (sizes from named constants) | **C** | gap-fill |

### benten-id — KeySetDocument codec (9)
| ID | Family | Gate |
|----|--------|------|
| KSD-1 | v1 canonical DAG-CBOR **golden-byte + golden-CID** pin `{v,sig,kem,sig_cp,kem_cp}` | **A** |
| KSD-2 | canonical key-order + definite-length **injectivity** | **A** |
| KSD-3 | **strict-canonical decode REJECT matrix** (C3): indefinite / dup-key / unsorted / non-minimal-int / trailing / extra-key | **B** |
| KSD-4 | encode→decode→encode round-trip byte-stable | **C** |
| KSD-5 | v=1 version pin + **forward-version typed-reject** | **B** |
| KSD-6 | sig_cp=0x0001 + kem_cp=0x647a **frozen field-value** pins | **A** |
| KSD-7 | kem-multikey **X25519-first golden** `[0xec‖x25519_32‖0x120c‖mlkem768_1184]` (C2, no reorder) | **A** |
| KSD-8 | kem-multikey malformed / wrong-length / wrong-codec / trailing reject | **B** |
| DOS-1 | **KeySetDocument bounded-decode DoS cap** — the SECOND untrusted input C7 does NOT cover (META #629) | **B** *(gap-fill)* |

### benten-id — resolvers (12)
| ID | Family | Gate |
|----|--------|------|
| DID-4 | **C6 multicodec-peek dispatch matrix** (0xed01→classical, 0x1211→composite; did:benten vs hybrid-did:key vs did:key+appended-tail) | **B** |
| RS-1 | resolve_signing classical (0xed01) arm round-trip | **C** |
| RS-2 | resolve_signing did:benten composite arm + trailing-CID strip, **zero-I/O** (killer-neutralized NEW fn) | **B** |
| RK-1 | resolve_kem happy-path — committed KEM key recovered | **C** |
| **RK-2 ★** | **resolve_kem CID 2nd-preimage fail-closed (FLAGSHIP-1 resolver layer)** | **B** |
| RK-3 | resolve_kem doc.sig == embedded-signing cross-check (spliced-sig reject) | **B** |
| RK-4 | resolve_kem kem_cp⟺components cross-check (algorithm-confusion reject, C2) | **B** |
| RK-5 | resolve_kem **PQ-floor** — 0x6400 classical-only reject (C5) | **B** |
| RK-6 | resolve_kem strict-canonical decode / no-raw-byte-compare (C3 consumer) | **B** |
| RK-7 | resolve_kem bare-did:key → **NoKemCommitment** degenerate | **B** |
| RK-8 | **C8** committed-key-set ⟺ recipient_key_generation precedence — id side | **B** *(under-spec)* |
| SEAM-1 | **cross-seam:** authority-signing-key == Drop-doc-committed-signing-key (confused-principal reject) | **B** *(gap-fill)* |

### benten-crypto-suite (1)
| ID | Family | Gate |
|----|--------|------|
| CS-1 | **FS-2** classical-from-bytes constructor (sig::PublicKey pq=None) so ONE helper handles both issuer shapes | **C** |

### benten-drop — Layer-C RecipientBinding + seal + GAP-KDB (11)
| ID | Family | Gate |
|----|--------|------|
| **DROP-1 ★** | **seal to substituted KEM key fails closed** — single 0x6510 + group 0x6520 + membership-set 0x6610 (FLAGSHIP-1 seal layer) | **B** |
| DROP-2 | RecipientBinding **sole-constructor, no-fallback-door** (C4) + seal-ineligible rejects | **B** |
| DROP-3 | **anti-downgrade** — no legacy raw-`&RecipientPublic` seal entry (Inv-23 catch-net; retires two-param seal) | **B** |
| DROP-4 | **C9 roster-replacement** — commitment + wrap-targets from ONE `&[RecipientBinding]` (retires layer_c.rs:1093 fabricated-DID) | **B** |
| DROP-5 | **C8** precedence — drop side (fail-closed on keyset⟺generation disagreement) | **B** *(under-spec)* |
| DROP-6 | seal→open round-trip + AAD threading (positive path) | **C** |
| **DROP-7 ★** | **Inv-23 real production-driving firing** (FLAGSHIP-1 invariant layer) | **B** |
| DROP-8 | **Layer-C sender-origin-auth** resolve_signing migration for did:benten sender (Scope-Min) | **B** *(gap-fill)* |
| DROP-9 | group audience_set_commitment **real-DID golden + sort-order-independence** (C9 wire bytes) | **A** *(gap-fill)* |
| DROP-10 | group-seal **empty `&[RecipientBinding]`** degenerate-cardinality reject | **C** *(gap-fill)* |
| DROP-11 | single-recipient did:benten **audience AAD golden** — u32 framing wire-transparency | **A** *(gap-fill)* |

### benten-id — Fork-A authority migration (13)
| ID | Family | Gate |
|----|--------|------|
| AUTH-1 | UCAN-walk did:benten composite issuer verifies (positive) | **C** |
| AUTH-2 | UCAN-walk backward-compat — 64-byte did:key unchanged | **C** |
| **AUTH-3 ★** | **UCAN-walk silent-PQ-strip reject (FLAGSHIP-2)** | **B** |
| AUTH-4 | both-halves bind SAME issuer key (cross-key half-splice reject) | **B** |
| AUTH-5 | per-link mixed-issuer dispatch (each link on its own multicodec) | **B** |
| AUTH-6 | unknown/reserved sig codepoint typed-reject (no silent fallback) | **B** |
| AUTH-7 | **ONE codepoint-dispatched helper** grep-audit (no 2nd Ed25519-only path) | **B** |
| AUTH-8 | RotationAttestation verify — did:benten prev composite + strip reject | **B** |
| AUTH-9 | accept_rotation_event authenticity gate hybrid + fail-closed BEFORE HLC/replay | **B** |
| AUTH-10 | Rotation backward-compat — did:key unchanged | **C** |
| AUTH-11 | **MAX_UCAN_ENVELOPE_BYTES re-sized** (admits 32-link composite, rejects over-cap, value frozen) | **A** |
| AUTH-12 | did:benten as UCAN **audience** (delegation target) | **C** *(gap-fill)* |
| AUTH-13 | DeviceAttestation::verify_signature_with — did:benten parent composite + strip reject | **B** |

### benten-engine (4)
| ID | Family | Gate |
|----|--------|------|
| ENG-1 | DeviceAttestationEnvelope::verify engine seam — did:benten device composite + strip reject; None-legacy skip preserved | **B** |
| ENG-2 | MAX_UCAN_ENVELOPE consumer parity — typed-CALL ucan_validate_chain + durable UCAN backend | **C** |
| ENG-3 | error-catalog mirror for new resolve_kem/RecipientBinding boundary-crossing ErrorCodes (CATALOG_VARIANT_COUNT) | **C** |
| ENG-4 | supersession record — ratified #1073 did:key-only-resolve clause RETIRED | **C** |

### docs / doc-conformance catch-nets (8)
| ID | Family | Gate |
|----|--------|------|
| DOCS-1 | Inv-23 + Compromise #67 registration catch-net (header/count coherence; baseline recovers Inv-1..22) | **C** |
| DOCS-2 | Compromise #67 honest disclosure + **no-overclaim** (TOFU/first-contact/bind-once; does NOT claim elimination) | **C** |
| DOCS-3 | SECURITY-PROOFS recipient-key premise **cites the binding** (Inv-23/RecipientBinding, drops address-book assumption) | **B** |
| DOCS-4 | THREAT-MODEL rung-4 recipient-key closure + **seal-path revocation-reach** residual (distinct from #67) | **C** |
| DOCS-5 | CRYPTO-CODEPOINTS 0x120c/0xec **WIRED** + did:benten method registered + 0xf0 **retired** | **C** *(guards freeze-perm values)* |
| DOCS-6 | V1-WIRE-FORMAT-INVENTORY registers did:benten + KeySetDocument surfaces (freeze-completeness) | **C** |
| DOCS-7 | Fork-A authority hybrid-migration doc-registration | **C** |
| DOCS-8 | honest-residual **offline-first-send doc-availability** disclosure (distinct from #67 AND revocation-reach) | **C** *(gap-fill)* |

---

## 2. Coverage Matrix (freeze-surface obligation × family)

| # | Ratified obligation | Families | Status |
|---|---------------------|----------|--------|
| 1 | did:benten method + byte layout, trailing-reject, no framing byte (C1) | DID-1,2,3,5,6; DOCS-5,6 | ✅ |
| 2 | KeySetDocument v1 DAG-CBOR, X25519-first (C2), no dev field | KSD-1..8; RK-4; DOCS-5 | ✅ *(dev-field only implicit → S1)* |
| 3a | resolve_signing (C6 method/multicodec-aware, composite, trailing-CID) | DID-4; RS-1,2 | ✅ |
| 3b | resolve_kem (C3 strict, CID 2nd-preimage, doc.sig==embedded, kem_cp⟺comp C2, PQ-floor C5) | RK-1..7; DOS-1 | ✅ |
| 4 | RecipientBinding (C4) + seal API single+group (C9) | DROP-1,2,3,4,6,9,10,11 | ✅ |
| 5 | Inv-23 | DROP-1,7; DOCS-1 | ✅ |
| 6 | F2 DoS cap for did:benten (C7) | DID-7; **DOS-1** (closes the 2nd-input hole C7 leaves) | ✅ |
| 7 | recipient_key_generation ⟺ keyset precedence (C8) | RK-8; DROP-5 | ⚠️ **under-spec → S2** |
| 8 | Compromise #67 TOFU residual | DOCS-2,4,8 | ✅ |
| 9 | FORK A — authority Ed25519→hybrid, silent-PQ-strip, envelope re-size | AUTH-1..13; ENG-1,2; CS-1; SEAM-1; DOCS-7 | ✅ |
| x-cut | no-hardcoded-sizes (#5); Row-D-13 injectivity; error-catalog mirror; #1073 supersession; cross-seam key-equality | AUDIT-1; DID-3/KSD-2/8; ENG-3; ENG-4; SEAM-1 | ✅ |

**HOLES: none hard.** The 9 gap-fills already closed the panel's per-dimension blind spots. Two soft items in §5.

---

## 3. Canary-First R3 Slicing

The canary is **benten-id** because the did:benten codec + KeySetDocument codec + resolve_signing/resolve_kem MINT the surface every downstream crate consumes (crypto-suite kem-wiring, benten-drop seal, Fork-A verify). R3 canary discipline = write the canary crate's red-phase files **and the shared fixtures first**, GATE on compile-as-red + fixtures usable cross-crate, then fan out.

**W0 — CANARY (benten-id), GATE before all fan-out.** Ship shared fixtures FIRST: did:benten builder, KeySetDocument builder, **throwaway-compute golden capture (M-20)**, RecipientBinding test constructors. Parallel red-phase sub-slices:
- **W0a** did:benten codec — DID-1, DID-2, DID-3, DID-5, DID-6, AUDIT-1
- **W0b** KeySetDocument codec — KSD-1..8
- **W0c** resolvers — DID-4, RS-1, RS-2, RK-1, **RK-2★**, RK-3, RK-4, RK-5, RK-6, RK-7
- **W0d** DoS bounds — DID-7, DOS-1
- **GATE:** all red(ignored) + compile + fixtures importable by benten-drop / benten-engine.

**W1 — crypto-suite kem-wiring** (tiny; dispatch immediately after GATE, may overlap W0c): CS-1.

**W2 — benten-drop Layer-C (GAP-KDB flagship), gates on W0 resolve_kem + RecipientBinding:** DROP-1★, DROP-2, DROP-3, DROP-4, DROP-6, DROP-7★, DROP-8, DROP-9, DROP-10, DROP-11. *C8 sub-slice (RK-8 + DROP-5) held until S2 resolved.*

**W3 — Fork-A authority migration (benten-id + benten-engine), CAN PARALLEL W2** (disjoint files; both gate only on W0 resolve_signing + W1 FS-2): AUTH-1..13, ENG-1, ENG-2, SEAM-1. *(SEAM-1 needs both resolve_signing and resolve_kem from W0.)*

**W4 — docs / invariants LAST** (tests reference real symbols, minted ErrorCodes, and post-fix doc counts): DOCS-1..8, ENG-3, ENG-4. *(DOCS-1/5/6 baseline-non-ignored parser arms are writable now; their RED arms flip in R5.)*

Critical path: **W0 → W2** (flagship-1) and **W0 → W3** (flagship-2) in parallel → W4. W1 is off the critical path.

---

## 4. Freeze-Gating Priorities

**TIER-A — freezes-permanent-bytes (MUST BE PERFECT — a wrong golden becomes permanent v1-beta wire):**
DID-1, DID-2, DID-5, KSD-1, KSD-2, KSD-6, KSD-7, DROP-9, DROP-11, AUTH-11.
→ **Reviewer law:** every golden hex/CID/tag captured via **throwaway-compute from the real encoder (M-20), never hand-authored** — a hand-authored golden matching a buggy encoder freezes the bug. Triple-check component order, X25519-first (C2), varints, sort-canonical (DROP-9), u32-vs-u16 AAD framing (DROP-11).

**TIER-B — security-critical (revert = real exploit):**
Flagships RK-2★, DROP-1★, DROP-7★, AUTH-3★; DID-3, DID-4, DID-6, DID-7; KSD-3, KSD-5, KSD-8; RS-2; RK-3, RK-4, RK-5, RK-6, RK-7; DROP-2, DROP-3, DROP-4, DROP-5, DROP-8; AUTH-4, AUTH-5, AUTH-6, AUTH-7, AUTH-8, AUTH-9, AUTH-13; ENG-1; SEAM-1; DOS-1; DOCS-3.
→ Each must have an `expect_err`/reject arm whose **revert flips Err→Ok** (the "would_fail_on_revert" is real). The 3-layer flagship + AUTH-7 grep-net must all go green together.

**TIER-C — important (positive-path, backward-compat, doc-conformance, DX):**
DID-2 counted in A; RS-1, RK-1, KSD-4; DROP-6, DROP-10; AUTH-1, AUTH-2, AUTH-10, AUTH-12; CS-1; ENG-2, ENG-3, ENG-4; AUDIT-1; DOCS-1, DOCS-2, DOCS-4, DOCS-5, DOCS-6, DOCS-7, DOCS-8.

---

## 5. Two flags for R3 (specify-before-red)

- **S1 (thin, cheap):** freeze item 2's **"NO dev field"** is only *implicitly* covered (KSD-3 unexpected-extra-key + KSD-1 golden field set). R3 must add an **explicit `dev`-field-present reject arm** to KSD-3 so the Shape-B-drops-the-dev-field decision is pinned, not incidental.
- **S2 (needs a design nail):** **C8** (recipient_key_generation ⟺ committed-key-set precedence) is **plan-acknowledged under-specified** ("R2/R5 nails exact semantics"). RK-8 + DROP-5 can pin the *role separation* (generation = intra-keypair freshness index; committed key-set = which keypair) and the *fail-closed-on-disagreement direction* now, but the **exact disagreement arm** (what counts as disagreement, and the precise typed reject) needs a design decision before its red test can assert a concrete `expect_err`. Hold the C8 sub-slice (RK-8 + DROP-5) until that decision lands; do not block W2's flagship families on it.

**Reviewer cross-cutting notes for R3:** (a) doc-conformance catch-nets (DOCS-1/5/6) MUST drive a **baseline non-ignored arm** over the on-disk doc that recovers today's Inv-1..22 / codepoints (proves the parser reads the doc, not a literal) — **never `assert_eq!(CONST,CONST)`**; (b) match house style — `canonical_bytes_v1_*.rs` for golden pins, `f_inj_*.rs` for injectivity/reject matrices, grep-absence pins (AUTH-7, AUDIT-1, DROP-3) mirror `ct_signature_eq` / `no_hardcoded_sizes` precedents; (c) every seal-fail-closed family exercises all three codepoints (0x6510/0x6520/0x6610) in one file per `f_lc_gap1` precedent.