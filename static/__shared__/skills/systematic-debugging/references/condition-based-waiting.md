# Condition-Based Waiting

Technique for the flaky-test case of `systematic-debugging`. Load it when a test waits on
the clock instead of on the thing it cares about.

## Contents

- [The problem it solves](#the-problem-it-solves)
- [When to use it](#when-to-use-it)
- [The core pattern](#the-core-pattern)
- [Common conditions](#common-conditions)
- [A polling helper](#a-polling-helper)
- [Mistakes that make polling worse than sleeping](#mistakes-that-make-polling-worse-than-sleeping)
- [When a fixed delay is correct](#when-a-fixed-delay-is-correct)

---

## The problem it solves

A test that sleeps for 50ms is asserting that the operation finishes in under 50ms on every
machine that will ever run it. That assertion is unrelated to the behaviour under test, and
it fails first under CI load, under parallel execution, and on the slowest contributor's
laptop.

The failure is also misdiagnosed as "flaky infrastructure" rather than as a wrong wait, so
the usual remediation is to raise the delay — which slows the suite and keeps the race.

**Wait for the condition you care about, not for a guess about how long it takes.**

---

## When to use it

- a test contains `setTimeout`, `sleep`, `time.sleep()`, `Thread.sleep` as a wait
- a test passes locally and fails under load or in CI
- tests time out when the suite runs in parallel
- the test is waiting for an async operation to complete

Do not convert a delay that is itself the subject of the test — debounce windows, throttle
intervals, retry backoff. Those are covered below.

---

## The core pattern

```typescript
// WRONG — guessing at timing
await new Promise(r => setTimeout(r, 50));
const result = getResult();
expect(result).toBeDefined();

// RIGHT — waiting for the condition, then asserting it
await waitFor(() => getResult() !== undefined, 'result to be produced');
expect(getResult()).toBeDefined();
```

Two properties change: the test finishes as soon as the condition holds (usually far faster
than the delay), and when it does fail, it fails with a message naming what never happened.

---

## Common conditions

| Waiting for | Condition |
|-------------|-----------|
| An event | `waitFor(() => events.find(e => e.type === 'DONE'), 'DONE event')` |
| A state transition | `waitFor(() => machine.state === 'ready', 'machine ready')` |
| A count | `waitFor(() => items.length >= 5, 'five items')` |
| A file | `waitFor(() => fs.existsSync(path), 'config file to exist')` |
| A compound condition | `waitFor(() => obj.ready && obj.value > 10, 'obj ready with value')` |

---

## A polling helper

```typescript
/**
 * Polls `condition` until it returns a truthy value, then resolves with it.
 *
 * The description is mandatory because the timeout message is the only
 * diagnostic a CI failure will leave behind: "Timeout waiting for X" is
 * actionable, a bare timeout is not.
 *
 * Throws if the condition has not held within `timeoutMs`.
 */
async function waitFor<T>(
  condition: () => T | undefined | null | false,
  description: string,
  timeoutMs = 5000,
): Promise<T> {
  const startTime = Date.now();

  while (true) {
    const result = condition();
    if (result) return result;

    if (Date.now() - startTime > timeoutMs) {
      throw new Error(`Timeout waiting for ${description} after ${timeoutMs}ms`);
    }

    // 10ms keeps the loop responsive without spinning the event loop; at 1ms
    // the polling itself starves the work it is waiting for.
    await new Promise(r => setTimeout(r, 10));
  }
}
```

Most test frameworks ship an equivalent (`waitFor`, `eventually`, `Awaitility`,
`require.Eventually`). Use the framework's version where one exists — it integrates with the
runner's own timeout and reporting.

---

## Mistakes that make polling worse than sleeping

| Mistake | Consequence | Fix |
|---------|-------------|-----|
| Polling every 1ms | The poll loop starves the async work it waits on | Poll every ~10ms |
| No timeout | A never-met condition hangs the suite instead of failing it | Always bound it, with a message |
| Reading state once before the loop | Polls a stale snapshot forever | Call the getter inside the loop |
| Generic description | CI shows "Timeout" with nothing to act on | Name the condition in the message |
| Condition with a side effect | The poll changes what it measures | Keep the predicate read-only |

---

## When a fixed delay is correct

Only when the delay is derived from known timing, and only after waiting for the condition
that starts the clock.

```typescript
// The tool emits partial output every 100ms. Verifying that partial output
// accumulates requires observing two ticks, so 200ms is derived, not guessed.
await waitForEvent(manager, 'TOOL_STARTED');     // 1. wait for the condition
await new Promise(r => setTimeout(r, 200));      // 2. then the known interval
```

Three requirements, all of them:

1. The triggering condition was awaited first — the delay never stands in for it.
2. The duration comes from a documented interval, not from trial and error.
3. A comment states the arithmetic, so the next reader can re-derive it when the interval
   changes.

A fixed delay without all three is the bug this technique removes.

## Attribution

Adapted from `condition-based-waiting.md` in `obra/superpowers` (MIT, © 2025 Jesse Vincent).
