---
name: systematic-debugging
description: >-
  Use at the first sign of a bug, test failure, flaky test, build failure, performance
  regression, or any behaviour you cannot explain — and before proposing or applying any
  fix. Covers the cases where guessing is most tempting: an obvious-looking one-line fix, an
  emergency, a failure deep in a stack trace, a multi-component system where the broken
  layer is unknown, and a bug where two or more fixes have already failed.
---

# Systematic Debugging

A fix applied before the cause is known is a guess. A guess that makes the symptom go away
is worse than one that does not, because it ends the investigation while the defect is
still in the code.

---

## The Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

Phase 1 incomplete means no fix may be proposed — not applied, not suggested, not sketched.

**Violating the letter of this process is violating the spirit of it.** "Here are the main
problems" followed by a list of changes is a proposal.

---

## When to use it

Any technical issue: test failures, production bugs, unexpected behaviour, performance
problems, build failures, integration failures, flaky tests.

Especially when:

- you are under time pressure — pressure is what makes guessing feel rational
- a quick fix looks obvious
- you have already tried a fix and it did not work
- you do not fully understand the failure

Do not skip because the issue seems simple. Simple bugs have root causes too, and the
process is fast on them.

---

## Phase 1 — Root cause investigation

Complete every step before moving on.

**1. Read the error and the full stack trace.** Not the first line. The whole trace, the
line numbers, the file paths, the error codes, and the warnings above it. Errors frequently
contain the answer, and the step that gets skipped is reading them.

**2. Reproduce it consistently.** Exact steps. Every time, or how often. If it is not
reproducible, gather more data — do not substitute a guess for a reproduction. An
unreproducible bug cannot be shown fixed.

**3. Check what changed.** Recent commits, the working diff, new dependencies, config
changes, environment differences between where it works and where it does not.

**4. Instrument the component boundaries before proposing anything.** In any system with
more than one component (CI → build → signing, API → service → database, loader → parser →
renderer), you cannot reason about which layer fails. Measure it:

```
For EACH component boundary:
  - log what data enters
  - log what data exits
  - verify environment and config actually propagated
  - capture state at that layer

Run once. Read the evidence. It names the failing component.
Then investigate that component only.
```

```bash
# Layer 1 — workflow: is the secret present at all?
echo "IDENTITY: $([ -n "${IDENTITY:-}" ] && echo SET || echo UNSET)"

# Layer 2 — build script: did it propagate into this process?
printenv IDENTITY >/dev/null 2>&1 && echo "IDENTITY in env" || echo "IDENTITY missing"

# Layer 3 — signing script: is the prerequisite state there?
security find-identity -v

# Layer 4 — the operation itself, verbose
codesign --sign "$IDENTITY" --verbose=4 "$APP"
```

This distinguishes "the secret is missing" from "the secret is present and the next layer
drops it" — two bugs with identical symptoms and no shared fix.

**5. Trace the data flow backward.** When the error surfaces deep in the call stack, the
place it surfaces is rarely the place it originates. Ask what passed the bad value, then
what passed it to that, until you reach where it was created. Fix there.

Full technique: `references/root-cause-tracing.md`.

---

## Phase 2 — Pattern analysis

**1. Find working examples in this same codebase.** Something similar almost certainly
works. That working case is your control.

**2. Read the reference implementation completely.** If you are following a pattern, a
library's example, or a sibling module — read every line. Skimming produces a partial
implementation that fails for a reason the skipped lines explain.

**3. List every difference between working and broken.** Every one, however small. "That
can't matter" is the assumption that costs the next two hours.

**4. Map dependencies and assumptions.** What other components does this need? What config,
environment, ordering, or platform does it assume? Which of those assumptions holds in the
working case and not in the broken one?

---

## Phase 3 — Hypothesis and testing

**1. State ONE hypothesis, in writing.** "I think X is the root cause because Y." Specific,
falsifiable. One. Writing it down is what prevents the shotgun fix, because a written
hypothesis can be shown wrong.

**2. Test it with the smallest possible change.** One variable. The change exists to answer
the question, not to fix the bug.

**3. On failure, form a NEW hypothesis.** Do not stack another fix on top of the last one.
Stacked fixes make the system harder to reason about and hide which change did what. Revert
the failed test change before trying the next.

**4. When you do not know, say so.** "I don't understand why X happens" is a usable
statement. A confident wrong explanation is not. Go read more, or ask.

---

## Phase 4 — Implementation

**1. Write a failing test that reproduces the bug BEFORE fixing it.** Simplest possible
reproduction; an automated test where a framework exists, a one-off script where it does
not. Without it you cannot distinguish "fixed" from "no longer reproducing today". The
quality bar for that test is in `writing-good-tests`.

**2. Implement ONE fix, at the root cause.** No "while I'm here" improvements, no bundled
refactoring, no second fix for the thing you noticed on the way. A bundled diff cannot be
reverted selectively when it turns out to be wrong.

