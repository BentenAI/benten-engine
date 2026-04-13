/**
 * Statistical helpers for benchmark measurements.
 */

import type { LatencyStats } from '../types.js';

export function computeStats(samples: number[]): LatencyStats {
  if (samples.length === 0) {
    return { p50: 0, p95: 0, mean: 0, min: 0, max: 0, count: 0 };
  }

  const sorted = [...samples].sort((a, b) => a - b);
  const count = sorted.length;
  const sum = sorted.reduce((acc, v) => acc + v, 0);

  return {
    p50: sorted[Math.floor(count * 0.50)]!,
    p95: sorted[Math.floor(count * 0.95)]!,
    mean: sum / count,
    min: sorted[0]!,
    max: sorted[count - 1]!,
    count,
  };
}

export function formatLatency(ms: number): string {
  if (ms < 0.001) return `${(ms * 1000).toFixed(2)}us`;
  if (ms < 1) return `${ms.toFixed(4)}ms`;
  return `${ms.toFixed(2)}ms`;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
}
