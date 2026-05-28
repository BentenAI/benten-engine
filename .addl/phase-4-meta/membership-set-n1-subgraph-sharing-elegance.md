# N1 — Per-recipient sub-graph sharing elegance survey

> **Top-banner re-orient (HANDOFF discipline).** This document is N1 of 4 parallel
> post-M-CONS refinement specialists investigating the elegant/strong/permanent
> shape for Benten's per-recipient sub-graph sharing primitive. Read BEYOND what's
> named here if you arrive cold: M-CONS at `74580ee6`, M2 primitive at `6170980b`,
> the M1b Atrium-membership-sharing cataloger at `1816ea60`, the in-tree
> `crates/benten-caps/src/restricted_spec.rs` (6-dimension `RestrictedScope`,
> `#[non_exhaustive]`, OPAQUE-arm REJECTED), the in-tree
> `crates/benten-core/src/subgraph_spec/{spec,walker,combinators}.rs` (4-thing thin
> core + intersect/union/filter combinators + walker-as-Subgraph fractal pin), and
> `docs/V1-FROZEN-INTERFACE.md` §15.f (the two-path key-derivation contract; Spike-E
> Interpretation-B; `K(N) = HKDF(K(predecessor), info="step"||edge_label||N.cid)`).
> The named brief inputs `RATIFIED-sharing-and-confidentiality-2026-05-21.md` and
> `SPIKE-E/F/H/H1` markdown artifacts do NOT exist as standalone files at HEAD; their
> content is realised IN-CODE under `benten-core/subgraph_spec/`, `benten-caps/`
> (`restricted_spec.rs`, `authorization_grant.rs`), and IN-DOC under
> `docs/V1-FROZEN-INTERFACE.md` §15.a–§15.i. M-CONS §10.7 already names the
> "Cryptree-confusion as naming-imports-folk-knowledge" hazard (M5 §1.2 +
> CGKA-LITE → MultiRecipientSealing rename). This N1 doc extends that audit to the
> sub-graph SHAPE question Ben surfaced (per-recipient ASYMMETRIC shape).
>
> Tree HEAD at write: `2172cb6d` (origin/main, fetched + verified at session start).
> Branch: `phase-4-meta-core/membership-set-n1-subgraph-sharing-elegance`.
> Date: 2026-05-28.

- **Role.** N1 — survey access-control primitives in literature for the SHAPE
  problem (asymmetric per-recipient sub-graph), evaluate elegant-shape candidates
  C/D/E/F/G/H against Benten's v1-FROZEN constraints, and recommend the most
  elegant/permanent shape per
  `feedback_extra_reflection_pass_for_elegant_permanent_shape.md`.
- **Scope.** Tasks 1–4 per the dispatch brief. NOT a re-litigation of MembershipSet
  unification (M-CONS owns). NOT a re-open of the Spike H+1.1 OPAQUE-arm decision
  (already RATIFIED-as-REJECTED). NOT a CGKA shape proposal (M5 owns; AtriumWithCGKA
  codepoint-reserved per F13).
- **Out-of-scope.** Authoring Rust code for any new variant. Rewriting V1-FROZEN-
  INTERFACE.md §15.x rows (that's R0 plan-doc surface).

---

## §0 Headline summary (read-cold)

**Verdict.** **OPTION H-NESTED-SPEC wins.** Promote the in-code `SubgraphSpec`
`combinators::intersect/union/filter` mechanism (which already ships at
`g-core-3w/subgraph-spec-walker @ 5c2947c8`) from "internal combinator surface" to
a **first-class grant-time composition primitive**: a single `AuthorizationGrant`
can carry a `Vec<SubgraphSpec>` (or equivalently a `Spec` produced by `union` of
sibling root-scoped specs) instead of exactly one `RestrictedScope`. The
asymmetric shape "Eve gets X + only Y-children-of-X1 + only Z-children-of-X2" then
becomes the union of three single-root sub-specs `union(spec_X, spec_X1_to_Y,
spec_X2_to_Z)`. Containment + intersection algebra is already proptest-verified
structural-containment-preserving. Per-recipient KEYING composes orthogonally: the
recipient walks the union'd spec under the existing path-tagged
`K(N) = HKDF(K(predecessor), info="step"||edge_label||N.cid)` chain — no new key
derivation needed; no ABE; no Cryptree clearance-key restructuring; no
opaque-arm structural unsoundness.

**Confidence.** HIGH on the composition direction (union/intersect already ship +
proptest-verified). MED-HIGH on the "this closes the asymmetric-shape limitation
cleanly" claim (depends on whether `Vec<Spec>` lift on the wire is acceptable in
the `Scope::RestrictedSelector` arm). MED on the cost estimate (+0.3 to +0.8
wave-days vs the M-CONS baseline; this is a SMALL extension, NOT a new primitive).

**The elegant move.** The brief frames asymmetric-shape as a NEW PROBLEM requiring
a NEW PRIMITIVE (Option D ABE, Option E access-tree, Option F selector, Option G
committed Merkle-set). **It is not.** Benten ALREADY has the structural piece:
the `intersect/union/filter` combinators are documented as "structural-containment-
preserving (proptest verified)" and union widens / intersect narrows / filter
narrows. The shape question collapses to **expose the union surface at the grant
boundary**. This is the elegant extension — strictly less code than any of
D/E/F/G; preserves CLAUDE.md baked-in #1 (12-primitive irreducibility — Spec stays
data + walker stays composed-from-12-primitives); preserves Spike H+1.1's named-
extension-slot discipline (union is a NAMED structural operation, not an opaque
witness arm); preserves the EXACTLY-2-arm `Scope::{Hashes | RestrictedSelector}`
frozen surface (the change is INSIDE the `RestrictedSelector` arm — make the
field a `RestrictedScopeSet` instead of a single `RestrictedScope`).

**Cross-panel.** This aligns with M-CONS §10.7 "naming-imports-folk-knowledge"
discipline (don't import Cryptree's hierarchical-grant intuition into Benten when
the actual mechanism is path-tagged-keys + Spec-as-data). It is COMPATIBLE with
the M-CONS-ratified MembershipSet primitive (sub-graph sharing is orthogonal to
the MembershipSet — Spec governs which Nodes a recipient may walk; MembershipSet
governs which RECIPIENTS share a `K_Set`; they compose by `(MembershipSet,
Spec)`-tuple at the seal seam).

---

## §1 Background — what Benten ships at HEAD

### §1.1 The 4-thing thin core `Spec`

`crates/benten-core/src/subgraph_spec/spec.rs` (per M1b §1.2(f) + V1-FROZEN
§15.h):

```rust
// Conceptual shape; actual code at the cited paths.
pub struct Spec {
    roots: BTreeSet<Cid>,                      // entry-point Node CIDs
    expansion: Expansion,                      // edge traversal rules
    inclusion: Inclusion,                      // per-Node admission predicate
    termination: Termination,                  // depth + cycle stop
}
```

Walker at `walker.rs:78` (`pub fn walk`); fractal pin at `walker.rs:183`
(`pub fn walker_as_subgraph() -> Subgraph`). BFS-order canonical enumeration.

### §1.2 `RestrictedScope` — the 6-dimension product (CAP layer)

`crates/benten-caps/src/restricted_spec.rs:103`:

```rust
#[non_exhaustive]
pub struct RestrictedScope {
    pub roots: Option<Vec<Cid>>,
    pub edge_allowlist: Option<Vec<String>>,
    pub max_depth: Option<u32>,
    pub label_allowlist: Option<Vec<String>>,
    pub label_denylist: Option<Vec<String>>,
    pub property_equalities: Option<BTreeMap<String, PropertyValue>>,
}
```

**Composition discipline:** `RestrictedScope::contains` is `&&`-composed across
all 6 dimensions (per-dimension narrowing checks AND-ed together). Reflexive +
transitive — decidable containment per dimension.

