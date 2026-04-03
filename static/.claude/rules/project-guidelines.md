---
description: "Core project standards for architecture, build/test, conventions, and security across this repository."
---

# Project Guidelines

Compatibility note: top-level template instruction files were removed. Platform guidance is now defined through instruction/rule trees under `.claude/rules`.

## Architecture

- Prefer hexagonal architecture for service boundaries and keep domain logic independent from infrastructure.
- Use design patterns deliberately, not accidentally. Choose the simplest pattern set that solves the real problem.
- Follow clean code principles: readable, maintainable, and intention-revealing.

## Build and Test

- Run `npm install` before building.
- Run `npm run lint` to check code quality.
- Run Prettier before commit: `npx prettier --check .`.
- Ensure all changes are committed before marking tasks complete.

## Conventions

- Use conventional commits.
- Use conventional branch naming.
- Delegate to specialist agents for domain-specific work.
- Before completing any task, run a single final-pass validation with `security-reviewer` and `documentation-specialist`.
- Reviewer subagents must return findings and concrete remediation steps (file edits and commands) in the same pass so the main thread can apply fixes without repeated report-only loops.

## Security

- Follow OWASP Top 10 guidelines. Never hardcode secrets.
- Validate and sanitize all user input at system boundaries.
- Use parameterized queries, not string concatenation for SQL.
- Apply defense in depth and least privilege principles.

## Hooks

Task-completion gates enforce one-shot validation before task completion:

1. Security review gate delegates once to `security-reviewer` and requires actionable fixes with each finding.
2. Documentation review gate delegates once to `documentation-specialist` and requires actionable fixes with each finding.
3. Lint and format check runs `npm run lint` and `npx prettier --check .`.
4. Commit check ensures no uncommitted changes remain.

Run these reviewer gates after the final code change for the task. If new edits are introduced after review, run one final-pass review again before completion.

## Skill Link Topology

- In `packages/platform-config/.claude/skills`, most entries link via `../../static/skills`.
- Docker grouped entries (`docker-agents`, `docker-core`, `docker-errors`, `docker-impl`, `docker-syntax`) link to grouped trees under `../../.github/skills`.
- Validate link integrity with: `find packages/platform-config/.claude/skills -xtype l -print`.
