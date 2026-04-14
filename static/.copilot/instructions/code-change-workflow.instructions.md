---
description: "Enforce a complete workflow for every code change request: mandatory planning, specialist evaluation from /agents, TODO generation with assignees (including testing and final checkup TODOs), reviewer-executed fixes, and atomic commits after each TODO. Applies unless the user explicitly opts out."
---

# Code Change Workflow

This rule is mandatory for every request that changes code unless the user explicitly says to skip this workflow.

Compatibility note: this file supersedes and merges the old `project-guidelines` instruction for code-change work.

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
2. If `workflow-orchestrator` is available under `/agents`, delegate high-level workflow coordination to it first.
3. Evaluate the request against specialists available under `/agents`.
4. Generate a TODO list assigning a specialist owner to each implementation TODO.
5. Generate testing TODOs (unit/integration/e2e as applicable) with specialist ownership.
6. Execute TODOs one at a time.
7. After each completed TODO, create an atomic commit containing only that TODO's changes.
8. Generate Final Checkup TODOs and execute them.
9. Run reviewer subagents for final checks.
10. Apply reviewer-requested fixes.
11. Run the mandatory Documentation Completion Step as the final pre-commit gate.
12. Create a final commit for final checkup and approvals.

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

## Mandatory code documentation gate

Code changes are not complete until documentation is complete.

For every code-change request, run a mandatory Documentation Completion Step as the final pre-commit gate.

Documentation Completion Step (required):

- Every new or modified function must be documented.
- Every new or modified structure, type, class, interface, and enum must be documented.
- Every new or modified non-trivial or complex code path must include intent-focused comments.
- Public APIs must include contract-level documentation (parameters, returns, errors, side effects).
- Internal logic must include maintenance-facing documentation where behavior is non-obvious.
- Documentation must be updated in the same change set as the code.

Documentation quality caveats (required):

- Documentation is mandatory, but low-value comments are not acceptable.
- Comments must explain intent, invariants, constraints, side effects, failure modes, and non-obvious decisions.
- Comments must not restate obvious syntax, variable names, or trivial control flow.

Commit restriction:

- The final commit is blocked until the Documentation Completion Step passes.

Specialist and skills requirement:

- Documentation verification and remediation must be delegated to `documentation-specialist`.
- Before closing Final Checkup TODOs or creating the final commit, the workflow must consult and apply all documentation skills below on relevant changed files:
  - `commenting-standards`
  - `documentation-best-practices`
  - `go-documentation-best-practices`
  - `javascript-documentation-best-practices`
  - `python-documentation-best-practices`
  - `rust-documentation-best-practices`
  - `typescript-documentation-best-practices`

Final Checkup TODOs must include these documentation checks:

- Run `documentation-specialist` review on all changed code.
- Verify every changed function has documentation appropriate to visibility and complexity.
- Verify every changed structure, type, class, interface, and enum has documentation.
- Verify complex logic comments capture intent, constraints, edge cases, and side effects.
- Verify API docs match current behavior (parameters, returns, errors, and examples where relevant).
- Verify stale or contradictory comments were corrected or removed.

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

- Shared local skills are embedded from `static/__shared__/skills` and installed under `~/.copilot/skills/<skill-id>/`.
- External skills are declared in `static/external-skills.toml` and installed under `~/.copilot/skills/<source-id>/`.
- Docker skills are sourced externally as individual ids (for example `docker-core-architecture`) rather than grouped static folders.
- Validate source mapping with: `grep '^id = "' static/external-skills.toml`.

## Stop condition

Task is complete only when:

- All implementation TODOs are done
- All testing TODOs are done
- All Final Checkup TODOs are done
- Mandatory Documentation Completion Step is done
- Atomic commits were created after each TODO
- Final checkup fixes are applied and committed
