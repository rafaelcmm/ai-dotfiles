---
name: ga4-measurement-architecture
description: >-
  Design and debug end-to-end GA4 measurement for a storefront whose checkout is
  off-domain — server-side purchase and refund events via Measurement Protocol,
  session stitching, Consent Mode v2, ad click id capture, marketplace channel
  labelling, and custom dimension registration. Use this skill when the user says
  "purchases are showing as direct", "conversions aren't reaching Google Ads",
  "set up GA4 for a headless store", "server-side purchase event", "Measurement
  Protocol", "session_id", "gclid", "consent mode", "my refunds aren't reversing
  revenue", "GA4 custom dimension not showing", "attribution is broken", or any
  task that connects a hosted/off-domain checkout back to the session that
  produced it. Load it BEFORE writing analytics code — every failure in this area
  is silent.
---

# GA4 measurement architecture

A hosted checkout on another domain cannot be instrumented from the page. The
`purchase` event therefore has to leave from the **server**, off an order
webhook, through the GA4 Measurement Protocol (MP). Everything hard about this
area follows from that one constraint.

Every failure mode below is silent: the event is accepted, the revenue is
correct, and only the attribution is wrong. Nothing errors. That is why this
skill exists and why it must be read before writing the code rather than after.

## The headline failure: `client_id` is not enough

An MP event carries **no cookie**. `client_id` identifies the *user*, not the
*session*. With `client_id` alone, GA4 opens a **new** session — and a new
session has no traffic source, so the purchase lands in `(direct)` and never
reaches Google Ads through the analytics conversion import.

`session_id` is the field that joins the event to the existing session. It is
the single most important parameter on a server-side purchase.

The symptom appears **only in the ads account**: `add_to_cart` and
`begin_checkout` arrive normally (they are browser events), while `purchase`
sits at zero. In GA4 itself the revenue looks right, which is what lets this
survive undetected for months.

## The attribution bridge

```
browser                          server action             platform          webhook
───────                          ─────────────             ────────          ───────
read client_id  ─┐
read session_id ─┼─► initiateCheckout ─► cart attributes ─► order ─► orders/create
read ad click id ┘                       (order attributes)        │
                                                                    ▼
                                                      MP purchase event
                                                      client_id + session_id
```

1. **Read the identifiers before redirecting.** After the redirect there is no
   second chance — the browser is on a domain you do not control.
2. **Read them concurrently under one short deadline** (~1500 ms total). Serial
   reads double the wait in front of someone who has already clicked buy.
3. **Attach as cart attributes.** The commerce platform copies them onto the
   order. Prefix a key with `_` to hide it from customer-facing order surfaces.
   Do **not** prefix the ad click id: its whole value is that the merchant can
   open an order in the admin UI and see that it came from an ad, and a key that
   might be hidden from the reader it exists for defeats its only purpose.
4. **The webhook verifies its signature, recovers the identifiers, and sends the
   event.**

## Trust boundaries

Every value on this path is untrusted at **every hop**, not just the first. A
server-action argument is an ordinary HTTP body. Cart attributes are writable by
anyone holding the cart token. The values end up inside the JSON body and query
string of an outbound request to Google.

Revalidate on **exit**, not only on entry:

| Value | Shape |
| --- | --- |
| `client_id` | two integers separated by a dot |
| `session_id` | a bare integer |
| ad click id | URL-safe base64, length-capped (~200 chars) |

An invalid value is **dropped, never thrown**. Losing one order's attribution
always beats breaking a checkout, or making a webhook respond non-200 and enter
the platform's retry loop.

## MP transport rules

- **Retry only when the request provably never reached Google.** `fetch`
  *throwing* means the exchange did not complete (DNS failure, reset socket,
  truncated TLS handshake) so Google cannot have processed the event and a
  resend cannot duplicate it. `fetch` *resolving* means the server saw the
  payload whatever the status says — return it as-is, even a 5xx. Retrying a
  resolved response can record the same purchase twice.
