# Operation Vocabulary: Stress Test -- Can 12 Primitives Cover All System Needs?

**Author:** Engine Philosophy Guardian
**Date:** 2026-04-11
**Status:** Adversarial review of the synthesized 12-primitive vocabulary
**Input:** The 12 primitives from the systems-perspective document, plus scenarios from CMS, commerce, sync, AI agents, games, workflows, and module installation

---

## The 12 Under Review

| # | Primitive | Summary |
|---|-----------|---------|
| 1 | READ | Retrieve from graph (by ID, query, view, traverse) |
| 2 | WRITE | Mutate graph (create, update, delete, CAS) |
| 3 | TRANSFORM | Pure data reshaping (sandboxed, no I/O) |
| 4 | BRANCH | Conditional routing (forward-only) |
| 5 | ITERATE | Bounded collection processing |
| 6 | WAIT | Suspend until signal/timeout |
| 7 | GATE | Capability check + validation |
| 8 | CALL | Execute another subgraph (with capability attenuation) |
| 9 | RESPOND | Terminal: produce output |
| 10 | EMIT | Fire-and-forget notification |
| 11 | INVOKE | WASM sandbox call (no re-entrancy, fuel-metered) |
| 12 | VALIDATE | Schema + referential integrity check |

Composed (not primitives):
- Retry = ITERATE + BRANCH + CALL
- Parallel = ITERATE with `parallel` flag
- Compensate = BRANCH (on error) + CALL (undo subgraph)
- DryRun = evaluation mode property
- Audit = EMIT to audit channel
- Map/Filter/Reduce = ITERATE + TRANSFORM

---

## Scenario 1: CMS -- Blog Post Creation with Validation, Capability, Slug, Notification

**User story:** Author creates a blog post. System checks permission, validates fields, generates slug, saves, emits event, sends notification.

### Subgraph

```
[GATE mode="capability" check="store:create:content/post"]
  --NEXT--> [VALIDATE schema="contentType:post"]
  --NEXT--> [TRANSFORM expression="{...$.value, slug: slugify($.value.title), createdAt: now(), updatedAt: now()}"]
  --NEXT--> [WRITE action="create" labels=["Content","Post"]]
  --NEXT--> [EMIT event="content:afterCreate"]
  --NEXT--> [CALL subgraph="notifications:notify_followers" inputMap={"postId": "$.id"}]
  --NEXT--> [RESPOND status=201]
```

### Verdict: WORKS CLEANLY

No awkwardness. This is the bread-and-butter case. 7 nodes, linear flow.

### Issue found: `slugify()` in TRANSFORM

TRANSFORM uses sandboxed expressions. Is `slugify()` a built-in expression function? The systems doc says "JSONPath-like, arithmetic, string ops." Slug generation involves lowercasing, replacing spaces with hyphens, removing special characters, deduplication (checking existing slugs in the graph). The pure string-manipulation part is fine in TRANSFORM, but deduplication requires a READ.

**Resolution:** Split into two steps: TRANSFORM for the naive slug, then a CALL to a `generate_unique_slug` subgraph that does READ + BRANCH + TRANSFORM (append counter if duplicate exists). This works but reveals that string utility functions either need to be built into the expression evaluator or delegated to INVOKE/CALL. The 12 primitives handle this; the expression evaluator's built-in function set is the open question.

---

## Scenario 2: Commerce -- Multi-Step Checkout

**User story:** Customer checks out. System validates cart, checks inventory for each item, charges payment, creates order, decrements inventory, sends confirmation email. If payment fails, undo inventory reservation. If order creation fails, refund payment.

### Subgraph

```
# Entry: checkout_handler
[GATE mode="capability" check="store:create:commerce/order"]
  --NEXT--> [READ mode="node" target="${cartId}"]
  --NEXT--> [BRANCH condition="$.result == null" mode="boolean"]
              --TRUE--> [RESPOND status=404]
              --FALSE-->
                [VALIDATE schema="checkout:cart"]
                --NEXT--> [ITERATE source="$.result.items" maxIterations=100 collectAs="inventoryChecks" parallel=true]
                            --BODY--> [CALL subgraph="commerce:check_inventory"]
                          --NEXT--> [BRANCH condition="$.inventoryChecks.some(c => !c.available)" mode="boolean"]
                                      --TRUE--> [RESPOND status=409 body="Insufficient inventory"]
                                      --FALSE-->
                                        # Payment (the dangerous part)
                                        [INVOKE runtime="commerce-stripe" entryPoint="chargeCard"
                                               args=["${$.paymentToken}", "${$.total}"]
                                               gasBudget=10000 timeout=30000]
                                        --NEXT--> [BRANCH condition="$.result.success" mode="boolean"]
                                                    --TRUE-->
                                                      # Create order
                                                      [WRITE action="create" labels=["Order"]]
                                                      --NEXT--> [ITERATE source="$.cart.items" maxIterations=100]
                                                                  --BODY--> [WRITE action="update" target="${$.item.inventoryId}"]
                                                                --NEXT--> [EMIT event="commerce:orderCreated"]
                                                                --NEXT--> [CALL subgraph="communications:send_confirmation"]
                                                                --NEXT--> [RESPOND status=201]
                                                    --FALSE-->
                                                      [RESPOND status=402 body="Payment failed"]
```

