---
name: paid-traffic-review
description: >-
  Recurring decision-grade review of a paid search / shopping account against
  its analytics property, producing an approved, actionable change plan.
  Use this skill when asked to "review the ads", "do the weekly ads review",
  "analyse my ads and analytics", "check the campaigns", "how are my campaigns
  doing", "what should I change in Google Ads", "which products should I exclude",
  "should I raise the budget", "optimise my paid traffic", or when any ads
  tuning, product exclusion or budget change is requested. Load it BEFORE
  reading any performance data — the order of operations is the method.
---

# Paid traffic review

Produce a review someone can act on, then stop and wait. **Never mutate the
account before a written plan is approved.**

Judgement lives in this file on purpose. Thresholds in markdown can be argued
with; thresholds buried in a script get obeyed. When you change one, say that
you changed it and why.

## 1. Pull one snapshot

Run a collector that **judges nothing**: it issues every query the review
reasons over, prints a human summary, and writes the full raw result to a
timestamped JSON file.

One snapshot, one timestamp. A review assembled from queries run at different
times compares windows that do not line up, and the discrepancy surfaces later
as an unexplainable number.

Read the **JSON**, not only the printed summary. The summary is a convenience;
the snapshot has the per-item rows, the full search-term list and the
per-campaign negative counts that most findings come from.

Queries to include: `references/gaql-recipes.md` in the `google-ads-operations`
skill. Analytics side: sessions and conversions by source/medium, plus the paid
subset, over the same windows.

Window shape: a recent window, the adjacent prior window of **equal length** so
week-over-week deltas are like-for-like, and a longer 30-day window so per-item
rows have enough volume to be worth reading.

## 2. Clear the blockers before analysing anything

These are **not** findings to list alongside others. Each one changes what the
rest of the data is allowed to mean. Check them first, every time.

**Blocker A — the purchase conversion action recorded nothing.**
If the primary purchase action shows zero conversions over 60 days, **no campaign
can be judged on ROAS**. Say that explicitly in the report. Name the proxies you
are using instead — click-through rate, cart-add volume, cart-add value — and
state that they are proxies. Never substitute a proxy silently and present it
where revenue belongs.

**Blocker B — paid clicks are not arriving as analytics sessions.**
Compare paid clicks against paid sessions over the same window. Normal loss is
10–30% (redirect drop-off, consent, tagging gaps). **Below ~60% capture means
spend is not reaching the site**, or tagging is dropping it. Until that clears,
every per-campaign conversion rate is measuring the measurement.

Only meaningful above a minimum click volume — below ~50 clicks in the window
the ratio is noise.

**Blocker C — purchases losing their source.**
Conversions landing in `(direct)` despite arriving from a session with a real
source means attribution is broken between the order and the analytics property.
This is the fixable half and it outranks optimisation: a campaign cannot be
credited for revenue the property never attributes to it.

**The discipline that goes with Blocker C:** some orders are *structurally*
unattributable — a marketplace or social-commerce channel whose checkout never
touches your site and arrives server-side with no client identifier. Those are
**not a defect**:

- Never count them in a paid ROAS. They inflate it with revenue no ad produced.
- Never open a task to "fix attribution" for them. That task can never close.
- Label them at ingestion so they are separable, and say in the report how many
  there were and that they are excluded by design.

Distinguishing "broken attribution" from "a channel with no web session at all"
is the difference between a week of work and a one-line exclusion.

When a blocker is live, state plainly that fixing it outranks every optimisation
in the report, and keep recommending it every review until it clears.

## 3. Thresholds

Every number below is a **calibratable default**, not a law. Each carries the
derivation and the scale it assumes — small-budget accounts in the order of tens
of currency units per day per campaign, where per-item volumes are thin. On a
account spending an order of magnitude more, the impression minimums rise and the
windows shorten.

### Excluding a product or segment

Exclude only when **all three** hold:

- Click-through rate at least **30% below the campaign's own average** — not
  below some absolute number. The campaign average is the only fair comparison
  because it already embeds the vertical, the device mix and the ad format.
- **Zero cart-adds** in the window. A cart-add is the earliest real signal of
  intent; something producing them is working even with a weak rate.
- **Enough impressions for the rate to mean anything**: ≥200 in a 30-day window,
  ≥300 if the campaign is younger than two weeks (young campaigns have unstable
  delivery, so the same count carries less information).

