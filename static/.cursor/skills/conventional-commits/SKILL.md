---
name: conventional-commits
description: Standardized commit message format using Conventional Commits for clear history, automation, and semantic versioning.
allowed-tools: [Read]
---

# Conventional Commits

Use a predictable commit format to improve readability, changelog generation, release automation, and code review context.

## Purpose

- Keep commit history understandable and searchable
- Enable automatic semantic versioning
- Generate reliable release notes
- Communicate intent and impact quickly

## When to Reference This Skill

Reference when:

- Writing commit messages
- Reviewing PR commit quality
- Designing release pipelines
- Migrating a team to structured commits

## Core Format

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Examples:

```text
feat(auth): add OAuth login flow
fix(api): handle empty payload in webhook parser
docs(readme): clarify local setup steps
refactor(core): split validation logic by domain
test(users): add integration tests for create endpoint
chore(ci): cache pnpm store in pipeline
```

## Allowed Types (Recommended)

| Type       | Use For                                  |
| ---------- | ---------------------------------------- |
| `feat`     | New user-facing or API behavior          |
| `fix`      | Bug fixes                                |
| `docs`     | Documentation-only changes               |
| `refactor` | Code changes without behavior change     |
| `test`     | New or updated tests                     |
| `chore`    | Tooling, deps, or maintenance            |
| `perf`     | Performance improvements                 |
| `build`    | Build system or dependency build changes |
| `ci`       | CI/CD configuration changes              |

## Breaking Changes

Use either method:

1. `!` after type/scope
2. `BREAKING CHANGE:` footer

```text
feat(api)!: remove v1 orders endpoint

BREAKING CHANGE: /v1/orders was removed; use /v2/orders
```

## Message Quality Rules

- Use imperative, present tense (`add`, `fix`, not `added`)
- Keep subject concise and specific
- Start subject lowercase (unless proper noun)
- Avoid trailing period in subject
- Put rationale/impact in body when needed

## Quick Checklist

```text
- [ ] Type is correct for the change
- [ ] Scope is meaningful (if used)
- [ ] Subject explains what changed
- [ ] Breaking change is explicitly marked
- [ ] Body explains why when context is non-obvious
```

## Team Conventions (Suggested)

- Enforce with commit linting in CI
- Keep scopes aligned with package/app names
- Squash or clean noisy WIP commits before merge
- Use one concern per commit when possible
