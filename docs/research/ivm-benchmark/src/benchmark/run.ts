/**
 * Benchmark Runner
 *
 * Runs all 3 algorithms against all 5 view patterns.
 * Measures read latency, write+IVM latency, memory overhead, and correctness.
 */

import { GraphStore } from '../types.js';
import type { IVMAlgorithm, ViewDefinition, BenchmarkResult, WriteOp } from '../types.js';
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

// Deterministic PRNG
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
  // Warm up
  for (let i = 0; i < 100; i++) {
    algo.readView(viewId);
  }
  // Measure
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

  // Apply random writes
  for (let i = 0; i < numWrites; i++) {
    const op = generator(random, data.metadata);
    algo.applyWrite(op);
  }

  // Read the IVM view
  const ivmResult = algo.readView(viewId);

  // Full recompute from scratch on the same graph
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
      // Allow small tolerance for floating point
      const totalMatch = Math.abs((ivmAgg['totalValue'] as number) - (expected['totalValue'] as number)) < 0.01;
      const countMatch = ivmAgg['attestationCount'] === expected['attestationCount'];
      if (!totalMatch) {
        console.log(`  Attestation totalValue mismatch: IVM=${ivmAgg['totalValue']}, expected=${expected['totalValue']}`);
      }
      if (!countMatch) {
        console.log(`  Attestation count mismatch: IVM=${ivmAgg['attestationCount']}, expected=${expected['attestationCount']}`);
      }
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
    const a = actual[i]!;
    const e = expected[i]!;
    if (a['_id'] !== e['_id']) {
      console.log(`  Row ${i} ID mismatch: got ${a['_id']}, expected ${e['_id']}`);
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
    if (!actualRule) {
      console.log(`  Missing governance rule: ${name}`);
      return false;
    }
    if (actualRule['value'] !== expectedRule['value']) {
      console.log(`  Governance rule "${name}" value mismatch: got ${actualRule['value']}, expected ${expectedRule['value']}`);
      return false;
    }
  }

  return true;
}

// ─── Main Benchmark Runner ───

