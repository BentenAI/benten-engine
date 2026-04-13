/**
 * Algorithm C: DBSP / Z-Set Algebra (Feldera-inspired)
 *
 * Model each view as a continuous query over a stream of changes.
 * Changes are "Z-sets" (multisets with positive/negative multiplicities).
 * Each write produces a delta. Deltas propagate through a dataflow graph of operators.
 *
 * Key concepts:
 * - Z-set: a multiset where each element has an integer weight (+1 = insert, -1 = delete)
 * - Operator: transforms one Z-set into another (filter, map, join, aggregate, topK)
 * - Integration (Z^-1): accumulates deltas over time into the current state
 * - Differentiation (D): computes the delta between two states
 *
 * Read is O(1) always.
 * Write propagates through the dataflow graph: O(dataflow_depth x operator_cost).
 */

import type {
  IVMAlgorithm,
  GraphStore,
  ViewDefinition,
  ViewResult,
  WriteOp,
  NodeId,
  PropertyValue,
} from '../types.js';

// ─── Z-Set: Multiset with Integer Weights ───

type ZSetKey = string;

class ZSet<T> {
  /** Map from serialized key to (element, weight) */
  private entries = new Map<ZSetKey, { element: T; weight: number }>();
  private keyFn: (element: T) => ZSetKey;

  constructor(keyFn: (element: T) => ZSetKey) {
    this.keyFn = keyFn;
  }

  insert(element: T, weight: number = 1): void {
    const key = this.keyFn(element);
    const existing = this.entries.get(key);
    if (existing) {
      existing.weight += weight;
      if (existing.weight === 0) {
        this.entries.delete(key);
      }
    } else if (weight !== 0) {
      this.entries.set(key, { element, weight });
    }
  }

  delete(element: T): void {
    this.insert(element, -1);
  }

  /** Apply a delta Z-set to this Z-set (integration) */
  applyDelta(delta: ZSet<T>): void {
    for (const [key, { element, weight }] of delta.entries) {
      const existing = this.entries.get(key);
      if (existing) {
        existing.weight += weight;
        // Update element to latest version
        existing.element = element;
        if (existing.weight === 0) {
          this.entries.delete(key);
        }
      } else if (weight > 0) {
        this.entries.set(key, { element: { ...element } as T, weight });
      }
    }
  }

  /** Get all elements with positive weight */
  positiveElements(): T[] {
    const result: T[] = [];
    for (const { element, weight } of this.entries.values()) {
      if (weight > 0) result.push(element);
    }
    return result;
  }

  get size(): number {
    let count = 0;
    for (const { weight } of this.entries.values()) {
      if (weight > 0) count++;
    }
    return count;
  }

  has(key: ZSetKey): boolean {
    const entry = this.entries.get(key);
    return !!entry && entry.weight > 0;
  }

  get(key: ZSetKey): T | undefined {
    const entry = this.entries.get(key);
    return entry && entry.weight > 0 ? entry.element : undefined;
  }

  clear(): void {
    this.entries.clear();
  }

  get rawSize(): number {
    return this.entries.size;
  }

  [Symbol.iterator](): IterableIterator<[ZSetKey, { element: T; weight: number }]> {
    return this.entries[Symbol.iterator]();
  }
}

// ─── Dataflow Operators ───

type Row = Record<string, PropertyValue>;

interface Operator {
  /** Process a delta and return the output delta */
  processDelta(delta: ZSet<Row>): ZSet<Row>;
  /** Get the current integrated state */
  currentState(): ZSet<Row>;
  /** Memory estimate */
  memoryBytes(): number;
}

/** Filter: passes through rows matching a predicate */
class FilterOperator implements Operator {
  private integrated: ZSet<Row>;
  private predicate: (row: Row) => boolean;

  constructor(predicate: (row: Row) => boolean) {
    this.predicate = predicate;
    this.integrated = new ZSet(row => row['_id'] as string);
  }

