# Exploration: The Self-Evaluating Graph -- Code AS Graph, Not Code IN Graph

**Created:** 2026-04-11
**Purpose:** Deep exploration of the foundational concept where there is NO distinction between data and code. The code IS Nodes and Edges. "Executing" an application means the graph evaluates itself by traversing operation Node subgraphs.
**Status:** Research exploration (pre-design)

---

## The Core Concept

A route handler is not a string of source code stored in a Node property. It is a SUBGRAPH of operation Nodes connected by control-flow Edges:

```
[RouteHandler: GET /api/posts]
    |--[FIRST_STEP]--> [QueryOp: find posts where published=true]
    |                      |--[NEXT_STEP]--> [TransformOp: to JSON]
    |                                           |--[NEXT_STEP]--> [ResponseOp: 200]
    |--[REQUIRES_CAPABILITY]--> [Capability: store:read:post/*]
```

"Execution" = walk the graph. Each operation Node type is a primitive the engine knows how to perform. The engine is an EVALUATOR for a self-modifying graph.

This document explores every dimension of this idea: precedents, evaluator design, performance, expressiveness, Turing completeness, self-modification, debugging, and edge deployment.

---

## 1. Precedents -- Who Has Done Something Like This?

### 1.1 Lisp/Scheme: The Original Homoiconic Language

**How they represent computation as graph:**
Lisp is the canonical example of code-as-data (homoiconicity). Programs are S-expressions -- nested lists that are simultaneously the program's syntax tree AND a data structure the program can manipulate. `(+ 1 2)` is both "add 1 and 2" and "a list of three elements." The `eval` function takes a data structure and executes it. `quote` prevents evaluation. Macros transform code-as-data at compile time.

**What worked:**
- Metaprogramming is trivial. Macros can generate arbitrary code because code IS data.
- REPLs are natural -- you can build, inspect, and execute code incrementally.
- The simplicity of the representation (everything is a list) is profound. There is one data structure, one evaluation rule.
- 70+ years of production use (1958-present). Clojure runs production systems at scale.

**What didn't:**
- S-expressions are not a graph -- they are trees. Sharing (two expressions referencing the same subexpression) requires explicit `let` bindings or mutation. Graphs naturally represent sharing; trees do not.
- "Read a program, modify it, evaluate it" is powerful but makes reasoning about behavior extremely difficult. Any function might rewrite any other function.
- Performance: interpreted Lisp is slow. Compiled Lisp (SBCL) is fast but loses some dynamism.

**Lessons for Benten:**
- The power of homoiconicity is REAL. When code and data share a representation, the system can introspect, transform, and compose itself.
- But Lisp's representation is trees. Benten's is graphs. Graphs are strictly more expressive (trees are a special case of DAGs, which are a special case of graphs). This means Benten can natively represent shared subexpressions, cycles, and multi-parent relationships that Lisp handles awkwardly.
- The danger of unrestricted self-modification is also real. Lisp programs that rewrite themselves are notoriously hard to debug.

### 1.2 Smalltalk: The Image IS the Running System

**How they represent computation:**
In Smalltalk, the entire system state -- all classes, all methods, all objects, the call stack, the UI -- exists as a single persistent "image." There is no separate source code. The program IS the running object graph. You modify the running system by sending messages to objects that represent the system itself. Want to add a method? Send `compile:` to a class object. Want to change behavior? Modify the method dictionary of a live object.

**What worked:**
- Live programming. You change code while the system runs. No compile-wait-restart cycle.
- Debugging is inspection. You can pause execution, inspect any object, modify it, and resume.
- The system is its own IDE, debugger, and runtime. There is no gap between development and execution.
- Self-contained deployment: ship the image, it runs. No dependency management.
- AI agents built with Smalltalk principles can modify their own behavior patterns and inspect their decision-making processes -- a concept directly relevant to Benten's AI-piloted vision.

**What didn't:**
- The image is opaque. Version control is extremely difficult. You cannot diff two images the way you diff two text files. Merging concurrent changes to an image is nearly impossible with standard tools.
- The image accumulates cruft. Dead objects, abandoned methods, experimental code -- all persist unless explicitly cleaned.
- Static analysis is nearly impossible. Since any object can receive any message, and methods can be added/removed at runtime, tools cannot reliably determine what code does without running it.
- Performance: the message-passing overhead is significant compared to static dispatch.

**Lessons for Benten:**
- The "image" concept is philosophically identical to "the graph IS the application." When Benten stores operation Nodes in its graph, the graph IS the running system, just as the Smalltalk image IS the running system.
- Smalltalk's version control problems are solved by Benten's version chains. Every mutation creates a version Node. The graph IS the version history. This is something Smalltalk never had.
- Smalltalk's cruft accumulation problem is mitigated by garbage collection on unreachable subgraphs.
- But Smalltalk's lesson about static analysis difficulty is a WARNING. If operation Nodes can be created, modified, and rewired at runtime, analyzing "what does this handler do?" becomes undecidable in the general case. This directly informs the Turing completeness question (Section 5).

### 1.3 Dataflow Languages: LabVIEW and Node-RED

**How they represent computation as graph:**
In LabVIEW (G language), programs ARE visual graphs. Nodes are operations (add, multiply, read-file, HTTP-request). Wires between nodes carry data. Execution is driven by data availability: a node fires when all its input wires have data. There is no instruction pointer, no program counter. The graph topology IS the execution order.

LabVIEW compiles these visual graphs to native machine code via an LLVM-based compiler. The "G" code is translated to a Dataflow Intermediate Representation, then to executable machine code. Performance is comparable to hand-written C for many workloads.

Node-RED is similar but interpreted (Node.js runtime). Each node is a JavaScript function. The runtime walks the graph, passing messages between nodes. Performance is limited by the Node.js event loop and per-node function call overhead.

**What worked:**
- Implicit parallelism. Two nodes with no data dependency between them can execute simultaneously without any explicit thread management. The scheduler handles it.
- Domain experts (hardware engineers, IoT integrators) can build real systems without traditional programming knowledge.
- LabVIEW's compilation to native code proves that graph-based computation CAN be fast.
- Frameworks like Cascade (2025) automate translation of imperative code to stateful dataflow programs, bridging the gap between textual and graph-based programming.

**What didn't:**
- **The Spaghetti Problem.** At scale, visual dataflow programs become incomprehensible tangles of nodes and wires. Research since the 1990s has consistently identified this as the fundamental scaling limitation. A 20-node flow is beautiful. A 2000-node flow is unmaintainable.
- Abstraction mechanisms are weak. LabVIEW has "SubVIs" (encapsulated subgraphs), but the visual metaphor breaks down for deep abstraction hierarchies. You cannot have a graph-of-graphs-of-graphs without cognitive overload.
- Version control and diffing are terrible. Graphs are not line-oriented text. Standard tools (git diff, code review) do not work.
- Debugging complex flows requires specialized tooling that understands the graph structure.

**Lessons for Benten:**
- LabVIEW PROVES that graph-based computation can compile to native-speed code. This is the most important precedent for performance (Section 3).
- The spaghetti problem is the biggest risk for Benten's operation Nodes. If a route handler is 5-10 Nodes, it is elegant. If complex business logic becomes 500 Nodes, it is a disaster. The escape hatch (ExternalOp / WASM) is critical -- it is the boundary between "graph-expressed logic" and "code-expressed logic."
- Benten is NOT a visual programming language. The operation graph is an internal representation evaluated by the engine, not a UI that humans draw. This sidesteps the worst of the spaghetti problem. But it does not eliminate the CONCEPTUAL spaghetti -- reasoning about graph-structured programs is inherently harder than reasoning about sequential code.