async function runBenchmarks(): Promise<void> {
  console.log('=== IVM Algorithm Benchmark ===\n');
  console.log('Generating benchmark data...');

  const originalData = generateBenchmarkData();
  console.log(`  Nodes: ${originalData.graph.nodeCount}`);
  console.log(`  Edges: ${originalData.graph.edgeCount}`);
  console.log('');

  const algorithms: IVMAlgorithm[] = [
    new EagerInvalidationAlgorithm(),
    new DependencyTrackedAlgorithm(),
    new DBSPAlgorithm(),
  ];

  const allResults: SingleViewBenchmarkResult[] = [];

  for (const algo of algorithms) {
    console.log(`\n--- ${algo.name} ---\n`);

    for (const viewDef of VIEW_DEFS) {
      console.log(`  View: ${viewDef.description} (${viewDef.id})`);

      // Fresh graph clone for each algorithm+view combo
      const graph = cloneGraph(originalData);
      algo.reset();
      algo.initialize(graph, VIEW_DEFS);

      // 1. Read latency (before any writes)
      const readLatencies = benchmarkReadLatency(algo, viewDef.id, NUM_READ_SAMPLES);
      const readStats = computeStats(readLatencies);

      // 2. Write + IVM update latency
      // Use a fresh clone for writes to keep isolation
      const writeGraph = cloneGraph(originalData);
      algo.reset();
      algo.initialize(writeGraph, VIEW_DEFS);
      const writeLatencies = benchmarkWriteLatency(algo, viewDef.id, originalData, NUM_WRITE_SAMPLES);
      const writeStats = computeStats(writeLatencies);

      // 3. Memory overhead
      const memBytes = algo.memoryOverhead();

      // 4. Correctness verification
      const correctGraph = cloneGraph(originalData);
      algo.reset();
      algo.initialize(correctGraph, VIEW_DEFS);
      const correct = verifyCorrectness(algo, viewDef.id, correctGraph, originalData, CORRECTNESS_WRITES);

      const result: SingleViewBenchmarkResult = {
        algorithm: algo.name,
        viewId: viewDef.id,
        viewDescription: viewDef.description,
        readP50: readStats.p50,
        readP95: readStats.p95,
        readMean: readStats.mean,
        writeP50: writeStats.p50,
        writeP95: writeStats.p95,
        writeMean: writeStats.mean,
        memoryBytes: memBytes,
        correct,
      };
      allResults.push(result);

      console.log(`    Read:  p50=${formatLatency(readStats.p50)}, p95=${formatLatency(readStats.p95)}, mean=${formatLatency(readStats.mean)}`);
      console.log(`    Write: p50=${formatLatency(writeStats.p50)}, p95=${formatLatency(writeStats.p95)}, mean=${formatLatency(writeStats.mean)}`);
      console.log(`    Memory: ${formatBytes(memBytes)}`);
      console.log(`    Correct: ${correct ? 'YES' : 'NO'}`);
      console.log('');
    }
  }

  // ─── Summary Tables ───

  console.log('\n=== SUMMARY TABLES ===\n');

  // Read latency table
  console.log('READ LATENCY (p50 / p95):');
  console.log(''.padEnd(35) + algorithms.map(a => a.name.substring(0, 30).padEnd(35)).join(''));
  for (const viewDef of VIEW_DEFS) {
    const cells = algorithms.map(algo => {
      const r = allResults.find(x => x.algorithm === algo.name && x.viewId === viewDef.id);
      return r ? `${formatLatency(r.readP50)} / ${formatLatency(r.readP95)}`.padEnd(35) : 'N/A'.padEnd(35);
    });
    console.log(`${viewDef.description.padEnd(35)}${cells.join('')}`);
  }

  console.log('');

  // Write latency table
  console.log('WRITE + IVM LATENCY (p50 / p95):');
  console.log(''.padEnd(35) + algorithms.map(a => a.name.substring(0, 30).padEnd(35)).join(''));
  for (const viewDef of VIEW_DEFS) {
    const cells = algorithms.map(algo => {
      const r = allResults.find(x => x.algorithm === algo.name && x.viewId === viewDef.id);
      return r ? `${formatLatency(r.writeP50)} / ${formatLatency(r.writeP95)}`.padEnd(35) : 'N/A'.padEnd(35);
    });
    console.log(`${viewDef.description.padEnd(35)}${cells.join('')}`);
  }

  console.log('');

  // Memory table
  console.log('MEMORY OVERHEAD:');
  console.log(''.padEnd(35) + algorithms.map(a => a.name.substring(0, 30).padEnd(35)).join(''));
  for (const viewDef of VIEW_DEFS) {
    const cells = algorithms.map(algo => {
      const r = allResults.find(x => x.algorithm === algo.name && x.viewId === viewDef.id);
      return r ? formatBytes(r.memoryBytes).padEnd(35) : 'N/A'.padEnd(35);
    });
    console.log(`${viewDef.description.padEnd(35)}${cells.join('')}`);
  }

  console.log('');

  // Correctness table
  console.log('CORRECTNESS:');
  console.log(''.padEnd(35) + algorithms.map(a => a.name.substring(0, 30).padEnd(35)).join(''));
  for (const viewDef of VIEW_DEFS) {
    const cells = algorithms.map(algo => {
      const r = allResults.find(x => x.algorithm === algo.name && x.viewId === viewDef.id);
      return r ? (r.correct ? 'PASS' : 'FAIL').padEnd(35) : 'N/A'.padEnd(35);
    });
    console.log(`${viewDef.description.padEnd(35)}${cells.join('')}`);
  }

  // Output JSON results for RESULTS.md generation
  const jsonPath = '/Users/benwork/Documents/benten-engine/prototypes/ivm/benchmark-results.json';
  const fs = await import('fs');
  fs.writeFileSync(jsonPath, JSON.stringify(allResults, null, 2));
  console.log(`\nRaw results written to ${jsonPath}`);
}

runBenchmarks().catch(console.error);
