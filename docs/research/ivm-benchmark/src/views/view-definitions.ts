/**
 * The 5 view patterns that the IVM system must maintain.
 * Each defines how to fully compute from scratch and what writes affect it.
 */

import type { GraphStore, Node, Edge, WriteOp, PropertyValue } from '../types.js';

// ─── View 1: Event Handler Resolution ───
// "All EventHandler Nodes connected to EventType 'content:afterCreate' via LISTENS_TO edges, sorted by priority."

export function computeEventHandlerView(graph: GraphStore, eventTypeName: string): Record<string, PropertyValue>[] {
  // Find the EventType node
  const eventTypes = graph.getNodesByLabel('EventType');
  const targetType = eventTypes.find(n => n.properties['name'] === eventTypeName);
  if (!targetType) return [];

  // Find all LISTENS_TO edges pointing to this EventType
  const inEdges = graph.getInEdges(targetType.id, 'LISTENS_TO');

  // Get the handler nodes and sort by priority
  const handlers: Record<string, PropertyValue>[] = [];
  for (const edge of inEdges) {
    const handler = graph.getNode(edge.from);
    if (handler && handler.labels.includes('EventHandler')) {
      handlers.push({ ...handler.properties, _id: handler.id });
    }
  }

  handlers.sort((a, b) => ((a['priority'] as number) ?? 0) - ((b['priority'] as number) ?? 0));
  return handlers;
}

export function eventHandlerAffected(op: WriteOp, graph: GraphStore): boolean {
  switch (op.kind) {
    case 'createNode':
      return op.node.labels.includes('EventHandler');
    case 'deleteNode': {
      const node = graph.getNode(op.id);
      return !!node && node.labels.includes('EventHandler');
    }
    case 'updateNode': {
      const node = graph.getNode(op.id);
      return !!node && (node.labels.includes('EventHandler') || node.labels.includes('EventType'));
    }
    case 'createEdge':
      return op.edge.type === 'LISTENS_TO';
    case 'deleteEdge': {
      const edge = graph.getEdge(op.id);
      return !!edge && edge.type === 'LISTENS_TO';
    }
  }
}

// ─── View 2: Capability Check ───
// "Does Entity X have Capability Y in Scope Z?"
// Stored as a map: `${entityId}:${capability}:${scope}` -> boolean

export function computeCapabilityView(graph: GraphStore): Map<string, boolean> {
  const result = new Map<string, boolean>();
  const grants = graph.getNodesByLabel('CapabilityGrant');
  for (const grant of grants) {
    const edges = graph.getInEdges(grant.id, 'GRANTED_TO');
    for (const edge of edges) {
      const entity = graph.getNode(edge.from);
      if (entity) {
        const cap = grant.properties['capability'] as string;
        const scope = grant.properties['scope'] as string;
        const key = `${entity.id}:${cap}:${scope}`;
        result.set(key, true);
      }
    }
    // Also check outgoing GRANTED_TO (entity -> grant)
    const outEdges = graph.getOutEdges(grant.id, 'GRANTED_TO');
    for (const edge of outEdges) {
      const entity = graph.getNode(edge.to);
      if (entity) {
        const cap = grant.properties['capability'] as string;
        const scope = grant.properties['scope'] as string;
        const key = `${entity.id}:${cap}:${scope}`;
        result.set(key, true);
      }
    }
  }
  // Also traverse from entities
  const entities = graph.getNodesByLabel('Entity');
  for (const entity of entities) {
    const outEdges = graph.getOutEdges(entity.id, 'HAS_CAPABILITY');
    for (const edge of outEdges) {
      const grant = graph.getNode(edge.to);
      if (grant && grant.labels.includes('CapabilityGrant')) {
        const cap = grant.properties['capability'] as string;
        const scope = grant.properties['scope'] as string;
        const key = `${entity.id}:${cap}:${scope}`;
        result.set(key, true);
      }
    }
  }
  return result;
}

export function capabilityCheckAffected(op: WriteOp, graph: GraphStore): boolean {
  switch (op.kind) {
    case 'createNode':
      return op.node.labels.includes('CapabilityGrant');
    case 'deleteNode': {
      const node = graph.getNode(op.id);
      return !!node && (node.labels.includes('CapabilityGrant') || node.labels.includes('Entity'));
    }
    case 'updateNode': {
      const node = graph.getNode(op.id);
      return !!node && node.labels.includes('CapabilityGrant');
    }
    case 'createEdge':
      return op.edge.type === 'HAS_CAPABILITY' || op.edge.type === 'GRANTED_TO';
    case 'deleteEdge': {
      const edge = graph.getEdge(op.id);
      return !!edge && (edge.type === 'HAS_CAPABILITY' || edge.type === 'GRANTED_TO');
    }
  }
}

// ─── View 3: Content Listing (Sorted + Paginated) ───
// "The 20 most recent published posts, sorted by date DESC"

export function computeContentListingView(graph: GraphStore, limit: number = 20): Record<string, PropertyValue>[] {
  const posts = graph.getNodesByLabel('Post');
  const published = posts.filter(p => p.properties['published'] === true);
  published.sort((a, b) => ((b.properties['publishedAt'] as number) ?? 0) - ((a.properties['publishedAt'] as number) ?? 0));
  return published.slice(0, limit).map(p => ({ ...p.properties, _id: p.id }));
}