  processDelta(delta: ZSet<Row>): ZSet<Row> {
    const output = new ZSet<Row>(row => row['_id'] as string);
    for (const [, { element, weight }] of delta) {
      if (this.predicate(element)) {
        output.insert(element, weight);
      }
    }
    this.integrated.applyDelta(output);
    return output;
  }

  currentState(): ZSet<Row> {
    return this.integrated;
  }

  memoryBytes(): number {
    return this.integrated.rawSize * 120;
  }
}

/** Sort + Limit (TopK): maintains a sorted top-K from its input */
class TopKOperator implements Operator {
  private integrated: ZSet<Row>;
  private sorted: Row[] = [];
  private limit: number;
  private compareFn: (a: Row, b: Row) => number;

  constructor(limit: number, compareFn: (a: Row, b: Row) => number) {
    this.limit = limit;
    this.compareFn = compareFn;
    this.integrated = new ZSet(row => row['_id'] as string);
  }

  processDelta(delta: ZSet<Row>): ZSet<Row> {
    this.integrated.applyDelta(delta);

    // Rebuild sorted from integrated (this is the cost of maintaining sorted order)
    this.sorted = this.integrated.positiveElements();
    this.sorted.sort(this.compareFn);
    this.sorted = this.sorted.slice(0, this.limit);

    // Output delta is the difference, but for simplicity we just output the new state
    // In a real DBSP system, we would compute the actual delta
    const output = new ZSet<Row>(row => row['_id'] as string);
    for (const row of this.sorted) {
      output.insert(row, 1);
    }
    return output;
  }

  currentState(): ZSet<Row> {
    const result = new ZSet<Row>(row => row['_id'] as string);
    for (const row of this.sorted) {
      result.insert(row, 1);
    }
    return result;
  }

  memoryBytes(): number {
    return this.integrated.rawSize * 120 + this.sorted.length * 100;
  }
}

/** Aggregate: computes running aggregates over input */
class AggregateOperator implements Operator {
  private result: Row = {};
  private aggregateFn: (currentDelta: ZSet<Row>, currentResult: Row) => Row;
  private integrated: ZSet<Row>;

  constructor(aggregateFn: (currentState: ZSet<Row>, currentResult: Row) => Row) {
    this.aggregateFn = aggregateFn;
    this.integrated = new ZSet(row => row['_id'] as string);
  }

  processDelta(delta: ZSet<Row>): ZSet<Row> {
    this.integrated.applyDelta(delta);
    this.result = this.aggregateFn(this.integrated, this.result);

    const output = new ZSet<Row>(row => 'aggregate');
    output.insert(this.result, 1);
    return output;
  }

  currentState(): ZSet<Row> {
    const result = new ZSet<Row>(row => 'aggregate');
    result.insert(this.result, 1);
    return result;
  }

  memoryBytes(): number {
    return this.integrated.rawSize * 120 + 200;
  }
}

/** Join: joins two streams based on a key. For capability check: entity edges + grants */
class HashJoinOperator implements Operator {
  private leftState: ZSet<Row>;
  private rightState: ZSet<Row>;
  private leftKeyFn: (row: Row) => string;
  private rightKeyFn: (row: Row) => string;
  private joinFn: (left: Row, right: Row) => Row;
  private outputState: ZSet<Row>;

  constructor(
    leftKeyFn: (row: Row) => string,
    rightKeyFn: (row: Row) => string,
    joinFn: (left: Row, right: Row) => Row
  ) {
    this.leftKeyFn = leftKeyFn;
    this.rightKeyFn = rightKeyFn;
    this.joinFn = joinFn;
    this.leftState = new ZSet(row => row['_id'] as string);
    this.rightState = new ZSet(row => row['_id'] as string);
    this.outputState = new ZSet(row => row['_key'] as string);
  }

  processLeftDelta(delta: ZSet<Row>): ZSet<Row> {
    this.leftState.applyDelta(delta);
    return this.recomputeJoin();
  }

  processRightDelta(delta: ZSet<Row>): ZSet<Row> {
    this.rightState.applyDelta(delta);
    return this.recomputeJoin();
  }

