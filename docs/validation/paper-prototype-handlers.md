# Paper Prototype: 5 Real Handlers with 12 Operation Primitives

**Created:** 2026-04-11
**Purpose:** Empirical validation that 12 primitives can express real handlers without degenerating into SANDBOX calls.
**Context:** Feasibility critic demanded paper-prototyping before building. Completeness critic identified gaps in multi-node transactions, collaborative editing, and ephemeral state.

> **⚠️ Primitive set revised 2026-04-14 — this document uses the ORIGINAL 12.**
>
> The validation below was performed against the original set: READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, **GATE**, CALL, RESPOND, EMIT, SANDBOX, **VALIDATE**.
>
> Current authoritative set (see [`../ENGINE-SPEC.md`](../ENGINE-SPEC.md) Section 3): READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, CALL, RESPOND, EMIT, SANDBOX, **SUBSCRIBE**, **STREAM**. GATE and VALIDATE were dropped; SUBSCRIBE and STREAM were added.
>
> **What this means for readers of this document:**
> - The 2.5% SANDBOX-rate finding still holds — neither GATE nor VALIDATE was SANDBOX-adjacent.
> - Handler examples that use GATE should be read as "capability check" (now expressed via the `requires` property on any Node).
> - Handler examples that use VALIDATE should be read as composition of BRANCH + TRANSFORM + RESPOND(error).
> - **Re-validation against the revised 12 is a Phase 1 task.** The revised set is expected to maintain or improve the SANDBOX rate (SUBSCRIBE + STREAM cover patterns previously forced into awkward compositions).
>
> The original validation is retained here for historical accuracy. Do not build Phase 1 handlers from this document — use ENGINE-SPEC + DSL-SPECIFICATION for the current primitive set.

---

## Primitives Reference (Quick)

| # | Primitive | Core Role |
|---|-----------|-----------|
| 1 | READ | Retrieve from graph (by ID, query, view, traverse) |
| 2 | WRITE | Mutate graph (create, update, delete, CAS) |
| 3 | TRANSFORM | Pure data reshaping (sandboxed expression, no I/O) |
| 4 | BRANCH | Conditional routing (forward-only, no cycles) |
| 5 | ITERATE | Bounded collection processing (mandatory maxIterations) |
| 6 | WAIT | Suspend until signal/timeout |
| 7 | GATE | Capability check / custom logic escape hatch |
| 8 | CALL | Execute another subgraph (capability attenuation, timeout) |
| 9 | RESPOND | Terminal: produce output |
| 10 | EMIT | Fire-and-forget notification |
| 11 | SANDBOX | WASM computation (no re-entrancy, fuel-metered) |
| 12 | VALIDATE | Schema + referential integrity check |

**Key design properties:**
- Subgraphs are DAGs (no cycles), with `transactional: true` making all WRITEs atomic (all-or-nothing)
- TRANSFORM expression language includes: arithmetic, string ops, `filter()`, `map()`, `some()`, `every()`, `find()`, `sum()`, `avg()`, `min()`, `max()`, `slugify()`, `now()`
- Context is a growing map -- each node's output accumulates, not overwrites
- WRITE supports conditional CAS via `expectedVersion`
- ITERATE has mandatory `maxIterations` and optional `parallel` flag

---

## Handler 1: Create Blog Post (CMS CRUD)

### Scenario

A logged-in user submits a new blog post with title, slug, content, and tags. The system validates input, checks capability, generates a slug, creates the post with version chain, creates HAS_TAG edges, emits a notification, and returns the created post.

### Complete Subgraph

```
Subgraph: "cms:create_blog_post"
transactional: true

[1. GATE check="store:create:content/post"]
  |
  |--ON_DENIED--> [2. RESPOND status=403 body={"error": "Insufficient permissions"}]
  |
  |--NEXT--> [3. VALIDATE schema="contentType:post" input="$.body"]
               |
               |--ON_ERROR--> [4. RESPOND status=422 body={"errors": "$.validationErrors"}]
               |
               |--NEXT--> [5. BRANCH condition="$.body.slug != null && $.body.slug != ''" mode="boolean"]
                            |
                            |--TRUE--> [6. TRANSFORM expr="{...$.body, slug: $.body.slug}"]
                            |            |
                            |            |--NEXT--> [*merge at 8*]
                            |
                            |--FALSE--> [7. TRANSFORM expr="{...$.body, slug: slugify($.body.title)}"]
                                         |
                                         |--NEXT--> [*merge at 8*]

[8. READ mode="query" target="MATCH (c:Content:Post {slug: $.prepared.slug}) RETURN c" options={limit: 1}]
  |
  |--NEXT--> [9. BRANCH condition="$.existing != null" mode="boolean"]
               |
               |--TRUE--> [10. TRANSFORM expr="{...$.prepared, slug: $.prepared.slug + '-' + randomSuffix(4)}"]
               |             |--NEXT--> [*merge at 11*]
               |
               |--FALSE--> [*merge at 11*]

[11. TRANSFORM expr="{...$.prepared, createdAt: now(), updatedAt: now(), status: 'draft', authorId: $.ctx.userId}"]
  |
  |--NEXT--> [12. WRITE action="create" labels=["Content","Post"]]
               |
               |--NEXT--> [13. BRANCH condition="$.body.tags != null && len($.body.tags) > 0" mode="boolean"]
                             |
                             |--TRUE--> [14. ITERATE source="$.body.tags" maxIterations=50 collectAs="tagEdges"]
                             |            |
                             |            |--BODY--> [15. CALL subgraph="cms:ensure_tag_and_link" 
                             |                              inputMap={tagName: "$.item", postId: "$.createdPost.id"}]
                             |            |
                             |            |--NEXT--> [*merge at 16*]
                             |
                             |--FALSE--> [*merge at 16*]

[16. EMIT event="content:afterCreate" data={contentType: "post", id: "$.createdPost.id", authorId: "$.ctx.userId"}]
  |
  |--NEXT--> [17. RESPOND status=201 body="$.createdPost"]
```

**Sub-subgraph: "cms:ensure_tag_and_link"**

```
transactional: false (parent is transactional)

[1. READ mode="query" target="MATCH (t:Tag {name: $.tagName}) RETURN t" options={limit: 1}]
  |
  |--NEXT--> [2. BRANCH condition="$.result == null" mode="boolean"]
               |
               |--TRUE--> [3. WRITE action="create" labels=["Tag"] data={name: "$.tagName"}]
               |             |--NEXT--> [*merge at 4*]
               |
               |--FALSE--> [*merge at 4*]

[4. WRITE action="createEdge" edgeType="HAS_TAG" edgeFrom="$.postId" edgeTo="$.tagId"]
  |
  |--NEXT--> [5. RESPOND channel="value" body="$.tagId"]
```

