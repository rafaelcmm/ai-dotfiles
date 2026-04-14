---
name: commenting-standards
description: Language-agnostic commenting and documentation standards. Mandates docblocks for all public constructs and inline comments for non-obvious logic. Comments explain WHY, not WHAT. Applies to any language or paradigm.
allowed-tools: [Read]
---

# Commenting Standards

Write code that is readable — then document it anyway.

Self-documenting code is a goal, not a guarantee. Comments remain indispensable for
communicating intent, constraints, assumptions, architectural obligations, and domain
context. These obligations are not optional and apply regardless of language.

## The Core Rule

**Comments must explain WHY the code does what it does**, not narrate WHAT it is doing.

Code explains mechanics. Comments explain meaning. Only the latter prevents
misinterpretation. The question is not whether code _can_ be understood without
comments; it is whether future readers — including your future self — should be forced
to reverse-engineer your thought process.

---

## Two Kinds of Comments

### Docblocks

Docblocks document the **contract** of a construct. They are **semantic** and
**interface-level** — describing what must remain true even if the implementation
changes.

**Required on:** all classes, modules, functions, methods, getters, setters, and
public properties.

Docblocks must answer:

- What guarantees does this construct provide to its callers?
- What assumptions does it rely upon?
- What do parameters and return values semantically mean (beyond their type)?
- What invariants or conditions must be preserved?
- How and when should this construct be used within the broader system?

### Inline Comments

Inline comments explain **implementation decisions** — the non-obvious reasoning
inside the body of a construct.

**Required when:** logic is non-trivial, interactions with external systems exist,
domain constraints motivate specific behavior, or side effects are not obvious from
syntax alone.

Inline comments must explain:

- Non-trivial control flow — why this branch or exit condition exists
- Domain rules and invariants the surrounding code enforces
- Compensating behaviors for system limitations, with a tracking reference if applicable
- Why a specific approach was chosen over obvious alternatives

> **Private members:** docblocks on private members may combine contract and
> implementation detail, since private members are themselves implementation details.

---

## Quality Checklist: Docblocks

Before finalizing a docblock, verify it:

- [ ] States the construct's **role in the broader system**, not just what it does
- [ ] Documents every non-trivial **parameter** — meaning, constraints, edge cases
- [ ] Documents the **return value** semantically, not just as a type echo
- [ ] States **behavioral guarantees** callers can rely on
- [ ] States **failure modes** — how errors, nulls, or empty states are communicated
- [ ] Notes any **concurrency, ordering, or lifecycle constraints**
- [ ] Does not restate the symbol name as the only content

## Quality Checklist: Inline Comments

Before finalizing inline comments, verify they:

- [ ] Explain **why** this approach was chosen, not what the code does
- [ ] State **domain invariants** the surrounding code must preserve
- [ ] Justify **non-obvious branching** or early-return decisions
- [ ] Reference an issue or ticket for known **workarounds**
- [ ] Are absent on trivially obvious statements

---

## Anti-Patterns

| Anti-pattern | Example | Why it fails |
|---|---|---|
| Name restatement | `// Saves the term` before `saveTermToHistory()` | Adds no information beyond the code |
| Vacuous parameter doc | `@param term — The term.` | Provides no semantic context |
| Empty class header | `/** SearchBox view model */` | Omits role, responsibilities, and lifecycle |
| Missing field semantics | Field with no comment | Reader cannot know what it represents or when it changes |
| No behavioral guarantees | Method docblock with only param/return info | Callers cannot reason about error handling or ordering |
| Undocumented control flow | Filter + reinsert with no explanation | The invariant being enforced is invisible |
| Comment narrating syntax | `// Increment i by 1` before `i++` | Restates the obvious; zero value added |

---

## Patterns to Follow

### Class / Module level

State:

- The construct's role in the broader system or architecture
- Its primary responsibilities (use a short bulleted list for multiple)
- Lifecycle, ownership, or initialization constraints when relevant

### Method / Function level

State:

- The behavioral contract — what it guarantees on success
- Its error handling strategy — when and how it signals failure
- Concurrency safety, idempotency, or ordering requirements when relevant
- Constraints on parameters beyond their type annotations

### Property / Field level

State:

- What the value semantically represents
- When and why it changes
- How external consumers (views, services, callers) should interpret it
- Whether it is persisted, reactive, or ephemeral

### Inline (inside implementation bodies)

State:

- Why a specific branch, filter, or sort order exists
- What invariant is being enforced
- Why a particular strategy was chosen over obvious alternatives

---

## Examples

### Bad: vacuous class docblock

```
/** SearchBox view model */
class SearchBoxVM { ... }
```

This tells the reader nothing beyond the name. It omits architectural role,
responsibilities, and lifecycle expectations. A reader still cannot understand
what this class owns, what it coordinates, or when it is valid to use.

### Good: meaningful class docblock

```
/**
 * SearchBoxVM is the view-model behind the global search box in the header.
 *
 * Responsible for:
 *  - Managing the current search term and loading state.
 *  - Fetching product suggestions with debouncing to avoid excessive API calls.
 *  - Maintaining a bounded, de-duplicated search history persisted in
 *    local storage so that the view can present recent terms reactively.
 */
class SearchBoxVM { ... }
```

---

### Bad: parameter docblock that restates the name

```
/**
 * Saves a search term to the history.
 * @param term — The term.
 */
```

"The term" conveys no context. What constitutes a valid term? What happens when it is
empty? Is it trimmed? Does it overwrite or deduplicate existing entries?

### Good: parameter docblock with real context

```
/**
 * Saves the current or provided search term into history, enforcing
 * uniqueness and a configurable maximum length. Most recent terms appear first.
 *
 * @param term — Optional explicit search string to save. When omitted, the
 *               currently active search term is used. The value is trimmed
 *               before saving; empty or whitespace-only values are ignored.
 */
```

---

### Bad: inline comment narrating mechanics

```
// Remove duplicates and add to front
history.filter(t => t !== term).unshift(term)
```

This describes what the code does, which is already readable. It does not explain
_why_ duplicates are removed or what ordering invariant is being maintained.

### Good: inline comment explaining intent and invariant

```
// Remove the existing occurrence before re-inserting at the front.
// This preserves recency ordering without creating duplicate entries.
history = history.filter(t => t !== term)
history.unshift(term)

// Enforce the configured maximum size so the list stays bounded.
history = history.slice(0, maxEntries)
```

---

### Bad: field with no documentation

```
private isLoading = false
```

The reader must trace all mutation sites to understand what triggers this, what
resets it, and how the view is supposed to consume it.

### Good: field with semantic documentation

```
/**
 * Indicates whether a search request is currently in flight.
 * The view binds to this value to render loading indicators and
 * disable input while a response is pending.
 */
private isLoading = false
```

---

## Closing Principles

- **Comments exist to preserve intent.** Code expresses behavior; comments express
  rationale.
- **Comments are not optional.** They are part of the deliverable software asset.
- **"Self-documenting code" is a myth** when used as a justification for omitting
  comments. At scale, it collapses under its own complexity.
- **Future maintainers should never need to reverse-engineer your thinking.**
  Comments are a structural component of maintainable software; adherence is
  expected on every construct written or modified.