export function contentListingAffected(op: WriteOp, graph: GraphStore): boolean {
  switch (op.kind) {
    case 'createNode':
      return op.node.labels.includes('Post');
    case 'deleteNode': {
      const node = graph.getNode(op.id);
      return !!node && node.labels.includes('Post');
    }
    case 'updateNode': {
      const node = graph.getNode(op.id);
      return !!node && node.labels.includes('Post');
    }
    default:
      return false;
  }
}

// ─── View 4: Governance Rule Resolution ───
// "Effective rules for Sub-Grove X, considering parent Grove hierarchy with overrides"

export interface GovernanceRule {
  ruleId: string;
  groveId: string;
  name: string;
  value: PropertyValue;
  depth: number; // distance from target grove
}

export function computeGovernanceView(graph: GraphStore, targetGroveId: string): Record<string, PropertyValue>[] {
  // Walk up the grove hierarchy collecting rules
  // Closer (lower depth) overrides further (higher depth)
  const ruleMap = new Map<string, GovernanceRule>();

  function collectRules(groveId: string, depth: number, visited: Set<string>): void {
    if (visited.has(groveId)) return;
    visited.add(groveId);

    const grove = graph.getNode(groveId);
    if (!grove) return;

    // Get rules attached to this grove
    const ruleEdges = graph.getOutEdges(groveId, 'HAS_RULE');
    for (const edge of ruleEdges) {
      const rule = graph.getNode(edge.to);
      if (rule && rule.labels.includes('GovernanceRule')) {
        const name = rule.properties['name'] as string;
        // Only keep the closest (lowest depth) rule
        const existing = ruleMap.get(name);
        if (!existing || depth < existing.depth) {
          ruleMap.set(name, {
            ruleId: rule.id,
            groveId,
            name,
            value: rule.properties['value'] ?? null,
            depth,
          });
        }
      }
    }

    // Walk to parent grove
    const parentEdges = graph.getOutEdges(groveId, 'PARENT_GROVE');
    for (const edge of parentEdges) {
      collectRules(edge.to, depth + 1, visited);
    }
  }

  collectRules(targetGroveId, 0, new Set());

  return Array.from(ruleMap.values()).map(r => ({
    ruleId: r.ruleId,
    groveId: r.groveId,
    name: r.name,
    value: r.value,
    depth: r.depth,
  }));
}

export function governanceAffected(op: WriteOp, graph: GraphStore): boolean {
  switch (op.kind) {
    case 'createNode':
      return op.node.labels.includes('GovernanceRule') || op.node.labels.includes('Grove');
    case 'deleteNode': {
      const node = graph.getNode(op.id);
      return !!node && (node.labels.includes('GovernanceRule') || node.labels.includes('Grove'));
    }
    case 'updateNode': {
      const node = graph.getNode(op.id);
      return !!node && node.labels.includes('GovernanceRule');
    }
    case 'createEdge':
      return op.edge.type === 'HAS_RULE' || op.edge.type === 'PARENT_GROVE';
    case 'deleteEdge': {
      const edge = graph.getEdge(op.id);
      return !!edge && (edge.type === 'HAS_RULE' || edge.type === 'PARENT_GROVE');
    }
  }
}

// ─── View 5: Knowledge Attestation Aggregate ───
// "Total attestation value of Knowledge Node Y, with attestor count"

export function computeAttestationView(graph: GraphStore, knowledgeNodeId: string): Record<string, PropertyValue> {
  const attestEdges = graph.getInEdges(knowledgeNodeId, 'ATTESTS_TO');
  let totalValue = 0;
  let count = 0;
  const attestors = new Set<string>();

  for (const edge of attestEdges) {
    const attestation = graph.getNode(edge.from);
    if (attestation && attestation.labels.includes('Attestation')) {
      totalValue += (attestation.properties['value'] as number) ?? 0;
      count++;
      const attestorEdges = graph.getInEdges(attestation.id, 'AUTHORED_BY');
      for (const ae of attestorEdges) {
        attestors.add(ae.from);
      }
      // Also check outgoing AUTHORED_BY
      const outAttestorEdges = graph.getOutEdges(attestation.id, 'AUTHORED_BY');
      for (const ae of outAttestorEdges) {
        attestors.add(ae.to);
      }
    }
  }

  return {
    knowledgeNodeId,
    totalValue,
    attestationCount: count,
    attestorCount: attestors.size,
  };
}

export function attestationAffected(op: WriteOp, graph: GraphStore): boolean {
  switch (op.kind) {
    case 'createNode':
      return op.node.labels.includes('Attestation');
    case 'deleteNode': {
      const node = graph.getNode(op.id);
      return !!node && node.labels.includes('Attestation');
    }
    case 'updateNode': {
      const node = graph.getNode(op.id);
      return !!node && node.labels.includes('Attestation');
    }
    case 'createEdge':
      return op.edge.type === 'ATTESTS_TO' || op.edge.type === 'AUTHORED_BY';
    case 'deleteEdge': {
      const edge = graph.getEdge(op.id);
      return !!edge && (edge.type === 'ATTESTS_TO' || edge.type === 'AUTHORED_BY');
    }
  }
}