**3. Verify.** The target test passes, the project suite is still green, the original
symptom is actually gone.

**REQUIRED BACKGROUND:** `verification-before-completion`. A fix is not fixed until a fresh
run says so in the same message as the claim.

**4. If the fix did not work, count the attempts.**

```
Fewer than 3 failed fixes → back to Phase 1 with the new information
3 or more failed fixes    → stop; this is an architecture problem
```

**5. Three failed fixes is not a failed hypothesis. It is a wrong design.**

The pattern that identifies it:

- each fix reveals new shared state or coupling somewhere else
- each fix creates new symptoms in a different place
- the next fix would require "massive refactoring"

Stop and question the fundamentals with the user: is this pattern sound, or are we keeping
it through inertia? Should the design change instead of absorbing another fix? Do not
attempt fix #4 before that conversation.

---

## Red flags — stop and return to Phase 1

- "Quick fix for now, investigate later"
- "Just try changing X and see if it works"
- "Let me make several changes and run the tests"
- "Skip the test, I'll verify manually"
- "It's probably X, let me fix that"
- "I don't fully understand this, but this might work"
- "The reference is long, I'll adapt the pattern"
- Listing fixes before tracing the data flow
- "One more fix attempt" after two have failed
- Each fix revealing a new problem somewhere else

## Redirections that mean you skipped a phase

When the user says any of these, you are already off-process. Return to Phase 1.

| They say | You did |
|----------|---------|
| "Is that actually happening?" | Assumed a behaviour instead of verifying it |
| "Will that show us anything?" | Proposed a change where instrumentation was needed |
| "Stop guessing" | Proposed fixes without understanding the failure |
| "Think harder about this" | Treated symptoms where the design is the issue |
| "Are we stuck?" | Been iterating without new information |

---

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "The issue is simple, the process is overkill" | Simple bugs have root causes too; the process takes minutes on them. |
| "It's an emergency, there's no time" | Systematic debugging is faster than guess-and-check thrashing. That is why it exists. |
| "Let me try this one thing first, then investigate" | The first fix sets the pattern for the whole session. |
| "I'll write the test after I confirm the fix works" | Then you never saw it fail, so you cannot prove it is fixed. |
| "Several changes at once saves round trips" | You lose the ability to tell which change did anything. |
| "The reference is too long, I'll adapt it" | Partial understanding of a pattern guarantees a bug inside it. |
| "I can see the problem" | You can see the symptom. They are not the same object. |
| "One more fix attempt" (after two failures) | Three failures is an architecture signal. Question the design. |
| "The error message is boilerplate" | Read it anyway. It frequently contains the answer. |
| "It passes on retry, so it's flaky, not broken" | Flakiness is a bug with a timing cause. Find it. |

---

## Quick reference

| Phase | Activities | Done when |
|-------|-----------|-----------|
| **1. Root cause** | Read the error, reproduce, check changes, instrument boundaries, trace backward | You can state WHAT fails and WHY |
| **2. Pattern** | Find a working example, read the reference fully, list differences | You can name every difference |
| **3. Hypothesis** | One written hypothesis, smallest test | Confirmed, or replaced by a new one |
| **4. Implementation** | Failing test, one fix at the root, verify | Symptom gone, suite green, evidence shown |

---

## When investigation finds no code-level root cause

Sometimes the cause is genuinely environmental, timing-dependent, or in someone else's
system. That conclusion is only available **after** the phases above, and it comes with
obligations:

1. Document what you investigated and what you ruled out.
2. Implement the appropriate handling — a retry, a timeout, an explicit error.
3. Add logging or monitoring so the next occurrence arrives with evidence attached.

Most "no root cause" findings are incomplete investigations. Before accepting one, name the
step you could still take.

---

## Supporting techniques

- `references/root-cause-tracing.md` — trace a bug backward through the call chain to the
  original trigger, and add stack-trace instrumentation when manual tracing dead-ends.
- `references/defense-in-depth.md` — four-layer validation, applied **after** a root cause
  is found for a bug that already shipped. Read its scope section before using it.
- `references/condition-based-waiting.md` — replace arbitrary delays with condition polling
  when the bug is a flaky or timing-dependent test.
- `bash scripts/find-polluter.sh '<path-to-check>' '<test-glob>' '<test-command>'` — bisects
  test files to find which one leaves unwanted state behind.

## Related

- `chestertons-fence` rule — governs removal. This skill governs diagnosis. When the fix is
  "delete this", both apply: you must know why the code exists before it goes.
- `writing-good-tests` — the quality bar for the Phase 4 reproduction test.
- `receiving-code-review` — when the bug report arrived as review feedback.

---

## Attribution

Adapted from the `systematic-debugging` skill in `obra/superpowers` (MIT, © 2025 Jesse Vincent).
