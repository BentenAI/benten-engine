# Benten Engine — A Primer

Benten is a runtime that treats data, code, history, and authorization as a single content-addressed graph. The project's headline is *"everything is a graph; materialized as anything"* — a backend, a frontend, an AI assistant, a community, all expressed in the same model and processed by the same engine.

This document explains the model, the structural choices that follow from it, and what those choices make possible. It assumes general familiarity with how computers store and exchange information but no specific background in software engineering.

## The model

A graph is a way of writing down relationships. It has two kinds of element: **nodes**, which represent things — a person, a post, a recipe, a calendar event — and **edges**, which represent relationships between two nodes and carry a label describing the relationship. A pair of edges might say *Ben — friend — Catherine* and *Ben — hobby — Woodworking*. Three nodes, two relationships, and an arbitrary slice of someone's social and personal life.

Almost any relational information can be written this way. Catherine has her own collection of nodes and edges describing her life; so does everyone else. Conceptually, those individual collections are subgraphs of a single hypothetical larger graph that contains every relationship anyone has ever recorded. Each person's view is a slice of that whole — the part they hold themselves, the part they've chosen to share, the part others have chosen to share with them.

Most software built on top of graph-shaped data treats the graph as passive: it's stored in a database, and separate application code performs operations on it — reading nodes, writing them, following edges, computing aggregates. The application code lives in a different layer, written in a different language, deployed on different infrastructure. Keeping the two layers consistent is the source of an enormous amount of accidental complexity: schema migrations, ORMs, query languages, caching layers, audit logs that have to be maintained alongside the data they describe, authorization systems that are themselves a third separate world.

Benten collapses that separation by representing application code as part of the same graph. A workflow — what a conventional system would call a function or an endpoint — is a small subgraph of operation nodes connected by control-flow edges. Operation nodes are drawn from a fixed vocabulary of twelve primitives:

- **READ** retrieves a node.
- **WRITE** persists one.
- **TRANSFORM** computes a value from existing values without side effects.
- **BRANCH** routes execution conditionally.
- **ITERATE** repeats a sub-workflow over the items of a collection.
- **WAIT** suspends execution until a signal or timeout.
- **CALL** invokes another registered workflow.
- **RESPOND** terminates the workflow with a result.
- **EMIT** announces a change without waiting for a response.
- **SANDBOX** runs a sealed-off WebAssembly computation under a strict resource budget.
- **SUBSCRIBE** reacts to changes that match a pattern.
- **STREAM** produces output incrementally.

These twelve are the complete vocabulary; anything an application needs to do is composed from them. A workflow that creates a blog post is a small subgraph: TRANSFORM the input into a node shape, WRITE it to storage, RESPOND with its identifier. A registered workflow is called a **handler**, and the engine identifies it by name — `post:create`, `crud:post`, and so on. Calling a handler asks the engine to walk its subgraph from the entry node, executing each operation in order.

Several useful properties follow from making code part of the graph. Because the operation vocabulary is bounded, every workflow is guaranteed to terminate within a knowable resource envelope; arbitrary unbounded computation lives behind the SANDBOX primitive and is fuel-metered. Because the engine is implemented in Rust, workflows execute close to the speed of equivalent hand-written native code — yet because each step is a node the engine can introspect, the same workflow can be visualized as a diagram, traced step by step, or composed and edited by an AI assistant the same way it would edit any other content. Every node, including every operation node and every registered handler, has a deterministic content-addressed fingerprint: identical content yields an identical identifier on every machine, with no coordination required. Authorization is itself part of the graph — a permission is a node, granting a permission is writing an edge, revoking it is writing another — so the question "who is allowed to do what" lives in the same world as the data it governs, with the same audit guarantees.

## Sharing and trust

The engine is designed for a setting where each user holds their own graph on their own devices. Most of what's stored is private. Some of it is shared by mutual agreement with specific others; the *Ben — friend — Catherine* edge, for instance, is part of both people's graphs, present in each because both have chosen to record it.

Sharing in this model is not publishing to a central service. Two engines synchronize peer-to-peer by exchanging the content-addressed nodes they've agreed to share. Each side cryptographically verifies what it receives — the content's fingerprint must match the identifier the sender claimed — and resolves any concurrent edits using data structures that merge cleanly without coordination. There is no platform sitting between the two participants; the agreement *is* the shared content, and either side can stop participating at any time without involving anyone else.