### 1.4 ComfyUI: Graph-Based AI Workflow Engine

**How they represent computation as graph:**
ComfyUI uses directed acyclic graphs (DAGs) where each node encapsulates a discrete AI operation (denoising, prompt encoding, ControlNet conditioning, LoRA adaptation, video synthesis). Nodes define explicit input/output signatures. The backend engine topologically sorts the graph and executes nodes sequentially, buffering intermediate results.

**What worked:**
- Intelligent caching: only parts of the graph that change between executions are re-executed. This is a form of incremental computation, directly analogous to Benten's IVM.
- Three caching strategies (Classic, LRU, Dependency-Aware) with content-based cache keys.
- Dynamic graph expansion: nodes can add subgraphs during execution. This is a limited form of self-modification.
- Lazy evaluation for optional inputs.

**What didn't:**
- Sequential execution after topological sort. No true parallelism within a single workflow execution (though multiple workflows can run concurrently).
- The "spaghetti diagram" problem at scale.
- No formal verification of workflows. Invalid graphs discovered only at runtime.

**Lessons for Benten:**
- ComfyUI's incremental execution (only re-execute changed subgraphs) is directly relevant. If Benten's IVM can be extended to operation subgraphs, it could cache the RESULTS of subgraph evaluation and only re-evaluate when the subgraph or its inputs change.
- The dynamic graph expansion pattern (nodes adding subgraphs during execution) is a safe, limited form of the self-modification concept. Worth considering as a middle ground.

### 1.5 Unreal Blueprints: Computation Graphs at Game Scale

**How they represent computation as graph:**
Blueprints are visual scripts where nodes represent operations (function calls, variable reads, math, control flow) connected by two kinds of wires: execution wires (white, defining order) and data wires (colored, carrying values). The Blueprint Virtual Machine (BPVM) interprets these graphs at runtime.

**What worked:**
- Rapid prototyping. Game designers (not programmers) can build interactive behavior.
- Hot-reloading: modify Blueprints while the game runs.
- Seamless interop with C++: Blueprints can call C++ functions and vice versa.

**What didn't:**
- Performance. The BPVM's main overhead is function calling -- each node dispatch has measurable cost. Complex Blueprints are 3-10x slower than equivalent C++ for CPU-bound logic.
- Epic DEPRECATED Blueprint Nativization (compiling Blueprints to C++) in UE5 due to maintenance complexity and compilation issues. This is extremely significant: they TRIED to close the performance gap by compiling graphs to native code and ABANDONED IT.
- At scale, Blueprints become unreadable. Large game projects use Blueprints for glue logic and C++ for performance-critical systems.

**What didn't (and the critical lesson for Benten):**
- The nativization failure is a red flag. Even with the resources of Epic Games, compiling visual graph programs to efficient native code proved too costly to maintain. The graph abstraction adds irreducible overhead that is hard to eliminate through compilation.

**Lessons for Benten:**
- Benten's operation Nodes are closer to an interpreted VM than compiled native code. The per-Node dispatch overhead WILL be measurable.
- The Unreal pattern (Blueprints for high-level flow, C++ for hot paths) maps directly to the proposed Benten pattern (operation Nodes for flow, WASM/ExternalOp for hot paths).
- DO NOT plan to "compile" operation graphs to native code. Epic tried and abandoned it. The escape hatch to real code (ExternalOp/WASM) is the right approach.

### 1.6 Shader Graphs: Computation Nodes That DO Compile

**How they represent computation as graph:**
Unity Shader Graph and Unreal Material Editor represent GPU shading computations as node networks. Unlike Blueprints, shader graphs ARE compiled -- to HLSL/GLSL/Metal shader code that runs on the GPU. Each node is a mathematical operation (sample texture, multiply vectors, lerp colors).

**What worked:**
- Shader graphs compile to efficient GPU code because the domain is constrained: no loops (or limited loops), no recursion, no side effects, no I/O. The graph is a pure dataflow pipeline with a fixed structure known at compile time.
- The domain restriction makes compilation tractable. The compiler knows every possible node type and can optimize aggressively.

**What didn't:**
- Compilation is slow (1-2 seconds per change in Unity). Not suitable for interactive development of complex logic.
- Only works because the domain is restricted. General-purpose computation in shader graphs is impractical.

**Lessons for Benten:**
- Shader graphs succeed precisely because they are NOT Turing complete. No unbounded loops, no recursion, no self-modification. The domain restriction is the source of their efficiency.
- If Benten's operation graphs are deliberately NOT Turing complete (Section 5), they become more analyzable, more optimizable, and more predictable -- at the cost of expressiveness.

### 1.7 Spreadsheets: The Most Successful Computation-as-Graph System

**How they represent computation as graph:**
A spreadsheet is a dependency graph where each cell is a node. Cells contain either data or formulas. Formulas reference other cells, creating edges. When a cell changes, the spreadsheet engine propagates updates through the dependency graph, recomputing only affected cells. This is reactive computation on a graph -- exactly what Benten's IVM does.

**What worked:**
- Spreadsheets are the most widely used programming paradigm on Earth. Hundreds of millions of users create computation graphs without knowing it.
- The reactive model (change a cell, dependents update) is intuitive and powerful.
- Topological sort of the dependency DAG ensures correct evaluation order.
- Incremental recomputation is natural -- only recompute what changed.
- The model is simple enough for non-programmers yet powerful enough for complex financial modeling.

**What didn't:**
- Cycles are forbidden (or produce errors). This is a deliberate design choice: DAGs, not arbitrary graphs.
- Scaling: large spreadsheets become slow because every cell is a separate node. There is no batching, no SIMD, no vectorization.
- No abstraction: you cannot define a "function" in a spreadsheet (except via macros, which are external to the graph model). Every computation must be expressed as cell formulas.
- The flat namespace (cell references) creates fragile dependencies. Insert a row and formulas break.

**Lessons for Benten:**
- Spreadsheets prove that reactive computation on a dependency graph is intuitive and powerful for end users. Benten's IVM is the same paradigm at the engine level.
- The "no cycles" restriction is important. Operation subgraphs should be DAGs (directed acyclic graphs) for the same reason spreadsheets require DAGs: to guarantee termination and enable topological evaluation.
- The lack of abstraction in spreadsheets is a warning. Benten's operation Nodes need a composition mechanism (subgraph encapsulation) to avoid the "flat namespace" problem.

### 1.8 Business Process Engines: BPMN, Temporal, Camunda

**How they represent computation as graph:**
BPMN engines represent business workflows as directed graphs. Nodes are activities (tasks, service calls, human approvals). Edges are transitions with conditions. The engine walks the graph, executing each activity and following transitions based on conditions.

Temporal takes a different approach: workflows are written as ordinary code (Go, Java, TypeScript) but execution is durable -- the engine records every step and can replay from any point. The "graph" is implicit in the control flow.

**What worked:**
- BPMN: visual representation of complex business processes. Non-technical stakeholders can understand and modify workflows.
- Temporal: durable execution solves the problem of long-running processes (hours, days, weeks) that survive crashes.
- Both provide audit trails by construction -- every step is recorded.

**What didn't:**
- BPMN engines add significant latency. They are designed for business processes (seconds to days per step), not for high-performance request handling (microseconds per step).
- Temporal's event-sourcing model adds latency compared to direct service-to-service calls. For sub-millisecond requirements, it is not appropriate.
- BPMN's visual graphs suffer the same spaghetti problem as all visual programming at scale.

