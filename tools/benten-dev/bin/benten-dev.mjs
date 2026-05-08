#!/usr/bin/env node
// Phase-3 G20-A3 wave-8a — `benten-dev` thin-CLI front-door per
// `docs/future/phase-3-backlog.md` §6.9. Wraps the Rust-side
// pretty-printer entry point at
// `tools/benten-dev/src/inspect_state.rs::pretty_print_envelope_bytes`
// (compiled as the `benten-dev` Cargo binary).
//
// Subcommands:
//   inspect-state <path>                  pretty-print suspended ExecutionState
//   inspect-state <path> --with-protocol-hints
//                                         additionally echo the 4-step resume
//                                         protocol headers
//
// Flags:
//   --help, -h
//   --version, -V
//
// The CLI runs by spawning the compiled `benten-dev` binary (built
// via `cargo build -p benten-dev`) so the JS wrapper never touches
// envelope bytes directly. This keeps the canonical-bytes parsing
// in Rust (one source of truth) while the JS surface stays the
// thin DX entry point per the §6.9 commitment.

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import process from "node:process";

const __dirname = dirname(fileURLToPath(import.meta.url));

const USAGE = `usage: benten-dev <subcommand> [args]

subcommands:
  inspect-state <path>                pretty-print a suspended ExecutionStateEnvelope
  inspect-state <path> --with-protocol-hints
                                       additionally echo the 4-step resume protocol

options:
  -h, --help                          print this help text and exit 0
  -V, --version                       print version and exit 0
`;

function findRustBinary() {
  // Prefer the workspace target/{debug,release}/benten-dev. Falls back
  // to PATH lookup so deployments that pre-install the binary also work.
  const workspaceRoot = resolve(__dirname, "..", "..", "..");
  const candidates = [
    join(workspaceRoot, "target", "release", "benten-dev"),
    join(workspaceRoot, "target", "debug", "benten-dev"),
    join(workspaceRoot, "target", "release", "benten-dev.exe"),
    join(workspaceRoot, "target", "debug", "benten-dev.exe"),
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  // PATH fallback — the binary may have been `cargo install`'d.
  return "benten-dev";
}

function main() {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    process.stderr.write(USAGE);
    process.exit(2);
  }

  const subcmd = args[0];

  if (subcmd === "--help" || subcmd === "-h") {
    process.stdout.write(USAGE);
    process.exit(0);
  }

  if (subcmd === "--version" || subcmd === "-V") {
    // The Rust binary owns the canonical version string; defer.
    const bin = findRustBinary();
    const r = spawnSync(bin, ["--version"], { encoding: "utf8" });
    if (r.error) {
      process.stderr.write(
        `benten-dev: cannot locate Rust binary (${bin}): ${r.error.message}\n` +
          `Run \`cargo build -p benten-dev\` to compile it.\n`,
      );
      process.exit(1);
    }
    if (r.stdout) process.stdout.write(r.stdout);
    if (r.stderr) process.stderr.write(r.stderr);
    process.exit(r.status ?? 0);
  }

  if (subcmd === "inspect-state") {
    const path = args[1];
    if (!path) {
      process.stderr.write("usage: benten-dev inspect-state <path>\n");
      process.exit(2);
    }
    const bin = findRustBinary();
    // Forward all args after `inspect-state` (so `--with-protocol-hints`
    // flows through to the Rust binary if/when it implements that flag).
    const r = spawnSync(bin, ["inspect-state", ...args.slice(1)], {
      encoding: "utf8",
    });
    if (r.error) {
      process.stderr.write(
        `benten-dev: cannot locate Rust binary (${bin}): ${r.error.message}\n` +
          `Run \`cargo build -p benten-dev\` to compile it.\n`,
      );
      process.exit(1);
    }
    if (r.stdout) process.stdout.write(r.stdout);
    if (r.stderr) process.stderr.write(r.stderr);
    process.exit(r.status ?? 0);
  }

  process.stderr.write(`benten-dev: unknown subcommand "${subcmd}"\n`);
  process.exit(2);
}

main();
