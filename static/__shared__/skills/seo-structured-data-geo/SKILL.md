---
name: seo-structured-data-geo
description: >-
  Emit correct, safe structured data (JSON-LD) and make pages quotable by AI
  answer engines. Use this skill when asked to "add schema markup", "add
  JSON-LD", "fix rich results", "get star ratings in Google", "add Product
  schema", "show up in AI Overviews", "get cited by ChatGPT or Perplexity", "add
  an llms.txt", "GEO", "AEO", "answer engine optimisation", or when working on
  breadcrumbs, ItemList, FAQPage, aggregateRating or Organization markup. Load it
  BEFORE writing a JSON-LD serializer — the escaping requirement is a security
  boundary, not a style choice.
---

# Structured data and answer-engine visibility

Two jobs that share one source of truth: making a page **machine-readable** for
search engines, and making it **text-extractable** for answer engines. A site can
be excellent at the first and score near zero on the second — that is the common
failure, because they consume different parts of the page.

## 1. Security first: `JSON.stringify` is not safe for a `<script>` tag

**`JSON.stringify` does not escape `<`.** A value containing the literal substring
`</script>` therefore closes the `<script type="application/ld+json">` element
early — regardless of being inside a JSON string — letting upstream data break out
of the script context into HTML.

Any graph incorporating text you do not author is an injection vector: third-party
API fields, user- or merchant-authored free text, anything syndicated. This is the
standard "JSON in HTML" problem.

```js
/**
 * Serializes a JSON-LD graph for embedding in a <script type="application/ld+json">.
 *
 * Replacing every `<` with the six literal characters `\u003c` is valid inside a
 * JSON string and round-trips transparently: JSON.parse decodes that escape back
 * to `<`, so no consumer (a structured-data parser, a test asserting on the
 * parsed object) sees any difference. It changes only the on-the-wire bytes, so
 * `</script>` can never appear intact in the emitted HTML no matter what the
 * input contains.
 *
 * Note the doubled backslash in the replacement below: the JS literal
 * `"\\u003c"` IS those six characters. Writing `"\u003c"` would be the `<`
 * character itself, making the replace a silent no-op.
 */
export function safeJsonLd(value) {
  return JSON.stringify(value).replace(/</g, "\\u003c");
}
```

**The load-bearing property is that there is exactly ONE shared serializer** that
every call site uses — not a per-call-site escape. A single serializer means a
future component cannot reintroduce the bug by copy-pasting the unsafe pattern.
Enforce it: no module may call `JSON.stringify` directly into
`dangerouslySetInnerHTML` (or any raw-HTML sink), and a test or lint rule should
say so.

## 2. One validator, two consumers

Product structured data and a product data feed must resolve the same facts
through the **same validator and the same source**. Availability, price, a release
or availability date — two independent resolutions will eventually disagree, and a
page↔feed mismatch is a disapproval trigger, not a cosmetic inconsistency.

The rule: **one validator, two consumers.** When the page and the feed each need
an availability date, both call the same function. When a feed needs to *drop* an
item for an unusable date, the page's markup must not simultaneously claim that
date is fine.

Corollary: never mark up a fact the page does not render. If the markup says
something the visible page does not, the markup is the thing that is wrong.

## 3. What to emit, and what each unlocks

| Graph | Where | Unlocks / why |
| --- | --- | --- |
| `Product` + `Offer` | Item pages | Price, availability and merchant eligibility. The single highest-value graph. |
| `BreadcrumbList` | Item **and** category pages | SERP breadcrumbs; hierarchy without a nested URL scheme. Commonly present on item pages and **missing on category pages**. |
| `ItemList` | Category pages | Merchant listing carousels. |
| `Organization` + `WebSite` | Global | Entity resolution, knowledge panel, site search. |
| `AggregateRating` / `Review` | Item pages | Star ratings — among the largest CTR levers in a SERP. **Only when a real review system exists.** |
| `MerchantReturnPolicy` | `Offer` | Free-listing enhancements and return-policy trust badges. |
| `OfferShippingDetails` | `Offer` | Shipping annotations — **only where a rate is genuinely static.** |
| `FAQPage` | Hubs and guides | Rich results are heavily restricted today; the real value is that the same Q&A structure is what answer engines extract. |