**Lessons for Benten:**
- The audit trail property (every step is recorded) comes for free with Benten's version chains. Every operation execution creates graph mutations, which create version Nodes.
- But the performance characteristics of BPMN/Temporal are a WARNING. These systems are designed for orchestration (10ms-1000ms per step), not for application runtime (sub-microsecond per step). Benten's operation Nodes must be MUCH faster than workflow engine steps.

### 1.9 Graph Rewriting Systems: The Theoretical Foundation

**How they represent computation as graph:**
In algebraic graph rewriting (Double-Pushout approach, Single-Pushout approach), computation IS graph transformation. A rewrite rule has a left-hand side (a pattern to match) and a right-hand side (the replacement). Execution = repeatedly apply rewrite rules until no more rules match (a "normal form" is reached). Lambda calculus can be encoded as graph reduction: beta-reduction becomes a graph rewrite rule.

**What worked:**
- Theoretical elegance. Graph rewriting is a universal model of computation.
- Sharing: unlike tree rewriting, graph rewriting can share subexpressions. This is the key advantage over term rewriting for implementing functional languages.
- Confluence: if a graph rewriting system is confluent, any order of rule application reaches the same result. This enables parallelism (apply rules in any order on any thread).

**What didn't:**
- Pure graph rewriting is slow. The pattern matching required to find where rules apply is expensive (subgraph isomorphism is NP-complete in the general case).
- Most practical implementations restrict the graph structure to reduce matching cost.
- The theory is beautiful but implementations are rare in production.

**Lessons for Benten:**
- Benten's operation Nodes are NOT graph rewriting in the algebraic sense. The engine does not pattern-match rewrite rules against the graph. Instead, it WALKS a known subgraph structure (FIRST_STEP -> NEXT_STEP -> ...). This is more like an interpreter walking an AST than a rewriting engine searching for patterns.
- The confluence property is relevant: if two operation Nodes are independent (no data dependency), they can execute in parallel, and the result is the same regardless of order.

### 1.10 HVM/Bend and Interaction Nets: Modern Graph Reduction

**How they represent computation as graph:**
HVM2 (Higher-order Virtual Machine 2) implements Yves Lafont's Interaction Combinators -- a model where computation happens by local graph rewrites on pairs of connected nodes. Programs compile to interaction nets (graphs of combinators), and execution = applying 6 simple rewrite rules in parallel until no more rewrites are possible.

**What worked (spectacularly):**
- Near-ideal parallel speedup: 400 MIPS (single thread, M3 Max) to 74,000 MIPS (32,768 threads, RTX 4090). 185x speedup on 32K threads.
- Confluence guarantees correct results regardless of execution order. Any thread can apply any rewrite.
- The model is simple: just 3 node types and 6 rules. Everything else is encoded.
- Bend (the high-level language) compiles to HVM2 and "scales like CUDA" without explicit parallelism annotations.

**What didn't:**
- The encoding of real programs into interaction combinators produces LARGE graphs. A simple function becomes hundreds of combinators.
- Memory overhead: interaction nets require significantly more memory than traditional representations.
- HVM/Bend is still experimental. Production adoption is minimal.
- Debugging: stepping through interaction combinator reductions is incomprehensible to humans.

**Lessons for Benten:**
- HVM proves that graph-based computation CAN be massively parallel with near-ideal speedup. The key is LOCALITY: each rewrite involves only two adjacent nodes, so there are no long-range dependencies.
- Benten's operation Nodes are NOT interaction nets. They are higher-level (QueryOp, TransformOp, ResponseOp), not low-level combinators. But the PRINCIPLE of local evaluation (each operation Node can be evaluated by looking only at its inputs and the next edge) should be preserved.
- The encoding overhead (real programs become large graphs) is a cautionary tale. Benten should keep operation Nodes at a high level of abstraction, not decompose everything into primitive combinators.

### 1.11 The Sea of Nodes: Compilers That Use Graphs as IR

**How they represent computation as graph:**
The "Sea of Nodes" is an intermediate representation (IR) used in production compilers. V8's TurboFan JIT compiler, HotSpot JVM's C2 compiler, and GraalVM all used/use Sea of Nodes. In this representation, both data flow and control flow are represented as a graph. Operations are nodes. Dependencies are edges. The partial ordering (vs. total ordering of sequential instructions) enables aggressive optimization because operations can be reordered freely.

**What worked:**
- Most optimizations become simple local graph transformations. This makes the optimizer modular and composable.
- The relaxed ordering exposes parallelism that sequential IRs hide.

**What didn't -- and this is critically important:**
- **V8 ABANDONED Sea of Nodes.** Starting in 2022, V8 replaced TurboFan's Sea of Nodes IR with Turboshaft, a traditional Control-Flow Graph (CFG) IR. The reasons:
  - Sea of Nodes was "poorly suited for JavaScript's dynamicity, making development and debugging too difficult."
  - Compilation speed doubled by switching to CFG.
  - The compiler code became "a lot simpler and shorter."
  - "Investigating bugs is usually much easier."
- This happened at Google, with world-class compiler engineers, after 8+ years of production use.

**Lessons for Benten:**
- Even for COMPILER IR (not user-facing), graph-based computation representations proved too complex to maintain. The debugging difficulty and cognitive overhead were real costs that outweighed the optimization benefits.
- This does NOT mean Benten's approach is doomed. V8's problem was that the entire program was a sea of nodes, including all the messy edge cases of JavaScript semantics. Benten's operation Nodes are a SMALL, HIGH-LEVEL graph (5-10 Nodes per handler) evaluated at runtime, not a fine-grained compiler IR.
- But it is a caution against decomposing logic too finely into graph form.

### 1.12 The Wolfram Physics Project: Hypergraph Rewriting as Universal Computation

**How they represent computation as graph:**
The Wolfram Physics Project models the universe itself as a hypergraph that evolves by rewriting rules. Computation IS the evolution of the hypergraph. The "multiway system" explores ALL possible rule application orderings simultaneously, producing a graph of all possible computational histories.

**What's relevant:**
- Compiling different models of computation into hypergraph rewriting has been demonstrated. Lambda calculus, cellular automata, Turing machines -- all encode as hypergraph rewrites.
- The multiway system is the most radical version of "graph evaluates itself" ever proposed.

**What's NOT relevant:**
- This is theoretical physics, not software engineering. The performance characteristics are irrelevant to Benten.
- The universality of hypergraph rewriting is a theoretical result, not a practical programming model.

**Lessons for Benten:**
- The theoretical result that ALL computation can be represented as graph rewriting supports the feasibility of Benten's approach.
- But "can be represented" is not "should be represented." The fact that you CAN encode a for-loop as a graph rewrite does not mean you SHOULD.

---

## 2. The Evaluator Design

### 2.1 How Does the Engine Walk an Operation Subgraph?

Two fundamental approaches: recursive and iterative. The right choice has deep implications.

**Recursive (each node calls the next):**
```
fn evaluate(node: NodeId, ctx: &mut Context) -> Result<Value> {
    let op = graph.get_node(node);
    let result = match op.label {
        "QueryOp" => execute_query(op, ctx),
        "TransformOp" => execute_transform(op, ctx),
        "ResponseOp" => execute_response(op, ctx),
        "ConditionalOp" => {
            let condition = evaluate_condition(op, ctx);
            let branch = if condition { "TRUE_BRANCH" } else { "FALSE_BRANCH" };
            let next = graph.get_edge(node, branch).target;
            evaluate(next, ctx)
        }
        _ => Err(UnknownOp)
    };
    
    if let Some(next_edge) = graph.get_edge(node, "NEXT_STEP") {
        ctx.set_result(result?);
        evaluate(next_edge.target, ctx)
    } else {
        result
    }
}
```

