# R0 Design Proposal (FINAL) — Phase-4-Meta-Composing: The Self-Composing Meta-Circular Admin (UI-Primitives-as-Graph)

*Finalized after four adversarial critiques (graph-abstraction-honesty · meta-circular-consistency-and-safety · capability-security-tauri-t3 · adoption-dx-designer-reality). Every critique-surfaced risk below is either RESOLVED in the design or FLAGGED as an explicit open question; a full traceability table sits at the end so nothing was silently dropped.*

---

## Executive recommendation (the decisive verdict, revised)

The thesis survives the critiques, but it survives **smaller and more honest** than the draft claimed. One load-bearing fact still dictates everything: **the UI-graph is a *data* graph, not a *handler* graph — UI structure is content (node labels + edge labels), and UI behavior is edges that hand off to the engine's existing 12 primitives.** That protects 12-primitive irreducibility, requires zero new `PrimitiveKind`, and makes meta-circularity fall out. Build on it.

But three concessions the critiques forced are now part of the design, not footnotes:

- **The graph owns *structure, binding, behavior-wiring, and state-machines* — not *live values, animation physics, or the CSS cascade.*** The elegant claim is "everything above the interaction-mechanics-and-live-value layer," and that layer is *pervasive*, not marginal. Naming that line precisely is what keeps the aesthetic true instead of aspirational.
- **Safety comes from a below-the-line Trusted Computing Base + guaranteed recovery, not from a clever commit gate.** A structural gate cannot prove a self-edit is *operable*; conceding that and moving the guarantee to an out-of-band, non-self-editable recovery rail is the honest and stronger design.
- **The graph is the IR, not the authoring surface.** A raw node/edge canvas is unbuildable-by-a-designer at UI density. The primary authoring surface must be visual direct-manipulation + a textual DSL, with the graph as the serialization underneath — and this is now a first-class Part of the design, not an afterthought (build-item #6).

Concretely I recommend:

1. **Taxonomy:** a *product* of three small closed **generator** sets — **5 node-types × 6 edge-types × an ~8–12-member interaction/platform kernel** — honestly counted (the generators are ~19–23; their *compositions* are the large space, and that is the point of a generative system, not a defect).
2. **The shadcn-vs-HTML line = the accessibility line.** Adopt the vetted headless *behavior* layer (Bits UI + Melt UI) at behavior granularity; express structure/layout/binding/styling/event-wiring as graph. shadcn is a seed library of compositions + a styling blueprint, never an opaque primitive.
3. **Tech foundation: Svelte 5 core (primary) → SolidJS (pre-vetted fallback) → Leptos-driven-by-IVM (parallel research track, shape (a)).** React is the maturity benchmark, not the pick; Slint is rejected for the admin. The choice is **localized to one swappable `Renderer` impl** over a toolkit-neutral graph IR — low-stakes and reversible — but the hiring/onboarding cost of a bespoke IR is real and is flagged, not waved away.
4. **Meta-circularity:** content-addressing means you never edit the running bytes. The only mutable object is the **CURRENT pointer** on the admin's version Anchor — and the *authority to move it* lives in a segregated, below-the-line TCB the admin cannot edit.

The through-line is unchanged and now defensible: **this is the same idea as the engine, one level up — a small closed set of composable primitives whose `ON` edges close into the engine's 12 verbs — with an explicitly named boundary where UI mechanics, live values, and the trust root live outside the graph.**

---

## Part 1 — The UI/UX-primitives-as-graph design system

### 1.1 The honest shape: generators vs. compositions (resolves the "product vs. sum" sleight)

The draft sold "a *product* of three closed sets" and then *sized* it as a sum (5+6+4). The critique is correct and the honest framing is actually **more** compelling: these are **generators**, like the engine's 12 primitives. Twelve verbs generate an unbounded space of handlers; nobody calls that "12 things do everything." Likewise **~19–23 UI generators** (5 nodes + 6 edges + an 8–12-member kernel) generate a large composition space — and *that space is the product (which node, under which pattern, via which edge)*. The value proposition is **not** "UI is only 15 things." It is: **the generator set is small, closed, and auditable; the composition space is large, content-addressed, forkable, and editable-in-admin.** I state the count honestly and drop the "≈15, therefore simple" rhetoric.

### 1.2 Sort 1 — Node primitives (the material): **5**

| Node | Binds to (HTML) | Absorbs | Notes |
|---|---|---|---|
| **Frame** | div/section + flex/grid | Box, Stack, Grid, Card-shell, Panel | the *only* container; layout algebra (stack/row/grid/flow/layer) is an **explicit property**, not a child node |
| **Text** | span/p/h1–h6/label | every typographic element | one node + `variant` prop |
| **Media** | img/svg/canvas/video | Icon | opaque pixel/vector leaf; **untrusted-source sanitization boundary applies (see Part 6)** |
| **Action** | button/a | every trigger | event source; edge-affinity `ON` |
| **Field** | input/textarea/native-select | Checkbox, Switch, Radio, Slider, Toggle | value-carrying; two-way `BINDS`; Choice/Toggle = `Field(bool/enum/number)` + styling |

**Action and Field are irreducible** (the platform grants AT-tree/IME/focus/native-picker semantics you'd otherwise reimplement badly). Absent by design: no Button/Card/Alert/Navbar/Modal/Overlay node — those are compositions. Overlays are *not* a node: they break the containment tree, so they are a **kernel pattern** (`layer/portal`) applied to a `Frame`, keeping "Frame is the only container" clean.

### 1.3 Sort 2 — Edge primitives (where the design system lives): **6**

1. **CONTAINS** — ordered parent→child slot-filling; per-child placement is an edge annotation.
2. **BINDS** — node property ⇄ data source (an engine IVM-view CID); one-way display / two-way Field; carries a projection. **The literal seam to the engine graph.**
3. **ON** — event → handler-subgraph CID (WRITE/CALL/BRANCH-rooted). **The meta-circular closure.**
4. **STYLES** — node → design-token Node. Appearance is a binding to a **typed-token** subgraph (see 1.6), never inline strings.
5. **STATE** — node → a *declared state* (loading/empty/error/disabled/selected/open). **Declares which states exist and what they gate — not their live value** (see 1.5). Drives presence-gating (→ BRANCH) or a reactive variant.
6. **ITERATES** — Frame ← collection + child template. Maps 1:1 to engine ITERATE + IVM.

Payoff, unchanged and correct: an Action's `ON` points at a WRITE/CALL subgraph and a Field's `BINDS` points at a READ/view subgraph — **the UI graph and the engine graph are literally one graph.**

### 1.4 Sort 3 — The interaction/platform kernel — the irreducible behavioral floor (revised: NOT four, and NOT marginal)

The draft asserted a 4-member floor mirroring Radix internals and analogized it to SANDBOX. **The critique is right on both counts:** the floor was never stress-tested, and calling it "the analog of SANDBOX" borrowed SANDBOX's *it's-just-the-margins* respectability for something that is in *nearly every interactive component*. Corrected:

**The kernel is the platform-semantics + imperative-mechanics floor, and it is pervasive.** Confirmed members (Radix-proven): `focus-scope`, `dismissal`, `presence/transition`, `layer/portal`. **Candidate members the critique surfaced that do NOT decompose into nodes+edges and must be probed at R1:** `list-virtualization/windowing`, `drag-and-drop`, `selection-model` (shift/ctrl range-select), `undo/redo stack`, `IME composition`, `observer-driven layout` (resize/intersection). Honest floor size: **8–12, not 4.**

**R1 MUST run the taxonomy adversarially against three named hard cases before R2:** a **sortable virtualized data table**, a **canvas/node editor** (the admin's own composition editor!), and a **rich-text Field**. If any of these forces a *new node or edge type* rather than a kernel member + composition, the taxonomy is wrong and we learn it in R1, cheaply. This replaces the draft's unearned confidence.

Reframed analogy: the kernel is the **honest imperative escape hatch** for interaction mechanics that don't decompose — but unlike SANDBOX it is on the hot path, so the correct statement is: **"a real, ever-present slice of UI lives in imperative `Renderer`-trait code outside nodes+edges — and that is fine, provided the boundary is named and the kernel set is closed and audited."** The meta-circular claim is scoped to "everything above the kernel + live-value layer."

### 1.5 The ephemeral/committed boundary — the make-or-break line, now drawn precisely (resolves the central contradiction)

The critique correctly caught a dead-center self-contradiction: `STATE(open)` is a first-class edge, yet "accordion open/closed" is also listed as renderer-local ephemeral; two-way `BINDS` says value⇄source, yet an uncommitted keystroke is ephemeral. **The resolution is a clean type distinction:**

> **The graph carries the durable state-*machine*; the runtime carries the live state-*value*.**

- `STATE(open)` the **edge** declares: *this Frame has an open/closed state that gates the visibility of that child.* That is durable structure — a fact about the composition.
- The **current** open/closed *value* (this instant, this session) is **renderer-local and never a Node.** Same for focus, hover, scroll position, uncommitted input text, in-flight animation frame.
- Two-way `BINDS` the **edge** declares: *this Field's committed value maps to data source Y.* The **uncommitted keystroke stream** is renderer-local; it becomes a graph fact only when a commit event (blur / submit / explicit save) fires an `ON`→WRITE. **This is exactly the seam that prevents "every keystroke is a graph write."**

Standing guard (loud, from the critique): **transient interaction state MUST live entirely in renderer runtime and MUST NEVER become a Node.** The graph describes structure + bindings + behavior-wiring + which-states-exist; it never stores which-state-is-live-right-now. **Open decision C (optimistic-write reconciliation)** is the residual of this line under shape-(b) network latency and gets genuine R1 treatment — it is now scoped, not hand-waved.

### 1.6 Styling and CSS — honest constraint, not a dodge (resolves the "deepest leak")

The critique is correct that a "STYLES edge to tokens" models *which token*, never *how the cascade resolves* (specificity, `:has()`, sibling combinators, container queries, stacking contexts, intrinsic-size negotiation). The resolution is **deliberate constraint, stated openly:**

- The graph owns **layout structure** (Frame's layout algebra as explicit, local properties) and **token references** (a flat, *resolved*, **typed-enum-valued** style bag per node — not arbitrary CSS strings).
- The graph does **NOT own the CSS cascade.** Non-local appearance ("red only inside a `.danger` ancestor and not `:last-child`") is **out of scope by design** — where such conditionality is genuinely needed, it is expressed as an explicit `STATE`/`BINDS` edge (making the non-locality *data*, legible in the graph), never smuggled in as a cascade rule.
- **Tokens are typed enum values, not strings** (this simultaneously resolves the CSS-injection attack in Part 6 — no `url()`, no arbitrary declarations). Tailwind classes are one `Renderer`-impl *translation* of toolkit-neutral tokens; a native renderer maps the same tokens to native styling. We concede that a Renderer *does* own the last-mile cascade translation, which slightly weakens "toolkit-neutral IR" — the IR is neutral over *tokens and layout*, and each Renderer owns cascade resolution. That is the honest boundary.

**Flagged for R1:** confirm the constrained token/layout model covers the admin's real screens without needing an escape into raw CSS. If it doesn't, the escape hatch must be a *named, sanitizable* one, not ad-hoc string smuggling. (Ties to Open Decision **G**.)

### 1.7 Animation — conceded, not claimed (resolves the hand-wave)

The draft folded all animation into "presence/transition." Corrected: **declarative transition *intent*** (which STATE transitions animate; token-valued duration/easing/reduced-motion) lives on the graph; **per-frame physics — FLIP, springs with velocity state, scroll-linked, staggered, layout animation** — is **renderer-local imperative**, part of the kernel, and explicitly outside the graph. We do not claim animation "lives in the graph." Reduced-motion is a VALIDATE-checkable a11y invariant (Part 5).

### 1.8 The shadcn-vs-HTML balance — three tiers, dependency line = a11y line (unchanged core, sharpened)

> **The one-line rule: adopt anything that is an accessibility/interaction state machine (Tier 1); express as graph anything that is structure, layout, data-binding, styling, or event-wiring (Tiers 0/2). The dependency line = the a11y line.**

- **Tier 0** — the 5 nodes + 6 edges built from fundamental HTML inside each `Renderer` impl = portable graph DATA.
- **Tier 1** — the kernel patterns, adopted from the vetted headless behavior layer, delivered as `Renderer`-trait code. Svelte: **Bits UI on Melt UI** (Melt's *attachable builder functions* are a near-perfect match to "behavior as an edge-attached capability"). This is **crypto-doctrine #5 applied to UI: never fork, never reimplement — wrap vetted upstream; the integration is glue.** Rebuilding a combobox's ARIA from `<div>`s is the UI equivalent of hand-rolling ML-KEM.
- **Tier 2** — shadcn-equivalents = **blessed SUBGRAPHS stored as data** (content-addressed, versioned via the shipped Anchor/Version machinery, forkable, editable-in-admin, shareable). **Tier 2 *is* the plugin mechanism already shipped.** shadcn-svelte's MIT source is lifted as a **styling/wiring blueprint**, never imported as opaque component structure — a black-box `.svelte` Dialog is un-serializable to DAG-CBOR and un-editable through the admin, i.e. a source-code-string wearing a node costume, exactly what code-as-graph exists to reject.

**The flagship collapse, with the critique's caveat folded in:** Dialog / Sheet / Popover / Tooltip / Toast / HoverCard / ContextMenu / DropdownMenu are **one template** — `Action(trigger)` + portal-`Frame(content)` + focus-scope + dismissal + presence + `STATE(open)` — parameterized by trigger-event, position, focus-policy, and dismissal-policy. **The critique is right that these parameters are a small behavioral policy language, not nothing.** So we name it: the template exposes a **closed, typed policy set** {trigger∈…, focus∈{trap,none,auto}, dismiss∈{esc,outside,blur,timer}, position∈…, lifecycle∈{single,queue}}. Toast's *queue + auto-dismiss timers* is the honest outlier — it is a `lifecycle=queue` policy the kernel interprets, and we flag queue-lifecycle as one of the kernel members R1 must validate. Eight components → one template + one small typed policy set. The complexity is *relocated into a named, closed policy space*, not vanished.

---

## Part 2 — Technology-foundation recommendation

### 2.1 What the thesis demands

Because the UI is a graph interpreted at runtime and kept live by the engine's IVM, the framework is **the effect layer a graph-walker drives**, not where the UI lives. Priorities: (1) reactivity that *composes with IVM instead of duplicating it*; (2) a *runtime* rendering API; (3) a mature accessible behavioral-primitive set (the un-reimplementable Tier 1); (4) a lean runtime for shapes (b)/(c).

### 2.2 The ranking

| Foundation | IVM-reactivity fit | Runtime interpretation | Headless a11y layer (load-bearing) | Bundle | Verdict |
|---|---|---|---|---|---|
| **Svelte 5** (+ Bits/Melt, shadcn-svelte seed) | Excellent — signals; runes are IVM one layer up | Viable (see 2.4) | **Mature: Bits UI + Melt UI, Radix-shaped** | Small, no VDOM | **PRIMARY** |
| **SolidJS** (+ Kobalte) | Excellent — uniform *runtime* signals | Cleanest | Younger/thinner (Kobalte) | ~7KB | **FALLBACK (pre-vetted)** |
| **Leptos** (IVM-driven) | *Perfect* — same Rust reactive graph, zero-serialization | N/A (native/wasm) | **Does not exist yet (6–18mo)** | Heavier wasm | **RESEARCH TRACK, shape (a)** |
| **React** (+ Radix) | Poor — VDOM is a redundant second reconciler | Friendliest | Best day-one breadth; **largest hiring pool** | ~45KB+ | **BENCHMARK, not chosen** |
| **Slint** | Good | canvas, not DOM | canvas kills browser a11y; license conflicts | heavy | **REJECTED (design ref; Phase-9+ watch)** |
| **Web Components** | Good (signal-adapter bridge) | Excellent | **You own the whole a11y layer** | Smallest | Fit is real; cost is in the tier we want to *buy* |

### 2.3 Why Svelte over Solid, and the honest cost (folds the ideological-purity critique)

React is out on architecture: **content-addressing already answers exactly what React's reconciler exists to guess.** Equal CID ⇒ subtree provably unchanged — *stronger* than key-heuristic diffing. A VDOM on top of a content-addressed IVM graph pays twice to solve a solved problem, in a project whose identity is a small closed set of composable primitives.

**But the critique's adoption point is valid and I fold it in rather than deflect it:** rejecting React on architecture *does* cost us the biggest ecosystem, the best day-one Radix breadth, and the largest hiring pool, in exchange for a bespoke node/edge IR + interpreter + UI-materializer that *no engineer has prior experience with.* That onboarding cost is real. The honest mitigation, and the reason I still recommend against React: **the durable thing a new hire must learn is the toolkit-neutral graph IR — and it is learned once and survives every framework swap.** Svelte itself is thin, shadcn-svelte is recognizable (kept as a visible handhold, not buried), and the framework is one swappable `Renderer` impl. This does not make the cost zero. **It stays Open Decision H** (architectural-fit vs hiring-pool), surfaced honestly.

Svelte beats Solid on four grounds: (1) **Tier-1 behavior-layer maturity is first-order** — Bits/Melt are mature and Radix-shaped; Kobalte is younger/thinner; (2) the runtime-interpretation friction resolves cleanly (2.4); (3) Ben's technically-well-founded lean; (4) **the pick is localized and reversible** — swapping Svelte→Solid touches one `Renderer` impl, not the graph IR. **Solid is the named, pre-vetted fallback.** Exclude SvelteKit (server-coupled SSR/router/adapter fights thin-client + Tauri; routing becomes a UI-graph concern — a route is a view-selection edge, more meta-circular anyway).

**Rust-native (Leptos-driven-by-IVM)** is a first-class *parallel research track* targeting shape (a) desktop first: most thesis-aligned (graph-authoring hides the framework, so "designers must learn Rust" does not apply; IVM↔fine-grained-reactivity is a genuine isomorphism with zero serialization, deleting UI-layer `errors.generated.ts`-class drift; no browser-a11y objection on native). Stand it behind the same `Renderer` trait; watch `dioxus-native`/Blitz (WGPU, no webview) harden. **Slint: rejected for the admin** (tri-license conflicts with dual-MIT/Apache + vendor-never-fork; canvas-not-DOM destroys browser a11y); mine as a binding-grammar reference; Phase-9+ embedded watch-list.

### 2.4 Runtime interpretation — and why it does NOT need `unsafe-eval` (resolves the CSP tension, C3-3 + C4-6)

The sharpest Svelte objection (runes are compiler-only) and the sharpest security objection (a runtime interpreter is the dynamic-code surface CSP kills) **share one resolution: compositions are DATA, not CODE.**

- The interpreter is a **data-driven registry**: it maps `node-type → a pre-compiled primitive component` and wires `BINDS→store` / `ON→engine-CALL`. It **never `eval`s markup or code** — the primitive vocabulary is a closed, pre-compiled set (~19–23 generators). Runtime dynamism is over *graph structure, bindings, events* — **never over component types.** So **no `unsafe-eval`.**
- Runes handle the compiled *inside* of each primitive; **Svelte's runtime store contract** (stores are runtime, unlike runes) handles the dynamic *between*, fed by engine SUBSCRIBE deltas. That is exactly the seam we need.
- **Dynamic styling is CSP-safe**: typed tokens → class toggling / constructable stylesheets / CSSOM, **never inline-string injection.** So **no `unsafe-inline`** (target).
- **CURRENT hot-swap re-materializes new *data* through the same locked bundle — no new *code* loads — so the CSP locked at load still fully describes the running program.** Hot-swap-without-reload is CSP-safe **by construction**, precisely because the meta-circular substrate is data, not code.

**This reconciles strict-CSP + interpreter-primary + hot-swap.** It also dissolves C4-6: **AOT does NOT kill self-edit.** The interpreter is itself a plugin/subgraph and **ships to production**; AOT is a *per-published-leaf* optimization for frozen CIDs on thin clients, not a global mode that disables editing. Self-edit uses the interpreter path in production. **Flagged for R1 (spike):** confirm we can reach *zero* `unsafe-inline` for dynamic styling under Svelte 5 (constructable stylesheets are the likely path); if a residual `unsafe-inline`-for-styles is unavoidable, that is a named, bounded CSP relaxation to ratify, not the open-ended interpreter-eval hole the critique feared. (Ties to Open Decision **D**.)

### 2.5 Rendering strategy — interpreter-primary, CID-keyed AOT-secondary (confirmed, Open Decision D)

Per-UI-subgraph selectable, keyed by CID immutability (the shape of IVM's per-view `Strategy` selection, one layer up): **runtime interpreter (default)** — the only path delivering live meta-circular edit (mutate graph → engine re-materializes → interpreter re-renders, zero build step); it *is* "the evaluator for the view layer" and ships as a subgraph. **CID-keyed AOT codegen (secondary)** — a frozen/published CID is immutable, so compile it to an optimized tree-shaken `.svelte` component; CID-immutability makes compile-cache-by-CID trivially correct and is the mitigation lever for interpreter+registry bundle weight on thin clients.

---

## Part 3 — Meta-circular self-composition architecture (heavily revised for safety)

### 3.1 The insight that dissolves the *regress* paradox (critique concedes this holds)

**Content-addressing means you never edit the thing you're running.** The admin composition is an immutable CID; the running admin is a *materialization*. Every edit authors a *new* immutable subgraph off to the side; nothing mutates on-screen bytes. **The only mutable object is the CURRENT pointer.** The safety critique explicitly **concedes** that the *infinite-regress render* attack fails against this design (MENTION-mode-for-the-editor + depth-1 USE preview + the reused install-time DFS visited-set cycle detector genuinely terminate). Good — R1 spends **zero** effort on regress and redirects entirely to the four real holes below.

Second load-bearing decision retained: **editing the admin is not a special mode** — the composition editor edits whatever CID its target-anchor parameter resolves to; point it at the admin's own anchor and self-editing is the *general* flow. **Refuse any design where "edit the admin" is a hand-written second implementation.**

### 3.2 The Trusted Computing Base — the "below-the-line" set (resolves C2-B2, C2-M6, C3-5, C3-6, and the one-flow-vs-excluded contradiction)

The critiques' deepest structural finding: the thesis wants **one universal edit flow over every composition**, but safety wants R0 + the validator + the pointer-authority *excluded* from that flow — and the draft named both desiderata without stating the mechanism. **Here is the mechanism.**

> **Draw an explicit line. Define a small, build-time-pinned, host/engine-enforced Trusted Computing Base (TCB) that is NOT an admin composition and is NOT reachable by the self-edit flow. The "one universal edit flow" applies to everything ABOVE the line. This is the Inv-15-style invariant for Composing: *the TCB is never a self-editable composition.***

The TCB (below the line, host-shell / engine code + build-time-pinned data — *not* graph the admin can rewrite):

1. **The CURRENT-flip authority** — a *segregated* capability, held by a host/recovery principal, **never present in the admin's editable capability graph.** The admin can *author* `V_draft`; it cannot, by itself, *become* CURRENT without the TCB validating and flipping. (Resolves C2-M6/C3-6.)
2. **The VALIDATE commit gate** — engine/host code, **not** an admin subgraph. (The draft's "expressible as SANDBOX/TRANSFORM→BRANCH→RESPOND" describes its *logic*; the critique is right that *expressibility ≠ location*. The gate's *location* is the TCB.) (Resolves C2-B2/C3-5.)
3. **R0 recovery** composition + its **mount trigger**, host-shell owned (3.5).
4. **Principal-binding** — the session principal is shell-bound and injected by the full peer (Part 6), never editable and never payload-supplied.

The line is the whole game. Everything above it is one universal, meta-circular flow; the four TCB elements are the deliberately-small, boring, non-meta-circular root of trust — the same shape as the engine's own "you compiled this in" trust boundary for extensions.

### 3.3 Bootstrap — genesis is build-time (unchanged; critique's migration point folded into Part 5/Part 7)

No runtime chicken-and-egg. Admin UI v0 is compiled + content-addressed at build into immutable genesis **`V0`**, installed under a user-DID-signed record, **permanently pinned (never GC'd)** — it is the immutable rollback floor. `V0` contains a **reflexive editor pane** whose `BINDS` READ the admin plugin's own nodes and render them as an editable node/edge canvas + property inspector (authored via the surfaces in Part 5, not raw). Startup: resolve admin anchor → CURRENT → `V_current` → `read_node_as(admin_principal, V_current)` → materializer → Renderer emits shell + editor pane targeting the admin's own anchor. Self-editing exists from moment zero, via the general flow. *(The honest cost — v0 must be rebuilt in the new IR before any of this works — is in Part 7, not hidden here.)*

### 3.4 Consistency under self-edit — reframed: recovery makes bricks non-fatal; the gate is defense-in-depth (resolves C2-B1, C2-M4, C2-M5)

The draft's load-bearing claim — *"you cannot commit an admin version that lacks the means to keep editing," enforced by a structural reachability check* — **does not hold, and the critique is right.** A structural gate can prove the editor node *exists* and is `CONTAINS`-reachable; it **cannot prove it is *operable*.** A self-edit passes the gate while shipping an editor that is `STYLED_BY` a `display:none`/`opacity:0`/zero-size token, occluded by a full-viewport transparent portal-`Frame`, `STATE_WHEN`-bound to `disabled=true`, or whose `ON` edges are rewired to a no-op — **all graph-topologically valid.** And the auto-fallback catches only *render panics*; every one of these bricks *renders successfully* into an unusable surface (C2-M4).

**The resolution is to move the guarantee, not to pretend the gate is stronger than it is:**

- **The guarantee is the below-the-line recovery rail (3.5), which is always mountable regardless of render success or semantic brick, plus the immutable-`V0` rollback floor.** *Bricks are non-fatal because recovery is out-of-band, not because the gate is clairvoyant.* This is the honest, and stronger, safety story.
- **A mandatory human preview-and-confirm step before every CURRENT flip.** You render the candidate (depth-1 USE, effect-isolated per below) and *see it* before it becomes CURRENT. A human catches "the editor is invisible" that a structural gate cannot decide.
- **The VALIDATE gate is retained as defense-in-depth**, and *does* run the checkable subset: cycle detection; every UI node resolves and every `BINDS` has a live source; **and best-effort operability heuristics** the critique itself enumerated — reject a candidate whose editor/recovery-path node resolves to a zero-size/`display:none`/`disabled` style, or is occluded by a fullscreen portal, or whose recovery-path `ON` edges don't resolve. These catch the *obvious* bricks structurally; they are explicitly **necessary-not-sufficient**, and the design no longer *depends* on them.

Flow: **STAGE** (WRITE draft to `private:<admin_did>:draft`, content-addressed; live editor keeps running off `V_current` throughout — no in-flight edit can break the surface you edit from) → **PREVIEW** (depth-1 USE, effect-isolated) → **VALIDATE** (TCB gate, checkable subset + a11y invariants per Part 5) → **human confirm** → **COMMIT** (TCB authority: `append_version(admin_anchor, V_draft)` then flip CURRENT). `V_{n-1}` stays fully intact and materializable; **rollback is a TCB pointer move**; fork semantics fall out (`VersionError::Branched`); version history is browsable inside the admin as a content-list over the anchor's Version Nodes.

**Preview effect-isolation (resolves C2-M5):** the draft's depth-1 bound limits *render recursion* but the critique is right that a previewed `Action`'s `ON` handler is a real WRITE/CALL subgraph that can fire on mount and mutate live anchors or flip CURRENT as a *side effect*. **Containment was on the wrong axis.** Fix: **preview runs handlers in a dry-run/scratch execution context** — the pointer-write cap and the live-anchor-write caps are **withheld**; writes are redirected to a throwaway scratch namespace and *simulated*, never applied. Depth-1 bounds render; the withheld-caps scratch context bounds *effects*.

### 3.5 The safety rail — recovery lives *below* the self-editable layer, per shape (resolves C2-B3, C2-M4)

A recovery editor a self-edit can brick is not recovery. The draft grounded "always-available" in shape (c)'s native shell; the critique is right that this **only holds for (c)**, and that making routing meta-circular ("a route is a view-selection edge") *destroys* the `/#recovery` trigger for shape (b) by putting it inside the editable composition. Corrected, **per shape**:

- **Shape (a) full peer / (c) embedded webview:** the **host shell** (a full peer) owns the trigger out-of-band — a native menu item / intercepted keychord — mounting TCB-owned `R0` regardless of what the webview composition did. (c)'s recovery is robust.
- **Shape (b) browser tab (no native shell):** recovery **cannot** live in the admin composition (its route table is editable). Resolution: **shape-(b) recovery is served out-of-band by the full peer it connects to** — the peer exposes a *host-served* recovery surface (a fixed, non-composition endpoint / a peer-side "reset my admin CURRENT to a known-good CID" control) reachable independent of the admin graph. Browser-shape recovery is therefore *a request to a healthy peer*, not a route inside the possibly-broken admin. **Flagged for R1:** specify this peer-served recovery endpoint concretely; it is the one genuinely shape-specific gap and must not be left as "the `/#recovery` route."
- **Auto-fallback safe mode** (all shapes): render panic on CURRENT → Renderer catches on the RESPOND error path → auto-mount `R0`. **Explicitly labeled a partial net** — it catches crashes, *not* semantic bricks; the human-confirm step + out-of-band trigger are what cover the non-crashing brick class.

Recovery is **guaranteed to terminate at a working admin** because permanently-pinned `V0` is an immutable, always-materializable floor and "set CURRENT back to `V0`" is always a valid TCB operation. **Two indirections carry everything — the CURRENT pointer (only mutable object) and the materializer (CID→UI) — and no new primitive is introduced.**

---

## Part 4 — Engine-substrate mapping

### 4.1 UI-graph is a DATA graph (protects 12-primitive irreducibility)

A `Button`/`Frame` is **not** a new `PrimitiveKind` — it is a **label on a Node in a data graph**, exactly like the shipped schema-driven-rendering vocabulary and the plugin-library subgraph (which added `anchor::`/`version::` labels + `ITEM_TYPE`/`VERSION_OF`/`CURRENT` edges *without minting a primitive*). **Standing guard: reject any `RENDER`/`UI`/`EVENT` PrimitiveKind proposal.** Node labels = the 5 UI primitives + Tier-2 composite labels (subgraphs). Edge labels = `CONTAINS`/`SLOT`, `BINDS_TO`, `STYLED_BY`, `ON_EVENT`, `STATE_WHEN`, `ITERATES_OVER`. **Behavior rides edges into the 12:** an `ON_CLICK` edge → a handler subgraph rooted at WRITE/CALL/BRANCH; runtime dispatch resolves the edge and invokes the evaluator via CALL semantics. **Interactivity is literally engine evaluation of code-as-graph.**

### 4.2 The Renderer-trait contract — a serializable, content-addressed render tree

The "Renderer holds no runtime types" rule *dictates the design*. What crosses the boundary is a **pure, serializable, content-addressed render tree (canonical DAG-CBOR at a CID)** — never Svelte handles, DOM refs, tokio types, or view-models carrying closures. A specialized **UI-materializer** (materializers are already generic/pluggable → additive, no interface change) walks the UI-graph, resolves `BINDS_TO` views + `STYLED_BY` tokens, and emits the render tree; each concrete `Renderer` translates render-tree nodes to its target. Content-addressing makes **subtree diffing free — O(changed-subtrees) by CID comparison** (React-style reconciliation as a *consequence*, not a hand-written diff). **Reinforced guard (from the critique): event handlers in the render tree are handler-subgraph CIDs (data), never function pointers/closures — forbid passing a live callback across the Renderer boundary at review time.**

### 4.3 Deployment-shape fit (one Svelte renderer serves b + c)

- **(a) Full peer:** materializer in-process; render tree → native renderer (`TauriRenderer`→webview today; future Leptos/Slint/Iced) over the bridged dual-runtime channel.
- **(b) Thin compute (wasm32):** render tree computed by a full peer; the wasm32 bundle receives the materialized tree + CID-diffed subtree patches over the authenticated thin-client protocol; Svelte `BrowserRender` maps render-tree nodes → components → DOM. **No IVM/sync in the bundle.**
- **(c) Embedded webview:** native shell is a full peer internally; the webview loads the **same wasm32 bundle as (b)** and receives the same render-tree patches over in-process IPC. **One renderer, one bundle, identical render-tree contract across all three shapes — only transport differs.**

### 4.4 Data-binding rides IVM → SUBSCRIBE → STREAM (no new primitive)

`Table --BINDS_TO--> viewCID`: the renderer SUBSCRIBEs. Upstream WRITE → IVM Algorithm B recomputes the canonical view → SUBSCRIBE fires → the full peer re-runs the UI-materializer for the affected subtree → new subtree CID → CID-diff → renderer patches. STREAM covers large/virtualized lists + progressive data. **Tightest reuse of the frozen substrate: server-IVM (incremental) → SUBSCRIBE delta → client signal/store (incremental) → minimal DOM patch — two dependency-tracked delta systems chained across the network boundary.** Keep the durable reactive graph in the engine (IVM); the client signals/stores are the disposable last mile — conflating them (durable/CID-addressed signals) would be a category error, and it is the same category error as putting live values in the graph (1.5).

---

## Part 5 — Authoring & DX (NEW — the adoption critique made this first-class)

The adoption critique's central charge is correct and is now answered structurally, not with a phrase: **the R0 as drafted answered "is the IR beautiful?" exhaustively and "can a designer make a screen?" almost not at all.** Node-graph editors reliably hit spaghetti at UI density (Blueprints/n8n/Node-RED); a mundane settings form is 100+ graph elements versus ~40 readable Svelte lines. Fixing this:

1. **The graph is the IR/serialization — NOT the primary authoring surface.** Raw node/edge canvas is the *inspector / power-user* view, not the default. Primary authoring is **two coordinated surfaces:**
   - **A visual direct-manipulation composer** (Figma-auto-layout-shaped: you manipulate the *rendered* UI — drag frames, set auto-layout, bind fields — and it *emits* graph). The critique is right that "we are formalizing something UI already is" cuts the *other* way — designers already have this formalization as **visual direct manipulation**, and `Frame`+flex *is* Figma auto-layout. We meet them there and treat the graph as the file format behind the canvas. **Figma-import is a flagged R1 research item** (map frames/constraints/auto-layout → Frame/CONTAINS/token bags).
   - **A textual DSL** — extend the *existing TypeScript DSL* so **one language authors UI + handlers together**, so authoring is not raw-graph hand-wiring. `crud('post')`-grade ergonomics for UI compositions is the target.
2. **A naming/alias resolution layer (resolves the CID-tax, C4-2).** Authors **never hand-wire opaque BLAKE3 hashes.** They reference handlers/views/tokens by **stable human names**; a resolution layer maps names→CIDs at author/build time. `onClick → save` where `save` resolves to a handler-subgraph CID. The unification ("UI graph and engine graph are one graph") stays true underneath; the *ceremony* the critique flagged is absorbed by the alias layer + DSL.
3. **Composition-level a11y invariants enforced in VALIDATE (resolves C4-3).** The critique is right that Tier-1 buys only *component-internal* ARIA; the hard half is *composition-level* and lives in edges/author choices. So the TCB VALIDATE gate enforces: **every `Field` has a resolvable label edge; heading levels are monotonic; required landmark structure present; deterministic focus order; reduced-motion honored.** A graph can no longer be "perfectly wired, pass VALIDATE, and WCAG-fail" on these checkable properties. We concede a11y is *not* free from Tier-1 alone.
4. **The worked example is an R1 gate, not a nicety.** R1 must produce a **real settings screen** end-to-end: count nodes/edges/CIDs, and **time a designer building it in the visual composer** vs the ~40-line Svelte baseline. If the composer can't get within a tolerable multiple, the authoring model is wrong and we learn it in R1.

**Honest residual:** even with the composer + DSL + alias layer, onboarding a new engineer to (Svelte 5 + the taxonomy + interpreter internals + content-addressing + meta-circular editing) is a real cost. Mitigation is the IR-learned-once argument (2.3); it is not zero. This stays visible as **Open Decision H.**

---

## Part 6 — Security model (NEW — folds the capability/T3 critique)

The T3 boundary is the *most* load-bearing surface in the self-composing model, and the critique surfaced six escalation paths. Dispositions:

1. **The IPC allowlist is theater under a dynamic graph — accept it and move the boundary (resolves C3-1).** Which handler-CIDs the webview exercises is authored at runtime in shareable graph data, unknowable at build time — so per-handler allowlisting is impossible and you are forced into **one generic Tauri command `invoke_handler(cid, event_payload)`.** The draft's "per-method cap-binding" is meaningless there. **The real security boundary is `CapabilityPolicy` resolution on the handler-CID's *resolved effect*, not the IPC method name.** T3's job shrinks to: expose exactly one narrow, well-audited command; do *all* authorization in the engine's capability layer against what the handler subgraph *actually does* when walked. Named honestly, this is defensible; pretending the allowlist secures anything is not.
2. **Principal is shell-bound, NEVER payload-supplied (resolves C3-2, confused-deputy).** The dispatch phrase "as this principal" must not travel in the IPC payload — a compromised admin-subgraph or plain webview XSS would request execution as `admin_principal` or `user-DID` by editing one field. **The session principal is bound shell-side from the authenticated session and injected by the full peer; it cannot appear in the payload.** The draft's `read_node_as(admin_principal, …)` is safe *only* because the shell/peer chooses the principal — now stated explicitly and placed in the TCB (Part 3.2 item 4). `invoke_handler` therefore takes **(cid, event_payload)** and the peer supplies the principal.
3. **CSP ↔ interpreter reconciled (resolves C3-3)** — see 2.4: compositions are data not code ⇒ no `unsafe-eval`; typed-token styling via constructable stylesheets ⇒ target zero `unsafe-inline`; hot-swap loads no new code ⇒ CSP-at-load stays valid. R1 spike confirms the zero-`unsafe-inline` styling path.
4. **STYLES/Media are sanitization boundaries under cross-Atrium sharing (resolves C3-4).** Tier-2 compositions + token/media subgraphs are shareable and thus **attacker-authorable.** Defenses: **tokens are typed enum values, not arbitrary CSS** (no `url()` exfil, no arbitrary declarations — same constraint as 1.6); **`Media` SVG is sanitized / script-stripped and CSP blocks inline script**; **full-viewport-overlay clickjacking of the recovery affordance is neutralized** because the recovery trigger is out-of-band host-owned (Part 3.5), not a composition element an overlay can cover. **Untrusted compositions cross an explicit sanitization boundary on import**, exactly as untrusted plugin content should.
5. **VALIDATE gate is below the self-editable line (resolves C3-5)** — identical to C2-B2; it is TCB code, so "commit N neuters the validator, commit N+1 removes the recovery rail" is structurally impossible.
6. **The CURRENT-flip cap is segregated (resolves C3-6)** — identical to C2-M6; held by the TCB/recovery principal, never in the admin's editable capability graph. A compromised admin running as `admin_principal` can author a malicious `V_draft` but cannot *become* CURRENT without the below-the-line gate + human confirm.

**Net:** the draft's guard "handlers are CIDs, not closures" closes code-injection-as-closure but does nothing for authorizing an *arbitrary attacker-authored handler-CID resolved at runtime under a privileged manifest.* The answer is: **authorize on resolved effect, bind principal shell-side, sanitize shared style/media, and segregate the pointer-flip cap below the self-editable line.** T3 hardening (build-item #9) is re-scoped around these five, not around per-method allowlisting.

---

## Standing architectural guards (reject future drift)

- No new `PrimitiveKind` for UI (no `RENDER`/`UI`/`EVENT`). UI structure is data; UI behavior is the 12.
- No live callbacks/closures across the `Renderer` boundary — event handlers are handler-subgraph CIDs.
- **No live/ephemeral state as Nodes** — the graph carries the durable state-*machine*; the runtime carries the live state-*value* (1.5).
- No shadcn/Tier-2 component adopted as an opaque primitive — Tier-2 is always a subgraph.
- Never reimplement the Tier-1 a11y/behavior kernel — wrap vetted upstream (crypto-doctrine #5, applied to UI).
- Styling is a node property + `STYLES` edge to **typed** tokens, never a per-utility edge and never arbitrary CSS strings; the graph does not own the CSS cascade.
- **The TCB is never a self-editable composition** (the Composing-tier invariant): CURRENT-flip authority + VALIDATE gate + R0 + principal-binding live below the self-editable line.
- The session principal is shell-bound, never payload-supplied; authorize on the handler's resolved effect, not the IPC method.
- Preview executes handlers in an effect-isolated dry-run (pointer + live-anchor caps withheld).

## Frozen-signature verifications — the ONLY places a UI need touches the freeze (check against actual v1-beta signatures at R1)

1. **`Renderer` trait input type.** The additive design needs the frozen `Renderer` to accept a CID / canonical DAG-CBOR blob. If it already takes a generic materialized-view/CID → the UI render-tree is pure additive schema. **If the frozen input is a narrower typed struct, the render-tree must fit within/extend it — confirm.**
2. **Event dispatch on the thin-client protocol.** A click in shape (b) must round-trip "invoke handler-subgraph-CID **with the peer-bound principal** and this event payload, and subscribe to the result." If the frozen thin-client session already exposes a CALL-shaped RPC + subscribe (it almost certainly must — "writes go via fetch to a full peer" *is* this), event dispatch is additive routing. **If not, this is the one UI need brushing the frozen wire — verify.**
3. **(NEW) CURRENT-flip cap segregation.** Confirm the frozen capability model can express a cap held *outside* an editable principal's graph (the TCB pointer-flip authority). If the shipped `CapabilityPolicy` / private-namespace machinery already supports a host/recovery principal holding a segregated write cap, this is additive; **if the only path to write the admin anchor's CURRENT runs through the admin's own capability graph, that is a real tension to resolve at R1.**

---

## Traceability — how the four critiques were folded in (nothing dropped)

| # | Critique finding | Disposition |
|---|---|---|
| C1-1 | "5×6×4≈15" product-vs-sum sleight | **RESOLVED** — reframed as *generators vs compositions*; honest count ~19–23 generators (1.1) |
| C1-2 | 4-pattern floor untested (drag/virtualize/select/undo/IME) | **RESOLVED-IN-DIRECTION + FLAGGED** — kernel restated as 8–12; R1 must stress-test 3 named hard cases (1.4) |
| C1-3 | Animation hand-waved | **RESOLVED-BY-CONCESSION** — transition *intent* on graph, per-frame physics renderer-local (1.7) |
| C1-4 | Ephemeral/committed self-contradiction (the center) | **RESOLVED** — durable state-*machine* vs live state-*value* line (1.5) |
| C1-5 | CSS cascade is a non-local leak | **RESOLVED-BY-CONSTRAINT** — graph owns layout+typed tokens; cascade out of scope; concede per-Renderer translation (1.6) |
| C1-6 | SANDBOX analogy oversells escape hatch as marginal | **RESOLVED** — kernel reframed as pervasive, not marginal (1.4) |
| C1-bonus | Dialog collapse partly cosmetic; params are a mini-language | **ACKNOWLEDGED** — named as a closed, typed policy set; queue-lifecycle → kernel probe (1.8) |
| C2-B1 | Structural gate can't stop semantic bricks | **RESOLVED-BY-REFRAME** — guarantee moves to recovery + human-confirm; gate is best-effort defense-in-depth (3.4) |
| C2-B2 | Validator inside blast radius | **RESOLVED** — validator is below-the-line TCB code (3.2) |
| C2-B3 | "Always-available" recovery only true for shape (c) | **RESOLVED (a/c) + FLAGGED (b)** — peer-served out-of-band recovery for browser shape (3.5) |
| C2-M4 | Auto-fallback detects wrong failure | **RESOLVED** — labeled partial net; out-of-band trigger + human-confirm cover non-crashing bricks (3.4/3.5) |
| C2-M5 | Depth-1 preview bounds recursion not effects | **RESOLVED** — effect-isolated dry-run scratch context, caps withheld (3.4) |
| C2-M6 | Rollback/CURRENT authority never established below the brick | **RESOLVED** — segregated CURRENT-flip cap in TCB (3.2) |
| C2-contradiction | One-flow vs excluded-TCB, no mechanism | **RESOLVED** — explicit line; one-flow applies above it (3.2) |
| C2-concession | Regress attack fails against the design | **ACCEPTED** — R1 spends zero effort on regress (3.1) |
| C3-1 | Static allowlist vs dynamic graph = theater | **RESOLVED-BY-REFRAME** — one generic command; authorize on resolved effect (Part 6.1) |
| C3-2 | Confused deputy: principal in payload | **RESOLVED** — shell-bound principal, never payload (Part 6.2) |
| C3-3 | CSP-locked vs interpreter-primary | **RESOLVED + R1 SPIKE** — compositions are data not code; CSP-safe styling; hot-swap loads no code (2.4/6.3) |
| C3-4 | STYLES/Media injection under sharing | **RESOLVED** — typed-enum tokens, sanitized SVG, sharing crosses a sanitization boundary (1.6/6.4) |
| C3-5 | VALIDATE gate inside the layer it protects | **RESOLVED** — TCB (Part 6.5) |
| C3-6 | CURRENT-write cap unsegregated | **RESOLVED** — TCB (Part 6.6) |
| C4-1 | Authoring surface unbuildable-by-designer | **RESOLVED-IN-DIRECTION + R1 GATE** — visual composer + DSL primary; graph is IR; worked example required (Part 5) |
| C4-2 | CID-wiring tax, no alias layer | **RESOLVED** — naming/alias resolution layer + DSL (Part 5.2) |
| C4-3 | A11y over-sold | **RESOLVED** — composition-a11y invariants in VALIDATE; concede Tier-1 scope (Part 5.3) |
| C4-4 | Framework ideological purity / hiring | **ACKNOWLEDGED + FLAGGED** — honest cost; IR-learned-once mitigation; Open Decision H (2.3) |
| C4-5 | No migration path; greenfield in legacy clothes | **ACKNOWLEDGED + FLAGGED** — honest rebuild framing + sequencing (Part 7, Open Decision I) |
| C4-6 | AOT kills the self-edit headline | **RESOLVED** — interpreter ships to prod; AOT is per-leaf optimization (2.4/2.5) |

---

## (a) RECOMMENDATION

**Proceed to the Cluster-B (Composing-Admin) ADDL on this design, with the three concessions treated as load-bearing, not cosmetic.** Build the admin as a UI-primitives-as-graph system — **5 node + 6 edge generators + an 8–12-member interaction/platform kernel** — where UI structure is *data* and UI behavior is `ON`/`BINDS` edges closing into the engine's existing 12 primitives (zero new `PrimitiveKind`). Target **Svelte 5** as one swappable `Renderer` impl over a toolkit-neutral graph IR (Solid pre-vetted fallback; Leptos-IVM a parallel shape-(a) research track). Make meta-circularity safe with an explicit **below-the-line Trusted Computing Base** — the CURRENT-flip authority, the VALIDATE gate, R0 recovery, and shell-bound principal are host/engine-enforced and never self-editable — so that **bricks are non-fatal because recovery is out-of-band, not because a commit gate is clairvoyant.** And treat **authoring DX (visual composer + DSL + name→CID alias layer) and composition-level a11y-in-VALIDATE as first-class R1 deliverables**, gated by a real worked-example built and timed by a designer. Every piece lands as additive schema + one materializer + one Renderer impl on the frozen v1-beta substrate — but the design is now honest about the pervasive layer (live values, animation physics, CSS cascade, interaction mechanics, and the trust root) that lives *outside* the graph, which is what makes the "everything is nodes + edges" claim *true* here instead of aspirational.

## (b) OPEN DECISIONS for Ben (the real forks)

Presented situation → options → my prediction, per the plain-English lens. These are genuine forks, not orchestrator calls.

- **A — Svelte-primary vs Solid-primary.** Both fit; Svelte wins on Tier-1 behavior-layer maturity (Bits/Melt > Kobalte) + your lean, and the pick is localized to one Renderer over a portable IR. *My prediction: confirm **Svelte primary, Solid sanctioned fallback** so R3/R5 can pivot without re-ratification.*
- **B — Rust-native (Leptos-IVM) as a funded parallel track now, or Phase-9+ watch-list.** It is the most thesis-aligned end-state (graph-authoring hides Rust; zero-serialization type-sharing deletes UI-layer drift machinery; no browser-a11y objection on native) but adds scope now. *My prediction: **fund it as a small parallel research track behind the Renderer trait, shape-(a) desktop first**, not on the ship-now critical path.*
- **C — Authoring-surface primary modality (the biggest adoption fork).** Visual-composer-primary (Figma-shaped, emits graph) vs DSL-primary vs graph-canvas-primary. The critique makes graph-canvas-primary a predictable failure at UI density. *My prediction: **visual-composer + DSL co-primary, graph-canvas as inspector**, with a designer-built worked example as the R1 go/no-go.*
- **D — Token expressiveness vs security/sanitizability.** Constrained typed-enum tokens (sanitizable, non-cascading, my design) vs richer CSS expressiveness (more designer power, reopens injection + cascade-in-the-graph). *My prediction: **start constrained**; add a named, sanitized escape hatch only if the worked example proves it insufficient.*
- **E — Framework architectural-fit vs hiring-pool (the honest React question).** Accept the bespoke-IR onboarding cost for architectural fit + IR-portability, or weight hiring-pool/ecosystem more and reconsider React-with-Radix as a *pragmatic* (if architecturally redundant) base. *My prediction: **accept the cost** — the durable asset is the toolkit-neutral IR, learned once — but I flag it as a real cost you may weight differently.*
- **F — Optimistic-write reconciliation model** under shape-(b) latency (local store write vs authoritative engine confirmation). Genuinely unsolved; needs real R1 design, not a default. *My prediction: **R1 design task**, likely optimistic-local + engine-confirmation-reconcile, but I won't pre-commit the model.*
- **G — Migration/rebuild scope acceptance.** The shipped admin-v0 must be rebuilt in the new IR before self-edit works; there is no free migration. *My prediction: **accept it as a rebuild** and frame the new IR as the successor generalization of the shipped schema-rendering vocabulary (the shipped materializer is the seed) — but you should see and sign off on an honest LOC/time estimate at R1.*

## (c) Suggested NEXT STEPS toward the Cluster-B (Composing-Admin) ADDL

1. **Ben ratifies the forks above (A–G).** A/D are quick confirms; C/E/G are the ones that reshape the plan — pause on those.
2. **Two R1 spikes before the R1 critic council** (cheap, de-risk the load-bearing unknowns): **(i) taxonomy stress-test** — build a sortable virtualized table, a rich-text Field, and the composition-editor-canvas itself against the 5/6/kernel set; does anything force a new node/edge? **(ii) CSP-strict interpreter spike** — prove the data-driven Svelte interpreter renders + hot-swaps with zero `unsafe-eval` and target-zero `unsafe-inline` (constructable stylesheets).
3. **Verify the three frozen-signature touchpoints** against the actual v1-beta signatures (Renderer input type; thin-client CALL+subscribe; CURRENT-flip cap segregation). These are the only places the additive design brushes the freeze — confirm additivity before R1 locks scope.
4. **Author the worked example** (a real settings screen) end-to-end in the intended authoring surface; count nodes/edges/CIDs and **time a designer** building it. This is the adoption go/no-go and should precede R2 test-landscape.
5. **Run the full Composing-Admin ADDL** per the pipeline: R0 (this doc, ratified) → **R1 critic council** (compose lenses by surface per Pattern 6 — the four critique lenses here are the seed set; add a *designer-persona* lens and a *frozen-interface-drift* lens) → iterate R1 to convergence → R2 test-landscape → R3 red-phase → R4 → R5 implementation (canary-first: the UI-materializer + render-tree schema is the canary the Svelte Renderer and every Tier-2 subgraph depend on) → R4b → R6 phase-close. Everything is additive to the frozen v1-beta substrate; nothing here reopens the freeze.
6. **Standing-guard the six guards + the TCB invariant** into the Composing plan-doc + dispatch-conventions at R0-ratification time, so later waves can't drift a UI `PrimitiveKind`, a payload-supplied principal, a live-value Node, or a self-editable validator back in.
---

## Addendum — R0 refinements from the Ben design conversation (2026-07-21; NOT-yet-ratified, feed Composing-Admin R1)

Three sharpenings surfaced when Ben interrogated the R0. They tighten the design; carry them into R1.

1. **The renderer is an ENGINE-LEVEL extension (CLAUDE.md #19), not an app-level plugin.** Clean split to hold: the UI *content* is an app-level subgraph (data — content-addressed, portable, editable-in-admin, UCAN-gated); the *renderer that paints it* is a Rust crate compiled into the binary (trust = "you compiled it in"), chosen by deployment shape (BrowserRender / TauriRender / future SvelteRender / Rust-native). Graph = score; renderer = orchestra. This is the same category as custom transports / storage backends / IVM strategies.

2. **Styling is graph, via typed STYLES→token nodes (Ben independently reinvented the `STYLES` edge).** A UI node's appearance is an EDGE to a typed style/design-token node, never an inline string — so the design system (tokens, themes, scales) is *itself* nodes+edges: content-addressed, forkable, versioned, editable-in-admin (change a token → everything using it restyles). REFINEMENT (security): typed/constrained tokens, NOT arbitrary CSS strings — raw CSS reopens injection + drags the cascade into the graph. Appearance-*intent* = graph; the cascade computation is sidestepped by the non-cascading token model.

3. **The mechanics line is "declare in the graph, execute in the renderer" — NOT "structure=graph, mechanics=renderer" (Ben's push; supersedes the coarser R0 framing).** Almost every mechanic splits: the DECLARATIVE half belongs in the graph, the IMPERATIVE runtime half in the renderer.
   - Animation: the *spec* ("slide-in 200ms ease-out") = graph node/property; the per-frame interpolation = renderer.
   - Interaction state: the *state machine* (states + transitions; the `STATE` edge) = graph; tracking which-state-now = renderer.
   - Focus / drag: the *declaration* (focus-trap here; reorderable here) = graph; the pointer/focus juggling = renderer.
   What genuinely stays out of the graph is **transient high-frequency runtime state** (current scroll px, in-flight animation frame, live drag position) — it changes 60×/s and has no durable/versionable meaning. So the true below-the-line is "ephemeral runtime state," not "mechanics." **Design directive for R1: push the declarative/graph line as far DOWN as it honestly goes** — pull every declarable mechanic (animation specs, state machines, focus/drag/dismissal declarations, style tokens) INTO the graph; leave only ephemeral runtime execution below.
   - **Parallel to lock in:** the renderer is the UI's `SANDBOX` — the bounded imperative escape hatch for what genuinely needs per-frame imperative code, exactly as SANDBOX is the engine's escape hatch from the declarative not-Turing-complete core. Honest caveat (critics): imperative mechanics are *more pervasive* in UI than SANDBOX is in engine work, so this hatch is wider — but the shape + discipline are identical.
