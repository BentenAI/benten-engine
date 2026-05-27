# Option F+ §6.2 envelope-layer-unification — EXTRA-REFLECTION-PASS C1 (elegant-shape lens)

**Branch:** `phase-4-meta-core/option-f-plus-critique-c1-elegant-shape`
**Role:** EXTRA-REFLECTION-PASS holistic critic per `feedback_extra_reflection_pass_for_elegant_permanent_shape` (Ben-ratified 2026-05-25). NOT a 10th lens reviewer — looks at the consolidated WHOLE (28 amendments + 13+1 Compromises + 3 invariants) for single elegant structural shapes that close N-findings at once vs greedy-sum-of-N.
**Date:** 2026-05-27
**Tree-state pre-flight:** worktree clean @ `2172cb6d` against `origin/main`; fetched `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` as the input.
**Authority:** ADVISORY. Final decisions rest with Ben. Per `feedback_review_finding_ground_truth_verify`, downstream DISAGREE-WITH-EXPLANATION first-class — the consolidator's Q1-Q5 + 5 meta-findings are explicitly CONFIRMED / CONTESTED / REFINED below, not assumed.

---

## §1 Executive verdict + confidence

**Top-line.** The 28-amendment surface is **largely irreducible at the wire-format-axis** (each Group A-J amendment closes a structurally distinct attack class) but **HIGHLY reducible at the artifact/document axis**. The elegant-shape opportunities are not "fewer amendments" — they are "fewer artifacts" + "fewer Compromise rows" + "tighter invariant set". Concretely:

- **Wire-bit-affecting amendments: 22 → 22 (NO reduction).** Every wire-bit amendment closes a distinct attack — the consolidator's per-amendment triage holds. Trying to pack multiple wire amendments under a single shape sacrifices forward-class-of-bug closure (a generic "be careful" amendment cannot replace specific TLV-length-injectivity or specific epoch-binding).
- **Compromise rows: 13 new + 1 extension → 7 mints + 7 entries that are paragraphs within a parent.** Strong elegant-shape: organize the 13 mints into 4 "compromise families" (#31-Drop-permanence-family / #C-Operational-out-of-scope-family / #SC-Supply-chain-family / #43-Metadata-leak-family) where 7 of the 13 collapse to NARRATIVE sub-rows under a parent. See §5.
- **Invariants: 3 → 3 (CONFIRM consolidator; CONTEST self-doubt at §9.1).** Inv-16/17/18 are at the right granularity. Splitting Inv-18 into 18a/18b/18c (the consolidator's self-doubt) would REDUCE elegance not increase it; merging Inv-16 + Inv-17 would conflate two distinct properties (confidentiality-construction vs primitive-floor).
- **Audit-deliverable docs: 4-7 separate `.md` files → 1 canonical `docs/ENVELOPE-V1-SPEC.md` + 2 satellite docs (THREAT-MODEL.md, SECURITY-POSTURE.md). The R0 plan-doc § 6 + § 14 + Inv-18 + #43 disclosure + Codepoint registry all reference a single authoritative wire-format spec.** This is the largest forward-class-of-bug closure available — cite-drift between THREAT-MODEL ↔ CRYPTO-CODEPOINTS ↔ INTERNALS ↔ V1-FROZEN-INTERFACE is the recurring failure-mode (see `feedback_pim_cite_drift_fp1_recurrence` + 5 cross-language drift instances). One spec-doc-of-record reduces N×(N-1)/2 mirror-pairs to N×1.
- **`canonical_binding()` function: confirm consolidator's MF1-implicit but EXTEND.** A single canonical_binding() with discriminator+aad_version+TLV-body-length-prefix closes U1+U3+U14 mechanically (3 amendments → 1 implementation). U2 strict-decode is the read-side mirror of the same shape; U9/U10 #[non_exhaustive] makes the discriminator open-set-safe. SIX amendments (U1+U2+U3+U9+U10+U14) are best understood as ONE permanence-shape "Versioned-Discriminated-Length-Prefixed Wire Object", with the amendments being checklist-items the implementation MUST satisfy — not 6 independent moves.

**Final amendment count after elegant-shape consolidation: 28 amendments REMAIN AS IS in the registry** (the consolidator's enumeration is structurally honest — each amendment IS a distinct attack-closure). However, **for IMPLEMENTATION + DOC-DELIVERABLE purposes**, they group into:

- **6 amendments → 1 "Versioned-Discriminated Wire Shape" implementation package** (U1+U2+U3+U9+U10+U14)
- **2 amendments → 1 "Permanence-Reservation Block" v1-beta wave delivery** (U11 + U13; reserve all escape/range/MLS/CGKA/Bird-of-Prey codepoints in one PR)
- **4 amendments → 1 "AAD-Identity-Binding Package"** (U4 + U5 + U19 + U20; one canonical_binding-input-vec + one identity-DID schema + one generation-counter schema close all 4 mechanically)
- **3 amendments → 1 "Layer-C Drop-Bundle Package"** (U17 + U18 + U25; multi-stanza + dual-CID + per-recipient-unlinkable land together; U25 is enforced by U17 implementation discipline + U22 reservation slot)
- **3 amendments → 1 "Privacy-Surface Slot-Reserve Package"** (U22 + U23 + U24; all three reserve shapes at v1-beta with deferred implementation; ship all three reservations in one PR)
- **3 amendments → 1 "Codepoint-Registry Governance Package"** (U8 + U16 + U29; registry doc + lifecycle states + cross-ecosystem-mapping table; all three live in `docs/CRYPTO-CODEPOINTS.md` + the registry-discipline scanner)

**Net amendment count after IMPLEMENTATION packaging: ~13-15 PRs land 28 amendments.** That's the elegant-shape opportunity — not "fewer rows in the registry" but "fewer PRs in the wave plan".

**Highest-leverage NAMED-DEFERRED elegant alternatives:**
- **MLS-PQ adoption pre-v1-beta** (would replace U13 codepoint-bracket reservations + U22 Sealed-Sender + #42 FS-gap Compromise with one MLS additive shape). NAMED-DEFERRED with revisit-trigger "draft-ietf-mls-pq-ciphersuites lands at WG-LC + audit-firm signs off on MLS-PQ tractability for ~5-person team".
- **Sealed-Sender as v1-beta DEFAULT codepoint instead of additive slot** (would replace U22 reservation + U25 invariant + ~3 wave-days of metadata-leak Compromise-narrative writing with shipped privacy posture). NAMED-DEFERRED with revisit-trigger "Ben weighs marketing-posture-vs-engineering-scope-cost" — surfaced as §5 Q5 already.
- **Multi-base CID via multihash-multicodec composed (U30 DAG-CBOR + U15 multikey + U29 cross-ecosystem-map = one IPLD-native framing) replacing 3 separate amendments with one IPLD-conformance package.** Detailed in §3.7.

**Confidence:** **HIGH** that wire-amendments are irreducible; **HIGH** that doc-deliverables can collapse from 4-7 to 1+2; **MED-HIGH** that implementation can package 28 amendments into ~13-15 PRs (depends on R3/R5 wave-decomposition agent capacity); **MED** that the consolidator's self-doubt on Inv-18 should be REJECTED (Inv-18 merge is correct, splitting would FRAGMENT not clarify). **MEDIUM** that MLS-PQ-pre-v1-beta is worth a Ben sanity-check despite my NAMED-DEFERRED disposition.

---

## §2 Cluster analysis (28 amendments grouped by structural-property targeted)

I cluster the 28 amendments by **architectural property they target**, NOT by which lens originated them. This re-cuts the consolidator's Group A-K bucketing along a different axis — the goal is to surface "what shape would close all N amendments in this cluster at once?"

### Cluster 1 — Wire-shape construction soundness (6 amendments)

**Members:** U1 (codepoint-in-AAD), U2 (strict-decode), U3 (length-injective TLV), U9 (#[non_exhaustive] on EnvelopePayload), U10 (#[non_exhaustive] on BindingContext), U14 (aad_version byte).

**Architectural property:** "Future-additive wire object with codepoint-discriminated dispatch + canonical-encoding + version-tagged AAD." Each amendment is a CHECKLIST-ITEM on a single shape: the **Versioned-Discriminated-Length-Prefixed Wire Object** pattern.

**Elegant shape candidate — CANDIDATE A (CONFIRM consolidator MF3 + EXTEND):**

```rust
// One trait + one impl shape close all 6 amendments mechanically.

pub trait CanonicalEnvelope: Sized {
    const ENVELOPE_VERSION: u8;  // U14 aad_version

    fn codepoint(&self) -> CryptoCodepoint;  // U1 + U7 BE-encoded

    fn canonical_binding(&self) -> CanonicalBinding;  // U3 TLV; produced by trait derive macro

    fn strict_decode(bytes: &[u8]) -> Result<Self, EnvelopeError>;  // U2; trait method, NOT generic
}

#[non_exhaustive]  // U9
pub enum EnvelopePayload { ... }

#[non_exhaustive]  // U10
pub enum BindingContext { ... }
```

The implementation cost is the LARGER of the 6 amendments individually — once the trait + derive-macro + strict_decode pattern is shipped, U1/U3/U14 are derived MECHANICALLY (one TLV serializer; one prefix-byte; one codepoint-in-AAD discipline). U9/U10 are 2 LOC each. U2 is one match-arm-pattern enforced at the trait level. The "greedy sum of 6" reading would treat these as 6 distinct PRs; the elegant shape ships ONE permanence package PR that lands all 6 invariants + the trait derive-macro + the strict_decode discipline at once.

**Reduction:** 6 amendments still appear in the registry; **1 implementation PR** lands all 6 + the trait scaffolding. Pattern-matches MF3 (consolidator already named (a)(b)(c) for U9+U11+U14 as a permanence package). EXTENDS MF3 to include U1+U2+U3+U10.

**Forward-class-of-bug closure:** Future amendments that extend the canonical-encoding shape (e.g., a hypothetical U-N for ML-KEM-1024 codepoint OR a metadata-preserving variant) land as enum extensions to existing #[non_exhaustive] enums + derive-macro re-derivation, NOT as wire-format-break PRs.

**Pick winner:** **ELEGANT SHAPE WINS at implementation axis.** Greedy-sum-of-6 is NOT what to do; ship the trait + macro + strict_decode in one canary PR (per `feedback_canary_first_parallel_implementation`), then 5 amendments are line-items in that PR's commit body.

**NAMED-deferred alternative:** "Use serde + ciborium with a 1-line derive instead of bespoke trait + macro" — REJECTED for v1-beta because ciborium does not give compile-time aad_version-prefix enforcement + does not enforce strict-decode-by-codepoint-tag-dispatch (it deserializes by Rust enum tag, not by Benten codepoint). For v2 if DAG-CBOR (U30) ships AND IPLD-codec tagging covers Benten codepoints, REVISIT — could subsume U3 entirely.

### Cluster 2 — AAD identity-binding (4 amendments)

**Members:** U4 (sender-DID-in-AAD), U5 (sealed-at + valid-until epoch), U19 (recipient_key_generation), U20 (k_principal_generation).

**Architectural property:** "Identity + freshness binding into AAD for variant-specific BindingContext rows."

**Elegant shape candidate — CANDIDATE B:**

```rust
pub struct IdentityBinding {
    pub sender_did: Option<Did>,  // U4
    pub grantor_did: Option<Did>,  // U4 (DeviceLink + RemotePermission)
    pub requester_did: Option<Did>,  // U4
    pub sealed_at_epoch_hour: Option<u32>,  // U5 + U28
    pub valid_until_epoch_hour: Option<u32>,  // U5
    pub recipient_key_generation: Option<u32>,  // U19
    pub k_principal_generation: Option<u32>,  // U20
}

// Each BindingContext variant declares which IdentityBinding fields are MANDATORY:
impl BindingContext {
    pub fn required_identity_fields(&self) -> RequiredFields { ... }
}
```

The 4 amendments share ONE struct + variant-specific mandatory-field declaration. The cross-variant-substitution defense (each variant MUST validate its required fields at decode-time) is one match-arm per variant, not 4 independent enforcement sites.

**Reduction:** 4 amendments remain as registry rows (each is distinct property); **1 struct + variant-validator implementation** package covers all 4. Pairs naturally with Cluster 1's canonical_binding() (which serializes the IdentityBinding TLV-encoded into the AAD).

**Forward-class-of-bug closure:** Adding a 5th identity field (e.g., `executor_did` for U21 ExecuteWorkflow) is one Option<Did> field + one required_identity_fields() row, not a wire-format break.

**Pick winner:** **ELEGANT SHAPE WINS at implementation axis.** Same caveat — amendments stay in registry; implementation packaging consolidates them.

**Tension with Cluster 4 privacy:** U4 (plaintext sender-DID in AAD) directly creates #43 metadata-leak; the elegant resolution is U22 Sealed-Sender additive codepoint (Cluster 5), NOT removing U4. The consolidator MF1 already names this tension; the elegant shape codifies the resolution as "every IdentityBinding-containing BindingContext variant MUST have a paired Sealed-Sender-codepoint sibling" — which is Inv-18 clause (c) already. Confirm Inv-18 (c).

### Cluster 3 — Codepoint permanence + governance (5 amendments)

**Members:** U7 (BE endianness), U8 (registry doc + IANA-disjoint range), U11 (escape + experimental + EnvelopeShape axis), U13 (FS-gap codepoint brackets + Compromise mint), U16 (CodepointLifecycle typed-state).

**Architectural property:** "Codepoint as 4th first-class axis with explicit governance + lifecycle + escape paths."

**Elegant shape candidate — CANDIDATE C ("Codepoint Lifecycle Governance Module"):**

```rust
// crates/benten-crypto-suite/src/codepoint.rs
//
// Single source of truth for all crypto codepoints. Replaces scattered
// `pub const ... = 0x...;` definitions across aead.rs / hpke.rs / drops.rs.

pub struct CodepointRegistryEntry {
    pub value: u16,          // U7 BE-encoded on-wire; const-asserted disjoint
    pub variant_name: &'static str,
    pub layer: Layer,         // A/B/C/D
    pub primitive: PrimitiveChoice,
    pub aad_binding_spec: AadBindingSpec,
    pub since_version: SemVer,
    pub lifecycle: CodepointLifecycle,  // U16: Live/Deprecated/Quarantined/Burned
    pub cross_ecosystem: CrossEcosystemIdentifiers,  // U29 (Cluster 6)
    pub rationale: &'static str,
}

pub const CODEPOINT_REGISTRY: &[CodepointRegistryEntry] = &[
    // U8 LOAD-BEARING entries:
    CodepointRegistryEntry { value: 0x6100, variant_name: "LAYER_A_VAULT_XCHACHA20", ... },
    CodepointRegistryEntry { value: 0x6300, variant_name: "LAYER_C_DROP", ... },
    // U11 escape:
    CodepointRegistryEntry { value: 0xFFFF, variant_name: "ESCAPE_EXTENDED", lifecycle: Reserved, ... },
    // U13 reservations (cheap):
    CodepointRegistryEntry { value: 0x6380, variant_name: "RESERVED_MLS_APPLICATION", lifecycle: Reserved, ... },
    // ... etc.
];

// Compile-time const-assert disjoint + IANA-range-conformance + every public codepoint
// has registry entry (via cite-drift-detector scanner).
```

The 5 amendments compose as ONE module. U7's BE-endianness is a struct-field invariant (value: u16 always BE-encoded via canonical_binding); U8 is the registry array; U11 is registry rows for escape/experimental; U13 is registry rows for reservations + a Compromise mint linked to the FS-gap reservation row; U16 is the lifecycle enum + state-transition discipline.

**Reduction:** 5 amendments remain; **1 module + 1 doc (`docs/CRYPTO-CODEPOINTS.md`) + 1 cite-drift-scanner rule** package covers all 5. Each future codepoint mint is one registry row + one .md table row (paired discipline).

**Forward-class-of-bug closure:** Future codepoint mints (e.g., post-v1-beta MLS-PQ, FrodoKEM, ML-DSA-65) land as registry rows that auto-flow into `docs/CRYPTO-CODEPOINTS.md` + auto-validate via scanner + auto-receive lifecycle state. The "5 scattered constants in 5 modules with no central registry" failure-mode (current state per L4 IMPL-B1's LE/BE discovery) becomes impossible.

**Pick winner:** **ELEGANT SHAPE WINS** at both implementation + doc axes.

**NAMED-deferred alternative:** "Use existing multicodec.csv via cargo-multicodec crate" — REJECTED for v1-beta because multicodec.csv is upstream-multiformats-governed (Benten can't unilaterally add Layer-C/D/Drop-bundle codepoints there at v1-beta velocity); REVISIT post-v1-GM if Benten ships PR to multicodec.csv adding the Benten range.

### Cluster 4 — Privacy / metadata-leak (6 amendments)

**Members:** U22 (Sealed-Sender slot), U23 (per-relay-unlinkability), U24 (size-class padding), U25 (per-recipient-unlinkable multi-stanza), U26 (cover-traffic — NAMED-DEFERRED), U28 (coarse 1-hour epoch buckets refining U5).

**Architectural property:** "Reduce wire-observable metadata for adversaries with relay-access or long-term-archive correlation capability."

**Elegant shape candidate — CANDIDATE D ("Privacy-Surface Reservation Block"):**

The 4 v1-beta-shipping members (U22 + U23 + U24 + U25 + U28; U26 NAMED-DEFERRED) form ONE "Privacy-Surface Reservation Block" PR: register all codepoint slots + invariant clauses + V1-FROZEN-INTERFACE-DEFERRED.md rows at v1-beta, defer implementations to G-CORE-PRIVACY-1 wave.

- U22 codepoint slot: 1 registry row (Cluster 3's registry).
- U23 transport-layer shape: 1 V1-FROZEN-INTERFACE-DEFERRED.md row + 1 trait sketch in iroh-transport adapter crate.
- U24 size-class bucket: 1 const array + 1 V1-FROZEN-INTERFACE-DEFERRED.md row.
- U25 per-recipient-unlinkable: 1 invariant clause in Inv-18 metadata-disclosure section (already present).
- U28 coarse epoch: refines U5's u64 → u32 hour-bucket; 1 struct-field edit in IdentityBinding (Cluster 2).

**Reduction:** 5 of 6 ship as ONE coordinated PR (cheap; reservation-only). 1 NAMED-DEFERRED (U26). Pairs with #43 Compromise mint.

**Forward-class-of-bug closure:** All future privacy-amendment work (cover-traffic per U26 if shaped-relay lands; DID-rotation per U27; future Sphinx-onion-routing) extends this block additively rather than retrofitting wire format.

**Pick winner:** **ELEGANT SHAPE WINS at PR-coordination axis.** Each amendment closes a distinct privacy attack; shipping them together exploits the "all reservation-only" cost-symmetry.

**NAMED-deferred alternative:** "Sealed-Sender as v1-beta DEFAULT (not additive slot) — replaces U22 reservation + U25 invariant + ~3 wave-days of #43 narrative writing with shipped privacy posture." This is **§5 Q5** in the consolidator. **DISAGREE-WITH-EXPLANATION-LITE with consolidator's MED-leaning-L6:** the aggressive-privacy stance is more defensible than the consolidator credited. See §7 Q5 below for full reasoning. Still NAMED-DEFERRED here because the cost question is real; surface for Ben.

### Cluster 5 — Atrium-integration composition (3 amendments)

**Members:** U17 (HpkeMultiBase variant + cross-stanza defense), U18 (dual-CID), U21 (ExecuteWorkflow variant slot).

**Architectural property:** "Compose §6.2 envelope with Atrium peer-mesh content-identity semantics + future workflow execution."

**Elegant shape candidate:** Each of these is genuinely distinct (multi-recipient composition / content-vs-transport identity / new variant slot). They CANNOT collapse to a single shape without losing per-amendment property. However, they SHARE a delivery vehicle — the L9-A1/A2/A5 trio lands as ONE Drop-bundle-design PR + ONE Atrium-replication-test-suite + ONE workflow-variant-slot reservation.

**Reduction:** 3 amendments → **1 implementation PR** ("Layer-C Drop-Bundle + Atrium Composition"). Co-author with Cluster 2 (U19 recipient_key_generation is L9-A3 and lives natively in this cluster's bindings).

**Forward-class-of-bug closure:** Adding DropToGroup / MlsGroupApplication / CapabilityRevocation variants (per U10 #[non_exhaustive]) extends without breaking.

**Pick winner:** **PR-PACKAGING WINS at implementation axis.** Each amendment stays in registry; one PR delivers all three.

### Cluster 6 — Cross-ecosystem interop + outer framing (3 amendments)

**Members:** U15 (Did multikey-codepoint canonical encoding + Did::Unknown), U29 (cross-ecosystem-identifier-as-content emit-discipline), U30 (DAG-CBOR outer framing).

**Architectural property:** "Make Benten envelope cross-stack-recognizable + IPLD-natively-decodable + future-DID-method-tolerant."

**Elegant shape candidate — CANDIDATE E ("IPLD-Native Outer Framing"):**

If U30 (DAG-CBOR) lands as LOAD-BEARING (consolidator leans this, Ben call): the elegant shape uses **IPLD-multicodec composition** — every Benten codepoint either IS a multicodec entry (some are already: BLAKE3 = 0x1e) OR maps via U29's cross_ecosystem map to a multicodec — AND every `Did` is multikey-encoded per U15 — AND the outer EncryptedEnvelope is DAG-CBOR-wrapped with Benten-private CBOR-tag 0xBE54.

The result: **ONE wire-format-decoder works across Benten + IPLD-aware tooling + CAR-file backups + future TS port + multibase CID conversion.** The 3 amendments compose as one IPLD-conformance package; cite-drift between them is impossible (one source-of-truth = the multicodec.csv + multikey.csv + Benten's CrossEcosystemIdentifiers table).

**Reduction:** 3 amendments remain; **1 IPLD-conformance package PR** delivers all three.

**Forward-class-of-bug closure:** Future cross-ecosystem amendments (e.g., JWE-emit-adapter, COSE-emit-adapter, age-stanza-emit-adapter) land as table rows in `cross_ecosystem.rs`, not as new wire-format amendments.

**Pick winner:** **ELEGANT SHAPE WINS** *IF* U30 promoted to LOAD-BEARING (§5 Q4 + Q5-adjacent Ben call). If U30 stays RECOMMENDED-not-LOAD-BEARING, the elegant shape degrades to "U15 + U29 still pair; U30 free-standing."

**Recommendation:** Promote U30 to LOAD-BEARING (CONFIRM consolidator's lean). The IPLD-alignment payoff is substantial; the ~5% size cost is dominated by the cite-drift-elimination + cross-language-port-ease benefits. Plus Benten already commits to BLAKE3 CIDs (IPLD-native) — staying ad-hoc-binary at v1-beta wire freezes an inconsistency that becomes a permanent oddity.

### Cluster 7 — Impl-engineering crate choices (6 amendments)

**Members:** U31 (libcrux-ml-kem), U32 (XChaCha20-Poly1305), U33 (canonical_binding via NAPI), U34 (opaque-handle pattern at NAPI), U35 (wasm_js getrandom feature), U36 (Tokio cancel-safety).

**Architectural property:** "Implementation-only choices that close attack classes mechanically OR prevent runtime hazards."

**Elegant shape candidate:** No single structural shape. Each is a distinct crate-choice / API-pattern / config-line. They DO share **delivery-vehicle proximity** — all 6 land in 1-2 PRs (the "impl-engineering baseline" PR + the "NAPI binding" PR), but that's PR-grouping, not amendment-collapse.

**Reduction:** 6 amendments → **2 implementation PRs**. No registry-row collapse.

**Pick winner:** **GREEDY-SUM WINS** here. The amendments are genuinely independent.

### Cluster 8 — Audit-deliverables (4 amendments)

**Members:** U37 (golden-vector corpus), U38 (dudect CI), U39 (kani injectivity proof), U40 (THREAT-MODEL.md).

**Architectural property:** "Artifacts that external auditors and Benten future-maintainers need."

**Elegant shape candidate:** No single structural shape, but the docs (U40 + much of the other 3's test-output) collapse to **fewer docs than the registry implies** (see §3.6 below).

**Reduction:** 4 amendments → **2 PRs** (audit-test-infra + audit-doc-mint).

**Pick winner:** **PR-PACKAGING WINS.** Standalone amendments preserved in registry.

### Cluster 9 — Forward-Secrecy / Compromise-disclosure (1 amendment + cross-cuts)

**Member:** U6 (Bernstein-Persichetti CT-Decap mitigation + #32 mint).

**Architectural property:** Cross-cutting; closes via U31 libcrux choice (Cluster 7). #32 mint is doc-only.

**Reduction:** No collapse needed; folds into Cluster 7's libcrux PR + Cluster 8's audit-doc PR.

### Cluster summary table

| Cluster | Amendments | Elegant-shape winner | Reduction (PRs) | Reduction (registry rows) |
|---|---|---|---|---|
| 1 Wire-shape soundness | U1, U2, U3, U9, U10, U14 | Canonical Trait+Macro | 6 → 1 PR | 6 → 6 (no row collapse; SHIP-AS-PACKAGE) |
| 2 AAD identity-binding | U4, U5, U19, U20 | IdentityBinding struct | 4 → 1 PR | 4 → 4 (no row collapse) |
| 3 Codepoint governance | U7, U8, U11, U13, U16 | CodepointRegistry module | 5 → 1 PR | 5 → 5 (no row collapse) |
| 4 Privacy / metadata | U22, U23, U24, U25, U28 (+U26 NAMED-DEF) | Reservation Block PR | 5 → 1 PR | 5 → 5 (no row collapse) |
| 5 Atrium composition | U17, U18, U21 | PR packaging | 3 → 1 PR | 3 → 3 |
| 6 Cross-ecosystem | U15, U29, U30 | IPLD-conformance package | 3 → 1 PR | 3 → 3 |
| 7 Impl-engineering | U31, U32, U33, U34, U35, U36 | greedy-sum | 6 → 2 PR | no collapse |
| 8 Audit deliverables | U37, U38, U39, U40 | PR packaging | 4 → 2 PR | no row collapse but 4 docs → ~1+2 (§3.6) |
| 9 Cross-cut | U6 (folds) | folds | 0 net | folds |
| **Totals** | **28 amendments (+#26 deferred) + #27 deferred** | — | **~12 implementation PRs** | **28 rows preserved** |

**Headline:** Registry stays at 28 (each row IS a distinct attack-closure); wave plan compresses from "potentially 28 PRs" to **~12 implementation PRs**. That's the elegant-shape reduction at the delivery axis.

---

## §3 Elegant-shape candidates beyond consolidator's meta-finding (c)

### §3.1 Single canonical_binding() with sufficient input vec (consolidator MF1-implicit; CONFIRMED + STRENGTHENED)

**Already implicit in consolidator §2 Group A/B narrative.** Explicit shape:

```rust
fn canonical_binding(
    codepoint: CryptoCodepoint,            // U1
    aad_version: u8,                        // U14
    binding_context: &BindingContext,       // U10 #[non_exhaustive] dispatch
    identity_binding: &IdentityBinding,     // U4 + U5 + U19 + U20 (Cluster 2)
) -> CanonicalBinding {
    // TLV-length-prefix every variable-length field (U3)
    // BE-encode all multi-byte ints (U7)
    // strict-decode = inverse function with codepoint-tag-dispatched variant lookup (U2)
    // #[non_exhaustive] enum tag-byte registry per Cluster 3
}
```

**Closes:** U1 + U2 + U3 + U4 + U5 + U7 + U14 + U19 + U20 (NINE amendments compose via one function with a sufficient input vec).

**Wire-shape cost:** ZERO (this IS the wire shape; the amendments are properties OF this function's correctness).

**Forward-class-of-bug closure:** Any future amendment that adds an AAD-bound field becomes "add a field to IdentityBinding OR a variant to BindingContext, regenerate canonical_binding via derive-macro" — not a wire-format break + not a separate audit-property to track.

**Recommendation:** **CONFIRM the implicit consolidator framing + ELEVATE to explicit Cluster 1 + Cluster 2 implementation contract.** The R0 plan-doc §6 EncryptedEnvelope section should be rewritten around `canonical_binding(codepoint, aad_version, binding_context, identity_binding)` as the singular wire-construction primitive.

### §3.2 Single `EncryptedEnvelope-v1` wire-format SPEC doc (CONTEST consolidator's 4-7 doc scatter)

The consolidator's §7 R0 skeleton + §6 disposition implies 4-7 separate `.md` files:
- `docs/CRYPTO-CODEPOINTS.md` (new; U8 + U11 + U13 + U15 + U16 + U22 + U29)
- `docs/THREAT-MODEL.md` (new; U40)
- `docs/SECURITY-POSTURE.md` (existing; extended with #32-#44 + #31 ext + #42 + #43)
- `docs/KEY-LIFECYCLE.md` (existing or new; U4 + U5 + U19 + U20)
- `docs/INVARIANT-COVERAGE.md` (existing; Inv-16 + Inv-17 + Inv-18)
- `docs/V1-FROZEN-INTERFACE.md` (existing; item 6 amendments)
- `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (existing; Row D-SS-1 + D-PAD-1 + D-COVER-1)
- `docs/INTERNALS.md` (existing; aead.rs commentary)

**Cite-drift surface among 7 docs = 7×6/2 = 21 pairwise mirror-relationships.** Per `feedback_pim_cite_drift_fp1_recurrence` + `feedback_pim_cross_language_rule_mirror` (5 confirmed instances at HEAD): this is the recurring failure-mode for Benten doc work.

**Elegant shape — CANDIDATE F ("ENVELOPE-V1-SPEC.md as authoritative root"):**

Mint **ONE canonical `docs/ENVELOPE-V1-SPEC.md`** as the authoritative wire-format spec, covering:
- §1 EncryptedEnvelope outer shape + DAG-CBOR-tag + codepoint dispatch (Cluster 1+3+6)
- §2 BindingContext variants + IdentityBinding mandatory-field matrix (Cluster 1+2)
- §3 Codepoint registry (all rows; the canonical table; was `CRYPTO-CODEPOINTS.md`)
- §4 Cross-ecosystem-identifier translation table (Cluster 6 U29)
- §5 Lifecycle state semantics (U16)
- §6 v1-beta-DEFERRED reservation slots (U22 + U23 + U24 + U25's invariant + ExecuteWorkflow + MLS-PQ + CGKA + Bird-of-Prey)
- §7 Compromise # backlinks to SECURITY-POSTURE.md
- §8 Invariant backlinks to INVARIANT-COVERAGE.md (Inv-16/17/18)

Keep ONLY 2 satellite docs:
- `docs/SECURITY-POSTURE.md` (existing) — extended with #32-#44 + #31 ext; backlink TO `ENVELOPE-V1-SPEC.md` for technical context.
- `docs/THREAT-MODEL.md` (new per U40) — adversary classes + IN/OUT/PARTIAL matrix; backlink TO `ENVELOPE-V1-SPEC.md` for primitive details.

Existing `INVARIANT-COVERAGE.md` + `V1-FROZEN-INTERFACE.md` + `V1-FROZEN-INTERFACE-DEFERRED.md` + `INTERNALS.md` get **one-line ENVELOPE-V1-SPEC.md pointer** instead of duplicated content.

**Reduction:** 7-8 doc files → **1 authoritative + 2 satellites + N existing files with pointers**. Cite-drift surface collapses from 21 pairwise → 3 pairwise (root + 2 satellites).

**Forward-class-of-bug closure:** Future amendments to envelope shape edit ONE doc; backlinks remain stable. The `feedback_pim_cite_drift_fp1_recurrence` failure class becomes structurally impossible for envelope-shape concerns.

**Pick winner:** **ELEGANT SHAPE WINS at doc-axis with HIGH confidence.** This is the **single highest-leverage elegant reduction in the entire registry**.

**NAMED-deferred alternative:** "Keep 7-doc scatter for lens-origin-cleanliness" — REJECTED. Lens-origin cleanliness is a process-internal property; cite-drift-elimination is a forward-class-of-bug closure with measured prior-art (5 confirmed instances).

### §3.3 "CompromiseFamily" pattern (CONTEST consolidator's 13-independent-mints framing)

The consolidator mints 13 new Compromises (#32-#44) + 1 #31 extension as **flat siblings**. The consolidator's MF4 already notes 4 of these (#32 + #35 + #43 + #42) cross-link to Compromise #31 as upstream. **Strengthen this finding:** the 13 mints fall into ~4 natural FAMILIES:

#### Family 1 — Drop-permanence-cascade family (cross-links to #31)
- **Parent:** Compromise #31 (existing, forever-valid drops).
- **Sub-rows:** #32 (Bernstein-Persichetti Decap CCA scales with replay opportunity), #35 (compromised-device retroactive decryption), #42 (Layer-C FS-gap), #43 (metadata-leak amplified by archive), #41 (cross-device-sync UX boundary).
- **Elegant shape:** ONE "Drop-permanence cascade" section in SECURITY-POSTURE.md with #31 as parent + 5 sub-headings naming the distinct downstream hazards + ONE diagram showing the cascade tree.
- **vs flat-13:** 5 of 13 mints become NARRATIVE SECTIONS under #31, not independent Compromise numbers. Counts drop to **#31 (existing, extended) + #32 (named PARENT-MITIGATION; libcrux closes mechanically) + #33-#34 + #36-#40 + #44 = 8 independent mints**. (#32 stays independent because its closure mechanism is crate-choice, not narrative.)

#### Family 2 — Operational-out-of-scope family
- **Parent:** Mint as **#33-family** "Operational/non-cryptographic boundaries" (umbrella).
- **Sub-rows:** #33 coercion, #34 password-knowledge, #36 RAM-residency, #37 no TEE, #38 physical side-channels.
- **Elegant shape:** ONE umbrella section "What Benten does NOT promise (operational/physical/duress threats)" with 5 sub-bullets.
- **vs flat-5:** Becomes **1 umbrella + 5 narrative sub-rows**. Counts as **#33 umbrella mint + sub-disclosures**.

#### Family 3 — Supply-chain-and-build family
- **Parent:** Mint as **#39-family** "Supply-chain + build posture".
- **Sub-rows:** #39 dep-pinning, #40 no reproducible builds.
- **Elegant shape:** ONE narrative section.
- **vs flat-2:** 1 umbrella mint, 1 narrative sub-row.

#### Family 4 — Long-term-cryptanalysis family
- **Parent:** **#44** BSI long-term-confidentiality.
- **Sub-rows:** none yet; future FrodoKEM / Classic-McEliece / hash-based-sig-only stories live here.
- **Elegant shape:** standalone mint #44 with named extension path.

**Reduction:** 13 new mints + 1 extension → **6 mint numbers** (#31 ext, #32, #33-family, #39-family, #42, #43, #44) + 7 narrative sub-rows under their parent. **8 fewer Compromise # references in cross-doc citations.**

**Forward-class-of-bug closure:** Future Compromise rows attach to the right family rather than landing as flat-N+15. The "we have 50 Compromises and no one can find which one applies" failure-mode for v1-GM external-audit gets prevented.

**Pick winner:** **ELEGANT SHAPE WINS at narrative axis.** Family-grouping is how external auditors read security-posture docs (per L5's age + Obsidian-Sync + Common-Criteria-ST shape precedent).

**NAMED-deferred alternative:** "Keep flat-13 mints with explicit MF4-style cross-link table at end of SECURITY-POSTURE.md" — REJECTED-but-VIABLE-FALLBACK. The flat-13 + cross-link-table approach is one row shorter to count but loses the narrative-grouping benefit. Ben call if family-mint feels over-engineered.

### §3.4 Could one "primitive-neutral" Inv-16 absorb Inv-17 + Inv-18?

**CONTEST.** The consolidator's self-doubt §9.1 frames Inv-18 as possibly-over-consolidated; consider also the dual:

- **Inv-16** = envelope-layer-unification + codepoint-discriminated primitive + AAD-binding + strict-decode + canonical-TLV + sender-DID + replay-window + ... (CONFIDENTIALITY-CONSTRUCTION property)
- **Inv-17** = PQ-classical-hybrid mandatory (KEM-PRIMITIVE-FLOOR property)
- **Inv-18** = codepoint-registry-discipline + metadata-disclosure + CodepointLifecycle (GOVERNANCE property)

These are at **3 different abstraction layers:**
- Inv-16 = construction-correctness ("when you build an envelope, here's what it looks like")
- Inv-17 = primitive-floor ("when you pick a KEM, here's the minimum bar")
- Inv-18 = process-discipline ("when you mint a codepoint, here's what governance applies")

**Merging Inv-16 + Inv-17 would conflate "what an envelope IS" with "what primitives ARE ACCEPTABLE INSIDE the envelope" — these are orthogonal.** The PQ-mandatory floor is a primitive-choice constraint that's enforced at codepoint-mint-time (via Inv-18 governance) and prevents flat-classical-only codepoint variants — but it's not a property of the envelope shape itself.

**Splitting Inv-18 into 18a/18b/18c (consolidator's self-doubt #1) would FRAGMENT** because:
- (a) registry-discipline, (b) metadata-disclosure pairing, (c) lifecycle-state are **THREE faces of the same governance property** ("how do we control + document + retire codepoints"). They're not 3 properties; they're 3 enforcement-rules for ONE property.

**Recommendation:** **CONFIRM Inv-16 + Inv-17 + Inv-18 as 3 invariants at proper granularity. REJECT consolidator's self-doubt at §9.1 item 1.**

**NAMED-deferred alternative:** "Mint Inv-19 ('cross-ecosystem identifier-as-content emit discipline') extracted from Inv-18 if §3.5s discipline grows substantial enough to warrant own invariant." Currently §3.5s is dispatch-conventions discipline; if it becomes load-bearing-protocol-spec-discipline, promote to Inv-19. NAMED-DEFERRED with revisit-trigger "3+ cross-ecosystem adapters land + §3.5s rule fires >5 times."

### §3.5 Could one Layer-C+D unified "external-cryptographic-grant" pattern absorb 4 variants?

The current BindingContext variants for Layer-C/D:
- `DropToRecipient` (Layer-C: encrypt-to-recipient)
- `DeviceLink` (Layer-D: device-onboarding)
- `RemotePermission` (Layer-D: remote-grant)
- `ExecuteWorkflow` (Layer-D: U21 hyper-scaling)

**They share a structural template:**
- Grantor identity (sender_did + grantor_did)
- Recipient/target identity (recipient_did OR target_device_did)
- Operation scope (operation_type / max_decrypt_count / workflow_cid)
- Validity window (sealed_at + valid_until)
- Recipient key generation (recipient_key_generation)

**Elegant shape candidate — CANDIDATE G ("ExternalCryptographicGrant"):**

```rust
#[non_exhaustive]
pub enum BindingContext {
    Vault { ... },
    PerNodeAead { ... },
    ExternalGrant(ExternalCryptographicGrant),
}

pub struct ExternalCryptographicGrant {
    pub grant_kind: GrantKind,  // DropToRecipient | DeviceLink | RemotePermission | ExecuteWorkflow
    pub grantor: IdentityBinding,  // Cluster 2 shape
    pub target: GrantTarget,
    pub scope: GrantScope,
    pub validity: ValidityWindow,
    pub recipient_key_generation: u32,  // U19
}
```

**Closes:** U4 (sender-DID for all 4 variants uniformly) + U5 (validity window uniformly) + U19 (recipient-key-generation uniformly) + U21 (ExecuteWorkflow is just a GrantKind variant + GrantScope::Workflow shape) — **3 amendments + 1 variant slot merge into ONE shape.**

**vs flat-4-variants:** The current Group F design has 4 BindingContext enum variants with mostly-overlapping fields + per-variant required_identity_fields() machinery. The elegant shape has 1 outer enum variant + 1 GrantKind discriminator + shared structure.

**CRITIQUE OF ELEGANT SHAPE:** This is a CANDIDATE but I am NOT confident-recommending it. Counter-arguments:
- Variant-specific fields exist (ExecuteWorkflow has workflow_cid + input_node_cids + max_decrypt_count; DeviceLink has device_attestation; DropToRecipient has multi-stanza CEK-wrapping). Forcing into one struct creates Option<...> proliferation.
- The codepoint discipline + cross-ecosystem-identifier discipline benefits from each variant having a DISTINCT codepoint that adapters map cleanly; merging variants risks codepoint dispatch ambiguity.
- The Cluster 1 #[non_exhaustive] discipline already handles future-additivity; CompactGrant doesn't gain there.

**Recommendation: NAMED-DEFERRED elegant alternative.** Revisit at R0 §4 design when full Layer-C/D wire format is pinned. If variant-specific fields are ≤2 per variant (low Option<...> proliferation), elegant shape wins. If variant-specific fields are ≥4 per variant, flat-4-variants wins. Surface to R0 design author as explicit decision-point.

**Revisit trigger:** R0 §4 Layer-C/D design author writes out full struct layouts for all 4 variants; count Option<...> field ratio; pick winner.

### §3.6 Codepoint Lifecycle Discipline as ONE pattern

**Already CANDIDATE C in §2 Cluster 3.** Confirms consolidator's MF3 + EXTENDS by including U7 (BE) + U8 (registry) + U13 (FS-gap brackets) + U29 (cross-ecosystem mapping) under one Codepoint module. See §2 Cluster 3.

### §3.7 IPLD-native composition (multibase CID + multicodec + multikey + DAG-CBOR)

**CANDIDATE E from §2 Cluster 6.** Confirms — if U30 promoted to LOAD-BEARING (recommended), then U15 + U29 + U30 merge into ONE IPLD-conformance package.

### §3.8 MLS-PQ adoption as future-additive replaces 4 amendments at v2

**NAMED-DEFERRED elegant alternative.** Currently the v1-beta plan reserves codepoint brackets for MLS-Application (U13) + MLS-Welcome (U13) + CGKA-Commit (U13) + Bird-of-Prey AKEM (U13) + ships HPKE-mode-base[X-Wing] as default Layer-C/D primitive. If draft-ietf-mls-pq-ciphersuites-04 matures to WG-LC AND tractability for a ~5-person team is validated, MLS-PQ could SUBSUME:
- U13 codepoint-bracket reservations (already-reserved becomes already-implemented)
- U22 Sealed-Sender (MLS group-application natively hides sender to non-group-members)
- #42 FS-gap (MLS CGKA ratcheting closes structural FS)
- #43 metadata-leak (MLS group-as-anonymity-set reduces per-pair-correlation)
- U25 per-recipient-unlinkable (CGKA already provides per-recipient framing)

**FIVE amendments + 2 Compromise mints' load-bearing weight reduces** if MLS-PQ ships pre-v1-beta.

**Why NAMED-DEFERRED not RECOMMENDED:** draft-ietf-mls-pq-ciphersuites-04 is not WG-LC; library support is nascent (no Rust impl with verified PQ-CGKA at audit-quality); ~5-person team scope-fit untested. The "absorbs 5 amendments + 2 Compromises" upside is real but the "needs 6+ months MLS-PQ ecosystem maturation + ~20 wave-days MLS integration" downside dominates at current calendar.

**Revisit trigger:** "draft-ietf-mls-pq-ciphersuites lands at WG-LC OR Rust MLS-PQ library achieves audit-firm sign-off OR Benten team grows past 10 engineers." Re-evaluate elegant shape at that trigger.

### §3.9 Bernstein-Persichetti CT-Decap via libcrux-ml-kem mechanically closes Compromise #32

**CONFIRM consolidator.** U6 + U31 + Compromise #32 fold into ONE crate-choice. The Compromise mint stays because **honest disclosure** is the right discipline even when the closure is mechanical — but the engineering-action is one Cargo.toml line.

**Reduction:** 2 amendments (U6 + U31) + 1 Compromise → ONE Cargo.toml edit + 1 SECURITY-POSTURE.md narrative section. Already captured in Cluster 7 elegant-shape.

### §3.10 DAG-CBOR + multicodec-tag + Did multikey resolve 3 L7/L8 amendments

**CONFIRM §3.7 + §2 Cluster 6.** This is the same elegant-shape under different name. CANDIDATE E in §2 + §3.7 are the canonical statement.

---

## §4 Reduction recommendation (final count + LOC/complexity delta)

### Net registry-row count after elegant-shape consolidation

| Surface | Pre-consolidation | Post-consolidation | Delta |
|---|---|---|---|
| Wire-affecting amendments | 22 | 22 (irreducible at registry-row axis) | 0 |
| Codepoint-reserve amendments | 6 | 6 (irreducible at registry-row axis) | 0 |
| Impl-only amendments | 8 (U30-U40 minus U30) | 8 | 0 |
| NAMED-deferred | 2 (U26, U27) | 2 | 0 |
| **TOTAL AMENDMENTS** | **28** | **28** | **0 (registry-row axis)** |
| Implementation PRs | ~28 (if naively 1-per-amendment) | **~12** | **-16 PRs / -57%** |
| Authoritative doc files | 7-8 | **3** (ENVELOPE-V1-SPEC + SECURITY-POSTURE + THREAT-MODEL) | **-4 to -5 docs / -57% to -63%** |
| Cite-drift pairwise mirror surface | 21 | 3 | **-18 pairs / -86%** |
| Compromise # mints | 13 new + 1 ext | **6 mints + 7 narrative sub-rows** | **-7 mint numbers (50% fewer cross-doc references)** |
| Invariants | 3 | **3 (CONFIRM; reject consolidator self-doubt)** | 0 |

### LOC / complexity delta

| Axis | Greedy-sum-of-N | Elegant-shape | Delta |
|---|---|---|---|
| Rust production LOC | ~2730 (L4 estimate) | ~2400-2600 (one canonical_binding + one CodepointRegistry replaces ~300-400 LOC of scattered impl) | -100 to -300 LOC |
| Rust test LOC | ~4200 (L4 estimate) | ~3800-4000 (one canonical_binding property test covers 6 amendments) | -200 to -400 LOC |
| Wave-days for impl | ~17.5 (L4) | ~14-16 (PR-packaging amortizes shared scaffolding) | -1.5 to -3.5 wave-days |
| Wave-days for docs | ~2 (L4) + ~4.25 (L5) = ~6.25 | ~4 (single ENVELOPE-V1-SPEC + 2 satellites) | -2 to -2.5 wave-days |
| Cite-drift fix-pass risk | 21 mirror-pairs × ~0.3 instances/year = ~6 instances expected | 3 mirror-pairs × ~0.3 = ~1 instance expected | **-5 cite-drift fix-passes/year saved** |

**Total wave-day saving: ~3-6 wave-days net at v1-beta** (small but real). **Larger ongoing saving: ~5 cite-drift fix-pass cycles/year saved post-v1-beta** (per existing 5-confirmed-instances at HEAD baseline).

**v1-beta calendar estimate:** consolidator's ~65-72 wave-days = ~13-15 weeks → elegant-shape-applied **~62-68 wave-days = ~12-14 weeks**. Still fits 7-15 week window comfortably.

### Critical winner picks

For each cluster, the WINNER:

| Cluster | Winner | Rationale |
|---|---|---|
| 1 Wire-shape soundness | **Elegant: Canonical Trait + Macro PR** | 6 amendments compose mechanically; forward-additivity at zero cost |
| 2 AAD identity-binding | **Elegant: IdentityBinding struct** | 4 amendments share input vec; per-variant required_fields() one match-arm |
| 3 Codepoint governance | **Elegant: CodepointRegistry module** | 5 amendments share registry source-of-truth; cite-drift impossible |
| 4 Privacy / metadata | **Elegant: Reservation Block PR** | 5 amendments are all reservation-shape; coordinate cost-symmetry |
| 5 Atrium composition | **Elegant: PR packaging** | 3 amendments share Drop-bundle delivery vehicle |
| 6 Cross-ecosystem | **Elegant: IPLD-conformance package** | IF U30 LOAD-BEARING (recommended) — 3 amendments compose IPLD-natively |
| 7 Impl-engineering | **Greedy-sum** | Genuinely independent; 2 PRs reasonable |
| 8 Audit deliverables | **Elegant: ENVELOPE-V1-SPEC root + 2 satellites** | 4-7 docs → 3; -86% cite-drift surface |
| 9 Cross-cut | Folds | — |

### NAMED-DEFERRED elegant alternatives

| Alternative | Closes | Status | Revisit trigger |
|---|---|---|---|
| MLS-PQ pre-v1-beta adoption | U13 reservations + U22 + #42 + #43 partial + U25 | NAMED-DEFERRED | draft-ietf-mls-pq-ciphersuites WG-LC OR audit-quality Rust MLS-PQ lib OR team ≥10 |
| Sealed-Sender as v1-beta DEFAULT | U22 reservation → ship implementation | NAMED-DEFERRED (§5 Q5) | Ben weighs marketing-vs-scope; recommendation: CONTEST consolidator MED-leaning-L6 — see §7 Q5 |
| Single ExternalCryptographicGrant variant | U4 + U5 + U19 + U21 unification | NAMED-DEFERRED | R0 §4 design author counts Option<...> field ratio per variant |
| Inv-19 cross-ecosystem extracted from Inv-18 | §3.5s discipline becomes own invariant | NAMED-DEFERRED | 3+ adapters land + §3.5s rule fires >5 times |
| ciborium-derive for canonical_binding | U3 simplification | NAMED-DEFERRED | DAG-CBOR (U30) v2 + IPLD-codec tagging covers Benten codepoints |
| Multicodec.csv upstream PR for Benten range | U8 IANA-disjoint-ness becomes ecosystem-conformant | NAMED-DEFERRED | post-v1-GM; Benten ships PR to multicodec.csv adding 0x6100..0x6FFF range |

---

## §5 Compromise # consolidation candidates (#31-family vs independent #32-#44)

**Per §3.3 family-grouping analysis.** Recommendation:

### CONSOLIDATED MINT SET (post-elegant-shape):

| Mint # | Family / Standalone | Closure / sub-rows |
|---|---|---|
| **#31** (existing, EXTENDED) | **Drop-permanence parent** | + L9-A3 retention-window clarification + cross-link tree to #32, #35, #41, #42, #43 |
| **#32** | Drop-permanence sub, but standalone (closure is crate-choice) | Bernstein-Persichetti Decap CCA; closed mechanically by libcrux U31 |
| **#33** | **Operational-out-of-scope umbrella** | Sub-rows: coercion / password-knowledge / RAM-residency / no-TEE / physical-side-channels (was #33+#34+#36+#37+#38 individually) |
| **#39** | **Supply-chain umbrella** | Sub-rows: dep-pinning / no-reproducible-builds (was #39+#40 individually) |
| **#42** | Drop-permanence sub | Layer-C FS-gap; cross-link to #31 parent |
| **#43** | Drop-permanence sub | Envelope metadata leakage; cross-link to #31 parent |
| **#44** | Long-term-cryptanalysis standalone | BSI long-term-confidentiality |
| ** **#41** | Drop-permanence sub | Cross-device-sync UX boundary; cross-link to #31 |
| ** **#35** | Drop-permanence sub | Compromised-device retroactive decryption; cross-link to #31 |

**Pre-consolidation:** 13 new mints + 1 extension = 14 distinct Compromise # references.
**Post-consolidation:** 6 first-class mints (#31 ext + #32 + #33 + #39 + #42 + #43 + #44 = 7) + 4 sub-rows under parents = **7 mint numbers + cleaner cross-reference structure**.

**Net delta:** -7 Compromise # numbers (50% fewer cross-doc references). Each cross-document mention of "#36 RAM-residency" becomes "#33-RAM-residency-sub". External audit reading benefits from family-grouping (per L5's Common-Criteria-ST precedent).

**Alternative (flat-13 preserved):** Keep current 13-mint flat structure + ADD cross-reference table at end of SECURITY-POSTURE.md showing #31's downstream-hazard tree (consolidator's MF4 action). **Cost:** 1 extra section to maintain. **Benefit:** lens-origin-cleanliness preserved. **Pick winner:** **family-grouping wins per external-audit-readability**; flat-13-with-table is the VIABLE FALLBACK.

---

## §6 Invariant consolidation candidates (3 → 1-2 if elegant-shape exists)

**Per §3.4 analysis.** Recommendation:

**REJECT both directions of self-doubt:**
- REJECT consolidator's self-doubt §9.1.1 ("split Inv-18 into 18a/18b/18c") — Inv-18's three faces are governance-rules for one property, not three properties.
- REJECT "merge Inv-16 + Inv-17" hypothetical — would conflate envelope-construction with primitive-floor.
- REJECT "merge Inv-16 + Inv-18" hypothetical — would conflate construction-property with process-discipline.

**CONFIRM Inv-16 / Inv-17 / Inv-18 at current 3-invariant granularity. HIGH confidence.**

**Marginal refinement (low-confidence):** Inv-16's phrasing in consolidator §4 is a single ~250-word paragraph naming U1..U20. Consider splitting Inv-16 phrasing into **(a) construction discipline (U1-U3, U9-U10, U14)** + **(b) identity-binding discipline (U4-U5, U19-U20)** + **(c) Atrium-composition discipline (U17-U18)** as three NUMBERED CLAUSES within Inv-16, NOT three invariants. This is presentation-only; helps `feedback_pim_test_isolation_process_scoped_shared_state`-style invariant-row-level test-coverage tracking. MED confidence; Ben call.

**Possible future invariant (NAMED-DEFERRED):** Inv-19 cross-ecosystem-emit-discipline extracted from Inv-18 (c) cross-ecosystem-identifier-mapping if §3.5s grows substantial. Currently appropriate as Inv-18 sub-clause.

---

## §7 Confirmation / contestation / refinement of consolidator's Q1-Q5

### Q1 — Option A (oqs-rs) vs Option B (libcrux-ml-kem)

**CONFIRM consolidator HIGH-recommend Option B (libcrux-ml-kem).** Verified secret-independence via hax/F* compile-time is the load-bearing factor; oqs-rs adds C-FFI attack surface + no compile-time CT verification. **No refinement needed.**

### Q2 — LE-vs-BE codepoint endianness migration

**CONFIRM consolidator recommendation: migrate aead.rs LE → BE pre-v1-beta-freeze.** Network-byte-order is canonical; ~1 wave-day cost dominated by future-maintainer-confusion-prevention. **REFINEMENT:** add to the Cluster 3 CodepointRegistry-module PR; pair the BE migration with the registry mint so the migration + registry land atomically (preserves elegant-shape Cluster 3 packaging).

### Q3 — L9 dual-CID vs prior-P2P-architect F-refinement-2

**CONFIRM consolidator HIGH-concur with L9's dual-CID.** F-refinement-2 conflates content-identity with authorization-identity; recipient-substitution defense is structurally better served by per-stanza AAD-binding (U17). **Forkability ratification (2026-05-27) makes dual-CID load-bearing.** No refinement needed.

### Q4 — JOSE alg-name HPKE-11 vs HPKE-11-KE

**CONFIRM consolidator: HPKE-11-KE (key-encryption mode).** L9 §3.1's flat-composition narrative confirms HPKE wraps the CEK which then bulk-encrypts. **REFINEMENT:** R0 §4 design author must verify by writing out the explicit Layer-C wire format byte-by-byte; this is the kind of decision that can flip if Layer-C actually bulk-encrypts via HPKE-direct (which would mean HPKE-11-integrated). **HIGH confidence in HPKE-11-KE based on consolidator's reading of L9; MED-HIGH until R0 byte-level verification.**

### Q5 — Sealed-Sender as v1-beta DEFAULT vs additive slot

**CONTEST consolidator's MED-leaning-L6 (additive-slot).** I lean the OTHER direction — **CONSIDER aggressive-privacy: ship Sealed-Sender as v1-beta DEFAULT codepoint.**

**Reasoning for contesting:**
1. **Wire-format permanence asymmetry.** If Sealed-Sender ships at v1-beta as additive slot + plaintext-sender stays default, **the plaintext-sender codepoint is FOREVER the well-supported default.** v1-beta wire-format freeze means callers who don't know to opt-in get metadata-leaking envelopes by default until end-of-time. Per `feedback_pim_18_shape_not_substance_pre_flight`, defaults stick.
2. **#43 Compromise narrative cost.** Shipping plaintext-sender default forces writing #43 honest-disclosure to ALL Benten users — "by default, your sender-DID is visible to relays + archive correlators." This is a marketing/positioning cost that compounds with v1-beta launch announcement.
3. **Engineering-scope cost asymmetry (consolidator claim).** Consolidator estimates "~5-8 more wave-days" for Sealed-Sender implementation. **CHALLENGE:** the Cluster 4 Privacy-Surface Reservation Block PR (per §2) already lands U22 + U23 + U24 + U25 invariants at v1-beta; promoting U22 from reservation-only to implementation-shipped adds ~3-5 wave-days (not 5-8), because the wire-format work is identical (codepoint mint + variant slot) — only the HPKE-mode-auth psk_id encryption + spam-mitigation grant token work differs. ~3-5 wave-days vs a permanent-default-metadata-leak is favorable.
4. **Spam-mitigation isn't a v1-beta-blocker.** Sealed-Sender's spam-mitigation pattern (Signal's per-recipient delivery tokens) is for adversarial spam-from-anyone deployment. Benten's v1-beta deployments are likely small-trust-circle peer-mesh networks where spam-from-unknown-DIDs isn't the primary threat. Spam-mitigation can ship at v1-GM as additive tightening.
5. **Capability-system composition.** Benten's existing capability discipline (per UCAN-style grants) provides some spam-mitigation as a by-product — recipients can require capability-bearing grants for inbound drops, which incidentally rate-limits unknown-DID drops. The Sealed-Sender vs capability-system composition story may be cleaner than consolidator's "Sealed-Sender spam-mitigation is real cost" framing suggests.

**Reasoning for NOT contesting hard (why I leave as NAMED-DEFERRED not REVERSE-RECOMMEND):**
1. I have not done full byte-level analysis of HPKE-mode-auth psk_id integration cost for Benten's specific Layer-C wire format.
2. The Sealed-Sender V2 per-recipient-unlinkable (U25) discipline is a hard engineering ask; promoting U25 from invariant-only to default-shipped is more like ~8-12 wave-days.
3. Ben's marketing-vs-scope weighting is a Ben judgment call, not a cryptographer call.

**RECOMMENDATION FOR Q5:** Surface to Ben as a more substantive fork than consolidator framed it. **DEFER-NAMED-NOW disposition** = surface to R0 plan author as an explicit re-decision-point with §7 reasoning above. Either disposition is defensible; the cost-asymmetry consolidator cited may underweight the permanent-default-leak cost.

### Meta-consolidator recommendations: §8 MF1-MF5

| MF | Disposition |
|---|---|
| MF1 sender-auth/metadata-privacy STRUCTURAL TENSION | **CONFIRM** — codified as Inv-18 (c) per §6 |
| MF2 Inv-15 + Inv-16 sibling-pattern | **CONFIRM + STRENGTHEN** — future Inv-N mints should cite-pattern against Inv-15+16 explicitly; codify as `feedback_invariant_discipline_generalizes_across_crypto_layers` candidate memory entry (DEFER-NAMED-NOW to memory-update at R6 phase-close convergence) |
| MF3 permanence-package (U9+U11+U14) | **CONFIRM + EXTEND to U1+U2+U3+U9+U10+U14** per §2 Cluster 1 |
| MF4 Compromise #31 downstream-hazard tree | **CONFIRM + STRENGTHEN** — codify family-grouping per §3.3 + §5 |
| MF5 lens-composition coverage gaps | **CONFIRM** + flag #5b (UX-affordance lens) as load-bearing if Q5 ratified as additive-slot (because user-facing Sealed-Sender opt-in UX becomes critical to avoid silent-leak default) |

---

## §8 Self-assessment + confidence per finding

### What I did
1. Tree-state pre-flight on worktree branch (clean against `2172cb6d`).
2. Fetched `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16`.
3. Read consolidated registry in full via 4 chunks (1-250, 250-599, 599-end, plus targeted re-reads).
4. Cluster-decomposed 28 amendments along ARCHITECTURAL-PROPERTY axis (not lens-origin axis).
5. For each cluster, asked: "what single elegant structural shape closes all members?"
6. Documented winner-picks + NAMED-DEFERRED alternatives.
7. Consolidated 14 Compromise mints into 6 family-groups + 4 narrative sub-rows.
8. Confirmed/contested consolidator's Q1-Q5 + MF1-MF5.

### Confidence summary

| Section | Confidence | Rationale |
|---|---|---|
| §1 Executive verdict | **HIGH** on direction; **MED-HIGH** on Q5 contest | Cluster analysis robust; Q5 stance is one-cryptographer-judgment, Ben final call |
| §2 Cluster 1-9 winner picks | **HIGH** on Clusters 1-3 + 6 + 8; **MED-HIGH** on Cluster 4 (U22 stance affects); **MED** on Cluster 5 (could split differently) | Cluster boundaries justified per architectural property |
| §3.1 canonical_binding nine-amendment closure | **HIGH** | Mechanical composition; consolidator MF1 implicit + extended |
| §3.2 ENVELOPE-V1-SPEC.md root | **HIGH** | Highest-leverage elegant reduction; -86% cite-drift surface |
| §3.3 Compromise-family grouping | **HIGH** structural; **MED-HIGH** on specific family boundaries | External-audit reading benefit clear; family boundaries are one judgment call |
| §3.4 Inv-16/17/18 stay at 3 | **HIGH** | 3 distinct abstraction layers; merging conflates |
| §3.5 ExternalCryptographicGrant | **MED** — NAMED-DEFERRED appropriate | Depends on Option<...> field ratio per variant; surface to R0 |
| §3.8 MLS-PQ pre-v1-beta NAMED-DEFERRED | **HIGH** that elegant-shape exists; **HIGH** that MLS-PQ not ready at calendar | Library maturity + team-scope-fit are real blockers |
| §4 reduction tally | **HIGH** on direction; **MED-HIGH** on specific PR counts | ~12 PRs estimate could be 10-14 depending on wave-decomposition |
| §5 Compromise mint reduction | **HIGH** on family-grouping winner; flat-13 is viable fallback | Auditor-readability dominates |
| §6 Inv stay at 3 | **HIGH** | Reject both consolidator self-doubt directions |
| §7 Q5 contest | **MED-HIGH** on direction (privacy-default-leak permanence); Ben final | Cost-asymmetry argument is real but engineering-scope estimate uncertain |
| §7 Q1-Q4 confirmations | **HIGH** | Consolidator readings sound |

### What I could be wrong about
1. **Q5 Sealed-Sender as default vs additive.** Engineering scope estimate for Sealed-Sender impl is ~3-5 vs ~5-8 wave-days — I'm contesting consolidator's higher estimate. Could be wrong if HPKE-mode-auth integration is more involved than I assumed.
2. **§3.5 ExternalCryptographicGrant.** I called this NAMED-DEFERRED but a different cryptographer could ratify as ELEGANT-SHIP-NOW; depends on per-variant field analysis I didn't do exhaustively.
3. **§3.3 Compromise family-grouping.** Could be over-engineered for v1-beta scale (only 14 Compromises total); the flat-13 + cross-link-table fallback is one row simpler.
4. **§4 wave-day savings (~3-6 net).** Small absolute number; could be noise. The cite-drift-cycle savings (~5/year) is the more durable claim.
5. **Cluster 5 Atrium composition could split differently.** I packaged U17+U18+U21 as one PR; some readings would split U21 (workflow variant slot) into its own PR for independent reservation.

### Lower-confidence areas (honest disclosure)
- Did NOT independently re-verify any of the 9 lens-review SHAs against the consolidator's claims; trusted consolidator's frozen-SHA citations.
- Did NOT do byte-level analysis of HPKE-mode-auth psk_id integration cost (relevant to Q5).
- Did NOT enumerate the multicodec.csv contents to verify U8 IANA-disjoint range claim (trusted L7's verification).
- Did NOT run kani or dudect locally to validate U39/U38 tractability claims.
- Used the consolidator's amendment numbering verbatim; if numbering is off-by-one or has dedup errors, my analysis inherits them.

### What this critique does NOT cover
- Per `feedback_extra_reflection_pass_for_elegant_permanent_shape`: I am NOT a 10th lens reviewer. I do NOT re-do construction soundness / impl-engineering / audit-readiness / privacy / cross-ecosystem / permanence / Atrium-integration analyses.
- Did NOT propose new amendments beyond the 28 + 13+1 Compromises + 3 invariants enumerated by consolidator.
- Did NOT challenge the NO-GO disposition on Option F+ pseudo-keypair (9-of-9 concur; unanimous, not in scope).

---

## §9 Citations

### §9.1 Primary input
- Consolidated registry: `phase-4-meta-core/option-f-plus-9-eyes-consolidated-registry @ fbdfeb16` → `.addl/phase-4-meta/option-f-plus-9-eyes-consolidated-registry.md` (939 lines)

### §9.2 9 lens reviews (frozen SHAs per consolidator §10.1) — read by reference; not independently re-verified
- L1: `phase-4-meta-core/option-f-plus-pseudo-keypair-review @ 6d4e173f`
- L2: `phase-4-meta-core/option-f-plus-second-opinion-cryptographer-review @ 7e900a3b`
- L3: `phase-4-meta-core/option-f-plus-third-reviewer-adversarial-design @ 13b624c3`
- L4: `phase-4-meta-core/option-f-plus-lens-l4-impl-engineering @ 4d4aae5f`
- L5: `phase-4-meta-core/option-f-plus-lens-l5-threat-model-audit-readiness @ 3f27f8e0`
- L6: `phase-4-meta-core/option-f-plus-lens-l6-privacy-metadata-leak @ 986e50bb`
- L7: `phase-4-meta-core/option-f-plus-lens-l7-cross-ecosystem-interop @ 208f98bb`
- L8: `phase-4-meta-core/option-f-plus-lens-l8-wire-format-stability @ d8d3c41c`
- L9: `phase-4-meta-core/option-f-plus-lens-l9-atrium-integration @ 1670aa03`
- e2r scope review: `phase-4-meta-core/encrypt-to-recipient-review-ffull-scope @ 220b5aae`

### §9.3 Disciplines invoked
- `feedback_extra_reflection_pass_for_elegant_permanent_shape` — Ben-ratified 2026-05-25; the load-bearing discipline for this critique
- `feedback_review_finding_ground_truth_verify` — downstream DISAGREE-WITH-EXPLANATION first-class
- `feedback_pim_cite_drift_fp1_recurrence` + `feedback_pim_cross_language_rule_mirror` — load-bearing for §3.2 ENVELOPE-V1-SPEC.md elegant-shape (5 confirmed cite-drift instances at HEAD)
- `feedback_canary_first_parallel_implementation` — load-bearing for §2 Cluster 1 PR-packaging
- `feedback_pim_18_shape_not_substance_pre_flight` — load-bearing for §7 Q5 defaults-stick argument
- `feedback_engine_primitives_vs_application_layer` — informs §3.5 ExternalCryptographicGrant NAMED-DEFERRED stance
- `feedback_no_defer_HARD_RULE` — all NAMED-DEFERRED dispositions in §4 + §7 carry explicit revisit-triggers per clause (b)

### §9.4 External standards referenced (not re-verified, propagated from consolidator §10.3)
- RFC 9180 HPKE; RFC 8949 CBOR; FIPS 203 ML-KEM; draft-ietf-mls-pq-ciphersuites-04
- multicodec.csv + multikey.csv (IPLD specifications)
- libcrux-ml-kem + hax/F* (Cryspen)
- Signal Sealed Sender V1 (2018) + V2 (2024)
- BSI TR-02102-1; ANSSI hybrid-mandatory guidance

---

**End of critique.** Recommended next-step: R0 plan-doc author reads §1 + §4 + §7 + §3.2 before drafting R0; surfaces Q5 contest + ENVELOPE-V1-SPEC.md root + family-grouping Compromise consolidation explicitly to Ben for ratification.
