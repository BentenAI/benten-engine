/**
 * Algorithm A: Eager Full Invalidation
 *
 * On any write, mark ALL views that COULD be affected as dirty.
 * On next read of a dirty view, recompute it from scratch.
 *
 * - Simplest. Baseline.
 * - Read is O(N) when dirty, O(1) when clean.
 * - Write is O(1) (just mark dirty).
 */

import type {
  IVMAlgorithm,
  GraphStore,
  ViewDefinition,
  ViewResult,
  WriteOp,
  PropertyValue,
} from '../types.js';
import {
  computeEventHandlerView,
  eventHandlerAffected,
  computeCapabilityView,
  capabilityCheckAffected,
  computeContentListingView,
  contentListingAffected,
  computeGovernanceView,
  governanceAffected,
  computeAttestationView,
  attestationAffected,
} from '../views/view-definitions.js';

interface CachedView {
  result: ViewResult;
  dirty: boolean;
  recomputeFn: () => Record<string, PropertyValue>[];
  affectedFn: (op: WriteOp, graph: GraphStore) => boolean;
}

export class EagerInvalidationAlgorithm implements IVMAlgorithm {
  readonly name = 'A: Eager Full Invalidation';
  private graph!: GraphStore;
  private views = new Map<string, CachedView>();

  initialize(graph: GraphStore, viewDefs: ViewDefinition[]): void {
    this.graph = graph;
    this.views.clear();

    for (const def of viewDefs) {
      const { recomputeFn, affectedFn } = this.createViewFunctions(def.id);
      const rows = recomputeFn();
      this.views.set(def.id, {
        result: { rows, dirty: false, lastUpdated: performance.now() },
        dirty: false,
        recomputeFn,
        affectedFn,
      });
    }
  }

  private createViewFunctions(viewId: string): {
    recomputeFn: () => Record<string, PropertyValue>[];
    affectedFn: (op: WriteOp, graph: GraphStore) => boolean;
  } {
    switch (viewId) {
      case 'event_handlers':
        return {
          recomputeFn: () => computeEventHandlerView(this.graph, 'content:afterCreate'),
          affectedFn: eventHandlerAffected,
        };
      case 'capability_check':
        return {
          recomputeFn: () => {
            const capMap = computeCapabilityView(this.graph);
            return Array.from(capMap.entries()).map(([key, val]) => ({ key, granted: val }));
          },
          affectedFn: capabilityCheckAffected,
        };
      case 'content_listing':
        return {
          recomputeFn: () => computeContentListingView(this.graph, 20),
          affectedFn: contentListingAffected,
        };
      case 'governance_rules':
        return {
          recomputeFn: () => computeGovernanceView(this.graph, 'grove-leaf-0'),
          affectedFn: governanceAffected,
        };
      case 'attestation_aggregate':
        return {
          recomputeFn: () => {
            const agg = computeAttestationView(this.graph, 'knowledge-0');
            return [agg];
          },
          affectedFn: attestationAffected,
        };
      default:
        throw new Error(`Unknown view: ${viewId}`);
    }
  }

  applyWrite(op: WriteOp): void {
    // Apply to graph
    this.graph.applyWrite(op);

    // Mark affected views as dirty (O(views) check, but just marking is O(1))
    for (const [, cached] of this.views) {
      if (cached.affectedFn(op, this.graph)) {
        cached.dirty = true;
      }
    }
  }

  readView(viewId: string): ViewResult {
    const cached = this.views.get(viewId);
    if (!cached) throw new Error(`Unknown view: ${viewId}`);

    if (cached.dirty) {
      // Full recompute
      const rows = cached.recomputeFn();
      cached.result = { rows, dirty: false, lastUpdated: performance.now() };
      cached.dirty = false;
    }

    return cached.result;
  }

  memoryOverhead(): number {
    // Just storing the cached results + dirty flags
    let bytes = 0;
    for (const [, cached] of this.views) {
      // Rough estimate: 100 bytes per row + overhead
      bytes += cached.result.rows.length * 100 + 64;
    }
    return bytes;
  }

  reset(): void {
    this.views.clear();
  }
}
