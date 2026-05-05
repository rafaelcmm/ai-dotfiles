---
description: "Use when creating, reviewing, or splitting files. Keep files small, focused, and co-located with what they serve. One file — one clear purpose. Covers file size limits, naming, co-location, and single responsibility."
---

# Objective Files

A file should have a single, stateable reason to exist. If you need more than one sentence to describe what a file does, it is doing too much.

## Normative Language

- MUST = mandatory rule.
- SHOULD = expected default; deviations require explicit justification.
- MAY = optional.
- When rules conflict, apply the stricter rule.
- If file type is unclear, treat it as a logic file and apply logic-file limits.

## The Core Questions

Before creating or reviewing a file, ask:

1. **What is the single responsibility of this file?**
2. **Who is the primary consumer of this file?**
3. **Are the things inside this file equally likely to change together?**

If the answers diverge across the contents of the file, split it.

If the answer is unknown, stop and investigate before adding new code to that file.

---

## File Size

Size is a symptom, not the root cause, but it is the easiest enforcement signal.

| Lines     | Signal                                                                                         |
| --------- | ---------------------------------------------------------------------------------------------- |
| <= 300     | Pass. File can evolve if responsibility remains singular. |
| 301 - 500 | Mandatory split-in-progress. Agent MUST decompose in same task unless user explicitly blocks. |
| > 500     | Hard violation for logic files. Agent MUST block unrelated feature growth until split plan exists. |

These thresholds apply to logic files. Configuration, fixture, and generated files MAY exceed limits, but MUST still keep single responsibility.

## Hard Limits By File Type

- Logic files MUST stay <= 300 lines when possible.
- Logic files between 301 and 500 lines MUST be actively split; no passive acceptance.
- Logic files > 500 lines MUST not receive unrelated additions.
- Generated, schema-mirror, and fixture files MAY exceed line limits, but MUST not mix responsibilities.
- Index files MUST only re-export. Business logic in `index.*` is a violation.

---

## Scope Rules

- One module, component, class, or domain concept per file.
- Helpers used only by one file live in that file or directly alongside it — not in a global `utils/`.
- Types and interfaces belong next to the code that owns them, not in a central `types/` file unless they are shared across multiple modules.
- A file that imports from every other part of the codebase is a coordinator, not a module — split or refactor.
- Mixing UI, domain logic, and data access in one file is a violation unless an explicit exception is documented.
- If exports serve different consumers or have different change cadence, agent MUST split by concern.

---

## Agent Enforcement Behavior

On every create or update, agent MUST run this gate before final output:

1. Responsibility gate: can file purpose be stated in one sentence without "and" or "misc"?
2. Consumer gate: do exports target same primary consumer?
3. Size gate: is file above 300 lines?

If any gate fails:

- Agent MUST stop feature accretion in that file.
- Agent MUST propose concrete split map (target file names + ownership boundaries).
- Agent SHOULD implement first extraction in same task when user did not block refactor scope.
- Agent MUST preserve behavior through tests or equivalent verification.

For files > 500 lines:

- Agent MUST block unrelated additions.
- Agent MUST deliver split plan before adding new logic.
- Agent MAY ask user only for tie-break choices, not for decomposition work agent can do.

---

## Exception Policy

Exceptions are allowed only when explicit, time-bound, and documented.

Valid exceptions:

- Generated code.
- Protocol/schema mirror files.
- Migration snapshots.
- Vendor lock-step files.

Invalid exceptions:

- "Faster for this PR"
- "Will fix later"
- "Too much work now"

Exception record MUST include:

- Reason
- Owner
- Expiry trigger
- Follow-up task reference

Without this record, exception is invalid.

---

## When to Split a File

Split when **any** of the following is true:

- You scroll past unrelated logic to find what you need.
- Two features share the file but change for different reasons.
- The same file is touched in nearly every PR across different features.
- You cannot name the file without using "and", "or", or "misc".
- A function deep in the file is reused by another module and must be extracted for import.
- Tests for the file are longer than the file and test unrelated behaviour in the same suite.

