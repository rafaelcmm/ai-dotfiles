---
name: merchant-center-operations
description: >-
  Operate a Merchant Center account as distinct from building its product feed —
  product ratings and the separate reviews feed, promotions for discount
  messaging in Shopping ads, account-level vs item-level diagnostics, and data
  source freshness. Use this skill when the user says "star ratings aren't
  showing", "product reviews feed", "set up a Merchant Center promotion", "my
  discount isn't showing in the ad", "items disapproved", "account suspended",
  "Merchant Center diagnostics", "product ratings programme", or asks why a
  Shopping ad does not reflect a sale. Load it before adding rating attributes to
  a product feed — that does nothing, and this skill explains why.
---

# Merchant Center operations

Building the feed is the `product-feed-engineering` skill. This is running the
account around it.

## Ratings never travel in the product feed

**Get this straight first — it is the most common and most expensive confusion in
this area.**

Star ratings reach Google through exactly **two** channels, and the product feed
is **neither**:

1. **A separate merchant-submitted reviews feed**, which Merchant Center consumes
   for Shopping ads and free listings.
2. **The `aggregateRating` in each product page's structured data**, which Google
   Search reads for organic rich results.

Consequences:

- Adding rating attributes to the product feed **does nothing**. They are not part
  of that schema.
- Looking for star ratings in the **product feed's** diagnostics finds nothing,
  because they were never supposed to be there.

Both mistakes cost a review cycle before anyone notices nothing changed.

## The reviews feed

### Joining reviews to offers

Join on the **same identifier the product feed publishes**, read **from the
product feed's own items** rather than resolved a second time.

If the two ever disagree, Google matches the review to a different offer than the
one it prices, and the rating silently never appears — with no error anywhere
explaining why. Reading the identifiers out of the already-built product feed
items makes disagreement structurally impossible rather than merely unlikely.

### Enrolment thresholds

- Roughly **50 reviews across the catalogue** before product ratings appear at
  all.
- **3+ reviews on a given product** before that product shows one.

Enrolling earlier wastes a review cycle. Do it when the catalogue crosses ~50,
not before.

### Keeping it alive

- Schedule the fetch **daily**; keep it uploading at least **monthly**. Google
  drops merchants whose review feed goes stale, and **the drop is silent**.
- **If the review platform's own Google syndication is ever enabled, remove your
  feed first.** Two sources for the same reviews is a duplicate-content problem,
  not redundancy. Many review platforms gate their own syndication behind a paid
  tier, which is the usual reason to ship your own feed at all — Google's
  merchant-submitted reviews feed is the documented alternative, costs nothing,
  uses data the site already renders, and keeps it under your control.

### Enrolment procedure

1. Merchant Center → **Growth → Manage programmes → Product Ratings** → opt in.
2. When asked how reviews will be submitted, choose **submit your own feed**.
3. Register the feed URL, scheduled **daily**.

## Promotions: the only discount messaging in a Shopping ad

**Shopping ads have no headline.** The ad is image + product title + price +
store name, all drawn from the feed. There is nowhere to write "20% off until
Friday".

A Merchant Center promotion is the **only** way a discount appears in the ad —
as an offer badge beside the price. It is free, and it typically lifts CTR, which
on a small budget means more clicks for the same money.

Without one, a sale campaign is an ordinary Shopping campaign with a suggestive
name, and the sale message lives only inside the site — invisible at exactly the
moment the shopper is comparing options on Google.

### Timing

Approval takes **24–48 hours**. Create the promotion on **day 1** of the window.
A promotion approved on day 8 of a 14-day sale served for six.

### The field everyone gets wrong

**"No code required."**

The interface defaults to asking for a promotional code. If a code is supplied
for a discount that applies automatically, the ad tells the shopper to use code
X; they arrive, look for where to enter it, cannot find it (the discount is
already applied), and leave believing they missed the offer.

**A promotion advertised wrongly converts worse than no promotion at all.**

### Scope

The promotion must cover **exactly** what the storefront discount covers. If
pre-order items and gift cards are excluded from the site discount, they must be
excluded here too, or the ad advertises a discount the checkout refuses.

Two ways, in order of preference:

1. **Filter by custom label** (e.g. `custom_label_3 ≠ pre-order`). Self-maintaining
   as the catalogue changes, and it reuses the segmentation the feed already
   publishes.
2. An explicit item list. Goes stale the moment a product is added, so every
   catalogue change silently narrows or widens the promotion.

### Setup fields

| Field | Note |
| --- | --- |
| Promotion id | Stable, descriptive, unique per promotion. |
| Type | Percentage off / amount off / free shipping — must match the storefront mechanic. |
| Applies to | All products, or a filtered subset (see Scope). |
| Code | **"No code required"** for an automatic discount. |
| Start / end | Inclusive window; create ≥48h before the start. |
| Destination | Shopping ads (and free listings where applicable). |

## Diagnostics: account level before item level

**Read account-level issues first.**

Policy, verification and claim problems suppress serving regardless of how clean
the product data is. Investigating items first leads to the conclusion "the feed
is broken" when the feed is fine and the account is suspended — and that wrong
conclusion costs a rebuild of something that was never wrong.

Order of investigation:

1. **Account-level issues** — policy violations, unverified or unclaimed website,
   missing business information, suspension.
2. **Data source status** — was the feed actually fetched, and did it process?
3. **Item-level issues** — only now.

## Item-level issues: group, do not scroll

Per-item status is **one record per product**. For a real catalogue that is
thousands of rows saying a handful of distinct things.

Turn it into a work list:

- **Group by issue code.**
- **Sort by affected-product count**, descending — that is the money ranking.
- **Keep a few example item ids per code** so each one is reproducible without
  re-querying.
- Note each issue's **servability** (disapproved vs demoted vs unaffected). A
  demotion and a disapproval need different urgency.

Use the `merchant` client in the `google-api-clients` skill for these reads — it
exposes `listProductStatuses`, `summarizeItemIssues` and `listAccountIssues`, and
follows pagination. A single-page read silently returns a prefix, which looks
like a shrinking catalogue rather than a truncated query.

**A product can be present, valid and still not served.** The product list does
not answer "is my feed serving" — the per-item destination statuses and
`itemLevelIssues` do.

## Data source freshness

A feed Google **stopped fetching** keeps serving its last good version until it
expires. The catalogue looks healthy right up to the moment it empties.

Check the **last-fetch outcome**, not merely that a data source exists. A source
whose last fetch failed is the state that precedes a silent catalogue-wide
outage.

This is also why the feed route must return **503 on failure rather than an empty
200** — see `product-feed-engineering`. An empty success reads as a deliberate
catalogue deletion.

## Pending operator actions

Several things here **cannot be done from a repository**. Any feed or measurement
work must end with an explicit **"pending operator actions"** section, because
otherwise the code ships, the behaviour does not change, and the gap is invisible
in the diff.

Typical contents:

- Register or re-schedule a data source.
- Opt into the Product Ratings programme.
- Create or correct a promotion.
- Claim and verify the website.
- Resolve an account-level policy issue.
- Register analytics custom dimensions (see `ga4-measurement-architecture`).
- Add the hosted checkout domain to the unwanted-referrals list.

Separate these from the code actions in the report. They have a different owner
and a different latency.

## Related skills

- `product-feed-engineering` — the feed itself: attributes, drops, caching, XML
  safety.
- `google-api-clients` — the Merchant client used for diagnostics reads.
- `paid-traffic-review` — where Shopping campaign decisions are made.
- `seo-structured-data-geo` — the page-side `aggregateRating`, the other half of
  how ratings reach Google.
