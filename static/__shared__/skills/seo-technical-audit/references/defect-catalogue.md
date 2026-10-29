# Technical SEO defect catalogue

Every defect below was found live, simultaneously, on one technically well-built
site. They are listed because each is **invisible in code review**, cheap to fix,
and expensive to leave. For each: the symptom, the cause, why it matters, and the
instrument that proves it fixed.

## Contents

- [1. Canonical inheritance — every page claims to be the home page](#1-canonical-inheritance-every-page-claims-to-be-the-home-page)
- [2. Soft 404 — unknown handles return HTTP 200](#2-soft-404-unknown-handles-return-http-200)
- [3. A robots.txt prefix collision blocking a real page](#3-a-robotstxt-prefix-collision-blocking-a-real-page)
- [4. robots.txt and sitemap contradicting each other](#4-robotstxt-and-sitemap-contradicting-each-other)
- [5. A sitemap asserting URLs that do not render](#5-a-sitemap-asserting-urls-that-do-not-render)
- [6. Missing `h1`, and a home page optimised for the wrong catalogue](#6-missing-h1-and-a-home-page-optimised-for-the-wrong-catalogue)
- [7. Images on a robots-disallowed path, and absent from server HTML](#7-images-on-a-robots-disallowed-path-and-absent-from-server-html)
- [8. The LCP element is decorative](#8-the-lcp-element-is-decorative)
- [9. A route that is accidentally uncacheable](#9-a-route-that-is-accidentally-uncacheable)
- [10. Duplicate body copy, templated descriptions, code-first titles](#10-duplicate-body-copy-templated-descriptions-code-first-titles)
- [11. Paginated lists that are JavaScript-only](#11-paginated-lists-that-are-javascript-only)
- [12. Facet and filter URLs indexable without rules](#12-facet-and-filter-urls-indexable-without-rules)
- [13. Retired URLs with no answer](#13-retired-urls-with-no-answer)
- [14. Indexable zero-item pages linked site-wide](#14-indexable-zero-item-pages-linked-site-wide)
- [15. Mobile tap targets and contrast](#15-mobile-tap-targets-and-contrast)
- [16. Redirect chains and stray routes](#16-redirect-chains-and-stray-routes)
- [17. Faster indexing — what to use and what to avoid](#17-faster-indexing-what-to-use-and-what-to-avoid)

---

## 1. Canonical inheritance — every page claims to be the home page

**Symptom.** Non-item routes (category index, contact, search, policy pages) emit
`<link rel="canonical">` pointing at the site root.

**Cause.** A canonical set in the root layout's metadata **inherits** into every
route that does not override it. Typically only the item and category detail
routes override it, so everything else inherits the root's `canonical: "/"`.

**Why it matters.** A canonical pointing at a *different* page is an instruction
to drop this page and credit that one. The category hub — usually the site's most
important non-item page — asks to be de-indexed.

**Severity caveat, measured.** Google may **override** a wrong canonical and index
the correct URL anyway. Verify before promising a recovery: if `googleCanonical`
already differs from `userCanonical` and the page is indexed, the signal is still
wrong and still wastes consolidation, but nothing is currently lost. Do not sell
an indexation recovery that cannot happen.

**Fix.** Delete the canonical from the root layout; leave a comment explaining why
it must not come back. Add an explicit self-canonical to the home page (it no
longer inherits one) and to every other route. Routes that must not be indexed get
a `robots` directive instead of a canonical.

**Verify.**
```bash
curl -s <sitemap> | grep -oP '(?<=<loc>)[^<]+' | while read -r u; do
  c=$(curl -s --compressed "$u" | grep -oP '(?<=<link rel="canonical" href=")[^"]*' | head -1)
  [ "$c" = "$u" ] || echo "MISMATCH $u -> $c"
done
```
Expect no output. Add a test asserting the root layout's metadata has no
canonical key — this is the only reason the bug will not return.

---

## 2. Soft 404 — unknown handles return HTTP 200

**Symptom.** `/<collection>/<random>` returns **200** with a "not found" body.
An unmatched route returns a correct 404, so the defect is scoped to dynamic
segments.

**Cause, layer 1.** The metadata function resolves successfully and returns a
fallback title for a handle it could not resolve, instead of terminating the
request. The response shell commits to 200 before the page component's
not-found call runs.

**Cause, layer 2 (framework-level, and the one that surprises people).** With
streaming rendering, a 200 status is sent to signal the request succeeded, and
**the status cannot be updated once headers are sent**. A not-found call thrown
partway through a streamed render cannot retroactively change the status line.
Any handle not covered by static params goes through this path — which is exactly
the unknown-handle case.

**Why it matters.** Soft 404s get indexed and dilute the namespace, and invalid
URLs accumulate as thin duplicates.

**Fix, in order.**
1. Extract one shared resolver per route and call the not-found function from
   **both** the metadata function and the page component. Delete the fallback
   title branches — they become unreachable.
2. If the status is still 200, that is the streaming limitation. Closing the
   remaining `dynamicParams = false` where the handle set is small, closed and
   fully known at build time; it genuinely returns 404 there.
3. Where new items appear between deploys, on-demand fallback is load-bearing and
   cannot be disabled. The real fix is an **existence check before the response
   starts streaming** — in middleware/proxy, returning a 404 response or
   rewriting to a not-found route. That needs a fast, edge-reachable "does this
   handle exist" lookup; getting its caching wrong either serves false 404s for
   genuinely new items or hammers the upstream API per request. Scope it
   deliberately rather than improvising it.
4. Known mitigation: frameworks commonly inject `noindex` into the streamed HTML
   regardless of status, which prevents indexation despite the 200. Document that
   as an accepted, measured trade-off — not as "fixed".

**Do not** reach for `force-dynamic` as a fix. Measured: it did not change the
status, and it destroyed every build-time-prerendered page on the route.

**Verify.** `curl -s -o /dev/null -w '%{http_code}' <route>/zzz-$RANDOM` → `404`,
plus a test asserting the not-found path is reached from both entry points.

---

## 3. A robots.txt prefix collision blocking a real page

**Symptom.** URL Inspection reports a page as `robotsTxtState: DISALLOWED` —
a page nobody intended to block.

**Cause.** **robots.txt matching is prefix-based, not path-segment-based.** A rule
written to block an account area at `/conta` also matches `/contato`,
`/contato-nos`, and anything else sharing those characters.

**Why it matters.** It can block the site's primary trust/E-E-A-T surface — the
page holding contact channels and the profiles that entity resolution looks for —
for the site's entire life. No amount of crawling finds this; only the search
engine's own verdict does.

**Fix.** Anchor the rule: `/conta$` for the bare index plus `/conta/` for
everything beneath. Then **audit every other prefix rule in the file** for the
same collision class. Add a test asserting no `Disallow` prefix-matches any known
public route, fed from the static route list.

**Verify.** URL Inspection → Test live URL → `robotsTxtState: ALLOWED`, and
confirm the routes you *did* mean to block are still `DISALLOWED`.

---

## 4. robots.txt and sitemap contradicting each other

**Symptom.** URLs appear in both the sitemap and a `Disallow` rule. Pages that
should be excluded carry no `robots` meta tag at all.

**Why it matters.** `Disallow` + sitemap-listed is the worst available
combination: the crawler cannot fetch the page to discover a `noindex`, but is
explicitly asked to index the URL. The result is the "indexed, though blocked by
robots.txt" state — a bare URL with no snippet.

**Fix.** Pick one mechanism per URL.
- Must not be indexed → **remove from sitemap**, **remove from `Disallow`**, and
  add `robots: { index: false, follow: false }`. A crawler must be able to *fetch*
  a page to read its directive; disallow + noindex is self-defeating.
- Internal search results → `index: false, follow: true` plus a self-canonical.
  `follow: true` is deliberate: search pages are a useful crawl path to items even
  when they should not rank. Keep them crawlable; the meta directive is sufficient.

**Verify.** A test importing both the sitemap and robots modules and asserting the
sets do not intersect.

---

## 5. A sitemap asserting URLs that do not render

**Symptom.** A submitted sitemap URL returns a not-found body (often as a 200 —
see §2).

**Cause.** The sitemap emits a fixed list of handles unconditionally, without
checking that each resolves. A page whose content is optional upstream (an
unpublished policy, an empty category) therefore gets submitted anyway.

**Why it matters.** The search engine was *invited* to index a page that says the
content does not exist.

**Fix.** Filter the sitemap by whatever resolution the route itself performs, and
**reuse the same mapping the route uses** rather than duplicating it — extract it
to a shared module so the two cannot drift. Never assert a URL the route cannot
render.

**Verify.** Every sitemap URL returns 200 with real content; a test asserts the
sitemap's handle set is a subset of the resolvable set.

---

## 6. Missing `h1`, and a home page optimised for the wrong catalogue

**Symptom.** Zero `<h1>` on the home page, with many `h2`/`h3`. Separately, the
home page title and description describe a product line that is a small fraction
of the indexable URLs, while item pages correctly target the real catalogue.

**Why it matters.** The most linked page on the site is the one most likely to
rank for brand and head terms, and it is describing something the site barely
sells.

**Fix.** Exactly one real `<h1>` in the DOM — real text, not an `aria-label`, not
an image. Prefer a visible heading; use a screen-reader-only one only if the
design truly cannot accommodate it, and raise that trade-off rather than
defaulting to it. Override the home page's title and description to lead with
what the catalogue actually is. Do not renumber the existing section headings.

**Watch for double branding.** If the layout applies a title template, page-level
titles must stay unbranded — appending the brand again is a recurring bug.

**Verify.** `curl -s <url> | grep -c '<h1'` → `1`; title length within the SERP
cap including the template suffix.

---

## 7. Images on a robots-disallowed path, and absent from server HTML

**Symptom.** Grid and thumbnail images are requested from a path under a
disallowed prefix (commonly `/api/`), and the raw HTML of a category page contains
**zero** `<img>` elements — images arrive only after hydration.

**Why it matters.** Every thumbnail is blocked for the image crawler, so the
site's most intent-heavy visual surface is invisible. This is frequently a rule
the codebase already wrote down and applied in one place while missing another —
e.g. a feed correctly avoiding `/api/` for its image URLs while the on-site grids
did not.

**Fix.** Move the image endpoint out of the disallowed prefix — only the path
moves; caching, rate limiting and any SSRF allowlist stay. Add an explicit comment
in the robots module stating that image endpoints must never live under the
disallowed prefix. Server-render the grid images. Give alt text real content
(identity attributes), not just a bare name.

**Verify.** `curl -s <category-url> | grep -c '/api/'` → `0`. Then URL Inspection
→ Test live URL → View crawled page, and confirm images are listed as fetched
resources rather than blocked.

---

## 8. The LCP element is decorative

**Symptom.** The reported LCP element is an `aria-hidden`, off-screen, blurred
full-bleed backdrop belonging to a carousel slide that is not visible
(`boundingRect.left` strongly negative).

**Why it matters.** The page's largest paint is something no user waits for, so
the metric measures the wrong thing and the real content appears to be fine.

**Fix.** Stop the decorative element from being LCP-eligible, and give the genuine
hero element a preload with `fetchpriority="high"`. Check the discovery
checklist — a preload that is discoverable and eagerly loaded but **not**
priority-hinted is the common gap.

**Verify.** Re-run Lighthouse mobile; confirm the LCP element selector is now the
real content, and that CLS did not regress.

---

## 9. A route that is accidentally uncacheable

**Symptom.** One page type returns `cache-control: private, no-cache, no-store`
and a CDN `MISS`, with a much higher TTFB than its siblings, despite declaring a
revalidation period in code. The back/forward cache audit also fails with
`MainResourceHasCacheControlNoStore`.

**Cause.** Reading a dynamic request API (search params) anywhere in a route's
render tree opts the **whole route** into dynamic rendering, silently overriding
the declared revalidation and dropping it from the CDN cache.

**Why it matters.** It is usually the highest-intent landing page type on the
site, and `no-store` also disables the browser's back/forward cache.

**Fix, with a caveat worth recording.** Splitting the dynamic slice into its own
suspense-wrapped child component **does not by itself** restore CDN caching under
the classic model — any dynamic read anywhere in the tree classifies the whole
route as dynamic, regardless of suspense nesting. Partial static shell plus
streamed dynamic hole is gated behind newer opt-in caching modes that may be
incompatible with the revalidation configuration already load-bearing elsewhere.
What the split *does* deliver immediately is real HTML streaming. **If the
acceptance criterion was "returns a CDN HIT", record explicitly that it was not
met** rather than letting a future phase assume it was. Verify framework
behaviour against the installed version's own docs, not from memory.

**Verify.** `curl -I <url>` for the cache header and CDN status; record TTFB.

---

## 10. Duplicate body copy, templated descriptions, code-first titles

**Symptom.** Hundreds of item pages share one identical block of body copy.
Meta descriptions are templated from the same formula. Titles lead with an
internal code rather than the name a searcher types.

**Why it matters.** This is the **CTR collapse** pattern. One audit measured
items ranking at position 2.0–4.8 while earning **0.87% CTR** — a page at
position 2 normally earns double digits. The rankings already existed; the click
was being lost at the snippet, because the searcher's query and the title's
leading token did not match.

**Diagnostic worth running.** Compare the site's own titles with the titles in its
product data feed. One audit found the **feed's titles were already better than
the site's** — and used the feed formula as the reference for fixing the pages.

**Scope it before rewriting.** Duplication is often confined to one product class;
other classes may have genuine merchant-written copy. Measure which, and do not
rewrite the good ones.

**Fix.** Per-item facts rendered as real copy, a title formula leading with the
query tokens users actually search, and per-item meta descriptions.

**Verify.**
```bash
curl -s <sitemap> | grep -oP '(?<=<loc>)[^<]+/<items>/[^<]+' | while read -r u; do
  curl -s --compressed "$u" | grep -oP '(?<=<meta name="description" content=")[^"]*'
done | sort | uniq -d
```
Expect no output.

---

## 11. Paginated lists that are JavaScript-only

**Symptom.** A category reports hundreds of items across many pages while the
served document contains only the first page's links. The pager renders as
`<button>` elements with **no `href`**. A `?page=N` parameter is not even read by
the route, so requesting it returns byte-identical links.

**Why it matters.** The bulk of the catalogue has **no crawlable internal link
path** from any category page. Items are discoverable only via the sitemap, so
category → item link equity is effectively zero.

**Fix.** Real `<a href>` pagination links using whatever parameter the route
actually reads. Remove or implement the parameter that does nothing.

**Verify.** Count crawlable pagination links in the raw HTML — expect ≥ 1 — and
confirm a deep item is reachable by following links only.

---

## 12. Facet and filter URLs indexable without rules

**Symptom.** Refinement state is mirrored into the URL, so every filter
combination becomes a distinct crawlable URL. The failure is visible on
competitor sites as indexed pages titled with raw query parameters, or page 900
of a 1,200-page list.

**The five non-negotiable rules.** Write them into the module so a future
contributor adding a facet does not rediscover them from Coverage data after the
damage is done:

1. **Default: `noindex, follow` on every facet combination.**
2. **Promote to `index` only via a hand-curated allowlist, and only where
   inventory actually exists.**
3. **Every faceted URL self-canonicalizes to the base URL** unless it is on the
   allowlist, in which case it self-canonicalizes to itself.
4. **Sort order is never indexable.** Pagination and cursor params get the same
   conservative default — they describe a *slice*, not an allowlistable set.
5. **A test asserts a URL not on the allowlist emits `noindex`.**

An **empty allowlist is a correct state**, not an oversight. Default to `noindex`
until a specific value is verified and added — never the other way around.
Declare the facet key list **in the guardrail module**, not imported from the
filter UI: a change to what the UI offers must never silently change which URLs
are treated as indexable without a reviewed edit.

---

## 13. Retired URLs with no answer

**Symptom.** After a catalogue or taxonomy change, hundreds of indexed URLs
resolve to nothing.

**A blanket redirect to the home page is not an option** — a mass redirect of
unrelated URLs to a root is read as a soft 404 and dilutes the signals the
surviving pages carry. Give each shape the answer that is true of it:

| Shape | Answer | Why |
| --- | --- | --- |
| A renamed category whose contents still exist | **301** | The shopper wants it; it moved |
| A retired category | **301** to the category index | A category page to a category page |
| A retired item with no equivalent | **410** | Gone permanently, and never coming back |
| A deleted asset/proxy endpoint indexed in image search | **410** | Same |

**410 rather than 404 is deliberate**: 410 tells the crawler the resource is
permanently gone and it drops the URL promptly, where a 404 is re-crawled for
months. Keep the resolver a pure module so the edge layer stays a string
comparison and the rules stay unit-testable without a request.

---

## 14. Indexable zero-item pages linked site-wide

**Symptom.** Empty category pages return 200 with a real heading and zero items,
are `index, follow` and self-canonical, and are linked from the site-wide nav —
the highest-priority internal link a site can give.

**Cause.** The sitemap correctly filters them by a "has items" check; the
navigation does not.

**Fix.** Make that same check the single source of truth and apply it to the nav.
Defend at the route level too: when a resolved category has no items, return
`robots: { index: false, follow: true }`. A category that is empty today may be
stocked tomorrow, so `noindex` is right and a 404 is not — and it must become
`index` again without a code change.

---

## 15. Mobile tap targets and contrast

**Symptom.** Carousel dots as 8×8 px buttons (minimum is 24×24 px); low-contrast
grey on white, accent-coloured price text and badge text failing contrast; a
button whose visible label and `aria-label` disagree.

**Why it matters.** These are conversion defects on the majority device first and
accessibility defects second. The `label-content-name-mismatch` case also breaks
voice navigation.

**Verify.** Lighthouse `target-size`, `color-contrast` and
`label-content-name-mismatch` audits on mobile, per page type.

---

## 16. Redirect chains and stray routes

- **Apex chains.** `http://apex` → `https://apex` → `https://www.apex` is two
  hops where every other entry form is one. Collapse at the host/platform level.
  Low value — do it after the rest.
- **Stray indexed routes.** A locale-prefix artifact or similar can be indexed
  while self-canonicalizing to the **wrong host** (apex instead of `www`). Find
  what generates it, then remove it or 301 it. Then grep the codebase to confirm
  nothing else emits a canonical on the non-canonical host.
- **Localisation gaps.** A label-override map covering only some handles leaves
  others rendering in the wrong language. Add a test asserting the handle set is a
  subset of the override map's keys, so the next addition cannot silently regress.

---

## 17. Faster indexing — what to use and what to avoid

- **IndexNow** is cheap and worth wiring into an existing item-update webhook.
- **Do not** build a submission pipeline against the Google Indexing API for
  normal content pages. It is officially limited to specific content types, and
  using it for product pages is well-known bad advice. Rely on a sitemap with
  accurate `lastmod` instead.
