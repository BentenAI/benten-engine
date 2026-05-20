# How It Works

A plain-English tour of Benten — what it is, what problem it solves, how the pieces fit together, and where the project is going.

---

## The idea

Software is usually built as three separate things: a database, an application, and a set of queries that go between them. Each has its own language, its own representation, its own runtime. When you want to change something, you're changing across three layers.

Benten collapses that. The database and the application are the same graph. A handler — the thing you'd usually write as application code — is a subgraph of operation Nodes that the engine reads and walks. When a request comes in, the engine reads the subgraph, executes the operations, reads and writes data, and returns a result. The handler, the data it operated on, the audit trail of what happened, and any capabilities that gate the access all live in the same content-addressed graph.

Because everything is content-addressed, two machines that have the same graph state agree bit-for-bit on what everything is. Sync is content exchange, not schema reconciliation. Because handlers are data, an AI agent can compose or fork them the same way it would edit any other Node.

## Concepts, in the order they're useful

**Node.** The basic unit. A Node has a label (`post`, `user`, `capability-grant`), properties (key-value pairs), and a CID — a content identifier derived from its bytes. Two Nodes with identical content have the same CID.

**Edge.** A typed directional link between two Nodes. Labels like `NEXT`, `ON_ERROR`, `GRANTED_TO`, `CURRENT` live on edges and carry the semantics of control flow, error routing, authorization, and version pointers.

**Primitive.** One of the 12 operations the engine knows how to execute:

```
READ      WRITE     TRANSFORM    BRANCH     ITERATE    WAIT
CALL      RESPOND   EMIT         SANDBOX    SUBSCRIBE  STREAM
```

Each corresponds to an Operation Node with its own property shape. `READ` reads a Node. `WRITE` writes one. `TRANSFORM` runs a pure expression. `BRANCH` routes on a condition. `CALL` invokes another subgraph. `RESPOND` is a terminal that produces the handler's final output. The 12 primitives are the complete vocabulary; anything an application wants to do is composed from these.

All 12 primitives have live executors at tag `phase-2b-close` (2026-05-03). WAIT shipped in Phase 2a; SANDBOX, STREAM, and SUBSCRIBE shipped in Phase 2b. The earlier eight (READ, WRITE, TRANSFORM, BRANCH, ITERATE, CALL, RESPOND, EMIT) shipped in Phase 1.

Phase 3 added a **typed-CALL** dispatch surface on top of the existing CALL primitive (NOT a 13th primitive — the 12-primitive commitment holds). When a CALL Node's `target` starts with the reserved `engine:typed:` prefix, the eval-side dispatch fork routes through the typed-CALL registry — a closed set of 10 engine-known ops (Ed25519 sign / verify, BLAKE3 hash, multibase encode/decode, DID resolve, UCAN chain validation, VC verify, keypair generation) needed by the Atrium / UCAN / DID story. See [`TYPED-CALL.md`](TYPED-CALL.md) for the engineer-facing reference. SANDBOX host-fns are intentionally narrow (`time` / `log` / `kv:read` / `random` only) — see [`HOST-FUNCTIONS.md`](HOST-FUNCTIONS.md). Engine-known fixed-shape compute belongs in typed-CALL.

**Subgraph.** A DAG of Operation Nodes wired with control-flow Edges. The engine walks this DAG to execute a handler. Because it's a DAG with bounded fan-out, execution is always bounded; the engine is not Turing complete by construction.

**Handler.** A subgraph registered with the engine under a name. `crud('post')` produces a handler with five actions — create, get, list, update, delete — each action its own walk through the subgraph.

**View.** A materialized query result kept current by subscribing to graph changes. A list-by-label read hits a view; it's O(1) because the view is already computed. Views are regular Nodes that an internal subscriber advances on every ChangeEvent.

