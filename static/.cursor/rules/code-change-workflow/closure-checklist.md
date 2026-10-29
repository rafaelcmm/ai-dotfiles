# Closure checklist

Read at closure, not during the work. The copy-paste template below is the evidence
block a completed code-change task reports against.

It lives outside `code-change-workflow.md` because it is consulted once, at the end,
while that rule is read on every code change — a template that is never needed
mid-task does not have to occupy context for the whole task.

Use this copy-paste closure template directly:

- Objective Files
  - [ ] File scope remains single-purpose.
  - [ ] No logic file grew past 300 lines without active split.
  - [ ] If any file exceeded limits, split plan or split implementation completed.
  - [ ] Any exception includes reason, owner, expiry trigger, follow-up task.
- Chesterton's Fence
  - [ ] Removed/simplified code has explicit reason it existed.
  - [ ] Evidence source captured (git history, issue, caller behavior, production constraint).
  - [ ] Verification proves behavior preserved.
- Commenting Standards
  - [ ] Every changed public symbol has docblock.
  - [ ] Docblocks include intent, guarantees, failure modes.
  - [ ] Non-obvious logic has why/invariant comments.
  - [ ] No low-value narration comments.
  - [ ] Comments describe the code as it now stands — no provenance, removed behavior, dated claims, or supersession notices.
  - [ ] No padding: depth matches visibility; required tags present with substantive descriptions.
  - [ ] History narration found in touched comments was deleted, not extended.
- Ceremony Classification
  - [ ] Classification stated out loud: spike, bounded, or architectural.
  - [ ] Every gate ran, whichever path was taken.
  - [ ] Any mid-task reclassification went up, and was stated.
- Code Change Workflow
  - [ ] Implementation TODOs complete.
  - [ ] Testing TODOs complete.
  - [ ] Final Checkup TODOs complete.
  - [ ] Security and documentation reviews executed in final checkup.
  - [ ] Documentation Completion Step passed.
  - [ ] Completion contract satisfied per TODO, with in-session evidence.
  - [ ] Reviewer loop closed within five rounds, or escalated with rulings.
- Evidence Summary
  - Files changed:
  - Checks run and outcomes:
  - Skipped checks and reason:
  - Ceremony classification and any escalation:
  - Rulings taken on the user's behalf, each with cost if wrong:
  - Remaining risk (if any):
