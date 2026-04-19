# AGENTS

Cross-tool operating contract for AI coding assistants.

## Scope

- Shared guidance reused across projects and providers.
- Provider-specific behavior belongs in overlays like `CLAUDE.md`.
- Keep this file short, stable, and conflict-free.

## Priority Order

1. User intent and safety constraints.
2. Workflow integrity and verification.
3. Small reversible edits over wide refactors.
4. Clear outcome evidence before completion.

## Rule Router

- Code change request: load workflow, fence, commenting, objective-file guidance.
- Prompt writing or adaptation: load prompt guidance.
- Brevity or caveman request: enable caveman policy.
- Removal or simplification request: enforce fence checks before deleting.

## Workflow Contract

- Plan before edits.
- Break work into TODOs with owner, scope, acceptance criteria, and verification step.
- Include implementation, testing, and final checkup TODO classes.
- Execute one TODO at a time; keep commits atomic.
- Call out blockers fast with safest fallback options.

## Verification Gate

- Run repo-relevant checks before claiming done.
- Do not mark complete without concrete validation evidence.
- If checks cannot run, state exactly what was skipped and why.

## Safety Guardrails

- Never run destructive actions without explicit user approval.
- Never discard unrelated local changes.
- Preserve unmanaged user files and local customizations.

## Documentation Gate

- Update documentation in same change where behavior changes.
- Remove stale guidance instead of layering conflicting rules.
- Prefer linking canonical policy location rather than duplicating policy text.