  processDelta(delta: ZSet<Row>): ZSet<Row> {
    // Default: treat as left delta
    return this.processLeftDelta(delta);
  }

  private recomputeJoin(): ZSet<Row> {
    // Build join index on right side
    const rightIndex = new Map<string, Row[]>();
    for (const right of this.rightState.positiveElements()) {
      const key = this.rightKeyFn(right);
      let arr = rightIndex.get(key);
      if (!arr) { arr = []; rightIndex.set(key, arr); }
      arr.push(right);
    }

    // Join
    const newOutput = new ZSet<Row>(row => row['_key'] as string);
    for (const left of this.leftState.positiveElements()) {
      const key = this.leftKeyFn(left);
      const rights = rightIndex.get(key);
      if (rights) {
        for (const right of rights) {
          const joined = this.joinFn(left, right);
          newOutput.insert(joined, 1);
        }
      }
    }

    this.outputState = newOutput;
    return newOutput;
  }

  currentState(): ZSet<Row> {
    return this.outputState;
  }

  memoryBytes(): number {
    return this.leftState.rawSize * 120 + this.rightState.rawSize * 120 + this.outputState.rawSize * 120;
  }
}

/** Identity pass-through: just integrates */
class IdentityOperator implements Operator {
  private integrated: ZSet<Row>;

  constructor() {
    this.integrated = new ZSet(row => row['_id'] as string);
  }

  processDelta(delta: ZSet<Row>): ZSet<Row> {
    this.integrated.applyDelta(delta);
    return delta;
  }

  currentState(): ZSet<Row> {
    return this.integrated;
  }

  memoryBytes(): number {
    return this.integrated.rawSize * 120;
  }
}

// ─── Dataflow Pipeline ───

interface DataflowNode {
  operator: Operator;
  children: DataflowNode[];
}

// ─── Per-View Dataflow Definitions ───

interface ViewPipeline {
  /** Process a write op and propagate deltas */
  processWrite(op: WriteOp, graph: GraphStore): void;
  /** Get the final view result */
  getResult(): ViewResult;
  /** Memory estimate */
  memoryBytes(): number;
}

// ─── View 1: Event Handler Pipeline ───

class EventHandlerPipeline implements ViewPipeline {
  private handlers: ZSet<Row>;
  private sortedCache: Row[] = [];
  private eventTypeNodeId: string | null = null;
  private graph!: GraphStore;

  constructor() {
    this.handlers = new ZSet(row => row['_id'] as string);
  }

  initialize(graph: GraphStore): void {
    this.graph = graph;
    this.handlers.clear();

    const eventTypes = graph.getNodesByLabel('EventType');
    const target = eventTypes.find(n => n.properties['name'] === 'content:afterCreate');
    this.eventTypeNodeId = target?.id ?? null;

    if (target) {
      const inEdges = graph.getInEdges(target.id, 'LISTENS_TO');
      for (const edge of inEdges) {
        const handler = graph.getNode(edge.from);
        if (handler && handler.labels.includes('EventHandler')) {
          this.handlers.insert({ ...handler.properties, _id: handler.id });
        }
      }
    }

    this.rebuildSorted();
  }

