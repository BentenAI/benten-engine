# Exploration: Fractal and Polycentric Governance for Groves

**Created:** 2026-04-11
**Purpose:** Deep exploration of how Groves (the DAO-like community tier) implement fractal nesting, polycentric federation, and polyfederation -- where governance rules are operation subgraphs, communities contain sub-communities, and a single community can belong to multiple parent structures simultaneously.
**Status:** Research exploration (pre-design)
**Dependencies:** `SPECIFICATION.md`, `explore-self-evaluating-graph.md`, `explore-content-addressed-hashing.md`, `explore-blockchain-assessment.md`, `operation-vocab-p2p.md`

---

## 1. The Three Concepts

### 1.1 Fractal Governance

**Definition:** A Grove contains sub-Groves, each with its own governance rules. Sub-Groves inherit governance from their parent but can override specific rules. The same pattern repeats at every level of nesting.

**Real-world analogy:** A national cooking community has regional chapters. The national Grove defines food safety standards. The Louisiana chapter adds Cajun cooking traditions. The New Orleans sub-chapter of the Louisiana chapter adds specific festival rules. Each level inherits from above and adds its own specificity.

**Key property:** Self-similarity. The governance mechanism at the national level is structurally identical to the governance mechanism at the local level. A vote is a vote. A rule is a rule. A capability grant is a capability grant. The only difference is scope.

### 1.2 Polycentric Governance

**Definition:** A Grove can federate with MULTIPLE parent structures simultaneously, across different governance dimensions. There is no single hierarchy -- governance emerges from overlapping, semi-autonomous authorities.

**Real-world analogy:** A local environmental group belongs to both a national environmental federation AND a city governance federation. It follows environmental policy from the national org and municipal policy from the city. When the two conflict, the local group resolves the tension using its own governance rules.

**Key property:** Overlapping authority. Multiple governance structures have legitimate claims over the same community. This is not a bug -- it is the mechanism by which complex governance actually works.

### 1.3 Polyfederation

**Definition:** Communities can wrap, fork, and sync with other communities across dimensions. Fork a community's governance, stay synced with upstream (receive edits to shared governance Nodes), or diverge entirely. The governance structure itself is subject to competitive pressure -- communities that govern well attract members.

**Key property:** Exit and voice at the governance level, not just the membership level. If you disagree with governance, you do not just leave -- you fork the governance and compete.

---

## 2. Precedent Analysis: Who Has Done This?

### 2.1 Elinor Ostrom's Eight Principles for Governing the Commons

Ostrom won the 2009 Nobel Prize in Economics for demonstrating that communities can successfully govern shared resources without either privatization or top-down government control. Her eight design principles, derived from studying successful commons institutions worldwide, are the theoretical foundation for everything that follows.

**The Eight Principles:**

1. **Clearly Defined Boundaries.** Who is a member? What is the resource? Both must be explicitly defined. A fishing village commons works because everyone knows who is allowed to fish and where the fishing grounds are.

2. **Congruence Between Rules and Local Conditions.** Rules must fit the specific resource and community. A water allocation rule that works in a Californian irrigation district would fail in a Balinese rice terrace. The rules must be adapted to local ecology.

3. **Collective-Choice Arrangements.** Most individuals affected by the rules can participate in modifying them. This is not just democracy -- it is specifically that the GOVERNED participate in making the rules that govern them.

4. **Monitoring.** Monitors who audit compliance are accountable to the community (or ARE the community members). External monitoring by strangers does not work. Peer monitoring by people with skin in the game does.

5. **Graduated Sanctions.** First violation gets a warning. Second gets a small penalty. Escalating consequences based on severity and context. Ostrom found that systems with only severe punishments (ban or nothing) fail because enforcers are reluctant to apply them.

6. **Conflict Resolution.** Cheap, accessible, local dispute resolution. If resolving a conflict requires lawyers and courts, the commons fails. Disputes must be resolvable at the community level.

7. **Minimal Recognition of Rights to Organize.** External authorities must not undermine the community's right to self-govern. A government that overrides community fishing rules destroys the commons even if the government's rules are "better."

8. **Nested Enterprises.** For resources that are part of larger systems, governance activities are organized in multiple nested layers. Local rules handle local issues. Regional rules handle cross-community issues. Neither layer should swallow the other.

**Application to Benten:**

| Ostrom Principle | Benten Mechanism |
|---|---|
| 1. Defined Boundaries | Subgraph boundaries + capability scopes. A Grove's membership is defined by who holds capabilities within the Grove's subgraph. |
| 2. Congruence with Local Conditions | Local governance operation subgraphs. Each Grove writes its own rules as operation Nodes. No universal governance imposed. |
| 3. Collective-Choice | Voting operation subgraphs. Members vote to modify the governance subgraph itself. The rules that govern rule-changes are themselves rules. |
| 4. Monitoring | Audit is the graph. Every mutation is a signed version Node. Any member can traverse the version chain and verify compliance. Monitoring IS reading the graph. |
| 5. Graduated Sanctions | Sanctions as capability attenuation. First offense: warning Node. Second: read-only for 24h (capability restriction). Third: capability revocation. The sanction subgraph defines the escalation path. |
| 6. Conflict Resolution | Dispute resolution operation subgraphs. When two members disagree, a governance operation subgraph defines the resolution process -- who mediates, how they are selected, what the possible outcomes are. |
| 7. Right to Organize | Fork capability. Any member can fork the Grove's subgraph and establish their own governance. The platform cannot prevent self-organization because data is sovereign. |
| 8. Nested Enterprises | **This is the entire subject of this document.** |

Ostrom's eighth principle is not just a nice-to-have -- it is the mechanism that makes the other seven principles scale beyond small groups. Without nesting, every commons governance institution is limited to Dunbar's number (~150 people). With nesting, the same principles work at any scale: family, neighborhood, city, nation, globe.

### 2.2 Federal/State/Local Government

**Structure:** Federal government sets constitutional constraints. States set laws within those constraints. Counties and cities set ordinances within state law. Each level has sovereignty within its domain but is constrained by the level above.

**What works:**
- **Preemption is explicit.** When federal law overrides state law, the doctrine of preemption has specific constitutional grounding and legal tests (express preemption, field preemption, conflict preemption). There is no ambiguity about WHAT is overridden.
- **Residual sovereignty.** Powers not explicitly granted to the federal government are retained by the states (10th Amendment). The default is local authority, not central authority. The parent must CLAIM authority; it does not have it by default.
- **Concurrent jurisdiction.** Many areas are governed by multiple levels simultaneously (taxation, environmental regulation, criminal law). This works because the levels have different scopes and mechanisms.
- **Laboratories of democracy.** States can experiment with different policies. Other states observe the results and adopt or reject. Governance evolves through competition.

**What fails:**
- **Preemption creep.** The federal government has expanded its preemption scope continuously since the New Deal. State autonomy has eroded over time because the higher level has structural incentives to centralize.
- **Unfunded mandates.** Higher levels imposing rules without providing resources to implement them. The mandate flows down; the cost stays at the bottom.
- **Capture by higher levels.** Local governance can be overridden by higher levels for political reasons, violating Ostrom's Principle 7.

**Lessons for Benten:**
- **Preemption must be explicit in the governance subgraph.** When a parent Grove overrides a child Grove's rule, the override must be a specific Node that references both the parent rule and the child rule, with content hashes for auditability.
- **Default is local sovereignty.** A child Grove's rules apply unless the parent Grove has EXPLICITLY claimed authority over that domain. This is the 10th Amendment principle: unspecified powers remain local.
- **Resource flows must be tracked.** If a parent Grove mandates behavior from a child Grove, the resource implications (storage quotas, computation costs, moderation labor) should be visible in the graph.

### 2.3 The Catholic Church: Vatican to Parish

**Structure:** Pope -> Roman Curia -> Archdiocese (Metropolitan) -> Diocese -> Parish. The Pope has universal authority. Bishops have full authority within their diocese, subject to Canon Law. Pastors have delegated authority within their parish.

