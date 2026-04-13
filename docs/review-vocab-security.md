# Security Review: 12 Operation Primitives

**Reviewer:** Security & Trust Auditor
**Date:** 2026-04-11
**Scope:** Synthesized 12-primitive operation vocabulary with 10 structural invariants
**Input documents:** `operation-vocab-systems.md` (10 primitives), `operation-vocab-security.md` (20 operations), `operation-vocab-dx.md` (18 types), `operation-vocab-p2p.md` (determinism classification), `critique-security.md`, `SPECIFICATION.md`
**Verdict:** The synthesis is defensible. The 12 primitives cover the attack surface without redundancy. The 10 structural invariants are the right invariants. But 4 of the 10 attack scenarios reveal real gaps that need design-time mitigation before implementation begins.

---

## Security Score: 8 / 10

**Rationale:** Up from 5/10 on the raw specification. The synthesis addresses CRITICAL-1 (Cypher injection -- READ/WRITE are structured, not raw Cypher), CRITICAL-2 (system zone -- invariant 7 makes CapabilityGrant unreachable), and partially addresses CRITICAL-3 (sync -- VALIDATE + structural validation at registration time). The remaining 2 points are lost on: (a) the resource exhaustion combinatorics of ITERATE x depth, and (b) the WASM output-to-graph boundary at INVOKE, both of which have mitigations available but not yet specified.

---

## The 12 Primitives and Their Security Profile

| # | Primitive | Category | Mutates Graph? | Crosses Trust Boundary? | Non-Deterministic? |
|---|-----------|----------|---------------|------------------------|-------------------|
| 1 | READ | Data access | No | No | Read-deterministic |
| 2 | WRITE | Data access | **Yes** | No | Write-deterministic |
| 3 | TRANSFORM | Computation | No | No | Pure |
| 4 | BRANCH | Control flow | No | No | Pure |
| 5 | ITERATE | Control flow | No | No | Pure (wrapper) |
| 6 | WAIT | Control flow | **Implicit** (serializes state) | **Yes** (external signal) | Non-deterministic |
| 7 | GATE | Enforcement | No (or transform) | No | Pure |
| 8 | CALL | Composition | No (delegates) | Partial (attenuated scope) | Depends on callee |
| 9 | RESPOND | Output | No | **Yes** (exits to channel) | Pure |
| 10 | EMIT | Output | No | **Yes** (fire-and-forget to reactive) | Write-deterministic |
| 11 | INVOKE | External | No (return value only) | **Yes** (WASM sandbox) | Non-deterministic |
| 12 | VALIDATE | Enforcement | No | No | Pure |

**Security-critical primitives (ranked by attack surface):**
1. **WRITE** -- the only primitive that mutates user-zone graph state
2. **INVOKE** -- the only primitive that executes Turing-complete code
3. **WAIT** -- the only primitive that introduces non-determinism into the otherwise total graph
4. **ITERATE** -- the only primitive with a multiplication effect on execution cost
5. **CALL** -- the only primitive that chains capability scopes

---

## Attack Scenario Analysis

### Attack 1: Resource Exhaustion via ITERATE x Depth

**The attack:** A module registers a subgraph with ITERATE at multiple depth levels. At depth 0: ITERATE maxIterations=10000. Inside the ITERATE body: CALL to a subgraph that itself contains ITERATE maxIterations=10000. Repeat for 8 levels of INVOKE nesting (invariant 4). Theoretical maximum: 10000^8 = 10^32 operations. Even with the depth cap of 64 (invariant 2) and fan-out cap of 16 (invariant 3), the multiplicative effect of nested iteration is the dominant cost factor.

**Feasibility: HIGH.** This is not theoretical. The systems-perspective document explicitly sets maxIterations ceiling at 10000 (operator-configurable). The INVOKE nesting limit is 8. Even a modest 1000 x 1000 x 1000 = 10^9 operation execution would take minutes on any hardware. A malicious community module could register this subgraph with legitimate-looking structure (each individual ITERATE has a "reasonable" bound).

**Evaluation:** The pre-computed cost model (security-perspective INV-3) is the intended mitigation. The engine computes total cost before execution begins. But the cost computation itself must handle the multiplicative case correctly.

**The real question is whether the cost model multiplies through CALL and ITERATE boundaries.** If the cost model sums costs (treating CALL as a single operation with its own cost), it underestimates. If it multiplies (ITERATE cost = maxIterations x body cost), it computes correctly but may itself be expensive for deeply nested structures.

**Mitigations required:**

