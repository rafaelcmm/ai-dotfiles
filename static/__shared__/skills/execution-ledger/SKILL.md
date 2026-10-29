---
name: execution-ledger
description: >-
  The append-only progress record that lets a long multi-step run survive compaction,
  a crash, or a new session without redoing finished work — what the file is, what each
  line must name, and how a resuming agent reads it. Use this skill when executing a
  plan with more than a handful of steps, when dispatching subagents across many tasks,
  when resuming work you do not fully remember, when a progress file already exists in
  the repo, when recording a judgment call made mid-run, or when a run is long enough
  that context will be compacted before it ends.
---

# Execution ledger

Conversation memory does not survive compaction. Their single most expensive observed
failure is a controller that lost its place and **re-dispatched entire completed task
sequences — 269 dispatches for roughly 22 tasks in one session.** Every commit it was
re-making already existed in git.

The ledger is the fix: a file on disk whose lines are true whether or not you remember
writing them.

## Harness task lists are a live view. The ledger is the record.

They answer different questions and neither substitutes for the other:

- A TODO list answers *what is in flight right now*. It is mutable, it is rewritten as
  state changes, and it holds no evidence.
- The ledger answers *what actually happened, and what proves it*. It is append-only, it
  names commits and test commands, and it is readable by an agent that has lost the
  session that produced it.

Keep both. Mark the TODO complete **and** append the ledger line — this is why the ledger
is not redundant with TODO tooling.

## The file

A **git-ignored** progress file, scoped to the plan or workstream it belongs to:

```
.agent-work/<plan-slug>/progress.md
```

Git-ignore it before the first write: put a self-ignoring `.gitignore` containing `*` at
`.agent-work/.gitignore`. Their workspace spec records scratch artifacts committed to a
shared branch and cleaned up twice, plus a report authored on another machine that then
materialized in every fresh worktree — `.gitignore` is powerless once a file is tracked.

**The first line names the plan the ledger belongs to:**

```
# Ledger — plan: docs/plans/2026-10-29-auth-token-rotation.md
```

Identity must be **structural**, not remembered. Their spec records three real cross-plan
contamination incidents in one repo: a follow-up plan read the previous plan's ledger as
its own progress and was told, by a straight-line reading, to skip tasks; a second plan's
briefs silently overwrote the first's at shared default paths; an abandoned stub ledger
outlived both. A per-plan directory plus a plan line in the file makes a foreign ledger
inert — nothing points at it, and anything that reads it can tell.

`git clean -fdx` destroys the whole workspace, because it is git-ignored scratch by
design. If that happens, recover from `git log` — the commits the ledger named are still
there. Do not run `git clean -fdx` to tidy up mid-run.

## One append-only line per completed unit

Each line names what finished, the commits that prove it, and the test command with its
result. A line with no commit range and no test result records a feeling.

```
Task 3: complete (commits 4f2a1c9..8be0d31, `npm test -- auth` 41/41 pass, review clean)
Task 4: complete (commits 8be0d31..1d77aa4, `cargo test` 112/112 pass, 2 minors parked)
Task 5: fix round 2/5 (3 addressed, 1 open — token TTL still hardcoded; commits 1d77aa4..9c04b12)
Task 6: minor (deferred): duplicated error mapping in handlers/*.rs
```

Never rewrite or delete an earlier line. A superseded line gets a later line that
supersedes it; an edited history is a history you cannot trust after compaction.

## Bookkeeping rides in the same tool call as the work

Every tool call is a turn that re-reads your whole context. A ledger append in its own
call doubles the turns for the same record. Append in the same call as the commit, the
same call as the test run, the same message as the TODO update.

## Resume semantics

When a ledger already exists where you are about to work:

- **Its first line names your plan** → lines reading `complete` are done. Do not redo
  them. Resume at the first unit without one. A unit whose last line is a fix round is
  mid-loop: resume that loop at the next round.
- **Its first line names a different plan** → it is another plan's progress. Leave it
  exactly where it is and start your own, fresh, in your own plan's directory.
- **It has no plan line at all** → treat it as foreign. You cannot prove it is yours.

**Trust the ledger and `git log` over your own recollection.** After compaction, what you
"remember" is a summary of a summary; the commits the ledger names are checkable facts.
When the two disagree, check the commits and ledger what you found.

## Rulings

Every judgment call you make on the user's behalf gets a line:

```
Ruling: <what you decided> — <why> — <what it costs if wrong>
```

The third field is the load-bearing one. "What it costs if wrong" is what lets the user
decide whether to look.

**All rulings surface in your final message.** Before you finish, collect every `Ruling:`
line from the ledger into a "Rulings I made" list, in the order you made them, each with
its cost-if-wrong. The list is exhaustive: if the ledger holds a ruling, the list holds
it. That list is the only place these decisions reach the user — **a ruling that dies
with the workspace was a decision made in secret.**

Keep the ledger until the user says the work is finished. It is their record, not your
scratch; deleting it is their call, not a cleanup step.

## Four things stop you

A ruling is for what you can decide. These four you do not decide:

1. An **irreversible or destructive** operation.
2. A **security-sensitive** action.
3. A **side effect outside this worktree** that norms say you ask about first — a merge, a
   push to a shared branch, a publish, anything another person will read and attribute to
   the user.
4. A **plan so broken that every path forward is a guess.**

For those, stop and ask. See the rules `acting-in-the-users-name` (anything carrying the
user's identity: draft it and hand it over, never send it) and `live-account-mutations`
(state that costs money or changes what customers see: plan → dry-run → approval →
verify). The stop list and those rules stack; satisfy all of them.

## What this skill deliberately does not adopt

The source material pairs its ledger with a **continuous-execution policy**: never pause
between tasks, never check in, rule on everything the plan does not answer. **We take the
ledger and not the autonomy.** Our model is supervised: approval, testing, and
documentation gates stay exactly where they are, and the ledger exists to make a
supervised run recoverable — not to replace the supervisor. Do not reintroduce a
no-checkpoint policy on the grounds that the ledger makes checkpoints redundant. It does
not; it makes them cheap.

## Common Rationalizations

| Excuse | Reality |
|---|---|
| "The TODO list already tracks this" | TODOs are a live view with no evidence and no survival. The ledger names commits and test results and is readable by a session that never saw yours. |
| "Ledger bookkeeping is overhead" | It is the only thing that survives compaction. Controllers without one re-dispatched completed sequences — 269 dispatches for ~22 tasks. |
| "I remember what I finished" | Post-compaction recollection is a summary of a summary. The commits are the facts. |
| "There's already a progress file here, I'll continue it" | Check its first line. A foreign ledger told a real session to skip tasks it had never run. |
| "I'll write the ledger up properly at the end" | A run that ends cleanly never needed it. The one that compacts mid-way has nothing. |
| "This decision is too small to ledger" | Then its line is one sentence. Unledgered decisions are invisible to the user, which is the whole failure. |
| "I'll append the ledger line in its own call, it's cleaner" | Each call re-reads your context. Bookkeeping rides with the work. |
| "I'll fix that earlier line now that I know better" | Append the correction. An edited ledger cannot be trusted by the agent that needs it most. |

## Related

- `subagent-dispatch` — the dispatch discipline the ledger records.
- `review-fix-loop` — writes a line per round and adjudicates at the cap.
- `context-compression` / `filesystem-context` — the theory of why on-disk state survives what context does not. This skill is the concrete record format.

## Attribution

Adapted from `obra/superpowers` (MIT, © 2025 Jesse Vincent); the 269-dispatch failure, the
three cross-plan contamination incidents, and the ruling format are their findings.