**What works:**
- **Subsidiarity.** The Catholic social teaching of subsidiarity (Quadragesimo anno, 1931) holds that decisions should be made at the lowest competent level. The Vatican does not decide what time Mass is held at St. Mary's parish in Springfield. The pastor decides.
- **Canonical autonomy.** Bishops exercise "full and immediate ordinary power" within their diocese (Canon 381). The Pope CAN intervene, but normally does not. This creates genuine local autonomy within a global hierarchy.
- **Metropolitan oversight without control.** An Archbishop has limited supervisory authority over suffragan dioceses -- they can ensure faith and discipline are observed, but they cannot micromanage. This is oversight, not governance.
- **1,600+ year track record.** The oldest continuously operating governance hierarchy in the world. It works (in the organizational sense) because the nesting is stable and the authority boundaries are clear.

**What fails:**
- **Unaccountable hierarchy.** Bishops are appointed, not elected. There is no collective-choice arrangement (Ostrom Principle 3) at the governance level. This creates accountability gaps.
- **Information asymmetry.** The hierarchy can suppress information that flows upward. The child abuse crisis was enabled by the hierarchical structure's ability to contain information at the diocesan level.
- **No exit right.** Members cannot fork the Diocese. You attend the parish assigned by geography. This violates the competitive pressure that makes governance evolve.

**Lessons for Benten:**
- **Subsidiarity as the default.** Parent Groves should have LIMITED and ENUMERATED powers over child Groves. Everything else is the child's domain. This mirrors canonical autonomy.
- **Oversight without control.** A parent Grove can monitor child Grove compliance (reading the graph), but should not directly modify child Grove governance. The parent can revoke the parent-child relationship (excommunication), but not micromanage.
- **Exit MUST be possible.** Unlike the Church, Benten's fork capability means any member or sub-Grove can leave, taking their data with them. This competitive pressure incentivizes good governance.

### 2.4 Reddit: Subreddit Governance

**Structure:** Reddit (platform) -> Subreddits (communities) -> Users. Each subreddit has its own moderators, rules, and culture. Reddit has site-wide policies that override subreddit rules.

**What works:**
- **Enormous variety.** 2.8 million subreddits with radically different governance approaches -- from r/AskHistorians (extremely strict moderation, academic standards) to r/AnarchyChess (chaos as governance). The platform supports diverse governance because each subreddit is autonomous.
- **Emergent governance.** Subreddit rules are written by moderators based on community needs. They evolve over time based on what works.
- **Low barrier to creation.** Anyone can create a subreddit. If you disagree with r/Python's moderation, you can create r/BetterPython. This is the "fork" mechanism.

**What fails:**
- **Moderator accountability.** Moderators are unelected autocrats within their subreddit. There is no formal mechanism for the community to override a moderator decision. This violates Ostrom's Principle 3.
- **No nesting.** Subreddits are flat -- there is no sub-subreddit mechanism. This means every subreddit is either too general (millions of members, impossible to moderate) or too specific (too few members to sustain). The lack of nesting is Reddit's most fundamental structural limitation.
- **No data sovereignty.** Reddit (the company) can and does override subreddit governance. The 2023 API pricing revolt demonstrated that the platform layer can destroy community governance at will. Users cannot fork their community and take it elsewhere because they do not own the data.
- **Inconsistent enforcement.** Each subreddit is a "private kingdom" with no training, no standards, and no accountability. The decentralization that enables diversity also enables abuse.

**Lessons for Benten:**
- **Reddit's flat structure is the anti-pattern.** Groves MUST support nesting, or they will hit the same scaling wall that subreddits hit.
- **Moderator selection must be formal.** The governance operation subgraph should define how moderators are chosen, not leave it to platform convention.
- **Data sovereignty is the exit right.** Reddit users cannot leave because they cannot take their community. Benten users can fork the subgraph. This changes the power dynamic fundamentally.

### 2.5 Matrix Protocol: Spaces and Room Hierarchies

**Structure:** Spaces (containers) -> Rooms (conversations) -> Sub-Spaces -> Rooms. Spaces are themselves rooms with special state events that define the hierarchy. A room can belong to multiple spaces.

**What works:**
- **Rooms as the universal primitive.** Spaces are rooms. The hierarchy is defined by state events within rooms, not by a separate hierarchy system. This is the Benten philosophy: use the same primitives at every level.
- **Multi-parent membership.** A room can appear in multiple spaces. This is polycentric: a #cooking room can be in both the Food space and the Hobbyist space.
- **Federated governance.** Each homeserver has sovereignty over its data. The power level system within rooms defines local governance. No central authority can override a homeserver's decisions.
- **Spaces can contain sub-spaces.** True nesting. A "Science" space can contain a "Physics" space, which contains a "Quantum Mechanics" space.

**What fails:**
- **Power levels are numeric, not semantic.** A user with power level 50 can do... what exactly? The mapping from numbers to capabilities is per-room configuration. This is flexible but opaque.
- **No governance inheritance.** A sub-space does not automatically inherit the parent space's governance rules. Each room is independently configured. This means that creating a hierarchy of 100 rooms requires configuring governance 100 times.
- **No formal voting or proposal system.** Governance is whoever has the highest power level. There is no built-in mechanism for collective decision-making.

**Lessons for Benten:**
- **Governance inheritance is the killer feature Matrix lacks.** If a child Grove automatically inherits parent governance (with override capability), creating nested structures becomes trivial instead of painful.
- **Multi-parent membership (a Grove in multiple parent Groves) is proven in Matrix.** The data model supports it.
- **Semantic capabilities, not numeric power levels.** Benten's capability Nodes with explicit domains and actions are far more expressive than Matrix's integer power levels.

### 2.6 DAO Platforms: Aragon and Colony

**Aragon:**
- Modular DAO framework with plugin architecture.
- Moving toward "automated governance" -- rule-based execution that reduces the need for constant proposal voting.
- Token Ownership Index to verify which tokens grant actual control rights.
- **Limitation:** Flat DAO structure. No native sub-DAO nesting. Sub-DAOs require deploying separate DAO contracts.

**Colony:**
- Domain hierarchy: DAOs are divided into domains (teams), each with specific responsibilities.
- Reputation-based voting: influence weighted by contribution, not token holdings. Reputation decays over time, ensuring influence tracks current participation.
- "Lazy consensus": decisions proceed unless objected to within a time window. This reduces governance overhead for routine operations.
- Domains are hierarchical: a DAO can have sub-domains, sub-sub-domains, etc.
- Permission delegation: peripheral domains can make unilateral decisions within their budget.

**Colony's domain hierarchy is the closest existing implementation to Benten's fractal Grove concept.** But Colony's hierarchy is within a single DAO instance on a single blockchain. It does not support cross-instance federation, forking, or polycentric membership.

**Lessons for Benten:**
- **Colony's domain model validates the concept.** Hierarchical domains with delegated authority and reputation-weighted governance work in practice.
- **Lazy consensus reduces governance fatigue.** Not every decision needs a vote. Operation subgraphs should support "proceed unless objected" as a governance mode.
- **Reputation decay is important.** Governance influence should track current participation, not historical contribution. This can be modeled as capability Nodes with time-based attenuation.
- **The blockchain constraint is what Benten transcends.** Colony is limited to one chain, one instance. Benten's subgraph sync means governance structures can span instances, fork, and federate -- things no blockchain DAO can do.

### 2.7 Git: Fork/Upstream Sync Model

**Structure:** Origin repository -> Fork -> Local development -> Pull request back to upstream (optional). Forks can diverge indefinitely or stay synced.

**What works:**
- **Fork is atomic and complete.** You get the entire history, not just the current state. The fork includes all the decisions that led to the current state.
- **Upstream sync is selective.** You can cherry-pick which upstream changes to incorporate. You are not forced to accept everything.
- **Divergence is visible.** Git tracks how far your fork has diverged from upstream. You can see exactly which changes are unique to your fork.
- **Pull requests as governance.** Contributing back to upstream requires approval from upstream maintainers. This is a formal governance mechanism for accepting external contributions.
- **Branching within a fork.** You can experiment with changes without affecting your main branch. This is governance experimentation at zero cost.

