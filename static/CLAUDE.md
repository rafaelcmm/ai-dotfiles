@AGENTS.md

# Claude Overlay

This file extends shared contract from `AGENTS.md`.

## Runtime Scope

- Apply only Claude-specific behavior and enforcement.
- Keep shared policy in `AGENTS.md`; do not duplicate unless needed for runtime clarity.

## Rule Loading

- Resolve intent, then load matching rule files from `.claude/rules` before edits.
- If multiple rules apply, load all relevant rules and follow strictest safe constraint.
- For non-trivial code changes, invoke `workflow-orchestrator` first when available.
- For Shopify-related prompts, load `shopify-skills` rule and consult matching `shopify-*` skills first.

## Caveman Execution

- Caveman mode enabled by caveman rule activation.
- Default caveman level is ultra unless user selects another level.
- Keep workflow communication pinned to caveman maximum (`ultra`) for token economics unless auto-clarity overrides it.
- Stop caveman on explicit user command (`stop caveman` or `normal mode`).
- Auto-clarity override for security warnings, irreversible actions, or clear user confusion.

## Workflow Execution Contract

- Produce and maintain TODO list with:
	- id
	- owner
	- scope
	- acceptance criteria
	- verification step
	- status
- Required TODO classes:
	- implementation
	- testing
	- final checkup
- Run `security-reviewer` and `documentation-specialist` during final checkup only, after implementation and testing work is complete.
- Apply reviewer feedback before closing final checkup.
- Documentation completion is final pre-close gate for changed behavior.

## Completion Criteria

- Requirements covered.
- Relevant checks executed or explicitly documented as skipped.
- Risks and limitations reported.
- Final summary includes what changed, evidence, and next optional actions.

## Runtime Integrations

- Hook and MCP configuration location: `.claude/settings.json`.
