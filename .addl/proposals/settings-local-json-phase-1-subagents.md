# Proposed `settings.local.json` update for Phase 1 subagent permissions

**Status:** DRAFT for Ben's review. Not applied.

**Goal:** Give Phase 1 R3+ subagents enough permissions to commit their own test and code slices (eliminating the orchestrator-commits-all bottleneck) without opening the door to destructive actions. Also prune the accumulated one-off allows from pre-work that are no longer needed.

**Approach:** Replace the current file's entire `permissions.allow` list with broader, category-based patterns, and add a `permissions.deny` list for the destructive operations that stay explicitly off-limits.

---

## Why this is needed

During the spike today, every `rust-engineer` subagent dispatch failed on the same wall:

- Agent 1 (initial spike): couldn't `git commit` the six slices — orchestrator had to do it
- Agent 2 (rust-cid fork D2 path): couldn't `git clone`, `cargo check`, or `git push` — task escalated back to orchestrator
- Agent 3 (rust-cid fork D4 path): same wall — orchestrator executed the fork migration inline

The orchestrator has broader permissions than spawned subagents by default. This works for the spike (one orchestrator doing mechanical plumbing) but breaks Phase 1 R5, where **4-8 parallel implementation groups each need to commit their own slices**. If every commit routes through the orchestrator, the parallel structure collapses into a serial bottleneck.

---

## Proposed file contents

```json
{
  "permissions": {
    "allow": [
      "//====================================================================",
      "// Rust toolchain — non-destructive Cargo operations",
      "//====================================================================",
      "Bash(cargo check:*)",
      "Bash(cargo build:*)",
      "Bash(cargo test:*)",
      "Bash(cargo nextest:*)",
      "Bash(cargo bench:*)",
      "Bash(cargo doc:*)",
      "Bash(cargo fmt:*)",
      "Bash(cargo clippy:*)",
      "Bash(cargo run:*)",
      "Bash(cargo fetch:*)",
      "Bash(cargo tree:*)",
      "Bash(cargo update:*)",
      "Bash(cargo search:*)",
      "Bash(cargo --version)",
      "Bash(cargo metadata:*)",

      "//====================================================================",
      "// Git — read-only and non-destructive write operations",
      "//====================================================================",
      "Bash(git status:*)",
      "Bash(git diff:*)",
      "Bash(git log:*)",
      "Bash(git show:*)",
      "Bash(git ls-files:*)",
      "Bash(git branch:*)",
      "Bash(git checkout:*)",
      "Bash(git switch:*)",
      "Bash(git add:*)",
      "Bash(git commit:*)",
      "Bash(git mv:*)",
      "Bash(git restore:*)",
      "Bash(git stash:*)",
      "Bash(git fetch:*)",
      "Bash(git clone https://github.com/*)",
      "Bash(git push origin:*)",
      "Bash(git rebase:*)",
      "Bash(git filter-branch:*)",
      "Bash(git config *)",
      "Bash(git remote:*)",
      "Bash(git tag:*)",

      "//====================================================================",
      "// gh CLI — read-only + PR/issue creation and editing",
      "//====================================================================",
      "Bash(gh auth status)",
      "Bash(gh auth:*)",
      "Bash(gh repo view:*)",
      "Bash(gh repo list:*)",
      "Bash(gh repo clone:*)",
      "Bash(gh pr view:*)",
      "Bash(gh pr list:*)",
      "Bash(gh pr diff:*)",
      "Bash(gh pr checks:*)",
      "Bash(gh pr comment:*)",
      "Bash(gh pr create:*)",
      "Bash(gh pr edit:*)",
      "Bash(gh pr review:*)",
      "Bash(gh issue view:*)",
      "Bash(gh issue list:*)",
      "Bash(gh issue comment:*)",
      "Bash(gh issue create:*)",
      "Bash(gh issue edit:*)",
      "Bash(gh api:*)",
      "Bash(gh run view:*)",
      "Bash(gh run list:*)",

      "//====================================================================",
      "// Filesystem — basic read-only inspection",
      "//====================================================================",
      "Bash(ls:*)",
      "Bash(find:*)",
      "Bash(mkdir:*)",
      "Bash(mkdir -p:*)",
      "Bash(touch:*)",
      "Bash(cat:*)",

      "//====================================================================",
      "// Web fetches (agents often need to cross-reference specs + deps)",
      "//====================================================================",
      "WebFetch(domain:crates.io)",
      "WebFetch(domain:docs.rs)",
      "WebFetch(domain:lib.rs)",
      "WebFetch(domain:github.com)",
      "WebFetch(domain:raw.githubusercontent.com)",
      "WebFetch(domain:gist.githubusercontent.com)",
      "WebFetch(domain:api.github.com)",
      "WebFetch(domain:rust-lang.github.io)",
      "WebFetch(domain:doc.rust-lang.org)",
      "WebFetch(domain:users.rust-lang.org)",
      "WebFetch(domain:rustsec.org)",
      "WebFetch(domain:www.redb.org)",
      "WebFetch(domain:ietf.org)",
      "WebFetch(domain:www.rfc-editor.org)",
      "WebFetch(domain:twittner.gitlab.io)",
      "WebFetch(domain:developer.blockchaincommons.com)",
      "WebSearch",

      "//====================================================================",
      "// curl — for ad-hoc registry and metadata lookups",
      "//====================================================================",
      "Bash(curl -s https://crates.io/api/*)",
      "Bash(curl -s https://api.github.com/*)",
      "Bash(curl -s https://raw.githubusercontent.com/*)",
      "Bash(curl -s -o /dev/null -w:*)",
      "Bash(curl -s -L:*)",

      "//====================================================================",
      "// Misc",
      "//====================================================================",
      "Bash(which:*)",
      "Bash(python3 -c:*)",
      "Read(//Users/benwork/.claude/**)",
      "Read(//Users/benwork/Documents/thrum/.claude/**)"
    ],

    "deny": [
      "//====================================================================",
      "// Destructive git — orchestrator must run these explicitly",
      "//====================================================================",
      "Bash(git reset --hard*)",
      "Bash(git clean -f*)",
      "Bash(git clean -d*)",
      "Bash(git commit --amend*)",
      "Bash(git commit --no-verify*)",
      "Bash(git push --force*)",
      "Bash(git push -f *)",
      "Bash(git push origin main*)",
      "Bash(git push origin master*)",
      "Bash(git push --delete*)",
      "Bash(git branch -D*)",
      "Bash(git checkout --theirs*)",
      "Bash(git checkout --ours*)",

      "//====================================================================",
      "// Package/release publish — never from an agent",
      "//====================================================================",
      "Bash(cargo publish*)",
      "Bash(cargo yank*)",
      "Bash(npm publish*)",

      "//====================================================================",
      "// GitHub — repo-level destructive operations",
      "//====================================================================",
      "Bash(gh repo delete*)",
      "Bash(gh repo archive*)",
      "Bash(gh release create*)",
      "Bash(gh release delete*)",
      "Bash(gh secret:*)",
      "Bash(gh ssh-key:*)",

      "//====================================================================",
      "// Filesystem — broad destructive operations",
      "//====================================================================",
      "Bash(rm -rf /*)",
      "Bash(rm -rf ~/*)",
      "Bash(rm -rf .*)"
    ]
  }
}
```