**What fails:**
- **No continuous sync.** Git forks do not automatically stay synced. You must manually fetch and merge upstream changes. For governance, this means a forked community must actively choose to stay current with the parent.
- **Merge conflicts.** When both upstream and fork modify the same code, manual conflict resolution is required. For governance, this means: what happens when both the parent and child Grove modify the same rule?
- **No multi-upstream.** A Git repository has one upstream. You can add multiple remotes, but the tooling assumes a single primary upstream. For polycentric governance, you need multiple upstream relationships.

**Lessons for Benten:**
- **Fork should be atomic and include history.** When you fork a Grove, you get the entire governance version chain, not just the current rules.
- **Continuous sync should be opt-in and selective.** A child Grove can choose to stay synced with parent governance (receiving updates automatically) or to diverge (receiving nothing). Partial sync -- accepting some parent governance changes but not others -- should also be supported.
- **Multi-upstream is essential for polycentricity.** A Grove must be able to sync governance from multiple parents simultaneously. This is the key extension beyond Git's model.

### 2.8 Noosphere / Subconscious Network

**Structure:** Individual notebooks -> linked via CIDs -> namespace system -> social graph of subscriptions. Built on content-addressed data with UCAN capabilities.

**What works:**
- **Subsidiarity as a design principle.** "Following Elinor Ostrom, Noosphere values self-determination and local governance, beginning with individuals and small Dunbar-scale communities (the Cozyweb), with governance decisions made from the bottom-up at the lowest practical level."
- **Credible exit.** Users own their data. They can leave any service without losing what they created. This is data sovereignty, the precondition for governance competition.
- **Content-addressed linking.** References between notebooks are CIDs. If the content changes, the CID changes. This provides tamper evidence for governance references.

**Lessons for Benten:** Noosphere validates the philosophical approach (Ostrom + content addressing + local-first) but does not implement community governance structures. Benten goes further by making governance itself a graph structure that can be forked, synced, and evaluated.

---

## 3. The Graph Model: How Governance Maps to Nodes and Edges

### 3.1 A Grove Is a Subgraph

A Grove is not a Node. A Grove is a **TraversalPattern** -- a subgraph defined by a query pattern. Everything reachable from a specific anchor Node via specific edge types is "in" the Grove.

```
[Grove Anchor: "national-cooking"]
    |--[HAS_GOVERNANCE]--> [Governance Subgraph Anchor]
    |                          |--[HAS_RULE]--> [Rule: food-safety-standard]
    |                          |--[HAS_RULE]--> [Rule: membership-voting-process]
    |                          |--[HAS_RULE]--> [Rule: dispute-resolution]
    |                          |--[HAS_ROLE]--> [Role: moderator]
    |                          |--[HAS_ROLE]--> [Role: member]
    |                          |--[HAS_SANCTION]--> [Sanction: graduated-enforcement]
    |
    |--[HAS_MEMBER]--> [Member: alice (capabilities: full)]
    |--[HAS_MEMBER]--> [Member: bob (capabilities: read-only)]
    |
    |--[HAS_CHILD_GROVE]--> [Grove Anchor: "louisiana-cooking"]
    |--[HAS_CHILD_GROVE]--> [Grove Anchor: "california-cooking"]
    |
    |--[HAS_CONTENT]--> [Content Node: recipe-1]
    |--[HAS_CONTENT]--> [Content Node: recipe-2]
```

The governance subgraph is itself made of Nodes and Edges. Rules are Nodes. Roles are Nodes. Sanctions are Nodes. The governance structure is data, not metadata. It is queryable, versionable, syncable, and forkable -- because it is made of the same primitives as everything else.

### 3.2 A Rule Is an Operation Subgraph

From `explore-self-evaluating-graph.md`, we know that computation is expressed as operation Node subgraphs. A governance rule is an operation subgraph that the engine evaluates when a relevant action occurs.

```
[Rule: food-safety-standard]
    |--[APPLIES_TO]--> [Action: create-recipe]
    |--[FIRST_STEP]--> [ValidateOp: check field "ingredients" exists]
    |                      |--[ON_FAILURE]--> [RejectOp: "Recipe must list ingredients"]
    |                      |--[NEXT_STEP]--> [ValidateOp: check field "allergens" exists]
    |                                           |--[ON_FAILURE]--> [RejectOp: "Recipe must list allergens"]
    |                                           |--[NEXT_STEP]--> [AllowOp: proceed]
    |--[ENACTED_BY]--> [Proposal: prop-47 (content hash: a3f7...)]
    |--[VERSION]--> v3 (hash: b2c8...) <-- v2 (hash: 91d4...) <-- v1 (hash: f0e2...)
```