### Verdict: WORKS, BUT COMPENSATION IS AWKWARD

The linear success path is clean. The problem is compensation. If WRITE (create order) succeeds but a subsequent WRITE (decrement inventory) fails, the order must be rolled back and the payment refunded.

The spec says: "Compensate = BRANCH (on error) + CALL (undo subgraph)." Let me try that.

```
# With compensation composed:
[INVOKE ... chargeCard]
  --NEXT--> [WRITE action="create" labels=["Order"]]
              --ON_ERROR--> [CALL subgraph="commerce:refund_payment" inputMap={"chargeId": "$.chargeId"}]
                              --NEXT--> [RESPOND status=500]
  --NEXT--> [ITERATE source="$.items" maxIterations=100]
              --BODY--> [WRITE action="update" ...]
                          --ON_ERROR--> [CALL subgraph="commerce:delete_order"]
                                          --NEXT--> [CALL subgraph="commerce:refund_payment"]
                                          --NEXT--> [RESPOND status=500]
```

**Problem: compensation chains get verbose and error-prone.** Each compensating CALL must also handle ITS failure. The DX doc's `Compensate` node solves this elegantly by declaratively pairing each step with its undo. With the 12 primitives, the developer must manually wire each ON_ERROR path and handle compensation-of-compensation.

**FINDING 1: Compensation is expressible but the composition is fragile.** The DX doc's Compensate node is syntactic sugar, but it prevents a class of bugs (forgetting to undo step 2 when step 3 fails). This is a DX issue, not an expressiveness issue. The 12 primitives CAN express it, but a responsible system would provide a compensation pattern helper (a well-known subgraph template, not a new primitive).

### Additional issue: INVOKE for payment

The payment call goes through INVOKE (WASM sandbox). Stripe's SDK requires HTTP access, which the sandbox does not have. So INVOKE here means: a host function within the WASM sandbox calls out to Stripe. The sandbox code computes what to call, and the host function makes the actual HTTP request. This works but means the INVOKE node needs to declare external HTTP access as a capability, not just graph access. The 12 primitives handle this via capability properties on INVOKE.

---

## Scenario 3: Sync -- Receive Remote Subgraph, Validate, Merge, Notify

**User story:** Instance B receives a subgraph from Instance A via CRDT sync. It must validate the incoming nodes, check capabilities, merge with local state, and notify local subscribers.

### Subgraph

```
# Subgraph: "sync:receive_delta"
[GATE mode="capability" check="sync:receive:${$.scope}"]
  --NEXT--> [ITERATE source="$.delta.nodes" maxIterations=10000 collectAs="validated"]
              --BODY--> [VALIDATE schema="${$.item.labels[0]}"]
                          --ON_ERROR--> [TRANSFORM expression="{rejected: true, nodeId: $.item.id, reason: $.error}"]
            --NEXT--> [BRANCH condition="$.validated.some(v => v.rejected)" mode="boolean"]
                        --TRUE--> [EMIT event="sync:validationWarning"]
                                    --NEXT--> [TRANSFORM expression="{nodes: $.validated.filter(v => !v.rejected)}"]
                                    --NEXT--> (continue to merge)
                        --FALSE--> (continue to merge)
            # Merge phase
            --NEXT--> [ITERATE source="$.nodes" maxIterations=10000 parallel=true collectAs="merged"]
                        --BODY--> [CALL subgraph="sync:merge_single_node"]
                      --NEXT--> [EMIT event="sync:deltaApplied"]
                      --NEXT--> [RESPOND channel="value"]
```

### Verdict: WORKS, BUT REVEALS A GAP

**Problem: The expression evaluator does complex things.** `$.validated.some(v => v.rejected)` and `$.validated.filter(v => !v.rejected)` are higher-order operations (lambdas over arrays). The systems doc says TRANSFORM uses "JSONPath-like, arithmetic, string ops." Can the sandboxed expression evaluator handle `filter` and `some` with predicates?

If yes: these are effectively MAP and FILTER inside TRANSFORM, which means TRANSFORM is doing what ITERATE + BRANCH should do. The line between "pure expression" and "iteration" blurs.

If no: the filter/some must be done as ITERATE + BRANCH + TRANSFORM, which adds 6+ nodes for what is conceptually a one-liner.

**FINDING 2: The expression evaluator's capability set determines how many nodes are needed.** A richer expression evaluator (with array.filter, array.some, array.map as built-ins) reduces node count dramatically. A minimal evaluator forces everything into ITERATE, making subgraphs verbose. This is not a primitives gap -- it is a design tension in TRANSFORM's scope.

