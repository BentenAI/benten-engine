# Exploration: Personal AI Assistant (Phase 6 scoping)

**Created:** 2026-04-14
**Status:** ACTIVE RESEARCH. Scoping doc for Phase 6 committed work. Full spec emerges during Phase 6 pre-work (ADDL process) — this is a thinking document, not a specification.
**Audience:** Anyone planning Phase 6, reviewing the adoption strategy, or writing specs that Phase 6 depends on.

---

## The Vision (Pillar 2 from VISION.md)

The Personal AI Assistant is the adoption driver. The pitch to end users is not "try our graph database" — it's:

> **Stop paying for ten pieces of software. One AI assistant running on hardware you trust organizes your life, generates the tools you need, and keeps full audit trails of every action.**

Replaced by the assistant + generated tools:
- Notion / Roam / Obsidian (knowledge base)
- ChatGPT / Claude subscription (conversational AI)
- Zapier / Make (workflow automation)
- Linear / Todoist / Things (task management)
- Readwise / Instapaper (read-later + highlights)
- Calendly / Cal.com (scheduling)
- ... and a long tail of vertical SaaS

All generated as operation subgraphs from the engine's primitives, all running on the user's own Benten instance, all composable with each other because they share the same data layer.

## Core Components

### 1. MCP (Model Context Protocol) Integration

The assistant talks to LLMs. MCP is the standard (10,000+ production servers by 2026). Benten's integration:
- MCP servers expose capabilities as UCAN-attenuated operation subgraphs
- Each MCP tool call is a subgraph execution with audit trail
- LLM providers plug in: OpenAI, Anthropic, Google, local (ollama, MLC-LLM, Apple on-device)
- Default routing: prefer local/cheaper/faster, fall back to cloud providers as needed

The MCP integration is straightforward if the engine is solid. The hard parts are below.

### 2. PARA Knowledge Organization

Tiago Forte's PARA method: **Projects, Areas, Resources, Archives.** The assistant organizes all user knowledge into this structure:

- **Projects:** finite outcomes with deadlines ("launch website", "plan Amy's birthday")
- **Areas:** ongoing responsibilities ("parenting", "health", "work role")
- **Resources:** topics of interest ("machine learning", "woodworking", "Japanese grammar")
- **Archives:** inactive items from all three above

