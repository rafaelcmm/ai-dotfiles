---
name: google-ads-operations
description: >-
  Engineering discipline for scripting mutations against a live Google Ads
  account — plan/apply separation, server-validated dry runs, idempotency,
  validators that catch what the API accepts silently. Use this skill whenever
  work will create, edit, pause or delete anything in an advertising account:
  "create a campaign", "change the budgets", "add negative keywords", "raise
  bids on these products", "build a Shopping campaign", "script our Google Ads
  changes", "automate campaign updates", "why did my mutate fail", "exclude
  these products from Shopping". Load it BEFORE writing any mutation code — the
  dry-run model is non-obvious and the expensive mistakes are all silent ones.
---

# Google Ads operations

Code that writes to an advertising account spends money on every run. The rules
below exist because each one was paid for once already.

Auth setup is the `google-api-access` skill. The transport is the
`google-api-clients` skill. This skill is about what you build on top.

## The plan/apply split

Two files per change, never one.

```
<change>.plan.mjs   pure data + exported validatePlan(); no I/O, no API calls
<change>.mjs        executor: imports the plan, prints it, applies it
<change>.plan.test  unit tests over the plan and its validator
```

The plan file holds every number, name, bid, budget, keyword and structural
decision as **declarative data**. The executor holds the API calls and nothing
else.

Why this split and not a single script: a reviewer approving a spend change has
to read the change. When thresholds and structure are interleaved with
pagination and error handling, the review degrades into trusting the author.
When the plan is a table, the diff *is* the proposal. It also makes the plan
unit-testable without a network, which is what lets a validator run in CI.

Split the plan further when parts of it change for different reasons. A product
catalogue table changes when the merchant adds a product; a margin assumption
changes when real unit costs land and re-prices everything at once. Those are
different diffs and deserve different files.

## Dry run by default

**This is the most important rule in this skill.**

```bash
node --env-file=.env scripts/ads/<change>.mjs            # dry run
node --env-file=.env scripts/ads/<change>.mjs --apply    # writes
```

The dry run is **not** a local simulation. It sends the real operations with
`validateOnly: true`, so Google type-checks every field server-side and rejects
a malformed criterion, an over-length headline or an invalid resource reference
**without writing anything**. A locally-simulated dry run proves only that your
own code runs; a `validateOnly` dry run proves Google accepts the payload.

Never make `--apply` the default and never add a `--yes` that skips the dry run.
Always run the dry run first and show the operator its output before asking for
approval.

## Print the plan in reviewer terms

Every printed line carries its **why**.

```
BUDGETS
  camp-a   12.00 -> 18.00/day   +6.00   50% of impression share lost to budget
  camp-b   20.00 -> 14.00/day   -6.00   zero conversions in 30d at 20.00/day
  net daily change: +0.00
```

A diff without reasons gets rubber-stamped. Print the net effect on daily spend
explicitly — a reviewer should never have to add the column up themselves to
discover a change doubles the account's burn.

## State idempotency per mutation class

Write it in the file's docblock, because the answer differs by operation and the
operator needs it the moment a run fails halfway.

| Mutation | Re-runnable? | Why |
| --- | --- | --- |
| Budget amount, bid amount, status | Yes | Absolute values. A second run is a no-op. |
| Criterion creation (negatives, keywords, audiences) | **No** | The API rejects duplicates. A partial run must be finished by **removing the already-created entries from the plan**, not by re-running it whole. |
| Resource creation (campaign, ad group, ad) | **No** | Creates a second copy with a new id. |
| Listing-group trees | **No** | Replace-in-place only; see `references/api-constraints.md`. |

## partialFailure stays off

Send `partialFailure: false` unless there is a specific reason not to. A batch
that half-applied leaves the account in a state that is neither the old one nor
the intended one, and reconstructing which half landed costs more than re-running
a clean failure. An outright rejection with nothing written is the cheaper
outcome.

