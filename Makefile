# Benten Engine — convenience targets.
#
# The `pre-push` target is **discipline-only convenience** — NOT a git hook.
# Phase-3 R6-R6-final + Phase-4-Foundation R1-triage Q5 ratification: the
# 5 workspace pre-push checks remain orchestrator discipline. This target
# exists so contributors can opt in with a single command rather than
# memorizing the list. Per `.addl/dispatch-conventions.md §3.5h`.
#
# Usage: `make pre-push` before pushing to GitHub or merging. Runs:
#   1. cargo fmt --check
#   2. cargo doc --workspace --no-deps
#   3. cargo clippy --workspace --all-targets ... -D warnings
#   4. cite-drift-detector (informational; orchestrator audits output)
#   5. codegen-regen (conditional: only if crates/benten-errors/src/lib.rs
#      changed since last commit)

.PHONY: pre-push pre-push-fmt pre-push-doc pre-push-clippy pre-push-cite-drift pre-push-codegen-conditional

pre-push: pre-push-fmt pre-push-doc pre-push-clippy pre-push-cite-drift pre-push-codegen-conditional
	@echo ""
	@echo "pre-push checks complete. Review cite-drift output before pushing."

pre-push-fmt:
	@echo "==> cargo fmt --check"
	cargo fmt --check

pre-push-doc:
	@echo "==> cargo doc --workspace --no-deps"
	cargo doc --workspace --no-deps

pre-push-clippy:
	@echo "==> cargo clippy --workspace --all-targets ..."
	cargo clippy --workspace --all-targets \
		--features benten-eval/testing,benten-engine/test-helpers,benten-graph/testing \
		-- -D warnings

pre-push-cite-drift:
	@echo "==> cite-drift-detector (informational)"
	@cargo run -p cite-drift-detector -- . --all || true

pre-push-codegen-conditional:
	@if git diff --name-only HEAD~1 2>/dev/null | grep -q "crates/benten-errors/src/lib.rs"; then \
		echo "==> codegen-regen (benten-errors changed)"; \
		npm run codegen:errors; \
	else \
		echo "==> codegen-regen skipped (benten-errors unchanged)"; \
	fi