When a member attempts to create a recipe, the engine:
1. Identifies the applicable Grove (from the member's capabilities and the target content scope).
2. Queries the governance subgraph for rules that APPLY_TO `create-recipe`.
3. Evaluates each matching rule's operation subgraph.
4. If any rule rejects, the write is blocked.

This is not hypothetical -- this is exactly how operation Nodes work in the specification. Governance rules are operation subgraphs with a specific trigger (APPLIES_TO) and a provenance chain (ENACTED_BY).

### 3.3 A Proposal Is a Version Chain

A governance proposal to change a rule is itself a Node with a version chain:

```
[Proposal: prop-47]
    |--[PROPOSES_CHANGE]--> [Rule: food-safety-standard]
    |--[NEW_RULE_VERSION]--> [Draft Rule: food-safety-standard-v4 (content hash: d5e9...)]
    |--[VOTING_PROCESS]--> [VotingConfig: {quorum: 51%, threshold: 67%, duration: 7d}]
    |
    |--[HAS_VOTE]--> [Vote: alice-approve (signed by alice.did, hash: 1a2b...)]
    |--[HAS_VOTE]--> [Vote: bob-reject (signed by bob.did, hash: 3c4d...)]
    |--[HAS_VOTE]--> [Vote: carol-approve (signed by carol.did, hash: 5e6f...)]
    |
    |--[RESULT]--> [TallyNode: {approve: 2, reject: 1, abstain: 0, outcome: PASSED}]
    |--[ENACTED_AT]--> [Version: v4 of food-safety-standard (hash: d5e9...)]
```

Every vote is a signed Node. The tally is a deterministic computation over the vote Nodes. The proposal references the exact content hash of the rule change, so bait-and-switch is impossible. The enacted version is linked to the proposal that authorized it. The entire governance history is traversable.

### 3.4 Capability Grants Flow Downward

A Grove's governance defines what capabilities its members hold. A sub-Grove inherits its parent's capabilities by default, but the parent can constrain what the child can grant:

```
[Parent Grove: national-cooking]
    |--[GRANTS_TO_CHILDREN]--> [CapabilityTemplate: {
    |                               domain: "content",
    |                               actions: ["create", "read", "update"],
    |                               scope: "recipe/*",
    |                               attenuate: true  // children can narrow but not widen
    |                           }]
    |
    |--[HAS_CHILD_GROVE]--> [Child Grove: louisiana-cooking]
                                |--[LOCAL_CAPABILITY]--> [CapabilityGrant: {
                                |                           domain: "content",
                                |                           actions: ["create", "read"],  // narrower than parent
                                |                           scope: "recipe/cajun/*",       // narrower than parent
                                |                       }]
                                |
                                |--[LOCAL_CAPABILITY]--> [CapabilityGrant: {
                                |                           domain: "content",
                                |                           actions: ["create", "read", "update", "DELETE"],
                                |                           scope: "recipe/*",
                                |                       }]
                                |                       // INVALID: child cannot grant "delete" 
                                |                       // because parent did not grant it.
                                |                       // Capability attenuation enforced by engine.
```

**Attenuation is the key mechanism.** A child Grove cannot grant capabilities that its parent did not grant. Capabilities can only narrow (attenuate) as they flow down the hierarchy. This is the UCAN attenuation principle applied to governance.

This means:
- The national cooking Grove says "members can create, read, and update recipes."
- The Louisiana chapter can further restrict: "our members can only create and read Cajun recipes."
- But the Louisiana chapter CANNOT say "our members can delete recipes" if the national Grove did not grant delete.

### 3.5 Governance Inheritance Model

When a child Grove evaluates a governance rule, the resolution order is:

1. **Check child Grove's governance subgraph** for a rule matching the action.
2. If the child has an explicit rule, apply it (local override).
3. If the child does NOT have a rule for this action, **traverse the HAS_CHILD_GROVE edge upward** to the parent Grove.
4. Check the parent's governance subgraph.
5. Repeat up the ancestry chain until a matching rule is found or the root is reached.
6. If no rule is found at any level, the default is DENY (fail-closed).

This is prototypal inheritance for governance. The child's rules shadow the parent's rules for the same action. Unshadowed parent rules are inherited automatically.

```
Resolution for "can alice create a recipe in louisiana-cooking?":

1. louisiana-cooking governance: no explicit create-recipe rule
2. Traverse HAS_CHILD_GROVE^-1 to national-cooking
3. national-cooking governance: Rule "food-safety-standard" APPLIES_TO create-recipe
4. Evaluate food-safety-standard operation subgraph against alice's data
5. food-safety-standard passes -> check capabilities
6. alice has capability: content:create:recipe/cajun/* in louisiana-cooking
7. Recipe scope is recipe/cajun/gumbo -> within capability scope
8. ALLOWED
```

**Override semantics:**

A child Grove can override a parent rule in three ways:

1. **REPLACE**: The child defines a rule for the same action. The child's rule runs INSTEAD OF the parent's rule.
2. **EXTEND**: The child defines an additional rule for the same action. BOTH the parent's rule and the child's rule must pass. The child's rule adds constraints; it cannot relax them.
3. **EXEMPT**: The child explicitly opts out of a parent rule. This requires the parent to have ALLOWED exemptions in the rule's metadata. Not all rules are exemptible.

```
[Child Rule: local-ingredient-preference]
    |--[OVERRIDES]--> [Parent Rule: food-safety-standard]
    |--[OVERRIDE_MODE]--> "EXTEND"    // both rules must pass
    |--[FIRST_STEP]--> [ValidateOp: check field "local_sourcing" >= 50%]
    |                      |--[ON_FAILURE]--> [RejectOp: "At least 50% local ingredients required"]
    |                      |--[NEXT_STEP]--> [AllowOp: proceed]
```

### 3.6 Governance Versioning

Because rules are Nodes with version chains, and because changes to rules are enacted through proposals, the entire governance history is traversable:

```
Timeline:
  t0: Grove created with governance v1 (3 rules)
  t1: Proposal 1 passed, Rule A updated to v2
  t2: Proposal 2 passed, Rule B added
  t3: Proposal 3 failed (insufficient votes)
  t4: Proposal 4 passed, Rule A updated to v3

Each proposal references the exact version of the rule it modifies.
Each rule version has a content hash.
The governance state at any point in time can be reconstructed by traversing version chains.
```

This means: if a sub-Grove forks at time t2, and the parent Grove later updates Rule A to v3, the fork still has Rule A at v2. If the fork later decides to re-sync, it can compare content hashes to identify what changed.

---

## 4. Fractal Nesting: Groves Within Groves

### 4.1 The Nesting Structure

```
[Grove: World Cooking Federation]
    |--[HAS_CHILD_GROVE]--> [Grove: North America Cooking]
    |                           |--[HAS_CHILD_GROVE]--> [Grove: USA Cooking]
    |                           |                           |--[HAS_CHILD_GROVE]--> [Grove: Louisiana Cooking]
    |                           |                           |                           |--[HAS_CHILD_GROVE]--> [Grove: New Orleans Cooking]
    |                           |                           |
    |                           |                           |--[HAS_CHILD_GROVE]--> [Grove: California Cooking]
    |                           |
    |                           |--[HAS_CHILD_GROVE]--> [Grove: Canada Cooking]
    |
    |--[HAS_CHILD_GROVE]--> [Grove: Europe Cooking]
                                |--[HAS_CHILD_GROVE]--> [Grove: France Cooking]
                                |--[HAS_CHILD_GROVE]--> [Grove: Italy Cooking]
```

Each level of nesting uses the SAME governance primitives:
- Grove anchor Node
- Governance subgraph (rules, roles, sanctions, voting processes)
- Membership (capability grants)
- Content scope

The "self-similar" property means: the mechanism by which World Cooking Federation governs is structurally identical to the mechanism by which New Orleans Cooking governs. The difference is only in scope and specificity.

### 4.2 What Flows Down the Hierarchy

| Element | Inheritance Behavior |
|---|---|
| **Rules** | Prototypal inheritance with REPLACE/EXTEND/EXEMPT overrides |
| **Capabilities** | Attenuation only (can narrow, cannot widen) |
| **Roles** | Inherited by default, child can add local roles |
| **Sanctions** | Inherited by default, child can add but not remove graduated steps |
| **Voting processes** | Inherited by default, child can override for local decisions |
| **Content** | NOT inherited. Each Grove owns its own content. Parent content is accessible only if explicitly shared. |
| **Membership** | NOT inherited. Being a member of the parent does not automatically make you a member of the child. Membership is per-Grove. |

### 4.3 The Recursion Limit

Fractal nesting can theoretically be infinite. In practice, several mechanisms constrain depth:

1. **Capability attenuation.** Each level can only narrow capabilities. After enough levels, the capabilities become so narrow that the child Grove cannot meaningfully operate. This is natural -- a sub-sub-sub-Grove of a specific cuisine in a specific neighborhood has very narrow scope by definition.

2. **Governance resolution performance.** Each governance check traverses the ancestry chain. Deeper nesting means more hops. IVM mitigates this: the effective governance rules for a Grove are a materialized view that is incrementally maintained. The traversal happens once (when the view is created or updated), not on every action.

3. **Social scaling.** Ostrom's research shows that commons governance works best at Dunbar-scale groups (~150 people). Nesting allows larger organizations, but each individual Grove should be at human scale. The hierarchy manages inter-group coordination; intra-group governance stays intimate.

4. **Explicit depth limit.** A parent Grove can set a `maxDepth` property on its governance, limiting how deeply sub-Groves can nest. This is optional -- most Groves will not need it.

### 4.4 Cross-Level Operations

Some governance operations necessarily span levels:

**Member ban that propagates down:**
When the World Cooking Federation bans a member, should the ban propagate to all sub-Groves? This depends on the rule:

```
[Rule: global-ban-propagation]
    |--[APPLIES_TO]--> [Action: ban-member]
    |--[PROPAGATION]--> "CASCADE"  // or "LOCAL_ONLY" or "ADVISORY"
```

- **CASCADE**: The ban applies to all sub-Groves. The member loses capabilities at every level.
- **LOCAL_ONLY**: The ban applies only at the level where it was issued. Sub-Groves can choose to honor it or not.
- **ADVISORY**: The ban is communicated to sub-Groves as a recommendation. Each sub-Grove's governance decides whether to honor it.

**Resource allocation across levels:**
If the World Cooking Federation has a storage quota, it can allocate portions to sub-Groves:

```
[CapabilityGrant: storage-quota]
    |--[TOTAL]--> 100GB
    |--[ALLOCATED_TO: North America]--> 40GB
    |--[ALLOCATED_TO: Europe]--> 40GB
    |--[RESERVED]--> 20GB
```

Sub-Groves can further subdivide their allocation. The total allocated to children cannot exceed the parent's allocation (same attenuation principle as capabilities).

---

## 5. Polycentric Governance: Multiple Parent Dimensions

### 5.1 The Multi-Parent Problem

A local environmental group can belong to:
- A national environmental federation (governance dimension: environmental policy)
- A city governance federation (governance dimension: municipal policy)
- A regional arts council (governance dimension: cultural programming)

Each parent governance structure has legitimate authority over different aspects of the local group's behavior. This is not a single hierarchy -- it is a DAG (directed acyclic graph) of governance relationships.

```
[Grove: Local Environmental Art Group]
    |--[FEDERATED_WITH]--> [Grove: National Environmental Federation]
    |                          |--[GOVERNANCE_DOMAIN]--> "environmental-policy"
    |                          |--[AUTHORITY_SCOPE]--> "environmental-impact/*"
    |
    |--[FEDERATED_WITH]--> [Grove: City Governance Federation]
    |                          |--[GOVERNANCE_DOMAIN]--> "municipal-compliance"
    |                          |--[AUTHORITY_SCOPE]--> "local-events/*, permits/*"
    |
    |--[FEDERATED_WITH]--> [Grove: Regional Arts Council]
                               |--[GOVERNANCE_DOMAIN]--> "cultural-programming"
                               |--[AUTHORITY_SCOPE]--> "exhibitions/*, performances/*"
```

### 5.2 Governance Domain Scoping

The key mechanism for polycentric governance is **domain-scoped authority**. Each parent Grove's authority is limited to a specific domain. The child Grove's governance resolution considers ALL parent Groves, but each parent's rules only apply within their declared authority scope.

```
Resolution for "can local-group host an outdoor art exhibition?":

1. Check local-group's own governance: no explicit rule for outdoor-exhibitions
2. Check parent: National Environmental Federation
   - Authority scope: environmental-impact/*
   - Rule: "environmental-impact-assessment required for outdoor events > 100 people"
   - Action scope: this exhibition is 200 people -> APPLIES
   - Result: must pass environmental impact assessment
3. Check parent: City Governance Federation
   - Authority scope: local-events/*, permits/*
   - Rule: "outdoor events require city permit"
   - Action scope: outdoor event -> APPLIES
   - Result: must have city permit
4. Check parent: Regional Arts Council
   - Authority scope: exhibitions/*, performances/*
   - Rule: "exhibitions must be open to public for at least 4 hours"
   - Action scope: exhibition -> APPLIES
   - Result: must be open 4+ hours
5. ALL applicable rules must pass (they govern different aspects)
6. ALLOWED if: environmental assessment passes AND city permit obtained AND open 4+ hours
```

### 5.3 Conflict Resolution Between Parents

What happens when two parent Groves have conflicting rules over the same domain?

**Case 1: Non-overlapping domains (no conflict).** The National Environmental Federation governs environmental policy. The City Federation governs permits. They do not overlap. No conflict is possible.

**Case 2: Overlapping domains (potential conflict).** Two parent Groves both claim authority over "events/*". The National Federation says "events must have recycling bins." The City Federation says "events must not have bins on public sidewalks."

Resolution strategies (configured per-Grove):

1. **Explicit priority.** The child Grove declares which parent takes precedence for which domain:
   ```
   [ConflictResolution: events-domain]
       |--[PRIMARY]--> [National Environmental Federation]
       |--[SECONDARY]--> [City Governance Federation]
       |--[SCOPE]--> "events/*"
   ```
   The primary parent's rules take precedence. The secondary parent's rules apply only where the primary is silent.

2. **Union (strictest wins).** All parent rules apply. When they conflict, the most restrictive interpretation prevails. The event must have recycling bins (national rule) AND those bins must not be on the sidewalk (city rule). Both constraints apply.

3. **Local override.** The child Grove writes its own rule for the conflicting domain, superseding both parents. This is only possible if both parents ALLOW local overrides for that domain.

4. **Mediation.** The child Grove requests mediation from an agreed-upon mediator Grove. The mediator evaluates both parent rules and issues a binding resolution. This is Ostrom's Principle 6 applied to inter-governance conflicts.

### 5.4 The Polycentric Governance DAG

The parent-child relationships form a DAG, not a tree. A cycle (Grove A is parent of B, B is parent of A) must be prevented -- this is the same cycle detection that `explore-self-evaluating-graph.md` describes for operation subgraphs.

```
Governance DAG:

    [Global Environmental Council]
            |                    \
            v                     v
    [National Env Fed]     [International Arts Council]
            |                    |           \
            v                    v            v
    [City Env Coalition]   [Regional Arts]   [National Arts Fed]
            \                   /
             v                 v
        [Local Environmental Art Group]
```

The Local Environmental Art Group has two parent chains that converge at different levels. Governance resolution must handle diamond dependencies: if both the City Environmental Coalition and Regional Arts Council inherit a rule from a shared ancestor, that rule should be applied once, not twice.

**Diamond resolution:** Use the same hybrid algorithm that the CMS materializer uses for composition diamond refs (from `SPECIFICATION.md` context: `resolved` cache for global deduplication + `visiting` set per-path for cycle detection). A governance rule identified by content hash is evaluated once, regardless of how many parent paths lead to it.

---

## 6. Polyfederation: Fork, Sync, Diverge

### 6.1 Forking a Grove

"Forking" a Grove means creating a new Grove whose initial governance subgraph is a copy of the source Grove's governance subgraph. The fork gets:

- All governance rules (as version Nodes, including history)
- All role definitions
- All sanction configurations
- All voting process definitions
- The capability template structure

The fork does NOT get:
- The source Grove's content (unless explicitly included in the fork scope)
- The source Grove's membership (the fork starts with the forker as sole member)
- Any ongoing relationship with the source Grove (unless the forker establishes one)

```
Forking "National Cooking" to create "Organic Cooking":

Source: [Grove: national-cooking]
    |--[HAS_GOVERNANCE]--> [Governance v7 (hash: abc123)]
            |--[HAS_RULE]--> [Rule: food-safety (v3, hash: def456)]
            |--[HAS_RULE]--> [Rule: voting-process (v2, hash: ghi789)]

Fork: [Grove: organic-cooking]
    |--[HAS_GOVERNANCE]--> [Governance v1 (hash: abc123)]  // SAME hash initially
            |--[HAS_RULE]--> [Rule: food-safety (v3, hash: def456)]  // SAME content
            |--[HAS_RULE]--> [Rule: voting-process (v2, hash: ghi789)]  // SAME content
    |--[FORKED_FROM]--> [Grove: national-cooking (at governance hash: abc123)]
```

At the moment of fork, the two Groves have identical governance (verified by content hash). From this point, they can diverge.

### 6.2 Staying Synced with Upstream

After forking, the organic-cooking Grove can choose to stay synced with national-cooking's governance updates:

```
[SyncRelationship: organic-cooking -> national-cooking]
    |--[SYNC_MODE]--> "SELECTIVE"
    |--[SYNC_SCOPE]--> ["governance/rules/food-safety/*"]  // only sync food safety rules
    |--[OVERRIDE_LOCAL]--> false  // local changes take precedence over upstream
    |--[LAST_SYNCED]--> governance hash: abc123
```

**Sync modes:**

1. **FULL**: The child accepts all governance changes from upstream. The child's governance is always a superset of the parent's (child can add rules but not remove inherited ones).

2. **SELECTIVE**: The child syncs only specific domains of governance from upstream. In the example, organic-cooking syncs food safety rules but makes its own voting process rules.

3. **ADVISORY**: The child receives upstream governance changes as proposals, not automatic updates. Each change must be approved by the child's governance process before taking effect.

4. **NONE**: No sync. The fork has fully diverged. The FORKED_FROM edge remains for provenance, but no data flows.

### 6.3 The Sync Mechanism

Governance sync uses the same content-addressed sync as all other subgraph sync in the engine:

1. The upstream Grove publishes a new governance version: `Governance v8 (hash: xyz789)`.
2. The synced child Grove's engine receives the update (via the CRDT sync protocol).
3. The engine compares `governance v8 hash: xyz789` against the child's current governance `v1 hash: abc123`.
4. The diff is computed: which rules changed, which were added, which were removed.
5. Based on the sync mode:
   - FULL: all changes are applied automatically.
   - SELECTIVE: only changes within the sync scope are applied.
   - ADVISORY: changes are presented as proposals to the child's governance.
6. If the child has local overrides for a changed rule, the merge strategy determines the result:
   - Upstream wins: the override is discarded.
   - Local wins: the override is preserved.
   - Conflict: the rule is flagged for manual resolution.

### 6.4 Governance Competition

This is the most radical implication of polyfederation: **governance itself is subject to competitive pressure.**

Scenario: Three cooking communities fork from the same source:
- Fork A keeps strict food safety rules, adds sustainability requirements.
- Fork B relaxes food safety rules, adds more creative freedom.
- Fork C keeps food safety rules, adds reputation-weighted voting.

Over time, members migrate to the community whose governance best serves their needs. Fork A attracts safety-conscious members. Fork B attracts experimental chefs. Fork C attracts members who value meritocratic governance.

The forks can observe each other's governance (if public) and adopt successful innovations. Fork A might see that Fork C's reputation-weighted voting reduces governance fatigue and adopt it. This is exactly the "laboratories of democracy" concept from federalism -- but applied to community governance, not state government.

**The engine enables this by making governance observable, forkable, and mergeable.** Governance is not opaque policy text -- it is executable operation subgraphs with content hashes. You can programmatically compare two Groves' governance, identify differences, and selectively merge.

### 6.5 Wrapping Existing Communities

A Grove can "wrap" an existing community by syncing with its data and governance while adding its own governance layer on top:

```
[Grove: curated-cooking (wrapper)]
    |--[WRAPS]--> [Grove: national-cooking (wrapped)]
    |--[SYNC_MODE]--> "FULL"  // mirror all content
    |--[ADDITIONAL_GOVERNANCE]--> [Rule: curator-approval-required]
    |--[ADDITIONAL_GOVERNANCE]--> [Rule: quality-rating-minimum]
```

The wrapper Grove:
- Syncs all content from the wrapped Grove.
- Applies its own additional governance rules (e.g., only showing content that passes a quality filter).
- Does NOT modify the wrapped Grove's data.
- Members of the wrapper see a curated view of the wrapped community.

This is the "polyfederated" pattern: the wrapper Grove is a new governance dimension applied to an existing community's data, without requiring permission from or modification of the original community.

---

## 7. Technical Implementation: Graph Primitives

### 7.1 New Node Labels

| Label | Purpose |
|---|---|
| `Grove` | Anchor Node for a community |
| `GovernanceRule` | An operation subgraph that validates actions |
| `GovernanceRole` | A named set of capabilities within a Grove |
| `GraduatedSanction` | An escalating enforcement policy |
| `VotingProcess` | Configuration for collective decision-making |
| `Proposal` | A proposed governance change |
| `Vote` | A signed vote on a proposal |
| `Tally` | The computed result of a voting process |
| `ConflictResolution` | A dispute resolution process |
| `SyncRelationship` | Configuration for upstream governance sync |
| `CapabilityTemplate` | What a parent Grove grants to children |

### 7.2 New Edge Types

| Edge Type | From | To | Purpose |
|---|---|---|---|
| `HAS_GOVERNANCE` | Grove | GovernanceRule/Role/Sanction | Links Grove to its governance subgraph |
| `HAS_CHILD_GROVE` | Grove | Grove | Fractal nesting |
| `FEDERATED_WITH` | Grove | Grove | Polycentric federation (multi-parent) |
| `FORKED_FROM` | Grove | Grove | Provenance: where governance was forked from |
| `WRAPS` | Grove | Grove | Polyfederation: governance overlay |
| `SYNCS_FROM` | Grove | Grove | Active governance sync relationship |
| `APPLIES_TO` | GovernanceRule | Action pattern | What actions trigger this rule |
| `OVERRIDES` | GovernanceRule | GovernanceRule | Child rule overriding parent rule |
| `ENACTED_BY` | GovernanceRule version | Proposal | Which proposal authorized this rule |
| `HAS_VOTE` | Proposal | Vote | Links votes to proposals |
| `PROPAGATION` | GovernanceRule | Scope | How rule cascades to children |
| `GRANTS_TO_CHILDREN` | Grove | CapabilityTemplate | What capabilities children inherit |
| `AUTHORITY_SCOPE` | FEDERATED_WITH edge | Scope pattern | Limits parent authority in polycentrism |

### 7.3 Materialized Views for Governance

IVM maintains these views for O(1) governance checks:

1. **Effective rules view**: For each Grove, the fully resolved set of governance rules (after inheritance, overrides, and merges). Updated when any rule in the ancestry chain changes.

2. **Effective capabilities view**: For each member in each Grove, the fully resolved capability set (after attenuation through the hierarchy). Updated when capabilities change at any level.

3. **Active proposals view**: For each Grove, the set of proposals currently in voting period. Updated when proposals are created or voting periods expire.

4. **Governance diff view**: For each sync relationship, the set of upstream changes not yet applied locally. Updated when upstream governance changes or local sync is applied.

### 7.4 Governance Resolution Algorithm

```
function resolveGovernance(grove: GroveId, action: Action): GovernanceResult {
    // 1. Get effective rules from materialized view (O(1))
    const rules = effectiveRulesView.get(grove, action);
    
    // 2. For each applicable rule, evaluate its operation subgraph
    for (const rule of rules) {
        const result = evaluateOperationSubgraph(rule.operationSubgraph, {
            action,
            actor: currentActor,
            grove,
            context: actionContext
        });
        
        if (result === REJECT) {
            return { allowed: false, reason: rule.rejectReason, rule: rule.id };
        }
    }
    
    // 3. Check capabilities (also O(1) from materialized view)
    const capabilities = effectiveCapabilitiesView.get(grove, currentActor);
    if (!matchesCapability(capabilities, action)) {
        return { allowed: false, reason: "Insufficient capabilities" };
    }
    
    return { allowed: true };
}
```

The critical insight: because of IVM, the "effective rules" and "effective capabilities" are pre-computed. Governance resolution is not a recursive traversal at action time -- it is a lookup. The recursive traversal happens when governance CHANGES, not when governance is CHECKED.

---

## 8. Ostrom's Principles Revisited: A Complete Mapping

Now that the technical model is defined, here is the complete mapping of Ostrom's eight principles to Benten's graph primitives:

### Principle 1: Clearly Defined Boundaries

**Mechanism:** Subgraph boundaries + capability scopes. A Grove's membership is the set of entities with capability Nodes scoped to that Grove. The boundary is cryptographically precise: you have a capability Node signed by a Grove authority, or you don't.

**Enforcement:** The engine checks capabilities at every operation boundary. There is no "gray area" membership. The boundary IS the set of granted capabilities.

### Principle 2: Congruence with Local Conditions

**Mechanism:** Governance rules are operation subgraphs written by the community. No universal rules are imposed. The national cooking Grove writes food safety rules appropriate for national-scale cooking. The New Orleans sub-Grove writes rules appropriate for New Orleans cooking culture.

**Enforcement:** Each Grove's governance subgraph is independent. The engine evaluates whatever rules the Grove has defined. The platform does not impose governance -- it provides the mechanism for communities to define their own.

### Principle 3: Collective-Choice Arrangements

**Mechanism:** VotingProcess Nodes define how governance changes are approved. The VotingProcess itself is governed by... a VotingProcess. This is the recursive foundation: the rules for changing rules are themselves rules.

**Bootstrap:** When a Grove is first created, the creator defines the initial VotingProcess. After creation, changing the VotingProcess requires the existing VotingProcess to approve the change. This prevents unilateral governance capture.

**Protection against capture:** The VotingProcess operation subgraph is content-hashed. Any modification is visible in the version chain. A governance change that modifies the voting process to concentrate power is detectable and auditable by any member with read access.

### Principle 4: Monitoring

**Mechanism:** The graph IS the audit trail. Every mutation is a signed version Node. Every governance action is an operation subgraph evaluation that produces a trace. Any member can query the governance history and verify compliance.

**Active monitoring:** Reactive subscriptions (from the engine specification) enable members to subscribe to governance changes. "Notify me when any rule in my Grove changes" is a subscription query pattern. This makes monitoring automatic, not manual.

### Principle 5: Graduated Sanctions

**Mechanism:** GraduatedSanction Nodes define escalation paths:
```
Level 1: Warning (capability: unchanged, notification: sent)
Level 2: Restricted (capability: attenuated to read-only for 24h)
Level 3: Suspended (capability: revoked for 7 days)
Level 4: Expelled (capability: permanently revoked)
```

Each level references the conditions that trigger escalation and the conditions that trigger de-escalation. The sanction state for each member is a version chain: you can see the full sanction history.

### Principle 6: Conflict Resolution

**Mechanism:** ConflictResolution Nodes define dispute processes as operation subgraphs:
1. Filing: Member creates a Dispute Node referencing the contested action.
2. Selection: The conflict resolution operation subgraph selects mediators (could be random selection from qualified members, elected council, or designated role).
3. Hearing: Both parties submit Evidence Nodes.
4. Decision: Mediator(s) create a Resolution Node with reasoning.
5. Appeal: If the governance allows appeals, the process recurses to a higher level.

The entire dispute process is in the graph -- transparent, auditable, and versioned.

### Principle 7: Minimal Recognition of Rights to Organize

**Mechanism:** Fork capability. The platform cannot prevent a community from self-organizing because:
1. Members own their data (local-first, not server-dependent).
2. Any member can fork the Grove's subgraph.
3. The fork includes governance history, enabling continuity.
4. The platform provides the mechanism but does not constrain its use.

The engine has NO governance policy of its own. It provides governance PRIMITIVES (rules, votes, capabilities, sanctions) and lets communities compose them. This is mechanism/policy separation applied to governance.

### Principle 8: Nested Enterprises

**Mechanism:** Everything in this document. The HAS_CHILD_GROVE edges, governance inheritance, capability attenuation, FEDERATED_WITH edges, and SyncRelationship Nodes create a system of nested enterprises where:
- Local governance handles local issues.
- Parent governance handles cross-community issues.
- Multiple parent governance dimensions handle cross-domain issues.
- Fork/sync/diverge enables governance evolution.

---

## 9. Edge Cases and Hard Problems

### 9.1 The Cascading Ban Problem

Scenario: World Cooking Federation bans Alice. Alice is a member of 17 sub-Groves at various nesting levels. What happens?

**If CASCADE:** Alice's capabilities are revoked in all 17 sub-Groves. This is fast (capability revocation at the parent propagates via materialized view updates) but potentially unjust (maybe Alice's behavior was only problematic at the top level).