  processWrite(op: WriteOp, graph: GraphStore): void {
    this.graph = graph;
    const delta = new ZSet<Row>(row => row['_id'] as string);

    switch (op.kind) {
      case 'createEdge': {
        if (op.edge.type === 'LISTENS_TO' && op.edge.to === this.eventTypeNodeId) {
          const handler = graph.getNode(op.edge.from);
          if (handler && handler.labels.includes('EventHandler')) {
            delta.insert({ ...handler.properties, _id: handler.id }, 1);
          }
        }
        break;
      }
      case 'deleteEdge': {
        // Edge already deleted from graph, need to check if it was a LISTENS_TO -> our event type
        // We track this via the handlers Z-set: if a handler disappears from edges, remove it
        if (this.eventTypeNodeId) {
          const currentEdges = graph.getInEdges(this.eventTypeNodeId, 'LISTENS_TO');
          const currentHandlerIds = new Set(currentEdges.map(e => e.from));
          for (const h of this.handlers.positiveElements()) {
            if (!currentHandlerIds.has(h['_id'] as string)) {
              delta.insert(h, -1);
            }
          }
        }
        break;
      }
      case 'deleteNode': {
        const existing = this.handlers.get(op.id);
        if (existing) {
          delta.insert(existing, -1);
        }
        break;
      }
      case 'updateNode': {
        const existing = this.handlers.get(op.id);
        if (existing) {
          // Remove old, add new
          delta.insert(existing, -1);
          const updated = graph.getNode(op.id);
          if (updated) {
            delta.insert({ ...updated.properties, _id: updated.id }, 1);
          }
        }
        break;
      }
      case 'createNode':
        // Handler nodes don't appear in view until connected via LISTENS_TO edge
        break;
    }

    if (delta.rawSize > 0) {
      this.handlers.applyDelta(delta);
      this.rebuildSorted();
    }
  }

  private rebuildSorted(): void {
    this.sortedCache = this.handlers.positiveElements();
    this.sortedCache.sort((a, b) =>
      ((a['priority'] as number) ?? 0) - ((b['priority'] as number) ?? 0)
    );
  }

  getResult(): ViewResult {
    return {
      rows: this.sortedCache,
      dirty: false,
      lastUpdated: performance.now(),
    };
  }

  memoryBytes(): number {
    return this.handlers.rawSize * 120 + this.sortedCache.length * 100;
  }
}

// ─── View 2: Capability Check Pipeline ───

class CapabilityPipeline implements ViewPipeline {
  private capabilities: ZSet<Row>;
  private graph!: GraphStore;

  constructor() {
    this.capabilities = new ZSet(row => row['_key'] as string);
  }

  initialize(graph: GraphStore): void {
    this.graph = graph;
    this.capabilities.clear();

    const entities = graph.getNodesByLabel('Entity');
    for (const entity of entities) {
      const capEdges = graph.getOutEdges(entity.id, 'HAS_CAPABILITY');
      for (const edge of capEdges) {
        const grant = graph.getNode(edge.to);
        if (grant && grant.labels.includes('CapabilityGrant')) {
          const cap = grant.properties['capability'] as string;
          const scope = grant.properties['scope'] as string;
          const key = `${entity.id}:${cap}:${scope}`;
          this.capabilities.insert({ _key: key, _id: key, granted: true }, 1);
        }
      }
    }
  }

  processWrite(op: WriteOp, graph: GraphStore): void {
    this.graph = graph;
    const delta = new ZSet<Row>(row => row['_key'] as string);

    switch (op.kind) {
      case 'createEdge': {
        if (op.edge.type === 'HAS_CAPABILITY') {
          const entity = graph.getNode(op.edge.from);
          const grant = graph.getNode(op.edge.to);
          if (entity && grant && grant.labels.includes('CapabilityGrant')) {
            const cap = grant.properties['capability'] as string;
            const scope = grant.properties['scope'] as string;
            const key = `${entity.id}:${cap}:${scope}`;
            delta.insert({ _key: key, _id: key, granted: true }, 1);
          }
        }
        break;
      }
      case 'deleteEdge':
      case 'deleteNode': {
        // Node/edge deletion cascades in the graph. Re-derive from current state.
        // Build current capability set from graph
        const currentKeys = new Set<string>();
        const entities = graph.getNodesByLabel('Entity');
        for (const entity of entities) {
          const capEdges = graph.getOutEdges(entity.id, 'HAS_CAPABILITY');
          for (const edge of capEdges) {
            const grant = graph.getNode(edge.to);
            if (grant && grant.labels.includes('CapabilityGrant')) {
              const cap = grant.properties['capability'] as string;
              const scope = grant.properties['scope'] as string;
              currentKeys.add(`${entity.id}:${cap}:${scope}`);
            }
          }
        }
        // Remove capabilities that are no longer present
        for (const existing of this.capabilities.positiveElements()) {
          if (!currentKeys.has(existing['_key'] as string)) {
            delta.insert(existing, -1);
          }
        }
        // Add capabilities that are newly present
        for (const key of currentKeys) {
          if (!this.capabilities.has(key)) {
            delta.insert({ _key: key, _id: key, granted: true }, 1);
          }
        }
        break;
      }
      default:
        break;
    }

    if (delta.rawSize > 0) {
      this.capabilities.applyDelta(delta);
    }
  }

