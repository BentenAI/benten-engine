# Senior-Cryptographer Pre-Commit Review: Bird-of-Prey vs LAMPS Composite ML-DSA as v1-beta Default Signature

**Reviewer:** Senior Cryptographer (engaged 2026-05-26)
**Decision-surface:** v1-beta wire-format freeze for Benten Engine signature codepoint default
**Authority of this document:** ADVISORY. Final disposition rests with Ben.

---

## 0. Reading-order note

This review is long because the decision is load-bearing on the v1-beta wire format. The TL;DR sits in §1. The most important sections for a binary decision are §1, §2, §6, §9, §11. The technical detail in §3-5 supports the recommendation; the codepoint table in §8 is operationally actionable; §10 anchors every claim to a primary source.

---

## 1. Executive recommendation

**CONDITIONAL NO-GO** on shipping Bird-of-Prey (or any draft-prabel-derived SUF-CMA-preserving construction) as the v1-beta DEFAULT at wire-format freeze.

**Recommended path:** Ship **LAMPS Composite ML-DSA (`id-MLDSA65-Ed25519-SHA512`)** as the v1-beta default at codepoint `0x0001` AND pair it with revoke-by-tuple-not-CID hardening at the application layer. Reserve codepoint `0x0002` (or higher) for a future SUF-CMA-preserving construction once one of three conditions is met: (a) the underlying Bird-of-Prey/draft-prabel/Silithium construction reaches IETF WG-adoption (LAMPS or CFRG) with consensus and IANA codepoint, (b) Bird-of-Prey appears in the formal EUROCRYPT 2026 LNCS proceedings with no errata, AND (c) an independent reference implementation in Rust exists that has been audited (or formally verified) at a quality level comparable to the underlying ML-DSA crate. Until all three hold, shipping fresh academic crypto in a v1-beta wire-format freeze on a 5-week-old, 1-contributor project is not defensible.

The malleability hazards the cryptographer-critic L12 finding identifies ARE REAL, but the right primary mitigation is **application-layer revoke-by-tuple plus content-CID identities derived from the canonical message body (NOT the signature)** — not switching the algorithm.

This is a disagreement with the directional framing in the brief. See §9 for the argument in full.

The remainder of this document substantiates this recommendation and provides the operational artifacts (codepoint table, hazard surface, test requirements, mitigation table) regardless of which path Ben ratifies.

---

## 2. Construction-choice recommendation (if a SUF-CMA-preserving construction MUST ship at v1-beta)

If despite §1 the directive remains "ship a SUF-CMA-preserving construction as default at v1-beta," the recommended construction is:

**draft-prabel-cfrg-suf-hybrid-sigs-01 Section 3 (Generic Binding Construction), instantiated as `Ed25519 || ML-DSA-65` with the binding `s2 = MLDSA.Sign(sk2, m' || s1)`.**

NOT the Bird-of-Prey "non-black-box Fiat-Shamir variant" (Section 4 of draft-prabel; "BoP-2" / "Bird-of-Prey-2" in the EUROCRYPT paper), because:

1. **Section 4 requires the second component to satisfy "message-bound security (MBS)" and "random-message validity (RMV)"** (draft-prabel §4.4). MBS is an emerging notion; RMV is a Janneck-novel property. The draft asserts both hold for ML-DSA/SLH-DSA/Falcon but cites no peer-reviewed standalone proof for ML-DSA's RMV property. This is a real foundation crack for a v1-beta wire freeze.
2. **Section 4 is non-black-box: it requires opening up the identification scheme** (`ID.Com`, `ID.Rsp`, `ID.ExtCom`). EdDSA implementations in Rust do not expose these primitives. Building them either forks `ed25519-dalek` or reimplements the EdDSA primitives — both violate Benten's CLAUDE.md baked-in #5 "never fork, never reimplement crypto primitives" discipline.
3. **The size benefit of Section 4 ("smaller than sum") is marginal in absolute bytes** (~32 bytes saved on a 3373-byte composite, i.e., <1%). The risk/reward is structurally bad.
4. **Section 3 has the cleaner security argument** (draft-prabel §6.2.2): forgery reduces to SUF-CMA of the second component via a tight two-case reduction. The argument is reproducible inside an undergraduate-level cryptography proof framework.

Comparison summary (cite-anchored to §10):

| Construction | SUF-CMA proof | Reqs on component 2 | Implementation | Maturity | Size (Ed25519+MLDSA65) |
|---|---|---|---|---|---|
| LAMPS Composite ML-DSA | EUF-CMA only (none for SUF against quantum) | EUF-CMA | Black-box | WG-adopted, draft-19, OID early-allocated | 3373 B |
| draft-prabel §3 (generic binding) | SUF-CMA preserved if s2 is SUF-CMA | SUF-CMA | Black-box; sequential sign | Individual draft, not WG-adopted, Bird-of-Prey EUROCRYPT'26 backing | ~3373 B |
| draft-prabel §4 / Bird-of-Prey-2 (Fiat-Shamir) | SUF-CMA via novel RMV property | MBS + RMV (RMV novel) | Non-black-box; requires ID-scheme internals | Same draft state; harder impl | ~3341 B (32 B savings) |
| Silithium / IACR 2025/2059 | Hybrid EU-CMA notion (new framework) | Two ID schemes; FS-with-aborts | Requires ML-DSA "external mu" API | PQCrypto 2026; PQShield | Smaller than concat per abstract; exact unspecified |