- GA4 does deduplicate purchases by `transaction_id`, but that is an undocumented
  backstop. Inflating revenue is worse than losing one event, so correctness must
  not lean on it.
- Send `engagement_time_msec: 1`. A server event has no measurable engagement,
  but **omitting** it makes GA4 treat the session as non-engaged, which
  suppresses the event from parts of the ecommerce funnel rather than merely
  leaving it unlabelled. `1` asserts "engaged, duration unknown" — the closest
  true statement available.
- The API secret is server-only. Guard the module so a build fails if it is ever
  bundled into a client chunk.

## Ad click ids (`gclid` / `gbraid` / `wbraid`)

`gbraid` and `wbraid` are the iOS-privacy counterparts of `gclid`, sent when App
Tracking Transparency prevents a user-level id (`gbraid` app-to-web, `wbraid`
web-to-web). Treat all three as one value with `gclid` taking precedence.

- **Capture on every route change, not just on mount.** Shopping campaigns land
  on product and category pages, not the home page, and the parameter is gone
  after the next navigation.
- Set gtag's `url_passthrough` so the id survives client-side navigations as a
  fallback — a fallback, not the mechanism.
- Store it for the same window as the account's conversion lookback (90 days by
  default).
- Last click overwrites the previous one, matching how Google Ads itself credits.
- **Gate it on the ad-storage consent signal consistently with whatever default
  the tag applies.** A code path that denies storage while the tag grants it
  drops the click id on nearly every paid visit — the two must agree.

Capturing the id is worth it even with nothing uploading it to Google Ads: it is
ground truth independent of the session join, the consent state and the
conversion import all holding. Uploading conversions against it as a *second*
path double-counts unless exactly one conversion action is primary — keep the
option open, do not take it by accident.

## Consent Mode v2

Four signals: `analytics_storage`, `ad_storage`, `ad_user_data`,
`ad_personalization`. All four must be declared in **both** `default` and
`update`.

- A signal missing from `default` is treated as **unset, not denied**.
- A signal missing from `update` keeps whatever `default` gave it — so an opt-out
  that does not actually opt out.
- Omitting any `ad_*` signal makes Google treat the property as having **no valid
  advertising consent state at all**, which suppresses conversion attribution
  even for users who never objected.

**One module must be the single source of truth** for which signals exist and
what values they take. Two call sites — the bootstrap default and the banner
update — that disagree break measurement silently. Declaring a signal in only
one of them is the exact failure this centralisation prevents.

Set `wait_for_update` (~500 ms) so a stored refusal can overtake the defaults
before the first hit leaves the browser. Under a granting default this window is
what protects the shopper: the banner restores a stored refusal in a mount
effect, and until that runs the defaults are granting.

Keep `ads_data_redaction` enabled — it strips ad identifiers from requests that
still go out under denied consent, which is what makes a refusal
privacy-meaningful rather than cosmetic while preserving Google's modelled
conversions for that traffic.

### Opt-in vs opt-out posture

**This is a jurisdiction-dependent legal decision. Decide it with counsel.** What
this skill supplies is the cost of each option, which is usually missing from the
conversation:

- Under an **opt-in** default (everything denied until the banner is answered),
  one measured deployment lost roughly **92% of its measurement** — 260 paid
  clicks produced 21 attributed `google / cpc` sessions over 13 days — because
  every conversion action on the account was imported from GA4. An unanswered
  banner is the common case, not the edge case.
- An **opt-out** default is defensible in some regimes and not others. Brazil's
  LGPD Art. 7º, IX permits processing under legitimate interest without prior
  consent, and first-party audience measurement of one's own storefront is the
  textbook case. ePrivacy/GDPR regimes generally require prior consent for the
  same storage.

Whichever posture is chosen, a **real one-click refusal** must be enforced at
three points, or it is cosmetic:

1. block writing the identifier,
2. block reading what is already stored,
3. **erase** what is already on the device.

## Refunds

