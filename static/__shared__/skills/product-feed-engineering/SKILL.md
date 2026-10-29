---
name: product-feed-engineering
description: >-
  Build and operate a product data feed for Google Merchant Center (or any
  comparable shopping channel) — attribute contract, availability precedence,
  stable ids, sale pricing, custom labels, caching and failure policy, and XML
  safety. Use this skill when the user says "build a Google Shopping feed",
  "Merchant Center feed", "product feed XML", "my items are disapproved", "sale
  price isn't showing", "items missing from Merchant Center", "google_product_category",
  "custom_label", "availability_date", "preorder feed", "feed returns empty", or
  any task that generates or changes the product data a shopping channel ingests.
  Load it BEFORE writing feed code — several attribute mistakes validate cleanly
  and fail silently, and one of them risks the account.
---

# Product feed engineering

A supplied feed replaces the channel's own crawl of your page markup. The
uploaded version wins and the crawled duplicate is removed automatically, so the
two can overlap safely during cutover — there is no flag day.

Three defects an uncontrolled crawl reliably produces, and which a feed fixes:

| Symptom | Cause |
| --- | --- |
| Products missing from the channel | A required attribute could not be resolved at crawl time. Images are the usual one: artwork fetched from a third-party CDN at render time gives a crawler no reliable `image_link`, and `image_link` is required. |
| Strange product names | The crawler harvested the wrong element. A variant option (a condition grade, a size) surfaces as the product name. |
| Wrong, unstable ids | Without a supplied `id` the channel generates its own, and they churn between crawls — so nothing downstream is stably keyed to a variant. |

## Architecture that makes the rules testable

One module per attribute group, one assembler, one serializer, and **exactly one
module that performs I/O**:

| Module | Responsibility |
| --- | --- |
| `feed.types` | The attribute contract. No logic. |
| `feed.identity` | `id`, `item_group_id`, `brand`, `gtin`, `mpn`, `identifier_exists` |
| `feed.text` | `title`, `description` |
| `feed.category` | `google_product_category`, `product_type` |
| `feed.offer` | `price`, `sale_price`, `condition`, images, shipping, price band |
| `feed.availability` | `availability`, `availability_date`, and the drop rule |
| `feed.item` | Assembles items; **owns the drop decision** |
| `feed.xml` | Serialization and escaping |
| `feed.source` | The only I/O: the catalogue walk |

Everything except `feed.source` is pure, so every attribute rule is unit-testable
with no network access. That is the property worth protecting.

**Do not put a per-item third-party lookup on the feed path.** A `Promise.all`
fan-out of one request per item opens hundreds of concurrent connections on a
cold cache against an upstream that throttles. If such a lookup is genuinely
unavoidable it must be explicitly batched, and the batching must be documented
where the next reader will see it before removing it.

## Identity

**`id` must be a stable, non-editable platform identifier** — the numeric variant
id. Never the SKU (sparsely populated; one real catalogue had SKUs on 13% of
variants). Never the slug/handle (shared across variants of one product, and
editable). Channel item history and ads product reporting key on this value, so
it must not derive from anything an operator can change in an admin UI.

`item_group_id` is the product-level id, emitted on **every** item including
single-variant products — it is required for variants in many markets and costs
nothing when there is only one.

**`identifier_exists` must be derived** from whether `gtin` and `mpn` were
actually emitted. Never hardcode it. Channels suspend data sources that set it to
`no` to silence validation errors. And never fabricate a `gtin` or `mpn` — emit
`mpn` only when the SKU is genuinely present, `gtin` only when the barcode is a
valid 8/12/13/14-digit value.

## Price and sale price

`price` carries the **pre-discount** (compare-at) price. `sale_price` carries the
**current** price, and only when the item is actually on sale.

**Emitting the discounted price in both fields validates cleanly and silently
suppresses the sale annotation.** The item serves, the price is correct, and the
strike-through and "special offer" badge never appear. This is the highest-value
line in this skill: it costs nothing to get right and there is no diagnostic
anywhere that reports it.

Format with the currency code and always two decimals.

## Availability precedence

The states are **ordered, and the order is load-bearing**, because each later
state also satisfies the earlier predicate:

```
1. pre-order            (tagged for pre-order)
2. backorder / made-to-order
3. in_stock / out_of_stock split
```

A made-to-order or pre-order variant **also reports as purchasable**, so checking
the plain in-stock/out-of-stock split first mislabels it. Resolve in the order
above or the two newest states never fire.

Keep sold-out items in the feed as `out_of_stock`. Removing them discards the
item history the channel has accumulated, which cannot be rebuilt.

### `availability_date`

Required whenever `availability` is `backorder` **or** `preorder`; omitted
otherwise. Capped by the channel at 365 days out.

A date is **unusable** when it is missing, unparseable, in the past, or beyond the
cap. An item with an unusable date is **dropped from the feed entirely** rather
than published with a bad or absent date.

> **Omitting an item costs one listing. A page↔feed mismatch risks the account.**
> The cheaper failure is chosen deliberately.

Two structural consequences:

- `resolveAvailability` cannot express "drop this item", so the exclusion lives
  one level up: the item builder returns `null` and the assembler records which
  items were dropped.
- **Dropped items must be counted and logged by identifier, separately per
  reason** (missing image vs unusable date vs anything else). A shrinking feed
  must be diagnosable rather than mysterious.

All date decisions must go through the **same validator the product page's
structured data uses**, so the page and the feed cannot publish two different
days for the same item.

