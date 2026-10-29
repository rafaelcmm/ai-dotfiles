---
name: writing-good-tests
description: >-
  Use when writing, reviewing, or changing tests — adding a test case, adding or widening a
  mock, adding a cleanup or helper method for tests, or judging whether an existing suite
  actually protects anything. Covers tests that pass but catch nothing, tests that fail on
  every intentional change, mocks that swallow the behaviour under test, and assertions on
  source text instead of behaviour. This is guidance on test quality; it says nothing about
  when in the cycle tests get written.
---

# Writing Good Tests

A test exists to catch a specific break. Two principles govern everything here:

```
1. Every test names the break it catches
2. Every test exercises the real thing
```

**Scope note.** This skill is about test *quality*, not test *ordering*. It applies
identically whether a test was written before the code, alongside it, or after it. Nothing
here mandates a failing test first; the project's testing workflow is defined in
`AGENTS.md` and is unchanged by this skill.

---

## The governing question

Before writing a test body, answer it:

> **What production change should make this test fail — and is that change a bug or a
> decision?**

A test earns its place by catching a wrong branch, a missing side effect, a wrong argument,
a boundary case, or a broken contract. If the only change that fails it is a deliberate
decision, the test will fire on every redesign and sleep through every defect.

---

## Principle 1 — Name the break

### Derive expectations independently

Use literals and hand-checked fixtures. Table-driven tests with literal `want` values are
the preferred shape. An expectation computed by the code under test — or by its helpers —
passes no matter what that code does.

```typescript
// WRONG — mirror assertion: the same builder computes both sides, so it is always true
const expected = buildSearchQuery({ tag: 'urgent' });
expect(buildSearchQuery({ tag: 'urgent' })).toBe(expected);

// RIGHT — hand-derived literal
expect(buildSearchQuery({ tag: 'urgent' })).toBe('tag:"urgent"');
```

A mirror assertion survives any rewrite of the builder, including one that returns the empty
string.

### No change detectors

Test the behaviour that depends on the decision, not the decision itself.

```typescript
// WRONG — change detector: fails when someone tunes the constant, passes when retries break
expect(MAX_RETRIES).toBe(5);

// RIGHT — the behaviour the constant exists to produce
// "a failing call is retried 5 times and the 6th attempt never happens"
```

The same applies to exact message wording and private structure. Assert the observable
consequence.

### Behaviour, not text

Asserting that a script, config file, skill, or template **contains an exact line** proves
only that the source is the source. It passes while the behaviour is broken, and fails on
every harmless rewording.

```
Scripts   → run them against controlled inputs; assert stdout, side effects, exit codes.
Configs   → load them through the real loader; assert the resolved behaviour.
Documents that instruct agents → test the consuming agent's behaviour.
Prose for humans → earns no test at all.
```

### Your code, not the framework

Test the contract your code makes at its boundaries — the route you register, the query you
emit, the payload you produce. Upstream mechanics are their maintainers' tests to write.
The classic failure: asserting that your router invokes a handler you registered. That is
the router's test.

When upstream behaviour genuinely surprised you, write one narrow characterization test
that names the assumption.

The same boundary applies inside your own code. Constructors, getters, constants, and
trivial forwarding earn a test only when they validate, normalize, default, derive, enforce,
or cause a side effect. Otherwise assert the first consumer-visible result that depends on
them.

### Gate function

```
BEFORE writing the test body:

  Name the production change that would make this test fail.

    Cannot name one               → redesign around an observable behaviour
    "The source text changed"     → run the artifact; assert its effects
    Only intentional decisions    → change detector; test the behaviour that
                                    depends on the decision

  Confirm the expected value is derived without the code under test.
    It reuses the code's logic or helpers
      → replace it with a literal or a hand-checked fixture
```

---

## Principle 2 — Exercise the real thing

### The mock earns no assertions

An assertion on a mock passes when the mock is present and fails when it is absent. It says
nothing about the component.

```typescript
// RIGHT — real behaviour
expect(screen.getByRole('navigation')).toBeInTheDocument();

// WRONG — mock existence; green even if the real nav never renders
expect(screen.getByTestId('sidebar-mock')).toBeInTheDocument();
```

The question to ask of any failing-or-passing assertion: **are we testing the behaviour of
a mock?** If yes, unmock the component or delete the assertion.

### Mock at the right level

Learn every side effect of the real method before replacing it. Mock the slow or external
operation; keep the parts the test depends on real.