## Notes on the deny list

- JSON doesn't actually support `//` comments; the heading comments above would be removed before applying. Keeping them in the draft for reviewability.
- `rm -rf` is allowed for local subdirectories (e.g., `rm -rf target/`) because restricting it too narrowly breaks legitimate cleanup. The denies target the dangerous forms that hit the filesystem root, home directory, or hidden files.
- `git rebase` and `git filter-branch` stay in allow. Rationale: both are history-rewriting but are bounded to the local branch unless combined with `push --force`. Blocking them would prevent legitimate cleanup. The push-force deny is the real safeguard.
- `git push origin:*` is allowed for feature branches but `git push origin main` is explicitly denied (main stays protected from direct subagent push).
- `gh pr edit` is allowed (Phase 1 agents may need to edit their own PR bodies in response to maintainer feedback). No denials on PR-level operations because they're reversible.

## Smoke-test plan before committing this

1. Apply the proposed file (replace current `settings.local.json`).
2. Dispatch one trivial subagent (`general-purpose` or `rust-engineer`) with a one-line task: "Run `git status`, `git log -1 --oneline`, and `cargo nextest run --workspace`. Report the output."
3. If the subagent executes all three without permission prompts, the config is working. If any command prompts, add the specific entry or refine the pattern and retest.
4. Dispatch a second trivial subagent: "Create a file at `/tmp/permissions-smoketest.txt` with the text 'ok', then `git status`." The write should succeed; git status should show the file as outside the repo. This confirms writes work.
5. If both smoke tests pass, commit the file (it's currently untracked — its presence on disk is what matters; committing it to the repo makes the permission baseline shareable with other contributors).

## What's NOT in this diff

- Browser automation (Playwright / Puppeteer): not needed in Phase 1.
- Network calls beyond GitHub / crates.io / rust docs: not needed.
- Docker / container operations: not needed (Benten is Rust-native, no container build in Phase 1).
- Database operations (psql, sqlite3): not needed (redb is file-based, no external DB).
- Direct file I/O to `~/.ssh`, `~/.aws`, `/etc/`, etc.: explicitly NOT added; any access would be a session-level prompt.

---

## Your approvals needed

1. **Approve the diff?** If yes, I apply by overwriting `.claude/settings.local.json` (the current file is still untracked, so the diff is a file-level replace).
2. **Approve committing the file to the repo?** Making it tracked means the baseline is visible to future contributors and auditable in `git log`. Alternatively, keep it untracked (project-local only, resets when cloned fresh).
3. **Run the smoke test after apply?** I'd dispatch one trivial subagent with a known-good task and report the result before declaring it done.
