---
name: documentation-best-practices
description: High-signal technical documentation standards for clarity, discoverability, and long-term maintainability.
allowed-tools: [Read]
---

# Documentation Best Practices

Write documentation that helps readers complete tasks quickly and understand decisions with minimal ambiguity.

## Purpose

- Improve onboarding speed and knowledge transfer
- Reduce repeated support questions
- Keep architecture and decisions transparent
- Enable reliable maintenance over time

## When to Reference This Skill

Reference when:

- Writing READMEs, runbooks, or architecture docs
- Documenting APIs and workflows
- Capturing decisions and tradeoffs
- Reviewing documentation quality in PRs

## Documentation Types

| Type         | Primary Question Answered              |
| ------------ | -------------------------------------- |
| Tutorial     | How do I learn this from zero?         |
| How-to Guide | How do I accomplish a specific task?   |
| Reference    | What are the exact interfaces/options? |
| Explanation  | Why is it designed this way?           |

## Writing Standards

- Start with user goal and prerequisites
- Prefer task-first structure with examples
- Use precise terminology consistently
- Keep examples copy-paste ready and tested
- Separate facts from recommendations clearly

## Structure Template

```text
Title
Summary (1-2 lines)
Prerequisites
Steps / Usage
Validation (how to confirm success)
Troubleshooting
Related links
```

## Maintenance Rules

- Co-locate docs with code ownership when possible
- Update docs in the same PR as behavior changes
- Mark version compatibility explicitly
- Remove stale sections instead of preserving outdated paths
- Use decision records (ADRs) for significant architecture changes

## Quick Checklist

```text
- [ ] Audience and goal are explicit
- [ ] Steps are sequential and verifiable
- [ ] Commands/examples are accurate
- [ ] Edge cases and failures are covered
- [ ] Last-updated context is clear
```
