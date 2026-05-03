// R3-F red-phase — D10 snapshot-blob TS-side surface (if exposed).
//
// Surface scoping note (verified against G10-A-wasip1 plan §3 + dx-r1-2b):
//   The plan §3 G10-A-wasip1 deliverables list `Engine::from_snapshot_blob`
//   + `Engine::export_snapshot_blob` as the RUST surface. The TS-side
//   exposure is NOT explicitly enumerated in the plan, but the snapshot-blob
//   handshake is fundamentally a TS-callable boundary on the wasm32 browser
//   target (the host that exports the blob from a Node engine is the same
//   process that hands it to a wasm32-unknown-unknown engine in the
//   browser — both speak through the napi / wasm-bindgen bridge).
//
//   These tests anticipate the natural TS mirror (engine.exportSnapshotBlob,
//   Engine.fromSnapshotBlob) so the surface shape is locked at red-phase.
//   If G10-A-wasip1 / G10-A-browser elects to expose ONLY the Rust surface
//   and route TS callers through a different mechanism, R5 mini-review
//   re-shapes these tests then. Either way, pinning the shape early prevents
//   the wasm-target bring-up from inventing a TS surface that diverges from
//   the Rust one.
//
// Pin sources: brief §"Surface ownership" item 7; r2-test-landscape.md
// §2.3 D10 row + §8 D10; plan §3 G10-A-wasip1 + D10-RESOLVED.

import { describe, it, expect } from "vitest";
import { Engine, crud } from "@benten/engine";