Pros: Natural for branching and recursion. Call stack tracks execution position.
Cons: Stack depth limits (a 1000-step pipeline overflows the stack). Harder to pause/resume.

**Iterative (a cursor walks the chain):**
```
fn evaluate(entry: NodeId, ctx: &mut Context) -> Result<Value> {
    let mut current = Some(entry);
    
    while let Some(node_id) = current {
        let op = graph.get_node(node_id);
        let result = dispatch(op, ctx)?;
        ctx.set_result(result);
        
        current = graph.get_edge(node_id, "NEXT_STEP").map(|e| e.target);
    }
    
    ctx.final_result()
}
```

Pros: No stack depth limit. Easy to pause/resume (save `current` node ID). Predictable memory usage.
Cons: Branching requires explicit stack management. Less natural for recursive structures.

**Recommendation: Iterative with an explicit execution stack.**

This is how most production virtual machines work (JVM, CPython, Lua). The evaluator maintains:
- A `current` node pointer (the "instruction pointer")
- An execution stack (for branching and subroutine calls)
- A context/environment (data flowing between operations)

```
struct Evaluator {
    current: Option<NodeId>,
    stack: Vec<StackFrame>,
    context: Context,
}

struct StackFrame {
    return_to: NodeId,
    saved_context: Context,
}
```

This gives you:
- Pause/resume (serialize the Evaluator state)
- Step-through debugging (advance `current` one node at a time)
- Bounded stack depth (reject operations that exceed depth limit)
- No Rust stack overflow risk

### 2.2 How Does Branching Work?

