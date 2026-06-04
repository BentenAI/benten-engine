# F-full R0 implementation plan — encryption substrate + MembershipSet primitive (Phase-4-Meta-Core)

> **R0.7 — `blake3::keyed_hash` precision + `0x6520` group-AAD blinding (Ben-ratified 2026-06-04).** Two
> freeze-record updates on top of R0.6. **(1) Construction-name precision (no byte change):** everywhere this
> doc writes the MembershipSet set-id commitment / §3.9 gossip topic as `HMAC(K_Set, …)`, the FROZEN primitive
> is **`blake3::keyed_hash(K_Set, …)`** — BLAKE3's native keyed MAC (the membership-set crate has NO hmac/sha2
> dependency; BLAKE3-keyed IS a MAC; truncate-to-32 is the native BLAKE3 output width), reused from the §3.9
> gossip topic; R5 routes both through the real `benten-crypto-suite` keyed MAC over `K_Set`. The abstract name
> `HMAC` is kept as the role-label; the bytes/construction are UNCHANGED. Sites clarified: §3.3 (commitment
> def), §3.9 (gossip topic), §3.10 (`0x6610` 11-field block), §4.1 (`0x6610` row). **(2) `0x6520` group AAD
> BLINDED (NEW wire change):** `0x6520 = LAYER_C_DROP_MULTI_RECIPIENT` (`EnvelopePayload::HpkeMultiBase` —
> the Layer-C **multi-recipient** group send; NOT a MembershipSet) carries the SAME recipient-roster
> social-graph leak (#61-class) as `0x6610`, with the same un-retrofittable-past-freeze deadline → it is
> blinded the same way: **`audience_set_commitment (32B) = BLAKE3(0x01 ‖ lp(did_0) ‖ lp(did_1) ‖ …)`** over the
> canonical SORTED recipient-DID list (lp = u32-BE length prefix) REPLACES the raw sorted recipient-DID-list;
> **`stanza_count` is bound alongside `stanza_index`** (truncation/censorship defense); **`body_cid` becomes a
> self-describing CIDv1** (`0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3`). UNLIKE `0x6610`, `0x6520` is NOT a
> MembershipSet so it carries **NO `membership_set_id_commitment`, NO `membership_set_generation`, NO
> `role_assignments_generation`**. The Layer-C drop-band field widths are PRESERVED (the §4.0
> width-unification-REJECTED note governs — `recipient_count` stays u16 BE per the band). All bindings
> (cross-stanza substitution U17; inter-member non-forgeability) PRESERVED; the relay sees only an opaque
> 32-byte tag. HONEST SCOPE: identity-HIDING not unlinkability (the commitment recurs for a static recipient
> set); full per-send unlinkability = **U25, CODEPOINT-RESERVE for v1-GM**, additive with no wire break. Edit
> sites: §3.3 (new `0x6520` blinded field-set + blinding rationale), §4.0 (`0x6520` row → point at field-set),
> §4.1 (`0x6520` row → point at field-set + new `0x6520` AAD-field-set row). **NOTE:** the f_lc_hpke F-LC-2
> corpus `0x6520` assembler + `F_LC_2_GROUP_STANZA_AAD_HEX` golden are the pre-blinding shape and MUST be
> migrated to this blinded field-set at R4.6/R5 (R0.7 deliberately RE-OPENS `0x6520`, superseding the corpus
> "not re-opened by R0.6" note).
>
> **R0.6 — Sealed-Sender AAD freeze record (Ben-ratified after a 6-lens design council, 2026-06-03).** This
> revises R0.5 with **7 RATIFIED Sealed-Sender envelope-AAD freeze decisions** (the v1-beta wire that freezes
> at G-CORE-9; AAD = authenticated-not-encrypted, i.e. plaintext on the wire): **(1)** `0x6510` single-recipient
> AAD = the minimal-sufficient union (Option A) `{aad_version, codepoint, audience(recipient DID,
> u32-BE-length-prefixed), body_cid, recipient_key_generation}`; **(2)** `0x6610` group per-stanza AAD = the
> BLINDED 11-field set — `audience_set_commitment = BLAKE3(0x01‖lp(did_i)…)` over the sorted DID list +
> `membership_set_id_commitment = HMAC(K_Set,"benten:setid:v1"‖id)/32` (the §3.9 gossip-topic construction)
> replace the prior raw roster + raw set-id, obeying the project's own §3.9 / Compromise #61 blinding posture
> (HONEST SCOPE: identity-HIDING not unlinkability — same commitment recurs for a static group; full per-send
> unlinkability = **U25, CODEPOINT-RESERVE for v1-GM**, additive with no wire break); **(3)** `body_cid` =
> self-describing CIDv1 (`0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3`) on BOTH `0x6510`/`0x6610`, NOT a bare fixed-32
> (CLAUDE.md baked-in #5; restores U3 length-injectivity); **(4)** `stanza_count` bound alongside `stanza_index`
> in `0x6610` (truncation/censorship defense); **(5)** NO `coarse_epoch` on the Drop wire (RULING-1 / §3.10 /
> M-14 — the 1-hr bucket is Layer-D-only; §3.3 prose + Compromise #43 corrected); **(6)** the token-binding
> (§3.11) AAD carries NO `coarse_epoch` (freshness = the token's own UCAN `nbf`/`exp` + the nonce-cache);
> **(7)** REJECTED width-unification — the Layer-C drop band and the MembershipSet band stay separately-frozen,
> codepoint-discriminated byte-strings (recorded so no future round re-litigates it). All binding properties
> (cross-stanza substitution U17; inter-member non-forgeability) are PRESERVED; the relay sees only opaque
> 32-byte tags. Edit sites: §3.3 (field-sets + coarse-epoch prose + blinding rationale + U25 scope), §3.10
> (`0x6610` 11-field AAD), §3.11 (token no-coarse_epoch), §4.0 (width-unification-rejected note), §4.1
> (`sealed_at` row + two new AAD-field-set rows), Compromise #43.
>
> **R0.5 — X-Wing-append + RoleId/MemberRef int-discriminant revision (branch
> `phase-4-meta-core/f-full-r0-plan-r05 @ e4fbfe73`).** [R0.5's own banner entry was absent in R0.5; this
> R0.6 pass restores a placeholder line so the lineage is unbroken — the integrator should expand it from the
> R0.5 commit message / §13 if a fuller entry is wanted. R0.5 applied the X-Wing-label-APPEND correction
> (XWingLabel appended, not prepended; `draft-connolly-cfrg-xwing-kem-10` §6) + the RoleId/MemberRef
> int-discriminant decisions.]
>
> **R0.4 — R4-review fix-pass revision.** This revises the R0.3 canonical R0 (branch
> `phase-4-meta-core/f-full-r0-plan-r1fp-r03`; the R0.2/R0.3 prose lineage carried below) by applying the
> **8 R4-review decisions Ben ratified 2026-06-02**: (1) §3.3 — the `0x6520` group send HONORS Sealed-Sender
> (inner-sender-DID sealed per-stanza, NOT plaintext AAD; F-LC-9); (2) §4.0 — formalize `0x6101`
> (`SymmetricAead` ChaCha20-Poly1305 12-byte; `0x6100` stays `SymmetricAeadXNonce` vault); (3) §3.6.B —
> canonical Admin = Moderator-set ∪ 5 admin-only abilities so Moderator ⊆ Admin (M-11); (4) §4.1 — WIDEN the
> M-19 LE→BE site-list (+`structural_kdf.rs:157`, `varsig.rs:47/107`, `sizes.rs:183`,
> `swap_matrix.rs:1539/1540/1548/1550`); (5) NQ-T2 RATIFIED (bucket ⟂ `valid_until`; strict-no-grace); (6)
> NQ-T3 RATIFIED (frozen 3-field AAD sufficient; runtime enforcement post-v1-beta, non-freeze-gating); (7)
> NQ-T4 RATIFIED (jti-keyed durable nonce-cache; per-device GUARANTEED + user-global best-effort-eventual;
> mint a named Compromise for the pre-sync cross-device window); (8) NQ-C5 RATIFIED
> (bucket=`(raw_unix_secs/3600)*3600` round-down no-jitter; nonce-cache orthogonal, not widened).
>
> **R0.2 lineage (retained).** R0.2 revised R0.1 (`6755ea41`) per the **R1 critic-council triage** (8 lenses;
> 3 BLOCKER + 20 MAJOR + 15 MINOR + 9 OBS; `.addl/phase-4-meta/r1-triage.md` @ `50115446`) + **Ben's 3
> ratified rulings** (Sealed-Sender DEFAULT; Compromise #31 = LAMPS keeps it / revocation → #62; X-Wing =
> real SHA3-256 construction at `0x647A`). The §0.3 verification log was **re-run against HEAD `2172cb6d`**
> that pass (the R0.1 log trusted the CLAUDE.md narrative and inverted two facts — corrected below). **§13 is
> the R1 fix-pass changelog** (every B/M/m/O finding → disposition). Convergence target: R1.2 re-review
> returns 0 BLOCKER/MAJOR.
>
> **Top-banner re-orient (HANDOFF discipline per `feedback_handoff_top_banner_re_orient.md` — read BEYOND
> what is named here).** This is the **finalized ADDL R0 plan-doc** for the **F-full** workstream:
> Benten's encryption-at-rest + encrypt-to-recipient substrate **plus** the unifying **MembershipSet**
> primitive, landing in **Phase-4-Meta-Core** (the v1-public-interface-freeze sub-phase) +
> **Phase-4-Meta-Composing** (UX-coupled), both **pre-`v1-beta`-tag**. It weaves TWO consolidated halves —
> the **encryption arc** and the **MembershipSet arc** — plus the graph-native (GN) and Rust-engine-plugin
> (EP) refinements, into ONE coherent scope. Pipeline: **R0 → R1 → R1.2 (this revision feeds it) → R2 → R3
> → R4 → R5 → R4b → R6 → tag** (per `feedback_addl_pipeline_full_observance`).
>
> **This R0 SUPERSEDES** (as the canonical F-full scope, for the pipeline going forward): M-CONS-FINAL
> (`a99dd0c7`) + the Option-F+ 9-eyes consolidated registry (`fbdfeb16`) + the e2r-ffull-scope-review
> (`220b5aae`), as integrated by the GN-1/GN-2/EP-1 refinements (`859fa51e`/`61f24553`/`7520ae4e`). It
> records the ratified decisions as **settled**; the ADDL pipeline still runs on this plan-doc (the
> consolidation work is R0-INPUT, not a substitute for R1+). **Where the two source docs CONFLICT on a
> wire value (the `0x6380/0x6390` collision), the 9-eyes registry wins** (later/more-specific; §0.4).
>
> **Read BEYOND what is named here** if you arrive cold: the consolidation chain in §0; CLAUDE.md
> baked-in #1/#3/#5/#7/#8/#15/#16/#17/#18/#19; the tracked ground-truth docs (`docs/INVARIANT-COVERAGE.md`,
> `docs/SECURITY-POSTURE.md`, `docs/V1-FROZEN-INTERFACE-DEFERRED.md`, `docs/ARCHITECTURE.md`,
> `docs/HOW-IT-WORKS.md`, `docs/PLUGIN-MANIFEST.md`); MEMORY.md disciplines (no-defer HARD RULE 12,
> extra-reflection-pass, push-application-layer-composition-before-engine-extension, iterate-to-convergence,
> canary-first, surface-arch-decisions-under-auth, plain-English-with-prediction).
>
> **Tree HEAD at write:** `2172cb6d` (= origin/main; fetched + verified at session start; clean tree). The
> `benten-membership-set` crate does **NOT** exist (this R0 plans it; ground-truth `ls crates/` → 14 crates,
> no membership-set). `docs/CRYPTO-CODEPOINTS.md` + `docs/future/compute-marketplace.md` do **NOT** exist
> (this R0 plans them). Branch: `phase-4-meta-core/f-full-r0-plan-r1fp`. Date: 2026-06-02.

- **Role.** R0-author — senior systems architect producing ONE coherent finalized plan-doc that
  consolidates the decided scope, names per-layer/per-primitive design, the frozen-interface inventory,
  the crate + wave decomposition, cost/timeline, exit criteria, and the R1-seeding open questions.
  ADVISORY/PLANNING; NO implementation code; NOT an implementer.
- **Posture.** Ratified decisions (§2) + Ben's 3 R1-pass rulings (§2.0) are **FIXED INPUTS** — applied, not
  re-litigated. Where a question is genuinely open, it is named as an **R2/R3 question** (§10), not given
  an invented resolution. Every not-now disposition carries a **HARD RULE 12** disposition (OUT-OF-SCOPE /
  BELONGS-NAMED-NOW / DISAGREE-WITH-EXPLANATION).
- **Ground-truth.** Every code/file/invariant/Compromise/codepoint citation is `git show`/`grep`-verified
  against HEAD `2172cb6d` before assertion (see the **re-run** §0.3 verification log).

---

## §0 Provenance + lineage

### §0.1 The consolidation chain (what this R0 supersedes, and in what order)

F-full emerged during the Phase-4-Meta-Core arc as the **confidentiality half** of the Principal primitive
(CLAUDE.md baked-in #15 + #18). Two design threads ran in parallel and have now been consolidated:

**Encryption arc** (the cipher/envelope/DAK substrate):
1. The 3-reviewer slate (cryptographer + P2P-architect + standards-skeptic) → **Combined Option F**
   (HPKE-RFC-9180 + MLKEM768-X25519 + multi-stanza + Inv-16 + X-Wing-mislabel corrective).
2. **e2r-ffull-scope-review** (`220b5aae`) — the **4-layer architecture A/B/C/D** + the **wave-sequencing**
   + the **Phase-4-Meta-Core / Phase-4-Meta-Composing split decision rule** (wire-format-affecting → Core;
   UX-coupled → Composing; both pre-tag). DAK substrate design, remote-permission-call protocol,
   multi-device key-wrap, identity-recovery flag.
3. Option-F+ pseudo-keypair pattern review → **NO-GO**; the unification belongs at the
   **envelope/codepoint-dispatch layer, not the primitive layer**.
4. **Option-F+ 9-eyes consolidated registry** (`fbdfeb16`) — the §6.2 codepoint-dispatched
   **`EncryptedEnvelope`** design; 28 unified amendments (U1–U28+); Compromises **#32–#44** + the #31
   extension; invariants **Inv-16/17/18**; the 5 Ben-call resolutions (Q1–Q5 + Q-extra). **This registry is
   the AUTHORITATIVE codepoint source** (§0.4).

**MembershipSet arc** (the unifying sharing primitive):
5. M-CONS-v1 → M-CONS-v2 → the critic round (M-C1/M-C2/M-C3) + the 6 P-specialists + the 3 cluster agents
   + the member/compute arc (CM-1/CM-2/PA-1/CE-1) →
6. **M-CONS-FINAL** (`a99dd0c7`) — the final MembershipSet design (its §10 is this R0's skeleton spine).
   2 primitives {MembershipSet, Drop}; 3 orthogonal axes; member-as-relation; derived nature;
   agents-as-plugins; `members_table` fusion; 5-value RoleId; federation; transport/D6;
   AuditAccessGradation; the 9 ratified Ben-calls (BC-1..9); Compromise housekeeping; Inv-19/20/21/22.
   **NOTE (R1 B-3):** M-CONS-FINAL's F8/F21 codepoint placement (`MembershipSetEncryption=0x6380`,
   `Sealed-Sender=0x6390`) is **SUPERSEDED by the 9-eyes** (those integers are MLS-Application/MLS-Welcome);
   see §0.4 + §4.0.

**Graph-native + engine-plugin refinements** (the tightening sweep over M-CONS-FINAL):
7. **GN-1** (`859fa51e`) — graph-native composition sweep. The audit log = content-addressed audit-event
   Nodes in a version-chain + IVM view + UCAN-gated (−4 codepoints); membership events = version-chain
   Nodes off the set anchor (−1 wire-enum); GovernanceConfig/economics/member-nature = top-level graph
   Nodes never inside the sealed Policy; **the frozen-crypto rule** that governs the whole freeze.
8. **GN-2** (`61f24553`) — the everything-is-graph crate-trajectory map; the **CE-1 SPLIT** framing (each
   crate = mechanism-half stays-Rust + data-half migrates-to-graph); `benten-membership-set` is the model
   SPLIT-endpoint crate.
9. **EP-1** (`7520ae4e`) — the Rust-engine-plugin formalization (the symmetric two-plugin model; the #19
   amendment; 3 openness tiers; no registry, trust unchanged).

**Supersession order (later/more-specific wins):** Ben's 3 R1-pass rulings (§2.0) > GN/EP refinements >
M-CONS-FINAL > 9-eyes registry > e2r-ffull > Combined Option F — **EXCEPT on codepoint integers, where the
9-eyes registry is authoritative over M-CONS-FINAL** (§0.4). Every divergence is noted in-line below.

### §0.2 Why this is consolidate-and-sequence, NOT greenfield discovery

Per `feedback_inverted_prework_post_campaign_phase`: F-full's scope was **pre-decided** by the design arc
above. This R0 **consolidates + sequences + names horizons + flags residual forks** — it does not
rediscover. The R1 critic council's job was reconciliation-audit + soundness-critique of the *plan as
written* (it did this; this R0.2 applies the result). Per `feedback_addl_pipeline_full_observance`: the
consolidation outputs are **R0-INPUT**, and the full ADDL pipeline (R1→R6) still runs on this substantive
plan-doc.

### §0.3 Ground-truth verification log (RE-RUN this pass, HEAD `2172cb6d`)

> **Why re-run.** The R1 triage flagged that the R0.1 verification log trusted the CLAUDE.md narrative and
> **inverted two facts** (#31 occupancy + the implied "LAMPS is in-tree"). Every "in-tree" claim below was
> independently `grep`/`sed`-verified against the tracked docs + code this pass. **Two corrections + three
> additions** vs R0.1 are flagged ⚠️.

| Claim | Evidence | Status |
|---|---|---|
| 14 crates in-tree; `benten-membership-set` does NOT exist | `ls -d crates/*/` → 14 (caps/core/crypto-suite/drop/dsl-compiler/engine/errors/eval/graph/id/ivm/platform-foundation/renderer-tauri/sync); membership-set absent | ✓ |
| `PrimitiveKind` = 12 irreducible (Read…Stream); no Agent/Plugin/Event | `crates/benten-core/src/subgraph.rs:71–93` (12 variants) | ✓ |
| `K(N)` structural-KDF chain live | `crates/benten-crypto-suite/src/structural_kdf.rs:8–9` (`K(root)=HKDF-SHA256(K_principal, "root"\|\|root_cid)`; `K(N)=HKDF-SHA256(K(pred), "step"\|\|edge_label\|\|N.cid)`) | ✓ |
| ⚠️ **`AeadEnvelope` is the SHIPPED type** (NOT `EncryptedEnvelope`); flat struct; untyped `&[u8]` AAD; **LE** codepoint | `crates/benten-crypto-suite/src/aead.rs:145` (`pub struct AeadEnvelope { format_version, cipher_codepoint, nonce, ciphertext }`); `:165` (`.to_le_bytes()`); `:244,277` (AAD index/count fields all `to_le_bytes`); grep `EncryptedEnvelope`/`BindingContext` = **ZERO** | ✓ (M-18 corrected) |
| ⚠️ **In-tree `0x647a` combiner is HKDF-SHA256, NOT real X-Wing** | `crates/benten-crypto-suite/src/cipher_suite.rs:37,45,78` — `combined = HKDF-SHA256(ss_x \|\| ss_mlkem \|\| ek_x \|\| ek_mlkem \|\| …, info="x-wing-v1-benten-0x647a")`; `sha3` IS already a dep (`:65 use sha3::Digest`). Real X-Wing = `SHA3-256(ss_M \|\| ss_X \|\| ct_X \|\| pk_X \|\| "\.//^\\")` per `draft-connolly-cfrg-xwing-kem-10` (label **APPENDED**; XWingLabel=`0x5c2e2f2f5e5c`; R4.2-corrected 2026-06-03). **The label says "X-Wing" but the construction is a Benten-private HKDF combiner** | ✓ (M-4 grounded; Ben ruling #3) |
| Cipher codepoints LIVE in-tree | `crates/benten-crypto-suite/src/codepoint.rs:199` `HYBRID_X25519_MLKEM768=0x647a` (LIVE) + `:205` `CLASSICAL_X25519=0x6400` (LIVE downgrade) + `:214` `HYBRID_MLKEM768_HQC=0x647b` (reserved) + `:228` `PURE_PQ_MLKEM768_ONLY=0x647c` (reserved-named, typed-reject `:268`) | ✓ (m-12: `0x6400` IS in-tree) |
| Sig codepoints in-tree | `codepoint.rs:49` `HYBRID_ED25519_MLDSA65=0x0001` (LIVE) + `:60` `CLASSICAL_ED25519=0x0002` + `:64` `HYBRID_MLDSA65_SLHDSA=0x0003` (reserved swap-matrix) | ✓ |
| Benten codepoint range = `0x6100..0x6FFF` IANA-disjoint | 9-eyes U8/U11 (`:559`); no in-tree assertion yet (a `CRYPTO-CODEPOINTS.md` doc-wave deliverable) | ✓ |
| `Scope` is EXACTLY-2-arm + 3rd-arm = HALT-AND-SURFACE | `crates/benten-caps/src/scope.rs:46` (`Hashes(Vec<Cid>)` \| `:51 RestrictedSelector(RestrictedScope)`); `:44` "Adding a third arm is a HALT-AND-SURFACE-TO-BEN escalation per HARD RULE 12" | ✓ |
| `system:Principal` + `create_principal` live | across `crates/benten-graph/tests/` + `benten-engine` (`engine.caps().create_principal`) | ✓ |
| Inv-14 derives "Plugin" (app-level subgraph) | `docs/INVARIANT-COVERAGE.md:320` | ✓ |
| ⚠️ **Inv-15 IS in-tree (REGISTERED, partially-enforced)** — NOT absent | `docs/INVARIANT-COVERAGE.md:1` header "**15 invariants**"; `:14` "Inv-15 REGISTERED … NOT-YET-FULLY-ENFORCED-BY-AUTOMATION"; `:49` the Inv-15 row. **The R1 M-15 premise ("not in INVARIANT-COVERAGE.md; header says 14 of 14") is STALE — Inv-15 IS registered.** In-tree invariants run **Inv-1…Inv-15**; **Inv-16…Inv-22 are design-mints** | ✓ (M-15 DISAGREE-WITH-EXPLANATION; §5.0) |
| ⚠️ **#31 occupancy INVERTED from R0.1**: in-tree **#31 = revocation-reach** (the summary-table row); the **LAMPS section is orphaned (NO table row)** | `docs/SECURITY-POSTURE.md:91` summary row `\| 31 \| Revocation reach in encryption-at-rest …` + `:30` callout "Compromise #31 — Drop bundle revocation reach" + `:2712` "Revocation reach (§R6) — Compromise #31 detail"; vs `:2433` "Compromise #31 — LAMPS Composite ML-DSA … EUF-CMA-only" prose with **no summary-table row** | ✓ (B-2 grounded; Ben ruling #2 = re-point) |
| ⚠️ **#30 is OCCUPIED (unaudited-PQ)** — do NOT move | `docs/SECURITY-POSTURE.md:91` summary row `\| 30 \| Unaudited PQ primitives in the v1-beta hybrid default …` (code-anchored to `0x647a` + `0x0001`; CLOSES at v1-GM / C-GM-AUDIT) | ✓ (M-5 grounded; #30 KEEP) |
| Highest Compromise # on main = **#31**; **#32+ unused** | `docs/SECURITY-POSTURE.md` (summary rows run 1…31; no #32+) | ✓ |
| `V1-FROZEN-INTERFACE-DEFERRED.md` max existing row = **D-27**; D-NN is the GLOBAL deferral namespace; this is the single destination doc | `grep 'Row D-NN'` → D-25/26/27 (`:852,1148,1215`); no D-28/D-29 | ✓ (M-16: D-28/D-29 are NEW mints) |
| `benten-sync` owns HLC / Loro / MST / iroh-transport / two-CID; **NO pub-sub/topic/broadcast** | `crates/benten-sync/src/`: `crdt.rs` (HLC-LWW; `:535 cmp_lex`) + `mst.rs`/`mst_proto.rs` (MST diff) + `transport_trait.rs:85` (`trait Transport` = `connect`/`accept_next`/`send_bytes`/`recv_bytes` ONLY) + `two_cid_store.rs` (DUAL-CID precedent) + Loro (`crdt.rs`/`Cargo.toml`) | ✓ (B-1 + M-10 + O-3 grounded) |
| ⚠️ **In-tree CRDT property rule = LARGER-HLC-wins (LWW)** | `crates/benten-sync/src/crdt.rs:535` — keeps the entry where `stamped.hlc.cmp_lex(&prev.hlc) == Greater` → larger HLC wins. **Inv-21's "smaller-HLC-wins for forks" is the OPPOSITE direction (deliberate)** | ✓ (M-7 grounded) |
| Nonce-cache replay-defense substrate is SHIPPED | `docs/SECURITY-POSTURE.md:1798` Compromise #25 CLOSED ("nonce-cache rejects replay of previously-seen sync envelopes"); `crates/benten-sync/src/handshake.rs` nonce-cache; replay tests | ✓ (M-1: substrate exists; Layer-D scope must be named) |
| ChangeEvent attribution triple is `Option` (bare `put_node` leaves unset) | `crates/benten-graph/src/store.rs:468` "Attribution fields (`actor_cid`,`handler_cid`,`capability_grant_cid`) are `Option` … a bare `put_node` … leaves them unset" | ✓ (m-15 GNC-2 grounded) |
| `docs/CRYPTO-CODEPOINTS.md` + `docs/future/compute-marketplace.md` do NOT exist | `ls` → absent (created as R0 doc-wave) | ✓ |

### §0.4 Codepoint source-of-truth resolution (B-3 / m-13)

The two source docs **conflict on three integers**. The resolution (later/more-specific + collision-free):

| Integer | M-CONS-FINAL (F8/F21) | 9-eyes registry (U13/U22) | RESOLUTION |
|---|---|---|---|
| `0x6380` | "MembershipSetEncryption family" | **MLS-Application bracket** (`0x6380..0x638F`) | **9-eyes wins** → MLS-Application; MembershipSetEncryption RELOCATES to `0x6600` (§4.0) |
| `0x6390` | "Sealed-Sender paired slot" | **MLS-Welcome bracket** (`0x6390..0x639F`) | **9-eyes wins** → MLS-Welcome; Sealed-Sender = `0x6510` (Ben ruling #1) |
| Sealed-Sender | `0x6390` | **`0x6510`** (sibling to `LAYER_C_DROP=0x6500`) | **`0x6510`** (Ben ruling #1; the ONE canonical value) |

The single authoritative allocation table is §4.0; `docs/CRYPTO-CODEPOINTS.md` (doc-wave) is its in-tree home.

---

## §2 Ratified-decisions inventory (FIXED — applied below, not re-litigated)

> **Ordering note:** §2 (the fixed inputs) precedes §1 (the architectural framing) deliberately — a cold
> reader should see the authority chain (esp. §2.0 Ben's 3 rulings) before the prose framing those decisions
> animate. §1 follows immediately below §2.

### §2.0 Ben's 3 R1-pass rulings (FIXED — highest authority; apply exactly)

| # | Ruling | Application |
|---|---|---|
| **BR-1** | **Sealed-Sender = DEFAULT** (not additive-slot). It is wire-affecting → ships at **Phase-4-Meta-Core**. Pull spam/abuse-mitigation + **Compromise #59** (re-scoped to abuse-control) into Core scope + timeline (~5–8 wave-days). Single canonical codepoint = **`0x6510`**. The improved **#43** metadata posture is recorded (Sealed-Sender DEFAULT means sender-DID is NOT in plaintext AAD on the default path). | §3.3 / §4.0 / §4.1 / §3.11 / §5.2 (#43, #59, #63) / §7.1 / §8 |
| **BR-2** | **Compromise #31 = LAMPS keeps it.** Re-derive against ACTUAL in-tree state: make LAMPS the **#31 summary-table row** + move revocation-reach to **#62** + update the **4 loci atomically** (the re-point touches **3 sites**: summary row `:91`, callout `:30`, detail `:2712`; the LAMPS prose `:2433` GAINS the summary row). **#30 stays occupied (unaudited-PQ — do NOT move).** | §5.0 / §5.2 |
| **BR-3** | **X-Wing corrective = real construction at `0x647A`.** Implement the actual X-Wing **SHA3-256** combiner per `draft-connolly-cfrg-xwing-kem` so `0x647A` is interop-faithful — NOT the HKDF-SHA256 stand-in. This is a **construction change** ⇒ new derived keys ⇒ regenerate ALL golden/KAT vectors. **Revise the LOC estimate UP** from "~24 LOC re-label". Ground-truth which combiner `aead.rs`/`cipher_suite.rs` computes as a **Wave-0 first step** (done: §0.3 — it's HKDF-SHA256). | §2.1 C-6 / §3.2 / §7.1 Wave-0 / §8 |

These reflect lead-architect direction post-R1. They override any conflicting framing in §2.1–§2.9 below.

### §2.1 Crypto / encryption (encryption-arc ratifications)

| # | Ratified decision | Source |
|---|---|---|
| C-1 | **Option-F+ NO-GO** → unification at the **ENVELOPE/codepoint layer** (§6.2 `EncryptedEnvelope`), NOT the primitive layer; pseudo-keypair-derived-from-symmetric-secret pattern rejected (root cause: feeding a password-derived seed into ML-KEM `KeyGenDerand` exposes the SampleNTT variable-time rejection-sampling side-channel, Arriaga "Tempo" IACR 2025/1399) | F+ review; 9-eyes |
| C-2 | **§6.2 codepoint-dispatched `EncryptedEnvelope`** with `EnvelopePayload` variant `SymmetricAead` (Layer-A vault) + `HpkeBase[MLKEM768-X25519]` (Layer-C drops + Layer-D wraps) + `HpkeMultiBase` (group multi-stanza) | 9-eyes §6.2 |
| C-3 | **Amendments 1–6** (U1–U6): codepoint committed in AAD/info (U1); strict-decode, no cross-variant fallback (U2); canonical-TLV length-injective (U3); sender-DID in AAD for non-vault (U4) — **NOTE BR-1: the DEFAULT path is Sealed-Sender, so the sender-DID-in-AAD U4 rule applies to the non-default plaintext-sender codepoint**; DeviceLink+RemotePermission bind sealed-at+valid-until epoch (U5); ML-KEM Decap CT-mitigation (U6) | 9-eyes Group A–C |
| C-4 | v1-beta **signature** default = **LAMPS Composite ML-DSA** `id-MLDSA65-Ed25519-SHA512` at `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`; EUF-CMA-only at construction, SUF-equivalent at app-layer via **Inv-15** (in-tree REGISTERED) | CLAUDE.md #5; Compromise #31 (LAMPS, per BR-2) |
| C-5 | v1-beta **encryption** default = **X25519⊕ML-KEM-768** (MLKEM768-X25519) at codepoint **`0x647A`** + **ChaCha20-Poly1305** bulk | CLAUDE.md #5 |
| C-6 | **X-Wing corrective (BR-3): REAL X-Wing SHA3-256 construction at `0x647A`** (NOT re-label). Current code (`cipher_suite.rs:37`) computes a Benten-private `HKDF-SHA256` combiner mislabeled "X-Wing" + uses an **LE** codepoint (`aead.rs:165`); codepoint `0x647A` is IETF-reserved for the X-Wing-identical MLKEM768-X25519. **Replace the combiner with `SHA3-256(ss_M \|\| ss_X \|\| ct_X \|\| pk_X \|\| XWingLabel)`** (label **APPENDED**; XWingLabel=`0x5c2e2f2f5e5c`; R4.2-corrected — NOT prepended) + LE→BE; **regenerate ALL golden/KAT vectors** (construction change ⇒ new keys). LOC ≈ **120–220** (combiner rewrite + BE sweep + full vector regen + interop KAT), NOT ~24. Pre-tag-must-fix; INDEPENDENT of F-full | cryptographer review; e2r §15.1; BR-3 |
| C-7 | Crypto-agility per baked-in **#5**: codepoint-dispatch + typed-reject (`UnsupportedAlgorithm`); never fork primitives; one integration crate is the only call site | CLAUDE.md #5 |
| C-8 | **Inv-17 hybrid-mandatory floor**: every KEM use site MUST be PQ-classical hybrid; **no pure-PQ codepoint LIVE/selectable** at v1-beta or v1-GM — reserved-named-typed-rejected arms (`0x647c`) are permitted for swap-matrix conformance only, audit-gated (m-3 sharpening) | 9-eyes Inv-17 |

### §2.2 Tactical picks

| Pick | Decision | Source |
|---|---|---|
| **Q1 ML-KEM impl** | **libcrux-ml-kem** (verified secret-independence via hax/F*; `check-secret-independence` CI gate; NOT oqs-rs FFI; NOT RustCrypto ml-kem best-effort). **NQ-C2 (R2):** the swap MUST preserve FIPS-203 encap/ciphertext/decap-key serialization byte-for-byte (cross-impl KAT) or golden vectors break | 9-eyes Q1 |
| **HPKE crate** | **Brendan McMillion `hpke`** (NOT Cryspen `hpke-rs` — 13 CVEs Feb 2026). **NQ-C1 (R2):** confirm it accepts a custom/PQ KEM (X25519MLKEM768 plugs into a real RFC-9180 context) OR Benten supplies the KEM + reuses only HPKE KDF/AEAD/key-schedule — determines whether "HPKE-RFC-9180" is byte-accurate or aspirational | e2r §3.3 |
| **OS keychain** | **`keyring-core` v1.0.0** (NOT legacy `keyring` which says "Do not depend on this crate!") | e2r §3.1 |
| **DAK KDF** | **Argon2id** v0x13 (RFC 9106; OWASP params `m_cost=19456, t_cost=2, p_cost=1`; user-tunable) | e2r §2.2 |

### §2.3 The 5 encryption-arc Ben-call resolutions (9-eyes §5)

| Q | Resolution | Source |
|---|---|---|
| **Q1** | libcrux-ml-kem (see §2.2) | 9-eyes |
| **Q2** | **BE codepoint endianness** — migrate `aead.rs` LE → BE pre-v1-beta-freeze (rides Wave-0; bumps `ENVELOPE_FORMAT_VERSION_V1 → V2` + regen golden vectors); network-byte-order canonical (RFC 9180 / FIPS 203 / MLS). **Re-scoped (M-19) — complete site-list:** ALL multiformats-framed integer wire/AAD/keying fields → BE (codepoint `aead.rs:165` + AAD `chunk_index`/`total_chunks`/`recipe_index`/`total_recipes` at `aead.rs:244,277` + `aead_wrap.rs` + platform-foundation + `structural_kdf.rs:157` + `varsig.rs:47,107` + `sizes.rs:183` + `swap_matrix.rs:1539,1540,1548,1550`) | 9-eyes Q2 |
| **Q3** | **DUAL-CID** — `envelope_blob_cid` public (transport; changes on reseal) + `plaintext_cid` graph-referenced (stable). MembershipSet: `plaintext_cid_local` LOCAL-ONLY + `plaintext_cid_set` HMAC-blinded + `envelope_blob_cid` public. Recipient-substitution defense in per-stanza AAD (U17), NOT the CID. **Extends in-tree `TwoCidStore`** (`benten-sync/src/two_cid_store.rs`, G-CORE-3e) — NOT net-new (O-3) | 9-eyes Q3; M-CONS-FINAL F18 |
| **Q4** | **HPKE-11-KE** (key-encryption mode) — HPKE wraps a CEK; CEK + per-Node `K(N)` bulk-encrypt | 9-eyes Q4 |
| **Q5 / Am4** | **Sealed-Sender DEFAULT (BR-1, RATIFIED).** Supersedes the 9-eyes consolidator's additive-slot lean. Sealed-Sender at `0x6510` is the v1-beta **default** Layer-C/drop codepoint; the plaintext-sender variant is a non-default sibling (U4 sender-DID-in-AAD applies to IT). Ships at Core; pulls abuse-mitigation (#59 re-scoped) into Core | **BR-1** |

### §2.4 The 4 layers + sub-phase split (e2r-ffull)

- **Layer-A** = real `K_principal` store (encrypted-at-rest under DAK; pull **G-CORE-3e** forward).
- **Layer-B** = per-Node AEAD with `K(N)` structural-KDF chain (Cryptree-aligned; X-Wing corrective).
- **Layer-C** = encrypt-to-recipient (HPKE + MLKEM768-X25519 + multi-stanza groups + Inv-16; **Sealed-Sender
  DEFAULT per BR-1**).
- **Layer-D** = DAK + device-auth + **remote-permission-call** (Signal-Provisioning + CTAP-2.2-inspired;
  **REQUIRED v1-beta**) + **multi-device-key-wrap** + **`ExecuteWorkflow` CODEPOINT-RESERVE (M-3)**.
- **Phase-4-Meta-Core** = wire-format-affecting (pre-freeze; incl. Sealed-Sender per BR-1).
  **Phase-4-Meta-Composing** = UX-coupled (biometric, Stronghold optional backend, device-link UX,
  remote-permission-call UX, identity-recovery `RecoveryHook`). **Both pre-`v1-beta`-tag.**

### §2.5 MembershipSet (M-CONS-FINAL; the 9 ratified Ben-calls BC-1..9 + Path-A.5)

| Decision | Ratified shape | Source |
|---|---|---|
| **2 primitives** | { MembershipSet, Drop }; `RestrictedScopeSet` is NOT a primitive (it is the K(N) chain + per-member edge-label allowlist inside a grant — a capability) | BC-1 / CA-1 |
| **3 orthogonal axes** | Scale (EXACTLY-3 `MembershipSetKind` = the ONLY keying axis) / Governance (Atrium→Garden→Grove presets, NOT crypto-Kinds) / Federation (`MemberRef::SubsetRef`) | BC-2 / CA-2 |
| **Member = `Principal`** | membership a RELATION; no Agent/Member/MemberKind type; **`member_type` deleted**; nature DERIVED (Inv-22); AI-agent = derived Plugin-flavor; trust categories → 2 | BC-3 / CM-1+PA-1 |
| **members_table fusion** | one `members_table: BTreeMap<Did, MemberEntry>` (fuses members+authorities+role_assignments); +disposition_class; +GovernanceConfig/metadata top-level | BC-4 / M-C1 |
| **RoleId SHIP ALL 5 active** | Admin/Moderator/Member/Viewer/Invitee — **BC-9 ratified ship-all-5**; define Moderator+Invitee **permission-sets at v1-beta** as UCAN templates (§3.6.B); governance *workflows* → Composing. **Two hard constraints (M-11/R1-Q-9): Invitee = derives ZERO content; Moderator ⊊ Admin** | **BC-9** |
| **key_retention_window_secs** | per-policy DEFAULT 7d (604800s), user-definable | BC-5 |
| **role_assignments_generation in AAD** | + `E_ROLE_STALE_AT_VERIFY` | BC-5 |
| **D6 iroh-gossip privacy** | HMAC-blinded topic + fork-rotation + OOB rendezvous; **iroh-gossip ONLY at v1-beta**. **Convergence rides existing MST anti-entropy; gossip = liveness/notification only (M-10)** | BC-7 / P2 D6 |
| **AuditAccessGradation** | config + suggested-defaults (Atrium/DeviceMesh=AdminOnly; SingleDevice=PublicAllMembers); **see §3.8 GN reframe** | BC-8 / P5 |
| **Path-A.5** | **Anchor+Version+CURRENT preserved**; immutable Version-Node-CIDs; key encryption to immutable Version-Node-CIDs (Inv-20 clause-f) | Path-A.5 |

### §2.6 GN graph-native wins (apply throughout)

- **Audit log** = content-addressed **audit-event Nodes in a version-chain + IVM view + UCAN-gated**
  (−4 codepoints). **Precision (m-15 GNC-2):** tamper-evidence is NOT "free" — the attribution triple is
  `Option` (`store.rs:468`); a bare `put_node` leaves it unset, so the audit-emitting WRITE path MUST be the
  enforced engine-API path (`benten-engine`), not a backend `put_node`.
- **Membership events** = **version-chain Nodes off the set anchor** (Anchor+Version+CURRENT; Inv-21 = the
  CRDT fork-tie-break over that DAG; −1 wire-enum). The keying-load-bearing `members_table` **snapshot**
  (CURRENT-materialization, AAD-bound) stays frozen.
- **GovernanceConfig + economics + member-nature** = **top-level graph Nodes, NEVER inside the sealed
  `MembershipSetPolicy`**. Garden/Grove governance sub-config = signed-config-Node content, NOT reserved
  sub-codepoints (−2 reserved sub-codepoints).
- **Precision (m-15 GNC-3):** an IVM view is a compiled `const` SSoT edit = Rust-engine-plugin LOC, not
  "free data" — call the GN wins "cheap additive", not "free".
- **The frozen-crypto rule** (§1.5) governs the whole freeze.
- **GN-4**: the audit-event **substrate** lands at v1-beta (cheap additive — wiring membership-admin WRITEs
  to emit audit-event Version Nodes + registering one IVM view); audit **query tooling** → Composing.

### §2.7 EP-1 (apply)

- Name the #19 category **"Rust engine plugin"** (qualifier always present; unqualified "plugin" = #18 graph
  plugin; "engine extension" = retained synonym).
- **Symmetric two-plugin model**: the engine ships a default roster of each kind. **Precision (m-15 GNC-4):**
  the open-backend seam roster is `KVBackend` / `BlobBackend` / `GraphBackend` / `Renderer` / `Transport` /
  `Materializer`; **IVM strategy is an ENUM (`benten_ivm::Strategy`), NOT a trait seam** (CLAUDE.md baked-in
  #2 — name it as enum-dispatch, not a backend trait).
- **3 openness tiers**: open backend seam / sealed policy seam (`CapabilityPolicy`, #830-locked
  `GrantReader`, `DeviceAuthBackend`) / enum-dispatch crypto.
- **No registry, trust unchanged** — recognition + naming + ~zero-code doc formalization; do NOT mint
  `EngineExtension` / `ExtensionRegistry`.

### §2.8 Compute (CE-1; first-class fungible per Ben)

- `PeerResource` graph Nodes (`ResourceKind { Compute | Storage | Bandwidth | Availability }`) +
  `OwnerRef { Member | Community | ThirdParty }`.
- Economics COMPOSES from {UCAN caveats + Credits-ledger-as-graph + signed `CommunityEconomicPolicy` Node};
  **ZERO MembershipSet wire field**.
- "local-free" is a **default `local_rate` policy-field**, NOT a hard engine rule.
- All Phase-5+ content → **`docs/future/compute-marketplace.md`** (created at R0 doc-wave; V1-FROZEN Rows
  **D-28/D-29** (NEW mints — §4.3)).

### §2.9 Canonical invariants / compromises

- **Inv-16/17/18** (encryption — 9-eyes) + **Inv-19** (Path-A.5 + MembershipSet keying-CRDT-input discipline)
  + **Inv-20** (MembershipSet primitive, **12 clauses**) + **Inv-21** (fork-tie-break HARD partition) +
  **Inv-22** (member-nature derived). **All design-mints** (Inv-15 is the highest in-tree; §5.0).
- **Compromise table** (BR-2): **#31 = LAMPS (KEPT as the in-tree summary-table row)**; revocation-reach →
  **#62**; **#30 = unaudited-PQ (KEPT; do-not-move)**; **#32–#44** = encryption arc; **#45–#61** =
  MembershipSet panel; **#62** = revocation-reach (re-pointed); **#63** = Sealed-Sender abuse-control
  (NEW per BR-1).

---

## §1 Architectural framing

### §1.1 What F-full is, in one paragraph

Benten is a Rust content-addressed graph DB (BLAKE3 + DAG-CBOR + CIDv1; `did:key` + UCAN; iroh + Loro
sync). Its **authority half** (capability-gating) is shipped and real, but only binds a *cooperating* engine
(CLAUDE.md baked-in #18). F-full builds the **confidentiality half**: a 4-layer encryption substrate
(encrypt-everything-at-rest + ephemeral-permission-per-operation + device-authentication +
remote-permission-call-from-another-device + multi-device-key-wrap) **plus** a unifying **MembershipSet**
primitive (the single, codepoint-dispatched `encrypt-to-N-recipients` mechanism that subsumes Atriums,
device-meshes, and single-device keying). Both land before the G-CORE-9 v1-public-interface freeze
(Phase-4-Meta-Core) and the device-link UX (Phase-4-Meta-Composing), both pre-`v1-beta`-tag.

### §1.2 The 4-layer encryption model (e2r-ffull A/B/C/D)

| Layer | Name | What it protects | Primitive |
|---|---|---|---|
| **A** | K_principal vault | The user's `K_principal` + user-DID signing key, at rest | `EncryptedEnvelope::SymmetricAead` (XChaCha20-Poly1305 — see m-4) under the **DAK** |
| **B** | per-Node AEAD | Every Node body, via `K(N) = HKDF-SHA256(K(predecessor), "step"\|\|edge_label\|\|N.cid)` (Cryptree-aligned structural-KDF chain; **real X-Wing corrective for the sharing-wrap**, per BR-3) | symmetric AEAD with structural-KDF-derived `K(N)` |
| **C** | encrypt-to-recipient | Sharing a Node / a Drop / a subgraph to N recipients | `EncryptedEnvelope::HpkeBase[MLKEM768-X25519]` + multi-stanza for groups; **Sealed-Sender DEFAULT (BR-1)**; Inv-16 |
| **D** | DAK + device-auth | Device-unlock, **remote-permission-call**, **multi-device-key-wrap**, **`ExecuteWorkflow` reserve** | DAK (Argon2id) + HPKE-encap (reuses Layer-C primitive); Signal-Provisioning + CTAP-2.2-inspired |

The four layers share ONE structural shape: *protect key material X under authorization context Y so
authorized party Z recovers X when Y holds* (e2r §14.1). The **§6.2 codepoint-dispatched `EncryptedEnvelope`**
is the unification — at the envelope/AAD-binding layer, NOT the primitive layer (Option-F+ NO-GO). One HPKE
primitive serves Layer-C drops + Layer-D wraps + remote-permission payloads; Layer-A uses a symmetric AEAD
variant of the same envelope; Layer-B per-Node AEAD stays its own symmetric primitive.

### §1.3 The MembershipSet primitive (M-CONS-FINAL)

**ONE identity primitive** (the existing `Principal`; baked-in #18). **TWO sharing primitives**
{ MembershipSet, Drop }. **THREE orthogonal axes** over ONE keying primitive:

- **A. Scale/identity** (the ONLY keying axis) — `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }`
  **EXACTLY-3** (frozen; §15.c HALT-AND-SURFACE).
- **B. Governance** — `Atrium → Garden → Grove` are **governance-policy PRESETS on the `Atrium` Kind, NOT
  crypto-Kinds** (`GovernanceConfig` Node + `RoleId` + N2 RBAC/UCAN). Garden/Grove DROP OUT of the Kind
  codepoint-reserve.
- **C. Federation** — polycentric cross-set linking via `MemberRef::SubsetRef` recursion (reserve-only at
  v1-beta; Inv-20 clause-k/l).

A **member is a `Principal` admitted to a MembershipSet — a RELATION, not a type.** Mint NO `Agent` type, NO
`Member` type, NO `MemberKind` enum; **`member_type` is DELETED**; member-nature is DERIVED (Inv-22). An
**AI-agent is a derived Plugin-flavor** (`is_autonomous_ai(Plugin)`); trust categories collapse to 2.

### §1.4 The everything-is-graph principle + the engine-plugin symmetry

Per GN-1/GN-2/EP-1, F-full is designed natively at the **everything-is-graph endpoint** (the CE-1 SPLIT
applied throughout): `benten-membership-set` is a **thin keying-glue Rust engine plugin** (mechanism-half),
and its entire governance/audit/members/economics/federation surface is **graph Nodes the engine walks**
(data-half), with ZERO new frozen wire field beyond the keying minimum.

The **two-plugin symmetry** (EP-1): everything that extends the engine is a *plugin*, of two kinds — **graph
plugin (#18)** (shareable subgraph + three-layer consent) and **Rust engine plugin (#19)** (compile-time
backend-trait impl; trust = "you compiled this in"). The engine ships a default roster of each.
`benten-membership-set`'s keying-glue is a Rust engine plugin; its data half is graph.

### §1.5 The frozen-crypto rule (governs the whole freeze)

> **If data is bound into the AEAD AAD, gates `K_Set` derivation/decryption, or must verify offline
> cross-engine → it is frozen wire-format. Everything else composes as graph** (content-addressed Nodes +
> the 12 primitives + UCAN capabilities + IVM materialized views + version-chains).

**The 4th-limb question (M-17).** `audit_log_query` is a public op-SIGNATURE over graph-native data. Per the
rule above, a READ op over graph data is NOT AAD-bound / K_Set-gating / crypto-verify — so it does NOT freeze
beyond the 12-primitive surface. **Resolution (M-17, consistent with §3.8's "thin scoped-READ" framing):**
`audit_log_query` is **GRAPH-NATIVE** (a UCAN-gated scoped-READ composing the existing 12 primitives + the
`audit:<set_id>:*` `RestrictedScope`), NOT a frozen op. **No new frozen op signature is added beyond the
12-primitive surface.** The "24 ops" figure from R0.1 was **unsourced** (not in M-CONS-FINAL) — it is
**RETRACTED**; the frozen op-surface is exactly the 12 primitives (NQ-W3 carries the explicit enumeration to
R2 for the conformance test).

This single line keeps the frozen surface at the **minimum** (the keying envelope + the EXACTLY-3 Kind enum +
the AAD tuple + the keying-bound `members_table` snapshot + the RoleId ordinal + Inv-16..22) and the
**maximum** surface as graph — so Phases 5–8 extend additively, never via wire-break.

### §1.6 The v1-beta-freeze posture (phase-ordering precision)

Per CLAUDE.md baked-in #15 (`feedback_phase_ordering_precision`): `v1-beta` is tagged **AFTER both
`phase-4-meta-core-close` AND `phase-4-meta-close`**. **Phase-4-Meta-Composing is NOT post-v1-beta-defer** —
it is pre-`v1-beta`-tag. Decision rule: **wire-format-affecting → Phase-4-Meta-Core** (which TERMINATES by
freezing the v1 public interface); **UX-coupled → Phase-4-Meta-Composing**. Default per
`feedback_orchestrator_defer_prediction_bias` = do-it-now; defer only for a specific dependency reason.
`v1-GM` is gated on the independent ml-dsa / ml-kem audit (NF-2 / C-GM-AUDIT). **The assessment-window sits
BETWEEN `phase-4-meta-close` and `v1-beta`** (NQ-A2; §9.2 corrected).

---

*(continued — §3 design, §4 wire inventory, §5 invariants/compromises, §6 crate plan, §7 waves, §8 cost,
§9 exit, §10 R2/R3 questions, §11 self-assessment, §12 citations, §13 R1 changelog)*

## §3 Design per layer + the MembershipSet primitive

### §3.1 Layer-A — `K_principal` vault (encrypt-everything-at-rest foundation)

**What it is.** A pre-unlock, on-disk DAG-CBOR file at `${BENTEN_DATA_DIR}/vault.cbor` holding the user's
`K_principal` (32 bytes; derives every per-Node `K(N)`) + the user-DID signing key, AEAD-encrypted under the
**DAK** (Device Authentication Key). Pulls **G-CORE-3e** forward (current `K_principal` is a STUB).

**Construction (e2r §2.2, §8.2):**
- KDF: **Argon2id v0x13** (RFC 9106; OWASP params; user-tunable; params persisted alongside salt).
- 16-byte salt from `OsRng`, stored unencrypted alongside the vault.
- Intermediate **HKDF-SHA256** over the Argon2id seed + codepoint label `"benten-dak-v1"` → the DAK (the
  HKDF info-tag is the codepoint slot for a future Argon2id-v2-param-set).
- **Vault AEAD nonce-width (m-4): XChaCha20-Poly1305 (24-byte nonce).** The vault is a re-encryption-heavy
  site (K_principal rotation re-seals; multi-device key-wrap re-seals) — a 12-byte ChaCha20 nonce hits the
  2^32 random-nonce birthday bound under heavy reseal. U32 prescribes XChaCha20 (24-byte) for these sites.
  **Both `SymmetricAead [u8;12]` and `SymmetricAeadXNonce [u8;24]` ship at v1-beta** (§4.1); the vault uses
  the XNonce variant. Rationale recorded in `SECURITY-PROOFS.md` (doc-wave).
- Vault payload (CBOR): `{ k_principal: [u8;32], user_did_signing_key: HybridSigningKeySerialized,
  user_did_creation_time: u64 }`; both keys live in the SAME vault (atomic lock/unlock; identity coherence).
- Memory hygiene: `secrecy::SecretBox<[u8;32]>` + `zeroize::Zeroize` on Drop (`secrecy` is a NEW Layer-A dep
  — O-1; §6.2 + Compromise #39 supply-chain row).
- Constant-time: wrong-password path runs Argon2id + AEAD-decrypt to completion (no fast-fail short-circuit).

**Design-rationale note (O-2).** Layer-A protects on the **adversary-axis** (offline-device-theft adversary
lacking the password) AND defines the **capability-axis** root (the unlocked `K_principal` is the keying root
for every per-Node `K(N)` Layer-B derives).

**Envelope shape (Inv-16):** `EncryptedEnvelope::SymmetricAeadXNonce { codepoint, aad_binding:
BindingContext::Vault { vault_version }, ... }`. The codepoint enters the `EncryptionCodepoint` enum
(wire-format-affecting — goes through G-CORE-9 freeze).

**Engine startup** (e2r §2.4): read vault → `DeviceAuthBackend::unlock(prompt)` → Argon2id → HKDF →
AEAD-decrypt → hydrate `UnlockedKeyMaterial { k_principal, user_did_signing_key }` → all per-Node AEAD +
UCAN-signing gated through the unlocked handle. Pre-unlock ops return `EngineLocked` typed-reject.

### §3.2 Layer-B — per-Node AEAD (Cryptree-aligned structural-KDF chain)

**What it is.** Every Node body is encrypted with `K(N)` derived via the live structural-KDF chain
(`structural_kdf.rs:9`): `K(root) = HKDF-SHA256(K_principal, "root"||root_cid)`;
`K(N) = HKDF-SHA256(K(predecessor), "step"||edge_label||N.cid)`. Cryptree-aligned (a member with
`K(predecessor)` derives all descendant keys). It composes with Layer-C: sharing a Node = encrypt the small
`K(N)` to the recipient via HPKE (KEM-DEM / key-encryption mode, Q4), recipient derives `K(N)` then
AEAD-Opens the body.

**X-Wing corrective (C-6 / BR-3 — REAL construction; lands FIRST in Wave-0, independent of F-full).**
Ground-truth (§0.3): in-tree `cipher_suite.rs:37` computes `combined = HKDF-SHA256(ss_x || ss_mlkem || ek_x
|| ek_mlkem || …, info="x-wing-v1-benten-0x647a")` — a **Benten-private HKDF combiner mislabeled "X-Wing"** —
and `aead.rs:165` writes the codepoint **LE**. **Two corrections:**
- **(a) Real X-Wing combiner.** Replace the HKDF-SHA256 combiner with the actual X-Wing construction per
  `draft-connolly-cfrg-xwing-kem-10`: `SHA3-256(ss_M || ss_X || ct_X || pk_X || XWingLabel)` — the label is **APPENDED** (XWingLabel = the 6 bytes `0x5c2e2f2f5e5c`, ASCII `\.//^\`), NOT prepended; the prepended form is the superseded v01-v02 construction and would freeze a non-interoperable KEM at the IETF-reserved `0x647A`. (R4.2-corrected 2026-06-03, verified against draft-10 §6.) (`ss_M` =
  ML-KEM-768 shared secret; `ss_X` = X25519 shared secret; `ct_X` = X25519 ciphertext/ephemeral-pubkey;
  `pk_X` = X25519 recipient pubkey; the exact label bytes + input ordering per the draft). `sha3` is ALREADY
  a dep (`cipher_suite.rs:65`). **This is a CONSTRUCTION change** — the derived key differs from the current
  HKDF output ⇒ **every golden/KAT vector regenerates** ⇒ also re-derive the classical-downgrade (`0x6400`)
  `classical_combine` consistently.
- **(b) BE codepoint (Q2 / M-19).** `aead.rs:165` + the 3 distinct `0x647a` encode-sites (the wire codepoint
  `:165`; the HKDF/combiner info-tag string `"x-wing-v1-benten-0x647a"` at `cipher_suite.rs:78`; the
  doc-comment `aead.rs:23`) — **the info-tag string is ASCII and is NOT endianness-affected** (m-1: state
  this explicitly so the LE→BE migration doesn't silently re-derive the combiner domain); only the on-wire
  `u16` flips LE→BE. Bump `ENVELOPE_FORMAT_VERSION_V1 → V2`.

**LOC (BR-3): ~120–220** (combiner rewrite ~40 + BE sweep across `aead.rs`/`aead_wrap.rs`/platform-foundation
~40 + full golden-vector regen + a `draft-connolly-cfrg-xwing-kem` interop KAT cross-check ~40–140), NOT
"~24 LOC re-label". Pre-tag-must-fix regardless of whether F-full lands (e2r §15.1).

**Wave-0 exit gates (m-2):** a FIPS-203 byte-compatibility KAT (RustCrypto `ml-kem 0.2` ↔ libcrux) +
`check-secret-independence` in CI + a conformance test asserting **NO `to_le_bytes` survives on any wire/AAD
path** (M-19).

**Residual after Layer-A (~50 LOC, e2r §5.1):** once `K_principal` is a real type, the existing combiner
plugs into it. No new wire format.

**Threat-model coherence note (R1-Q-1 CONFIRMED — KEEP BOTH).** Layer-B per-Node AEAD and Layer-A DAK-vault
have OVERLAPPING at-rest-theft coverage but UNIQUE roles. **Layer-A** defends the root secret vs the
offline-at-rest adversary lacking the password. **Layer-B exists PRIMARILY to enable Cryptree selective
sharing** (a recipient gets `K(N)` for a subtree WITHOUT `K_principal`) — dropping Layer-B destroys selective
sharing. Complementary, not duplicative.

### §3.3 Layer-C — encrypt-to-recipient (HPKE + MLKEM768-X25519 + multi-stanza; Sealed-Sender DEFAULT)

**What it is.** The `encrypt-to-N-recipients` mechanism (the heart of the MembershipSet primitive).
**Single-recipient:** `EncryptedEnvelope::HpkeBase[MLKEM768-X25519]` (HPKE-RFC-9180 mode_base; codepoint
`0x647A` KEM; ChaCha20-Poly1305 AEAD; key-encryption mode per Q4). **Multi-recipient/groups:**
`HpkeMultiBase { cek_aead_ciphertext, cek_aead_nonce, stanzas: Vec<HpkeRecipientStanza> }` (U17). **Group
sends honor Sealed-Sender (BR-1 / F-LC-9 — RATIFIED).** EVERY group send is Sealed-Sender by default: the
inner-sender-DID is bound **INSIDE** the sealed/encrypted part **per stanza** (HPKE inner-payload sender-DID +
post-decrypt-verify), NOT in the plaintext on-wire AAD — consistent with the single-recipient drop
(`0x6510`) and the MembershipSet group (`0x6610`). **`sender_did` is NOT a plaintext AAD field on the default
path.** The **plaintext-sender group variant** (sender-DID on-wire-authenticated in AAD) is an explicitly
**non-default** sibling shape (the same non-default disposition as the `0x6500` plaintext-sender
single-recipient drop; U4 sender-DID-in-AAD applies to IT, and Inv-18's paired-disclosure clause is satisfied
because the DEFAULT group send is the metadata-hiding shape).

**FROZEN AAD field-sets (RATIFIED — these are the v1-beta wire; G-CORE-9-frozen; everything in the AAD is
authenticated-not-encrypted, i.e. plaintext on the wire).**

- **`0x6510` single-recipient (`DROP_TO_RECIPIENT_SEALED_SENDER`) AAD = the minimal-sufficient union (Option
  A):** `{ aad_version (0x01, u8), codepoint (0x6510, u16 BE), audience (recipient DID, u32-BE
  length-prefixed), body_cid (self-describing CIDv1 — see §3.3 body_cid framing), recipient_key_generation
  (u32 BE) }`. NO stanza-index, NO recipient-DID-LIST, NO sender_did (sealed inside the ciphertext).
- **`0x6610` MembershipSet group per-stanza AAD (BLINDED; RATIFIED):** `{ aad_version (0x01, u8), codepoint
  (0x6610, u16 BE), body_cid (self-describing CIDv1), member_count (u32 BE), audience_set_commitment (32B),
  stanza_index (u32 BE), stanza_count (u32 BE), member_key_generation (u32 BE),
  membership_set_id_commitment (32B), membership_set_generation (u32 BE), role_assignments_generation
  (u32 BE) }`. The **sealed-inner-sender-DID stays INSIDE the ciphertext** (NOT a plaintext AAD field; per
  F-LC-9). TWO fields are BLINDED vs the prior raw shape:
  - **`audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)`** over the CANONICAL SORTED
    recipient-DID list (replaces the prior raw `sorted recipient-DID-list`).
  - **`membership_set_id_commitment = HMAC(K_Set, "benten:setid:v1" || membership_set_id)` truncated to 32
    bytes** — the SAME construction §3.9 already uses for the gossip topic (replaces the prior raw
    `membership_set_id`). **Frozen primitive (R0.7 precision):** `HMAC` here denotes
    **`blake3::keyed_hash(K_Set, "benten:setid:v1" || membership_set_id)`** — BLAKE3's native keyed MAC, NOT
    HMAC-SHA256 (`benten-membership-set` carries no hmac/sha2 dep; BLAKE3-keyed IS a MAC; truncate-to-32 is the
    native BLAKE3 output width). The abstract name is kept; the bytes are unchanged. R5 routes through the real
    `benten-crypto-suite` keyed MAC over `K_Set`.
  - **`stanza_count` is bound alongside `stanza_index`** as a truncation/censorship defense: without it an
    active relay can silently drop trailing stanzas to censor a co-recipient and each surviving stanza still
    verifies.

**Why the group AAD is BLINDED (rationale — freeze record).** The prior `0x6610` AAD published the raw
membership roster + raw set-id in plaintext, which CONTRADICTS the project's own already-ratified §3.9 /
Compromise #61 blinding posture (set-identifying material is never published in the clear). Blinding makes the
group AAD obey that rule. Recipients hold `K_Set` + the member list, so they recompute + verify both
commitments — ALL binding properties (cross-stanza substitution U17; inter-member non-forgeability) are
PRESERVED; the relay sees only opaque 32-byte tags.

- **`0x6520` `LAYER_C_DROP_MULTI_RECIPIENT` group per-stanza AAD (BLINDED; RATIFIED R0.7).** `0x6520` is the
  Layer-C **multi-recipient** group send (`EnvelopePayload::HpkeMultiBase`) — DISTINCT from the `0x6610`
  MembershipSet group; it is NOT a MembershipSet. It carries the SAME recipient-roster social-graph leak
  (#61-class) as `0x6610`, with the same un-retrofittable-past-freeze deadline, so it is BLINDED the same way.
  Frozen on-wire AAD (BE; everything authenticated-not-encrypted): `{ aad_version (0x01, u8), codepoint
  (0x6520, u16 BE), body_cid (self-describing CIDv1, 36B), recipient_count (u16 BE), audience_set_commitment (32B), stanza_index
  (u32 BE), stanza_count (u32 BE), recipient_key_generation (u32 BE) }`. The
  **sealed-inner-sender-DID stays INSIDE the ciphertext** per stanza (post-decrypt-verified; F-LC-9 / BR-1) —
  NOT a plaintext AAD field. Three fields change from the prior raw `0x6520` shape:
  - **`audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)`** over the CANONICAL SORTED
    recipient-DID list (lp = u32-BE length prefix) — the IDENTICAL construction to `0x6610` — REPLACES the
    prior raw `sorted recipient-DID-list (len u16 BE || bytes per DID)`.
  - **`stanza_count` is bound alongside `stanza_index`** (relay-truncation/censorship defense — an active
    relay cannot silently drop trailing stanzas; each survivor fails the bound count).
  - **`body_cid` is a self-describing CIDv1** (`0x01 0x71 0x1e 0x20 || 32-byte BLAKE3 digest` = 36 bytes),
    NOT a bare fixed-32 (CLAUDE.md baked-in #5; restores U3 length-injectivity) — consistent with
    `0x6510`/`0x6610`.
  - **UNLIKE `0x6610`:** `0x6520` is NOT a MembershipSet, so it carries **NO `membership_set_id_commitment`,
    NO `membership_set_generation`, NO `role_assignments_generation`** (those are MembershipSet-only fields).
  - **Field widths follow the Layer-C drop band, NOT the MembershipSet band** (the §4.0
    width-unification-REJECTED note governs): `recipient_count` is **u16 BE** (the band's existing
    convention), the same per-band width the prior raw `0x6520` shape used for its recipient cardinality.
  - **Rationale (freeze record):** same #61 / §3.9 anti-fingerprint posture as `0x6610` — recipients hold the
    member list and recompute + verify `audience_set_commitment`; the relay sees only an opaque 32-byte tag;
    all bindings (cross-stanza substitution U17; inter-member non-forgeability) PRESERVED. **Honest scope:**
    identity-HIDING not unlinkability (the commitment recurs for a static recipient set); full per-send
    unlinkability = **U25, CODEPOINT-RESERVE for v1-GM**, additive with no wire break.


**HONEST SCOPE of the blinding (freeze record).** This achieves identity-HIDING, NOT unlinkability — the same
commitment recurs for a static group, so a network observer can still link sends to "the same unknown group."
Full per-send unlinkability (salt/nonce-rotated commitments) is **U25, CODEPOINT-RESERVE for v1-GM**, additive
over this field with no wire break. The change degrades the leak from "plaintext roster" (identity-revealing)
to "linkable opaque tag" (correlation-only).

**`body_cid` is a self-describing CIDv1, NOT a bare fixed-32-byte digest (BR — applies to BOTH `0x6510` and
`0x6610`).** `body_cid` is multihash-length-prefixed: `0x01 0x71 0x1e 0x20 || 32-byte BLAKE3 digest`. A bare
fixed-32-byte digest would bake a hash-width assumption into a frozen wire, contradicting CLAUDE.md baked-in #5
("never hardcode key/sig/ciphertext sizes"; the multiformats framing is the permanent commitment; pre-blessed
agile fallbacks = SHA-512/256 + SHA3-256). Self-describing CIDv1 is multihash-length-prefixed by construction
⇒ restores U3 length-injectivity for free.

**Sealed-Sender DEFAULT (BR-1 — RATIFIED at Core).** The v1-beta default Layer-C/drop codepoint is
**`DROP_TO_RECIPIENT_SEALED_SENDER = 0x6510`** (sibling to `LAYER_C_DROP = 0x6500`). On the default path the
sender-DID is bound **INSIDE** the ciphertext (via HPKE inner-payload sender-DID + post-decrypt-verify; AAD
carries only audience (NO coarse epoch — the 1-hr bucket is Layer-D-only per RULING-1 / §3.10 / M-14)) — so
the sender-DID is NOT on the wire in plaintext. The
**plaintext-sender variant `LAYER_C_DROP = 0x6500`** is the non-default sibling; U4 (sender-DID-in-AAD)
applies to IT, and Inv-18's paired-disclosure clause is satisfied because the DEFAULT is the metadata-hiding
shape. **Abuse/spam mitigation (re-scoped #59 → §3.11):** with no plaintext sender identity, abuse-control
needs its own mechanism — named as v1-beta-Core work (~5–8 wave-days), not capability-discipline-by-default.

**DUAL-CID (Q3, U18; extends in-tree `TwoCidStore`):** `envelope_blob_cid = BLAKE3(serialized
EncryptedEnvelope)` (transport; changes on reseal; iroh-blobs handle) vs `plaintext_cid = BLAKE3(canonical
DropBundlePayload)` (stable; graph-referenced). MembershipSet adds `plaintext_cid_local` (LOCAL-ONLY,
NEVER-serialized — O-7 R5 kani-pin) + `plaintext_cid_set` (HMAC-blinded). Recipient-substitution defense is
in per-stanza AAD (U17), NOT the CID. **This EXTENDS `benten-sync/src/two_cid_store.rs`** (O-3) — not net-new.

**Inv-16 mint** + key-retention: `recipient_key_generation` bound into `BindingContext` + per-stanza (U19);
recipient-side key-retention ≥1-year grace (vault-encrypted); revocation-reach (now **#62**) clarified
"forever-valid within retention window".

**FS-gap honest disclosure (Compromise #42 / #56).** HPKE-mode-base is structurally non-FS at the
long-term-sk axis (2030-sk-compromise recovers 2026 envelopes). Partial FS via application-layer key
rotation; full FS via CGKA/MLS-PQ (post-v1-beta, codepoint-bracket-reserved per U13 — `0x6380..0x63CF`).
Journalist per-message FS is a SEPARATE design class (Compromise #56 — kept sharply distinct from
#42/#52 so the audit does not read it as a duplicate; R1-Q-8 CONFIRMED mint-all).

**IND-CCA2-under-adversarial-recipient-seed (M-6 / 9-eyes flagged gap).** No formal-methods lens has
confirmed HPKE-mode-base[MLKEM768-X25519] IND-CCA2 tractability under the **adversarially-chosen-recipient-
pubkey** model — and the device-link / remote-permission flows DO admit a chosen-recipient-pubkey surface
(malicious device B supplies an adversarial pubkey). This connects to #45 (MAL-BIND-K-CT/K-PK) + #59
(KEM-key-confirmation). Carried as a **§9.3 audit-deliverable line** + a **§10.2 risk row**; it is an
R0-INPUT (do not drop).

### §3.4 Layer-D — DAK + device-auth + remote-permission-call + multi-device-key-wrap + ExecuteWorkflow

**DAK substrate (e2r §2).** `DeviceAuthBackend` trait (sealed per #7) with `unlock(prompt) ->
SecretBox<[u8;32]>` + `lock()` + `supports_biometric()` + `supports_remote_unlock()`. Default impl = the
Benten-vended Argon2id + XChaCha20-Poly1305 + (optional) `keyring-core` stack — works in all three deployment
shapes (full peer / thin compute / embedded webview, baked-in #17), incl. headless full-peers
(password-via-env-var `BENTEN_VAULT_PASSWORD` or password-via-IPC; no Tauri / no keyring dependency).

**Remote-permission-call (e2r §6; REQUIRED v1-beta; Signal-Provisioning + CTAP-2.2-inspired).** TWO flows
under one wire protocol: **R1 Remote-unlock** (device A unlocks device B without B's local password) + **R2
Remote permission-grant** (device A issues an ephemeral UCAN delegation or HPKE-wrapped key material to B).
Wire shape (e2r §6.4):
`PermissionRequest { request_id, requesting_device_did, requesting_device_pubkey, operation:
{ Decrypt(node_cid) | SignUcanDelegation(scope, audience, expires_at) | RemoteUnlock | ExecuteWorkflow(…) },
reason, timestamp, ephemeral_signing_key, nonce }` (signed by B) → A validates + approval-UX + signs grant →
`PermissionGrant { request_id, granted_at, valid_until, operation_result, audit_node_cid }` (signed by A's
user-DID-signing-key).

**`ExecuteWorkflow` CODEPOINT-RESERVE (M-3 — Ben's headline rented-compute use case).** Add the
`PermissionOperation::ExecuteWorkflow { workflow_cid: Cid, input_node_cids: Vec<Cid>, max_decrypt_count: u32,
result_recipient_pubkey: HybridKemPubKey, executor_did: Did }` variant slot now (U21). The variant-slot +
the AAD-binding of `(executor_did, max_decrypt_count, result_recipient_pubkey)` are FROZEN at v1-beta;
**runtime enforcement (no-egress / bounded-decrypt) is post-v1-beta** (NQ-T3 confirms the frozen AAD scope is
SUFFICIENT to express the constraint even though enforcement defers). **Executor-trust threat entry** added
to §10.2: a rented executor running an arbitrary handler subgraph on untrusted compute is a SANDBOX-escape
class re-asked at the rented-compute boundary.

**Replay defense (M-1 — load-bearing).** The remote-permission/device-link replay window rides on an **outer
UCAN-token/`jti`-keyed nonce-cache that is net-new at v1-beta** (it re-uses the Compromise #25
durable-CAS-marker *pattern*, NOT the #25 sync-frame instance — see §3.10), NOT on generation-counters
(counters defend key-staleness/substitution; the nonce-cache defends replay) and NOT on the coarse 1-hour
epoch bucket. The **nonce-cache is promoted to a NAMED load-bearing v1-beta REQUIREMENT** (§3.10).

**Pre-merge security mini-review REQUIRED (e2r §6.7; M-12 — now SCOPED).** Owner = the threat-model lens
(Pattern 6). **Six pass-classes** (e2r's 5 + audit-Node-binding): (1) replay; (2) device-key-revocation
interaction; (3) clock-skew; (4) confused-deputy; (5) UI-deception; (6) **audit-Node-binding** — the grant
MUST be REJECTED if `audit_node_cid` is absent/unresolvable (so a "grant without audit trail" is
non-constructible). §9.1 item-4 reads "mini-review passed, clean on all six." NQ-T1 (R2): is the audit-Node
encrypted + replicated to all the user's devices so a malicious device can't grant-and-hide?

**Multi-device key-wrap-on-device-link (e2r §7; Signal-Provisioning precedent).** Device B generates a fresh
device-keypair + device-encryption-pubkey (X25519+ML-KEM-768) + a fresh local DAK; displays
`ProvisioningOffer` QR. Device A scans, confirms device-fingerprint, HPKE-encrypts `ProvisioningInnerPayload
{ k_principal, user_did_signing_key, user_did_pubkey, atrium_memberships, provisioning_session_id, granted_at }`
to B's pubkey, signs with user-DID-signing-key, writes a `DeviceAttestation` Node, transmits over iroh.
Device B HPKE-decrypts, verifies, stores under its own DAK, mints its own DeviceAttestation. Wire structs
codepoint-frozen at Core. Properties: HPKE IND-CCA2 confidentiality; user-DID-signature authentication; no FS
for `K_principal` (identity-equivalent by design); session-layer FS via `provisioning_session_id`; replay
defense via session-id binding. Revocation (e2r §7.4): RotationLog entry (`benten-id::RotationLog`); cuts
future grants, NOT retroactive un-decryption.

**User-DID private-key at-rest (e2r §8).** The user-DID signing key lives in the same DAK-vault as
`K_principal` (NOT split at v1-beta default; split-storage is a future-additive `DeviceAuthBackend` impl).
Vault file is small + DAK-encrypted (safe on untrusted cloud, Bitwarden-equivalent threat model); NOT
multi-device-synced (different DAKs); device-link key-wrap (§7) is the propagation path.

**Tight-`exp` default mandate (m-6; Compromise #60 + #52).** `SignUcanDelegation` MUST mint with a SHORT
default `expires_at` (the v1-beta `DeviceAuthBackend` / remote-permission default — not operator-supplied),
because under ship-all-5 + remote-permission an ephemeral UCAN survives an RBAC role-downgrade (Compromise #60)
and a removed/downgraded member is bounded ONLY by `exp` (Compromise #52 fork-on-kick has no PCS against the
already-issued attenuation). The short default caps that survival window; long-lived grants are an explicit
opt-out, never the default.

**Blast-radius ladder (O-6).** `Decrypt`=1 Node < `SignUcanDelegation`=attenuated-exp-bounded <
`ExecuteWorkflow`=bounded-decrypt-count < `RemoteUnlock`/device-link=full `K_principal`-permanent. The
trust-tier × operation matrix → `docs/THREAT-MODEL.md` skeleton (doc-wave).

### §3.5 The MembershipSet primitive (the central novel structural addition)

**`MembershipSet` (keying primitive).** EXACTLY-3 `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` +
keying-reserves (`AtriumWithRotatingGroupKey`, `EphemeralLobby`). Pre-R0 orienting shape (NOT impl):

```rust
// Pre-R0 orienting shape; canonical CBOR wire shape; the keying-frozen minimum.
struct MembershipSet {
    kind: MembershipSetKind,                       // EXACTLY-3; the ONLY keying axis
    members_table: BTreeMap<Did, MemberEntry>,     // fused; keying-AAD-bound snapshot (canonical-CBOR; NQ-W4)
    metadata: MembershipSetMetadata,               // top-level; admin-rename
    governance: GovernanceConfigRef,               // top-level Node REFERENCE (graph), NOT inside Policy
    policy: MembershipSetPolicy,                    // sealed; key_retention_window_secs (default 7d) etc.
}

struct MemberEntry {                               // one per admitted Principal DID; ZERO nature field
    role: RoleId,                                  // governance axis (default Member); ALL 5 active (BC-9)
    is_authority: bool,
    sig_pubkey: Option<SigPubKey>,                 // present iff is_authority
    admitted_at_hlc: Hlc,                          // when admitted (see §3.5.HLC vs Inv-21 created_at_hlc)
    member_ref: MemberRef,                         // KEYING/FEDERATION axis (NOT nature)
}

enum MemberRef {                                   // homogeneous-per-Kind keying variant + federation reserve
    UserDid,                                       // Atrium
    DeviceDid(DeviceAttestation),                  // DeviceMesh
    LocalDevice,                                   // SingleDevice
    SubsetRef(MembershipSetId, KSetAcquisitionPath),  // FEDERATION reserve (refused at v1-beta; Inv-20 k)
}

enum RoleId { Admin = 4, Moderator = 3, Member = 2, Viewer = 1, Invitee = 0 }  // ALL 5 active (BC-9)
```

**`members_table` fusion** replaces the cross-table invariant by construction (one DID → one record). **Net
new frozen member field = ZERO.** **NQ-W4 (R2):** pin the exact length-injective (U3) canonical-CBOR byte
encoding of the AAD-bound `BTreeMap<Did, MemberEntry>` snapshot (`MemberEntry` field order + `Option<SigPubKey>`
presence-encoding + `Hlc` encoding) or two engines materialize divergent AAD for the same membership.
**Canonical enum representation (R4.2 F4-007 ruling, Ben 2026-06-03): ALL `MemberEntry` enums serialize as an
INTEGER discriminant in canonical-CBOR/AAD — `RoleId` as its `u8` ordinal AND `MemberRef` as a `u8`-tagged
variant (NOT a text string).** Int-discriminant is canonical for determinism + AAD compactness; the golden
vectors must encode `member_ref` as an integer tag, symmetric with `role`. (Fixes the asymmetric int/text
representation frozen in the R3 golden.)

**`Drop` (one-shot non-member share).** The existing `benten-drop/` content-bundle — a sealed sibling of the
graph (content = encrypted Nodes + a SubgraphSpec; envelope = frozen crypto wire; provably carries no
`K_Set`). KEEP-AS-IS (GN-1 §2.5).

**`RestrictedScopeSet` COLLAPSE (BC-1 / CA-1).** Not a primitive — it is the K(N) chain (`structural_kdf.rs`)
+ a per-member edge-label allowlist inside a grant (a capability). Wire bytes + containment algebra
(`combinators.rs` union/intersect; `scope.rs` 2-arm) KEPT verbatim; the collapse changes the name + mental
model, not the bytes.

### §3.5.HLC `created_at_hlc` vs `admitted_at_hlc` (M-7 definition gap)

- **`admitted_at_hlc`** (`MemberEntry`) = the HLC stamp of the WRITE that admitted this member to the set
  (per-member; LWW-resolved like any property — LARGER HLC wins, the in-tree `crdt.rs:535` rule).
- **`created_at_hlc`** (Inv-21) = the HLC stamp of the **set anchor's creation event** (per-set; immutable;
  the fork-tie-break discriminant). They are DISTINCT clocks: `admitted_at_hlc` is a member-property; only
  `created_at_hlc` participates in the Inv-21 fork-tie-break.

### §3.6 The three orthogonal axes

- **§3.6.A Scale/identity** (the ONLY keying axis): `MembershipSetKind` EXACTLY-3 + 2 keying-reserves.
  Per-Kind constructors validate cardinality (Atrium ≥1 admin; DeviceMesh exactly-1 user-DID admin;
  SingleDevice exactly-1 self-admin). Per-Kind wire-cost (Compromise #46: Atrium 32 / DeviceMesh 5 /
  SingleDevice 1).
- **§3.6.B Governance**: `GovernanceConfig` top-level **signed config Node** (in-tree `InstallRecord`
  precedent; `PLUGIN-MANIFEST.md:68`) with `tier: GovernanceTier { Flat (Atrium) | Moderated (Garden) |
  Polycentric (Grove) }`. Promotion = add the Node + admin grants + roles; NO re-key, NO new identity, NO
  Kind change. **5-value `RoleId` — ALL 5 ACTIVE at v1-beta (BC-9); permission-sets pinned at Core as UCAN
  templates (M-11 / R1-Q-9):**

  | RoleId | ordinal | UCAN ability-template (v1-beta, pinned at canary; golden-vector test) | hard constraint |
  |---|---|---|---|
  | **Admin** | 4 | `{read, write, share, moderate_content}` ∪ `{admit-member, kick-member, rotate-keys, assign-roles, edit-governance-config}` — i.e. the full Moderator set PLUS the five admin-only abilities | superset (⊇ Moderator — M-11) |
  | **Moderator** | 3 | `{read, write, share, moderate_content}` (hide/flag) — **NO admit-member / kick-member / rotate-keys / assign-roles / edit-governance-config** | **Moderator ⊆ Admin (M-11)** |
  | **Member** | 2 | read / write (own) / share-within-policy | — |
  | **Viewer** | 1 | read only | — |
  | **Invitee** | 0 | **NONE — derives ZERO content** (pre-acceptance handshake state only; no `K(N)`, no read) | **Invitee = zero content (M-11)** |

  The RoleId *ordinal* is keying-frozen (`role_assignments_generation` ∈ AAD); the per-role *authorization
  semantics* compose from UCAN + the signed `GovernanceConfig` permission-config Node, NOT a frozen
  permission-flags bitfield. **Ordinal supersedes M-CONS-FINAL (M-13):** M-CONS-FINAL had Viewer=0/Invitee=1;
  this R0 pins **Invitee=0 (the zero-content floor) / Viewer=1**. Because the ordinal is AAD-keying-bound, the
  exact table is pinned at canary with a golden-vector test; the swap is flagged "ordinal supersedes
  M-CONS-FINAL per ship-all-5". (The stale Inv-20 clause-j "3 active, 2 reserved" wording is corrected to
  "all-5-active" — M-13 / WF-9 / MD-9.) Governance *workflows* (voting/moderation UX) → Composing.
- **§3.6.C Federation**: `MemberRef::SubsetRef(MembershipSetId, KSetAcquisitionPath)` — an edge to another
  MembershipSet anchor (reserved-and-refused at v1-beta). Inv-20 clause-k: `MEMBERSHIP_RECURSION_MAX_DEPTH =
  4` + cycle-detect. **`KSetAcquisitionPath` frozen field set (M-9):** the FROZEN minimal fields making
  depth-4 + cycle-detect **offline-decidable** = `{ target_set_id: MembershipSetId, hop_path:
  Vec<MembershipSetId> (carries the visited-set so cycle-detect is PATH-CARRIED not receiver-local;
  bounded len ≤ 4), acquisition_proof_cid: Cid }`. Path-carried (not receiver-local) so an offline verifier
  with only the wire bytes can decide depth + cycle without external graph state. Inv-20 clause-l: Model-B
  (independent-`K_Set`-per-set) default; Model-A opt-in is post-v1-beta additive. NQ-D3 (R2) refines the exact
  field encoding.

### §3.7 The member model — Principal + derived nature

**Member = a Principal admitted to a MembershipSet (a RELATION).** Derived nature (Inv-22):
`is_ai_operated(did) = (did.method() == "agent")` (`did:agent:` is an OPTIONAL external-interop alias on the
`benten-id` multikey allowlist — NOT a stored discriminator); `is_plugin(Principal)` = "runs a
manifest-bounded subgraph?" (Inv-14 already derives "Plugin" at `INVARIANT-COVERAGE.md:320`);
`is_autonomous_ai(Plugin)` = "manifest declares autonomous operation?" (PA-1); AI-ownership = derived from
the UCAN delegation graph (`root_issuers(agent_did)` walked against `members_table`). Any cached nature flag
is an **IVM-materialized derived view**, never stored member state. **Net new frozen member field = ZERO.**
Trust categories → 2 (Users+Devices / Plugins-incl-agents; a Plugin-Principal is a member iff explicitly
admitted — a DEFAULT not a prohibition). **Precision (m-15 GNC-7):** the `MemberRef` variant is
**Kind-determined** (UserDid↔Atrium / DeviceDid↔DeviceMesh / LocalDevice↔SingleDevice), NOT a per-member
nature discriminator — it does not encode operator-nature (Inv-22 boundary).

### §3.8 Governance-as-graph-Nodes + audit-as-version-chain (GN wins)

- **Audit log** (GN-1 §2.1): membership admin ops (admit/kick/role-change) are WRITEs that advance the audit
  sequence + fire `ChangeEvent` carrying the `(actor_cid, handler_cid, capability_grant_cid)` attribution
  triple (`store.rs:468`). **Precision (m-15 GNC-2):** that triple is `Option`; a bare `put_node` leaves it
  unset — so the audit-emitting path MUST be the **enforced engine-API WRITE** (the
  `is_actor_active`-gated `Engine` path), not a backend `put_node`. Tamper-evidence = a version-chain
  (Crosby-Wallach ≡ Anchor + immutable Version Nodes + CURRENT; content-addressing gives the property cheaply
  once the enforced path is used). `AuditAccessGradation` = the **UCAN read-scope** on the `audit:<set_id>:*`
  `RestrictedScope` (`AdminOnly` = only Admins hold the read cap; `PublicAllMembers` = all members hold it);
  **the scope string `audit:<set_id>:*` is a `RestrictedScope` arm — m-15 GNC-1 / M-17**. The 4 "reserved
  variants" (MemberOnly_OwnEvents / ThresholdAdmin_M_of_N / TimeLocked / Anonymized_Aggregate) =
  UCAN-caveat / IVM-view compositions, **NOT frozen codepoints**. `audit_log_query` is a GRAPH-NATIVE
  scoped-READ (§1.5; NOT a frozen op). Compromise #58 (insider-correlation) survives as an honest disclosure;
  **m-7: clause-d "per-recipient unlinkability = network-observer-only" — it does NOT protect against a
  malicious admin** (carried into §3.8 + THREAT-MODEL.md so the audit narrative is not over-read).
- **Membership events** (GN-1 §2.2): the set's evolution = the Anchor+Version+CURRENT version-chain
  (DAG-shape for forks). A "fork" IS the DAG-branch the version-chain already supports; **Inv-21's
  fork-tie-break is the CRDT-convergence rule over that DAG** (§3.8.Inv-21). The keying `members_table`
  **snapshot** (CURRENT-materialization, AAD-bound) stays frozen; the event *log* is graph-native.

### §3.8.Inv-21 Fork-tie-break (M-7 asymmetry + M-8 totality — load-bearing)

**The asymmetry is DELIBERATE and must be stated explicitly (M-7).** The in-tree CRDT property rule is
**LARGER-HLC-wins (LWW)** (`crdt.rs:535`: keep the entry where `cmp_lex(prev) == Greater`). Inv-21's
fork-tie-break is the **OPPOSITE**: **SMALLER `created_at_hlc` wins** = **oldest-anchor-wins** for forks.
**Justification:** for a property, last-writer-wins (newest intent) is correct; for a *set-identity fork*,
the **canonical lineage is the OLDEST anchor** (the original set), so a later adversarial re-fork can never
displace the original by simply stamping a larger HLC. These are two different convergence rules over two
different object classes (property vs set-identity); the R0 does NOT claim byte-equivalence (correcting the
R0.1 "byte-equivalent" framing).

**Totality (M-8 — required for the kani convergence proof).** `smaller MembershipSetId` can NEVER
disambiguate two concurrent forks off the SAME anchor (they share `membership_set_id`). When `created_at_hlc`
ties, the terminal discriminator is the **forking event's Version-Node CID** (content-addressed ⇒ always
distinct for two distinct fork events) — NOT `MembershipSetId`. **Total order = `(created_at_hlc ASC, then
fork_event_version_node_cid ASC)`.** This is total (the Version-Node CID is unique per event), satisfying the
kani convergence proof obligation. ALL event-authors fork (not just Admin); the losing fork's CRDT-vector
increments MUST NOT merge into the winner (archived-not-discarded). NQ-D2 (R2) pins the exact total ordering
+ the kani proof shape.

### §3.9 Transport / D6 (iroh-gossip privacy) + convergence model (M-10)

`TransportConfig` — iroh-gossip impl ships at v1-beta behind `GossipPlusBlobs` (BC-7). Privacy (P2 D6):
HMAC-blinded topic `topic = HMAC(K_Set, membership_set_id || generation_summary)` (truncated; **R0.7
precision: the frozen primitive is `blake3::keyed_hash(K_Set, membership_set_id || generation_summary)` — the
SAME BLAKE3-keyed MAC used for `membership_set_id_commitment`, truncate-to-32 being the native BLAKE3 width;
no hmac/sha2 dep; bytes unchanged**) + fork-rotation
(topic rotates on fork) + OOB rendezvous (bootstrap invite via a Layer-C Drop-bundle). The blinded-topic is
keying-derived crypto wire (frozen-crypto rule); the transport selection itself is a runtime/network +
codepoint-dispatch concern (NOT graph-data). Closes Compromise **#61** (fingerprint-leak via gossip topic).

**Convergence vs gossip-delivery (M-10 — the actual hard problem).** Ground-truth (§0.3): the in-tree
`Transport` trait has connect/accept/send/recv but **NO pub-sub/topic/broadcast** — gossip is genuinely
net-new. **Convergence is backed by the EXISTING MST anti-entropy** (`benten-sync/src/mst.rs`); **iroh-gossip
is LIVENESS/notification ONLY** (it tells a peer "something changed, come anti-entropy"). This de-risks the
wave: gossip's no-causal-delivery / out-of-order / duplicate-delivery properties do NOT threaten convergence
because the MST diff + HLC causal order + Inv-21 partition rule are the convergence backstop, and the MST
exchange is order-independent + idempotent. **NQ-D1 (R2):** does the new `GossipTransport` land as a
`benten-sync` trait or in `benten-membership-set`? (strong lean: `benten-sync`, alongside the existing
transport + MST.) R2 seeds the gossip×HLC×Inv-21 test-family.

**`generation_summary` definition (O-5 / m-11 / NQ-D4):** for steady-state rendezvous the topic is derived
from the **set-generation counter** (NOT a per-member generation-vector) so all current members of a
generation compute the same topic and meet; a genuine FORK rotates the generation ⇒ rotates the topic;
losing-fork members re-converge by anti-entropy onto the winning fork's NEW topic (they learn it via the MST
exchange + the OOB rendezvous fallback). Topic-rotation freshness rides generation-counters, NOT a
time-bucket (m-11). OOB-bootstrap first-contact metadata residue cross-links #43/#62 (m-11).

### §3.10 Crypto-suite encryption hardening + the nonce-cache requirement (M-1)

- `role_assignments_generation: u32` in the `0x6610` group AAD (the 11-field set below; supersedes the prior
  "9-tuple" framing) + `E_ROLE_STALE_AT_VERIFY` (rejects a stanza sealed under a stale role-snapshot).
- The **`0x6610` group per-stanza AAD** (Inv-20 clause-c; BLINDED — RATIFIED) — **MembershipSet group sends
  honor Sealed-Sender (F-LC-9):** the inner-sender-DID is bound INSIDE the sealed/encrypted part per stanza
  (NOT in plaintext AAD), so the on-wire AAD binds the 11-field set `{ aad_version (0x01, u8),
  codepoint (0x6610, u16 BE), body_cid (self-describing CIDv1), member_count (u32 BE),
  audience_set_commitment (32B), stanza_index (u32 BE), stanza_count (u32 BE),
  member_key_generation (u32 BE), membership_set_id_commitment (32B), membership_set_generation (u32 BE),
  role_assignments_generation (u32 BE) }`. The sealed-inner-sender-DID is the post-decrypt-verified
  inner-payload sender-DID, NOT an on-wire plaintext field. **Two fields are BLINDED** (per Compromise #61 /
  §3.9 posture): `audience_set_commitment = BLAKE3(0x01 || lp(did_0) || lp(did_1) || …)` over the canonical
  SORTED recipient-DID list (replaces the raw member-DID-list), and `membership_set_id_commitment =
  HMAC(K_Set, "benten:setid:v1" || membership_set_id)` truncated to 32 bytes — **R0.7 precision: `HMAC` here
  is `blake3::keyed_hash(K_Set, …)`, BLAKE3's native keyed MAC (no hmac/sha2 dep; truncate-to-32 = native
  BLAKE3 width; bytes unchanged)** — (the §3.9 gossip-topic
  construction; replaces the raw `membership_set_id`). **`stanza_count` is bound alongside `stanza_index`** as
  a truncation/censorship defense (an active relay cannot silently drop trailing stanzas — each survivor would
  fail the bound count). Recipients hold `K_Set` + the member list ⇒ recompute + verify both commitments ⇒ all
  binding properties (cross-stanza substitution U17; inter-member non-forgeability) PRESERVED; the relay sees
  only opaque 32-byte tags. **`body_cid` is a self-describing CIDv1** (`0x01 0x71 0x1e 0x20 || 32-byte BLAKE3
  digest`), NOT a bare fixed-32 (CLAUDE.md baked-in #5 "never hardcode … sizes"; restores U3 length-injectivity
  for free). **Honest scope:** this is identity-HIDING, NOT unlinkability — the commitment recurs for a static
  group (network-observer correlation remains); full per-send unlinkability = U25, CODEPOINT-RESERVE for v1-GM,
  additive with no wire break. Load-bearing for inter-member non-forgeability; SECURITY-PROOFS.md states the
  per-stanza-LIVE vs envelope-CONSTANT decomposition. **Precision (m-15 GNC-5):** the AAD assembly passes
  OPAQUE bytes to the crypto-suite (`benten-membership-set` assembles the field-set ⇒ canonical bytes ⇒ hands
  `&[u8]` to `benten-crypto-suite`); the crypto-suite has NO reverse dependency on membership-set (the AAD is
  opaque to it).
- `key_retention_window_secs` (default 604800s = 7d, user-definable).
- Per-stanza `Option<ChainedStateTlv>` codepoint-reserve sub-slot (pairs with `RotatingGroupKeyChainedMode`
  reserve) + the future-ratification-gate AAD-bind test arm.

**The nonce-cache as a NAMED load-bearing v1-beta REQUIREMENT (M-1 / m-8 / NQ-T4).** The replay window for
DeviceLink + RemotePermission rides on a **UCAN-token/`jti`-keyed nonce-cache that is NET-NEW at v1-beta** — it
re-uses the durable-CAS-marker *pattern* shipped as Compromise #25, NOT the #25 sync-frame instance itself
(#25 rejects replay of previously-seen *sync envelopes*; the Layer-D cache is a distinct `jti`-keyed instance,
zero shipped at HEAD), NOT generation-counters and NOT the coarse 1-hour bucket. v1-beta MUST pin the
nonce-cache contract:
- **Scope:** per-device AND user-global semantics defined (does device C reject a nonce device B consumed? —
  NQ-T4 R2; default = per-device-durable + best-effort-global-via-sync).
- **Retention:** ≥ the full 1-hour bucket window (so an intra-hour replay is always caught).
- **Durability:** survives engine restart (persisted, not RAM-only).
- **Multi-device:** the cross-device shared-rejection semantics (NQ-T4).
- **Test pin:** an **intra-hour-replay-rejected-by-the-nonce-cache** test at the Layer-D wave (a re-presented
  PermissionGrant within the same 1-hr bucket is rejected by the nonce-cache, NOT by the time field).

**The 1-hour bucket is Layer-D-ONLY (M-14).** `DropToRecipient` carries **NO `sealed_at`/`valid_until`** —
drops are forever-valid (per #62; freshness rides recipient-key-generation + the nonce-cache, NOT a
timestamp). The U28 1-hour coarse bucket applies ONLY to the DeviceLink + RemotePermission epoch fields
(U5). L6's `DropToRecipient { sealed_at_epoch_seconds, valid_until_epoch_seconds }` HIGH-leak finding is
**closed-by-exclusion** (an implementer following that struct would leak ~1-sec timestamps the bucket never
reaches; the R0 states the exclusion explicitly so the struct is never built). **Granularity rationale (m-9 /
m-10):** 1-hour is the knee of the curve (33→14 bits/year of timing entropy; coarser degrades Layer-D UX;
finer is wasted given the nonce-cache does the actual replay defense). Default = **round-down, NO jitter**
(m-9: jitter↔clock-skew can double-widen the effective window to ≤2×jitter+skew — round-down-no-jitter avoids
it cleanly; NQ-C5 R2). Rationale → SECURITY-PROOFS.md.

### §3.11 Sealed-Sender abuse-control mechanism (BR-1 — NEW Core scope)

Because the DEFAULT Layer-C path (`0x6510`) carries NO plaintext sender identity, abuse/spam control cannot
rely on per-sender filtering. The v1-beta-Core abuse-control mechanism (~5–8 wave-days; Signal's delivery-
token pattern adapted to Benten's capability discipline):
- **Recipient-issued delivery tokens.** A recipient (or a MembershipSet admin on behalf of members) issues
  short-lived, rate-limited UCAN-backed *delivery tokens*; a Sealed-Sender envelope without a valid token is
  refused at the receive boundary (BEFORE decrypt). This binds abuse-control to the capability spine without
  re-exposing the sender identity.
- **Per-token rate-limit + revocation** via the existing UCAN `nbf`/`exp` + revocation substrate.
- **Compromise #63 (NEW)** records the residual: Sealed-Sender shifts abuse-control from sender-identity
  filtering to recipient-issued tokens; a recipient who over-issues tokens re-admits spam (accepted
  trade-off; mitigated by default-conservative token rate-limits).
This is the scope BR-1 pulls into Core. It is wire-affecting only in the token-binding AAD (a Sealed-Sender
sub-field), so it MUST land pre-freeze. **The token-binding AAD carries NO `coarse_epoch`** (freeze record):
freshness rides the delivery token's own UCAN `nbf`/`exp` + the `jti`-keyed nonce-cache (§3.10), never a
time-bucket — consistent with the 1-hr bucket being Layer-D-ONLY (RULING-1 / §3.10 / M-14).


---

## §4 Wire-format / frozen-interface inventory

Disposition legend: **FREEZE** (frozen wire-format, interface-frozen at G-CORE-9 + externally audited) /
**CODEPOINT-RESERVE** (slot locked at v1-beta; impl deferred) / **GRAPH-NATIVE** (composes from Nodes + 12
primitives + UCAN + IVM + version-chains; NOT frozen wire) / **PHASE-LATER-DEFER** (named destination +
horizon). The frozen-crypto rule (§1.5) is applied throughout.

### §4.0 Canonical codepoint allocation table (B-3 / m-13 — THE single source of truth)

> **Authoritative home = `docs/CRYPTO-CODEPOINTS.md`** (created at the doc-wave; this table is its content).
> **Reconciles BOTH source docs (9-eyes wins the `0x6380/0x6390` collision; §0.4).** All integers are in the
> Benten envelope range **`0x6100..0x6FFF`** (IANA-disjoint per U8/U11 — it is a **Benten-private namespace**;
> the *component-algorithm* IDs inside HPKE/COSE reference the IANA HPKE/COSE registries, but the
> *envelope-selector* codepoints are Benten-owned and avoid the IANA HPKE `kem_id`/`kdf_id`/`aead_id` 16-bit
> ranges by living in this private band). **Sig codepoints** live in a SEPARATE `0x00xx` namespace (in-tree).
> **Intra-`0x6100..0x6FFF` non-collision is ASSERTED below** (a CI scanner enforces it — Inv-18 / NQ-W2).

| Codepoint | Symbol | Family / band | State @ v1-beta | Source / notes |
|---|---|---|---|---|
| `0x0001` | `SigCodepoint::HYBRID_ED25519_MLDSA65` | Sig (separate namespace) | **LIVE** (in-tree) | LAMPS Composite ML-DSA; default sig (#31) |
| `0x0002` | `SigCodepoint::CLASSICAL_ED25519` | Sig | reserved (in-tree) | non-default downgrade |
| `0x0003` | `SigCodepoint::HYBRID_MLDSA65_SLHDSA` | Sig | reserved-swap-matrix (in-tree) | typed-reject default |
| `0x6100..0x61FF` | Layer-A vault / symmetric-AEAD band | Vault / symmetric | band reserved | U11 |
| `0x6100` | `SymmetricAeadXNonce` (XChaCha20-Poly1305, 24-byte nonce) | Vault (Layer-A) | **FREEZE + SHIP** | vault envelope; reseal-heavy 24-byte nonce (m-4) |
| `0x6101` | `SymmetricAead` (ChaCha20-Poly1305, 12-byte nonce) | Symmetric-AEAD | **FREEZE** | distinct construction from `0x6100`'s XChaCha20-24B; both ship at v1-beta (§4.1; U12/U32) |
| `0x6400` | `CipherSuiteCodepoint::CLASSICAL_X25519` | Cipher / Layer-B+C downgrade | **LIVE** (in-tree; **m-12**) | classical-only X25519 downgrade arm |
| `0x647a` | `CipherSuiteCodepoint::HYBRID_X25519_MLKEM768` | Cipher / KEM default | **LIVE** (in-tree) | C-5; **real X-Wing SHA3-256 @ BR-3**; ChaCha20-Poly1305 bulk |
| `0x647b` | `CipherSuiteCodepoint::HYBRID_MLKEM768_HQC` | Cipher / NF-1 end-state | reserved (in-tree; typed-reject) | NF-1 PQ⊕PQ |
| `0x647c` | `CipherSuiteCodepoint::PURE_PQ_MLKEM768_ONLY` | Cipher / swap-matrix | reserved-named (in-tree; typed-reject; audit-gated) | Inv-17: never LIVE at v1-beta/v1-GM |
| `0x6500` | `LAYER_C_DROP` (plaintext-sender) | Layer-C drop / recipient band | **FREEZE** (non-default sibling) | U4 sender-DID-in-AAD applies here |
| `0x6510` | `DROP_TO_RECIPIENT_SEALED_SENDER` | Layer-C drop / recipient band | **FREEZE + SHIP — v1-beta DEFAULT (BR-1)** | sender-DID inside ciphertext; #43-improving |
| `0x6520` | `LAYER_C_DROP_MULTI_RECIPIENT` (`HpkeMultiBase` group) | Layer-C group multi-stanza | **FREEZE** | U17 cross-stanza AAD defense; **per-stanza AAD BLINDED (R0.7) — see §3.3 / §4.1 `0x6520` AAD field-set** (audience_set_commitment + stanza_count + self-describing body_cid; NOT a MembershipSet — no set-id/generation fields) |
| `0x6310..0x631F` | DeviceLink band | Layer-D device-link | **FREEZE** | `ProvisioningOffer`/`Payload`/`InnerPayload` |
| `0x6320..0x632F` | RemotePermission band | Layer-D remote-permission | **FREEZE** | `PermissionRequest`/`PermissionGrant`; incl. `ExecuteWorkflow` reserve |
| `0x6380..0x638F` | **MLS-Application** | FS-future bracket | **CODEPOINT-RESERVE** | U13 (9-eyes; NOT MembershipSet) |
| `0x6390..0x639F` | **MLS-Welcome** | FS-future bracket | **CODEPOINT-RESERVE** | U13 (9-eyes; NOT Sealed-Sender) |
| `0x63A0..0x63AF` | CGKA-Commit | FS-future bracket | **CODEPOINT-RESERVE** | U13 |
| `0x63B0..0x63BF` | Bird-of-Prey AKEM | FS-future bracket | **CODEPOINT-RESERVE** | U13 (SUF-preserving sig future) |
| `0x63C0..0x63CF` | draft-prabel | FS-future bracket | **CODEPOINT-RESERVE** | U13 |
| `0x6600` | `MEMBERSHIP_SET_ENCRYPTION` (set-keying envelope) | **MembershipSet band (RELOCATED from `0x6380`)** | **FREEZE** | **§0.4 collision fix**; was M-CONS-FINAL `0x6380` (collided MLS-Application) |
| `0x6610` | `MEMBERSHIP_SET_GROUP_MULTI_STANZA` | MembershipSet band | **FREEZE** | group K_Set multi-stanza |
| `0x6620` | `MEMBERSHIP_SET_SUBSET_REF` (federation `KSetAcquisitionPath`) | MembershipSet band | **CODEPOINT-RESERVE** | refused at v1-beta; Inv-20 k |
| `0x6700..0x67FF` | revocation / lifecycle band | lifecycle | **FREEZE** (scaffolding) | U16; `CodepointLifecycle` |
| `0xFE00..0xFFFE` | experimental range | experimental | **CODEPOINT-RESERVE** | U11 |
| `0xFFFF` | extended-codepoint escape | escape | **CODEPOINT-RESERVE** | U11 |

**Non-collision assertion:** every assigned integer above is unique; the MembershipSet band (`0x6600..0x66FF`)
is disjoint from the MLS/FS-future bracket (`0x6380..0x63CF`), the drop/recipient band (`0x6500..0x652F`), the
Layer-D bands (`0x6310..0x632F`), and the cipher band (`0x6400`/`0x647x`). **Sealed-Sender has exactly ONE
canonical value (`0x6510`).** The CI scanner (NQ-W2 / Inv-18) asserts intra-band non-collision + that no
Benten envelope codepoint lands in an IANA HPKE registry range.

**REJECTED: width-unification (freeze record — do not re-litigate).** A future round MUST NOT unify the
u16/u32 per-band length-prefix widths across the Layer-C drop band and the MembershipSet band. The two bands
are separately-frozen, codepoint-discriminated byte-strings (the corpus author adjudicated this deliberately);
each band's AAD field-set (§4.1, §3.3, §3.10) is frozen as authored. There is no cross-band parsing path that
would benefit from a unified width, and unifying them would be a wire-break for one band.

### §4.1 Encryption envelope + codepoints (Layers A–D)

| Surface | Disposition | Notes |
|---|---|---|
| `EncryptedEnvelope` shape (`EnvelopePayload` `#[non_exhaustive]` + `BindingContext` `#[non_exhaustive]`) | **FREEZE** | Inv-16; codepoint-dispatch; strict-decode (U2); typed-reject. **Renames in-tree `AeadEnvelope` (M-18; §4.1.MIGRATION)** |
| `EnvelopePayload::SymmetricAead [u8;12]` (ChaCha20-Poly1305) | **FREEZE** | U12 |
| `EnvelopePayload::SymmetricAeadXNonce [u8;24]` (XChaCha20-Poly1305; **Layer-A vault uses THIS** — m-4) | **FREEZE** | U12/U32 (ship both at v1-beta) |
| `EnvelopePayload::HpkeBase[MLKEM768-X25519]` (Layer-C single / Layer-D wraps) at `0x647A` | **FREEZE** | C-5; HPKE-11-KE (Q4); real X-Wing (BR-3) |
| `EnvelopePayload::HpkeMultiBase` (group multi-stanza) at `0x6520` | **FREEZE** | U17; cross-stanza AAD defense; **per-stanza AAD BLINDED (R0.7) — see the `0x6520` AAD field-set row below** |
| **Sealed-Sender `DROP_TO_RECIPIENT_SEALED_SENDER` at `0x6510`** | **FREEZE + SHIP — v1-beta DEFAULT (BR-1)** | flips from R0.1 CODEPOINT-RESERVE; ships at Core; pulls #59/#63 abuse-control into Core; R1-Q-2 |
| Plaintext-sender `LAYER_C_DROP` at `0x6500` (non-default sibling) | **FREEZE** | U4 sender-DID-in-AAD; Compromise #43 (now improved by default-Sealed-Sender) |
| AAD codepoint binding + `aad_version: u8` prefix + canonical-TLV length-injective | **FREEZE** | U1/U3/U14 |
| **`0x6510` single-recipient AAD field-set** = `{aad_version(0x01,u8), codepoint(0x6510,u16 BE), audience(recipient DID, u32-BE length-prefixed), body_cid(self-describing CIDv1), recipient_key_generation(u32 BE)}` (minimal-sufficient union, Option A) | **FREEZE** | D1; NO stanza-index / NO sender_did (sealed inside ciphertext) / NO coarse_epoch; `body_cid` self-describing (D3; #5) |
| **`0x6610` group per-stanza AAD field-set** = `{aad_version(0x01,u8), codepoint(0x6610,u16 BE), body_cid(self-describing CIDv1), member_count(u32 BE), audience_set_commitment(32B), stanza_index(u32 BE), stanza_count(u32 BE), member_key_generation(u32 BE), membership_set_id_commitment(32B), membership_set_generation(u32 BE), role_assignments_generation(u32 BE)}` (11 fields; BLINDED) | **FREEZE** | D2; `audience_set_commitment`=BLAKE3 over sorted DIDs + `membership_set_id_commitment`=HMAC(K_Set,"benten:setid:v1"‖id)/32 — **R0.7: `HMAC`=`blake3::keyed_hash` (native BLAKE3 keyed MAC; no hmac/sha2 dep; bytes unchanged)** — (§3.9 / #61); `stanza_count` truncation-defense (D4); sealed-sender-DID inside ciphertext; identity-HIDING not unlinkable (U25 v1-GM) |
| **`0x6520` group per-stanza AAD field-set** = `{aad_version(0x01,u8), codepoint(0x6520,u16 BE), body_cid(self-describing CIDv1, 36B), recipient_count(u16 BE), audience_set_commitment(32B), stanza_index(u32 BE), stanza_count(u32 BE), recipient_key_generation(u32 BE)}` (8 fields; BLINDED; NOT a MembershipSet) | **FREEZE** | R0.7; Layer-C multi-recipient group (`HpkeMultiBase`); `audience_set_commitment`=BLAKE3(0x01‖lp(did)…) over sorted recipient DIDs (replaces raw roster); `stanza_count` truncation-defense; `body_cid` self-describing (#5); NO membership_set_id/generation/role fields (NOT `0x6610`); `recipient_count` u16 BE per Layer-C drop band (§4.0 width-unification-rejected); sealed-sender-DID inside ciphertext (F-LC-9); identity-HIDING not unlinkable (U25 v1-GM); **corpus F-LC-2 assembler+golden migrate at R4.6/R5** |
| `sealed_at` + `valid_until` epoch (DeviceLink + RemotePermission **ONLY** — M-14) | **FREEZE** | U5; coarse 1-hour bucket (U28) is **Layer-D-ONLY (RULING-1 / §3.10)**; **DropToRecipient + the `0x6510`/`0x6610` AAD carry NO coarse_epoch** — Sealed-Sender Drop freshness rides recipient-key-generation + the nonce-cache, NOT a time field |
| **BE endianness** — ALL multiformats-framed integer wire/AAD fields → BE. **Complete M-19 site-list:** codepoint `aead.rs:165` + AAD `chunk_index`/`total_chunks`/`recipe_index`/`total_recipes` `aead.rs:244,277` + `aead_wrap.rs` + platform-foundation + **`structural_kdf.rs:157`** + **`varsig.rs:47` + `varsig.rs:107`** + **`sizes.rs:183`** + **`swap_matrix.rs:1539,1540,1548,1550`**; `ENVELOPE_FORMAT_VERSION_V2` | **FREEZE** | Q2 / U7 / M-19; X-Wing corrective bundles this; conformance test asserts no `to_le_bytes` survives on any wire/AAD/keying path |
| DUAL-CID (`envelope_blob_cid` + `plaintext_cid`; MembershipSet adds `plaintext_cid_local` LOCAL-ONLY + `plaintext_cid_set` HMAC-blinded) — **extends in-tree `TwoCidStore`** | **FREEZE** | Q3 / U18 / F18 / O-3 |
| `recipient_key_generation` + `k_principal_generation` tracking | **FREEZE** | U19/U20 |
| Codepoint registry IANA-disjoint (`0x6100..0x6FFF`) + scanner + `Did` multikey + `Did::Unknown` | **FREEZE** | U8/U11/U15; Inv-18; §4.0 |
| `0xFFFF` extended-codepoint escape + `0xFE00..0xFFFE` experimental range | **CODEPOINT-RESERVE** | U11 |
| `CodepointLifecycle` typed-state (Live/Deprecated/Quarantined/Burned) | **FREEZE** (scaffolding) | U16; Inv-18 |
| Layer-A vault format (`vault.cbor`; DAG-CBOR; Argon2id params + salt + 24-B nonce header) | **FREEZE** | e2r §2.2; m-4 |
| Layer-D `ProvisioningOffer` / `ProvisioningPayload` / `ProvisioningInnerPayload` (device-link) | **FREEZE** | e2r §7.2 |
| Layer-D `PermissionRequest` / `PermissionGrant` (remote-permission-call) | **FREEZE** | e2r §6.4 |
| **`PermissionOperation::ExecuteWorkflow { workflow_cid, input_node_cids, max_decrypt_count, result_recipient_pubkey, executor_did }` variant slot + AAD-binding** | **CODEPOINT-RESERVE** (slot + AAD frozen; runtime enforcement post-v1-beta) | **M-3 / U21** — Ben's rented-compute use case; was ABSENT in R0.1 |
| `BENTEN_VAULT_PASSWORD` env-var name + headless IPC channel format | **FREEZE** | e2r §4.4 |
| MLS-PQ / CGKA / Bird-of-Prey / draft-prabel codepoint brackets (`0x6380..0x63CF`) | **CODEPOINT-RESERVE** | U13; FS-gap future-additive |
| `RotatingGroupKeyChainedMode` + `ChainedStateTlv` sub-slot | **CODEPOINT-RESERVE** | M-C2-B-2 / F-FE-4 |
| Per-relay-unlinkability shape (U23) + size-class buckets (U24) + per-recipient-unlinkable (U25) | **CODEPOINT-RESERVE** | impl v1-GM-defer |
| `IdentityRecoveryBackend` `RecoveryArtifact` DAG-CBOR **codepoint slot** (the wire byte) | **CODEPOINT-RESERVE** at Core | **m-14: reserve ONLY the artifact codepoint at Core; the `RecoveryHook` TRAIT SHAPE defers to Composing** (don't freeze a trait ahead of its undesigned protocol — social-recovery vs Shamir vs hardware-token differ structurally) |

### §4.1.MIGRATION One `ENVELOPE_FORMAT_VERSION_V2` step (M-18 / M-19 / M-20)

The following are **ONE consolidated V1→V2 migration**, co-scheduled on Wave-0 (NOT three independent V2
bumps):
1. **BE endianness** (Q2 / M-19) — all multiformats-framed integer wire/AAD fields.
2. **Real X-Wing combiner** (BR-3) — construction change ⇒ golden-vector regen.
3. **`AeadEnvelope` → `EncryptedEnvelope` rename + structural lift + typed `BindingContext` AAD** (M-18) —
   the shipped `AeadEnvelope` (flat struct, untyped `&[u8]` AAD; `aead.rs:145`) becomes the codepoint-dispatched
   `EncryptedEnvelope` with a typed `BindingContext`. This is a rename+lift of a SHIPPED type, not a greenfield
   mint (grep `EncryptedEnvelope`/`BindingContext` = ZERO at HEAD).

All three ride **one** `ENVELOPE_FORMAT_VERSION_V1 → V2` bump + **one** full golden-vector regeneration pass.
**Hard DAG edge (M-20):** `[Wave-0] ──► [every encryption canary]` — Wave-0 MUST MERGE before any canary
authors envelope bytes, so all new codepoints are authored at **V2 + BE + EncryptedEnvelope** from their first
commit (no canary authors LE/V1 bytes that would force a second migration before freeze). NQ-W1 (R2) ratifies
the single-bump.

### §4.2 MembershipSet primitive surface (with GN wins)

| Surface | Disposition | Notes |
|---|---|---|
| `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` EXACTLY-3 | **FREEZE** | §15.c HALT-AND-SURFACE; Garden/Grove NOT here |
| `AtriumWithRotatingGroupKey` + `EphemeralLobby` keying-reserves | **CODEPOINT-RESERVE** | keying-relevant |
| `members_table: BTreeMap<Did, MemberEntry>` (fused snapshot; AAD-bound; canonical-CBOR per NQ-W4) | **FREEZE** | replaces 3 slots; ZERO nature field; CURRENT-materialization of the event version-chain |
| `MemberEntry { role, is_authority, sig_pubkey, admitted_at_hlc, member_ref }` | **FREEZE** | net new nature/kind field = ZERO; `admitted_at_hlc` ≠ Inv-21 `created_at_hlc` (§3.5.HLC) |
| `MemberRef::SubsetRef` federation variant + `KSetAcquisitionPath { target_set_id, hop_path, acquisition_proof_cid }` (path-carried cycle-detect; M-9) | **CODEPOINT-RESERVE** at `0x6620` | refused at v1-beta; Inv-20 k; offline-decidable |
| `RoleId` 5-value wire width — **ALL 5 ACTIVE** (BC-9); ordinal `Invitee=0…Admin=4` (supersedes M-CONS-FINAL — M-13) | **FREEZE width + all-5-active + ordinal** | ordinal is keying-AAD-bound; permission-sets = UCAN templates (§3.6.B); Invitee=zero-content, Moderator⊊Admin |
| `MembershipSetMetadata` (top-level) | **FREEZE** | admin-rename |
| `GovernanceConfig` (top-level Node REFERENCE) | **GRAPH-NATIVE** | signed config Node (InstallRecord precedent); tier preset |
| Garden/Grove governance sub-config (voting/moderation policy) | **GRAPH-NATIVE** | signed-config-Node content, NOT reserved sub-codepoints (−2 reserve) |
| `key_retention_window_secs` (default 604800) | **FREEZE** | M-C2 R-MCV2-6 |
| 0x6610 group AAD field-set incl. `role_assignments_generation` + `E_ROLE_STALE_AT_VERIFY` (opaque bytes to crypto-suite — m-15 GNC-5) | **FREEZE** | Inv-20 clause-c |
| Audit log (audit-event Nodes + version-chain + IVM view) | **GRAPH-NATIVE** | GN-1; −1 structure; enforced-WRITE-path (m-15 GNC-2) |
| `AuditAccessGradation` 4 reserved variants | **GRAPH-NATIVE** | GN-1; UCAN caveats / IVM projections; −4 codepoints |
| `audit:<set_id>:*` audit-read scope | **GRAPH-NATIVE** (a `RestrictedScope` arm — m-15 GNC-1) | NOT a frozen op; UCAN read-cap |
| `audit_log_query` op | **GRAPH-NATIVE** (scoped-READ; **NOT a frozen op** — M-17) | composes 12 primitives + UCAN; "24 ops" figure RETRACTED |
| Membership events (`MembershipEvent`) | **GRAPH-NATIVE** | GN-1; version-Node content; −1 wire-enum family. Inv-21 = CRDT tie-break over the DAG |
| Per-member K(N) walk-scope (ex-F28; backward-compat dispatch) | **FREEZE** | bytes unchanged; re-labeled |
| TransportConfig `GossipPlusBlobs` impl + HMAC-blinded topic + fork-rotation + OOB (gossip = liveness-only; MST = convergence — M-10) | **FREEZE + SHIP** | BC-7 / P2 D6 |
| TransportConfig Willow / iroh-roq / iroh-live | **CODEPOINT-RESERVE** | BC-7 |
| **MembershipSetEncryption codepoint family (`0x6600`) + group multi-stanza (`0x6610`)** | **FREEZE** | **RELOCATED from `0x6380` (§0.4 collision fix)** |
| **`member_type {Human,Device,Agent}` field** | **DROPPED** | CM-1+PA-1 (nature derived) |

### §4.3 Compute / economics (CE-1; ZERO MembershipSet freeze hook)

| Surface | Disposition | Notes |
|---|---|---|
| `PeerResource` / `ResourceKind` / `OwnerRef` / `CommunityEconomicPolicy` | **PHASE-LATER-DEFER** | `docs/future/compute-marketplace.md` (NEW) + V1-FROZEN **Rows D-28/D-29 (NEW — minted at doc-wave; in-tree max = D-27, M-16)**; the single destination doc = `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (the global D-NN namespace); GRAPH-NATIVE when activated |
| `economic_policy: Option<opaque>` reserve on `MembershipSetPolicy` | **DROPPED** | CE-1 H2; economics composes; ZERO freeze hook |
| Garden/Grove governance impl (voting/moderation) | **PHASE-LATER-DEFER** | Phase-4-Meta-Composing+ |
| Federation recursion impl (Model-A opt-in) | **PHASE-LATER-DEFER** | post-v1-beta |

### §4.4 Net frozen-surface summary (O-8 — single canonical tally)

Per the frozen-crypto rule, the frozen wire surface is the **minimum**: the §6.2 `EncryptedEnvelope`
codepoint family (Layers A/B/C/D incl. Sealed-Sender `0x6510` default) + the EXACTLY-3 `MembershipSetKind` +
the `members_table` keying snapshot + the 0x6610 group AAD field-set + the RoleId 5-value ordinal + DUAL-CID + the
device-link/remote-permission/vault formats + Inv-16..22. **GN wins (cheap-additive) shrink the surface
further:** −1 audit structure, −1 `MembershipEvent` wire-enum, −4 AuditAccessGradation codepoints, −2
Garden/Grove sub-codepoints = **−8 net** (the R0.1 "−6" double-counted; this is the single canonical tally —
O-8). **CE-1 drops the economic reserve** (ZERO hook). **BR-1 ADDS** the Sealed-Sender default ship +
abuse-control AAD sub-field (NOT a net-shrink item; honestly counted). The member side adds **ZERO** new
frozen field; the compute side adds **ZERO** reserve.


---

## §5 Invariants (Inv-16..22) + canonical Compromise table

### §5.0 Reconciliation with real in-tree state (RE-VERIFIED this pass)

**Invariants (M-15 — corrected).** `docs/INVARIANT-COVERAGE.md` on main registers **Inv-1…Inv-15**. **Inv-15
IS in-tree** (header `:1` "15 invariants"; `:14` "Inv-15 REGISTERED … NOT-YET-FULLY-ENFORCED-BY-AUTOMATION";
the Inv-15 row at `:49`; enforcement-completion is on the G-CORE-PQ-WIRE-1 path). **Inv-16…Inv-22 are
design-mints** that do NOT yet exist in-tree. So the **baseline is "Inv-1..15 in-tree; Inv-16..22 are
design-mints"** — this R0's §5.1 corrects the R0.1 off-by-one only in the OTHER direction the triage assumed:
**Inv-15 is NOT a "design-mint-to-register"; it is already registered** (see §13 M-15 DISAGREE-WITH-EXPLANATION).
The doc-wave REGISTERS Inv-16..22 (NOT Inv-15) and does NOT touch the INVARIANT-COVERAGE.md header count (it is
already correct at "15").

**Compromises (BR-2 — re-derived against ACTUAL in-tree state).** `docs/SECURITY-POSTURE.md` highest # on main
= **#31**, and the **#31 occupancy is the OPPOSITE of the R0.1 claim**:
- In-tree **#31 = revocation-reach** — it IS the summary-table row (`:91`), the callout (`:30`), and the
  detail (`:2712`).
- The **LAMPS section (`:2433`) is ORPHANED** — prose with NO summary-table row.
- **#30 = unaudited-PQ** is occupied (`:91`) — do NOT move.

**Ben ruling BR-2: LAMPS keeps #31.** The correct plan = **make LAMPS the #31 summary-table row** (the LAMPS
prose at `:2433` gains the summary row + becomes the canonical #31) + **move revocation-reach to #62** (a NEW
slot above the whole panel) + **update the 3 re-point sites atomically**:
1. Summary-table row `:91` — change the `| 31 | Revocation reach …` row to `| 31 | LAMPS Composite ML-DSA
   EUF-CMA-only NOT SUF-CMA (CLOSED-equiv via Inv-15) |` AND ADD a `| 62 | Revocation reach in
   encryption-at-rest … |` row.
2. Callout `:30` — re-point "Compromise #31 — Drop bundle revocation reach" to "**Compromise #62** — …".
3. Detail `:2712` — re-title "Revocation reach (§R6) — Compromise **#62** detail".
4. (No 4th site moves; the LAMPS prose at `:2433` is the one that GAINS the #31 summary row — it does not
   itself renumber.)

**#30 stays at #30.** The tracked-doc PR cascade (a SEPARATE workstream from this plan-doc) applies this to
`SECURITY-POSTURE.md`; this R0 records the correct plan. (The R0.1's "LAMPS keeps #31 because it's the in-tree
occupant" rationale was backwards — LAMPS is NOT the in-tree occupant; the ruling stands but the
justification is "BR-2 ratifies LAMPS-keeps-31 + the tracked-doc occupant moves to #62", not "LAMPS is
already there".)

### §5.1 Invariant set (Inv-16..22)

| Inv | Title | Status |
|---|---|---|
| Inv-1..14 | (Phase 1–4-Foundation) | in-tree, enforced |
| Inv-15 | Sig-bundle-CIDs never load-bearing identifiers (3-layer decomposition) | **in-tree, REGISTERED, partially-enforced** (NOT a design-mint; M-15) |
| **Inv-16** | Envelope-layer-unification + codepoint-dispatch + AAD-binding + strict-decode + canonical-TLV + sender-DID-or-Sealed-Sender + replay-window (primitive-neutral; full U1–U6 + U14/U17/U18/U19/U20 reference) | **design-mint** (encryption arc) |
| **Inv-17** | Hybrid-cryptography-mandatory floor (every KEM use site = PQ + classical; no pure-PQ codepoint LIVE/selectable at v1-beta or v1-GM; reserved-named-typed-rejected swap-matrix arms permitted + audit-gated — m-3; ANSSI/BSI/NIST SP 800-227 §4.4 aligned) | **design-mint** |
| **Inv-18** | Codepoint-registry-discipline + metadata-disclosure invariant + `CodepointLifecycle` typed-state (registry rows in `CRYPTO-CODEPOINTS.md`; IANA-disjoint range; plaintext-sender-AAD variants MUST disclose at SECURITY-POSTURE + a paired Sealed-Sender sibling MUST exist — **satisfied by `0x6510` being the DEFAULT**; Live→Deprecated→Quarantined→Burned) | **design-mint** |
| **Inv-19** | Encryption-substrate keying-function CRDT-input discipline (Path-A.5 K(V) at API boundary type-restricts payload to immutable Version-Node-CID + MembershipSet) | **design-mint** |
| **Inv-20** | MembershipSet primitive invariant — **12 clauses** (a–l): K_Set via multi-stanza-HPKE-Encap / FORK-ONLY rotation / 0x6610 group AAD field-set / per-recipient-unlinkability (**network-observer-only — m-7**) / generation-CRDT / Path-A.5 K(V) / TransportConfig+gossip / per-member K(N) walk-scope / per-DID MemberEntry fusion / **5-value RoleId all-5-active (corrected from "3 active 2 reserved" — M-13) + retention** / **clause-k federation recursion-bound (depth=4 + PATH-CARRIED cycle-detect — M-9)** / **clause-l Model-B independent-K_Set default** | **design-mint** (this doc) |
| **Inv-21** | MembershipSet-fork-tie-break HARD partition — **SMALLER `created_at_hlc` wins (oldest-anchor-wins; DELIBERATELY OPPOSITE to the in-tree LARGER-HLC-wins property LWW at `crdt.rs:535` — M-7)**; **tie-break TOTAL via the forking-event Version-Node CID (NOT MembershipSetId — M-8)**; ALL event-authors; losing fork's CRDT-vector MUST NOT merge into the winner; archived-not-discarded; pinned by kani + property tests | **NEW design-mint** |
| **Inv-22** | Member-nature is derived, never stored (no `MemberEntry` field, no Policy field, no wire slot; any cached nature flag is an IVM-materialized derived view; `MemberRef` is Kind-determined NOT a nature discriminator — m-15 GNC-7; in-tree precedent Inv-14) | **NEW design-mint** |

### §5.2 Canonical Compromise table (collisions resolved per BR-2; `disposition_class` per M-C1 C-A2)

`disposition_class`: `ATO` = Accepted-Trade-Off; `SGD` = Substrate-Guarantee-Disclosure; `CHD` =
Composition-Hazard-Honest-Disclosure; `OOS` = Out-Of-Scope; `MIT` = Mitigated-Open.

**#31 resolution (BR-2; ground-truth-verified — see §5.0):** **#31 = LAMPS** (the summary row LAMPS gains);
**#30 = unaudited-PQ (KEEP)**; **#62 = revocation-reach** (re-pointed from the in-tree #31 occupant); **#63 =
Sealed-Sender abuse-control (NEW, BR-1)**.

| # | Title | class | Source / resolution |
|---|---|---|---|
| #30 | Unaudited PQ primitives in the v1-beta hybrid default (`ml-dsa`/`ml-kem`; MITIGATED-by-hybrid; CLOSES-at-v1-GM / C-GM-AUDIT) | MIT | **IN-TREE; KEEP — do not move (M-5).** #32 is a Decap-axis REFINEMENT of #30; the **#30↔#32↔C-GM-AUDIT** chain is explicit |
| #31 | LAMPS Composite ML-DSA EUF-CMA-only NOT SUF-CMA (CLOSED-equiv via Inv-15) | MIT | **LAMPS KEEPS #31 (BR-2)** — the orphaned `:2433` prose gains the summary-table row |
| #32 | ML-KEM-768 Decap chosen-ciphertext side-channel (libcrux CT-mitigation) — a Decap-axis refinement of #30 | MIT | encryption arc (9-eyes); **explicitly cross-linked to #30 (M-5)** |
| #33 | Coercion / wrench attack OUT-OF-SCOPE — **+Layer-D approval-coercion vector (m-5):** coerced-approving-device (`RemoteUnlock`/`SignUcanDelegation` coercion) makes a coerced grant look legitimate forever via the audit-Node (distinct from #34 password-coercion) | OOS | 9-eyes; m-5 disclosure-scope extension |
| #34 | Password-knowledge implies full access (Argon2id defense-in-depth) | ATO | 9-eyes |
| #35 | Compromised-device retroactive decryption (no past-content FS at v1-beta; CGKA deferred) | ATO | 9-eyes |
| #36 | RAM-residency / coredump / swap forensic-extraction OUT-OF-SCOPE (zeroize+secrecy best-effort) | OOS | 9-eyes |
| #37 | No TEE / sealed-enclave attestation at v1-beta+v1-GM | SGD | 9-eyes |
| #38 | Physical-presence side-channels OUT-OF-SCOPE | OOS | 9-eyes |
| #39 | Supply-chain dependency-pinning posture (PARTIAL; cargo deny + RustSec + McMillion-not-Cryspen; **+`secrecy` new Layer-A dep — O-1**) | SGD | 9-eyes; O-1 |
| #40 | Build-time / reproducible-builds + SLSA-3+ posture (post-v1-GM) | SGD | 9-eyes |
| #41 | Cross-device-sync UX-vs-cryptographic boundary — **+revocation-propagation-lag (O-4):** a revoked device exercises a stale grant during partition (sub-clause; bounded by tight `exp`) | ATO | 9-eyes; O-4 |
| #42 | Layer-C FS-gap (HPKE-mode-base recipient long-term sk decrypts forever) | ATO | 9-eyes (U13) |
| #43 | Envelope metadata leakage to untrusted relays — **IMPROVED by BR-1:** Sealed-Sender DEFAULT removes plaintext sender-DID on the default path; **NO coarse-epoch on the Drop wire** (RULING-1 / §3.10 / M-14 — the 1-hr bucket is Layer-D-only); group-AAD set-identifying material is BLINDED (audience_set_commitment + membership_set_id_commitment per #61). Residual on the default Drop wire = audience (recipient DID) + the linkable-but-blinded group tags (mitigation roadmap U22–U28; full per-send unlinkability = U25 v1-GM-reserve) | ATO | 9-eyes (L6); BR-1; #61 |
| #44 | Long-term-confidentiality posture (BSI TR-02102-1; X-Wing acceptable-migration-window) | OOS | 9-eyes |
| #45 | ML-KEM-768 MAL-BIND-K-CT/K-PK binding-properties (connects to M-6 IND-CCA2-adversarial-recipient-seed) | ATO | MembershipSet panel; M-6 |
| #46 | HpkeMultiBase O(N) wire-cost > 32 recipients (Atrium 32 / DeviceMesh 5 / SingleDevice 1) | ATO | panel |
| #47 | Collaborative-edit-via-re-drop accepted v1-beta trade-off | ATO | panel |
| #48 | MembershipSet-shape-leak (shared_key compromise → fingerprint that generation; recovery via fork) | ATO | panel |
| #49 | MembershipSet-member-acting-as-storage-host trust-boundary collapse | ATO | panel |
| #50 | Permanence-stewardship dependency disclosure | SGD | panel |
| #51 | Tauri NAPI-RS marshaling-boundary side-channels | ATO | panel |
| #52 | MembershipSet-no-PCS-against-removed-members (fork-on-kick) + in-set role-downgrade subsume | ATO | panel (M-C3 F-FE-5) |
| #53 | TransportConfig-codepoint-reserve (NARROWED): iroh-gossip SHIPS; Willow/iroh-roq/iroh-live reserve | ATO | panel (Ben Q1) |
| #54 | Continuous-rotation deferral + post-v1-beta `AtriumWithRotatingGroupKey` revisit-trigger | ATO | panel (N3) |
| #55 | GDPR-RTBF honest-architectural-disclosure (P2P-by-design; apps layer crypto-shredding) | SGD | panel |
| #56 | Journalist per-message FS deferral (SEPARATE design class from group-key rotation — kept sharply distinct from #42/#52 per R1-Q-8) | SGD | panel (MINT confirmed) |
| #57 | RestrictedScopeSet/grant immutability honest-disclosure (absorbs/cross-links #62) | SGD | panel |
| #58 | Audit-log insider-correlation (admin full visibility; **per-recipient-unlinkability is network-observer-only — m-7**; threshold-admin opt-in closes) | CHD | panel (M-C2-B-3 + P5) |
| #59 | KEM-key-confirmation under multi-stanza-HPKE-Encap — **+ re-scoped to Sealed-Sender abuse-control surface (BR-1; see #63)** | SGD | panel (M-C3); BR-1 |
| #60 | RBAC-role-transition does not invalidate prior UCAN attenuations — **+composition note (m-6):** under ship-all-5 + remote-permission an ephemeral UCAN survives role-downgrade; only `exp` bounds it ⇒ **tight-`exp` default mandate in §3.4** (cross-link #52) | SGD | panel (M-C3 F-FE-2); m-6 |
| #61 | MembershipSet-fingerprint-leak via iroh-gossip topic (CLOSED by P2 D6 HMAC-blinded topic) | CHD | panel (M-C3) |
| #62 | Revocation reach in encryption-at-rest (already-derived keys decryptable; Drops forever-valid within retention window) | SGD | **RE-POINTED from the in-tree #31 occupant (BR-2)**; cross-linked #57; the 3 in-tree loci re-point atomically (§5.0) |
| #63 | Sealed-Sender abuse-control trade-off — no plaintext sender ⇒ abuse-control rides recipient-issued delivery tokens; over-issuing re-admits spam (mitigated by conservative default token rate-limits) | ATO | **NEW (BR-1; §3.11)** |


---

## §6 Crate plan

### §6.1 The 15th crate — `benten-membership-set` (thin keying-glue Rust engine plugin)

Per GN-2 §3 + §15: `benten-membership-set` is a **permanent but thin** crate — a keying-glue mechanism (Rust
engine plugin / mechanism-half) whose entire governance/audit/members/economics/federation surface is **graph
Nodes** (data-half), with ZERO new frozen wire field beyond the keying minimum. It is the **model
SPLIT-endpoint crate**.

**What lives in the crate (RUST-mechanism, frozen):**
- The EXACTLY-3 `MembershipSetKind` enum + its codepoint family (`0x6600`/`0x6610`/`0x6620`).
- The multi-stanza-HPKE-Encap keying glue (delegates primitives to `benten-crypto-suite`; never forks).
- The `members_table` snapshot canonical-CBOR serialization (the AAD-bound keying minimum; NQ-W4).
- Per-Kind constructors + cardinality validation; **the 0x6610 group AAD field-set assembly → OPAQUE bytes handed to the
  crypto-suite** (m-15 GNC-5; no reverse dep); the fork-tie-break (Inv-21) CRDT rule; the federation
  recursion-bound (Inv-20 k).

**What lives as graph (data-half, GRAPH-NATIVE, never inside the sealed Policy):**
- `GovernanceConfig` / `RoleId` permission-semantics (UCAN templates) / Garden/Grove sub-config (signed Nodes).
- The members table relation + derived nature (IVM views).
- The audit log (audit-event Nodes + version-chain + IVM + UCAN; enforced-WRITE-path — m-15 GNC-2).
- Federation `SubsetRef` links; Compute / economics (`PeerResource` + `CommunityEconomicPolicy` — Phase-5+).

**Dependencies (B-1 — `benten-sync` ADDED):**
- `benten-crypto-suite` (HPKE / AEAD / structural-KDF primitives — the ONLY crypto call site, #5).
- `benten-core` (Node / Version-chain / CID).
- `benten-caps` (UCAN / grants — RBAC composition).
- `benten-id` (DID + DeviceAttestation + RotationLog).
- `benten-graph` (audit-sequence + ChangeEvent).
- **`benten-sync` (NEW — B-1; load-bearing).** The MembershipSet sync model runs ON `benten-sync`: HLC
  ordering (`crdt.rs`), Loro CRDT merge, MST anti-entropy diff (`mst.rs` — the **convergence backstop**, M-10),
  and the iroh `Transport` (`transport_trait.rs`). A membership-set crate that does the AAD-bound snapshot but
  cannot reach HLC-LWW merge / MST diff / transport **cannot converge a fork**. **Direction: `benten-sync` is
  UPSTREAM; `benten-membership-set` DEPENDS ON it** (never the reverse). **iroh-gossip placement (B-1 / NQ-D1):
  the new `GossipTransport` lands in `benten-sync`** (alongside the existing `Transport` + MST, where the
  transport + anti-entropy machinery already lives), NOT in `benten-membership-set` (carried as NQ-D1 to R2
  for final ratification; strong lean = `benten-sync`).
- **Never** the reverse (the keying crate is a leaf-ish addition; crypto-suite + sync do NOT depend on it).

### §6.2 The encryption substrate (lands across existing crates)

- `benten-crypto-suite` (RUST-FOREVER; the sharpest #5 floor): the §6.2 `EncryptedEnvelope` type (renamed from
  `AeadEnvelope` — M-18) + `EnvelopePayload`/`BindingContext` variants + codepoint dispatch + **the real X-Wing
  SHA3-256 combiner (BR-3)** + BE migration (M-19) + Argon2id DAK + HPKE (McMillion) + libcrux-ml-kem swap +
  the AAD-binding canonical_binding + **`secrecy` (NEW dep — O-1)**.
- `benten-graph` (Layer-A vault on-disk; per-Node AEAD at the storage boundary; ChangeEvent/audit-seq present).
- `benten-id` (DeviceAttestation + RotationLog present; `did:agent:` optional multikey alias).
- `benten-engine` (composition root: `Engine` vault + `UnlockedKeyMaterial` + `EngineLocked` typed-reject; the
  `DeviceAuthBackend` sealed trait; remote-permission-call + device-link wire protocol wiring; the
  enforced-WRITE audit-emit path — m-15 GNC-2).
- `benten-drop` (Layer-C Drop bundle already a content-bundle; extends with `HpkeMultiBase` + DUAL-CID +
  Sealed-Sender default).
- `benten-sync` (the new `GossipTransport` — B-1 / NQ-D1; HMAC-blinded topic + fork-rotation + OOB rendezvous).

### §6.3 The SPLIT model + what's Rust-engine-plugin vs graph

Per EP-1 / GN-2: every crate touched is a SPLIT crate — mechanism-half stays Rust (a Rust engine plugin),
data-half is graph. The `DeviceAuthBackend` trait is a **sealed policy seam** (Tier-2; Benten-internal, #7).
The crypto-suite codepoint dispatch is the **enum-dispatch crypto** Tier-3 (additive `match` arm + reserved
IANA-disjoint codepoint per #5). **IVM strategy is enum-dispatch (`benten_ivm::Strategy`), NOT a backend trait
seam** (m-15 GNC-4 / baked-in #2). No new `EngineExtension` / `ExtensionRegistry`; no registry; trust unchanged.

---

## §7 Wave-decomposition + sequencing

**Discipline:** canary-first (`feedback_canary_first_parallel_implementation`); parallelism cap = 7 implementer
agents (CLAUDE.md §13); Strategy-C batch-merge when ≥3 PRs accumulate; mini-review after every group;
wire-format-affecting → Phase-4-Meta-Core; UX-coupled → Phase-4-Meta-Composing; both pre-`v1-beta`-tag.

### §7.1 Phase-4-Meta-Core waves (wire-format-affecting; before G-CORE-9 freeze)

**Wave-0 (lands FIRST, independent; HARD upstream of every canary — M-20): the consolidated
`ENVELOPE_FORMAT_VERSION_V2` step** (~120–220 LOC; C-6/BR-3/Q2/M-18/M-19). FIRST sub-step = **ground-truth
which combiner `aead.rs`/`cipher_suite.rs` computes** (done: HKDF-SHA256, §0.3). Then: (a) **real X-Wing
SHA3-256 combiner** at `0x647A` (BR-3) + consistent classical-downgrade rewrite; (b) **BE sweep** across all
multiformats-framed wire/AAD integer fields (M-19); (c) **`AeadEnvelope`→`EncryptedEnvelope` rename + typed
`BindingContext`** (M-18); (d) **ONE** `V1→V2` bump + **full golden-vector regen** + a
`draft-connolly-cfrg-xwing-kem` interop KAT + the FIPS-203 cross-impl KAT (m-2) + the
`check-secret-independence` CI gate + a conformance test asserting NO `to_le_bytes` survives on any wire/AAD
path. Pre-tag-must-fix INDEPENDENT of F-full. **Merges before any canary authors envelope bytes.**

**Encryption-substrate canaries (parallel — touch different types; ALL author V2+BE+EncryptedEnvelope bytes
per the Wave-0 DAG edge):**
- **Canary-ENC-1: Layer-A K_principal store** (pull G-CORE-3e forward; ~600–900 LOC). Mints the `K_principal`
  storage type + the vault on-disk DAG-CBOR format + `EncryptedEnvelope::SymmetricAeadXNonce` (m-4) + DAK
  substrate (Argon2id) + `secrecy` (O-1). Everything Layer-B/D depends on this.
- **Canary-ENC-2: Layer-C HPKE-mode-base[MLKEM768-X25519] + Sealed-Sender DEFAULT** (~1,800–2,400 LOC + ~5–8
  wave-days abuse-control per BR-1). Mints the `EncryptToRecipientCodepoint` + `HpkeContext` + the §6.2
  `EncryptedEnvelope` `HpkeBase`/`HpkeMultiBase` variants + **Sealed-Sender `0x6510` default + abuse-control
  delivery-token binding (§3.11)** + Inv-16 mint + libcrux-ml-kem + McMillion hpke. **NQ-C1 (HPKE
  KEM-extensibility) resolved BEFORE this canary.** Layer-D wire pieces reuse this primitive.

**MembershipSet canary (depends on Canary-ENC-2 AND `benten-sync` — B-1):**
- **Canary-MS-PRIMITIVE** (the `benten-membership-set` crate canary). Mints `MembershipSet` + EXACTLY-3
  `MembershipSetKind` + `members_table`/`MemberEntry` + per-Kind constructors + the 0x6610 group AAD field-set (incl.
  `role_assignments_generation`) + Inv-20 (12 clauses) + **Inv-21 (smaller-HLC tie-break, TOTAL via
  Version-Node CID — M-7/M-8) pinned by kani + property tests** + Inv-22 + the **5-value RoleId all-active +
  the pinned ordinal table (Invitee=0…Admin=4) with a golden-vector test + the UCAN ability-templates
  (Invitee=zero-content, Moderator⊊Admin — M-11/M-13)** + `E_ROLE_STALE_AT_VERIFY` + DUAL-CID (extends
  `TwoCidStore`) + Path-A.5. `benten-sync` is a hard dependency.

**Fan-out (parallel, after canaries land; subject to cap-7):**
- Layer-B per-Node AEAD residual (~50 LOC; falls out of Layer-A).
- Layer-D DAK trait + at-rest K_principal/user-DID-key encrypt (~500–800 LOC).
- Layer-D multi-device key-wrap-on-device-link envelope (~400–600 LOC; uses Layer-C HPKE).
- Layer-D remote-permission-call wire protocol + `ExecuteWorkflow` reserve (~500–700 LOC; uses Layer-C HPKE;
  **+ the intra-hour-replay-rejected nonce-cache test — M-1**; **pre-merge security mini-review REQUIRED, owner
  = threat-model lens, clean on all SIX pass-classes — M-12**).
- Layer-D `keyring-core` integration + file-vault fallback + Tauri-shell IPC smoke (~350–550 LOC; minimum-glue).
- **Wave-MS-TRANSPORT: iroh-gossip `GossipTransport` in `benten-sync` (B-1/NQ-D1) + HMAC-blinded topic +
  fork-rotation + OOB rendezvous** (~1,500–2,500 LOC; the largest single uncertainty; closes #61; **convergence
  = MST anti-entropy, gossip = liveness-only — M-10**; R2 seeds the gossip×HLC×Inv-21 test-family).
- Wave-MS-GOVERNANCE-AUDIT (mostly graph-native): GovernanceConfig signed Node + RoleId permission-sets (all-5
  UCAN templates) + audit-event Node version-chain substrate (enforced-WRITE path — m-15 GNC-2) + IVM audit
  view + UCAN read-cap default-issuance (GN-4: substrate at v1-beta; query tooling → Composing).
- **Doc-wave (LAST group per ADDL rule 6):** create `docs/CRYPTO-CODEPOINTS.md` (the §4.0 table) +
  `docs/THREAT-MODEL.md` (the O-6 blast-radius ladder + trust-tier×op matrix + m-5/m-7 scoping) +
  `docs/SECURITY-PROOFS.md` (the AAD per-stanza-LIVE decomposition + the m-4/m-9/m-10 rationales) +
  `docs/future/compute-marketplace.md`; **REGISTER Inv-16..22** (Inv-15 already registered — M-15) in
  `INVARIANT-COVERAGE.md`; land Compromise **#32–#63 + the #31/#62 re-point (BR-2; 3 atomic sites — §5.0)** in
  `SECURITY-POSTURE.md` with `disposition_class`; **V1-FROZEN Rows D-28/D-29 (NEW mints — M-16)** in
  `docs/V1-FROZEN-INTERFACE-DEFERRED.md`; EP-1 ARCHITECTURE roster table + "Rust engine plugin" naming.

**Phase-4-Meta-Core close → tag `phase-4-meta-core-close` → G-CORE-9 v1-interface freeze → external
cryptographer audit window OPENS (audit baseline = this tag).**

### §7.2 Phase-4-Meta-Composing waves (UX-coupled; on the frozen substrate; in parallel with audit)

- Biometric layer (`tauri-plugin-biometric`; macOS Touch ID + Windows Hello + iOS/Android; ~400–600 LOC).
- Device-link UX flow (QR scan + approval dialog; ~400–600 LOC).
- Remote-permission-call UX flow (approval dialog + notification; ~300–500 LOC).
- Stronghold optional `DeviceAuthBackend` backend (opt-in; Tauri-only; ~200–300 LOC).
- Identity-recovery `RecoveryHook` **stub trait** (~50–150 LOC; **the `RecoveryArtifact` codepoint slot was
  reserved at Core — m-14; the TRAIT SHAPE is designed HERE** once the protocol choice — social-recovery vs
  Shamir vs hardware-token — is made at the v1-assessment-window).
- Audit query tooling (richer AuditAccessGradation: threshold-admin M-of-N as UCAN grants; anonymized-aggregate
  as IVM views; GN-4).
- Governance *workflows* (voting/moderation UX for Garden/Grove presets; permission-sets already defined at Core).

**Phase-4-Meta-Composing close → tag `phase-4-meta-close` → v1-assessment-window (between the tags) → (audit
findings closed) → tag `v1-beta`.** (NQ-A2: the assessment-window sits BETWEEN `phase-4-meta-close` and
`v1-beta`, per CLAUDE.md #15 — corrected from R0.1's folding it into Composing exit-criteria.)

### §7.3 The dependency DAG (summary)

```
[Wave-0: V2 = X-Wing-SHA3-256 + BE + EncryptedEnvelope-rename]
        │  (HARD upstream — M-20: MUST merge before ANY canary authors envelope bytes)
        ├──────────────────────────────────────────────────┐
        ▼                                                    ▼
[Canary-ENC-1 Layer-A] ──┬──► [Layer-B residual]   [DAK substrate]  (Argon2id + XChaCha20; NO HPKE dep;
                         │                          ├──► [at-rest encrypt]   parallelizes with Layer-C,
                         │                          ├──► [keyring-core + file-vault]   depends ONLY on Wave-0)
                         │                          └──► [Tauri-shell IPC smoke]
                         └──► [Canary-ENC-2 Layer-C HPKE + Sealed-Sender DEFAULT + abuse-control]
                                   │
       [benten-sync (EXISTING; HLC/Loro/MST/transport)] ──┐ (B-1 upstream dep)
                                   │                       ▼
                                   ├──► [Canary-MS-PRIMITIVE  (benten-membership-set; deps benten-sync)]
                                   │         ├──► [Wave-MS-TRANSPORT  (GossipTransport in benten-sync; MST=convergence)]
                                   │         └──► [Wave-MS-GOVERNANCE-AUDIT  (graph-native)]
                                   └──► [Layer-D wire pieces  (need [DAK substrate] + Canary-ENC-2; use Layer-C HPKE)]
                                              ├──► [multi-device key-wrap]
                                              └──► [remote-permission-call + ExecuteWorkflow reserve + nonce-cache test + 6-class mini-review]
[Doc-wave] (LAST group)
──── phase-4-meta-core-close + G-CORE-9 freeze + external audit OPENS ────
[biometric] [device-link UX] [remote-permission UX] [Stronghold] [RecoveryHook trait] [audit query tooling] [governance workflows]
──── phase-4-meta-close + v1-assessment-window + (audit closed) + v1-beta ────
──── (independent ml-dsa/ml-kem audit per NF-2/C-GM-AUDIT) ──── v1-GM ────
```

**Parallelization exposed (NEW-1).** The `[DAK substrate]` (Argon2id + XChaCha20 vault + at-rest encrypt +
keyring-core/file-vault + Tauri-shell smoke) has NO HPKE dependency — it depends only on Wave-0, so it
proceeds **in parallel with the Layer-C spine (Canary-ENC-2)** rather than gated behind it. Only the
`[Layer-D wire pieces]` (multi-device key-wrap + remote-permission-call, which DO use Layer-C HPKE) sit
downstream of Canary-ENC-2. Collapsing the old single `[Layer-D DAK]` node hid this; the split exposes a
~1.5-day wall-clock parallelization (the DAK-vault track runs alongside the ~1,800–2,400-LOC Layer-C canary).

---

## §8 Cost estimate + honest timeline

### §8.1 LOC + wave-days (O-9: cost units are NON-additive — bracket per spine, not summed)

| Spine | LOC range | wave-days |
|---|---|---|
| Wave-0 V2 migration (X-Wing-SHA3-256 + BE + EncryptedEnvelope rename + full vector regen + interop KAT) | ~120–220 | ~2–4 (BR-3 raises from "~1 wave-day"; construction change + golden-vector regen) |
| Encryption substrate — Core (Layers A/B/C/D wire + DAK + remote-permission + multi-device + **Sealed-Sender default + ~5–8 wd abuse-control per BR-1**) | ~5,500–7,000 | ~58–80 incl. test corpus + audit-deliverables + 30% buffer |
| MembershipSet primitive — Core (primitive + transport + governance/audit graph-native + RBAC) | folded above + iroh-gossip ~1,500–2,500 | ~80–104 central (net −5 to −7 vs M-CONS-v2 via GN/CE-1 simplifications) |
| Phase-4-Meta-Composing (biometric + device-link UX + remote-permission UX + Stronghold + RecoveryHook + audit-query + governance-workflows) | ~1,300–2,400 | ~1–2 days AI-agent wall-clock + ADDL overhead |

**These rows are NOT additive (O-9):** the encryption + MembershipSet spines overlap (the MembershipSet
keying glue IS Layer-C); the wave-day brackets are per-spine, the LOC partially shared. **Net structural
simplification vs prior consolidations:** member side = ZERO new frozen fields; compute side = ZERO reserve;
GN wins remove −8 frozen surfaces (§4.4). **BR-1 ADDS** ~5–8 wave-days (Sealed-Sender abuse-control at Core);
**BR-3 ADDS** ~1–3 wave-days (real X-Wing + full vector regen vs the ~24-LOC re-label).

### §8.2 Honest timeline (headline = ~7–15 weeks; R1-Q-6 CONFIRMED)

**The headline framing is ~7–15 weeks (~2–4 months).** The "~4–5 weeks" figure is the **AI-agent-dispatch
tempo floor ONLY** (bounded by the ~3-person-week external audit); it is NOT the headline (R1-Q-6: invert the
presentation so the honest number leads — avoids the "minimum-viable" / "v1.x-defer" framing traps). The
~7–15-week bracket includes ADDL convergence variance (Phase-4-Foundation R6 took 8 rounds) + Layer-D
cross-platform glue + audit-finding-remediation buffer + the BR-1 abuse-control + BR-3 vector-regen additions.
- **Phase-4-Meta-Core**: ~9–18 days agent-dispatch (canary-first + cap-7; +6–8-round R6 ~3–5 days).
- **External cryptographer audit window**: ~3 person-weeks (the binding wall-clock constraint; baseline =
  `phase-4-meta-core-close`; concurrent with Composing).
- **Phase-4-Meta-Composing**: ~21 days (audit-window-limited; UX overlaps).

### §8.3 Largest uncertainties

- iroh-gossip integration (±1.5–4.5 wave-days inside Wave-MS-TRANSPORT) — de-risked by M-10 (MST is the
  convergence backstop; gossip is liveness-only).
- Sealed-Sender abuse-control engineering (BR-1; ±2–4 wave-days; the delivery-token design depth).
- Real X-Wing combiner + full golden-vector regen (BR-3; ±1–3 wave-days; the interop KAT cross-check).
- Cross-platform Layer-D glue (Linux headless / Tauri-mobile / Verso).
- Cryptographer audit findings (could push v1-beta back ~1–2 weeks; mitigated by the pre-audit
  cryptographer-review + the remote-permission-call 6-class pre-merge mini-review). **Audit-on-frozen-wire
  re-freeze contingency (NQ-A1):** if a high/critical wire-format finding surfaces post-freeze, the
  conservative fallback = reserve-codepoints-implement-minimal + a new tag (NOT a silent post-tag wire-break).

---

## §9 Exit criteria + the v1-beta-tag gate

### §9.1 Phase-4-Meta-Core exit criteria (→ tag `phase-4-meta-core-close` → G-CORE-9 freeze)

1. The §6.2 `EncryptedEnvelope` codepoint family (Layers A/B/C/D, incl. **Sealed-Sender `0x6510` default**)
   implemented + the full bidirectional swap matrix conformance-tested (hybrid-default + classical/no-enc/non-PQ
   downgrades + the NF-1 PQ⊕PQ non-default arm) per CLAUDE.md #5 / G-CORE-3c.
2. **Wave-0 landed:** Layer-A K_principal vault (real, not stub; XChaCha20 — m-4) + Layer-B per-Node AEAD on
   real K_principal + **real X-Wing SHA3-256 corrective (BR-3) + BE migration + EncryptedEnvelope rename** + the
   interop KAT + the "no `to_le_bytes` on wire/AAD" conformance test all green.
3. Layer-C encrypt-to-recipient (single + multi-stanza + Sealed-Sender default + abuse-control delivery-tokens)
   + Inv-16 + DUAL-CID landed.
4. Layer-D DAK + at-rest K_principal/user-DID-key encrypt + remote-permission-call wire + `ExecuteWorkflow`
   reserve + multi-device key-wrap wire landed; **the remote-permission-call pre-merge security mini-review
   PASSED, clean on all SIX pass-classes (M-12: replay / device-key-revocation / clock-skew / confused-deputy /
   UI-deception / audit-Node-binding — grant rejected if `audit_node_cid` absent/unresolvable)**; the
   **intra-hour-replay-rejected-by-nonce-cache test (M-1)** green.
5. The MembershipSet primitive (EXACTLY-3 Kind + members_table + **5-value RoleId all-active with the pinned
   ordinal golden-vector test + Invitee-derives-ZERO-content + Moderator⊊Admin UCAN templates — M-11/M-13** +
   0x6610 group AAD field-set + **Inv-20 (12 clauses) + Inv-21 (smaller-HLC, TOTAL-via-Version-Node-CID, kani-proven —
   M-7/M-8) + Inv-22** + Path-A.5 + DUAL-CID) + iroh-gossip transport (HMAC-blinded topic + fork-rotation + OOB;
   convergence = MST) landed.
6. Governance/audit/economics graph-native substrate (GovernanceConfig signed Node + audit-event version-chain
   via the **enforced-WRITE path — m-15 GNC-2** + IVM view + UCAN read-cap on the `audit:<set_id>:*`
   RestrictedScope) landed; ZERO economic freeze hook confirmed; ZERO member nature field confirmed.
7. **REGISTER Inv-16..22** in `INVARIANT-COVERAGE.md` (**Inv-15 already registered — do NOT re-register; do NOT
   touch the "15 invariants" header — M-15**); Compromise **#32–#63 + the #31/#62 re-point (BR-2; 3 atomic
   sites)** landed in `SECURITY-POSTURE.md` with `disposition_class`; `CRYPTO-CODEPOINTS.md` (the §4.0 table) +
   `THREAT-MODEL.md` + `SECURITY-PROOFS.md` + `compute-marketplace.md` created; **V1-FROZEN Rows D-28/D-29 (NEW
   mints — M-16)** in `docs/V1-FROZEN-INTERFACE-DEFERRED.md`; EP-1 roster + naming landed.
8. ADDL R6 phase-close convergence council returns 0 substantive (BLOCKER/MAJOR) findings (full-council
   per-round per Q5; iterate-to-convergence).
9. G-CORE-9 v1-interface freeze applied (the frozen surface = §4's FREEZE rows; the §4.0 codepoint table is the
   authoritative wire registry; the maximum graph surface confirmed additively-extensible).

### §9.2 Phase-4-Meta-Composing exit criteria (→ tag `phase-4-meta-close`)

10. Biometric + device-link UX + remote-permission-call UX + Stronghold optional backend + `RecoveryHook` stub
    trait (the `RecoveryArtifact` codepoint reserved at Core — m-14; trait SHAPE designed here) + audit query
    tooling + governance workflows landed on the frozen substrate (NO wire-format break).

### §9.3 The v1-assessment-window + the v1-beta-tag gate (NQ-A2 — window BETWEEN the tags)

11. **v1-assessment-window** (per CLAUDE.md #15; sits BETWEEN `phase-4-meta-close` and `v1-beta`): end-to-end
    platform use-test; identity-recovery protocol choice surfaced (feeds the m-14 `RecoveryHook` trait shape);
    missing_docs sweep; small arch cleanups.
12. **Pre-tag external-cryptographer audit** (baseline = `phase-4-meta-core-close`; ~3 person-weeks; PQShield /
    Cure53 / NCC Group — NOT Cryspen): findings triaged per HARD-RULE-12; no unresolved high/critical on the
    hybrid trust path (or remediated + re-verified). **Audit-deliverable lines (M-6 / §3.3):** the HPKE-mode-base
    IND-CCA2-under-adversarial-recipient-seed analysis (connects #45 MAL-BIND + #59 KEM-key-confirmation) is an
    explicit audit scope item. Then tag `v1-beta`.
13. **`v1-GM` gate (separate, post-v1-beta):** the independent `ml-dsa` + `ml-kem` audit of the pinned versions
    (NF-2 / C-GM-AUDIT; pinned==audited; Ben sign-off; **closes Compromise #30**).


---

## §10 Risks + open questions to seed R2 / R3 (HARD RULE 12 — every item resolved-in-plan OR named-NOW)

### §10.1 Resolved-in-plan (the R1 R1-Q-1..9 — no longer open)

R1-Q-1..R1-Q-9 are **CLOSED** by the R1 council (7 CONFIRM + R1-Q-2 ratified by BR-1 + R1-Q-4 refined by M-1):
KEEP-both-layers (Q1); Sealed-Sender DEFAULT (Q2/BR-1); §14.1 unified-envelope correctly NAMED-deferred under
C-1 NO-GO (Q3); DAG-CBOR + 1-hr-bucket both load-bearing, replay rides the nonce-cache (Q4/M-1); audit
substrate at Core / tooling at Composing (Q5); ~7–15wk headline (Q6); Garden/Grove signed-Node not codepoints
(Q7); mint #56/#59/#61 (Q8); RoleId permission-sets at Core as UCAN templates (Q9). All folded into §2–§9.

### §10.2 Risks (for R2/R5 attention)

| Risk | Severity | Mitigation |
|---|---|---|
| Scope-creep past `phase-4-meta-core-close` (freeze) gate | HIGH | The §7 wave-set is the contract; the frozen-crypto rule is the membership test |
| Remote-permission-call wire has a lurking attack class (highest single-piece security risk) | HIGH | The 6-class pre-merge security mini-review (M-12); conservative op set |
| **`ExecuteWorkflow` rented-executor exfiltrates decrypted plaintext (SANDBOX-escape re-asked at rented-compute boundary — M-3)** | HIGH | AAD-bound `executor_did`/`max_decrypt_count`/`result_recipient_pubkey` frozen at v1-beta (SUFFICIENT to express the no-egress constraint); runtime enforcement post-v1-beta (NQ-T3) |
| **HPKE-mode-base IND-CCA2 under adversarially-chosen-recipient-seed unproven (M-6)** | MED-HIGH | §9.3 audit-deliverable line; connects #45 + #59; do not drop |
| Multi-device key-wrap exfiltrates K_principal to wrong device | HIGH | `provisioning_session_id` binding + user device-fingerprint confirmation UX |
| Sealed-Sender abuse-control under-delivers (BR-1) | MED | recipient-issued delivery-tokens + conservative default rate-limits (#63); pre-merge review |
| Cross-platform Layer-D glue drifts | MED | minimum-glue at Core; per-platform CI matrix; the rest at Composing |
| Argon2id params age poorly | LOW-MED | params in vault header (user-upgradeable) + HKDF-info codepoint slot |
| **Inv-21 totality fails the kani proof (M-8)** | MED | TOTAL order = `(created_at_hlc ASC, fork_event_version_node_cid ASC)`; NQ-D2 pins the proof shape |
| **iroh-gossip convergence under out-of-order/dup delivery (M-10)** | MED | MST anti-entropy is the convergence backstop; gossip = liveness-only; R2 gossip×HLC×Inv-21 test-family |

### §10.3 New R2 questions (wire-format / freeze-gating — carried-NOW per HARD RULE 12)

- **NQ-W1 (one V2 bump).** Ratify ONE `ENVELOPE_FORMAT_VERSION_V2` covering BR-3 X-Wing + BE + the
  `AeadEnvelope`→`EncryptedEnvelope` lift (NOT three bumps). [§4.1.MIGRATION]
- **NQ-W2 (codepoint registry authority + scanner).** Is `CRYPTO-CODEPOINTS.md` (the §4.0 table) authored at R2
  (prerequisite) rather than the trailing doc-wave? What CI scanner enforces intra-`0x6100..0x6FFF`
  non-collision + IANA-disjointness at freeze (Inv-18)?
- **NQ-W3 (frozen op-surface enumerated).** Confirm the "24 ops" figure stays RETRACTED and the frozen op
  surface is exactly the 12 primitives; `audit_log_query` is graph-native (M-17). Produce the conformance
  enumeration.
- **NQ-W4 (`members_table` canonical bytes).** Pin the length-injective (U3) byte encoding of the AAD-bound
  `BTreeMap<Did, MemberEntry>` (field order + `Option<SigPubKey>` presence + `Hlc` encoding).
- **NQ-W5 (`RecoveryHook` artifact-only at Core).** Confirm only the `RecoveryArtifact` codepoint slot freezes
  at Core; the trait shape defers to Composing (m-14).

### §10.4 New R2 questions (distributed-systems — gate the kani proof)

- **NQ-D1 (gossip placement + convergence model).** `GossipTransport` in `benten-sync` (lean) vs
  `benten-membership-set`? Convergence backed by MST anti-entropy (gossip = liveness-only — M-10)?
- **NQ-D2 (total Inv-21 tie-break).** Precise `(created_at_hlc, fork_event_version_node_cid)` total order +
  the exact `created_at_hlc`/`admitted_at_hlc`/version-chain-HLC relationship for the kani proof (M-7/M-8).
- **NQ-D3 (`KSetAcquisitionPath` frozen fields).** Confirm `{ target_set_id, hop_path (path-carried
  cycle-detect, len ≤ 4), acquisition_proof_cid }` makes depth-4+cycle-detect offline-decidable (M-9).
- **NQ-D4 (`generation_summary` definition).** Set-generation (not per-member-vector) for steady-state
  rendezvous + how losing-fork members re-converge onto the winning fork's NEW topic (O-5/m-11).

### §10.5 New R2 questions (threat-model — gate the §6.7 mini-review)

- **NQ-T1 (audit-Node encryption + replication).** Is the `PermissionGrant` audit-Node encrypted + replicated
  to all of the user's devices so a malicious device can't grant-and-hide?
- **NQ-T2 (`valid_until` clock vs the 1-hr bucket on the remote-permission surface) — RATIFIED (Ben 2026-06-02).**
  The 1-hour metadata bucket (round-down) and the `valid_until` enforcement clock are **orthogonal,
  separately-encoded fields**. **Rule:** `valid_until` is encoded at **full 1-second granularity** and
  enforced **STRICTLY** — `present > valid_until → reject`, with **NO grace/skew window**; the coarse 1-hour
  bucket is **never consulted for expiry**. A coarsened bucket therefore cannot widen the coercion/replay
  window (it does not touch the enforcement clock). (Note: the "≥1-year grace" mentioned at §3.3 is the
  recipient-side **drop key-retention** window — Compromise #62 — NOT a `valid_until` grace.) R1-Q-4 closed
  at Layer-D.
- **NQ-T3 (`ExecuteWorkflow` no-egress enforcement) — RATIFIED (Ben 2026-06-02).** The frozen 3-field
  `ExecuteWorkflow` AAD tuple `(executor_did, max_decrypt_count, result_recipient_pubkey)` is **SUFFICIENT to
  express** the no-egress / bounded-decrypt constraint. **Rule:** the AAD-bound 3-field constraint is frozen
  at v1-beta; **runtime enforcement** (that the rented executor cannot exfiltrate plaintext beyond
  `result_recipient_pubkey` via EMIT/WRITE) **stays post-v1-beta — it is NOT freeze-gating** (M-3). Freezing
  the AAD scope now is what makes the post-v1-beta enforcement non-wire-breaking.
- **NQ-T4 (nonce-cache spec) — RATIFIED (Ben 2026-06-02).** **Rule:** the nonce-cache is **`jti`-keyed**,
  **durable** (survives engine restart — persisted, not RAM-only), and retention is **≥ the full 1-hr bucket
  window**. **Scope:** **per-device-durable is GUARANTEED**; **user-global is best-effort-eventual-via-sync
  (NOT synchronous)** — a nonce consumed on device B is rejected on device C only after sync propagates the
  cache entry. The pre-sync cross-device replay window MUST be DISCLOSED as a named Compromise: **mint
  Compromise #<next>** — "best-effort-eventual cross-device nonce-rejection window" (the window between a
  nonce being consumed on one device and the rejection propagating to the user's other devices via sync); see
  the SECURITY-POSTURE doc-cascade (M-1).

### §10.6 New R2 questions (crypto / standards-interop)

- **NQ-C1 (HPKE KEM-extensibility).** Does McMillion `hpke` accept a custom/PQ KEM (X25519MLKEM768 plugs into a
  real RFC-9180 context) or does Benten supply the KEM + reuse only the HPKE KDF/AEAD/key-schedule? Resolve
  BEFORE Canary-ENC-2 (determines whether "HPKE-RFC-9180" is byte-accurate or aspirational).
- **NQ-C2 (libcrux ↔ RustCrypto ml-kem FIPS-203 KAT).** Cross-impl serialization KAT so golden vectors survive
  the swap (m-2).
- **NQ-C3 (cross-ecosystem conformance vectors).** Should v1-beta verify a LAMPS Composite ML-DSA sig from
  BouncyCastle/OpenSSL-3.5/OpenPGP-PQC against Benten's verifier (the "ecosystem interop" justification for
  LAMPS is asserted; the swap-matrix tests internal round-trips only)?
- **NQ-C4 (did:key hybrid-pubkey multicodec).** What multicodec prefix do the PQ-hybrid sig/KEM pubkeys use in
  `did:key`? If no registered multiformats value, what private-value-with-fallback is reserved at G-CORE-9?
- **NQ-C5 (jitter ↔ clock-skew) — RATIFIED (Ben 2026-06-02).** **Rule:** the epoch bucket =
  `(raw_unix_secs / 3600) * 3600` — **round-DOWN, NO jitter**, deterministic (`bucket % 3600 == 0`). This
  avoids the ≤2×jitter+skew window-widening (m-9): with no jitter there is no jitter↔skew interaction to
  widen. The **nonce-cache window does NOT need widening** — bucket and nonce-cache are **orthogonal**
  (bucket = metadata privacy; nonce-cache = replay defense). They are tuned independently.

### §10.7 New questions (audit-gate sequencing)

- **NQ-A1 (audit-on-frozen-wire → re-freeze contingency).** A high/critical wire-format finding post-freeze =
  reserve-codepoints-implement-minimal + a new tag (NOT a silent post-tag wire-break). Name the conservative
  fallback explicitly (§8.3).
- **NQ-A2 (assessment-window placement).** Confirmed: tag sequence = `phase-4-meta-close` → assessment-window →
  `v1-beta` (§9.3; corrected from R0.1's folding it into Composing exit-criteria).

### §10.8 R3 test-landscape seeds (NAMED-NOW per HARD RULE 12; captured by R3, not resolved here)

- `plaintext_cid_local` NEVER-serialized kani/property test (O-7).
- The intra-hour-replay-rejected-by-nonce-cache test (M-1).
- The Inv-21 kani convergence proof (totality + smaller-HLC; M-8).
- The RoleId ordinal golden-vector test + Invitee-zero-content + Moderator⊊Admin UCAN-template tests (M-11/M-13).
- The "no `to_le_bytes` on wire/AAD" conformance test + the X-Wing interop KAT + the FIPS-203 cross-impl KAT
  (M-19/BR-3/m-2).
- The gossip×HLC×Inv-21 convergence-under-out-of-order test-family (M-10).

---

## §11 Self-assessment + convergence-readiness

| Finding-class | Conf | Reasoning |
|---|---|---|
| §0.3 ground-truth RE-RUN (the 2 inversions corrected) | **HIGH** | every claim `grep`/`sed`-verified this pass against HEAD `2172cb6d`; #31/#30 occupancy + Inv-15-is-registered + D-27-max + AeadEnvelope-is-shipped + HKDF-combiner all directly cited |
| Ben's 3 rulings applied (BR-1/BR-2/BR-3) | **HIGH** | Sealed-Sender DEFAULT @ `0x6510` + #31-LAMPS/#62-revocation re-point (3 atomic sites) + real X-Wing SHA3-256 — all wired through §2.0/§3/§4/§5/§7/§8 |
| Canonical codepoint table (§4.0) | **HIGH** on the collision fix (9-eyes wins `0x6380/0x6390`; MembershipSet relocated to `0x6600`; Sealed-Sender = `0x6510`); **MED** on the exact MembershipSet sub-band choice (`0x6600` is a free band; R2/Ben may re-place) | the 9-eyes is the authoritative source; the relocation is collision-free + asserted; the specific `0x6600` is a defensible free-band pick |
| B-1 `benten-sync` dependency + gossip placement | **HIGH** on the dep (sync owns HLC/Loro/MST/transport — ground-truthed); **MED** on gossip landing in benten-sync (lean, carried as NQ-D1) | the dependency is structural + verified; the physical-landing is a clean R2 question |
| Inv-21 asymmetry + totality (M-7/M-8) | **HIGH** | the in-tree LARGER-HLC LWW (`crdt.rs:535`) is ground-truthed; the smaller-HLC fork rule is justified; totality via Version-Node CID is correct |
| M-15 Inv-15 DISAGREE-WITH-EXPLANATION | **HIGH** | Inv-15 IS registered in-tree (header "15 invariants"; `:49` row) — the triage premise was stale; recorded as a HARD-RULE-12(c) disagreement, not a silent skip |
| Cost (BR-1 +5–8wd abuse-control; BR-3 +1–3wd vector-regen; ~7–15wk headline) | **MED** | the additions are well-grounded; iroh-gossip + audit-window dominate the bracket |

**Convergence-readiness self-assessment.** Every B-1..B-3, M-1..M-20, m-1..m-15, O-1..O-9 from the triage has
a disposition in §13. The 3 BLOCKERs are closed (B-1 sync dep + DAG; B-2 #31 re-derived per BR-2; B-3 the §4.0
canonical table). All 20 MAJORs are applied except where I DISAGREE-WITH-EXPLANATION (M-15 — Inv-15 IS in-tree;
the fix is "do NOT re-register Inv-15", which I applied). The §5 R2/R3 questions are carried as named NQ-* seeds
(HARD RULE 12). **I expect R1.2 to converge (0 BLOCKER/MAJOR)** provided the R1.2 reviewers ground-truth Inv-15
themselves (the one place this revision DISAGREES with the triage rather than complies).

**The 3 items I could NOT fully close in-plan (correctly deferred, not skipped):**
1. **NQ-C1 (HPKE KEM-extensibility byte-accuracy)** — whether McMillion `hpke` admits the X25519MLKEM768 KEM
   into a real RFC-9180 context, or whether "HPKE-RFC-9180" is aspirational, is a code-investigation that must
   precede Canary-ENC-2 (named-NOW as a freeze-gating R2 question, not invented).
2. **The exact MembershipSet sub-band integers within `0x6600..0x66FF`** — `0x6600`/`0x6610`/`0x6620` are a
   collision-free pick; the final sub-allocation is a `CRYPTO-CODEPOINTS.md` authoring decision (NQ-W2) that
   needs Ben's bless on the table (reproduced in the return message).
3. **The Inv-21 kani proof shape** (NQ-D2) — the totality argument is stated; the formal proof construction is
   R3/R5 work.

---

## §12 Citations

**Primary inputs (frozen SHAs; all `git show`-verified at write):** R0.1 `6755ea41`; R1 triage `50115446`;
M-CONS-FINAL `a99dd0c7`; GN-1 `859fa51e`; GN-2 `61f24553`; EP-1 `7520ae4e`; 9-eyes consolidated registry
`fbdfeb16` (**authoritative codepoint source — §0.4**); e2r-ffull-scope-review `220b5aae`.

**Tree-pinned (HEAD `2172cb6d`; RE-VERIFIED this pass):** `crates/benten-crypto-suite/src/structural_kdf.rs:8–9`
(K(N) chain); `crates/benten-crypto-suite/src/aead.rs:145,165,244,277` (`AeadEnvelope` shipped + LE codepoint +
LE AAD index fields); `crates/benten-crypto-suite/src/cipher_suite.rs:37,65,78` (HKDF-SHA256 combiner mislabeled
"X-Wing"; `sha3` already a dep); `crates/benten-crypto-suite/src/codepoint.rs:49,199,205,214,228` (sig + cipher
codepoints; `0x6400` LIVE; `0x647c` reserved-typed-reject); `crates/benten-caps/src/scope.rs:44,46,51` (Scope
2-arm + 3rd-arm HALT-AND-SURFACE); `crates/benten-core/src/subgraph.rs:71–93` (PrimitiveKind 12);
`crates/benten-sync/src/crdt.rs:535` (LARGER-HLC LWW); `crates/benten-sync/src/{mst.rs,transport_trait.rs:85,
two_cid_store.rs}` (MST + Transport-no-pubsub + DUAL-CID precedent); `crates/benten-graph/src/store.rs:468`
(ChangeEvent attribution triple = `Option`); `crates/benten-drop/`; `docs/INVARIANT-COVERAGE.md:1,14,49,320`
(Inv-1..15 in-tree; Inv-15 REGISTERED partially-enforced; Inv-14 derives Plugin); `docs/SECURITY-POSTURE.md:30,
91,1798,2433,2712` (#30 unaudited-PQ + #31=revocation-reach summary occupant + #25 nonce-cache CLOSED + LAMPS
orphaned prose + revocation-reach detail); `docs/V1-FROZEN-INTERFACE-DEFERRED.md` (max Row D-27; the global
D-NN namespace); `docs/PLUGIN-MANIFEST.md:68` (InstallRecord signed-config-Node precedent);
`docs/CRYPTO-CODEPOINTS.md` + `docs/future/compute-marketplace.md` (created at R0 doc-wave); `system:Principal`
+ `create_principal` (live).

**CLAUDE.md baked-in + MEMORY.md disciplines:** #1 (12-primitive irreducibility — preserved); #2 (IVM
enum-dispatch not a backend trait — m-15 GNC-4); #3 (code-as-graph — GN strengthens); #5 (crypto-agility —
codepoint-dispatch + typed-reject; never fork; the X-Wing corrective IS the #5 example done right per BR-3); #7
(sealed CapabilityPolicy + DeviceAuthBackend); #8 (version-chains — audit + membership-event DAG); #15
(v1-beta freeze — Core + Composing both pre-tag; assessment-window between the tags); #16 (SANDBOX escape
hatch — re-asked at the ExecuteWorkflow rented-compute boundary); #17 (three deployment shapes); #18 (Principal
one-per-DID; authority vs confidentiality halves); #19 (Rust engine plugins — EP-1). `feedback_no_defer_HARD_RULE`
(every deferral NAMED-NOW); `feedback_extra_reflection_pass_for_elegant_permanent_shape`;
`feedback_review_finding_ground_truth_verify` (the §0.3 re-run + the M-15 DISAGREE);
`feedback_iterate_critical_reviews_to_convergence` (R1→R1.2); `feedback_canary_first_parallel_implementation`;
`feedback_surface_arch_decisions_under_auth` (the 3 Ben rulings); `feedback_addl_pipeline_full_observance`;
`feedback_phase_ordering_precision`; `feedback_inverted_prework_post_campaign_phase`; `feedback_plain_english_surfaces`.

---

**End of F-full R0.7 plan-doc.** Supersedes M-CONS-FINAL + the 9-eyes registry + e2r-ffull as the canonical
F-full scope (9-eyes wins codepoint collisions); R0.6 adds the 7 Ben-ratified Sealed-Sender AAD freeze
decisions (the v1-beta envelope wire); R0.7 adds the `blake3::keyed_hash` construction-name precision + the
`0x6520` group-AAD blinding (audience_set_commitment + stanza_count + self-describing body_cid). Hand to the
R1.2 re-review per the iterate-to-convergence discipline.

---

## §13 R1 fix-pass changelog (every triage finding → disposition)

**Legend:** APPLIED = R0 edited per the triage disposition; DISAGREE = HARD-RULE-12(c) with explanation;
CARRIED-R2/R3 = named NQ-* seed per HARD RULE 12(b). **Base = R0.1 `6755ea41` + triage `50115446` + Ben rulings
BR-1/BR-2/BR-3.**

### BLOCKERS

| # | Finding | Disposition | Where |
|---|---|---|---|
| **B-1** | `benten-sync` missing from §6.1 deps + §7.3 DAG (owns HLC/Loro/MST/iroh) | **APPLIED** — added `benten-sync` as a load-bearing upstream dep; stated direction (sync upstream); iroh-gossip `GossipTransport` lands in `benten-sync` (NQ-D1) | §6.1, §6.2, §7.1, §7.3, §10.4 |
| **B-2** | In-tree #31 INVERTED (it's revocation-reach, NOT LAMPS; LAMPS prose orphaned; #30 occupied; 3 sites) | **APPLIED per BR-2** — §0.3 re-run confirms inversion; LAMPS keeps #31 (gains the summary row) + revocation→#62 (3 atomic re-point sites) + #30 stays | §0.3, §5.0, §5.2 |
| **B-3** | `0x6380/0x6390` collision between source docs; Sealed-Sender at 2 values | **APPLIED** — ONE authoritative §4.0 table; 9-eyes wins (MLS-App/MLS-Welcome); MembershipSetEncryption → `0x6600`; Sealed-Sender = `0x6510` (BR-1) | §0.4, §4.0 |

### MAJORS

| # | Finding | Disposition | Where |
|---|---|---|---|
| **M-1** | R1-Q-4 decoupling partly false; replay rides the nonce-cache; promote nonce-cache to v1-beta requirement | **APPLIED** — honest decoupling map; nonce-cache = NAMED load-bearing requirement (scope/retention/durability/multi-device) + intra-hour-replay test; 1-hr bucket Layer-D-only | §3.4, §3.10, §10.5 NQ-T4 |
| **M-2** | Sealed-Sender DEFAULT vs additive-slot self-contradiction + wrong cross-ref (R1-Q-7→R1-Q-2) | **APPLIED per BR-1** — Sealed-Sender DEFAULT @ Core @ `0x6510`; §4.1 flipped to FREEZE+SHIP; cross-ref corrected to R1-Q-2; #59/#63 abuse-control into Core | §2.0 BR-1, §2.3 Q5, §3.3, §3.11, §4.0, §4.1 |
| **M-3** | `ExecuteWorkflow` absent from §3.4 + §4.1 | **APPLIED** — added as CODEPOINT-RESERVE (variant slot + AAD binding frozen; runtime enforcement post-v1-beta) + executor-trust threat entry | §3.4, §4.1, §10.2, §10.5 NQ-T3 |
| **M-4** | X-Wing corrective ambiguous (re-label vs real construction) | **APPLIED per BR-3** — REAL X-Wing SHA3-256 construction; ground-truthed (HKDF-SHA256 in-tree); LOC revised up to ~120–220; full vector regen | §0.3, §2.1 C-6, §3.2, §7.1 Wave-0, §8.1 |
| **M-5** | Encryption Compromise table omits in-tree #30 | **APPLIED** — #30 added (KEEP, do-not-move); #32 = Decap-refinement-of-#30; #30↔#32↔C-GM-AUDIT chain explicit | §0.3, §5.2 |
| **M-6** | HPKE IND-CCA2-under-adversarial-recipient-seed gap surfaced nowhere | **APPLIED** — §3.3 risk note + §10.2 risk row + §9.3 audit-deliverable line; connects #45 + #59 | §3.3, §9.3, §10.2 |
| **M-7** | Inv-21 smaller-HLC is OPPOSITE the in-tree LARGER-HLC LWW; created_at vs admitted_at undefined | **APPLIED** — asymmetry stated + justified (oldest-anchor-wins for forks vs LWW for properties; ground-truthed `crdt.rs:535`); `created_at_hlc` vs `admitted_at_hlc` defined; "byte-equivalent" framing dropped | §0.3, §3.5.HLC, §3.8.Inv-21, §5.1 |
| **M-8** | Inv-21 tie-break non-TOTAL (smaller MembershipSetId can't disambiguate same-anchor forks) | **APPLIED** — terminal discriminator = forking-event Version-Node CID (always distinct); total order `(created_at_hlc, version_node_cid)` | §3.8.Inv-21, §5.1, §10.4 NQ-D2 |
| **M-9** | `KSetAcquisitionPath` traversal semantics unpinned at freeze | **APPLIED** — frozen fields `{target_set_id, hop_path (path-carried cycle-detect, len ≤ 4), acquisition_proof_cid}`; offline-decidable; path-carried not receiver-local | §3.6.C, §4.2, §10.4 NQ-D3 |
| **M-10** | iroh-gossip convergence-correctness under-weighted; gossip net-new | **APPLIED** — Transport-no-pubsub ground-truthed; convergence = MST anti-entropy, gossip = liveness-only; gossip×HLC×Inv-21 R2 test-family seeded | §0.3, §3.9, §6.1, §7.1, §10.4 NQ-D1 |
| **M-11** | R1-Q-9 under-scoped; Invitee-zero-content + Moderator⊊Admin load-bearing | **APPLIED** — pinned UCAN ability-set table; Invitee=ZERO-content + Moderator⊊Admin hard constraints in §9.1 exit-5 | §3.6.B, §9.1 |
| **M-12** | §6.7 security mini-review has no owner/pass-criteria/audit-Node sub-question | **APPLIED** — owner = threat-model lens; SIX pass-classes (incl. audit-Node-binding: grant rejected if audit_node_cid absent); §9.1 item-4 "clean on all six" | §3.4, §9.1 |
| **M-13** | RoleId ordinal silently changed M-CONS-FINAL→R0 + AAD-keying-bound | **APPLIED** — explicit "ordinal supersedes M-CONS-FINAL" note; pinned table (Invitee=0…Admin=4) at canary + golden-vector test; Inv-20 clause-j "all-5-active" correction | §3.6.B, §4.2, §5.1 |
| **M-14** | Cross-source wire-inconsistency: does DropToRecipient carry sealed_at/valid_until? | **APPLIED** — stated explicitly: DropToRecipient carries NEITHER (forever-valid per #62; freshness = recipient-key-generation + nonce-cache); 1-hr bucket Layer-D-ONLY; L6 drop-timestamp finding closed-by-exclusion | §3.10, §4.1 |
| **M-15** | Inv-15 claimed in-tree but allegedly absent; reclassify as design-mint-to-register | **DISAGREE-WITH-EXPLANATION** — ground-truth: **Inv-15 IS in-tree** (`INVARIANT-COVERAGE.md:1` "15 invariants"; `:14` REGISTERED; `:49` row). The triage premise ("not in the doc; header 14 of 14") is STALE. The CORRECT fix: do NOT re-register Inv-15 + do NOT touch the header; REGISTER only Inv-16..22. §5.0 baseline reads "Inv-1..15 in-tree; Inv-16..22 design-mints" (already correct in R0.1 §5.0). | §0.3, §5.0, §5.1, §7.1 doc-wave, §9.1 item-7 |
| **M-16** | §4.3/§9.1 cite D-28/D-29 as extant; in-tree max = D-27 | **APPLIED** — D-28/D-29 marked "(NEW — minted at doc-wave)"; single destination doc named (`docs/V1-FROZEN-INTERFACE-DEFERRED.md`, the global D-NN namespace) | §4.3, §7.1 doc-wave, §9.1 item-7 |
| **M-17** | `audit_log_query` FREEZE while audit log is graph-native; "24 ops" unsourced | **APPLIED** — `audit_log_query` reclassified GRAPH-NATIVE (scoped-READ, not a frozen op); "24 ops" RETRACTED; frozen op-surface = the 12 primitives; the `audit:<set_id>:*` scope = a `RestrictedScope` arm | §1.5, §3.8, §4.2 |
| **M-18** | `AeadEnvelope`→`EncryptedEnvelope` is a rename+lift of a SHIPPED type, not a fresh mint | **APPLIED** — §4.1.MIGRATION states the rename+structural-lift+typed-AAD as ONE V1→V2 migration co-scheduled with Wave-0; ground-truthed (AeadEnvelope shipped flat, untyped AAD) | §0.3, §4.1, §4.1.MIGRATION, §6.2 |
| **M-19** | BE-migration blast radius understated as "~1 wave-day" | **APPLIED** — re-scoped: ALL multiformats-framed integer wire/AAD fields → BE (codepoint + AAD index/count fields `:244,277` + aead_wrap + platform-foundation), single audit pass + full golden-vector regen + conformance test asserting no `to_le_bytes` survives | §2.3 Q2, §3.2, §4.1, §4.1.MIGRATION, §7.1 |
| **M-20** | Wave-0 not a structural upstream of canaries in the DAG | **APPLIED** — hard DAG edge `[Wave-0] ──► [every canary]`; all new codepoints authored V2+BE from first commit | §4.1.MIGRATION, §7.1, §7.3 |

### MINORS

| # | Finding | Disposition | Where |
|---|---|---|---|
| **m-1** | X-Wing info-tag ASCII; Q2 LE→BE must state info-tag NOT endianness-affected; 3 encode-sites | **APPLIED** — 3 `0x647a` encode-sites enumerated; info-tag string is ASCII / not endianness-affected stated | §3.2 |
| **m-2** | libcrux swap + LE→BE need a FIPS-203 KAT cross-check as a Wave-0 exit gate | **APPLIED** — Wave-0 exit gate + `check-secret-independence` CI | §3.2, §7.1, §10.6 NQ-C2 |
| **m-3** | Inv-17 wording tension with reserved `0x647c` | **APPLIED** — Inv-17 sharpened: no pure-PQ codepoint LIVE/selectable; reserved-named-typed-rejected arms permitted + audit-gated | §2.1 C-8, §5.1 |
| **m-4** | Layer-A 12-byte ChaCha20 nonce vs U32 XChaCha20 (24-byte) for reseal-heavy sites | **APPLIED** — vault uses XChaCha20-Poly1305 (24-byte); both `SymmetricAead`/`SymmetricAeadXNonce` ship | §3.1, §4.1 |
| **m-5** | Coerced-approving-device not in Compromise table | **APPLIED** — #33 disclosure-scope extended to the Layer-D approval-coercion vector | §5.2 #33 |
| **m-6** | #60 role-transition not connected to ship-all-5 + remote-permission compositions | **APPLIED** — composition note + tight-`exp` default mandate (cross-link #52) | §3.4, §5.2 #60 |
| **m-7** | Inv-20 clause-d "per-recipient unlinkability = network-observer-only" not surfaced | **APPLIED** — network-observer-only-vs-insider scoping carried into §3.8 + THREAT-MODEL skeleton | §3.8, §5.1, §5.2 #58 |
| **m-8** | R0 never states replay/freshness rides on counters+nonce-cache (overlaps M-1) | **APPLIED** (with M-1) — §3.10 cites Compromise #25 shipped nonce-cache substrate | §3.10 |
| **m-9** | Jitter-vs-round-down sub-fork pre-decided; jitter↔skew can double-widen | **APPLIED** — explicit sub-question; default round-down-no-jitter; jitter↔skew interaction noted | §3.10, §10.6 NQ-C5 |
| **m-10** | 1-hour granularity rationale unrecorded | **APPLIED** — 33→14 bits/year rationale recorded → SECURITY-PROOFS.md | §3.10, §7.1 doc-wave |
| **m-11** | D6 topic-rotation freshness source + OOB residue implicit | **APPLIED** — rotation rides generation-counters; OOB-bootstrap cross-link #43/#62 | §3.9 |
| **m-12** | `0x6400` classical-downgrade arm absent from §4.1 inventory | **APPLIED** — `0x6400` row added (LIVE in-tree) with lifecycle state + BE/golden-vector coverage | §0.3, §4.0 |
| **m-13** | MembershipSet codepoints scattered; no single table; non-collision unasserted | **APPLIED** — single §4.0 allocation table (canonical home `CRYPTO-CODEPOINTS.md`); intra-band non-collision asserted; IANA-disjoint named | §0.4, §4.0 |
| **m-14** | `RecoveryHook` freezes a trait shape ahead of its protocol | **APPLIED** — Core reserves ONLY the `RecoveryArtifact` codepoint slot; trait shape defers to Composing | §4.1, §7.2, §9.2, §10.3 NQ-W5 |
| **m-15** | Graph-native precision cluster (GNC-2..7) | **APPLIED** — 5 precision edits: attribution-triple-is-Option-enforced-path (GNC-2); IVM-view = Rust-plugin-LOC not "free" (GNC-3); EP-1 seam roster + IVM-enum-not-trait (GNC-4); AAD opaque-bytes-no-reverse-dep (GNC-5); MemberRef Kind-determined not nature (GNC-7); `audit:<set_id>:*` = RestrictedScope (GNC-1) | §2.6, §2.7, §3.7, §3.8, §3.10, §6.1, §6.3 |

### OBS

| # | Finding | Disposition | Where |
|---|---|---|---|
| **O-1** | `secrecy` new Layer-A dep | **APPLIED** (BELONGS-NAMED-NOW) — §6.2 dep list + #39 supply-chain row | §3.1, §5.2 #39, §6.2 |
| **O-2** | §3.2 rationale should name adversary-axis + capability-axis | **APPLIED** — design-rationale note | §3.1 |
| **O-3** | In-tree DUAL-CID precedent (`TwoCidStore`) unreferenced | **APPLIED** — §0.3 row + §3.3/§4.1 reframed as "extends in-tree two-CID mapping" | §0.3, §3.3, §4.1 |
| **O-4** | Revocation-propagation-lag inherited unbounded | **APPLIED** (BELONGS-NAMED-NOW) — sub-clause of Compromise #41 | §5.2 #41 |
| **O-5** | `generation_summary` under-specified for lagging-member rendezvous | **APPLIED** — set-generation (not per-member-vector) definitional sentence | §3.9, §10.4 NQ-D4 |
| **O-6** | Blast-radius ladder + trust-tier×op matrix | **APPLIED** (BELONGS-NAMED-NOW) — §3.4 ladder + THREAT-MODEL.md skeleton | §3.4, §7.1 doc-wave |
| **O-7** | `plaintext_cid_local` never-serialized needs an R5 kani/property test | **CARRIED-R3** — named R3 test-landscape seed | §3.3, §10.8 |
| **O-8** | §4.4 reduction-tally double-counts; §15.c guard referenced not specified | **APPLIED** — single canonical −8 tally; §15.c HALT-AND-SURFACE inlined (3rd-Kind-arm escalation) | §4.4, §1.3 |
| **O-9** | Sequencing/cost/standards refinements (DAK-parallelizable; cost non-additive; re-freeze contingency; HPKE-KEM-extensibility; did:key multicodec; full HPKE triple) | **APPLIED + CARRIED** — cost-non-additive note (§8.1); DAK parallelizable in DAG (§7.3); re-freeze contingency (NQ-A1); HPKE-KEM-extensibility (NQ-C1); did:key multicodec (NQ-C4); full HPKE triple noted | §7.3, §8.1, §10.6, §10.7 |

### The §5 R2/R3 questions from the triage (all carried-NOW per HARD RULE 12)

NQ-W1..W5 (§10.3), NQ-D1..D4 (§10.4), NQ-T1..T4 (§10.5), NQ-C1..C5 (§10.6), NQ-A1..A2 (§10.7) — every triage
§5 question is reproduced as a named NQ-* seed; NQ-A2 is RESOLVED in-plan (§9.3).

