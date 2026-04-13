/**
 * Data Generator: Creates realistic datasets for benchmarking.
 *
 * Generates:
 * - View 1: 100 EventTypes, 500 EventHandlers
 * - View 2: 1000 Entities, 5000 Capability grants
 * - View 3: 50,000 Content Nodes (Posts)
 * - View 4: 10 Groves with 3-level hierarchy
 * - View 5: 10,000 Knowledge Nodes with 100,000 attestations
 */

import { GraphStore } from '../types.js';
import type { Node, Edge, WriteOp } from '../types.js';

let edgeCounter = 0;

function nextEdgeId(): string {
  return `edge-${edgeCounter++}`;
}

// Deterministic pseudo-random number generator (Mulberry32)
function mulberry32(seed: number): () => number {
  return function() {
    let t = (seed += 0x6D2B79F5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export interface GeneratedData {
  graph: GraphStore;
  metadata: {
    eventTypeIds: string[];
    eventHandlerIds: string[];
    entityIds: string[];
    capabilityGrantIds: string[];
    postIds: string[];
    groveIds: string[];
    leafGroveIds: string[];
    governanceRuleIds: string[];
    knowledgeNodeIds: string[];
    attestationIds: string[];
    attestorIds: string[];
  };
}

export function generateBenchmarkData(): GeneratedData {
  const graph = new GraphStore();
  const random = mulberry32(42);
  edgeCounter = 0;

  // ─── View 1: Event Handlers ───

  const eventTypeIds: string[] = [];
  const eventTypeNames = [
    'content:afterCreate', 'content:afterUpdate', 'content:afterDelete',
    'content:beforeCreate', 'content:beforeUpdate', 'content:beforeDelete',
    'auth:login', 'auth:logout', 'auth:signup',
    'composition:afterCreate', 'composition:afterUpdate',
  ];

  for (let i = 0; i < 100; i++) {
    const id = `eventtype-${i}`;
    eventTypeIds.push(id);
    graph.addNode({
      id,
      labels: ['EventType'],
      properties: {
        name: i < eventTypeNames.length ? eventTypeNames[i]! : `custom:event${i}`,
      },
    });
  }

  const eventHandlerIds: string[] = [];
  for (let i = 0; i < 500; i++) {
    const id = `handler-${i}`;
    eventHandlerIds.push(id);
    graph.addNode({
      id,
      labels: ['EventHandler'],
      properties: {
        name: `handler-${i}`,
        priority: Math.floor(random() * 1000),
        module: `module-${Math.floor(random() * 20)}`,
      },
    });

    // Each handler listens to 1-3 event types
    const numEvents = 1 + Math.floor(random() * 3);
    const usedTypes = new Set<number>();
    for (let j = 0; j < numEvents; j++) {
      let typeIdx = Math.floor(random() * eventTypeIds.length);
      while (usedTypes.has(typeIdx)) typeIdx = (typeIdx + 1) % eventTypeIds.length;
      usedTypes.add(typeIdx);
      graph.addEdge({
        id: nextEdgeId(),
        type: 'LISTENS_TO',
        from: id,
        to: eventTypeIds[typeIdx]!,
        properties: {},
      });
    }
  }

  // ─── View 2: Capabilities ───

  const entityIds: string[] = [];
  for (let i = 0; i < 1000; i++) {
    const id = `entity-${i}`;
    entityIds.push(id);
    graph.addNode({
      id,
      labels: ['Entity'],
      properties: {
        name: `user-${i}`,
        type: random() > 0.5 ? 'user' : 'module',
      },
    });
  }

  const capabilityGrantIds: string[] = [];
  const capabilities = ['store:read', 'store:write', 'store:delete', 'compose:edit', 'compose:publish', 'admin:access', 'media:upload', 'media:delete'];
  const scopes = ['content/*', 'composition/*', 'media/*', 'admin/*', 'global', 'user/*'];

  for (let i = 0; i < 5000; i++) {
    const id = `capgrant-${i}`;
    capabilityGrantIds.push(id);
    const cap = capabilities[Math.floor(random() * capabilities.length)]!;
    const scope = scopes[Math.floor(random() * scopes.length)]!;
    graph.addNode({
      id,
      labels: ['CapabilityGrant'],
      properties: { capability: cap, scope },
    });

    // Grant to a random entity
    const entityIdx = Math.floor(random() * entityIds.length);
    graph.addEdge({
      id: nextEdgeId(),
      type: 'HAS_CAPABILITY',
      from: entityIds[entityIdx]!,
      to: id,
      properties: {},
    });
  }

  // ─── View 3: Content (Posts) ───

  const postIds: string[] = [];
  const baseTime = Date.now();
  for (let i = 0; i < 50000; i++) {
    const id = `post-${i}`;
    postIds.push(id);
    const published = random() > 0.3; // 70% published
    graph.addNode({
      id,
      labels: ['Post'],
      properties: {
        title: `Post Title ${i}`,
        published,
        publishedAt: published ? baseTime - Math.floor(random() * 86400000 * 365) : 0,
        author: `user-${Math.floor(random() * 100)}`,
      },
    });
  }

  // ─── View 4: Governance (Groves with 3-level hierarchy) ───

  const groveIds: string[] = [];
  const leafGroveIds: string[] = [];
  const governanceRuleIds: string[] = [];

  // Level 0: 2 root groves
  for (let i = 0; i < 2; i++) {
    const id = `grove-root-${i}`;
    groveIds.push(id);
    graph.addNode({
      id,
      labels: ['Grove'],
      properties: { name: `Root Grove ${i}`, level: 0 },
    });
  }

  // Level 1: 3-4 child groves per root (8 total)
  const level1Ids: string[] = [];
  for (let i = 0; i < 2; i++) {
    const numChildren = 3 + Math.floor(random() * 2);
    for (let j = 0; j < numChildren; j++) {
      const id = `grove-mid-${i}-${j}`;
      level1Ids.push(id);
      groveIds.push(id);
      graph.addNode({
        id,
        labels: ['Grove'],
        properties: { name: `Mid Grove ${i}-${j}`, level: 1 },
      });
      graph.addEdge({
        id: nextEdgeId(),
        type: 'PARENT_GROVE',
        from: id,
        to: `grove-root-${i}`,
        properties: {},
      });
    }
  }

  // Level 2: 2-3 leaf groves per mid grove
  for (const midId of level1Ids) {
    const numChildren = 2 + Math.floor(random() * 2);
    for (let j = 0; j < numChildren; j++) {
      const id = `grove-leaf-${leafGroveIds.length}`;
      leafGroveIds.push(id);
      groveIds.push(id);
      graph.addNode({
        id,
        labels: ['Grove'],
        properties: { name: `Leaf Grove ${leafGroveIds.length - 1}`, level: 2 },
      });
      graph.addEdge({
        id: nextEdgeId(),
        type: 'PARENT_GROVE',
        from: id,
        to: midId,
        properties: {},
      });
    }
  }

  // Governance rules: 5-10 rules per grove
  const ruleNames = [
    'moderation_level', 'max_post_length', 'allow_anonymous', 'voting_threshold',
    'ban_duration', 'content_review', 'membership_type', 'fork_policy',
    'data_retention', 'ai_access_level',
  ];

  for (const groveId of groveIds) {
    const numRules = 5 + Math.floor(random() * 6);
    const usedRules = new Set<string>();
    for (let i = 0; i < numRules; i++) {
      const ruleName = ruleNames[Math.floor(random() * ruleNames.length)]!;
      if (usedRules.has(ruleName)) continue;
      usedRules.add(ruleName);

      const ruleId = `rule-${governanceRuleIds.length}`;
      governanceRuleIds.push(ruleId);
      graph.addNode({
        id: ruleId,
        labels: ['GovernanceRule'],
        properties: {
          name: ruleName,
          value: Math.floor(random() * 100),
        },
      });
      graph.addEdge({
        id: nextEdgeId(),
        type: 'HAS_RULE',
        from: groveId,
        to: ruleId,
        properties: {},
      });
    }
  }

  // ─── View 5: Knowledge + Attestations ───

  const knowledgeNodeIds: string[] = [];
  for (let i = 0; i < 10000; i++) {
    const id = `knowledge-${i}`;
    knowledgeNodeIds.push(id);
    graph.addNode({
      id,
      labels: ['KnowledgeNode'],
      properties: {
        title: `Knowledge ${i}`,
        topic: `topic-${Math.floor(random() * 50)}`,
      },
    });
  }

  const attestorIds: string[] = [];
  for (let i = 0; i < 500; i++) {
    const id = `attestor-${i}`;
    attestorIds.push(id);
    graph.addNode({
      id,
      labels: ['Attestor'],
      properties: { name: `Attestor ${i}`, reputation: Math.floor(random() * 100) },
    });
  }

  const attestationIds: string[] = [];
  for (let i = 0; i < 100000; i++) {
    const id = `attestation-${i}`;
    attestationIds.push(id);
    const knowledgeIdx = Math.floor(random() * knowledgeNodeIds.length);
    const attestorIdx = Math.floor(random() * attestorIds.length);
    const value = 1 + Math.floor(random() * 10);

    graph.addNode({
      id,
      labels: ['Attestation'],
      properties: { value },
    });
    graph.addEdge({
      id: nextEdgeId(),
      type: 'ATTESTS_TO',
      from: id,
      to: knowledgeNodeIds[knowledgeIdx]!,
      properties: {},
    });
    graph.addEdge({
      id: nextEdgeId(),
      type: 'AUTHORED_BY',
      from: id,
      to: attestorIds[attestorIdx]!,
      properties: {},
    });
  }

  return {
    graph,
    metadata: {
      eventTypeIds,
      eventHandlerIds,
      entityIds,
      capabilityGrantIds,
      postIds,
      groveIds,
      leafGroveIds,
      governanceRuleIds,
      knowledgeNodeIds,
      attestationIds,
      attestorIds,
    },
  };
}

// ─── Write Operation Generators ───

export type WriteGenerator = (random: () => number, metadata: GeneratedData['metadata']) => WriteOp;

export function generateEventHandlerWrite(random: () => number, meta: GeneratedData['metadata']): WriteOp {
  const r = random();
  if (r < 0.4) {
    // Register new handler
    const id = `handler-new-${Math.floor(random() * 100000)}`;
    const typeIdx = Math.floor(random() * meta.eventTypeIds.length);
    return {
      kind: 'createNode',
      node: {
        id,
        labels: ['EventHandler'],
        properties: {
          name: id,
          priority: Math.floor(random() * 1000),
          module: `module-${Math.floor(random() * 20)}`,
        },
      },
    };
  } else if (r < 0.7) {
    // Update handler priority
    const idx = Math.floor(random() * meta.eventHandlerIds.length);
    return {
      kind: 'updateNode',
      id: meta.eventHandlerIds[idx]!,
      properties: { priority: Math.floor(random() * 1000) },
    };
  } else {
    // Remove handler
    const idx = Math.floor(random() * meta.eventHandlerIds.length);
    return {
      kind: 'deleteNode',
      id: meta.eventHandlerIds[idx]!,
    };
  }
}

export function generateCapabilityWrite(random: () => number, meta: GeneratedData['metadata']): WriteOp {
  const r = random();
  if (r < 0.5) {
    // Grant capability
    const capabilities = ['store:read', 'store:write', 'store:delete', 'compose:edit'];
    const scopes = ['content/*', 'composition/*', 'media/*'];
    const id = `capgrant-new-${Math.floor(random() * 100000)}`;
    return {
      kind: 'createNode',
      node: {
        id,
        labels: ['CapabilityGrant'],
        properties: {
          capability: capabilities[Math.floor(random() * capabilities.length)]!,
          scope: scopes[Math.floor(random() * scopes.length)]!,
        },
      },
    };
  } else {
    // Revoke: delete a grant
    const idx = Math.floor(random() * meta.capabilityGrantIds.length);
    return {
      kind: 'deleteNode',
      id: meta.capabilityGrantIds[idx]!,
    };
  }
}

export function generateContentWrite(random: () => number, meta: GeneratedData['metadata']): WriteOp {
  const r = random();
  if (r < 0.3) {
    // Create new post
    const id = `post-new-${Math.floor(random() * 100000)}`;
    const published = random() > 0.3;
    return {
      kind: 'createNode',
      node: {
        id,
        labels: ['Post'],
        properties: {
          title: `New Post ${id}`,
          published,
          publishedAt: published ? Date.now() - Math.floor(random() * 86400000) : 0,
          author: `user-${Math.floor(random() * 100)}`,
        },
      },
    };
  } else if (r < 0.7) {
    // Update post (publish/unpublish/change date)
    const idx = Math.floor(random() * meta.postIds.length);
    const published = random() > 0.3;
    return {
      kind: 'updateNode',
      id: meta.postIds[idx]!,
      properties: {
        published,
        publishedAt: published ? Date.now() - Math.floor(random() * 86400000) : 0,
      },
    };
  } else {
    // Delete post
    const idx = Math.floor(random() * meta.postIds.length);
    return {
      kind: 'deleteNode',
      id: meta.postIds[idx]!,
    };
  }
}

export function generateGovernanceWrite(random: () => number, meta: GeneratedData['metadata']): WriteOp {
  const r = random();
  if (r < 0.6) {
    // Update rule value
    const idx = Math.floor(random() * meta.governanceRuleIds.length);
    return {
      kind: 'updateNode',
      id: meta.governanceRuleIds[idx]!,
      properties: { value: Math.floor(random() * 100) },
    };
  } else {
    // Add new rule to a grove
    const ruleNames = ['moderation_level', 'max_post_length', 'allow_anonymous', 'voting_threshold'];
    const ruleId = `rule-new-${Math.floor(random() * 100000)}`;
    return {
      kind: 'createNode',
      node: {
        id: ruleId,
        labels: ['GovernanceRule'],
        properties: {
          name: ruleNames[Math.floor(random() * ruleNames.length)]!,
          value: Math.floor(random() * 100),
        },
      },
    };
  }
}

export function generateAttestationWrite(random: () => number, meta: GeneratedData['metadata']): WriteOp {
  const r = random();
  if (r < 0.5) {
    // New attestation
    const id = `attestation-new-${Math.floor(random() * 100000)}`;
    return {
      kind: 'createNode',
      node: {
        id,
        labels: ['Attestation'],
        properties: { value: 1 + Math.floor(random() * 10) },
      },
    };
  } else if (r < 0.8) {
    // Update attestation value
    const idx = Math.floor(random() * meta.attestationIds.length);
    return {
      kind: 'updateNode',
      id: meta.attestationIds[idx]!,
      properties: { value: 1 + Math.floor(random() * 10) },
    };
  } else {
    // Revoke attestation
    const idx = Math.floor(random() * meta.attestationIds.length);
    return {
      kind: 'deleteNode',
      id: meta.attestationIds[idx]!,
    };
  }
}

export const writeGenerators: Record<string, WriteGenerator> = {
  event_handlers: generateEventHandlerWrite,
  capability_check: generateCapabilityWrite,
  content_listing: generateContentWrite,
  governance_rules: generateGovernanceWrite,
  attestation_aggregate: generateAttestationWrite,
};