```typescript
// WRONG — the mock swallows the config write that duplicate detection reads
vi.mock('ToolCatalog', () => ({
  discoverAndCacheTools: vi.fn().mockResolvedValue(undefined)
}));

// RIGHT — mock only the slow server startup; the config write stays real
vi.mock('MCPServerManager');
```

When unsure what a method actually does, run the test against the real implementation first
and observe what has to happen.

### Make doubles specific

A fake that accepts anything verifies nothing. When arguments, call counts, or ordering are
part of the contract, assert them. Give each branch — success, error, malformed — its own
fixture or spy, so the wrong branch cannot satisfy the expectation.

### Mirror real data completely

Mock the complete structure as it exists in reality, including the documented fields your
test does not read. A partial mock fails silently when downstream code reads an omitted
field: the test stays green while integration breaks.

### Production classes carry production methods only

Cleanup that only tests need lives in test utilities, never as a `destroy()` on the
production class.

```
Is this method called only from test files?          → test utility
Does this class own this resource's lifecycle?  No   → test utility
```

A test-only method on a production class is also a production API that someone will
eventually call.

### Prefer real components over complex mocks

Switch to an integration test with real components when:

- mock setup outgrows the test logic
- the mock is missing methods the real component has
- tests break when the mock changes rather than when the code changes

The question: **do we need to be using a mock here?**

### Wait on conditions, never on the clock

An arbitrary `sleep`, `setTimeout`, or `time.sleep()` encodes a guess about speed. It passes
on your machine and fails under CI load, and it fails for a reason unrelated to the
behaviour. Poll for the condition the test actually cares about. See
`systematic-debugging/references/condition-based-waiting.md` for the pattern, including the
narrow case where a timed wait is correct.

### Gate function

```
BEFORE adding a mock or a test helper:

  List the real method's side effects.
    Keep the ones the test depends on real.
    Mock the slow or external level below them.

  Mock responses mirror the complete real structure.

  A method only tests call lives in test utilities, not in production.

  About to assert on the mock itself?
    Unmock it, or delete the assertion.
```

---

## The Mutation Check

Run this as the finishing step on every test file. Mentally mutate the production code; at
least one test must fail for each realistic mutation:

- wrong constant or wrong argument
- wrong branch handler
- missing state change or missing side effect
- empty or default return value
- missing validation for zero, empty, nil, unauthorized, or malformed input

A mutation that nothing catches marks the behaviour as unprotected — or marks the test as
tautological. Either way, the suite is reporting safety it does not provide.

---

## Quick reference

| When you... | Do |
|-------------|-----|
| Write any test | Name the break it catches — a bug, not a decision |
| Build an expected value | Derive it by hand; never with the code under test |
| Test a script, config, or document | Run it and assert effects; never grep its text |
| Reach for a dependency test | Test your boundary contract, not their documented mechanics |
| Want to assert on a mocked element | Test the real component, or unmock it |
| Are about to mock a method | Learn its side effects; mock the slow or external level |
| Build a mock response | Mirror the real structure completely |
| Write a permissive fake | Assert the arguments, counts, and ordering that are contract |
| Need cleanup only tests use | Put it in a test utility |
| Watch mock setup balloon | Switch to an integration test with real components |
| Feel tempted to sleep | Poll the condition instead |
| Finish a test file | Run the Mutation Check |

---

## Warning signs

- Setup and assertion share the same object or builder, guaranteeing equality
- The test can fail only through a panic, a crash, or a missing selector
- The test fails on every intentional change and never on accidental breakage
- Expected values are hidden behind loops, builders, or helpers
- The test greps source text, or asserts that a removed symbol stays removed
- The test would still pass if only the framework remained
- The test exists for coverage and checks no outcome or side effect
- An assertion checks a `*-mock` test ID, or fails when the mock is removed
- A method on a production class is called only from test files
- Mock setup is more than half the test, or you cannot explain why the mock is needed
- Something is mocked "just to be safe"
- A fixed delay appears anywhere outside a test of timing behaviour itself

---

## Related

- `verification-before-completion` — a green suite of tautological tests is evidence of
  nothing; the Mutation Check is what makes green mean something.
- `systematic-debugging` — its Phase 4 asks for a test that reproduces the bug; the quality
  bar for that test is here.
- `clean-code`, `commenting-standards` — apply to test code as written, not at a discount.

---

## Attribution

Adapted from `writing-good-tests.md` in `obra/superpowers` (MIT, © 2025 Jesse Vincent).
The tests-first ordering material in the original is deliberately omitted; this project's
testing workflow is unchanged.