**Silithium (IACR 2025/2059 by Devevey/Guerreau/Roméas):** also a credible candidate, but: (a) introduces a new "Hybrid EU-CMA" security definition that is not yet WG-vetted or in IETF, (b) requires the ML-DSA "external mu" API which is supported in `mldsa-native` and discussed for `dotnet/runtime` but is NOT exposed in any production Rust ML-DSA crate Benten could pull in today (`libcrux-ml-dsa`, `fips204`, `pqcrypto-mldsa` — none expose external-mu to Rust callers per current crates.io documentation), and (c) the construction is Fiat-Shamir + EC-Schnorr, not EdDSA. Switching from Ed25519/EdDSA to EC-Schnorr is a separate fork beyond Benten's current crypto stack. Silithium is a strong candidate for a SECOND post-v1-beta codepoint when the Rust ecosystem catches up; it is not buildable today within Benten's "never fork crypto primitives" rule.

---

## 3. Implementation-hazard surface + required mitigations (if Bird-of-Prey ships)

### 3.1 Constant-time requirements

The binding construction itself (concatenation, hashing, sequential signing) involves no secret-dependent branches and inherits constant-time properties from its components. **The hazard surface is entirely inherited from ML-DSA.**

**Critical finding (load-bearing for ANY Benten path involving ML-DSA, whether LAMPS or Bird-of-Prey):**
- **RUSTSEC-2025-0144** (Jan 2026): `ml-dsa` crate has a timing side-channel in the `Decompose` algorithm used during signing.
- **GHSA-5x2r-hc65-25f9** (Jan 2026): `ml-dsa` crate verification accepts signatures with repeated hint indices (FIPS 204 spec violation; commit `b01c3b7` changed `<` to `<=`).
- **libcrux-ml-dsa GHSA-fhvh-vw7h-9xf3** (May 2026): AVX2 implementation of `use_hint` mishandled edge case; signatures that should reject are accepted.
- **Verification Theatre (IACR 2026/192):** identifies 13 vulnerabilities in libcrux including V8 + V9 — two FIPS 204 specification violations in ML-DSA verifier, despite formal verification. V8 = doubled-norm bound rendering a FIPS 204 security check dead code; V9 = hint deserialization checks wrong counter.

This is the most important finding in this review and it CUTS BOTH WAYS for Bird-of-Prey vs LAMPS. Every Benten signature path that includes ML-DSA inherits this entire hazard surface; algorithm choice does not change it.

### 3.2 Side-channel surfaces

- **Timing:** ML-DSA's rejection-sampling loop is timing-variable but designed to leak only the iteration count (not secret data); however, the `Decompose` finding shows real implementations have shipped exploitable timing leaks.
- **Cache:** AVX2 implementations (libcrux) have already shipped a cache-related bug. `fips204` claims portable constant-time but is unaudited.
- **Fault injection:** ML-DSA hint structure is fault-sensitive (the duplicate-hint-indices finding is a near-miss for fault attacks). Out of scope for Benten v1-beta but tag for v1-GM audit.

### 3.3 Required mitigations

- **MUST:** Pin a single ML-DSA crate version and version-pin in `Cargo.toml` with a comment naming the audit/CVE state at pin-time. Document the pin in `SECURITY-POSTURE.md` as Compromise # (TBD).
- **MUST:** Subscribe to RustSec advisories for `ml-dsa`, `libcrux-ml-dsa`, `fips204` and require triage within 48h of any new advisory.
- **MUST:** Defer adoption of `external mu` API even if it becomes available in Rust until the chosen ML-DSA crate's external-mu path has its own audit. The pre-hash external-mu API has been flagged as a hazard surface (see `adams-bridge` issue #54 "Don't use a prehashed version of ML-DSA").
- **SHOULD:** Choose `libcrux-ml-dsa` for the partial formal verification (despite the Verification Theatre findings — partial verification > none), pinned at a version >= the GHSA-fhvh-vw7h-9xf3 fix (0.0.9 per the advisory).
- **NICE-TO-HAVE:** Pre-v1-GM: fuzz the Benten signature wrapper boundary against the chosen ML-DSA crate using `cargo-fuzz` with the FIPS 204 KAT vectors as a corpus seed.

---

## 4. Test-coverage requirements

### 4.1 KAT (Known-Answer-Test) sources

**For LAMPS Composite ML-DSA (recommended v1-beta default):**
- LAMPS draft references "Appendix E test vectors" in the draft text but the test-vector body was truncated in my fetch.
- Cross-reference: `lamps-wg/draft-composite-sigs` GitHub repo holds the source; KATs are in `vectors/` per typical IETF draft convention. Pin commit SHA when used.
- For the underlying ML-DSA-65 component: NIST CAVP ACVP test vectors (canonical FIPS 204 test set). RustCrypto and libcrux both test against these.

**For draft-prabel §3 (if pursued as 2nd codepoint):**
- **No test vectors provided in the draft.** This is explicitly stated in draft-prabel-01 (verified: §10 of this review). Benten would need to author its own KATs and contribute upstream — adding standardization burden onto a 5-week-old project.

**For Bird-of-Prey:**
- No reference implementation. No KAT corpus. Benten would author its own.

### 4.2 Property tests (Benten-side)

For whichever construction ships, the following property tests are MANDATORY pre-tag:

