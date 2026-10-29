---
name: review-fix-loop
description: >-
  The bounded loop that turns review findings into verified fixes instead of endless
  churn: how many rounds it gets, who fixes in each, what a re-review is allowed to look
  at, and who decides when it will not converge. Use this skill when a review comes back with findings that must be fixed,
  when a fix has been made and needs verifying, when a review loop is on its third or
  fourth pass, when a reviewer keeps surfacing new issues every round, when you are about
  to dispatch fixers for a batch of findings, or when you think a finding is simply wrong.
---

# Review → fix loop

An unbounded loop is the default failure. Their loop was literally "repeat until
approved", each re-review a fresh full review of the whole diff, so a nondeterministic
reviewer surfaced new findings every round instead of verifying fixes: implement, review,
fix, review, review, fix, review. **Review-loop count was the biggest run-to-run cost
variance they measured.** The loop below is bounded by construction.

**REQUIRED BACKGROUND:** load `receiving-code-review` — it governs how findings are read
and which ones are actually true before any of them enters this loop.

## What enters the loop

Enters: a spec gap, any Critical or Important finding, and any "cannot verify from the
diff" item you checked yourself and confirmed is a real gap.

Does not enter:

- **Minor findings.** Ledger each one as it arrives (`Task <N>: minor (deferred): <one
  line>`) and hand that list to the final review so it can triage what must be fixed
  before merge. A roll-up nobody reads is a silent discard.
- **A finding that contradicts what the plan's text mandates.** The plan's authority is
  not the loop's business. Surface it to the user with both sides — the finding and the
  plan text that requires the thing — and ledger the outcome.

## Five rounds, and who fixes in each

A round is **one fix dispatch plus one scoped re-review**. Five rounds maximum per task.

| Rounds | Fixer | Why |
|---|---|---|
| 1–3 | **Resume the original implementer.** Send the open findings verbatim. | Its context is intact: it knows the task, the code, and its own choices. Ownership beats a drive-by patcher, and a fresh fixer rebuilds the whole context first. |
| 4–5 | **A fresh implementer, one tier up.** Hand it the brief path, the report-file path, the open findings, and the framing: "a prior implementer attempted this task N times; you own it now. Read the report file for what was tried." | A loop that survives three resumes usually means the implementer cannot see its own problem. Fresh eyes and a capability bump in one move. |

If your harness cannot send another message to a live subagent, a "resume" is a fresh
dispatch carrying the brief path, the report-file path, and the findings — the report file
is the persistent memory either way.

**Never fix the findings yourself in the dispatching session.** Your fixes skip review
entirely, and they fill the context you need for coordination.

Every round, either way: the implementer fixes, re-runs the tests covering the amended
code, appends its fix report to the **same** report file, and returns the short status
contract. Before re-dispatching the reviewer, confirm the fix report contains the covering
tests, the command run, and its output. Name the covering test files in the fix message —
a one-line fix does not need the whole suite.

## The re-review is scoped to the findings

This is what makes the loop converge. The re-reviewer is a different contract from a
reviewer, so give it its own prompt rather than reusing the full-review one.

**It receives:** the brief, the updated report file, the original findings list copied
verbatim, and a diff of the fix range only (the base being the head the previous review
saw).

**It returns:**

1. **A verdict per finding, in order** — `ADDRESSED` or `NOT ADDRESSED`, each with
   `path:line` evidence. "Attempted" is not addressed: the specific defect must no longer
   exist.
2. **New breakage inside the fix diff**, with severity. New Critical or Important breakage
   joins the open findings list.
3. **Out-of-scope observations** — anything noticed entirely outside the fix diff. These
   are non-blocking, they go to the ledger for the final review, and they never extend the
   loop.

A fresh full review each round is the churn engine. The broad whole-branch review at the
end is the safety net for everything a scoped re-review deliberately does not look at.

## One omnibus fixer per findings batch

