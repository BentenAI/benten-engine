# Comment-Opportunity Scan — Implementation Libraries

Scan date: 2026-05-26
Scope: PQ + classical crypto implementation libraries (Rust/RustCrypto, TS/JS/noble, Java/BouncyCastle, AWS, Cloudflare, Go, Python, HPKE) where Benten Engine could legitimately comment, contribute, or watch.
Persona: implementation-library-ecosystem scout.
Method: WebFetch + WebSearch + `gh api` cross-verification of every primary claim.

Benten context: Currently on `ml-dsa = "0.1"` + `ml-kem = "0.2"` (RustCrypto), BLAKE3-based hash, planning v1-beta PQ-hybrid signature default = Ed25519 + ML-DSA-65 (LAMPS-COMPSIG-aligned) + X-Wing-style encryption envelope (`0x647A`). TS mirror via napi-rs; no separate JS PQ lib in use.

---

## 1. paulmillr/noble-post-quantum — L14 LAMPS-COMPSIG opportunity DEEP DIVE

### 1.1. State of `src/hybrid.ts` (verified at HEAD)

**Verified facts** (raw source pulled from `https://raw.githubusercontent.com/paulmillr/noble-post-quantum/main/src/hybrid.ts`):

- **`combineSigners()` function EXISTS** at approximately line 570 (composite signature infrastructure shipped).
- The file header carries the explicit comment **"There is no specs for this, but can be useful"** directly above `combineSigners`.
- **NO LAMPS-COMPSIG preset is exported.** No `mldsa65_ed25519`, `HashMLDSA65_Ed25519_SHA512`, or any concrete composite signature scheme. Only the generic `combineSigners` infrastructure + `ecSigner` wrapper exist.
- KEM side has rich presets (`ml_kem768_x25519`, `XWing` alias, `QSF_ml_kem768_p256`, `KitchenSink_*`, etc.). The signature side is empty.

**Verified open issues** (via `gh api`): only 2 — `#25 Implement NTRU Prime` (2025-05-12) + `#17 Implement HQC from FIPS-207` (2025-03-13). **NEITHER addresses LAMPS-COMPSIG nor composite signatures.** No open PR exists.

### 1.2. LAMPS draft state (post-comment-header)

- `draft-ietf-lamps-pq-composite-sigs` is at **draft-18 (April 9, 2026)**, intended status **Standards Track**, **in IETF Last Call** since January 2026, expiring 2026-10-11. <https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/18/>
- Pure-form composite combinations include `MLDSA44-ECDSA-P256-SHA256`, `MLDSA65-Ed25519-SHA512`, `MLDSA87-ECDSA-P384-SHA384`, etc.
- Hash-form variants (`HashMLDSA*`) accept a pre-hashed digest — important for Benten because the engine is already content-addressed (BLAKE3 hash of Node bytes); the signing path naturally has a hash in hand.
- **The hybrid.ts header comment is now stale** — there IS a spec, and it's in Last Call.

### 1.3. Maintainer pattern (paulmillr) + analogous PR precedent

**Verified via `gh api /repos/paulmillr/noble-post-quantum/contributors`**: 6 contributors total; paulmillr = 164 commits; external = panva (2), MurasameKei (1), FiloSottile (1), jiep (1), tob-scott-a (1).

External-contributor PR pattern (verified via `gh api`):

| PR | Author | Affiliation | Opened | Merged | Wall-clock | What it added |
|----|--------|-------------|--------|--------|------------|---------------|
| #39 | tob-scott-a | Trail of Bits | ~Apr 2026 | Apr 11, 2026 | days | Wycheproof KAT vectors |
| #34 | panva | (external) | Nov 21, 2025 | Dec 20, 2025 | ~30 days | `concrete-hybrid-kems` — ML-KEM768-X25519, ML-KEM768-P256, ML-KEM1024-P384 presets (analogous shape to LAMPS-COMPSIG) |
| #35 | FiloSottile | Filippo Valsorda | ~Dec 2025 | Dec 22, 2025 | days | Bug fix on ecdhKem default rand length |
| #31 | MurasameKei | (external) | ~Oct 2025 | Oct 15, 2025 | days | Doc fix |