**Capability.** A grant — "this actor may write Nodes matching this pattern" — stored as a Node with a `GRANTED_TO` edge. Writes hit a pre-write policy hook; the default policy (`NoAuth`) allows everything; a `GrantBacked` policy checks the grant graph. Phase 3 added a durable `UCANBackend` over `benten-id`'s claim envelope + chain validation surface — UCAN grants attenuate on delegation, propagate revocations, and validate `nbf`/`exp` time-windows at chain-walk time with constant-time signature comparison.

**Anchor + Version + CURRENT.** When you need history (undo, audit, time-travel), you opt into the version-chain pattern: an Anchor Node with stable identity points at the latest Version Node via a CURRENT edge. Version Nodes are immutable once written; updating is "write a new Version, advance CURRENT atomically." Ephemeral data doesn't pay the versioning cost.

## What happens when you call a handler

A concrete walk-through. You've registered `crud('post')`. Now:

```typescript
await engine.call(handler.id, 'post:create', { title: 'Hello', body: 'Works.' });
```

Here's what the engine does:

1. **Locate the subgraph.** The handler's CID points at a registered subgraph. The engine reads it — reading a subgraph is reading a Node.
2. **Find the action.** `post:create` is an entry point on the subgraph. The evaluator starts at the Operation Node bound to that entry.
3. **Walk.** The evaluator steps through Operation Nodes one at a time. For `create`, the flow is TRANSFORM (build the Node from your input) → WRITE (persist it) → RESPOND (return the CID).
4. **At each WRITE, check the capability hook.** The pre-write hook consults the policy. `NoAuth` approves; `GrantBacked` walks the grant graph. Denial terminates the walk with `E_CAP_DENIED`.
5. **Commit the transaction.** WRITEs are wrapped in a storage transaction (ACID via redb). On commit, content is hashed into CIDs, the audit log is advanced, and ChangeEvents fire.
6. **Update views.** The IVM subscriber sees the ChangeEvents and updates any views whose subscription patterns match. `post:list` will see the new Node on the next read.
7. **Return.** RESPOND's value is returned to the caller.

This is the same code path for the native Rust API, the napi-rs TypeScript wrapper, and eventually a WASM-hosted variant. The engine is agnostic to how the call arrived.

## Why content addressing is load-bearing

Two Benten machines with the same set of Nodes agree bit-for-bit on what everything is. A Node's CID is a hash of its canonical DAG-CBOR bytes; byte-identical content yields byte-identical CIDs. This has consequences:

- **Sync is content exchange.** Phase 3 shipped Atrium peer-to-peer sync (iroh QUIC transport + Merkle Search Tree diff + Loro CRDT merge): two peers synchronizing don't reconcile schema-level changes; they exchange content-addressed Nodes and the receiver cryptographically verifies what arrived. Browser engines participate as thin-client views *into* full peers (laptop / phone-OS app / desktop) over a fetch/POST + SSE protocol rather than running iroh / Loro themselves; full peers are the sync participants.
- **Handlers are referenceable by hash.** A subgraph registered under `crud:post` has a CID. Forking it points at a different CID. Sending a handler across the network lets the receiver verify it matches what you claimed before running it.
- **Audit trails are free.** Every write produces a content-hashed change record; the history is the graph.
- **Dedup is automatic.** Two writers producing identical content produce a single Node.

Content addressing also shapes security. The default read posture returns `null` for both "not found" and "access denied" — a byte-identical response, so an unauthorized caller cannot distinguish existence from permission by probing CIDs. Explicit diagnostic methods gated behind a debug capability exist for operators who need to distinguish the two.

## Operator surfaces: caps + observability

Capability administration is reached via `engine.caps()`, which returns an `EngineCapsHandle` exposing `create_principal`, `grant_capability`, `revoke_capability`, `install_ucan_proof`, and a `create_view` helper for materialized-view registration. Each method routes through the same pre-write capability policy the evaluator uses, so administrative grants and runtime checks see one consistent grant graph; revocation is durable and propagates through the cap-recheck cascade on the next read.

