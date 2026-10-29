# GAQL recipes

Queries for the reads that operational scripts and recurring reviews depend on.
Each one states the decision it informs — a query whose output changes no
decision does not belong in an audit.

Conventions used below:

- `CUSTOMER_ID` — 10-digit account id, no hyphens. Passed per invocation.
- `START_DATE` / `END_DATE` — `YYYY-MM-DD`, inclusive.
- Use `searchStream` for everything here except the single-row account lookup;
  `search` paginates and silently returns a prefix.
- Monetary metrics come back in micros. Divide by 1,000,000.

## Contents

- [Account currency and timezone](#account-currency-and-timezone)
- [Campaign performance with impression-share diagnostics](#campaign-performance-with-impression-share-diagnostics)
- [Conversion-action health over a 60-day window](#conversion-action-health-over-a-60-day-window)
- [Campaign-level goal configuration](#campaign-level-goal-configuration)
- [Search terms](#search-terms)
- [Negative keyword counts per campaign](#negative-keyword-counts-per-campaign)
- [Shopping product performance](#shopping-product-performance)
- [Listing group structure](#listing-group-structure)
- [Ad group and keyword bids](#ad-group-and-keyword-bids)
- [Asset / RSA inventory](#asset-rsa-inventory)
- [Change history](#change-history)

## Account currency and timezone

Informs: how to format and interpret every other number here. Run this first.

```sql
SELECT
  customer.id,
  customer.descriptive_name,
  customer.currency_code,
  customer.time_zone,
  customer.manager,
  customer.test_account
FROM customer
```

A `test_account: true` explains an otherwise inexplicable absence of data. The
timezone determines what "yesterday" means — reporting dates are in the
account's timezone, not yours.

## Campaign performance with impression-share diagnostics

Informs: the budget decision. The two lost-impression-share metrics are what
separate "demand exists and the budget is capping it" from "the ads are not
competitive".

```sql
SELECT
  campaign.id,
  campaign.name,
  campaign.status,
  campaign.advertising_channel_type,
  campaign.bidding_strategy_type,
  campaign_budget.amount_micros,
  metrics.impressions,
  metrics.clicks,
  metrics.ctr,
  metrics.average_cpc,
  metrics.cost_micros,
  metrics.conversions,
  metrics.conversions_value,
  metrics.search_budget_lost_impression_share,
  metrics.search_rank_lost_impression_share,
  metrics.search_impression_share
FROM campaign
WHERE segments.date BETWEEN 'START_DATE' AND 'END_DATE'
  AND campaign.status != 'REMOVED'
ORDER BY metrics.cost_micros DESC
```

Read it this way: budget-lost share high and rank-lost share low means the
budget is the constraint. The reverse means raising the budget buys nothing — the
ads are not winning the auctions they already enter.

The `search_*` impression-share metrics are unavailable on Shopping campaigns in
some configurations; query Shopping budget pressure through
`metrics.search_budget_lost_impression_share` where present and fall back to
comparing daily spend against the daily budget.

## Conversion-action health over a 60-day window

Informs: **whether any ROAS-based judgement is defensible at all.** Run this
before reading any performance data.

```sql
SELECT
  conversion_action.id,
  conversion_action.name,
  conversion_action.category,
  conversion_action.type,
  conversion_action.status,
  conversion_action.primary_for_goal,
  conversion_action.include_in_conversions_metric,
  conversion_action.click_through_lookback_window_days,
  metrics.all_conversions,
  metrics.all_conversions_value
FROM conversion_action
WHERE segments.date DURING LAST_30_DAYS
```

Three failure states to detect explicitly:

- A `PURCHASE`-category action with `primary_for_goal: true` and zero
  conversions → nothing can be judged on ROAS.
- Every action with `include_in_conversions_metric: false` → the Conversions
  column is empty by configuration, and no bidding strategy can optimise.
- A long lookback window relative to the reporting window → conversions are
  still arriving for clicks inside the window, so recent numbers understate.

Widen to 60 days by replacing the `DURING` clause with an explicit
`BETWEEN 'START_DATE' AND 'END_DATE'` range. A 30-day window on a low-volume
account can be empty by chance; 60 days distinguishes "quiet" from "broken".

## Campaign-level goal configuration

Informs: whether a goal is actually biddable. Pairs with the query above —
either half being misconfigured produces the same empty Conversions column.

```sql
SELECT
  campaign.id,
  campaign.name,
  campaign.selective_optimization.conversion_actions,
  campaign.bidding_strategy_type
FROM campaign
WHERE campaign.status = 'ENABLED'
```

## Search terms

Informs: which negatives to add. For Shopping campaigns negatives are the
**only** lever, so this is the highest-value query in a Shopping review.

```sql
SELECT
  search_term_view.search_term,
  search_term_view.status,
  campaign.name,
  ad_group.name,
  metrics.impressions,
  metrics.clicks,
  metrics.ctr,
  metrics.cost_micros,
  metrics.conversions
FROM search_term_view
WHERE segments.date BETWEEN 'START_DATE' AND 'END_DATE'
  AND metrics.impressions > 0
ORDER BY metrics.cost_micros DESC
LIMIT 500
```

Terms with cost and zero conversions are candidates. Terms with impressions and
zero clicks are a different signal — the ad is being shown and ignored, which
points at the listing or the offer rather than the term.

## Negative keyword counts per campaign

Informs: the cheapest available win. A live campaign with **zero** negatives is
almost always leaking budget. Check this every review.

```sql
SELECT
  campaign.id,
  campaign.name,
  campaign_criterion.keyword.text,
  campaign_criterion.keyword.match_type,
  campaign_criterion.negative
FROM campaign_criterion
WHERE campaign_criterion.type = 'KEYWORD'
  AND campaign_criterion.negative = TRUE
  AND campaign.status != 'REMOVED'
```

Count per campaign on the client side and compare against the set of campaigns
with impressions. Shared negative lists attached to the campaign are a separate
resource — query `shared_set` and `campaign_shared_set` to see those, otherwise a
campaign using only a shared list reads as having zero negatives.

```sql
SELECT
  campaign.id,
  campaign.name,
  shared_set.id,
  shared_set.name,
  shared_set.type
FROM campaign_shared_set
WHERE campaign.status != 'REMOVED'
```

## Shopping product performance

Informs: per-item exclusion and bid decisions. `segments.product_item_id` is the
feed's own id, which is how a decision here maps back to a feed row.

```sql
SELECT
  segments.product_item_id,
  segments.product_title,
  segments.product_type_l1,
  segments.product_custom_attribute0,
  campaign.name,
  metrics.impressions,
  metrics.clicks,
  metrics.ctr,
  metrics.cost_micros,
  metrics.conversions,
  metrics.conversions_value
FROM shopping_performance_view
WHERE segments.date BETWEEN 'START_DATE' AND 'END_DATE'
ORDER BY metrics.impressions DESC
LIMIT 1000
```

Use a 30-day window at minimum: a 7-day window on a per-item breakdown produces
impression counts too small for a click-through rate to mean anything.

## Listing group structure

Informs: whether the Shopping tree is what the plan intended. The catch-all
check (`listing_group_filter.type` / negative catch-alls) cannot be done any
other way without clicking through the UI.

```sql
SELECT
  campaign.name,
  ad_group.name,
  ad_group_criterion.criterion_id,
  ad_group_criterion.listing_group.type,
  ad_group_criterion.listing_group.case_value.product_item_id.value,
  ad_group_criterion.listing_group.case_value.product_custom_attribute.value,
  ad_group_criterion.negative,
  ad_group_criterion.cpc_bid_micros,
  ad_group_criterion.status
FROM ad_group_criterion
WHERE ad_group_criterion.type = 'LISTING_GROUP'
  AND campaign.status != 'REMOVED'
```

A subdivision whose catch-all unit is **not** negative opens the whole feed to
that campaign. This query is how you detect it.

## Ad group and keyword bids

Informs: bid changes, and whether a bid in the account matches the plan that set
it.

```sql
SELECT
  campaign.name,
  ad_group.id,
  ad_group.name,
  ad_group.status,
  ad_group.cpc_bid_micros,
  metrics.impressions,
  metrics.clicks,
  metrics.cost_micros,
  metrics.conversions
FROM ad_group
WHERE segments.date BETWEEN 'START_DATE' AND 'END_DATE'
  AND ad_group.status != 'REMOVED'
```

```sql
SELECT
  campaign.name,
  ad_group.name,
  ad_group_criterion.keyword.text,
  ad_group_criterion.keyword.match_type,
  ad_group_criterion.cpc_bid_micros,
  ad_group_criterion.effective_cpc_bid_micros,
  ad_group_criterion.quality_info.quality_score,
  metrics.impressions,
  metrics.clicks,
  metrics.average_cpc,
  metrics.conversions
FROM keyword_view
WHERE segments.date BETWEEN 'START_DATE' AND 'END_DATE'
  AND ad_group_criterion.status != 'REMOVED'
ORDER BY metrics.cost_micros DESC
```

`effective_cpc_bid_micros` is what actually applies — it reflects ad-group-level
inheritance, so a criterion-level bid of zero does not mean no bid.

## Asset / RSA inventory

Informs: ad copy refreshes, and whether an ad is serving with fewer assets than
intended.

```sql
SELECT
  campaign.name,
  ad_group.name,
  ad_group_ad.ad.id,
  ad_group_ad.ad.type,
  ad_group_ad.status,
  ad_group_ad.ad_strength,
  ad_group_ad.ad.responsive_search_ad.headlines,
  ad_group_ad.ad.responsive_search_ad.descriptions,
  metrics.impressions,
  metrics.clicks,
  metrics.conversions
FROM ad_group_ad
WHERE ad_group_ad.status != 'REMOVED'
```

## Change history

Informs: what changed between two reviews, including changes made in the UI by a
human rather than by a script.

```sql
SELECT
  change_event.change_date_time,
  change_event.change_resource_type,
  change_event.resource_change_operation,
  change_event.changed_fields,
  change_event.user_email,
  campaign.name
FROM change_event
WHERE change_event.change_date_time DURING LAST_14_DAYS
ORDER BY change_event.change_date_time DESC
LIMIT 500
```

`change_event` requires an explicit date filter and caps at 30 days of history.
It is the only way to discover that an account drifted outside your scripts —
which is the usual explanation for a plan file that no longer matches reality.