**A sold-out pre-order reports `out_of_stock`, not `preorder`.** `preorder` means
"you are accepting orders for this product"; once the capped allocation is fully
reserved you are not, and the landing page says reservations are closed. A feed
still claiming `preorder` there is the classic page↔feed disapproval trigger. The
item stays as an honest `out_of_stock`, which preserves its history.

## Categorisation

Look category ids up in the channel's **published taxonomy file**. Never guess — a
wrong id is accepted silently and quietly misplaces the product where nobody
shopping for it will look.

Key the resolution on whatever field is **actually populated**. In one real
catalogue 159 of 170 products had an empty platform product-type field, which
made a `productType`-driven mapping resolve almost everything to the fallback;
the working mapping was tag-driven. Measure before choosing the input.

Report items that reached the fallback (`isCategoryFallback` + a build-log
warning) so the mapping grows from real data instead of guesswork.

**How to add a category mapping:**

1. Find the id in the channel's taxonomy file. Do not guess.
2. Add it to the category constant with its **full path** in the docblock.
3. Add the matching values to the relevant tag/field set, add a branch to the
   resolver **and** the mirror condition in the fallback detector — they must
   agree or the fallback report lies.
4. Add a test case using a real catalogue value.

`product_type` is your own hierarchy and is unconstrained — a sensible default is
`{first collection title} > {product title}`.

## Custom labels are a published interface

`custom_label_0..4` are the axes ads listing groups subdivide on. **Treat the
slugs as a published API:**

- **Changing a value orphans** any listing group built on the old one, silently.
- **Adding a value** means a listing group written earlier has never seen it and
  **stops serving those items**, with no error. A group keyed on
  `custom_label_3 = in-stock` does not automatically pick up a new
  `made-to-order` value.

Every time a label gains or changes a value, the operational consequence for
existing listing groups is part of the change, not a follow-up.

Do not invent an axis to fill a free label. A segment built on a fabricated
attribute is worse than one segment fewer.

### Fit price bands to the distribution, not to round numbers

Measure the catalogue first: min, median, p75, max, and the modal price.

In one real catalogue, round bands (25 / 50 / 100 / 250) put **126 of 203 items
in a single bucket and 4 in the top one**. A bucket holding most of the catalogue
cannot be steered, and a bucket holding four items never accumulates enough
conversions for smart bidding to learn anything. Boundaries fitted to the actual
distribution instead yielded roughly even fifths.

The specific cut points are an output of the method, not a value to copy. Re-fit
when the price distribution shifts materially.

## Caching and failure policy

**On any failure, return 503 with no body.**

Never return a valid-but-empty feed. A successful fetch containing no items reads
to the channel as "the merchant deleted their catalogue" and can mass-disable
listings. A 503 makes the channel retry and keep serving the last good version.

Two caching facts that bite:

- **The effective revalidation window is the lowest `revalidate` on the route**,
  not the one declared at the top of the file. A data fetch inside the handler
  with a shorter window silently wins. Fresher than declared is harmless;
  believing the declared number is not.
- **If a CDN serves `stale-while-revalidate`, any date-valued attribute can be
  served that much older than it was generated.** A lead time computed as
  "today + N business days" must stay comfortably ahead of the stale window or a
  stale response advertises a date already in the past. Lowering a lead time
  requires revisiting the cache window — document that coupling at both ends.

## Hosting the feed

- **The feed must not live on a robots-disallowed path.** The channel's fetcher
  honours `robots.txt`, so a feed under a disallowed prefix (commonly `/api/`)
  is simply never fetched.
- Point `image_link` at a **stable canonical origin** rather than routing the
  channel's daily crawl through your own image proxy. It is an avoidable hop and
  an avoidable reliability dependency for URLs that already have a directly
  fetchable home.
- Mind the channel's minimum image dimensions (Google's 500×500 minimum becomes
  enforced 2027-01-31). Verify real asset sizes rather than assuming.
- **The absolute site URL environment variable must be the production domain.**
  Every item `link` is built from it. Set to a preview or tunnel URL, the feed
  advertises links on an unclaimed domain and every item is disapproved.

## Security: the serializer is the injection boundary

Titles, descriptions, vendors and tags are **merchant-authored free text** flowing
into an XML document a third party parses.

- Every interpolated value passes through an escape function. There must be no
  path that writes an unescaped value, and none may be added.
- **CDATA is deliberately not used.** It relocates the problem to `]]>`.
- XML-illegal control characters are removed by an **explicit code-point filter**,
  not a regex character class. A range of literal control bytes in source is
  invisible to a reviewer and gets corrupted by tooling.
- Keep the I/O module server-only. The feed may expose only data already public
  on the storefront.

## Verify against a built feed

Assert on the real output, not on unit tests alone:

```
items                 == expected variant count (and 0 unexplained drops)
unique ids            == item count
required attributes   100% for id, title, description, link, image_link,
                      price, availability, item_group_id
empty elements        0
disallowed-path URLs  0
longest title         < cap
category distribution matches the mapping you intended
label distribution    matches the segmentation you intended
```

A drop count greater than zero must be explained by the logged reasons, per
identifier. An unexplained drop is a defect even when the feed validates.

## Related skills

- `merchant-center-operations` — running the account: reviews feed, promotions,
  diagnostics. Ratings do **not** travel in this feed.
- `seo-structured-data-geo` — the page-side structured data that must agree with
  every value here.
- `google-api-clients` — reading per-item status back from the channel.