**If ADVISORY:** Each sub-Grove receives a "ban recommendation" Node. Each sub-Grove's governance decides independently. This is just, but slow (17 separate governance processes) and potentially inconsistent.

**Recommended default:** CASCADE for capability revocations by the parent (immediate safety), ADVISORY for everything else. Sub-Groves can override the CASCADE default by explicitly granting Alice local capabilities that bypass the parent's revocation -- but only if the parent's governance allows exemptions.

### 9.2 The Governance Fork Bomb

Scenario: A malicious actor forks a Grove 10,000 times, each fork syncing from the original. The original Grove now has 10,000 downstream sync relationships consuming bandwidth.

**Mitigation:** Sync is opt-in from BOTH sides. The upstream Grove can:
1. Set a maximum number of sync relationships (throttle).
2. Require capability exchange before establishing sync (the downstream Grove must present a valid capability).
3. Rate-limit sync requests.
4. Revoke sync relationships.

The fork itself is not a problem (creating a copy of the governance subgraph is local). The sync bandwidth is the attack vector, and it is addressed by the sync protocol's rate limiting and capability requirements.

### 9.3 The Polycentric Diamond Conflict

Scenario: A Grove is federated with parents A and B. Both A and B inherit a rule from common ancestor C. Ancestor C updates the rule. Both A and B sync the update. The child Grove now receives the same rule change from two paths.

