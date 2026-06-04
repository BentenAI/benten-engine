// G24-C wave-6b SUBSTANTIVE pin (un-ignored; D-4F-2 consumer pin).
//
// Asserts the composed-view creator emits a view whose representation
// is a subgraph shape consumed by the generalized IVM Algorithm B
// kernel (D-4F-2 + G23-0). Pins admin UI v0 as a consumer of D-4F-2's
// IVM-subgraph generalization — the failure mode being defended
// against is admin UI v0 introducing a parallel view-materialization
// path that diverges from the engine's IVM kernel.
//
// ## Closes
//
// G24-C + D-4F-2 consumer (`r2-test-landscape.md` §2.8 row 1)

import { describe, test, expect } from "vitest";
import {
  ComposedViewCreator,
  type ComposedViewCreatorBridge,
  type UserViewSpec,
} from "../src/index.js";

// In-memory test bridge that exercises the substantive D-4F-2
// contract: `registerSubgraphView` MUST forward the spec through the
// kernel pathway (we record it for assertion) + `materializerProvenance`
// MUST return the canonical Algorithm B kernel identifier. A parallel
// admin-ui-only materialization path would surface as a non-matching
// provenance string OR as a divergence between the spec the creator
// emitted and the spec the bridge captured.
class IvmRoundTripBridge implements ComposedViewCreatorBridge {
  public registeredSpec: UserViewSpec | null = null;
  public registeredPrincipal: string | null = null;
  public registeredCid: string = "";

  async readNodeAs(_principal: string, _cid: string): Promise<unknown> {
    return null;
  }
  async onChangeAsWithCursor(): Promise<{ readonly subscriptionId: string }> {
    return { subscriptionId: "unused-in-emit-test" };
  }
  async registerSubgraphView(
    spec: UserViewSpec,
    principal: string,
  ): Promise<{ readonly cid: string }> {
    this.registeredSpec = spec;
    this.registeredPrincipal = principal;
    // Synthesise a content-addressed CID stand-in. The Rust-side test
    // pin asserts CID stability against the canonical hashing path;
    // this test pin asserts the spec ROUND-TRIPS through the bridge
    // unchanged.
    const cid = `bafy-test-${spec.viewId}`;
    this.registeredCid = cid;
    return { cid };
  }
  async materializerProvenance(cid: string): Promise<string> {
    if (cid !== this.registeredCid) {
      throw new Error(
        `materializerProvenance: unknown cid ${cid} (expected ${this.registeredCid})`,
      );
    }
    // The canonical kernel identifier per D-4F-2 + G23-0a. A parallel
    // admin-ui-only materializer would surface a different string here.
    return "benten-ivm::strategy::AlgorithmB";
  }
  async revokeCapability(): Promise<void> {
    /* not used in this test */
  }
  async callAs(): Promise<{ readonly cid: string }> {
    return { cid: "unused-in-emit-test" };
  }
}

describe("composed_view_creator_emits_subgraph_shaped_view_via_generalized_ivm (D-4F-2 consumer)", () => {
  test("composed view representation is a subgraph; materializes via IVM kernel", async () => {
    const bridge = new IvmRoundTripBridge();
    const principal = "did:key:test-user-principal";

    // Production arm (G24-C wave-6b): mount → select → save.
    const creator = ComposedViewCreator.mount({ bridge, principal });
    await creator.selectAnchorPattern("notes-by-tag");
    await creator.selectProjection(["title", "body"]);
    const result = await creator.save({ name: "notes-by-work-tag" });

    // The persisted view's spec captured by the bridge MUST carry the
    // D-4F-2 kernel-input shape — `viewId` + `labelPattern` +
    // `projection` mirror `benten_ivm::subgraph_spec::SubgraphSpec`.
    expect(bridge.registeredSpec).not.toBeNull();
    const spec = bridge.registeredSpec!;
    expect(spec.viewId).toBe("notes-by-work-tag");
    expect(spec.labelPattern).toEqual({
      kind: "exact",
      label: "notes-by-tag",
    });
    expect([...spec.projection]).toEqual(["title", "body"]);
    // User-defined views ALWAYS carry `typedOutputProjection: null` per
    // the Rust-side `SubgraphSpec::user_view` constructor. Canonical
    // views 4/5 (governance_inheritance/version_current) are the only
    // shape with non-null typed-output.
    expect(spec.typedOutputProjection).toBeNull();
    // Self-reference flag MUST be false at user-defined-view emit time
    // (the kernel's `register_subgraph` rejects `true` with
    // `SelfReferentialSubgraphRejected` per mat-r1-13).
    expect(spec.selfReferential).toBe(false);
    // The captured principal is the user's DID (the walk-principal the
    // kernel attributes future change events to).
    expect(bridge.registeredPrincipal).toBe(principal);

    // Materialization path: the view's provenance string is the
    // canonical generalized Algorithm B kernel identifier. A parallel
    // admin-ui-only materializer would surface a different string.
    const provenance = await bridge.materializerProvenance(result.cid);
    expect(provenance).toBe("benten-ivm::strategy::AlgorithmB");

    // §3.6f SUBSTANCE: the saved CID is non-empty + the bridge's
    // recorded CID matches what `save` returned. Would-FAIL-if-no-op'd:
    // a no-op save() that returned an empty CID + skipped the bridge
    // call would fail this assertion.
    expect(result.cid.length).toBeGreaterThan(0);
    expect(bridge.registeredCid).toBe(result.cid);
  });

  test("user_view spec rejects collision with canonical view ids per Rust-side constructor mirror", async () => {
    const bridge = new IvmRoundTripBridge();
    const creator = ComposedViewCreator.mount({
      bridge,
      principal: "did:key:test-principal",
    });
    await creator.selectAnchorPattern("notes-by-tag");
    await creator.selectProjection(["title"]);
    // Picking `capability_grants` (one of the 5 canonical view ids per
    // `benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS`) MUST be rejected
    // — the user-view constructor enforces this so callers don't
    // accidentally shadow a hardcoded canonical kernel.
    await expect(
      creator.save({ name: "capability_grants" }),
    ).rejects.toThrow(/canonical view id/);
  });
});