**Recommendation:** TRANSFORM should support `filter()`, `map()`, `some()`, `every()`, `find()` as built-in array operations (pure, no I/O, bounded by input size). These are not iteration in the engine sense (they do not walk new graph territory), they are value-level array operations. This keeps ITERATE for graph-touching loops and TRANSFORM for data-level array operations.

### Additional concern: Transaction boundary

The merge phase writes many nodes. If one merge fails, should all merges roll back? The spec says "If the subgraph is transactional, all WRITEs are rolled back." Sync merges should be all-or-nothing for consistency. The `transactional: true` property on the root node handles this. No gap here.

---

## Scenario 4: AI Agent -- Discover Schema, Create Content, Check Approval, Publish

**User story:** An AI agent discovers available content types, creates a draft blog post, requests human approval, and publishes upon approval.

### Subgraph

```
# Step 1: Discover schema
[READ mode="query" target="MATCH (ct:ContentType) RETURN ct"]
  --NEXT--> [BRANCH condition="$.result.find(ct => ct.id == 'post') != null" mode="boolean"]
              --TRUE-->
                # Step 2: Create draft
                [TRANSFORM mode="template" template={"title": "...", "content": "...", "status": "draft"}]
                --NEXT--> [VALIDATE schema="contentType:post"]
                --NEXT--> [WRITE action="create" labels=["Content","Post"]]
                --NEXT--> [EMIT event="content:afterCreate"]

                # Step 3: Request approval
                --NEXT--> [WRITE action="create" labels=["ApprovalRequest"]]
                --NEXT--> [EMIT event="approval:requested"]
                --NEXT--> [WAIT mode="signal" signalId="approval:${$.postId}" timeout=604800000]
                            --NEXT--> [BRANCH condition="$.signal.approved" mode="boolean"]
                                        --TRUE-->
                                          # Step 4: Publish
                                          [WRITE action="update" target="${$.postId}" data={"status": "published"}]
                                          --NEXT--> [EMIT event="content:published"]
                                          --NEXT--> [RESPOND channel="value" body={"published": true}]
                                        --FALSE-->
                                          [RESPOND channel="value" body={"published": false, "reason": "$.signal.reason"}]
                            --ON_TIMEOUT-->
                              [RESPOND channel="value" body={"published": false, "reason": "approval_timeout"}]
              --FALSE-->
                [RESPOND channel="value" body={"error": "content type 'post' not found"}]
```

### Verdict: WORKS WELL

This is an excellent test of WAIT. The subgraph suspends for up to 7 days waiting for human approval. The engine serializes the execution state. When a human signals approval, execution resumes. This is the workflow pattern expressed purely in the 12 primitives.

**No issues found.** WAIT + BRANCH + WRITE covers the human-in-the-loop pattern completely.

### Observation: AI agents composing subgraphs

The more interesting question: can an AI agent CONSTRUCT a subgraph at runtime? The spec says INV-2 (no self-modification during execution). An agent would need to: (1) construct a subgraph definition via WRITE, (2) register it (system-zone operation), (3) CALL it. Steps 1-2 require platform-level capabilities. This is intentionally restrictive -- an agent cannot create arbitrary executable code without operator approval. The 12 primitives correctly enforce this boundary.

---

## Scenario 5: Game -- Update Position, Check Collision, Update Leaderboard, Notify

**User story:** Player moves. Server updates position, checks collision rules, updates leaderboard score if conditions met, notifies nearby players.

### Subgraph

```
# Subgraph: "game:player_move"
[GATE mode="capability" check="game:write:player/${$.playerId}"]
  --NEXT--> [VALIDATE schema="playerMove"]
  --NEXT--> [WRITE action="update" target="${$.playerId}" data={"x": "${$.newX}", "y": "${$.newY}"}]

  # Collision detection
  --NEXT--> [READ mode="query" target="MATCH (p:Player) WHERE p.x > ${$.newX - 10} AND p.x < ${$.newX + 10} AND p.y > ${$.newY - 10} AND p.y < ${$.newY + 10} AND p.id != ${$.playerId} RETURN p"]
  --NEXT--> [ITERATE source="$.result" maxIterations=50 collectAs="collisions"]
              --BODY--> [CALL subgraph="game:check_collision_pair"]
            --NEXT--> [BRANCH condition="len($.collisions.filter(c => c.hit)) > 0" mode="boolean"]
                        --TRUE--> [CALL subgraph="game:resolve_collisions"]
                                    --NEXT--> (continue)
                        --FALSE--> (continue)

  # Leaderboard update
  --NEXT--> [CALL subgraph="game:update_leaderboard"]

  # Notify nearby players
  --NEXT--> [READ mode="query" target="MATCH (p:Player) WHERE distance(p, ${$.newPos}) < 100 RETURN p.connectionId"]
  --NEXT--> [ITERATE source="$.result" maxIterations=200 parallel=true]
              --BODY--> [EMIT event="game:playerMoved" data={"playerId": "${$.playerId}", "pos": "${$.newPos}"}]
            --NEXT--> [RESPOND status=200]
```

