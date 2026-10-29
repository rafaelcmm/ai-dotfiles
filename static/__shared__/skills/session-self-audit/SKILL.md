---
name: session-self-audit
description: >-
  Use when a coding-agent session went wrong and you want to know why —
  repeated work, an ignored plan, stalls and retries, "done"/"fixed"/
  "verified" claims with nothing behind them, conflicting instructions, or
  a skill that should have fired on a turn and didn't. Reads the
  session's own transcript and produces one local markdown report for the
  user; it files nothing, posts nothing, and exports nothing anywhere
  else.
---

# Session Self-Audit

A post-mortem that reads a session's own transcript and reports what was
wasted, evidenced line by line. You report what the transcript shows; you
do not fix the underlying skill or rule, and you do not evaluate the code
the session produced — only the process that produced it.

**Core principle:** every finding cites `path:line` plus a quote. No
citation, no finding.

**This skill produces a local report only.** The output is one markdown
file written to disk for the user to read. It never creates an issue,
never comments anywhere, never pushes, and never packages a shareable
bundle. If asked to turn the report into a bug report or to send it
anywhere, hand the user the report path and stop — that action is theirs,
not this skill's.

## Workflow

Create a todo per step.

1. **Intake.** Ask one question at a time until you can write a
   one-paragraph problem statement: which session(s), the turn range if
   known, what was expected, what happened, and the observable that
   matters — wall-clock, tokens, repeated actions, one specific action.
   "It took too long" is a complaint, not a problem statement; press for
   the observable. Nothing below starts until this is answered.
2. **Locate.** Resolve the session's transcript file(s) to absolute
   paths — the main transcript plus any subagent transcripts — using
   whatever this harness exposes (session tools, configured storage,
   local help or documentation, bounded filesystem inspection). Confirm
   identity with the session id, working directory, timestamps, and
   matching content; recency alone is not confirmation. Before counting
   anything, establish and write down the record meanings you will hand
   to every analyst: what marks a human-typed prompt versus injected
   content, how a tool call matches its result, and whether each usage
   counter is incremental or cumulative. Do not infer a format from a
   different harness, and do not turn a missing field into a zero.
3. **Dispatch analysts in parallel.** One subagent per dimension, each
   given directly in its prompt: the resolved session file paths and the
   record meanings from step 2, any RANGE you're scoping it to,
   `prompts/analyst-common.md`, and one dimension file —
   `cost-and-time.md`, `repeated-work.md`, `plan-adherence.md`,
   `quality-evidence.md`, `stumbles.md`, `request-conflicts.md`,
   `skill-timeline.md`. Split a dimension by turn range when the
   transcript is long. Discard any returned finding without a
   `path:line`.
4. **Synthesize.** Fill every section of `assets/report-template.md` in
   order, write it to a local path, tell the user the path, and show it.

## Quick reference

All seven analysts always run. This table says which to read yourself
first in step 2 and which findings to lead with in the verdict.

| Complaint | Read first, lead with |
|---|---|
| "It took too long" | cost-and-time, stumbles |
| "Why did it do this extra work?" | repeated-work, plan-adherence |
| "Why is it so expensive?" | cost-and-time |
| "It ignored the plan" | plan-adherence, compaction lines first |
| "Skill X never fired" | skill-timeline |

## Hard rules

- **Context safety.** One transcript line can be a megabyte. Follow
  `references/context-safety.md` on every session file, every time.
- **Read-only.** Never modify, move, or delete a session file.
- **Exact paths to subagents.** A subagent's "current session" is its
  own. Pass absolute paths and ids, never a relative reference.
- **Human prompts only.** Hook output, system reminders, and tool results
  are not the user's words. In a subagent transcript, "user" is the
  parent agent.
- **Local report only.** No GitHub issue, no ticket, no comment, no push,
  no export bundle, no zip. One markdown file, on this machine, for the
  user.
- **Intake before analysis.** Nothing in steps 2–4 starts until the
  problem statement is answered. An already-scoped request — one
  specific event, or the exact analysis to run — is itself the statement:
  answer it, then proceed. A whole-session "why" is a complaint, not a
  statement.

## Red flags

| Thought | Reality |
|---------|---------|
| "The problem is obvious, skip intake" | The problem statement scopes everything downstream. Ask. |
| "I'll sweep everything now and ask at the end" | An unscoped sweep spends budget on the wrong question. Ask first. |
| "They want a bug report, so I'll build a bundle" | There is no bundle in this skill. Hand them the report path; they decide what to do with it. |
| "Small, targeted edit, no restructuring needed" | Not your call. Report the evidence; the reader decides. |
| "The price per token is well known" | Numbers you did not compute from the transcript are invented. Cite or drop. |

## Attribution

Adapted from `obra/superpowers`'s `diagnosing-superpowers` skill (MIT,
© 2025 Jesse Vincent). The outward-facing paths — GitHub issue filing and
the scrubbed share-bundle export — were removed entirely; this skill is
local-report-only by design, not by omission.