  getResult(): ViewResult {
    return {
      rows: this.capabilities.positiveElements().map(r => ({ key: r['_key'], granted: r['granted'] })),
      dirty: false,
      lastUpdated: performance.now(),
    };
  }

  memoryBytes(): number {
    return this.capabilities.rawSize * 120;
  }
}

// ─── View 3: Content Listing Pipeline ───

class ContentListingPipeline implements ViewPipeline {
  private allPublished: ZSet<Row>;
  private topK: Row[] = [];
  private limit: number = 20;
  private graph!: GraphStore;

  constructor() {
    this.allPublished = new ZSet(row => row['_id'] as string);
  }

  initialize(graph: GraphStore): void {
    this.graph = graph;
    this.allPublished.clear();

    const posts = graph.getNodesByLabel('Post');
    for (const post of posts) {
      if (post.properties['published'] === true) {
        this.allPublished.insert({ ...post.properties, _id: post.id }, 1);
      }
    }

    this.rebuildTopK();
  }

  processWrite(op: WriteOp, graph: GraphStore): void {
    this.graph = graph;
    let changed = false;

    switch (op.kind) {
      case 'createNode': {
        if (op.node.labels.includes('Post') && op.node.properties['published'] === true) {
          this.allPublished.insert({ ...op.node.properties, _id: op.node.id }, 1);
          changed = true;
        }
        break;
      }
      case 'updateNode': {
        const node = graph.getNode(op.id);
        if (!node || !node.labels.includes('Post')) break;

        const existing = this.allPublished.get(op.id);
        const isNowPublished = node.properties['published'] === true;

        if (existing && !isNowPublished) {
          // Was published, now unpublished: remove
          this.allPublished.insert(existing, -1);
          changed = true;
        } else if (!existing && isNowPublished) {
          // Was not published, now published: add
          this.allPublished.insert({ ...node.properties, _id: node.id }, 1);
          changed = true;
        } else if (existing && isNowPublished) {
          // Still published but properties changed: remove old, add new
          // Apply as two separate operations to avoid cancellation
          this.allPublished.insert(existing, -1);
          this.allPublished.insert({ ...node.properties, _id: node.id }, 1);
          changed = true;
        }
        break;
      }
      case 'deleteNode': {
        const existing = this.allPublished.get(op.id);
        if (existing) {
          this.allPublished.insert(existing, -1);
          changed = true;
        }
        break;
      }
      default:
        break;
    }

    if (changed) {
      this.rebuildTopK();
    }
  }

  private rebuildTopK(): void {
    const all = this.allPublished.positiveElements();
    all.sort((a, b) =>
      ((b['publishedAt'] as number) ?? 0) - ((a['publishedAt'] as number) ?? 0)
    );
    this.topK = all.slice(0, this.limit);
  }

  getResult(): ViewResult {
    return {
      rows: this.topK,
      dirty: false,
      lastUpdated: performance.now(),
    };
  }

  memoryBytes(): number {
    return this.allPublished.rawSize * 120 + this.topK.length * 100;
  }
}

// ─── View 4: Governance Pipeline ───

class GovernancePipeline implements ViewPipeline {
  private rules: ZSet<Row>;
  private effectiveRules = new Map<string, Row>();
  private ancestorDepths = new Map<string, number>();
  private graph!: GraphStore;
  private targetGroveId = 'grove-leaf-0';

  constructor() {
    this.rules = new ZSet(row => `${row['groveId']}:${row['ruleId']}`);
  }