Operational observability lives next to it: `engine.metrics_snapshot()` returns a `BTreeMap<String, f64>` of counter / gauge / histogram-summary metrics covering sync hop depth, SANDBOX cold-start latency, view-materialization budget consumption, cap-recheck cache hit/miss counts, and the Phase-3 sync-receive failure counters (`E_SYNC_FORGED_DEVICE_ATTESTATION`, `E_SYNC_REVOKED_DURING_SESSION`, etc.). The map shape is intentionally flat so it composes cleanly with Prometheus / OpenTelemetry exporters built on top by an operator-side plugin.

## Plugins, in plain English

When a software platform talks about "plugins," it usually means binary blobs the platform loads at runtime — JS bundles, Python modules, Rust dynamic libraries. Benten's plugins are different: a plugin is just **more graph**.

Concretely: a plugin is a *subgraph* — a bundle of operation Nodes (handlers, materializers, the SANDBOX nodes those use, edges connecting them) packaged for sharing. You install a plugin the same way you install any other subgraph: it gets a CID, the user grants it capabilities, and the engine's evaluator walks it when its handlers are called. There is no separate plugin runtime. There is no JS loader, no FFI bridge, no embedded interpreter. The same evaluator that runs your `crud('post')` handler runs the plugin's handlers.

Two things follow from this:

**Plugins are content-addressed and shareable through Atriums.** A plugin's CID is the cryptographic hash of its bytes. Two users running "the same plugin" mean it bit-for-bit. You can publish a plugin into your Atrium peer group; someone in the group can install it and verify they got what you sent. There is no central plugin registry — discovery is peer-to-peer, the same way your data is.

**Each plugin has its own identity — actually four distinct identity concepts that don't get conflated.** Phase 4-Foundation ratified the four-identity-concepts model (CLAUDE.md baked-in #18 "Implementation refinements ratified 2026-05-11"). They are:

1. **Content-CID** — what the plugin IS, bit-for-bit. Two users running "the same plugin" mean it bit-for-bit because the CID is the cryptographic hash of the plugin's bytes.
2. **Peer-DID signature on the original content** — provenance: who originally authored or shared this content. Verifiable; `benten-id` RotationLog handles peer-DID key rotation.
3. **Plugin-DID** — minted fresh **by the caller** (admin UI / installer code in your engine) at install time, then pre-inserted into the engine's `PluginDidStore` BEFORE `install_plugin` runs (caller-mint-first contract per `docs/PLUGIN-MANIFEST.md §3`). A `did:key:...` shape with a fresh Ed25519 keypair the caller-side install code holds + zeroizes on drop. Plugin-DID acts as the UCAN audience for caps you grant the plugin AND as a constrained issuer if the plugin re-delegates to other plugins within its manifest envelope. (The engine NEVER mints plugin-DIDs internally — the Ed25519-derives-DID-from-public-key property + Step 8 of install_plugin's two-pin enforcement make this structurally adversary-resistant.)
4. **User-DID** — your trust anchor. You sign install records (committing your consent locally) and issue UCAN caps with `audience=plugin-DID`.

When you install a plugin, your engine grants the plugin-DID an attenuated UCAN — a capability token that says "this plugin can do these things on my behalf, but not more." The engine checks that token on every read and write the plugin's subgraph triggers. Plugin-DID has no inherent authority — its issuance is bounded by what your manifest envelope allows + the chain must still trace back to a user-DID-issued root grant.

**Cross-plugin and cross-schema references use content-CID, not author-DID.** A plugin that says `accepts_content: [<schema_cid>, ...]` is naming the bytes it will accept; key rotations don't break these references.

How does the user stay in control without being prompted on every action? **A two-step consent model:**

1. **Install-time manifest.** Every plugin ships a manifest with two halves:
    - **Requires:** caps the plugin needs to function ("read my notes," "write to a sandbox", "use the time host-fn").
    - **Shares:** policy for what *other* plugins it will hand caps to ("any AI assistant," "plugins from this author," "none").
   Both halves are signed by the plugin author. The user reviews the manifest at install time and either consents to the envelope or declines.