## Build new structure PAUSED

Anything created by a script starts `PAUSED`. A human unpauses after reading the
dry run and the plan. A script that creates a serving campaign has spent money
before anyone reviewed its arithmetic.

## Validators catch what the API accepts silently

The API rejects malformed requests. It does not reject *wrong* ones. The
validator's job is the second category — and these are exactly the defects that
are invisible in the UI without cross-referencing several screens.

Generalizable checks worth writing every time:

- **Positive catch-all in a Shopping listing tree.** A subdivision missing its
  "everything else" unit is rejected by Google. A subdivision whose catch-all is
  *positive* rather than negative is **accepted** — and silently opens the whole
  feed to a budget sized for a handful of products.
- **An item advertised by two campaigns.** The account bids against itself. Not
  an API error.
- **An item advertised by none.** Paid for in the feed, never served. Also not an
  API error.
- **Derived arithmetic.** Any bid or budget computed from a formula rather than
  typed in gets a test, because the formula is the thing deciding whether a sale
  is profitable.
- **Platform limits that truncate instead of failing.** Asset text lengths,
  minimum asset counts, sub-unit rounding on money. A sub-cent bid is truncated,
  not rejected, so the rounding has to happen in the plan for the printed
  numbers to be honest.

Assert the committed plan validates clean as the first test, so a rule tightened
later cannot silently invalidate what already shipped.

## Cost-aware throttling for bulk writes

A multi-hundred-operation run must degrade into *waiting*, not into a wall of
throttle errors the caller has to distinguish from real failures.

Where the API reports its own capacity (remaining budget, restore rate), read it
from the previous response and sleep for the deficit before the next call, plus a
small slack so a rounding error does not land you straight back in the same
state. Where it does not, bound concurrency and back off exponentially on
`RESOURCE_EXHAUSTED`. Retry once on a network failure; do not retry a validation
error, which will fail identically forever.

## Retire scripts by guarding them, not deleting them

A superseded script **throws on start** with a message naming what replaced it
and why:

```js
if (!process.argv.includes("--i-know-this-is-superseded")) {
  throw new Error(
    "SUPERSEDED by rebalance.mjs, which re-sized the whole account and reversed " +
    "the decision this script encodes. Both write the same budget ids, so running " +
    "this again silently reverts it."
  );
}
```

Two scripts addressing the same budget ids revert each other with no error
anywhere. Deleting the old one instead loses the data inside it — negative lists
and structural decisions that are often documented nowhere else and are exactly
what a rebuild-from-scratch would need.

## Verify against the live account

After applying, query the account again and check the result. Do **not** trust
the script's own success output: it reports that the API accepted the write, not
that the account is in the intended state. A mutate can succeed and still leave a
listing tree that serves nothing.

Then commit the plan change on its own branch so the account's history is
readable in git. The plan file becomes the account's changelog — the Ads change
history UI will not tell you *why*.

## Checklist before `--apply`

- [ ] Dry run executed with `validateOnly: true` and its output shown.
- [ ] `validatePlan()` returns no problems; plan tests pass.
- [ ] Idempotency posture stated for every mutation in the run.
- [ ] Net daily spend change computed and read out loud.
- [ ] New structure is `PAUSED`.
- [ ] A human approved the printed plan.
- [ ] Post-apply verification query written before the apply, not after.

## References

- [references/api-constraints.md](references/api-constraints.md) — API behaviours
  that cost real time: version sunsets, Shopping-specific rejections, listing
  trees, conversion-goal configuration traps.
- [references/gaql-recipes.md](references/gaql-recipes.md) — queries for the
  reads these scripts and reviews depend on.

## Related skills

- `google-api-access` — authorizing the credential.
- `google-api-clients` — the REST transports.
- `paid-traffic-review` — the recurring review that decides *what* to change.
- `product-feed-engineering` — the feed that Shopping campaigns advertise.
