# Defense-in-Depth Validation

Remediation technique for Phase 4 of `systematic-debugging`.

## Contents

- [Scope — read this first](#scope--read-this-first)
- [Why more than one layer](#why-more-than-one-layer)
- [The four layers](#the-four-layers)
- [Applying the pattern](#applying-the-pattern)
- [Worked example](#worked-example)
- [When one layer is the right answer](#when-one-layer-is-the-right-answer)

---

## Scope — read this first

**This pattern applies only when all three conditions hold:**

1. You have already **found the root cause** through Phase 1 investigation.
2. The bug **already shipped**, or already corrupted state outside the process — wrong
   directory written, wrong record updated, wrong credential used.
3. The bad value **crosses multiple layers** before causing damage, so a single check can be
   bypassed by another code path, a refactor, or a test mock.

**It is not a general coding posture.** Applied by default, layered validation adds
unrequested code at every boundary: it inflates files against the size limits in the
`objective-files` rule, and it builds defences for inputs nothing produces, which the YAGNI
gate in the code-change workflow exists to prevent.

```
New code, no bug yet          → validate at the boundary that owns the invariant. One layer.
Bug found, contained, fixed   → fix at the source. One layer, plus its test.
Bug found, already shipped,
  crossed several layers      → this pattern.
```

Deleting an existing layer is governed by the `chestertons-fence` rule: a validation that
looks redundant is frequently the one that caught the last incident.

---

## Why more than one layer

One validation says "we fixed the bug". Layered validation says "the bug cannot recur
through any path we know of". The difference matters only for a bug that has already proven
it reaches production.

Each layer catches a different class of recurrence:

- **entry validation** catches the common case and gives the clearest error
- **business-logic validation** catches values assembled internally, past the entry point
- **environment guards** catch context-specific danger — the operation is fine in
  production and catastrophic in a test run
- **debug instrumentation** is what you have left when all three are bypassed

---

## The four layers

### Layer 1 — Entry point validation

Reject obviously invalid input at the API boundary, with an error that names the problem.

```typescript
function createProject(name: string, workingDirectory: string) {
  if (!workingDirectory || workingDirectory.trim() === '') {
    throw new Error('workingDirectory cannot be empty');
  }
  if (!existsSync(workingDirectory)) {
    throw new Error(`workingDirectory does not exist: ${workingDirectory}`);
  }
  if (!statSync(workingDirectory).isDirectory()) {
    throw new Error(`workingDirectory is not a directory: ${workingDirectory}`);
  }
  // ...
}
```

### Layer 2 — Business logic validation

Assert the value makes sense for **this** operation. This is the layer that catches values
constructed internally, which never passed through layer 1.

```typescript
function initializeWorkspace(projectDir: string, sessionId: string) {
  if (!projectDir) {
    throw new Error('projectDir required for workspace initialization');
  }
  // ...
}
```

### Layer 3 — Environment guards

Refuse operations that are legitimate in one context and destructive in another. The guard
encodes the context, not the value.

```typescript
async function gitInit(directory: string) {
  // A test run must never initialize a repository outside the temp tree; doing so
  // writes into the working copy and the damage outlives the test process.
  if (process.env.NODE_ENV === 'test') {
    const normalized = normalize(resolve(directory));
    const tmpDir = normalize(resolve(tmpdir()));

    if (!normalized.startsWith(tmpDir)) {
      throw new Error(`Refusing git init outside temp dir during tests: ${directory}`);
    }
  }
  // ...
}
```

### Layer 4 — Debug instrumentation

Capture the context that identifies the caller, so the next occurrence arrives diagnosed.

```typescript
async function gitInit(directory: string) {
  logger.debug('About to git init', {
    directory,
    cwd: process.cwd(),
    stack: new Error().stack,
  });
  // ...
}
```

This layer prevents nothing. It exists so that a recurrence costs minutes instead of a
second investigation from scratch.

---

## Applying the pattern

1. **Trace the data flow.** Where does the bad value originate, and every place it is used.
   (`references/root-cause-tracing.md`)
2. **Map the checkpoints.** List each boundary the value actually crosses — not every
   boundary in the system.
3. **Add a layer at each mapped checkpoint**, each with a message that identifies which
   layer rejected the value.
4. **Test each layer independently.** Construct an input that bypasses layer 1 and assert
   layer 2 catches it. A layer with no test is an assumption.

Every added check is production code: it carries a docblock or an intent comment explaining
which failure it exists to prevent, per `commenting-standards`. A bare `if (!x) throw` with
no stated reason is the next engineer's candidate for deletion.

---

## Worked example

**Bug:** an empty `projectDir` caused `git init` to run in the source tree during tests.

**Data flow:** test setup produces `''` → `Project.create(name, '')` →
`WorkspaceManager.createWorkspace('')` → `git init` runs in `process.cwd()`.

**Layers added:**

| Layer | Location | Catches |
|-------|----------|---------|
| 1 | `Project.create()` | empty, nonexistent, or non-directory paths at the boundary |
| 2 | `WorkspaceManager` | an empty `projectDir` assembled internally |
| 3 | `WorktreeManager` | any `git init` outside the temp tree while `NODE_ENV=test` |
| 4 | before `git init` | the stack trace of whoever called it, for the next time |

**Why all four were needed, and this is the part that justifies the pattern:** during
testing, each layer caught cases the others missed. Alternative code paths reached the
workspace manager without passing through `Project.create()`. Mocks replaced the
business-logic check entirely. One platform normalized paths differently and only the
environment guard rejected it. The instrumentation identified a structural misuse none of
the three throws described.

---

## When one layer is the right answer

Most of the time. A bug caught in development, fixed at its source, and covered by a test
that reproduces it needs no layering — the test is the second layer.

Reach for four layers when the bug escaped, the blast radius was outside the process, and
you can name the distinct paths each layer covers. If you cannot name them, you are adding
code against a hypothetical.

## Attribution

Adapted from `defense-in-depth.md` in `obra/superpowers` (MIT, © 2025 Jesse Vincent). The
scope conditions are this project's addition, to keep the pattern from conflicting with the
`objective-files` rule and the YAGNI gate.
