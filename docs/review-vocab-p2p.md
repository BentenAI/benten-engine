# Review: 12 Operation Primitives -- P2P Sync Correctness

**Reviewer:** Correctness & Edge Cases Agent
**Date:** 2026-04-11
**Scope:** The 12 unified primitives (READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, GATE, CALL, RESPOND, EMIT, INVOKE, VALIDATE) evaluated against P2P sync constraints from `operation-vocab-p2p.md` and `operation-vocab-security.md`.
**Correctness Score: 6.5/10**

---

## 0. The Primitive Consolidation Problem

The four vocab documents propose different primitive sets:

| Document | Count | Primitives |
|----------|-------|------------|
| Systems/Engine | 10 | READ, WRITE, TRANSFORM, BRANCH, ITERATE, GATE, CALL, RESPOND, EMIT, WAIT |
| P2P | ~40 | Fine-grained: op:CreateNode, op:UpdateNode, op:Query, op:Transform, op:Filter, op:Map, op:Reduce, op:Condition, op:ForEach, op:Parallel, op:Collect, op:TryCatch, op:Abort, op:EmitEvent, op:CheckCapability, op:RequireCapability, op:WithCapability, op:ExternalCall, op:WasmCall, etc. |
| Security | 14 | CheckCapability, AttenuateCapability, RequireIdentity, EnforceBudget, ReadGraph, ReadView, ReadCapability, WriteGraph, ConditionalWrite, Invoke, InvokeSubgraph, Branch, Merge, Transform, ValidateSchema, ValidateRelationship, AuditPoint, DryRun, ValidateRemoteOrigin, SanitizeIncoming |
| DX | 18 | Query, Read, Write, Delete, Validate, Transform, Branch, Gate, Sequence, Parallel, Loop, Defer, Invoke, Notify, Webhook, Guard, Compensate, SubgraphRef |

The user's question references 12 primitives: READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, GATE, CALL, RESPOND, EMIT, INVOKE, VALIDATE. This is the Systems set (10) plus INVOKE and VALIDATE -- two operations the Security and DX docs argue are irreducible but the Systems doc subsumes under GATE and CALL respectively.

**CRITICAL: There is no document that defines the consolidated 12.** The four perspectives were never reconciled into a single canonical vocabulary. This review treats the 12-primitive list as the target and evaluates it against P2P constraints, noting where the P2P document's finer-grained model exposes gaps in the 12-primitive model.

---

## 1. Determinism Classification -- Detailed Analysis

### 1.1 READ -- NOT deterministic across instances

**Classification:** Read-deterministic (sync-with-context)

READ's output depends on graph state. Two instances with different graph states produce different outputs for the same READ. This is correct behavior, not a bug -- but it has a critical consequence for synced subgraphs:

**The problem:** A synced handler subgraph containing READ nodes is structurally valid (no sync-forbidden operations) but produces DIFFERENT RESULTS on different instances. The P2P doc classifies this as "sync-with-context" -- the handler syncs, but the result depends on local data.

**Edge case:** Instance A installs a handler that READs a configuration Node. Instance B syncs the handler but does not have the configuration Node. The handler executes on B and gets `null`. If the handler does not guard against `null` (no BRANCH after the READ), it crashes or produces garbage.

**Verdict:** READ is correctly classified as sync-with-context. But the 12-primitive model has no mechanism to DECLARE data dependencies. The P2P doc's fine-grained model has `READS_FROM` edges from operation nodes to their data dependencies, which enables the engine to validate "does the receiving instance have all Nodes this handler depends on?" The 12-primitive model relies on runtime null checks (BRANCH after READ), which is a weaker guarantee.

**Recommendation:** READ should have an optional `requiresData` property that declares Node labels or IDs that must exist for the handler to function. The engine can validate these dependencies during sync acceptance.

### 1.2 WRITE -- Deterministic input, non-deterministic output

**Classification:** Write-deterministic (sync-result-only)

Given the same input data, WRITE produces the same mutation. But:

1. **Version numbers differ per instance.** Instance A's version chain for Node X may be at version 7; Instance B's may be at version 12 (different edit histories). A handler that WRITEs to Node X and returns `{ id, version }` produces different `version` values on each instance. If a downstream operation uses the version number (e.g., for optimistic locking or display), behavior diverges.

2. **Anchor IDs for creates.** When WRITE creates a new Node, the anchor ID is generated locally. Instance A generates `anchor-abc`, Instance B generates `anchor-def`. If the handler is triggered by the same event on both instances, both create a Node -- but with different IDs. After sync, BOTH Nodes exist (add-wins CRDT). This is a **duplicate creation problem**.

