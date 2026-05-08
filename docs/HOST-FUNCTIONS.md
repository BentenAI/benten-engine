# SANDBOX Host Functions

This document is the **operator-facing surface** for the SANDBOX
host-function set. The authoritative codegen source is
[`host-functions.toml`](../host-functions.toml) at the workspace root.
Drift is enforced bidirectionally by
`crates/benten-engine/tests/host_functions_doc_drift_against_toml.rs`
(TOML → MD) and
`crates/benten-engine/tests/host_functions_md_drift_against_toml.rs`
(MD → TOML); any addition / rename / deletion in the TOML must be
mirrored in this file or CI fails.

Phase 3 (G17-A2) ships **four** host functions (`time`, `log`, `kv:read`,
`random`) and **two** named manifests (`compute-basic`,
`compute-with-kv`). The `random` host-fn is the Phase-3 lift of the
Phase-2b deferral (CLAUDE.md baked-in #16 / Compromise #16 closure):
workspace CSPRNG = `getrandom` direct (per **D-PHASE-3-11**
RESOLVED-at-R1); per-call entropy budget defaults to 4096 bytes
(per **r1-wsa-8**); module manifests may override via the additive
optional `host_fns.random.budget_bytes_per_call` field on
`ModuleManifest`.

**Runtime status (post-wave-8b/8h):** the host-fn trampoline is fully
wired through wasmtime's `Linker::func_wrap`; every listed failure mode
fires from a real production execution path. The wave-8b runtime
invocation pipeline (Compromise #4 closure narrative) closed the prior
"declarations + cap-recheck schema landed; live trampoline pending"
disclaimer that earlier drafts of this doc carried. Named-manifest
dispatch (`manifest: "<name>"`) consults the engine's `installed_modules`
state via the wave-8h `manifest_registry()` accessor, so a SANDBOX node
referencing an `install_module`-installed manifest resolves through the
production lookup path rather than failing with
`E_SANDBOX_MANIFEST_UNKNOWN`.

## How to call a host function

A SANDBOX node names either a manifest (D2 hybrid `by_name`) or an
explicit cap set (`by_caps`). Either path resolves to the same
`CapBundle` enforced by the `wasmtime` host. The WASM module then
`extern`-imports each host function it needs from the `benten` module
namespace; calls fail with a typed error if the bundle does not include
the required cap.

```typescript
import { subgraph } from "@benten/engine";

// Manifest-by-name (preferred — D2 named-bundle DX sugar):
const handler = subgraph("summarize")
  .read({ label: "doc", by: "id", value: "$input.doc_id" })
  .sandbox({
    module: "summarizer-v1",          // CID of an installed module
    manifest: "compute-with-kv",      // includes time + log + kv:read
    fuel: 1_000_000,                  // wasmtime fuel cap
    wallclockMs: 30_000,
    outputLimitBytes: 1_048_576,
  })
  .respond({ body: "$result" })
  .build();

// Explicit-caps (advanced — D2 by_caps for ad-hoc bundles):
.sandbox({
  module: "tinyhash-v1",
  caps: ["host:compute:log", "host:compute:time"],   // sorted
  fuel: 250_000,
})
```

## Per-call enforcement model

| Field | Default | Surface |
|-------|---------|---------|
| `cap_recheck` | `per_call` | When the host trampoline re-consults the policy. `per_call` means every host-fn invocation; `per_boundary` means once at SANDBOX entry (D18 fail-secure default). |
| `bypass_output_budget` | `false` | If `false`, the host-fn's wire-bytes count against the SANDBOX output budget (centralized D17 PRIMARY trampoline counting). |
| `requires_async` | `false` | Reserved for future async host-fns; through Phase 3 every host-fn is sync (D19). Phase-3 iroh-driven sync runs entirely inside `benten-sync`, not as a SANDBOX host-fn, so this carrier is still unused at Phase 3 close. |

A host-fn invocation that exhausts a per-call cap fires
`E_INV_SANDBOX_OUTPUT` (Inv-7) at the trampoline boundary; the SANDBOX
node's outcome is `E_SANDBOX_HOST_FN_BUDGET_EXCEEDED` with the
host-fn name in the diagnostic.

## Compatibility — `since:` annotations

Every host-fn is annotated with the Phase in which it landed. Phase
2b's set is the post-Phase-2a baseline. Future additions append to the
TOML; `since` defaults to `2b` for omitted entries (codegen pin).

---

## host_fn.time

- **Cap required:** `host:compute:time`
- **Recheck cadence:** `per_boundary`
- **Since:** 2b (G7-A)
- **Description:** Returns monotonic time, **coarsened to 100 ms granularity** per ESC-16 + sec-pre-r1-06 §2.1.

### Argument schema

```text
() -> i64    // milliseconds since an opaque process-start epoch
```

The return value is monotonic-non-decreasing within one process. The
100 ms coarsening closes the timezone-leak + clock-fingerprinting side
channel that motivated the compromise: a SANDBOX module cannot
distinguish two host invocations that fall inside the same 100 ms
bucket, which defeats most timing-based exfiltration attacks.

### Permission semantics

`per_boundary` recheck means the policy is consulted **once at SANDBOX
entry**. A revoke between the entry check and the host-fn call still
permits the call; revoke is observed at the next SANDBOX boundary.
This is the looser of the two cadences and matches the threat-model
verdict in sec-pre-r1-06: timing read-only is an extremely-low-impact
side channel relative to per-call recheck overhead.

### Failure modes

- Cap missing at SANDBOX entry → `E_CAP_DENIED` (`host:compute:time`).
- Engine clock not initialised (initialisation race) → `E_SANDBOX_HOST_FN_INTERNAL`.

---

## host_fn.log

- **Cap required:** `host:compute:log`
- **Recheck cadence:** `per_boundary`
- **Per-call byte cap:** 64 KiB (65 536 bytes; sec-pre-r1-06 §2.2)
- **Since:** 2b (G7-A)
- **Description:** Writes a string from the SANDBOX module to the engine log sink.

### Argument schema

```text
(ptr: i32, len: i32) -> ()    // utf-8 bytes in WASM linear memory
```

The host trampoline copies up to 64 KiB out of the module's linear
memory per call. Larger payloads truncate with a one-line marker
appended to the sink (`"<truncated, NN bytes dropped>"`); the call
returns success — truncation is observably distinct from failure but
not error-coded.

### Permission semantics

`per_boundary` recheck — same cadence as `time`. The 64 KiB per-call
byte cap (NOT the per-SANDBOX cumulative output cap) is enforced at
the trampoline; exceeding it within a single host-fn call is
truncation, not error.

### Failure modes

- Cap missing → `E_CAP_DENIED` (`host:compute:log`).
- Engine log sink unavailable (Phase 3+ when sink is configurable; tracked separately from §6.10) → `E_SANDBOX_HOST_FN_INTERNAL`.
- Cumulative SANDBOX output budget exceeded → `E_INV_SANDBOX_OUTPUT` (Inv-7).

---

## host_fn."kv:read"

- **Cap required:** `host:compute:kv:read`
- **Recheck cadence:** `per_call` (TOCTOU defense)
- **Per-call read cap:** 1 000 reads (sec-pre-r1-06 §2.4 — read-amplification DOS bound)
- **Since:** 2b (G7-A)
- **Description:** Reads a value by CID from the engine KV backend, subject to caps.

### Argument schema

```text
(cid_ptr: i32, cid_len: i32, out_ptr: i32, out_cap: i32) -> i32
// returns: positive = number of bytes copied; -1 = not found; -2 = cap denied
```

The CID is parsed off the WASM module's linear memory (CIDv1 bytes,
length must match the multibase-decoded form). Lookup goes through the
engine's `KVBackend::get_blob` plumbing; the per-call recheck consults
the configured `CapabilityPolicy` for `host:compute:kv:read` against
the **resolved-Node's label** (not just the cap-string), giving
label-scoped revoke fidelity. A denial post-entry returns `-2` to the
module and re-charges nothing against the read budget.

### Permission semantics

`per_call` recheck — the **strictest** cadence in the Phase-2b
host-fn set. Every invocation re-asks the policy whether
`host:compute:kv:read` is still granted on this caller. A revoke that
lands between two host-fn calls within the same SANDBOX execution is
observed on the **next call**, returning `-2`. This closes the TOCTOU
window the looser `per_boundary` cadence would leave open for
sensitive read paths.

The 1 000-call read budget (per **single SANDBOX primitive call**, not
per WASM module instance) bounds read-amplification DOS — a malicious
or buggy module cannot loop-read indefinitely against the engine
backend.

### Failure modes

- Cap missing at first call → `E_CAP_DENIED` (`host:compute:kv:read`).
- Cap revoked mid-execution → host-fn returns `-2`; module decides whether to abort.
- 1 000-read budget exhausted → `E_SANDBOX_HOST_FN_BUDGET_EXCEEDED` (`kv:read`).
- CID malformed in linear memory → `E_SANDBOX_HOST_FN_INVALID_ARGS`.
- Cumulative SANDBOX output budget exceeded → `E_INV_SANDBOX_OUTPUT` (Inv-7).

---

## Named manifests

A manifest bundles a sorted cap-list under a single name. Modules
declared with `manifest: "compute-basic"` get the bundled caps without
having to enumerate them; manifests are codegen-emitted into a static
registry (`ManifestRegistry`) so the named-by-name path is free of
runtime registration.

`register_runtime(name, bundle)` is **reserved** in 2b — calls return
`E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED`. Phase 8 (the marketplace)
lifts the deferral.

### Manifest sorting

Each manifest's `caps` list **MUST be sorted** for DAG-CBOR
canonical-bytes stability per D9. The codegen pipeline asserts
sorted-canonical at build time; an unsorted manifest entry is a
build-fail.

### compute-basic

- **Caps:** `host:compute:log`, `host:compute:time`
- **Description:** Time + log (no KV, no network). The cheapest
  compute-only manifest; appropriate for pure-function modules
  (formatters, hashers, parsers).

### compute-with-kv

- **Caps:** `host:compute:kv:read`, `host:compute:log`, `host:compute:time`
- **Description:** `compute-basic` + `kv:read` (`per_call` cap-recheck).
  The manifest for read-side enrichment modules (joiners, summarisers
  that need to fetch related Nodes by CID).

---

## host_fn.random

- **Cap required:** `host:random:read`
- **Recheck cadence:** `per_call` (entropy is sensitive; mirrors
  `kv:read`'s TOCTOU posture)
- **Per-call entropy budget:** 4096 bytes (default per r1-wsa-8;
  per-manifest override available)
- **Since:** 3 (G17-A2)
- **Description:** Fills a guest buffer with CSPRNG bytes drawn from
  `getrandom` (the workspace CSPRNG decision per D-PHASE-3-11
  RESOLVED-at-R1; CLAUDE.md baked-in #16 closure).

### Argument schema

```text
(out_ptr: i32, out_len: i32) -> i32
// returns: 0 = success (out_len bytes written to guest memory at out_ptr)
// typed denials surface through the host-fn ABI (NOT a wasmtime trap)
```

The trampoline writes `out_len` bytes of CSPRNG entropy into the
guest's linear memory at `out_ptr`. Bounds-check failures (`out_ptr`
or `out_ptr + out_len` past the module's declared memory) trap as
`MemoryOutOfBounds`. CSPRNG draw failures (operationally never
expected on supported platforms) collapse to `E_SANDBOX_MODULE_INVALID`
with the underlying `getrandom` error in the message.

### Permission semantics

`per_call` recheck — every invocation re-asks the policy whether
`host:random:read` is still granted on this caller. A revoke that
lands between two host-fn calls within the same SANDBOX execution is
observed on the next call. The 4096-byte default budget is per
INVOCATION (each call must fit) per r1-wsa-8; the cumulative
per-primitive output budget enforced at `CountedSink` is the second
ceiling that bounds aggregate entropy across multiple calls.

A module manifest may override the per-call budget via the additive
optional field:

```toml
[host_fns.random]
budget_bytes_per_call = 1024  # tighter than the 4096 default
```

This flows through the engine's `ModuleManifest::host_fns.random.budget_bytes_per_call`
into `SandboxConfig::random_budget_bytes_per_call` at SANDBOX dispatch
(per `crates/benten-engine/src/primitive_host.rs::execute_sandbox`).

### Failure modes

- Cap missing / revoked at host-fn entry → `E_SANDBOX_HOST_FN_DENIED`.
- Single-call entropy request exceeds the per-call budget →
  `E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED` (cap-string carrier
  identifies `random:per_call_budget_exceeded` + carries
  `requested=<n>` + `budget=<n>` for operator hints).
- `out_ptr` / `out_ptr+out_len` outside guest memory → wasmtime
  `MemoryOutOfBounds` trap (mapped to `E_SANDBOX_MODULE_INVALID`).
- Cumulative SANDBOX output budget exceeded → `E_INV_SANDBOX_OUTPUT`.

---

## Where the source-of-truth lives

| Artifact | Path | Role |
|----------|------|------|
| Source-of-truth TOML | [`host-functions.toml`](../host-functions.toml) | Authoritative — drives drift detectors. Codegen `build.rs` pipeline is aspirational (deferred to Phase-3+ per the `#[ignore]`'d tests at `crates/benten-eval/tests/sandbox_capability_intersection_at_init.rs:119` + `sandbox_named_manifest.rs:5`); current state is hand-mirrored. |
| Hand-mirrored host shim | `crates/benten-eval/src/sandbox/host_fns.rs` | The actual file — a single `.rs` (not codegen output), kept in lockstep with the TOML by drift detectors |
| Operator doc | `docs/HOST-FUNCTIONS.md` (this file) | Surface contract for handler authors |
| TOML→MD drift detector | `crates/benten-engine/tests/host_functions_doc_drift_against_toml.rs` | Asserts every TOML entry has a doc section |
| MD→TOML drift detector | `crates/benten-engine/tests/host_functions_md_drift_against_toml.rs` | Asserts no fictional doc entries |