### Verdict: WORKS, BUT REVEALS PERFORMANCE CONCERNS

The subgraph is correct. Every step is expressible. But:

**FINDING 3: Real-time game tick at 60fps = 16ms budget. This subgraph has 10+ nodes including graph queries.** The spec promises microsecond per-node evaluation. With IVM (materialized views for nearby players, leaderboard), the READ steps are O(1). The critical question is whether the engine can execute a 10-node subgraph in under 1ms total. If each node is 10us (including IVM reads), total is 100us -- well within budget. If nodes average 100us (including WRITE + IVM propagation), total is 1ms -- tight but workable.

This is not a vocabulary gap. The 12 primitives handle the logic. The question is engine performance, which is outside the vocabulary scope.

**FINDING 4: Spatial queries.** `distance(p, point) < 100` is a spatial query. Is this a built-in function in READ's Cypher? Or does it require a custom IVM view? Spatial indexing is an engine capability question, not a vocabulary question. The vocabulary correctly delegates this to READ mode="query".

### Notification fan-out

ITERATE with `parallel=true` over 200 nearby players, each producing an EMIT. This is the notification fan-out pattern. The engine needs to handle 200 parallel EMITs efficiently. Again, an engine implementation concern, not a vocabulary gap.

---

## Scenario 6: Workflow -- Content Approval (Editor -> Reviewer -> Publisher)

**User story:** Author submits content. Editor reviews. If approved, reviewer checks. If approved, publisher publishes. At any stage, rejection returns to author with feedback.

### Subgraph

```
# Subgraph: "workflow:content_approval"
[GATE mode="capability" check="content:submit:${$.contentType}"]
  --NEXT--> [WRITE action="update" target="${$.contentId}" data={"status": "pending_editor"}]
  --NEXT--> [EMIT event="workflow:editorReviewNeeded"]

  # Stage 1: Editor review
  --NEXT--> [WAIT mode="signal" signalId="review:editor:${$.contentId}" timeout=259200000]  # 3 days
              --NEXT--> [BRANCH condition="$.signal.decision" mode="match"]
                          --MATCH:approve-->
                            [WRITE action="update" target="${$.contentId}" data={"status": "pending_reviewer"}]
                            --NEXT--> [EMIT event="workflow:reviewerReviewNeeded"]

                            # Stage 2: Reviewer review
                            --NEXT--> [WAIT mode="signal" signalId="review:reviewer:${$.contentId}" timeout=259200000]
                                        --NEXT--> [BRANCH condition="$.signal.decision" mode="match"]
                                                    --MATCH:approve-->
                                                      [WRITE action="update" target="${$.contentId}" data={"status": "pending_publisher"}]
                                                      --NEXT--> [EMIT event="workflow:publisherReviewNeeded"]

                                                      # Stage 3: Publisher publish
                                                      --NEXT--> [WAIT mode="signal" signalId="review:publisher:${$.contentId}" timeout=259200000]
                                                                  --NEXT--> [BRANCH condition="$.signal.decision" mode="match"]
                                                                              --MATCH:approve-->
                                                                                [WRITE action="update" target="${$.contentId}" data={"status": "published", "publishedAt": "now()"}]
                                                                                --NEXT--> [EMIT event="content:published"]
                                                                                --NEXT--> [RESPOND channel="value" body={"published": true}]
                                                                              --MATCH:reject-->
                                                                                [CALL subgraph="workflow:return_to_author" inputMap={"feedback": "$.signal.feedback"}]
                                                                              --DEFAULT-->
                                                                                [RESPOND channel="value" body={"error": "unexpected decision"}]
                                                                  --ON_TIMEOUT-->
                                                                    [CALL subgraph="workflow:timeout_escalation"]
                                                    --MATCH:reject-->
                                                      [CALL subgraph="workflow:return_to_author"]
                                        --ON_TIMEOUT-->
                                          [CALL subgraph="workflow:timeout_escalation"]
                          --MATCH:reject-->
                            [CALL subgraph="workflow:return_to_author"]
                          --MATCH:revise-->
                            [WRITE action="update" target="${$.contentId}" data={"status": "revision_requested"}]
                            --NEXT--> [EMIT event="workflow:revisionRequested"]
                            --NEXT--> [RESPOND channel="value" body={"status": "revision_requested"}]
              --ON_TIMEOUT-->
                [CALL subgraph="workflow:timeout_escalation"]
```

### Verdict: WORKS, BUT NESTING DEPTH IS THE REAL PROBLEM

The primitives handle every step. Three sequential WAIT nodes with BRANCH decisions between them. Each stage is clean in isolation. But the overall subgraph nests 3 WAIT stages deep, each with MATCH branches and timeout handlers. This is a ~25-node subgraph with significant nesting.

**FINDING 5: Multi-stage workflows produce deep nesting.** The DAG becomes wide and deep. Each WAIT creates a branching point with NEXT + ON_TIMEOUT. Each BRANCH inside creates more branches. The result is correct but hard to read as a flat DAG.