2. **Runtime delegation inside the envelope.** Once installed, plugins can delegate caps to each other within their manifest's `shares` policy without further user prompts. If your AI assistant plugin needs to read your calendar plugin's data, the calendar's manifest decides whether the AI gets a cap; the engine validates the chain. The user is involved at install, not on every access.

This is the shape that makes Phase-6 AI assistants work. An AI assistant declares in its manifest "I integrate with calendar, notes, email"; the user reviews and consents at install; the assistant runs autonomously across all three without further dialog boxes. It is also the shape that survives Phase-8 decentralized plugin discovery: plugins are signed by their author and content-addressed, so users trust the *manifest signature* directly rather than relying on a central registry to police anything.

There is a second, rarer category: **engine extensions** — Rust crates compiled into the engine binary. These are for things like a custom persistence backend or a new transport — extensions that change the engine itself. They have no UCAN, no manifest, no install flow; you compile them in or you don't. They are for platform builders, not app users. The two categories — app-level plugins (subgraphs, content-addressed, sharable) and engine-level extensions (Rust crates, compile-time, trusted-by-build) — stay deliberately separate.

The plugin-manifest schema and install / upgrade / share flows land in Phase 4-Foundation (Benten Platform v1 foundation). The engine surface that backs them — the evaluator's principal-aware read path — shipped at PR #184 in the pre-v1 cleanup window, since it's independent of the manifest decisions. Today the engine has the foundational pieces (DID + UCAN + content-addressed subgraphs + principal-aware read surface); the manifest schema layers on top in Phase 4-Foundation. See [`PLUGIN-MANIFEST.md`](PLUGIN-MANIFEST.md) for the full schema spec.

**A plugin is a workflow is a subgraph.** They're the same shape; manifest presence is what distinguishes a plugin from a workflow. Promoting a workflow to a plugin = adding a manifest. The plugin library subgraph holds all your installed versions + forks (content-addressed; cheap); your active graph holds references to specific plugin-versions you're using now; switching active version = updating the reference. Composition is recursive: meta-plugins reference sub-plugins. Versioning extends the Phase-1 anchor + Version Node pattern to DAG-shape (forks land on branches; CURRENT can point at any branch tip; per-device-local).

**Update model is pull-not-push.** Receiver-controlled; the engine never auto-pulls a plugin update. Updates are content-addressed; if the CID changes, the shape changed; if `requires` GREW, full re-consent fires; otherwise silent. Plugin discovery in Phase 4-Foundation v0 is direct content-addressed-share over Atriums (out-of-band handshake; user pulls from a peer they trust). Decentralized self-discovered registry is Phase 4-Meta scope per `docs/future/phase-4-backlog.md §3.1`.

**Admin UI v0 is the first plugin.** Phase 4-Foundation ships an admin UI that is itself a content-addressed shareable subgraph installed via signed manifest envelope. See [`ADMIN-UI.md`](ADMIN-UI.md) for the user flows + the 4-category navigation IA (Plugins / Workflows / Content Types / Views) over the unified subgraph substrate.

**Key rotation** has an MVP at Phase 4-Foundation (`SelfRevocation` attestation + out-of-band new-key trust). A richer decentralized-identity-and-attestation substrate ("**Kith**" working name; Phase 5+ exploratory) would supersede the MVP if it lands; scaffold at [`docs/future/kith-decentralized-identity.md`](future/kith-decentralized-identity.md). Phase 4-Foundation does not depend on Kith — the MVP rotation suffices for v1.

## What the engine is not

- **Not Turing complete.** Every handler terminates. The escape hatch for genuine compute — arbitrary TypeScript, ML inference, an image resize — is SANDBOX (WASM via wasmtime, fuel-metered).
- **Not a relational database.** You can model relational shapes in a graph, but the engine isn't optimized for row-oriented scans across large tables. Read patterns that scale are the ones that hit IVM views.
- **Not an app framework.** It's the engine underneath one. You write handlers in TypeScript; the DSL produces registered subgraphs. A CMS, a chat service, a personal assistant runs on top.
- **Not finished.** Phase 1, 2a, 2b, and 3 shipped at named tags; the full 12-primitive vocabulary is live and Atriums (peer-to-peer sync over iroh + Loro) are real. The current state is usable for local single-process work, for multi-device sync between full peers a single user owns, and for evaluating whether the model fits a problem you have. The Benten Engine v1 milestone gate is a deliberate post-Phase-3 PAUSE-AND-ASSESS step.

