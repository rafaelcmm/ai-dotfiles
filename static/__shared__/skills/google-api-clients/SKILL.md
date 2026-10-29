---
name: google-api-clients
description: >-
  Minimal, dependency-free REST transports for Google Ads, GA4 (Data and Admin),
  Search Console and Merchant Center, sharing one Application Default
  Credentials login. Use this skill when writing code that reads or mutates any
  of those APIs: "query Google Ads with GAQL", "pull a GA4 report", "register a
  GA4 custom dimension", "get Search Console clicks and impressions", "inspect a
  URL in Search Console", "check which products are disapproved in Merchant
  Center", "list Merchant Center promotions", or when an official SDK feels too
  heavy for a handful of calls. Load it before writing a Google API client by
  hand — these transports already encode the failure modes.
---

# Google API clients

Five small REST transports over one credential. Use them instead of the official
SDKs when the need is a handful of calls: the Google Ads SDK in particular is a
large gRPC dependency whose only use in an operational script would be a few
one-shot requests, and it would outweigh the scripts it serves.

Authentication setup is **not** covered here — see the `google-api-access`
skill. These clients assume a working `authorized_user` ADC credential.

## Files

| File | Purpose |
| --- | --- |
| `scripts/clients/google-auth.mjs` | ADC → bearer token. The only module that reads credentials. Also a CLI: `node google-auth.mjs` prints a token. |
| `scripts/clients/google-ads.mjs` | GAQL search, streaming search, mutate with server-side validation, micros helpers |
| `scripts/clients/ga4-data.mjs` | `runReport`, `runRealtimeReport`, property metadata |
| `scripts/clients/ga4-admin.mjs` | Custom dimensions, key events, data streams, account discovery |
| `scripts/clients/search-console.mjs` | Search Analytics (auto-paged), sitemaps, URL inspection |
| `scripts/clients/merchant.mjs` | Product statuses, issue summaries, account issues, data sources, promotions |

Copy the ones a project needs into the repo rather than importing them from the
skill directory — they are a starting shape, and a project that owns them can
pin its own API versions and extend them.

## One credential, deliberately

Every client calls `authHeaders()` from `google-auth.mjs`. That is the point, not
an implementation detail: a report correlating ad spend with sessions only means
something if both halves came from the same authorization. Duplicating the token
exchange per client would allow the two to drift onto different credentials with
different scopes.

The token is cached for the process lifetime and refreshed one minute before
expiry. A script finishing in seconds exchanges once; a long-running process does
not fail at the 60-minute mark.

## What each client encodes

**`google-ads.mjs`**

- **Version pinned, with sunset translation.** Google sunsets versions on a
  rolling ~1-year cadence and a dead version does not degrade — it is served by
  Google's generic front end as an **HTML 404**. The client detects an HTML body
  on a 404 and reports "set `GOOGLE_ADS_API_VERSION`" instead of printing HTML.
- **Preflight read before anything else.** One cheap query proves the version
  exists, the developer token is accepted and the account is reachable. Without
  it a bad version fails on the first write, which is only harmless when the
  first write is the first operation — a version rejected mid-sequence strands a
  half-built campaign.
- **`validateOnly` for dry runs.** `mutate(..., { validateOnly: true })` sends
  the real operations and lets Google type-check every field server-side without
  writing. A dry run that is a local simulation proves nothing; this one does.
- **`partialFailure` off by default.** A half-applied batch is harder to reason
  about and harder to resume than an outright failure with nothing written.
- **`login-customer-id` omitted when absent.** A blank value is an error in
  itself, so a falsy env var must not become an empty header.
- **`searchStream` for anything report-shaped.** The non-streaming endpoint
  paginates, so a wide report silently returns a prefix unless the caller follows
  `nextPageToken`. `search` is for bounded lookups only.
- **Customer id is an argument, not an env var**, so a stale shell cannot steer a
  mutation at the wrong account. It is also shape-validated (10 digits, no
  hyphens).

