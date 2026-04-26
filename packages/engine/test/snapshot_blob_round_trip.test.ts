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

    await expect(dst.call(post.id, "post:create", { title: "x" })).rejects.toMatchObject({
      code: "E_BACKEND_READ_ONLY",
    });

    await src.close();
    await dst.close();
  });
});
