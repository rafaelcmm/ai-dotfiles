---
name: typescript-documentation-best-practices
description: >
  Comprehensive guide for documenting TypeScript code using TSDoc, TypeDoc, and
  structured doc comments. Use this skill whenever writing, reviewing, or improving
  TypeScript documentation — including TSDoc blocks, @param/@returns/@remarks tags,
  interface and type documentation, generic type documentation, decorators, module
  comments, and API surface documentation. Trigger on any .ts or .tsx file that
  needs documentation added, improved, or reviewed — including libraries, APIs,
  React components, Angular services, NestJS modules, and any TypeScript codebase.
---

# TypeScript Documentation Best Practices

## Philosophy

TypeScript's type system already documents *shapes* — so documentation should focus on
**semantics, constraints, side-effects, and intent** rather than repeating type info.
Use TSDoc (the Microsoft standard) rather than raw JSDoc to get proper tooling support
across TypeDoc, API Extractor, VS Code, and ESLint.

**Core rule:** Every exported symbol must have a TSDoc comment. Leverage types to avoid
restating what the compiler already enforces — document *why* and *how to use it safely*.

---

## Scope and restraint

Canonical policy: `commenting-standards`. This section applies it to TSDoc.

### Coverage is absolute, length is proportional

Every exported symbol keeps its TSDoc comment — the Core rule above is not
relaxed. What scales is depth, never presence. A self-evident signature is fully
documented by one assertive sentence; depth is owed in proportion to visibility,
subtlety, and how far a mistake travels. A public API whose misuse corrupts data
owes the full contract plus the specific way callers get it wrong.

Padding is a defect, not thoroughness. A `@remarks` block restating the summary,
or a twelve-line docblock on a two-line type guard, buries the line that mattered.

### TSDoc tags are required, and economy never removes one

`@param`, `@returns`, `@throws`, `@remarks`, `@example`, `@defaultValue`,
`@deprecated` and `@typeParam` are part of the contract. VS Code hovers, `tsc`,
TypeDoc, API Extractor and every generated reference consume them. They are
**required on exported symbols**.

Brevity applies to a tag's **description**, never to its **presence**. Dropping a
tag to shorten a docblock removes a machine-readable fact and is a defect.

This reconciles with the Philosophy's "leverage types to avoid restating them":
TSDoc omits types by design, so a `@param` description must not repeat the
TypeScript annotation. It carries what the type cannot express — units, valid
ranges, edge values, ownership, what an empty value means. The tag appears; the
type does not.

```diff
-@param amount - The amount, as a number.
-@returns A number.
+@param amount - Major currency units. Fractional values are rounded, not truncated.
+@returns Integer micros, as the API requires.
```

### Document the present

A TSDoc comment states what the symbol does now; it is not a change log. `git
log`, `git blame` and the pull request own history, keep it accurate for free,
and attribute it. Never write provenance (`Extracted from UserService when...`),
removed behaviour (`This used to batch requests`), dated claims (`Until
2026-09-17 the opposite applied`), supersession notices, or plan archaeology.
Such a comment goes wrong at the next edit, and it is the one kind of staleness a
reader cannot detect, because nothing in the file contradicts it.

Four things legitimately carry time or context, each being present and actionable:
a `TODO`/`FIXME` with a tracking reference; a workaround with its upstream issue
and the condition that retires it; `@deprecated` naming the replacement, which is
TSDoc's own mechanism for exactly this; and an invariant with its expiry trigger.

### Assertive voice and the stop rule

Lead with one declarative sentence; present tense, active voice, indicative mood.
Cut hedges (`basically`, `it is important to note`) and self-reference (`This
function...` says nothing a reader cannot see).

A docblock is finished when a competent caller can use the symbol correctly
without reading its body. Implementation reasoning belongs in an inline comment
at the line it explains, design rationale in a design doc or ADR, history nowhere.

### Before and after

```ts
/**
 * This function is basically a utility which is used in order to parse a
 * duration string into milliseconds.
 *
 * @remarks
 * It is important to note that it returns a number. Originally this lived in
 * `http-client.ts`, but it was extracted during the Q3 refactor when the retry
 * logic needed it too. Previously it did not accept the `h` unit.
 *
 * @param input - The input, which is a string.
 * @returns A number, which is the milliseconds.
 */
export function parseDurationMs(input: string): number;
```