**The solution is already in the vocabulary: CALL.** Factor each stage into its own subgraph:

```
[CALL subgraph="workflow:editor_review"]
  --NEXT--> [BRANCH condition="$.approved" mode="boolean"]
              --TRUE--> [CALL subgraph="workflow:reviewer_review"]
                          --NEXT--> [BRANCH condition="$.approved"]
                                      --TRUE--> [CALL subgraph="workflow:publisher_publish"]
                                      --FALSE--> [CALL subgraph="workflow:return_to_author"]
              --FALSE--> [CALL subgraph="workflow:return_to_author"]
```

Now the top-level subgraph is 7 nodes. Each stage subgraph is 5-6 nodes. Much more readable. No new primitives needed -- CALL solves the complexity problem by decomposition.

---

## Scenario 7: Module Install -- Validate Subgraphs, Register Types, Create Defaults

**User story:** A new module is installed. Its operation subgraphs must be validated for safety, its content types registered, field types registered, and default content created.

### Subgraph

```
# Subgraph: "system:install_module" (platform-level, not user-authored)
[GATE mode="capability" check="system:install:module"]
  --NEXT--> [READ mode="node" target="${$.moduleId}"]
  --NEXT--> [BRANCH condition="$.result == null" mode="boolean"]
              --TRUE--> [RESPOND status=404 body="Module not found"]
              --FALSE-->
                # Phase 1: Validate all operation subgraphs in the module
                [READ mode="query" target="MATCH (m:Module {id: '${$.moduleId}'})-[:CONTAINS]->(s:Subgraph) RETURN s"]
                --NEXT--> [ITERATE source="$.result" maxIterations=1000 collectAs="validationResults"]
                            --BODY--> [CALL subgraph="system:validate_operation_subgraph"]
                          --NEXT--> [BRANCH condition="$.validationResults.every(r => r.valid)" mode="boolean"]
                                      --FALSE--> [RESPOND status=422 body={"errors": "$.validationResults.filter(r => !r.valid)"}]
                                      --TRUE-->
                                        # Phase 2: Register content types
                                        [READ mode="query" target="MATCH (m:Module {id: '${$.moduleId}'})-[:DEFINES]->(ct:ContentTypeDef) RETURN ct"]
                                        --NEXT--> [ITERATE source="$.result" maxIterations=100]
                                                    --BODY--> [WRITE action="create" labels=["ContentType"]]
                                                  --NEXT-->

                                                  # Phase 3: Register field types
                                                  [READ mode="query" target="MATCH (m:Module {id: '${$.moduleId}'})-[:DEFINES]->(ft:FieldTypeDef) RETURN ft"]
                                                  --NEXT--> [ITERATE source="$.result" maxIterations=100]
                                                              --BODY--> [WRITE action="create" labels=["FieldType"]]
                                                            --NEXT-->

                                                            # Phase 4: Create default content
                                                            [READ mode="query" target="MATCH (m:Module {id: '${$.moduleId}'})-[:SEEDS]->(d:DefaultContent) RETURN d"]
                                                            --NEXT--> [ITERATE source="$.result" maxIterations=100]
                                                                        --BODY--> [CALL subgraph="system:create_default_content"]
                                                                      --NEXT-->

                                                                      # Phase 5: Activate module
                                                                      [WRITE action="update" target="${$.moduleId}" data={"status": "active"}]
                                                                      --NEXT--> [EMIT event="module:installed"]
                                                                      --NEXT--> [RESPOND status=200]
```

### Verdict: WORKS

Module installation is a sequence of READ + ITERATE + WRITE + VALIDATE patterns. The subgraph validation in Phase 1 (calling `system:validate_operation_subgraph` for each subgraph) is the most interesting part -- it is the engine validating code before it runs, which is a core safety property.

**No primitives missing.** The 12 cover module installation completely.

**FINDING 6: This subgraph is platform-level.** It runs with system capabilities. The distinction between user-authored subgraphs (capability-attenuated) and platform subgraphs (full access) is not a vocabulary concern -- it is a trust tier concern handled by GATE's capability check system.

---

## Cross-Scenario Analysis: Attempting to Break the Vocabulary

### Test A: Can I express "do X atomically with CAS semantics"?

WRITE has `conditional CAS` in its description. Example: update status to "published" only if current status is "draft."

```
[WRITE action="update" target="${$.id}" data={"status": "published"} expectedVersion="${$.currentVersion}"]
  --ON_ERROR--> [BRANCH condition="$.error.code == 'VERSION_CONFLICT'" mode="boolean"]
                  --TRUE--> [RESPOND status=409]
                  --FALSE--> [RESPOND status=500]
```

Works. CAS via `expectedVersion` on WRITE. No gap.

### Test B: Can I express "aggregate over a query result"?

Compute the total revenue from all orders.