describe("D10 snapshot-blob — TS surface round-trip", () => {
  it("engine.exportSnapshotBlob produces bytes that Engine.fromSnapshotBlob accepts", async () => {
    const src = await Engine.open(":memory:");
    const post = await src.registerSubgraph(crud("post"));
    await src.call(post.id, "post:create", { title: "first" });
    await src.call(post.id, "post:create", { title: "second" });

    // Export — the Rust side computes the canonical DAG-CBOR encoding; the
    // wrapper transports it as a Buffer / Uint8Array.
    const blob: Uint8Array = await src.exportSnapshotBlob();
    expect(blob.byteLength).toBeGreaterThan(0);

    // Import via the static constructor — mirrors the Engine.open factory
    // shape. The imported engine is a read-mostly view in Phase 2b
    // (snapshot_blob KVBackend is read-only per D10 BackendError::ReadOnly).
    const dst = await Engine.fromSnapshotBlob(blob);

    // The snapshot-blob carries the storage-layer Nodes (the post:create
    // writes from src above) but does NOT carry the in-memory handler
    // registry — `Engine::from_snapshot_blob` hydrates Nodes only. Re-
    // register the canonical CRUD handler on the dst engine so the
    // subsequent `post:list` dispatch resolves. Re-registering the
    // canonical CRUD shape is structurally identical so the post.id
    // round-trips. Note: registerCrud writes only to in-memory handler
    // tables (no backend mutation) — the read-only contract holds for
    // user-facing data writes (asserted in the sibling test below).
    const dstPost = await dst.registerSubgraph(crud("post"));
    expect(dstPost.id).toBe(post.id);

    // Reads round-trip — the dst engine sees the same labels + counts as src.
    const dstPosts = await dst.call(post.id, "post:list", {});
    expect(dstPosts.list).toBeDefined();
    expect(dstPosts.list!.length).toBe(2);

    await src.close();
    await dst.close();
  });

  it("snapshot-blob bytes are stable across repeated export (canonical bytes)", async () => {
    // D10 + sec-pre-r1-09: the snapshot-blob DAG-CBOR encoding sorts the
    // BTreeMap by key, so two exports of the same engine state produce
    // byte-identical output. Pins the canonical-bytes discipline at the TS
    // boundary so a future PR that switches to a non-canonical encoder
    // trips this test, not a downstream sync bug.
    const e = await Engine.open(":memory:");
    const post = await e.registerSubgraph(crud("post"));
    await e.call(post.id, "post:create", { title: "x" });

    const a = await e.exportSnapshotBlob();
    const b = await e.exportSnapshotBlob();
    expect(Buffer.from(a).equals(Buffer.from(b))).toBe(true);

    await e.close();
  });

  it("Engine.fromSnapshotBlob rejects writes (read-only contract)", async () => {
    // D10 BackendError::ReadOnly — the TS surface MUST surface this as a
    // typed error so callers don't silently corrupt the dst engine by
    // attempting writes against a snapshot-blob-backed instance.
    const src = await Engine.open(":memory:");
    const post = await src.registerSubgraph(crud("post"));
    const blob = await src.exportSnapshotBlob();

    const dst = await Engine.fromSnapshotBlob(blob);
    // Re-register the handler on the dst engine — registerCrud touches
    // only the in-memory handler tables (no backend write), so the
    // read-only contract holds for the subsequent `post:create` write
    // attempt that this test pins.
    await dst.registerSubgraph(crud("post"));

    await expect(dst.call(post.id, "post:create", { title: "x" })).rejects.toMatchObject({
      code: "E_BACKEND_READ_ONLY",
    });

    await src.close();
    await dst.close();
  });

  // R6-R3 r6-r3-arch-1 — load-bearing end-to-end test pin (per
  // dispatch-conventions.md §3.6b). The MAJOR finding was that PR #68
  // wired the read-only-snapshot enforcement at PrimitiveHost::put_node
  // ONLY — leaving PrimitiveHost::delete_node undefended. A handler
  // dispatched via `engine.call(handler, ':delete', {cid})` against a
  // snapshot-blob engine SILENTLY DELETED nodes, bypassing D10. This
  // test drives the production `engine.call` entry point through the
  // CRUD `delete` action (which routes to `PrimitiveHost::delete_node`
  // via the WRITE primitive's op="delete" + target_cid path) and
  // asserts the call rejects with E_BACKEND_READ_ONLY. The earlier
  // "rejects writes" test only exercised the put-direction (post:create
  // → put_node) so the delete-direction gap was silently undefended.
  // Would FAIL if `delete_node` were silently no-op'd back to its
  // pre-fix permissive behavior.
  it("Engine.fromSnapshotBlob rejects deletes (read-only contract — delete-path symmetry r6-r3-arch-1)", async () => {
    const src = await Engine.open(":memory:");
    const post = await src.registerSubgraph(crud("post"));
    // Create a node so the dst engine has something to attempt to
    // delete; capture its CID so the delete dispatch resolves to a real
    // target_cid (rather than the delete_missing route).
    const created = await src.call(post.id, "post:create", { title: "deletable" });
    expect(created.cid).toBeDefined();
    const blob = await src.exportSnapshotBlob();

    const dst = await Engine.fromSnapshotBlob(blob);
    await dst.registerSubgraph(crud("post"));

    // Sanity: the snapshot blob carried the created node — listing the
    // dst should surface it. This pin is BEFORE the delete attempt so a
    // future regression that loses the snapshot-import won't masquerade
    // as a passing read-only test.
    const dstPosts = await dst.call(post.id, "post:list", {});
    expect(dstPosts.list).toBeDefined();
    expect(dstPosts.list!.length).toBe(1);

    // Load-bearing assertion: delete-via-dispatch against a snapshot-blob
    // engine MUST surface E_BACKEND_READ_ONLY. Pre-r6-r3-arch-1 this
    // SUCCEEDED silently (PrimitiveHost::delete_node had no
    // is_read_only_snapshot guard).
    await expect(
      dst.call(post.id, "post:delete", { cid: created.cid }),
    ).rejects.toMatchObject({
      code: "E_BACKEND_READ_ONLY",
    });

    // Defence-in-depth: after the rejected delete, the node MUST still
    // be present in the dst engine — proving the delete didn't partially
    // apply before the error surfaced.
    const afterReject = await dst.call(post.id, "post:list", {});
    expect(afterReject.list!.length).toBe(1);

    await src.close();
    await dst.close();
  });
});
