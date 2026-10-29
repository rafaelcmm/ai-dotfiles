# Advanced TypeScript Documentation Topics

Release tags and API surface documentation, class inheritance documentation,
module/file documentation, and TypeDoc configuration.

## Contents

- [Release Tags and API Surface Documentation](#release-tags-and-api-surface-documentation)
- [Class Inheritance Documentation](#class-inheritance-documentation)
- [Module and File Documentation](#module-and-file-documentation)
- [TypeDoc Configuration](#typedoc-configuration)

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

