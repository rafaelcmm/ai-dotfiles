---
name: javascript-documentation-best-practices
description: >
  Comprehensive guide for documenting JavaScript code with JSDoc, inline comments,
  and module-level documentation. Use this skill whenever writing, reviewing, or
  improving JavaScript documentation, JSDoc blocks, inline comments, @param/@returns
  annotations, typedefs, examples, or any JS code comment structure. Trigger on any
  JS file that needs documentation added, improved, or reviewed — including Node.js,
  browser scripts, ES modules, CommonJS, and framework code (React, Vue, Express, etc.).
---

# JavaScript Documentation Best Practices

## Philosophy

Good JavaScript documentation answers **"how do I use this?"** not just **"what does this do?"**.
Comments explain the *why* (intent, trade-offs, non-obvious decisions). JSDoc annotations explain
the *what* (types, params, returns, side-effects). Code should be readable enough to explain *how*.

**Core rule:** Every exported function, class, method, constant, and typedef must have a JSDoc block.
Internal helpers need at minimum a one-line description. Inline comments explain *why*, not *what*.

---

## Comment Types and When to Use Each

### 1. JSDoc Blocks (`/** ... */`) — For All Exported & Public Symbols

Used for: functions, classes, methods, constructors, constants, typedefs, modules, callbacks.
Must be placed **immediately before** the declaration — no blank line between comment and code.

```js
/**
 * Calculates the compound interest for a principal amount.
 *
 * Uses the standard formula: A = P(1 + r/n)^(nt)
 * Note: rate should be expressed as a decimal (e.g., 0.05 for 5%).
 *
 * @param {number} principal - The initial investment amount in USD.
 * @param {number} rate - Annual interest rate as a decimal (0–1).
 * @param {number} n - Number of times interest is compounded per year.
 * @param {number} t - Time in years.
 * @returns {number} The total amount after compound interest is applied.
 * @throws {RangeError} If rate is negative or greater than 1.
 *
 * @example
 * const total = compoundInterest(1000, 0.05, 12, 10);
 * console.log(total); // 1647.01
 */
function compoundInterest(principal, rate, n, t) { ... }
```

### 2. Single-line Comments (`//`) — For Inline Explanations

Used for: explaining *why* a non-obvious decision was made, noting a gotcha, flagging a workaround.
Place **above** the line it describes (not inline unless very short).

```js
// Use indexOf instead of includes for IE11 compatibility
const found = arr.indexOf(val) !== -1;

// Must be parsed before the DOM is ready — do not defer this script
initAuthToken();
```

### 3. Block Comments (`/* ... */`) — For Multi-line Non-JSDoc Notes

Used for: section separators, algorithm descriptions, large TODO blocks.
**Never** start with `/**` (double-star) unless it's a JSDoc block.

```js
/*
 * Sorting strategy: use radix sort for numeric arrays > 10k elements,
 * fallback to Array.prototype.sort for all other cases. Benchmarked
 * in Chrome 118 — radix is 3x faster at 50k elements.
 */
function smartSort(arr) { ... }
```

---

## JSDoc Tag Reference — Comprehensive

### Function Parameters

```js
/**
 * @param {string} name - Required string parameter.
 * @param {number} [age] - Optional parameter (square brackets).
 * @param {boolean} [active=true] - Optional with default value.
 * @param {string|number} id - Union type (accepts string OR number).
 * @param {Object} options - Object parameter.
 * @param {string} options.host - Nested property of `options`.
 * @param {number} [options.port=3000] - Optional nested property with default.
 * @param {...string} tags - Rest/variadic parameter.
 */
```

### Return Values

```js
/**
 * @returns {Promise<User[]>} Resolves with an array of User objects.
 * @returns {string|null} The token string, or null if not authenticated.
 * @returns {void} For functions with no return value.
 */
```

### Errors and Exceptions

```js
/**
 * @throws {TypeError} If `config` is not a plain object.
 * @throws {Error} If the network request fails after 3 retries.
 */
```

### Types — Defining Reusable Shapes