**The P2P doc's solution (Section 3.3):** WRITEs are classified as "sync-result-only." The handler's DEFINITION syncs, but its EXECUTION results sync as data (version chain Nodes), not as re-execution. Instance B does not re-execute the handler -- it receives Instance A's write results via CRDT.

**But this raises a harder question:** If a handler triggers on an event (e.g., `content:afterCreate`) and BOTH instances receive that event (because the triggering content synced), do BOTH instances execute the handler? If yes, you get duplicate writes. If no, who decides which instance executes?

**Verdict:** The 12-primitive model does not address this. The P2P doc implicitly assumes result-only sync (Section 3.4), but the mechanism for preventing duplicate execution of event-triggered handlers is not specified. This is a **design gap**, not a bug in the primitives.

**Recommendation:** Handlers triggered by synced events need an execution assignment protocol -- either "only the instance that originated the event executes the handler" (origin-executes) or "all instances execute but writes are deduplicated via content-addressing" (execute-and-dedup). The 12 primitives need a way to annotate handlers with their execution model.

### 1.3 TRANSFORM -- Deterministic (with caveats)

**Classification:** Pure (sync-safe)

TRANSFORM uses the sandboxed expression evaluator (`@benten/expressions` semantics). Same inputs always produce same outputs. No side effects. **This is the cleanest primitive for sync.**

**Caveat:** The expression language must be IDENTICAL across instances. If Instance A runs engine version 1.2 (where `len("emoji")` counts codepoints) and Instance B runs engine version 1.3 (where `len("emoji")` counts grapheme clusters), TRANSFORM is no longer deterministic across instances.

**Verdict:** Pure and sync-safe, contingent on expression language versioning. The P2P doc does not address expression language version divergence. This is a LOW-severity gap -- it only manifests during engine version transitions.

**Recommendation:** The expression language version should be part of the content hash for operation subgraphs. A handler authored with expression language v2 has a different hash than the same handler under v3, forcing explicit migration.

### 1.4 BRANCH -- Deterministic given same inputs

**Classification:** Pure (sync-safe)

BRANCH evaluates a condition expression and selects a path. Given the same input value, it always selects the same path. The condition expression is subject to the same sandboxed evaluator constraints as TRANSFORM.

**Edge case:** BRANCH conditions that reference capability state. The P2P doc's `op:Condition` is pure (data-only conditions). But the Security doc's `Branch` supports `CapabilityHeld` conditions -- checking whether the current actor has a capability. This makes BRANCH non-deterministic across instances (different actors, different capabilities).

**Verdict:** BRANCH is sync-safe ONLY if its condition is data-only. If BRANCH can evaluate capability checks (as the Security doc proposes), it becomes sync-with-context. The 12-primitive model does not distinguish between data-BRANCH and capability-BRANCH.

**Recommendation:** BRANCH should be restricted to data-only conditions in synced subgraphs. Capability checks should go through GATE (which is explicitly non-deterministic across instances -- see 1.7).

### 1.5 ITERATE -- Deterministic if order is defined

**Classification:** Pure (sync-safe) for sequential mode; potentially non-deterministic for parallel mode

Sequential ITERATE over a collection produces results in collection order. If the collection order is deterministic (e.g., sorted by a stable key), ITERATE is deterministic.

**Problem: parallel ITERATE.** When `parallel=true`, the execution order of items is non-deterministic. If the BODY subgraph has side effects (WRITEs), the order of writes may differ between instances. The final state depends on CRDT merge, but intermediate states (version numbers, `_producedAt` timestamps) will differ.

**Deeper problem:** If the BODY contains a BRANCH whose condition depends on data written by a PREVIOUS iteration (read-your-own-writes within the same transaction), parallel ITERATE introduces a data race. Item 3's BRANCH might see Item 1's write on Instance A but not on Instance B (due to different scheduling).

**Verdict:** Sequential ITERATE is sync-safe. Parallel ITERATE is sync-safe ONLY if the BODY is side-effect-free (Pure or Read-deterministic operations only). The 12-primitive model does not enforce this constraint.

**Recommendation:** Synced subgraphs should reject parallel ITERATE with WRITE operations in the BODY. This is a structural validation rule (add to Section 10.1 of the P2P doc).

### 1.6 WAIT -- NOT deterministic

**Classification:** Non-deterministic (sync-forbidden in subgraph definitions)

WAIT depends on external signals, human approval, or wall-clock timeouts. Two instances CANNOT produce the same result for a WAIT.