**PR #36 ("ML-DSA primitives and threshold signing", BlobMaster41)** was closed Feb 2026 with author comment "not what i wanted to do" — author-closed, NOT maintainer-rejected. Self-withdrawn out of scope-mismatch.

**Pattern**: paulmillr accepts well-scoped external PRs from credible contributors. PR #34 is the **direct analogue**: an external contributor adding *concrete-hybrid-* presets that the codebase already had infrastructure for. ~30-day open→merge wall-clock with substantive review iteration.

### 1.4. PR shape estimate for Benten LAMPS-COMPSIG contribution

**Effort estimate: ~150-300 LOC + tests (within L14's 150-250 LOC band — confirmed).**

Concrete shape:
1. Pure-form preset `mldsa65_ed25519` calling `combineSigners(undefined, expandSeedXof, mlDsa65, ecSigner(ed25519))`.
2. Hash-form preset `HashMLDSA65_Ed25519_SHA512` (LAMPS draft-18 §6 variant) accepting pre-computed digest + algorithm parameter (HARDER — requires hash-context wiring; the generic `combineSigners` may not handle this without extension).
3. OID/encoding alignment per LAMPS draft-18 §A (ASN.1 encoding for X.509 PKI use — relevant if Benten wants PKI interop; OPTIONAL for Benten-internal use since Benten uses CBOR not ASN.1).
4. Test vectors from LAMPS draft-18 appendix.
5. Optional: add a few more popular variants (`MLDSA44-Ed25519`, `MLDSA87-Ed448`).

**Risk: header comment update**. The "There is no specs for this" comment must be updated/removed in the same PR — paulmillr may want that called out explicitly in the PR description.

### 1.5. Recommended action

**Action**: `CONTRIBUTE-PR-AFTER-G-CORE-3c-LANDS` (or pair with G-CORE-3c implementation — could be a Benten-team author with concrete production-use as motivation in the PR body).

**Sequencing**: dispatch only after Benten's own LAMPS-COMPSIG wire-format ratifies in G-CORE-3c so the PR can cite "production use at Benten v1-beta" as motivation. Pre-G-CORE-3c, file a discussion/issue first naming Benten as a downstream-with-concrete-plans, ask paulmillr if a LAMPS-COMPSIG preset PR would be welcomed, and (if yes) what shape he prefers (pure-form only, hash-form too, OID/ASN.1 encoding scope, etc.).

**Pre-comment GATE**: G-CORE-3c implementation choice MUST settle "which LAMPS-COMPSIG variant Benten ships" so the PR proposal matches. Most likely answer = `MLDSA65-Ed25519-SHA512` pure-form (matches the v1-beta default ratified in CLAUDE.md baked-in #5).

**Issue/discussion shape** (recommended pre-PR, ~3-5 sentences):
> "Hi — Benten Engine (content-addressed graph database, Rust + TS DSL) is shipping PQ-hybrid signatures at v1-beta this year using LAMPS-COMPSIG `MLDSA65-Ed25519-SHA512` as the default. Our TS-side currently has no PQ-hybrid composite-signature option; the `combineSigners()` infrastructure is exactly right but the `'There is no specs for this'` comment is now stale (draft-ietf-lamps-pq-composite-sigs is at draft-18, in IETF Last Call as of Jan 2026, with concrete pure-form + hash-form variants specified). Would a PR adding a `mldsa65_ed25519` preset (+ optionally the SSH/CMS-aligned HashML-DSA variants) be welcomed? Following the shape of panva's PR #34 for concrete-hybrid-kems."

**Adoption likelihood: HIGH-MEDIUM.** Spec exists; infrastructure exists; precedent PR #34 exists; the maintainer comment is stale (gives Benten the entry-point). Risk = paulmillr deferring until LAMPS is RFC (not Last-Call-draft); mitigation = follow BouncyCastle's draft-XX-implemented precedent.

---

## 2. RustCrypto open issues — Benten-as-adopter contribution opportunities

### 2.1. RustCrypto/signatures

**ml-dsa 0.1.0 SHIPPED 2026-05-17** (PR #1356; verified via `gh api`). Benten's `ml-dsa = "0.1"` Cargo constraint now pulls 0.1.0 stable — past the rc cycle. Notable changelog entries directly relevant to Benten:
- Side-channel fix #1144 (Barrett reduction, security-labeled).
- Constant-time `PartialEq` #1286.
- `ZeroizeOnDrop` for `SigningKey` #1345.
- Heap offload for large values #1320/#1344/#1345 (Benten's per-Node signing paths may benefit — `MaybeBox` ergonomics).
- External-μ support #1023/#1074 (relevant for content-addressed prehash use-case — exact Benten pattern).
- Migration from `sha3` → `shake` #1355 (Benten may want to check if its `Cargo.lock` deduplicates cleanly).

**Open ml-dsa issue #1020 — ZeroizeOnDrop for KeyPair** (opened 2025-07-16, no assignee, no PR). KeyPair type still doesn't implement ZeroizeOnDrop directly — relies on inner SigningKey's zeroize. **Benten contribution opportunity** = file a +1 comment confirming it's a real production concern (KeyPair is the surface a content-addressed-graph engine actually holds in long-lived RotationLog state — fragility cost is real for us). OR contribute the trivial `#[derive(ZeroizeOnDrop)]` patch — likely <20 LOC.

**Open issue #816 — DSA Wycheproof vectors** (opened 2024-04-23). Classical DSA, not ML-DSA — Benten doesn't use classical DSA. IRRELEVANT.

**Recommended action — issue #1020**:
- **COMMENT-NOW** (+ optionally trivial-PR): file a brief "+1, production-relevant" comment citing Benten's use of `SigningKey`/`KeyPair` in long-lived state.
- Comment shape (~3 sentences): *"Production use confirmation: Benten Engine (content-addressed graph database; v1-beta shipping with PQ-hybrid signatures) holds long-lived `ml_dsa::KeyPair` state in RotationLog and Atrium sync surfaces. The fragility concern is real — a future refactor of `SigningKey` zeroize semantics would silently regress us. Happy to send the trivial `#[derive(ZeroizeOnDrop)]` PR if maintainers agree on shape; otherwise filing this +1 to bump priority."*

### 2.2. RustCrypto/KEMs

**Open issue #25 — Constant-time evaluation of `Encode::<U1>::decode`** (opened 2024-06-03, no assignee, no PR). Asks for verification that Rust compilation doesn't introduce a secret-dependent branch (analogous to a known Clang issue with the Kyber reference impl). **Currently stalled for ~2 years.**

This is HIGH-VALUE for Benten because:
1. Benten ships `ml-kem` for X-Wing-hybrid encryption (`0x647A`) — every ciphertext decap runs through this code path.
2. Project Eleven's 2025-2026 audit found another timing side-channel in the sibling `ml-dsa` decompose (CVE-2026-22705 / GHSA-hcp2-x6j4-29j7) — **the bug-class IS active**. Independent investigation here is overdue.
3. tob-scott-a (Trail of Bits, also a noble-PQ external contributor) appears to be doing the active constant-time work on ml-dsa — there's a clear "who's doing the analogous work on ml-kem" gap.

**Recommended action**:
- **COMMENT-NOW**: file a comment naming Benten as a current production user planning v1-beta, and asking whether anyone (tob-scott-a / Trail of Bits / Project Eleven) has scheduled constant-time analysis of the corresponding ml-kem decap path. Reference the ml-dsa CVE-2026-22705 precedent.
- Comment shape (~3 sentences): *"Bumping this — Benten Engine (production user of `ml-kem` for X-Wing-hybrid encryption at codepoint 0x647A) would benefit. Given Project Eleven's recent disclosure of a sibling side-channel in `ml-dsa` decompose (GHSA-hcp2-x6j4-29j7), the same analysis methodology against `Encode::<U1>::decode` (and ideally the full decap path) seems overdue. Is anyone scheduled to do this analysis, or would maintainers welcome a contributor running the analyzer + filing findings?"*

**Open issue #264 — Incremental ML-KEM** (opened 2026-02-17). Probably IRRELEVANT for Benten (Benten doesn't stream KEM key material — full per-message envelope).

**Open issue #268 — `X25519DecapsulationKey` Decapsulate vs TryDecapsulate** (opened 2026-02-18). Trait-design question; orthogonal to Benten's use unless we hit a typed-error need post-G-CORE-3c.
- **WATCH-ONLY**: tag for G-CORE-3c review.

### 2.3. RustCrypto/elliptic-curves

Scanned all 12 open issues (via `gh api`): zero PQ-related. Active issues are k256/p256/p521 internal optimizations + ed448-goldilocks tracking. **IRRELEVANT for Benten's current PQ-hybrid scope.**

### 2.4. RustCrypto/traits

Scanned all open issues (via `gh api`); zero PQ-related. **#2401 (`signature` v4 tracking issue, 2026-05-02)** is the active forward planning thread for the next breaking release of the `signature` trait. Composite-signature trait shape questions (if any arose post-LAMPS-COMPSIG implementation) would land here.

- **WATCH-ONLY** for now; **COMMENT-IF-COMPOSITE-API-QUESTION-ARISES** during G-CORE-3c.

### 2.5. RustCrypto/RSA

GHSA-c38w-74pg-36hr (Marvin Attack timing side-channel) — disclosed earlier; not Benten-relevant (Benten doesn't use RSA). Noted as pattern: timing-CVE-class is active across RustCrypto.

---

## 3. BouncyCastle + AWS + Cloudflare adoption-state snapshot

### 3.1. BouncyCastle Java — actively shipping LAMPS-COMPSIG

**Verified state** (via search + bcgit/bc-java issue scan):

- BC Java **1.79** (late 2024) added initial NIST PQC + composite-signature support, aligned to **draft-ietf-lamps-pq-composite-sigs-07**.
- BC Java **1.80** added keytool compatibility + PREHASH variants.
- BC Java **1.82** + LTS **2.73.9** (2026) continue active maintenance.
- Issues closed in 2025-2026: #2162 (hash input to composite ML-DSA), #2073 (`HashMLDSA65-Ed25519-SHA512` PKCS#8 import), #1934 (PQC IETF-Hackathon composite ML-DSA + EDDSA X.509).
- PR #1546 merged the experimental composite implementation.

**Implication for Benten**: BouncyCastle's `HashMLDSA65-Ed25519-SHA512` is a directly Benten-compatible interop target. Future Java/JVM Benten consumers (Phase 6+ AI-agent integrations, Phase 7 Garden) could share signed artifacts cross-language.

**Stale-draft caveat**: BC Java is on draft-07; LAMPS is now at draft-18 + Last-Call. **BC is the upstream-most production-shipping implementation.** Benten should match draft-18 (the in-Last-Call version), not draft-07 — but BC's experience proves the design ships at production scale.

**Recommended action**:
- **WATCH-ONLY** on bcgit/bc-java — no open issue Benten can usefully comment on.
- Note BC's draft-XX-version as a reference for Benten's wire-format ratification in G-CORE-3c.

### 3.2. AWS aws-lc-rs — issue #773 (ML-DSA support)

**Verified via `gh api`**: opened 2025-04-10, **status open** (despite earlier search saying "closed" — search snippet was inaccurate). No assignee, no comments visible in fetch.

**Note**: WebFetch said "Closed" but didn't show a closure event. Treating as OPEN per gh-api-style verification gap; Benten doesn't depend on aws-lc-rs so the distinction doesn't change actionability.

aws-lc itself has ML-DSA support; aws-lc-rs (Rust bindings) is the lagging piece. Benten could in principle migrate from RustCrypto ml-dsa → aws-lc-rs ml-dsa to gain FIPS-validation downstream — but **only relevant for v1-GM era** (post-audit) and **only if Benten ever needs FIPS-140-3 inheritance** (no such requirement in current plan; CLAUDE.md baked-in #5 commits to vetted-upstream-wrapping NOT FIPS chain).

**Recommended action**: **WATCH-ONLY**. Re-evaluate at v1-GM if FIPS-inheritance becomes a sales/compliance lever. No comment-now value.

### 3.3. Cloudflare circl issue #587 — Composite Asymmetric Schemes Support

**Verified live thread** (via `gh api /repos/cloudflare/circl/issues/587/comments`):

- Opened 2026-02-27 by Konicai (Crypto4A).
- Crypto4A offered to contribute the implementation.
- Crypto4A asked (2026-03-18): (1) can `sign.Scheme` API be broken to accommodate variable-size composite ML-DSA keys+sigs? (2) what about Brainpool constant-time requirement?
- armfazh (Cloudflare maintainer) asked for the spec reference.
- **bwesterb (Cloudflare cryptographer, X-Wing co-author)** responded 2026-04-13 with TWO critical concerns:
  1. **"Support is not very useful to us unless there is TLS integration which requires upstream support."**
  2. **"With the many choices of composites there is a real risk of fragmentation; see slide 29 and onwards of [my RWPQC 2026 presentation](https://westerbaan.name/~bas/rwpqc2026/bas.pdf). If we add composites, it'd just be MLDSA44P256 for now."**

**Implications for Benten — HIGH IMPACT**:

1. **Convergence signal**: Cloudflare's expressed preferred composite = `MLDSA44P256`. Benten's CLAUDE.md baked-in #5 commits to `MLDSA65-Ed25519`. These are **DIFFERENT level + curve choices** — Benten's choice is closer to what BouncyCastle ships (`HashMLDSA65-Ed25519-SHA512`) but DIFFERENT from what Cloudflare would prefer.
2. **Fragmentation risk is a real ecosystem-level concern.** Bwesterb is a credible voice (X-Wing co-author). Benten should at minimum read slide 29+ of his presentation before final G-CORE-3c-RATIFICATION (PDF couldn't be extracted in this scan — defer to human/orchestrator manual read).
3. **MLDSA65 vs MLDSA44**: Benten's MLDSA-65 choice gives Category-3 security (192-bit classical, ~192-bit quantum); MLDSA-44 = Category-2 (128-bit). BC LTS ships both; the LAMPS draft-18 covers both. Benten's MLDSA-65 choice is more conservative.

**Recommended action**:
- **COMMENT-AFTER-bwesterb-slide-review-by-Ben**: Benten should NOT immediately comment until Ben reads Westerbaan's fragmentation slides + decides whether to:
  - (a) Stay with MLDSA65-Ed25519 (BouncyCastle-aligned, ecosystem fragmenting risk).
  - (b) Reconsider to MLDSA44P256 (Cloudflare-aligned, slightly less conservative security).
  - (c) Ship MULTIPLE composite variants (heavy implementation cost but maximum interop).
- After Ben's call, an *informational* comment on circl #587 noting Benten as another production-bound user of LAMPS-COMPSIG would add a data point to the convergence/fragmentation discussion.
- Comment shape (after-decision; ~4 sentences): *"Adding a data point as another upstream-bound production user of LAMPS-COMPSIG: Benten Engine (content-addressed graph database, Rust + TS DSL, v1-beta in 2026) is shipping `[MLDSA65-Ed25519-SHA512 | MLDSA44P256]` based on [reasoning]. BouncyCastle is already shipping `HashMLDSA65-Ed25519-SHA512` at production scale; the convergence question feels open in practice. To bwesterb's fragmentation concern (slide 29+) — would a community survey help anchor the discussion?"*

**This is the single highest-leverage informational comment in this scan** because Cloudflare's choice has TLS-ecosystem downstream effects on what becomes the de-facto-default LAMPS variant.

### 3.4. PyCA cryptography — open issues

- **#14827** "Provide HashML-DSA" (opened 2026-05-08) — recent, no PR yet, no assignee.
- **#14419** "SLH-DSA support" (opened 2026-03-04).

Benten doesn't depend on PyCA cryptography. Python-side interop is not in scope for v1.

**Recommended action**: **IRRELEVANT-FOR-NOW**. Re-evaluate at Phase 5+ if Python bindings become a roadmap item.

### 3.5. Go cryptography (golang.org/x/crypto + crypto/hpke)

- `crypto/hpke` in stdlib now provides MLKEM768X25519 (X-Wing alias) per Go crypto-hpke proposal Issue #75300.
- Filippo Valsorda maintains `filippo.io/hpke` as a research/preview package — X-Wing implementation reference.
- **No actionable open thread** — Go's PQ + X-Wing landscape is mature and converging on stdlib.

**Recommended action**: **WATCH-ONLY** on `crypto/hpke` stdlib stabilization (relevant if/when Benten ships a Go client binding at Phase 5+).

---

## 4. HPKE / X-Wing adjacent

### 4.1. X-Wing implementations cross-language

Verified state:
- **Codepoint**: 25722 = 0x647A (matches Benten's planned codepoint exactly).
- **Cloudflare circl** ships X-Wing (PR #471).
- **Go stdlib `crypto/hpke`**: MLKEM768X25519 (X-Wing alias).
- **TS `@hpke/hybridkem-x-wing`**: NPM package available.
- **Rust**: noble-post-quantum has `XWing` alias; no major RustCrypto-blessed X-Wing crate found in scan (the X-Wing combiner is small — ~24 LOC per CLAUDE.md baked-in #5; Benten implements its own combiner per design).

**No actionable open thread.** The X-Wing implementation landscape is converging — codepoint stable across ecosystems.

**Recommended action**: **WATCH-ONLY**. Re-evaluate if Benten ever needs an interop test against another X-Wing implementation (cross-language envelope decryption test).

### 4.2. ML-DSA-B + BLAKE3 — Project Eleven's Suite-B initiative (NEW OPPORTUNITY surfaced mid-scan)

**Verified state** (Project Eleven blog 2025-2026 + Taurus blog):

- **ML-DSA-B** = ML-DSA with SHAKE replaced by **BLAKE3** for internal hashing.
- Performance claims: up to 60× message pre-hash, 20% signature, 30% verification on x86_64.
- Initiative by Project Eleven + Taurus (JP Aumasson) + Zcash (Zooko) — credible cryptographic team.
- **Rust implementation = fork of RustCrypto ml-dsa** (NOT upstream yet).
- Companion: SLH-DSA-B (BLAKE3 variant of SLH-DSA).

**Implication for Benten — HIGH POTENTIAL ALIGNMENT**:

Benten's CLAUDE.md baked-in #5 commits to BLAKE3 as the v1 default hash. **ML-DSA-B uses BLAKE3 internally — this is a natural Benten alignment Benten should be aware of.**

HOWEVER — critical caveats:
1. **ML-DSA-B is NOT FIPS-204 standard** — it's a Suite-B variant. Shipping ML-DSA-B would deviate from the NIST-standardized algorithm, weakening the "vetted-upstream" framing in CLAUDE.md baked-in #5.
2. **The independent audit Benten commits to (NF-2 / C-GM-AUDIT) is for vanilla ml-dsa**, not ML-DSA-B. Switching variants resets the audit dependency.
3. **Interop**: BC/Cloudflare/AWS/etc. all ship FIPS-204 ML-DSA, not ML-DSA-B. Adopting ML-DSA-B fragments Benten away from the LAMPS-COMPSIG ecosystem.

**Recommended action**:
- **WATCH-ONLY for v1-beta** (do NOT adopt ML-DSA-B as default; stay FIPS-204 vanilla).
- **COMMENT-WORTHY at Phase 5+**: if Project Eleven's Suite-B initiative gains broader adoption + Benten's BLAKE3 commitment aligns, file a public comment naming Benten as an interested downstream observer of the Rust ML-DSA-B fork.
- **Surface to Ben**: this is exactly the kind of "convergent-with-Benten's-stack" optimization that warrants awareness even if NOT adopted. The signal is "BLAKE3 keeps showing up in serious crypto initiatives — Benten's hash choice is increasingly validated."

---

## 5. Cluster pattern observations

### 5.1. The "production user comment" pattern is repeatedly valuable

Most-leverage Benten comments across this scan share a shape:
- File on a low-activity but technically-correct open issue (ml-dsa #1020, ml-kem #25, circl #587).
- Establish Benten as a real downstream production user with concrete v1-beta scope.
- Make a specific factual offer (file a small PR, run a tool, share interop data).
- DON'T file general "thanks" comments or "+1" with no substance.

This pattern scales to ~5 high-leverage comments total across the implementation library ecosystem. Higher volume risks comment-noise reputation damage.

### 5.2. The LAMPS-COMPSIG ecosystem is at a fragmentation inflection point

Three implementations are concretely shipping:
- BouncyCastle Java: `HashMLDSA65-Ed25519-SHA512` (PRODUCTION, draft-07-aligned).
- noble-post-quantum: generic `combineSigners` infrastructure only (NO presets).
- Cloudflare circl: planning `MLDSA44P256` only (different L + curve).

The LAMPS draft is in **Last Call** — RFC publication imminent. The next ~6 months will see implementation convergence (or fragmentation). **Benten's choice of `MLDSA65-Ed25519-SHA512` aligns with BouncyCastle** and is the most-conservative-security choice. This is a defensible choice but Benten should be aware Cloudflare may push the ecosystem differently.

**Action implication**: Benten's G-CORE-3c implementation should explicitly DOCUMENT the choice rationale (vs MLDSA44P256, vs MLDSA87-Ed448) so future re-evaluation has a paper trail.

### 5.3. Side-channel CVE-class is active in RustCrypto PQ libs

Recent timeline:
- 2026-01-09: GHSA-hcp2-x6j4-29j7 ml-dsa decompose timing side-channel (CVE-2026-22705).
- 2026-XX-XX: GHSA-5x2r-hc65-25f9 ml-dsa repeated-hint signature verification.
- Project Eleven's "The Belt is Vacant" + "PQ Implementation Vulnerabilities Volume 1" framing — explicit "more bugs expected" posture.
- ml-kem #25 constant-time evaluation OPEN for ~2 years — analogous bug-class unverified.

**Implication for Benten**: the v1-beta security audit (NF-2 / C-GM-AUDIT) is real necessary load-bearing — not a checkbox. Benten's hybrid construction (Ed25519 ⊕ ML-DSA-65) is the right safety posture because the classical half is the audited floor while the PQ half goes through this CVE-prone settling period.

**Action implication**: Benten should TRACK each new ml-dsa/ml-kem CVE as a v1-beta release gate (force minimum-patched-version pin). The `Cargo.lock` deny-rule should require ml-dsa >= 0.1.0 + tracking each new advisory.

### 5.4. Trail of Bits + Project Eleven are the active PQ-Rust security ecosystem

- tob-scott-a (Trail of Bits Senior Security Engineer) contributes to both RustCrypto ml-dsa (PR #1144 Barrett reduction, PR #1245 ctutils constant-time) and noble-post-quantum (PR #39 Wycheproof KAT).
- Project Eleven funds the audit findings + maintains the ML-DSA-B Suite-B fork.

Benten should consider: when commissioning the NF-2 audit, these two organizations are the natural fit (already deep in this exact code path). Trail of Bits's existing public work on ml-dsa/ml-kem constant-time analysis dramatically lowers per-engagement cost.

**Action implication (not in scope of this scan but worth surfacing)**: when Ben funds the NF-2 audit, Trail of Bits and Project Eleven are the leading candidates. Building a relationship via well-targeted comments NOW (e.g. noble-post-quantum LAMPS-COMPSIG PR + ml-dsa #1020 + ml-kem #25 comments) is also relationship-building for the audit conversation later.

---

## Top-line recommended action queue (priority order)

1. **PRE-G-CORE-3c**: Ben reads Westerbaan RWPQC 2026 slides 29+ on composite fragmentation before final MLDSA65 vs MLDSA44 vs multi-variant ratification. (PDF saved by WebFetch tool at `/Users/benwork/.claude/projects/-Users-benwork-Documents-benten-engine/962eba1a-4171-40fc-9246-b86b29ecaa0a/tool-results/webfetch-1779781281640-61ddn5.pdf`.)
2. **DURING-G-CORE-3c**: file noble-post-quantum LAMPS-COMPSIG discussion/issue (pre-PR scoping ping to paulmillr) → land contribution PR after G-CORE-3c implementation.
3. **NEAR-TERM (orchestrator-direct, low-cost)**: comment on RustCrypto/KEMs #25 (constant-time evaluation, bumps a 2-year-old issue) and RustCrypto/signatures #1020 (ZeroizeOnDrop for KeyPair, optionally + trivial PR).
4. **AFTER-G-CORE-3c-RATIFIES**: informational comment on cloudflare/circl #587 sharing Benten's variant choice as a data point in the convergence/fragmentation discussion.
5. **AT-NF-2-AUDIT-SCOPING-TIME**: leverage relationships built above when commissioning the v1-beta audit (Trail of Bits + Project Eleven are the natural fit).

---

## Sources

Primary github issues + PRs verified via `gh api` (noble-post-quantum + RustCrypto/signatures + RustCrypto/KEMs + RustCrypto/traits + cloudflare/circl). Web sources:

- <https://github.com/paulmillr/noble-post-quantum> + `/src/hybrid.ts` + `/pulls/34` + `/pulls/36`
- <https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/18/>
- <https://github.com/RustCrypto/signatures/issues/1020>
- <https://github.com/RustCrypto/KEMs/issues/25>
- <https://github.com/RustCrypto/signatures/pull/1144> (Barrett reduction)
- <https://github.com/RustCrypto/signatures/pull/1356> (ml-dsa 0.1.0)
- <https://github.com/cloudflare/circl/issues/587> (+ comments via gh api)
- <https://github.com/aws/aws-lc-rs/issues/773>
- <https://www.bouncycastle.org/resources/latest-nist-pqc-standards-and-more-bouncy-castle-java-1-79/>
- <https://www.bouncycastle.org/resources/pqc-and-lightweight-cryptography-updates-bouncy-castle-1-80-java/>
- <https://github.com/RustCrypto/signatures/security/advisories/GHSA-hcp2-x6j4-29j7>
- <https://rustsec.org/advisories/RUSTSEC-2025-0144.html>
- <https://www.projecteleven.com/blog/the-state-of-post-quantum-cryptography-in-rust-the-belt-is-vacant>
- <https://blog.projecteleven.com/posts/announcing-ml-dsa-b-optimizing-post-quantum-signatures-with-blake3>
- <https://github.com/pyca/cryptography/issues/14827>
- <https://westerbaan.name/~bas/rwpqc2026/bas.pdf> (binary PDF; fetched but not extracted — Ben to review)
- <https://datatracker.ietf.org/doc/draft-connolly-cfrg-xwing-kem/>