```ts
/**
 * Parses a duration string such as `"30s"` or `"5m"` into milliseconds.
 *
 * @param input - A positive integer followed by `ms`, `s`, `m` or `h`.
 *   Whitespace is not tolerated.
 * @returns Whole milliseconds, rounded up — a sub-millisecond duration becomes
 *   `1` rather than `0`, so a caller cannot build a zero-delay retry loop.
 * @throws {RangeError} If the unit is unrecognized or the value is not a
 *   positive integer.
 */
export function parseDurationMs(input: string): number;
```

Shorter because the padding left, not because information did. Every tag
survived; `@returns` gained the fact a caller actually needs.

---


## Documenting Constructs

TSDoc vs JSDoc, comment anatomy, and worked examples for types, interfaces,
enums, type aliases, classes, generics, and React (TSX) components are in
[references/documenting-constructs.md](references/documenting-constructs.md).

## Inline Comments in TypeScript

### Comment the WHY, not the type (the type is already there)

```ts
// ✅ Good — explains a constraint the type doesn't capture
// Offset must be page-aligned; the kernel rejects unaligned mmap() calls
const offset = Math.floor(rawOffset / PAGE_SIZE) * PAGE_SIZE;

// ✅ Good — explains a non-obvious library behavior
// Prisma silently ignores `undefined` fields in updates but treats `null` as a
// deliberate SET NULL. Always use undefined for "don't change this field".
await db.user.update({ where: { id }, data: { avatar: undefined } });

// ❌ Bad — restates what TypeScript already shows you
const user: User = getUser(); // user is of type User
```

### Document type assertion reasons

```ts
// We checked for null two lines above; TypeScript's control flow loses the
// narrowing across the async boundary — safe to assert here
const config = cachedConfig as AppConfig;
```

### Compiler directives with explanation

```ts
// @ts-ignore — third-party @types/foo is missing this overload (issue #123)
// Remove when @types/foo > 3.2.0 is available.
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
const result = legacyLib.obscureMethod(args);
```

---


## Advanced Topics

Release tags, class inheritance documentation, module/file documentation, and
TypeDoc configuration are in
[references/advanced-topics.md](references/advanced-topics.md).

## Anti-Patterns

```ts
// ❌ Repeating type info that TypeScript already provides
/**
 * @param name - A string parameter.          ← type already in signature
 * @returns string                             ← return type already in signature
 */
function greet(name: string): string { ... }

// ✅ Correct — describe meaning, not type
/**
 * @param name - The user's display name (1–50 chars, no HTML allowed).
 * @returns A localized greeting string for the user's locale.
 */
function greet(name: string): string { ... }

// ❌ Single-line format for exported symbols
/** Does the thing. */ export function doThing() { ... }

// ✅ Correct — always multi-line for exports
/**
 * Does the thing.
 */
export function doThing() { ... }

// ❌ Missing @typeParam for generic functions
/**
 * Wraps a value in a container.
 */
function wrap<T>(value: T): Container<T> { ... }

// ✅ Correct
/**
 * Wraps a value in an immutable container.
 *
 * @typeParam T - The type of the wrapped value.
 */
function wrap<T>(value: T): Container<T> { ... }
```

---

## Documentation Checklist

- [ ] Every exported symbol has a TSDoc block with a summary line
- [ ] `@remarks` used for multi-paragraph or complex explanations
- [ ] `@param` documents semantics/constraints (not the type — TS handles that)
- [ ] `@returns` describes what the value *means*, not just its type
- [ ] `@throws` documents every error that can propagate
- [ ] `@typeParam` present for every generic type parameter
- [ ] `@example` with fenced ` ```ts ` blocks for public APIs
- [ ] `@deprecated` includes migration path and removal version
- [ ] `@since` and release tags (`@public`/`@beta`/`@internal`) on public APIs
- [ ] Interface properties have inline `/** */` doc comments
- [ ] Enum members are individually documented
- [ ] `@override` / `@virtual` on class hierarchy methods
- [ ] `@fileoverview` at top of every module file
- [ ] No type annotations in `@param` (redundant in TypeScript)
- [ ] `@ts-ignore` / `@ts-expect-error` always has an explanation comment
