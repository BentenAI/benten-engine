# `benten-graph` benchmark suite

Four criterion 0.8 benches covering the storage layer's hot paths. Their
purpose, gate policy, and the Phase-1 compromises shaping each one are
documented here; read this before tweaking numbers or retargeting CI gates.

## Quick reference

| Bench                    | Scope                                          | Gate? |
|--------------------------|------------------------------------------------|-------|
| `get_create_node`        | `get_node` hot-cache + `create_node` immediate | Gated — §14.6 direct |
| `durability_modes`       | Immediate vs. Group vs. Async write cost        | Informational (§14.6 derived) |
| `concurrent_writers`     | Writes/sec at 1/2/4/8/16 writer threads         | Informational (§14.6 direct) |
| `multi_mb_roundtrip`     | 1 MB / 10 MB / 100 MB put+get round-trip        | Informational (no §14.6 entry) |

"Gated" benches fail CI if the median drifts outside the documented range.
"Informational" benches always pass; the trend line across releases is the
signal worth tracking, not a single-point threshold.

## `get_create_node`

Owned by G2 (already green at spike end). Measures the two §14.6 direct
targets at the storage layer:

| Bench                                       | §14.6 target                        |
|---------------------------------------------|-------------------------------------|
| `get_node/hot_cache`                        | 1–50 µs hot cache                   |
| `get_node_batch_100/hot_cache_same_cid`     | amortized < 50 µs per lookup        |
| `create_node_immediate/default_durability`  | 100–500 µs realistic, Immediate     |

Engine-level equivalents that add public-API overhead live in
`crates/benten-engine/benches/roundtrip.rs`; the delta between the two
layers is the bindings + API cost.

## `durability_modes`

Owned by G3-B (this file). Produces three numbers per mode — single-write
latency, batch-100 commit latency, and sustained single-write throughput —
so the Immediate / Group / Async delta is visible at the op, transaction,
and throughput layers.

### What each sub-bench measures

| Bench id                                       | Workload                                      | Why                                                                 |
|------------------------------------------------|-----------------------------------------------|---------------------------------------------------------------------|
| `durability_modes/single_write/<mode>`         | 1 `put` → 1 commit                            | Audit-trail / capability-grant workload (one fsync per record)      |
| `durability_modes/batch_100/<mode>`            | 100 puts → 1 commit via `put_batch`           | Bulk import workload; amortizes commit overhead across the batch    |
| `durability_modes/throughput/<mode>`           | tight `put` loop over a fixed wall-clock      | Sustained writes/sec under best-case contention (single writer)     |

Criterion reports `time/iter`. For `throughput/<mode>`, that number is the
per-write elapsed time — invert to get writes/sec (e.g., 200 µs/iter = 5k
writes/sec). `batch_100` is tagged `Throughput::Elements(100)` so criterion
prints "elem/sec" directly.

### Modes

* **`Immediate`** — `redb::Durability::Immediate`; fsync before commit
  returns. Strongest durability. Default for disk-backed stores.
* **`Group`** — *intended* to batch fsyncs across commits. **Phase 1
  collapses to `Immediate`** because redb v4 exposes only
  `Durability::Immediate` / `Durability::None`. The construction path
  emits a one-shot stderr warning so operators aren't misled by
  benchmark output. The bench **demonstrates the collapse**: single-
  write and throughput numbers for `group` should be statistically
  indistinguishable from `immediate`. When Phase 2 revisits (if redb
  grows grouped-commit support) this bench becomes the regression
  signal that the amortization actually delivers a win.
* **`Async`** — `redb::Durability::None`; commit returns before fsync.
  Durability is best-effort; a crash may lose the last several commits.
  Test-only / ephemeral-view friendly.

### Expected delta

* `single_write/async` should be roughly 5–10× faster than
  `single_write/immediate` on a typical NVMe SSD — the fsync dominates
  the commit cost when durability is on. If `async` isn't clearly
  faster, the filesystem is ignoring fsync (tmpfs, `/dev/shm`, certain
  CI overlays) or the workload is CPU-bound in serialization rather
  than I/O-bound in fsync.
* `batch_100/<mode>` should be *much* less than 100× the
  `single_write/<mode>` cost — one commit covers the whole batch, so
  amortization is visible.
* `group` ≈ `immediate` at every sub-bench. Any `group < immediate`
  measurement is a signal the collapse was bypassed; any
  `group > immediate` is noise.

### Phase-1 gate contract

