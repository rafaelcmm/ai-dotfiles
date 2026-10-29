# Code Review Report — <PR/MR title or branch>

> Source: <github/gitlab URL or `branch...base`> · Reviewed: <date> · Author: <author>

## Verdict

**<✓ Approve | ✓ Approve with follow-ups | ⚠️ Request changes | ✗ Blocked>**

<One paragraph: the bottom line. What's the recommendation and why. If "Approve with
follow-ups", list the tracked follow-ups. Remember: approval means "ready to ship and remaining
concerns are tracked", not "perfect".>

---

## What this change does

<Reviewer's own-words summary of the problem being solved and the approach taken — written from
having read the description AND the diff, not just restating the title.>

- **Description present?** <yes / no — if no, note it as a yellow flag>
- **Scope matches description?** <yes / mismatch — describe the mismatch>

---

## Change profile

| Attribute | Value |
|---|---|
| Files changed | <n> |
| Lines added / removed | <+a / -d> |
| Size assessment | <small / medium / large — large + unreviewable → recommend splitting> |
| Risk level | <low / medium / high> — <one line why> |
| Shared / contract surface touched? | <yes (what) / no> |
| Frontend code touched? | <yes / no> |
| New dependencies added? | <yes (which) / no> |

---

## Findings

> Each finding: `path:line` — the concern — *why it matters* — suggested action. The `path:line`
> is not optional: a finding with no citation is not a finding.
> Phrase as questions where intent is unknowable. Feedback is on the code, not the author.
> A rationale quoted from the author or the plan is recorded next to the finding, never used to
> lower its severity; a defect the plan mandates is Blocking, labelled *plan-mandated*.
> Omit a severity section entirely if it has no findings.

### ✗ Blocking — must be resolved before merge

- **`path/to/file.ext:NN`** — <concern> — *why it matters* — <action>

### 🔧 Suggestions — would improve the change, not blocking

- **`path/to/file.ext:NN`** — <concern> — *why* — <action>

### 🔨 Nitpicks — minor; take or leave

- **`path/to/file.ext:NN`** — <concern>

### ❓ Questions — for the author

- **`path/to/file.ext:NN`** — <the question, e.g. "I'd have expected X here — can you walk me
  through the reasoning?">

### 📌 Observations — worth knowing, not necessarily a problem

- <observation>

### ✓ Praise — good decisions worth reinforcing

- **`path/to/file.ext:NN`** — <what was done well>

---

## ⚠️ Cannot verify from diff

> Requirements and behaviors that live in unchanged code, span tasks, or depend on context you
> were not given. These are routed to the coordinator, which holds the plan and the cross-task
> context, and each one is resolved before the work is called complete. An item here is neither a
> pass nor a failure — it is an open question with an owner.

| # | What could not be verified | Why the diff can't settle it | Who resolves it / what to check |
|---|---|---|---|
| 1 | <e.g. "rate limiting still applies to the new endpoint"> | <the limiter config is not in this diff> | <coordinator: confirm `limiter.yaml` covers `/v2/export`> |

<"None — every requirement was verifiable from the diff." if that is true.>

---

## Manual verification required

> Things static review could NOT confirm. A human must do these before approving. Be specific
> and actionable. This section is mandatory — the report alone is never sufficient to merge.

- [ ] <e.g. Run the new payment flow end-to-end against the sandbox gateway — static review can't
  confirm the third-party callback shape.>
- [ ] <e.g. Confirm with the author whether the contract change to `getUser()` returning `null`
  was intentional; if so, ensure the migration note is added to the description.>
- [ ] <e.g. Check consumer `mobile-app` repo (not accessible here) for usages of the changed
  `<Button>` prop interface.>
- [ ] <e.g. Load-test or profile the N+1 in `report.py:88` if this path runs at scale.>

---

## Rulings I made

> Every judgment call taken on the user's behalf during this review: a severity you set where the
> rubric was ambiguous, a finding you decided not to raise, a scope line you drew, an assumption
> you reviewed against. One line each, in the order you made them, each with **what it costs if
> this ruling is wrong**. A ruling that stays in your head is a decision made in secret.

| # | Ruling | Cost if wrong |
|---|---|---|
| 1 | <e.g. "Graded the missing `null` check Suggestion, not Blocking, because the only caller is internal."> | <If an external caller exists, this ships a crash path.> |

<"None — no judgment calls were required." if that is true.>

---

## Pre-approval checklist

> Every row carries `file:line` evidence or the reason it could not be verified. A ✓ with an
> empty evidence cell is not a pass — downgrade it to ⚠️ and say what blocked verification.

| # | Check | Status | Evidence (`file:line`) or why not verified |
|---|---|---|---|
| 1 | I understand what problem this change is solving | <✓/⚠️/✗> | |
| 2 | I've traced through at least one non-happy-path scenario | <✓/⚠️/✗> | |
| 3 | I've checked the code surrounding the diff, not just the diff | <✓/⚠️/✗> | |
| 4 | I've considered existing behavior if a core shared component changed | <✓/⚠️/✗/n.a.> | |
| 5 | I've asked about anything I don't understand rather than assuming | <✓/⚠️/✗> | |
| 6 | Error handling is adequate for the risk profile of this change | <✓/⚠️/✗> | |
| 7 | Semantic contracts of public interfaces are preserved (or breaks communicated) | <✓/⚠️/✗/n.a.> | |
| 8 | I've thought about concurrent / out-of-order execution where async | <✓/⚠️/✗/n.a.> | |
| 9 | Security-sensitive operations (auth, input, data exposure) are correct | <✓/⚠️/✗/n.a.> | |
| 10 | Test coverage is appropriate for the risk of this change | <✓/⚠️/✗> | |
| 11 | I've identified and labeled any blocking issues clearly | <✓/⚠️/✗> | |

**Legend:** ✓ pass, with evidence · ⚠️ could not verify (routed above) · ✗ fail (see Blocking) ·
n.a. not applicable, with the reason