**Frozen surface:** `#[non_exhaustive]` is APPLIED so future named dimensions
(`time_window`, `geo_bound`, `cardinality_bound`) are a minor-version bump (per
`docs/V1-FROZEN-INTERFACE.md` §15.b). **Opaque-refinement-witness arm REJECTED**
as structurally unsound (Spike H+1.1 §b.SEC #4 — the witness binds the
attestation, not the spec body; NP-hard satisfiability over arbitrary `SubgraphSpec`;
non-portable across SubgraphSpec evolution).

### §1.3 `Scope` — the EXACTLY-2-arm UCAN scope enum

`crates/benten-caps/src/scope.rs:46` (per M1b §1.2(g) + V1-FROZEN §15.c):

```rust
// NOT #[non_exhaustive] — third arm = HALT-AND-SURFACE-TO-BEN
pub enum Scope {
    Hashes(Vec<Cid>),
    RestrictedSelector(RestrictedScope),
}
```

### §1.4 `AuthorizationGrant` — the ONE signed wire-artifact

`crates/benten-caps/src/authorization_grant.rs:238` (per V1-FROZEN §15.d):

```rust
pub struct AuthorizationGrant {
    pub ucan: UcanEnvelope,
    pub key_material: GrantKeyMaterial,
    pub binding_sig: Vec<u8>,
    pub audience_binding: Cid,
    pub issuer_verifying_key: Vec<u8>,
    pub audience_pubkey: Option<Vec<u8>>,
}
```

`binding_sig` covers `(ucan, key_material, audience)` together — neither can be
substituted. Consistent across the online ALPN handler (G-CORE-3e) and the
offline Drop bundle (G-CORE-3f).

### §1.5 Path-tagged key derivation (Spike-E Interpretation-B)

`crates/benten-crypto-suite/src/structural_kdf.rs` + V1-FROZEN §15.f:

```text
K(root) = HKDF-SHA256(K_principal, info = "root" || root_cid)
K(N)    = HKDF-SHA256(K(predecessor), info = "step" || edge_label || N.cid)
```

