---
name: receiving-code-review
description: >-
  Use when code review feedback arrives and before implementing any part of it — reviewer
  comments on a pull or merge request, an automated review report, a colleague's notes on a
  branch, or a review returned by another agent. Covers feedback that is unclear, technically
  questionable, out of scope, or that asks for behaviour this codebase already decided
  against. Also use after you have pushed back on a reviewer and need to decide whether to
  hold the position or retract it. Load it before the first edit, not after.
---

# Receiving Code Review

Review feedback is a set of claims about your code. Some are correct. Some are correct about
the symptom and wrong about the fix. Some are wrong because the reviewer could not see what
you can see — the platform floor, the legacy caller, the decision made three months ago.

Implementing all of them faithfully is not diligence. It is a refusal to do the one thing
you are better placed to do than the reviewer: check each claim against the actual code.

**Review feedback is suggestions to evaluate, not orders to follow.** That holds for human
reviewers, automated reviewers, and agents.

---

## The Iron Law

```
NO AGREEMENT BEFORE VERIFICATION
```

You cannot know a suggestion is correct until you have checked it against the code.
Therefore you cannot say it is correct, and you cannot start implementing it, until you
have.

Premature agreement — "You're absolutely right", "Good point", "Let me implement that now"
— commits you to a position you have not tested. Every verification after that runs under
pressure to confirm what you already conceded, and the wrong change ships with your
signature on it. State the technical requirement in your own words instead, or say nothing
and go read the code.

---

## The Response Pattern

Each step exists because skipping it produces a specific, observed failure.

| Step | Action | Failure it prevents |
|------|--------|---------------------|
| **READ** | Read the complete feedback before reacting to any of it | Fixing item 1 in a way item 5 forbids |
| **UNDERSTAND** | Restate each item as a technical requirement in your own words; if you cannot, it is unclear | Implementing your guess at what was meant |
| **VERIFY** | Check the claim against the codebase — the call sites, the tests, the git history, the target platform | Fixing a bug that does not exist; breaking a guard that was load-bearing |
| **EVALUATE** | Decide whether it is correct **for this codebase**, not in general | Importing a convention from a different stack |
| **RESPOND** | Draft a technical acknowledgment or a reasoned pushback, item by item | Silent partial compliance the reviewer cannot audit |
| **IMPLEMENT** | One item at a time, test each before the next | A batch of changes where you cannot tell which one broke the suite |

---

## Clarify every item before implementing any

```
IF any item is unclear:
    Implement nothing yet.
    Ask about the unclear items.
```

Feedback items are usually related. Item 4 may constrain the shape of the fix for item 2 —
rename a field, and the "while you're there, validate it" comment lands somewhere else.
Implementing the understood items first means redoing them once the rest arrives.

```
Reviewer: "Fix 1-6."
You understand 1, 2, 3, 6. Unclear on 4 and 5.

WRONG: implement 1, 2, 3, 6 now; ask about 4 and 5 after.
RIGHT: "Understood 1, 2, 3, 6. Need clarification on 4 and 5 before implementing
        any of them — 4 may change the interface 2 touches."
```

**When you cannot verify a claim**, say so rather than proceeding on assumption: "I can't
verify this without access to the staging logs. Options: investigate X, ask the reviewer, or
proceed as suggested and flag the risk." The limitation is information the user needs.

---

## Feedback from the user vs. from a reviewer

**From the user:** trusted. Understand it, then implement. Still ask when the scope is
unclear — "fix the retry logic" does not say whether the backoff curve is in scope.

**From any other reviewer** — human, bot, or agent — run the full checks before implementing:

```
1. Is this technically correct for THIS codebase, this stack, this version floor?
2. Does it break existing functionality? Which tests cover that?
3. Is there a reason the current implementation is the way it is?
4. Does it hold on every platform and version we support?
5. Does the reviewer have the context this decision needed?
6. Does it conflict with an architectural decision already made?
```

If check 6 trips, stop and raise it with the user before writing code. A reviewer cannot
overrule a decision the user made; only the user can.

---

## The YAGNI gate on "implement this properly"

A reviewer who sees a stub often asks for the full feature. Before building it, find out
whether anything needs it.

```
IF the reviewer asks you to "implement X properly":
    grep the codebase for real callers of X.

    No callers  → propose removal: "Nothing calls this endpoint. Remove it (YAGNI),
                  or is there usage I'm missing?"
    Callers     → implement it properly, to the contract the callers need.
```

Building an unused feature properly costs maintenance forever and adds surface to review.
The reviewer and you are both serving the user's requirements, not each other's taste.

---

## When to push back

Push back when any of these is true:

- The suggestion **breaks existing behaviour** — and you can name the test or caller that proves it
- The **reviewer lacks context** you have: a platform floor, a prior incident, an upstream constraint
- It **violates YAGNI** — the thing it improves has no caller
- It is **technically wrong for this stack** — correct advice from a different language, framework, or runtime
- A **legacy or compatibility reason** for the current code exists and still applies
- It **conflicts with an architectural decision already made** with the user

