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

## TSDoc vs JSDoc in TypeScript

TypeScript natively understands **TSDoc** — a standardized superset of JSDoc designed
for TypeScript tooling. Key differences:

| Aspect | JSDoc | TSDoc |
|--------|-------|-------|
| Type annotations in params | `{string}` in `@param` | Omit — TS types handle this |
| Cross-references | `{@link Foo}` | `{@link Foo}` (same) |
| Extended description | Not standardized | `@remarks` block |
| Release status | None | `@public`, `@beta`, `@alpha`, `@internal` |
| Override markers | None | `@virtual`, `@override`, `@sealed` |
| Summary separation | First paragraph | Up to first blank line |

---

## Comment Structure and Anatomy

A TSDoc comment has three distinct sections:

```ts
/**
 * [SUMMARY] — Brief one-sentence description. Shown in hover tooltips and
 * module-level listings. Keep this to one line.
 *
 * [REMARKS — optional] Start with @remarks. May be multi-paragraph and lengthy.
 * Does not repeat the summary. Use for caveats, algorithmic detail, and context.
 *
 * @remarks
 * This method uses a two-phase commit strategy internally. The first phase
 * acquires a distributed lock; the second phase persists the transaction.
 * If the process crashes between phases, a recovery job handles cleanup.
 *
 * @param userId - The UUID of the user to update. Must already exist.
 * @param patch - Partial update payload. Unset fields are left unchanged.
 * @returns A promise resolving to the updated user entity.
 * @throws {NotFoundError} If no user exists with the given `userId`.
 * @throws {ValidationError} If `patch` fails schema validation.
 *
 * @example
 * ```ts
 * const updated = await updateUser('uuid-123', { name: 'Alice' });
 * console.log(updated.name); // 'Alice'
 * ```
 *
 * @since 2.0.0
 * @public
 */
async function updateUser(userId: string, patch: Partial<User>): Promise<User> { ... }
```

---

## Documenting Types and Interfaces

Every property in an exported interface or type must be documented:

```ts
/**
 * Configuration for the HTTP retry policy.
 *
 * @remarks
 * Retries use exponential backoff with jitter. The total wait time between
 * the first attempt and the last retry is bounded by `maxDelay * attempts`.
 */
export interface RetryConfig {
  /** Maximum number of retry attempts before the request fails. */
  maxAttempts: number;

  /**
   * Initial delay in milliseconds before the first retry.
   * Subsequent delays are doubled up to `maxDelay`.
   *
   * @defaultValue 200
   */
  initialDelay?: number;

  /**
   * Maximum delay cap in milliseconds between retries.
   *
   * @defaultValue 10000
   */
  maxDelay?: number;

  /**
   * HTTP status codes that should trigger a retry.
   * Defaults to `[429, 500, 502, 503, 504]`.
   */
  retryOn?: number[];

  /**
   * Optional callback invoked before each retry attempt.
   * Useful for logging or circuit-breaker logic.
   *
   * @param attempt - The current attempt number (1-indexed).
   * @param error - The error that triggered this retry.
   */
  onRetry?: (attempt: number, error: Error) => void;
}
```

### Documenting Enums

```ts
/**
 * Represents the lifecycle state of an async job.
 *
 * State transitions: `Pending → Running → (Completed | Failed | Cancelled)`
 */
export enum JobStatus {
  /** Job is queued but has not started yet. */
  Pending = 'pending',

  /** Job is actively being processed by a worker. */
  Running = 'running',

  /** Job finished successfully. Result is available. */
  Completed = 'completed',

  /**
   * Job encountered an unrecoverable error.
   * Check `job.error` for the failure reason.
   */
  Failed = 'failed',

  /** Job was manually cancelled before completion. */
  Cancelled = 'cancelled',
}
```

### Documenting Type Aliases

```ts
/**
 * A branded UUID string type to prevent accidental assignment of
 * plain strings to ID fields.
 *
 * @example
 * ```ts
 * function getUser(id: UserId): Promise<User> { ... }
 * getUser('not-a-uuid'); // TS error — string is not assignable to UserId
 * ```
 */
export type UserId = string & { readonly __brand: 'UserId' };

/**
 * Represents any value that can be serialized to JSON.
 * Use this instead of `any` when the exact shape is unknown.
 */
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue };
```

---

## Documenting Classes

```ts
/**
 * A strongly-typed event emitter with compile-time event name checking.
 *
 * @remarks
 * Unlike Node's EventEmitter, this class enforces that event names and
 * payload types are declared upfront via the generic `Events` type parameter.
 *
 * @typeParam Events - A record type mapping event names to their payload types.
 *
 * @example
 * ```ts
 * type AppEvents = {
 *   login: { userId: string; timestamp: Date };
 *   logout: { userId: string };
 * };
 *
 * const emitter = new TypedEmitter<AppEvents>();
 * emitter.on('login', ({ userId }) => console.log(userId)); // fully typed
 * emitter.emit('login', { userId: '123', timestamp: new Date() });
 * ```
 *
 * @public
 */
export class TypedEmitter<Events extends Record<string, unknown>> {
  /**
   * Registers a listener for the given event.
   *
   * @param event - The event name to listen for.
   * @param listener - Callback invoked with the event payload.
   * @returns `this` for chaining.
   */
  on<K extends keyof Events>(
    event: K,
    listener: (payload: Events[K]) => void,
  ): this { ... }

  /**
   * Emits an event, invoking all registered listeners synchronously.
   *
   * @remarks
   * Listeners are called in the order they were registered.
   * Errors thrown inside a listener propagate to the caller of `emit`.
   *
   * @param event - The event name to emit.
   * @param payload - The data passed to each listener.
   */
  emit<K extends keyof Events>(event: K, payload: Events[K]): void { ... }
}
```

