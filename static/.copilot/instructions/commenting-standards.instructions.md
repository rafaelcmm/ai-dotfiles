---
description: "Load when writing, reviewing, or modifying any code in any language. Enforces docblock and inline comment standards. Comments explain WHY — not WHAT. Applies to every construct written or changed."
---

# Commenting Standards

This instruction is mandatory whenever writing, reviewing, or modifying code.

**Skill path:** `skills/commenting-standards/SKILL.md`

Load `commenting-standards` before producing or reviewing any code output.

## Activation

Apply to every request that:

- Writes new code (any language, any paradigm)
- Modifies existing code
- Reviews code for quality or correctness
- Adds, changes, or removes any function, method, class, module, type, or property

## Core mandate

### Docblocks

Every class, module, function, method, getter, setter, and public property must have a
docblock. A docblock documents the **contract** — what the construct guarantees,
requires, and communicates.

A compliant docblock:

- States the construct's role in the broader system (not just what it does)
- Documents parameters with their semantic meaning, constraints, and edge cases —
  never just their type
- States behavioral guarantees callers can rely on
- States failure modes — how errors, nulls, or empty states are communicated
- Notes concurrency, ordering, or lifecycle constraints when relevant

### Inline comments

Inline comments are required on any implementation logic that is non-trivial,
domain-driven, or whose intent is not obvious from the syntax alone.

A compliant inline comment:

- Explains **why** — not what the code does
- States invariants the surrounding code must preserve
- Justifies non-obvious branches or early exits
- References an issue or ticket number when documenting a known workaround

## Prohibited patterns

- Docblocks that only restate the symbol name: `/** Saves term. */`
- Parameter descriptions that echo the name: `@param term — The term.`
- Inline comments that narrate syntax: `// increments i`
- Undocumented public properties or fields
- Methods with no docblock, regardless of perceived obviousness

## Enforcement

- Every new or modified construct must pass both checklists from the
  `commenting-standards` skill before marking it complete.
- Missing or low-value comments are a blocking defect, not a style suggestion.
- "Self-documenting code" is not an acceptable reason to omit documentation.
