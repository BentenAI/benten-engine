# P4-FF UX Architecture — RestrictedScopeSet × Path-A.5-Immutability Composition

> **HANDOFF top-banner re-orient for fresh-agent-post-compact (foundational discipline per
> `feedback_handoff_top_banner_re_orient.md`):** this document is the UX-shapes survey for
> MembershipSet M-C2-v2 Scenario F-F failure + Blind spot B-1 (RBAC × RestrictedScopeSet ×
> Path-A.5 immutable Version-Node-CID). It evaluates 7 candidate UX shapes (A–G) for
> retraction semantics when a grant references content-addressed immutable Subgraphs whose
> "removal" mints new Version-Node-CIDs the grant cannot see. Pairs with N3 §2.3 / M-CONS-v2
> §1.3 #55-GDPR-RTBF "P2P-by-design honest-architectural-disclosure" pattern; this is the
> Path-A.5-immutability sibling of #55. Read BEYOND what's named here if you arrive cold:
> M-CONS-v2 at `e62ff540`, M-C2 critique at `0533d0cf` §1 F-F + B-1, M-C3 fresh-eyes at
> `cac10631`, N1 RestrictedScopeSet at `ed592770`, Path-A.5 specialist at `5f50a028`, N3
> #55 framing at `298c80d9`, plus CLAUDE.md baked-in #18 (Version-Node-CID immutability),
> #19 (plugin + extension trust + UCAN delegation), MEMORY.md feedback rules
> (extra-reflection-pass for elegant permanent shape; HARD-RULE no-defer; engine-primitives-
> vs-application-layer; surface-arch-decisions-under-broad-auth).
>
> Tree HEAD at write: `2172cb6d` (origin/main, fetched + verified at session start).
> Branch: `phase-4-meta-core/membership-set-p4-ff-restrictedscopeset-immutability-ux`.
> Date: 2026-05-28.

- **Role.** UX-privacy architect — multi-shape survey + chosen-default justification +
  Compromise #57 mint text + R0 plan-doc deltas.
- **Posture.** ADVISORY. Surface ALL viable shapes, score, recommend a winner per
  `feedback_extra_reflection_pass_for_elegant_permanent_shape`, then catalog the dominated
  alternatives as NAMED-deferred-with-trigger per HARD RULE clause-b (not "later").
