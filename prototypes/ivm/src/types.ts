/**
 * Core types for IVM prototype.
 * Simplified graph model that mirrors Benten engine concepts.
 */

// ─── Graph Primitives ───

export type NodeId = string;
export type EdgeId = string;

export interface Node {
  id: NodeId;
  labels: string[];
  properties: Record<string, PropertyValue>;
}

export type PropertyValue = string | number | boolean | null;

export interface Edge {
  id: EdgeId;
  type: string;
  from: NodeId;
  to: NodeId;
  properties: Record<string, PropertyValue>;
}

// ─── Write Operations ───

export type WriteOp =
  | { kind: 'createNode'; node: Node }
  | { kind: 'updateNode'; id: NodeId; properties: Record<string, PropertyValue> }
  | { kind: 'deleteNode'; id: NodeId }
  | { kind: 'createEdge'; edge: Edge }
  | { kind: 'deleteEdge'; id: EdgeId };

// ─── View Types ───

export interface ViewResult {
  /** The materialized result data */
  rows: ReadonlyArray<Record<string, PropertyValue>>;
  /** Whether the view is stale (only used by Algorithm A) */
  dirty: boolean;
  /** Timestamp of last update */
  lastUpdated: number;
}

// ─── View Definition ───

export interface ViewDefinition {
  id: string;
  description: string;
}

// ─── Algorithm Interface ───

export interface IVMAlgorithm {
  readonly name: string;

  /**
   * Initialize the algorithm with the graph data and view definitions.
   * Must compute initial view results.
   */
  initialize(graph: GraphStore, views: ViewDefinition[]): void;

  /**
   * Apply a write operation and update affected views.
   * Returns the time taken for the write + IVM maintenance.
   */
  applyWrite(op: WriteOp): void;

  /**
   * Read a materialized view result.
   */
  readView(viewId: string): ViewResult;

  /**
   * Get memory overhead estimate in bytes.
   */
  memoryOverhead(): number;

  /**
   * Reset all state.
   */
  reset(): void;
}

// ─── Graph Store (shared backing store) ───

export class GraphStore {
  private nodes = new Map<NodeId, Node>();
  private edges = new Map<EdgeId, Edge>();
  /** Index: nodeId -> edges where node is `from` */
  private outEdges = new Map<NodeId, Set<EdgeId>>();
  /** Index: nodeId -> edges where node is `to` */
  private inEdges = new Map<NodeId, Set<EdgeId>>();
  /** Index: label -> nodeIds */
  private labelIndex = new Map<string, Set<NodeId>>();

  addNode(node: Node): void {
    this.nodes.set(node.id, { ...node, properties: { ...node.properties } });
    for (const label of node.labels) {
      let set = this.labelIndex.get(label);
      if (!set) {
        set = new Set();
        this.labelIndex.set(label, set);
      }
      set.add(node.id);
    }
  }

  getNode(id: NodeId): Node | undefined {
    return this.nodes.get(id);
  }

  updateNodeProperties(id: NodeId, properties: Record<string, PropertyValue>): void {
    const node = this.nodes.get(id);
    if (!node) return;
    Object.assign(node.properties, properties);
  }

  deleteNode(id: NodeId): Node | undefined {
    const node = this.nodes.get(id);
    if (!node) return undefined;
    this.nodes.delete(id);
    for (const label of node.labels) {
      this.labelIndex.get(label)?.delete(id);
    }
    // Clean up edges
    const out = this.outEdges.get(id);
    if (out) {
      for (const eid of out) this.edges.delete(eid);
      this.outEdges.delete(id);
    }
    const inc = this.inEdges.get(id);
    if (inc) {
      for (const eid of inc) this.edges.delete(eid);
      this.inEdges.delete(id);
    }
    return node;
  }

  addEdge(edge: Edge): void {
    this.edges.set(edge.id, { ...edge, properties: { ...edge.properties } });
    let outSet = this.outEdges.get(edge.from);
    if (!outSet) {
      outSet = new Set();
      this.outEdges.set(edge.from, outSet);
    }
    outSet.add(edge.id);
    let inSet = this.inEdges.get(edge.to);
    if (!inSet) {
      inSet = new Set();
      this.inEdges.set(edge.to, inSet);
    }
    inSet.add(edge.id);
  }

  getEdge(id: EdgeId): Edge | undefined {
    return this.edges.get(id);
  }

  deleteEdge(id: EdgeId): Edge | undefined {
    const edge = this.edges.get(id);
    if (!edge) return undefined;
    this.edges.delete(id);
    this.outEdges.get(edge.from)?.delete(id);
    this.inEdges.get(edge.to)?.delete(id);
    return edge;
  }

  getNodesByLabel(label: string): Node[] {
    const ids = this.labelIndex.get(label);
    if (!ids) return [];
    const result: Node[] = [];
    for (const id of ids) {
      const node = this.nodes.get(id);
      if (node) result.push(node);
    }
    return result;
  }

  getOutEdges(nodeId: NodeId, type?: string): Edge[] {
    const edgeIds = this.outEdges.get(nodeId);
    if (!edgeIds) return [];
    const result: Edge[] = [];
    for (const eid of edgeIds) {
      const edge = this.edges.get(eid);
      if (edge && (!type || edge.type === type)) result.push(edge);
    }
    return result;
  }

  getInEdges(nodeId: NodeId, type?: string): Edge[] {
    const edgeIds = this.inEdges.get(nodeId);
    if (!edgeIds) return [];
    const result: Edge[] = [];
    for (const eid of edgeIds) {
      const edge = this.edges.get(eid);
      if (edge && (!type || edge.type === type)) result.push(edge);
    }
    return result;
  }

  get nodeCount(): number {
    return this.nodes.size;
  }

  get edgeCount(): number {
    return this.edges.size;
  }

  getAllNodes(): Node[] {
    return Array.from(this.nodes.values());
  }

  getAllEdges(): Edge[] {
    return Array.from(this.edges.values());
  }

  applyWrite(op: WriteOp): void {
    switch (op.kind) {
      case 'createNode':
        this.addNode(op.node);
        break;
      case 'updateNode':
        this.updateNodeProperties(op.id, op.properties);
        break;
      case 'deleteNode':
        this.deleteNode(op.id);
        break;
      case 'createEdge':
        this.addEdge(op.edge);
        break;
      case 'deleteEdge':
        this.deleteEdge(op.id);
        break;
    }
  }
}

// ─── Benchmark Types ───

export interface LatencyStats {
  p50: number;
  p95: number;
  mean: number;
  min: number;
  max: number;
  count: number;
}

export interface BenchmarkResult {
  algorithm: string;
  viewId: string;
  readLatency: LatencyStats;
  writeLatency: LatencyStats;
  memoryOverheadBytes: number;
  correctnessVerified: boolean;
}
