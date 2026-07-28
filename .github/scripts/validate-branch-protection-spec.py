#!/usr/bin/env python3
"""Self-consistency validator for `.github/branch-protection.yml`.

F-015 closure (R6 round #1, 2026-07-27).

WHY THIS EXISTS. `branch-protection-spec-check.yml` could not fail. Its strict
flag defaulted to `false`, its live-state fetch degraded to diffing `{}` when no
PAT was configured, and the workflow was not itself a required context — so the
one thing guarding the required-check set emitted warnings nobody was obliged to
read. A detector that cannot fail is decoration.

WHAT THIS VALIDATES. Everything about the spec that is checkable WITHOUT a PAT,
as hard failures:

  1. The spec parses and has the expected top-level shape.
  2. No duplicate context strings in the required set or the pending set.
  3. The required set and the pending set are disjoint. A context cannot be
     simultaneously "we enforce this" and "we cannot enforce this yet".
  4. Every context string resolves to a real job `name:` somewhere in
     `.github/workflows/`. This catches typos and renames — a required context
     that matches no job NEVER reports, and GitHub treats a never-reporting
     required context as blocking, so the typo bricks `main` permanently.
  5. Every REQUIRED context's workflow actually triggers on `pull_request`,
     without a `paths:` filter, and its job is not gated behind an
     `if:` that excludes pull-request events. This is the same deadlock class
     as (4) and it is the reason `bench-threshold-drift.yml` and
     `workspace benches (nightly)` are excluded from the required set.
  6. Every pending entry carries a non-empty `blocker:` — the honest reason it
     is not enforceable today (C5's four cross-browser echo stubs, CE-02's
     `exit 0` bundle-size stub). Ambition is allowed; unexplained ambition is
     not.

MUTATION THAT MUST MAKE THIS FAIL: add `- "no such job"` to
`protection.required_status_checks.contexts` -> check 4 fails. Add
`- "workspace benches (nightly)"` -> check 5 fails (schedule-only job). Both
mutations are exercised for real by the `spec-self-consistency-self-test` job in
`branch-protection-spec-check.yml`; if this validator ever stops detecting them,
that job goes red.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

SPEC_DEFAULT = ".github/branch-protection.yml"
WORKFLOW_DIR = Path(".github/workflows")

# `${{ matrix.leg }}` and friends render into the context string at run time, so
# a job name is a TEMPLATE. Turn it into a regex to match contexts against.
EXPR = re.compile(r"\$\{\{[^}]*\}\}")


def job_name_to_regex(name: str) -> re.Pattern[str]:
    parts = [re.escape(p) for p in EXPR.split(name)]
    return re.compile(r"\A" + ".+".join(parts) + r"\Z")


def load_workflows() -> list[tuple[Path, dict]]:
    out = []
    for path in sorted(WORKFLOW_DIR.glob("*.yml")) + sorted(WORKFLOW_DIR.glob("*.yaml")):
        try:
            doc = yaml.safe_load(path.read_text())
        except yaml.YAMLError as exc:  # a malformed workflow is its own bug
            print(f"::error::{path} is not valid YAML: {exc}")
            raise SystemExit(1)
        if isinstance(doc, dict):
            out.append((path, doc))
    return out


def triggers(doc: dict) -> dict:
    # PyYAML parses the bare key `on:` as the boolean True (YAML 1.1 truthiness).
    raw = doc.get("on", doc.get(True))
    if raw is None:
        return {}
    if isinstance(raw, str):
        return {raw: None}
    if isinstance(raw, list):
        return {k: None for k in raw}
    return raw if isinstance(raw, dict) else {}


def runs_on_pull_request(doc: dict) -> tuple[bool, str]:
    on = triggers(doc)
    if "pull_request" not in on:
        return False, "workflow has no `pull_request` trigger"
    cfg = on["pull_request"] or {}
    if isinstance(cfg, dict) and ("paths" in cfg or "paths-ignore" in cfg):
        return False, (
            "`pull_request` carries a paths filter; the workflow will not run on "
            "PRs that miss the filter, and GitHub blocks on an absent required context"
        )
    return True, ""


def job_runs_on_pull_request(job: dict) -> tuple[bool, str]:
    cond = job.get("if")
    if cond is None:
        return True, ""
    text = str(cond)
    # Only flag conditions that demonstrably exclude pull-request events. A
    # broad `if: always()` or a matrix-conditional is fine.
    if re.search(r"event_name\s*==\s*'(schedule|workflow_dispatch|push)'", text) and (
        "pull_request" not in text
    ):
        return False, f"job `if:` excludes pull-request events: {text!r}"
    if re.search(r"event_name\s*!=\s*'pull_request'", text):
        return False, f"job `if:` excludes pull-request events: {text!r}"
    return True, ""


def main() -> int:
    spec_path = sys.argv[1] if len(sys.argv) > 1 else SPEC_DEFAULT
    errors: list[str] = []

    spec = yaml.safe_load(Path(spec_path).read_text())
    if not isinstance(spec, dict) or "protection" not in spec:
        print(f"::error::{spec_path}: missing top-level `protection` key")
        return 1

    protection = spec["protection"]
    required = list(
        (protection.get("required_status_checks") or {}).get("contexts") or []
    )
    pending_entries = spec.get("pending_promotion") or []

    # ---- 2. duplicates ------------------------------------------------------
    for label, items in (("required", required), ("pending", [e.get("context") for e in pending_entries])):
        seen: set[str] = set()
        for ctx in items:
            if ctx in seen:
                errors.append(f"duplicate {label} context: {ctx!r}")
            seen.add(ctx)

    pending = [e.get("context") for e in pending_entries]

    # ---- 3. disjoint --------------------------------------------------------
    for ctx in sorted(set(required) & set(pending)):
        errors.append(
            f"context {ctx!r} appears in BOTH the required set and pending_promotion"
        )

    # ---- 6. pending entries explain themselves ------------------------------
    for entry in pending_entries:
        if not entry.get("context"):
            errors.append("a pending_promotion entry has no `context`")
        elif not str(entry.get("blocker") or "").strip():
            errors.append(
                f"pending_promotion entry {entry['context']!r} has no `blocker:` — "
                "say why it cannot be enforced today"
            )

    # ---- 4 + 5. contexts resolve to real, PR-reachable jobs ------------------
    workflows = load_workflows()
    index: list[tuple[re.Pattern[str], Path, dict, dict]] = []
    for path, doc in workflows:
        for job in (doc.get("jobs") or {}).values():
            if not isinstance(job, dict):
                continue
            name = job.get("name")
            if not name:
                continue
            index.append((job_name_to_regex(str(name)), path, doc, job))

    def resolve(ctx: str):
        return [entry for entry in index if entry[0].match(ctx)]

    for ctx in required:
        matches = resolve(ctx)
        if not matches:
            errors.append(
                f"REQUIRED context {ctx!r} matches no job `name:` in .github/workflows/. "
                "A required context that never reports blocks every PR forever."
            )
            continue
        if not any(runs_on_pull_request(doc)[0] and job_runs_on_pull_request(job)[0] for _, _, doc, job in matches):
            _, path, doc, job = matches[0]
            why = runs_on_pull_request(doc)[1] or job_runs_on_pull_request(job)[1]
            errors.append(
                f"REQUIRED context {ctx!r} ({path}) never runs on a pull request: {why}"
            )

    for ctx in pending:
        if ctx and not resolve(ctx):
            errors.append(
                f"pending_promotion context {ctx!r} matches no job `name:` in "
                ".github/workflows/ — it is unreachable ambition, not deferred ambition."
            )

    if errors:
        print("::error::branch-protection spec self-consistency FAILED")
        for err in errors:
            print(f"  - {err}")
        return 1

    print(
        f"spec self-consistency OK: {len(required)} required contexts, "
        f"{len(pending)} pending, all resolve to PR-reachable jobs."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
