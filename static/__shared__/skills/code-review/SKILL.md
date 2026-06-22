---
name: code-review
description: >-
  Run a thorough, lead-developer-level review of code that is in merge/review state — a
  GitHub pull request, a GitLab merge request, or a local branch diff — and produce a
  structured report of what must change, what should change, and what a human must verify
  manually before approval. Use this skill whenever the user asks to "review" a PR/MR/branch,
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

The companion agent in `agents/code-reviewer.md` is the autonomous executor for this skill.
Use the agent when you want the review run end-to-end and handed back as a report; use this
SKILL.md directly when you are reviewing inline yourself.

---

## The core mental model: debug before you approve

Adopt the posture of a debugger, not a proofreader. **Start from the assumption that this
change is a regression and try to prove yourself wrong.** Most reviewers start with "this is
probably fine" and skim for exceptions — flip it. Assume there is a bug, then trace the data
flow, the control paths, and the edge cases until you are satisfied it works. This single shift
is what separates a real review from a rubber stamp.

Two corollaries drive most of the work:

- **Review the change, not the diff.** Bugs live in the interaction between changed and
  unchanged code. A function that now returns `null` under a new condition may be consumed by a
  caller three files away that never expected `null`. The diff shows the change; the bug lives
  elsewhere. Always open surrounding files, callers, and consumers.
- **If you don't understand why it works, don't pass it.** Recreate it — open a sandbox, write
  a small test, trace it by hand. The best review comments often begin with "I wasn't sure why
  this worked, so I tried it, and found…"

---

## Workflow

Run these phases in order. Don't skip the early ones — most of a review's leverage comes from
understanding the change before reading a single changed line.

### Phase 1 — Understand the change before reading the changed lines

You're handed the diff and its context (description, ticket) — acquiring it is not your job here.
Your job starts with reading it:
1. **Read the description and context first.** Understand *what problem is being solved and why*
   before any changed line. A change with no description is a yellow flag: it suggests the author
   hasn't articulated their thinking, which often correlates with under-thought code. Note it.
2. **Check scope vs. description.** Does the description match what the change actually touches? A
   change titled "fix login bug" that touches the auth service, a shared session utility, and a
   frontend form should prompt the question: what are all three doing in one change?
3. **Check size.** Large changes are easy to rubber-stamp. If the change is too large to hold in
   your head, say so, and ask whether it can be split (e.g., refactor and feature as separate
   changes). A team that merges large unreviewed changes is accumulating debt, not saving time.
4. **Know your position.** Be honest about familiarity. Scrutinize the areas you know deepest;
   where you're less familiar, lean on structural analysis — does the code make sense on its own
   terms, follow established patterns, and avoid inventing new patterns without a reason?

### Phase 2 — Map the changed surface and expand outward

Diffs isolate changed lines, but bugs live in the interaction between changed and unchanged code.
Before reviewing logic, classify *what kind of thing* changed — that determines how far you must
look outward — then go read that surrounding code and every consumer:

| Changed thing | Where to look next |
|---|---|
| A utility / helper function | Every caller — find all usages across the codebase. |
| A function's return contract (e.g., now returns `null`) | Every call site — do callers handle the new case? |
| A model / schema / type | Serializers, validators, factories, fixtures, every reference to the shape. |
| A shared / base component (button, modal, input, table) | **Every usage.** The diff alone is insufficient (see `frontend.md`). |
| An exported function / public API / emitted event | Every downstream consumer — the implicit-contract surface (see `review-lenses.md`). |
| A new code path / flow | Parallel paths doing the same job (legacy endpoint, mobile flow, batch vs. interactive). |

When you genuinely can't tell why something works, **recreate it** — read it line by line, or
note that a sandbox/test is needed to confirm. Tricky async flows, caching, state transitions,
and ordering guarantees especially reward this. Anything you cannot reach or run (a consumer in
another repo, runtime behavior, perf under load, author intent) goes into the report's **Manual
verification required** section — never silently into "looks fine."

### Phase 3 — Review through the lenses

Apply each review lens to the mapped surface. The lenses are catalogued in detail in the
reference files; load the relevant one when you reach it:

- **Architecture & design** — does the change belong in this layer? Right abstraction? Right
  responsibility? Implicit contracts preserved? → `references/review-lenses.md`
- **Abstractions & cognitive load** — wrong abstraction, premature abstraction, complexity
  disguised as elegance. → `references/review-lenses.md`
- **Error handling & edge cases** — trace the *unhappy* paths and state-consistency on partial
  failures. → `references/review-lenses.md`
- **Security, performance, observability** — input handling, authz, data exposure, specific
  (not vague) perf concerns, debuggability in production. → `references/review-lenses.md`
- **Tests & documentation** — do the tests test *behavior* or just implementation? Is coverage
  appropriate to the *risk* of the change? → `references/review-lenses.md`
- **Frontend-specific** (only if frontend code changed) — JS doing HTML/CSS's job, event-driven
  components, async React patterns, shared-component blast radius. → `references/frontend.md`
- **AI-generated-code smells** — plausible-but-meaningless names, ill-fitting boilerplate,
  wrong-level abstractions, hallucinated APIs. → `references/ai-generated-code.md`

### Phase 4 — Write the report

