# Review lenses — what to look for

This is the catalogue of substantive checks. Apply each lens to the mapped change surface. Not
every lens applies to every change; use judgment about which are load-bearing for *this* change.

## Contents

1. Architecture & design (think at multiple levels)
2. Implicit contracts (semantic contracts on public interfaces)
3. Parallel entry points
4. Abstractions & cognitive load
5. Complexity disguised as elegance
6. Error handling & edge cases
7. State consistency
8. Security
9. Performance
10. Observability
11. Tests
12. Documentation

---

## 1. Architecture & design — think at multiple levels simultaneously

A good reviewer moves fluidly between levels. Don't let a clean implementation distract from a
design problem: **well-written code in the wrong place is still wrong.**

- **System level** — Does this change belong in this service/module/layer? Does it create
  coupling between things that should stay decoupled? Does it introduce a circular dependency?
- **Design level** — Is this the right abstraction? Will it generalize, or is it already fighting
  future requirements? Is the responsibility assigned to the right place?
- **Implementation level** — Is the code correct, readable, efficient? Does it handle the failure
  paths?
- **Surface level** — Naming, formatting, conventions. Lowest priority, but worth noting where
  they obscure intent.

## 2. Implicit contracts — the most-missed source of regressions

Every public interface is a **semantic contract**: a promise about externally observable
behavior. This includes every exported function, every component prop, every API endpoint, and
every emitted event. **Breaking a semantic contract is a regression even if every unit test
passes.**

When a core interface changes:
- Enumerate **every downstream consumer** and ask whether the contract still holds for each.
- This is most critical in shared utilities, base components, and anything in a design system or
  core library — the blast radius is largest there.
- If the contract changed *intentionally*, it must be called out explicitly in the description
  and the migration path must be clear. If that's missing, it's a finding.

## 3. Parallel entry points

When a change adds a new code path, ask: **is there a parallel path that does the same thing?**
The test suite typically covers the happy path through the *new* entry point while the *old* one
quietly breaks. Common cases: a new checkout flow vs. the existing mobile checkout; a new API
endpoint vs. a legacy endpoint; changes to shared business logic reached by multiple callers.

## 4. Abstractions & cognitive load

Good abstractions let you reason about the system without holding every detail in your head. But
abstraction isn't free, and it fails in two distinct ways:

- **The wrong abstraction** groups things that don't actually belong together — usually by
  noticing a surface-level similarity and abstracting before the real shape of the problem is
  understood. It then accretes flags and `if (specialCase)` branches to handle real-world
  variation. Sandi Metz's rule applies: **prefer duplication over the wrong abstraction.** A
  shared utility that has grown a forest of special-case conditionals is a candidate for being
  broken apart. (The WET — "write everything twice" — argument is the counterweight to over-DRY.)
- **Premature abstraction** extracts code before the domain is understood well enough to know the
  right shape. Rule of thumb: the first time you write something you don't know enough; the
  second time you see a pattern; the third time you can abstract well. Watch for PRs that extract
  a utility/hook/base class from a **single** use case — that's usually premature.

As a reviewer, ask: does this abstraction *reduce* cognitive load, or does it add a layer of
indirection that makes the code harder to follow without actually simplifying anything?

## 5. Complexity disguised as elegance

Some code reads as clever when it's just complex. Long one-liners, deeply nested ternaries,
chained transforms on ambiguous data, and "magic" via implicit framework behavior all reduce
readability in the name of brevity. Ask whether the clever version is actually easier to
understand than the simple version would be. Usually it isn't.

## 6. Error handling & edge cases — trace the unhappy paths

**The most common reviewer mistake is following the happy path and stopping there. Bugs live in
the branches not taken.** For every piece of logic, ask:
- What happens when this **fails**?
- What happens when data is `null`, empty, malformed, or unexpected?
- What happens when this is called **out of order, twice, or concurrently**?
- What happens when the network is slow or unavailable?
- What happens when the user does something unexpected at exactly the wrong moment?

Failure paths that crash silently, return wrong data without erroring, or leave state
inconsistent are the hardest production bugs to diagnose. Catch them here.

## 7. State consistency

Whenever a change updates **multiple things** — several state variables, a database *and* a
cache, server *and* client state — ask what happens if the update fails **halfway through**. Is
there a case where `userProfile` updates but `userPermissions` doesn't? Where the DB is written
but the cache isn't invalidated? Partial updates that leave inconsistent state are the source of
some of the most confusing production bugs.

## 8. Security — part of every review

You don't need to be a security specialist; you need to ask the questions routinely:
- **Input handling** — Is user-controlled data validated before use? Is it interpolated into SQL
  queries, shell commands, or HTML? (SQL injection, XSS, command injection still happen from
  missed validation.)
- **Authentication & authorization** — Does a new endpoint/action check permissions? Is the
  authz check in the right place — **server-side**, not just client-side?
- **Sensitive data exposure** — Is anything logged that shouldn't be? Are secrets, tokens, or PII
  passed through URLs, stored in plaintext, or serialized into client-visible state?
- **Dependency surface** — Does the change add a third-party dependency? Is it well-maintained?
  Does it introduce known vulnerabilities?

## 9. Performance — be specific or stay quiet

"This might be slow" is rarely useful. When you raise a perf concern, name the **mechanism**:
N+1 queries, unnecessary re-renders, synchronous work on the main thread, unbounded caching,
missing pagination. Then ask whether it's **theoretical or measurable** — sometimes the apparent
problem is inconsequential in practice and the fix costs more than it saves. But don't dismiss a
real problem as premature: if an N+1 will clearly hurt at any reasonable scale, say so.

## 10. Observability — can you debug this at 2am?

Ask whether the code can be debugged in production when something goes wrong:
- Are errors caught and logged in a way that will actually surface them?
- Are meaningful events instrumented (analytics, monitoring, structured logging)?
- Are error messages useful, or will an on-call engineer get a generic "something went wrong"
  with no information?

Missing observability is the kind of tech debt that doesn't bite until it's urgent.

## 11. Tests — review them as carefully as the code

A PR *with* tests is not automatically better-tested — it depends entirely on what the tests
test. Watch for:
- **Tests that test the implementation, not the behavior** — mocking everything and asserting a
  specific function was called proves the code runs a specific way, not that the feature works.
  It breaks when the implementation changes: maintenance cost without safety value.
- **Tests that only cover the happy path** — valid data, network succeeds; nothing about
  failure, empty, or malformed input.
- **Tests that would pass even if the code were wrong** — e.g., `expect(result).toBeDefined()`
  on a function that always returns something tests nothing meaningful.

Don't reject a PR just because coverage isn't perfect. Ask: **given the risk profile of this
change, is the coverage appropriate?** A one-line copy fix and a new payment flow have very
different expectations.

## 12. Documentation — part of the PR

Interfaces, non-obvious behavior, architectural decisions, and anything a future developer would
need but can't infer from the code belong in comments, docstrings, or external docs — **in the
same PR that introduces them.** A particularly valuable form is the **decision record**: a brief
note on *why* the code was written this way rather than the more obvious alternative. That's the
document that prevents future Chesterton's Fence violations.