**Resolution:** Content-addressed deduplication. The rule change has a content hash. The child Grove's sync engine recognizes that the two incoming changes (from A and from B) have the same content hash and applies the change once.

This is the same diamond-ref resolution from the CMS materializer, applied to governance sync.

### 9.4 The Frozen Governance Attack

Scenario: A Grove's governance requires 67% approval to change any rule. An attacker (or faction) acquires 34% of the voting power and vetoes every governance change. The governance is frozen -- it cannot evolve.

**Mitigations:**
1. **Governance timeout.** A rule that has not been reviewed/renewed within N time periods automatically expires. This forces periodic re-approval and prevents permanent governance lock-in.
2. **Fork as ultimate exit.** If governance is frozen, dissatisfied members fork and create a new Grove with updated governance. The frozen Grove loses members and becomes irrelevant. Governance competition resolves the deadlock.
3. **Supermajority decay.** The required threshold for governance changes decreases over time if no changes have passed. If the Grove has not passed a governance change in 6 months, the threshold drops from 67% to 60%. If 12 months, 55%. This creates pressure against permanent vetoes.

### 9.5 The Ancestry Chain Depth Problem

Scenario: A governance check requires traversing a 50-level ancestry chain to find the applicable rule.

**Resolution:** IVM. The effective rules view is pre-computed. The ancestry traversal happens when governance changes (which is rare -- governance changes are the slow path). When governance is checked (which is frequent -- every user action), the check is O(1) against the materialized view.

