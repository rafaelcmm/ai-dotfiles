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

> Each finding: `path:line` — the concern — *why it matters* — suggested action.
> Phrase as questions where intent is unknowable. Feedback is on the code, not the author.
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

## Pre-approval checklist

| # | Check | Status | Note |
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

**Legend:** ✓ pass · ⚠️ needs manual check (see section above) · ✗ fail (see Blocking) · n.a. not applicable