Mandatory split triggers:

- File > 300 logic lines.
- More than one domain concept.
- More than one primary consumer group.
- Name requires "and", "or", "misc", "helpers", or "common" to describe mixed concerns.

---

## Bad Examples

### 1. The "utils" dumping ground

```
src/
  utils.ts        ← 800 lines: date formatting, API error handling,
                     localStorage wrappers, string truncation, auth helpers
```

**Why this is wrong:** These utilities belong to different domains and change for different reasons. A bug fix in the auth helper requires touching the same file as a date formatting change, increasing diff noise and merge conflicts.

---

### 2. A component file that owns too much

```typescript
// UserPage.tsx — 620 lines

// --- types (lines 1–40)
type User = { ... }
type Address = { ... }
type BillingInfo = { ... }

// --- API calls (lines 41–130)
async function fetchUser() { ... }
async function updateUser() { ... }
async function fetchBillingInfo() { ... }

// --- helper logic (lines 131–260)
function formatAddress(address: Address) { ... }
function validateBillingForm(data: BillingInfo) { ... }

// --- sub-components (lines 261–500)
function AddressCard() { ... }
function BillingSection() { ... }

// --- main component (lines 501–620)
export function UserPage() { ... }
```

**Why this is wrong:** Types, API calls, formatting logic, sub-components, and the page component all change for different reasons. A designer changing `AddressCard` touches the same file as a backend engineer changing `fetchBillingInfo`.

---

### 3. A single `hooks.ts` file for everything

```
src/
  hooks.ts    ← useAuth, useCart, useSearch, useTheme, useNotifications
```

**Why this is wrong:** Each hook belongs to a different domain. They cannot be independently tested, tree-shaken, or maintained without touching every other hook in the file.

---

### 4. Types centralised away from their owners

```
src/
  types/
    index.ts    ← 400 lines of every interface and type in the app
  features/
    cart/
      Cart.tsx  ← imports CartItem, Discount, ShippingOption from ../../types
    auth/
      Login.tsx ← imports User, Session, AuthError from ../../types
```

**Why this is wrong:** `types/index.ts` is implicitly coupled to every feature. Adding a new type field requires modifying a shared file, and the type ownership is invisible from the feature directory.

---

## Good Examples

### 1. Utilities split by domain

```
src/
  lib/
    dates.ts          ← date formatting and relative time helpers
    currency.ts       ← formatting, rounding, locale display
    strings.ts        ← truncation, slugify, capitalise
  features/
    auth/
      auth.utils.ts   ← helpers used only within the auth feature
```

---

### 2. Component file with a clear boundary

```
src/features/user/
  UserPage.tsx              ← page component only; composes sub-components
  UserPage.test.tsx         ← integration test for the page
  UserAddress.tsx           ← AddressCard sub-component
  UserBilling.tsx           ← BillingSection sub-component
  user.api.ts               ← fetchUser, updateUser, fetchBillingInfo
  user.types.ts             ← User, Address, BillingInfo
  user.utils.ts             ← formatAddress, validateBillingForm
```

Each file answers one question. A designer can open `UserAddress.tsx` without reading API code. A backend engineer can change `user.api.ts` without touching components.

---

### 3. Hooks split by domain

```
src/features/
  auth/
    useAuth.ts
  cart/
    useCart.ts
  search/
    useSearch.ts
  theme/
    useTheme.ts
```

---

### 4. Types co-located with their owner

```
src/features/
  cart/
    cart.types.ts     ← CartItem, Discount, ShippingOption
    Cart.tsx          ← imports from ./cart.types
  auth/
    auth.types.ts     ← User, Session, AuthError
    Login.tsx         ← imports from ./auth.types
```

Shared cross-feature types live in a `shared/types/` only when two or more unrelated features depend on the same contract.

---