**`ga4-data.mjs`** — flattens the API's positional `dimensionValues` /
`metricValues` arrays back onto their names. Those arrays are only meaningful
against the request that produced them, and zipping them once here removes an
easy class of off-by-one misreads where a metric is charted under a neighbouring
metric's label. Rejects a `G-XXXXXXX` measurement id, which the Data API answers
with a generic not-found. `getMetadata()` lists what the property actually
accepts, which is the fastest way to confirm a custom dimension is registered.

**`ga4-admin.mjs`** — split from the Data client because the Admin API **rejects
`analytics.readonly`** and requires `analytics.edit`. That split is not cosmetic:
a credential authorized for reporting only reads reports happily and fails every
Admin call, so keeping both in one module invites the conclusion that the
credential is broken rather than narrow. A 403 here carries that hint.
`ensureCustomDimension` and `ensureKeyEvent` are idempotent — GA4 caps custom
dimensions per property and deleting one is irreversible, so a duplicate consumes
a slot that cannot be reclaimed.

**`search-console.mjs`** — auto-pages Search Analytics. The API caps a response
at 25,000 rows and gives no total, so a caller that requests one page and stops
receives a silent prefix of its own data, which is worse than an error because
truncated numbers still look plausible. Paging stops on a short page, the only
available signal. `inspectUrl` is the single programmatic source of truth for
which canonical **Google** chose — a self-canonical in your own HTML proves
nothing. Compare `googleCanonical` against `userCanonical` to catch
canonical-inheritance bugs. Inspection quota is tight (~2,000/day), so sample
per template rather than sweeping a catalogue.

**`merchant.mjs`** — `listProductStatuses`, not the product list, is what answers
"is my feed actually serving": a product can be present, valid and still not
served, and `itemLevelIssues` is where that is reported. `summarizeItemIssues`
groups thousands of per-product records by issue code with a few example ids,
turning a scroll into a work list. **Read `listAccountIssues` first** — policy,
verification and claim problems suppress serving regardless of product data
quality, so investigating items first concludes "the feed is broken" when the
feed is fine and the account is suspended. Every list endpoint is paged, for the
same silent-prefix reason as above.

## Property and account identifiers

Each one has a form that is easy to get wrong and is reported as a 403 rather
than an explanation:

| API | Identifier | Trap |
| --- | --- | --- |
| Google Ads | 10-digit customer id | No hyphens. Manager id goes in `login-customer-id`, not here. |
| GA4 | numeric property id | Not the `G-XXXXXXX` measurement id. |
| Search Console | property string | `sc-domain:example.com` for a domain property; the full origin **with trailing slash** for a URL-prefix property. Different properties, different data. |
| Merchant Center | account id | Under an advanced (multi-client) account, the **sub-account** id. The parent returns its own, usually empty, list rather than an error. |

`search-console.mjs`'s `listSites()` and `merchant.mjs`'s `authInfo()` are
no-argument reads that return the exact strings the credential can use — call
them rather than guessing.

## Error handling convention

Every client surfaces the API's own response body on failure, not a bare status
code. These APIs return highly specific field-level diagnostics — which criterion
was malformed, which dimension combination is invalid, which product failed — and
that detail is the difference between a fixable and an inscrutable failure. The
only bodies deliberately **not** forwarded are from the token endpoint, whose
error response can echo request parameters.

Where a status has one overwhelmingly likely cause, a hint is appended: a 403
from the Admin API names the readonly/edit scope confusion; a 403 from Search
Console lists the three things to check; an HTML 404 from Ads names the sunset.

## Credentials never come from argv

Read from the environment or the credential file only. An argument is visible in
shell history and in any process listing.

## Related skills

- `google-api-access` — authorizing the credential these clients consume.
- `google-ads-operations` — the plan/apply architecture to wrap mutations in.
- `paid-traffic-review` — the recurring review these transports feed.
- `product-feed-engineering`, `merchant-center-operations` — what the Merchant
  reads are for.