1. **Sign-then-verify correctness:** ∀ valid `(sk, pk, m, ctx)`: `Verify(pk, m, Sign(sk, m, ctx), ctx) == true`. Proptest with randomized inputs; ≥10,000 cases per arm. (Standard.)
2. **Cross-component forgery rejection:** If only ONE component verifies (the other replaced by random/zero bytes), Verify MUST reject. Test both halves. **This is the actual non-separability/binding test.**
3. **Tampered-message rejection:** ∀ `(sk, m, ctx)`, m' ≠ m: `Verify(pk, m', Sign(sk, m, ctx), ctx) == false`. Proptest with single-bit, single-byte, and arbitrary mutations.
4. **Domain-separator integrity:** Signatures over `(m, ctx1)` MUST NOT verify under `(m, ctx2)` for ctx1 ≠ ctx2. Critical for application-layer scope separation.
5. **Sig-format malleability proptest:** Mutate sig bytes; assert Verify ALWAYS returns false. For LAMPS-default this proptest WILL find the ECDSA-style malleability in ML-DSA-65's randomized signing arm — that finding is expected and documents the EUF-CMA-only gap explicitly.
6. **Public-key substitution test (BUFF):** Sign under sk1/pk1; produce a different pk2 such that Verify(pk2, m, sig) succeeds. EdDSA and ML-DSA both claim resistance, but the BUFF property of the composite is not formally analyzed in LAMPS Section 9.2. Document the result.

### 4.3 Fuzzing surfaces

- **MUST:** Fuzz the signature parser (`Deserialize(sig_bytes)`) — the multi-component signature parser is the highest-risk surface for type-confusion bugs.
- **MUST:** Fuzz the verify entry point (`Verify(pk, m, ctx, sig)`) with arbitrary inputs.
- **SHOULD:** Fuzz cross-codepoint boundary (sig signed under codepoint A presented to verifier expecting codepoint B; verifier MUST reject with typed-unsupported-algorithm, not silently fail-open).

### 4.4 Required cryptographer-eyes-count before v1-beta tag

**For LAMPS-default path (my recommendation):**
- 1 external cryptographer review of the Rust wrapper around the chosen ML-DSA crate (NOT the crate itself; the wrapper). ~1-2 days work for an experienced reviewer.
- The underlying LAMPS draft already has years of IETF LAMPS-WG review.

**For Bird-of-Prey / draft-prabel path:**
- 2 external cryptographers minimum (one for the construction, one for the implementation).
- One must be familiar with Fiat-Shamir non-black-box variants if Section 4 is chosen.
- Estimated cost: ~2-4 weeks elapsed; ~$15K-50K if engaging via PQShield, Cure53, NCC Group, or Cryspen.
- **This is the kind of expense the C-GM-AUDIT line item already plans for ml-dsa/ml-kem** (per CLAUDE.md baked-in #15). Adding a third audit lane on a new combiner construction expands that budget.

---

## 5. Standards-posture analysis

### 5.1 Comparison

| Document | Status | WG-Adoption | RFC timeline | Implementations |
|---|---|---|---|---|
| draft-ietf-lamps-pq-composite-sigs-19 | WG-adopted | LAMPS-WG | Standards-Track; pub probably Q4 2026 – H1 2027 | BouncyCastle 1.80+ (Java/.NET), OpenSSL 3.5 partial, AWS KMS, Thales Luna HSM, Mozilla NSS evaluating, OpenPGP-PQC mandates it |
| draft-prabel-cfrg-suf-hybrid-sigs-01 | **Individual** (not WG-adopted) | CFRG (mailing-list venue only) | No WG-adoption call observed in CFRG list as of May 2026 | No production implementations known |
| draft-ietf-pquip-hybrid-signature-spectrums | WG-adopted (PQUIP) | PQUIP-WG | Informational; classification doc only | N/A |

The dispatcher brief stated "draft-prabel is INDIVIDUAL submission (NOT WG-adopted). Note from datatracker: 'This I-D is not endorsed by the IETF and has no formal standing in the IETF standards process.'" This is correct and reproduced in draft-prabel-01 boilerplate. Verified.

### 5.2 WG-adoption timeline for Bird-of-Prey/draft-prabel

Realistic best case for IETF WG-adoption of a Bird-of-Prey-style SUF-CMA hybrid:
- CFRG adoption call: not yet announced; mailing list discussion ongoing as of 2026-Q2.
- Earliest call-for-adoption: late 2026.
- WG-adoption to first WG draft: 3-6 months.
- WG-draft to IANA codepoint early-allocation: another 6-12 months.
- **Realistic: late 2027 to mid-2028 before a published WG-blessed alternative exists.**

The EUROCRYPT 2026 LNCS proceedings appearance for Bird-of-Prey is much sooner (EUROCRYPT 2026 conference is May 2026; proceedings likely September 2026). But conference publication ≠ standards adoption ≠ Rust reference implementation.

### 5.3 Supersession risk

The hybrid-signature subfield is moving fast in 2026:
- Bird-of-Prey (2025/1844) → EUROCRYPT 2026
- Silithium (2025/2059) → PQCrypto 2026
- Best of Both KEMs (2025/1444) → analogous KEM-side work
- Multiple competing IETF drafts: draft-prabel-cfrg-suf-hybrid-sigs (CFRG), draft-prabel-jose-pq-composite-sigs (JOSE), draft-ounsworth-pq-composite-sigs (LAMPS), draft-sun-ssh-composite-sigs (SSH)

The risk of any one construction being superseded within 12-24 months is HIGH. The risk of LAMPS Composite ML-DSA being superseded is LOW (it's the WG-blessed Schelling point with mandatory OpenPGP adoption per draft-ietf-openpgp-pqc-17 §"MUST implement ML-DSA-65+Ed25519").

### 5.4 Recovery path

The crypto-agility framing in CLAUDE.md baked-in #5 explicitly states "Any future algorithm... is added as an additive impl + already-reserved codepoint — never a wire-break, engine fork, or existing-content migration." This is well-architected.

Recovery path under LAMPS-default + later-SUF-add: mint `0x0002` for a Bird-of-Prey-derived codepoint when it stabilizes; existing v1-beta content continues to verify under `0x0001` indefinitely; new content can opt into `0x0002`.

Recovery path under Bird-of-Prey-default + flaw-found: mint `0x0002` for LAMPS fallback; existing v1-beta content under Bird-of-Prey `0x0001` would need to be **re-signed** under `0x0002` for forward security if the flaw is structural. This is more painful: per CLAUDE.md baked-in #5 there's NO migration of existing content — but the entire EXISTING content base would still verify under Bird-of-Prey, which is the broken codepoint. The codepoint cannot be dropped without wire break, but every relier on its security would need to know not to trust new signatures under it.

**The recovery cost asymmetry is real:** LAMPS-broken-later → migrate forward to SUF-CMA construction (clean). SUF-CMA-construction-broken-later → trust-erosion of existing v1-beta corpus.

---

## 6. Comparison to LAMPS-default + revoke-by-tuple alternative

This is the most important section in this review.

### 6.1 What does the L12 finding actually identify?

L12 finds three malleability hazards in the proposed LAMPS-default plan:
1. **UCAN revocation by sig-CID:** `Engine::revoke_capability_by_grant_cid(grant_cid)` — an attacker who produces a different sig-CID for the same grant payload bypasses revocation.
2. **Plugin manifest signatures (CID identity = sig-CID-derived):** multiplicity hazard if a plugin's "identity" is the CID of its signed manifest including the signature.
3. **Sync merge proofs + device attestations:** same shape.

### 6.2 Root cause analysis

The root cause is NOT the LAMPS algorithm. The root cause is **using the SIGNATURE CID as the identity/revocation key for the SIGNED PAYLOAD.** This conflates two separate things:
- **Payload identity** = should be the content-CID of the canonical payload BODY.
- **Authentication artifact** = the signature, which can be any one of multiple valid signatures over that payload.

EUF-CMA already provides the property "no new signature on a new message." It does NOT provide "no new signature on an old message." If the application uses signature identity as if SUF-CMA holds, that is an application-layer bug.

### 6.3 The architectural fix

Even with a SUF-CMA-preserving construction at the signature layer, **the right architecture is identity-by-payload-CID and revocation-by-payload-tuple, NOT identity-by-sig-CID.** This is because:

1. **Future migrations:** When Benten eventually rotates to a new codepoint (e.g., post-PQ-only NF-1 PQ⊕PQ), the SIGNATURE bytes change but the PAYLOAD does not. Identity-by-sig-CID forces re-keying the entire app-layer identity model at every algorithm rotation. Identity-by-payload-CID is stable across rotations.
2. **Re-signing semantics:** If a user re-signs the same UCAN under a rotated key, the OLD identity (sig-CID) becomes orphaned. The user's mental model is "this is the SAME capability, just re-signed." Identity-by-payload-CID matches the mental model.
3. **Revocation completeness:** Revoke-by-(issuer, subject, cap, audience, validity) tuple matches the SEMANTIC revocation target. Revoke-by-sig-CID is one level of indirection away from the actual revocation intent — even with SUF-CMA, a malicious issuer could re-sign with a NEW (cap, validity) tuple and the old sig-CID revocation would not catch the new one, because that "new one" is a legitimately-different UCAN.

In other words: **revoke-by-tuple is what UCAN semantics actually want. Revoke-by-sig-CID is a shortcut that BOTH (a) creates the malleability hazard L12 found AND (b) doesn't even correctly implement revocation under proper UCAN semantics.** Fixing the shortcut is the right primary mitigation regardless of signature algorithm choice.

### 6.4 What does the UCAN spec actually say?

Per ucan-wg/spec: "Revoked delegation should be referenced by its canonical CID." (UCAN.xyz revocation spec.) The UCAN spec itself uses CID, but the CID is the CID of the UCAN PAYLOAD (canonical CBOR/JSON of `{iss, aud, att, prf, exp, ...}`), NOT the CID of `{payload, signature}` bundle.

If Benten's `Engine::revoke_capability_by_grant_cid` is revoking by the CID of the PAYLOAD (which is what UCAN spec actually requires), then there is NO malleability hazard from LAMPS — the payload CID is deterministic across signature representations.

If Benten's implementation is revoking by the CID of the `{payload, signature}` bundle, that is a divergence from UCAN spec semantics that should be fixed regardless of algorithm choice.

**This is the single most important question for Ben to verify before any algorithm change:** does `grant_cid` in `Engine::revoke_capability_by_grant_cid` refer to the canonical-payload CID (UCAN-spec-aligned, no malleability) or the signature-bundle CID (UCAN-spec-divergent, malleability hazard)? If the former, the L12 finding is mis-stated and the algorithm doesn't need to change. If the latter, the fix is to switch to payload-CID, NOT to switch algorithms.

### 6.5 Algorithm-fix vs application-fix tradeoff

| Mitigation | Cost | Risk profile | Benten-fit |
|---|---|---|---|
| Switch v1-beta default to Bird-of-Prey | 4-8 weeks impl + audit + KAT authoring; new code base | Adds fresh academic crypto to v1-beta wire freeze; ml-dsa hazard surface UNCHANGED; new construction-specific bugs possible | BAD: violates "never fork/reimpl crypto"; outside team's expertise; standards-not-ready |
| Switch v1-beta default to draft-prabel §3 | 2-4 weeks impl + audit + KAT authoring | Simpler construction than Bird-of-Prey-2; still pre-WG-adoption | MARGINAL: violates "WG-blessed defaults"; ml-dsa hazard unchanged |
| Stay LAMPS-default + revoke-by-payload-CID | 0 work if already correct; ~3-5 day audit + fix if not | Aligns with UCAN spec; standards-blessed; ecosystem-interop | GOOD: matches existing posture, fixes root cause |
| Stay LAMPS-default + revoke-by-tuple | ~1-2 weeks app-layer rework | Stronger than revoke-by-payload-CID (covers cross-payload semantic equivalence); standards-blessed | EXCELLENT: closes both L12 finding AND latent UCAN-spec edge cases |

The math is clear: **fix the application layer.** Algorithm churn does not buy what algorithm churn appears to buy here.

### 6.6 The non-CID identity hazards (plugin manifests, device attestations)

For plugin manifests where "plugin identity = CID of signed-manifest":
- Same fix: define plugin identity as the CID of the MANIFEST PAYLOAD (canonical CBOR), not of the `{payload, signature}` bundle. Then signature malleability cannot produce a "different plugin with the same content" — because the identity is content, not signature.

For device attestations:
- Same fix: device identity = content-CID of attestation payload. Replay-by-malleability defeated by the application-layer replay defense (session-nonce in CLAUDE.md status: "session-nonce replay defense" already shipped per PR #163).

The L12 hazards are real but are architectural. Algorithm change does not close them; application-layer hygiene does, and the application-layer fix is the right shape regardless.

---

## 7. Risks + mitigations table

| Risk | Severity | Probability | Mitigation | Owner |
|---|---|---|---|---|
| ml-dsa Rust crate ships new vuln post-v1-beta-tag (RUSTSEC pattern) | HIGH | HIGH (3 ml-dsa CVEs in Jan-May 2026) | Pin version + 48h triage SLA + subscribe RustSec | Benten core |
| LAMPS draft changes wire format pre-RFC | MED | LOW (draft-19 stable; OID early-allocated) | Pin to OID 1.3.6.1.5.5.7.6.48 + draft revision in code comments | Benten core |
| Bird-of-Prey paper finds a flaw in EUROCRYPT'26 final review | MED | MED (preprint stage; one author) | Defer adoption until LNCS proceedings + 90-day cool-down | N/A if LAMPS-default |
| draft-prabel §4 RMV property turns out not to hold for ML-DSA | HIGH (if BoP-2 chosen) | LOW-MED (Janneck-novel property) | Use §3 generic construction only, not §4 | Benten core if BoP path |
| UCAN revoke-by-sig-CID malleability exploited at v1-beta | MED-HIGH | LOW (requires attacker to mint forgery + bypass app-layer checks) | Switch to revoke-by-payload-CID OR revoke-by-tuple at app layer | Benten core (mandatory pre-v1-beta) |
| Plugin manifest sig-CID identity conflated | MED | LOW (requires attacker to mint forgery on signed manifest) | Define plugin identity = manifest-payload-CID | Benten core |
| Audit cost overruns for new crypto construction | LOW | HIGH (BoP path) | Use LAMPS path; reuse planned ml-dsa/ml-kem audit | Project budget |
| Future migration cost if codepoint rotation needed | LOW | LOW (crypto-agility framework already in CLAUDE.md #5) | None; framework already exists | N/A |
| Trust-erosion of v1-beta corpus if default-codepoint broken | HIGH | LOW-MED (LAMPS path); MED (BoP path) | Choose well-vetted construction for default | Ben |
| Ecosystem-interop friction (PKIX / OpenPGP / SSH) | MED | LOW (LAMPS path); HIGH (non-LAMPS path) | Stay LAMPS-default for interop | Ben (positioning call) |
| Composite signature format mis-parsed by third-party tools | LOW | LOW (Composite ML-DSA has multi-impl interop) | Test against BouncyCastle + OpenSSL 3.5 fixtures | Benten core |

---

## 8. Codepoint layout recommendation

For v1-beta-tag wire-format freeze (recommended):

| Codepoint | Construction | Role | Status |
|---|---|---|---|
| `0x0001` | LAMPS Composite ML-DSA `id-MLDSA65-Ed25519-SHA512` (OID `1.3.6.1.5.5.7.6.48`) | **v1-beta DEFAULT** | LIVE at v1-beta |
| `0x0002` | Reserved for SUF-CMA-preserving hybrid (Bird-of-Prey / draft-prabel §3 / Silithium — TBD) | Future opt-in | typed-reject at v1-beta; mint when WG-adopted |
| `0x0003` | Classical Ed25519 | Opt-in classical-only fallback (e.g., interop with pre-PQ verifiers) | LIVE at v1-beta if needed |
| `0x0004` | Reserved for NF-1 PQ⊕PQ (ML-DSA-65 + SLH-DSA) | Future opt-in (already-named in CLAUDE.md #5) | typed-reject at v1-beta; mint when component impls mature |
| `0x0005` | Reserved for hash-only/Falcon/future | Future opt-in | typed-reject at v1-beta |

**Rationale for keeping `0x0001` = LAMPS:** matches the ecosystem Schelling point. OpenPGP-PQC RFC requires it. BouncyCastle, OpenSSL, AWS KMS all support it. Benten's plugin-manifest signatures, UCAN-grant signatures, device-attestation envelopes all interop naturally with externally-issued composite signatures.

**Rationale against placing Bird-of-Prey at `0x0001`:** at v1-beta-tag the construction is pre-WG-adoption, has no production Rust impl, no test vectors, one peer-reviewed publication still in proceedings preparation. Locking it into the default codepoint trades ecosystem-interop + standards-blessing for a theoretical security property the application layer ALREADY HAS A BETTER FIX FOR (revoke-by-payload-CID/tuple).

**If Ben overrides §1 and insists on a SUF-CMA-preserving default at v1-beta-tag:** then use draft-prabel §3 (NOT Bird-of-Prey-2) at `0x0001`, place LAMPS Composite ML-DSA at `0x0002` (preserving interop opt-in), and DEFER the v1-beta tag by 8-12 weeks for: external cryptographer review of the construction implementation + Rust wrapper + KAT authoring + cross-verification against a second independent implementation if available. This explicitly enlarges the C-GM-AUDIT scope per CLAUDE.md #5 / #15.

---

## 9. Honest disagreement

The directional framing in the orchestrator brief is: "Ben's tentative directional preference: switch v1-beta DEFAULT to a SUF-CMA-preserving construction (most likely Bird-of-Prey)."

**I disagree.** Reasons:

1. **The L12 finding identifies an APPLICATION-LAYER bug, not an ALGORITHM-LAYER bug.** Revoke-by-sig-CID is a Benten-application-layer shortcut that the UCAN spec itself does not require (UCAN spec uses payload-CID). The fix is to align with UCAN spec and revoke by payload-CID or by semantic tuple. Switching algorithms does NOT close the application-layer hazard if the same shortcut is reproduced elsewhere; aligning the application layer DOES close it permanently. **Fix the actual bug.**

2. **Bird-of-Prey is pre-standards.** It is one paper (EUROCRYPT 2026), one (combined) IETF individual draft, zero Rust reference implementations, zero test-vector corpora. The author is one person. The peer review is one conference cycle. v1-beta wire-format freeze is a structural commitment that locks in the choice. Locking in pre-standards crypto on a structural commitment is exactly the failure mode CLAUDE.md baked-in #5 warns against ("never fork, never reimplement, crypto primitives; integration crate is concat/hash/codepoint/envelope glue only").

3. **The ml-dsa Rust ecosystem is unstable.** Three ml-dsa-related advisories in 4 months (Jan-May 2026). Verification Theatre paper identifying 13 vulnerabilities in libcrux including FIPS 204 verifier violations. Adding a NEW Bird-of-Prey wrapper construction on top of this unstable foundation multiplies the audit + monitoring surface. Conservative: stick with the ecosystem-standard composite and put all audit eyes on the ml-dsa wrapper.

4. **The Bird-of-Prey size advantage is marginal.** The "smaller than sum" claim (BoP-2 / draft-prabel §4) saves ~32 bytes on a 3373-byte sig — a <1% size win at the cost of: non-black-box impl, requires unexposed ML-DSA "external mu" API, novel RMV property not standardized. The black-box variant (BoP-1 / draft-prabel §3) has the SAME size as LAMPS Composite ML-DSA (~3373 B). The only thing it buys is SUF-CMA. Which can be obtained instead by application-layer revoke-by-tuple.

5. **The ecosystem-interop argument is significant.** OpenPGP-PQC mandates `ML-DSA-65+Ed25519`. PKIX (LAMPS draft) ratifies it. JOSE (draft-prabel-jose) wraps it for web protocols. BouncyCastle 1.80+ ships it. OpenSSL 3.5 partial-supports it. Choosing Bird-of-Prey at v1-beta means Benten's signatures are NOT interoperable with any of these — a future Benten user wanting to verify a Benten capability inside a PKIX certificate workflow would need a custom verifier. This contradicts the project's "be a good citizen in the post-quantum transition" framing.

6. **The 5-week-old, 1-contributor framing is load-bearing.** Benten Engine is, per CLAUDE.md and the project state, a young project preparing its first tagged release. The right risk posture is to USE WG-BLESSED CONSTRUCTIONS and FOCUS ENGINEERING INVESTMENT on the application layer + the ml-dsa wrapper. Bringing fresh academic crypto into a v1-beta wire freeze on a project this young is a structural mistake — not because the crypto is bad, but because the project does not have the cryptographer-hours-on-staff to maintain it indefinitely under the inevitable next round of academic findings.

**The right architectural shape:** Ship LAMPS at `0x0001`. Fix revoke-by-payload-CID-or-tuple. Reserve `0x0002` with typed-reject. Plan the SUF-CMA upgrade path as a post-v1-beta codepoint addition (which the crypto-agility framework already supports without wire break). Use the audit budget on the ml-dsa wrapper + UCAN/manifest application layer, not on a third-party construction.

---

## 10. Evidence base (cite-anchored)

### LAMPS Composite ML-DSA
- **draft-ietf-lamps-pq-composite-sigs-19** (May 2026): https://datatracker.ietf.org/doc/draft-ietf-lamps-pq-composite-sigs/
- Section 9.2 Security Considerations: "Composite ML-DSA will be EUF-CMA secure if at least one of its component algorithms is EUF-CMA secure and the pre-hashed message representative PH is collision resistant."
- Section 9.2.2 SUF-CMA: **"While some of the algorithm combinations defined in this specification are likely to be SUF-CMA secure against classical adversaries, none are SUF-CMA secure against a quantum adversary."**
- **"Composite ML-DSA is NOT RECOMMENDED for use in applications where it has not been shown that EUF-CMA is acceptable."** (verbatim from Section 9.2.2)
- Section 8.1.2 IANA: OID `1.3.6.1.5.5.7.6.48` for `id-MLDSA65-Ed25519-SHA512`, early-allocated 2025-10-20.
- Composite signature size: ML-DSA-65 (3309 B) + Ed25519 (64 B) = **3373 bytes**. (Note: orchestrator brief said 3409; the correct figure is 3373.)
- Section 9 additional subsections: 9.3 Key Reuse; 9.4 Use of Prefix for attack mitigation; 9.5 Policy for Deprecated and Acceptable Algorithms.
- Domain separator: `"CompositeAlgorithmSignatures2025"` prefix.

### Bird of Prey / draft-prabel
- **Bird-of-Prey paper:** Janneck, "Bird of Prey: Practical Signature Combiners Preserving Strong Unforgeability," IACR ePrint 2025/1844, https://eprint.iacr.org/2025/1844. Accepted at **EUROCRYPT 2026** (not CRYPTO 2026 as orchestrator brief stated). Single author. Springer chapter https://link.springer.com/chapter/10.1007/978-3-032-25317-0_8 (paywall).
- **draft-prabel-cfrg-suf-hybrid-sigs-01** (March 2026): https://datatracker.ietf.org/doc/draft-prabel-cfrg-suf-hybrid-sigs/
- Authors: Lucas Prabel (Huawei), Guilin Wang (Huawei), Jonas Janneck (Ruhr Bochum, also BoP author), Tirumaleswar Reddy (Nokia), John Preuß Mattsson (Ericsson).
- **Status (verbatim from datatracker boilerplate):** "This Internet-Draft is not endorsed by the IETF and has no formal standing in the IETF standards process." Individual draft, not WG-adopted.
- Section 3 (Generic Binding): `s1 = Sign_1(sk1, m'); s2 = Sign_2(sk2, m' || s1); s = (s1 || s2)`. Black-box. Requires component 2 to be SUF-CMA.
- Section 4 (Fiat-Shamir non-black-box / BoP-2): requires component 2 to satisfy MBS + RMV. Non-black-box; opens ID-scheme primitives. Saves ~32 B on Ed25519+ML-DSA-65 by omitting classical commitment.
- Section 6.3 (Non-Separability): **"The hybrid construction in this document achieves WNS... However, SNS is not achieved."** (verbatim) — note: even Bird-of-Prey only achieves WNS, not SNS.
- Origin construction credited to **[BH23]** Bindel & Hale (IACR 2023/423, "A Note on Hybrid Signature Schemes," July 2023, https://eprint.iacr.org/2023/423).

### Silithium / IACR 2025/2059
- **IACR ePrint 2025/2059** (Nov 2025, last revision April 2026): Devevey, Guerreau, Roméas (PQShield), "Compact, Efficient and Non-Separable Hybrid Signatures." https://eprint.iacr.org/2025/2059
- Combines EC-Schnorr (classical) + ML-DSA (PQ) via Fiat-Shamir.
- Introduces "Hybrid EU-CMA" security notion (cross-protocol + separability + recombination attacks).
- **Requires ML-DSA implementation with "external μ" option** (per PQShield abstract). Not currently exposed in `libcrux-ml-dsa`, `fips204`, or `pqcrypto-mldsa` Rust crates.
- Targeted for PQCrypto 2026.

### Hybrid Signature Spectrums (formal definitions)
- **draft-ietf-pquip-hybrid-signature-spectrums-07** (multiple versions; published as informational): https://datatracker.ietf.org/doc/draft-ietf-pquip-hybrid-signature-spectrums/
- Defines WNS (Weak Non-Separability): "guarantee that an adversary cannot simply 'remove' one of the component signatures without evidence left behind."
- Defines SNS (Strong Non-Separability): "an adversary cannot take as input a hybrid signature... and output a valid component signature... that will verify correctly."
- Notes LAMPS Composite achieves WNS-via-message-label (1-out-of-n approved scheme); does NOT analyze it under BUFF.

### Rust ML-DSA implementation hazards (load-bearing)
- **RUSTSEC-2025-0144** (published 2026-01-27): `ml-dsa` Timing side-channel in Decompose. https://rustsec.org/advisories/RUSTSEC-2025-0144.html
- **GHSA-5x2r-hc65-25f9** (published 2026-01-27): `ml-dsa` accepts signatures with repeated hint indices (commit `b01c3b7` changed `<` to `<=`). https://github.com/RustCrypto/signatures/security/advisories/GHSA-5x2r-hc65-25f9
- **GHSA-fhvh-vw7h-9xf3** (published May 2026): `libcrux-ml-dsa` AVX2 `use_hint` mishandled edge case; fixed in 0.0.9. https://advisories.gitlab.com/cargo/libcrux-ml-dsa/GHSA-fhvh-vw7h-9xf3/
- **Verification Theatre (IACR 2026/192):** Identifies 13 vulnerabilities in libcrux + hpke-rs; V8 + V9 are FIPS 204 spec violations in ML-DSA verifier; affects all consumers including Google internal usage. https://eprint.iacr.org/2026/192

### Ecosystem
- **draft-ietf-openpgp-pqc-17** (2026): "A conformant implementation MUST implement ML-DSA-65+Ed25519 and ML-KEM-768+X25519." https://datatracker.ietf.org/doc/html/draft-ietf-openpgp-pqc
- BouncyCastle 1.80+ supports Composite ML-DSA in CMS SignedData (Java + .NET).
- OpenSSL 3.5 partial support for ML-DSA composite; AWS KMS post-quantum signature support; Thales Luna HSM ML-DSA programming guide.
- ANSSI position (per Synacktiv): "The Concat combiner, as the only generic construction recommended by ANSSI for signature combination."

### UCAN spec
- UCAN.xyz revocation spec: "Revoked delegation should be referenced by its canonical CID." UCANs are immutable but time-bound; store indexed by CID. (https://ucan.xyz/revocation/)

---

## 11. Extra-reflection-pass output: more elegant permanent shape

Per `feedback_extra_reflection_pass_for_elegant_permanent_shape`: after per-question triage, take a holistic pass for a single elegant structural shape that closes multiple findings at once.

### 11.1 The elegant shape

**Identity-by-canonical-payload-CID + Authentication-by-codepoint-dispatched-signature + Revocation-by-semantic-tuple.**

This three-layer decoupling is more elegant than any algorithm change because:

1. **Closes L12 finding entirely**: payload-CID is deterministic across signature representations; tuple-revocation matches semantic intent.
2. **Closes plugin-manifest multiplicity**: plugin identity = manifest-payload-CID, not bundle-CID.
3. **Closes device-attestation replay**: attestation identity = payload-CID; replay defense handled by session-nonce (already shipped).
4. **Closes UCAN-spec-divergence latent bug**: aligns with UCAN.xyz revocation spec.
5. **Decouples algorithm rotation from identity-layer changes**: when NF-1 PQ⊕PQ codepoint is added, plugin IDs / UCAN grant IDs do NOT change.
6. **Decouples application layer from cryptographic SUF-CMA assumption**: the application layer no longer NEEDS SUF-CMA from the signature algorithm. EUF-CMA + canonical-payload identity is sufficient for application correctness.
7. **Reduces v1-beta tag cryptographic risk surface**: standard LAMPS + ml-dsa-wrapper-audit is the entire crypto surface; no novel constructions.
8. **Preserves the option to upgrade to SUF-CMA later as a 2nd codepoint** without re-keying application identity or breaking wire format.

### 11.2 Why this is the actually-correct shape

Look at the L12 finding's full hazard list:
- UCAN revocation by sig-CID — APPLICATION-LAYER use of sig-CID as identity.
- Plugin manifest CID = sig-CID-derived — APPLICATION-LAYER use of sig-CID as identity.
- Sync merge proofs + device attestations — APPLICATION-LAYER same pattern.

**Every single hazard is the same architectural mistake at different sites: the application layer treats signature-CID as if it had SUF-CMA-equivalent uniqueness.** The right fix is to not do that — at every site, derive identity from the canonical payload. This is a one-time application-layer refactor of finite size that closes N findings + makes future crypto-rotations trivial.

Switching algorithms papers over the architectural mistake without fixing it (the next "use sig-CID as identity" pattern that appears WILL bite under any algorithm that loses SUF-CMA in some future setting, e.g. the NF-1 PQ⊕PQ codepoint's underlying assumptions).

### 11.3 Operational consequence

Replace the algorithm-change initiative with this engineering plan:

1. **Audit pass:** grep Benten codebase for every site using `sig_cid` / `signature.cid()` / similar patterns. Catalog the use cases.
2. **For each site:** classify as (a) legitimate use of sig-CID as opaque artifact identifier (rare; usually wrong); (b) misuse of sig-CID as semantic identity (most cases) → fix to canonical-payload-CID; (c) misuse of sig-CID as revocation key → fix to revoke-by-semantic-tuple.
3. **Ship this BEFORE v1-beta tag** as a "v1-beta hardening pass."
4. **Reserve `0x0002` codepoint** for future SUF-CMA-preserving construction (Bird-of-Prey when WG-adopted, OR Silithium when external-mu API lands in Rust, OR a future construction). Typed-reject at v1-beta.
5. **At post-v1-beta-tag:** revisit the SUF-CMA codepoint addition when standards land. No wire break required (crypto-agility framework already in CLAUDE.md #5).

This is more work in the application layer than just switching algorithms appears to be, but it (a) actually closes the hazards, (b) is reusable across future algorithm rotations, (c) does not lock in pre-standards crypto, (d) does not enlarge the audit budget, and (e) aligns with UCAN spec semantics that Benten already inherits.

---

## 12. Self-assessment

### Confidence level
- **HIGH confidence** on: (a) LAMPS Composite ML-DSA's WG-adoption status + OID + spec details; (b) draft-prabel's individual-draft status + construction details; (c) ml-dsa Rust ecosystem hazard surface (CVE catalog from RustSec); (d) the architectural argument that L12 finding is application-layer not algorithm-layer.
- **MEDIUM confidence** on: (a) exact Bird-of-Prey-2 size savings (paper paywalled; relied on derived sources); (b) Silithium's exact construction details (only abstract-level fetched); (c) draft-prabel §4 RMV property's standalone vetting status.
- **LOWER confidence** on: (a) whether Benten's `Engine::revoke_capability_by_grant_cid` actually uses payload-CID or bundle-CID (would need to read Benten source); (b) whether plugin manifests in Benten's current shipped code truly derive identity from sig-CID or from payload-CID. **Ben should verify these two questions before acting on any recommendation here.**

### Additional review I would want before commit
1. **Direct source-read of Benten's `Engine::revoke_capability_by_grant_cid`** and the plugin-manifest CID derivation. (My recommendations in §6.4 + §11 hinge on this.) Confirms or refutes the L12 finding shape at the application layer.
2. **A second independent cryptographer's read of the EUROCRYPT 2026 final-camera-ready version of Bird-of-Prey** if Ben overrides §1. The IACR preprint is what I had access to; the LNCS version may have changed.
3. **A vendor cryptographer review** (PQShield, Cure53, NCC Group, or Cryspen) of any chosen path. Their hourly cost is a real engineering investment, and at v1-beta-tag freeze the cost is justified.
4. **Pre-tag audit of the Rust wrapper around ml-dsa** regardless of which algorithm path is chosen. The wrapper layer is where most real-world bugs live (the FIPS 204 spec violations + the timing side-channels are wrapper-adjacent in their impact patterns).
5. **A read of LAMPS draft Section 9.4 ("Use of Prefix for attack mitigation")** which I did not get verbatim text for. May change the BUFF analysis slightly.

### What this review does NOT cover
- The X-Wing-style hybrid KEM (encryption side; CLAUDE.md #5). Out of scope; orthogonal decision.
- The Wave DID / Kith identity-recovery question. Mentioned in CLAUDE.md but orthogonal.
- The `0x0004` NF-1 PQ⊕PQ codepoint design. Stays as named-deferred in CLAUDE.md #5.
- Whether to ship classical-only `Ed25519` at `0x0003`. Recommended yes for interop; full justification deferred.

### Self-critique
I'm aware that the brief asked for a GO/NO-GO recommendation and Ben's tentative directional preference was GO-on-Bird-of-Prey. My CONDITIONAL NO-GO disagrees. I've tried to be transparent about that disagreement in §9 rather than equivocate. If my read of the application layer is wrong (i.e., if Benten ALREADY uses payload-CID and not bundle-CID, and revoke-by-tuple is unworkable for some structural reason I don't see), the §6 analysis collapses and the SUF-CMA path becomes more attractive. I would update this review on receipt of that information.

The recommendation in §1 should be read as: **"the conservative path serves the project better at v1-beta, AND there's a deeper architectural shape that closes the L12 hazards better than algorithm choice does." Not: "Bird-of-Prey is bad crypto."** Bird-of-Prey is good academic crypto. It's just the wrong commitment for this project at this moment in its lifecycle.

---

*End of review. Reviewer: senior cryptographer, engaged 2026-05-26.*
