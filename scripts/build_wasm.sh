#!/usr/bin/env bash
# Phase-2b G7-B / D26-RESOLVED — dev-only WASM fixture regenerator.
#
# Compiles the .wat fixture sources under
# crates/benten-eval/tests/fixtures/sandbox/ into committed .wasm
# binaries. This script is **dev-only** — it is NOT a CI dependency.
# CI runs against the committed .wasm bytes via
# `tests/fixture_wasm_hashes_stable` (drift detector).
#
# Why dev-only? wsa-12 noted CI shell-portability issues:
#   - Windows runners don't have bash by default.
#   - macOS runners need wabt (wat2wasm) installed.
#   - Linux runners need rust-target-add wasm32-wasip1 for Rust-built
#     fixtures.
# Committing pre-built bytes side-steps all three. The .wat source is
# the canonical input; the .wasm output is the cache. The drift detector
# enforces equality.
#
# Prerequisites (dev machines only):
#   - wabt (Homebrew: `brew install wabt`)
#   - or wasmtime CLI (`wasmtime compile` for some fixtures)
#
# Usage:
#   ./scripts/build_wasm.sh           # rebuild all .wat → .wasm
#   ./scripts/build_wasm.sh --check   # verify .wasm bytes match .wat sources
#                                       (mirrors the drift-detector test;
#                                       useful pre-commit)
#
# When adding a new .wat fixture:
#   1. Author the .wat under crates/benten-eval/tests/fixtures/sandbox/.
#   2. Run `./scripts/build_wasm.sh` to materialise the .wasm.
#   3. Commit BOTH the .wat AND the .wasm. The drift detector will pin
#      both against future drift.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
FIXTURE_ROOT="${REPO_ROOT}/crates/benten-eval/tests/fixtures/sandbox"

CHECK_MODE=0
if [[ "${1:-}" == "--check" ]]; then
    CHECK_MODE=1
fi

if ! command -v wat2wasm >/dev/null 2>&1; then
    echo "ERROR: wat2wasm not found. Install wabt (e.g. 'brew install wabt')." >&2
    exit 1
fi

shopt -s nullglob

# G7-B-owned fixtures only (depth_nest_* + output_overflow_*). Sibling
# subdirectories under FIXTURE_ROOT (e.g. `escape/`) are owned by other
# G7-* briefs and are NOT regenerated here — each owning brief ships its
# own build invocation. Restricting the glob to the FIXTURE_ROOT top
# level keeps this script's failure surface scoped to G7-B's own .wat.
count=0
diff_count=0
for wat in "${FIXTURE_ROOT}"/*.wat; do
    count=$((count + 1))
    wasm="${wat%.wat}.wasm"
    if [[ "${CHECK_MODE}" == 1 ]]; then
        # Build to a temp file and diff bytes.
        tmp="$(mktemp -t bentenwasm.XXXXXX)"
        # shellcheck disable=SC2064
        trap "rm -f '${tmp}'" EXIT
        wat2wasm "${wat}" -o "${tmp}"
        if [[ ! -f "${wasm}" ]]; then
            echo "DRIFT: ${wat} has no committed .wasm at ${wasm}" >&2
            diff_count=$((diff_count + 1))
        elif ! cmp -s "${tmp}" "${wasm}"; then
            echo "DRIFT: ${wasm} bytes differ from compile of ${wat}" >&2
            diff_count=$((diff_count + 1))
        fi
        rm -f "${tmp}"
        trap - EXIT
    else
        wat2wasm "${wat}" -o "${wasm}"
        echo "built: ${wasm}"
    fi
done

if [[ "${CHECK_MODE}" == 1 ]]; then
    if [[ "${diff_count}" -gt 0 ]]; then
        echo "FAIL: ${diff_count} of ${count} fixture(s) drift" >&2
        exit 2
    fi
    echo "OK: ${count} fixture(s) match"
else
    echo "OK: ${count} fixture(s) built"
fi
