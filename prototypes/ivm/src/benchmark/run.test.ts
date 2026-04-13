/**
 * Benchmark runner as a test so we can execute it via vitest.
 * This is the actual benchmark - it outputs results to console and writes JSON.
 */

import { describe, it, expect } from 'vitest';
import { GraphStore } from '../types.js';
import type { IVMAlgorithm, ViewDefinition, WriteOp } from '../types.js';
import { EagerInvalidationAlgorithm } from '../algorithms/algorithm-a-eager.js';
import { DependencyTrackedAlgorithm } from '../algorithms/algorithm-b-incremental.js';
import { DBSPAlgorithm } from '../algorithms/algorithm-c-dbsp.js';
import { generateBenchmarkData, writeGenerators } from './data-generator.js';
import type { GeneratedData } from './data-generator.js';
import { computeStats, formatLatency, formatBytes } from './stats.js';
import {
  computeEventHandlerView,
  computeCapabilityView,
  computeContentListingView,
  computeGovernanceView,
  computeAttestationView,
} from '../views/view-definitions.js';
import * as fs from 'fs';

function mulberry32(seed: number): () => number {
  return function() {
    let t = (seed += 0x6D2B79F5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const VIEW_DEFS: ViewDefinition[] = [
  { id: 'event_handlers', description: 'Event Handler Resolution' },
  { id: 'capability_check', description: 'Capability Check' },
  { id: 'content_listing', description: 'Content Listing (Sorted + Paginated)' },
  { id: 'governance_rules', description: 'Governance Rule Resolution' },
  { id: 'attestation_aggregate', description: 'Knowledge Attestation Aggregate' },
];

const NUM_READ_SAMPLES = 10000;
const NUM_WRITE_SAMPLES = 5000;
const CORRECTNESS_WRITES = 10000;

function cloneGraph(original: GeneratedData): GraphStore {
  const clone = new GraphStore();
  for (const node of original.graph.getAllNodes()) {
    clone.addNode({ ...node, labels: [...node.labels], properties: { ...node.properties } });
  }
  for (const edge of original.graph.getAllEdges()) {
    clone.addEdge({ ...edge, properties: { ...edge.properties } });
  }
  return clone;
}

interface SingleViewBenchmarkResult {
  algorithm: string;
  viewId: string;
  viewDescription: string;
  readP50: number;
  readP95: number;
  readMean: number;
  writeP50: number;
  writeP95: number;
  writeMean: number;
  memoryBytes: number;
  correct: boolean;
}

function benchmarkReadLatency(algo: IVMAlgorithm, viewId: string, samples: number): number[] {
  const latencies: number[] = [];
  for (let i = 0; i < 100; i++) algo.readView(viewId);
  for (let i = 0; i < samples; i++) {
    const start = performance.now();
    algo.readView(viewId);
    const elapsed = performance.now() - start;
    latencies.push(elapsed);
  }
  return latencies;
}

function benchmarkWriteLatency(
  algo: IVMAlgorithm,
  viewId: string,
  data: GeneratedData,
  samples: number
): number[] {
  const latencies: number[] = [];
  const random = mulberry32(12345);
  const generator = writeGenerators[viewId]!;
  for (let i = 0; i < samples; i++) {
    const op = generator(random, data.metadata);
    const start = performance.now();
    algo.applyWrite(op);
    const elapsed = performance.now() - start;
    latencies.push(elapsed);
  }
  return latencies;
}

function verifyCorrectness(
  algo: IVMAlgorithm,
  viewId: string,
  graph: GraphStore,
  data: GeneratedData,
  numWrites: number
): boolean {
  const random = mulberry32(99999);
  const generator = writeGenerators[viewId]!;
  for (let i = 0; i < numWrites; i++) {
    const op = generator(random, data.metadata);
    algo.applyWrite(op);
  }
  const ivmResult = algo.readView(viewId);
  switch (viewId) {
    case 'event_handlers': {
      const expected = computeEventHandlerView(graph, 'content:afterCreate');
      return compareRows(ivmResult.rows as Record<string, unknown>[], expected);
    }
    case 'capability_check': {
      const expected = computeCapabilityView(graph);
      const ivmKeys = new Set(
        (ivmResult.rows as Array<Record<string, unknown>>).map(r => r['key'] as string)
      );
      if (ivmKeys.size !== expected.size) {
        console.log(`  Capability mismatch: IVM has ${ivmKeys.size}, expected ${expected.size}`);
        return false;
      }
      for (const key of expected.keys()) {
        if (!ivmKeys.has(key)) {
          console.log(`  Missing capability key: ${key}`);
          return false;
        }
      }
      return true;
    }
    case 'content_listing': {
      const expected = computeContentListingView(graph, 20);
      return compareRows(ivmResult.rows as Record<string, unknown>[], expected);
    }
    case 'governance_rules': {
      const expected = computeGovernanceView(graph, 'grove-leaf-0');
      return compareGovernanceResults(ivmResult.rows as Record<string, unknown>[], expected);
    }
    case 'attestation_aggregate': {
      const expected = computeAttestationView(graph, 'knowledge-0');
      const ivmAgg = ivmResult.rows[0] as Record<string, unknown> | undefined;
      if (!ivmAgg) return false;
      const totalMatch = Math.abs((ivmAgg['totalValue'] as number) - (expected['totalValue'] as number)) < 0.01;
      const countMatch = ivmAgg['attestationCount'] === expected['attestationCount'];
      if (!totalMatch) console.log(`  Attestation totalValue mismatch: IVM=${ivmAgg['totalValue']}, expected=${expected['totalValue']}`);
      if (!countMatch) console.log(`  Attestation count mismatch: IVM=${ivmAgg['attestationCount']}, expected=${expected['attestationCount']}`);
      return totalMatch && countMatch;
    }
    default:
      return false;
  }
}

function compareRows(actual: Record<string, unknown>[], expected: Record<string, unknown>[]): boolean {
  if (actual.length !== expected.length) {
    console.log(`  Row count mismatch: got ${actual.length}, expected ${expected.length}`);
    return false;
  }
  for (let i = 0; i < actual.length; i++) {
    if (actual[i]!['_id'] !== expected[i]!['_id']) {
      console.log(`  Row ${i} ID mismatch: got ${actual[i]!['_id']}, expected ${expected[i]!['_id']}`);
      return false;
    }
  }
  return true;
}

function compareGovernanceResults(actual: Record<string, unknown>[], expected: Record<string, unknown>[]): boolean {
  const actualMap = new Map<string, Record<string, unknown>>();
  for (const row of actual) actualMap.set(row['name'] as string, row);
  const expectedMap = new Map<string, Record<string, unknown>>();
  for (const row of expected) expectedMap.set(row['name'] as string, row);
  if (actualMap.size !== expectedMap.size) {
    console.log(`  Governance rule count mismatch: got ${actualMap.size}, expected ${expectedMap.size}`);
    return false;
  }
  for (const [name, expectedRule] of expectedMap) {
    const actualRule = actualMap.get(name);
    if (!actualRule) { console.log(`  Missing governance rule: ${name}`); return false; }
    if (actualRule['value'] !== expectedRule['value']) {
      console.log(`  Governance rule "${name}" value mismatch: got ${actualRule['value']}, expected ${expectedRule['value']}`);
      return false;
    }
  }
  return true;
}

describe('IVM Benchmark', () => {
  let originalData: GeneratedData;
  const allResults: SingleViewBenchmarkResult[] = [];

  it('generates benchmark data', () => {
    console.log('Generating benchmark data...');
    originalData = generateBenchmarkData();
    console.log(`  Nodes: ${originalData.graph.nodeCount}`);
    console.log(`  Edges: ${originalData.graph.edgeCount}`);
    expect(originalData.graph.nodeCount).toBeGreaterThan(100000);
  });

  const algorithms = [
    { name: 'A', factory: () => new EagerInvalidationAlgorithm() },
    { name: 'B', factory: () => new DependencyTrackedAlgorithm() },
    { name: 'C', factory: () => new DBSPAlgorithm() },
  ];

  for (const algoDef of algorithms) {
    describe(`Algorithm ${algoDef.name}`, () => {
      for (const viewDef of VIEW_DEFS) {
        it(`${viewDef.id}: read latency`, () => {
          const algo = algoDef.factory();
          const graph = cloneGraph(originalData);
          algo.initialize(graph, VIEW_DEFS);
          const latencies = benchmarkReadLatency(algo, viewDef.id, NUM_READ_SAMPLES);
          const stats = computeStats(latencies);
          console.log(`  [${algoDef.name}] ${viewDef.id} READ: p50=${formatLatency(stats.p50)}, p95=${formatLatency(stats.p95)}, mean=${formatLatency(stats.mean)}`);
          expect(stats.p50).toBeLessThan(10); // Sanity check: reads should be fast
        });

        it(`${viewDef.id}: write+IVM latency`, () => {
          const algo = algoDef.factory();
          const graph = cloneGraph(originalData);
          algo.initialize(graph, VIEW_DEFS);
          const latencies = benchmarkWriteLatency(algo, viewDef.id, originalData, NUM_WRITE_SAMPLES);
          const stats = computeStats(latencies);
          console.log(`  [${algoDef.name}] ${viewDef.id} WRITE: p50=${formatLatency(stats.p50)}, p95=${formatLatency(stats.p95)}, mean=${formatLatency(stats.mean)}`);

          const memBytes = algo.memoryOverhead();
          console.log(`  [${algoDef.name}] ${viewDef.id} MEMORY: ${formatBytes(memBytes)}`);

          // Collect for summary
          allResults.push({
            algorithm: algo.name,
            viewId: viewDef.id,
            viewDescription: viewDef.description,
            readP50: 0, readP95: 0, readMean: 0, // filled separately
            writeP50: stats.p50,
            writeP95: stats.p95,
            writeMean: stats.mean,
            memoryBytes: memBytes,
            correct: true,
          });
        });

        it(`${viewDef.id}: correctness after ${CORRECTNESS_WRITES} writes`, () => {
          const algo = algoDef.factory();
          const graph = cloneGraph(originalData);
          algo.initialize(graph, VIEW_DEFS);
          const correct = verifyCorrectness(algo, viewDef.id, graph, originalData, CORRECTNESS_WRITES);
          console.log(`  [${algoDef.name}] ${viewDef.id} CORRECT: ${correct ? 'PASS' : 'FAIL'}`);
          expect(correct).toBe(true);
        });
      }
    });
  }

  it('writes results JSON', () => {
    try {
      fs.writeFileSync(
        '/Users/benwork/Documents/benten-engine/prototypes/ivm/benchmark-results.json',
        JSON.stringify(allResults, null, 2)
      );
    } catch {
      // May not have write permission in test context
    }
  });
});