The IVM cost is paid at governance-change time, not at action-check time. Since governance changes are orders of magnitude less frequent than user actions, this is the correct optimization.

### 9.6 The Split-Brain Governance Problem

Scenario: Two instances of a Grove are partitioned (network split). Both instances process governance changes independently. When the partition heals, the governance states have diverged.

**Resolution:** Governance Nodes use the same CRDT merge as all other Nodes. Per-field last-write-wins with HLC for rule properties. Add-wins for edges (if both sides added different rules, both rules appear). Structural validation on merge (if the merged state violates governance constraints, a conflict Node is created for manual resolution).

The critical insight: governance CHANGES (proposals, votes, tallies) are append-only events. They do not conflict in the CRDT sense -- two votes from different partitions can both be valid. The TALLY is what might diverge, and re-computing the tally from the merged vote set resolves it deterministically.

---

## 10. Implementation Strategy

### Phase 1: Fractal Nesting (Foundation)
- Grove anchor Nodes with HAS_CHILD_GROVE edges
- Governance subgraphs with rules as operation subgraphs
- Prototypal inheritance for rule resolution
- Capability attenuation through the hierarchy
- Materialized views for effective rules and capabilities
- Basic voting (proposal, vote, tally) as operation subgraphs

### Phase 2: Polycentric Federation
- FEDERATED_WITH edges with AUTHORITY_SCOPE
- Multi-parent governance resolution with domain scoping
- Conflict resolution between parents (priority, union, local override)
- Diamond dependency deduplication
- Governance DAG cycle detection

### Phase 3: Polyfederation
- FORKED_FROM edges with governance provenance
- SyncRelationship Nodes (FULL, SELECTIVE, ADVISORY, NONE)
- Content-hash-based governance diff and merge
- Governance competition (observable, comparable governance)
- WRAPS edge for governance overlays

### Phase 4: Governance Evolution
- Governance timeout / auto-expiry
- Reputation-weighted voting (Colony-inspired)
- Lazy consensus (proceed unless objected)
- Graduated sanction automation
- Cross-Grove governance analytics (which governance patterns attract/retain members)

---

## 11. What Makes This Different From Everything Else

| System | What It Lacks That Benten Provides |
|---|---|
| **Reddit** | No nesting, no formal governance, no data sovereignty, no fork capability |
| **Matrix** | No governance inheritance, no formal voting, no fork/sync |
| **Aragon** | Flat DAO, blockchain-bound, no fractal nesting, no P2P sync |
| **Colony** | Single-chain, no cross-instance federation, no fork/diverge/merge |
| **Discord** | No nesting, no formal governance, no data sovereignty, centrally controlled |
| **Git** | Single upstream, no multi-parent, no governance evaluation, no IVM |
| **Noosphere** | Individual-focused, no community governance structures |
| **Federal Government** | Not digital, not forkable, not content-addressed, preemption creep |
| **Catholic Church** | No collective-choice, no fork/exit right, appointment not election |

