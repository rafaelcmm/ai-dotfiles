# Product feed attribute checklist

A pre-launch and pre-merge gate. For each attribute: the rule, and the **silent
failure** if it is wrong. "Silent" means the feed validates and serves — nothing
reports the problem.

Work top to bottom. Identity and offer errors are the expensive ones.

---

## Identity

| Attribute | Rule | Silent failure if wrong |
| --- | --- | --- |
| `id` | Stable, non-editable platform identifier (numeric variant id). | Item history and ads product reporting are keyed to a value an operator can change in the admin UI. A title edit resets the item's history. |
| `id` (anti-pattern) | **Never** the SKU. | Sparsely populated in most catalogues — items with no SKU get no id and vanish. |
| `id` (anti-pattern) | **Never** the slug/handle. | Shared across variants of one product → duplicate ids, and only one variant serves. |
| `item_group_id` | Product-level id. Emit on **every** item, including single-variant products. | Variants are not grouped; required in several markets, so items are disapproved there and serve elsewhere — a partial, confusing outage. |
| `brand` | Platform vendor field, capped at the channel's length limit. | Truncation mid-word, or a missing brand on a category where the channel requires it. |
| `gtin` | Only when the barcode is a valid 8/12/13/14-digit value. Never fabricated. | A fabricated or malformed GTIN matches someone else's product, or triggers a data-quality suspension. |
| `mpn` | The SKU when genuinely non-blank, capped. Never fabricated. | A duplicate `mpn` across distinct products reads as the same product listed twice. |
| `identifier_exists` | **Derived**: `no` only when both `gtin` and `mpn` were actually omitted. | Hardcoding it to silence validation errors is grounds for data-source suspension. |

- [ ] `id` derives only from an immutable platform identifier
- [ ] No item carries a fabricated `gtin` or `mpn`
- [ ] `identifier_exists` is computed, with a test proving both branches
- [ ] `item_group_id` present on 100% of items

---

## Text

| Attribute | Rule | Silent failure if wrong |
| --- | --- | --- |
| `title` | Must *describe* the landing page's product. Truncate on a word boundary, no ellipsis. Enrichment (variant option, line/series, grade) is within spec; the title need not match the `<h1>` byte for byte. | A title led by an internal code, or whose only variant dimension surfaces as the name, is unsearchable and reads as a different product than the page. |
| `description` | Platform description with whitespace collapsed. Below a floor (~40 chars), substitute composed catalogue facts rather than shipping a near-empty field. | An empty-ish description is accepted and the item simply ranks nowhere. |

- [ ] Longest title measured against the cap on a real build
- [ ] Truncation happens on a word boundary
- [ ] A short-description fallback exists and is tested

---

## Offer

| Attribute | Rule | Silent failure if wrong |
| --- | --- | --- |
| `price` | The **pre-discount** (compare-at) price when on sale, else the current price. Currency code, two decimals. | — |
| `sale_price` | The **current** price, only when actually on sale. | **Emitting the discounted price in both fields validates and silently suppresses the sale annotation.** The strike-through and offer badge never appear. No diagnostic reports this. |
| `condition` | Derived from real catalogue state. Omit rather than guess. | A guessed condition on second-hand or graded stock is a policy problem, not a data one. |
| `image_link` | Variant image → product featured image. **None → drop the item and count it.** | A missing required attribute drops the item anyway; doing it yourself makes it countable and diagnosable. |
| `additional_image_link` | Up to the channel limit, excluding the primary. | — |
| `shipping` | Only when it can be stated correctly. | Wrong shipping annotations are a page↔feed mismatch. |

- [ ] A sale item emits two **different** values in `price` and `sale_price`
- [ ] A non-sale item emits no `sale_price` at all
- [ ] Items dropped for a missing image are counted and logged by identifier
- [ ] `price` always carries the currency code and exactly two decimals

---

## Availability

