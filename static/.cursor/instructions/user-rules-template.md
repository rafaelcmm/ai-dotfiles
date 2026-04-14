# Cursor User Rules Template

Use this template in Cursor Settings -> Rules -> User Rules.

Purpose: enforce deterministic autonomous workflow globally, without adding project files.

## Suggested User Rule

For non-trivial code-change requests, run an autonomous workflow unless I explicitly opt out:

- Plan first before editing.
- If available, delegate workflow coordination to `workflow-orchestrator`.
- Break work into implementation TODOs with acceptance criteria.
- Add testing TODOs and execute TODOs one at a time.
- Delegate specialist tasks proactively when scope matches a custom subagent.
- Run final checkup before completion: security review, performance review when relevant, documentation review, and repository lint/format/build checks.
- Do not mark complete without verification evidence.

When requests are simple and explicitly marked fast mode, do direct execution with minimal process overhead.

## Optional stricter add-on

When code changes are requested, always include:

- explicit TODO status tracking
- post-change verification summary
- unresolved risk list when full verification is not possible