### Metrics

| Metric | Value |
|--------|-------|
| **Total Nodes (main subgraph)** | 17 |
| **Total Nodes (sub-subgraph, per tag)** | 5 |
| **SANDBOX count** | 0 |
| **SANDBOX percentage** | **0%** |

### Analysis

**What works well:**
- The entire handler is expressed with zero SANDBOX calls. All logic is orchestration: validate, branch, read, write, emit.
- `transactional: true` on the root means the slug dedup check + WRITE + tag edge creation are atomic. If any tag edge fails, the entire post creation rolls back. This directly addresses the completeness critic's concern about multi-node transactions.
- BRANCH for conditional slug generation is clean and readable.
- ITERATE for tag edge creation with CALL to a reusable sub-subgraph is the right decomposition.
- The expression evaluator handles `slugify()`, `randomSuffix()`, spread operators, and `now()` -- all pure, bounded operations.

**What's awkward:**
- The slug deduplication check is naive (one read + append suffix). A production system might need a retry loop. This would require ITERATE with a BRANCH exit condition, adding 3-4 more nodes. Not impossible, but the subgraph grows.
- Merge points (where TRUE/FALSE branches reconverge) are implicit in DAG structure. The notation gets verbose. This is a DX concern, not an expressiveness concern.

**What's impossible:**
- Nothing. Every step maps to a primitive without contortion.

---

## Handler 2: Multi-Step Checkout (Commerce)

### Scenario

A user checks out their cart. The system reads cart items, checks inventory for each item, calculates total (prices x quantities, tax, discount), charges payment (external provider), creates order Node, reduces inventory for each item, sends confirmation email, emits notification, and handles failure at any step with compensation (restore inventory, refund payment).

### Complete Subgraph

```
Subgraph: "commerce:checkout"
transactional: true

[1. GATE check="store:create:commerce/order"]
  |
  |--ON_DENIED--> [2. RESPOND status=403 body={"error": "Cannot create orders"}]
  |
  |--NEXT--> [3. READ mode="node" target="$.body.cartId"]
               |
               |--ON_NOT_FOUND--> [4. RESPOND status=404 body={"error": "Cart not found"}]
               |
               |--NEXT--> [5. VALIDATE schema="checkout:cart" input="$.cart"]
                            |
                            |--ON_ERROR--> [6. RESPOND status=422 body={"errors": "$.validationErrors"}]
                            |
                            |--NEXT--> [7. ITERATE source="$.cart.items" maxIterations=200 
                                               collectAs="inventoryChecks" parallel=true]
                                         |
                                         |--BODY--> [8. CALL subgraph="commerce:check_item_inventory"
                                                           inputMap={productId: "$.item.productId", 
                                                                     qty: "$.item.quantity"}]
                                         |
                                         |--NEXT--> [9. BRANCH condition="$.inventoryChecks.some(c => !c.available)" 
                                                              mode="boolean"]
                                                      |
                                                      |--TRUE--> [10. RESPOND status=409 
                                                      |                body={error: "Insufficient inventory",
                                                      |                      unavailable: "$.inventoryChecks.filter(c => !c.available)"}]
                                                      |
                                                      |--FALSE--> [*continue*]

# Calculate total
[11. TRANSFORM expr="{
       subtotal: sum($.cart.items.map(i => i.price * i.quantity)),
       tax: sum($.cart.items.map(i => i.price * i.quantity)) * $.taxRate,
       discount: 0
     }"]
  |
  |--NEXT--> [12. BRANCH condition="$.body.discountCode != null" mode="boolean"]
               |
               |--TRUE--> [13. CALL subgraph="commerce:apply_discount"
               |                  inputMap={code: "$.body.discountCode", subtotal: "$.totals.subtotal"}]
               |             |--NEXT--> [14. TRANSFORM expr="{...$.totals, discount: $.discountResult.amount,
               |                                              total: $.totals.subtotal + $.totals.tax - $.discountResult.amount}"]
               |             |--NEXT--> [*merge at 15*]
               |
               |--FALSE--> [15. TRANSFORM expr="{...$.totals, total: $.totals.subtotal + $.totals.tax}"]

# Charge payment (external -- SANDBOX required)
[16. SANDBOX runtime="commerce-payments" entryPoint="chargeCard"
            args={token: "$.body.paymentToken", amount: "$.totals.total", currency: "$.currency"}
            gasBudget=50000 timeout=30000]
  |
  |--ON_ERROR--> [17. RESPOND status=402 body={"error": "Payment failed", "details": "$.error"}]
  |
  |--NEXT--> [18. WRITE action="create" labels=["Order"] 
                        data={userId: "$.ctx.userId", cartId: "$.cart.id", items: "$.cart.items",
                              subtotal: "$.totals.subtotal", tax: "$.totals.tax", 
                              discount: "$.totals.discount", total: "$.totals.total",
                              paymentChargeId: "$.chargeResult.chargeId", status: "confirmed"}]
               |
               |--ON_ERROR--> [19. CALL subgraph="commerce:refund_payment"
               |                        inputMap={chargeId: "$.chargeResult.chargeId"}]
               |                |--NEXT--> [20. RESPOND status=500 body={"error": "Order creation failed, payment refunded"}]
               |
               |--NEXT--> [21. ITERATE source="$.cart.items" maxIterations=200]
                             |
                             |--BODY--> [22. WRITE action="update" target="$.item.inventoryNodeId"
                                                  data={quantity: "$.item.currentStock - $.item.quantity"}
                                                  expectedVersion="$.item.inventoryVersion"]
                             |
                             |--NEXT--> [23. EMIT event="commerce:orderCreated" 
                                                  data={orderId: "$.order.id", userId: "$.ctx.userId", total: "$.totals.total"}]
                                          |
                                          |--NEXT--> [24. CALL subgraph="communications:send_order_confirmation"
                                                               inputMap={orderId: "$.order.id", userId: "$.ctx.userId"}]
                                                       |
                                                       |--NEXT--> [25. RESPOND status=201 body="$.order"]
```

### Compensation Analysis