## The path from here

**Phase 1 (shipped 2026-04-21)** gave the engine a working 8-primitive evaluator, content-addressed storage with MVCC, hand-written IVM views, a pluggable capability policy, TypeScript bindings, a scaffolder, and debug tooling. The canonical fixture CID `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` is stable across Linux / macOS / Windows.

**Phase 2a (closed at tag `phase-2a-close`, 2026-04-25)** finished the evaluator. Added WAIT (suspend/resume with DAG-CBOR persisted state). Completed structural invariants 8 + 11 + 13 + 14: system-zone enforcement at runtime, multiplicative iteration budgets across CALL/ITERATE nesting, immutability rejection for non-dedup writes, causal attribution threaded through every trace step. Hardened capability TOCTOU with wall-clock revocation checks on a dual monotonic + HLC clock source. Shipped the 4-step resume protocol that guards against tampered state, principal mismatch, stale subgraph references, and mid-eval grant revocation.

**Phase 2b (closed at tag `phase-2b-close`, 2026-05-03)** added the WASM SANDBOX (wasmtime-backed, fuel-metered, capability-manifested host functions), the STREAM and SUBSCRIBE primitives (user-visible back-pressured output and reactive change notifications), and Algorithm B production-registered with per-view strategy selection. Algorithm B's non-canonical-view-ID generalization (user-defined view IDs declaring `Strategy::B`) was finished in Phase 3.

**Phase 3 (closed at tag `phase-3-close`)** added P2P sync as **Atriums** — durable UCAN capability chains over `benten-id` (DID-based identity, Ed25519 envelopes, claim-chain validation), iroh QUIC transport, Merkle Search Tree diff, Loro CRDT merge, and HLC timestamps. The native crate count grew from 8 to 10 with `benten-id` (9th, identity + claims) and `benten-sync` (10th, sync runtime — native-only, excluded from wasm32 targets). Algorithm B was generalized so user-defined view IDs run under `Strategy::B` with their actual label patterns rather than being coerced to `ContentListingView` semantics. Browser engines participate as thin-client views into full peers via a fetch/POST + SSE protocol (no iroh / Loro / SANDBOX in the wasm32 bundle), with optional IndexedDB cache for snapshot data. After Phase 3, two Benten instances hold the same graph and exchange Nodes cryptographically — the first configuration where "Benten communities" become real.

