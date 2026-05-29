# P1 — MCV2-C-2 Resolution: RestrictedScopeSet × MembershipSet Orthogonality

**Branch.** `phase-4-meta-core/membership-set-p1-c2-restrictedscopeset-orthogonality`
**Base.** `2172cb6d` (worktree HEAD; on `main`-tracking branch).
**Inputs read.**
- M-CONS-v2 consolidator @ `e62ff540` (1295 LOC; §5.1 Inv-20 10-clause; §7.4 RestrictedScopeSet; §17 ratifications).
- M-C2 v2 critique @ `0533d0cf` (931 LOC; §3.2 MCV2-C-2 setup + 3-path enumeration).
- N1 sub-graph elegance @ `ed592770` (1067 LOC; §3 Option H; §4 Tasks; key derivation chain).
- M2 primitive design @ `6170980b` (901 LOC; Layer-C as `MembershipSet::encrypt_to_set`).
- `crates/benten-drop/` (HEAD: lib.rs 93 LOC, bundle.rs 793 LOC; INTERNALS.md). Ships at v1-beta.
- `crates/benten-caps/src/authorization_grant.rs` (AuthorizationGrant{ucan, key_material, binding_sig} at HEAD).
- RATIFIED-sharing-and-confidentiality-2026-05-21.md + SPIKE-E/F/H: NOT present in `git ls-tree origin/main` — consumed VIA citations in N1/M-CONS-v2/INTERNALS.md.

**Disposition headline.** **FIX-NOW via Option (d) — N1-narrowed semantic.** `RestrictedScopeSet` is restricted to grant-time per-member sub-graph view-restriction WITHIN a MembershipSet Eve is ALREADY a member of. Non-member sharing routes through Drop-bundle Layer-C (path b) as a structurally distinct primitive. The mis-framing in M-CONS-v2 Inv-20 clause-h that EXTENDS RestrictedScopeSet composition to non-members is the unsoundness — not the underlying mechanism. Cost: **~0.5–0.8 wave-days** doc-only; ZERO code change to `benten-caps` / `benten-drop` / `benten-membership-set` skeleton. Strictly less code than (a) (HPKE-recipient amendment to Inv-19) and (c) (fork-on-share automation). Closes M-CONS-v2 R-MCV2-2 cleanly.

---

## §1 The contradiction restated

**M-CONS-v2 Inv-20 clause-h (final phrasing, line 503-507 of `/tmp/m-cons-v2.md`):**

> homogeneous-per-Kind MemberKey variant (M-CONS-v1 from M4 §11.5) + **`RestrictedScopeSet`-as-disjunction-of-`RestrictedScope` at `Scope::RestrictedSelector` arm (M-CONS-v2 from N1) — union-algebra-containment-lifted; orthogonal to MembershipSet at the `(MembershipSet, RestrictedScopeSet)` seal-seam tuple**

**M-CONS-v2 Inv-19 (kept from M-CONS-v1):**

> Encryption-substrate keying-function CRDT-input discipline (Path-A.5 + MembershipSet).

**Inv-20 clause-a:**

> `shared_key: K_Set` distributed via **multi-stanza-HPKE-Encap** to MEMBERS.

**The contradiction (M-C2 §3.2 setup line 552-558):** Alice has Atrium A with K_Set-A and 10 members; Eve is NOT a member. Alice wants to share a sub-graph (X + Y-children-of-X1) with Eve. Per F28 Eve gets a `RestrictedScopeSet` grant. Per Inv-20 clause-h that "composes orthogonally with MembershipSet" — but how does Eve obtain K_Set-A? Clause-a says K_Set is HPKE-Encap'd to MEMBERS. Eve isn't a member. The composition rule is **silent**, hence unsound.

**Why this is M-CONS-v2's bug, NOT N1's bug.** N1 §3.6 row (line 663 of `/tmp/n1.md`):

> Integration with MembershipSet: Orthogonal: `(MembershipSet, RestrictedScopeSet)` tuple at seal seam. MembershipSet decides RECIPIENTS-share-K_Set; RestrictedScopeSet decides which sub-graphs **each recipient** may walk. **PASS — clean orthogonality preserved.**

N1's framing presumes each recipient IS already among "RECIPIENTS-share-K_Set" — i.e., already a member. The "asymmetric shape" benefit Option H buys is for INTRA-member differentiation: 10 Atrium members each with a different sub-graph view. N1 NEVER claims to admit non-members to the K_Set.

The unsoundness is introduced by M-CONS-v2 §1.1 (the N1 promotion text) and §7.4 leaving the (member-vs-non-member) precondition implicit, then M-C2 §3.2 framing 3-path candidates to fix the gap. The cleanest fix is to make the precondition EXPLICIT (option d), not to invent a 4th cryptographic seam.