```js
/**
 * @typedef {Object} UserProfile
 * @property {string} id - UUID v4 identifier.
 * @property {string} name - Display name (1–100 chars).
 * @property {string} email - Validated email address.
 * @property {'admin'|'user'|'guest'} role - Access level.
 * @property {Date} createdAt - Account creation timestamp.
 * @property {Object} [preferences] - Optional preferences object.
 * @property {boolean} [preferences.darkMode=false] - UI theme preference.
 */

/**
 * @typedef {function(Error|null, UserProfile=): void} UserCallback
 */
```

### Classes

```js
/**
 * Manages a pool of reusable database connections.
 *
 * Connections are lazily created and returned to the pool after use.
 * The pool automatically evicts idle connections older than `idleTimeout`.
 *
 * @class
 * @example
 * const pool = new ConnectionPool({ host: 'localhost', maxSize: 10 });
 * const conn = await pool.acquire();
 * await conn.query('SELECT 1');
 * pool.release(conn);
 */
class ConnectionPool {
  /**
   * Creates a new connection pool.
   *
   * @param {Object} config - Pool configuration.
   * @param {string} config.host - Database host.
   * @param {number} [config.maxSize=5] - Maximum concurrent connections.
   * @param {number} [config.idleTimeout=30000] - Idle connection TTL in ms.
   */
  constructor(config) { ... }

  /**
   * Acquires an available connection from the pool.
   * Blocks until a connection is available or `timeout` is reached.
   *
   * @param {number} [timeout=5000] - Max wait time in milliseconds.
   * @returns {Promise<Connection>} An active database connection.
   * @throws {TimeoutError} If no connection becomes available within `timeout`.
   */
  async acquire(timeout = 5000) { ... }
}
```

### Lifecycle and Status Tags

```js
/**
 * @deprecated since v3.0.0 — Use `fetchUserById(id)` instead.
 * Will be removed in v4.0.0.
 */
function getUser(id) { ... }

/**
 * @since 2.1.0
 * @version 2.3.0
 * @author Jane Smith <jane@example.com>
 */

/**
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API}
 * @see {@link UserProfile} for the shape of resolved data.
 */
```

### Async and Generators

```js
/**
 * Streams paginated results from the API.
 *
 * @generator
 * @yields {UserProfile} One user profile per iteration.
 *
 * @example
 * for await (const user of streamUsers({ limit: 100 })) {
 *   console.log(user.name);
 * }
 */
async function* streamUsers(opts) { ... }
```

### Callbacks

```js
/**
 * @callback RequestHandler
 * @param {import('express').Request} req - Incoming HTTP request.
 * @param {import('express').Response} res - Outgoing HTTP response.
 * @param {Function} next - Calls the next middleware in the chain.
 * @returns {void}
 */

/**
 * Registers a route handler for GET requests.
 *
 * @param {string} path - URL path pattern (e.g., '/users/:id').
 * @param {RequestHandler} handler - Route handler function.
 */
```

---

## Module-Level Documentation

Every file should have a module-level JSDoc at the top:

```js
/**
 * @fileoverview Authentication utilities for JWT token management.
 *
 * Provides functions for signing, verifying, and refreshing JWT tokens.
 * Tokens follow RFC 7519 and use RS256 signing by default.
 *
 * @module auth/tokens
 * @requires jsonwebtoken
 * @see {@link https://tools.ietf.org/html/rfc7519 RFC 7519}
 */
```

---

## Inline Comment Best Practices

### ✅ Comment the WHY, not the WHAT

```js
// ✅ Good — explains the reason, not the mechanics
// Delay by one tick so the DOM has flushed before we read offsetHeight
setTimeout(updateLayout, 0);

// ❌ Bad — just restates the code
// Set timeout to 0 milliseconds
setTimeout(updateLayout, 0);
```

### ✅ Explain non-obvious business logic

```js
// Free tier users are limited to 3 exports/month (billing cycle resets on the 1st)
// This check must run BEFORE the exportData() call — not inside it
if (user.tier === 'free' && user.exportsThisMonth >= 3) {
  throw new QuotaExceededError();
}
```

### ✅ Annotate workarounds and browser quirks

```js
// Safari 15 does not support structuredClone — polyfill via JSON roundtrip
const clone = typeof structuredClone === 'function'
  ? structuredClone(obj)
  : JSON.parse(JSON.stringify(obj));
```