**A Node reachable by multiple paths gets multiple distinct keys** (per §15.f
line 1617). The envelope records which canonical path produced each ciphertext.
This is **explicitly a feature for selective-share** (§15.f line 1618:
"Path-tagged keys: a Node reachable by multiple paths gets multiple distinct
keys (feature for selective-share...").

### §1.6 The `combinators::intersect/union/filter` surface

`crates/benten-core/src/subgraph_spec/combinators.rs` (g-core-3w branch, landed):

```text
intersect(a, b) ⊆ a + ⊆ b                  (narrows)
a ⊆ union(a, b) + b ⊆ union(a, b)          (widens)
filter(a, p) ⊆ a                           (narrows; p can only constrain)

Commutativity + associativity hold on canonical bytes.
```

`intersect`:
- `roots = a.roots ∩ b.roots`
- `edges = (source, label, target) triples present in BOTH`
- `max_depth = min(a.max_depth, b.max_depth)`
- `inclusion = narrower restricted-spec (allowlist intersect; denylist union)`

`union` (symmetric, widens). `filter(a, p)` narrows by composable predicate `p`.

**Proptest verified** structural-containment-preserving at
`tf3w_combinators_intersect_union_filter_proptest.rs`. This is the load-bearing
discovery for this N1 analysis.

---

## §2 Task 1 — Survey of access-control primitives

This section evaluates each named system for **whether it natively expresses
asymmetric per-recipient sub-graph shape** ("Eve gets X + only Y-children-of-X1 +
only Z-children-of-X2, where Y ≠ Z, in a single grant"). Format per system:
**mechanism / asymmetric-shape verdict / Benten-import-cost / source**.

### §2.1 Cryptree (Grolimund/Meisser/Schmid/Wattenhofer 2006, SRDS '06)

**Mechanism.** Five-key hierarchical structure rooted at each folder: Clearance
Key (CK) — revealed to grant access; Subfolder Key (SK) — manages descendants;
Files Key (FK) — files in folder; Data Key (DK) — per-file content-key
(`FK_p(f) → DK_f` link); Backlink Key (BK) — parent reference for granting access
to individual files. Recursive grant: hand over CK of folder X → recipient
derives SK_X, FK_X, all child CKs transitively. "Cryptographic links" are either
symmetric (knowledge of K1 derives K2) or asymmetric (encrypt K2 under K1's
public-key). Lazy revocation: re-encrypt items on next change (kicked party
keeps old keys ⇒ can keep accessing items they cached but not subsequent
changes).

**Asymmetric-shape verdict.** **NO.** Cryptree's elegance lies entirely in
HIERARCHICAL grant ("give the whole subtree under X in constant time"). It does
NOT express "X + only Y-children-of-X1 + only Z-children-of-X2" — to share that
shape you'd issue THREE Clearance Keys (one per root), and the recipient would
have NO way to express "only the Y-subtree under X1, not the rest of X1's
children" without either (a) restructuring the folder tree to separate Y from
the rest of X1's children (graph pre-tagging in our terminology), or (b)
revealing X1's CK and accepting the recipient sees X1's other children too.

**Benten-import-cost.** Importing the Cryptree key-graph structurally would
DESTROY Benten's path-tagged `K(N)` invariant (Path-A.5 — K keys to immutable
Version-Node-CID; not to a separate per-folder CK). M-CONS §10.7 already names
this hazard: "Atrium-fork semantic ≠ Cryptree mechanism." **DEFER —
DISAGREE-WITH-EXPLANATION (HARD RULE clause-c).**

**Source.**
- Grolimund et al., "Cryptree: A Folder Tree Structure for Cryptographic File
  Systems," SRDS '06 ([ACM DL](https://dl.acm.org/doi/10.1109/SRDS.2006.15);
  [ETH PDF](https://tik-db.ee.ethz.ch/file/146566189b90f952b8ab1dcf98010781/srds06.pdf)).
- [SlideToDoc Cryptree summary](https://slidetodoc.com/cryptree-a-folder-tree-structure-for-cryptographic-file/)
  confirms the 5-key structure + lazy revocation. The PDF could not be auto-
  extracted (FlateDecode) so quoting is via the slide summary; primary source
  is the ACM DL paper.

### §2.2 CP-ABE / KP-ABE (Bethencourt-Sahai-Waters 2007, SP '07)

**Mechanism.** Attribute-Based Encryption: ciphertext carries an ACCESS POLICY
expressed as a monotonic boolean formula (or threshold-tree) over ATTRIBUTES;
each recipient's secret key is bound to a SET OF ATTRIBUTES. Decryption succeeds
iff the recipient's attribute set satisfies the ciphertext's policy.
Collusion-resistant via per-user randomization. KP-ABE inverts: ciphertext carries
attributes; key carries policy.

**Asymmetric-shape verdict.** **YES in theory; with substantial structural cost.**
You can tag each Benten Node with attributes (`["parent=X1", "Y-shaped"]` vs
`["parent=X2", "Z-shaped"]`) and Eve's key would be issued under the policy
`(parent=X ∧ depth≤1) ∨ (parent=X1 ∧ Y-shaped) ∨ (parent=X2 ∧ Z-shaped)`.

**Benten-import-cost.** **PROHIBITIVE for v1-beta.**
- Pairing-based crypto (BLS12-381 or similar) is NOT in Benten's hybrid floor
  (Inv-17 mandates ML-KEM + classical KEM; CP-ABE adds a third asymmetric
  primitive with its own KEM-IND-CCA story and PQ-security concerns).
- Ciphertext size grows with policy complexity (O(|policy|) group elements per
  ciphertext); the per-Node AEAD chunk-size constant (V1-FROZEN §15.g
  `IROH_BLOCK_SIZE = 16 KiB`) is broken.
- POLICY IS VISIBLE in the ciphertext (or there's a "hidden-policy" CP-ABE
  variant with worse security proofs and even bigger ciphertexts) — breaks the
  Compromise #48 MembershipSet-shape-leak boundary disclosure.
- REVOCATION is notoriously hard in ABE (revocation requires attribute-key
  rotation; lazy revocation similar to Cryptree but no PCS).
- CLAUDE.md baked-in #5 crypto-agility is violated: ABE is not codepoint-
  swappable with HKDF + HPKE.

**DEFER — DISAGREE-WITH-EXPLANATION.** Revisit-trigger: "the attribute-vocabulary
of Benten Nodes stabilises into ~5–8 discrete categories AND a use-case emerges
where the asymmetric-shape pattern is the DOMINANT use-case (>30% of grants)" —
neither is true at v1-beta or v1-GM.

**Source.**
- Bethencourt, Sahai, Waters, "Ciphertext-Policy Attribute-Based Encryption,"
  IEEE SP '07 ([UT Austin PDF](https://www.cs.utexas.edu/~bwaters/publications/papers/cp-abe.pdf);
  [UCLA PDF](http://web.cs.ucla.edu/~sahai/work/web/2007%20Publications/SSP2007.pdf)).
- Goyal-Pandey-Sahai-Waters KP-ABE ("universal access tree" generalisation).

### §2.3 Proxy Re-Encryption (PRE)

**Mechanism.** Data owner encrypts to themselves once; a re-encryption key
`rk_{A→B}` lets a semi-trusted proxy transform A's ciphertext into B's
ciphertext WITHOUT learning the plaintext. Recent PRE+ABE hybrids (Ateniese et
al. NDSS '05; arXiv:2212.06889 multi-recipient threshold) enable fine-grained
delegation.

**Asymmetric-shape verdict.** **PARTIAL.** PRE solves "give Bob access to A's
ciphertext without re-encrypting" but does NOT inherently express asymmetric
sub-graph SHAPE — the SHAPE question is still encoded in the policy layer (which
ciphertexts are PRE'd to Bob). For Benten this is equivalent to "issue Bob N
separate grants" — no shape compression.

**Benten-import-cost.** PRE adds a third trust principal (the proxy) which
violates Benten's "no third-party trust" property — the engine ships
encrypt-at-rest with the recipient as the sole decryptor; introducing a proxy
breaks the threat model documented in `docs/SECURITY-POSTURE.md`. **DEFER —
DISAGREE-WITH-EXPLANATION.** Revisit-trigger: cloud-relay model emerges as
v1.x feature (would need a Compromise mint).

**Source.**
- Ateniese, Fu, Green, Hohenberger, "Improved Proxy Re-Encryption Schemes with
  Applications to Secure Distributed Storage," NDSS '05
  ([PDF](https://spqrlab1.github.io/papers/ateniese-proxy-reenc-ndss05.pdf)).
- Eprint 2018/426 "Adaptively Secure Proxy Re-encryption."

### §2.4 Tahoe-LAFS file capabilities

**Mechanism.** Each file/dir has paired capabilities: `rwcap` (read-write) +
`rocap` (read-only) + `verify-cap` (verify-only). Directory nodes store
children as `(name, rocap, rwcap, metadata)` — the `rwcap` is encrypted under
the dirnode's writekey so read-only users cannot decrypt it. **Transitive
read-only inheritance:** when a read-only user adds a child, they put the
child's `rocap` in BOTH the `rwcap` AND `rocap` slots — read-only users of the
parent cannot decrypt the (encrypted) `rwcap` slot → forced read-only inherited
access. Twelve capability types in total.

**Asymmetric-shape verdict.** **NO.** Sharing is at DIRNODE GRANULARITY —
sharing a directory URI grants consistent access to all descendants reachable
through that dirnode. To express "Eve gets X + only Y-children-of-X1 + only
Z-children-of-X2," you must create three separate dirnodes (or three sibling
dirnodes in a synthetic root) and share each cap separately. WebFetch quote:
"The design offers 'strong' delegation but assumes 'grant boundaries' align
with directory boundaries."

**Benten-import-cost.** Tahoe-LAFS's rwcap/rocap mechanism maps cleanly onto
Benten's existing `K_principal` → `K(N)` derivation chain — Benten's "Bob holds
`K(N)` can decrypt subsequent Bob-reads" semantic is structurally Tahoe-equivalent.
**Tahoe confirms Benten's current design is on a well-trodden path**; the
asymmetric-shape limitation is shared. **CITE-AS-PEER-CONFIRMATION.**

**Source.**
- [Tahoe-LAFS Capabilities wiki](https://tahoe-lafs.org/trac/tahoe-lafs/wiki/Capabilities)
- [Tahoe-LAFS Directory Nodes spec](https://tahoe-lafs.readthedocs.io/en/latest/specifications/dirnodes.html)
- [Tahoe URIs spec](https://tahoe-lafs.readthedocs.io/en/tahoe-lafs-1.12.1/specifications/uri.html)

### §2.5 UCAN / ZCap-LD / Fission delegation

**Mechanism.** UCAN: JWT-shaped capability tokens with `aud` (audience),
`iss` (issuer), `cap` (capability set), `prf` (proof chain), `nbf`/`exp` time
window. Delegation chain: each link attenuates the parent's capabilities.
ZCap-LD: similar but JSON-LD shape with "caveats" attached as restrictions.
Benten ALREADY uses UCAN at `crates/benten-caps/` (Fission spec).

**Asymmetric-shape verdict.** **NOT NATIVE.** UCAN's `cap` field expresses
"resource + ability" tuples; attenuation narrows resources OR abilities. UCAN
does NOT have a native "sub-graph" type — Benten ratified the `Scope::
RestrictedSelector(RestrictedScope)` arm exactly to encode sub-graph shape inside
the UCAN `cap`. Per the UCAN delegation spec, multiple `cap` entries within a
single UCAN are conjunction-of-AUTHORITIES (logical OR over capabilities the
holder may invoke). **The asymmetric shape "X + Y_under_X1 + Z_under_X2" maps
naturally onto a `Vec<Scope::RestrictedSelector>` inside ONE UCAN.** This is
the structural lever for Option H.

**Benten-import-cost.** Zero — already-in-tree. The question is purely whether
the `Scope` enum admits multi-Spec inside the `RestrictedSelector` arm.

**Source.**
- [UCAN Specification](https://ucan.xyz/specification/)
- [Fission UCAN Guide](https://fission.codes/blog/a-guide-to-ucans/)
- [ucan-wg/delegation](https://github.com/ucan-wg/delegation)
- [W3C-CCG ZCap-LD v0.3](https://w3c-ccg.github.io/zcap-spec/)

### §2.6 Jazz CoValue Groups (local-first, Garden 2025)

**Mechanism.** Jazz CoValues each have an OWNER (Group or Account). Groups
grant access to multiple users with roles `admin/writer/reader`. Group holds
a SHARED READ KEY (XSalsa20); on member removal the key ROTATES + is re-shared
to remaining members. Each CoValue session uses BLAKE3 append-only hashing
signed Ed25519 per transaction.

**Asymmetric-shape verdict.** **NO — single Group = single shape.** A Group
grants UNIFORM access to all CoValues it owns; asymmetric per-member shape
requires multiple Groups (one per shape) and adding each member to the
appropriate subset of Groups. This is **structurally the same pattern as
Benten today** (multiple `AuthorizationGrant`s for multiple shapes).

**Benten-import-cost.** Jazz's Group ≈ Benten's MembershipSet; the SHAPE
question is orthogonal in BOTH systems. Jazz handles forward-secrecy on
member removal via key rotation (which Benten Compromise #52 explicitly DOES
NOT provide for Atrium-Kind). **CITE-AS-PEER-CONFIRMATION** of the
"shape ≠ MembershipSet" orthogonality. Forward-secrecy difference is the
M-CONS-named Compromise #52 trade-off and is NOT the shape-question.

**Source.**
- [Jazz docs: Groups as permission scopes](https://jazz.tools/docs/react/permissions-and-sharing/overview)
  (404 at fetch time; cached search snippet quotes
  "Jazz encrypts data with a shared read key (XSalsa20)... When someone is
  removed from a Group, the read key rotates")
- [Jazz CHANGELOG](https://github.com/garden-co/jazz/blob/main/CHANGELOG.md)

### §2.7 IPFS encryption patterns (general)

**Mechanism.** IPFS itself is unencrypted; encryption is layered ABOVE via
e.g. lit-protocol / Estuary / textile-hub. Most patterns are "encrypt-with-AES,
key-wrap with X25519, distribute key-wraps in metadata." No native sub-graph
shape primitive.

**Asymmetric-shape verdict.** **NO native.** Same pattern as Benten without
the `SubgraphSpec` walker.

**Benten-import-cost.** Benten is already STRUCTURALLY AHEAD of typical IPFS
patterns via the path-tagged-key + walker-as-Subgraph design. No import.

### §2.8 PESTO / Cwtch / Briar group-sharing

**Mechanism.** Forward-secret messaging-group protocols (Cwtch onion-routed +
Tor-DH; Briar relay-free Bluetooth/WiFi). All implement a FLAT group-key model
analogous to Jazz's Group — no sub-graph shape.

**Asymmetric-shape verdict.** **NO — flat group only.**

**Benten-import-cost.** None for shape (these are messaging, not data
sub-graph). Cwtch's metadata-hiding is relevant to Sealed-Sender (F21) but
not shape.

### §2.9 Yjs / Automerge encryption (Local-First Crypto 2024–2025)

**Mechanism.** Yjs encryption (`y-protocols/auth`) and Automerge encryption
(experimental) currently rely on per-document symmetric keys. Per-recipient
sub-document sharing is an OPEN PROBLEM in the local-first community — the
"Local-First Crypto" workshop at the Local-First Conference 2024 explicitly
named this gap.

**Asymmetric-shape verdict.** **NO — open problem.**

**Benten-import-cost.** None — but the gap is the same. Benten's
`SubgraphSpec` + path-tagged-keys design **IS a candidate answer for the
Local-First Crypto community's open question.** That's a coincidental
publishability claim, not a structural import.

### §2.10 Survey verdict

**No surveyed system natively expresses asymmetric per-recipient sub-graph
shape in a single grant** (Cryptree, Tahoe-LAFS, Jazz, Cwtch, IPFS-layers all
require multiple grants/groups). **CP-ABE expresses it via policy formulas but
at prohibitive structural cost** (pairing-based crypto; policy visibility;
revocation difficulty; bigger ciphertexts). **The pattern across local-first +
encrypted-fs systems is "per-shape Group/dirnode."**

**This is empirical confirmation that Benten's current state-of-the-art is
also the field's state-of-the-art.** It also says: the elegant move is NOT
to import a foreign primitive; it is to **expose the structural lever Benten
already has** (`combinators::union` over `SubgraphSpec`s) at the grant
boundary.

---

## §3 Task 2 — Evaluate elegant-shape candidates

### §3.1 Option C — Extend `RestrictedScope` per-Node termination set + property predicates

**Shape.** Add NAMED fields to `RestrictedScope`:
- `node_termination_set: Option<BTreeSet<Cid>>` — walker MUST stop at any
  listed Cid (Termination per the 4-thing thin core).
- `node_inclusion_set: Option<BTreeSet<Cid>>` — walker MUST visit only listed
  Cids (Inclusion).
- Richer `property_equalities` (regex, range, set-membership).

**Asymmetric shape closure.** PARTIAL. With `node_inclusion_set` you can
hand-list every Node Eve may walk — but this STATIC ENUMERATION breaks the
walker-as-Subgraph fractal property (the walker would need to consult an
enumerated set rather than walk-by-edge-label) AND defeats the structural
predicate philosophy.

**Tradeoffs.**
- + Strictly additive (`#[non_exhaustive]` already in place); no wire break.
- + Cheap to implement (~0.3 wave-days per field).
- – Static-enumeration approach scales O(|N|) in wire size; doesn't compose
  cleanly with walker-as-Subgraph.
- – Adding fields incrementally is the SLOW road to expressiveness
  ("predicate creep"); Spike H+1.1's REJECTION of OPAQUE-arm was partly
  motivated by avoiding this creep.
- – DOES NOT capture "Y-children-of-X1 + Z-children-of-X2" elegantly — you'd
  need to enumerate all Y-children + all Z-children, defeating the SPEC
  philosophy.

**Verdict.** **SUFFICIENT but NOT ELEGANT.** Defer as additive named slots
for narrow use-cases (e.g. `node_denial_set` for "block this specific Cid"
GDPR-style takedowns). NAMED-DEFER as candidate Option C-LITE if Option H is
declined.

### §3.2 Option D — Attribute-Based Encryption (CP-ABE)

Covered in §2.2. **DEFER — DISAGREE-WITH-EXPLANATION.** Pairing-based crypto
+ revocation difficulty + policy visibility + Inv-17 + Compromise #48 break
make this structurally incompatible with v1-FROZEN.

### §3.3 Option E — Per-Node access-tree expression

**Shape.** Each Node carries its OWN access-tree expression (similar to CP-ABE
but evaluated at the engine layer not the crypto layer). Recipient holds an
ATTRIBUTE SET; engine evaluates `node.access_tree.satisfies(recipient.attrs)`
to decide admission.

**Tradeoffs.**
- + Avoids ABE's crypto cost — pure predicate evaluation.
- – DOUBLES per-Node metadata storage cost (every Node carries its own
  policy fragment).
- – BREAKS Benten's "key-IS-the-access" property — admission becomes a
  RUNTIME PREDICATE EVALUATION not a CRYPTO PROPERTY. A recipient who somehow
  obtains `K(N)` for a Node whose access-tree they don't satisfy can still
  decrypt the bytes; admission control becomes server-mediated.
- – Composability with offline Drop bundles is dubious (recipient needs to
  trust the EVALUATOR; offline Drops are recipient-self-evaluating).

**Verdict.** **DEFER — DISAGREE-WITH-EXPLANATION.** The "key-IS-the-access"
property is load-bearing for Benten's encrypt-at-rest threat model
(SECURITY-POSTURE.md). Moving to predicate-evaluation-at-engine moves Benten
into the AWS-IAM / OPA / Cedar policy-engine category — a DIFFERENT product.

### §3.4 Option F — Selector-based addressing (producer evaluates)

**Shape.** Recipient gets a "selector expression" (e.g. JSONPath-like or
Datalog) that the PRODUCER evaluates per-Node to determine inclusion. The
selector is part of the grant; the producer pre-computes the inclusion set
on each new Node it produces and re-keys/re-shares accordingly.

**Tradeoffs.**
- + Very expressive (Datalog can express arbitrary monotone shapes).
- – Producer-side computation cost grows with selector complexity.
- – Recipient cannot offline-walk new Nodes without consulting the producer
  (or re-evaluating selector themselves over the full graph — which means
  they need the full graph to evaluate, defeating the partial-view goal).
- – The selector evaluator is a NEW evaluator-extension violating CLAUDE.md
  baked-in #1 (12-primitive irreducibility). Benten ALREADY has SubgraphSpec
  as data + walker as composed-from-12-primitives.

**Verdict.** **DEFER — DISAGREE-WITH-EXPLANATION.** Violates CLAUDE.md #1.
Option H subsumes this — `union` of multiple Specs IS a structural selector.

### §3.5 Option G — Committed access-set (Merkle tree of allowed Node-CIDs)

**Shape.** Producer pre-commits a Merkle tree T_Eve = MerkleRoot({Cid_1,
Cid_2, ...}) of every Cid Eve may access. Eve walks Benten + verifies each
Node's Cid is a leaf of T_Eve via Merkle proof.

**Tradeoffs.**
- + Audit-friendly: T_Eve is a single root commitment; admin can verify
  Eve's access shape by single hash.
- + Static + simple cryptographic semantics (no new KEM, no ABE).
- – Pre-commits FUTURE Nodes too — at issuance time the producer must
  enumerate every Node Eve will EVER access. Defeats incremental-Atrium
  growth. For an Atrium that grows over time, producer must RE-ISSUE T_Eve
  on every new Node addition.
- – Doesn't compose with `SubgraphSpec`-style RULES — Eve gets a STATIC
  enumeration, not a RULE.
- – Wire-size O(|N|) in the enumerated set; doesn't shrink for "X +
  Y-subtree" style shapes.

**Verdict.** **DEFER — DISAGREE-WITH-EXPLANATION** as primary mechanism.
**NAMED-DEFER** as a sibling-feature: T_Eve can ride alongside SubgraphSpec
as an AUDIT-COMMITMENT — Producer commits to "Eve's grant covers exactly this
shape AT THIS MOMENT" via a Merkle root over the BFS-canonical walk result,
for AUDIT purposes (Task 3 §4.2). This is a clean addition to F25
(audit-deliverable pack) without becoming the primary mechanism. Revisit-
trigger: regulatory audit requirement emerges in Phase-5+ Garden tier.

### §3.6 Option H — Nested-Spec (THE RECOMMENDATION)

**Shape.** Promote `SubgraphSpec::union` from internal combinator to
grant-time composition primitive. `RestrictedSelector` arm of `Scope` carries
a `RestrictedScopeSet` instead of a single `RestrictedScope`:

```rust
// Before (V1-FROZEN §15.c at HEAD):
pub enum Scope {
    Hashes(Vec<Cid>),
    RestrictedSelector(RestrictedScope),
}

// After (Option H — minimal-delta extension):
pub enum Scope {
    Hashes(Vec<Cid>),
    RestrictedSelector(RestrictedScopeSet),
}

pub struct RestrictedScopeSet {
    /// Disjunction (OR) of restricted scopes. Recipient may walk under
    /// ANY of these scopes; the effective sub-graph is the structural
    /// union. EXACTLY-ONE shape for backward-compat: a single-element
    /// Vec is wire-equivalent to the old single-RestrictedScope shape
    /// IF we choose to wire-encode a one-element vec identically (codepoint-
    /// dispatched per CLAUDE.md #5).
    pub scopes: Vec<RestrictedScope>,
    /// Optional Merkle audit-commitment (Option G as sibling-feature; see §3.5).
    pub audit_commitment: Option<Cid>,
}
```

**Why this is the elegant move:**

1. **The structural lever already exists.** `combinators::union` is
   proptest-verified containment-preserving. The change is exposing it at the
   wire boundary, not inventing a new primitive.
2. **Asymmetric shape ⇒ disjunction of single-root scopes.** "X +
   Y-children-of-X1 + Z-children-of-X2" =
   `RestrictedScopeSet { scopes: [scope_X_depth1, scope_X1_Y, scope_X2_Z] }`.
   Each sub-scope is a STANDARD `RestrictedScope` over its own root.
3. **Per-recipient KEYING composes orthogonally.** Recipient walks each
   sub-scope under the existing path-tagged `K(N) = HKDF(K(predecessor), ...)`
   chain. No new key derivation, no ABE, no Cryptree CK restructuring.
   `AuthorizationGrant.key_material` carries the K-set for the union'd walk.
4. **Containment + intersection are well-defined.**
   `RestrictedScopeSet(A).contains(RestrictedScopeSet(B))` ⇔ ∀ b ∈ B.scopes,
   ∃ a ∈ A.scopes such that `a.contains(b)`. This is the
   "set-of-sub-specs" containment lift; it's decidable, transitive, reflexive
   — same algebra as the per-scope containment. UCAN attenuation rule:
   delegated `RestrictedScopeSet` must be contained-in the parent — i.e. every
   delegated sub-scope is contained by SOME parent sub-scope.
5. **Spike H+1.1 OPAQUE-arm REJECTION is preserved.** The change is NOT
   adding an opaque-refinement-witness arm; it is exposing a STRUCTURAL
   combinator. The witness-binds-attestation-not-spec-body
   structural unsoundness named in Spike H+1.1 §b.SEC #4 does NOT apply —
   each sub-scope is itself a fully-structured `RestrictedScope`, decidable
   in all 6 dimensions.
6. **EXACTLY-2-arm `Scope` enum preserved.** The change is INSIDE the
   `RestrictedSelector` arm; no third arm. HALT-AND-SURFACE-TO-BEN escape
   valve preserved.
7. **Backward compatibility.** A single-element `RestrictedScopeSet` is
   wire-equivalent to the old single-`RestrictedScope` shape under a
   codepoint-discriminated encoding (single-scope codepoint = old shape;
   multi-scope codepoint = new shape). Migration: bump the codepoint and
   accept both for one release.
8. **Walker-as-Subgraph fractal pin preserved.** The walker over a
   `RestrictedScopeSet` is `union(walk(scope_1), walk(scope_2), ...)`
   composed via `combinators::union`. The composed walker IS still a Subgraph
   per `walker_as_subgraph()`.
9. **AuthorizationGrant binding-sig closure.** `binding_sig` covers
   `(ucan, key_material, audience)` — `ucan` carries the new
   `RestrictedScopeSet` body; binding sig closes over canonical bytes of the
   set. No new signature primitive.

**Drawbacks.**
- Wire-size grows linearly with number of sub-scopes (but recipient typically
  has 1–5 sub-scopes; not a scaling concern under Compromise #46's
  per-Kind recipient-count ceiling).
- Cite-drift cross-language mirror (§3.5g): TS-side mirror must encode the
  same RestrictedScopeSet shape. Bounded ~50 LOC TS-side.
- New canonical-CBOR encoding for the Vec — needs a TLV-length-injectivity
  test pin (F3 family).

**Tradeoffs against Benten's specific constraints.**

| Constraint | Option H compatibility |
|---|---|
| **v1-beta freeze** | EXTENDS frozen surface additively via codepoint-dispatch; no break. The `Scope::RestrictedSelector` arm changes its inner type; codepoint-discriminated old-vs-new is the safe migration. **CONFIRM: needs explicit V1-FROZEN-INTERFACE.md §15.c amendment with old-shape codepoint reserved + new-shape codepoint minted.** |
| **CLAUDE.md #5 crypto-agility** | No crypto change; pure structural composition. **PASS.** |
| **CLAUDE.md #1 12-primitive irreducibility** | Spec stays data; walker stays composed-from-12-primitives. Union is structural-graph combinator (already shipping). **PASS.** |
| **Forkability semantics** | Per-recipient sub-graph grants are grant-layer concerns; Atrium-fork rotates `K_Atrium` and re-issues grants under the new K. RestrictedScopeSet survives the fork as a RE-ISSUED grant (same as single-scope grants today). **PASS — same as status quo.** |
| **Integration with K(N) chain** | Each sub-scope walks under the existing `K(predecessor) → K(N)` chain. Multiple roots ⇒ multiple `K(root)` derivations, each per Spike-E §15.f. **PASS.** |
| **Integration with MembershipSet** | Orthogonal: `(MembershipSet, RestrictedScopeSet)` tuple at seal seam. MembershipSet decides RECIPIENTS-share-K_Set; RestrictedScopeSet decides which sub-graphs each recipient may walk. **PASS — clean orthogonality preserved.** |

---

## §4 Task 3 — Specific design questions

### §4.1 Re-keying on shape change

**Question.** If Eve's grant changes (more shape; less shape), do existing
Node-keys still work or does the producer need to re-key?

**Answer under Option H.** **No re-keying needed for shape MUTATION.** The
path-tagged keys `K(N) = HKDF(K(predecessor), info="step"||edge_label||N.cid)`
are determined by the WALK PATH, not by the grant. If Eve's grant widens
("add Z-children-of-X2"), the producer issues a NEW AuthorizationGrant whose
`key_material` includes the K-set for the new sub-scopes — the underlying
`K(N)` for any Z-child Node Eve newly reaches is HKDF-derivable from her
`K_principal` through the canonical path; the producer just hands over the
intermediate keys.

If Eve's grant narrows ("remove Y-children-of-X1"), the existing `K(N)` for
Y-children Eve already saw are STILL VALID for already-derived data (this is
Compromise #31 — revocation reach in encryption-at-rest; documented OPEN
architectural trade-off). New writes against the narrowed shape REKEY only
if combined with Atrium-fork — and Atrium-fork is the M-CONS-ratified
mechanism for that (FORK-ONLY rotation per F17).

**So:** shape mutation is RE-GRANT, not RE-KEY. Aligns cleanly with
V1-FROZEN §15.i revocation-reach documentation.

### §4.2 Auditability — proving "I have access to this shape" to a 3rd party

**Question.** How does Eve prove to an admin "I have access to shape S"?

**Answer under Option H.**
1. **Grant-presentation.** Eve presents her `AuthorizationGrant` — the admin
   verifies `binding_sig`, decodes `ucan.scope.RestrictedSelector` ⇒
   `RestrictedScopeSet`, and reads the disjunction of sub-scopes directly.
   The set IS the proof (signed by issuer).
2. **Walk-witness.** Eve runs the walker over `RestrictedScopeSet`, captures
   the BFS-canonical enumeration, and produces a Merkle root over the
   `WalkResult.enumerated: Vec<(Cid, StructuralPath)>` (per V1-FROZEN §15.h).
   Admin re-walks the same set + checks the Merkle root matches. This is the
   Option G sibling-feature — `audit_commitment: Option<Cid>` field on
   `RestrictedScopeSet` is the optional pre-commit hook.
3. **Containment proof.** Admin verifies `RestrictedScopeSet(Eve).contains(
   RestrictedScopeSet(target_shape))` via the per-scope containment algebra
   — fully decidable.

The audit story is STRICTLY BETTER than today: today's single-`RestrictedScope`
shape requires multiple grants for asymmetric shapes, and the admin must
aggregate them; Option H produces ONE grant with the shape in canonical bytes.

### §4.3 Compose with Atrium-fork

**Question.** When Atrium forks, do per-recipient sub-graph grants survive
the fork?

**Answer.** **Yes, with RE-ISSUANCE** — same as the status quo. Atrium-fork
rotates `K_Atrium` (per M-CONS F17 + M2 §4 op-table fork); ALL existing
grants under the pre-fork K_Atrium must be RE-ISSUED under the post-fork
K_Atrium for forward access. The `RestrictedScopeSet` body of the grant is
INVARIANT under fork (it's a structural specification); only the
`key_material` half rotates.

Concretely: pre-fork Eve has Grant_v1 with `RestrictedScopeSet S` +
`key_material K_v1`. Post-fork, producer re-issues Grant_v2 with the SAME `S`
+ new `key_material K_v2`. The fork-survival pattern is identical to today.

Compromise #52 (no PCS against removed members) and Compromise #48
(MembershipSet-shape-leak) apply unchanged.

### §4.4 Compose with Path-A.5 immutable Version-Node-CIDs

**Question.** How does sub-graph sharing of mutable Anchor+Version+CURRENT
graph compose with the new Version-Node-CID after CRDT merge?

**Answer.** Path-A.5's discipline: `K` keys to IMMUTABLE Version-Node-CIDs;
CURRENT is a MUTABLE POINTER to a Version-Node-CID. Sub-graph sharing under
Option H targets `roots: BTreeSet<Cid>` of Version-Node-CIDs; Eve walks
the IMMUTABLE Version-Node from those roots.

**Post-CRDT-merge.** After Loro CRDT merge produces a new Version-Node-CID
V_new, the producer has two options:
1. **Re-issue grant against V_new.** New grant carries
   `RestrictedScopeSet { scopes: [scope_with_root_V_new, ...] }`. Strict
   freshness: Eve sees only the merged-state.
2. **Use Anchor-CID as the root + carry CURRENT-pointer-as-walk-step.**
   This is OUT-OF-SCOPE for v1-beta — Anchor-CID-as-root requires the
   walker to resolve Anchor → CURRENT → Version at walk-time, which violates
   the path-tagged-K(N) invariant (key would depend on a MUTABLE pointer).

**v1-beta answer:** Option (1). Producer re-issues grants on Version
advancement. This is the M-CONS Inv-20 clause-f "Path-A.5 K(V) discipline at
API boundary (type-restricts payload to immutable Version-Node-CID)." Option
H is compatible — the RestrictedScopeSet just lists Version-Node-CIDs.

**v1.x+ answer (NAMED-DEFER):** explore an Anchor-rooted scope that resolves
CURRENT at walk-start (one-shot resolution; the rest of the walk is over the
immutable Version subgraph). Revisit-trigger: producer re-issuance cost is
identified as a UX pain-point in Phase-4-Meta-Composing usage.

### §4.5 Compose with multi-stanza HPKE per L9 A1

**Question.** When sharing same content to N recipients with N different
sub-graph shapes, do they get N HPKE stanzas with N different K-sets?

**Answer.** **Yes, but the K-sets COMPOSE STRUCTURALLY** rather than being
N independent random keys.

Under Option H, each recipient's `AuthorizationGrant.key_material` contains
the K-set derived for their `RestrictedScopeSet`. Because the K(N) chain is
PATH-TAGGED, two recipients walking the SAME Node N via the SAME path will
end up with the SAME K(N) (which is the point of MembershipSet — they share
ciphertext bytes and dedup-blind-CID per Inv-20).

For SHAPE-DIFFERENT recipients, the K-set Eve gets is a SUBSET of the K-set
Charlie gets (where Eve.shape ⊆ Charlie.shape) or partially OVERLAPS (where
the shapes intersect but neither contains the other).

**Multi-stanza HPKE per L9 A1 carries N stanzas** (one per recipient),
each stanza carrying a recipient-specific WRAPPED-K-SET payload. The
multi-stanza framing per F4 + F5 + F17 is unchanged; what's in each
stanza's wrapped payload may differ per recipient.

**Net cost:** O(N) stanzas (same as today) × O(|shape|) per stanza payload.
Compromise #46 ceiling per-Kind (Atrium 32; DeviceMesh 5) applies unchanged.

---

## §5 Task 4 — Recommendation

### §5.1 Pick: Option H

**Option H-NESTED-SPEC is the elegant winner.** Per
`feedback_extra_reflection_pass_for_elegant_permanent_shape.md`:

- **Single elegant structural shape closing N findings at once.** Closes the
  asymmetric-shape limitation Ben surfaced WITHOUT minting a new primitive.
  Reuses the already-shipping `combinators::union` + path-tagged `K(N)` +
  proptest-verified containment-preserving algebra.
- **Strictly less code than alternatives.** ~80 LOC for the
  `RestrictedScopeSet` type + ~50 LOC for the codepoint-dispatch migration +
  ~30 LOC TS mirror + 2 doc rows (V1-FROZEN §15.c amendment + SECURITY-POSTURE
  §S&C amendment). Total ~160 LOC. Compare: Option D ABE ~2000+ LOC + pairing
  crypto crate dependency; Option E access-tree ~1000+ LOC + new evaluator;
  Option F selector ~800+ LOC + new evaluator; Option G Merkle-set ~400 LOC +
  audit-side-only.
- **Forward-class-of-bug closure.** The structural lift `single Scope →
  set-of-Scopes` is the ONCE-AND-DONE generalization. Future shape requests
  (e.g. "Eve gets nested Atrium-of-Atriums sub-shapes") compose under the
  same union/intersect algebra without further lifts.
- **Preserves all 5 frozen-surface disciplines:** EXACTLY-2-arm Scope enum
  preserved; OPAQUE-arm REJECTED preserved; walker-as-Subgraph fractal pin
  preserved; path-tagged K(N) chain preserved; AuthorizationGrant binding-sig
  closure preserved.

### §5.2 NAMED-deferred alternatives (per HARD RULE clause-b)

| Alternative | Status | Revisit-trigger |
|---|---|---|
| **Option C-LITE** — incremental NAMED dimensions (`node_termination_set`, `node_denial_set`) | NAMED-DEFER to V1-FROZEN-INTERFACE-DEFERRED.md Row D-NEW-1 (alongside the §15.b extension slots already named) | A specific named use-case emerges (e.g. GDPR per-Cid denial; Phase-5 Gardens-recursion) that's NOT cleanly expressible as a union of sub-scopes. |
| **Option G-AUDIT-SIBLING** — Merkle-commitment of BFS-canonical walk result as audit field on `RestrictedScopeSet.audit_commitment: Option<Cid>` | NAMED-DEFER to Audit Wave (post-v1-GM); ride alongside F25 audit-deliverable pack | Regulatory-audit requirement emerges in Phase 5+ Garden tier (e.g. SOC2-Type-II-like). |
| **Option D — CP-ABE** | DEFER — DISAGREE-WITH-EXPLANATION per HARD RULE clause-c | Stable attribute vocabulary (~5–8 categories) + asymmetric-shape becomes >30% of grants + pairing-crypto-PQ-story matures. None hold at v1-beta or v1-GM. |
| **Option E — per-Node access-tree** | DEFER — DISAGREE-WITH-EXPLANATION | Benten pivots from "key-IS-the-access" to predicate-engine model. Not anticipated. |
| **Option F — selector-evaluator** | DEFER — DISAGREE-WITH-EXPLANATION | Violates CLAUDE.md #1. Subsumed by Option H (union IS a structural selector). |
| **Option G as PRIMARY** (committed enumeration) | DEFER — DISAGREE-WITH-EXPLANATION as primary; ADOPT as audit-sibling per row 2 above | Incremental-Atrium-growth model abandoned. Not anticipated. |
| **Anchor-rooted scope** (resolve CURRENT at walk-start) | NAMED-DEFER to Phase-4-Meta-Composing exploration | Producer re-issuance cost identified as UX pain-point during Phase-4-Meta-Composing. |

### §5.3 Cost estimate vs current restricted-spec design

| Wave-item | Cost (wave-days) | Notes |
|---|---|---|
| `RestrictedScopeSet` type + canonical-CBOR encoding | +0.3 | ~80 LOC + 1 proptest pin (containment lift) |
| Codepoint-dispatch migration (old single-scope codepoint + new set codepoint) | +0.2 | F1 family extension; 1 cite-drift scanner row |
| `Scope::RestrictedSelector` arm inner-type swap | +0.1 | Trivial; 1 typed-reject test pin |
| AuthorizationGrant binding-sig closure check | +0.1 | Existing signature path; 1 e2e pin |
| TS-side mirror (§3.5g cross-language rule-mirror) | +0.1 | ~30 LOC TS |
| V1-FROZEN-INTERFACE.md §15.c amendment | +0.1 | Doc-only |
| SECURITY-POSTURE.md amendment (asymmetric-shape closure + per-Compromise-#48 sharpening) | +0.1 | Doc-only |
| MembershipSet-Spec doc cross-link (orthogonality clarification) | +0.05 | Doc-only |
| Audit-deliverable pack F25 sub-row (audit_commitment field; Option G-AUDIT-SIBLING) | +0.1 | NAMED-DEFER landing; can ship in same wave or defer |
| **Subtotal** | **+0.95 to +1.2 wave-days** | Modest LOAD-BEARING extension |

vs. baseline (M-CONS §2 wave-day accumulation, ~78–100 wave-days): **+1% wave-day
delta**; **~160 LOC code; ~150 LOC doc.** This is in the same scale as F-A1
(+0.2 wave-days) + F-A2 (+0.5 wave-days) M-CONS sharpenings — i.e. a small
ELEGANCE-EXTENSION amendment, not a new wave.

### §5.4 R0 plan-doc implications

If Ben ratifies, the R0 plan-doc gets the following amendments:

1. **§2 Final-final amendment registry — add F28:**
   `F28 | RestrictedScopeSet (sub-graph-asymmetric-shape; SubgraphSpec-union
   exposed at grant boundary; backward-compat via codepoint-dispatch) | LB |
   YES | v1-beta-LB | F8, F17 | benten-caps::scope + RestrictedScope + V1-FROZEN
   §15.c amendment | HIGH | N1 §3.6 + g-core-3w combinators @ 5c2947c8`.

2. **§3 Compromise # registry — no new mints.** The existing #31 (revocation
   reach) + #46 (HPKE per-Kind recipient ceiling) + #48 (MembershipSet-shape-
   leak) all apply unchanged. The asymmetric-shape closure surfaces NO new
   negative architectural trade-off.

3. **§4 Invariant set — extend Inv-20 clause-h (M-CONS extension):**
   "homogeneous-per-Kind MemberKey variant" becomes
   "homogeneous-per-Kind MemberKey variant; RestrictedScopeSet-as-disjunction-
   of-RestrictedScope at Scope::RestrictedSelector arm (union-algebra-
   containment-lifted)."

4. **§6 Wave-sequencing — fold into Wave-MS-PRIMITIVE.** Option H lands in
   the SAME wave as the MembershipSet primitive crate (canary-first; +1.0
   wave-day fold absorbed within Wave-MS-PRIMITIVE's ~3.5–4.5 wave-day
   sequential canary).

5. **§7 Cost re-estimate — net delta:** +0.95 to +1.2 wave-days; M-CONS net
   "–0.8 to –2.0 wave-days" becomes "**+0.15 to –0.8 wave-days**" — still
   within the ~80–101 baseline noise floor.

6. **§8 R0 plan-doc skeleton — add new §17.x row:**
   `§17.X RestrictedScopeSet — sub-graph asymmetric-shape closure` with
   the design + 6 frozen-surface preservation pins + 1 backward-compat
   codepoint-dispatch row.

7. **§9 Open Ben-call decisions — add D8 (my-pred RATIFY):**
   "D8 — Ratify Option H (RestrictedScopeSet) as the asymmetric-shape
   closure mechanism, OR keep status quo (multiple grants per shape) +
   accept the limitation as documented Compromise. **my-pred: RATIFY Option H
   bundled with F17 in Wave-MS-PRIMITIVE.** Net cost +0.95–1.2 wave-days;
   strictly additive; preserves all 5 frozen-surface disciplines; the
   structural lever (union combinator) already ships."

8. **§10 Pattern-induction — Pattern P-N1-1:**
   "**Already-shipping combinator surfaces are the cheapest source of grant-
   layer expressiveness.** When asked 'should we add a new primitive for
   X?', first audit whether an existing combinator-surface (intersect /
   union / filter / map) can be promoted from internal to wire-layer.
   Discovered via N1 sub-graph-sharing survey 2026-05-28; codifies an
   instance of the broader 'engine-primitives-vs-application-layer'
   discipline `feedback_engine_primitives_vs_application_layer.md`."

---

## §6 Sibling-panel composability (M-CONS + N2/N3/N4)

This N1 doc is one of 4 parallel post-M-CONS refinement specialists. I do
NOT know what N2/N3/N4 are dispatched against; the brief named me as "N1 of
4." The Option H recommendation is COMPATIBLE with arbitrary sibling-panel
outputs because it operates at a different layer:

- **Orthogonal to MembershipSet kind/policy/transport.** Option H modifies
  `Scope` (UCAN-cap layer); MembershipSet modifies `K_Set` distribution
  (HPKE-stanza layer). They compose by tuple at the seal seam.
- **Orthogonal to CGKA reservation (F13 + AtriumWithCGKA codepoint).**
  Whether we eventually ship CGKA does not change the SHAPE algebra.
- **Orthogonal to TransportConfig (F27).** Wire-transport does not affect
  scope shape.
- **Orthogonal to MembershipEvent (F-A2).** Audit-event shape per-Kind
  is independent of per-recipient sub-graph shape.

If a sibling panelist proposes a different mechanism (e.g. N2 might propose
a `ScopeRule` predicate-DSL), the elegant-shape pass at M-DEC convergence
should ask: does the proposal preserve all 5 frozen-surface disciplines AND
land in strictly-less-code than Option H? My-pred: no proposal will beat
~160 LOC + zero-new-primitive + zero-crypto-change.

---

## §7 Self-assessment

**Strengths.**
- The "structural lever already ships" discovery (combinators @ 5c2947c8) is
  the load-bearing finding. Without that g-core-3w branch existing, Option H
  would require minting `union` from scratch and would be a structurally
  larger change.
- Literature survey converges on the "no system natively expresses asymmetric
  shape" finding across Cryptree / Tahoe-LAFS / Jazz / Cwtch / IPFS-layers /
  Yjs+Automerge, which CONFIRMS Benten is on field-state-of-the-art and the
  elegant move is to expose an internal structural lever, not import a
  foreign primitive.
- All 5 frozen-surface preservation pins are explicit (§3.6 H drawbacks +
  §4.1–§4.5 design-questions table).

**Weaknesses.**
- I did not verify the codepoint-dispatch migration is byte-clean against
  the existing wire-format. The §15.c codepoint reservation table needs
  cite-drift verification post-ratification. Bounded; resolvable in
  Wave-MS-PRIMITIVE.
- I did not enumerate `RestrictedScopeSet`'s interaction with EVERY one of
  the 28 unified amendments — only the load-bearing ones (F1/F8/F17/F18/F25
  + AuthorizationGrant + binding-sig + walker-as-Subgraph). A full pass
  through F1–F27 + F-A1/F-A2 for Option H interaction would harden the
  cost estimate; deferred to M-DEC convergence.
- The Jazz `permissions-and-sharing/overview` URL 404'd at fetch; my Jazz
  characterization is drawn from the encryption-doc + CHANGELOG snippets
  the WebSearch returned + the Jazz docs I've previously read. If a
  sibling panelist has direct access to current Jazz docs, the §2.6
  characterization should be cross-checked.
- I did not run a primary-source PDF extraction of the Cryptree paper (the
  binary PDF could not be auto-extracted by WebFetch). The Cryptree
  characterization is via the SlideToDoc summary + ACM DL abstract + the
  search-result aggregation. Confidence: MED-HIGH; the 5-key structure
  + lazy revocation + recursive-grant constant-time properties are
  unambiguously documented across multiple secondary sources.

**Confidence floor.** HIGH on the Option H direction (proptest-verified
union combinator already ships); MED-HIGH on the wave-day cost (+0.95 to
+1.2); MED on the §5.4-R0-plan-doc enumeration (depends on sibling-panel
convergence).

**HARD RULE 12 disposition self-check.**
- Option C-LITE: NAMED-DEFER (clause-b) — destination row + revisit-trigger.
- Option D/E/F/G-primary/Anchor-rooted: DISAGREE-WITH-EXPLANATION (clause-c)
  with reason cited.
- Option H: ADOPT-NOW (recommendation).
- No "later" / "TBD" / "Phase-N follow-up" / phantom-destination tokens
  in this doc. ✓

**Plain-English lens self-check** (per
`feedback_plain_english_surfaces.md`): plain-situation = Ben surfaced
asymmetric-shape limitation; concrete-options = C/D/E/F/G/H; my-prediction =
RATIFY Option H bundled with F17; confirm-or-redirect at M-DEC. ✓

---

## §8 Citations

### §8.1 Primary in-tree (Benten)

- `crates/benten-core/src/subgraph_spec/spec.rs` (4-thing thin core)
- `crates/benten-core/src/subgraph_spec/walker.rs` (BFS walker;
  `walker_as_subgraph()` fractal pin at line 183)
- `crates/benten-core/src/subgraph_spec/combinators.rs` (intersect/union/
  filter; g-core-3w branch @ 5c2947c8) — **the load-bearing structural lever**
- `crates/benten-caps/src/restricted_spec.rs:103` (`RestrictedScope` 6-dim)
- `crates/benten-caps/src/scope.rs:46` (`Scope::{Hashes | RestrictedSelector}`)
- `crates/benten-caps/src/authorization_grant.rs:238` (`AuthorizationGrant`)
- `crates/benten-crypto-suite/src/structural_kdf.rs` (HKDF-SHA256 derive_step
  + derive_root; Spike-E Interpretation-B)
- `docs/V1-FROZEN-INTERFACE.md` §15.a–§15.i (the full frozen-surface set)
- `docs/SECURITY-POSTURE.md` §S&C + Compromise #31 (revocation reach)
- `.addl/phase-4-meta/membership-set-m-cons-consolidator.md` @ 74580ee6
  (the M-CONS reference doc)
- `.addl/phase-4-meta/membership-set-m2-primitive-design.md` @ 6170980b
  (M2 primitive design)
- `.addl/phase-4-meta/membership-set-cataloger-m1b-atrium-membership-sharing.md`
  @ 1816ea60 (M1b cataloger)

### §8.2 Memory rules consulted

- `feedback_extra_reflection_pass_for_elegant_permanent_shape.md` —
  Ben-ratified 2026-05-25 R6 R2; this N1 doc explicitly applies the
  "one elegant structural shape closing N findings" pass.
- `feedback_engine_primitives_vs_application_layer.md` — push
  application-layer composition before engine extension. Option H is
  EXACTLY this discipline applied at the grant-wire boundary.
- `feedback_no_defer_HARD_RULE.md` — HARD RULE 12 dispositions explicit
  in §5.2 + §7 self-check.
- `feedback_plain_english_surfaces.md` — plain-situation +
  concrete-options + my-prediction + confirm-or-redirect structure
  preserved in §0 + §5 + §7.

### §8.3 Literature (external)

- **Cryptree.** Grolimund, Meisser, Schmid, Wattenhofer, "Cryptree: A
  Folder Tree Structure for Cryptographic File Systems," SRDS '06.
  [ACM DL](https://dl.acm.org/doi/10.1109/SRDS.2006.15) ·
  [ETH PDF](https://tik-db.ee.ethz.ch/file/146566189b90f952b8ab1dcf98010781/srds06.pdf) ·
  [SlideToDoc summary](https://slidetodoc.com/cryptree-a-folder-tree-structure-for-cryptographic-file/) ·
  [ResearchGate](https://www.researchgate.net/publication/220960841_Cryptree_A_Folder_Tree_Structure_for_Cryptographic_File_Systems)
- **CP-ABE.** Bethencourt, Sahai, Waters, "Ciphertext-Policy Attribute-
  Based Encryption," IEEE SP '07.
  [UT Austin PDF](https://www.cs.utexas.edu/~bwaters/publications/papers/cp-abe.pdf) ·
  [UCLA PDF](http://web.cs.ucla.edu/~sahai/work/web/2007%20Publications/SSP2007.pdf) ·
  [Eprint 2008/290](https://eprint.iacr.org/2008/290.pdf)
- **PRE.** Ateniese, Fu, Green, Hohenberger, "Improved Proxy Re-Encryption
  Schemes with Applications to Secure Distributed Storage," NDSS '05.
  [PDF](https://spqrlab1.github.io/papers/ateniese-proxy-reenc-ndss05.pdf) ·
  [Eprint 2018/426](https://eprint.iacr.org/2018/426.pdf) (adaptive
  security)
- **Tahoe-LAFS.**
  [Capabilities wiki](https://tahoe-lafs.org/trac/tahoe-lafs/wiki/Capabilities) ·
  [Directory Nodes spec](https://tahoe-lafs.readthedocs.io/en/latest/specifications/dirnodes.html) ·
  [URIs spec](https://tahoe-lafs.readthedocs.io/en/tahoe-lafs-1.12.1/specifications/uri.html)
- **UCAN / Fission / ZCap.**
  [UCAN spec](https://ucan.xyz/specification/) ·
  [Fission UCAN guide](https://fission.codes/blog/a-guide-to-ucans/) ·
  [ucan-wg/delegation](https://github.com/ucan-wg/delegation) ·
  [W3C-CCG ZCap-LD](https://w3c-ccg.github.io/zcap-spec/)
- **Jazz CoValue.** [Jazz docs root](https://jazz.tools/docs/react/reference/encryption)
  (URL 404'd at fetch; secondary characterization via search snippets) ·
  [Jazz CHANGELOG](https://github.com/garden-co/jazz/blob/main/CHANGELOG.md)
- **PRE + graph DB.** "Privacy-Preserving Multi-User Graph Intersection
  Scheme for Wireless Communications in Cloud-Assisted Internet of
  Things," MDPI Sensors 25(6):1892, 2025.
  [Article](https://www.mdpi.com/1424-8220/25/6/1892)
- **Multi-recipient threshold encryption.** arXiv:2210.06889 (hidden-
  multiplier scheme) — surveyed for completeness; not Benten-applicable.
- **Multiparty Selective Disclosure via ABE.** arXiv:2505.09034v1 (May
  2025) — confirms ABE is the standard hammer for asymmetric-shape, and
  the ciphertext-size + revocation drawbacks at v1-beta scale.

---

**End N1 doc.**