1. **Total operation budget per subgraph execution (not per-node).** The pre-computed cost must be a single number: `totalOps = product of all ITERATE maxIterations on any path from entry to terminal`. This number is computed at registration time. If it exceeds the platform ceiling (e.g., 10^7 operations), the subgraph is rejected at registration, not at execution. This is O(|nodes|) to compute via topological traversal.

2. **Cumulative iteration budget.** In addition to per-ITERATE maxIterations, enforce a cumulative iteration counter across all ITERATEs in a single execution. When the counter hits the platform ceiling, execution aborts. This is the runtime defense-in-depth for any cost model bug.

3. **CALL cost must include callee cost.** When computing subgraph cost, CALL nodes must recursively include the callee subgraph's total cost multiplied by the number of times the CALL can execute (1 for sequential, maxIterations for inside ITERATE). The INVOKE nesting limit (8) bounds this recursion.

**Verdict: Feasible attack, mitigable. CRIT -- must be addressed in the cost model specification before implementation.**

---

### Attack 2: Capability Escalation via CALL Chain

**The attack:** Module A has capabilities {read:content/*, write:content/*}. Module A's subgraph calls Module B's subgraph via CALL with `isolated=true`. Module B's subgraph calls Module C's subgraph. At each hop, can the capability envelope somehow widen?

**Feasibility: LOW.** The attenuation model is well-designed. CALL with `isolated=true` creates an intersection of the caller's envelope and the callee's own grants. The security-perspective document (INV-1) states attenuation is irreversible within an execution path. The key property: `callee_effective = caller_envelope INTERSECT callee_grants`. Since intersection can only shrink or preserve, never widen, escalation is structurally impossible.

**Edge case 1: CALL with `isolated=false`.** The systems-perspective document defaults `isolated` to `true`. But if a subgraph explicitly sets `isolated=false`, the callee inherits the caller's full envelope without intersection with its own grants. This means a callee could exercise capabilities it was not directly granted, as long as the caller has them. This is by design (trusted composition), but it creates a confused-deputy risk: Module A calls Module B with `isolated=false`, and Module B's logic (which may have been written assuming its own limited grants) now operates with Module A's broader capabilities.

**Edge case 2: Capability grant changes during execution.** If a capability is revoked while a subgraph is executing, does the revocation take effect mid-execution? The security document says the envelope is snapshotted at execution start (MVCC). This means a revocation during execution does NOT affect the running subgraph. This is correct for consistency but creates a window where revoked capabilities are still exercisable.

**Mitigations required:**

1. **Default `isolated=true` is correct and must be non-overridable for community-tier modules.** Only platform and verified modules should be able to set `isolated=false`. The registration-time validator must reject community-tier subgraphs that use `isolated=false`.

2. **Revocation window is acceptable** given MVCC semantics, but the CausalRecord must record the capability envelope snapshot at execution start, so post-hoc audit can identify executions that used subsequently-revoked capabilities.

**Verdict: Low feasibility for direct escalation. The `isolated=false` edge case needs tier-gating. HIGH -- not a vulnerability but a design constraint that must be enforced.**

---

### Attack 3: Data Exfiltration via READ + EMIT

**The attack:** A module with `read:content/*` capability reads sensitive content data via READ, then uses EMIT to fire an event containing that data. A co-conspirator module (or a module the attacker also controls) has an event handler that receives the EMIT and writes the data to an attacker-controlled location, or sends it via INVOKE (WASM HTTP call).

**Feasibility: MEDIUM.** This is the classic covert channel problem. The data flows are: READ (capability-checked) -> TRANSFORM (shapes data) -> EMIT (fire-and-forget, capability for the event namespace required). The question is whether EMIT checks that the payload does not contain data the receiver is not authorized to see.

**Analysis:** EMIT produces a notification into the reactive system. The reactive system delivers the event to all subscribers. If subscriber B has `read:seo/*` but not `read:content/*`, and the EMIT payload contains content data, B receives data it could not have read directly.

This is a fundamental tension: EMIT is fire-and-forget for performance, but capability-checking every subscriber's envelope against the payload would be O(subscribers x payload_fields) on every emit.

**Mitigations required:**

1. **EMIT namespace scoping.** The emitting module can only EMIT to event namespaces within its capability scope. A module with `read:content/*` can emit to `content:*` events. A module with `read:seo/*` can subscribe to `seo:*` events. Cross-namespace delivery requires the subscriber to have a `subscribesTo` declaration that is validated at registration time (this mirrors the existing Thrum V3-4.5 `subscribesTo` pattern).

2. **Payload capability tagging.** Each EMIT payload is tagged with the minimum capability required to receive it (derived from the READ operations that produced the data). Subscribers without the required capability receive a redacted payload or no delivery. This is the principled solution but has performance cost.

3. **Pragmatic alternative: accept the covert channel risk for same-operator modules.** In a single-operator deployment (one person runs the instance), all installed modules are operator-chosen. The covert channel is only a risk when modules from different trust domains run on the same instance. For the initial release, document this as a known limitation and enforce strict namespace isolation for community-tier modules.

**Verdict: Medium feasibility. The covert channel is real but bounded by namespace isolation. HIGH -- needs explicit design decision on payload tagging vs. accepted risk.**

---

### Attack 4: System Zone Breach via Operation Combination

**The attack:** Can any combination of READ, WRITE, CALL, INVOKE produce a path to CapabilityGrant nodes despite invariant 7 ("system-zone labels unreachable from user operations")?

**Feasibility: LOW (if implemented correctly).** Invariant 7 states that system-zone labels are unreachable from user operations. The security-perspective document defines the system zone as: CapabilityGrant, OperationDef, Anchor, VersionMeta, IVM definition nodes. These are prefixed with `__benten:` and the query planner rejects traversals that would cross the zone boundary.

**Attack vector 1: READ with a traversal pattern that follows edges into the system zone.** If a user-zone Node has an edge to an Anchor Node (which is system-zone), a READ with `maxDepth=2` might traverse: user Node -> edge -> Anchor Node. The defense is that the traversal stops at the zone boundary. But this requires the traversal engine to check every visited node's labels against the zone classifier. If the zone classifier has a bug (e.g., a label that was supposed to be system-zone but is not in the blocklist), the traversal leaks.

**Attack vector 2: INVOKE returning system-zone data.** If a WASM sandbox has a host function that reads the graph, and the host function's capability membrane has a bug that allows reading system-zone nodes, the sandbox could exfiltrate system data via its return value.

**Attack vector 3: Content-addressed hash collision.** Invariant 6 says subgraphs are content-addressed. If an attacker can produce a subgraph with the same hash as a system-zone operation subgraph, and the engine uses the hash as a lookup key, the attacker's subgraph could be executed with system-zone privileges. This requires a hash collision (SHA-256: 2^128 operations for a birthday attack), which is infeasible with current technology.

**Mitigations required:**

1. **Zone enforcement at the storage layer, not the query layer.** The graph storage engine must have a two-partition architecture: system-zone nodes in one partition, user-zone nodes in another. Traversal never crosses partitions. This is defense-in-depth beyond label checking.

2. **Host function allowlist for INVOKE.** The host functions exposed to WASM sandboxes must be a hardcoded allowlist, not derived from capabilities. Even if a capability check passes, a host function that reads system-zone data must not exist in the sandbox API.

3. **Hash algorithm must be collision-resistant.** SHA-256 or BLAKE3. Not MD5, not SHA-1.

**Verdict: Low feasibility given correct implementation. The storage-layer partitioning is the key defense. MEDIUM -- design constraint, not a vulnerability.**

---

### Attack 5: WASM Output Manipulation (INVOKE Escape)

**The attack:** INVOKE executes Turing-complete code in a WASM sandbox. The sandbox returns a value. That value flows into subsequent graph operations (WRITE, TRANSFORM, BRANCH). Can the sandbox craft its return value to manipulate subsequent operations?

**Feasibility: MEDIUM.** The sandbox cannot call back into the graph (no re-entrancy -- correctly designed, lessons from the DAO hack). But its return value IS data that the graph trusts. Specific vectors:

**Vector 1: Return value triggers a BRANCH that leads to a privileged path.** If a BRANCH condition evaluates `$.invokeResult.isAdmin == true`, a malicious INVOKE could return `{isAdmin: true}`. The defense: BRANCH conditions should never evaluate unvalidated INVOKE output for authorization decisions. Authorization is GATE's job, not BRANCH's.

**Vector 2: Return value contains oversized data.** The sandbox returns a 1GB JSON string as its result. The TRANSFORM that processes it consumes unbounded memory. The defense: the INVOKE node should enforce a maxOutputSize limit on the return value.

**Vector 3: Return value contains property names that collide with internal engine fields.** If the graph engine uses `__benten_*` prefixed properties internally, and the sandbox returns `{__benten_capability: "admin"}`, a downstream WRITE might store this as if it were a system property. The defense: the engine must strip or reject reserved-prefix properties from INVOKE return values before they enter the data flow.

**Vector 4: Return value contains crafted expression strings.** If the return value flows into a TRANSFORM expression that evaluates it (second-order injection), the sandbox could inject malicious expressions. The defense: TRANSFORM expressions are pre-compiled at registration time, not constructed from runtime data. This is invariant 9 (immutable once registered).

**Mitigations required:**

1. **maxOutputSize on INVOKE.** Mandatory property. Default: 1MB. Operator-configurable ceiling: 100MB. Return values exceeding the limit produce `INVOKE_OUTPUT_EXCEEDED`.

2. **Reserved property prefix stripping.** The engine must strip all `__benten:*` prefixed properties from any Value that enters the user zone from an external source (INVOKE return, WAIT signal, sync payload). This is a one-line filter applied at three boundaries.

3. **VALIDATE after INVOKE.** The registration-time structural validator should warn (or require) that every INVOKE node is followed by a VALIDATE node before any WRITE node. This is the "never trust user input" principle applied to sandbox output.

4. **Expressions are never constructed from runtime data.** TRANSFORM expressions are ASTs compiled at registration time. Runtime data is bound to expression variables, not interpolated into expression strings. This is invariant 9 applied to expressions.

**Verdict: Medium feasibility. Multiple vectors, all mitigable. CRIT -- maxOutputSize and reserved prefix stripping must be in the specification.**

---

### Attack 6: Timing Side Channels via BRANCH + WAIT

**The attack:** A module uses BRANCH to test a condition (e.g., "does user X have admin capability?" via GATE mode=capability with a REJECT path that does not abort). Based on the result, it takes a fast path or a slow path (WAIT with different timeouts). An external observer measures response time to infer the BRANCH result.

**Feasibility: LOW for capability inference.** The `ReadCapability` operation only returns the caller's own capabilities, not other entities'. A module cannot directly test another user's capabilities. The only way to infer another user's capabilities is to attempt an action on their behalf and observe whether it succeeds, which requires the confused-deputy pattern from Attack 2.

**Feasibility: MEDIUM for data inference.** A module reads data via READ, uses BRANCH to test a property ("is this user's balance > $10000?"), and takes a measurably different execution path. The timing difference reveals the property value without the module explicitly returning it.

**Analysis:** Timing side channels are a known limitation of any system that allows conditional execution. Complete mitigation requires constant-time execution for all paths, which is incompatible with practical computation. This is an accepted risk in virtually all application-level security models (databases, web servers, APIs).

**Mitigations available but not recommended:**

1. **Constant-time BRANCH** (pad execution to the slowest path). Impractical -- negates the performance benefit of branching.

2. **Noise injection** (add random delay to all RESPOND operations). Reduces signal-to-noise ratio but does not eliminate the channel.

3. **Audit detection.** CausalRecords capture which BRANCH path was taken. If a module exhibits a pattern of branching on sensitive data and producing timing-variable responses, the audit system can flag it.

**Verdict: Accepted risk. Timing side channels are inherent to conditional execution. Document as a known limitation. MEDIUM -- audit-based detection is sufficient.**

---

### Attack 7: Version Chain Manipulation via WRITE

**The attack:** Can a WRITE operation affect version chain pointers (Anchor -> CURRENT -> Version) despite system-zone protection?

**Feasibility: VERY LOW.** Version chain management is handled by the engine implicitly when processing WRITE operations. A user-zone WRITE says "update Node X with properties Y." The engine internally: (1) creates a new Version Node, (2) copies properties, (3) updates the NEXT_VERSION edge, (4) moves the CURRENT pointer. Steps 1-4 are system-zone operations performed by the engine, not by the user's WRITE.

**The only attack vector:** A WRITE that directly specifies a system-zone Node ID as its target. This is blocked by the zone enforcement in Attack 4's analysis. The WRITE validator rejects any mutation targeting a system-zone Node.

**Edge case: Version number injection.** If WRITE accepts an `expectedVersion` for optimistic concurrency, can a malicious module supply a fabricated version number to cause a legitimate concurrent write to fail? Yes -- but this is denial-of-service, not data corruption. The concurrent writer gets `VERSION_CONFLICT` and retries. The attacker gains nothing except nuisance.

**Mitigations:** Already handled by zone enforcement (invariant 7). No additional mitigations needed.

**Verdict: Very low feasibility. Zone enforcement is sufficient. LOW.**

---

### Attack 8: Sync Poisoning (Malicious Remote Subgraph)

**The attack:** A remote peer sends an operation subgraph via sync that passes structural validation (DAG property, depth/fan-out limits, etc.) but behaves maliciously when executed.

**Feasibility: MEDIUM.** Structural validation ensures the subgraph terminates and has bounded cost. But "structurally valid" does not mean "semantically safe." Examples:

**Vector 1: A subgraph that reads all content and emits it to a public event namespace.** Structurally valid (READ -> EMIT is legal). Semantically: data exfiltration. Defense: capability enforcement. The subgraph's GATE/CheckCapability nodes are validated against the syncing peer's capability grants. If the peer does not have `read:content/*`, the subgraph cannot include a READ of content data.

**Vector 2: A subgraph that writes misleading data.** Structurally valid (VALIDATE -> WRITE is legal). Semantically: content pollution. Defense: the VALIDATE node references a schema definition Node. The schema must already exist in the local graph. If the remote peer sends a subgraph that references a schema the local instance does not have, the VALIDATE fails at execution time.

**Vector 3: A subgraph that includes an INVOKE node pointing to a non-existent runtime.** Structurally valid. At execution time, the INVOKE fails with `SANDBOX_NOT_FOUND`. This is a nuisance, not a security issue.

**Vector 4: A subgraph with a GATE that references a capability the local instance does not recognize.** The GATE fails closed (capability not held). Execution aborts. Safe.

**The deeper issue: should synced operation subgraphs be executable at all?** The P2P-perspective document describes "installing a module is syncing a subgraph." This means operation subgraphs arrive from untrusted peers and are expected to be executed. The structural validation (10 invariants) plus capability enforcement (GATE at entry) are the two layers of defense. Both must hold.

**Mitigations required:**

1. **Synced subgraphs are quarantined until operator approval.** The engine stores the subgraph but does not make it executable until the operator (human or automated policy) reviews and approves it. This is the "app store" model. Approval can be automated for verified publishers (UCAN chain from a trusted root).

2. **Subgraph content-addressed hash verification.** The peer must declare the hash of the subgraph it is sending. The receiving instance computes the hash of the received subgraph and compares. Mismatch = rejected. This prevents MITM modification during sync.

3. **Registration-time re-validation on the receiver.** Even if the sender says "this subgraph is valid," the receiver runs all 10 structural invariant checks locally. Never trust remote validation.

**Verdict: Medium feasibility, well-mitigated by quarantine + local re-validation. HIGH -- quarantine model must be specified.**

---

### Attack 9: TRANSFORM Expression Injection

**The attack:** Can TRANSFORM expressions access anything outside their input data? Can they be used to read graph state, invoke side effects, or access prototype chains?

**Feasibility: VERY LOW.** TRANSFORM uses the same sandboxed expression evaluator as `@benten/expressions` (the Thrum TypeScript package, to be ported to Rust). The security properties of this evaluator are well-established:

1. **No graph access.** Expressions operate on a Value (JSON-like data). There is no `$graph`, no `$store`, no `$db` variable.

2. **No side effects.** No assignment operators. No function calls (only built-in functions like `len()`, `now()`, `join()`). No `eval()`.

3. **No prototype access.** `toString`, `valueOf`, `__proto__`, `constructor` are blocked. Property access uses `safeGet()` which checks against a blocklist.

4. **No computed property keys.** `$[variableName]` is rejected. Only literal property paths are allowed.

5. **Step limit.** The expression evaluator has a hard step limit (the systems document says 10,000 operations). Expressions exceeding this fail with `EXPRESSION_LIMIT`.

6. **AST compiled at registration time.** Expressions are parsed to an AST when the subgraph is registered (invariant 8). At runtime, the engine walks the pre-compiled AST with bound variables. There is no string-to-code conversion at runtime.

**The remaining risk:** Built-in functions. If the engine adds a built-in function that has side effects (e.g., `httpGet()`, `readFile()`), it would break the pure-function invariant. The built-in function set must be audited and frozen.

**Mitigations:** Already handled by the expression evaluator's design. The Rust port must preserve all 6 properties above. The built-in function allowlist must be explicitly defined and reviewed.

**Verdict: Very low feasibility. The expression sandbox is well-designed. LOW.**

---

### Attack 10: DAG Path Explosion (Depth Bomb)

**The attack:** A DAG with depth 64 (invariant 2) and fan-out 16 (invariant 3) at each level has 16^64 potential paths. This is approximately 10^77 -- far more than atoms in the observable universe. Can an attacker exploit this?

**Feasibility: VERY LOW for execution, but needs clarification for analysis.**

**Why it does not matter for execution:** The engine does not enumerate all paths. It walks the DAG from entry to terminal, following one execution path determined by BRANCH conditions. The number of nodes visited on any single execution is at most 64 (depth limit) x 16 (fan-out per node, if parallel) = 1024 nodes. This is bounded. The 16^64 number is the total number of POSSIBLE paths, but only ONE path is taken per execution (or a small number for parallel fan-out that converges at Merge points).

**Where it DOES matter:**

1. **Cost pre-computation.** If the cost model considers the worst-case path (as it should for budget enforcement), and the worst-case includes fan-out, the pre-computed cost could be maxIterations^depth for the multiplicative case. This is Attack 1 territory, not Attack 10 territory. Fan-out without ITERATE does not multiply cost -- it selects one of N paths (BRANCH) or executes N paths that converge (parallel CALL + Merge).

2. **Registration-time structural validation.** Invariant 8 says subgraphs are validated at registration time. The validator must traverse the entire subgraph to check invariants 1-7. For a subgraph with 64 x 16 = 1024 nodes, this is fast. But if the subgraph has 16^64 edges (connecting every node at level K to every node at level K+1), the validator would need to process 16^64 edges. **This is the actual attack surface.**

**The defense:** The total number of NODES in a subgraph must be bounded (not just depth and fan-out). A subgraph with depth 64 and fan-out 16 has at most 64 x 16 = 1024 nodes (if fan-out is per-level, not cumulative). If fan-out is cumulative (each of the 16 children at level 1 has 16 children at level 2), the total nodes are 16^64. This must be clarified.

**Mitigations required:**

1. **Total node count limit.** Add an 11th invariant: maximum total nodes per subgraph (e.g., 4096). This bounds the validator's work and prevents the fan-out explosion. With depth 64 and total nodes 4096, average fan-out is ~1.1, which is realistic for operation subgraphs.

2. **Total edge count limit.** Edges should also be bounded (e.g., 8192, or 2x the node limit). This prevents dense graphs with many edges between a small number of nodes.

3. **Clarify fan-out semantics.** Invariant 3 says "max fan-out: 16." This must mean "each node has at most 16 outgoing execution edges," NOT "each level has at most 16 nodes." With per-node fan-out of 16 and depth of 64, the maximum total nodes without iteration is 16 + 16^2 + ... + 16^64 (geometric series), which is astronomical. The total node count limit prevents this.

**Verdict: Very low feasibility for execution (only one path is taken). The registration-time validator needs a total node/edge count bound. MEDIUM -- add the 11th invariant.**

---

## OWASP Top 10 2025 Checklist

| # | Category | Status | Notes |
|---|----------|--------|-------|
| A01 | Broken Access Control | **STRONG** | Capability enforcement at every operation boundary (GATE, CheckCapability). Attenuation-only model prevents escalation. System zone unreachable from user operations. |
| A02 | Security Misconfiguration | **MODERATE** | Defaults are secure (isolated=true, maxIterations required, fail-closed). Risk: operator misconfigures budget ceilings or disables quarantine for synced subgraphs. |
| A03 | Software Supply Chain | **STRONG** | Content-addressed hashing (invariant 6) enables verification. Synced subgraphs are quarantined. UCAN chain proves provenance. Immutable once registered (invariant 9). |
| A04 | Cryptographic Failures | **NEEDS SPEC** | UCAN-compatible structure is correct, but key management, rotation, and revocation propagation are unspecified. The P2P critique (CRITICAL-3) identified HLC clock manipulation. |
| A05 | Injection | **STRONG** | READ/WRITE are structured (not raw Cypher). TRANSFORM expressions are sandboxed (no eval, no prototype access). INVOKE sandbox has capability membrane. No string-to-code conversion at runtime. |
| A06 | Insecure Design | **STRONG** | Deliberately not Turing complete. Bounded execution. Pre-computed cost. Attenuation-only capabilities. Immutable subgraphs. These are design-level security decisions. |
| A07 | Authentication Failures | **MODERATE** | RequireIdentity operation exists. Agent-on-behalf-of-user model prevents confused deputy. Risk: the spec does not detail session management, token expiry, or credential storage. |
| A08 | Software and Data Integrity | **STRONG** | Content-addressed hashing. MVCC prevents dirty reads. Optimistic concurrency (expectedVersions) prevents lost updates. ConditionalWrite prevents TOCTOU. |
| A09 | Logging and Monitoring | **STRONG** | Causal attribution (invariant 10) produces unsuppressible audit records. CausalRecord tree captures every operation, BRANCH path, WRITE mutation, and INVOKE result. |
| A10 | Exceptional Condition Handling | **MODERATE** | ON_ERROR edges handle failures. Transactional subgraphs roll back on any failure. Risk: WAIT timeout handling could leave orphaned state if the serialized context is not cleaned up. |

---

## Plugin/Module Trust Analysis

**What can a malicious community-tier module do?**

| Attack | Possible? | Why/Why Not |
|--------|-----------|-------------|
| Read data outside its scope | No | GATE/CheckCapability at subgraph entry. READ enforces scope. |
| Write data outside its scope | No | WRITE enforces capability scope. |
| Escalate its own capabilities | No | Attenuation-only model (INV-1). |
| Modify its own subgraph at runtime | No | INV-2: no self-modification. |
| Modify other modules' subgraphs | No | System zone, dedicated API only. |
| Exhaust resources | **Bounded** | Pre-computed cost model + cumulative budget. But see Attack 1 -- the cost model must multiply through ITERATE. |
| Exfiltrate data via EMIT | **Possible (bounded)** | See Attack 3. Namespace isolation limits delivery, but the covert channel exists. |
| Time side-channel inference | **Possible** | See Attack 6. Inherent to conditional execution. |
| Inject via INVOKE return value | **Possible (mitigable)** | See Attack 5. maxOutputSize + reserved prefix stripping needed. |
| Crash the engine | No | Rust memory safety. WASM isolation. |
| Corrupt the graph | No | System zone + MVCC + transactions. |

**Trust tier enforcement at registration time:**

| Constraint | Community | Verified | Platform |
|------------|-----------|----------|----------|
| `isolated` on CALL | Must be `true` | Can be `false` | Can be `false` |
| INVOKE gas budget | Capped at operator-set ceiling | Higher ceiling | Unlimited |
| EMIT namespaces | Own namespace only | Own + declared subscriptions | All |
| maxIterations ceiling | Lower (e.g., 1000) | Higher (e.g., 10000) | Operator-configurable |
| Subgraph node count | Lower (e.g., 256) | Higher (e.g., 1024) | 4096 |
| Quarantine on sync | Always quarantined | Auto-approved if UCAN chain valid | N/A (local) |

---

## Structural Invariant Audit

| # | Invariant | Sound? | Gap |
|---|-----------|--------|-----|
| 1 | Subgraphs are DAGs | Yes | Topological sort at registration time. Correct. |
| 2 | Max depth: 64 | Yes | Bounds single-path execution. Correct. |
| 3 | Max fan-out: 16 | **Needs clarification** | Per-node fan-out is fine. But without a total node count limit, cumulative fan-out is unbounded. Add invariant 11. |
| 4 | Max INVOKE nesting: 8 | Yes | Bounds sandbox-in-sandbox chains. Correct. |
| 5 | Determinism classification | Yes | Per the P2P document's classification matrix. Critical for sync safety. |
| 6 | Content-addressed hashing | Yes | Prevents MITM and enables deduplication. Use SHA-256 or BLAKE3. |
| 7 | System zone unreachable | **Critical** | Must be enforced at storage layer, not just query layer. Two-partition architecture recommended. |
| 8 | Registration-time validation | Yes | All structural checks before any execution. Correct. |
| 9 | Immutable once registered | Yes | New version for changes, old version preserved. Correct. |
| 10 | Causal attribution | Yes | Unsuppressible. System zone. Correct. |

**Recommended additions:**

| # | New Invariant | Rationale |
|---|--------------|-----------|
| 11 | Max total nodes per subgraph: 4096 | Prevents registration-time validator explosion from cumulative fan-out |
| 12 | Max total edges per subgraph: 8192 | Prevents dense graphs that slow validation |
| 13 | Max INVOKE output size: 1MB default | Prevents memory exhaustion from WASM return values |
| 14 | Cumulative iteration budget per execution | Runtime defense-in-depth against cost model bugs |

---

## Consolidated Findings by Severity

### CRITICAL (must address before implementation)

1. **Attack 1: ITERATE x Depth cost model.** The pre-computed cost model must multiply through ITERATE and CALL boundaries. Without this, a module can construct a subgraph with 10^9+ effective operations while each individual ITERATE has a "reasonable" maxIterations. Add cumulative iteration budget as runtime defense-in-depth.

2. **Attack 5: INVOKE output boundary.** WASM sandbox return values enter the graph data flow without size limits, reserved-prefix filtering, or mandatory validation. Add maxOutputSize, strip `__benten:*` prefixes, and recommend (or require) VALIDATE after INVOKE.

### HIGH (must address during design phase)

3. **Attack 3: EMIT covert channel.** Namespace isolation is necessary but not sufficient. Make an explicit design decision on payload capability tagging vs. accepted risk. Document either way.

4. **Attack 8: Sync quarantine model.** Synced operation subgraphs must not be auto-executable. Specify quarantine, operator approval, and UCAN-based auto-approval for verified publishers.

5. **Attack 2: `isolated=false` tier restriction.** Community-tier modules must not be able to set `isolated=false` on CALL. Enforce at registration time.

6. **Attack 10: Total node/edge count invariant.** Without it, the fan-out invariant does not bound the total subgraph size. Add invariants 11 and 12.

### MEDIUM (should address, not blocking)

7. **Attack 4: Storage-layer partitioning.** System zone enforcement should be at the storage layer, not just the query layer. Defense in depth.

8. **Attack 6: Timing side channels.** Document as known limitation. Add audit-based detection patterns.

9. **OWASP A04: Cryptographic spec.** UCAN key management, rotation, and revocation propagation are unspecified. Not a vocabulary issue but a system-level gap.

10. **OWASP A10: WAIT state cleanup.** Timed-out WAITs must clean up their serialized context. Specify garbage collection for orphaned WAIT states.

### LOW (no action needed)

11. **Attack 7: Version chain manipulation.** Zone enforcement handles this.
12. **Attack 9: TRANSFORM injection.** Expression sandbox is well-designed.

---

## Primitive Completeness Assessment

**Is anything missing from the 12?** Comparing against the four perspective documents:

| Security doc operation | Covered by 12? | How |
|----------------------|----------------|-----|
| CheckCapability | Yes | GATE mode=capability |
| AttenuateCapability | Yes | CALL with isolated=true + capability narrowing |
| RequireIdentity | Partial | GATE mode=condition on identity. Consider whether identity checks deserve a dedicated GATE mode. |
| EnforceBudget | **Missing** | Budget enforcement is an engine concern (INV-3), not a primitive. Correct to omit if the engine enforces budgets transparently. If modules should be able to set per-section budgets, it needs to be a GATE mode or a CALL property. |
| ReadGraph | Yes | READ |
| ReadView | Yes | READ mode=view |
| ReadCapability | Partial | GATE mode=capability (non-failing check, returns boolean). Or TRANSFORM with a built-in `hasCapability()` function. |
| WriteGraph | Yes | WRITE |
| ConditionalWrite | Yes | GATE + WRITE in sequence (atomic within transaction) |
| Invoke | Yes | INVOKE |
| InvokeSubgraph | Yes | CALL |
| Branch | Yes | BRANCH |
| Merge | **Implicit** | Not a separate primitive. Multiple CALL fan-out + convergence at NEXT edge after all complete. May need explicit specification for `awaitAll` vs `awaitAny` semantics. |
| Transform | Yes | TRANSFORM |
| ValidateSchema | Yes | VALIDATE |
| ValidateRelationship | Partial | VALIDATE with a relationship-specific schema. Or a GATE mode. |
| AuditPoint | **Implicit** | Covered by invariant 10 (automatic causal attribution). Explicit audit points could be an EMIT to a reserved `audit:*` namespace. |
| DryRun | **Missing as primitive** | Engine-level concern. The engine wraps any subgraph in dry-run mode. Not a primitive -- correct to omit. |
| ValidateRemoteOrigin | **Missing as primitive** | Sync-level concern. Handled before the operation subgraph executes. Correct to omit from the operation vocabulary. |
| SanitizeIncoming | **Missing as primitive** | Same as above -- sync layer, not operation layer. |

**Assessment:** The 12 primitives are complete for the operation vocabulary. Budget enforcement, dry-run, sync validation, and merge semantics are correctly handled at the engine layer rather than as operation primitives. The only consideration is whether Merge (parallel convergence) needs to be explicit if CALL fan-out is supported. If the engine automatically waits for all parallel CALLs before continuing, Merge is implicit. If the module developer needs to choose `awaitAll` vs `awaitAny`, Merge needs to be explicit or CALL needs a `parallel` property.

---

## Final Verdict

The 12-primitive vocabulary is **defensible and well-scoped**. The 10 structural invariants are the right invariants. The synthesis correctly balances the systems perspective (minimality), security perspective (enforcement depth), DX perspective (usability), and P2P perspective (determinism classification).

The two critical gaps -- ITERATE cost multiplication and INVOKE output boundary -- are design-time issues, not architectural flaws. They can be specified now and enforced in the implementation.

The accepted risks (timing side channels, EMIT covert channel within same operator) are reasonable for an initial release and consistent with industry practice.

**Score: 8/10.** Address the 2 CRITs and 4 HIGHs, and this vocabulary is production-grade.

---

## Sources

- [OWASP Top 10:2025](https://owasp.org/Top10/2025/)
- [OWASP Top 10 2025 Key Changes](https://orca.security/resources/blog/owasp-top-10-2025-key-changes/)
- [OWASP Top 10 2025 Developer Guide](https://www.aikido.dev/blog/owasp-top-10-2025-changes-for-developers)