The `transactional: true` flag on the subgraph provides atomicity for all graph WRITEs (order creation, inventory decrements). If any WRITE fails (e.g., inventory WRITE #22 hits a VERSION_CONFLICT), ALL WRITEs in the subgraph are rolled back -- the order Node and all previously decremented inventory are undone.

**The exception:** The SANDBOX call to the payment provider (node 16) is EXTERNAL. It cannot be rolled back by the graph transaction. This is why node 19 exists as an explicit compensation path: if order creation fails AFTER payment succeeds, the system must explicitly refund.

The compensation structure:

```
Payment succeeds (SANDBOX) --> Order creation fails (WRITE) --> Explicit refund (CALL)
                           --> Order succeeds --> Inventory fails (WRITE) --> Transaction rolls back ALL WRITEs
                                                                             (order + inventory are rolled back)
                                                                             --> Explicit refund needed? YES.
```

**Critical insight:** Because `transactional: true` rolls back ALL WRITEs on ANY failure, the only compensation needed is for the external SANDBOX call. Graph-internal operations are atomic. This is far cleaner than the Temporal saga pattern where every step needs its own compensating action.

BUT there is a subtlety: if WRITE node 22 (inventory decrement) fails on item #5 of 10, the transaction rolls back nodes 18 (order) and 22 (inventories 1-4) -- but node 16 (payment) already succeeded externally. The ON_ERROR for the ITERATE body would need to propagate up to trigger the refund. This requires an error propagation path:

```
[22. WRITE ... inventoryDecrement]
  |--ON_ERROR--> [error bubbles up to ITERATE]
                   |--ON_ERROR--> [26. CALL subgraph="commerce:refund_payment"]
                                    |--NEXT--> [27. RESPOND status=500 body={"error": "Inventory conflict, payment refunded"}]
```

Adding these compensation nodes:

### Updated Metrics

| Metric | Value |
|--------|-------|
| **Total Nodes (main subgraph)** | 27 |
| **SANDBOX count** | 1 (payment charge) |
| **SANDBOX percentage** | **3.7%** |

### Analysis

**What works well:**
- `transactional: true` is the hero. It eliminates 90% of the compensation complexity. All graph-internal operations (order + inventory) are atomic without any explicit undo logic.
- The only SANDBOX call is the payment provider -- genuinely external I/O that cannot be expressed as graph operations.
- ITERATE with `parallel: true` for inventory checks uses the engine's parallelism.
- TRANSFORM for total calculation with `sum()` and `map()` is concise.
- CAS via `expectedVersion` on inventory decrements prevents overselling.

**What's awkward:**
- Compensation for the external payment requires explicit wiring (nodes 19-20 and 26-27). This is the fragility the review-vocab-systems doc identified (Finding 1). The compensation is expressible but manual -- forgetting to wire the refund path would be a bug.
- The error propagation from ITERATE body failure up to the compensation handler is not obvious from the DAG notation. The engine needs to define clearly: when a node inside an ITERATE body errors, does the error propagate to the ITERATE node's ON_ERROR edge?
- Discount code validation as a separate CALL subgraph adds 2-3 more nodes of indirection.

**What's impossible:**
- Nothing is impossible. The handler is fully expressed.
- The completeness critic's "multi-node transaction" concern is addressed by `transactional: true` on the subgraph. This is the key finding: making subgraph evaluation atomic is sufficient for multi-node transactions. No TRANSACTION primitive needed.

---

## Handler 3: Knowledge Attestation (Token Economics)

### Scenario

A user attests to a knowledge Node, paying a fee. The system reads the knowledge Node, checks the user has enough credits, checks for duplicate attestation, debits the user, distributes credits to existing attestors proportionally, creates the ATTESTED_BY edge, updates the attestation aggregate, and emits a notification.

### Complete Subgraph

```
Subgraph: "knowledge:attest"
transactional: true

[1. GATE check="knowledge:attest:*"]
  |
  |--ON_DENIED--> [2. RESPOND status=403 body={"error": "Cannot attest"}]
  |
  |--NEXT--> [3. READ mode="node" target="$.body.knowledgeNodeId"]
               |
               |--ON_NOT_FOUND--> [4. RESPOND status=404 body={"error": "Knowledge node not found"}]
               |
               |--NEXT--> [5. READ mode="node" target="$.ctx.userId" projection=["creditBalance"]]
                            |
                            |--NEXT--> [6. TRANSFORM expr="{
                                              attestCost: $.knowledgeNode.baseAttestCost * 
                                                          (1 + $.knowledgeNode.attestorCount * 0.1),
                                              userBalance: $.user.creditBalance
                                            }"]
                                         |
                                         |--NEXT--> [7. BRANCH condition="$.calc.userBalance < $.calc.attestCost" 
                                                              mode="boolean"]
                                                      |
                                                      |--TRUE--> [8. RESPOND status=402 
                                                      |                body={error: "Insufficient credits",
                                                      |                      required: "$.calc.attestCost",
                                                      |                      balance: "$.calc.userBalance"}]
                                                      |
                                                      |--FALSE--> [*continue*]

# Check for duplicate attestation
[9. READ mode="query" target="MATCH (u:User {id: $.ctx.userId})-[:ATTESTED_BY]->(k:Knowledge {id: $.body.knowledgeNodeId}) RETURN u"
          options={limit: 1}]
  |
  |--NEXT--> [10. BRANCH condition="$.existing != null" mode="boolean"]
               |
               |--TRUE--> [11. RESPOND status=409 body={"error": "Already attested"}]
               |
               |--FALSE--> [*continue*]

# Debit attestor
[12. WRITE action="update" target="$.ctx.userId"
           data={creditBalance: "$.user.creditBalance - $.calc.attestCost"}
           expectedVersion="$.user.version"]
  |
  |--ON_CONFLICT--> [13. RESPOND status=409 body={"error": "Balance changed, retry"}]
  |
  |--NEXT--> [14. READ mode="query" 
                        target="MATCH (a)-[e:ATTESTED_BY]->(k:Knowledge {id: $.body.knowledgeNodeId}) RETURN a, e"
                        options={sort: "e.timestamp asc"}]
               |
               |--NEXT--> [15. BRANCH condition="len($.existingAttestors) > 0" mode="boolean"]
                             |
                             |--TRUE--> [16. TRANSFORM expr="{
                             |                  totalWeight: sum($.existingAttestors.map(a => a.edge.cost)),
                             |                  distributions: $.existingAttestors.map(a => ({
                             |                    userId: a.node.id,
                             |                    share: (a.edge.cost / sum($.existingAttestors.map(x => x.edge.cost))) 
                             |                           * $.calc.attestCost * 0.8
                             |                  }))
                             |                }"]
                             |             |
                             |             |--NEXT--> [17. ITERATE source="$.distributions.distributions" 
                             |                                      maxIterations=10000 collectAs="credits"]
                             |                          |
                             |                          |--BODY--> [18. WRITE action="update" target="$.item.userId"
                             |                                                data={creditBalance: "$.item.currentBalance + $.item.share"}]
                             |                          |
                             |                          |--NEXT--> [*merge at 19*]
                             |
                             |--FALSE--> [*merge at 19*]

# Create attestation edge
[19. WRITE action="createEdge" edgeType="ATTESTED_BY" 
           edgeFrom="$.ctx.userId" edgeTo="$.body.knowledgeNodeId"
           data={cost: "$.calc.attestCost", timestamp: "now()", position: "$.knowledgeNode.attestorCount + 1"}]
  |
  |--NEXT--> [20. WRITE action="update" target="$.body.knowledgeNodeId"
                        data={attestorCount: "$.knowledgeNode.attestorCount + 1",
                              totalAttestationValue: "$.knowledgeNode.totalAttestationValue + $.calc.attestCost"}]
               |
               |--NEXT--> [21. EMIT event="knowledge:attested" 
                                     data={knowledgeNodeId: "$.body.knowledgeNodeId", 
                                           attestorId: "$.ctx.userId",
                                           cost: "$.calc.attestCost",
                                           attestorCount: "$.knowledgeNode.attestorCount + 1"}]
                             |
                             |--NEXT--> [22. RESPOND status=201 body={
                                                attestation: {knowledgeNodeId: "$.body.knowledgeNodeId",
                                                              cost: "$.calc.attestCost",
                                                              position: "$.knowledgeNode.attestorCount + 1"}}]
```

### The Multi-Credit-Transfer Atomicity Test

The completeness critic specifically called this out: "if there are 100 existing attestors, the fee distribution requires 100 WRITE operations. If WRITE #73 fails, the first 72 attestors have been credited but #73-#100 have not."

**Answer:** `transactional: true` on the subgraph. ALL WRITEs -- the user debit (node 12), all 100 attestor credits (node 18 iterated), the attestation edge (node 19), and the aggregate update (node 20) -- are in a single atomic transaction. If WRITE #73 fails, ALL writes roll back. The user's credits are restored. No attestor gets partial credit. No attestation edge is created.

This is the strongest possible answer to the critic. The atomicity is not "eventual via compensation" -- it is true database-level atomicity because `transactional: true` means the engine collects all WRITEs and commits them as a single atomic unit.

### Metrics

| Metric | Value |
|--------|-------|
| **Total Nodes** | 22 |
| **SANDBOX count** | 0 |
| **SANDBOX percentage** | **0%** |

### Analysis

**What works well:**
- Zero SANDBOX calls. All logic -- credit math, proportional distribution, edge creation -- is expressed with TRANSFORM, ITERATE, and WRITE.
- The TRANSFORM expression language handles the proportional distribution calculation inline: `a.edge.cost / sum(...) * attestCost * 0.8`. This is arithmetic + array operations, all within the expression evaluator's specified capabilities.
- `transactional: true` makes the multi-party credit transfer atomic. This is the definitive answer to the critic's concern.
- CAS via `expectedVersion` on the user's credit balance (node 12) prevents double-debit race conditions.
- The escalating cost formula (`baseAttestCost * (1 + attestorCount * 0.1)`) is pure TRANSFORM.

**What's awkward:**
- Node 18 (credit each attestor) does a WRITE inside an ITERATE. For a knowledge Node with 10,000 attestors, this is 10,000 WRITEs in a single transaction. The question is whether the engine can handle a transaction with 10,000+ atomic writes. This is a performance/scaling concern, not an expressiveness concern.
- The proportional distribution TRANSFORM (node 16) is dense. The nested `map()` and `sum()` calls, while within the expression evaluator's capabilities, push the boundary of what is "readable" in a graph node's expression property. A CALL to a dedicated distribution subgraph might be cleaner.
- The current attestor READ (node 14) returns ALL attestors. For a popular knowledge Node with 100,000 attestors, this could be a large result set. Pagination would help, but paginated credit distribution across multiple ITERATE passes would break the atomicity guarantee (if the second page fails, the first page's credits are already committed). The answer is: keep it in one transaction, accept the memory cost, and set a practical attestor limit.

**What's impossible:**
- Nothing is impossible. But there is a practical scaling ceiling: a single transaction with N writes where N = attestor count. The engine needs to handle large transactions gracefully, or the system needs to cap attestor count (which the governance system could configure).

---

## Handler 4: Governance Vote (Grove)

### Scenario

A member votes on a proposal in a Grove. The system reads the proposal, verifies it is still open, checks voter membership, checks for duplicate votes, determines vote weight based on the community's voting mechanism (1p1v, quadratic, or token-weighted), creates the Vote Node (signed), updates the vote tally, checks if the threshold is reached, and if so, executes the proposal (which is itself an operation subgraph).

### Complete Subgraph

```
Subgraph: "governance:cast_vote"
transactional: true

[1. GATE check="governance:vote:${$.body.groveId}"]
  |
  |--ON_DENIED--> [2. RESPOND status=403 body={"error": "Cannot vote in this Grove"}]
  |
  |--NEXT--> [3. READ mode="node" target="$.body.proposalId"]
               |
               |--ON_NOT_FOUND--> [4. RESPOND status=404 body={"error": "Proposal not found"}]
               |
               |--NEXT--> [5. BRANCH condition="$.proposal.status != 'open'" mode="boolean"]
                             |
                             |--TRUE--> [6. BRANCH condition="$.proposal.status == 'expired'" mode="boolean"]
                             |            |--TRUE--> [7. RESPOND status=410 body={"error": "Proposal expired"}]
                             |            |--FALSE--> [8. RESPOND status=409 body={"error": "Proposal already resolved"}]
                             |
                             |--FALSE--> [9. BRANCH condition="now() > $.proposal.deadline" mode="boolean"]
                                          |
                                          |--TRUE--> [10. WRITE action="update" target="$.body.proposalId" 
                                          |                      data={status: "expired"}]
                                          |            |--NEXT--> [11. RESPOND status=410 body={"error": "Proposal just expired"}]
                                          |
                                          |--FALSE--> [*continue*]

# Verify membership
[12. READ mode="query" 
           target="MATCH (u:User {id: $.ctx.userId})-[:MEMBER_OF]->(g:Grove {id: $.body.groveId}) RETURN u"
           options={limit: 1}]
  |
  |--NEXT--> [13. BRANCH condition="$.membership == null" mode="boolean"]
               |
               |--TRUE--> [14. RESPOND status=403 body={"error": "Not a member of this Grove"}]
               |
               |--FALSE--> [*continue*]

# Check duplicate vote
[15. READ mode="query"
           target="MATCH (v:Vote {proposalId: $.body.proposalId, voterId: $.ctx.userId}) RETURN v"
           options={limit: 1}]
  |
  |--NEXT--> [16. BRANCH condition="$.existingVote != null" mode="boolean"]
               |
               |--TRUE--> [17. RESPOND status=409 body={"error": "Already voted on this proposal"}]
               |
               |--FALSE--> [*continue*]

# Determine vote weight based on voting mechanism
[18. READ mode="query" 
           target="MATCH (g:Grove {id: $.body.groveId})-[:GOVERNED_BY]->(gc:GovernanceConfig) RETURN gc"]
  |
  |--NEXT--> [19. CALL subgraph="governance:calculate_vote_weight"
                        inputMap={mechanism: "$.govConfig.votingMechanism",
                                  voterId: "$.ctx.userId",
                                  groveId: "$.body.groveId"}]
               |
               |--NEXT--> [*continue*]

# Create Vote Node
[20. WRITE action="create" labels=["Vote"]
           data={proposalId: "$.body.proposalId",
                 voterId: "$.ctx.userId",
                 groveId: "$.body.groveId",
                 choice: "$.body.choice",
                 weight: "$.voteWeight",
                 timestamp: "now()"}]
  |
  |--NEXT--> [21. WRITE action="createEdge" edgeType="VOTED_ON"
                        edgeFrom="$.vote.id" edgeTo="$.body.proposalId"]
               |
               |--NEXT--> [22. CALL subgraph="governance:update_tally"
                                     inputMap={proposalId: "$.body.proposalId",
                                               choice: "$.body.choice",
                                               weight: "$.voteWeight"}]
                             |
                             |--NEXT--> [23. BRANCH condition="$.tallyResult.thresholdReached" mode="boolean"]
                                          |
                                          |--TRUE--> [24. CALL subgraph="governance:execute_proposal"
                                          |                     inputMap={proposalId: "$.body.proposalId",
                                          |                               outcome: "$.tallyResult.outcome"}]
                                          |            |
                                          |            |--NEXT--> [25. EMIT event="governance:proposalResolved"
                                          |                                   data={proposalId: "$.body.proposalId",
                                          |                                         outcome: "$.tallyResult.outcome"}]
                                          |            |--NEXT--> [26. RESPOND status=200 
                                          |                              body={voted: true, proposalResolved: true,
                                          |                                    outcome: "$.tallyResult.outcome"}]
                                          |
                                          |--FALSE--> [27. EMIT event="governance:voteCast"
                                                                data={proposalId: "$.body.proposalId",
                                                                      voterId: "$.ctx.userId"}]
                                                        |
                                                        |--NEXT--> [28. RESPOND status=200 
                                                                          body={voted: true, proposalResolved: false,
                                                                                currentTally: "$.tallyResult.tally"}]
```

**Sub-subgraph: "governance:calculate_vote_weight"**

```
[1. BRANCH condition="$.mechanism" mode="match"]
  |
  |--MATCH:"one_person_one_vote"--> [2. TRANSFORM expr="{weight: 1}"]
  |                                   |--NEXT--> [6. RESPOND channel="value" body="$.weight"]
  |
  |--MATCH:"quadratic"--> [3. READ mode="query"
  |                              target="MATCH (u:User {id: $.voterId})-[:HOLDS]->(t:Token {groveId: $.groveId}) RETURN sum(t.amount) as tokens"]
  |                          |--NEXT--> [4. TRANSFORM expr="{weight: sqrt($.tokens)}"]
  |                          |--NEXT--> [6. RESPOND ...]
  |
  |--MATCH:"token_weighted"--> [3b. READ mode="query" ... (same token query)]
  |                              |--NEXT--> [5. TRANSFORM expr="{weight: $.tokens}"]
  |                              |--NEXT--> [6. RESPOND ...]
  |
  |--DEFAULT--> [6b. RESPOND channel="error" body={"error": "Unknown voting mechanism"}]
```

**Sub-subgraph: "governance:update_tally"**

```
transactional: false (parent is transactional)

[1. READ mode="query" 
         target="MATCH (v:Vote {proposalId: $.proposalId}) RETURN v.choice, sum(v.weight) as totalWeight GROUP BY v.choice"]
  |
  |--NEXT--> [2. TRANSFORM expr="{
                    tally: $.results,
                    totalVotesCast: sum($.results.map(r => r.totalWeight))
                  }"]
               |
               |--NEXT--> [3. READ mode="node" target="$.proposalId" projection=["threshold", "thresholdType"]]
                             |
                             |--NEXT--> [4. BRANCH condition="$.proposal.thresholdType" mode="match"]
                                          |
                                          |--MATCH:"majority"--> 
                                            [5. TRANSFORM expr="{
                                                  thresholdReached: $.tally.some(r => r.totalWeight > $.calc.totalVotesCast / 2),
                                                  outcome: $.tally.find(r => r.totalWeight > $.calc.totalVotesCast / 2)?.choice ?? null
                                                }"]
                                          |
                                          |--MATCH:"supermajority"-->
                                            [6. TRANSFORM expr="{
                                                  thresholdReached: $.tally.some(r => r.totalWeight > $.calc.totalVotesCast * 0.667),
                                                  outcome: $.tally.find(r => r.totalWeight > $.calc.totalVotesCast * 0.667)?.choice ?? null
                                                }"]
                                          |
                                          |--MATCH:"absolute"-->
                                            [7. TRANSFORM expr="{
                                                  thresholdReached: $.tally.some(r => r.totalWeight >= $.proposal.threshold),
                                                  outcome: $.tally.find(r => r.totalWeight >= $.proposal.threshold)?.choice ?? null
                                                }"]
                                          |
                                          |--NEXT--> [8. RESPOND channel="value" body="$.tallyResult"]
```

### Does Handler 4 Need the Full Governance Resolution?

**Yes, partially.** The vote weight calculation (sub-subgraph) needs to know the community's voting mechanism, which is a governance configuration. The threshold check (update_tally sub-subgraph) needs to know the threshold type and value. Both are READ operations on governance Nodes -- not full governance resolution.

The FULL governance resolution (fractal inheritance, polycentric override modes) would only be needed if the governance CONFIG ITSELF needs resolving through parent Groves. For a single Grove with its own config, a simple READ suffices. For inherited governance, `governance:calculate_vote_weight` would need to CALL `governance:resolve_effective_config` which walks the parent chain -- but this is exactly what IVM materializes. The "effective governance config" is a materialized view, so it is a single READ, not a traversal.

**Answer: No, the handler does NOT need full governance resolution at vote time, because IVM pre-materializes the effective config.**

### Metrics

| Metric | Value |
|--------|-------|
| **Total Nodes (main subgraph)** | 28 |
| **Total Nodes (calculate_vote_weight)** | 6-7 (depends on mechanism branch) |
| **Total Nodes (update_tally)** | 8 |
| **SANDBOX count** | 0 |
| **SANDBOX percentage** | **0%** |

### Analysis

**What works well:**
- Zero SANDBOX calls. Voting mechanisms (1p1v, quadratic, token-weighted) are all expressible as TRANSFORM with arithmetic: `sqrt()` for quadratic, direct sum for token-weighted.
- BRANCH mode="match" handles the multi-way mechanism dispatch cleanly.
- `transactional: true` ensures the vote + tally update + potential proposal execution are atomic. If the proposal execution fails, the vote is not recorded -- which is the correct semantic.
- The IVM-materialized governance config eliminates the need for recursive governance resolution at vote time. This is a strong validation of the IVM design.

**What's awkward:**
- The threshold check in update_tally is verbose -- three branches for three threshold types, each with a similar TRANSFORM. A more expressive TRANSFORM (with functions) could consolidate this, but the current expression evaluator handles it.
- The `governance:execute_proposal` subgraph (node 24) is opaque in this prototype. If the proposal is "revoke Alice's moderator access," the execution involves WRITE to remove a capability edge. If it is "change the voting mechanism," it involves WRITE to update the governance config. The proposal's execution subgraph would be stored as a separate operation subgraph in the graph -- a meta-capability. This is a feature of code-as-graph: proposals can reference their own execution logic.
- `GROUP BY` in the READ query (node 1 of update_tally) assumes the query language supports aggregation. If not, the grouping must be done in TRANSFORM after a flat READ, which would require a `groupBy()` expression function.

**What's impossible:**
- Nothing impossible. The most complex aspect (vote weight with quadratic formula) is pure arithmetic.
- Liquid delegation (delegating your vote to another member) would add a traversal step: READ the delegation chain, accumulate weights. This is expressible as a CALL to a delegation-resolver subgraph, or better yet, as an IVM materialized view of effective vote weights.

---

## Handler 5: AI Agent Content Generation

### Scenario

An AI agent creates a content page on behalf of a user. The system receives the generation request, checks the agent's capabilities, checks the delegating user's capabilities (agent cannot exceed user permissions), calls SANDBOX to generate content via LLM API, validates the generated content against the content type schema, checks for prohibited content (moderation), creates the content Node attributed to both agent and user, emits notification, and returns the content for user review (draft status).

### Complete Subgraph

```
Subgraph: "ai:generate_content"
transactional: true

# Check agent's own capability to create content
[1. GATE check="content:create:${$.body.contentType}" 
         context={agentId: "$.ctx.agentId"}]
  |
  |--ON_DENIED--> [2. RESPOND status=403 body={"error": "Agent lacks content creation capability"}]
  |
  |--NEXT--> [*continue*]

# Check delegating user's capability (agent cannot exceed user's permissions)
[3. READ mode="query"
         target="MATCH (a:Agent {id: $.ctx.agentId})-[:DELEGATED_BY]->(u:User) RETURN u"
         options={limit: 1}]
  |
  |--ON_NOT_FOUND--> [4. RESPOND status=403 body={"error": "Agent has no delegating user"}]
  |
  |--NEXT--> [5. GATE check="content:create:${$.body.contentType}" 
                       context={userId: "$.delegatingUser.id"}]
               |
               |--ON_DENIED--> [6. RESPOND status=403 
                                     body={"error": "Delegating user lacks content creation capability",
                                           "agentId": "$.ctx.agentId",
                                           "userId": "$.delegatingUser.id"}]
               |
               |--NEXT--> [*continue*]

# Validate the generation request
[7. VALIDATE schema="ai:generationRequest" input="$.body"]
  |
  |--ON_ERROR--> [8. RESPOND status=422 body={"errors": "$.validationErrors"}]
  |
  |--NEXT--> [*continue*]

# Read the content type schema for the LLM prompt
[9. READ mode="query" 
         target="MATCH (ct:ContentType {id: $.body.contentType}) RETURN ct"]
  |
  |--NEXT--> [10. TRANSFORM expr="{
                     prompt: 'Generate content for type: ' + $.contentTypeSchema.label +
                             '. Fields: ' + join(keys($.contentTypeSchema.fields), ', ') +
                             '. Topic: ' + $.body.topic +
                             '. Style: ' + ($.body.style ?? 'professional') +
                             '. Constraints: ' + ($.body.constraints ?? 'none') +
                             '. Return valid JSON matching the field schema.',
                     fieldDefs: $.contentTypeSchema.fields
                   }"]
               |
               |--NEXT--> [*continue*]

# Call LLM via SANDBOX (the actual generation)
[11. SANDBOX runtime="ai-llm-provider" entryPoint="generateContent"
             args={prompt: "$.llmInput.prompt", 
                   fieldDefs: "$.llmInput.fieldDefs",
                   model: "$.body.model ?? 'default'",
                   temperature: "$.body.temperature ?? 0.7"}
             gasBudget=500000 timeout=60000 maxOutput=1048576]
  |
  |--ON_ERROR--> [12. RESPOND status=502 body={"error": "Content generation failed", "details": "$.error"}]
  |
  |--NEXT--> [*continue*]

# Validate generated content against content type schema
[13. VALIDATE schema="contentType:${$.body.contentType}" input="$.generatedContent"]
  |
  |--ON_ERROR--> [14. RESPOND status=422 
                        body={"error": "Generated content failed validation",
                              "validationErrors": "$.validationErrors",
                              "generatedContent": "$.generatedContent"}]
  |
  |--NEXT--> [*continue*]

# Content moderation check
[15. SANDBOX runtime="ai-moderation" entryPoint="checkContent"
             args={content: "$.generatedContent", 
                   policy: "$.body.moderationPolicy ?? 'default'"}
             gasBudget=100000 timeout=15000]
  |
  |--NEXT--> [16. BRANCH condition="$.moderationResult.flagged" mode="boolean"]
               |
               |--TRUE--> [17. RESPOND status=451 
               |                 body={"error": "Content flagged by moderation",
               |                       "flags": "$.moderationResult.flags",
               |                       "generatedContent": "$.generatedContent"}]
               |
               |--FALSE--> [*continue*]

# Create the content Node -- dual attribution
[18. TRANSFORM expr="{
         ...$.generatedContent,
         status: 'draft',
         createdAt: now(),
         updatedAt: now(),
         createdByAgent: $.ctx.agentId,
         createdByUser: $.delegatingUser.id,
         generationMeta: {
           model: $.body.model ?? 'default',
           topic: $.body.topic,
           style: $.body.style ?? 'professional',
           timestamp: now()
         }
       }"]
  |
  |--NEXT--> [19. WRITE action="create" labels=["Content", $.body.contentType]
                        data="$.preparedContent"]
               |
               |--NEXT--> [20. WRITE action="createEdge" edgeType="CREATED_BY_AGENT"
                                     edgeFrom="$.content.id" edgeTo="$.ctx.agentId"
                                     data={delegatingUser: "$.delegatingUser.id",
                                           generationMeta: "$.preparedContent.generationMeta"}]
                             |
                             |--NEXT--> [21. WRITE action="createEdge" edgeType="CREATED_BY_USER"
                                                   edgeFrom="$.content.id" edgeTo="$.delegatingUser.id"
                                                   data={viaAgent: "$.ctx.agentId"}]
                                          |
                                          |--NEXT--> [22. EMIT event="content:afterCreate" 
                                                               data={contentType: "$.body.contentType",
                                                                     id: "$.content.id",
                                                                     agentId: "$.ctx.agentId",
                                                                     userId: "$.delegatingUser.id",
                                                                     status: "draft"}]
                                                       |
                                                       |--NEXT--> [23. EMIT event="ai:contentGenerated"
                                                                             data={contentId: "$.content.id",
                                                                                   agentId: "$.ctx.agentId",
                                                                                   userId: "$.delegatingUser.id"}]
                                                                    |
                                                                    |--NEXT--> [24. RESPOND status=201 
                                                                                      body={content: "$.content",
                                                                                            status: "draft",
                                                                                            reviewRequired: true}]
```

### How Many Nodes for Dual Attribution?

**Answer: 3 nodes.** One WRITE to create the content Node (node 19), one WRITE to create the CREATED_BY_AGENT edge (node 20), and one WRITE to create the CREATED_BY_USER edge (node 21). The content Node's properties also include `createdByAgent` and `createdByUser` as denormalized fields for quick access without traversal.

This is clean. The graph's native edge model handles dual attribution naturally. In a relational system, you would need a junction table or two foreign keys; in the graph, it is two edges.

### Metrics

| Metric | Value |
|--------|-------|
| **Total Nodes** | 24 |
| **SANDBOX count** | 2 (LLM generation + moderation) |
| **SANDBOX percentage** | **8.3%** |

### Analysis

**What works well:**
- The dual-capability check (agent's own + delegating user's) is two GATE nodes. Simple, declarative, and correct. The agent cannot exceed the user's permissions because both GATEs must pass.
- Dual attribution is 3 WRITE nodes -- the graph model makes this trivial. No junction tables, no awkward schema gymnastics.
- The LLM call is correctly a SANDBOX -- it is genuinely external I/O that cannot be expressed as graph operations.
- Content moderation is also correctly a SANDBOX -- it calls external AI models.
- VALIDATE catches malformed LLM output before it enters the graph. This is a safety gate that prevents bad AI-generated content from polluting the data.
- `transactional: true` ensures that if moderation catches something AFTER VALIDATE passes (race condition), or if any WRITE fails, everything rolls back.
- Draft status is enforced in the TRANSFORM (node 18) -- the agent cannot publish directly, only create drafts. Publication requires a separate handler with user approval.

**What's awkward:**
- The LLM prompt construction in TRANSFORM (node 10) is string concatenation. For a production system, the prompt would be much more complex (system instructions, few-shot examples, output format specification). This would push the TRANSFORM expression to its limit. A CALL to a prompt-builder subgraph, or even a SANDBOX call to a prompt template engine, would be more maintainable.
- Two consecutive SANDBOX calls (generation + moderation) mean two WASM sandbox spin-ups. The performance cost of WASM context creation matters here. If the engine pools WASM instances, this is mitigated.
- The error response for moderation (node 17, status 451) returns the generated content to the caller so they can see what was flagged. This may be a security concern -- the flagged content is being transmitted. A production system might redact the content and only return the flag categories.

**What's impossible:**
- Nothing is impossible. The two SANDBOX calls are genuinely necessary (LLM + moderation are external services). Everything else is pure graph operations.

---

## Cross-Handler Summary

### Overall Metrics

| Handler | Total Nodes | SANDBOX Count | SANDBOX % | Sub-subgraphs |
|---------|-------------|---------------|-----------|----------------|
| 1. Create Blog Post | 17 | 0 | **0%** | 1 (tag linking) |
| 2. Multi-Step Checkout | 27 | 1 | **3.7%** | 2 (check_inventory, refund_payment) |
| 3. Knowledge Attestation | 22 | 0 | **0%** | 0 |
| 4. Governance Vote | 28 | 0 | **0%** | 2 (vote_weight, update_tally) |
| 5. AI Agent Content Gen | 24 | 2 | **8.3%** | 0 |
| **TOTAL** | **118** | **3** | **2.5%** | **5** |

### The Key Metric: SANDBOX Percentage

**2.5% of all nodes are SANDBOX calls.** This is dramatically below the 30% threshold the feasibility critic set, and far below the 50% threshold that would indicate "the primitives are insufficient."

The 3 SANDBOX calls across all 5 handlers are:
1. Payment provider charge (external HTTP API)
2. LLM content generation (external AI model)
3. Content moderation check (external AI model)

Every single SANDBOX call is for genuinely external I/O -- calling services outside the graph. None of them are "I couldn't express this logic in the primitives, so I escaped to WASM." This is exactly the design intent: SANDBOX is the I/O boundary, not a logic crutch.

### What the Primitives Handle Well

1. **CRUD with validation:** VALIDATE + WRITE + RESPOND is the bread and butter. Clean, linear, no awkwardness.

2. **Conditional logic:** BRANCH handles single conditions. BRANCH mode="match" handles multi-way dispatch (voting mechanisms, threshold types). Forward-only constraint is never a limitation for these handlers.

3. **Bounded iteration:** ITERATE with maxIterations handles tag creation (50), inventory checks (200), attestor credit distribution (10,000), and vote tallying. The `parallel` flag accelerates independent operations.

4. **Multi-node transactions:** `transactional: true` on the subgraph makes ALL WRITEs atomic. This is the single most important design decision for handler expressiveness. It eliminates the need for explicit compensation for graph-internal operations. The completeness critic's concern about multi-node transaction atomicity is fully resolved.

5. **Capability enforcement:** GATE is clean and declarative. The dual-capability check for AI agents (agent + delegating user) is two GATE nodes, not a complex composition.

6. **Expression evaluation:** TRANSFORM handles slug generation, tax calculation, proportional distribution, quadratic vote weight, and prompt construction -- all without SANDBOX. The expression evaluator's built-in functions (sum, map, filter, sqrt, slugify, now, join, keys) are sufficient.

### What Is Awkward

1. **Compensation for external calls.** When a SANDBOX call succeeds (payment charge) but a subsequent graph WRITE fails, the transaction rolls back the WRITEs but not the external call. Manual compensation wiring is needed. This is inherent -- external calls are not rollback-able -- but the wiring is verbose and error-prone. **Recommendation:** Provide a compensation pattern template or a `compensate` property on SANDBOX nodes that names the undo subgraph.

2. **Merge points in DAGs.** When TRUE/FALSE branches reconverge, the DAG notation becomes ambiguous. The engine needs a defined semantics for convergence: does the merge point receive data from whichever branch executed? What if both branches write to the same context key? **Recommendation:** Define that branch merge points receive the context as modified by the executed branch, and the unexecuted branch's context modifications do not apply.

3. **Large transactions.** Handler 3 (attestation) can produce 10,000+ WRITEs in a single transaction. Handler 2 (checkout) with 200 items produces ~200 WRITEs. The engine must handle large atomic transactions without excessive memory pressure. **Recommendation:** Document transaction size limits as a capability-configurable property.

4. **Expression density.** Some TRANSFORM nodes have dense expressions (proportional distribution, threshold calculations). While technically within the expression evaluator's capability, they push readability limits. **Recommendation:** Consider a TRANSFORM variant that references a named expression template stored as a graph Node, or simply encourage CALL decomposition for complex expressions.

### What Is Impossible (Nothing Critical)

1. **Real-time collaborative editing.** None of these handlers need it, but the completeness critic flagged it. CRDT text types are needed at the engine level, not the primitive level. WRITE with LWW on text fields silently discards concurrent edits. This is an engine feature gap, not a primitive gap.

2. **Ephemeral state.** None of these handlers need "Alice is currently editing" presence indicators. If they did, WRITE would pollute the version history. An ephemeral state mechanism (writes that do not create version Nodes) would be needed at the engine level.

3. **Streaming responses.** All handlers return a single RESPOND. A streaming endpoint (e.g., streaming LLM tokens to the client) would need the reactive subscription layer below the primitive level.

These are all engine-level capabilities, not primitive gaps. The 12 primitives correctly delegate these concerns to the engine layer.

### Specific Questions Answered

**Can Handler 3 (attestation) do the multi-credit-transfer atomically?**
YES. `transactional: true` makes all WRITEs (user debit + N attestor credits + edge creation + aggregate update) atomic. If any WRITE fails, all roll back. No compensation needed for graph-internal operations. This definitively resolves the completeness critic's concern.

**Can Handler 2 (checkout) do compensation correctly without a COMPENSATE primitive?**
YES, with a caveat. Graph-internal operations use `transactional: true` (no compensation needed -- they just roll back). External SANDBOX calls (payment) require explicit compensation wiring via ON_ERROR --> CALL(refund). This works but is manual and fragile. A `compensate` property on SANDBOX (naming the undo subgraph) would reduce the error surface without adding a new primitive.

**Does Handler 4 (voting) need the full governance resolution for determining vote weight?**
NO. The governance config is an IVM materialized view. Vote weight calculation is a simple READ + TRANSFORM (arithmetic). Full governance resolution (parent Grove inheritance) happens at IVM maintenance time (when governance changes), not at vote time. Voting is O(1) for config lookup.

**How many Nodes does Handler 5 (AI agent) need for dual-attribution?**
3 Nodes: 1 WRITE (content Node with denormalized agent/user IDs) + 2 WRITE (CREATED_BY_AGENT edge + CREATED_BY_USER edge). The graph's native edge model makes dual attribution trivial.

---

## Verdict

**The 12 primitives pass the paper-prototype test.** Five real handlers spanning CMS CRUD, multi-step commerce, token economics, governance, and AI agents are fully expressed with a SANDBOX percentage of 2.5%. Every SANDBOX call is for genuinely external I/O, not for logic that the primitives cannot express.

The key enabler is `transactional: true` on subgraphs. Without it, every multi-WRITE handler would need explicit compensation, and the SANDBOX percentage would look the same but the overall complexity would be dramatically higher. With it, the primitives are clean, expressive, and safe.

**No 13th primitive is needed.** The closest candidate would be a COMPENSATE primitive, but the review-vocab-systems analysis correctly identifies this as a pattern (composition), not a primitive. A `compensate` property on SANDBOX nodes would cover the remaining awkwardness.

### Recommendations

1. **Ship `transactional: true` as a first-class subgraph property.** It is load-bearing for every handler that does more than one WRITE. Document it prominently.

2. **Add a `compensate` property to SANDBOX nodes** that names the undo subgraph to call if a subsequent operation fails. Not a new primitive -- a property on an existing one. This prevents the "forgot to wire the refund" class of bugs.

3. **Define merge-point semantics for DAG branches.** When branches reconverge, the engine must specify how context is merged. This is an execution model question, not a vocabulary question, but it affects every handler with BRANCH.

4. **Document transaction size limits.** Handler 3 can produce 10,000+ WRITEs per transaction. The engine needs configurable per-capability transaction size limits to prevent resource exhaustion.

5. **Ensure the expression evaluator includes:** `sqrt()`, `keys()`, `join()`, `slugify()`, `randomSuffix()`, `groupBy()` in addition to the already-specified `filter()`, `map()`, `some()`, `every()`, `find()`, `sum()`, `avg()`, `min()`, `max()`. These were all needed in the prototypes.

6. **Define error propagation from ITERATE bodies.** When a WRITE inside an ITERATE body fails and the subgraph is transactional, does the entire transaction roll back? (It should.) Does the ITERATE node's ON_ERROR edge fire? (It should.) This needs to be explicit in the spec.

---

## Sources

- [Temporal: Saga Compensating Transactions](https://temporal.io/blog/compensating-actions-part-of-sagas)
- [Saga Pattern in Microservices (Temporal)](https://temporal.io/blog/mastering-saga-patterns-for-distributed-transactions-in-microservices)
- [Solving Distributed Transactions with Saga Pattern and Temporal](https://medium.com/skyro-tech/solving-distributed-transactions-with-the-saga-pattern-and-temporal-27ccba602833)
- [LangGraph AI Framework 2025: Multi-Agent Orchestration](https://latenode.com/blog/ai-frameworks-technical-infrastructure/langgraph-multi-agent-orchestration/langgraph-ai-framework-2025-complete-architecture-guide-multi-agent-orchestration-analysis)
- [NVIDIA Graph Execution Engine](https://docs.nvidia.com/holoscan/sdk-user-guide/gxf/doc/composer/graphcomposer_graph_runtime.html)
- [Process Orchestration Models: Enterprise Agent Workflows Beyond MCP](https://medium.com/@raktims2210/process-orchestration-models-how-enterprises-build-large-scale-agent-workflows-beyond-mcp-6aa6b24a81d3)
