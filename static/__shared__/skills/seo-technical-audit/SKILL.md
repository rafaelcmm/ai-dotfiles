---
name: seo-technical-audit
description: >-
  Run a technical SEO audit and turn it into an ordered, evidence-backed
  execution plan. Use this skill when asked to "audit SEO", "why isn't my site
  ranking", "fix our indexing", "improve Core Web Vitals", "we're invisible in
  Google", "review our technical SEO", "our pages aren't indexed", "organic
  traffic is flat", or when planning any multi-phase SEO programme on an
  existing site. Load it BEFORE proposing fixes — the ordering of the work is
  what determines whether any of it pays off, and the common defects are all
  invisible without the right instrument.
---

# Technical SEO audit

An audit is not a list of findings. It is a **measured baseline**, an **ordering**,
and an **acceptance criterion per change**. A findings list without those three is
a set of opinions that cannot be verified later.

The method below came out of auditing a technically well-built storefront that
was nearly invisible in search. The engineering was good; the marketing surface
was broken. That combination is the common case, and it is why "the code is fine"
is not evidence that SEO is fine.

## 1. Evidence before opinions

Produce a **dated, measured baseline** before proposing anything. Every later
phase cites it instead of re-deriving it.

- Record **how each number was obtained** — the exact command, tool, flags and
  form factor. A baseline whose provenance is lost cannot be compared to the next
  one, which makes the whole audit unrepeatable.
- Write at the top: *do not re-derive these facts*. When a fact stops matching
  production, append to a **drift log** at the bottom of the baseline rather than
  silently editing the number — the discrepancy is itself a finding.
- Separate **lab** from **field** data explicitly. Lab data tells you what to fix;
  only field data tells you whether real users on real networks got faster.

### Field data corrects the plan — say so out loud

A plan drafted before field data arrives will contain claims the data overturns.
When that happens, **state which claims were wrong and revise the severity**,
rather than quietly editing them. Two real examples worth internalising:

- A canonical bug was framed as pages "self-cancelling from the index". URL
  Inspection showed Google was **overriding** the bad canonical and indexing the
  correct URL anyway. The signal was still wrong and still worth fixing — it
  wastes consolidation and will not always be overridden — but severity dropped
  from Critical to High, and the predicted "indexed page count will rise" outcome
  had to be **dropped**, because there was nothing to recover. Promising a
  recovery that cannot happen destroys trust in the rest of the plan.
- A site assumed to be 16 months old in search turned out to have **three months**
  of data, with impressions compounding ~6.7× month over month with no SEO work
  done. Every projection then landed on a rising base, not a flat one — which
  changes what the work can be credited for.

## 2. The ordering principle

**Fix what makes existing pages cancel themselves out before improving or adding
pages.** Growing the indexable surface is pointless while the existing catalogue
reads as near-duplicates pointing at the wrong canonical.

| # | Phase | Why here |
| --- | --- | --- |
| 1 | Indexing-critical | Cheap and enormous. The difference between being in the index and not. |
| 2 | Mobile performance | Independent of everything else. Ship in parallel with 1. |
| 3 | Asset crawlability | Images and deep links must be fetchable before content work means anything. |
| 4 | Content depth and uniqueness | Where growth actually starts. |
| 5 | Structured data | Depends on the content source built in 4. |
| 6 | New surfaces | Only once the existing pages are healthy. |

Phases 1 and 2 are independent of each other and of everything else — **ship them
first**. Everything from 4 onward depends on a content source that phase 4 builds,
so attempting 5 or 6 early means building it twice.

State the sequencing rationale in the plan. A reader who disagrees with the order
should be able to argue with the reason, not guess at it.

## 3. Use the right instrument, and name what you could not measure

| Question | Instrument | What it cannot tell you |
| --- | --- | --- |
| What do we serve? | `curl -s --compressed -D -` (no JS) | What Google did with it |
| What did Google actually do? | Search Console Coverage | Why, at URL level |
| Which canonical did Google pick? | URL Inspection (`googleCanonical` vs `userCanonical`) | Anything about pages you don't inspect |
| Is this URL crawlable? | URL Inspection `robotsTxtState` | — |
| Lab performance | Lighthouse, mobile form factor, simulated throttling | Whether real devices improved |
| Field performance | CrUX / field vitals | What to change |

**A self-canonical in your own HTML proves nothing about which canonical Google
chose.** Only URL Inspection does. The same applies to crawl-blocking: a
robots.txt prefix collision that had blocked a site's main trust page since launch
was found by **Google's own verdict**, not by any amount of crawling.

When an instrument is unavailable in the environment, **write down what that
leaves unverified** and carry it as an open question. Do not assert an unmeasured
claim, and do not let a missing tool silently become a skipped check. Record it in
the baseline's "unverified" list so the next pass knows what to close.

## 4. Mobile is the acceptance criterion