**The fundamental problem with WAIT in synced handlers:** If a handler definition includes a WAIT node and that handler syncs to another instance, the handler now has TWO wait points -- one on each instance. A signal sent to Instance A's WAIT does not automatically resolve Instance B's WAIT.

**Scenario analysis:**
- **Approval workflows:** Handler syncs. User triggers handler on Instance A. Handler WAITs for approval. Admin approves on Instance A. WAIT resolves, handler continues, WRITE occurs. The WRITE result syncs to Instance B. Instance B never executed the handler -- it received the result. **This works correctly** under result-only sync.
- **But:** If the handler is triggered on BOTH instances (event-triggered), BOTH instances WAIT. Who approves? If Admin approves on Instance A, does Instance B's WAIT resolve? No -- signals are local. Instance B's WAIT times out.

**Verdict:** WAIT is correctly classified as non-deterministic. The P2P doc's rule (Section 3.4: "synced subgraphs must not contain non-deterministic operations") should apply. But WAIT is essential for long-running operations. The resolution is that WAIT HANDLERS sync their DEFINITION but not their EXECUTION state. Each instance executes independently, and results sync via CRDT.

**Recommendation:** WAIT should be flagged as "execution-local" in the structural validation rules. A synced handler containing WAIT is structurally valid (the DEFINITION can sync) but the engine must ensure only one instance executes it per triggering event (back to the execution assignment problem from 1.2).

### 1.7 GATE -- Non-deterministic across instances

**Classification:** Depends on mode:
- `capability` mode: Read-deterministic (depends on local capability graph)
- `validate` mode: Pure (deterministic given same schema and data)
- `condition` mode: Pure (data-only condition)
- `transform` mode: Pure (expression-based)

**The GATE problem you asked about:** A synced handler has GATE checks (capability mode). Instance A has different capabilities than Instance B. The same handler may pass GATE on A but fail on B.

**This is CORRECT BEHAVIOR, not a sync bug.** GATE is the local enforcement point. When a handler syncs from Instance A to Instance B, Instance B's GATE checks use Instance B's capability grants. If Instance B has not granted the module the required capabilities, the handler correctly fails. This is capability enforcement working as designed.

**But there is a subtlety:** If the GATE failure causes the handler to ABORT (no REJECT path), Instance B never produces a result for that handler invocation. Instance A produces a result (the handler succeeds). After sync, Instance B has the result from Instance A but its own GATE says "this shouldn't have been allowed." This is an INCONSISTENCY -- Instance B has data that its local policy would have rejected.

**This is the same problem as Holochain DNA version mismatch (P2P doc Section 5.2, Case 1).** The solution options are:
1. **Reject the data during sync** (strict: Instance B's validation rejects data produced by handlers that Instance B's GATE would reject). This breaks data sync for legitimate cases (Instance A has broader permissions).
2. **Accept the data, tag it** (lenient: Instance B accepts the data but tags it with `_producedBy` showing it came from a handler execution that would fail locally). This is the P2P doc's version-stamped approach.
3. **Accept the data unconditionally** (permissive: GATE is local enforcement only; synced data is not re-validated). This is the simplest but weakest.

**Verdict:** GATE's non-determinism across instances is inherent and correct. But the 12-primitive model does not specify how GATE failures on the receiving instance interact with synced data. This is a **HIGH-severity design gap**.

**Recommendation:** Adopt option 2 (accept-and-tag). During sync inbound validation (`SanitizeIncoming` in the Security doc), validate data against the receiving instance's schemas but NOT against its capability grants. Capability grants govern what LOCAL handlers can do, not what synced data is allowed to contain.

### 1.8 CALL -- Deterministic if the called subgraph is deterministic

**Classification:** Inherits from the called subgraph

CALL invokes another named subgraph. Its determinism class is the called subgraph's determinism class. If the called subgraph contains only Pure/Read-deterministic operations, CALL is sync-safe. If the called subgraph contains WAIT or INVOKE, CALL inherits their non-determinism.

**Cross-subgraph analysis is required.** When validating a synced handler, the engine must transitively check all CALLed subgraphs for non-deterministic operations. The P2P doc mentions this (Section 12.4: "the engine must check that the handler call graph across subgraphs is also acyclic") but focuses on cycle detection, not determinism propagation.

**Edge case:** CALL references a subgraph that does not exist on the receiving instance. The handler syncs, but the called subgraph has not synced yet (ordering problem). Instance B tries to execute the handler, CALL fails with "subgraph not found."

**Verdict:** CALL is structurally correct but depends on transitive analysis and sync ordering guarantees that are not specified in the 12-primitive model.