Authorization across this distributed setting uses the same mechanism authorization uses locally: capability nodes with edges naming the principals they're granted to. A grant says "this principal may access this scope of content"; revoking it writes a counter-record that the engine consults at every sensitive operation. Identity itself is cryptographic — a principal is identified by a public key, and every shared edit is signed — so impersonation requires forging a signature rather than compromising a central account. A community in this model is a group of users whose graphs overlap because they've agreed to share specific content with each other; the community's rules, membership, and history are themselves part of the shared subgraph, replaceable by the same kind of operations that maintain everything else.

## What this enables

A few specific futures the project is reaching for, each made tractable by the unified model:

- **Personal AI assistants.** An assistant that runs against the user's own graph can read their notes, calendar, and conversations directly, and can compose new handlers to perform new tasks without requiring a software update or sending data to an external service.
- **Communities without platforms.** A group can hold shared records, governance, and history collectively, with each member running an engine on their own machine and synchronizing peer-to-peer. No platform owns the data or controls the rules.
- **Forkable applications.** A CMS, chat tool, or knowledge base is a few hundred handlers on top of the engine. Forking is cloning the relevant subgraph, editing the operation nodes, and registering the result under a new fingerprint; users can verify which fork they're running and switch between alternatives freely.
- **Verifiable provenance.** Because every handler and every shared piece of content has a fingerprint and a signed history, recipients can independently verify what they received before acting on it — without needing to trust an intermediary.
- **An economic layer.** Later phases extend the model to peer-to-peer compute and storage markets, with payments and contracts represented as graph operations and a price-stable token settling them.

## Status

The engine is built in phases, each of which stays usable as later phases build on top. **Phase 1** shipped in April 2026 and provides the core engine: eight of the twelve primitives, content-addressed storage, materialized views that stay current automatically, a pluggable capability system, TypeScript bindings, and a project scaffolder. **Phase 2a** (closed April 2026 at tag `phase-2a-close`) completed the evaluator with the WAIT primitive and hardened several integrity properties: every step now records the principal that initiated it, mid-workflow capability revocations are detected, and the engine's internal state is protected from user code. **Phase 2b** (closed May 2026 at tag `phase-2b-close`) added the remaining three primitives — SANDBOX, STREAM, SUBSCRIBE — bringing all 12 primitives to production-runtime LIVE, and added a more flexible view system that supports user-registered queries. **Phase 3** (closed May 2026 at tag `phase-3-close`, pending) added the peer-to-peer sync layer (Atriums over iroh QUIC + Merkle Search Tree diff + Loro CRDT merge) and decentralized identity (DIDs, Ed25519 envelopes, durable UCAN capability chains). Browser engines participate as thin-client views into full peers (laptop / phone-OS app / desktop) rather than running the sync stack in-bundle, with optional IndexedDB caching for snapshot data. The native crate count grew from eight to ten with `benten-id` and `benten-sync` (the sync runtime is native-only, excluded from wasm32 targets). The project now pauses at a deliberate post-Phase-3 PAUSE-AND-ASSESS step to determine what (if anything) gates a Benten Engine v1 release. Subsequent phases compose applications on top of the engine: a CMS migration, a platform layer, a personal AI assistant, community spaces, and the economic layer.

## Limitations and further reading

Benten is not Turing-complete by design — every workflow terminates, with the SANDBOX primitive available as the bounded escape hatch for genuinely open-ended computation. It is not a relational database; reads that scale are reads that hit pre-computed views, not arbitrary scans across large tables. It is not a full application framework; it is the runtime underneath one. And it is not finished: the model and the implementation will continue to evolve past the post-Phase-3 v1-milestone-gate.

For the working version, see [`QUICKSTART.md`](QUICKSTART.md) — ten minutes from `npx` to a green test. For the technical structure of the engine itself — the ten native crates, the structural invariants, the storage layer, the request flow — see [`ARCHITECTURE.md`](ARCHITECTURE.md). [`GLOSSARY.md`](GLOSSARY.md) defines the precise terminology, and [`ERROR-CATALOG.md`](ERROR-CATALOG.md) catalogues every error the engine surfaces. The engine is small; if any of these documents are unclear, the source is the ground truth.
