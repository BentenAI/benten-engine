# MembershipSet unification — M3 amendment-by-amendment transformation analysis

> Top-banner re-orient (HANDOFF discipline). This is M3 of the 5-specialist MembershipSet
> panel. Read BEYOND what's named here if you arrive cold: the 9-eyes consolidated registry
> at SHA `fbdfeb16`, the 5 critique-round outputs (`3f5a4351`, `9c548e5f`, `79c99aa5`,
> `6ea9718a`, `8e374a9d`), AtriumPolicy `e90900b4`, Q3-revisit `23f76e24`, L10/L11/L12
> (`a81d6da2`, `68eadd0c`, `7947f32c`), Path-A.5 `5f50a028`, and the 3 cataloger outputs
> (`3618e051`, `1816ea60`, `50eb901d`). M2's primitive design was running in parallel at
> dispatch-time and may have landed since; if so, reconcile this transformation analysis
> against M2's final shape.

- Author: M3 specialist (amendment-transformation under MembershipSet unification)
- Tree HEAD at write: `2172cb6d` (origin/main, fetched + verified at start of session)
- Branch: `phase-4-meta-core/membership-set-m3-amendment-transformation`
- Date: 2026-05-27
- Scope: ONE pass per amendment / Compromise / invariant from the post-critique registry
  showing whether MembershipSet unification ELIMINATES / SIMPLIFIES / leaves UNCHANGED /
  EXPANDS / merely RENAMES the row. Aggregate delta + wave-day delta + simplified registry
  + wave-sequencing implications + cost-benefit verdict.
- Out-of-scope: primitive Rust API design (M2 owns); red-team attack catalog (M4 owns);
  CGKA literature survey (M5 owns); transport-configurability per-Atrium (M6 owns).

---

## §1 Executive verdict + confidence

**Headline.** MembershipSet unification is **NET-ELEGANCE-WIN (MODEST)** — confidence
**MED-HIGH** — but the win is smaller than the "12-primitives-irreducible" framing
suggests. It is a **net –2 to –4 amendments** (depending on which simplification axis
Ben ratifies), **net –200 to –400 LOC**, **net –1 to –3 wave-days** off the consolidated
~80-101-wave-day envelope. The win is dominated by ONE structural collapse: the
"multi-stanza-HPKE + shared-secret + FORK-ONLY-rotation" pattern that recurs in
**4 sites** (multi-device K_principal wrap; K_Atrium distribution; Drop-bundle
encrypt-to-recipient; cross-Atrium federation grants) becomes ONE substrate with ONE
test family + ONE invariant + ONE wire codepoint family + ONE doc shape. That collapse
discharges or shrinks U17 + U18 + U19 + U20 + U25 + U41(L10)/U42(Q3) + the L9 A-family +
the Path-A.5 K(V) keying discipline + the Q3 Option D K_DedupScope generalization.

**What MembershipSet does NOT collapse (the honest reckoning).**

1. **The 4 distinct identity-concepts do NOT merge.** K_principal-rotation generation
   (per-user device set), DAK (per-device-per-user), per-DID hybrid-sig keypair, and the
   Inv-15 payload-CID-as-identity discipline are STRUCTURALLY distinct trust-boundary
   anchors and must remain so. MembershipSet unifies the **shared-secret-fanout shape**,
   NOT the identity layer. M1c §10.2 names this clearly: DAK is strictly per-device, K(N)
   derivation chain is path-tagged NOT membership-shared, hybrid-sig keypair is per-DID,
   and Compromise-#31 / Inv-15 closure is at the identity layer not the MembershipSet
   layer. Forcing these under one umbrella OBSCURES rather than clarifies.

2. **Two operations do NOT fit the primitive cleanly.** `dedup_blind_cid` (Q3 Option D's
   blinded plaintext-CID HMAC under K_Atrium per U18-triple) is content-equality-oracle
   defense, semantically orthogonal to encrypt-to-recipient. `fork` (Atrium-fork +
   MembershipSet-fork) is governance, not crypto. Forcing both into a single
   `MembershipSet` API surface generates leaky abstraction (M2 will ratify whether to
   include them as inherent vs. as a sibling trait; consolidator-style judgment leans
   SIBLING-TRAITS).

3. **Typed-variants (`Kind: Atrium | DeviceMesh | SingleDevice`) re-introduces per-Kind
   complexity** in admin-rotation (D7 only-Atrium), AtriumPolicy (D1-D5 only-Atrium), and
   K_principal protection (only DeviceMesh has DAK-Argon2id derivation). Net new
   match-arms in ~12 sites; mostly mechanical but not free.

**Net per-task verdicts (preview; details in §2-§8).**