## Co-location Rules

Co-location means: **put a file as close as possible to the code it serves.**

| File type          | Lives next to                                               |
| ------------------ | ----------------------------------------------------------- |
| Component test     | The component file                                          |
| Component styles   | The component file                                          |
| Feature types      | The feature directory                                       |
| Feature utilities  | The feature directory                                       |
| Shared utilities   | `lib/` or `shared/` at the nearest common ancestor          |
| Mocks and fixtures | The test file or a `__mocks__` folder in the same directory |
| Config for a tool  | The root of the scope the tool applies to                   |

### Bad: fixtures scattered in a global folder

```
src/
  __fixtures__/
    user.json
    cart.json
    product.json
tests/
  cart/
    cart.test.ts       ← imports from ../../src/__fixtures__/cart.json
  user/
    user.test.ts       ← imports from ../../src/__fixtures__/user.json
```

### Good: fixtures next to the tests that use them

```
src/
  features/
    cart/
      cart.test.ts
      cart.fixture.ts   ← CartItem[], Discount[] test data
    user/
      user.test.ts
      user.fixture.ts   ← User[], Session test data
```

---

## Naming Signals

A filename should make the file's purpose obvious without opening it.

| Pattern              | Meaning                                |
| -------------------- | -------------------------------------- |
| `feature.types.ts`   | Type definitions for a feature         |
| `feature.utils.ts`   | Pure helpers scoped to a feature       |
| `feature.api.ts`     | Data-fetching layer for a feature      |
| `feature.store.ts`   | State management for a feature         |
| `feature.test.ts`    | Unit/integration tests                 |
| `feature.fixture.ts` | Test data factories or static fixtures |
| `useFeature.ts`      | React hook for a feature               |
| `Feature.tsx`        | React component                        |

Avoid opaque names: `helpers.ts`, `misc.ts`, `common.ts`, `stuff.ts`, `index.ts` as the primary logic file (index files should only re-export).

---

## Split Playbook (Mandatory)

When splitting, use this order:

1. Extract types/contracts (`feature.types.*`).
2. Extract side effects/integration (`feature.api.*` or adapter files).
3. Extract pure logic (`feature.utils.*` or domain service).
4. Keep coordinator/component focused on orchestration only.

After split:

- Imports must point to nearest ownership boundary.
- File names must state one purpose.
- Removed mixed concerns must not re-enter coordinator file.

---

## Decision Process

```text
1. Can you name this file without "and", "or", or "misc"?
   - NO  → split by responsibility before writing more code.

2. Is every export in this file used by the same consumers?
   - NO  → extract the diverging exports to their own file.

3. Is the file over 300 lines of logic?
   - YES → identify the natural split: types, utils, sub-components, API layer.

4. Does a helper or type live far from the code that uses it?
   - YES → move it closer; co-locate unless it is genuinely shared.

5. Is there a test, style, or fixture file that lives more than one directory away from what it tests?
   - YES → move it next to the file it serves.

6. Is file over 500 lines and request is unrelated feature work?
  - YES → stop; split first or document valid exception record.
```

---

## Required Evidence In Final Agent Response

Agent MUST report:

- Files affected by split and resulting line counts.
- Why each boundary was chosen.
- What remains unsplit and why.
- Any exception record fields if exception used.

Claims without this evidence are non-compliant.

---

## Summary

| Principle             | Rule                                                            |
| --------------------- | --------------------------------------------------------------- |
| Single responsibility | One purpose, one file — stateable in one sentence               |
| Size limit            | <=300 pass; 301-500 mandatory split; >500 hard-stop for unrelated growth |
| No dumping grounds    | No `utils.ts`, `helpers.ts`, or `types/index.ts` catch-alls     |
| Co-location           | Tests, styles, fixtures, and types live next to what they serve |
| Naming                | Filename reveals purpose without reading the contents           |
| Shared code           | Lives at the nearest common ancestor, not always at the root    |