### A complete `Offer` is the common gap

Audit for these specifically: `priceValidUntil`, `itemCondition`, `availability`
mapped truthfully across every state the catalogue actually has (including
made-to-order and pre-order), `gtin`/`mpn`, `hasMerchantReturnPolicy`,
`shippingDetails`. Each absence is a quiet loss of eligibility rather than an
error anyone reports.

**Identifier discipline.** Emit `gtin` only when the barcode is a valid GTIN
length — a non-GTIN barcode in that field is rejected. `mpn` is the
manufacturer's part number: an identifier, not a keyword field, so use the raw
code, never a display name. **Never fabricate either.**

### Fences worth respecting — and lifting correctly

Documented omissions are fences. Read `chestertons-fence` before removing one.

- **`aggregateRating` without a review system is structured-data spam** and a
  manual-action risk. The fix is to *build reviews*, not to emit markup. Until
  then, change nothing.
- **`shippingDetails` where shipping is carrier-calculated** has no truthful
  static value. But a **free-shipping threshold** is a static, knowable fact: for
  items at or above it, a zero rate is accurate. Emit it only for that case, keep
  omitting it otherwise, and **update the docblock to record the narrowed scope
  and why** — the next reader must not think the fence was dropped carelessly.
- **A legal-facing claim is not a markup tweak.** A return-policy category and
  especially a return-fees value are commitments about consumer rights and who
  pays return shipping. Asserting "returns not permitted" where local law grants
  an unconditional withdrawal window is an inaccurate statement about the law,
  published machine-readably, and it suppresses trust enhancements. Confirm the
  values with the owner, and make sure the published policy page says the same
  thing — markup contradicting the policy page is a suspension risk.
- **Stale `sameAs` is a fence to re-check, not delete.** "Omitted until real
  profile URLs exist" is correct when written and stops being correct the moment a
  profile exists. A live profile whose bio links back to the site is exactly the
  **bidirectional signal** entity resolution looks for, and emitting it is free.
  Read contact values from the same source the contact page uses rather than
  hardcoding a second copy; omit a field whose source is unset rather than
  emitting an empty string. Conversely, a **dead** profile left in `sameAs` is a
  wrong claim about the entity — audit it, do not accumulate it.

### `ItemList` — keep it minimal

`position` + `url` per entry is the documented minimum for a listing carousel.
Do **not** inline full `Product` objects per card: it inflates an already large
document and duplicates what the item page says authoritatively. Emit only the
items actually rendered on that page.

## 4. Machine-readable is not text-extractable

A site can emit better structured data than every competitor and still be
uncitable. Search engines' shopping graphs read the JSON-LD; **answer engines
retrieve from rendered text.** If there is not one quotable sentence on the page
that is specific to *that* item, there is nothing to cite.

Score the two separately. A useful breakdown:

| Dimension | What is measured |
| --- | --- |
| Citability | Is there an extractable, standalone, item-specific passage? |
| Structural readability | Do identity facts render as sentences, headings and tables — or as disconnected UI chips? |
| Multi-modal | Image quality, meaningful alt text, data tables |
| Authority / brand | `sameAs`, reviews, bidirectional profile links |
| Technical accessibility | Server-rendered, crawlable, fast |

The common shape of the failure: every fact that identifies the item — code,
number, variant, grade, compatibility, legality — renders as isolated chips. A
boilerplate-stripping extractor sees tokens, not a sentence. Worse, definitions
that *do* exist often render only for the **currently selected variant**, making
them client-state-dependent rather than statically indexable.

### The fix: a lead sentence plus a data table

Both matter, for different consumers. The sentence is what gets **quoted**; the
table is what answer engines privilege for **direct-answer extraction**.

```
<Name> is a <attribute> <thing> from <collection> (<code>), catalogued as
<identifier>, available in <variant> with <qualifier>. It is <status> for
<applicable contexts>.

| Attribute  | Value |
|------------|-------|
| Collection | …     |
| Identifier | …     |
| Attribute  | …     |
```

One component, every item page. **Never fabricate an attribute** — omit a row
rather than guess. A quotable sentence that is wrong is worse than no sentence.

