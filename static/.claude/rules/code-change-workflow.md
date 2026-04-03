---
name: code-change-workflow
description: Enforce a complete workflow for every code change request: mandatory planning, specialist evaluation from /agents, TODO generation with assignees (including testing and final checkup TODOs), reviewer-executed fixes, and atomic commits after each TODO. Applies unless the user explicitly opts out.
---

# Code Change Workflow

This rule is mandatory for every request that changes code unless the user explicitly says to skip this workflow.

Compatibility note: this file supersedes and merges the old `project-guidelines` rule for code-change work.

## Activation

Use this workflow whenever the request includes implementing, modifying, refactoring, fixing, removing, or generating code and configuration.

Do not skip unless the user explicitly asks not to use this workflow.

## Architecture standards

- Prefer hexagonal architecture for service boundaries and keep domain logic independent from infrastructure.
- Use design patterns deliberately, not accidentally. Choose the simplest pattern set that solves the real problem.
- Follow clean code principles: readable, maintainable, and intention-revealing.

## Build and test standards

- Run `npm install` before building.
- Run `npm run lint` to check code quality.
- Run `npx prettier --check .` before completion.
- Ensure all changes are committed before marking tasks complete.

## Security standards

- Follow OWASP Top 10 guidelines. Never hardcode secrets.
- Validate and sanitize all user input at system boundaries.
- Use parameterized queries, not string concatenation for SQL.
- Apply defense in depth and least privilege principles.

## Required sequence

1. Plan first, then change code.
2. Evaluate the request against specialists available under `/agents`.
3. Generate a TODO list assigning a specialist owner to each implementation TODO.
4. Generate testing TODOs (unit/integration/e2e as applicable) with specialist ownership.
5. Execute TODOs one at a time.
6. After each completed TODO, create an atomic commit containing only that TODO's changes.
7. Generate Final Checkup TODOs and execute them.
8. Run reviewer subagents for final checks.
9. Apply reviewer-requested fixes.
10. Create a final commit for final checkup and approvals.

## Conventions

- Use conventional commits.
- Use conventional branch naming.
- Delegate to specialist agents for domain-specific work.

## Specialist evaluation requirements

Before writing code:

- Identify required specialists from `/agents` based on the request.
- Delegate analysis to relevant specialists for planning and TODO decomposition.
- Every implementation TODO must include:
  - clear scope
  - assigned specialist
  - expected output
  - acceptance criteria

## TODO requirements

For every code-change request, TODOs are mandatory and must include:

- Implementation TODOs (feature/fix/refactor tasks)
- Testing TODOs (must never be omitted)
- Final Checkup TODOs

Final Checkup TODOs must include at least:

- Security review
- Performance review (when performance can be affected)
- Documentation review
- Lint/format/build checks relevant to the repo

Final Checkup TODOs must run near completion, not after every code change. If new edits are introduced after final reviews, rerun one final-pass review.

## Atomic commit requirements

After each completed TODO:

- Commit immediately (atomic commit)
- Include only files related to that TODO
- Use a conventional commit message describing that specific TODO

Do not batch multiple TODOs into one commit.

## Reviewer execution requirements

When running reviewer subagents for Final Checkup TODOs:

- Reviewer subagents must provide actionable remediation.
- Reviewer subagents are responsible for performing or returning exact changes needed to resolve their findings in the same pass.
- Do not close the TODO until requested fixes are applied and verified.

## Skill link topology

- In `packages/platform-config/.claude/skills`, most entries link via `../../static/skills`.
- Docker grouped entries (`docker-agents`, `docker-core`, `docker-errors`, `docker-impl`, `docker-syntax`) link to grouped trees under `../../.github/skills`.
- Validate link integrity with: `find packages/platform-config/.claude/skills -xtype l -print`.

## Stop condition

Task is complete only when:

- All implementation TODOs are done
- All testing TODOs are done
- All Final Checkup TODOs are done
- Atomic commits were created after each TODO
- Final checkup fixes are applied and committed