---

## §2 Task 1 — Cryptographic + architectural soundness of (a), (b), (c)

### §2.1 Threat-model rubric

For each path, evaluate against:

- **R1. IND-CCA2** — adversary cannot distinguish two ciphertexts of equal length even with decryption oracle (modulo own-decap).
- **R2. INT-CTXT** — adversary cannot forge a ciphertext that decrypts to a non-⊥ plaintext under an honest key.
- **R3. Per-recipient unlinkability** — Inv-20 clause-d; relay/network observer cannot link two stanzas as targeting the same recipient absent side-channel.
- **R4. Composes with Path-A.5** — `K(N) = HKDF(K(predecessor), info="step"||edge_label||N.cid)` chain remains keyed by Eve's grant-issued `K(root)` per Spike-E §15.f.
- **R5. Composes with N1 walk discipline** — RestrictedScopeSet `combinators::union` walker stays a Subgraph per `walker_as_subgraph()`; sub-scope containment lift remains decidable.
- **R6. Composes with Inv-19 (CRDT-input discipline)** — Path-A.5 keying-function still takes Version-Node-CID as input; no MUTABLE input crosses the seam.
- **R7. Composes with Inv-20 clause-a (K_Set distribution discipline)** — K_Set is HPKE-Encap'd to members, no others.
- **R8. Composes with Inv-20 clause-e (generation-CRDT-vector per-member-DID partition)** — partition keyed by member-DID set; non-members must not appear in the partition map.

### §2.2 Path (a) — Per-stanza HPKE-Encap K_Set to Eve

**Mechanism.** Alice produces an extra HPKE stanza targeting Eve's KEM pubkey, carrying K_Set-A. Eve decrypts the stanza, recovers K_Set-A, then walks per RestrictedScopeSet. Effectively: Eve becomes a "shadow member" at the cryptographic layer but is NOT in `members`/`role_assignments`.

**Score.**
- R1 IND-CCA2: PASS — HPKE-KEM-DEM construction preserves IND-CCA2 across recipients including Eve.
- R2 INT-CTXT: PASS — AEAD bind closes over stanza; no forgery vector introduced.
- R3 Per-recipient unlinkability: **CONDITIONAL** — if Eve's stanza is added to the same envelope's `stanzas[]`, the `sorted_member_did_list` AAD field (Inv-20 clause-c) MUST include Eve's DID or it forgery-binds the wrong list; if Eve's DID is added, then `sorted_member_did_list` no longer matches `members` and stanza-shape leaks the augmented list to every recipient. So unlinkability is preserved against external observer but Eve is OUTED to other recipients.
- R4 Path-A.5: PASS — Eve still walks via K(N) chain.
- R5 N1 walk discipline: PASS.
- R6 Inv-19: PASS.
- **R7 Inv-20 clause-a: FAIL.** K_Set is distributed to a non-member. The discipline is "via multi-stanza-HPKE-Encap to MEMBERS"; admitting Eve here destroys the type-system audit that members ⊆ stanza-recipients.
- **R8 Inv-20 clause-e: FAIL.** generation-CRDT-vector is partitioned PER member-DID. Eve has no entry; admitting Eve's K_Set acquisition means Eve participates in K_Set-rotation events (FORK-ONLY) without being in the partition map.
- **R9 audit-event surface:** Eve's `member_key_generation` becomes undefined; F-N2-A MembershipEvent stream has no event recording Eve's K_Set acquisition, so the audit trail per Inv-20 clause-j RBAC × UCAN composition LOSES a signed event for a key-material disclosure.

**Cost to fix.** Requires Inv-19 + Inv-20 clause-a to be AMENDED ("…to MEMBERS, OR to non-member UCAN-grantees whose `RestrictedScopeSet` scope is attested by `binding_sig`…"). This expands the trust model surface considerably (new threat-model rows; auditor must reason about 2 K_Set-acquisition paths). ~3–5 wave-days incl. THREAT-MODEL.md + SECURITY-PROOFS.md amendments + kani harness extension + golden vectors for the non-member stanza-shape + an additional MembershipEvent variant.

**Verdict.** Cryptographically sound but **violates 2 Inv-20 sub-clauses** + expands the audit-deliverable surface. **REJECTED.**

### §2.3 Path (b) — Layer-C single-recipient Drop-bundle, K_Set bypassed

**Mechanism.** Alice does NOT extend the Atrium-A K_Set distribution at all. She constructs a `DropBundle` (per `crates/benten-drop/src/bundle.rs::DropBundle`) targeting Eve's DID as `audience`. The bundle:
- Embeds a `RestrictedScope`-shaped `SubgraphSpec` per N1 (single-element or N1 multi-element RestrictedScopeSet).
- Embeds per-Node AEAD ciphertexts (`Vec<EncryptedContent>`) freshly produced by `benten-graph::aead_wrap` under a one-shot CEK derived from `K_principal_alice`.
- Embeds a single `AuthorizationGrant{ucan, key_material, binding_sig}` whose `key_material` carries the K(root) entries Eve needs to walk per Spike-E §15.f.
- Envelope-sig (Ed25519) by Alice closes the outer header.

