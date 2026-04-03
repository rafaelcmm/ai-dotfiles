---
name: conventional-branches
description: Branch naming conventions that improve traceability, CI automation, and team collaboration.
allowed-tools: [Read]
---

# Conventional Branches

Adopt predictable branch names so purpose, ownership, and release intent are obvious from the branch alone.

## Purpose

- Make branch intent immediately clear
- Improve issue/PR traceability
- Support workflow automation in CI/CD
- Reduce naming conflicts and ambiguity

## When to Reference This Skill

Reference when:

- Creating feature/fix/release/hotfix branches
- Defining team Git workflow standards
- Configuring branch-based CI rules

## Recommended Naming Pattern

```text
<type>/<ticket-or-context>-<short-kebab-summary>
```

Examples:

```text
feat/RL-142-add-oauth-login
fix/RL-201-handle-null-user-profile
chore/update-eslint-config
docs/improve-contributing-guide
release/2026-03-31
hotfix/RL-233-payment-timeout
```

## Common Branch Types

| Type       | Use For                               |
| ---------- | ------------------------------------- |
| `feat`     | New features                          |
| `fix`      | Bug fixes                             |
| `chore`    | Tooling/config/dependency maintenance |
| `docs`     | Documentation work                    |
| `refactor` | Internal structure changes            |
| `release`  | Release preparation                   |
| `hotfix`   | Urgent production fixes               |

## Naming Rules

- Use lowercase kebab-case segments
- Keep names short but descriptive
- Include issue ID when available
- Avoid personal names and ambiguous labels (`test`, `new`, `misc`)
- Avoid long-lived shared branches besides protected branches

## Protected Branch Model (Suggested)

- `main`: production-ready, protected
- `develop` (optional): integration branch for teams using Git Flow
- short-lived topic branches: merged and deleted after PR

## Quick Checklist

```text
- [ ] Branch type matches intent
- [ ] Issue/ticket ID included when applicable
- [ ] Name is kebab-case and concise
- [ ] Branch is scoped to one concern
- [ ] PR target branch follows team workflow
```
