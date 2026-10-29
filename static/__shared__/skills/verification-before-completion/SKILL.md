---
name: verification-before-completion
description: >-
  Use before claiming any work is complete, fixed, passing, clean, or ready — and before
  any action that depends on such a claim, such as committing, closing a TODO, moving to
  the next task, or handing a branch back to the user. Also use when a subagent reports
  success, when you are about to write "should work now", and when a test or build result
  you are relying on came from an earlier message rather than this one.
---

# Verification Before Completion

A completion claim is a factual assertion about the state of the repository. It is either
backed by output you just read, or it is a guess wearing the costume of a report.

**Violating the letter of this rule is violating the spirit of it.** Paraphrases,
synonyms, and implications of success are all claims.

---

## The Iron Law

```
NO COMPLETION CLAIM WITHOUT FRESH VERIFICATION EVIDENCE IN THIS MESSAGE
```

If the verification command did not run in this message, you cannot say it passes. Not
"still passes", not "should pass", not "passing as of the last run".

---

## The Gate Function

```
BEFORE stating any status, or expressing satisfaction with the work:

1. IDENTIFY   Which command proves this specific claim?
2. RUN        Execute it in full. Fresh. No subset, no cached result.
3. READ       The whole output. The exit code. The failure count.
4. VERIFY     Does the output confirm the claim?
                 No  → report the actual state, with the output
                 Yes → state the claim together with the evidence
5. CLAIM      Only now.

A skipped step makes the output a claim about a command you did not run.
```

---

## What each claim requires

| Claim | Requires | Not sufficient |
|-------|----------|----------------|
| Tests pass | The test command's output, 0 failures | A previous run, or "should pass" |
| Linter clean | The linter's output, 0 errors | A partial check, extrapolation from one file |
| Build succeeds | The build command, exit 0 | The linter passing |
| Formatting clean | The formatter's check output | Having formatted the files you remember touching |
| Bug fixed | The original symptom retested, now absent | The code changed where you believed the bug was |
| Regression test works | Red-green confirmed: it fails without the fix, passes with it | It passes once, with the fix in place |
| Agent completed | A VCS diff showing the changes | The agent reporting success |
| Requirements met | A line-by-line walk of the request against the diff | Tests passing |
| Nothing else broke | The project suite, not your file | Your new test being green |

The `Agent completed` row is the one most often skipped. A subagent's report is a claim by
another agent, not evidence. Read the diff.

---

## Evidence you cannot see is not evidence that does not exist

When reported evidence looks truncated, garbled, or you cannot find the output it refers
to: re-read it at its stated path, or ask for it. **Re-running the suite to regenerate what
you failed to read is not verification** — it replaces someone else's result with yours and
tells you nothing about whether their claim was true. Illegibility of evidence is not
invalidation of it. If it is genuinely missing, report that as a gap.

---

## Verify the project suite, not your slice of it

A green run of the test you wrote is not a green suite.

```
Before calling a change done, run the project's own test command
(bare `pytest`, `npm test`, `cargo test` — whatever the repo uses),
even when the task named a single file or a single module.
```

**A scope statement bounds the deliverable, not your verification.** Any failure that run
shows — including one you did not cause — goes in your report by name. A red test you
watched scroll past and did not mention is a report falsified by omission.

Pre-existing failures are reported as pre-existing, with the evidence that they are. They
are never reported as nothing.

---

## Patterns

**Tests**
```
RIGHT: [run the test command] [read: 34/34 passed] → "All 34 tests pass."
WRONG: "Should pass now." / "Looks correct." / "That fix handles it."
```

**Regression test for a bug**
```
RIGHT: write test → run (fails) → apply fix → run (passes) → report both runs
WRONG: "I've added a regression test." (never seen red, so never proven to fail)
```

**Build**
```
RIGHT: [run build] [read: exit 0] → "Build succeeds."
WRONG: "The linter is clean." (a linter does not compile)
```

**Requirements**
```
RIGHT: re-read the request → checklist each item → verify each against the diff
       → report completion or name the gaps
WRONG: "Tests pass, so the task is done."
```

**Delegated work**
```
RIGHT: agent reports success → read the VCS diff → verify the changes exist and match
       → report the actual state
WRONG: relay the agent's report as your own finding
```

---

## Red flags — stop and run the command

- "should", "probably", "seems to", "ought to"
- Satisfaction expressed before output read: "Great!", "Perfect!", "Done!", "All set!"
- About to commit, open a PR, or hand the branch over without a fresh run
- Relaying a subagent's success report
- A partial check standing in for the full one
- "Just this once" — or being tired and wanting the task over
- Any wording that implies success when no verification has run

Commit, PR, and hand-off appear here as triggers that the gate must run before them. They
are not instructions to perform any of those actions.

---

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Should work now" | Run the command. |
| "I'm confident" | Confidence is not evidence. |
| "Just this once" | The exception is the whole failure mode. |
| "The linter passed" | A linter is not a compiler. |
| "The agent said success" | Read the diff. |
| "I ran it two messages ago" | The code changed since. Run it again. |
| "A partial check is enough" | A partial check proves the part you checked. |
| "That failure was already there" | Then report it as pre-existing, by name. |
| "The task only named one file" | Scope bounds the deliverable, not the verification. |
| "I'm tired" | Exhaustion is not an exemption. |
| "I worded it differently, so the rule doesn't apply" | Spirit over letter. Any implication of success is a claim. |
| "The evidence looked truncated so I re-ran it" | Re-read it, or ask. Regenerating is not verifying. |

---

## Applies to

Every variation of a success, completion, or correctness claim: exact phrasings,
paraphrases, synonyms, implications, and expressions of satisfaction. Also every status
written into a TODO list, plan, or report the user will act on.

---

## Related

- `writing-good-tests` — a green suite of tautological tests is evidence of nothing.
- `systematic-debugging` — its Phase 4 closes through this gate.
- The `Verification Gate` in `AGENTS.md`, which this skill operationalizes.

---

## Attribution

Adapted from the `verification-before-completion` and `test-driven-development` skills in
`obra/superpowers` (MIT, © 2025 Jesse Vincent).