Why PARA:
- Validated by thousands of knowledge workers
- Maps cleanly to graph structure (each category is a labeled Node type)
- Scales from individual use to team/family use
- Supports AI-assisted organization better than tag-only systems (because it's hierarchical AND cross-linked)

Design questions for Phase 6:
- How much of PARA is enforced schema vs. user-configurable?
- How does the assistant maintain PARA structure as the user adds knowledge?
- Migration: can a user import from Notion/Obsidian/Roam and have the assistant PARA-organize on import?

### 3. On-Demand Tool Generation

The killer feature. User says "I need a habit tracker that ties to my gym schedule and sends me a text if I miss more than two days," and the assistant:

1. Interprets the intent (LLM with structured output)
2. Generates an operation subgraph composing WRITE, READ, IVM, EMIT primitives
3. Validates the subgraph against the engine's structural invariants
4. Presents a preview to the user (visualized via `.toMermaid()`)
5. Registers the subgraph; user can use the tool immediately
6. If needed, generates UI components via the platform's rendering pipeline (Phase 5 feature)

This is why code-as-graph matters. The assistant isn't generating *code* — it's composing *primitives*. Which means:
- No arbitrary code execution risk (bounded DAGs only)
- Full audit trail of what the tool does
- Tool is modifiable after creation (user can tweak the subgraph)
- Tool composes with other tools automatically (they share the graph)

Design questions for Phase 6:
- How much assistance vs. automation in tool generation? (fully automatic vs. propose-and-confirm vs. guided)
- How does the assistant learn from tool usage and iterate?
- What's the failure mode when the assistant can't generate what the user asked for? (fallback to SANDBOX? fallback to "I need help from a human developer"?)

### 4. Local-First Execution

The assistant runs on the user's Benten instance. LLM calls go out to remote providers (or local models when available), but:
- User's data never leaves their control
- Capability grants are checked locally before any outbound call
- Outbound calls are minimized (cached embeddings, local retrieval, smaller models for simple tasks)
- When the user pays for remote inference, it's a regular peer transaction in Credits (Phase 8)

This is a key differentiator vs. ChatGPT/Claude/Gemini: those platforms hold the user's data. Benten's assistant holds nothing remotely that isn't encrypted or ephemeral.

### 5. Intent Declaration + Provenance (from Security Critic)

From the security critic's "AI agent principal confusion" finding — THE hardest problem. Mitigation pattern for Phase 6:

- **Intent declarations.** User signs a statement of what the assistant is authorized to do in this session: "organize my email", "research X topic", "plan trip Y". The assistant operates within declared intent; stepping outside requires confirmation.
- **Provenance tree.** Every agent action links back to the signed intent that authorized it. Causal attribution (Invariant 14) extends to the user-intent Node.
- **Two-agent quorum for sensitive actions.** For spending, publishing, or irreversible writes, a separate auditor agent (differently trained, different capability grants) reviews the primary agent's proposed action before execution.
- **Anomaly thresholds with human-in-loop.** Spend > $5 without explicit approval? Write to 100+ Nodes in a minute? Novel capability invocation? → pause and ask the user.
- **Agent capability TTLs in minutes.** Cap time-to-live is short; user biometric re-auth extends. Lost device window is bounded.

## Integration With Other Phases

- **Phase 1-2 engine:** the assistant's tool generation composes engine primitives. If primitives are wrong, tools are wrong.
- **Phase 3 sync:** the assistant syncs across the user's devices via Atrium (phone + laptop + home server share the user's data).
- **Phase 4 Thrum migration:** Thrum's content types, modules, and admin become Nodes the assistant can introspect and compose tools around.
- **Phase 5 platform features:** schema-driven rendering is how the assistant's generated tools get UIs without the assistant having to write Svelte/React.
- **Phase 7 Gardens:** the assistant can work within a family/team Garden — "plan dinner" can consult everyone's calendar, dietary preferences, and shopping list.
- **Phase 8 Credits:** the assistant can spend Credits on behalf of the user (peer compute, LLM inference, external services) within capability limits.

## Open Questions

1. **LLM provider economics.** In Phase 6 MVP, we rely on external LLM providers. Who pays? Does the user bring their API key, or does BentenAI wholesale and resell? Or do we only support local models initially?

2. **The "PARA + free text" tension.** Some users want strict PARA structure; others want free-form capture that gets organized later. The assistant needs to handle both gracefully.

3. **Tool generation safety.** What if the assistant generates a tool that, when run, breaks the user's graph? Validation catches structural errors but not logical ones. Dry-run mode? Staged rollout?

4. **Continuity across sessions.** How does the assistant remember? Graph Nodes for conversation history, PARA entries, intent tree. But state management during multi-turn tool generation is non-trivial.

5. **Multi-user households.** If Alice and Bob share a home server, do they share an assistant? Have separate assistants on the same instance? The capability system supports separation, but the UX is ambiguous.

6. **Competitive position.** Apple's on-device AI, Google's Gemini Nano, and various open-source local AI projects are racing. When the Personal AI Assistant ships (Phase 6), what's the pitch vs. those?

7. **Revenue model.** The assistant itself might be free (drives adoption). Revenue comes from Credits usage for compute/storage. Is that right, or should Premium features exist?

## Related Research

- PARA method: https://fortelabs.co/blog/para/
- Model Context Protocol: https://modelcontextprotocol.io
- Intent-based systems: CowSwap, Anoma
- Local-first AI: MLC-LLM, ollama, Apple Intelligence
- Agent security: OWASP LLM Top 10, NIST AI Risk Management Framework

## Source Material

This is scoping for Phase 6. Full spec emerges during Phase 6 pre-work. Related ideas in:
- [`docs/VISION.md`](../VISION.md) — the three-pillar vision
- [`docs/ARCHITECTURE.md`](../ARCHITECTURE.md) — how the assistant composes on the engine
- [`docs/research/explore-distributed-compute-vision.md`](explore-distributed-compute-vision.md) — section on AI agents as economic actors
- Security critic review (2026-04-14) — AI agent principal confusion as the biggest unaddressed risk