Produce the report in the exact structure defined below (full template in
`assets/report-template.md`). The report must separate three things clearly:
- **what blocks merge** (Blocking findings),
- **what would improve the change** (Suggestions/Nitpicks/Questions/Observations/Praise),
- **what a human must verify manually** because static review cannot confirm it.

---

## How to phrase findings (this matters as much as the findings)

The single most important habit in lead-level review is **asking questions instead of issuing
corrections.** Prefer "I'd have expected X here — can you walk me through the reasoning?" over
"this is wrong, do X." Questions respect that the author may have a reason you don't know, open a
conversation instead of closing one, make the author re-examine their own thinking (which is how
they often find the bug themselves), and are more likely to be *accurate* — you will sometimes be
wrong, and a misguided correction erodes trust where a good question never does.

Apply these principles when writing every finding:
- **Chesterton's Fence.** Before flagging something as wrong/redundant/over-complex, assume it
  was written that way for a reason. Strange workarounds often exist because of a framework bug,
  browser quirk, or race condition; a "redundant" check may guard a historical edge case. Ask
  "is there a specific case this guards against?" rather than "this can be simplified." But the
  fence is *not* an excuse to avoid hard conversations — if a refactor or fix is genuinely right,
  say so plainly. Understand before acting; don't leave bad code alone forever.
- **Don't blindly accept the answer either.** If the author explains and you still disagree, say
  so, restate the concern differently, or ask for a third opinion. The goal is good software, not
  a closed thread.
- **Feedback on the code, not the person.** "This function doesn't handle the null case" — not
  "you forgot the null case." Especially for AI-generated-code findings: never be accusatory.
  "This pattern doesn't match how we do X here — can you walk me through the intent?" holds the
  same standard regardless of who (or what) wrote the code.
- **Be specific and actionable.** "This seems off" is useless. "I think this returns an incorrect
  value when `items` is empty — can you trace that path?" is actionable. If you propose a change,
  show the code; don't make the author guess what "refactor this" means.
- **Explain the *why*, not just the *what*.** A review is a teaching tool — every finding should
  leave the author more capable, so state the reasoning and point to the relevant pattern,
  doc, or reference implementation when it applies. This is why the report format requires a "why
  it matters" on every finding and treats Praise as a first-class label.

---

## Severity vocabulary (use these labels on every finding)

Not all comments are equal, and conflating them creates confusion. Label every finding so the
author knows what is required versus optional:

- **✗ Blocking** — must be fixed before merge. Correctness bug, security hole, broken contract,
  significant design problem.
- **🔧 Suggestion** — would be better another way; flagged for consideration, not blocking.
- **🔨 Nitpick** — minor style/preference; take it or leave it.
- **❓ Question** — genuinely unsure; want to understand intent before judging.
- **📌 Observation** — worth knowing, not necessarily a problem.
- **✓ Praise** — good decision worth reinforcing explicitly. Don't skip this; positive
  reinforcement of good practice matters as much as correction.

---

## Report structure

Always produce the report using the template in `assets/report-template.md`. Its sections, in
order, are:

1. **Verdict** — one of: ✓ Approve · ✓ Approve with follow-ups · ⚠️ Request changes · ✗ Blocked,
   plus a one-paragraph rationale. Approval is not a stamp of perfection — it means the code is
   ready to ship and any remaining concerns are tracked.
2. **What this change does** — the reviewer's own-words summary of the problem solved and the
   approach taken. Flag here if the description was missing or didn't match scope.
3. **Change profile** — files/lines touched, risk level and why, whether shared/contract surface
   was touched, scope-vs-description verdict.
4. **Findings** — grouped by the severity labels above, each with `file:line`, the concern,
   *why it matters*, and a suggested action.
5. **Manual verification required** — a checklist of things static review cannot confirm and a
   human must do before approving (run the code, confirm intent with the author, check a consumer
   the reviewer couldn't access, load-test a perf concern, etc.). This is mandatory — never imply
   the report alone is sufficient to merge.
6. **Pre-approval checklist** — the 11-item checklist below, each marked ✓ pass / ⚠️ needs manual
   check / ✗ fail, with a note.

### The pre-approval checklist (always evaluate all 11)

1. I understand what problem this change is solving.
2. I've traced through at least one non-happy-path scenario.
3. I've checked the code surrounding the diff, not just the diff itself.
4. I've considered what happens to existing behavior if a core shared component changed.
5. I've asked about anything I don't understand rather than assuming it's fine.
6. I'm satisfied that error handling is adequate for the risk profile of this change.
7. I've checked that the semantic contracts of any public interfaces are preserved, or the
   breaking change is intentional and communicated.
8. I've thought about concurrent or out-of-order execution where async code is involved.
9. I've considered whether any security-sensitive operations (auth, input handling, data
   exposure) are correct.
10. The test coverage is appropriate for the risk of this change.
11. I've identified and labeled any blocking issues clearly.

---

## Guiding principle

The measure of a great review is **not the number of issues found** — it's whether the software
that ships is better (more correct, more maintainable, better understood) than it would have been
without the conversation. Optimize for that, not for finding the most nits.

## Reference files

- `references/review-lenses.md` — the full catalogue of what to look for: architecture, implicit
  contracts, abstractions, error handling/edge cases, security, performance, observability,
  tests, documentation.
- `references/frontend.md` — frontend-specific review (load only if frontend code changed).
- `references/ai-generated-code.md` — detecting and responding to AI-generated code smells.
- `assets/report-template.md` — the exact output report template to fill in.
