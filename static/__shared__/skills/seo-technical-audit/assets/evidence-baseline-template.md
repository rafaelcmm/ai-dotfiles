# SEO evidence baseline — measured YYYY-MM-DD

Every number below was captured live against `<origin>` or read out of this
repository on **YYYY-MM-DD**.

> **Do not re-derive these facts.** They are the shared premise for every phase.
> If a fact here no longer matches production, stop, append to the **Drift log**
> at the bottom, and only then adjust the affected phase.

---

## A. How this evidence was captured

| Source | Command / tool | Notes |
| --- | --- | --- |
| Live HTML, headers | `curl -s --compressed -D -` | No JS. This is what a raw crawler sees. |
| Lab performance | `npx lighthouse --form-factor=mobile --screenEmulation.mobile --throttling-method=simulate` | Record the browser binary and version. |
| Sitemap | `<sitemap url>` | N `<loc>` entries. |
| Product feed | `<feed url>` | N `<item>` entries. |
| Search Console | API / UI, property `<property>` | Record the exact property form: `sc-domain:` vs URL-prefix with trailing slash. |
| Field vitals | CrUX, **mobile** form factor | |
| Analytics | property `<id>`, last N days | |

**Instruments unavailable this pass, and what that leaves unverified:**

- `<tool>` — unavailable because `<reason>`. Leaves `<claim>` **unverified**.
  (Carry into the "Unverified" section below. Do not assert it.)

---

## B. Site shape

Sitemap (N URLs):

| Bucket | Count |
| --- | --- |
| `/<items>/*` | |
| `/<categories>/*` | |
| Static | |

**Structural read:** which share of indexable URLs belongs to which product line,
and which revenue line has little or no search surface.

URL/slug shape notes — any internal code or prefix in slugs that is **not** query
language.

---

## C. Response headers and cacheability (measured)

| URL | HTTP | `cache-control` | CDN cache | Prerendered | TTFB |
| --- | --- | --- | --- | --- | --- |
| `/` | | | | | |
| `/<category>` | | | | | |
| `/<category>/<handle>` | | | | | |
| `/<items>/<handle>` | | | | | |

Flag any page type that is unexpectedly dynamic or CDN-uncacheable, and name the
mechanism, not just the symptom.

**Security headers — record as known-good and do not change:**

---

## D. Indexing signals (measured, confirm each twice)

| URL | HTTP | `<link rel="canonical">` | `<meta name="robots">` | `<title>` |
| --- | --- | --- | --- | --- |
| `/<items>/<invalid-handle>` | | | | |
| `/<categories>/<invalid-handle>` | | | | |
| `/<unmatched-route>` | | | | |
| `/` | | | | |
| `/<category-index>` | | | | |
| `/<contact>` | | | | |
| `/<policy>` | | | | |
| `/<search>?q=…` | | | | |
| `/<category>/<handle>?page=2` | | | | |
| `/<category>/<handle>?sort=…` | | | | |

Re-confirm any soft 404 with a **cache-busting random handle** and record the age
and CDN status, so it cannot be dismissed as a stale CDN artifact.

Canonical declarations in the codebase (`grep -rn "canonical"`), and which routes
override the layout default:

robots.txt rules vs sitemap contents — list any URL appearing in both:

Heading census on the home page: `h1` × N, `h2` × N, `h3` × N.

### Search engine verdicts (URL Inspection — the only authority on these)

| URL | verdict | coverageState | `robotsTxtState` | userCanonical | **googleCanonical** |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

---

## E. Server-rendered HTML vs client-rendered content

| Page | item links in raw HTML | `<img>` in raw HTML | refs to disallowed paths |
| --- | --- | --- | --- |
| | | | |

Note which elements are server-rendered and which arrive only after hydration,
and cross-check every asset path against robots.txt.

Crawlable pagination links in raw HTML: N. Items reported by the UI vs items
linked in the document: N vs N.

---

## F. Lab performance — mobile, simulated throttling