```
[READ mode="query" target="MATCH (o:Order {status: 'completed'}) RETURN o.total"]
  --NEXT--> [TRANSFORM expression="sum($.result)"]
  --NEXT--> [RESPOND status=200]
```

This requires `sum()` as a built-in in the expression evaluator. If the evaluator has `sum()`, `avg()`, `min()`, `max()` -- it works. If not, you need ITERATE + TRANSFORM to accumulate. See Finding 2.

Alternatively: IVM materializes the aggregate. Then it is just READ mode="view".

### Test C: Can I express "rate limiting"?

Reject requests if the user has made more than 100 requests in the last minute.

```
[READ mode="view" target="rate_limit:${$.userId}:per_minute"]
  --NEXT--> [BRANCH condition="$.result.count >= 100" mode="boolean"]
              --TRUE--> [RESPOND status=429]
              --FALSE--> [WRITE action="update" target="rate_limit:${$.userId}" data={"count": "$.result.count + 1"}]
                           --NEXT--> (continue)
```

Works via IVM (materialized view of request count). The rate limit window is maintained by the view definition. No gap in the vocabulary.

### Test D: Can I express "streaming/pagination"?

Return content items in pages of 20.

```
[READ mode="query" target="MATCH (c:Content) RETURN c" options={"limit": 20, "offset": "${$.page * 20}", "sort": "createdAt desc"}]
  --NEXT--> [READ mode="query" target="MATCH (c:Content) RETURN count(c)"]
  --NEXT--> [TRANSFORM mode="template" template={"items": "$.results[0]", "total": "$.results[1].count", "page": "${$.page}", "pages": "ceil($.results[1].count / 20)"}]
  --NEXT--> [RESPOND status=200]
```

Works. Standard pagination via READ options. The second READ for count could be an IVM view for O(1).

**What about true streaming (Server-Sent Events, WebSocket)?** RESPOND produces a single output. It cannot produce a stream of outputs. For SSE/WebSocket, the pattern would be: subscribe via WAIT, and for each change, EMIT to a connection-specific channel.

**FINDING 7: Streaming output is not directly expressible as a single subgraph.** A streaming endpoint would need: (1) an initial subgraph that sets up a subscription (via WAIT/reactive), and (2) per-event subgraphs triggered by reactive notifications that each produce a partial RESPOND to the connection. This works via the reactive layer (IVM subscriptions trigger subgraphs), but the "streaming response" concept does not map to a single RESPOND node.

**Is this a gap?** Not really. The engine's reactive subscription layer handles streaming at a layer below the operation vocabulary. A subscribing client receives graph change notifications directly via the reactive system -- no operation subgraph needed per-notification. The initial subscription setup is a single subgraph. This is architecturally correct: streaming is a transport concern, not an operation concern.

### Test E: Can I express "bulk import with progress tracking"?

Import 10,000 records from a CSV, updating a progress indicator.

```
[VALIDATE schema="import:csv"]
  --NEXT--> [TRANSFORM expression="parseCSV($.body)"]  # Needs CSV parsing in TRANSFORM or INVOKE
  --NEXT--> [ITERATE source="$.rows" maxIterations=10000 collectAs="results"]
              --BODY--> [VALIDATE schema="contentType:${$.contentType}"]
                          --NEXT--> [WRITE action="create" labels=["Content","${$.contentType}"]]
                          --ON_ERROR--> [TRANSFORM expression="{error: $.error, row: $.index}"]
            --NEXT--> [TRANSFORM mode="template" template={"imported": "len($.results.filter(r => !r.error))", "failed": "len($.results.filter(r => r.error))"}]
            --NEXT--> [RESPOND status=200]
```

**Problem 1:** CSV parsing. `parseCSV()` is complex string processing -- not suitable for the sandboxed expression evaluator. This needs INVOKE (WASM sandbox) or a built-in engine function. The 12 primitives handle this via INVOKE.

**Problem 2:** Progress tracking. The subgraph runs synchronously (all 10,000 iterations complete before RESPOND). There is no way to emit progress during ITERATE. EMIT inside the ITERATE body would work for fire-and-forget progress notifications:

```
--BODY--> [WRITE ...]
            --NEXT--> [BRANCH condition="$.index % 100 == 0"]
                        --TRUE--> [EMIT event="import:progress" data={"imported": "$.index"}]
```

This works. EMIT inside ITERATE for periodic progress updates. The consumer subscribes to `import:progress` events. No gap.

### Test F: Can I express "distributed lock / mutex"?

Ensure only one instance processes a particular job.

```
[WRITE action="create" labels=["Lock"] data={"jobId": "${$.jobId}", "owner": "${$.instanceId}", "expiresAt": "now() + 60000"}]
  --ON_ERROR--> [BRANCH condition="$.error.code == 'DUPLICATE'" mode="boolean"]
                  --TRUE--> [RESPOND status=409 body="Lock held by another instance"]
                  --FALSE--> [RESPOND status=500]
  --NEXT--> (do the work)
  --NEXT--> [WRITE action="delete" target="${$.lockId}"]
  --NEXT--> [RESPOND status=200]
```