Eve consumes via `DropBundle::consume_offline(&recipient_kp)` and walks per `RestrictedScope`-spec.

K_Set-A is NEVER revealed to Eve. The Drop-bundle key material is FRESH per-issue, derived from Alice's `K_principal_alice` (the K(N) chain per Path-A.5), NOT from K_Set-A.

**Score.**
- R1 IND-CCA2: PASS — `benten-crypto-suite::aead::wrap` per-Node AEAD + Ed25519 envelope-sig is the v1-beta substrate.
- R2 INT-CTXT: PASS — defense-in-depth (envelope-sig + per-Node AEAD tags); Spike G measurement at <12% overhead.
- R3 Per-recipient unlinkability: PASS — Drop-bundle is a separate artifact bound to a single audience; no multi-stanza envelope shape; no cross-recipient correlation surface.
- R4 Path-A.5: PASS — `K(N)` chain is exactly the mechanism the AuthorizationGrant `key_material` populates. This is the SHIPPING design.
- R5 N1 walk discipline: PASS — RestrictedScopeSet sits in `spec: RestrictedScopeSpec` directly; walker logic identical.
- R6 Inv-19: PASS — `K(V)` discipline is at the Path-A.5 K(N) seam; no change.
- R7 Inv-20 clause-a: **N/A — bypassed cleanly.** Drop-bundle is a separate primitive; K_Set is not in the picture. This is the CRUX of the architectural answer: **Inv-20 governs `MembershipSet::encrypt_to_set` only; Drop-bundle is its OWN seal-seam.**
- R8 Inv-20 clause-e: N/A — same reason.
- R9 audit-event surface: Drop-bundle issuance is auditable via the `binding_sig` over `(ucan, key_material, audience)`; Spike G defense-in-depth pin already covers tamper detection. Cleaner audit than (a).