Plan §3 names the G3 row target as "Group bench hits < 500 µs per
write". Because Group collapses to Immediate, the same number drives
both. On a healthy NVMe this holds; on slow filesystems (cold disks,
FUSE, CI runners with disk-backed overlay FS) the target may not. The
bench surfaces the measured number honestly — it does not adjust the
window to pass. **Informational-not-gated** until the Group collapse is
resolved in Phase 2.

### Phase-1 compromises (this bench specifically)

1. **Group collapses to Immediate.** Documented in
   `RedbBackend::open_or_create_with_durability` and re-surfaced as a
   one-shot stderr warning. The enum variant is preserved so consumers
   don't face a semver break when Phase 2 wires real grouped commit.
2. **No separate fsync-latency histogram.** We measure
   total-commit-latency, not the fsync component in isolation. Peeling
   those apart requires `fstrace`/`strace`/equivalent and is a Phase 2
   profiling exercise, not a criterion bench.
3. **2-second default throughput window.** The plan calls for 5-second
   sustained measurement; the bench defaults to 2 s so CI wall-clock
   stays bounded. Pass `cargo bench -p benten-graph --bench
   durability_modes -- --measurement-time 5` for the on-spec window.

## `concurrent_writers`

Owned by G3-B (this file). redb serializes writes on a single writer
lock, so this bench characterizes the contention curve rather than
discovering parallelism that doesn't exist. N writer threads (1, 2, 4,
8, 16) write disjoint keys against the same backend; criterion's
`Throughput::Elements(N)` reports the throughput at each N.

Expected shape: roughly flat writes/sec across all N, with tail
latency growing as N increases (more threads queuing on the same
lock). Any curve that *scales* (throughput climbing with N) indicates
redb changed its locking model or we broke the single-writer
invariant somehow — either is worth investigating.

Tail-latency (p50/p95/p99 per-op) characterization is explicitly NOT
in this bench's scope; criterion's output is a median ± confidence
interval across iterations. Per-op distributions are owned by the
integration-test landscape (`tests/integration/contention_*`).

## `multi_mb_roundtrip`

Owned by G3-B (this file). Payload sizes: 1 MB, 10 MB, 100 MB. Each
iteration puts a Node whose `blob` property is `Value::Bytes(vec![0;
size])` and immediately reads it back by CID, asserting the CID is
stable across puts. Throughput is reported in MB/s.

Expected behavior:

* Put cost scales linearly with payload size (DAG-CBOR encoding is O(n);
  redb page writes are O(n)).
* Get cost similarly linear.
* MB/s throughput should be roughly constant across sizes — the
  constant factor is the interesting signal. A >2× drop between sizes
  points at an upstream redb regression or a CBOR-encoder allocation
  pattern that scales super-linearly.

**Memory note:** at 100 MB, each iteration transiently holds ~200 MB
of heap (encoded buffer + decoded Node). Systems with <2 GB free RAM
should skip the 100 MB variant by editing the `size_mb` array.

## Running the suite

```bash
# All benches, default criterion windows
cargo bench -p benten-graph

# Just durability_modes, fast CI window
cargo bench -p benten-graph --bench durability_modes -- --measurement-time 2

# Just durability_modes, on-spec 5-second throughput window
cargo bench -p benten-graph --bench durability_modes -- --measurement-time 5

# Compile-only check (matches the CI gate)
cargo bench --no-run -p benten-graph
```

Baselines land in `target/criterion/` as HTML reports; comparing across
commits is a `cargo bench --baseline <name>` workflow (see criterion
0.8 docs).

## Interpreting surprises

| Observation                                          | Likely cause                                                                           |
|------------------------------------------------------|----------------------------------------------------------------------------------------|
| `group` faster than `immediate`                      | Group-collapse warning bypassed, or redb grew grouped-commit (revisit `to_redb_durability`) |
| `async` ≈ `immediate`                                | Filesystem ignoring fsync (tmpfs, overlay FS) or CPU-bound in serialization           |
| `concurrent_writers` *scales* with thread count      | redb changed its locking model, or single-writer invariant broken upstream            |
| `multi_mb_roundtrip` super-linear in payload size    | redb page-store regression, or CBOR encoder allocating Θ(n²)                          |
| `get_node/hot_cache` > 50 µs                         | redb page cache cold (warm-up not hitting), or index read path regressed              |
| `create_node_immediate` > 500 µs                     | Disk subsystem slow (FUSE, network-backed FS), or extra index write crept in          |

When a bench fails on CI, the first debugging step is almost always to
check the disk subsystem — fsync-heavy workloads are unusually
sensitive to the runner's storage.