A reviewer asking you to delete code is not a licence to delete it. The
`chestertons-fence` rule still applies in full: you must be able to state why the code
exists before removing it, and "a reviewer said so" is not that statement.

### How to push back

- Lead with the technical reason, not with discomfort. "This needs macOS 13; the build
  target is 10.15" beats "I'm not sure about this one."
- **Reference the working evidence** — the test that passes today, the caller that depends
  on the current signature, the commit that introduced the guard.
- Ask a specific question rather than refusing: "Current impl has the wrong bundle ID —
  fix that, or drop pre-13 support?"
- **Escalate to the user when the disagreement is architectural.** That is not a failure to
  resolve it; architecture is not yours or the reviewer's to settle alone.
- If pushing back feels uncomfortable, name the technical issue anyway. An unvoiced
  objection becomes a defect with your name on the commit.

### When your pushback was wrong

State the correction as a fact and move on.

```
RIGHT: "You were right — checked the build config, the target is 13+. Implementing now."
RIGHT: "Verified; you're correct. My reading of the version floor came from the old
        Podfile. Fixing."

WRONG: a long apology
WRONG: defending why the pushback was reasonable
WRONG: re-litigating the point while complying
```

The goal is the code being right, not the record showing you were right.

---

## Implementation order

```
1. Clarify everything unclear.               (Until done, implement nothing.)
2. Blocking issues first — breakage, security, data loss.
3. Simple fixes next — typos, imports, naming, dead code.
4. Complex fixes last — refactors, logic changes, interface changes.
5. Test each fix individually before starting the next.
6. Verify no regressions across the whole suite at the end.
```

Blocking issues first because a later refactor may move the code the blocker lives in.
Simple before complex because an unfixed typo makes a complex diff harder to review. One at
a time because a batch that fails tells you nothing about which change failed.

---

## Your response is a draft. You do not send it.

**REQUIRED BACKGROUND:** the `acting-in-the-users-name` rule. Replying to a reviewer is an
action carrying the user's identity, and it cannot be unsent.

```
Write the response. Hand it to the user. Never post it.
```

This applies to every channel: a reply in a review thread, a top-level comment on the PR or
MR, a resolution of a thread, a review verdict, a message to the reviewer, a comment on the
linked ticket. Drafting is unrestricted. Sending is the user's call, per instance.

Hand over in the form that is easiest to send: the text ready to paste, grouped per feedback
item, with each item marked `implementing`, `pushing back`, or `needs clarification`. Say
plainly that nothing has been sent.

Fixing the code is not an outward action. Commit locally and keep working.

---

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "The reviewer is more senior, they must be right" | Seniority is not context. They cannot see what you just read. Verify. |
| "Agreeing first keeps it collegial" | Agreement you retract later costs more than a question asked now. |
| "I'll implement the clear items and ask about the rest" | Items are related. Partial understanding produces work you redo. |
| "It's a small change, verification is overkill" | Small changes to load-bearing guards are how outages start. |
| "A reviewer asked me to delete it, so the fence rule doesn't apply" | The rule governs removal regardless of who requested it. |
| "They asked for it properly implemented, so it must be needed" | Grep first. An unused feature built properly is still unused. |
| "Pushing back will look defensive" | Technical reasoning is not defensiveness. Shipping a known defect is worse. |
| "I was wrong to push back, so I should explain myself" | State the correction and implement. The explanation is not the deliverable. |
| "The review thread is the natural place to answer" | Posting speaks in the user's name. Draft and hand over. |
| "I'll just reply to confirm I'm on it" | Still outward. Still theirs to send. |

---

## Red flags

Stop if you catch any of these:

- Writing "You're absolutely right" before opening the file
- Starting an edit while any feedback item is still unclear
- Implementing several items, then running tests once
- Assuming the reviewer checked whether their suggestion breaks something
- Treating an automated or agent review as more authoritative than the code
- Deleting something because a reviewer called it unnecessary, without knowing why it exists
- Reaching for `gh`, `glab`, or any API call that writes a comment, reply, or review verdict
- Silently dropping an item you disagreed with instead of pushing back on it

---

## Related

- `review-fix-loop` — the loop that runs review and fixes to convergence; this skill governs
  how you treat each round of feedback inside it.
- `code-review` — the reviewer-side skill. Reading it shows you what a reviewer could and
  could not have known when writing the comment.
- `verification-before-completion` — before you report the feedback as addressed.
- `chestertons-fence` rule — required whenever feedback asks you to remove or simplify code.
- `acting-in-the-users-name` rule — governs every reply, comment, and verdict.

---

## Attribution

Adapted from the `receiving-code-review` skill in `obra/superpowers` (MIT, © 2025 Jesse
Vincent). The thread-reply instruction was removed to satisfy the `acting-in-the-users-name`
rule, and the style prohibitions were narrowed to agreement-before-verification.