Items merely below average are **noise** at these budgets. Leave them. Excluding
the bottom half of a catalogue every week is activity, not optimisation.

**The exception: real impressions and zero clicks.** Exclude regardless of the
margin rule, and flag it as a **probable broken listing** — bad image, wrong
price, misleading title — not a bidding problem. The ad was shown and ignored.
Raising its bid shows it to more people who will also ignore it.

### Raising bids

Only for items with **actual conversion signal**. Cap the increase at about
**+40% per review**.

In a budget-capped campaign a bid rise buys impression share by giving up click
volume — it is a trade, not an improvement, and it is only worth making when the
evidence is real. The cap exists so one review cannot move a bid far enough that
the next review cannot read the result.

### Raising budgets

Require **both**:

- **≥50% of impression share lost to budget**, not to rank. Lost-to-rank share
  means the ads are not winning auctions they already enter; more budget buys
  nothing there.
- **No live blocker from step 2.** Raising spend on an account that cannot
  measure conversions buys unmeasurable traffic.

### The one-variable rule

**Never change exclusions, bids and budgets in the same review.** Three
variables at once makes the outcome unattributable — the next review cannot tell
which change produced the result, so the information is lost permanently.

When you decline a change for this reason, say so. "Correct, but not this week,
because it would make the budget change unreadable" is a complete answer.

### Margin and what a click is worth

**Gross profit per order decides what a click is worth, not margin percentage.**
A high-margin, low-ticket item can be worth less per order than a low-margin,
high-ticket one.

```
breakeven CVR = CPC / gross profit per order
```

Compare that against the observed conversion rate. If breakeven CVR exceeds
anything the account has ever achieved, the bid is wrong regardless of how the
campaign is performing relative to itself.

**An assumed margin is an assumption, not a measurement.** Ask for real unit
costs rather than treating a placeholder as evidence. State the assumed figure in
the report every time you use it, so nobody downstream mistakes it for a
measured one.

### Negative keywords

For Shopping campaigns, negatives are the **only** lever — there are no keywords
to bid on. A live campaign with **zero negatives** is the cheapest available win.
Check the per-campaign negative count every single review.

Governance, once the list is non-trivial:

- One **shared** list plus one list **per niche or campaign theme**.
- **A niche negative must never be promoted to the shared list.** The same term
  is waste in one campaign and the core search term in another. Promoting it
  silently blinds the campaign that depends on it.
- **A term naming something you actually sell must never become a negative**,
  however much waste it appears to carry. Check the catalogue before adding.
- Watch for the "wants the instructions, not the product" category — searches for
  files, templates, tutorials, DIY guides, or how-to content around what you
  sell. These are high-volume, low-cost-per-click, and convert at zero. They are
  usually the single largest waste bucket for a maker or own-manufacture
  catalogue.

### Thin baselines

Early results on an account with few historical conversions are **priors, not
measurements**. A campaign launched against a catalogue with a handful of sales
in 90 days has no established baseline; its first weeks cannot confirm or refute
the bids that were set from assumptions. Say so rather than reading the noise.

## 4. Report, then stop

Deliver the written report **before touching the account**. Use
[assets/report-template.md](assets/report-template.md).

Required contents:

- Blockers first, with the consequence spelled out.
- What changed week over week, **with numbers**.
- Findings ranked by **money at stake**, not by how interesting they are.
- For each proposed change: the evidence, the expected effect, the risk.
- What you are deliberately **not** changing, and why.
- **One clear recommendation**, not a menu. A menu moves the decision back onto
  someone who asked you to make it.
- An explicit statement that **nothing has been applied**.

Then stop and ask for approval.

## 5. Apply, then verify

Apply only approved changes, through plan scripts — see the
`google-ads-operations` skill. Dry run first, show its output, then `--apply`.

After applying, **query the live account again** to confirm the state. The
script's success output reports that the API accepted the write, not that the
account is as intended.

Commit the plan change so the account's history is readable in git. The platform's
own change history records what changed and never why.

## Related skills

- `google-ads-operations` — how to apply changes safely.
- `google-api-access` / `google-api-clients` — reading the data.
- `ga4-measurement-architecture` — fixing Blockers B and C at the source.
- `product-feed-engineering` — what Shopping campaigns actually advertise.