Works via conditional WRITE (unique constraint on jobId). The lock has a TTL via `expiresAt` (the engine or a background job cleans up expired locks). This is a composition of WRITE + BRANCH, not a new primitive.

### Test G: Can I express "content transformation pipeline" (like the CMS materializer)?

```
[READ mode="node" target="${compositionId}"]
  --NEXT--> [ITERATE source="$.blocks" maxIterations=500 collectAs="visible"]
              --BODY--> [CALL subgraph="cms:evaluate_visibility"]
            --NEXT--> [ITERATE source="$.visible" maxIterations=500 collectAs="resolved"]
              --BODY--> [BRANCH condition="$.item.compositionRef != null"]
                          --TRUE--> [CALL subgraph="cms:render_composition"]  # Recursive
                          --FALSE--> [TRANSFORM expression="$.item"]
            --NEXT--> [ITERATE source="$.resolved" maxIterations=500 collectAs="bound"]
              --BODY--> [CALL subgraph="cms:resolve_block_data"]
            --NEXT--> [TRANSFORM mode="template" template={"blocks": "$.bound", "meta": "..."}]
            --NEXT--> [RESPOND channel="value"]
```

This is already in the systems doc. Works. The recursive CALL is bounded by timeout and depth limit.

---

## Redundancy Analysis: Can Any Two Primitives Be Merged?

### GATE vs VALIDATE

GATE has modes: `capability`, `validate`, `condition`, `transform`. VALIDATE checks schema + referential integrity. Could VALIDATE be absorbed into GATE mode="validate"?

**Argument for merging:** GATE mode="validate" and VALIDATE both check data against a schema. Having both creates confusion about which to use.

**Argument against:** VALIDATE is richer (referential integrity, schema modes "strict"/"partial"). GATE mode="validate" is a simpler pass/fail check. VALIDATE returns structured error details (field-level errors). GATE returns pass/reject.

**FINDING 8: GATE and VALIDATE overlap significantly.** The security doc's `ValidateSchema` and `ValidateRelationship` are two separate operations. The systems doc merges them into one `VALIDATE`. The recommendation: **keep them separate.** GATE is about authorization and interception (middleware pattern). VALIDATE is about data integrity (schema pattern). They serve different conceptual roles even when they look similar syntactically. Merging them would create a Swiss Army knife node that does too many things.

### GATE vs BRANCH

GATE with `onReject: "skip"` behaves like BRANCH (continue on one path or another). But GATE can transform data (mode="transform"), which BRANCH cannot. GATE is middleware; BRANCH is router. They are genuinely different.

**Verdict: No merge. Correct as-is.**

### CALL vs INVOKE

CALL executes a graph subgraph. INVOKE executes WASM code. Could they be merged into a single "execute external thing" primitive?

**Argument for merging:** Both execute something and return a result. The difference is the execution environment.

**Argument against:** The security properties are fundamentally different. CALL shares the transaction, inherits (attenuated) capabilities, and is inspectable. INVOKE is sandboxed, metered, non-reentrant, and opaque. Merging them would either weaken CALL's guarantees or add unnecessary complexity to every CALL.

**Verdict: No merge. The security boundary between graph-native and sandboxed execution is the most important architectural distinction in the system.**

### EMIT vs RESPOND

EMIT continues execution; RESPOND terminates it. EMIT is fire-and-forget; RESPOND blocks the caller until the value is delivered. They are genuinely different.

**Verdict: No merge. Correct as-is.**

### READ vs WRITE

Obviously different. One observes, one mutates.

**Verdict: No merge.**

---

## What Is Actually Missing?

### Missing 1: AGGREGATE (Debatable)

Aggregation (sum, count, avg, min, max, group-by) over query results is extremely common. Currently expressed as READ + TRANSFORM (with `sum()` etc. in the expression evaluator) or as IVM views.

If the expression evaluator has aggregate functions: no gap.
If it does not: every aggregation requires ITERATE + TRANSFORM with accumulator.

**Recommendation: Not a new primitive. Add aggregate functions to the expression evaluator.**

### Missing 2: MERGE/JOIN (Debatable)

Combining data from two different READs into a single structure. Currently: two READs followed by a TRANSFORM with `merge` mode. This works but is verbose for a very common pattern.

```
[READ mode="node" target="${$.userId}"]  # user
  --NEXT--> [READ mode="query" target="MATCH (o:Order)-[:PLACED_BY]->(u:User {id: '${$.userId}'}) RETURN o"]  # orders
  --NEXT--> [TRANSFORM mode="merge" template={"user": "$.reads[0]", "orders": "$.reads[1]"}]
```

**Problem:** The second READ's input is not the user node -- it is the original context plus the first READ's result. How does data flow between sequential READs? The context accumulates: each node's output is merged into the context for the next node.

**Recommendation: Not a new primitive. Context accumulation is an execution model feature, not a vocabulary feature. Clarify that the execution context is a growing map, not a replacement chain.**

