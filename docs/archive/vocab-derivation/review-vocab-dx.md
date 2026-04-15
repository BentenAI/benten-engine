# Review: 12 Operation Primitives -- Developer Experience

**Reviewer:** Developer Experience Agent
**Date:** 2026-04-11
**Input documents:** `operation-vocab-systems.md` (10 primitives), `operation-vocab-dx.md` (18 types), `operation-vocab-security.md` (16+ types), `operation-vocab-p2p.md`
**Synthesized vocabulary under review:** 12 types (READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, GATE, CALL, RESPOND, EMIT, INVOKE, VALIDATE)
**Lens:** How does a module developer experience this vocabulary?

---

## Preamble: What the Synthesis Did

The systems perspective proposed 10 primitives. The DX perspective proposed 18. The security perspective proposed 16+. The synthesis landed on 12 by:

- Merging the DX's separate `Query`/`Read` into a single `READ` (with `mode` property)
- Merging `Delete` into `WRITE` (as `action: "delete"`)
- Folding `Sequence`, `Parallel`, `Defer` into edges and properties on existing types
- Folding `Notify`, `Webhook` into `EMIT` and entry-point triggers
- Keeping `GATE` as the TypeScript escape hatch (from DX) while also absorbing the security perspective's `CheckCapability` and `ValidateSchema` semantics via `mode`
- Adding `INVOKE` (WASM sandbox) from the security perspective
- Adding `VALIDATE` as a distinct type (from the security perspective's strong argument for schema enforcement as a first-class operation)

The 12: **READ, WRITE, TRANSFORM, BRANCH, ITERATE, WAIT, GATE, CALL, RESPOND, EMIT, INVOKE, VALIDATE.**

This review walks through all 10 DX questions.

---

## Question 1: Can a Developer Build a Basic CRUD API?

### The exercise: a "Post" content type with create/read/update/delete/list

**List posts (GET /api/posts):**

```
[GATE mode="capability" check="store:read:content/post"]
  --NEXT--> [READ mode="query" target="MATCH (p:Post {status:'published'}) RETURN p"
             options={limit: 20, offset: "$query.offset", sort: "createdAt"}]
    --NEXT--> [TRANSFORM mode="template"
               template={"items": "$.result", "total": "len($.result)"}]
      --NEXT--> [RESPOND status=200]
```

**4 Nodes.** Reasonable.

**Get single post (GET /api/posts/:id):**

```
[GATE mode="capability" check="store:read:content/post"]
  --NEXT--> [READ mode="node" target="${routeParams.id}"]
    --NEXT--> [BRANCH condition="$.result != null" mode="boolean"]
                --TRUE-->  [RESPOND status=200]
                --FALSE--> [RESPOND status=404]
```

**5 Nodes** (including two RESPOND leaves). Compare to TypeScript:

```typescript
const post = await store.getRecord('posts', id);
if (!post) return json(null, { status: 404 });
return json(post);
```

3 lines vs 5 Nodes. Not terrible, but the graph version requires the developer to explicitly model the null-check as a BRANCH, which in TypeScript is a single `if` statement. This is the honest tax of graph-based programming.

**Create post (POST /api/posts):**

```
[GATE mode="capability" check="store:create:content/post"]
  --NEXT--> [VALIDATE schema="contentType:post"]
    --NEXT--> [GATE mode="transform" check="merge($.value, {createdAt: now(), updatedAt: now()})"]
      --NEXT--> [WRITE action="create" labels=["Content","Post"]]
        --NEXT--> [EMIT event="content:afterCreate"]
          --NEXT--> [RESPOND status=201]
    --REJECT--> [RESPOND status=400]  // validation failure
  --REJECT--> [RESPOND status=403]    // capability failure
```

**8 Nodes** (including error RESPONDs). This is where graph starts to feel heavy. The equivalent TypeScript is ~12 lines including error handling, which reads faster. But the graph version makes error paths explicit and visible, which is a genuine advantage for debugging and auditing.

**Update post (PUT /api/posts/:id):**

```
[GATE mode="capability" check="store:update:content/post"]
  --NEXT--> [READ mode="node" target="${routeParams.id}"]
    --NEXT--> [BRANCH condition="$.result != null" mode="boolean"]
      --TRUE-->  [VALIDATE schema="contentType:post"]
                   --NEXT--> [WRITE action="update" target="${routeParams.id}"]
                     --NEXT--> [EMIT event="content:afterUpdate"]
                       --NEXT--> [RESPOND status=200]
                   --REJECT--> [RESPOND status=400]
      --FALSE--> [RESPOND status=404]
  --REJECT--> [RESPOND status=403]
```

**10 Nodes.** Getting verbose.

**Delete post (DELETE /api/posts/:id):**

```
[GATE mode="capability" check="store:delete:content/post"]
  --NEXT--> [READ mode="node" target="${routeParams.id}"]
    --NEXT--> [BRANCH condition="$.result != null" mode="boolean"]
      --TRUE-->  [WRITE action="delete" target="${routeParams.id}"]
                   --NEXT--> [EMIT event="content:afterDelete"]
                     --NEXT--> [RESPOND status=200]
      --FALSE--> [RESPOND status=404]
  --REJECT--> [RESPOND status=403]
```

**8 Nodes.**

### Total for full CRUD: ~35 Nodes, ~40 Edges

**Verdict:** This is viable but verbose for basic CRUD. The DX document's `crud()` shorthand is essential -- it must ship as a first-class builder, not an afterthought. Without it, a developer writing their first module will look at 35 Nodes for a CRUD API and think "I could have written this in TypeScript in 50 lines." They would be right.

**Recommendation:** The `crud()` shorthand from the DX document MUST be the primary interface for CRUD modules. The expanded 35-Node graph should be the generated output, not the authored input. The DSL should read:

```typescript
const postRoutes = crud('Post', {
  schema: 'contentType:post',
  capability: 'store:content/post',
  timestamps: true,
  events: true,
});
```

One call. Five subgraphs. 35 Nodes generated. Zero authored. This is the only way CRUD DX competes with writing TypeScript.

---

## Question 2: Naming Intuition

### Each name evaluated against the "what would a developer guess?" test

| Proposed Name | Intuitive? | Developer's first guess | Concern | Alternative considered |
|---------------|------------|------------------------|---------|----------------------|
| **READ** | Yes | "Get data from somewhere" | Correct. Universal verb. | QUERY, FETCH -- both narrower |
| **WRITE** | Yes | "Save data somewhere" | Correct. But absorbing DELETE feels odd ("I'm going to WRITE a deletion"). | MUTATE is more precise but less familiar |
| **TRANSFORM** | Yes | "Change the shape of data" | Correct. Standard ETL vocabulary. | MAP -- too narrow (suggests array operation) |
| **BRANCH** | Yes | "Choose a path" | Correct. Git vocabulary helps. | SWITCH -- implies exhaustive matching only |
| **ITERATE** | Mostly | "Loop over items" | Correct, but "iterate" has a Turing-complete connotation. Developers think `for` loops. They will be surprised by `maxIterations`. | FOREACH -- more bounded-feeling |
| **WAIT** | Yes | "Pause until something happens" | Correct. Clear and simple. | AWAIT -- too tied to async/await semantics |
| **GATE** | Problematic | "Some kind of check?" | Overloaded. Does double duty (capability check + validation + condition + transform). A developer encountering GATE for the first time will not know what it does. The systems doc gives it 4 modes, which means it is really 4 different operations hiding behind one name. | See Question 4 below |
| **CALL** | Yes | "Call a function/subgraph" | Correct. Universal programming concept. | INVOKE -- but that is already taken for WASM |
| **RESPOND** | Yes | "Send a response back" | Correct for HTTP. Less intuitive for event handlers ("I'm not responding to anyone, I'm returning a value"). | RETURN -- more universal but loses HTTP semantics |
| **EMIT** | Yes | "Fire an event" | Correct. Matches existing Thrum vocabulary exactly. | NOTIFY, PUBLISH -- both viable but EMIT has momentum |
| **INVOKE** | Problematic | "Call something?" | Collides with CALL. A developer will ask: "When do I use CALL vs INVOKE?" The answer ("CALL is for subgraphs, INVOKE is for WASM sandboxes") requires understanding the architecture. | SANDBOX, COMPUTE, RUN_WASM -- all more descriptive |
| **VALIDATE** | Yes | "Check if data is valid" | Correct. But proximity to GATE's `mode="validate"` creates confusion: "Is VALIDATE the same as GATE with validate mode?" | See Question 5 below |

### Naming score: 8/12 immediately intuitive, 4/12 need explanation

**The two naming collisions are the biggest DX problem:**

1. **GATE vs VALIDATE** -- Both check things. The distinction (GATE is inline interception on a path; VALIDATE checks against a schema) is architecturally clean but developer-invisible.

2. **CALL vs INVOKE** -- Both execute something. The distinction (CALL is subgraph-to-subgraph; INVOKE crosses the WASM boundary) is a system architecture concern that should not leak into the developer vocabulary.

**Recommendation on INVOKE naming:** Rename `INVOKE` to `SANDBOX` or `COMPUTE`. The name should scream "this is the escape hatch into general-purpose code" rather than being a synonym for CALL. The DX document called this concept `Gate` (TypeScript escape hatch), and the security document called it `Invoke` (WASM sandbox). These are the same concept at different abstraction levels. The developer-facing name should be `COMPUTE` or `EXEC` -- something that signals "arbitrary logic happens here."

---

## Question 3: What Does TRANSFORM Actually Look Like?

### The expression language

TRANSFORM uses the same sandboxed evaluator as `@benten/expressions` (jsep + custom AST walker). Based on the existing Thrum implementation, the developer can write:

**What works today in `@benten/expressions`:**

```
// Comparison
record.status === "published"

// Logical operators
record.published && user.role === "admin"

// Null coalescing
record.title ?? "Untitled"

// String methods (re-implemented, safe)
record.title.includes("draft")
record.email.startsWith("admin")
record.slug.endsWith("-v2")

// Nested property access
record.author.name
user.profile.avatar.url

// Array element access
record.tags[0]
```

**What the developer WANTS to write but CANNOT:**

```
// Arithmetic beyond simple binary ops
item.price * item.quantity              // This SHOULD work (binary *)
items.reduce((sum, i) => sum + i.price * i.qty, 0)  // CANNOT -- no function defs, no reduce

// String manipulation
title.toLowerCase()                     // CANNOT -- not in ALLOWED_METHODS
`Hello ${user.name}`                    // CANNOT -- no template literals
title.replace(/[^a-z]/g, '-')          // CANNOT -- no regex

// Object construction
{ ...input, timestamp: now() }          // CANNOT -- no spread, no function calls

// Conditional (ternary)
status === "draft" ? "Draft" : "Published"  // MAY work (jsep supports ternary)
```

### The gap is significant

The current `@benten/expressions` was built for permission conditions -- simple boolean checks like `record.status === "published"`. TRANSFORM needs a superset: arithmetic, object construction, array operations, string manipulation.

The systems document says TRANSFORM supports `len()`, `now()`, `join()` as built-in functions. These do not exist in the current `@benten/expressions`. They would need to be added.

**What `item.price * item.quantity` looks like:**

If the expression evaluator supports arithmetic binary operators (which jsep does parse), then yes, `item.price * item.quantity` would work. But the current `ALLOWED_BINARY_OPS` in `@benten/expressions` only includes comparison and logical operators -- no `*`, `+`, `-`, `/`.

**Recommendation:** The expression language needs a clear, documented capability matrix. The developer should know at a glance:

| Category | Supported | Examples |
|----------|-----------|---------|
| Property access | Yes | `$.input.title`, `$.result[0].name` |
| Comparisons | Yes | `===`, `!==`, `>`, `<`, `>=`, `<=` |
| Logical | Yes | `&&`, `\|\|`, `??`, `!` |
| Arithmetic | Needed | `+`, `-`, `*`, `/`, `%` |
| String methods | Partial | `includes`, `startsWith`, `endsWith` |
| Built-in functions | Needed | `len()`, `now()`, `join()`, `keys()`, `values()` |
| Object construction | Needed | `{ title: $.input.title, slug: slugify($.input.title) }` |
| Ternary | Check | `$.status === "draft" ? "Draft" : "Published"` |
| Template literals | No | Use TRANSFORM `template` mode instead |
| Loops/iteration | No | Use ITERATE node instead |
| Function definitions | No | Use GATE/INVOKE node instead |

Without arithmetic and object construction, TRANSFORM is too weak for the "80% case" it claims to cover. A developer cannot even compute `subtotal = price * quantity` without dropping to a GATE node, which defeats the purpose.

---

## Question 4: Should GATE Be Split?

### The problem

GATE currently has 4 modes:

| Mode | What it does | Developer mental model |
|------|-------------|----------------------|
| `capability` | Check if actor has a permission | "Am I allowed?" |
| `validate` | Check data against a schema | "Is this valid?" |
| `condition` | Evaluate a boolean expression | "Is this true?" |
| `transform` | Apply a transformation, always passes | "Clean this up" |

These are four fundamentally different operations:

1. **Capability** is a security operation. It answers "who can do what."
2. **Validate** is a data integrity operation. It answers "does this conform to a schema."
3. **Condition** is a control flow operation. It is BRANCH with only one output path.
4. **Transform** is a data operation. It is TRANSFORM that always succeeds.

Putting all four under one name creates a Swiss Army knife node -- technically capable but confusing. A developer reading `GATE mode="transform"` will think "this is a security check that... transforms data?" The name `GATE` has a strong connotation of "pass/fail check." Using it for transformations violates the principle of least surprise.

### The synthesis already partially resolved this

The 12-type vocabulary has both GATE and VALIDATE as separate types. This is the right call. But GATE still retains `mode="capability"`, `mode="condition"`, and `mode="transform"`.

**Recommendation:** Strip GATE down to ONE job: the TypeScript/WASM escape hatch.

- **Capability checking** is already handled by GATE `mode="capability"`, but this should be a property on ANY node. Every node should support an optional `requires` capability property that the engine checks before execution. This eliminates the need for standalone capability-check nodes in most cases. For complex capability logic (conditional capabilities, capability intersection), keep GATE `mode="capability"`.

- **Condition checking** is a degenerate BRANCH (one path out). It should be BRANCH with `mode="assert"` that follows the REJECT edge on falsy, NEXT on truthy. This is more honest than calling it a "gate."

- **Transform mode** should not be on GATE at all. This is just TRANSFORM. If the developer wants a transform that "always passes," they use TRANSFORM -- which by definition always passes.

The cleaned-up GATE:

```
GATE: Execute a registered handler function (TypeScript, WASM, or future Rust plugin).
      This is the escape hatch for logic too complex for the expression language.
      Properties:
        handler: string   -- registered handler ID
        timeout: number   -- max execution time
      Edges:
        NEXT: receives handler return value
        ON_ERROR: handler threw
```

This makes GATE's purpose unmistakable: "complex logic lives here." No modes. No overloading. The DX document's original conception of Gate was exactly this -- the TypeScript escape hatch. The synthesis overloaded it.

---

## Question 5: VALIDATE vs GATE -- When Do You Use Which?

### Under the current design

| Scenario | Use VALIDATE | Use GATE mode="validate" |
|----------|-------------|------------------------|
| Check data against a content type schema | Yes | Also yes |
| Check data against a custom schema | Yes | Also yes |
| Validate with custom logic beyond schema | No | Yes (but now it is a handler, not a schema check) |

The distinction is: VALIDATE checks against a declared schema stored in the graph. GATE `mode="validate"` is a handler that does validation logic in TypeScript.

**This is actually clear once you understand it, but the naming obscures it.** The developer's question is not "VALIDATE or GATE?" -- it is "Do I have a schema for this, or do I need custom logic?"

### With the recommended GATE simplification

If GATE is stripped to "escape hatch only," the confusion disappears:

- **VALIDATE** = check data against a declared schema. Produces structured field-level errors. Schema is a Node in the graph.
- **GATE** = run custom logic. May do validation, computation, or anything else. Returns whatever the handler returns.

The developer's decision tree becomes:

```
Is there a schema for this data?
  Yes -> VALIDATE
  No  -> Do I need custom validation logic?
    Yes -> GATE (handler that validates and returns errors)
    No  -> Skip validation
```

**Verdict:** VALIDATE and GATE are complementary, not competing, IF GATE is not overloaded with a `validate` mode. The recommended simplification from Question 4 resolves this entirely.

---

## Question 6: The INVOKE Escape Hatch (WASM)

### What the developer experiences

The security document describes INVOKE as calling into QuickJS-in-WASM with a gas budget and capability membrane. The developer:

1. Writes a TypeScript/JavaScript function
2. It is compiled to WASM (or runs in QuickJS-in-WASM)
3. It receives serialized JSON arguments
4. It can call host functions (graph reads, writes) but only within granted capabilities
5. It returns serialized JSON

### DX pain points

**Testing:** How does a developer test their INVOKE handler? The WASM boundary means they cannot use Vitest directly. They need either:
- A test harness that simulates the WASM sandbox locally (like PGlite simulates PostgreSQL)
- The ability to run the same code in both WASM and native Node.js (dual-target compilation)

The DX document does not address this. The security document does not address this. This is a DX gap that will block adoption.

**Debugging:** When code runs inside QuickJS-in-WASM:
- No standard `console.log` (unless host function provided)
- No breakpoints (WASM debugging tools exist but are immature)
- No stack traces that map to original TypeScript line numbers (unless source maps are threaded through)
- Gas exhaustion errors say "gas exhausted" but not "you ran out of gas in function X at line Y"

**The serialization boundary:** Every argument and return value crosses a JSON serialization boundary. This means:
- No `Date` objects (they become strings)
- No `Map` or `Set` (they become `{}`)
- No class instances (they lose their prototype)
- No functions (obviously, but developers will try)
- No `BigInt` (JSON does not support it)

Developers who write TypeScript all day will hit this wall repeatedly. They will write `return new Date()` and get `"2026-04-11T..."` as a string on the other side.

**Recommendation:** 

1. **Dual-target execution.** In development and testing, INVOKE handlers run as plain TypeScript functions in the Node.js process (no WASM boundary). In production, they run in WASM. The developer never notices the boundary during development. The engine provides a `createTestEngine()` that uses native execution for all INVOKE handlers.

2. **SerializationError with field-level diagnostics.** When a return value cannot be serialized, the error should say `INVOKE handler "commerce/calculateTax" returned a value that cannot cross the WASM boundary: field "timestamp" is a Date object. Use .toISOString() or a number timestamp instead.`

3. **Gas budget estimation tool.** `engine.estimateGas('commerce/calculateTax', sampleInput)` runs the handler and reports gas consumption. This lets developers right-size their budgets instead of guessing.

---

## Question 7: Error Handling -- "What if READ Finds Nothing?"

### The current design

The systems document's READ has only `NEXT` and `DEPENDS_ON` edges. No `ON_EMPTY` or `ON_NOT_FOUND`. To handle "not found," the developer must add a BRANCH after READ:

```
[READ mode="node" target="${id}"]
  --NEXT--> [BRANCH condition="$.result != null" mode="boolean"]
    --TRUE-->  [RESPOND status=200]
    --FALSE--> [RESPOND status=404]
```

The DX document's `Read` has `ON_NOT_FOUND`. The `Query` has `ON_EMPTY`:

```
[Read: id="${id}"]
  --NEXT-->        [RESPOND status=200]
  --ON_NOT_FOUND-> [RESPOND status=404]
```

### The DX difference is enormous

The BRANCH-after-READ pattern adds a node to every single read operation. Across a module with 20 read operations, that is 20 extra BRANCH nodes and 40 extra edges -- all expressing the same thing: "handle not found."

The `ON_NOT_FOUND` edge approach makes this a property of READ itself. The engine follows `ON_NOT_FOUND` when the result is null/empty. No extra node needed.

**This is the same argument that Rust makes for `Result<T, E>` over exceptions, and that Go makes for `val, err := ...` over try/catch.** The error path should be at the operation site, not in a separate control-flow construct.

**Recommendation:** READ should have typed error edges:

| READ mode | Success edge | Error edges |
|-----------|-------------|-------------|
| `node` | `NEXT` (receives the Node) | `ON_NOT_FOUND` (Node does not exist) |
| `query` | `NEXT` (receives result set) | `ON_EMPTY` (result set is empty, optional -- empty result is often valid) |
| `view` | `NEXT` (receives view data) | `ON_NOT_FOUND` (view does not exist) |
| `traverse` | `NEXT` (receives traversal results) | `ON_EMPTY` (traversal yielded nothing) |

Similarly, WRITE should have:
- `ON_CONFLICT` (version conflict on update)
- `ON_NOT_FOUND` (target does not exist for update/delete)

These error edges are OPTIONAL. If the developer does not connect them, the engine treats the condition as a hard error (the subgraph aborts with a structured error). This is fail-closed by default, explicit handling by choice. Best of both worlds.

---

## Question 8: Cognitive Load -- How Many Things to Learn?

### Counting what the developer must internalize

**12 operation types** with their properties:

| Type | Properties to learn | Edge types to learn |
|------|--------------------|--------------------|
| READ | target, mode (4), params, projection, options | NEXT, ON_NOT_FOUND, ON_EMPTY |
| WRITE | action (5), target, labels, edgeType/From/To | NEXT, ON_CONFLICT |
| TRANSFORM | expression, template, mode (5), fields | NEXT, MERGE_FROM |
| BRANCH | condition, mode (2) | TRUE, FALSE, MATCH:*, DEFAULT |
| ITERATE | source, maxIterations, parallel, collectAs | BODY, NEXT, ON_ERROR |
| WAIT | until, mode (3), timeout, signalId | NEXT, ON_TIMEOUT, ON_ERROR |
| GATE | handler (or check+mode in systems version) | NEXT, ON_ERROR (or REJECT) |
| CALL | subgraph, inputMap, outputMap, timeout, isolated | NEXT, ON_ERROR, ON_TIMEOUT |
| RESPOND | status, headers, channel | (none -- terminal) |
| EMIT | event, async | NEXT |
| INVOKE | runtimeId, entryPoint, args, gasBudget, memoryLimit, timeout, capabilities | NEXT, ON_ERROR |
| VALIDATE | schema, mode (2) | NEXT, ON_INVALID |

**Total surface area:**
- 12 node types
- ~45 properties across all types
- ~20 unique edge types
- 4 modes on READ, 5 actions on WRITE, 5 modes on TRANSFORM, 2 modes on BRANCH, 3 modes on WAIT

**Compared to learning a small API (12 functions):**

```typescript
// The equivalent API
read(target, opts);        // 1 function, ~5 params
write(action, data, opts); // 1 function, ~5 params
transform(expression);     // 1 function, 1 param
branch(condition);         // 1 function, 1 param
iterate(source, body);     // 1 function, ~3 params
wait(until, timeout);      // 1 function, ~3 params
gate(handler);             // 1 function, 1 param
call(subgraph, args);      // 1 function, ~4 params
respond(status, body);     // 1 function, ~3 params
emit(event, payload);      // 1 function, 2 params
invoke(fn, args, budget);  // 1 function, ~6 params
validate(schema, data);    // 1 function, 2 params
```

As functions, the same 12 operations have ~36 parameters total. As graph nodes, they have ~45 properties + ~20 edge types. **The graph representation costs ~80% more cognitive surface area** because edge types carry semantics that in a function API would be implicit (return values, error handling).

**But the graph gives something back:** the edge types make error paths and control flow explicit and visible. In a function API, error handling is whatever the developer happens to write. In the graph, `ON_NOT_FOUND`, `ON_CONFLICT`, `ON_INVALID` are visible in the structure. You can audit whether error handling exists by traversing the graph.

**Recommendation:** The cognitive load is manageable IF:

1. The developer learns the types incrementally, not all at once.
2. There is a clear ordering: READ/WRITE/RESPOND first (basic CRUD), then VALIDATE/BRANCH/TRANSFORM (data quality), then GATE/CALL (composition), then ITERATE/EMIT/WAIT/INVOKE (advanced).
3. The CRUD shorthand means most developers never manually create more than 6-8 of the 12 types.

**Suggested learning ladder:**

| Day 1 | Day 2 | Day 3 | Week 2 |
|-------|-------|-------|--------|
| READ, WRITE, RESPOND | VALIDATE, BRANCH, TRANSFORM | GATE, CALL, EMIT | ITERATE, WAIT, INVOKE |
| "I can build a basic API" | "I can validate and shape data" | "I can compose and extend" | "I can handle complex flows" |

---

## Question 9: The TypeScript DSL -- Is It Simpler?

### The DSL from the DX document

```typescript
const checkoutFlow = subgraph('commerce/checkout', (flow) => {
  flow
    .guard('commerce:checkout')
    .validate('CheckoutSchema')
    .compensate('Checkout Transaction', (tx) => {
      tx
        .step(
          gate('commerce/calculateTotal'),
          { undo: () => { /* noop */ } }
        )
        .step(
          invoke('commerce/stripe', 'chargeCard'),
          { undo: invoke('commerce/stripe', 'refund') }
        )
        .step(
          write('Order', { mode: 'create', data: '$result' }),
          { undo: (ctx) => delete_('Order', ctx.orderId) }
        );
    })
    .defer('email', notify('email', 'order-confirmation', { to: '$input.email' }))
    .onFailure(notify('email', 'checkout-failed', { to: '$input.email' }));
});
```

### The equivalent plain TypeScript (what the developer writes today in Thrum V3)

```typescript
async function checkoutHandler(ctx: ModuleRouteContext): Promise<Response> {
  await requirePermission(ctx.user, 'commerce:checkout');
  
  const input = await validateBody(ctx.request, CheckoutSchema);
  
  const result = await runWorkflow({
    steps: [
      {
        handler: async () => calculateTotal(input),
        compensate: async () => {},
      },
      {
        handler: async () => stripe.chargeCard(input),
        compensate: async (result) => stripe.refund(result.chargeId),
      },
      {
        handler: async (prev) => store.insertRecord('orders', { ...input, chargeId: prev.chargeId }),
        compensate: async (result) => store.deleteRecord('orders', result.id),
      },
    ],
  });
  
  await emailQueue.enqueue('order-confirmation', { to: input.email, order: result });
  return json(result, { status: 201 });
}
```

### Comparison

| Dimension | DSL | Plain TypeScript |
|-----------|-----|-----------------|
| Line count | ~20 | ~22 |
| Readability | Fluent chain, reads top-to-bottom | Imperative, reads top-to-bottom |
| Type safety | Builder enforces valid node sequences | Full TypeScript type checking |
| Error handling | Implicit (edge types) | Explicit (try/catch or workflow compensate) |
| Reusability | SubgraphRef references | Function calls |
| Runtime modification | Yes (graph mutation) | No (code is compiled) |
| AI introspection | Yes (traverse graph) | No (opaque function body) |
| Debugging | Execution trace + visual | Breakpoints + stack trace |
| Testing | `engine.executeSubgraph()` + mocks | Direct function call + mocks |

**Honest assessment:** The DSL is NOT simpler than plain TypeScript for this case. It is approximately the same complexity. The DSL wins on runtime modification, AI introspection, and visual debugging. TypeScript wins on type safety, IDE support, and familiarity.

**The DSL becomes simpler than TypeScript when:**
- Using `crud()` shorthand (1 line vs 50+ lines of handler code)
- Composing reusable subgraphs across modules (SubgraphRef vs copy-paste)
- Adding cross-cutting concerns (insert a fraud-check node vs modifying every handler)

**The DSL becomes more complex than TypeScript when:**
- The logic is primarily computation (tax calculation, search ranking)
- The flow has deep conditional nesting (3+ levels of branching)
- The developer needs tight TypeScript integration (generics, type narrowing)

**Recommendation:** The DSL should be presented as the PRIMARY way to define orchestration flows, with explicit guidance on when to drop to plain TypeScript (the GATE escape hatch). The 70/30 rule from the DX document is the right framing: 70% of a module is orchestration (use DSL), 30% is computation (use GATE/INVOKE).

---

## Question 10: Learning Curve

### Assumptions: Developer knows TypeScript and REST APIs

**Hour 1-2: Understanding the mental model**

The developer reads the docs. They encounter the concept: "Your route handler is a graph of operation nodes." Their first reaction is skepticism: "Why would I want to express a function as a graph?"

This is the critical onboarding moment. The docs must answer "why graph?" with concrete benefits the developer cares about:
- "You can insert a fraud check into any checkout flow without modifying the original code"
- "When your checkout fails, you see exactly which step broke and what data it had"
- "An AI agent can read your flow and suggest improvements"
- "Your module's flows can be modified at runtime without redeployment"

If the docs instead lead with "the graph is the universal substrate for computation" or "code IS Nodes and Edges," the developer zones out. They do not care about philosophy. They care about shipping features.

**Hour 2-4: Building a CRUD API**

With the `crud()` shorthand, the developer generates their first subgraphs. This should take 30 minutes. If it takes longer, the shorthand API needs work.

They then customize -- adding a GATE handler for business logic, adding an EMIT for notifications. This is where they learn 4-5 operation types by using them.

**Hour 4-8: Building a multi-step flow**

The developer builds something like a checkout flow or a content approval workflow. They encounter BRANCH, CALL, and possibly WAIT. They learn about error edges and compensation.

This is where graph-based programming starts to pay off. The developer sees their flow as a visible structure, not an opaque function. They can share it with a teammate who reads the graph without reading code.

**Day 2-3: Testing and debugging**

The developer writes tests using `createTestEngine()`. They learn about execution traces. They encounter their first debugging session using the visual trace in the admin panel.

If the testing tools work well (mocked services, assertion helpers, snapshot testing), this is a positive experience. If the tools are immature, this is where developers abandon the graph model and write plain TypeScript.

**Week 1: Productive with caveats**

After a week, the developer can build modules using 8-10 of the 12 operation types. They use GATE for complex logic and do not yet need INVOKE or WAIT.

**Week 2+: Fluent**

The developer knows all 12 types, understands when to use graph vs GATE, and can compose subgraphs across modules.

### Estimated learning curve: 2-3 days to productive, 1-2 weeks to fluent

**Compared to:**
- Express.js: 1-2 hours to productive (but no structure, no error handling, no reusability)
- SvelteKit: 1-2 days to productive (routing, load functions, form actions)
- Remix: 1-2 days to productive (loaders, actions, nested routes)
- tRPC: 2-4 hours to productive (if you already know TypeScript)
- Payload CMS: 1-2 days to productive (config-driven, similar "define your schema" model)

**The operation vocabulary is in the same ballpark as SvelteKit/Remix for learning curve, which is acceptable for a platform of this ambition.** It is slower than tRPC (which is a thin layer over existing knowledge) but faster than learning a new programming language.

**The critical risk is Day 1.** If the developer cannot get a CRUD API running in under 30 minutes, they will not reach Day 2. The `crud()` shorthand and a "Hello World" tutorial that goes from zero to deployed endpoint in 10 minutes are existential requirements.

---

## Summary Scorecard

| Dimension | Score (1-10) | Notes |
|-----------|-------------|-------|
| **CRUD viability** | 7 | Works but verbose without shorthand. Shorthand is essential. |
| **Naming intuition** | 7 | 8/12 names are clear. GATE overloading and CALL/INVOKE collision hurt. |
| **Expression language** | 5 | Too weak for TRANSFORM's claimed 80% coverage. Needs arithmetic + object construction. |
| **GATE clarity** | 4 | 4 modes is 3 too many. Should be TypeScript escape hatch only. |
| **VALIDATE vs GATE** | 6 | Clear with recommended simplification, confusing without it. |
| **INVOKE DX** | 4 | WASM boundary creates testing, debugging, and serialization pain. Needs dual-target execution. |
| **Error handling** | 6 | BRANCH-after-READ is verbose. Typed error edges on READ/WRITE would be much better. |
| **Cognitive load** | 7 | 12 types is manageable. ~80% more surface area than function API, but error paths are explicit. |
| **DSL ergonomics** | 7 | Comparable to plain TypeScript for orchestration. CRUD shorthand is the killer feature. |
| **Learning curve** | 7 | 2-3 days is competitive with SvelteKit/Remix. Day 1 experience is critical. |

**Overall DX score: 6.5/10**

The vocabulary is architecturally sound but has 4 specific DX issues that would frustrate developers:

1. **P0: GATE overloading** -- Split it. TypeScript escape hatch only.
2. **P0: Expression language too weak** -- Add arithmetic and object construction to TRANSFORM.
3. **P1: CALL vs INVOKE naming collision** -- Rename INVOKE to COMPUTE or SANDBOX.
4. **P1: Missing error edges on READ/WRITE** -- Add ON_NOT_FOUND, ON_CONFLICT, ON_EMPTY.

Fix these four issues and the score rises to 8/10.

---

## Prioritized Recommendations

### P0: Fix before any developer touches this

1. **Simplify GATE to one purpose.** Remove capability, validate, condition, and transform modes. GATE = "run a registered handler function." Capability checking becomes a property on any node (`requires: "store:read:content/*"`). Condition checking becomes BRANCH `mode="assert"`. Transform-always-passes is just TRANSFORM.

2. **Extend the expression language.** Add `+`, `-`, `*`, `/`, `%` to allowed binary operators. Add built-in functions: `len()`, `now()`, `keys()`, `values()`, `join()`, `upper()`, `lower()`, `trim()`, `round()`, `floor()`, `ceil()`, `min()`, `max()`, `coalesce()`. Add object literal construction: `{ field: expr }`. Without these, 30-40% of TRANSFORM use cases require GATE instead, making the graph verbose.

### P1: Fix before the first module developer tries to build something real

3. **Rename INVOKE to SANDBOX (or COMPUTE).** The name must signal "WASM boundary" not "function call." CALL is for subgraphs. SANDBOX is for arbitrary code. The names should be unmistakable.

4. **Add typed error edges to READ and WRITE.** `ON_NOT_FOUND` on READ `mode="node"`. `ON_EMPTY` on READ `mode="query"` (optional). `ON_CONFLICT` on WRITE `action="update"` with version checking. `ON_NOT_FOUND` on WRITE `action="update"/"delete"` when target does not exist. These save 1 BRANCH node per read/write operation, which across a module saves 20-30 nodes.

5. **Ship the `crud()` shorthand as a first-class API.** Not an afterthought, not a helper, not in an appendix. The CRUD shorthand should be the first thing in the "Getting Started" guide. It turns the 35-node full CRUD into a 1-line call. Without it, the first impression is "this is verbose."

### P2: Fix before the ecosystem grows

6. **Dual-target execution for SANDBOX/INVOKE.** In development/test: handlers run as plain TypeScript. In production: handlers run in WASM. The developer should never have to think about the WASM boundary during development.

7. **Capability checking as a node property, not a standalone node.** `{ type: "READ", target: "...", requires: "store:read:content/*" }` instead of a separate GATE node before every READ. The engine checks `requires` before executing the node. Standalone GATE `mode="capability"` is still available for complex capability logic, but 90% of cases are a simple "does the actor have this permission?" that should not cost an extra node.

8. **Learning ladder in documentation.** Structure the docs as 4 tiers: Tier 1 (READ, WRITE, RESPOND -- build a basic API), Tier 2 (VALIDATE, BRANCH, TRANSFORM -- data quality), Tier 3 (GATE, CALL, EMIT -- composition), Tier 4 (ITERATE, WAIT, SANDBOX -- advanced). Each tier has a tutorial that builds on the previous.

Sources:
- [Next.js vs Remix vs SvelteKit Comparison](https://www.nxcode.io/resources/news/nextjs-vs-remix-vs-sveltekit-2025-comparison)
- [Full-Stack DX Trends 2025-2026](https://dev.to/prashant_sharma_2558e2093/the-full-stack-convergence-strategic-trends-and-the-content-pillars-of-developer-experience-3465)
