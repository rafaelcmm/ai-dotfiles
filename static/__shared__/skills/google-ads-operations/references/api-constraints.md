# Google Ads API constraints that cost real time

Each entry is a behaviour the API does not document where you will look for it,
or reports in a way that sends the investigation somewhere else.

## Contents

- [Version sunsets return HTML, not an API error](#version-sunsets-return-html-not-an-api-error)
- [Preflight one cheap read before any mutation sequence](#preflight-one-cheap-read-before-any-mutation-sequence)
- [`login-customer-id` is omitted, not blanked](#login-customer-id-is-omitted-not-blanked)
- [Money is micros; round, never truncate](#money-is-micros-round-never-truncate)
- [`search` paginates; `searchStream` does not](#search-paginates-searchstream-does-not)
- [Do not set a flag to its own default](#do-not-set-a-flag-to-its-own-default)
- [Shopping campaigns reject language criteria](#shopping-campaigns-reject-language-criteria)
- [Shopping listing groups have no in-place UNIT → SUBDIVISION conversion](#shopping-listing-groups-have-no-in-place-unit-subdivision-conversion)
- [Conversion goals can be configured to count nothing](#conversion-goals-can-be-configured-to-count-nothing)
- [Criterion creation is not idempotent](#criterion-creation-is-not-idempotent)
- [Developer token access levels](#developer-token-access-levels)

## Version sunsets return HTML, not an API error

Google sunsets API versions on a rolling cadence of roughly one year. A sunset
version does not degrade and does not return a structured error — the request
falls through to Google's generic web front end and comes back as an **HTML 404**.

Consequences:

- Pin the version explicitly. Never resolve "latest" at runtime.
- Prefer the newest version you have verified against a real account: sunset is
  the observed failure mode, so the newest version has the most remaining life.
- Detect the sunset case (`status === 404 && body.includes("<!DOCTYPE html>")`)
  and translate it into the single action that fixes it — set the version
  override and re-run. Printing the HTML tells the operator nothing.
- Expose the version as an environment override so the fix needs no code change.

## Preflight one cheap read before any mutation sequence

Before the first write, run a trivial query (`SELECT customer.id FROM customer
LIMIT 1`). It proves three things at once: the API version exists, the developer
token is accepted, and the account is reachable.

Without it, a bad version fails on the *first write*. That is only harmless when
the first write is the first operation — a version rejected partway through a
multi-step build strands a half-created campaign that then has to be found and
cleaned up by hand.

## `login-customer-id` is omitted, not blanked

Required only when the target account sits under a manager (MCC) account.

A **blank** header value is an error in its own right, not an omission. Build the
header conditionally:

```js
...(loginCustomerId ? { "login-customer-id": loginCustomerId } : {})
```

A falsy environment variable that becomes an empty header is a common and
confusing failure, because the error does not mention the header.

## Money is micros; round, never truncate

All monetary fields are integers in millionths of the account currency unit.

- The API rejects a non-integer, so the conversion must round.
- Truncation silently under-bids by up to one micro on every call.
- Sub-unit bids are truncated by the API rather than rejected, so rounding has to
  happen in your plan for the printed numbers to match what the account receives.

```js
const toMicros = (amount) => Math.round(amount * 1_000_000);
```

## `search` paginates; `searchStream` does not

The non-streaming endpoint returns one page and a `nextPageToken`. A
report-shaped query against it **silently returns a prefix** of your own data
unless you follow the token — and a truncated report still looks plausible,
which is worse than an error.

Use `searchStream` for anything report-shaped and `search` only for bounded
lookups where you know the row count is small.

## Do not set a flag to its own default

Several campaign fields are rejected on *create* with
`OPERATION_NOT_PERMITTED_FOR_CONTEXT` even when the value you are sending is
already the default the API would apply.

The lesson generalizes: send the fields you are deliberately changing. A field
set to the value it would already have is pure risk — it can be rejected in one
context, accepted in another, and never changes behaviour either way.

## Shopping campaigns reject language criteria

Shopping campaigns accept `LOCATION` criteria only. Adding a language criterion
fails the mutate.

Search campaigns take both. If a build routine is shared between campaign types,
the criteria block has to branch — this is a frequent source of "the same code
works for one campaign and not the other".

## Shopping listing groups have no in-place UNIT → SUBDIVISION conversion

A listing-group node cannot be converted from a unit to a subdivision. To
subdivide an existing node you **remove it and recreate the subtree in one atomic
mutate**. Doing it in two mutates leaves the campaign serving nothing in between.

Two further rules:

- **Every subdivision needs an "everything else" node.** Google rejects a
  subdivision without one.
- **That catch-all must be negative** (excluded). A *positive* catch-all is
  accepted by the API and silently opens the entire feed to that campaign's
  budget. This is the single most expensive silent defect in Shopping structure —
  write a validator for it.

## Conversion goals can be configured to count nothing

Two independent settings can each make the Conversions column empty while every
event arrives correctly:

- A campaign-level or account-level goal with `biddable: false`.
- No conversion action with `include_in_conversions_metric: true`.

With either in place, nothing reaches the Conversions column and **no bidding
strategy can optimise**, because there is no signal to optimise against. Smart
Bidding on such an account is optimising for an empty set.

This is invisible unless specifically queried — the UI shows the conversion
actions existing and receiving data. Query `conversion_action` and the campaign's
goal configuration as part of any account audit, and never enable a
value-based bidding strategy before confirming both.

Related trap: turning a goal on *before* the conversion pipeline is proven makes
the strategy optimise toward an empty or fractional signal. Fix the measurement
first, then enable the goal.

## Criterion creation is not idempotent

Google rejects a duplicate criterion. A bulk negative-keyword run that fails
partway cannot be re-run as-is: the already-created entries must be removed from
the plan first.

This is the main argument for keeping `partialFailure: false` — an all-or-nothing
failure is re-runnable, a partial one is not.

## Developer token access levels

A developer token at *test account* access level works only against test
accounts. Against a production account the errors are unrelated to the actual
cause. Apply for Basic access in the manager account's API Center before
debugging anything else.
