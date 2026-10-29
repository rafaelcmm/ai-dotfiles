---
description: "Always use the growth platform skills first when the request targets Google Ads, GA4, Merchant Center, product feeds, Search Console, or SEO work."
---

# Growth Platform Skills Rule

Sync note: this rule must stay semantically identical across Claude rules and Copilot instructions; update both files in the same change.

When the request includes paid traffic, analytics, product feed, or search visibility context, always consult the growth platform skills before generic skills.

## Activation

Activate this rule when request mentions Google Ads, GAQL, Performance Max, Shopping campaigns, Merchant Center, product feed, GTIN, custom labels, Google Analytics, GA4, Measurement Protocol, Consent Mode, gclid, conversion tracking, attribution, Search Console, GSC, SEO, structured data, JSON-LD, Core Web Vitals, canonical, sitemap, robots.txt, llms.txt, GEO, AEO, answer engines, or product reviews feeds.

## Activation Scope

Activate only when request primarily targets growth platform work — advertising, measurement, feeds, or search visibility. Do not activate when one of these terms appears in passing.

## Mandatory behavior

- Resolve scope first, then load only the most specific skill or minimal set of skills that match the request:
  - `google-api-access` — authorizing or troubleshooting access to any Google API
  - `google-api-clients` — writing code against Google Ads, GA4, Search Console, or Merchant Center
  - `google-ads-operations` — reading or mutating a Google Ads account from code
  - `paid-traffic-review` — recurring account review, diagnosis, and change proposals
  - `ga4-measurement-architecture` — analytics instrumentation, consent, attribution, server-side events
  - `product-feed-engineering` — building or fixing a product data feed
  - `merchant-center-operations` — Merchant Center account, reviews feed, promotions, diagnostics
  - `seo-technical-audit` — diagnosing and sequencing technical SEO work
  - `seo-structured-data-geo` — JSON-LD, rich results, answer-engine visibility
- Load `google-api-access` first whenever a Google credential is not already proven working. The common failures in that area are silent, and auth code written without it is usually wrong in a way that looks like a permissions problem.
- Prefer the most specific skill for the request.
- If request spans multiple growth domains, combine the relevant skills before falling back to generic cross-domain skills.
- The external `google-ads-*` skills are strategy and knowledge-base references. Use them for campaign strategy, keyword research, and audit heuristics; use `google-ads-operations` for the code that touches the account.

## Safety interaction

Any request that would mutate a live advertising account, analytics configuration, feed registration, or Merchant Center setting additionally triggers the `live-account-mutations` rule. Load both. The stricter constraint applies.

## Priority

Growth platform skill guidance overrides generic framework guidance only within growth platform decisions, and never overrides user intent, safety constraints, or higher-priority workflow contracts.