### Missing 3: Nothing for Batch/Bulk Operations (Not Missing)

Bulk create 100 items = ITERATE with WRITE in the body. The `parallel` flag on ITERATE handles concurrent writes. No gap.

### Missing 4: Nothing for Schema Introspection (Not Missing)

"What content types exist?" = READ mode="query" with label filter. Content type definitions are Nodes in the graph. The vocabulary handles this as data access, not as a special operation.

### Missing 5: Nothing for Capability Introspection

"What capabilities do I have?" Is this a READ? The capabilities are in the system zone. A regular READ cannot access system zone Nodes. The security doc's `ReadCapability` operation solves this, but it is not one of the 12.

**FINDING 9: Capability introspection requires either a special READ mode or GATE mode.** The current 12 do not have it. A developer cannot write `[READ mode="my_capabilities"]` because capabilities are system-zone. GATE mode="capability" checks a SPECIFIC capability but does not list all capabilities.

**Recommendation: Add a `mode` to GATE or READ for capability introspection.** GATE mode="introspect" returns the current capability envelope. This is a minor extension, not a new primitive. It enables AI agents to check their own capabilities before constructing a write, avoiding wasted computation.

---

## Summary of Findings

| # | Finding | Severity | Resolution |
|---|---------|----------|------------|
| 1 | Compensation is expressible but fragile -- easy to forget undo steps | MEDIUM | Provide a well-documented compensation subgraph TEMPLATE (not a new primitive). Module tooling should enforce paired undo registrations. |
| 2 | Expression evaluator scope determines subgraph verbosity | MEDIUM | Document that TRANSFORM's expression language includes `filter()`, `map()`, `some()`, `every()`, `find()`, `sum()`, `avg()`, `min()`, `max()` as array built-ins. These are pure, bounded, and do not touch the graph. |
| 3 | Real-time games need sub-ms evaluation of 10+ node subgraphs | LOW | Engine performance concern, not vocabulary gap. IVM covers the hot-path reads. |
| 4 | Spatial queries are a READ/IVM concern, not a vocabulary concern | LOW | Engine feature, not primitive. |
| 5 | Multi-stage workflows nest deeply | LOW | CALL decomposition solves it. Standard pattern. |
| 6 | Platform vs user subgraphs differ in trust, not vocabulary | LOW | Handled by capability system. |
| 7 | Streaming output does not map to a single RESPOND | LOW | Correctly handled by the reactive layer below the vocabulary. |
| 8 | GATE and VALIDATE overlap but serve different conceptual roles | LOW | Keep separate. Document when to use which. |
| 9 | Capability introspection not covered by the 12 | MEDIUM | Add GATE mode="introspect" or READ mode for own-capabilities. |

---

## Verdict: Can the 12 Cover All System Needs?

**YES, with two clarifications.**

1. **The expression evaluator is load-bearing.** If TRANSFORM's expression language is anemic (just property access and arithmetic), many operations that should be one node become 5+ nodes (ITERATE + BRANCH + TRANSFORM). The expression evaluator MUST include array operations (filter, map, some, find) and aggregate functions (sum, avg, count). These are pure, bounded, and do not violate the "no I/O" constraint. Without these, the vocabulary is technically complete but practically verbose.

2. **Capability introspection needs a home.** The 12 primitives check specific capabilities (GATE) but cannot list available capabilities. This is needed for AI agents (dry-run planning) and for conditional UI (show publish button only if user can publish). Resolution: extend GATE with an introspection mode, not a new primitive.

**No scenario required a 13th primitive.** Every use case -- CMS, commerce, sync, AI agents, games, workflows, module installation -- is expressible with the 12. Compensation is the weakest composition (fragile without tooling), but it is expressible.

**No redundancy found.** All 12 earn their place. The closest overlap is GATE/VALIDATE, and they serve genuinely different roles (authorization vs data integrity).

### Proposed Adjustments

1. **Explicitly document the TRANSFORM expression built-in set.** Include: property access, arithmetic, string ops, comparison, ternary, `len()`, `now()`, `filter()`, `map()`, `some()`, `every()`, `find()`, `sum()`, `avg()`, `min()`, `max()`, `join()`, `split()`, `slugify()`. These are all pure and bounded.

2. **Add GATE mode="introspect"** returning the current execution context's capability envelope. Pure read, no mutation, no information disclosure (returns only the caller's own capabilities).

3. **Define the context accumulation model.** Each node's output is merged into a growing context map. Sequential READs produce `$.read1`, `$.read2` etc., not overwrites. This is implied in the spec but should be explicit.

4. **Provide a compensation template.** A well-known subgraph pattern with step/undo pairing that tooling can enforce. Not a primitive, but a blessed composition.

5. **Clarify `transactional` as a subgraph property.** Not a node type. Multiple WRITEs in a transactional subgraph are all-or-nothing. This is stated in the spec but bears repeating since it replaces a common "TRANSACTION" primitive found in other systems.