**Recommendation:** CALL should declare its subgraph dependency as a `DEPENDS_ON` edge from the handler to the called subgraph. The sync protocol should ensure all dependencies arrive before the dependent handler is marked as executable.

### 1.9 RESPOND -- Deterministic

**Classification:** Pure (sync-safe)

RESPOND is a terminal node that packages output. Given the same input data, it produces the same output. No edge cases worth noting.

### 1.10 EMIT -- Non-deterministic side effect

**Classification:** Write-deterministic locally, sync-forbidden for the side effect

**The question you asked:** If a handler EMITs a notification, and the handler syncs to another instance, does the EMIT fire on both instances?

**Answer:** It depends on the execution model (see 1.2). Under result-only sync, only the executing instance fires the EMIT. The EMIT's side effect (reactive notifications to local subscribers) is LOCAL. The data produced by the handler (WRITEs) syncs, but the EMIT does not.

**But under event-triggered execution on both instances:** Both instances EMIT. This means subscribers on BOTH instances fire. If the subscriber itself produces WRITEs, you get cascading duplicates.

**The P2P doc classifies EmitEvent as "sync-result-only"** (Section 3.3), which is correct for the data mutation case. But EMIT's real purpose is side effects (notifications, cache invalidation, webhook delivery). These side effects are inherently local and should NOT sync.

**Scenario:** Handler EMITs `content:afterCreate`. Instance A's SEO module listens and updates an SEO score. Instance B's SEO module also listens. If the handler executes on both instances, both SEO modules fire, producing two independent SEO score updates. After CRDT merge, one wins (LWW). This is wasteful but not incorrect.

**But:** If the handler EMITs `commerce:sendReceipt` which triggers an email, and the handler executes on both instances, the customer receives TWO receipts. This IS incorrect.

**Verdict:** EMIT is correctly non-deterministic. The 12-primitive model needs to distinguish between idempotent EMITs (graph-internal: re-execution is wasteful but safe) and non-idempotent EMITs (external: re-execution causes real-world duplicate effects).

**Recommendation:** EMIT should have a `deliveryMode` property: `local` (fire on this instance only), `exactly-once` (fire on exactly one instance in the sync group, using distributed lock or leader election), or `broadcast` (fire on all instances, for truly idempotent operations).

### 1.11 INVOKE -- Non-deterministic

**Classification:** Non-deterministic (sync-forbidden)

INVOKE delegates to a WASM/QuickJS sandbox. The sandbox may use randomness, time, or external APIs. The P2P doc classifies `op:ExternalCall` and `op:WasmCall` as non-deterministic and sync-forbidden.

**The question you asked:** Can INVOKE appear in a synced subgraph?

**Answer:** The HANDLER DEFINITION containing INVOKE can sync. But INVOKE is a non-deterministic operation, so the receiving instance MUST flag the handler as requiring local execution. The handler's results (data mutations from WRITEs that follow the INVOKE) sync as data, not as re-execution.

**Edge case:** A handler contains INVOKE followed by WRITE. The INVOKE result feeds into the WRITE. Instance A executes: INVOKE returns `{ tax: 12.50 }`, WRITE creates `{ total: 112.50 }`. The WRITE result syncs to Instance B. Instance B's engine receives the Node with `total: 112.50` -- it does not re-run the tax calculation.

**But:** If Instance B later needs to re-process the data (migration, version upgrade), it must re-execute the INVOKE. The INVOKE may produce a different result (tax rates changed). The `_producedBy` tag (P2P doc Section 5.3) helps identify which data needs re-processing.

**Verdict:** INVOKE is correctly non-deterministic. The sync model (result-only) handles it. The 12-primitive model is consistent here.

### 1.12 VALIDATE -- Deterministic (with schema versioning caveat)

**Classification:** Pure (sync-safe) given same schema

VALIDATE checks data against a schema definition. If the schema is the same on both instances, VALIDATE produces the same result.

**The question you asked:** What if the schema differs per instance (out of sync)?

**Answer:** This is the validator version mismatch problem (P2P doc Section 5.2, Case 1). If Instance A has `ContentType:page` v1 (allows empty title) and Instance B has `ContentType:page` v2 (requires title), data created on A (empty title) passes VALIDATE on A but would fail VALIDATE on B.