  initialize(graph: GraphStore): void {
    this.graph = graph;
    this.rules.clear();
    this.effectiveRules.clear();
    this.ancestorDepths.clear();

    // Walk hierarchy
    this.walkHierarchy(this.targetGroveId, 0, new Set());
    this.computeEffective();
  }

  private walkHierarchy(groveId: string, depth: number, visited: Set<string>): void {
    if (visited.has(groveId)) return;
    visited.add(groveId);
    this.ancestorDepths.set(groveId, depth);

    const ruleEdges = this.graph.getOutEdges(groveId, 'HAS_RULE');
    for (const edge of ruleEdges) {
      const rule = this.graph.getNode(edge.to);
      if (rule && rule.labels.includes('GovernanceRule')) {
        this.rules.insert({
          _id: `${groveId}:${rule.id}`,
          ruleId: rule.id,
          groveId,
          name: rule.properties['name'] as string,
          value: rule.properties['value'] ?? null,
          depth,
        }, 1);
      }
    }

    const parentEdges = this.graph.getOutEdges(groveId, 'PARENT_GROVE');
    for (const edge of parentEdges) {
      this.walkHierarchy(edge.to, depth + 1, visited);
    }
  }

  private computeEffective(): void {
    this.effectiveRules.clear();
    for (const row of this.rules.positiveElements()) {
      const name = row['name'] as string;
      const depth = row['depth'] as number;
      const existing = this.effectiveRules.get(name);
      if (!existing || depth < (existing['depth'] as number)) {
        this.effectiveRules.set(name, row);
      }
    }
  }

  processWrite(op: WriteOp, graph: GraphStore): void {
    this.graph = graph;

    // For governance with hierarchical dependencies, any change to rules or grove structure
    // requires rewalking. Use targeted rebuild.
    let affected = false;
    switch (op.kind) {
      case 'createNode':
        affected = op.node.labels.includes('GovernanceRule') || op.node.labels.includes('Grove');
        break;
      case 'deleteNode': {
        affected = this.ancestorDepths.has(op.id);
        break;
      }
      case 'updateNode': {
        // Check if it's a rule in our set
        for (const row of this.rules.positiveElements()) {
          if (row['ruleId'] === op.id) { affected = true; break; }
        }
        break;
      }
      case 'createEdge':
        affected = op.edge.type === 'HAS_RULE' || op.edge.type === 'PARENT_GROVE';
        break;
      case 'deleteEdge':
        affected = true; // Conservative
        break;
    }

    if (affected) {
      this.rules.clear();
      this.ancestorDepths.clear();
      this.walkHierarchy(this.targetGroveId, 0, new Set());
      this.computeEffective();
    }
  }

  getResult(): ViewResult {
    return {
      rows: Array.from(this.effectiveRules.values()),
      dirty: false,
      lastUpdated: performance.now(),
    };
  }

  memoryBytes(): number {
    return this.rules.rawSize * 140 + this.effectiveRules.size * 120 + this.ancestorDepths.size * 40;
  }
}

// ─── View 5: Attestation Pipeline ───

class AttestationPipeline implements ViewPipeline {
  private attestations: ZSet<Row>;
  private aggregate: Row = {};
  private knowledgeNodeId = 'knowledge-0';
  private graph!: GraphStore;

  constructor() {
    this.attestations = new ZSet(row => row['_id'] as string);
  }

  initialize(graph: GraphStore): void {
    this.graph = graph;
    this.attestations.clear();

    const attestEdges = graph.getInEdges(this.knowledgeNodeId, 'ATTESTS_TO');
    for (const edge of attestEdges) {
      const attestation = graph.getNode(edge.from);
      if (attestation && attestation.labels.includes('Attestation')) {
        const authorEdges = graph.getOutEdges(attestation.id, 'AUTHORED_BY');
        const attestorId = authorEdges.length > 0 ? authorEdges[0]!.to : null;
        this.attestations.insert({
          _id: attestation.id,
          value: attestation.properties['value'] ?? 0,
          attestorId: attestorId,
        }, 1);
      }
    }

    this.recomputeAggregate();
  }