**Phase 4-Foundation (closed at tag `phase-4-foundation-close`, 2026-05-14)** shipped the substantive Benten Platform engineering — the layer that makes the engine end-to-end usable through a UI rather than only through the napi/TS surface. It added the admin UI v0 (the first plugin, default module, and Foundation entrypoint) on a 4-category navigation IA (Plugins / Workflows / Content Types / Views), the full plugin manifest format with install-time consent + per-plugin DID minting + manifest envelope chain validation + private-namespace caps + DAG-shape versioning, the decentralized self-discovered registry on top of Atriums, the schema-driven rendering pipeline (schemas as subgraphs of typed-field Nodes — 8-label vocabulary `FieldScalar` / `FieldEnum` / `FieldUnion` / `FieldList` / `FieldMap` / `FieldObject` / `FieldRef` / `SchemaRoot`), the materializer pipeline (composed via IVM-subgraph generalization), and the module ecosystem tooling at scale. The workspace grew from ten to twelve crates with `benten-platform-foundation` (11th — schema-rendering compiler + materializer + plugin manifest + admin UI v0 + `Renderer` trait abstraction; the v1 platform-shippable surface) and `benten-renderer-tauri` (12th — Tauri 2.x renderer engine extension per CLAUDE.md baked-in #19; embedded-webview deployment shape (c) on top of the same wasm32 bundle as the browser tab). Three deployment shapes are first-class: (a) **full peer** — native Rust on user hardware, full Atrium sync participation; (b) **thin compute surface** — wasm32 in a browser tab or edge worker, stateless reads against snapshot data + writes via fetch to a full peer; (c) **embedded webview** — native shell wraps the same wasm32 bundle, talks to its embedded full peer via in-process IPC. **Phase 4-Meta** then layers self-composing admin meta-circular work + Phase-3-deferred items + the v1-assessment-window on top; the `v1` tag follows.

**Phase 4-Meta-Core (in flight)** lifts the v1-gate substrate that the Principal primitive's confidentiality half depends on (per CLAUDE.md baked-in #18). The **storage-partition seam** (issue #989) is the first canary: `WriteContext::namespace_did: Option<Cid>` carries a per-DID storage scope across the `GraphBackend::put_node_with_context` surface, and `RedbBackend::scoped(did)` returns a per-DID view that structurally confines reads to that partition's keyspace. The C1 invariant — **a write under DID-A is not visible under DID-B** — holds end-to-end across the storage layer and across the post-commit change-subscriber fan-out. `BrowserBackend` (the thin-client cache for deployment shapes (b)/(c) per baked-in #17) does not yet implement an in-RAM partition; it fails CLOSED on `Some(namespace_did)` with `GraphError::NamespacedWriteUnsupported` rather than silently dropping the scope. The shape lands in Phase 4-Meta-Core (before the v1-API freeze) precisely because it is a public `WriteContext` change — adding it post-`v1` would be SemVer-breaking. **It is the structural hook that the encryption substrate (#1301) plugs into**: capability-gating binds only a cooperating engine, so a decentralized personal-data platform whose vision (Phases 7-8 untrusted-host / peers-hold-ciphertext / Kith selective-disclosure) requires per-principal confidentiality on hardware *other* principals control needs encryption sealing each per-DID partition. Storage-partition (authority half, here) + per-partition encryption (confidentiality half, #1301) together compose the multi-tenant substrate that v1-beta tags on.

**Phases 5–8** are applications composed on top of the v1 platform: a CMS migration that exercises the engine under realistic load (Phase 5 — first reference application), a Personal AI Assistant MVP (MCP, PARA knowledge organization, on-demand tool composition), community spaces, and a USD-pegged currency for the network's economic layer.

Each phase layers on the previous without requiring changes below. The engine's abstractions were designed so that Phase 3 sync doesn't need Phase 1 changes; Phase 6 AI composition doesn't need Phase 2 changes.

## Where to go from here

- **Try it.** See [`QUICKSTART.md`](QUICKSTART.md) for the 10-minute path. `npx create-benten-app my-app` gives you a scaffolded project with a `crud('post')` handler.
- **Understand the architecture.** [`ARCHITECTURE.md`](ARCHITECTURE.md) walks the twelve crates (eight foundational + `benten-id` + `benten-sync` + `benten-platform-foundation` + `benten-renderer-tauri`), the invariant set, the storage layer, and the evaluator's request flow.
- **Read the error catalog.** [`ERROR-CATALOG.md`](ERROR-CATALOG.md) is the stable contract: every error the engine surfaces, by discriminant, with context.
- **Read the typed-CALL reference.** [`TYPED-CALL.md`](TYPED-CALL.md) covers the Phase-3 typed-CALL dispatch surface — the 10 engine-known ops + their cap requirements + the SANDBOX-vs-typed-CALL decision tree.
- **Look at the glossary.** [`GLOSSARY.md`](GLOSSARY.md) names the concepts above and a few more.
- **Read the source.** The engine is deliberately small. `crates/benten-engine/src/lib.rs` is the integration surface; `crates/benten-eval/src/evaluator.rs` is the walk loop; `crates/benten-graph/src/redb_backend.rs` is the storage. If the docs are confusing, the code is the ground truth.