### ✅ Use TODO/FIXME/HACK with context

```js
// TODO(jane): Replace with WebSocket once infra supports it (see JIRA-1234)
// FIXME: Race condition when two tabs call this simultaneously — needs a lock
// HACK: API returns 200 for 404s when the item is soft-deleted — treat empty body as 404
```

### ❌ Avoid noise comments

```js
// ❌ Never do this — comments that restate the code add zero value
let count = 0; // initialize count to 0
i++; // increment i
return result; // return the result
```

---

## Examples Best Practices

Always include at least one `@example` for public functions:

```js
/**
 * Debounces a function call by the given delay.
 *
 * @param {Function} fn - The function to debounce.
 * @param {number} delay - Wait time in milliseconds.
 * @returns {Function} A debounced version of `fn`.
 *
 * @example <caption>Debounce a search input handler</caption>
 * const handleSearch = debounce((query) => {
 *   fetchResults(query);
 * }, 300);
 * searchInput.addEventListener('input', (e) => handleSearch(e.target.value));
 *
 * @example <caption>Debounce with immediate invocation</caption>
 * const save = debounce(persist, 1000);
 * save(data); // triggers immediately, then waits 1s before allowing next call
 */
```

---

## Type Annotations Without TypeScript

Use JSDoc types to get TypeScript-like checking in plain JS:

```js
// @ts-check  ← add at top of file for full TS checking in VS Code

/** @type {Map<string, Set<number>>} */
const userSessions = new Map();

/** @type {Readonly<{apiUrl: string, timeout: number}>} */
const CONFIG = Object.freeze({ apiUrl: '/api', timeout: 5000 });

/**
 * @template T
 * @param {T[]} items
 * @param {function(T): boolean} predicate
 * @returns {T|undefined}
 */
function findFirst(items, predicate) {
  return items.find(predicate);
}
```

---

## ESLint and Tooling Integration

### eslint-plugin-jsdoc — Enforce documentation

```json
// .eslintrc.json
{
  "plugins": ["jsdoc"],
  "rules": {
    "jsdoc/require-jsdoc": ["warn", {
      "require": {
        "FunctionDeclaration": true,
        "MethodDefinition": true,
        "ClassDeclaration": true
      }
    }],
    "jsdoc/require-param": "warn",
    "jsdoc/require-returns": "warn",
    "jsdoc/check-types": "error",
    "jsdoc/valid-types": "error"
  }
}
```

### Generate docs with JSDoc CLI

```json
// jsdoc.json
{
  "source": { "include": ["src"], "includePattern": ".+\\.js$" },
  "opts": { "destination": "docs/", "recurse": true },
  "plugins": ["plugins/markdown"]
}
```

```bash
npx jsdoc -c jsdoc.json
```

---

## Anti-Patterns to Avoid

```js
// ❌ Missing @param types — JSDoc without types is half-documented
/**
 * @param name
 * @returns the greeting
 */

// ❌ Redundant description that just repeats the name
/** Gets the user. */
function getUser() { ... }

// ❌ Lying documentation — worse than no docs
/** @returns {string} */ // but actually returns null sometimes
function getName() { return user?.name; }

// ❌ Commented-out dead code without explanation
// function oldHandler(req, res) { ... }

// ❌ Giant wall of comment with no structure
/**
 * This function does a lot of things. First it validates the input, then
 * it fetches from the database, then it formats the response... [200 words]
 */
```

---

## Documentation Checklist

When adding or reviewing docs, verify:

- [ ] Every exported function/class/method has a JSDoc block
- [ ] All `@param` tags have types `{type}` and descriptions
- [ ] `@returns` documents type and what it represents
- [ ] `@throws` documents every exception that can escape
- [ ] At least one `@example` for any non-trivial public function
- [ ] `@deprecated` includes what to use instead and since when
- [ ] `@typedef` defined for any complex object shape used in 2+ places
- [ ] Module-level `@fileoverview` present in every file
- [ ] Inline comments explain WHY, not WHAT
- [ ] No noise comments restating the code
- [ ] No commented-out code without explanation
- [ ] All async functions document their Promise resolution value