  processWrite(op: WriteOp, graph: GraphStore): void {
    this.graph = graph;
    const delta = new ZSet<Row>(row => row['_id'] as string);

    switch (op.kind) {
      case 'createEdge': {
        if (op.edge.type === 'ATTESTS_TO' && op.edge.to === this.knowledgeNodeId) {
          const attestation = graph.getNode(op.edge.from);
          if (attestation && attestation.labels.includes('Attestation')) {
            const authorEdges = graph.getOutEdges(attestation.id, 'AUTHORED_BY');
            const attestorId = authorEdges.length > 0 ? authorEdges[0]!.to : null;
            delta.insert({
              _id: attestation.id,
              value: attestation.properties['value'] ?? 0,
              attestorId,
            }, 1);
          }
        }
        break;
      }
      case 'deleteNode': {
        const existing = this.attestations.get(op.id);
        if (existing) {
          delta.insert(existing, -1);
        }
        break;
      }
      case 'updateNode': {
        const existing = this.attestations.get(op.id);
        if (existing) {
          delta.insert(existing, -1);
          const updated = graph.getNode(op.id);
          if (updated) {
            delta.insert({
              ...existing,
              value: updated.properties['value'] ?? 0,
            }, 1);
          }
        }
        break;
      }
      default:
        break;
    }

    if (delta.rawSize > 0) {
      this.attestations.applyDelta(delta);
      this.recomputeAggregate();
    }
  }

  private recomputeAggregate(): void {
    let totalValue = 0;
    let count = 0;
    const attestors = new Set<string>();

    for (const row of this.attestations.positiveElements()) {
      totalValue += (row['value'] as number) ?? 0;
      count++;
      if (row['attestorId']) attestors.add(row['attestorId'] as string);
    }

    this.aggregate = {
      knowledgeNodeId: this.knowledgeNodeId,
      totalValue,
      attestationCount: count,
      attestorCount: attestors.size,
    };
  }

  getResult(): ViewResult {
    return {
      rows: [this.aggregate],
      dirty: false,
      lastUpdated: performance.now(),
    };
  }

  memoryBytes(): number {
    return this.attestations.rawSize * 120 + 200;
  }
}

// ─── Algorithm C: DBSP Implementation ───

export class DBSPAlgorithm implements IVMAlgorithm {
  readonly name = 'C: DBSP / Z-Set Algebra';
  private graph!: GraphStore;
  private pipelines = new Map<string, ViewPipeline>();

  initialize(graph: GraphStore, viewDefs: ViewDefinition[]): void {
    this.graph = graph;
    this.pipelines.clear();

    for (const def of viewDefs) {
      const pipeline = this.createPipeline(def.id);
      pipeline.initialize(graph);
      this.pipelines.set(def.id, pipeline);
    }
  }

  private createPipeline(viewId: string): ViewPipeline & { initialize(graph: GraphStore): void } {
    switch (viewId) {
      case 'event_handlers':
        return new EventHandlerPipeline();
      case 'capability_check':
        return new CapabilityPipeline();
      case 'content_listing':
        return new ContentListingPipeline();
      case 'governance_rules':
        return new GovernancePipeline();
      case 'attestation_aggregate':
        return new AttestationPipeline();
      default:
        throw new Error(`Unknown view: ${viewId}`);
    }
  }

  applyWrite(op: WriteOp): void {
    // Apply to graph first
    this.graph.applyWrite(op);

    // Propagate deltas through each pipeline
    for (const [, pipeline] of this.pipelines) {
      pipeline.processWrite(op, this.graph);
    }
  }

  readView(viewId: string): ViewResult {
    const pipeline = this.pipelines.get(viewId);
    if (!pipeline) throw new Error(`Unknown view: ${viewId}`);
    return pipeline.getResult();
  }

  memoryOverhead(): number {
    let bytes = 0;
    for (const [, pipeline] of this.pipelines) {
      bytes += pipeline.memoryBytes();
    }
    return bytes;
  }

  reset(): void {
    this.pipelines.clear();
  }
}