| Task | Verdict | Net delta |
|---|---|---|
| Task 1 — 28+ amendments | **NET-ELEGANCE-WIN (MODEST)** | 3-5 amendments ELIMINATED (mostly U17/U19/U25 collapse into U-MS); 4-5 SIMPLIFIED (U18, U20, L9 A-family); 18+ UNCHANGED; 2-3 EXPANDED with per-Kind sub-cases (AtriumPolicy U41-AP; D7 admin discipline); 1 RENAMED-ONLY (U18 plaintext_cid_atrium → plaintext_cid_membership_set). |
| Task 2 — 14+ Compromises | **NET-NEUTRAL** | 0 closed; 1 NEW (`#48`: MembershipSet-shape-leak); 2 RENAMED (#41 cross-device → MembershipSet-DeviceMesh-kind; #45 mults Q3 → MembershipSet-distribution); 11 UNCHANGED. |
| Task 3 — Invariants | **NEW Inv-20 MINTED** | Inv-15/16/17/18 untouched; Inv-19 (L11 keying CRDT-input) simplifies under Path-A.5 + MembershipSet (immutable-Version-Node-CID is the canonical KDF input + MembershipSet is the canonical fanout substrate); NEW Inv-20 = MembershipSet primitive invariant (single fanout substrate; FORK-ONLY rotation; per-Kind variant exhaustiveness). |
| Task 4 — AtriumPolicy → MembershipSetPolicy | **PARTIAL GENERALIZATION** | D1, D2, D3, D5, D6 generalize cleanly. D4 (admin entity) + D7 (single-vs-threshold admin) ONLY meaningful for Atrium Kind (DeviceMesh = user-IS-admin via DAK; SingleDevice = N/A). Per-Kind variant required. |
| Task 5 — Simplified registry | **23-25 amendments + 14-15 Compromises + 5 invariants** (was 28+ / 14+ / 4) | See §6. |
| Task 6 — Wave-sequencing | **+1 NEW Wave (`Wave-MS-PRIMITIVE`) canary-first; –1 wave merge (Wave-G AtriumPolicy folds under)** | Net Δ = 0 waves; sequencing tightened. |
| Task 7 — Cost-benefit | **NET-ELEGANCE-WIN (MODEST)** | Wave-day delta **–1 to –3** off 80-101 baseline; LOC delta **–200 to –400**; audit-surface delta **–8 to –12 paragraphs of docs**, **–1 invariant phrasing-redundancy** (Inv-19 simplifies). Not the dramatic "12-primitives-irreducible" win, but a real consolidation. |

**Confidence breakdown.**
- HIGH on the "4-site fanout collapse" (M1c §10.3 explicitly identifies it; L9 + Q3 + L11
  + Path-A.5 + L10 all bumped into adjacent slices of it).
- MED-HIGH on the per-Kind typed-variant cost being modest (mechanical match-arms in
  ~12 sites; not free but bounded).
- MED on the AtriumPolicy → MembershipSetPolicy generalization correctness (D4 admin
  semantics for DeviceMesh-Kind needs Ben's call; my reading is "user-IS-admin via DAK
  is structurally cleaner than minting a degenerate admin-DID-of-self").
- MED on the wave-day delta being NET-WIN rather than break-even — depends sensitively
  on how cleanly M2 lands the primitive crate (Wave-MS-PRIMITIVE canary-first cost is
  ~1.5-2.5 wave-days; if it slips or develops Kind-specific complexity beyond what M1c §10
  predicts, break-even is plausible).
- MED-LOW on a few orthogonal-but-touched amendments (U21 ExecuteWorkflow per-Kind
  semantics: only DeviceMesh and Atrium Kinds make sense for hyper-scaling, not
  SingleDevice — but enumerating exhaustively at v1-beta is a freedom-of-future-impl
  question, not a structural blocker).

**What I am NOT confident about + would defer to Ben + M2-M6.**
- Whether `dedup_blind_cid` and `fork` belong INSIDE the MembershipSet primitive's
  API surface or as sibling traits. Both options work; sibling-traits gives a cleaner
  primitive but proliferates trait imports at call sites. Defer to M2.
- Whether the `SingleDevice` Kind is actually needed (a SingleDevice MembershipSet of
  size-1 is degenerate; could be modeled as DeviceMesh with one member). My reading:
  SingleDevice is worth keeping as a Kind for clearer type-level intent + audit-readability,
  but it's a judgment call.
- Whether MembershipSet should be defined in a NEW `benten-membership-set` crate or
  added to an existing crate (`benten-id` or `benten-crypto-suite`). Crate placement
  affects wave-sequencing.

---

## §2 Per-amendment transformation table

Format per row:
- **#** | **Original U#** | **Title (abbrev)** | **Transformation** (a-e per brief) |
  **Rationale + cites** | **Wave-day Δ** | **Compromise/Inv impact**

Transformation letters:
- **(a) ELIMINATED** — discharged by MembershipSet primitive
- **(b) SIMPLIFIED** — shrinks under MembershipSet
- **(c) UNCHANGED** — orthogonal
- **(d) EXPANDED** — grows with per-Kind sub-cases
- **(e) RENAMED-ONLY** — semantics identical; identifier changes

### §2.1 Group A — Construction soundness (U1-U2)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 1 | U1 | Codepoint in AAD/info | **(c) UNCHANGED** | MembershipSet operates ABOVE the envelope-framing layer (it produces inputs to multi-stanza HPKE; doesn't change the AAD-binding discipline itself). U1 applies identically to every Seal call regardless of whether the recipient set was provided ad-hoc or via a MembershipSet handle. Cites: 9-eyes registry U1 (origin L2/Am1); M1c §11.4 Q12 ("MembershipSet names the cryptographic primitive — multi-stanza-HPKE + FORK-ONLY-rotation — composed BELOW the AAD layer"). | 0 |
| 2 | U2 | Strict-decode; no cross-variant fallback | **(c) UNCHANGED** | Decoder discipline is decoder-layer; MembershipSet is sender-side composition. Codepoint variants for `MembershipSetEncryption` (new) get added to U9's `#[non_exhaustive]` `EnvelopePayload` enum, but the strict-decode rule applies to them identically. | 0 |

### §2.2 Group B — Canonical encoding + AAD shape (U3-U5 + R-C2 + U28)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 3 | U3 | Canonical TLV length-injectivity | **(c) UNCHANGED** | TLV is wire-encoding discipline. MembershipSet uses it via the underlying envelope variant; no change. | 0 |
| 4 | U4 | Sender-DID in AAD (non-Vault variants) | **(b) SIMPLIFIED** | When the recipient set is realized via a MembershipSet handle, the sender-DID-binding rule **uniformly** applies across all 4 sites (Drop, K_Atrium distribution, multi-device wrap, federation grant) instead of 4 parallel per-site discussions. U22 Sealed-Sender sibling-slot reservation also generalizes uniformly: every `MembershipSetEncryption` codepoint gets a paired SealedSender-codepoint slot under Inv-18b. **Sub-amendment reduction**: U4's prose can collapse the per-site rationale into "for every MembershipSet-realized envelope, sender-DID is AAD-bound EXCEPT for the Vault MembershipSet (size-1; sender = recipient)." Cite: AtriumPolicy §4.7 (policy_version fingerprinting parallel); Q3-revisit §6.1 (Q-extra). | –0.2 (doc) |
| 5 | U5 | Replay-window (DeviceLink + RemotePermission) | **(b) SIMPLIFIED (modestly)** | Per-Kind: Vault MembershipSet (SingleDevice / size-1) excluded as today. DeviceMesh MembershipSet for multi-device-wrap inherits U5 (the wrap is a credential-class envelope). Atrium MembershipSet inherits U5 for any RemotePermission-class envelope; not for K_Atrium-distribution-class (which is forever-valid distribution-event per Compromise #31 ext). The per-Kind exclusion table replaces today's "Vault + DropToRecipient EXCLUDED" enumeration with "size-1-SingleDevice-Vault Kind + K_Atrium-distribution-event Kind EXCLUDED". | –0.1 (doc) |
| 6 | R-C2 (C2) | Composed clock-skew rule (U5 ↔ U28 composition) | **(c) UNCHANGED** | Clock-skew composition is orthogonal to membership shape. Carries through to MembershipSetPolicy verifier-side semantics intact. | 0 |
| 7 | U28 | Coarse 1-hour epoch buckets | **(c) UNCHANGED** | Hour-bucket math is per-envelope; MembershipSet doesn't change it. | 0 |

### §2.3 Group C — Side-channel mitigation (U6)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 8 | U6 | Bernstein-Persichetti Decap CT | **(c) UNCHANGED** | Crypto-impl primitive; MembershipSet uses libcrux ML-KEM Decap once per stanza per recipient; the CT property applies per-stanza. | 0 |

### §2.4 Group D — Wire-format pinning (U7-U8 + U11)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 9 | U7 | BE codepoint on-wire | **(c) UNCHANGED** | Wire-encoding discipline. | 0 |
| 10 | U8 | Codepoint registry IANA-disjoint + cite-drift scanner | **(b) SIMPLIFIED** | The 4 parallel multi-stanza codepoints (Drop multi-recipient `0x6301` per U17; K_Atrium distribution `0x6340` per Q3 §7.1; multi-device wrap codepoint TBD; federation grant codepoint TBD) collapse to ONE `MEMBERSHIP_SET_ENCRYPTION = 0x6380` codepoint family with `MembershipSetKind: u8` discriminator. Registry rows reduce 4 → 1 family-row with 4 Kind variants. Inv-18a registry discipline cleaner. | –0.3 (registry consolidation) |
| 11 | U11 | Escape codepoint + experimental range | **(c) UNCHANGED** | Permanence scaffolding. | 0 |

### §2.5 Group E — Permanence / wire-format-evolution (U9-U16)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 12 | U9 | `EnvelopePayload` `#[non_exhaustive]` + typed-reject | **(d) EXPANDED (1 variant)** | Adds 1 new variant: `MembershipSetEncryption { kind, stanzas, k_set_wrapped_under_membership_set_hpke, body_aead_ciphertext, ... }`. Replaces N=4 future-additive variants the 9-eyes registry was reserving piecemeal. Net amendments: U9 stays + DELETES 3 reserved-piecemeal variants from `0x6300..0x639F` reservation table per U13. | +0.1 (new variant) –0.3 (consolidate 4 variant reservations) = –0.2 |
| 13 | U10 | `BindingContext` `#[non_exhaustive]` + typed-reject | **(d) EXPANDED (1 variant)** | Adds `BindingContext::MembershipSetSeal { membership_set_id, generation, kind, ... }` to bind the membership-set-realized envelope's targeting + generation discipline into AAD. Net 1 added variant. | +0.1 |
| 14 | U11 | Reserve 0xFFFF escape + EnvelopeShape codepoint axis | **(c) UNCHANGED** | Already covered in §2.4. | 0 |
| 15 | U12 | Nonce-length variant discrimination | **(c) UNCHANGED** | Per-codepoint nonce-length; MembershipSet uses the per-stanza-HPKE codepoint's nonce-length discipline. | 0 |
| 16 | U13 | FS-gap honest-disclosure + MLS-PQ codepoint reservation | **(b) SIMPLIFIED** | MLS-PQ + CGKA reservation in `0x63A0..0x63CF` bracket aligns naturally with the new MembershipSet codepoint family. Future CGKA-shipped MembershipSet variant is the structural FS-gap closure; reservation discipline cleaner because future MembershipSet variants are ALREADY codepoint-discriminated within the family. M5 surveys this. | –0.1 (doc) |
| 17 | U14 | `aad_version: u8` prefix | **(c) UNCHANGED** | Canonicalization-version-bind; per-envelope, MembershipSet-agnostic. | 0 |
| 18 | U15 | Did multikey + Did::Unknown | **(c) UNCHANGED** | DID encoding discipline. | 0 |
| 19 | U16 | CodepointLifecycle typed-state | **(c) UNCHANGED** | Lifecycle scaffolding; applies to MembershipSetEncryption codepoint family identically. | 0 |

### §2.6 Group F — Atrium-integration (U17-U21 — the heaviest collapse zone)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 20 | U17 | HpkeMultiBase + cross-stanza substitution defenses | **(a) ELIMINATED → folded into MembershipSetEncryption primitive** | HpkeMultiBase IS the MembershipSet realization for `Kind::Atrium` and the per-stanza AAD-binding discipline (`recipient_did_list, sender_did, stanza_index, recipient_key_generation`) becomes a PROPERTY of `MembershipSetEncryption` instead of a per-variant amendment. Net: U17's wire-format-shape reduces to ONE row "every MembershipSet realization uses per-stanza-HPKE with the canonical AAD-binding tuple". Cites: L9/A1 (U17 origin); M1c §10.3 ("multi-stanza-HPKE IS the MembershipSet-encryption primitive"); R-C7 (C2) (universal size-class buckets per codepoint = per-MembershipSet-family). | **–2.0** (U17 wave-day estimate was 3-4 days for multi-stanza wire-format + per-stanza AAD work; folds into MembershipSet primitive impl wave) |
| 21 | U18 | Dual-CID → triple-CID after Q3 | **(b) SIMPLIFIED → "plaintext_cid_membership_set" generalization** | After Q3 ratification, U18 became `plaintext_cid_local + plaintext_cid_atrium + envelope_blob_cid` where `plaintext_cid_atrium = HMAC-SHA256(K_Atrium, plaintext_cid_local)`. Under MembershipSet, this generalizes: `plaintext_cid_membership_set = HMAC-SHA256(K_MembershipSet, plaintext_cid_local)` where K_MembershipSet is the MembershipSet's `shared_key`. The `dedup_scope_id` field per Q3 §5.6 Option I naturally maps to `MembershipSetId` + `MembershipSetKind` (the scope_kind discriminator becomes the MembershipSet Kind). Net: U18 + Q3's NEW U41-suggest (dedup_scope_id) + NEW U42-suggest (K_Atrium key mgmt) all collapse into "MembershipSet provides the canonical dedup-blind-CID + the shared-key management; U18 names the triple-CID shape parameterized over MembershipSet". Cites: Q3-revisit §5.6 (Option I framing already anticipates this); §7.1 NEW U41-suggest + U42-suggest. | **–1.5** (Q3 §7.2 cost estimate had U41 + U42 + Inv-19 + scope-resolution at ~3 wave-days; folds into MembershipSet ~1-1.5 wave-day primitive work + reuse) |
| 22 | U19 | recipient_key_generation: u32 | **(a) ELIMINATED → folded into MembershipSet `generation: u64`** | The per-MembershipSet `generation: u64` field IS the recipient-key-rotation generation for the Atrium-Kind and Drop-bundle Kind cases. The "envelope sealed to a generation I no longer have" detection becomes a MembershipSet-level operation. Net: U19's per-recipient counter discipline becomes a per-MembershipSet generation counter (one source of truth). Cites: L9/A3 (U19 origin); M2 design sketch (`generation: u64` field in MembershipSet); M1c §10.3 ("rotation = FORK-ONLY (recipient retains past)"). | **–1.5** (U19 wave-day estimate ~2 days; folds into MembershipSet `add_member` / `remove_member` ops which are already in M2's primitive ops list) |
| 23 | U20 | k_principal_generation in Vault + AAD | **(b) SIMPLIFIED → folded under DeviceMesh-Kind MembershipSet generation** | K_principal IS the shared_key of a DeviceMesh-Kind MembershipSet whose members are the user's devices. The `k_principal_generation: u32` becomes the DeviceMesh MembershipSet's `generation: u64`. The `KPrincipalRotation` Atrium-replicated Node becomes a `MembershipSetGeneration` Atrium-Node parameterized by Kind. Net: U20's per-Vault-generation-tracking becomes a per-MembershipSet-generation-tracking primitive. **BUT**: U42 (L11)'s per-device vector counter `BTreeMap<Did, u32>` still applies — concurrent multi-device rotation needs CRDT-vector resolution; this is now framed as "MembershipSet-generation is a CRDT-vector under per-Kind Loro per-property LWW per device-DID partition" (Path-A.5 §5.5). | **–1.0** (U20 ~2 days + U42 ~2-3 days → consolidated ~2 days under MembershipSet `update_generation` primitive op) |
| 24 | U21 | ExecuteWorkflow variant | **(d) EXPANDED (per-Kind)** | ExecuteWorkflow is meaningful for `Kind::Atrium` and `Kind::DeviceMesh` (delegated executor in own-mesh or cross-Atrium); NOT for `Kind::SingleDevice` (no delegation needed). Adds per-Kind sub-arms in the BindingContext::ExecuteWorkflow variant. Modest. NOTE: per C3 §6.4 R4 (per-request-nonce) carries through unchanged. | +0.3 (per-Kind sub-arms) |

### §2.7 Group G — Privacy / metadata-leak (U22-U28)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 25 | U22 | Sealed-Sender additive codepoint slot | **(b) SIMPLIFIED → per-MembershipSet-Kind paired-slot discipline** | Inv-18b's "paired SealedSender slot for every plaintext-sender variant" generalizes: for the `MembershipSetEncryption` codepoint family, mint `MembershipSetEncryptionSealedSender` sibling family. ONE pair instead of 4 parallel pair-reservations. | –0.2 (codepoint registry consolidation) |
| 26 | U23 | Per-relay-unlinkability (transport blinding) | **(c) UNCHANGED** | Transport layer; orthogonal to MembershipSet. M6 owns the transport-configurability per-Atrium question; transport blinding applies per-envelope regardless of MembershipSet shape. | 0 |
| 27 | R-C7 (C2) | Universal size-class buckets (not per-codepoint) | **(b) SIMPLIFIED** | After MembershipSet consolidates the 4-fanout codepoints into one family, R-C7's "universal" framing applies naturally: ONE bucket schedule for the MembershipSetEncryption family. | –0.1 (doc) |
| 28 | U24 | Padding to size-class buckets | **(b) SIMPLIFIED** | Same as R-C7 above. | –0.1 (doc) |
| 29 | U25 | Per-recipient-unlinkable multi-stanza | **(a) ELIMINATED → invariant folded into MembershipSet primitive's invariant** | Per-recipient-unlinkability becomes a structural property of `MembershipSetEncryption` (each stanza an unlinkable copy by construction). Inv-18b clause "every multi-recipient envelope shape MUST provide per-recipient unlinkability" applies once per MembershipSetEncryption family. NET: U25 row becomes a CLAUSE of Inv-20 (the new MembershipSet primitive invariant) rather than an independent amendment. | –0.5 (rolled into Inv-20) |
| 30 | U26 | Cover-traffic NAMED-DEFERRED | **(c) UNCHANGED** | Post-v1-GM deferral; orthogonal. | 0 |
| 31 | U27 | DID-rotation discipline NAMED-DEFERRED | **(c) UNCHANGED** | Post-v1 UX; orthogonal. | 0 |

### §2.8 Group H — Cross-ecosystem interop (U29-U30)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 32 | U29 | Cross-ecosystem-identifier emit-discipline | **(b) SIMPLIFIED** | Cross-ecosystem map row count reduces from 4 fanout codepoints to 1 family-codepoint with Kind discriminator. JOSE/COSE/multicodec emit-table cleaner. | –0.2 (table consolidation) |
| 33 | U30 | DAG-CBOR outer framing | **(c) UNCHANGED** | Outer-framing decision; orthogonal. (R3 strict-deterministic-CBOR-decode also unchanged.) | 0 |

### §2.9 Group I — Impl-engineering (U31-U36)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 34 | U31 | libcrux-ml-kem swap | **(c) UNCHANGED** | Crate choice; orthogonal. | 0 |
| 35 | U32 | XChaCha20-Poly1305 for re-used-key sites | **(c) UNCHANGED** | AEAD choice; orthogonal. | 0 |
| 36 | U33 | NAPI single-source canonical_binding | **(c) UNCHANGED** | Cross-lang discipline; MembershipSet API also crosses NAPI but the canonical_binding discipline is the same. | 0 |
| 37 | U34 | NAPI opaque-handle pattern | **(d) EXPANDED (one new handle type)** | NEW: `MembershipSetHandle` (analogous to `VaultHandle`) returned at NAPI boundary. ~50 LOC of opaque-handle wrapping; bounded. | +0.2 |
| 38 | U35 | wasm_js getrandom cfg at shell | **(c) UNCHANGED** | Build config. | 0 |
| 39 | U36 | Tokio cancel-safety wrap | **(c) UNCHANGED** | Async discipline; MembershipSet ops are blocking-crypto and need `spawn_blocking` wrap per same discipline. | 0 |

### §2.10 Group J — Test-corpus + CT-validation + formal-methods (U37-U39)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 40 | U37 | Golden-vector corpus | **(b) SIMPLIFIED** | One MembershipSet round-trip golden-vector family covers 4 fanout sites in one. Estimated ~16 vectors reduce to ~6 family-vectors (Atrium / DeviceMesh / SingleDevice × encrypt / dedup-blind-cid). | –0.5 (vector authoring) |
| 41 | U38 | dudect-bencher CI integration | **(c) UNCHANGED** | Per-primitive CT bench; orthogonal. | 0 |
| 42 | U39 | kani injectivity proof of canonical_serialize_tlv | **(b) SIMPLIFIED (slightly)** | kani harness extends to MembershipSet seal/open with bounded member-count; ONE harness covers 4 fanout cases. | –0.3 (kani harness consolidation) |

### §2.11 Group K — Audit-deliverable docs (U40)

| # | U# | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 43 | U40 | THREAT-MODEL.md mint | **(b) SIMPLIFIED** | THREAT-MODEL.md adversary rows describing multi-recipient / multi-device threats reduce from 4 parallel-treatment rows to 1 MembershipSet-Kind-parameterized row family. ~20% doc reduction in the multi-fanout section. | –0.3 (doc) |

### §2.12 Post-critique additions (C2 R-C1..R-C10; C3 U41 + U42; C5 U39-promotion + U39b-d; L10 U41-U43; L11 U41-U44 + #45 + Inv-19; Q3 5 pieces; AtriumPolicy 5 D1-D5)

| # | Origin | Title | Trans | Rationale + cites | Δ days |
|---|---|---|---|---|---|
| 44 | C2 R-C1 | Sealed-Sender multi-stanza cross-stanza binding via HPKE-mode-auth psk_id | **(b) SIMPLIFIED** | Generalizes to "Sealed-Sender MembershipSetEncryption variant binds sender-via-psk_id at the MembershipSet level"; one rule instead of per-fanout-codepoint. | –0.1 |
| 45 | C2 R-C2 | Composed clock-skew rule | (covered above) | — | 0 |
| 46 | C2 R-C3 | Per-recipient-unlinkability semantic bound | **(a) ELIMINATED → Inv-20 clause** | Folds into Inv-20 alongside U25. | (rolled) |
| 47 | C2 R-C4 | k_principal_generation in BindingContext::DeviceLink | **(b) SIMPLIFIED** | Now MembershipSet-generation across all Kind variants including DeviceLink (= multi-device-wrap Kind). | –0.1 |
| 48 | C2 R-C5 | Cross-ecosystem-emit on escape codepoints | **(c) UNCHANGED** | Escape-codepoint discipline orthogonal. | 0 |
| 49 | C2 R-C6 | Diagnostic-only error-disambiguation for U40 | **(c) UNCHANGED** | Audit-deliverable doc detail. | 0 |
| 50 | C2 R-C7 | Universal (not per-codepoint) U24 size-class buckets | (covered above #27) | — | 0 |
| 51 | C2 R-C8 | Split Inv-18 into Inv-18a/b/c | **(c) UNCHANGED** | Pure invariant-row split; orthogonal to MembershipSet shape. C3 §6.2 + C5 F-1 concur. | 0 |
| 52 | C2 R-C9 | Composition-properties test family for U37 | **(b) SIMPLIFIED** | Adds MembershipSet composition test family; same simplification as U37. | (rolled) |
| 53 | C2 R-C10 | Formal-methods lens deferral row | **(c) UNCHANGED** | Doc-only V1-FROZEN-INTERFACE-DEFERRED.md row. | 0 |
| 54 | C3 §6.1 U41 | MAL-BIND-K-CT/PK binding-properties + Compromise #45 | **(b) SIMPLIFIED** | The per-stanza AAD-binding defense generalizes from "per-stanza" to "per-MembershipSet-stanza"; ONE structural defense statement; exception for ExecuteWorkflow + RecoveryHook caller-supplied-pubkey unchanged. Cite: C3 §6.1; FIPS 203 + libcrux validate_public_key() per U31. **Compromise #45 renumbered to #45-MAL-BIND** (collision with L10 #45 multi-stanza wire-cost + Q3 #45 K_Atrium-leak; see §3 for canonical renumbering). | –0.1 (doc consolidation) |
| 55 | C3 §6.2 F2 | Split Inv-18 into Inv-18a/b/c | (covered #51) | — | 0 |
| 56 | C3 §6.3 R3 | strict-deterministic-CBOR-decode for U30 | **(c) UNCHANGED** | DAG-CBOR strictness; orthogonal. | 0 |
| 57 | C3 §6.4 R4 | Per-request-nonce for ExecuteWorkflow | **(c) UNCHANGED** | U21 sub-clause; per-request-nonce is per-envelope orthogonal to MembershipSet shape. | 0 |
| 58 | C3 §6.5 R5 | Permanence-triangle naming in CRYPTO-CODEPOINTS.md | **(c) UNCHANGED** | Doc-only naming; orthogonal. | 0 |
| 59 | C3 §6.6 R6 | Compromise #46 multiformats stewardship dependency | **(c) UNCHANGED** | Supply-chain disclosure; orthogonal. | 0 |
| 60 | C3 §6.7 R7 | U29 alg-name resolver function | **(b) SIMPLIFIED** | Resolver function maps per-Kind cleanly. | –0.1 |
| 61 | C3 §6.8 U42 | Standardized validation-error-code taxonomy | **(c) UNCHANGED** | EnvelopeError enum hardening; orthogonal. | 0 |
| 62 | C3 §6.9 #47 | Tauri NAPI-RS marshaling boundary side-channels | **(c) UNCHANGED** | NAPI-boundary disclosure; orthogonal (MembershipSetHandle gets the same hardening per U34 generalization). | 0 |
| 63 | C5 U39 promotion + U39a-d | kani harness families | **(b) SIMPLIFIED** | One MembershipSet-kani-harness covers 4 fanout cases per #42 above. | (rolled) |
| 64 | L10 U41 | Argon2idParameterTier wire-format-pin | **(d) EXPANDED (per-Kind)** | Argon2idParameterTier ONLY meaningful for `Kind::DeviceMesh` Vault MembershipSet (single user's devices; DAK derivation tier). For `Kind::Atrium` MembershipSet there's no Argon2id; for `Kind::SingleDevice` Vault there is. Per-Kind variant: `MembershipSetVaultBindingContext::Argon2idParameterTier { ... }` (DeviceMesh + SingleDevice Kinds only). | +0.2 |
| 65 | L10 U42 | recommended_max_recipients_per_drop + Compromise #45-L10 | **(b) SIMPLIFIED** | MembershipSet-Kind-parameterized: per-Kind recommendation table (Atrium: 32; DeviceMesh: typical 5; SingleDevice: 1). Doc-row reduction. | –0.1 |
| 66 | L10 U43 | G-CORE-PERF-1 bench infra | **(c) UNCHANGED** | Bench infra; orthogonal. Benches MembershipSet primitive ops once instead of per-fanout, slight savings absorbed into U37 simplification. | 0 |
| 67 | L11 U41 | CRDT-merge re-encryption + predecessor-CID AAD | **(b) SIMPLIFIED MASSIVELY UNDER Path-A.5** | Per Path-A.5 §4.8 + §5.3: U41-L11's "re-encryption-on-merge discipline" is **structurally dissolved** when K(V) keys to immutable Version-Node-CIDs (the universal precedent). MembershipSet inherits this: a MembershipSet operation always seals to an IMMUTABLE Version-Node-CID, NEVER to a mutable Anchor-CID. The MembershipSet primitive REINFORCES Path-A.5 by giving the impl-team ONE place to encode "MembershipSet.encrypt_to_set always takes a Version-Node-CID input". Net: U41-L11 collapses from "3-4 wave-days re-encryption discipline" to "0.5-1 wave-day documentation that K(V) is the canonical KDF input + MembershipSet enforces it at the API boundary". | –2.0 |
| 68 | L11 U42 | K_principal-rotation vector + log CRDT-merge rule | (covered above #23 with U20) | — | (rolled) |
| 69 | L11 U43 | Offline-replay carve-out (received_at_epoch) | **(c) UNCHANGED** | Recipient-local state; orthogonal to MembershipSet shape. | 0 |
| 70 | L11 U44 | Drop-bundle predecessor-CID chain + error-discrimination | **(b) SIMPLIFIED UNDER Path-A.5 + MembershipSet** | predecessor-CID chain ALREADY in Version-Node AAD under Path-A.5; MembershipSet adds the per-recipient stanza dimension cleanly. Substitution-attack vs CRDT-divergence error discrimination becomes a MembershipSet-level concept (per-MembershipSet-stanza recipient-list mismatch = substitution; predecessor-CID mismatch = divergence). | –1.0 |
| 71 | L11 U21-ext | ExecuteWorkflow result-provenance binding | (covered above #24 with U21) | — | (rolled) |
| 72 | L11 Inv-19 | Crypto-keying CRDT-input discipline | **(b) SIMPLIFIED → "KDF input must be CRDT-immutable; MembershipSet-shared-key + Version-Node-CID are the canonical inputs"** | Per Path-A.5 §5.3 simplified-Inv-19. Under MembershipSet, the static-invariant form strengthens: "every Benten KDF either reads (a) immutable substrate (Version-Node-CID, codepoint, BE encoding, TLV) OR (b) a MembershipSet-shared-key whose generation is CRDT-resolved per Inv-20." Two clean cases instead of an open-ended discipline. | –0.3 |
| 73 | Q3 Option D U18-update (triple-CID) | (covered above #21 with U18) | — | — | (rolled) |
| 74 | Q3 §7.1 NEW U41-suggest (dedup_scope_id) | (covered above #21 with U18) | — | — | (rolled) |
| 75 | Q3 §7.1 NEW U42-suggest (K_Atrium key mgmt) | **(a) ELIMINATED → MembershipSet primitive IS the K_X key mgmt for the Atrium Kind** | This was the candidate amendment most-tightly anticipating MembershipSet. Q3's `K_Atrium derivation + distribution + rotation` is verbatim the `Kind::Atrium` MembershipSet's `shared_key` + `multi-stanza-HPKE distribution` + `FORK-ONLY rotation`. Folds in entirely. | –1.5 |
| 76 | Q3 §7.1 NEW Inv-19-suggest (orphan-envelopes typed-reject) | **(b) SIMPLIFIED → folded into Inv-20** | "Every non-None dedup_scope_id maps to a vault-resolvable K_DedupScope for at least one recipient" becomes "every MembershipSetEncryption envelope's `membership_set_id` MUST resolve to a MembershipSet known to at least one recipient" — Inv-20 clause. | –0.2 (rolled into Inv-20 doc) |
| 77 | Q3 §7.1 NEW Compromise #45-Q3 (K_Atrium-leak fingerprint) | **(e) RENAMED → Compromise #48 MembershipSet-shape-leak** | Generalizes: leak of a MembershipSet's shared_key discloses MembershipSet-fingerprint forward+backward for that generation. Per-Kind severity (Atrium = membership-fingerprint; DeviceMesh = device-set-fingerprint = small + already-known; SingleDevice = trivial). | (–0.1 doc consolidation) |
| 78 | Q3 §7.1 NEW Compromise #46-Q3 (member-storage-host insider) | **(e) RENAMED → Compromise #49 MembershipSet-member-acting-as-storage-host** | Generalizes from "Atrium-member-storage-host" to "MembershipSet-member-acting-as-storage-host"; same trust-boundary collapse argument. | 0 (rename) |
| 79 | AtriumPolicy D1 (policy_version_at_seal in AAD) | **(b) SIMPLIFIED → MembershipSetPolicy generalization** | See §5 for full transformation. Net: AtriumPolicy schema fields generalize to MembershipSetPolicy schema; per-Kind defaults; one `policy_version_at_seal: u32` AAD field works for both Atrium-Kind credential-validity AND DeviceMesh-Kind device-validity. | (covered in §5; –0.5 doc collapse) |
| 80 | AtriumPolicy D2 (hybrid grandfather rule) | **(c) UNCHANGED** | Grandfather semantic identical per-Kind. | 0 |
| 81 | AtriumPolicy D3 (refresh_required Phase-4-Meta-Composing) | **(c) UNCHANGED** | UX-coupled phase placement; orthogonal. | 0 |
| 82 | AtriumPolicy D4 (single-admin-DID + Atrium-replicated Node) | **(d) EXPANDED (per-Kind)** | Per-Kind admin semantic: `Kind::Atrium` = explicit admin-DID (current AtriumPolicy.admin_did); `Kind::DeviceMesh` = user IS admin (admin-DID = the user's principal-DID by construction; degenerate AtriumPolicy.admin_did = principal_did); `Kind::SingleDevice` = N/A (no policy). | +0.3 (per-Kind doc + sig-validation) |
| 83 | AtriumPolicy D5 (permissive_default) | **(c) UNCHANGED** | Default semantic identical per-Kind. | 0 |
| 84 | AtriumPolicy D6 (refresh-source: OCSP-style fresh-attestation) | **(c) UNCHANGED** | Refresh mechanism identical per-Kind (where applicable). | 0 |
| 85 | AtriumPolicy D7 (single-vs-threshold admin) | **(d) EXPANDED (per-Kind; only-Atrium-meaningful)** | Threshold-admin is ONLY meaningful for Kind::Atrium; trivial for DeviceMesh (user-is-admin); N/A for SingleDevice. Per-Kind variant in MembershipSetPolicy schema. | +0.2 |
| 86 | Path-A.5 K(V) keying discipline | **(b) SIMPLIFIED via MembershipSet API enforcement** | Path-A.5 §5 prescribes "K(V) = KDF(K_principal-gen, V.cid) where V.cid is a Version-Node-CID". MembershipSet enforces this at the API boundary by typing `MembershipSet::encrypt_to_set(payload: VersionNodeCid, ...)` instead of `payload: NodeCid` — the type system prevents the L11-error of keying to a mutable Anchor-CID. Cite: Path-A.5 §6.2 ("Every successful encrypted-CRDT or content-addressed-encrypted system in the survey keys encryption to an IMMUTABLE substrate"). | –0.5 (API hardening folds in; net positive elegance) |

### §2.13 Aggregate amendment delta

| Category | Count |
|---|---|
| ELIMINATED (a) — discharged by MembershipSet | **5** (U17, U19, U25, Q3 §7.1 NEW U42-suggest = K_Atrium key mgmt, C2 R-C3 per-recipient-unlinkability) |
| SIMPLIFIED (b) — shrinks under MembershipSet | **17** (U4, U5, U8, U13, U18, U20, U22, U24, U29, U37, U39, U40, C2 R-C1, L10 U41/U42, L11 U41/U44, L11 Inv-19, Q3 Inv-19-suggest, AtriumPolicy D1, Path-A.5 K(V)) |
| UNCHANGED (c) — orthogonal | **~30** (most wire-format + crypto-primitive + audit-deliverable rows) |
| EXPANDED (d) — grows with per-Kind | **6** (U9, U10, U21, U34, L10 U41 Argon2id-tier, AtriumPolicy D4 + D7) |
| RENAMED-ONLY (e) | **2** (Q3 §7.1 #45, #46 → MembershipSet-shape-leak Compromise #48/#49) |

Net amendment-count reduction = **5 ELIMINATED + 17 SIMPLIFIED ≈ –3 to –5 net amendments**
(the SIMPLIFIED group is doc-shrinkage that doesn't always reduce row count; the ELIMINATED
group does). Net wave-day delta accumulated = **–10.1 to –10.3 wave-days off the
~80-101-baseline** BEFORE adding the MembershipSet primitive impl cost.

**Adding MembershipSet primitive impl cost** (canary-first Wave-MS-PRIMITIVE):
- New crate `benten-membership-set`: ~1500-2000 LOC; **~3.5-4.5 wave-days** (M2 will refine)
- NAPI bindings for MembershipSetHandle: **~0.5 wave-day** (folded into U34 expansion)
- Golden vectors + kani harness for MembershipSet ops: **~0.5 wave-day** (folded into U37 + U39 simplifications)
- AtriumPolicy → MembershipSetPolicy refactor: **~0.5 wave-day** doc + minor schema work
- THREAT-MODEL.md MembershipSet-row mints: **~0.3 wave-day**
- **Subtotal new cost: ~4.8-5.8 wave-days**

**NET wave-day delta after subtracting new cost from accumulated savings:**
- Savings: –10.1 to –10.3
- New cost: +4.8 to +5.8
- **NET: –4.3 to –5.5 wave-days** (NET-ELEGANCE-WIN; bigger than initial verdict's –1 to –3)

Honest hedge: the +4.8-5.8 new cost is conservative; if M2's primitive design develops
Kind-specific complexity (e.g., per-Kind initialization paths balloon beyond the design
sketch), the new cost could rise to 7-8 wave-days and the net becomes –2 to –3 (still
positive but more modest). My initial executive-verdict –1 to –3 was the conservative
bracket; this more careful pass gives –3 to –5 as my central estimate.

---

## §3 Per-Compromise-mint transformation

The brief notes "14+ Compromise mints (Compromise #30 + #31 + #32-#44 + #45 collisions)".
Resolution of #45 collision is mandatory first.

**#45 collision resolution** (4 sites mint #45 in the post-critique registry; they are
distinct compromises and need distinct numbers):

| Site | What it claims | Canonical # |
|---|---|---|
| C3 §6.1 (MAL-BIND) | ML-KEM-768 MAL-BIND-K-CT/K-PK binding-properties | **#45** (chronological priority; C3 was the first to land at 79c99aa5) |
| L10 §7 (multi-stanza wire-cost) | HpkeMultiBase O(N) wire-cost asymptotic > 32 recipients | **#46** (rename L10 #45 → #46) |
| L11 §4 (collaborative-edit-via-re-drop) | True multi-writer encrypted-CRDT not supported at v1-beta | **#47** (rename L11 #45 → #47) |
| Q3 §7.1 (K_Atrium-leak) | K_Atrium leak discloses Atrium-membership-fingerprint | **#48** (rename Q3 #45 → #48; further generalized to MembershipSet-shape-leak under M3) |
| Q3 §7.1 (member-storage-host insider) | Atrium-member-storage-host trust-boundary collapse | **#49** (rename Q3 §7.1 #46 → #49) |
| C3 §6.6 (multiformats stewardship) | Permanence-stewardship dependency disclosure | **#50** (rename C3 §6.6 candidate #46 → #50) |
| C3 §6.9 (Tauri NAPI-RS marshaling) | NAPI-RS marshaling-boundary side-channels | **#51** (rename C3 §6.9 candidate #47 → #51) |

Post-rename, the unified Compromise table is **#31-extension + #32-#51** = 20 compromises.

### §3.1 Per-Compromise transformation

| # | Title | MembershipSet transformation | Cite |
|---|---|---|---|
| #31-ext | Drop-bundle composition with encrypt-to-recipient ("forever-valid" = "within recipient's key-retention window") | **(b) SIMPLIFIED**: per-MembershipSet-Kind retention-window language ("MembershipSet member's `shared_key` retention is ≥1-year-grace"); UNIFIES across the 4 fanout sites. | M3 §2.6 #22 |
| #32 | ML-KEM-768 Decap CCA side-channel surface | **(c) UNCHANGED** | C3 §6.1 cite |
| #33 | Coercion / xkcd-538 OUT-OF-SCOPE | **(c) UNCHANGED** | L5-C2 |
| #34 | Password-knowledge implies full access | **(c) UNCHANGED** | L5-C3 |
| #35 | Compromised-device retroactive decryption (no past-content FS) | **(b) SIMPLIFIED**: per-Kind framing — `Kind::DeviceMesh` MembershipSet member-compromise + `Kind::Atrium` member-compromise have parallel disclosure rows; one structural language ("member-compromise within MembershipSet-Kind X retains MembershipSet-shared-key for retention-window Y"). | L5-C4 |
| #36 | RAM-residency / coredump / swap OUT-OF-SCOPE | **(c) UNCHANGED** | L5-C1 |
| #37 | No TEE / sealed-enclave attestation | **(c) UNCHANGED** | L5-C5 |
| #38 | Physical-presence side-channels OUT-OF-SCOPE | **(c) UNCHANGED** | L5-C6 |
| #39 | Supply-chain dependency-pinning posture | **(c) UNCHANGED** | L5-C7 |
| #40 | Reproducible-builds + SLSA-3+ posture | **(c) UNCHANGED** | L5-C8 |
| #41 | Cross-device-sync UX-vs-cryptographic boundary (compromised-device-on-mesh) | **(e) RENAMED → "MembershipSet-Kind::DeviceMesh compromised-member-on-set"**: explicit Kind discriminator strengthens the framing (the property is structural-to-DeviceMesh-MembershipSet not bespoke); composes with #35 + #48 cleanly. | L5-C10 |
| #42 | Layer-C FS-gap (HPKE-mode-base recipient long-term sk decrypts forever) | **(c) UNCHANGED** | L8/Am13 |
| #43 | Envelope metadata leakage to untrusted relays | **(b) SIMPLIFIED**: per-MembershipSet-Kind metadata-leak disclosure (Atrium-Kind = directed-graph + member-DID-list; DeviceMesh-Kind = device-list visible but less-sensitive; SingleDevice-Kind = no member-list to leak). | L6/§6.1 |
| #44 | BSI long-term-confidentiality OUT-OF-SCOPE | **(c) UNCHANGED** | L5-C-LTC1 |
| #45 (C3 §6.1) | MAL-BIND-K-CT/K-PK binding-properties | **(b) SIMPLIFIED**: per-MembershipSet-Kind exception list (caller-supplied-pubkey flows = ExecuteWorkflow + RecoveryHook in any Kind; defense via FIPS 203 validate_public_key() per U31, generalized). | C3 §6.1 |
| #46 (L10 #45 rename) | HpkeMultiBase O(N) wire-cost asymptotic > 32 recipients | **(b) SIMPLIFIED**: per-MembershipSet-Kind recommended max (Atrium 32; DeviceMesh typical 5; SingleDevice 1). | L10 §7 |
| #47 (L11 #45 rename) | Collaborative-edit-via-re-drop accepted v1-beta trade-off | **(c) UNCHANGED** | L11 §4 |
| #48 (Q3 #45 rename + GENERALIZED) | **MembershipSet-shape-leak (shared_key compromise → MembershipSet-fingerprint forward+backward; recovery via fork)** | **(e) RENAMED + GENERALIZED**: subsumes Q3's K_Atrium-leak as `Kind::Atrium` instance; adds DeviceMesh-Kind + (degenerate) SingleDevice-Kind variants. ONE Compromise row replaces 4 parallel per-fanout disclosures. | M3 generalization of Q3 §7.1 |
| #49 (Q3 §7.1 #46 rename) | MembershipSet-member-acting-as-storage-host trust-boundary collapse | **(b) SIMPLIFIED**: per-Kind storage-host-trust composition row. | Q3 §7.1 |
| #50 (C3 §6.6 #46 rename) | Permanence-stewardship dependency disclosure (multiformats + BLAKE3 + libcrux + RustCrypto + McMillion-hpke) | **(c) UNCHANGED** | C3 §6.6 |
| #51 (C3 §6.9 #47 rename) | Tauri NAPI-RS marshaling-boundary side-channels | **(c) UNCHANGED** | C3 §6.9 |

**NEW Compromise minted by MembershipSet unification:** none beyond the renamed/generalized #48
(MembershipSet-shape-leak). The unification doesn't OPEN new disclosures; it cleanly
re-frames existing ones with per-Kind discriminators. **Verdict: Compromise count net-neutral
(20 before any consolidation; 20 after MembershipSet) BUT 4-5 rows simplify in prose by
collapsing per-fanout language into per-Kind-parameterized language.**

---

## §4 Per-invariant transformation

| Inv | Title | MembershipSet transformation | Cite |
|---|---|---|---|
| Inv-15 | Signature-bundle-CID identifier discipline (Compromise #30 closure; 3-layer-decomposition) | **(c) UNCHANGED** | Recently-merged ratification 2026-05-27 (commit 2172cb6d HEAD); M1c §10.2 explicitly: Inv-15 closure is at the IDENTITY layer, not the MembershipSet layer. |
| Inv-16 | Envelope-layer-unification + codepoint-dispatch + AAD-binding + strict-decode + canonical-TLV + sender-DID + replay-window | **(b) SIMPLIFIED MODESTLY**: phrasing references MembershipSet-realized envelope variants uniformly instead of enumerating 4 sites. Inv-16 clause "multi-recipient drops use HpkeMultiBase variant" generalizes to "MembershipSet-realized envelopes use the MembershipSetEncryption codepoint family"; clauses U18 + U19 + U20 generalize per §2. | 9-eyes registry §4 |
| Inv-17 | Hybrid-cryptography-mandatory floor (PQ + classical) | **(c) UNCHANGED** | Crypto-agility floor; orthogonal. |
| Inv-18a | Codepoint-registry-discipline + IANA-disjoint range + CodepointLifecycle | **(b) SIMPLIFIED**: registry row count reduces (4 fanout codepoints → 1 family with Kind discriminator); discipline language identical. | C2 R-C8 + C3 §6.2 split |
| Inv-18b | Metadata-disclosure paired-SealedSender-slot discipline | **(b) SIMPLIFIED**: one paired-slot rule for the MembershipSetEncryption family instead of 4 per-fanout pair-reservations. | C2 R-C8 |
| Inv-18c | CodepointLifecycle typed-state | **(c) UNCHANGED** | Decoder-state machine. |
| Inv-19 | Encryption-substrate keying-function CRDT-input discipline | **(b) SIMPLIFIED via Path-A.5 + MembershipSet API hardening**: "every Benten KDF reads either (a) immutable substrate inputs (Version-Node-CID, codepoint, BE encoding, TLV) OR (b) a MembershipSet-shared-key whose generation is CRDT-resolved per Inv-20". Two clean cases instead of open-ended discipline. | L11 + Path-A.5 §5.3 |
| **Inv-20 (NEW; MINTED BY THIS ANALYSIS)** | **MembershipSet primitive invariant** | **(NEW)**: see below. | M3 mint |

### §4.1 Inv-20 — MembershipSet primitive invariant (proposed)

**Phrasing (proposed).**

> Every Benten encrypt-to-N-recipients use site (N≥1) dispatches through the single
> `MembershipSet` primitive with codepoint-discriminated `MembershipSetKind` variant
> (Atrium / DeviceMesh / SingleDevice; `#[non_exhaustive]` per U10 discipline). The
> primitive provides: (a) `shared_key: K_Set` distributed via multi-stanza-HPKE-Encap to
> each member's HPKE-pubkey; (b) FORK-ONLY rotation discipline (parent-MembershipSet
> retains historical shared_key on member-leave; child-MembershipSet mints fresh
> shared_key on fork); (c) per-stanza AAD-binding tuple `(codepoint, body-CID,
> sorted-member-DID-list, sender_did, stanza-index, member-key-generation,
> membership_set_id, membership_set_generation)` (per U17 + U19 + U20 absorbed); (d)
> per-recipient unlinkability INVARIANT (per U25 absorbed); (e) generation-CRDT-vector
> per-member-DID partition (per U42 + U20 absorbed); (f) Path-A.5 K(V) discipline at
> the API boundary (MembershipSet ops type-restrict payload to immutable
> Version-Node-CID).
>
> Composes-with Inv-15 (payload-identity), Inv-16 (envelope-layer), Inv-17
> (hybrid-floor), Inv-18 (codepoint-registry + metadata-disclosure + CodepointLifecycle),
> Inv-19 (KDF-input-immutability).
>
> Enforcement plan: (1) crate-discipline scanner: every `encrypt_to_n` callsite in
> `benten-crypto-suite` + `benten-engine` + `benten-drop` + `benten-id` MUST dispatch
> via `benten_membership_set::MembershipSet::encrypt_to_set`; direct multi-stanza-HPKE
> calls are forbidden outside the `benten-membership-set` crate itself.
> (2) cite-drift-detector extension: every per-Kind handler MUST be present (exhaustive
> `match` on MembershipSetKind in every consumer; `#[non_exhaustive]` ensures additions
> are explicit). (3) property tests: round-trip + fork-survival + generation-vector-merge
> per-Kind.

**Composition with prior invariants.** Inv-20 SUBSUMES Inv-19's clause-b ("MembershipSet-
shared-key" branch); it SPECIALIZES Inv-16's "multi-recipient" sub-clause; it
INSTANTIATES Inv-18a/b discipline for the MembershipSetEncryption codepoint family.

**Why not merge Inv-20 into Inv-16 or Inv-19?** Inv-16 is envelope-layer; Inv-19 is
KDF-input layer. Inv-20 is the API-primitive layer ABOVE the envelope (it provides
inputs INTO Inv-16-conformant envelopes). Audit-firm reading is cleaner with three
distinct layers + named-cross-references than one mega-invariant. C3 §6.2 F2's
Inv-18 split-rationale applies analogously here.

**Confidence on Inv-20 phrasing.** MED-HIGH. The clause list is comprehensive per my
walk of U17/U18/U19/U20/U25/U42/Q3-§7.1-suggested-invariants; M2 will refine the exact
phrasing alongside the primitive's final API. Defer the FINAL wording to M2's primitive
+ this M3 analysis joint synthesis.

---

## §5 AtriumPolicy → MembershipSetPolicy generalization (specifically)

AtriumPolicy ratified at `e90900b4` mints 5 D1-D5 + 2 deferred decisions (D6 refresh-source
mechanism + D7 single-vs-threshold admin). Per-decision transformation under
`MembershipSetPolicy` generalization:

| D# | Decision | MembershipSetPolicy transformation |
|---|---|---|
| **D1** | `atrium_policy_cid` in AAD? Answer: NO at v1-beta; YES to single `policy_version_at_seal: u32` in BindingContext. | **(b) SIMPLIFIED → `membership_set_policy_version_at_seal: u32`**. Same answer for ALL Kinds (single u32 sufficient; per-Kind policy resolution via `(membership_set_id, policy_version)` tuple at verifier-side). Net: ONE AAD field rule across Kinds, not three. |
| **D2** | Hybrid grandfather rule (grandfather `valid_until` ceilings + new `refresh_required` applies forward on first verifier-touch) | **(c) UNCHANGED**. Semantic identical per-Kind. The Matrix `m.room.history_visibility` precedent cited in AtriumPolicy §1.2 applies identically. |
| **D3** | `refresh_required` v1-beta-CODEPOINT-RESERVE + impl at Phase-4-Meta-Composing | **(c) UNCHANGED**. UX-coupled phase placement; per-Kind impl scheduling identical. |
| **D4** | Admin entity = single-admin-DID + content-addressed signed CBOR-tagged Atrium-replicated Node | **(d) EXPANDED PER-KIND — the substantial generalization**. Per-Kind admin semantic: `Kind::Atrium` = explicit `admin_did: PolicyAdminDid` (current AtriumPolicy.admin_did field unchanged); `Kind::DeviceMesh` = **user IS admin** via DAK (degenerate `admin_did = principal_did` by construction; signature is the user's own hybrid-sig with the user's principal key); `Kind::SingleDevice` = N/A (single-device MembershipSet has no policy beyond UCAN scope's own exp). **Schema generalization**: `MembershipSetPolicyPayload { schema_version, membership_set_id, membership_set_kind, policy_version, fields..., signed_by: MembershipSetPolicyAdminEntity, signature }` where `MembershipSetPolicyAdminEntity` is itself a Kind-discriminated enum (`AtriumAdminDid(Did) \| DeviceMeshUserPrincipal(Did) \| SingleDeviceNone`). |
| **D5** | Default policy = `permissive_default()` (no Atrium-level ceiling) | **(c) UNCHANGED**. Per-Kind `permissive_default()` constructor returns sensible defaults for each Kind; semantic identical (no policy ceiling beyond UCAN scope `exp`). |
| **D6** (deferred) | Refresh source mechanism = OCSP-style fresh-attestation pull | **(c) UNCHANGED**. Refresh mechanism orthogonal to MembershipSet Kind (where applicable). DeviceMesh-Kind MembershipSet's refresh-authority is the user's principal-DID (signs FreshAttestation with principal-keypair); Atrium-Kind's refresh-authority is the Atrium's `refresh_attestation_authorities` set. Per-Kind authority-resolution rule. |
| **D7** (deferred) | Admin = single-DID or threshold-of-N | **(d) EXPANDED PER-KIND** — threshold-admin is ONLY meaningful for `Kind::Atrium` (multiple potential admins). For `Kind::DeviceMesh`, user IS sole-admin by construction (threshold would mean threshold-of-user's-own-devices, which is a device-recovery concern not an admin concern). For `Kind::SingleDevice`, N/A. |

**Net impact on AtriumPolicy → MembershipSetPolicy refactor:**

- Schema fields reduce from per-Kind-bespoke to one parameterized schema with per-Kind variant fields.
- `AtriumPolicy::permissive_default()` becomes `MembershipSetPolicy::permissive_default(kind: MembershipSetKind)`.
- Issuer-side clamp (AtriumPolicy §3.1) becomes Kind-discriminated.
- Verifier-side rogue-issuer defense (AtriumPolicy §3.2 step 3) generalizes per-Kind.
- AtriumPolicy crate `crates/benten-atrium-policy` becomes `crates/benten-membership-set-policy` (or lives inside `crates/benten-membership-set` alongside the primitive — M2 call). New crate has ~600-800 LOC (vs AtriumPolicy's planned ~500-700 LOC); +100-200 LOC net for the per-Kind dispatch.
- Wave-day cost: AtriumPolicy estimated ~5-7 wave-days at `e90900b4`; MembershipSetPolicy generalization estimate ~6-8 wave-days (+1 day for per-Kind dispatch; folded into Wave-MS-PRIMITIVE rather than separate Wave-G).

**Wave-folding implication.** AtriumPolicy was scheduled as **Wave-G** under §7 of the
F-full R0 plan-doc skeleton (per `e90900b4` §1.1). Under MembershipSet unification,
**Wave-G FOLDS INTO Wave-MS-PRIMITIVE** — AtriumPolicy and MembershipSet ship from the
same crate (or sibling crate in the same wave); no separate Wave-G dispatch needed.
This is the wave-merge alluded to in §1 Task 6.

---

## §6 Simplified registry (final form)

**Renumbering scheme.** Original 28 unified amendments (U1-U40 with gaps) + post-critique
additions get re-keyed as **E1-EN** ("E" = Elegant; explicit MembershipSet-unification
context). Compromises keep #-numbers per §3 canonical resolution. Invariants Inv-15
through Inv-20.

### §6.1 Simplified amendment registry (E1-E25)

| E# | Title | Source | Severity | Disposition | MembershipSet transform | Wave-days |
|---|---|---|---|---|---|---|
| **E1** | Codepoint committed in AAD/info | U1 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E2** | Strict-decode; no cross-variant fallback | U2 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E3** | Canonical TLV length-injectivity | U3 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E4** | Sender-DID + member-DID-list in AAD (non-Vault MembershipSet variants) | U4 (simplified) | LOAD-BEARING | v1-beta-LOAD-BEARING | SIMPLIFIED | (in base) |
| **E5** | Replay-window (per-MembershipSet-Kind exclusion table) | U5 (simplified) + R-C2 + U28 | LOAD-BEARING | v1-beta-LOAD-BEARING | SIMPLIFIED | (in base) |
| **E6** | Bernstein-Persichetti Decap CT mitigation | U6 + Compromise #32 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E7** | BE codepoint on-wire | U7 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E8** | Codepoint registry IANA-disjoint + cite-drift scanner + MembershipSetEncryption family | U8 (simplified) | LOAD-BEARING | v1-beta-LOAD-BEARING | SIMPLIFIED | (in base, –0.3) |
| **E9** | `EnvelopePayload` `#[non_exhaustive]` + typed-reject + MembershipSetEncryption variant | U9 (expanded) | LOAD-BEARING | v1-beta-LOAD-BEARING | EXPANDED | +0.1 |
| **E10** | `BindingContext` `#[non_exhaustive]` + typed-reject + MembershipSetSeal variant | U10 (expanded) | LOAD-BEARING | v1-beta-LOAD-BEARING | EXPANDED | +0.1 |
| **E11** | Escape codepoint + experimental range + EnvelopeShape axis | U11 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E12** | Nonce-length variant discrimination | U12 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E13** | FS-gap honest-disclosure + MLS-PQ + CGKA codepoint reservation | U13 (simplified) + Compromise #42 | LOAD-BEARING | v1-beta-LOAD-BEARING | SIMPLIFIED | (in base) |
| **E14** | `aad_version: u8` prefix | U14 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E15** | Did multikey + Did::Unknown | U15 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E16** | CodepointLifecycle typed-state | U16 | LOAD-BEARING | v1-beta-LOAD-BEARING | UNCHANGED | (in base) |
| **E17** | **MembershipSet primitive (Atrium/DeviceMesh/SingleDevice Kind; multi-stanza-HPKE distribution; FORK-ONLY rotation; per-stanza AAD; generation-CRDT-vector)** | NEW (absorbs U17 + U19 + U25 + R-C3 + L11 U42 + Q3 §7.1 U42-suggest + Path-A.5 K(V)) | LOAD-BEARING | v1-beta-LOAD-BEARING | NEW (the unification primitive) | **+3.5-4.5** (Wave-MS-PRIMITIVE canary-first) |
| **E18** | Triple-CID model parameterized over MembershipSet (`plaintext_cid_local + plaintext_cid_membership_set + envelope_blob_cid`; `plaintext_cid_membership_set = HMAC-SHA256(K_MembershipSet, plaintext_cid_local)`) + dedup_scope as MembershipSet-Kind | U18 (simplified) + Q3 Option I generalization + Q3 §7.1 U41-suggest dedup_scope_id | LOAD-BEARING | v1-beta-LOAD-BEARING | SIMPLIFIED | (in MembershipSet wave; –1.5 vs Q3 §7.2 baseline) |
| **E19** | MembershipSet-generation tracking (CRDT-vector per-member-DID partition; replaces U19 recipient_key_generation + U20 k_principal_generation + U42-L11) | (absorbs U19 + U20 + U42-L11) | LOAD-BEARING | v1-beta-LOAD-BEARING | SIMPLIFIED | (in MembershipSet wave; –2.5 vs baseline) |
| **E20** | `PermissionOperation::ExecuteWorkflow` variant + per-Kind sub-arms (Atrium + DeviceMesh; not SingleDevice) + per-request-nonce | U21 (expanded) + L11 U21-ext + C3 §6.4 R4 | LOAD-BEARING | v1-beta-LOAD-BEARING | EXPANDED | +0.3 |
| **E21** | Sealed-Sender additive codepoint family slot (paired with MembershipSetEncryption family) | U22 (simplified) + Inv-18b discipline + C2 R-C1 | LOAD-BEARING (slot) + RECOMMENDED (impl) | v1-beta-CODEPOINT-RESERVE + v1-GM-DEFER impl | SIMPLIFIED | (in base; –0.2) |
| **E22** | Per-relay-unlinkability + universal size-class buckets + cover-traffic deferral + DID-rotation deferral | U23 + U24 + R-C7 + U26 + U27 | MED-HIGH / DEFERRABLE per row | mixed | mixed | (in base; –0.2) |
| **E23** | Cross-ecosystem-identifier emit-discipline (per-Kind map row) + DAG-CBOR outer framing | U29 + U30 + C3 §6.7 R7 | LOAD-BEARING / RECOMMENDED | v1-beta-LOAD-BEARING | SIMPLIFIED | (in base; –0.3) |
| **E24** | Impl-engineering pack (libcrux + XChaCha20 + NAPI canonical_binding + NAPI opaque-handle + MembershipSetHandle + wasm_js cfg + cancel-safety + Argon2idParameterTier per-Kind) | U31 + U32 + U33 + U34 (expanded) + U35 + U36 + L10 U41 (expanded) | LOAD-BEARING | v1-beta-LOAD-BEARING | mixed | (in base; +0.4 net) |
| **E25** | Audit-deliverable pack (golden-vector corpus + dudect CI + kani injectivity + THREAT-MODEL.md + SECURITY-PROOFS.md + perf-bench infra + offline-replay carve-out + drop-bundle predecessor-CID chain + MAL-BIND-K-PK doc + cite-drift cross-language config-mirror + per-Kind discipline) | U37 (simplified) + U38 + U39 (simplified) + U40 (simplified) + C5 SECURITY-PROOFS + L10 U43 + L11 U43 + L11 U44 (simplified) + C3 §6.1 U41 + C3 §6.5 R5 + C3 §6.8 U42 + AtriumPolicy 5 D1-D5 → MembershipSetPolicy generalization (per §5) | mixed | mixed | (in base; –2.0 net via Path-A.5 + MembershipSet) |

**Total amendment count: 25 E-rows** (down from 28 unified U-rows + post-critique additions
nominally ~40-45 amendments when counting C2 R-C1..R-C10 + C3 U41-U42 + C5 U39-promotion +
L10 U41-U43 + L11 U41-U44 + Q3 5-pieces + AtriumPolicy 5-D's separately). The E-row
consolidation is **net –3 to –5 row-count** versus the most-aggressive amendment-counting
of the consolidated registry; net **substantial doc-shrinkage** across all rows even where
count is unchanged.

### §6.2 Simplified Compromise registry (Compromises #31-ext + #32-#51 = 20 mints + 1 ext)

Per §3 canonical resolution; no further consolidation.

### §6.3 Simplified invariant set (5 invariants)

- **Inv-15** — Signature-bundle-CID identifier discipline (unchanged; recently-merged HEAD)
- **Inv-16** — Envelope-layer-unification + codepoint-dispatch + AAD-binding (simplified prose; per-MembershipSet-Kind uniform language)
- **Inv-17** — Hybrid-cryptography-mandatory floor (unchanged)
- **Inv-18** (a/b/c per C2 R-C8 split) — Codepoint-registry + metadata-disclosure + CodepointLifecycle (simplified per §4)
- **Inv-19** — KDF-input immutability discipline (simplified per Path-A.5 + Inv-20 composition)
- **Inv-20 (NEW)** — MembershipSet primitive invariant (per §4.1)

Total: **5 first-order invariants + 3 sub-invariants** (Inv-18 split into 18a/b/c per
C2 R-C8 + C3 §6.2 ratification). Up from 4 first-order invariants (Inv-15/16/17/18) in
the consolidated registry; the increment is justified by Inv-20 naming the new primitive's
discipline and Inv-19 already-anticipated by L11 (pre-MembershipSet).

### §6.4 Aggregate deltas vs baseline

| Metric | Baseline (9-eyes registry + post-critique consolidated) | Simplified (post-MembershipSet) | Δ |
|---|---|---|---|
| Amendments | ~28 unified + ~14 post-critique = ~42 rows of substantive amendment | 25 E-rows | **–17 rows of registry text** (but only **–3 to –5 net-distinct-amendments**; the rest is consolidation of multi-row clusters into single E-rows) |
| Compromises | 14 (#32-#44 + #31-ext) + 4 collision-#45s = 18 mints OR 21 if counting C3 §6.6+§6.9 candidates | 20 mints + #31-ext (renumbered + 1 generalized via Q3 #45 → #48) | **+2 net mints from collision-resolution; –0 from MembershipSet (rename-only)** |
| Invariants | 4 (Inv-15/16/17/18) | 5 (Inv-15/16/17/18/19/20) + Inv-18 split into 3 sub | **+1 first-order (Inv-20) + +1 (Inv-19) from L11 + 1 split per C2 R-C8** = **+3 row-count net** but bounded |
| Per-L4 §7 LOC estimate baseline | ~3500-4500 LOC across crypto-suite + drop + sync + new crates | ~3100-4100 LOC (–400 net via MembershipSet consolidation; +500 for new benten-membership-set crate; –500-900 from consolidating 4 parallel implementations into one) | **–200 to –400 LOC net** |
| Per-L5 §5 audit-page baseline | ~80-100 pages (THREAT-MODEL.md + SECURITY-POSTURE.md updates + KEY-LIFECYCLE.md + per-amendment paragraphs) | ~70-90 pages (–8 to –12 pages via per-Kind-parameterized-row collapse) | **–8 to –12 audit pages** |
| Wave-day estimate baseline | ~80-101 wave-days (per consolidator §6.5 + L11 +10-14 + Q3 +5.75 + AtriumPolicy +5-7 + C5 +1.5-3 SECURITY-PROOFS + L10 +3-5 PERF-1 - some overlap) | ~75-96 wave-days | **–4 to –5 wave-days net** (range = –1 conservative to –5.5 central; my best central estimate **–4.3 to –5.5**) |

**Headline aggregate Δ.** Per the central estimate:
- **–3 to –5 amendments**
- **+1 invariant (Inv-20) + simplification of Inv-16/18/19 prose**
- **–200 to –400 LOC**
- **–8 to –12 audit pages**
- **–4 to –5 wave-days**

---

## §7 Wave-sequencing implications

### §7.1 Wave-merge: AtriumPolicy Wave-G folds into Wave-MS-PRIMITIVE

Per §5, AtriumPolicy's planned Wave-G (~5-7 wave-days at `e90900b4`) collapses into
Wave-MS-PRIMITIVE — the same crate (or sibling crate in the same wave) ships both
primitive + policy generalization. **Net: –1 wave dispatch, ~+1 wave-day per crate (the
per-Kind dispatch overhead) absorbed within Wave-MS-PRIMITIVE.**

### §7.2 New wave: Wave-MS-PRIMITIVE (canary-first; required)

Because MembershipSet is owned-by-many-crates (the 4 fanout sites are in `benten-crypto-suite`,
`benten-engine`, `benten-drop`, `benten-id`) but the substrate is ONE crate (new
`benten-membership-set`), this wave MUST land canary-first per the standard "canary-first
for parallel-N waves" discipline (per
`feedback_canary_first_parallel_implementation.md`; G16-A + G21-T1 precedents).

**Wave-MS-PRIMITIVE composition:**

1. **Canary**: `benten-membership-set` crate lands first (M2's primitive design realized;
   ~3.5-4.5 wave-days). Includes:
   - `MembershipSet { id, kind, members, shared_key, policy, generation, parent_membership_set_id }` struct
   - Operations: `create / add_member / remove_member / encrypt_to_set / fork / update_policy` (with `dedup_blind_cid` as sibling trait per M2's call)
   - Per-Kind constructors: `MembershipSet::new_atrium / new_device_mesh / new_single_device`
   - MembershipSetPolicy (generalized per §5)
   - Golden vectors + kani harness + property tests
   - NAPI bindings (MembershipSetHandle per U34 expansion)
2. **Fan-out (4 parallel agents AFTER canary merges)**:
   - Sub-wave-MS-A: `benten-crypto-suite` migrates HpkeMultiBase usage to MembershipSet (~1 day)
   - Sub-wave-MS-B: `benten-drop` migrates Drop-bundle multi-recipient to MembershipSet (~1.5 days)
   - Sub-wave-MS-C: `benten-engine` migrates multi-device-key-wrap to MembershipSet (~1.5 days)
   - Sub-wave-MS-D: `benten-id` (or wherever Atrium-membership lives) migrates K_Atrium-distribution to MembershipSet (~1.5 days)
3. **Wave-MS-FINAL** (single agent): cross-crate integration test + final goldens (~0.5 day)

**Total Wave-MS-PRIMITIVE: ~9-10 wave-days** (matches the +4.8-5.8 new cost calculus in
§2.13 plus the migration sub-waves which were ALREADY counted in the baseline ~80-101
wave-days for the 4 fanout sites).

### §7.3 Other wave-sequencing impacts

- **Wave-C (Layer-C HPKE-multi-stanza)** per F-full §8 wave decomposition: SHRINKS because
  HpkeMultiBase is no longer wave-C's central deliverable (it becomes a Sub-wave-MS-A
  responsibility). Wave-C still owns Layer-C HPKE-mode-base + Compromise #42 disclosure
  + single-recipient drop flow. **–1 to –2 wave-days off Wave-C.**
- **Wave-G (AtriumPolicy)**: DELETED per §7.1. **–5 to –7 wave-days off Wave-G (folded
  into Wave-MS-PRIMITIVE).**
- **Wave-D (Layer-D DAK substrate + device-link + remote-permission + multi-device-key-wrap)**:
  PARTIAL SHRINK — multi-device-key-wrap becomes Sub-wave-MS-C; DAK + device-link +
  remote-permission stay. **–1.5 to –2 wave-days off Wave-D.**
- **Wave-Q3 (Q3 Option D triple-CID + K_Atrium + dedup_scope)**: ABSORBED into
  Sub-wave-MS-D. **–5.75 wave-days off Q3 estimate** (per Q3 §7.2 incremental).
- **Wave-L11-CRDT (L11 U41-U44 + Inv-19)**: PARTIALLY ABSORBED (U41 dissolves under
  Path-A.5; U42 absorbs into MembershipSet generation; U43 + U44 standalone). **–4 to –5
  wave-days off L11 estimate.**

**Net wave-sequencing implication:** 1 NEW wave (Wave-MS-PRIMITIVE), 1 wave deleted
(Wave-G), ~3 waves shrink (Wave-C, Wave-D, Wave-L11-CRDT). **Net wave-count Δ ≈ 0**;
**net wave-day delta = –4.3 to –5.5** as computed in §2.13.

### §7.4 Canary discipline reminder

**Wave-MS-PRIMITIVE MUST go canary-first** because:
- `benten-membership-set` is the owner of the new primitive API the 4 fanout sites
  consume.
- Parallel dispatch of all 4 sub-waves without canary risks 4-way merge-conflict on
  the primitive's API shape if the M2-design drifts during implementation.
- Per `feedback_canary_first_parallel_implementation.md`: "when one track owns API
  surface others consume, dispatch canary FIRST; fan out parallel-(N-1) only after
  canary merges."

---

## §8 Cost-benefit summary + wave-day delta

### §8.1 The honest reckoning

**Verdict: NET-ELEGANCE-WIN (MODEST).** Confidence MED-HIGH.

The win is real but not the dramatic "12-primitives-irreducible" transformative shift
the brief framing might suggest. The win is dominated by:

1. **The 4-site multi-stanza-HPKE-fanout consolidation** (M1c §10.3's central
   observation; cleanly absorbs U17 + U19 + U25 + multi-device-wrap + K_Atrium-dist +
   Drop-bundle + federation; this is the largest single source of savings).
2. **Q3's K_Atrium-as-instance-of-K_DedupScope insight** (Q3 §5.6 Option I framing
   already anticipated MembershipSet by another name; ratifying MembershipSet realizes
   the elegant-shape Q3 named).
3. **AtriumPolicy → MembershipSetPolicy generalization** (per-Kind dispatch instead of
   Atrium-bespoke; ~+1 day Kind dispatch absorbed by the wave-merge savings).
4. **Path-A.5 K(V) API-boundary enforcement** (MembershipSet's `encrypt_to_set(payload:
   VersionNodeCid, ...)` type-restriction enforces Inv-19's simplified form at the
   compiler-checked layer).

The losses are real too:
1. **+1 invariant** (Inv-20) — modest audit-surface increase.
2. **Per-Kind typed-variants in ~6 amendments** — mechanical sub-arms; bounded but
   not free.
3. **New crate `benten-membership-set`** — adds a workspace member; modest workspace
   bookkeeping.
4. **Canary-first Wave-MS-PRIMITIVE adds 1 sequencing constraint** — modest serialization
   cost (~+0 wave-days; the canary IS the work, not an additional wait).

### §8.2 Quantitative deltas (central estimate)

| Metric | Δ |
|---|---|
| Amendment count | **–3 to –5** |
| Invariant count | **+1 (Inv-20)** + Inv-19 simplification + Inv-18 split |
| Compromise count | **0 net** (renames + generalizations) |
| LOC | **–200 to –400** |
| Audit pages | **–8 to –12** |
| Wave-days | **–4.3 to –5.5** (conservative bracket: –1 to –3; my central: –4 to –5) |

### §8.3 Sensitivity analysis

**Where the verdict could flip to BREAK-EVEN:**
- If M2's primitive design develops Kind-specific complexity beyond what M1c §10
  predicts (e.g., per-Kind init paths balloon to ~1000 LOC each), the +5.8 new-cost
  estimate could become +8-10 wave-days. With savings unchanged at –10.1 to –10.3,
  net becomes –0 to –2 (break-even territory).
- If M4 red-team surfaces a structural attack against the MembershipSet primitive
  (e.g., cross-Kind confused-deputy through shared codepoint family), defending
  via per-Kind sub-codepoints could undo the U8 + U22 codepoint-registry savings
  (–0.5 day each direction).
- If M5's CGKA survey suggests MembershipSet is the wrong shape for the future
  CGKA-additive variant (e.g., CGKA needs per-epoch-rekeying which doesn't fit
  FORK-ONLY-rotation), the v1-beta-CODEPOINT-RESERVE for MLS-PQ family may need
  different shape, weakening U13's simplification (–0.1 day swing).

**Where the verdict could strengthen to NET-ELEGANCE-WIN (LARGE):**
- If Ben ratifies a "single MembershipSet implementation across language ports
  (Rust + TS + future)" as the canonical cross-language seam, the §3.5g Inv-18a
  cross-language rule-mirror discipline gets cleaner (one primitive crosses NAPI;
  TS implementation is one struct not four).
- If Path-A.5 ratifies more aggressively (e.g., DELETES Path-A as a not-named-option
  per HARD RULE 12), Inv-19's simplification under MembershipSet becomes the
  STRUCTURAL closure of an entire class of L11 hazards, justifying the +1
  invariant.

**Where the verdict could flip to NET-COMPLEXITY-LOSS:**
- Only one structural failure mode: if `dedup_blind_cid` and `fork` end up belonging
  INSIDE the MembershipSet primitive API (not as sibling traits), the API surface
  bloats to ~8 ops which is non-trivial cognitive load. M2's call.
- Or: if the 4 fanout sites DON'T actually share enough structure (e.g., federation
  grants turn out to need per-recipient-policy-customization that MembershipSet's
  uniform `policy` field can't express cleanly). Honest reading: M1c §10.3 strongly
  argues structure IS shared, but a sibling-trait per-fanout-call-site customization
  hook may be needed.

### §8.4 Final recommendation

**Recommend Ben ratify MembershipSet unification at R0**, BUT with two explicit pre-conditions:
1. **M2's primitive design is finalized + Ben-ratified BEFORE Wave-MS-PRIMITIVE
   dispatches**. The canary-first discipline requires the primitive to be a fixed point.
2. **Sibling-trait vs inherent-method placement for `dedup_blind_cid` and `fork` is
   explicitly called by Ben** (M2 surfaces both options + estimates relative cognitive
   load; this M3 analysis assumes sibling-traits; if Ben prefers inherent, M3's wave-day
   delta tightens by ~–0.3 days due to test-family colocation but cognitive load
   increases).

With these pre-conditions, **MembershipSet unification ships at v1-beta-LOAD-BEARING**
absorbing AtriumPolicy + Q3 + L11-U42 work; saves ~4-5 wave-days; cleans the multi-stanza
fanout into one principled substrate. **NET-ELEGANCE-WIN (MODEST) — confidence MED-HIGH.**

---

## §9 Self-assessment + confidence per finding

### §9.1 What I did + how I worked

- Read the 9-eyes consolidated registry in full (`fbdfeb16`, 939 lines), all 5
  critique-round outputs (~3000 lines combined), Q3-revisit (607 lines), AtriumPolicy
  design (782 lines), L10/L11/L12 (~1746 lines combined), Path-A.5 (533 lines), and
  the 3 cataloger outputs (~2000 lines combined). All read via ref-pinned `git show`
  at frozen SHAs per §3.5h cite-drift discipline.
- Walked every amendment row in the consolidated registry + every post-critique
  addition + every D-decision in AtriumPolicy. Per-row transformation per the
  brief's (a)-(e) classification scheme.
- M2's primitive design was running in parallel during my session and did NOT land
  before my output. I worked from the M2 design sketch in the brief (`MembershipSet
  { id, kind, members, shared_key, policy, generation, parent_membership_set_id }`
  + 7 ops + `dedup_blind_cid` + `fork` as "do-NOT-fit" candidates). All M2-dependent
  judgments are flagged "M2-pending" or "M2 will refine" and represent my best-guess
  given the design sketch.
- Worked in worktree-isolation per §3.5; tree-state pre-flight at HEAD `2172cb6d`
  verified clean; all reads via absolute paths under `${WORKTREE_ROOT}`.

### §9.2 Confidence summary per finding-class

| Finding class | Confidence | Reasoning |
|---|---|---|
| 4-site fanout collapse (E17 mint; U17/U19/U25 ELIMINATED) | **HIGH** | M1c §10.3 explicitly anticipates; L9 + Q3 + L10 + L11 all bumped into adjacent slices; the structure is robustly evident across reviewers. |
| Per-Kind typed-variant cost (~6 amendments EXPANDED) | **MED-HIGH** | Sub-arms are mechanical; bounded; estimate is conservative. |
| AtriumPolicy → MembershipSetPolicy generalization (D4 + D7 per-Kind admin) | **MED** | Defensible analysis but Ben may have a different read on the DeviceMesh user-IS-admin framing. |
| Path-A.5 K(V) API-boundary enforcement (Inv-19 simplification) | **HIGH** | Path-A.5 already argues this universally per §6.2; MembershipSet just adds the type-system enforcement. |
| Wave-day delta central estimate (–4.3 to –5.5) | **MED** | Many small per-amendment estimates compound; conservative bracket (–1 to –3) is honest hedge if many small estimates skew high. |
| Inv-20 phrasing (proposed) | **MED-HIGH** | Clause list is comprehensive per my walk; M2 will refine. |
| Compromise renumbering (#45-collision resolution; +#48 generalization) | **HIGH** | Mechanical; 4 distinct mints DO need 4 distinct numbers; chronological priority is clean. |
| Sibling-traits vs inherent-method for dedup_blind_cid + fork | **LOW** | Honest hedge — I assumed sibling-traits; M2's call dominates. |
| Wave-MS-PRIMITIVE canary-first sequencing | **HIGH** | Standard discipline per `feedback_canary_first_parallel_implementation.md`. |
| Wave-G folding into Wave-MS-PRIMITIVE | **MED-HIGH** | AtriumPolicy is small enough to colocate; crate placement (separate vs combined) is M2's call. |

### §9.3 What I could be wrong about

- **M2-pending items**: any judgment dependent on the final primitive API shape may
  need revision. Most flagged in-text.
- **My amendment-count delta** is sensitive to how Ben counts "per-critique additions"
  (some were minor refinements that don't add row-count; others were new amendments).
  If the baseline is counted more aggressively at 40+ rows, my delta of –3 to –5
  understates the row-count savings (which would actually be –5 to –7).
- **My wave-day delta** is sensitive to how the +4.8-5.8 new-cost estimate lands; if
  M2's primitive design adds Kind-specific complexity beyond my read, central estimate
  could weaken toward break-even.
- **My assumption that `Kind::SingleDevice` is worth keeping** is judgment-based; could
  collapse SingleDevice into degenerate DeviceMesh-Kind-of-1 (no real savings; modest
  audit-readability loss).
- **My handling of L10 U41 (Argon2idParameterTier per-Kind)** — this is genuinely
  Kind-specific (only DeviceMesh + SingleDevice Vault MembershipSet have Argon2id); my
  treatment as EXPANDED is correct but the field placement (Vault-context only) means
  the cost is bounded.

### §9.4 Lower-confidence areas (honest disclosure)

- The **cross-Atrium-federation grant** as a 4th fanout site is my read of M1c §10.3
  bullet (4); I did not independently verify this from m1b/m1c text. If federation
  grants are NOT actually a candidate 4th site, the 4-site collapse becomes a 3-site
  collapse and savings tighten slightly.
- The **L11 U41-L11 simplification under Path-A.5** is my reading that K(V) keying to
  immutable Version-Node-CIDs dissolves L11's "re-encryption on merge" discipline; this
  IS what Path-A.5 §4.8 prescribes but the engineering of "MembershipSet enforces K(V)
  input at API boundary" needs M2's primitive-API ratification.
- The **AtriumPolicy D4 admin per-Kind variants** — specifically the DeviceMesh-Kind
  "user IS admin via DAK" framing — is my structural-cleanliness call; an alternative
  framing ("DeviceMesh has no admin; AtriumPolicy doesn't apply") is equally defensible.

### §9.5 What this analysis does NOT cover

- M2's final primitive API ratification (M2 owns)
- Red-team attack catalog against MembershipSet (M4 owns)
- CGKA literature survey for v2 MembershipSet variant (M5 owns)
- Transport-configurability per-Atrium (M6 owns)
- Final wave-day estimate adjudication once all 5 M-panels reconvene
- Adjudication of disagreements with my analysis from M2-M6 (orchestrator's job)

---

## §10 Citations

### §10.1 Inputs (frozen SHAs; ref-pinned per §3.5h discipline)

- **Cataloger outputs**:
  - M1a multi-device sync: `3618e051`
  - M1b Atrium membership + sharing: `1816ea60`
  - M1c key management: `50eb901d`
- **9-eyes consolidated registry**: `fbdfeb16`
- **Critique-round outputs**:
  - C1 elegant-shape: `3f5a4351`
  - C2 composability: `9c548e5f`
  - C3 fresh-eyes cryptographer: `79c99aa5`
  - C4 process-discipline: `6ea9718a`
  - C5 formal-methods: `8e374a9d`
- **Q3-revisit (Option D community lens)**: `23f76e24`
- **AtriumPolicy design**: `e90900b4`
- **L10 perf / wire-size**: `a81d6da2`
- **L11 CRDT / eventual-consistency**: `68eadd0c`
- **L12 dead-code identification**: `7947f32c`
- **Path-A vs Path-B specialist review (Path-A.5 winner)**: `5f50a028`
- **Tree HEAD at analysis**: `2172cb6d` (origin/main fetched 2026-05-27)

### §10.2 Benten internal references

- `docs/V1-FROZEN-INTERFACE.md` items 6 (envelope shape) + 15 (AuthorizationGrant)
- `docs/SECURITY-POSTURE.md` Compromise #30 + #31 + ratified extensions
- `docs/INVARIANT-COVERAGE.md` Inv-1 / Inv-5 / Inv-10 / Inv-13 / Inv-15 (recently-merged HEAD)
- `crates/benten-crypto-suite/src/aead.rs` (LE-vs-BE codepoint cite per L4 IMPL-B1)
- `crates/benten-crypto-suite/src/structural_kdf.rs` (KDF chain; Spike-E Interpretation-B path-tagged form)
- `crates/benten-crypto-suite/src/codepoint.rs` (current cipher_codepoint enum)
- `crates/benten-crypto-suite/INTERNALS.md` (current envelope §6.2 framing)
- `CLAUDE.md` baked-in #5 (crypto-agility), #15 (v1-beta interface freeze), #17 (multi-process), #18 (subgraph-plugin three-layer consent), #19 (extension trust model)
- `feedback_canary_first_parallel_implementation.md` (canary-first sequencing discipline)
- `feedback_engine_primitives_vs_application_layer.md` (12-primitives-irreducible commitment; the framing context for "is MembershipSet a new primitive?")
- `feedback_handoff_top_banner_re_orient.md` (top-banner discipline; honored at file top)
- `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` (the elegant-shape lens this analysis applies)
- HARD RULE — no "later" disposition (`feedback_no_defer_HARD_RULE.md`)
- §3.5h pre-merge JSON validation discipline
- §3.6f SHAPE-not-SUBSTANCE pre-flight (followed in writing this doc — every cite verified at author-time via git show / grep)

### §10.3 External standards + drafts (transitively cited via input artifacts)

- IETF RFC 9180 (HPKE) — multi-stanza-HPKE base + per-stanza AAD discipline
- IETF RFC 8949 (CBOR) §4.2.1 deterministic encoding
- IETF RFC 9421 (HTTP signature) — sibling identifier-discipline precedent
- FIPS 203 (ML-KEM) — Decap CT requirement + validate_public_key()
- NIST SP 800-227 §4.4 hybrid-combiners-with-concatenation
- Bernstein-Persichetti IACR 2024/2051 ML-KEM Decap side-channel
- `draft-sfluhrer-cfrg-ml-kem-security-considerations-04` (MAL-BIND-K-CT/K-PK; per C3 §6.1)
- ANSSI mandatory-hybridation guidance
- BSI TR-02102-1 long-term-confidentiality floor (#44)
- Albrecht-Bellare 2024 (key-committing AEAD; per C5 CT-5)
- libcrux-ml-kem (Cryspen verified ML-KEM with `check-secret-independence`)
- Willow Confidential Sync (`willowprotocol.org/specs/confidential-sync/` per L11 §5)
- Yjs + Serenity Notes + Automerge + CRDX + secsync + Jazz + iroh-docs + Signal Sealed Sender V2 (Path-A.5 §6.1 precedent survey)
- MLS-PQ / draft-prabel CGKA (codepoint reservation per U13)
