# Live Account Mutations

This rule is mandatory for every request that changes state in a live external account — anything that spends money, alters what customers see, or reconfigures a third-party platform.

Sync note: this rule must stay semantically identical across Claude rules, Copilot instructions, and Cursor rules; update all three files in the same change.

## Normative Language

- MUST = mandatory rule.
- SHOULD = expected default; deviations require explicit justification.
- MAY = optional.
- When rules conflict, apply stricter rule.

## Why this is separate from code safety

Code-change safety assumes the blast radius is the repository and that git is the undo. Neither holds here. A budget raise spends real money within minutes, a feed change can mass-disable a catalogue's listings, a conversion-action edit silently invalidates weeks of bidding data, and none of it is in version control. Reverting the commit does not revert the account.

## Activation

Activate when the request would create, edit, pause, or delete any of:

- advertising campaigns, ad groups, ads, budgets, bids, bidding strategies, audiences, keywords, negative keywords, or product/listing groups
- analytics configuration: key events, conversion actions, custom dimensions, data streams, referral exclusions, attribution settings
- product data feeds, feed registrations, data sources, or promotions
- merchant, marketplace, or storefront settings that affect what shoppers see
- DNS, email sending domains, payment configuration, or webhook registrations
- any third-party platform record reached through an API or admin panel

Also activate when the request is ambiguous about whether it writes. Treat "update the campaign" as a write until proven otherwise.

## Required sequence

1. **Assume read-only tooling cannot write, and verify what can.** Many official MCP servers and integrations are read-only by design. Do not assume a tool mutates; do not assume it does not.
2. **Produce a written plan before touching anything.** For each proposed change: the current value, the proposed value, the evidence that justifies it, the expected effect, and the risk if it is wrong.
3. **Encode the plan as reviewable data, not imperative code.** A declarative plan module with a validator can be unit-tested and diffed; a script that computes its changes inline cannot be reviewed before it runs.
4. **Dry-run first, server-side where the API supports it.** A dry run MUST exercise the real API with a validate-only flag when one exists, so the platform type-checks every field. A local simulation is not a dry run and proves only that the script runs.
5. **Show the dry-run output to the user.** Every line carries its reason. A diff without reasons gets rubber-stamped.
6. **Stop and request approval. State plainly that nothing has been applied.** Do not proceed on an implied yes.
7. **Apply only the approved plan.** No opportunistic extra changes discovered mid-run.
8. **Verify against the live account with a fresh read**, not the script's own output.
9. **Commit the plan change** so the account's history is readable in version control.

## Mandatory gates

Before applying, the agent MUST be able to state:

1. **Idempotency gate.** What happens if this runs twice. Absolute-value writes are usually no-ops; creations usually are not. If a partial run is possible, how it is resumed.
2. **Blast-radius gate.** What else keys on the value being changed. Published interfaces — feed labels, conversion action names, campaign ids, custom dimension parameter names — are depended on by configuration that lives outside the repo, and changing one orphans the dependent silently.
3. **Reversibility gate.** How the change is undone, and whether undo is actually possible. Some platform operations are irreversible (deleting a custom dimension, removing an item's accumulated history). An irreversible change MUST be called out as such before approval, not after.
4. **Attribution gate.** Whether this change can be evaluated. Changing several independent variables in one pass makes the outcome unattributable; when that is unavoidable, say so and say what is being given up.

If any gate cannot be answered, the agent MUST NOT apply the change.

## Defaults

- Scripts that mutate MUST be dry-run by default and write only behind an explicit flag.
- Partial-failure modes SHOULD be disabled. A half-applied batch is harder to reason about and resume than a clean failure with nothing written.
- A superseded mutation script MUST refuse to run with a message naming what replaced it, rather than being deleted. Two scripts addressing the same records silently revert each other.
- Credentials MUST come from the environment or a credentials file, never from command-line arguments.
- Account, property, and customer identifiers SHOULD be explicit arguments rather than ambient environment values, so a stale shell cannot steer a write at the wrong account.

## Red lines

Never, without explicit per-instance approval:

- raise or reallocate budgets
- pause, delete, or remove campaigns, conversion actions, or data sources
- change a bidding strategy
- edit anything whose current value has not been read first
- mutate an account while a measurement blocker is live and unacknowledged — optimising against a signal known to be broken makes the account worse, not better

## Required Evidence In Final Agent Response

Agent MUST report:

- What was changed, with before and after values.
- The dry-run output and the approval it was given.
- The post-apply verification read and its result.
- The idempotency, blast-radius, reversibility, and attribution answers.
- What was deliberately not changed, and why.

Claims without this evidence are non-compliant.
