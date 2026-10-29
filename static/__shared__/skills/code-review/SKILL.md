---
name: code-review
description: >-
  A thorough, lead-developer-level review of code that is in merge/review state — a GitHub pull
  request, a GitLab merge request, a local branch diff, or one task of a plan being gated before
  the next — reported as what blocks merge, what should change, and what a human must verify.
  Use this skill whenever the user asks to "review" a PR/MR/branch,
  asks "is this ready to merge", pastes a GitHub/GitLab link and wants feedback, asks for a
  "code review" or "diff review", or wants a pre-merge checklist evaluated against real code.
  Trigger it even when the user phrases it casually ("look over this branch", "check my changes
  before I merge") — any request that amounts to reviewing changes before they ship should use
  this skill.
---

# Code Review

A skill for running a rigorous, design-aware code review of changes that are in merge state,
and reporting the results so the author and reviewer know exactly what blocks merge, what is
worth changing, and what still needs a human to verify.

This skill encodes a lead developer's review philosophy: **code review is not a quality gate
that catches typos — it is a collaborative thinking process about design, system behavior, and
long-term maintainability.** The most valuable output of a review is often a question that
starts a conversation, not a list of corrections. Hold that frame the whole way through.

The `code-reviewer` agent is the autonomous executor for this skill: dispatch it to run the
review end-to-end and hand back a report (see *Dispatch economics*); use this SKILL.md directly
when you are the one reading the diff.

---

## The core mental model: debug before you approve

Adopt the posture of a debugger, not a proofreader. **Start from the assumption that this change
is a regression and try to prove yourself wrong.** Reviewers who start at "this is probably fine"
and skim for exceptions rubber-stamp. Assume there is a bug, then trace the data flow, the
control paths and the edge cases until you are satisfied it works.

Two corollaries drive most of the work:

- **Review the change, not the diff.** Bugs live in the interaction between changed and
  unchanged code. A function that now returns `null` under a new condition may be consumed by a
  caller three files away that never expected `null`. The diff shows the change; the bug lives
  elsewhere. Always open surrounding files, callers, and consumers.
- **If you don't understand why it works, don't pass it.** Recreate it — open a sandbox, write a
  small test, trace it by hand. The best review comments begin "I wasn't sure why this worked, so
  I tried it, and found…"

---

## Two review modes — pick one before you start

| Mode | Use when | Reading budget |
|---|---|---|
| **Merge-readiness review** | A PR/MR/branch is judged ready to ship. | Phase 2 in full — surrounding files, callers, consumers. |
| **Task-scoped gate** | One task of a plan is gated before the next; a whole-branch review comes later. | Read the diff and stop. |

If nobody told you which mode you are in, you are in merge-readiness review.

In **task-scoped gate** mode the diff is your view of the change: its context lines *are* the
changed files, so open a changed file separately only when a hunk you must judge is cut off
mid-function — and say so in the report. Inspect code outside the diff only to evaluate a
**concrete named risk**: one focused check per risk, naming both the risk and what you checked.
Cross-cutting changes are legitimate named risks — when the diff changes lock ordering, a
function or API contract, or shared mutable state, checking the call sites is the right method.
Repo-wide crawling with no named risk is the failure this mode prevents: diff-scoped reviewers
measured 6–16 tool calls in 14–65s, while unscoped ones crawled repo-wide in 7 of 8 cases
(worst: 50+ Bash calls, ~200s) — 4–8× the cost on identical tasks.

---

## Workflow

Run these phases in order. Most of a review's leverage comes from the early ones — understanding
the change before reading a single changed line.

### Phase 1 — Understand the change before reading the changed lines

You're handed the diff and its context (description, ticket) — acquiring it is not your job here.
Your job starts with reading it:
1. **Read the description and context first.** Understand *what problem is being solved and why*
   before any changed line. No description is a yellow flag — the author hasn't articulated their
   thinking, which correlates with under-thought code. Note it.
2. **Check scope vs. description.** Does the description match what the change actually touches?
   "Fix login bug" touching the auth service, a shared session utility and a frontend form
   prompts the question: what are all three doing in one change?
3. **Check size.** Large changes are easy to rubber-stamp. Too large to hold in your head → say
   so and ask whether it can be split (refactor and feature as separate changes). A team merging
   large unreviewed changes is accumulating debt, not saving time.
4. **Know your position.** Be honest about familiarity. Scrutinize what you know deepest; where
   you're less familiar, lean on structural analysis — does the code make sense on its own terms,
   follow established patterns, and avoid inventing new ones without a reason?

### Phase 2 — Map the changed surface and expand outward

This phase is merge-readiness mode; in task-scoped gate mode the named-risk budget above replaces
it. Bugs live in the interaction between changed and unchanged code, so before reviewing logic,
classify *what kind of thing* changed — that sets how far you look outward — then go read that
surrounding code and every consumer:

| Changed thing | Where to look next |
|---|---|
| A utility / helper function | Every caller — find all usages across the codebase. |
| A function's return contract (e.g., now returns `null`) | Every call site — do callers handle the new case? |
| A model / schema / type | Serializers, validators, factories, fixtures, every reference to the shape. |
| A shared / base component (button, modal, input, table) | **Every usage.** The diff alone is insufficient (see `frontend.md`). |
| An exported function / public API / emitted event | Every downstream consumer — the implicit-contract surface (see `review-lenses.md`). |
| A new code path / flow | Parallel paths doing the same job (legacy endpoint, mobile flow, batch vs. interactive). |

When you can't tell why something works, **recreate it** — read it line by line, or note that a
sandbox/test is needed. Async flows, caching, state transitions and ordering guarantees reward
this most. Anything you cannot reach or run (a consumer in another repo,
runtime behavior, perf under load, author intent) goes into **Manual verification required** —
never silently into "looks fine."

### Phase 3 — Review through the lenses

Apply each review lens to the mapped surface. Load `references/review-lenses.md` here and work
it: architecture & design across levels · implicit contracts on public interfaces · parallel
entry points · wrong and premature abstractions · complexity disguised as elegance · error
handling and the *unhappy* paths · state consistency on partial failure · security · specific
(not vague) performance · observability · tests that test behavior · documentation · spec
silence is not permission. Two lenses load conditionally:

- **Frontend-specific**, only if frontend code changed. → `references/frontend.md`
- **AI-generated-code smells** — plausible-but-meaningless names, ill-fitting boilerplate,
  hallucinated APIs. → `references/ai-generated-code.md`

### Phase 4 — Write the report

Produce the report in the structure below — full template in `assets/report-template.md`. It
separates what blocks merge, what would improve the change, what you could not verify from the
diff, and what a human must verify manually because static review cannot confirm it.

---

## Test budget — verify by reading

The implementer already ran the tests and reported results for exactly this code. Verify those
claims by reading. Run a test only when reading raises a specific doubt no existing run answers,
and then one focused test — never a package-wide suite, a race detector run, or a high-count
loop. When heavy validation is warranted, *recommend* it in the report instead of running it.
When you cannot run commands here, name the test you would run.

Warnings or other noise in the reported test output are findings — test output should be pristine.

Evidence you cannot see is not evidence that doesn't exist. If the report or its test evidence
looks truncated, re-read the file at its stated path; if it is genuinely missing or garbled,
report that as a gap. Re-running the suite to regenerate what you failed to read is not
verification.

---

## The evidence rule — `file:line` or "could not verify"

**Every check is answered with a `file:line` citation, or with "could not verify" plus the reason.
A bare "yes" is not an answer.** This binds every finding, every lens you report on, and all 11
pre-approval checklist items.

Reviewers pass defects the prompt pointed them straight at: in live evals an accessible-name
check and a temp-dir-cleanup check both came back "yes" while the defect sat in the reviewed
diff. The citation is what makes an answer falsifiable. Cite, don't narrate — the line reference
is the evidence; a paragraph describing your search is not.

---

## Distrust the report — including its rationales

The description, PR body, implementer report, and commit messages are unverified claims about
the code. Check each against the diff.

Design rationales are claims too. "Left it per YAGNI", "kept it simple deliberately",
"intentional" is the author grading their own work: **a stated rationale never downgrades a
finding's severity.** Record the rationale beside the finding and leave the severity where the
code puts it.

The plan gets the same treatment: if the plan or brief mandates something this rubric calls a
defect, that IS a finding — report it as **✗ Blocking**, labelled *plan-mandated*. The plan's
authorship does not grade its own work; the user decides.

**How this sits with Chesterton's Fence** (below): the fence governs what you may *delete* —
understand why code exists before removing it. It does not let a self-authored rationale lower a
severity. "Is there a specific case this guards against?" is the fence; accepting "I kept it
simple deliberately" as the answer to a correctness finding is not.

---

## How to phrase findings (this matters as much as the findings)

The single most important habit in lead-level review is **asking questions instead of issuing
corrections.** Prefer "I'd have expected X here — can you walk me through the reasoning?" over
"this is wrong, do X." A question respects a reason you may not know, makes the author re-examine
their own thinking (which is how they often find the bug themselves), and survives being wrong —
you will sometimes be wrong, and a misguided correction erodes trust where a question never does.

Apply these principles when writing every finding:
- **Chesterton's Fence.** Before flagging something as wrong, redundant or over-complex, assume
  it was written that way for a reason: strange workarounds exist because of framework bugs,
  browser quirks and race conditions, and a "redundant" check may guard a historical edge case.
  Ask "is there a specific case this guards against?" rather than "this can be simplified." The
  fence is *not* cover for avoiding a hard conversation — when a refactor or fix is genuinely
  right, say so plainly. And it governs what you may *delete*; it never makes the author's stated
  rationale a verdict (see *Distrust the report*).
- **Don't blindly accept the answer either.** If the author explains and you still disagree, say
  so, restate the concern differently, or ask for a third opinion. The goal is good software, not
  a closed thread.
- **Feedback on the code, not the person.** "This function doesn't handle the null case", not
  "you forgot the null case." Never accusatory about AI authorship: "this pattern doesn't match
  how we do X here — can you walk me through the intent?" holds the same standard regardless of
  who or what wrote the code.
- **Be specific and actionable.** "This seems off" is useless. "I think this returns an incorrect
  value when `items` is empty — can you trace that path?" is actionable. If you propose a change,
  show the code.
- **Explain the *why*, not just the *what*.** A review is a teaching tool — every finding should
  leave the author more capable, so state the reasoning and point to the relevant pattern, doc,
  or reference implementation when it applies.

---

## Severity vocabulary (use these labels on every finding)

Conflating severities creates confusion. Label every finding so the author knows what is
required versus optional:

- **✗ Blocking** — must be fixed before merge. Correctness bug, security hole, broken contract,
  significant design problem.
- **🔧 Suggestion** — would be better another way; flagged for consideration, not blocking.
- **🔨 Nitpick** — minor style/preference; take it or leave it.
- **❓ Question** — genuinely unsure; want to understand intent before judging.
- **📌 Observation** — worth knowing, not necessarily a problem.
- **✓ Praise** — good decision worth reinforcing explicitly. Don't skip this; positive
  reinforcement of good practice matters as much as correction.

**The trust threshold for Blocking:** blocking means *this cannot be trusted until it is fixed* —
incorrect or fragile behavior, a missed requirement, a security hole, a broken contract, or
maintainability damage you would hold a merge over. Three maintainability defects are concretely
merge-blocking: **verbatim duplication of a logic block, swallowed errors, and tests that assert
nothing.** "Coverage could be broader" and polish are Nitpicks.

---

## Report structure

Always produce the report using the template in `assets/report-template.md`. Its required slots,
in order:

1. **Verdict** — ✓ Approve · ✓ Approve with follow-ups · ⚠️ Request changes · ✗ Blocked, plus a
   one-paragraph rationale. Approval is not a stamp of perfection — it means the code is ready to
   ship and the remaining concerns are tracked.
2. **What this change does** — your own-words summary of problem and approach. Flag a missing
   description or a scope mismatch here.
3. **Change profile** — files/lines, risk level and why, shared/contract surface, scope verdict.
4. **Findings** — grouped by the severity labels above, each `file:line` — concern — *why it
   matters* — action.
5. **⚠️ Cannot verify from diff** — requirements that live in unchanged code or span tasks, each
   routed to the coordinator to resolve before the work is called complete, never silently passed.
6. **Manual verification required** — what static review cannot confirm and a human must do
   before approving (run the code, confirm intent, reach a consumer you couldn't access,
   load-test a perf concern). Mandatory: the report alone is never sufficient to merge.
7. **Rulings I made** — every judgment call you took on the user's behalf, each with what it costs
   if wrong. A ruling that stays in your head is a decision made in secret.
8. **Pre-approval checklist** — the 11 items below, each ✓ pass / ⚠️ could not verify / ✗ fail,
   each carrying its `file:line` evidence or the reason it could not be verified.

### What the report is

Your final message is the report itself. Its first line is the verdict. Every line after it is a
verdict, a finding carrying `file:line`, a check you ran, or a required heading from the
template. It ends on its last section.

### The pre-approval checklist (always evaluate all 11)

Answer each item with `file:line` evidence or an explicit "could not verify". A ✓ with no
citation is not a pass.

1. I understand what problem this change is solving.
2. I traced at least one non-happy-path scenario.
3. I checked the code surrounding the diff, not just the diff.
4. I considered what happens to existing behavior if a core shared component changed.
5. I asked about anything I don't understand rather than assuming it's fine.
6. Error handling is adequate for the risk profile of this change.
7. Semantic contracts of public interfaces are preserved, or the break is intentional and
   communicated.
8. I considered concurrent or out-of-order execution where async code is involved.
9. Security-sensitive operations (auth, input handling, data exposure) are correct.
10. Test coverage is appropriate for the risk of this change.
11. Blocking issues are identified and labeled clearly.

---

## Dispatch economics — when you are the coordinator

Reviewing a diff inline burns the context you need to keep driving the work. When you
are coordinating rather than reviewing, dispatch the review so the diff and the
evaluation live in the reviewer's context and only the findings come back.

Mechanics, the package script, what never goes in the dispatch prompt, and where the
findings go: [references/dispatching-a-review.md](references/dispatching-a-review.md).

## Guiding principle

The measure of a great review is **not the number of issues found** — it's whether the software
that ships is more correct, more maintainable and better understood than it would have been
without the conversation. Optimize for that, not for nits.

## Bundled files

- `references/review-lenses.md` — the full catalogue of checks (load in Phase 3).
- `references/frontend.md` — frontend review (load only if frontend code changed).
- `references/ai-generated-code.md` — AI-generated code smells.
- `assets/report-template.md` — the output report template to fill in.
- `assets/review-package.sh` — `bash assets/review-package.sh BASE HEAD` writes the commit list,
  `--stat` and `git diff -U10` to a file the reviewer reads in one call.

## Attribution

The two review modes and their named-risk budget, the test budget, the evidence rule, the
report-distrust posture, the Blocking trust threshold, dispatch economics, and the read-only
guarantee (quoted verbatim) are adapted from
[obra/superpowers](https://github.com/obra/superpowers) — MIT, © 2025 Jesse Vincent.
