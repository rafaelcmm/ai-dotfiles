# Documentation Completion Step — full requirements

The gate itself is stated in `code-change-workflow.md`. This file holds the
enumerations: what must be documented, the quality caveats, the delegation rules,
the documentation skills to apply, and the final-checkup checks.

Read it when running the Documentation Completion Step or a final-checkup
documentation review.

Code changes are not complete until documentation is complete.

For every code-change request, run a mandatory Documentation Completion Step as the final pre-commit gate.

Documentation Completion Step (required):

- Every new or modified function must be documented.
- Every new or modified structure, type, class, interface, and enum must be documented.
- Every new or modified non-trivial or complex code path must include intent-focused comments.
- Public APIs must include contract-level documentation (parameters, returns, errors, side effects).
- Internal logic must include maintenance-facing documentation where behavior is non-obvious.
- Documentation must be updated in the same change set as the code.

Documentation quality caveats (required):

- Documentation is mandatory, but low-value comments are not acceptable.
- Comments must explain intent, invariants, constraints, side effects, failure modes, and non-obvious decisions.
- Comments must not restate obvious syntax, variable names, or trivial control flow.
- Comments must describe the current state of the code. In-code documentation is not a change log: no provenance, removed behavior, dated claims, supersession notices, or plan archaeology. Version control owns history.
- Coverage is absolute but depth is proportional. A self-evident signature is fully documented by one assertive sentence; padding a docblock to look thorough is a defect because it buries the line that mattered.

Commit restriction:

- The final commit is blocked until the Documentation Completion Step passes.

Specialist and skills requirement:

- Documentation verification and remediation must be delegated to `documentation-specialist`.
- Security verification and remediation must be delegated to `security-reviewer` during Final Checkup TODO execution.
- Do not invoke `documentation-specialist` or `security-reviewer` during each implementation TODO unless the user explicitly overrides this workflow.
- Before closing Final Checkup TODOs or creating the final commit, the workflow must consult and apply all documentation skills below on relevant changed files:
  - `commenting-standards`
  - `documentation-best-practices`
  - `go-documentation-best-practices`
  - `javascript-documentation-best-practices`
  - `python-documentation-best-practices`
  - `rust-documentation-best-practices`
  - `typescript-documentation-best-practices`

Final Checkup TODOs must include these documentation checks:

- Run `documentation-specialist` review on all changed code.
- Verify every changed function has documentation appropriate to visibility and complexity.
- Verify every changed structure, type, class, interface, and enum has documentation.
- Verify complex logic comments capture intent, constraints, edge cases, and side effects.
- Verify API docs match current behavior (parameters, returns, errors, and examples where relevant).
- Verify stale or contradictory comments were corrected or removed.
- Verify no comment narrates change history, and that any found in touched code was deleted.
- Verify docblock depth is proportional and free of hedging, self-reference, and type restatement.