| Attribute | Rule | Silent failure if wrong |
| --- | --- | --- |
| `availability` | Resolution order is **pre-order → backorder/made-to-order → in_stock/out_of_stock split**. | Each later state also satisfies the earlier predicate. Checking the plain split first means the two newest states never fire. |
| `availability` (sold-out) | Sold-out items stay in the feed as `out_of_stock`. | Removing them discards accumulated item history, which cannot be rebuilt. |
| `availability` (sold-out pre-order) | A fully-reserved pre-order reports `out_of_stock`, **not** `preorder`. | `preorder` means "accepting orders". Claiming it while the page says reservations are closed is the classic page↔feed disapproval trigger. |
| `availability_date` | Required with `backorder` and `preorder`, omitted otherwise. Capped at 365 days out. | Missing it alongside `preorder` disapproves the item. |
| `availability_date` (unusable) | Missing, unparseable, past, or beyond the cap → **drop the item**. | Omitting an item costs one listing; a mismatch risks the account. Choose the cheaper failure deliberately. |
| `availability_date` (source) | Same validator the product page's structured data uses. | Page and feed publishing two different dates is a mismatch the channel detects and you do not. |

- [ ] One internal classifier backs `availability`, `availability_date` and the drop decision, so they cannot disagree
- [ ] The drop decision lives in the item assembler, not the attribute resolver
- [ ] Each drop reason is counted separately and logged by identifier
- [ ] A computed lead time stays ahead of the CDN stale-while-revalidate window

---

## Categorisation

| Attribute | Rule | Silent failure if wrong |
| --- | --- | --- |
| `google_product_category` | Id copied from the channel's published taxonomy file. Never guessed. | A wrong id is accepted silently and misplaces the product where nobody shopping for it looks. |
| resolution input | Key on a field that is **actually populated** — measure first. | A `productType`-driven mapping against a catalogue with an empty `productType` resolves everything to the fallback. |
| fallback reporting | Items reaching the fallback are reported in the build log. | The mapping never improves because nobody knows which items need it. |
| `product_type` | Your own hierarchy; unconstrained. | — |

- [ ] Every category id was copied from the taxonomy file, with its full path in a docblock
- [ ] The resolver and the fallback detector agree (mirror conditions tested)
- [ ] Fallback items are logged by identifier

---

## Campaign labels

| Attribute | Rule | Silent failure if wrong |
| --- | --- | --- |
| `custom_label_0..4` | **A published interface.** Slugs are stable. | Changing a value orphans every ads listing group built on the old one, silently. |
| new label value | Adding a value means existing listing groups have never seen it. | Those items **stop serving** with no error. A group keyed on `in-stock` does not pick up a new `made-to-order` value. |
| price band | Cut points fitted to the measured distribution, not round numbers. | Round bands pile most of the catalogue into one unsteerable bucket and leave a top bucket too small to accumulate conversions. |
| unused label | Leave unemitted rather than inventing an axis. | A segment built on a fabricated attribute is worse than one segment fewer. |

- [ ] Every label change lists its consequence for existing listing groups
- [ ] Price bands were fitted against measured min/median/p75/max
- [ ] Label distribution verified on a real build

---

## Route, hosting and safety

- [ ] Feed path is **not** robots-disallowed (the channel's fetcher honours `robots.txt`)
- [ ] On any failure the route returns **503 with no body** — never a valid-but-empty feed
- [ ] Absolute site URL env var is the **production** domain in production
- [ ] `image_link` points at a stable canonical origin, not an in-house proxy
- [ ] Image dimensions meet the channel minimum, verified on real assets
- [ ] Every interpolated value passes through the escape function; no unescaped path exists
- [ ] CDATA is not used
- [ ] XML-illegal control characters removed by explicit code-point filter
- [ ] The I/O module is server-only

---

## Final build assertion

```
items                 == expected variant count, drops all explained
unique ids            == item count
required attributes   100% (id, title, description, link, image_link,
                      price, availability, item_group_id)
empty elements        0
disallowed-path URLs  0
longest title         < cap
```
