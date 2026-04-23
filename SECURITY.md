# Security Policy

## Supported versions

Benten Engine is in active pre-release development as of 2026-04. Only the latest commit on `main` receives security updates. No versioned releases have shipped yet.

## Reporting a vulnerability

**Do not open a public GitHub issue for security-sensitive findings.**

Email security reports to **ben@benten.ai** with:

- A description of the vulnerability
- Steps to reproduce (ideally a minimal proof-of-concept)
- Your assessment of impact + severity
- Any disclosure timeline constraints on your side

We aim to acknowledge reports within 72 hours and to land a mitigation or documented workaround within 14 days for critical-severity findings. For lower-severity findings, we will work with you on a mutually-agreed timeline.

## Disclosure

Once a fix lands, we will publish:

- A GitHub Security Advisory with the affected commit range + CVE if applicable
- A changelog entry noting the fix
- Credit for the reporter (with permission)

## Scope

This policy covers the `benten-engine` repository and its published crates / npm packages. Third-party dependencies are out of scope — report those upstream.

## Out of scope

- Findings that require an attacker to already have root / filesystem access to the host machine
- Theoretical issues without a demonstrated exploit path
- Denial-of-service via pathological inputs to the TypeScript DSL (Phase 1's threat model is embedded/single-process — the process is trusted)
- Documented known limitations (see the release notes and changelog entries for the current commit); new findings that sharpen the scope of a known limitation ARE in scope