When a review returns a batch of findings — a final whole-branch review especially —
dispatch **one** fixer with the complete findings list. Never one fixer per finding: each
rebuilds context from zero and re-runs suites, and one of their real final-review fix
waves **cost more than all of that run's tasks combined.**

Then run exactly one scoped re-review over the fix wave.

## No early exit

You do not adjudicate before the cap. Deciding a finding is wrong in round two to end the
loop is pre-judging with a different name, and it reopens the hole that "do not flag this"
instructions opened in their runs.

Round five's re-review is the only place adjudication happens.

## At the cap

Stop dispatching. Judge each still-open finding against the brief, the plan, and the
cross-task context the reviewer never had:

- **The reviewer is wrong, or the point is contestable** → park it:
  `Task <N>: parked — <finding> — Ruling: <why the code stands>`. Both sides reach the
  final review.
- **Real, but nothing downstream builds on it** → park it the same way, with a ruling that
  says it is real and deferred.
- **Real and load-bearing** — a later task builds on it, or it reveals a plan defect →
  this is the end of the loop's authority. Ledger it and take it to the user. Parking a
  structural failure silently lets every dependent task build on it.

Every adjudication is a ledger entry. A silent discard is forbidden.

## Escalate to the user when any of these is true

- **Five rounds are spent** and findings remain open.
- **The same issue has been disputed three times** — reviewer flags, fixer declines or
  "fixes" it unchanged, reviewer flags again. Three passes on one disagreement is not
  convergence.
- **Two reviewer outputs came back malformed** — no per-finding verdicts, no `path:line`
  evidence, or prose where the contract asked for a list. You cannot run a gate you cannot
  read.

Escalation is: the open findings, what each round changed, your ruling on each, and what
it costs if you are wrong.

## Reviewers are advisory, not blocking

A reviewer can be wrong. Their cheap reviewers argued *for* defects — a DRY violation
praised as YAGNI, an assert-nothing test called plan-compliant. The loop's job is to make
a reviewer's claim cheap to check and expensive to ignore, not to make it binding.

So: every finding is read and answered, none is dropped without a ledgered ruling, and the
ruling at the cap belongs to you — never to the reviewer, and never to the implementer
that wrote the code.

## Common Rationalizations

| Excuse | Reality |
|---|---|
| "Close enough on spec compliance" | A reviewer that found spec gaps found that it is not done. Fix, or reach the cap and adjudicate. Those are the only two exits. |
| "I'll fix it myself, dispatching is overhead" | Controller fixes skip review and pollute the context you coordinate from. Resume the implementer. |
| "One more round will converge" | Past the cap, rounds do not converge — the failure is structural. Adjudicate and route. |
| "The reviewer will just find something new anyway" | That is the symptom of unscoped re-reviews. A scoped re-review verifies fixes; new findings on untouched code go to the ledger. |
| "This finding is obviously wrong, I'll drop it" | You adjudicate at the cap, and every ruling is a ledger entry. Silent discards are forbidden. |
| "The fix was small, skip the re-review" | Unreviewed fixes are how regressions land. Every round ends with a scoped re-review. |
| "One fixer per finding is cleaner to track" | Each rebuilds context and re-runs suites. Their per-finding fix wave cost more than every task in the run. |
| "Reviews slow the loop down" | The loop without reviews is unverified churn. The reviews are its brakes and its steering. |
| "Round 3 failed, I'll resume it once more" | Rounds 4–5 are a fresh implementer one tier up. A fourth resume is the same blind spot getting another turn. |

## Related

- `receiving-code-review` — required background: how to read findings and judge which are true.
- `subagent-dispatch` — the dispatch contract, the evidence rule, and why you never tell a reviewer what to skip.
- `execution-ledger` — where every round, park, and ruling is recorded.
- `model-tier-economics` — why rounds 4–5 go up a tier, and why a reviewer is never on the cheapest tier.

## Attribution

Adapted from `obra/superpowers` (MIT, © 2025 Jesse Vincent); the five-round cap, the scoped
re-review contract, and the cost findings quoted here are their measurements.