A `ConditionalOp` node has two outgoing edges: `TRUE_BRANCH` and `FALSE_BRANCH`. The evaluator evaluates the condition (from the node's properties or the current context), then follows the appropriate edge.

```
[ConditionalOp: ctx.user.role == 'admin']
    |--[TRUE_BRANCH]--> [QueryOp: find all posts]
    |--[FALSE_BRANCH]--> [QueryOp: find published posts]
```

Both branches may converge at a `MergeOp` node (or simply continue to their own `NEXT_STEP` chains). The evaluator does not need special merge handling -- it simply follows edges.

For multi-way branching (switch/case), a `MatchOp` node can have N outgoing edges labeled with match values:

```
[MatchOp: ctx.request.method]
    |--[CASE:GET]--> [...]
    |--[CASE:POST]--> [...]
    |--[CASE:DELETE]--> [...]
    |--[DEFAULT]--> [ResponseOp: 405]
```

### 2.3 How Do Loops Work?

A `LoopOp` node has:
- A `BODY` edge pointing to the first node of the loop body
- An `EACH_ITEM` or `WHILE_CONDITION` property
- A `NEXT_STEP` edge for after the loop completes

For iteration over a collection:
```
[LoopOp: for item in ctx.posts]
    |--[BODY]--> [TransformOp: format item]
    |               |--[NEXT_STEP]--> [AccumulateOp: push to results]
    |--[NEXT_STEP]--> [ResponseOp: 200, ctx.results]
```

The evaluator:
1. Evaluates the collection expression
2. For each item: pushes a StackFrame, sets `ctx.current_item`, evaluates the BODY subgraph, pops the StackFrame
3. After all items: follows NEXT_STEP

**Critical: loop iteration count MUST be bounded.** See Section 5 on Turing completeness.

### 2.4 How Does Data Flow Between Operations?

Three approaches, each with tradeoffs:

**Option A: Shared mutable context (like a thread-local environment)**
Each operation reads from and writes to a shared `Context` object. Operations access previous results via named keys.

```
QueryOp writes:     ctx["posts"] = [...]
TransformOp reads:  ctx["posts"]
TransformOp writes: ctx["json"] = serialize(ctx["posts"])
ResponseOp reads:   ctx["json"]
```

Pros: Simple. Familiar (like request/response middleware chains).
Cons: Implicit data flow. Hard to reason about which operations depend on which data. Name collisions.

**Option B: Edge-carried data (each edge carries a value)**
The result of each operation is attached to the outgoing edge. The next operation receives it as input.

```
QueryOp --[NEXT_STEP, data=[post1, post2]]--> TransformOp
TransformOp --[NEXT_STEP, data='{"posts":[...]}']--> ResponseOp
```

Pros: Explicit data flow. Each operation's inputs and outputs are clear. Natural for parallel evaluation (no shared state).
Cons: Complex for operations that need data from non-adjacent nodes. Either you thread data through every intermediate node, or you need a mechanism for "reaching back" in the graph.

**Option C: Hybrid (context + explicit bindings)**
A shared context exists, but operations declare which keys they read and write. The engine validates data flow at graph validation time (before execution).

```
[QueryOp: read=[], write=[posts]]
[TransformOp: read=[posts], write=[json]]
[ResponseOp: read=[json], write=[]]
```

**Recommendation: Option C (Hybrid).** This gives the simplicity of a shared context while enabling static analysis of data flow. The engine can:
- Validate that every `read` has a corresponding `write` by a preceding operation
- Detect unused writes (dead data)
- Determine which operations are independent (no read/write overlap) for parallel execution
- Provide clear error messages when data flow is broken

### 2.5 Error Handling

A `TryCatchOp` node has:
- A `TRY` edge pointing to the guarded subgraph
- A `CATCH` edge pointing to the error handler subgraph
- A `NEXT_STEP` edge for after the try/catch

```
[TryCatchOp]
    |--[TRY]--> [QueryOp: might fail]
    |               |--[NEXT_STEP]--> [TransformOp]
    |--[CATCH]--> [LogOp: log error]
    |                 |--[NEXT_STEP]--> [ResponseOp: 500]
    |--[NEXT_STEP]--> [ResponseOp: 200]  (reached if TRY succeeds)
```

The evaluator pushes a catch frame onto the execution stack. If any operation in the TRY subgraph returns an error, the evaluator pops back to the catch frame and follows the CATCH edge.

This is identical to how exception handling works in bytecode VMs (JVM's exception table, Python's try/except blocks). The graph representation is the bytecode.

### 2.6 Subroutines and Subgraph Invocation

A `CallSubgraphOp` node references another operation subgraph by its anchor NodeId. The evaluator:
1. Pushes a StackFrame with the return-to node
2. Jumps to the entry point of the referenced subgraph
3. When the subgraph completes, pops the StackFrame and resumes

This enables reusable operation subgraphs -- the graph equivalent of function calls. Combined with capability checking on the REQUIRES_CAPABILITY edge, this is also the mechanism for module-to-module invocation.

---

## 3. Performance

### 3.1 The Per-Node Overhead Budget

For a route handler with 5-10 operation Nodes, the total overhead must be imperceptible. Let's set the budget:

- Target response time for a simple API endpoint: <1ms
- Number of operation Nodes: 5-10
- Per-node overhead budget: <10 microseconds (to leave room for actual work like DB queries)

What does "per-node overhead" include?
1. **Graph lookup:** Fetch the node from the graph store. With IVM, this is O(1) -- a hash lookup. ~100ns.
2. **Edge traversal:** Find the NEXT_STEP edge. With adjacency lists, this is O(degree). For operation nodes with 1-3 outgoing edges: ~50ns.
3. **Dispatch:** Match the node's label to the handler function. With a jump table or match statement: ~10ns.
4. **Context access:** Read/write the shared context. With a HashMap: ~50ns per access.
5. **Bookkeeping:** Update the evaluator's current pointer, check depth limits: ~10ns.

**Total estimated overhead: ~200-300ns per node.** For 10 nodes: ~2-3 microseconds. This is well within budget.

### 3.2 How Does This Compare to V8?

V8's Ignition bytecode interpreter processes one bytecode instruction per ~10-50ns (depending on the instruction). A simple function call is ~50-100ns. A property lookup is ~20-50ns.

Benten's operation Nodes are MUCH coarser-grained than bytecode instructions. A single QueryOp does the work of hundreds of bytecode instructions. So the comparison is:

| Approach | Overhead per "step" | Steps per handler | Total overhead |
|----------|-------------------|-------------------|----------------|
| V8 Ignition (bytecode) | ~10-50ns | ~1000 | ~10-50us |
| V8 TurboFan (JIT compiled) | ~1-5ns | ~1000 | ~1-5us |
| Benten operation Nodes | ~200-300ns | ~5-10 | ~1-3us |
| Native Rust function calls | ~1-2ns | ~5-10 | ~5-20ns |

Benten's graph evaluation is competitive with V8's bytecode interpreter and comparable to JIT-compiled code -- because the operation Nodes are high-level and few. The overhead is in the graph traversal, but the actual WORK (executing a database query, transforming data) dominates.

**The danger zone:** If operation Nodes become too fine-grained (a Node for every addition, every comparison, every variable assignment), the per-Node overhead dominates and performance collapses. This is exactly what happened with Unreal Blueprints.

**The safe zone:** If operation Nodes stay at the level of "query the database," "transform a data structure," "check a capability," the overhead is negligible compared to the work each node does.

### 3.3 Can the Evaluator JIT-Compile Hot Subgraphs?

In principle, yes. Cranelift (a Rust-native JIT compiler) could compile a "frozen" operation subgraph into native code:

1. Identify hot subgraphs (evaluated >N times)
2. Translate the graph into Cranelift IR (CLIF)
3. Compile to native machine code
4. Cache the compiled function
5. On next evaluation, call the native function instead of walking the graph

Cranelift compiles ~10x faster than LLVM, making it suitable for JIT. It is used in production by Wasmtime and the Rust compiler itself.

**However:** This is a massive engineering effort and should NOT be in the first release. The initial evaluator should be an interpreter. JIT compilation is an optimization for later, IF profiling shows that graph-walking overhead is a bottleneck.

**Furthermore:** JIT compilation of operation subgraphs only works if those subgraphs are stable. If the graph is being modified at runtime (self-modification, Section 6), JIT-compiled code must be invalidated. This creates complexity.

**Recommendation:** Build the interpreter first. Measure. If the per-Node overhead is acceptable (it should be, given the analysis above), JIT compilation may never be needed. If it IS needed, Cranelift is the right tool.

### 3.4 IVM as the Real Performance Story

The per-Node evaluation overhead is a distraction from the REAL performance story: IVM eliminates query cost entirely.

Without IVM: "Handle GET /api/posts" = walk 5 operation Nodes + execute a database query (1-10ms).
With IVM: "Handle GET /api/posts" = walk 5 operation Nodes + read a pre-computed materialized view (0.01ms).

The database query dominates the response time. IVM eliminates it. The graph-walking overhead (2-3 microseconds) is noise.

This is why the engine specification focuses on IVM as the key innovation, not the operation Node evaluator. The evaluator is just the glue that orchestrates pre-computed results.

---

## 4. Expressiveness

### 4.1 What CAN Be Expressed as Operation Nodes?

The proposed operation Node types cover common web application patterns:

| Operation | What It Does | Sufficient for |
|-----------|-------------|----------------|
| QueryOp | Read from the graph (via IVM) | CRUD reads, listings, search |
| CreateOp | Write a Node/Edge to the graph | CRUD creates |
| UpdateOp | Modify a Node's properties | CRUD updates |
| DeleteOp | Remove a Node/Edge | CRUD deletes |
| TransformOp | Map/filter/reshape data in context | JSON serialization, field selection, data shaping |
| ConditionalOp | Branch on a condition | Auth checks, feature flags, content routing |
| LoopOp | Iterate over a collection | Batch operations, list rendering |
| ResponseOp | Construct an HTTP response | API endpoints |
| TryCatchOp | Error handling | Graceful degradation |
| CallSubgraphOp | Invoke another operation subgraph | Code reuse, module invocation |
| ExternalOp | Execute WASM module | Complex computation, custom logic |

### 4.2 What CANNOT Be Expressed (Without ExternalOp)?

- **String manipulation:** Regex, templates, string building. You COULD add StringOp nodes (concat, split, replace, match), but this quickly leads to a proliferation of fine-grained nodes.
- **Complex math:** Beyond basic arithmetic. Financial calculations, statistics, ML inference.
- **Date formatting and timezone handling.**
- **Closures and higher-order functions:** The graph has no concept of "a function that returns a function." You can model this as "a subgraph that produces a reference to another subgraph," but the ergonomics are poor.
- **Recursive algorithms:** Tree traversal, Fibonacci, recursive descent parsing. The graph can represent recursion (a subgraph that calls itself), but the evaluator needs stack depth limits and the mental model is unwieldy.
- **Anything requiring libraries:** Cryptographic hashing, image processing, PDF generation.

### 4.3 The Expressiveness Cliff

There is a natural boundary between what operation Nodes express well and what they express poorly:

**Operation Nodes excel at:** Orchestration. "Get this data, check this condition, transform it, return it." The logic is about COORDINATION between high-level operations.

**Operation Nodes struggle with:** Computation. "Parse this string with a regex, compute the SHA-256 hash, format this date as ISO-8601." The logic is about MANIPULATION of values.

This is the same boundary that separates workflow engines from programming languages. BPMN excels at "step 1, then step 2, if condition then step 3a else step 3b." It struggles at "compute the Levenshtein distance between two strings."

### 4.4 The ExternalOp Escape Hatch

ExternalOp (WASM) is the answer to the expressiveness cliff. It is not a failure of the model -- it is a DESIGN FEATURE. The graph represents the orchestration; WASM represents the computation.

```
[RouteHandler: GET /api/posts/search]
    |--[FIRST_STEP]--> [QueryOp: find all posts]
    |--[NEXT_STEP]--> [ExternalOp: wasm/search-scorer.wasm, input: {posts, query}]
    |                   (scores and ranks posts by relevance using custom algorithm)
    |--[NEXT_STEP]--> [TransformOp: take top 10]
    |--[NEXT_STEP]--> [ResponseOp: 200]
```

The WASM module runs in a sandbox with explicit capabilities. It receives JSON in, returns JSON out. It cannot modify the graph, access the network, or read the filesystem -- unless the capability graph grants it.

### 4.5 At What Point Does This Become a Visual Programming Language?

**It becomes a VPL when humans author operation graphs directly.** If a developer is manually creating QueryOp Nodes, wiring NEXT_STEP Edges, and debugging by inspecting the graph, then it IS a visual programming language (or at least a graph-based programming language), and it inherits all the scaling problems documented in Section 1.3.

**It does NOT become a VPL if:**
- Operation graphs are generated by AI agents that compile high-level intent to graph structure
- Operation graphs are generated by a higher-level DSL that compiles to the graph
- Operation graphs are generated by the platform (e.g., route registration creates the graph automatically)
- Humans interact with the system at a level ABOVE the operation graph (natural language, configuration, visual builder) and the graph is an implementation detail

**The key insight:** The operation graph is an INTERMEDIATE REPRESENTATION, like bytecode. Java developers do not write JVM bytecode by hand. They write Java, which compiles to bytecode. Similarly, Benten users should not manually author operation graphs. They should author at a higher level (natural language, DSL, visual builder), and the platform compiles that to operation Nodes.

This changes the evaluation calculus entirely. The spaghetti problem is irrelevant if humans never see the graph. The abstraction is not for human readability -- it is for machine evaluation, introspection, and modification.

---

## 5. The Turing Completeness Question

### 5.1 Is This System Turing Complete?

With `ConditionalOp` + `LoopOp` + `QueryOp` + `CreateOp`, is this Turing complete?

Yes, almost certainly. A LoopOp that iterates based on a condition (while-loop), combined with the ability to create Nodes (arbitrary memory), gives you a Turing machine. The graph state is the tape, the evaluator is the head, the operation Nodes are the transition rules.

### 5.2 Should It Be Turing Complete?

**The case FOR Turing completeness:**
- Maximum flexibility. Any computation can be expressed.
- No artificial limitations that frustrate developers.
- AI agents can generate arbitrarily complex operation graphs.

**The case AGAINST Turing completeness:**

1. **Termination is undecidable.** If the operation graph language is Turing complete, you CANNOT determine whether a given operation graph will halt. A malicious or buggy module could create an infinite loop that consumes resources forever.

2. **Static analysis becomes impossible.** You cannot determine, in general, what resources a Turing-complete operation graph will access, how much memory it will consume, or how long it will run. This means capability enforcement must be RUNTIME-only (no static verification).

3. **Datalog is deliberately not Turing complete -- and that is a feature.** Datalog programs are guaranteed to terminate in polynomial time. This enables static analysis, optimization, and reasoning about program behavior. Souffle (a practical Datalog) extends Datalog with arithmetic to become Turing-equivalent, but this explicitly breaks the termination guarantee.

4. **Total functional programming (Turner, 2004) demonstrates that restricting recursion to structurally decreasing arguments guarantees termination while retaining most practical expressiveness.** You can express any primitive recursive function. You cannot express the Ackermann function or arbitrary while-loops, but you rarely need to.

5. **Shader graphs succeed precisely because they are not Turing complete.** No unbounded loops means the compiler can analyze, optimize, and bound the execution of every shader. This is why GPUs are fast: they trade generality for predictability.

### 5.3 Recommendation: Deliberately NOT Turing Complete

The operation graph language should be deliberately limited:

**Allowed:**
- Sequential execution (NEXT_STEP)
- Conditional branching (ConditionalOp with TRUE_BRANCH/FALSE_BRANCH)
- Bounded iteration (LoopOp over a finite collection, with a maximum iteration count)
- Subgraph calls (with a maximum call depth)
- All query/create/update/delete operations
- Data transformation

**NOT allowed (in the graph):**
- Unbounded while-loops (no "loop until condition" without a hard iteration limit)
- Recursive subgraph calls without structural decrease (a subgraph cannot call itself with the same or larger input)
- Arbitrary memory allocation (cannot create Nodes in an unbounded loop)

**The escape hatch:** ExternalOp (WASM) IS Turing complete. If you need unbounded computation, use WASM. The engine enforces resource limits on WASM (fuel, memory, time) via the capability system.

**What this gives you:**
- **Guaranteed termination:** Every operation graph halts. The engine can PROVE this statically.
- **Bounded resource usage:** The engine can compute an upper bound on execution time and memory for any operation graph.
- **Static capability analysis:** The engine can determine, before execution, every capability an operation graph will require.
- **Optimizability:** The engine can reorder, parallelize, and optimize operations because it knows the graph will terminate.
- **Safety:** A malicious module cannot create a denial-of-service via an infinite-loop operation graph.

**What you give up:**
- Some computation must use ExternalOp/WASM instead of being expressed in the graph. This is the RIGHT tradeoff: the graph is for orchestration, WASM is for computation.

### 5.4 Enforcement

The engine validates operation graphs at DEFINITION TIME (when the graph is created/modified):

1. **No WHILE edges without a MAX_ITERATIONS property.** LoopOp must specify a bound.
2. **No recursive CallSubgraphOp cycles without structural decrease.** The engine detects cycles in the call graph and rejects them unless each recursive call operates on a strictly smaller input.
3. **Maximum graph depth.** A flat limit (e.g., 100 Nodes per operation subgraph) prevents absurdly large graphs.
4. **Maximum call depth.** CallSubgraphOp calls are limited to N levels of nesting.

These checks run at write time (when operation Nodes are created). They are O(graph size), not O(execution), so they do not add runtime overhead.

---

## 6. Self-Modification

### 6.1 The Power

A handler can create/modify other Nodes, including operation Nodes. This means:
- An AI agent can write new route handlers by creating operation subgraphs
- A module can extend another module's behavior by inserting operation Nodes into its graph
- The system can optimize itself by rewriting operation subgraphs based on profiling data
- User-defined automations (IFTTT-style) are operation subgraphs created at runtime

This is the most powerful aspect of the self-evaluating graph. The application can build itself.

### 6.2 The Danger

**Infinite self-modification loops:** Operation A creates a new operation B. Operation B modifies operation A. Operation A now creates a different operation B. The system oscillates forever.

**Semantic drift:** Small modifications accumulate until the system's behavior is unrecognizable from its original design. This is the "Ship of Theseus" problem for code.

**Reasoning failure:** If any operation can modify any other operation, understanding "what does this system do?" requires simulating the entire execution history. This is the halting problem in disguise.

**Security:** If a compromised module can rewrite other modules' operation graphs, the entire system is compromised.

### 6.3 Safeguards

**1. Capability-mediated modification.**
Creating or modifying operation Nodes requires explicit capabilities:
- `graph:write:operation/*` -- can create any operation Node (extremely privileged)
- `graph:write:operation/self/*` -- can modify only operation Nodes owned by this module (via scope/namespace)
- `graph:read:operation/*` -- can read (introspect) operation graphs but not modify them

Most modules would have ONLY `graph:write:operation/self/*`. They can define their own handlers but cannot modify others'.

**2. Validation on write.**
Every modification to an operation subgraph triggers re-validation:
- Termination check (Section 5.4)
- Capability check (does the modified graph require capabilities the author does not have?)
- Schema check (does each operation Node have the required properties?)
- Connectivity check (is the graph still reachable from entry points?)

**3. Version chains as undo.**
Every modification to an operation subgraph creates a new version. If a self-modification produces bad behavior, rollback is one operation: move the CURRENT pointer back.

**4. Modification depth limits.**
During a single evaluation, the number of operation-Node-creating/modifying operations is bounded. A single handler cannot create 10,000 new operation Nodes in one execution.

**5. Sandbox for self-modification.**
Modifications to operation Nodes take effect only AFTER the current evaluation completes. An operation cannot modify the graph it is currently being evaluated from. This prevents "sawing off the branch you're sitting on" and makes the system deterministic within a single evaluation.

### 6.4 When Is Self-Modification Appropriate?

**YES:**
- AI agents generating new handlers in response to user requests
- Admin-configured automations (workflow builder UI)
- Module installation (defining new operation subgraphs)
- A/B testing (creating variant operation subgraphs)

**NO:**
- Hot-path request handling modifying its own graph on every request
- Unbounded self-optimization (operation graphs that rewrite themselves to be "better")
- Cross-module modification (module A rewriting module B's handlers)

---

## 7. Debugging and Observability

### 7.1 Stepping Through Nodes

Traditional debuggers step through lines of code. Here, you step through Nodes. The evaluator's iterative design (Section 2.1) makes this natural:

```
> step
  [1] QueryOp: find posts where published=true
  context: { posts: [Post{id: 1, title: "Hello"}, Post{id: 2, title: "World"}] }

> step
  [2] TransformOp: to JSON
  context: { posts: [...], json: '{"posts":[...]}' }

> step
  [3] ResponseOp: 200
  result: HTTP 200, body: '{"posts":[...]}'
```

Each step shows:
- The current Node (with label and properties)
- The context state after execution
- The next Node to be evaluated
- The execution stack (for nested subgraph calls)

### 7.2 Replay

Since the operation graph is immutable during evaluation (Section 6.3, safeguard 5), you can replay any evaluation by:
1. Saving the entry Node ID and the initial context
2. Re-walking the same graph with the same inputs

If the graph HAS been modified between the original evaluation and the replay, version chains let you replay against the ORIGINAL version: walk the graph as it existed at a specific version.

### 7.3 Error Traces

A traditional stack trace is:
```
Error at file.ts:42 in function processPost
  called from file.ts:28 in function handleRequest
  called from router.ts:15 in function dispatch
```

A graph-based error trace is:
```
Error at Node[QueryOp, id: abc123, label: "find posts"]
  step 2 of subgraph[RouteHandler: GET /api/posts, id: def456]
  called from subgraph[Router: /api/*, id: ghi789]
  
  Context at failure:
    { query: "published=true", user: { id: 1, role: "admin" } }
  
  Graph path: ghi789 --[FIRST_STEP]--> def456 --[FIRST_STEP]--> abc123
```

The trace includes:
- The failing Node (with its ID, label, and properties)
- The full subgraph call chain
- The context state at the point of failure
- The graph path (sequence of Node IDs and Edge types) from entry to failure

This is strictly MORE informative than a file:line trace because:
- The Node properties show WHAT was being done, not just WHERE
- The context shows the data state at failure
- The graph path is the equivalent of a call stack, but with richer metadata

### 7.4 Observability Integration

Every operation Node evaluation can emit structured telemetry:
- Node ID, label, and type
- Execution duration
- Input/output context diff
- Capability checks performed
- Errors and retries

Since operation Nodes are coarse-grained (5-10 per handler), the telemetry volume is manageable. Each request produces 5-10 spans, not thousands.

This integrates naturally with OpenTelemetry: each operation Node evaluation is a span, the subgraph is a trace.

---

## 8. Edge Deployment

### 8.1 The Graph IS the Deployable Artifact

The entire application IS the graph. Deploy the graph to edge = deploy the application. No build step, no compilation, no bundling.

More precisely:
1. The engine binary (Rust, compiled for the target architecture) is deployed once.
2. The graph (persisted as redb files or serialized format) is deployed/synced as data.
3. The engine loads the graph and begins evaluating.

This is analogous to the JVM (deployed once) + JAR files (deployed as data). The engine is the runtime; the graph is the program.

### 8.2 Minimum Engine Binary Size

Based on comparable Rust projects:

| Component | Estimated Size |
|-----------|---------------|
| Core graph (Node, Edge, indexes) | ~200KB |
| Evaluator (operation dispatch) | ~100KB |
| IVM engine | ~300KB |
| Cypher parser + planner | ~500KB |
| Capability enforcement | ~100KB |
| Persistence (redb) | ~500KB |
| CRDT sync | ~300KB |
| Total (stripped, LTO) | ~2-3MB |

For comparison:
- WasmEdge (WASM runtime): ~2MB
- SQLite (entire database engine): ~1.5MB
- Deno (JavaScript runtime): ~30MB
- Node.js (JavaScript runtime): ~40MB

A 2-3MB engine binary is deployable to:
- Edge nodes (Cloudflare Workers, Deno Deploy, Fly.io)
- IoT devices (Raspberry Pi, routers)
- Mobile apps (embedded engine)
- Browser (via WASM, though the WASM binary would be ~4-6MB)

### 8.3 WASM Deployment

The engine compiled to WASM enables:
- Browser-based instances (local-first applications)
- Edge workers (Cloudflare Workers support WASM)
- Offline-capable mobile apps

WASM has specific constraints:
- Single-threaded (no true concurrency, though SharedArrayBuffer exists)
- No direct filesystem access (must use virtual FS or IndexedDB)
- Memory limits (typically 4GB max)

The engine architecture should accommodate these constraints:
- Single-threaded evaluation mode (for WASM)
- Pluggable persistence backend (redb for native, IndexedDB for WASM)
- Bounded memory usage (essential anyway for edge deployment)

### 8.4 Sync as Deployment

With CRDT sync built into the engine, "deployment" can be:
1. Edge node syncs with the origin server
2. Origin server pushes new operation Nodes (the "code")
3. Edge node's engine picks up the new subgraphs
4. Next request evaluates the new handlers

No restart, no downtime, no deploy pipeline. The code IS data, and data syncs.

---

## 9. Synthesis: What This All Means for Benten

### 9.1 The Design Stance

After reviewing all precedents, the right design stance is:

**Operation Nodes are an ORCHESTRATION layer, not a COMPUTATION layer.**

They represent the flow of a request handler: "query this, check that, transform, respond." They do NOT represent fine-grained computation: "add these numbers, compare these strings, format this date."

This is the stance that makes the design work:
- Performance is fine (5-10 coarse Nodes, not 1000 fine Nodes)
- Expressiveness is sufficient (orchestration + ExternalOp for computation)
- The spaghetti problem is avoided (small graphs, not sprawling ones)
- Turing completeness is avoidable (bounded loops, no unbounded recursion)
- Self-modification is safe (scoped, validated, versioned)
- Debugging is tractable (5-10 steps to inspect, not thousands)

### 9.2 The Operation Node Taxonomy

Based on the analysis, the minimal set of operation Node types:

**Flow control:**
- `SequenceOp` -- NEXT_STEP chain (implicit, just follow edges)
- `ConditionalOp` -- TRUE_BRANCH / FALSE_BRANCH
- `MatchOp` -- Multi-way branch (CASE:x edges)
- `LoopOp` -- Bounded iteration over a collection (BODY edge, MAX_ITERATIONS)
- `TryCatchOp` -- Error handling (TRY / CATCH edges)
- `CallSubgraphOp` -- Invoke another operation subgraph

**Data operations:**
- `QueryOp` -- Read from graph (via IVM view or direct query)
- `CreateOp` -- Create Node/Edge
- `UpdateOp` -- Update Node properties
- `DeleteOp` -- Delete Node/Edge
- `TransformOp` -- Reshape data in context (map, filter, select, merge)

**I/O:**
- `ResponseOp` -- Construct HTTP response
- `ExternalOp` -- Execute WASM module (the Turing-complete escape hatch)

**Meta:**
- `ValidateOp` -- Validate data against a schema
- `CapabilityCheckOp` -- Explicit capability check (beyond the automatic enforcement)
- `EmitOp` -- Emit a reactive notification

That is 15 operation types. Each is a primitive the engine knows how to evaluate. Complex behavior is composed by connecting them with edges.

### 9.3 What This Document Does NOT Answer

- **Concrete Rust type definitions** for operation Nodes. (Needs a separate design document.)
- **The query language for QueryOp.** Is it Cypher? A subset? A custom DSL?
- **The transformation language for TransformOp.** JSONPath? JMESPath? A custom expression language?
- **The condition language for ConditionalOp.** Boolean expressions? The @benten/expressions evaluator adapted?
- **The WASM interface contract for ExternalOp.** Input/output format? Capability passing?
- **Performance benchmarks.** The estimates in Section 3 are theoretical. Benchmarking requires implementation.

These are follow-up explorations, each building on the foundation established here.

### 9.4 The Key Insight

The self-evaluating graph is NOT a visual programming language. It is NOT a workflow engine. It is NOT a graph rewriting system.

It is a **graph-native application runtime** where the application's behavior is stored in the same substrate as the application's data. The engine evaluates this substrate by walking it.

The precedents show this is feasible (Lisp, Smalltalk, LabVIEW, HVM prove it). They also show the pitfalls (spaghetti at scale, self-modification risks, Turing completeness dangers, debugging difficulty).

The design stance that avoids the pitfalls: **high-level orchestration nodes, bounded evaluation, WASM escape hatch, capability-mediated self-modification, version chains for auditability.**

The result is a system where:
- The code is introspectable (it IS the graph)
- The code is versionable (version chains)
- The code is syncable (CRDT sync moves code between instances)
- The code is securable (capabilities enforce what code can do)
- The code is modifiable (AI agents can write new handlers)
- The code is debuggable (step through Nodes, replay evaluations)
- The code is deployable (sync the graph to edge)
- The code terminates (bounded evaluation guarantees)

This is the foundation.

---

## Sources

### Precedent Research
- [Graph Rewriting Systems - Wikipedia](https://en.wikipedia.org/wiki/Graph_rewriting)
- [Interaction Nets and Lambda Calculus](https://hal.science/inria-00133323)
- [Hypergraph Rewriting - Wolfram Institute](https://wolframinstitute.org/research/hypergraph-rewriting)
- [Homoiconicity - Wikipedia](https://en.wikipedia.org/wiki/Homoiconicity)
- [Smalltalk Image-based Development](https://peerdh.com/blogs/programming-insights/smalltalk-image-based-development)
- [Smalltalk and AI Agents](https://volodymyrpavlyshyn.medium.com/smalltalk-the-language-that-changed-everything-and-why-it-still-matters-for-ai-agents-8c3f1bf50c1d)
- [Lessons in Software Evolution from Smalltalk](https://link.springer.com/chapter/10.1007/978-3-642-11266-9_7)
- [LabVIEW Data Flow Programming](https://control.com/technical-articles/data-flow-programming-in-labview/)
- [Benefits of Graphical Programming in LabVIEW](https://www.ni.com/en/shop/labview/benefits-of-programming-graphically-in-ni-labview.html)
- [Dataflow Programming - Devopedia](https://devopedia.org/dataflow-programming)

### Performance and Compilation
- [HVM2 Paper - Parallel Evaluator for Interaction Combinators](https://raw.githubusercontent.com/HigherOrderCO/HVM/main/paper/HVM2.pdf)
- [HVM Benchmarks](https://gist.github.com/VictorTaelin/47a1379a4ad2729417e2dc0210f79cf4)
- [Bend - Massively Parallel Programming Language](https://github.com/HigherOrderCO/Bend)
- [Virtual Machine Dispatch Experiments in Rust](https://pliniker.github.io/post/dispatchers/)
- [Cranelift JIT Compiler](https://cranelift.dev/)
- [Cranelift E-Graph Optimization](https://github.com/bytecodealliance/rfcs/blob/main/accepted/cranelift-egraph.md)
- [V8 Maglev JIT](https://v8.dev/blog/maglev)
- [V8 Leaving the Sea of Nodes (Turboshaft)](https://v8.dev/blog/leaving-the-sea-of-nodes)
- [Blueprint Performance Guidelines](https://intaxwashere.github.io/blueprint-performance/)
- [C++ vs Blueprints - 2026 Guide](https://www.wholetomato.com/blog/c-versus-blueprints-which-should-i-use-for-unreal-engine-game-development/)

### Graph Execution and Workflow Engines
- [ComfyUI Technical Deep Dive](https://medium.com/@mucahitceylan/comfyui-a-technical-deep-dive-into-the-ultimate-stable-diffusion-workflow-engine-df1a7db3f7f5)
- [ComfyUI Graph Execution and Caching](https://deepwiki.com/hiddenswitch/ComfyUI/4.2-graph-execution-and-caching)
- [Temporal Workflow Engine Guide](https://www.kunalganglani.com/blog/temporal-workflow-engine-guide)
- [Temporal Performance Bottlenecks](https://docs.temporal.io/troubleshooting/performance-bottlenecks)
- [DAGRS - Rust DAG Task Orchestration](https://github.com/dagrs-dev/dagrs)
- [SubMicro Trading System - 890ns Latency](https://submicro.krishnabajpai.me/)

### Visual Programming and Scaling
- [Scaling Up Visual Programming Languages (IEEE)](https://ieeexplore.ieee.org/document/366157/)
- [On the Limits of Visual Programming Languages](https://www.researchgate.net/publication/220630729_On_the_limits_of_visual_programming_languages)
- [Visual Programming Pros and Cons 2026](https://www.weweb.io/blog/visual-programming-what-it-is-types-pros-cons)
- [Unity Shader Graph Performance](https://blog.s-schoener.com/2024-11-17-unity-shader-graph-perf/)

### Turing Completeness and Termination
- [Datalog Termination Guarantees - Wikipedia](https://en.wikipedia.org/wiki/Datalog)
- [Souffle Datalog for Static Analysis](https://souffle-lang.github.io/tutorial)
- [Total Functional Programming - Turner 2004](https://www.jucs.org/jucs_10_7/total_functional_programming/jucs_10_07_0751_0768_turner.pdf)
- [Turing Completeness - Wikipedia](https://en.wikipedia.org/wiki/Turing_completeness)
- [Self-Modifying AI Risks - ISACA 2025](https://www.isaca.org/resources/news-and-trends/isaca-now-blog/2025/unseen-unchecked-unraveling-inside-the-risky-code-of-self-modifying-ai)

### Edge and WASM Deployment
- [WasmEdge Runtime](https://wasmedge.org/)
- [WebAssembly at the Edge - Stealth Cloud](https://stealthcloud.ai/cloud-paradigms/wasm-edge-computing/)
- [Rust and WebAssembly 2025](https://observabilityguy.medium.com/why-rust-is-quietly-ruling-webassembly-in-2025-536bf4709aa6)
- [State of WebAssembly 2025-2026](https://platform.uno/blog/the-state-of-webassembly-2025-2026/)
- [Spreadsheet Dependency Graphs - HyperFormula](https://hyperformula.handsontable.com/guide/dependency-graph.html)
- [Reactive Graph Programming](https://blog.machinezoo.com/transparent-reactive-programming)

### Compiler IR and Graph Representations
- [Sea of Nodes - Wikipedia](https://en.wikipedia.org/wiki/Sea_of_nodes)
- [Cliff Click - A Simple Graph-Based IR (1995)](https://www.oracle.com/technetwork/java/javase/tech/c2-ir95-150110.pdf)
- [TurboFan JIT - V8](https://v8.dev/blog/turbofan-jit)
- [An Algebraic Theory of Graph Reduction - JACM](https://dl.acm.org/doi/10.1145/174147.169807)
- [Graph Rewriting Semantics for Functional Languages](https://link.springer.com/chapter/10.1007/3-540-63172-0_35)
- [Delta-Nets: Optimal Parallel Lambda Reduction (2025)](https://arxiv.org/html/2505.20314v1)
