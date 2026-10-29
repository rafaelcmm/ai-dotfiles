# Root Cause Tracing

Backward tracing technique for Phase 1, step 5 of `systematic-debugging`. Load it when the
error surfaces deep in a call stack and the origin of the bad value is unknown.

## Contents

- [The problem it solves](#the-problem-it-solves)
- [When to use it](#when-to-use-it)
- [The tracing process](#the-tracing-process)
- [When manual tracing dead-ends: add instrumentation](#when-manual-tracing-dead-ends-add-instrumentation)
- [Finding which test leaves state behind](#finding-which-test-leaves-state-behind)
- [Worked example](#worked-example)
- [Stack trace tips](#stack-trace-tips)

---

## The problem it solves

A bug manifests where the operation finally fails: `git init` runs in the wrong directory, a
file lands in the wrong place, a database opens at the wrong path. The instinct is to fix it
there — add a guard, correct the path at the call site.

That is a symptom fix. The bad value was constructed somewhere else and will be constructed
again, by the next caller, through a path your guard does not cover.

**Trace backward through the call chain until you find the original trigger, then fix
there.**

---

## When to use it

- the error happens deep in execution rather than at an entry point
- the stack trace shows a long call chain
- it is unclear where an invalid value originated
- you need to know which test or code path triggers a problem

If you can trace backward, do. If tracing genuinely dead-ends — a value arrives from a
boundary you cannot see past — fix at the furthest point you can establish, and record in
the commit why that is the furthest point.

---

## The tracing process

**1. Observe the symptom precisely.** The exact message, with the values in it.

```
Error: git init failed in ~/project/packages/core
```

**2. Find the immediate cause.** The code that directly performs the failing operation.

```typescript
await execFileAsync('git', ['init'], { cwd: projectDir });
```

**3. Ask what called this**, and what value it passed.

```
WorktreeManager.createSessionWorktree(projectDir, sessionId)
  ← Session.initializeWorkspace()
  ← Session.create()
  ← test, at Project.create()
```

**4. Keep going up.** At each level, ask what the value was at that level, not what it was
supposed to be.

```
projectDir = ''          // empty string
→ '' as cwd resolves to process.cwd()
→ which is the source directory
```

**5. Find the original trigger.** Where did the bad value come into existence?

```typescript
const context = setupCoreTest();          // returns { tempDir: '' } initially
Project.create('name', context.tempDir);  // read before beforeEach populated it
```

The root cause is the module-level read of a value that is only valid after setup — not the
`git init` call that reported it.

---

## When manual tracing dead-ends: add instrumentation

Capture the call chain at the point of the dangerous operation.

```typescript
async function gitInit(directory: string) {
  const stack = new Error().stack;
  console.error('DEBUG git init:', {
    directory,
    cwd: process.cwd(),
    nodeEnv: process.env.NODE_ENV,
    stack,
  });

  await execFileAsync('git', ['init'], { cwd: directory });
}
```

**In tests, use `console.error`, not the logger.** Test harnesses commonly suppress or
buffer logger output, and a diagnostic you cannot see reads as a diagnostic that found
nothing.

**Log before the operation, not in its error handler.** If the operation hangs, corrupts
state, or exits the process, the handler never runs.

Capture and filter:

```bash
npm test 2>&1 | grep 'DEBUG git init'
```

Then read the traces for the pattern: the same test file each time? the same parameter? a
single call or every call?

---

## Finding which test leaves state behind

When unwanted state appears during a test run but you do not know which test creates it,
bisect by file:

```bash
bash scripts/find-polluter.sh '.git' 'src/**/*.test.ts' 'npm test'
```

It runs the matching test files one at a time and stops at the first one after which the
checked path exists. See the script's usage block for its arguments and exit codes.

---

## Worked example

**Symptom:** a `.git` directory appears inside `packages/core/` — the source tree.

**Trace chain:**

1. `git init` ran in `process.cwd()` — because its `cwd` argument was empty
2. `WorktreeManager` was called with an empty `projectDir`
3. `Session.create()` passed the empty string through
4. the test read `context.tempDir` before `beforeEach` assigned it
5. `setupCoreTest()` returns `{ tempDir: '' }` until that assignment

**Root cause:** a top-level initializer reading a value that is only valid after setup runs.

**Fix at the source:** make `tempDir` a getter that throws when read before `beforeEach`.
The empty string can no longer be produced, so no downstream code can receive it.

**Then, because this bug had already shipped:** add validation at the layers the value
passed through, so a future regression fails loudly and early rather than silently writing
to the wrong directory. See `references/defense-in-depth.md`, and read its scope section —
that layering is a response to a shipped bug, not a default coding posture.

---

## Stack trace tips

| Situation | Do |
|-----------|-----|
| Running under a test harness | `console.error`, not the logger — logger output may be suppressed |
| Dangerous or hanging operation | Log before it, never only in the catch |
| Value looks right but behaves wrong | Log the type and the raw form, not the formatted string |
| Need the call chain | `new Error().stack` at the operation site |
| Noisy output | Prefix every diagnostic with a unique token and `grep` for it |
| Done investigating | Remove the instrumentation in the same change as the fix |

---

## Key principle

**Never fix only where the error appears.** Trace to where the bad value was created. If you
cannot reach that point, say so explicitly rather than letting the symptom fix stand in for
a diagnosis.

## Attribution

Adapted from `root-cause-tracing.md` in `obra/superpowers` (MIT, © 2025 Jesse Vincent).
