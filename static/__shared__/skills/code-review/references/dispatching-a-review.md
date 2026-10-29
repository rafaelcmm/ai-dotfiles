# Dispatching a review

Read this when you are the coordinator handing a review to a subagent rather than
performing the review yourself. It does not apply when you are the reviewer.

## Contents

- [Why dispatch at all](#why-dispatch-at-all)
- [What the reviewer receives](#what-the-reviewer-receives)
- [What never goes in the prompt](#what-never-goes-in-the-prompt)
- [Where the findings go](#where-the-findings-go)


Reviewing the diff inline burns the context you need to keep driving the work. Dispatch the
`code-reviewer` agent: the diff and the evaluation live in its context, and only the findings
come back.

- **Hand it precisely crafted context, never session history.** The diff range, the requirements
  it judges against, the global constraints binding the change (version floors, naming rules,
  platform requirements), and the review mode. Session history puts the reviewer on your thought
  process instead of the work product.
- **Build the package outside your context.** `bash assets/review-package.sh BASE HEAD` writes
  the commit list, `--stat` and `git diff -U10` to a file and prints the path; hand over the
  path. The reviewer reads the change in one call and the diff never enters your context. The
  wide `-U10` context is the point — it lets the reviewer judge hunks without opening files,
  which is what makes the diff-scope budget survivable.
- **Don't pre-judge findings.** Never tell a reviewer to ignore or not flag a specific issue;
  adjudicate suspected false positives after the report arrives.
- **Reviewers never dispatch sub-reviewers.** "A reviewer you spawn duplicates one of them at
  full cost, and its verdict counts for nothing." A diff too large for one pass is reviewed in
  passes by the same reviewer, which says so in its report.
- **The report is the deliverable and it goes to the user.** No reviewer posts a review, comments
  on a PR/MR, or publishes anything.

**Read-only guarantee — state it to every reviewer:** "Your review is read-only on this checkout.
Do not mutate the working tree, the index, HEAD, or branch state in any way."

---