---

## Documenting Generics

Always use `@typeParam` to explain what each type parameter represents:

```ts
/**
 * Wraps an async operation with automatic retry and timeout logic.
 *
 * @typeParam T - The resolved value type of the wrapped operation.
 * @typeParam E - The error type thrown on failure. Defaults to `Error`.
 *
 * @param operation - An async function to wrap. Must be idempotent.
 * @param config - Retry and timeout configuration.
 * @returns A promise that resolves to `T` or rejects with `E`.
 *
 * @example
 * ```ts
 * const result = await withRetry<User, ApiError>(
 *   () => api.getUser(id),
 *   { maxAttempts: 3, timeout: 5000 }
 * );
 * ```
 */
export async function withRetry<T, E extends Error = Error>(
  operation: () => Promise<T>,
  config: RetryConfig,
): Promise<T> { ... }
```

---

## Documenting React Components (TSX)

```tsx
/**
 * Props for the `DataTable` component.
 */
export interface DataTableProps<T> {
  /**
   * Array of row data to display.
   * An empty array renders the empty state (see `emptyMessage`).
   */
  rows: T[];

  /**
   * Column definitions controlling rendering and behavior.
   * Columns are displayed in array order.
   */
  columns: ColumnDef<T>[];

  /**
   * Message shown when `rows` is empty.
   *
   * @defaultValue `'No results found.'`
   */
  emptyMessage?: string;

  /**
   * Callback fired when the user clicks a row.
   * Not called for header or footer rows.
   */
  onRowClick?: (row: T, index: number) => void;
}

/**
 * A generic, typed data table with sorting, pagination, and empty-state handling.
 *
 * @typeParam T - The shape of a single row's data object.
 *
 * @remarks
 * Sorting is client-side only. For server-side sorting, handle `onSortChange`
 * and update `rows` externally.
 *
 * @example
 * ```tsx
 * <DataTable
 *   rows={users}
 *   columns={[
 *     { key: 'name', header: 'Name' },
 *     { key: 'email', header: 'Email' },
 *   ]}
 *   onRowClick={(user) => navigate(`/users/${user.id}`)}
 * />
 * ```
 */
export function DataTable<T>({ rows, columns, emptyMessage, onRowClick }: DataTableProps<T>) { ... }
```

---

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

## Release Tags and API Surface Documentation

Use TSDoc release tags to communicate stability:

```ts
/** @public */      // Stable, part of the public API — breaking changes require major version bump
/** @beta */        // Unstable, may change — consumers should expect breaking changes in minors
/** @alpha */       // Experimental — may be removed or changed at any time
/** @internal */    // Not part of the public API — do not use outside this package

/**
 * @deprecated Use `createSession()` instead. Will be removed in v5.0.
 * @public
 */
export function openSession() { ... }
```

---

## Class Inheritance Documentation

```ts
/**
 * Base class for all data repository implementations.
 *
 * @remarks
 * Subclasses must implement `findById` and `save`. All other methods
 * are implemented in terms of those two.
 *
 * @virtual — Override `buildQuery` to customize query construction.
 */
export abstract class BaseRepository<T extends Entity> {
  /**
   * Finds an entity by its primary key.
   * @virtual — Subclasses must override this method.
   */
  abstract findById(id: string): Promise<T | null>;

  /**
   * Persists an entity, inserting or updating as appropriate.
   * @virtual — Subclasses must override this method.
   */
  abstract save(entity: T): Promise<T>;
}

/**
 * PostgreSQL implementation of {@link BaseRepository}.
 * @public
 */
export class PostgresRepository<T extends Entity> extends BaseRepository<T> {
  /** @override */
  async findById(id: string): Promise<T | null> { ... }

  /** @override */
  async save(entity: T): Promise<T> { ... }
}
```

---

## Module and File Documentation

Every `.ts` file should have a module-level comment:

```ts
/**
 * @fileoverview Rate limiter middleware using a sliding window algorithm.
 *
 * Provides per-IP and per-user rate limiting for Express routes.
 * Backed by Redis for distributed deployments.
 *
 * @module middleware/rate-limit
 *
 * @see {@link https://redis.io/docs/manual/keyspace-notifications/ Redis Keyspace Notifications}
 * @since 3.1.0
 */
```

---

## TypeDoc Configuration

```json
// typedoc.json
{
  "entryPoints": ["src/index.ts"],
  "out": "docs",
  "plugin": ["typedoc-plugin-markdown"],
  "excludePrivate": true,
  "excludeInternal": true,
  "includeVersion": true,
  "readme": "README.md",
  "commentStyle": "jsdoc",
  "validation": {
    "notExported": true,
    "invalidLink": true,
    "notDocumented": true
  }
}
```

Enforce documentation coverage:

```ts
// tsconfig.json — enables missing-doc warnings via compiler
{
  "compilerOptions": {
    "noImplicitAny": true,
    "strict": true
  }
}

// Or use eslint-plugin-jsdoc / eslint-plugin-tsdoc:
// "tsdoc/syntax": "warn"
```

---

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