When mobile is the majority of traffic, every criterion is a **mobile** criterion.
A change that improves desktop and regresses mobile is a failed change.

Set the targets per project and put the numbers in the plan. Example set from a
mobile-dominant storefront (~80% mobile sessions):

- Home: Performance ≥ 85, LCP < 2.5 s, TBT < 200 ms, CLS ≤ 0.02
- Detail and category pages: ≥ 90

Also measure **what mobile does differently**. One audit found mobile ranking
*better* than desktop (position 6.2 vs 12.5) while converting *worse* (CTR 0.76%
vs 1.18%) — which points the work at the snippet, not at rankings.

Guard what is already good. If CLS is already 0, "do not introduce layout shift
while optimising LCP" belongs in the acceptance criteria.

## 5. Write the plan so a smaller model can execute it

Each phase file names the **exact file, the exact symbols, the exact copy**, and
its own acceptance criteria. Forbid improvising field names, slugs, copy and file
boundaries — improvisation is where a cheap plan becomes an expensive one.

Include:

- A **blocking map**: which phases depend on which, and which are independent.
- **Per-section acceptance criteria**, written as commands with expected output
  wherever possible (`curl -o /dev/null -w '%{http_code}'` → `404`).
- A **commit plan**: one commit per section, conventional commits, a named branch.
- A **"known-good — do not fix these"** section. Security headers, a deliberate
  sitemap filter, a documented omission: list them with their rationale so a
  well-meaning contributor does not revert a fence. Anything carrying a written
  justification in code is a fence — read `chestertons-fence` before touching it.

## 6. Guardrails: a fix without a test comes back

Measurement that runs in CI beats measurement someone remembers to run. Every
defect the audit found gets a regression test, because **these bugs are invisible
in code review and obvious in a test**:

- The root layout's metadata exports no inherited canonical key.
- No sitemap URL matches any robots.txt `Disallow` rule (import both modules and
  cross-check).
- No `Disallow` rule prefix-matches a known public route.
- The not-found path is reached from both the metadata function and the page
  component for an unknown handle.
- A faceted URL not on the indexable allowlist emits `noindex`.
- No two pages of the same template share a meta description.
- No structured-data field is emitted with an empty or undefined value.

Add **drift monitoring** on top: snapshot canonicals, titles and robots
directives, then diff periodically. The class of bug that starts most of these
audits is a metadata change that silently alters directives site-wide. Run it
monthly and after any change to the layout, robots or sitemap modules.

## 7. Authority and clusters calibrate the plan

- **Authority analysis calibrates the TIMELINE, not the task list.** A plan that
  ignores an authority gap promises results it cannot deliver. If a domain has no
  measurable link footprint, long-tail and item-level queries are winnable in
  months while head terms are not winnable at all on the plan's horizon — say
  that plainly instead of implying otherwise. Where a number does not exist,
  write "no measurement", not a low estimate.
- **SERP-evidenced topic clusters determine the BUILD ORDER of new pages**, and
  should **supersede** a phase's own internal ordering when the two disagree. One
  audit's content phase ordered hubs by catalogue size; SERP evidence showed the
  largest opening was a newer, uncontested category — so the order changed. Build
  in the order evidence supports, not the order that is easy.
- Distinguish a **time-boxed** opening (an uncontested emerging category) from a
  permanent one. The first has a deadline.

## 8. Report honestly: repairs are not growth

Frame phases 1–3 as **repairs**, not a growth project. A site serving 200s for
missing content, declaring the wrong canonical and hiding its images behind a
robots block is not under-optimised — it is broken. Fixing that does not improve
rankings so much as make ranking possible. Growth starts at the content phase.

Give the owner horizons, including the "never":

| Horizon | Expected |
| --- | --- |
| Weeks 1–2 | Crawl-blocked pages become crawlable; soft-404 counts fall |
| Weeks 2–4 | Field vitals move toward "Good" — a real but weak ranking factor; the larger return is conversion on the majority device |
| Months 2–3 | Impressions on item- and category-level queries appear. **Position moves before traffic.** |
| Months 3–6 | New hubs and guides begin ranking. Meaningful organic traffic starts here. |
| Never | Beating a long-established incumbent on head terms. Treat head terms as internal-linking anchors and a paid budget line, never as organic rank targets. |

## Definition of done

- Every phase's acceptance criteria met and evidenced.
- Lint, format, test and build pass.
- The measured mobile targets met on each page type.
- Zero URLs emit a canonical pointing at a different page's URL.
- Regression tests added for every defect fixed.
- The baseline's "unverified" list is either closed or carried forward explicitly.

## Related skills

- `references/defect-catalogue.md` — symptom → cause → verification for the
  high-frequency defects.
- `assets/evidence-baseline-template.md` — the baseline to fill in first.
- `seo-structured-data-geo` — structured data, and being quotable by answer engines.
- `product-feed-engineering` — when a product data feed is in scope, it is often
  the highest-traffic surface the site owns.
