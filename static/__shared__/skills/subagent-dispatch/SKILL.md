---
name: subagent-dispatch
description: >-
  The operational discipline that keeps subagent dispatch from costing more than it
  saves — which work is safe to run concurrently, which must stay sequential, what a
  dispatch is allowed to carry, and what a worker owes back. Use this skill when about
  to spawn one or more subagents, when deciding whether to parallelize, when several
  test files or subsystems are failing at once, when a dispatched agent comes back
  BLOCKED or NEEDS_CONTEXT, when a worker has spawned helpers or reviewers of its own,
  when waiting on dispatched children, or when merging what they returned.
---

# Subagent dispatch

A dispatch is not free and not cheap. In their measured runs a dispatch-plus-review
cycle carried roughly **$0.60–1.00 of fixed overhead** before any useful work happened,
and a controller's own coordination was about half the dollars of a whole run. Dispatch
buys exactly two things: a clean context for the worker, and a reviewer who did not
write the code. Everything below protects those two properties and refuses to pay for
anything else.

**REQUIRED BACKGROUND:** load `model-tier-economics` before your first dispatch — every
dispatch sets a model and an effort, and that skill holds which tier each role takes.

## Parallelize only when every one of these holds

Check them against the actual work, not against how the work feels:

- **2+ tasks that are genuinely independent** — one task's outcome cannot change another's.
- **No shared state** — no two workers write the same file, table, port, fixture, or branch.
- **No sequential dependency** — none of them consumes what another produces.
- **Each is understandable alone** — a worker can succeed knowing its own task and nothing
  about its siblings.

Typical fit: three test files failing for three unrelated root causes; three independent
subsystem investigations; a fan-out read of a large codebase where you want conclusions,
not file dumps.

To run them concurrently, issue every dispatch **in the same message**. One dispatch per
message is sequential execution with parallel intent.

## Never parallel

- **Related failures.** If fixing one might fix the others, they are one investigation.
  Dispatching three agents at one root cause buys three conflicting diagnoses.
- **Work that needs whole-system state.** If understanding requires holding the whole
  picture, splitting the picture is the failure.
- **Exploratory debugging.** You do not yet know what is broken, so you cannot scope a
  worker. Investigate first, then dispatch what the investigation isolated.
- **Agents that would touch the same files.** Concurrent edits to one file produce a merge
  you have to redo by hand.
- **More than one implementation subagent, ever.** Parallelism is for investigation and
  for independent fixes — never for concurrent writes. Two implementers in flight is a
  conflict you pay for twice: once in the collision, once in the re-review.

## The brief is a file; the dispatch carries its path

Write the worker's requirements to a file and put the **path** in the dispatch. Exact
values — numbers, magic strings, signatures, test cases, expected output — live only in
that file, so there is one copy and no paraphrase.

The dispatch itself carries, and carries nothing else:

1. One line on where this work fits.
2. The brief path, introduced as "read this first — it is your requirements, with the
   exact values to use verbatim".
3. Interfaces and decisions from earlier work that the brief cannot know.
4. Your resolution of any ambiguity you already noticed in the brief.
5. The report-file path and the report contract below.

**Never paste session history into a dispatch.** One of their real dispatches reached
**42k characters, 99% of it pasted prior-task history**. Everything you paste into a
dispatch — and everything a worker prints back to you — stays resident in your context
for the rest of the session and is re-read on every later turn. Hand artifacts over as
files, and have workers write their detail to a report file rather than into your context.

## Every dispatch names a model and a reasoning effort

Both, explicitly, every time:

- An **omitted model inherits the session's model** — usually the most expensive one. Their
  guidance decayed mid-session exactly once and inherited the top tier for 17 dispatches,
  about +$5 on one run.
- Setting **`model` without an effort resets effort to that model's default**, not to
  yours. A quiet downgrade of reasoning is the hardest failure to attribute later.

This applies at every depth, including dispatches issued by something you dispatched.

## Workers do not dispatch

State it in the brief: the worker does all of its own work, spawns no helpers, and above
all spawns no reviewer. Self-review means reading your own diff. Review arrives from the
dispatcher, after the report.

Their evidence: **9 of 9 observed depth-2 spawns were worker-issued reviewers, and all 9
duplicated the review the dispatcher ran anyway** — a full extra review seat per task,
whose verdict counts for nothing in the process. "An independent review would strengthen
my report" means the review is already scheduled.

## The report contract

Require these four statuses, verbatim, and give each its defined response. A worker that
cannot name its state will narrate instead, and you will act on prose.

| Status | What it means | Your response |
|---|---|---|
| `DONE` | Work complete, no doubts | Review it. Never skip the review because the report sounds confident. |
| `DONE_WITH_CONCERNS` | Complete, with doubts stated | Read the concerns first. Correctness or scope concerns get resolved before review; observations get recorded and the review proceeds. |
| `BLOCKED` | Cannot complete | Diagnose the blocker: missing context → supply it; needs more reasoning → re-dispatch one tier up; too large → split it; the plan is wrong → rule on the correction and carry the ruling into the re-dispatch. |
| `NEEDS_CONTEXT` | Information was not provided | Supply exactly what is missing and re-dispatch. |