**This is handled by the sync inbound validation layer** (`SanitizeIncoming` in the Security doc). When Instance B receives a Node from Instance A, it validates against B's local schema. If validation fails, B can:
1. Reject the Node (strict: breaks sync for legitimate data)
2. Accept the Node with a validation warning flag (lenient: accept-and-tag)
3. Accept the Node because it was valid when created (trust-origin: defer to A's validation)

**Verdict:** VALIDATE is pure given the same schema. The schema versioning problem is not a VALIDATE bug -- it is a sync protocol design question. The 12-primitive model correctly separates VALIDATE (the operation) from sync validation policy (the protocol layer).

**Recommendation:** Schema definitions should include a version number in their content hash. The sync handshake should compare schema versions. If schemas diverge, the sync protocol should negotiate (upgrade, fork, or flag).

---

## 2. Which Primitives Can Appear in Synced Subgraphs?

Based on the analysis above:

| Primitive | In Synced Subgraph? | Condition |
|-----------|---------------------|-----------|
| READ | Yes | Sync-with-context. Must guard null results. Should declare data dependencies. |
| WRITE | Yes | Result-only sync. The handler definition syncs; execution results sync as data. |
| TRANSFORM | Yes | Fully sync-safe. Expression language version must match. |
| BRANCH | Yes | Only with data-only conditions. Capability-checking BRANCHes need special handling. |
| ITERATE | Yes (sequential) | Parallel ITERATE with WRITE in BODY should be rejected. |
| WAIT | Conditional | Definition syncs. Execution is local. Requires execution assignment protocol. |
| GATE | Yes | Capability GATEs are local enforcement. Handler results sync; GATE outcomes do not. |
| CALL | Yes | Transitively inherits determinism of called subgraph. Requires dependency ordering. |
| RESPOND | Yes | Fully sync-safe. |
| EMIT | Conditional | Definition syncs. Execution side effects are local. Needs `deliveryMode` for non-idempotent EMITs. |
| INVOKE | Conditional | Definition syncs. Execution is local. Results sync as data (result-only). |
| VALIDATE | Yes | Sync-safe given same schema version. Schema divergence handled at protocol layer. |

**Summary:** All 12 primitives can appear in synced handler DEFINITIONS. But 4 primitives (WAIT, EMIT, INVOKE, and parallel ITERATE with WRITE) require execution-locality semantics -- the definition syncs but execution is constrained to one instance per triggering event.

---

## 3. The GATE Problem -- Deep Analysis

**Question:** A synced handler has GATE checks. Instance A has different capabilities than Instance B. The same handler may pass GATE on A but fail on B. Is this correct behavior or a sync bug?

**Answer: It is correct behavior -- GATE is local capability enforcement.** The handler's capability requirements are part of the handler's metadata (capability manifest, extracted by static analysis per P2P doc Section 4.1). When a handler syncs to Instance B, the operator on Instance B decides whether to grant the required capabilities to the module. If they do, the handler executes normally. If they do not, the handler is installed but cannot execute (GATE rejects).

This is analogous to installing a mobile app that requires camera permission. The app code syncs to the phone. The OS prompts "allow camera access?" If the user denies, the app is installed but camera features do not work.

**The deeper concern:** What happens to DATA produced by the handler on Instance A (where GATE passed) when it syncs to Instance B (where GATE would fail)?

**Scenario:**
1. Instance A has module `seo-plugin` with `store:write:seo/*` capability.
2. Handler `seo:scoreContent` contains: GATE(store:write:seo/*) -> READ(content) -> TRANSFORM -> WRITE(seo-score).
3. Handler executes on A, produces a `SeoScore` Node.
4. The `SeoScore` Node syncs to Instance B.
5. Instance B has NOT granted `store:write:seo/*` to the SEO module.
6. Does Instance B accept the `SeoScore` Node?

**Answer:** YES. The GATE applies to LOCAL handler execution, not to SYNCED DATA acceptance. The `SanitizeIncoming` layer (Security doc Section 8.2) validates Node structure (labels, properties, schema) but does NOT re-run capability checks on synced data. The data was legitimately produced on Instance A under Instance A's capability grants.

**This is the correct design.** If synced data were re-validated against the receiving instance's capability grants, most sync scenarios would break -- Instance A and Instance B will always have different capability configurations. Sync operates on DATA trust (schema validity), not EXECUTION trust (capability enforcement).

**Exception:** If Instance B explicitly blocks the `SeoScore` label in its sync agreement (via `allowedLabels` in `SanitizeIncoming`), the Node is rejected. This gives operators control over what data enters their instance.

---

## 4. EMIT Across Instances -- Deep Analysis

**Question:** If a handler EMITs a notification, and the handler syncs to another instance, does the EMIT fire on both instances?

**Answer:** The EMIT fires ONLY on the instance that executes the handler. EMIT is a local side effect.

**But the real question is: does the handler execute on both instances?**

There are three scenarios:

**Scenario A: Direct invocation.** User on Instance A triggers an API endpoint that executes the handler. Only Instance A executes. EMIT fires only on A. The handler's WRITE results sync to B. **No duplication.**

**Scenario B: Event-triggered, event originates locally.** Content is created on Instance A. `content:afterCreate` event fires on A. Handler executes on A. EMIT fires on A. The content syncs to B. Does `content:afterCreate` fire on B when the synced content arrives?

This is the **critical undecided question.** Two options:
1. **No re-fire on sync.** The event fired on the originating instance. The receiving instance does not re-fire it. Data arrives already processed. **This prevents duplication but means Instance B's event-triggered handlers never run on synced data.**
2. **Re-fire on sync.** When synced data arrives, the engine fires the same events locally so local handlers process the data. **This causes EMITs to fire on both instances.**

**The P2P doc does not explicitly decide this.** But the result-only sync model (Section 3.4) implies option 1: execution results sync, not execution triggers. If the handler's WRITEs already synced, re-executing the handler would produce duplicate data.

**Scenario C: Event-triggered, both instances receive the event independently.** Instance A and Instance B both observe the same external event (e.g., both subscribe to a webhook). Both execute the handler. Both EMIT. Both WRITE. After sync, duplicate Nodes exist (two SeoScore Nodes for the same content). CRDT merge cannot deduplicate these because they have different anchor IDs.

**Verdict:** EMIT itself is fine -- it fires locally, as designed. The problem is handler execution multiplicity, which is an EXECUTION ASSIGNMENT problem, not an EMIT problem. The 12-primitive model punts on this.

**Recommendation:** Add an `executionPolicy` metadata field to handler subgraphs:
- `origin-only`: Execute only on the instance where the triggering event originated. Other instances receive results via sync.
- `local`: Execute on every instance that has the handler installed and receives the trigger. Caller is responsible for idempotency.
- `leader-elected`: Execute on one designated instance (requires distributed coordination).

---

## 5. WAIT Across Instances -- Deep Analysis

**Question:** A handler WAITs for human approval. If the handler synced to another instance, there are now TWO wait points. Who approves? Both?

**Answer:** Under the correct execution model (origin-only), there is only ONE wait point -- on the instance where the handler was triggered. The handler definition syncs, but the execution state (the pending WAIT) does not.

**But what if the user wants approval to happen on a DIFFERENT instance?** For example: content is created on Instance A (mobile). Approval happens on Instance B (desktop). The handler is on Instance A, WAITing for a signal.

**Option 1: Signal relay.** Instance B sends a signal to Instance A (via the sync channel). Instance A's WAIT resolves. The handler continues on Instance A. This requires the sync protocol to support signal messages, not just data Nodes. The current spec does not include signal relay.

**Option 2: Distributed WAIT.** The WAIT's signal ID is published to the sync group. Any instance in the group can send the signal. The originating instance's WAIT resolves when the signal arrives (from any source). This requires:
- Signal Nodes in the sync protocol (not currently specified)
- Authentication of the signaler (can Instance B's admin approve Instance A's WAIT?)
- Conflict resolution (what if two instances send conflicting signals?)

**Option 3: Approval as data.** Instead of WAIT-for-signal, the handler WRITEs a `PendingApproval` Node and RESPONDs immediately. A separate handler watches for `Approval` Nodes (reactive subscription). When an admin creates an `Approval` Node (on any instance), it syncs to the originating instance, triggering the reactive handler. No WAIT needed -- the pattern is decomposed into two handlers connected by data sync.

**Verdict:** Option 3 is the most sync-compatible approach. WAIT with external signals should be decomposed into two handlers connected by data Nodes. This keeps all sync primitives data-centric (Nodes and Edges) rather than introducing a separate signal channel.

**Recommendation:** WAIT should be documented as "execution-local only." For cross-instance approval flows, the recommended pattern is "write intent, watch for approval data, execute on approval arrival." The 12 primitives already support this (WRITE + READ subscription + CALL).

---

## 6. Version-Stamped Results -- Where Does Tagging Happen?

**Question:** When a handler produces data via WRITE, the P2P doc proposed tagging with the handler version. Where in the 12 primitives does this tagging happen?

**Answer:** The tagging SHOULD be automatic on every WRITE, applied by the ENGINE, not by the handler.

**Current gap:** The 12-primitive WRITE definition does not mention version tagging. The P2P doc (Section 5.3) proposes `_producedBy` and `_producedAt` properties on version Nodes, but this is described as a convention, not as a WRITE primitive property.

**The correct design:**

```
WRITE { action: "create", labels: [...], data: {...} }
  Engine automatically adds:
    _producedBy: "handler:{handlerId}/v{version}"
    _producedAt: HLC_TIMESTAMP
    _instanceId: LOCAL_INSTANCE_ID
```

These properties are added by the engine during WRITE execution, not by the handler. The handler CANNOT suppress or override them (INV-5 from the Security doc: causal attribution is mandatory and immutable).

**Where in the 12 primitives:** The tagging is an ENGINE BEHAVIOR, not a primitive. It happens inside the WRITE primitive's implementation. The handler author does not need to know about it (it is transparent). The sync protocol uses these tags for conflict resolution, version-aware merge, and retroactive re-processing.

**Recommendation:** Add `_producedBy`, `_producedAt`, and `_instanceId` to the WRITE primitive's documentation as ENGINE-INJECTED metadata. Make it explicit that handlers cannot control these fields.

---

## 7. Can All 12 Primitives Be Represented as Nodes and Edges?

**Question:** Can all 12 primitives be represented as Nodes and Edges that sync via CRDT? Any primitive that requires special encoding?

### 7.1 Node Representation

Every primitive becomes a Node with a label indicating its type:

```
Node {
  labels: ["OperationNode", "READ"],
  properties: {
    target: "content/${routeParams.id}",
    mode: "node",
    projection: ["title", "content", "status"]
  }
}
```

All 12 primitives can be represented this way. Properties are typed key-value pairs. No primitive requires data types that cannot be expressed as properties.

### 7.2 Edge Representation

Control flow and data dependencies are edges:

```
Edge { type: "NEXT", from: read_node, to: transform_node }
Edge { type: "TRUE", from: branch_node, to: success_path }
Edge { type: "BODY", from: iterate_node, to: body_subgraph }
Edge { type: "DEPENDS_ON", from: write_node, to: read_node }
```

All edge types from the Systems doc (NEXT, BODY, TRUE, FALSE, MATCH:*, DEFAULT, REJECT, MERGE_FROM, DEPENDS_ON, ON_ERROR, ON_TIMEOUT) can be represented as typed edges with optional properties.

### 7.3 Special Encoding Needed

**WAIT state serialization.** A WAIT in progress has execution state (the suspended context, signal ID, timeout). This state must be serialized to the graph as a `PendingOperation` Node. The P2P doc does not specify how WAIT state Nodes interact with CRDT sync. If a `PendingOperation` Node syncs to another instance, that instance might try to resume it (incorrectly).

**Recommendation:** `PendingOperation` Nodes should be in the system zone (per Security doc INV-4) and excluded from sync by default. Only the result of a completed WAIT (the handler's final output) syncs.

**ITERATE intermediate state.** During parallel ITERATE, there are N concurrent sub-executions. Each produces intermediate results. If the ITERATE is interrupted (crash, timeout), the intermediate state exists. How does this interact with transactions? The Systems doc says "if any WRITE in a transactional subgraph fails, all preceding WRITEs are rolled back." This implies ITERATE-internal WRITEs are transactional. But parallel ITERATE with transaction semantics requires serializable isolation, which is expensive.

**Recommendation:** Parallel ITERATE should be restricted to read-only or side-effect-free BODY subgraphs in transactional contexts. If WRITE is needed in a parallel ITERATE, the transaction semantics should be per-item (each iteration is its own mini-transaction), not per-subgraph.

**CALL stack depth.** CALL creates a stack of execution frames. The stack depth is bounded (default: 20). But the stack itself is not a graph structure -- it is execution runtime state. During WAIT (which suspends execution), the entire call stack must be serialized. This is feasible but requires careful encoding.

**Recommendation:** Execution state (call stack, ITERATE cursor, BRANCH history) should be serialized as a `SuspendedExecution` Node linked to the `PendingOperation` Node via a `HAS_STATE` edge. This keeps all state in the graph. The serialization format should be specified.

### 7.4 CRDT Sync for Operation Subgraphs

Operation subgraph Nodes use the standard CRDT sync mechanism (per-field LWW for properties, add-wins for edges). But operation subgraphs have an additional constraint: **structural integrity.** A partial sync (some Nodes arrive, some do not) could produce a broken subgraph (orphan nodes, dangling edges).

**Recommendation:** Operation subgraphs should sync ATOMICALLY -- all Nodes and Edges in the subgraph arrive in a single delta batch. The content hash (P2P doc Section 11.1) verifies integrity on receipt. Partial subgraphs are rejected.

---

## 8. Confirmed Design Gaps

### GAP-1: Execution Assignment Protocol (CRITICAL)

**Impact:** Duplicate execution of handlers on multiple instances.

The 12 primitives do not specify WHO executes a handler when the triggering event is visible to multiple instances. Without this, event-triggered handlers (the most common handler type) produce duplicate WRITEs, duplicate EMITs, and duplicate INVOKE side effects.

**Fix:** Add `executionPolicy` to handler metadata. Implement origin-only execution for the default case. Provide leader-election for distributed execution scenarios.

### GAP-2: Data Dependency Declaration for READ (HIGH)

**Impact:** Handlers fail at runtime on instances missing required data.

READ produces `null` when a required Node does not exist. The 12-primitive model relies on BRANCH guards (runtime). The sync protocol should validate data dependencies (pre-execution).

**Fix:** Add `requiresData` to READ. Engine validates during sync acceptance.

### GAP-3: EMIT Delivery Semantics Across Instances (HIGH)

**Impact:** Non-idempotent side effects (emails, webhooks) fire multiple times.

EMIT has no `deliveryMode`. All EMITs are fire-and-forget local side effects. For non-idempotent operations, this causes real-world duplication.

**Fix:** Add `deliveryMode: "local" | "exactly-once" | "broadcast"` to EMIT.

### GAP-4: Parallel ITERATE + WRITE Safety (MEDIUM)

**Impact:** Non-deterministic write ordering in synced handlers.

Parallel ITERATE with WRITE operations in the BODY produces different write orderings on different instances.

**Fix:** Structural validation rule: reject parallel ITERATE with WRITE in BODY for synced subgraphs.

### GAP-5: Atomic Subgraph Sync (MEDIUM)

**Impact:** Partial subgraph sync produces broken handlers.

Standard CRDT sync may deliver Nodes incrementally. A partially synced operation subgraph is structurally invalid.

**Fix:** Operation subgraphs sync as atomic batches. Content hash verifies integrity.

### GAP-6: Expression Language Versioning (LOW)

**Impact:** TRANSFORM produces different results on different engine versions.

Expression language semantics may change between engine versions. Content hash does not capture expression language version.

**Fix:** Include expression language version in subgraph content hash.

---

## 9. Recommendations Summary

| Priority | Recommendation | Affects |
|----------|---------------|---------|
| CRITICAL | Define execution assignment protocol for event-triggered handlers | WRITE, EMIT, INVOKE, WAIT |
| HIGH | Add `requiresData` to READ for dependency declaration | READ, sync protocol |
| HIGH | Add `deliveryMode` to EMIT | EMIT |
| HIGH | Reconcile the four vocab documents into one canonical 12-primitive spec | All primitives |
| MEDIUM | Reject parallel ITERATE + WRITE in synced subgraphs | ITERATE structural validation |
| MEDIUM | Atomic subgraph sync with content hash verification | Sync protocol |
| MEDIUM | Specify WAIT state serialization format | WAIT |
| LOW | Include expression language version in content hash | TRANSFORM, BRANCH |
| LOW | Document CALL dependency ordering in sync protocol | CALL |

---

## 10. What the 12 Primitives Get RIGHT

Despite the gaps, the 12-primitive consolidation is sound:

1. **The set IS irreducible.** Each primitive covers a distinct capability. None can be composed from the others without significant loss (the Systems doc's "Rejected" section is convincing).

2. **The determinism spectrum maps cleanly to sync safety.** Pure (TRANSFORM, VALIDATE, BRANCH, RESPOND), Read-deterministic (READ, GATE in validate/condition mode), Write-deterministic (WRITE, EMIT), and Non-deterministic (WAIT, INVOKE) align with the P2P doc's sync-safe/sync-with-context/sync-result-only/sync-forbidden classification.

3. **The separation between handler DEFINITION sync and handler EXECUTION sync is the right model.** All 12 primitives can appear in synced handler definitions. The execution model (which instances execute, how results propagate) is orthogonal to the primitive vocabulary.

4. **Content-addressing works.** All 12 primitives can be hashed (their properties are deterministic value types). Subgraph hashing provides integrity verification for sync.

5. **The non-Turing-complete constraint holds.** ITERATE has `maxIterations`. CALL has `timeout` and `depth`. WAIT has `timeout`. The DAG structure prevents cycles. No primitive introduces unbounded computation.

The gaps are in the PROTOCOL layer (execution assignment, delivery semantics, state serialization), not in the PRIMITIVE layer. The 12 primitives are a solid instruction set. The execution engine around them needs the additional specifications identified in Section 8.