- **Out-of-scope.** Per-message FS (#56 EphemeralRatchet territory); per-subject crypto-
  shredding for GDPR-RTBF at Atrium-data layer (#55 territory; orthogonal). This doc is
  about per-recipient SUB-GRAPH grant retraction, not per-subject content erasure.
- **Inputs.** All artifacts named in the top banner; WebSearch on Tahoe-LAFS read-caps,
  Signal disappearing messages, IPFS pinning, UCAN revocation spec, Apple Family Sharing,
  Matrix Megolm key invalidation, Google Drive shared-link revocation, Spotify Padlock
  per-user-key crypto-shredding.

---

## §1 Executive recommendation + headline framing

### §1.1 The failure (M-C2 F-F + Blind spot B-1) restated in UX terms

Alice (admin) grants Eve a `RestrictedScopeSet` for "X subgraph + Y-children-of-X1" via
N1's elegant 4-thing thin core + nested-spec union. Per Path-A.5 (CLAUDE.md baked-in #18 +
Inv-13), the grant's `roots: Vec<Cid>` references **immutable Version-Node-CIDs**: X is
some `Cid_X@t0`; X1 is `Cid_X1@t0`. Eve's grant carries these CIDs by-value, plus the
walker semantics + path-tagged K(N) chain to derive read-keys.

Alice now decides "remove the Y-child from under X1." Per Path-A.5, this writes a NEW
immutable Version-Node `Cid_X1@t1` whose subtree doesn't include Y, then atomically
advances the CURRENT-edge of X1's Anchor from `Cid_X1@t0` to `Cid_X1@t1`. **Eve's grant
still references `Cid_X1@t0`.** Per Inv-13 + Sec-6 (Version-Nodes immutable forever),
`Cid_X1@t0` continues to be resolvable; Eve continues to derive the K(N) chain for the
OLD subtree forever, including the Y-child Alice believed she "removed."

Alice's UX expectation (drawn from Google Drive, Slack, Discord, every cloud-centralized
permission model she's used for 20 years): "remove → it's gone." Reality under Path-A.5
+ N1 + content-addressed P2P: "remove → newest pointer doesn't reach it; old pointer
Eve was handed still reaches it forever; storage hosts (other Atrium peers, Drop hosts,
DHT pin caches) retain the old Version-Node bytes."

The gap is **not a crypto bug**. The gap is the gap between Alice's UX mental model and
the substrate guarantee Benten makes via Path-A.5. It is the exact-same class of gap that
#55 GDPR-RTBF surfaced for per-subject content erasure ("P2P-by-design semantic; strong-
RTBF lives at application layer"), promoted to the per-recipient sub-graph grant layer.

### §1.2 Headline recommendation — winner + alternatives

**Pick Shape H = (A + D + C with role assignments by use-case).** Specifically:

- **DEFAULT for ordinary grants = Shape A (honest-disclosure)** + Shape C semantic
  ("forward-stable view; CRDT-merges after grant-time may shift Atrium-current-pointer
  without retracting your view"). This is the structurally-correct framing per Path-A.5;
  it is the Compromise #57 mint per M-C2 R-MCV2-(B-1); it composes orthogonally with #55
  GDPR-RTBF P2P-by-design pattern.

- **OPT-IN for sensitive grants = Shape D (crypto-shredding via per-grant K(grant) seal).**
  When the admin marks a grant as `revocable=true` at issue-time (a new
  `AuthorizationGrant` field), Benten derives the K(N) chain through an additional
  per-grant key K(grant) — `K(N|grant) = HKDF(K(N), info="grant"||grant_id)`. Revoking
  destroys K(grant) on the issuer side (and broadcasts a revocation marker via UCAN
  revocation spec); replicated storage retains ciphertext that Eve can no longer derive
  a key for. UX: "Revoke Eve's access" button works (within the constraint that any
  ciphertext Eve already locally cached + decrypted remains in Eve's RAM/disk forever —
  no crypto defeats "we already gave her the plaintext").

- **OPT-IN for short-lived grants = Shape E (time-bounded grants with explicit expiry).**
  UCAN's `not_after` claim already supports this at the cap layer per CLAUDE.md #19.
  Surface it as a first-class UI affordance at grant-issue time.

- **WATCH-LIST mint (NAMED-DEFERRED per HARD RULE clause-b, not "later") =
  Shape B (admin-rotates-grant on each significant update).** This is the
  active-maintenance shape; rejected as DEFAULT because it converts a one-time admin
  action into perpetual obligation; preserved as an **application-layer pattern** any
  Benten frontend can implement on top of (A + D) without engine change. Documented as
  an `AtriumPolicy` configuration knob: `grant_refresh_required_secs: Option<u64>`
  parallels the existing `policy.refresh_required_secs` for attestation refresh.

- **NEW Shape G (proposed below) = "Grant follows CURRENT-pointer" (mutable-grant
  semantic).** A grant that targets `(Anchor-CID, follow_current: true)` rather than
  `(Version-Node-CID, follow_current: false)`. UX: more like Google Drive (the grant
  "sees the latest"); explicitly contradicts Path-A.5 immutability for the grant target;
  surfaced as a SEPARATE Compromise registry entry (candidate #57b) for codepoint-
  reserve at v1-beta. **Rejected as default** because it inverts the Path-A.5
  invariant; **preserved as opt-in** for use-cases where Alice's mental model is
  "current-state share" not "snapshot share."

**Composition with #55.** Compromise #57 is the per-grant-sub-graph sibling of #55's
per-subject-data framing. Both belong to the new disposition class
`Composition-Hazard-Honest-Disclosure` proposed in M-C2 R-MCV2-9(b). Together they form
a two-clause disclosure surface that audit deliverables + admin UI prompts cite in
parallel: "(a) Sharing X gives the recipient a snapshot view; CRDT-merges don't retract
their view (Compromise #57). (b) Strong erasure of data the recipient downloaded is not
mechanically achievable in any P2P system that ever delivered the bytes (Compromise #55)."

### §1.3 Why Shape H, not pure-A or pure-D

The **extra-reflection-pass** discipline (`feedback_extra_reflection_pass_for_elegant_
permanent_shape`) asks: is there a single elegant structural shape closing N findings at
once, strictly less code than greedy-sum-of-N? Yes — **a 4-field extension to
`AuthorizationGrant`** (`revocable: bool`, `grant_key_handle: Option<GrantKeyHandle>`,
`not_after: Option<Hlc>`, `follow_current: bool`) closes Shapes A + C + D + E + G in
one elegant move:

- `revocable=false, follow_current=false, not_after=None` → Shape A pure default
  (immutable snapshot grant; honest-disclosure via Compromise #57).
- `revocable=true, grant_key_handle=Some(_)` → Shape D crypto-shredding revocation.
- `not_after=Some(_)` → Shape E time-bounded.
- `follow_current=true` → Shape G CURRENT-pointer-following grant (codepoint-reserve
  at v1-beta; v1.x landing if Phase-N use-cases surface).
- Shape C "forward-only view" is the implicit semantic of `follow_current=false`; named
  explicitly in admin UI + docs.

The 4 fields are wire-format-additive (`AuthorizationGrant` is non-frozen at this point
in Wave-MS-PRIMITIVE per the canary-first execution per `feedback_canary_first_parallel
_implementation`). Cost: ~1.5 wave-days incremental on top of N1's already-budgeted
~0.95–1.2 wave-days; preserves the elegant 4-thing thin core + 6-dimension
RestrictedScope; matches Path-A.5 hybrid posture; aligns with UCAN revocation spec.

### §1.4 Confidence

- HIGH (~85%) that Shape A (honest-disclosure) is the structurally-correct DEFAULT —
  Path-A.5 immutability is the substrate guarantee; pretending otherwise mis-shapes the
  audit deliverable + future amendments.
- HIGH (~85%) that Shape D (crypto-shredding per-grant-key) is the right OPT-IN for
  admins who need the "revoke button works" UX, and that it composes orthogonally with
  Shape A on the wire (new `grant_key_handle` field; no break).
- MED-HIGH (~75%) that Shape E (time-bounded) belongs in the same 4-field expansion —
  it's already at the cap layer via UCAN `not_after`; promoting it to first-class
  grant-issue UI surface is cheap.
- MED (~60%) that Shape G (follow_current) is worth a v1-beta codepoint-reserve vs a
  Phase-N revisit. Documented as codepoint-reserve sub-field; my-pred RESERVE because
  the wire-format-additive cost is ~zero and the Phase-N revisit cost without reserve
  is much higher.
- HIGH (~90%) that Shape B (admin-rotates-on-update) is correctly rejected-as-default
  AND correctly preserved as an application-layer pattern via `AtriumPolicy::
  grant_refresh_required_secs`.

---

## §2 Shape-by-shape evaluation matrix

Per shape: (a) cryptographic soundness, (b) UX matches user mental models, (c) composes
with Path-A.5 immutability, (d) composes with N1 RestrictedScopeSet, (e) cost in
wave-days incremental.

### §2.1 Shape A — Honest-disclosure (Compromise #57)

> **Semantic.** Sharing X gives Eve immutable view of X at grant-time. To retract, admin
> rotates grant (mints new grant superseding the old via UCAN revocation chain) or forks
> MembershipSet. Storage hosts retain old ciphertext; Eve can continue to decrypt it as
> long as she retains K(grant-derived).
>
> **UX surface.** Admin sees a tooltip at grant-time: *"Eve will see this content from
> when you share it. If you remove items later, Eve's view will still include them. To
> fully retract Eve's access, use 'Revoke grant' (creates a new grant chain); some
> already-downloaded content remains on Eve's device."*

| Dimension | Score | Note |
|---|---|---|
| (a) Crypto soundness | A+ | No new crypto; matches Path-A.5 + N1 verbatim. Strictly the substrate guarantee. |
| (b) UX mental-model match | C- | Most admins (Google-Drive-trained) expect "revoke = gone." Mismatch surfaced by tooltip + admin-UI warning. |
| (c) Composes with Path-A.5 | A+ | This IS Path-A.5's UX face. Compromise #57 names the substrate guarantee honestly. |
| (d) Composes with N1 RestrictedScopeSet | A+ | N1 already references `roots: Vec<Cid>` as immutable Version-Node-CIDs; Shape A is the framing that makes N1 honest. |
| (e) Wave-day cost | 0.2 | Pure doc + 1 admin-UI tooltip string + 1 Compromise mint. Cheapest shape. |

**Precedent.** Tahoe-LAFS read-caps are the closest exact analogue (per WebSearch):
"to revoke a read-cap on an immutable file, you simply forget (delete) the capability
string; the immutable nature of these files means access control is primarily managed
through capability distribution rather than capability revocation at the server level."
Tahoe explicitly accepts this. IPFS pinning (per WebSearch): "the immutable nature of
content addressing combined with the pinning mechanism creates an inherent challenge:
once content is pinned and distributed across the network, it becomes difficult to
revoke or remove entirely." UCAN revocation spec: "revocation does not guarantee that
the Agent can no longer access to the capability in question; if an Agent is able to
construct a valid proof chain without relying on the revoked proof, they still have
access." All three precedents converge on Shape A's honest framing.

**Recommendation.** ADOPT AS DEFAULT (with implicit Shape C semantic).

### §2.2 Shape B — Admin-rotates-grant on each significant update

> **Semantic.** Admin must explicitly re-grant Eve after each meaningful CRDT-merge that
> changes the shape Eve has access to. Active grant-maintenance.
>
> **UX surface.** Admin sees periodic prompts: *"Eve's view of X needs to be refreshed.
> Grant new view?"*

| Dimension | Score | Note |
|---|---|---|
| (a) Crypto soundness | A | No new crypto; admin-triggered re-issue. |
| (b) UX mental-model match | F | Converts one-time admin action into perpetual obligation. Notification fatigue. Misses-by-default if admin ignores prompt. Worst UX. |
| (c) Composes with Path-A.5 | B | Doesn't fight Path-A.5; just re-issues grants pointing at fresh Version-Node-CIDs. |
| (d) Composes with N1 RestrictedScopeSet | A | N1's containment algebra makes re-issue mechanical (same SubgraphSpec; new CID roots). |
| (e) Wave-day cost | 1.0 | UI prompts + change-detection on Eve-visible-subtree + audit-event for re-grants. |

**Precedent.** No exact precedent. Matrix Megolm (per WebSearch) has a related shape:
"when a member leaves a room, the client should invalidate any active outbound Megolm
session, to ensure that a new session is used next time the user sends a message" —
admin-implicit-rotation on membership-change-events. Apple Family Sharing relies on
admin-implicit-removal for purchases, not periodic re-grants.

**Recommendation.** REJECT AS DEFAULT. PRESERVE AS APPLICATION-LAYER PATTERN via
`AtriumPolicy::grant_refresh_required_secs` knob — admin opts a specific Atrium into
"prompt me when grants get stale" UX without engine code change. Engine-primitives-vs-
application-layer discipline per `feedback_engine_primitives_vs_application_layer`:
this is exactly the kind of "engine doesn't need this, application composes from
primitives" case.

### §2.3 Shape C — Forward-only grants (default-deny-after-grant-time)

> **Semantic.** Eve's grant only gives view of content existing at-grant-time; any content
> added after grant-time is INVISIBLE unless explicitly added via grant-extension.
> CRDT-merge-removed content remains visible (immutable).
>
> **UX surface.** Clearer "frozen-in-time view" UX vs ambiguous current-state.

| Dimension | Score | Note |
|---|---|---|
| (a) Crypto soundness | A | Trivially derivable from Path-A.5 (grant references at-grant-time Version-Node-CID; CURRENT-pointer advances are simply not in Eve's grant scope). |
| (b) UX mental-model match | B+ | "Frozen-in-time view" is intuitive once labeled. Eve sees a snapshot date in her UI. Matches Tahoe-LAFS read-cap semantics. |
| (c) Composes with Path-A.5 | A+ | This IS the implicit semantic of `follow_current=false`. |
| (d) Composes with N1 RestrictedScopeSet | A | RestrictedScopeSet roots are CIDs by construction; the time-frozen nature is a property of the CID, not the scope-set. |
| (e) Wave-day cost | 0.1 | Just documentation; semantic is already structurally present. |

**Recommendation.** ADOPT AS THE EXPLICIT DEFAULT SEMANTIC. Eve's UI shows "snapshot of
X from <date>"; admin UI shows "Eve sees X as it was on <date>." Shape A is the policy
framing; Shape C is the UX framing of the SAME default. Bundled.

### §2.4 Shape D — Cryptographic-revocation via per-grant-key destruction

> **Semantic.** Each grant uses a derived per-grant-key K(grant). Revoking destroys
> K(grant) on the issuer side. Storage hosts retain ciphertext (unreadable by Eve
> post-revoke without K(grant)). Crypto-shredding pattern.
>
> **UX surface.** Admin sees "Revoke Eve's access" button that actually works (modulo
> already-downloaded-plaintext caveat).

| Dimension | Score | Note |
|---|---|---|
| (a) Crypto soundness | A | Derived-key crypto-shredding is well-understood. Spotify Padlock (per WebSearch) uses exact pattern at scale. |
| (b) UX mental-model match | A | Matches the universal "revoke button" mental model. With clear caveat about already-decrypted content. |
| (c) Composes with Path-A.5 | A | Layers ON TOP of K(N) chain without disturbing it: K(N\|grant) = HKDF(K(N), info="grant"\|\|grant_id). Path-A.5 invariants untouched. |
| (d) Composes with N1 RestrictedScopeSet | A | The K(grant) is per-AuthorizationGrant, orthogonal to per-RestrictedScope. |
| (e) Wave-day cost | 2.5 | Per-grant key derivation + revocation broadcast (UCAN revocation marker per spec) + admin-side key destruction discipline + receiver-side derivation. Wave-MS-PRIMITIVE incremental. |

**Precedent.** Spotify Padlock (per WebSearch): "per-user unique key approach: removing
or overwriting one key in one place causes a user's data to be inaccessible." UCAN
Revocation Spec: "Revocations MUST be immutable and irreversible. If a Revocation was
issued in error, it MUST NOT be retracted — a new, unique UCAN delegation MAY be issued
(e.g. by updating the nonce or changing the time bounds)." Plutus filesystem (2003,
per WebSearch) is the original distributed-file-sharing crypto-shredding precedent.

**Caveat (important).** Crypto-shredding does NOT defeat "Eve already downloaded and
decrypted the plaintext, retained it in her local cache." This is the same fundamental
limit named in #55 (P2P-by-design). Shape D narrows the revocation surface to "Eve
cannot decrypt FUTURE-discovered ciphertext" — useful when Eve had access to a large
sub-graph she hadn't fully traversed yet, or where future Atrium peers might serve old
ciphertext to Eve. It does NOT achieve "make Eve forget what she already read."

**Recommendation.** ADOPT AS OPT-IN via `revocable: bool` flag on AuthorizationGrant.
When true, mint K(grant) at issue-time; bind to grant_id in audit log; expose "Revoke"
admin action that destroys K(grant) + broadcasts UCAN revocation marker.

### §2.5 Shape E — Time-bounded grants

> **Semantic.** Grants have explicit expiry (e.g., 30 days). Eve must re-request after
> expiry. Aligns with N3 #54 attestation-refresh pattern + UCAN `not_after` claim.
>
> **UX surface.** Admin sets grant duration at issue-time; expiry surfaces in Eve's UI.

| Dimension | Score | Note |
|---|---|---|
| (a) Crypto soundness | A | UCAN `not_after` is well-understood. Trivially verifiable at receiver. |
| (b) UX mental-model match | A | "Share for 30 days" is a universal mental model (Signal disappearing messages, Google Drive expiring links). |
| (c) Composes with Path-A.5 | A | Independent of Version-Node-CID immutability; expiry is a UCAN-claim attribute. |
| (d) Composes with N1 RestrictedScopeSet | A | UCAN `not_after` applies to the AuthorizationGrant; RestrictedScopeSet sits inside the grant's `cap`. Orthogonal. |
| (e) Wave-day cost | 0.3 | Cap layer ALREADY supports `not_after` per CLAUDE.md #19. Promote to first-class grant-issue UI affordance. |

**Precedent.** Signal disappearing messages (per WebSearch): per-chat TTL with timer
starting after read; "the Signal service does not know whether or not a message is a
Disappearing Message" — privacy-preserving deletion. UCAN `not_after` claim is exactly
this primitive at the capability layer. Google Drive expiring links is the cloud
analogue.

**Recommendation.** ADOPT AS OPT-IN via `not_after: Option<Hlc>` on AuthorizationGrant
(already exists at UCAN cap layer). Surface as admin UI affordance "Share until <date>."
Bundle with Shape D for "revoke OR expiry" semantics.

### §2.6 Shape F — Original brief's hybrid (A + D + C)

This is approximately my Shape H below but without Shape G (CURRENT-pointer-following
grant) or Shape E (time-bounded) as first-class. Strictly dominated by Shape H. NOT
SELECTED.

### §2.7 Shape G (NEW) — "Grant follows CURRENT-pointer" (mutable-grant semantic)

> **Semantic.** A grant that targets `(Anchor-CID, follow_current: true)` rather than
> `(Version-Node-CID, follow_current: false)`. Eve sees "the current state of X" — when
> Alice CRDT-merges removal of Y-child, Eve's next read traverses the NEW Version-Node
> and Y-child is gone for Eve.
>
> **UX surface.** Grant-issue UI offers two modes: "Snapshot share" (Shape A/C default)
> vs "Live share" (Shape G). For "Live share," the tooltip reads: *"Eve sees the current
> state of X, including any later changes. Removing items from X removes them from Eve's
> view (subject to Eve's local cache)."*

| Dimension | Score | Note |
|---|---|---|
| (a) Crypto soundness | B+ | Requires Eve to re-derive K(N) for each new Version-Node CURRENT advances to. The N1 path-tagged K(N) chain re-derives mechanically; the new wrinkle is K(N|grant) at new CIDs. SOUND but adds a per-grant K-rotation surface. |
| (b) UX mental-model match | A+ | This is the Google Drive / Dropbox / iCloud mental model. Most natural for cloud-migration users. |
| (c) Composes with Path-A.5 | C+ | INVERTS the Path-A.5 invariant (which is "key encryption to immutable Version-Node-CID, not to anchor + CURRENT pointer"). Allowable as an EXPLICIT opt-in but never as default; otherwise undermines Path-A.5's whole point. |
| (d) Composes with N1 RestrictedScopeSet | B+ | Requires RestrictedScopeSet roots semantic to bifurcate: each root has a `follow_current: bool` discriminant. Wire-format-additive at v1-beta if reserved now. |
| (e) Wave-day cost | 1.5 (codepoint-reserve) / ~4–6 (full impl) | Reserve sub-field at v1-beta is cheap; full impl is post-v1-beta. |

**Precedent.** Google Drive shared links (per WebSearch): "You can revoke access to
shared links by changing the setting from 'Anyone with the link' to 'Restricted' and
removing specific individuals" — server-mediated mutable-grant semantics. Apple Family
Sharing access revocation (per WebSearch): "When a family member leaves or is removed,
they immediately lose access to shared subscriptions and content purchased by other
members" — server-mediated current-state share. Matrix encrypted rooms (per WebSearch):
"by default, new members cannot access historical messages unless keys are explicitly
shared" — analogous to Shape C "forward-only from grant-time" plus Shape G "future
content visible if member still in room."

**Recommendation.** CODEPOINT-RESERVE AT v1-BETA via `follow_current: bool` field on
AuthorizationGrant (default `false`). Full impl deferred to Phase-N (post-v1-beta).
Documented as NAMED-DEFERRED per HARD RULE clause-b with the trigger: "any production
Benten deployment whose primary admin demographic comes from cloud-share-link mental
models AND where the engine-primitives-vs-application-layer analysis shows Shape D +
Shape B-as-app-layer-pattern doesn't cover the use-case."

### §2.8 New Shape proposals — H (the winner) + I (a discarded alternative)

#### Shape H (CHOSEN) — 4-field AuthorizationGrant expansion bundling A+C+D+E + G-reserve

Already described in §1.2. The elegant-shape closure of Shapes A, C, D, E (full impl) + G
(codepoint-reserve). Default = `{revocable: false, grant_key_handle: None, not_after:
None, follow_current: false}` ≡ Shape A + Shape C semantics. Each field is independently
opt-in. ~1.5 wave-days incremental.

**Why this is the elegant permanent shape.** It is strictly less code than implementing
A + D + E + G as four separate features. It is wire-format-additive at v1-beta (the
4 fields are new on `AuthorizationGrant` which is non-frozen in Wave-MS-PRIMITIVE). It
forward-closes the entire "future grant-semantics extensions need fresh wire shape"
hazard class: any future shape (Shape I-X) extends the same 4-field bundle. It mirrors
the Path-A.5 hybrid framing exactly: keep Path-A.5 immutability as the DEFAULT, allow
opt-in alternatives via explicit semantic flags. It composes with N1's
RestrictedScopeSet without modifying N1's 4-thing thin core. It composes with #55's
P2P-by-design honest-disclosure pattern by adopting #55's framing for the DEFAULT case
and offering crypto-shredding as the explicit OPT-IN — the two compromises (#55 + #57)
become parallel statements at parallel layers (data layer + grant layer).

#### Shape I (DISCARDED) — Ephemeral re-key on CRDT-merge

> **Idea.** When CRDT-merge mints a new Version-Node-CID, the engine ALSO rotates the
> K(N) for that CID's parent — effectively breaking Eve's old K(N) chain.

**Why discarded.** Catastrophically expensive: every CRDT-merge becomes a key-rotation
event for every active grant referencing any ancestor of the merged node. Quadratic in
(grants × CRDT-merges). Fights Path-A.5 (which says K is bound to immutable Version-
Node-CID, not rotated). Defeats the purpose of immutable Version Nodes. Would essentially
re-introduce the "Path-A as L11 framed it" fork that Path-A.5 rejected. NOT VIABLE.

---

## §3 Precedent survey (full)

| System | Mechanism | Revocation works? | Lessons for Benten |
|---|---|---|---|
| **Google Drive shared link** | Server-mediated centralized ACL on shared URL | Yes, server flips bit; client can't decrypt without server's permission | Mental-model anchor for Shape G "follow current"; Benten cannot replicate the server-mediated flip in P2P |
| **Apple Family Sharing** | Server-mediated subscription-state + local-device App-Store enforcement | Yes for re-download; NO for already-downloaded content (per WebSearch: "previously downloaded items may remain accessible on devices unless manually deleted") | Honest about the "already on device" caveat; the parallel to #55's "already delivered bytes" framing |
| **Signal disappearing messages** | Per-chat TTL; timer-based local deletion | Yes within TTL semantic; not against malicious recipient who screenshots | Shape E precedent; service is unaware of disappearing-state (privacy by design) |
| **Tahoe-LAFS read-caps** | Capability-string distribution = access; revocation = forget the cap-string | No server-side revocation for immutable files | Direct Shape A precedent: "immutable read-caps cannot be revoked; access control is via cap distribution" — EXACTLY Path-A.5's posture |
| **IPFS pinning** | Pinned content cannot be garbage-collected; immutable | No revocation; relies on garbage collection if NOT pinned | Direct Shape A precedent: "once content is pinned and distributed, difficult to revoke or remove entirely" |
| **Matrix Megolm** | Sender-side outbound-session rotation on member-leave; backward-only invalidation | Yes for future messages; receiver retains old session keys for backread | Shape G precedent (forward-only revocation works for future writes) + Shape A precedent (past content not retracted) |
| **Slack pinned content** | Server-mediated ACL on workspace + channel | Yes (server-side) | Cloud-centralized; not P2P; not directly applicable |
| **Discord pinned content** | Server-mediated ACL on channel | Yes (server-side) | Same as Slack |
| **Wire / Matrix shared content** | Per-room key + Megolm session | Partial; new members default to no-history; departing members keep what they had | See Matrix above |
| **UCAN delegation chain (CLAUDE.md #19)** | Revocation by issuer; chain-walking verifier | "Last line of defense"; "eventually consistent"; "does not guarantee Agent can no longer access" (per spec) | Direct Shape A precedent for the grant-layer specifically; matches Benten's existing cap discipline |
| **Spotify Padlock** | Per-user unique key crypto-shredding | Yes (per WebSearch) | Direct Shape D precedent at production scale |
| **Plutus filesystem (2003)** | Per-grant-key crypto-shredding | Yes | Origin of the distributed-file-sharing crypto-shredding pattern |

**Net precedent reading.** Every distributed / content-addressed / P2P system in the
survey converges on the same honest framing: "you cannot revoke what you already gave
the recipient; you can either (a) accept this and document it (Tahoe-LAFS, IPFS, UCAN
spec, Matrix-past-history), or (b) crypto-shred future-discoverable content via per-
grant keys (Spotify Padlock, Plutus, Shape D)." Shapes A + D together capture the
precedent space. Shape G (follow_current) is the cloud-centralized model that does NOT
generalize to P2P without re-introducing trusted-server mediation.

---

## §4 Composition with Compromise #57 + #55 P2P-by-design pattern

### §4.1 The pattern itself

N3 §2.3 + M-CONS-v2 §1.3 established the **P2P-by-design honest-architectural-disclosure
pattern** for #55:

> A content-addressed P2P system cannot unilaterally erase content that has been
> replicated to other devices' local stores — that's the substrate guarantee that gives
> Benten its trustlessness. Applications that need strong-RTBF (legal compliance
> contexts) layer it on top via per-subject crypto-shredding sub-encryption layer
> (separate per-subject keys; delete subject's key shreds only their data); this lives
> at the Atrium-data layer, not MembershipSet. **This is the honest architectural
> posture; not "a limitation we should fix."**

The structural shape is: (1) name the substrate guarantee; (2) name where the
application layer can compose stronger semantics if needed; (3) refuse to "fix" the
substrate to mimic centralized-server semantics; (4) classify the compromise as
`Composition-Hazard-Honest-Disclosure` per the R-MCV2-9(b) third class.

### §4.2 #57 EXTENDS #55, doesn't close it

#55 is about **per-subject data erasure** (e.g., GDPR Article 17: erase Carol's
personal data from the Atrium). #57 is about **per-recipient grant retraction** (e.g.,
admin no longer wants Eve to be able to see the Y-child of X1). These are different
problems:

- #55 problem: "the Atrium contains data ABOUT Carol; erase it." Benten's answer:
  per-subject crypto-shredding at the Atrium-data layer (separate per-subject keys;
  delete Carol's key → Carol's data unreadable for all members).
- #57 problem: "Eve has a grant to view X; admin wants to retract Eve's view of part of
  X." Benten's answer (Shape H = A + C default; D + E + G opt-in): substrate guarantee
  + opt-in stronger primitives.

The two compromises share the disposition class `Composition-Hazard-Honest-Disclosure`.
The two cited together form Benten's complete posture on "revocation" in the P2P
content-addressed model:

> **The Benten composite posture.** (a) Per-subject data erasure (#55): substrate
> guarantee + application-layer per-subject crypto-shredding. (b) Per-recipient grant
> retraction (#57): substrate guarantee + opt-in per-grant crypto-shredding + opt-in
> time-bounded grants + codepoint-reserve for follow-current-pointer grants. (c) Already-
> delivered plaintext (orthogonal to both): no crypto system defeats this; the recipient
> who held the bytes once holds them forever.

This is structurally elegant: a 2-clause honest-disclosure surface that admin UIs +
audit deliverables cite in parallel, with clear delineation of which clause covers
which problem.

### §4.3 Does Shape H extend or close the P2P-by-design framing?

**Extends.** Shape H keeps the P2P-by-design semantic as the DEFAULT (Shape A + C),
adopts #55's framing verbatim for the per-grant-sub-graph layer, AND offers explicit
opt-in mechanical revocation (Shape D) for use-cases where the substrate-guarantee
default is insufficient. This is BOTH-AND, not either-or. Crucially, Shape D's
crypto-shredding remains honest about its own limit — "Eve cannot decrypt future-
discovered ciphertext" without claiming "Eve has forgotten what she already read."

The single most-important property of Shape H: **a Benten admin who reads the tooltip
for the default grant comes away with a CORRECT mental model.** They are not surprised
later. If they need stronger revocation, they explicitly opt into Shape D / E / G at
grant-issue time, with a tooltip that explains what each opt-in actually achieves.

This composability discipline is the same one #55 chose: do not pretend the substrate
behaves like a centralized server; offer application-layer composition for stronger
semantics; document the gap honestly so admins make informed choices.

---

## §5 Compromise #57 mint text (final)

```markdown
**#57** — **RestrictedScopeSet grant retraction honest-architectural-disclosure per
Path-A.5 immutability semantic.** (Disposition class: `Composition-Hazard-Honest-
Disclosure` per R-MCV2-9(b).)

**Substrate guarantee.** Per Path-A.5 (CLAUDE.md baked-in #18 + Inv-13), Benten's
Version-Node-CIDs are immutable forever. A `RestrictedScopeSet` grant's `roots: Vec<Cid>`
references Version-Node-CIDs by-value. When the admin "removes" a child of a granted
root via CRDT-merge, Path-A.5 mints a NEW Version-Node-CID with the child absent and
advances the Anchor's CURRENT-edge; the OLD Version-Node-CID continues to exist forever
and remains resolvable. The grant — referencing the OLD CID — continues to give the
recipient view of the OLD subtree, including the "removed" child. **This is correct by
substrate design; not a bug.** The substrate guarantee is what gives Benten its
content-addressed-immutability + offline-merge-correctness; pretending otherwise would
re-introduce a trusted-server mediation surface that contradicts the engine's whole
posture.

**Application-layer compositions Benten ships at v1-beta.** Four orthogonal opt-in
flags on `AuthorizationGrant` close the retraction-UX gap WITHOUT fighting Path-A.5:

1. **`revocable: bool` + `grant_key_handle: Option<GrantKeyHandle>`** — when true, the
   K(N) chain folds in a per-grant K(grant); revocation destroys K(grant) on the issuer
   side + broadcasts a UCAN revocation marker (per UCAN Revocation Spec). Storage hosts
   retain ciphertext; the recipient cannot derive the read-key for future-discovered
   ciphertext. (Caveat: ciphertext the recipient already decrypted locally remains in
   their local cache forever — orthogonal to crypto-shredding, same fundamental limit
   as #55.)
2. **`not_after: Option<Hlc>`** — UCAN `not_after` claim promoted to first-class grant-
   issue UI affordance. Eve must re-request after expiry. Matches Signal-disappearing-
   messages + Google-Drive-expiring-link mental model.
3. **`follow_current: bool`** — CODEPOINT-RESERVE ONLY at v1-beta. When true (post-
   v1-beta), the grant targets `Anchor-CID` rather than `Version-Node-CID`; recipient
   re-derives K(N) for each new CURRENT advance. Matches Google-Drive-style "see the
   latest" mental model. INVERTS Path-A.5 default; explicit opt-in.
4. **`grant_refresh_required_secs: Option<u64>` on `AtriumPolicy`** — application-layer
   pattern (per `feedback_engine_primitives_vs_application_layer`); engine emits prompts
   when grants get stale; admin re-issues. Mirrors `policy.refresh_required_secs` for
   attestation refresh.

**Default behavior.** `{revocable: false, grant_key_handle: None, not_after: None,
follow_current: false}` ≡ honest-disclosure: snapshot grant at issue-time CID; admin
tooltip reads *"Sharing X gives the recipient a snapshot view of X. Items added or
removed via CRDT-merge after grant-time will not retract the recipient's view. To
fully revoke, use 'Revoke grant' with a revocable grant (creates new grant chain); some
already-downloaded content remains on the recipient's device."*

**Composition with #55.** #57 is the per-grant-sub-graph sibling of #55's per-subject-
data framing. Together: (a) #55 = per-subject data erasure; substrate guarantee + per-
subject crypto-shredding at Atrium-data layer. (b) #57 = per-grant retraction; substrate
guarantee + opt-in per-grant crypto-shredding + opt-in time-bounded grants + codepoint-
reserve for follow-current grants. (c) Both share the `Composition-Hazard-Honest-
Disclosure` class. (d) Neither defeats "the recipient who held bytes once holds them
forever."

**Cost.** ~1.5 wave-days incremental on N1's already-budgeted ~0.95–1.2 wave-days:
4 fields on `AuthorizationGrant` + per-grant key derivation (HKDF info string extension)
+ revocation marker broadcast + admin-UI tooltips + 1 Compromise mint + 1 codepoint-
reserve sub-field. Wire-format-additive in Wave-MS-PRIMITIVE canary. No Path-A.5 change,
no K(N) chain change beyond the optional K(grant) info-string extension.

**Origin of mint.** M-C2 v2 Scenario F-F + Blind spot B-1 (`0533d0cf`); R-MCV2-(B-1)
ratification proposed; this doc completes the design + composes with the wider shape
survey.
```

---

## §6 R0 plan-doc implications

Per `feedback_inverted_prework_post_campaign_phase`: R0 plan-doc is composing against
M-CONS-v2.1 (the refined consolidator integrating M-C1/M-C2/M-C3 critique + this UX
shape survey). The following sections are LANDING-BEARING in R0:

### §6.1 `AuthorizationGrant` 4-field expansion (Wave-MS-PRIMITIVE addendum)

R0 plan-doc §-add to Wave-MS-PRIMITIVE wave-spec:

```rust
// Per Compromise #57 (M-C2 R-MCV2-(B-1) closure)
// Wire-format-additive; Wave-MS-PRIMITIVE canary lands these in the same wave that
// lands N1 RestrictedScopeSet (per `feedback_canary_first_parallel_implementation`).
pub struct AuthorizationGrant {
    pub iss: Did,
    pub aud: Did,
    pub cap: Vec<Capability>,           // RestrictedScopeSet lives here (N1 F28)
    pub not_after: Option<Hlc>,         // [NEW per #57(2)] UCAN-spec
    pub revocable: bool,                // [NEW per #57(1)] default false
    pub grant_key_handle: Option<GrantKeyHandle>,  // [NEW per #57(1)] Some iff revocable
    pub follow_current: bool,           // [NEW per #57(3)] CODEPOINT-RESERVE at v1-beta;
                                        //   default false; full impl post-v1-beta
    // ... existing fields ...
}

pub struct GrantKeyHandle {
    pub grant_id: GrantId,              // BLAKE3(canonical-CBOR(grant))
    pub key_commitment: [u8; 32],       // commitment to K(grant); HKDF info source
}
```

### §6.2 `AtriumPolicy` extension (Shape B as application-layer pattern)

R0 plan-doc §-add to `AtriumPolicy` design:

```rust
// Per #57(4): admin opts an Atrium into "prompt me when grants get stale" UX
pub struct AtriumPolicy {
    // ... existing fields ...
    pub grant_refresh_required_secs: Option<u64>,  // [NEW per #57(4)] None = no prompts
}
```

### §6.3 ErrorCode mint

R0 plan-doc §-add to ErrorCode catalog (per §3.5g Cross-language rule-mirror discipline):

| ErrorCode | Trigger | TS-mirror |
|---|---|---|
| `E_GRANT_REVOKED` | Grant's K(grant) destroyed; receiver attempts decrypt | YES (cap layer error class) |
| `E_GRANT_EXPIRED` | `not_after < now()` at decrypt-time | YES |
| `E_GRANT_FOLLOW_CURRENT_NOT_YET_SUPPORTED` | `follow_current=true` at v1-beta (codepoint-reserve) | YES |

### §6.4 Audit deliverable §-rows

R0 plan-doc §-add to audit deliverable spec:

- **§-row #57.a — Substrate guarantee disclosure.** Audit doc cites #57's substrate-
  guarantee paragraph verbatim alongside #55's. Admin onboarding flow surfaces both.
- **§-row #57.b — Per-grant key K(grant) derivation chain.** When `revocable=true`, the
  derivation is `K(N|grant) = HKDF(K(N), info="grant"||grant_id || N.cid)`. Doc this
  alongside the existing Path-A.5 K(N) chain doc.
- **§-row #57.c — UCAN revocation marker broadcast.** When admin revokes, a UCAN
  revocation marker (per spec) is broadcast through the standard `MembershipEvent`
  audit stream (F-A2). Receivers verify revocation BEFORE attempting K(grant) derivation.

### §6.5 V1-FROZEN-INTERFACE.md §15.c-§15.f update

R0 plan-doc §-add to V1-FROZEN-INTERFACE.md:

- §15.c `Scope` enum + `RestrictedScopeSet` already in N1's scope per ed592770; this doc
  doesn't modify it.
- §15.f K(N) two-path key-derivation contract — extend to mention OPTIONAL K(grant) info-
  string when grant has `revocable=true`: `K(N|grant) = HKDF(K(N), info="grant"||grant_id)`.
- New §15.g — `AuthorizationGrant` extension fields (the 4-field bundle).

### §6.6 Admin-UI tooltips (spec doc; impl is application-layer)

R0 plan-doc §-add to MembershipSet-Spec admin-UI guidance section:

| Grant flag combination | Admin tooltip at issue-time | Recipient tooltip |
|---|---|---|
| Default (all-default) | "Eve will see X as it is right now. Future edits won't add or remove from Eve's view. To fully retract, use 'Revoke grant' with a Revocable grant." | "You see X as it was on <date>." |
| `revocable=true` | "Eve will see X as it is right now. You can 'Revoke' at any time, which prevents Eve from decrypting any X content she hasn't already downloaded." | "You see X as it was on <date>. Alice can revoke your access at any time." |
| `not_after=Some(...)` | "Eve will see X until <date>, then access expires." | "You see X as it was on <date>. Access expires on <date>." |
| `revocable=true, not_after=Some(...)` | Combined | Combined |
| `follow_current=true` (post-v1-beta) | "Eve sees X as it changes; Alice's edits flow through. Eve's view will reflect removals." | "You see the current state of X." |

---

## §7 Self-assessment + confidence + open questions

### §7.1 What I'm confident about (HIGH)

- **Shape A + C is the structurally-correct DEFAULT.** Path-A.5 immutability is the
  substrate guarantee; #55's P2P-by-design framing applies verbatim at the grant layer.
  Every distributed precedent in the survey (Tahoe-LAFS, IPFS, UCAN spec, Matrix-past-
  history) converges on this honest framing.
- **Shape D is the right OPT-IN for "revoke button works" UX.** Spotify Padlock + Plutus
  precedents at production scale. Crypto-shredding via per-grant K(grant) layers
  orthogonally on top of N1's path-tagged K(N) chain without disturbing Path-A.5.
- **Shape E (UCAN `not_after`) is already-shipping at the cap layer; promoting to first-
  class grant-issue UI is cheap + matches universal mental models.**
- **Shape H bundles A+C+D+E (full) + G (codepoint-reserve) elegantly in 4 fields on
  `AuthorizationGrant`.** Strictly less code than implementing each separately.
  Wire-format-additive in Wave-MS-PRIMITIVE canary. Composes orthogonally with N1 +
  Path-A.5 + #55 P2P-by-design pattern.

### §7.2 What I'm medium-confident about (MED-HIGH)

- **Shape G (follow_current) codepoint-reserve vs Phase-N revisit.** I recommend reserve
  because the cost is ~zero (1 bool field) and the Phase-N revisit without reserve is
  much higher. There's a residual risk that we never need it and the reserved
  codepoint is dead weight forever. Acceptable trade-off given the wire-format-additivity.
- **Shape B as application-layer pattern via `AtriumPolicy::grant_refresh_required_secs`
  knob.** Conceptually right per `feedback_engine_primitives_vs_application_layer`. The
  residual question is whether ANY v1-beta application actually wires the prompt UX.
  My-pred: no, but the knob's existence is cheap doc + 1 field on AtriumPolicy.

### §7.3 Open questions for Ben (surfacing per `feedback_surface_arch_decisions_under_auth`)

These are real architectural forks where my Auto Mode judgment shouldn't substitute for
Ben's. The plain-English framing per `feedback_plain_english_surfaces`:

**Q1 — Default opt-in for `revocable=true`?** Right now I'm proposing
`revocable: false` as default (matches Tahoe-LAFS + UCAN-spec posture). An alternative
default is `revocable: true` (matches Google-Drive mental model; closer to what
non-expert admins expect). The cost of default=true is: every grant carries a K(grant);
every revocation broadcasts a UCAN revocation marker; the K(N) chain ALWAYS folds in
K(grant). The cost of default=false is: admins must explicitly tick "Revocable" when
they want the revoke button.

- My-pred: KEEP default=false. Matches the honest-disclosure posture; admin who wants
  revoke button explicitly opts in; doesn't bloat the common case with K(grant) overhead.
- Confirm-or-redirect.

**Q2 — Codepoint-reserve Shape G (`follow_current`) at v1-beta?** Reserve costs ~zero
wire-format; risk is dead-weight codepoint. Defer-without-reserve costs Phase-N wire-
format-additive landing (still possible but more invasive).

- My-pred: RESERVE. Matches the Q1 iroh-gossip + N4 RotatingGroupKeyChainedMode
  codepoint-reserve discipline; cheap insurance against the Google-Drive-mental-model
  use-case ever becoming load-bearing in Benten production.
- Confirm-or-redirect.

**Q3 — `grant_refresh_required_secs` on `AtriumPolicy` ships at v1-beta or post-v1-beta?**
Ships at v1-beta = 1 field on AtriumPolicy + 0 engine code (application-layer pattern).
Ships post-v1-beta = AtriumPolicy gets wire-format-additive change later.

- My-pred: SHIP AT v1-BETA AS DOC-ONLY FIELD-RESERVE. Same logic as Q2 reserve. ~0.1
  wave-day cost.
- Confirm-or-redirect.

### §7.4 What I am NOT confident about

- **Whether any v1-beta admin UI vendor will actually render the tooltips at the
  recommended cadence.** This is downstream of Benten engine; mention in
  MembershipSet-Spec admin-UI guidance but not a v1-beta-LB obligation.
- **Whether `follow_current` (Shape G) is the right shape vs Shape B-as-app-layer-
  pattern for the "Google Drive mental model" use-case.** Both are viable; G is more
  honest about what it costs (re-derives K on every CURRENT advance) but Shape B
  as-app-layer is simpler. Could be a Phase-N empirical question.

---

## §8 Refinement recommendations (R-P4FF-1 .. R-P4FF-6)

| ID | Statement | Wave-day cost | Wire-affecting? | Closes |
|---|---|---|---|---|
| **R-P4FF-1** | Mint Compromise #57 per §5 text. Disposition class `Composition-Hazard-Honest-Disclosure` (per R-MCV2-9(b)). | 0.2 (doc-only) | NO | M-C2 F-F + B-1 |
| **R-P4FF-2** | Extend `AuthorizationGrant` with 4 fields: `revocable: bool`, `grant_key_handle: Option<GrantKeyHandle>`, `not_after: Option<Hlc>` (UCAN-spec-existing; promote to first-class field), `follow_current: bool` (codepoint-reserve at v1-beta). | ~1.5 (Wave-MS-PRIMITIVE canary) | YES (wire-additive; v1β-LB if minted now; codepoint-additive for `follow_current`) | Shape H bundle |
| **R-P4FF-3** | Extend K(N) HKDF info-string per §15.f update: when `revocable=true`, fold `info="grant"||grant_id` into the per-grant K(grant) derivation. Doc + impl in Wave-MS-PRIMITIVE. | ~0.5 (impl + doc) | NO (HKDF info-string change; AAD-binding additive) | Shape D opt-in |
| **R-P4FF-4** | Mint 3 ErrorCodes per §6.3 + TS-mirror per §3.5g Cross-language rule-mirror discipline. | ~0.3 (3 ErrorCodes + TS-mirror) | NO | Shape D + E + G runtime error surface |
| **R-P4FF-5** | Add `grant_refresh_required_secs: Option<u64>` to `AtriumPolicy` (doc-only field-reserve at v1-beta; application-layer prompt UX deferred to Phase-N). | ~0.1 (1 field + doc) | YES (1 AtriumPolicy field; v1β-LB if minted now) | Shape B as app-layer pattern |
| **R-P4FF-6** | Audit deliverable §-rows #57.a/.b/.c per §6.4. Spec-doc admin-UI tooltip guidance per §6.6. V1-FROZEN-INTERFACE.md §15.g per §6.5. | ~0.4 (doc-only) | NO | Spec coverage |

**Cumulative cost:** **~3.0 wave-days** on top of N1's already-budgeted ~0.95–1.2 wave-
days. Approximately the same magnitude as M-C2 R-MCV2-(B-1)'s original "0.2 wave-day
doc-only mint"; the additional ~2.8 wave-days arise because the elegant-shape closure
folds in Shapes D + E + G alongside Shape A. Trade-off worth taking per the
extra-reflection-pass discipline: the alternative (mint #57 doc-only, defer the 4-field
expansion to Phase-N) leaves the admin with no opt-in path beyond "fork the
MembershipSet" — much worse UX, and a Phase-N wire-format break is much more expensive
than a v1-beta wire-format-additive expansion.

---

## §9 Summary + disposition

### §9.1 Disposition

**RECOMMEND ADOPTION** of Shape H (4-field `AuthorizationGrant` expansion bundling
Shapes A + C + D + E + G-codepoint-reserve) + Compromise #57 mint per §5 + R-P4FF-1..6
refinements per §8 + 3 Q-surfaces for Ben per §7.3.

**Out-of-scope per HARD RULE clause-a:**
- Per-message FS (#56 EphemeralRatchet territory).
- Per-subject crypto-shredding for GDPR-RTBF at Atrium-data layer (#55 territory).
- Application-layer admin-UI vendor implementation choices.
- Already-downloaded-plaintext recovery (no crypto defeats this).

**Named-deferred per HARD RULE clause-b (not "later"):**
- Shape G (`follow_current=true`) full impl deferred to **post-v1-beta Phase-N revisit**;
  trigger = "any production Benten deployment whose primary admin demographic comes
  from cloud-share-link mental models AND where Shape B-as-app-layer-pattern doesn't
  cover the use-case." Codepoint-reserved at v1-beta per R-P4FF-2.
- Shape B prompt UX deferred to **application-layer landing in Phase-N**; the
  `AtriumPolicy::grant_refresh_required_secs` knob ships at v1-beta as field-reserve
  per R-P4FF-5.

**Disagreement-with-explanation per HARD RULE clause-c:** None. M-C2's R-MCV2-(B-1)
proposal to "mint Compromise #57 doc-only" is REFINED — the elegant-shape closure
expands it to Shape H (4-field expansion bundling) at +2.8 wave-day cost. The doc-only
mint alone leaves admins with worse UX than necessary; the expansion preserves Path-A.5
defaults + offers honest opt-ins.

### §9.2 Composition with M-CONS-v2.1 (the next consolidator)

M-CONS-v2.1 (the post-M-C1/M-C2/M-C3 consolidator per M-C2's "Refine before R0 plan-doc
authoring" disposition) should fold this doc's R-P4FF-1..6 into the M-CONS-v2.1
amendment registry. The 4-field expansion of `AuthorizationGrant` is wire-format-
additive in Wave-MS-PRIMITIVE canary (per `feedback_canary_first_parallel_
implementation`); no Wave resequencing. Compromise # numbering: #57 is already pre-
allocated in M-CONS-v2 §6 / M-C2 R-MCV2-(B-1); this doc supplies its final text.

### §9.3 Net wave-day delta vs M-CONS-v2 baseline

- M-CONS-v2 (e62ff540) central: ~89–115 wave-days.
- M-C2 (0533d0cf) +2.5–3.5 wave-day refinement bundle (R-MCV2-1..9 + R-MCV2-B-1..B-4).
- This doc's R-P4FF-1..6 +3.0 wave-days incremental on M-C2's R-MCV2-(B-1) ~0.2.
- M-CONS-v2.1 expected central: **~92–118 wave-days** (within bracket; same magnitude).

No phase-bracket movement. No canary re-design. Wave-MS-PRIMITIVE canary still
load-bearing; absorbs R-P4FF-2 + R-P4FF-3 alongside N1's RestrictedScopeSet landing.

### §9.4 200-word summary (for handoff use)

The M-C2 F-F failure (RestrictedScopeSet grants reference immutable Version-Node-CIDs
under Path-A.5; CRDT-merge "removals" mint new CIDs the grant cannot see; admin's
revocation intent is silently violated) is resolved via Shape H — a 4-field expansion to
`AuthorizationGrant` (`revocable`, `grant_key_handle`, `not_after`, `follow_current`)
bundling Shapes A (honest-disclosure default) + C (forward-only-view semantic, implicit)
+ D (per-grant crypto-shredding revocation, opt-in) + E (UCAN `not_after`, opt-in) +
G (CURRENT-pointer-following grant, codepoint-reserve only at v1-beta). Compromise #57
mints as the per-grant-sub-graph sibling of #55's per-subject-data P2P-by-design
honest-disclosure pattern; both share the `Composition-Hazard-Honest-Disclosure`
disposition class. Precedent survey (Tahoe-LAFS, IPFS, UCAN-spec, Matrix-past-history,
Spotify Padlock, Plutus) converges on Shape A + Shape D as the honest distributed-system
posture. Net cost: ~3.0 wave-days incremental on M-C2's original ~0.2-day doc-only
mint, in exchange for closing 5 shapes elegantly in 4 wire-additive fields. 3 Q-surfaces
for Ben (default for `revocable`; codepoint-reserve Shape G yes/no; ship Shape-B-knob
at v1-beta). No Wave resequencing; M-CONS-v2.1 central rises ~3 wave-days to ~92–118.

---

**End of document.**