Without a `refund` event every refund stays counted as revenue forever.

- GA4 reverses a purchase only when the refund carries the **same
  `transaction_id`**. Beware a merchant-configured order-name prefix: if orders
  are named `#ABC1027` while the purchase's `transaction_id` was the raw order
  number `1027`, deriving one from the other by stripping the `#` yields
  `ABC1027` and makes **every** refund fail to match, with no error anywhere.
  Use the platform's numeric order number field, not its display name.
- **Date the event by the refund, not the order.** An order from months ago
  refunded today would fall outside GA4's ~72-hour backdating window and be
  dropped entirely. An out-of-range timestamp is worse than none: omitting it
  makes GA4 use receipt time and keep the event.
- **Take the value from the transactions list, not the line items.** Only
  transactions include refunded shipping and tax, and a shipping-only refund has
  no line items at all — summing line subtotals reports it as zero.
- Count only successful refund transactions. `pending` and `failure` mean no
  money has moved.
- A restock-only refund with no money returned emits nothing.
- For a partial refund, use the refund line's own quantity, not the original
  order line's — record what came back, not what was bought.

## Marketplace and non-web channels

Orders from a marketplace channel sync into the commerce platform and reach the
**same order webhook** as a web order, so they go out as ordinary `purchase`
events with nothing to distinguish them. Revenue, conversion rate and any ROAS
computed on top silently include sales that never touched the site.

Because they have no `client_id`, they land as sessionless purchases in
`(direct)` — an unexplained `(not set)` acquisition group is very often exactly
this.

Keep sending them (they are real revenue) but **label** them with an
`order_source` parameter mapped from the platform's source field. Surface an
unrecognised channel under its **raw handle** rather than collapsing it to
`unknown`: an unknown label is a prompt to extend the map, a swallowed one is
invisible.

**These orders are structurally unattributable, not broken.** Never count them in
an Ads ROAS, and never open a task to "fix" their attribution — that task can
never close.

## Custom dimension registration

An event parameter is accepted, stored, and appears in **no report** until it is
registered as a custom dimension (GA4 Admin → Custom definitions, or the Admin
API — see the `google-api-clients` skill).

**Scope is load-bearing.** An item-scoped parameter registered as event-scoped
produces a dimension that exists and is always empty. This is the most common
reason a correctly-emitted parameter looks broken.

Registration is an operator action on a panel, not a code change. It belongs in
the "pending operator actions" section of any measurement work.

## Referral exclusion

The hosted checkout's own domain appears as a `referral` when the shopper
returns, re-attributing the session away from its real source. Add it to the
data stream's unwanted-referrals list. Another panel action.

## Verification ladder

Fastest first, because the last step has a 24–48 hour lag:

1. **GA4 realtime report** with a test purchase — the purchasing session must
   keep its source and not become `(direct)`.
2. **Traffic acquisition**, by session source/medium — purchases should stop
   concentrating in the unattributed bucket.
3. **The Google Ads conversion action** should leave zero. This lags 24–48 hours
   on GA4 import latency; do not conclude failure before then.

Also confirm, before trusting any bidding strategy: the conversion goal is
biddable and at least one conversion action has
`include_in_conversions_metric` set. Otherwise nothing reaches the Conversions
column and no strategy can optimise for it — invisible unless specifically
queried.

## Enhanced conversions: evaluate honestly

Sending hashed email/phone in MP's `user_data` depends on the `ad_user_data`
signal. Under an opt-out posture the blocked group is exactly the group whose
attribution the technique would recover — it is blocked on both sides. Weigh
that against the PII path you then have to maintain, plus the panel toggle it
requires. At low volume it does not pay for itself. Revisit at materially higher
volume.

## Related skills

- `webhook-ingestion-hardening` — signature verification and retry posture for
  the order webhook this depends on.
- `google-api-clients` — GA4 Data and Admin API transports.
- `paid-traffic-review` — how these measurement blockers gate any ads analysis.