| Page | Perf | A11y | BP | SEO | FCP | **LCP** | **TBT** | CLS | TTI |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| | | | | | | | | | |

### F.1 The LCP element, per page

```
selector:
snippet:
boundingRect:
```

LCP discovery checklist: `requestDiscoverable`, `eagerlyLoaded`, `priorityHinted`.

### F.2 Main-thread cost

| Group | ms |
| --- | --- |
| Script Evaluation | |
| Rendering | |
| Style & Layout | |

Heaviest scripts, with total vs scripting ms. Raw document size uncompressed and
gzipped — the **uncompressed** figure is what the mobile main thread parses.

### F.3 Other failing audits worth acting on

| Audit | Page(s) | Detail |
| --- | --- | --- |
| | | |

Record any **unresolved discrepancy** between a tool's verdict and the raw HTML
rather than picking the answer you prefer.

---

## G. Content inventory and duplication

| Measure | Value |
| --- | --- |
| Item pages total | |
| **Distinct** body-copy blocks across item pages | |
| **Distinct** meta descriptions across item pages | |
| Distinct titles / title formula | |
| Categories with an empty description | |

Which product classes have genuine authored copy (**do not rewrite these**) and
which share boilerplate.

---

## H. Structured data coverage

Graphs present per page type:

| Page type | Graphs emitted | Notable gaps |
| --- | --- | --- |
| Item | | |
| Category | | |
| Global | | |

Deliberate, documented omissions — **standing fences, do not "fix"**:

| Omission | Stated reason | Still true? |
| --- | --- | --- |
| | | |

---

## I. Product data feed (if any)

| Attribute | Count / N | Status |
| --- | --- | --- |
| | | |

Compare the feed's titles and descriptions against the site's own — the feed is
sometimes already better, and then it is the reference formula.

---

## J. Known-good — do not "fix" these

Anything carrying a written justification in code is a fence. List it with its
rationale so a well-meaning contributor does not revert it.

-

---

## K. Field data — search performance and analytics

Property: `<property>`. First day with any data: **YYYY-MM-DD** (age in search is
often much shorter than the domain's age — check before projecting).

| Month | Clicks | Impressions |
| --- | --- | --- |
| | | |

Growth trend (month over month multiple), and whether projections land on a rising
or flat base:

### Totals since first data

| | Clicks | Impressions | CTR | Avg position |
| --- | --- | --- | --- | --- |
| All | | | | |
| Mobile | | | | |
| Desktop | | | | |

### Queries — position vs CTR

| Query | Impressions | Clicks | Position | CTR |
| --- | --- | --- | --- | --- |
| | | | | |

A page at position 2 normally earns double-digit CTR. If positions are good and
CTR is not, **the click is being lost at the snippet** — that is a title and
description finding, not a rankings finding.

What query *shape* users actually type (names, numbers, codes, modifiers):

### Sessions by device and channel

| Device | Sessions | Share |
| --- | --- | --- |
| | | |

| Channel | Sessions |
| --- | --- |
| | |

State what share of sessions organic actually is, so any organic multiple is
applied to the right base.

---

## L. Competitive and authority position

| Domain | Crawl-graph presence | Earliest archive capture | Measured link data |
| --- | --- | --- | --- |
| us | | | |
| incumbent | | | |

Where a number does not exist, write **"no measurement"** — not a low estimate.
"Not in the crawl graph" ≠ "no authority".

Calibration — what this means for which query classes are winnable, and on what
horizon:

| Horizon | Realistic | Not realistic |
| --- | --- | --- |
| 6 months | | |
| 24 months | | |

---

## M. Corrections this baseline forces on the plan

Field data both validates and corrects a plan drafted before it arrived. List
every claim it **overturned**, with the revised severity — do not silently edit
the original claim.

1. **CORRECTION —**
2. **NEW —**

---

## N. Unverified in this pass

Carry these as open questions, not assumptions:

-

---

## Drift log

*(Append when production stops matching this file. Format: date · fact · new
value · who checked.)*