Two rules bind both sides. The worker's: **"Never silently produce work you're unsure
about."** Yours: never ignore an escalation, and **never force the same model to retry
unchanged** — if it said it was stuck, something has to change before it runs again.

The report itself is short — status, commits, a one-line test summary, concerns, and the
report-file path. The detail goes in the file.

## Batch small same-shape work

When several items are each a small independent edit of the same kind — the same one-line
fix, constant, or field addition across files — compose **one** brief listing every file
and its change, send the batch to a single worker, and review the diff as one unit. At
~$0.60–1.00 of fixed overhead per cycle, one dispatch per trivial edit is the overhead
eating the work.

Reserve one-dispatch-per-item for work that needs its own judgment, its own tests, or its
own review surface.

## Do not pre-judge a reviewer

Never tell a reviewer what not to flag. If you believe a finding would be a false
positive, let the reviewer raise it and adjudicate it in the loop.

**Stop if the prompt you are writing contains** "do not flag", "don't treat X as a
defect", "at most Minor", or "the plan chose". You are buying yourself out of a review
loop. Their live run caught a controller inventing a plan constraint to steer a reviewer
off a planted DRY violation — the defect then shipped.

## Evidence or discard

Put the contract in the brief: **every finding cites `path:line` with a short quote of the
code it is about.** Findings without one are dropped, and the brief says so.

Their reviewers passed defects the prompt had pointed them at by answering checks with
bare "yes" — an accessible-name check and a temp-dir-cleanup check both got unsupported
affirmatives while the defect sat in the reviewed diff. A citation is the cheapest proof
that reading happened.

## Waiting on children

- **Never poll with short timeouts.** **67.1% of their waits timed out** in the measured
  corpus; each timeout is a turn that re-reads your whole context and buys nothing.
- **Never sit in one open-ended wait** either. A silent wait makes a stuck or lost child
  indistinguishable from a slow one.
- **While you have local work, do not wait.** Update the ledger, package the next review,
  read a report. Child results arrive on their own.
- **When genuinely idle, wait in bounded stretches** of five to ten minutes. Between
  stretches, reconcile your live children: list them and chase any that finished without
  reporting. Bounded stretches keep nearly all of a long wait's efficiency while
  guaranteeing a lost child surfaces in minutes rather than at the end of the session.

## Merging what comes back

1. **Read each summary.** You cannot integrate what you did not read.
2. **Check for overlapping edits.** Two workers that touched one file is a conflict even
   when git merged it cleanly.
3. **Run the full suite**, not the per-worker tests they each already ran.
4. **Spot-check the diffs.** Agents make *systematic* errors — the same wrong idea applied
   uniformly, which no individual test catches.

## Common Rationalizations

| Excuse | Reality |
|---|---|
| "These failures look related but I'll fan out anyway, it's faster" | Related failures are one root cause. You get N diagnoses of one bug and N conflicting fixes. |
| "Two implementers in parallel will halve the wall clock" | They write to the same tree. You pay for the collision and then review it twice. |
| "Pasting the prior tasks' state gives the worker useful background" | It gives it 42k characters of your history, resident for the rest of your session. The brief is the background. |
| "The default model is fine for this one" | The default is inherited, not chosen — usually the most expensive tier. Name the model and the effort. |
| "The worker spawning its own reviewer is free extra assurance" | It is a duplicate seat on the same diff; 9/9 observed depth-2 reviewers were exactly that. Flag it as a defect. |
| "I'll tell the reviewer to skip that one — I know it's intentional" | That is pre-judging, and it is how a planted defect shipped. Let it be raised, then adjudicate. |
| "One dispatch per file is cleaner to track" | Each costs $0.60–1.00 of fixed overhead. Same-shape edits go in one brief. |
| "I'll just poll every 30 seconds so I notice immediately" | Two thirds of their waits timed out. Do local work, then wait in bounded stretches. |
| "All agents reported success, so the merge is done" | Agents make systematic errors. Full suite, then spot-check. |

## Red flags

Stop and fix the dispatch if you catch yourself:

- composing a dispatch longer than the brief it points at
- writing "state after tasks 1–3" into a prompt
- dispatching a reviewer without a diff file
- dispatching without a model line
- re-dispatching an unchanged task to the model that just said BLOCKED
- accepting a finding with no `path:line`
- having a worker report in prose instead of one of the four statuses

## Related

- `model-tier-economics` — which tier each dispatched role takes, and why cheap judgment costs more.
- `execution-ledger` — the record that keeps a compacted dispatcher from re-dispatching finished work.
- `review-fix-loop` — the bounded loop that consumes review findings.
- `multi-agent-patterns` — architecture-level theory (supervisor, swarm, hierarchical, isolation mechanisms). This skill is the session-level discipline; that one is the design space.
- `filesystem-context` — why routing results through files beats message passing.

## Attribution

Adapted from `obra/superpowers` (MIT, © 2025 Jesse Vincent), whose measurements — dispatch
overhead, the 42k-character dispatch, 9/9 worker-issued reviewers, the 67.1% wait-timeout
rate — are cited here as theirs.