Apply the same to category descriptions: prepend one **factual direct-answer
sentence** before any flavour copy. Flavour copy answers nothing.

## 5. Glossaries, guides and clusters

- **Build the glossary for domain terms a general-audience model does not know.**
  This is usually the cheapest, highest-value content gap: the data already exists
  in the codebase as labels, the query often has no good answer in the target
  language, and it doubles as a purchase-objection remover. Keep each answer
  self-contained, ~80–120 words, answering the question **in the first sentence**,
  and link it from wherever the term appears in the UI. Wrap the Q&A in
  `FAQPage`.
- **Topic clusters: a hub plus genuinely distinct spokes**, where each spoke
  answers a question the SERP shows is actually being asked. Build in the order
  the evidence supports, not the order that is easy — and let SERP evidence
  override a plan's internal ordering when they disagree.
- **Be blunt about what is not winnable.** Broad national commercial queries
  ("best shop for X", "where to buy X") go to the highest-authority, most-linked,
  most-reviewed source. Markup quality closes the gap on item-specific and
  definitional queries; it does not compensate for an authority deficit on
  brand-authority queries. Say which is which.

## 6. AI crawler access

Leaving AI crawlers allowed is usually correct. Blocking one protects a content
moat; a catalogue of boilerplate descriptions, third-party facts and publicly
comparison-shopped prices has none, so blocking only shrinks the surface on which
the brand can be cited. Write the reasoning down so nobody "hardens" it later.

Two distinctions worth recording:

- A **training/grounding opt-out** signal for a vendor's model is **not** the same
  as eligibility for that vendor's AI answers in search — the latter follows
  ordinary indexability.
- A vendor's **live retrieval** crawler is not its **training** crawler. The
  retrieval one is the actual citation channel. Keep both allowed.

Named per-crawler `Allow: /` groups are functionally redundant with `*`; the only
value is making intent legible to a future auditor. Low priority.

## 7. `llms.txt` — add it, do not count it as a win

**No major consumer answer engine has publicly confirmed it parses `/llms.txt`.**
Consumption today is concentrated in developer-tool and agent contexts, not
shopping-answer engines. Recommending it "because it exists" is cargo cult.

Add it anyway — it is minutes of work, has no downside, and is a free hedge if
adoption grows or a user points an agent at the URL directly. **Do not report it
as a citation-driving change.** Shape: a one-paragraph description of what the
site is, then grouped links (catalogue, institutional, optional/sitemap). Verify
every claim in it against what the site actually says, and do not list a URL that
is not yet a real page.

Skip content-licensing manifests unless there is genuinely valuable original
content to license.

## 8. Anti-patterns

- Marking up content that is **not on the page**.
- Inventing a rating, a review, or a shipping rate.
- A `FAQPage` of questions nobody asked.
- **Fabricating an attribute axis to create a segment.** A segment built on a
  fabricated value is worse than one segment fewer — it will be wrong, and
  anything built on it inherits the error.
- Emitting a field with an empty or `undefined` value. Add a test asserting no
  graph contains one; that is how invalid JSON-LD ships unnoticed.
- Two independent resolutions of the same fact (see §2).

## 9. Validation before merge

```bash
# Every JSON-LD block on the page must parse
curl -s <url> | python3 -c "import sys,re,json; [json.loads(m) for m in re.findall(r'<script type=\"application/ld\+json\">(.*?)</script>', sys.stdin.read(), re.S)]"
```

Then run the URL through the vendor's rich-results test and a schema validator.
Acceptance criteria worth stating explicitly:

- Every JSON-LD block on every page type parses as valid JSON.
- Zero rich-result errors on an item page, a category page and a hub.
- Item `description` is item-specific, not store boilerplate.
- Return-policy markup matches the published policy page.
- No `aggregateRating` anywhere until a real review system exists.
- Category pages emit `BreadcrumbList` + `ItemList`.
- Tests cover the **negative** cases: no collection → no collection name; no valid
  identifier → no identifier key; below threshold → no shipping details.

## Related skills

- `seo-technical-audit` — the audit methodology and the ordering of the work.
- `product-feed-engineering` — the feed side of the one-validator rule.
