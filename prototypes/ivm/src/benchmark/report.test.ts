/**
 * Focused benchmark that captures detailed numbers for the RESULTS.md report.
 * Runs each algorithm against each view and records precise latencies.
 */

import { describe, it, expect } from 'vitest';
import { GraphStore } from '../types.js';
import type { IVMAlgorithm, ViewDefinition, WriteOp, PropertyValue } from '../types.js';
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
  { id: 'event_handlers', description: 'View 1: Event Handler Resolution' },
  { id: 'capability_check', description: 'View 2: Capability Check' },
  { id: 'content_listing', description: 'View 3: Content Listing' },
  { id: 'governance_rules', description: 'View 4: Governance Rules' },
  { id: 'attestation_aggregate', description: 'View 5: Attestation Aggregate' },
];

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

interface BenchResult {
  algorithm: string;
  viewId: string;
  readP50: number;
  readP95: number;
  readMean: number;
  writeP50: number;
  writeP95: number;
  writeMean: number;
  memoryBytes: number;
  correct: boolean;
  initTimeMs: number;
}

describe('IVM Full Benchmark Report', () => {
  const results: BenchResult[] = [];
  let originalData: GeneratedData;

  it('generates data', () => {
    originalData = generateBenchmarkData();
    console.log(`Graph: ${originalData.graph.nodeCount} nodes, ${originalData.graph.edgeCount} edges`);
  });

  const algoFactories = [
    { label: 'A: Eager Invalidation', factory: () => new EagerInvalidationAlgorithm() },
    { label: 'B: Dep-Tracked Incremental', factory: () => new DependencyTrackedAlgorithm() },
    { label: 'C: DBSP / Z-Set', factory: () => new DBSPAlgorithm() },
  ];

  for (const af of algoFactories) {
    for (const vd of VIEW_DEFS) {
      it(`${af.label} | ${vd.id}`, () => {
        const algo = af.factory();

        // === Initialize ===
        const graph = cloneGraph(originalData);
        const initStart = performance.now();
        algo.initialize(graph, VIEW_DEFS);
        const initTime = performance.now() - initStart;

        // === Read latency (10,000 reads, no writes in between) ===
        // Warm up
        for (let i = 0; i < 200; i++) algo.readView(vd.id);
        const readSamples: number[] = [];
        for (let i = 0; i < 10000; i++) {
          const s = performance.now();
          algo.readView(vd.id);
          readSamples.push(performance.now() - s);
        }
        const readStats = computeStats(readSamples);

        // === Write + IVM latency (5,000 writes) ===
        const random = mulberry32(12345);
        const gen = writeGenerators[vd.id]!;
        const writeSamples: number[] = [];
        for (let i = 0; i < 5000; i++) {
          const op = gen(random, originalData.metadata);
          const s = performance.now();
          algo.applyWrite(op);
          writeSamples.push(performance.now() - s);
        }
        const writeStats = computeStats(writeSamples);

        const mem = algo.memoryOverhead();

        // === Correctness (fresh clone, 10,000 writes, then compare) ===
        const cAlgo = af.factory();
        const cGraph = cloneGraph(originalData);
        cAlgo.initialize(cGraph, VIEW_DEFS);
        const cRandom = mulberry32(99999);
        for (let i = 0; i < 10000; i++) {
          const op = gen(cRandom, originalData.metadata);
          cAlgo.applyWrite(op);
        }
        const ivmResult = cAlgo.readView(vd.id);
        let correct = false;
        switch (vd.id) {
          case 'event_handlers': {
            const exp = computeEventHandlerView(cGraph, 'content:afterCreate');
            correct = ivmResult.rows.length === exp.length;
            break;
          }
          case 'capability_check': {
            const exp = computeCapabilityView(cGraph);
            const ivmKeys = new Set((ivmResult.rows as Array<Record<string, unknown>>).map(r => r['key'] as string));
            correct = ivmKeys.size === exp.size;
            break;
          }
          case 'content_listing': {
            const exp = computeContentListingView(cGraph, 20);
            correct = ivmResult.rows.length === exp.length &&
              ivmResult.rows.every((r, i) => (r as Record<string, unknown>)['_id'] === (exp[i] as Record<string, unknown>)?.['_id']);
            break;
          }
          case 'governance_rules': {
            const exp = computeGovernanceView(cGraph, 'grove-leaf-0');
            const expMap = new Map(exp.map(r => [r['name'] as string, r]));
            const actMap = new Map((ivmResult.rows as Array<Record<string, unknown>>).map(r => [r['name'] as string, r]));
            correct = expMap.size === actMap.size;
            for (const [name, eRule] of expMap) {
              const aRule = actMap.get(name);
              if (!aRule || aRule['value'] !== eRule['value']) { correct = false; break; }
            }
            break;
          }
          case 'attestation_aggregate': {
            const exp = computeAttestationView(cGraph, 'knowledge-0');
            const agg = ivmResult.rows[0] as Record<string, unknown> | undefined;
            correct = !!agg &&
              Math.abs((agg['totalValue'] as number) - (exp['totalValue'] as number)) < 0.01 &&
              agg['attestationCount'] === exp['attestationCount'];
            break;
          }
        }

        const r: BenchResult = {
          algorithm: af.label,
          viewId: vd.id,
          readP50: readStats.p50,
          readP95: readStats.p95,
          readMean: readStats.mean,
          writeP50: writeStats.p50,
          writeP95: writeStats.p95,
          writeMean: writeStats.mean,
          memoryBytes: mem,
          correct,
          initTimeMs: initTime,
        };
        results.push(r);

        console.log(`  Init: ${initTime.toFixed(1)}ms | Read p50=${formatLatency(readStats.p50)} p95=${formatLatency(readStats.p95)} | Write p50=${formatLatency(writeStats.p50)} p95=${formatLatency(writeStats.p95)} | Mem=${formatBytes(mem)} | Correct=${correct}`);
        expect(correct).toBe(true);
      });
    }
  }

  it('writes report JSON', () => {
    const path = '/Users/benwork/Documents/benten-engine/prototypes/ivm/benchmark-results.json';
    fs.writeFileSync(path, JSON.stringify(results, null, 2));
    console.log(`\nWrote ${results.length} results to ${path}`);

    // Print summary table
    console.log('\n=== LATENCY TABLE (all times in milliseconds) ===\n');
    const header = ['View', 'Algo', 'Read p50', 'Read p95', 'Write p50', 'Write p95', 'Memory', 'Init', 'Correct'];
    console.log(header.map((h, i) => h.padEnd(i === 0 ? 25 : i === 1 ? 30 : 12)).join(''));
    console.log('-'.repeat(160));

    for (const r of results) {
      const row = [
        r.viewId.padEnd(25),
        r.algorithm.padEnd(30),
        formatLatency(r.readP50).padEnd(12),
        formatLatency(r.readP95).padEnd(12),
        formatLatency(r.writeP50).padEnd(12),
        formatLatency(r.writeP95).padEnd(12),
        formatBytes(r.memoryBytes).padEnd(12),
        `${r.initTimeMs.toFixed(1)}ms`.padEnd(12),
        r.correct ? 'PASS' : 'FAIL',
      ];
      console.log(row.join(''));
    }
  });
});