**Cost.** ZERO new wave-days. The mechanism IS the shipping `benten-drop` crate (G-CORE-3f, PR #1340 + follow-ups landed through G-CORE-9 FREEZE). All R6 reality + revocation reach already documented per RATIFIED-sharing-and-confidentiality-2026-05-21.md §R6 + Compromise #31.

**Verdict.** **Structurally sound + already shipping.** The Inv-20 clause-h overreach is what creates a problem; if (b) is named as the non-member path explicitly, no contradiction remains.

**Subtle composition note.** Path (b) is NOT "RestrictedScopeSet composed with MembershipSet"; it is "RestrictedScopeSet composed with the Drop-bundle Layer-C primitive". The (MembershipSet, RestrictedScopeSet, K_Set-acquisition-path) triple M-C2 R-MCV2-2 calls for is best resolved by stating: K_Set-acquisition-path is **NOT** a third axis of the same primitive; it is a Kind-discriminated CHOICE OF PRIMITIVE — `MembershipSet::encrypt_to_set` (member path; K_Set-rooted) vs `DropBundle::issue` (non-member path; Path-A.5-rooted).

### §2.4 Path (c) — Fork Atrium A → A' = {Eve} + re-encrypt

**Mechanism.** Alice forks Atrium A into A' with `members = {Alice, Eve}`, mints fresh K_Set-A', re-encrypts the target sub-graph under K_Set-A', distributes K_Set-A' to Eve via the normal multi-stanza-HPKE-Encap.

**Score.**
- R1 IND-CCA2: PASS.
- R2 INT-CTXT: PASS.
- R3 Per-recipient unlinkability: PASS — A' is a fresh MembershipSet with fresh ID + parent_membership_set_id chain.
- R4 Path-A.5: PASS — A' walks K(N) chain under K_Set-A' rather than K_Set-A.
- R5 N1 walk discipline: PASS.
- R6 Inv-19: PASS.
- R7 Inv-20 clause-a: PASS — Eve is a member of A'.
- R8 Inv-20 clause-e: PASS — A' has its own generation-CRDT-vector.
- R9 audit-event surface: PASS — A' fork-event is a first-class MembershipEvent.

**Cost.**
- Re-encryption O(|sub-graph|) wall-clock per share; for non-trivial graphs this is minutes or worse.
- Adds a new MembershipSet (A') for every one-off share. A user with 50 share-relationships ends up with 50+ derived MembershipSets, each with FORK-ONLY rotation lineage. Storage host insider risk per Compromise #49 multiplied.
- Loses the N1 elegance: the whole POINT of RestrictedScopeSet was asymmetric INTRA-Atrium views without forking.
- Wave-days: ~2–3 to wire automated fork-on-share + an admin UX dialog; ~5–7 if you want a "minimal-fork" optimization (only re-encrypting the sub-graph + lazily migrating other data).

**Verdict.** Cryptographically sound but **expensive + UX-hostile + defeats N1's unification benefit**. **REJECTED** as the primary mechanism; surface to user as a FALLBACK when they want PCS-style forward secrecy on the share (which is exactly what a fresh Atrium gives them).

### §2.5 Ranking

| Path | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | Wave-days | Verdict |
|------|----|----|----|----|----|----|----|----|----|-----------|---------|
| (a) per-stanza K_Set to Eve | ✓ | ✓ | ⚠ | ✓ | ✓ | ✓ | **✗** | **✗** | ✗ | 3–5 (Inv-amend) | REJECT |
| (b) Drop-bundle Layer-C | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | N/A | N/A | ✓ | **0** (ships) | **ACCEPT as non-member path** |
| (c) Fork-on-share | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 2–7 | KEEP as fallback (PCS-share) |

---

## §3 Task 2 — UX implications

| Path | Eve's mental model | Alice's mental model | Member-set growth |
|------|---------------------|-----------------------|-------------------|
| (a) HPKE-recipient extension | "I'm in this Atrium" — but seeing only some content | "I added Eve with restricted view" | Atrium membership semantics become ambiguous (admin-list ≠ cryptographic-recipient-list); admin counts undercounted |
| (b) Drop-bundle | "Alice shared X with me" (familiar Dropbox / Signal Note-to-Self / WhatsApp share model) | "I sent X to Eve" (familiar gmail-share / iMessage-share model) | Zero growth (Atrium membership unchanged) |
| (c) Forked Atrium | "Alice gave me a copy" | "I made a copy for Eve" | Atrium proliferation — UX inundated with mini-Atriums |
| (d) N1-narrowed + (b) for non-members | Members: "I see only my part of our shared space"; non-members: "Alice shared X with me" | Same — uses different control depending on whether recipient is in the Atrium | Zero growth except per-fork |

**Winner: (d) — heterogeneous UX matches user mental models.** Members of Alice's Atrium have an "I see only my part of our shared space" view (per-member sub-graph view restriction). Non-members get a "Alice shared X with me" UX (Drop-bundle delivery). These ARE different mental models in real human practice — sharing with my therapist (Drop-bundle; ephemeral; tightly-scoped) is different from giving my financial-advisor read-only access to my budget-graph WITHIN my household Atrium (RestrictedScopeSet; persistent; sub-graph-scoped).

Path (a) is UX-confusing because Eve's mental model is "member with restrictions" but the membership-list machinery (D6/D7 admin counts, RBAC role display, member-key-rotation timeline) doesn't reflect her presence cleanly. Path (c) is UX-hostile under any share volume.

---

## §4 Task 3 — N1's actual claim re-examined; option (d)

**N1's claim verbatim (line 660-663 of `/tmp/n1.md`):**

> Per-recipient sub-graph grants are grant-layer concerns; Atrium-fork rotates `K_Atrium` and re-issues grants under the new K. RestrictedScopeSet survives the fork as a RE-ISSUED grant (same as single-scope grants today). **PASS — same as status quo.**

And line 614 (Option H derivation):

> Recipient walks each sub-scope under the existing path-tagged `K(N) = HKDF(K(predecessor), ...)` chain. No new key derivation, no ABE, no Cryptree CK restructuring. `AuthorizationGrant.key_material` carries the K-set for the union'd walk.

**N1's mental model.** `AuthorizationGrant.key_material` carries the K(root) entries the recipient needs. The K(N) chain is Path-A.5 path-tagged; it is derivable by ANYONE who has K(predecessor) and the canonical-path metadata. This is **Tahoe/Cryptree-style read-capability semantics**: the recipient holds path-tagged keys for their own slice of the tree, derived hierarchically from a root capability the issuer hands them.

K_Set-A (the Atrium-A shared key) is a DIFFERENT key: it's the encrypt-to-N-recipients HPKE-derived `shared_key` used by `MembershipSet::encrypt_to_set` to bind the seal-seam AAD (Inv-20 clause-c) and key the MultiRecipientSealing CEK. **K_Set-A and K(N) live on different seal-seams.**

**The mis-framing.** M-CONS-v2 §7.4 ("at `Scope::RestrictedSelector` arm") + Inv-20 clause-h's "orthogonal at (MembershipSet, RestrictedScopeSet) seal-seam tuple" conflates these into ONE seal-seam. They're not; they're TWO seal-seams, each governed by its own invariant family:

| Seal-seam | Invariant | Key | Distribution discipline | Granted to |
|-----------|-----------|-----|--------------------------|------------|
| MultiRecipientSealing (Inv-20 clause-a/c) | Inv-20 + Inv-16 | K_Set | multi-stanza-HPKE-Encap to members | MEMBERS only |
| Per-Node AEAD (Path-A.5 / Inv-19 / Spike-E) | Inv-19 + Inv-16 | K(N) via HKDF chain rooted at K_principal of producer | AuthorizationGrant.key_material with K(root) handoff | MEMBERS via member-K_Set-derived K(root) **OR** any UCAN-grantee via Drop-bundle |

**Option (d): RestrictedScopeSet restricted to per-member sub-graph view restriction; non-member sharing uses Drop-bundle Layer-C.**

Formally:
- **Precondition.** `MembershipSet::encrypt_to_set_with_restricted_scope(recipient_did, RestrictedScopeSet)` requires `recipient_did ∈ self.members`. Returns `E_RESTRICTED_SCOPE_RECIPIENT_NOT_MEMBER` otherwise.
- **Non-member path.** `benten-drop::DropBundle::issue(audience_did, RestrictedScopeSpec, …)` — already shipping — is the non-member share primitive. The `RestrictedScopeSpec` field can be N1-extended to carry a multi-element set (single AuthorizationGrant; Path-A.5 K(N) chain; no K_Set involvement).
- **Composition rule.** `(MembershipSet, RestrictedScopeSet)` IS orthogonal — at the per-member view-restriction seam — exactly as N1 framed it. The (non-member, RestrictedScopeSet) composition is via `(DropBundle, RestrictedScopeSet)`, a DIFFERENT orthogonal pair.
- **Inv-20 clause-h reworded.** Drop the "orthogonal to MembershipSet at the (MembershipSet, RestrictedScopeSet) seal-seam tuple" gloss; add: "applicable at grant boundary to **members of the MembershipSet** only; non-member sharing uses Drop-bundle Layer-C per §5".

**Why option (d) is strictly stronger than option (b)-integrated.** Option (b)-integrated ("specify (MembershipSet, RestrictedScopeSet, K_Set-acquisition-path) triple with a per-Kind table naming Drop-bundle as the non-member acquisition path") creates a 3-axis composition table where one axis (K_Set-acquisition-path) is actually a Kind-discriminated CHOICE OF PRIMITIVE. Option (d) collapses that into "use the right primitive for the recipient class" — strictly less surface, strictly less audit-deliverable text, strictly less THREAT-MODEL.md cross-product to reason about.

This matches the [HARD RULE 12 disposition pattern](https://github.com/...) — when there are 3 candidates one of which is "stop pretending the framing was right", that's a (b) variant in the form "BELONGS-NAMED-NOW to a more focused primitive: §5 of M-CONS-v2 already names Drop-bundle".

---

## §5 Task 4 — Recommendation

**RATIFY OPTION (d).** **Cost: 0.5–0.8 wave-days doc-only.** Justification against M-CONS-v2 + prior Ben ratifications:

1. **CLAUDE.md baked-in #1 (12-primitive irreducibility).** Option (d) keeps the primitive boundary at the existing 12; it does NOT mint a new primitive. RestrictedScopeSet stays a grant-layer composition over existing primitives.

2. **Ben ratification "iroh-gossip SCOPE-IN at v1-beta" (Q1, M-CONS-v2 §17).** Untouched by (d).

3. **Ben ratification "AtriumWithCGKA → AtriumWithRotatingGroupKey rename" (M-CONS-v2 §17).** Untouched.

4. **Ben ratification "Option-B per-message-ratcheting codepoint-reserve" (M-CONS-v2 §17 from N4).** Untouched.

5. **Inv-20 10-clause structure.** Clause-h is the ONLY clause touched. Wording change is local; the 10-clause skeleton stays. Other clauses (a, b, c, d, e, f, g, i, j) are unchanged.

6. **M2 primitive design §5.5 "Drop-bundle-to-recipients is one of the 4 unification sites".** Option (d) keeps the unification benefit FOR MEMBER-MEMBER multi-recipient seals while ALSO keeping the Drop-bundle path explicit for non-member sharing. The unification claim was always "the 4 cryptographic sites SHARE THE multi-stanza-HPKE substrate"; that's a statement about substrate sharing, not about substrate use cases. Drop-bundle's `EncryptedContent` per-Node AEAD substrate IS shared with `MembershipSet::encrypt_to_set` (both use `benten-crypto-suite::aead::wrap`). Substrate-unification stays intact under (d).

7. **N1 elegance preserved.** Option H union-of-scopes containment lift is unchanged. Backward-compat single-element shape unchanged. UCAN attenuation rule unchanged. `combinators::union` walker reuse unchanged. Spike H+1.1 opaque-arm rejection unchanged.

8. **F28 amendment.** F28 line in M-CONS-v2 §3 (line 391) keeps the "sub-graph asymmetric-shape closure" framing; add a sub-row: "F28-MEMBER-ONLY-PRECONDITION — `encrypt_to_set_with_restricted_scope` requires recipient ∈ members; non-member case is Drop-bundle Layer-C per §5."

9. **R-MCV2-2 closure.** M-C2 §3.2 R-MCV2-2 asked for "explicit (MembershipSet, RestrictedScopeSet, K_Set-acquisition-path) triple + per-MembershipSet-Kind composition table." Option (d) closes it via the SIMPLER framing: 2 orthogonal pairs ((MembershipSet, RestrictedScopeSet) for members; (Drop-bundle, RestrictedScopeSet) for non-members). No 3-axis table needed.

10. **Wave-MS-PRIMITIVE canary.** F28 was already absorbed into the canary at +0.95–1.2 wave-days (M-CONS-v2 §15 + §16). Option (d) does NOT add to the canary; doc-deliverables (THREAT-MODEL.md row, SECURITY-PROOFS.md proof obligation cleanup) land in Wave-MS-PRIMITIVE final integration.

11. **N1-D1 ratification preserved.** M-CONS-v2 §17 line 802 ("Ratify N1 Option H NESTED-SPEC + add F28 + extend Inv-20 clause-h + fold into Wave-MS-PRIMITIVE canary") stays in force; clause-h wording is REFINED, not REVERTED.

**Cost in wave-days.** 0.5–0.8 wave-days breakdown:

- 0.2 wd: M-CONS-v2 §1.1 + §5.1 + §7.4 wording refinement; F28 sub-row addition.
- 0.15 wd: THREAT-MODEL.md row "non-member-sub-graph-share goes through Drop-bundle; do not extend K_Set distribution to non-members".
- 0.1 wd: SECURITY-PROOFS.md remove the (currently-unprovable) (MembershipSet, RestrictedScopeSet) non-member composition lemma; ADD the cleaner per-member view-restriction lemma + an inter-primitive choice-of-primitive lemma.
- 0.1 wd: ErrorCode addition `E_RESTRICTED_SCOPE_RECIPIENT_NOT_MEMBER` to `benten-errors` + TS mirror (§3.5g cross-language rule-mirror discipline).
- 0.1 wd: V1-FROZEN-INTERFACE.md §15.c amendment text refinement (precondition spelled out).
- 0.05–0.15 wd: cite-drift cross-check + per-Kind dispatch table update in CRYPTO-CODEPOINTS.md (RestrictedScopeSet codepoint reserved for Atrium Kind only; DeviceMesh/SingleDevice DO NOT carry RestrictedScopeSet).

---

## §6 Task 5 — R0 plan-doc implications

### §6.1 M-CONS-v2 §6 (MembershipSetPolicy walkthrough) implications

M-CONS-v2 §6 covers MembershipSetPolicy per-Kind authority cardinality. Option (d) implications:

- **§6 add per-Kind row "RestrictedScopeSet applicable?"** — currently in M-C2 §1.2 table (line 387). Move that table into M-CONS-v2 §6. Column entries: `Atrium=Y (members only)`; `DeviceMesh=N`; `SingleDevice=N`. The "members only" precondition lives at construction of the RestrictedScope grant inside `MembershipSet::encrypt_to_set_with_restricted_scope`. Non-Atrium Kinds reject the call at the typesystem-or-runtime boundary.

- **§6 add cross-reference to §5.** When a non-member needs sub-graph access, §6 explicitly cross-refs §5 (Layer-C encrypt-to-recipient drops). The user doesn't get a "no" — they get a redirect to the right primitive.

- **§6 RBAC × RestrictedScopeSet composition.** N2's 3-role RBAC (Admin > Member > Viewer) gates the AUTHORITY to issue a RestrictedScopeSet grant. Per-member view restriction is INSIDE the Atrium's RBAC tree: a Member with role=Viewer cannot issue grants; a Member with role=Member can issue grants for their own-scope sub-graphs; an Admin can issue grants spanning the Atrium. Option (d) adds this as one row to §6's RBAC × per-Kind matrix.

### §6.2 M-CONS-v2 §7 (R0 skeleton) implications

§7 currently lays out wire format. Implications:

- **§7.4 RestrictedScopeSet at `Scope::RestrictedSelector` arm** — UNCHANGED wire format (Option H is preserved). Add a sentence: "Wire encoding is identical for member-path (MembershipSet seal) and non-member-path (Drop-bundle seal); the difference is which outer envelope wraps it. Per N1, single-element + multi-element codepoint-dispatch preserves backward-compat."

- **§7.5 NEW sub-section: K_Set-acquisition-path-discrimination.** Spell out: (i) member path uses MultiRecipientSealing envelope with K_Set HPKE-distributed; (ii) non-member path uses DropBundle envelope with Path-A.5 K(N) chain via AuthorizationGrant.key_material. The TWO envelopes are codepoint-dispatched at F8 (MembershipSetEncryption codepoint family) — DropBundle has its OWN codepoint sub-family at MultiRecipientSealing-codepoint-NEIGHBOR (per benten-drop INTERNALS §3).

- **§7.6 NEW table: "Recipient class × primitive choice".** 2×2 table: (member, non-member) × (single-recipient, multi-recipient). Member-single = MembershipSet::encrypt_to_set with N=1 (degenerate). Member-multi = standard. Non-member-single = DropBundle. Non-member-multi = N DropBundles OR fork-on-share (option c) if PCS desired. This makes the choice-of-primitive deterministic for implementers.

### §6.3 SECURITY-POSTURE / SECURITY-PROOFS

- **NEW lemma: per-member view restriction soundness.** Given Atrium with K_Set + members + RestrictedScopeSet grants, the view of each member is exactly the union of sub-scopes in their RestrictedScopeSet — provable via Option H union-algebra containment lift (Spike-F decidability + N1 §3 walker-as-Subgraph composition).

- **REMOVED claim: (MembershipSet, RestrictedScopeSet) non-member composition.** Was unprovable; remove.

- **NEW lemma: choice-of-primitive soundness.** For any (sender, recipient, sub-graph-shape) triple, exactly ONE of {MembershipSet::encrypt_to_set_with_restricted_scope, DropBundle::issue} is well-typed; the choice is determined by `recipient ∈ MembershipSet.members`. This is an INVERTED Path-B (typed-choice-at-API-boundary) rather than an unproven orthogonal-composition.

### §6.4 THREAT-MODEL.md rows

- **NEW row: Non-member shadow-membership rejection.** Document that Inv-20 clause-a is unmodified: K_Set is HPKE-Encap'd to MEMBERS only; non-member share is the Drop-bundle primitive. ANY implementation that extends K_Set distribution to non-members is a SECURITY BUG.

- **UPDATED row: RestrictedScopeSet × MembershipSet composition.** Refine to "applicable per-member only; non-member sharing uses Drop-bundle per §5".

### §6.5 V1-FROZEN-INTERFACE.md §15.c

- Amendment: name the `recipient ∈ self.members` precondition as part of the FROZEN API contract. `MembershipSet::encrypt_to_set_with_restricted_scope` returns `Result<_, E_RESTRICTED_SCOPE_RECIPIENT_NOT_MEMBER>` if violated. Frozen across v1.x.

### §6.6 cite-drift cross-language mirror (§3.5g)

- New ErrorCode `E_RESTRICTED_SCOPE_RECIPIENT_NOT_MEMBER` requires:
  - Rust `benten_errors::ErrorCode::RestrictedScopeRecipientNotMember`
  - TS `ErrorCode` class entry
  - drift-defense surface entry per pim-N cross-language rule-mirror
- Per pim-N-ratification-must-close-origin (§3.6h): the SAME PR that lands the new ErrorCode lands the precondition check at the construction site in `benten-membership-set` (when minted) or in a doc-pin to the construction site in the M6 §10.4 V1-FROZEN row when the crate is still skeleton.

---

## §7 What changes downstream — execution-ready checklist

When Wave-MS-PRIMITIVE canary runs (~4.5–6 wd sequential per M-CONS-v2 §15), the implementer brief MUST:

1. Mint `MembershipSet::encrypt_to_set` AND `MembershipSet::encrypt_to_set_with_restricted_scope` as TWO public methods; the second's first action is `if !self.members.contains(&recipient) { return Err(E_RESTRICTED_SCOPE_RECIPIENT_NOT_MEMBER) }`.
2. Reject calls on non-Atrium Kinds via `if self.kind != MembershipSetKind::Atrium { return Err(E_RESTRICTED_SCOPE_NON_ATRIUM_KIND) }` — second new ErrorCode (per the per-Kind table in §6.1).
3. Wire test fixture: golden vector for (member with RestrictedScopeSet — happy path) + (non-member with RestrictedScopeSet — E_RESTRICTED_SCOPE_RECIPIENT_NOT_MEMBER) + (non-Atrium Kind — E_RESTRICTED_SCOPE_NON_ATRIUM_KIND).
4. THREAT-MODEL.md row LANDS in same PR as the construction site (§3.5g pre-flight rule).
5. SECURITY-PROOFS.md "choice-of-primitive soundness" lemma LANDS in same PR.
6. Drop-bundle path is UNCHANGED — no code edit; only docs cross-link.
7. pim-N §3.6h enforcement: the M-CONS-v2 wording refinement PR (the FIRST place clause-h is reworded) MUST close (or DEFER-NAMED) the origin instance of the unsoundness — which is M-CONS-v2's own §7.4 + §5.1 wording. So that PR lands the §1.1 / §5.1 / §7.4 edits TOGETHER.

---

## §8 Cross-cut audit: does (d) affect other M-C2 findings?

Quick scan:

- **MCV2-C-1 (Bob role-change race).** Independent of (d) — about RBAC × CRDT race; resolved by R-MCV2-1 (role_generation AAD-bind). UNCHANGED by (d).
- **MCV2-C-3 (iroh-gossip topic-id × unlinkability).** Independent — transport-layer concern. UNCHANGED.
- **MCV2-C-4 (member-driven vs admin-driven mutation race).** Independent — CRDT-partition discipline. UNCHANGED.
- **MCV2-F-A through F-F.** F-F (RestrictedScopeSet × Path-A.5 immutable) explicitly cites Eve as a non-member; with (d), Eve's grant is via DropBundle, and the OLD-X1-immutable behavior is PRESERVED (Drop-bundle is forever-valid per Compromise #31). Disposition unchanged ("correct by Compromise #31"); just the framing references Drop-bundle now.
- **MCV2-B-1 (RBAC × RestrictedScopeSet × Path-A.5 immutable).** With (d), Eve's RBAC role is "non-member"; her RestrictedScopeSet is via DropBundle; RBAC × RestrictedScopeSet × Path-A.5 narrows to per-member composition only, simplifying B-1. Closure of B-1 is STRICTLY CHEAPER under (d).
- **MCV2-B-4 (per-Kind specialization × RestrictedScopeSet shape).** Already noted M-C2 line 393: "RestrictedScopeSet semantics differ per-Kind — only Atrium needs it." Option (d) hardens this: REJECT RestrictedScopeSet on non-Atrium Kinds at constructor. B-4 closure cheaper.

**Result.** Option (d) is consistent with closure dispositions of all other M-C2 findings; in 3 cases (F-F, B-1, B-4) it makes their closures STRICTLY CHEAPER.

---

## §9 Disposition + summary

**Disposition.** **FIX-NOW per HARD RULE 12 clause-a (in-scope, fixable now).** Option (d) ratified as elegant clean separation; option (b) folded as the named primitive (Drop-bundle Layer-C) for non-member sharing; option (a) REJECTED (violates Inv-20 clause-a/e); option (c) KEPT as FALLBACK for users who explicitly want PCS-style share semantics.

**Net wave-day impact.** +0.5–0.8 wd absorbed into Wave-MS-PRIMITIVE canary final integration. STRICTLY LESS than M-C2 R-MCV2-2's proposed (MembershipSet, RestrictedScopeSet, K_Set-acquisition-path) triple specification (~1.0–1.5 wd) because (d) eliminates the third axis rather than parameterizing over it.

**LOC impact.** ~10 Rust LOC (2 ErrorCode entries + 2 precondition checks) + ~10 TS LOC (mirror) + ~80 docs LOC (M-CONS-v2 §1.1/§5.1/§6/§7 refinements + THREAT-MODEL.md row + SECURITY-PROOFS.md lemma swap + V1-FROZEN-INTERFACE.md §15.c). Total ~100 LOC.

**Audit-deliverable impact.** Strictly REDUCES SECURITY-PROOFS.md surface (one unprovable lemma REMOVED + two cleaner lemmas ADDED). Strictly REDUCES THREAT-MODEL.md surface (one cross-product row REMOVED + one warning row ADDED). NET reduction of audit pages.

**Composition with prior Ben ratifications.** Compatible with all 5 (Q1 iroh-gossip SCOPE-IN; N2 24-op as-is; #55 GDPR-RTBF P2P honest-disclosure; Option-B per-message-ratcheting reserve; AtriumWithRotatingGroupKey rename). No re-litigation required.

**Recommended next action (for Ben).** RATIFY option (d); Ben pre-authorizes M-CONS-v2 wording refinement to land in the next P1-batch (the "P1 resolution batch" covering R-MCV2-1 through R-MCV2-8 doc-only refinements). The benten-drop crate continues shipping as the non-member primitive without modification.

---

**Out-of-scope items surfaced (NOT deferred — recorded for visibility).** None — all implications close inside Wave-MS-PRIMITIVE canary or its final integration sub-wave.

**Open Q for Ben (only one).** Should the FALLBACK option (c) "fork-on-share for PCS-style semantics" be SURFACED as a separate UX path at v1-beta, or DEFERRED to Phase-4-Meta-Composing? My prediction: defer — there's no v1-beta surface for "share with PCS" today, and adding it now stretches v1-beta scope. Defer-named destination: M-CONS-Composing forkable-share UX backlog.

— end —
