---
name: clean-code
description: Practical clean code principles for readability, maintainability, and safe refactoring.
allowed-tools: [Read]
---

# Clean Code

Write code that is easy to understand, easy to change, and hard to misuse.

## Purpose

- Reduce cognitive load for maintainers
- Prevent defect-prone complexity
- Improve testability and refactor safety
- Keep design aligned with domain language

## When to Reference This Skill

Reference when:

- Writing new features
- Refactoring legacy modules
- Reviewing pull requests for maintainability
- Defining team coding standards

## Core Principles

1. **Meaningful names**: encode intent, not implementation detail
2. **Small focused units**: one function/class, one responsibility
3. **Readable flow**: reduce nesting and hidden side effects
4. **Clear boundaries**: isolate I/O, domain logic, and frameworks
5. **Continuous improvement**: leave code cleaner than found

## Code Smells to Watch

- Long functions with multiple responsibilities
- Deep nesting and boolean flag combinations
- Repeated logic across modules
- Primitive obsession for rich domain concepts
- Leaky abstractions and unclear ownership

## Refactoring Heuristics

- Prefer early returns over nested conditionals
- Extract intent-revealing helper functions
- Replace magic values with named constants/types
- Encapsulate invariants in constructors/factories
- Delete dead code instead of commenting it out

## Testing Alignment

- Tests should describe behavior, not implementation details
- Keep deterministic tests with explicit setup
- Use test names in business language
- Mock only external boundaries, not core logic

## PR Review Checklist

```text
- [ ] Names are domain-meaningful
- [ ] Functions have a single clear purpose
- [ ] Error handling is explicit and actionable
- [ ] Duplication is removed or intentionally accepted
- [ ] Tests cover critical behavior and edge cases
```

## Practical Rule of Thumb

If a future maintainer cannot explain what a unit does in one sentence, simplify it.