Benten's unique combination:
1. **Governance as executable graph.** Rules are not policy documents -- they are operation subgraphs the engine evaluates.
2. **Fractal + polycentric + polyfederated.** All three simultaneously. No existing system combines them.
3. **Content-addressed governance.** Every rule has a hash. Every change has a provenance chain. Governance is tamper-evident.
4. **Fork as a first-class operation.** Governance competition is built into the system, not bolted on.
5. **IVM for governance resolution.** Checking governance is O(1). Changing governance is the expensive operation. This is the correct optimization for systems where governance changes rarely but is checked constantly.
6. **Ostrom-complete.** The system provides mechanisms for all eight design principles. No existing digital platform does this.

---

## 12. Open Questions

1. **Should governance rules be Turing-complete?** The self-evaluating graph exploration recommends "deliberately NOT Turing complete" for operation Nodes. This applies to governance rules too. A governance rule that loops forever is a denial-of-service attack on the Grove. Bounded evaluation (fuel/gas metering) is essential.

2. **How do we handle governance migration?** If a Grove has been operating for years with governance v47, and a member forks to create a new Grove, what is the "minimum viable governance" for the fork? Must the fork inherit the full history, or can it start fresh while acknowledging provenance?

3. **What is the maximum practical nesting depth?** Ostrom studied systems with 2-4 levels of nesting. Does the model break down at 10 levels? 50? The IVM resolution should handle it technically, but the cognitive load on humans managing deep hierarchies is a real concern.

4. **How do we prevent governance capture through voter apathy?** In many DAOs, voter participation drops below 10% within a year. The lazy consensus model (proceed unless objected) helps, but it also means a small active minority effectively governs. Is this acceptable?

5. **Should the platform provide governance templates?** "Start with a standard governance for a cooking community" vs. "build your governance from scratch." Templates accelerate adoption but create homogeneity. The engine should provide templates as forkable governance subgraphs, not as built-in policy.

6. **How do cross-instance governance audits work?** If Grove A (on instance 1) is a child of Grove B (on instance 2), and a member on instance 1 wants to verify that Grove B's governance is legitimate, they need to traverse governance Nodes that live on a different instance. Does this require full sync of the parent's governance, or can it work with Merkle proofs?

7. **What governance primitives should exist in the engine vs. in modules?** The engine should provide the mechanism (evaluate operation subgraph, check capability, version chain). The governance patterns (voting, sanctions, dispute resolution) could be modules. This keeps the engine thin while enabling governance diversity.

---

## Sources

### Elinor Ostrom and Polycentric Governance
- [Ostrom's Eight Design Principles for a Successfully Managed Commons](https://www.agrariantrust.org/ostroms-eight-design-principles-for-a-successfully-managed-commons/)
- [Elinor Ostrom's 8 Rules for Managing the Commons](https://earthbound.report/2018/01/15/elinor-ostroms-8-rules-for-managing-the-commons/)
- [Eight Design Principles for Successful Commons](https://patternsofcommoning.org/uncategorized/eight-design-principles-for-successful-commons/)
- [Beyond Markets and States: Polycentric Governance of Complex Economic Systems (Nobel Lecture)](https://www.nobelprize.org/uploads/2018/06/ostrom_lecture.pdf)
- [Polycentric Systems of Governance: A Theoretical Model for the Commons](https://onlinelibrary.wiley.com/doi/10.1111/psj.12212)
- [Polycentric Governance in Theory and Practice](https://mcginnis.pages.iu.edu/polycentric%20governance%20theory%20and%20practice%20Feb%202016.pdf)
- [A Revised Ostrom's Design Principles for Collective Governance of the Commons](https://www.lifewithalacrity.com/article/a-revised-ostroms-design-principles-for-collective-governance-of-the-commons/)
- [Blockchain as Commons: Applying Ostrom's Polycentric Approach](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4250547)

### Fractal Governance and Nested Organizations
- [Fractal Organization (Sociocracy 3.0)](https://patterns.sociocracy30.org/fractal-organization.html)
- [Fractal Governance: How Self-Similar Architectures Enable Multi-Level Coordination](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=6011775)
- [Fractal Organisation Theory](https://journals.isss.org/index.php/proceedings56th/article/viewFile/1796/663)
- [Circles in Sociocracy: An Effective Organizational Structure](https://www.sociocracyforall.org/organizational-circle-structure-in-sociocracy/)

### DAO Governance Platforms
- [Aragon: The Home of Onchain Organizations](https://www.aragon.org/)
- [Aragon Pushes Automated Governance to Replace Endless DAO Proposals](https://outposts.io/article/aragon-pushes-automated-governance-to-replace-endless-dao-535574ad-bcfb-470a-8203-8fa09abcb483)
- [Colony: Quick and Easy DAO Setup](https://colony.io/product)
- [Reputation-Based Voting in DAOs: Democratizing Governance](https://blog.colony.io/what-is-reputation-based-voting-governance-in-daos/)
- [DAO Tools Comparison: Aragon vs DAOstack vs Colony](https://www.rapidinnovation.io/post/dao-tools-comparison-aragon-vs-daostack-vs-colony)
- [DAO Development Complete Guide 2026](https://calmops.com/web3/dao-development-2026-complete-guide/)

### Matrix Protocol
- [Spaces and Room Organization (Matrix Spec Proposals)](https://deepwiki.com/matrix-org/matrix-spec-proposals/4.1-spaces-and-room-organization)
- [The Matrix Space Beta](https://matrix.org/blog/2021/05/17/the-matrix-space-beta/)
- [Matrix Specification: Rooms and Events](https://matrix.org/docs/matrix-concepts/rooms_and_events/)
- [MSC1772: Groups as Rooms](https://github.com/matrix-org/matrix-doc/blob/matthew/msc1772/proposals/1772-groups-as-rooms.md)

### Federal Governance and Preemption
- [The Supremacy Clause and the Doctrine of Preemption](https://www.findlaw.com/litigation/legal-system/the-supremacy-clause-and-the-doctrine-of-preemption.html)
- [Federal Preemption: A Legal Primer (Congressional Research Service)](https://www.congress.gov/crs-product/R45825)
- [How Is the Church Governed? (Catholic Project)](https://catholicproject.catholic.edu/how-is-the-church-governed/)

### Reddit Governance
- [The Federalists of the Internet? Reddit's Decentralized Content Moderation](https://lawreview.unl.edu/federalists-internet-what-online-platforms-can-learn-reddits-decentralized-content-moderation/)
- [Reddit Moderation is Broken: The Illusion of the Commons](https://www.real-morality.com/post/reddit-moderation-is-broken-the-illusion-of-the-commons/)

### Content-Addressed Data and CRDTs
- [Merkle DAGs (IPFS Documentation)](https://docs.ipfs.tech/concepts/merkle-dag/)
- [Merkle-CRDTs: Merkle-DAGs Meet CRDTs](https://arxiv.org/pdf/2004.00107)
- [Graph-Based Security and Entitlements: Transforming Access Control](https://enterprise-knowledge.com/graph-based-security-entitlements-transforming-access-control-for-the-modern-enterprise/)
- [Capability-Based Delegation Model in RBAC](https://dl.acm.org/doi/10.1145/1809842.1809861)
- [Graph-Powered Authorization: Relationship-Based Access Control (AWS)](https://aws.amazon.com/blogs/database/graph-powered-authorization-relationship-based-access-control-for-access-management/)

### Noosphere Protocol
- [Noosphere Design Principles](https://github.com/subconsciousnetwork/noosphere/blob/main/design/principles.md)
- [Noosphere: A Protocol for Thought](https://newsletter.squishy.computer/p/noosphere-a-protocol-for-thought)

### Bluesky / AT Protocol
- [Federation Architecture (Bluesky Documentation)](https://docs.bsky.app/docs/advanced-guides/federation-architecture)
- [Protocol Check-in Fall 2025 (Bluesky)](https://docs.bsky.app/blog/protocol-checkin-fall-2025)
- [Polycentricity and Legitimacy in Digital Governance (2025)](https://www.tandfonline.com/doi/full/10.1080/1369118X.2025.2552374)
- [Concluding Reflections: Polycentricity in Digital Governance (2026)](https://www.tandfonline.com/doi/full/10.1080/1369118X.2026.2624711)
